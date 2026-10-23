//! Wiring one session's sources while `start` spawns them: each source's producer, its audio
//! channel, and the client — or, with a second caption language, the relay and two clients —
//! reading it.

use std::sync::Arc;

use anyhow::{Context, Result};
use tauri::async_runtime::JoinHandle as AsyncJoinHandle;
use tauri::AppHandle;
use tokio::sync::mpsc::{channel, error::TrySendError, Receiver, Sender};
use tokio_util::sync::CancellationToken;

use super::capture::{report_source_failure, CaptureTarget};
use super::options::caption_languages;
use super::settings::ProviderSettings;
use super::ActiveSession;
use crate::audio::fixture::run_rehearsal;
use crate::audio::sink::{AudioSink, CaptureCompletion};
use crate::audio::AudioChunk;
use crate::realtime::{run_session, Events, PauseRx, RealtimeProtocol};
use crate::timing::SessionClock;
use crate::types::{AudioLevel, Origin, Provider, StartOptions, TargetLanguage};
use crate::whisper::{self, spool::Spool, LoadedModel, LocalSession};

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
        let mut open = 0;
        for output in &outputs {
            if !matches!(output.try_send(chunk.clone()), Err(TrySendError::Closed(_))) {
                open += 1;
            }
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
            self.lane,
        ))
    }
}

/// Collects one session's sources while `start` spawns them.
pub(super) struct SessionBuilder<'a> {
    pub(super) app: &'a AppHandle,
    pub(super) options: &'a StartOptions,
    pub(super) settings: ProviderSettings,
    pub(super) api_key: String,
    pub(super) capture: CaptureTarget,
    pub(super) target_rate: u32,
    pub(super) clock: SessionClock,
    pub(super) level_tx: Sender<AudioLevel>,
    pub(super) session: ActiveSession,
    pub(super) local_model: Option<Arc<LoadedModel>>,
}

impl SessionBuilder<'_> {
    pub(super) fn add_source(&mut self, origin: Origin) -> Result<()> {
        if self.options.provider.is_local_whisper() {
            return self.add_local_source(origin);
        }
        let (audio_tx, audio_rx) = channel::<AudioChunk>(AUDIO_CHANNEL_CAPACITY);
        let cancel = self.session.cancel.child_token();
        self.session.sources.push(cancel.clone());
        self.spawn_producer(origin, audio_tx.into(), &cancel)?;

        let targets = caption_languages(self.options);
        if targets.len() == 1 {
            // One language: the client reads the producer directly and owns the source's
            // token, so its end stops the capture, as it always has.
            return self.add_client(origin, 0, targets[0], audio_rx, cancel);
        }
        // Two languages: each client gets its own copy of the audio and a child token, so
        // one failing leaves the other captioning. The relay stops the capture when both
        // have gone.
        let mut outputs = Vec::with_capacity(targets.len());
        for (lane, target) in (0u8..).zip(targets) {
            let (lane_tx, lane_rx) = channel::<AudioChunk>(AUDIO_CHANNEL_CAPACITY);
            outputs.push(lane_tx);
            self.add_client(origin, lane, target, lane_rx, cancel.child_token())?;
        }
        self.session
            .relay_tasks
            .push(tauri::async_runtime::spawn(relay_audio(
                audio_rx, outputs, cancel,
            )));
        Ok(())
    }

    fn add_client(
        &mut self,
        origin: Origin,
        lane: u8,
        target: TargetLanguage,
        audio_rx: Receiver<AudioChunk>,
        cancel: CancellationToken,
    ) -> Result<()> {
        let io = ClientIo {
            app: self.app.clone(),
            origin,
            lane,
            audio_rx,
            cancel,
            clock: self.clock,
            pause: self.session.pause.subscribe(),
        };
        let client = self.settings.spawn_client(io, &self.api_key, target)?;
        self.session.client_tasks.push(client);
        Ok(())
    }

    fn add_local_source(&mut self, origin: Origin) -> Result<()> {
        // Thirty seconds of scheduling slack for the disk writer. Inference never consumes
        // this queue; it reads its own spool and may lag by much longer without losing audio.
        let (sender, input) = channel(300);
        let spool = Arc::new(Spool::new()?);
        let cancel = self.session.cancel.child_token();
        self.session.sources.push(cancel.clone());
        let sink = AudioSink::Local {
            sender,
            spool: spool.clone(),
            clock: self.clock,
            pause: self.session.pause.subscribe(),
            cancel: cancel.clone(),
        };
        self.spawn_producer(origin, sink, &cancel)?;
        let model = self
            .local_model
            .clone()
            .context("Whisper model was not loaded")?;
        self.session
            .client_tasks
            .push(tauri::async_runtime::spawn(whisper::run(LocalSession {
                app: self.app.clone(),
                model,
                language: self.options.spoken_language.clone(),
                // The only difference between the two Whisper providers.
                translate: self.options.provider.can_translate(),
                origin,
                input,
                spool,
                capture_cancel: cancel,
                abort: self.session.abort.clone(),
                failure: self.session.failure.clone(),
            })));
        Ok(())
    }

    /// Start whatever feeds one source's audio channel.
    fn spawn_producer(
        &mut self,
        origin: Origin,
        audio_tx: AudioSink,
        cancel: &CancellationToken,
    ) -> Result<()> {
        if self.options.provider == Provider::OnDevice {
            // The deterministic demo emits its own level/caption timeline and never opens a
            // capture device. Close the unused producer immediately.
            drop(audio_tx);
            return Ok(());
        }

        let app = self.app.clone();
        let mut capture_completion = audio_tx.completion_guard();
        let level_tx = self.level_tx.clone();
        let target_rate = self.target_rate;
        let cancel = cancel.clone();
        match self.options.rehearsal {
            // Rehearsal swaps the capture device for a bundled recording and changes nothing
            // else: same channel, same chunk shape, same engine below it.
            Some(language) => {
                self.session
                    .fixture_tasks
                    .push(tauri::async_runtime::spawn(async move {
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
                let capture = self.capture.clone();
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
                self.session.capture_threads.push(handle);
            }
        }
        Ok(())
    }
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
fn catching_panics(produce: impl FnOnce() -> Result<()>) -> Result<()> {
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
        completion.finish(result.as_ref().err().map(|e| format!("{e:#}")));
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
            assert_eq!(spool.capture_error.lock().unwrap().as_deref(), recorded);
            assert_eq!(cancel.is_cancelled(), panics);
            assert!(events.take().is_empty(), "the transcriber reports it");

            drop(completion);
            assert!(matches!(input.try_recv(), Err(TryRecvError::Disconnected)));
            assert_eq!(spool.capture_error.lock().unwrap().as_deref(), recorded);
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
