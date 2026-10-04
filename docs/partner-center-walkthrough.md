# Partner Center submission walkthrough — 1.6.0

Target: **Live Translation & Subtitles 1.6.0**, Product ID `9PFB8LR3RR9X`.
Published on GitHub, not yet submitted to the Store. Download the
[combined x64 + ARM64 bundle](https://github.com/fmadore/Live-translation/releases/download/v1.6.0/Live.Translation_1.6.0.msixbundle).
Use the [release handoff](store-updates.md#release-160-handoff) for package evidence and
outstanding acceptance checks.

## Before Partner Center

1. Verify the unsigned `Live.Translation_1.6.0.msixbundle` contains both native packages at
   **1.6.0.0**, with the assigned [Store identity](microsoft-store.md#store-identity-assigned).
2. Test signed per-architecture MSIX packages on x64 and ARM64. Use
   [packaging instructions](packaging-msix.md); the side-by-side Whisper Test app is separate
   evidence and must not be uploaded as the Store package.
3. Complete the [acceptance checklist](microsoft-store.md#acceptance-checklist), including
   offline Whisper after download, cloud features, Stop/backlog, overlay, export and WACK.
4. Refresh [screenshots](store-screenshots/README.md) and review all three listing languages.
5. Check the published [privacy policy](https://fmadore.github.io/Live-translation/privacy),
   including Whisper model downloads and temporary local audio buffering.

## Pricing and properties

Retain the existing free, public Productivity listing and its supported markets. The app
sells no subscriptions, credits or API access. Optional cloud usage is billed by the user's
chosen provider. Keep the publisher, product name and identity unchanged.

| Field | Value / review point |
| --- | --- |
| Privacy | `https://fmadore.github.io/Live-translation/privacy` |
| Website | `https://github.com/fmadore/Live-translation` |
| Support | `https://github.com/fmadore/Live-translation/issues` |
| Purchases outside Microsoft commerce | No in-product offering; provider dependencies and charges disclosed in listing |
| Accessibility certification | Do not claim certification without a completed audit |
| Generative AI | Yes: optional Gemini/OpenAI translation generates text |
| Personal-information transmission | Cloud modes send selected audio directly to third parties; local Whisper does not |
| Recommended memory | 8 GB; model/CPU determine Whisper speed |

Review the current age-rating questionnaire for third-party connections: cloud providers and
user-requested model downloads use the network. Microphone hardware is needed only for
microphone capture; system capture and the demo do not require one.

## Packages

Replace the previous submission packages with the single unsigned multi-architecture
`Live.Translation_1.6.0.msixbundle`. The Store signs accepted packages. Keep locally signed
test copies separate; do not upload an NSIS, MSI or Whisper Test artifact here.

| Manifest element | Value |
| --- | --- |
| Identity name | `49346FMadore.LiveTranslationSubtitles` |
| Publisher | `CN=5D0ECC96-3998-452E-B7E9-29BE9B576F86` |
| Publisher display name | `FMadore` |
| Version | `1.6.0.0` |
| Architectures / family | `x64`, `arm64` / `Windows.Desktop` |

## Store listings

Use the separate English (United States), French (France) and German (Germany) blocks in
[store-listing.md](store-listing.md). Complete German review and screenshots before adding
that listing if it is not already present. For each language, paste:

1. **Description**: full paragraphs, up to 10,000 characters.
2. **Features**: one supplied line per entry, at most 20 entries of 200 characters each.
3. **Short description**: the supplied paragraph, kept below 270 characters for compact views.
4. **What's new**: the 1.6.0 block, under 1,500 characters.
5. **Additional system requirements**: separate supplied lines, each under 200 characters.
6. New packaged-app screenshots with matching localized captions.

Whisper and useful live captions lead the copy. Each full description discloses the initial
model download and optional paid cloud dependencies in its opening paragraph. The demo
appears only as a secondary, explicitly scripted feature.

## Submission options

Paste [Notes for certification](store-listing.md#notes-for-certification). Check that the
model download works from a clean install. Resolve any required cloud test credentials
privately before submission; no personal keys belong in public metadata or screenshots.

A clean install now opens on Whisper, Base and automatic language detection. Verify that
Download model requires a click and that Start remains unavailable until a model is ready.
For the setup-free fallback, explicitly select Built-in demo from the engine list. An existing
installation keeps its previously saved engine and audio choices.

Suggested `runFullTrust` justification:

> Win32 desktop app packaged as MSIX. The full-trust process handles microphone/WASAPI
> capture, local CPU Whisper inference and temporary audio buffering, verified model downloads,
> Windows Credential Manager, a click-through overlay and transcript export. No driver,
> service, auto-start task, telemetry or developer relay.

## Final review and submission

Confirm all sections are complete, packages and copy agree on 1.6.0.0, screenshots match the
final UI, and the privacy page is current. Test both credential-free routes: Whisper for real
speech after download, and the scripted demo for a setup-free display check. Neither route
substitutes for testing the advertised cloud features.

Submit manually when the [handoff checklist](store-updates.md#release-160-handoff) is complete.
Record the submission date and certification result there. Do not mark 1.6.0 live until
Partner Center or the public listing confirms publication.
