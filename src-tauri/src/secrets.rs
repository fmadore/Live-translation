//! API-key storage. Prefers the OS keychain (via the `keyring` crate); falls back to a
//! `GEMINI_API_KEY` environment variable / `.env` for development. The key is only ever
//! read on the Rust side and used to open the WebSocket — it never reaches the renderer.

use anyhow::{Context, Result};

const SERVICE: &str = "org.stias.live-translation";
const ACCOUNT: &str = "gemini-api-key";

fn entry() -> Result<keyring::Entry> {
    keyring::Entry::new(SERVICE, ACCOUNT).context("failed to open OS keychain entry")
}

/// Store the key in the OS keychain.
pub fn set_api_key(key: &str) -> Result<()> {
    let key = key.trim();
    if key.is_empty() {
        anyhow::bail!("API key is empty");
    }
    entry()?
        .set_password(key)
        .context("failed to write key to keychain")?;
    Ok(())
}

/// Remove the key from the keychain (no-op if absent).
pub fn clear_api_key() -> Result<()> {
    match entry()?.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(e).context("failed to delete key from keychain"),
    }
}

/// Resolve the key for use: keychain first, then `GEMINI_API_KEY` from the environment/.env.
pub fn resolve_api_key() -> Result<String> {
    if let Ok(pw) = entry().and_then(|e| e.get_password().map_err(Into::into)) {
        if !pw.trim().is_empty() {
            return Ok(pw);
        }
    }
    if let Ok(env_key) = std::env::var("GEMINI_API_KEY") {
        if !env_key.trim().is_empty() {
            return Ok(env_key);
        }
    }
    anyhow::bail!("No Gemini API key found. Save one in the operator window or set GEMINI_API_KEY.")
}

/// True if a key is available from either source.
pub fn has_api_key() -> bool {
    resolve_api_key().is_ok()
}
