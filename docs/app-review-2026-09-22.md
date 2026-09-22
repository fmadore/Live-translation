# App review — 22 September 2026

Scope: a read-only review of version 1.5.0 covering refactoring, efficiency and visual design.
The review covered the Rust core, the frontend state modules, the Svelte component layer, CSS
and design tokens, the build and CI. The operator and overlay windows were also inspected in a
browser preview. Nothing was changed. Findings marked **verified** were rechecked against the
code by a second pass. Findings marked *unmeasured* rest on reading the code, not on profiling.

Baseline: `npm test` passes 356 tests in 42 files, and `npm run check` reports 0 errors and 0
warnings. `cargo clippy -W clippy::pedantic` reports 137 lib warnings; the meaningful ones are
listed under R-refactors.

The 5 September review ([archive](archive/app-review-2026-09-05.md)) closed its items. Nothing
below repeats them.

## Implementation tracker — batch 1

Batch 1 is the quick, low-risk group from §6. Status is updated as each item lands; notes
record what changed and how it was verified.

| Item | Change | Status |
| --- | --- | --- |
| D2 | Only the topmost modal handles Escape/Tab | Done |
| D5 | `.env` and host overrides debug-only; per-provider settings | Done |
| D6 | Microphone and typeface selects use `ui/Select` | Done |
| D9 | Rail width cap reaches child components | Done |
| D3 (part) | Snapshot built in the queued job; history list refresh not per line | Done |
| E1 | `crate-type = ["rlib"]` | Pending release-build check |
| E2 | Tray skips unchanged labels; command runs on the main thread | Done; desktop check pending |
| E4 | Borrowed caption text; no copy of Gemini binary frames | Done |
| E5 | Activity notes throttled per source | Done |
| E6 | History search index normalised once per session | Done |
| E8 | Bundle `.woff2` fonts only | Done |
| Dead code | `latestCaption`, `prepare_ondevice_model`, `const open`, duplicate locale push | Done |

### Batch 1 implementation notes

- **D2** — [`ModalPrompt.svelte`](../src/lib/ModalPrompt.svelte) keeps a module-level stack of
  open dialogs. Each dialog registers on mount and removes itself on teardown, and its window
  key handler returns early unless it is on top. The handler moved from `on:keydown` to
  `onkeydown`. The new [`ModalPrompt.svelte.test.ts`](../src/lib/ModalPrompt.svelte.test.ts)
  stacks a dialog under the unsaved prompt and covers three cases: Escape answers only the top
  dialog, a mid-cycle Tab is left alone, and the lower dialog gets the keyboard back when the
  top one closes. Two of the three tests fail with the guard removed.
- **D5** — `session.rs` gains `ProviderSettings::resolve(provider)`, which reads only the
  selected provider's variables. `*_WS_HOST` is honoured only when `debug_assertions` is on.
  Model overrides still apply in release, so a renamed model can be pinned without a new
  build. `lib.rs` loads `.env` in debug builds only, and `.env.example` says so. Three unit
  tests cover the fix: a malformed Mistral delay no longer blocks Gemini, model overrides and
  defaults combine, and host overrides depend on build type. The provider arms now match on
  the settings.
- **D6** — The microphone select is now `Field` + `Select`. It has a visible "Microphone
  device" label, like the output picker beside it. The typeface select is `Select` inside its
  existing label. Both duplicated `.select-row`/`.chevron` blocks and their weaker focus ring
  are gone. In the browser preview the selects are 38 px and 35 px tall (the typeface select
  was 12.5 px), with the shared 2 px `--focus` ring.
- **D9** — The cap is `.rail > :global(*)` with `max-width: 23.75rem`. Browser verification
  showed that the review's `:global` fix alone was not enough. `MeetingProfiles` sets a 12 px
  font, so `23.75em` capped it at 285 px beside 380 px siblings. `rem` resolves to the rail's
  own column width for every child. In the two-column layout at 1200 × 820, all rail children
  measure 325 px, the same as before.
