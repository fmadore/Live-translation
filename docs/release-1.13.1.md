# Live Translation & Subtitles 1.13.1

The fixes from the Mac app's first run on a real Mac, on top of everything in 1.13.0. The
captions now stay over full-screen presentations, Stop never hangs on a permission question, and
a Mac that keeps system audio from the app says so instead of passing for a quiet room. A few of
the fixes reach Windows too.

## macOS

- **The overlay stays over full-screen apps.** In 1.13.0 the captions disappeared whenever
  another application was in full screen: a slideshow, Keynote or PowerPoint presenting, a
  browser in full screen. They now stay above it, in every Space, as they do on the desktop.
- **Stop no longer hangs on "Stopping…".** Stopping a system-audio session while macOS was still
  asking whether the app may record system audio left the window on "Stopping…" until the
  question was answered. Stop now ends the session within a few seconds.
- **A refused permission no longer looks like a quiet room.** If System Audio Recording has been
  refused, macOS does not report an error; the app simply receives silence. While something is
  playing and only silence arrives, the live status and the audio test now say so and name the
  setting to open: System Settings › Privacy & Security › Screen & System Audio Recording.
- **Clearer messages for what you can act on.** With no microphone connected, the app says so
  rather than pointing at the privacy settings, and the microphone list reads "No microphone
  found". When the application being captioned quits, or the output being captioned is unplugged,
  the message says exactly that.
- **The audio test** says "system audio" rather than Windows' "WASAPI loopback", and its tick no
  longer disappears when the test stops.
- **The Mac App Store version** no longer logs an error at every start, and opening it again
  still brings back the running copy.
- German's *Raummikrofon* label no longer breaks across two lines.

The Mac app was tried on an M2 Mac mini running macOS 26 for this release: system audio, one
application, local Whisper on the graphics processor, Gemini, export, the Keychain, the overlay
and the App Store's sandbox. Microphone capture, sleep and some overlay details have still to be
tried on a Mac with a microphone; the [macOS notes](https://github.com/fmadore/Live-translation/blob/main/docs/macos.md)
list what was and was not tested. Intel Macs are not supported.

## Windows

- **Stop cannot hang on a stuck audio device with local Whisper.** If a capture device stopped
  answering, Stop already gave up on it after three seconds, but local Whisper kept waiting for
  its audio to end. It now finishes with what it has.
- **Clearer messages** when the captioned application closes, when the captioned output is
  disconnected, and when no microphone is connected; a local Whisper source that loses its audio
  says the capture failed rather than that transcription could not finish.

Everything in [1.13.0](https://github.com/fmadore/Live-translation/releases/tag/v1.13.0) is
included: the Mac app, Whisper on the graphics processor, speech detection, Large v3 Turbo,
names and terms for Gemini Transcribe and Whisper, and the question before audio first goes to a
cloud provider. 1.13.0 was published on GitHub only and was not submitted to the Store, so the
Store goes from 1.12.0 straight to 1.13.1.

## Downloads

- **Windows:** for the Microsoft Store, upload
  [Live.Translation_1.13.1.msixbundle](https://github.com/fmadore/Live-translation/releases/download/v1.13.1/Live.Translation_1.13.1.msixbundle),
  with native x64 and ARM64 packages at version **1.13.1.0**; the Store signs accepted packages.
  The separate MSIX files are for per-architecture testing, and the EXE and MSI installers for
  installing outside the Store (they are unsigned).
- **macOS 15 or later, Apple silicon:** `Live.Translation_1.13.1_aarch64.dmg`, signed with the
  developer's Developer ID and notarised by Apple.

The [release handoff](https://github.com/fmadore/Live-translation/blob/main/docs/store-updates.md#release-1131-handoff)
records the package checks and the remaining Store steps.
