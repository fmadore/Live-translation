# Partner Center submission walkthrough — 1.6.1

Target: **Live Translation & Subtitles 1.6.1**, Product ID `9PFB8LR3RR9X`. Published on
GitHub on 6 October 2026, not yet submitted; the Store serves 1.6.0 (MSIX 1.6.0.0), confirmed
live by the maintainer on 6 October 2026. Download the combined x64 + ARM64 bundle,
[Live.Translation_1.6.1.msixbundle](https://github.com/fmadore/Live-translation/releases/download/v1.6.1/Live.Translation_1.6.1.msixbundle)
(11,195,249 bytes, SHA-256 `86ef814f…a6ccea0`).
The [release handoff](store-updates.md#release-161-handoff) has the package evidence and
outstanding acceptance checks.

1.6.1 is a maintenance and visual release: a new app icon and a refreshed interface, with no
new features, runtime dependency or privacy changes. The listing keeps its positioning; paste
every field again from [store-listing.md](store-listing.md), which notes what changed. The
screenshots and any separately uploaded Store logo also change.

## Before Partner Center

1. Verify the unsigned `Live.Translation_1.6.1.msixbundle` contains both native packages at
   **1.6.1.0**, with the assigned [Store identity](microsoft-store.md#store-identity-assigned),
   and that both carry the new wave-to-words `Square44x44Logo`, `Square150x150Logo`,
   `Wide310x150Logo` and `StoreLogo` assets.
2. Test signed per-architecture MSIX packages on x64 and ARM64. Use
   [packaging instructions](packaging-msix.md); the side-by-side Whisper Test app is separate
   evidence and must not be uploaded as the Store package.
3. Complete the [acceptance checklist](microsoft-store.md#acceptance-checklist), including
   offline Whisper after download, cloud features, Stop/backlog, overlay, export and WACK.
4. Refresh [screenshots](store-screenshots/README.md) in all three listing languages: the
   icon and interface changed in 1.6.1, and the 1.6.0 capture is still pending.
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
`Live.Translation_1.6.1.msixbundle`. The Store signs accepted packages. Keep locally signed
test copies separate; do not upload an NSIS, MSI or Whisper Test artifact here.

| Manifest element | Value |
| --- | --- |
| Identity name | `49346FMadore.LiveTranslationSubtitles` |
| Publisher | `CN=5D0ECC96-3998-452E-B7E9-29BE9B576F86` |
| Publisher display name | `FMadore` |
| Version | `1.6.1.0` |
| Architectures / family | `x64`, `arm64` / `Windows.Desktop` |

## Store listings

Use the separate English (United States), French (France) and German (Germany) blocks in
[store-listing.md](store-listing.md). Complete German review and screenshots before adding
that listing if it is not already present. For each language, paste:

1. **Description**: full paragraphs, up to 10,000 characters.
2. **Features**: one supplied line per entry, at most 20 entries of 200 characters each.
3. **Short description**: the supplied paragraph, kept below 270 characters for compact views.
4. **What's new**: the 1.6.1 block, under 1,500 characters.
5. **Additional system requirements**: separate supplied lines, each under 200 characters.
6. New packaged-app screenshots with matching localized captions, showing the new icon and
   refreshed interface.
7. **Store logos**: see below.

The package's tiles and `StoreLogo` carry the new wave-to-words icon, so the bundle updates
the icon the Store takes from the package, unless Partner Center's optional **1:1 App tile icon
(300 × 300)** has been uploaded, which overrides it. In each listing language's **Store logos**
section, replace any uploaded app tile icon with a 300 × 300 PNG of the new mark (rendered
from `src-tauri/icons/source.svg`), or remove it so the Store uses the package image. Check any
uploaded promotional art for the old glyph too.

Whisper and useful live captions lead the copy. Each full description discloses the initial
model download and optional paid cloud dependencies in its opening paragraph. The demo
appears only as a secondary, explicitly scripted feature.

## Submission options

Paste [Notes for certification](store-listing.md#notes-for-certification). Resolve any
required cloud test credentials privately before submission; no personal keys belong in public
metadata or screenshots.

A clean install opens on Whisper, Base and automatic language detection. Verify that the model
download works, requires a click, and that Start stays unavailable until a model is ready. The
setup-free fallback is Built-in demo, selected explicitly from the engine list. An existing
installation keeps its saved engine and audio choices.

Suggested `runFullTrust` justification:

> Win32 desktop app packaged as MSIX. The full-trust process handles microphone/WASAPI capture, local CPU Whisper inference and temporary audio buffering, verified model downloads, Windows Credential Manager, a click-through overlay and transcript export. No driver, service, auto-start task, telemetry or developer relay.

## Final review and submission

Confirm all sections are complete, packages and copy agree on 1.6.1.0, screenshots and Store
logos match the final UI and new icon, and the privacy page is current. Test both
credential-free routes: Whisper for real speech after download, and the scripted demo for a
setup-free display check. Neither substitutes for testing the advertised cloud features.

Submit manually when the [handoff checklist](store-updates.md#release-161-handoff) is complete,
and record the submission date and certification result there. Do not mark 1.6.1 live until
Partner Center or the public listing confirms publication.
