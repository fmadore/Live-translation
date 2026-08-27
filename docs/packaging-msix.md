# MSIX packaging

How to build the Microsoft Store package on your own machine, sign it so Windows will
install it, and check the things CI cannot check — microphone capture, WASAPI loopback,
Credential Manager, the overlay and the export path, all running under package identity.

[`microsoft-store.md`](microsoft-store.md) has the why: the Store policy gates, the assigned
identity values, and the phased plan. This file is the mechanics. It covers Phase C's
"build locally, sign, install, verify gates 6, 7 and 8".

## What is committed

Tauri has no MSIX bundle target, so packaging goes through
[`@choochmeque/tauri-windows-bundle`](https://github.com/Choochmeque/tauri-windows-bundle),
a devDependency. It drives `tauri build --no-bundle`, assembles the package payload, and
hands it to the Windows SDK's `MakeAppx`.

| Path | What it is |
| --- | --- |
| `src-tauri/gen/windows/bundle.config.json` | Publisher, capabilities, signing. Read at build time. |
| `src-tauri/gen/windows/AppxManifest.xml.template` | The manifest, with `{{PLACEHOLDER}}`s the bundler fills in per architecture. |
| `src-tauri/gen/windows/Assets/*.png` | Store logo and tiles, copied into the package verbatim. |
| `src-tauri/icons/{StoreLogo,Square44x44Logo,Square150x150Logo}.png` | `tauri icon` output for the MSIX sizes; the source the bundler regenerates `Assets/` from. |
| `package.json` → `bundle:msix:{x64,arm64}` | Architecture-specific Store package commands. |
| `.github/workflows/release.yml` → `msix` job | The same command on `windows-latest`, attaching an unsigned package to the release. |

`src-tauri/gen/` is otherwise generated and ignored; `.gitignore` carries an exception for
`gen/windows/`.

Three things in there are not what they look like:

- **`"displayName": "live-translation"` in `bundle.config.json` is a filename, not a label.**
  The bundler derives the packaged executable's name from it (whitespace stripped, plus
  `.exe`) and copies that file out of the Rust target directory. Tauri leaves the cargo
  binary name alone, so the file to find is `live-translation.exe`. Every string a user
  actually sees is a literal in the manifest template.
- **`Identity/Name` and both display names are literals, not placeholders.** The bundler
  derives the identity name from `tauri.conf.json`'s `identifier` (`org.stias.live-translation`)
  and the display name from `productName`, and neither is what Partner Center assigned. The
  reserved name also carries an ampersand, which has to be `&amp;` in XML. Ingestion fails if
  any of these differ by a character, so they are written out in the template and checked
  against *Store identity (assigned)* in `microsoft-store.md`.
- **`"publisher"` in `tauri.conf.json` is load-bearing.** Tauri defaults the Windows publisher
  to the second element of `identifier`, so `org.stias.live-translation` used to name the
  workshop venue as the author: *stias* in the packaged executable's `CompanyName` and in the
  NSIS/MSI installers attached to a GitHub release. `"publisher": "Frédérick Madore"` sets it
  explicitly. The Store identity is unaffected either way — that comes from
  `bundle.config.json` — so the identifier itself must not be touched.

The manifest the bundler produces carries the matching `x64` or `arm64` architecture:

```xml
<Identity
  Name="49346FMadore.LiveTranslationSubtitles"
  Publisher="CN=5D0ECC96-3998-452E-B7E9-29BE9B576F86"
  Version="1.0.5.0"
  ProcessorArchitecture="x64" />
```

with `runFullTrust` and `microphone`. The built-in demonstration uses neither capability nor
network access. The package carries no speech model, Windows AI runtime, WinRT recognizer, or
`systemAIModels` declaration. Optional live capture uses microphone/WASAPI through the
full-trust desktop process.

## Prerequisites

- **Rust stable** with both Store targets: `rustup target add x86_64-pc-windows-msvc aarch64-pc-windows-msvc`.
- **Windows SDK**, for `MakeAppx.exe` (packing) and `SignTool.exe` (signing). Installed by
  the Visual Studio "Desktop development with C++" workload or the standalone SDK. Nothing
  needs to be on `PATH`: the packer resolves the SDK through
  `HKLM\SOFTWARE\Microsoft\Windows Kits\Installed Roots` → `KitsRoot10`.
- **Node 22 or 24**, then `npm ci`.

## Build the package

```powershell
npm ci
npm run bundle:msix:arm64
```