- **D3 (part)** — `persist()` captures the session and takes a shallow snapshot when its queued
  job runs, so superseded appends copy nothing. `TranscriptHistory.svelte` refreshes on a
  revision at most every 5 s. Mount, Refresh, rename and delete still refresh immediately. A
  new fake-timer test sends five writes in a burst and expects one re-list after 5 s; it fails
  with the throttle set to zero. The dead `open` constant and its wrapper block were removed.
  The write throttle and JSONL format stay in batch 2.
- **E1** — `Cargo.toml` builds only `rlib`. The release-build timing and artefact check are
  recorded below once complete.
- **E2** — `tray.rs` remembers the text and enabled state last written to each item and skips
  unchanged writes. During a session this means one status write per second instead of six.
  `set_tray_state` is now a sync command: Tauri runs those on the main thread, and
  `tauri-runtime-wry`'s `send_user_message` runs main-thread tasks inline, so there are no
  blocking round trips. The frontend is unchanged, because the core now does the
  de-duplication. The native tray still needs a desktop run.
- **E4** — `Caption<'a>` borrows the turn's text for the moment it is serialized, and binary
  frames are parsed with `std::str::from_utf8(&bytes)`.
- **E5** — `noteActivity` ignores a repeat of the same source and kind within 500 ms. The
  labels it feeds use a 3 s threshold.
- **E6** — `historySearch.ts` caches each listed session's NFKC-folded text in a `WeakMap`.
  Listed sessions are never mutated, because a rename re-lists them.
- **E8** — A `woff2-only` pre-transform in `vite.config.js` drops fontsource's `.woff`
  fallbacks before Vite resolves their URLs. The build output went from 1.2 MB (54 font files)
  to 782 KB (27 `.woff2`). In the browser preview, the five weights in use load from `.woff2`.
- **Dead code** — Removed:
  - the `latestCaption` store and its five writes;
  - the `prepare_ondevice_model` command, its permission, command-name entry, `tauri.ts`
    wrapper, `ondevice::prepare` and its no-op test;
  - the redundant `onMount` overlay push. The two language effects became one, so load sends
    one full overlay config instead of three.

Verification on 22 September 2026:

| Check | Result |
| --- | --- |
| `npm test` | 360 passed in 43 files (4 new) |
| `npm run check` | 0 errors, 0 warnings |
| `npm run format:check` | Passed |
| `npm run build` | Passed; 27 `.woff2`, no `.woff` |
| `cargo fmt --check` | Passed |
| `cargo clippy --locked --all-targets --all-features -- -D warnings` | Passed |
| `cargo test --locked --all-features` | 76 passed, 2 ignored (3 new, 1 removed with `prepare`) |
| Browser preview (operator, EN, 1200 × 820 and stacked) | Selects, focus ring, rail widths, fonts and Escape verified; no console errors |

## 1. Defects found while reviewing

These are behavioural problems, not style issues, so they go first.

