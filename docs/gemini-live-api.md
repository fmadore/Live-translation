# Gemini 3.5 Live Translate — API surface (verified June 2026)

Verified against the live Google announcement and developer docs at scaffolding time.
**Re-verify before the event** — this API is in preview and evolves quickly.

Sources:
- Announcement: <https://blog.google/innovation-and-ai/models-and-research/gemini-models/gemini-live-3-5-translate/>
- Live translation guide: <https://ai.google.dev/gemini-api/docs/live-api/live-translate>
- WebSockets reference: <https://ai.google.dev/api/live>
- Get started (WebSockets): <https://ai.google.dev/gemini-api/docs/live-api/get-started-websocket>

## Connection

```
wss://generativelanguage.googleapis.com/ws/google.ai.generativelanguage.v1beta.GenerativeService.BidiGenerateContent?key=<API_KEY>
```

- **Model:** `gemini-3.5-live-translate-preview` (sent as `models/gemini-3.5-live-translate-preview`).
- **Auth (dev):** API key as the `key` query parameter.
- **Auth (production / client-side):** mint an **ephemeral token** on the `v1alpha`
  endpoint and connect with that, so the long-lived key never ships in the client. We keep
  the key in the OS keychain and connect from Rust, which is equivalent in trust terms; the
  ephemeral-token path is a future hardening step (see `docs/architecture.md`).

## Audio format

| Direction | Format |
|-----------|--------|
| Input  | raw **16-bit PCM, 16 kHz, mono, little-endian**, sent in ~100 ms chunks |
| Output | raw 16-bit PCM, **24 kHz**, mono, little-endian (we discard it — captions only) |

MIME type on input chunks: `audio/pcm;rate=16000`.

## Why we read transcriptions, not text parts

The Live Translate model is **speech-to-speech**: its primary output is translated *audio*.
To show **text captions** we enable input & output transcription and render the
**output transcription** (the translated text). We never play the returned audio, so it
can't talk over the live speaker.

## Setup message (first frame after connect)

```json
{
  "setup": {
    "model": "models/gemini-3.5-live-translate-preview",
    "generationConfig": {
      "responseModalities": ["AUDIO"],
      "translationConfig": {
        "targetLanguageCode": "en",
        "echoTargetLanguage": false
      }
    },
    "inputAudioTranscription": {},
    "outputAudioTranscription": {}
  }
}
```

- `targetLanguageCode` — BCP-47 caption language (`en` or `fr` here). Source language is
  auto-detected.
- `echoTargetLanguage` — when input is already in the target language: `true` repeats it,
  `false` stays silent. We use `false`.

The server replies with `{"setupComplete": {}}` when ready for audio.

## Streaming audio

```json
{
  "realtimeInput": {
    "mediaChunks": [
      { "mimeType": "audio/pcm;rate=16000", "data": "<base64 PCM-16>" }
    ]
  }
}
```

> Newer revisions of the API also accept `realtimeInput.audio` as a single Blob. If
> `mediaChunks` is rejected, switch the field in `gemini/protocol.rs`.

## Server messages

```json
{ "serverContent": {
    "inputTranscription":  { "text": "… recognised source text (delta) …" },
    "outputTranscription": { "text": "… translated text (delta) …" },
    "modelTurn": { "parts": [ { "inlineData": { "mimeType": "audio/pcm;rate=24000", "data": "…" } } ] },
    "turnComplete": true
} }
```

- Transcription `text` arrives as **deltas**; concatenate within a turn.
- `turnComplete: true` finalizes the current caption; we then reset the accumulator and
  bump the turn id.
- `modelTurn` audio is ignored.
- `{ "goAway": {…} }` warns the connection is about to close — we proactively reconnect.

## Resilience

- Send a chunk roughly every 100 ms.
- Auto-reconnect on socket drop or `goAway`, re-sending the setup frame, with exponential
  backoff. A conference can't tolerate a dead session.
