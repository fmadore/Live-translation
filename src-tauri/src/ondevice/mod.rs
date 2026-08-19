//! Keyless on-device captioning.
//!
//! This is the one caption path that needs no credential: audio never leaves the machine and
//! no provider key is involved. It exists for three reasons — an offline fallback when the
//! venue network fails, a first run that captions immediately instead of demanding an API
//! key, and Microsoft Store policy 10.8.3, which bars individual publishers from requiring
//! "API secret keys" for a product's *primary* functionality. See
//! `docs/microsoft-store.md`.
//!
//! Same-language only. Windows exposes no on-device translation API, so translated captions
//! remain the cloud providers' job.
//!
//! The driver here is deliberately engine-agnostic: it owns audio consumption, backpressure,
//! turn bookkeeping and status reporting, exactly as `realtime.rs` does for the WebSocket
//! providers, and delegates recognition itself to a [`Recognizer`]. Swapping the recognizer
//! (see `engine.rs`) touches nothing else.

use anyhow::Result;
use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc::Receiver;
use tokio_util::sync::CancellationToken;

use crate::audio::AudioChunk;
use crate::realtime::{emit_caption, TurnAccumulator};
use crate::types::{events, Origin, SessionState, StatusUpdate};

mod engine;
mod whisper;

/// How many chunks may queue between the async driver and the recognition thread before the
/// driver starts discarding the oldest. Recognition is CPU-bound and may transiently fall
/// behind; live captions are worth more than complete ones, so drop rather than lag.
const RECOGNIZER_QUEUE: usize = 8;

pub struct OnDeviceConfig {
    pub origin: Origin,
    /// BCP-47 hint for the recognizer (e.g. `en-US`). `None` follows the system language.
    pub language_tag: Option<String>,
}

/// What a recognizer reports back as audio is fed to it.
///
/// `dead_code` is allowed because the variants are constructed only by a [`Recognizer`]
/// implementation, and no engine is wired up yet — see `engine.rs` for the pending choice.
/// Remove the attribute along with that decision.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecognitionEvent {
    /// Interim text for the utterance in progress. Replaces any previous partial.
    Partial(String),
    /// The utterance is complete. The next event starts a new turn.
    Final(String),
}

/// A local speech recognizer driven by pushed PCM.
///
/// Implementations run on a dedicated native thread — recognition is blocking and CPU-bound,
/// so it must not sit on the async runtime. Audio arrives as mono PCM-16 at 16 kHz, the rate
/// `Provider::OnDevice.input_sample_rate()` pins the capture pipeline to.
pub trait Recognizer: Send {
    /// Engine name, for logs and operator-facing status text.
    fn name(&self) -> &'static str;

    /// Feed one chunk and return whatever it produced. Returning an empty vector is normal:
    /// most chunks land mid-utterance.
    fn accept(&mut self, samples: &[i16]) -> Result<Vec<RecognitionEvent>>;

    /// Flush at end of session so a trailing utterance is not lost.
    fn finish(&mut self) -> Result<Vec<RecognitionEvent>> {
        Ok(Vec::new())
    }
}

/// Decode one captured chunk into samples. Capture always produces whole little-endian
/// frames; a trailing odd byte would mean a truncated chunk, so ignore it rather than
/// fabricating a sample from it.
fn decode_pcm16(pcm_le: &[u8]) -> Vec<i16> {
    pcm_le
        .chunks_exact(2)
        .map(|pair| i16::from_le_bytes([pair[0], pair[1]]))
        .collect()
}

