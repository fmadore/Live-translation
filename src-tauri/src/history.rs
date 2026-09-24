//! Opt-in session history, separate from the disposable recovery spool.
//!
//! A session file is a log: one JSON record per line, a header first and then finalized lines
//! and progress records as they happen (the format is `src/lib/history.ts`'s, and opaque here).
//! `write_history` replaces a whole file; `append_history` adds records to one, so a long
//! session no longer rewrites everything it has already saved every few seconds. Files from
//! before the log format are a single JSON object, and are still read and renamed.
use crate::recovery::{remove_snapshot, replace_snapshot, StoredRecovery};
use std::{
    io::{Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    sync::Mutex,
};
use tauri::{AppHandle, Manager};

static HISTORY_IO: Mutex<()> = Mutex::new(());

fn session_path(dir: &Path, id: &str) -> Result<PathBuf, String> {
    // UUIDs only; renderer input must never become an arbitrary filesystem path.
    if id.len() != 36
        || !id.bytes().enumerate().all(|(i, b)| {
            if [8, 13, 18, 23].contains(&i) {
                b == b'-'
            } else {
                b.is_ascii_hexdigit()
            }
        })
    {
        return Err("invalid session id".into());
    }
    Ok(dir.join(format!("{id}.json")))
}

fn directory(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_local_data_dir()
        .map(|p| p.join("history"))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn write_history(app: AppHandle, id: String, contents: String) -> Result<(), String> {
    let dir = directory(&app)?;
    let path = session_path(&dir, &id)?;
    tauri::async_runtime::spawn_blocking(move || {
        let _lock = HISTORY_IO.lock().unwrap_or_else(|p| p.into_inner());
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        replace_snapshot(&path, |f| f.write_all(contents.as_bytes())).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Add records to an existing session log and flush them. A missing file is an error rather
/// than a new one, so a log never starts without its header: the renderer answers the error
/// by writing the whole session. A record torn by an earlier failed write is closed off with
/// a newline first, so it cannot swallow the next one.
fn append_session(path: &Path, contents: &str) -> std::io::Result<()> {
    let mut file = std::fs::OpenOptions::new()
        .read(true)
        .append(true)
        .open(path)?;
    let mut last = [0u8; 1];
    if file.seek(SeekFrom::End(-1)).is_ok() && file.read_exact(&mut last).is_ok() && last != *b"\n"
    {
        file.write_all(b"\n")?;
    }
    file.write_all(contents.as_bytes())?;
    file.sync_data()
}

#[tauri::command]
pub async fn append_history(app: AppHandle, id: String, contents: String) -> Result<(), String> {
    let path = session_path(&directory(&app)?, &id)?;
    tauri::async_runtime::spawn_blocking(move || {
        let _lock = HISTORY_IO.lock().unwrap_or_else(|p| p.into_inner());
        append_session(&path, &contents).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

fn read_sessions(dir: &Path) -> Result<Vec<StoredRecovery>, String> {
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(vec![]),
        Err(e) => return Err(e.to_string()),
    };
    let mut sessions = vec![];
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.extension().is_none_or(|e| e != "json") {
            continue;
        }
        let id = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        if session_path(dir, id).is_err()
            || !entry.file_type().map_err(|e| e.to_string())?.is_file()
        {
            continue;
        }
        sessions.push(StoredRecovery {
            path: id.to_string(),
            contents: String::from_utf8_lossy(&std::fs::read(path).map_err(|e| e.to_string())?)
                .into_owned(),
        });
    }
    Ok(sessions)
}

/// The log format's version, as its header records it.
const LOG_VERSION: u64 = 2;

/// Whether a session file is a log, judged by its header record.
fn is_log(raw: &[u8]) -> bool {
    let header = raw.split(|&b| b == b'\n').next().unwrap_or_default();
    serde_json::from_slice::<serde_json::Value>(header)
        .is_ok_and(|record| record["version"] == LOG_VERSION)
}

fn rename_session(path: &Path, title: &str) -> Result<(), String> {
    let raw = std::fs::read(path).map_err(|e| e.to_string())?;
    let title = title.trim().chars().take(120).collect::<String>();
    // A log takes the rename as one more record; the latest title record wins when it is read.
    if is_log(&raw) {
        let mut record = serde_json::to_string(&serde_json::json!({ "title": title }))
            .map_err(|e| e.to_string())?;
        record.push('\n');
        return append_session(path, &record).map_err(|e| e.to_string());
    }
    let mut session: serde_json::Value = serde_json::from_slice(&raw).map_err(|e| e.to_string())?;
    let record = session.as_object_mut().ok_or("invalid session")?;
    record.insert("title".into(), title.into());
    let contents = serde_json::to_vec(&session).map_err(|e| e.to_string())?;
    replace_snapshot(path, |f| f.write_all(&contents)).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn rename_history(app: AppHandle, id: String, title: String) -> Result<(), String> {
    let path = session_path(&directory(&app)?, &id)?;
    tauri::async_runtime::spawn_blocking(move || {
        let _lock = HISTORY_IO.lock().unwrap_or_else(|p| p.into_inner());
        rename_session(&path, &title)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn list_history(app: AppHandle) -> Result<Vec<StoredRecovery>, String> {
    let dir = directory(&app)?;
    tauri::async_runtime::spawn_blocking(move || {
        let _lock = HISTORY_IO.lock().unwrap_or_else(|p| p.into_inner());
        read_sessions(&dir)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn delete_history(app: AppHandle, id: String) -> Result<(), String> {
    let path = session_path(&directory(&app)?, &id)?;
    tauri::async_runtime::spawn_blocking(move || {
        let _lock = HISTORY_IO.lock().unwrap_or_else(|p| p.into_inner());
        remove_snapshot(&path).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rename_preserves_latest_disk_contents_and_does_not_recreate_missing_sessions() {
        let dir = std::env::temp_dir().join(format!(
            "history-rename-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("session.json");
        let raw = br#"{"lines":[{"text":"newest caption"}],"futureField":42}"#;
        std::fs::write(&path, raw).unwrap();
        rename_session(&path, " Meeting ").unwrap();
        let record: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        assert_eq!(record["title"], "Meeting");
        assert_eq!(record["lines"][0]["text"], "newest caption");
        assert_eq!(record["futureField"], 42);
        std::fs::remove_file(&path).unwrap();
        assert!(rename_session(&path, "gone").is_err());
        std::fs::remove_dir(dir).unwrap();
    }
    fn temp_file(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "{name}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    #[test]
    fn appends_extend_a_log_and_never_create_one() {
        let path = temp_file("history-append");
        assert!(append_session(&path, "{\"line\":1}\n").is_err());
        assert!(!path.exists());
        std::fs::write(&path, "{\"version\":2}\n").unwrap();
        append_session(&path, "{\"line\":1}\n").unwrap();
        append_session(&path, "{\"line\":2}\n").unwrap();
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            "{\"version\":2}\n{\"line\":1}\n{\"line\":2}\n"
        );
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn a_torn_record_cannot_swallow_the_next_append() {
        let path = temp_file("history-torn");
        std::fs::write(&path, "{\"version\":2}\n{\"line\":").unwrap();
        append_session(&path, "{\"line\":2}\n").unwrap();
        let contents = std::fs::read_to_string(&path).unwrap();
        assert_eq!(contents.lines().last(), Some("{\"line\":2}"));
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn renaming_a_log_appends_a_title_record() {
        let path = temp_file("history-rename-log");
        std::fs::write(&path, "{\"version\":2,\"id\":\"x\"}\n{\"line\":{}}\n").unwrap();
        rename_session(&path, "  Keynote ").unwrap();
        let contents = std::fs::read_to_string(&path).unwrap();
        assert_eq!(contents.lines().count(), 3);
        assert_eq!(contents.lines().last(), Some("{\"title\":\"Keynote\"}"));
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn history_survives_reopening_and_deletion_is_per_session() {
        let dir = std::env::temp_dir().join(format!(
            "live-history-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        assert!(read_sessions(&dir).unwrap().is_empty());
        std::fs::create_dir(&dir).unwrap();
        let first = session_path(&dir, "12345678-1234-1234-1234-123456789abc").unwrap();
        let second = session_path(&dir, "12345678-1234-1234-1234-123456789abd").unwrap();
        replace_snapshot(&first, |f| f.write_all(b"complete session")).unwrap();
        replace_snapshot(&second, |f| f.write_all(b"second session")).unwrap();
        // Incomplete writes and unrelated files are never offered as saved sessions.
        std::fs::write(first.with_extension("json.pending"), "partial").unwrap();
        std::fs::write(dir.join("unrelated.json"), "unrelated").unwrap();
        assert_eq!(read_sessions(&dir).unwrap().len(), 2);
        // A corrupt record remains individually deletable without hiding the good one.
        std::fs::write(&second, [0xff, 0x00]).unwrap();
        let reopened = read_sessions(&dir).unwrap();
        assert_eq!(reopened.len(), 2);
        assert!(reopened.iter().any(|s| s.contents == "complete session"));
        remove_snapshot(&first).unwrap();
        remove_snapshot(&first).unwrap();
        assert_eq!(read_sessions(&dir).unwrap().len(), 1);
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn session_ids_cannot_escape_the_history_directory() {
        let dir = Path::new("history");
        for id in [
            "../transcript",
            "C:\\secrets",
            "",
            "../../../../../../../../../../../abc",
            "00000000-0000-0000-0000-00000000000/",
        ] {
            assert!(session_path(dir, id).is_err());
        }
        assert_eq!(
            session_path(dir, "12345678-1234-1234-1234-123456789abc").unwrap(),
            dir.join("12345678-1234-1234-1234-123456789abc.json")
        );
    }
}
