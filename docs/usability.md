# Meeting profiles and live controls

Features introduced in **v1.4.0**, with the operator interface reorganized in **1.4.1**.
See the [release checklist](store-updates.md#release-151-handoff).
This update adds no new scroll-back or return-to-live controls.

## Session controls (1.4.2)

Start and Stop share a persistent action bar immediately below the app header. The bar
stays visible while either column scrolls, including the stacked layout at narrow widths
or enlarged text sizes. Rehearse appears beside Start. Setup, live captions and the saved
transcript all remain below the session controls. Readiness gates and keyboard shortcuts
are unchanged.

## Meeting profiles

Use the profile picker above step 01, then **Manage profiles**. Choose **Save current setup…**,
enter a name and save. Each profile row offers **Load profile** and a management menu for
**Rename** or confirmed deletion. Each save creates a named snapshot.
Profiles persist locally across restart, using `meeting.profiles` in webview local storage.

A profile includes output mode, provider, caption language, audio source and device identifiers,
caption appearance, reading pace, hold time, cleanup toggle and the current overlay rectangle.
It excludes API keys, rehearsal mode, transcripts and application process identities. Loading
does not start capture or change the opt-in history/recovery preferences.

Loading is disabled during capture, an audio test or another setup operation. The app enumerates
devices before applying the profile. Missing endpoints fall back to the Windows default with a
notice; application capture remains unselected until the operator explicitly selects a process.
The preflight must be checked again. A saved overlay rectangle is constrained to an available
display's work area; a disconnected projector falls back to the primary display. Check placement
after display or scaling changes. Only the operator window may invoke the placement commands.

## Reading and saved sessions

Settings has **Captions**, **Reading**, **Transcript history** and **App** tabs.
The dialog and tab bar retain their dimensions when switching tabs; the content scrolls
independently and starts at the top of each newly selected tab. Short tabs intentionally
leave spare space. Use Left/Right arrows, Home or End within the tab bar, and Tab to
reach its content. Changes apply immediately; close with the corner button or Escape.

See [caption layout](caption-layout.md#reading-pace-and-preview) for the adjustable reading pause,
Steadier updates and presets. See [transcript history](transcript-history.md#titles-and-search)
for session titles, full-text search and inclusive date/language filters.

## Input status

The running rail describes each active source independently. Connection, reconnection and error
states take precedence. A recent caption is shown as **Receiving captions**; otherwise a recent
level above the existing RMS signal threshold is **Receiving audio**. When audio is arriving
but captions have not arrived for 15 seconds, the app suggests checking the provider/connection.
Quiet inputs say **Listening — no recent audio signal**. These are observations, not speech
detection or a diagnosis of provider failure. The built-in demo uses simulated input events.

## Keyboard controls

These shortcuts apply while the operator window has focus, outside inputs, selects, editable
text and dialogs. Repeated keydown and IME composition do not trigger commands. They are not
system-wide hotkeys and cannot control the app while another application is focused.

| Shortcut | Action |
| --- | --- |
| Ctrl+Shift+Space | Start / stop; respects the same readiness and busy gates as the buttons |
| Ctrl+Shift+O | Show / hide the overlay |
| Ctrl+Shift+Up / Down | Increase / decrease caption size |
| F2 | Switch translation direction while stopped |

Shortcut help lives in **Settings → App**. A shortcut hint sits beside Start. Existing overlay
move-mode keys are unchanged.

## Verification

The final 1.4.1 frontend suite passed all **337 tests**. On 20 September, browser checks
confirmed identical dialog/tab-bar positions across all four tabs at 1280×800 and a
narrow preview, with independently scrolling content. The profile selector and manage
button had matching top/bottom edges and 40 px heights at default text size.
French date hints were visually checked; automated German regressions cover valid and
invalid dates, leap years, calendar event handling, inclusive bounds and filter reset.
These checks do not certify the native calendar popup or installed MSIX behavior.

Automated tests cover interim batching without starvation, independent sources, immediate
finalization, stop cancellation, preference validation, profile round trips and hardware checks,
profile controls, safe placement, concurrent history renames, search, keyboard guards and live
status classification. Gemini tests cover Smart final/interim precedence and empty finals.

Impeccable browser inspection on 2026-09-19 covered appearance previews, the Large room preset,
profile save/load feedback, history search and title editing, separate active/quiet input statuses,
and the stop shortcut. The checks used the real components with a temporary synthetic backend
at 1280×800 and 800×600, including French and German interfaces and 200% accessibility text
scaling. The two-source stable overlay was also inspected at 1000×400. Long unbroken session
titles now wrap without horizontal scrolling, and history date controls use a dark color scheme
so their calendar icons remain visible. Keyboard focus was checked in the search/date controls;
the tested pages reported no browser console errors. The temporary backend was removed.

Earlier browser inspection covered shortcut help at 1280×720 and 800×600. Native mixed-DPI
placement, keyboard operation in the packaged app, and a fresh paid provider probe still require
release acceptance. These browser checks do not exercise native storage or cloud services.
