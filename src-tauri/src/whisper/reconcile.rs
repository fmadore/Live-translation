//! What each recognised segment adds to the transcript. Pure: no model, no audio, so the
//! decisions at window boundaries can be tested directly.

pub use super::engine::RawSegment;
use crate::realtime::TurnCounter;

/// A caption ready for the transcript.
#[derive(Debug, PartialEq)]
pub struct Line {
    pub text: String,
    pub start_ms: u64,
    pub end_ms: u64,
    pub turn: u64,
}

// whisper.cpp's own defaults (`no_speech_thold` 0.6, `logprob_thold` -1.0, as in OpenAI's
// reference implementation). whisper.cpp applies them to a whole decoding pass, averaging the
// log-probability over every token in the window, so one confident sentence can carry a weak
// trailing segment past it — the "Thank you." that Whisper invents over room noise. Applying the
// same pair to each segment drops that tail and nothing else: confident speech has a mean
// log-probability well above -1 however noisy the window is judged to be, so the window's
// no-speech score alone (the filter this replaces) can no longer drop it.
const NO_SPEECH_THRESHOLD: f32 = 0.6;
const LOGPROB_THRESHOLD: f32 = -1.0;

fn is_silence(segment: &RawSegment) -> bool {
    segment.no_speech > NO_SPEECH_THRESHOLD && segment.mean_logprob < LOGPROB_THRESHOLD
}

/// Carries the last caption across windows, so the second of overlap between two windows is
/// captioned once and caption times only ever move forward.
pub struct Reconciler {
    /// Where each line's turn id comes from: the source's, so ids continue from the engine the
    /// source had before (`realtime::TurnCounter`).
    turns: TurnCounter,
    previous_text: String,
    previous_end: u64,
    /// The words of the last line captioned, and how many lines in a row have said them since.
    last_said: Vec<String>,
    repeats: usize,
}

impl Default for Reconciler {
    /// A source's first engine: its first line is turn 1.
    fn default() -> Self {
        Self::continuing(TurnCounter::starting_at(1))
    }
}

impl Reconciler {
    /// Lines numbered from `turns`.
    pub fn continuing(turns: TurnCounter) -> Self {
        Self {
            turns,
            previous_text: String::new(),
            previous_end: 0,
            last_said: Vec::new(),
            repeats: 0,
        }
    }

    /// `window_start_ms` is the window's capture time and `duration_ms` its real length; Whisper
    /// sees short windows padded, and timestamps in the padding are clamped back to the audio.
    pub fn accept(
        &mut self,
        window_start_ms: u64,
        duration_ms: u64,
        segment: &RawSegment,
    ) -> Option<Line> {
        if is_silence(segment) {
            return None;
        }
        // A negative timestamp counts as the window's start.
        let at = |cs: i64| {
            let ms = u64::try_from(cs).unwrap_or(0).saturating_mul(10);
            window_start_ms + ms.min(duration_ms)
        };
        let (start, end) = (at(segment.start_cs), at(segment.end_cs));
        if end <= self.previous_end {
            return None;
        }
        // U+FFFD is what lossy decoding leaves of a broken byte sequence: nothing to read.
        let cleaned = segment.text.replace('\u{FFFD}', "");
        if is_credit(&cleaned) {
            return None;
        }
        let collapsed = collapse_repeats(cleaned.trim());
        let text = trim_overlap(&self.previous_text, &collapsed, start < self.previous_end);
        if text.is_empty() {
            return None;
        }
        // The same line a third time running is Whisper looping over noise, not a speaker:
        // people repeat themselves, but rarely three windows in a row.
        let said = words(text);
        if said == self.last_said {
            self.repeats += 1;
        } else {
            self.last_said = said;
            self.repeats = 0;
        }
        if self.repeats >= 2 {
            self.previous_end = end;
            return None;
        }
        let turn = self.turns.next();
        let start = start.max(self.previous_end);
        let line = Line {
            text: text.to_owned(),
            start_ms: start,
            end_ms: end.max(start),
            turn,
        };
        self.previous_text = line.text.clone();
        self.previous_end = end;
        Some(line)
    }
}

/// How many times running a word, or a group of words, is repeated before it counts as a loop:
/// Whisper's known failure on noise and silence, which people saying "non, non, non" do not reach.
const LOOP_WORD: usize = 6;
const LOOP_GROUP: usize = 4;
/// The longest group compared, in words. A word of a script written without spaces is often two
/// characters, so there a group may hold twice as many.
const LOOP_SPAN: usize = 8;

