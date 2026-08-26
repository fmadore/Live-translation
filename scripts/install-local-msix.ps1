#Requires -RunAsAdministrator
[CmdletBinding()]
param(
    [ValidateSet('arm64', 'x64')]
    [string]$Architecture = 'arm64',

    [string]$Version = '1.0.5'
)

$ErrorActionPreference = 'Stop'

$repository = Split-Path -Parent $PSScriptRoot
$packageName = "Live Translation & Subtitles_$Version.0_$Architecture.msix"
$packagePath = Join-Path $repository "src-tauri\target\msix\$packageName"
$pfxPath = Join-Path $repository 'live-translation-dev.pfx'
$cerPath = Join-Path $repository 'live-translation-dev.cer'

foreach ($requiredPath in @($packagePath, $pfxPath, $cerPath)) {
    if (-not (Test-Path -LiteralPath $requiredPath -PathType Leaf)) {
        throw "Required file not found: $requiredPath"
    }
}

$signTool = Get-ChildItem "${env:ProgramFiles(x86)}\Windows Kits\10\bin\*\x64\signtool.exe" |
    Sort-Object FullName -Descending |
    Select-Object -First 1
if (-not $signTool) {
    throw 'SignTool.exe was not found. Install the Windows SDK first.'
}

$password = Read-Host -AsSecureString 'Password for live-translation-dev.pfx'
$signingCertificate = Import-PfxCertificate `
    -FilePath $pfxPath `
    -CertStoreLocation 'Cert:\CurrentUser\My' `
    -Password $password
$trustedCertificate = Import-Certificate `
    -FilePath $cerPath `
    -CertStoreLocation 'Cert:\LocalMachine\TrustedPeople'

if ($signingCertificate.Thumbprint -ne $trustedCertificate.Thumbprint) {
    throw "PFX/CER mismatch: $($signingCertificate.Thumbprint) != $($trustedCertificate.Thumbprint)"
}
if (-not $signingCertificate.HasPrivateKey) {
    throw 'The imported PFX does not expose its private signing key.'
}

& $signTool.FullName sign `
    /fd SHA256 `
    /sha1 $signingCertificate.Thumbprint `
    $packagePath
if ($LASTEXITCODE -ne 0) {
    throw "SignTool failed with exit code $LASTEXITCODE"
}

& $signTool.FullName verify /pa $packagePath
if ($LASTEXITCODE -ne 0) {
    throw "Signature verification failed with exit code $LASTEXITCODE"
}

Add-AppxPackage -Path $packagePath -ForceApplicationShutdown -ForceUpdateFromAnyVersion

$installed = Get-AppxPackage 49346FMadore.LiveTranslationSubtitles
if (-not $installed -or $installed.Version.ToString() -ne "$Version.0") {
    throw "Installation verification failed; expected package version $Version.0"
}

Write-Host "Installed $($installed.Name) $($installed.Version) ($Architecture)." -ForegroundColor Green
Write-Host 'Open Live Translation & Subtitles from the Start menu.' -ForegroundColor Green
