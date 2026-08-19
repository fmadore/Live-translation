# Architecture

## Data flow

```text
microphone (cpal) ─┐
                   ├─ downmix → low-pass/resample → PCM16 → bounded channel ─┐
system (WASAPI) ───┘                                                         │
                                                                             ▼
                     one realtime WebSocket per active source and provider
                     ├─ Gemini/OpenAI: translated transcript
                     └─ Mistral: same-language transcript
                                      │
                                      ▼
                          Tauri caption/status/level events
                                ┌─────┴─────┐
                         operator UI    overlay window
```

1. **Capture.** Each source owns a native thread. `CaptureState` downmixes device samples,
   applies a four-stage anti-alias low-pass when downsampling, resamples to 16 kHz
   (Gemini/Mistral) or 24 kHz (OpenAI), converts to mono PCM16 LE, and forms roughly 100 ms
   chunks. CPAL integer and floating-point sample formats are normalized through reusable
   scratch buffers.
2. **Backpressure.** Audio crosses a bounded five-chunk channel (at most about 500 ms), and
   meter updates cross a bounded eight-event channel. The realtime callback never blocks.
   After a network stall the consumer coalesces queued chunks to the newest one instead of
   replaying stale speech; reconnect-buffered audio is discarded outright.
3. **Provider session.** `session.rs` validates the selected mode/provider pair, resolves its
   key, and spawns one `realtime::run_session` task per source. `realtime.rs` owns connection
   timeouts, setup/audio pumping, message controls, idle caption segmentation, stable-connection
   backoff reset, retryable-vs-permanent HTTP classification, and turn finalization.
4. **Render/export.** Caption events are broadcast to both windows. The operator store tracks
   pending turns independently by `(origin, turnId)`; finalized lines can be exported in
   chronological plain text or Markdown. Mistral places its source-language transcription in
   the normal caption `text` field, so overlay and export need no provider-specific branch.

## Provider contracts

| Provider | Mode | Input | Caption event | Graceful stop |
|---|---|---|---|---|
| Gemini 3.5 Live Translate | Translate | 16 kHz PCM16 | `serverContent.outputTranscription` | WebSocket close |
| OpenAI Realtime Translate | Translate | 24 kHz PCM16, 200 ms engine frames | `session.output_transcript.delta` | `session.close`, drain to `session.closed` |
| Mistral Voxtral Mini Realtime | Transcribe | 16 kHz PCM16 | `transcription.text.delta` | `input_audio.flush`, then `input_audio.end`, drain to `transcription.done` |

The Mistral provider is intentionally unavailable in translation mode: Voxtral Mini
Transcribe Realtime is a speech-to-text model, not a live translation model.

## Concurrency and shutdown

`SessionManager` serializes start/stop operations with a lifecycle mutex, preventing an older
stop request from cancelling a newly started session. A parent `CancellationToken` owns the
whole run and each source gets a child token. A capture failure cancels only that source, so
the other half of “Both” can continue.

Stop first cancels capture, then each provider flushes its pending input and briefly drains
tail transcript events before the socket closes. Capture threads are joined, pending captions
are finalized, status/meters return to idle, and the monitor drops its stale current caption.

## Security

- Each provider key has a separate OS-keychain entry, with `.env` fallback for development.
- Keys never enter the Svelte renderer. Gemini currently authenticates in the WebSocket URL;
  OpenAI and Mistral use `Authorization: Bearer` headers from Rust.
- The webview CSP blocks outbound connections.
- CI uses least-privilege read permissions except the tag-driven release workflow, which needs
  `contents: write` to upload installers.

## Remaining operational limits

- Windows is the only supported target. The Linux CI lane exists to catch regressions in the
  non-`cfg(windows)` code; it produces no release artifact and has no system-loopback backend.
- Provider integrations are contract-tested locally but not called from CI: live tests need
  billable secrets and representative bilingual audio. Rehearse all three providers manually.
- The low-latency resampler is optimized for speech, not archival audio production.
- Gemini/OpenAI still generate translated audio server-side even though the app discards it;
  account for provider audio-output pricing.