| ID | Priority | Finding | Suggested change | Effort |
| --- | --- | --- | --- | --- |
| D1 | High | **Anti-alias filter muffles speech** (verified). [`audio/resample.rs:21-24`](../src-tauri/src/audio/resample.rs) cascades four one-pole low-passes at 0.45 × output rate. For 48 → 16 kHz (Gemini, Gemini Transcribe, Mistral): −2.6 dB at 3 kHz, −4.3 dB at 4 kHz, −10.4 dB at 7 kHz. Aliasing content at 10–12 kHz is only down 16–20 dB. Providers receive roughly telephone-bandwidth audio with folded sibilants. The effect on recognition accuracy is *unmeasured*. | Replace with 2–3 Butterworth biquads (4th/6th order), or a 32–64-tap polyphase FIR that also replaces linear interpolation (48→16 and 48→24 are integer ratios). Add passband/stopband tests at 2/4/6 kHz and 10/12/15 kHz. Compare against the rehearsal fixtures before shipping. | M |
| D2 | High | **Nested dialogs trap the keyboard** (verified). Every [`ModalPrompt`](../src/lib/ModalPrompt.svelte) installs its own window `keydown` handler (line 90). Nothing closes Settings when the window's X or tray Quit opens a quit prompt. With both open, one Escape dismisses both. Tab is pulled into Settings and then back to the prompt's first button, so **Discard is unreachable from the keyboard**. | Keep a module-level stack in `<script module>` so only the topmost modal handles keys. Alternatively, close Settings before any quit prompt opens. Migrate `on:keydown` to `onkeydown` at the same time. | S |
| D3 | High (opt-in feature) | **History write amplification** (verified). [`history.ts:129-135`](../src/lib/history.ts) copies the line array twice per final caption. `persist()` then serialises and fsyncs the whole session file each time. Over a 2,000-line (≈3 h) session that is ≈0.5 GB written; at 5,000 lines ≈3.3 GB. With the History tab open, every write also bumps `historyRevision`, and [`TranscriptHistory.svelte:83-88`](../src/lib/TranscriptHistory.svelte) re-lists and decodes every stored session. | Build the snapshot inside the queued job. Throttle append-triggered writes to ≈5 s, with `finish`/`flush` forcing a write. Bump the revision only on begin/finish/rename/delete/retry. A later option is an append-only JSONL format. | S → L |
| D4 | Medium | **Stable reading keeps unbounded overlay history** (verified). In Stable mode, [`overlay/+page.svelte:263-266`](../src/routes/overlay/+page.svelte) appends every turn with no cap, and `scheduleExpiry` returns early (line 181). History is only cleared when the whole session stops. `lines` re-runs `cleanSpeech` over the full history on every caption (≈4 ms at 187k characters vs 0.2 ms at the 12k Fit cap). The overlay then re-lays out one large `<p>`. | Cap Stable history in large whole-turn chunks, enough for a few viewports. Move the cleaned lead into its own `$derived` that depends only on `history[origin]` and `hideFillers`. | S–M |
| D5 | Medium | **Dev overrides active in release builds** (verified). [`lib.rs:86`](../src-tauri/src/lib.rs) calls `dotenvy::dotenv()` unconditionally. That search covers the working directory and its parents. [`session.rs:183-204`](../src-tauri/src/session.rs) honours `*_WS_HOST`, so a stray `.env` can redirect an API key to another host. Those lines also read all nine provider variables for every provider: a malformed `MISTRAL_TARGET_STREAMING_DELAY_MS` stops a Gemini session from starting. | Gate `.env` loading and host overrides behind `cfg(debug_assertions)`. Resolve only the selected provider's settings (`ProviderSettings::resolve(provider)`). | S |
| D6 | Medium | **Weak focus ring on two selects** (verified). `.select-row select:focus-visible` in [`+page.svelte:2174`](../src/routes/+page.svelte) and [`CaptionAppearance.svelte:291`](../src/lib/CaptionAppearance.svelte) replaces the shared ring with `--accent-border` (42 % alpha), about 2.7:1 over `--panel-2`. The CSS block is duplicated verbatim in both files. Its padding sits on the wrapper, so the clickable select is only **12.5 px tall** in a 34 px box. | Use the existing [`ui/Select`](../src/lib/ui/Select.svelte) primitive for the microphone and typeface selects, and delete both `.select-row` blocks. | S |
| D7 | Medium | **Sessions ended by the core are not finished in history.** `applyStatus` settles the clock when the last source ends ([`stores.ts:89-108`](../src/lib/stores.ts)) but never calls `sessionHistory.finish()`. After a provider failure, the next Start/Stop/quit stamps `endedAt`, which inflates the recorded duration. The end-of-session pair is also duplicated in `stores.ts`, `sessionController.ts` and `quit.ts`. | One `endTranscriptSession()` in `stores.ts`, called from all three places and when `isRunning` drops to false. Confirm the intended behaviour first. | S |
| D8 | Low | **Transcript monitor state resets on Start/Stop.** `TranscriptMonitor` is mounted in two branches ([`+page.svelte:1345, 1510`](../src/routes/+page.svelte)), so the chosen export format and follow/scroll state are lost at every transition. | Hoist to one instance outside the `{#if $isRunning}` branch. | S |
| D9 | Low | **Rail width cap misses child components** (verified). `.rail > *` ([`+page.svelte:1936`](../src/routes/+page.svelte)) is scoped. `MeetingProfiles` therefore stretches to full width in the stacked layout at larger Windows text sizes, while its siblings stay at 23.75 em. The same trap will catch every component extracted from the rail. | `.rail > :global(*)`. Do this before any extraction. | S |
| D10 | Decision | **`panic = "abort"` disables the recovery paths the code relies on.** The `join().is_err()` handling ([`session.rs:551, 583`](../src-tauri/src/session.rs)), `JoinError` paths and poison recovery are dead in release. A panic in any capture/provider thread kills the process, and the transcript lives in the renderer (the recovery spool is off by default). | Drop `panic = "abort"` (≈ a few percent binary size, *estimate*), or keep it and document the trade-off. | S |

