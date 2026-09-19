# Microsoft Store updates

Every update to this app is submitted **by hand** through Partner Center. That is not a
preference and not a stopgap — it is the only route this account has, for a reason Microsoft
enforces and we cannot work around. [Why by hand](#why-by-hand) has the detail; the short
version is that the submission API is a Company-account feature and this is an Individual
account.

The route below is what shipped every version so far. It takes about five minutes once the
packages are built.

## Release 1.3.0 handoff

Release target: **1.3.0**, MSIX **1.3.0.0**, prepared 19 September 2026.
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
- [ ] Publish v1.3.0 and verify CI plus the release installer workflow.
- [ ] Download and inspect the final x64 + ARM64 bundle and record its SHA-256.
- [ ] Smoke-test the final packaged app: two saved demo sessions, restart/reopen, copy/export/delete, opt-out, interrupted save, Stable reading and cleanup settings.
- [ ] Complete native keyboard, scaling, mixed-DPI, contrast, tray and existing audio/export regression checks.
- [ ] Refresh EN/FR screenshots and add DE screenshots from the final MSIX; committed PNGs remain historical 1.2.2 captures.
- [ ] Obtain native German review of the interface and listing; German speech recognition remains unverified and is not newly advertised.
- [ ] Run Windows App Certification Kit against final packages.
- [ ] Verify the public privacy-policy link shows the 19 September 2026 history-retention policy.
- [ ] Upload **Live.Translation_1.3.0.msixbundle** and the prepared listing fields in Partner Center; submit and verify certification.

History and cleanup are off by default. History stores raw caption text and session metadata
locally until explicitly deleted; disabling it does not delete existing sessions. No audio is
saved and no additional provider requests are introduced. Keep these facts in the Store copy.
The default built-in demonstration still requires no key, account, microphone or network.

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
history holds the changes and the working tree always holds what is on the listing right now.
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
