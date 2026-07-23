//! Shared realtime WebSocket session runner.
//!
//! Both providers stream audio the same way: connect, send one setup frame, then pump
//! ~100 ms PCM chunks up and transcript events down until cancelled, reconnecting on drop.
//! This module owns that machinery — the reconnect/backoff loop, the turn accumulator, the
//! select loop, stale-audio handling, and the caption/status emits — so a provider only
//! implements [`RealtimeProtocol`]: how to connect, what the setup and audio frames look
//! like, and how to turn server messages into captions.

use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use base64::Engine;
use futures_util::{SinkExt, StreamExt};
use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc::UnboundedReceiver;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::{self, handshake::client::Request, Message};
use tokio_util::sync::CancellationToken;

use crate::audio::AudioChunk;
use crate::types::{events, Caption, Origin, SessionState, StatusUpdate};

const INITIAL_BACKOFF: Duration = Duration::from_secs(1);
const MAX_BACKOFF: Duration = Duration::from_secs(16);
/// A connection that stayed up at least this long counts as healthy: the next drop starts
/// the backoff ladder from the bottom again instead of wherever earlier flaps left it.
const STABLE_CONNECTION: Duration = Duration::from_secs(30);
/// "Disarm" sentinel for the idle-finalize timer (far in the future).
const IDLE: Duration = Duration::from_secs(24 * 60 * 60);

/// Accumulates transcript deltas for the current turn of one audio source.
#[derive(Default)]
pub struct TurnAccumulator {
    pub id: u64,
    pub source: String,
    pub translated: String,
}

impl TurnAccumulator {
    /// Close out the current turn and advance to the next id.
    pub fn next_turn(&mut self) {
        self.id += 1;
        self.source.clear();
        self.translated.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.source.is_empty() && self.translated.is_empty()
    }
}

/// Provider-specific parts of a realtime translation session. Everything else — the
/// reconnect loop, audio pump, turn bookkeeping — lives in [`run_session`].
pub trait RealtimeProtocol {
    /// Short provider name for logs and operator-facing errors ("Gemini", "OpenAI").
    const NAME: &'static str;

    fn origin(&self) -> Origin;

    /// The WebSocket connection request (URL and any auth headers).
    fn connect_request(&self) -> Result<Request>;

    /// The first frame sent after connect (session / setup configuration), as JSON text.
    fn setup_json(&self) -> Result<String>;

    /// A frame carrying one base64-encoded PCM-16 chunk, as JSON text.
    fn audio_json(&self, base64_pcm: String) -> Result<String>;

    /// Apply one server frame, mutating the accumulator and emitting captions as needed.
    /// Returns `true` if the frame carried transcript activity — used to re-arm the
    /// idle-finalize timer when [`finalize_after`](Self::finalize_after) is set.
    fn handle_message(&mut self, app: &AppHandle, text: &str, acc: &mut TurnAccumulator) -> bool;

    /// For streams with no turn-complete event: finalize the current caption after this
    /// much time without transcript activity. `None` disables the timer.
    fn finalize_after(&self) -> Option<Duration> {
        None
    }
}

/// Run a translation session for one audio source, reconnecting until cancelled.
pub async fn run_session<P: RealtimeProtocol>(
    app: AppHandle,
    mut proto: P,
    mut audio_rx: UnboundedReceiver<AudioChunk>,
    cancel: CancellationToken,
) {
    let origin = proto.origin();
    let mut backoff = INITIAL_BACKOFF;
    let mut first = true;
    // Lives across reconnects so turn ids stay monotonic for the whole session.
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

        // Captions are live: audio that piled up while the socket was down is stale by
        // definition, and replaying it would put us tens of seconds behind the speaker.
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
            Ok(()) => {
                // Clean close. If we weren't cancelled, the server hung up — reconnect.
                if cancel.is_cancelled() {
                    break;
                }
                tracing::warn!(?origin, "{} stream closed; reconnecting", P::NAME);
            }
            Err(e) => {
                // A 4xx handshake rejection (bad key, bad model) won't fix itself; stop
                // with a clear message instead of looping "Reconnecting…" forever.
                if let Some(status) = handshake_rejection(&e) {
                    tracing::error!(
                        ?origin,
                        "{} rejected the connection: HTTP {status}",
                        P::NAME
                    );
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
                tracing::error!(?origin, "{} stream error: {e:#}", P::NAME);
                emit_status(
                    &app,
                    SessionState::Reconnecting,
                    Some(format!("{e}")),
                    origin,
                );
            }
        }

        first = false;
        if cancel.is_cancelled() {
            break;
        }
        if connected_at.elapsed() >= STABLE_CONNECTION {
            backoff = INITIAL_BACKOFF;
        }

        // Backoff, but stay responsive to cancellation.
        tokio::select! {
            _ = cancel.cancelled() => break,
            _ = tokio::time::sleep(backoff) => {}
        }
        backoff = (backoff * 2).min(MAX_BACKOFF);
    }

    tracing::info!(?origin, "{} session loop ended", P::NAME);
}

