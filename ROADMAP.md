# Roadmap

This file combines the current delivery plan with the completed implementation history.
GitHub milestones are the source of truth for active work; the phase checklists below preserve
why earlier architectural decisions were made.

## Current status — 1.5.0 release

The latest GitHub release is **v1.5.0** (22 September 2026). Version 1.3.0 delivered local
transcript history (#81), Stable reading (#79), overlay filler cleanup (#80), and the German
interface. Those three issues are closed. Thanks to @valentinrabot for the meeting feedback.

**1.4.0 adds usability improvements.** It adds meeting profiles,
reading persistence and update pace, appearance previews/presets, history titles and search,
per-source input status, and operator shortcuts. Gemini Smart final results now replace
speculative interim text, including all-filler empty results. Impeccable visual checks also
fixed long history-title overflow and date-picker icon contrast.

See the [release handoff](docs/store-updates.md#release-150-handoff) for verified
checks and remaining native/package acceptance. EN/FR/DE Store copy targets 1.5.0; the last
documented Store release is 1.2.4, with no claim of a new Partner Center submission.
Citation metadata identifies 1.5.0, dated 22 September 2026.

**1.5.0 adds searchable caption languages and favourites (#78).** Provider support, persisted
targets, F2, demo/rehearsal types and RTL captions are reconciled. Live acceptance is still
pending in [language coverage](docs/language-coverage.md); #78 remains open.

**1.4.1 refines the operator interface.** Settings has four tabs in a stable window,
profiles sit above setup with aligned controls, and history uses a list/detail view.
Presets, swatches, localized tray labels and EN/FR/DE date fields complete the UI audit.
The final frontend suite has 337 passing tests. Store acceptance remains pending.

The customizable filter list remains open in [#85](https://github.com/fmadore/Live-translation/issues/85).
No new scroll-back or return-to-live controls are included in this update.

**1.4.2 keeps session controls visible.** Start and Stop share a persistent top bar;
setup and transcripts scroll underneath.

## Historical delivery context

The sections below preserve earlier plans and acceptance criteria. They are not the current
release checklist. Version 1.0.5 first passed certification; 1.1.0 followed on 27 August 2026.
Store updates are submitted manually through [Partner Center](docs/store-updates.md).

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

**Historical post-1.1.0 implementation work.** The list below records what was added
after that package; it does not describe the current Store rollout:

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
- English, French and German layouts are visually checked at the minimum window size and on the
  overlay.
- Tray, graceful quit, Credential Manager, microphone, and loopback behavior pass in the Store
  MSIX on Windows 11.

What that leaves is a **Windows session**, not more code: the Narrator, contrast-theme and
text-scaling walks in [`docs/accessibility.md`](docs/accessibility.md), the tray's keyboard
operation from #22, French screenshots for the Store listing, and a native French speaker's
review. The text-scaling layout is verified at the window's 980 × 660 minimum at every step of
the slider — no clipping, no overflow, no overlap — but verified in a browser preview at a
forced factor, which is not the same as a real slider on a real Windows machine.

## 1.2 — Windows integration and responsive captions

Implemented across the 1.2 releases:

- [#28 — device lifecycle and output selection](https://github.com/fmadore/Live-translation/issues/28):
  implemented; see [audio device testing](docs/audio-device-testing.md) for remaining hardware checks.
- [#27 — application audio capture](https://github.com/fmadore/Live-translation/issues/27):
  included in 1.2.3. Selection was confirmed locally; the full isolation matrix remains pending.
  Closing the selected application stops that capture rather than switching to all system audio.
- [#26 — native Save As and SRT/VTT export](https://github.com/fmadore/Live-translation/issues/26):
  included in 1.2.3. Save As was confirmed locally; complete export verification is recorded
  in [transcript export](docs/transcript-export.md).
- [#77 — responsive caption layout](https://github.com/fmadore/Live-translation/issues/77):
  implemented for 1.2.4; local test installation and release documentation are prepared.
  User feedback, packaged Windows checks and fresh screenshots remain pending.

Release acceptance still includes Teams/Zoom, browser child processes, device changes,
sleep/wake, mixed-DPI displays, application isolation and native transcript exports.
Use the [1.4.2 handoff](docs/store-updates.md#release-150-handoff) for the current checklist.

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
  [#54 — configurable caption width](https://github.com/fmadore/Live-translation/issues/54) —
  **landed**; the measure is an operator control now, and the tail budget that decides how much
  of a long streaming turn is shown moves with it, so widening a caption no longer quietly
  changes how much of the slide it covers — and
  [#55 — operator-chosen typeface, size and colours](https://github.com/fmadore/Live-translation/issues/55),
  whose **typeface half has landed**. The issue left the route open — bundle more `@fontsource`
  families, or offer faces Windows already ships — and it is the system faces, because the two
  objections to them both turned out to be answerable: weight 600 resolves to a real Bold in
  every face offered (measured, not assumed), and a face's presence is checkable with a canvas
  width probe that needs no permission, so a font this machine lacks never reaches the control.
  See `src/lib/captionFont.ts`.

  The **colour half has landed too**, and the trap it carried turned out to be the useful part.
  A ratio measured against the scrim's own swatch is worthless, because the slide shows through
  it — so `src/lib/captionColour.ts` composites the scrim as it is *thinned under the text*
  over both a white and a black slide, then the halo that rings the glyph, then the ink. Two
  things fell out of writing the check that no colour wheel would have surfaced: a fixed black
  halo is the thing that swallows dark ink rather than the thing that rescues it, so the halo
  follows the ink; and sRGB alpha is not perceptually uniform, so the 0.52 that buys the
  lead-in 5.7:1 as white ink buys it only 4.2:1 as black ink — the steps are floors now, dimming
  as far as the design asks or as far as the bar allows, whichever is less dim. The defaults are
  unchanged to the digit, which a test asserts against the literals the stylesheet used to
  carry.

  What both halves were missing was a way to reach them. They rendered only inside a running
  session, so choosing a typeface meant starting one — and the first operator to look for them
  reasonably concluded the build was stale. The controls are a single snippet rendered in two
  places now: the running rail, where the size is what gets nudged mid-session, and a settings
  panel reached from a gear in the title bar, which is the one control whose position does not
  depend on what the session is doing. The panel also collects the interface language and the
  tray preference, which were being rendered into both the rail and the pre-flight sheet. It
  points at placement mode as its preview, because the stand-in caption the overlay already
  shows while being positioned is set in whatever the panel is choosing.
- [#78 — a searchable caption-language dropdown with favourites](https://github.com/fmadore/Live-translation/issues/78)
  is implemented in 1.5.0 with 78 Gemini targets, 13 OpenAI targets, searchable
  localized names and persistent favourites. German (#56) is included in both translation
  lists. DemoLanguage preserves the exhaustive English/French script and fixture matches.
  F2 swaps the first two supported favourites; unsupported targets block Start. Live endpoint
  and speech acceptance are tracked in [language coverage](docs/language-coverage.md).
- **The interface is now English, French and German.** `src/lib/i18n/de.ts` is a full catalog,
  the selector is built from `LOCALES` so it needed no markup, and `detectLocale` matches on the
  primary subtag, so `de-AT` and `de-CH` open in German. This is independent of the caption
  language, as [`docs/localization.md`](docs/localization.md) has always insisted: a German
  interface does not imply a German caption target, and a German caption target is #78. Still
  owed before it can be advertised: German Store copy, German screenshots, and a native-speaker
  review — the same gate French is waiting on.
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
certification history; [`docs/store-updates.md`](docs/store-updates.md) has the update
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
- **More target languages** — implemented in the working tree via #78: searchable, provider-scoped
  targets and favourites, separate demo/rehearsal types. Live acceptance is tracked in
  `docs/language-coverage.md`. Supersedes the earlier proposal for a select
  instead of two buttons. Superseded by
  [#56](https://github.com/fmadore/Live-translation/issues/56).
- **Latency metrics** in the operator monitor (audio-sent → first-delta round trip).
- **Session cost estimate** (audio minutes streamed per provider).
- **Rehearsal mode** — play a bundled FR/EN sample file through the pipeline to validate
  keys/models before the event without speaking.
- **Billable provider smoke workflow** — an explicitly manual workflow could exercise live
  credentials and a golden audio fixture. It is intentionally not automatic because it costs
  money and CI secrets are not available to forked pull requests.
