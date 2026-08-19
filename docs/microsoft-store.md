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

1. Policy **10.8.3** classifies **"API secret keys"** as financial information, and bars
   individual accounts from requiring it for **primary functionality**. Today Live Translation
   cannot caption a single word without a Gemini/OpenAI/Mistral key. A company account is out
   of scope, so the fix has to be technical: a **keyless on-device captioning path**, after
   which the provider keys are an optional upgrade rather than a precondition.
2. Certification runs the app on a clean machine. With no key, the tester sees an app that
   does nothing — policy **10.3.1** wants a working demo credential in *Notes for
   certification*, and handing a live paid key to a stranger is not acceptable here. The same
   keyless path answers this.
3. An MSIX package version's **first segment cannot be `0`**, so `0.5.3` cannot ship. The
   Store forces a **1.0.0** release.

Gates 1 and 2 collapse into one piece of work, described under *On-device captioning* below.
It is the critical path for this whole plan, and it does not displace any existing provider —
Gemini, OpenAI and Mistral stay exactly as they are.

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

### 1. API keys as "financial information" — policy 10.8.3

> "If your product requires financial account information, you must submit that product from a
> company account type. Products from individual accounts cannot require financial
> information for primary functionality." Financial information "includes, but is not limited
> to … **API secret keys**, private keys, or recovery phrases."

**A company account is out of scope**, so the only way through is to stop *requiring* the
key. Note the exact wording: the prohibition is on requiring financial information for
**primary functionality**. It is not a ban on accepting an API key at all.

Two things in combination:

1. **Ship a keyless on-device subtitle path** (see *On-device captioning* below) so the app
   captions speech out of the box with no credential of any kind. Provider keys become an
   optional upgrade — better accuracy, and the only route to translation — rather than a
   precondition. This is what actually takes the app out of 10.8.3's scope.
2. **Argue the scoping in Notes for certification.** Section 10.8's own opening line limits it
   to products that "include in-product purchase, subscriptions, virtual currency, billing
   functionality or capture financial information". Live Translation has none of the first
   four. The key it accepts is a third-party service credential the user already holds, is
   sent only to the provider that issued it, grants access to no account balance, and buys
   nothing in-app.

Even with the keyless path, budget for one rejection round — the enumeration names "API secret
keys" literally, and a reviewer may apply it literally. **If certification rejects it anyway,
there is no fallback inside the Store**: with a company account ruled out, the app stays on
GitHub releases and the SmartScreen mitigations below become the permanent answer rather than
a stopgap. Weigh that before investing in Phase C.

### 2. Testability without a key — policy 10.3.1

> "If your product requires login credentials, provide us with a working demo account using
> the **Notes for certification** field."

Certification needs to see the app work on a clean machine. Three things, in order of value:

- **The keyless on-device subtitle path** (gate 1) — the tester installs, speaks, and sees
  captions. No credential to hand over, nothing to revoke afterwards.
- **Rehearsal mode** — already on the roadmap under "Future ideas". Play a bundled short FR/EN
  fixture through the pipeline so the overlay, move mode and export can be exercised without a
  microphone at all. Cheap once the keyless path exists, and useful on event day.
- **Notes for certification** text explaining that captions work out of the box, that provider
  keys are an optional upgrade for accuracy and translation, and linking the three provider
  signup pages.

Last resort if both slip: issue a Mistral key (cheapest at $0.006/min) with a low spend cap,
hand it over once, and revoke it after certification.

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
| Building the keyless on-device subtitle path (gate 1, the critical path) | the real cost — a new capture-to-caption engine, plus UI |
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

- [ ] Enrol in Partner Center as an **individual** and complete identity verification
      (government ID plus selfie). Company accounts are out of scope, so gate 1 is answered in
      code, not paperwork.
- [ ] Draft the 10.8.3 scoping argument for Notes for certification now, while the reasoning
      is fresh — it is needed at submission and it sanity-checks the keyless design.
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

### Phase D — app changes

**The first item is the critical path — gates 1 and 2 both depend on it, and it should start
before Phase C rather than after.**

- [x] **Keyless on-device backend, engine-independent half** — `Provider::OnDevice`,
      `ondevice/run_session`, the operator UI, and unit tests. Gemini, OpenAI and Mistral are
      untouched.
- [ ] **Pick and implement the recognizer** in `ondevice/engine.rs` — `whisper-rs` ships
      today; the Speech Recognition Windows AI API is the migration target once it leaves the
      experimental channel. Inbox `Windows.Media.SpeechRecognition` is ruled out: no audio
      input API.
