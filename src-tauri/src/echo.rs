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
//! system line it repeats has arrived. A finished microphone line spoken while the system
//! source was sounding (loud on the meter, or captioned, within a few seconds of it in session
//! time) therefore waits before it is committed, still shown as an interim, for a system line
//! to match it. Each lane keeps a queue of such lines, because local Whisper gives every
//! segment of a window its own turn and sends them back to back: each waits for its own match,
//! and they leave in the order they came, dropped when matched, committed when their own time
//! is up or when the system source has captioned past them. The windows show one line per
//! track and commit it when the next turn arrives, so while lines wait, the microphone turn
//! after them is kept back too, its latest caption sent once they have gone. A microphone turn
//! still being spoken is withdrawn as soon as one of its captions repeats a system line, and
//! stays withdrawn until its final, which alone may bring it back: an interim hovering at the
//! threshold would otherwise blink.
//!
//! A withdrawal is the turn's final caption with no text: the transcript commits nothing for it
//! and the windows clear the turn. A line already committed is never taken back.
//!
//! The decisions are pure (`Guard`, no Tauri, a clock passed in) and tested here; `EchoGuard`
//! is the app's copy, which every caption passes through on its way out of the core
//! (`Events::caption` for `AppHandle`), and which emits under its lock, so a withdrawal can
//! never overtake the caption it withdraws. Words are split once, as a caption grows, and only
//! the last `TAIL` of a turn are kept with their pairs, so the work under that lock does not
//! grow with a turn's length.

