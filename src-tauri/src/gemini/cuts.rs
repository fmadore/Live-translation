//! Where Transcribe Live's segments end, when the client chooses (issue #281).
//!
//! Left to itself, Transcribe Live cuts continuous speech every 8 to 11 seconds whether or not
//! the speaker pauses, and the audio at each cut reaches no text (`docs/gemini-live-api.md`,
//! "Segmentation is the model's"). With [`Segmentation::Client`] the setup turns Gemini's
//! automatic activity detection off, and the client marks the segments itself with
//! `activityStart` and `activityEnd`, at pauses the bundled Silero model finds.
//!
//! - **Every chunk is inside an activity.** The first chunk on a connection is preceded by
//!   `activityStart`, and a cut is `activityEnd` then `activityStart` straight away, between two
//!   chunks. Silero only places the cuts: a word it misses is still sent, and still inside a
//!   segment. A quiet room therefore sends what it sends today, all of the audio.
//! - **Where a cut goes** ([`Planner`]): after [`PAUSE`] of non-speech that follows at least
//!   [`MIN_SPEECH`] of speech. A speaker who does not pause gets a cut in a shorter gap between
//!   words once the segment has run [`SOFT_MAX`] from its first speech ([`GAP`]), in any frame
//!   below the speech threshold from [`HARD_MAX`], and at [`LONGEST`] wherever the stream is. A
//!   cut is only placed in silence the model has already heard: the audio sent so far must all
//!   have been analysed, except at [`LONGEST`].
//! - **Where Silero runs**: on a thread of its own per source ([`Detector`]), never in the
//!   audio callback and never on the async runtime. The client hands it each chunk through a
//!   bounded channel as the chunk is sent, and reads the answers back before the next one,
//!   100 ms later; audio it could not take (queue full, thread gone) counts as speech, so a
//!   detector that falls behind delays cuts, and never places one in speech. The thread ends
//!   when the client is dropped, with its session (`run_session`, cancelled by the session's
//!   tokens).
//! - **Every `activityStart` gets its `activityEnd`.** Pause, Stop and a handover's drain send
//!   `activityEnd` as their closing frame instead of `audioStreamEnd` (which the reference says
//!   to send only with automatic detection); a socket that failed takes its activity with it,
//!   and a new connection starts with none open (`setupComplete`), so its first chunk opens
//!   one. Audio held through a handover is sent on the new socket, after its `activityStart`.
//! - **When Silero cannot run** — the model is missing or will not load, or the processor
//!   lacks the instruction set ggml was built for (`whisper::cpu`, the check that refuses local
//!   Whisper on an x64 PC without AVX2) — the source starts with Gemini's own detection, as
//!   before, and logs why.

use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, SyncSender, TrySendError, sync_channel};

use super::protocol::{CLIENT_ACTIVITY_END, CLIENT_ACTIVITY_START};

/// Who ends Transcribe Live's segments.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Segmentation {
    /// Gemini's automatic activity detection, as through 1.16: cuts every 8 to 11 seconds of
    /// continuous speech, with a word lost at each.
    // Unused while `SEGMENTATION` is `Client`: the switch's other position.
    #[allow(dead_code)]
    Gemini,
    /// The client, at pauses Silero finds, with automatic detection disabled.
    Client,
}

/// The one switch. Google's reference and its transcription guide document the setup field and
/// the two signals exactly as the client sends them (read 28 November 2026), but whether a cut
/// at a pause keeps every word has not been seen on the wire yet: run
/// `live_probe::continuous_speech_cut_by_the_client` and set this back to
/// [`Segmentation::Gemini`] if it loses words or holds finals back.
pub const SEGMENTATION: Segmentation = Segmentation::Client;

/// The bundled speech-detection model for a new Gemini Transcribe source, or `None` when the
/// segments are Gemini's.
pub fn speech_model(app: &tauri::AppHandle) -> Option<PathBuf> {
    match SEGMENTATION {
        Segmentation::Gemini => None,
        Segmentation::Client => crate::whisper::vad::bundled(app),
    }
}

const RATE: usize = 16_000;
/// Silero's window at 16 kHz: 512 samples, 32 ms.
pub(super) const FRAME: usize = 512;

const fn frames(ms: usize) -> usize {
    ms * RATE / 1_000 / FRAME
}

