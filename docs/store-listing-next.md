# Next Store submission — 1.13.0

**Tagged 10 November 2026; not yet submitted.** The Store serves **1.12.0.0**, live as reported by the maintainer on
10 November 2026; the submission and certification dates were not recorded. The next
submission is **1.13.0.0**, replacing it: one unsigned bundle with native x64 and ARM64
packages, at
`https://github.com/fmadore/Live-translation/releases/download/v1.13.0/Live.Translation_1.13.0.msixbundle`
once the tag is built.

What's new and the certification notes' first paragraph cover the changes since 1.12.0. What
1.12.0 brought to the listing is already in Partner Center and stays.

The positioning is unchanged. Lead the listing with offline Whisper captions, the transparent
overlay and optional cloud translation; the scripted demo is a secondary way to try the display
controls. New installations open on Whisper / Base / automatic detection, and existing saved
setups are preserved. Model download and capture remain explicit user actions. The Store
listing is for Windows: 1.13.0 also ships a Mac app, which the listing does not mention.

## What 1.13.0 adds

From the [release notes](release-1.13.0.md): Whisper on the graphics processor (a Vulkan helper
on Windows), bundled speech detection, loop and credit filters and Large v3 Turbo
([#208](https://github.com/fmadore/Live-translation/pull/208),
[local Whisper notes](local-whisper.md#graphics-processor)); names and terms for Gemini
Transcribe and Whisper ([#193](https://github.com/fmadore/Live-translation/pull/193)); a
question asked once for each cloud provider before audio is first sent to it (in
[#202](https://github.com/fmadore/Live-translation/pull/202)); the switch that leaves out the
call heard by the room microphone with Both sources
([#212](https://github.com/fmadore/Live-translation/pull/212)); and the fixes of the
[10 November app review](app-review-2026-11-10.md)
([#213](https://github.com/fmadore/Live-translation/pull/213), open). The macOS port in the same
release does not reach the Microsoft Store.

## What the submission needs

In the Store since 1.12.0, and kept:

- **Privacy policy (Store Policy 10.5, personal information).** Caption text can contain
  personal information, and Captions in Zoom sends it to Zoom once the user starts sharing. The
  published [privacy policy](privacy.md) describes that flow in *Optional captions in Zoom* and
  in the Zoom line under *Third-party processing*. Sharing is opt-in by construction: nothing is
  sent until the user pastes a link and chooses **Start sharing**, and the dialog says, before
  that button, that caption text goes to Zoom even when Whisper makes it.
- **Notes for certification.** The second paragraph of the notes in
  [store-listing.md](store-listing.md#notes-for-certification) says what Captions in Zoom sends
  and where, that it is off until the user starts it, and how a tester can exercise it (a Zoom
  meeting they host; a free Basic account is enough, confirmed by the maintainer on 27 October
  2026) or skip it (pasting a non-Zoom address, which the app refuses without sending anything).
- **Trademark (Policies 10.1 and 11.2).** Zoom is named only descriptively; the app shows no Zoom
  logo, and neither the screenshots nor the logo use Zoom's artwork. Each description ends with
  "Zoom is a trademark of Zoom Communications, Inc. Live Translation & Subtitles is not affiliated
  with or endorsed by Zoom." in its language. Zoom is not added to the keywords.
- **Descriptions and features.** The Zoom sentences in the overlay, privacy and requirements
  paragraphs, and the twentieth feature line, the most Partner Center allows.

New in 1.13.0:

- **A second executable in the package.** `whisper-vulkan.exe` sits beside the app's
  executable. It is part of the same full-trust app, started and stopped only by it, opens no
  network connection, and needs no new capability: it reaches the graphics processor through
  the graphics driver already installed. The certification notes say so.
- **A bundled model.** `models/ggml-silero-v5.1.2.bin` (864 KiB, Silero VAD, MIT) ships in the
  package. The Whisper speech models are still downloaded only at the user's request; Large v3
  Turbo is a new one, about 547 MiB.
- **The question before audio first goes to a cloud provider.** It names the provider, says
  what is sent and under whose terms, gives the privacy policy's address and waits for **Send**;
  **Cancel**, which has the focus, starts nothing. The answer is kept on the device for that
  provider. A certification tester with cloud credentials meets it on the first cloud session,
  so the notes' *Optional cloud features* paragraph describes it.
- **Names and terms to Gemini.** The list now goes to Gemini Transcribe as well as Mistral;
  Whisper keeps it on the PC. The certification notes say so.
- **Privacy policy.** Its effective date is **5 November 2026**, and it already covers the macOS
  app, the question before audio first goes to each provider, names and terms for Gemini, and
  the Vulkan helper, which uses only the computer's own memory. It also keeps 1.12.0's Zoom
  sections, so 1.13.0 changes nothing in it. Check that the published page shows that date
  before submitting.
- **Screenshots.** The current images are from 1.7.1. 1.13.0 changes what they would show: the
  setup gains *Use the graphics processor* under the model and, with Both sources, *Leave out the
  call heard by the room mic*; a Start that cannot be pressed has a neutral look; the
  custom-colour chips in Settings › Captions moved. The toolbar has had *Zoom captions* since
  1.12.0. See the [screenshot plan](store-screenshots/README.md).

No product declaration changes: the purchases-outside-Microsoft-commerce box ticked for 1.11.0
stays ticked, and neither Zoom captions nor anything in 1.13.0 involves a payment. The MSIX
capabilities are unchanged (`runFullTrust` and `microphone`); `AppxManifest.xml.template` and
`bundle.config.json` are unchanged since `v1.12.0`.

## After 1.13.0: DeepL

The release after 1.13.0 adds **DeepL** (DeepL Voice API, by DeepL SE, Cologne) as a fourth
cloud engine, beside Google Gemini, OpenAI and Mistral. It serves live translation only:
DeepL hears which language is spoken and writes the caption language the operator chooses, in
42 of the app's caption languages ([coverage](language-coverage.md)). Its listing copy, to be
written with that release's [store-listing.md](store-listing.md):

- **Descriptions.** The optional-cloud-translation sentence names DeepL with the other
  providers, under the user's own DeepL API key. Say plainly that DeepL needs a **paid DeepL
  API plan**: Voice is not on DeepL's free API, and the app refuses a free key (one ending in
  `:fx`) with its own message. Nothing in the copy may suggest a free DeepL tier.
- **Features.** One line, in place of an existing one (the list is at Partner Center's maximum
  of twenty): live translation through Gemini, OpenAI or DeepL with the user's own key.
- **Privacy policy.** Its effective date becomes **11 November 2026**: DeepL is named as a
  recipient (the audio of the chosen sources while a DeepL session runs, and the caption
  language), with DeepL's privacy policy linked. The question before audio first goes to a
  provider asks about DeepL on its own. If 1.13.0 is submitted after this change reaches
  `main`, the published policy already shows the new date and names DeepL, which that package
  does not contain; that is harmless, but say so to the maintainer.
- **Certification notes.** The *Optional cloud features* paragraph adds DeepL: a tester needs a
  key from a paid DeepL API plan to exercise it, and can see the refusal of a free key without
  one.
- **Keywords.** Consider "DeepL" only if a keyword slot is free; it is named descriptively, as
  the other providers are.

## The copy

[store-listing.md](store-listing.md) is the 1.13.0 copy, with every block in English, French and
German: What's new, the descriptions' new offline-subtitles paragraph and the sentences on names
and terms and on the question before audio goes to a provider, the changed feature lines, the
second requirements line and the certification notes. The French and German sentences new in
1.13.0 need the maintainer's review.

- [EN/FR/DE descriptions, What's new and certification notes](store-listing.md)
- [Release notes](release-1.13.0.md)
- [Build evidence and acceptance checklist](store-updates.md#release-1130-handoff)
- [Captions in Zoom guide](zoom-captions.md), which the app's dialog opens
- [Screenshot plan](store-screenshots/README.md)
- [Partner Center walkthrough](partner-center-walkthrough.md)

Before submission, complete:

- the validation tag and the maintainer's test of the Local Test build, as itemised in the
  [handoff](store-updates.md#release-1130-handoff);
- the final-package checks, including `whisper-vulkan.exe` and the bundled model in both
  packages, and the checks still open from earlier releases;
- the screenshot decision;
- the German and French copy review of the 1.13.0 sentences;
- Windows App Certification Kit on the final packages, the first with a second executable.

Use the normal Store identity: the side-by-side Local Test installation is for feedback, not the
Store submission package.
