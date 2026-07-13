//! Tauri commands exposed to the front-end. Thin wrappers over `secrets`, `audio`, and the
//! `SessionManager`. Errors are surfaced to JS as strings.
//!
//! All commands are `async` so they run off the main thread: keychain access and
//! `stop_session` (which joins capture threads) can block for long enough to stutter the UI.

use tauri::{AppHandle, Manager, State};

use crate::audio::list_input_devices;
use crate::secrets;
use crate::session::SessionManager;
use crate::types::{AudioDevice, Provider, StartOptions};

const OVERLAY_LABEL: &str = "overlay";

#[tauri::command]
pub async fn list_microphones() -> Vec<AudioDevice> {
    list_input_devices()
}

#[tauri::command]
pub async fn has_api_key(provider: Provider) -> bool {
    secrets::has_api_key(provider)
}

#[tauri::command]
pub async fn set_api_key(provider: Provider, key: String) -> Result<(), String> {
    secrets::set_api_key(provider, &key).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn clear_api_key(provider: Provider) -> Result<(), String> {
    secrets::clear_api_key(provider).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn start_session(
    app: AppHandle,
    manager: State<'_, SessionManager>,
    options: StartOptions,
) -> Result<(), String> {
    manager.start(&app, options).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn stop_session(
    app: AppHandle,
    manager: State<'_, SessionManager>,
) -> Result<(), String> {
    manager.stop(&app);
    Ok(())
}

/// Toggle click-through on the caption overlay. Enabled while captioning so the overlay
/// never steals clicks from the slides; disabled by "Move overlay" in the operator window
/// so the overlay can be dragged and resized into place.
#[tauri::command]
pub async fn set_overlay_click_through(app: AppHandle, enabled: bool) -> Result<(), String> {
    if let Some(win) = app.get_webview_window(OVERLAY_LABEL) {
        win.set_ignore_cursor_events(enabled)
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub async fn show_overlay(app: AppHandle, visible: bool) -> Result<(), String> {
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
pub async fn save_transcript(
    app: AppHandle,
    content: String,
    filename: String,
) -> Result<String, String> {
    let dir = app
        .path()
        .document_dir()
        .or_else(|_| app.path().download_dir())
        .unwrap_or_else(|_| std::env::temp_dir())
        .join("Live-translation");

    std::fs::create_dir_all(&dir).map_err(|e| format!("could not create {dir:?}: {e}"))?;

    let safe: String = filename
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || matches!(c, '-' | '_' | '.') {
                c
            } else {
                '-'
            }
        })
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
