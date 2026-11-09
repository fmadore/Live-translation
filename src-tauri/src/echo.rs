//! Room-microphone lines that repeat the system audio, kept out of the captions.
//!
//! With Both sources, a meeting played through the room's loudspeakers reaches the room
//! microphone too, so the remote speaker is captioned twice: once from the system audio, as
//! said, and once from the microphone, as the room heard it. The guard drops the microphone's
//! copy, and only on what the two engines wrote: a microphone line is withdrawn when its words
//! are, nearly enough, words the system source captioned at the same moment. Audio can tell an
//! echo from speech only so far (a person talking over the loudspeaker sounds much like the
//! loudspeaker to a correlation), and text costs nothing to compare. Nothing is lost when it is
//! wrong the other way: what it drops from the microphone is on screen from the system source.
//!
//! The two engines do not finish in step, so a microphone line can be complete before the
//! system line it repeats has arrived. While the system source is sounding, a finished
//! microphone line therefore waits a few seconds before it is committed (still shown, as an
//! interim) for a system line to match it; and a microphone turn still being spoken is
//! withdrawn as soon as a system line matches it. A line that waited out its time is committed
//! as it was.
//!
//! A withdrawal is the turn's final caption with no text: the transcript commits nothing for it
//! and the windows clear the turn. A line already committed is never taken back.
//!
//! The decisions are pure (`Guard`, no Tauri, a clock passed in) and tested here; `EchoGuard`
//! is the app's copy, which every caption passes through on its way out of the core
//! (`Events::caption` for `AppHandle`), and which emits under its lock, so a withdrawal can
//! never overtake the caption it withdraws.

use std::collections::HashSet;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use tauri::{AppHandle, Manager};

use crate::sync::lock;
use crate::types::{AudioLevel, Caption, Origin};
use crate::whisper::reconcile::unspaced;

/// How far apart, in session time, a system caption and a microphone caption may be and still
/// be the same speech: the engines stamp a caption when they emit it, and two of them on two
/// copies of one voice do not emit together.
const SLACK_MS: u64 = 4_000;
/// How long a system caption is kept to compare with, after its last update.
const RETAIN: Duration = Duration::from_secs(20);
/// At most this many system turns per caption language are kept.
const MAX_HEARD: usize = 32;
/// Only the end of a long system turn is compared: continuous speech can stream for minutes
/// in one turn, and the more text there is to compare with, the likelier a chance match.
const HEARD_TAIL: usize = 200;
/// The system source counts as sounding for this long after it was last loud or captioned.
const ACTIVE: Duration = Duration::from_secs(5);
/// Loud enough to be speech on the loudspeakers: the level the operator window counts as
/// audio activity.
const LOUD_RMS: f32 = 0.02;
/// How long a finished microphone line waits for the system line it may repeat. Local Whisper
/// decodes the two sources in turn, ten seconds of audio at a time, so its system line can come
/// a whole decoding pass later than a cloud engine's.
pub const HOLD_CLOUD: Duration = Duration::from_secs(4);
pub const HOLD_LOCAL: Duration = Duration::from_secs(10);
/// Two caption languages at most: lane 0 and lane 1.
const LANES: usize = 2;

/// A caption the guard keeps, or makes, to emit later.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnedCaption {
    pub turn_id: u64,
    pub text: String,
    pub source_text: String,
    pub final_: bool,
    pub origin: Origin,
    pub lane: u8,
    pub start_ms: u64,
    pub end_ms: u64,
}

impl OwnedCaption {
    fn of(caption: &Caption<'_>) -> Self {
        Self {
            turn_id: caption.turn_id,
            text: caption.text.to_owned(),
            source_text: caption.source_text.to_owned(),
            final_: caption.final_,
            origin: caption.origin,
            lane: caption.lane,
            start_ms: caption.start_ms,
            end_ms: caption.end_ms,
        }
    }

    /// The final caption that withdraws a microphone turn: no text, so nothing is committed.
    fn withdrawal(turn_id: u64, lane: u8, start_ms: u64, end_ms: u64) -> Self {
        Self {
            turn_id,
            text: String::new(),
            source_text: String::new(),
            final_: true,
            origin: Origin::Microphone,
            lane,
            start_ms,
            end_ms,
        }
    }

    pub fn as_caption(&self) -> Caption<'_> {
        Caption {
            turn_id: self.turn_id,
            text: &self.text,
            source_text: &self.source_text,
            final_: self.final_,
            origin: self.origin,
            lane: self.lane,
            start_ms: self.start_ms,
            end_ms: self.end_ms,
        }
    }
}

