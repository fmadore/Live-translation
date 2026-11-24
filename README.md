# Live Translation & Subtitles

Live captions over meetings, lectures, video calls and slides on Windows and macOS. Run speech
transcription locally with **Whisper**, or translate into English offline and into dozens of other
languages with your own **Google Gemini**, **OpenAI** or **DeepL** API key, all in a transparent
overlay.

<a href="https://apps.microsoft.com/detail/9PFB8LR3RR9X?referrer=appbadge&mode=direct"><img src="https://get.microsoft.com/images/en-us%20dark.svg" alt="Get it from Microsoft" height="48"></a>&nbsp;&nbsp;<a href="https://buymeacoffee.com/fmadore"><img src="https://cdn.buymeacoffee.com/buttons/v2/default-yellow.png" alt="Buy me a coffee" height="48"></a>

Free, with every feature included. If the app is useful to you, you can
[support its development](#support-development).

[Website](https://fmadore.github.io/Live-translation/) ·
[GitHub releases](https://github.com/fmadore/Live-translation/releases) ·
[Privacy](docs/privacy.md) · [Documentation](docs/README.md)

**Windows 11, native x64 and ARM64 · macOS 15+ on Apple silicon (in testing) · English, French
and German interface · MIT licence**

- **Latest release:** **[1.16.0](https://github.com/fmadore/Live-translation/releases/tag/v1.16.0)**
  (23 November 2026; Microsoft Store submission pending). Adds OpenAI live subtitles, one-click
  fallback to offline Whisper when a cloud engine drops out mid-session, fixed Gemini subtitles
  that no longer repeat earlier speech, buffered opening words after Start, and fixes from the
  20 November audit ([release notes](docs/release-1.16.0.md),
  [Store handoff](docs/store-updates.md#release-1160-handoff)).
- **Microsoft Store:** **[1.15.0](https://github.com/fmadore/Live-translation/releases/tag/v1.15.0)**
  (17 November 2026) is currently live in the
  [Microsoft Store](https://apps.microsoft.com/detail/9PFB8LR3RR9X), bringing the cloud spending
  guard, transcript markers and Find, break notices on the overlay, optional system-wide shortcuts,
  and bulk actions in Transcript history ([release notes](docs/release-1.15.0.md)).
- **macOS:** Each GitHub release includes a notarised disk image, tested on an M2 Mac mini running
  macOS 26 ([macOS notes](docs/macos.md)). A free Mac App Store build (1.16.0) is currently in
  Apple's review ([submission notes](docs/mac-app-store.md)).

## What it does

- **Offline subtitles.** Whisper runs locally with 99 spoken languages or automatic detection,
  using your graphics processor when available (Metal on macOS, Vulkan on Windows) and falling back
  to the CPU otherwise. Download Tiny (31 MiB), Base (57 MiB), Small (182 MiB) or Large v3 Turbo
  (547 MiB) once, then caption completely offline with no account or API key. Built-in speech
  detection skips silence and music before Whisper processes them. Speed and accuracy depend on
  your hardware, model and language; on x64, Whisper requires AVX2.
- **Offline translation into English.** Whisper translates speech from any of its 99 languages into
  English locally, with no API key (Large v3 Turbo only transcribes). Pick Small for translation,
  as Tiny and Base translate poorly.
- **Cloud translation and subtitles.** Translate into 78 languages with Gemini, 42 with DeepL or
  13 with OpenAI using your own API key and internet connection (provider usage charges apply).
  DeepL requires a paid DeepL API plan and is marked experimental because it has not yet run in a
  live session. Gemini, Mistral and OpenAI also provide same-language subtitles, with OpenAI
  reusing its translation key. A custom list of names and terms helps Gemini, Mistral, OpenAI and
  Whisper spell speaker names, institutions and jargon accurately. If a cloud engine stops
  mid-session, you can switch to offline Whisper in one click without ending the session.
- **Flexible audio capture.** Capture a microphone, system audio, a single application (its process
  tree on Windows, or the app and its helpers on macOS), or microphone and system audio together.
  When capturing both, an echo filter suppresses the remote call picked up by the room microphone
  so the call is captioned once. Check input levels before going live.
- **Captions where people look.** Place a transparent, always-on-top, click-through overlay over
  slides, calls or any other window; it remembers its position for each display setup. Adjust
  typeface, colour, contrast, size and reading pace, and switch between Fit window, Compact and
  Stable reading, with right-to-left script support. For game voice chat in another language, run
  the game in windowed or borderless mode so the separate overlay window sits cleanly on top.
- **Bilingual audiences.** Display two translation languages side by side, or show the original
  speech beneath translations in Fit window or Compact when the engine supplies it. Adding a second
  translation language opens an extra cloud session per audio source and increases provider usage.
- **Meeting controls.** Pause and resume without splitting the transcript, save named meeting
  profiles, search or pin caption languages, and edit the overlay's filler-word filter while
  keeping the raw transcript intact.
- **A transcript you can keep.** Follow the full transcript in its own tab during a session, copy
  the last 5, 10 or 30 minutes as plain text, and export to text, Markdown, SRT or WebVTT. Optional
  local history adds session titles, search and reopening; optional crash recovery protects unsaved
  captions. Both are off by default.
- **Captions in Zoom** (since 1.12.0). Send one caption language directly to a Zoom meeting's
  built-in caption bar using the API token copied by the host, so attendees simply turn captions on
  in Zoom. Only caption text leaves the app, and only once you start sharing
  ([guide](docs/zoom-captions.md)). Zoom is a trademark of Zoom Communications, Inc.; this
  project is not affiliated with Zoom.

## Start captioning

1. Install from the [Microsoft Store](https://apps.microsoft.com/detail/9PFB8LR3RR9X) or grab an
   installer from [GitHub releases](https://github.com/fmadore/Live-translation/releases) (Whisper
   requires 1.6.0+, and offline English translation requires 1.7.0+). Store packages are signed and
   update automatically; GitHub installers are unsigned and can check for new releases once a day
   if you opt in (see [new versions](#new-versions)).
2. For offline captions, choose **Subtitles → Whisper**, pick your audio source and spoken
   language, then click **Download model**. Base balances speed and accuracy; try Tiny on slower
   hardware. For translation, choose **Live translation**, then pick Gemini, OpenAI or DeepL with
   your API key and target language, or **Local Whisper** for offline translation into English.
3. Test the audio source, start the session, and use **Move overlay** to position the captions.
   Lock the overlay when you are ready to make it click-through again.
4. Click **Pause** during breaks. Cloud connections close after their final turn, and Whisper stops
   accepting new audio while finishing any backlog. **Hide overlay** only hides captions from view.
5. Click **Stop captions**, let Whisper finish any pending audio, then choose **Save as…**. Turn on
   automatic saving under **Settings → Transcript history** if you want sessions kept locally.

A fresh installation opens on **Whisper** (Base, automatic language detection); existing setups are
kept on upgrade. A scripted English/French demo lets you test the overlay with zero configuration.

## Privacy and local processing

With Whisper, audio never leaves your computer. Pending speech sits in a temporary file that is
deleted automatically when closed, never as a saved recording. Models download from Hugging Face
only when you ask for them and are verified by SHA-256 before use. Cloud engines stream audio
directly to the provider you choose, and API keys stay in Windows Credential Manager or the macOS
Keychain. The developer receives no audio, keys, transcripts or telemetry.
See the [privacy policy](docs/privacy.md) for full details on temporary storage, history and
recovery.

## New versions

Microsoft Store and Mac App Store copies update automatically. Copies installed from GitHub (the
Windows installer or MSI, or the Mac disk image) do not self-update, but can notify you when a new
release is out. From the second launch, the start screen asks once: *Check for new versions? This
asks GitHub once a day.* If you click **Turn on**, the app checks GitHub's latest release at most
once a day, a few seconds after launch and never during a session. When an update is available, it
shows *Version X.Y.Z is available* with **Release notes** (opening GitHub in your browser) and
**Not now** (dismissed until the next version). You download and install updates yourself. The
check stays off until you enable it and can be toggled under **Settings › App › Updates**. The
request sends only the app's name and version; see the
[privacy policy](docs/privacy.md#new-version-check-copies-installed-from-github). Store copies
never check.

## Support development

Live Translation & Subtitles is free, and every captioning, translation and export feature stays
free. If the app helps you, you can support its development on
[Buy Me a Coffee](https://buymeacoffee.com/fmadore). Contributions are optional and unlock no extra
features; cloud providers bill their own usage separately. The same link appears under Settings ›
App, except in the Mac App Store build.

## Build and contribute

On Windows, install Node 22.13+ or 24, rustup (`rust-toolchain.toml` pins the Rust version, which
rustup installs on first use), Visual Studio C++ Build Tools and the Windows SDK. Building Whisper
also requires LLVM/Clang with libclang, CMake and Ninja; follow the
[native compiler setup](docs/local-whisper.md#building) before building the desktop app. On macOS
(Apple silicon), install Xcode Command Line Tools, rustup, Node and CMake (required by Whisper's
native build; already present on CI runners); see [macOS](docs/macos.md).

```powershell
npm ci
npm run tauri dev
```

`npm run dev` previews the interface in a browser without audio capture or native engines.
Before committing, run:

```powershell
npm run verify
npm run verify:rust
```

`npm run verify` runs `format:check`, `check`, `lint`, `knip`, `check:languages`, `test`, `build`
and `test:style` in sequence, stopping on the first failure. `npm run verify:rust` runs
`cargo fmt --all --check`, `cargo clippy --locked --all-targets --all-features -- -D warnings` and
`cargo test --locked --all-features` on `src-tauri/Cargo.toml`, matching CI.
`npm run verify:vulkan` lints and tests the Whisper Vulkan helper; it requires the Vulkan SDK and,
on Windows, a short `CARGO_TARGET_DIR` such as `C:\wv` (see
[local Whisper](docs/local-whisper.md#graphics-processor)).

Build installers with `npm run tauri build`, or Store packages with `npm run bundle:msix:arm64` /
`npm run bundle:msix:x64` in the matching compiler environment. See
[MSIX packaging](docs/packaging-msix.md), [architecture](docs/architecture.md),
[security](SECURITY.md) and the [roadmap](ROADMAP.md). `npm run test:style` checks the operator
window's styles and overflow in Edge, while `npm run test:e2e` builds a debug copy of the app and
drives it end to end (requiring the Rust toolchain); both test the output of the latest
`npm run build`. CI checks the frontend, Rust on Linux, Windows x64/ARM64 and macOS, both
Playwright suites, dependencies and workflows, and reports coverage (`npm run test:coverage`
locally); every push also builds a Mac test app. Tagged releases produce installers, a combined
Store bundle and a notarised Mac disk image; Partner Center and App Store submissions are triggered
manually.

Originally created for the workshop *Digital Humanities and Artificial Intelligence in African
Studies*, STIAS, Stellenbosch, 21–24 September 2026. Thanks to
[@valentinrabot](https://github.com/valentinrabot) for proposing local transcription in
[issue #98](https://github.com/fmadore/Live-translation/issues/98) and for his meeting feedback.
[Citation metadata](CITATION.cff) · [MIT licence](LICENSE).
