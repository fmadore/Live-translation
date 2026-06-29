//! API-key storage, per translation provider. Prefers the OS keychain (via the `keyring`
//! crate); falls back to a `GEMINI_API_KEY` / `OPENAI_API_KEY` environment variable (or `.env`)
//! for development. Keys are only ever read on the Rust side to open the WebSocket — they never
//! reach the renderer.

use anyhow::{Context, Result};

use crate::types::Provider;

const SERVICE: &str = "org.stias.live-translation";

/// Keychain account name (one entry per provider).
fn account(provider: Provider) -> &'static str {
    match provider {
        Provider::Gemini => "gemini-api-key",
        Provider::OpenAi => "openai-api-key",
    }
}

/// Environment-variable fallback name.
fn env_var(provider: Provider) -> &'static str {
    match provider {
        Provider::Gemini => "GEMINI_API_KEY",
        Provider::OpenAi => "OPENAI_API_KEY",
    }
}

fn label(provider: Provider) -> &'static str {
    match provider {
        Provider::Gemini => "Gemini",
        Provider::OpenAi => "OpenAI",
    }
}

fn entry(provider: Provider) -> Result<keyring::Entry> {
    keyring::Entry::new(SERVICE, account(provider)).context("failed to open OS keychain entry")
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
    if let Ok(pw) = entry(provider).and_then(|e| e.get_password().map_err(Into::into)) {
        if !pw.trim().is_empty() {
            return Ok(pw);
        }
    }
    if let Ok(env_key) = std::env::var(env_var(provider)) {
        if !env_key.trim().is_empty() {
            return Ok(env_key);
        }
    }
    anyhow::bail!(
        "No {} API key found. Save one in the operator window or set {}.",
        label(provider),
        env_var(provider)
    )
}

/// True if a key for this provider is available from either source.
pub fn has_api_key(provider: Provider) -> bool {
    resolve_api_key(provider).is_ok()
}
