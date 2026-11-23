//! OpenAI's Realtime transcription session with `gpt-live-transcribe`: same-language subtitles
//! from the OpenAI key the translation engine already uses. See `docs/openai-transcribe-api.md`.
//!
//! Like Mistral and Gemini Transcribe, the transcript *is* the audience caption, so it goes in
//! the accumulator's `text`. Two things are this session's own. Its deltas stream as speech
//! arrives, but it has no turn detection: the client commits the audio buffer at the end of each
//! turn (`after_audio`), from the loudness of the audio it sends. And a commit only bounds the
//! server's audio item: its completion repeats the deltas already shown, so caption lines close
//! on the idle boundary, held open while a sentence is unfinished, as Mistral's do.

use std::collections::HashSet;
use std::time::Duration;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio_tungstenite::tungstenite::handshake::client::Request;

use crate::realtime::{
    CaptionUpdate, MessageControl, MessageOutcome, RealtimeProtocol, TurnAccumulator,
    bearer_request, ends_sentence, parse_or_log, provider_error,
};
use crate::types::Origin;

pub const DEFAULT_OPENAI_LIVE_TRANSCRIBE_MODEL: &str = "gpt-live-transcribe";

/// Close a line once its text has been still this long, unless its sentence is unfinished.
const FINALIZE_AFTER: Duration = Duration::from_millis(900);

/// The commit that ends an audio turn.
const COMMIT: &str = r#"{"type":"input_audio_buffer.commit"}"#;

#[derive(Clone)]
pub struct OpenAiTranscribeConfig {
    pub api_key: String,
    pub model: String,
    pub host: String,
    /// The session's glossary, sent as `keywords`.
    pub keywords: Vec<String>,
    /// The spoken language the operator chose (a Whisper code), sent as the one entry of
    /// `languages`. `None`: the model detects it.
    pub language: Option<String>,
    pub origin: Origin,
}

impl OpenAiTranscribeConfig {
    /// The protocol the realtime runner drives for this source, with nothing sent yet.
    pub fn into_client(self) -> OpenAiTranscribeClient {
        OpenAiTranscribeClient {
            config: self,
            turn: Turn::default(),
            pending: 0,
            closing: false,
            streamed: HashSet::new(),
        }
    }

    fn ws_url(&self) -> String {
        // No `model` in the URL: with one, OpenAI opens a conversation session, which refuses
        // the transcription update.
        format!("wss://{}/v1/realtime?intent=transcription", self.host)
    }

    fn session_update(&self) -> SessionUpdate<'_> {
        let keywords = self
            .keywords
            .iter()
            .filter_map(|term| keyword(term))
            .collect();
        let languages = self
            .language
            .as_deref()
            .map(|code| vec![language_code(code)])
            .unwrap_or_default();
        SessionUpdate {
            kind: "session.update",
            session: Session {
                kind: "transcription",
                audio: Audio {
                    input: Input {
                        format: Format {
                            kind: "audio/pcm",
                            rate: 24_000,
                        },
                        transcription: Transcription {
                            model: &self.model,
                            keywords,
                            languages,
                        },
                        turn_detection: None,
                    },
                },
            },
        }
    }
}

/// A glossary entry as a keyword: OpenAI refuses the whole update for a keyword holding `<`,
/// `>` or a line break, so those go and the rest of the entry is kept.
fn keyword(term: &str) -> Option<String> {
    let kept: String = term
        .chars()
        .map(|c| {
            if matches!(c, '<' | '>' | '\r' | '\n') {
                ' '
            } else {
                c
            }
        })
        .collect();
    let kept = kept.split_whitespace().collect::<Vec<_>>().join(" ");
    (!kept.is_empty()).then_some(kept)
}

/// The spoken-language selector's code as OpenAI spells it. The selector lists Whisper's
/// languages, which are ISO 639-1 apart from Whisper's own `jw` for Javanese (ISO `jv`) and
/// Hawaiian's `haw`, sent as it is.
fn language_code(code: &str) -> String {
    match code {
        "jw" => "jv".to_string(),
        other => other.to_string(),
    }
}

// ---- Wire types (outgoing) -------------------------------------------------------------

#[derive(Debug, Serialize)]
struct SessionUpdate<'a> {
    #[serde(rename = "type")]
    kind: &'static str,
    session: Session<'a>,
}

#[derive(Debug, Serialize)]
struct Session<'a> {
    #[serde(rename = "type")]
    kind: &'static str,
    audio: Audio<'a>,
}

#[derive(Debug, Serialize)]
struct Audio<'a> {
    input: Input<'a>,
}

