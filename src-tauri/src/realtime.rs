//! Shared realtime WebSocket runner for translation and transcription providers.
//! It owns bounded audio consumption, connection timeouts, graceful shutdown, transcript
//! turn bookkeeping, and reconnect backoff so provider modules only describe their wire format.

use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use base64::Engine;
use futures_util::{stream::SplitSink, stream::SplitStream, SinkExt, StreamExt};
use tauri::{AppHandle, Emitter};
use tokio::net::TcpStream;
use tokio::sync::mpsc::Receiver;
use tokio_tungstenite::tungstenite::{self, handshake::client::Request, Message};
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use tokio_util::sync::CancellationToken;

use crate::audio::AudioChunk;
use crate::types::{events, Caption, Origin, SessionState, StatusUpdate};

const CONNECT_TIMEOUT: Duration = Duration::from_secs(15);
const CLOSE_DRAIN_TIMEOUT: Duration = Duration::from_secs(4);
const INITIAL_BACKOFF: Duration = Duration::from_secs(1);
const MAX_BACKOFF: Duration = Duration::from_secs(16);
const STABLE_CONNECTION: Duration = Duration::from_secs(30);
const IDLE: Duration = Duration::from_secs(24 * 60 * 60);
/// Four queued 100 ms chunks means the five-slot producer queue was effectively full.
const STALE_AUDIO_BACKLOG: usize = 4;

type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;

#[derive(Default)]
pub struct TurnAccumulator {
    pub id: u64,
    pub source: String,
    pub translated: String,
}

impl TurnAccumulator {
    pub fn next_turn(&mut self) {
        self.id += 1;
        self.source.clear();
        self.translated.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.source.is_empty() && self.translated.is_empty()
    }
}

#[derive(Debug, Default)]
pub enum MessageControl {
    #[default]
    Continue,
    Reconnect,
    Fatal(String),
    Closed,
}

#[derive(Debug, Default)]
pub struct MessageOutcome {
    pub transcript_activity: bool,
    pub control: MessageControl,
}

impl MessageOutcome {
    pub fn activity() -> Self {
        Self {
            transcript_activity: true,
            control: MessageControl::Continue,
        }
    }

    pub fn control(control: MessageControl) -> Self {
        Self {
            transcript_activity: false,
            control,
        }
    }
}

pub trait RealtimeProtocol {
    const NAME: &'static str;

    fn origin(&self) -> Origin;
    fn connect_request(&self) -> Result<Request>;
    fn setup_json(&self) -> Result<String>;
    fn audio_json(&self, base64_pcm: String) -> Result<String>;

    /// Provider frames used to flush pending audio before the socket is closed.
    fn closing_json(&self) -> Result<Vec<String>> {
        Ok(Vec::new())
    }

    fn handle_message(
        &mut self,
        app: &AppHandle,
        text: &str,
        acc: &mut TurnAccumulator,
    ) -> MessageOutcome;

    fn finalize_after(&self) -> Option<Duration> {
        None
    }
}

enum RunEnd {
    Stopped,
    Reconnect,
    Fatal(String),
}

