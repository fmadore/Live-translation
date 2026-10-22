# App review — 22 October 2026

Scope: a read-only review ahead of the release that ships the light theme (#159, the System /
Light / Dark setting). It covers the theme itself, then the whole app again for defects,
security, efficiency, refactoring, testing, CI and the release process. Items marked Done in
the [6 October review](app-review-2026-10-06.md) are not repeated unless they regressed. Nothing
was changed.

**Status (22 October 2026):** the maintainer chose to keep System as the default and announce it
(L3). Batch 1 (§9) is on `fix/light-theme-before-1.9.0`: L1, L2, L4's guard, L5, L6 (glow and
preview edge), L7, L9, L10, L11 and C1, with the 1.9.0 release body and What's new. L6's meter tip
and text-field borders were left as they are: the light `--warn` is the theme's amber, and the
field edges predate the theme.

Labels: **verified** findings were traced in the code; **plausible** ones need a run, hardware
or a live provider to confirm. **Rechecked** means the orchestrating pass confirmed the finding
independently (in the code, the built app, the browser preview or GitHub) after a reviewer
reported it.

## Baseline

The local checks ran on `9741ccc` (the tip of `feat/light-theme`). `origin/main` is `03da24d`,
the merge of #159, which adds only #158 (`TranscriptHistory.svelte`, +9 lines) and a docs
commit on top. CI on `03da24d` passed all nine jobs. Local `main` is three commits behind
`origin/main`, and the remote `feat/light-theme` branch is deleted.

| Check | Result |
| --- | --- |
| `npm run lint`, `check`, `format:check`, `knip`, `check:languages` | Pass; 0 errors, 0 warnings |
| `npm run test:coverage` | 722 passed, 1 skipped, 75 files. Lines 88.4%, branches 82.4%, statements 87.2%, functions 84.6% |
| `npm run build`, then `npm run test:style` | Pass; 130 snapshot tests |
| `npm audit --omit=dev` | 0 vulnerabilities |
| `cargo clippy --all-targets --locked -D warnings` (aarch64) | Clean |
| `cargo test --locked` | 239 passed, 5 ignored |
| `cargo fmt --check` | Clean |
| Browser preview (`npm run dev`, 1200×820, Light stored) | Operator window, settings dialog and Theme picker render; the overlay keeps the dark tokens and `color-scheme: dark` with `appearance.theme = light` in storage |

**Overall.** The app is in good shape: no High defects in the core, every gate is green, and the
6 October batches are holding. The theme's structure is sound: one attribute, a token block,
both themes under the contrast tests, and the overlay kept apart. What stops it shipping as it
is comes down to four things, all small:

- In Light, a selected card or tile barely differs from an unselected one (L1).
- A likely dark frame at startup on light desktops (L2).
- An unannounced change of look for existing users whose Windows is in light mode (L3).
- A release workflow that publishes a tag before its Store packages are built (C1). Without
  release candidates, nothing tests those packages before then.

---

## 1. Light theme