#[derive(Debug, Serialize)]
struct Input<'a> {
    format: Format,
    transcription: Transcription<'a>,
    /// Always sent, as `null`: the default session has server VAD, which this model refuses.
    turn_detection: Option<()>,
}

#[derive(Debug, Serialize)]
struct Format {
    #[serde(rename = "type")]
    kind: &'static str,
    rate: u32,
}

#[derive(Debug, Serialize)]
struct Transcription<'a> {
    model: &'a str,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    keywords: Vec<String>,
    /// At least one entry when present, so left out rather than sent empty.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    languages: Vec<String>,
}

#[derive(Debug, Serialize)]
struct AudioAppend {
    #[serde(rename = "type")]
    kind: &'static str,
    audio: String,
}

// ---- Wire types (incoming) -------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct ServerEvent {
    #[serde(rename = "type", default)]
    kind: String,
    #[serde(default)]
    item_id: Option<String>,
    #[serde(default)]
    delta: Option<String>,
    #[serde(default)]
    transcript: Option<String>,
    #[serde(default)]
    error: Option<serde_json::Value>,
}

// ---- Turns ------------------------------------------------------------------------------

/// The client's end-of-turn detection, from the loudness of the 24 kHz chunks it sends. It
/// decides only where the audio buffer is committed, never what is captioned: a turn committed
/// early or late still streams every word. So a plain level threshold will do, and in a room too
/// noisy for it a turn is committed at `MAX_TURN` all the same.
#[derive(Debug, Default)]
struct Turn {
    /// Audio sent since the last commit.
    buffered: Duration,
    /// Audio since the last chunk loud enough to be speech.
    quiet: Duration,
    /// Whether any chunk since the last commit was.
    heard: bool,
}

/// About −40 dBFS: speech near a microphone is well above it, a quiet room below.
const SPEECH_RMS: f64 = 328.0;
/// The pause that ends a turn, the default silence of OpenAI's own server VAD.
const END_OF_TURN: Duration = Duration::from_millis(500);
/// No shorter turn is committed on a pause: a breath is not the end of a turn.
const MIN_TURN: Duration = Duration::from_secs(1);
/// A turn is committed at this length whatever the audio sounds like.
const MAX_TURN: Duration = Duration::from_secs(20);

impl Turn {
    /// Count one chunk of 24 kHz PCM16; whether the buffer should be committed after it.
    fn push(&mut self, pcm_le: &[u8]) -> bool {
        let samples = pcm_le.len() / 2;
        let length = Duration::from_micros(samples as u64 * 1_000_000 / 24_000);
        self.buffered += length;
        if rms(pcm_le) >= SPEECH_RMS {
            self.heard = true;
            self.quiet = Duration::ZERO;
        } else {
            self.quiet += length;
        }
        let ended = self.heard && self.quiet >= END_OF_TURN && self.buffered >= MIN_TURN;
        if ended || self.buffered >= MAX_TURN {
            *self = Self::default();
            true
        } else {
            false
        }
    }
}

#[expect(
    clippy::cast_precision_loss,
    reason = "a 100 ms chunk is 2,400 samples; the level needs no more precision than f64 has"
)]
fn rms(pcm_le: &[u8]) -> f64 {
    let (samples, _) = pcm_le.as_chunks::<2>();
    if samples.is_empty() {
        return 0.0;
    }
    let sum: f64 = samples
        .iter()
        .map(|&pair| f64::from(i16::from_le_bytes(pair)).powi(2))
        .sum();
    (sum / samples.len() as f64).sqrt()
}

// ---- The protocol -----------------------------------------------------------------------

/// One OpenAI transcription source: its configuration, plus what the current connection has
/// sent and been told.
pub struct OpenAiTranscribeClient {
    config: OpenAiTranscribeConfig,
    turn: Turn,
    /// Commits sent on this connection whose completion has not arrived.
    pending: usize,
    /// Whether the close drain has committed the last of the audio.
    closing: bool,
    /// Items whose deltas have been shown, until their completion. A completion for an item
    /// that streamed none carries text no delta did.
    streamed: HashSet<String>,
}

impl RealtimeProtocol for OpenAiTranscribeClient {
    const NAME: &'static str = "OpenAI";

    fn origin(&self) -> Origin {
        self.config.origin
    }

    fn connect_request(&self) -> Result<Request> {
        bearer_request(Self::NAME, &self.config.ws_url(), &self.config.api_key)
    }

    fn setup_json(&self) -> Result<Option<String>> {
        Ok(Some(serde_json::to_string(&self.config.session_update())?))
    }

