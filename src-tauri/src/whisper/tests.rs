//! The local pipeline's tests: its endings through a stand-in for Whisper, segmentation, the
//! task's parameters, and the opt-in smoke tests with a real model.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::Result;
use tokio_util::sync::CancellationToken;

use super::backend::*;
use super::models::{self, ModelId, ModelManager};
use super::pipeline::*;
use super::spool::{Spool, TimedChunk};
use super::*;
use crate::errors::{AppError, id};
use crate::realtime::Events;
use crate::types::{Caption, Origin, SessionState, StatusUpdate};

fn speech(start_ms: u64) -> TimedChunk {
    TimedChunk {
        start_ms,
        pcm: 100i16.to_le_bytes().repeat(1600),
    }
}

#[derive(Debug, PartialEq)]
enum Recorded {
    Status(SessionState, Option<&'static str>),
    Caption(String),
    Progress(u64),
}

/// What a local source reports, in order, in place of the app's events.
#[derive(Clone, Default)]
struct Recorder(Arc<Mutex<Vec<Recorded>>>);

impl Recorder {
    fn push(&self, recorded: Recorded) {
        self.0.lock().unwrap().push(recorded);
    }

    fn statuses(&self) -> Vec<Recorded> {
        let recorded = self.0.lock().unwrap();
        let statuses = recorded
            .iter()
            .filter(|r| matches!(r, Recorded::Status(..)));
        statuses
            .map(|r| match r {
                Recorded::Status(state, id) => Recorded::Status(*state, *id),
                _ => unreachable!(),
            })
            .collect()
    }

    fn captions(&self) -> usize {
        let recorded = self.0.lock().unwrap();
        recorded
            .iter()
            .filter(|r| matches!(r, Recorded::Caption(_)))
            .count()
    }

    fn last_progress(&self) -> Option<u64> {
        let recorded = self.0.lock().unwrap();
        recorded.iter().rev().find_map(|r| match r {
            Recorded::Progress(pending_ms) => Some(*pending_ms),
            _ => None,
        })
    }
}

impl Events for Recorder {
    fn caption(&self, caption: Caption<'_>) {
        self.push(Recorded::Caption(caption.text.to_string()));
    }

    fn status(&self, update: StatusUpdate) {
        let id = update.message.map(|message| message.id);
        self.push(Recorded::Status(update.state, id));
    }
}

impl LocalEvents for Recorder {
    fn progress(&self, progress: Progress) {
        self.push(Recorded::Progress(progress.pending_ms));
    }
}

/// A local source with nothing captured yet, and the producer end of its input.
fn pipeline() -> (Pipeline<Recorder>, tokio::sync::mpsc::Sender<TimedChunk>) {
    let (input_tx, input) = tokio::sync::mpsc::channel(300);
    let pipeline = Pipeline {
        events: Recorder::default(),
        origin: Origin::Microphone,
        input,
        spool: Arc::new(Spool::new().unwrap()),
        capture_cancel: CancellationToken::new(),
        abort: CancellationToken::new(),
        abandoned: CancellationToken::new(),
        failure: Arc::new(Mutex::new(None)),
    };
    (pipeline, input_tx)
}

/// Whisper's stand-in: one caption per speech window, through the production pump.
fn echo(
    spool: &Spool,
    abort: &CancellationToken,
    emit: Emit<'_>,
    report: Report<'_>,
) -> Result<()> {
    pump(spool, abort, Duration::from_millis(10), |window| {
        spool.complete(window.consumed);
        if window.speech {
            let end_ms = window.start_ms + window.samples.len() as u64 / 16;
            emit("words", window.start_ms, end_ms, 0);
        }
        report();
        Ok(())
    })
}

fn whisper_error(failure: &Mutex<Option<AppError>>) -> String {
    let failure = failure.lock().unwrap();
    let error = failure.as_ref().expect("the session recorded no failure");
    assert_eq!(error.id, id::WHISPER_SESSION);
    error.detail.clone().unwrap_or_default()
}

const RUNNING: Recorded = Recorded::Status(SessionState::Running, None);
const IDLE: Recorded = Recorded::Status(SessionState::Idle, None);
const FAILED: Recorded = Recorded::Status(SessionState::Error, Some(id::WHISPER_SESSION));

/// Stop closes the input; every frame accepted before it is still captioned, and only then
/// is the source Idle.
#[tokio::test]
async fn stop_drains_every_accepted_frame_then_reports_idle() {
    let (pipeline, input) = pipeline();
    let (events, spool, failure) = (
        pipeline.events.clone(),
        pipeline.spool.clone(),
        pipeline.failure.clone(),
    );
    for i in 0..20 {
        input.send(speech(i * 100)).await.unwrap();
    }
    drop(input);
    drive(pipeline, echo).await;

    assert_eq!(events.statuses(), [RUNNING, IDLE]);
    assert!(events.captions() > 0);
    assert_eq!(spool.pending_ms(), 0);
    assert_eq!(events.last_progress(), Some(0));
    assert!(failure.lock().unwrap().is_none());
}

/// First run on a real Mac: with the System Audio Recording question unanswered, the capture
/// thread waited inside Core Audio and never let go of its input, so the source never saw the
/// end of it and Stop sat on "Stopping…". Once Stop gives up on that thread, the source drains
/// what it has and ends as it would have.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_source_whose_capture_thread_was_left_behind_still_ends() {
    let (pipeline, input) = pipeline();
    let (events, abandoned) = (pipeline.events.clone(), pipeline.abandoned.clone());
    for i in 0..20 {
        input.send(speech(i * 100)).await.unwrap();
    }
    // `input` stays alive, as the wedged thread would keep it.
    let driving = tokio::spawn(drive(pipeline, echo));
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    assert!(!driving.is_finished(), "ended before the input did");
    abandoned.cancel();
    tokio::time::timeout(std::time::Duration::from_secs(10), driving)
        .await
        .expect("the source waited on the abandoned input")
        .unwrap();
    assert_eq!(events.statuses(), [RUNNING, IDLE]);
    drop(input);
}

