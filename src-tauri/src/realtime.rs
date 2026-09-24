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
use tokio::sync::watch;
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

/// Whether the operator has paused the session. One sender per session, a receiver per client.
pub type PauseRx = watch::Receiver<bool>;

/// Wait until the session is no longer paused. False when it ends first — stopped, or its
/// pause sender dropped with the session.
pub async fn wait_for_resume(pause: &mut PauseRx, cancel: &CancellationToken) -> bool {
    loop {
        if !*pause.borrow_and_update() {
            return true;
        }
        tokio::select! {
            _ = cancel.cancelled() => return false,
            changed = pause.changed() => if changed.is_err() { return false; },
        }
    }
}

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
    /// The provider announced that it is about to close a healthy connection (Gemini's
    /// `goAway` ahead of its session cap). A planned move rather than a failure: reconnect at
    /// once, without backoff, and send the audio that queued up while the new socket opened.
    Handover,
    Fatal(String),
    Closed,
}

/// What a provider message did to the current turn. The runner turns this into a caption
/// event, so handlers only update the accumulator and never need an `AppHandle`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum CaptionUpdate {
    #[default]
    None,
    /// The turn changed and is still open.
    Interim,
    /// The turn is complete: emit it as final, then start the next one.
    Final,
}

#[derive(Debug, Default)]
pub struct MessageOutcome {
    pub caption: CaptionUpdate,
    pub transcript_activity: bool,
    pub setup_complete: bool,
    pub control: MessageControl,
}

impl MessageOutcome {
    pub fn caption(caption: CaptionUpdate) -> Self {
        Self {
            caption,
            ..Self::default()
        }
    }

    /// A caption update that also counts as target-text activity, which restarts the
    /// provider's idle-finalize timer.
    pub fn activity(caption: CaptionUpdate) -> Self {
        Self {
            caption,
            transcript_activity: true,
            ..Self::default()
        }
    }

    pub fn setup_complete() -> Self {
        Self {
            setup_complete: true,
            ..Self::default()
        }
    }

