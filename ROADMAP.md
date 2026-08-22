# Roadmap

This file combines the current delivery plan with the completed implementation history.
GitHub milestones are the source of truth for active work; the phase checklists below preserve
why earlier architectural decisions were made.

## Current status — 1.0 Store certification

Version **1.0.0** has been submitted to the Microsoft Store and is in certification. Do not
change the submitted package while certification is running. Continue preparatory work on a
branch, then ship it through a normal post-certification update.

- [Microsoft Store 1.0 milestone](https://github.com/fmadore/Live-translation/milestone/1)
- [Store distribution issue #13](https://github.com/fmadore/Live-translation/issues/13)

## Planning conventions

- A milestone is a release outcome, not a promised date.
- Each active issue owns one independently testable change and carries its acceptance criteria.
- High-priority correctness, data-safety, and build work lands before optional features in the
  same milestone.
- Windows behavior is verified in an installed MSIX as well as browser/Tauri development; audio
  changes also require real hardware and a meeting-app test.
- Experimental Windows APIs remain outside Store builds until Microsoft documents them as stable
  and Store-eligible.

## 1.1 — Polish, French, and tray

**Goal:** make the first update safer in a live event, accessible on Windows, bilingual at the
interface level, and comfortable to leave running without an open operator window.

Recommended implementation order:

1. **Correctness and reproducible builds**
   - [#20 — truthful audio preflight](https://github.com/fmadore/Live-translation/issues/20)
   - [#21 — align the F2 promise with actual behavior](https://github.com/fmadore/Live-translation/issues/21)
   - [#29 — prevent Tauri IPC calls in browser previews](https://github.com/fmadore/Live-translation/issues/29)
   - [#30 — document and pin down libclang for whisper builds](https://github.com/fmadore/Live-translation/issues/30)
2. **Lifecycle and transcript safety**
   - [#25 — protect long and unsaved transcripts](https://github.com/fmadore/Live-translation/issues/25)
   - [#22 — system tray controls and safe close/quit](https://github.com/fmadore/Live-translation/issues/22)
3. **Inclusive and bilingual UI**
   - [#24 — Windows accessibility and high-contrast pass](https://github.com/fmadore/Live-translation/issues/24)
   - [#23 — French app and Store localization](https://github.com/fmadore/Live-translation/issues/23)
4. **Defense in depth**
   - [#31 — separate operator and overlay capabilities](https://github.com/fmadore/Live-translation/issues/31)

Tray behavior is deliberately explicit: normal minimize keeps Windows taskbar semantics; an
operator can choose **Minimize to tray** or enable **Keep running in the tray when I close the
window**. A live session must never disappear silently. The tray always provides Open,
Show/Hide overlay, Stop session, and Quit; Quit drains the session and protects unsaved text.

Definition of done for 1.1:

- All issue acceptance criteria and CI checks pass.
- Keyboard, Narrator, contrast-theme, reduced-motion, and 225% text-scale checks pass.
- English and French layouts are visually checked at the minimum window size and on the overlay.
- Tray, graceful quit, Credential Manager, microphone, and loopback behavior pass in the Store
  MSIX on Windows 11.

## 1.2 — Windows integration

**Goal:** narrow capture to the intended source and make the resulting captions useful outside
the app.

1. [#28 — audio hot-plug and loopback-output selection](https://github.com/fmadore/Live-translation/issues/28)
2. [#27 — per-application WASAPI loopback capture](https://github.com/fmadore/Live-translation/issues/27)
3. [#26 — native Save As plus SRT/VTT export](https://github.com/fmadore/Live-translation/issues/26)

Device lifecycle work comes first because both all-system and per-process capture need a shared,
recoverable device model. The process-capture implementation must retain all-system loopback as
a fallback on unsupported Windows builds.

Definition of done for 1.2:

- Teams/Zoom, Chromium child processes, USB/Bluetooth removal, dock changes, sleep/wake, and
  mixed-DPI multi-monitor scenarios pass the hardware test matrix.
- Process capture excludes unrelated notification/media audio.
- Markdown, text, SRT, and VTT exports round-trip through a native picker in the Store MSIX.

## Research and unscheduled work

- [#32 — Windows AI speech-recognition prototype](https://github.com/fmadore/Live-translation/issues/32)
  is intentionally unmilestoned and cannot enter a Store build while the API is experimental.
- [#12 — automatic FR ⇄ EN direction](https://github.com/fmadore/Live-translation/issues/12)
  needs a measured provider-switching design before it becomes a release commitment.
- Event glossary for names, institutions, acronyms, and specialist terminology.
- Optional bilingual overlay with source text and translated text.
- Privacy-preserving diagnostic export: versions, devices, reconnects, queue drops, and sanitized
  errors, but never keys or audio.
- Persist overlay position/size and add multi-monitor overlay presets.
- Expand beyond English/French once the localized selector and provider capability model exist.
- Measure audio-to-first-caption latency and make rate-card verification dates visible.

## Completed delivery history

Findings from the July and August 2026 reviews were organized into the implementation phases
below. Checked items have landed; the git history references the phase numbers.

## Phase 1 — Correctness (Rust core)

- [x] **Surface capture-stream errors properly.** The cpal error callback emitted the
  `status` event with an empty payload, which crashed the front-end listener; it now emits a
  real `StatusUpdate` with `state: error` and a message.
- [x] **No leaked capture threads on partial start failure.** If the second source failed to
  spawn, the first source's thread and client task ran forever (dropping a
  `CancellationToken` does not cancel it). `SessionManager::start` now holds a drop guard
  that cancels everything on any error path.
- [x] **Per-origin status.** Up to four tasks (two captures + two clients in *Both* mode)
  raced on a single global session state, so one source's `reconnecting` clobbered the
  other's `running`, and a capture error showed *Error* + a Start button while the other
  source was still live. `StatusUpdate` now carries its origin and the operator UI
  aggregates: worst state wins for display, Stop stays available while anything is active.
- [x] **Level metering off the real-time audio thread.** RMS/peak events were serialized and
  sent over webview IPC from inside the cpal callback; they now go through a channel to a
  dedicated emitter task.

## Phase 2 — Reliability & architecture (Rust core)

- [x] **Shared realtime session runner** (`realtime.rs`). The Gemini and OpenAI clients
  duplicated ~130 lines (reconnect/backoff loop, turn accumulator, select loop, emit
  helpers); each is now a small `RealtimeProtocol` impl and the runner owns the rest, so
  every fix below lands in one place.
- [x] **Backoff resets after a stable connection.** Previously it doubled forever, so late in
  a long session every routine reconnect waited the full 16 s.
- [x] **Fail fast on rejected handshakes.** A 4xx WebSocket handshake (bad API key, bad
  model) now stops with a clear error instead of looping "Reconnecting…" forever.
- [x] **Drop stale audio before reconnecting.** Audio buffered while the socket was down was
  replayed on reconnect, putting captions tens of seconds behind live speech.
- [x] **Turn ids survive reconnects** (accumulator lives outside the connect loop), so the
  front-end no longer sees `turnId` restart at 0 mid-session.
- [x] **`Caption` serializes via serde** (`#[serde(rename = "final")]`) instead of a
  hand-built JSON mirror.
- [x] **Remove the unreachable Speech→Text engine.** The UI for it was dropped in v0.2.2 but
  the whole backend path (mode enum, setup prompt, model id, env plumbing) remained.
- [x] **Drop unused dependencies** (`url`, `thiserror`).

## Phase 3 — Efficiency (audio path)

- [x] **No per-wake copy in WASAPI loopback decode** (`make_contiguous` instead of
  collecting the deque into a fresh `Vec` every ~10 ms).
- [x] **Minimize callback allocations**: reusable conversion/resampling scratch buffers;
  one owned PCM allocation remains per completed chunk so the callback can transfer it to
  the async pipeline without blocking.
- [x] **Async Tauri commands.** Sync commands run on the main thread; keychain access,
  transcript file I/O, and `stop_session` (which joins capture threads) could block the UI.

## Phase 4 — Front-end correctness & UX

- [x] **Captions keyed by (origin, turnId).** Turn ids are per-source counters, so in *Both*
  mode mic and system captions merged or clobbered each other in the transcript and overlay.
  The transcript tracks one pending turn per origin; the overlay renders one line per origin.
- [x] **Stable transcript keys** (monotonic id instead of `time + text`, which collided).
- [x] **Level meters reset when the session ends** instead of freezing at the last value.
- [x] **`Origin` type** in TypeScript (`'microphone' | 'system'`) — `Caption.origin` was
  typed as `AudioSource`, which wrongly included `'both'`.
- [x] **Overlay move mode.** The overlay is click-through and undecorated, so it could never
  be repositioned. A "Move overlay" toggle in the operator window disables click-through and
  turns the overlay into a drag region with a visible outline; toggling it off restores
  click-through. (This also wires up the previously dead `set_overlay_click_through` command.)
- [x] **Deduplicate font-size logic** (shared load/clamp helpers used by both windows).
- [x] **Reuse the `isRunning` derived store** instead of a page-local copy.

## Phase 5 — Tooling & housekeeping

- [x] **CI workflow**: `svelte-check`, `cargo fmt --check`, `cargo clippy -D warnings`,
  `cargo test` on every push/PR (release.yml only built installers).
- [x] **Config nits**: Vite `envPrefix` glob (`TAURI_ENV_*` never matched — prefixes are
  literal), `engines` field in package.json, wrong `custom-protocol` feature comment.
- [x] **Docs**: README / architecture.md / .env.example updated for the removed Speech→Text
  engine and the new session runner.
- [x] Version bump to **0.3.0**.

## Phase 6 — Subtitles, hardening, and CI (August 2026)

- [x] **Mistral Live subtitles** using `voxtral-mini-transcribe-realtime-2602`, with a
  dedicated mode so transcription cannot be mistaken for translation.
- [x] **Plain-text and Markdown export**, shared by both translated captions and subtitles,
  with pure formatter tests.
- [x] **Current provider contracts**: Gemini's documented AUDIO/setup/blob shape; OpenAI's
  translation session schema and graceful `session.close`; Mistral's official SDK frames.
- [x] **Serialized start/stop lifecycle**, per-source cancellation, connection timeout,
  graceful tail draining, proactive Gemini `goAway`, retryable 429 handling, and turn
  finalization across reconnects.
- [x] **Bounded audio and meter channels**, nonblocking callbacks, periodic pending-buffer
  compaction, broader CPAL sample-format support, and an anti-alias filter before downsampling.
- [x] **Frontend refactor** into API-key and transcript-monitor components; configuration
  locks while starting/running/stopping and stale async key checks are ignored.
- [x] **Toolchain/security maintenance**: current SvelteKit/Svelte/Vite/Tauri packages,
  frontend unit tests, Windows+Linux Rust CI, npm/RustSec audits, actionlint, Dependabot, and
  current release actions.
- [x] Version bump to **0.4.0**.

## Phase 7 — Distribution and Store submission

The MSIX packaging path, keyless on-device subtitle engine, 1.0.0 version, privacy policy,
Partner Center material, and certification notes have landed. Version 1.0.0 is now in Microsoft
Store certification. The full analysis, route comparison, and submission plan live in
[`docs/microsoft-store.md`](docs/microsoft-store.md). Summary: package as **MSIX** and submit
to the **Microsoft Store**, which re-signs the package with a Microsoft certificate at no
cost, removing the warning entirely.

The critical path is not packaging but a **keyless on-device subtitle engine**. Store policy
10.8.3 classifies provider **API keys** as financial information and bars individual accounts
from requiring them for primary functionality; a company account is out of scope, so captions
have to work with no credential at all. The same work answers policy 10.3.1's demand that
certification be able to test the app. **Gemini, OpenAI and Mistral are unaffected**: the
on-device engine is an additional keyless provider under Live subtitles, and translation stays
cloud-only because Windows has no on-device translation API.

This has landed: `Provider::OnDevice`, the `ondevice::run_session` driver, whisper.cpp behind
a `Recognizer` seam, the operator UI, and unit tests. Inbox `Windows.Media.SpeechRecognition`
turned out to be unusable here — it has no audio-input API and always opens the default
microphone, so it can serve neither system-loopback audio nor *Both* mode. The Whisper-derived
Speech Recognition Windows AI API is the migration target once it leaves the Windows App SDK
experimental channel; it would remove the bundled model entirely — and would restore native
ARM64, since **ARM64 Windows had to be dropped from releases**: ggml refuses to build with
MSVC on ARM ("MSVC is not supported for ARM, use clang") and whisper-rs-sys pins the Visual
Studio CMake generator, which always uses cl.exe. ARM64 machines run the x64 build under
emulation in the meantime. Still to verify on real hardware: caption latency and accuracy.

The repository remains at **1.0.0** while certification is pending. GitHub/NSIS releases remain
available as a fallback distribution path.

macOS support was dropped as part of this; Windows is the only supported target and the Linux
CI lane is a compile check only.

## Earlier future ideas (superseded where linked above)

- **Persist overlay position/size** across launches (tauri-plugin-window-state).
- **Per-origin caption styling** on the overlay (e.g. a subtle "🎤 / 🔊" prefix when both
  sources are live).
- **More target languages** — the enum is trivially extensible; the UI needs a select
  instead of two buttons.
- **Latency metrics** in the operator monitor (audio-sent → first-delta round trip).
- **Session cost estimate** (audio minutes streamed per provider).
- **Rehearsal mode** — play a bundled FR/EN sample file through the pipeline to validate
  keys/models before the event without speaking.
- **Billable provider smoke workflow** — an explicitly manual workflow could exercise live
  credentials and a golden audio fixture. It is intentionally not automatic because it costs
  money and CI secrets are not available to forked pull requests.
