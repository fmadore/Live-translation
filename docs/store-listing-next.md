# Next Store submission — 1.11.0

**Prepared; not yet tagged or submitted.** 1.10.0 (MSIX 1.10.0.0) was submitted and is in
certification, as reported by the maintainer on 26 October 2026; the last confirmed live Store
version is **1.9.0.0** (`Get-AppxPackage`, 25 October 2026). The next submission is
**1.11.0.0**: one unsigned bundle with native x64 and ARM64 packages, at
`https://github.com/fmadore/Live-translation/releases/download/v1.11.0/Live.Translation_1.11.0.msixbundle`.
Partner Center takes one submission at a time, so submit 1.11.0 once 1.10.0 is live.

The positioning is unchanged. Lead the listing with offline Whisper captions, the transparent
overlay and optional cloud translation; the scripted demo is a secondary way to try the display
controls. New installations open on Whisper / Base / automatic detection, and existing saved
setups are preserved. Model download and capture remain explicit user actions.

1.11.0 adds an optional Buy Me a Coffee link
([#182](https://github.com/fmadore/Live-translation/pull/182), part of
[#180](https://github.com/fmadore/Live-translation/issues/180)) and fixes an empty band at the
bottom of the operator window. It is the first version with a way to pay anyone, so it needs
three things the earlier submissions did not:

- **Properties › Product declarations:** tick *This product allows users to make purchases, but
  does not use the Microsoft Store commerce system*. Store Policy 10.8.2 requires the use of a
  third-party payment provider to be noted in Partner Center, and Microsoft's
  [product declarations guide](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/product-declarations)
  says to tick this box for any app that does not use Microsoft Commerce.
- **Notes for certification:** the new second paragraph in [store-listing.md](store-listing.md)
  explains the link, names Buy Me a Coffee as the payment provider and states that a donation
  unlocks nothing.
- **Descriptions:** one added sentence per language saying the link is optional and unlocks
  nothing (Policy 10.8.4).

The Microsoft Store supporter add-on in #180 is not part of 1.11.0.

- [EN/FR/DE descriptions, What's new and certification notes](store-listing.md)
- [Release notes](release-1.11.0.md)
- [Build evidence and acceptance checklist](store-updates.md#release-1110-handoff)
- [Screenshot plan](store-screenshots/README.md)
- [Partner Center walkthrough](partner-center-walkthrough.md)

Before submission, complete:
- the maintainer's test of the Local Test build of the merge commit, above all the support link
  opening the default browser from Settings and from the start screen, and the operator window
  no longer scrolling as a whole;
- the final-package checks, and the checks still open from earlier releases;
- the screenshots: the start screen gains a one-line credit at its foot, which needs no retake;
  retake them only if the 1.9.0 retake is still outstanding;
- the German copy review;
- Windows App Certification Kit.

The privacy policy gains a *Support link* section and takes effect on 27 October 2026 for
1.11.0; check that the published page shows it before submitting. Use the normal Store identity:
the side-by-side Local Test installation is for feedback, not the Store submission package.