    fn audio_json(&self, base64_pcm: String) -> Result<String> {
        Ok(serde_json::to_string(&AudioAppend {
            kind: "input_audio_buffer.append",
            audio: base64_pcm,
        })?)
    }

    fn after_audio(&mut self, pcm_le: &[u8]) -> Option<String> {
        self.turn.push(pcm_le).then(|| {
            self.pending += 1;
            COMMIT.to_string()
        })
    }

    /// `session.updated` confirms the update, so a refused keyword or language code ends the
    /// source during setup rather than after its audio has started.
    fn wait_for_setup_complete(&self) -> bool {
        true
    }

    /// Commit what is still buffered, so its last words come back before the socket closes.
    /// Nothing buffered, nothing to send: committing an empty buffer is an error.
    fn closing_json(&mut self) -> Result<Vec<String>> {
        if self.turn.buffered.is_zero() {
            return Ok(Vec::new());
        }
        self.turn = Turn::default();
        self.pending += 1;
        self.closing = true;
        Ok(vec![COMMIT.to_string()])
    }

    fn drain_complete(&self, _outcome: &MessageOutcome) -> bool {
        self.closing && self.pending == 0
    }

    fn handle_message(&mut self, text: &str, acc: &mut TurnAccumulator) -> MessageOutcome {
        let Some(event) = parse_or_log::<ServerEvent>(Self::NAME, text) else {
            return MessageOutcome::default();
        };

        match event.kind.as_str() {
            // A new connection: nothing committed on it yet.
            "session.created" => {
                self.turn = Turn::default();
                self.pending = 0;
                self.closing = false;
                self.streamed.clear();
            }
            "session.updated" => return MessageOutcome::setup_complete(),
            "conversation.item.input_audio_transcription.delta" => {
                let delta = event.delta.as_deref().unwrap_or_default();
                // Each delta brings its own leading space; a line does not start with one.
                let delta = if acc.text.is_empty() {
                    delta.trim_start()
                } else {
                    delta
                };
                if !delta.is_empty() {
                    if let Some(item) = event.item_id {
                        self.streamed.insert(item);
                    }
                    acc.text.push_str(delta);
                    return MessageOutcome::activity(CaptionUpdate::Interim);
                }
            }
            // The deltas already carried this text, unless none came for the item.
            "conversation.item.input_audio_transcription.completed" => {
                self.pending = self.pending.saturating_sub(1);
                let streamed = event
                    .item_id
                    .is_some_and(|item| self.streamed.remove(&item));
                let transcript = event.transcript.as_deref().unwrap_or_default().trim();
                if !streamed && !transcript.is_empty() {
                    if !acc.text.is_empty() {
                        acc.text.push(' ');
                    }
                    acc.text.push_str(transcript);
                    return MessageOutcome::activity(CaptionUpdate::Final);
                }
            }
            // One turn lost, not the session: OpenAI keeps it open.
            "conversation.item.input_audio_transcription.failed" => {
                self.pending = self.pending.saturating_sub(1);
                if let Some(item) = &event.item_id {
                    self.streamed.remove(item);
                }
                tracing::warn!(
                    origin = ?self.config.origin,
                    error = ?event.error,
                    "OpenAI could not transcribe a turn"
                );
            }
            // The same payload as the translation session's, read the same way: the
            // 60-minute cap (`session_expired`) hands over, transient errors reconnect, and the
            // rest stop the source with OpenAI's message.
            "error" => {
                let Some(error) = event.error else {
                    return MessageOutcome::control(MessageControl::Fatal(
                        "OpenAI transcription error".to_string(),
                    ));
                };
                return provider_error(self, Some("OpenAI transcription error"), &error);
            }
            _ => {}
        }
        MessageOutcome::default()
    }

    fn finalize_after(&self) -> Option<Duration> {
        Some(FINALIZE_AFTER)
    }

