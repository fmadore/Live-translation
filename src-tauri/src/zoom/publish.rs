//! What of the app's captions goes to Zoom, and in what pieces. Pure: no I/O and no clock, so
//! every rule is tested on plain captions.
//!
//! Zoom shows each caption it receives as it is, so only text that can no longer change may be
//! sent: attendees must never read a hypothesis the engine later revises. A final caption is
//! always settled. An interim one is settled up to its last complete sentence, but only from an
//! engine that builds a turn by appending (see `Publisher::interims_grow`); waiting for the end
//! of a long Gemini Live Translate turn would hold a whole paragraph back.

use crate::types::{Caption, Origin};

/// The longest caption posted to Zoom, in characters. Zoom documents no limit; this follows
/// Microsoft's guidance for Teams captions, two short lines of a caption bar.
pub(super) const CAPTION_CHARS: usize = 120;

/// How long an interim run with no sentence end may grow before its finished words are shared
/// anyway, in characters: a speaker who never pauses for a full stop still reaches the meeting.
const RUN_CHARS: usize = 160;

/// The shortest settled interim piece worth a caption of its own, in characters. A "Yes." on
/// its own would flash past; it waits to go out with the sentence after it.
const MIN_INTERIM_CHARS: usize = 20;

/// Sentence terminators that end a sentence only when whitespace follows, so `3.5` and `e.g`
/// inside a word do not.
const TERMINATORS: [char; 4] = ['.', '!', '?', '…'];

/// Full-width terminators. Chinese and Japanese put no space after them, so any following
/// character confirms the sentence has ended.
const FULL_WIDTH_TERMINATORS: [char; 3] = ['。', '！', '？'];

/// Closing quotes and brackets, which belong to the sentence they close: `He said "Stop." Then`.
const CLOSERS: [char; 11] = ['"', '\'', '”', '’', '»', ')', ']', '」', '』', '）', '】'];

/// Which captions a share sends: one caption language, from the sources the operator chose.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Selection {
    pub lane: u8,
    pub origins: Vec<Origin>,
}

impl Selection {
    pub(super) fn wants(&self, caption: &Caption<'_>) -> bool {
        caption.lane == self.lane && self.origins.contains(&caption.origin)
    }
}

/// How far one stream's current turn has been shared.
#[derive(Debug)]
struct Stream {
    origin: Origin,
    lane: u8,
    turn_id: u64,
    /// Byte offset into the turn's text: everything before it has been published.
    published: usize,
}

/// Per-stream progress through the caption session's turns. Reset when a session starts,
/// because turn ids start again with it.
#[derive(Debug, Default)]
pub(super) struct Publisher {
    /// Whether this session's engine only ever appends to an interim caption, so the start of
    /// one can be trusted before its turn ends. False by default: finals only is always safe.
    interims_grow: bool,
    streams: Vec<Stream>,
}

impl Publisher {
    pub(super) fn new(interims_grow: bool) -> Self {
        Self {
            interims_grow,
            streams: Vec::new(),
        }
    }

    /// The text `caption` settles that has not been published yet, trimmed, or `None`.
    pub(super) fn take<'t>(&mut self, caption: &Caption<'t>) -> Option<&'t str> {
        let text = caption.text;
        if text.trim().is_empty() || !(caption.final_ || self.interims_grow) {
            return None;
        }
        let at = self.stream(caption);
        let published = self.streams[at].published;
        let Some(unsent) = text.get(published..) else {
            // Only an engine that went back on text it had appended gets here. Whatever it
            // says now, the part already shown in Zoom cannot be corrected, so say nothing more
            // for this turn rather than guess where the new text resumes.
            tracing::debug!(
                origin = ?caption.origin,
                lane = caption.lane,
                turn_id = caption.turn_id,
                published,
                len = text.len(),
                "caption no longer extends what Zoom was sent; skipping the rest of its turn"
            );
            if caption.final_ {
                self.streams.swap_remove(at);
            }
            return None;
        };
        if caption.final_ {
            self.streams.swap_remove(at);
            let rest = unsent.trim();
            return (!rest.is_empty()).then_some(rest);
        }
        let cut = settled(unsent)?;
        let piece = unsent[..cut].trim();
        if piece.chars().count() < MIN_INTERIM_CHARS {
            return None;
        }
        self.streams[at].published += cut;
        Some(piece)
    }

    /// The index of `caption`'s stream, started over at the beginning of its turn when the turn
    /// is new to it.
    fn stream(&mut self, caption: &Caption<'_>) -> usize {
        let found = self
            .streams
            .iter()
            .position(|s| s.origin == caption.origin && s.lane == caption.lane);
        match found {
            Some(at) => {
                let stream = &mut self.streams[at];
                if stream.turn_id != caption.turn_id {
                    stream.turn_id = caption.turn_id;
                    stream.published = 0;
                }
                at
            }
            None => {
                self.streams.push(Stream {
                    origin: caption.origin,
                    lane: caption.lane,
                    turn_id: caption.turn_id,
                    published: 0,
                });
                self.streams.len() - 1
            }
        }
    }
}