That builds the front end, compiles the app for `aarch64-pc-windows-msvc` with `--no-bundle`
(so no NSIS or MSI installer — use `npm run tauri build` for those), stages the payload in
`src-tauri/target/appx/arm64/`, and writes:

```text
src-tauri/target/msix/Live Translation & Subtitles_1.0.5.0_arm64.msix
src-tauri/target/msix/Live Translation & Subtitles_1.0.5.0.msixbundle
```

The names come from the manifest's `DisplayName`; CI renames the bundle to
`Live.Translation_<version>_arm64.msixbundle` before attaching it to the release. CI builds
and publishes the corresponding x64 pair in parallel. Upload both `.msixbundle` files to the
submission. For local verification the plain `.msix` is easier to sign and install.

Sanity checks worth a glance on the staged payload:

```powershell
Get-ChildItem src-tauri/target/appx/arm64 -Recurse -Name
```

`live-translation.exe`, `AppxManifest.xml`, `Assets\*.png`, and the two neutral rehearsal
fixtures should be there. There should be no `windows-ai` directory or self-contained ML DLL.

Do **not** pass `--regenerate-assets`. It rebuilds `gen/windows/Assets/` from
`src-tauri/icons/`, and its wide-tile generator writes an all-black 310×150 image; the
committed `Wide310x150Logo.png` is the corrected one.

## Sign it, so Windows will install it

For this repository's local certificate files, the shortest safe path is the checked-in helper.
Open Windows PowerShell **as Administrator** from the repository root; it prompts securely for
the PFX password, trusts the matching public certificate in Local Machine → Trusted People,
signs, verifies, installs, and verifies version 1.0.5:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\install-local-msix.ps1 -Architecture arm64
```

Use `-Architecture x64` on an x64 test machine. The manual equivalent follows.

Windows installs no unsigned MSIX. The Store re-signs the submitted package with its own
certificate, which is the whole reason this route removes the SmartScreen warning — so the
certificate below exists only to get the package onto your own machine, and is thrown away
afterwards.

The certificate's **subject must equal the manifest's `Publisher` exactly**, or the install
fails with a publisher mismatch.

1. Create it (normal, non-elevated PowerShell — this writes to your own store):

   ```powershell
   $cert = New-SelfSignedCertificate -Type Custom `
     -Subject "CN=5D0ECC96-3998-452E-B7E9-29BE9B576F86" `
     -KeyUsage DigitalSignature `
     -FriendlyName "Live Translation dev signing" `
     -CertStoreLocation Cert:\CurrentUser\My `
     -TextExtension @("2.5.29.37={text}1.3.6.1.5.5.7.3.3","2.5.29.19={text}")
   ```

   The two extensions are what make it usable here: extended key usage *code signing*
   (`1.3.6.1.5.5.7.3.3`) and an empty basic-constraints extension (not a CA).

