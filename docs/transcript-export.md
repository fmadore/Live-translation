# Transcript export

Export your session transcript as plain text, Markdown, SubRip (SRT) or WebVTT, or copy recent
minutes to the clipboard while a meeting is still under way.

Choose a format in the transcript toolbar and click **Save as…** (**Enregistrer sous…**). The
chosen format carries across Start, Stop, restarts and the History view. Your system's native Save
As dialog opens, asks before overwriting an existing file, and adds the default file extension.
The app remembers the last successful export folder across restarts, falling back to Documents if
that folder no longer exists. The overlay window has no filesystem access.

Cancelling the dialog leaves the transcript, saved status and recovery copy untouched; cancelling
from the quit prompt keeps the app open. Saving from the quit prompt uses the toolbar's chosen
format, falling back to Markdown if WebVTT or SRT is selected on an older recovered transcript
with no cue timing. Each export writes a snapshot of finalised captions—anything arriving while
the dialog is open stays unsaved and eligible for recovery. Writes go to a flushed temporary file
in the target folder before replacing the destination, so a failed write never truncates an
existing file.

Markdown and plain text preserve paragraph grouping and localised source labels. WebVTT uses voice
tags, and SRT prefixes each cue with its localised source name. Both keep every finalised line,
order cues by start time, allow overlapping microphone and system cues, and support durations past
24 hours. Single-frame turns get a minimum 1 ms duration.

Cue timing comes from the app's monotonic session clock rather than wall-clock timestamps or an
audio recording. Cloud provider latency is included, so these are caption timings rather than
forced speech alignment. Whisper maps segment offsets back to capture timestamps so processing
delays do not drift cues; wait for Stop to finish Whisper's backlog before exporting a complete
transcript (discarding the backlog inserts an incomplete-transcript notice). Reconnects share the
session clock, and separate sessions kept in one transcript are appended after the previous last
cue without idle gaps. Older recovery files without timing remain exportable as Markdown or plain
text, with timed formats disabled and explained rather than given guessed timestamps.

## Reading back and copying during a session

While a session runs, the stage offers two tabs: **Live** (current turns with the transcript
underneath) and **Transcript**, which gives the full log the entire stage. Your choice is
remembered. The transcript view displays turns still being spoken at the bottom, plus the original
speech under each paragraph when **Include original speech** is ticked, so what you see matches
what Save and Copy write. New lines never jump the scroll position while you are scrolled up or
selecting text; click **Jump to latest** to return to the bottom, and press Ctrl+A inside the log
to select only the transcript.

**Copy** puts plain text on the clipboard—matching a plain-text export, with speaker labels and no
header—ready to paste into chat or notes mid-meeting. Choose the last 5, 10 or 30 minutes or the
whole transcript, including turns still being spoken. Captions that began before the selected
window and extend into it are copied whole. A recovered transcript without timing can only be
copied whole. Copying does not mark the transcript as saved.

Session titles, history search, reading pace, hold duration and the local filler filter never
alter exported text or the date-based filename. Raw export text is what the provider returned:
Gemini Transcribe returns it word for word, fillers and false starts included. See
[history titles and search](transcript-history.md#titles-and-search) and
[reading controls](caption-layout.md#reading-pace-and-preview).

## Bilingual and two-language exports

Translation keeps what was said before translating it. Whenever a transcript includes original
speech, **Include original speech** appears beside the format selector (and in History), and the
quit prompt follows the same choice. Plain text adds `Original: …` beneath each paragraph,
Markdown uses a `> *Original:* …` blockquote, and WebVTT and SRT put the original on an italic
second line (`<i>…</i>`). Subtitles have no separate original, so the toggle is hidden. It is off
by default.

Two-language sessions tag every line with its language. Paragraphs are grouped by language rather
than alternating line by line: plain text and Markdown export one headed section per language,
while WebVTT and SRT interleave cues by start time with source and language labels so subtitle
players stack them cleanly. Original speech, when included, appears once under the first language.
Single-language transcripts export as usual.

## Markers

Transcript markers (**Add marker**, see
[meeting controls](usability.md#markers-and-find-in-the-transcript)) store an optional label and
the exact session timestamp at which you added them. Because they sit beside the caption lines
rather than inside them, timed exports, the overlay and Zoom sharing never mistake a marker for
speech, and each marker cleanly ends the paragraph before it.

- **Plain text:** a divider line between paragraphs, `--- Talk 2 starts · 0:42:10 ---`.
- **Markdown:** a heading, `## Talk 2 starts · 0:42:10` (or `###` under each language heading in
  a two-language export).
- **WebVTT:** a `NOTE 0:42:10 Talk 2 starts` comment block before the first cue at or after that
  moment (a marker-only document contains just these comments). Players ignore comments, cue
  numbering is unaffected, and any `-->` inside the label is escaped to `->`.
- **SubRip:** omitted, because SRT has no comment syntax and a marker cue would appear as a
  subtitle.

Blank markers are numbered automatically in the interface language (**Section 2**,
**Abschnitt 2**). Copy includes markers inside its time window, and History Copy and Save format
markers identically.

## Verification

Automated tests cover cancelled saves and quits, captions arriving while the dialog is open,
atomic replacement and failed-write clean-up, format persistence, untimed recovery, overlapping
sources, session restarts, escaping and durations past 24 hours.

Native manual checks (issue #26, introduced in 1.2.3) remain part of release verification for
development, NSIS and Store MSIX builds:

- Save each format to a chosen folder, including paths with accents and spaces.
- Reopen Save As and restart the app; confirm the last successful folder is remembered.
- Pick an existing filename, decline overwriting, then overwrite on a second attempt.
- Cancel from both the toolbar and the quit prompt; confirm nothing is written and the app stays
  open.
- Try an unwritable destination; confirm the previous file and unsaved indicator survive.
- Open exported SRT and WebVTT files in a subtitle player and check two-source captions.
