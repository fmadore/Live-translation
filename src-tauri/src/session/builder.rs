//! Wiring a session's sources into their slots: each source's producer, its audio
//! channel, and the client — or, with a second caption language, the relay and two clients —
//! reading it.

use std::sync::Arc;

use anyhow::{Context, Result};
use tauri::AppHandle;
use tauri::async_runtime::JoinHandle as AsyncJoinHandle;
use tokio::sync::mpsc::{Receiver, Sender, channel, error::TrySendError};
use tokio_util::sync::CancellationToken;

use super::capture::{report_source_failure, source_failure};
use super::options::caption_languages;
use super::settings::ProviderSettings;
use super::{ActiveSession, SourceSlot};
use crate::audio::AudioChunk;
use crate::audio::fixture::run_rehearsal;
use crate::audio::sink::{AudioSink, CaptureCompletion};
use crate::realtime::{Events, PauseRx, RealtimeProtocol, Track, TurnCounter, run_session};
use crate::timing::SessionClock;
use crate::types::{Origin, Provider, TargetLanguage};
use crate::whisper::{self, LocalSession, spool::Spool};

/// At most half a second of 100 ms chunks. The realtime consumer coalesces queued chunks
/// to the newest one after a stall, favoring live latency over replaying stale speech.
pub(super) const AUDIO_CHANNEL_CAPACITY: usize = 5;

/// Copy one source's audio to each of its caption-language clients. Never blocks on either:
/// a client that has fallen behind drops chunks on its own queue, as a lone client would at
/// the producer's. Ends when the producer does, once every client has gone — and then
/// cancels the source, which stops its capture — or when the source is cancelled, so Stop
/// never waits on a capture thread that is wedged and still holds the producer end.
async fn relay_audio(
    mut input: Receiver<AudioChunk>,
    outputs: Vec<Sender<AudioChunk>>,
    source: CancellationToken,
) {
    let _stop_capture = source.clone().drop_guard();
    while let Some(chunk) = tokio::select! {
        _ = source.cancelled() => None,
        chunk = input.recv() => chunk,
    } {
        let Some((last, rest)) = outputs.split_last() else {
            break;
        };
        let accepts = |output: &Sender<AudioChunk>, chunk| {
            !matches!(output.try_send(chunk), Err(TrySendError::Closed(_)))
        };
        // The last output takes the chunk itself; only the ones before it need a copy.
        let mut open = rest
            .iter()
            .filter(|output| accepts(output, chunk.clone()))
            .count();
        if accepts(last, chunk) {
            open += 1;
        }
        if open == 0 {
            break;
        }
    }
}

/// The per-source plumbing every client gets, whichever provider it speaks.
pub(super) struct ClientIo {
    pub(super) app: AppHandle,
    pub(super) origin: Origin,
    /// Which of the session's caption languages this client produces; see `Caption::lane`.
    lane: u8,
    pub(super) audio_rx: Receiver<AudioChunk>,
    pub(super) cancel: CancellationToken,
    pub(super) clock: SessionClock,
    pub(super) pause: PauseRx,
    /// Where this track's turn ids come from; see `TurnCounter`.
    turns: TurnCounter,
}

impl ClientIo {
    pub(super) fn spawn_realtime<P: RealtimeProtocol + Send + 'static>(
        self,
        proto: P,
    ) -> AsyncJoinHandle<()> {
        tauri::async_runtime::spawn(run_session(
            self.app,
            proto,
            self.audio_rx,
            self.cancel,
            self.clock,
            self.pause,
            Track {
                lane: self.lane,
                turns: self.turns,
            },
        ))
    }
}

/// Builds a session's sources, each into its slot: `start` with the session's engine. It holds
/// what is particular to an engine; the session holds the rest.
pub(super) struct SessionBuilder<'a> {
    pub(super) app: &'a AppHandle,
    pub(super) settings: ProviderSettings,
    pub(super) api_key: String,
}