/// A capture failure recorded on the spool — a capture-thread panic, say (B8) — reaches the
/// operator once what was captured before it has been captioned.
#[tokio::test]
async fn a_capture_failure_is_reported_after_the_audio_before_it() {
    let (pipeline, input) = pipeline();
    let (events, spool, failure) = (
        pipeline.events.clone(),
        pipeline.spool.clone(),
        pipeline.failure.clone(),
    );
    for i in 0..20 {
        input.send(speech(i * 100)).await.unwrap();
    }
    *spool.capture_error.lock().unwrap() = Some(AppError::with(
        id::SYSTEM_CAPTURE,
        "the capture thread panicked: device lost",
    ));
    drop(input);
    drive(pipeline, echo).await;

    assert_eq!(
        events.statuses(),
        [
            RUNNING,
            Recorded::Status(SessionState::Error, Some(id::SYSTEM_CAPTURE))
        ]
    );
    assert!(
        events.captions() > 0,
        "the audio before the failure was dropped"
    );
    // Worded as the capture failure it is, not as a transcription that could not finish.
    assert_eq!(
        failure.lock().unwrap().clone(),
        Some(AppError::with(
            id::SYSTEM_CAPTURE,
            "the capture thread panicked: device lost"
        ))
    );
}

/// A panicking inference worker must still stop capture, or the writer, waiting for input
/// that never ends, would hold Stop forever; and the source must say it failed.
#[tokio::test]
async fn an_inference_panic_stops_capture_and_is_reported() {
    let (pipeline, input) = pipeline();
    let (events, failure, capture) = (
        pipeline.events.clone(),
        pipeline.failure.clone(),
        pipeline.capture_cancel.clone(),
    );
    // As capture does: its producer end goes once its token is cancelled.
    let capture_stopped = capture.clone();
    tokio::spawn(async move {
        capture_stopped.cancelled().await;
        drop(input);
    });
    tokio::time::timeout(
        Duration::from_secs(10),
        drive(pipeline, |_, _, _, _| panic!("ggml aborted")),
    )
    .await
    .expect("the writer was left waiting for input");

    assert!(capture.is_cancelled());
    assert_eq!(events.statuses(), [RUNNING, FAILED]);
    let detail = whisper_error(&failure);
    assert!(
        detail.starts_with("Whisper worker stopped unexpectedly"),
        "{detail}"
    );
}