- [ ] First-run state that captions immediately with no key, and presents provider keys as an
      optional upgrade (accuracy, and translation) rather than a precondition.
- [ ] **Rehearsal mode** (gate 2) — bundled FR/EN fixture through the full pipeline.
- [ ] Permission-denied microphone path with an actionable message (gate 6).
- [ ] WebView2 presence assertion (gate 9).
- [ ] *Deferred:* re-target the Speech Recognition Windows AI API once it leaves the Windows
      App SDK experimental channel — free, no bundled model, and NPU-accelerated on Copilot+
      hardware. Tracked in `ondevice/engine.rs`.

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

## macOS support: dropped

Done — this route is Windows-only by construction, macOS system-audio capture was gated on a
$99/year Apple Developer membership before any of its capture code became useful, and the
STIAS event laptop is Windows. Removed from `release.yml`, `tauri.conf.json` (`macOSPrivateApi`,
`icon.icns`), `Cargo.toml` (the `macos-private-api` feature), the roadmap's system-loopback
item, and the README/SECURITY/architecture prose.

Linux stays as a **CI compile check only**: the Ubuntu Rust lane is cheap and catches
regressions in the non-`cfg(windows)` code. It produces no release artifact.

## On-device captioning (the 10.8.3 mitigation)

With a company account off the table, this stops being a nice-to-have and becomes the
load-bearing part of the plan. It also pays for itself three other ways: it removes the
API-key wall from first run, it gives certification something to test (gate 2), and it gives
the workshop a fallback when the venue network fails.

**None of this replaces Gemini, OpenAI or Mistral.** They stay exactly as they are — the
on-device engine is an additional, keyless source of captions, and the cloud providers remain
the quality path and the *only* path for translation.

### What Windows can and cannot do on-device

