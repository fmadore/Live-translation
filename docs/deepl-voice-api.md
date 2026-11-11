# DeepL Voice API

Cloud translation integration: speech in, translated captions out.

**Status: experimental.** Built from DeepL's documentation and tested against its documented
messages and loopback servers, but not yet run against the live API, which needs a paid DeepL
API plan. The engine picker says so in its note on DeepL.

Verified 11 November 2026 against DeepL's Voice documentation:
[the Voice API reference](https://developers.deepl.com/api-reference/voice),
[request session](https://developers.deepl.com/api-reference/voice/request-session),
[understanding voice sessions](https://developers.deepl.com/docs/voice/understanding-voice-sessions),
[Voice API requirements](https://developers.deepl.com/docs/voice/voice-api-requirements),
[the real-time quickstart](https://developers.deepl.com/docs/voice/real-time-voice-quickstart),
[supported voice languages](https://developers.deepl.com/docs/voice/supported-voice-languages)
and the WebSocket's
[AsyncAPI document](https://developers.deepl.com/api-reference/voice/voice.asyncapi.yaml). Also
read: DeepL's Python reference client,
[`examples/voice/cli/deepl-voice-api-cli.py`](https://github.com/DeepL/deepl-python/tree/main/examples/voice/cli)
in `deepl-python`, for how it builds the socket URL and assembles transcripts. **No live session
has run yet:** everything below the REST host check is from the documentation, and the
[live probe](#live-probe) is the step that confirms it.

## Scope

DeepL Voice transcribes speech and translates it in one session. The app uses it for
**Translated captions** only (`Provider::DeepL`, serde `deepl`, `can_translate`): DeepL detects
the spoken language (`source_language_mode: "auto"`, no `source_language`), so the operator
picks no spoken language, and the caption is the translation. The source transcript goes to the
operator's monitor as `source`. There is no DeepL subtitle (same-language) variant in this
version, and the engine takes no glossary (`takes_glossary` is false): DeepL's glossaries and
spoken terms are account objects referenced by id, not a word list sent with the session.

- **Audio:** 16 kHz mono PCM16 little-endian, `audio/pcm;encoding=s16le;rate=16000`, in the
  100 ms chunks every capture path produces (3,200 bytes, inside DeepL's 50–250 ms
  recommendation and its 1 s / 100 KB cap, and a whole number of 2-byte frames).
- **Key:** its own credential, account `deepl-api-key` in Credential Manager or the Keychain,
  `DEEPL_API_KEY` in a debug build's `.env`. It travels only in the REST requests'
  `Authorization: DeepL-Auth-Key <key>` header, marked sensitive; the socket carries a
  single-use session token instead, so the key never appears in a WebSocket handshake.
- **Plan:** DeepL makes Voice "available to all customers with a paid DeepL API subscription".
  Its Growth plan lists speech hours; the free Developer plan lists none.
- **Billing:** per open stream. Like every engine, one session per source per caption language.

## The paid host only

Every request goes to `api.deepl.com`. DeepL's convention sends Free keys (those ending in
`:fx`) to `api-free.deepl.com`, but on 11 November 2026 that host answered
`POST /v3/voice/realtime` with **404 even without credentials**: the route does not exist there.
`api.deepl.com` answered the same request without credentials with **403**. So Voice is
paid-plan only, and a `:fx` key can never reach it:

- `secrets::check_plan` refuses a DeepL key ending in `:fx` when it is saved, and again at Start
  for a key from the environment, as `error.deeplPaidPlan` (raised with no detail: the sentence is
  complete, and a detail would not be translated). No request is ever made with such a key.
- `DEEPL_API_HOST` points a debug build elsewhere (release builds ignore it, like the other
  `*_HOST` overrides). The session endpoint is always reached over HTTPS.

## Connecting

A connection starts over REST, before the socket, in `RealtimeProtocol::prepare`. The runner
calls it before every connection, cuts it short on Pause and Stop as it does the handshake,
holds a handover's audio meanwhile, and bounds it by the 15-second connect timeout.

### Request a session

```text
POST https://api.deepl.com/v3/voice/realtime
Authorization: DeepL-Auth-Key <key>
Content-Type: application/json

{"source_media_content_type":"audio/pcm;encoding=s16le;rate=16000","message_format":"json",
 "source_language_mode":"auto","target_languages":["de"]}
```

The answer is `{ streaming_url, token, session_id }`: for example
`wss://api.deepl.com/v3/voice/realtime/connect` and a one-time token. `formality` is not sent
(the app has no setting for it, and DeepL's default applies). `X-DeepL-Reporting-Tag` is not
sent.

### Open the socket

`{streaming_url}?token=<token>`, the token percent-encoded as DeepL's Python client encodes it.
The client uses the URL DeepL returned, never a fixed one, and in a release build refuses one
that is not `wss`. There is no setup message (`setup_json` returns `None`): audio follows the
handshake.

### Resume, or start again

Tokens are single-use and short-lived; opening a socket with a used token ends the session, and
only the latest token may resume one. DeepL keeps connections "as long as possible" but may
close one at any time. The client therefore keeps the latest token and a flag for whether its
session can still be resumed:

- **A socket that dropped** (closed, reset, or silent past the runner's 30-second inbound
  timeout) resumes its session: `GET https://api.deepl.com/v3/voice/realtime?token=<latest>`
  with the same `Authorization` header answers with a new `{ streaming_url, token }`, and the
  session's options and translation context carry over. If DeepL refuses the resume with a
  4xx other than 401, 403, 408 or 429 (an outdated or spent token: "a 400-level error" in the
  quickstart's words), the client requests a new session in the same step.
- **After DeepL's `error`, after `end_of_stream`, and after our `end_of_source_media`** (sent on
  every Pause and Stop) the session is over, and the next connection requests a new one.

### REST failures

| Status | Meaning | What the source does |
| --- | --- | --- |
| 401, 403 | Missing or invalid key, or a plan without Voice | Ends: `error.providerRejected`, detail `DeepL — HTTP 403` |
| 400 | Invalid options | Ends: `error.providerStopped`, detail `DeepL — HTTP 400: <DeepL's message>` |
| 456 | Quota exceeded | Ends: `error.providerStopped`, detail `DeepL — HTTP 456: <DeepL's message>` |
| 408, 429, 5xx, network | Transient | Reconnecting with backoff; the status and message are the detail |
| Other 4xx | Unexpected refusal | Ends: `error.providerRejected` |

Error bodies come in two shapes, `{ message, code? }` from the API and `{ error: { message } }`
from DeepL's edge; both are read. DeepL asks for its `X-Trace-ID` response header with any report,
so every refused request logs it (with the status and message, never the key or the token).
`reqwest` errors are stripped of their URL before they are kept, because a resume's URL carries
the token.

## The stream

Every message is a JSON text frame with one top-level key naming it.

Client to DeepL:

```json
{"source_media_chunk":{"data":"<base64 PCM16>"}}
{"end_of_source_media":{}}
```

DeepL to client (the documented examples):

```json
{"source_transcript_update":{"concluded":[{"language":"en","text":"Hello, how are you","start_time":0,"end_time":1500}],"tentative":[{"language":"en","text":" today?","start_time":1500,"end_time":2000}]}}
{"target_transcript_update":{"language":"es","concluded":[{"text":"Hola, ¿cómo estás","start_time":0,"end_time":1500}],"tentative":[{"text":" hoy?","start_time":1500,"end_time":2000}]}}
{"end_of_source_transcript":{}}
{"end_of_target_transcript":{"language":"fr"}}
{"end_of_stream":{}}
{"error":{"request_type":"source_media_chunk","error_code":400,"reason_code":4000403,"error_message":"Audio format not supported"}}
```

Concluded segments are final and arrive once; tentative ones replace the tentative ones before
them. `end_of_stream` is the last message, after which closing is safe. `target_media_chunk` and
`end_of_target_media` are speech the app never asks for, and are ignored like any other unknown
message.

### Quiet rooms

DeepL ends a session after 30 seconds without data. A microphone delivers chunks continuously,
silence included, but system audio can deliver none while nothing plays: event-driven WASAPI
loopback is not signalled when no stream renders (Windows behaviour, not measured in this app). The runner therefore sends what `RealtimeProtocol::keepalive` asks for: 100 ms
of silence whenever no audio has gone out for 10 seconds. Speech resets the wait, so a talking
room sends nothing extra. The silence is a fraction of a percent of audio, if DeepL bills it.

## How a caption line closes

The caption is the target transcript; the source transcript is the operator's monitor. A turn
holds the target text DeepL has concluded since it began, then the current tentative text.
Every target update is an interim caption, and counts as activity for the idle boundary; source
updates are interims that do not (the source leads the translation, and would close a line
before its translation arrived). A line closes as final, on the first of:

1. **A concluded sentence.** When DeepL concludes target text and the turn's concluded text ends
   a sentence (`.`, `?`, `!`, `…` and their CJK, Arabic and Devanagari forms, closing quotes and
   brackets aside, `realtime::wire::ends_sentence`, shared with Mistral), the line is that
   concluded text alone. Tentative text after it is the next sentence's start, and opens the
   next line with the following update. One concluded segment is one line when DeepL concludes
   whole sentences; a run of phrase-sized segments makes one line per sentence.
2. **The idle boundary.** No target text for 1 s closes the line (`FINALIZE_AFTER`), for a
   speaker who trails off without punctuation, and for scripts without sentence punctuation
   (Thai). While DeepL still holds tentative text the line stays open (`keeps_turn_open`), for
   at most the runner's four-second `MAX_HOLD`.
3. **`end_of_stream`**, which closes whatever is open; mid-stream it also moves the source to a
   new session.

A line closed on a timer may have shown tentative text that DeepL concludes later. Target
segments ending no later than what a final line already showed are dropped, so the words are not
printed twice. Segment times are trusted within one connection only: after a reconnect nothing
is dropped (duplicated words are the lesser fault), and a segment without times is always kept.

The monitor's source text for a line is the source DeepL concluded before the line closed. The
source leads, so the start of the next sentence's source can appear under the line before it.

Because tentative text is replaced, an interim line's start can change; a Zoom share therefore
publishes DeepL's final lines only (`zoom::interims_only_grow`).

## Graceful stop and errors

On Pause and Stop the client sends `end_of_source_media`; DeepL concludes every tentative
segment, sends the remaining updates, `end_of_source_transcript`, `end_of_target_transcript`
and finally `end_of_stream`, which ends the drain (`MessageControl::Closed`), within the
runner's four-second limit.

DeepL's `error` message ends its session. Its `error_code` is HTTP-style, read by the shared
`realtime::wire::classify`: 408, 429 and 5xx reconnect, to a **new** session, with backoff;
everything else, and quota or key wording under any code, ends the source with DeepL's message,
for example `DeepL — Audio format not supported (error 400, reason 4000403)`.

A socket handshake refused with 400, 401, 403 or 404 is treated as every provider's is: as
permanent. For DeepL that would mean a token refused straight after DeepL issued it, which
the documentation gives no reason to expect.

## Languages

Captions go into the 42 catalog languages DeepL translates into (`TargetLanguage::supported_by`;
the list is in [language coverage](language-coverage.md)). `deepl_target_code` maps each to the
spelling in DeepL's `target_languages` enum. Where the two differ:

| Catalog | Sent | Why |
| --- | --- | --- |
| `en` | `en-US` | DeepL distinguishes American and British English; the catalog has one English, and American is the choice. |
| `no` | `nb` | DeepL's only Norwegian is Bokmål. |
| `zh-Hans`, `zh-Hant` | `zh-HANS`, `zh-HANT` | The enum's own casing. BCP 47 is case-insensitive, but the enum may be matched exactly. |
| `pt-BR`, `pt-PT` | `pt-BR`, `pt-PT` | Unchanged. |

All four rows follow the documented enum and are **unverified** until the live probe sends them.
DeepL also lists `ga`, `mt` and `tl` (Tagalog, the catalog's `fil`) as targets; the catalog does
not offer them for DeepL in this version.

On the spoken side, DeepL auto-detects among its source languages, but some of them are served
by partners. Per DeepL's language table, partner-transcribed languages (Arabic among them) are
not auto-detected: speech in them is not recognised unless the session names it as its source,
which the app does not do.

## Limits

- **Targets per session:** DeepL takes up to five. The app opens one session per caption
  language, so a second caption language is a second session over the same audio, billed again.
  Follow-up: share one session between both lanes.
- **Input speed:** capped at twice real time. Capture sends at real time. After a planned
  handover the runner replays the audio that queued while it reconnected, about a second, in a
  burst; whether DeepL throttles that is unverified, and a planned handover only follows a
  mid-stream `end_of_stream` after at least 30 seconds of a healthy session.
- **Session length and concurrency:** DeepL states no maximum for either.

## What is unverified

Until the live probe runs with a paid key:

- the whole stream: message shapes, the cadence of tentative and concluded updates, and whether
  concluded target segments are sentence-sized, which decides how lines break;
- whether segment times are monotonic within a connection, and across a resume;
- the target codes in [languages](#languages), `en-US` and `zh-HANS` especially;
- the keepalive: that 100 ms of silence every 10 seconds keeps a quiet session open;
- what a resume returns once the latest token has expired, and how long tokens live;
- whether DeepL closes the socket itself after `end_of_stream`;
- the 2× real-time cap against the handover replay.

Verified: on 11 November 2026, `api-free.deepl.com` answered `POST /v3/voice/realtime` with 404
and `api.deepl.com` with 403, both without credentials.

## Live probe

An ignored test streams the bundled English rehearsal recording through the production client
and the real runner to DeepL, translated into German or into `DEEPL_PROBE_TARGET` (a catalog
code), and prints every final line with its source text and the time it closed, and any status
that carried a reason. It ends with a Stop, so it also exercises `end_of_source_media` and the
drain. It bills the key, refuses a `:fx` key, and CI never runs it:

```bash
DEEPL_API_KEY=... cargo test --manifest-path src-tauri/Cargo.toml --lib live_probe -- --ignored --nocapture
```

`DEEPL_API_KEY` may also come from `.env` at the repository root. Record what it shows here, in a
dated section like [Mistral's live check](mistral-realtime-api.md#live-check-21-october-2026),
and move each confirmed item out of [what is unverified](#what-is-unverified).

## Where it lives

| File | Part |
| --- | --- |
| `src-tauri/src/deepl/client.rs` | `DeepLClient`: session state, `prepare`, turn mapping |
| `src-tauri/src/deepl/session.rs` | The REST requests and how their statuses end or retry |
| `src-tauri/src/deepl/protocol.rs` | Wire types and `deepl_target_code` |
| `src-tauri/src/deepl/tests.rs` | Turn and error tests, loopback sessions, the live probe |
| `src-tauri/src/realtime/` | `prepare`, the optional setup frame and `keepalive` in the shared runner |
| `src-tauri/src/secrets.rs` | The `deepl-api-key` credential and `check_plan` |
