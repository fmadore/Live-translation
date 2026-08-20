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
| `package.json` → `bundle:msix` | `tauri-windows-bundle build --arch x64 --runner npm`. |
| `.github/workflows/release.yml` → `msix` job | The same command on `windows-latest`, attaching an unsigned package to the release. |

`src-tauri/gen/` is otherwise generated and ignored; `.gitignore` carries an exception for
`gen/windows/`.

Two things in there are not what they look like:

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

The manifest the bundler produces for x64 carries:

```xml
<Identity
  Name="49346FMadore.LiveTranslationSubtitles"
  Publisher="CN=5D0ECC96-3998-452E-B7E9-29BE9B576F86"
  Version="1.0.0.0"
  ProcessorArchitecture="x64" />
```

with `runFullTrust` (restricted) and `microphone` (device) as the only capabilities. Loopback
capture needs none — a full-trust package runs outside an AppContainer — and neither does
Credential Manager.

## Prerequisites

- **Rust stable** with the x64 target: `rustup target add x86_64-pc-windows-msvc`.
- **CMake and the MSVC C++ toolset.** whisper.cpp is built from source by `whisper-rs`.
- **Windows SDK**, for `MakeAppx.exe` (packing) and `SignTool.exe` (signing). Installed by
  the Visual Studio "Desktop development with C++" workload or the standalone SDK. Nothing
  needs to be on `PATH`: the packer resolves the SDK through
  `HKLM\SOFTWARE\Microsoft\Windows Kits\Installed Roots` → `KitsRoot10`.
- **Node 22 or 24**, then `npm ci`.
- **The speech model.** `npm run fetch:whisper-model` downloads `ggml-base-q5_1.bin` into
  `src-tauri/resources/models/`. It is a bundled Tauri resource and is not committed. Without
  it the package builds *successfully* and ships without on-device subtitles, so do not skip
  it. (CI fails the job instead of shipping that.)

## Build the package

```powershell
npm ci
npm run fetch:whisper-model
npm run bundle:msix
```

That builds the front end, compiles the app for `x86_64-pc-windows-msvc` with `--no-bundle`
(so no NSIS or MSI installer — use `npm run tauri build` for those), stages the payload in
`src-tauri/target/appx/x64/`, and writes:

```text
src-tauri/target/msix/Live Translation & Subtitles_1.0.0.0_x64.msix
src-tauri/target/msix/Live Translation & Subtitles_1.0.0.0.msixbundle
```

The names come from the manifest's `DisplayName`; CI renames the bundle to
`Live.Translation_<version>_x64.msixbundle` before attaching it to the release. The
`.msixbundle` is the submission artifact — today it wraps the single x64 package, and it is
where an arm64 package would join it. For local verification the plain `.msix` is easier to
sign and install; they contain the same payload.

Sanity checks worth a glance on the staged payload:

```powershell
Get-ChildItem src-tauri/target/appx/x64 -Recurse -Name
```

`live-translation.exe`, `AppxManifest.xml`, `Assets\*.png` and `models\ggml-*.bin` should all
be there — the model directory is what the on-device engine resolves through Tauri's resource
directory, which under MSIX is the package root next to the executable.

Do **not** pass `--regenerate-assets`. It rebuilds `gen/windows/Assets/` from
`src-tauri/icons/`, and its wide-tile generator writes an all-black 310×150 image; the
committed `Wide310x150Logo.png` is the corrected one.

## Sign it, so Windows will install it

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
   & $signtool sign /fd SHA256 /f .\live-translation-dev.pfx /p '<password>' `
     ".\src-tauri\target\msix\Live Translation & Subtitles_1.0.0.0_x64.msix"
   ```

   No timestamp URL: the certificate is valid for a year and the package is disposable.

5. Install it:

   ```powershell
   Add-AppxPackage -Path ".\src-tauri\target\msix\Live Translation & Subtitles_1.0.0.0_x64.msix"
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
| **Microphone (gate 6)** | The app appears under Settings → Privacy & security → Microphone, and captions work with it on. Toggle it off, start a session: the failure must be a legible message naming that Settings path, not a raw cpal error. |
| **WASAPI loopback (gate 7)** | Join a real Teams or Zoom call from the same machine, run *System audio* or *Both*, and confirm the far end is captioned. This is the highest-risk item in the whole plan: it cannot be tested in CI and it invalidates the route if it fails. |
| **Credential Manager (gate 8)** | Save a provider key in an *unpackaged* build first (`npm run tauri build`, or dev), then start the MSIX build and confirm the key is already there. `Control Panel → Credential Manager → Windows Credentials` should show one generic credential `org.stias.live-translation`, not two. |
| **On-device engine** | Pick *On-device · no key* under Live subtitles and speak. Captions prove `models\ggml-*.bin` survived into the package. |
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

The `msix` job in `.github/workflows/release.yml` runs the same three commands on
`windows-latest` after the installer job has created the release, then renames the bundle and
attaches it with `gh release upload`. It signs nothing: Partner Center expects an unsigned
package and the Store applies its own signature. The job fails rather than uploading if the
speech model is missing from the staged payload.
