//! One connection's life: `open` (connect and setup), `pump` (audio out, provider messages in)
//! and `graceful_close`. Every way it can end is a `RunEnd`; what happens next is the
//! policy's decision.

use std::collections::VecDeque;
use std::time::Duration;

use anyhow::Context;
use base64::Engine;
use futures_util::{stream::SplitSink, stream::SplitStream, SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc::Receiver;
use tokio::time::{sleep, sleep_until, timeout, Instant, MissedTickBehavior};
use tokio_tungstenite::tungstenite::protocol::CloseFrame;
use tokio_tungstenite::tungstenite::{Bytes, Message};
use tokio_tungstenite::{connect_async_with_config, MaybeTlsStream, WebSocketStream};
use tokio_util::sync::CancellationToken;

use super::policy::RunEnd;
use super::wire::{classify, Verdict};
use super::{
    apply_caption, emit_caption, finalize_accumulator, CaptionUpdate, Events, MessageControl,
    MessageOutcome, PauseRx, RealtimeProtocol, Signal, TurnAccumulator,
};
use crate::audio::AudioChunk;
use crate::types::{SessionState, StatusUpdate};

const CONNECT_TIMEOUT: Duration = Duration::from_secs(15);
pub(super) const SETUP_TIMEOUT: Duration = Duration::from_secs(15);
pub(super) const CLOSE_DRAIN_TIMEOUT: Duration = Duration::from_secs(4);
/// How long a connection must have produced no caption at all, with nothing pending, before a
/// close skips its drain. Speech still in flight surfaces as a caption within the providers'
/// one-to-three-second latency, so after three quiet seconds there is nothing to wait for — and
/// a Pause in a silent room should not hold the operator for the whole drain.
pub(super) const QUIET_BEFORE_CLOSE: Duration = Duration::from_secs(3);
/// One frame is at most a few kilobytes, so a send that has made no progress for this long is
/// a path that has stopped carrying data, not a slow one.
pub(super) const SEND_TIMEOUT: Duration = Duration::from_secs(3);
/// A healthy peer answers a ping even when it has nothing to say, so a quiet provider —
/// silence in the room, or a translation stream between turns — still proves the path works.
const PING_INTERVAL: Duration = Duration::from_secs(15);
/// Two unanswered pings: the path is gone (a Wi-Fi handoff, a dropped NAT entry) even though
/// the operating system has not noticed yet.
pub(super) const INBOUND_TIMEOUT: Duration = Duration::from_secs(30);
const IDLE: Duration = Duration::from_secs(24 * 60 * 60);
/// The longest the idle boundary leaves a turn open at the provider's word that the speaker is
/// mid-sentence (`keeps_turn_open`), counted from the turn's last text. Mistral ends a turn
/// after three seconds of silence whatever its model thinks, so a turn still held past this
/// has lost its end, not merely paused.
pub(super) const MAX_HOLD: Duration = Duration::from_secs(4);
/// Four queued 100 ms chunks means the five-slot producer queue was effectively full.
const STALE_AUDIO_BACKLOG: usize = 4;
/// About seven seconds of 100 ms chunks: a handover's drain of the old socket (at most
/// `CLOSE_DRAIN_TIMEOUT`), then the new one's TLS handshake and setup round trip, with room to
/// spare. Beyond it the oldest audio goes, so a stuck reconnect cannot grow it.
const HANDOVER_BACKLOG: usize = 70;

type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;

/// What one connection reads from and reports to, borrowed from `run_session` for its length.
pub(super) struct SocketIo<'a, E> {
    pub events: &'a E,
    pub audio_rx: &'a mut Receiver<AudioChunk>,
    pub cancel: &'a CancellationToken,
    pub pause: &'a mut PauseRx,
    /// Outlives a connection: a handover fills it while the old socket drains, and the new
    /// one sends it once set up.
    pub backlog: &'a mut Backlog,
}

struct Connection {
    write: SplitSink<Socket, Message>,
    read: SplitStream<Socket>,
    /// When the peer last sent anything at all, pongs included.
    last_inbound: Instant,
}

