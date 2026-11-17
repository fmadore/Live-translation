# Live Translation & Subtitles 1.15.0

Tools for the person running the captions: a spending guard for cloud engines, markers and Find
in the transcript, a notice on the overlay for breaks, and optional Pause and overlay keys that
work while slides or a meeting app are in front. Copies installed from GitHub can now say when a
new version is out. It also carries the fixes of the
[16 November app review](https://github.com/fmadore/Live-translation/blob/main/docs/app-review-2026-11-16.md).

## For operators

- **Spending guard for cloud engines** (Settings › App › Cloud spending, all off by default).
  Pause a session when nothing has been captioned for 5, 10, 15 or 30 minutes, and be told once
  when the estimated cost passes a chosen amount or the session has run for a chosen number of
  hours. The pause is the ordinary Pause, so the session, its clock and the transcript stay as
  they are, and nothing is sent or billed until you resume. The warnings never stop a session.
  Local Whisper and the built-in demo cost nothing and are left alone.
- **Markers in the transcript.** **Add marker** (Ctrl+Shift+M, ⌘⇧M on a Mac) puts a line such
  as "Talk 2 starts" or a speaker's name into the transcript; left blank, it is a numbered
  section break. Markers are kept in the transcript, its exports, the history and the recovery
  copy, and are never shown on the overlay.
- **Find in the live transcript.** Ctrl+F (⌘F) on the Transcript tab finds words in everything
  said so far, highlights each match and steps through them, with the same matching as the
  History search.
- **A notice on the overlay for breaks.** *Show a notice on the overlay…* replaces the captions
  with one line, such as "Back soon" or "Short break", until you clear it or, if you choose,
  until captions resume after a pause. It is not written to the transcript or sent to Zoom, and
  it goes when the session stops.
- **System-wide Pause and Show/hide overlay keys** (Settings › App, off by default).
  Ctrl+Alt+Shift+P and Ctrl+Alt+Shift+O (⌃⌥⇧P and ⌃⌥⇧O on a Mac) also work while slides or a
  meeting app are in front, only while a session is running or paused. The app learns of these
  two key combinations and nothing else you type. There is no Start or Stop key: those stay in
  the app's window. If another app already holds a combination, the setting says so.

## New versions, for copies installed from GitHub

The Microsoft Store and the Mac App Store update their copies themselves, and those copies never
check. A copy installed from GitHub (the Windows installer or MSI, or the Mac disk image) does not
update itself, but from its second launch it asks once: *Check for new versions? This asks GitHub
once a day.* Turned on, it asks GitHub for the latest release at most once a day, never during a
session, and shows *Version X.Y.Z is available* with **Release notes** and **Not now**. Nothing
is downloaded or installed, and the request carries only the app's name and version. The switch
is under Settings › App › Updates
([#217](https://github.com/fmadore/Live-translation/issues/217)).

## Fixes and smaller changes

- **The overlay with four rows.** With both sources and two caption languages in the default
  strip, every row showed its label and no caption. Rows now close up and shrink, the original
  speech gives way first, and the operator window says under Caption size when captions are
  drawn smaller than chosen, or have no room.
- **DeepL** (still experimental): a concluded sentence is no longer dropped when it ends before
  the last sentence-final, the start of the next sentence shows at once, Pause and Stop wait for
  the text DeepL still holds, and source text is deduplicated like the translation.
- **Both sources with local Whisper:** a microphone line decoded late behind a backlog no longer
  passes the echo filter as new speech; system turns are now kept by session time.
- **The pre-flight** counts both sources in the cost row ("×2 sources"), and its tag no longer
  overflows in German or at 225% text size. On a new install, its model row has a **Download**
  button with progress.
- Returning to Live translation keeps the cloud engine you chose last, instead of Gemini.
- The language picker says when the chosen engine lacks a language, instead of "No matching
  languages".
- Escape on the first "hide to tray" explanation no longer quits the app.
- A cloud provider that closes each connection as it opens is no longer retried for ever.
- **Mistral:** a source with nothing to hear for 30 seconds, such as system audio while nobody
  speaks, no longer ends ("No new audio received for 30s"): the app sends a little silence to
  keep the session open. When Mistral's recogniser fails during a session, the app now reconnects
  instead of stopping that source.
- Settings and the Zoom captions dialog open with focus on their first control, not on Close;
  icons follow the text size; the empty History tab leads with what it is for.
- Windows: an unsupported system-audio format is refused when capture opens, instead of
  captioning silence; the graphics-processor helper's timeout also covers sending it audio.

Nothing in 1.15.0 adds a permission or a capability. The
[privacy policy](https://github.com/fmadore/Live-translation/blob/main/docs/privacy.md),
effective 16 November 2026, covers the check for new versions, the system-wide keys and markers.
None of these changes had run on Windows hardware or on a Mac when the release was prepared; the
[review](https://github.com/fmadore/Live-translation/blob/main/docs/app-review-2026-11-16.md#6-notes-from-doing-it)
lists what is checked by hand.

## Downloads

- **Windows:** for the Microsoft Store, upload
  [Live.Translation_1.15.0.msixbundle](https://github.com/fmadore/Live-translation/releases/download/v1.15.0/Live.Translation_1.15.0.msixbundle),
  with native x64 and ARM64 packages at version **1.15.0.0**; the Store signs accepted packages.
  The separate MSIX files are for per-architecture testing, and the EXE and MSI installers for
  installing outside the Store (they are unsigned).
- **macOS 15 or later, Apple silicon:** `Live.Translation_1.15.0_aarch64.dmg`, signed with the
  developer's Developer ID and notarised by Apple.

The [release handoff](https://github.com/fmadore/Live-translation/blob/main/docs/store-updates.md#release-1150-handoff)
records the package checks and the remaining Store steps.