const fn samples(ms: usize) -> usize {
    ms * RATE / 1_000
}

/// A frame is speech from this probability: whisper.cpp's default threshold.
const SPEECH: f32 = 0.5;
/// A frame is quiet below this, whisper.cpp's own lower threshold; one in between is neither,
/// and ends a run of quiet frames.
const QUIET: f32 = SPEECH - 0.15;
/// The pause that ends a segment: 15 frames, 480 ms. Shorter than Gemini's own (its default
/// end of speech, and the 800 ms tried in #281), so a speaker's sentences get their own
/// segments; long enough that the gaps inside a sentence rarely reach it.
pub(super) const PAUSE: usize = frames(480);
/// Speech a segment must hold before a pause can end it: 7 frames, 224 ms. A cough or a click
/// is not a segment of its own.
pub(super) const MIN_SPEECH: usize = frames(250);
/// From here, measured from the segment's first speech, a shorter gap ends it: 6 s.
pub(super) const SOFT_MAX: usize = samples(6_000);
/// That gap: 5 frames, 160 ms, the pause between two words.
pub(super) const GAP: usize = frames(160);
/// From here any frame below the speech threshold ends the segment: 7.5 s.
pub(super) const HARD_MAX: usize = samples(7_500);
/// The longest segment: 8 s from its first speech, cut wherever the stream is. Just under the
/// 8 to 11 s Gemini's own detection allowed, so if Gemini keeps a limit of its own in manual
/// mode, the client's cut comes first.
pub(super) const LONGEST: usize = samples(8_000);
/// Already-analysed audio run again before each new block, because the model's memory starts
/// afresh on each call (`Vad::probabilities`): 12 frames, 384 ms. Each 100 ms chunk then costs
/// about five frames of work for its three; twice this context found the same cuts on the
/// rehearsal recording for twice the work (`silero_cuts_the_rehearsal_at_its_pauses`).
const CONTEXT: usize = frames(384) * FRAME;
/// Chunks the detector thread can be behind before the client stops handing it audio: 1.6 s.
const QUEUE: usize = 16;

/// Why a segment ends where it does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Cut {
    /// A real pause, [`PAUSE`] long.
    Pause,
    /// A long segment's shorter gap between words.
    Gap,
    /// [`LONGEST`], wherever the stream is.
    Longest,
}

/// The cut decisions, from the samples sent and the speech probability of each analysed frame.
/// Pure: positions are sample counts since the source started.
#[derive(Debug, Default)]
pub(super) struct Planner {
    /// Samples sent.
    sent: usize,
    /// Samples whose speech is known, or counted as speech because they could not be analysed.
    analysed: usize,
    /// Where the segment's first speech was, once it has some.
    first_speech: Option<usize>,
    /// Speech frames in the segment.
    speech: usize,
    /// Quiet frames at the end of what has been analysed.
    quiet: usize,
}

impl Planner {
    pub(super) fn sent(&mut self, samples: usize) {
        self.sent += samples;
    }

    /// The speech probability of the next frames.
    pub(super) fn frames(&mut self, probabilities: &[f32]) {
        for &p in probabilities {
            if p >= SPEECH {
                self.speech_frame();
            } else if p < QUIET {
                self.quiet += 1;
                // Too little speech to end a segment with, and a pause after it: a click or a
                // cough, forgotten, so it neither times the segment nor counts toward it.
                if self.quiet == PAUSE && self.speech < MIN_SPEECH {
                    self.first_speech = None;
                    self.speech = 0;
                }
            } else {
                self.quiet = 0;
            }
            self.analysed += FRAME;
        }
    }

    /// Samples that will not be analysed: counted as speech, so that nothing is cut on a guess.
    pub(super) fn unknown(&mut self, samples: usize) {
        for _ in 0..samples.div_ceil(FRAME) {
            self.speech_frame();
        }
        self.analysed += samples;
    }

    fn speech_frame(&mut self) {
        self.first_speech.get_or_insert(self.analysed);
        self.speech += 1;
        self.quiet = 0;
    }

