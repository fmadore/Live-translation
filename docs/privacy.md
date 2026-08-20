# Privacy Policy — Live Translation & Subtitles

- **Effective date:** 20 August 2026
- **Application:** Live Translation & Subtitles, a Windows desktop application
- **Developer:** Frédérick Madore

## What this app is

Live Translation & Subtitles captures audio from a microphone, from whatever is playing on
the PC (Windows system audio), or from both, turns that speech into text, and shows the
result as captions in an on-screen overlay. It is a desktop application that runs on your
own computer. There is no account, no sign-in, and no service operated by the developer
sitting behind it.

Captions are produced in one of two ways, and the difference is the most important thing in
this policy:

- **A cloud engine** — Google Gemini, OpenAI, or Mistral — reached with an API key that you
  obtain from that provider yourself and enter into the app. While a session runs, audio is
  streamed to the provider you selected.
- **The on-device engine** — a local speech recognizer (whisper.cpp) bundled with the app.
  It needs no API key and no network connection, and audio never leaves your computer.

You choose which one to use, and the app tells you which is active before you start.

## Audio

**Nothing is captured until you start a session.** Capture begins when you start a session
and ends when you stop it. Between sessions the app opens no audio device and sends nothing.

**With a cloud engine,** captured audio is downmixed to mono PCM and streamed over an
encrypted WebSocket connection (`wss://`) to the provider you selected — Google, OpenAI, or
Mistral — for as long as the session runs. It goes to that provider and to no one else. The
developer operates no server, no relay, and no proxy: the connection is between your computer
and the provider, authenticated with your own API key and billed to your own account with
that provider. The developer receives nothing.

**With the on-device engine,** audio is processed entirely on your computer by the bundled
speech model. Nothing is transmitted: in this mode the app makes no network connection at
all.

**The app stores no audio, in either mode.** Captured audio lives only in short-lived memory
buffers on its way to the recognizer — a fraction of a second in the transport queue, plus,
for the on-device engine, the utterance currently being recognized — and is discarded once it
has been used. No audio file is ever written to disk, and no recording is kept.

## Captions and transcripts

Captions appear in the overlay and in the operator window as they arrive. Finalized lines are
kept in memory for the duration of the run so that you can read back over them and export
them.

Transcripts are written to disk **only when you click Save text or Save Markdown.** The file
is written under `Documents\Live-translation\` (falling back to your Downloads folder, and
then to your temporary folder, if Documents is unavailable). It stays on your computer; the
app never uploads it anywhere. If you never click save, the transcript is discarded when the
session log is cleared or the app closes.

A saved transcript contains the caption text, the source-language text the recognizer heard,
a label for each audio source (Microphone or System), and the date and time of the export.
It contains whatever was said in the room or on the call, so treat the file with the same
care you would give a meeting recording, and tell participants when a session is being
captioned and saved.

## API keys

If you use a cloud engine, the API key you enter is stored in **Windows Credential Manager**,
the operating system's own credential store, under the service name
`org.stias.live-translation`, with a separate entry per provider. Keys are read only by the
app's Rust core, only when a session opens a connection, and only to authenticate that
connection to the provider that issued the key. They are never written to the app's own files
or logs, never passed into the app's user-interface layer, and never sent to the developer or
to anyone other than the issuing provider.

You can overwrite or delete a stored key at any time from the app's key panel, or from
Windows Credential Manager directly. The on-device engine needs no key at all.

## Preferences stored on your device

The app remembers a few interface preferences in the local storage of the WebView that
renders its windows: the caption text size, whether the caption overlay has already been
positioned on a display, and your last session setup (audio source, translation or subtitle
mode, caption language, chosen engine, and the name of the microphone device you picked).
These are settings, not records: no identifier, no history of use, and nothing about what was
said. They stay on your computer and go away when you uninstall the app.

## What the developer collects

**Nothing.** There is no telemetry, no analytics, no crash reporting, no usage statistics, no
advertising, and no advertising identifier. There are no accounts and no registration. The
app contacts no server belonging to the developer, because there is none.

The only outbound network traffic the app produces is the connection to the cloud provider
you selected, while a session is running.

## Third-party processing

When you run a session on a cloud engine, that provider processes the streamed audio under
its own terms and privacy policy, on the account whose API key you supplied. **Your contract
is with that provider, not with the developer of this app.** How long they retain data,
whether they use it to improve their models, and what controls or deletion rights you have
are decided by them and set out in their policies:

- **Google (Gemini API)** — [Google Privacy Policy](https://policies.google.com/privacy) and
  the [Gemini API Additional Terms of Service](https://ai.google.dev/gemini-api/terms)
- **OpenAI** — [OpenAI Privacy Policy](https://openai.com/policies/privacy-policy/)
- **Mistral AI** — [Mistral AI Privacy Policy](https://legal.mistral.ai/terms/privacy-policy)

One detail worth stating plainly: in translation mode, Gemini and OpenAI also generate spoken
audio of the translation on their side. The app requests text captions, discards that
generated audio without playing or saving it, and never sends it anywhere — but it is
produced on the provider's servers and your provider account is billed for it.

If you would prefer that no audio leave your machine at all, use the on-device engine: it is
available for same-language subtitles and needs no provider and no key.

## Children

The app has no accounts, collects nothing, and is not directed at children.

## Changes to this policy

If the app's handling of data changes, this document changes with it and the effective date
above is updated. It lives in the application's public source repository, so every revision
is visible in that repository's history.

## Contact

Questions about this policy, or about how the app handles data, can be raised as an issue at
<https://github.com/fmadore/Live-translation/issues>.

The app is developed and published by Frédérick Madore.
