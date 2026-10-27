# Microsoft Store submission

**Current Store release: 1.11.0 / MSIX 1.11.0.0**, accepted as reported by the maintainer on
26 October 2026, the day it was tagged; it replaced 1.10.0. It added an optional Buy Me a
Coffee link that unlocks nothing, the app's first way to pay anyone, so its submission ticked a
Product declaration and explained the link in the certification notes
([walkthrough](partner-center-walkthrough.md#product-declarations-the-support-link)); that
declaration stays ticked.

**Next submission: 1.12.0 / MSIX 1.12.0.0**, tagged on 27 October 2026 and not yet submitted.
1.12.0 adds Captions in Zoom, which shares one caption language with a Zoom meeting's own
caption bar, and fixes flashes and layout shifts in setup and the transcript. It is the first
version that can send caption text to a third party the user names, and the first whose listing
names another company's product, so its submission checks the published privacy policy's Zoom
section, explains the feature in the certification notes and carries a trademark line in each
description ([what it needs](store-listing-next.md)).
See the [1.12.0 handoff](store-updates.md#release-1120-handoff),
[listing copy](store-listing.md) and [Partner Center walkthrough](partner-center-walkthrough.md).

## Product and certification approach

Whisper gives real local speech recognition without an account or API key: the user downloads
a verified multilingual model once, then captions microphone or system audio offline, in the
spoken language or translated into English. CPU speed, model choice and language affect
latency and accuracy. Optional cloud translation and subtitles
need the user's provider account, key and network connection, with possible usage charges.
Every localized description discloses these dependencies at the start.

New installations open on **Whisper**, with Base and automatic language detection selected.
The user chooses Download model and then Start; no download or capture starts automatically.
Existing saved setups are preserved on upgrade. The English/French scripted demo remains a
setup-free way to check the overlay and export flow, and must always be labelled as a demo,
never as speech recognition.

1.0.3 failed certification because Start Subtitles did not work on the review device. 1.0.5
removed Windows AI Speech/ML and the Windows speech recognizer, and introduced the demo. Whisper
(1.6.0) is a different CPU implementation, so those failures do not describe it.

Since 1.7.0, Whisper is built for AVX2 on x64 and for dot-product instructions on ARM64, and
checks the processor before any of its code runs. On an x64 processor without AVX2, such as
some budget Celeron and Pentium Silver PCs, Whisper is listed as unavailable with the reason and
a selected Whisper, including the first-launch default, becomes the built-in demo. A clean
install on a reviewer's PC therefore always opens on something that starts. The listing
requirements and certification notes say so; see
[processor requirements](local-whisper.md#processor-requirements).

Review the current [Microsoft Store policies](https://learn.microsoft.com/en-us/windows/apps/publish/store-policies)
when submitting: metadata must describe dependencies and costs accurately, advertised features
must work on supported devices, and reviewers need a usable test path. The certification notes
provide Whisper setup and the demo fallback. Resolve access needed to test cloud features
before submission; a passing local check does not imply certification.

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

## Acceptance checklist

- [ ] Manifests and lockfiles agree on 1.12.0; both MSIX manifests and the combined bundle
  report 1.12.0.0 and the assigned identity, and both packages carry the wave-to-words icon
  assets introduced in 1.6.1.
- [ ] The taskbar, Start and Alt+Tab show the installed app's icon without a plate, with the
  taskbar in the dark and the light theme; each package carries `resources.pri` and the
  [icon variants](packaging-msix.md#icon-variants-and-resourcespri).
- [ ] Both signed native packages are installed and exercised on their target hardware.
- [ ] Clean-install Whisper model download, verification, cancellation/removal and offline
  restart work without API keys. Test English and French; record non-Latin language checks.
- [ ] First launch selects Whisper / Base / automatic detection without starting a download
  or audio capture. An upgrade preserves an existing cloud or scripted-demo selection. On an
  x64 PC without AVX2, if one is available, Whisper is listed as unavailable and the demo is
  selected.
- [ ] Whisper translation into English with Small works offline on x64 and ARM64; any caption
  language but English is refused.
- [ ] Real microphone, system and application capture work; a failed source preserves captions.
- [ ] Whisper Pause admits no new audio; existing backlog can finish. Stop drains; confirmed
  discard shows the incomplete-transcript notice. Export after completion contains the tail.
- [ ] Cloud transcription and translation work with suitable credentials, including a
  non-EN/FR target, two targets, original speech where supplied, and Pause/Resume. Mistral
  subtitles on Voxtral Realtime 3 start with a names-and-terms list, and close lines at the
  speaker's turn ends rather than mid-sentence.
- [ ] The Transcript tab shows the whole session during a run; Copy writes each time window as
  plain text, during a session and after Stop.
- [ ] Settings › App › Theme: Light, Dark, and System while Windows' app mode is switched;
  the title bar follows an explicit choice; a light desktop never shows a dark first frame; a
  contrast theme still takes over. The caption overlay and its move toolbar look the same in
  every theme.
- [ ] Alt+F4 while placing the overlay hides it, the toggle and the tray say so, and Show
  overlay brings it back; an API key with a space or an invisible character is refused when
  saved.
- [ ] Overlay move mode under package identity: drag, edge resize, nudge, Snap to bottom and
  Escape's undo, now that they run through the overlay's own commands.
- [ ] System audio with local Whisper running keeps up without gaps; System audio and the
  microphone list no applications until One application is chosen.
- [ ] The support link in Settings › App and at the foot of the start screen opens
  `https://buymeacoffee.com/fmadore` in the default browser under package identity, not in a
  window of the app, and is not shown during a session; the operator window does not scroll as
  a whole with the names-and-terms list shown.
- [ ] Partner Center: the Product declaration for purchases outside the Microsoft Store
  commerce system is ticked, and the certification notes explain the support link (Store
  Policy 10.8.2).
- [ ] Scripted demo starts with no hardware, network or credentials, in English and French.
- [ ] Overlay placement, its restore across launches and mixed-DPI displays, all reading
  modes, RTL text, filler list, profiles and shortcuts pass the
  [layout](caption-layout.md#release-verification) and [accessibility](accessibility.md) checks.
- [ ] Text/Markdown/SRT/VTT export, history, recovery, quit prompts and tray behavior work under package identity.
- [ ] Windows App Certification Kit passes on the final packages.
- [ ] EN/FR/DE copy, final-package screenshots and the public privacy policy match this version.

Record evidence and exceptions in the [release handoff](store-updates.md#release-1120-handoff).
Previous release checks are historical evidence, not sign-off for new binaries.

## Package identity and signing

The Store signs accepted packages. Local testing needs signed inner MSIX packages and a
trusted matching certificate; signing only the outer bundle is insufficient. Follow
[MSIX packaging](packaging-msix.md) and use a test account/machine if a Store-signed install
would conflict. Do not remove an existing Store installation or its data as a build step.