    pub fn control(control: MessageControl) -> Self {
        Self {
            control,
            ..Self::default()
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

    /// Parse one server message into `acc`. Pure: the runner emits whatever caption the
    /// returned outcome asks for, and advances the turn after a final one.
    fn handle_message(&mut self, text: &str, acc: &mut TurnAccumulator) -> MessageOutcome;

    fn finalize_after(&self) -> Option<Duration> {
        None
    }
}

enum RunEnd {
    Stopped,
    Reconnect,
    Handover,
    /// The operator paused: the connection was closed gracefully, its last turn flushed.
    Paused,
    Fatal(String),
}

pub async fn run_session<P: RealtimeProtocol>(
    app: AppHandle,
    mut proto: P,
    mut audio_rx: Receiver<AudioChunk>,
    cancel: CancellationToken,
    clock: SessionClock,
    mut pause: PauseRx,
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
    // Set when the last connection ended in a planned handover. The source stays Running
    // through one, and what it queued meanwhile is live speech rather than a stall's backlog.
    let mut handover = false;
    // Set when the connection closed for a pause, so the next one is a fresh start rather
    // than a recovery: no backoff, and Connecting rather than Reconnecting.
    let mut resuming = false;

    while !cancel.is_cancelled() {
        // Paused before connecting: at the start, or after a connection closed for it.
        if *pause.borrow() {
            emit_status(&app, SessionState::Paused, None, origin);
            if !wait_for_resume(&mut pause, &cancel).await {
                break;
            }
            resuming = true;
            handover = false;
        }
        if !handover {
            emit_status(
                &app,
                if first || std::mem::take(&mut resuming) {
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
        }

        let connected_at = Instant::now();
        let catch_up = std::mem::take(&mut handover);
        let io = SocketIo {
            app: &app,
            audio_rx: &mut audio_rx,
            cancel: &cancel,
            pause: &mut pause,
        };
        let mut paused = false;
        match connect_and_run(io, &mut proto, &mut acc, catch_up).await {
            Ok(RunEnd::Stopped) => break,
            Ok(RunEnd::Paused) => {
                tracing::info!(?origin, "{} paused; connection closed", P::NAME);
                paused = true;
                // Also covers a resume that arrived while the close was draining.
                resuming = true;
            }
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
            Ok(RunEnd::Handover) => {
                handover = planned_handover(connected_at.elapsed());
                tracing::info!(
                    ?origin,
                    planned = handover,
                    "{} asked to move the session; reconnecting",
                    P::NAME
                );
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
        // Waiting out a backoff here is what used to turn Gemini's ten-minute cap into a
        // caption gap: every second of it was speech dropped before the next connect.
        if handover || paused {
            continue;
        }

        // A small per-source offset prevents two failed "Both" sessions from reconnecting
        // in lock-step and producing synchronized request spikes.
        let jitter = match origin {
            Origin::Microphone => Duration::ZERO,
            Origin::System => Duration::from_millis(173),
        };
        // A pause during the backoff ends the wait; the top of the loop then holds the
        // source until it resumes, rather than reconnecting only to close again.
        tokio::select! {
            _ = cancel.cancelled() => break,
            _ = tokio::time::sleep(backoff + jitter) => {}
            Ok(()) = pause.changed() => {}
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

/// Whether a handover request is the planned move it claims to be. A provider that asks to
/// move straight after accepting a connection is refusing it, and reconnecting at once would
/// hammer it; that case keeps the ordinary backoff.
fn planned_handover(uptime: Duration) -> bool {
    uptime >= STABLE_CONNECTION
}

/// The chunk to send next, given the one just received. Normally a backlog that filled the
/// queue is a network stall, so it is coalesced to the newest chunk rather than replayed late.
/// While `catch_up` is set — straight after a planned handover — the backlog is at most the
/// half-second the new socket took to open, so it is sent in order until the queue is empty.
fn next_chunk(
    mut chunk: AudioChunk,
    audio_rx: &mut Receiver<AudioChunk>,
    catch_up: &mut bool,
) -> AudioChunk {
    if !*catch_up && audio_rx.len() >= STALE_AUDIO_BACKLOG {
        while let Ok(newer) = audio_rx.try_recv() {
            chunk = newer;
        }
    }
    *catch_up &= !audio_rx.is_empty();
    chunk
}

fn finish_source(cancel: &CancellationToken, finalize: impl FnOnce(), report: impl FnOnce()) {
    cancel.cancel();
    finalize();
    report();
}

/// What one connection reads from and reports to, borrowed from `run_session` for its length.
struct SocketIo<'a> {
    app: &'a AppHandle,
    audio_rx: &'a mut Receiver<AudioChunk>,
    cancel: &'a CancellationToken,
    pause: &'a mut PauseRx,
}

async fn connect_and_run<P: RealtimeProtocol>(
    io: SocketIo<'_>,
    proto: &mut P,
    acc: &mut TurnAccumulator,
    mut catch_up: bool,
) -> Result<RunEnd> {
    let SocketIo {
        app,
        audio_rx,
        cancel,
        pause,
    } = io;
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
                MessageControl::Reconnect | MessageControl::Handover | MessageControl::Closed => {
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

    // Paused while this connection was being set up.
    if *pause.borrow_and_update() {
        graceful_close(app, proto, &mut write, &mut read, acc).await;
        return Ok(RunEnd::Paused);
    }

    loop {
        tokio::select! {
            _ = cancel.cancelled() => {
                graceful_close(app, proto, &mut write, &mut read, acc).await;
                return Ok(RunEnd::Stopped);
            }

            // Closed rather than left idle: an open socket can still bill, some providers
            // drop one that goes quiet, and Gemini's session cap keeps counting.
            Ok(()) = pause.changed() => {
                if *pause.borrow_and_update() {
                    graceful_close(app, proto, &mut write, &mut read, acc).await;
                    return Ok(RunEnd::Paused);
                }
            }

            maybe_chunk = audio_rx.recv() => {
                match maybe_chunk {
                    Some(chunk) => {
                        let chunk = next_chunk(chunk, audio_rx, &mut catch_up);
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
                    MessageControl::Handover => return Ok(RunEnd::Handover),
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
    let outcome = match message {
        Message::Text(text) => proto.handle_message(&text, acc),
        // Parsed in place: Gemini sends its JSON as binary frames, and Live Translate's carry
        // the (discarded) output audio, so copying each frame first was the larger cost.
        Message::Binary(bytes) => std::str::from_utf8(&bytes)
            .map(|text| proto.handle_message(text, acc))
            .unwrap_or_default(),
        Message::Close(_) => MessageOutcome::control(MessageControl::Reconnect),
        _ => MessageOutcome::default(),
    };
    apply_caption(outcome.caption, acc, |acc, final_| {
        emit_caption(app, proto.origin(), acc, final_);
    });
    outcome
}

/// Emit what a handler asked for, then advance past a final turn. Split out from the socket
/// path so tests can drive handlers through exactly the sequence the runner uses.
fn apply_caption(
    update: CaptionUpdate,
    acc: &mut TurnAccumulator,
    emit: impl FnOnce(&mut TurnAccumulator, bool),
) {
    match update {
        CaptionUpdate::None => {}
        CaptionUpdate::Interim => emit(acc, false),
        CaptionUpdate::Final => {
            emit(acc, true);
            acc.next_turn();
        }
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
/// something to bury in the timestamp. Public for the built-in demo, which scripts its own
/// timeline rather than parsing a provider.
pub fn emit_caption(app: &AppHandle, origin: Origin, acc: &mut TurnAccumulator, final_: bool) {
    let end_ms = acc.clock.elapsed_ms();
    let start_ms = *acc.started_ms.get_or_insert(end_ms);
    let _ = app.emit(
        events::CAPTION,
        Caption {
            turn_id: acc.id,
            text: &acc.translated,
            source_text: &acc.source,
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

/// Drives a provider's `handle_message` the way the runner does, recording captions instead
/// of emitting them, so each wire format can be tested without an `AppHandle` or a socket.
#[cfg(test)]
pub(crate) mod test_support {
    use super::*;

    /// One caption event as the runner would have emitted it.
    #[derive(Debug, PartialEq, Eq)]
    pub struct Emitted {
        pub turn_id: u64,
        pub text: String,
        pub source_text: String,
        pub final_: bool,
    }

    impl Emitted {
        pub fn interim(turn_id: u64, text: &str, source_text: &str) -> Self {
            Self::new(turn_id, text, source_text, false)
        }

        pub fn final_(turn_id: u64, text: &str, source_text: &str) -> Self {
            Self::new(turn_id, text, source_text, true)
        }

        fn new(turn_id: u64, text: &str, source_text: &str, final_: bool) -> Self {
            Self {
                turn_id,
                text: text.to_string(),
                source_text: source_text.to_string(),
                final_,
            }
        }

        fn of(acc: &TurnAccumulator, final_: bool) -> Self {
            Self::new(acc.id, &acc.translated, &acc.source, final_)
        }
    }

    pub struct Harness<P> {
        pub proto: P,
        pub acc: TurnAccumulator,
        pub captions: Vec<Emitted>,
    }

    impl<P: RealtimeProtocol> Harness<P> {
        pub fn new(proto: P) -> Self {
            Self {
                proto,
                acc: TurnAccumulator::new(SessionClock::start()),
                captions: Vec::new(),
            }
        }

        /// One server frame, through the same caption path as `handle_socket_message`.
        pub fn send(&mut self, frame: &str) -> MessageOutcome {
            let outcome = self.proto.handle_message(frame, &mut self.acc);
            let captions = &mut self.captions;
            apply_caption(outcome.caption, &mut self.acc, |acc, final_| {
                captions.push(Emitted::of(acc, final_));
            });
            outcome
        }

        /// The runner's idle timer firing, as `finalize_accumulator` handles it.
        pub fn finalize_idle(&mut self) {
            if !self.acc.is_empty() {
                self.captions.push(Emitted::of(&self.acc, true));
                self.acc.next_turn();
            }
        }
    }
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

    #[tokio::test]
    async fn a_paused_source_waits_for_resume_or_for_the_session_to_end() {
        let cancel = CancellationToken::new();

        let (tx, mut rx) = watch::channel(false);
        assert!(
            wait_for_resume(&mut rx, &cancel).await,
            "not paused: no wait"
        );

        tx.send_replace(true);
        let resume = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(20)).await;
            tx.send_replace(false);
            tx
        });
        assert!(wait_for_resume(&mut rx, &cancel).await);
        let tx = resume.await.unwrap();

        // Stop while paused.
        tx.send_replace(true);
        cancel.cancel();
        assert!(!wait_for_resume(&mut rx, &cancel).await);

        // The session going away while paused ends the wait too.
        let (tx, mut rx) = watch::channel(true);
        drop(tx);
        assert!(!wait_for_resume(&mut rx, &CancellationToken::new()).await);
    }

    fn chunk(tag: u8) -> AudioChunk {
        AudioChunk { pcm_le: vec![tag] }
    }

    #[test]
    fn a_stall_backlog_is_coalesced_to_the_newest_chunk() {
        let (tx, mut rx) = tokio::sync::mpsc::channel(8);
        for tag in 1..=5 {
            tx.try_send(chunk(tag)).unwrap();
        }
        let first = rx.try_recv().unwrap();
        let mut catch_up = false;
        assert_eq!(next_chunk(first, &mut rx, &mut catch_up).pcm_le, [5]);
        assert!(rx.is_empty());
    }

    /// After a planned handover the queue is live speech from while the socket reopened, so
    /// every chunk goes out in order — and once it drains, a later stall coalesces again.
    #[test]
    fn a_handover_backlog_is_sent_in_order_then_coalescing_resumes() {
        let (tx, mut rx) = tokio::sync::mpsc::channel(8);
        for tag in 1..=5 {
            tx.try_send(chunk(tag)).unwrap();
        }
        let mut catch_up = true;
        let mut sent = Vec::new();
        while let Ok(first) = rx.try_recv() {
            sent.push(next_chunk(first, &mut rx, &mut catch_up).pcm_le[0]);
        }
        assert_eq!(sent, [1, 2, 3, 4, 5]);
        assert!(!catch_up);

        for tag in 6..=10 {
            tx.try_send(chunk(tag)).unwrap();
        }
        let first = rx.try_recv().unwrap();
        assert_eq!(next_chunk(first, &mut rx, &mut catch_up).pcm_le, [10]);
    }

    #[test]
    fn only_a_handover_after_a_stable_connection_skips_the_backoff() {
        assert!(planned_handover(Duration::from_secs(600)));
        assert!(planned_handover(STABLE_CONNECTION));
        assert!(!planned_handover(Duration::from_secs(2)));
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
