# Microsoft Store updates

Every update to this app is submitted **by hand** through Partner Center. That is not a
preference and not a stopgap — it is the only route this account has, for a reason Microsoft
enforces and we cannot work around. [Why by hand](#why-by-hand) has the detail; the short
version is that the submission API is a Company-account feature and this is an Individual
account.

The route below is what shipped every version so far. It takes about five minutes once the
packages are built.

## Release 1.10.0 handoff

**Prepared; not yet tagged.** The Store serves **1.9.0.0**, confirmed with `Get-AppxPackage`
(`SignatureKind` `Store`) on 25 October 2026. Target app version: **1.10.0**, MSIX
**1.10.0.0**. What's new and the certification notes cover the changes since 1.9.0.

1.10.0 is a reliability release with no new feature:

- [#171](https://github.com/fmadore/Live-translation/pull/171): batches 4–6 of the
  [22 October review](app-review-2026-10-22.md). A Gemini `goAway` handover drains the old
  socket and replays the audio captured meanwhile (B2); a transient failure during setup
  retries, five in a row still ending the source, through one error classifier for all three
  providers (B1, S1). System loopback uses a 200 ms buffer, drains every packet per wake and
  zero-fills SILENT packets (B5). The overlay reports ready, says when Alt+F4 hid it, restores a
  usable two-display placement as made and gets coalesced pushes (U5, B3, B10, E2). In the
  operator window, Start stays blocked until the run reports (U7), the transcript is mounted
  once (S5), recovery is offered in Settings (U14), a failing spool retries and says so (U3),
  and captions carry `lang` and `dir` (U12).
- [#177](https://github.com/fmadore/Live-translation/pull/177): the application list, which
  opens a `PROCESS_QUERY_LIMITED_INFORMATION | SYNCHRONIZE` handle on every process with a
  visible window, is built only in One application mode, so whole-output capture opens no
  handle a kernel anti-cheat could see. The README and Store copy gained the game voice chat
  sentence, feature line and keyword.
- Dependabot: `keyring-core` 1.0.1 ([#174](https://github.com/fmadore/Live-translation/pull/174)),
  two dev tools ([#173](https://github.com/fmadore/Live-translation/pull/173)) and
  `taiki-e/install-action` ([#172](https://github.com/fmadore/Live-translation/pull/172)).

**Changed under the surface:**
- **The overlay's capability shrinks (B4).** It keeps only `core:event:allow-listen` and
  `allow-unlisten` plus overlay-only app commands (`report_overlay_state`,
  `start_overlay_drag`, nudge, snap, undo); `command_names.rs` and an e2e check pin it. Overlay
  dragging moved from `data-tauri-drag-region` to `start_overlay_drag`, and Snap to bottom and
  Escape's undo into Rust, so **move mode needs a hand check on the installed package**.
- The package still declares only `runFullTrust` and `microphone`; the operator window's
  permissions are unchanged.

The [privacy policy](privacy.md) is unchanged. [GitHub release copy](release-1.10.0.md) ·
[EN/FR/DE Store fields](store-listing.md) ·
[Partner Center walkthrough](partner-center-walkthrough.md).

### Preparation and validation

- [x] #171 and #177 merged, each after all its CI jobs passed; then #172–#174.
- [x] App manifests, root lockfile entries and citation set to 1.10.0; `npm test` checks that
  all six agree. The citation date, 26 October 2026, is the planned tag date; correct it in
  the release PR if the tag moves to another day.
- [x] Lockfiles: nothing changes besides the two root version lines (`git diff --text`).
- [x] Validation of the release branch on Windows 11 ARM64:
  - `npm run test:coverage`: 818 passed, 1 skipped, in 82 files.
  - `check` (0 errors, 0 warnings, `--fail-on-warnings`), `lint`, `knip`, `format:check`,
    `check:languages` and `build` pass; `test:style`: 143 passed.
  - `cargo fmt --check`, Clippy with `-D warnings` and `cargo test --locked` (312 passed,
    5 ignored) on the pinned 1.99.1 toolchain. `test:e2e` was not rerun: nothing it drives
    changed since [#171](https://github.com/fmadore/Live-translation/pull/171), where it passed.
- [x] The Store fields are within Partner Center's limits (What's new: EN 1,021, FR 1,256,
  DE 1,231 of 1,500 characters); the certification notes name 1.10.0.0.

### Testing

- [ ] Merge the release PR.
- [ ] **Maintainer test** of the side-by-side Local Test build of the merge commit (ARM64),
  **before the tag** (review C18):
  - **system audio with local Whisper running** under load: no gaps, no runaway backlog (B5);
  - **overlay move mode**: drag, edge resize, nudge, Snap to bottom on mixed DPI, Escape's
    undo, and Escape with a display removed (B4);
  - **Alt+F4** on the overlay, then the toggle and the tray say it is hidden and bring it
    back (B3);
  - a strip placed across two displays reopens as placed (B10);
  - **a Gemini session of 20 minutes or more**, across at least one handover: no lost words
    at the switch (B1, B2);
  - Start, then a fast second click or the shortcut: one session, "Starting…" shown (U7);
  - System audio, then One application: the application list appears only once One
    application is chosen (#177);
  - Narrator on U3's note, U7's "Starting…" and an Arabic or Hebrew caption (U12); N1 in
    French at 225%.
- [ ] **No release candidate.** Tag `v1.10.0` straight from the merge once the test passes. A
  bug found afterwards ships as 1.10.1; never move a tag.

Still open from earlier handoffs, never recorded as done, on x64 and ARM64 unless a line says
otherwise:

- [ ] OpenAI's and Mistral's `session_expired` in a live session; a WebView2 reload of the
  overlay (U5).
- [ ] From 1.9.0: Light, Dark and System while switching Windows' app mode; a contrast theme;
  a Mistral rehearsal; Narrator focus on Start and Stop.
- [ ] Mistral on real French and English speech: the glossary, line breaks at sentence and turn
  ends, Pause/Resume/Stop and a brief network drop.
- [ ] The Transcript tab in a long session, and Narrator on the Live and Transcript tabs.
- [ ] A short live session with Gemini Translate, Gemini Transcribe and OpenAI.
- [ ] System audio on x64; a 24-bit microphone; Whisper Small translating real speech.
- [ ] Overlay restore with a laptop at 150% and a projector at 100%; an x64 PC without AVX2.
- [ ] A model download behind a proxy; quitting with a large Whisper backlog.
- [ ] The error sentences in English, French and German; the icon in Start, Alt+Tab, the tray
  and on a light taskbar.

### Release and Store acceptance

- [ ] Tag `v1.10.0` (annotated: "Live Translation & Subtitles v1.10.0") on the merge of the
  release PR, and push it.
- [ ] The workflow publishes the release with `docs/release-1.10.0.md` once the bundle is
  attached; if a job fails, the release stays a draft and the failed job is re-run.
- [ ] Verify `Live.Translation_1.10.0.msixbundle` with the [1.8.0 checks](#release-180-handoff)
  at **1.10.0.0**, and record the sizes and SHA-256 digests here.
- [ ] Sideload-test the per-architecture MSIX packages and walk the
  [accessibility release checklist](accessibility.md#release-checklist-manual-on-windows).
- [ ] **Screenshots:** nothing they show changed in 1.10.0. Retake them only if the 1.9.0
  retake (Settings › App › Theme on **Dark**) is still outstanding; see
  [store-screenshots](store-screenshots/README.md).
- [ ] Run Windows App Certification Kit.
- [ ] Submit the single combined bundle in Partner Center with the 1.10.0 What's new and the
  updated certification notes; paste the descriptions, features and keywords too unless #177's
  changes are already in the live listing. Use a gradual rollout (route step 9): audio capture
  and the session lifecycle changed.
- [ ] Record submission and certification, and confirm with
  `Get-AppxPackage 49346FMadore.LiveTranslationSubtitles` (`SignatureKind` `Store`, Version
  `1.10.0.0`).

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
| 1.9.0 | [22 October 2026](https://github.com/fmadore/Live-translation/releases/tag/v1.9.0) · [body](release-1.9.0.md) | Live as 1.9.0.0, confirmed with `Get-AppxPackage` on 25 October 2026; submission and certification dates not recorded | A light theme that follows Windows' app mode (#159) and batches 1–3 of the 22 October review (#160, #162). |
| 1.8.0 | [21 October 2026](https://github.com/fmadore/Live-translation/releases/tag/v1.8.0) · [body](release-1.8.0.md) | Live as 1.8.0.0, reported by the maintainer on 21 October 2026 and confirmed with `Get-AppxPackage`; submission and certification dates not recorded; replaced by 1.9.0 | The Transcript tab with Copy (#155) and Mistral subtitles on Voxtral Realtime 3 (#156). |
| 1.7.1 | [20 October 2026](https://github.com/fmadore/Live-translation/releases/tag/v1.7.1) · [body](release-1.7.1.md) | Live as 1.7.1.0, confirmed by the maintainer on 21 October 2026; submission and certification dates not recorded; replaced by 1.8.0 | 1.7.0's app code, with the icon shown on the taskbar and in Start without an accent-coloured plate (#152). |
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

### Release 1.9.0 handoff

**Live as 1.9.0.0**: `Get-AppxPackage` reports `SignatureKind` `Store` on 25 October 2026;
the submission and certification dates were not recorded. It replaced 1.8.0.
- **What it shipped:** a light theme, System by default, following Windows' app mode
  ([#159](https://github.com/fmadore/Live-translation/pull/159)); batches 1–3 of the
  [22 October review](app-review-2026-10-22.md)
  ([#160](https://github.com/fmadore/Live-translation/pull/160),
  [#162](https://github.com/fmadore/Live-translation/pull/162)); Tauri 2.12.4
  ([#169](https://github.com/fmadore/Live-translation/pull/169)). The operator window gained
  `core:window:allow-set-theme`; the CSP gained `object-src`, `base-uri`, `form-action` and
  `frame-src 'none'`.
- **Tag:** `v1.9.0` (annotated) points to `756f874`, the merge of
  [#170](https://github.com/fmadore/Live-translation/pull/170); no candidate.
- **Build:** the first run of draft-first publishing: all
  [installer/MSIX/bundle jobs](https://github.com/fmadore/Live-translation/actions/runs/39801417265)
  passed and the release published itself with its body about 20 minutes after the tag push.
- **Package checks:** all 30 passed on the published assets on 23 October 2026 (the
  [1.8.0 checks](#release-180-handoff) at 1.9.0.0).
  - Bundle: 11,866,825 bytes, SHA-256
    `e738b2706795e7777e7d4369566e48d9b57234113956e3109d28f0630cd6b8c2`.
  - ARM64 MSIX: 5,811,541 bytes, SHA-256
    `35e74cdeba60346ce647842ae712e096c5b7cb813fe1cbdea391c4ab4088e000`.
  - x64 MSIX: 6,053,276 bytes, SHA-256
    `87b5bda1f7a5106a3502317c1cf25c345947a9b0cc81305c06d671647763b43f`.
- **Not recorded:** the maintainer's test of the Local Test build, the sideload test, WACK,
  the screenshot retake and the manual checks. They carry over to
  [1.10.0](#release-1100-handoff).

### Release 1.8.0 handoff

**Live as 1.8.0.0**, reported by the maintainer on 21 October 2026; `Get-AppxPackage` reports
`SignatureKind` `Store`. It replaced 1.7.1.
- **What it shipped:** the Live and Transcript tabs, with Copy
  ([#155](https://github.com/fmadore/Live-translation/pull/155)), and Mistral subtitles on
  Voxtral Mini Transcribe Realtime 3 (`voxtral-mini-transcribe-realtime-3-0`, $0.48 an hour,
  45 languages), with lines closed at the end of turn and a names-and-terms list sent as
  `context_bias` ([#156](https://github.com/fmadore/Live-translation/pull/156)).
- **Known risk:** Mistral lists Realtime 3 as a public preview, and release builds pin `-3-0`.
  If Mistral withdraws it, Mistral subtitles stop with Mistral's error until an update ships.
- **Tags:** `v1.8.0` (annotated) points to `502ba34`, the merge of
  [#157](https://github.com/fmadore/Live-translation/pull/157). A `v1.8.0-rc.1` tag was deleted
  before its build made a draft.
- **Build:** the [release-commit CI](https://github.com/fmadore/Live-translation/actions/runs/39581397954)
  and all four [installer/MSIX/bundle jobs](https://github.com/fmadore/Live-translation/actions/runs/39581877713)
  passed.
- **Package checks:** all 36 passed on the published assets:
  - GitHub's digests match;
  - the bundle holds exactly both packages, byte for byte, with no signature;
  - the assigned identity and publisher, at 1.8.0.0;
  - native PE machine types;
  - only `runFullTrust` and `microphone`;
  - the rehearsal WAVs and licence notices, and no weights;
  - the 46 `Assets/` files byte-identical to the repository, with a single `resources.pri`.
- **Bundle:** 11,855,153 bytes, SHA-256
  `a8eaa3bd42e21cc10f344cf99b950adbaf914256994d2da6b4e4740102591121`.
- **Not recorded:** the maintainer's test of the Local Test build, the sideload test, WACK,
  the screenshot retake and the manual checks. They carried over to [1.9.0](#release-190-handoff).

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
