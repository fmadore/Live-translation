# Microsoft Store distribution

Plan for shipping Live Translation through the Microsoft Store so that other people can
install it without the SmartScreen "Windows protected your PC / unknown publisher" wall that
today's unsigned NSIS installer produces.

Researched 19 August 2026. Every Store policy number below is from the
[Microsoft Store Policies](https://learn.microsoft.com/en-us/windows/apps/publish/store-policies);
re-check them before submitting, because they move.

## Verdict

**Yes — go to the Store, but as an MSIX package, and not as the event-day install path for
STIAS.**

The intuition that it "got attractive" is right, and for a bigger reason than usual:
Microsoft removed the developer registration fee in 2026 for
[individual](https://learn.microsoft.com/en-us/windows/apps/publish/whats-new-individual-developer)
and [company](https://learn.microsoft.com/en-us/windows/apps/publish/whats-new-company-developer)
accounts alike, so the entire cost of the recommended route is now zero. The part that
actually solves the warning is the signing model: an MSIX submitted to the Store **does not
need a code-signing certificate**, because Microsoft re-signs the package with its own
certificate during publishing. A Store install never takes the unknown-publisher path at all.

Three gates are specific to *this* app and are the reason this document exists rather than a
one-line "use MSIX":

1. Policy **10.8.3** classifies **"API secret keys"** as financial information, and products
   that require it for primary functionality **must be submitted from a company account**.
   Live Translation cannot caption a single word without a Gemini/OpenAI/Mistral key.
2. An MSIX package version's **first segment cannot be `0`**, so `0.5.3` cannot ship. The
   Store forces a **1.0.0** release.
3. Certification runs the app on a clean machine. With no key, the tester sees an app that
   does nothing — policy **10.3.1** wants a working demo credential in *Notes for
   certification*, and handing a live paid key to a stranger is not acceptable here.

None of the three is fatal. All three need a decision before any packaging work starts.

## Route comparison

| Route | Cost | Does it kill the warning? | Effort | Verdict |
| --- | --- | --- | --- | --- |
| **Store, MSIX package** | **free** | **Yes — Microsoft signs it; Store installs bypass SmartScreen entirely** | Medium: a new packaging pipeline | **Recommended** |
| Store, "EXE or MSI app" listing | free account, but you still buy a certificate | No — the Store only links to *your* installer, which must already be signed | Low | Pointless: it solves discovery, not trust |
| Azure Artifact Signing (ex-Trusted Signing) | $9.99/month | Partly — reputation-based, same as an OV certificate; no instant trust | Low | **Likely ineligible**: individual sign-up is US/Canada only, EU only as an organization |
| OV / EV certificate | ~$200–600/year (EV needs a hardware token) | EV yes, OV only after reputation accrues | Low | Expensive for a workshop tool |
| Status quo (unsigned NSIS) | free | No | none | Current state |

Note the trap in row two. Tauri's own
[Microsoft Store guide](https://v2.tauri.app/distribute/microsoft-store/) documents exactly
that route — reserve the name under "EXE or MSI app" and point the listing at a hosted
installer — because Tauri's bundler emits only MSI and NSIS. That route explicitly requires a
code-signed installer and therefore **does not** solve the problem we are trying to solve.
Getting the Microsoft signature means producing an MSIX, which is outside the Tauri bundler
and needs one of the tools below.

## Packaging toolchain

Tauri has no MSIX bundle target. Two tools close the gap; both wrap `makeappx`/`makepri` from
the Windows SDK.

- **[`@choochmeque/tauri-windows-bundle`](https://github.com/Choochmeque/tauri-windows-bundle)**
  — Tauri-aware. Reads `tauri.conf.json` for name/version/icons, generates the
  `Package.appxmanifest`, auto-adds the `runFullTrust` restricted capability that every Tauri
  app needs, and — the deciding feature — builds **x64 and ARM64 into one `.msixbundle`** with
  `--arch x64,arm64`. `release.yml` already builds both architectures, so this maps onto the
  existing matrix with no rework.
- **[`winapp` CLI](https://github.com/microsoft/winappCli)** — Microsoft's own, in public
  preview since January 2026, with an
  [official Tauri guide](https://github.com/microsoft/winappCli/blob/main/docs/guides/tauri.md).
  `winapp init` writes the manifest and assets, `winapp cert generate` makes a dev
  certificate, `winapp pack .\dist --cert .\devcert.pfx` produces the MSIX. First-party and
  the safer long-term bet, but the documented flow is single-architecture.

**Recommendation:** use `tauri-windows-bundle` for the shipped artifact, and keep `winapp`
installed for local work — `winapp run` grants package identity to an unpackaged build, which
is how you debug the packaged-only failures in Phase D without re-signing an MSIX each time.

## Gates and unknowns specific to this app

### 1. Account type — policy 10.8.3

> "If your product requires financial account information, you must submit that product from a
> company account type. Products from individual accounts cannot require financial
> information for primary functionality." Financial information "includes, but is not limited
> to … **API secret keys**, private keys, or recovery phrases."

Read literally, an individual-account submission of this app is rejectable. Options, best
first:

- **Submit from a company account** in the name of the institution that the workshop belongs
  to (ZMO / STIAS). Company registration is free as of May 2026 but requires Entra ID and an
  organisational verification, so it needs a real human on the institution's side and lead
  time. This is the safe route.
- **Submit from an individual account and argue the point** in Notes for certification: the
  key is a third-party service credential the user already holds, the app never transmits it
  anywhere except to the provider that issued it, and no payment happens in-app. This may
  well pass — plenty of bring-your-own-key apps are on the Store — but budget for one
  rejection round.

Decide this **first**. It gates the timeline more than any code.

### 2. Testability without a key — policy 10.3.1

Certification needs to see the app work. Two things fix this together:

- **Rehearsal mode** — already on the roadmap under "Future ideas". Play a bundled short FR/EN
  audio fixture through the pipeline. If it runs offline against a canned transcript, the
  certification tester can exercise capture, the overlay, move mode, and export with no key
  and no billing. This turns a nice-to-have into a submission enabler and is the single
  highest-value code item in this plan.
- **Notes for certification** text explaining the BYO-key model, linking the three provider
  signup pages, and pointing at rehearsal mode.

If rehearsal mode slips, the fallback is issuing a Mistral key (cheapest at $0.006/min) with a
low spend cap, used once and revoked.

### 3. Privacy policy — policy 10.5.1

> "Product types that inherently have access to Personal Information must always have privacy
> policies. These include, but are not limited to, Desktop Bridge and Win32 products."

This app captures a microphone and streams audio to third parties, so it is squarely in
scope, and Partner Center requires a **public URL**, not a repository file. The substance
already exists in [`SECURITY.md`](../SECURITY.md) under *How this app handles credentials* and
*What leaves the machine*; it needs restating as a standalone `docs/privacy.md` published via
GitHub Pages. It must name: what audio is captured and when, that it is transmitted to the
selected provider under that provider's terms, that keys live in Windows Credential Manager,
that transcripts are written only locally on explicit user action, and that the app collects
no telemetry.

### 4. Listing disclosure — policy 10.2.4

> "Your product may depend on non-integrated software … to deliver its primary functionality
> if you disclose the dependency **at the beginning of the description**."

So the Store description must open with the paid-API-key requirement — not bury it. Something
like: *"Requires your own API key from Google Gemini, OpenAI, or Mistral. These services bill
per minute of audio; see the pricing table in the description."* The cost table in the README
should be restated in the listing so nobody installs expecting a free tool.

### 5. Version number

The last segment of an MSIX version is reserved for the Store and must be `0`; the **first
segment cannot be `0`**. `0.5.3` is therefore not packageable. Ship the Store debut as
**1.0.0** (`1.0.0.0` in the manifest), bumping `package.json`, `src-tauri/Cargo.toml`, and
`src-tauri/tauri.conf.json` together. Every subsequent Store submission must strictly
increase.

### 6. Microphone under package identity — needs hardware verification

Unpackaged Win32 apps read the microphone with no per-app gate. A packaged app is different:
it declares `<DeviceCapability Name="microphone"/>` and is then subject to
**Settings → Privacy & security → Microphone**, per app. The realistic failure on event day is
an operator whose fresh install is toggled off, seeing a bare cpal error.

Two consequences:

- Add the capability to the manifest (`tauri-windows-bundle` exposes a `device` capability
  category for this).
- Teach `audio/capture.rs` to recognise a permission-denied device open and surface a
  `StatusUpdate` that names the exact Settings path, instead of the raw cpal string. The
  per-origin `StatusUpdate` plumbing from Phase 1 of the roadmap already carries this.

### 7. WASAPI loopback under package identity — needs hardware verification

**The biggest genuine unknown in this plan.** System-audio capture is the feature that makes
this app worth installing, and it is the one CI cannot check: a packaged full-trust app runs
at medium integrity outside an AppContainer, so `wasapi` loopback on the default render
endpoint *should* behave identically — but "should" is not a rehearsal. Verify on real
hardware, capturing a Zoom or Teams call, before anything is submitted. Do this early: if
loopback needs a manifest capability or breaks under identity, it invalidates the whole route
and you want to know in Phase C, not Phase E.

### 8. Credential Manager under package identity — verify

`keyring` v4 uses Windows Credential Manager, which is not virtualised for packaged apps, so
the `org.stias.live-translation` service entries should resolve unchanged — including keys
saved by an earlier unpackaged install. Confirm by saving a key unpackaged and reading it from
the MSIX build.

### 9. WebView2 — low risk, but assert it

Windows 11 ships the Evergreen Runtime and Windows 10 has received it through Edge for years,
so the practical risk is small. Declaring it as an MSIX package dependency is
[known to be unreliable](https://github.com/MicrosoftEdge/WebView2Feedback/issues/1165), so do
not: instead fail loudly and legibly at startup if the runtime is absent. Note that this
diverges from the NSIS build, which currently uses Tauri's default downloadBootstrapper.

### 10. Updates

The app ships no updater plugin today, and that is the right shape — the Store owns updates
for Store installs. If `tauri-plugin-updater` is ever added for the GitHub-release build, it
must be compiled out of the Store package; a self-updating Store app is a certification
failure.

## Timeline against the STIAS workshop

The workshop is **21–24 September 2026**; this was written on **19 August 2026**. That is
about five weeks, and the critical path is not code:

| Step | Realistic elapsed |
| --- | --- |
| Company-account verification (if taking route 1 of gate 1) | days to weeks, dependent on the institution |
| Identity verification (government ID + selfie) | hours to days |
| First-submission certification | up to ~3 business days, longer for a first-time publisher |
| One rejection round on 10.8.3 or 10.3.1 | add a week |

**Recommendation: do not make the Store the event-day install path.** Keep the signed-nothing
NSIS installer as the STIAS plan, and treat the Store as the durable answer for everyone who
comes to the project afterwards. A rejection on 10.8.3 the week before the workshop, with the
answer being "get a company account", is a bad place to be.

### Cheap mitigation for the event itself

Independent of the Store, and worth doing this week:

- A short **Installing on Windows** section in the README with the literal SmartScreen
  click-path (*More info → Run anyway*) and a screenshot, so a nervous attendee has something
  official-looking to follow.
- **Publish SHA-256 checksums** with each release so the download can be verified. `release.yml`
  does not emit them today.
- SECURITY.md already states installers are unsigned; link it from the README download section
  rather than leaving it to be discovered.

## Phased plan

### Phase A — decisions and paperwork (no code, start immediately)

- [ ] Decide the account type against gate 1 (company via the institution, or individual with
      an argued 10.8.3 case). Everything else waits on this.
- [ ] Enrol in Partner Center and complete identity verification.
- [ ] Reserve the app name. "Live Translation" is generic and probably taken; reserve
      **"Live Translation & Subtitles"** to match the README title.
- [ ] Note the assigned publisher identity (`CN=…`) and package identity name — the manifest
      must match them byte for byte or ingestion fails.

### Phase B — repository preparation

- [ ] `docs/privacy.md` + GitHub Pages, per gate 3.
- [ ] Version bump to **1.0.0** across `package.json`, `Cargo.toml`, `tauri.conf.json`.
- [ ] Drop macOS (see below) so the release matrix is Windows-only.
- [ ] README: Installing on Windows section; SHA-256 checksums in `release.yml`.

### Phase C — packaging pipeline

- [ ] `npx @choochmeque/tauri-windows-bundle@latest init`; commit `bundle.config.json` and the
      manifest template.
- [ ] Manifest: publisher identity from Phase A, `runFullTrust`, `microphone` device
      capability, `1.0.0.0`.
- [ ] Build locally, sign with a self-signed certificate, install, and **verify gates 6, 7 and
      8 on real hardware** — microphone, WASAPI loopback against a live Teams/Zoom call, and
      Credential Manager round-trip. Also re-check the transparent click-through overlay and
      the `Documents/Live-translation/` export path under package identity.
- [ ] Add an `msix` job to `release.yml` producing an unsigned `.msixbundle`
      (`--arch x64,arm64`) as a release asset. Keep the NSIS installer alongside it — the
      GitHub release stays the fallback channel.

### Phase D — app changes for packaged reality

- [ ] **Rehearsal mode** (gate 2) — bundled FR/EN fixture through the full pipeline.
- [ ] Permission-denied microphone path with an actionable message (gate 6).
- [ ] First-run state with no key configured that explains the BYO-key model in-app rather
      than looking broken.
- [ ] WebView2 presence assertion (gate 9).

### Phase E — listing and submission

- [ ] Screenshots (1366×768 minimum) — operator window, overlay over a slide, move mode.
- [ ] Description opening with the API-key dependency (gate 4) and carrying the cost table.
- [ ] IARC age rating questionnaire.
- [ ] Notes for certification: BYO-key explanation, rehearsal-mode walkthrough, 10.8.3
      argument if submitting from an individual account.
- [ ] Submit; expect one round of feedback.

### Phase F — after launch

- [ ] README install section leads with the Store badge, GitHub releases demoted to "advanced
      / offline".
- [ ] Consider a `winget` manifest pointing at the Store package — one more install path, no
      extra signing.
- [ ] Store submissions on tag, ideally via the Partner Center submission API from
      `release.yml`.

## Dropping macOS

Raised alongside this plan, and it fits: this route is Windows-only by construction, macOS
system-audio capture is
[gated on a $99/year Apple Developer membership](../ROADMAP.md) before any of its capture code
becomes useful, and the STIAS event laptop is Windows. Removing it deletes a CI lane and a
whole unresolved signing question.

Surface area:

- `.github/workflows/release.yml` — drop the `macos-latest` / `aarch64-apple-darwin` matrix
  entry.
- `src-tauri/tauri.conf.json` — drop `macOSPrivateApi`, drop `icons/icon.icns` from the bundle
  icon list.
- `src-tauri/Cargo.toml` — drop the `macos-private-api` Tauri feature.
- `src-tauri/icons/icon.icns` — delete.
- `ROADMAP.md` — retire the macOS system-loopback item, replaced by this document.
- `README.md`, `SECURITY.md`, `docs/architecture.md` — remove macOS mentions (Keychain,
  Gatekeeper, BlackHole).

Linux is a separate question: the Rust CI lane on Ubuntu is cheap and catches
non-`cfg(windows)` regressions, so keep it as a build check even with no Linux release
target.

## Sources

- [Tauri — Microsoft Store distribution](https://v2.tauri.app/distribute/microsoft-store/)
- [Microsoft Store Policies](https://learn.microsoft.com/en-us/windows/apps/publish/store-policies)
- [Free developer registration for individual developers](https://learn.microsoft.com/en-us/windows/apps/publish/whats-new-individual-developer)
- [Revamped company onboarding with zero registration fees](https://learn.microsoft.com/en-us/windows/apps/publish/whats-new-company-developer)
- [App package requirements for MSIX apps](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/app-package-requirements)
- [Package version numbering](https://learn.microsoft.com/en-us/windows/uwp/publish/package-version-numbering)
- [App capability declarations](https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/app-capability-declarations)
- [Code signing options for Windows app developers](https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/code-signing-options)
- [`Choochmeque/tauri-windows-bundle`](https://github.com/Choochmeque/tauri-windows-bundle)
- [`microsoft/winappCli`](https://github.com/microsoft/winappCli)
