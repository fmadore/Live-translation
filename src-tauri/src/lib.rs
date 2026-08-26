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
mod recovery;
mod secrets;
mod session;
mod types;

use tauri::{Emitter, Manager};

use overlay::OVERLAY_LABEL;
use recovery::{CloseGuard, ACK_TIMEOUT, OPERATOR_LABEL};
use session::SessionManager;
use types::events;

/// Fail loudly and legibly if the WebView2 Runtime is absent, instead of launching into
/// nothing.
///
/// A packaged desktop app must not silently fail when the runtime is missing: the MSIX
/// package deliberately does not declare WebView2 as a package dependency (declaring it is
/// known to be unreliable), and the release build is a `windows` subsystem binary, so an
/// unexplained exit is all the user would otherwise see. Windows 11 ships the Evergreen
/// Runtime and Windows 10 has had it through Edge for years, so this is a last-resort path.
/// See gate 9 in `docs/microsoft-store.md`.
fn assert_webview_runtime() {
    let Err(error) = tauri::webview_version() else {
        return;
    };
    tracing::error!("WebView2 runtime unavailable: {error}");

    #[cfg(windows)]
    {
        use windows_sys::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, MB_OK};

        // Win32 wants NUL-terminated UTF-16.
        fn wide(text: &str) -> Vec<u16> {
            text.encode_utf16().chain(std::iter::once(0)).collect()
        }

        let text = wide(
            "Live Translation & Subtitles needs the Microsoft Edge WebView2 Runtime, which is \
             not installed on this PC.\n\nInstall it from \
             https://developer.microsoft.com/microsoft-edge/webview2/ and start the app again.",
        );
        let caption = wide("Live Translation & Subtitles");
        // SAFETY: both buffers are NUL-terminated and outlive this modal, synchronous call.
        unsafe {
            MessageBoxW(
                std::ptr::null_mut(),
                text.as_ptr(),
                caption.as_ptr(),
                MB_ICONERROR | MB_OK,
            );
        }
    }

    std::process::exit(1);
}

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

    assert_webview_runtime();

    tauri::Builder::default()
        .manage(SessionManager::default())
        .manage(CloseGuard::default())
        .invoke_handler(tauri::generate_handler![
            commands::list_microphones,
            commands::has_api_key,
            commands::set_api_key,
            commands::clear_api_key,
            commands::ondevice_readiness,
            commands::prepare_ondevice_model,
            commands::start_session,
            commands::stop_session,
            commands::start_audio_test,
            commands::stop_audio_test,
            commands::set_overlay_click_through,
            commands::show_overlay,
            commands::save_transcript,
            recovery::ack_close,
            recovery::write_recovery,
            recovery::read_recovery,
            recovery::clear_recovery,
            recovery::set_close_guard,
            recovery::confirm_close,
        ])
        .on_window_event(|window, event| {
            // The overlay is a caption surface with no controls; only the operator window
            // can be holding an answer the app needs.
            if window.label() != OPERATOR_LABEL {
                return;
            }
            match event {
                tauri::WindowEvent::CloseRequested { api, .. } => {
                    // Intercept only while the front-end says there is something to lose.
                    if !window.state::<CloseGuard>().should_intercept() {
                        return;
                    }
                    api.prevent_close();
                    let attempt = window.state::<CloseGuard>().begin_attempt();
                    // Broadcast, like every other core event: the front-end's `listen`
                    // registers against `EventTarget::Any`, which a labeled `emit_to` would
                    // filter straight back out.
                    let _ = window.emit(events::CLOSE_REQUESTED, ());

                    // …and make sure the window still closes if nothing answers. Without this
                    // a renderer that wedged while the transcript was unsaved would leave the
                    // operator with a window that only Task Manager can shut.
                    let window = window.clone();
                    tauri::async_runtime::spawn(async move {
                        tokio::time::sleep(ACK_TIMEOUT).await;
                        let guard = window.state::<CloseGuard>();
                        if guard.acknowledged(attempt) {
                            return;
                        }
                        tracing::warn!(
                            "front-end did not acknowledge close attempt {attempt}; closing anyway"
                        );
                        guard.release();
                        let _ = window.close();
                    });
                }
                // The overlay is undecorated, always-on-top and absent from the taskbar, so
                // outliving the operator window would leave a caption layer floating over
                // everything with nothing left that can dismiss it.
                tauri::WindowEvent::Destroyed => window.app_handle().exit(0),
                _ => {}
            }
        })
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
