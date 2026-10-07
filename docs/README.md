# Documentation

These guides describe **1.7.0**, prepared on 7 October 2026 and not yet published: offline
translation into English with Whisper, an overlay that reopens where it was placed, and cloud
sessions and Whisper that hold up better through a long meeting. The latest GitHub release is
**[1.6.1](https://github.com/fmadore/Live-translation/releases/tag/v1.6.1)** (6 October 2026),
which will not be submitted to the Store; the Store serves **1.6.0**, confirmed live by the
maintainer on 6 October 2026, until 1.7.0 replaces it.
The [6 October app review](app-review-2026-10-06.md) records the work in 1.7.0 and the feature candidates still open; the [22 September app review](app-review-2026-09-22.md), [archived reviews](archive/) and older release handoffs are historical records.

## Using the app

- [Local Whisper](local-whisper.md): models, languages, offline setup, translation into English,
  pending audio and Stop.
- [Language coverage](language-coverage.md): translation targets, spoken-language selection and verification limits.
- [Audio checks](audio-device-testing.md) and [application capture](application-capture.md).
- [Meeting controls and profiles](usability.md): Pause, two caption languages and shortcuts.
- [Caption layout](caption-layout.md): overlay placement, reading modes and filler-word lists.
- [Transcript export](transcript-export.md) and [optional history](transcript-history.md).
- [Privacy](privacy.md), [accessibility checks](accessibility.md) and [interface localization](localization.md).

## Development

- [Architecture](architecture.md) and [native Whisper build requirements](local-whisper.md#building).
- Provider integration notes: [Gemini](gemini-live-api.md), [OpenAI](openai-realtime-api.md), [Mistral](mistral-realtime-api.md).
- [Security policy](../SECURITY.md), [roadmap](../ROADMAP.md) and [release history](release-notes.md).

## Release 1.7.0 (in preparation)

- [Release notes](release-1.7.0.md) and [release handoff](store-updates.md#release-170-handoff).
- 1.6.1: [release notes](release-1.6.1.md) and [release handoff](store-updates.md#release-161-handoff), GitHub only.
- 1.6.0: [release notes](release-1.6.0.md) and [release handoff](store-updates.md#release-160-handoff).
- [Store copy in English, French and German](store-listing.md) and [screenshot plan](store-screenshots/README.md).
- [MSIX packaging](packaging-msix.md), [Store identity and acceptance](microsoft-store.md),
  [Partner Center walkthrough](partner-center-walkthrough.md).