use std::collections::{HashSet, VecDeque};
use std::hash::{DefaultHasher, Hash, Hasher};
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
/// Only the end of a long turn is compared, on both sides: continuous speech can stream for
/// minutes in one turn, and the more text there is to compare with, the likelier a chance
/// match. The microphone's line is cut the same way, or a long echoed turn would share too few
/// of its pairs with what is kept of the system's to count as one.
const TAIL: usize = 200;
/// Loud enough to be speech on the loudspeakers: the level the operator window counts as
/// audio activity.
const LOUD_RMS: f32 = 0.02;
/// Meter readings and system captions this close together, in session time, are one stretch of
/// the system source sounding.
const MERGE_MS: u64 = 1_000;
/// At most this many stretches of the system sounding are kept: minutes of speech, more than a
/// local backlog. A line older than all of them is taken as spoken over the loudspeaker.
const MAX_SPANS: usize = 512;
/// How long a finished microphone line waits, at most, for the system line it may repeat. A
/// line also goes as soon as the system source captions something that starts past it, so
/// these are the waits when the system has gone quiet since.
pub const HOLD_CLOUD: Duration = Duration::from_secs(4);
/// Local Whisper decodes the two sources in turn, ten seconds of audio at a time and 28 once a
/// backlog has built up, so its system line can come a whole decoding pass of a long window
/// after the microphone's. Only lines spoken while the system was sounding wait at all.
pub const HOLD_LOCAL: Duration = Duration::from_secs(30);
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

    /// The withdrawal of this caption's turn.
    fn withdrawn(&self) -> Self {
        Self::withdrawal(self.turn_id, self.lane, self.start_ms, self.end_ms)
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

/// A caption's words as it grows, split once: each reading splits only what was added since
/// the last whole word, and only the last `TAIL` words are kept, with their pairs.
#[derive(Default)]
struct Words {
    /// The text as last read.
    text: String,
    /// How much of `text`, in bytes, is whole words already split into `settled`.
    split: usize,
    /// The words of `text[..split]`: the last `TAIL` to `2 * TAIL` of them.
    settled: Vec<String>,
    /// The last `TAIL` words of the whole text, case-folded and without punctuation.
    tail: Vec<String>,
    /// The word pairs of `tail`.
    pairs: HashSet<u64>,
}

impl Words {
    fn read(&mut self, text: &str) {
        if text == self.text {
            return;
        }
        // Engines that revise a line rewrite what it said; start the split again then.
        let extends = text.len() >= self.split
            && text.as_bytes()[..self.split] == self.text.as_bytes()[..self.split];
        if !extends {
            self.split = 0;
            self.settled.clear();
        }
        let unchanged = self.split;
        // The prefix compared equal and ended on a character boundary, so it still does.
        self.split += settled_words(&text[self.split..], &mut self.settled);
        if self.settled.len() > 2 * TAIL {
            self.settled.drain(..self.settled.len() - TAIL);
        }
        // What follows the last whole word can only be a word still being written.
        let rest: String = text[self.split..]
            .chars()
            .flat_map(char::to_lowercase)
            .collect();
        let keep = TAIL - usize::from(!rest.is_empty());
        self.tail.clear();
        self.tail
            .extend_from_slice(&self.settled[self.settled.len().saturating_sub(keep)..]);
        if !rest.is_empty() {
            self.tail.push(rest);
        }
        self.pairs = self
            .tail
            .windows(2)
            .map(|pair| pair_key(&pair[0], &pair[1]))
            .collect();
        // Copy only what changed: the text up to the old split compared equal above.
        self.text.truncate(unchanged);
        self.text.push_str(&text[unchanged..]);
    }
}

/// Push the whole words of `text` (each followed by a separator, or a character of a script
/// written without spaces) and return how many bytes they take; a word still being written at
/// the end is left out. In scripts written without spaces each character is a word: there is
/// no other boundary to compare on.
fn settled_words(text: &str, words: &mut Vec<String>) -> usize {
    let mut word = String::new();
    let mut settled = 0;
    for (at, c) in text.char_indices() {
        if unspaced(c) {
            if !word.is_empty() {
                words.push(std::mem::take(&mut word));
            }
            if c.is_alphanumeric() {
                words.push(c.to_string());
            }
            settled = at + c.len_utf8();
        } else if c.is_alphanumeric() {
            word.extend(c.to_lowercase());
        } else {
            if !word.is_empty() {
                words.push(std::mem::take(&mut word));
            }
            settled = at + c.len_utf8();
        }
    }
    settled
}

/// A word pair, as a key: a set of keys costs no allocation per pair, and two pairs sharing a
/// key by chance would at worst count one more pair in common.
fn pair_key(first: &str, second: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    (first, second).hash(&mut hasher);
    hasher.finish()
}

/// A system turn, as it last read.
struct Heard {
    turn_id: u64,
    text: Words,
    source: Words,
    start_ms: u64,
    end_ms: u64,
    seen: Instant,
}

/// The microphone turn being spoken.
struct MicTurn {
    turn_id: u64,
    start_ms: u64,
    /// Whether the windows show it: it has had text, and has not been withdrawn since.
    shown: bool,
    /// Withdrawn as an echo: only its final may bring it back.
    withdrawn: bool,
    text: Words,
    source: Words,
}

/// A microphone line waiting its turn in the lane's queue: a finished line waiting for the
/// system line it may repeat, or a caption kept back behind one.
struct Held {
    caption: OwnedCaption,
    text: Words,
    source: Words,
    /// When it goes as it is, if nothing has matched it by then.
    until: Instant,
    /// Whether the windows show it, so that dropping it takes a withdrawal.
    shown: bool,
    /// Matched: dropped when its turn in the queue comes.
    echo: bool,
}

impl Held {
    /// A caption with nothing to wait for of its own, only lines ahead of it.
    fn behind(caption: OwnedCaption, shown: bool, echo: bool, now: Instant) -> Self {
        Self {
            caption,
            text: Words::default(),
            source: Words::default(),
            until: now,
            shown,
            echo,
        }
    }
}

#[derive(Default)]
struct Lane {
    heard: Vec<Heard>,
    mic: Option<MicTurn>,
    /// Oldest first; the windows show only the first.
    held: VecDeque<Held>,
    /// The latest caption of the microphone turn being spoken, kept back while lines are held
    /// ahead of it.
    waiting: Option<OwnedCaption>,
}

/// When, in session time, the system source sounded: loud on the meter, or captioned.
#[derive(Default)]
struct Sounding {
    spans: VecDeque<(u64, u64)>,
    /// Stretches ending before this have been forgotten.
    forgotten: u64,
}

impl Sounding {
    fn note(&mut self, start_ms: u64, end_ms: u64) {
        if let Some(last) = self.spans.back_mut()
            && start_ms <= last.1.saturating_add(MERGE_MS)
            && end_ms.saturating_add(MERGE_MS) >= last.0
        {
            last.0 = last.0.min(start_ms);
            last.1 = last.1.max(end_ms);
            return;
        }
        if self.spans.len() >= MAX_SPANS
            && let Some((_, end)) = self.spans.pop_front()
        {
            self.forgotten = self.forgotten.max(end);
        }
        self.spans.push_back((start_ms, end_ms));
    }

    /// Whether the system sounded around a microphone line: an echo of it would have.
    fn around(&self, start_ms: u64, end_ms: u64) -> bool {
        let from = start_ms.saturating_sub(SLACK_MS);
        let to = end_ms.saturating_add(SLACK_MS);
        from < self.forgotten
            || self
                .spans
                .iter()
                .any(|&(start, end)| start <= to && end >= from)
    }
}

#[derive(Default)]
pub struct Guard {
    /// Only a live session with both sources has anything to compare.
    enabled: bool,
    hold: Duration,
    lanes: [Lane; LANES],
    /// When the session clock read zero, to put meter readings in session time.
    started: Option<Instant>,
    /// When the meter heard the system source, as it happened.
    loud: Sounding,
    /// What the system source's captions cover. Kept apart from `loud`: a local engine's
    /// captions can trail the meter by a backlog, and each record merges only with its own
    /// latest stretch.
    captioned: Sounding,
}

impl Guard {
    /// Start over for a new session, whose clock starts `now`. `enabled` when it captures both
    /// sources; `hold` is how long a finished microphone line may wait.
    pub fn begin(&mut self, enabled: bool, hold: Duration, now: Instant) {
        *self = Self {
            enabled,
            hold,
            started: Some(now),
            ..Self::default()
        };
    }

    /// A meter reading: the system source sounding is a reason to hold a microphone line.
    pub fn level(&mut self, origin: Origin, rms: f32, now: Instant) {
        if self.enabled && origin == Origin::System && rms >= LOUD_RMS {
            let at = self.started.map_or(0, |started| {
                u64::try_from(now.saturating_duration_since(started).as_millis())
                    .unwrap_or(u64::MAX)
            });
            self.loud.note(at, at);
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

    /// What may go out now that time has passed: held lines whose time is up, committed as
    /// they were, in order, and what was kept back behind them.
    pub fn due(&mut self, now: Instant) -> Vec<OwnedCaption> {
        let mut out = Vec::new();
        for lane in &mut self.lanes {
            settle(lane, now, &mut out);
        }
        out
    }

    /// The session has ended: every held line is committed, and nothing more is compared.
    pub fn finish(&mut self) -> Vec<OwnedCaption> {
        let mut out = Vec::new();
        for lane in &mut self.lanes {
            while let Some(line) = lane.held.pop_front() {
                release(line, &mut out);
            }
            out.extend(lane.waiting.take());
        }
        *self = Self::default();
        out
    }

    fn system(&mut self, index: usize, caption: &Caption<'_>, now: Instant) -> Outcome {
        let spoke = !caption.text.trim().is_empty();
        if spoke {
            self.captioned.note(caption.start_ms, caption.end_ms);
        }
        let lane = &mut self.lanes[index];
        remember(&mut lane.heard, caption, now);
        for line in lane.held.iter_mut().filter(|line| !line.echo) {
            let said = &line.caption;
            if repeats(
                &lane.heard,
                &line.text,
                &line.source,
                said.start_ms,
                said.end_ms,
                true,
            ) {
                tracing::debug!(lane = index, turn = said.turn_id, "withdrew a held echo");
                line.echo = true;
            } else if spoke && caption.start_ms > said.end_ms.saturating_add(SLACK_MS) {
                // The system source has captioned past it: its echo would be here by now.
                line.until = line.until.min(now);
            }
        }
        // A microphone turn still being spoken is compared when its next caption arrives,
        // a fraction of a second away; withdrawing it from here would change nothing.
        let mut outcome = Outcome::pass();
        settle(lane, now, &mut outcome.before);
        outcome
    }

    fn microphone(&mut self, index: usize, caption: &Caption<'_>, now: Instant) -> Outcome {
        let sounding = self.loud.around(caption.start_ms, caption.end_ms)
            || self.captioned.around(caption.start_ms, caption.end_ms);
        let until = if sounding { now + self.hold } else { now };
        let lane = &mut self.lanes[index];
        let mut outcome = Outcome::default();
        if lane
            .mic
            .as_ref()
            .is_none_or(|turn| turn.turn_id != caption.turn_id)
        {
            // A turn that ended without a final: the windows commit what they show of it when
            // the next turn arrives, so what was kept back of it goes first, in its place.
            if let Some(left) = lane.waiting.take() {
                lane.held.push_back(Held::behind(left, false, false, now));
            }
            lane.mic = Some(MicTurn {
                turn_id: caption.turn_id,
                start_ms: caption.start_ms,
                shown: false,
                withdrawn: false,
                text: Words::default(),
                source: Words::default(),
            });
        }
        let queued = !lane.held.is_empty();
        let blank = caption.text.trim().is_empty();
        if !caption.final_ {
            let Some(turn) = lane.mic.as_mut() else {
                return Outcome::pass();
            };
            if turn.withdrawn {
                return outcome;
            }
            turn.text.read(caption.text);
            turn.source.read(caption.source_text);
            let echo = !blank
                && repeats(
                    &lane.heard,
                    &turn.text,
                    &turn.source,
                    caption.start_ms,
                    caption.end_ms,
                    false,
                );
            if echo {
                turn.withdrawn = true;
                lane.waiting = None;
                if turn.shown {
                    turn.shown = false;
                    let withdrawal = OwnedCaption::withdrawal(
                        turn.turn_id,
                        caption.lane,
                        turn.start_ms,
                        caption.end_ms,
                    );
                    if queued {
                        lane.held
                            .push_back(Held::behind(withdrawal, true, true, now));
                    } else {
                        outcome.before.push(withdrawal);
                    }
                }
            } else if queued {
                lane.waiting = Some(OwnedCaption::of(caption));
            } else {
                turn.shown |= !blank;
                outcome.pass = true;
            }
            return outcome;
        }
        let Some(mut turn) = lane.mic.take() else {
            return Outcome::pass();
        };
        lane.waiting = None;
        turn.text.read(caption.text);
        turn.source.read(caption.source_text);
        let echo = !blank
            && repeats(
                &lane.heard,
                &turn.text,
                &turn.source,
                caption.start_ms,
                caption.end_ms,
                true,
            );
        if echo {
            tracing::debug!(lane = index, turn = caption.turn_id, "dropped an echo");
        }
        let line = Held {
            caption: OwnedCaption {
                start_ms: turn.start_ms.min(caption.start_ms),
                ..OwnedCaption::of(caption)
            },
            text: turn.text,
            source: turn.source,
            until: if blank { now } else { until },
            shown: turn.shown,
            echo,
        };
        if !queued {
            if echo {
                if line.shown {
                    outcome.before.push(line.caption.withdrawn());
                }
                return outcome;
            }
            if blank || !sounding {
                outcome.pass = true;
                return outcome;
            }
        }
        if !line.echo && line.until > now {
            outcome.wake = Some(line.until);
        }
        lane.held.push_back(line);
        settle(lane, now, &mut outcome.before);
        outcome
    }
}

/// Let out, in order, the lane's held lines that are settled: dropped ones (withdrawn, if
/// shown), and those whose time is up. The first still waiting is shown as an interim; once
/// none is left, the caption kept back behind them goes too.
fn settle(lane: &mut Lane, now: Instant, out: &mut Vec<OwnedCaption>) {
    while let Some(first) = lane.held.front_mut() {
        if !first.echo && first.until > now {
            if !first.shown {
                first.shown = true;
                out.push(OwnedCaption {
                    final_: false,
                    ..first.caption.clone()
                });
            }
            return;
        }
        if let Some(line) = lane.held.pop_front() {
            release(line, out);
        }
    }
    if let Some(waiting) = lane.waiting.take() {
        if let Some(turn) = lane
            .mic
            .as_mut()
            .filter(|turn| turn.turn_id == waiting.turn_id)
        {
            turn.shown |= !waiting.text.trim().is_empty();
        }
        out.push(waiting);
    }
}

/// A held line leaving the queue: withdrawn if it was an echo the windows show, dropped if
/// they never did, and otherwise sent as it was.
fn release(line: Held, out: &mut Vec<OwnedCaption>) {
    if !line.echo {
        out.push(line.caption);
    } else if line.shown {
        out.push(line.caption.withdrawn());
    }
}

/// Keep a system caption's latest words to compare with, and forget what is too old.
fn remember(heard: &mut Vec<Heard>, caption: &Caption<'_>, now: Instant) {
    heard.retain(|turn| now.saturating_duration_since(turn.seen) < RETAIN);
    if let Some(turn) = heard
        .iter_mut()
        .find(|turn| turn.turn_id == caption.turn_id)
    {
        turn.text.read(caption.text);
        turn.source.read(caption.source_text);
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
    let mut turn = Heard {
        turn_id: caption.turn_id,
        text: Words::default(),
        source: Words::default(),
        start_ms: caption.start_ms,
        end_ms: caption.end_ms,
        seen: now,
    };
    turn.text.read(caption.text);
    turn.source.read(caption.source_text);
    heard.push(turn);
}

/// Whether a microphone line says what the system source captioned around the same time, in
/// the caption language or, where both engines give it, in the original speech. `finished`
/// lets a line of one or two words count.
fn repeats(
    heard: &[Heard],
    text: &Words,
    source: &Words,
    start_ms: u64,
    end_ms: u64,
    finished: bool,
) -> bool {
    let near: Vec<&Heard> = heard
        .iter()
        .filter(|turn| {
            turn.start_ms <= end_ms.saturating_add(SLACK_MS)
                && turn.end_ms.saturating_add(SLACK_MS) >= start_ms
        })
        .collect();
    if near.is_empty() {
        return false;
    }
    let texts: Vec<&Words> = near.iter().map(|turn| &turn.text).collect();
    if similar(text, &texts, finished) {
        return true;
    }
    let sources: Vec<&Words> = near.iter().map(|turn| &turn.source).collect();
    similar(source, &sources, finished)
}

/// Whether `said` is, nearly enough, part of `heard`, the system turns' tails read one after
/// the other. From three words on: at least two of its word pairs, and three in five, are
/// pairs `heard` has. Pairs rather than words, because any two sentences in a language share
/// their common words, and pairs they rarely share by chance; three in five, because two
/// engines hearing one voice through different microphones do not write every word alike. A
/// line of one or two words (finished lines only: a turn is that short as it starts) must
/// appear in `heard` exactly as it is.
fn similar(said: &Words, heard: &[&Words], finished: bool) -> bool {
    let heard: Vec<&Words> = heard
        .iter()
        .copied()
        .filter(|turn| !turn.tail.is_empty())
        .collect();
    // The pairs that span two turns, as if their tails were one text.
    let bridges: Vec<u64> = heard
        .windows(2)
        .filter_map(|two| Some(pair_key(two[0].tail.last()?, two[1].tail.first()?)))
        .collect();
    let has =
        |key: u64| bridges.contains(&key) || heard.iter().any(|turn| turn.pairs.contains(&key));
    let words = &said.tail;
    match words.len() {
        0 => false,
        1 => finished && heard.iter().any(|turn| turn.tail.contains(&words[0])),
        2 => finished && has(pair_key(&words[0], &words[1])),
        _ => {
            let total = words.len() - 1;
            let shared = words
                .windows(2)
                .filter(|pair| has(pair_key(&pair[0], &pair[1])))
                .count();
            shared >= 2 && shared * 5 >= total * 3
        }
    }
}

/// The app's guard, a Tauri state. Emits under its lock: see the module's notes.
#[derive(Default)]
pub struct EchoGuard(Mutex<Guard>);

impl EchoGuard {
    pub fn begin(&self, enabled: bool, hold: Duration) {
        lock(&self.0).begin(enabled, hold, Instant::now());
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

    fn caption<'a>(origin: Origin, turn: u64, text: &'a str, final_: bool, at: u64) -> Caption<'a> {
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

    fn mic(turn: u64, text: &str, final_: bool, at: u64) -> Caption<'_> {
        caption(Origin::Microphone, turn, text, final_, at)
    }

    fn system(turn: u64, text: &str, final_: bool, at: u64) -> Caption<'_> {
        caption(Origin::System, turn, text, final_, at)
    }

    /// A guard for a session whose clock started at `now`.
    fn guard(now: Instant) -> Guard {
        let mut guard = Guard::default();
        guard.begin(true, HOLD_CLOUD, now);
        guard
    }

    /// The loudspeakers sounding through the first `seconds` of the session, read every second.
    fn loud(guard: &mut Guard, now: Instant, seconds: u64) {
        for second in 0..=seconds {
            guard.level(Origin::System, 0.1, now + Duration::from_secs(second));
        }
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

    /// A text's words, as the guard compares them.
    fn tokens(text: &str) -> Vec<String> {
        let mut words = Words::default();
        words.read(text);
        words.tail
    }

    const SPEECH: &str = "The budget for next year will be presented on Thursday morning";
    const ROOM_HEARD: &str = "the budget for next year will be presented on thursday";

    /// Three lines of a Whisper window, as the room heard them.
    const WINDOW: [&str; 3] = [
        "the budget for next year will be presented on thursday",
        "the committee has approved the new schedule for the spring",
        "questions can be sent by email before the meeting ends",
    ];

    #[test]
    fn a_microphone_line_repeating_the_system_one_is_dropped() {
        let now = Instant::now();
        let mut guard = guard(now);
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
        let now = Instant::now();
        let mut guard = guard(now);
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
    fn an_interim_hovering_at_the_threshold_stays_withdrawn_until_its_final() {
        let now = Instant::now();
        let mut guard = guard(now);
        assert!(guard.caption(&mic(4, "the budget", false, 0), now).pass);
        guard.caption(&system(9, SPEECH, false, 100), now);
        let outcome = guard.caption(&mic(4, ROOM_HEARD, false, 0), now);
        assert_eq!(shape(&outcome.before), [(4, "", true)]);
        // More words, and too few of its pairs repeat the system now: still withdrawn, rather
        // than back on screen until the next caption takes it away again.
        let longer = "the budget for next year will be presented on thursday and \
                      frankly I do not think anyone here agrees with that plan";
        assert_eq!(
            guard.caption(&mic(4, longer, false, 0), now),
            Outcome::default()
        );
        assert_eq!(
            guard.caption(&mic(4, ROOM_HEARD, false, 0), now),
            Outcome::default()
        );
        assert_eq!(
            guard.caption(&mic(4, longer, false, 0), now),
            Outcome::default()
        );
        // The final is judged afresh and brings it back; the system sounded, so it waits.
        let outcome = guard.caption(&mic(4, longer, true, 0), now);
        assert!(!outcome.pass);
        assert_eq!(shape(&outcome.before), [(4, longer, false)]);
        assert_eq!(shape(&guard.due(now + HOLD_CLOUD)), [(4, longer, true)]);
    }

    #[test]
    fn a_finished_line_waits_while_the_system_sounds_and_goes_when_its_echo_arrives() {
        let now = Instant::now();
        let mut guard = guard(now);
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
        let now = Instant::now();
        let mut guard = guard(now);
        guard.level(Origin::System, 0.1, now);
        guard.caption(&mic(2, "I would like to ask a question", true, 0), now);
        guard.caption(&system(7, SPEECH, true, 200), now);
        assert!(guard.due(now + Duration::from_secs(1)).is_empty());
        let due = guard.due(now + HOLD_CLOUD);
        assert_eq!(shape(&due), [(2, "I would like to ask a question", true)]);
    }

    #[test]
    fn a_window_of_finished_lines_each_wait_for_their_own_echo() {
        let now = Instant::now();
        let mut guard = guard(now);
        loud(&mut guard, now, 8);
        // One Whisper window: three turns, back to back, before the system's window.
        let outcome = guard.caption(&mic(1, WINDOW[0], true, 0), now);
        assert_eq!(shape(&outcome.before), [(1, WINDOW[0], false)]);
        for (turn, text, at) in [(2, WINDOW[1], 3_000), (3, WINDOW[2], 6_000)] {
            let outcome = guard.caption(&mic(turn, text, true, at), now);
            assert!(!outcome.pass);
            assert!(outcome.before.is_empty(), "kept back behind the first");
            assert_eq!(outcome.wake, Some(now + HOLD_CLOUD));
        }
        // The system's window, as said; each drops its copy, and the next copy shows meanwhile.
        let later = now + Duration::from_secs(2);
        let said = [
            "The budget for next year will be presented on Thursday.",
            "The committee has approved the new schedule for the spring.",
            "Questions can be sent by email before the meeting ends.",
        ];
        let outcome = guard.caption(&system(11, said[0], true, 200), later);
        assert_eq!(
            shape(&outcome.before),
            [(1, "", true), (2, WINDOW[1], false)]
        );
        let outcome = guard.caption(&system(12, said[1], true, 3_200), later);
        assert_eq!(
            shape(&outcome.before),
            [(2, "", true), (3, WINDOW[2], false)]
        );
        let outcome = guard.caption(&system(13, said[2], true, 6_200), later);
        assert_eq!(shape(&outcome.before), [(3, "", true)]);
        assert!(guard.due(now + HOLD_CLOUD).is_empty());
    }

    #[test]
    fn a_window_of_finished_lines_nothing_repeats_is_committed_in_order() {
        let now = Instant::now();
        let mut guard = guard(now);
        loud(&mut guard, now, 8);
        for (turn, text, at) in [
            (1, WINDOW[0], 0),
            (2, WINDOW[1], 3_000),
            (3, WINDOW[2], 6_000),
        ] {
            guard.caption(&mic(turn, text, true, at), now);
        }
        let later = now + Duration::from_secs(1);
        let outcome = guard.caption(
            &system(11, "Let us take a short break now", true, 200),
            later,
        );
        assert!(outcome.before.is_empty());
        assert_eq!(
            shape(&guard.due(now + HOLD_CLOUD)),
            [
                (1, WINDOW[0], true),
                (2, WINDOW[1], true),
                (3, WINDOW[2], true)
            ]
        );
    }

    #[test]
    fn held_lines_leave_in_order_each_on_its_own_time() {
        let now = Instant::now();
        let mut guard = guard(now);
        guard.level(Origin::System, 0.1, now);
        guard.caption(&mic(1, WINDOW[0], true, 0), now);
        let second = now + Duration::from_secs(1);
        guard.caption(&mic(2, WINDOW[1], true, 1_000), second);
        // Spoken well after the system went quiet, so it need not wait, but it may not
        // overtake the lines ahead of it.
        let outcome = guard.caption(&mic(3, WINDOW[2], true, 20_000), second);
        assert!(!outcome.pass);
        assert!(outcome.before.is_empty());
        assert_eq!(outcome.wake, None);
        // The first one's time is up; the second shows while it waits out its own.
        assert_eq!(
            shape(&guard.due(now + HOLD_CLOUD)),
            [(1, WINDOW[0], true), (2, WINDOW[1], false)]
        );
        assert_eq!(
            shape(&guard.due(second + HOLD_CLOUD)),
            [(2, WINDOW[1], true), (3, WINDOW[2], true)]
        );
    }

    #[test]
    fn the_next_microphone_turn_waits_behind_the_held_one() {
        let now = Instant::now();
        let mut guard = guard(now);
        guard.level(Origin::System, 0.1, now);
        guard.caption(&mic(2, "I would like to ask a question", true, 0), now);
        // The windows would commit the held line as soon as another turn showed.
        for text in ["about the", "about the budget"] {
            let outcome = guard.caption(&mic(3, text, false, 1_500), now);
            assert_eq!(outcome, Outcome::default());
        }
        assert_eq!(
            shape(&guard.due(now + HOLD_CLOUD)),
            [
                (2, "I would like to ask a question", true),
                (3, "about the budget", false)
            ]
        );
        // Nothing is held any more: the turn goes on at once.
        assert!(
            guard
                .caption(
                    &mic(3, "about the budget please", false, 1_500),
                    now + HOLD_CLOUD
                )
                .pass
        );
    }

    #[test]
    fn a_system_caption_past_a_held_line_lets_it_go() {
        let now = Instant::now();
        let mut guard = guard(now);
        guard.level(Origin::System, 0.1, now);
        guard.caption(&mic(1, "I would like to ask a question", true, 0), now);
        // Still in reach of the line: it may yet be followed by its echo.
        let outcome = guard.caption(&system(5, "Let us take a short break", true, 3_000), now);
        assert!(outcome.before.is_empty());
        // Past it: any echo of it would have come by now.
        let outcome = guard.caption(&system(6, "Welcome back everyone", true, 6_000), now);
        assert_eq!(
            shape(&outcome.before),
            [(1, "I would like to ask a question", true)]
        );
    }

    #[test]
    fn a_turn_longer_than_the_tail_is_still_recognised() {
        let now = Instant::now();
        let mut guard = guard(now);
        // 400 words, more than the system's tail of 200 and the 333 at which the whole line
        // would share too few of its pairs with that tail.
        let said: Vec<String> = (0..400).map(|i| format!("word{i}")).collect();
        let said = said.join(" ");
        guard.caption(&system(1, &said, false, 0), now);
        let outcome = guard.caption(&mic(1, &said, false, 300), now);
        assert!(!outcome.pass, "the long interim is withdrawn");
        assert_eq!(
            guard.caption(&mic(1, &said, true, 300), now),
            Outcome::default()
        );
    }

    #[test]
    fn words_are_split_once_as_a_line_grows() {
        let mut words = Words::default();
        let mut text = String::new();
        for i in 0..500 {
            text.push_str(&format!("Word{i}, "));
            words.read(&text);
            // A word still being written.
            words.read(&format!("{text}Wor"));
        }
        let mut all: Vec<String> = (0..500).map(|i| format!("word{i}")).collect();
        all.push("wor".to_owned());
        assert_eq!(words.tail, all[all.len() - TAIL..]);
        assert!(words.settled.len() <= 2 * TAIL);
        // A revision is split again from the start.
        words.read("The budgie, said Bob");
        assert_eq!(words.tail, ["the", "budgie", "said", "bob"]);
        words.read("The budget said");
        assert_eq!(words.tail, ["the", "budget", "said"]);
    }

    #[test]
    fn local_speech_while_the_system_is_quiet_passes_at_once() {
        let now = Instant::now();
        let mut guard = guard(now);
        guard.level(Origin::System, 0.001, now);
        assert!(
            guard
                .caption(&mic(1, "Good morning everyone", true, 0), now)
                .pass
        );
        // Loud, but long enough before the line was said.
        guard.level(Origin::System, 0.2, now);
        let later = now + Duration::from_secs(9);
        assert!(
            guard
                .caption(&mic(2, "Let us begin", true, 9_000), later)
                .pass
        );
    }

    #[test]
    fn local_speech_over_the_system_is_kept() {
        let now = Instant::now();
        let mut guard = guard(now);
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
        let now = Instant::now();
        let mut guard = guard(now);
        guard.caption(&system(1, SPEECH, true, 0), now);
        // Someone in the room quoting it a minute later, the system quiet since: passed.
        assert!(guard.caption(&mic(1, ROOM_HEARD, true, 60_000), now).pass);
    }

    #[test]
    fn old_system_captions_are_forgotten() {
        let now = Instant::now();
        let mut guard = guard(now);
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
        let now = Instant::now();
        let mut guard = guard(now);
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
        guard.due(now + HOLD_CLOUD);
        // A turn starting with words the system said stays on screen until it says more.
        assert!(
            guard
                .caption(&mic(4, "thank you", false, 0), now + HOLD_CLOUD)
                .pass
        );
    }

    #[test]
    fn the_original_speech_is_compared_when_both_engines_give_it() {
        let now = Instant::now();
        let mut guard = guard(now);
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
        let now = Instant::now();
        let mut guard = guard(now);
        guard.caption(&system(1, "我们现在开始今天的会议", true, 0), now);
        assert_eq!(
            guard.caption(&mic(1, "现在开始今天的会", true, 0), now),
            Outcome::default()
        );
    }

    #[test]
    fn caption_languages_are_compared_separately() {
        let now = Instant::now();
        let mut guard = guard(now);
        guard.caption(&system(1, SPEECH, true, 0), now);
        let mut second = mic(1, ROOM_HEARD, true, 0);
        second.lane = 1;
        // Lane 1 has heard nothing from the system; it is held only because the system spoke.
        assert!(!guard.caption(&second, now).before.is_empty());
    }

    #[test]
    fn a_session_without_both_sources_passes_everything() {
        let now = Instant::now();
        let mut guard = Guard::default();
        guard.begin(false, HOLD_CLOUD, now);
        guard.level(Origin::System, 0.5, now);
        guard.caption(&system(1, SPEECH, true, 0), now);
        assert!(guard.caption(&mic(1, ROOM_HEARD, true, 0), now).pass);
    }

    #[test]
    fn finishing_commits_what_is_held_and_stops_comparing() {
        let now = Instant::now();
        let mut guard = guard(now);
        guard.level(Origin::System, 0.1, now);
        guard.caption(&mic(2, "One last question", true, 0), now);
        guard.caption(&mic(3, "and one more", false, 2_000), now);
        assert_eq!(
            shape(&guard.finish()),
            [(2, "One last question", true), (3, "and one more", false)]
        );
        guard.caption(&system(1, SPEECH, true, 0), now);
        assert!(guard.caption(&mic(3, ROOM_HEARD, true, 0), now).pass);
    }

    #[test]
    fn an_empty_final_passes_and_ends_the_turn() {
        let now = Instant::now();
        let mut guard = guard(now);
        assert!(
            guard
                .caption(&mic(1, "Hello there friends", false, 0), now)
                .pass
        );
        assert!(guard.caption(&mic(1, "", true, 0), now).pass);
        assert_eq!(withdrawal(1).text, "");
    }
}