impl Connection {
    /// Send one frame, bounded. On a dead path a send otherwise waits until Windows gives up
    /// retransmitting, minutes in which the client sees neither Pause nor Stop.
    async fn send(&mut self, message: Message, what: &'static str) -> Result<(), RunEnd> {
        match timeout(SEND_TIMEOUT, self.write.send(message)).await {
            Ok(Ok(())) => Ok(()),
            Ok(Err(error)) => Err(anyhow::Error::from(error).context(what).into()),
            Err(_) => {
                tracing::warn!(
                    "{what}: no progress for {} seconds; reconnecting",
                    SEND_TIMEOUT.as_secs()
                );
                Err(RunEnd::Reconnect)
            }
        }
    }

    /// Takes the frame already built: a borrow of the protocol held across the send would
    /// make every client future require a `Sync` protocol.
    async fn send_audio(&mut self, frame: anyhow::Result<String>) -> Result<(), RunEnd> {
        self.send(Message::Text(frame?.into()), "failed to send audio chunk")
            .await
    }

    /// Best effort: the connection is ending either way.
    async fn close(&mut self) {
        let _ = timeout(SEND_TIMEOUT, self.write.send(Message::Close(None))).await;
    }
}

/// Audio held through a handover: while the old socket drains, then while the new one connects
/// and sets up. The five-slot producer queue alone keeps half a second and drops the rest, so
/// the gap becomes latency instead.
#[derive(Default)]
pub(super) struct Backlog {
    chunks: VecDeque<AudioChunk>,
    dropped: usize,
}

impl Backlog {
    fn push(&mut self, chunk: AudioChunk) {
        if self.chunks.len() == HANDOVER_BACKLOG {
            self.chunks.pop_front();
            self.dropped += 1;
        }
        self.chunks.push_back(chunk);
    }

    /// Forget what is held: after a failure or a pause it is stale, not a handover's gap.
    pub(super) fn clear(&mut self) {
        self.chunks.clear();
        self.dropped = 0;
    }
}

/// How one connection ended, and whether the provider had accepted its setup first: the
/// policy's cap on setup failures counts only those in a row.
pub(super) struct Ended {
    pub end: RunEnd,
    pub established: bool,
}

pub(super) async fn connect_and_run<P: RealtimeProtocol, E: Events>(
    mut io: SocketIo<'_, E>,
    proto: &mut P,
    acc: &mut TurnAccumulator,
    catch_up: bool,
) -> Ended {
    let conn = match open(&mut io, proto, acc, catch_up).await {
        Ok(conn) => conn,
        Err(end) => {
            return Ended {
                end,
                established: false,
            }
        }
    };
    Ended {
        end: stream(io, proto, acc, conn, catch_up).await,
        established: true,
    }
}

/// Everything after setup: report Running, replay a handover's backlog, then pump.
async fn stream<P: RealtimeProtocol, E: Events>(
    io: SocketIo<'_, E>,
    proto: &mut P,
    acc: &mut TurnAccumulator,
    mut conn: Connection,
    catch_up: bool,
) -> RunEnd {
    // Paused while this connection was being set up. Nothing has been streamed on it, so there
    // is nothing to flush, and reporting Running first would only make the status flicker.
    if *io.pause.borrow_and_update() {
        conn.close().await;
        return RunEnd::Paused;
    }

    let origin = proto.origin();
    tracing::info!(?origin, "{} setup complete; streaming audio", P::NAME);
    io.events.status(StatusUpdate::lane(
        origin,
        acc.lane,
        SessionState::Running,
        None,
    ));

    if io.backlog.dropped > 0 {
        tracing::warn!(
            ?origin,
            dropped = io.backlog.dropped,
            "the handover took longer than its backlog holds; dropped the oldest audio"
        );
    }
    let backlog = std::mem::take(io.backlog);
    for chunk in backlog.chunks {
        if let Err(end) = conn.send_audio(audio_json(proto, &chunk)).await {
            return end;
        }
    }

    pump(io, proto, acc, conn, catch_up).await
}

