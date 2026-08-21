# Partner Center submission walkthrough

The click-by-click sequence for the first submission of **Live Translation & Subtitles**
(Store ID `9PFB8LR3RR9X`), with every field this app needs an answer for.

The long-form copy lives in [`store-listing.md`](store-listing.md): the EN and FR store
descriptions (§1, §2), the notes for certification (§3), the IARC guidance (§4) and the
screenshot shot-list (§5). This file sequences the screens and supplies the *short* fields
that sheet does not carry. Where a block is fenced as `text`, paste it as-is.

Partner Center's layout shifts between releases. Section names below are the stable ones; if a
field appears that is not described here, the rule of thumb is that this app sells nothing,
collects nothing, and ships no server — answer from that.

---

## Step 0 — before opening Partner Center

1. **Download the package.** The `v1.0.0` release carries two files. Partner Center takes the
   bundle:

   - `Live.Translation_1.0.0_x64.msixbundle` → this is the upload.
   - `Live.Translation_1.0.0_x64.msix` → side-loading only, for taking the screenshots.

2. **Install v1.0.0 locally** and take the four screenshots from §5 of
   [`store-listing.md`](store-listing.md). They must be **≥1366×768 PNG**, and the operator
   window opens at 1200×820, so widen it or capture a whole 1920×1080 screen. Do not upscale.

3. Have the privacy policy URL to hand: <https://fmadore.github.io/Live-translation/privacy>

A submission can be saved and left half-finished; nothing goes to certification until the
final **Submit to the Store** button on the overview page.

---

## Step 1 — Pricing and availability

| Field | Answer |
| --- | --- |
| Markets | **All markets** (the default). Free app, two listing languages, no export concern. |
| Audience | **Public audience** |
| Discoverability | **Make this product available and discoverable in the Store** |
| Schedule | **Release as soon as it passes certification** |
| Base price | **Free** |
| Free trial | **No free trial** |
| Sale pricing | none |
| Organizational licensing | Tick **Make my product available to organizations with Store-managed (online) volume licensing**. It costs nothing and lets a university or conference centre deploy it centrally, which is exactly this app's audience. Leave the *disconnected (offline)* licensing box unticked. |

---

## Step 2 — Properties

**Category:** `Productivity`. Leave the subcategory unset — none of the offered subcategories
(document management, note taking, etc.) fits a captioning tool better than the parent does.
`Utilities & tools` is the defensible second choice if certification ever pushes back.

**Privacy policy URL** (required — the app transmits audio to a third party on the optional
paths):

```text
https://fmadore.github.io/Live-translation/privacy
```

**Website:**

```text
https://github.com/fmadore/Live-translation
```

**Support contact info:**

```text
https://github.com/fmadore/Live-translation/issues
```

### Product declarations

Three boxes arrive **ticked by default** (alternate drives, OneDrive backup, record/broadcast
clips). Two of those are right for this app and one is not, so go through the list rather than
accepting the defaults.

| Declaration | Set it to | Why |
| --- | --- | --- |
| Allows purchases, but not through the Microsoft commerce system | **unticked** | The app sells nothing. Ticking it contradicts the 10.8.3 argument in the certification notes and invites exactly the review this submission is designed to avoid. |
| Tested to meet accessibility guidelines | **unticked** | Tempting for a captioning tool, but this declares that *the app's own interface* was engineered and tested for screen readers, keyboard navigation, 4.5:1 contrast and high-contrast mode. That work has not been done. Declaring it falsely earns bad reviews from the people who rely on it. |
| Customers can install to alternate drives or removable storage | **ticked** (default) | No reason to pin the install to the system drive. |
| Windows can include this product's data in automatic OneDrive backups | **ticked** (default) | What lives in app data is the WebView2 profile: overlay font size, saved placement, last session setup. Harmless to back up, and it means settings follow the operator to a new machine. API keys are *not* included — they sit in Windows Credential Manager, which is the user's vault and not part of the package's app data. |
| Customers can use Windows 10/11 features to record and broadcast clips | **untick it** | Ticked by default, but it applies only to the Games category, and Partner Center says so in the warning right underneath. On a non-game it has no effect. Do not declare a game feature on a productivity app. |
| Supports pen and ink input | **unticked** | No stylus, inking or annotation surface. |
| Incorporates generative AI features | **ticked** | See below. |