async fn connect_and_run<P: RealtimeProtocol>(
    app: &AppHandle,
    proto: &mut P,
    audio_rx: &mut UnboundedReceiver<AudioChunk>,
    cancel: &CancellationToken,
    acc: &mut TurnAccumulator,
) -> Result<()> {
    let request = proto.connect_request()?;
    let (ws, _resp) = connect_async(request)
        .await
        .context("WebSocket connect failed")?;
    let (mut write, mut read) = ws.split();

    // Hand the provider its session configuration before any audio.
    write
        .send(Message::Text(proto.setup_json()?))
        .await
        .context("failed to send setup")?;

    let origin = proto.origin();
    tracing::info!(?origin, "connected to {}", P::NAME);
    emit_status(app, SessionState::Running, None, origin);

    // Idle-finalize timer for streams with no turn lifecycle: armed on transcript
    // activity, disarmed (far-future) otherwise.
    let finalize_after = proto.finalize_after();
    let finalize = tokio::time::sleep(IDLE);
    tokio::pin!(finalize);
    let rearm = |finalize: &mut std::pin::Pin<&mut tokio::time::Sleep>, active: bool| {
        if let (true, Some(after)) = (active, finalize_after) {
            finalize.as_mut().reset(tokio::time::Instant::now() + after);
        }
    };

    loop {
        tokio::select! {
            biased;

            _ = cancel.cancelled() => {
                let _ = write.send(Message::Close(None)).await;
                return Ok(());
            }

            maybe_chunk = audio_rx.recv() => {
                match maybe_chunk {
                    Some(chunk) => {
                        let data = base64::engine::general_purpose::STANDARD.encode(&chunk.pcm_le);
                        write
                            .send(Message::Text(proto.audio_json(data)?))
                            .await
                            .context("failed to send audio chunk")?;
                    }
                    // All capture senders dropped: the session is stopping.
                    None => {
                        let _ = write.send(Message::Close(None)).await;
                        return Ok(());
                    }
                }
            }

            maybe_msg = read.next() => {
                match maybe_msg {
                    Some(Ok(Message::Text(text))) => {
                        let active = proto.handle_message(app, &text, acc);
                        rearm(&mut finalize, active);
                    }
                    Some(Ok(Message::Binary(bytes))) => {
                        // The server occasionally frames JSON as binary.
                        if let Ok(text) = String::from_utf8(bytes) {
                            let active = proto.handle_message(app, &text, acc);
                            rearm(&mut finalize, active);
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => return Ok(()),
                    Some(Ok(_)) => { /* ping/pong handled by the library */ }
                    Some(Err(e)) => return Err(e).context("WebSocket read error"),
                }
            }

            _ = &mut finalize => {
                // Idle gap with no new transcript: close out the current caption.
                if !acc.is_empty() {
                    emit_caption(app, origin, acc, true);
                    acc.next_turn();
                }
                finalize.as_mut().reset(tokio::time::Instant::now() + IDLE);
            }
        }
    }
}

/// If the error chain contains a WebSocket handshake rejected with a 4xx status, return it.
fn handshake_rejection(e: &anyhow::Error) -> Option<u16> {
    e.chain()
        .find_map(|cause| match cause.downcast_ref::<tungstenite::Error>() {
            Some(tungstenite::Error::Http(resp)) if resp.status().is_client_error() => {
                Some(resp.status().as_u16())
            }
            _ => None,
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