/// Connect and, where the provider asks for it, wait for its setup acknowledgement. Pause and
/// Stop are honoured throughout; a slow handshake must not hold either for up to 30 seconds.
async fn open<P: RealtimeProtocol, E: Events>(
    io: &mut SocketIo<'_, E>,
    proto: &mut P,
    acc: &mut TurnAccumulator,
    catch_up: bool,
) -> Result<Connection, RunEnd> {
    // Both are built from the source's settings alone, so one that cannot be built now never
    // can: through the ordinary error path, a key with a space in it reconnected every 16
    // seconds for the rest of the event. Built before connecting, so failing costs no socket.
    let request = proto.connect_request().map_err(RunEnd::Malformed)?;
    let setup = proto.setup_json().map_err(RunEnd::Malformed)?;
    // Nagle off: each 100 ms audio frame ends in a part-filled segment, which Nagle would hold
    // until the provider acknowledged the previous one — a round trip of added latency, or a
    // delayed-ACK timer, on every chunk of live speech.
    let connecting = timeout(
        CONNECT_TIMEOUT,
        connect_async_with_config(request, None, true),
    );
    tokio::pin!(connecting);
    let ws = loop {
        tokio::select! {
            _ = io.cancel.cancelled() => return Err(RunEnd::Stopped),
            // Abandoning the handshake drops the half-open socket; there is no session to close.
            Ok(()) = io.pause.changed() => {
                if *io.pause.borrow_and_update() {
                    return Err(RunEnd::Paused);
                }
            }
            Some(chunk) = io.audio_rx.recv(), if catch_up => io.backlog.push(chunk),
            connected = &mut connecting => {
                let (ws, _response) = connected
                    .context("WebSocket connect timed out")?
                    .context("WebSocket connect failed")?;
                break ws;
            }
        }
    };
    let (write, read) = ws.split();
    let mut conn = Connection {
        write,
        read,
        last_inbound: Instant::now(),
    };
    conn.send(Message::Text(setup.into()), "failed to send setup")
        .await?;

    let origin = proto.origin();
    tracing::info!(?origin, "connected to {}; waiting for setup", P::NAME);
    if !proto.wait_for_setup_complete() {
        return Ok(conn);
    }

    let setup = sleep(SETUP_TIMEOUT);
    tokio::pin!(setup);
    loop {
        let message = tokio::select! {
            _ = io.cancel.cancelled() => {
                conn.close().await;
                return Err(RunEnd::Stopped);
            }
            Ok(()) = io.pause.changed() => {
                if *io.pause.borrow_and_update() {
                    conn.close().await;
                    return Err(RunEnd::Paused);
                }
                continue;
            }
            Some(chunk) = io.audio_rx.recv(), if catch_up => {
                io.backlog.push(chunk);
                continue;
            }
            // A slow setup is a provider having a bad moment as often as one refusing it, and
            // Gemini Transcribe goes through setup every ten minutes: retried, within a cap.
            _ = &mut setup => {
                conn.close().await;
                return Err(RunEnd::SetupFailed {
                    reason: format!(
                        "{} did not confirm session setup within {} seconds",
                        P::NAME,
                        SETUP_TIMEOUT.as_secs()
                    ),
                    close_code: None,
                });
            }
            message = conn.read.next() => message,
        };

        let Some(message) = message else {
            return Err(RunEnd::SetupFailed {
                reason: format!(
                    "{} closed the connection before accepting session setup",
                    P::NAME
                ),
                close_code: None,
            });
        };
        let message = message.context("WebSocket read error during setup")?;
        conn.last_inbound = Instant::now();
        match &message {
            Message::Close(frame) => return Err(setup_closed::<P>(frame.as_ref())),
            Message::Ping(_) | Message::Pong(_) => continue,
            _ => {}
        }

        let outcome = handle_socket_message(io.events, proto, message, acc);
        match outcome.control {
            MessageControl::Continue if outcome.signal == Signal::SetupComplete => return Ok(conn),
            MessageControl::Continue => {}
            MessageControl::Fatal(message) => return Err(RunEnd::Fatal(message)),
            // The provider's own transient error, or its session ending before it began.
            MessageControl::Reconnect
            | MessageControl::Handover { .. }
            | MessageControl::Closed => {
                conn.close().await;
                return Err(RunEnd::SetupFailed {
                    reason: format!(
                        "{} closed the connection before accepting session setup",
                        P::NAME
                    ),
                    close_code: None,
                });
            }
        }
    }
}

