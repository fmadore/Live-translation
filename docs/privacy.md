# Privacy Policy — Live Translation & Subtitles

**Effective date:** 24 August 2026

## What this app is

Live Translation & Subtitles is a Windows desktop application with two kinds of operation:

- a built-in scripted demonstration of the caption display, overlay, timer, level meter,
  transcript, and export workflow; and
- optional live speech captioning or translation through a third-party provider selected and
  configured by the user: Google Gemini, OpenAI, or Mistral.

The built-in demonstration is explicitly labeled as a demonstration. It does not capture or
recognize speech.

## Audio

The built-in demonstration does not open a microphone or system-audio device. Its simulated
level data and English/French caption text are bundled inside the application and remain on the
device.

When a live provider is selected, the app captures only the audio source selected by the user:
a chosen microphone, Windows system audio, or both. Audio is processed in memory and streamed
directly to the selected provider for the duration of the session. The application does not
write captured audio to disk.

## Captions and transcripts

Caption text is held in memory while the app runs, and the whole session is kept: the transcript
is never shortened or discarded behind the user's back. A transcript is written to disk only when
the user explicitly chooses to save it, or through the optional recovery copy described below.
The app does not automatically upload saved transcripts.

Closing the app with unsaved caption text asks whether to save or discard it first.

## Optional local recovery copy

Recovery is **off by default**. When the user switches on *Keep a local recovery copy while
captioning*, the app periodically writes the finalized caption lines of the current session to a
single file in its own local application-data folder on the same PC:

`%LOCALAPPDATA%\org.stias.live-translation\recovery\transcript.json`
ecovery	ranscript.json`

- It contains finalized caption text only — no audio, no API key, no provider identity, and no
  device name.
- It is never uploaded, transmitted, or shared. The developer operates no service that could
  receive it.
- There is one file and no history: each write replaces the previous one.
- It is deleted as soon as the transcript is saved, the log is cleared, the unsaved-transcript
  prompt is answered, the recovery option is switched off, or a recovered transcript is restored
  or refused at the next launch.
- If the app is closed cleanly, no recovery file remains. One is only ever found at startup after
  a crash or a power cut, and the user is asked whether to restore or delete it.

Deleting the file by hand at any time is safe and removes the data.

## API keys

No API key is needed for the built-in demonstration. Optional provider keys are stored in
Windows Credential Manager and read only by the native application core. A key is sent only to
the provider that issued it to authenticate the live connection. The developer does not receive,
copy, proxy, or store provider keys.

## Preferences stored on the device

The app locally stores ordinary interface preferences, including the last selected mode,
provider, audio source, language, overlay position, caption size, whether the optional recovery
copy is enabled, and whether closing the window leaves the app running in the notification area.
These preferences contain no captured audio, caption text, or provider key.

## Running in the notification area

The app does not start with Windows and does not run as a background service. It appears in the
notification area only while it is running, and closing the window quits it unless the user turns
on *Keep running in the tray when I close this window*. With that on, the first time the window
is hidden the app says so explicitly, and quitting from the notification-area icon stops any
active capture before it exits.

## What the developer collects

The developer operates no server, relay, account system, telemetry, advertising, analytics, or
crash-reporting service for this app. The developer receives no audio, captions, transcripts,
API keys, device identifiers, or usage data from the application.

## Third-party processing

Starting a live session sends audio directly to the chosen provider under the user’s relationship
with that provider. Their terms and privacy policies apply:

- [Google Privacy Policy](https://policies.google.com/privacy)
- [OpenAI Privacy Policy](https://openai.com/policies/privacy-policy/)
- [Mistral AI Privacy Policy](https://mistral.ai/terms/#privacy-policy)

The built-in demonstration contacts none of these services.

## Children

The app is not directed to children and the developer knowingly collects no personal information
from children.

## Changes to this policy

Material changes will be published with a new effective date. The policy applicable to a release
is included with its Store listing and repository documentation.

## Contact

For privacy questions, use the publisher support contact shown on the Microsoft Store listing for
Live Translation & Subtitles (Product ID `9PFB8LR3RR9X`).
