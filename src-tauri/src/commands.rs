//! Tauri commands exposed to the front-end. Thin wrappers over `secrets`, `audio`, and the
//! `SessionManager`. Errors are surfaced to JS as strings.
//!
//! All commands are `async`; blocking keychain, device, filesystem, and thread-join work is
//! explicitly delegated to Tauri's blocking pool.

use tauri::{AppHandle, Manager, State};

use crate::audio::list_input_devices;
use crate::ondevice::{self, OnDeviceReadiness};
use crate::overlay::{self, OVERLAY_LABEL};
use crate::secrets;
use crate::session::SessionManager;
use crate::types::{AudioDevice, Provider, StartOptions};

#[tauri::command]
pub async fn list_microphones() -> Result<Vec<AudioDevice>, String> {
    tauri::async_runtime::spawn_blocking(list_input_devices)
        .await
        .map_err(|error| format!("microphone enumeration task failed: {error}"))
}

#[tauri::command]
pub async fn has_api_key(provider: Provider) -> Result<bool, String> {
    tauri::async_runtime::spawn_blocking(move || secrets::has_api_key(provider))
        .await
        .map_err(|error| format!("keychain task failed: {error}"))
}

#[tauri::command]
pub async fn set_api_key(provider: Provider, key: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || secrets::set_api_key(provider, &key))
        .await
        .map_err(|error| format!("keychain task failed: {error}"))?
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn clear_api_key(provider: Provider) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || secrets::clear_api_key(provider))
        .await
        .map_err(|error| format!("keychain task failed: {error}"))?
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn ondevice_readiness(app: AppHandle) -> Result<OnDeviceReadiness, String> {
    tauri::async_runtime::spawn_blocking(move || ondevice::readiness(&app))
        .await
        .map_err(|error| format!("local speech readiness check failed: {error}"))
}

#[tauri::command]
pub async fn prepare_ondevice_model(app: AppHandle) -> Result<OnDeviceReadiness, String> {
    tauri::async_runtime::spawn_blocking(move || {
        ondevice::prepare().map_err(|error| error.to_string())?;
        Ok(ondevice::readiness(&app))
    })
    .await
    .map_err(|error| format!("local speech setup task failed: {error}"))?
}

#[tauri::command]
pub async fn start_session(
    app: AppHandle,
    manager: State<'_, SessionManager>,
    options: StartOptions,
) -> Result<(), String> {
    manager
        .start(&app, options)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn stop_session(
    app: AppHandle,
    manager: State<'_, SessionManager>,
) -> Result<(), String> {
    manager.stop(&app).await;
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
        // Click-through and no-activate go together: while captioning the overlay must never
        // take focus, but in move mode it has to in order to be dragged.
        overlay::set_no_activate(&win, enabled);
    }
    Ok(())
}

#[tauri::command]
pub async fn show_overlay(app: AppHandle, visible: bool) -> Result<(), String> {
    if let Some(win) = app.get_webview_window(OVERLAY_LABEL) {
        let r = if visible { win.show() } else { win.hide() };
        r.map_err(|e| e.to_string())?;
        if visible {
            overlay::raise(&win);
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

    tauri::async_runtime::spawn_blocking(move || {
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
    })
    .await
    .map_err(|error| format!("transcript write task failed: {error}"))?
}