pub async fn run_session<P: RealtimeProtocol>(
    app: AppHandle,
    mut proto: P,
    mut audio_rx: Receiver<AudioChunk>,
    cancel: CancellationToken,
) {
    let origin = proto.origin();
    let mut backoff = INITIAL_BACKOFF;
    let mut first = true;
    let mut acc = TurnAccumulator::default();

    while !cancel.is_cancelled() {
        emit_status(
            &app,
            if first {
                SessionState::Connecting
            } else {
                SessionState::Reconnecting
            },
            None,
            origin,
        );

        let mut dropped = 0usize;
        while audio_rx.try_recv().is_ok() {
            dropped += 1;
        }
        if dropped > 0 {
            tracing::info!(
                ?origin,
                dropped,
                "dropped stale audio chunks before connect"
            );
        }

        let connected_at = Instant::now();
        match connect_and_run(&app, &mut proto, &mut audio_rx, &cancel, &mut acc).await {
            Ok(RunEnd::Stopped) => break,
            Ok(RunEnd::Fatal(message)) => {
                tracing::error!(?origin, provider = P::NAME, %message, "provider stopped the session");
                emit_status(&app, SessionState::Error, Some(message), origin);
                return;
            }
            Ok(RunEnd::Reconnect) => {
                tracing::warn!(?origin, "{} stream closed; reconnecting", P::NAME);
            }
            Err(error) => {
                if let Some(status) = fatal_handshake_rejection(&error) {
                    emit_status(
                        &app,
                        SessionState::Error,
                        Some(format!(
                            "{} rejected the connection (HTTP {status}) — check the API key and model access",
                            P::NAME
                        )),
                        origin,
                    );
                    return;
                }
                tracing::error!(?origin, "{} stream error: {error:#}", P::NAME);
                emit_status(
                    &app,
                    SessionState::Reconnecting,
                    Some(error.to_string()),
                    origin,
                );
            }
        }

        finalize_accumulator(&app, origin, &mut acc);
        first = false;
        if cancel.is_cancelled() {
            break;
        }
        if connected_at.elapsed() >= STABLE_CONNECTION {
            backoff = INITIAL_BACKOFF;
        }

        // A small per-source offset prevents two failed "Both" sessions from reconnecting
        // in lock-step and producing synchronized request spikes.
        let jitter = match origin {
            Origin::Microphone => Duration::ZERO,
            Origin::System => Duration::from_millis(173),
        };
        tokio::select! {
            _ = cancel.cancelled() => break,
            _ = tokio::time::sleep(backoff + jitter) => {}
        }
        backoff = (backoff * 2).min(MAX_BACKOFF);
    }

    finalize_accumulator(&app, origin, &mut acc);
    tracing::info!(?origin, "{} session loop ended", P::NAME);
}

async fn connect_and_run<P: RealtimeProtocol>(
    app: &AppHandle,
    proto: &mut P,
    audio_rx: &mut Receiver<AudioChunk>,
    cancel: &CancellationToken,
    acc: &mut TurnAccumulator,
) -> Result<RunEnd> {
    let request = proto.connect_request()?;
    let connected = tokio::select! {
        _ = cancel.cancelled() => return Ok(RunEnd::Stopped),
        result = tokio::time::timeout(CONNECT_TIMEOUT, connect_async(request)) => {
            result.context("WebSocket connect timed out")?
        }
    };
    let (ws, _response) = connected.context("WebSocket connect failed")?;
    let (mut write, mut read) = ws.split();

    write
        .send(Message::Text(proto.setup_json()?.into()))
        .await
        .context("failed to send setup")?;

    let origin = proto.origin();
    tracing::info!(?origin, "connected to {}", P::NAME);
    emit_status(app, SessionState::Running, None, origin);

    let finalize_after = proto.finalize_after();
    let finalize = tokio::time::sleep(IDLE);
    tokio::pin!(finalize);

    loop {
        tokio::select! {
            _ = cancel.cancelled() => {
                graceful_close(app, proto, &mut write, &mut read, acc).await;
                return Ok(RunEnd::Stopped);
            }

            maybe_chunk = audio_rx.recv() => {
                match maybe_chunk {
                    Some(mut chunk) => {
                        // Preserve ordinary short scheduling backlogs. Only coalesce once
                        // the bounded queue was effectively full, which indicates a real
                        // network stall and avoids replaying stale live speech.
                        if audio_rx.len() >= STALE_AUDIO_BACKLOG {
                            while let Ok(newer) = audio_rx.try_recv() {
                                chunk = newer;
                            }
                        }
                        let data = base64::engine::general_purpose::STANDARD.encode(&chunk.pcm_le);
                        write
                            .send(Message::Text(proto.audio_json(data)?.into()))
                            .await
                            .context("failed to send audio chunk")?;
                    }
                    None => {
                        graceful_close(app, proto, &mut write, &mut read, acc).await;
                        return Ok(RunEnd::Stopped);
                    }
                }
            }

            maybe_message = read.next() => {
                let Some(message) = maybe_message else {
                    return Ok(RunEnd::Reconnect);
                };
                let outcome = handle_socket_message(app, proto, message.context("WebSocket read error")?, acc);
                if outcome.transcript_activity {
                    if let Some(after) = finalize_after {
                        finalize.as_mut().reset(tokio::time::Instant::now() + after);
                    }
                }
                match outcome.control {
                    MessageControl::Continue => {}
                    MessageControl::Reconnect => return Ok(RunEnd::Reconnect),
                    MessageControl::Fatal(message) => return Ok(RunEnd::Fatal(message)),
                    MessageControl::Closed => return Ok(RunEnd::Stopped),
                }
            }

            _ = &mut finalize => {
                finalize_accumulator(app, origin, acc);
                finalize.as_mut().reset(tokio::time::Instant::now() + IDLE);
            }
        }
    }
}

