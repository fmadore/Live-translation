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

use std::fmt::Display;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::Serialize;
use tauri::{AppHandle, Manager};

use crate::atomic_file;
use crate::errors::{AppError, id};
use crate::sync::lock;

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

/// Where the spool lives. `failure` is the id of what the caller was about to do, which is
/// what has failed if there is no data directory to do it in.
fn recovery_path(app: &AppHandle, failure: &'static str) -> Result<PathBuf, AppError> {
    app.path()
        .app_local_data_dir()
        .map(|dir| dir.join(RECOVERY_DIR).join(RECOVERY_FILE))
        .map_err(|error| AppError::with(failure, format!("no application data directory: {error}")))
}

/// A file operation that failed. The detail is the path as Windows writes it, not as Rust
/// debug-prints it: an operator told which file to look at should not have to undo doubled
/// backslashes and quotes first.
fn file_error(failure: &'static str, path: &Path, error: impl Display) -> AppError {
    AppError::with(failure, format!("{} — {error}", path.display()))
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
pub async fn write_recovery(app: AppHandle, contents: String) -> Result<String, AppError> {
    let path = recovery_path(&app, id::RECOVERY_WRITE)?;
    tauri::async_runtime::spawn_blocking(move || {
        let _io = lock(&RECOVERY_IO);
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| file_error(id::RECOVERY_WRITE, dir, e))?;
        }
        atomic_file::replace(&path, contents.as_bytes())
            .map_err(|e| file_error(id::RECOVERY_WRITE, &path, e))?;
        Ok(path.to_string_lossy().into_owned())
    })
    .await
    .map_err(|error| AppError::with(id::TASK_FAILED, error))?
}