    /// Whether to end the segment before the next chunk, and why.
    pub(super) fn decide(&self) -> Option<Cut> {
        let first = self.first_speech?;
        let length = self.sent.saturating_sub(first);
        if length >= LONGEST {
            return Some(Cut::Longest);
        }
        let heard_all = self.sent.saturating_sub(self.analysed) < FRAME;
        if !heard_all || self.speech < MIN_SPEECH {
            return None;
        }
        if self.quiet >= PAUSE {
            Some(Cut::Pause)
        } else if (length >= SOFT_MAX && self.quiet >= GAP)
            || (length >= HARD_MAX && self.quiet > 0)
        {
            Some(Cut::Gap)
        } else {
            None
        }
    }

    /// A segment starts at the next chunk: after a cut, or on a new connection.
    pub(super) fn start_segment(&mut self) {
        self.first_speech = None;
        self.speech = 0;
    }
}

/// Speech probabilities for a block of 16 kHz samples, one per 512-sample frame.
pub(super) trait Model: Send {
    fn probabilities(&mut self, samples: &[f32]) -> Option<Vec<f32>>;
}

impl Model for crate::whisper::vad::Vad {
    fn probabilities(&mut self, samples: &[f32]) -> Option<Vec<f32>> {
        crate::whisper::vad::Vad::probabilities(self, samples)
    }
}

/// Feeds the model whole frames, each block after [`CONTEXT`] of the audio before it.
struct Analyser {
    model: Box<dyn Model>,
    context: Vec<f32>,
    pending: Vec<f32>,
}

impl Analyser {
    fn new(model: Box<dyn Model>) -> Self {
        Self {
            model,
            context: Vec::with_capacity(CONTEXT + 4 * FRAME),
            pending: Vec::with_capacity(4 * FRAME),
        }
    }

    /// The probabilities of the whole frames `samples` completes; a failure counts as speech.
    fn feed(&mut self, samples: &[f32]) -> Vec<f32> {
        self.pending.extend_from_slice(samples);
        let new = self.pending.len() / FRAME;
        if new == 0 {
            return Vec::new();
        }
        let before = self.context.len() / FRAME;
        self.context.extend(self.pending.drain(..new * FRAME));
        let found = self
            .model
            .probabilities(&self.context)
            .filter(|p| p.len() == before + new)
            .map_or_else(|| vec![1.0; new], |p| p[before..].to_vec());
        let keep = self.context.len().saturating_sub(CONTEXT);
        self.context.drain(..keep);
        found
    }
}

/// Where Silero runs for one source.
enum Detector {
    /// Its own thread, in production.
    Thread {
        audio: SyncSender<Vec<f32>>,
        found: Receiver<Vec<f32>>,
    },
    /// In the caller, answering at once: tests that need cuts at exact chunks.
    #[cfg(test)]
    Inline {
        analyser: Analyser,
        found: Vec<Vec<f32>>,
    },
}

impl Detector {
    fn spawn(model: Box<dyn Model>) -> Option<Self> {
        let (audio, input) = sync_channel::<Vec<f32>>(QUEUE);
        let (output, found) = sync_channel(QUEUE);
        let mut analyser = Analyser::new(model);
        std::thread::Builder::new()
            .name("gemini-speech-cuts".into())
            .spawn(move || {
                // Ends when the client, and with it `audio`, is dropped.
                while let Ok(samples) = input.recv() {
                    if output.send(analyser.feed(&samples)).is_err() {
                        break;
                    }
                }
            })
            .inspect_err(|error| tracing::warn!("speech detection thread: {error}"))
            .ok()?;
        Some(Self::Thread { audio, found })
    }

    /// Hand over a chunk; `Err` with its length when it will not be analysed.
    fn offer(&mut self, samples: Vec<f32>) -> Result<(), usize> {
        match self {
            Self::Thread { audio, .. } => audio.try_send(samples).map_err(|error| match error {
                TrySendError::Full(samples) | TrySendError::Disconnected(samples) => samples.len(),
            }),
            #[cfg(test)]
            Self::Inline { analyser, found } => {
                found.push(analyser.feed(&samples));
                Ok(())
            }
        }
    }

    /// The probabilities found since the last call.
    fn found(&mut self) -> Vec<Vec<f32>> {
        match self {
            Self::Thread { found, .. } => found.try_iter().collect(),
            #[cfg(test)]
            Self::Inline { found, .. } => std::mem::take(found),
        }
    }
}