/// The text with a word said over and over, or a group of up to eight words, kept once. Words
/// repeated a few times, as people do, are left. In scripts written without spaces each character
/// counts as a word, as in `echo.rs`: "谢谢谢谢谢谢谢谢" is a loop. Ported from Kalamu's
/// `whisper.rs`.
fn collapse_repeats(text: &str) -> String {
    let all = tokens(text);
    let keys: Vec<String> = all.iter().map(|token| normalise(token.text)).collect();
    let span = if text.chars().any(unspaced) {
        2 * LOOP_SPAN
    } else {
        LOOP_SPAN
    };
    let mut out: Vec<&Token> = Vec::new();
    let mut i = 0;
    'words: while i < all.len() {
        for n in 1..=span {
            let needed = if n == 1 { LOOP_WORD } else { LOOP_GROUP };
            if i + n * needed > all.len() || keys[i..i + n].iter().all(String::is_empty) {
                continue;
            }
            let mut times = 1;
            while i + (times + 1) * n <= all.len()
                && keys[i + times * n..i + (times + 1) * n] == keys[i..i + n]
            {
                times += 1;
            }
            if times >= needed {
                out.extend(&all[i..i + n]);
                i += times * n;
                continue 'words;
            }
        }
        out.push(&all[i]);
        i += 1;
    }
    let mut joined = String::with_capacity(text.len());
    for token in out {
        if token.spaced && !joined.is_empty() {
            joined.push(' ');
        }
        joined.push_str(token.text);
    }
    joined
}

/// A word as `collapse_repeats` compares it, with the punctuation that follows it.
struct Token<'a> {
    text: &'a str,
    /// Whether a space came before it; the words of an unspaced script join up again without.
    spaced: bool,
}

/// The words of `text`: what lies between spaces, except that each letter of a script written
/// without spaces is a word of its own. Punctuation stays with the word before it, so text in
/// other scripts splits exactly as on whitespace.
fn tokens(text: &str) -> Vec<Token<'_>> {
    let mut out = Vec::new();
    for word in text.split_whitespace() {
        let mut start = 0;
        // Whether the word being read has a letter yet, and whether that letter belongs to an
        // unspaced script, so that only punctuation may follow it.
        let (mut lettered, mut closed) = (false, false);
        for (index, c) in word.char_indices() {
            if !c.is_alphanumeric() {
                continue;
            }
            if closed || (lettered && unspaced(c)) {
                out.push(Token {
                    text: &word[start..index],
                    spaced: start == 0,
                });
                start = index;
            }
            lettered = true;
            closed = unspaced(c);
        }
        out.push(Token {
            text: &word[start..],
            spaced: start == 0,
        });
    }
    out
}

/// Subtitle credits Whisper learnt from its training data and writes over silence or music.
fn is_credit(text: &str) -> bool {
    let text = text.trim().to_lowercase();
    text.contains("amara.org")
        || ["sous-titres réalisés", "sous-titrage st", "subtitles by"]
            .iter()
            .any(|credit| text.starts_with(credit))
}

/// A line's words as compared for a loop: case and punctuation dropped.
fn words(text: &str) -> Vec<String> {
    text.split_whitespace()
        .map(normalise)
        .filter(|w| !w.is_empty())
        .collect()
}

/// Scripts written without spaces between words, where the only boundaries to compare on are
/// characters: Chinese, Japanese, Thai, Lao, Khmer, Myanmar and Tibetan. Korean uses spaces.
pub(crate) fn unspaced(c: char) -> bool {
    matches!(c,
        '\u{0E00}'..='\u{0EFF}' // Thai, Lao
        | '\u{0F00}'..='\u{0FFF}' // Tibetan
        | '\u{1000}'..='\u{109F}' // Myanmar
        | '\u{1780}'..='\u{17FF}' // Khmer
        | '\u{3000}'..='\u{30FF}' // CJK punctuation, Hiragana, Katakana
        | '\u{3100}'..='\u{312F}' // Bopomofo
        | '\u{31F0}'..='\u{31FF}' // Katakana extensions
        | '\u{3400}'..='\u{4DBF}' // CJK extension A
        | '\u{4E00}'..='\u{9FFF}' // CJK unified ideographs
        | '\u{F900}'..='\u{FAFF}' // CJK compatibility ideographs
        | '\u{FF66}'..='\u{FF9F}' // Half-width Katakana
        | '\u{20000}'..='\u{2FA1F}' // CJK extensions B onwards
    )
}

