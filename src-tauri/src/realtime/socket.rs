//! One connection's life: `open` (connect and setup), `pump` (audio out, provider messages in)
//! and `graceful_close`. Every way it can end is a `RunEnd`; what happens next is the
//! policy's decision.

use std::collections::VecDeque;
use std::time::Duration;

use anyhow::Context;
use futures_util::{SinkExt, StreamExt, stream::SplitSink, stream::SplitStream};
use tokio::net::TcpStream;
use tokio::sync::mpsc::Receiver;
use tokio::time::{Instant, MissedTickBehavior, sleep, sleep_until, timeout};
use tokio_tungstenite::tungstenite::protocol::CloseFrame;
use tokio_tungstenite::tungstenite::{Bytes, Message};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async_with_config};
use tokio_util::sync::CancellationToken;

use super::policy::{RunEnd, unconfirmed_end};
use super::turn_timer::TurnTimer;
use super::wire::{Close, Verdict};
use super::{
    CaptionUpdate, Events, MessageControl, MessageOutcome, PauseRx, RealtimeProtocol, Signal,
    TurnAccumulator, apply_caption, emit_caption, finalize_accumulator,
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
    /// Whether the peer has sent a message of its own (a text or binary frame, not a pong or
    /// a close) since setup: for a provider without a setup acknowledgement, the sign that it
    /// took the connection.
    heard: bool,
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

    /// One chunk's audio frame, then whatever the provider sends after it
    /// (`RealtimeProtocol::after_audio`). Both are built before either is sent, for the reason
    /// `send_audio` gives.
    async fn send_chunk<P: RealtimeProtocol>(
        &mut self,
        proto: &mut P,
        pcm_le: &[u8],
    ) -> Result<(), RunEnd> {
        let frame = proto.audio_frame(pcm_le);
        let after = proto.after_audio(pcm_le);
        self.send_audio(frame).await?;
        match after {
            Some(after) => {
                self.send(Message::Text(after.into()), "failed to send audio chunk")
                    .await
            }
            None => Ok(()),
        }
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
/// policy's cap on setup failures counts only those in a row. For a provider that confirms no
/// setup, a connection it ended at once without a word is counted as a failed setup
/// (`policy::unconfirmed_end`).
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
    let mut conn = match open(&mut io, proto, acc, catch_up).await {
        Ok(conn) => conn,
        Err(end) => {
            return Ended {
                end,
                established: false,
            };
        }
    };
    let opened = Instant::now();
    let end = stream(io, proto, acc, &mut conn, catch_up).await;
    if proto.wait_for_setup_complete() {
        return Ended {
            end,
            established: true,
        };
    }
    let end = unconfirmed_end(end, opened.elapsed(), conn.heard, P::NAME);
    Ended {
        established: !matches!(end, RunEnd::SetupFailed { .. }),
        end,
    }
}

/// Everything after setup: report Running, replay a handover's backlog, then pump.
async fn stream<P: RealtimeProtocol, E: Events>(
    io: SocketIo<'_, E>,
    proto: &mut P,
    acc: &mut TurnAccumulator,
    conn: &mut Connection,
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
        if let Err(end) = conn.send_chunk(proto, &chunk.pcm_le).await {
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
    prepare(io, proto, catch_up).await?;
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
        heard: false,
    };
    if let Some(setup) = setup {
        conn.send(Message::Text(setup.into()), "failed to send setup")
            .await?;
    }

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

/// The provider's own step before connecting (`RealtimeProtocol::prepare`), with Pause and Stop
/// honoured and the audio of a handover's gap held meanwhile, as during the handshake.
async fn prepare<P: RealtimeProtocol, E: Events>(
    io: &mut SocketIo<'_, E>,
    proto: &mut P,
    catch_up: bool,
) -> Result<(), RunEnd> {
    let preparing = timeout(CONNECT_TIMEOUT, proto.prepare());
    tokio::pin!(preparing);
    loop {
        tokio::select! {
            _ = io.cancel.cancelled() => return Err(RunEnd::Stopped),
            Ok(()) = io.pause.changed() => {
                if *io.pause.borrow_and_update() {
                    return Err(RunEnd::Paused);
                }
            }
            Some(chunk) = io.audio_rx.recv(), if catch_up => io.backlog.push(chunk),
            prepared = &mut preparing => {
                return match prepared {
                    Ok(result) => result.map_err(RunEnd::from),
                    Err(_) => Err(RunEnd::Failed(anyhow::anyhow!(
                        "{} did not prepare the connection within {} seconds",
                        P::NAME,
                        CONNECT_TIMEOUT.as_secs()
                    ))),
                };
            }
        }
    }
}

/// " (close code 1011: Internal error.)", or nothing for a Close that carried neither.
fn close_detail(close: &Close) -> String {
    let described = close.describe();
    if described.is_empty() {
        described
    } else {
        format!(" ({described})")
    }
}

/// A Close frame in answer to setup, read by `Close::verdict`: 1007, 1008 or a reason about
/// the key or the quota end the source; anything else, a bare Normal Closure included, is
/// retried like any setup failure, within the cap.
pub(super) fn setup_closed<P: RealtimeProtocol>(frame: Option<&CloseFrame>) -> RunEnd {
    let close = Close::of(frame);
    let detail = close_detail(&close);
    match close.verdict() {
        Verdict::Fatal => RunEnd::Fatal(format!("{} rejected session setup{detail}", P::NAME)),
        Verdict::Retry | Verdict::Handover => RunEnd::SetupFailed {
            reason: format!("{} closed the connection during setup{detail}", P::NAME),
            close_code: close.code,
        },
    }
}

/// A Close frame once streaming, read the same way as during setup. A refusal ends the source
/// with the provider's reason; any other close reconnects with backoff, and the code and reason
/// it carried are the Reconnecting detail, so a provider that keeps closing says why. A Close
/// with neither says no more than the socket ending, and reconnects as that does.
pub(super) fn stream_closed<P: RealtimeProtocol>(frame: Option<&CloseFrame>) -> RunEnd {
    let close = Close::of(frame);
    let detail = close_detail(&close);
    match close.verdict() {
        Verdict::Fatal => RunEnd::Fatal(format!("{} closed the session{detail}", P::NAME)),
        Verdict::Retry | Verdict::Handover if detail.is_empty() => RunEnd::Reconnect,
        Verdict::Retry | Verdict::Handover => {
            RunEnd::Failed(anyhow::anyhow!("{} closed the connection{detail}", P::NAME))
        }
    }
}

async fn pump<P: RealtimeProtocol, E: Events>(
    io: SocketIo<'_, E>,
    proto: &mut P,
    acc: &mut TurnAccumulator,
    conn: &mut Connection,
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
    // When the open turn is finalized on a timer; `finalize` sleeps until its deadline.
    let mut timer = TurnTimer::new(proto.finalize_after(), Instant::now());
    let finalize = sleep_until(timer.deadline());
    tokio::pin!(finalize);
    let mut ping = tokio::time::interval_at(Instant::now() + PING_INTERVAL, PING_INTERVAL);
    ping.set_missed_tick_behavior(MissedTickBehavior::Delay);
    // Re-armed lazily from `last_inbound` when it fires, rather than reset on every frame.
    let silence = sleep_until(conn.last_inbound + INBOUND_TIMEOUT);
    tokio::pin!(silence);
    // When this connection last changed a caption. Starts at the connection, not at "never":
    // audio sent in its first seconds may still be on its way back.
    let mut last_caption = Instant::now();
    // When audio last went out, for the provider's keepalive; re-armed lazily, like `silence`.
    let keepalive = proto.keepalive();
    let mut last_audio = Instant::now();
    let idle_audio = sleep_until(keepalive.as_ref().map_or_else(
        || Instant::now() + super::turn_timer::IDLE,
        |keepalive| last_audio + keepalive.after,
    ));
    tokio::pin!(idle_audio);

    loop {
        tokio::select! {
            _ = cancel.cancelled() => {
                graceful_close(events, proto, conn, acc, last_caption).await;
                return RunEnd::Stopped;
            }

            // Closed rather than left idle: an open socket can still bill, some providers
            // drop one that goes quiet, and Gemini's session cap keeps counting.
            Ok(()) = pause.changed() => {
                if *pause.borrow_and_update() {
                    graceful_close(events, proto, conn, acc, last_caption).await;
                    return RunEnd::Paused;
                }
            }

            maybe_chunk = audio_rx.recv() => {
                match maybe_chunk {
                    Some(chunk) => {
                        let chunk = next_chunk(chunk, audio_rx, &mut catch_up);
                        if let Err(end) = conn.send_chunk(proto, &chunk.pcm_le).await {
                            return end;
                        }
                        last_audio = Instant::now();
                    }
                    None => {
                        graceful_close(events, proto, conn, acc, last_caption).await;
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
                match &message {
                    Message::Ping(_) | Message::Pong(_) => continue,
                    Message::Close(frame) => return stream_closed::<P>(frame.as_ref()),
                    _ => conn.heard = true,
                }
                let outcome = handle_socket_message(events, proto, message, acc);
                if outcome.caption != CaptionUpdate::None {
                    last_caption = Instant::now();
                }
                match outcome.signal {
                    Signal::TranscriptActivity => {
                        timer.activity(Instant::now());
                        finalize.as_mut().reset(timer.deadline());
                    }
                    Signal::EndOfTurn(grace) => {
                        timer.end_of_turn(Instant::now(), grace);
                        finalize.as_mut().reset(timer.deadline());
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
                        let queue = Queue {
                            audio_rx: &mut *audio_rx,
                            backlog: &mut *backlog,
                            cancel,
                        };
                        tracing::info!(?origin, ?limit, "{} draining before the handover", P::NAME);
                        drain(events, proto, conn, acc, last_caption, limit, Some(queue))
                            .await;
                        // A Stop during the drain ended it: the source is over, not moving.
                        if cancel.is_cancelled() {
                            return RunEnd::Stopped;
                        }
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

            () = &mut idle_audio, if keepalive.is_some() => {
                let Some(keepalive) = &keepalive else { continue };
                let deadline = last_audio + keepalive.after;
                if deadline <= Instant::now() {
                    if let Err(end) = conn.send_chunk(proto, &keepalive.silence.pcm_le).await {
                        return end;
                    }
                    last_audio = Instant::now();
                }
                idle_audio.as_mut().reset(last_audio + keepalive.after);
            }

            () = &mut finalize => {
                if timer.fire(Instant::now(), || proto.keeps_turn_open(acc)) {
                    finalize_accumulator(events, origin, acc);
                }
                finalize.as_mut().reset(timer.deadline());
            }
        }
    }
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

/// Where a handover's drain puts the audio that arrives while it runs: the next socket's. And
/// the source's Stop, which ends a handover's drain at once.
struct Queue<'a> {
    audio_rx: &'a mut Receiver<AudioChunk>,
    backlog: &'a mut Backlog,
    cancel: &'a CancellationToken,
}

/// The next chunk for a draining handover to hold; never, for a Pause or Stop.
async fn next_queued(queue: &mut Option<Queue<'_>>) -> Option<AudioChunk> {
    match queue {
        Some(queue) => queue.audio_rx.recv().await,
        None => std::future::pending().await,
    }
}

/// When a Stop arrives during a handover's drain; never, for the drain of a Pause or Stop,
/// which is that Stop's own flush.
async fn stopped(cancel: Option<&CancellationToken>) {
    match cancel {
        Some(cancel) => cancel.cancelled().await,
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
/// or does not wait at all when the turn is empty, the provider holds nothing still to come
/// (`keeps_turn_open`), and nothing has been captioned for `QUIET_BEFORE_CLOSE`. Whatever happens here is reported as the Pause, Stop or handover that
/// started it: a provider that rejects a closing frame must not turn a clean stop into an
/// error, and the accumulated turn is finalized by the runner either way. A handover passes a
/// `queue`, so the speech that goes on meanwhile waits for the next socket rather than being
/// sent to one that has stopped listening; a Stop then ends the drain at once.
async fn drain<P: RealtimeProtocol, E: Events>(
    events: &E,
    proto: &mut P,
    conn: &mut Connection,
    acc: &mut TurnAccumulator,
    last_caption: Instant,
    limit: Duration,
    mut queue: Option<Queue<'_>>,
) {
    // DeepL can hold the tentative start of a sentence that is not on screen yet: the
    // accumulator is empty, and only the drain brings those words back.
    let quiet = acc.is_empty()
        && !proto.keeps_turn_open(acc)
        && last_caption.elapsed() >= QUIET_BEFORE_CLOSE;
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
    let cancel = queue.as_ref().map(|queue| queue.cancel);
    loop {
        tokio::select! {
            _ = &mut deadline => break,
            () = stopped(cancel) => break,
            Some(chunk) = next_queued(&mut queue) => {
                if let Some(queue) = &mut queue {
                    queue.backlog.push(chunk);
                }
            }
            message = conn.read.next() => {
                let Some(Ok(message)) = message else { break; };
                match message {
                    Message::Ping(_) | Message::Pong(_) => continue,
                    Message::Text(_) | Message::Binary(_) => conn.heard = true,
                    _ => {}
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
    use std::sync::Mutex;

    use tokio::sync::{mpsc, watch};
    use tokio_tungstenite::accept_async;

    use super::*;
    use crate::timing::SessionClock;
    use crate::types::{Caption, Origin};

    /// The states a connection reported.
    #[derive(Default)]
    struct States(Mutex<Vec<SessionState>>);

    impl super::super::Events for States {
        fn caption(&self, _caption: Caption<'_>) {}

        fn status(&self, update: StatusUpdate) {
            self.0.lock().unwrap().push(update.state);
        }
    }

    /// A provider like DeepL: no setup acknowledgement, so `open` returns as the socket opens
    /// and `stream` is where a Pause is first looked at. Its frame for a chunk tagged 0xff
    /// cannot be built.
    struct NoAck;

    impl RealtimeProtocol for NoAck {
        const NAME: &'static str = "NoAck";

        fn origin(&self) -> Origin {
            Origin::Microphone
        }

        fn connect_request(
            &self,
        ) -> anyhow::Result<tokio_tungstenite::tungstenite::handshake::client::Request> {
            anyhow::bail!("these tests open their own sockets")
        }

        fn setup_json(&self) -> anyhow::Result<Option<String>> {
            Ok(None)
        }

        fn audio_json(&self, base64_pcm: String) -> anyhow::Result<String> {
            anyhow::ensure!(base64_pcm != "/w==", "the frame could not be built");
            Ok(base64_pcm)
        }

        fn handle_message(&mut self, _text: &str, _acc: &mut TurnAccumulator) -> MessageOutcome {
            MessageOutcome::default()
        }
    }

    /// An open connection, and the server's end of it.
    async fn connected() -> (Connection, WebSocketStream<TcpStream>) {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("ws://{}", listener.local_addr().unwrap());
        let (client, server) = tokio::join!(connect_async_with_config(url, None, true), async {
            let (stream, _) = listener.accept().await.unwrap();
            accept_async(stream).await.unwrap()
        });
        let (write, read) = client.unwrap().0.split();
        let conn = Connection {
            write,
            read,
            last_inbound: Instant::now(),
            heard: false,
        };
        (conn, server)
    }

    /// `stream` on `conn` with `backlog` to replay, paused or not; how it ended and what it
    /// reported.
    async fn stream_with(
        conn: &mut Connection,
        backlog: &mut Backlog,
        paused: bool,
    ) -> (RunEnd, Vec<SessionState>) {
        let events = States::default();
        let (_audio, mut audio_rx) = mpsc::channel(5);
        let cancel = CancellationToken::new();
        let (_pause_tx, mut pause) = watch::channel(paused);
        let io = SocketIo {
            events: &events,
            audio_rx: &mut audio_rx,
            cancel: &cancel,
            pause: &mut pause,
            backlog,
        };
        let mut acc = TurnAccumulator::new(SessionClock::start(), 0);
        let end = stream(io, &mut NoAck, &mut acc, conn, false).await;
        (end, events.0.into_inner().unwrap())
    }

    /// A Pause that lands once the socket is open but before streaming starts: for a provider
    /// without a setup acknowledgement there is no setup wait to catch it, so `stream` closes
    /// the socket, reports nothing, and sends none of the handover's backlog.
    #[tokio::test]
    async fn a_pause_as_the_socket_opens_closes_it_before_running() {
        let (mut conn, mut server) = connected().await;
        let mut backlog = Backlog::default();
        backlog.push(chunk(1));
        let (end, states) = stream_with(&mut conn, &mut backlog, true).await;
        assert!(matches!(end, RunEnd::Paused), "{end:?}");
        assert!(states.is_empty(), "reported {states:?}");
        assert!(matches!(server.next().await, Some(Ok(Message::Close(_)))));
    }

    /// A frame that cannot be sent in the middle of a handover's replay ends the connection
    /// there: the chunks before it went out, none after it, and nothing is pumped.
    #[tokio::test]
    async fn a_failure_in_the_middle_of_the_backlog_replay_ends_the_connection() {
        let (mut conn, mut server) = connected().await;
        let mut backlog = Backlog::default();
        for tag in [1, 0xff, 3] {
            backlog.push(chunk(tag));
        }
        let (end, states) = stream_with(&mut conn, &mut backlog, false).await;
        let RunEnd::Failed(error) = end else {
            panic!("expected a failure, got {end:?}");
        };
        assert_eq!(error.to_string(), "the frame could not be built");
        assert_eq!(states, [SessionState::Running]);
        assert!(backlog.chunks.is_empty(), "the replay took the backlog");
        drop(conn);
        assert_eq!(server.next().await.unwrap().unwrap(), Message::text("AQ=="));
        assert!(!matches!(server.next().await, Some(Ok(Message::Text(_)))));

        // The same for a write the socket refuses.
        let (mut conn, _server) = connected().await;
        conn.write.close().await.unwrap();
        let mut backlog = Backlog::default();
        backlog.push(chunk(1));
        backlog.push(chunk(2));
        let (end, _) = stream_with(&mut conn, &mut backlog, false).await;
        let RunEnd::Failed(error) = end else {
            panic!("expected a failure, got {end:?}");
        };
        assert_eq!(error.to_string(), "failed to send audio chunk");
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
