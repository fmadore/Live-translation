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
use crate::errors::{id, AppError};
use crate::timing::SessionClock;
use crate::types::{events, Caption, Origin, SessionState, StatusUpdate};

const CONNECT_TIMEOUT: Duration = Duration::from_secs(15);
const SETUP_TIMEOUT: Duration = Duration::from_secs(15);
const CLOSE_DRAIN_TIMEOUT: Duration = Duration::from_secs(4);
const INITIAL_BACKOFF: Duration = Duration::from_secs(1);
const MAX_BACKOFF: Duration = Duration::from_secs(16);
const STABLE_CONNECTION: Duration = Duration::from_secs(30);
const IDLE: Duration = Duration::from_secs(24 * 60 * 60);
/// Four queued 100 ms chunks means the five-slot producer queue was effectively full.
const STALE_AUDIO_BACKLOG: usize = 4;

type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;

pub struct TurnAccumulator {
    pub id: u64,
    pub source: String,
    pub translated: String,
    /// Shared with every other source in this session, so their captions land on one
    /// timeline. Lives here because the accumulator is the thing that outlives a reconnect.
    clock: SessionClock,
    /// When this turn first had any text, in ms since the session started. `None` until the
    /// first caption is emitted for it — see `emit_caption`.
    started_ms: Option<u64>,
}

impl TurnAccumulator {
    pub fn new(clock: SessionClock) -> Self {
        Self {
            id: 0,
            source: String::new(),
            translated: String::new(),
            clock,
            started_ms: None,
        }
    }

    pub fn next_turn(&mut self) {
        self.id += 1;
        self.source.clear();
        self.translated.clear();
        self.started_ms = None;
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
    pub setup_complete: bool,
    pub control: MessageControl,
}

impl MessageOutcome {
    pub fn activity() -> Self {
        Self {
            transcript_activity: true,
            setup_complete: false,
            control: MessageControl::Continue,
        }
    }

    pub fn setup_complete() -> Self {
        Self {
            transcript_activity: false,
            setup_complete: true,
            control: MessageControl::Continue,
        }
    }

