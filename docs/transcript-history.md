# Transcript history

Implements [issue #81](https://github.com/fmadore/Live-translation/issues/81).

The operator window has a **Transcript history** section below the session controls.
Enable **Automatically save sessions locally** before starting a meeting. This is off by
default. Enabling it mid-session saves subsequent finalized lines; it does not retroactively
archive the transcript already in memory.

Each Start creates a separate session. Finalized lines are progressively saved to a flushed,
atomically replaced JSON file in
`%LOCALAPPDATA%\io.github.fmadore.live-translation\history\<session-id>.json`.
Stop drains and saves trailing partial text before finishing the session. A crash leaves the
last successfully saved snapshot available. Empty sessions create no file.

**Browse sessions** lists start date/time, duration, and known language information. Source
speech is labelled auto-detected when the provider does not report its language. Demonstration
and rehearsal sources have known languages. Unfinished sessions show duration through the last
save. Select a session to read its raw captions and available source transcription, copy it, or
export Markdown, plain text, WebVTT or SRT through the existing Save As dialog. Timed formats
require valid cue timing. Browsing and exporting history never replaces the active transcript.

History remains after clearing the active transcript, export, quitting, and disabling automatic
history. Use **Delete**, then **Delete permanently**, to remove a session. Deleting the current
session prevents further history writes for that session; the next Start creates a new one.
Unreadable records are shown individually and remain deletable. Save failures stay visible with
a retry action; pending failed sessions are retained in memory until retried or the app exits.

There is no automatic expiry and no upload. Files are unencrypted local text. No audio, keys,
device names or application identities are saved. History and the optional single recovery
copy are independent. See [privacy](privacy.md).

## Verification

Automated coverage includes opt-in behavior, progressive writes, raw text preservation,
separate session timelines, trailing partials, duration finalization, restart/readback, atomic
replacement, interrupted writes, write failures and retry, corruption, deletion races, path
validation, operator-only command permissions, and the history UI's copy/export/delete actions.

Native release acceptance should also exercise the built-in demonstration with history enabled,
restart the packaged app, reopen two sessions, export and delete one, and confirm the other is
still available. Abruptly end a disposable demo run and check its last saved snapshot. No paid
provider call is needed for these checks.

## Titles and search

New in **1.4.0 (in preparation)**.

Saved sessions can be renamed (up to 120 characters); a blank title restores the date label.
Titles are stored inside the existing local session JSON, including when the active session
continues receiving captions. Older files without titles remain readable. Changing a title
never replaces or edits caption text.

Search matches session titles, caption text and available source text. Date bounds include
both selected local calendar days. The caption-language filter uses the translation target,
or the known source language for subtitles; automatically detected languages remain marked
unknown rather than guessed. Filters do not delete sessions or change export contents.
