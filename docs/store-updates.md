# Microsoft Store updates

Every update to this app is submitted **by hand** through Partner Center. That is not a
preference and not a stopgap — it is the only route this account has, for a reason Microsoft
enforces and we cannot work around. [Why by hand](#why-by-hand) has the detail; the short
version is that the submission API is a Company-account feature and this is an Individual
account.

The route below is what shipped every version so far. It takes about five minutes once the
packages are built.

## Release 1.8.0 handoff

**Live in the Store.** Tagged and published on GitHub on 21 October 2026; the maintainer reported
the Store release, and `Get-AppxPackage` on their machine reports the Store-signed **1.8.0.0**.
The previous Store version was **1.7.1.0**. Target app version: **1.8.0**,
MSIX **1.8.0.0**. What's new and the certification notes cover the changes since 1.7.1.

1.8.0 is two features:

- [#155](https://github.com/fmadore/Live-translation/pull/155) (closes #124): **Live** and
  **Transcript** tabs while a session runs. Transcript gives the whole log the full stage, with
  the turns still being spoken at its foot. **Copy** writes the last 5, 10 or 30 minutes, or the
  whole transcript, as plain text, during a session and after Stop. Selecting text holds the
  reading position, Ctrl+A selects only the log, and original speech shows under each
  paragraph when exports include it. Interface only; see [transcript export](transcript-export.md).
- [#156](https://github.com/fmadore/Live-translation/pull/156) (closes #153): Mistral subtitles
  move to **Voxtral Mini Transcribe Realtime 3** (`voxtral-mini-transcribe-realtime-3-0`).
  - **Rate:** $0.48 an hour of audio instead of $0.36.
  - **Languages:** 45 languages and dialects by Mistral's count, instead of 13.
  - **Line closing:** a line closes at Mistral's detected end of turn, and is held open
    mid-sentence. The shared realtime runner gains the mechanism (`Signal::EndOfTurn`,
    `keeps_turn_open`), which only Mistral uses.
  - **Names and terms:** a list in step 03 of the setup is sent as `context_bias`, and is
    saved with the setup and in meeting profiles.
  - See the [Mistral API notes](mistral-realtime-api.md).

**Known risk:** Mistral lists Realtime 3 as a **public preview**. The app pins `-3-0`, and
release builds cannot override the model. If Mistral withdraws or changes the preview, Mistral
subtitles stop with Mistral's error until an update ships. Whisper, Gemini, OpenAI and the demo
are unaffected.

**Unchanged:**
- No dependency changes.
- No new command permissions: Copy uses the webview's clipboard API.
- The package still declares only `runFullTrust` and `microphone`.

The [privacy policy](privacy.md) takes effect on 21 October 2026. It adds the names-and-terms
list, which is kept locally and sent to Mistral, and Copy, which puts transcript text on the
Windows clipboard.
[GitHub release copy](release-1.8.0.md) · [EN/FR/DE Store fields](store-listing.md) ·
[Partner Center walkthrough](partner-center-walkthrough.md).

### Preparation and validation

- [x] #155 and #156 merged, each after all its CI jobs passed.
- [x] App manifests, root lockfile entries and citation set to 1.8.0. The citation date,
  provisionally 21 October 2026, is corrected to the tag date when tagging.
- [x] Lockfiles: nothing changes besides the two root version lines (`git diff --text`), and
  `cargo check --locked` passes.
- [x] Validation of the release branch on Windows 11 ARM64:
  - `npm test`: 699 passed in 74 files.
  - `check` (0 errors, 0 warnings), `lint`, `knip`, `format:check`, `check:languages` and
    `build` pass.
  - Rust tests, Clippy and the style suite passed on #155 and #156 and in their CI runs; this
    preparation changes no code.
- [x] The Store fields are within Partner Center's limits.

### Testing

- [x] Merge the release PR ([#157](https://github.com/fmadore/Live-translation/pull/157),
  `502ba34`).
- [x] **No release candidate.** At the maintainer's request, `v1.8.0` was tagged straight from
  the merge of #157. A `v1.8.0-rc.1` tag pushed first was deleted, and its build cancelled,
  before it created a draft.
- [ ] **Maintainer test** of the side-by-side Local Test build of `502ba34` (ARM64), installed
  on 21 October 2026. Not reported yet.

Manual checks new in 1.8.0, on x64 and ARM64 unless a line says otherwise:

- [ ] **Mistral on real speech**, in French and in English:
  - a few names in the glossary, including one the model gets wrong without it;
  - captions and their line breaks, at the ends of sentences and of turns, including a quick
    Q&A exchange;
  - Pause, Resume and Stop;
  - a brief network drop, after which captions should resume.

  Note any `error` payload in the log, and whether the glossary fixed the name. This is also
  the open verification in the [roadmap](../ROADMAP.md#open-verification).
- [ ] **The Transcript tab in a long session**:
  - switching tabs while captions arrive;
  - each Copy window pasted into Notepad;
  - a selection holding still while new text arrives;
  - Copy after Stop;
  - with Both sources, and with two caption languages.
- [ ] **Narrator** on the Live and Transcript tabs, the row still being spoken, the Copy menu,
  and the glossary's added and removed announcements.

Carried over from 1.7.0 and 1.7.1, never recorded as done and still open against the Store
build:

- [ ] **A short live session with each other cloud provider** (Gemini Translate, Gemini
  Transcribe and OpenAI) with Pause, Resume and Stop. On Gemini Translate, Stop mid-sentence
  should translate the last words and end well before four seconds.
- [ ] **System audio on x64**: an output and one application, alone and with the microphone
  (ARM64 was tested on 20 October 2026).
- [ ] **A 24-bit microphone**: levels, Test audio and captions.
- [ ] **Whisper Small translating real speech into English**: quality, whether it keeps up, and
  the setup sheet's English-only note.
- [ ] **Overlay restore with a laptop at 150% and a projector at 100%**: the "Placement across
  launches" and "Display changes while running" rows of the
  [caption layout matrix](caption-layout.md#release-verification).
- [ ] **An x64 PC without AVX2**, if one is available: Whisper listed as unavailable with its
  reason, a first launch on the built-in demo, and a cloud engine that still starts.
- [ ] **A model download behind a proxy** set in Windows Settings.
- [ ] **Quitting with a large Whisper backlog**: the prompt gives the seconds pending, and the
  app exits a few seconds after closing.
- [ ] **Narrator** on Pause/Resume and Move overlay/Done, and on the input-status
  announcements.
- [ ] **The error sentences in English, French and German**: a failed recovery write, a
  failed history write and a failed export from the History tab.
- [ ] **The icon** in Start's app list, Alt+Tab, the tray and the Store tile, and on a light
  taskbar; move mode over a white and a dark slide; Settings tabs, steppers at their limits,
  keyboard focus, contrast themes and enlarged text.
- [ ] Fix what testing shows. `v1.8.0` is published, so a fix ships as 1.8.1; never move a
  tag.

### Release and Store acceptance

- [x] Tag `v1.8.0` (annotated: "Live Translation & Subtitles v1.8.0") on `502ba34`; the
  citation date, 21 October 2026, is the tag date. Release-commit CI and all installer/MSIX/bundle
  jobs passed.
- [x] Publish the prepared body: `gh release edit v1.8.0 --notes-file docs/release-1.8.0.md`,
  with the title "Live Translation & Subtitles 1.8.0".
- [x] Verify `Live.Translation_1.8.0.msixbundle` against
  [the 1.7.1 checks](#release-171-handoff), at **1.8.0.0**:
  - GitHub's asset digests;
  - a bundle holding exactly both packages, byte for byte, with no signature;
  - manifests with the [assigned identity](microsoft-store.md#store-identity-assigned) and
    publisher;
  - native PE machine types;
  - only `runFullTrust` and `microphone`;
  - the rehearsal WAVs and Whisper licence notices, and no model weights;
  - the 46 `Assets/` files byte-identical to the repository, and a single `resources.pri`.

  Record sizes and SHA-256 below.
- [ ] Sideload-test the final per-architecture MSIX packages and walk the
  [accessibility release checklist](accessibility.md#release-checklist-manual-on-windows).
- [ ] **Screenshots:** the 21 October 2026 set predates the Live and Transcript tabs. Retake
  `3-bilingual.png`, which shows a running session, and `1-whisper.png` if the tabs show
  there, in every language, from the 1.8.0 MSIX. A Transcript-tab screenshot would show the
  new feature; it is optional. See [store-screenshots](store-screenshots/README.md).
- [ ] Run Windows App Certification Kit.
- [x] Submit the single combined bundle in Partner Center with the 1.8.0 What's new and the
  updated descriptions, features and certification notes. Use a gradual rollout (route
  step 9): the realtime runner's line closing changed, and Mistral's model is a preview.
- [x] Record submission and certification. Live: `Get-AppxPackage 49346FMadore.LiveTranslationSubtitles`
  reports `SignatureKind` `Store` and Version `1.8.0.0`.

### 1.8.0 artifact verification

Tag `v1.8.0` (annotated) points to `502ba346b19223fec5fc32e0f3bc2a0d7df9fe79`, the merge of
[#157](https://github.com/fmadore/Live-translation/pull/157). The
[release-commit CI](https://github.com/fmadore/Live-translation/actions/runs/39581397954) and
all four [installer/MSIX/bundle jobs](https://github.com/fmadore/Live-translation/actions/runs/39581877713)
passed. The GitHub release body is [release-1.8.0.md](release-1.8.0.md).

Upload **[Live.Translation_1.8.0.msixbundle](https://github.com/fmadore/Live-translation/releases/download/v1.8.0/Live.Translation_1.8.0.msixbundle)**
to Partner Center. These are the published CI assets; each downloaded file's SHA-256 matches
GitHub's asset digest. Local copies and `verification.json` are in the ignored directory
`src-tauri/target/release-1.8.0/published/`.

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| [Live.Translation_1.8.0.msixbundle](https://github.com/fmadore/Live-translation/releases/download/v1.8.0/Live.Translation_1.8.0.msixbundle) | 11,855,153 | `a8eaa3bd42e21cc10f344cf99b950adbaf914256994d2da6b4e4740102591121` |
| [Live.Translation_1.8.0_x64.msix](https://github.com/fmadore/Live-translation/releases/download/v1.8.0/Live.Translation_1.8.0_x64.msix) | 6,044,339 | `8b0ecb08b4f482c8d6e3143f579e3163a322d0c20795a6661d1f9cc06dca9b2a` |
| [Live.Translation_1.8.0_arm64.msix](https://github.com/fmadore/Live-translation/releases/download/v1.8.0/Live.Translation_1.8.0_arm64.msix) | 5,808,806 | `89be2f3c48b334a26de1e4a5dce35a52f1277cba4490aa6b96b7ddde0a002fd6` |

All 36 checks passed:
- **Digests:** GitHub's digests match for the bundle and both packages.
- **Bundle:** it holds exactly both packages, byte for byte, with no signature. Its 1.8.0.0
  manifest has the assigned identity and publisher.
- **Each package:**
  - the assigned identity, publisher, display name `FMadore`, version 1.8.0.0 and
    architecture;
  - only `runFullTrust` and `microphone`, and no signature;
  - both rehearsal WAVs and the three Whisper licence notices, and no model weights;
  - the 46 `Assets/` files, byte-identical to the repository, and a single `resources.pri`.
- **Executables:** PE machine `0x8664` (x64, 12,412,416 bytes) and `0xAA64` (ARM64,
  10,852,352 bytes).

This verifies packaging, not installed-app behaviour or certification.

## The route

1. **Bump the version** in `package.json`, `src-tauri/Cargo.toml` and
   `src-tauri/tauri.conf.json`, synchronize the root app versions in `package-lock.json`
   and `src-tauri/Cargo.lock`, and set `version` and `date-released` in `CITATION.cff`.
   `npm test` fails until all six files agree (`src/lib/version.test.ts`). All three manifests
   must match and be **higher than the version already in the Store** — Partner Center
   rejects a package that does not increase. The first segment can never be `0`. Commit them
   together as `chore(release): X.Y.Z`, with the release body in `docs/release-X.Y.Z.md`
   ([`release-notes.md`](release-notes.md) has the register). The body has to be **in the
   tagged commit**: the workflow publishes it from there, and refuses a plain tag without it.
2. **Tag and push after manual testing**: `git tag vX.Y.Z` then `git push origin vX.Y.Z`,
   with the tested version; never reuse a published tag. The
   [`Release installers`](../.github/workflows/release.yml) workflow first checks that the tag
   is `v` followed by the version in all three manifests, before anything is built. It then
   creates the GitHub release as a **draft** and attaches the NSIS and MSI installers, both
   architectures' `.msix` and one multi-architecture `Live.Translation_<version>.msixbundle`.
   Only once the bundle is attached does it publish the release, titled "Live Translation &
   Subtitles X.Y.Z", with `docs/release-X.Y.Z.md` as the body. Download the bundle from there.
3. **If a job fails, nothing is public.** The release stays a draft; re-run the failed jobs,
   and the `bundle` job publishes it when it passes. If the version check fails, nothing was
   built and no release exists for the tag, so it can be deleted (`git push origin --delete
   vX.Y.Z` and `git tag -d vX.Y.Z`) and the fixed commit tagged again. A hyphenated tag
   (`vX.Y.Z-rc.1`) builds the same assets into a draft prerelease, which is never published
   automatically.
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
| 1.7.1 | [20 October 2026](https://github.com/fmadore/Live-translation/releases/tag/v1.7.1) · [body](release-1.7.1.md) | Live as 1.7.1.0, confirmed by the maintainer on 21 October 2026; submission and certification dates not recorded | 1.7.0's app code, with the icon shown on the taskbar and in Start without an accent-coloured plate (#152). |
| 1.7.0 | [20 October 2026](https://github.com/fmadore/Live-translation/releases/tag/v1.7.0) · [body](release-1.7.0.md) | Not submitted; superseded by 1.7.1 | Offline Whisper translation into English and overlay placement per display layout (#121), overlay recovery when a display goes (#139), cloud sessions that reconnect (#116, #122). |
| 1.6.1 | [6 October 2026](https://github.com/fmadore/Live-translation/releases/tag/v1.6.1) · [body](release-1.6.1.md) | Live as 1.6.1.0, confirmed by the maintainer on 20 October 2026; submission and certification dates not recorded; replaced by 1.7.1 | The wave-to-words icon (#111) and the 5 October design review (#112); no new features. |
| 1.6.0 | [4 October 2026](https://github.com/fmadore/Live-translation/releases/tag/v1.6.0) · [body](release-1.6.0.md) | Live as 1.6.0.0, confirmed 6 October 2026; replaced by 1.6.1 | Local Whisper (#99), two translation targets, original speech in overlay and exports, Pause/Resume, editable filler words. |
| 1.5.1 | [23 September 2026](https://github.com/fmadore/Live-translation/releases/tag/v1.5.1) · [body](release-1.5.1.md) | Live as 1.5.1.0, confirmed 23 September 2026 | Batches 1–2 of the [22 September app review](app-review-2026-09-22.md) (#86): cleaner downsampled audio, release builds ignore `.env` and host overrides, a failing capture or provider thread ends only its source. |
| 1.5.0 | [22 September 2026](https://github.com/fmadore/Live-translation/releases/tag/v1.5.0) · [body](release-1.5.0.md) | Not submitted | Searchable provider-scoped caption languages, favourites, F2 favourite swapping and RTL overlay direction (#78). |
| 1.4.2 | [21 September 2026](https://github.com/fmadore/Live-translation/releases/tag/v1.4.2) | Not submitted | Start and Stop in a persistent action bar below the header. |
| 1.4.1 | [20 September 2026](https://github.com/fmadore/Live-translation/releases/tag/v1.4.1) | Not submitted | The [18-point operator UI audit](ui-audit-implementation.md): tabbed Settings, profiles above setup, a session browser, localized tray. |
| 1.4.0 | [19 September 2026](https://github.com/fmadore/Live-translation/releases/tag/v1.4.0) | Not submitted | Meeting profiles, 2–30 second caption persistence, appearance presets, history titles and search, operator shortcuts. |
| 1.3.0 | [19 September 2026](https://github.com/fmadore/Live-translation/releases/tag/v1.3.0) | Not submitted | Transcript history (#81), Stable reading (#79), overlay filler cleanup (#80) and the German interface. |
| 1.2.4 | [14 September 2026](https://github.com/fmadore/Live-translation/releases/tag/v1.2.4) | Live; acceptance date not recorded | Responsive captions: Fit window and optional Compact layout (#77). |
| 1.2.3 | [8 September 2026](https://github.com/fmadore/Live-translation/releases/tag/v1.2.3) | No submission recorded | Native Save As with timed SRT/VTT export (#26); capture of a selected application (#27). |

### Release 1.7.1 handoff

**Live as 1.7.1.0**, confirmed by the maintainer on 21 October 2026; the submission and
certification dates were not recorded.
- **What it shipped:** it replaced 1.6.1 in the Store, bringing 1.7.0's app code (1.7.0 itself
  was not submitted) and the unplated taskbar icon
  ([#152](https://github.com/fmadore/Live-translation/pull/152)).
- **Tags:** `v1.7.1` (annotated) and `v1.7.1-rc.1` both point to `2b52552`, the merge of #152.
- **Build:** release-commit CI and all four installer/MSIX/bundle jobs passed.
- **Package checks:** all 50 passed on the published assets. They are the 40 checks of 1.7.0
  at 1.7.1.0. In each package, the 46 `Assets/` files are byte-identical to the repository,
  and a single `resources.pri` (9,296 bytes) indexes the 14 unplated sizes.
- **Bundle:** 11,838,755 bytes, SHA-256
  `286ec7b988192e9df680477d6df33d3636c812cbb520fd4c9ea09cdf9b5f2ac9`.
- **Checked:** the taskbar icon, on ARM64 with a dark taskbar. The EN/FR/DE screenshots were
  recaptured from the 1.7.1 ARM64 MSIX on 21 October 2026
  ([#154](https://github.com/fmadore/Live-translation/pull/154)).
- **Not recorded:** the icon in Start, in Alt+Tab and on a light taskbar, and the 1.7.0 manual
  checks. They carry over to [1.8.0](#release-180-handoff).

### Release 1.7.0 handoff

**Published on GitHub on 20 October 2026 and not submitted;** 1.7.1 took its app code to the
Store.
- **Tags:** `v1.7.0` (annotated) points to `5ad2a0f`, the `chore(release): 1.7.0` commit.
  `v1.7.0-rc.1` points to `bffcd31`.
- **Build:** release-commit CI and all four installer/MSIX/bundle jobs passed.
- **Package checks:** all 40 passed. The bundle holds exactly both packages, byte for byte,
  with no signature. Manifests report 1.7.0.0 with the assigned identity, PE machine types are
  native, the package declares only `runFullTrust` and `microphone`, and it carries the
  rehearsal WAVs and licence notices but no weights.
- **Bundle:** 11,424,375 bytes, SHA-256
  `550765859b61c447b65f040b6fec54a47f457019b27194839489312ce5fd042b`.
- **Maintainer test:** the Local Test build of `f6a2448` on ARM64, on 20 October 2026,
  including system audio on wasapi 0.26. No x64 run is recorded. The other manual checks were
  not itemised and carry over to [1.8.0](#release-180-handoff).

### Release 1.6.1 handoff

Live as 1.6.1.0, confirmed by the maintainer on 20 October 2026: a Store-signed
`49346FMadore.LiveTranslationSubtitles_1.6.1.0_arm64` (`SignatureKind` `Store`) was installed on
the maintainer's PC on 6 October 2026. The submission and certification dates were not
recorded. It replaced 1.6.0 in the Store; 1.7.0 carries its open checks. Tag `v1.6.1` points to
`d206091`. Release-commit CI and all four installer/MSIX/bundle jobs passed, and all 37 package
checks passed on the published assets: GitHub digests, a bundle holding exactly both packages
byte for byte, 1.6.1.0 manifests with the assigned identity, native PE machine types, only
`runFullTrust` and `microphone`, rehearsal WAVs and licence notices, no weights or signature,
and the four new tile images. The bundle was 11,195,249 bytes, SHA-256
`86ef814f0e53baae6ff821db8cc263efa745982a7dfea945d8fda7d69a6ccea0`. The icon check on both
architectures, the smoke test of the new styles, screenshots and Windows App Certification Kit
were not recorded as done before submission.

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
