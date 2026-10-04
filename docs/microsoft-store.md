# Microsoft Store submission

**Next target: 1.6.0 / MSIX 1.6.0.0, in preparation.** The last confirmed Store release is
1.5.1, live since 23 September 2026. See the [current handoff](store-updates.md#release-160-handoff),
[listing copy](store-listing.md) and [Partner Center walkthrough](partner-center-walkthrough.md).

## Product and certification approach

Whisper adds real local speech recognition without an account or API key. A user downloads a
verified multilingual model once, then uses microphone or system audio offline. CPU speed,
model choice and language affect latency and accuracy. Optional cloud translation/subtitles
still require the user's provider account, key and network connection, with possible usage
charges. These dependencies are disclosed at the beginning of every localized description.

New installations open on **Whisper**, with Base and automatic language detection selected.
The user chooses Download model and then Start; no download or capture starts automatically.
Existing saved setups are preserved on upgrade. The English/French scripted demo remains
available as a setup-free way to check the overlay and export flow. It must always be
labelled as a demo, never as speech recognition.

Historical context: 1.0.3 failed certification because Start Subtitles did not work on the
review device. Windows AI Speech/ML and the Windows speech recognizer were removed in 1.0.5,
which introduced the deterministic demo. Whisper in 1.6.0 is a different CPU implementation
with downloadable models; those earlier failures do not describe the current recognizer.

Review the current [Microsoft Store policies](https://learn.microsoft.com/en-us/windows/apps/publish/store-policies)
when submitting. Metadata must accurately describe dependencies and costs, advertised
features must work on supported devices, and reviewers need a usable test path. The notes
provide Whisper setup and the demo fallback. Resolve access needed to test cloud features
before submission; passing a local check does not imply certification.

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

- [ ] Manifests and lockfiles agree on 1.6.0; both MSIX manifests and the combined bundle
  report 1.6.0.0 and the assigned identity.
- [ ] Both signed native packages are installed and exercised on their target hardware.
- [ ] Clean-install Whisper model download, verification, cancellation/removal and offline
  restart work without API keys. Test English and French; record non-Latin language checks.
- [ ] First launch selects Whisper / Base / automatic detection without starting a download
  or audio capture. An upgrade preserves an existing cloud or scripted-demo selection.
- [ ] Real microphone, system and application capture work; a failed source preserves captions.
- [ ] Whisper Pause admits no new audio; existing backlog can finish. Stop drains; confirmed
  discard shows the incomplete-transcript notice. Export after completion contains the tail.
- [ ] Cloud transcription and translation work with suitable credentials, including a
  non-EN/FR target, two targets, original speech where supplied, and Pause/Resume.
- [ ] Scripted demo starts with no hardware, network or credentials, in English and French.
- [ ] Overlay placement, all reading modes, RTL text, filler list, profiles and shortcuts pass
  the [layout](caption-layout.md#release-verification) and [accessibility](accessibility.md) checks.
- [ ] Text/Markdown/SRT/VTT export, history, recovery, quit prompts and tray behavior work under package identity.
- [ ] Windows App Certification Kit passes on the final packages.
- [ ] EN/FR/DE copy, final-package screenshots and the public privacy policy match this version.

Record evidence and exceptions in the [release handoff](store-updates.md#release-160-handoff).
Previous release checks are historical evidence, not sign-off for new binaries.

## Package identity and signing

The Store signs accepted packages. Local testing needs signed inner MSIX packages and a
trusted matching certificate; signing only the outer bundle is insufficient. Follow
[MSIX packaging](packaging-msix.md) and use a test account/machine if a Store-signed install
would conflict. Do not remove an existing Store installation or its data as a build step.