**The generative AI declaration is a yes.** Microsoft's wording covers apps that generate
content dynamically using AI models, "accessed via cloud services or APIs" as well as built in.
Translation is the part that decides it: Gemini and OpenAI's realtime models produce
target-language text that did not previously exist, which is generation rather than lookup, and
they are a headline feature of the app. The bundled whisper recognizer is a neural model too,
even though transcription alone is closer to rendering existing speech than to creating
content.

Ticking it brings the Store's expectations for AI content into scope, and this app already
meets them: it is open source, and the *Support contact info* field points at the public issue
tracker, which is the means for a user to report a problem to the developer. The store
description already names Google, OpenAI and Mistral and explains what is streamed to them.
There is no separate obligation to moderate captions — the app reproduces speech that people in
the room are already saying, to those same people, and publishes nothing.

If a declaration asks whether the product **accesses, collects or transmits personal
information**, answer **yes** and point at the privacy policy. Audio leaves the machine on the
optional cloud paths. The developer receives nothing, but the honest answer is still yes, and
it is consistent with both the privacy policy and the IARC third-party answer.

### System requirements

Microphone is **Recommended**, not Minimum — captioning Windows system audio needs no
microphone at all, and that is the first-run default. Declaring it as minimum hardware would
misstate the app and could hide it from machines that can run it perfectly.

| Item | Minimum | Recommended |
| --- | --- | --- |
| Microphone | — | ✔ |
| Memory | 4 GB | 8 GB |

Leave touch, keyboard, mouse, camera, NFC, Bluetooth, telephony, DirectX, video memory and
graphics unset.

---

## Step 3 — Age ratings

Run the IARC questionnaire. §4 of [`store-listing.md`](store-listing.md) lists the expected
answer for each question area and flags the two easy mistakes — **answer the live
questionnaire from the app's behaviour, not from that sheet**, since IARC rewords its
questions regularly.

The one deliberate **yes** is the third-party connection question: with a cloud engine
selected, audio is streamed to Google, OpenAI or Mistral. Expected outcome is the lowest
rating from every board (PEGI 3, ESRB Everyone, USK 0). Anything higher means a question was
misread — go back through it rather than accepting the rating.

---

## Step 4 — Packages

Upload **`Live.Translation_1.0.0_x64.msixbundle`**.

Ingestion validates the manifest identity against the account. These three must match byte for
byte, and they already do — they are hardcoded in
[`AppxManifest.xml.template`](../src-tauri/gen/windows/AppxManifest.xml.template):

| Manifest element | Value |
| --- | --- |
| `Identity/Name` | `49346FMadore.LiveTranslationSubtitles` |
| `Identity/Publisher` | `CN=5D0ECC96-3998-452E-B7E9-29BE9B576F86` |
| `Properties/PublisherDisplayName` | `FMadore` |

All three were read back out of the built `v1.0.0` bundle and match, as does the package
version `1.0.0.0` — first segment non-zero, last segment zero, which is what the Store
requires.

After the upload settles:

- **Device family availability** — `Windows.Desktop` is the only family the package declares,
  with a minimum of `10.0.17763.0` (Windows 10 version 1809). Leave it ticked; leave the other
  families alone.
- **Gradual package rollout** — off. A first submission has nobody to roll out to.
- The package is **unsigned on purpose**. The Store signs it with the publisher certificate at
  ingestion. Do not upload a self-signed build.

If ingestion rejects the package, the error names the offending element; nothing in the
listing needs redoing, only the package.

---

## Step 5 — Store listings

Create **English (United States)** first, then **French (France)**. `fr-fr` rather than
`fr-ca`: Windows falls back from Canadian French to French, so one listing covers both, and
the reverse is not true.

### Product name

Pick **Live Translation & Subtitles** from the reserved names.

### Description

Paste §1 (EN) and §2 (FR) of [`store-listing.md`](store-listing.md) verbatim. Do not
reformat: those blocks are plain text on purpose, because the field does not render Markdown,
and the cost disclosure in the opening lines is what satisfies policy 10.2.4. It must stay at
the top.

### Short description

`en-us`:

```text
Real-time captions for bilingual meetings, lectures and conference sessions. Subtitles work free and offline with the bundled on-device recognizer, with no account and no API key. Live English-French translation is optional and uses your own provider key.
```

