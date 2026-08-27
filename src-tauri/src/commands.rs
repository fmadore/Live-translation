//! Tauri commands exposed to the front-end. Thin wrappers over `secrets`, `audio`, and the
//! `SessionManager`. Errors are surfaced to JS as an `AppError` — an id the interface
//! translates plus the untranslated technical detail; see `errors.rs`.
//!
//! All commands are `async`; blocking keychain, device, filesystem, and thread-join work is
//! explicitly delegated to Tauri's blocking pool.

use tauri::{AppHandle, Manager, State};

use crate::audio::list_input_devices;
use crate::errors::{id, AppError};
use crate::ondevice::{self, OnDeviceReadiness};
use crate::overlay::{self, OVERLAY_LABEL};
use crate::secrets;
use crate::session::SessionManager;
use crate::textscale;
use crate::types::{AudioDevice, AudioSource, Provider, StartOptions};

#[tauri::command]
pub async fn list_microphones() -> Result<Vec<AudioDevice>, AppError> {
    tauri::async_runtime::spawn_blocking(list_input_devices)
        .await
        .map_err(|error| AppError::with(id::DEVICE_ENUMERATION, error))
}

#[tauri::command]
pub async fn has_api_key(provider: Provider) -> Result<bool, AppError> {
    tauri::async_runtime::spawn_blocking(move || secrets::has_api_key(provider))
        .await
        .map_err(|error| AppError::with(id::KEYCHAIN, error))
}

#[tauri::command]
pub async fn set_api_key(provider: Provider, key: String) -> Result<(), AppError> {
    tauri::async_runtime::spawn_blocking(move || secrets::set_api_key(provider, &key))
        .await
        .map_err(|error| AppError::with(id::KEYCHAIN, error))?
        .map_err(|error| AppError::with(id::KEYCHAIN, error))
}

#[tauri::command]
pub async fn clear_api_key(provider: Provider) -> Result<(), AppError> {
    tauri::async_runtime::spawn_blocking(move || secrets::clear_api_key(provider))
        .await
        .map_err(|error| AppError::with(id::KEYCHAIN, error))?
        .map_err(|error| AppError::with(id::KEYCHAIN, error))
}

#[tauri::command]
pub async fn ondevice_readiness(app: AppHandle) -> Result<OnDeviceReadiness, AppError> {
    tauri::async_runtime::spawn_blocking(move || ondevice::readiness(&app))
        .await
        .map_err(|error| AppError::with(id::DEMO_UNAVAILABLE, error))
}

#[tauri::command]
pub async fn prepare_ondevice_model(app: AppHandle) -> Result<OnDeviceReadiness, AppError> {
    tauri::async_runtime::spawn_blocking(move || {
        ondevice::prepare().map_err(|error| AppError::with(id::DEMO_UNAVAILABLE, error))?;
        Ok(ondevice::readiness(&app))
    })
    .await
    .map_err(|error| AppError::with(id::DEMO_UNAVAILABLE, error))?
}

#[tauri::command]
pub async fn start_session(
    app: AppHandle,
    manager: State<'_, SessionManager>,
    options: StartOptions,
) -> Result<(), AppError> {
    manager
        .start(&app, options)
        .await
        .map_err(|error| AppError::with(id::SESSION_START, format!("{error:#}")))
}

#[tauri::command]
pub async fn stop_session(
    app: AppHandle,
    manager: State<'_, SessionManager>,
) -> Result<(), AppError> {
    manager.stop(&app).await;
    Ok(())
}

/// Start level-only capture from the preflight so the operator can confirm a source is
/// actually producing sound. Opens no provider connection and keeps no audio; see
/// `SessionManager::start_test`.
#[tauri::command]
pub async fn start_audio_test(
    app: AppHandle,
    manager: State<'_, SessionManager>,
    source: AudioSource,
    mic_device_name: Option<String>,
) -> Result<(), AppError> {
    manager
        .start_test(&app, source, mic_device_name)
        .await
        .map_err(|error| AppError::with(id::AUDIO_TEST_START, format!("{error:#}")))
}

#[tauri::command]
pub async fn stop_audio_test(
    app: AppHandle,
    manager: State<'_, SessionManager>,
) -> Result<(), AppError> {
    manager.stop_test(&app).await;
    Ok(())
}

/// Toggle click-through on the caption overlay. Enabled while captioning so the overlay
/// never steals clicks from the slides; disabled by "Move overlay" in the operator window
/// so the overlay can be dragged and resized into place.
#[tauri::command]
pub async fn set_overlay_click_through(app: AppHandle, enabled: bool) -> Result<(), AppError> {
    if let Some(win) = app.get_webview_window(OVERLAY_LABEL) {
        win.set_ignore_cursor_events(enabled)
            .map_err(|error| AppError::with(id::OVERLAY_WINDOW, error))?;
        // Click-through and no-activate go together: while captioning the overlay must never
        // take focus, but in move mode it has to in order to be dragged.
        overlay::set_no_activate(&win, enabled);
    }
    Ok(())
}

#[tauri::command]
pub async fn show_overlay(app: AppHandle, visible: bool) -> Result<(), AppError> {
    if let Some(win) = app.get_webview_window(OVERLAY_LABEL) {
        let r = if visible { win.show() } else { win.hide() };
        r.map_err(|error| AppError::with(id::OVERLAY_WINDOW, error))?;
        if visible {
            overlay::raise(&win);
        }
    }
    Ok(())
}

/// The operator's Windows text-scale factor, asked for once as the window boots.
///
/// A command rather than only an event because the window has to lay itself out before it
/// could possibly have subscribed to anything, and an operator who needs 225% text should
/// never see one frame of 9.5px type. Changes after boot arrive on `events::TEXT_SCALE`.
#[tauri::command]
pub async fn text_scale_factor() -> f64 {
    textscale::current()
}

/// Write the transcript to a `Live-translation` folder under the user's Documents directory
/// (falling back to Downloads, then the temp dir). `filename` is sanitized. Returns the path.
#[tauri::command]
pub async fn save_transcript(
    app: AppHandle,
    content: String,
    filename: String,
) -> Result<String, AppError> {
    let dir = app
        .path()
        .document_dir()
        .or_else(|_| app.path().download_dir())
        .unwrap_or_else(|_| std::env::temp_dir())
        .join("Live-translation");

    tauri::async_runtime::spawn_blocking(move || {
        std::fs::create_dir_all(&dir)
            .map_err(|e| AppError::with(id::TRANSCRIPT_DIR, format!("{} — {e}", dir.display())))?;

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
        std::fs::write(&path, content).map_err(|e| {
            AppError::with(id::TRANSCRIPT_WRITE, format!("{} — {e}", path.display()))
        })?;
        Ok(path.to_string_lossy().into_owned())
    })
    .await
    .map_err(|error| AppError::with(id::TASK_FAILED, error))?
}