/// Where the settled part of an interim's unsent text ends, as a byte offset: after its last
/// complete sentence, or, in a run of more than `RUN_CHARS` with no sentence end, before its
/// last word, which may still be growing.
fn settled(unsent: &str) -> Option<usize> {
    if let Some(&end) = sentence_ends(unsent).last() {
        return Some(end);
    }
    if unsent.chars().count() > RUN_CHARS {
        return unsent.rfind(char::is_whitespace);
    }
    None
}

/// The byte offset just past each confirmed sentence end in `text`, in order: a run of
/// terminators and closing marks followed by whitespace, or by anything at all when the run
/// holds a full-width terminator. A terminator at the very end confirms nothing yet.
fn sentence_ends(text: &str) -> Vec<usize> {
    let mut ends = Vec::new();
    let mut chars = text.char_indices().peekable();
    while let Some((at, c)) = chars.next() {
        let mut full_width = FULL_WIDTH_TERMINATORS.contains(&c);
        if !(full_width || TERMINATORS.contains(&c)) {
            continue;
        }
        // `?!`, `...` and `.”` are one sentence end, after their last mark.
        let mut end = at + c.len_utf8();
        while let Some(&(next_at, next)) = chars.peek() {
            let terminator = FULL_WIDTH_TERMINATORS.contains(&next);
            if !(terminator || TERMINATORS.contains(&next) || CLOSERS.contains(&next)) {
                break;
            }
            full_width |= terminator;
            end = next_at + next.len_utf8();
            chars.next();
        }
        if chars
            .peek()
            .is_some_and(|&(_, next)| full_width || next.is_whitespace())
        {
            ends.push(end);
        }
    }
    ends
}

/// Cut published text into Zoom captions of at most `CAPTION_CHARS` characters, on one line.
/// Whole sentences are packed together while they fit; a longer sentence is split at its last
/// space before the limit, and a word longer than the limit (Chinese and Japanese have no
/// spaces) at the limit itself.
pub(super) fn chunk(text: &str) -> Vec<String> {
    // Zoom shows one line per caption: newlines and runs of spaces become one space.
    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut pieces = Vec::new();
    let mut rest = text.as_str();
    while !rest.is_empty() {
        // The offset of the first character past the limit; none when the rest fits.
        let Some((limit, _)) = rest.char_indices().nth(CAPTION_CHARS) else {
            pieces.push(rest.to_owned());
            break;
        };
        let cut = sentence_ends(rest)
            .into_iter()
            .take_while(|&end| end <= limit)
            .last()
            .or_else(|| {
                if rest[limit..].starts_with(' ') {
                    Some(limit)
                } else {
                    rest[..limit].rfind(' ')
                }
            })
            .filter(|&cut| cut > 0)
            .unwrap_or(limit);
        let (piece, tail) = rest.split_at(cut);
        pieces.push(piece.trim_end().to_owned());
        rest = tail.trim_start();
    }
    pieces
}

#[cfg(test)]
mod tests {
    use super::*;

