# Live Translation & Subtitles 1.13.0

Live Translation & Subtitles now runs on a Mac with Apple silicon, as a notarised disk image
with each release. Local Whisper gets faster and cleaner on both systems: it runs on the graphics
processor where there is a usable one, skips silence and music before it listens, no longer
writes the loops and stray credits it invents over noise, and offers a more accurate model,
Large v3 Turbo. The names-and-terms list now reaches Gemini Transcribe and Whisper too, and the
app asks once before audio is first sent to each cloud provider.

## macOS

- **A Mac app for Apple silicon, macOS 15 and later**
  ([#195](https://github.com/fmadore/Live-translation/issues/195),
  [#202](https://github.com/fmadore/Live-translation/pull/202)). Each release now attaches
  `Live.Translation_1.13.0_aarch64.dmg`, signed with the developer's Developer ID and notarised
  by Apple, so Gatekeeper opens it after its usual question. The interface, the engines, the
  overlay, the transcript, history and export are the same as on Windows.
- **What it listens to.** A microphone; the sound the Mac plays on a chosen output, through a
  Core Audio tap (macOS asks once for *System Audio Recording*; no screen is recorded); or one
  application and its helper processes. Safari plays through a process macOS shares with other
  apps, so it is captured with the output; the setup says so.
- **The overlay** floats above slides, calls and full-screen presentations in every Space, lets
  clicks through while captions run, and stays when the app is hidden with ⌘H.
- **macOS's words and keys.** The Keychain for API keys, the menu bar for the tray, System
  Settings for permissions, "this Mac" for "this PC", ⌘ shortcuts written as macOS writes them
  (⇧⌘P), in English, French and German. ⌘Q stops the session cleanly and asks about an unsaved
  transcript, as Quit in the menu bar does.
- **A Mac App Store version** (free, sandboxed, with no support link) is built and ready to
  upload, but has not been submitted yet.

The Mac app is built, tested and started on Apple silicon in GitHub Actions, Whisper's smoke
test included, but **has not yet been used on a real Mac**. System audio, application capture,
the overlay above full-screen apps and a refused permission are the parts to try first; please
report what does not work with
[a new issue](https://github.com/fmadore/Live-translation/issues/new/choose). Intel Macs are not
supported.

## Local Whisper: faster, quieter, more accurate

- **On the graphics processor**
  ([#208](https://github.com/fmadore/Live-translation/pull/208)). On a Mac, Whisper runs on the
  graphics processor through Metal. On Windows it runs on the graphics processor through Vulkan,
  in `whisper-vulkan.exe`, a small program installed with the app and started only by it: a card
  from NVIDIA, AMD or Intel's Arc A and B series first, otherwise integrated graphics. If the PC
  has none it can use, or the graphics processor fails or stops answering during a session,
  Whisper carries on on the processor without losing what it was transcribing. Snapdragon PCs
  stay on the processor, because their graphics processor computed wrong answers through Vulkan
  in Kalamu's tests. **Use the graphics processor**, under the model, turns it off; under it,
  the setup says where the latest session decoded and, if the graphics processor could not be
  used, what it reported, which the stage also shows during the session. How much faster it is
  depends on the computer and has not been measured yet: try your own event's speech with the
  switch on and off.
- **Speech detection.** Silero's voice-activity model, part of the app, looks for speech in each
  stretch of audio before Whisper does. A stretch without any is not transcribed at all, so a
  quiet room, a paused call or music costs nothing and Whisper has nothing to invent captions
  over; a stretch with speech is transcribed from just before the first words to just after the
  last, with its captions at their real times.
- **Loops and credits cut.** A word said over and over ("merci merci merci merci merci merci"),
  the same line three windows running, and the subtitle credits Whisper learnt from its training
  data ("Subtitles by the Amara.org community") are removed from the captions. Words people
  really repeat ("non, non, non") are kept.
- **Large v3 Turbo** (547 MiB), the most accurate offline model, for subtitles. It keeps up with
  speech on a graphics processor or a fast processor; try it before an event. It was trained
  without Whisper's translate task, so for translation into English keep Small: translation does
  not offer Turbo.

## One voice, one caption, with both sources

With **Both** sources, a meeting played through the room's loudspeakers also reaches the room
microphone, so its remote speaker used to be captioned twice. The app now leaves out the
microphone's copy: a microphone line is dropped when its words are, nearly enough, words the
system audio was captioned with at the same moment. It compares what the two engines wrote, not
the sound, so someone in the room speaking over the call is kept, and what it drops is always on
screen from the system audio. While the call is sounding, a finished microphone line waits a few
seconds (ten with Whisper) for the system line it may repeat before it reaches the transcript;
it is on screen meanwhile. It applies with every engine, and with headphones it simply finds
nothing to drop.

## Names and terms for Gemini Transcribe and Whisper

The setup's **Names and terms** list, until now sent only to Mistral, also goes to Gemini
Transcribe, which favours those spellings, and to local Whisper, which reads it before every
stretch of audio ([#193](https://github.com/fmadore/Live-translation/pull/193)). Whisper keeps
as many terms from the top of the list as its prompt holds, a few dozen, and the setup says so
under the list; a caption that only reads the list back over noise is dropped. The translation
engines document no vocabulary field, so they still take none.

## Asked once before audio goes to a cloud provider

Before the first session or rehearsal with Gemini, OpenAI or Mistral, the app now says which
company receives the audio, under whose account and terms, that the developer receives nothing,
and where the privacy policy is, and waits for **Send**; **Cancel** starts nothing. It asks once
per company, so both Gemini engines share one answer. Local Whisper and the built-in
demonstration send nothing and are never asked about.

## Under the surface

- The Windows packages carry `whisper-vulkan.exe` beside the app and the 864 KiB speech-detection
  model under `models/`; the release checks that each MSIX package holds both. The helper
  reaches the graphics processor through the driver already installed, opens no network
  connection, and adds no MSIX capability. Whisper's decoding settings and the messages between
  the app and the helper are one source file compiled into both.
- whisper.cpp's own speech-detection switch does nothing through the interface the app uses, so
  the app runs the voice-activity model itself.
- `src-tauri` is a Cargo workspace with the helper as its second package, built only for
  releases, with LunarG's Vulkan SDK; `tauri dev` and ordinary builds do not need it.
- CI builds and tests the helper on Windows and Linux, and runs the whole Whisper smoke test
  through it on Mesa's software renderer; the macOS lanes run on macOS 26 with Xcode 26.
- Dependency updates: tokio 1.54.1 and other Rust patch releases, and npm overrides of minimatch
  and glob for audit advisories.

With Whisper, audio still never leaves the computer: on Windows it reaches the helper only
through the computer's own memory. Cloud translation and cloud subtitles still require your own
provider account and API key, and may incur usage charges; Captions in Zoom, when you start it,
still sends caption text only. The [privacy policy](privacy.md) describes each.

For the Microsoft Store, upload
[Live.Translation_1.13.0.msixbundle](https://github.com/fmadore/Live-translation/releases/download/v1.13.0/Live.Translation_1.13.0.msixbundle),
containing native x64 and ARM64 packages at version **1.13.0.0**. The Store signs accepted
packages. Separate MSIX files are provided for per-architecture testing; EXE/MSI installers are
for installation outside the Store. `Live.Translation_1.13.0_aarch64.dmg` is the Mac app.