## 2. Efficiency

| ID | Finding | Change | Effort |
| --- | --- | --- | --- |
| E1 | **Release builds link three crate types.** `crate-type = ["staticlib", "cdylib", "rlib"]` ([`Cargo.toml:17`](../src-tauri/Cargo.toml)) is the mobile template default. Each release build does separate fat-LTO links, including a 152 MB unused `.lib` and an unused DLL. | `crate-type = ["rlib"]` (the app is Windows-only). Build-time gain only. | S |
| E2 | **Tray menu rewritten every second** (verified). The effect at [`+page.svelte:311-323`](../src/routes/+page.svelte) depends on `elapsedMs`. Each tick sends all six labels. Each `set_text` is a blocking main-thread round trip made from a Tokio worker while a `std::Mutex` is held ([`tray.rs:75-100`](../src-tauri/src/tray.rs)). | Send labels only when the locale or overlay changes, and send the status line separately. In Rust, cache the last-applied values and make the command synchronous. | S |
| E3 | **Caption fitting lays out oversized candidates.** `fitCaptionTail` binary-searches over up to 12k characters of lead plus the current turn. That is about 12 `getBoundingClientRect` calls per origin per partial caption, and the first probes are 6k+ character paragraphs. `lineHeight` is re-read via `getComputedStyle` every run ([`OverlayCaptionLine.svelte:36-53`](../src/routes/overlay/OverlayCaptionLine.svelte)). Frame cost is *unmeasured*. | Pre-cut to an estimated tail from box area ÷ glyph area × 2 before searching. Cache line height per `fontKey`. | S |
| E4 | **Caption strings cloned on every emit.** `realtime.rs:503-504` clones both accumulated strings per interim caption, so the cost grows quadratically with turn length. Gemini binary frames are also copied before parsing ([`realtime.rs:418`](../src-tauri/src/realtime.rs), `bytes.to_vec()`). | Borrow `&str` in `Caption`. Use `std::str::from_utf8(&bytes)`. | S |
| E5 | **Activity bookkeeping at audio-meter rate.** `noteActivity` runs on every level event (20–40 Hz, [`+page.svelte:250`](../src/routes/+page.svelte)), but the labels it feeds change on a 1 s tick with 3 s thresholds. | Skip if the last note for that source is < 1 s old. | S |
| E6 | **History search re-normalises every line on every keystroke** ([`historySearch.ts:17`](../src/lib/historySearch.ts)). With 50 sessions that is ≈85 ms per keystroke. | Cache a normalised search string per session at decode time. | S |
| E7 | **Every transcript paragraph stays in the DOM.** The monitor also re-groups the full array per final line. At 5,000 lines the grouping is only 0.19 ms; the DOM size is the real cost (*unmeasured*). | Measure first. Then render the recent tail with a "show earlier" control, or use `content-visibility: auto` on paragraphs. | M |
| E8 | **348 KB of `.woff` fonts ship but are never loaded.** WebView2 always takes the `.woff2`. There are 54 font files for 7 weights × 3 subsets × 2 formats. | Declare `@font-face` for `.woff2` only, or switch Archivo to `@fontsource-variable/archivo` (one file per subset for all weights). | S |
| E9 | **Smaller Rust hot-path items.** The redundant `resampled` buffer in `capture.rs:269, 340` (the resampler already appends). Loopback `get_state()` runs at ≈100 Hz (`loopback.rs:165-174`). The overlay keep-on-top loop makes two main-thread round trips every 500 ms (`overlay.rs:107-114`). The keychain read blocks the async runtime inside the lifecycle lock (`session.rs:176`). | Write straight into `pending`. Throttle the state checks to 1 Hz. Fold the visibility check into the main-thread closure. Use `spawn_blocking` for the keychain. | S each |
| E10 | **Dev loop.** Vitest takes 41 s, 67 % of it environment setup across 42 isolated workers. CI's `rust` job `needs: frontend`, although it builds the frontend itself. | Try `isolate: false` on the `logic` project only (*unverified*; module-level stores may leak between files). Drop the `needs` so the Windows Rust lane starts immediately. The `windows-speech-v1` cache key and comment are stale. | S |

