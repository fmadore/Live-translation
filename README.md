# Live Translation & Subtitles

[![Get it from the Microsoft Store](https://get.microsoft.com/images/en-us%20dark.svg)](https://apps.microsoft.com/detail/9PFB8LR3RR9X)

Real-time captions for hybrid rooms and events. The desktop app captures a presenter’s
microphone, Windows system audio (whatever is playing — Zoom, Teams, a browser tab, a media
player), or both and renders captions in a transparent, always-on-top overlay. Built for the
[**Digital Humanities and Artificial Intelligence in African Studies** / **Humanités
numériques et intelligence artificielle en études
africaines**](https://fmadore.github.io/stias-dh-ai-workshop-2026/) workshop (STIAS,
Stellenbosch, 21–24 September 2026).

It has two deliberately separate modes:

- **Live translation** — speech is auto-detected and translated into the selected caption language by
  Google Gemini (`gemini-3.5-live-translate-preview`) or OpenAI
  (`gpt-realtime-translate`). Their generated audio is discarded; only transcript text is
  displayed. Gemini also captions speech that is already in the selected target language,
  so mixed-language meetings do not go blank during same-language passages.
- **Subtitles** — a built-in English/French product demonstration requires no setup, while
  real-time same-language speech recognition uses Mistral Voxtral Mini Transcribe Realtime
  (`voxtral-mini-transcribe-realtime-2602`) or Google Gemini
  (`gemini-3.5-transcribe-live`). Both detect the spoken language themselves; Gemini covers
  over 70 languages and cleans fillers and false starts out of the subtitle, Voxtral is flat-rate
  and has no session length limit. The transcript can be saved as plain `.txt` or Markdown; SRT/VTT export and the native Windows Save As dialog are available in version 1.2.3.

The app opens on a deterministic **Built-in demo**: no publisher key, account, microphone,
language pack, network, or per-minute charge. It drives the real caption UI, overlay, elapsed
timer, level meter, transcript and export path using clearly labelled bundled scripted
content; it does not recognize live speech. Live microphone and system subtitles use Mistral
or Gemini, and live translation uses Gemini or OpenAI with your own provider key.
[`docs/microsoft-store.md`](docs/microsoft-store.md) explains why that split is what made
Microsoft Store distribution possible.

How the app handles audio, provider keys and transcripts is set out in
[`docs/privacy.md`](docs/privacy.md), published at
<https://fmadore.github.io/Live-translation/privacy>.

Provider details and verified wire formats are documented in
[`docs/gemini-live-api.md`](docs/gemini-live-api.md),
[`docs/openai-realtime-api.md`](docs/openai-realtime-api.md), and
[`docs/mistral-realtime-api.md`](docs/mistral-realtime-api.md).

## Caption languages (1.5.0)

Coverage depends on the selected engine: Gemini Live Translate offers **78 target languages**;
OpenAI Realtime Translate offers **13**. This includes German, Japanese and Spanish. The
built-in demo remains English/French only, and live subtitle engines auto-detect speech.
The interface stays independently available in English, French and German.

Step 03 lets you search by name, native name or language code and pin favourites. English and
French are pinned initially. Unsupported favourites stay visible with an explanation after
an engine change; an unsupported selection blocks Start until you choose another language.
F2 swaps the first two favourites while stopped, only when both are supported. Rehearsal
recordings remain English/French. See [language coverage and verification](docs/language-coverage.md).
Live-provider and packaged-app acceptance are still pending; see the verification notes above.

## Release status

**Latest GitHub release: [v1.5.1](https://github.com/fmadore/Live-translation/releases/tag/v1.5.1).**
See [what changed in 1.5.1](docs/release-1.5.1.md) and the
[release handoff](docs/store-updates.md#release-151-handoff).

1.5.1 is a maintenance release from the [22 September app review](docs/app-review-2026-09-22.md).
It keeps more speech detail when 44.1/48 kHz audio is downsampled for a provider. Release
builds ignore `.env` and provider host overrides. A failing capture or provider thread ends
that source instead of closing the app. A quit prompt over Settings keeps keyboard access.
History saves at most every 5 seconds, and Stable reading's overlay context is bounded.
[What changed in 1.5.0](docs/release-1.5.0.md) covers the caption-language features.

Introduced in 1.3.0: Optional local transcript history saves finalized captions progressively
and lets you reopen, copy, export or delete earlier sessions. Stable reading anchors captions
at the top left and advances by whole lines. Optional filler cleanup affects only the overlay;
raw transcripts remain intact. The interface is available in English, French and German.

See [transcript history](docs/transcript-history.md), [caption layout](docs/caption-layout.md),
and the [current release handoff](docs/store-updates.md#release-151-handoff).
The GitHub release and prepared Store submission are separate: final packaged acceptance,
fresh screenshots and Partner Center submission remain tracked in that handoff.

Thanks to **@valentinrabot** for the detailed meeting feedback and suggestions in
[#79](https://github.com/fmadore/Live-translation/issues/79),
[#80](https://github.com/fmadore/Live-translation/issues/80), and
[#81](https://github.com/fmadore/Live-translation/issues/81), following the earlier
responsive-caption suggestion in [#77](https://github.com/fmadore/Live-translation/issues/77).

## Install

**[Get it from the Microsoft Store](https://apps.microsoft.com/detail/9PFB8LR3RR9X)** —
native x64 and ARM64, signed by Microsoft, and it updates itself. This is the recommended
route.

The [releases page](https://github.com/fmadore/Live-translation/releases) also carries an
unsigned x64 NSIS installer and MSI. Being unsigned, they meet a SmartScreen "Windows
protected your PC" warning on first launch — choose **More info → Run anyway**.

Either way you need Windows 11 and the Microsoft Edge WebView2 Runtime, which current
Windows 11 installs already have.

While the app is running it keeps an icon in the notification area, so a live session stays
reachable — open the window, show or hide the overlay, stop the session, or quit — with the
operator window out of the way. Closing the window quits the app unless you turn on **Keep
running in the tray when I close this window**.

## Version 1.4.2 session controls

Start and Stop now share a persistent bar below the header. Both remain accessible while
setup, captions and transcripts scroll below them, including narrow windows. Readiness
checks, capture behavior and keyboard shortcuts are unchanged.

## Version 1.4.1 interface update

The next Store submission target is **1.5.1**. Settings now has Captions, Reading,
Transcript history and App tabs. Profiles sit above setup, saved sessions use a list/detail
view, presets and previews stay visible, and the tray follows the interface language.
See the [current release handoff](docs/store-updates.md#release-151-handoff) for native testing and
packaging status. Version 1.4.1 is released on GitHub.

## Version 1.4.0 usability improvements

Version 1.4.0 is released on GitHub. Installer availability and remaining native/Store checks
are tracked in the [release handoff](docs/store-updates.md#release-151-handoff).

Caption appearance now offers a 2–30 second reading pause for finished Fit/Compact captions,
optional Steadier interim updates, and bright/dark previews with reading presets. Named meeting
profiles restore setup, appearance and overlay placement after checking audio devices. History
supports titles and text/date/language search. Per-source status distinguishes recent audio,
captions, silence and reconnection, with a notice when audio arrives without captions.

Operator-window shortcuts: **Ctrl+Shift+Space** starts/stops, **Ctrl+Shift+O** shows/hides the
overlay, and **Ctrl+Shift+Up/Down** changes caption size. They do not intercept text fields or
dialogs. **F2** continues to switch translation direction while stopped.
Gemini Smart transcription now treats final results as authoritative, including empty results
that retract speculative filler text. The local overlay filter remains optional and separate;
its word list can be edited in Settings → Reading ([#85](https://github.com/fmadore/Live-translation/issues/85)).
See [meeting profiles and live controls](docs/usability.md) for behavior and verification.

## Architecture

```text
Tauri app (Rust core + SvelteKit front-end)
├── Audio capture
│   ├── Microphone — cpal
│   └── System audio — WASAPI loopback on Windows (any app's output)
│       └── mono PCM16: 16 kHz Gemini/Mistral or 24 kHz OpenAI
├── Bounded realtime pipeline — one capture + WebSocket session per source
│   ├── Gemini/OpenAI → translated transcript captions
│   └── Mistral → same-language subtitle captions
├── Built-in deterministic demo → caption/level/status events without capture or network
└── Windows
    ├── Operator — mode/source/provider controls, meters, monitor, export
    └── Overlay — transparent, always-on-top, click-through captions
```

The shared runner provides connection timeouts, bounded queues, stale-audio discard,
exponential reconnect backoff, provider-error classification, turn isolation, and graceful
provider flushes. Keys remain in Windows Credential Manager and are used only by Rust. See
[`docs/architecture.md`](docs/architecture.md) for the complete flow.

## Prerequisites (building from source)

**Windows only.** System-audio capture is WASAPI loopback and the app is not built or
released for any other platform. The Linux lane in CI is a compile check for the
non-`cfg(windows)` code, not a supported target.

- Windows 11
- Node.js **24 LTS** and npm (Node.js **22.12+** remains CI-tested)
- Stable Rust
- [Tauri prerequisites for Windows](https://tauri.app/start/prerequisites/)
- No key is needed for the built-in demonstration. Live modes need the corresponding provider key:
  - [Google AI Studio](https://aistudio.google.com/api-keys) for Gemini translation
  - [OpenAI](https://platform.openai.com/api-keys) for OpenAI translation
  - [Mistral Studio](https://console.mistral.ai/api-keys) for Mistral subtitles
  - Gemini subtitles reuse the same AI Studio key as Gemini translation — save it once

## Running costs

Every provider bills per minute of streamed audio, so the meter runs for as long as a session
is open. Rates verified 27 August 2026 against
[Gemini](https://ai.google.dev/gemini-api/docs/pricing),
[OpenAI](https://developers.openai.com/api/docs/pricing) and
[Mistral](https://mistral.ai/pricing/api) pricing.

| Mode | Model | Rate | One hour |
| --- | --- | --- | --- |
| Translation | `gemini-3.5-live-translate-preview` | $0.0053/min audio in + $0.0315/min audio out | **$1.25–2.21** |
| Translation | `gpt-realtime-translate` | $0.034/min | $2.04 |
| ↳ source monitor | `gpt-realtime-whisper` | $0.017/min | $1.02 |
| Translation | OpenAI total | | **$3.06** |
| Subtitles | `voxtral-mini-transcribe-realtime-2602` | $0.006/min | **$0.36** |
| Subtitles | `gemini-3.5-transcribe-live` | $0.005/min audio in + $0.004/min text out | **$0.30–0.54** |
| Built-in caption demonstration | bundled scripted content | free | **$0.00** |

Both Gemini rows work the same way: the input leg is billed on the full wall clock because
silence stays in the stream, while the output leg accrues only while the model is producing
something — pauses, slide changes and Q&A gaps lower the bill, hence the ranges. For
translation that output is audio, charged even though the app discards it; for subtitles it is
the transcript text, which is far cheaper. OpenAI is duration-billed and therefore flat:
silence costs the same as speech, and the `gpt-realtime-whisper` source transcription that
feeds the operator monitor is a separate charge on top. Voxtral is likewise flat.

The two subtitle engines cost about the same, so choose on behaviour rather than price.
Gemini covers over 70 languages, detects the spoken one per utterance, and applies Google's
smart transcription — fillers, false starts and spoken self-corrections are cleaned out, and
punctuation and casing are applied. Against that, its live sessions cap at ten minutes, so a
long room session reconnects several times an hour with a one- to two-second caption gap each
time; and it decides its own segment boundaries, which for a speaker who rarely pauses can
mean a caption that grows to a paragraph before it settles. Voxtral has no session cap and
finalizes on short pauses, so it produces shorter, more even lines. Both behaviours are
measured rather than assumed — see [`docs/gemini-live-api.md`](docs/gemini-live-api.md).

Selecting **Both** as the source doubles every figure — the pipeline opens one capture and
one WebSocket session per origin.

Scaled to the workshop, the programme's captionable sessions (panels, keynotes, discussions
and plenaries — the Day 2 and Day 3 afternoons are excursions) total **18 hours**, or about
21 hours of wall clock if the session is left running through coffee breaks. Translation over
those hours costs roughly **$25–45 on Gemini** or **$55–65 on OpenAI** for a single source,
doubling to about $50–90 and $110–130 if the room microphone and the Teams feed are both
captioned throughout.

Both translation models are preview-tier; re-check the rates alongside the wire formats
before the event.

## Development

```bash
npm install
npm test
npm run check
npm run build

# Frontend preview only (no native capture)
npm run dev

# Full desktop app
npm run tauri dev
```

The operator stores each provider key separately in Windows Credential Manager. For
development, copy `.env.example` to an uncommitted `.env`
and set `GEMINI_API_KEY`, `OPENAI_API_KEY`, or `MISTRAL_API_KEY`. Only debug builds
(`npm run tauri dev`) load `.env`; release builds ignore it.

### Testing

Vitest 5 uses the existing Node 22.12+/24 and Vite 8 toolchain. DOM matchers are
registered in `vitest-setup-client.ts`; `src/vitest.d.ts` bridges jest-dom 7.0.1
types to Vitest 5’s matcher interface until upstream provides compatible declarations.

`npm test` runs two Vitest 5 projects. Pure logic runs in **node** — faster, and a test cannot
start depending on the DOM by accident. Anything that renders a component opts in to
**jsdom** by being named `*.svelte.test.ts`; those get Testing Library, jest-dom matchers and
automatic cleanup via `vitest-setup-client.ts`.

User-facing text belongs in the message catalogs, not in a component:
[`docs/localization.md`](docs/localization.md) has the contract — the interface language is
independent of the caption language, the Rust core names failures and the catalog words them,
and `npm run check` plus `npm test` fail on anything missing.

UI work has one more standard to meet: [`docs/accessibility.md`](docs/accessibility.md) has
the contrast, focus, heading, announcement and contrast-theme rules, which of them
`npm test` enforces on its own, and the manual Narrator / contrast-theme / text-scaling walk
that belongs to a release.

Before opening a PR that touches the UI, run the **browser-preview smoke test**: `npm run dev`,
load <http://localhost:5173>, and check the console is clean. The operator window degrades
deliberately without a Tauri runtime, and this catches the class of bug where a component
invokes a Tauri command before the app knows it is running in a browser. Note that this
preview cannot exercise audio capture, provider sessions or the overlay — anything touching
those needs `npm run tauri dev` on Windows.

Build Windows installers with `npm run tauri build`. The Store packages are built separately —
`npm run bundle:msix:x64` and `npm run bundle:msix:arm64` write an unsigned `.msix` each, which
CI then combines into the multi-architecture bundle described in
[`docs/packaging-msix.md`](docs/packaging-msix.md).

## Event-day workflow

1. Choose **Live translation** or **Subtitles**, then select the audio source. For
   translation, select the target language and Gemini/OpenAI provider — **F2** swaps the
   direction, but only before you start, because the provider is given the target once at
   session start. (**Subtitles** opens on the built-in demonstration; switch the provider to
   Mistral for live speech.)
2. Start the session and confirm the source meter and live monitor move.
3. Use **Move overlay** to position/resize captions on the projector, then lock it back into
   click-through mode. In move mode the overlay itself has the keyboard: **Enter** locks it in
   place, **Esc** cancels and restores where it was, arrow keys nudge by a pixel (**Shift** for
   10), and **+**/**−** resize the text. **Hide overlay** blanks the captions mid-session (a
   video clip, a coffee break) without stopping anything.
   For system audio or Both, **System capture** also offers **One application**. Select an
   open application window, then run the audio test. This captures its process tree, not an
   individual browser tab. Application capture requires Windows build 20348 or later;
   unsupported systems retain the existing output capture option. See
   [application capture](docs/application-capture.md) for scope and testing limits.
4. Choose Markdown, plain text, WebVTT, or SubRip in the transcript's format selector,
   then **Save as…**. The native Windows dialog lets you choose the folder and filename,
   confirms overwrites, and remembers the last successful destination folder.
   Timed exports require caption timing; older untimed recovery files can still be saved
   as text or Markdown. See [transcript export](docs/transcript-export.md).
5. Open **Settings → Transcript history** and optionally enable **Automatically save sessions locally**.
   Finalized raw lines are saved progressively; select a saved session in the list
   for copying, export, or deletion. History is off by default and remains until you delete it.
   See [transcript history](docs/transcript-history.md).
6. In **Settings → Reading**, choose **Stable reading** for top-left text that scrolls by whole
   lines. **Hide filler words** optionally cleans obvious hesitations in the overlay while
   preserving the raw transcript; add or remove words in the list under it. See
   [caption layout](docs/caption-layout.md#the-word-list).
7. Rehearse the real Zoom + room-microphone + projector chain before the event. The realtime
   provider surfaces should be re-verified shortly beforehand.

## Project layout

```text
src/                         SvelteKit operator and overlay windows
  lib/ApiKeyPanel.svelte     provider key management
  lib/TranscriptMonitor.svelte monitor and text/Markdown/SRT/VTT export
  lib/transcript.ts          pure export formatting (unit tested)
src-tauri/src/audio/         capture, metering, resampling
src-tauri/src/realtime.rs    shared WebSocket lifecycle
src-tauri/src/{gemini,openai,mistral}/ provider protocols
src-tauri/src/ondevice/      deterministic built-in caption demonstration
.github/workflows/           CI, tag-driven releases, Store submission
```

## CI and maintenance

Pull requests and `main` pushes run frontend tests/type-check/build/audit, Rust format,
Clippy and tests on Linux and Windows, RustSec, and actionlint. Dependabot checks npm, Cargo,
and GitHub Actions weekly.

Pushing a `v*` tag builds the installers, both architectures' MSIX, and the multi-architecture
`.msixbundle` the Store submission uses. That bundle is then uploaded to Partner Center by
hand: the submission API is a Company-account feature, and this is an Individual account —
see [`docs/store-updates.md`](docs/store-updates.md).

## License

MIT — see [LICENSE](LICENSE).