/// What to do with one caption: emit `before` first, then the caption itself when `pass`, and
/// call `Guard::due` again at `wake`.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Outcome {
    pub before: Vec<OwnedCaption>,
    pub pass: bool,
    pub wake: Option<Instant>,
}

impl Outcome {
    fn pass() -> Self {
        Self {
            pass: true,
            ..Self::default()
        }
    }
}

/// A system turn, as it last read.
struct Heard {
    turn_id: u64,
    text: String,
    source_text: String,
    start_ms: u64,
    end_ms: u64,
    seen: Instant,
}

/// The microphone turn being spoken.
struct MicTurn {
    turn_id: u64,
    start_ms: u64,
    end_ms: u64,
    /// Whether the windows show it: it has had text, and has not been withdrawn since.
    shown: bool,
}

/// A finished microphone line waiting for the system line it may repeat. The windows show it
/// as an interim meanwhile.
struct Held {
    caption: OwnedCaption,
    until: Instant,
}

#[derive(Default)]
struct Lane {
    heard: Vec<Heard>,
    mic: Option<MicTurn>,
    held: Option<Held>,
}

#[derive(Default)]
pub struct Guard {
    /// Only a live session with both sources has anything to compare.
    enabled: bool,
    hold: Duration,
    lanes: [Lane; LANES],
    /// When the system source was last loud or captioned.
    system_active: Option<Instant>,
}

impl Guard {
    /// Start over for a new session. `enabled` when it captures both sources; `hold` is how long
    /// a finished microphone line may wait.
    pub fn begin(&mut self, enabled: bool, hold: Duration) {
        *self = Self {
            enabled,
            hold,
            ..Self::default()
        };
    }

    /// A meter reading: the system source sounding is a reason to hold a microphone line.
    pub fn level(&mut self, origin: Origin, rms: f32, now: Instant) {
        if self.enabled && origin == Origin::System && rms >= LOUD_RMS {
            self.system_active = Some(now);
        }
    }

    pub fn caption(&mut self, caption: &Caption<'_>, now: Instant) -> Outcome {
        let lane = usize::from(caption.lane);
        if !self.enabled || lane >= LANES {
            return Outcome::pass();
        }
        match caption.origin {
            Origin::System => self.system(lane, caption, now),
            Origin::Microphone => self.microphone(lane, caption, now),
        }
    }

    /// The held lines whose time is up, to commit as they were.
    pub fn due(&mut self, now: Instant) -> Vec<OwnedCaption> {
        self.lanes
            .iter_mut()
            .filter_map(|lane| match &lane.held {
                Some(held) if held.until <= now => lane.held.take().map(|held| held.caption),
                _ => None,
            })
            .collect()
    }

    /// The session has ended: every held line is committed, and nothing more is compared.
    pub fn finish(&mut self) -> Vec<OwnedCaption> {
        let released = self
            .lanes
            .iter_mut()
            .filter_map(|lane| lane.held.take().map(|held| held.caption))
            .collect();
        self.begin(false, Duration::ZERO);
        released
    }

    fn system(&mut self, index: usize, caption: &Caption<'_>, now: Instant) -> Outcome {
        if !caption.text.trim().is_empty() {
            self.system_active = Some(now);
        }
        let lane = &mut self.lanes[index];
        remember(&mut lane.heard, caption, now);
        let mut outcome = Outcome::pass();
        if let Some(held) = &lane.held {
            let line = &held.caption;
            if repeats(&lane.heard, &line.as_caption()) {
                tracing::debug!(lane = index, turn = line.turn_id, "withdrew a held echo");
                outcome.before.push(OwnedCaption::withdrawal(
                    line.turn_id,
                    line.lane,
                    line.start_ms,
                    line.end_ms,
                ));
                lane.held = None;
            }
        }
        // A microphone turn still being spoken has no text kept here: it is compared when its
        // next caption arrives. Withdrawing it now would need that text, and its next caption,
        // a fraction of a second away, does the same with less to keep.
        outcome
    }

