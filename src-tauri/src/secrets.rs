//! API-key storage, per translation provider. Prefers the OS keychain (via the `keyring`
//! crate); falls back to a `GEMINI_API_KEY` / `OPENAI_API_KEY` environment variable (or `.env`)
//! for development. Keys are only ever read on the Rust side to open the WebSocket — they never
//! reach the renderer.

use anyhow::{Context, Result};

use crate::types::Provider;

const SERVICE: &str = "io.github.fmadore.live-translation";

/// The service name up to 1.1.0, when the bundle identifier still named the workshop venue
/// rather than the author. Read once per provider so the rename does not cost an operator the
/// key they saved, then deleted — a credential this app no longer writes should not sit in
/// Credential Manager under a name that misstates who put it there.
const LEGACY_SERVICE: &str = "org.stias.live-translation";

/// Credential-store account name (one entry per provider). `None` for backends that need
/// no credential.
fn account(provider: Provider) -> Option<&'static str> {
    match provider {
        // Both Gemini backends read the same credential on purpose: one key from AI Studio
        // covers translation and subtitles, so switching mode never asks for it again.
        Provider::Gemini | Provider::GeminiTranscribe => Some("gemini-api-key"),
        Provider::OpenAi => Some("openai-api-key"),
        Provider::Mistral => Some("mistral-api-key"),
        Provider::OnDevice | Provider::Whisper | Provider::WhisperTranslate => None,
    }
}

/// Environment-variable fallback name.
fn env_var(provider: Provider) -> Option<&'static str> {
    match provider {
        Provider::Gemini | Provider::GeminiTranscribe => Some("GEMINI_API_KEY"),
        Provider::OpenAi => Some("OPENAI_API_KEY"),
        Provider::Mistral => Some("MISTRAL_API_KEY"),
        Provider::OnDevice | Provider::Whisper | Provider::WhisperTranslate => None,
    }
}

fn label(provider: Provider) -> &'static str {
    match provider {
        Provider::Gemini | Provider::GeminiTranscribe => "Gemini",
        Provider::OpenAi => "OpenAI",
        Provider::Mistral => "Mistral",
        Provider::OnDevice => "Built-in demo",
        Provider::Whisper | Provider::WhisperTranslate => "Whisper",
    }
}

fn entry_in(service: &str, provider: Provider) -> Result<keyring::Entry> {
    let account =
        account(provider).with_context(|| format!("{} needs no API key", label(provider)))?;
    keyring::Entry::new(service, account).context("failed to open OS keychain entry")
}

fn entry(provider: Provider) -> Result<keyring::Entry> {
    entry_in(SERVICE, provider)
}

/// Move a key saved under the pre-1.2.0 service name across, and hand it back.
///
/// Only reached when the current name holds nothing, so a migrated key is written once and
/// never looked for again. An update must not cost the operator their provider key: finding
/// out it is gone happens at the moment they press Start, which is five minutes before an
/// event rather than five days.
fn take_legacy_key(provider: Provider) -> Option<String> {
    let legacy = entry_in(LEGACY_SERVICE, provider).ok()?;
    let key = legacy.get_password().ok()?;
    if key.trim().is_empty() {
        return None;
    }
    // A write that fails leaves the old entry untouched, so the next launch tries again. A
    // delete that fails is not worth surfacing: the key has already moved, and the worst case
    // is a stale duplicate rather than a lost credential.
    entry(provider).ok()?.set_password(&key).ok()?;
    let _ = legacy.delete_credential();
    Some(key)
}

/// A character in a pasted key that no provider issues, and where it is. Its `Display` is the
/// operator's detail for `error.apiKeyFormat`, so it says which character and where without
/// repeating the key: a status line gets screenshotted and pasted into tickets.
#[derive(Debug, PartialEq, Eq)]
pub struct KeyFormat {
    character: char,
    /// One-based, in characters of the trimmed key.
    position: usize,
}

impl std::fmt::Display for KeyFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "character U+{:04X} at position {}",
            u32::from(self.character),
            self.position
        )
    }
}

impl std::error::Error for KeyFormat {}

/// Refuse a key that cannot be one. Every provider's keys are printable ASCII without spaces,
/// and a character outside that — a zero-width space or a line break picked up with the paste,
/// a space where a wrapped key was rejoined — can never authenticate. Saved anyway, it made a
/// header or a URL that could not be built, which the source then retried every 16 seconds
/// for the rest of the event.
fn check_format(key: &str) -> std::result::Result<(), KeyFormat> {
    match key
        .chars()
        .zip(1..)
        .find(|(character, _)| !matches!(character, '!'..='~'))
    {
        Some((character, position)) => Err(KeyFormat {
            character,
            position,
        }),
        None => Ok(()),
    }
}

/// Store the provider's key in the OS keychain. Whitespace around it is dropped, as a paste
/// often brings some; anything else a key cannot contain is refused with a `KeyFormat`.
pub fn set_api_key(provider: Provider, key: &str) -> Result<()> {
    let key = key.trim();
    if key.is_empty() {
        anyhow::bail!("API key is empty");
    }
    check_format(key)?;
    entry(provider)?
        .set_password(key)
        .context("failed to write key to keychain")?;
    Ok(())
}