/// One source's cuts: its detector, its planner, and whether an activity is open on the
/// connection.
pub(super) struct SpeechCuts {
    detector: Detector,
    planner: Planner,
    open: bool,
    /// Whether the detector has been seen to fall behind, so it is logged once.
    behind: bool,
}

impl SpeechCuts {
    /// Silero from `model`, on a thread of its own; `None` (logged) when it cannot run here.
    pub(super) fn start(model: &Path) -> Option<Self> {
        let vad = crate::whisper::vad::Vad::load_for_cloud(model)?;
        Detector::spawn(Box::new(vad)).map(Self::with_detector)
    }

    #[cfg(test)]
    pub(super) fn inline(model: Box<dyn Model>) -> Self {
        Self::with_detector(Detector::Inline {
            analyser: Analyser::new(model),
            found: Vec::new(),
        })
    }

    #[cfg(test)]
    pub(super) fn threaded(model: Box<dyn Model>) -> Self {
        Self::with_detector(Detector::spawn(model).unwrap())
    }

    fn with_detector(detector: Detector) -> Self {
        Self {
            detector,
            planner: Planner::default(),
            open: false,
            behind: false,
        }
    }

    /// The frames before this chunk: `activityStart` when no activity is open, `activityEnd`
    /// and `activityStart` where the planner cuts. Then the chunk goes to the detector.
    pub(super) fn before_audio(&mut self, pcm_le: &[u8]) -> Vec<String> {
        for found in self.detector.found() {
            self.planner.frames(&found);
        }
        let mut frames = Vec::new();
        if !self.open {
            self.open = true;
            self.planner.start_segment();
            frames.push(CLIENT_ACTIVITY_START.to_string());
        } else if let Some(cut) = self.planner.decide() {
            tracing::debug!(?cut, "Gemini Transcribe: the client ends the segment");
            self.planner.start_segment();
            frames.push(CLIENT_ACTIVITY_END.to_string());
            frames.push(CLIENT_ACTIVITY_START.to_string());
        }
        let samples: Vec<f32> = pcm_le
            .as_chunks::<2>()
            .0
            .iter()
            .map(|&b| f32::from(i16::from_le_bytes(b)) / 32_768.0)
            .collect();
        self.planner.sent(samples.len());
        if let Err(lost) = self.detector.offer(samples) {
            if !self.behind {
                self.behind = true;
                tracing::warn!("speech detection fell behind; no cut until it catches up");
            }
            self.planner.unknown(lost);
        }
        frames
    }

    /// The closing frame of a Pause, Stop or handover: the open activity's end, or nothing
    /// when no audio went out on this connection.
    pub(super) fn closing(&mut self) -> Vec<String> {
        if std::mem::take(&mut self.open) {
            vec![CLIENT_ACTIVITY_END.to_string()]
        } else {
            Vec::new()
        }
    }

    /// A new connection has no activity open; its first chunk opens one.
    pub(super) fn new_connection(&mut self) {
        self.open = false;
    }
}

#[cfg(test)]
pub(super) mod tests {
    use super::*;

    /// One chunk of 100 ms.
    pub const CHUNK: usize = 1_600;

    /// Speech wherever a frame is loud: what a loudness gate would say, for exact tests.
    pub struct Loudness;

    impl Model for Loudness {
        fn probabilities(&mut self, samples: &[f32]) -> Option<Vec<f32>> {
            Some(
                samples
                    .chunks(FRAME)
                    .map(|f| {
                        if f.iter().any(|s| s.abs() > 0.01) {
                            0.9
                        } else {
                            0.0
                        }
                    })
                    .collect(),
            )
        }
    }

    /// A 100 ms chunk, loud or silent.
    pub fn chunk(loud: bool) -> Vec<u8> {
        let level: i16 = if loud { 8_000 } else { 0 };
        std::iter::repeat_n(level.to_le_bytes(), CHUNK)
            .flatten()
            .collect()
    }

    fn feed(planner: &mut Planner, p: f32, n: usize) {
        for _ in 0..n {
            planner.sent(FRAME);
            planner.frames(&[p]);
        }
    }

    const SECOND: usize = RATE / FRAME;

