//! Gemini 3.5 Transcribe Live: same-language subtitles over the same `BidiGenerateContent`
//! socket the translate client uses, with the same 16 kHz PCM frame and the same API key.
//!
//! Unlike Live Translate, the transcript *is* the audience caption, so it goes in the
//! accumulator's `text`, where Live Translate puts its translation, and reaches the export
//! through the ordinary caption path — the arrangement Mistral already uses.

use std::time::Duration;

use anyhow::Result;
use tokio_tungstenite::tungstenite::handshake::client::Request;

use super::gemini_request;
use super::protocol::{
    AUDIO_STREAM_END, RealtimeInputMessage, ServerMessage, TranscribeSetupMessage,
};
use crate::realtime::{
    CaptionUpdate, MessageOutcome, RealtimeProtocol, TurnAccumulator, parse_or_log,
};
use crate::types::Origin;

pub const DEFAULT_TRANSCRIBE_MODEL: &str = "gemini-3.5-transcribe-live";

/// Safety net, not the normal path: the server finalizes a segment when the speaker pauses,
/// and this only fires if an interim hypothesis is left stranded with no final behind it.
/// Deliberately longer than Mistral's 900 ms so it never races the server's own
/// finalization and splits one utterance across two transcript lines.
const FINALIZE_AFTER: Duration = Duration::from_secs(3);

#[derive(Clone)]
pub struct GeminiTranscribeConfig {
    pub api_key: String,
    pub model: String,
    pub host: String,
    /// The chosen spoken language as `languageCodes` (see [`language_codes`]); empty for
    /// automatic detection.
    pub language_codes: &'static [&'static str],
    /// The session's names and terms, sent as `customVocabulary`.
    pub custom_vocabulary: Vec<String>,
    pub origin: Origin,
}

/// Transcribe Live's `languageCodes` for a spoken language from the app's list
/// (`src/lib/whisperLanguages.json`, which names languages the way Whisper does), or `None`
/// for a language Google's table does not list.
///
/// Google's table names locales (`fr-FR`, `cmn-Hans-CN`), not languages, so a language listed
/// under several regions gets all of them, and nothing outside the table is sent. Two codes
/// differ from Whisper's: Javanese is `jw` there and `jv-ID` here, and Tagalog is sent as
/// Filipino. Norwegian is Bokmål; Nynorsk has no row. Arabic has one row, Egyptian.
pub fn language_codes(spoken: &str) -> Option<&'static [&'static str]> {
    Some(match spoken {
        "af" => &["af-ZA"],
        "am" => &["am-ET"],
        "ar" => &["ar-EG"],
        "as" => &["as-IN"],
        "az" => &["az-AZ"],
        "be" => &["be-BY"],
        "bg" => &["bg-BG"],
        "bn" => &["bn-BD", "bn-IN"],
        "bs" => &["bs-BA"],
        "ca" => &["ca-ES"],
        "cs" => &["cs-CZ"],
        "da" => &["da-DK"],
        "de" => &["de-DE"],
        "el" => &["el-GR"],
        "en" => &["en-US", "en-GB", "en-IN"],
        "es" => &["es-419", "es-US"],
        "et" => &["et-EE"],
        "fa" => &["fa-IR"],
        "fi" => &["fi-FI"],
        "fr" => &["fr-FR"],
        "gl" => &["gl-ES"],
        "gu" => &["gu-IN"],
        "ha" => &["ha-NG"],
        "he" => &["he-IL"],
        "hi" => &["hi-IN"],
        "hr" => &["hr-HR"],
        "hu" => &["hu-HU"],
        "hy" => &["hy-AM"],
        "id" => &["id-ID"],
        "is" => &["is-IS"],
        "it" => &["it-IT"],
        "ja" => &["ja-JP"],
        "jw" => &["jv-ID"],
        "ka" => &["ka-GE"],
        "kk" => &["kk-KZ"],
        "km" => &["km-KH"],
        "kn" => &["kn-IN"],
        "ko" => &["ko-KR"],
        "ln" => &["ln-CD"],
        "lt" => &["lt-LT"],
        "lv" => &["lv-LV"],
        "mk" => &["mk-MK"],
        "ml" => &["ml-IN"],
        "mn" => &["mn-MN"],
        "mr" => &["mr-IN"],
        "ms" => &["ms-MY"],
        "mt" => &["mt-MT"],
        "my" => &["my-MM"],
        "ne" => &["ne-NP"],
        "nl" => &["nl-NL"],
        "no" => &["nb-NO"],
        "pa" => &["pa-IN", "pa-Guru-IN"],
        "pl" => &["pl-PL"],
        "pt" => &["pt-BR", "pt-PT"],
        "ro" => &["ro-RO"],
        "ru" => &["ru-RU"],
        "sd" => &["sd-Arab-IN"],
        "sk" => &["sk-SK"],
        "sl" => &["sl-SI"],
        "sr" => &["sr-RS"],
        "sv" => &["sv-SE"],
        "sw" => &["sw-KE"],
        "te" => &["te-IN"],
        "tg" => &["tg-TJ"],
        "th" => &["th-TH"],
        "tl" => &["fil-PH"],
        "tr" => &["tr-TR"],
        "uk" => &["uk-UA"],
        "uz" => &["uz-UZ"],
        "vi" => &["vi-VN"],
        "zh" => &["cmn-Hans-CN"],
        _ => return None,
    })
}

