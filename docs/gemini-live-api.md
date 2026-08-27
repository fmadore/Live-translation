# Gemini Live API

Two Gemini models, one `BidiGenerateContent` socket, one API key:

| Model | Status | Mode it serves |
| --- | --- | --- |
| `gemini-3.5-live-translate-preview` | Preview | Translated captions |
| `gemini-3.5-transcribe-live` | Stable | Same-language subtitles |

Re-verified 27 August 2026 against Google’s
[Live translation guide](https://ai.google.dev/gemini-api/docs/live-api/live-translate)
(last updated 2026-07-23), the
[Live transcription guide](https://ai.google.dev/gemini-api/docs/live-api/live-transcribe)
(last updated 2026-08-26), the
[model list](https://ai.google.dev/gemini-api/docs/models) and the
[Live API reference](https://ai.google.dev/api/live).

Translation is still preview and has no stable equivalent — Gemini 3.5 Transcribe went GA,
Live Translate did not — so re-check it before the event. Its wire format is unchanged since
the 12 August 2026 verification.

## Gemini 3.5 Live Translate

### Connection and audio

```text
wss://generativelanguage.googleapis.com/ws/google.ai.generativelanguage.v1beta.GenerativeService.BidiGenerateContent?key=<API_KEY>
```

- Model: `gemini-3.5-live-translate-preview`, sent as `models/...` in setup.
- Input: mono PCM16 LE at 16 kHz, base64 in `realtimeInput.audio`.
- Output modality: `AUDIO`. The app discards model audio and displays
  `outputTranscription`; this can still incur audio-output charges.

### Setup

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

### Streaming and events

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

## Gemini 3.5 Transcribe Live

Same endpoint, same audio frame, same key — this is a subtitle engine, not a translator, so
the setup message and the transcription fields differ.

### Setup

```json
{
  "setup": {
    "model": "models/gemini-3.5-transcribe-live",
    "generationConfig": {
      "responseModalities": ["TEXT"]
    },
    "inputAudioTranscription": {
      "languageCodes": [],
      "mode": "SMART"
    }
  }
}
```

- `responseModalities: ["TEXT"]` is what separates the transcription pipeline from a live
  agent. There is no generated audio to discard and no output sidecar to read.
- `inputAudioTranscription` is a `BidiGenerateContentSetup` field here too, and carries
  configuration rather than being an empty marker.
- `languageCodes: []` enables automatic language identification across utterances, including
  code-switching — the right default for a bilingual room. The guide's table lists 84 entries,
  which are **83 distinct BCP-47 codes and 77 distinct languages** once locale variants
  collapse (`en-US`/`en-GB`/`en-IN`, `bn-BD`/`bn-IN`, and so on). User-facing copy says "over
  70 languages" from that count — not the "100+" a first reading of the page suggests.
- `mode: "SMART"` removes fillers and false starts, resolves spoken self-corrections, and
  applies punctuation and casing. Subtitles are read off an overlay by an audience, so
  readability beats a faithful record of every "um". `VERBATIM` is Google's default; it is
  the alternative, not an additional setting. SMART rules out word-level annotations, which
  this app does not use.
- `customVocabulary` (up to 1,000 terms, best under 100) is available and not yet wired up.

### Streaming and events

The audio frame is byte-for-byte the one live translate uses — 16 kHz mono PCM16 LE, base64
in `realtimeInput.audio`, 100 ms chunks.

- `serverContent.interimInputTranscription.text` is a speculative partial hypothesis, revised
  as the speaker talks.
- `serverContent.inputTranscription.text` is the finalized, authoritative transcript for one
  speech segment, emitted when the speaker pauses.

**Both describe the same segment, so each replaces the caption buffer rather than extending
it.** This is the one place the two Gemini clients genuinely diverge: live translate appends
`inputTranscription` deltas, and appending them here would repeat every revised hypothesis on
screen. One finalized segment becomes one transcript line, broken where the speaker paused.

`{"realtimeInput":{"audioStreamEnd":true}}` is sent on close so a last segment can still
arrive during the shared runner's drain window.

### What the wire does that the guide does not mention

All three were found by running `live_probe` (below) against the real endpoint on
27 August 2026, and none of them appear in Google's documentation.

- **Replies arrive as WebSocket _binary_ frames carrying UTF-8 JSON, not text frames.** The
  guide's own sample reads `event.data` in a browser, where the difference is invisible. A
  client that matches only on text frames sees the socket connect, accept setup, and then stay
  completely silent. `realtime.rs` already decodes `Message::Binary`, so both Gemini clients
  were unaffected — but it is the first thing to check if this ever appears to hang.
- **`generationComplete` closes a segment**, not the `turnComplete` the translate path sends.
  Nothing in the transcription guide names it. It is what lets the client finalize a segment
  that produced only a speculative hypothesis and no confirmed text.
- **`setupComplete` *is* sent**, even though the guide's WebSocket sample never waits for one.
  Waiting is correct and matches the Live API contract.

### Segmentation is the model's, and it is not sentence-shaped

Observed over the bundled twenty-second recording: with continuous speech and few pauses the
model will run two sentences into a single segment, joining them without a space
(`…Use this recording toThe overlay before…`), and it splits mid-word where it does break
(`…every sentence is` / `described and shown on screen…`, from "transcribed"). Segmentation
also varies between runs on identical audio.

The consequence for the overlay is that a caption grows until the model closes the segment,
which on continuous speech can mean a paragraph rather than a line. This is the model's
behaviour, not something the client corrects: splitting on our own heuristic would invent
sentence boundaries the model did not report, and re-flowing text that is still being revised
would make the caption jump. Voxtral's 900 ms idle finalization produces shorter lines, and
that is a fair reason to prefer it for a speaker who does not pause much.

### The ten-minute cap

> Live transcription sessions support continuous streaming for up to 10 minutes.

Mistral Voxtral has no such limit, so this is the one operational difference between the two
subtitle engines. A room session therefore reconnects several times an hour. Nothing special
handles it: `goAway` (or the socket closing) returns the shared runner to its reconnect path,
the in-flight turn is finalized into the transcript, and because the connection was stable for
well over 30 seconds the backoff resets to one second. The cost is a roughly one- to
two-second caption gap every ten minutes. Choose Voxtral for a session where that gap matters
more than Gemini's language coverage.

Speaker diarization and word-level timestamps are not available over the Live API — they
belong to the non-streaming `gemini-3.5-transcribe` model, which takes uploaded files rather
than a socket and cannot serve this app.

## Re-verifying against the live endpoint

The serialization tests pin both clients to the shapes documented above. They cannot tell us
the documentation is *right* — and it demonstrably is not everywhere, since the translate guide
still shows the transcription sidecars nested under `generationConfig`, which the server
rejects. `gemini::transcribe::live_probe` closes that gap: it puts the production setup message,
audio frame and response types in front of Google's servers with the bundled rehearsal
recording as input, and reports the interim and finalized segments it gets back.

```bash
GEMINI_API_KEY=... cargo test -p live-translation --lib live_probe -- --ignored --nocapture
```

It is `#[ignore]`d, so CI never runs it and it cannot bill anyone by accident. Run it before an
event, and after any change to either setup message. It costs a few cents: about twenty seconds
of streamed audio. It reads `GEMINI_API_KEY` from the environment or from a `.env` at the repo
root — `dotenvy` walks up from the crate directory, so `.env.example` is the right thing to
copy.

Its assertions are deliberately about the *shape of the exchange* rather than about the words
recognized: that setup is accepted, that `generationComplete` arrives, and that speculative
updates outnumber finalized segments — the last being what makes the client's
replace-don't-append rule correct. Transcription accuracy is not something a test should pin.