impl SessionBuilder<'_> {
    /// Start the producer and the clients of `slot`'s source, with `slot`'s engine. What has
    /// been started is in the slot even when this fails part-way, so draining the slot releases
    /// it.
    pub(super) fn build_source(
        &self,
        session: &ActiveSession,
        slot: &mut SourceSlot,
    ) -> Result<()> {
        if slot.engine.is_local_whisper() {
            return self.build_local_source(session, slot);
        }
        let (audio_tx, audio_rx) = channel::<AudioChunk>(AUDIO_CHANNEL_CAPACITY);
        let cancel = slot.cancel.clone();
        spawn_producer(self.app, session, slot, audio_tx.into())?;

        let targets = caption_languages(&session.options);
        if targets.len() == 1 {
            // One language: the client reads the producer directly and owns the source's
            // token, so its end stops the capture, as it always has.
            let client = self.client(session, slot, 0, targets[0], audio_rx, cancel)?;
            slot.clients.push(client);
            return Ok(());
        }
        // Two languages: each client gets its own copy of the audio and a child token, so
        // one failing leaves the other captioning. The relay stops the capture when both
        // have gone.
        let mut outputs = Vec::with_capacity(targets.len());
        for (lane, target) in (0u8..).zip(targets) {
            let (lane_tx, lane_rx) = channel::<AudioChunk>(AUDIO_CHANNEL_CAPACITY);
            outputs.push(lane_tx);
            let client = self.client(session, slot, lane, target, lane_rx, cancel.child_token())?;
            slot.clients.push(client);
        }
        slot.relay = Some(tauri::async_runtime::spawn(relay_audio(
            audio_rx, outputs, cancel,
        )));
        Ok(())
    }

    fn client(
        &self,
        session: &ActiveSession,
        slot: &SourceSlot,
        lane: u8,
        target: TargetLanguage,
        audio_rx: Receiver<AudioChunk>,
        cancel: CancellationToken,
    ) -> Result<AsyncJoinHandle<()>> {
        let io = ClientIo {
            app: self.app.clone(),
            origin: slot.origin,
            lane,
            audio_rx,
            cancel,
            clock: session.clock,
            pause: session.pause.subscribe(),
            turns: slot.turns[usize::from(lane)].clone(),
        };
        self.settings.spawn_client(io, &self.api_key, target)
    }

    fn build_local_source(&self, session: &ActiveSession, slot: &mut SourceSlot) -> Result<()> {
        // Thirty seconds of scheduling slack for the disk writer. Inference never consumes
        // this queue; it reads its own spool and may lag by much longer without losing audio.
        let (sender, input) = channel(300);
        let spool = Arc::new(Spool::new()?);
        let cancel = slot.cancel.clone();
        let sink = AudioSink::Local {
            sender,
            spool: spool.clone(),
            clock: session.clock,
            pause: session.pause.subscribe(),
            cancel: cancel.clone(),
        };
        spawn_producer(self.app, session, slot, sink)?;
        let model = session
            .model
            .clone()
            .context("Whisper model was not loaded")?;
        slot.clients
            .push(tauri::async_runtime::spawn(whisper::run(LocalSession {
                app: self.app.clone(),
                model,
                language: session.options.spoken_language.clone(),
                // The only difference between the two Whisper providers.
                translate: slot.engine.can_translate(),
                glossary: session.options.glossary.clone(),
                origin: slot.origin,
                input,
                spool,
                capture_cancel: cancel,
                abort: session.abort.clone(),
                abandoned: slot.abandoned.clone(),
                failure: session.failure.clone(),
                pause: session.pause.subscribe(),
                // Whisper writes one language, so the source's first lane.
                turns: slot.turns[0].clone(),
            })));
        Ok(())
    }
}

/// Start whatever feeds one source's audio channel, at the rate its engine takes.
fn spawn_producer(
    app: &AppHandle,
    session: &ActiveSession,
    slot: &mut SourceSlot,
    audio_tx: AudioSink,
) -> Result<()> {
    if slot.engine == Provider::OnDevice {
        // The deterministic demo emits its own level/caption timeline and never opens a
        // capture device. Close the unused producer immediately.
        drop(audio_tx);
        return Ok(());
    }

    let app = app.clone();
    let origin = slot.origin;
    let mut capture_completion = audio_tx.completion_guard();
    let level_tx = session.level_tx.clone();
    let target_rate = slot.engine.input_sample_rate();
    let cancel = slot.cancel.clone();
    match session.options.rehearsal {
        // Rehearsal swaps the capture device for a bundled recording and changes nothing
        // else: same channel, same chunk shape, same engine below it.
        Some(language) => {
            slot.fixture = Some(tauri::async_runtime::spawn(async move {
                let result = run_rehearsal(
                    &app,
                    language,
                    target_rate,
                    level_tx,
                    audio_tx,
                    cancel.clone(),
                )
                .await;
                finish_producer(result, capture_completion.as_mut(), &app, origin, &cancel);
            }));
        }
        None => {
            let capture = session.capture.clone();
            let handle = std::thread::Builder::new()
                .name(format!("capture-{origin:?}"))
                .spawn(move || {
                    produce(
                        || capture.run(origin, target_rate, level_tx, audio_tx, &cancel),
                        capture_completion.as_mut(),
                        &app,
                        origin,
                        &cancel,
                    );
                })
                .context("failed to spawn capture thread")?;
            slot.capture = Some(handle);
        }
    }
    Ok(())
}

