# Next Store submission — 1.8.0

**Tagged `v1.8.0` and published on GitHub on 21 October 2026; not yet submitted.** The last
confirmed Store version is
**1.7.1.0**, confirmed by the maintainer on 21 October 2026. The next submission is
**1.8.0.0**: one unsigned bundle with native x64 and ARM64 packages, at
`https://github.com/fmadore/Live-translation/releases/download/v1.8.0/Live.Translation_1.8.0.msixbundle`.

The positioning is unchanged. Lead the listing with offline Whisper captions, the transparent
overlay and optional cloud translation; the scripted demo is a secondary way to try the display
controls. New installations open on Whisper / Base / automatic detection, and existing saved
setups are preserved. Model download and capture remain explicit user actions.

1.8.0 adds a Transcript tab for reading and copying the transcript during a session. It also
moves Mistral subtitles to Voxtral Realtime 3, which Mistral lists as a public preview, with
lines that end at the speaker's turn and a names-and-terms list. The descriptions, features,
What's new and certification notes say so. The short descriptions, keywords and requirements
are unchanged.

- [EN/FR/DE descriptions, features, short descriptions and What's new](store-listing.md)
- [Release notes](release-1.8.0.md)
- [Build evidence and acceptance checklist](store-updates.md#release-180-handoff)
- [Screenshot plan](store-screenshots/README.md)
- [Partner Center walkthrough](partner-center-walkthrough.md)

Before submission, complete:
- the release-candidate checks, above all a Mistral session on real speech with a
  names-and-terms list;
- the final-package checks;
- a retake of the running-session screenshot (`3-bilingual.png`) in each language;
- the German copy review;
- Windows App Certification Kit.

The privacy policy takes effect on 21 October 2026 for 1.8.0. It adds the names and terms,
which are stored locally and sent to Mistral, and Copy, which places transcript text on the
clipboard. Use the normal Store identity: the side-by-side Local Test installation is for
feedback, not the Store submission package.
