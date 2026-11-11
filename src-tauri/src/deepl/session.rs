//! The REST half of a DeepL Voice session: asking for one, and resuming one after its socket
//! dropped. Each answer is a single-use streaming URL and token for the next socket.
//!
//! Nothing here may log the key or a token. The key travels only in the `Authorization`
//! header, marked sensitive; a reconnect carries the token in its query, so a `reqwest::Error`
//! is stripped of its URL before it is kept, as `zoom::sender` does for Zoom's.

use std::time::Duration;

use anyhow::Context;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderValue};
use reqwest::{Client, RequestBuilder, Url};

use super::protocol::{SessionRequest, Ticket, rest_error_message};
use crate::realtime::PrepareError;

/// Where the session requests go: Voice is served by the paid API host only.
const SESSION_PATH: &str = "/v3/voice/realtime";
/// The connection itself; the runner bounds the whole step by its connect timeout.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

/// How a session request failed.
#[derive(Debug)]
pub enum Refusal {
    /// DeepL answered with this status, and this message when it gave one.
    Status { status: u16, message: String },
    /// No usable answer: the network failed, or the body was not a session.
    Failed(anyhow::Error),
}

impl Refusal {
    /// The runner's end for a failed request, as DeepL documents its statuses: 401 and 403 are
    /// the key or a plan without Voice; 400 (a request it will not take) and 456 (the quota is
    /// spent) are worded by DeepL and never retried; 429 and 5xx are retried with backoff.
    pub fn into_prepare_error(self) -> PrepareError {
        match self {
            Self::Failed(error) => PrepareError::Failed(error),
            Self::Status { status, message } => match status {
                400 | 456 => PrepareError::Fatal(describe(status, &message)),
                408 | 429 | 500..=599 => PrepareError::Failed(anyhow::anyhow!(
                    "session request answered {}",
                    describe(status, &message)
                )),
                // 401 and 403 among them.
                400..=499 => PrepareError::Rejected(status),
                _ => PrepareError::Failed(anyhow::anyhow!(
                    "unexpected answer to the session request: {}",
                    describe(status, &message)
                )),
            },
        }
    }

    /// Whether a reconnect's refusal means the session cannot be resumed — the token is spent,
    /// outdated or expired, "a 400-level error" in DeepL's words — so a new one is requested.
    /// Not the key's refusal, and not a rate limit, which a new session would meet as well.
    pub fn session_gone(&self) -> bool {
        matches!(self, Self::Status { status, .. }
            if (400..=499).contains(status) && !matches!(status, 401 | 403 | 408 | 429))
    }
}

fn describe(status: u16, message: &str) -> String {
    if message.is_empty() {
        format!("HTTP {status}")
    } else {
        format!("HTTP {status}: {message}")
    }
}

/// The client every session request of one source goes through.
pub fn client() -> reqwest::Result<Client> {
    let client = Client::builder()
        // Plain HTTP exists only for the tests' loopback server.
        .https_only(cfg!(not(test)))
        .connect_timeout(CONNECT_TIMEOUT)
        // A redirect would carry the request somewhere DeepL did not answer from.
        .redirect(reqwest::redirect::Policy::none());
    // A developer's own proxy setting must not intercept the tests' loopback server.
    #[cfg(test)]
    let client = client.no_proxy();
    client.build()
}

/// `Authorization: DeepL-Auth-Key <key>`, marked sensitive so no `Debug` prints it.
pub fn auth_header(api_key: &str) -> anyhow::Result<HeaderValue> {
    let mut header = HeaderValue::from_str(&format!("DeepL-Auth-Key {api_key}"))
        .context("DeepL API key is not a valid header value")?;
    header.set_sensitive(true);
    Ok(header)
}

fn endpoint(api_base: &str) -> Result<Url, Refusal> {
    Url::parse(&format!("{api_base}{SESSION_PATH}"))
        .context("failed to build the DeepL session URL")
        .map_err(Refusal::Failed)
}

