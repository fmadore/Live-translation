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

## Two engines (operator-selectable)

The app supports two paths from speech to translated text; the operator picks one and we
compare them at the rehearsal.

### A. Live Translate (dedicated, `gemini-3.5-live-translate-preview`)

**Speech-to-speech**: its primary output is translated *audio*. There is **no text-only
mode** — text is available only as a sidecar via `outputAudioTranscription`. We render that
output transcription as the caption and never play the audio, so it can't talk over the
speaker. Purpose-tuned for translation across 70+ languages. Note: input transcription for
this model arrives as one complete message (not streamed partials); output transcription
streams.

### B. Speech → Text (general, `gemini-live-2.5-flash`)

A general half-cascade Live model with `responseModalities: ["TEXT"]` and a translate
**system instruction**. Audio in → translated **text** out, with **no audio synthesized**.
The translated text arrives in `serverContent.modelTurn.parts[].text`. More promptable
(academic terminology, formatting) and lighter, but not purpose-tuned for simultaneous
interpretation. Model id is configurable via `GEMINI_STT_MODEL`.

> ⚠️ Verify both model ids before the event — preview ids change. The
> `gemini-live-2.5-flash` family is the half-cascade Live model recommended for TEXT output;
> `*-native-audio` variants are optimized for audio output instead.

## Setup message (first frame after connect)

**Engine A — Live Translate:**

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

**Engine B — Speech → Text:**

```json
{
  "setup": {
    "model": "models/gemini-live-2.5-flash",
    "generationConfig": { "responseModalities": ["TEXT"] },
    "systemInstruction": {
      "parts": [{ "text": "…translate the incoming speech into English…" }]
    },
    "inputAudioTranscription": {}
  }
}
```

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
- Caption text source by engine:
  - **Live Translate** → `outputTranscription.text` (`modelTurn` audio is ignored).
  - **Speech → Text** → `modelTurn.parts[].text` (there is no `outputTranscription`).
- `{ "goAway": {…} }` warns the connection is about to close — we proactively reconnect.

## Resilience

- Send a chunk roughly every 100 ms.
- Auto-reconnect on socket drop or `goAway`, re-sending the setup frame, with exponential
  backoff. A conference can't tolerate a dead session.
