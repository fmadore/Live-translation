# Architecture

## Overview

```
┌──────────────────────────── Tauri app (one process) ────────────────────────────┐
│                                                                                   │
│  Rust core                                                                        │
│  ┌─────────────────────┐   AudioChunk (16 kHz mono PCM-16, 100 ms)               │
│  │ audio::capture       │ ──┐  via tokio unbounded channel                       │
│  │  (cpal microphone)   │   │                                                     │
│  └─────────────────────┘   ├──▶ gemini::client ── WebSocket ──▶ Gemini 3.5 Live  │
│  ┌─────────────────────┐   │     (one task per source)            Translate      │
│  │ audio::loopback      │ ──┘                                                     │
│  │  (WASAPI, Windows)   │        ◀── inputTranscription / outputTranscription     │
│  └─────────────────────┘                                                          │
│           │ level meter (RMS/peak)        │ Caption events                        │
│           ▼                               ▼                                       │
│  ┌────────────────────────── Tauri events (broadcast) ──────────────────────┐    │
│  └───────────────┬───────────────────────────────────┬──────────────────────┘    │
│                  ▼                                     ▼                           │
│        Operator window (/)                   Caption overlay (/overlay)            │
│        controls + monitor + meters           transparent · always-on-top ·        │
│                                              click-through captions                │
└───────────────────────────────────────────────────────────────────────────────────┘
```

## Data flow

1. **Capture.** One thread per active source. `cpal` (mic) / WASAPI loopback (system) hands
   interleaved device-rate samples to `CaptureState`, which downmixes to mono, resamples to
   the provider's input rate (16 kHz Gemini / 24 kHz OpenAI; streaming linear resampler),
   converts to PCM-16 LE, and emits ~100 ms `AudioChunk`s.
   It also computes an RMS/peak level (~20 Hz) for the meter.
2. **Translate.** `session.rs` dispatches on the selected **provider** and spawns
   `realtime::run_session` per source — the shared runner that owns the reconnect/backoff
   loop (backoff resets after a stable connection; a 4xx handshake stops with an error
   instead of retrying), drops audio that went stale while disconnected, and pumps the
   select loop. Each provider implements the `RealtimeProtocol` trait with only its
   specifics. **Gemini** (`gemini::client`): setup frame for the translate model, caption
   text from `outputTranscription`, deltas accumulating until `turnComplete`. **OpenAI**
   (`openai::client`): connects to `/v1/realtime/translations` (auth via an `Authorization`
   header), sends a `session.update`, reads `output_transcript` deltas — that stream has no
   turn-complete event, so the runner finalizes captions after a short idle gap. Source
   text (operator monitor) comes from the input transcription in both. Model ids are
   overridable via env vars (`GEMINI_TRANSLATE_MODEL` / `OPENAI_TRANSLATE_MODEL`).
3. **Render.** Captions are emitted as Tauri events. Tauri broadcasts events to **all**
   windows, so the operator monitor and the overlay both receive them with no extra plumbing.

## Why one client per source

Running mic and system as independent sessions keeps them simple and lets the operator
translate both at once (e.g. a French speaker in the room while an English speaker is on
Zoom). Each session reconnects independently, so one dropping doesn't disturb the other.

## Concurrency & shutdown

A single `CancellationToken` per session is shared by every capture thread and client task.
`stop()` cancels it: capture threads exit their park loop and drop their channel senders,
which makes each client's `audio_rx.recv()` return `None`, closing the WebSocket cleanly.

## Security

- API keys (one per provider: Gemini, OpenAI) live in the **OS keychain** (`keyring`) or a
  dev `.env`; they are read only in Rust — Gemini's goes in the WebSocket URL, OpenAI's in an
  `Authorization` header. They never reach the renderer, and the CSP blocks outbound
  `connect-src` from the webview.
- **Hardening (future):** mint **ephemeral tokens** on the `v1alpha` endpoint and connect
  with those, so even the Rust process holds only a short-lived credential. See
  `docs/gemini-live-api.md`.

## Known gaps / next steps

- **WASAPI loopback** (`audio/loopback.rs`) is written to the `wasapi` crate's loopback
  pattern but **not yet compiled/run on Windows** — validate during the rehearsal.
- **OpenAI** is wired as a second provider (`gpt-realtime-translate`); see
  `docs/openai-realtime-api.md`. **DeepL Voice** remains a candidate for a further provider
  (native speech→**text** translation, excellent FR⇄EN;
  `wss://api.deepl.com/v3/voice/realtime/connect`) — it needs a separate key and its schema
  confirmed. The `Provider` enum in `types.rs` (with per-provider client/model/rate selection
  in `session.rs`) is the seam to extend.
- Ephemeral-token auth not yet implemented (Gemini); key is read only in Rust for now.
- App icons are placeholders generated by `tauri icon`; swap in real branding before the event.

### Recently added

- Operator-side **caption font-size control**: persisted in `localStorage` (shared by both
  windows) and pushed live to the overlay via the `overlay-config` event.
- **Transcript to disk**: `save_transcript` writes a timestamped Markdown file to
  `Documents/Live-translation/` (operator "Save transcript" button).
