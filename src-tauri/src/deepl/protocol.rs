//! Wire types for the DeepL Voice API: the REST session request, the WebSocket's messages, and
//! the target-language codes it takes. Shapes follow DeepL's OpenAPI and AsyncAPI documents as
//! read on 11 November 2026; see `docs/deepl-voice-api.md`.

use serde::{Deserialize, Serialize};

use crate::types::TargetLanguage;

/// The only input format this app sends: what every capture path produces for a 16 kHz
/// provider, mono PCM16 little-endian.
pub const PCM_16K: &str = "audio/pcm;encoding=s16le;rate=16000";

/// `POST /v3/voice/realtime`. The session's options are fixed here, over REST, which is why the
/// socket opens with no setup message.
#[derive(Debug, Serialize)]
pub struct SessionRequest<'a> {
    pub source_media_content_type: &'static str,
    pub message_format: &'static str,
    /// `auto`, with no `source_language`: DeepL detects what is spoken, as the operator picks
    /// no spoken language for a translation engine.
    pub source_language_mode: &'static str,
    /// One code: one session per caption language, like every other engine's lanes. DeepL
    /// takes up to five, so a later version could share one session between both lanes.
    pub target_languages: [&'a str; 1],
}

impl<'a> SessionRequest<'a> {
    pub fn translate(target: &'a str) -> Self {
        Self {
            source_media_content_type: PCM_16K,
            message_format: "json",
            source_language_mode: "auto",
            target_languages: [target],
        }
    }
}

/// The single-use streaming URL and token a session request, or a reconnect, answers with.
/// No `Debug`: the token opens a socket on the operator's account, and does not belong in a log.
#[derive(Deserialize)]
pub struct Ticket {
    pub streaming_url: String,
    pub token: String,
}

/// DeepL's error body over REST: `{ message, code? }`, or `{ error: { message } }` from its
/// edge before the request reached the API. The message, or nothing.
pub fn rest_error_message(body: &str) -> String {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(body) else {
        return String::new();
    };
    value
        .get("message")
        .or_else(|| value.get("error")?.get("message"))
        .and_then(serde_json::Value::as_str)
        .map(|message| message.trim().to_string())
        .unwrap_or_default()
}

