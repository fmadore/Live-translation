# Live Translation & Subtitles 1.6.0

Whisper brings local multilingual subtitles to Windows, alongside cloud translation and
new controls for bilingual meetings.

- **Offline speech recognition with Whisper.** Download a multilingual Tiny, Base or Small
  model, then transcribe microphone, system or application audio on your CPU without an
  account or API key. Choose from 99 spoken languages or automatic detection. Models are
  verified before use; no audio goes to a speech service. Native x64 and ARM64 are supported.
- **Finish the whole transcript.** If recognition falls behind, pending audio is held in an
  automatically deleted local temporary file. Stop lets Whisper finish; discarding pending
  audio requires confirmation and marks the transcript incomplete. Performance varies by PC,
  model and language. Whisper provides same-language subtitles, not local translation.
- **Two caption languages.** Translate into two languages at once with Gemini or OpenAI.
  Each extra target uses a separate cloud session per audio source, increasing provider usage.
- **Original speech alongside translations.** Include available source text in exports and,
  optionally, beneath translations in the Fit window or Compact overlay.
- **Pause and resume.** Keep the session and transcript together during breaks. Cloud
  connections close after flushing their final turn; Whisper stops new input and continues
  processing the existing backlog. Ctrl+Shift+P toggles Pause.
- **Your filler-word list.** Edit the words hidden from the overlay. Raw transcripts, history
  and exports retain the engine's text.
- **Smoother long sessions.** More efficient history writes and caption fitting, improved
  Gemini reconnect handling, and a more consistent operator interface.

The interface remains available in English, French and German. Cloud translation and cloud
subtitles still require your own provider account and API key and may incur usage charges.
The optional scripted English/French demo remains available for trying the display controls.

See [local Whisper](local-whisper.md) for model sizes, storage and performance details and
the [privacy policy](privacy.md) for local buffering and cloud processing.

For the Microsoft Store, the planned submission artifact is
`Live.Translation_1.6.0.msixbundle`, containing x64 and ARM64 packages at version **1.6.0.0**.
This file is prepared release copy, not a publication announcement. Build evidence and
remaining acceptance checks are tracked in the [release handoff](store-updates.md#release-160-handoff).
