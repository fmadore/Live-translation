# Live Translation on macOS

Tracking issue: [#195](https://github.com/fmadore/Live-translation/issues/195). This page
records what the macOS port does, why, how to get a test build, and the checklist for a first run
on a real Mac (a rented Scaleway Mac mini). The Mac App Store's own steps are in
[mac-app-store.md](mac-app-store.md).

## Status (11 November 2026)

**First run on a real Mac done; its bugs fixed.** CI compiles, lints and tests the Rust core on
Apple silicon, runs the multilingual Whisper smoke test there, builds the app and starts it on
the runner, in and out of the App Sandbox. On 10–11 November the app was built from source and
driven on an M2 Mac mini (macOS 26.7.3, no microphone, speakers and an HDMI display), both the
direct build and the sandboxed one, through the accessibility interface; see
[First run](#first-run-1011-november-2026). *Tested* below means that run, not a real meeting.

| Area | macOS implementation | State |
| --- | --- | --- |
| Microphone | cpal (Core Audio), as on Windows | error paths tested (no microphone); capture to test |
| System audio | Core Audio process tap on the chosen output (`audio/tap.rs`) | tested, direct and sandboxed |
| Application capture | a tap of the application's process and its helpers | tested (direct) |
| Device changes | Core Audio notifications, as Windows' endpoint notifications | tested |
| Caption overlay | screen-saver window level, every Space, beside full-screen apps; not key while click-through | level, transparency, move mode tested; click-through, Spaces, full screen to test |
| Transcript export | native Save panel (`NSSavePanel`, through rfd); atomic write | tested, direct and sandboxed |
| External links | `NSWorkspace` opens the support page and the Zoom guide | to test |
| API keys | login Keychain (keyring's Apple backend) | tested, direct and sandboxed |
| Local Whisper | Metal on the graphics processor (switch in the setup), CPU fallback with ARMv8.2 dot-product and FP16 and Accelerate's BLAS | tested (Metal, Base model) |
| Quit and Dock | ⌘Q runs the tray's Quit (stop, finalize, ask); Dock icon reopens the window | tested |
| Interface | macOS's words in EN/FR/DE (`i18n/mac.ts`); ⌘ shortcuts written ⇧⌘P | tested |
| Packaging | `.app` and `.dmg`, ad hoc in CI; signed and notarised in each release | built locally; notarised build to test |
| Mac App Store | free; sandboxed build, signing and upload workflow; no support link | sandbox spike tested, not submitted |

## Decisions

- **Apple silicon only, macOS 15 and later for now.** The Core Audio tap API exists from macOS
  14.2 and cpal documents its loopback from 14.6; 15 is the issue's provisional baseline and the
  oldest release likely to be tested soon. Lower it to 14.6 once it has run there.
  `src-tauri/tauri.macos.conf.json` sets it, which also sets the deployment target the native
  Whisper build compiles for.
- **System audio through a Core Audio tap, not ScreenCaptureKit.** The issue proposed
  prototyping ScreenCaptureKit first and taps if that made audio-only capture awkward. A tap is
  audio only by design, needs no screen-recording permission, and its process list gives
  application capture directly. `src-tauri/src/audio/tap.rs` creates a private process tap and
  a private aggregate device holding only that tap, reads it from Core Audio's I/O callback, and
  destroys both when the source stops. macOS asks once for *System Audio Recording*, with the
  text of `NSAudioCaptureUsageDescription`. The tap's float buffers go to the same
  `CaptureState` as the microphone's: downmix, resampling, chunks and meter.
  - cpal 0.18 can also record an output through a tap, and the first version used it, but it
    opens a device's *input* when the device has one: a USB headset or speakerphone would have
    captioned its microphone while the operator believed it was the computer's sound. Our own
    tap names the output by its UID, so every output works the same.
  - **The output mode** taps every process playing to the chosen output. The stream stays on
    that output, as on Windows: it never follows a new default, and an output that disappears
    ends the source with an error.
  - **The application mode** taps a stereo mix of the selected application's own process and
    its helpers, the processes whose bundle identifier extends the application's
    (`com.google.Chrome.helper` for `com.google.Chrome`), which is where browsers and Electron
    apps play. The list is checked every second, so a helper that starts playing later joins.
    The application is identified by PID and start time (`proc_pidinfo`), so a recycled PID
    never stands in for it, and its exit ends the source with an error: it never widens to the
    whole output. Safari plays through a WebKit process macOS shares with other apps, so it is
    captured with the output mode; the interface says so. This is not Windows' process tree,
    and browser tabs are not isolated.
  - **A refused permission may look like silence.** Whether macOS fails the tap or feeds it
    zeros when System Audio Recording is refused is one of the things to test; the level meter
    shows it either way. The acceptance criterion is that a refusal never passes for a working
    session, so this needs an answer before a release is announced.
- **Device changes** are reported by Core Audio property listeners (devices, default input,
  default output), which wake the capture threads' presence checks and refresh the interface's
  lists, as Windows' endpoint notifications do.
- **The overlay floats by window level, not a raising timer.** On Windows the overlay is raised
  every 500 ms because topmost windows compete within one band. On macOS it is set once to the
  screen-saver level, joins every Space, is allowed beside full-screen windows (Keynote,
  PowerPoint, a browser in full screen), and stays when the app is hidden with ⌘H. The timer
  does not run on macOS. While click-through it cannot become the key window (tao's
  `focusable`), the counterpart of Windows' `WS_EX_NOACTIVATE`; move mode makes it focusable
  again. Public AppKit only (`NSWindow` level, collection behaviour, `orderFrontRegardless`).
  Tauri 2.12's `macOSPrivateApi` is a no-op and is not set; transparency is Tauri's own.
- **Export keeps its atomic write.** The Save panel replaces Windows' Save As through the same
  crate (rfd). Inside the App Sandbox the panel grants the chosen file and nothing beside it, so
  the staging file cannot be made in that folder; only then is the file written in place
  (`export::write_export`). Outside the sandbox nothing changes.
- **⌘Q is the app's own Quit.** Tauri's default macOS menu is kept for its Edit shortcuts (⌘V
  to paste a key), with Quit replaced by one that runs the tray's Quit: the system's would end
  the process at once, without stopping the session cleanly or asking about an unsaved
  transcript. Closing the operator window behaves as on Windows (the app quits, or hides to the
  menu bar if the operator asked for that); the Dock icon brings a hidden window back.
- **The interface speaks macOS where it differs.** `src/lib/platform.ts` is the interface's one
  test of the system. `src/lib/i18n/mac.ts` replaces, in all three languages, every message that
  names a Windows place: the Keychain for Credential Manager, the menu bar for the notification
  area, System Settings paths for `ms-settings:` ones, "this Mac" for "this PC". Components are
  unchanged. Shortcuts use Command, are announced as Meta and written as macOS writes them
  (⇧⌘P); Control is left alone, as Windows leaves the Windows key.
- **Per-feature `cfg`, as before.** The Windows code already keeps each system's implementation
  beside the shared logic of its feature (`loopback.rs`, `overlay.rs`, `export.rs`,
  `support.rs`); macOS follows the same pattern rather than a new `platform/` layer.
- **Distribution.** Each GitHub release attaches an Apple silicon disk image signed with the
  Developer ID and notarised (`release.yml`, job `macos`), and is published only once it is
  there. The Mac App Store version is free, like Windows; the trial and unlock proposed in #195
  were dropped on 4 November 2026. It is the only sandboxed build, which the core recognises by
  `APP_SANDBOX_CONTAINER_ID`, and shows no support link (App Review Guideline 3.1.1). See
  [mac-app-store.md](mac-app-store.md).
- **Two signed variants of one build for testing** (`.github/workflows/macos.yml`):
  - the direct app, in a disk image, with the hardened runtime and one entitlement, the
    microphone (`src-tauri/macos/Entitlements.plist`), as a Developer ID build will be;
  - the sandbox spike, the same app re-signed ad hoc inside the App Sandbox
    (`src-tauri/macos/Sandbox.plist`: network client, microphone, user-selected files), to learn
    early what a Mac App Store build breaks. No App ID: that needs a provisioning profile.

## Getting a test build

1. GitHub → **Actions → macOS test build**. Every push to a branch of the repository that
   touches the app builds one. For a signed and notarised build, **Run workflow** on the branch
   with *sign* ticked (needs the secrets below).
2. Download the artifact `live-translation-macos-<commit>` (kept 14 days). It holds:
   - `Live Translation_<version>_aarch64.dmg`, the direct app;
   - `Live-Translation-sandbox.zip`, the sandboxed app;
   - `commit.txt`, the commit it was built from.
3. Copy it to the Mac. From Windows: `scp live-translation-macos-*.zip user@<address>:~/`.
   A file copied this way is not quarantined, so Gatekeeper does not ask; downloaded in Safari,
   an ad-hoc build is refused as "damaged" until it is cleared:
   `xattr -dr com.apple.quarantine "/Applications/Live Translation.app"`. A notarised build
   only asks the usual question.

### Developer ID secrets

Needed by every release (the disk image) and optional for a test build. The Developer ID
certificate made for Kalamu belongs to the team (`3HMSG8WB7K`), not to one app, so the same six
repository secrets work here (Settings → Secrets and variables → Actions):
`APPLE_CERTIFICATE` (the `.p12`, base64), `APPLE_CERTIFICATE_PASSWORD`,
`APPLE_SIGNING_IDENTITY` (`Developer ID Application: Frédérick Madore (3HMSG8WB7K)`),
`APPLE_ID`, `APPLE_PASSWORD` (an app-specific password) and `APPLE_TEAM_ID`. The test-build workflow
reads them only when run by hand with *sign* ticked, never on a push, and both it and the
release stop if one is missing rather than sign ad hoc silently. A signed build keeps its microphone and System Audio
Recording permissions and its Keychain access across rebuilds; an ad-hoc build may be asked
again after each one.

## Test checklist: a day on a Scaleway Mac

A Scaleway Mac mini has speakers but **no microphone**, and VNC carries neither direction of
sound. So system audio is tested by playing files on the Mac itself (`afplay`), and the
microphone only for its permission and error paths, unless a virtual input such as BlackHole
is installed for the day. Tick each box and note failures in [Results](#results): what you did,
what you expected, what happened, with a screenshot or the log.

**Before:** a cloud provider test key with a low spending cap (Gemini, OpenAI or Mistral), the
artifact on the Mac, and macOS's version noted (`sw_vers`). To keep the app's log, start it
from Terminal:

```bash
RUST_LOG=live_translation_lib=debug,warn "/Applications/Live Translation.app/Contents/MacOS/live-translation" 2>&1 | tee ~/lt-direct.log
```

The speech samples ship inside the app:
`/Applications/Live Translation.app/Contents/Resources/fixtures/rehearsal-en.wav` and
`rehearsal-fr.wav`. Play one, in a second Terminal, with `afplay <file>`.

### Part A: the direct app (disk image)

- [ ] Open the disk image, drag the app to Applications, open it. Both windows appear: the
      operator window, and the transparent caption overlay with no frame or shadow.
- [ ] **Keyless demonstration:** the engine *Built-in demo* plays captions with no key, no
      network and no audio captured.
- [ ] **Whisper offline:** download the Tiny or Base model; start a session with *system audio*
      as the source and the Mac mini's speakers as the output; `afplay rehearsal-en.wav`.
      macOS asks for *System Audio Recording*, naming Live Translation with the description
      text. Allow: captions appear in the overlay and the transcript; the level meter moves.
- [ ] **Whisper on Metal:** the log says `Whisper model loaded device=metal`; time the same
      recording with *Use the graphics processor* on and off (`device=cpu`), and try Large v3
      Turbo. A silent stretch produces no captions (speech detection).
- [ ] **French and translation:** the same with `rehearsal-fr.wav`, spoken language French, and
      translation into English.
- [ ] **Refusal:** System Settings → Privacy & Security → *Screen & System Audio Recording*
      (*System Audio Recording Only* section): turn Live Translation off, restart the app, start
      again. Note exactly what happens: an error, or a session that runs silent. A silent
      session counts as a failure to fix.
- [ ] **Output list:** the system-audio picker lists the speakers and any other output. If a
      USB headset is at hand, it is listed and its output is captured, not its microphone.
- [ ] **One application:** open a YouTube video in Chrome (or Firefox, or play a file in
      QuickTime Player); in system capture choose *One application* and that app. Captions
      follow it; `afplay` in Terminal at the same time is not captioned. Quit the app: the source
      stops with an error and nothing else is captured. Try Safari too and note what happens
      (expected: silence, as the interface warns).
- [ ] **Device change:** with a session running on the speakers, plug or unplug an output
      (or create one in Audio MIDI Setup). The device lists refresh without pressing Refresh.
- [ ] **Microphone path:** with no input device, starting a microphone session fails with an
      error rather than running silent. With BlackHole installed, macOS asks for the
      microphone once with the description text, and captions follow what is played into it.
- [ ] **Cloud provider:** paste the test key (⌘V works in the field), start a cloud session on
      system audio, play a sample. Quit and reopen: the key is still there (Keychain). Note any
      Keychain password prompt.
- [ ] **Overlay:** move mode lets you drag and resize it; locking makes it click-through again
      (clicks reach the window behind). Typing in another app while captions run: the overlay
      never takes the keyboard.
- [ ] **Full screen:** open a file in Keynote or a video in Safari full screen; the overlay
      stays above it. Switch Spaces (Control-→): it follows. ⌘H on Live Translation: the
      overlay stays.
- [ ] **Export:** save the transcript as TXT, Markdown, WebVTT and SRT: the Save panel opens,
      remembers the folder, asks before replacing an existing file, and writes a folder and file
      name with accents (`~/Documents/Données Frédérick/transcription.md`).
- [ ] **Links:** the support link and the Zoom guide open in Safari.
- [ ] **Quit:** with unsaved lines, ⌘Q asks about them; with a live session, it stops it first.
      With *Keep running in the menu bar when I close this window* on, the red button hides the
      window; the menu-bar icon and the Dock icon bring it back.
- [ ] **Sleep:** with a session running, sleep the Mac (Apple menu → Sleep) and wake it over
      VNC. Note what the source does.
- [ ] **WebKit:** make the operator window narrow and then full screen; nothing scrolls
      sideways or overlaps. Switch the interface to French and German.
- [ ] **Mac wording:** Settings and the API key panel say Keychain and menu bar, never
      Credential Manager, the notification area or "this PC", in all three languages; the
      keyboard help writes ⇧⌘Space, and ⇧⌘Space starts and stops a session.

### Part B: the sandboxed app

Quit the direct app first. Unzip `Live-Translation-sandbox.zip` (Finder or `ditto -x -k`) and
start `Live Translation (sandbox).app` from Terminal the same way, logging to
`~/lt-sandbox.log`.

- [ ] It starts; its data is in `~/Library/Containers/io.github.fmadore.live-translation/`.
- [ ] Whisper: the model downloads into the container and captions system audio
      (`afplay`). **The main question of the spike: does the tap work inside the sandbox?**
- [ ] Cloud provider over the network; the key saved and read back from the Keychain.
- [ ] Export to `~/Documents` and to a new folder; replacing an existing file works. Note
      whether the log says the write fell back to writing in place.
- [ ] Links open in Safari; the support link (Buy Me a Coffee) is absent from Settings and the
      start screen, as in the store build.
- [ ] One application, as in Part A: does the tap work for a single app inside the sandbox?

### Results

Copy into the issue, one line per failure; keep "CI passed", "native automated test passed"
and "real meeting passed" apart.

| Part | Step | Build (commit, signing) | macOS, Mac | What happened |
| --- | --- | --- | --- | --- |
| | | | | |

## First run (10–11 November 2026)

An M2 Mac mini, macOS 26.7.3, no microphone; built from `main` at `f42ae86` with
`npm run tauri build` (ad hoc signature, not notarised), then retested with the fixes below.
The windows were driven through the macOS accessibility interface rather than by hand, so
everything here is "native automated test passed", not "real meeting passed".

**Worked as designed:** the README's checks (921 Vitest tests, 430 Rust tests, Clippy, the
Whisper smoke test on Metal); the bundle (hardened runtime, microphone entitlement, both usage
descriptions, macOS 15 minimum, arm64 only, no non-system library); both windows, the overlay
at the screen-saver level and fully transparent; the keyless demo; the Base model downloaded,
verified and run on Metal (Rehearse); System Audio Recording asked for with the app's own text,
and captions from `afplay` once allowed, without restarting the session; one application
(QuickTime Player) captured while `afplay` was left out, and its quitting ending the source
instead of widening to the output; device lists refreshed on an output appearing and going, and
a removed output ending the source; the Save panel (accented name, Replace, no staging file left
behind); a key saved to the login Keychain, kept across a restart with no prompt, and removed;
⇧⌘Space, ⇧⌘P and ⇧⌘O; ⌘Q stopping a live session and asking about unsaved lines; hiding to the
menu bar and back through the Dock; overlay move mode (arrows nudge, Enter locks, no longer
focusable afterwards); the Mac wording in all three languages. **In the App Sandbox:** the tap
captures system audio (the spike's question), the model downloads into the container, export
writes in place and truncates a longer file it replaces, and the support link is absent.

**Found and fixed on this branch:**

- *Stop hung on "Stopping…"* when a system-audio session was stopped while the System Audio
  Recording question was still on screen. Until it is answered, `coreaudiod` makes every I/O
  call of the app wait (stopping or destroying an I/O proc, creating the next one), and the
  capture thread waiting there still held the session's audio channel, which a local Whisper
  source reads until it closes. Stop already gave up on such a thread after three seconds; now
  the source stops waiting for its input too (`ActiveSession::abandoned`), and the tap lets go
  of its capture state first and is taken down on a thread of its own (`audio/tap.rs`). Retested:
  Stop takes four seconds with the question unanswered.
- *A refused permission looked like a quiet room*: after "Don't Allow" the tap runs and delivers
  exact zeros, with no error. While another process plays into what is tapped and only zeros
  arrive for three seconds, the meter events are marked `silenced` and the interface names the
  setting to open (`SilenceWatch` in `audio/loopback.rs`). Retested: the hint appeared three
  seconds into `afplay` on a refused build and went once it stopped. There is no public API to
  ask for the permission itself; a stream that really plays digital silence (a muted call that
  keeps its output open) shows the same hint, which disappears with the first real sound. While
  the question is still on screen nothing is shown: Core Audio answers nothing until then, and
  the question is in front of the operator anyway.
- *No microphone* was reported as a blocked privacy setting (`'!obj'` from a default input
  device that does not exist) and the picker still offered "System default"; it now says no
  microphone is connected, in both places.
- *A closed application or a disconnected output* is worded in full (`error.applicationClosed`,
  `error.outputDisconnected`) instead of an English detail, and a local Whisper source reports a
  capture failure as the capture's, not as "Local transcription could not finish".
- *The single-instance plugin failed in the sandbox* (its socket is in `/tmp`) and logged an
  error at every start; it is not installed there, where Launch Services keeps one instance.
- *RAUMMIKROFON broke as RAUMMIKROFO/N* in WebKit, whose text is a little wider than WebView2's,
  and the hidden date input of History overflowed its box by 2px; the tray logged a warning on
  every ⌘Q for the application menu's event.

**Found while taking the App Store screenshots, and fixed:**

- *The overlay vanished over full-screen applications* (Preview in full screen and its slideshow;
  the same holds for Keynote, PowerPoint or a browser in full screen). For an application with a
  Dock icon, macOS lets a window into another application's full-screen Space only if it is a
  non-activating panel, whatever its level and collection behaviour; tried in a prototype, a
  plain window with the panel's style bit stays out and a panel stays in. The overlay's class is
  now switched to an `NSPanel` subclass that keeps tao's `focusable` ivar and focus methods
  (`overlay.rs`), checked to have the same layout first. Retested: on the desktop, over a
  full-screen window and over a slideshow; move mode, the arrow keys and Enter still work.
- *The audio test said "WASAPI loopback"* on a Mac, in all three languages; `i18n/mac.ts` now says
  "system audio".
- *The audio check's tick went away when the test stopped.* The tap's private aggregate device is
  invisible to other applications but not to this one, which saw it as an input that appeared and
  vanished with each capture, as if a device had been plugged in. Every device list now leaves
  the app's own devices out (`audio::is_own_device`).

**Still open:** a history entry's duration can run past the session (one recorded "17:08" for a
talk of under a minute that was stopped after three minutes), as if the entry stayed open after
Stop until the next action.

**Cloud provider, after the fixes:** Gemini live translation (English sample to French) through
Rehearse, in the direct and the sandboxed build: the WebSocket connects through the system's
TLS (Security.framework, where Windows uses SChannel) and is set up in under half a second,
the captions arrive in French, Gemini's planned session handovers (about every nine minutes)
reconnect without a gap, Stop drains and ends cleanly, and the sandbox's network entitlement
covers the WebSocket. The key there was copied into the app's entry with `security
add-generic-password`, so macOS asked for the login password at the first two reads in each
build (one to check a key exists, one at Start); a key pasted into the app's own field asks
nothing. While that question waits, Start only says "Starting…". Not tried: OpenAI, Mistral,
and a cloud session on system audio rather than the bundled sample.

**Not done in that run:** microphone capture (no input device), click-through, Spaces and full
screen, sleep, the support and Zoom links, SRT/WebVTT export, and turning System Audio Recording off in System Settings after allowing it (the refusal was
tested by answering "Don't Allow"). Also seen: the first session after a restart
waits about 7 s on "Starting…" while Metal compiles Whisper's shaders, and the application menu
stays in English when the interface is French or German.

**Building locally:** `npm run tauri build` makes the disk image by asking Finder, through
AppleScript, to lay out its window; from a terminal without the Automation permission that
waits for an answer and then fails. `CI=true npm run tauri build` skips the layout, as CI does.

## Not done yet

1. **The rest of the first-run checklist** (above); then the decisions the issue leaves to
   evidence: the oldest macOS supported (14.6 if it runs there), and Whisper's speed on Metal,
   now on by default, measured against the CPU with the setup's graphics-processor switch.
2. **A refused System Audio Recording permission** is now flagged while something plays (seen
   after "Don't Allow"); still to check after turning it off in System Settings.
3. **The Developer ID secrets** in this repository before the next release tag, or the release's
   `macos` job stops (on purpose) and the release stays a draft until they are added and the
   job is re-run.
4. **The Mac App Store submission**: the steps in [mac-app-store.md](mac-app-store.md), the
   screenshots on a Mac, and a TestFlight run.
5. **README badge and store link** once the Mac App Store version is public.
