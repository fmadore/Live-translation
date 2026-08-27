# Microsoft Store updates

> [!IMPORTANT]
> **This does not work on our account, and cannot be made to.** Verified 27 August 2026 while
> setting 1.1.0 up. Everything below is accurate and unreachable; keep it for the day the
> constraint lifts, and submit by hand until then — see [Submitting by hand](#submitting-by-hand).
>
> The submission API authenticates as a Microsoft Entra application, which has to live in a
> tenant associated with the Partner Center account. Tenant association, user management and
> Entra applications are **Company-account features**. Ours is an Individual account, and
> Microsoft is explicit that
> [Entra ID sign-up "is currently supported only for Company accounts"](https://learn.microsoft.com/en-us/windows/apps/publish/partner-center/open-a-developer-account?tabs=individual).
> In the dashboard this shows up as `account-settings/organization/tenant-management`
> rendering a blank page. Nothing is misconfigured; the feature is not there.
>
> Converting is not a way out. Partner Center does not support changing an account from
> Individual to Company — it needs a *new* account, which means a new publisher identity, which
> means a new Store listing at a new URL. The current identity
> (`49346FMadore.LiveTranslationSubtitles`) is baked into every package we have shipped, so
> switching would abandon the published listing and its installs. Not a trade worth making for
> this app.

Shipping an update by hand means building the packages, opening Partner Center, creating a
submission, replacing every package, and pressing Submit. The
[`Microsoft Store submission`](../.github/workflows/store-submission.yml) workflow does the
build-and-upload half from a GitHub release, and stops before the part that reaches users.

**What it does not do:** change the listing text, screenshots, pricing, age rating, or
availability. Those still go through Partner Center. This workflow only replaces the package.

## Submitting by hand

The route that works. About five minutes, and it is what shipped every version so far.

1. **Build the packages** — steps 1 and 2 of [The shape of an update](#the-shape-of-an-update)
   still apply: bump the three version files, tag, let `Release installers` build. Download
   `Live.Translation_<version>.msixbundle` from the release.
2. **Sideload-test first.** Install the bare per-architecture `.msix` with
   [`scripts/install-local-msix.ps1`](../scripts/install-local-msix.ps1), *not* the bundle:
   signing a bundle does not sign the packages inside it.
3. Partner Center → **Apps and games** → *Live Translation & Subtitles* → **Create new
   submission**. In French: **Applications et jeux**, **Créer une soumission**.
4. Under **Packages**, **delete every package already listed**, then upload the single
   `.msixbundle`. Deleting matters: 1.0.5 went up as two separate per-architecture packages,
   and leaving one behind ships a stale architecture silently. One bundle in, nothing else.
5. If the listing text changed, paste it in the same submission — the paste-ready English and
   French copy is in [`store-listing.md`](store-listing.md). Editing that file changes nothing
   on its own; the Store only knows what is typed into Partner Center.
6. Set a **gradual rollout** percentage for anything touching audio capture or the session
   lifecycle. Start at 10% and raise it once the crash and review data look clean.
7. **Submit to the Store**, then watch certification. A clean submission is not a pass — 1.0.3
   uploaded perfectly and then failed policy 10.1.2.10.

## The shape of an update

Steps 1–3 are the build, and are what actually gets used. Steps 4–5 are the automated tail
that this account cannot reach — do [Submitting by hand](#submitting-by-hand) instead.

1. Bump the version in `package.json`, `src-tauri/Cargo.toml` and `src-tauri/tauri.conf.json`.
   All three must match, and all three must be **higher than the version already in the
   Store** — Partner Center rejects a package that does not increase. The first segment can
   never be `0`.
2. Tag and push: `git tag v1.1.0 && git push origin v1.1.0`. The
   [`Release installers`](../.github/workflows/release.yml) workflow builds the NSIS installer,
   both architectures' `.msix`, and one multi-architecture `Live.Translation_<version>.msixbundle`,
   and attaches them to the GitHub release.
3. Sideload-test the build before submitting it. Signing a bundle does not sign the packages
   inside it, so install the bare per-architecture `.msix` with
   [`scripts/install-local-msix.ps1`](../scripts/install-local-msix.ps1), not the bundle.
4. *(Unavailable — Individual account.)* Actions → **Microsoft Store submission** → **Run
   workflow**. Give it the tag. Leave **commit** unticked.
5. Open Partner Center, read the draft submission, and press Submit.

Ticking **commit** collapses steps 4 and 5 into one run. Do that only for an update you have
already tested, because nothing after it is reversible without a new submission.

**rollout** takes a percentage. `10` releases the update to a tenth of installs and holds the
rest, which is worth using for anything touching audio capture or the session lifecycle; you
raise or halt it from Partner Center afterwards.

## Why the package is one bundle

The Store submission is a single multi-architecture `.msixbundle` containing both the x64 and
the ARM64 package, rather than two single-architecture bundles.

This is forced by how `msstore publish` updates a submission: it uploads one package and marks
**only the first existing package with a matching file extension** for deletion. Given two
`.msixbundle` packages in the submission, one new bundle would replace one of them and leave
the other in place — so the app would ship with a stale package for one architecture, silently.
One bundle in, one bundle out, no ambiguity.

> **The first automated submission needs a manual check.** Version 1.0.5 was uploaded to
> Partner Center by hand as separate per-architecture packages. The first run of this workflow
> cannot clean those up for the reason above. Leave **commit** unticked, then confirm in
> Partner Center that the submission contains the new bundle and nothing from 1.0.5 before
> submitting. Later updates replace cleanly.

## One-time setup

**Blocked on an Individual account — see the note at the top of this file.** Steps 1–3 need
Partner Center screens that only Company accounts have, so step 4 has no values to fill in.
Kept because it is correct, and because the constraint is Microsoft's rather than ours.

Everything here is done once, and all of it is on the account side — none of it is in this
repository.

### 1. Give the Partner Center account a Microsoft Entra tenant

Partner Center → **Account settings** → **Tenants**. Either
[associate an existing Entra tenant](https://learn.microsoft.com/en-us/windows/apps/publish/partner-center/associate-existing-azure-ad-tenant-with-partner-center-account)
or [create one](https://learn.microsoft.com/en-us/windows/apps/publish/partner-center/create-new-azure-ad-tenant).
The submission API authenticates against the tenant, not the Microsoft account used to sign in,
so there is no way around this step.

### 2. Register an application in the tenant

[Entra admin center](https://entra.microsoft.com/) → **Identity** → **Applications** →
**App registrations** → **New registration**. It needs no redirect URI and no API permissions;
it exists only to hold a client secret.

Then **Certificates & secrets** → **New client secret**. Copy the value immediately — it is
shown once. Note its expiry: when it lapses the workflow fails to authenticate, and the fix is
a new secret and an updated repository secret.

### 3. Give that application the Manager role in Partner Center

Partner Center → **Account settings** → **User management** → **Microsoft Entra applications**
→ add the application you registered, with the **Manager** role. Without this the credentials
authenticate but every submission call is refused.

### 4. Add four repository secrets

Repository → **Settings** → **Secrets and variables** → **Actions**:

| Secret | Where it comes from |
| --- | --- |
| `AZURE_AD_TENANT_ID` | Entra admin center → Identity → Overview → Tenant ID |
| `AZURE_AD_APPLICATION_CLIENT_ID` | The app registration's Application (client) ID |
| `AZURE_AD_APPLICATION_SECRET` | The client secret value from step 2 |
| `SELLER_ID` | Partner Center → Account settings → Identifiers → Seller ID |

The workflow checks all four before it touches anything and names the missing ones.

## Constraints worth knowing

- **Free products only.** Microsoft supports Store update operations through GitHub Actions for
  free products; paid products are not covered. Live Translation & Subtitles is free.
- **One pending submission per product.** A run started while a submission is already open in
  Partner Center will fail. Finish or delete the pending one first — `msstore submission delete
  9PFB8LR3RR9X` does the latter.
- **The CLI is in preview**, and the published documentation trails it. Learn documents a
  `--inputFile` option for `msstore publish`; the released CLI has no such option, and takes the
  package path as its positional argument instead. The workflow is written against the CLI, not
  the doc.
- **Certification can still fail.** A green workflow means the package was accepted for
  submission, not that it passed review. 1.0.3 was uploaded perfectly well and then failed
  policy 10.1.2.10.

## Automating listing text as well

`msstore submission updateMetadata <productId> <json>` can replace the listing metadata from a
JSON file committed to the repository, which would make the Store description reviewable in a
pull request. It is not wired up here: the listing changes rarely, and a bad metadata push is
more disruptive than a bad package push because it goes live on the product page. The
paste-ready listing text lives in [`store-listing.md`](store-listing.md).

## Sources

- [Publish app updates to Microsoft Store with GitHub Actions](https://learn.microsoft.com/en-us/windows/apps/publish/msstore-dev-cli/github-actions)
- [Microsoft Store Developer CLI commands](https://learn.microsoft.com/en-us/windows/apps/publish/msstore-dev-cli/commands)
- [msstore-cli source](https://github.com/microsoft/msstore-cli)
- [Package version numbering](https://learn.microsoft.com/en-us/windows/uwp/publish/package-version-numbering)
