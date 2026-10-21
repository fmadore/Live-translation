# Mistral Voxtral Mini Transcribe Realtime 3 API

Cloud subtitle integration. For local subtitles without an account or key, see
[Whisper](local-whisper.md).

Verified 21 October 2026 against Mistral's Voxtral 3 documentation:
[realtime transcription](https://docs.mistral.ai/studio/audio/speech-to-text/transcribe-in-realtime),
[turn detection](https://docs.mistral.ai/studio/audio/speech-to-text/turn-detection),
[context biasing](https://docs.mistral.ai/studio/audio/speech-to-text/bias-context-and-vocabulary),
[diarization](https://docs.mistral.ai/studio/audio/speech-to-text/diarize-speakers),
[troubleshooting](https://docs.mistral.ai/studio/audio/speech-to-text/troubleshooting),
[limits](https://docs.mistral.ai/studio/audio/specs-and-limits/limits),
[migration from Voxtral 2](https://docs.mistral.ai/studio/audio/migrate-from-voxtral-2) and the
[model card](https://docs.mistral.ai/models/voxtral-mini-transcribe-realtime-3-0). Also checked:
the realtime models generated in the `mistralai` Python SDK 3.2.0 and the official
[`mistralai` realtime connection source](https://github.com/mistralai/client-python/blob/main/src/mistralai/extra/realtime/connection.py).
The same day, a [live check](#live-check-21-october-2026) ran both models. The 1 August 2026
verification of Realtime 2602 used Mistral's earlier guide, which it now files under
[deprecated pages](https://docs.mistral.ai/studio/audio/deprecated/speech_to_text/realtime_transcription),
and its [client-auth page](https://docs.mistral.ai/studio/audio/deprecated/speech_to_text/realtime_transcription/client_auth).

## Scope

`voxtral-mini-transcribe-realtime-3-0` (Voxtral Mini Transcribe Realtime 3, released 20 October
2026) produces same-language realtime transcription. It does **not** translate, so the UI
exposes it under **Live subtitles**. Mistral lists it as a Public Preview Premier model at
$0.008 per minute of audio ($0.48 an hour), and counts 45 languages and dialects for it without
publishing the list. The app sends 16 kHz mono signed 16-bit little-endian PCM with a default
target streaming delay of 480 ms, Mistral's recommended starting point. Override the delay with
`MISTRAL_TARGET_STREAMING_DELAY_MS` to test the latency/accuracy trade-off. Realtime 3 accepts
any integer from 80 to 1200, or 2400. Any other value gets an `error` event that stops the
source.

The previous model, `voxtral-mini-transcribe-realtime-2602`, is still generally available
at $0.006 per minute ($0.36 an hour). It covers 13 languages, and its weights are open under
Apache 2.0. It speaks the same protocol: set `MISTRAL_TRANSCRIBE_MODEL` in a debug build to
compare the two. Mistral's migration page has a deprecation-timeline heading but no dates yet.
Realtime 3 has no open weights: its card lists no licence and no weights.

## Connection and setup

```text
wss://api.mistral.ai/v1/audio/transcriptions/realtime?model=voxtral-mini-transcribe-realtime-3-0
Authorization: Bearer <MISTRAL_API_KEY>
```

Mistral's examples use the alias `voxtral-mini-transcribe-realtime-3`, and
`voxtral-mini-transcribe-realtime-latest` also resolves to Realtime 3. The app pins `-3-0`, so a
moved alias cannot change the model under a release. `session.created` names the checkpoint
being served: `voxtral-mini-transcribe-realtime-3-0-4` on 21 October.

```json
{
  "type": "session.update",
  "session": {
    "audio_format": { "encoding": "pcm_s16le", "sample_rate": 16000 },
    "target_streaming_delay_ms": 480
  }
}
```

Realtime 3 fixes a session's settings when the first audio chunk arrives. A later
`session.update` gets the error `Session fields cannot be changed after audio has started.` The
app sends its only `session.update` straight after the handshake, before any audio. It does not
wait for `session.created`, which Mistral's guide asks for, but the server applies the early
update: on 21 October `session.updated` echoed the 480 ms delay on both models. 2602 has
accepted the same order since 0.4.0.

Each audio chunk is base64 encoded:

```json
{ "type": "input_audio.append", "audio": "<base64 PCM16>" }
```

The server emits `session.created`, `session.updated`, and incremental
`transcription.text.delta` events whose `text` is displayed directly. Deltas only append: one
never corrects earlier text, and each carries its own spaces and punctuation. Pauses finalize a
subtitle line through the shared 900 ms idle boundary.

## Session features not used yet

Realtime 3 adds these `session.update` fields. They are listed here for the follow-ups in
[#153](https://github.com/fmadore/Live-translation/issues/153). The client ignores event types
it does not know, so nothing changes until the app sends a field.

| Field | Shape and default | What it adds |
| --- | --- | --- |
| `language` | Two-letter ISO 639-1 code; unset | Pins the spoken language. Unset, the model detects it. The app has no spoken-language selector for Mistral ([language coverage](language-coverage.md)). |
| `context_bias` | Array of strings; `[]` | Keyphrases (spaces allowed) or a description of the recording, set for the whole session. Entries share the 65,536-token context window with the audio. In realtime, Mistral finds a keyphrase more reliable than a description. Fits a per-session glossary of speaker names, institutions and jargon. |
| `turn_detection` | Object; unset | `{}` applies the defaults: `start_threshold` 0.5, `eager_end_threshold` 0.5, `end_threshold` 0.33, `min_silence_ms` 500, `end_timeout_ms` 3000. Adds `turn.start`, `turn.eager_end`, `turn.resume` and `turn.end` events. Each carries `time_s` in audio time, and `turn.end` also carries an `end_reason` of `threshold` or `timeout`. The last words of a turn arrive in deltas *after* `turn.end`: keep appending until the streaming delay has passed or the next `turn.start`. This could finalize a line instead of the 900 ms idle timer alone. |
| `diarize` | Boolean; `false` | Replaces `transcription.text.delta` with `transcription.segment` events: `{ text, speaker_id, start: null, end: null }`, labels `speaker_1`, `speaker_2`… The client must read these events before diarization is switched on, or every caption goes blank. Labels hold within one session only, so a reconnect renumbers speakers. Cannot share a session with `turn_detection`. |
| `speaker_references` | `{ name: base64 clip }`; unset | Names known speakers: up to 8 clips of 3 to 15 seconds. Requires `diarize`. |

`session.created` also reports an undocumented `verbatim: false` on both models.

## Graceful stop

Like the official SDK, the client sends `input_audio.flush`, then `input_audio.end`, and drains
through `transcription.done` (or a four-second safety timeout) before closing. Remaining
accumulated text is finalized and included in exports. Mistral's Voxtral 3 guide shows only
`input_audio.end`, but Realtime 3 still accepts the flush: the live check sent both. A
`transcription.done` carries `text`, `segments`, `language` (null in every run so far, whatever
was spoken), `model` and `usage`.

## Errors and the end of a session

A `transcription.done` that arrives mid-stream, outside a close drain, still finalizes its
caption, and the source then moves to a new session instead of ending.

The SDK types an `error` event's payload as `{ message, code }`: `message` a string or an
object, `code` an integer it describes only as an internal code for debugging. No list of codes
is published, so the classification is deliberately narrow: codes in HTTP's transient range
(408, 429, 500, 502–504), or a `type`/`code` of `server_error`, `rate_limit_error`,
`rate_limited`, `timeout` or `session_expired` inside an object `message`, reconnect with
backoff; a 429 whose message mentions a quota, billing, payment or credit does not. A failure
inside Mistral's own backend also reconnects, recognised by its wording rather than its code: a
gRPC `UNAVAILABLE` or `DEADLINE_EXCEEDED`, `grpc_status:14`, "gRPC connection error" or
"connection reset by peer" (a bare "unavailable", which could describe a model the key cannot
use, does not count). Everything else — including any code not listed — stops the source with
Mistral's message, as every error did before 1.7.0.

One real payload has been seen so far, on 7 October 2026 in 1.6.0, mid-session: code 3803 with
a gRPC `UNAVAILABLE` from an internal Mistral service. It ended the source then; since
[#122](https://github.com/fmadore/Live-translation/pull/122) it reconnects, and a test pins it.

Realtime 3 documents its session errors, all with code 3051: a setting changed after audio
started, a delay outside the accepted values, an invalid `language`, `diarize` together with
`turn_detection`, speaker references without `diarize`, out-of-order turn thresholds, and a
field it does not accept (`Extra inputs are not permitted`). Each needs a setting changed, so
they stay fatal; a test pins two of them. A model the key cannot use is refused at the
handshake with HTTP 403, which the runner already treats as permanent ("check the API key and
model access"). **Live check pending:** record further mid-session error payloads here and
widen the rules from them.

Long-lived keys are safe here because the Rust backend, not a web page, opens the WebSocket.
Mistral’s short-lived `rt_*` / `Sec-WebSocket-Protocol` flow is for browser clients that
cannot set an `Authorization` header.

## Live check, 21 October 2026

The check ran two sessions per model, delay and language on the bundled rehearsal recordings,
`rehearsal-en.wav` (20.8 s) and `rehearsal-fr.wav` (23.2 s). Each session followed the app's
sequence: `session.update` straight after the handshake, 100 ms chunks of 16 kHz PCM16 at
real-time pace, then flush and end. The recordings are Windows text-to-speech, so they test the
protocol and relative timing, not accuracy on real speakers.

- **Protocol:** unchanged. Both models sent `session.created`, `session.updated`, append-only
  deltas and `transcription.done`, and no errors.
- **Accuracy:** every run of both models, at every delay, returned the script word for word.
  These recordings cannot tell the models apart.
- **Latency:** at a given delay, Realtime 3's words arrived within 0.03 s of 2602's (the
  median over a run's words). The delay shifts both models by about the same amount.
- **End of the drain:** Realtime 3 sent `transcription.done` 0.5 to 1.4 s after
  `input_audio.end`, against 0.25 to 0.6 s for 2602. Both are well inside the four-second
  timeout.

| Model | Delay | Word timing against 2602 at 480 ms, EN / FR | First text, EN / FR | `done` after `end` |
| --- | --- | --- | --- | --- |
| 2602 | 240 ms | −0.21 s / −0.24 s | 0.8 s / 1.1 s | 0.25–0.3 s |
| 2602 | 480 ms | 0.00 s / 0.00 s | 1.0 s / 1.3 s | 0.3–0.45 s |
| 2602 | 960 ms | +0.50 s / +0.49 s | 1.5 s / 1.8 s | 0.4–0.6 s |
| Realtime 3 | 240 ms | −0.18 s / −0.25 s | 0.8 s / 1.0 s | 0.45–0.8 s |
| Realtime 3 | 480 ms | +0.02 s / +0.02 s | 1.0 s / 1.3 s | 0.7–1.05 s |
| Realtime 3 | 960 ms | +0.53 s / +0.50 s | 1.5 s / 1.8 s | 0.6–1.4 s |

First text counts from the first audio chunk, so it includes each recording's silence before
the first word. The default stays at 480 ms. A shorter delay would bring words 0.2 s sooner, but
its accuracy cost is unmeasured, since the synthetic voices are too clean to show one. A
real-speech comparison is still to do: French and English conference audio, with names and
room acoustics, at 240, 480 and 960 ms.
