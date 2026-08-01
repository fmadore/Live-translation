//! Live Captions — Tauri desktop core for translation and transcription.
//!
//! Captures microphone and system-loopback audio, streams it to Google Gemini 3.5 Live
//! Translate over a WebSocket, and renders the returned translation as live captions in a
//! transparent, always-on-top overlay window.

mod audio;
mod commands;
mod gemini;
mod mistral;
mod openai;
mod realtime;
mod secrets;
mod session;
mod types;

use tauri::Manager;

use session::SessionManager;

const OVERLAY_LABEL: &str = "overlay";

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
            if let Some(overlay) = app.get_webview_window(OVERLAY_LABEL) {
                let _ = overlay.set_ignore_cursor_events(true);
                let _ = overlay.set_always_on_top(true);
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running the Live Captions app");
}
