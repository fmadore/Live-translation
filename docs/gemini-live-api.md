# Gemini Live API

Two Gemini models share one `BidiGenerateContent` socket and one API key. For subtitles without a
cloud connection, see [local Whisper](local-whisper.md).

| Model | Status | Mode it serves |
| --- | --- | --- |
| `gemini-3.5-live-translate-preview` | Preview | Translated captions |
| `gemini-3.5-transcribe-live` | Stable | Same-language subtitles |

Re-verified 27 August 2026 against Google’s
[Live translation guide](https://ai.google.dev/gemini-api/docs/live-api/live-translate)
(last updated 2026-07-23), the
[Live transcription guide](https://ai.google.dev/gemini-api/docs/live-api/live-transcribe)
(last updated 2026-08-26), the [model list](https://ai.google.dev/gemini-api/docs/models) and
the [Live API reference](https://ai.google.dev/api/live). Live Translate is still preview with
no stable equivalent (Transcribe went GA), so re-check it before an event; its wire format is
unchanged since the 12 August 2026 verification. Manual activity detection, which Transcribe
Live now uses (see [cutting at real pauses](#cutting-at-real-pauses)), was checked against the
reference and the transcription guide on 28 November 2026.

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

The SDKs flatten both levels into one `LiveConnectConfig`, but on the wire they are distinct:

- `inputAudioTranscription` and `outputAudioTranscription` are fields of
  `BidiGenerateContentSetup` itself ([Live API reference](https://ai.google.dev/api/live)).
  Nested under `generationConfig`, they make the server reject the whole session with
  `Unknown name "inputAudioTranscription" at 'setup.generation_config': Cannot find field.`
- `translationConfig` *is* a `generationConfig` field, and is Gemini Developer API only —
  the SDKs raise on it in Enterprise/Vertex mode.

Both placements are pinned by `python-genai`'s converter tests (`tests/live/test_live.py`:
`test_bidi_setup_to_api_with_input_transcription`,
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
- `goAway` triggers an immediate planned reconnect; see [the ten-minute cap](#the-ten-minute-cap).
- A provider `error` becomes a terminal operator-visible status instead of a silent log.

### Closing

Pause and Stop send `{"realtimeInput":{"audioStreamEnd":true}}`, then drain until the
`turnComplete` that follows it (at most four seconds) before closing — unless no turn is open
and nothing has been captioned for three seconds, when they close at once. Closing at once, as the
client did through 1.6.1, discards whatever translation of the last one to three seconds of
speech is still in flight. **Live check pending:** the reference documents `audioStreamEnd` for
the Live API in general, and no probe has yet shown how Live Translate answers it. The client
is safe either way: an `error` or a close during the drain ends it quietly — no error status —
and the turn accumulated so far is still finalized. Things to confirm on the first live run:
the drain ends well before four seconds on Stop mid-sentence, the last words are translated,
and nothing is logged as `provider error while closing`.

## Gemini 3.5 Transcribe Live

Same endpoint, audio frame and key; a subtitle engine rather than a translator, so the setup
message and transcription fields differ.

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
      "mode": "VERBATIM"
    },
    "realtimeInputConfig": {
      "automaticActivityDetection": { "disabled": true }
    }
  }
}
```

`realtimeInputConfig` is sent when the client cuts the stream itself, the default since issue
#281; without Silero it is left out and Gemini's own detection segments the speech. See
[cutting at real pauses](#cutting-at-real-pauses).

- `responseModalities: ["TEXT"]` separates the transcription pipeline from a live agent: no
  generated audio to discard, no output sidecar to read.
- `inputAudioTranscription` is a `BidiGenerateContentSetup` field here too, and carries
  configuration rather than being an empty marker.
- `languageCodes: []` enables automatic language identification across utterances, including
  code-switching — the right default for a bilingual room. The guide's table lists 84 entries:
  **83 distinct BCP-47 codes and 77 distinct languages** once locale variants collapse
  (`en-US`/`en-GB`/`en-IN`, `bn-BD`/`bn-IN`, …). User-facing copy therefore says "over 70
  languages", not "100+".
- A chosen spoken language goes in `languageCodes` instead; see
  [the spoken-language hint](#the-spoken-language-hint).
- `mode: "VERBATIM"`, Google's default, returns a literal transcript, one final per speech
  segment, punctuated and cased, fillers and false starts included; the overlay's
  [Hide filler words](caption-layout.md#hide-filler-words) filter takes the fillers out on
  screen. Through 1.15.0 the app sent `SMART`, which cleans the text up but is a dictation
  mode; see [why VERBATIM](#why-verbatim-not-smart). Google's Gemini 3.8 Live models
  ([15 September 2026 announcement](https://blog.google/innovation-and-ai/technology/developers-tools/build-real-time-voice-applications-gemini-audio/))
  are conversational audio models, not replacements for this integration.
- `customVocabulary` carries the setup's **Names and terms** list, inside
  `inputAudioTranscription` beside `languageCodes`, and is left out when the list is empty.
  Google documents up to 1,000 terms with the best results at up to 100; the app's list holds
  100 (see [the glossary limits](mistral-realtime-api.md#glossary)). The guide says nothing
  about changing it mid-session, so the app sends it once, in the setup message.
  `transcribe::live_probe` sends two terms, which proves the server accepts the field where the
  client puts it, not how much it helps. Live Translate documents no such field ("no support
  for tools or instructions"), so translated captions take no list.

### The spoken-language hint

When a start request carries a spoken language (`StartOptions.spokenLanguage`, issue #249), the
core sends it as `languageCodes`, to steady recognition of short sentences; with none it sends
`[]` and the model detects the language. The
[guide](https://ai.google.dev/gemini-api/docs/live-api/live-transcribe) (read 23 November 2026)
asks for BCP-47 codes "to bias recognition toward specific languages" and lists them by locale
(`fr-FR`, `cmn-Hans-CN`), while the app's spoken-language list (`src/lib/whisperLanguages.json`)
names languages as Whisper does (`fr`, `zh`). `gemini::transcribe::language_codes` maps one onto
the other:

- Only codes from Google's table are sent. A language the table lists under several locales
  gets all of them: `en` → `en-US`, `en-GB`, `en-IN`; `es` → `es-419`, `es-US` (the guide's
  example `es-ES` is not in its table); `pt` → `pt-BR`, `pt-PT`; `bn` and `pa` likewise.
- Whisper's `jw` is Javanese, `jv-ID`; Tagalog (`tl`) is sent as Filipino, `fil-PH`; Norwegian
  (`no`) is Bokmål, `nb-NO`, and Nynorsk has no row; Arabic has one row, Egyptian `ar-EG`.
- 71 of Whisper's 99 languages map. A start with one of the other 28 (Tamil, Urdu, Yoruba,
  Welsh, …) is refused rather than quietly left to automatic detection. Six table rows have no
  Whisper language: Aromanian, Cantonese, Cebuano, Kabuverdianu, Kyrgyz and Oriya.
- `spoken_languages_map_onto_googles_table` pins the table and the mapping, and checks that the
  interface's list (`src/lib/geminiTranscribeLanguages.json`) holds the same 71.

Step 03 of the setup shows Whisper's spoken-language picker for Gemini Transcribe too, with
automatic detection first and only those 71 languages. Both engines share one choice: a language
Whisper knows and Gemini does not stays saved for Whisper, the Gemini picker shows automatic
detection, and that is what the run sends (`spokenLanguageFor` in `providers.ts`, used by the
picker, the running rail and `sessionController.ts`).

**Live check, 23 November 2026** (`transcribe::live_probe`):

- `every_spoken_language_hint_is_accepted` set up a session, without audio, for each of the 71
  code lists: every one was accepted. So were `fr`, `es-ES` and `xx-XX`, which the client never
  sends: **the server does not check the codes at setup**, so acceptance proves the field's
  place, not the codes' validity.
- `the_french_recording_under_a_right_and_a_wrong_hint` streamed the French rehearsal recording
  under `fr-FR`, then under the English codes. Both came back as the same French text, word for
  word; the English hint neither translated it nor garbled it. On twenty seconds of clear speech
  the hint is a bias, not a constraint. What a wrong hint does to a short sentence, where it
  should matter most, was not tried. Both runs returned the whole recording as one finalized
  segment.

**Live check pending:** a real session with the hint, including a short sentence under a wrong
one, before it is advertised.

### Streaming and events

The audio frame is the one live translate uses — 16 kHz mono PCM16 LE, base64 in
`realtimeInput.audio`, 100 ms chunks.

- `serverContent.interimInputTranscription.text` is a speculative partial hypothesis, revised
  as the speaker talks.
- `serverContent.inputTranscription.text` is the finalized, authoritative transcript for one
  speech segment, emitted when the speaker pauses.
- `voiceActivity` (`ACTIVITY_START`, `ACTIVITY_END`), beside an empty `serverContent`, brackets
  each segment. The guide does not mention it.

**Both describe the same segment, so each replaces the caption buffer rather than extending
it.** Live translate appends `inputTranscription` deltas; doing so here would repeat every
revised hypothesis on screen. One finalized segment becomes one transcript line.

### A segment can end without a final

On 23 November 2026, in both VERBATIM runs of the three-passage stream below, the second segment
of the English passage ended (`ACTIVITY_END`) with no `inputTranscription` at all. Its interim
text stayed pending, the next segment's first interim replaced it, and the words were lost. The
client now makes that interim text the segment's caption on `ACTIVITY_END`. A final that still
arrives for it, before the next `ACTIVITY_START` or interim, is not captioned a second time.
`client_tests::verbatim_segments_are_captioned_one_by_one_and_none_is_lost` replays the
recording, and `a_late_final_for_a_stranded_segment_is_not_captioned_twice` covers the late
final, which has not been seen on the wire. When the final comes first, as it does for every
other segment, `ACTIVITY_END` finds nothing pending and does nothing.

### Why VERBATIM, not SMART

The [guide](https://ai.google.dev/gemini-api/docs/live-api/live-transcribe) says that in SMART
mode `input_transcription` "returns the cumulative, cleaned, and formatted transcript for the
session so far", and that SMART turns on voice editing: the server keeps the text and applies
spoken instructions ("delete that", "make it more professional") to all of it. Through 1.15.0
the client took each final for its own segment, so every caption after the first repeated the
session before it.

`live_probe::smart_finals_across_three_passages` showed it on 23 November 2026, streaming the
French recording, six seconds of silence, the English one, silence, and the French one again.
Four finals came back, each starting with the whole of the one before, and:

- **The repeated passage disappeared.** It was shown as interim text, then dropped from the
  final as a repetition.
- **Spoken edits would apply.** A speaker saying "delete that" edits the text instead of being
  captioned.
- **Finals came late, and later as the session went on:** about 15 seconds after the first
  passage ended and about 30 seconds after the audio ended, against about 2.5 and 8 seconds in
  VERBATIM on the same stream (both measured against the server's own audio offsets). Even the
  `voiceActivity` frames were held back.

The maintainer chose VERBATIM. Its finals are segments, so the client captions each as it comes
and keeps no memory of earlier ones: a segment that opens with the previous one's words keeps
them (`a_segment_sharing_its_first_word_with_the_last_keeps_it`).

**Live check, 23 November 2026** (`captions_across_three_passages_as_the_app_shows_them`, the
production client through the real session runner): five captions, in order: the French
passage; "This is a rehearsal recording for the live caption system."; the stranded segment,
from its interim text ("speaker is presenting to an audience … Captions appear a"); the rest of
the English passage; and the French passage again, whole. Each came within about a second of
its passage's audio, except the last, three seconds after the audio stopped. The stranded
caption has what its interim text had: it misses the segment's first word ("A") and stops at
"Captions appear a", so "moment after the words" reached no caption. Before the fix the whole
segment was lost.

When a message carries both, the finalized text wins. An empty final or a segment close with no
final retracts the speculative caption rather than keeping it as speech, and the front end
discards the pending empty turn (regression-tested). The
local [Hide filler words](caption-layout.md#hide-filler-words) filter is separate.

On close, the open activity's `{"realtimeInput":{"activityEnd":{}}}` ends the last segment
when the client cuts the stream; with Gemini's own detection,
`{"realtimeInput":{"audioStreamEnd":true}}` is sent instead (the reference says to send it
only with automatic detection). Either way a last segment can still arrive during the shared
runner's drain window. The drain ends on the authoritative final that
follows it, rather than always waiting out the four-second window. The server sends no
acknowledgement when there is nothing left to flush, so a close with no turn open and nothing
captioned for three seconds skips the wait altogether.

### What the wire does that the guide does not mention

Found with `live_probe` (below) against the real endpoint on 27 August 2026:

- **Replies arrive as WebSocket _binary_ frames carrying UTF-8 JSON.** The guide's browser
  sample hides the difference; a client matching only text frames has setup accepted and then
  hears nothing. `realtime/socket.rs` decodes `Message::Binary` — check this first if a session
  ever appears to hang.
- **`generationComplete` closes a segment**, not the `turnComplete` the translate path sends.
  It lets the client finalize a segment that produced only a speculative hypothesis.
- **`setupComplete` *is* sent**, although the guide's sample never waits for one. Waiting is
  correct and matches the Live API contract.

### Segmentation is the model's, and it is not sentence-shaped

On the bundled twenty-second recording, continuous speech runs two sentences into one segment
without a space (`…Use this recording toThe overlay before…`), breaks can fall mid-word
(`…every sentence is` / `described and shown on screen…`, from "transcribed"), and segmentation
varies between runs on identical audio. A caption therefore grows until the model closes the
segment — on continuous speech, a paragraph. The client does not re-split: that would invent
sentence boundaries and make text still being revised jump. Voxtral closes a line at a pause
after a sentence or at a detected turn end, which gives shorter lines: a fair reason to prefer
it for a speaker who rarely pauses.

**Continuous speech, 24 November 2026** (`every_frame_of_continuous_speech`: the English
recording twice with no silence between, as Rehearse loops it, every frame printed). Transcribe
Live closed a segment every 8 to 11 seconds whether or not the speaker paused, and the audio at
each cut reached neither segment's text: "every sentence is" / "scribed and shown", "Use this"
/ "The overlay" ("recording to check" lost), "words and are" / "sentence becomes clear"
("corrected once the" lost). In the app's own Rehearse, three runs on the same day showed the
same losses in the transcript ("every sentence is scribed", "Use this the overlay", "corrected
once the comes clear"), with or without the two mends below. They are the service's: no frame carries the missing words, and no final arrived
after its segment's `ACTIVITY_END`. Two things the client can mend and does:

- **A final shorter than its interim.** The last segment's interim read "sentence becomes clear.
  Use this recording to check the overlay before the room fills up." and its final only the
  part from "Use this". When a final of three words or more is exactly the end of its last
  interim (words compared without case or punctuation), the interim's opening words are kept
  (`keep_dropped_opening`); any other final wins as it is.
- **Sentences run together.** Interims and finals carried "as it is spoken.Captions appear".
  A space goes back after `.`, `!` or `?` that follows a word of two or more lower-case letters
  and precedes a capital (`space_sentences`), so "U.S.Army" and "3.5" stay as they are.

Two setups were tried on the same stream and changed nothing worth having:
`realtimeInputConfig.automaticActivityDetection` with `prefixPaddingMs: 500` and
`silenceDurationMs: 800` was accepted (the transcribe guide documents only `disabled`) and gave
the same cuts, one word lost at each ("transcribed" became "described"; "check" and "A" were
lost); with `disabled: true` and no `activityStart` from the client nothing at all was
transcribed. Cutting the stream at real pauses on the client, below, is the remaining way to
keep every word.

### Cutting at real pauses

Issue #281. The setup turns Gemini's automatic activity detection off and the client marks the
segments with `activityStart` and `activityEnd`, at pauses the bundled Silero model
([local Whisper's speech detection](local-whisper.md#speech-detection)) finds, so a cut falls in
silence rather than in a word. `gemini::cuts` holds it; `SEGMENTATION` there is the one switch
back to Gemini's segments, with no setting in the interface.

What Google documents (Live API reference and
[transcription guide](https://ai.google.dev/gemini-api/docs/live-api/live-transcribe), read
28 November 2026): `automaticActivityDetection.disabled` — "If disabled, the client must send
activity signals"; `activityStart` and `activityEnd` are empty objects in `realtimeInput` that
"can only be sent if automatic (i.e. server-side) activity detection is disabled", and the
guide's push-to-talk example for `gemini-3.5-transcribe-live` sends exactly these frames;
`audioStreamEnd` "should only be sent when automatic activity detection is enabled". The
client follows that to the letter. Nothing documented says whether audio outside an activity is
transcribed or billed (the pricing page bills Transcribe Live's input at 25 audio tokens a
second, nothing about silence), so the client leaves none outside one.

- **Every chunk is inside an activity.** The first chunk on a connection follows an
  `activityStart`; a cut is `activityEnd` then `activityStart`, between two chunks. Silero only
  places the cuts: a word it misses is still sent, inside a segment. A quiet room sends what it
  sent before, all of its audio, in one long activity that holds no speech and is never cut.
- **Where a cut goes.** After 480 ms of non-speech that follows at least 224 ms of speech
  (Silero's frames are 32 ms; a frame is speech from 0.5, quiet below 0.35). A speaker who
  never pauses is cut in a 160 ms gap between words once the segment has run 6 s from its first
  speech, in any quiet frame from 7.5 s, and at 8 s wherever the stream is: just under the 8 to
  11 s Gemini's own detection allowed, in case it keeps such a limit in manual mode. A cut goes
  only in silence Silero has already heard, except the 8 s one. There is no look-ahead, which
  would delay every caption. On the rehearsal recording the cuts land at its sentence ends, and
  once in a gap between words (`cuts::tests::silero_cuts_the_rehearsal_at_its_pauses`).
- **Where Silero runs.** On a thread of its own per source (two with Both), fed each chunk
  through a bounded queue as the chunk is sent; the answers are read before the next chunk.
  Each call re-reads 384 ms of earlier audio, because whisper.cpp starts the model's memory
  afresh each time. That cost about 21 ms of one core per second of audio per source in the
  Linux container this was built in (a release build); it has not been measured on a Windows
  PC or a Mac. Audio the thread cannot take counts as speech, so a detector that
  falls behind delays cuts and never makes one in speech. The thread ends with the client.
- **Pause, Stop, handover and reconnect.** Each close drain sends the open activity's
  `activityEnd` and waits for its final, as it waited after `audioStreamEnd`. A socket that
  fails takes its activity with it, and every new connection opens its own with its first
  chunk, after the audio a handover held. The loopback tests in `transcribe::cut_tests` pin
  each case.
- **When Silero cannot run** — the model is missing or will not load, or the processor lacks
  the instruction set ggml was built for (an x64 PC without AVX2, where local Whisper is
  refused) — the source starts with Gemini's own detection, as before, and the log says why.
- **Segment ends.** Whether `voiceActivity` frames and `generationComplete` still come in
  manual mode, and whether a segment's final comes before or after `ACTIVITY_END`, has not been
  seen. The client works either way: a final ends its segment whenever it comes; an
  `ACTIVITY_END` with no final makes the segment's interim text its caption only once the next
  segment shows itself (its `ACTIVITY_START`, its first interim, or a close with no final), or
  after the three-second idle boundary, instead of at once as with Gemini's segments.

**Live check pending.** No key was used to build this. Before a release, run the two probes
below with a key and read the "missing words" line: `continuous_speech_cut_by_the_client`
prints every frame both ways, and `captions_of_continuous_speech_cut_by_the_client` the captions
the app would show. If words are still lost at the cuts, or finals do not come, set
`SEGMENTATION` back to `Gemini`.

### The ten-minute cap

> Live transcription sessions support continuous streaming for up to 10 minutes.

Mistral Voxtral has no such limit, so this is the one operational difference between the two
subtitle engines: a room session reconnects several times an hour.

`goAway` arrives before the cut, and both Gemini clients answer it with
`MessageControl::Handover`: a planned move, not a failure. After a connection that lasted at
least 30 seconds the runner finalizes the in-flight turn, keeps the source Running and
reconnects at once with no backoff. While the new socket's TLS handshake and setup round trip
run it keeps reading the capture into a backlog of up to about three seconds, and sends it in
order once setup completes, so the gap shows up as a moment of extra latency rather than lost
words. A `goAway` straight after connecting still backs off, so a server refusing the session
is never hammered. Opening the new socket before closing the old one would remove even the
latency; the runner holds one socket per client today.

Speaker diarization and word-level timestamps are not available over the Live API; they belong
to the non-streaming `gemini-3.5-transcribe` model, which takes uploaded files and cannot serve
this app.

## Re-verifying against the live endpoint

Serialization tests pin both clients to the shapes above but cannot show the documentation is
right — and it is not everywhere: the translate guide still nests the transcription sidecars
under `generationConfig`. `gemini::transcribe::live_probe` sends the production setup, audio
frame and response types to Google with the bundled rehearsal recording and reports the
interim and finalized segments.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --lib the_documented_shape_is_the_shape_the_server_accepts -- --ignored --nocapture
```

It is `#[ignore]`d, so CI never runs it or bills anyone. Run it before an event and after any
change to either setup message; it costs a few cents (about twenty seconds of audio). The key is
the one saved in the app, read in process and never printed; failing that, `GEMINI_API_KEY` from
the environment or a `.env` at the repo root (`dotenvy` walks up from the crate directory; copy
`.env.example`). Its assertions concern the shape of the exchange, not the words: setup is
accepted, `generationComplete` arrives, and speculative updates outnumber finalized segments,
which is what makes replace-don't-append correct.

The same module holds the other probes above. `every_spoken_language_hint_is_accepted` sends no
audio; `the_french_recording_under_a_right_and_a_wrong_hint` streams forty seconds; the three
three-passage probes stream about eighty seconds each:
`captions_across_three_passages_as_the_app_shows_them` (the production client through the
session runner, the one to rerun), `verbatim_finals_across_three_passages` (every frame) and
`smart_finals_across_three_passages` (the old mode, for comparison).
`every_frame_of_continuous_speech` streams the English recording twice without a pause, about
fifty seconds, and prints every frame, interims included: the one to rerun for how segments
end in continuous speech. It and the other probes above use Gemini's own segmentation, for
comparison.

The client's cuts have two probes of their own, each streaming the English recording twice
without a pause, eight seconds of silence, and the recording again (about seventy seconds):

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --lib continuous_speech_cut_by_the_client -- --ignored --nocapture
cargo test --manifest-path src-tauri/Cargo.toml --lib captions_of_continuous_speech_cut_by_the_client -- --ignored --nocapture
```

The first prints every server frame and every `activityStart` and `activityEnd` the client
sends, then counts the `voiceActivity` frames and `generationComplete`s and prints the finals;
the second runs the production client through the session runner and prints its final
captions. Both end with the words of the script no final carried: `missing words: []` is the
result #281 is after.

## Target-language catalog

Rechecked against the [live translation guide](https://ai.google.dev/gemini-api/docs/live-api/live-translate)
on 2026-09-22 (page updated 2026-09-16): 78 languages / 79 codes. The app shows one Norwegian
row and sends `no`; `nb` is a search alias. The canonical catalog, with each code's provider
memberships, is `src/lib/languages.json`; generated Rust and TypeScript types share it. This is
documentation verification, not a live-speech pass; see [language coverage](language-coverage.md).
