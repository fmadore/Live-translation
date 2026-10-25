# Documentation

These guides describe **[1.10.0](https://github.com/fmadore/Live-translation/releases/tag/v1.10.0)**,
prepared and awaiting its tag and Store submission. It completes the fixes of the
[22 October app review](app-review-2026-10-22.md), which also lists what is left: cloud
sessions that keep their words through a Gemini handover, system audio that keeps up under
load, an overlay with fewer permissions, and no process handles outside One application
capture. The Store serves **[1.9.0](https://github.com/fmadore/Live-translation/releases/tag/v1.9.0)**
(22 October 2026), live as confirmed with `Get-AppxPackage` on 25 October 2026: a light theme
that follows Windows' light or dark app mode, and the review's first three batches.
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

## Release 1.10.0 (awaiting its tag and Store submission)

- [Release notes](release-1.10.0.md) and [release handoff](store-updates.md#release-1100-handoff).
- 1.9.0, the version in the Store: [release notes](release-1.9.0.md) and [release handoff](store-updates.md#release-190-handoff).
- 1.8.0: [release notes](release-1.8.0.md) and [release handoff](store-updates.md#release-180-handoff).
- 1.7.1: [release notes](release-1.7.1.md) and [release handoff](store-updates.md#release-171-handoff).
- 1.7.0, GitHub only, which reached the Store as 1.7.1: [release notes](release-1.7.0.md) and [release handoff](store-updates.md#release-170-handoff).
- 1.6.1: [release notes](release-1.6.1.md) and [release handoff](store-updates.md#release-161-handoff).
- [Store copy in English, French and German](store-listing.md) and [screenshot plan](store-screenshots/README.md).
- [MSIX packaging](packaging-msix.md), [Store identity and acceptance](microsoft-store.md),
  [Partner Center walkthrough](partner-center-walkthrough.md).
