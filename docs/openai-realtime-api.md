# OpenAI Realtime Translate API

Verified 1 August 2026 with the official
[Realtime translation guide](https://developers.openai.com/api/docs/guides/realtime-translation)
and [translation client-event reference](https://developers.openai.com/api/reference/resources/realtime/translation-client-events).

## Connection and audio

```text
wss://api.openai.com/v1/realtime/translations?model=gpt-realtime-translate
Authorization: Bearer <OPENAI_API_KEY>
```

WebSocket input is base64 mono PCM16 LE at 24 kHz. The engine consumes 200 ms frames; the
capture path’s 100 ms chunks are valid and the service buffers two at a time. Silence remains
part of the continuous stream.

## Setup

```json
{
  "type": "session.update",
  "session": {
    "audio": {
      "input": {
        "transcription": { "model": "gpt-realtime-whisper" },
        "noise_reduction": { "type": "near_field" }
      },
      "output": { "language": "fr" }
    }
  }
}
```

The current schema explicitly supports output language, optional source transcription, and
input noise reduction. Source language is auto-detected.

Audio frames use `session.input_audio_buffer.append`. The app reads
`session.input_transcript.delta` for the operator’s source monitor and
`session.output_transcript.delta` for translated captions; output audio is ignored.

Only output transcript activity arms the 900 ms idle caption boundary. This prevents a
source delta from prematurely finalizing a caption before its translation arrives.

## Graceful stop

The client sends `{"type":"session.close"}`, stops appending audio, and continues reading
until `session.closed` or a four-second safety timeout. OpenAI documents that immediately
closing the socket can lose translated output still draining from the session.