`fr-fr`:

```text
Des sous-titres en temps réel pour les réunions, les cours et les sessions de conférence bilingues. Le sous-titrage fonctionne gratuitement et hors ligne grâce au moteur embarqué, sans compte ni clé d'API. La traduction français-anglais en direct est optionnelle et utilise votre propre clé.
```

### Product features

Up to 20 entries, 200 characters each. One per line, entered as separate features.

`en-us`:

```text
Free live subtitles with no API key: the bundled on-device recognizer runs entirely on your PC, offline.
Live translation between English and French with your own Google Gemini or OpenAI key.
A transparent, click-through caption overlay that sits over slides, Teams or Zoom without intercepting a click.
Caption the room microphone, Windows system audio, or both at once, side by side.
Drag, nudge and resize the overlay on the projector, then lock it into place.
Export the transcript to plain text or Markdown, saved only when you ask.
A running cost meter: streamed minutes and a dollar estimate for the engine you picked.
Rehearsal mode plays a bundled speech sample through the real pipeline, so you can test before an event.
```

`fr-fr`:

```text
Des sous-titres en direct sans clé d'API : le moteur embarqué fonctionne entièrement sur votre PC, hors ligne.
La traduction en direct entre le français et l'anglais, avec votre propre clé Google Gemini ou OpenAI.
Une incrustation transparente qui laisse passer les clics et se superpose aux diapositives, à Teams ou à Zoom.
Sous-titrez le micro de la salle, le son système de Windows, ou les deux à la fois, côte à côte.
Déplacez, ajustez et redimensionnez l'incrustation sur le vidéoprojecteur, puis verrouillez-la.
Exportez la transcription en texte brut ou en Markdown, enregistrée uniquement à votre demande.
Un compteur de coût : les minutes transmises et une estimation en dollars pour le moteur choisi.
Un mode répétition qui joue un extrait sonore dans la vraie chaîne de traitement, pour essayer avant l'événement.
```

### Screenshots

The four from §5 of [`store-listing.md`](store-listing.md), in that order. The first is what
appears in search results, and it is the one that has to show *"No key needed"* and a
*"Free"* cost row.

Screenshots are per-language, but the same four files can be uploaded to both listings — the
app's own interface is English either way.

### What's new in this version

`en-us`:

```text
First release. Live subtitles from a bundled on-device recognizer that needs no key and no network, optional English-French translation through your own provider key, a transparent click-through overlay with move mode, capture from the microphone and Windows system audio at once, transcript export, a running cost meter, and a rehearsal mode for testing before an event.
```

`fr-fr`:

```text
Première version. Des sous-titres en direct grâce à un moteur embarqué qui ne demande ni clé ni réseau, la traduction français-anglais en option avec votre propre clé, une incrustation transparente qui laisse passer les clics, la capture simultanée du micro et du son système, l'export des transcriptions, un compteur de coût et un mode répétition pour préparer un événement.
```

### Search terms

Seven maximum, 30 characters each. These are invisible in the listing and exist only for
Store search, so spend them on words the description does not already rank for.

```text
live captions
subtitles
real-time translation
speech to text
meeting accessibility
sous-titres en direct
traduction en direct
```

### Additional system requirements

```text
Requires the Microsoft Edge WebView2 Runtime, which is already present on Windows 11 and on Windows 10 machines kept current through Microsoft Edge. An internet connection is needed only for the optional cloud engines; the bundled on-device engine works offline. ARM64 devices install and run the x64 package under Windows emulation, where only the on-device engine is slower.
```

### Copyright, licence, attribution

| Field | Value |
| --- | --- |
| Copyright and trademark info | `© 2026 Frédérick Madore. MIT licence.` |
| Additional license terms | `MIT licence: https://github.com/fmadore/Live-translation/blob/main/LICENSE` |
| Developed by | `Frédérick Madore` |

Leave Store logos and trailers empty. The package supplies its own tile assets, and a trailer
is not worth producing for a first submission.

---

## Step 6 — Submission options

**Notes for certification** — paste §3 of [`store-listing.md`](store-listing.md) verbatim.
This is the single most important field in the whole submission. It tells the reviewer how to
see the primary functionality with no key, no microphone and no network, and it pre-empts the
policy 10.8.3 objection before it is raised.