/// Refuse a spoken language Transcribe Live cannot be told about, rather than quietly fall
/// back to automatic detection the operator did not choose.
pub fn validate_spoken_language(spoken: Option<&str>) -> Result<()> {
    if let Some(code) = spoken {
        anyhow::ensure!(
            language_codes(code).is_some(),
            "Gemini Transcribe does not support spoken language {code}"
        );
    }
    Ok(())
}

impl RealtimeProtocol for GeminiTranscribeConfig {
    const NAME: &'static str = "Gemini Transcribe";

    fn origin(&self) -> Origin {
        self.origin
    }

    fn connect_request(&self) -> Result<Request> {
        gemini_request(Self::NAME, &self.host, &self.api_key)
    }

    fn setup_json(&self) -> Result<Option<String>> {
        Ok(Some(serde_json::to_string(
            &TranscribeSetupMessage::live_transcribe(
                &self.model,
                self.language_codes,
                &self.custom_vocabulary,
            ),
        )?))
    }

    fn audio_json(&self, base64_pcm: String) -> Result<String> {
        Ok(serde_json::to_string(&RealtimeInputMessage::pcm16(
            base64_pcm,
        ))?)
    }

    fn wait_for_setup_complete(&self) -> bool {
        true
    }

    fn closing_json(&mut self) -> Result<Vec<String>> {
        Ok(vec![AUDIO_STREAM_END.to_string()])
    }

    /// The authoritative final that follows `audioStreamEnd` is the flush it asked for. Waiting
    /// on past it held every Pause and Stop for the runner's whole safety timeout.
    fn drain_complete(&self, outcome: &MessageOutcome) -> bool {
        outcome.caption == CaptionUpdate::Final
    }

    fn handle_message(&mut self, text: &str, acc: &mut TurnAccumulator) -> MessageOutcome {
        let Some(msg) = parse_or_log::<ServerMessage>(Self::NAME, text) else {
            return MessageOutcome::default();
        };

        if let Some(outcome) = msg.control(self, "Gemini Transcribe error") {
            return outcome;
        }

        let Some(content) = msg.server_content else {
            return MessageOutcome::default();
        };

        // Empty finalized captions retract speculative text in both windows and the pending
        // transcript. Never archive an all-filler interim as confirmed speech.
        match apply_transcription(&content, acc) {
            Some(true) => MessageOutcome::activity(CaptionUpdate::Final),
            Some(false) => MessageOutcome::activity(CaptionUpdate::Interim),
            None => MessageOutcome::default(),
        }
    }

    fn finalize_after(&self) -> Option<Duration> {
        Some(FINALIZE_AFTER)
    }
}

