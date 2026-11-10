# App review — 10 November 2026

Scope: a review of `main` at `ab38f81` (55 commits after v1.12.0, before 1.13.0 is tagged): the
Rust core, the interface, the visual design, CI, the release workflow, dependencies and the
documents. Items the [6 October](app-review-2026-10-06.md) and
[22 October](app-review-2026-10-22.md) reviews fixed or declined are not repeated. This file is
also the tracker for the work that followed: each item carries its status.

**How it was done:** four reviewers read the code (audio and Whisper; providers, session and
Zoom; the interface; tooling and documents), and the orchestrating pass rechecked B1 and B2 in
the source and walked the interface in a browser preview (dark, light, 980 × 660, Settings, the
overlay). No session was run: the preview has no Tauri runtime. CI on `main` was green.

**Status values:** *Open*, *Done* (on `fix/review-2026-11-10`), *Deferred* (with the reason),
*Maintainer* (a decision or an action outside the repository).

Labels: **verified** findings were traced in the code; **plausible** ones need a run, hardware
or a live provider to confirm.

## Summary

No high-severity defect. The findings that matter: a rehearsal's captions reach a live Zoom
share (B1), the Both-sources echo filter lets most of a Whisper window's echo through (B2, B5,
B6), a provider's mid-stream close is retried for ever without its reason (B3), and the release
workflow's Mac, Vulkan and package-check steps will run for the first time on the 1.13.0 tag
(C1).

## 1. Defects

| ID | Finding | Severity | Where | Status |
| --- | --- | --- | --- | --- |
| B1 | A rehearsal or the built-in demo posts its sample captions to a Zoom share left running: `begin_session` resets only the publisher, and a rehearsal runs as the system origin. Verified. | Medium | `src-tauri/src/zoom.rs` | Done |
| B2 | Echo filter: the reconciler gives each Whisper segment its own turn, and a held microphone line is committed as soon as the next turn arrives, so of a window's n segments only the last waits for its match. `HOLD_LOCAL` is also a fixed 10 s, shorter than a 28 s catch-up window. Verified. | Medium | `src-tauri/src/echo.rs` | Done |
| B3 | A Close frame in mid-stream is always a plain reconnect: its code and reason are dropped, so a 1007 or 1008 after setup loops with backoff for ever and the operator never sees why. Verified. | Medium | `src-tauri/src/realtime/socket.rs` | Done |
| B4 | The previous run's whole-session Idle event can arrive after the new run's `start_session` has answered; the page then clears the new run's state and flashes the setup screen. Plausible. | Medium | `src/lib/sessionStatus.ts`, `sessionController.ts` | Done |
| B5 | Echo filter: only the last 200 words of a system turn are kept, but the microphone caption is compared whole, so an echoed turn past about 333 words reappears. Verified in the logic. | Medium-low | `src-tauri/src/echo.rs` | Done |
| B6 | Echo filter: an interim hovering at the threshold alternates between a withdrawal and a pass, so the line blinks in the overlay and in Zoom. Verified. | Low | `src-tauri/src/echo.rs` | Done |
| B7 | One window failing on the graphics processor (`Response::Failed`, after which the helper can still take another) replaces the helper and moves the rest of the session to the processor. Verified. | Low | `src-tauri/src/whisper/mod.rs` | Done |
| B8 | The cloud consent prompt puts focus on Send; the other decision prompts focus the safe answer. Verified. | Low | `src/lib/ProviderConsentPrompt.svelte` | Done |
| B9 | SRT export escapes text as HTML, so "Q&A" is written `Q&amp;A`. SRT has no entities. Plausible (depends on the player). | Low | `src/lib/transcript.ts` | Done |
| B10 | Loading a model in the Vulkan helper waits up to 120 s with the lifecycle lock held and no way to stop. Verified. | Low | `src-tauri/src/whisper/gpu.rs` | Done |
| B11 | Stop or Pause during a handover drain is ignored for up to 4 s. Verified. | Low | `src-tauri/src/realtime/socket.rs` | Done |
| B12 | A Normal Closure (1000) during setup is classified fatal and ends the source. Plausible. | Low | `src-tauri/src/realtime/socket.rs`, `wire.rs` | Done |
| B13 | The `Authorization` header value is not marked sensitive, so a `Debug` of the request would print the key. Verified. | Low | `src-tauri/src/realtime/wire.rs` | Done |
| B14 | A cloud client cut off at the drain timeout loses its open turn's final. Plausible. | Low | `src-tauri/src/session/mod.rs` | Done |
| B15 | On macOS the tray keeps Windows' click conventions (menu on right click only). Not run on a Mac. | Low | `src-tauri/src/tray.rs` | Done |
| B16 | Windows' property-value notifications (Bluetooth endpoints emit them repeatedly) each count as a device change. Plausible. | Low | `src-tauri/src/audio/devices.rs` | Done |
| B17 | One rejected `listen` leaves every other listener subscribed at teardown, with an unhandled rejection. Verified. | Low | `src/routes/+page.svelte` | Done |
| B18 | Renaming a session deleted a moment before shows the English text `Session deleted` and marks history as not saving. Verified. | Low | `src/lib/history.ts` | Done |
| B19 | The History preview repeats the original speech under both languages' lines, without `lang` or `dir`. Verified. | Low | `src/lib/TranscriptHistory.svelte` | Done |

