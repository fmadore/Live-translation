# Mistral Voxtral Mini Realtime Transcription API

Cloud subtitle integration. For local subtitles without an account or key, see
[Whisper](local-whisper.md).

Verified 1 August 2026 against Mistral’s
[realtime transcription guide](https://docs.mistral.ai/studio-api/audio/speech_to_text/realtime_transcription),
[client-auth documentation](https://docs.mistral.ai/studio-api/audio/speech_to_text/realtime_transcription/client_auth),
and the official [`mistralai` realtime connection source](https://github.com/mistralai/client-python/blob/main/src/mistralai/extra/realtime/connection.py).

## Scope

`voxtral-mini-transcribe-realtime-2602` produces same-language realtime transcription. It
does **not** translate, so the UI exposes it under **Live subtitles**. The app sends 16 kHz
mono signed 16-bit little-endian PCM with a default target streaming delay of 480 ms. Override
the delay with `MISTRAL_TARGET_STREAMING_DELAY_MS` to test the latency/accuracy trade-off.

## Connection and setup

```text
wss://api.mistral.ai/v1/audio/transcriptions/realtime?model=voxtral-mini-transcribe-realtime-2602
Authorization: Bearer <MISTRAL_API_KEY>
```

```json
{
  "type": "session.update",
  "session": {
    "audio_format": { "encoding": "pcm_s16le", "sample_rate": 16000 },
    "target_streaming_delay_ms": 480
  }
}
```

Each audio chunk is base64 encoded:

```json
{ "type": "input_audio.append", "audio": "<base64 PCM16>" }
```

The server emits `session.created`, `session.updated`, and incremental
`transcription.text.delta` events whose `text` is displayed directly. Pauses finalize a
subtitle line through the shared 900 ms idle boundary.

## Graceful stop

Like the official SDK, the client sends `input_audio.flush`, then `input_audio.end`, and drains
through `transcription.done` (or a four-second safety timeout) before closing. Remaining
accumulated text is finalized and included in exports.

Long-lived keys are safe here because the Rust backend, not a web page, opens the WebSocket.
Mistral’s short-lived `rt_*` / `Sec-WebSocket-Protocol` flow is for browser clients that
cannot set an `Authorization` header.
