# Store update preparation — 1.5.1

Target **1.5.1** (MSIX **1.5.1.0**). The [EN/FR/DE listing](store-listing.md) is the single
source for paste-ready fields. The [release handoff](store-updates.md#release-151-handoff)
tracks checks, packages, screenshots, privacy publication and manual submission.

The latest published GitHub release is [v1.5.1](https://github.com/fmadore/Live-translation/releases/tag/v1.5.1).
It supersedes 1.4.2 and 1.5.0, neither of which was submitted to the Store. The next
submission should use 1.5.1 after native acceptance and release packaging. Expected artifact:
**Live.Translation_1.5.1.msixbundle**, containing x64 and ARM64. Its manifests, Store identity
and SHA-256 are recorded in the handoff once the release workflow has built it.
A local x64 installer is not the Store upload artifact.

The 1.5.1 "What's new" blocks open with this release's fixes: speech detail in the audio sent
to engines, a transcript that survives one failing source, lighter history saving, and keyboard
access to a closing prompt over Settings. They keep the 1.5.0 caption-language paragraphs:
searchable targets, favourites, F2 and RTL captions. For customers updating from 1.2.4 they
also retain the 1.3.0–1.4.2 additions: profiles, history with search and export, reading
presets, steadier updates and persistent Start/Stop. GitHub and all three Store "What's new"
blocks thank **@valentinrabot** for feedback, not implementation.

The custom word-list request [#85](https://github.com/fmadore/Live-translation/issues/85)
and live language acceptance in [#78](https://github.com/fmadore/Live-translation/issues/78)
remain open. Native package acceptance, new screenshots, German review and certification
remain outstanding. See the [submission walkthrough](partner-center-walkthrough.md).
