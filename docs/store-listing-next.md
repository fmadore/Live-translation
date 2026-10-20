# Next Store submission — 1.7.0

**Tagged `v1.7.0` on 20 October 2026; not yet submitted.** The last confirmed Store version is
**1.6.1.0**, confirmed by the maintainer on 20 October 2026. The next submission is
**1.7.0.0**: one unsigned bundle with native x64 and ARM64 packages, at
`https://github.com/fmadore/Live-translation/releases/download/v1.7.0/Live.Translation_1.7.0.msixbundle`
once the `v1.7.0` release workflow finishes.

The positioning is unchanged: lead the listing with offline Whisper captions, the transparent
overlay and optional cloud translation; the scripted demo is a secondary way to try the display
controls. New installations open on Whisper / Base / automatic detection, existing saved setups
are preserved, and model download and capture remain explicit user actions.

This submission replaces 1.6.1, which already brought the wave-to-words icon and the refreshed
interface to the Store. 1.7.0 adds offline translation into English with Whisper, an overlay
that reopens where it was placed and moves onto a connected screen when its display is
unplugged, cloud sessions that reconnect instead of ending, and faster Whisper, which on x64
now needs a processor with AVX2. The descriptions, features, requirements, certification notes
and What's new say so. Every listing language still needs new screenshots.

- [EN/FR/DE descriptions, features, short descriptions and What's new](store-listing.md)
- [Release notes](release-1.7.0.md)
- [Build evidence and acceptance checklist](store-updates.md#release-170-handoff)
- [Screenshot plan](store-screenshots/README.md)
- [Partner Center walkthrough](partner-center-walkthrough.md)

Before submission, complete the release-candidate checks, final-package checks, the icon check
on both architectures, screenshots, German copy review and Windows App Certification Kit. The
privacy policy takes effect on 7 October 2026 for 1.7.0: it adds translation into English,
downloads through a proxy, discarding pending audio on quit, the remembered export format and
the overlay placement per display arrangement. Use the normal Store identity; the side-by-side
Whisper Test installation is for feedback and is not the Store submission package.
