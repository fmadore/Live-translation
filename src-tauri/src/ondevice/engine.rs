//! Recognizer selection — the one pluggable piece of the on-device path.
//!
//! `mod.rs` owns everything engine-independent (audio consumption, backpressure, turn
//! bookkeeping, status reporting). This module answers one question: which local speech
//! recognizer actually turns PCM into text.
//!
//! **No engine is wired up yet.** The choice has real costs — package size, build
//! toolchain, caption quality — and is recorded here rather than in a commit message so the
//! next person sees why. The candidates, with the constraint that decides between them:
//!
//! Every candidate must accept **pushed PCM**. That is not a stylistic preference: the app
//! captures system audio over WASAPI loopback and mixes device selection into the pipeline,
//! so a recognizer that opens its own microphone can caption neither system audio nor a
//! chosen device, and cannot serve *Both* mode at all.
//!
//! - **`Windows.Media.SpeechRecognition` (inbox WinRT)** — *ruled out.* Stable since
//!   Windows 10 and already projected by the `windows` crate, but its API surface has no
//!   audio input of any kind: no stream, file, buffer or device selector. It always opens
//!   the system default capture device itself. Verified against the generated bindings for
//!   `windows` 0.62 — `SpeechRecognizer` exposes only constraints, timeouts, UI options and
//!   the continuous-session handle.
//!
//! - **Speech Recognition Windows AI API** — the natural long-term answer. Whisper-derived,
//!   free, runs on the NPU on Copilot+ PCs and on CPU elsewhere, and crucially accepts
//!   caller-pushed audio via `SpeechAudioProvider`, which fits this trait exactly. Two
//!   blockers today: it is in the Windows App SDK **experimental channel**, which cannot
//!   back a Microsoft Store submission, and it is a `Microsoft.Windows.*` App SDK type
//!   rather than an OS `Windows.*` one, so the `windows` crate does not project it — Rust
//!   consumption needs the App SDK bootstrapper and a projection of its own.
//!
//! - **`whisper-rs` (whisper.cpp)** — shippable today. Accepts PCM directly, fully offline,
//!   no Windows version dependency, quality far above anything else stable here. Costs a
//!   bundled model (~75 MB tiny, ~142 MB base), a C++/CMake step in CI, and CPU headroom
//!   during a session.
//!
//! Implement one, return it from [`new_recognizer`], and the rest of the path is already
//! built.

use anyhow::Result;

use super::Recognizer;

/// Build the platform's local recognizer, or explain why there isn't one.
///
/// `language_tag` is a BCP-47 hint (`en-US`, `fr-FR`); `None` follows the system language.
pub fn new_recognizer(language_tag: Option<&str>) -> Result<Box<dyn Recognizer>> {
    let _ = language_tag;
    anyhow::bail!(
        "On-device captions are not available in this build — no local recognizer is \
         compiled in yet. Choose Mistral for subtitles, or Gemini/OpenAI for translation."
    )
}