/// The operator discarding the backlog while Stop waits ends the source in an error that
/// says the transcript is incomplete, not in a clean Idle.
#[tokio::test]
async fn discarding_the_backlog_is_reported_as_an_incomplete_transcript() {
    let (pipeline, input) = pipeline();
    let (events, failure) = (pipeline.events.clone(), pipeline.failure.clone());
    input.send(speech(0)).await.unwrap();
    drop(input);
    pipeline.abort.cancel();
    drive(pipeline, echo).await;

    assert_eq!(events.statuses(), [RUNNING, FAILED]);
    assert_eq!(events.captions(), 0);
    assert_eq!(
        whisper_error(&failure),
        "Remaining audio was discarded; the transcript is incomplete"
    );
}

#[test]
fn inference_threads_leave_room_for_capture_and_stop_at_eight() {
    if cfg!(target_arch = "x86_64") {
        assert_eq!(
            [1, 2, 4, 8, 12, 16, 32].map(inference_threads),
            [1, 1, 2, 4, 6, 8, 8]
        );
    } else {
        assert_eq!(
            [1, 2, 4, 8, 12, 16].map(inference_threads),
            [1, 1, 3, 7, 8, 8]
        );
    }
}

#[test]
fn the_task_reaches_whisper_with_the_spoken_language_still_set() {
    // whisper-rs exposes no getters; its Debug output shows the native parameters.
    for translate in [false, true] {
        let params = format!("{:?}", engine::decode_params(Some("fr"), translate, 4, &[]));
        assert!(
            params.contains(&format!("translate: {translate},")),
            "{params}"
        );
        assert!(params.contains("n_threads: 4,"), "{params}");
        assert!(params.contains("detect_language: false,"), "{params}");
        assert!(params.contains("prompt_n_tokens: 0,"), "{params}");
    }
}

#[test]
fn the_glossary_prompt_reaches_every_window() {
    let params = format!("{:?}", engine::decode_params(None, false, 4, &[50, 51, 52]));
    assert!(params.contains("prompt_n_tokens: 3,"), "{params}");
    // The text of earlier windows still stays out.
    assert!(params.contains("no_context: true,"), "{params}");
}

#[test]
fn quiet_input_releases_held_speech_without_waiting_for_more_audio() {
    let spool = Arc::new(Spool::new().unwrap());
    for i in 0..20 {
        spool.push(&speech(i * 100)).unwrap();
    }
    let (sent, windows) = std::sync::mpsc::channel();
    let reader = spool.clone();
    let worker = std::thread::spawn(move || {
        pump(
            &reader,
            &CancellationToken::new(),
            Duration::from_millis(100),
            |window| {
                reader.complete(window.consumed);
                sent.send((window.start_ms, window.consumed, window.speech))
                    .unwrap();
                Ok(())
            },
        )
    });
    // Input is still open, as during Pause: only going idle can release these two seconds.
    let window = windows.recv_timeout(Duration::from_secs(10)).unwrap();
    assert_eq!(window, (0, 20 * 1600, true));
    assert_eq!(spool.pending_ms(), 0);
    // Resuming after the pause starts a new window at its own time.
    spool.push(&speech(9_000)).unwrap();
    let window = windows.recv_timeout(Duration::from_secs(10)).unwrap();
    assert_eq!(window, (9_000, 1600, true));
    spool.finish();
    worker.join().unwrap().unwrap();
    assert!(windows.try_recv().is_err());
}

#[test]
fn a_backlog_is_worked_through_in_long_windows_then_normal_ones() {
    let spool = Spool::new().unwrap();
    for i in 0..400 {
        spool.push(&speech(i * 100)).unwrap();
    }
    spool.finish();
    let mut cuts = Vec::new();
    pump(
        &spool,
        &CancellationToken::new(),
        Duration::from_secs(5),
        |window| {
            spool.complete(window.consumed);
            cuts.push((window.start_ms, window.samples.len()));
            Ok(())
        },
    )
    .unwrap();
    // 40 s queued: one 28 s window, after which 13 s remain and 10 s windows resume.
    assert_eq!(
        cuts,
        [
            (0, 28 * 16_000),
            (27_000, 10 * 16_000),
            (36_000, 4 * 16_000)
        ]
    );
    assert_eq!(spool.pending_ms(), 0);
}

