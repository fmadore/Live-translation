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

## Target-language catalog — documentation checked 2026-09-22

The [official cookbook](https://developers.openai.com/cookbook/examples/voice_solutions/realtime_translation_guide)
still names 13 targets. The development catalog lists `en es pt fr ja ru zh de ko hi id vi it`.
The source is `src/lib/languages.json`, shared by generated Rust and TypeScript types.

**Endpoint verification is pending.** On 2026-09-22 the opt-in probe could not resolve an
OpenAI API key, including outside the sandbox. Portuguese `pt` versus `pt-BR`/`pt-PT` and
Chinese `zh` versus `zh-Hans`/`zh-Hant` must not be represented as proven until it runs.
The probe opens a separate translation session per candidate, uses the production update
payload, waits for `session.updated` or `error`, prints only language verdicts, and closes
without transmitting audio. It tests all 13 targets plus the four ambiguous alternatives.

Save the key in the app (or set `OPENAI_API_KEY`), then run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --lib probe_target_language_codes -- --ignored --nocapture
```

Record returned codes/verdicts here and update the catalog if needed. A successful session
update proves configuration acceptance, not translation quality; the non-EN/FR real-speech
check on both providers remains separate in [language coverage](language-coverage.md).