    #[test]
    fn a_pause_after_speech_ends_the_segment() {
        let mut planner = Planner::default();
        feed(&mut planner, 0.9, SECOND);
        feed(&mut planner, 0.1, PAUSE - 1);
        assert_eq!(planner.decide(), None);
        feed(&mut planner, 0.1, 1);
        assert_eq!(planner.decide(), Some(Cut::Pause));
        planner.start_segment();
        // The silence goes on in the next segment, which has no speech to end.
        feed(&mut planner, 0.0, 60 * SECOND);
        assert_eq!(planner.decide(), None);
    }

    #[test]
    fn speech_resuming_within_the_pause_keeps_the_segment() {
        let mut planner = Planner::default();
        feed(&mut planner, 0.9, SECOND);
        feed(&mut planner, 0.1, PAUSE - 2);
        feed(&mut planner, 0.9, 3);
        feed(&mut planner, 0.1, PAUSE - 2);
        assert_eq!(planner.decide(), None);
        // A frame the model is unsure of is not part of a pause.
        feed(&mut planner, 0.4, 1);
        feed(&mut planner, 0.1, PAUSE - 1);
        assert_eq!(planner.decide(), None);
        feed(&mut planner, 0.1, 1);
        assert_eq!(planner.decide(), Some(Cut::Pause));
    }

    #[test]
    fn silence_or_a_click_alone_is_never_cut() {
        let mut planner = Planner::default();
        feed(&mut planner, 0.0, 120 * SECOND);
        assert_eq!(planner.decide(), None);
        feed(&mut planner, 0.9, MIN_SPEECH - 1);
        feed(&mut planner, 0.0, 10 * SECOND);
        assert_eq!(planner.decide(), None);
        // Forgotten: speech a minute later is timed from its own start.
        feed(&mut planner, 0.0, 60 * SECOND);
        feed(&mut planner, 0.9, SECOND);
        assert_eq!(planner.decide(), None);
    }

    /// A speaker who never pauses: from `SOFT_MAX` a gap between words ends the segment, from
    /// `HARD_MAX` any quiet frame, and at `LONGEST` the stream is cut wherever it is.
    #[test]
    fn a_speaker_who_never_pauses_is_cut_at_the_least_speech_time_allows() {
        let soft = SOFT_MAX / FRAME;
        let mut planner = Planner::default();
        // Before SOFT_MAX a gap does not end a segment.
        feed(&mut planner, 0.9, soft - 20);
        feed(&mut planner, 0.1, GAP);
        assert_eq!(planner.decide(), None);
        feed(&mut planner, 0.9, 20);
        feed(&mut planner, 0.1, GAP - 1);
        assert_eq!(planner.decide(), None);
        feed(&mut planner, 0.1, 1);
        assert_eq!(planner.decide(), Some(Cut::Gap));

        // From HARD_MAX one quiet frame is enough.
        let mut planner = Planner::default();
        feed(&mut planner, 0.9, HARD_MAX / FRAME + 1);
        assert_eq!(planner.decide(), None);
        feed(&mut planner, 0.2, 1);
        assert_eq!(planner.decide(), Some(Cut::Gap));

        // No gap at all: cut at LONGEST, timed from the first speech, not the segment's start.
        let mut planner = Planner::default();
        feed(&mut planner, 0.0, 5 * SECOND);
        feed(&mut planner, 0.9, LONGEST / FRAME - 1);
        assert_eq!(planner.decide(), None);
        feed(&mut planner, 0.9, 1);
        assert_eq!(planner.decide(), Some(Cut::Longest));
        planner.start_segment();
        feed(&mut planner, 0.9, 1);
        assert_eq!(planner.decide(), None);
    }

    /// A cut goes only in silence the model has heard; audio it could not analyse is speech.
    #[test]
    fn audio_not_analysed_yet_holds_the_cut_back_except_at_the_longest() {
        let mut planner = Planner::default();
        feed(&mut planner, 0.9, SECOND);
        feed(&mut planner, 0.0, PAUSE);
        planner.sent(FRAME);
        assert_eq!(planner.decide(), None, "a frame sent, not analysed yet");
        planner.frames(&[0.0]);
        assert_eq!(planner.decide(), Some(Cut::Pause));

        planner.sent(CHUNK);
        planner.unknown(CHUNK);
        assert_eq!(planner.decide(), None, "audio the detector did not take");

        let mut planner = Planner::default();
        feed(&mut planner, 0.9, SECOND);
        planner.sent(LONGEST);
        assert_eq!(planner.decide(), Some(Cut::Longest));
    }

