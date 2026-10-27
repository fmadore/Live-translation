# Documentation

These guides describe **[1.12.0](https://github.com/fmadore/Live-translation/releases/tag/v1.12.0)**
(27 October 2026), pending Store certification: captions in a Zoom meeting's own caption bar,
fixes for flashes and layout shifts in setup and the transcript, and a pre-flight that says why
Start waits for an application. The Store serves
**[1.11.0](https://github.com/fmadore/Live-translation/releases/tag/v1.11.0)** (26 October 2026),
accepted as reported by the maintainer the same day: an optional Buy Me a Coffee link that
unlocks nothing, and a fix for an empty band at the bottom of the operator window. 1.10.0 before
it completed the fixes of the [22 October app review](app-review-2026-10-22.md), which also
lists what is left.
The [6 October app review](app-review-2026-10-06.md) records the work in 1.7.0 and the feature candidates still open; the [22 September app review](app-review-2026-09-22.md), [archived reviews](archive/) and older release handoffs are historical records.

## Using the app

- [Local Whisper](local-whisper.md): models, languages, offline setup, translation into English,
  pending audio and Stop.
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
- Provider integration notes: [Gemini](gemini-live-api.md), [OpenAI](openai-realtime-api.md), [Mistral](mistral-realtime-api.md).
- [Security policy](../SECURITY.md), [roadmap](../ROADMAP.md) and [release history](release-notes.md).

## Release 1.12.0 (tagged 27 October 2026, not yet submitted)

- [Release notes](release-1.12.0.md), [release handoff](store-updates.md#release-1120-handoff)
  and [what the next Store submission needs](store-listing-next.md).

## Release 1.11.0 (in the Store)

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
