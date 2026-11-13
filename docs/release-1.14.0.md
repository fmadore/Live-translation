# Live Translation & Subtitles 1.14.0

DeepL joins Gemini and OpenAI as a cloud translation engine, as an experimental option for
people with a paid DeepL API plan. Mistral subtitles no longer stop when Mistral's transcription
times out, and the overlay no longer shows empty rows when both audio sources share a shallow
strip.

## DeepL, experimental

- **A fourth cloud engine, for live translation.** Choose **Live translation**, then **DeepL**,
  and paste your own DeepL API key. DeepL hears which language is spoken and writes the caption
  language you choose, in 42 of the app's caption languages
  ([language coverage](https://github.com/fmadore/Live-translation/blob/main/docs/language-coverage.md)).
  There is no DeepL subtitle (same-language) mode, and DeepL takes no list of names and terms.
- **A paid DeepL API plan is needed.** DeepL's Voice API is not part of its free API plan. A free
  DeepL API key, one ending in `:fx`, is refused when you save it, with a sentence saying why,
  and no request is ever made with it.
- **Experimental.** The engine was built from DeepL's documentation and tested against its
  documented messages, but it has not yet run in a live session; the engine picker says so. The
  line breaks, the timing and some language codes are still to be confirmed with a paid key
  ([DeepL notes](https://github.com/fmadore/Live-translation/blob/main/docs/deepl-voice-api.md)).
- **Privacy.** Like the other cloud engines, DeepL receives the audio of the chosen sources
  directly from your computer, under your own DeepL account and terms, and the app asks once
  before audio first goes to DeepL. The key is kept in Windows Credential Manager or the macOS
  Keychain. The [privacy policy](https://github.com/fmadore/Live-translation/blob/main/docs/privacy.md),
  effective 11 November 2026, names DeepL.

## Fixes

- **Mistral subtitles reconnect after a transcription timeout.** During a live stream on
  11 November 2026, Mistral ended a session with "Timeout waiting for response from streaming
  transcription" (code 3804), and the app stopped the source, leaving the operator to press Start
  again mid-event. The app now reconnects, as it already did after Mistral's code 3803.
- **Every row of the overlay shows a line with both sources.** With the microphone and system
  audio both on, the overlay splits its height between the Room and Remote rows. In a strip
  sized for one source, each row was too shallow for one line, so the labels appeared with no
  caption under them. The rows are now drawn smaller when they need to be, down to 20 px, so
  each shows a line; the window keeps its size. Reported from a real Zoom meeting with Mistral
  subtitles.

Nothing in this release is specific to Windows or macOS. Neither the DeepL engine nor the two
fixes has been tried on a Mac.

## Downloads

- **Windows:** for the Microsoft Store, upload
  [Live.Translation_1.14.0.msixbundle](https://github.com/fmadore/Live-translation/releases/download/v1.14.0/Live.Translation_1.14.0.msixbundle),
  with native x64 and ARM64 packages at version **1.14.0.0**; the Store signs accepted packages.
  The separate MSIX files are for per-architecture testing, and the EXE and MSI installers for
  installing outside the Store (they are unsigned).
- **macOS 15 or later, Apple silicon:** `Live.Translation_1.14.0_aarch64.dmg`, signed with the
  developer's Developer ID and notarised by Apple.

The [release handoff](https://github.com/fmadore/Live-translation/blob/main/docs/store-updates.md#release-1140-handoff)
records the package checks and the remaining Store steps.