    /// Each block is analysed after the audio before it, and only whole frames are.
    #[test]
    fn the_analyser_runs_whole_frames_after_their_context() {
        struct Echo(std::sync::Arc<std::sync::Mutex<Vec<usize>>>);
        impl Model for Echo {
            fn probabilities(&mut self, samples: &[f32]) -> Option<Vec<f32>> {
                self.0.lock().unwrap().push(samples.len());
                Some(samples.chunks(FRAME).map(|f| f[0]).collect())
            }
        }
        let lengths = std::sync::Arc::default();
        let mut analyser = Analyser::new(Box::new(Echo(std::sync::Arc::clone(&lengths))));
        let mut all = Vec::new();
        for i in 0..20 {
            #[expect(clippy::cast_precision_loss, reason = "small test values")]
            let block = vec![i as f32; CHUNK];
            all.extend(analyser.feed(&block));
        }
        // 32,000 samples: 62 whole frames, each answered once, in order.
        assert_eq!(all.len(), 20 * CHUNK / FRAME);
        let expected: Vec<f32> = (0..all.len())
            .map(|f| {
                #[expect(clippy::cast_precision_loss, reason = "small test values")]
                let chunk = (f * FRAME / CHUNK) as f32;
                chunk
            })
            .collect();
        assert_eq!(all, expected);
        let lengths = lengths.lock().unwrap();
        assert!(
            lengths
                .iter()
                .all(|&n| n % FRAME == 0 && n <= CONTEXT + CHUNK + FRAME)
        );
        assert_eq!(lengths.last(), Some(&(CONTEXT + 3 * FRAME)));

        // A model that fails counts as speech.
        struct Broken;
        impl Model for Broken {
            fn probabilities(&mut self, _samples: &[f32]) -> Option<Vec<f32>> {
                None
            }
        }
        assert_eq!(
            Analyser::new(Box::new(Broken)).feed(&[0.0; CHUNK]),
            [1.0; 3]
        );
    }

    /// The frames before each chunk: the activity opens with the first, a pause cuts it, and a
    /// close ends it once.
    #[test]
    fn activity_frames_open_cut_and_close() {
        let mut cuts = SpeechCuts::inline(Box::new(Loudness));
        assert_eq!(cuts.closing(), Vec::<String>::new(), "nothing sent");
        let mut seen = Vec::new();
        for loud in [true; 10].into_iter().chain([false; 8]) {
            for frame in cuts.before_audio(&chunk(loud)) {
                seen.push(frame);
            }
            seen.push(if loud { "speech" } else { "quiet" }.to_string());
        }
        let start = CLIENT_ACTIVITY_START.to_string();
        let end = CLIENT_ACTIVITY_END.to_string();
        assert_eq!(seen[0], start);
        let cut = seen.iter().position(|f| *f == end).expect("a cut");
        assert_eq!(seen[cut + 1], start);
        // 15 quiet frames heard, then the cut. Five quiet chunks hold only 14 whole ones (the
        // frame across the last loud chunk's end is speech), so it comes after the sixth.
        assert_eq!(
            seen[..cut].iter().filter(|f| *f == "quiet").count(),
            6,
            "{seen:?}"
        );
        assert_eq!(cuts.closing(), [end]);
        assert_eq!(cuts.closing(), Vec::<String>::new(), "closed once");
        cuts.new_connection();
        assert_eq!(cuts.before_audio(&chunk(false)), [start]);
    }