| ID | Priority | Finding | Suggested change | Effort |
| --- | --- | --- | --- | --- |
| L1 | **High** | **In Light, selection is weak, and hover looks more selected than the selection** (verified; rechecked in the preview). See the note below the table. | In the light block, set `--accent-border` to about `rgba(11, 117, 80, 0.75)` (≈3.4:1 on white). Optionally give pressed `.ui-card`/`.ui-tool` an `inset 0 0 0 1px var(--accent-border)` in both themes. In `palette.test.ts`, require the selected edge, composited on `--surface-0`/`-1`, to contrast more than `--line-hover` (Dark passes 2.54 > 1.86; Light fails 1.60 < 2.58 today). | S |
| L2 | Medium | **Likely dark frame at startup on a light desktop** (structure verified; the flash is plausible). With `ssr = false`, the built `index.html` is an empty body, a render-blocking `0.css` in `<head>`, and a bootstrap that awaits two imports before mounting. `followTheme()` runs before Svelte's first paint but not before the shell's. In that frame `body` is `--surface-0` at its **dark** value. The sequence: a light title bar, then a white WebView, then dark, then light. With an explicit choice that differs from Windows, the title bar also flips once `setTheme` lands. | An inline `<script>` in `src/app.html` `<head>`. It skips `/overlay/`, reads `appearance.theme` in a try/catch, falls back to `matchMedia('(prefers-color-scheme: light)')`, and sets `data-theme` on `<html>`. Tauri hashes inline scripts into the CSP at build time; the SvelteKit bootstrap already relies on this. Add a test that the key in `app.html` matches `THEME_KEY`. The fuller fix, which also removes the white frame and the title-bar flip: `visible: false` on the operator window, then `show_operator` after the first render, with a Rust fallback timer. | S (M) |
| L3 | Medium | **Existing users on light app mode get a light window on update, unannounced** (verified). `loadTheme()` defaults to `system`, and the Windows 11 default app mode is light. The Store listing and screenshots all show dark, and `docs/store-screenshots/README.md` doesn't pin a theme, so a retake on a light-mode PC will come out light. | **A decision for the maintainer:** keep `system` and say so in What's new ("choose Dark to keep the previous look"); or migrate existing installs to `dark` (write `dark` when any other preference key already exists, before `persisted` saves `system`). Either way, pin the theme in the screenshot plan. | S |
| L4 | Low (latent) | **A theme change reaches both windows** (verified against the tao, tauri-runtime-wry and wry sources). `setTheme` ends in `ICoreWebView2Profile::SetPreferredColorScheme`, and the profile is shared, so choosing Light or Dark flips the overlay's `prefers-color-scheme` too. No visible effect today: no `src` CSS uses that media query, and the overlay's `color-scheme` is single-valued. One side effect: going from an explicit choice back to System applies a stale `matchMedia` for one IPC round trip (a brief wrong-theme flash). | Say so in `theme.ts`. Add a test that fails on any `prefers-color-scheme` in `src/**`, so a future rule can't leak into the overlay. Optionally drive System in Tauri from `getCurrentWindow().theme()`/`onThemeChanged`. | S |
| L5 | Low | **The toolbar badge is no longer the app icon in Light** (verified). `OperatorToolbar.svelte:108-126` paints the mark with palette tokens, so in Light it becomes a pale tile with a dark-green wave: the inverse of `source.svg`, which sits in the title bar just above it. | Theme-invariant brand tokens with the `source.svg` values (tile #191d23→#0e1013, wave #5ad1a0, bar #e9ebef), outside what the re-declare test requires. | S |
| L6 | Low | **Smaller Light rendering issues** (code verified; look plausible). See the list below the table. | Adjust each in the light block or the component. | S |
| L7 | Medium | **The feature has no rendered or behavioural coverage** (verified). `e2e/style.config.ts` pins `colorScheme: 'dark'`, so all 130 snapshots are dark. `followTheme()` is untested: no test stubs `matchMedia`, and nothing covers the live switch, unsubscribing or the title-bar call. The picker has no component test. `titleBar()` swallows every error (`theme.ts:79-86`), so a missing `core:window:allow-set-theme` would pass silently. The re-declare test compares names only, so pasting the dark values passes. It also misses tokens written as `#fff`, 8-digit hex, `rgb()`/`hsl()`, named colours or `inset` shadows. | (1) A light pass in `computed.spec.ts`: seed `appearance.theme` through `openOperator` the way `ui.locale` is seeded, cover EN 100% over the main states, and assert `core.callsTo('plugin:window|set_theme')`. Add one `colorScheme: 'light'` case with nothing stored, for System → light and `null`. (2) `followTheme` unit tests with a fake `matchMedia`. (3) A picker test: `aria-pressed`, storage, `data-theme`. (4) An overlay test asserting no `data-theme` with `light` stored. (5) `theme.ts` in the coverage floors. (6) A `console.debug` in `titleBar`'s catch in dev. | S–M |
| L8 | Low | **Under a contrast theme, System always resolves to Light** (verified; tao's `should_use_dark_mode` is false in high contrast). Harmless: forced colours repaint, and the forced-colors rules use only system keywords. | Note it in the manual pass in `docs/accessibility.md`. | S |
| L9 | Low | **Copy.** FR: the hint opens on a bare "Système suit…"; write « Système » suit le mode d'application clair ou sombre de Windows. DE: the catalog's policy is to follow the Windows shell, which says "Modus", so consider "Farbmodus" over "Farbschema". EN is fine. The picker's markup matches the language cards exactly. | Two catalog edits. | S |
| L10 | Low | **Docs to update.** `docs/accessibility.md:31` (add the theme cards to the `aria-pressed` list; add L1 and L8 to the manual pass); `docs/architecture.md` (the light block and `theme.ts` beside `textScale`); release notes and the three What's new; `store-listing-next.md`; a README bullet; `OperatorToolbar.svelte:108` ("the app icon in miniature" until L5 lands); stale comments at `CaptionAppearance.svelte:234` and `SettingsDialog.svelte:123`. | With the release commit. | S |

L1 in detail. The edges, measured on white:

| Edge | Light | Dark |
| --- | --- | --- |
| Selected (`--accent-border`) | 1.60:1 | 2.54:1 |
| Unselected (`--line-strong`) | 1.52:1 | 1.30:1 |
| Hovered (`--line-hover`) | **2.58:1** | 1.86:1 |

- The selected wash is 1.09:1 on white, so the fill is effectively white.
- Where it shows:
  - The mode cards keep their checkmark.
  - The source tiles fall back on hue alone: `--text-muted` and `--accent-soft` have almost the same luminance, a WCAG 1.4.1 risk.
  - Pressed `.ui-tool` buttons (caption presets, the selected history session) look emptier than the grey unpressed ones.
- `docs/accessibility.md` promises that the selected mode, source and engine "stay distinguishable".

L6 in detail:

- The live dot's glow, `0 0 8px var(--accent)` (`OperatorToolbar.svelte:174`), turns into a dark-green smudge.
- The level meter's hot tip, `--warn` #995200 (`LevelMeter.svelte:91`), reads brown rather than amber.
- The bright caption preview is white on a white dialog with a 1.52:1 edge (`CaptionPreview.svelte:73`). `--line-hover` would read better.
- Text-field borders are 1.52:1. This predates the theme, but it shows more on white.

**Well done:** no component needed a colour change. The capability is scoped to the operator.
`setTheme(null)` hands the title bar back to Windows. Unknown and blocked stored values fall back
to System. The recomputed light ratios match the claimed floors (`--text-muted` 5.48–6.55, `--accent`
4.78–5.71, `--faint` 3.20–3.82, white on `--accent` 5.71). The overlay never reads the setting.

## 2. Defects

### Cloud sessions and capture (Rust)

| ID | Priority | Finding | Suggested change | Effort |
| --- | --- | --- | --- | --- |
| B1 | Medium | **A transient failure during setup ends the source for the event** (code verified; Gemini's close codes plausible). A setup timeout, an EOF, or *any* Close frame during setup is `Fatal` (`realtime/socket.rs:237-282`), and so is every Gemini `error` JSON (`gemini/mod.rs:45-47`). Gemini Transcribe goes through setup about every 10 minutes, so a 1011/1013 ("try again later") on one handover leaves that source in Error until the operator restarts. Mid-stream, the same Close reconnects. | A `RunEnd::SetupFailed { reason, close_code }`. 1007/1008 and auth, quota or billing wording → `Fatal`; 1001/1011/1012/1013, timeout and EOF → retry with backoff. Or: fatal only on a fresh start, capped at about 5 consecutive failures. Add a loopback test that closes with 1011 during setup. | S–M |
| B2 | Medium | **A `goAway` handover drops the old socket without draining it** (code verified; the size of the loss is plausible). `MessageControl::Handover` returns at once (`realtime/socket.rs:385-394`), with no `graceful_close` and no Close frame. The last 1–3 s of audio already sent (more on Live Translate) never come back as text, and the open turn finalizes mid-sentence. That is one fragment lost per handover, several times an hour. `catch_up` covers only the reconnect gap. | Drain within `min(timeLeft, ~4 s)`: send `audioStreamEnd`, read to `drain_complete`, queue incoming chunks into the backlog (raise `HANDOVER_BACKLOG` to about 60–80), then connect and replay. Make-before-break later. | M |
| B3 | Medium | **Closing the overlay destroys it for the rest of the run** (close path verified and rechecked; Alt+F4 delivery very likely). `lib.rs:158-163` returns early for any label but `operator`, so the overlay's `CloseRequested` goes through. In move mode the overlay takes focus, and tao keeps `WS_SYSMENU`, so Alt+F4 (or Task View's X) destroys it. After that, Show overlay does nothing and `set_overlay_placement` fails with "Overlay unavailable"; only a restart, which ends the session, brings it back. | `prevent_close()` + `hide()` for the overlay label. Optionally `maximizable: false`. | S |
| B5 | Medium | **System loopback can overflow under load** (D14, still open; code verified, loss plausible). `audio/loopback.rs:146-148` takes the *minimum* device period as the buffer duration (wasapi returns `(default, min)`). Each wake reads one packet (`:99-107`, `:266-278`), so any delay longer than about one period builds a backlog that never clears. SILENT and DATA_DISCONTINUITY flags are ignored. Whisper's busy-waiting workers create exactly that starvation when system audio and local Whisper run together. | `buffer_duration_hns` of about 2 000 000 (200 ms); loop while `get_next_packet_size() > 0`; zero-fill SILENT packets; log discontinuities (rate-limited). Hardware check. | S |
| B6 | Low–Med | **A malformed key loops "Reconnecting" forever** (verified). `secrets.rs:82-91` saves anything non-empty after `trim()`, which keeps U+200B and inner spaces. `connect_request()?` (`realtime/socket.rs:173`) turns the header or URI build error into `Failed`, retried every 16 s, though it can never succeed. | Reject non-printable-ASCII characters and whitespace in `set_api_key`; map request-build and setup-JSON errors to `Fatal`. | S |
| B7 | Low–Med | **Error details keep only the outermost context** (verified). `AppError::with(id, anyhow_err)` uses `to_string()` (`errors.rs:30-35`). At the call sites below the cause never reaches the operator, so "model download behind a proxy" (an open manual check) can't be diagnosed. | An `AppError::chain` formatting with `{:#}`, used at these sites. | S |
| B8 | Low | **A capture-thread panic in a cloud session shows as a quiet Idle** (path verified; trigger plausible). A panic in `capture.run` (`session/builder.rs:533-538`), e.g. wasapi's `ReleaseBuffer(..).unwrap()` on invalidation, skips `finish_producer`. The client sees the channel close, `Stopped`, Idle mid-session. | `catch_unwind(AssertUnwindSafe(..))` mapped to an error, then always `finish_producer`. | S |
| B9 | Low | **Mistral's end-of-turn grace can be cut short** (verified; contradicts `realtime/mod.rs:150-153` "no sooner, and no later"). `EndOfTurn` moves the finalize deadline only `if end < finalize.deadline()` (`socket.rs:375-381`). An earlier idle deadline fires first, and with no hold the turn finalizes before the grace. Trailing words or the final "." then become a lone line. | Reset to `end` unconditionally once `turn_end` is set; add a test with the idle deadline earlier than the grace. Then a quick Mistral rehearsal. | S |
| B10 | Low | **Restoring a placement clamps it to one monitor** (verified). `placement.rs:306-339` fits the rectangle to the monitor under its top-left corner, whereas `usable()`/`rescue()` deliberately allow a strip across two displays. A strip stretched across two projectors shrinks to one on the next launch. | If `usable()`, apply as is; else `rescue()`. This also removes the duplicated monitor selection. Mixed-DPI check. | S |
| B11 | Low | **Model downloads do blocking I/O on a tokio worker** (verified). `File::create`, `write_all` per chunk and a 190 MB `sync_all` inside `async fn download_from` (`whisper/models.rs:163, 258-259, 297`). This can stall a worker for seconds on a slow disk or under antivirus, during a cloud session. | `spawn_blocking`, or a writer thread fed by a channel. | S |

B7's call sites:

- `realtime/mod.rs:317`: a reconnect shows "WebSocket connect failed" and hides whether it was a 429, DNS, TLS or proxy failure.
- `commands.rs:26-66`: keychain errors lose the Windows error.
- `commands.rs:131-173`: model downloads lose reqwest's source chain.

### Operator and overlay (frontend)

| ID | Priority | Finding | Suggested change | Effort |
| --- | --- | --- | --- | --- |
| U1 | Medium | **Rehearse after a self-ended session runs with the wrong rail** (verified; rechecked). When every source ends in error, the core keeps a dead `ActiveSession`. The next Start or Rehearse calls `stop_active`, which emits a whole-session Idle (`session/mod.rs:208-216`) *after* `beginSession`. That Idle sets `sessionStartedAt` to null, and `+page.svelte:176-178` resets `rehearsing` on null. The rehearsal then shows the saved source, its meter, its activity row and its cost (D17 again, by another path), and the old Whisper failure returns to the status line. | Reset `rehearsing` on the `$isRunning` true→false edge, not on the clock. Ignore the reset and message of a whole-session Idle while a start is in flight (expose `starting` from `sessionController`). Page test: a source fails, then Rehearse, with an Idle during start. | S |
| U2 | Medium | **Two live regions announce on a timer** (code verified; rechecked; Narrator behaviour plausible). `WhisperActivity.svelte:51-57` is `role="status"` around "Audio waiting to be processed · N s". Progress arrives about once a second, and `pending_ms` includes the window being filled, so it ticks through every Whisper session. `WhisperModels.svelte:97` announces "Downloading N%" every 500 ms. Both contradict `docs/accessibility.md:30` ("nothing that changes on a timer"). | Drop the role from the ticking text and announce transitions only. Show the strip only when the backlog exceeds one window (about 12 s) or while finalizing. Let `<progress>` carry the percentage. | S |
| U3 | Medium | **When background saving fails, the operator is barely told** (verified). The first failed recovery write sets `stopped = true` for good (`recovery.ts:72-87`), and its message lives only in `statusMessage`, which the next status overwrites. History write failures show only in Settings → History. The idle page keeps saying the text is "spooled". | Keep retrying the spool with backoff, reporting once per failure streak; add a persistent "Recovery copy failing" / "History not saving" note beside the transcript's save badge. | S |
| U4 | Medium | **Keyboard focus is lost on Start and Stop** (code verified; Narrator plausible). The pressed button is disabled at once (`busy`), then removed by the `{#if $isRunning}` swap (`SessionControls.svelte:40-127`), so focus falls to `<body>`. `GlossaryList` already handles this case well. | After `tick()`, focus the counterpart; use `aria-disabled` while busy so focus isn't dropped first. | S |
| U5 | Med–Low | **The overlay has no ready handshake** (plausible). It only listens (`overlay/+page.svelte:64-66`). The operator pushes the caption languages once, then only on change (`nativeSync.svelte.ts:87-95`). If the overlay registers late (cold start, WebView2 reload), two-language row labels stay empty and the captions carry `lang=""`. Also, a stale `interactive: true` push can land just after the overlay's own Lock. | The overlay emits `overlay-state { ready: true }` once its listeners resolve; the operator answers with a full push. | S |
| U6 | Med–Low | **False "no captions for 15 seconds" alert after Resume or a slow reconnect** (verified). Staleness is measured from `max(sessionStartedAt, lastCaption)` (`liveActivity.ts:21-22`), and `lastAudio` keeps moving while paused. A coffee break longer than 15 s raises the alert, and Narrator speaks it, the moment the source resumes. | Record per origin when it last became `running` and measure from that. | S |
| U7 | Low | **Start re-enables between `start_session` resolving and the first status** (plausible). A fast second click or the shortcut restarts the session just begun. | Add `$sessionStartedAt !== null` to `startBlockers.busy`. | S |
| U9 | Low | **The unsaved-at-quit prompt accepts Escape during Save, and every button during Discard** (verified; narrow timing). | Keep the prompt busy for either answer; pass `onDismiss` only when idle. | S |
| U10 | Low | **The key panel can credit the wrong engine** (plausible). `ApiKeyPanel.svelte:70-96` reads `provider` after `await setApiKey`, and the engine cards stay enabled while it saves. | Capture `provider` before the await. | S |
| U11 | Low | **Small i18n slips** (verified). See the list below the table. | Count-aware strings, `Intl.NumberFormat` for currency, `formatElapsed`, store keys rather than finished strings. | S |
| U12 | Low | **Captions in the operator window have no `lang`/`dir`** (verified). In `LiveTurns` and `TranscriptMonitor`, Arabic, Hebrew, Persian and Urdu lay out left-to-right, and Narrator reads with the interface voice. The overlay already does this right (`OverlayCaptionLine.svelte:109-112`). | `lang` and `dir={captionDirection(...)}` per turn and paragraph. | S |
| U13 | Low | **History action errors carry the wrong headline** (open since 6 October; verified). `TranscriptHistory.svelte:324` prefixes copy, export, rename and delete failures with "Transcript history could not be updated." | A separate catalog line. | S |
| U14 | Low | **The crash-recovery opt-in can't be reached before a first session** (verified). It lives only in `TranscriptMonitor`, which mounts while idle only when the transcript is non-empty. | Mirror it in Settings next to History. | S |
| N1 | Nit | **The Mistral engine card's model id breaks off a lone "0"** (seen in the preview, French, 1200 px). `voxtral-mini-transcribe-realtime-3-0` wraps with `overflow-wrap: anywhere`. | Break at hyphens (`<wbr>` after each `-`), or ellipsis with a `title`. | S |

U11 in detail:

- The recovery prompt uses a plural pronoun for one line in all three catalogs, and French prints "0 lignes".
- `formatUsd` prints `$0.32` in French and German, beside a localized contrast figure.
- History durations print "125:00" for 2 h.
- `captionLanguageError` and `closeError` are stored as finished strings, so they don't re-word on a language switch.

## 3. Security

| ID | Priority | Finding | Suggested change | Effort |
| --- | --- | --- | --- | --- |
| B4 | Medium | **The overlay can do more than `architecture.md` says** (mechanism verified; capability file rechecked). `capabilities/overlay.json` grants `core:event:default`, which includes `allow-emit`/`allow-emit-to` with unscoped names. It also grants the core window setters, whose commands accept any `label`. See the note below the table. Defence in depth only: the CSP is strict and caption text is escaped. `command_names.rs` checks only app commands, so drift in the core permissions goes unguarded. | Send `overlay-state` through a new app command granted to the overlay; give the overlay only `core:event:allow-listen`/`allow-unlisten`; replace the core window setters with overlay-only commands acting on `OVERLAY_LABEL`; pin the overlay's full permission list in the capability test. | S–M |
| B12 | Low | **Keys can reach logs and proxies** (trace leak verified; header auth plausible). With `RUST_LOG=trace`, tungstenite logs the full handshake, including OpenAI's and Mistral's `Authorization: Bearer`. Gemini's key travels in the URL query, unencoded (`gemini/mod.rs:21-27`), where TLS-inspecting proxies log it. | Always cap `tungstenite`/`tokio_tungstenite` at `info` in the env filter (now). Probe whether Live accepts `x-goog-api-key` as a header (later). | S |
| B13 | Low | **Cloud WebSockets ignore the Windows proxy** (plausible). `connect_async_with_config` dials directly; only reqwest has `system-proxy`. On a proxied network, every provider loops "Reconnecting", with the cause hidden (B7). | Document it now; an HTTP CONNECT tunnel later. | S / M |
| B14 | Low | **The CSP lacks `object-src`, `base-uri`, `form-action`, `frame-src`** (verified). `default-src` covers none of the last three. | Append `; object-src 'none'; base-uri 'none'; form-action 'none'; frame-src 'none'`, then run `test:e2e` and `test:style`. | S |
| C6 | Medium | **GitHub security features are off** (rechecked through the API). Dependabot security updates, secret scanning and push protection are all disabled, and `audit.yml` has no Rust job. | The maintainer switches them on in Settings → Code security (free for a public repo). Add a scheduled `cargo audit` to `audit.yml`. | S |
| C5 | Medium | **Release and CI tokens are wider than needed** (config verified). `actions/checkout` leaves the token in `.git/config` before `npm ci` runs 334 packages' install scripts. That token is write-scoped in `release.yml`. Third-party actions use mutable refs: `dtolnay/rust-toolchain@stable`, `Swatinem/rust-cache@v2`, `tauri-apps/tauri-action@action-v1.0.0`, `rustsec/audit-check@v2.0.0`. Two others are already SHA-pinned. | `persist-credentials: false` on every checkout; pin third-party actions by SHA with a version comment, which Dependabot keeps current. | S–M |

What the overlay can do through B4:

- emit `tray-command` `stop-session`, which stops a billable session without a prompt;
- emit `quit` or `close-requested`;
- emit fake `caption` events, which flow into the transcript, history, the recovery spool and exports;
- emit fake `status` events;
- move or resize the operator window.

## 4. Efficiency

None of these needs action for 1.9.0. The overlay's per-caption cost is already bounded.

| ID | Finding | Suggested change | Effort |
| --- | --- | --- | --- |
| E1 | The recovery spool rewrites and fsyncs the whole transcript every 8 s while anything is unsaved (`recovery.ts`, `recovery.rs:249-263`). That is O(n) JSON, IPC and an atomic write per tick on an all-day event, and `nativeSync.svelte.ts:72` claims "only the unsaved part". | Reuse history's append-log format. | M |
| E2 | Every operator push makes the overlay call `setHold` (restarting every expiry timer) and `setPace` (flushing paced text). A colour or opacity drag pushes on every input event. Mid-session, a drag keeps finished captions up and releases steady-paced text early. | Call them only when the value changed; throttle pushes to one per animation frame. | S |
| E3 | The overlay ships all three catalogs, a 111 KB shared chunk. `languageRows` builds a new `Intl.DisplayNames` per row on every keystroke. | Lazy-load inactive catalogs; cache one `DisplayNames` per locale. | S |
| E4 | Whisper spool: three `write` syscalls per chunk and a `set_len(0)` on almost every read. The segmenter allocates a `Vec` per chunk, and each Start hashes the whole model, peaking at about 2× its size. | Truncate past a threshold; the rest is fine as is. | S |
| E5 | `TranscriptMonitor` regroups the whole log on every commit and renders every paragraph. | Window it if all-day sessions grow. | M |

## 5. Refactoring

- **S1 — one provider error vocabulary.** OpenAI `error_control`, Mistral `recoverable`,
  Gemini's always-fatal `control()` and the setup-phase close handling (B1) all classify errors
  independently. A shared `realtime::wire::classify(code, kind, message)` fixes B1 cleanly and
  makes the next provider cheaper. M.
- **S2 — an `Events` seam for `whisper::run` and `SessionManager`.** Both are tied to a concrete
  `AppHandle`, which is why D7, `finish_producer`, B8 and local-session orchestration have no
  tests. Make them generic over `R: Runtime` (`tauri::test::mock_app`), or inject `Events`. M.
- **S3 — small Rust tidy-ups.**
  - `.lock().unwrap_or_else(|e| e.into_inner())` appears 29 times; make it one crate-wide helper.
  - `start_audio_test(mic_device_name)` carries an id or a name; rename it to `mic_device`.
  - `ProviderSettings::Whisper` reaches `spawn_client` only to bail; encode local versus realtime in the type.
  - After B7, `AppError::chain` becomes the default.
  
  S.
- **S4 — split `+page.svelte`.** It has about 280 script lines. Extract a `launchController.svelte.ts`
  for `rehearsing`, `startBlockers` and `launch`, which also makes U1 testable, and
  `IdleStage`/`RunningStage` components. M.
- **S5 — split `TranscriptMonitor` (581 lines).** It is still mounted twice, so the copy window,
  the follow state and an in-flight save reset on every Start and Stop. Split it into
  `TranscriptActions` and `TranscriptLog`, and move recovery to Settings (U14). M.
- **S6 — de-duplicate.**
  - The move/place-overlay button is written three times (`LiveRail`, `SettingsDialog`, `PreflightChecklist`).
  - The export-format `<select>` is written twice.
  - `overlay` and `preflight` are prop-drilled through four or five components; use a context.
  - `loadLanguageFavourites` bypasses `persisted.ts`.
  - Split `SetupSheet` into its four steps.
  
  S–M.
- **Build size:** `sha2 = "0.11"` (`Cargo.toml:55`) next to wry's and secret-service's 0.10.9
  ships two copies (rechecked in `Cargo.lock`). The 6 October review warned against this bump;
  go back to `"0.10"` (B15, S).

## 6. Testing

| ID | Priority | Gap | Suggested change | Effort |
| --- | --- | --- | --- | --- |
| C2 | Medium | **The style probe can't see grid placement** (verified). `e2e/style/probe.ts:235-262` doesn't record `grid-template-rows`, `align-self`, `justify-self`, `grid-row`, `grid-column`, `align-content`, `z-index` or `transform`. #158's fix changed no snapshot, so it can regress unseen. | Add them to `OWN`, regenerate, read the diff. | S |
| C12 | Low | **The contract test checks names, not shapes** (verified; rechecked). Payload fields, command argument names and `WHISPER_MODELS` against `ModelId` are unchecked. One mismatch is live: TS types `AudioTestUpdate.message` as `string \| null` (`types.ts:292-295`), while Rust sends `Option<AppError>`. It works only because `statusMessage` accepts both. `OnDeviceReadiness.engine`/`.canPrepare` are never read. | Fix the type now and add `ModelId` to the enum list; compare `#[serde(rename_all)]` struct fields and `#[tauri::command]` parameters with the TS side later. | S / M |
| C13 | Low | **Untested Rust with real consequences** (verified). `secrets.rs` has 0 tests: a regression in its legacy-keychain migration loses users' keys on update. Also untested: `decode_interleaved` (i16, i32, f32 and unknown formats), `tray.rs` caching, and the runner cases for B1, B2 and B9 (the loopback harness already supports all three). | keyring's mock credential builder for `secrets.rs`; table tests for the decoder; three runner tests. | M |
| C21 | Low | **The stall test uses real time** (plausible). `whisper/models.rs:534`: 150 ms gaps against a 500 ms stall timeout. It can flake on a busy machine. | Raise the test's timeout to about 2 s. | S |

Page-level regressions to add with their fixes: U1 (Rehearse after a self-ended run), U4 (focus
after Start/Stop), U5 (overlay mounting after the first push), U6 (Resume after more than 15 s),
U3 (spool after a failed write). There are no test files for `SettingsDialog`,
`PreflightChecklist`, `OperatorToolbar` or `OverlayMoveChrome`. No brittle tests of note: queries
go by role and name, and timers are mocked.

## 7. CI, tooling and dependencies

| ID | Priority | Finding | Suggested change | Effort |
| --- | --- | --- | --- | --- |
| C1 | **High** | **The tag push is the first time the Store packages are built, and it publishes before they exist** (verified; rechecked in `release.yml:86-97`). See the note below the table. | (a) Always create a draft; publish from the `bundle` job with `gh release edit $tag --draft=false --notes-file docs/release-$v.md`. (b) A dry-run path: `workflow_dispatch` with a `ref` and no upload, or PRs touching `src-tauri/gen/windows/**`, `tauri.conf.json`, `Cargo.*` or `release.yml`, building msix and bundle as artifacts. That gives a candidate's assurance with no rc tag. (c) A first step that fails when the tag isn't `v` + the `package.json` version, or the manifests disagree. | M |
| C3 | Medium | **The Actions cache is over 10 GB and evicting** (rechecked: 38 entries, 10.39 GB). 5.6 GB of it sits under `refs/tags/*` keys, which no other run can restore, so every release build is cold (about 21 minutes). PR runs also save about 660 MB ARM caches each. | `save-if: false` on rust-cache in `release.yml`; `save-if: ${{ github.ref == 'refs/heads/main' }}` in `ci.yml`; delete the tag caches. | S |
| C4 | Medium | **The ARM Rust lane spends 2–2.5 min on `npm ci`** (timing verified). `rust (windows-11-arm)` is the critical path (median 6.0 min). It installs npm only to build `build/` for `generate_context!`. | Use a stub `build/index.html` for clippy and tests, or build once on Ubuntu and download it. Expected wall clock: about 6.6 → 4.5 min. | S |
| C7 | Medium | **Warnings can't fail CI** (verified). `svelte-check` runs without `--fail-on-warnings`; it has 0 warnings today, so locking that in is free. | Add the flag; add ESLint `@typescript-eslint/switch-exhaustiveness-check` (Provider, SessionState and ThemeChoice mirror Rust enums), `svelte/no-unused-svelte-ignore`, `svelte/no-at-html-tags`. | S |
| C8 | Med–Low | **Nothing checks the version files agree** (verified). `package.json`, `package-lock.json` (twice), `Cargo.toml`, `Cargo.lock`, `tauri.conf.json` and `CITATION.cff` are unchecked, and the runbook's step 1 omits `CITATION.cff`, which 1.8.0 changed. | A `version.test.ts`; add `CITATION.cff` to the runbook. | S |
| C9 | Med–Low | **`cancel-in-progress` also cancels `main` runs** (plausible). A second merge within about 7 minutes cancels the release merge's run, leaving the commit to tag without a green CI. | `cancel-in-progress: ${{ github.event_name == 'pull_request' }}`. | S |
| C10 | Low | **CI floats on stable Rust** (config verified). `rust-version = "1.90"` is never tested, and a new stable's Clippy lints can turn CI red under `-D warnings` on release day. | `rust-toolchain.toml`, bumped on purpose. | S |
| C11 | Low | **A yanked crate ships in the exe** (verified). `chacha20 0.10.1` comes through `rand` ← `tungstenite`. Five other RustSec warnings are Linux-only. The audit action runs on Node 20 and compiles cargo-audit every run (2–3 min). | `cargo update -p chacha20` (lockfile only); `audit.toml` ignores with reasons; `taiki-e/install-action` + `cargo audit`. | S |
| C14 | Low | **Coverage floors.** `theme.ts` isn't among the per-file floors, and there is no global floor. | Add it after L7; a global ratchet just under today's numbers. | S |
| C15 | Low | **TypeScript strictness.** `noUncheckedIndexedAccess`, `noFallthroughCasesInSwitch`, `noImplicitReturns` and `noImplicitOverride` are off. The last three are cheap. | Enable them. | S–M |
| C16 | Low | **Status docs are stale on `main`** (rechecked). `README.md:17`, `ROADMAP.md:28` and `docs/README.md:4,28` say 1.8.0 "awaits Store submission", which `store-updates.md` contradicts. `store-listing-next.md` still mentions release-candidate checks. | Fix in the release commit. | S |
| C17 | Low | **Leftovers on GitHub.** Six draft releases: three rc drafts with assets and three empty ones (v1.6.0, v1.2.4, v1.2.3). rc tags v1.0.0-rc.1…4, v1.1.0-rc.1, v1.7.0-rc.1 and v1.7.1-rc.1 remain. | The maintainer deletes them, if wanted (permanent). | S |
| C18 | Low | **The manual-check backlog keeps growing.** About 12 checks have carried over since 1.7.0, and 1.8.0 shipped without its sideload test, WACK, screenshot retake or Local Test report ticked. With no candidates, the Local Test build of the release merge is the only gate. | Triage each: do it, automate it, or drop it explicitly. Make the Local Test pass a precondition for tagging. | S |
| C19 | Low | **Dependabot has no `groups`.** About 10 PRs a week each run the full CI, and Tauri's npm and Rust halves arrive separately. | Groups for `@tauri-apps/*` + `tauri*`, npm dev minor+patch, Cargo patch. | S |
| C20 | Low | **`ubuntu-latest` moved to 26.04 on 19 October** (plausible). The rust-ubuntu lane apt-installs WebKitGTK 4.1. | Pin `ubuntu-24.04`, or verify 26.04. | S |

C1 in detail:

- `releaseDraft` is true only for hyphenated tags. A plain `vX.Y.Z` therefore publishes the GitHub release at once, with an empty body (no `releaseBody`), before the msix and bundle jobs run.
- If one of them then fails, a public release exists without its Store bundle, and the version is spent.
- Nothing checks that the tag matches `package.json`, `tauri.conf.json` and `Cargo.toml`. The bundle takes its name and `/bv` from `package.json`, while the MSIX version comes from `tauri.conf.json`.

**CI timing** (ten most recent runs): wall clock about 6.6 min. On the critical path,
`rust (windows-11-arm)` takes 6.0 min: `npm ci` 124–158 s, cache restore about 63 s, clippy 29 s,
tests 33 s, smoke test 41 s. Then `rust (windows-latest)` 4.3, `e2e` 3.0, `rust-security` 2.9,
`rust (ubuntu)` 2.6, `styles` 2.3, frontend 1.1–1.2. A release takes about 21 min, always cold
(C3).

**Dependencies** (22 October). Nothing is deprecated and no Dependabot PRs are open. Behind the
registry:

- npm patches: `@playwright/test` 1.64.1, `@tauri-apps/api`/`cli` 2.12.4, `vite` 8.3.5, `vitest`/`@vitest/coverage-v8` 5.0.5, `jsdom` 30.1.3, `svelte-check` 4.7.7.
- npm minors: `@sveltejs/kit` 3.1.0, `eslint` 10.13.0, `typescript-eslint` 8.72.0, `knip` 6.42.1.
- Rust: `tokio` 1.54.1; `tauri` 2.12.5, `tauri-build` 2.7.4, `tauri-plugin-single-instance` 2.5.4.
- Held on purpose: `typescript` 7 (peers cap at 6) and `@types/node` 26 (the floor is Node 22).

Recommendation: keep 1.9.0 free of dependency changes except the `chacha20` lockfile bump, and
merge the next Dependabot batch after the tag.

## 8. Re-assessment of the 6 October open items

- **D14:** reported again as B5, at higher priority.
- **E3 (VAD):** still the biggest lever for Whisper quality and CPU. With the −80 dBFS gate, a real
  microphone always produces 10 s "speech" windows, which adds CPU pressure that makes B5 worse.
- **F3–F8:** unchanged, still optional. F6's per-source noise reduction is still open (OpenAI sends
  `near_field` for every source).
- **"Still needs Windows hardware or live providers":** all of it is still open. Two items carry
  more weight now:
  - the live Gemini handover, which would expose B1 and B2;
  - pong replies: a provider that doesn't answer pings makes a quiet room reconnect about every 31 s.
- **Order:** land B7 before the proxy download check, or that check can't be diagnosed.

## 9. Suggested batches

1. **Before tagging 1.9.0 (needed):**
   - L1, with its palette test.
   - L2, the inline script.
   - L3, once the maintainer decides between keeping System and migrating to Dark.
   - L5, L6 and L9.
   - L7: the `followTheme` and picker tests, and a light style pass.
   - C1: draft-first publishing and the version guard.
   - C16 and L10, with the release commit.
2. **Small, safe fixes (recommended for 1.9.0; otherwise 1.9.1):**
   - Rust: B3, B6, B7, B8, B9, B11, B12 (log cap), B14 and B15.
   - Frontend: U1, U2, U6, U9, U11, U13 and the C12 type fix.
   - Lockfile: the C11 `chacha20` bump.
   - Each fix is S and lands with a test. B9 needs a quick Mistral rehearsal; U4 needs a Narrator pass.
3. **CI hygiene (no app change; any time):**
   - Workflow and tooling changes: C2–C5, C7–C10, C14, C15, C19 and C20.
   - For the maintainer: C6 and C17.
4. **Robustness that needs providers or hardware:** B1 (with S1), B2, B5, B10, B13, U3 and U5.
5. **Security hardening:** B4, and B12's header authentication after a live probe.
6. **Structure and tests:** S2–S6, C13, U7, U10, U12, U14, and E1–E5 as needed.

## 10. Release checklist for 1.9.0

No release candidate. Tag `vX.Y.Z` straight from the release merge.

1. Land batch 1 (and whatever of batch 2 is chosen) through PRs against `main`. Each needs all
   nine jobs green, then a merge by hand.
2. Branch `chore/release-1.9.0` from a fresh `origin/main`, not from `feat/light-theme` (merged
   and deleted remotely) and not from the stale local `main`.
3. Bump the version: `package.json`, `package-lock.json` (both places), `src-tauri/Cargo.toml`,
   `src-tauri/Cargo.lock`, `src-tauri/tauri.conf.json` and `CITATION.cff`. Then run
   `cargo check --locked`.
4. Write the docs in the same commit: `docs/release-1.9.0.md` and a row in `release-notes.md`;
   What's new in EN/FR/DE in `store-listing.md`, saying whether System is the default (L3); a
   features line; the 1.9.0 handoff in `store-updates.md`; C16's status lines.
5. Run every local gate, including `test:style` after `build`, and `test:e2e` with the cargo
   environment.
6. Merge, and confirm `main`'s run on the merge commit finishes green and isn't cancelled (C9).
7. Test the Local Test build of the merge commit. Check:
   - Light, Dark, and System while switching Windows' app mode;
   - that the title bar follows;
   - that the overlay's move toolbar stays dark;
   - first launch on a light desktop (L2);
   - a contrast theme;
   - every Settings tab in Light;
   - the History filter row in French.
8. Tag annotated `v1.9.0` ("Live Translation & Subtitles v1.9.0") and push. Without C1, add the
   body afterwards with `gh release edit v1.9.0 --notes-file docs/release-1.9.0.md`.
9. Verify the bundle at 1.9.0.0 with the bundle checks from 1.8.0. Then sideload, WACK, and
   screenshots in EN/FR/DE with the theme pinned (L3), adding one shot of the theme setting.
   Submit through Partner Center with the single `.msixbundle`. Once live, confirm with
   `Get-AppxPackage`.
