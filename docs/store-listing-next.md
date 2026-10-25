# Next Store submission — 1.10.0

**Prepared; not yet tagged or submitted.** The last confirmed Store version is **1.9.0.0**, live
as confirmed with `Get-AppxPackage` (`SignatureKind` `Store`) on 25 October 2026. The next
submission is **1.10.0.0**: one unsigned bundle with native x64 and ARM64 packages, at
`https://github.com/fmadore/Live-translation/releases/download/v1.10.0/Live.Translation_1.10.0.msixbundle`.

The positioning is unchanged. Lead the listing with offline Whisper captions, the transparent
overlay and optional cloud translation; the scripted demo is a secondary way to try the display
controls. New installations open on Whisper / Base / automatic detection, and existing saved
setups are preserved. Model download and capture remain explicit user actions.

1.10.0 is a reliability release: batches 4–6 of the [22 October review](app-review-2026-10-22.md)
([#171](https://github.com/fmadore/Live-translation/pull/171)) and the anti-cheat fix to
application listing ([#177](https://github.com/fmadore/Live-translation/pull/177)). What's new
and the first paragraph of the certification notes say so. The descriptions, features and
keywords carry #177's sentence, feature line and keyword for players following voice chat in
another language, if they were not already pasted into the live listing; the short descriptions
and requirements are unchanged.

- [EN/FR/DE descriptions, features, short descriptions and What's new](store-listing.md)
- [Release notes](release-1.10.0.md)
- [Build evidence and acceptance checklist](store-updates.md#release-1100-handoff)
- [Screenshot plan](store-screenshots/README.md)
- [Partner Center walkthrough](partner-center-walkthrough.md)

Before submission, complete:
- the maintainer's test of the Local Test build of the merge commit, above all system audio
  with local Whisper, overlay move mode, Alt+F4 then the toggle and the tray, and a long
  Gemini session;
- the final-package checks, and the checks still open from 1.8.0 and 1.9.0;
- the screenshots: nothing they show changed in 1.10.0, so retake them only if the 1.9.0
  retake is still outstanding;
- the German copy review;
- Windows App Certification Kit.

The privacy policy is unchanged: it takes effect on 21 October 2026 for 1.8.0. Use the normal
Store identity: the side-by-side Local Test installation is for feedback, not the Store
submission package.