    /// With no turn events to go by, an unfinished sentence is the sign the speaker is still
    /// talking: its line stays open at the idle boundary, for up to the runner's `MAX_HOLD`.
    fn keeps_turn_open(&self, acc: &TurnAccumulator) -> bool {
        !acc.text.is_empty() && !ends_sentence(&acc.text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::realtime::Signal;
    use crate::realtime::test_support::{Emitted, Harness, handshake};

    /// The events of the live probe of 23 November 2026 (`docs/openai-transcribe-api.md`), as
    /// the server sent them: the English rehearsal recording, committed once 8 s in and once at
    /// its end.
    const RECORDED: &str = include_str!("fixtures/live-transcribe-en.jsonl");

    fn config(api_key: &str) -> OpenAiTranscribeConfig {
        OpenAiTranscribeConfig {
            api_key: api_key.to_string(),
            model: DEFAULT_OPENAI_LIVE_TRANSCRIBE_MODEL.to_string(),
            host: super::super::DEFAULT_OPENAI_HOST.to_string(),
            keywords: Vec::new(),
            language: None,
            origin: Origin::Microphone,
        }
    }

    fn harness() -> Harness<OpenAiTranscribeClient> {
        Harness::new(config("").into_client())
    }

    fn setup(config: OpenAiTranscribeConfig) -> serde_json::Value {
        serde_json::from_str(&config.into_client().setup_json().unwrap().unwrap()).unwrap()
    }

    /// 100 ms of 24 kHz PCM16 at a constant level.
    fn chunk(level: i16) -> Vec<u8> {
        level.to_le_bytes().repeat(2_400)
    }

    #[test]
    fn the_handshake_names_no_model_and_authenticates_with_a_bearer_header() {
        let request = config("sk-test").into_client().connect_request().unwrap();
        assert_eq!(
            handshake(request),
            "wss://api.openai.com/v1/realtime?intent=transcription\n\
             GET /v1/realtime?intent=transcription HTTP/1.1\r\n\
             Host: api.openai.com\r\n\
             Connection: Upgrade\r\n\
             Upgrade: websocket\r\n\
             Sec-WebSocket-Version: 13\r\n\
             Sec-WebSocket-Key: <key>\r\n\
             authorization: Bearer sk-test\r\n\
             \r\n"
        );
        let error = config("sk-\ntest")
            .into_client()
            .connect_request()
            .unwrap_err();
        assert_eq!(
            error.to_string(),
            "OpenAI API key is not a valid header value"
        );
    }

    /// The update the live probe saw accepted, field for field.
    #[test]
    fn the_session_update_turns_server_vad_off_and_leaves_out_unset_hints() {
        assert_eq!(
            setup(config("")),
            serde_json::json!({
                "type": "session.update",
                "session": {
                    "type": "transcription",
                    "audio": { "input": {
                        "format": { "type": "audio/pcm", "rate": 24000 },
                        "transcription": { "model": "gpt-live-transcribe" },
                        "turn_detection": null
                    }}
                }
            })
        );
    }

    #[test]
    fn the_glossary_goes_as_keywords_and_the_spoken_language_as_a_language_hint() {
        let mut config = config("");
        config.keywords = vec![
            "Université Joseph Ki-Zerbo".into(),
            "<IWAC>".into(),
            "two\r\nlines".into(),
            "<>".into(),
        ];
        config.language = Some("fr".into());
        let transcription = &setup(config.clone())["session"]["audio"]["input"]["transcription"];
        assert_eq!(
            transcription["keywords"],
            serde_json::json!(["Université Joseph Ki-Zerbo", "IWAC", "two lines"])
        );
        assert_eq!(transcription["languages"], serde_json::json!(["fr"]));
        assert!(transcription.get("language").is_none(), "never both");

        config.language = Some("jw".into());
        let transcription = &setup(config)["session"]["audio"]["input"]["transcription"];
        assert_eq!(transcription["languages"], serde_json::json!(["jv"]));
    }

    #[test]
    fn audio_goes_in_append_events() {
        let frame = config("").into_client().audio_json("AAA=".into()).unwrap();
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&frame).unwrap(),
            serde_json::json!({ "type": "input_audio_buffer.append", "audio": "AAA=" })
        );
    }

    /// The recorded session through the client: the setup is confirmed by the `session.updated`
    /// after `session.created`, every word streams into one growing line with no leading space,
    /// and the completions add nothing the deltas did not.
    #[test]
    fn the_recorded_session_streams_every_word_once() {
        let mut h = harness();
        let mut setups = 0;
        let mut last = String::new();
        for line in RECORDED.lines() {
            let outcome = h.send(line);
            setups += usize::from(outcome.signal == Signal::SetupComplete);
            assert!(
                matches!(outcome.control, MessageControl::Continue),
                "{line}"
            );
            assert_ne!(outcome.caption, CaptionUpdate::Final, "{line}");
            if let Some(caption) = h.captions.last() {
                last.clone_from(&caption.text);
            }
        }
        assert_eq!(setups, 2, "one per session.updated");
        assert!(h.captions.iter().all(|c| !c.text.starts_with(' ')));
        assert_eq!(
            last,
            "This is a rehearsal recording for the live caption system. A speaker is \
             presenting to an audience, and every sentence is transcribed and Shown on screen \
             as it is spoken. Captions appear a moment after the words and are corrected once \
             the sentence becomes clear. Use this recording to check the overlay before the \
             room fills up."
        );
        assert!(h.proto.streamed.is_empty(), "both items completed");
    }

