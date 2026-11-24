# Documentation

The developers' index. The public home page on GitHub Pages,
<https://fmadore.github.io/Live-translation/>, is [`index.html`](index.html) in this folder
([#252](https://github.com/fmadore/Live-translation/issues/252)); it links here as *Developer
documentation*, through GitHub, because Pages publishes this file only as raw Markdown once an
`index.html` exists. Pages publishes `docs/` and nothing above it, so a link from these guides to
a file elsewhere in the repository uses its GitHub URL; `src/lib/docsLinks.test.ts` fails on a
relative link that leaves `docs/`.

Jekyll renders every other Markdown page here with [`_layouts/default.html`](https://github.com/fmadore/Live-translation/blob/main/docs/_layouts/default.html),
which gives it the home page's masthead, palette and footer from
[`assets/site.css`](assets/site.css); [`assets/site.js`](assets/site.js) adds a `#` link to each
heading, the section being read in the bar under the masthead and an *On this page* list. The
user guides the home page lists are in [`_data/guides.yml`](https://github.com/fmadore/Live-translation/blob/main/docs/_data/guides.yml), which names them
*Guide* and lists the others at the foot of each; a new guide goes there and on the home page.

These guides describe `main`, released as
**[1.16.1](https://github.com/fmadore/Live-translation/releases/tag/v1.16.1)** (24 November
2026; the Store submission is still to be made): Gemini subtitles on Google's VERBATIM mode, a second or two after a pause. The version in the Store is
**[1.16.0](https://github.com/fmadore/Live-translation/releases/tag/v1.16.0)** (23 November
2026, MSIX 1.16.0.0), read from the installed package on 24 November 2026: OpenAI subtitles,
continuing a cloud source with offline Whisper in the same session when its engine stops, a
spoken language for Gemini subtitles, Gemini subtitles that no longer repeat earlier speech, the
first words after Start kept on cloud engines, and the fixes of the 20 November audit. Before
it, the Store served **[1.15.0](https://github.com/fmadore/Live-translation/releases/tag/v1.15.0)**
(17 November 2026, MSIX 1.15.0.0): a
spending guard for cloud engines, markers and Find in the transcript, a notice on the overlay
for breaks, opt-in system-wide Pause and overlay keys, several History sessions deleted or saved
at once, and the fixes of the [16 November app review](app-review-2026-11-16.md), which also
lists what is still to check by hand. It replaced
**[1.14.0](https://github.com/fmadore/Live-translation/releases/tag/v1.14.0)** (13 November
2026): DeepL as an experimental fourth cloud translation engine, Mistral reconnecting after a
transcription timeout, and overlay rows that each show a line with both sources. Before it, the
Store served **[1.13.1](https://github.com/fmadore/Live-translation/releases/tag/v1.13.1)** (11 November
2026): the fixes from the Mac app's first run on a real Mac, on top of 1.13.0's Mac app, Whisper
on the graphics processor, speech detection, Large v3 Turbo, names and terms for Gemini
Transcribe and Whisper, and the question before audio first goes to a cloud provider. 1.10.0
completed the fixes of the [22 October app review](app-review-2026-10-22.md), which also
lists what is left. The [10 November app review](app-review-2026-11-10.md) lists the findings
fixed in 1.13.0 and what is still to check by hand.
The [6 October app review](app-review-2026-10-06.md) records the work in 1.7.0 and the feature candidates still open; the [22 September app review](app-review-2026-09-22.md), [archived reviews](archive/) and older release handoffs are historical records.

## Using the app

- [Local Whisper](local-whisper.md): models (Large v3 Turbo included), languages, offline setup,
  translation into English, speech detection, the graphics processor, pending audio and Stop.
- [Language coverage](language-coverage.md): translation targets, spoken-language selection and verification limits.
- [Audio checks](audio-device-testing.md) and [application capture](application-capture.md).
- [Meeting controls and profiles](usability.md): Pause, two caption languages and shortcuts.
- [Caption layout](caption-layout.md): overlay placement, reading modes and filler-word lists.
- [Captions in Zoom](zoom-captions.md): sharing one caption language with a Zoom meeting's
  caption bar (from 1.12.0).
- [Transcript export](transcript-export.md) and [optional history](transcript-history.md).
- [Privacy](privacy.md), [accessibility checks](accessibility.md) and [interface localization](localization.md).

## Development

- [Architecture](architecture.md) and [native Whisper build requirements](local-whisper.md#building).
- [macOS port](macos.md): what is implemented, the test build and the checklist for a first run
  on a Mac ([#195](https://github.com/fmadore/Live-translation/issues/195)), and the
  [Mac App Store submission](mac-app-store.md), with its
  [decision record](decisions/2026-11-05-mac-app-store.md).
- Design notes in [decisions/](decisions/): the
  [one-click fallback engine](decisions/2026-11-23-fallback-engine.md) for a cloud source that
  ends mid-session (proposed, [#248](https://github.com/fmadore/Live-translation/issues/248)).
- Provider integration notes: [Gemini](gemini-live-api.md), [OpenAI translation](openai-realtime-api.md), [OpenAI subtitles](openai-transcribe-api.md), [Mistral](mistral-realtime-api.md), [DeepL](deepl-voice-api.md).
  Research for a possible fifth engine, not built:
  [Soniox](soniox-realtime-api.md) ([#250](https://github.com/fmadore/Live-translation/issues/250)).
- [Security policy](https://github.com/fmadore/Live-translation/blob/main/SECURITY.md),
  [roadmap](https://github.com/fmadore/Live-translation/blob/main/ROADMAP.md) and
  [release history](release-notes.md).
- [Repository statistics](repository-statistics.md): downloads per version and visits to the
  repository, recorded daily on the `stats` branch.

## Release 1.16.1 (24 November 2026, Store submission pending)

- [Release notes](release-1.16.1.md): Gemini subtitles on Google's VERBATIM mode, as 1.16.0
  announced, and a segment Gemini ends without a final kept.
- [Release handoff](store-updates.md#release-1161-handoff) and
  [the Store copy](store-listing.md).

## Release 1.16.0 (23 November 2026, live in the Store)

- [Release notes](release-1.16.0.md): OpenAI subtitles, continuing a cloud source with offline
  Whisper when its engine stops, a spoken language for Gemini subtitles, Gemini subtitles that
  no longer repeat earlier speech, the first words after Start kept, and the fixes of the
  20 November audit.
- [Release handoff](store-updates.md#release-1160-handoff) and
  [what the Store submission needs](store-listing-next.md).

## Release 1.15.0 (tagged 17 November 2026, in the Store)

- [Release notes](release-1.15.0.md): a spending guard for cloud engines, markers and Find in
  the transcript, a notice on the overlay for breaks, opt-in system-wide Pause and overlay keys,
  a new-version notice for copies installed from GitHub, and the fixes of the
  [16 November app review](app-review-2026-11-16.md).
- [Release handoff](store-updates.md#release-1150-handoff).

## Release 1.14.0 (tagged 13 November 2026, replaced in the Store by 1.15.0)

- [Release notes](release-1.14.0.md): DeepL as an experimental fourth cloud translation engine
  (paid DeepL API plan only), Mistral reconnecting after a transcription timeout, and overlay rows
  that each show a line when both sources share a shallow strip.
- [Release handoff](store-updates.md#release-1140-handoff); 1.14.0.0 is live in the Store, read
  from the installed package on 17 November 2026. [DeepL notes](deepl-voice-api.md).

## Release 1.13.1 (tagged 11 November 2026, replaced by 1.14.0; in Mac App Review)

- [Release notes](release-1.13.1.md): the fixes from the Mac app's first run on a real Mac.
- [Release handoff](store-updates.md#release-1131-handoff); 1.13.1.0 was live in the Store from
  at least 13 November 2026 until 1.14.0.0 replaced it. 1.13.1 is the Mac build in App Review.
  [Mac App Store screenshots](store-screenshots/README.md#mac-app-store-english).

## Release 1.12.0 (tagged 27 October 2026, replaced by 1.13.1)

- [Release notes](release-1.12.0.md) and [release handoff](store-updates.md#release-1120-handoff).

## Release 1.13.0 (tagged 10 November 2026, published on GitHub, never submitted)

- [Release notes](release-1.13.0.md): the Mac app, Whisper on the graphics processor,
  speech detection, Large v3 Turbo, names and terms for Gemini Transcribe and Whisper, and the
  prompt before audio first goes to a cloud provider.
- [Release handoff](store-updates.md#release-1130-handoff) and
  [what the Store submission needs](store-listing-next.md).
- Whisper on the graphics processor (Metal on a Mac, a Vulkan helper on Windows), bundled speech
  detection, loop and credit filters and Large v3 Turbo: [local Whisper](local-whisper.md#graphics-processor).

## Release 1.11.0 (replaced by 1.12.0) and earlier

- [Release notes](release-1.11.0.md) and [release handoff](store-updates.md#release-1110-handoff).
- 1.10.0: [release notes](release-1.10.0.md) and [release handoff](store-updates.md#release-1100-handoff).
- 1.9.0: [release notes](release-1.9.0.md) and [release handoff](store-updates.md#release-190-handoff).
- 1.8.0: [release notes](release-1.8.0.md) and [release handoff](store-updates.md#release-180-handoff).
- 1.7.1: [release notes](release-1.7.1.md) and [release handoff](store-updates.md#release-171-handoff).
- 1.7.0, GitHub only, which reached the Store as 1.7.1: [release notes](release-1.7.0.md) and [release handoff](store-updates.md#release-170-handoff).
- 1.6.1: [release notes](release-1.6.1.md) and [release handoff](store-updates.md#release-161-handoff).
- [Store copy in English, French and German](store-listing.md) and [screenshot plan](store-screenshots/README.md).
- [MSIX packaging](packaging-msix.md), [Store identity and acceptance](microsoft-store.md),
  [Partner Center walkthrough](partner-center-walkthrough.md).