/// A Close frame in answer to setup, read by its code and reason the way an `error` event is.
/// 1007 and 1008 (a setup the server will not take), or a reason about the key or the quota,
/// end the source; a 1011 or 1013 ("try again later") is retried like any setup failure.
/// A Close with no code at all says no more than the socket ending would, and is retried too.
fn setup_closed<P: RealtimeProtocol>(frame: Option<&CloseFrame>) -> RunEnd {
    let code = frame.map(|frame| u16::from(frame.code));
    let reason = frame.map_or("", |frame| frame.reason.trim());
    let detail = if reason.is_empty() {
        String::new()
    } else {
        format!(": {reason}")
    };
    let verdict = match code {
        Some(code) => classify(Some(u64::from(code)), &[], reason),
        None => Verdict::Retry,
    };
    match verdict {
        Verdict::Fatal => RunEnd::Fatal(format!("{} rejected session setup{detail}", P::NAME)),
        Verdict::Retry | Verdict::Handover => RunEnd::SetupFailed {
            reason: format!("{} closed the connection during setup{detail}", P::NAME),
            close_code: code,
        },
    }
}

async fn pump<P: RealtimeProtocol, E: Events>(
    io: SocketIo<'_, E>,
    proto: &mut P,
    acc: &mut TurnAccumulator,
    mut conn: Connection,
    mut catch_up: bool,
) -> RunEnd {
    let SocketIo {
        events,
        audio_rx,
        cancel,
        pause,
        backlog,
    } = io;
    let origin = proto.origin();
    let finalize_after = proto.finalize_after();
    let finalize = sleep(IDLE);
    tokio::pin!(finalize);
    // Set by `Signal::EndOfTurn` until the turn is finalized, and from then on the finalize
    // deadline itself: later text pushes it neither back, as it pushes back the idle timer,
    // nor forward to an idle boundary that falls inside the grace.
    let mut turn_end: Option<Instant> = None;
    // When the turn last grew, for `MAX_HOLD`.
    let mut last_activity = Instant::now();
    let mut ping = tokio::time::interval_at(Instant::now() + PING_INTERVAL, PING_INTERVAL);
    ping.set_missed_tick_behavior(MissedTickBehavior::Delay);
    // Re-armed lazily from `last_inbound` when it fires, rather than reset on every frame.
    let silence = sleep_until(conn.last_inbound + INBOUND_TIMEOUT);
    tokio::pin!(silence);
    // When this connection last changed a caption. Starts at the connection, not at "never":
    // audio sent in its first seconds may still be on its way back.
    let mut last_caption = Instant::now();

    loop {
        tokio::select! {
            _ = cancel.cancelled() => {
                graceful_close(events, proto, &mut conn, acc, last_caption).await;
                return RunEnd::Stopped;
            }

            // Closed rather than left idle: an open socket can still bill, some providers
            // drop one that goes quiet, and Gemini's session cap keeps counting.
            Ok(()) = pause.changed() => {
                if *pause.borrow_and_update() {
                    graceful_close(events, proto, &mut conn, acc, last_caption).await;
                    return RunEnd::Paused;
                }
            }

            maybe_chunk = audio_rx.recv() => {
                match maybe_chunk {
                    Some(chunk) => {
                        let chunk = next_chunk(chunk, audio_rx, &mut catch_up);
                        if let Err(end) = conn.send_audio(audio_json(proto, &chunk)).await {
                            return end;
                        }
                    }
                    None => {
                        graceful_close(events, proto, &mut conn, acc, last_caption).await;
                        return RunEnd::Stopped;
                    }
                }
            }

            maybe_message = conn.read.next() => {
                let message = match maybe_message {
                    None => return RunEnd::Reconnect,
                    Some(Err(error)) => {
                        return anyhow::Error::from(error).context("WebSocket read error").into();
                    }
                    Some(Ok(message)) => message,
                };
                conn.last_inbound = Instant::now();
                // Keep-alive traffic proves the path and nothing else; tungstenite answers
                // the provider's pings itself.
                if matches!(message, Message::Ping(_) | Message::Pong(_)) {
                    continue;
                }
                let outcome = handle_socket_message(events, proto, message, acc);
                if outcome.caption != CaptionUpdate::None {
                    last_caption = Instant::now();
                }
                match outcome.signal {
                    Signal::TranscriptActivity => {
                        last_activity = Instant::now();
                        // Text inside a turn's grace is its last words arriving; the end
                        // already set is when they have had time to.
                        if let (Some(after), None) = (finalize_after, turn_end) {
                            finalize.as_mut().reset(Instant::now() + after);
                        }
                    }
                    Signal::EndOfTurn(grace) => {
                        let end = Instant::now() + grace;
                        let end = turn_end.map_or(end, |earlier| earlier.min(end));
                        turn_end = Some(end);
                        // Even when the idle boundary would come first: closing there is
                        // what made the final "." a line of its own.
                        finalize.as_mut().reset(end);
                    }
                    Signal::None | Signal::SetupComplete => {}
                }
                match outcome.control {
                    MessageControl::Continue => {}
                    MessageControl::Reconnect => return RunEnd::Reconnect,
                    // The last one to three seconds of audio sent are still being worked on,
                    // and dropping the socket now lost them as text once per handover. Drain
                    // it first, within the time the provider says it has left, holding the
                    // speech that goes on meanwhile for the next socket.
                    MessageControl::Handover { time_left } => {
                        let limit = time_left
                            .map_or(CLOSE_DRAIN_TIMEOUT, |left| left.min(CLOSE_DRAIN_TIMEOUT));
                        let queue = Queue { audio_rx: &mut *audio_rx, backlog: &mut *backlog };
                        tracing::info!(?origin, ?limit, "{} draining before the handover", P::NAME);
                        drain(events, proto, &mut conn, acc, last_caption, limit, Some(queue))
                            .await;
                        return RunEnd::Handover;
                    }
                    // A provider ending its session mid-stream (OpenAI's `session.closed`,
                    // Mistral's `transcription.done`) is not the operator's Stop, so it moves
                    // the source to a new session like `goAway` does, with nothing left to
                    // drain. The policy turns one straight after connecting into an ordinary
                    // backoff.
                    MessageControl::Closed => return RunEnd::Handover,
                    MessageControl::Fatal(message) => return RunEnd::Fatal(message),
                }
            }

            _ = ping.tick() => {
                let ping = Message::Ping(Bytes::new());
                if let Err(end) = conn.send(ping, "failed to send ping").await {
                    return end;
                }
            }

            () = &mut silence => {
                let deadline = conn.last_inbound + INBOUND_TIMEOUT;
                if deadline > Instant::now() {
                    silence.as_mut().reset(deadline);
                } else {
                    tracing::warn!(
                        ?origin,
                        "{} sent nothing, not even a pong, for {} seconds; reconnecting",
                        P::NAME,
                        INBOUND_TIMEOUT.as_secs()
                    );
                    return RunEnd::Reconnect;
                }
            }

            () = &mut finalize => {
                let now = Instant::now();
                let ended = turn_end.is_some_and(|end| end <= now);
                let hold = finalize_after.filter(|_| {
                    !ended
                        && now.duration_since(last_activity) < MAX_HOLD
                        && proto.keeps_turn_open(acc)
                });
                if let Some(after) = hold {
                    // Mid-sentence: look again a boundary later.
                    finalize.as_mut().reset(turn_end.unwrap_or(now + after));
                } else {
                    finalize_accumulator(events, origin, acc);
                    turn_end = None;
                    finalize.as_mut().reset(now + IDLE);
                }
            }
        }
    }
}

