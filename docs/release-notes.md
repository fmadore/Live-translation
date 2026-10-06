# Release notes

Paste-ready text for the **GitHub release** body, which is a different audience from the Store
listing and wants a different register.

Store copy is written for someone deciding whether to install; a release body is read by
someone who already has the app, or who is looking at the source. It can name versions, link
issues and say what changed under the surface. It should still not be a commit log — the
commit log is one click away and nobody wants it twice.

Each body is written before tagging, in its own `release-X.Y.Z.md` file. The
[`Release installers`](../.github/workflows/release.yml) workflow creates the release with an
empty body; set it once the workflow has finished, so no build job recreates the release first:
`gh release edit vX.Y.Z --notes-file docs/release-X.Y.Z.md`. Store status for each version is in
the [Store release history](store-updates.md#release-history).

---

## v1.6.1 — prepared, not yet published

[Release body](release-1.6.1.md) · [Store handoff](store-updates.md#release-161-handoff).
A maintenance and visual release: the new wave-to-words app icon and the October design
review (Windows 11 geometry, one status pill, selector-bar tabs, one stepper control and a
readable move-mode preview). No new features, runtime dependency or privacy changes; the
dev-only `source-map-js` is patched for a security advisory. No tag, GitHub release or Store
submission exists yet.

## Published releases

Bodies before 1.5.0 were kept in this file and are published on their release pages; the
drafts are in its [commit history](https://github.com/fmadore/Live-translation/commits/main/docs/release-notes.md).
Releases without prepared text keep an empty body.

| Release | Date | Body | Summary |
| --- | --- | --- | --- |
| [v1.6.0](https://github.com/fmadore/Live-translation/releases/tag/v1.6.0) | 4 October 2026 | [release-1.6.0.md](release-1.6.0.md) | Local Whisper, two caption languages, bilingual output, Pause and editable filler words. |
| [v1.5.1](https://github.com/fmadore/Live-translation/releases/tag/v1.5.1) | 23 September 2026 | [release-1.5.1.md](release-1.5.1.md) | Audio, safety and history fixes from the September app review. |
| [v1.5.0](https://github.com/fmadore/Live-translation/releases/tag/v1.5.0) | 22 September 2026 | [release-1.5.0.md](release-1.5.0.md) | Searchable caption languages and persistent favourites. |
| [v1.4.2](https://github.com/fmadore/Live-translation/releases/tag/v1.4.2) | 21 September 2026 | Release page | Start and Stop in a persistent bar below the header. |
| [v1.4.1](https://github.com/fmadore/Live-translation/releases/tag/v1.4.1) | 20 September 2026 | Release page | A clearer operator interface: tabbed Settings, profiles above setup, a session browser. |
| [v1.4.0](https://github.com/fmadore/Live-translation/releases/tag/v1.4.0) | 19 September 2026 | Release page | Meeting profiles, reading controls, history search and operator shortcuts. |
| [v1.3.0](https://github.com/fmadore/Live-translation/releases/tag/v1.3.0) | 19 September 2026 | Release page | Transcript history, Stable reading, filler-word cleanup and the German interface. |
| [v1.2.4](https://github.com/fmadore/Live-translation/releases/tag/v1.2.4) | 14 September 2026 | Release page | Captions that adapt to the overlay window (#77). |
| [v1.2.3](https://github.com/fmadore/Live-translation/releases/tag/v1.2.3) | 8 September 2026 | Release page | Native Save As with SRT/VTT export (#26) and single-application capture (#27). |
| [v1.2.2](https://github.com/fmadore/Live-translation/releases/tag/v1.2.2) | 8 September 2026 | Release page | Bilingual interface, caption appearance controls, and audio capture and recovery fixes. |

1.2.0 and 1.2.1 were prepared here but never tagged; v1.2.2 is the first release that carries
their changes.