    /// Between the recording's sentences the text stops for over a second, so the idle boundary
    /// closes each line there; mid-sentence it holds the line open.
    #[test]
    fn an_unfinished_sentence_keeps_its_line_open() {
        let mut h = harness();
        let open = |h: &Harness<OpenAiTranscribeClient>| h.proto.keeps_turn_open(&h.acc);
        assert!(!open(&h), "nothing to hold");
        h.send(r#"{"type":"conversation.item.input_audio_transcription.delta","item_id":"i1","delta":" Captions appear a"}"#);
        assert!(open(&h));
        h.send(r#"{"type":"conversation.item.input_audio_transcription.delta","item_id":"i1","delta":" moment later."}"#);
        assert!(!open(&h));
        h.finalize_idle();
        h.send(r#"{"type":"conversation.item.input_audio_transcription.delta","item_id":"i1","delta":" Next"}"#);
        assert_eq!(
            h.captions,
            [
                Emitted::interim(0, "Captions appear a", ""),
                Emitted::interim(0, "Captions appear a moment later.", ""),
                Emitted::final_(0, "Captions appear a moment later.", ""),
                Emitted::interim(1, "Next", ""),
            ]
        );
    }

    #[test]
    fn a_completion_without_deltas_supplies_its_transcript() {
        let mut h = harness();
        let outcome = h.send(r#"{"type":"conversation.item.input_audio_transcription.completed","item_id":"i9","content_index":0,"transcript":" Only the summary. ","usage":{"type":"duration","seconds":2}}"#);
        assert_eq!(outcome.caption, CaptionUpdate::Final);
        assert_eq!(h.captions, [Emitted::final_(0, "Only the summary.", "")]);
        // An empty one is not a caption.
        h.send(r#"{"type":"conversation.item.input_audio_transcription.completed","item_id":"i10","transcript":""}"#);
        assert_eq!(h.captions.len(), 1);
    }

    #[test]
    fn a_turn_that_fails_is_logged_and_the_session_goes_on() {
        let mut h = harness();
        let outcome = h.send(r#"{"type":"conversation.item.input_audio_transcription.failed","item_id":"msg_003","content_index":0,"error":{"type":"transcription_error","code":"audio_unintelligible","message":"The audio could not be transcribed.","param":null}}"#);
        assert!(matches!(outcome.control, MessageControl::Continue));
        assert!(h.captions.is_empty());
    }

    /// The translation session's error vocabulary, which this one shares.
    #[test]
    fn the_session_cap_hands_over_transient_errors_reconnect_and_the_rest_are_fatal() {
        let mut h = harness();
        for (error, expected) in [
            (
                r#"{"type":"invalid_request_error","code":"session_expired","message":"Your session hit the maximum duration of 60 minutes."}"#,
                "handover",
            ),
            (
                r#"{"type":"server_error","message":"The server had an error"}"#,
                "reconnect",
            ),
            (
                r#"{"type":"rate_limit_error","code":"rate_limit_exceeded"}"#,
                "reconnect",
            ),
            (
                r#"{"type":"rate_limit_error","code":"insufficient_quota"}"#,
                "fatal",
            ),
            (
                r#"{"type":"invalid_request_error","code":"invalid_api_key"}"#,
                "fatal",
            ),
            (
                r#"{"type":"invalid_request_error","code":"invalid_value","param":"session.audio.input.transcription.languages"}"#,
                "fatal",
            ),
        ] {
            let frame = format!(r#"{{"type":"error","error":{error}}}"#);
            let got = match h.send(&frame).control {
                MessageControl::Handover { .. } => "handover",
                MessageControl::Reconnect => "reconnect",
                MessageControl::Fatal(_) => "fatal",
                _ => "other",
            };
            assert_eq!(got, expected, "{error}");
        }
        assert!(matches!(
            h.send(r#"{"type":"error"}"#).control,
            MessageControl::Fatal(_)
        ));
    }

    /// A pause after speech commits the turn; silence alone, or a pause before a second of
    /// audio, does not, until the turn reaches its longest.
    #[test]
    fn a_turn_is_committed_at_a_pause_after_speech_or_at_its_longest() {
        let mut client = config("").into_client();
        let mut commits = |pattern: &[(i16, usize)]| {
            let mut at = Vec::new();
            let mut n = 0;
            for &(level, count) in pattern {
                for _ in 0..count {
                    n += 1;
                    if let Some(frame) = client.after_audio(&chunk(level)) {
                        assert_eq!(frame, COMMIT);
                        at.push(n);
                    }
                }
            }
            at
        };
        // One second of speech, then the pause: committed on its fifth quiet chunk.
        assert_eq!(commits(&[(3_000, 10), (20, 10)]), [15]);
        // A breath inside a sentence is not a turn's end.
        assert_eq!(
            commits(&[(3_000, 10), (20, 3), (3_000, 5), (20, 4)]),
            [0_usize; 0]
        );
        assert_eq!(
            commits(&[(20, 1)]),
            [1],
            "the fifth quiet chunk after speech"
        );
        // A short word, then a pause: held until a second of audio has gone.
        assert_eq!(commits(&[(3_000, 2), (20, 10)]), [10]);
        // Twenty seconds of a quiet room, or of a noisy one, commit all the same (counting the
        // 0.2 s left over from the turn above).
        assert_eq!(commits(&[(20, 200)]), [198]);
        assert_eq!(commits(&[(3_000, 200)]), [198]);
        assert_eq!(client.pending, 5);
    }

    /// Pause and Stop commit what is buffered and drain until every commit has completed; with
    /// nothing buffered there is nothing to commit.
    #[test]
    fn the_close_drain_commits_the_rest_and_ends_on_its_completion() {
        let mut h = harness();
        assert!(h.proto.closing_json().unwrap().is_empty());
        h.proto.after_audio(&chunk(3_000));
        assert_eq!(h.proto.closing_json().unwrap(), [COMMIT]);
        let delta = h.send(r#"{"type":"conversation.item.input_audio_transcription.delta","item_id":"i2","delta":" last words."}"#);
        assert!(!h.proto.drain_complete(&delta));
        let done = h.send(r#"{"type":"conversation.item.input_audio_transcription.completed","item_id":"i2","transcript":"last words."}"#);
        assert!(h.proto.drain_complete(&done));
        assert_eq!(h.captions, [Emitted::interim(0, "last words.", "")]);

        // A new connection starts with nothing pending and no drain.
        h.send(r#"{"type":"session.created","session":{"type":"transcription"}}"#);
        assert!(!h.proto.drain_complete(&MessageOutcome::default()));
        assert!(h.proto.closing_json().unwrap().is_empty());
    }

    #[test]
    fn the_level_of_a_chunk_is_its_root_mean_square() {
        assert!(rms(&[]).abs() < f64::EPSILON);
        assert!((rms(&chunk(-1_000)) - 1_000.0).abs() < 1e-9);
        let mixed: Vec<u8> = [3_i16, -4].iter().flat_map(|s| s.to_le_bytes()).collect();
        assert!((rms(&mixed) - 12.5_f64.sqrt()).abs() < 1e-9);
    }

    /// What `run_session` reported, for the runs through the real runner below.
    #[derive(Debug)]
    enum Seen {
        Running,
        Caption(std::time::Instant, String, bool),
        Problem(String),
    }

    struct Record(std::sync::mpsc::Sender<Seen>);

    impl crate::realtime::Events for Record {
        fn caption(&self, caption: crate::types::Caption<'_>) {
            let seen = Seen::Caption(
                std::time::Instant::now(),
                caption.text.to_string(),
                caption.final_,
            );
            let _ = self.0.send(seen);
        }

        fn status(&self, update: crate::types::StatusUpdate) {
            let seen = match update.message {
                Some(error) => Seen::Problem(format!("{:?}: {:?}", update.state, error.detail)),
                None if update.state == crate::types::SessionState::Running => Seen::Running,
                None => return,
            };
            let _ = self.0.send(seen);
        }
    }

    /// The client over plain `ws://` to a loopback address; everything else is its own.
    struct Loopback(OpenAiTranscribeClient, String);

    impl RealtimeProtocol for Loopback {
        const NAME: &'static str = OpenAiTranscribeClient::NAME;

        fn origin(&self) -> Origin {
            self.0.origin()
        }
        fn connect_request(&self) -> Result<Request> {
            bearer_request(Self::NAME, &self.1, &self.0.config.api_key)
        }
        fn setup_json(&self) -> Result<Option<String>> {
            self.0.setup_json()
        }
        fn audio_json(&self, base64_pcm: String) -> Result<String> {
            self.0.audio_json(base64_pcm)
        }
        fn after_audio(&mut self, pcm_le: &[u8]) -> Option<String> {
            self.0.after_audio(pcm_le)
        }
        fn wait_for_setup_complete(&self) -> bool {
            self.0.wait_for_setup_complete()
        }
        fn closing_json(&mut self) -> Result<Vec<String>> {
            self.0.closing_json()
        }
        fn drain_complete(&self, outcome: &MessageOutcome) -> bool {
            self.0.drain_complete(outcome)
        }
        fn handle_message(&mut self, text: &str, acc: &mut TurnAccumulator) -> MessageOutcome {
            self.0.handle_message(text, acc)
        }
        fn finalize_after(&self) -> Option<Duration> {
            self.0.finalize_after()
        }
        fn keeps_turn_open(&self, acc: &TurnAccumulator) -> bool {
            self.0.keeps_turn_open(acc)
        }
    }

    /// The type of the next text frame the client sent; `None` once it has closed.
    async fn next_frame(
        ws: &mut tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
    ) -> Option<String> {
        use futures_util::StreamExt;
        use tokio_tungstenite::tungstenite::Message;
        loop {
            match ws.next().await? {
                Ok(Message::Text(text)) => {
                    let value: serde_json::Value = serde_json::from_str(&text).unwrap();
                    return Some(value["type"].as_str().unwrap().to_string());
                }
                Ok(Message::Close(_)) | Err(_) => return None,
                Ok(_) => {}
            }
        }
    }

    /// Frame types from the client up to and including its next commit.
    async fn until_commit(
        ws: &mut tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
    ) -> Vec<String> {
        let mut kinds = Vec::new();
        while let Some(kind) = next_frame(ws).await {
            let commit = kind == "input_audio_buffer.commit";
            kinds.push(kind);
            if commit {
                break;
            }
        }
        kinds
    }

    /// The whole client through the real runner, against a loopback server that answers as
    /// OpenAI did in the recorded session: it waits for `session.updated`, commits after a
    /// pause in the speech, and on Stop commits the rest and closes once that has completed.
    // Two workers: the test waits on its captions with a blocking receive.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn the_runner_streams_commits_and_drains_against_a_loopback_server() {
        use futures_util::SinkExt;
        use tokio_tungstenite::tungstenite::Message;

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!(
            "ws://{}/v1/realtime?intent=transcription",
            listener.local_addr().unwrap()
        );
        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let mut ws = tokio_tungstenite::accept_async(stream).await.unwrap();
            assert_eq!(next_frame(&mut ws).await.unwrap(), "session.update");
            // `session.created`, then the update's `session.updated`, as recorded.
            for line in RECORDED.lines().take(2) {
                ws.send(Message::text(line)).await.unwrap();
            }
            let sent = until_commit(&mut ws).await;
            for line in [
                r#"{"type":"input_audio_buffer.committed","item_id":"i1","previous_item_id":null}"#,
                r#"{"type":"conversation.item.input_audio_transcription.delta","item_id":"i1","delta":" Hello there."}"#,
                r#"{"type":"conversation.item.input_audio_transcription.completed","item_id":"i1","transcript":"Hello there."}"#,
            ] {
                ws.send(Message::text(line)).await.unwrap();
            }
            let closing = until_commit(&mut ws).await;
            for line in [
                r#"{"type":"conversation.item.input_audio_transcription.delta","item_id":"i2","delta":" Bye."}"#,
                r#"{"type":"conversation.item.input_audio_transcription.completed","item_id":"i2","transcript":"Bye."}"#,
            ] {
                ws.send(Message::text(line)).await.unwrap();
            }
            let closed = next_frame(&mut ws).await.is_none();
            (sent, closing, closed)
        });

        let (events, seen) = std::sync::mpsc::channel();
        let (audio, audio_rx) = tokio::sync::mpsc::channel(5);
        let (_pause, pause_rx) = tokio::sync::watch::channel(false);
        let cancel = tokio_util::sync::CancellationToken::new();
        let session = tokio::spawn(crate::realtime::run_session(
            Record(events),
            Loopback(config("sk-test").into_client(), url),
            audio_rx,
            cancel.clone(),
            crate::timing::SessionClock::start(),
            pause_rx,
            0,
        ));
        let next = || seen.recv_timeout(Duration::from_secs(10)).unwrap();
        assert!(matches!(next(), Seen::Running));
        // A second of speech and half a second of quiet, then one more word for the drain.
        for level in [3_000; 10].into_iter().chain([0; 5]).chain([3_000]) {
            let pcm_le = chunk(level);
            audio
                .send(crate::audio::AudioChunk { pcm_le })
                .await
                .unwrap();
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        let mut captions: Vec<(String, bool)> = Vec::new();
        while !captions.iter().any(|(text, _)| text == "Hello there.") {
            if let Seen::Caption(_, text, final_) = next() {
                captions.push((text, final_));
            }
        }
        cancel.cancel();
        let (sent, closing, closed) = tokio::time::timeout(Duration::from_secs(10), server)
            .await
            .unwrap()
            .unwrap();
        tokio::time::timeout(Duration::from_secs(10), session)
            .await
            .unwrap()
            .unwrap();
        captions.extend(seen.try_iter().filter_map(|seen| match seen {
            Seen::Caption(_, text, final_) => Some((text, final_)),
            _ => None,
        }));

        let mut expected = vec!["input_audio_buffer.append".to_string(); 15];
        expected.push("input_audio_buffer.commit".to_string());
        assert_eq!(sent, expected);
        assert_eq!(
            closing,
            ["input_audio_buffer.append", "input_audio_buffer.commit"]
        );
        assert!(
            closed,
            "the client closed once the drain's commit completed"
        );
        let finals: Vec<&str> = captions
            .iter()
            .filter(|(_, final_)| *final_)
            .map(|(text, _)| text.as_str())
            .collect();
        assert_eq!(finals.join(" "), "Hello there. Bye.", "{captions:?}");
    }

    /// Both rehearsal recordings through the real runner and the real endpoint, at the pace of
    /// speech, printing the captions it closes. The French run also sends a glossary and the
    /// spoken language, to show OpenAI takes both where the client puts them.
    #[tokio::test]
    #[ignore = "streams the bundled fixtures to OpenAI and bills the key; run with --ignored"]
    async fn the_rehearsal_recordings_through_the_real_endpoint() {
        use crate::audio::fixture::load_fixture;
        use crate::audio::resample::{Resampler, f32_to_pcm16_le};
        use crate::audio::{AudioChunk, chunk_samples};

        for (file, keywords, language) in [
            ("rehearsal-en.wav", Vec::new(), None),
            (
                "rehearsal-fr.wav",
                vec!["sous-titres".to_string(), "Live Translation".to_string()],
                Some("fr".to_string()),
            ),
        ] {
            let key = crate::secrets::resolve_api_key(crate::types::Provider::OpenAi)
                .expect("save an OpenAI key in the app");
            let mut config = config(&key);
            drop(key);
            config.keywords = keywords;
            config.language = language;
            let samples = load_fixture(&std::path::Path::new("resources/fixtures").join(file))
                .expect("the bundled rehearsal fixture should load");
            let mut speech = Vec::new();
            Resampler::new(16_000, 24_000).process(&samples, &mut speech);

            let (events, seen) = std::sync::mpsc::channel();
            let (audio, audio_rx) = tokio::sync::mpsc::channel(5);
            let (_pause, pause_rx) = tokio::sync::watch::channel(false);
            let cancel = tokio_util::sync::CancellationToken::new();
            let session = tokio::spawn(crate::realtime::run_session(
                Record(events),
                config.into_client(),
                audio_rx,
                cancel.clone(),
                crate::timing::SessionClock::start(),
                pause_rx,
                0,
            ));
            // As in the room: the operator starts the session, then the speaker speaks. Audio
            // queued while the session sets up is dropped as stale when it starts streaming.
            loop {
                match seen.try_recv() {
                    Ok(Seen::Running) => break,
                    Ok(Seen::Problem(problem)) => panic!("{problem}"),
                    Ok(Seen::Caption(..)) => {}
                    Err(_) => tokio::time::sleep(Duration::from_millis(10)).await,
                }
            }
            let started = std::time::Instant::now();
            for chunk in speech.chunks(chunk_samples(24_000)) {
                let mut pcm_le = Vec::new();
                f32_to_pcm16_le(chunk, &mut pcm_le);
                audio.send(AudioChunk { pcm_le }).await.unwrap();
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            // Two seconds of the room going quiet, then Stop.
            for _ in 0..20 {
                let pcm_le = vec![0; 4_800];
                audio.send(AudioChunk { pcm_le }).await.unwrap();
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            cancel.cancel();
            session.await.unwrap();

            println!("{file}:");
            let mut finals = 0;
            for event in seen.try_iter() {
                match event {
                    Seen::Caption(at, text, true) => {
                        finals += 1;
                        println!("{:>8.1?}  {text}", at.duration_since(started));
                    }
                    Seen::Problem(problem) => panic!("{problem}"),
                    _ => {}
                }
            }
            assert!(finals > 0, "no caption from {file}");
        }
    }
}