/// What `read_recovery` does on its blocking thread, given the two paths: discard an
/// interrupted replacement, then read the current spool, and the legacy one only when the
/// current one is absent. A spool that is not UTF-8 is not the renderer's, and fails as
/// `error.recoveryRead` naming the file, which the operator can then delete.
fn read_spool(path: &Path, legacy: Option<&Path>) -> Result<Option<StoredRecovery>, AppError> {
    // An interrupted replacement never supersedes the last committed snapshot.
    atomic_file::discard_staging(path)
        .map_err(|error| file_error(id::RECOVERY_READ, &atomic_file::staging_path(path), error))?;
    let found = match std::fs::read_to_string(path) {
        Ok(contents) => Some((path, contents)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => match legacy {
            Some(legacy) => match std::fs::read_to_string(legacy) {
                Ok(contents) => Some((legacy, contents)),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
                Err(error) => return Err(file_error(id::RECOVERY_READ, legacy, error)),
            },
            None => None,
        },
        Err(error) => return Err(file_error(id::RECOVERY_READ, path, error)),
    };
    // The path travels with the contents because the prompt shows it: told the wrong one,
    // an operator pressing Delete would be told a file was removed that still exists.
    Ok(found.map(|(path, contents)| StoredRecovery {
        path: path.to_string_lossy().into_owned(),
        contents,
    }))
}

/// Read the spool left behind by a previous run. A missing file is the normal case — the
/// app was closed cleanly — and answers `None` rather than an error.
#[tauri::command]
pub async fn read_recovery(app: AppHandle) -> Result<Option<StoredRecovery>, AppError> {
    let path = recovery_path(&app, id::RECOVERY_READ)?;
    // The pre-1.2.0 location is consulted only when the current one is empty, and only for
    // reading. An operator whose 1.1.0 session died and who then updated would otherwise open
    // 1.2.0 to no prompt at all — the transcript still on disk, under a directory the app no
    // longer looks at.
    let legacy = legacy_recovery_path(&app);
    tauri::async_runtime::spawn_blocking(move || {
        let _io = lock(&RECOVERY_IO);
        read_spool(&path, legacy.as_deref())
    })
    .await
    .map_err(|error| AppError::with(id::TASK_FAILED, error))?
}

/// Delete the spool: on save, on clear, on discard, when recovery is switched off, and once
/// a recovered transcript has been taken or refused. An absent file is success.
#[tauri::command]
pub async fn clear_recovery(app: AppHandle) -> Result<(), AppError> {
    let path = recovery_path(&app, id::RECOVERY_DELETE)?;
    // Both locations, because a spool that was *read* from the pre-1.2.0 directory has to be
    // deletable from it too. Clearing only the current path would leave the recovered
    // transcript on disk and offer it again at every launch.
    let legacy = legacy_recovery_path(&app);
    tauri::async_runtime::spawn_blocking(move || {
        let _io = lock(&RECOVERY_IO);
        if let Some(legacy) = legacy {
            atomic_file::remove(&legacy)
                .map_err(|error| file_error(id::RECOVERY_DELETE, &legacy, error))?;
        }
        atomic_file::remove(&path).map_err(|error| file_error(id::RECOVERY_DELETE, &path, error))
    })
    .await
    .map_err(|error| AppError::with(id::TASK_FAILED, error))?
}

#[cfg(test)]
mod tests {
    use super::*;

    // Formatted with `{:?}`, this reached the operator as "C:\\Users\\…", in quotes.
    #[test]
    fn a_failure_names_the_file_as_windows_writes_it() {
        let path = Path::new(r"C:\Users\Operator\recovery\transcript.json");
        let error = file_error(id::RECOVERY_WRITE, path, "Access is denied. (os error 5)");
        assert_eq!(error.id, "error.recoveryWrite");
        assert_eq!(
            error.detail.as_deref(),
            Some(r"C:\Users\Operator\recovery\transcript.json — Access is denied. (os error 5)")
        );
    }

    /// A scratch directory of the test's own, holding a current and a legacy spool path.
    struct Scratch {
        dir: PathBuf,
        current: PathBuf,
        legacy: PathBuf,
    }

    impl Scratch {
        fn new(name: &str) -> Self {
            let dir =
                std::env::temp_dir().join(format!("recovery-test-{}-{name}", std::process::id()));
            let _ = std::fs::remove_dir_all(&dir);
            let current = dir.join("current").join(RECOVERY_DIR).join(RECOVERY_FILE);
            let legacy = dir
                .join(LEGACY_IDENTIFIER)
                .join(RECOVERY_DIR)
                .join(RECOVERY_FILE);
            for path in [&current, &legacy] {
                std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            }
            Self {
                dir,
                current,
                legacy,
            }
        }

        fn read(&self) -> Result<Option<StoredRecovery>, AppError> {
            read_spool(&self.current, Some(&self.legacy))
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.dir);
        }
    }

    #[test]
    fn the_current_spool_is_preferred_over_the_legacy_one() {
        let scratch = Scratch::new("prefer");
        std::fs::write(&scratch.current, "current").unwrap();
        std::fs::write(&scratch.legacy, "legacy").unwrap();
        let found = scratch.read().unwrap().unwrap();
        assert_eq!(found.contents, "current");
        assert_eq!(found.path, scratch.current.to_string_lossy());
        // The legacy file is only read, never touched.
        assert_eq!(std::fs::read_to_string(&scratch.legacy).unwrap(), "legacy");
    }

    #[test]
    fn the_legacy_spool_is_read_only_when_the_current_one_is_absent() {
        let scratch = Scratch::new("legacy");
        assert!(scratch.read().unwrap().is_none());
        assert!(read_spool(&scratch.current, None).unwrap().is_none());
        std::fs::write(&scratch.legacy, "from 1.1.0").unwrap();
        let found = scratch.read().unwrap().unwrap();
        assert_eq!(found.contents, "from 1.1.0");
        // The prompt names the file the transcript came from, so Delete removes that one.
        assert_eq!(found.path, scratch.legacy.to_string_lossy());
    }

    /// An interrupted replacement leaves a staging file beside the spool: it is removed before
    /// the read and never supersedes the committed snapshot.
    #[test]
    fn a_staging_file_is_discarded_before_the_read() {
        let scratch = Scratch::new("staging");
        let staging = atomic_file::staging_path(&scratch.current);
        std::fs::write(&scratch.current, "committed").unwrap();
        std::fs::write(&staging, "half-writ").unwrap();
        assert_eq!(scratch.read().unwrap().unwrap().contents, "committed");
        assert!(!staging.exists());

        // A staging file alone is no spool.
        std::fs::remove_file(&scratch.current).unwrap();
        std::fs::write(&staging, "half-writ").unwrap();
        assert!(read_spool(&scratch.current, None).unwrap().is_none());
        assert!(!staging.exists());
    }

    /// Bytes the renderer never wrote are not handed over as text: the read fails as
    /// `error.recoveryRead` naming the file, without a panic, and the file is left for the
    /// operator to inspect or delete.
    #[test]
    fn a_spool_that_is_not_utf8_fails_as_a_read_error_naming_the_file() {
        let scratch = Scratch::new("utf8");
        std::fs::write(&scratch.current, [0xFF, 0xFE, b'{', 0xC3]).unwrap();
        std::fs::write(&scratch.legacy, "legacy").unwrap();
        let error = scratch.read().unwrap_err();
        assert_eq!(error.id, id::RECOVERY_READ);
        let detail = error.detail.unwrap();
        assert!(
            detail.starts_with(&scratch.current.display().to_string()),
            "{detail}"
        );
        // Not silently replaced by the legacy spool, and not deleted.
        assert!(scratch.current.exists());
    }
}
