# 2026-11-05 · Mac App Store: what was checked, and the two pieces added

**Status:** decided. Completes the store build of
[#202](https://github.com/fmadore/Live-translation/pull/202) before its first upload.

## Context

The macOS port (#195, [macos.md](../macos.md)) already holds a Mac App Store build modelled on
Kalamu's, which went through the same steps with the same Apple team (`3HMSG8WB7K`) on
4 November 2026: the store's entitlements (`src-tauri/macos/AppStore.plist`), configuration
(`src-tauri/tauri.appstore.conf.json`), container migration, `ITSAppUsesNonExemptEncryption`
false in `src-tauri/Info.plist`, an `.icns` icon with its 1024 × 1024 image, and an upload job
the same as Kalamu's. This record compares it with Kalamu's finished setup (its
`docs/mac-app-store.md`, `.github/workflows/app-store.yml` and decision records of
4 November 2026) and checks what the App Sandbox changes for this app. Nothing here has run on a
Mac.

## What the sandbox changes

| What | Under the sandbox |
| --- | --- |
| Microphone | `com.apple.security.device.audio-input` and `NSMicrophoneUsageDescription`: in place. |
| System audio and one-application capture | Core Audio process taps. **Unverified**: no entitlement is documented and reports disagree (some say taps work sandboxed, others that they deliver silence). One-application capture also reads other processes' start times through `proc_pidinfo`, which the sandbox may refuse. Only a Mac with *System Audio Recording* granted can tell. See [mac-app-store.md](../mac-app-store.md#the-open-question-system-audio-in-the-sandbox) for the two fallbacks. |
| Caption overlay | Public AppKit window levels and Spaces behaviour: no entitlement. |
| Cloud providers, Zoom, model downloads | `com.apple.security.network.client`: in place. |
| API keys | The login Keychain, for the app's own items. Keys saved by the disk image belong, for the Keychain, to another app: expect one more prompt or re-entering them. |
| Whisper models, history, recovery copy | The data folder lands in the container through `HOME`; the disk image's folder is moved in on first launch by `container-migration.plist`. |
| Transcript export | The Save panel's file (`files.user-selected.read-write`), with a direct write when the atomic rename is refused. |
| Bundled programs | None: Whisper is linked into the app, so Kalamu's step re-signing FFmpeg to inherit the sandbox has no counterpart. |
| Support link | Hidden in the sandboxed build (Guideline 3.1.1). |

## Decision

Two pieces of Kalamu's setup were missing; both are added.

1. **A sandbox check before every upload.** The *Mac App Store* workflow gains a `sandbox` job,
   run on pull requests that touch the store build and before `upload`, which now runs only on
   demand with *upload* ticked. It builds with the store's configuration (the *macOS test
   build*'s sandboxed variant re-signs the direct build, so it never carried the migration
   manifest), signs ad hoc with `Sandbox.plist`, leaves a disk image's data folder behind,
   starts the app with `open`, and fails unless the app is still running, has made its data
   folder in the container, and the old folder was moved there.
2. **A question before audio first goes to each cloud provider** (Guideline 5.1.2(i), which asks
   for explicit permission before personal data is shared with third-party AI; Kalamu asks the
   same way). On both systems, since the Microsoft Store version sends the same audio:
   - once per company (`Gemini`, `OpenAI`, `Mistral`), so the two Gemini engines share one
     answer; local Whisper and the built-in demo send nothing and are never asked about;
   - asked by the launch path that Start, Rehearse and the start shortcut share
     (`LaunchController.launch`), so none of them skips it; a rehearsal sends the sample
     recording to the provider, so it asks too;
   - in the app's own modal (`ProviderConsentPrompt.svelte`), in the interface's language: it
     names the provider, says the audio goes to it with whatever is said, under the user's
     account, key and the provider's terms, that the developer receives nothing, and gives the
     privacy policy's address. *Send* records the answer and starts; *Cancel* and Escape start
     nothing;
   - kept in the webview's storage with the other preferences (`privacy.aiConsent`,
     `src/lib/aiConsent.ts`), only ever added to. Removing a provider's key stops anything being
     sent to it.

   Unlike Kalamu, the core does not enforce it: a session starts only from the interface, and
   the start command carries no provider choice the interface did not make.

## Consequences

- Existing installs are asked once at their next cloud session with each provider.
- The privacy policy (effective 5 November 2026), the store guide and its review notes describe
  the question.
- The style tests answer it on their way to a running session.
- The first TestFlight run must try system audio and one-application capture in the store
  build before anything is submitted for review.
