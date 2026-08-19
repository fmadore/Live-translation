//! Live Captions — Tauri desktop core for translation and transcription.
//!
//! Captures microphone and system-loopback audio, streams it to Google Gemini 3.5 Live
//! Translate over a WebSocket, and renders the returned translation as live captions in a
//! transparent, always-on-top overlay window.

mod audio;
mod commands;
mod gemini;
mod mistral;
mod ondevice;
mod openai;
mod overlay;
mod realtime;
mod secrets;
mod session;
mod types;

use tauri::Manager;

use overlay::OVERLAY_LABEL;
use session::SessionManager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Load .env for development (GEMINI_API_KEY etc.); ignored if absent.
    let _ = dotenvy::dotenv();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "live_translation_lib=info,warn".into()),
        )
        .init();

    tauri::Builder::default()
        .manage(SessionManager::default())
        .invoke_handler(tauri::generate_handler![
            commands::list_microphones,
            commands::has_api_key,
            commands::set_api_key,
            commands::clear_api_key,
            commands::start_session,
            commands::stop_session,
            commands::set_overlay_click_through,
            commands::show_overlay,
            commands::save_transcript,
        ])
        .setup(|app| {
            // Make the overlay click-through from the start so it floats over slides
            // without ever stealing a click. Toggle via `set_overlay_click_through`.
            if let Some(window) = app.get_webview_window(OVERLAY_LABEL) {
                let _ = window.set_ignore_cursor_events(true);
                overlay::set_no_activate(&window, true);
                overlay::raise(&window);
            }
            // Other topmost windows (Zoom, Teams, a slideshow) jump above the overlay when
            // they are activated, so keep re-raising it for as long as the app runs.
            overlay::spawn_topmost_keeper(app.handle());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running the Live Captions app");
}
