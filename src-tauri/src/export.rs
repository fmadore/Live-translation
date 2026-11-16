//! Native Save As and atomic transcript writes. Only the operator may invoke this.
#[cfg(any(windows, target_os = "macos", test))]
use std::io;
#[cfg(any(windows, target_os = "macos", test))]
use std::path::Path;
#[cfg(any(windows, target_os = "macos", test))]
use std::sync::atomic::{AtomicU64, Ordering};

#[cfg(any(windows, target_os = "macos", test))]
static NEXT_FILE: AtomicU64 = AtomicU64::new(0);

/// `atomic_file::replace_via`, staged under a hidden name of this process's own. The folder is
/// the operator's, so a name derived from the target's could already belong to one of their
/// files; this one cannot, and would be refused rather than truncated if it somehow existed.
#[cfg(any(windows, target_os = "macos", test))]
fn atomic_write(path: &Path, content: &[u8]) -> io::Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| io::Error::other("missing parent directory"))?;
    let staging = parent.join(format!(
        ".live-translation-{}-{}.tmp",
        std::process::id(),
        NEXT_FILE.fetch_add(1, Ordering::Relaxed)
    ));
    crate::atomic_file::replace_via(path, &staging, content)
}

/// The Save dialog's filter for a suggested file name: the format's name and its extension.
/// `None` for an extension the renderer never asks for.
#[cfg(any(windows, target_os = "macos", test))]
fn format_filter(filename: &str) -> Option<(&'static str, &str)> {
    let extension = Path::new(filename).extension()?.to_str()?;
    let label = match extension {
        "md" => "Markdown",
        "txt" => "TXT",
        "vtt" => "WebVTT",
        "srt" => "SubRip",
        _ => return None,
    };
    Some((label, extension))
}

/// The folder the last successful export went to, from the preference file, when it still
/// exists. A missing, unreadable or malformed preference is no preference.
#[cfg(any(windows, target_os = "macos", test))]
fn remembered_directory(preference: &Path) -> Option<std::path::PathBuf> {
    let bytes = std::fs::read(preference).ok()?;
    serde_json::from_slice::<std::path::PathBuf>(&bytes)
        .ok()
        .filter(|dir| dir.is_dir())
}

#[cfg(any(windows, target_os = "macos"))]
pub fn save(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    content: String,
    filename: String,
) -> Result<Option<String>, crate::errors::AppError> {
    use crate::errors::{AppError, id};
    use tauri::Manager;
    // A dedicated thread keeps the dialog's STA isolated from audio workers' COM state. On
    // macOS rfd runs the panel on the main thread and waits for it here.
    std::thread::spawn(move || {
        let Some((label, extension)) = format_filter(&filename) else {
            return Err(AppError::with(
                id::TRANSCRIPT_WRITE,
                "unsupported transcript format",
            ));
        };
        let preference = app
            .path()
            .app_local_data_dir()
            .ok()
            .map(|p| p.join("export-directory.json"));
        let remembered = preference.as_deref().and_then(remembered_directory);
        let mut dialog = rfd::FileDialog::new()
            .set_parent(&window)
            .set_file_name(&filename)
            .add_filter(label, &[extension]);
        if let Some(dir) = remembered.or_else(|| app.path().document_dir().ok()) {
            dialog = dialog.set_directory(dir);
        }
        // Windows' IFileSaveDialog and macOS's NSSavePanel supply the normal overwrite
        // confirmation.
        let Some(path) = dialog.save_file() else {
            return Ok(None);
        };
        write_export(&path, content.as_bytes()).map_err(|e| {
            AppError::with(id::TRANSCRIPT_WRITE, format!("{} — {e}", path.display()))
        })?;
        // Remember only successful saves. A preference failure must not undo a completed export.
        if let (Some(pref), Some(parent)) = (preference, path.parent()) {
            if let Some(dir) = pref.parent() {
                let _ = std::fs::create_dir_all(dir);
            }
            if let Ok(bytes) = serde_json::to_vec(parent) {
                let _ = atomic_write(&pref, &bytes);
            }
        }
        Ok(Some(path.to_string_lossy().into_owned()))
    })
    .join()
    .map_err(|_| {
        crate::errors::AppError::with(
            crate::errors::id::TASK_FAILED,
            "save dialog thread panicked",
        )
    })?
}