**Dead code:**
- `latestCaption` ([`stores.ts:149`](../src/lib/stores.ts)) is written on every caption and never read.
- `prepare_ondevice_model` is still registered and has a `tauri.ts:65` wrapper, but has no caller.
- `const open = true` in `TranscriptHistory.svelte:27`.
- The locale is pushed twice on mount (`+page.svelte:240, 474-478`).

## 3. Refactoring

### Rust core

- **R1 — Make the provider handlers pure.** Each client calls `emit_caption`/`next_turn` itself (`gemini/client.rs:115`, `openai/client.rs:113`, `mistral/client.rs:98`, `gemini/transcribe.rs:114`). Because they need an `AppHandle`, **Gemini Translate, OpenAI and Mistral handlers have no tests**. That includes Mistral's delta/done duplication guard. Change: return `CaptionUpdate { None, Interim, Final }` in `MessageOutcome` and let the runner emit. Optionally coalesce interims to ≈10 Hz there. M; risk medium (ordering), so add the tests in the same change.
- **R2 — Split `SessionManager::start`** ([`session.rs:147-415`](../src-tauri/src/session.rs), 269 lines). Five near-identical provider arms become a generic `spawn_realtime<P: RealtimeProtocol>`. The capture-thread `match`, level forwarder and join blocks are duplicated in `start_test` and become `run_capture`, `spawn_level_forwarder` and `join_threads`. Pairs with D5. M.
- **R3 — Small shared helpers, not a new abstraction.** `realtime.rs` already holds the connect/backoff/drain skeleton well. The remaining ≈50 duplicated lines become `bearer_request()`, `gemini_request()`, `parse_or_log<T>()` and `ServerMessage::control()`. The `export.rs` `atomic_write` duplicates `recovery.rs` `replace_snapshot`. The ten microphone sample-format arms (`capture.rs:133-229`) become one generic `build::<T: SizedSample>`. S.
- **R4 — Error consistency.** `recovery.rs`, `history.rs` and `lifecycle.rs:132` return English `String` errors instead of `AppError` ids. `{path:?}` shows operators doubled backslashes (`recovery.rs:110, 113, 147, 151, 177, 179`); use `.display()`. M.
- **R5 — Naming.**
  - Replace `MessageOutcome`'s two booleans with an enum.
  - `MistralConfig.received_delta` is runtime state on a config struct.
  - `TurnAccumulator.translated` also holds transcription text.
  - `emit_status` and the COM `Apartment` guard are each duplicated.
  - The crate doc in `lib.rs:1-5` still says Gemini-only.
  - Pedantic clippy flags `capture.rs:99` (138 lines), `realtime.rs:266` (122 lines, which splits cleanly into `await_setup` + `pump`) and `loopback.rs:67` (102 lines).

