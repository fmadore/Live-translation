# Live Translation on macOS

Tracking issue: [#195](https://github.com/fmadore/Live-translation/issues/195). This page
records what the macOS port does so far, why, how to get a test build, and the checklist for a
first run on a real Mac (a rented Scaleway Mac mini).

## Status (4 November 2026)

**First implementation; nothing has run on a real Mac yet.** CI compiles, lints and tests the
Rust core on Apple silicon, runs the multilingual Whisper smoke test there, builds the app and
starts it on the runner, in and out of the App Sandbox. Everything below marked *to test* needs
a Mac with a person at it. The interface still uses Windows wording in places (see
[Not done yet](#not-done-yet)).

| Area | macOS implementation | State |
| --- | --- | --- |
| Microphone | cpal (Core Audio), as on Windows | to test |
| System audio | Core Audio process tap on the chosen output, through cpal | to test |
| Application capture | not offered: the picker reports it unsupported | deferred |
| Caption overlay | screen-saver window level, every Space, beside full-screen apps; not key while click-through | to test |
| Transcript export | native Save panel (`NSSavePanel`, through rfd); atomic write | to test |
| External links | `NSWorkspace` opens the support page and the Zoom guide | to test |
| API keys | login Keychain (keyring's Apple backend) | to test |
| Local Whisper | CPU, ARMv8.2 dot-product and FP16, Accelerate's BLAS; Metal off | CI smoke test |
| Quit and Dock | ⌘Q runs the tray's Quit (stop, finalize, ask); Dock icon reopens the window | to test |
| Packaging | `.app` and `.dmg`, ad hoc by default, Developer ID and notarisation on demand | CI |
| Mac App Store | sandbox spike build only; no StoreKit, no upload | spike |

## Decisions

- **Apple silicon only, macOS 15 and later for now.** The Core Audio tap API exists from macOS
  14.2 and cpal documents its loopback from 14.6; 15 is the issue's provisional baseline and the
  oldest release likely to be tested soon. Lower it to 14.6 once it has run there.
  `src-tauri/tauri.macos.conf.json` sets it, which also sets the deployment target the native
  Whisper build compiles for.
- **System audio through a Core Audio tap, not ScreenCaptureKit.** The issue proposed
  prototyping ScreenCaptureKit first and taps if that made audio-only capture awkward. cpal
  0.18, already the microphone's library, records an output device through a tap: opening an
  output for input makes it create a private process tap of every process's output and a
  private aggregate device to read it, both destroyed with the stream
  (`cpal::host::coreaudio::macos::loopback`). So the system source is audio only, adds no
  dependency, needs no screen-recording permission, and reuses the microphone's conversion,
  resampling and metering (`audio::capture::open_stream`). macOS asks once for *System Audio
  Recording*, with the text of `NSAudioCaptureUsageDescription`.
  - **An output that also has inputs is refused.** For a device with an input of its own (a USB
    headset or speakerphone) cpal opens that input instead of tapping the output, which would
    caption a microphone while the operator believes it is the computer's sound. Such outputs
    are left out of the list and refused by id (`audio::devices::tappable`). Capturing them needs
    a tap of our own (`CATapDescription` with the device's UID); so does application capture,
    with a process list. Both are the next audio step if testing shows they matter.
  - **A refused permission may look like silence.** Whether macOS fails the tap or feeds it
    zeros when System Audio Recording is refused is one of the things to test; the level meter
    shows it either way. The acceptance criterion is that a refusal never passes for a working
    session, so this needs an answer before a beta.
  - The stream stays on the output it opened, as on Windows: it never follows a new default,
    and an output that disappears ends the source with an error (checked every second; macOS
    sends this app no device-change notification yet).
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
- **Per-feature `cfg`, as before.** The Windows code already keeps each system's implementation
  beside the shared logic of its feature (`loopback.rs`, `overlay.rs`, `export.rs`,
  `support.rs`); macOS follows the same pattern rather than a new `platform/` layer.
- **Two signed variants of one build** (`.github/workflows/macos.yml`):
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

### Developer ID secrets (optional)

The Developer ID certificate made for Kalamu belongs to the team (`3HMSG8WB7K`), not to one app,
so the same six repository secrets work here (Settings → Secrets and variables → Actions):
`APPLE_CERTIFICATE` (the `.p12`, base64), `APPLE_CERTIFICATE_PASSWORD`,
`APPLE_SIGNING_IDENTITY` (`Developer ID Application: Frédérick Madore (3HMSG8WB7K)`),
`APPLE_ID`, `APPLE_PASSWORD` (an app-specific password) and `APPLE_TEAM_ID`. The workflow reads
them only when run by hand with *sign* ticked, never on a pull request, and stops if one is
missing rather than sign ad hoc silently. A signed build keeps its microphone and System Audio
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
- [ ] **French and translation:** the same with `rehearsal-fr.wav`, spoken language French, and
      translation into English.
- [ ] **Refusal:** System Settings → Privacy & Security → *Screen & System Audio Recording*
      (*System Audio Recording Only* section): turn Live Translation off, restart the app, start
      again. Note exactly what happens: an error, or a session that runs silent. A silent
      session counts as a failure to fix.
- [ ] **Output list:** the system-audio picker lists the speakers (and any other output); an
      output that also has a microphone is absent.
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
      With *Keep running in the tray when I close this window* on (still worded for Windows), the
      red button hides the window; the menu-bar icon and the Dock icon bring it back.
- [ ] **Sleep:** with a session running, sleep the Mac (Apple menu → Sleep) and wake it over
      VNC. Note what the source does.
- [ ] **WebKit:** make the operator window narrow and then full screen; nothing scrolls
      sideways or overlaps. Switch the interface to French and German.

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
- [ ] Links open in Safari.

### Results

Copy into the issue, one line per failure; keep "CI passed", "native automated test passed"
and "real meeting passed" apart.

| Part | Step | Build (commit, signing) | macOS, Mac | What happened |
| --- | --- | --- | --- | --- |
| | | | | |

## Not done yet

In rough order:

1. **What the Scaleway day finds**, fixed first.
2. **Interface wording and shortcuts for macOS.** Texts in all three languages still name
   Windows places: Credential Manager, Windows Settings privacy pages, the notification area,
   "this PC", Windows build 20348 for application capture, the Windows default device. The
   shortcut help writes Ctrl; on a Mac it should be ⌘ where the shortcut is a command
   shortcut. A capability from the core (system, application capture available) should drive
   these rather than components testing the system.
3. **Audio:** device-change notifications on macOS; a tap of our own for outputs that have
   inputs and for capturing one application (or an explicit deferral in the interface); a
   definite answer for a refused System Audio Recording permission.
4. **Release:** the disk image in `release.yml`, signed and notarised; the README, privacy
   policy (with a new effective date), security policy and store listing naming macOS.
5. **Mac App Store** (Phase 3 of #195): publishing team and paid-app agreement, provisioning,
   StoreKit trial and unlock, App Store review notes; the support link reviewed against
   guideline 3.1.1, as Kalamu hides it in its store build.
