# Next Store submission — 1.9.0

**Tagged `v1.9.0` and published on GitHub on 22 October 2026; not yet submitted.** The last
confirmed Store version is
**1.8.0.0**, confirmed by the maintainer on 21 October 2026. The next submission is
**1.9.0.0**: one unsigned bundle with native x64 and ARM64 packages, at
`https://github.com/fmadore/Live-translation/releases/download/v1.9.0/Live.Translation_1.9.0.msixbundle`.

The positioning is unchanged. Lead the listing with offline Whisper captions, the transparent
overlay and optional cloud translation; the scripted demo is a secondary way to try the display
controls. New installations open on Whisper / Base / automatic detection, and existing saved
setups are preserved. Model download and capture remain explicit user actions.

1.9.0 adds a light theme for the operator window: Settings › App › Theme offers System, Light
and Dark. System is the default and follows Windows' light or dark app mode, so an upgrade on a
light-mode PC opens light; What's new says so and how to keep the dark look. The caption
overlay is unchanged in every theme. It also carries fixes from the
[22 October review](app-review-2026-10-22.md). What's new, one added feature line and the
first paragraph of the certification notes say so. The descriptions, short descriptions,
keywords and requirements are unchanged.

- [EN/FR/DE descriptions, features, short descriptions and What's new](store-listing.md)
- [Release notes](release-1.9.0.md)
- [Build evidence and acceptance checklist](store-updates.md#release-190-handoff)
- [Screenshot plan](store-screenshots/README.md)
- [Partner Center walkthrough](partner-center-walkthrough.md)

Before submission, complete:
- the maintainer's test of the Local Test build of the merge commit, above all the theme in
  Light, Dark and System, and a Mistral rehearsal;
- the final-package checks, and the checks still open from 1.8.0;
- a retake of the screenshots from the 1.9.0 MSIX in each language, with Settings › App ›
  Theme set to Dark;
- the German copy review;
- Windows App Certification Kit.

The privacy policy is unchanged: it takes effect on 21 October 2026 for 1.8.0, and the theme
is a local preference. Use the normal Store identity: the side-by-side Local Test installation
is for feedback, not the Store submission package.
