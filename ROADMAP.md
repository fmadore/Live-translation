# Roadmap

This file combines the current delivery plan with the completed implementation history.
GitHub milestones are the source of truth for active work; the phase checklists below preserve
why earlier architectural decisions were made.

## Current status — 1.1.0 is live

Version **1.0.5** passed certification and is published at
[apps.microsoft.com/detail/9PFB8LR3RR9X](https://apps.microsoft.com/detail/9PFB8LR3RR9X).
Version **1.1.0** passed certification on 27 August 2026 and is what the Store now hands to
anyone who installs or updates the app. Updates are uploaded to Partner Center by hand, as
described in [`docs/store-automation.md`](docs/store-automation.md) — the submission API
authenticates as a Microsoft Entra application, which Partner Center offers only to Company
accounts.

Version **1.0.5** was the release before it, and the one that first got through.

It took several attempts. The 1.0.3 submission failed policy 10.1.2.10 because **Start
Subtitles** did nothing on the review device, and neither credential-free Windows recognizer
was portable enough to fix it: the experimental Windows AI Speech/ML component crashed
natively on the target ARM64 Surface, and `Windows.Media.SpeechRecognition` depended on
privacy consent, installed speech languages, network behavior and a usable default
microphone. 1.0.5 dropped on-device recognition entirely in favour of a deterministic bundled
demonstration that needs no device, account, language pack or network.

That is the standing constraint on everything below: **the default path has to work on a
machine nobody configured**, because that is the machine certification runs on.

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

**Shipped in 1.1.0**, in the order they were built:

1. **Correctness**
   - [x] [#20 — truthful audio preflight](https://github.com/fmadore/Live-translation/issues/20)
   - [x] [#21 — align the F2 promise with actual behavior](https://github.com/fmadore/Live-translation/issues/21)
   - [x] [#29 — prevent Tauri IPC calls in browser previews](https://github.com/fmadore/Live-translation/issues/29)
2. **Lifecycle and transcript safety**
   - [x] [#25 — protect long and unsaved transcripts](https://github.com/fmadore/Live-translation/issues/25)
   - [x] [#22 — system tray controls and safe close/quit](https://github.com/fmadore/Live-translation/issues/22) —
     code complete; keyboard/Narrator operation of the tray menu and the packaged-build walk
     are manual and stay open until they are run on Windows.

**Built after the 1.1.0 package was cut**, and therefore due in the next release. Everything
below is on `main` and in none of the bytes the Store is serving:

3. **Inclusive and bilingual UI**
   - [#24 — Windows accessibility and high-contrast pass](https://github.com/fmadore/Live-translation/issues/24) —
     contrast, focus, headings, live regions, `aria-busy`, contrast themes and the modal focus
     trap have landed, with [`docs/accessibility.md`](docs/accessibility.md) carrying the
     standard and the release walk. **Text scaling has now landed too**, which was the last
     criterion open. Windows' *Make text bigger* does not reach WebView2 content
     ([WebView2Feedback#1662](https://github.com/MicrosoftEdge/WebView2Feedback/issues/1662)),
     so `src-tauri/src/textscale.rs` reads `UISettings.TextScaleFactor`, follows its change
     event, and the operator window multiplies one type ramp by it. The half that is easy to
     miss is that honouring a text setting is not only a question of type: a 225% caption in a
     380px rail is clipped, not accessible. So every gutter and width that carries text is
     measured in `em` and the two-column layout is a container query in `em`, which stacks the
     columns into one scrolling column at the point they would start clipping and restores
     them when the window is widened. The overlay opts out, like it does for contrast themes —
     its captions are projected content the operator sizes for the room.
   - [#23 — French app and Store localization](https://github.com/fmadore/Live-translation/issues/23) —
     landed. Typed catalogs, a language selector independent of the caption language, and
     `AppError { id, detail }` in place of every user-facing string the Rust core used to
     format: **the core names failures, the interface words them**. It came after the
     accessibility pass on purpose — that pass added accessible names and announcements, which
     are strings. [`docs/localization.md`](docs/localization.md) is the contract. What is left
     is not code: French screenshots for the Store listing, and a native speaker's review.
4. **Defense in depth**
   - [#31 — separate operator and overlay capabilities](https://github.com/fmadore/Live-translation/issues/31) —
     landed. The overlay could invoke every command in the app; it now has one. Splitting the
     capability file alone would have been documentary, so `build.rs` declares an app ACL
     manifest, which is what makes Tauri enforce the split at runtime.

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

What that leaves is a **Windows session**, not more code: the Narrator, contrast-theme and
text-scaling walks in [`docs/accessibility.md`](docs/accessibility.md), the tray's keyboard
operation from #22, French screenshots for the Store listing, and a native French speaker's
review. The text-scaling layout is verified at the window's 980 × 660 minimum at every step of
the slider — no clipping, no overflow, no overlap — but verified in a browser preview at a
forced factor, which is not the same as a real slider on a real Windows machine.

## 1.2 — Windows integration

**Goal:** narrow capture to the intended source and make the resulting captions useful outside
the app.

1. [#28 — audio hot-plug and loopback-output selection](https://github.com/fmadore/Live-translation/issues/28)
2. [#27 — per-application WASAPI loopback capture](https://github.com/fmadore/Live-translation/issues/27)
3. [#26 — native Save As plus SRT/VTT export](https://github.com/fmadore/Live-translation/issues/26)
   — its timing prerequisite has landed. The issue asks for cues built on "explicit monotonic
   caption timing rather than display timestamps", and there was no caption timing anywhere in
   the app: neither `Caption` nor `TranscriptLine` carried a clock, and the only one in reach
   was the renderer's arrival time, which is the thing the issue rules out. Captions now carry
   an interval stamped in the core (`src-tauri/src/timing.rs`), so what is left of #26 is the
   picker and the formatters.

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
  stays unmilestoned. It was tried during the 1.0.x certification attempts and crashed natively on the target
  ARM64 Surface, so it
  is now blocked on evidence rather than on ambition: it needs to survive a clean ARM64 and a
  clean x64 machine before it can be considered, and it cannot enter a Store build while the
  API is experimental.
- [#12 — automatic FR ⇄ EN direction](https://github.com/fmadore/Live-translation/issues/12)
  needs a measured provider-switching design before it becomes a release commitment.
- Event glossary for names, institutions, acronyms, and specialist terminology.
- Optional bilingual overlay with source text and translated text.
- Privacy-preserving diagnostic export: versions, devices, reconnects, queue drops, and sanitized
  errors, but never keys or audio.
- Persist overlay position/size and add multi-monitor overlay presets.
- Overlay caption presentation, which is a set of constants in
  `src/routes/overlay/+page.svelte` today:
  [#54 — configurable caption width](https://github.com/fmadore/Live-translation/issues/54) and
  [#55 — operator-chosen typeface, size and colours](https://github.com/fmadore/Live-translation/issues/55).
  A caption line is capped at `30ch` and centred, so a region snapped across a presentation
  display spends most of its width on scrim, and every colour is a literal — the font size is
  the only thing an operator can change. Both are small and both serve the room this app was
  built for, but neither belongs to 1.2's goal, so they wait here for a presentation milestone.
  #55 is the one with a trap: the audience view opts out of contrast themes on purpose, its
  dimmed steps are alphas of white, and the scrim is semi-transparent over a slide nobody
  controls — so operator-chosen colours need a composite contrast check, not a colour wheel.
- [#56 — German as a caption output language](https://github.com/fmadore/Live-translation/issues/56)
  now carries the standing note about expanding beyond English and French. The enum is still
  trivially extensible and both Rust matches on it are exhaustive, so the compiler names most of
  the work; what it cannot name is what stops being derivable once there are three languages —
  the F2 flip, and the rehearsal rule that picks *the language the room is not reading*. It also
  finally owes the localized selector the two hard-coded language cards have been standing in
  for. The interface stays English and French:
  [`docs/localization.md`](docs/localization.md) keeps the caption language and the UI language
  independent, and a German caption target does not imply a German UI.
- Measure audio-to-first-caption latency and make rate-card verification dates visible. The
  clock this needs now exists: `timing::SessionClock` stamps every caption, so the missing
  half is a mark on the audio side to measure against.

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

Done. MSIX packaging, a keyless default path, the privacy policy, the Partner Center material
and the certification notes all landed, and 1.0.5 is published. The Store re-signs the accepted
package with a Microsoft certificate at no cost, which is what removes the SmartScreen
"unknown publisher" wall the unsigned NSIS installer still meets.
[`docs/microsoft-store.md`](docs/microsoft-store.md) has the Store identity and the
certification history; [`docs/store-automation.md`](docs/store-automation.md) has the update
path.

The critical path was never packaging but a **keyless default path**. Store policy 10.8.3
classifies provider **API keys** as financial information and bars individual accounts from
requiring them for primary functionality; a company account is out of scope, so the app has to
do something useful with no credential at all. The same requirement answers policy 10.3's
demand that certification be able to test the app. **Gemini, OpenAI and Mistral are
unaffected**: they remain the live paths, and translation stays cloud-only.

Three keyless recognizers were tried and all three failed on a machine nobody had configured.
whisper.cpp worked but cost a 142 MB bundled model, a CMake/C++/libclang build dependency, and
native ARM64 — ggml refuses to build with MSVC on ARM ("MSVC is not supported for ARM, use
clang") and whisper-rs-sys pins the Visual Studio CMake generator, which always uses cl.exe.
Inbox `Windows.Media.SpeechRecognition` has no audio-input API and always opens the default
microphone, so it can serve neither system-loopback audio nor *Both* mode, and on the review
device it produced no captions at all. The experimental Windows AI Speech/ML component crashed
natively on the same ARM64 hardware.

1.0.5 stopped trying to recognize speech without a key. `Provider::OnDevice` now drives a
deterministic bundled demonstration — the same caption, overlay, level-meter, elapsed-clock,
transcript and export paths, with no device opened and nothing claimed about recognition.
Dropping whisper.cpp removed the model, the C++ toolchain and the ARM64 blocker at once, so
Store packages are now native x64 **and** ARM64.

The lesson is recorded because it will apply to the next attempt: a keyless path that depends
on the reviewer's hardware, language packs or privacy settings is not a keyless path.

macOS support was dropped as part of this; Windows is the only supported target and the Linux
CI lane is a compile check only.

## Phase 8 — The first update (1.1.0, August 2026)

Submitted and accepted on 27 August 2026, and live in the Store. Everything here is in that
package; the milestone itself stays open for the three issues listed under *1.1* above, none of
which are in it.

- [x] **Gemini 3.5 Transcribe Live as a second subtitle engine** beside Voxtral. Both detect
  the spoken language themselves, and one Gemini key now covers subtitles and translation.
- [x] **The transcript became a document** rather than a scrolling log: nothing is discarded
  however long the session runs, unsaved changes are visible, Clear asks before discarding,
  and an optional local recovery spool survives a crash or a power cut.
- [x] **A system tray that keeps a live session reachable.** Off by default — closing the
  window still quits — but with it on, closing hides the window and the session keeps
  captioning. Quit drains the session and protects unsaved text, and a second launch
  activates the existing window instead of opening a second capture process.
- [x] **A truthful pre-flight audio check** that reports only what it has actually heard from
  the selected source.
- [x] **F2 stopped promising a mid-session direction flip** the backend cannot perform: the
  target language is handed to the provider once, at session start.
- [x] **Browser previews stopped invoking Tauri commands**, so `npm run dev` no longer shows
  an IPC error the operator can neither act on nor dismiss.

## Earlier future ideas (superseded where linked above)

- **Persist overlay position/size** across launches (tauri-plugin-window-state).
- **Per-origin caption styling** on the overlay (e.g. a subtle "🎤 / 🔊" prefix when both
  sources are live).
- **More target languages** — the enum is trivially extensible; the UI needs a select
  instead of two buttons. Superseded by
  [#56](https://github.com/fmadore/Live-translation/issues/56).
- **Latency metrics** in the operator monitor (audio-sent → first-delta round trip).
- **Session cost estimate** (audio minutes streamed per provider).
- **Rehearsal mode** — play a bundled FR/EN sample file through the pipeline to validate
  keys/models before the event without speaking.
- **Billable provider smoke workflow** — an explicitly manual workflow could exercise live
  credentials and a golden audio fixture. It is intentionally not automatic because it costs
  money and CI secrets are not available to forked pull requests.