fn audio_json<P: RealtimeProtocol>(proto: &P, chunk: &AudioChunk) -> anyhow::Result<String> {
    proto.audio_json(base64::engine::general_purpose::STANDARD.encode(&chunk.pcm_le))
}

/// The chunk to send next, given the one just received. Normally a backlog that filled the
/// queue is a network stall, so it is coalesced to the newest chunk rather than replayed late.
/// While `catch_up` is set — straight after a planned handover — the queue is live speech from
/// the gap, so it is sent in order until it is empty.
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

fn handle_socket_message<P: RealtimeProtocol, E: Events>(
    events: &E,
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
        emit_caption(events, proto.origin(), acc, final_);
    });
    outcome
}

/// Where a handover's drain puts the audio that arrives while it runs: the next socket's.
struct Queue<'a> {
    audio_rx: &'a mut Receiver<AudioChunk>,
    backlog: &'a mut Backlog,
}

/// The next chunk for a draining handover to hold; never, for a Pause or Stop.
async fn next_queued(queue: &mut Option<Queue<'_>>) -> Option<AudioChunk> {
    match queue {
        Some(queue) => queue.audio_rx.recv().await,
        None => std::future::pending().await,
    }
}

/// Close on a Pause or Stop, within `CLOSE_DRAIN_TIMEOUT`.
async fn graceful_close<P: RealtimeProtocol, E: Events>(
    events: &E,
    proto: &mut P,
    conn: &mut Connection,
    acc: &mut TurnAccumulator,
    last_caption: Instant,
) {
    drain(
        events,
        proto,
        conn,
        acc,
        last_caption,
        CLOSE_DRAIN_TIMEOUT,
        None,
    )
    .await;
}

