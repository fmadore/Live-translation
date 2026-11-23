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

impl GeminiTranscribeConfig {
    pub fn into_client(self) -> GeminiTranscribeClient {
        GeminiTranscribeClient {
            config: self,
            document: String::new(),
        }
    }
}

/// The client: its configuration, and the transcript the server holds for this connection.
pub struct GeminiTranscribeClient {
    config: GeminiTranscribeConfig,
    /// In `SMART` mode each `inputTranscription` is not its own segment but "the complete
    /// updated replacement text for the session so far": every earlier turn again, cleaned
    /// up, then the new one. This is the last such text, so a final captions only what it
    /// adds. Cleared by the `setupComplete` that opens each connection, since a new session
    /// starts from an empty text.
    document: String,
}

impl RealtimeProtocol for GeminiTranscribeClient {
    const NAME: &'static str = "Gemini Transcribe";

    fn origin(&self) -> Origin {
        self.config.origin
    }

    fn connect_request(&self) -> Result<Request> {
        gemini_request(Self::NAME, &self.config.host, &self.config.api_key)
    }

    fn setup_json(&self) -> Result<Option<String>> {
        Ok(Some(serde_json::to_string(
            &TranscribeSetupMessage::live_transcribe(
                &self.config.model,
                self.config.language_codes,
                &self.config.custom_vocabulary,
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

        if msg.setup_complete.is_some() {
            self.document.clear();
        }
        if let Some(outcome) = msg.control(self, "Gemini Transcribe error") {
            return outcome;
        }

        let Some(content) = msg.server_content else {
            return MessageOutcome::default();
        };

        // Empty finalized captions retract speculative text in both windows and the pending
        // transcript. Never archive an all-filler interim as confirmed speech.
        match apply_transcription(&content, acc, &mut self.document) {
            Some(true) => MessageOutcome::activity(CaptionUpdate::Final),
            Some(false) => MessageOutcome::activity(CaptionUpdate::Interim),
            None => MessageOutcome::default(),
        }
    }

    fn finalize_after(&self) -> Option<Duration> {
        Some(FINALIZE_AFTER)
    }
}

/// Final text wins even when a frame also carries an interim hypothesis. A final is the
/// session's whole text so far (`document` holds the previous one), and the caption is what it
/// adds; an interim is the turn being spoken, as it is spoken.
fn apply_transcription(
    content: &super::protocol::ServerContent,
    acc: &mut TurnAccumulator,
    document: &mut String,
) -> Option<bool> {
    if let Some(final_text) = &content.input_transcription {
        acc.text = added_text(document, &final_text.text).to_owned();
        document.clone_from(&final_text.text);
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

/// What `document` adds to `shown`, the text the previous final carried. Normally it extends
/// it, and the new turn is the rest. When the model has also rewritten earlier text (it
/// cleans up across turns, and applies spoken edits such as "delete that" to the whole text)
/// the captions already shown cannot change, so this starts at the last word the two still
/// share. Nothing new, as when a turn only repeated earlier words, is an empty caption, which
/// retracts the turn's interim text.
fn added_text<'a>(shown: &str, document: &'a str) -> &'a str {
    if let Some(rest) = document.strip_prefix(shown) {
        return rest.trim_start();
    }
    let shared = shown
        .char_indices()
        .zip(document.chars())
        .take_while(|((_, a), b)| a == b)
        .last()
        .map_or(0, |((at, c), _)| at + c.len_utf8());
    let start = document[..shared]
        .char_indices()
        .rev()
        .find(|(_, c)| c.is_whitespace())
        .map_or(0, |(at, c)| at + c.len_utf8());
    document[start..].trim_start()
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
        assert_eq!(
            apply_transcription(&frame, &mut acc, &mut String::new()),
            Some(true)
        );
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
            assert_eq!(
                apply_transcription(&content(raw), &mut acc, &mut String::new()),
                Some(true)
            );
            assert!(acc.is_empty());
        }
    }

    #[test]
    fn interims_revise_the_segment_and_a_final_closes_it() {
        use crate::realtime::test_support::{Emitted, Harness};
        let mut h = Harness::new(
            GeminiTranscribeConfig {
                api_key: String::new(),
                model: DEFAULT_TRANSCRIBE_MODEL.to_string(),
                host: crate::gemini::DEFAULT_HOST.to_string(),
                language_codes: &[],
                custom_vocabulary: Vec::new(),
                origin: Origin::Microphone,
            }
            .into_client(),
        );
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
            handshake(config.clone().into_client().connect_request().unwrap()),
            handshake(translate.connect_request().unwrap())
        );

        // The operator reads this as the reconnecting status's detail.
        config.host = "not a host".to_string();
        let error = config.into_client().connect_request().unwrap_err();
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
            config("fr").into_client().setup_json().unwrap().unwrap(),
            r#"{"setup":{"model":"models/gemini-3.5-transcribe-live","generationConfig":{"responseModalities":["TEXT"]},"inputAudioTranscription":{"languageCodes":["fr-FR"],"mode":"SMART"}}}"#
        );
        let mut h = Harness::new(config("en").into_client());
        assert!(
            config("en")
                .into_client()
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
        let mut mapped: Vec<&str> = spoken
            .iter()
            .copied()
            .filter(|c| language_codes(c).is_some())
            .collect();
        assert_eq!(mapped.len(), 71);
        // The interface offers exactly these (`src/lib/geminiTranscribeLanguages.json`).
        let mut offered: Vec<String> = serde_json::from_str(include_str!(
            "../../../src/lib/geminiTranscribeLanguages.json"
        ))
        .unwrap();
        mapped.sort_unstable();
        offered.sort_unstable();
        assert_eq!(offered, mapped);
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

    fn client() -> GeminiTranscribeClient {
        GeminiTranscribeConfig {
            api_key: String::new(),
            model: DEFAULT_TRANSCRIBE_MODEL.to_string(),
            host: crate::gemini::DEFAULT_HOST.to_string(),
            language_codes: &[],
            custom_vocabulary: Vec::new(),
            origin: Origin::Microphone,
        }
        .into_client()
    }

    fn final_frame(text: &str) -> String {
        serde_json::json!({ "serverContent": { "inputTranscription": { "text": text } } })
            .to_string()
    }

    // The three passages of `live_probe::smart_finals_across_three_passages`, as the server
    // finalized them on 23 November 2026: French, then English in two turns.
    const FRENCH: &str = "Ceci est un enregistrement de répétition pour le système de sous-titre en direct. Une oratrice s'adresse à un public et chaque phrase est transcrite puis affichée à l'écran. Les sous-titres apparaissent juste après les mots, puis sont corrigés lorsque la phrase devient claire.\n\nUtilisez cet enregistrement pour régler l'affichage avant l'arrivée du public.";
    const ENGLISH: &str = "This is a rehearsal recording for the live caption system. A speaker is presenting to an audience, and every sentence is transcribed and shown on screen as it is spoken. Captions appear a moment after.";
    const ENGLISH_END: &str = "Use this recording to check the overlay before the room fills up.";

    /// The recorded SMART session. Every final repeated the whole session so far, and the
    /// fourth, after the French passage was spoken again, was the third unchanged: SMART
    /// cleaned the repetition away. Captioned as they came, through 1.15.0, the second
    /// caption repeated the first and the third repeated both. The finals, their order and
    /// the frames between them are as recorded; the interim text of the last turn is not.
    #[test]
    fn a_smart_final_captions_only_what_it_adds() {
        use crate::realtime::test_support::{Emitted, Harness};
        let finals = [
            FRENCH.to_string(),
            format!("{FRENCH} {ENGLISH}"),
            format!("{FRENCH} {ENGLISH} {ENGLISH_END}"),
            format!("{FRENCH} {ENGLISH} {ENGLISH_END}"),
        ];
        for pair in finals.windows(2) {
            assert!(pair[1].starts_with(&pair[0]), "the recording is cumulative");
        }
        let mut h = Harness::new(client());
        h.send(r#"{"setupComplete":{}}"#);
        for (turn, text) in finals.iter().enumerate() {
            h.send(r#"{"serverContent":{},"voiceActivity":{"type":"ACTIVITY_START"}}"#);
            if turn == 3 {
                h.send(
                    r#"{"serverContent":{"interimInputTranscription":{"text":"Ceci est un enregistrement"}}}"#,
                );
            }
            h.send(&final_frame(text));
            h.send(r#"{"serverContent":{"generationComplete":true}}"#);
            h.send(r#"{"serverContent":{},"voiceActivity":{"type":"ACTIVITY_END"}}"#);
        }
        assert_eq!(
            h.captions,
            [
                Emitted::final_(0, FRENCH, ""),
                Emitted::final_(1, ENGLISH, ""),
                Emitted::final_(2, ENGLISH_END, ""),
                Emitted::interim(3, "Ceci est un enregistrement", ""),
                // Nothing new: the repeated passage's interim text is retracted.
                Emitted::final_(3, "", ""),
            ]
        );
    }

    /// A new connection is a new session with an empty text, so its first final is all new.
    #[test]
    fn each_connection_starts_from_an_empty_text() {
        use crate::realtime::test_support::{Emitted, Harness};
        let mut h = Harness::new(client());
        h.send(r#"{"setupComplete":{}}"#);
        h.send(&final_frame(ENGLISH));
        // The ten-minute handover: the next socket's session knows nothing of the last.
        h.send(r#"{"setupComplete":{}}"#);
        h.send(&final_frame(ENGLISH_END));
        assert_eq!(
            h.captions,
            [
                Emitted::final_(0, ENGLISH, ""),
                Emitted::final_(1, ENGLISH_END, ""),
            ]
        );
    }

    /// When SMART rewrites earlier text, the shown captions stay as they are and the next
    /// one starts at the first word that changed.
    #[test]
    fn a_rewrite_of_earlier_text_resumes_at_the_first_changed_word() {
        assert_eq!(
            added_text(
                "We meet at two. See you",
                "We meet at three. See you there."
            ),
            "three. See you there."
        );
        assert_eq!(added_text("Hello.", "Hello. Bonjour."), "Bonjour.");
        assert_eq!(added_text("Hello.", "Hello."), "");
        assert_eq!(added_text("Delete this.", ""), "");
        assert_eq!(added_text("", "Première phrase."), "Première phrase.");
        // A change in the very first word leaves nothing shared.
        assert_eq!(added_text("Écoute.", "Écoutez bien."), "Écoutez bien.");
    }

    #[test]
    fn the_close_drain_ends_on_the_final_after_the_stream_end() {
        use crate::realtime::test_support::Harness;
        let mut h = Harness::new(
            GeminiTranscribeConfig {
                api_key: String::new(),
                model: DEFAULT_TRANSCRIBE_MODEL.to_string(),
                host: crate::gemini::DEFAULT_HOST.to_string(),
                language_codes: &[],
                custom_vocabulary: Vec::new(),
                origin: Origin::Microphone,
            }
            .into_client(),
        );
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
        let config = config.into_client();

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

    type Socket = tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >;

    /// Send the production setup and wait for `setupComplete`. `Err` carries the server's
    /// error or close frame, which name the field it refused and nothing secret.
    async fn setup(config: &GeminiTranscribeConfig) -> Result<Socket, String> {
        setup_in_mode(config, None).await
    }

    /// [`setup`], with the transcription `mode` replaced when one is given.
    async fn setup_in_mode(
        config: &GeminiTranscribeConfig,
        mode: Option<&str>,
    ) -> Result<Socket, String> {
        let client = config.clone().into_client();
        let mut setup: serde_json::Value =
            serde_json::from_str(&client.setup_json().unwrap().unwrap()).unwrap();
        if let Some(mode) = mode {
            setup["setup"]["inputAudioTranscription"]["mode"] = mode.into();
        }
        let (mut socket, _) = connect_async(client.connect_request().unwrap())
            .await
            .expect("the Live API should accept the connection");
        socket
            .send(Message::Text(setup.to_string().into()))
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
            let frames = config.clone().into_client();
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

    /// Three passages with six seconds of silence between them — French, English, French —
    /// under the production setup, printing every frame but the interim ones in arrival
    /// order. On 23 November 2026 it showed that each SMART final is "the complete updated
    /// replacement text for the session so far", as the guide says, not its own turn: the
    /// recording behind `smart_tests::a_smart_final_captions_only_what_it_adds`.
    #[tokio::test]
    #[ignore = "streams about eighty seconds of the bundled fixtures to Google and bills the key; run with --ignored"]
    async fn smart_finals_across_three_passages() {
        three_passages(None).await;
    }

    /// The same stream in `VERBATIM` mode, Google's default, for comparison.
    #[tokio::test]
    #[ignore = "streams about eighty seconds of the bundled fixtures to Google and bills the key; run with --ignored"]
    async fn verbatim_finals_across_three_passages() {
        three_passages(Some("VERBATIM")).await;
    }

    async fn three_passages(mode: Option<&'static str>) {
        let french = load_fixture(std::path::Path::new("resources/fixtures/rehearsal-fr.wav"))
            .expect("the bundled rehearsal fixture should load");
        let english = load_fixture(std::path::Path::new(FIXTURE))
            .expect("the bundled rehearsal fixture should load");
        let silence = vec![0.0_f32; RATE as usize * 6];
        let stream: Vec<f32> = [&french[..], &silence, &english, &silence, &french].concat();
        println!(
            "streaming {:.1}s: French, 6 s silence, English, 6 s silence, French",
            stream.len() as f32 / RATE as f32
        );

        let config = config(&[]);
        let socket = setup_in_mode(&config, mode)
            .await
            .unwrap_or_else(|e| panic!("setup refused: {e}"));
        let (mut write, mut read) = socket.split();
        let frames = config.clone().into_client();
        let started = tokio::time::Instant::now();
        let sender = tokio::spawn(async move {
            let mut pcm = Vec::new();
            for chunk in stream.chunks(chunk_samples(RATE)) {
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

        let deadline = started + Duration::from_secs(150);
        let mut interim = 0usize;
        let mut last_interim = String::new();
        let mut finals: Vec<String> = Vec::new();
        let mut drained_at = None;
        while let Ok(Some(Ok(message))) = tokio::time::timeout_at(deadline, read.next()).await {
            let Some(json) = frame_json(&message) else {
                continue;
            };
            let parsed: ServerMessage = serde_json::from_str(&json)
                .unwrap_or_else(|e| panic!("unparsed server message: {e} :: {json}"));
            assert!(parsed.error.is_none(), "provider error: {:?}", parsed.error);
            let Some(content) = parsed.server_content else {
                println!("{:>6.1}s {json}", started.elapsed().as_secs_f32());
                continue;
            };
            if let Some(text) = &content.interim_input_transcription
                && content.input_transcription.is_none()
            {
                interim += 1;
                last_interim.clone_from(&text.text);
                continue;
            }
            if interim > 0 {
                println!("        ({interim} interim, the last: {last_interim:?})");
                interim = 0;
            }
            println!("{:>6.1}s {json}", started.elapsed().as_secs_f32());
            if let Some(text) = content.input_transcription {
                finals.push(text.text);
            }
            // Once the audio is all sent, give the server ten seconds to flush and stop.
            if sender.is_finished() {
                let at = *drained_at.get_or_insert_with(tokio::time::Instant::now);
                if at.elapsed() > Duration::from_secs(10) {
                    break;
                }
            }
        }
        drop(sender.await.unwrap());
        println!("{} finalized segment(s)", finals.len());
        for (i, text) in finals.iter().enumerate() {
            println!("--- final {i} ({} chars)\n{text}", text.chars().count());
        }
        assert!(finals.len() > 1, "expected more than one final");
    }
}
