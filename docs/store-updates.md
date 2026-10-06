# Microsoft Store updates

Every update to this app is submitted **by hand** through Partner Center. That is not a
preference and not a stopgap — it is the only route this account has, for a reason Microsoft
enforces and we cannot work around. [Why by hand](#why-by-hand) has the detail; the short
version is that the submission API is a Company-account feature and this is an Individual
account.

The route below is what shipped every version so far. It takes about five minutes once the
packages are built.

## Release 1.6.1 handoff

**[v1.6.1 published on GitHub](https://github.com/fmadore/Live-translation/releases/tag/v1.6.1) on 6 October 2026; Store submission
pending.** Last confirmed live Store version: **1.6.0.0**, confirmed by the maintainer on
6 October 2026. Published app version: **1.6.1**, MSIX **1.6.1.0**.

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
- [x] App manifests and root lockfile entries set to 1.6.1; citation release date
  6 October 2026, the publication date.
- [x] No runtime dependency, package-manifest template or capability change since `v1.6.0`.
  Besides the version lines, the only lockfile change is the dev-only `source-map-js` 1.2.1 →
  1.2.2 (GHSA-68fv-2mgg-jv7q), which the CI audit step required; it is not shipped in the app.
- [x] Frontend validation on the #112 branch: 550 tests in 59 files, Svelte check (zero
  errors and warnings), Prettier and the production build.
- [x] #112, #113 and #114 merged as `3b3eab4`, `1632589` and `d206091`, each after all seven
  CI jobs passed: frontend on Node 22 and 24, Rust on Ubuntu, Windows x64 and Windows ARM64,
  Rust security and workflow lint.
- [x] Rust tests, formatting and Clippy passed in CI. They were not run locally: this machine
  has no LLVM/libclang, which the Whisper bindings need since #99.

### Remaining Store acceptance

- [x] Merge #112, then the release PR (#113) and the docs trim (#114).
- [x] Tag `v1.6.1` from `main`. Release-commit CI and all four installer/MSIX/bundle jobs
  passed; the downloaded assets are verified below and the prepared GitHub release body is
  published.
- [x] Verify `Live.Translation_1.6.1.msixbundle` contains native x64 and ARM64 packages at
  **1.6.1.0** with the [assigned identity](microsoft-store.md#store-identity-assigned), and
  that both carry the new `Square44x44Logo`, `Square150x150Logo`, `Wide310x150Logo` and
  `StoreLogo` assets. Sizes and SHA-256 are recorded below.
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

Tag `v1.6.1` points to `d20609195cbbc20f9b1b9a44c5934f2ee6dd5c65`. The
[release-commit CI](https://github.com/fmadore/Live-translation/actions/runs/37418786176) and all four
[installer/MSIX/bundle jobs](https://github.com/fmadore/Live-translation/actions/runs/37418813127) passed. The GitHub
release body is [release-1.6.1.md](release-1.6.1.md).

Upload **[Live.Translation_1.6.1.msixbundle](https://github.com/fmadore/Live-translation/releases/download/v1.6.1/Live.Translation_1.6.1.msixbundle)**
to Partner Center. These are the published CI assets; each downloaded file's SHA-256 matches
GitHub's asset digest. Local copies and `verification.json` are in the ignored directory
`src-tauri/target/release-1.6.1/published/`.

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| [Live.Translation_1.6.1.msixbundle](https://github.com/fmadore/Live-translation/releases/download/v1.6.1/Live.Translation_1.6.1.msixbundle) | 11,195,249 | `86ef814f0e53baae6ff821db8cc263efa745982a7dfea945d8fda7d69a6ccea0` |
| [Live.Translation_1.6.1_x64.msix](https://github.com/fmadore/Live-translation/releases/download/v1.6.1/Live.Translation_1.6.1_x64.msix) | 5,692,715 | `fb2e4958fab3e1832dffff0ec1f3c42c84d7ac3af3fa8f434407e4b1670ed5dd` |
| [Live.Translation_1.6.1_arm64.msix](https://github.com/fmadore/Live-translation/releases/download/v1.6.1/Live.Translation_1.6.1_arm64.msix) | 5,500,528 | `2d7451224fa8039e7314dd3b0602e28a67e6b43ed936ec7a448d6f70b88cc50c` |

All 37 checks passed. The bundle contains exactly the separately downloaded x64 and ARM64
packages, byte-for-byte. Bundle and inner manifests report **1.6.1.0**, identity
`49346FMadore.LiveTranslationSubtitles`, publisher `CN=5D0ECC96-3998-452E-B7E9-29BE9B576F86`
and publisher display name `FMadore`. Executable PE machine values are `0x8664` (x64,
12,067,328 bytes) and `0xAA64` (ARM64, 10,647,040 bytes). Each package includes both
rehearsal WAVs and all three Whisper license notices, declares only `runFullTrust` and
`microphone`, contains no model weights or signature, and carries the new
`Square44x44Logo`, `Square150x150Logo`, `Wide310x150Logo` and `StoreLogo`, byte-identical to
`src-tauri/gen/windows/Assets/`. This verifies packaging, not installed-app behaviour or
certification.

## The route

1. **Bump the version** in `package.json`, `src-tauri/Cargo.toml` and
   `src-tauri/tauri.conf.json`, and synchronize the root app versions in `package-lock.json`
   and `src-tauri/Cargo.lock`. All three manifests must match and be **higher than the
   version already in the Store** — Partner Center rejects a package that does not increase.
   The first segment can never be `0`. Commit them together as `chore(release): X.Y.Z`.
2. **Tag and push after manual testing**: `git tag vX.Y.Z` then `git push origin vX.Y.Z`,
   with the tested version; never reuse a published tag. The
   [`Release installers`](../.github/workflows/release.yml) workflow builds the NSIS installer,
   both architectures' `.msix`, and one multi-architecture
   `Live.Translation_<version>.msixbundle`, and attaches them to the GitHub release. Download
   the bundle from there.
3. **Paste the release body.** The workflow creates the release with an empty one; the
   paste-ready text and how to apply it are in [`release-notes.md`](release-notes.md).
4. **Sideload-test before submitting.** Install the bare per-architecture `.msix` with
   [`scripts/install-local-msix.ps1`](../scripts/install-local-msix.ps1), *not* the bundle —
   signing a bundle does not sign the packages inside it. Walk the manual release checklist in
   [`accessibility.md`](accessibility.md#release-checklist-manual-on-windows) against this
   install, because package identity changes how Windows treats the window.
5. **Re-capture the screenshots** in every listing language if anything they show has
   changed — see [Screenshots](#screenshots).
6. Partner Center → **Apps and games** → *Live Translation & Subtitles* → **Create new
   submission**. In French: **Applications et jeux**, **Créer une soumission**.
7. Under **Packages**, **delete every package already listed**, then upload the single
   `.msixbundle`. One bundle in, nothing else — see
   [Why the package is one bundle](#why-the-package-is-one-bundle).
8. **Paste the listing text** for every language if it changed, from
   [`store-listing.md`](store-listing.md). **What's new in this version** has to be rewritten
   for every release, in every listing language. Editing that file changes nothing on its own;
   the Store only knows what is typed into Partner Center.
9. Set a **gradual rollout** percentage for anything touching audio capture or the session
   lifecycle. Start at 10% and raise it once the crash and review data look clean.
10. **Submit to the Store**, then watch certification: an accepted upload is not a pass.

[`partner-center-walkthrough.md`](partner-center-walkthrough.md) has the screen-by-screen
version of steps 6–10.

## Screenshots

The Store listing is per-language, so each language has its own set under
`docs/store-screenshots/<language>/`, named in upload order — the order is what a visitor
scrolls through. The [capture plan](store-screenshots/README.md) lists the current set and its
status. Each set is re-captured and overwritten in place; git history holds the previous ones.

Capture on Windows at the display scaling the app is actually used at, from a **sideloaded
MSIX** rather than a dev build: package identity changes how the window is drawn. Store rules:

- `.png`, at least 1366×768 and under 50 MB. Ten desktop screenshots at most; Microsoft
  recommends five to eight.
- Anything that matters belongs in the **top two-thirds**, because the Store draws its own text
  overlays across the bottom third.
- No logo, wordmark or marketing line burned into the image. The per-screenshot Description
  field in Partner Center carries that instead, in 200 characters, doubling as the alt text a
  screen reader announces.

Re-capture whenever the UI they show changes; the
[acceptance checklist](microsoft-store.md#acceptance-checklist) includes it.

## Why by hand

Verified 27 August 2026.

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
switching would abandon the published listing and its installs.

A `Microsoft Store submission` workflow driving the `msstore` CLI was removed because a
workflow that cannot run reads as an option that exists. It is in the git history;
[If the account ever becomes a Company account](#if-the-account-ever-becomes-a-company-account)
has what it needed.

## Why the package is one bundle

The Store submission is a single multi-architecture `.msixbundle` containing both the x64 and
the ARM64 package, rather than two single-architecture bundles.

Partner Center treats packages in a submission as a set to be replaced, and it is on the person
doing the replacing to remove the old ones — which is why step 7 says delete every package
already listed. One bundle in and nothing else leaves no room for an architecture to survive
from the previous submission and ship stale. 1.0.5 went up as two per-architecture packages and
had to be cleaned up by hand for exactly this reason.

## Constraints worth knowing

- **One pending submission per product.** Finish or delete the pending one before starting
  another.
- **Certification can still fail.** 1.0.3 uploaded perfectly well and then failed policy
  10.1.2.20 — the reviewer pressed **Start Subtitles** and nothing happened on their device.
  See [`microsoft-store.md`](microsoft-store.md) for what the bundled demonstration exists to
  prevent.
- **The listing is not in this repository.** `store-listing.md` is paste-ready source text, not
  a deployment. Nothing in `docs/` reaches the Store without someone typing or uploading it.

## If the account ever becomes a Company account

None of this is possible on an Individual account; it is kept because it is correct.

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

What the removed workflow had learned: Microsoft supports Store update operations through
GitHub Actions for **free products only**, which this app is; and the `msstore` CLI is in
preview with documentation that trails it — Learn documents an `--inputFile` option for
`msstore publish` that the released CLI does not have, taking the package path as a positional
argument instead.

`msstore submission updateMetadata` could also replace the listing text from a committed JSON
file, making the Store description reviewable in a pull request. But a bad metadata push is
more disruptive than a bad package push: it goes live on the product page rather than waiting
for certification.

## Release history

Earlier handoffs, one row each. Their full checklists, CI runs, artifact sizes and SHA-256
digests are in this file's
[commit history](https://github.com/fmadore/Live-translation/commits/main/docs/store-updates.md);
each release page has the assets and the published body. Checks left unticked when a version
was submitted were not recorded as done and stay open against the Store build.

| Version | GitHub release | Store | Summary |
| --- | --- | --- | --- |
| 1.6.0 | [4 October 2026](https://github.com/fmadore/Live-translation/releases/tag/v1.6.0) · [body](release-1.6.0.md) | Live as 1.6.0.0, confirmed 6 October 2026 | Local Whisper (#99), two translation targets, original speech in overlay and exports, Pause/Resume, editable filler words. |
| 1.5.1 | [23 September 2026](https://github.com/fmadore/Live-translation/releases/tag/v1.5.1) · [body](release-1.5.1.md) | Live as 1.5.1.0, confirmed 23 September 2026 | Batches 1–2 of the [22 September app review](app-review-2026-09-22.md) (#86): cleaner downsampled audio, release builds ignore `.env` and host overrides, a failing capture or provider thread ends only its source. |
| 1.5.0 | [22 September 2026](https://github.com/fmadore/Live-translation/releases/tag/v1.5.0) · [body](release-1.5.0.md) | Not submitted | Searchable provider-scoped caption languages, favourites, F2 favourite swapping and RTL overlay direction (#78). |
| 1.4.2 | [21 September 2026](https://github.com/fmadore/Live-translation/releases/tag/v1.4.2) | Not submitted | Start and Stop in a persistent action bar below the header. |
| 1.4.1 | [20 September 2026](https://github.com/fmadore/Live-translation/releases/tag/v1.4.1) | Not submitted | The [18-point operator UI audit](ui-audit-implementation.md): tabbed Settings, profiles above setup, a session browser, localized tray. |
| 1.4.0 | [19 September 2026](https://github.com/fmadore/Live-translation/releases/tag/v1.4.0) | Not submitted | Meeting profiles, 2–30 second caption persistence, appearance presets, history titles and search, operator shortcuts. |
| 1.3.0 | [19 September 2026](https://github.com/fmadore/Live-translation/releases/tag/v1.3.0) | Not submitted | Transcript history (#81), Stable reading (#79), overlay filler cleanup (#80) and the German interface. |
| 1.2.4 | [14 September 2026](https://github.com/fmadore/Live-translation/releases/tag/v1.2.4) | Live; acceptance date not recorded | Responsive captions: Fit window and optional Compact layout (#77). |
| 1.2.3 | [8 September 2026](https://github.com/fmadore/Live-translation/releases/tag/v1.2.3) | No submission recorded | Native Save As with timed SRT/VTT export (#26); capture of a selected application (#27). |

### Release 1.6.0 handoff

Live as 1.6.0.0, confirmed by the maintainer on 6 October 2026; the submission, certification
and go-live dates were not recorded. Tag `v1.6.0` points to `41cee07`; release-commit CI and
all installer/MSIX/bundle jobs passed, and each downloaded asset's SHA-256 matched GitHub's
digest. Native x64/ARM64 hardware tests, cloud language acceptance (#78), keyboard/Narrator,
enlarged-text and mixed-DPI checks, final screenshots and Windows App Certification Kit were
not recorded as done before submission.

### Release 1.5.1 handoff

Live as 1.5.1.0, confirmed 23 September 2026; it replaced 1.2.4 in the Store. Tag `v1.5.1`
points to `e6a4e18`. Keeping panic unwinding (review item D10) grew the executable; net of
shipping only `.woff2` fonts, the bundle went from 7.96 MB to 9.22 MB. Installed-package
checks, the OpenAI target-code probe, live non-EN/FR speech (#78), screenshots and Windows App
Certification Kit were not recorded as done before submission.

### Release 1.5.0 handoff

Not submitted; superseded by 1.5.1, which carried its open checks. Tag `v1.5.0` points to
`d1ad615`.

## Sources

- [Open a developer account](https://learn.microsoft.com/en-us/windows/apps/publish/partner-center/open-a-developer-account?tabs=individual)
- [Publish app updates to Microsoft Store with GitHub Actions](https://learn.microsoft.com/en-us/windows/apps/publish/msstore-dev-cli/github-actions)
- [Microsoft Store Developer CLI commands](https://learn.microsoft.com/en-us/windows/apps/publish/msstore-dev-cli/commands)
- [Package version numbering](https://learn.microsoft.com/en-us/windows/uwp/publish/package-version-numbering)
- [App screenshots, images, and trailers](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/screenshots-and-images)
