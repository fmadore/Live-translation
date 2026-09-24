# Transcript export — issue #26

Included in **1.2.3**. The user confirmed native Save As on the ARM64 test MSIX on
8 September 2026. The detailed acceptance matrix below remains partially unverified.

Choose a format in the transcript toolbar and press **Save as…** (**Enregistrer sous…**).
Windows opens its native Save As dialog, owned by the operator window. Select any writable
folder and filename. The format is chosen in the toolbar before opening the dialog.
Windows supplies overwrite confirmation and the default file extension. The app remembers
the folder after a successful save, including across restarts; a missing folder falls back
to Documents. No filesystem capability is granted to the overlay.

Cancelling leaves the transcript, saved marker and recovery copy unchanged. Cancelling
Save As from the quit prompt keeps the application open. A successful export writes a
snapshot of finalized captions; captions arriving while the dialog is open remain unsaved
and eligible for recovery. Writes use a flushed temporary file in the chosen directory,
followed by replacement, so a failed write does not truncate an existing export.

Markdown and plain text retain paragraph grouping and localized source labels. WebVTT uses
voice tags, and SRT prefixes each cue with its localized source name. Both retain all
finalized text, order cues by start time, permit simultaneous microphone/system cues,
and support durations over 24 hours. One-frame turns receive a minimum 1 ms interval.
Timing comes from the core's monotonic session clock, not wall-clock timestamps or an
audio recording. Provider latency is included; these are caption timings, not forced
alignment with speech. Reconnects share the session clock. Separate sessions retained in
one transcript are concatenated after its last cue; idle time between sessions is omitted.
Older recovery files without timing remain exportable as Markdown/text, with timed export
disabled and an explanation rather than guessed timestamps or silently omitted text.

## Bilingual and two-language exports

Translation keeps what was said before it was translated. When a transcript holds any such
original speech, **Include original speech** appears beside the format selector (and in the
history view); the quit prompt's save follows the same choice. Text puts `Original: …` under
each paragraph, Markdown a `> *Original:* …` quote, and WebVTT and SubRip set the original as
each cue's italic second line (`<i>…</i>`). Subtitles have no separate original, so the option
is not offered for them. It is off by default.

A session captioned in two languages records each line's language. Its paragraphs are grouped
per language rather than alternating line by line; text and Markdown export one section per
language, headed by its name; WebVTT and SubRip interleave both languages' cues by start time,
each labelled with its source and language, which most players show as stacked bilingual
subtitles. The original speech, when included, is written once, under the first language. A
transcript in one language exports exactly as before.

## Verification

Automated tests cover cancelled saves and quit, captions arriving during a save, atomic
replacement and failed replacement cleanup, format selection, untimed recovery, source
overlap, partial replacement, session restart, escaping, and long-duration timestamps.

Native acceptance checks remain manual, separately in development, NSIS and Store MSIX:

- Save each format to a chosen folder, including a path with accents and spaces.
- Reopen Save As and restart the app; verify that the last successful folder is selected.
- Choose an existing filename; decline replacement, then accept it on a second attempt.
- Cancel from both the toolbar and the quit prompt; verify no write and no app exit.
- Try an unwritable destination; verify the prior file and unsaved status survive.
- Open exported SRT/VTT in a subtitle player and check both-source captions.

Issue #26 is closed at the user’s request for release 1.2.3. Closure records implementation
and the reported ARM64 test; it does not certify every packaging scenario above.

## Titles, cleanup and reading controls in 1.4.0

Saved session titles and history search help find a session; they do not rewrite its exported
caption text or change the date-based export filename. Reading pace, hold duration and the
optional local filler filter affect only presentation. Raw transcript/export text means the
text returned by the provider: Gemini Smart transcription can already have removed fillers
and corrected false starts before the app receives a final caption. See
[history titles/search](transcript-history.md#titles-and-search) and [reading controls](caption-layout.md#reading-pace-and-preview).
