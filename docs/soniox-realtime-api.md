# Soniox real-time API (research, issue #250)

Research notes for a possible fifth cloud engine. **Nothing here is built**: no Soniox code is in
the app, and no Soniox key has been used, so every fact below is from Soniox's documentation and
policies, not from a live session. The [recommendation](#recommendation) closes the file; the
[manual protocol](#manual-quality-protocol) is the billable run that decides it.

Read in full on 23 November 2026 (Soniox moved its documentation from `soniox.com/docs` to
`docs.soniox.com`; the old links redirect):

- [WebSocket API reference](https://docs.soniox.com/api-reference/stt/websocket-api), with its
  list of error messages;
- [real-time transcription](https://docs.soniox.com/stt/rt/real-time-transcription) (tokens,
  audio formats);
- [speech-to-text translation](https://docs.soniox.com/translation/stt-translation) and
  [real-time translation](https://docs.soniox.com/translation/stt-translation/rt-translation)
  (`one_way`, `two_way`, token format);
- [models and changelog](https://docs.soniox.com/stt/models);
- [supported languages](https://docs.soniox.com/translation/supported-languages) (translation)
  and [for transcription](https://docs.soniox.com/stt/concepts/supported-languages);
- [context](https://docs.soniox.com/stt/concepts/context),
  [language hints](https://docs.soniox.com/stt/concepts/language-hints),
  [language restrictions](https://docs.soniox.com/stt/concepts/language-restrictions) and
  [language identification](https://docs.soniox.com/stt/concepts/language-identification);
- [endpoint detection](https://docs.soniox.com/stt/rt/endpoint-detection) and
  [manual finalization](https://docs.soniox.com/stt/rt/manual-finalization);
- [ping and keepalive](https://docs.soniox.com/stt/rt/connection-keepalive),
  [stream reconnect](https://docs.soniox.com/stt/rt/stream-reconnect),
  [error handling](https://docs.soniox.com/stt/rt/error-handling), the
  [errors reference](https://docs.soniox.com/api-reference/errors) and
  [real-time limits](https://docs.soniox.com/stt/rt/limits-and-quotas);
- [WebSocket authentication](https://docs.soniox.com/authentication/websocket-authentication),
  [console limits](https://docs.soniox.com/console/limits) and
  [projects and regions](https://docs.soniox.com/console/projects);
- [pricing](https://soniox.com/pricing);
- [security and privacy](https://docs.soniox.com/security-and-privacy),
  [data residency](https://docs.soniox.com/data-residency), the
  [Privacy Policy](https://soniox.com/policies/privacy-policy) and the
  [Terms of Service](https://soniox.com/policies/terms-of-service) (both last updated
  29 June 2026).

Not read: the data processing agreement and the subprocessor list, which Soniox shows only
inside its Console, after sign-in ([policies](https://soniox.com/policies)).

## What the issue got right and wrong

The issue was written from search summaries on 20 November 2026. Read in full:

- **Confirmed:** one WebSocket returns transcript and translation tokens together, told apart by
  `translation_status`; `one_way` takes a single `target_language`; `two_way` takes
  `language_a` and `language_b`; about $0.12 an hour for real-time transcription, billed by
  token.
- **Corrected:** the current model is `stt-rt-v6` (released 3 November 2026). `stt-rt-v5` is
  deprecated and is removed on 15 December 2026, after which requests for it are served by v6;
  `stt-rt-v4` is already an alias for v5.
- **Corrected:** Soniox's pricing FAQ says translation is included "at no extra cost", but its
  token table bills translated text as output tokens. The estimate of about $0.18 an hour with
  translation is the honest one; see [cost](#cost).

## Scope for this app

Soniox's real-time model transcribes speech and, when asked, translates it on the same stream.
That would let one engine serve **Translated captions** (`one_way`, the caption being the
translation and the transcript going to the operator's monitor as `source`) and, as a second
provider variant like Gemini's two rows, **Live subtitles** (no translation block). A first
version would do translation only.

## Connection and setup

```text
wss://stt-rt.soniox.com/transcribe-websocket
Authorization: Bearer <SONIOX_API_KEY>
```

The key goes in the `Authorization` header of the upgrade request, which the runner's
`bearer_request` already builds. The older `api_key` field of the start message is deprecated
and refused from 15 January 2027, so it must not be used. A bad key does not fail the handshake:
the socket opens, then an error frame (401, `unauthenticated`) arrives and Soniox closes it.

The first text frame is the start request. For this app it would be:

```json
{
  "model": "stt-rt-v6",
  "audio_format": "pcm_s16le",
  "sample_rate": 16000,
  "num_channels": 1,
  "enable_endpoint_detection": true,
  "context": { "terms": ["Université Joseph Ki-Zerbo", "IWAC"] },
  "translation": { "type": "one_way", "target_language": "en" }
}
```

- **Audio:** 16 kHz mono PCM16 little-endian, as every capture path produces, in 100 ms chunks
  (3,200 bytes, far below the 2,097,144-byte cap on one message). Soniox documents audio as
  **binary** frames. One of its error messages says a text frame carrying standard base64 is
  also accepted, but no page documents that.
- **No acknowledgement:** nothing confirms the start request. The first message is a token
  response, or an error frame and a close. With `enable_reconnect` the server first sends a
  `reconnect` message, and a JSON ping sent after the start request is answered only once the
  stream has started; either could serve as an acknowledgement if one is ever needed.
- **Optional fields not proposed for the first version:** `language_hints` and
  `language_hints_strict` (bias or restrict the spoken language; the app has no spoken-language
  selector for cloud engines), `enable_language_identification` (a `language` on each token;
  translation tokens carry `language` and `source_language` anyway),
  `enable_speaker_diarization`, `max_endpoint_delay_ms` (500–3000, default 2000),
  `endpoint_sensitivity` (−1 to 1), `endpoint_latency_adjustment_level` (0–3),
  `idle_timeout_ms` (3000–20000, default 20000), `enable_reconnect`, `client_reference_id`.
- **Undocumented fields** that appear only in the error list: `translation.source_languages`,
  `translation.exclude_source_languages` and `max_non_final_tokens_duration_ms`. Not to be
  relied on.

## The token stream

Every response is `{ tokens, final_audio_proc_ms, total_audio_proc_ms }`. A token has `text`,
`is_final`, `confidence`, `start_ms`/`end_ms` (spoken tokens only), and with translation
`translation_status`, `language` and `source_language`.

- **Final tokens** are sent once and never change. **Non-final tokens** are provisional: each
  response carries the whole current non-final tail, which replaces the previous one.
- `translation_status` is `original` (spoken text that will be translated), `none` (spoken text
  that will not be, such as a third language under `two_way`) or `translation` (translated text,
  with `source_language`).
- Transcript tokens come first; translation tokens follow in chunks, before the sentence ends.
  Soniox warns that the two are not mapped token for token. Translation tokens have no times.
- Tokens are sub-word pieces carrying their own spaces (`" ever"`, `"yon"`, `"e"`), so text is
  concatenated as received, like Mistral's deltas.
- `<end>` is a special final token that marks an endpoint (with `enable_endpoint_detection`);
  `<fin>` answers a manual `{"type": "finalize"}`. Neither is caption text.
- `{ "tokens": [], "finished": true, ... }` ends the stream, after which Soniox closes the
  socket.

This is the shape DeepL's concluded and tentative segments already have in this app: final
tokens are concluded text, the non-final tail is tentative text.

## How Soniox would map onto `RealtimeProtocol`

| Trait item | Soniox |
| --- | --- |
| `prepare` | Nothing (default). |
| `connect_request` | `bearer_request` to the regional host; see [where the data goes](#privacy-facts-for-docsprivacymd). |
| `setup_json` | The start request above. |
| `wait_for_setup_complete` | `false`, like OpenAI, Mistral and DeepL; the runner's `UNCONFIRMED_OPEN` rule covers a socket closed before any message. |
| `audio_frame` | Binary PCM, which needs a small runner change: the trait returns a `String` and `SocketIo` sends text. Base64 in text frames would work without it if the undocumented acceptance holds, at a third more upload. |
| `keepalive` | 100 ms of silence after 10 s without audio, as for Mistral and DeepL: Soniox ends a stream after 20 s without a client message (408, `request_timeout`). Silence costs nothing extra, since the whole stream duration is billed anyway. |
| `closing_json` | One empty text frame, `""`, which ends the stream. An empty binary frame is an empty audio chunk and does not. |
| `drain_complete` | The response with `finished: true` (`MessageControl::Closed`). |
| `handle_message` | Tokens into the turn; see below. |
| `finalize_after` | About 1 s, as DeepL, to measure. |
| `keeps_turn_open` | While non-final translation text is pending, or final `original` text has no translation yet. |

### Turns, interims and finals

A turn would hold the final translation tokens received since it began, then the current
non-final translation tail; `source` the same for `original` and `none` tokens. Every response
that changes the translation is an interim with `Signal::TranscriptActivity`; a change in the
transcript alone is an interim without it (the transcript leads, as with DeepL). A line closes on
the first of:

1. **A finished sentence:** final translation text that ends a sentence
   (`realtime::wire::ends_sentence`). The line is that final text; the tail opens the next one.
   This is DeepL's rule 1 unchanged.
2. **An endpoint:** `<end>` finalizes the speaker's utterance, but its translation may still be
   on the way. `Signal::EndOfTurn(grace)` with a grace of the measured translation lag would
   close the line then, as Mistral does with `turn.end`. Soniox designed endpointing for voice
   agents; in a continuous talk it may rarely fire (Mistral's English rehearsal recording had a
   single `turn.end`), so rule 1 does most of the work.
3. **The idle boundary**, held by `keeps_turn_open` up to the runner's four-second `MAX_HOLD`.
4. **`finished`**, which closes whatever is open; mid-stream it hands over to a new stream, as
   `end_of_stream` does for DeepL.

Open question for the [protocol](#manual-quality-protocol): under `one_way`, speech already in
the target language. The docs say `one_way` translates every language into the target, and show
`none` only for `two_way`. If target-language speech comes back as `none` tokens, the caption
must show those tokens (the equivalent of Gemini's `echoTargetLanguage`), not stay blank.

### Session length and errors

- **Session limit:** 300 minutes of stream, fixed. Reaching it ends the stream with 413,
  `max_duration_reached`. The client should treat that as a planned handover (the shared
  `classify` would call an unknown 413 fatal), so a five-hour session reconnects once instead of
  ending. Gemini, by comparison, reconnects every ten minutes.
- **Error frame:** `{ tokens: [], error_code, error_type, error_message, more_info, request_id }`,
  then the socket closes. `error_code` is an HTTP status and `error_type` a stable name to branch
  on. Logging `request_id` (never the key) is what Soniox support asks for.

| Code | `error_type` | Source should |
| --- | --- | --- |
| 400 | `invalid_request`, `model_not_available` | End: a setting is wrong (an unsupported pair, a context over 8,000 tokens). |
| 401 | `unauthenticated` | End (`classify` already reads the word). |
| 402 | `organization_balance_exhausted`, `organization_monthly_budget_exhausted`, `project_monthly_budget_exhausted` | End: unknown codes stay fatal; the balance wording is not in `FATAL_WORDING`, but nothing needs to be. |
| 403 | `permission_denied` | End. |
| 408 | `request_timeout` | Reconnect (no message for the idle timeout, or a backend deadline). |
| 413 | `max_duration_reached` | Planned handover. |
| 429 | `limit_exceeded`, `max_concurrent_connections_reached` | Reconnect with backoff (requests per minute, concurrent streams). |
| 500 | `internal_error` | Reconnect. |
| 503 | `service_unavailable` | Reconnect; Soniox says to start a new stream at once. |

A dropped socket without a close frame (status 1006) keeps the stream's concurrency slot until
the idle timeout. The runner's own WebSocket pings every 15 seconds and its 30-second inbound
timeout detect such a drop; whether Soniox answers protocol-level pings is not documented (it
documents a JSON `{"type": "ping"}` answered by `{"type": "pong"}`).

Soniox also asks for audio at real time or close to it and tolerates "brief buffering". The
runner's replay of about a second of queued audio after a handover should be inside that, but it
is unmeasured.

### Stream reconnect, later

With `enable_reconnect`, a dropped stream can continue on a new connection with the same
transcript and bill: the server numbers responses (`seq`), the client buffers its messages until
a pong confirms them and resends from `last_request + 1`. The runner already reconnects by
starting a new stream, losing only the context of the old one. Stream reconnect is worth it only
if conference Wi-Fi drops prove common; it is not proposed for the first version.

### Limits

- 10 concurrent streams and 100 requests a minute per organization by default, raised on request
  in the Console. The app's worst case is four streams (two sources, two caption languages).
- 300 minutes per stream, not raisable.
- A context of at most 8,000 tokens, about 10,000 characters.
- A monthly spending limit can be set per organization and per project in the Console; Soniox
  then refuses requests with 402. It is a useful backstop beside the app's own spending guard.

## Two caption languages

`one_way` takes one target, so a second caption language is a second session over the same
audio, each billed for the audio and its transcription, exactly as the app does for every engine
today.

`two_way` changes that for one case: when the two caption languages are the two spoken languages
of the room (French and English at the workshop), one `two_way` session returns every utterance
in **both** languages, the spoken text as `original` and the other language as `translation`.
Lane French would be every French token, original or translated; lane English every English
token. That halves the bill, but it needs a client that feeds two lanes from one socket, which
is the same change as [#247](https://github.com/fmadore/Live-translation/issues/247) (one DeepL
session for two caption languages), not something the first version would do.

## Bilingual Q&A (#12)

[#12](https://github.com/fmadore/Live-translation/issues/12) asks for an **Auto (FR ⇄ EN)**
caption language in which every utterance is captioned in the language it was not spoken in. Its
option C, provider-side detection, was "not available today on either model". Soniox's `two_way`
is that option: with `language_a: "fr"` and `language_b: "en"`, the caption would be the
`translation` tokens alone, and each one says which language it is in. No detector, no reconnect,
no hysteresis on our side: Soniox's language identification decides.

What it would still need, and what is unknown:

- **Language identification errors.** Soniox warns that in real time a token's language can be
  misread at first and revised. A one-word interjection ("OK, merci") or a code-switched
  sentence may go either way. The protocol's alternating recording tests this.
- **A third language** comes back as `none`, transcribed but not translated. Auto mode would have
  to show it as spoken or drop it.
- **Per-line language.** A `Caption` carries no language; the overlay takes the lane's. An auto
  lane switches language line by line, so captions would need to carry it for `lang` and text
  direction.
- **Pairs.** Some `two_way` pairs are refused ("not supported" is in the error list); French and
  English are expected to work but unverified.
- The interface and type work #12 already lists (a third caption-language choice, `bcp47()` with
  no single code).

So `two_way` is the strongest lead #12 has had, but only for a Soniox user, and only after the
alternating test below.

## Glossary

Soniox is the first translation engine with a vocabulary field that works with translation.
`context` has four optional parts, sent once in the start request:

| Part | Shape | Use |
| --- | --- | --- |
| `general` | `{ key, value }` pairs, 2 to 8 recommended | Domain, topic, organisation, speakers. |
| `text` | A string | Background: a description, a programme. |
| `terms` | Strings | Names and words to spell the listed way. |
| `translation_terms` | `{ source, target }` pairs | How a term is translated; a name mapped to itself is kept unchanged. |

- The setup's **Names and terms** list maps directly onto `terms`. Mapping each entry to itself in
  `translation_terms` as well would keep speakers' and institutions' names untranslated, which is
  what the list is for. `Provider::takes_glossary` and `providerTakesGlossary` would include
  Soniox.
- **Size:** 8,000 tokens in all, about 10,000 characters. The app allows 100 entries of up to 200
  characters, up to 20,000 characters, so a full list can exceed Soniox's cap and end the session
  with 400 (`Context is too long`). The core would need a Soniox-specific size check at Start
  (`glossary::validate`), not a refusal from the provider.
- Soniox says a term is used only where it fits the sentence, and recommends writing terms in the
  language people speak them in.
- Context is billed as input text tokens at $4.00 per million: a full 8,000-token context costs
  $0.032 per stream started, a typical list a fraction of a cent. Every reconnect starts a stream
  and sends it again.
- A talk title or programme could later go in `general` or `text`; the app has no field for that.

## Languages

60 languages, the same list for transcription and translation, with translation between any two
(Soniox counts 3,600+ pairs) in one model. Against `src/lib/languages.json` (80 rows):

- **57 codes match catalog rows exactly.** Soniox's `tl` (Tagalog) is the catalog's `fil`.
- **Variants:** Soniox has one `zh` and one `pt`. It does not say which Chinese script or which
  Portuguese it writes, so `zh-Hans`, `zh-Hant`, `pt-BR` and `pt-PT` would each need a decision or
  a probe; offering only `zh-Hans` and `pt-BR` after a check is the cautious start. The catalog's
  plain `pt` and `zh` rows are OpenAI's.
- **Not in the catalog:** Bosnian (`bs`) and Welsh (`cy`).
- **Catalog rows Soniox lacks (18, all Gemini's):** Akan, Amharic, Armenian, Burmese, Georgian,
  Hausa, Icelandic, Javanese, Khmer, Kinyarwanda, Lao, Mongolian, Nepali, Sindhi, Sinhala,
  Sundanese, Uzbek, Zulu.
- **Every one of DeepL's 42 catalog rows** is in Soniox's list, Chinese and Portuguese without
  the script or region choice above.

Spoken language is detected automatically, mid-sentence switches included; `language_hints`
biases it without restricting it. The `/models` endpoint returns the list programmatically.

## Cost

Soniox bills real-time streams by token
([pricing](https://soniox.com/pricing), read 23 November 2026):

| Token | Real-time price | Per hour |
| --- | --- | --- |
| Input audio, for the **whole stream duration**, silence included | $2.00 / M | about 30,000 tokens: $0.06 |
| Output text, the transcript | $4.00 / M | about 15,000 tokens per hour of speech: $0.06 |
| Output text, the translation | $4.00 / M | about as many again: $0.06 |
| Input text, the context | $4.00 / M | once per stream, at most $0.032 |

So about **$0.12 an hour** for transcription and **$0.18 an hour** with translation, per stream.
Both are estimates: output tokens follow how much is said, and the translation's token count
follows the target language. A silent room still bills the audio tokens ($0.06 an hour), as
manual finalization's page also warns; Pause closes the stream, so a paused session bills
nothing. Diarization and language identification cost nothing extra.

Per hour of audio, one stream (one source, one caption language):

| Engine | Mode | In the app (`providers.ts`) | Provider's page, read 23 November 2026 |
| --- | --- | --- | --- |
| **Soniox `stt-rt-v6`** | Translate | — | ≈ $0.18 (token estimate above) |
| **Soniox `stt-rt-v6`** | Transcribe | — | ≈ $0.12 |
| Gemini Live Translate | Translate | $1.25–2.21 | $0.0368 a minute = $2.21 ([pricing](https://aistudio.google.com/docs/pricing)) |
| OpenAI `gpt-realtime-translate` | Translate | $3.06 | $0.034 a minute = **$2.04** ([pricing](https://developers.openai.com/api/docs/pricing)) |
| DeepL Voice | Translate | ≈ $1.85 | not public; third-party figure ([DeepL notes](deepl-voice-api.md)) |
| Gemini Transcribe Live | Transcribe | $0.30–0.54 | about $0.009 a minute = $0.54 ([pricing](https://aistudio.google.com/docs/pricing)) |
| Mistral Voxtral Realtime 3 | Transcribe | $0.48 | $0.008 a minute ([Mistral notes](mistral-realtime-api.md#scope), 21 October) |

For a six-hour conference day with both sources and two caption languages (four streams):
Soniox about $4.30, DeepL about $44, OpenAI about $49, Gemini Live Translate about $53. With one
`two_way` session per source serving both languages, Soniox would be about $2.20.

Side finding, outside this issue: OpenAI's pricing page now lists `gpt-realtime-translate` at
$0.034 a minute ($2.04 an hour), against the $3.06 in `providers.ts`. Worth its own check.

## Privacy facts for `docs/privacy.md`

- **Who:** Soniox Inc., 1045 Helm Lane, Foster City, California 94404, United States. Its
  Terms are governed by Californian law, with San Francisco courts.
- **Where:** by default the United States (`stt-rt.soniox.com`). A Soniox *project* can instead be
  created in the European Union (`stt-rt.eu.soniox.com`), Japan (`stt-rt.jp.soniox.com`) or India
  (`stt-rt.in.soniox.com`); its audio and transcripts are then processed and stored in that
  region, while account, usage and billing data may be processed elsewhere. A region is fixed
  when the project is created, needs Soniox's approval (requested in the Console, with funds in
  the account), and its key must be used with its region's host. The app would therefore need a
  region choice, or at least an EU option, which belongs with the provider settings of
  [#249](https://github.com/fmadore/Live-translation/issues/249). What a key sent to the wrong
  region's host gets is not documented.
- **Retention:** real-time requests are "processed transiently" and not stored. Logs hold
  operational metadata only (request and project ids, durations, token counts, errors), never
  audio, transcripts, translations or context. Stored content exists only for the async API (30
  days), which the app would not use.
- **Training:** Soniox does not use customer content, context and glossary terms included, to
  train, fine-tune, evaluate or improve its models.
- **Role:** for content processed on a customer's behalf Soniox acts as a processor; the user is
  the customer, under their own account and key, as with every engine.
- **Claims:** SOC 2 Type 2, ISO/IEC 27001:2022, GDPR and HIPAA compliance; documents in the
  Console.
- **Accounts:** for business users and people of at least 18.

A paragraph for `docs/privacy.md`, in the shape of DeepL's, if Soniox is adopted:

> With **Soniox**, the audio of the chosen sources is streamed, while a Soniox session runs, to
> Soniox Inc. (Foster City, California, United States), or to the region the user chose for
> their Soniox project, together with the chosen caption language and the names and terms entered
> in the setup, if any; Soniox returns the transcript of what was said and its translation.
> Soniox states that it does not store real-time audio or text and does not train on it. The
> Soniox key is stored in Windows Credential Manager or the macOS Keychain like the others and
> sent only to Soniox.

It would also need the Soniox privacy link in the providers' list, a new effective date, the
sentence that translation engines are not sent the names and terms changed, and the consent
prompt naming Soniox. The difference from DeepL is the one that matters for a European user: a
US company processing in the US unless the user sets up an EU project.

## What a fifth engine costs

The issue's point stands: it is not one Rust module. Adding Soniox means:

- a `Provider` variant (serde `soniox`) in `types.rs` and `types.ts`, with the contract test;
- `src-tauri/src/soniox/` (client, protocol types, tests with recorded messages and a loopback
  server, an ignored live probe), and the binary-frame change in the runner;
- a `soniox-api-key` credential in `secrets.rs`, a key URL and rate in `providers.ts`;
- a `soniox` source and memberships in `languages.json`, regenerated;
- the engine card, its note and its errors in `en.ts`, `fr.ts` and `de.ts`, and the Mac overrides
  if any wording differs;
- the region choice if it is offered;
- `docs/privacy.md` with a new effective date, `docs/store-listing-next.md` in three languages,
  the README and architecture tables;
- one more engine card in a picker that already shows Gemini, OpenAI, DeepL, Mistral, Gemini
  Transcribe and Whisper.

## Manual quality protocol

For the maintainer, with a Soniox key; it bills the key, about $0.10 in all on Soniox, plus a
few cents on Gemini and Mistral. Soniox's terms allow internal evaluation for choosing a vendor;
they forbid using outputs to build datasets or benchmarks, so keep the results as this decision's
record. Do not put the key in the repository, a screenshot or a log.

### 1. Setup

1. Create a Soniox account and a project (United States is the default; an EU project needs an
   approved request first). Add funds, create an API key, and set a monthly limit of a few
   dollars on the project.
2. `pip install websockets`, and save the script below outside the repository as
   `soniox_probe.py`. It streams a 16 kHz mono WAV in 100 ms binary chunks at real time, as the
   app does, prints every final token with its time since the start request and its
   `translation_status`, and writes every response to a JSON Lines log.
3. `export SONIOX_API_KEY=...` in the shell (for an EU project, also
   `export SONIOX_URL=wss://stt-rt.eu.soniox.com/transcribe-websocket`).

```python
# soniox_probe.py: issue #250's manual check. Bills the key in SONIOX_API_KEY.
# python soniox_probe.py <file.wav> <none|one_way|two_way> <target> <log.jsonl> [term ...]
import asyncio, json, os, sys, time, wave
import websockets

URL = os.environ.get("SONIOX_URL", "wss://stt-rt.soniox.com/transcribe-websocket")

def start_request(mode, target, terms):
    config = {"model": "stt-rt-v6", "audio_format": "pcm_s16le", "sample_rate": 16000,
              "num_channels": 1, "enable_endpoint_detection": True}
    if terms:
        config["context"] = {"terms": terms,
                             "translation_terms": [{"source": t, "target": t} for t in terms]}
    if mode == "one_way":
        config["translation"] = {"type": "one_way", "target_language": target}
    elif mode == "two_way":
        config["translation"] = {"type": "two_way", "language_a": "fr", "language_b": "en"}
    return config

async def main(path, mode, target, log_path, terms):
    audio = wave.open(path)
    assert (audio.getnchannels(), audio.getsampwidth(), audio.getframerate()) == (1, 2, 16000)
    headers = {"Authorization": f"Bearer {os.environ['SONIOX_API_KEY']}"}
    async with websockets.connect(URL, additional_headers=headers) as ws:
        await ws.send(json.dumps(start_request(mode, target, terms)))
        t0 = time.monotonic()

        async def send_audio():
            while chunk := audio.readframes(1600):  # 100 ms
                await ws.send(chunk)  # bytes: a binary frame
                await asyncio.sleep(0.1)
            await ws.send("")  # empty text frame: end of stream

        sender = asyncio.create_task(send_audio())
        with open(log_path, "w", encoding="utf-8") as log:
            async for message in ws:
                t = time.monotonic() - t0
                response = json.loads(message)
                log.write(json.dumps({"t": round(t, 3), **response}, ensure_ascii=False) + "\n")
                for token in response.get("tokens", []):
                    if token.get("is_final"):
                        status = token.get("translation_status", "-")
                        print(f"{t:7.2f} {status:11} {token.get('language', ''):3} {token['text']!r}")
                if response.get("error_code") or response.get("finished"):
                    print(f"{t:7.2f}", response)
                    break
        sender.cancel()

asyncio.run(main(*sys.argv[1:5], sys.argv[5:]))
```

### 2. Protocol checks on the rehearsal recordings

`src-tauri/resources/fixtures/rehearsal-fr.wav` (23.2 s) and `rehearsal-en.wav` (20.8 s) are
16 kHz mono PCM16, Windows text-to-speech. They test the protocol and timing, not accuracy:
Mistral returned both word for word at every setting. For the alternating test, join them:

```bash
ffmpeg -i rehearsal-fr.wav -i rehearsal-en.wav -i rehearsal-fr.wav \
  -filter_complex concat=n=3:v=0:a=1 -ar 16000 -ac 1 -sample_fmt s16 alternating.wav
```

| Run | Command (after `python soniox_probe.py`) | What to record |
| --- | --- | --- |
| P1 | `rehearsal-fr.wav one_way en p1.jsonl` | Time of the first `original` and first `translation` token; for each sentence, the lag from its last final `original` token to its last final `translation` token; where `<end>` falls against the translation; whether any translation token is ever non-final (in the log). |
| P2 | `rehearsal-en.wav one_way fr p2.jsonl` | The same, other direction. |
| P3 | `rehearsal-en.wav one_way en p3.jsonl` | Speech already in the target: `none` tokens, `translation` tokens that repeat the speech, or nothing. Decides the [open question](#turns-interims-and-finals). |
| P4 | `rehearsal-fr.wav none - p4.jsonl` | Transcription alone: words against the script, `<end>` count. |
| P5 | `alternating.wav two_way - p5.jsonl` | For #12: each sentence's `original` language and its translation's; any mislabelled sentence; the lag at each switch. |
| P6 | `rehearsal-fr.wav one_way de p6.jsonl "Live Translation"` | A third language with a context: accepted, and the term kept. |

Then the comparison on the same recordings, in the app (Rehearse) and with the existing probes:

- **Gemini Live Translate:** Rehearse with caption language English (it plays the French
  recording) and then French (it plays the English one). Export each transcript. Note when the
  first caption appeared and how the lines broke.
- **Mistral (subtitles):** `cargo test --manifest-path src-tauri/Cargo.toml --lib
  realtime_3_closes -- --ignored --nocapture` (French recording through the real runner); compare
  its lines and their timing with P4.
- **DeepL**, only if a paid DeepL key is at hand: its `live_probe` (see the
  [DeepL notes](deepl-voice-api.md#live-probe)) against P2's direction.

### 3. Quality on real speech

The decision rests here, because the rehearsal voices are too clean to rank engines.

1. Take two excerpts of about five minutes from recorded conference talks the maintainer may use:
   one in French, one in English, each with several proper names (speakers, places,
   institutions). Convert each:
   `ffmpeg -i talk.mp4 -ss 00:10:00 -t 300 -ac 1 -ar 16000 -sample_fmt s16 talk-fr.wav`.
2. Write the names down as the glossary, and correct a reference transcript of the first two
   minutes of each by hand.
3. Soniox, four runs per excerpt: `one_way` into the other language without terms and with them;
   `none` without terms and with them.
4. Gemini Live Translate and Mistral (subtitles) in the app, on the same excerpts: play each file
   in a media player, capture that application, Start, then export the transcript. Give Mistral
   the same names and terms. Run DeepL too if a paid key is available.
5. Score, per engine and direction:
   - **Translation:** read the translation sentence by sentence against the reference and mark
     each correct, minor error, wrong or missing; report the share correct.
   - **Names:** of the listed names spoken, how many are spelled right in the transcript and kept
     right in the translation, with and without the glossary.
   - **Transcription** (Soniox `none` against Mistral): word errors over the two corrected
     minutes.
   - **Latency:** from Soniox's log, the median and worst lag from the end of a sentence's spoken
     tokens to its translation; for the app, the time from a sentence's end to its caption,
     timed on a screen recording.
   - **Lines:** lines per minute, and how many end mid-sentence.
6. Check the Soniox Console's usage log for one run: the audio and output token counts confirm or
   correct the $0.12 and $0.18 an hour estimates.

### 4. Pass criteria

Soniox goes ahead as an experimental engine if all of these hold:

- translations correct on at least as many sentences as Gemini's, within five points, in both
  directions;
- with the glossary, more names right than without it, and at least as many as Gemini;
- a median translation lag under 2.5 s, and none of the protocol's questions answered in a way the
  mapping above cannot take;
- for #12, P5 labels every sentence's language correctly (recorded, but not part of the gate for
  the first version).

Record the results in a dated section of this file, as
[Mistral's live check](mistral-realtime-api.md#live-check-21-october-2026) does.

## What is unverified

- Everything about the live stream: response cadence, whether translation tokens are ever
  non-final, where `<end>` falls against the translation, the translation lag.
- What `one_way` returns for speech already in the target language.
- Base64 audio in text frames (named only in an error message).
- Whether Soniox answers WebSocket protocol pings, and whether they count against its idle
  timeout (the keepalive silence makes the second moot).
- Which Chinese script and which Portuguese `zh` and `pt` produce.
- That `fr`/`en` is an accepted `two_way` pair.
- What a key used against the wrong region's host gets.
- Whether Soniox gives new accounts free credit (not stated on the pages read).
- The data processing agreement and subprocessors, behind the Console's sign-in.
- How the runner's handover replay (about a second of audio in a burst) sits with Soniox's
  real-time pacing rule.

## Recommendation

**Wait for the manual protocol, then adopt Soniox as an experimental translation engine if it
passes; do not write code before.** Cheapness alone would not justify a fifth engine, but Soniox
brings two things no current engine has: names and terms that reach the translation
(`terms` and `translation_terms`), and in `two_way` a provider-side answer to #12's bilingual
Q&A. It also covers DeepL's 42 catalog languages at about a tenth of every translation engine's
price, holds a stream for five hours instead of Gemini's ten minutes, and its protocol maps onto
the runner with one small change (binary audio frames), mostly reusing DeepL's
concluded-and-tentative turn rules. Against it: translation quality is entirely unmeasured, it is
a US company processing in the US unless the user obtains an EU project, it has shipped four
model generations in thirteen months (v3 to v6, the old names routed to the new models), and it
adds a fifth engine's interface, catalogue, Store and privacy cost.

The first version should be translation only (`one_way`, one session per caption language),
with the glossary, labelled experimental like DeepL. Subtitles, `two_way` for #12, and one
session feeding two lanes (with #247) come after it has been used live. If Soniox passes and
DeepL still has never run live, consider whether Soniox should take DeepL's experimental place
rather than sit beside it: that keeps four cloud translation engines, though DeepL's processing
in Germany is the better privacy story for European users. If the protocol fails on quality,
reject Soniox and close #250 with the results recorded here.