/// A word as compared across the boundary: case-folded, punctuation dropped, so "We," at the
/// start of one window matches "we" at the end of the last.
pub(super) fn normalise(word: &str) -> String {
    word.chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

/// Remove from the start of `current` what `previous` already ended with. Only reconciles an
/// actual time overlap; a repeated phrase later in a meeting is valid.
pub fn trim_overlap<'a>(previous: &str, current: &'a str, overlaps: bool) -> &'a str {
    if !overlaps {
        return current;
    }
    // On the first letter: an opening "「" or "…" says nothing about the script.
    if current
        .chars()
        .find(|c| c.is_alphanumeric())
        .is_some_and(unspaced)
    {
        trim_characters(previous, current)
    } else {
        trim_words(previous, current)
    }
}

fn trim_words<'a>(previous: &str, current: &'a str) -> &'a str {
    let before: Vec<String> = previous
        .split_whitespace()
        .map(normalise)
        .filter(|w| !w.is_empty())
        .collect();
    // Each comparable word of `current` with the byte offset just past it; pure punctuation
    // ("—", "…") has nothing to compare and is skipped.
    let mut words = Vec::new();
    let mut rest = current;
    while let Some(start) = rest.find(|c: char| !c.is_whitespace()) {
        let word_len = rest[start..]
            .find(char::is_whitespace)
            .unwrap_or(rest.len() - start);
        let end = start + word_len;
        let word = normalise(&rest[start..end]);
        let offset = current.len() - rest.len() + end;
        if !word.is_empty() {
            words.push((word, offset));
        }
        rest = &rest[end..];
    }
    // Longest match first: with "yes yes" on both sides a one-word match would leave a "yes".
    for k in (1..=before.len().min(words.len())).rev() {
        let suffix = &before[before.len() - k..];
        if words[..k].iter().map(|(w, _)| w).eq(suffix.iter()) {
            let remainder = &current[words[k - 1].1..];
            // Drop punctuation left dangling at the cut: ", and then" reads as "and then".
            return remainder
                .trim_start_matches(|c: char| c.is_whitespace() || !c.is_alphanumeric());
        }
    }
    current
}

fn trim_characters<'a>(previous: &str, current: &'a str) -> &'a str {
    // Letters and digits only, as `trim_words` compares them: Whisper ends nearly every window
    // in these scripts with "。", "、" or "？", which no start of the next window would match.
    // Each letter comes with the byte offset just past it.
    fn letters(text: &str) -> Vec<(char, usize)> {
        text.char_indices()
            .filter(|(_, c)| c.is_alphanumeric())
            .flat_map(|(at, c)| {
                c.to_lowercase()
                    .map(move |lower| (lower, at + c.len_utf8()))
            })
            .collect()
    }
    let before: Vec<char> = letters(previous).into_iter().map(|(c, _)| c).collect();
    let after = letters(current);
    // Longest match first, all of `current` included: repeated characters such as "你好你好"
    // otherwise leave an extra "你好" at the next window boundary. Two characters is a word in
    // these scripts; one is too often a coincidence.
    for k in (2..=before.len().min(after.len())).rev() {
        if after[..k]
            .iter()
            .map(|(c, _)| c)
            .eq(&before[before.len() - k..])
        {
            // Drop punctuation left dangling at the cut: "、ですね" reads as "ですね".
            return current[after[k - 1].1..]
                .trim_start_matches(|c: char| c.is_whitespace() || !c.is_alphanumeric());
        }
    }
    current
}

#[cfg(test)]
mod tests {
    use super::*;

    fn segment(text: &str, start_cs: i64, end_cs: i64) -> RawSegment {
        RawSegment {
            text: text.to_owned(),
            start_cs,
            end_cs,
            no_speech: 0.1,
            mean_logprob: -0.3,
        }
    }

    #[test]
    fn overlap_is_unicode_safe_and_does_not_remove_later_repetitions() {
        assert_eq!(
            trim_overlap("bonjour tout le monde", "tout le monde et bienvenue", true),
            "et bienvenue"
        );
        assert_eq!(trim_overlap("你好世界朋友", "世界朋友再见", true), "再见");
        assert_eq!(trim_overlap("yes yes", "yes yes", false), "yes yes");
    }

    #[test]
    fn complete_overlap_wins_over_a_repeated_partial_suffix() {
        assert_eq!(trim_overlap("yes yes", "yes yes", true), "");
        assert_eq!(trim_overlap("il dit oui oui", "oui oui", true), "");
        assert_eq!(trim_overlap("你好你好你好", "你好你好", true), "");
    }

