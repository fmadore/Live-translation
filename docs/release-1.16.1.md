# Live Translation & Subtitles 1.16.1

Gemini subtitles as 1.16.0 described them: word for word, about a second after the speaker
pauses.

## Gemini subtitles

- **Gemini now transcribes verbatim.** 1.16.0's notes said the app asked Gemini for a verbatim
  transcript, but that change had not made it into the release: 1.16.0 still used Gemini's
  SMART mode. Its captions no longer repeated the session, yet they could come 15 to 30 seconds
  after the speech, a sentence spoken twice was dropped, and a spoken "delete that" edited the
  text instead of being captioned. 1.16.1 asks for Google's VERBATIM transcription: each caption
  arrives about a second after the speaker pauses, punctuated and word for word, fillers
  included (the overlay's [filler filter](https://fmadore.github.io/Live-translation/caption-layout.html#hide-filler-words)
  can still hide them on screen; the transcript keeps them).
- **A segment Gemini ends without a final transcript is kept.** Now and then Gemini closes a
  stretch of speech without sending its final text, and the next stretch's first words replaced
  it on screen. The app now keeps that segment from the last words it showed, so only the few
  words Gemini never sent are missing.
- The engine card and the note under *Hide filler words* say Gemini writes word for word, in
  English, French and German.

Checked against Google's service on 24 November 2026 with a French and English recording:
captions came 3 to 5 seconds after each passage ended, with no repetition, and the French
passage spoken a second time was captioned whole. Nothing else changes: the same audio, names
and terms and spoken language go to Google, and the
[privacy policy](https://fmadore.github.io/Live-translation/privacy.html) is unchanged.

Everything in [1.16.0](https://github.com/fmadore/Live-translation/releases/tag/v1.16.0) is
included: OpenAI subtitles, continuing a cloud source with offline Whisper, a spoken language
for Gemini, the first words after Start kept with every cloud engine, and the fixes of the
20 November audit.

## Downloads

- **Windows:** for the Microsoft Store, upload
  [Live.Translation_1.16.1.msixbundle](https://github.com/fmadore/Live-translation/releases/download/v1.16.1/Live.Translation_1.16.1.msixbundle),
  with native x64 and ARM64 packages at version **1.16.1.0**; the Store signs accepted packages.
  The separate MSIX files are for per-architecture testing, and the EXE and MSI installers for
  installing outside the Store (they are unsigned).
- **macOS 15 or later, Apple silicon:** `Live.Translation_1.16.1_aarch64.dmg`, signed with the
  developer's Developer ID and notarised by Apple.

The [release handoff](https://github.com/fmadore/Live-translation/blob/main/docs/store-updates.md#release-1161-handoff)
records the package checks and the remaining Store steps.
