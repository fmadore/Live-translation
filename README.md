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
│        → resampled to 16 kHz mono 16-bit PCM (the format Gemini Live expects)
│
├── Gemini Live client (Rust)
│   └── WebSocket bidi stream → sends 100 ms PCM chunks,
│       receives input + output transcription in real time.
│       API key kept in the OS keychain, used only from Rust — never in the front-end.
│
└── Two windows (SvelteKit)
    ├── Operator window   — source selector, FR⇄EN direction, start/stop, level meter
    └── Caption overlay   — frameless, transparent, always-on-top, click-through captions
```

See [`docs/gemini-live-api.md`](docs/gemini-live-api.md) for the verified API surface and
[`docs/architecture.md`](docs/architecture.md) for the design and data flow.

## Status

Milestone scaffold. The full project structure, operator UI, caption overlay, audio
capture (mic + Windows loopback), resampling, the Gemini Live WebSocket client, secure
key storage, and the Tauri command/event wiring are in place. What remains is hardware
validation: a Windows build and a rehearsal against a real Zoom call + room mic.

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

On first launch, open the operator window and paste your Gemini API key — it is stored in
the OS keychain (Windows Credential Manager / macOS Keychain / Secret Service), never on disk
in plaintext and never committed. Alternatively set `GEMINI_API_KEY` in an uncommitted `.env`
(see [`.env.example`](.env.example)) for development.

### Building a Windows installer

```bash
npm run tauri build           # produces .msi and .exe (NSIS) in src-tauri/target/release/bundle
```

## Running the app at the event

1. Run the **operator window** on the laptop. Pick the audio source (mic / system / both)
   and the translation direction, then **Start**.
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
  src/secrets.rs         OS keychain storage for the API key
  src/commands.rs        Tauri commands exposed to the front-end
docs/                    API notes and architecture
```

## License

MIT — see [LICENSE](LICENSE).