fn handle_socket_message<P: RealtimeProtocol>(
    app: &AppHandle,
    proto: &mut P,
    message: Message,
    acc: &mut TurnAccumulator,
) -> MessageOutcome {
    match message {
        Message::Text(text) => proto.handle_message(app, &text, acc),
        Message::Binary(bytes) => String::from_utf8(bytes.to_vec())
            .map(|text| proto.handle_message(app, &text, acc))
            .unwrap_or_default(),
        Message::Close(_) => MessageOutcome::control(MessageControl::Reconnect),
        _ => MessageOutcome::default(),
    }
}

async fn graceful_close<P: RealtimeProtocol>(
    app: &AppHandle,
    proto: &mut P,
    write: &mut SplitSink<Socket, Message>,
    read: &mut SplitStream<Socket>,
    acc: &mut TurnAccumulator,
) {
    let frames = match proto.closing_json() {
        Ok(frames) => frames,
        Err(error) => {
            tracing::warn!(provider = P::NAME, "failed to build closing frame: {error}");
            Vec::new()
        }
    };

    if frames.is_empty() {
        let _ = write.send(Message::Close(None)).await;
        return;
    }
    for frame in frames {
        if write.send(Message::Text(frame.into())).await.is_err() {
            return;
        }
    }

    let deadline = tokio::time::sleep(CLOSE_DRAIN_TIMEOUT);
    tokio::pin!(deadline);
    loop {
        tokio::select! {
            _ = &mut deadline => break,
            message = read.next() => {
                let Some(Ok(message)) = message else { break; };
                if matches!(
                    handle_socket_message(app, proto, message, acc).control,
                    MessageControl::Closed | MessageControl::Fatal(_)
                ) {
                    break;
                }
            }
        }
    }
    let _ = write.send(Message::Close(None)).await;
}

fn finalize_accumulator(app: &AppHandle, origin: Origin, acc: &mut TurnAccumulator) {
    if !acc.is_empty() {
        emit_caption(app, origin, acc, true);
        acc.next_turn();
    }
}

/// Permanent client errors will not improve on retry. Rate limits and timeouts remain
/// retryable and use the normal exponential backoff path.
fn fatal_handshake_rejection(error: &anyhow::Error) -> Option<u16> {
    error.chain().find_map(|cause| {
        let tungstenite::Error::Http(response) = cause.downcast_ref::<tungstenite::Error>()? else {
            return None;
        };
        let status = response.status().as_u16();
        matches!(status, 400 | 401 | 403 | 404 | 405 | 410 | 422).then_some(status)
    })
}

pub fn emit_caption(app: &AppHandle, origin: Origin, acc: &TurnAccumulator, final_: bool) {
    let _ = app.emit(
        events::CAPTION,
        Caption {
            turn_id: acc.id,
            text: acc.translated.clone(),
            source_text: acc.source.clone(),
            final_,
            origin,
        },
    );
}

fn emit_status(app: &AppHandle, state: SessionState, message: Option<String>, origin: Origin) {
    let _ = app.emit(
        events::STATUS,
        StatusUpdate {
            state,
            message,
            origin: Some(origin),
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accumulator_advances_without_reusing_text() {
        let mut acc = TurnAccumulator {
            id: 7,
            source: "hello".into(),
            translated: "bonjour".into(),
        };
        acc.next_turn();
        assert_eq!(acc.id, 8);
        assert!(acc.is_empty());
    }
}