/// Remove the provider's key from the keychain (no-op if absent).
pub fn clear_api_key(provider: Provider) -> Result<()> {
    // The legacy entry too, and first: a Remove that left it behind would be undone by the
    // migration on the next launch, which reads as the app refusing to forget a key.
    if let Ok(legacy) = entry_in(LEGACY_SERVICE, provider) {
        let _ = legacy.delete_credential();
    }
    match entry(provider)?.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
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
    if let Some(migrated) = take_legacy_key(provider) {
        return Ok(migrated);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_as_the_providers_issue_them_pass() {
        for key in [
            // A made-up key in a Gemini key's shape, joined at compile time: written out whole
            // it matches GitHub's Google API key pattern and raises a secret-scanning alert.
            concat!("AIza", "SyA-1b2C3d4E5f6G7h8I9j0K_lMnOpQrStU"),
            "sk-proj-Ab3dEf_gH1jK-lMn0pQ",
            "x8Yq2LmN4pR6sT0vW3zA5bC7dE9fG1hJ",
        ] {
            assert_eq!(check_format(key), Ok(()), "{key}");
        }
    }

    #[test]
    fn a_character_no_key_contains_is_named_with_its_position() {
        for (key, character, position) in [
            // Picked up with a paste from a web page or a chat.
            ("sk-abc\u{200B}def", '\u{200B}', 7),
            // A key wrapped across two lines and rejoined.
            ("sk-abc def", ' ', 7),
            ("sk-abc\ndef", '\n', 7),
            ("\tsk", '\t', 1),
            ("sk-abcé", 'é', 7),
        ] {
            assert_eq!(
                check_format(key),
                Err(KeyFormat {
                    character,
                    position
                }),
                "{key:?}"
            );
        }
    }

    /// The detail reaches a status line; the key must not.
    #[test]
    fn the_detail_names_the_character_and_never_the_key() {
        let error = check_format("sk-secret\u{200B}value").unwrap_err();
        let detail = error.to_string();
        assert_eq!(detail, "character U+200B at position 10");
        assert!(!detail.contains("secret"));
    }

    /// Swaps keyring's in-memory mock store in for Credential Manager, once for the whole test
    /// binary, and serializes the tests that use it: the store is process-wide, and these tests
    /// write the same entries. Each test starts from an empty keychain.
    ///
    /// `None` where keyring has no platform store to begin with (a Linux runner without a
    /// Secret Service): `keyring::Entry::new` refuses to make entries there, mock or not.
    fn mock_keychain() -> Option<std::sync::MutexGuard<'static, ()>> {
        static INSTALL: std::sync::Once = std::sync::Once::new();
        static SERIAL: std::sync::Mutex<()> = std::sync::Mutex::new(());
        // keyring's `v1` entries install the platform store the first time they are used;
        // let that happen first, or it would replace the mock.
        if keyring::Entry::store_status().is_err() {
            eprintln!("no platform credential store; skipping");
            return None;
        }
        INSTALL.call_once(|| {
            keyring_core::set_default_store(keyring_core::mock::Store::new().unwrap());
        });
        let guard = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
        for provider in [Provider::Gemini, Provider::OpenAi, Provider::Mistral] {
            for service in [SERVICE, LEGACY_SERVICE] {
                let _ = entry_in(service, provider).unwrap().delete_credential();
            }
        }
        Some(guard)
    }

    fn stored(service: &str, provider: Provider) -> Option<String> {
        entry_in(service, provider).unwrap().get_password().ok()
    }

    fn store(service: &str, provider: Provider, key: &str) {
        entry_in(service, provider)
            .unwrap()
            .set_password(key)
            .unwrap();
    }

    #[test]
    fn a_saved_key_is_trimmed_stored_and_resolved() {
        let Some(_keychain) = mock_keychain() else {
            return;
        };
        set_api_key(Provider::Mistral, "  x8Yq2LmN4pR6sT0v\n").unwrap();
        assert_eq!(
            stored(SERVICE, Provider::Mistral).as_deref(),
            Some("x8Yq2LmN4pR6sT0v")
        );
        assert_eq!(
            resolve_api_key(Provider::Mistral).unwrap(),
            "x8Yq2LmN4pR6sT0v"
        );
        assert!(has_api_key(Provider::Mistral));
    }

    /// One key from AI Studio covers translation and subtitles.
    #[test]
    fn both_gemini_engines_read_the_one_gemini_key() {
        let Some(_keychain) = mock_keychain() else {
            return;
        };
        set_api_key(Provider::GeminiTranscribe, "AIza-shared").unwrap();
        assert_eq!(resolve_api_key(Provider::Gemini).unwrap(), "AIza-shared");
    }

    /// B6: a key that cannot be one never reaches the keychain, and does not displace the one
    /// that is there.
    #[test]
    fn a_malformed_key_leaves_the_stored_one_alone() {
        let Some(_keychain) = mock_keychain() else {
            return;
        };
        set_api_key(Provider::OpenAi, "sk-good").unwrap();
        for bad in ["sk-abc\u{200B}def", "sk-abc def", "", "   "] {
            assert!(set_api_key(Provider::OpenAi, bad).is_err(), "{bad:?}");
        }
        assert_eq!(
            stored(SERVICE, Provider::OpenAi).as_deref(),
            Some("sk-good")
        );
    }

    /// An update must not cost the operator the key they saved under the pre-1.2.0 name: it is
    /// found, moved across, and the old entry removed.
    #[test]
    fn a_key_under_the_legacy_name_is_moved_across_on_first_use() {
        let Some(_keychain) = mock_keychain() else {
            return;
        };
        store(LEGACY_SERVICE, Provider::OpenAi, "sk-from-1-1");
        assert_eq!(resolve_api_key(Provider::OpenAi).unwrap(), "sk-from-1-1");
        assert_eq!(
            stored(SERVICE, Provider::OpenAi).as_deref(),
            Some("sk-from-1-1")
        );
        assert_eq!(stored(LEGACY_SERVICE, Provider::OpenAi), None);
        // Found under the current name from now on.
        assert_eq!(resolve_api_key(Provider::OpenAi).unwrap(), "sk-from-1-1");
    }

    /// The current name wins; the legacy entry is not even read once there is one.
    #[test]
    fn a_key_under_the_current_name_is_not_replaced_by_a_legacy_one() {
        let Some(_keychain) = mock_keychain() else {
            return;
        };
        store(SERVICE, Provider::Mistral, "current");
        store(LEGACY_SERVICE, Provider::Mistral, "legacy");
        assert_eq!(resolve_api_key(Provider::Mistral).unwrap(), "current");
        assert_eq!(
            stored(LEGACY_SERVICE, Provider::Mistral).as_deref(),
            Some("legacy")
        );
    }

    /// A migration whose write fails keeps the old entry, so the next launch tries again rather
    /// than the key being lost between the two.
    #[test]
    fn a_failed_migration_keeps_the_legacy_key() {
        let Some(_keychain) = mock_keychain() else {
            return;
        };
        store(LEGACY_SERVICE, Provider::Mistral, "legacy");
        let current = entry(Provider::Mistral).unwrap();
        let mock: &keyring_core::mock::Cred = current.inner.as_any().downcast_ref().unwrap();
        mock.set_error(keyring_core::Error::Invalid(
            "mock".into(),
            "write refused".into(),
        ));
        assert_eq!(take_legacy_key(Provider::Mistral), None);
        assert_eq!(
            stored(LEGACY_SERVICE, Provider::Mistral).as_deref(),
            Some("legacy")
        );
        // The next attempt goes through.
        assert_eq!(
            take_legacy_key(Provider::Mistral).as_deref(),
            Some("legacy")
        );
        assert_eq!(stored(LEGACY_SERVICE, Provider::Mistral), None);
    }

    /// A blank legacy entry is no key: nothing is migrated and nothing written.
    #[test]
    fn a_blank_legacy_entry_is_not_migrated() {
        let Some(_keychain) = mock_keychain() else {
            return;
        };
        store(LEGACY_SERVICE, Provider::OpenAi, "  ");
        assert_eq!(take_legacy_key(Provider::OpenAi), None);
        assert_eq!(stored(SERVICE, Provider::OpenAi), None);
    }

    /// Remove takes the legacy entry too, or the migration would bring the key back on the
    /// next launch; and removing what is not there is not an error.
    #[test]
    fn clearing_a_key_removes_both_names_and_tolerates_neither() {
        let Some(_keychain) = mock_keychain() else {
            return;
        };
        store(SERVICE, Provider::OpenAi, "current");
        store(LEGACY_SERVICE, Provider::OpenAi, "legacy");
        clear_api_key(Provider::OpenAi).unwrap();
        assert_eq!(stored(SERVICE, Provider::OpenAi), None);
        assert_eq!(stored(LEGACY_SERVICE, Provider::OpenAi), None);
        assert_eq!(take_legacy_key(Provider::OpenAi), None);
        clear_api_key(Provider::OpenAi).unwrap();
    }

    /// Backends that need no credential are ready without one, and have no entry to write.
    #[test]
    fn keyless_engines_are_always_ready_and_store_nothing() {
        for provider in [
            Provider::OnDevice,
            Provider::Whisper,
            Provider::WhisperTranslate,
        ] {
            assert!(has_api_key(provider), "{provider:?}");
            assert!(set_api_key(provider, "anything").is_err(), "{provider:?}");
            assert!(resolve_api_key(provider).is_err(), "{provider:?}");
        }
    }

    /// Refused before Credential Manager is opened, and surrounding whitespace is still only
    /// trimmed: positions count from the key's first real character.
    #[test]
    fn a_malformed_key_is_refused_before_anything_is_stored() {
        let error = set_api_key(Provider::OpenAi, "  sk-abc def\n").unwrap_err();
        assert_eq!(
            error.downcast_ref::<KeyFormat>(),
            Some(&KeyFormat {
                character: ' ',
                position: 7
            })
        );
    }
}
