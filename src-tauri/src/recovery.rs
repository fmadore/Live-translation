//! Crash-recovery spool and the close guard that protects an unsaved transcript.
//!
//! Two small pieces of the same promise (issue #25): a session's captions should survive a
//! crash if the operator asked for that, and closing the window should never silently throw
//! them away.
//!
//! The spool is deliberately dumb. The core writes and reads one opaque UTF-8 file and never
//! looks inside it — the snapshot format lives in `src/lib/document.ts`, and the front-end
//! puts only finalized caption lines in it. There is no key material and no audio anywhere
//! near this path: keys live in Windows Credential Manager and are never sent to the
//! renderer, and captured audio is never written to disk at all.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Manager, State};

use crate::session::SessionManager;

/// Label of the window the operator drives. The overlay is a caption surface with no
/// controls, so it is not the window a close guard applies to.
pub const OPERATOR_LABEL: &str = "operator";

/// Exactly one spool file, overwritten in place. Retention is that simple on purpose: it
/// holds the transcript that is currently unsaved and nothing else, so it never accumulates
/// history and never outlives the text it protects.
const RECOVERY_DIR: &str = "recovery";
const RECOVERY_FILE: &str = "transcript.json";

/// A spool found on disk. `contents` is handed over verbatim for the front-end to parse;
/// `path` is shown to the operator so they know exactly which file to delete.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredRecovery {
    pub path: String,
    pub contents: String,
}

/// How long an intercepted close waits for the front-end to say it is on it. Generous next to
/// an event-loop round trip and short next to a person noticing a window that did not close.
pub const ACK_TIMEOUT: Duration = Duration::from_secs(3);

/// Whether a window close has to be held open for an answer, and whether one has been given.
///
/// The front-end keeps `guard` current (unsaved text, or a live session that still has to be
/// drained). Two things keep this from ever producing a window that will not close: while
/// `guard` is false the close event is not touched at all, and an interception that the
/// front-end does not acknowledge within `ACK_TIMEOUT` is released — so a renderer that wedges
/// or dies *while* something was unsaved costs the operator three seconds, not Task Manager.
#[derive(Default)]
pub struct CloseGuard {
    guard: AtomicBool,
    confirmed: AtomicBool,
    /// Closes intercepted so far, and how many of them the front-end answered for. Counters
    /// rather than a flag so a second close attempt cannot be satisfied by the first ack.
    attempts: AtomicU64,
    acked: AtomicU64,
}

impl CloseGuard {
    pub fn set(&self, guard: bool) {
        self.guard.store(guard, Ordering::Relaxed);
    }

    /// True while a close should be intercepted. Once the operator has answered the prompt
    /// this stays false, so the quit that follows goes straight through.
    pub fn should_intercept(&self) -> bool {
        !self.confirmed.load(Ordering::Relaxed) && self.guard.load(Ordering::Relaxed)
    }

    /// Record an interception and return its number, for the watchdog to check against.
    pub fn begin_attempt(&self) -> u64 {
        self.attempts.fetch_add(1, Ordering::Relaxed) + 1
    }

    /// Whether the front-end took responsibility for attempt `attempt`.
    pub fn acknowledged(&self, attempt: u64) -> bool {
        self.acked.load(Ordering::Relaxed) >= attempt
    }

    fn ack(&self) {
        self.acked
            .store(self.attempts.load(Ordering::Relaxed), Ordering::Relaxed);
    }

    fn confirm(&self) {
        self.confirmed.store(true, Ordering::Relaxed);
    }

    /// Give up on an unanswered interception: the operator asked to close and nothing is
    /// listening, so holding the window shut protects nobody.
    pub fn release(&self) {
        self.confirm();
    }
}

fn recovery_path(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_local_data_dir()
        .map(|dir| dir.join(RECOVERY_DIR).join(RECOVERY_FILE))
        .map_err(|error| format!("no application data directory: {error}"))
}

/// Overwrite the spool. Called on a timer while recovery is enabled and the document is
/// unsaved; a no-op path otherwise, because nothing else ever calls it.
#[tauri::command]
pub async fn write_recovery(app: AppHandle, contents: String) -> Result<String, String> {
    let path = recovery_path(&app)?;
    tauri::async_runtime::spawn_blocking(move || {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| format!("could not create {dir:?}: {e}"))?;
        }
        std::fs::write(&path, contents).map_err(|e| format!("could not write {path:?}: {e}"))?;
        Ok(path.to_string_lossy().into_owned())
    })
    .await
    .map_err(|error| format!("recovery write task failed: {error}"))?
}

/// Read the spool left behind by a previous run. A missing file is the normal case — the
/// app was closed cleanly — and answers `None` rather than an error.
#[tauri::command]
pub async fn read_recovery(app: AppHandle) -> Result<Option<StoredRecovery>, String> {
    let path = recovery_path(&app)?;
    tauri::async_runtime::spawn_blocking(move || match std::fs::read_to_string(&path) {
        Ok(contents) => Ok(Some(StoredRecovery {
            path: path.to_string_lossy().into_owned(),
            contents,
        })),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(format!("could not read {path:?}: {error}")),
    })
    .await
    .map_err(|error| format!("recovery read task failed: {error}"))?
}

/// Delete the spool: on save, on clear, on discard, when recovery is switched off, and once
/// a recovered transcript has been taken or refused. An absent file is success.
#[tauri::command]
pub async fn clear_recovery(app: AppHandle) -> Result<(), String> {
    let path = recovery_path(&app)?;
    tauri::async_runtime::spawn_blocking(move || match std::fs::remove_file(&path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!("could not delete {path:?}: {error}")),
    })
    .await
    .map_err(|error| format!("recovery delete task failed: {error}"))?
}

/// Sent the moment the front-end receives a `close-requested`, before it starts stopping the
/// session. It only claims "something is handling this"; the answer follows separately.
#[tauri::command]
pub async fn ack_close(close_guard: State<'_, CloseGuard>) -> Result<(), String> {
    close_guard.ack();
    Ok(())
}

#[tauri::command]
pub async fn set_close_guard(
    close_guard: State<'_, CloseGuard>,
    guard: bool,
) -> Result<(), String> {
    close_guard.set(guard);
    Ok(())
}

/// The operator answered the unsaved-transcript prompt and chose to leave.
///
/// Stopping the session again is intentional belt-and-braces: the front-end already awaited
/// a stop before it prompted, so this is normally an immediate no-op, but it means the
/// guarantee that quitting releases the capture devices does not depend on the renderer
/// having got that far.
#[tauri::command]
pub async fn confirm_close(
    app: AppHandle,
    manager: State<'_, SessionManager>,
    close_guard: State<'_, CloseGuard>,
) -> Result<(), String> {
    close_guard.confirm();
    manager.stop(&app).await;
    app.exit(0);
    Ok(())
}
