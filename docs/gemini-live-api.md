# Gemini 3.5 Live Translate API

Verified 12 August 2026 against Google’s
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
      "translationConfig": {
        "targetLanguageCode": "en",
        "echoTargetLanguage": true
      }
    },
    "inputAudioTranscription": {},
    "outputAudioTranscription": {}
  }
}
```

The two levels are easy to confuse, because the SDKs flatten them into one `LiveConnectConfig`
object. On the wire they are distinct:

- `inputAudioTranscription` and `outputAudioTranscription` are fields of
  `BidiGenerateContentSetup` itself ([Live API reference](https://ai.google.dev/api/live)).
  Nesting them under `generationConfig` makes the server reject the whole session with
  `Unknown name "inputAudioTranscription" at 'setup.generation_config': Cannot find field.`
- `translationConfig` *is* a `generationConfig` field, and is Gemini Developer API only —
  the SDKs raise on it in Enterprise/Vertex mode.

Both placements are pinned by the converter tests in Google’s own SDK (`python-genai`
`tests/live/test_live.py`: `test_bidi_setup_to_api_with_input_transcription`,
`test_bidi_setup_to_api_with_translation_config`). Older revisions used
`realtimeInput.mediaChunks`; that shape is not used here.

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
- `echoTargetLanguage` stays enabled so speech already in the target language still appears
  in the caption stream; this is essential for bilingual meetings.
- `serverContent.turnComplete` finalizes and advances the per-source turn.
- `goAway` asks the shared runner to reconnect immediately rather than waiting for a socket
  failure.
- A provider `error` becomes a terminal operator-visible status instead of a silent log.
