//! What the providers' protocols share on the wire, so each provider module only spells out
//! what is its own.

use anyhow::{Context, Result};
use serde::de::DeserializeOwned;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::handshake::client::Request;
use tokio_tungstenite::tungstenite::http::{header::AUTHORIZATION, HeaderValue};

use super::MessageControl;

/// A WebSocket handshake authenticated with an `Authorization: Bearer` header, as OpenAI and
/// Mistral both require. `provider` names it in the errors, which the operator reads as the
/// detail of a reconnecting status.
pub fn bearer_request(provider: &str, url: &str, api_key: &str) -> Result<Request> {
    let mut request = url
        .into_client_request()
        .with_context(|| format!("failed to build {provider} request"))?;
    let bearer = HeaderValue::from_str(&format!("Bearer {api_key}"))
        .with_context(|| format!("{provider} API key is not a valid header value"))?;
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
/// is up. Specific on purpose: a bare "unavailable" could also describe a model the key cannot
/// use.
const TRANSIENT_WORDING: [&str; 5] = [
    "statuscode.unavailable",
    "grpc_status:14",
    "statuscode.deadline_exceeded",
    "grpc connection error",
    "connection reset by peer",
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
    fn a_frame_that_does_not_parse_is_skipped_rather_than_fatal() {
        let parsed: Option<serde_json::Value> = parse_or_log("Test", r#"{"type":"ok"}"#);
        assert_eq!(parsed, Some(serde_json::json!({"type": "ok"})));
        assert_eq!(
            parse_or_log::<serde_json::Value>("Test", "{truncated"),
            None
        );
    }
}