    fn microphone(&mut self, index: usize, caption: &Caption<'_>, now: Instant) -> Outcome {
        let active = self
            .system_active
            .is_some_and(|at| now.saturating_duration_since(at) < ACTIVE);
        let hold = self.hold;
        let lane = &mut self.lanes[index];
        let mut outcome = Outcome::default();
        // A new microphone turn means the held one is over: the windows would commit what they
        // show of it the moment the new turn arrives, so it is committed as it was, first.
        if lane
            .held
            .as_ref()
            .is_some_and(|held| held.caption.turn_id != caption.turn_id)
            && let Some(held) = lane.held.take()
        {
            outcome.before.push(held.caption);
        }
        if lane
            .mic
            .as_ref()
            .is_none_or(|turn| turn.turn_id != caption.turn_id)
        {
            lane.mic = Some(MicTurn {
                turn_id: caption.turn_id,
                start_ms: caption.start_ms,
                end_ms: caption.end_ms,
                shown: false,
            });
        }
        let Some(turn) = lane.mic.as_mut() else {
            return Outcome::pass();
        };
        turn.end_ms = caption.end_ms;
        if caption.text.trim().is_empty() {
            if caption.final_ {
                lane.mic = None;
            }
            outcome.pass = true;
            return outcome;
        }
        let echo = repeats(&lane.heard, caption);
        let shown = turn.shown;
        let withdrawal =
            OwnedCaption::withdrawal(turn.turn_id, caption.lane, turn.start_ms, caption.end_ms);
        if !caption.final_ {
            if echo {
                if shown {
                    outcome.before.push(withdrawal);
                }
                turn.shown = false;
            } else {
                turn.shown = true;
                outcome.pass = true;
            }
            return outcome;
        }
        lane.mic = None;
        if echo {
            tracing::debug!(lane = index, turn = caption.turn_id, "dropped an echo");
            if shown {
                outcome.before.push(withdrawal);
            }
        } else if active {
            // Shown at once, committed once its wait is over or nothing has matched it.
            let line = OwnedCaption::of(caption);
            outcome.before.push(OwnedCaption {
                final_: false,
                ..line.clone()
            });
            let until = now + hold;
            lane.held = Some(Held {
                caption: line,
                until,
            });
            outcome.wake = Some(until);
        } else {
            outcome.pass = true;
        }
        outcome
    }
}

/// Keep a system caption's latest text to compare with, and forget what is too old.
fn remember(heard: &mut Vec<Heard>, caption: &Caption<'_>, now: Instant) {
    heard.retain(|turn| now.saturating_duration_since(turn.seen) < RETAIN);
    if let Some(turn) = heard
        .iter_mut()
        .find(|turn| turn.turn_id == caption.turn_id)
    {
        turn.text.clear();
        turn.text.push_str(caption.text);
        turn.source_text.clear();
        turn.source_text.push_str(caption.source_text);
        turn.end_ms = caption.end_ms;
        turn.seen = now;
        return;
    }
    if caption.text.trim().is_empty() {
        return;
    }
    if heard.len() >= MAX_HEARD {
        heard.remove(0);
    }
    heard.push(Heard {
        turn_id: caption.turn_id,
        text: caption.text.to_owned(),
        source_text: caption.source_text.to_owned(),
        start_ms: caption.start_ms,
        end_ms: caption.end_ms,
        seen: now,
    });
}

/// Whether a microphone caption says what the system source captioned around the same time,
/// in the caption language or, where both engines give it, in the original speech.
fn repeats(heard: &[Heard], caption: &Caption<'_>) -> bool {
    let near: Vec<&Heard> = heard
        .iter()
        .filter(|turn| {
            turn.start_ms <= caption.end_ms.saturating_add(SLACK_MS)
                && turn.end_ms.saturating_add(SLACK_MS) >= caption.start_ms
        })
        .collect();
    if near.is_empty() {
        return false;
    }
    let short_ok = caption.final_;
    let tail = |text: &str| {
        let words = tokens(text);
        let skip = words.len().saturating_sub(HEARD_TAIL);
        words.into_iter().skip(skip)
    };
    let heard_text: Vec<String> = near.iter().flat_map(|turn| tail(&turn.text)).collect();
    if similar(&tokens(caption.text), &heard_text, short_ok) {
        return true;
    }
    if caption.source_text.trim().is_empty() {
        return false;
    }
    let heard_source: Vec<String> = near
        .iter()
        .flat_map(|turn| tail(&turn.source_text))
        .collect();
    similar(&tokens(caption.source_text), &heard_source, short_ok)
}

