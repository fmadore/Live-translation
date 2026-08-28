# Microsoft Store updates

Every update to this app is submitted **by hand** through Partner Center. That is not a
preference and not a stopgap — it is the only route this account has, for a reason Microsoft
enforces and we cannot work around. [Why by hand](#why-by-hand) has the detail; the short
version is that the submission API is a Company-account feature and this is an Individual
account.

The route below is what shipped every version so far. It takes about five minutes once the
packages are built.

## The route

1. **Bump the version** in `package.json`, `src-tauri/Cargo.toml` and
   `src-tauri/tauri.conf.json`. All three must match, and all three must be **higher than the
   version already in the Store** — Partner Center rejects a package that does not increase.
   The first segment can never be `0`. Commit them together as `chore(release): X.Y.Z` so the
   bump is one reviewable change rather than three scattered ones.
2. **Tag and push**: `git tag v1.2.0 && git push origin v1.2.0`. The
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
   [Screenshots](#screenshots). Both languages.
6. Partner Center → **Apps and games** → *Live Translation & Subtitles* → **Create new
   submission**. In French: **Applications et jeux**, **Créer une soumission**.
7. Under **Packages**, **delete every package already listed**, then upload the single
   `.msixbundle`. Deleting matters: 1.0.5 went up as two separate per-architecture packages,
   and leaving one behind ships a stale architecture silently. One bundle in, nothing else.
8. **Paste the listing text** for both languages if it changed. The paste-ready English and
   French copy is in [`store-listing.md`](store-listing.md), and **What's new in this version**
   has to be rewritten for every release. Editing that file changes nothing on its own; the
   Store only knows what is typed into Partner Center.
9. Set a **gradual rollout** percentage for anything touching audio capture or the session
   lifecycle. Start at 10% and raise it once the crash and review data look clean.
10. **Submit to the Store**, then watch certification. A clean submission is not a pass — 1.0.3
   uploaded perfectly and then failed policy 10.1.2.10.

[`partner-center-walkthrough.md`](partner-center-walkthrough.md) has the screen-by-screen
version of steps 6–10, including the French labels.

## Screenshots

The Store listing is per-language, so it needs its own screenshots in each language, and they
are kept in the repository:

```text
docs/store-screenshots/en/    English listing
docs/store-screenshots/fr/    French listing
```

Four per language, named for the order they are uploaded in — the order is what a visitor
scrolls through, so it is part of the listing rather than an implementation detail:

| File | What it shows |
| --- | --- |
| `1-idle.png` | The idle screen: Built-in demo, Demo audio, Free, and the Start demo button. |
| `2-running-english.png` | A running English demo: Demo status, a moving meter, elapsed time, and an overlay caption. |
| `3-running-french.png` | A running French demo, showing a French caption. |
| `4-provider.png` | Optional. A live provider's configuration, showing its key requirement and estimated cost. |

**One current set, overwritten in place.** They are re-captured rather than accumulated, so the
history holds the changes and the working tree always holds what is on the listing right now.
The alternative — a folder per version — grows the repository by a few megabytes every release
to preserve something nobody reads twice.

Captured on Windows at the display scaling the app is actually used at, from a **sideloaded
MSIX** rather than a dev build, for the same reason the accessibility walk uses one: package
identity changes how the window is drawn. Store screenshots must be at least 1366×768.

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
  well and then failed policy 10.1.2.10 — the reviewer pressed **Start Subtitles** and nothing
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