**Publishing hold options** — none. Publish as soon as it passes certification.

**Restricted capabilities** — `runFullTrust` **is** a restricted capability, and Partner Center
detects it in the manifest and asks you to justify it before the submission can proceed. This
is routine for every Win32 desktop app packaged as MSIX; it is not a sign that anything is
wrong with the package. (`microphone` is a device capability, not a restricted one, and needs
no justification.)

Paste this into *Why do you need the runFullTrust capability and how will it be used in your
product?*:

```text
This product is a Win32 desktop application packaged as MSIX through the Desktop Bridge. Its manifest declares EntryPoint="Windows.FullTrustApplication", which requires runFullTrust by construction: the capability is added by the packaging tool for every desktop application of this kind, and the application cannot start without it. It is not used to reach anything beyond the desktop APIs listed below.

The application is built with Tauri: a Rust core with a WebView2 user interface. It puts real-time captions on screen for bilingual meetings, lectures and conference sessions. Full trust is required for these specific Win32 APIs:

1. WASAPI loopback capture. To caption Windows system audio (a Teams or Zoom call, a browser tab, a media player), the application opens the default audio render endpoint for capture through the WASAPI COM interfaces IMMDeviceEnumerator, IAudioClient and IAudioCaptureClient. Loopback capture is a Win32-only API and is not reachable from a sandboxed UWP process. This is the default audio source on a fresh install and the path that provides the product's keyless primary functionality.

2. Microphone capture through the Win32 audio APIs, for captioning a room microphone. The microphone device capability is declared separately in the manifest.

3. On-device speech recognition. The package bundles whisper.cpp, a native C++ inference library, together with a GGML model file. Executing that native inference code and loading the model from the package install folder requires full trust. This is what allows the product to caption with no API key and no network connection at all.

4. Windows Credential Manager. If the user chooses one of the optional cloud engines, the API key they created on their own account with that provider is stored through CredWrite and read back with CredRead, under the service name org.stias.live-translation. Nothing else is stored there, and the key is transmitted only to the provider that issued it.

5. A transparent, click-through, always-on-top overlay window. The caption overlay is a second window that has to sit above full-screen presentation software without intercepting mouse input, so it is created undecorated and transparent, kept topmost, and made click-through by setting the extended window style that makes it ignore cursor events. It is repositioned and resized across displays when the operator places it on the projector.

6. Writing transcript exports. On an explicit user action only, the transcript is written to a Live-translation folder under the user's Documents directory. Nothing is written to disk otherwise; captured audio is never persisted.

The application installs no driver and no NT service, registers no background task, and does not start automatically. It runs only while the user has it open. It contacts no server operated by the developer: there is no telemetry, no analytics and no crash reporting. Network access occurs only when the user has selected a cloud engine, in which case audio is streamed over a wss:// connection to Google, OpenAI or Mistral using the key that user supplied.

The application is open source and the whole of the above can be verified in the code: https://github.com/fmadore/Live-translation

Privacy policy: https://fmadore.github.io/Live-translation/privacy
```

---

## Step 7 — Submit

Return to the submission overview. Every section should read **Complete**. Click **Submit to
the Store**.

What happens next:

- Preprocessing and certification typically finish within a few hours to three days for a
  first submission. Watch the email on the Partner Center account.
- **Expect a possible rejection on policy 10.8.3** (API keys read as financial information).
  The scoping argument is already written into the certification notes; if it comes back
  anyway, reply pointing at that section rather than rewriting the app. The substance of the
  argument is that captioning — the primary functionality — runs on a fresh install with no
  credential at all.
- If certification asks for anything that needs a code change, fix it, bump the version to
  `1.0.1` in `package.json`, `src-tauri/tauri.conf.json` and `src-tauri/Cargo.toml`, tag it,
  and upload the new bundle into the same submission.

---

## After it passes

- The listing goes live at <https://apps.microsoft.com/detail/9PFB8LR3RR9X>.
- Add that URL and the `ms-windows-store://pdp/?productid=9PFB8LR3RR9X` protocol link to the
  README.
- Housekeeping left over from testing: delete the draft releases `v1.0.0-rc.1` … `v1.0.0-rc.4`
  from GitHub, and remove the local development signing certificate — the commands are at the
  end of [`packaging-msix.md`](packaging-msix.md).
