//! The voluntary-support link (issue #180): Buy Me a Coffee, opened in the default browser.
//!
//! The command takes no argument on purpose. Opening a URL the webview chose would hand the
//! interface a way to launch anything the shell can open; this opens one fixed page and
//! nothing else. A plain `target="_blank"` link is not enough either: with no new-window
//! handler, WebView2 opens it in a popup of its own rather than in the operator's browser.

use crate::errors::{id, AppError};

/// The creator's page. Also in `src/lib/support.ts`, which shows it and opens it in a browser
/// preview, in the README and in `.github/FUNDING.yml`; a test below keeps the first in step.
pub const SUPPORT_URL: &str = "https://buymeacoffee.com/fmadore";

#[tauri::command]
pub async fn open_support_page() -> Result<(), AppError> {
    tauri::async_runtime::spawn_blocking(|| open(SUPPORT_URL))
        .await
        .map_err(|e| AppError::with(id::TASK_FAILED, e))?
}

/// Open one of the app's own fixed pages in the default browser: this one, or the Zoom setup
/// guide (`zoom::GUIDE_URL`). Never a URL the interface chose.
#[cfg(windows)]
pub(crate) fn open(url: &str) -> Result<(), AppError> {
    use std::ptr::{null, null_mut};
    use windows_sys::Win32::System::Com::{
        CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE,
    };
    use windows_sys::Win32::UI::Shell::ShellExecuteW;
    use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

    let verb: Vec<u16> = "open".encode_utf16().chain([0]).collect();
    let file: Vec<u16> = url.encode_utf16().chain([0]).collect();

    // SAFETY: both strings are NUL-terminated UTF-16 buffers that outlive the call, and the
    // null owner window, parameters and directory are documented as allowed. COM is set up
    // first because ShellExecute may hand the URL to a shell extension that needs it; the
    // blocking-pool thread is ours for the duration, and an `S_FALSE` (already initialised)
    // still has to be balanced, so any non-failure result is uninitialised again.
    let result = unsafe {
        let com = CoInitializeEx(
            null(),
            (COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE).cast_unsigned(),
        );
        let instance = ShellExecuteW(
            null_mut(),
            verb.as_ptr(),
            file.as_ptr(),
            null(),
            null(),
            SW_SHOWNORMAL,
        );
        if com >= 0 {
            CoUninitialize();
        }
        instance.addr()
    };

    // Anything above 32 is success; the documented failures are the small values below it.
    if result > 32 {
        Ok(())
    } else {
        Err(AppError::with(
            id::OPEN_LINK,
            format!("ShellExecute returned {result}"),
        ))
    }
}

#[cfg(not(windows))]
pub(crate) fn open(_url: &str) -> Result<(), AppError> {
    Err(AppError::with(
        id::OPEN_LINK,
        "opening a browser is only implemented on Windows",
    ))
}

#[cfg(test)]
mod tests {
    use super::SUPPORT_URL;

    const FRONT_END: &str = include_str!("../../src/lib/support.ts");

    #[test]
    fn the_page_is_https() {
        assert!(SUPPORT_URL.starts_with("https://"));
    }

    #[test]
    fn the_front_end_names_the_same_page() {
        assert!(
            FRONT_END.contains(&format!("'{SUPPORT_URL}'")),
            "src/lib/support.ts shows a different support page from the one this opens"
        );
    }
}
