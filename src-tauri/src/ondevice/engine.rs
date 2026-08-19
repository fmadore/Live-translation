//! Recognizer selection and model resolution — the pluggable piece of the on-device path.
//!
//! `mod.rs` owns everything engine-independent (audio consumption, backpressure, turn
//! bookkeeping, status reporting). This module answers one question: which local speech
//! recognizer turns PCM into text.
//!
//! Every candidate must accept **pushed PCM**. That is not a stylistic preference: the app
//! captures system audio over WASAPI loopback and offers microphone selection, so a
//! recognizer that opens its own audio device can caption neither system audio nor a chosen
//! microphone, and cannot serve *Both* mode at all. The candidates, and why the current one
//! won:
//!
//! - **`whisper-rs` (whisper.cpp)** — *in use.* Takes PCM directly, fully offline, no
//!   Windows version dependency, and the best quality available from anything stable. Costs
//!   a bundled model and a C++/CMake step in the build.
//!
//! - **Speech Recognition Windows AI API** — the migration target. Whisper-derived, free,
//!   no bundled model, NPU-accelerated on Copilot+ PCs and CPU elsewhere, and it accepts
//!   caller-pushed audio via `SpeechAudioProvider`. Two blockers today: it is in the Windows
//!   App SDK **experimental channel**, which cannot back a Microsoft Store submission, and
//!   it is a `Microsoft.Windows.*` App SDK type rather than an OS `Windows.*` one, so the
//!   `windows` crate does not project it — Rust use needs the App SDK bootstrapper and a
//!   projection of its own. Revisit when it reaches the stable channel: it would drop the
//!   model from the installer entirely.
//!
//! - **`Windows.Media.SpeechRecognition` (inbox WinRT)** — *ruled out.* Stable since Windows
//!   10 and already projected by the `windows` crate, but its API surface has no audio input
//!   of any kind: no stream, file, buffer or device selector. It always opens the system
//!   default capture device itself. Verified against the generated bindings for `windows`
//!   0.62, where `SpeechRecognizer` exposes only constraints, timeouts, UI options and the
//!   continuous-session handle.

use std::path::{Path, PathBuf};

use anyhow::Result;
use tauri::{AppHandle, Manager};

use super::Recognizer;

/// Directory the model is bundled into, relative to the app's resource directory. Its
/// contents come from `scripts/fetch-whisper-model.mjs` via `bundle.resources` in
/// `tauri.conf.json`; the file keeps its real name (`ggml-base-q5_1.bin`, `ggml-small.bin`,
/// …) so which model shipped is visible on disk.
const BUNDLED_MODEL_DIR: &str = "models";

/// Override for development, so a checkout can point at a model without a rebuild — and so a
/// larger model can be trialled on capable hardware without changing what ships.
const MODEL_PATH_ENV: &str = "WHISPER_MODEL_PATH";

/// Whisper wants a bare language code; the pipeline carries BCP-47 tags.
fn whisper_language(tag: Option<&str>) -> Option<&str> {
    tag.and_then(|tag| tag.split(['-', '_']).next())
        .filter(|code| !code.is_empty())
}

/// Locate the speech model: the environment override first, then the bundled resource.
fn model_path(app: &AppHandle) -> Result<PathBuf> {
    if let Some(path) = std::env::var_os(MODEL_PATH_ENV) {
        let path = PathBuf::from(path);
        anyhow::ensure!(
            path.is_file(),
            "{MODEL_PATH_ENV} points at {}, which is not a file",
            path.display()
        );
        return Ok(path);
    }

    let dir = app
        .path()
        .resolve(BUNDLED_MODEL_DIR, tauri::path::BaseDirectory::Resource)
        .map_err(|error| anyhow::anyhow!("could not locate the app resource directory: {error}"))?;

    first_model_in(&dir).ok_or_else(|| {
        anyhow::anyhow!(
            "The on-device speech model is missing from this build. Run \
             `npm run fetch:whisper-model` and rebuild, or set {MODEL_PATH_ENV} to a \
             ggml model file."
        )
    })
}

/// Pick the model to load from the bundled directory. Sorted so that a directory which
/// somehow holds more than one `.bin` still resolves to the same file on every launch
/// instead of following filesystem order.
fn first_model_in(dir: &Path) -> Option<PathBuf> {
    let mut models: Vec<PathBuf> = std::fs::read_dir(dir)
        .ok()?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_file() && path.extension().is_some_and(|extension| extension == "bin")
        })
        .collect();
    models.sort();
    models.into_iter().next()
}

/// Build the local recognizer, or explain why there isn't one.
///
/// `language_tag` is a BCP-47 hint (`en-US`, `fr-FR`); `None` lets the engine auto-detect.
pub fn new_recognizer(app: &AppHandle, language_tag: Option<&str>) -> Result<Box<dyn Recognizer>> {
    let path = model_path(app)?;
    let recognizer = super::whisper::WhisperRecognizer::new(&path, whisper_language(language_tag))?;
    Ok(Box::new(recognizer))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_lookup_is_deterministic_and_ignores_non_models() {
        let dir = std::env::temp_dir().join("live-translation-model-scan-test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        assert_eq!(
            first_model_in(&dir),
            None,
            "an empty directory has no model"
        );

        std::fs::write(dir.join("README.txt"), b"not a model").unwrap();
        assert_eq!(first_model_in(&dir), None, "only .bin files count");

        std::fs::write(dir.join("ggml-small.bin"), b"x").unwrap();
        std::fs::write(dir.join("ggml-base.bin"), b"x").unwrap();
        // Sorted, so the same file is chosen on every launch regardless of readdir order.
        assert_eq!(
            first_model_in(&dir).unwrap().file_name().unwrap(),
            "ggml-base.bin"
        );

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn bcp47_tags_reduce_to_whisper_language_codes() {
        assert_eq!(whisper_language(Some("en-US")), Some("en"));
        assert_eq!(whisper_language(Some("fr-FR")), Some("fr"));
        assert_eq!(whisper_language(Some("fr")), Some("fr"));
        // Auto-detect rather than guessing.
        assert_eq!(whisper_language(None), None);
        assert_eq!(whisper_language(Some("")), None);
        assert_eq!(whisper_language(Some("-FR")), None);
    }
}
