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

/// The preference file in the app's data folder, if it has one.
#[cfg(any(windows, target_os = "macos"))]
fn directory_preference(app: &tauri::AppHandle) -> Option<std::path::PathBuf> {
    use tauri::Manager;
    app.path()
        .app_local_data_dir()
        .ok()
        .map(|p| p.join("export-directory.json"))
}

/// Where a Save or folder dialog opens: the remembered folder while it exists, else Documents.
#[cfg(any(windows, target_os = "macos"))]
fn initial_directory(
    app: &tauri::AppHandle,
    preference: Option<&Path>,
) -> Option<std::path::PathBuf> {
    use tauri::Manager;
    preference
        .and_then(remembered_directory)
        .or_else(|| app.path().document_dir().ok())
}

/// Remember `dir` for the next dialog, written whole or not at all. A failure is ignored: a
/// preference must not undo the export that has just succeeded.
#[cfg(any(windows, target_os = "macos", test))]
fn remember_directory(preference: &Path, dir: &Path) {
    if let Some(parent) = preference.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(bytes) = serde_json::to_vec(dir) {
        let _ = atomic_write(preference, &bytes);
    }
}

#[cfg(any(windows, target_os = "macos"))]
pub fn save(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    content: String,
    filename: String,
) -> Result<Option<String>, crate::errors::AppError> {
    use crate::errors::{AppError, id};
    // A dedicated thread keeps the dialog's STA isolated from audio workers' COM state. On
    // macOS rfd runs the panel on the main thread and waits for it here.
    std::thread::spawn(move || {
        let Some((label, extension)) = format_filter(&filename) else {
            return Err(AppError::with(
                id::TRANSCRIPT_WRITE,
                "unsupported transcript format",
            ));
        };
        let preference = directory_preference(&app);
        let mut dialog = rfd::FileDialog::new()
            .set_parent(&window)
            .set_file_name(&filename)
            .add_filter(label, &[extension]);
        if let Some(dir) = initial_directory(&app, preference.as_deref()) {
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
        // Remember only successful saves.
        if let (Some(pref), Some(parent)) = (preference, path.parent()) {
            remember_directory(&pref, parent);
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

/// One transcript of a batch export: its suggested file name and its formatted text.
#[derive(Debug, serde::Deserialize)]
#[cfg_attr(not(any(windows, target_os = "macos")), allow(dead_code))]
pub struct ExportFile {
    pub filename: String,
    pub content: String,
}

/// What a batch export did: the folder, the names the files were written under, and the files
/// it could not write.
#[derive(Debug, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BatchExport {
    pub directory: String,
    pub written: Vec<String>,
    pub failed: Vec<crate::errors::ItemFailure>,
}

/// A name the renderer may ask for: a bare file name, not hidden, in one of the four formats.
/// Anything with a separator or a `..` would be a path, and is refused.
#[cfg(any(windows, target_os = "macos", test))]
fn plain_name(filename: &str) -> bool {
    !filename.starts_with('.')
        && !filename.contains(['/', '\\', ':'])
        && Path::new(filename).file_name().and_then(|n| n.to_str()) == Some(filename)
        && format_filter(filename).is_some()
}

/// A name in `dir` that neither an existing file nor an earlier file of this batch has:
/// `name.ext`, then `name (2).ext`, `name (3).ext`… A batch never overwrites the operator's
/// files, since there is no dialog to confirm it file by file. Names are compared without case,
/// as Windows and macOS compare them; `taken` starts as the folder's own names, lowercased.
#[cfg(any(windows, target_os = "macos", test))]
fn free_name(
    dir: &Path,
    filename: &str,
    taken: &mut std::collections::HashSet<String>,
) -> std::path::PathBuf {
    let path = Path::new(filename);
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(filename);
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or_default();
    let mut candidate = filename.to_owned();
    let mut n = 1;
    while taken.contains(&candidate.to_lowercase()) || dir.join(&candidate).exists() {
        n += 1;
        candidate = format!("{stem} ({n}).{extension}");
    }
    taken.insert(candidate.to_lowercase());
    dir.join(candidate)
}

/// Write every file into `dir`, each under a free name. A file that cannot be written is
/// reported and the others still are. `write` is `write_export`, a parameter for the tests.
#[cfg(any(windows, target_os = "macos", test))]
fn write_files(
    dir: &Path,
    files: Vec<ExportFile>,
    write: impl Fn(&Path, &[u8]) -> io::Result<()>,
) -> BatchExport {
    use crate::errors::{AppError, ItemFailure, id};
    // What the folder already holds, without case, so a name differing only in case from an
    // existing file is never used: the same on every file system, case-sensitive or not.
    let mut taken: std::collections::HashSet<String> = std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .map(|entry| entry.file_name().to_string_lossy().to_lowercase())
                .collect()
        })
        .unwrap_or_default();
    let mut outcome = BatchExport {
        directory: dir.to_string_lossy().into_owned(),
        written: vec![],
        failed: vec![],
    };
    for file in files {
        if !plain_name(&file.filename) {
            outcome.failed.push(ItemFailure {
                item: file.filename,
                error: AppError::with(id::TRANSCRIPT_WRITE, "unsupported transcript file name"),
            });
            continue;
        }
        let path = free_name(dir, &file.filename, &mut taken);
        match write(&path, file.content.as_bytes()) {
            Ok(()) => outcome.written.push(
                path.file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default(),
            ),
            Err(e) => outcome.failed.push(ItemFailure {
                item: file.filename,
                error: AppError::with(id::TRANSCRIPT_WRITE, format!("{} — {e}", path.display())),
            }),
        }
    }
    outcome
}

/// Ask for a folder, then write every transcript into it. `None` when the operator cancels.
#[cfg(any(windows, target_os = "macos"))]
pub fn save_many(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    files: Vec<ExportFile>,
) -> Result<Option<BatchExport>, crate::errors::AppError> {
    use crate::errors::{AppError, id};
    // A dedicated thread, as for one file: see `save`.
    std::thread::spawn(move || {
        let preference = directory_preference(&app);
        let mut dialog = rfd::FileDialog::new().set_parent(&window);
        if let Some(dir) = initial_directory(&app, preference.as_deref()) {
            dialog = dialog.set_directory(dir);
        }
        let Some(dir) = dialog.pick_folder() else {
            return Ok(None);
        };
        let outcome = write_files(&dir, files, write_export);
        // Remembered once something was written there, like a single save.
        if let (Some(pref), false) = (preference, outcome.written.is_empty()) {
            remember_directory(&pref, &dir);
        }
        Ok(Some(outcome))
    })
    .join()
    .map_err(|_| AppError::with(id::TASK_FAILED, "folder dialog thread panicked"))?
}

#[cfg(not(any(windows, target_os = "macos")))]
pub fn save_many(
    _: tauri::AppHandle,
    _: tauri::WebviewWindow,
    _: Vec<ExportFile>,
) -> Result<Option<BatchExport>, crate::errors::AppError> {
    Err(crate::errors::AppError::with(
        crate::errors::id::TRANSCRIPT_WRITE,
        "native transcript export requires Windows or macOS",
    ))
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
    fn a_batch_export_takes_bare_names_only() {
        assert!(plain_name("transcript-20261117-101500.srt"));
        assert!(plain_name("Réunion (2).md"));
        for refused in [
            "../escape.txt",
            "sub/file.txt",
            "sub\\file.txt",
            "C:file.txt",
            ".hidden.txt",
            "captions.json",
            "",
        ] {
            assert!(!plain_name(refused), "{refused}");
        }
    }

    /// Never overwrites a file already there, nor an earlier file of the same batch, and one
    /// file that fails does not stop the others.
    #[test]
    fn a_batch_export_keeps_existing_files_and_reports_each_failure() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.txt"), "theirs").unwrap();
        let file = |filename: &str, content: &str| ExportFile {
            filename: filename.into(),
            content: content.into(),
        };
        let files = vec![
            file("a.txt", "one"),
            file("A.txt", "two"),
            file("../b.txt", "refused"),
            file("fail.srt", "locked"),
            file("c.md", "three"),
        ];
        let outcome = write_files(dir.path(), files, |path, content| {
            if path.ends_with("fail.srt") {
                Err(io::Error::other("locked"))
            } else {
                atomic_write(path, content)
            }
        });
        assert_eq!(outcome.written, ["a (2).txt", "A (3).txt", "c.md"]);
        assert_eq!(
            outcome
                .failed
                .iter()
                .map(|f| f.item.as_str())
                .collect::<Vec<_>>(),
            ["../b.txt", "fail.srt"]
        );
        let read = |name: &str| std::fs::read_to_string(dir.path().join(name)).unwrap();
        assert_eq!(read("a.txt"), "theirs");
        assert_eq!(read("a (2).txt"), "one");
        assert_eq!(read("A (3).txt"), "two");
        assert_eq!(read("c.md"), "three");
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

    #[test]
    fn a_remembered_folder_is_read_back_and_replaces_the_last_one() {
        let dir = tempfile::tempdir().unwrap();
        let (first, second) = (dir.path().join("first"), dir.path().join("second"));
        std::fs::create_dir_all(&first).unwrap();
        std::fs::create_dir_all(&second).unwrap();
        // The data folder may not exist yet on a first export.
        let preference = dir.path().join("data").join("export-directory.json");
        remember_directory(&preference, &first);
        assert_eq!(remembered_directory(&preference), Some(first));
        remember_directory(&preference, &second);
        assert_eq!(remembered_directory(&preference), Some(second));
        // Replaced in place: no staging file is left beside it.
        let names: Vec<_> = std::fs::read_dir(preference.parent().unwrap())
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect();
        assert_eq!(names, ["export-directory.json"]);
        // A folder that cannot hold the file leaves no preference and no error.
        let blocked = dir.path().join("blocked");
        std::fs::write(&blocked, b"a file, not a folder").unwrap();
        let unwritable = blocked.join("export-directory.json");
        remember_directory(&unwritable, dir.path());
        assert_eq!(remembered_directory(&unwritable), None);
    }
}
