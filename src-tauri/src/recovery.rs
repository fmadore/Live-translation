//! Crash-recovery spool for an unsaved transcript (issue #25).
//!
//! The half of that promise a prompt cannot keep: if the operator asked for it, a session's
//! captions survive a crash or a power cut. What happens when the window is *closed* lives
//! in `lifecycle.rs`.
//!
//! The spool is deliberately dumb. The core writes and reads one opaque UTF-8 file and never
//! looks inside it — the snapshot format lives in `src/lib/document.ts`, and the front-end
//! puts only finalized caption lines in it. There is no key material and no audio anywhere
//! near this path: keys live in Windows Credential Manager and are never sent to the
//! renderer, and captured audio is never written to disk at all.

use std::path::PathBuf;

use serde::Serialize;
use tauri::{AppHandle, Manager};

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
