# Microsoft Store updates

Every update to this app is submitted **by hand** through Partner Center. That is not a
preference and not a stopgap — it is the only route this account has, for a reason Microsoft
enforces and we cannot work around. [Why by hand](#why-by-hand) has the detail; the short
version is that the submission API is a Company-account feature and this is an Individual
account.

The route below is what shipped every version so far. It takes about five minutes once the
packages are built.

## Release 1.6.1 handoff

**Prepared on 6 October 2026; not tagged, published or submitted.** Last confirmed live
Store version: **1.6.0.0**, confirmed by the maintainer on 6 October 2026. Target app
version: **1.6.1**, MSIX **1.6.1.0**.

A maintenance and visual release: no new features, no runtime dependency changes and no
privacy change. The **wave-to-words** mark, a speech wave settling into a caption line on a dark
tile, replaces the stock Lucide "languages" glyph in Start, the taskbar, the tray, the Store
tiles and the toolbar badge ([#111](https://github.com/fmadore/Live-translation/pull/111)).
[#112](https://github.com/fmadore/Live-translation/pull/112) applies the 5 October 2026
design review:

- Windows 11 geometry: 4px corners in the page, 8px for dialogs and flyouts.
- One consistent status pill; Settings tabs become a selector bar; default buttons are
  distinguishable from ghost buttons; the settings gear and dialog close share one icon button.
- One stepper control everywhere; − and + now disable at their limits.
- Move mode: the toolbar is a flyout that no longer wraps, and the placement preview is drawn
  in the real caption colour, outline and backing, so it stays readable over a white slide.
- The icon and Store tiles are redrawn in the app's palette colours; the Whisper model block
  takes the rail's spacing; the Remote speaker chip is neutral, so mint now means live,
  primary or on.
- The pre-flight intro points to Start at the top of the window, in English, French and German.
- The GitHub social preview is redrawn in Archivo and IBM Plex Mono with the token colours.

Developer-facing: new tokens (`-wash`/`-chip`/`-border` steps for every hue, `--radius-overlay`,
`--control-sm/md/lg`, `--tracking-caps`, `--leading-*`), shared `.ui-card` and `.ui-tool.icon`
components, and new `typeScale`/`spacing` guard tests.
[GitHub release copy](release-1.6.1.md) · [EN/FR/DE Store fields](store-listing.md) ·
[Partner Center walkthrough](partner-center-walkthrough.md).

### Preparation and validation

- [x] PR #111 merged as `324332f6d01f9020f7cbd3997d97376df9f98cd4`.
- [x] App manifests and root lockfile entries set to 1.6.1; citation release date set to
  6 October 2026. Reset it to the publication date if tagging happens later, as for 1.6.0.
- [x] No runtime dependency, package-manifest template or capability change since `v1.6.0`.
  Besides the version lines, the only lockfile change is the dev-only `source-map-js` 1.2.1 →
  1.2.2 (GHSA-68fv-2mgg-jv7q), which the CI audit step required; it is not shipped in the app.
- [x] Frontend validation on the #112 branch: 550 tests in 59 files, Svelte check (zero
  errors and warnings), Prettier and the production build.
- [ ] PR #112 CI, including the Rust jobs, is pending.
- [ ] Rust tests, formatting and Clippy were not run locally: this machine has no
  LLVM/libclang, which the Whisper bindings need since #99. CI installs it and is the gate.

### Remaining Store acceptance

- [ ] Merge #112, then this release PR.
- [ ] Tag `v1.6.1` from `main` after the merge. Pass release-commit CI and all
  installer/MSIX/bundle jobs, verify the downloaded assets and publish the prepared GitHub
  release body.
- [ ] Verify `Live.Translation_1.6.1.msixbundle` contains native x64 and ARM64 packages at
  **1.6.1.0** with the [assigned identity](microsoft-store.md#store-identity-assigned), and
  that both carry the new `Square44x44Logo`, `Square150x150Logo`, `Wide310x150Logo` and
  `StoreLogo` assets. Record sizes and SHA-256 below.
- [ ] On both architectures, check the new icon in Start, the taskbar, the tray and the Store
  tile.
- [ ] Smoke-test capture, Whisper and overlay move mode over a white and a dark slide; Settings
  tabs; steppers at their limits; keyboard focus; Windows contrast themes and enlarged text.
  The shared button and tab styles changed, so earlier checks do not cover them.
- [ ] Recapture EN/FR/DE Store screenshots, because the interface and icon changed. The
  [capture plan](store-screenshots/README.md) is still pending from 1.6.0.
- [ ] Run Windows App Certification Kit, which also reports missing package images.
- [ ] Upload the new `.github/social-preview.png` in the GitHub repository settings. This is
  manual and not part of the Store.
- [ ] Submit the single combined bundle in Partner Center with the 1.6.1 What's new text.
- [ ] Record submission/certification and confirm the Store version before marking it live.

### 1.6.1 artifact verification

Pending: no tag, release assets or bundle exist yet. After tagging, record the tag commit,
CI runs, bundle and package sizes, SHA-256 digests and manifest values here, as for 1.6.0.

## Historical release handoffs

The sections below preserve the status and evidence recorded for earlier releases. Old
upload instructions, pending checks and listing links do not select the current submission;
use the 1.6.1 handoff above. The shared listing file now contains 1.6.1 copy; Git history
preserves earlier text.

## Release 1.6.0 handoff

**1.6.0 is live in the Microsoft Store**, MSIX **1.6.0.0**, as the maintainer confirmed on
6 October 2026; the exact go-live date was not recorded. GitHub release:
[v1.6.0](https://github.com/fmadore/Live-translation/releases/tag/v1.6.0), published on
4 October 2026. It replaces 1.5.1 as the Store version.

Includes local Whisper (#99), two translation targets, original speech in overlay/exports,
Pause/Resume, editable filler words and the remaining September review improvements.
[GitHub release copy](release-1.6.0.md) · [EN/FR/DE Store fields](store-listing.md) ·
[Partner Center walkthrough](partner-center-walkthrough.md).

The README is shortened around the current features, with detailed guides linked from the
[documentation index](README.md). Store copy leads with offline captions and the overlay;
the scripted demo remains a secondary setup aid. Fresh installations select Whisper / Base /
automatic detection; downloads and capture require user action. Existing saved setups are
preserved. @valentinrabot is credited for proposing local transcription in issue #98.
Privacy/security documentation now distinguishes cloud streaming, Whisper's temporary local
audio buffering and optional saved text history. No dependency versions change in preparation.

### Preparation and validation

- [x] PR #99 merged as `3717e5968d674af2e31818ae9ebd488caebccc75`; all seven PR CI jobs passed.
- [x] App manifests and root lockfile entries set to 1.6.0; citation release date set to
  4 October 2026 at publication.
- [x] README, active user/developer guides, EN/FR/DE Store descriptions, features, short
  descriptions, What's new, requirements and certification steps updated.
- [x] The prior PR validation passed 538 frontend tests, 120 Rust tests (3 ignored), Clippy,
  type checks and build; ARM64 English/French Whisper fixture inference also passed.
- [x] Release-preparation validation: 543 frontend tests, 120 Rust tests (3 opt-in tests ignored),
  Svelte check (zero errors/warnings), catalog parity, production build, Prettier, Rust
  formatting and all-target Clippy with warnings denied. `npm audit` reports zero vulnerabilities.
- [x] Store field lengths, localized screenshot captions, relative file links, UTF-8,
  version consistency and unchanged dependency resolutions checked.
- [x] First-launch regressions cover absent/corrupt saved setup, preserving explicit demo
  selection and requiring a click for the initial model download. A fresh browser origin
  shows Whisper / Base / automatic detection and the revised subtitle copy, with no console
  warnings or errors. Native capture remains outside browser verification.
- [x] Rebuild and inspect native ARM64 and x64 MSIX packages and the combined unsigned bundle
  after changing the first-launch default.

### Store acceptance

- [ ] Test the final signed x64 and ARM64 MSIX on their native hardware: model download,
  cancellation/removal, offline relaunch, microphone/system/application/Both capture,
  Whisper backlog, Pause/Resume, Stop/discard and all export formats.
- [ ] Complete cloud language acceptance (#78), two-target translation and original-speech
  checks with suitable provider credentials; record provider and date.
- [ ] Complete keyboard/Narrator, enlarged text, mixed-DPI overlay, profiles, history,
  recovery, tray and quit-prompt checks from [Store acceptance](microsoft-store.md#acceptance-checklist).
- [ ] Capture final MSIX screenshots in EN/FR/DE and review the German listing. Existing
  repository screenshots are historical; see the [capture plan](store-screenshots/README.md).
- [x] Confirm the public privacy policy responds successfully and includes the
  4 October Whisper update, model downloads and temporary local audio buffering.
- [ ] Run Windows App Certification Kit.
- [x] Tag `v1.6.0`, pass release-commit CI and all installer/MSIX/bundle jobs, verify
  downloaded assets and publish the prepared GitHub release body.
- [x] Submit the published combined bundle in Partner Center and pass certification. **Live in
  the Store** as 1.6.0.0 (confirmed by the maintainer on 6 October 2026; the submission and
  certification dates were not recorded).

The unchecked items above were not recorded as done before submission. They stay open as
checks against the Store build.

The already installed **Live Translation Whisper Test** is a separate ARM64 NSIS build of
PR #99 (version label 1.5.1), with its own identity/preferences. It remains useful for feedback
but is not the Store submission artifact or evidence of 1.6.0 MSIX acceptance.

### Published 1.6.0 artifact verification

Tag `v1.6.0` points to `41cee07bddc6ad412787972798c277bdb46cacdb`.
All seven [release-commit CI jobs](https://github.com/fmadore/Live-translation/actions/runs/37210371969)
and all four [installer/MSIX/bundle jobs](https://github.com/fmadore/Live-translation/actions/runs/37210374213)
passed. The GitHub release body is [release-1.6.0.md](release-1.6.0.md).

Upload **[Live.Translation_1.6.0.msixbundle](https://github.com/fmadore/Live-translation/releases/download/v1.6.0/Live.Translation_1.6.0.msixbundle)**
to Partner Center. These are the published CI assets; each downloaded file's SHA-256 matches
GitHub's asset digest. Local copies and verification reports are in the ignored directory
`src-tauri/target/release-1.6.0/published/`.

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| [Live.Translation_1.6.0.msixbundle](https://github.com/fmadore/Live-translation/releases/download/v1.6.0/Live.Translation_1.6.0.msixbundle) | 11,129,466 | `8b24e343ee505f4b5efa7dd0278fa82e5c926592299f27a85ba00fac876ff430` |
| [Live.Translation_1.6.0_x64.msix](https://github.com/fmadore/Live-translation/releases/download/v1.6.0/Live.Translation_1.6.0_x64.msix) | 5,658,699 | `1efc608f9502f29cc6015424fe1f3ea19020b59ccfdd3d2efbd2194ba8897bbf` |
| [Live.Translation_1.6.0_arm64.msix](https://github.com/fmadore/Live-translation/releases/download/v1.6.0/Live.Translation_1.6.0_arm64.msix) | 5,468,759 | `1926305273bc30e4ee1cdb3fa02d714082eb1333469ffd24155052ff54712b07` |

The bundle contains exactly the separately downloaded x64 and ARM64 packages, byte-for-byte.
Bundle and inner manifests report **1.6.0.0**, identity `49346FMadore.LiveTranslationSubtitles`,
publisher `CN=5D0ECC96-3998-452E-B7E9-29BE9B576F86` and publisher display name `FMadore`.
The executable PE machine values confirm native `0x8664` (x64) and `0xAA64` (ARM64).
Both packages include the English/French rehearsal WAVs and all three Whisper license notices,
declare only `runFullTrust` and `microphone`, and contain no model weights or signature.
The Store signs accepted packages; final signed-package acceptance remains tracked above.
1.6.0 was then submitted through Partner Center and is live; see above.

### Local candidate artifact verification (before tagging)

Rebuilt locally on 4 October 2026 after the Whisper first-launch change, based on the
release-preparation branch. The revised frontend and localized subtitle copy are embedded.
ARM64 was built natively; x64 was cross-compiled on the ARM64 host. Both builds used Rust
1.98.1, LLVM/Clang 23.1.2, the normal committed Store configuration and the production frontend.
No local-test identity override was applied. Files are in the ignored local directory
`src-tauri/target/release-1.6.0/`; retain them separately from any signed sideload copies.

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| `Live.Translation_1.6.0.msixbundle` | 11,119,129 | `aabf00457ce9519e198d0c16ca056a540a33da06791ac13948f3beead3e8b5e6` |
| `Live.Translation_1.6.0_arm64.msix` | 5,464,924 | `79368b857a297f5d18fb8dfbf5c4b8a3d97ac2f01e9f279dc39160bd63467c56` |
| `Live.Translation_1.6.0_x64.msix` | 5,652,196 | `2645d4141aa45dab29f797949af20ce53f6632ef68413068e4b27fd02a248276` |

The bundle contains exactly the verified x64 and ARM64 packages, byte-for-byte. Bundle and
inner manifests report **1.6.0.0**, identity `49346FMadore.LiveTranslationSubtitles`, publisher
`CN=5D0ECC96-3998-452E-B7E9-29BE9B576F86`, and inner publisher display name `FMadore`.
Executable PE machine values are `0xAA64` (ARM64, 10,606,080 bytes) and `0x8664` (x64,
12,043,776 bytes). Each package includes both rehearsal WAVs and all three Whisper license
notices, declares only `runFullTrust` and `microphone`, and carries no model weights or
signature. Local `verification.json` and `SHA256SUMS.txt` accompany the artifacts.

This verifies packaging and architecture, not native x64 runtime behavior or Store
certification. Final signed-package acceptance and fresh screenshots remain pending above.
These local candidate files are superseded for submission by the published CI assets
above. Independently rebuilt packages need not be byte-identical.

## Release 1.5.1 handoff

**1.5.1 is live in the Microsoft Store**, MSIX **1.5.1.0**, as the maintainer confirmed on
23 September 2026. GitHub release: [v1.5.1](https://github.com/fmadore/Live-translation/releases/tag/v1.5.1).
It replaces 1.2.4 as the Store version; 1.3.0 through 1.5.0 were never submitted.

A maintenance release from batches 1–2 of the [22 September app review](app-review-2026-09-22.md)
([#86](https://github.com/fmadore/Live-translation/pull/86)). It sends providers a cleaner
downsampled signal and ignores `.env` and host overrides in release builds. A failing capture or
provider thread now ends that source instead of the app. Stacked dialogs keep keyboard access.
History saves at most every 5 seconds, and Stable reading's overlay context is bounded. All
1.5.0 language, favourite and RTL features remain included.
[Release notes](release-1.5.1.md) · [EN/FR/DE listing](store-listing.md).

Package note: keeping panic unwinding (review item D10) grows the executable from 5.2 MB to
9.2 MB; compressed, it grows by about 43%. Only `.woff2` fonts ship now, which saves about
0.4 MB. Net, the Store bundle grows from 7.96 MB to 9.22 MB; see the verification below.

- [x] Apply and merge review batches 1–2; CI green on Rust 1.98 (#86, merge `a276b68`).
- [x] Synchronize manifests, lockfiles and citation metadata to 1.5.1.
- [x] Prepare GitHub release notes and EN/FR/DE Store "What's new" text.
- [x] Local validation: 368 frontend tests, 80 Rust tests (2 ignored), type checks, production
  build, formatting, catalog parity, `npm audit` (3 low, none moderate+) and Clippy on 1.98.1.
- [x] **Before tagging:** a live session from a 48 kHz microphone or system source with at
  least one provider. The maintainer confirmed it on 23 September 2026. The rehearsal fixtures
  are 16 kHz and never pass through the new filter.
- [x] Tag `v1.5.1`; confirm release-commit CI and all installer/MSIX/bundle jobs.
- [x] Download the combined x64/ARM64 bundle; record its size, SHA-256, architectures and
  embedded 1.5.1.0 manifests below.
- [ ] Test installed x64/ARM64 packages:
  - quit prompt over Settings (Escape/Tab reach only the prompt; **Discard and close** is
    reachable by keyboard);
  - tray labels and status during a session;
  - the labelled microphone selector;
  - history during and after a long session, including one ended by a provider error;
  - Stable reading over a long session;
  - a `GEMINI_WS_HOST` user environment variable pointing elsewhere is ignored, and Gemini
    still connects;
  - plus the 1.5.0 carry-overs: keyboard/Narrator, enlarged text, unsupported selections,
    favourites/profile/history persistence, live audio, overlay and export.
- [ ] Complete the OpenAI target-code probe and live non-EN/FR speech on both translation
  providers. #78 remains open; see [coverage](language-coverage.md).
- [ ] Capture fresh packaged-app EN/FR/DE screenshots and complete native German review.
- [ ] Run Windows App Certification Kit and check the public privacy policy.
- [x] Submit 1.5.1 through Partner Center and pass certification. **Live in the Store**
  (confirmed 23 September 2026).

The unchecked items above were not recorded as done before submission. They stay open as
checks against the Store build.

### 1.5.1 artifact verification

Verified on 23 September 2026. Tag `v1.5.1` points to release commit `e6a4e18`.
[Release-commit CI](https://github.com/fmadore/Live-translation/actions/runs/35825869697)
and [all installer/MSIX/bundle jobs](https://github.com/fmadore/Live-translation/actions/runs/35825872669)
passed. The release contains EXE, MSI, separate x64/ARM64 MSIX packages and the combined
bundle. Its GitHub body is [release-1.5.1.md](release-1.5.1.md).

Upload [Live.Translation_1.5.1.msixbundle](https://github.com/fmadore/Live-translation/releases/download/v1.5.1/Live.Translation_1.5.1.msixbundle)
after the remaining acceptance checks. Downloaded size: **9,222,548 bytes**, up from 7,960,437
for 1.5.0 (+16%). That is the unwinding cost from D10: the packaged `live-translation.exe` is
9,649,152 bytes on x64 and 8,565,760 bytes on ARM64. Local SHA-256 matches the GitHub asset
digest: `3aad94e8b1f4a8f28cdc36f894b664b4add592bbd65a48719e8a6c940a8d204a`.

The bundle and both embedded package manifests report **1.5.1.0**. The packages target
**x64** and **arm64** (`Windows.Desktop`, MinVersion 10.0.17763.0), use identity
`49346FMadore.LiveTranslationSubtitles`, publisher `CN=5D0ECC96-3998-452E-B7E9-29BE9B576F86`
and publisher display name `FMadore`. They declare only `runFullTrust` and `microphone`, carry
no signature (the Store signs accepted packages), and each includes nonempty English (665,006
bytes) and French (743,246 bytes) rehearsal WAV fixtures. This verifies packaging, not
installed-app operation or Store certification. 1.5.1 was then submitted through Partner Center
and is live; see above.

## Release 1.5.0 handoff

Historical handoff; superseded by [1.5.1](#release-151-handoff).

Next Store submission target: **1.5.0**, MSIX **1.5.0.0**, dated 22 September 2026.
GitHub release: [v1.5.0](https://github.com/fmadore/Live-translation/releases/tag/v1.5.0).
This supersedes unsubmitted 1.4.2. The last documented Store release remains 1.2.4.

Includes searchable provider-scoped caption languages, persistent favourites, unsupported-target
Start/Rehearse gates, F2 favourite swapping, separate English/French demo/rehearsal types,
expanded transcript/profile metadata and RTL overlay direction. Dropdown labels and stars are
vertically centered. All earlier profile, history, reading and persistent-control improvements
remain included. [Release notes](release-1.5.0.md) · [EN/FR/DE listing](store-listing.md).

- [x] Implement #78 and document the behavior and pending live acceptance.
- [x] Synchronize manifests, lockfiles and citation metadata to 1.5.0.
- [x] Prepare GitHub release notes and EN/FR/DE Store listing text.
- [x] Local validation: 356 frontend tests, 74 Rust tests, type checks, production build,
  formatting, catalog parity and Clippy. Two billable/network probes are opt-in.
- [x] Browser layout checks at 980×660 in EN/FR/DE, centered rows, Japanese/Arabic font fallback
  and Arabic stable-reading direction.
- [x] Confirm release-commit CI and all installer/MSIX/bundle jobs.
- [x] Download the combined x64/ARM64 bundle; record its size, SHA-256, architectures and
  embedded 1.5.0.0 manifests below.
- [ ] Complete OpenAI target-code probe, especially Portuguese/Chinese variants, and live
  non-EN/FR speech on both translation providers. #78 remains open; see [coverage](language-coverage.md).
- [ ] Test installed x64/ARM64 packages, including keyboard/Narrator, enlarged text,
  unsupported selections, favourites/profile/history persistence, live audio, overlay and export.
- [ ] Capture fresh packaged-app EN/FR/DE screenshots and complete native German review.
- [ ] Run Windows App Certification Kit and check the public privacy policy.
- [ ] Upload **Live.Translation_1.5.0.msixbundle** in Partner Center, apply prepared listing fields,
  then submit and record certification/rollout. No Store submission has been made.

### 1.5.0 artifact verification

Verified on 22 September 2026. Tag `v1.5.0` points to release commit `d1ad615`.
[Release-commit CI](https://github.com/fmadore/Live-translation/actions/runs/35703510948)
and [all installer/MSIX/bundle jobs](https://github.com/fmadore/Live-translation/actions/runs/35703560787)
passed. The release contains EXE, MSI, separate x64/ARM64 MSIX packages and the combined bundle.

Upload [Live.Translation_1.5.0.msixbundle](https://github.com/fmadore/Live-translation/releases/download/v1.5.0/Live.Translation_1.5.0.msixbundle)
after the remaining acceptance checks. Downloaded size: **7,960,437 bytes**.
Local SHA-256 matches the GitHub asset digest:
`737b95a417030666b55dc0f8a4850b12b29e528424206fe2dd4a387836e494ce`.

The bundle and both embedded package manifests report **1.5.0.0**. The packages target
**x64** and **arm64**, use identity `49346FMadore.LiveTranslationSubtitles` and publisher
`CN=5D0ECC96-3998-452E-B7E9-29BE9B576F86`, and each includes nonempty English and French
rehearsal WAV fixtures. This verifies packaging, not live-provider acceptance, installed-app
operation or Store certification. No Partner Center submission has been made.

## Release 1.4.2 handoff

Historical handoff; superseded by [1.5.0](#release-150-handoff).

Next Store submission target: **1.4.2**, MSIX **1.4.2.0**, dated 21 September 2026.
GitHub release: [v1.4.2](https://github.com/fmadore/Live-translation/releases/tag/v1.4.2).
This supersedes the unsubmitted 1.4.1 target. The last documented Store release is 1.2.4.

Start and Stop share a persistent action bar below the header. Setup, live captions and
saved transcripts remain beneath it. Readiness gates and session behavior are unchanged.
All 1.4.1 UI, profile, history and localized date improvements remain included.

- [x] Implement persistent session controls and update usage documentation.
- [x] Synchronize manifests/lockfiles and citation metadata to 1.4.2.
- [x] Prepare EN/FR/DE listing updates and GitHub release notes.
- [x] Confirm final automated validation and GitHub installer workflow completion.
- [x] Verify the combined x64/ARM64 bundle, embedded 1.4.2.0 manifests and SHA-256.
- [ ] Test installed x64 and ARM64 packages: Start/Stop while scrolled, narrow windows,
  enlarged text, audio/overlay behavior, history and profile persistence/export, localized tray.
- [ ] Capture fresh packaged-app EN/FR/DE screenshots; complete native German review.
- [ ] Run Windows App Certification Kit and check the public privacy policy.
- [ ] Submit **Live.Translation_1.4.2.msixbundle** and prepared listing fields manually in
  Partner Center; record certification and rollout. This release has not been submitted.

### Verified 1.4.2 artifacts — 21 September 2026

Release commit: `c3ea981`. [CI](https://github.com/fmadore/Live-translation/actions/runs/35647744590)
and [all installer/MSIX/bundle jobs](https://github.com/fmadore/Live-translation/actions/runs/35647744806)
completed successfully. Local frontend validation passed 337 tests, formatting, type checks
and the production build. The release includes EXE, MSI, separate x64/ARM64 MSIX packages,
and the combined Store bundle.

Upload [Live.Translation_1.4.2.msixbundle](https://github.com/fmadore/Live-translation/releases/download/v1.4.2/Live.Translation_1.4.2.msixbundle)
to Partner Center after the remaining native acceptance checks. Downloaded size:
**7,947,693 bytes**. SHA-256 verified against GitHub:
`63d7a38a345b4135d962233aa8d24dfca0f2c48536d2b7ab0eed631cbd03b7b4`.

Both embedded manifests report **1.4.2.0**, architectures **x64** and **arm64**,
identity `49346FMadore.LiveTranslationSubtitles`, publisher
`CN=5D0ECC96-3998-452E-B7E9-29BE9B576F86`, and publisher display name `FMadore`.
Both include `live-translation.exe`, declare only `microphone` and `runFullTrust`,
and are unsigned for Store signing. This verifies packaging, not installed-app operation
or Store certification. EN/FR/DE What's new fields are within the 1,500-character limit.

## Release 1.4.1 handoff

Historical record; the current submission target is [1.4.2](#release-142-handoff).
The 1.4.1 combined bundle is now available on its release page; its GitHub SHA-256 is
`3f6827fd76ce8852f7a2eedb73cb97db4d28c23e24b7af90ac5af4d8eb3631d6`.

Next Store submission target: **1.4.1**, MSIX **1.4.1.0**, prepared 20 September 2026.
This replaces the unsubmitted 1.4.0 target. The last documented Store release remains 1.2.4;
v1.4.0 is published on GitHub and its existing tag/assets must not be overwritten.
GitHub release: [v1.4.1](https://github.com/fmadore/Live-translation/releases/tag/v1.4.1),
20 September 2026. Installer workflow completion must be verified separately.
Version 1.4.1 has not been submitted to Partner Center.

Publication checkpoint, 20 September 2026: tag `v1.4.1` points to `fb35c7a`.
The release page is public. [Release installers](https://github.com/fmadore/Live-translation/actions/runs/35529783692)
and [release-commit CI](https://github.com/fmadore/Live-translation/actions/runs/35529783699)
were still running at this documentation check, with no release assets attached yet.
Follow those runs for current status; no final bundle digest or manifest verification is
recorded until the tagged artifacts exist.

This patch includes the [18-point operator UI audit](ui-audit-implementation.md): tabbed
Settings, profiles above setup with save/rename/delete actions, a session browser in History,
visible appearance presets/previews, curated colour swatches, consistent controls/type/labels,
and localized tray actions with session status. All 1.4.0 features remain included.

- [x] Implement the UI audit and correct accented labels in UTF-8.
- [x] Update EN/FR/DE Store copy, GitHub release draft, usage docs and screenshot instructions.
- [x] Synchronize app manifests and root lockfile versions to 1.4.1.
- [x] Record the final 1.4.1 build and validation results below.
- [ ] Verify the built app on Windows: settings/tab keyboard navigation, live appearance, profile save/rename/load/delete, history filters/export/delete and localized tray status/actions.
- [ ] Complete the inherited 1.4.0 native audio, overlay, mixed-DPI, accessibility, persistence and export checks on x64 and ARM64.
- [x] Publish v1.4.1 on GitHub at the user’s request; update CITATION.cff to 1.4.1 / 20 September 2026. Native acceptance remains pending.
- [ ] Verify release workflow, both embedded 1.4.1.0 manifests and SHA-256 for **Live.Translation_1.4.1.msixbundle**.
- [ ] Capture final-package EN/FR/DE screenshots; existing PNGs are historical 1.2.2 captures. Complete native German review.
- [ ] Run Windows App Certification Kit and verify the public privacy-policy page.
- [ ] Upload the combined bundle and [prepared listing fields](store-listing.md) manually; record certification and rollout.

Keep profile/history privacy statements and the feedback credit to @valentinrabot. This UI
patch introduces no extra audio storage or provider requests. A local executable/NSIS installer
is for native testing, not a replacement for the x64 + ARM64 Store bundle.

### Pre-publication follow-up fixes

Settings now retains a stable frame and tab bar with a scrollable body. Profile controls
share aligned heights. History date fields have localized EN/FR/DE format hints,
calendar selection, date validation and German date-filter regression coverage.
Final frontend validation: all 337 tests passed, production build succeeded, and
formatting and Svelte/TypeScript checks passed.
The local NSIS artifact recorded below predates these follow-up fixes; use the tagged
release workflow artifacts for the final 1.4.1 build.

### Earlier local build and validation — 20 September 2026

- Clean `npm ci`; frontend formatting and Svelte/TypeScript checks passed.
- All 333 frontend tests passed with `--maxWorkers=2`. The initial default-worker run
  encountered worker startup timeouts while the native build was compiling.
- Rust formatting, all-target/all-feature Clippy with warnings denied, and 73 Rust tests
  passed. The one billable provider probe remains ignored.
- `npm audit --audit-level=moderate` passed (three low-severity findings remain).
- `npm run tauri build -- --bundles nsis` completed, including the production frontend.
  Application file/product version: **1.4.1**.
- Local x64 installer: `src-tauri/target/release/bundle/nsis/Live Translation_1.4.1_x64-setup.exe`
  (**3,394,500 bytes**). SHA-256:
  `7e807763e462e16cecb9937dda7a31ee2110fa21ab75fc1268d6571786a80e5b`.

This confirms compilation and packaging, not installed-app acceptance or Store certification.

## Release 1.4.0 handoff

Historical preparation record; the next Store target is now [1.4.1](#release-141-handoff).

GitHub release: **1.4.0**, MSIX **1.4.0.0**, dated 19 September 2026.
[Release and installers](https://github.com/fmadore/Live-translation/releases/tag/v1.4.0).
Installer build completion and final native acceptance are separate from tagging/publication.
The last documented Store release is 1.2.4; **1.4.0 has not been submitted to Partner Center**.
This supersedes the 1.3.0 submission target; its record and bundle digest below are historical.

Included: named meeting profiles, 2–30 second completed-caption persistence, Steadier interim
updates, bright/dark appearance previews and presets, session titles and text/date/language
search, per-source live status and operator shortcuts. Gemini Smart transcription now gives
final results precedence and clears all-filler speculative text. Long history titles wrap and
date-picker icons remain visible. No new scroll-back controls or customizable word list are
included; [#85](https://github.com/fmadore/Live-translation/issues/85) remains open.

- [x] Implement features and document behavior, privacy and limitations.
- [x] Pass 331 frontend tests, formatting, Svelte/TypeScript checks and production build after the visual fixes.
- [x] Implementation validation: 73 Rust tests passed, one paid-provider test ignored, and all-features Clippy passed before the documentation/version preparation.
- [x] Impeccable browser checks: 1280×800, 800×600, French/German, 200% text size, synthetic history/profile/live flows and 1000×400 two-source overlay. See [verification](usability.md#verification).
- [x] Prepare EN/FR/DE Store copy and GitHub notes, retaining thanks to @valentinrabot for feedback.
- [x] Set app manifests and root lockfile versions to 1.4.0; dependency versions are unchanged.
- [ ] Verify final native packages on x64 and ARM64, including the existing audio/export/tray regressions.
- [ ] Save/load profiles across restart, disconnected devices and monitors; confirm application reselection and safe mixed-DPI placement.
- [ ] Verify persistence limits, Steadier updates, presets, reset and two-source layouts in the installed app.
- [ ] Verify history titles/search, restart persistence, rename during recording, copy/export/delete and opt-out with disposable transcripts.
- [ ] Verify all operator shortcuts, input/dialog guards, text scaling, keyboard focus and contrast themes in the installed app.
- [ ] Optionally run a fresh billable Gemini Smart transcription probe; record provider/model and results separately from contract tests.
- [ ] Capture final-package EN/FR screenshots and add DE screenshots; committed PNGs are still historical 1.2.2 captures. Complete native German review.
- [x] Preparation CI passed on `4e2c0db`; set citation version/date for the 1.4.0 publication.
- [ ] Verify publication, final release-commit CI and installer workflow completion.
- [ ] Verify the installer workflow, both embedded 1.4.0.0 manifests, Store identity, architectures and downloaded bundle SHA-256. Expected asset: **Live.Translation_1.4.0.msixbundle**; no digest exists yet.
- [ ] Run Windows App Certification Kit and verify the published privacy policy includes profile storage and session titles.
- [ ] Submit the bundle and [listing fields](store-listing.md) manually through Partner Center; record certification and rollout status.

History and local filler cleanup remain opt-in. Profiles store no API keys or application
process IDs and do not enable history/recovery. Gemini Smart cleanup occurs upstream, so
“raw” history/export means provider-returned text, not a guaranteed verbatim recording.
Citation metadata identifies 1.4.0 with the publication date 19 September 2026.

## Release 1.3.0 handoff

GitHub release: **1.3.0**, MSIX **1.3.0.0**, published 19 September 2026.
The prior documented Store release is 1.2.4. **1.3.0 has not been submitted to Partner Center.**

This release adds local transcript history (#81), Stable reading (#79), optional overlay
filler cleanup (#80), and the German interface added since 1.2.4. Thank **@valentinrabot**
for feedback and suggestions in the three feature issues, not for implementation.

- [x] Implement and test history, stable captions and display-only cleanup; keep history and cleanup opt-in.
- [x] Prepare EN/FR/DE listing descriptions, features, What's new, and certification instructions.
- [x] Document local retention, deletion and unchanged raw transcripts in the privacy policy.
- [x] Pass 314 frontend and 69 Rust tests, type checks, formatting, Clippy, production build and browser layout checks.
- [x] Patch devalue to 5.9.4; moderate-threshold npm audit passes (three inherited low cookie-chain findings remain).
- [x] Synchronize manifests, root lockfile versions and citation metadata to 1.3.0.
- [x] Publish v1.3.0 and verify CI plus the release installer workflow.
- [x] Download and inspect the final x64 + ARM64 bundle and record its SHA-256.
- [ ] Smoke-test the final packaged app: two saved demo sessions, restart/reopen, copy/export/delete, opt-out, interrupted save, Stable reading and cleanup settings.
- [ ] Complete native keyboard, scaling, mixed-DPI, contrast, tray and existing audio/export regression checks.
- [ ] Refresh EN/FR screenshots and add DE screenshots from the final MSIX; committed PNGs remain historical 1.2.2 captures.
- [ ] Obtain native German review of the interface and listing; German speech recognition remains unverified and is not newly advertised.
- [ ] Run Windows App Certification Kit against final packages.
- [x] Verify the public privacy-policy link shows the 19 September 2026 history-retention policy (HTTP 200 after Pages deployment).
- [ ] Upload **Live.Translation_1.3.0.msixbundle** and the prepared listing fields in Partner Center; submit and verify certification.

History and cleanup are off by default. History stores raw caption text and session metadata
locally until explicitly deleted; disabling it does not delete existing sessions. No audio is
saved and no additional provider requests are introduced. Keep these facts in the Store copy.
The default built-in demonstration still requires no key, account, microphone or network.

Release source: `03ed191`, tagged `v1.3.0`. [CI](https://github.com/fmadore/Live-translation/actions/runs/35438586678)
and the [release workflow](https://github.com/fmadore/Live-translation/actions/runs/35438589774)
passed. The [published release](https://github.com/fmadore/Live-translation/releases/tag/v1.3.0)
contains EXE/MSI installers, both MSIX packages and the combined Store bundle. Issues #79,
#80 and #81 are closed with release links and thanks to @valentinrabot.

The downloaded **Live.Translation_1.3.0.msixbundle** and both embedded package manifests
identify the Store name and publisher, version **1.3.0.0**, and **x64 + ARM64** architectures.
Its SHA-256 matches the GitHub asset digest:
`bad9b6a32c241e49750741c9e8d8344eddbb110e377b2ef9caa3badfa6720369`.
This verifies package contents and download integrity; the manual acceptance and Store
submission checks above remain pending.

## Release 1.2.4 handoff — done

Released and live. Kept for the record of what was checked. Keep issue #77 open for the
maintainer’s reply.
GitHub release v1.2.4 was published on 14 September 2026 with the feedback credit.
The Store-signed copy installed on the test PC was 1.2.2.0; that local observation
is not a claim about the version currently offered to every Store customer.

This update adds responsive captions for [#77](https://github.com/fmadore/Live-translation/issues/77).
Thank **@valentinrabot for the feedback and suggestion**, not for implementation.
The exact acknowledgment is included in the GitHub and EN/FR Store release copy.

- [x] Implement Fit window, optional Compact layout, bounded recent context, and EN/FR controls.
- [x] Upgrade the test runner to stable Vitest 5.0.0 with the jest-dom matcher type bridge.
- [x] Pass 287 frontend tests, Svelte checks, formatting, and the production frontend build.
- [x] Verify browser resizing at 600 × 260, 1200 × 600 and 1200 × 850, Compact mode, and large text.
- [x] Build a native ARM64 release executable and install/launch a separate local test copy.
- [x] Prepare README, layout documentation, EN/FR listing fields, certification steps and release notes.
- [x] Synchronize application manifests and root app versions in both lockfiles to 1.2.4.
- [ ] Verify the 160px initial overlay and Align to bottom reset to roughly two lines per visible source.
- [ ] User confirms the responsive captions in the installed local test app.
- [ ] Verify final x64 and ARM64 MSIX packages using the [caption layout matrix](caption-layout.md#release-verification).
- [ ] Complete native display scaling, mixed-DPI, font, keyboard and contrast checks.
- [ ] Recheck demo, live captions, Save As, transcript export, application capture and tray behavior.
- [ ] Refresh both screenshot sets, especially the overlay and appearance images; existing PNGs are the 1.2.2 set.
- [ ] Run Windows App Certification Kit against the final packages.
- [x] Commit/tag v1.2.4, push, and verify the GitHub release workflow and both architectures.
- [x] Update CITATION.cff to 1.2.4 with the release date 14 September 2026.
- [x] Download and inspect Live.Translation_1.2.4.msixbundle; publish the prepared GitHub release body with the feedback credit.
- [ ] Publish the updated privacy documentation and verify the Store privacy-policy link.
- [x] Upload the bundle and EN/FR copy manually in Partner Center; submit and verify certification. **Accepted; 1.2.4 is live.**

One loose end the repository can see and Partner Center cannot: **the committed PNGs were not
refreshed for 1.2.4.** `chore(release): 1.2.4` touched only
`docs/store-screenshots/README.md`, not a single image, and the last actual capture was
`0382eac` on 8 September. So whatever screenshots the live listing carries, the repository no
longer answers "do these still match the build?" — which is the entire reason the sets are kept
here. Re-capture from the live 1.2.4 MSIX and commit, or note deliberately that the listing
still shows an older UI.

The installed **Live Translation Local Test** was updated on 14 September to **1.2.4**,
including the shallow bottom-alignment preset. It is an unpackaged ARM64 executable with a
separate application identity, not a 1.2.4 MSIX or a Store submission artifact. Launching it
successfully does not establish that the manual caption tests passed. The Store installation
was preserved. Build the final packages from the normal configuration, without the ignored
local-test override under src-tauri/target/.

Use **Live.Translation_1.2.4.msixbundle** containing x64 and ARM64 for Partner Center.
The local executable and a single-architecture test bundle are not substitutes.

Release source: `75ef719`, tagged `v1.2.4`. [CI](https://github.com/fmadore/Live-translation/actions/runs/34818981260)
and the [release workflow](https://github.com/fmadore/Live-translation/actions/runs/34818980978)
passed. The downloaded bundle's manifest identifies both x64 and ARM64 packages at
1.2.4.0 with the Store identity. Its SHA-256 matches the GitHub asset digest:
`e2e6b88c9885c4d247094156229007e1cf6c3d31db0842fe6c6cfed4574a724c`.
This verifies package contents and download integrity; the manual checks above remain pending.

## Release 1.2.3 handoff

Historical record; use the 1.2.4 handoff above for current release work.

Previous release target: **1.2.3** (MSIX **1.2.3.0**). The user confirmed that the prior
Store version was updated, and that Save As and application selection work on the local
ARM64 1.2.3.0 test package. The subsequent start/rehearsal layout fix was visually checked
in English and French and is included in the release source.

- [x] Implement native Save As and timed SRT/VTT export (#26).
- [x] Implement capture of a selected application and its child processes (#27).
- [x] Prepare English/French [listing fields](store-listing.md) and [release notes](release-notes.md).
- [x] Synchronize application manifests and lockfiles to 1.2.3.
- [ ] Verify release workflow success and download the **x64 + ARM64** bundle.
- [ ] Smoke-test the final package, including the layout fix. Full NSIS/x64 and audio
      isolation/device-transition matrices remain unverified; see
      [export checks](transcript-export.md#verification) and
      [application capture checks](application-capture.md#verification).
- [ ] Refresh screenshots that show the changed idle controls or transcript toolbar in
      both languages; review captions against the actual images. Existing images remain
      the 1.2.2 set, not new captures of 1.2.3.
- [ ] Submit the bundle and both listing languages manually in Partner Center.

Use Live.Translation_1.2.3.msixbundle from the GitHub release. The locally generated
ARM64-only test bundle is not the multi-architecture Store artifact.

## The route

1. **Bump the version** in `package.json`, `src-tauri/Cargo.toml` and
   `src-tauri/tauri.conf.json`, and synchronize the root app versions in `package-lock.json`
   and `src-tauri/Cargo.lock`. All three manifests must match and be **higher than the
   version already in the Store** — Partner Center rejects a package that does not increase.
   The first segment can never be `0`. Commit them together as `chore(release): X.Y.Z` so the
   bump is one reviewable change rather than three scattered ones.
2. **Tag and push after manual testing**: `git tag vX.Y.Z` then `git push origin vX.Y.Z` (replace `X.Y.Z` with the tested version; do not reuse the published `v1.2.2` tag). The
   [`Release installers`](../.github/workflows/release.yml) workflow builds the NSIS installer,
   both architectures' `.msix`, and one multi-architecture
   `Live.Translation_<version>.msixbundle`, and attaches them to the GitHub release. Download
   the bundle from there.
3. **Paste the release body.** The workflow creates the release with an empty one; the
   paste-ready text is in [`release-notes.md`](release-notes.md). This is a different audience
   from the Store listing — people who already have the app — so it is written separately
   rather than reused.
4. **Sideload-test before submitting.** Install the bare per-architecture `.msix` with
   [`scripts/install-local-msix.ps1`](../scripts/install-local-msix.ps1), *not* the bundle —
   signing a bundle does not sign the packages inside it. Walk the manual release checklist in
   [`accessibility.md`](accessibility.md#release-checklist-manual-on-windows) against this
   install, because package identity changes how Windows treats the window.
5. **Re-capture the screenshots** if anything they show has changed — see
   [Screenshots](#screenshots). Every listing language.
6. Partner Center → **Apps and games** → *Live Translation & Subtitles* → **Create new
   submission**. In French: **Applications et jeux**, **Créer une soumission**.
7. Under **Packages**, **delete every package already listed**, then upload the single
   `.msixbundle`. Deleting matters: 1.0.5 went up as two separate per-architecture packages,
   and leaving one behind ships a stale architecture silently. One bundle in, nothing else.
8. **Paste the listing text** for every language if it changed. The paste-ready copy is in
   [`store-listing.md`](store-listing.md), and **What's new in this version** has to be
   rewritten for every release, in every language the listing has. Editing that file changes
   nothing on its own; the Store only knows what is typed into Partner Center.
9. Set a **gradual rollout** percentage for anything touching audio capture or the session
   lifecycle. Start at 10% and raise it once the crash and review data look clean.
10. **Submit to the Store**, then watch certification. A clean submission is not a pass — 1.0.3
   uploaded perfectly and then failed policy 10.1.2.20.

[`partner-center-walkthrough.md`](partner-center-walkthrough.md) has the screen-by-screen
version of steps 6–10, including the French labels.

## Screenshots

The Store listing is per-language, so it needs its own screenshots in each language, and they
are kept in the repository:

```text
docs/store-screenshots/en/    English listing
docs/store-screenshots/fr/    French listing
```

Five per language, named for the order they are uploaded in — the order is what a visitor
scrolls through, so it is part of the listing rather than an implementation detail:

| File | What it shows |
| --- | --- |
| `1-idle.png` | The pre-flight screen while idle: provider key, microphone, overlay placement, hourly cost. |
| `2-running.png` | A running demo: Demo status, a moving meter, elapsed time, and a caption. |
| `3-overlay.png` | The overlay in placement mode: drag handles, size readout, and the stand-in caption. |
| `4-appearance.png` | Caption appearance: Fit window / Compact, size, typeface, caption colour and backing; line width appears only in Compact. |
| `5-contrast.png` | The contrast readout, naming a step that falls below its target. |

Three of those five are there to make an argument rather than to describe a screen. The
appearance panel and its contrast readout have no equivalent in this category, and a listing
that stops at the demo never mentions them; the overlay shot is the captions where the room
reads them, rather than the window that produces them.

**One current set, overwritten in place.** They are re-captured rather than accumulated, so the
history holds previous sets. The working tree currently contains historical 1.2.2 captures;
replace them only with verified final-package screenshots and update the capture record.
The alternative — a folder per version — grows the repository by a few megabytes every release
to preserve something nobody reads twice.

Captured on Windows at the display scaling the app is actually used at, from a **sideloaded
MSIX** rather than a dev build, for the same reason the accessibility walk uses one: package
identity changes how the window is drawn. Store screenshots must be `.png`, at least 1366×768
and under 50 MB.

Two Store rules shape the framing rather than just the acceptance. Anything that matters
belongs in the **top two-thirds**, because the Store draws its own text overlays across the
bottom third. And no logo, wordmark or marketing line may be burned into the image — the
per-screenshot Description field in Partner Center carries that instead, in 200 characters,
doubling as the alt text a screen reader announces. Ten desktop screenshots are the ceiling
and Microsoft recommends five to eight.

Re-capture when the UI in them changes — that is a checklist item in
[`microsoft-store.md`](microsoft-store.md). Never upload screenshots from the removed Windows
Speech implementation.

## Why by hand

Verified 27 August 2026 while setting 1.1.0 up.

The submission API authenticates as a Microsoft Entra application, which has to live in a
tenant associated with the Partner Center account. Tenant association, user management and
Entra applications are **Company-account features**. This is an Individual account, and
Microsoft is explicit that
[Entra ID sign-up "is currently supported only for Company accounts"](https://learn.microsoft.com/en-us/windows/apps/publish/partner-center/open-a-developer-account?tabs=individual).
In the dashboard it shows up as `account-settings/organization/tenant-management` rendering a
blank page. Nothing is misconfigured; the feature is not there.

**Converting is not a way out.** Partner Center does not support changing an account from
Individual to Company — it needs a *new* account, which means a new publisher identity, which
means a new Store listing at a new URL. The current identity
(`49346FMadore.LiveTranslationSubtitles`) is baked into every package shipped so far, so
switching would abandon the published listing and its installs. Not a trade worth making for
this app.

A `Microsoft Store submission` workflow driving the `msstore` CLI was written and then removed,
because a workflow that cannot run reads as an option that exists. It is in the git history if
the constraint ever lifts; [If the account ever becomes a
Company account](#if-the-account-ever-becomes-a-company-account) has what it needed.

## Why the package is one bundle

The Store submission is a single multi-architecture `.msixbundle` containing both the x64 and
the ARM64 package, rather than two single-architecture bundles.

Partner Center treats packages in a submission as a set to be replaced, and it is on the person
doing the replacing to remove the old ones — which is why step 6 says delete every package
already listed. One bundle in and nothing else leaves no room for an architecture to survive
from the previous submission and ship stale. 1.0.5 went up as two per-architecture packages and
had to be cleaned up by hand for exactly this reason.

## Constraints worth knowing

- **One pending submission per product.** Partner Center allows one open submission at a time.
  Finish or delete the pending one before starting another.
- **Certification can still fail.** An accepted upload is not a pass. 1.0.3 uploaded perfectly
  well and then failed policy 10.1.2.20 — the reviewer pressed **Start Subtitles** and nothing
  happened on their device. See [`microsoft-store.md`](microsoft-store.md) for what that cost
  and what the bundled demonstration exists to prevent.
- **The listing is not in this repository.** `store-listing.md` is paste-ready source text, not
  a deployment. Nothing in `docs/` reaches the Store without someone typing or uploading it.

## If the account ever becomes a Company account

Kept because it is correct, and because the constraint is Microsoft's rather than ours. None of
this is doable today.

1. **Give the Partner Center account a Microsoft Entra tenant.** Partner Center → **Account
   settings** → **Tenants**. Either
   [associate an existing tenant](https://learn.microsoft.com/en-us/windows/apps/publish/partner-center/associate-existing-azure-ad-tenant-with-partner-center-account)
   or [create one](https://learn.microsoft.com/en-us/windows/apps/publish/partner-center/create-new-azure-ad-tenant).
   The API authenticates against the tenant, not the Microsoft account used to sign in.
2. **Register an application in the tenant.** [Entra admin center](https://entra.microsoft.com/)
   → **Identity** → **Applications** → **App registrations** → **New registration**. No
   redirect URI and no API permissions; it exists only to hold a client secret. Then
   **Certificates & secrets** → **New client secret**, copied immediately because it is shown
   once, and noted for its expiry.
3. **Give that application the Manager role.** Partner Center → **Account settings** → **User
   management** → **Microsoft Entra applications**. Without it the credentials authenticate and
   every submission call is refused.
4. **Add four repository secrets** — `AZURE_AD_TENANT_ID`,
   `AZURE_AD_APPLICATION_CLIENT_ID`, `AZURE_AD_APPLICATION_SECRET` (step 2) and `SELLER_ID`
   (Partner Center → Account settings → Identifiers).

Two things the removed workflow had learned, worth keeping for whoever writes the next one:
Microsoft supports Store update operations through GitHub Actions for **free products only**,
which this app is; and the `msstore` CLI is in preview with documentation that trails it —
Learn documents an `--inputFile` option for `msstore publish` that the released CLI does not
have, taking the package path as a positional argument instead.

`msstore submission updateMetadata` could also replace the listing text from a committed JSON
file, which would make the Store description reviewable in a pull request. Worth wanting, but
a bad metadata push is more disruptive than a bad package push, because it goes live on the
product page rather than waiting for certification.

## Sources

- [Open a developer account](https://learn.microsoft.com/en-us/windows/apps/publish/partner-center/open-a-developer-account?tabs=individual)
- [Publish app updates to Microsoft Store with GitHub Actions](https://learn.microsoft.com/en-us/windows/apps/publish/msstore-dev-cli/github-actions)
- [Microsoft Store Developer CLI commands](https://learn.microsoft.com/en-us/windows/apps/publish/msstore-dev-cli/commands)
- [Package version numbering](https://learn.microsoft.com/en-us/windows/uwp/publish/package-version-numbering)
- [App screenshots, images, and trailers](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/screenshots-and-images)