/// Run one on-device caption session for a single origin. Mirrors
/// `realtime::run_session`'s signature so `session.rs` treats every backend alike.
pub async fn run_session(
    app: AppHandle,
    config: OnDeviceConfig,
    mut audio_rx: Receiver<AudioChunk>,
    cancel: CancellationToken,
) {
    let origin = config.origin;
    emit_status(&app, SessionState::Connecting, None, origin);

    // Engine construction can be slow — a first-run model load, or a download. Keep it off
    // the async runtime and cancellable.
    let language_tag = config.language_tag.clone();
    let engine_app = app.clone();
    let recognizer = tokio::select! {
        _ = cancel.cancelled() => return,
        built = tauri::async_runtime::spawn_blocking(move || {
            engine::new_recognizer(&engine_app, language_tag.as_deref())
        }) => built,
    };

    let recognizer = match recognizer {
        Ok(Ok(recognizer)) => recognizer,
        Ok(Err(error)) => {
            tracing::error!(?origin, "on-device recognizer unavailable: {error:#}");
            emit_status(&app, SessionState::Error, Some(format!("{error}")), origin);
            return;
        }
        Err(error) => {
            tracing::error!(?origin, "on-device recognizer task failed: {error}");
            emit_status(
                &app,
                SessionState::Error,
                Some("on-device recognizer failed to start".into()),
                origin,
            );
            return;
        }
    };

    let engine_name = recognizer.name();
    tracing::info!(?origin, engine = engine_name, "on-device recognizer ready");

    // Audio crosses to a native thread; events come back. Both are bounded so neither side
    // can grow without limit if the other stalls.
    let (sample_tx, sample_rx) = std::sync::mpsc::sync_channel::<Vec<i16>>(RECOGNIZER_QUEUE);
    let (event_tx, mut event_rx) = tokio::sync::mpsc::channel::<RecognitionEvent>(RECOGNIZER_QUEUE);

    let worker_origin = origin;
    let worker = std::thread::Builder::new()
        .name(format!("ondevice-{origin:?}"))
        .spawn(move || recognition_loop(recognizer, sample_rx, event_tx, worker_origin));
    let worker = match worker {
        Ok(handle) => handle,
        Err(error) => {
            tracing::error!(?origin, "failed to spawn recognition thread: {error}");
            emit_status(
                &app,
                SessionState::Error,
                Some("failed to start the on-device recognizer".into()),
                origin,
            );
            return;
        }
    };

    emit_status(&app, SessionState::Running, None, origin);

    let mut acc = TurnAccumulator::default();
    // Dropping `sample_tx` is what tells the worker to flush and exit, so it lives in an
    // Option and is released before the thread is joined.
    let mut sample_tx = Some(sample_tx);

    loop {
        tokio::select! {
            _ = cancel.cancelled() => break,

            maybe_chunk = audio_rx.recv() => {
                let Some(chunk) = maybe_chunk else { break };
                let Some(tx) = sample_tx.as_ref() else { break };
                let samples = decode_pcm16(&chunk.pcm_le);
                if samples.is_empty() {
                    continue;
                }
                match tx.try_send(samples) {
                    Ok(()) => {}
                    Err(std::sync::mpsc::TrySendError::Full(_)) => {
                        // Recognition is behind. Discarding the newest chunk keeps the
                        // already-queued audio contiguous, which matters: a recognizer fed a
                        // stream with a hole mid-word produces worse text than one that
                        // simply lags a little and recovers.
                        tracing::debug!(?origin, "on-device recognizer behind; dropped a chunk");
                    }
                    Err(std::sync::mpsc::TrySendError::Disconnected(_)) => break,
                }
            }

            maybe_event = event_rx.recv() => {
                let Some(event) = maybe_event else { break };
                apply_event(&app, origin, &mut acc, event);
            }
        }
    }

    // Release the sender so the worker sees the end of the stream, flushes, and exits.
    drop(sample_tx.take());

    // Drain whatever the flush produced before reporting the session finished.
    while let Some(event) = event_rx.recv().await {
        apply_event(&app, origin, &mut acc, event);
    }

    if tauri::async_runtime::spawn_blocking(move || worker.join())
        .await
        .map(|joined| joined.is_err())
        .unwrap_or(true)
    {
        tracing::warn!(?origin, "on-device recognition thread did not exit cleanly");
    }

    if !acc.is_empty() {
        emit_caption(&app, origin, &acc, true);
        acc.next_turn();
    }
    tracing::info!(?origin, engine = engine_name, "on-device session ended");
}

/// Blocking recognition loop. Owns the recognizer for the whole session so engines that hold
/// native state (a model handle, a COM apartment) never cross threads.
fn recognition_loop(
    mut recognizer: Box<dyn Recognizer>,
    sample_rx: std::sync::mpsc::Receiver<Vec<i16>>,
    event_tx: tokio::sync::mpsc::Sender<RecognitionEvent>,
    origin: Origin,
) {
    while let Ok(samples) = sample_rx.recv() {
        match recognizer.accept(&samples) {
            Ok(events) => {
                for event in events {
                    if event_tx.blocking_send(event).is_err() {
                        return;
                    }
                }
            }
            Err(error) => {
                tracing::error!(?origin, "on-device recognition failed: {error:#}");
                return;
            }
        }
    }

    match recognizer.finish() {
        Ok(events) => {
            for event in events {
                if event_tx.blocking_send(event).is_err() {
                    return;
                }
            }
        }
        Err(error) => tracing::warn!(?origin, "on-device recognizer flush failed: {error:#}"),
    }
}