/// Whether `said` is, nearly enough, part of `heard`. From three words on: at least two of its
/// word pairs, and three in five, are pairs `heard` has. Pairs rather than words, because any
/// two sentences in a language share their common words, and pairs they rarely share by
/// chance; three in five, because two engines hearing one voice through different microphones
/// do not write every word alike. A line of one or two words (finished lines only: a turn is
/// that short as it starts) must appear in `heard` exactly as it is.
fn similar(said: &[String], heard: &[String], short_ok: bool) -> bool {
    match said.len() {
        0 => false,
        1 | 2 => short_ok && heard.windows(said.len()).any(|run| run == said),
        _ => {
            let pairs: HashSet<(&str, &str)> = heard
                .windows(2)
                .map(|pair| (pair[0].as_str(), pair[1].as_str()))
                .collect();
            let total = said.len() - 1;
            let shared = said
                .windows(2)
                .filter(|pair| pairs.contains(&(pair[0].as_str(), pair[1].as_str())))
                .count();
            shared >= 2 && shared * 5 >= total * 3
        }
    }
}

/// A caption's words, case-folded and without punctuation. In scripts written without spaces,
/// each character is a word: there is no other boundary to compare on.
fn tokens(text: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut word = String::new();
    for c in text.chars() {
        if unspaced(c) {
            if !word.is_empty() {
                words.push(std::mem::take(&mut word));
            }
            if c.is_alphanumeric() {
                words.push(c.to_string());
            }
        } else if c.is_alphanumeric() {
            word.extend(c.to_lowercase());
        } else if !word.is_empty() {
            words.push(std::mem::take(&mut word));
        }
    }
    if !word.is_empty() {
        words.push(word);
    }
    words
}

/// The app's guard, a Tauri state. Emits under its lock: see the module's notes.
#[derive(Default)]
pub struct EchoGuard(Mutex<Guard>);

impl EchoGuard {
    pub fn begin(&self, enabled: bool, hold: Duration) {
        lock(&self.0).begin(enabled, hold);
    }

    pub fn level(&self, level: &AudioLevel) {
        lock(&self.0).level(level.source, level.rms, Instant::now());
    }

