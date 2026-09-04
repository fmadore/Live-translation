# Partner Center submission walkthrough

Submission target: **Live Translation & Subtitles 1.0.5**, Product ID `9PFB8LR3RR9X`.

## Before Partner Center

1. Build and verify both native packages:
   - `Live.Translation_1.0.5_x64.msixbundle`
   - `Live.Translation_1.0.5_arm64.msixbundle`
2. Install the signed ARM64 package on the target Surface and complete the manual test in
   [`microsoft-store.md`](microsoft-store.md).
3. Capture the new screenshots listed in [`store-listing.md`](store-listing.md). Do not reuse
   screenshots from the removed Windows Speech implementation.
4. Confirm the privacy policy is published at
   <https://fmadore.github.io/Live-translation/privacy>.

## Pricing and availability

| Field | Answer |
| --- | --- |
| Markets | All markets |
| Audience | Public audience |
| Discoverability | Available and discoverable in the Store |
| Schedule | Release as soon as certification passes |
| Base price | Free |
| Free trial / sale | None |
| Organizational licensing | Store-managed online licensing enabled |

The app sells no keys, credits, subscriptions, or other in-product offering.

## Properties

- Category: **Productivity**.
- Privacy policy: `https://fmadore.github.io/Live-translation/privacy`
- Website: `https://github.com/fmadore/Live-translation`
- Support: `https://github.com/fmadore/Live-translation/issues`
- Purchases outside Microsoft commerce: **No**.
- Accessibility certification: **No** unless a separate accessibility audit is completed.
- Generative AI: **Yes**, because optional Gemini/OpenAI live translation generates text.
- Personal-information transmission: **Yes**, because optional live modes send audio directly
  to a third-party provider; the developer receives none of it.

Microphone should not be listed as a minimum system requirement: the built-in default demo opens
no device. It is required only for a live Room mic session. Recommended memory: 8 GB.

## Age rating

Answer the third-party connection question **Yes** because live modes can connect to Google,
OpenAI, or Mistral. The built-in demo itself has no network access.

## Packages

Upload both unsigned Store bundles for version `1.0.5.0`. The Store signs accepted packages.
Do not upload the locally self-signed test package.

Verify these manifest values:

| Manifest element | Value |
| --- | --- |
| `Identity/Name` | `49346FMadore.LiveTranslationSubtitles` |
| `Identity/Publisher` | `CN=5D0ECC96-3998-452E-B7E9-29BE9B576F86` |
| `Properties/PublisherDisplayName` | `FMadore` |
| Version | `1.0.5.0` |

Windows Desktop is the only device family. Native x64 and ARM64 packages are supplied.

## Store listings

Create separate English (United States) and French (France) listings. Partner Center has three
different fields; do not combine them:

1. **Description** — full paragraphs only.
2. **Features / Fonctionnalités** — one feature per line; Partner Center renders the lines as
   bullets. Up to 20 entries.
3. **Short description / Description courte** — one short paragraph, maximum 270 characters.

All six exact copy-and-paste blocks are in [`store-listing.md`](store-listing.md):

- English full Description
- English Features
- English Short description — 175 characters
- French full Description
- French Fonctionnalités
- French Description courte — 171 characters

The provider dependency remains in the first paragraph of each full Description. The feature
list is intentionally not duplicated inside the Description.

### What’s new

English:

```text
Version 1.0.5 replaces device-dependent Windows speech recognition with a clearly labeled built-in caption demonstration that works on x64 and ARM64 without a microphone, language pack, network, account, or key. Optional live providers are unchanged.
```

Français :

```text
La version 1.0.5 remplace la reconnaissance vocale Windows dépendante de l’appareil par une démonstration intégrée clairement identifiée, compatible x64 et ARM64 sans microphone, module linguistique, réseau, compte ni clé. Les fournisseurs en direct restent inchangés.
```

Additional system requirements:

```text
Requires Windows 11 and Microsoft Edge WebView2 Runtime. The built-in demo works offline and without capture hardware. Live modes require internet access, a compatible third-party provider account and API key, and permission for the selected audio source.
```

## Submission options

Paste **Notes for certification** from [`store-listing.md`](store-listing.md), including the
exact default walkthrough and Product ID.

For `runFullTrust`, use this concise justification:

> Win32 desktop app packaged as MSIX; Windows.FullTrustApplication requires runFullTrust. Used
> for optional WASAPI microphone/system capture, Windows Credential Manager, the click-through
> overlay, and explicit transcript export. The built-in demo uses no device or network. No
> driver, service, background task, auto-start, relay, telemetry, analytics, or experimental ML
> runtime.

## Submit

Before clicking **Submit to the Store**, confirm:

- every section says Complete;
- the uploaded packages show the version in `src-tauri/tauri.conf.json` and the intended
  architectures;
- the first sentence of both descriptions discloses the live-provider dependency;
- a screenshot shows Built-in demo / Demo audio / Free / Start demo subtitles. This used to
  name the *first* screenshot, and the guarantee is what matters rather than the position:
  something in the set has to show that the default action needs no key and costs nothing, or
  10.1.2 has nothing to look at. Shot 1 now opens on the provider pre-flight, so shot 3 is
  where a reviewer finds it;
- certification notes contain the deterministic six-step test;
- no field or screenshot claims the built-in path recognizes speech.

If certification raises 10.1.2 again, reply with Product ID `9PFB8LR3RR9X`, the exact 1.0.5
test steps, and a short screen recording showing the installed ARM64 package completing them.