/// Write the chosen file. Atomic everywhere it can be: see [`atomic_write`].
///
/// In the macOS App Sandbox the Save panel grants the chosen file and nothing beside it, so
/// the staging file cannot be created in its folder. Only then is the file written in place,
/// which leaves a partly written transcript if the disk fails mid-write, but never one from
/// another session. Outside the sandbox nothing changes.
#[cfg(any(windows, target_os = "macos"))]
fn write_export(path: &Path, content: &[u8]) -> io::Result<()> {
    match atomic_write(path, content) {
        #[cfg(target_os = "macos")]
        Err(error) if error.kind() == io::ErrorKind::PermissionDenied => {
            tracing::warn!("no access beside the export (App Sandbox); writing it in place");
            std::fs::write(path, content)
        }
        other => other,
    }
}

#[cfg(not(any(windows, target_os = "macos")))]
pub fn save(
    _: tauri::AppHandle,
    _: tauri::WebviewWindow,
    _: String,
    _: String,
) -> Result<Option<String>, crate::errors::AppError> {
    Err(crate::errors::AppError::with(
        crate::errors::id::TRANSCRIPT_WRITE,
        "native transcript export requires Windows or macOS",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn replaces_complete_file_and_cleans_up_after_failed_replacement() {
        let dir = std::env::temp_dir().join(format!("export-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("captions.txt");
        atomic_write(&path, b"old").unwrap();
        atomic_write(&path, "Français — new".as_bytes()).unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "Français — new");
        assert!(atomic_write(&dir, b"cannot replace a directory").is_err());
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 1);
        std::fs::remove_file(path).unwrap();
        std::fs::remove_dir(dir).unwrap();
    }

    #[test]
    fn the_dialog_filter_follows_the_four_transcript_formats_only() {
        assert_eq!(format_filter("captions.md"), Some(("Markdown", "md")));
        assert_eq!(format_filter("captions.txt"), Some(("TXT", "txt")));
        assert_eq!(
            format_filter("Réunion 2026-11-16.vtt"),
            Some(("WebVTT", "vtt"))
        );
        assert_eq!(format_filter("a.b.srt"), Some(("SubRip", "srt")));
        for refused in [
            "captions",
            "captions.",
            "captions.json",
            "captions.SRT",
            ".srt",
        ] {
            assert_eq!(format_filter(refused), None, "{refused}");
        }
    }

    /// Staged under a hidden name of the process's own, never one derived from the target's,
    /// and a taken staging name is refused rather than truncated.
    #[test]
    fn a_taken_staging_name_is_refused_not_truncated() {
        let dir = std::env::temp_dir().join(format!("export-staging-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let next = NEXT_FILE.load(Ordering::Relaxed);
        // Whatever the counter says next is already taken by a file of the operator's.
        let taken = dir.join(format!(
            ".live-translation-{}-{next}.tmp",
            std::process::id()
        ));
        std::fs::write(&taken, "theirs").unwrap();
        let path = dir.join("captions.srt");
        let first = atomic_write(&path, b"one");
        // Another test may have taken that number first; either way the operator's file stands.
        assert_eq!(std::fs::read_to_string(&taken).unwrap(), "theirs");
        if first.is_err() {
            atomic_write(&path, b"one").unwrap();
        }
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "one");
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn the_remembered_folder_is_used_only_while_it_exists() {
        let dir = std::env::temp_dir().join(format!("export-pref-{}", std::process::id()));
        let chosen = dir.join("chosen");
        std::fs::create_dir_all(&chosen).unwrap();
        let preference = dir.join("export-directory.json");
        assert_eq!(remembered_directory(&preference), None);
        std::fs::write(&preference, serde_json::to_vec(&chosen).unwrap()).unwrap();
        assert_eq!(remembered_directory(&preference), Some(chosen.clone()));
        std::fs::remove_dir(&chosen).unwrap();
        assert_eq!(remembered_directory(&preference), None);
        std::fs::write(&preference, b"not json").unwrap();
        assert_eq!(remembered_directory(&preference), None);
        std::fs::remove_dir_all(dir).unwrap();
    }
}
