# Live Translation & Subtitles

Live captions over your meetings, video calls and slides. Transcribe speech locally with
**Whisper**, or translate it — into English locally with Whisper, or into many languages with
**Google Gemini**, **OpenAI** or **DeepL** — in a transparent overlay on Windows or macOS.

<a href="https://apps.microsoft.com/detail/9PFB8LR3RR9X?referrer=appbadge&mode=direct"><img src="https://get.microsoft.com/images/en-us%20dark.svg" alt="Get it from Microsoft" height="48"></a>&nbsp;&nbsp;<a href="https://buymeacoffee.com/fmadore"><img src="https://cdn.buymeacoffee.com/buttons/v2/default-yellow.png" alt="Buy me a coffee" height="48"></a>

Free, with every feature included. If it is useful to you, you can
[support its development](#support-development).

[GitHub releases](https://github.com/fmadore/Live-translation/releases) ·
[Privacy](docs/privacy.md) · [Documentation](docs/README.md)

**Windows 11, native x64 and ARM64 · macOS 15+ on Apple silicon (in testing) · English, French
and German interface · MIT license**

The macOS version is new: it has had its first run on a real Mac (an M2 Mac mini on macOS 26),
and the [macOS notes](docs/macos.md) give the results and the open points. Each GitHub release
attaches a notarised disk image. A free Mac App Store version, 1.13.1, is still in Apple's review
([submission notes](docs/mac-app-store.md)).

**[1.15.0](https://github.com/fmadore/Live-translation/releases/tag/v1.15.0)** was tagged and
published on GitHub on 17 November 2026; its Store submission is still to be made. For the
person running the captions, it adds a spending guard for cloud engines, markers and Find in the
transcript, a notice on the overlay for breaks, opt-in system-wide Pause and overlay keys, and
deleting or saving several saved sessions at once in History. Copies installed from GitHub get a
new-version notice, and the release has the fixes of the
[16 November review](docs/app-review-2026-11-16.md). See its
[release notes](docs/release-1.15.0.md) and
[Store handoff](docs/store-updates.md#release-1150-handoff).
**[1.14.0](https://github.com/fmadore/Live-translation/releases/tag/v1.14.0)** (13 November 2026)
is the version in the [Microsoft Store](https://apps.microsoft.com/detail/9PFB8LR3RR9X): DeepL as
an experimental fourth cloud translation engine (paid DeepL API plan only), Mistral subtitles
that reconnect after a transcription timeout, and an overlay that shows a line in every row when
both sources share a shallow strip ([release notes](docs/release-1.14.0.md)).

## What it does

- **Offline subtitles.** Whisper runs on your computer with 99 spoken-language choices or
  automatic detection: on the graphics processor where there is a usable one (Metal on a Mac,
  Vulkan on Windows), otherwise on the processor. Download Tiny (31 MiB), Base (57 MiB), Small
  (182 MiB) or Large v3 Turbo (547 MiB) once; then caption without an account, API key or
  internet connection. Speech detection skips silence and music before Whisper sees them. Speed
  and accuracy depend on the computer, model and language; on x64, Whisper needs a processor
  with AVX2.
- **Offline translation into English.** Whisper also translates any of those languages into
  English, with the Tiny, Base and Small models and no key (Large v3 Turbo cannot translate).
  English is its only target; choose Small, since
  Tiny and Base translate poorly.
- **Cloud translation and subtitles.** Gemini offers 78 translation targets in the app's
  catalog, DeepL 42 and OpenAI 13. DeepL needs a paid DeepL API plan, since its Voice API is not
  on the free one, and is experimental: it has not yet run in a live session. Gemini and Mistral
  also offer same-language subtitles. With them and with local Whisper, a list of names and
  terms helps spell speakers' names, institutions and jargon. Cloud engines need your own
  compatible account, API key and internet connection; provider usage charges apply.
- **Your choice of audio.** Capture a microphone, the computer's system audio, one application
  (its process tree on Windows, the app and its helpers on macOS), or microphone and system
  audio together. With both, you can leave out the room mic's copy of a call played through the
  room's loudspeakers, so the call is captioned once. Test levels before starting.
- **Captions where people look.** Place a transparent, always-on-top, click-through overlay
  over slides or calls; it reopens where you last placed it on each display setup. Adjust
  fonts, colour, contrast, size and reading pace; choose Fit window, Compact or Stable reading.
  Right-to-left captions are supported. Gamers can follow voice chat in another language: the
  overlay is a separate window, not something injected into the game, so run the game in
  windowed or borderless mode.
- **Bilingual audiences.** Show two translation languages at once, or include the original
  speech beneath translations in Fit window or Compact when the provider supplies it. A second
  translation language opens another cloud session per source and increases provider usage.
- **Meeting controls.** Pause and resume without splitting the transcript, reuse named meeting
  profiles, and search or pin caption languages. Customise the overlay's filler-word filter
  while keeping the raw transcript intact.
- **A transcript you can keep.** Read the whole transcript in its own tab during a session, and
  copy the last 5, 10 or 30 minutes as plain text. Export text, Markdown, SRT or WebVTT.
  Optional local history adds session titles, search and reopening; optional recovery protects
  unsaved captions. Both are off by default.
- **Captions in Zoom** (from 1.12.0). Share one caption language with a Zoom
  meeting's own caption bar, using the caption API token the host copies in Zoom; attendees
  turn captions on in Zoom. Only caption text is sent, and only once you start sharing
  ([guide](docs/zoom-captions.md)). Zoom is a trademark of Zoom Communications, Inc.; this
  project is not affiliated with Zoom.

## Start captioning

1. Install from the [Microsoft Store](https://apps.microsoft.com/detail/9PFB8LR3RR9X), or use
   an installer from [GitHub releases](https://github.com/fmadore/Live-translation/releases).
   Check the release version: Whisper requires 1.6.0, and its translation into English 1.7.0.
   GitHub installers are unsigned; Store packages are signed and update through the Store.
   A copy installed from GitHub can tell you when a new version is out (see
   [new versions](#new-versions)).
2. For local captions, choose **Subtitles → Whisper**, select your audio source and spoken
   language, then **Download model**. Base balances speed and quality; try Tiny on slower PCs.
   For translation, choose **Live translation**, then Gemini, OpenAI or DeepL with your key and
   a target language, or **Local Whisper** to translate into English without a key.
3. Test the source, start the session, and use **Move overlay** to position captions. Lock
   the overlay to make it click-through again.
4. **Pause** during breaks. Cloud connections close after their final turn; Whisper stops
   accepting new audio while processing any pending speech. **Hide overlay** only hides captions.
5. **Stop captions**, let Whisper finish any pending audio, then **Save as…**. Enable history in
   **Settings → Transcript history** if you want sessions saved automatically on this PC.

A fresh installation opens on **Whisper** with Base and automatic language detection;
existing saved setups are preserved. A scripted English/French demo shows the overlay without
any setup.

## Privacy and local processing

Whisper audio stays on the computer. Pending audio uses an automatically deleted temporary file;
it is not a saved recording. Models download from Hugging Face only when requested and are
verified before use. Cloud audio goes directly to the chosen provider. API keys stay in
Windows Credential Manager or the macOS Keychain. The developer receives no audio, keys,
transcripts or telemetry.
See the [privacy policy](docs/privacy.md) for temporary storage, history and recovery details.

## New versions

The Microsoft Store and the Mac App Store update their copies themselves. A copy installed from
GitHub (the Windows installer or MSI, or the Mac disk image) does not update itself, but it can
tell you when a new version is out. From the second launch, the start screen asks once:
*Check for new versions? This asks GitHub once a day.* With **Turn on**, the app asks GitHub
for the latest release at most once a day, a few seconds after launch and never during a
session, and shows *Version X.Y.Z is available* with **Release notes** (the release page, in
your browser) and **Not now** (hidden until the next version). You download and install it
yourself, as you did the first time. The check is off until you turn it on, and the switch is under
**Settings › App › Updates**. The request carries only the app's name and version; see the
[privacy policy](docs/privacy.md#new-version-check-copies-installed-from-github). Store copies
never check.

## Support development

Live Translation & Subtitles is free, and every captioning, translation and export feature stays
free. If the app is useful to you, you can help fund its development on
[Buy Me a Coffee](https://buymeacoffee.com/fmadore). Supporting is entirely optional and does not
unlock additional features; cloud providers bill their own usage separately. The same link is
in the app under Settings › App, except in the Mac App Store version.

## Build and contribute

Use Windows with Node 22.13+ or 24, rustup (`rust-toolchain.toml` pins the Rust version, which
rustup installs on first use), Visual Studio C++ Build Tools and the Windows SDK. Whisper also
needs LLVM/Clang with libclang, CMake and Ninja. Follow the
[native compiler setup](docs/local-whisper.md#building) before building the desktop app. On a
Mac (Apple silicon), install Xcode's command-line tools, rustup, Node and CMake (Whisper's native
build needs it; CI's runners have it already); see [macOS](docs/macos.md).

```powershell
npm ci
npm run tauri dev
```

`npm run dev` previews the interface in a browser; it cannot capture audio or run native engines.
Before committing, run:

```powershell
npm run verify
npm run verify:rust
```

`npm run verify` runs `format:check`, `check`, `lint`, `knip`, `check:languages`, `test`,
`build` and `test:style` in turn, stopping at the first failure. `npm run verify:rust` runs
`cargo fmt --all --check`, `cargo clippy --locked --all-targets --all-features -- -D warnings`
and `cargo test --locked --all-features` on `src-tauri/Cargo.toml`, with CI's flags.
`npm run verify:vulkan` lints and tests the Whisper Vulkan helper as CI does; it needs the
Vulkan SDK, and on Windows a short `CARGO_TARGET_DIR` such as `C:\wv` (see
[local Whisper](docs/local-whisper.md#graphics-processor)).

Build installers with `npm run tauri build`; build Store packages with
`npm run bundle:msix:arm64` or `npm run bundle:msix:x64` in the matching compiler environment.
See [MSIX packaging](docs/packaging-msix.md), [architecture](docs/architecture.md),
[security](SECURITY.md) and the [roadmap](ROADMAP.md). `npm run test:style` checks the
operator window's styles and overflow in Edge; `npm run test:e2e` builds a separate debug copy
of the app and drives it end to end, and needs the Rust toolchain. Both test the output of the
last `npm run build`. CI checks the frontend, Rust on Linux, Windows x64/ARM64 and macOS, both
Playwright suites, dependencies and workflows, and reports test coverage
(`npm run test:coverage` locally); every push builds a Mac test app. Tagged releases build
installers, a combined Store bundle and a notarised Mac disk image; Partner Center and App Store
submissions are started by hand.

Originally developed for the workshop *Digital Humanities and Artificial Intelligence in
African Studies*, STIAS, Stellenbosch, 21–24 September 2026. Thanks to
[@valentinrabot](https://github.com/valentinrabot) for proposing local transcription in
[issue #98](https://github.com/fmadore/Live-translation/issues/98) and for his meeting feedback.
[Citation metadata](CITATION.cff) · [MIT license](LICENSE).