    #[test]
    fn overlap_matches_whole_words_not_fragments_of_them() {
        // Matching characters turned this into "we go": "here" is the end of "there".
        assert_eq!(
            trim_overlap("and then we went there", "here we go", true),
            "here we go"
        );
        assert_eq!(
            trim_overlap("a seaside", "side by side", true),
            "side by side"
        );
    }

    #[test]
    fn overlap_ignores_case_and_punctuation() {
        assert_eq!(
            trim_overlap("and so we", "We will start now.", true),
            "will start now."
        );
        assert_eq!(
            trim_overlap("bonjour tout le monde.", "Monde, et bienvenue", true),
            "et bienvenue"
        );
        assert_eq!(
            trim_overlap("C'est l'heure.", "l’heure — on commence", true),
            "on commence"
        );
        assert_eq!(trim_overlap("Thanks, everyone!", "Everyone.", true), "");
    }

    #[test]
    fn scripts_without_spaces_fall_back_to_characters() {
        assert_eq!(
            trim_overlap("今日はいい天気", "いい天気ですね", true),
            "ですね"
        );
        assert_eq!(trim_overlap("สวัสดีครับ", "ครับวันนี้", true), "วันนี้");
        // One shared character is not enough to call it a repeat.
        assert_eq!(trim_overlap("我们", "们好", true), "们好");
    }

    #[test]
    fn punctuation_does_not_hide_an_overlap_in_scripts_without_spaces() {
        assert_eq!(
            trim_overlap("我们今天讨论预算。", "讨论预算，然后休息", true),
            "然后休息"
        );
        assert_eq!(
            trim_overlap("今日はいい天気。", "いい天気ですね", true),
            "ですね"
        );
        assert_eq!(
            trim_overlap("今日はいい天気。", "「いい天気」、ですね", true),
            "ですね"
        );
        assert_eq!(trim_overlap("สวัสดีครับ!", "ครับ วันนี้", true), "วันนี้");
        assert_eq!(trim_overlap("你好你好。", "你好你好。", true), "");
        // Punctuation does not count towards the two characters either.
        assert_eq!(trim_overlap("我们。", "们，好", true), "们，好");

        let mut reconciler = Reconciler::default();
        reconciler
            .accept(0, 10_000, &segment("今日はいい天気。", 600, 1_000))
            .unwrap();
        let line = reconciler
            .accept(9_000, 10_000, &segment("いい天気ですね。", 0, 300))
            .unwrap();
        assert_eq!(line.text, "ですね。");
    }

    #[test]
    fn character_loops_are_cut_back_in_scripts_without_spaces() {
        // Chinese.
        assert_eq!(collapse_repeats("谢谢谢谢谢谢谢谢谢谢"), "谢");
        assert_eq!(
            collapse_repeats("他说谢谢大家谢谢大家谢谢大家谢谢大家谢谢大家。"),
            "他说谢谢大家"
        );
        assert_eq!(collapse_repeats("好好好，我知道了。"), "好好好，我知道了。");
        // Japanese: a ten-character phrase is a group of five words or so.
        assert_eq!(
            collapse_repeats(
                "ありがとうございますありがとうございますありがとうございますありがとうございます"
            ),
            "ありがとうございます"
        );
        assert_eq!(
            collapse_repeats("はいはいはい、わかりました。"),
            "はいはいはい、わかりました。"
        );
        // Thai.
        assert_eq!(collapse_repeats("ครับครับครับครับครับ"), "ครับ");
        assert_eq!(collapse_repeats("ดีดี ครับครับ ขอบคุณ"), "ดีดี ครับครับ ขอบคุณ");
        // Spaces around words of other scripts are kept as they were.
        assert_eq!(collapse_repeats("我用 iPhone 拍照"), "我用 iPhone 拍照");
    }

    #[test]
    fn segment_times_are_clamped_to_the_audio_and_never_go_backwards() {
        let mut reconciler = Reconciler::default();
        // Padding: a 600 ms window reports a segment running to 1 s.
        let line = reconciler
            .accept(5_000, 600, &segment("short phrase", 0, 100))
            .unwrap();
        assert_eq!((line.start_ms, line.end_ms, line.turn), (5_000, 5_600, 1));
        // Negative timestamps are read as the window start.
        let line = reconciler
            .accept(6_000, 10_000, &segment("next", -5, 200))
            .unwrap();
        assert_eq!((line.start_ms, line.end_ms, line.turn), (6_000, 8_000, 2));
        // A segment ending before the last caption ended adds nothing.
        assert!(
            reconciler
                .accept(6_500, 10_000, &segment("old", 0, 100))
                .is_none()
        );
        // One that starts inside the last caption is moved to its end.
        let line = reconciler
            .accept(7_000, 10_000, &segment("later", 50, 300))
            .unwrap();
        assert_eq!((line.start_ms, line.end_ms), (8_000, 10_000));
    }

