# App review — 16 November 2026

Scope: a review of `main` at `442bc02` (1.14.0 prepared), followed by the work it led to, on
the branch `ccr-a1526fde-jevpxv`. Items the [10 November](app-review-2026-11-10.md),
[22 October](app-review-2026-10-22.md) and [6 October](app-review-2026-10-06.md) reviews fixed
or declined are not repeated. This file is also the tracker: each item carries its status.

**How it was done:** three reviewers read the Rust core, the interface (with a browser preview
at 1180 × 760 and 980 × 660, dark and light, in three languages and up to 225% text) and the
CI, tests and dependencies; the orchestrating pass rechecked the main findings in the source.
Eight implementation agents then worked on separate files, in two waves, and their work was
checked together.

**Checked here** (Linux): `cargo fmt`, `clippy -D warnings --all-features` and `cargo test`
(529 passed, 7 ignored); `npm run verify`'s steps (1,109 tests; coverage 95.4% of lines). The
style suite ran in Edge for Linux, where 28 computed snapshots differ from the Windows
baselines on an untouched tree too; no other test fails, and the same 28 fail after the work.

**Not checked:** nothing ran on Windows or on a Mac. `#[cfg(windows)]` and macOS code is not
compiled on Linux: CI's lanes do that. No provider was called.

**Status values:** *Done*, *Deferred* (with the reason), *Maintainer*.

## Summary

No high-severity defect. What mattered: four overlay rows showed four labels and no caption
(B1), DeepL could drop concluded words without a trace (B2, B3), a microphone line decoded
behind a Whisper backlog passed the echo filter (B4, found while writing its test), the
pre-flight understated a Both-sources bill (B5), and CI rebuilt the Windows Vulkan helper from
cold on every run (C1).

## 1. Defects

| ID | Finding | Severity | Where | Status |
| --- | --- | --- | --- | --- |
| B1 | Both sources × two caption languages in the default 1280 × 160 strip: the 20 px floor did not fit a row, so every row drew its label and no text. Seen in the preview. | Medium | `src/lib/captionLayout.ts`, `src/routes/overlay/+page.svelte` | Done |
| B2 | DeepL: a concluded segment ending no later than the last sentence-final was dropped, with no log line. Segment-time order is unverified. | Medium | `src-tauri/src/deepl/client.rs` | Done |
| B3 | DeepL: the tentative start of the next sentence was not shown until it changed, and a Pause or Stop skipped the drain while the provider still held it. | Low-medium | `deepl/client.rs`, `realtime/socket.rs` | Done |
| B4 | Echo filter: system turns were forgotten 20 s after they arrived, on the clock, so a microphone line local Whisper decoded later passed as new speech. | Medium | `src-tauri/src/echo.rs` | Done |
| B5 | The pre-flight cost row never showed "×2 sources", and its "×2 languages" tag overflowed the glyph column in German and at 225%. | Medium-low | `PreflightChecklist.svelte`, `ui/ChecklistRow.svelte` | Done |
| B6 | Returning to Live translation always selected Gemini. | Low | `src/lib/setupActions.ts` | Done |
| B7 | The language picker answered "No matching languages" for a language the engine lacks. | Low | `src/lib/languages.ts` | Done |
| B8 | Escape on the first "hide to tray" explanation quit the app. | Low | `TrayHidePrompt.svelte` | Done |
| B9 | A provider without a setup acknowledgement that closed each socket as it opened was retried for ever. | Low | `realtime/policy.rs`, `socket.rs` | Done |
| B10 | DeepL source text was not deduplicated as the translation is. | Low | `deepl/client.rs` | Done |
| B11 | An unsupported loopback sample format logged a warning per packet and captioned silence. Not compiled here. | Low | `audio/loopback.rs` | Done |
| B12 | The Vulkan helper's timeout did not cover the write of a request. | Low | `whisper/gpu.rs` | Done |

## 2. Interface

| ID | Finding | Status |
| --- | --- | --- |
| U1 | A new install says "Download a model before starting" with no action in the row. | Done: a Download button with progress, sharing the model step's state |
| U2 | The operator is not told when the overlay draws smaller than the chosen size. | Done: a notice under Caption size |
| U3 | Settings and the Zoom dialog open with focus on Close. | Done |
| U4 | Icons do not follow text scaling. | Done: sized in `rem` |
| U5 | The empty History tab leads with filters. | Done |

## 3. Efficiency

| ID | Finding | Status |
| --- | --- | --- |
| E1 | Stable reading laid out an unbounded live turn on every delta; every track was recomputed on any track's delta. | Done |
| E2 | `inFlightLines` rebuilt per delta for a value only Copy needs. | Done |
| E3 | DeepL: four allocations per update; each audio chunk allocated three times. | Done |
| E4 | Echo guard: the pair set rebuilt and four lists made per check. | Done in part: refilled in place, one list. An incremental structure was not clearly safe in that code |
| E5 | Capture threads poll their cancellation token every 100 ms. | Deferred: the waking half runs only on Windows and macOS, and the gain is ten wake-ups a second |
| E6 | Long transcripts. | No change needed: 4,000 lines render in one to two frames in the preview |

