# Gemini 3.5 Live Translate API

Verified 1 August 2026 against Google’s
[Live translation guide](https://ai.google.dev/gemini-api/docs/live-api/live-translate) and
[Live API reference](https://ai.google.dev/api/live). This is a preview model; re-check it
before the event.

## Connection and audio

```text
wss://generativelanguage.googleapis.com/ws/google.ai.generativelanguage.v1beta.GenerativeService.BidiGenerateContent?key=<API_KEY>
```

- Model: `gemini-3.5-live-translate-preview`, sent as `models/...` in setup.
- Input: mono PCM16 LE at 16 kHz, base64 in `realtimeInput.audio`.
- Output modality: `AUDIO`. The app discards model audio and displays
  `outputTranscription`; this can still incur audio-output charges.

## Setup

```json
{
  "setup": {
    "model": "models/gemini-3.5-live-translate-preview",
    "generationConfig": {
      "responseModalities": ["AUDIO"],
      "inputAudioTranscription": {},
      "outputAudioTranscription": {},
      "translationConfig": {
        "targetLanguageCode": "en",
        "echoTargetLanguage": false
      }
    }
  }
}
```

Google’s current guide places both transcription settings and `translationConfig` inside
`generationConfig`. Older revisions used root-level transcription fields and
`realtimeInput.mediaChunks`; those shapes are no longer used here.

## Streaming and events

```json
{
  "realtimeInput": {
    "audio": {
      "mimeType": "audio/pcm;rate=16000",
      "data": "<base64 PCM16>"
    }
  }
}
```

- `serverContent.inputTranscription.text` is source-language monitor text.
- `serverContent.outputTranscription.text` is translated caption text.
- `serverContent.turnComplete` finalizes and advances the per-source turn.
- `goAway` asks the shared runner to reconnect immediately rather than waiting for a socket
  failure.
- A provider `error` becomes a terminal operator-visible status instead of a silent log.