    #[test]
    fn the_second_of_overlap_between_windows_is_captioned_once() {
        let mut reconciler = Reconciler::default();
        reconciler
            .accept(0, 10_000, &segment(" and then we went there", 600, 1_000))
            .unwrap();
        // The next window starts a second earlier and hears the same last words.
        let line = reconciler
            .accept(
                9_000,
                10_000,
                &segment(" We went there, and here we go", 0, 300),
            )
            .unwrap();
        assert_eq!(line.text, "and here we go");
        assert_eq!(line.start_ms, 10_000);
        // A segment that is nothing but the overlap is dropped without advancing the turn.
        let mut reconciler = Reconciler::default();
        reconciler.accept(0, 10_000, &segment("yes yes", 800, 1_000));
        assert!(
            reconciler
                .accept(9_000, 10_000, &segment("Yes, yes.", 0, 150))
                .is_none()
        );
        let line = reconciler
            .accept(9_000, 10_000, &segment("no", 150, 200))
            .unwrap();
        assert_eq!(line.turn, 2);
    }

    #[test]
    fn invalid_utf8_leaves_readable_text_and_never_an_empty_caption() {
        let mut reconciler = Reconciler::default();
        let line = reconciler
            .accept(0, 5_000, &segment("caf\u{FFFD} au lait", 0, 100))
            .unwrap();
        assert_eq!(line.text, "caf au lait");
        assert!(
            reconciler
                .accept(0, 5_000, &segment(" \u{FFFD}\u{FFFD} ", 100, 200))
                .is_none()
        );
    }

    #[test]
    fn loops_are_cut_back_and_what_people_say_twice_is_kept() {
        assert_eq!(
            collapse_repeats("Il a dit merci merci merci merci merci merci merci."),
            "Il a dit merci"
        );
        assert_eq!(
            collapse_repeats("on y va, on y va, on y va, on y va, on y va et puis"),
            "on y va, et puis"
        );
        assert_eq!(
            collapse_repeats("non, non, non, je ne crois pas"),
            "non, non, non, je ne crois pas"
        );
        let mut reconciler = Reconciler::default();
        let looped = reconciler
            .accept(
                0,
                5_000,
                &segment("thank you thank you thank you thank you thank you", 0, 100),
            )
            .unwrap();
        assert_eq!(looped.text, "thank you");
    }

    #[test]
    fn subtitle_credits_are_dropped() {
        let mut reconciler = Reconciler::default();
        for credit in [
            "Sous-titres réalisés par la communauté d'Amara.org",
            " Subtitles by the Amara.org community",
            "Sous-titrage ST' 501",
        ] {
            assert!(
                reconciler
                    .accept(0, 5_000, &segment(credit, 0, 100))
                    .is_none()
            );
        }
        assert!(
            reconciler
                .accept(0, 5_000, &segment("Merci à Amara pour son exposé", 0, 100))
                .is_some()
        );
    }

    #[test]
    fn the_same_line_a_third_time_running_is_dropped() {
        let mut reconciler = Reconciler::default();
        let mut shown = Vec::new();
        for (i, text) in ["Merci.", "Merci.", "merci", "Merci !", "Au revoir."]
            .iter()
            .enumerate()
        {
            let start = i as u64 * 10_000;
            if let Some(line) = reconciler.accept(start, 10_000, &segment(text, 0, 100)) {
                shown.push(line.text);
            }
        }
        assert_eq!(shown, ["Merci.", "Merci.", "Au revoir."]);
    }

    #[test]
    fn only_unlikely_text_in_a_window_judged_silent_is_dropped() {
        let mut reconciler = Reconciler::default();
        let judged = |text: &str, no_speech: f32, mean_logprob: f32| RawSegment {
            no_speech,
            mean_logprob,
            ..segment(text, 0, 100)
        };
        // The window looks like noise and the model was unsure: a hallucination.
        assert!(
            reconciler
                .accept(0, 5_000, &judged("Thank you.", 0.9, -1.4))
                .is_none()
        );
        // The same window score with confident text is speech, and kept.
        assert!(
            reconciler
                .accept(0, 5_000, &judged("Merci à tous", 0.9, -0.2))
                .is_some()
        );
        // Unsure text in a window that clearly holds speech is kept too.
        assert!(
            reconciler
                .accept(1_000, 5_000, &judged("mumbled", 0.2, -1.6))
                .is_some()
        );
    }
}
