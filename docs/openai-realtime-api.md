# OpenAI Realtime Translate — API surface (verified June 2026)

Verified against OpenAI's developer docs and the official cookbook at implementation time.
**Re-verify before the event** — these models are new (May 2026) and the realtime API evolves.

Sources:

- Announcement: <https://openai.com/index/advancing-voice-intelligence-with-new-models-in-the-api/>
- Realtime translation guide: <https://developers.openai.com/api/docs/guides/realtime-translation>
- gpt-realtime-translate cookbook: <https://developers.openai.com/cookbook/examples/voice_solutions/realtime_translation_guide>
- Model page: <https://developers.openai.com/api/docs/models/gpt-realtime-translate>

## The model

`gpt-realtime-translate` — a dedicated, streaming **speech-to-speech** translation model
(70+ input languages → 13 output languages, ~$0.034/min). Like Gemini's Live Translate, its
native output is translated *audio*; we never play it and instead render the **output
transcript** as the caption. Source-language transcription (for the operator monitor) is a
sub-config that runs `gpt-realtime-whisper`.

> ⚠️ Re-verify the model id and the event/field names below before the event. The
> translations endpoint is a preview surface; the cookbook event names use a `session.` prefix
> and our parser matches on the `type` *suffix* to stay robust if that changes.

## Connection

```
wss://api.openai.com/v1/realtime/translations?model=gpt-realtime-translate
```

- **Auth (server-side):** an HTTP header on the WebSocket handshake — `Authorization: Bearer
  <OPENAI_API_KEY>`. Unlike Gemini, the key is **not** a query parameter. (Browser clients use
  an ephemeral `client_secret` instead; we connect from Rust, so we use the key directly.)
- No `OpenAI-Beta` header is needed for this GA-style endpoint.

## Audio format

| Direction | Format |
|-----------|--------|
| Input  | base64 **16-bit PCM, 24 kHz, mono, little-endian**, sent in ~100 ms chunks |
| Output | translated audio (24 kHz PCM16) — **discarded**; we read the transcript only |

Note the input rate is **24 kHz** (Gemini uses 16 kHz). `session.rs` picks the rate from
`Provider::input_sample_rate()` and the capture pipeline resamples to it.

## Setup message (first frame after connect)

```json
{
  "type": "session.update",
  "session": {
    "audio": {
      "input": {
        "transcription": { "model": "gpt-realtime-whisper" },
        "noise_reduction": { "type": "near_field" }
      },
      "output": { "language": "en" }
    }
  }
}
```

- `session.audio.output.language` — BCP-47 target (`en` or `fr` here). Source is auto-detected.
- `session.audio.input.transcription.model` — enables source-language transcription (monitor).

## Streaming audio

```json
{ "type": "session.input_audio_buffer.append", "audio": "<base64 PCM-16>" }
```

The stream is continuous: there is **no** `response.create` and **no** turn-complete event.

## Server events

- `*.input_transcript.delta`  — recognised source text (delta) → operator monitor.
- `*.output_transcript.delta` — translated text (delta) → caption.
- `*.output_transcript.done` / `.completed` — final transcript, if the server sends one.
- `*.output_audio.delta`      — translated audio; **ignored**.

Because there is no turn boundary, the client accumulates `output_transcript` deltas and
**finalizes a caption after a short idle gap** (`FINALIZE_AFTER`, ~900 ms with no new text),
or immediately on a `*.done` / `*.completed` event. See `openai/client.rs`.

## Notes vs Gemini

- Auto-bidirectional (FR⇄EN per utterance) is **not** available here: like Gemini's dedicated
  translate model, the target language is fixed for the session.
- Reconnect/backoff, level metering, and the caption event shape are shared with the Gemini
  path (`realtime.rs`); only the connection, setup frame, and event schema differ.
