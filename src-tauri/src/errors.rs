//! What the core says when something goes wrong, in a form the front end can translate.
//!
//! The rule (issue #23): **the core names the failure, the interface words it.** A command or
//! a status event returns a stable id plus, where there is one, the untranslated technical
//! detail underneath it — a Windows error, a provider's own message, a path. The catalog in
//! `src/lib/i18n` owns the sentence; `describeError` in `src/lib/errors.ts` puts the two
//! together and falls back to the detail alone for an id it does not know, so a core that has
//! learned a new failure is never silent in an older interface.
//!
//! Ids are `dotted.lowerCamel`, grouped by what failed rather than by which command noticed.
//! `id_list` is read by a front-end test that checks the catalog has a sentence for each.

use serde::Serialize;

/// A failure on its way to the operator.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AppError {
    /// Stable identifier; the interface looks up the sentence for it.
    pub id: &'static str,
    /// Technical text — an OS message, a provider's own wording, a path. Never translated,
    /// always shown after the sentence, because it is what makes a report actionable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

impl AppError {
    /// Every failure the core reports today has technical text worth keeping, so there is no
    /// `new(id)` constructor: an id with nothing under it would be a report nobody can act on.
    ///
    /// The detail is written in its alternate form, so an `anyhow::Error` keeps every cause
    /// rather than only the outermost context: "WebSocket connect failed" alone cannot say
    /// whether it was DNS, TLS, a proxy or a 429. Anyhow's alternate form, `outer: cause: root`,
    /// walks the context chain and then each underlying error's `source()`, so a reqwest or
    /// keyring error inside keeps its own causes too. Plain text and OS errors ignore the flag.
    pub fn with(id: &'static str, detail: impl std::fmt::Display) -> Self {
        Self {
            id,
            detail: Some(format!("{detail:#}")),
        }
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.detail {
            Some(detail) => write!(f, "{}: {detail}", self.id),
            None => write!(f, "{}", self.id),
        }
    }
}

/// Every id the core can emit. Adding a variant here without a catalog entry fails the
/// front-end test that reads this file.
pub mod id {
    pub const WHISPER_MODEL: &str = "error.whisperModel";
    pub const WHISPER_SESSION: &str = "error.whisperSession";
    /// This processor lacks the instruction sets the Whisper engine was built for.
    pub const WHISPER_CPU: &str = "error.whisperCpu";
    /// Enumerating input devices failed before any device was opened.
    pub const DEVICE_ENUMERATION: &str = "error.deviceEnumeration";
    /// Windows Credential Manager refused a read or a write.
    pub const KEYCHAIN: &str = "error.keychain";
    /// A pasted API key holds a character no provider key has — whitespace inside it, or
    /// anything outside printable ASCII — so it was not saved. The detail says which and
    /// where, never the key.
    pub const API_KEY_FORMAT: &str = "error.apiKeyFormat";
    /// The bundled demonstration could not be prepared or inspected.
    pub const DEMO_UNAVAILABLE: &str = "error.demoUnavailable";
    /// A session could not be started — bad options, no key, no device.
    pub const SESSION_START: &str = "error.sessionStart";
    /// A pause or resume reached no running session.
    pub const SESSION_PAUSE: &str = "error.sessionPause";
    /// The preflight level test could not open what it was asked to listen to.
    pub const AUDIO_TEST_START: &str = "error.audioTestStart";
    /// The overlay window refused a window-level change.
    pub const OVERLAY_WINDOW: &str = "error.overlayWindow";
    /// The operator window refused to hide itself to the tray.
    pub const OPERATOR_WINDOW: &str = "error.operatorWindow";
    /// An export did not reach the file the operator chose in Save As: the write failed, or
    /// the format or the platform cannot be exported at all.
    pub const TRANSCRIPT_WRITE: &str = "error.transcriptWrite";
    /// The crash-recovery spool could not be written while the transcript was unsaved.
    pub const RECOVERY_WRITE: &str = "error.recoveryWrite";
    /// A spool left by an earlier run could not be read at start-up.
    pub const RECOVERY_READ: &str = "error.recoveryRead";
    /// A spool that is no longer needed could not be deleted.
    pub const RECOVERY_DELETE: &str = "error.recoveryDelete";
    /// A session file in the history folder could not be read, written or deleted. One id for
    /// all three, unlike the spool's: the History tab puts its own sentence in front of it, and
    /// the detail names the file and the reason.
    pub const HISTORY_STORAGE: &str = "error.historyStorage";
    /// Windows would not open the support page in the default browser.
    pub const OPEN_LINK: &str = "error.openLink";
    /// A blocking task panicked or was cancelled. Nothing the operator did.
    pub const TASK_FAILED: &str = "error.taskFailed";
    /// The microphone stopped delivering audio mid-session.
    pub const MIC_STREAM: &str = "error.micStream";
    /// Microphone capture could not start; on Windows this is usually the privacy setting.
    pub const MIC_CAPTURE: &str = "error.micCapture";
    /// System (loopback) capture failed.
    pub const SYSTEM_CAPTURE: &str = "error.systemCapture";
    /// The provider refused the connection outright — wrong key, or no access to the model.
    pub const PROVIDER_REJECTED: &str = "error.providerRejected";
    /// The provider ended the session and said why.
    pub const PROVIDER_STOPPED: &str = "error.providerStopped";
    /// The stream dropped and is being retried.
    pub const PROVIDER_RECONNECTING: &str = "error.providerReconnecting";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_as_an_id_and_an_optional_detail() {
        let bare = serde_json::to_string(&AppError {
            id: id::KEYCHAIN,
            detail: None,
        })
        .unwrap();
        assert_eq!(bare, r#"{"id":"error.keychain"}"#);

        let detailed =
            serde_json::to_string(&AppError::with(id::TRANSCRIPT_WRITE, "access denied")).unwrap();
        assert_eq!(
            detailed,
            r#"{"id":"error.transcriptWrite","detail":"access denied"}"#
        );
    }

    // The detail is the half that survives an interface which has never heard of the id, so
    // it has to be there whenever there is something to say.
    #[test]
    fn keeps_the_technical_text_verbatim() {
        let error = AppError::with(id::MIC_CAPTURE, "Access is denied. (0x80070005)");
        assert_eq!(
            error.detail.as_deref(),
            Some("Access is denied. (0x80070005)")
        );
    }

    /// Keeping only the outermost context is how "model download behind a proxy" became a
    /// report nobody could act on.
    #[test]
    fn a_chain_keeps_every_cause() {
        let error = anyhow::anyhow!("No such host is known. (os error 11001)")
            .context("WebSocket connect failed");
        assert_eq!(
            AppError::with(id::PROVIDER_RECONNECTING, error).detail,
            Some("WebSocket connect failed: No such host is known. (os error 11001)".into())
        );
    }

    /// A plain `std::error::Error` underneath — reqwest's, keyring's — names its own cause
    /// through `source()`, and that survives as well.
    #[test]
    fn a_chain_walks_the_sources_of_an_error_underneath() {
        #[derive(Debug)]
        struct Request(std::io::Error);
        impl std::fmt::Display for Request {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("error sending request")
            }
        }
        impl std::error::Error for Request {
            fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
                Some(&self.0)
            }
        }

        let refused = std::io::Error::other("proxy refused the tunnel");
        let error = anyhow::Error::new(Request(refused)).context("Model download failed");
        assert_eq!(
            AppError::with(id::WHISPER_MODEL, error).detail,
            Some("Model download failed: error sending request: proxy refused the tunnel".into())
        );
    }
}