#[test]
fn discarding_stops_the_pump_without_flushing() {
    let spool = Spool::new().unwrap();
    spool.push(&speech(0)).unwrap();
    spool.finish();
    let abort = CancellationToken::new();
    abort.cancel();
    let mut windows = 0;
    pump(&spool, &abort, Duration::from_millis(10), |_| {
        windows += 1;
        Ok(())
    })
    .unwrap();
    assert_eq!(windows, 0);
}

#[test]
fn advertised_languages_match_the_native_multilingual_vocabulary() {
    let choices: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("../../../src/lib/whisperLanguages.json")).unwrap();
    let mut ids = std::collections::HashSet::new();
    for choice in choices {
        let code = choice["code"].as_str().unwrap();
        validate_language(Some(code)).unwrap();
        assert!(ids.insert(whisper_rs::get_lang_id(code).unwrap()));
    }
    assert_eq!(ids.len(), 99);
}

/// Put the pinned Tiny model in `dir`. Downloading it costs CI 40 to 100 s a lane, so when
/// `WHISPER_SMOKE_MODEL_CACHE` names a directory, which CI restores between runs, a copy
/// there that matches the pin is used instead, and a download refills it. The download
/// itself stays covered, against a local server, by the tests in `models.rs`.
async fn smoke_model(manager: &ModelManager, dir: &std::path::Path) {
    let id = ModelId::Tiny;
    let Some(cache) = std::env::var_os("WHISPER_SMOKE_MODEL_CACHE").filter(|path| !path.is_empty())
    else {
        manager.download(dir, id).await.unwrap();
        return;
    };
    let cache = std::path::PathBuf::from(cache);
    let cached = cache.join(id.file());
    let started = std::time::Instant::now();
    match models::read_verified(&cached, id) {
        Ok(bytes) => {
            // The bytes just verified, rather than a second read of the file.
            std::fs::write(dir.join(id.file()), bytes).unwrap();
            eprintln!("{} ({:.2?})", cached.display(), started.elapsed());
            return;
        }
        Err(error) => eprintln!("{}: {error:#}; downloading", cached.display()),
    }
    manager.download(dir, id).await.unwrap();
    eprintln!("{} downloaded ({:.2?})", id.file(), started.elapsed());
    std::fs::create_dir_all(&cache).expect("WHISPER_SMOKE_MODEL_CACHE must be writable");
    std::fs::copy(dir.join(id.file()), &cached)
        .expect("WHISPER_SMOKE_MODEL_CACHE must be writable");
}

/// The bundled speech-detection model, from the source tree.
fn bundled_vad() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("resources")
        .join(vad::RESOURCE)
}

/// Explicit opt-in: downloads the 31 MiB pinned model (or takes it from CI's cache), then
/// exercises the production spool, segmentation, speech detection, inference and
/// EOF-draining path against both bundled fixtures, and translates the French one into
/// English with its language detected. CI runs this on native x64 and ARM64 on the
/// processor, and on Apple silicon with Metal, without a microphone or provider key.
#[tokio::test]
#[ignore = "downloads a Whisper model; run with --ignored local_whisper_smoke"]
async fn local_whisper_smoke() {
    let placement = Placement {
        metal: cfg!(target_os = "macos"),
        helper: None,
        allow_software: false,
        abort: &CancellationToken::new(),
    };
    let expected = if cfg!(target_os = "macos") {
        "metal"
    } else {
        "cpu"
    };
    smoke(&placement, expected).await;
}

/// The same, through the Vulkan helper named by `WHISPER_VULKAN_HELPER` (built with
/// `cargo build -p whisper-vulkan`). A software renderer is allowed, so CI's Linux lane runs
/// it on Mesa's lavapipe with `GGML_VK_VISIBLE_DEVICES=0`: slow, but the whole path.
#[tokio::test]
#[ignore = "needs the Vulkan helper; run with --ignored local_whisper_vulkan_smoke"]
async fn local_whisper_vulkan_smoke() {
    let helper = PathBuf::from(
        std::env::var_os("WHISPER_VULKAN_HELPER").expect("set WHISPER_VULKAN_HELPER"),
    );
    let placement = Placement {
        metal: false,
        helper: Some(&helper),
        allow_software: true,
        abort: &CancellationToken::new(),
    };
    smoke(&placement, "vulkan: ").await;
}