/// Fold one recognition event into the turn accumulator.
///
/// Returns `Some(final_)` when the caller should emit a caption. Kept free of `AppHandle` so
/// the turn bookkeeping — the part that is easy to get wrong and hard to notice — is unit
/// tested without standing up a Tauri app.
///
/// Subtitles are the audience text itself, so they go in `translated` and `source` stays
/// empty: the same shape the Mistral transcription path uses, which keeps the operator
/// monitor and the text/Markdown export working unchanged.
fn fold_event(acc: &mut TurnAccumulator, event: RecognitionEvent) -> Option<bool> {
    match event {
        RecognitionEvent::Partial(text) if text.trim().is_empty() => None,
        RecognitionEvent::Partial(text) => {
            acc.translated = text;
            Some(false)
        }
        // An empty final still closes a turn that a partial already opened, so a trailing
        // hypothesis is committed to the transcript rather than left dangling.
        RecognitionEvent::Final(text) if text.trim().is_empty() => {
            (!acc.is_empty()).then_some(true)
        }
        RecognitionEvent::Final(text) => {
            acc.translated = text;
            Some(true)
        }
    }
}

fn apply_event(
    app: &AppHandle,
    origin: Origin,
    acc: &mut TurnAccumulator,
    event: RecognitionEvent,
) {
    if let Some(final_) = fold_event(acc, event) {
        emit_caption(app, origin, acc, final_);
        if final_ {
            acc.next_turn();
        }
    }
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

/// Map the caption language onto a recognizer hint. Same-language transcription has no
/// translation target, so this only tells the engine which language to expect.
pub fn language_tag(target: crate::types::TargetLanguage) -> Option<String> {
    Some(
        match target {
            crate::types::TargetLanguage::En => "en-US",
            crate::types::TargetLanguage::Fr => "fr-FR",
        }
        .to_string(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn partial(text: &str) -> RecognitionEvent {
        RecognitionEvent::Partial(text.to_string())
    }

    fn final_(text: &str) -> RecognitionEvent {
        RecognitionEvent::Final(text.to_string())
    }

    #[test]
    fn decodes_little_endian_frames() {
        let bytes = [0x01, 0x00, 0xff, 0xff, 0x00, 0x01];
        assert_eq!(decode_pcm16(&bytes), vec![1, -1, 256]);
    }

    #[test]
    fn ignores_a_truncated_trailing_byte() {
        let bytes = [0x01, 0x00, 0x7f];
        assert_eq!(decode_pcm16(&bytes), vec![1]);
    }

    #[test]
    fn partials_replace_rather_than_append() {
        let mut acc = TurnAccumulator::default();
        assert_eq!(fold_event(&mut acc, partial("bon")), Some(false));
        assert_eq!(fold_event(&mut acc, partial("bonjour tout")), Some(false));
        // A recognizer re-sends the whole hypothesis each time; appending would produce
        // "bonbonjour tout".
        assert_eq!(acc.translated, "bonjour tout");
        assert_eq!(acc.id, 0, "a partial must not advance the turn");
    }

    #[test]
    fn final_commits_and_advances_the_turn() {
        let mut acc = TurnAccumulator::default();
        fold_event(&mut acc, partial("bonjour"));
        assert_eq!(
            fold_event(&mut acc, final_("bonjour tout le monde")),
            Some(true)
        );
        assert_eq!(acc.translated, "bonjour tout le monde");

        // The caller emits, then advances; the next turn starts clean under a new id.
        acc.next_turn();
        assert_eq!(acc.id, 1);
        assert!(acc.is_empty());
        assert_eq!(fold_event(&mut acc, partial("deuxième")), Some(false));
        assert_eq!(acc.translated, "deuxième");
    }

    #[test]
    fn empty_partials_are_ignored() {
        let mut acc = TurnAccumulator::default();
        assert_eq!(fold_event(&mut acc, partial("   ")), None);
        assert!(acc.is_empty());
    }

    #[test]
    fn empty_final_closes_an_open_turn_but_not_an_idle_one() {
        let mut acc = TurnAccumulator::default();
        // Nothing in flight: silence must not emit an empty caption.
        assert_eq!(fold_event(&mut acc, final_("")), None);

        // With a hypothesis in flight, the same event commits it.
        fold_event(&mut acc, partial("merci"));
        assert_eq!(fold_event(&mut acc, final_("")), Some(true));
        assert_eq!(acc.translated, "merci");
    }

    #[test]
    fn subtitles_go_in_the_audience_field_and_leave_source_empty() {
        // The transcript export and operator monitor rely on this shape, matching Mistral.
        let mut acc = TurnAccumulator::default();
        fold_event(&mut acc, final_("same-language text"));
        assert_eq!(acc.translated, "same-language text");
        assert!(acc.source.is_empty());
    }

    #[test]
    fn language_hints_are_bcp47() {
        use crate::types::TargetLanguage;
        assert_eq!(language_tag(TargetLanguage::En).as_deref(), Some("en-US"));
        assert_eq!(language_tag(TargetLanguage::Fr).as_deref(), Some("fr-FR"));
    }
}