### Frontend state

- **R6 — One appearance schema.** The eight appearance settings are listed by hand in about seven places: controller push/reset, `CaptionAppearance`'s defaults check, profile save/load, `profiles.ts`, `OverlayConfig` (with different field names) and the overlay's init. The defaults (38, 30, 4, `'fit'`, `'archivo'`) are repeated as literals, so adding one setting takes about ten edits. Change: `DEFAULT_APPEARANCE`, `normalizeAppearance()` and `toOverlayConfig()`, ideally backed by one `appearance` store. Presets move to a `PRESETS` table shared with `CaptionPreview`. M.
- **R7 — A `persisted(key, parse, serialize)` helper.** It replaces 13 `writable(loadX())` + `subscribe(setItem)` pairs in `stores.ts` and five identical boolean loaders, and applies one try/catch policy (today only `languageFavourites` has one). S.
- **R8 — Decouple history from the recovery format.** `decodeSession` parses twice and validates against `RECOVERY_VERSION` ([`history.ts:33-34`](../src/lib/history.ts)). Bumping the recovery format would make every saved history session read as "unreadable". Change: export `readLines(unknown)` from `document.ts`, with each decoder checking its own version. S.
- **R9 — Contract tests for shared names.** `errors.test.ts` already cross-checks error ids against Rust. Add the same `readFileSync` check for `tauri.ts` command names against `command_names.rs`, `EVT` against `types.rs` events, and the TS unions against the serde enums. Also reuse `Locale`/`ORIGINS`/mode lists instead of re-declaring them. S.
- **R10 — Controller classes.** The three `.svelte.ts` controllers return long hand-written getter lists. Classes with `$state` fields remove about 100 lines. The store/runes mix itself is fine: keep shared state in stores and page-scoped state in runes. S/M.

### Component layer

- **R11 — Split `+page.svelte`** (2,846 lines: script 553, markup 1,130, **style 1,163**). Prerequisites: move the repeated utility classes to `app.css` and apply the D9 `:global` fix. The repeated classes are `.kicker` (3 definitions), `.hint` (6 definitions in 5 variants), `.divider`, `.key`/`kbd` (3 styles) and `.grow`. Suggested order, cheapest first:

  | Unit | Moves | Risk |
  | --- | --- | --- |
  | `SettingsDialog.svelte` + `Tabs` primitive | 578-639, 1543-1648, ≈215 CSS lines; removes the `% 4` magic numbers | low |
  | `OperatorTitlebar.svelte`, `SessionControls.svelte`, `DeviceRecoveryBanner.svelte`, `LiveTurns.svelte` | ≈450 lines together | low |
  | `PreflightChecklist.svelte` + `ChecklistRow.svelte` | 1351-1537 plus `ApiKeyPanel`'s copy of the row styles | low–medium |
  | `LiveRail.svelte` | 803-961, ≈310 CSS lines | low–medium |
  | `SetupSheet.svelte` + `ChoiceButton` | 962-1281, ≈570 CSS lines | medium |

  Script modules follow the existing controller pattern: `sessionClock`, `deviceFailure`, `recoveryOffer`, `nativeSync` and `setupActions`. The page should end at about 500–600 lines. `deviceRecovery.svelte.test.ts` renders the whole page and will catch regressions.