/// Run a capture thread's work, turning a panic into an error so `finish_producer` always
/// runs. One that unwound past it — wasapi's `ReleaseBuffer(..).unwrap()` on an invalidated
/// device, say — closed the audio channel the way Stop does, and a cloud source went quietly
/// Idle mid-session instead of saying its capture had died. A local source was covered by its
/// completion guard, but only with a generic message; it now records the panic's own.
///
/// `AssertUnwindSafe` is sound here: the device, the audio channel and the meter go down with
/// the closure, and the one thing used after it, the cancellation token, is synchronised
/// internally, so an unwind cannot leave it half-updated.
pub(super) fn catching_panics(produce: impl FnOnce() -> Result<()>) -> Result<()> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(produce)).unwrap_or_else(|payload| {
        let message = payload
            .downcast_ref::<&str>()
            .copied()
            .or_else(|| payload.downcast_ref::<String>().map(String::as_str))
            .unwrap_or("no message");
        Err(anyhow::anyhow!("the capture thread panicked: {message}"))
    })
}

/// A capture thread's whole life: the capture itself, then `finish_producer` whatever way it
/// ended, a panic included.
fn produce(
    capture: impl FnOnce() -> Result<()>,
    completion: Option<&mut CaptureCompletion>,
    events: &impl Events,
    origin: Origin,
    cancel: &CancellationToken,
) {
    finish_producer(catching_panics(capture), completion, events, origin, cancel);
}

