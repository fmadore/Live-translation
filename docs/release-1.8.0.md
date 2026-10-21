# Live Translation & Subtitles 1.8.0

You can now read back and copy the transcript while a session runs. Mistral subtitles move to
Voxtral Realtime 3, with lines that end where the speaker does and a list of names and terms.

## Read and copy the transcript during a session

- **The whole transcript, during a session.** While a session runs, **Live** and **Transcript**
  tabs sit above the stage. Transcript gives the session's whole transcript the full window,
  with its own scrollbar, and shows the lines still being spoken at the bottom. Until now, one
  long turn of continuous speech could fill the stage and push the transcript out of sight
  ([#124](https://github.com/fmadore/Live-translation/issues/124)). The app remembers which tab
  you chose.
- **Copy the last few minutes.** **Copy** puts the last 5, 10 or 30 minutes, or the whole
  transcript, on the clipboard as plain text, during the session and after Stop. It includes
  the lines still being spoken. A caption that began before the chosen window is copied whole.
  A transcript recovered without timing can only be copied whole.
- **Read without being pulled along.** While you select text in the transcript, new captions
  no longer scroll it, and Ctrl+A selects the transcript alone. With **Include original speech**
  on, the original appears under each paragraph, as Save and Copy write it.

## Mistral subtitles on Voxtral Realtime 3

- **The new model.** Mistral's Voxtral Mini Transcribe Realtime 3 replaces Realtime 2602.
  Mistral counts 45 languages and dialects for it, up from 13, but publishes no list. It costs
  **$0.48 per hour of audio**, up from $0.36, billed by Mistral to your own key; the engine card
  shows the new rate. Mistral lists Realtime 3 as a public preview.
- **Lines that end where the speaker does.** With Mistral, a caption line now closes when
  Mistral detects that the speaker has finished, about half a second after the last word. A
  line no longer breaks in the middle of a sentence because the text paused. After a pause, a
  line still closes at the end of a sentence, so a long talk does not become one endless line.
- **Names and terms.** With Voxtral selected, step 03 of the setup has a **Names and terms**
  list: speaker names, institutions, acronyms and jargon for Voxtral to favour for the whole
  session, up to 100 entries. It is sent to Mistral with each session.
  - It applies from the next Start.
  - It is saved with your setup and in meeting profiles, and kept when you switch engines.
  - Its effect on accuracy has not been measured yet.

Under the surface: the realtime runner can close a line at a provider's detected end of turn,
and hold one open while the speaker is mid-sentence. Only Mistral uses this; the other engines
are unchanged. A live test streams the bundled French recording through the real Mistral
client. The notes on Mistral's API were checked against its Voxtral 3 documentation on
21 October 2026, including a live comparison with 2602 at three delays.
See the [Mistral API notes](https://github.com/fmadore/Live-translation/blob/v1.8.0/docs/mistral-realtime-api.md)
([#156](https://github.com/fmadore/Live-translation/pull/156), closing
[#153](https://github.com/fmadore/Live-translation/issues/153); the transcript reader is
[#155](https://github.com/fmadore/Live-translation/pull/155)).

The [privacy policy](https://fmadore.github.io/Live-translation/privacy) now covers the names
and terms, which are stored on your PC and sent to Mistral, and Copy, which places transcript
text on the Windows clipboard. The app still sends nothing to the developer. Cloud translation
and cloud subtitles still require your own provider account and API key, and may incur usage
charges.

For the Microsoft Store, upload
[Live.Translation_1.8.0.msixbundle](https://github.com/fmadore/Live-translation/releases/download/v1.8.0/Live.Translation_1.8.0.msixbundle),
containing native x64 and ARM64 packages at version **1.8.0.0**. The Store signs accepted packages.
Separate MSIX files are provided for per-architecture testing; EXE/MSI installers are for
installation outside the Store. The
[release handoff](https://github.com/fmadore/Live-translation/blob/main/docs/store-updates.md#release-180-handoff)
records the package evidence and remaining Store acceptance checks.