    /// The detector thread answers each chunk, so cuts follow as they do inline; a detector
    /// that cannot keep up counts what it missed as speech.
    #[test]
    fn the_detector_thread_answers_every_chunk() {
        let mut cuts = SpeechCuts::threaded(Box::new(Loudness));
        let mut cut_at = None;
        for (i, loud) in [true; 10].into_iter().chain([false; 10]).enumerate() {
            if cuts.before_audio(&chunk(loud)).len() == 2 {
                cut_at = Some(i);
                break;
            }
            // The 100 ms a real chunk takes, shortened: the thread answers in microseconds.
            let until = std::time::Instant::now() + std::time::Duration::from_secs(5);
            while cuts.planner.sent - cuts.planner.analysed >= FRAME
                && std::time::Instant::now() < until
            {
                for found in cuts.detector.found() {
                    cuts.planner.frames(&found);
                }
                std::thread::yield_now();
            }
        }
        assert_eq!(
            cut_at,
            Some(16),
            "before the seventh quiet chunk, as inline"
        );

        // A model stuck on its first block: the queue fills, and what the thread could not
        // take counts as speech, so no pause is found in the silence after it.
        struct Stuck(std::sync::Mutex<std::sync::mpsc::Receiver<()>>);
        impl Model for Stuck {
            fn probabilities(&mut self, samples: &[f32]) -> Option<Vec<f32>> {
                let _ = self.0.lock().unwrap().recv();
                Loudness.probabilities(samples)
            }
        }
        let (release, stuck) = std::sync::mpsc::channel();
        let mut stalled = SpeechCuts::threaded(Box::new(Stuck(std::sync::Mutex::new(stuck))));
        for _ in 0..10 {
            stalled.before_audio(&chunk(true));
        }
        for _ in 0..2 * QUEUE {
            assert_eq!(stalled.before_audio(&chunk(false)).len(), 0);
        }
        assert!(stalled.behind);
        assert_eq!(stalled.planner.decide(), None);
        drop(release);
    }

    /// The real model on the rehearsal recording, as the client streams it: once with two
    /// seconds of silence in the middle, and twice without a pause, as Rehearse loops it.
    /// Prints where each cut lands and what Silero cost per second of audio.
    #[test]
    fn silero_cuts_the_rehearsal_at_its_pauses() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("resources");
        let english =
            crate::audio::fixture::load_fixture(&root.join("fixtures/rehearsal-en.wav")).unwrap();
        let silence = vec![0.0_f32; 2 * RATE];
        for (name, stream, silent) in [
            (
                "with a silence",
                [&english[..], &silence, &english].concat(),
                Some(english.len()..english.len() + silence.len()),
            ),
            ("continuous", [&english[..], &english].concat(), None),
        ] {
            let vad = crate::whisper::vad::Vad::load(&root.join(crate::whisper::vad::RESOURCE))
                .expect("the bundled model loads");
            let mut cuts = SpeechCuts::inline(Box::new(vad));
            let started = std::time::Instant::now();
            let mut at = Vec::new();
            let mut sent = 0usize;
            let mut kinds = Vec::new();
            for block in stream.chunks(CHUNK) {
                let mut pcm = Vec::new();
                crate::audio::resample::f32_to_pcm16_le(block, &mut pcm);
                for found in cuts.detector.found() {
                    cuts.planner.frames(&found);
                }
                let kind = cuts.planner.decide();
                if cuts.before_audio(&pcm).len() == 2 {
                    at.push(sent);
                    kinds.push(kind.expect("a cut has its reason"));
                }
                sent += block.len();
            }
            #[expect(clippy::cast_precision_loss, reason = "seconds for a printout")]
            let seconds = |n: usize| n as f32 / RATE as f32;
            println!(
                "{name}: {:.1}s of audio, Silero {:.1} ms per second of audio, cuts at {:?}",
                seconds(stream.len()),
                started.elapsed().as_secs_f32() * 1_000.0 / seconds(stream.len()),
                at.iter()
                    .zip(&kinds)
                    .map(|(&n, kind)| format!("{:.2}s {kind:?}", seconds(n)))
                    .collect::<Vec<_>>()
            );
            // No segment runs past the longest allowed, plus the silence before its speech.
            let mut last = 0;
            for &cut in &at {
                assert!(cut - last <= LONGEST + samples(2_000), "{name}: {at:?}");
                last = cut;
            }
            // The recording ends in a moment of silence of its own, so the pause may be found
            // there, just before the two seconds added.
            if let Some(silent) = silent {
                let pause = silent.start - samples(1_500)..silent.end;
                assert!(
                    at.iter()
                        .zip(&kinds)
                        .any(|(cut, kind)| pause.contains(cut) && *kind == Cut::Pause),
                    "{name}: no cut at the pause: {at:?}"
                );
            }
        }
    }
}
