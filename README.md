# Live Translation (FR ⇄ EN)

Real-time speech translation desktop app for the workshop **"Digital Humanities and
AI in African Studies"** (STIAS, Stellenbosch, 21–24 Sept 2026). Companion tool to the
[conference website](https://github.com/fmadore/stias-dh-ai-workshop-2026).

It captures two kinds of audio — a **presenter at the laptop** (microphone) and **audio
coming out of the laptop** (system/loopback, e.g. a remote speaker on Zoom) — translates
speech live between **French and English** (any language is auto-detected) using your choice
of **Google Gemini** or **OpenAI** live-translation models, and shows the result as **live
captions** in a transparent, always-on-top overlay that can float over a PowerPoint
presentation.

> **Output is text captions only.** The Gemini engine is **Live Translate**
> (`gemini-3.5-live-translate-preview`) run in **text mode** — it returns the translation as text
> with no audio synthesized, so there are no audio-output costs. You can also switch the
> **provider** to **OpenAI** (`gpt-realtime-translate`). See
> [`docs/gemini-live-api.md`](docs/gemini-live-api.md) and
> [`docs/openai-realtime-api.md`](docs/openai-realtime-api.md) for details.

## Architecture

```
Tauri app (Rust core + SvelteKit front-end)
├── Audio capture (Rust)
│   ├── Microphone        — cpal (cross-platform)        → presenter in the room
│   └── System loopback   — WASAPI loopback (Windows)    → Zoom / remote speaker
│        → resampled to mono 16-bit PCM (16 kHz for Gemini, 24 kHz for OpenAI)
│
├── Translation client (Rust) — Gemini or OpenAI, chosen per session
│   └── WebSocket bidi stream → sends 100 ms PCM chunks, receives the translated
│       text + source transcription in real time. Text-only — no audio is synthesized.
│       The API key lives in the OS keychain, used only from Rust — never in the front-end.
│
└── Two windows (SvelteKit)
    ├── Operator window   — source + caption-language selector, start/stop, level meter
    └── Caption overlay   — frameless, transparent, always-on-top, click-through captions
```

See [`docs/gemini-live-api.md`](docs/gemini-live-api.md) for the verified API surface and
[`docs/architecture.md`](docs/architecture.md) for the design and data flow.

## Status

Working and released. Operator UI, caption overlay (rolling subtitle-style auto-clear),
mic + Windows-loopback capture, resampling, the Gemini and OpenAI Live WebSocket clients,
text-only translation, secure per-provider key storage, transcript saving, and the
multi-platform release installers are all in place and verified on Windows. Re-verify the
model ids and run a full rehearsal (real Zoom call + room mic) before the event — the Live
APIs are in preview.

## Prerequisites

- **Node.js** ≥ 20 and **npm**
- **Rust** (stable) — <https://rustup.rs>
- **Tauri prerequisites** for your OS — <https://tauri.app/start/prerequisites/>
  - Windows: Microsoft C++ Build Tools + WebView2 (bundled on Win 11)
- An API key for whichever provider you use:
  - **Gemini** with access to `gemini-3.5-live-translate-preview` — <https://aistudio.google.com/apikey>
  - **OpenAI** with access to `gpt-realtime-translate` — <https://platform.openai.com/api-keys>

## Getting started

```bash
npm install

# Front-end only (browser, no audio/Tauri APIs):
npm run dev

# Full desktop app (recommended):
npm run tauri dev
```

On first launch, open the operator window and paste your provider's API key — it is stored in
the OS keychain (Windows Credential Manager / macOS Keychain / Secret Service), never on disk
in plaintext and never committed. Alternatively set `GEMINI_API_KEY` in an uncommitted `.env`
(see [`.env.example`](.env.example)) for development.

### Building a Windows installer

```bash
npm run tauri build           # produces .msi and .exe (NSIS) in src-tauri/target/release/bundle
```

## Running the app at the event

1. Run the **operator window** on the laptop. Pick the audio source (mic / system / both)
   and the caption language, then **Start**.
2. Drag the **caption overlay** onto the projector output and let it float over the
   PowerPoint (PowerPoint windowed or in Presenter view). The overlay is click-through, so
   it never steals clicks from your slides.
3. **Rehearse the full chain** (Zoom audio + room mic → captions on the projector) before
   the session — system-audio routing is the most failure-prone piece.

## Project layout

```
src/                     SvelteKit front-end
  routes/+page.svelte    Operator / control window
  routes/overlay/        Caption overlay window
  lib/                   Shared stores, types, Tauri bridge
src-tauri/               Rust core
  src/audio/             cpal microphone + WASAPI loopback capture, resampling
  src/gemini/            Gemini Live WebSocket client
  src/openai/            OpenAI Realtime translation client
  src/secrets.rs         OS keychain storage for the per-provider API keys
  src/commands.rs        Tauri commands exposed to the front-end
docs/                    API notes and architecture
```

## License

MIT — see [LICENSE](LICENSE).
