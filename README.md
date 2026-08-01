# Live Translation & Subtitles

Real-time captions for the **Digital Humanities and AI in African Studies** workshop
(STIAS, Stellenbosch, 21–24 September 2026). The desktop app captures a presenter’s
microphone, Windows system/Zoom audio, or both and renders captions in a transparent,
always-on-top overlay.

It has two deliberately separate modes:

- **Live translation** — speech is auto-detected and translated into English or French by
  Google Gemini (`gemini-3.5-live-translate-preview`) or OpenAI
  (`gpt-realtime-translate`). Their generated audio is discarded; only transcript text is
  displayed.
- **Live subtitles** — Mistral Voxtral Mini Transcribe Realtime
  (`voxtral-mini-transcribe-realtime-2602`) produces same-language text without translating
  it. The transcript can be saved as plain `.txt` or Markdown.

Provider details and verified wire formats are documented in
[`docs/gemini-live-api.md`](docs/gemini-live-api.md),
[`docs/openai-realtime-api.md`](docs/openai-realtime-api.md), and
[`docs/mistral-realtime-api.md`](docs/mistral-realtime-api.md).

## Architecture

```text
Tauri app (Rust core + SvelteKit front-end)
├── Audio capture
│   ├── Microphone — cpal
│   └── System/Zoom — WASAPI loopback on Windows
│       └── mono PCM16: 16 kHz Gemini/Mistral or 24 kHz OpenAI
├── Bounded realtime pipeline — one capture + WebSocket session per source
│   ├── Gemini/OpenAI → translated transcript captions
│   └── Mistral → same-language subtitle captions
└── Windows
    ├── Operator — mode/source/provider controls, meters, monitor, export
    └── Overlay — transparent, always-on-top, click-through captions
```

The shared runner provides connection timeouts, bounded queues, stale-audio discard,
exponential reconnect backoff, provider-error classification, turn isolation, and graceful
provider flushes. Keys remain in the OS keychain and are used only by Rust. See
[`docs/architecture.md`](docs/architecture.md) for the complete flow.

## Prerequisites

- Node.js **24 LTS** and npm (Node.js **22.12+** remains CI-tested)
- Stable Rust
- [Tauri prerequisites](https://tauri.app/start/prerequisites/) for the target OS
- At least one provider key:
  - [Google AI Studio](https://aistudio.google.com/apikey) for Gemini translation
  - [OpenAI](https://platform.openai.com/api-keys) for OpenAI translation
  - [Mistral Studio](https://console.mistral.ai/api-keys) for Mistral subtitles

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

The operator stores each provider key separately in Windows Credential Manager, macOS
Keychain, or Secret Service. For development, copy `.env.example` to an uncommitted `.env`
and set `GEMINI_API_KEY`, `OPENAI_API_KEY`, or `MISTRAL_API_KEY`.

Build Windows installers with `npm run tauri build`.

## Event-day workflow

1. Choose **Live translation** or **Live subtitles**, then select the audio source. For
   translation, select the target language and Gemini/OpenAI provider.
2. Start the session and confirm the source meter and live monitor move.
3. Use **Move overlay** to position/resize captions on the projector, then lock it back into
   click-through mode.
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
.github/workflows/           frontend/Rust/security/workflow CI + releases
```

## CI and maintenance

Pull requests and `main` pushes run frontend tests/type-check/build/audit, Rust format,
Clippy and tests on Linux and Windows, RustSec, and actionlint. Dependabot checks npm, Cargo,
and GitHub Actions weekly. Installer releases remain tag-driven.

## License

MIT — see [LICENSE](LICENSE).
