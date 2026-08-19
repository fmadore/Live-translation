# Architecture

## Data flow

```text
microphone (cpal) ─┐
                   ├─ downmix → low-pass/resample → PCM16 → bounded channel ─┐
system (WASAPI) ───┘                                                         │
                                                                             ▼
                     one client per active source and provider
                     ├─ Gemini/OpenAI: translated transcript (WebSocket)
                     ├─ Mistral: same-language transcript (WebSocket)
                     └─ On-device: same-language transcript (local, keyless)
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
   key if the backend needs one, and spawns one client task per source. For the cloud
   providers that is `realtime::run_session`, which owns connection timeouts, setup/audio
   pumping, message controls, idle caption segmentation, stable-connection backoff reset,
   retryable-vs-permanent HTTP classification, and turn finalization.
   `Provider::OnDevice` instead spawns `ondevice::run_session`: no socket, no key, no
   reconnect loop. It consumes the same bounded audio channel and emits the same caption and
   status events, so everything downstream is unchanged. Recognition is blocking and
   CPU-bound, so it runs on a dedicated native thread behind a bounded queue; when the
   recognizer falls behind, the newest chunk is dropped rather than the queue reordered,
   which keeps the audio the recognizer *does* see contiguous.
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
| On-device recognizer | Transcribe | 16 kHz PCM16, pushed | `RecognitionEvent::Partial`/`Final` | drop the sample sender, drain the flush |

Mistral and the on-device engine are intentionally unavailable in translation mode: Voxtral
Mini Transcribe Realtime is a speech-to-text model, and Windows exposes no on-device
translation API. `session.rs` enforces this through `Provider::can_translate` rather than a
provider list, so a new translating backend cannot silently become a subtitle engine.

### On-device recognizer

`ondevice/engine.rs` is the single pluggable point; `ondevice/whisper.rs` is the current
implementation, whisper.cpp via `whisper-rs`. The binding constraint is that a recognizer must
accept **pushed PCM**: the app captures system audio over WASAPI loopback and offers device
selection, so an engine that opens its own microphone can serve neither system audio nor
*Both* mode. That constraint ruled out the inbox `Windows.Media.SpeechRecognition` namespace,
whose API surface has no audio input at all — verified against the generated `windows` 0.62
bindings. The Speech Recognition Windows AI API is the migration target once it leaves the
Windows App SDK experimental channel; `engine.rs` records both.

Whisper transcribes a buffer, not a stream, so realtime captions come from a sliding window:
audio accumulates into the current utterance, inference runs every 2 s of new audio for an
interim caption, and the utterance is committed after 800 ms of silence or at a 15 s cap. An
energy gate keeps the model away from pure silence, where whisper reliably invents text, and
wholly bracketed outputs (`[BLANK_AUDIO]`, `(soft music)`) are filtered before they can reach
the overlay. Model weights are shared through a process-wide cache, so *Both* mode loads one
copy and gives each origin its own decode state. The timing constants are untuned against
real conference audio on the event hardware — that needs a rehearsal.

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
