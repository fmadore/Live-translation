# Partner Center submission walkthrough — 1.11.0

Target: **Live Translation & Subtitles 1.11.0**, Product ID `9PFB8LR3RR9X`. Prepared, not yet
tagged or submitted. 1.10.0 (MSIX 1.10.0.0) is in certification, as reported by the maintainer
on 26 October 2026; Partner Center takes one submission at a time, so create this one once
1.10.0 is live. Once the tag is published, download the combined x64 + ARM64 bundle from
`https://github.com/fmadore/Live-translation/releases/download/v1.11.0/Live.Translation_1.11.0.msixbundle`.
The [release handoff](store-updates.md#release-1110-handoff) has the package evidence and
outstanding acceptance checks.

1.11.0 adds an optional Buy Me a Coffee link, in Settings › App and at the foot of the start
screen, and fixes an empty band at the bottom of the operator window. It is the first version
through which anyone can be paid, so beyond What's new it changes three things in Partner
Center: a **Product declaration** under [Pricing and properties](#pricing-and-properties), a
second paragraph in the certification notes, and one sentence in each description. Paste the
descriptions, What's new and the certification notes again from
[store-listing.md](store-listing.md); the features, short descriptions and keywords are
unchanged. What's new covers the changes since 1.10.0; the certification notes already name
1.11.0.0.

## Before Partner Center

1. Verify the unsigned `Live.Translation_1.11.0.msixbundle` contains both native packages at
   **1.11.0.0**, with the assigned [Store identity](microsoft-store.md#store-identity-assigned),
   and that both carry the wave-to-words `Square44x44Logo`, `Square150x150Logo`,
   `Wide310x150Logo` and `StoreLogo` assets, the
   [icon variants](packaging-msix.md#icon-variants-and-resourcespri) and `resources.pri`.
2. Test signed per-architecture MSIX packages on x64 and ARM64. Use
   [packaging instructions](packaging-msix.md); the side-by-side Whisper Test app is separate
   evidence and must not be uploaded as the Store package.
3. Complete the [acceptance checklist](microsoft-store.md#acceptance-checklist), including
   offline Whisper transcription and translation after download, cloud features, the
   Transcript tab and Copy, the theme in Light, Dark and System, Stop/backlog, overlay
   placement across launches and move mode, system audio under load, the support link opening
   the default browser, export and WACK.
4. The only visible change in 1.11.0 is a one-line credit at the foot of the start screen,
   which needs no retake. If the 1.9.0 retake of the [screenshots](store-screenshots/README.md)
   is still outstanding, retake them from the 1.11.0 MSIX in all three listing languages, with
   Settings › App › Theme set to **Dark** whatever mode the PC is in, so the set shows one
   theme. The 21 October 2026 set, from the 1.7.1 MSIX, predates the Live and Transcript tabs.
5. Check the published [privacy policy](https://fmadore.github.io/Live-translation/privacy)
   reads "27 October 2026 (documents the 1.11.0 release)" and has the *Support link* section.
   It is published from `docs/privacy.md` on `main`, so it updates when the release PR merges.

## Pricing and properties

Retain the existing free, public Productivity listing and its supported markets. The app
sells no subscriptions, credits or API access; the optional Buy Me a Coffee link is a
voluntary donation that unlocks nothing. Optional cloud usage is billed by the user's
chosen provider. Keep the publisher, product name and identity unchanged.

| Field | Value / review point |
| --- | --- |
| Privacy | `https://fmadore.github.io/Live-translation/privacy` |
| Website | `https://github.com/fmadore/Live-translation` |
| Support | `https://github.com/fmadore/Live-translation/issues` |
| Purchases outside Microsoft commerce | **Tick it from 1.11.0** (Product declarations, below): the Buy Me a Coffee link; provider dependencies and charges disclosed in listing |
| Accessibility certification | Do not claim certification without a completed audit |
| Generative AI | Yes: optional Gemini/OpenAI translation and local Whisper translation into English generate text |
| Personal-information transmission | Cloud modes send selected audio directly to third parties, and Mistral also receives any names and terms entered for it; local Whisper does not |
| Recommended memory | 8 GB; model/CPU determine Whisper speed |

### Product declarations: the support link

Store Policy 10.8.2 lets a voluntary donation that gives nothing in return go through a secure
third-party payment provider, and says: "You must note the use of a secure third-party purchase
API in Partner Center during the submission process." From 1.11.0, note it in two places:

1. **Properties › Product declarations** (*Propriétés › Déclarations de produit*): tick
   **This product allows users to make purchases, but does not use the Microsoft Store commerce
   system**. Microsoft's
   [product declarations guide](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/product-declarations)
   says to tick it for any app that does not use Microsoft Commerce. Leave the other
   declarations as they were.
2. **Submission options › Notes for certification**: the second paragraph of the
   [notes](store-listing.md#notes-for-certification) explains the link, names Buy Me a Coffee as
   the provider and states that nothing is unlocked.

The app itself already meets the rest of 10.8.2: the payment happens in the browser after
installation, on Buy Me a Coffee's own site, which identifies itself, authenticates the
supporter and confirms the payment; the app collects no payment or financial information. The
descriptions say the link is optional and unlocks nothing (10.8.4). Do not add a Store add-on
or in-app purchase for this release.

Review the current age-rating questionnaire for third-party connections: cloud providers and
user-requested model downloads use the network. Microphone hardware is needed only for
microphone capture; system capture and the demo do not require one.

## Packages

Replace the previous submission packages with the single unsigned multi-architecture
`Live.Translation_1.11.0.msixbundle`. The Store signs accepted packages. Keep locally signed
test copies separate; do not upload an NSIS, MSI or Whisper Test artifact here.

| Manifest element | Value |
| --- | --- |
| Identity name | `49346FMadore.LiveTranslationSubtitles` |
| Publisher | `CN=5D0ECC96-3998-452E-B7E9-29BE9B576F86` |
| Publisher display name | `FMadore` |
| Version | `1.11.0.0` |
| Architectures / family | `x64`, `arm64` / `Windows.Desktop` |

## Store listings

Use the separate English (United States), French (France) and German (Germany) blocks in
[store-listing.md](store-listing.md). Complete German review and screenshots before adding
that listing if it is not already present. For each language, paste:

1. **Description**: full paragraphs, up to 10,000 characters. 1.11.0 adds one sentence on the
   support link to each.
2. **Features**: one supplied line per entry, at most 20 entries of 200 characters each.
3. **Short description**: the supplied paragraph, kept below 270 characters for compact views.
4. **What's new**: the 1.11.0 block, under 1,500 characters. It covers the changes since
   1.10.0, which the Store serves once its certification completes.
5. **Additional system requirements**: separate supplied lines, each under 200 characters,
   unchanged since 1.7.1.
6. Packaged-app screenshots with matching localized captions, showing the current interface
   in the Dark theme.
7. **Store logos**: see below.

The package's tiles and `StoreLogo` carry the wave-to-words icon, unchanged since 1.6.1, so the
bundle leaves the icon the Store takes from the package as it is. Since 1.7.1 the package also
carries unplated taskbar sizes of the icon and a `resources.pri`, so Windows no longer draws it
on an accent-coloured plate on the taskbar and in Start. Partner Center's optional
**1:1 App tile icon (300 × 300)**, if uploaded, overrides it: in each listing language's
**Store logos** section, check that any uploaded app tile icon shows the wave-to-words mark. If
it still shows the old glyph, replace it with a 300 × 300 PNG rendered from
`src-tauri/icons/source.svg`, or remove it so the Store uses the package image. Check any
uploaded promotional art for the old glyph too.

Whisper and useful live captions lead the copy. Each full description discloses the initial
model download and optional paid cloud dependencies in its opening paragraph. The demo
appears only as a secondary, explicitly scripted feature.

## Submission options

Paste [Notes for certification](store-listing.md#notes-for-certification), including the
second paragraph on the support link. Resolve any
required cloud test credentials privately before submission; no personal keys belong in public
metadata or screenshots.

A clean install opens on Whisper, Base and automatic language detection. Verify that the model
download works, requires a click, and that Start stays unavailable until a model is ready. The
setup-free fallback is Built-in demo, selected explicitly from the engine list, or selected for
the reviewer on an x64 processor without AVX2, where Whisper is listed as unavailable. An existing
installation keeps its saved engine and audio choices.

Suggested `runFullTrust` justification:

> Win32 desktop app packaged as MSIX. The full-trust process handles microphone/WASAPI capture, local CPU Whisper transcription and translation with temporary audio buffering, verified model downloads, Windows Credential Manager, a click-through overlay and transcript export. No driver, service, auto-start task, telemetry or developer relay.

## Final review and submission

Confirm all sections are complete, the Product declaration for purchases outside Microsoft
commerce is ticked, packages and copy agree on 1.11.0.0, screenshots and Store
logos match the final UI and the wave-to-words icon, and the privacy page is current. Test both
credential-free routes: Whisper for real speech after download, and the scripted demo for a
setup-free display check. Neither substitutes for testing the advertised cloud features.

A gradual rollout ([route step 9](store-updates.md#the-route)) is not needed: neither audio
capture nor the session lifecycle changed in 1.11.0.

Submit manually when the [handoff checklist](store-updates.md#release-1110-handoff) is complete,
and record the submission date and certification result there. Do not mark 1.11.0 live until
Partner Center or the public listing confirms publication.
