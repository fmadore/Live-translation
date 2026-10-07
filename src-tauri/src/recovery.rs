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
use std::sync::Mutex;

use serde::Serialize;
use tauri::{AppHandle, Manager};

use crate::atomic_file;

/// Exactly one committed spool, replaced from a flushed sibling file (`atomic_file`). It
/// holds the transcript that is currently unsaved and nothing else, so it never accumulates
/// history and never outlives the text it protects.
const RECOVERY_DIR: &str = "recovery";
const RECOVERY_FILE: &str = "transcript.json";

// The renderer orders operations; the core also excludes concurrent filesystem access,
// including during shutdown. Hold this only on blocking threads, never across an await.
static RECOVERY_IO: Mutex<()> = Mutex::new(());

/// A spool found on disk. `contents` is handed over verbatim for the front-end to parse;
/// `path` is shown to the operator so they know exactly which file to delete.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredRecovery {
    pub path: String,
    pub contents: String,
}

/// The identifier this app used up to 1.1.0. `app_local_data_dir()` is derived from the
/// current one, so the 1.2.0 rename moved this directory — and a spool written by 1.1.0 sits
/// under the old name, which is exactly the crash the feature exists to survive.
const LEGACY_IDENTIFIER: &str = "org.stias.live-translation";

fn recovery_path(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_local_data_dir()
        .map(|dir| dir.join(RECOVERY_DIR).join(RECOVERY_FILE))
        .map_err(|error| format!("no application data directory: {error}"))
}

/// Where 1.1.0 would have left a spool: the same parent directory, under the old identifier.
///
/// Read and deleted, never written — nothing after this release puts a file there, so the
/// path exists only to finish emptying it. `None` when the data directory has no parent,
/// which cannot happen on Windows but is not worth a panic to assert.
fn legacy_recovery_path(app: &AppHandle) -> Option<PathBuf> {
    let dir = app.path().app_local_data_dir().ok()?;
    Some(
        dir.parent()?
            .join(LEGACY_IDENTIFIER)
            .join(RECOVERY_DIR)
            .join(RECOVERY_FILE),
    )
}

/// Overwrite the spool. Called on a timer while recovery is enabled and the document is
/// unsaved; a no-op path otherwise, because nothing else ever calls it.
#[tauri::command]
pub async fn write_recovery(app: AppHandle, contents: String) -> Result<String, String> {
    let path = recovery_path(&app)?;
    tauri::async_runtime::spawn_blocking(move || {
        let _io = RECOVERY_IO.lock().unwrap_or_else(|p| p.into_inner());
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| format!("could not create {dir:?}: {e}"))?;
        }
        atomic_file::replace(&path, contents.as_bytes())
            .map_err(|e| format!("could not write {path:?}: {e}"))?;
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
    // The pre-1.2.0 location is consulted only when the current one is empty, and only for
    // reading. An operator whose 1.1.0 session died and who then updated would otherwise open
    // 1.2.0 to no prompt at all — the transcript still on disk, under a directory the app no
    // longer looks at.
    let legacy = legacy_recovery_path(&app);
    tauri::async_runtime::spawn_blocking(move || {
        let _io = RECOVERY_IO.lock().unwrap_or_else(|p| p.into_inner());
        // An interrupted replacement never supersedes the last committed snapshot.
        atomic_file::discard_staging(&path)
            .map_err(|error| format!("could not remove incomplete recovery snapshot: {error}"))?;
        let found = match std::fs::read_to_string(&path) {
            Ok(contents) => Some((path, contents)),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => match legacy {
                Some(legacy) => match std::fs::read_to_string(&legacy) {
                    Ok(contents) => Some((legacy, contents)),
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
                    Err(error) => return Err(format!("could not read {legacy:?}: {error}")),
                },
                None => None,
            },
            Err(error) => return Err(format!("could not read {path:?}: {error}")),
        };
        // The path travels with the contents because the prompt shows it: told the wrong one,
        // an operator pressing Delete would be told a file was removed that still exists.
        Ok(found.map(|(path, contents)| StoredRecovery {
            path: path.to_string_lossy().into_owned(),
            contents,
        }))
    })
    .await
    .map_err(|error| format!("recovery read task failed: {error}"))?
}

/// Delete the spool: on save, on clear, on discard, when recovery is switched off, and once
/// a recovered transcript has been taken or refused. An absent file is success.
#[tauri::command]
pub async fn clear_recovery(app: AppHandle) -> Result<(), String> {
    let path = recovery_path(&app)?;
    // Both locations, because a spool that was *read* from the pre-1.2.0 directory has to be
    // deletable from it too. Clearing only the current path would leave the recovered
    // transcript on disk and offer it again at every launch.
    let legacy = legacy_recovery_path(&app);
    tauri::async_runtime::spawn_blocking(move || {
        let _io = RECOVERY_IO.lock().unwrap_or_else(|p| p.into_inner());
        if let Some(legacy) = legacy {
            atomic_file::remove(&legacy)
                .map_err(|error| format!("could not delete {legacy:?}: {error}"))?;
        }
        atomic_file::remove(&path).map_err(|error| format!("could not delete {path:?}: {error}"))
    })
    .await
    .map_err(|error| format!("recovery delete task failed: {error}"))?
}
