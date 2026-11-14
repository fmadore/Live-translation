//! What the providers' protocols share on the wire, so each provider module only spells out
//! what is its own.

use anyhow::{Context, Result};
use serde::de::DeserializeOwned;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::handshake::client::Request;
use tokio_tungstenite::tungstenite::http::{HeaderValue, header::AUTHORIZATION};
use tokio_tungstenite::tungstenite::protocol::CloseFrame;

use super::{MessageControl, MessageOutcome, RealtimeProtocol};

/// A WebSocket handshake authenticated with an `Authorization: Bearer` header, as OpenAI and
/// Mistral both require. `provider` names it in the errors, which the operator reads as the
/// detail of a reconnecting status.
pub fn bearer_request(provider: &str, url: &str, api_key: &str) -> Result<Request> {
    let mut request = url
        .into_client_request()
        .with_context(|| format!("failed to build {provider} request"))?;
    let mut bearer = HeaderValue::from_str(&format!("Bearer {api_key}"))
        .with_context(|| format!("{provider} API key is not a valid header value"))?;
    // So a `Debug` of the request or its headers prints `Sensitive`, never the key.
    bearer.set_sensitive(true);
    request.headers_mut().insert(AUTHORIZATION, bearer);
    Ok(request)
}

/// One provider frame, parsed — or logged and skipped. A frame that does not parse is not worth
/// ending a live session over: the providers add and reshape events without notice, and every
/// field a client reads is optional for the same reason.
pub fn parse_or_log<T: DeserializeOwned>(provider: &str, text: &str) -> Option<T> {
    match serde_json::from_str(text) {
        Ok(message) => Some(message),
        Err(error) => {
            tracing::debug!("unparsed {provider} message: {error} :: {text}");
            None
        }
    }
}

/// Whether the text ends a sentence, closing quotes and brackets aside, and the (often
/// non-breaking) space French sets before them. Mistral holds a line open until one ends;
/// DeepL closes a line when its concluded translation reaches one.
pub fn ends_sentence(text: &str) -> bool {
    text.trim_end_matches(|c: char| {
        c.is_whitespace() || matches!(c, '"' | '\'' | '”' | '’' | '»' | ')' | ']')
    })
    .ends_with(['.', '?', '!', '…', '。', '？', '！', '؟', '।'])
}

/// What a provider's failure means for the source. Every provider's `error` event, and a close
/// during setup, is read into one of these by `classify`, so the providers cannot drift apart
/// on what is worth a reconnect.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// A planned end of a healthy session, as a session cap is: move to a new one.
    Handover,
    /// A failure on the provider's side that a new connection can get past: retry with backoff.
    Retry,
    /// Nothing a reconnect fixes: a bad key, an exhausted quota, a request it will not take.
    Fatal,
}

impl Verdict {
    /// As a message's control. `fatal` is the operator's detail, built only when it is needed.
    pub fn control(self, fatal: impl FnOnce() -> String) -> MessageControl {
        match self {
            Self::Handover => MessageControl::Handover { time_left: None },
            Self::Retry => MessageControl::Reconnect,
            Self::Fatal => MessageControl::Fatal(fatal()),
        }
    }
}

/// Wording that no reconnect fixes, whatever code it comes under: a 429 is also how an exhausted
/// quota or plan limit is reported, and a "try again" close can carry a rejected key.
const FATAL_WORDING: [&str; 9] = [
    "quota",
    "billing",
    "payment",
    "credit",
    "api key",
    "api_key",
    "unauthenticated",
    "unauthorized",
    "permission denied",
];

/// Wording that marks a failure inside a provider's backend rather than in the request: gRPC's
/// UNAVAILABLE and DEADLINE_EXCEEDED, and a connection reset between its services. Seen live
/// from Mistral on 7 October 2026 as code 3803, "gRPC connection error: <AioRpcError …
/// StatusCode.UNAVAILABLE … Connection reset by peer …>"; a new session reaches a backend that
/// is up. And on 11 November 2026, during a live stream, as code 3804, "Timeout waiting for
/// response from streaming transcription.": its recogniser stalled, and a new session gets
/// another. Specific on purpose: a bare "unavailable" could also describe a model the key cannot
/// use, and a bare "timeout" a limit the request broke.
const TRANSIENT_WORDING: [&str; 6] = [
    "statuscode.unavailable",
    "grpc_status:14",
    "statuscode.deadline_exceeded",
    "grpc connection error",
    "connection reset by peer",
    "timeout waiting for response",
];

