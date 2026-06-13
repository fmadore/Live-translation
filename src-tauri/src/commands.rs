//! Tauri commands exposed to the front-end. Thin wrappers over `secrets`, `audio`, and the
//! `SessionManager`. Errors are surfaced to JS as strings.

use tauri::{AppHandle, Manager, State};

use crate::audio::list_input_devices;
use crate::secrets;
use crate::session::SessionManager;
use crate::types::{AudioDevice, StartOptions};

const OVERLAY_LABEL: &str = "overlay";

#[tauri::command]
pub fn list_microphones() -> Vec<AudioDevice> {
    list_input_devices()
}

#[tauri::command]
pub fn has_api_key() -> bool {
    secrets::has_api_key()
}

#[tauri::command]
pub fn set_api_key(key: String) -> Result<(), String> {
    secrets::set_api_key(&key).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn clear_api_key() -> Result<(), String> {
    secrets::clear_api_key().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn start_session(
    app: AppHandle,
    manager: State<'_, SessionManager>,
    options: StartOptions,
) -> Result<(), String> {
    manager.start(&app, options).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn stop_session(app: AppHandle, manager: State<'_, SessionManager>) {
    manager.stop(&app);
}

/// Toggle click-through on the caption overlay so it never steals clicks from PowerPoint.
#[tauri::command]
pub fn set_overlay_click_through(app: AppHandle, enabled: bool) -> Result<(), String> {
    if let Some(win) = app.get_webview_window(OVERLAY_LABEL) {
        win.set_ignore_cursor_events(enabled)
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn show_overlay(app: AppHandle, visible: bool) -> Result<(), String> {
    if let Some(win) = app.get_webview_window(OVERLAY_LABEL) {
        let r = if visible { win.show() } else { win.hide() };
        r.map_err(|e| e.to_string())?;
        if visible {
            let _ = win.set_always_on_top(true);
        }
    }
    Ok(())
}

/// Write the transcript to a `Live-translation` folder under the user's Documents directory
/// (falling back to Downloads, then the temp dir). `filename` is sanitized. Returns the path.
#[tauri::command]
pub fn save_transcript(app: AppHandle, content: String, filename: String) -> Result<String, String> {
    let dir = app
        .path()
        .document_dir()
        .or_else(|_| app.path().download_dir())
        .unwrap_or_else(|_| std::env::temp_dir())
        .join("Live-translation");

    std::fs::create_dir_all(&dir).map_err(|e| format!("could not create {dir:?}: {e}"))?;

    let safe: String = filename
        .chars()
        .map(|c| if c.is_alphanumeric() || matches!(c, '-' | '_' | '.') { c } else { '-' })
        .collect();
    let safe = if safe.trim_matches(|c| c == '-' || c == '.').is_empty() {
        "transcript.md".to_string()
    } else {
        safe
    };

    let path = dir.join(safe);
    std::fs::write(&path, content).map_err(|e| format!("could not write {path:?}: {e}"))?;
    Ok(path.to_string_lossy().into_owned())
}