- **R12 — Split the overlay route** (939 lines):
  - `overlayCaptions.svelte.ts`: per-origin state, timers, presenter and `lines`. This makes it testable without font and `ResizeObserver` stubs.
  - `overlayPlacement.ts`: window moves and the key map.
  - `OverlayMoveChrome.svelte`.
  - `overlayFixtures.ts`.

  `captionColour.test.ts:310` reads these files by path; update its list.
- **R13 — Use the primitives that already exist.** `ui/` primitives are used in only four components.
  - **Stepper:** built by hand in `CaptionAppearance` and the overlay toolbar. It needs `format` and button-label props first.
  - **Select/Field:** the microphone select has no visible label; also the typeface select and `SystemCapturePicker`'s three hand-built fields.
  - **Preference:** the close-to-tray and recovery checkboxes.
  - **ToolButton:** the ghost button is defined three times identically. It needs `variant`, `size` and `wide` props.
  - **New primitives:** `ChoiceButton` (the two mode cards are identical 36-line blocks), `ChecklistRow`, `Icon` (21 inline SVGs in `+page.svelte`; the check path is copied six times) and `Kbd`.
- **R14 — Effects that should be derived values.**
  - `LanguagePicker.svelte:43-46` writes state it reads.
  - `+page.svelte:148-150` mirrors a global `hasKey` store that only this page uses.
  - `DateField.svelte:8-11`.
  - The Start button's disabled logic is written three slightly different ways; make one `canStart`.

## 4. Visual design

The operator UI is coherent and carefully accessible: a single focus ring, forced-colours
support, a contrast-tested palette, and an em-based container query that follows Windows text
size. The opportunities are mostly consolidation. The design system has grown by accretion,
and the tokens no longer describe what they render.

- **V1 — Collapse the type ramp.** `app.css` defines 16 `--type-*` tokens, but four of them
  (`--type-9-5`, `-10`, `-10-5`, `-11`) all render 11 px, so the pixel names are wrong. The idle
  operator screen renders **12 distinct sizes**. Half-pixel pairs (11/11.5, 12/12.5, 13/13.5)
  are indistinguishable at a glance and read as inconsistency rather than hierarchy. Proposal:
  seven role tokens, updating `typeScale.test.ts` to enforce the smaller set.

  | Role | Size |
  | --- | --- |
  | `caption` | 11 |
  | `small` | 12 |
  | `body` | 13 |
  | `label` | 14 |
  | `title` | 17 |
  | `heading` | 21 |
  | `display` | 27 |
- **V2 — Collapse the grey ramp and remove legacy aliases.** Text uses seven greys, including
  near-duplicates:
  - `--muted` `#8b93a1`, `--muted-2` `#848c99` and `--muted-3` `#7d858f`;
  - `--text-soft` `#c3c9d2` and `--text-dim` `#b9c0ca`.

  `--accent-2`, `--accent-soft` and `--focus` are all `#7fdcb6`. `--panel`/`--panel-2` and
  `--hairline`/`--border-2` differ by 3 RGB steps. Proposal: four text levels (bright, body,
  secondary, muted ≥ 4.5:1), three surfaces and two lines. Then delete the legacy names; the
  `app.css` header already calls them legacy.
- **V3 — Add spacing and radius tokens.** Components use 22 distinct px spacing values alongside
  25 rem/em values, and 9 hard-coded radii (3–20 px) beside the two radius tokens. A
  4-px-based scale (`--space-1…6`) and a `--radius-sm` for chips/kbd would absorb nearly all
  of them.
- **V4 — Bring overlay chrome under the palette tests.** Move-mode styles in
  `overlay/+page.svelte:686-920` hold 27 hex literals, at least 10 of them exact copies of
  tokens. They sit outside `palette.test.ts`'s contrast guard. The comment says the overlay
  "does not inherit the operator's surfaces", but `+layout.svelte` already loads `app.css`
  there.