**Speech-to-text: yes.** Microsoft's
[Speech Recognition Windows AI API](https://learn.microsoft.com/en-us/windows/ai/apis/speech-recognition)
does real-time on-device transcription from live audio, free, no network, no key, built on
components of OpenAI Whisper. Both French and English are supported.

**Translation: no.** There is no on-device translation API on Windows; the Windows AI
catalogue has no translator and lists "Live Caption Translations" as *not yet supported*. So
**Live translation stays cloud-only and stays key-gated.** The keyless path covers **Live
subtitles** — which is enough for 10.8.3, because captioning is the primary functionality and
it works with no credential.

### Copilot+ is an upgrade, not the gate — and that matters

The instinct to target Copilot+ PCs is understandable, but **do not restrict the feature to
them**. If the keyless mode only ran on Copilot+ hardware, the 10.8.3 argument would become
hardware-dependent and a certification tester on an ordinary VM would still meet a key-walled
app — defeating the whole point.

Fortunately that restriction no longer exists. At Build 2026 Microsoft
[extended Windows AI APIs from NPU-only to CPU and GPU](https://blogs.windows.com/windowsdeveloper/2026/06/02/build-2026-furthering-windows-as-the-trusted-platform-for-development/).
On a Copilot+ PC the model is preinstalled and runs on the NPU; elsewhere it downloads on
demand through Windows Update the first time the app calls `EnsureReadyAsync`, then runs on
CPU. Copilot+ buys latency and battery, not access.

### Engine selection: what is ruled out, and what is left

A recognizer here must accept **pushed PCM**. That is not a preference: the app captures
system audio over WASAPI loopback and offers microphone selection, so an engine that opens
its own audio device can caption neither system audio nor a chosen microphone, and cannot
serve *Both* mode at all.

**That rules out the inbox `Windows.Media.SpeechRecognition` namespace**, which this document
previously recommended. Its API surface has no audio input of any kind — no stream, file,
buffer or device selector; it always opens the system default capture device itself.
Verified against the generated bindings for the `windows` crate 0.62, where `SpeechRecognizer`
exposes only constraints, timeouts, UI options and the continuous-session handle. It could
back a microphone-only demo, but not this app's actual feature set.

Two candidates remained, and **`whisper-rs` is now implemented**:

| Engine | Accepts pushed PCM | Ships today? | Quality | Cost |
| --- | --- | --- | --- | --- |
| **`whisper-rs` (whisper.cpp)** — *shipped* | Yes — PCM directly | **Yes** — stable, no Windows version dependency | Best available from anything stable | Bundled model, a C++/CMake step in the build, CPU headroom |
| **Speech Recognition Windows AI API** — *migration target* | Yes — `SpeechAudioProvider` | **No** — Windows App SDK *experimental channel*, which cannot back a Store submission. Also a `Microsoft.Windows.*` App SDK type, not an OS `Windows.*` one, so the `windows` crate does not project it: Rust use needs the App SDK bootstrapper and a projection of its own | Whisper-derived; NPU on Copilot+, CPU elsewhere | Free, and **no bundled model at all** |

The Windows AI API remains the better long-term answer and the one to migrate to the moment
it reaches the stable channel: free, NPU-accelerated where hardware allows, and it would drop
the model from the installer entirely. It simply cannot carry a Store submission today.

**On installer size** — the obvious objection to whisper — the bundled model defaults to the
5-bit quantized `ggml-base-q5_1` at **57 MiB** rather than full `base` at 141 MiB. That is a
small accuracy cost on an engine already documented as less accurate than Voxtral, and it
keeps the Store package in a defensible range. `WHISPER_MODEL` selects another size at build
time and `WHISPER_MODEL_PATH` overrides at runtime, so this is tunable after a rehearsal
rather than baked in.

### What is built

Complete and on the branch:

- `Provider::OnDevice` end to end, with `requires_api_key` and `can_translate` predicates so
  the keyless path is a first-class backend rather than a special case. `session.rs` skips
  key resolution for it, and validates mode/provider through the capability rather than a
  provider list.
- `ondevice/mod.rs` — the session driver, mirroring `realtime::run_session`'s signature:
  bounded audio consumption, drop-newest backpressure, turn bookkeeping, per-origin status,
  graceful flush on stop. Unit tested (partials replace rather than append, an empty final
  closes an open turn but not an idle one, subtitles land in the audience field).
- `ondevice/engine.rs` — the pluggable point and model resolution;
  `ondevice/whisper.rs` — whisper.cpp behind the `Recognizer` trait, with a sliding-window
  strategy (whisper transcribes buffers, not streams), an energy gate that keeps the model
  away from silence where it invents text, a filter for bracketed non-speech annotations, and
  a shared model cache so *Both* mode loads one copy of the weights.
- `scripts/fetch-whisper-model.mjs` plus a release-workflow step; the model is bundled as a
  Tauri resource and deliberately not committed.
- Operator UI — **Mistral Voxtral | On-device · no key** under Live subtitles, the API-key
  panel hidden for the keyless engine, Start no longer gated on a key, and a language hint
  for the recognizer.

**Not yet verified:** caption latency and accuracy against real conference audio, and the
build on the `windows-11-arm` runner — whisper.cpp compiles from source, so ARM64 Windows is
a genuine risk to check before relying on a release. Neither can be tested from CI alone.

### Where it fits in the architecture

`session.rs` spawns one `realtime::run_session` per active source, each consuming the bounded
audio channel and emitting `Caption` / `StatusUpdate` events. An on-device engine is a sibling
of that task, not a `RealtimeProtocol` implementation — it skips the WebSocket entirely but
consumes the same channel and emits the same events, so the operator UI, the overlay, the
per-origin turn accounting and the transcript export all work unchanged. It appears in the UI
as a fourth provider under **Live subtitles**, alongside Mistral.

Two consequences worth planning for: the model download on first use needs the loading-UI
pattern (`EnsureReadyAsync` progress) if the AI API is ever adopted, and **Windows AI APIs
require package identity** — which the MSIX build supplies anyway, but which the NSIS build
would need a sparse package to obtain. The inbox API has no such requirement.

### One speculative option, worth an experiment and not a plan

The app renders in WebView2, and Edge 148 ships on-device `Translator` and `LanguageDetector`
JavaScript APIs covering 145+ languages at no cost. Whether those surface inside WebView2, at
what runtime version, and whether they are usable for realtime caption text is unknown. If
they are, an on-device *translation* leg becomes possible after all — which would close the
one gap above. Timebox it to an afternoon before believing it.

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
- [Speech Recognition with Windows AI APIs](https://learn.microsoft.com/en-us/windows/ai/apis/speech-recognition)
- [What are Windows AI APIs?](https://learn.microsoft.com/en-us/windows/ai/apis/)
- [Build 2026 — Windows as the trusted platform for development](https://blogs.windows.com/windowsdeveloper/2026/06/02/build-2026-furthering-windows-as-the-trusted-platform-for-development/)
- [`Windows.Media.SpeechRecognition` namespace](https://learn.microsoft.com/en-us/uwp/api/windows.media.speechrecognition)