/// Settle how a producer — capture thread or rehearsal playback — ended. A local source
/// records the outcome on its spool, where the transcriber picks it up and reports it, so a
/// failure only has to stop that source's capture here. A realtime source has no such reader,
/// so its failure goes to the operator directly. The caller keeps the completion guard, so the
/// local input closes only after this has run.
fn finish_producer(
    result: Result<()>,
    completion: Option<&mut CaptureCompletion>,
    events: &impl Events,
    origin: Origin,
    cancel: &CancellationToken,
) {
    let local = completion.is_some();
    if let Some(completion) = completion {
        completion.finish(result.as_ref().err().map(|e| source_failure(origin, e)));
    }
    if let Err(error) = result {
        if local {
            cancel.cancel();
        } else {
            report_source_failure(events, origin, &error, cancel);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    fn chunk(tag: u8) -> AudioChunk {
        AudioChunk { pcm_le: vec![tag] }
    }

    #[test]
    fn a_panicking_capture_thread_ends_as_an_error_that_says_why() {
        // A formatted message, as `unwrap` and `expect` make: the payload is a `String`.
        let code = "AUDCLNT_E_DEVICE_INVALIDATED";
        let error = catching_panics(|| panic!("ReleaseBuffer failed: {code}")).unwrap_err();
        assert_eq!(
            error.to_string(),
            "the capture thread panicked: ReleaseBuffer failed: AUDCLNT_E_DEVICE_INVALIDATED"
        );
        // A literal: the payload is a `&'static str`.
        let error = catching_panics(|| panic!("device lost")).unwrap_err();
        assert_eq!(
            error.to_string(),
            "the capture thread panicked: device lost"
        );
    }

    /// Ordinary endings pass through untouched: a device error keeps its own type, which
    /// `source_failure` reads to choose the operator's sentence.
    #[test]
    fn a_capture_thread_that_returns_is_passed_through() {
        use crate::audio::capture::MicrophoneRuntimeError;

        assert!(catching_panics(|| Ok(())).is_ok());
        let error = catching_panics(|| {
            Err(anyhow::Error::new(MicrophoneRuntimeError(
                cpal::Error::new(cpal::ErrorKind::DeviceNotAvailable),
            )))
        })
        .unwrap_err();
        assert!(error.downcast_ref::<MicrophoneRuntimeError>().is_some());
    }

    /// B8: a panic in a cloud source's capture thread used to unwind past `finish_producer`,
    /// and the source went quietly Idle mid-session. It is now that source's error.
    #[test]
    fn a_capture_panic_on_a_cloud_source_is_reported_as_its_failure() {
        use crate::errors::id;
        use crate::session::tests::Statuses;
        use crate::types::SessionState;

        let events = Statuses::default();
        let cancel = CancellationToken::new();
        produce(
            || panic!("ReleaseBuffer failed: AUDCLNT_E_DEVICE_INVALIDATED"),
            None,
            &events,
            Origin::System,
            &cancel,
        );
        assert!(cancel.is_cancelled(), "the source's client stops with it");
        let [failure] = &events.take()[..] else {
            panic!("expected exactly one status");
        };
        assert_eq!(failure.state, SessionState::Error);
        assert_eq!(failure.origin, Some(Origin::System));
        let message = failure.message.as_ref().unwrap();
        assert_eq!(message.id, id::SYSTEM_CAPTURE);
        assert_eq!(
            message.detail.as_deref(),
            Some("the capture thread panicked: ReleaseBuffer failed: AUDCLNT_E_DEVICE_INVALIDATED")
        );
    }

    /// A local source's capture outcome goes on its spool, where the transcriber reports it
    /// after draining what was captured, and only then does its input close. A panic records
    /// its own message there rather than the guard's generic one; a clean end records none.
    #[test]
    fn a_local_capture_records_its_outcome_before_its_input_closes() {
        use crate::session::tests::Statuses;
        use tokio::sync::mpsc::error::TryRecvError;

        for (panics, recorded) in [
            (true, Some("the capture thread panicked: device lost")),
            (false, None),
        ] {
            let (sender, mut input) = channel(1);
            let spool = Arc::new(Spool::new().unwrap());
            let cancel = CancellationToken::new();
            let sink = AudioSink::Local {
                sender,
                spool: spool.clone(),
                clock: SessionClock::at(0),
                pause: tokio::sync::watch::channel(false).1,
                cancel: cancel.clone(),
            };
            let mut completion = sink.completion_guard();
            let events = Statuses::default();
            produce(
                move || {
                    drop(sink);
                    assert!(!panics, "device lost");
                    Ok(())
                },
                completion.as_mut(),
                &events,
                Origin::Microphone,
                &cancel,
            );
            assert!(matches!(input.try_recv(), Err(TryRecvError::Empty)));
            let detail = || {
                let error = spool.capture_error.lock().unwrap().clone();
                error.and_then(|e| e.detail)
            };
            assert_eq!(detail().as_deref(), recorded);
            assert_eq!(cancel.is_cancelled(), panics);
            assert!(events.take().is_empty(), "the transcriber reports it");

            drop(completion);
            assert!(matches!(input.try_recv(), Err(TryRecvError::Disconnected)));
            assert_eq!(detail().as_deref(), recorded);
        }
    }

    #[tokio::test]
    async fn the_relay_copies_each_chunk_to_both_languages_and_outlives_one_of_them() {
        let (input_tx, input_rx) = channel(8);
        let (first_tx, mut first) = channel(8);
        let (second_tx, second) = channel(8);
        let source = CancellationToken::new();
        let relay = tokio::spawn(relay_audio(
            input_rx,
            vec![first_tx, second_tx],
            source.clone(),
        ));

        input_tx.send(chunk(1)).await.unwrap();
        drop(second);
        input_tx.send(chunk(2)).await.unwrap();
        assert_eq!(first.recv().await.unwrap().pcm_le, [1]);
        assert_eq!(first.recv().await.unwrap().pcm_le, [2]);
        assert!(!source.is_cancelled(), "one language is still captioning");

        // The last client gone: the relay ends and stops the capture.
        drop(first);
        input_tx.send(chunk(3)).await.unwrap();
        tokio::time::timeout(Duration::from_secs(1), relay)
            .await
            .unwrap()
            .unwrap();
        assert!(source.is_cancelled());
    }

    #[tokio::test]
    async fn the_relay_ends_when_its_source_is_stopped_even_if_capture_has_not() {
        let (_wedged_capture, input_rx) = channel(8);
        let (lane_tx, _lane) = channel(8);
        let source = CancellationToken::new();
        let relay = tokio::spawn(relay_audio(input_rx, vec![lane_tx], source.clone()));
        source.cancel();
        tokio::time::timeout(Duration::from_secs(1), relay)
            .await
            .expect("Stop waited on the capture thread")
            .unwrap();
    }
}