/// Final text wins even when a frame also carries an interim hypothesis.
fn apply_transcription(
    content: &super::protocol::ServerContent,
    acc: &mut TurnAccumulator,
) -> Option<bool> {
    if let Some(final_text) = &content.input_transcription {
        acc.text.clone_from(&final_text.text);
        return Some(true);
    }
    if content.generation_complete.unwrap_or(false) || content.turn_complete.unwrap_or(false) {
        if acc.is_empty() {
            return None;
        }
        acc.text.clear();
        acc.source.clear();
        return Some(true);
    }
    if let Some(interim) = &content.interim_input_transcription {
        acc.text.clone_from(&interim.text);
        return Some(false);
    }
    None
}

#[cfg(test)]
mod smart_tests {
    use super::*;
    use crate::realtime::{MessageControl, Signal};
    fn content(raw: &str) -> super::super::protocol::ServerContent {
        serde_json::from_str::<ServerMessage>(raw)
            .unwrap()
            .server_content
            .unwrap()
    }
    #[test]
    fn smart_final_replaces_interim_even_in_the_same_frame() {
        let mut acc = TurnAccumulator::new(crate::timing::SessionClock::start(), 0);
        let frame = content(
            r#"{"serverContent":{"interimInputTranscription":{"text":"Um, I, I mean hello"},"inputTranscription":{"text":"Hello."}}}"#,
        );
        assert_eq!(apply_transcription(&frame, &mut acc), Some(true));
        assert_eq!(acc.text, "Hello.");
    }
    #[test]
    fn empty_smart_final_and_bare_close_retract_fillers() {
        for raw in [
            r#"{"serverContent":{"inputTranscription":{"text":""}}}"#,
            r#"{"serverContent":{"generationComplete":true}}"#,
        ] {
            let mut acc = TurnAccumulator::new(crate::timing::SessionClock::start(), 0);
            acc.text = "Um, uh".into();
            assert_eq!(apply_transcription(&content(raw), &mut acc), Some(true));
            assert!(acc.is_empty());
        }
    }

