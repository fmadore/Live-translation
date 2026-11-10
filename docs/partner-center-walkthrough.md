# Partner Center submission walkthrough — 1.13.0

Target: **Live Translation & Subtitles 1.13.0**, Product ID `9PFB8LR3RR9X`. **Not yet tagged.**
The Store serves 1.12.0 (MSIX 1.12.0.0), live as reported by the maintainer on 10 November
2026; 1.13.0.0 replaces it. Keep this walkthrough as the template for the
next submission. The combined x64 + ARM64 bundle will be at
`https://github.com/fmadore/Live-translation/releases/download/v1.13.0/Live.Translation_1.13.0.msixbundle`.
The [release handoff](store-updates.md#release-1130-handoff) has the package evidence and
outstanding acceptance checks.

1.13.0 runs Whisper on a suitable graphics processor through `whisper-vulkan.exe`, a helper
program in the package, skips silence with a bundled speech-detection model, offers Large v3
Turbo, sends names and terms to Gemini subtitles too, and asks once for each cloud provider
before audio is first sent to it. Beyond What's new it changes three things in Partner Center:
the certification notes describe the helper, the bundled model and the question; the
descriptions' offline-subtitles paragraph, a sentence on names and terms and one on the question
change, with three feature lines and the second additional system requirement; and the package
carries a second executable, which WACK and certification will see. It keeps everything 1.12.0
brought, already in the Store: the published **privacy policy**'s Zoom section,
the certification notes' **paragraph on Zoom** and each description's **trademark line** (see
[Captions in Zoom](#captions-in-zoom-privacy-notes-and-trademark) and
[store-listing-next.md](store-listing-next.md)). There is no new product declaration: the one
ticked for 1.11.0's support link stays ticked. Paste the descriptions, features, What's new, the
additional system requirements and the certification notes again from
[store-listing.md](store-listing.md); the short descriptions and keywords are unchanged. What's
new covers the changes since 1.12.0; the certification notes already name 1.13.0.0.

## Before Partner Center

1. Verify the unsigned `Live.Translation_1.13.0.msixbundle` contains both native packages at
   **1.13.0.0**, with the assigned [Store identity](microsoft-store.md#store-identity-assigned),
   and that both carry the wave-to-words `Square44x44Logo`, `Square150x150Logo`,
   `Wide310x150Logo` and `StoreLogo` assets, the
   [icon variants](packaging-msix.md#icon-variants-and-resourcespri) and `resources.pri`, and
   that each package holds `whisper-vulkan.exe` and `models/ggml-silero-v5.1.2.bin`.
2. Test signed per-architecture MSIX packages on x64 and ARM64. Use
   [packaging instructions](packaging-msix.md); the side-by-side Whisper Test app is separate
   evidence and must not be uploaded as the Store package.
3. Complete the [acceptance checklist](microsoft-store.md#acceptance-checklist), including
   offline Whisper transcription and translation after download, cloud features, the
   Transcript tab and Copy, the theme in Light, Dark and System, Stop/backlog, overlay
   placement across launches and move mode, system audio under load, the support link opening
   the default browser, Captions in Zoom, Whisper on the graphics processor and its fallback,
   speech detection, Large v3 Turbo, the question before audio first goes to a cloud provider,
   export and WACK.
4. The current [screenshots](store-screenshots/README.md) do not show the *Zoom captions*
   button (1.12.0) or 1.13.0's setup switches, disabled Start and custom-colour chips. Decide
   whether to retake them; if so, retake them from the 1.13.0 MSIX in all three listing
   languages, with Settings › App › Theme set to **Dark** whatever mode the PC is in, so the set shows one theme, and keep any Zoom
   window and Zoom's artwork out of them. The 21 October 2026 set, from the 1.7.1 MSIX,
   predates the Live and Transcript tabs.
5. Check the published [privacy policy](https://fmadore.github.io/Live-translation/privacy)
   reads "5 November 2026" and has the *Optional captions in Zoom* section, the *macOS*
   section, and the Zoom line and the question asked before audio first goes to a provider
   under *Third-party processing*. 1.13.0 changes nothing in it. It is published from
   `docs/privacy.md` on `main`.

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
| Personal-information transmission | Cloud modes send selected audio directly to third parties, after the app asks once for each provider, and Mistral and Gemini subtitles also receive any names and terms entered; local Whisper sends nothing. From 1.12.0, Captions in Zoom, once the user starts it, sends caption text (never audio), including Whisper's, directly to Zoom |
| Recommended memory | 8 GB; Large v3 Turbo takes about 1 GB more; model, processor and graphics processor determine Whisper speed |

### Product declarations: the support link

Store Policy 10.8.2 lets a voluntary donation that gives nothing in return go through a secure
third-party payment provider, and says: "You must note the use of a secure third-party purchase
API in Partner Center during the submission process." Since 1.11.0 it is noted in two places,
and 1.13.0 keeps both:

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
   it serves shows the 5 November 2026 date and the *Optional captions in Zoom* section (step 5
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
`Live.Translation_1.13.0.msixbundle`. The Store signs accepted packages. Keep locally signed
test copies separate; do not upload an NSIS, MSI or Whisper Test artifact here.

| Manifest element | Value |
| --- | --- |
| Identity name | `49346FMadore.LiveTranslationSubtitles` |
| Publisher | `CN=5D0ECC96-3998-452E-B7E9-29BE9B576F86` |
| Publisher display name | `FMadore` |
| Version | `1.13.0.0` |
| Architectures / family | `x64`, `arm64` / `Windows.Desktop` |

## Store listings

Use the separate English (United States), French (France) and German (Germany) blocks in
[store-listing.md](store-listing.md). Complete German review and screenshots before adding
that listing if it is not already present. For each language, paste:

1. **Description**: full paragraphs, up to 10,000 characters. 1.13.0 replaces the
   offline-subtitles paragraph, ends the cloud paragraph with Gemini, Mistral and Whisper taking
   the names-and-terms list, and adds a sentence on the question before audio is sent to the
   privacy paragraph. 1.12.0's Zoom sentences and the trademark line, the last paragraph, stay.
2. **Features**: one supplied line per entry, at most 20 entries of 200 characters each. 1.13.0
   changes the first, second and fifth lines; the twentieth, on Captions in Zoom, stays.
3. **Short description**: the supplied paragraph, kept below 270 characters for compact views.
4. **What's new**: the 1.13.0 block, under 1,500 characters. It covers the changes since
   1.12.0, the version the Store serves.
5. **Additional system requirements**: separate supplied lines, each under 200 characters.
   1.13.0 changes the second (Whisper: 31–547 MiB, processor or graphics processor).
6. Packaged-app screenshots with matching localized captions, showing the current interface
   in the Dark theme.
7. **Store logos**: see below.
8. **16:9 Super hero art**, below the Store logos under **Trailers and additional assets** ›
   **Windows and Xbox image** (in the French interface: *Bandes-annonces et ressources
   supplémentaires* › *Image de Windows et de Xbox* › *Image de super héros 16:9*; expand the
   section if its details are hidden): upload
   [`super-hero-art.png`](store-screenshots/super-hero-art.png) (1920 × 1080). It has no text,
   so the same file goes into every listing language. Leave the **Xbox images** slots empty:
   their 1920 × 1080 titled hero art must carry the product title.

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
first paragraph on the helper, the bundled model and the question before audio is sent, the
second on Captions in Zoom and the third on the support link. Resolve any
required cloud test credentials privately before submission; no personal keys belong in public
metadata or screenshots.

A clean install opens on Whisper, Base and automatic language detection. Verify that the model
download works, requires a click, and that Start stays unavailable until a model is ready. The
setup-free fallback is Built-in demo, selected explicitly from the engine list, or selected for
the reviewer on an x64 processor without AVX2, where Whisper is listed as unavailable. An existing
installation keeps its saved engine and audio choices.

Suggested `runFullTrust` justification:

> Win32 desktop app packaged as MSIX. The full-trust process handles microphone/WASAPI capture, local Whisper transcription and translation with temporary audio buffering (on the processor, or on the graphics processor through the bundled whisper-vulkan.exe helper it starts), verified model downloads, Windows Credential Manager, a click-through overlay and transcript export. No driver, service, auto-start task, telemetry or developer relay.

## Final review and submission

Confirm all sections are complete, the Product declaration for purchases outside Microsoft
commerce is still ticked, each description ends with the Zoom trademark line, packages and copy
agree on 1.13.0.0, screenshots and Store
logos match the final UI and the wave-to-words icon, each listing carries the super hero art,
and the privacy page is current. Test both
credential-free routes: Whisper for real speech after download, and the scripted demo for a
setup-free display check. Neither substitutes for testing the advertised cloud features.

Rollout ([route step 9](store-updates.md#the-route)): **gradual**, starting at 10%. 1.13.0
changes capture (wasapi 0.27) and the session lifecycle (the question before a cloud session,
the echo filter's held lines, the Whisper helper's start, fallback and end).

Submit manually when the [handoff checklist](store-updates.md#release-1130-handoff) is complete,
and record the submission date and certification result there. Do not mark a version live
until Partner Center or the public listing confirms publication. 1.12.0 is live as reported by
the maintainer on 10 November 2026; its submission and certification dates were not recorded.
