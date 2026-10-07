//! What the providers' protocols share on the wire, so each provider module only spells out
//! what is its own.

use anyhow::{Context, Result};
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::handshake::client::Request;
use tokio_tungstenite::tungstenite::http::{header::AUTHORIZATION, HeaderValue};

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
