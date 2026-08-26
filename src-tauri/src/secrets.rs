//! API-key storage, per translation provider. Prefers the OS keychain (via the `keyring`
//! crate); falls back to a `GEMINI_API_KEY` / `OPENAI_API_KEY` environment variable (or `.env`)
//! for development. Keys are only ever read on the Rust side to open the WebSocket — they never
//! reach the renderer.

use anyhow::{Context, Result};

use crate::types::Provider;

const SERVICE: &str = "org.stias.live-translation";

/// Credential-store account name (one entry per provider). `None` for backends that need
/// no credential.
fn account(provider: Provider) -> Option<&'static str> {
    match provider {
        Provider::Gemini => Some("gemini-api-key"),
        Provider::OpenAi => Some("openai-api-key"),
        Provider::Mistral => Some("mistral-api-key"),
        Provider::OnDevice => None,
    }
}

/// Environment-variable fallback name.
fn env_var(provider: Provider) -> Option<&'static str> {
    match provider {
        Provider::Gemini => Some("GEMINI_API_KEY"),
        Provider::OpenAi => Some("OPENAI_API_KEY"),
        Provider::Mistral => Some("MISTRAL_API_KEY"),
        Provider::OnDevice => None,
    }
}

fn label(provider: Provider) -> &'static str {
    match provider {
        Provider::Gemini => "Gemini",
        Provider::OpenAi => "OpenAI",
        Provider::Mistral => "Mistral",
        Provider::OnDevice => "Built-in demo",
    }
}

fn entry(provider: Provider) -> Result<keyring::Entry> {
    let account =
        account(provider).with_context(|| format!("{} needs no API key", label(provider)))?;
    keyring::Entry::new(SERVICE, account).context("failed to open OS keychain entry")
}

/// Store the provider's key in the OS keychain.
pub fn set_api_key(provider: Provider, key: &str) -> Result<()> {
    let key = key.trim();
    if key.is_empty() {
        anyhow::bail!("API key is empty");
    }
    entry(provider)?
        .set_password(key)
        .context("failed to write key to keychain")?;
    Ok(())
}

/// Remove the provider's key from the keychain (no-op if absent).
pub fn clear_api_key(provider: Provider) -> Result<()> {
    match entry(provider)?.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(e).context("failed to delete key from keychain"),
    }
}

/// Resolve the provider's key for use: keychain first, then the environment/.env fallback.
pub fn resolve_api_key(provider: Provider) -> Result<String> {
    let Some(env_var) = env_var(provider) else {
        anyhow::bail!("{} needs no API key", label(provider));
    };
    if let Ok(pw) = entry(provider).and_then(|e| e.get_password().map_err(Into::into)) {
        if !pw.trim().is_empty() {
            return Ok(pw);
        }
    }
    if let Ok(env_key) = std::env::var(env_var) {
        if !env_key.trim().is_empty() {
            return Ok(env_key);
        }
    }
    anyhow::bail!(
        "No {} API key found. Save one in the operator window or set {}.",
        label(provider),
        env_var
    )
}

/// True once this provider is ready to start a session. Backends that need no credential
/// are always ready, so the operator's Start button is never gated on a key for them.
pub fn has_api_key(provider: Provider) -> bool {
    if !provider.requires_api_key() {
        return true;
    }
    resolve_api_key(provider).is_ok()
}