async fn smoke(placement: &Placement<'_>, expected_device: &str) {
    let temp = tempfile::tempdir().unwrap();
    // Non-ASCII on purpose: a Windows username can put such characters in the real path.
    let dir = temp.path().join("réunion-日本語");
    std::fs::create_dir(&dir).unwrap();
    let manager = ModelManager::default();
    smoke_model(&manager, &dir).await;
    let model = load_lease(
        manager.lease(&dir, ModelId::Tiny).unwrap(),
        Some(&bundled_vad()),
        placement,
    )
    .unwrap();
    assert!(model.vad.is_some(), "speech detection did not load");
    let device = model.device();
    eprintln!("device: {device}");
    assert!(device.starts_with(expected_device), "{device}");
    // The names-and-terms prompt, tokenized by the real model: the top of a long list fits.
    let long: Vec<String> = (0..100).map(|i| format!("Université numéro {i}")).collect();
    let prompt = glossary_prompt(&model, &long);
    assert!(
        prompt.len() > 10 && prompt.len() < long.len(),
        "{}",
        prompt.len()
    );
    assert!(prompt.tokens.len() <= 223);
    let glossary = ["IWAC".to_string(), "Voxtral".to_string()];
    for (fixture, language, translate, expected, terms) in [
        ("rehearsal-en.wav", None, false, Some("caption"), &[][..]),
        // The same recording with a prompt still reads as what was said.
        (
            "rehearsal-en.wav",
            None,
            false,
            Some("caption"),
            &glossary[..],
        ),
        ("rehearsal-fr.wav", Some("fr"), false, Some("public"), &[]),
        // Tiny translates poorly, and its wording moves with the instruction set: ARM64 said
        // "This is a recording of repetition…", x64 "This is an arrangement of repetition…".
        // So the translation is held to being English, not to any one word.
        ("rehearsal-fr.wav", None, true, None, &[]),
    ] {
        let samples = crate::audio::fixture::load_fixture(
            &std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("resources/fixtures")
                .join(fixture),
        )
        .unwrap();
        let started = std::time::Instant::now();
        let text = decode_samples(
            &model,
            &samples,
            &Task {
                language,
                translate,
                glossary: terms,
            },
        );
        // The time is the one number CI can compare across instruction-set changes.
        eprintln!("{fixture} ({:.2?}): {text}", started.elapsed());
        match expected {
            Some(word) => assert!(text.to_lowercase().contains(word), "{fixture}: {text}"),
            None => assert!(reads_as_english(&text), "{fixture}: {text}"),
        }
    }
    quiet_audio_with_a_prompt(&model, &glossary);
}

/// The names-and-terms prompt over audio with little or no speech in it, where Whisper is
/// known to write the prompt back: OpenWhispr found speech detection that strips most of a
/// pause-heavy recording left Whisper decoding near-silence seeded with the prompt, and its
/// terms became the transcript. Here, silence and faint noise caption nothing made of the
/// terms, and speech broken by long pauses still reads as what was said.
fn quiet_audio_with_a_prompt(model: &LoadedModel, glossary: &[String]) {
    let task = Task {
        language: Some("en"),
        translate: false,
        glossary,
    };
    let only_terms = |text: &str| {
        let words: Vec<String> = text.split_whitespace().map(reconcile::normalise).collect();
        let terms: Vec<String> = glossary.iter().map(|t| reconcile::normalise(t)).collect();
        words.iter().any(|w| !w.is_empty())
            && words.iter().all(|w| w.is_empty() || terms.contains(w))
    };
    let silence = vec![0.0_f32; 16_000 * 25];
    let text = decode_samples(model, &silence, &task);
    eprintln!("silence with a prompt: {text:?}");
    assert!(text.trim().is_empty(), "silence with a prompt: {text}");

    // Faint, steady noise, about -50 dBFS: a quiet room with nobody speaking.
    let mut seed = 0x2545_f491_u32;
    let noise: Vec<f32> = (0..16_000 * 25)
        .map(|_| {
            seed ^= seed << 13;
            seed ^= seed >> 17;
            seed ^= seed << 5;
            #[expect(clippy::cast_precision_loss, reason = "noise needs no precision")]
            let unit = seed as f32 / u32::MAX as f32;
            (unit - 0.5) * 0.006
        })
        .collect();
    let text = decode_samples(model, &noise, &task);
    eprintln!("noise with a prompt: {text:?}");
    assert!(
        !only_terms(&text),
        "noise with a prompt read the terms back: {text}"
    );

    // The English rehearsal with six seconds of silence after every three of speech.
    let speech = crate::audio::fixture::load_fixture(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("resources/fixtures/rehearsal-en.wav"),
    )
    .unwrap();
    let paused: Vec<f32> = speech
        .chunks(16_000 * 3)
        .flat_map(|part| {
            part.iter()
                .copied()
                .chain(std::iter::repeat_n(0.0, 16_000 * 6))
        })
        .collect();
    let text = decode_samples(model, &paused, &task);
    eprintln!("paused speech with a prompt: {text}");
    assert!(
        text.to_lowercase().contains("caption"),
        "paused speech: {text}"
    );
    assert!(
        !only_terms(&text),
        "paused speech read the terms back: {text}"
    );
}