    /// Pass `caption` on through `deliver`, or what the guard makes of it.
    pub fn filter(
        &self,
        app: &AppHandle,
        caption: &Caption<'_>,
        deliver: fn(&AppHandle, &Caption<'_>),
    ) {
        let mut guard = lock(&self.0);
        let outcome = guard.caption(caption, Instant::now());
        for early in &outcome.before {
            deliver(app, &early.as_caption());
        }
        if outcome.pass {
            deliver(app, caption);
        }
        drop(guard);
        if let Some(wake) = outcome.wake {
            let app = app.clone();
            tauri::async_runtime::spawn(async move {
                tokio::time::sleep_until(wake.into()).await;
                let Some(state) = app.try_state::<EchoGuard>() else {
                    return;
                };
                let mut guard = lock(&state.0);
                for line in guard.due(Instant::now()) {
                    deliver(&app, &line.as_caption());
                }
            });
        }
    }

    /// The session has ended: commit what is still held, before its Idle goes out.
    pub fn finish(&self, app: &AppHandle, deliver: fn(&AppHandle, &Caption<'_>)) {
        let mut guard = lock(&self.0);
        for line in guard.finish() {
            deliver(app, &line.as_caption());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn caption(
        origin: Origin,
        turn: u64,
        text: &'static str,
        final_: bool,
        at: u64,
    ) -> Caption<'static> {
        Caption {
            turn_id: turn,
            text,
            source_text: "",
            final_,
            origin,
            lane: 0,
            start_ms: at,
            end_ms: at + 1_000,
        }
    }

    fn mic(turn: u64, text: &'static str, final_: bool, at: u64) -> Caption<'static> {
        caption(Origin::Microphone, turn, text, final_, at)
    }

    fn system(turn: u64, text: &'static str, final_: bool, at: u64) -> Caption<'static> {
        caption(Origin::System, turn, text, final_, at)
    }

    fn guard() -> Guard {
        let mut guard = Guard::default();
        guard.begin(true, HOLD_CLOUD);
        guard
    }

    fn withdrawal(turn: u64) -> OwnedCaption {
        OwnedCaption::withdrawal(turn, 0, 0, 0)
    }

    /// The emitted captions' turn, text and finality, without their times.
    fn shape(captions: &[OwnedCaption]) -> Vec<(u64, &str, bool)> {
        captions
            .iter()
            .map(|c| (c.turn_id, c.text.as_str(), c.final_))
            .collect()
    }

    const SPEECH: &str = "The budget for next year will be presented on Thursday morning";
    const ROOM_HEARD: &str = "the budget for next year will be presented on thursday";

    #[test]
    fn a_microphone_line_repeating_the_system_one_is_dropped() {
        let mut guard = guard();
        let now = Instant::now();
        assert!(guard.caption(&system(1, SPEECH, true, 0), now).pass);
        let outcome = guard.caption(&mic(1, ROOM_HEARD, true, 300), now);
        assert_eq!(
            outcome,
            Outcome::default(),
            "never shown, so nothing to withdraw"
        );
    }

    #[test]
    fn a_shown_microphone_turn_is_withdrawn_once_it_repeats_the_system() {
        let mut guard = guard();
        let now = Instant::now();
        // The microphone's engine starts first, on words too few to judge.
        assert!(guard.caption(&mic(4, "the budget", false, 0), now).pass);
        assert!(guard.caption(&system(9, SPEECH, false, 100), now).pass);
        let outcome = guard.caption(&mic(4, ROOM_HEARD, false, 0), now);
        assert!(!outcome.pass);
        assert_eq!(shape(&outcome.before), [(4, "", true)]);
        // Withdrawn once; its final is dropped without a second withdrawal.
        let outcome = guard.caption(&mic(4, ROOM_HEARD, true, 0), now);
        assert_eq!(outcome, Outcome::default());
    }

    #[test]
    fn a_finished_line_waits_while_the_system_sounds_and_goes_when_its_echo_arrives() {
        let mut guard = guard();
        let now = Instant::now();
        guard.level(Origin::System, 0.1, now);
        let outcome = guard.caption(&mic(2, ROOM_HEARD, true, 0), now);
        assert!(!outcome.pass);
        // Still on screen, as an interim, while it waits.
        assert_eq!(shape(&outcome.before), [(2, ROOM_HEARD, false)]);
        assert_eq!(outcome.wake, Some(now + HOLD_CLOUD));
        let outcome = guard.caption(&system(7, SPEECH, true, 200), now + Duration::from_secs(1));
        assert!(outcome.pass);
        assert_eq!(shape(&outcome.before), [(2, "", true)]);
        assert!(guard.due(now + HOLD_CLOUD).is_empty());
    }

    #[test]
    fn a_held_line_nothing_repeats_is_committed_when_its_time_is_up() {
        let mut guard = guard();
        let now = Instant::now();
        guard.level(Origin::System, 0.1, now);
        guard.caption(&mic(2, "I would like to ask a question", true, 0), now);
        guard.caption(&system(7, SPEECH, true, 200), now);
        assert!(guard.due(now + Duration::from_secs(1)).is_empty());
        let due = guard.due(now + HOLD_CLOUD);
        assert_eq!(shape(&due), [(2, "I would like to ask a question", true)]);
    }

    #[test]
    fn the_next_microphone_turn_commits_the_held_one_first() {
        let mut guard = guard();
        let now = Instant::now();
        guard.level(Origin::System, 0.1, now);
        guard.caption(&mic(2, "I would like to ask a question", true, 0), now);
        let outcome = guard.caption(&mic(3, "about the budget", false, 1_500), now);
        assert!(outcome.pass);
        assert_eq!(
            shape(&outcome.before),
            [(2, "I would like to ask a question", true)]
        );
    }

    #[test]
    fn local_speech_while_the_system_is_quiet_passes_at_once() {
        let mut guard = guard();
        let now = Instant::now();
        guard.level(Origin::System, 0.001, now);
        assert!(
            guard
                .caption(&mic(1, "Good morning everyone", true, 0), now)
                .pass
        );
        // Loud, but long enough ago.
        guard.level(Origin::System, 0.2, now);
        let later = now + ACTIVE;
        assert!(
            guard
                .caption(&mic(2, "Let us begin", true, 9_000), later)
                .pass
        );
    }

    #[test]
    fn local_speech_over_the_system_is_kept() {
        let mut guard = guard();
        let now = Instant::now();
        guard.caption(
            &system(
                1,
                "I think we should move to the next item on the agenda",
                false,
                0,
            ),
            now,
        );
        // Common words and one shared pair are not an echo.
        for text in [
            "I think so too",
            "Can we move on please",
            "I think we need a break",
        ] {
            let outcome = guard.caption(&mic(5, text, false, 200), now);
            assert!(outcome.pass, "{text}");
        }
    }

    #[test]
    fn captions_far_apart_in_time_are_not_compared() {
        let mut guard = guard();
        let now = Instant::now();
        guard.caption(&system(1, SPEECH, true, 0), now);
        // Someone in the room quoting it a minute later: shown, and held only because the
        // system source has just been captioned.
        let outcome = guard.caption(&mic(1, ROOM_HEARD, true, 60_000), now);
        assert_eq!(shape(&outcome.before), [(1, ROOM_HEARD, false)]);
    }

    #[test]
    fn old_system_captions_are_forgotten() {
        let mut guard = guard();
        let now = Instant::now();
        guard.caption(&system(1, SPEECH, true, 0), now);
        let later = now + RETAIN;
        // Any system caption prunes; this one says something else.
        guard.caption(&system(2, "Next question", true, 1_000), later);
        let outcome = guard.caption(&mic(1, ROOM_HEARD, true, 500), later);
        // The system was just captioned, so the line is held rather than dropped.
        assert!(!outcome.pass);
        assert_eq!(shape(&outcome.before), [(1, ROOM_HEARD, false)]);
    }

    #[test]
    fn short_finished_lines_must_appear_whole() {
        let mut guard = guard();
        let now = Instant::now();
        guard.caption(&system(1, "Yes, thank you very much.", true, 0), now);
        assert_eq!(
            guard.caption(&mic(1, "Thank you.", true, 0), now),
            Outcome::default()
        );
        assert_eq!(
            guard.caption(&mic(2, "Yes", true, 0), now),
            Outcome::default()
        );
        // Two words the system never said together, held because the system just spoke.
        assert!(
            !guard
                .caption(&mic(3, "Thank yes", true, 0), now)
                .before
                .is_empty()
        );
        // A turn starting with words the system said stays on screen until it says more.
        assert!(guard.caption(&mic(4, "thank you", false, 0), now).pass);
    }

    #[test]
    fn the_original_speech_is_compared_when_both_engines_give_it() {
        let mut guard = guard();
        let now = Instant::now();
        let mut sys = system(1, "Le budget sera présenté jeudi matin", true, 0);
        sys.source_text = SPEECH;
        guard.caption(&sys, now);
        // The two translations differ; what was heard did not.
        let mut room = mic(
            1,
            "Le budget de l'année prochaine sera présenté jeudi",
            true,
            0,
        );
        room.source_text = ROOM_HEARD;
        assert_eq!(guard.caption(&room, now), Outcome::default());
    }

    #[test]
    fn unspaced_scripts_compare_character_pairs() {
        assert_eq!(
            tokens("会议 现在开始。"),
            ["会", "议", "现", "在", "开", "始"]
        );
        assert_eq!(tokens("Hello, World! 2026"), ["hello", "world", "2026"]);
        let mut guard = guard();
        let now = Instant::now();
        guard.caption(&system(1, "我们现在开始今天的会议", true, 0), now);
        assert_eq!(
            guard.caption(&mic(1, "现在开始今天的会", true, 0), now),
            Outcome::default()
        );
    }

    #[test]
    fn caption_languages_are_compared_separately() {
        let mut guard = guard();
        let now = Instant::now();
        guard.caption(&system(1, SPEECH, true, 0), now);
        let mut second = mic(1, ROOM_HEARD, true, 0);
        second.lane = 1;
        // Lane 1 has heard nothing from the system; it is held only because the system spoke.
        assert!(!guard.caption(&second, now).before.is_empty());
    }

    #[test]
    fn a_session_without_both_sources_passes_everything() {
        let mut guard = Guard::default();
        guard.begin(false, HOLD_CLOUD);
        let now = Instant::now();
        guard.level(Origin::System, 0.5, now);
        guard.caption(&system(1, SPEECH, true, 0), now);
        assert!(guard.caption(&mic(1, ROOM_HEARD, true, 0), now).pass);
    }

    #[test]
    fn finishing_commits_what_is_held_and_stops_comparing() {
        let mut guard = guard();
        let now = Instant::now();
        guard.level(Origin::System, 0.1, now);
        guard.caption(&mic(2, "One last question", true, 0), now);
        assert_eq!(shape(&guard.finish()), [(2, "One last question", true)]);
        guard.caption(&system(1, SPEECH, true, 0), now);
        assert!(guard.caption(&mic(3, ROOM_HEARD, true, 0), now).pass);
    }

    #[test]
    fn an_empty_final_passes_and_ends_the_turn() {
        let mut guard = guard();
        let now = Instant::now();
        assert!(
            guard
                .caption(&mic(1, "Hello there friends", false, 0), now)
                .pass
        );
        assert!(guard.caption(&mic(1, "", true, 0), now).pass);
        assert_eq!(withdrawal(1).text, "");
    }
}
