# Roadmap

Findings from the July 2026 code review, organized into implementation phases.
Items are checked off as they land; the git history references the phase numbers.

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

## Phase 7 — Distribution (planned)

Installers are unsigned, so every recipient meets a SmartScreen "unknown publisher" wall.
The full analysis, route comparison and phased plan live in
[`docs/microsoft-store.md`](docs/microsoft-store.md). Summary: package as **MSIX** and submit
to the **Microsoft Store**, which re-signs the package with a Microsoft certificate at no
cost, removing the warning entirely. Three app-specific gates decide the schedule — Store
policy 10.8.3 treats provider **API keys** as financial information and may force a company
account, an MSIX version's first segment cannot be `0` (so the Store debut is **1.0.0**), and
certification needs a way to exercise the app **without** a paid key, which promotes
*rehearsal mode* below from nice-to-have to prerequisite. Not on the critical path for the
September 2026 workshop; the unsigned installer remains the event-day route.

## Future ideas (not scheduled)

- **Persist overlay position/size** across launches (tauri-plugin-window-state).
- **Per-origin caption styling** on the overlay (e.g. a subtle "🎤 / 🔊" prefix when both
  sources are live).
- **More target languages** — the enum is trivially extensible; the UI needs a select
  instead of two buttons.
- **Latency metrics** in the operator monitor (audio-sent → first-delta round trip).
- **Session cost estimate** (audio minutes streamed per provider).
- **macOS system-loopback capture** — currently Windows-only (WASAPI). Not scheduled: the
  STIAS event laptop is Windows. **The gate is code signing, not the audio code.** Both
  native routes are TCC-gated and reportedly need a binary signed with a Developer ID
  certificate (Team ID); ad-hoc/unsigned — what `release.yml` produces today — cannot hold
  the permission on recent macOS. That means an Apple Developer Program membership
  ($99/yr) plus signing + notarization secrets in CI *before* any capture code is useful.
  Three routes, cheapest first:
  - **BlackHole** (or any virtual audio device): no signing, no new backend. It registers
    as a normal CoreAudio input device, so it should already work as a source today — but
    it is tagged `Origin::Microphone`, so Both mode labels its captions wrong. Making it
    first-class ("treat this input device as the System origin") is a small change to
    `audio/capture.rs` plus the operator UI, and needs no Apple account. Costs the operator
    a driver install and a Multi-Output Device on the day.
  - **Core Audio process taps** (macOS 14.4+): the right native answer. Audio-only
    permission (`NSAudioCaptureUsageDescription`) rather than the scarier Screen Recording
    prompt. ~300–400 lines against `objc2-core-audio`. Reference: `insidegui/AudioCap`.
  - **ScreenCaptureKit** (macOS 13+): less code via the `screencapturekit` crate (v8, well
    maintained), but needs Screen Recording permission — a harder sell on a shared
    conference laptop.

  Either native backend slots in behind the existing `run_system_loopback` seam in
  `audio/loopback.rs`, and needs no echo filtering: the app never plays audio, so there is
  no self-capture feedback loop. Note that a macOS backend can only be *compile*-verified
  from CI — TCC grants and real audio flow need a physical Mac, so budget a rehearsal.
- **Rehearsal mode** — play a bundled FR/EN sample file through the pipeline to validate
  keys/models before the event without speaking.
- **Billable provider smoke workflow** — an explicitly manual workflow could exercise live
  credentials and a golden audio fixture. It is intentionally not automatic because it costs
  money and CI secrets are not available to forked pull requests.
