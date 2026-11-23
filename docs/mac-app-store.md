# Mac App Store submission

How Live Translation & Subtitles reaches the Mac App Store: what to set up once at Apple and on
GitHub, what to answer in App Store Connect, the listing and review notes, and the steps for each
version. The port itself is described in [macos.md](macos.md); the Microsoft Store's steps are in
[microsoft-store.md](microsoft-store.md). Kalamu went through the same steps first
(`docs/mac-app-store.md` in that repository), with the same team.

## Status (23 November 2026)

**1.16.0 (build 9) was submitted for review on 23 November 2026 at 21:07** and is *Waiting for
Review*, set to release automatically once approved (the maintainer's choice). It replaced
**1.13.1**, which had been *Waiting for Review* since 11 November without an answer: the
account was in order (agreements, bank account, tax forms and the EU Digital Services Act
declaration all active), so the wait was App Review's queue. 1.13.1 was withdrawn the same
evening (*Developer Rejected*), and its version record was renamed 1.16.0 and given build 9,
uploaded from the `v1.16.0` tag with the *Mac App Store* workflow (run 45222738780). 1.14.0 and
1.15.0 were never uploaded. Since 1.13.1 the build adds 1.14.0's DeepL engine, 1.15.0's operator
features (spending guard, markers, Find, the overlay notice, the
[system-wide keys](#what-app-review-may-question)), and 1.16.0's OpenAI subtitles, continuing a
cloud source with offline Whisper, the Gemini subtitles fix and the 20 November audit's fixes,
including the System Audio Recording hint that no longer returns after a pause in a call. None
of these has been tried on a Mac or in TestFlight yet. The listing's primary language is English
(U.K.). The description, review notes, marketing URL and subtitle below are the text submitted
with 1.16.0. The store build never checks for new versions; only the disk image can. The English
screenshots are in [store-screenshots/mac/en](store-screenshots/README.md#mac-app-store-english).
The 5 November upload is a 1.12-era build without those fixes: do not submit it. The decision of 4 November 2026: the Mac
App Store version is **free**, like the Windows one, with no in-app purchase. The trial and
unlock proposed in [#195](https://github.com/fmadore/Live-translation/issues/195) are dropped, so
no paid-apps agreement, StoreKit code or in-app product is needed.

What is ready in the repository:

- the sandboxed build: its entitlements (`src-tauri/macos/AppStore.plist`), the store's
  configuration (`src-tauri/tauri.appstore.conf.json`) and the container migration that moves
  the disk image's data folder into the sandbox (`src-tauri/macos/container-migration.plist`);
- a sandboxed variant built and started on every push by the *macOS test build* workflow, to
  try on a Mac before the store build exists;
- the *Mac App Store* workflow's `sandbox` job, which builds the app with the store's
  configuration, starts it in the sandbox and checks its container and the moved data folder,
  on pull requests that touch the store build and before every upload;
- the question before audio first goes to each cloud provider (Guideline 5.1.2(i)), on both
  systems ([decision](decisions/2026-11-05-mac-app-store.md));
- the *Mac App Store* workflow, which signs the app and its installer package and uploads them
  to App Store Connect;
- no support link in the store build (App Review Guideline 3.1.1; see `src-tauri/src/support.rs`);
- the privacy policy's macOS section and the provider question (effective 5 November 2026; the 16 November 2026 version added DeepL, the system-wide keys, markers and the disk image's check for new versions, and the policy now in force is effective 23 November 2026).

Every secret below and the `APPLE_APP_ID` variable were set on 5 November 2026, the provisioning
profile and the app's Apple ID last, after steps 1, 3 and 5. The workflow's upload job then ran
from `main` the same day and passed: the build is in App Store Connect. That build was not
submitted; 1.13.1 was, then was withdrawn after twelve days unanswered, and 1.16.0 (build 9) is
in App Review since 23 November 2026. What only the maintainer can do: answering App Review, and
a TestFlight run of the store build itself, which is also where the system-wide keys are first
tried in the sandbox. The one question that was open,
[system audio in the sandbox](#the-open-question-system-audio-in-the-sandbox), is answered.

## The open question: system audio in the sandbox

**Answered on 11 November 2026: taps work in the App Sandbox.** The sandbox spike (the store's
entitlements, ad hoc signed) captured system audio on an M2 Mac mini on macOS 26 once *System
Audio Recording* was allowed, downloaded a model into its container, reached Gemini and wrote
exports in place ([macOS notes](macos.md#first-run-1011-november-2026)). One-application capture
was tried with the direct build only. What follows is the question as it stood.

System audio and one-application capture use Core Audio process taps
(`src-tauri/src/audio/tap.rs`). Apple documents no entitlement for them and says nothing about
the App Sandbox; developers report both that taps work sandboxed with the microphone
entitlement and that they deliver silence there. One-application capture also reads other
processes' start times (`proc_pidinfo`), which the sandbox may refuse. No runner can grant
*System Audio Recording*, so only a Mac can tell. If taps fail in the store build:

- ship it with microphone capture only, saying so in the listing, and keep system audio in the
  disk image; or
- capture system audio through ScreenCaptureKit, which sandboxed apps use, at the cost of the
  broader *Screen Recording* permission.

## Once: at Apple

In [Certificates, Identifiers & Profiles](https://developer.apple.com/account/resources), with
the team `3HMSG8WB7K`:

1. **Identifier**: an explicit App ID for macOS, bundle ID `io.github.fmadore.live-translation`.
   No capability to tick: the sandbox, the microphone and file access are entitlements.
2. **Certificates**: the *Apple Distribution* and *Mac Installer Distribution* certificates made
   for Kalamu belong to the team and sign any of its apps, so their `.p12` files and passwords
   (in the password manager) are reused. Only if they are lost, make new ones as Kalamu's
   `docs/mac-app-store.md` describes (OpenSSL on Windows, `.p12` with SHA-1/3DES).
3. **Profile**: a *Mac App Store Connect* distribution profile for this App ID and the Apple
   Distribution certificate. Download it; it expires with the certificate.

In [App Store Connect](https://appstoreconnect.apple.com):

4. **API key**: the team key made for Kalamu (*App Manager* role) works for every app of the
   team; reuse its `.p8`, key ID and issuer ID.
5. **Apps → New App**: platform macOS, name `Live Translation & Subtitles`, primary language
   English (U.S.), bundle ID `io.github.fmadore.live-translation`, SKU `live-translation`, full
   access. Then **App Information → General → Apple ID**: the number the upload needs.

Free apps need no paid-apps agreement, tax or banking forms.

## Once: on GitHub

**Repository secrets** of this repository (Settings → Secrets and variables → Actions), the same
names and, apart from the profile, the same values as Kalamu's:

| Secret | Value |
| --- | --- |
| `APPLE_DISTRIBUTION_CERTIFICATE` | `distribution.p12`, base64 (`base64 -w0 distribution.p12`) |
| `APPLE_DISTRIBUTION_CERTIFICATE_PASSWORD` | its password |
| `APPLE_DISTRIBUTION_IDENTITY` | `Apple Distribution: Frédérick Madore (3HMSG8WB7K)` |
| `APPLE_INSTALLER_CERTIFICATE` | `installer.p12`, base64 |
| `APPLE_INSTALLER_CERTIFICATE_PASSWORD` | its password |
| `APPLE_INSTALLER_IDENTITY` | `3rd Party Mac Developer Installer: Frédérick Madore (3HMSG8WB7K)` |
| `APPLE_PROVISIONING_PROFILE` | **this app's** `.provisionprofile` from step 3, base64 |
| `APPLE_API_KEY_ID` | the API key's ID |
| `APPLE_API_ISSUER` | the issuer ID |
| `APPLE_API_KEY` | the `.p8` file's text |

**Repository variable** (same page, *Variables*): `APPLE_APP_ID`, the app's Apple ID from step 5.
The upload job stops at its first step, naming what is missing.

The disk image in each GitHub release needs the six Developer ID secrets listed in
[macos.md](macos.md#developer-id-secrets).

## Once: in App Store Connect

**Pricing and Availability**: free, all countries and regions.

**App Privacy → Data Types**: *No, we do not collect data from this app.* Apple counts data as
collected when it leaves the device in a way that lets the developer or its partners access it
beyond serving the request. The developer receives nothing: with Whisper or the built-in demo
nothing leaves the Mac; with a cloud engine, audio goes from the Mac to the provider the user
chose, under the user's own account and API key and that provider's terms; caption text goes to
Zoom only when the user starts sharing with a meeting. If App Review asks for the providers to be
declared anyway, the cautious answer is *Audio Data* (and *Other User Content* for Zoom
captions), not linked to the user, not used for tracking, purpose *App Functionality*.
**Privacy Policy URL**: `https://github.com/fmadore/Live-translation/blob/main/docs/privacy.md`.

**Age Rating**: answer *None* or *No* to every question: no violence, sexual content, profanity,
drugs, gambling, horror or medical content of the app's own; no unrestricted web access (the
app's only pages are its own; links open in the browser); no user-generated content shared with
others, no messaging or chat; no advertising; no location; no dating. The app captions and
translates speech the user chooses to capture and has no conversational assistant; say so where a
question touches generated content. The expected rating is 4+.

**Export compliance**: answered by the build. `src-tauri/Info.plist` declares
`ITSAppUsesNonExemptEncryption` false (HTTPS and the system's own encryption only).

**Content rights**: *No*: the app contains, shows or accesses no third-party content of its own.

## The listing (English, U.K., as submitted with 1.16.0)

Every paragraph is one unwrapped line for pasting. Screenshots must show the Mac app
(Guideline 2.3.3), at one of App Store Connect's Mac sizes (1280 × 800, 1440 × 900, 2560 × 1600
or 2880 × 1800), with no alpha channel. The current set, six images at 1280 × 800 from the
sandboxed build, is in [store-screenshots/mac/en](store-screenshots/README.md#mac-app-store-english).

**Name** (30 characters at most): `Live Translation & Subtitles`

**Subtitle** (30): `Live captions over your slides`

**Category**: Productivity; secondary: Education.

**Keywords** (100 characters, commas, no spaces):

```text
captions,subtitles,translate,transcribe,whisper,offline,overlay,meeting,lecture,zoom,accessibility
```

**Promotional text** (170):

```text
Live captions over your slides, calls and videos. Transcribe offline with Whisper on your Mac's graphics processor, or translate with your own Gemini or OpenAI key.
```

**Description**:

```text
Put live captions over any meeting, lecture, video call or slide deck. Live Translation & Subtitles transcribes speech on your own Mac with Whisper: download a model once, and captions run offline with no account or subscription. Whisper can also translate into English offline. When your audience needs another language, add cloud translation with your own Google Gemini, OpenAI or DeepL API key; provider usage charges may apply, and DeepL needs a paid DeepL API plan.

Captions appear in a transparent overlay that floats above your slides, your call or a full-screen presentation, in every Space, and lets clicks pass through to the window beneath. Choose what to listen to: a room microphone, the sound your Mac plays, a single app, or the microphone and system audio together. In a Zoom meeting, you can also share one caption language with Zoom's own caption bar, using the caption link the host copies in Zoom.

For offline subtitles, pick a multilingual Whisper model and one of 99 spoken languages, or let the app detect the language. The Tiny, Base, Small and Large v3 Turbo models download separately, at about 31, 57, 182 and 547 MiB. Once a model is installed, recognition runs on your Mac's graphics processor with no account, API key or internet connection, and built-in speech detection skips silence and music, so Whisper does not invent captions over a quiet room. Whisper captions in the language being spoken, or translates it into English (with Small; Large v3 Turbo only transcribes); for any other language, use cloud translation: Gemini into 78 languages, DeepL into 42 or OpenAI into 13, with two languages on screen at once if you like. DeepL is experimental. Gemini, Mistral and OpenAI also offer cloud subtitles in the spoken language with your own key, and a list of names and terms helps them and Whisper spell speakers' names and acronyms. If a cloud engine stops during a session, the app offers to carry on with offline Whisper in the same session.

Tune captions for the room: typeface, size, colours, backing, how long lines stay on screen and how quickly they update. Save meeting profiles for the rooms you use often, and pause during breaks, with an optional notice such as "Back soon" on the overlay. Optional system-wide keys pause captions and show or hide the overlay while your slides or meeting app are in front. With a cloud engine, an optional spending guard pauses a quiet session and tells you when the estimated cost passes a limit you set.

During a session, find words in the transcript and add markers for a talk's start or a speaker's name. Copy the transcript, export it as text, Markdown, SRT or WebVTT, and keep an optional local history of past sessions.

With Whisper, audio stays on your Mac. With a cloud engine, audio goes straight to the provider you chose, and the app asks before it first sends audio to each provider. API keys are kept in the macOS Keychain. The developer receives no audio, keys, transcripts or telemetry, and the app is free, with nothing to buy.

Use the interface in English, French or German, whatever the caption language. An optional English/French demo lets you try the display without any setup; it plays a prepared script and does not recognize speech.

Requires a Mac with Apple silicon and macOS 15 or later. macOS asks for access to the microphone and to System Audio Recording the first time you use them; no screen content is recorded. Capturing a single app also captures its helper processes and notifications. Sharing with Zoom needs a meeting whose host has turned on manual captions and allowed the caption API token for third-party services.

Zoom is a trademark of Zoom Communications, Inc. Live Translation & Subtitles is not affiliated with or endorsed by Zoom.
```

**Support URL**: `https://github.com/fmadore/Live-translation/issues`. **Marketing URL**:
`https://fmadore.github.io/Live-translation/` (since 1.16.0; the GitHub repository before).

## Review notes

Paste into *App Review Information → Notes*. A test key with a low spending cap may go in the
review fields, never in the listing.

```text
Live Translation & Subtitles puts live captions of speech over other apps. It is free, with no in-app purchase and no account.

Real speech recognition without any account or API key:
1. Open the app. In Subtitles, the engine Local Whisper is selected. Download the Tiny (about 31 MiB) or Base (about 57 MiB) model with the Download button; the model is data for the speech recognizer already in the app, downloaded from Hugging Face over HTTPS and checked against its SHA-256 before use. No code is downloaded. Recognition runs on the Mac's graphics processor through Metal (the Use the graphics processor switch under the model moves it to the processor), and a speech-detection model bundled in the app skips silence before recognition.
2. Choose System audio as the source and start. macOS asks for System Audio Recording (the app records sound only, never the screen): allow it. Play any speech on the Mac, for example a video in Safari, or the bundled sample at /Applications/Live Translation.app/Contents/Resources/fixtures/rehearsal-en.wav. Captions appear in the operator window and in the transparent overlay.
3. Or choose a microphone as the source: macOS asks for microphone access.
4. Stop, then export the transcript as text, Markdown, SRT or WebVTT with the Save panel.

Setup-free check without audio: choose the engine Built-in demo and click Start demo subtitles. It plays a prepared English or French script through the overlay, timer, level meter and export, without capturing or recognizing any audio, and says so on screen.

Optional cloud engines (Google Gemini, OpenAI and DeepL for translation; Gemini, Mistral and OpenAI for subtitles) use the user's own API key, entered in the app and kept in the macOS Keychain; audio then goes directly from the Mac to that provider, which may charge the user. DeepL, marked experimental in the app, needs a paid DeepL API plan; a free DeepL key (ending in :fx) is refused with an explanation and nothing is sent. Before the first session with each provider, the app asks for permission, naming the provider and what is sent, with the privacy policy's address; nothing is sent on Cancel. If a cloud engine ends a source during a session (a key, quota or network error), a banner offers to continue that source with offline Whisper; it never switches on its own and sends nothing new. The app sells no access and the developer receives nothing.

Optional system-wide keys (Settings > App, off by default): Control-Option-Shift-P pauses captions and Control-Option-Shift-O shows or hides the overlay. They are registered with Carbon's RegisterEventHotKey only while a session runs, need no Accessibility or Input Monitoring permission, and the app is told of these two combinations only, nothing else typed.

The overlay uses public AppKit only: a borderless transparent window at a high window level that joins every Space and full-screen apps and ignores mouse events while captions run. This store build never checks for updates and shows no support or donation link.
```

## Each version

1. **Release it as usual** (the release workflow builds the Windows packages and the notarised
   disk image). The store build reads the version from `src-tauri/tauri.conf.json`.
2. **Upload**: Actions → *Mac App Store* → *Run workflow*, on the tag, with *upload* ticked.
   It builds the sandboxed app and checks that it starts in its container, then signs it and
   the package, validates and uploads. The build number is the workflow's run
   number, which always rises, as App Store Connect requires.
3. **Wait for processing**: the build appears under TestFlight within about an hour, or App Store
   Connect emails the reason it was refused.
4. **TestFlight on a real Mac** (the first time at least), with the sandbox part of the checklist
   in [macos.md](macos.md#part-b-the-sandboxed-app), plus: the data folder moved from the disk
   image's version, and no support link.
5. **The version page**: choose the build, paste the listing and the review notes, add the
   screenshots, and choose *Manually release this version* for the first one.
6. **Submit for review**, release once approved, and note the store link in the README, the
   roadmap and the release notes.

## What App Review may question

- **Downloaded models (2.4.5(iv), 2.5.2).** Whisper models are downloaded from Settings: weights
  for the recognizer in the reviewed build, not code. The review notes say so. If refused, the
  fallback is to bundle the Tiny model in the store build. The speech-detection model (Silero
  VAD, 864 KiB) is already bundled, in `Contents/Resources/models/`.
- **Bring-your-own keys and third-party AI (5.1.2(i)).** Before the first session or rehearsal
  with each provider, the app asks, names the provider and what is sent, and gives the privacy
  policy's address, as Kalamu does; the review notes point to it.
- **The overlay (2.4.5, 4).** Public AppKit only; it never takes focus while captions run.
- **System-wide shortcuts (2.4.5, 5.1.1).** The opt-in Pause and Show/hide overlay keys (⌃⌥⇧P,
  ⌃⌥⇧O; off by default, held only while a session runs) use Carbon's `RegisterEventHotKey`
  through `tauri-plugin-global-shortcut` 2.4 and `global-hotkey` 0.8, read in their source
  (`global-hotkey`'s `platform_impl/macos`): an application event handler and a registered hot
  key, which needs no entitlement, no Accessibility and no Input Monitoring permission, and is
  available in the App Sandbox. The crate's `CGEventTap`, which would need Input Monitoring, is
  used only for media keys, which the app never registers. Not yet tried in the sandboxed
  build: check on the first TestFlight build that the keys work and that macOS asks nothing.
- **Screenshots (2.3.3)** must show the Mac app.
- **Privacy manifest** (`PrivacyInfo.xcprivacy`): not checked whether App Store Connect wants one
  for this app; the upload's validation names it if so.