/// Run 16 kHz samples through the production spool, segmentation and inference, as a
/// source would, and return the captions joined.
fn decode_samples(model: &LoadedModel, samples: &[f32], task: &Task<'_>) -> String {
    let spool = Spool::new().unwrap();
    for (i, chunk) in samples.chunks(1600).enumerate() {
        #[expect(
            clippy::cast_possible_truncation,
            reason = "the samples are 16-bit PCM over 32768, so this restores them exactly"
        )]
        let pcm = chunk
            .iter()
            .flat_map(|s| ((*s * 32768.0) as i16).to_le_bytes())
            .collect();
        spool
            .push(&TimedChunk {
                start_ms: i as u64 * 100,
                pcm,
            })
            .unwrap();
    }
    spool.finish();
    let mut text = String::new();
    let mut last_end = 0;
    transcribe(
        model,
        task,
        &spool,
        &CancellationToken::new(),
        |caption, start, end, _| {
            assert!(start >= last_end && end >= start);
            assert!(end <= samples.len() as u64 / 16);
            last_end = end;
            text.push_str(caption);
            text.push(' ');
        },
        || {},
    )
    .unwrap();
    assert_eq!(
        spool.pending_ms(),
        0,
        "Stop must drain the final partial window"
    );
    text
}

/// Whether `text` is English rather than French, judged by function words, which survive
/// any rewording: plenty of English ones and few French ones.
fn reads_as_english(text: &str) -> bool {
    const ENGLISH: [&str; 7] = ["the", "is", "and", "to", "of", "this", "a"];
    const FRENCH: [&str; 9] = ["le", "la", "les", "est", "et", "une", "des", "sont", "du"];
    let lower = text.to_lowercase();
    let words: Vec<&str> = lower
        .split(|c: char| !c.is_alphabetic())
        .filter(|w| !w.is_empty())
        .collect();
    let count = |set: &[&str]| words.iter().filter(|w| set.contains(w)).count();
    let (english, french) = (count(&ENGLISH), count(&FRENCH));
    english >= 5 && english > 3 * french
}

#[test]
fn english_is_told_from_french_by_its_function_words() {
    // What Tiny produced on the two CI architectures, and the French it was translating.
    assert!(reads_as_english(concat!(
        "This is a recording of repetition for the subject system to direct. ",
        "A portrait is a public and each sentence is translated later. to the screen."
    )));
    assert!(reads_as_english(concat!(
        "This is an arrangement of repetition for the subject system to direct. ",
        "A portrait is a public and each sentence is translated later. ",
        "to the screen, then to the screen."
    )));
    assert!(!reads_as_english(concat!(
        "Ceci est un enregistrement de répétition pour le système de sous-titrage en direct. ",
        "Une oratrice s'adresse à un public et chaque phrase est transcrite puis affichée à ",
        "l'écran. Les sous-titres apparaissent juste après les mots."
    )));
}

#[test]
fn validates_multilingual_choices_and_rejects_injection_or_unavailable_tokens() {
    for language in [
        None,
        Some("fr"),
        Some("en"),
        Some("ar"),
        Some("de"),
        Some("sw"),
        Some("ja"),
    ] {
        validate_language(language).unwrap();
    }
    for language in ["fake", "yue", "fr\0suffix"] {
        assert!(validate_language(Some(language)).is_err());
    }
}