    #[test]
    fn interims_revise_the_segment_and_a_final_closes_it() {
        use crate::realtime::test_support::{Emitted, Harness};
        let mut h = Harness::new(GeminiTranscribeConfig {
            api_key: String::new(),
            model: DEFAULT_TRANSCRIBE_MODEL.to_string(),
            host: crate::gemini::DEFAULT_HOST.to_string(),
            language_codes: &[],
            custom_vocabulary: Vec::new(),
            origin: Origin::Microphone,
        });
        assert_eq!(
            h.send(r#"{"setupComplete":{}}"#).signal,
            Signal::SetupComplete
        );
        h.send(r#"{"serverContent":{"interimInputTranscription":{"text":"Hel"}}}"#);
        h.send(r#"{"serverContent":{"interimInputTranscription":{"text":"Hello there"}}}"#);
        let last = h.send(r#"{"serverContent":{"inputTranscription":{"text":"Hello there."}}}"#);
        assert_eq!(last.signal, Signal::TranscriptActivity);
        // A segment close with nothing pending is not a caption.
        h.send(r#"{"serverContent":{"generationComplete":true}}"#);
        assert_eq!(
            h.captions,
            [
                Emitted::interim(0, "Hel", ""),
                Emitted::interim(0, "Hello there", ""),
                Emitted::final_(0, "Hello there.", ""),
            ]
        );
        assert!(matches!(
            h.send(r#"{"goAway":{}}"#).control,
            MessageControl::Handover { .. }
        ));
    }

    /// The same endpoint and key as Live Translate; only the setup message tells them apart.
    #[test]
    fn the_handshake_matches_live_translate() {
        use crate::realtime::test_support::handshake;
        let mut config = GeminiTranscribeConfig {
            api_key: "gemini-test".to_string(),
            model: DEFAULT_TRANSCRIBE_MODEL.to_string(),
            host: crate::gemini::DEFAULT_HOST.to_string(),
            language_codes: &[],
            custom_vocabulary: Vec::new(),
            origin: Origin::Microphone,
        };
        let translate = crate::gemini::GeminiConfig {
            api_key: config.api_key.clone(),
            model: crate::gemini::DEFAULT_TRANSLATE_MODEL.to_string(),
            host: config.host.clone(),
            target_language_code: "fr".to_string(),
            origin: Origin::Microphone,
        };
        assert_eq!(
            handshake(config.connect_request().unwrap()),
            handshake(translate.connect_request().unwrap())
        );

        // The operator reads this as the reconnecting status's detail.
        config.host = "not a host".to_string();
        let error = config.connect_request().unwrap_err();
        assert_eq!(
            error.to_string(),
            "failed to build Gemini Transcribe request"
        );
    }

    /// What `live_probe::the_french_recording_under_a_right_and_a_wrong_hint` found on
    /// 23 November 2026: the server took this setup, and under an English hint still wrote the
    /// French recording in French, word for word as under the French one. The text is as
    /// recorded; the frame around it is the shape `live_probe` captured.
    const FRENCH_UNDER_AN_ENGLISH_HINT: &str = "Ceci est un enregistrement de répétition pour le système de sous-titre en direct. Une oratrice s'adresse à un public et chaque phrase est transcrite puis affichée à l'écran. Les sous-titres apparaissent juste après les mots, puis sont corrigés lorsque la phrase devient claire. Utilisez cet enregistrement pour régler l'affichage avant l'arrivée du public.";

    #[test]
    fn a_spoken_language_hint_is_sent_as_the_server_accepted_it() {
        use crate::realtime::test_support::{Emitted, Harness};
        let config = |spoken| GeminiTranscribeConfig {
            api_key: String::new(),
            model: DEFAULT_TRANSCRIBE_MODEL.to_string(),
            host: crate::gemini::DEFAULT_HOST.to_string(),
            language_codes: language_codes(spoken).unwrap(),
            custom_vocabulary: Vec::new(),
            origin: Origin::Microphone,
        };
        assert_eq!(
            config("fr").setup_json().unwrap().unwrap(),
            r#"{"setup":{"model":"models/gemini-3.5-transcribe-live","generationConfig":{"responseModalities":["TEXT"]},"inputAudioTranscription":{"languageCodes":["fr-FR"],"mode":"SMART"}}}"#
        );
        let mut h = Harness::new(config("en"));
        assert!(
            config("en")
                .setup_json()
                .unwrap()
                .unwrap()
                .contains(r#""languageCodes":["en-US","en-GB","en-IN"]"#)
        );
        assert_eq!(
            h.send(r#"{"setupComplete":{}}"#).signal,
            Signal::SetupComplete
        );
        let frame = serde_json::json!({
            "serverContent": { "inputTranscription": { "text": FRENCH_UNDER_AN_ENGLISH_HINT } }
        });
        h.send(&frame.to_string());
        assert_eq!(
            h.captions,
            [Emitted::final_(0, FRENCH_UNDER_AN_ENGLISH_HINT, "")]
        );
    }

    /// The "Supported languages" table of Google's Live transcription guide, read on
    /// 23 November 2026: 84 rows, `en-IN` among them twice.
    const GOOGLE_TABLE: [&str; 84] = [
        "af-ZA",
        "am-ET",
        "ar-EG",
        "hy-AM",
        "as-IN",
        "az-AZ",
        "be-BY",
        "bn-BD",
        "bn-IN",
        "bs-BA",
        "bg-BG",
        "rup-BG",
        "my-MM",
        "yue-Hant-HK",
        "ca-ES",
        "ceb",
        "km-KH",
        "hr-HR",
        "cs-CZ",
        "da-DK",
        "nl-NL",
        "en-GB",
        "en-IN",
        "en-US",
        "et-EE",
        "fa-IR",
        "fil-PH",
        "fi-FI",
        "fr-FR",
        "gl-ES",
        "ka-GE",
        "de-DE",
        "el-GR",
        "gu-IN",
        "ha-NG",
        "he-IL",
        "hi-IN",
        "hu-HU",
        "is-IS",
        "en-IN",
        "id-ID",
        "it-IT",
        "ja-JP",
        "jv-ID",
        "kea-CV",
        "kn-IN",
        "kk-KZ",
        "ko-KR",
        "ky-KG",
        "lv-LV",
        "ln-CD",
        "lt-LT",
        "mk-MK",
        "ms-MY",
        "ml-IN",
        "mt-MT",
        "cmn-Hans-CN",
        "mr-IN",
        "mn-MN",
        "ne-NP",
        "nb-NO",
        "or-IN",
        "pl-PL",
        "pt-BR",
        "pt-PT",
        "pa-IN",
        "pa-Guru-IN",
        "ro-RO",
        "ru-RU",
        "sr-RS",
        "sd-Arab-IN",
        "sk-SK",
        "sl-SI",
        "es-419",
        "es-US",
        "sw-KE",
        "sv-SE",
        "tg-TJ",
        "te-IN",
        "th-TH",
        "tr-TR",
        "uk-UA",
        "uz-UZ",
        "vi-VN",
    ];

    /// Only codes from Google's table are sent, each table row reachable from the app's
    /// spoken-language list is offered, and every language the mapping knows is one the list
    /// holds.
    #[test]
    fn spoken_languages_map_onto_googles_table() {
        let catalogue: Vec<serde_json::Value> =
            serde_json::from_str(include_str!("../../../src/lib/whisperLanguages.json")).unwrap();
        let spoken: Vec<&str> = catalogue
            .iter()
            .filter_map(|l| l["code"].as_str())
            .collect();
        let mut sent = std::collections::BTreeSet::new();
        for code in &spoken {
            for &gemini in language_codes(code).unwrap_or_default() {
                assert!(GOOGLE_TABLE.contains(&gemini), "{code} -> {gemini}");
                sent.insert(gemini);
            }
        }
        assert_eq!(
            spoken
                .iter()
                .filter(|c| language_codes(c).is_some())
                .count(),
            71
        );
        // The table's rows Whisper's list has no language for.
        let unreachable: Vec<_> = GOOGLE_TABLE
            .iter()
            .filter(|code| !sent.contains(*code))
            .collect();
        assert_eq!(
            unreachable,
            [
                &"rup-BG",
                &"yue-Hant-HK",
                &"ceb",
                &"kea-CV",
                &"ky-KG",
                &"or-IN"
            ]
        );
        assert!(language_codes("yo").is_none() && language_codes("nn").is_none());
        assert!(validate_spoken_language(Some("yo")).is_err());
        assert!(
            validate_spoken_language(Some("fr")).is_ok() && validate_spoken_language(None).is_ok()
        );
    }

    #[test]
    fn the_close_drain_ends_on_the_final_after_the_stream_end() {
        use crate::realtime::test_support::Harness;
        let mut h = Harness::new(GeminiTranscribeConfig {
            api_key: String::new(),
            model: DEFAULT_TRANSCRIBE_MODEL.to_string(),
            host: crate::gemini::DEFAULT_HOST.to_string(),
            language_codes: &[],
            custom_vocabulary: Vec::new(),
            origin: Origin::Microphone,
        });
        let interim =
            h.send(r#"{"serverContent":{"interimInputTranscription":{"text":"And fin"}}}"#);
        assert!(!h.proto.drain_complete(&interim));
        let last = h.send(r#"{"serverContent":{"inputTranscription":{"text":"And finally."}}}"#);
        assert!(h.proto.drain_complete(&last));
    }
}

/// Opt-in probe against the real endpoint. Ignored by default, so CI never runs it and it
/// never bills anyone by accident:
///
/// ```text
/// GEMINI_API_KEY=... cargo test -p live-translation --lib live_probe -- --ignored --nocapture
/// ```
///
/// Serialization tests pin this client to the *documented* shape. They cannot tell us the
/// documentation is right — and it demonstrably is not everywhere, since the live translate
/// guide still shows the transcription sidecars nested under `generationConfig`, a placement
/// the server rejects outright. This closes that gap by putting the production setup message,
/// audio frame and response types in front of Google's servers with the bundled rehearsal
/// recording as input. Re-run it before an event, as `docs/gemini-live-api.md` asks.
#[cfg(test)]
mod live_probe {
    use std::time::Duration;

    use base64::Engine;
    use futures_util::{SinkExt, StreamExt};
    use tokio_tungstenite::connect_async;
    use tokio_tungstenite::tungstenite::Message;

    use super::*;
    use crate::audio::fixture::load_fixture;
    use crate::audio::{chunk_samples, resample::f32_to_pcm16_le};
    use crate::gemini::DEFAULT_HOST;
    use crate::gemini::protocol::AUDIO_STREAM_END;

    /// The Live API replies in **binary** frames carrying UTF-8 JSON, not text frames —
    /// undocumented, and the reason this probe reads them the way `realtime/socket.rs` does
    /// rather than matching on `Message::Text`. A text-only reader sees the socket go silent.
    fn frame_json(message: &Message) -> Option<String> {
        match message {
            Message::Text(text) => Some(text.to_string()),
            Message::Binary(bytes) => String::from_utf8(bytes.to_vec()).ok(),
            _ => None,
        }
    }

    const FIXTURE: &str = "resources/fixtures/rehearsal-en.wav";
    const RATE: u32 = 16_000;
    /// The recording is about twenty seconds; the rest is head-room for the closing drain.
    const DEADLINE: Duration = Duration::from_secs(60);

    #[tokio::test]
    #[ignore = "streams the bundled fixture to Google and bills the key; run with --ignored"]
    async fn the_documented_shape_is_the_shape_the_server_accepts() {
        let config = GeminiTranscribeConfig {
            // Proves the server takes the field where the client puts it, not that it helps.
            custom_vocabulary: vec!["Gemini".to_string(), "Voxtral".to_string()],
            ..config(&[])
        };

        let samples = load_fixture(std::path::Path::new(FIXTURE))
            .expect("the bundled rehearsal fixture should load");
        let per_chunk = chunk_samples(RATE);
        println!(
            "probing {} with {:.1}s of bundled speech",
            config.model,
            samples.len() as f32 / RATE as f32
        );

        let (socket, _) = connect_async(config.connect_request().unwrap())
            .await
            .expect("the Live API should accept the connection");
        let (mut write, mut read) = socket.split();

        // Failure here is the one this probe exists for: a setup field in the wrong place
        // comes back as `Unknown name ... Cannot find field`, not as a bad transcript.
        write
            .send(Message::Text(config.setup_json().unwrap().unwrap().into()))
            .await
            .unwrap();
        let deadline = tokio::time::Instant::now() + DEADLINE;
        loop {
            let message = tokio::time::timeout_at(deadline, read.next())
                .await
                .expect("timed out waiting for setupComplete")
                .expect("the server closed before confirming setup")
                .unwrap();
            if let Message::Close(frame) = &message {
                panic!("the server rejected setup: {frame:?}");
            }
            if let Some(json) = frame_json(&message) {
                let parsed: ServerMessage = serde_json::from_str(&json)
                    .unwrap_or_else(|e| panic!("unparsed setup reply: {e} :: {json}"));
                assert!(
                    parsed.error.is_none(),
                    "the server rejected setup: {:?}",
                    parsed.error
                );
                if parsed.setup_complete.is_some() {
                    println!("setup accepted");
                    break;
                }
            }
        }

        // Pace the fixture at wall-clock speed. Dumping twenty seconds of audio at once is a
        // different thing from a live room, and this probe is only worth what its realism is.
        let sender = tokio::spawn(async move {
            let mut pcm = Vec::new();
            for chunk in samples.chunks(per_chunk) {
                pcm.clear();
                f32_to_pcm16_le(chunk, &mut pcm);
                let data = base64::engine::general_purpose::STANDARD.encode(&pcm);
                if write
                    .send(Message::Text(config.audio_json(data).unwrap().into()))
                    .await
                    .is_err()
                {
                    return write;
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            let _ = write.send(Message::Text(AUDIO_STREAM_END.into())).await;
            write
        });

        let mut interim = Vec::new();
        let mut finals: Vec<String> = Vec::new();
        let mut closes = 0usize;
        while let Ok(Some(Ok(message))) = tokio::time::timeout_at(deadline, read.next()).await {
            let Some(json) = frame_json(&message) else {
                continue;
            };
            let parsed: ServerMessage = serde_json::from_str(&json)
                .unwrap_or_else(|e| panic!("unparsed server message: {e} :: {json}"));
            assert!(parsed.error.is_none(), "provider error: {:?}", parsed.error);
            let Some(content) = parsed.server_content else {
                continue;
            };
            if let Some(t) = content.interim_input_transcription {
                println!("  interim: {}", t.text);
                interim.push(t.text);
            }
            if let Some(t) = content.input_transcription {
                println!("  FINAL:   {}", t.text);
                finals.push(t.text);
            }
            if content.generation_complete.unwrap_or(false) {
                println!("  [generationComplete]");
                closes += 1;
            }
            // The fixture is one short passage; one finalized segment is enough to prove the
            // field names, and waiting for the socket to close would only cost wall clock.
            if !finals.is_empty() && sender.is_finished() {
                break;
            }
        }

        assert!(
            !finals.iter().any(String::is_empty) && !finals.is_empty(),
            "expected at least one non-empty inputTranscription; interim seen: {}",
            interim.len()
        );
        assert!(
            closes > 0,
            "expected generationComplete to close a segment; the client relies on it to \
             finalize a segment that yields no transcript"
        );
        // Interim is a revision of the segment in progress, never an increment of it. If this
        // ever stops holding, the client's replace-don't-append rule is wrong.
        assert!(
            interim.len() > finals.len(),
            "expected many speculative updates per finalized segment, got {} and {}",
            interim.len(),
            finals.len()
        );
        println!(
            "ok — {} interim update(s), {} finalized segment(s), {} segment close(s)",
            interim.len(),
            finals.len(),
            closes
        );
    }

    /// The production client's configuration, with the key the app saved (or `GEMINI_API_KEY`),
    /// read in process and never printed.
    fn config(language_codes: &'static [&'static str]) -> GeminiTranscribeConfig {
        let _ = dotenvy::dotenv();
        GeminiTranscribeConfig {
            api_key: crate::secrets::resolve_api_key(crate::types::Provider::GeminiTranscribe)
                .expect("save a Gemini key in the app, or set GEMINI_API_KEY"),
            model: DEFAULT_TRANSCRIBE_MODEL.to_string(),
            host: DEFAULT_HOST.to_string(),
            language_codes,
            custom_vocabulary: Vec::new(),
            origin: Origin::Microphone,
        }
    }

    /// Send the production setup and wait for `setupComplete`. `Err` carries the server's
    /// error or close frame, which name the field it refused and nothing secret.
    async fn setup(
        config: &GeminiTranscribeConfig,
    ) -> Result<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
        String,
    > {
        let (mut socket, _) = connect_async(config.connect_request().unwrap())
            .await
            .expect("the Live API should accept the connection");
        socket
            .send(Message::Text(config.setup_json().unwrap().unwrap().into()))
            .await
            .unwrap();
        let deadline = tokio::time::Instant::now() + Duration::from_secs(20);
        loop {
            let message = match tokio::time::timeout_at(deadline, socket.next()).await {
                Err(_) => return Err("timed out waiting for setupComplete".into()),
                Ok(None) => return Err("closed before setupComplete".into()),
                Ok(Some(Err(error))) => return Err(error.to_string()),
                Ok(Some(Ok(message))) => message,
            };
            if let Message::Close(frame) = &message {
                return Err(format!("{frame:?}"));
            }
            if let Some(json) = frame_json(&message) {
                let parsed: ServerMessage = serde_json::from_str(&json)
                    .unwrap_or_else(|e| panic!("unparsed setup reply: {e} :: {json}"));
                if let Some(error) = parsed.error {
                    return Err(error.to_string());
                }
                if parsed.setup_complete.is_some() {
                    return Ok(socket);
                }
            }
        }
    }

    /// Every `languageCodes` list the client can send, set up and closed without a byte of
    /// audio — so nothing is transcribed — plus three codes it never sends, to show whether
    /// the server checks the field at all.
    #[tokio::test]
    #[ignore = "opens a Gemini session per spoken language without sending audio; run with --ignored"]
    async fn every_spoken_language_hint_is_accepted() {
        let catalogue: Vec<serde_json::Value> =
            serde_json::from_str(include_str!("../../../src/lib/whisperLanguages.json")).unwrap();
        let mut refused = Vec::new();
        for spoken in catalogue.iter().filter_map(|l| l["code"].as_str()) {
            let Some(codes) = language_codes(spoken) else {
                continue;
            };
            let verdict = setup(&config(codes)).await;
            println!(
                "{spoken} {codes:?}: {:?}",
                verdict.as_ref().map(|_| "accepted")
            );
            if let Ok(mut socket) = verdict {
                let _ = socket.close(None).await;
            } else {
                refused.push(spoken);
            }
        }
        for codes in [&["fr"][..], &["es-ES"], &["xx-XX"]] {
            let verdict = setup(&config(codes)).await;
            println!(
                "(not sent) {codes:?}: {:?}",
                verdict.as_ref().map(|_| "accepted")
            );
            if let Ok(mut socket) = verdict {
                let _ = socket.close(None).await;
            }
        }
        assert!(refused.is_empty(), "refused: {refused:?}");
    }

    /// Stream the French recording once per hint and print the finalized segments: the right
    /// hint, and English, to see what a wrong one does.
    #[tokio::test]
    #[ignore = "streams the bundled French fixture to Google twice and bills the key; run with --ignored"]
    async fn the_french_recording_under_a_right_and_a_wrong_hint() {
        let samples = load_fixture(std::path::Path::new("resources/fixtures/rehearsal-fr.wav"))
            .expect("the bundled rehearsal fixture should load");
        for spoken in ["fr", "en"] {
            let config = config(language_codes(spoken).unwrap());
            let socket = setup(&config)
                .await
                .unwrap_or_else(|e| panic!("{spoken}: setup refused: {e}"));
            println!(
                "=== hint {spoken} {:?}: setup accepted",
                config.language_codes
            );
            let (mut write, mut read) = socket.split();
            let audio = samples.clone();
            let frames = config.clone();
            let sender = tokio::spawn(async move {
                let mut pcm = Vec::new();
                for chunk in audio.chunks(chunk_samples(RATE)) {
                    pcm.clear();
                    f32_to_pcm16_le(chunk, &mut pcm);
                    let data = base64::engine::general_purpose::STANDARD.encode(&pcm);
                    let frame = frames.audio_json(data).unwrap();
                    if write.send(Message::Text(frame.into())).await.is_err() {
                        return write;
                    }
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
                let _ = write.send(Message::Text(AUDIO_STREAM_END.into())).await;
                write
            });
            let deadline = tokio::time::Instant::now() + DEADLINE;
            let mut finals = Vec::new();
            while let Ok(Some(Ok(message))) = tokio::time::timeout_at(deadline, read.next()).await {
                let Some(json) = frame_json(&message) else {
                    continue;
                };
                let parsed: ServerMessage = serde_json::from_str(&json)
                    .unwrap_or_else(|e| panic!("unparsed server message: {e} :: {json}"));
                assert!(parsed.error.is_none(), "provider error: {:?}", parsed.error);
                if let Some(text) = parsed.server_content.and_then(|c| c.input_transcription) {
                    println!("  FINAL: {}", text.text);
                    finals.push(text.text);
                }
                // The final after `audioStreamEnd` is the last one the recording yields.
                if sender.is_finished() && !finals.is_empty() {
                    let quiet = tokio::time::Instant::now() + Duration::from_secs(4);
                    while let Ok(Some(Ok(message))) =
                        tokio::time::timeout_at(quiet, read.next()).await
                    {
                        let Some(json) = frame_json(&message) else {
                            continue;
                        };
                        if let Ok(parsed) = serde_json::from_str::<ServerMessage>(&json)
                            && let Some(text) =
                                parsed.server_content.and_then(|c| c.input_transcription)
                        {
                            println!("  FINAL: {}", text.text);
                            finals.push(text.text);
                        }
                    }
                    break;
                }
            }
            drop(sender.await.unwrap());
            assert!(
                finals.iter().any(|t| !t.is_empty()),
                "{spoken}: no transcript"
            );
        }
    }
}
