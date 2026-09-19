//! Opt-in session history, separate from the disposable recovery spool.
use crate::recovery::{remove_snapshot, replace_snapshot, StoredRecovery};
use std::{
    io::Write,
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

fn rename_session(path: &Path, title: &str) -> Result<(), String> {
    let raw = std::fs::read(path).map_err(|e| e.to_string())?;
    let mut session: serde_json::Value = serde_json::from_slice(&raw).map_err(|e| e.to_string())?;
    let record = session.as_object_mut().ok_or("invalid session")?;
    record.insert(
        "title".into(),
        title.trim().chars().take(120).collect::<String>().into(),
    );
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
