# Store update preparation — 1.4.2

Target **1.4.2** (MSIX **1.4.2.0**). The [EN/FR/DE listing](store-listing.md) is the single
source for paste-ready fields. The [release handoff](store-updates.md#release-142-handoff)
tracks checks, packages, screenshots, privacy publication and manual submission.

The latest published GitHub release is [v1.4.2](https://github.com/fmadore/Live-translation/releases/tag/v1.4.2),
which has not been submitted to the Store. The next submission should use 1.4.2 after native
acceptance and release packaging. Expected artifact: **Live.Translation_1.4.2.msixbundle**,
containing x64 and ARM64. The release workflow has completed and the downloaded bundle's
two 1.4.2.0 manifests, Store identity and SHA-256 are verified in the handoff.
A local x64 installer is not the Store upload artifact.

The copy also covers tabbed Settings, profile management in the setup rail, history browsing,
visible presets, colour swatches and the localized tray. It retains the 1.4.0 additions:
meeting profiles, reading controls/previews, history titles/search, input status, shortcuts
and Gemini Smart finalization. It also retains history/Stable reading/local
cleanup and German-interface information for customers updating from 1.2.4. GitHub and all
three Store “What's new” blocks thank **@valentinrabot** for feedback, not implementation.

The custom word-list request [#85](https://github.com/fmadore/Live-translation/issues/85)
remains open. Native package acceptance, new screenshots, German review and certification
remain outstanding. See the [submission walkthrough](partner-center-walkthrough.md).
