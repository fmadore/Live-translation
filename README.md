# Live Translation & Subtitles

Real-time captions for hybrid rooms and events. The desktop app captures a presenter’s
microphone, Windows system audio (whatever is playing — Zoom, Teams, a browser tab, a media
player), or both and renders captions in a transparent, always-on-top overlay. Built for —
and first deployed at — the **Digital Humanities and Artificial Intelligence in African
Studies** / **Humanités numériques et intelligence artificielle en études africaines**
workshop (STIAS, Stellenbosch, 21–24 September 2026).

It has two deliberately separate modes:

- **Live translation** — speech is auto-detected and translated into English or French by
  Google Gemini (`gemini-3.5-live-translate-preview`) or OpenAI
  (`gpt-realtime-translate`). Their generated audio is discarded; only transcript text is
  displayed. Gemini also captions speech that is already in the selected target language,
  so mixed-language meetings do not go blank during same-language passages.
- **Subtitles** — a built-in English/French product demonstration requires no setup, while
  real-time same-language speech recognition uses Mistral Voxtral Mini Transcribe Realtime
  (`voxtral-mini-transcribe-realtime-2602`). The transcript can be saved as plain `.txt` or
  Markdown.

The Store build opens on a deterministic **Built-in demo**: no publisher key, account,
microphone, language pack, network, or per-minute charge. It drives the real caption UI,
overlay, elapsed timer, level meter, transcript, and export path using clearly labeled bundled
scripted content; it does not recognize live speech. Live microphone/system subtitles use
Mistral, while live translation uses Gemini or OpenAI with the user's own provider key.
See
[`docs/microsoft-store.md`](docs/microsoft-store.md) for why it also matters for Microsoft
Store distribution. The Store name **Live Translation & Subtitles** is reserved (Store ID
`9PFB8LR3RR9X`); once the first submission passes certification the listing will be at
<https://apps.microsoft.com/detail/9PFB8LR3RR9X>. How the app handles audio, provider keys and
transcripts is set out in [`docs/privacy.md`](docs/privacy.md), published at
<https://fmadore.github.io/Live-translation/privacy>.

Provider details and verified wire formats are documented in
[`docs/gemini-live-api.md`](docs/gemini-live-api.md),
[`docs/openai-realtime-api.md`](docs/openai-realtime-api.md), and
[`docs/mistral-realtime-api.md`](docs/mistral-realtime-api.md).

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

## Prerequisites

**Windows only.** System-audio capture is WASAPI loopback and the app is not built or
released for any other platform. The Linux lane in CI is a compile check for the
non-`cfg(windows)` code, not a supported target.

- Windows 11 (native x64 and ARM64 Store packages)
- Node.js **24 LTS** and npm (Node.js **22.12+** remains CI-tested)
- Stable Rust
- [Tauri prerequisites for Windows](https://tauri.app/start/prerequisites/)
- No key is needed for the built-in demonstration. Live modes need the corresponding provider key:
  - [Google AI Studio](https://aistudio.google.com/apikey) for Gemini translation
  - [OpenAI](https://platform.openai.com/api-keys) for OpenAI translation
  - [Mistral Studio](https://console.mistral.ai/api-keys) for Mistral subtitles

## Running costs

Every provider bills per minute of streamed audio, so the meter runs for as long as a session
is open. Rates verified 10 August 2026 against
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
| Built-in caption demonstration | bundled scripted content | free | **$0.00** |

Gemini's input leg is billed on the full wall clock because silence stays in the stream, but
its expensive output leg accrues only while the model generates translated speech — pauses,
slide changes and Q&A gaps lower the bill, hence the range. That output audio is charged even
though the app discards it. OpenAI is duration-billed and therefore flat: silence costs the
same as speech, and the `gpt-realtime-whisper` source transcription that feeds the operator
monitor is a separate charge on top.

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
and set `GEMINI_API_KEY`, `OPENAI_API_KEY`, or `MISTRAL_API_KEY`.

Build Windows installers with `npm run tauri build`.

## Event-day workflow

1. Choose **Live translation** or **Live subtitles**, then select the audio source. For
   translation, select the target language and Gemini/OpenAI provider.
2. Start the session and confirm the source meter and live monitor move.
3. Use **Move overlay** to position/resize captions on the projector, then lock it back into
   click-through mode. In move mode the overlay itself has the keyboard: **Enter** locks it in
   place, **Esc** cancels and restores where it was, arrow keys nudge by a pixel (**Shift** for
   10), and **+**/**−** resize the text. **Hide overlay** blanks the captions mid-session (a
   video clip, a coffee break) without stopping anything; **F2** flips the caption language.
4. Use **Save text** or **Save Markdown** after captions finalize. Files are written under
   `Documents/Live-translation/` (with Downloads/temp fallbacks).
5. Rehearse the real Zoom + room-microphone + projector chain before the event. The realtime
   provider surfaces should be re-verified shortly beforehand.

## Project layout

```text
src/                         SvelteKit operator and overlay windows
  lib/ApiKeyPanel.svelte     provider key management
  lib/TranscriptMonitor.svelte monitor and text/Markdown export
  lib/transcript.ts          pure export formatting (unit tested)
src-tauri/src/audio/         capture, metering, resampling
src-tauri/src/realtime.rs    shared WebSocket lifecycle
src-tauri/src/{gemini,openai,mistral}/ provider protocols
src-tauri/src/ondevice/      deterministic built-in caption demonstration
.github/workflows/           frontend/Rust/security/workflow CI + releases
```

## CI and maintenance

Pull requests and `main` pushes run frontend tests/type-check/build/audit, Rust format,
Clippy and tests on Linux and Windows, RustSec, and actionlint. Dependabot checks npm, Cargo,
and GitHub Actions weekly. Installer releases remain tag-driven.

## License

MIT — see [LICENSE](LICENSE).
