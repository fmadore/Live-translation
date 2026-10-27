# Microsoft Store updates

Every update to this app is submitted **by hand** through Partner Center. That is not a
preference and not a stopgap — it is the only route this account has, for a reason Microsoft
enforces and we cannot work around. [Why by hand](#why-by-hand) has the detail; the short
version is that the submission API is a Company-account feature and this is an Individual
account.

The route below is what shipped every version so far. It takes about five minutes once the
packages are built.

## Release 1.12.0 handoff

**Prepared; not yet tagged or submitted.** The Store serves **1.11.0.0**, accepted as reported
by the maintainer on 26 October 2026. Target app version: **1.12.0**, MSIX **1.12.0.0**. What's
new and the certification notes cover the changes since 1.11.0. Awaiting the maintainer's test
of the Local Test build.

1.12.0 adds Captions in Zoom and two sets of fixes:

- [#189](https://github.com/fmadore/Live-translation/pull/189) (the first item of the
  [#187](https://github.com/fmadore/Live-translation/issues/187) triage): **Captions in Zoom**.
  The operator pastes the caption link (API token) a meeting's host copies with *Show Captions ›
  Set up manual captioner › Copy the API token*, and the app posts one caption language's
  finished text directly to that Zoom URL, so attendees see it in Zoom's own caption bar. Off
  until started; outlives Stop and Start; ends with *Stop sharing* or when the app quits. States:
  sharing, waiting (HTTP 400, the meeting has not started), refused (401, 403 or 404),
  unreachable. *Send test caption* posts one fixed line. The link is held in memory only and
  never logged. [Guide](zoom-captions.md).
- On this branch (`afe95b9`): the toolbar button reads *Zoom captions* (*Sous-titres Zoom*,
  *Zoom-Untertitel*) with a generic captions mark, not Zoom's logo, which needs Zoom's written
  licence; Zoom is named only descriptively.
- [#188](https://github.com/fmadore/Live-translation/pull/188): no flashes or layout shifts in
  setup and the transcript (*Jump to latest* and the copy confirmation float, no key-form flash
  between Gemini and OpenAI, *Refresh devices* busy only when asked, a steady Whisper backlog
  strip, no stored device labelled missing before the first list, the second caption language
  in the searchable picker).
- On this branch (`c107a4c`): with system capture set to one application and none chosen, or the
  chosen one closed, the pre-flight's audio row asks for the application instead of reading as
  ready while Start and Test audio are disabled.
- No Dependabot updates since 1.11.0. `8fc6960` makes the style tests reload once when a bundle
  chunk is refused (test-only).

**Changed under the surface:**
- **Five new operator commands** (`src-tauri/src/zoom.rs`): `start_zoom_captions`,
  `stop_zoom_captions`, `send_zoom_test_caption`, `zoom_captions_status` and `open_zoom_guide`,
  pinned to the operator window in `command_names.rs` and `capabilities/operator.json`; the
  overlay's capability is unchanged. `open_zoom_guide` takes no argument and opens the fixed
  guide URL through `support::open`, like `open_support_page`. The link must be `https` on
  `zoom.us`, a `*.zoom.us` host or a `*.zoomgov.com` host, and is checked before anything is sent.
- Every engine's captions reach the share in `Events::caption`, and `SessionManager::start` tells
  it a session has begun; the share only queues, and sends on its own task. No crate was added
  (`reqwest` was already a dependency) and `Cargo.toml` is unchanged.
- The package still declares only `runFullTrust` and `microphone`; `AppxManifest.xml.template`
  is unchanged since `v1.11.0`.
- **New for the Store:** the app can now send user content (caption text, including Whisper's)
  to a third party the user names, Zoom. The [privacy policy](privacy.md) covers it in
  *Optional captions in Zoom*; the listing names Zoom descriptively with a trademark line; the
  certification notes explain how to exercise or skip the feature. See
  [store-listing-next.md](store-listing-next.md).

The [privacy policy](privacy.md) gains a pointer to the Zoom section, a paragraph on personal
information in shared captions, and a new effective date, 27 October 2026.
[GitHub release copy](release-1.12.0.md) · [EN/FR/DE Store fields](store-listing.md) ·
[Partner Center walkthrough](partner-center-walkthrough.md).

### Preparation and validation

- [x] #188 and #189 merged.
- [ ] `feat/zoom-captions-label` (`afe95b9`, `c107a4c` and these docs) reviewed and merged after
  all its CI jobs pass.
- [ ] **Version bump** in the release PR: app manifests, root lockfile entries and
  `CITATION.cff` (`version`, `date-released`) set to 1.12.0; `npm test` checks that all six
  agree. Commit as `chore(release): 1.12.0` with `docs/release-1.12.0.md`, and replace "Not yet
  tagged" in [release-notes.md](release-notes.md) with the tag date.
- [ ] Lockfiles: nothing changes besides the two root version lines (`git diff --text`).
- [ ] Validation of the release branch: `npm run test:coverage`, `check`, `lint`, `knip`,
  `format:check`, `check:languages`, `build` and `test:style`; `cargo fmt --check`, Clippy with
  `-D warnings` and `cargo test --locked`. Record the counts here.
- [ ] The Store fields are within Partner Center's limits. Counted when drafted, not yet checked
  in Partner Center: What's new EN 702, FR 900, DE 838 of 1,500 characters; 20 of 20 features,
  the longest 135 of 200; descriptions EN 4,694, FR 5,642, DE 5,264 of 10,000.
- [ ] Update the version-bound guides in the release PR, as 1.11.0's did:
  [partner-center-walkthrough.md](partner-center-walkthrough.md) (1.12.0 target, no new product
  declaration, the published privacy page showing 27 October 2026 and the Zoom section),
  [microsoft-store.md](microsoft-store.md), [packaging-msix.md](packaging-msix.md) and the
  "current for 1.11.0" lines in other guides; README, ROADMAP and docs/README once tagged.

### Testing

- [ ] **Maintainer test** of the side-by-side Local Test build of the release merge (ARM64),
  **before the tag**:
  - **Captions in Zoom in a real meeting**: the [live checks](zoom-captions.md#live-verification)
    the guide sets as its condition for shipping; at least host setup, the start check against
    a meeting not yet started, one started and an expired link, *Send test caption*, captions
    arriving from Whisper and from a cloud engine, Stop and Start while sharing, and *Stop
    sharing*, with the Zoom client version, account type and date recorded in the guide;
  - the *Step-by-step setup guide* link opens the guide in the default browser;
  - the button's label and states, and the dialog, in English, French and German, with Narrator;
  - the pre-flight asking for an application with One application selected and none chosen,
    and after the chosen application closes;
  - the setup and transcript no longer jumping: *Jump to latest*, Copy's confirmation,
    switching Gemini and OpenAI, *Refresh devices*, a Whisper backlog at its threshold.
- [ ] **No release candidate.** Tag `v1.12.0` straight from the merge once the test passes. A
  bug found afterwards ships as 1.12.1; never move a tag.

Still open from the [1.11.0 handoff](#release-1110-handoff) and earlier, never recorded as done:
its list carries over unchanged.

### Release and Store acceptance

- [ ] Tag `v1.12.0` (annotated: "Live Translation & Subtitles v1.12.0") on the merge of the
  release PR, and push it.
- [ ] The workflow publishes the release with `docs/release-1.12.0.md` once the bundle is
  attached; if a job fails, the release stays a draft and the failed job is re-run.
- [ ] Verify `Live.Translation_1.12.0.msixbundle` with the [1.8.0 checks](#release-180-handoff)
  at **1.12.0.0**, and record the sizes and SHA-256 digests here.
- [ ] Sideload-test the per-architecture MSIX packages and walk the
  [accessibility release checklist](accessibility.md#release-checklist-manual-on-windows).
- [ ] **Screenshots:** the toolbar gains a *Zoom captions* button, which the 1.7.1 images do
  not show. Decide whether to retake; if so, keep Zoom's windows and artwork out of them. See
  [store-screenshots](store-screenshots/README.md).
- [ ] Run Windows App Certification Kit.
- [ ] Check that the published [privacy policy](https://fmadore.github.io/Live-translation/privacy)
  shows the 27 October 2026 date and the *Optional captions in Zoom* section.
- [ ] Submit the single combined bundle in Partner Center with the 1.12.0 What's new, the three
  descriptions (with the Zoom trademark line), the twentieth feature line and the updated
  certification notes. The product declarations are unchanged from 1.11.0. Rollout: capture is
  untouched, but every caption and each session start now pass through the Zoom share; decide
  between a full and a gradual rollout.
- [ ] Record submission and certification, and confirm with
  `Get-AppxPackage 49346FMadore.LiveTranslationSubtitles` (`SignatureKind` `Store`, Version
  `1.12.0.0`).

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
| 1.11.0 | [26 October 2026](https://github.com/fmadore/Live-translation/releases/tag/v1.11.0) · [body](release-1.11.0.md) | Accepted as 1.11.0.0, reported by the maintainer on 26 October 2026; submission and certification dates not recorded | An optional Buy Me a Coffee link that unlocks nothing, and no empty band at the bottom of the operator window (#182). |
| 1.10.0 | [26 October 2026](https://github.com/fmadore/Live-translation/releases/tag/v1.10.0) · [body](release-1.10.0.md) | Live as 1.10.0.0, reported by the maintainer on 26 October 2026 and confirmed with `Get-AppxPackage` the same day; submission and certification dates not recorded; replaced by 1.11.0 | Batches 4–6 of the 22 October review (#171), no process handles outside One application capture (#177) and move mode left on Alt+F4 (#179). |
| 1.9.0 | [22 October 2026](https://github.com/fmadore/Live-translation/releases/tag/v1.9.0) · [body](release-1.9.0.md) | Live as 1.9.0.0, confirmed with `Get-AppxPackage` on 25 October 2026; submission and certification dates not recorded; replaced by 1.10.0 | A light theme that follows Windows' app mode (#159) and batches 1–3 of the 22 October review (#160, #162). |
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

### Release 1.11.0 handoff

**Accepted as 1.11.0.0**, reported by the maintainer on 26 October 2026, the day it was
tagged; the submission and certification dates were not recorded. A `Get-AppxPackage` reading
of 1.11.0.0 with `SignatureKind` `Store` is still to be recorded: when the acceptance was
reported, the maintainer's PC still had the Store's 1.10.0.0. It replaced 1.10.0.
- **What it shipped:** an optional *Support development* link to
  `https://buymeacoffee.com/fmadore` in Settings › App and at the foot of the start screen,
  which unlocks nothing and is never shown during a session, opened by the new
  `open_support_page` operator command; `.sr-only` text pinned to its containing block, so the
  operator window no longer scrolls as a whole and leaves an empty band
  ([#182](https://github.com/fmadore/Live-translation/pull/182)). Dependabot's #181.
- **Tested:** the side-by-side Local Test build of `7eb9f3d` (ARM64), "it works", reported by
  the maintainer on 26 October 2026 before the tag; the support link, its three languages,
  the operator window that no longer scrolls and the three themes are recorded as passed
  together.
- **Tag:** `v1.11.0` (annotated) points to `bb22137`, the citation-date commit on top of the
  merge of [#185](https://github.com/fmadore/Live-translation/pull/185); no candidate.
- **Build:** all [installer/MSIX/bundle jobs](https://github.com/fmadore/Live-translation/actions/runs/40289558151)
  passed, and the release published itself with its body on 26 October 2026, about 18 minutes
  after the tag push.
- **Assets** (GitHub's digests):
  - Bundle: 11,950,514 bytes, SHA-256
    `346cf1212847e27a0844bf299c694c560f24fc860c8554463796f2fcd8ba5444`.
  - ARM64 MSIX: 5,856,615 bytes, SHA-256
    `8b9e7a5918cd37279dc8c339ff6032dc5081376e292d9a51b884ff07033caca8`.
  - x64 MSIX: 6,091,888 bytes, SHA-256
    `e8931317d9add9349d7c3eff628eea5f400d0d802c87fd113d7e2b2e3fb6d202`.
- **Not recorded:** the package checks on the published assets, the sideload test, WACK and
  the manual checks below. They carry over to the next release.

Still open from this and earlier handoffs, never recorded as done, on x64 and ARM64 unless a line says
otherwise:

- [ ] From 1.10.0: system audio with local Whisper under load; overlay move mode (drag, edge
  resize, nudge, Snap to bottom on mixed DPI, Escape's undo, and Escape with a display
  removed); Alt+F4 on the overlay, then the toggle and the tray; a strip across two displays;
  a Gemini session of 20 minutes or more across a handover; Start then a fast second click;
  the application list appearing only in One application; Narrator on the recovery note,
  "Starting…" and an Arabic or Hebrew caption.
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

### Release 1.10.0 handoff

**Live as 1.10.0.0**, reported by the maintainer on 26 October 2026, the day it was tagged;
the submission and certification dates were not recorded. That morning the maintainer's PC
still had the Store's 1.9.0.0; later that day `Get-AppxPackage` showed 1.10.0.0 with
`SignatureKind` `Store`. It replaced 1.9.0 and was replaced the same day by 1.11.0.
- **What it shipped:** batches 4–6 of the [22 October review](app-review-2026-10-22.md)
  ([#171](https://github.com/fmadore/Live-translation/pull/171)): a Gemini handover that drains
  the old connection, setup failures retried, system loopback that keeps up under load, an
  overlay with fewer permissions, and operator-window fixes. No process handles outside One
  application capture ([#177](https://github.com/fmadore/Live-translation/pull/177)), and move
  mode left when Alt+F4 hides the overlay
  ([#179](https://github.com/fmadore/Live-translation/pull/179)). Dependabot's #172–#174.
- **Tag:** `v1.10.0` (annotated) points to `cfb427b`, the merge of
  [#179](https://github.com/fmadore/Live-translation/pull/179); no candidate.
- **Build:** all [installer/MSIX/bundle jobs](https://github.com/fmadore/Live-translation/actions/runs/40274338725)
  passed, and the release published itself with its body on 26 October 2026, about 15 minutes
  after the tag push.
- **Assets** (GitHub's digests):
  - Bundle: 11,942,706 bytes, SHA-256
    `38ccdb02d35894834699779fb46d2017529badefea0f2c1949fc7d4b6eaad984`.
  - ARM64 MSIX: 5,848,300 bytes, SHA-256
    `52cfa87ef6a234d1d9d69d6f0e107e628bbd78050b49b9f8d21052f11e565d45`.
  - x64 MSIX: 6,092,395 bytes, SHA-256
    `edebc78f2a725fa90502be177a80e7257463bb05399ac0593285fd68821f07ca`.
- **Not recorded:** the package checks on the published assets, the maintainer's test of the
  Local Test build, the sideload test, WACK and the manual checks. They carry over to
  [1.11.0](#release-1110-handoff), which did not record them either.

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
  the screenshot retake and the manual checks. They carried over to
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