2. Export it:

   ```powershell
   $password = Read-Host -AsSecureString "Password for the dev PFX"
   Export-PfxCertificate -Cert "Cert:\CurrentUser\My\$($cert.Thumbprint)" `
     -FilePath .\live-translation-dev.pfx -Password $password
   ```

   `*.pfx` is git-ignored, so a checkout is a safe place to keep it while you work.

3. Trust it, in an **elevated** PowerShell:

   ```powershell
   Import-PfxCertificate -FilePath .\live-translation-dev.pfx `
     -CertStoreLocation Cert:\LocalMachine\TrustedPeople `
     -Password (Read-Host -AsSecureString "Password for the dev PFX")
   ```

   `LocalMachine\TrustedPeople` — not `Root`. Both stores satisfy the MSIX installer, but
   Trusted People is trusted *for installing signed app packages* and nothing else, whereas a
   certificate in Trusted Root Certification Authorities is trusted for everything on the
   machine, including TLS. A throwaway key with a private half sitting in your Downloads
   folder does not belong in Root.

4. Sign the package. `signtool.exe` lives under the SDK; take the newest:

   ```powershell
   $signtool = (Get-ChildItem "${env:ProgramFiles(x86)}\Windows Kits\10\bin\*\x64\signtool.exe" |
     Sort-Object FullName -Descending | Select-Object -First 1).FullName
   $password = Read-Host -AsSecureString "Password for the dev PFX"
   $cert = Import-PfxCertificate -FilePath .\live-translation-dev.pfx `
     -CertStoreLocation Cert:\CurrentUser\My -Password $password
   & $signtool sign /fd SHA256 /sha1 $cert.Thumbprint `
     ".\src-tauri\target\msix\Live Translation & Subtitles_1.0.5.0_arm64.msix"
   ```

   No timestamp URL: the certificate is valid for a year and the package is disposable.

5. Install it:

   ```powershell
   Add-AppxPackage -Path ".\src-tauri\target\msix\Live Translation & Subtitles_1.0.5.0_arm64.msix"
   ```

   Installing a *rebuilt* package with the same version fails with `0x80073CFB` ("same
   identity, different contents") — Windows only swaps packages in place when the version
   increases. Remove the old one first; Credential Manager keys survive, WebView-stored
   preferences reset:

   ```powershell
   Remove-AppxPackage 49346FMadore.LiveTranslationSubtitles_1.0.0.0_x64__6yxybgjxsxtpc
   ```

6. Confirm the identity is the one Partner Center assigned — the family name's hash suffix is
   derived from `Name` plus `Publisher`, so this is a byte-for-byte check of both:

   ```powershell
   Get-AppxPackage 49346FMadore.LiveTranslationSubtitles |
     Select-Object Name, PackageFamilyName, Version, InstallLocation
   ```

   `PackageFamilyName` must read `49346FMadore.LiveTranslationSubtitles_6yxybgjxsxtpc`. If the
   suffix differs, the manifest does not match the reservation and the Store will reject the
   upload.

## Verify under package identity

Launch from the Start menu (not the staged `.exe` — that runs without identity and proves
nothing). Then work through the list; the first three are the open gates in
`microsoft-store.md`.

| Check | What "pass" looks like |
| --- | --- |
| **Built-in demo (certification default)** | Without changing Windows settings or attaching audio hardware, click *Start demo subtitles*. Demo status, elapsed time, level movement, partial/final captions, overlay, Stop, and export all work. Repeat in English and French. |
| **Microphone (optional live mode)** | With Mistral/Gemini/OpenAI configured, the app appears under Settings → Privacy & security → Microphone and captures with access on. With access blocked, it reports the exact Settings path. |
| **WASAPI loopback (gate 7)** | Join a real Teams or Zoom call from the same machine, run *System audio* or *Both*, and confirm the far end is captioned. This is the highest-risk item in the whole plan: it cannot be tested in CI and it invalidates the route if it fails. |
| **Credential Manager (gate 8)** | Save a provider key in an *unpackaged* build first (`npm run tauri build`, or dev), then start the MSIX build and confirm the key is already there. `Control Panel → Credential Manager → Windows Credentials` should show one generic credential `org.stias.live-translation`, not two. |
| **Overlay** | Transparent background, click-through to the window behind, always on top over a full-screen slide deck, and move mode still drags it. |
| **Export** | Export a transcript and confirm it lands in `Documents\Live-translation\`. Full-trust packages are not filesystem-virtualised, so this should be the real Documents folder — check that it is, and not a `Packages\…\LocalCache` redirect. |

Anything that fails here is a bug to fix before submission, not a packaging setting to tweak:
the same binary runs in both shapes.

## Remove the dev certificate afterwards

```powershell
Get-AppxPackage 49346FMadore.LiveTranslationSubtitles | Remove-AppxPackage
Remove-Item .\live-translation-dev.pfx

# Elevated:
Get-ChildItem Cert:\LocalMachine\TrustedPeople |
  Where-Object Subject -eq "CN=5D0ECC96-3998-452E-B7E9-29BE9B576F86" | Remove-Item

# Your own store:
Get-ChildItem Cert:\CurrentUser\My |
  Where-Object FriendlyName -eq "Live Translation dev signing" | Remove-Item
```

Leaving the certificate in Trusted People means the machine keeps installing anything signed
with that key. Remove it when the verification round is done.

## Debugging without repacking

Signing and reinstalling an MSIX for every code change is not a debug loop. Microsoft's
[`winapp` CLI](https://github.com/microsoft/winappCli) grants package identity to an
unpackaged build through loose-layout registration, so packaged-only behaviour can be
reproduced against an ordinary build:

```powershell
winget install Microsoft.winappcli --source winget
winapp run
```

`microsoft-store.md` recommends keeping it installed for exactly this. It also has its own
`winapp init` / `winapp cert` / `winapp pack` flow, which is a first-party alternative to the
packaging above — single-architecture, and not what the release workflow uses.

## What CI does

The `msix` job in `.github/workflows/release.yml` runs the same build command on
`windows-latest` after the installer job has created the release, then renames the bundle and
attaches it with `gh release upload`. It signs nothing: Partner Center expects an unsigned
package and the Store applies its own signature.