/// Error names that a new connection can get past: OpenAI's and Mistral's `type`/`code` names,
/// and the transient statuses of Google's `google.rpc.Code`, compared in lower case.
const TRANSIENT_NAMES: [&str; 11] = [
    "server_error",
    "rate_limit_error",
    "rate_limit_exceeded",
    "rate_limited",
    "overloaded",
    "overloaded_error",
    "service_unavailable",
    "timeout",
    "unavailable",
    "internal",
    "deadline_exceeded",
];

/// Classify one provider failure. `code` is numeric: an HTTP status, a WebSocket close code
/// (1000–4999, which no HTTP status shares), or a provider's own internal code, which matches
/// neither list below. `names` are its symbolic names (`type`, `code`, `status`), `message` its
/// wording. In order:
///
/// - `session_expired` is a session cap — OpenAI's 60 minutes — so it hands over.
/// - Quota, billing and authentication wording is fatal, whatever the code says.
/// - A provider backend failing to reach its own services is retried, whatever the code says.
/// - Timeouts, rate limits and server errors, as HTTP statuses, close codes 1001 (going away),
///   1011 (internal error), 1012 (restart) and 1013 (try again later), or transient names, are
///   retried.
/// - Everything else is fatal — close codes 1007 and 1008 (a payload or policy the server will
///   not take), a bad request, an unknown code — so a persistent failure reports itself
///   instead of reconnecting in a loop.
pub fn classify(code: Option<u64>, names: &[&str], message: &str) -> Verdict {
    let names: Vec<String> = names.iter().map(|name| name.to_lowercase()).collect();
    let message = message.to_lowercase();
    let says = |words: &[&str]| {
        words
            .iter()
            .any(|word| message.contains(word) || names.iter().any(|name| name.contains(word)))
    };
    if names.iter().any(|name| name == "session_expired") {
        Verdict::Handover
    } else if says(&FATAL_WORDING) {
        Verdict::Fatal
    } else if TRANSIENT_WORDING.iter().any(|word| message.contains(word))
        || matches!(
            code,
            Some(408 | 429 | 500 | 502 | 503 | 504 | 1001 | 1011 | 1012 | 1013)
        )
        || names
            .iter()
            .any(|name| TRANSIENT_NAMES.contains(&name.as_str()))
    {
        Verdict::Retry
    } else {
        Verdict::Fatal
    }
}

/// A provider's `error` event as every client acts on it: read by `classify_json`; when it ends
/// the source, worded `"{label}: {error}"` (the error alone without a label) for the operator,
/// who reads it after the provider's name; otherwise logged, since the operator sees only the
/// Reconnecting or the handover it causes.
pub fn provider_error<P: RealtimeProtocol>(
    proto: &P,
    label: Option<&str>,
    error: &serde_json::Value,
) -> MessageOutcome {
    let control = classify_json(error).control(|| match label {
        Some(label) => format!("{label}: {error}"),
        None => error.to_string(),
    });
    if !matches!(control, MessageControl::Fatal(_)) {
        let origin = proto.origin();
        tracing::warn!(?origin, %error, "{} reported an error a new session can get past", P::NAME);
    }
    MessageOutcome::control(control)
}

/// A WebSocket Close frame from the provider, read the same way during setup and mid-stream.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Close {
    /// `None` for a Close with no status at all (1005 on the wire).
    pub code: Option<u16>,
    pub reason: String,
}

impl Close {
    pub fn of(frame: Option<&CloseFrame>) -> Self {
        Self {
            code: frame.map(|frame| u16::from(frame.code)),
            reason: frame.map_or_else(String::new, |frame| frame.reason.trim().to_string()),
        }
    }

    /// Only a close that says the server will not take what it was sent ends the source: 1007
    /// (a payload it cannot read), 1008 (a policy it will not waive), or the key, quota or
    /// billing wording `classify` treats as fatal under any code. Every other close, a Normal
    /// Closure (1000) and codes nobody documents included, is a connection ending: retried,
    /// with backoff, and during setup within the cap on failures in a row, so one that keeps
    /// coming still reports itself.
    pub fn verdict(&self) -> Verdict {
        let wording = self.reason.to_lowercase();
        if matches!(self.code, Some(1007 | 1008))
            || FATAL_WORDING.iter().any(|word| wording.contains(word))
        {
            Verdict::Fatal
        } else {
            Verdict::Retry
        }
    }