    fn caption(origin: Origin, lane: u8, turn_id: u64, text: &str, final_: bool) -> Caption<'_> {
        Caption {
            turn_id,
            text,
            source_text: "",
            final_,
            origin,
            lane,
            start_ms: 0,
            end_ms: 0,
        }
    }

    fn interim(turn_id: u64, text: &str) -> Caption<'_> {
        caption(Origin::Microphone, 0, turn_id, text, false)
    }

    fn final_(turn_id: u64, text: &str) -> Caption<'_> {
        caption(Origin::Microphone, 0, turn_id, text, true)
    }

    /// Feed captions through a publisher and collect what it would send.
    fn published(interims_grow: bool, captions: &[Caption<'_>]) -> Vec<String> {
        let mut publisher = Publisher::new(interims_grow);
        captions
            .iter()
            .filter_map(|caption| publisher.take(caption).map(str::to_owned))
            .collect()
    }

    #[test]
    fn a_final_after_shared_interims_sends_only_the_rest() {
        assert_eq!(
            published(
                true,
                &[
                    interim(1, "We are glad you could join us."),
                    interim(1, "We are glad you could join us. Today we"),
                    interim(1, "We are glad you could join us. Today we look at"),
                    final_(
                        1,
                        "We are glad you could join us. Today we look at the budget."
                    ),
                ]
            ),
            [
                "We are glad you could join us.",
                "Today we look at the budget."
            ]
        );
    }

    /// Gemini Transcribe Live revises its hypothesis, and the demo and Whisper send finals
    /// only: nothing interim may reach Zoom from them, however settled it looks.
    #[test]
    fn an_engine_that_revises_shares_finals_only() {
        assert_eq!(
            published(
                false,
                &[
                    interim(1, "Their going to the station. And then"),
                    interim(1, "They're going to the station. And then"),
                    final_(1, "They're going to the station. And then home."),
                ]
            ),
            ["They're going to the station. And then home."]
        );
    }

    #[test]
    fn a_new_turn_starts_from_its_beginning() {
        assert_eq!(
            published(
                true,
                &[
                    interim(1, "The first turn has one sentence. Then"),
                    // Turn 1 never finalized here; turn 2 is new text from offset 0.
                    interim(2, "A second turn begins right here. And"),
                    final_(2, "A second turn begins right here. And ends."),
                    // A final with no interims before it is shared whole.
                    final_(3, "Short."),
                ]
            ),
            [
                "The first turn has one sentence.",
                "A second turn begins right here.",
                "And ends.",
                "Short."
            ]
        );
    }

    #[test]
    fn an_interim_is_cut_after_its_last_complete_sentence_closing_quotes_included() {
        assert_eq!(
            published(
                true,
                &[
                    // The last terminator has nothing after it yet: it may still become "3.5".
                    interim(1, "Growth was about 3."),
                    interim(1, "Growth was about 3.5 percent. She said \"Stop!\" Then"),
                ]
            ),
            ["Growth was about 3.5 percent. She said \"Stop!\""]
        );
    }

    #[test]
    fn a_full_width_terminator_ends_a_sentence_without_a_space() {
        assert_eq!(
            published(
                true,
                &[
                    interim(1, "今日は会議にご参加いただきありがとうございます。"),
                    interim(
                        1,
                        "今日は会議にご参加いただきありがとうございます。「始めましょう。」次に"
                    ),
                ]
            ),
            ["今日は会議にご参加いただきありがとうございます。「始めましょう。」"]
        );
    }

    #[test]
    fn a_long_run_with_no_sentence_end_is_shared_up_to_its_last_whole_word() {
        let run = "and so we went on talking ".repeat(7); // 182 characters
        let growing = format!("{run}abo");
        let mut publisher = Publisher::new(true);
        let first = publisher.take(&interim(1, &growing)).unwrap();
        assert_eq!(first, run.trim_end());
        // The half-word stays behind and goes out complete.
        let finished = format!("{run}about it.");
        assert_eq!(publisher.take(&final_(1, &finished)), Some("about it."));
    }

    #[test]
    fn a_settled_piece_shorter_than_twenty_characters_waits_for_more() {
        assert_eq!(
            published(
                true,
                &[
                    interim(1, "Yes. And"),
                    interim(1, "Yes. And then we went home. So"),
                ]
            ),
            ["Yes. And then we went home."]
        );
        // A final goes whatever its length.
        assert_eq!(published(true, &[final_(1, "Yes.")]), ["Yes."]);
    }

    #[test]
    fn only_the_chosen_lane_and_sources_are_wanted() {
        let selection = Selection {
            lane: 1,
            origins: vec![Origin::System],
        };
        assert!(selection.wants(&caption(Origin::System, 1, 1, "x", true)));
        assert!(!selection.wants(&caption(Origin::System, 0, 1, "x", true)));
        assert!(!selection.wants(&caption(Origin::Microphone, 1, 1, "x", true)));
    }

    /// Two sources and two lanes are separate streams; one's progress is not another's.
    #[test]
    fn each_source_and_lane_keeps_its_own_progress() {
        let first = "The microphone speaks first here. ";
        let mut publisher = Publisher::new(true);
        let mic = caption(Origin::Microphone, 0, 1, first, false);
        assert_eq!(publisher.take(&mic), Some(first.trim()));
        let system = caption(Origin::System, 0, 1, first, true);
        assert_eq!(publisher.take(&system), Some(first.trim()));
        let other_lane = caption(Origin::Microphone, 1, 1, first, true);
        assert_eq!(publisher.take(&other_lane), Some(first.trim()));
        let mic_final = caption(Origin::Microphone, 0, 1, first, true);
        assert_eq!(publisher.take(&mic_final), None);
    }

    #[test]
    fn blank_captions_are_ignored() {
        assert!(published(true, &[final_(1, "  \n ")]).is_empty());
        assert!(published(true, &[interim(1, "")]).is_empty());
    }

    /// An offset that falls inside a character, or past the end, must neither panic nor send
    /// a mangled tail.
    #[test]
    fn a_turn_that_went_back_on_its_text_publishes_nothing_more() {
        let mut publisher = Publisher::new(true);
        let shared = "Tout est prêt pour la soirée d'été. Et";
        assert_eq!(
            publisher.take(&interim(1, shared)),
            Some("Tout est prêt pour la soirée d'été.")
        );
        // The offset now points just past "été."; a revision that spells the apostrophe as
        // the typographic one, two bytes longer, moves it into the middle of the last "é".
        let revised = "Tout est prêt pour la soirée d’été. Et";
        assert!(!revised.is_char_boundary(shared.find(" Et").unwrap()));
        assert_eq!(publisher.take(&interim(1, revised)), None);
        // Past the end of a final that came back shorter.
        assert_eq!(publisher.take(&final_(1, "Tout")), None);
        // The turn is forgotten with its final; the next one starts clean.
        assert_eq!(publisher.take(&final_(2, "Fin.")), Some("Fin."));
    }

    #[test]
    fn short_text_is_one_caption_on_one_line() {
        assert_eq!(
            chunk("Hello\nworld.\r\n  How   are you?"),
            ["Hello world. How are you?"]
        );
    }

    #[test]
    fn sentences_are_packed_while_they_fit() {
        let a = "The first sentence is about forty characters.";
        let b = "The second one is a little bit longer than that.";
        let c = "The third will not fit with the first two.";
        let pieces = chunk(&format!("{a} {b} {c}"));
        assert_eq!(pieces, [format!("{a} {b}"), c.to_owned()]);
    }

    #[test]
    fn a_long_sentence_is_split_at_its_last_space_before_the_limit() {
        let sentence = "word ".repeat(60); // 300 characters
        let pieces = chunk(&sentence);
        assert!(
            pieces.iter().all(|p| p.chars().count() <= CAPTION_CHARS),
            "{pieces:?}"
        );
        assert!(
            pieces.iter().all(|p| p.split(' ').all(|w| w == "word")),
            "{pieces:?}"
        );
        assert_eq!(pieces.join(" "), sentence.trim_end());
        // A space exactly at the limit still makes a full-length caption.
        let exact = format!("{} tail", "x".repeat(CAPTION_CHARS));
        assert_eq!(
            chunk(&exact),
            ["x".repeat(CAPTION_CHARS), "tail".to_owned()]
        );
    }

    #[test]
    fn a_word_longer_than_the_limit_is_cut_at_a_character_boundary() {
        // Multibyte characters with no spaces, as Chinese and Japanese are written.
        let text = "語".repeat(300);
        let pieces = chunk(&text);
        let lengths: Vec<usize> = pieces.iter().map(|p| p.chars().count()).collect();
        assert_eq!(lengths, [120, 120, 60]);
        assert_eq!(pieces.concat(), text);
    }

    #[test]
    fn full_width_sentences_are_packed_and_cut_between_sentences() {
        let sentence = format!("{}。", "語".repeat(49)); // 50 characters
        let pieces = chunk(&sentence.repeat(3));
        assert_eq!(pieces, [sentence.repeat(2), sentence]);
    }
}