/// One audio frame: base64 of 16 kHz mono PCM16. Base64's alphabet needs no JSON escaping.
pub fn source_media_chunk(base64_pcm: &str) -> String {
    format!(r#"{{"source_media_chunk":{{"data":"{base64_pcm}"}}}}"#)
}

/// No more audio: DeepL concludes every tentative segment, then ends the stream.
pub const END_OF_SOURCE_MEDIA: &str = r#"{"end_of_source_media":{}}"#;

/// A server message: one top-level key names it. Those this client does not act on
/// (`end_of_source_transcript`, `end_of_target_transcript`, and the speech messages it never
/// asks for) parse to all-`None` and are ignored.
#[derive(Debug, Default, Deserialize)]
pub struct ServerMessage {
    #[serde(default)]
    pub source_transcript_update: Option<TranscriptUpdate>,
    #[serde(default)]
    pub target_transcript_update: Option<TranscriptUpdate>,
    #[serde(default)]
    pub end_of_stream: Option<serde_json::Value>,
    #[serde(default)]
    pub error: Option<StreamError>,
}

/// The source or a target transcript's update. Concluded segments are final and arrive once;
/// the tentative ones replace the tentative ones before them.
#[derive(Debug, Default, Deserialize)]
pub struct TranscriptUpdate {
    #[serde(default)]
    pub concluded: Vec<Segment>,
    #[serde(default)]
    pub tentative: Vec<Segment>,
}

#[derive(Debug, Default, Deserialize)]
pub struct Segment {
    #[serde(default)]
    pub text: String,
    /// Milliseconds into the audio, as DeepL documents them: integers, read as numbers of any
    /// kind so a fractional one cannot make the whole message unreadable.
    #[serde(default)]
    pub end_time: f64,
}

/// DeepL's `error` message, which ends the session.
#[derive(Debug, Default, Deserialize)]
pub struct StreamError {
    #[serde(default)]
    pub request_type: Option<String>,
    #[serde(default)]
    pub error_code: Option<u64>,
    #[serde(default)]
    pub reason_code: Option<u64>,
    #[serde(default)]
    pub error_message: Option<String>,
}

impl StreamError {
    /// For the operator: "Audio format not supported (error 400, reason 4000403)".
    pub fn describe(&self) -> String {
        let message = self
            .error_message
            .as_deref()
            .filter(|message| !message.trim().is_empty())
            .unwrap_or("Voice API error");
        let codes: Vec<String> = [
            self.error_code.map(|code| format!("error {code}")),
            self.reason_code.map(|code| format!("reason {code}")),
        ]
        .into_iter()
        .flatten()
        .collect();
        if codes.is_empty() {
            message.to_string()
        } else {
            format!("{message} ({})", codes.join(", "))
        }
    }
}

/// The code DeepL's `target_languages` takes for a catalog language, spelled as its OpenAPI
/// enum spells it. `None` for a language it does not translate into (`supported_by` says the
/// same, and start validation stops such a session first).
///
/// Where the catalog and DeepL differ: English is sent as American English (`en-US`; DeepL
/// distinguishes it from `en-GB` and the catalog has one English), Norwegian as Bokmål (`nb`,
/// DeepL's only Norwegian), and the Chinese scripts as `zh-HANS` / `zh-HANT`, the enum's own
/// casing. These four follow the documented enum and have not been through the live probe.
pub fn deepl_target_code(language: TargetLanguage) -> Option<&'static str> {
    use TargetLanguage as L;
    Some(match language {
        L::Ar => "ar",
        L::Bn => "bn",
        L::Bg => "bg",
        L::Ca => "ca",
        L::ZhHans => "zh-HANS",
        L::ZhHant => "zh-HANT",
        L::Hr => "hr",
        L::Cs => "cs",
        L::Da => "da",
        L::Nl => "nl",
        L::En => "en-US",
        L::Et => "et",
        L::Fi => "fi",
        L::Fr => "fr",
        L::Gl => "gl",
        L::De => "de",
        L::El => "el",
        L::He => "he",
        L::Hi => "hi",
        L::Hu => "hu",
        L::Id => "id",
        L::It => "it",
        L::Ja => "ja",
        L::Ko => "ko",
        L::Lv => "lv",
        L::Lt => "lt",
        L::Ms => "ms",
        L::No => "nb",
        L::Pl => "pl",
        L::PtBr => "pt-BR",
        L::PtPt => "pt-PT",
        L::Ro => "ro",
        L::Ru => "ru",
        L::Sk => "sk",
        L::Sl => "sl",
        L::Es => "es",
        L::Sv => "sv",
        L::Ta => "ta",
        L::Th => "th",
        L::Tr => "tr",
        L::Uk => "uk",
        L::Vi => "vi",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Provider;

    /// `target_languages` as DeepL's OpenAPI document lists it on 11 November 2026: the base
    /// codes, then the regional variants.
    const DOCUMENTED_TARGETS: [&str; 52] = [
        "ar", "bg", "bn", "ca", "cs", "da", "de", "el", "en", "es", "et", "fi", "fr", "ga", "gl",
        "he", "hi", "hr", "hu", "id", "it", "ja", "ko", "lt", "lv", "ms", "mt", "nb", "nl", "pl",
        "pt", "ro", "ru", "sk", "sl", "sv", "ta", "th", "tl", "tr", "uk", "vi", "zh", "en-GB",
        "en-US", "es-419", "fr-CA", "fr-FR", "pt-BR", "pt-PT", "zh-HANS", "zh-HANT",
    ];

    #[test]
    fn every_catalog_language_deepl_serves_has_a_documented_code_and_no_other_does() {
        let catalog: serde_json::Value =
            serde_json::from_str(include_str!("../../../src/lib/languages.json")).unwrap();
        let mut served = 0;
        for row in catalog["languages"].as_array().unwrap() {
            let language: TargetLanguage = serde_json::from_value(row["code"].clone()).unwrap();
            let code = deepl_target_code(language);
            assert_eq!(
                code.is_some(),
                language.supported_by(Provider::DeepL),
                "{language:?}"
            );
            if let Some(code) = code {
                served += 1;
                assert!(DOCUMENTED_TARGETS.contains(&code), "{language:?} -> {code}");
            }
        }
        assert_eq!(served, 42);
        assert_eq!(deepl_target_code(TargetLanguage::En), Some("en-US"));
        assert_eq!(deepl_target_code(TargetLanguage::No), Some("nb"));
        assert_eq!(deepl_target_code(TargetLanguage::ZhHant), Some("zh-HANT"));
    }

    #[test]
    fn the_session_request_is_pcm_at_16_khz_into_one_auto_detected_translation() {
        assert_eq!(
            serde_json::to_string(&SessionRequest::translate("de")).unwrap(),
            r#"{"source_media_content_type":"audio/pcm;encoding=s16le;rate=16000","message_format":"json","source_language_mode":"auto","target_languages":["de"]}"#
        );
    }

    #[test]
    fn client_frames_match_the_asyncapi_document() {
        assert_eq!(
            source_media_chunk("AAA="),
            r#"{"source_media_chunk":{"data":"AAA="}}"#
        );
        let value: serde_json::Value = serde_json::from_str(END_OF_SOURCE_MEDIA).unwrap();
        assert_eq!(value, serde_json::json!({"end_of_source_media": {}}));
    }

    #[test]
    fn rest_errors_are_read_in_both_documented_shapes() {
        assert_eq!(
            rest_error_message(r#"{"message":"Quota exceeded","code":"quota"}"#),
            "Quota exceeded"
        );
        assert_eq!(
            rest_error_message(r#"{"error":{"message":"Bad gateway"}}"#),
            "Bad gateway"
        );
        assert_eq!(rest_error_message("<html>busy</html>"), "");
        assert_eq!(rest_error_message(""), "");
    }

    /// The documented examples, verbatim.
    #[test]
    fn server_messages_parse_by_their_one_key() {
        let source: ServerMessage = serde_json::from_str(
            r#"{"source_transcript_update":{"concluded":[{"language":"en","text":"Hello, how are you","start_time":0,"end_time":1500}],"tentative":[{"language":"en","text":" today?","start_time":1500,"end_time":2000}]}}"#,
        )
        .unwrap();
        let update = source.source_transcript_update.unwrap();
        assert_eq!(update.concluded[0].text, "Hello, how are you");
        assert!((update.tentative[0].end_time - 2000.0).abs() < f64::EPSILON);

        let target: ServerMessage = serde_json::from_str(
            r#"{"target_transcript_update":{"language":"es","concluded":[{"text":"Hola, ¿cómo estás","start_time":0,"end_time":1500}],"tentative":[{"text":" hoy?","start_time":1500,"end_time":2000}]}}"#,
        )
        .unwrap();
        let update = target.target_transcript_update.unwrap();
        assert_eq!(update.concluded[0].text, "Hola, ¿cómo estás");
        assert_eq!(update.tentative[0].text, " hoy?");

        let end: ServerMessage = serde_json::from_str(r#"{"end_of_stream":{}}"#).unwrap();
        assert!(end.end_of_stream.is_some());
        for ignored in [
            r#"{"end_of_source_transcript":{}}"#,
            r#"{"end_of_target_transcript":{"language":"fr"}}"#,
            r#"{"target_media_chunk":{"language":"fr","data":["AAA="],"duration":20}}"#,
        ] {
            let message: ServerMessage = serde_json::from_str(ignored).unwrap();
            assert!(
                message.source_transcript_update.is_none()
                    && message.target_transcript_update.is_none()
                    && message.end_of_stream.is_none()
                    && message.error.is_none(),
                "{ignored}"
            );
        }

        let error: ServerMessage = serde_json::from_str(
            r#"{"error":{"request_type":"source_media_chunk","error_code":400,"reason_code":4000403,"error_message":"Audio format not supported"}}"#,
        )
        .unwrap();
        let error = error.error.unwrap();
        assert_eq!(error.request_type.as_deref(), Some("source_media_chunk"));
        assert_eq!(
            error.describe(),
            "Audio format not supported (error 400, reason 4000403)"
        );
        assert_eq!(StreamError::default().describe(), "Voice API error");
    }
}