    /// The code and reason, for the operator's detail and the log: "close code 1011: Internal
    /// error encountered." Empty for a Close that carried neither.
    pub fn describe(&self) -> String {
        match (self.code, self.reason.as_str()) {
            (None, "") => String::new(),
            (None, reason) => reason.to_string(),
            (Some(code), "") => format!("close code {code}"),
            (Some(code), reason) => format!("close code {code}: {reason}"),
        }
    }
}

/// `classify` for an `error` object as the providers send one: OpenAI's `{ type, code,
/// message }`, Mistral's `{ message, code }` with an integer code and a message that may itself
/// be an object, Google's `{ code, message, status }`. A bare string is read as the message.
pub fn classify_json(error: &serde_json::Value) -> Verdict {
    let code = error
        .get("code")
        .and_then(|code| code.as_u64().or_else(|| code.as_str()?.parse().ok()));
    let message = error.get("message");
    let names: Vec<&str> = [error, message.unwrap_or(&serde_json::Value::Null)]
        .into_iter()
        .flat_map(|value| ["type", "code", "status"].map(|key| value.get(key)))
        .filter_map(|name| name?.as_str())
        .collect();
    let message = match message.or(error.as_str().map(|_| error)) {
        Some(serde_json::Value::String(text)) => text.clone(),
        Some(other) => other.to_string(),
        None => String::new(),
    };
    classify(code, &names, &message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn close_codes_retry_what_the_server_says_to_retry_and_stop_on_the_rest() {
        for (code, reason, verdict) in [
            (1001, "", Verdict::Retry),
            (1011, "Internal error encountered.", Verdict::Retry),
            (1012, "", Verdict::Retry),
            (1013, "Try again later", Verdict::Retry),
            (1007, "Invalid JSON payload received.", Verdict::Fatal),
            (1008, "Requested entity was not found.", Verdict::Fatal),
            (1000, "", Verdict::Fatal),
            (4000, "", Verdict::Fatal),
            // Wording outranks a retryable code: retrying a rejected key or a spent quota
            // only repeats the rejection.
            (
                1011,
                "API key not valid. Please pass a valid API key.",
                Verdict::Fatal,
            ),
            (1013, "You exceeded your current quota", Verdict::Fatal),
        ] {
            assert_eq!(
                classify(Some(code), &[], reason),
                verdict,
                "{code} {reason}"
            );
        }
    }

    fn close(code: Option<u16>, reason: &str) -> Close {
        Close {
            code,
            reason: reason.into(),
        }
    }

    #[test]
    fn a_close_ends_the_source_only_when_it_refuses_what_it_was_sent() {
        for (code, reason, verdict) in [
            (Some(1007), "Invalid JSON payload received.", Verdict::Fatal),
            (
                Some(1008),
                "Requested entity was not found.",
                Verdict::Fatal,
            ),
            (Some(1008), "", Verdict::Fatal),
            (Some(1011), "API key not valid.", Verdict::Fatal),
            (
                Some(1000),
                "You exceeded your current quota",
                Verdict::Fatal,
            ),
            (None, "", Verdict::Retry),
            (Some(1000), "", Verdict::Retry),
            (Some(1000), "session ended", Verdict::Retry),
            (Some(1001), "", Verdict::Retry),
            (Some(1011), "Internal error encountered.", Verdict::Retry),
            (Some(1013), "Try again later", Verdict::Retry),
            (Some(4000), "", Verdict::Retry),
        ] {
            assert_eq!(close(code, reason).verdict(), verdict, "{code:?} {reason}");
        }
    }

    #[test]
    fn a_close_is_described_by_its_code_and_reason() {
        assert_eq!(close(None, "").describe(), "");
        assert_eq!(close(Some(1000), "").describe(), "close code 1000");
        assert_eq!(
            close(Some(1011), "Internal error.").describe(),
            "close code 1011: Internal error."
        );
        let frame = CloseFrame {
            code: 1013.into(),
            reason: "  Try again later \n".into(),
        };
        assert_eq!(
            Close::of(Some(&frame)),
            close(Some(1013), "Try again later")
        );
        assert_eq!(Close::of(None), close(None, ""));
    }

    #[test]
    fn the_bearer_key_is_never_printed_by_debug() {
        let request = bearer_request("Test", "wss://example.invalid/", "sk-secret").unwrap();
        let header = &request.headers()[AUTHORIZATION];
        assert!(header.is_sensitive());
        assert_eq!(header.to_str().unwrap(), "Bearer sk-secret");
        assert!(!format!("{request:?}").contains("sk-secret"));
    }

    #[test]
    fn error_objects_from_each_provider_are_read_the_same_way() {
        for (error, verdict) in [
            // OpenAI.
            (
                r#"{"type":"invalid_request_error","code":"session_expired","message":"Your session hit the maximum duration of 60 minutes."}"#,
                Verdict::Handover,
            ),
            (
                r#"{"type":"server_error","message":"The server had an error"}"#,
                Verdict::Retry,
            ),
            (
                r#"{"type":"rate_limit_error","code":"rate_limit_exceeded"}"#,
                Verdict::Retry,
            ),
            (
                r#"{"type":"rate_limit_error","code":"insufficient_quota"}"#,
                Verdict::Fatal,
            ),
            (
                r#"{"type":"invalid_request_error","code":"invalid_api_key"}"#,
                Verdict::Fatal,
            ),
            (r#"{"type":"something_new"}"#, Verdict::Fatal),
            (r#""a bare string""#, Verdict::Fatal),
            (
                r#""server overloaded, StatusCode.UNAVAILABLE""#,
                Verdict::Retry,
            ),
            // Mistral.
            (
                r#"{"message":"Service unavailable","code":503}"#,
                Verdict::Retry,
            ),
            (
                r#"{"message":"Rate limit exceeded","code":"429"}"#,
                Verdict::Retry,
            ),
            (
                r#"{"message":"Monthly quota exceeded","code":429}"#,
                Verdict::Fatal,
            ),
            (
                r#"{"message":{"type":"server_error","detail":"retry"},"code":3000}"#,
                Verdict::Retry,
            ),
            (
                r#"{"code":3803,"message":"gRPC connection error: StatusCode.UNAVAILABLE"}"#,
                Verdict::Retry,
            ),
            (
                r#"{"code":3804,"message":"Timeout waiting for response from streaming transcription."}"#,
                Verdict::Retry,
            ),
            (
                r#"{"message":"Model voxtral-mini-transcribe is unavailable for this key","code":3001}"#,
                Verdict::Fatal,
            ),
            (r#"{"message":"Unauthorized","code":401}"#, Verdict::Fatal),
            // Google, as `google.rpc.Status`.
            (
                r#"{"code":503,"message":"The service is currently unavailable.","status":"UNAVAILABLE"}"#,
                Verdict::Retry,
            ),
            (
                r#"{"code":500,"message":"Internal error encountered.","status":"INTERNAL"}"#,
                Verdict::Retry,
            ),
            (
                r#"{"code":429,"message":"Resource has been exhausted (e.g. check quota).","status":"RESOURCE_EXHAUSTED"}"#,
                Verdict::Fatal,
            ),
            (
                r#"{"code":400,"message":"bad","status":"INVALID_ARGUMENT"}"#,
                Verdict::Fatal,
            ),
            (
                r#"{"code":403,"message":"The caller does not have permission","status":"PERMISSION_DENIED"}"#,
                Verdict::Fatal,
            ),
        ] {
            let value: serde_json::Value = serde_json::from_str(error).unwrap();
            assert_eq!(classify_json(&value), verdict, "{error}");
        }
    }

    #[test]
    fn a_verdict_becomes_the_control_the_runner_acts_on() {
        assert!(matches!(
            Verdict::Handover.control(String::new),
            MessageControl::Handover { time_left: None }
        ));
        assert!(matches!(
            Verdict::Retry.control(|| unreachable!("only a fatal verdict words its detail")),
            MessageControl::Reconnect
        ));
        let MessageControl::Fatal(detail) = Verdict::Fatal.control(|| "refused".into()) else {
            panic!("expected a fatal control");
        };
        assert_eq!(detail, "refused");
    }

    #[test]
    fn sentence_ends_include_closing_quotes_and_other_scripts() {
        for text in [
            "direct\u{a0}?",
            "« Bonjour.\u{a0}»",
            "(fin.)",
            "完了。",
            "انتهى؟",
            "Ende!  ",
        ] {
            assert!(ends_sentence(text), "{text}");
        }
        for text in ["Utilisez cet", "à l'", "Dr", "", "and,"] {
            assert!(!ends_sentence(text), "{text}");
        }
    }

    #[test]
    fn a_frame_that_does_not_parse_is_skipped_rather_than_fatal() {
        let parsed: Option<serde_json::Value> = parse_or_log("Test", r#"{"type":"ok"}"#);
        assert_eq!(parsed, Some(serde_json::json!({"type": "ok"})));
        assert_eq!(
            parse_or_log::<serde_json::Value>("Test", "{truncated"),
            None
        );
    }
}