/// Ask for a new session: `POST /v3/voice/realtime`.
pub async fn request(
    http: &Client,
    api_base: &str,
    auth: &HeaderValue,
    body: &SessionRequest<'_>,
) -> Result<Ticket, Refusal> {
    let body = serde_json::to_vec(body)
        .context("failed to serialize the DeepL session request")
        .map_err(Refusal::Failed)?;
    let request = http
        .post(endpoint(api_base)?)
        .header(AUTHORIZATION, auth.clone())
        .header(CONTENT_TYPE, "application/json")
        .body(body);
    exchange(request, "session request").await
}

/// Resume a session whose socket dropped: `GET /v3/voice/realtime?token=<latest token>`. Its
/// options and translation context carry over.
pub async fn reconnect(
    http: &Client,
    api_base: &str,
    auth: &HeaderValue,
    token: &str,
) -> Result<Ticket, Refusal> {
    let mut url = endpoint(api_base)?;
    url.query_pairs_mut().append_pair("token", token);
    let request = http.get(url).header(AUTHORIZATION, auth.clone());
    exchange(request, "session reconnect").await
}

async fn exchange(request: RequestBuilder, what: &'static str) -> Result<Ticket, Refusal> {
    let sent = async {
        let response = request.send().await?;
        let status = response.status();
        let trace_id = response
            .headers()
            .get("x-trace-id")
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned);
        let body = response.text().await?;
        Ok::<_, reqwest::Error>((status, trace_id, body))
    };
    let (status, trace_id, body) = sent.await.map_err(|error| {
        Refusal::Failed(
            anyhow::Error::new(error.without_url()).context(format!("DeepL {what} failed")),
        )
    })?;
    if !status.is_success() {
        let message = rest_error_message(&body);
        // DeepL asks for its trace id with any report of a failed request.
        tracing::warn!(
            status = status.as_u16(),
            trace_id = trace_id.as_deref().unwrap_or("none"),
            %message,
            "DeepL {what} refused"
        );
        return Err(Refusal::Status {
            status: status.as_u16(),
            message,
        });
    }
    serde_json::from_str(&body).map_err(|error| {
        tracing::warn!(
            trace_id = trace_id.as_deref().unwrap_or("none"),
            "DeepL {what} answered without a session: {error}"
        );
        Refusal::Failed(anyhow::anyhow!(
            "DeepL {what} answered without a streaming URL and token"
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn status(status: u16, message: &str) -> Refusal {
        Refusal::Status {
            status,
            message: message.into(),
        }
    }

    #[test]
    fn statuses_end_the_source_or_retry_as_deepl_documents_them() {
        for (code, expected) in [
            (401, "rejected"),
            (403, "rejected"),
            (404, "rejected"),
            (400, "fatal"),
            (456, "fatal"),
            (429, "retry"),
            (500, "retry"),
            (503, "retry"),
            (302, "retry"),
        ] {
            let got = match status(code, "Quota exceeded").into_prepare_error() {
                PrepareError::Rejected(rejected) => {
                    assert_eq!(rejected, code);
                    "rejected"
                }
                PrepareError::Fatal(message) => {
                    assert_eq!(message, format!("HTTP {code}: Quota exceeded"));
                    "fatal"
                }
                PrepareError::Failed(_) => "retry",
                PrepareError::Malformed(_) => "malformed",
            };
            assert_eq!(got, expected, "{code}");
        }
        let PrepareError::Fatal(message) = status(456, "").into_prepare_error() else {
            panic!("a spent quota is not retried");
        };
        assert_eq!(message, "HTTP 456");
    }

    #[test]
    fn only_a_spent_session_falls_back_to_a_new_one() {
        for (code, gone) in [
            (400, true),
            (404, true),
            (410, true),
            (401, false),
            (403, false),
            (429, false),
            (503, false),
        ] {
            assert_eq!(status(code, "").session_gone(), gone, "{code}");
        }
        assert!(!Refusal::Failed(anyhow::anyhow!("offline")).session_gone());
    }

    #[test]
    fn the_key_is_never_printed_by_debug() {
        let header = auth_header("deepl-secret").unwrap();
        assert!(header.is_sensitive());
        assert_eq!(header.to_str().unwrap(), "DeepL-Auth-Key deepl-secret");
        assert!(!format!("{header:?}").contains("deepl-secret"));
        assert!(auth_header("bad\nkey").is_err());
    }
}