## 4. CI, tests and tooling

| ID | Finding | Status |
| --- | --- | --- |
| C1 | The Windows Vulkan helper builds outside the cached target folder: about 10 of CI's 14 minutes, and every release waits for it. | Done: its folder is cached, saved by main only |
| C2 | Three smoke steps pass if their test is renamed. | Done |
| C3 | A tag push starts a second macOS build. | Done |
| C4 | Docs-only changes run the whole matrix. | Done for pull requests. Main always runs everything: the release gate counts a run with skipped jobs as passed |
| C5 | The coverage ratchet was 3 to 5 points under the totals. | Done: 93/92/91/85 |
| C6 | Nothing pinned the CSP, the MSIX identity or a shipped version's stored settings. | Done: `security.test.ts`, `msixIdentity.test.ts`, `storedState.test.ts` |
| C7 | No single verify command; no pull-request template; Dependabot without a cooldown; two `run:` lines interpolating the tag. | Done |
| C8 | Untested: `OverlayMoveChrome`, `TranscriptLog`, `PreflightChecklist`, `recovery.rs`, `export.rs`, session shutdown past a wedged thread, the move from the graphics processor. | Done |
| C9 | No property or fuzz tests; Rust coverage not gated; no link or spell check. | Deferred: hardening without a defect behind it |

## 5. Features

| ID | Feature | Status |
| --- | --- | --- |
| F1 | A new-version notice for copies installed from GitHub ([#217](https://github.com/fmadore/Live-translation/issues/217)). | Done: off until turned on, asked once from the second launch |
| F2 | System-wide Pause and Show/Hide overlay keys. | Done: opt-in, held only while a session runs |
| F3 | A spending guard for cloud engines. | Done: all off by default |
| F4 | Markers in the transcript. | Done |
| F5 | Find in the live transcript. | Done |
| F6 | A notice on the overlay for breaks. | Done |
| F7 | A one-click fallback engine when a source ends on a key or quota error. | Deferred: it restarts a source inside a running session, which is session-core work of its own |
| F8 | Saving the transcript to a chosen folder at Stop. | Deferred: the Mac App Store sandbox needs a security-scoped bookmark for a folder kept across launches |

## 6. Notes from doing it

**Decisions taken, for the maintainer to confirm**

- **F1's default.** The issue left it open; this takes its middle way: nothing is requested
  until the user turns the check on, and eligible copies are asked once, from the second
  launch (`ASK_FROM_LAUNCH` and the `Unanswered` default in `src-tauri/src/update.rs`).
- **F2's keys:** Ctrl+Alt+Shift+P and Ctrl+Alt+Shift+O (⌃⌥⇧ on a Mac), fixed. On layouts where
  Ctrl+Alt is AltGr, those characters cannot be typed while a session holds the keys.
- **B9** applies to OpenAI and Mistral too, but only to a socket that ends within five seconds
  without sending anything. **B3**'s drain change means Mistral also drains while it says the
  speaker is mid-sentence.
- **B1** does not resize the overlay: rows close up and shrink, the original speech gives way
  first, and below 12 px the row is reported as having no room.
- **C4:** `prepare` in `release.yml` accepts a run whose jobs were skipped, so a docs-only
  commit on main still runs everything.

**Left**

- A DeepL tail that was never on screen is still lost when its socket drops: fixing it means
  knowing what a resumed session resends.
- DeepL source text can run ahead of its translation at a sentence end: source and target times
  are not known to share a timeline.
- The reduced-motion rule in `app.css` gave a measuring element its old font size;
  `OverlayCaptionLine` reads its line height the same way and may be affected.

**Still to check by hand**

- **CI on this branch:** the Windows and macOS lanes compile `update.rs`'s package-identity
  check, the loopback format check, `export.rs`'s helpers and the global-shortcut plugin; the
  style snapshots, merged by hand from Linux runs, pass in Edge on Windows; the Vulkan cache's
  first save and second-run restore.
- **Overlay:** four rows on a projector; the "drawn smaller" notice in the rail; the break
  notice with reduced motion on.
- **DeepL, with a paid key:** the debug log's dropped segments, which answer whether segment
  times increase; the tail after a sentence end; a refused socket ending the source after five.
- **Echo filter:** Both sources with local Whisper under a backlog.
- **Update notice** (#217's acceptance criteria): an installer copy and the disk image ask from
  the second launch and show the notice once turned on; the Store MSIX, a sideloaded MSIX and
  the Mac App Store build show nothing and send nothing, read with a proxy.
- **System-wide keys:** over full-screen PowerPoint and Keynote, from the MSIX and from the
  sandboxed Mac build; no permission prompt on a Mac; the refusal line when another app holds a
  combination.
- **Find:** Ctrl+F keeps WebView2's own find bar shut; highlight colours in the contrast themes.
- **Narrator and VoiceOver:** the guard notice, the find count, marker headings, the update
  line, each read once.
- **Native-speaker review** of the new French and German strings.
