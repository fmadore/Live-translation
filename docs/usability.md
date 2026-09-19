# Meeting profiles and live controls

These improvements are in development after v1.3.0. Scroll-back playback is not included.

## Meeting profiles

Expand **Meeting profiles** on the idle setup screen, enter a name and choose **Save current
setup**. Each save creates a named snapshot; delete an obsolete profile with confirmation.
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

The Settings and idle setup screen both include expandable shortcut help. Existing overlay
move-mode keys are unchanged.

## Verification

Automated tests cover interim batching without starvation, independent sources, immediate
finalization, stop cancellation, preference validation, profile round trips and hardware checks,
profile controls, safe placement, concurrent history renames, search, keyboard guards and live
status classification. Gemini tests cover Smart final/interim precedence and empty finals.

Browser inspection checks the appearance preview, presets, profile controls and shortcut help
at 1280×720 and 800×600, including the German settings. Native mixed-DPI placement, keyboard
operation in the packaged app, and a fresh paid provider probe still require release acceptance.
