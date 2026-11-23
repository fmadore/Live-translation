# OpenAI Realtime transcription API

Cloud subtitle integration ([#251](https://github.com/fmadore/Live-translation/issues/251)).
OpenAI's translated captions use a different endpoint and session; see the
[Realtime Translate notes](openai-realtime-api.md). For subtitles without an account or key,
see [local Whisper](local-whisper.md).

Verified 23 November 2026 against OpenAI's
[Realtime transcription guide](https://developers.openai.com/api/docs/guides/realtime-transcription),
the [Transcription overview](https://developers.openai.com/api/docs/guides/transcription), the
model pages for [`gpt-live-transcribe`](https://developers.openai.com/api/docs/models/gpt-live-transcribe)
and [`gpt-realtime-whisper`](https://developers.openai.com/api/docs/models/gpt-realtime-whisper),
the [Realtime client events](https://developers.openai.com/api/reference/resources/realtime/client-events)
and [server events](https://developers.openai.com/api/reference/resources/realtime/server-events)
references, and the [pricing page](https://developers.openai.com/api/docs/pricing). The same day
a [live probe](#live-probe-23-november-2026) confirmed the endpoint and the session shape.

## Scope

`gpt-live-transcribe` is OpenAI's streaming speech-to-text model: audio in, same-language text
out, as word deltas. It does not translate, so the UI exposes it under **Live subtitles**,
beside Voxtral, Gemini Transcribe and Whisper.

- **Recommended model.** The Transcription overview names `gpt-live-transcribe` as the
  starting model for live audio, and says `gpt-realtime-whisper` "isn't the recommended
  starting model for a new transcription integration"; existing integrations may keep it. The
  translation engine still uses `gpt-realtime-whisper` for its source monitor, which is a
  different session (`openai/client.rs`).
- **Endpoint.** Both model pages list Realtime transcription
  (`v1/realtime/transcription_sessions`) as their only supported endpoint; `v1/live/sessions`,
  which the issue asked about, is marked unsupported for both. Server-side WebSocket clients
  reach it as `wss://api.openai.com/v1/realtime?intent=transcription`.
- **Price.** $0.017 per minute of realtime audio ($1.02 an hour) for either model, on the model
  pages and the pricing page, read on 23 November 2026. Not available on the free tier. Dearer
  than Voxtral ($0.48 an hour) and Gemini Transcribe ($0.30–0.54); Whisper is free.
- **Rate limits.** The `gpt-live-transcribe` page lists 2,000 requests a minute at the Build
  tier; one session per source is far below that.
- **Not returned.** No word timestamps, speaker labels, confidence scores or detected
  language. The app needs none of them.

## Connection and setup

```text
wss://api.openai.com/v1/realtime?intent=transcription
Authorization: Bearer <OPENAI_API_KEY>
```

The URL carries no `model`: the guide warns that the model-based URL opens a conversation
session, which refuses the transcription update below. The same OpenAI key as the translation
engine authenticates it.

`session.created` arrives first and describes a default session with `server_vad` turn
detection and no transcription. The client then sends one `session.update`:

```json
{
  "type": "session.update",
  "session": {
    "type": "transcription",
    "audio": {
      "input": {
        "format": { "type": "audio/pcm", "rate": 24000 },
        "transcription": {
          "model": "gpt-live-transcribe",
          "keywords": ["Université Joseph Ki-Zerbo", "IWAC"],
          "languages": ["fr"]
        },
        "turn_detection": null
      }
    }
  }
}
```

- **Audio.** Base64 mono PCM16 little-endian at 24 kHz, the only PCM rate the session takes
  (G.711 μ-law and A-law are the alternatives), in `input_audio_buffer.append` events of up
  to 15 MiB each. The app sends its usual 100 ms chunks, as for the translation engine.
- **Turn detection.** Must be `null` or absent: the guide says `gpt-live-transcribe` supports
  neither `server_vad` nor `semantic_vad`, and the default session has `server_vad`, so the
  update sets `null` explicitly.
- **`keywords`.** "Product names, acronyms, and other literal terms that may appear in the
  audio": hints, not required output. Each keyword stays on one line, with no `<`, `>`, carriage
  return or line feed; a keyword with one of those makes the server reject the update. No limit
  on their number or length is published.
- **`prompt`.** Free text describing the recording or its setting. It is rejected past "the
  model's length limit", which is not published.
- **`languages`.** Expected input languages, at least one item when present: ISO 639-1 codes
  (`en`, `fr`), selected ISO 639-3 codes (`eng`, `yue`, `cmn`) and regional `zh` codes
  (`zh-cn`, `zh-tw`, `zh-hk`). Unsupported or malformed codes are rejected. The model takes
  `languages` instead of the older singular `language`, and the guide says not to send both.
- **`delay`.** `minimal`, `low`, `medium`, `high` or `xhigh`, trading latency for accuracy. The
  guide shows it with `gpt-live-transcribe`; the client-event reference says it is "only
  supported with `gpt-realtime-whisper`". The probe sent `low`: no error, but `session.updated`
  did not echo it, so whether this model applies it is unknown and the app leaves it out.
- **`noise_reduction`** (`near_field` or `far_field`) and `include` (log probabilities) are
  available and unused.

A later `session.update` may change the transcription settings mid-session.

## Events

Deltas stream as speech arrives, **before** any commit:

```json
{ "type": "conversation.item.input_audio_transcription.delta", "item_id": "item_003", "content_index": 0, "delta": " Hello," }
```

Each delta carries a word or part of one, with its own leading space and punctuation. Deltas
only append: in the probe, none corrected earlier text.

The client ends an audio turn with `{"type":"input_audio_buffer.commit"}`. The server answers
`input_audio_buffer.committed` (with the new `item_id`), `conversation.item.added` and
`conversation.item.done`, the last deltas of that turn, then
`conversation.item.input_audio_transcription.completed`:

```json
{ "type": "conversation.item.input_audio_transcription.completed", "item_id": "item_003", "content_index": 0, "transcript": "Hello, how are you?", "usage": { "type": "duration", "seconds": 9 } }
```

The completed `transcript` is the item's deltas joined, with the first space trimmed. Audio
appended after a commit starts a new item. OpenAI says completions of different turns may
arrive out of order, and that `item_id` matches them to their commits. A turn the model cannot
transcribe ends with `conversation.item.input_audio_transcription.failed`, which carries an
`error` object (`audio_unintelligible` in the reference's example) instead of a transcript.
Committing an empty buffer is an `error`.

With no server turn detection, the guide asks the client to detect the end of speech itself
and commit there.

## Errors and the end of a session

`error` events carry `error.type` (`invalid_request_error`, `server_error`, …), an optional
`error.code`, `message`, `param` and the `event_id` of the client event at fault. "Most errors
are recoverable and the session will stay open." This is the same payload as the translation
session, so the same shared classification applies
([Realtime Translate notes](openai-realtime-api.md#errors-and-the-end-of-a-session)).

The probe's `session.created` had an `expires_at` 60 minutes after the session opened, the
Realtime API's session cap. At the cap OpenAI sends `session_expired`, which the shared
classification already hands over to a new session.

No keepalive is documented. The runner's WebSocket pings prove the path while the room is quiet;
whether a transcription session ends after a long stretch without audio has not been observed.

## Live probe, 23 November 2026

One session, from an ignored test that read the saved key in-process and printed only server
events: the update above (with `prompt`, two `keywords`, `languages: ["en"]` and `delay: "low"`
sent as a second update), then `rehearsal-en.wav` (20.8 s, Windows text-to-speech) resampled
to 24 kHz and sent in 100 ms chunks at real-time pace, with one commit 8 s in and one at the end.

- **Endpoint and setup:** accepted. `session.updated` echoed the model, `turn_detection: null`,
  the `prompt`, the `keywords` and the `languages`; not `delay`.
- **First text:** 1.5 s after the first chunk, one word per delta, each 0.2 to 0.5 s apart.
- **Deltas without a commit:** yes, from the first word.
- **A commit mid-sentence:** the words being spoken at the commit ("transcribed and") still
  arrived as deltas of the committed item, 0.1 s after `committed`; its completion followed 0.4
  s after the commit. The next item began cleanly at the next word ("Shown on screen…"): no
  word was lost or split.
- **Accuracy:** both items' transcripts were the script word for word.
- **Usage:** each completion reported its audio as `{"type":"duration","seconds":9}` and
  `13`.