/// Flush what the provider still holds, then close. Ends on the provider's own signal that it
/// has finished (`drain_complete`, or `Closed`), on its close or error, or after `limit` —
/// or does not wait at all when the turn is empty and nothing has been captioned for
/// `QUIET_BEFORE_CLOSE`. Whatever happens here is reported as the Pause, Stop or handover that
/// started it: a provider that rejects a closing frame must not turn a clean stop into an
/// error, and the accumulated turn is finalized by the runner either way. A handover passes a
/// `queue`, so the speech that goes on meanwhile waits for the next socket rather than being
/// sent to one that has stopped listening.
async fn drain<P: RealtimeProtocol, E: Events>(
    events: &E,
    proto: &mut P,
    conn: &mut Connection,
    acc: &mut TurnAccumulator,
    last_caption: Instant,
    limit: Duration,
    mut queue: Option<Queue<'_>>,
) {
    let quiet = acc.is_empty() && last_caption.elapsed() >= QUIET_BEFORE_CLOSE;
    let frames = match proto.closing_json() {
        Ok(frames) => frames,
        Err(error) => {
            tracing::warn!(provider = P::NAME, "failed to build closing frame: {error}");
            Vec::new()
        }
    };

    if frames.is_empty() {
        conn.close().await;
        return;
    }
    for frame in frames {
        if conn
            .send(Message::Text(frame.into()), "failed to send closing frame")
            .await
            .is_err()
        {
            return;
        }
    }
    // Still sent, best effort, so the provider ends its session cleanly; just not waited on.
    if quiet {
        conn.close().await;
        return;
    }

    let deadline = sleep(limit);
    tokio::pin!(deadline);
    loop {
        tokio::select! {
            _ = &mut deadline => break,
            Some(chunk) = next_queued(&mut queue) => {
                if let Some(queue) = &mut queue {
                    queue.backlog.push(chunk);
                }
            }
            message = conn.read.next() => {
                let Some(Ok(message)) = message else { break; };
                if matches!(message, Message::Ping(_) | Message::Pong(_)) {
                    continue;
                }
                let outcome = handle_socket_message(events, proto, message, acc);
                if proto.drain_complete(&outcome) {
                    break;
                }
                match outcome.control {
                    // A `goAway` now changes nothing: the socket is closing anyway.
                    MessageControl::Continue | MessageControl::Handover { .. } => {}
                    MessageControl::Closed | MessageControl::Reconnect => break,
                    MessageControl::Fatal(message) => {
                        tracing::info!(
                            provider = P::NAME,
                            %message,
                            "provider error while closing; ending the drain"
                        );
                        break;
                    }
                }
            }
        }
    }
    conn.close().await;
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn the_handover_backlog_keeps_the_newest_seven_seconds() {
        let mut backlog = Backlog::default();
        for tag in 1..=75 {
            backlog.push(chunk(tag));
        }
        assert_eq!(backlog.dropped, 5);
        let kept: Vec<u8> = backlog.chunks.iter().map(|c| c.pcm_le[0]).collect();
        assert_eq!(kept, (6..=75).collect::<Vec<u8>>());

        backlog.clear();
        assert!(backlog.chunks.is_empty());
        assert_eq!(backlog.dropped, 0);
    }
}
