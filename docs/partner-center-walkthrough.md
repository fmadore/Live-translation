# Partner Center submission walkthrough — 1.12.0

Target: **Live Translation & Subtitles 1.12.0**, Product ID `9PFB8LR3RR9X`. **Tagged on
27 October 2026; not yet submitted.** The Store serves 1.11.0 (MSIX 1.11.0.0), accepted as
reported by the maintainer on 26 October 2026. Keep this walkthrough as the template for the
next submission. The combined x64 + ARM64 bundle is at
`https://github.com/fmadore/Live-translation/releases/download/v1.12.0/Live.Translation_1.12.0.msixbundle`.
The [release handoff](store-updates.md#release-1120-handoff) has the package evidence and
outstanding acceptance checks.

1.12.0 adds **Captions in Zoom**, which shares one caption language with a Zoom meeting's own
caption bar through the caption link (API token) the host copies in Zoom, and fixes flashes and
layout shifts in setup and the transcript; the pre-flight now says why Start waits for an
application. It is the first version that can send caption text to a third party the user
names, and the first whose listing names another company's product, so beyond What's new it
changes three things in Partner Center: the published **privacy policy** must show its Zoom
section, the certification notes gain a **paragraph on Zoom**, and each description gains a
**trademark line** (see [Captions in Zoom](#captions-in-zoom-privacy-notes-and-trademark) and
[store-listing-next.md](store-listing-next.md)). There is no new product declaration: the one
ticked for 1.11.0's support link stays ticked. Paste the descriptions, features, What's new and
the certification notes again from [store-listing.md](store-listing.md); the short
descriptions, keywords and additional system requirements are unchanged. What's new covers the
changes since 1.11.0; the certification notes already name 1.12.0.0.

## Before Partner Center

1. Verify the unsigned `Live.Translation_1.12.0.msixbundle` contains both native packages at
   **1.12.0.0**, with the assigned [Store identity](microsoft-store.md#store-identity-assigned),
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
   the default browser, Captions in Zoom, export and WACK.
4. The toolbar gains a *Zoom captions* button, which the current
   [screenshots](store-screenshots/README.md) do not show. Decide whether to retake them; if so,
   retake them from the 1.12.0 MSIX in all three listing languages, with Settings › App › Theme
   set to **Dark** whatever mode the PC is in, so the set shows one theme, and keep any Zoom
   window and Zoom's artwork out of them. The 21 October 2026 set, from the 1.7.1 MSIX,
   predates the Live and Transcript tabs.
5. Check the published [privacy policy](https://fmadore.github.io/Live-translation/privacy)
   reads "27 October 2026 (documents the 1.12.0 release)" and has the *Optional captions in
   Zoom* section and the Zoom line under *Third-party processing*. It is published from
   `docs/privacy.md` on `main`, so it updates when the release PR merges.

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
| Purchases outside Microsoft commerce | **Ticked since 1.11.0; leave it ticked** (Product declarations, below): the Buy Me a Coffee link; provider dependencies and charges disclosed in listing. Captions in Zoom involve no payment |
| Accessibility certification | Do not claim certification without a completed audit |
| Generative AI | Yes: optional Gemini/OpenAI translation and local Whisper translation into English generate text |
| Personal-information transmission | Cloud modes send selected audio directly to third parties, and Mistral also receives any names and terms entered for it; local Whisper does not. From 1.12.0, Captions in Zoom, once the user starts it, sends caption text (never audio), including Whisper's, directly to Zoom |
| Recommended memory | 8 GB; model/CPU determine Whisper speed |

### Product declarations: the support link

Store Policy 10.8.2 lets a voluntary donation that gives nothing in return go through a secure
third-party payment provider, and says: "You must note the use of a secure third-party purchase
API in Partner Center during the submission process." Since 1.11.0 it is noted in two places,
and 1.12.0 keeps both:

1. **Properties › Product declarations** (*Propriétés › Déclarations de produit*): tick
   **This product allows users to make purchases, but does not use the Microsoft Store commerce
   system**. Microsoft's
   [product declarations guide](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/product-declarations)
   says to tick it for any app that does not use Microsoft Commerce. Leave the other
   declarations as they were.
2. **Submission options › Notes for certification**: the third paragraph of the
   [notes](store-listing.md#notes-for-certification) explains the link, names Buy Me a Coffee as
   the provider and states that nothing is unlocked.

The app itself already meets the rest of 10.8.2: the payment happens in the browser after
installation, on Buy Me a Coffee's own site, which identifies itself, authenticates the
supporter and confirms the payment; the app collects no payment or financial information. The
descriptions say the link is optional and unlocks nothing (10.8.4). Do not add a Store add-on
or in-app purchase for this release.

### Captions in Zoom: privacy, notes and trademark

Captions in Zoom needs no product declaration, but three checks in this submission:

1. **Privacy policy URL** (Store Policy 10.5): the URL above is unchanged; check that the page
   it serves shows the 27 October 2026 date and the *Optional captions in Zoom* section (step 5
   of [Before Partner Center](#before-partner-center)).
2. **Notes for certification**: the second paragraph of the
   [notes](store-listing.md#notes-for-certification) says what the feature sends and where, that
   it is off until the user starts it, how a tester can exercise it in a Zoom meeting they host
   (a free Basic account is enough) and how to skip it by pasting a non-Zoom address, which the
   app refuses without sending anything.
3. **Trademark** (Policies 10.1 and 11.2): each description ends with the line that Zoom is a
   trademark of Zoom Communications, Inc. and that the app is not affiliated with or endorsed by
   Zoom, in its language. Zoom is named only descriptively, the app shows no Zoom logo, and
   Zoom is not added to the keywords.

Review the current age-rating questionnaire for third-party connections: cloud providers and
user-requested model downloads use the network. Microphone hardware is needed only for
microphone capture; system capture and the demo do not require one.

## Packages

Replace the previous submission packages with the single unsigned multi-architecture
`Live.Translation_1.12.0.msixbundle`. The Store signs accepted packages. Keep locally signed
test copies separate; do not upload an NSIS, MSI or Whisper Test artifact here.

| Manifest element | Value |
| --- | --- |
| Identity name | `49346FMadore.LiveTranslationSubtitles` |
| Publisher | `CN=5D0ECC96-3998-452E-B7E9-29BE9B576F86` |
| Publisher display name | `FMadore` |
| Version | `1.12.0.0` |
| Architectures / family | `x64`, `arm64` / `Windows.Desktop` |

## Store listings

Use the separate English (United States), French (France) and German (Germany) blocks in
[store-listing.md](store-listing.md). Complete German review and screenshots before adding
that listing if it is not already present. For each language, paste:

1. **Description**: full paragraphs, up to 10,000 characters. 1.12.0 adds to each a sentence
   on Captions in Zoom, one on the caption text it sends in the privacy paragraph, one on the
   host's Zoom settings in the requirements paragraph, and the trademark line as the last
   paragraph.
2. **Features**: one supplied line per entry, at most 20 entries of 200 characters each. 1.12.0
   adds a twentieth line, on Captions in Zoom.
3. **Short description**: the supplied paragraph, kept below 270 characters for compact views.
4. **What's new**: the 1.12.0 block, under 1,500 characters. It covers the changes since
   1.11.0, the version the Store serves.
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
second paragraph on Captions in Zoom and the third on the support link. Resolve any
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
commerce is still ticked, each description ends with the Zoom trademark line, packages and copy
agree on 1.12.0.0, screenshots and Store
logos match the final UI and the wave-to-words icon, and the privacy page is current. Test both
credential-free routes: Whisper for real speech after download, and the scripted demo for a
setup-free display check. Neither substitutes for testing the advertised cloud features.

Rollout ([route step 9](store-updates.md#the-route)): audio capture is untouched in 1.12.0,
but every caption and each session start now pass through the Zoom share; decide between a full
and a gradual rollout.

Submit manually when the [handoff checklist](store-updates.md#release-1120-handoff) is complete,
and record the submission date and certification result there. Do not mark a version live
until Partner Center or the public listing confirms publication. 1.11.0's acceptance was
reported by the maintainer on 26 October 2026.
