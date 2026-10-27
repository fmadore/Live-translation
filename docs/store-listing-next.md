# Next Store submission — 1.12.0

**Tagged 27 October 2026; not yet submitted.** The Store serves **1.11.0.0**, accepted as
reported by the maintainer on 26 October 2026. The next submission is **1.12.0.0**: one
unsigned bundle with native x64 and ARM64 packages, at
`https://github.com/fmadore/Live-translation/releases/download/v1.12.0/Live.Translation_1.12.0.msixbundle`.

The positioning is unchanged. Lead the listing with offline Whisper captions, the transparent
overlay and optional cloud translation; the scripted demo is a secondary way to try the display
controls. New installations open on Whisper / Base / automatic detection, and existing saved
setups are preserved. Model download and capture remain explicit user actions.

1.12.0 adds **Captions in Zoom**
([#189](https://github.com/fmadore/Live-translation/pull/189), part of
[#187](https://github.com/fmadore/Live-translation/issues/187)): the operator pastes the caption
link (API token) the meeting's host copied in Zoom, and the app posts one caption language
directly to Zoom's caption bar. It also fixes flashes and layout shifts in setup and the
transcript ([#188](https://github.com/fmadore/Live-translation/pull/188)) and says why Start
waits for an application. It is the first version that can send **user content to a third
party other than the speech provider the user chose**, including captions made offline with
Whisper, and the first whose listing names another company's product. So it needs things the
earlier submissions did not:

- **Privacy policy (Store Policy 10.5, personal information).** Caption text can contain
  personal information, such as the names of people speaking, and sharing sends it to Zoom. The
  published [privacy policy](privacy.md) must describe that flow: its *Optional captions in Zoom*
  section and the Zoom line under *Third-party processing* do, with the effective date
  27 October 2026 for 1.12.0. Sharing is opt-in by construction: nothing is sent until the user
  pastes a link and chooses **Start sharing**, and the dialog says, before that button, that
  caption text goes to Zoom even when Whisper makes it. Check that the published page shows the
  new date and the Zoom section before submitting.
- **Notes for certification.** The new second paragraph in [store-listing.md](store-listing.md)
  says what the feature sends and where, that it is off until the user starts it, and how a
  tester can exercise it (a Zoom meeting they host, with manual captioning and the third-party
  caption API token allowed) or skip it (opening the dialog and pasting a non-Zoom address,
  which the app refuses without sending anything). A free Basic Zoom account is enough (confirmed
  by the maintainer on 27 October 2026), but the certification team cannot be expected to have
  one; the credential-free Whisper and demo routes still cover the rest of the app.
- **Trademark (Policies 10.1 and 11.2).** Zoom is named only descriptively, to say what the
  feature works with; the app shows no Zoom logo (its toolbar mark is a generic captions
  symbol, deliberately, because Zoom's logo needs Zoom's written licence), and neither the
  screenshots nor the logo use Zoom's artwork. Each description ends with "Zoom is a trademark of
  Zoom Communications, Inc. Live Translation & Subtitles is not affiliated with or endorsed by
  Zoom." in its language, so the line sits wherever the listing names Zoom. Zoom is not added
  to the keywords.
- **Descriptions, features and What's new.** One sentence per description on Zoom captions,
  one on the caption text it sends in the privacy paragraph, a twentieth feature line (the most
  Partner Center allows), and a new What's new in all three languages.

No product declaration changes: the purchases-outside-Microsoft-commerce box ticked for 1.11.0
stays ticked, and Zoom captions involve no payment. The MSIX capabilities are unchanged
(`runFullTrust` and `microphone`).

- [EN/FR/DE descriptions, What's new and certification notes](store-listing.md)
- [Release notes](release-1.12.0.md)
- [Build evidence and acceptance checklist](store-updates.md#release-1120-handoff)
- [Captions in Zoom guide](zoom-captions.md), which the app's dialog opens
- [Screenshot plan](store-screenshots/README.md)
- [Partner Center walkthrough](partner-center-walkthrough.md)

Before submission, complete:
- the maintainer's test of the Local Test build of the release merge: Captions in Zoom already
  passed in a real meeting on a free account (27 October 2026), so this is a final pass over
  the merge, including the *Show captions on this screen too* switch, the pre-flight asking for
  an application, and the setup and transcript no longer jumping;
- the final-package checks, and the checks still open from earlier releases;
- the screenshots: the toolbar gains a *Zoom captions* button, which the current images (from
  1.7.1) do not show; decide whether a retake is wanted, and if one is taken, keep any Zoom
  window and Zoom's artwork out of it;
- the German copy review, now including the Zoom sentences;
- Windows App Certification Kit.

Use the normal Store identity: the side-by-side Local Test installation is for feedback, not the
Store submission package.