- **V5 — One toolbar instead of three horizontal strips.** Under the native Windows title bar
  (which already says "Live Translation & Subtitles"), the app draws its own title bar with the
  same name, a subtitle, the status pill and the gear. Below that comes the Start/Rehearse bar.
  That is ≈140 px of chrome on a 660 px minimum-height window. Proposal: one bar:

  `[brand mark] [Start] [Rehearse]   …   [status pill + elapsed] [⚙]`

  Status then sits next to the control that changes it, and the stage gains ≈40 px.
- **V6 — Attach the keyboard hint to its action.** The `Ctrl Shift Space` chip floats after
  Rehearse and reads as a third button. It is also hard-coded English, written in two formats
  across `+page.svelte:786` and `KeyboardHelp.svelte:9-15`. Show it as secondary text inside the
  Start button or its tooltip, add `aria-keyshortcuts`, and source it from `shortcuts.ts` and
  i18n.
- **V7 — One primary and one ghost button.** The mint gradient primary is defined three times
  (`.start`, `ApiKeyPanel .save`, overlay `.primary`), while `ModalPrompt`'s primary is flat
  `--accent`. The ghost button is defined three times. Disabled opacity is 0.5 globally and 0.45
  in two places. Folding these into `ToolButton` variants (R13) settles them.
- **V8 — Smaller polish items:**
  - **Colour swatches:** screen readers announce raw hex ("#fff0b3"), and the selected state is
    drawn exactly like the focus ring. Use named colours and a distinct selected mark (check or
    inner ring).
  - **Small targets:** the typeface select is 12.5 px tall (fixed by D6), Reset is 24 px and
    Move overlay is 27 px. Aim for ≥ 32 px in a mouse-first desktop app.
  - **Dialog breakpoints:** settings-dialog breakpoints use viewport px (`CaptionAppearance:234`,
    `TranscriptHistory:365`). The dialog is 52 em wide, so at 225 % text size those breakpoints
    fire at the wrong point. Declare a container on `.prompt` and use em queries.
  - **Label mismatch:** the profile picker's visible label is "Meeting profiles" but its
    accessible name is "Choose a profile" (WCAG 2.5.3).
  - **Store screenshots:** these predate 1.4.x (the old app name, bottom Start button and
    single-page Settings). This is already tracked in `docs/store-screenshots/README.md`.

## 5. Keep as is

- The shared realtime runner: setup, jittered backoff, fatal-HTTP classification, stale-audio
  drain and the drop guard.
- Bounded `try_send` from audio callbacks.
- Staged file replacement.
- The lean dependency set and the size-tuned release profile.
- The newest-first transcript array: it costs 0.11 ms per prepend at 5,000 lines, and
  `saveDocument` relies on replacement.
- The recovery coordinator.
- The `.svelte.ts` controller pattern with explicit `dispose`.
- The type-scale and palette tests.
- Keyed `{#each}` everywhere.
- `LevelMeter`'s transform-only animation.
- Forced-colours support.
- `Messages = typeof en` plus the Rust error-id contract test.
- Tauri only delivers events to webviews that listen, so the overlay does not pay for 20 Hz
  meter events.

## 6. Suggested batches

1. **Quick fixes, low risk:**
   - D2 modal stack;
   - D5 env/`.env` gating;
   - D6 via `ui/Select`;
   - D9 `:global`;
   - E1 crate-type;
   - E2 tray split;
   - E4;
   - E5;
   - E8 fonts;
   - dead code;
   - the D3 revision/snapshot part plus E6.
2. **Audio and persistence:**
   - D1 filter, with tests and a listening/ASR comparison on the rehearsal fixtures;
   - D3 write throttle;
   - D4 Stable cap;
   - D7;
   - D10 decision.
3. **Structure:**
   - R1 with the missing provider tests;
   - R2;
   - R6;
   - R7;
   - R9;
   - R11/R12 in the order given.
4. **Design system:**
   - V1–V4 token consolidation, one PR, visual diff in EN/FR/DE at 100 % and 225 %;
   - V5–V7 layout and buttons;
   - V8.