## 2. Accessibility and wording

| ID | Finding | Where | Status |
| --- | --- | --- | --- |
| A1 | Status regions that are mounted together with their text, which screen readers often miss (the page's own rule). | `MeetingProfiles`, `LanguagePicker`, `SystemCapturePicker`, `TranscriptActions` | Done |
| A2 | Notices kept as finished sentences stay in the old language after the interface language changes (the U11 class). | `MeetingProfiles`, `TranscriptHistory`, `ZoomCaptionsDialog` | Done |

## 3. Visual design

| ID | Finding | Status |
| --- | --- | --- |
| V1 | The pre-flight intro says "Four checks"; Live translation shows three rows. | Done |
| V2 | A disabled Start is the same green at half opacity and still reads as the button to press. | Done |
| V3 | The paragraph describing Rehearse sits unlabelled under the checklist, away from the Rehearse button. | Done |
| V4 | Settings › Captions: the two custom-colour buttons wrap to two lines in French. | Done |
| V5 | The setup rail needs scrolling to reach the engine and the model, even at 1180 × 760. | Deferred: a layout decision for the maintainer, not a defect. |

## 4. Efficiency

| ID | Finding | Where | Status |
| --- | --- | --- | --- |
| E1 | The echo check re-tokenises the microphone text and every nearby system turn on each delta, under the lock every caption waits on. | `src-tauri/src/echo.rs` | Done |
| E2 | The fallback to the processor reads and hashes the whole model again while holding the inference lock. | `src-tauri/src/whisper/mod.rs`, `models.rs` | Done |
| E3 | The tray state, labels included, crosses IPC every second while a session runs. | `src/lib/nativeSync.svelte.ts` | Done |
| E4 | Several passes over the whole transcript for each finalized line (`hasTranscriptTiming` twice, `hasOriginalSpeech` in two components). | `TranscriptActions`, `TranscriptMonitor` | Done |
| E5 | The display watcher makes three main-thread round trips every second. | `src-tauri/src/placement.rs` | Done |
| E6 | Interim captions carry the whole turn on every delta, to both windows, the echo guard and Zoom. | `src-tauri/src/realtime/mod.rs` | Deferred: changes the caption contract of both windows; measure a long turn first. |

## 5. Refactoring

| ID | Change | Size | Status |
| --- | --- | --- | --- |
| S1 | Echo guard: a hold queue per lane, cached tokens, the same tail on both sides (with B2, B5, B6, E1). | S–M | Done |
| S2 | One close-frame classification for setup and mid-stream (with B3, B12). | S | Done |
| S3 | A "live" flag on the session for anything that leaves the machine (with B1). | S | Done |
| S4 | Split `whisper/mod.rs` (1333 lines, half of them tests) into backend, pipeline and tests. | M | Done |
| S5 | Move the turn-finalize timer out of the socket pump into a unit-tested type. | M | Done |
| S6 | One provider error helper in place of the three copies. | S | Done |
| S7 | One "park while present" helper for the three capture loops. | S | Done |
| S8 | Tests for the Vulkan helper's failure paths (timeout, unreadable answer, helper dying). | M | Done |
| S9 | Interface: one helper for subscribing to core events; consent moved into the launch controller; a shared two-step confirm button. | S each | Done |
| S10 | A shared endpoint type for the provider configurations. | S–M | Deferred: no defect behind it; revisit with a fourth engine (as S3 of 22 October). |

## 6. CI, release and dependencies

| ID | Finding | Status |
| --- | --- | --- |
| C1 | The release's `bundle` job waits on `macos`, and the macOS, Vulkan-helper and package-check steps have never run in a release. Push a hyphenated validation tag (a draft) before `v1.13.0`. | Maintainer |
| C2 | Nothing checks that CI passed on the tagged commit. | Done |
| C3 | `msix` waits on `build` without using anything from it: about nine minutes per release. | Done |
| C4 | The Dependabot ignore on `sha2 >= 0.11` now causes the duplicate digest family it was meant to prevent. | Done |
| C5 | "Configure native Whisper compiler" is written four times across workflows. | Done |
| C6 | `cargo-audit` and `install-action` pins are behind; the `prepare` job does not check `whisper-vulkan/Cargo.toml`'s version; `audit.toml` comments speak of Windows only. | Done |
| C7 | Dependencies behind: wasapi 0.26 → 0.27, rfd 0.17 → 0.18, `@sveltejs/kit` 3.1.1 → 3.2.0, vite 8.3.5 → 8.3.7, Playwright 1.64 → 1.65. | Done |
| C8 | Dependabot alert GHSA-gcqx-w48m-w2h4 (`deepmerge`, low, development only, no fixed version): dismiss as tolerable or wait for upstream. | Maintainer |
| C9 | `macos.yml` builds in release profile on every push to every branch. | Maintainer: the test build on every push was a deliberate choice for the port. |

## 7. Documents

| ID | Finding | Status |
| --- | --- | --- |
| D1 | `ROADMAP.md`: the status heading does not say 1.13.0 is in development; "a notarised disk image in each release" (none yet); the App Store upload of 5 November is not recorded; the 1.0.5 entry still says macOS support was dropped. | Done |
| D2 | `README.md` says 1.12.0 is pending Store certification; `ROADMAP.md` and `docs/README.md` say it is not yet submitted. | Maintainer: which is true |
| D3 | `docs/store-updates.md`: the v1.12.0 tag and release steps are unticked although both happened. | Done |
| D4 | `docs/mac-app-store.md` lists two secrets as missing that are now set; `docs/release-1.13.0.md` says the App Store build has not been uploaded. | Done |
| D5 | `docs/app-review-2026-10-22.md`: C6 is now closed (security updates, secret scanning, the Rust audit job). | Done |

## 8. Issues

- [#195](https://github.com/fmadore/Live-translation/issues/195) (macOS): the code is in; the
  first run on a real Mac is the gate.
- [#203](https://github.com/fmadore/Live-translation/issues/203) (Mac tip jar): waits on the App
  Store Connect agreements.
- [#187](https://github.com/fmadore/Live-translation/issues/187) (caption sharing): every
  approved workstream has shipped; it can be closed, or cut down to the deferred OBS output.
- [#175](https://github.com/fmadore/Live-translation/issues/175) (Store experiment): waits on the
  traffic check in Partner Center.
- [#12](https://github.com/fmadore/Live-translation/issues/12) and
  [#32](https://github.com/fmadore/Live-translation/issues/32): research, unchanged.

## 9. Batches

- **A. Correctness:** B1–B19, A1, A2, with S1, S2 and S3, which fix them.
- **B. Release preparation:** C2–C7, D1, D3–D5.
- **C. Polish:** V1–V4, E1–E5.
- **D. Structure:** S4–S9.

## 10. Notes from doing it

All four batches are on `fix/review-2026-11-10` (10 November 2026), written by six
implementation agents on separate files and checked together: `cargo fmt`, `clippy -D warnings`
and `cargo test` on Windows ARM64, and the npm suite with the style snapshots. Not compiled
here: the macOS modules and the `whisper-vulkan` helper (no Vulkan SDK on the machine); CI's
lanes do both.

**Done differently from the finding**

- **B1** keeps the share's status as it was during a rehearsal or the demo: it reports Zoom's
  last answer and the count Zoom accepted, and both stay true. Nothing is sent, and a line is
  logged. A test caption from the dialog still goes out.
- **B2** grew past a queue. A line waits only when the system source was sounding within four
  seconds of it in session time (not the clock, which means nothing under a Whisper backlog);
  the microphone turn behind held lines is kept back, because the windows commit a pending turn
  when the next one begins; `HOLD_LOCAL` went from 10 s to 30 s, with an early release once a
  system caption has passed the line. A microphone line far from any system sound now passes at
  once. [Architecture](architecture.md#one-voice-captioned-twice) describes the rules.
- **B3/B12:** only 1007, 1008 and the fatal wording (key, quota, billing) end a source. Every
  other close is retried, so an undocumented code such as 4000 during setup, fatal at once
  before, is now retried within the five-in-a-row cap.
- **B4** is fixed in the interface, without a run id from the core: the page knows when the
  core may still hold a session it started, and ignores the one whole-session Idle that the next
  start will cause. It relies on the core sending a whole-session Idle only from `stop_active`
  (`session/mod.rs`); a second place that sends one must revisit `sessionStatus.ts`.
- **B7** retries a failed window once on the helper; an unreadable or out-of-step answer now
  ends the helper, as its stream can no longer be trusted.
- **B10** lets quitting and discarding end a model load that hangs. The Stop button still waits
  for the lifecycle lock; interrupting a load from Stop would mean routing it through discard.
- **B14** finalizes an aborted client's open turn from a drop guard, and shutdown waits for the
  aborted tasks so that last line arrives before Idle.
- **B16** keeps the property notification but only for the three name keys, so a renamed device
  still refreshes the lists.
- **B9:** SRT text is written as it is; only `-->` is changed (to `->`).
- **E2** trusts a model checked earlier in the session while its length and last-write time are
  unchanged, a load by the helper counting as a check (the helper refuses a file whose SHA-256
  differs).
- **E3** shows the session's length in whole minutes in the tray, so the state crosses IPC once
  a minute.
- **V2** gives every disabled primary button a neutral surface, a border and muted text at full
  opacity. **V4** puts each custom-colour chip at the end of its own swatches, labelled
  "Custom…" (its accessible name still says which colour).
- **C2** waits for CI rather than failing: `prepare` polls for up to an hour, because a tag is
  usually pushed while main's CI is still running. A backfill of an old tag whose CI run has
  been deleted now fails.
- **C3** also made `bundle` wait for `build`, so the release is still published only with its
  installers; the publish step checks they are attached.
- **C6:** `cargo-audit` stays at 0.22.2: 0.23.0 came out on 10 November and `install-action`'s
  manifest does not carry it yet. `version.test.ts` and `prepare` now check the helper crate's
  version too.
- **C7:** wasapi 0.27 takes `Direction`, `SampleType` and `StreamMode` by value and a session
  identifier on `initialize_client` (none is passed). sha2 0.11 needed no code change.

**Left**

- The two array copies per finalized line (`history.ts`, `transcriptLog.ts`) stay: the write
  snapshot is shallow and relies on the array being replaced, and the transcript is a store
  read by derived values.
- An optional later step for B2: the Whisper side saying how far the system source has been
  decoded would let held lines go exactly then.
- E6, S10 and V5, as marked above; C1, C8, C9 and D2 are the maintainer's.
- The workflows were not linted locally (no `actionlint`); CI lints them.

**Still to check by hand**

- **Zoom:** a rehearsal with sharing on posts nothing in a real meeting.
- **Echo filter:** a real room with loudspeakers, local Whisper under a backlog; whether the
  brief interim of each held line is noticeable; how long local speech over the loudspeaker
  waits; that the meter's session times line up with caption times.
- **Cloud engines:** how each provider's real mid-stream close codes are classified.
- **Graphics processor:** retry then fallback, and quitting during a hung load, with a real
  card through the Vulkan helper.
- **Audio devices:** a Bluetooth endpoint no longer causes repeated checks; renaming a device in
  Sound settings still refreshes the lists; system capture on wasapi 0.27 on x64 and ARM64.
- **Session:** Start, then Rehearse, after a run whose sources all failed; Stop straight after
  Start; no flash of the setup screen and no stuck clock.
- **Narrator:** each notice is read once (profile saved or loaded, no matching language, the
  application notes, no timing, long session); Rehearse reads only its sentence; the consent
  prompt opens on Cancel.
- **By eye:** the disabled Start in Dark, Light and the contrast themes; the custom-colour chips
  at 980 px in French and German; the tray's minutes in all three languages; the projector
  unplugged and plugged back (display watcher).
- **macOS:** the menu-bar item opens its menu on a click; the loopback's parking loop, which
  could not be compiled on Windows.
- **Release:** the first run of the CI gate, of `msix` without `build`, and of the shared
  compiler action, on a hyphenated validation tag (C1).