    pub fn control(control: MessageControl) -> Self {
        Self {
            transcript_activity: false,
            setup_complete: false,
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

    /// Whether this provider requires an explicit setup acknowledgement before accepting
    /// audio. Gemini's Live API contract requires clients to wait for `setupComplete`.
    fn wait_for_setup_complete(&self) -> bool {
        false
    }

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
    clock: SessionClock,
) {
    // Aborting a client task must release its producer too. Normal terminal exits below
    // also finalize text before reporting that the source has ended.
    let _capture_guard = cancel.clone().drop_guard();
    let origin = proto.origin();
    let mut backoff = INITIAL_BACKOFF;
    let mut first = true;
    // Outside the connect loop, so turn ids and turn start times both survive a reconnect.
    let mut acc = TurnAccumulator::new(clock);
    let mut terminal_error = None;

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
                // The provider's own wording, which is not ours to translate; the interface
                // frames it and prints it verbatim.
                let detail = format!("{} — {message}", P::NAME);
                terminal_error = Some(AppError::with(id::PROVIDER_STOPPED, detail));
                break;
            }
            Ok(RunEnd::Reconnect) => {
                tracing::warn!(?origin, "{} stream closed; reconnecting", P::NAME);
            }
            Err(error) => {
                if let Some(status) = fatal_handshake_rejection(&error) {
                    terminal_error = Some(AppError::with(
                        id::PROVIDER_REJECTED,
                        format!("{} — HTTP {status}", P::NAME),
                    ));
                    break;
                }
                tracing::error!(?origin, "{} stream error: {error:#}", P::NAME);
                emit_status(
                    &app,
                    SessionState::Reconnecting,
                    Some(AppError::with(id::PROVIDER_RECONNECTING, error)),
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

    // A cancellation originating in capture already carries its own error. Do not
    // overwrite it with Idle; whole-session Stop publishes Idle after the drain.
    let report_idle = !cancel.is_cancelled();
    finish_source(
        &cancel,
        || finalize_accumulator(&app, origin, &mut acc),
        || {
            if let Some(error) = terminal_error {
                emit_status(&app, SessionState::Error, Some(error), origin);
            } else if report_idle {
                emit_status(&app, SessionState::Idle, None, origin);
            }
        },
    );
    tracing::info!(?origin, "{} session loop ended", P::NAME);
}

fn finish_source(cancel: &CancellationToken, finalize: impl FnOnce(), report: impl FnOnce()) {
    cancel.cancel();
    finalize();
    report();
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
    tracing::info!(?origin, "connected to {}; waiting for setup", P::NAME);

    if proto.wait_for_setup_complete() {
        let setup = tokio::time::sleep(SETUP_TIMEOUT);
        tokio::pin!(setup);

        loop {
            let message = tokio::select! {
                _ = cancel.cancelled() => {
                    let _ = write.send(Message::Close(None)).await;
                    return Ok(RunEnd::Stopped);
                }
                _ = &mut setup => {
                    return Ok(RunEnd::Fatal(format!(
                        "{} did not confirm session setup within {} seconds",
                        P::NAME,
                        SETUP_TIMEOUT.as_secs()
                    )));
                }
                message = read.next() => message,
            };

            let Some(message) = message else {
                return Ok(RunEnd::Fatal(format!(
                    "{} closed the connection before accepting session setup",
                    P::NAME
                )));
            };
            let message = message.context("WebSocket read error during setup")?;
            if let Message::Close(frame) = &message {
                let reason = frame
                    .as_ref()
                    .map(|frame| frame.reason.trim())
                    .filter(|reason| !reason.is_empty())
                    .map(|reason| format!(": {reason}"))
                    .unwrap_or_default();
                return Ok(RunEnd::Fatal(format!(
                    "{} rejected session setup{reason}",
                    P::NAME
                )));
            }

            let outcome = handle_socket_message(app, proto, message, acc);
            match outcome.control {
                MessageControl::Continue if outcome.setup_complete => break,
                MessageControl::Continue => {}
                MessageControl::Fatal(message) => return Ok(RunEnd::Fatal(message)),
                MessageControl::Reconnect | MessageControl::Closed => {
                    return Ok(RunEnd::Fatal(format!(
                        "{} closed the connection before accepting session setup",
                        P::NAME
                    )));
                }
            }
        }
    }

    tracing::info!(?origin, "{} setup complete; streaming audio", P::NAME);
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

/// Emit a caption, timed against the session clock.
///
/// A turn starts at its *first* caption, not when the previous turn ended: the silence
/// between two people speaking belongs to neither subtitle. A turn that arrives complete in
/// one message therefore has `start_ms == end_ms`, which is the truth about that caption —
/// giving a cue a minimum on-screen duration is a decision for whatever renders it, not
/// something to bury in the timestamp.
pub fn emit_caption(app: &AppHandle, origin: Origin, acc: &mut TurnAccumulator, final_: bool) {
    let end_ms = acc.clock.elapsed_ms();
    let start_ms = *acc.started_ms.get_or_insert(end_ms);
    let _ = app.emit(
        events::CAPTION,
        Caption {
            turn_id: acc.id,
            text: acc.translated.clone(),
            source_text: acc.source.clone(),
            final_,
            origin,
            start_ms,
            end_ms,
        },
    );
}

fn emit_status(app: &AppHandle, state: SessionState, message: Option<AppError>, origin: Origin) {
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

    #[tokio::test]
    async fn terminal_exit_stops_its_capture_and_finalizes_before_reporting() {
        let session = CancellationToken::new();
        let source = session.child_token();
        let sibling = session.child_token();
        let capture = source.clone();
        let worker = tokio::spawn(async move { capture.cancelled().await });
        let events = std::cell::RefCell::new(Vec::new());
        finish_source(
            &source,
            || events.borrow_mut().push("caption-final"),
            || events.borrow_mut().push("provider-error"),
        );
        tokio::time::timeout(Duration::from_secs(1), worker)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(*events.borrow(), ["caption-final", "provider-error"]);
        assert!(!sibling.is_cancelled());
        assert!(!session.is_cancelled());
    }

    #[test]
    fn authentication_rejections_are_terminal_but_rate_limits_are_retryable() {
        for (status, fatal) in [(401, true), (403, true), (429, false), (503, false)] {
            let response = tungstenite::http::Response::builder()
                .status(status)
                .body(None)
                .unwrap();
            let error = anyhow::Error::from(tungstenite::Error::Http(Box::new(response)));
            assert_eq!(fatal_handshake_rejection(&error), fatal.then_some(status));
        }
    }

    fn accumulator_at(elapsed_ms: u64) -> TurnAccumulator {
        TurnAccumulator::new(SessionClock::at(elapsed_ms))
    }

    #[test]
    fn accumulator_advances_without_reusing_text() {
        let mut acc = accumulator_at(0);
        acc.id = 7;
        acc.source = "hello".into();
        acc.translated = "bonjour".into();
        acc.next_turn();
        assert_eq!(acc.id, 8);
        assert!(acc.is_empty());
    }

    /// The whole point of `started_ms`: an interim caption and the final that replaces it are
    /// one cue, so they have to agree on where that cue begins. Emitting is what stamps it,
    /// because that is the first moment the turn is known to have any text.
    #[test]
    fn a_turn_keeps_the_start_time_of_its_first_caption() {
        let mut acc = accumulator_at(4_000);
        let first = *acc.started_ms.get_or_insert(acc.clock.elapsed_ms());
        assert!((4_000..5_000).contains(&first), "got {first}ms");

        // A later caption in the same turn must not move the start.
        let again = *acc.started_ms.get_or_insert(9_999);
        assert_eq!(again, first);
    }

    /// …and the next turn must not inherit it, or every cue after the first would claim to
    /// have started when the session did.
    #[test]
    fn the_next_turn_starts_its_own_clock() {
        let mut acc = accumulator_at(4_000);
        acc.started_ms = Some(4_000);
        acc.next_turn();
        assert_eq!(acc.started_ms, None);
    }
}
