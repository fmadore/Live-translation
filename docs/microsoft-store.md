# Microsoft Store distribution

**Published:** 1.1.0, native x64 and ARM64, at
<https://apps.microsoft.com/detail/9PFB8LR3RR9X>. Accepted 27 August 2026; 1.0.5 was the
release before it. Updates go through [`store-automation.md`](store-automation.md).

The package the Store is serving was cut from the `v1.1.0` tag. Work merged to `main` after
that tag — the accessibility pass, the capability split, the French interface, Windows text
scaling — is **not** in it and ships in the next submission.

## Certification objective

The 1.0.3 submission failed policy 10.1.2.10 because **Start Subtitles** was unusable on the
review device. Two attempted credential-free Windows recognizers were not sufficiently portable:

- the experimental Windows AI Speech/ML component crashed natively on the target ARM64 Surface;
- `Windows.Media.SpeechRecognition` depended on Windows privacy consent, installed speech
  languages, network behavior, and a usable default microphone, and still produced no captions
  on the target ARM64 machine.

Version 1.0.5 removes both paths. The default path is now a transparent, deterministic
**Built-in demo** that opens no device, needs no account/key/language pack/network, and drives
the real session status, elapsed clock, level meter, partial/final caption events, overlay,
transcript, Stop flow, and export. Live microphone/system speech remains available through the
user’s chosen Gemini, OpenAI, or Mistral provider.

This is not presented as local speech recognition. The UI, Store descriptions, privacy policy,
screenshots, and certification notes all call it a demonstration and state that it uses bundled
scripted content.

## Why this is the lowest-risk certification path

The relevant current Microsoft Store policies are:

- **10.1 / 10.1.1:** metadata and the first-run experience must accurately represent features
  and limitations.
- **10.1.2:** the product must be fully functional on targeted systems and devices.
- **10.2.4:** a non-integrated dependency used for primary functionality must be disclosed at
  the beginning of the description.
- **10.3:** the product must be testable; required credentials must be supplied in certification
  notes.
- **10.4.1 / 10.4.2:** supported devices must be compatible and the product must remain
  responsive and not close unexpectedly.
- **10.8.3:** products from individual accounts cannot require financial information, which
  explicitly includes API secret keys, for primary functionality.

The 1.0.5 approach addresses the reported failure by removing every environmental dependency
from the default Start action. It also avoids misleading the reviewer: live recognition is a
separate optional provider-dependent capability disclosed in the first sentence of the listing.

Certification can never be guaranteed before Microsoft reviews the exact submission. The main
remaining policy risk is how Microsoft interprets 10.8.3 in relation to optional BYOK live
features. The strongest defensible submission is therefore the present one: the default complete
caption-presentation workflow needs no secret, while all live-provider dependencies and costs are
prominent and accurate.

## Store identity (assigned)

| Manifest element | Value |
| --- | --- |
| `Package/Identity/Name` | `49346FMadore.LiveTranslationSubtitles` |
| `Package/Identity/Publisher` | `CN=5D0ECC96-3998-452E-B7E9-29BE9B576F86` |
| `Package/Properties/PublisherDisplayName` | `FMadore` |

| Store value | Value |
| --- | --- |
| Package Family Name | `49346FMadore.LiveTranslationSubtitles_6yxybgjxsxtpc` |
| Product ID | `9PFB8LR3RR9X` |
| Store URL | <https://apps.microsoft.com/detail/9PFB8LR3RR9X> |

## Checklist for the next update

1.0.5 and 1.1.0 each cleared every item below and were accepted. Work through them again for
each update rather than trusting that a passing package stays passing — 1.0.3 was a working
build that failed on a machine configured differently from the developer's.

- [ ] Version raised in `package.json`, `src-tauri/Cargo.toml` and `src-tauri/tauri.conf.json`,
  all three matching and all three above the published version. All three read `1.1.0` today,
  which is the version now published, so the next submission cannot reuse them.
- [ ] The default Start action still needs no account, key, microphone, language pack or
  network, and still says so in the UI.
- [ ] Nothing in the UI, listing, screenshots or privacy policy describes the bundled
  demonstration as speech recognition.
- [ ] Live-provider dependency still disclosed in the first sentence of the EN and FR
  descriptions (policy 10.2.4).
- [ ] Certification notes in [`store-listing.md`](store-listing.md) still let a reviewer with
  no credentials exercise the app.
- [ ] Signed per-architecture `.msix` installed and manually run on x64 **and** ARM64 —
  [`../scripts/install-local-msix.ps1`](../scripts/install-local-msix.ps1). A bundle cannot be
  side-loaded: signing it does not sign the packages inside.
- [ ] Meter movement, elapsed clock, English and French captions, overlay, Stop and transcript
  export all confirmed under package identity.
- [ ] Windows App Certification Kit run against the final package.
- [ ] Screenshots re-captured if any of them no longer match the build.

## Store package identity and signing

The Store re-signs an accepted MSIX with a Microsoft certificate. Local sideload testing needs a
self-signed certificate whose subject exactly matches the manifest Publisher; install that
certificate into the local trusted store before `Add-AppxPackage`. See
[`packaging-msix.md`](packaging-msix.md).

## Required manual test

From a clean signed install:

1. Launch without changing Windows speech, language, microphone, or network settings.
2. Verify the default cards read **Subtitles**, **Demo audio**, **English**, **Built-in demo**.
3. Click **Start demo subtitles**.
4. Verify Demo status, a moving Demo meter, advancing Elapsed time, partial and final English
   captions in both windows.
5. Stop, save the transcript, select Français, and repeat.
6. Close and reopen the app; repeat after clearing `session.options` or resetting the app so the
   fresh-install path is exercised.
7. Optionally test each live provider with the publisher’s own key, then clear that key before
   capturing/submitting the package.

## Sources

- [Microsoft Store Policies](https://learn.microsoft.com/en-us/windows/apps/publish/store-policies)
- [MSIX certification process](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/app-certification-process)
- [Get your app certified FAQ](https://learn.microsoft.com/en-us/windows/apps/publish/faq/get-your-app-certified)
- [App package requirements](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/app-package-requirements)
- [Package version numbering](https://learn.microsoft.com/en-us/windows/uwp/publish/package-version-numbering)
