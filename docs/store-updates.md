# Microsoft Store updates

Every update to this app is submitted **by hand** through Partner Center. That is not a
preference and not a stopgap — it is the only route this account has, for a reason Microsoft
enforces and we cannot work around. [Why by hand](#why-by-hand) has the detail; the short
version is that the submission API is a Company-account feature and this is an Individual
account.

The route below is what shipped every version so far. It takes about five minutes once the
packages are built.

## Release 1.14.0 handoff

**Prepared; not yet tagged or submitted** (tag planned for 13 November 2026). The Store serves
**1.13.1.0**, read with `Get-AppxPackage 49346FMadore.LiveTranslationSubtitles` on the
maintainer's ARM64 PC on 13 November 2026 (Version 1.13.1.0, SignatureKind Store); the
submission and certification dates were not recorded ([1.13.1's handoff](#release-1131-handoff)).
Target app version: **1.14.0**, MSIX **1.14.0.0**, in `Live.Translation_1.14.0.msixbundle`, with
`Live.Translation_1.14.0_aarch64.dmg` for the Mac. What's new and the certification notes' first
paragraph cover the changes since 1.13.1; the descriptions, the fourth feature line and the
certification notes' *Optional cloud features* add DeepL ([store-listing.md](store-listing.md),
[what changed](store-listing-next.md#what-1140-adds)).

Since `v1.13.1`:

- [#220](https://github.com/fmadore/Live-translation/pull/220): DeepL (DeepL Voice API, DeepL SE)
  as a fourth cloud engine, for live translation only, into 42 caption languages
  ([DeepL notes](deepl-voice-api.md)). Voice is on DeepL's paid API only: a free key (ending in
  `:fx`) is refused when saved and again at Start (`error.deeplPaidPlan`), with no request made.
  Marked **experimental** in the engine picker: built from DeepL's documentation and tested
  against its documented messages and loopback servers, but never run against the live API. The
  shared realtime runner gains three hooks (a preparation step before each connection, an
  optional setup message, a keepalive), no-ops for Gemini, OpenAI and Mistral, whose wire output
  is unchanged. Privacy policy effective 11 November 2026, naming DeepL.
- `1d9a5b6`: Mistral's "Timeout waiting for response from streaming transcription" (code 3804),
  seen in a live stream on 11 November 2026, now reconnects like 3803 instead of stopping the
  source.
- [#221](https://github.com/fmadore/Live-translation/pull/221): with both sources, an overlay
  strip sized for one source (160 px at 150 %, font 52) left each row too shallow for one line,
  so the Room and Remote labels showed over empty rows; rows are now drawn at the largest size
  that fits, down to 20 px. Reported from a real Zoom meeting with Mistral subtitles. The release
  PR is merged after #221.

**Changed under the surface:** no new command or permission; the CSP (`tauri.conf.json`), both
windows' capabilities and `command_names.rs` are unchanged since `v1.13.1`; DeepL goes through
the existing session and key commands as a new `Provider` value (`deepl`, on both sides of the
contract). One new error id, `error.deeplPaidPlan`, in `errors.rs` and the three catalogs. One
new credential, `deepl-api-key`, in Credential Manager or the Keychain. No new Rust or npm
dependency (`Cargo.lock` and `package-lock.json` change only in the version). The MSIX template,
`bundle.config.json` and the package's contents are unchanged. The
[privacy policy](privacy.md) changed in #220 (effective 11 November 2026, DeepL named as a
recipient).

### Preparation

- [x] #220 merged; `1d9a5b6` on `main`.
- [ ] #221 merged, then the release PR, which carries the version bump in the seven files of
  [the route](#the-route) and `CITATION.cff` (1.14.0, 13 November 2026) and
  `docs/release-1.14.0.md`.
- [ ] The release PR's CI passes, the macOS lane included (nothing in it is platform-specific,
  and none of it has been tried on a Mac).
- [x] **DeepL:** the maintainer decided on 13 November 2026 to submit 1.14.0 with DeepL untested
  against the live API and labelled experimental, as the app and the copy say. A live session
  with a key from a paid DeepL API plan (the ignored `live_probe`, then real speech in the app)
  is still owed, checking what [the DeepL notes](deepl-voice-api.md) list as unverified.
- [ ] **Overlay:** both sources in a 160 px strip on the installed package: each row shows a
  line in Fit window, Compact and Stable reading.
- [ ] The French and German sentences new in 1.14.0 (What's new, the descriptions' DeepL
  sentences, the fourth feature line, the certification notes) reviewed by the maintainer; the
  German listing still awaits a native speaker's review.
- [ ] Partner Center limits: What's new EN 588, FR 709, DE 718 of 1,500 characters (counted in
  [store-listing.md](store-listing.md)); feature line 4 EN 130, FR 144, DE 153 of 200; the
  certification notes' first paragraph is 1,009 characters, with no limit recorded.

### Release and Store acceptance

- [ ] Tag `v1.14.0` (annotated: "Live Translation & Subtitles v1.14.0") on the merge of the release
  PR and push it; the workflow publishes the release with `docs/release-1.14.0.md` once the bundle
  and the disk image are attached.
- [ ] Verify the bundle at **1.14.0.0** with the [1.8.0 checks](#release-180-handoff), plus
  `whisper-vulkan.exe` and `models/ggml-silero-v5.1.2.bin` in both packages; the disk image signed
  and stapled (`spctl -a -t open --context context:primary-signature -v`, `xcrun stapler validate`).
- [ ] The open checks of the [1.13.1](#release-1131-handoff) and [1.13.0](#release-1130-handoff)
  handoffs (maintainer test on Windows, WACK, sideload and accessibility checks, the Store
  screenshots decision) apply to 1.14.0.
- [ ] The published privacy page reads "11 November 2026" and names DeepL.
- [ ] Submit `Live.Translation_1.14.0.msixbundle` in Partner Center with the 1.14.0 copy
  ([walkthrough](partner-center-walkthrough.md)); rollout **gradual**, starting at 10%.
- [ ] **Mac:** the [Mac App Store steps](mac-app-store.md#each-version) with the 1.14.0 build from
  its tag, as the version after 1.13.1, which is in App Review (13 November 2026).

## Release 1.13.1 handoff

**Tagged and published on GitHub on 11 November 2026; live in the Store** as 1.13.1.0, read with
`Get-AppxPackage` on the maintainer's ARM64 PC on 13 November 2026 (SignatureKind Store). The
submission and certification dates and the rollout were not recorded; the checks below that
stay unticked were not recorded as done and stay open against the Store build. 1.14.0 follows it
([handoff](#release-1140-handoff)). 1.13.0 was tagged and published on GitHub on 10 November 2026
and **never submitted**; 1.13.1 replaced it, as 1.7.1 once replaced 1.7.0, and replaced 1.12.0.0
in the Store. Target app version: **1.13.1**, MSIX **1.13.1.0**, in
`Live.Translation_1.13.1.msixbundle`, with `Live.Translation_1.13.1_aarch64.dmg` for the Mac.
What's new and the certification notes still cover everything since 1.12.0, now with one
sentence on 1.13.1's own fixes ([store-listing.md](store-listing.md),
[what changed](store-listing-next.md#what-1131-adds)).

1.13.1 is the Mac app's first run on a real Mac (an M2 Mac mini on macOS 26.7.3, direct and
sandboxed builds; [macOS notes](macos.md#first-run-1011-november-2026)) and its fixes:

- [#218](https://github.com/fmadore/Live-translation/pull/218): Stop no longer hangs while the
  System Audio Recording question is unanswered, and on Windows too a local Whisper source no
  longer waits on a capture thread Stop has given up on (`ActiveSession::abandoned`); a refused
  permission is flagged while something plays (`AudioLevel.silenced`, new optional field on both
  sides of the contract); no microphone, a closed application and a disconnected output each get
  their own error id (`error.micNone`, `error.applicationClosed`, `error.outputDisconnected`), on
  both systems; no single-instance plugin in the App Sandbox; RAUMMIKROFON on one line in
  WebKit; the hidden date input no longer overflows; README: CMake for a Mac build.
- The overlay as a non-activating panel on macOS, so it stays over full-screen apps; the audio
  test's macOS wording; the app's own tap devices left out of device lists. Gemini was tried in
  both Mac builds.
- [#214](https://github.com/fmadore/Live-translation/pull/214): two npm dev-dependency updates.
- The Mac App Store screenshots, English, in
  [store-screenshots/mac/en](store-screenshots/README.md#mac-app-store-english).

**Changed under the surface:** no new command or permission; the CSP and both windows'
capabilities are unchanged since `v1.13.0`. Three new error ids in `errors.rs` and the catalogs.
The MSIX template, `bundle.config.json` and the package's contents are unchanged. For macOS
only, `objc2-app-kit` gains its `NSPanel` feature. The [privacy policy](privacy.md) needs no
change: nothing new is sent or kept.

### Preparation

- [x] #218 merged; the overlay, wording and device-list fixes and the screenshots in the release
  PR, which carries the version bump in the seven files of [the route](#the-route) and
  `CITATION.cff` (1.13.1, 11 November 2026) and `docs/release-1.13.1.md`.
- [ ] The release PR's CI passes, the Windows lanes included (the typed capture errors touch
  `windows_impl` and `applications.rs`, which the branch could not compile on a Mac).
- [ ] The French and German sentences new in 1.13.1 (What's new, the certification notes, the
  macOS strings in `i18n/mac.ts`) reviewed by the maintainer.
- [ ] Partner Center limits: What's new EN 1,175, FR 1,354, DE 1,330 of 1,500 characters
  (counted in [store-listing.md](store-listing.md)); the certification notes' first paragraph
  is 1,411 characters, with no limit recorded.

### Release and Store acceptance

- [x] Tag `v1.13.1` (annotated: "Live Translation & Subtitles v1.13.1") on the merge of the release
  PR and push it; the workflow publishes the release with `docs/release-1.13.1.md` once the bundle
  and the disk image are attached. Tagged 11 November 2026; the release was published the same
  day with the bundle and the disk image.
- [ ] Verify the bundle at **1.13.1.0** with the [1.8.0 checks](#release-180-handoff), plus
  `whisper-vulkan.exe` and `models/ggml-silero-v5.1.2.bin` in both packages; the disk image signed
  and stapled (`spctl -a -t open --context context:primary-signature -v`, `xcrun stapler validate`).
- [ ] The open checks of the [1.13.0 handoff](#release-1130-handoff) (maintainer test on Windows,
  WACK, sideload and accessibility checks, the Store screenshots decision) apply to 1.13.1.
- [ ] Submit `Live.Translation_1.13.1.msixbundle` in Partner Center with the 1.13.1 copy
  ([walkthrough](partner-center-walkthrough.md)); rollout **gradual**, starting at 10%. It was
  submitted and is live (read 13 November 2026); the date and the rollout were not recorded.
- [x] **Mac:** the [Mac App Store steps](mac-app-store.md#each-version) with the 1.13.1 build from
  its tag and the screenshots in `store-screenshots/mac/en`; not the 5 November upload. Submitted,
  and in App Review on 13 November 2026 (the maintainer); the dates were not recorded.

## Release 1.13.0 handoff

**Tagged and published on GitHub on 10 November 2026; never submitted: 1.13.1 replaces it
([handoff](#release-1131-handoff)).** Its open checks carry over to 1.13.1. The Store then served **1.12.0.0**, live as reported by
the maintainer on 10 November 2026; the submission and certification dates were not recorded
([1.12.0's handoff](#release-1120-handoff)). Target app version: **1.13.0**, MSIX **1.13.0.0**,
in `Live.Translation_1.13.0.msixbundle`, with `Live.Translation_1.13.0_aarch64.dmg` for the Mac.
What's new and the certification notes cover the changes since 1.12.0. What 1.12.0 brought to
the listing (the Zoom privacy section, the trademark line, the certification notes' Zoom
paragraph) is already in the Store and stays.

1.13.0 adds the Mac app, a faster and quieter local Whisper, and the fixes of a review:

- [#202](https://github.com/fmadore/Live-translation/pull/202) (for
  [#195](https://github.com/fmadore/Live-translation/issues/195)): **the macOS app** for Apple
  silicon, macOS 15 and later, with Core Audio taps for system audio and one application, the
  Keychain, the menu bar, macOS's words in all three languages and ⌘ shortcuts; a notarised disk
  image built by the release workflow, and a free Mac App Store build uploaded by its own
  workflow. In the same PR: **the question asked once for each cloud provider** before audio is
  first sent to it, on both systems. [#204](https://github.com/fmadore/Live-translation/pull/204)
  moved the macOS jobs to macOS 26 with Xcode 26. Not run on a real Mac
  ([macOS notes](macos.md)).
- [#208](https://github.com/fmadore/Live-translation/pull/208): **Whisper on the graphics
  processor** (Metal on a Mac; on Windows, Vulkan in the `whisper-vulkan.exe` helper, with
  Snapdragon PCs kept on the processor), **bundled speech detection** (Silero VAD), the **loop
  and credit filters** and **Large v3 Turbo** (547 MiB, transcription only), with *Use the
  graphics processor* under the model ([local Whisper](local-whisper.md#graphics-processor)).
- [#193](https://github.com/fmadore/Live-translation/pull/193): the **names-and-terms list** also
  goes to Gemini Transcribe (`customVocabulary`) and to local Whisper as its decoder prompt, cut
  to the terms that fit.
- [#212](https://github.com/fmadore/Live-translation/pull/212): with Both sources, **Leave out
  the call heard by the room mic**, off by default; the setup says where the latest Whisper
  session decoded and why the graphics processor could not be used.
- [#213](https://github.com/fmadore/Live-translation/pull/213), open, from this branch: the
  fixes of the [10 November app review](app-review-2026-11-10.md) (B1–B19, A1, A2, V1–V4,
  E1–E5, S1–S9, C2–C7), including no rehearsal captions in a Zoom share, close frames read and
  reported, a Start that cannot be pressed no longer looking pressable, and the release
  workflow's CI gate, `msix` without `build` and one compiler action.
- [#211](https://github.com/fmadore/Live-translation/pull/211): the Rust crates on the 2024
  edition, and `AGENTS.md`. [#210](https://github.com/fmadore/Live-translation/pull/210): daily
  download and visit statistics on the `stats` branch (repository only).
- Dependabot: [#196](https://github.com/fmadore/Live-translation/pull/196)–[#200](https://github.com/fmadore/Live-translation/pull/200),
  [#205](https://github.com/fmadore/Live-translation/pull/205)–[#207](https://github.com/fmadore/Live-translation/pull/207)
  (tokio 1.54.1 and other patch releases), and the npm overrides of minimatch and glob
  ([#201](https://github.com/fmadore/Live-translation/pull/201)). The review's C7 brought wasapi
  0.27, rfd 0.18 and sha2 0.11.

**Changed under the surface:**
- **One new operator command, `support_link_allowed`** (`src-tauri/src/support.rs`), in
  `command_names.rs` and `capabilities/operator.json`: it answers false only in the Mac App
  Store build, which hides the support link. The overlay's capability is unchanged since
  `v1.12.0`, and the CSP is unchanged. One new event, `whisper-engine`, tells the operator
  window where each Whisper source decodes.
- **New files in the Windows packages and installers:** `whisper-vulkan.exe` beside
  `live-translation.exe`, declared as a sidecar in `tauri.vulkan.conf.json`, which only release
  and MSIX builds lay in; and `models/ggml-silero-v5.1.2.bin` (885,098 bytes, 864 KiB), through
  the new `resources/models/*` entry in `tauri.conf.json`. The release workflow's `msix` job
  refuses a package that lacks either. The helper is started only by the app, put in a job
  object that ends it with the app, reads audio only through standard input, and opens no
  network connection.
- **The package still declares only `runFullTrust` and `microphone`:**
  `AppxManifest.xml.template` and `bundle.config.json` are unchanged since `v1.12.0` (and the
  template since `v1.11.0`).
- **Cargo:** `src-tauri` is a workspace with `whisper-vulkan` as its second package (whisper-rs
  with its `vulkan` feature, serde, serde_json and sha2, all already in the app's graph); the
  edition is 2024. For Windows no crate was added: wasapi 0.26 → 0.27, rfd 0.17.2 → 0.18.0 and
  sha2 0.10 → 0.11, and `windows-sys` gained the `Win32_Security` and
  `Win32_System_JobObjects` features for the job object. For macOS only: whisper-rs with
  `metal`, libc and the `objc2` crates for AppKit and Core Audio.
- **New macOS configuration** (no effect on Windows): `tauri.macos.conf.json`,
  `tauri.appstore.conf.json` and the entitlements under `src-tauri/macos/`.
- **The release workflow** now builds the Vulkan helper, the signed and notarised disk image in
  a `macos` job that `bundle` waits on, and the package-content check (#202, #208); #213 adds
  the CI gate on the tagged commit, `msix` without `build`, the shared compiler action and the
  helper's version in `prepare`. **None of these has run on a tag yet.**
- **New for the Store:** a second executable and a bundled model in the package, and a question
  before audio first goes to a cloud provider, which the certification notes describe. See
  [store-listing-next.md](store-listing-next.md).

The [privacy policy](privacy.md) needs no change for 1.13.0. Its effective date is 5 November
2026 and it already covers what 1.13.0 sends and keeps: the macOS app (Keychain, *System Audio
Recording*, the Mac App Store sandbox and container), the per-provider question and the answer
kept on the device, names and terms sent to Gemini subtitles and kept local with Whisper, and
the Vulkan helper, reached only through the computer's own memory. Its *Preferences* paragraph
still says "the names and terms entered for Mistral subtitles", which remains true but no longer
names the other engines: an editorial change for the maintainer, without a new date.
[GitHub release copy](release-1.13.0.md) · [EN/FR/DE Store fields](store-listing.md) ·
[Partner Center walkthrough](partner-center-walkthrough.md) ·
[Mac App Store steps](mac-app-store.md).

### Preparation and validation

- [x] #193, #202, #204, #208, #211 and #212 merged, with the dependency updates #196–#201 and
  #205–#207.
- [ ] [#213](https://github.com/fmadore/Live-translation/pull/213) (`fix/review-2026-11-10`, the
  10 November review's fixes and these docs) reviewed and merged after all its CI jobs pass,
  including the macOS lanes and the helper's, which the branch could not compile locally.
- [x] 1.12.0's Store status recorded: live as 1.12.0.0, reported by the maintainer on
  10 November 2026, in [1.12.0's row](#release-history) and section below, and in README,
  ROADMAP and docs/README.
- [x] **Version bump** in the release PR, in the seven files of [the route](#the-route):
  `package.json`, `src-tauri/Cargo.toml`, `src-tauri/whisper-vulkan/Cargo.toml` and
  `src-tauri/tauri.conf.json`; the `live-translation` and `whisper-vulkan` entries of
  `src-tauri/Cargo.lock`, the root entries of `package-lock.json`; and `version` and
  `date-released` in `CITATION.cff`, set to 1.13.0 and the tag date. `npm test` checks that they
  agree. Commit as `chore(release): 1.13.0` with `docs/release-1.13.0.md`, and replace "Not yet
  tagged" in [release-notes.md](release-notes.md) with the tag date. Done on 10 November 2026,
  the tag date.
- [x] Lockfiles: nothing changes besides the root version lines (`git diff --text`).
- [ ] Validation of the release branch: `npm run test:coverage`, `check`, `lint`, `knip`,
  `format:check`, `check:languages`, `build` and `test:style`; `cargo fmt --check`, Clippy with
  `-D warnings` and `cargo test --locked`. Record the counts here.
- [ ] **Validation tag, before the real tag.** The release workflow's macOS disk image, Vulkan
  helper build, package-content check, CI gate on the tagged commit, `msix` without `build` and
  shared compiler action have never run in a release (review item C1). Push a hyphenated tag
  such as `v1.13.0-validation.1` on a commit that carries the 1.13.0 version and that CI has
  passed on (the release PR's head): `prepare` accepts `v<version>-<suffix>` and the workflow
  builds every asset into a **draft prerelease that is never published**. This is a one-off
  rehearsal of the workflow for this release, the maintainer's decision; it is not a release
  candidate of the app, and the convention of tagging `vX.Y.Z` straight from the release merge
  stands. Check that every job passes, that both MSIX packages list `whisper-vulkan.exe` and
  `models/ggml-silero-v5.1.2.bin`, and that the disk image is attached, signed and stapled
  (`spctl -a -t open --context context:primary-signature -v` and `xcrun stapler validate` on a
  Mac, if one is at hand). Then delete the draft and the tag:
  `gh release delete v1.13.0-validation.1 --yes`, `git push origin --delete
  v1.13.0-validation.1` and `git tag -d v1.13.0-validation.1`. Not done: the maintainer
  asked for `v1.13.0` to be tagged directly on 10 November 2026, so these steps run for the
  first time on the release itself; a job that fails leaves the release a draft.
- [ ] The Store fields are within Partner Center's limits. Counted when drafted (characters in
  each block, line breaks included), not yet checked in Partner Center: What's new EN 1,043,
  FR 1,200, DE 1,177 of 1,500; 20 of 20 features, the longest 135 of 200 (FR); descriptions
  EN 5,050, FR 6,071, DE 5,621 of 10,000; short descriptions 264, 268 and 265 of 270; additional
  requirements, the longest 170 of 200; certification notes 8,488 (no limit is recorded in these
  documents). French What's new is the closest to its limit.
- [ ] The French and German sentences new in 1.13.0 (What's new, the offline-subtitles
  paragraph, names and terms, the question before audio is sent, the feature lines and the
  requirements line) reviewed by the maintainer.
- [ ] Update the version-bound guides in the release PR, as 1.12.0's did:
  [partner-center-walkthrough.md](partner-center-walkthrough.md),
  [microsoft-store.md](microsoft-store.md) and [packaging-msix.md](packaging-msix.md) (prepared
  for 1.13.0 on this branch; check them against the merge); README, ROADMAP and docs/README once
  tagged.

### Testing

- [ ] **Maintainer test** of the side-by-side Local Test build of the release merge (ARM64, and
  x64 where a PC is at hand), **before the tag**, from the
  [10 November review's hand checks](app-review-2026-11-10.md#10-notes-from-doing-it) and the
  open points of [local Whisper](local-whisper.md#graphics-processor):
  - **Graphics processor:** on a PC with a usable card, the setup and the log name it
    (`Whisper model loaded device=vulkan: …`); with *Use the graphics processor* off, `cpu`; on
    a Snapdragon, the processor. Time the same speech with the switch on and off (speed has not
    been measured). A window that fails is retried once, then the session moves to the
    processor; quitting during a hung model load does not wait on it; the stage and the setup
    say why the graphics processor could not be used.
  - **Speech detection and filters:** a quiet room, a paused call and music produce no
    invented captions; speech after silence is captioned at its real time; a word said over
    and over, and stray subtitle credits, are removed.
  - **Large v3 Turbo:** downloads, verifies and transcribes real speech, keeping up on the
    graphics processor or a fast processor; translation does not offer it.
  - **Names and terms:** with Gemini Transcribe, the list's spellings appear; with Whisper, the
    note under the list, a long list cut to what fits, and no caption that only reads the list
    back over noise.
  - **The question before audio is sent:** on the first session and on the first rehearsal with
    Gemini, OpenAI and Mistral, in English, French and German; Cancel has the focus and starts
    nothing; one answer covers both Gemini engines; Whisper and the demo are never asked about;
    Narrator reads it with Cancel focused.
  - **Both sources:** *Leave out the call heard by the room mic* in a real room with loudspeakers,
    with a cloud engine and with local Whisper under a backlog; whether the brief interim of a
    held line is noticeable; how long local speech over the loudspeaker waits; that the meter's
    session times line up with caption times. Off, captions are as in 1.12.0.
  - **Zoom:** a rehearsal with sharing on posts nothing in a real meeting.
  - **Cloud engines:** how each provider's real mid-stream close codes are classified, and the
    reason shown while reconnecting.
  - **Audio devices:** a Bluetooth endpoint no longer causes repeated checks; renaming a device
    in Sound settings still refreshes the lists; system capture on wasapi 0.27 on x64 and ARM64.
  - **Session:** Start, then Rehearse, after a run whose sources all failed; Stop straight after
    Start; no flash of the setup screen and no stuck clock.
  - **Narrator:** each notice read once (profile saved or loaded, no matching language, the
    application notes, no timing, long session); Rehearse reads only its sentence.
  - **By eye:** the disabled Start in Dark, Light and the contrast themes; the custom-colour
    chips at 980 px in French and German; the tray's minutes in all three languages; the
    projector unplugged and plugged back.
  - SRT export of a line with `Q&A` and `<`; History showing the original speech once per line.
- [ ] **First run on a real Mac**, with Part A of the [macOS checklist](macos.md#part-a-the-direct-app-disk-image)
  on the disk image (the validation tag's, or the release's), including the menu-bar item opening
  its menu on a click and the loopback's parking loop, which the review could not compile on
  Windows. Record the results in [macOS notes](macos.md#results). The Windows release does not
  wait on it; the Mac App Store submission does.
- [ ] **No release candidate.** Tag `v1.13.0` straight from the merge once the test passes. A
  bug found afterwards ships as 1.13.1; never move a tag.

Still open from the [1.12.0 handoff](#release-1120-handoff), the
[1.11.0 handoff](#release-1110-handoff) and earlier, never recorded as done: their lists carry
over unchanged.

### Release and Store acceptance

- [ ] Tag `v1.13.0` (annotated: "Live Translation & Subtitles v1.13.0") on the merge of the
  release PR, and push it.
- [ ] The workflow waits for CI on the tagged commit, then publishes the release with
  `docs/release-1.13.0.md` once the bundle and the disk image are attached; if a job fails, the
  release stays a draft and the failed job is re-run.
- [ ] Verify `Live.Translation_1.13.0.msixbundle` with the [1.8.0 checks](#release-180-handoff)
  at **1.13.0.0**, plus `whisper-vulkan.exe` (native for each architecture) and
  `models/ggml-silero-v5.1.2.bin` in both packages, and record the sizes and SHA-256 digests
  here.
- [ ] Sideload-test the per-architecture MSIX packages and walk the
  [accessibility release checklist](accessibility.md#release-checklist-manual-on-windows); under
  package identity, the helper starts on a PC with a usable graphics processor.
- [ ] **Screenshots:** the current images, from 1.7.1, do not show the *Zoom captions* button
  (1.12.0), *Use the graphics processor* under the model, *Leave out the call heard by the room
  mic* under Both, the disabled Start's new look or the custom-colour chips' new place. Decide
  whether to retake; if so, keep Zoom's windows and artwork out of them. See
  [store-screenshots](store-screenshots/README.md).
- [ ] Run Windows App Certification Kit on the final packages, the first with a second
  executable.
- [ ] Check that the published [privacy policy](https://fmadore.github.io/Live-translation/privacy)
  shows the 5 November 2026 date, the *Optional captions in Zoom* section, the *macOS* section
  and the question under *Third-party processing*.
- [ ] Submit the single combined bundle in Partner Center with the 1.13.0 What's new, the three
  descriptions (with the Zoom trademark line), the changed feature lines, the second requirements
  line and the updated certification notes. The product declarations are unchanged from 1.12.0.
  Rollout: **gradual**, starting at 10% ([route step 9](#the-route)): capture changed (wasapi
  0.27) and the session lifecycle changed (the question before a cloud session, the echo
  filter's held lines, the Whisper helper's start, fallback and end).
- [ ] Record submission and certification, and confirm with
  `Get-AppxPackage 49346FMadore.LiveTranslationSubtitles` (`SignatureKind` `Store`, Version
  `1.13.0.0`).
- [ ] **Mac:** the disk image is on the release; then the [Mac App Store steps for each
  version](mac-app-store.md#each-version): upload the 1.13.0 build from the tag, TestFlight on a
  real Mac with [Part B](macos.md#part-b-the-sandboxed-app) (system audio in the sandbox is the
  open question), the version page, and submission for review. The 5 November 2026 build in App
  Store Connect has not been submitted for review as far as this repository records.

## The route

1. **Bump the version** in `package.json`, `src-tauri/Cargo.toml`,
   `src-tauri/whisper-vulkan/Cargo.toml` and `src-tauri/tauri.conf.json`, synchronize the root
   app versions in `package-lock.json` and `src-tauri/Cargo.lock` (the `live-translation` and
   `whisper-vulkan` entries), and set `version` and `date-released` in `CITATION.cff`.
   `npm test` fails until the files it reads agree (`src/lib/version.test.ts`); the release
   workflow also refuses a tag when the four manifests differ. They must match and be
   **higher than the version already in the Store** — Partner Center
   rejects a package that does not increase. The first segment can never be `0`. Commit them
   together as `chore(release): X.Y.Z`, with the release body in `docs/release-X.Y.Z.md`
   ([`release-notes.md`](release-notes.md) has the register). The body has to be **in the
   tagged commit**: the workflow publishes it from there, and refuses a plain tag without it.
2. **Tag and push after manual testing**: `git tag vX.Y.Z` then `git push origin vX.Y.Z`,
   with the tested version; never reuse a published tag. The
   [`Release installers`](../.github/workflows/release.yml) workflow first checks that the tag
   is `v` followed by the version in all four manifests, then that CI passed on the tagged
   commit, waiting up to an hour for a run still going, before anything is built. It then
   creates the GitHub release as a **draft** and attaches the NSIS and MSI installers, both
   architectures' `.msix` and one multi-architecture `Live.Translation_<version>.msixbundle`.
   Only once the bundle is attached does it publish the release, titled "Live Translation &
   Subtitles X.Y.Z", with `docs/release-X.Y.Z.md` as the body. Download the bundle from there.
3. **If a job fails, nothing is public.** The release stays a draft; re-run the failed jobs,
   and the `bundle` job publishes it when it passes; it refuses to publish until the `build`
   job's installers are attached. If the version or CI check fails, nothing was built and no
   release exists for the tag, so it can be deleted (`git push origin --delete
   vX.Y.Z` and `git tag -d vX.Y.Z`) and the fixed commit tagged again. A hyphenated tag
   (`vX.Y.Z-rc.1`) builds the same assets into a draft prerelease, which is never published
   automatically.
4. **Sideload-test before submitting.** Install the bare per-architecture `.msix` with
   [`scripts/install-local-msix.ps1`](../scripts/install-local-msix.ps1), *not* the bundle —
   signing a bundle does not sign the packages inside it. Walk the manual release checklist in
   [`accessibility.md`](accessibility.md#release-checklist-manual-on-windows) against this
   install, because package identity changes how Windows treats the window.
5. **Re-capture the screenshots** in every listing language if anything they show has
   changed — see [Screenshots](#screenshots).
6. Partner Center → **Apps and games** → *Live Translation & Subtitles* → **Create new
   submission**. In French: **Applications et jeux**, **Créer une soumission**.
7. Under **Packages**, **delete every package already listed**, then upload the single
   `.msixbundle`. One bundle in, nothing else — see
   [Why the package is one bundle](#why-the-package-is-one-bundle).
8. **Paste the listing text** for every language if it changed, from
   [`store-listing.md`](store-listing.md). **What's new in this version** has to be rewritten
   for every release, in every listing language. Editing that file changes nothing on its own;
   the Store only knows what is typed into Partner Center.
9. Set a **gradual rollout** percentage for anything touching audio capture or the session
   lifecycle. Start at 10% and raise it once the crash and review data look clean.
10. **Submit to the Store**, then watch certification: an accepted upload is not a pass.

[`partner-center-walkthrough.md`](partner-center-walkthrough.md) has the screen-by-screen
version of steps 6–10.

## Screenshots

The Store listing is per-language, so each language has its own set under
`docs/store-screenshots/<language>/`, named in upload order — the order is what a visitor
scrolls through. The [capture plan](store-screenshots/README.md) lists the current set and its
status. Each set is re-captured and overwritten in place; git history holds the previous ones.

Capture on Windows at the display scaling the app is actually used at, from a **sideloaded
MSIX** rather than a dev build: package identity changes how the window is drawn. Store rules:

- `.png`, at least 1366×768 and under 50 MB. Ten desktop screenshots at most; Microsoft
  recommends five to eight.
- Anything that matters belongs in the **top two-thirds**, because the Store draws its own text
  overlays across the bottom third.
- No logo, wordmark or marketing line burned into the image. The per-screenshot Description
  field in Partner Center carries that instead, in 200 characters, doubling as the alt text a
  screen reader announces.

Re-capture whenever the UI they show changes; the
[acceptance checklist](microsoft-store.md#acceptance-checklist) includes it.

## Why by hand

Verified 27 August 2026.

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
switching would abandon the published listing and its installs.

A `Microsoft Store submission` workflow driving the `msstore` CLI was removed because a
workflow that cannot run reads as an option that exists. It is in the git history;
[If the account ever becomes a Company account](#if-the-account-ever-becomes-a-company-account)
has what it needed.

## Why the package is one bundle

The Store submission is a single multi-architecture `.msixbundle` containing both the x64 and
the ARM64 package, rather than two single-architecture bundles.

Partner Center treats packages in a submission as a set to be replaced, and it is on the person
doing the replacing to remove the old ones — which is why step 7 says delete every package
already listed. One bundle in and nothing else leaves no room for an architecture to survive
from the previous submission and ship stale. 1.0.5 went up as two per-architecture packages and
had to be cleaned up by hand for exactly this reason.

## Constraints worth knowing

- **One pending submission per product.** Finish or delete the pending one before starting
  another.
- **Certification can still fail.** 1.0.3 uploaded perfectly well and then failed policy
  10.1.2.20 — the reviewer pressed **Start Subtitles** and nothing happened on their device.
  See [`microsoft-store.md`](microsoft-store.md) for what the bundled demonstration exists to
  prevent.
- **The listing is not in this repository.** `store-listing.md` is paste-ready source text, not
  a deployment. Nothing in `docs/` reaches the Store without someone typing or uploading it.

## If the account ever becomes a Company account

None of this is possible on an Individual account; it is kept because it is correct.

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

What the removed workflow had learned: Microsoft supports Store update operations through
GitHub Actions for **free products only**, which this app is; and the `msstore` CLI is in
preview with documentation that trails it — Learn documents an `--inputFile` option for
`msstore publish` that the released CLI does not have, taking the package path as a positional
argument instead.

`msstore submission updateMetadata` could also replace the listing text from a committed JSON
file, making the Store description reviewable in a pull request. But a bad metadata push is
more disruptive than a bad package push: it goes live on the product page rather than waiting
for certification.

## Release history

Earlier handoffs, one row each. Their full checklists, CI runs, artifact sizes and SHA-256
digests are in this file's
[commit history](https://github.com/fmadore/Live-translation/commits/main/docs/store-updates.md);
each release page has the assets and the published body. Checks left unticked when a version
was submitted were not recorded as done and stay open against the Store build.

| Version | GitHub release | Store | Summary |
| --- | --- | --- | --- |
| 1.12.0 | [27 October 2026](https://github.com/fmadore/Live-translation/releases/tag/v1.12.0) · [body](release-1.12.0.md) | Live as 1.12.0.0, reported by the maintainer on 10 November 2026; submission and certification dates not recorded; replaced by 1.13.1 | Captions in Zoom (#189, #190), no flashes or layout shifts in setup and the transcript (#188), and a pre-flight that asks for the application to capture. |
| 1.11.0 | [26 October 2026](https://github.com/fmadore/Live-translation/releases/tag/v1.11.0) · [body](release-1.11.0.md) | Accepted as 1.11.0.0, reported by the maintainer on 26 October 2026; submission and certification dates not recorded; replaced by 1.12.0 | An optional Buy Me a Coffee link that unlocks nothing, and no empty band at the bottom of the operator window (#182). |
| 1.10.0 | [26 October 2026](https://github.com/fmadore/Live-translation/releases/tag/v1.10.0) · [body](release-1.10.0.md) | Live as 1.10.0.0, reported by the maintainer on 26 October 2026 and confirmed with `Get-AppxPackage` the same day; submission and certification dates not recorded; replaced by 1.11.0 | Batches 4–6 of the 22 October review (#171), no process handles outside One application capture (#177) and move mode left on Alt+F4 (#179). |
| 1.9.0 | [22 October 2026](https://github.com/fmadore/Live-translation/releases/tag/v1.9.0) · [body](release-1.9.0.md) | Live as 1.9.0.0, confirmed with `Get-AppxPackage` on 25 October 2026; submission and certification dates not recorded; replaced by 1.10.0 | A light theme that follows Windows' app mode (#159) and batches 1–3 of the 22 October review (#160, #162). |
| 1.8.0 | [21 October 2026](https://github.com/fmadore/Live-translation/releases/tag/v1.8.0) · [body](release-1.8.0.md) | Live as 1.8.0.0, reported by the maintainer on 21 October 2026 and confirmed with `Get-AppxPackage`; submission and certification dates not recorded; replaced by 1.9.0 | The Transcript tab with Copy (#155) and Mistral subtitles on Voxtral Realtime 3 (#156). |
| 1.7.1 | [20 October 2026](https://github.com/fmadore/Live-translation/releases/tag/v1.7.1) · [body](release-1.7.1.md) | Live as 1.7.1.0, confirmed by the maintainer on 21 October 2026; submission and certification dates not recorded; replaced by 1.8.0 | 1.7.0's app code, with the icon shown on the taskbar and in Start without an accent-coloured plate (#152). |
| 1.7.0 | [20 October 2026](https://github.com/fmadore/Live-translation/releases/tag/v1.7.0) · [body](release-1.7.0.md) | Not submitted; superseded by 1.7.1 | Offline Whisper translation into English and overlay placement per display layout (#121), overlay recovery when a display goes (#139), cloud sessions that reconnect (#116, #122). |
| 1.6.1 | [6 October 2026](https://github.com/fmadore/Live-translation/releases/tag/v1.6.1) · [body](release-1.6.1.md) | Live as 1.6.1.0, confirmed by the maintainer on 20 October 2026; submission and certification dates not recorded; replaced by 1.7.1 | The wave-to-words icon (#111) and the 5 October design review (#112); no new features. |
| 1.6.0 | [4 October 2026](https://github.com/fmadore/Live-translation/releases/tag/v1.6.0) · [body](release-1.6.0.md) | Live as 1.6.0.0, confirmed 6 October 2026; replaced by 1.6.1 | Local Whisper (#99), two translation targets, original speech in overlay and exports, Pause/Resume, editable filler words. |
| 1.5.1 | [23 September 2026](https://github.com/fmadore/Live-translation/releases/tag/v1.5.1) · [body](release-1.5.1.md) | Live as 1.5.1.0, confirmed 23 September 2026 | Batches 1–2 of the [22 September app review](app-review-2026-09-22.md) (#86): cleaner downsampled audio, release builds ignore `.env` and host overrides, a failing capture or provider thread ends only its source. |
| 1.5.0 | [22 September 2026](https://github.com/fmadore/Live-translation/releases/tag/v1.5.0) · [body](release-1.5.0.md) | Not submitted | Searchable provider-scoped caption languages, favourites, F2 favourite swapping and RTL overlay direction (#78). |
| 1.4.2 | [21 September 2026](https://github.com/fmadore/Live-translation/releases/tag/v1.4.2) | Not submitted | Start and Stop in a persistent action bar below the header. |
| 1.4.1 | [20 September 2026](https://github.com/fmadore/Live-translation/releases/tag/v1.4.1) | Not submitted | The [18-point operator UI audit](ui-audit-implementation.md): tabbed Settings, profiles above setup, a session browser, localized tray. |
| 1.4.0 | [19 September 2026](https://github.com/fmadore/Live-translation/releases/tag/v1.4.0) | Not submitted | Meeting profiles, 2–30 second caption persistence, appearance presets, history titles and search, operator shortcuts. |
| 1.3.0 | [19 September 2026](https://github.com/fmadore/Live-translation/releases/tag/v1.3.0) | Not submitted | Transcript history (#81), Stable reading (#79), overlay filler cleanup (#80) and the German interface. |
| 1.2.4 | [14 September 2026](https://github.com/fmadore/Live-translation/releases/tag/v1.2.4) | Live; acceptance date not recorded | Responsive captions: Fit window and optional Compact layout (#77). |
| 1.2.3 | [8 September 2026](https://github.com/fmadore/Live-translation/releases/tag/v1.2.3) | No submission recorded | Native Save As with timed SRT/VTT export (#26); capture of a selected application (#27). |

### Release 1.12.0 handoff

**Live as 1.12.0.0**, reported by the maintainer on 10 November 2026; the submission and
certification dates were not recorded, and no `Get-AppxPackage` reading of 1.12.0.0 is recorded.
It was tagged and published on GitHub on 27 October 2026. It replaced 1.11.0.
- **What it shipped:** Captions in Zoom, one caption language posted to a Zoom meeting's own
  caption bar through the caption link (API token) the host copies, held in memory only, behind
  five new operator commands ([#189](https://github.com/fmadore/Live-translation/pull/189));
  a *Zoom captions* toolbar button with a generic captions mark, and *Show captions on this
  screen too* ([#190](https://github.com/fmadore/Live-translation/pull/190)); no flashes or
  layout shifts in setup and the transcript
  ([#188](https://github.com/fmadore/Live-translation/pull/188)); a pre-flight that asks for the
  application to capture. The privacy policy gained its Zoom sections, effective 27 October 2026.
- **Validation:** recorded on 27 October 2026 (ARM64): Vitest 866 passed, 1 skipped, in 84
  files; svelte-check 556 files, 0 errors, 0 warnings; style 143 passed; Rust 358 passed,
  5 ignored; fmt, Clippy, lint, knip, format and languages clean.
- **Tested:** Captions in Zoom passed in a real meeting on a free (Basic) account on 27 October
  2026, on the Local Test build of #189 ([live verification](zoom-captions.md#live-verification)).
  The maintainer's pass over the release merge is not recorded.
- **Tag:** `v1.12.0` (annotated) points to `ebff465`, the merge of
  [#191](https://github.com/fmadore/Live-translation/pull/191); no candidate.
- **Build:** all [installer/MSIX/bundle jobs](https://github.com/fmadore/Live-translation/actions/runs/40471938625)
  passed, and the release published itself with its body on 27 October 2026, about 19 minutes
  after the tag push.
- **Assets** (GitHub's digests):
  - Bundle: 12,106,535 bytes, SHA-256
    `2fbf5291d07853418c43110a86cbca0d58ce891d4b66a5565b6d2b4a4e12495a`.
  - ARM64 MSIX: 5,931,640 bytes, SHA-256
    `7498d1d9bca907d31cb58ed510f2af7cc87338325c3b51404bee4c4a0ab845f0`.
  - x64 MSIX: 6,172,884 bytes, SHA-256
    `91d694bc8c00f26d0618f8c3fed6056b467431358890d6fb0d96a7ef59ee47d3`.
- **Not recorded:** the Store fields checked in Partner Center, the package checks on the
  published assets, the sideload test, WACK, the screenshot decision, the privacy page check,
  the submission and certification dates, and a `Get-AppxPackage` reading. They carry over to
  the next release.

Still open from this handoff, never recorded as done:

- [ ] The maintainer's test of the release merge: Captions in Zoom again, with *Show captions on
  this screen too* (host setup, the start check against a meeting not yet started, one started
  and an expired link, *Send test caption*, captions from Whisper and from a cloud engine, Stop
  and Start while sharing, *Stop sharing*), with the Zoom client version, account type and date
  recorded in the [guide](zoom-captions.md#live-verification); the *Step-by-step setup guide*
  link; the button's label and states and the dialog in English, French and German, with
  Narrator; the pre-flight asking for an application; the setup and transcript no longer
  jumping.
- [ ] The Store fields checked in Partner Center.
- [ ] `Live.Translation_1.12.0.msixbundle` verified with the [1.8.0 checks](#release-180-handoff)
  at 1.12.0.0; the sideload test and the
  [accessibility release checklist](accessibility.md#release-checklist-manual-on-windows); WACK.
- [ ] The screenshot decision for the *Zoom captions* button.
- [ ] The published privacy page showing the *Optional captions in Zoom* section.
- [x] Submission in Partner Center and certification: live as 1.12.0.0, reported by the
  maintainer on 10 November 2026; dates not recorded.
- [ ] `Get-AppxPackage` at 1.12.0.0 with `SignatureKind` `Store`.

### Release 1.11.0 handoff

**Accepted as 1.11.0.0**, reported by the maintainer on 26 October 2026, the day it was
tagged; the submission and certification dates were not recorded. A `Get-AppxPackage` reading
of 1.11.0.0 with `SignatureKind` `Store` is still to be recorded: when the acceptance was
reported, the maintainer's PC still had the Store's 1.10.0.0. It replaced 1.10.0.
- **What it shipped:** an optional *Support development* link to
  `https://buymeacoffee.com/fmadore` in Settings › App and at the foot of the start screen,
  which unlocks nothing and is never shown during a session, opened by the new
  `open_support_page` operator command; `.sr-only` text pinned to its containing block, so the
  operator window no longer scrolls as a whole and leaves an empty band
  ([#182](https://github.com/fmadore/Live-translation/pull/182)). Dependabot's #181.
- **Tested:** the side-by-side Local Test build of `7eb9f3d` (ARM64), "it works", reported by
  the maintainer on 26 October 2026 before the tag; the support link, its three languages,
  the operator window that no longer scrolls and the three themes are recorded as passed
  together.
- **Tag:** `v1.11.0` (annotated) points to `bb22137`, the citation-date commit on top of the
  merge of [#185](https://github.com/fmadore/Live-translation/pull/185); no candidate.
- **Build:** all [installer/MSIX/bundle jobs](https://github.com/fmadore/Live-translation/actions/runs/40289558151)
  passed, and the release published itself with its body on 26 October 2026, about 18 minutes
  after the tag push.
- **Assets** (GitHub's digests):
  - Bundle: 11,950,514 bytes, SHA-256
    `346cf1212847e27a0844bf299c694c560f24fc860c8554463796f2fcd8ba5444`.
  - ARM64 MSIX: 5,856,615 bytes, SHA-256
    `8b9e7a5918cd37279dc8c339ff6032dc5081376e292d9a51b884ff07033caca8`.
  - x64 MSIX: 6,091,888 bytes, SHA-256
    `e8931317d9add9349d7c3eff628eea5f400d0d802c87fd113d7e2b2e3fb6d202`.
- **Not recorded:** the package checks on the published assets, the sideload test, WACK and
  the manual checks below. They carry over to the next release.

Still open from this and earlier handoffs, never recorded as done, on x64 and ARM64 unless a line says
otherwise:

- [ ] From 1.10.0: system audio with local Whisper under load; overlay move mode (drag, edge
  resize, nudge, Snap to bottom on mixed DPI, Escape's undo, and Escape with a display
  removed); Alt+F4 on the overlay, then the toggle and the tray; a strip across two displays;
  a Gemini session of 20 minutes or more across a handover; Start then a fast second click;
  the application list appearing only in One application; Narrator on the recovery note,
  "Starting…" and an Arabic or Hebrew caption.
- [ ] OpenAI's and Mistral's `session_expired` in a live session; a WebView2 reload of the
  overlay (U5).
- [ ] From 1.9.0: Light, Dark and System while switching Windows' app mode; a contrast theme;
  a Mistral rehearsal; Narrator focus on Start and Stop.
- [ ] Mistral on real French and English speech: the glossary, line breaks at sentence and turn
  ends, Pause/Resume/Stop and a brief network drop.
- [ ] The Transcript tab in a long session, and Narrator on the Live and Transcript tabs.
- [ ] A short live session with Gemini Translate, Gemini Transcribe and OpenAI.
- [ ] System audio on x64; a 24-bit microphone; Whisper Small translating real speech.
- [ ] Overlay restore with a laptop at 150% and a projector at 100%; an x64 PC without AVX2.
- [ ] A model download behind a proxy; quitting with a large Whisper backlog.
- [ ] The error sentences in English, French and German; the icon in Start, Alt+Tab, the tray
  and on a light taskbar.

### Release 1.10.0 handoff

**Live as 1.10.0.0**, reported by the maintainer on 26 October 2026, the day it was tagged;
the submission and certification dates were not recorded. That morning the maintainer's PC
still had the Store's 1.9.0.0; later that day `Get-AppxPackage` showed 1.10.0.0 with
`SignatureKind` `Store`. It replaced 1.9.0 and was replaced the same day by 1.11.0.
- **What it shipped:** batches 4–6 of the [22 October review](app-review-2026-10-22.md)
  ([#171](https://github.com/fmadore/Live-translation/pull/171)): a Gemini handover that drains
  the old connection, setup failures retried, system loopback that keeps up under load, an
  overlay with fewer permissions, and operator-window fixes. No process handles outside One
  application capture ([#177](https://github.com/fmadore/Live-translation/pull/177)), and move
  mode left when Alt+F4 hides the overlay
  ([#179](https://github.com/fmadore/Live-translation/pull/179)). Dependabot's #172–#174.
- **Tag:** `v1.10.0` (annotated) points to `cfb427b`, the merge of
  [#179](https://github.com/fmadore/Live-translation/pull/179); no candidate.
- **Build:** all [installer/MSIX/bundle jobs](https://github.com/fmadore/Live-translation/actions/runs/40274338725)
  passed, and the release published itself with its body on 26 October 2026, about 15 minutes
  after the tag push.
- **Assets** (GitHub's digests):
  - Bundle: 11,942,706 bytes, SHA-256
    `38ccdb02d35894834699779fb46d2017529badefea0f2c1949fc7d4b6eaad984`.
  - ARM64 MSIX: 5,848,300 bytes, SHA-256
    `52cfa87ef6a234d1d9d69d6f0e107e628bbd78050b49b9f8d21052f11e565d45`.
  - x64 MSIX: 6,092,395 bytes, SHA-256
    `edebc78f2a725fa90502be177a80e7257463bb05399ac0593285fd68821f07ca`.
- **Not recorded:** the package checks on the published assets, the maintainer's test of the
  Local Test build, the sideload test, WACK and the manual checks. They carry over to
  [1.11.0](#release-1110-handoff), which did not record them either.

### Release 1.9.0 handoff

**Live as 1.9.0.0**: `Get-AppxPackage` reports `SignatureKind` `Store` on 25 October 2026;
the submission and certification dates were not recorded. It replaced 1.8.0.
- **What it shipped:** a light theme, System by default, following Windows' app mode
  ([#159](https://github.com/fmadore/Live-translation/pull/159)); batches 1–3 of the
  [22 October review](app-review-2026-10-22.md)
  ([#160](https://github.com/fmadore/Live-translation/pull/160),
  [#162](https://github.com/fmadore/Live-translation/pull/162)); Tauri 2.12.4
  ([#169](https://github.com/fmadore/Live-translation/pull/169)). The operator window gained
  `core:window:allow-set-theme`; the CSP gained `object-src`, `base-uri`, `form-action` and
  `frame-src 'none'`.
- **Tag:** `v1.9.0` (annotated) points to `756f874`, the merge of
  [#170](https://github.com/fmadore/Live-translation/pull/170); no candidate.
- **Build:** the first run of draft-first publishing: all
  [installer/MSIX/bundle jobs](https://github.com/fmadore/Live-translation/actions/runs/39801417265)
  passed and the release published itself with its body about 20 minutes after the tag push.
- **Package checks:** all 30 passed on the published assets on 23 October 2026 (the
  [1.8.0 checks](#release-180-handoff) at 1.9.0.0).
  - Bundle: 11,866,825 bytes, SHA-256
    `e738b2706795e7777e7d4369566e48d9b57234113956e3109d28f0630cd6b8c2`.
  - ARM64 MSIX: 5,811,541 bytes, SHA-256
    `35e74cdeba60346ce647842ae712e096c5b7cb813fe1cbdea391c4ab4088e000`.
  - x64 MSIX: 6,053,276 bytes, SHA-256
    `87b5bda1f7a5106a3502317c1cf25c345947a9b0cc81305c06d671647763b43f`.
- **Not recorded:** the maintainer's test of the Local Test build, the sideload test, WACK,
  the screenshot retake and the manual checks. They carried over to
  [1.10.0](#release-1100-handoff).

### Release 1.8.0 handoff

**Live as 1.8.0.0**, reported by the maintainer on 21 October 2026; `Get-AppxPackage` reports
`SignatureKind` `Store`. It replaced 1.7.1.
- **What it shipped:** the Live and Transcript tabs, with Copy
  ([#155](https://github.com/fmadore/Live-translation/pull/155)), and Mistral subtitles on
  Voxtral Mini Transcribe Realtime 3 (`voxtral-mini-transcribe-realtime-3-0`, $0.48 an hour,
  45 languages), with lines closed at the end of turn and a names-and-terms list sent as
  `context_bias` ([#156](https://github.com/fmadore/Live-translation/pull/156)).
- **Known risk:** Mistral lists Realtime 3 as a public preview, and release builds pin `-3-0`.
  If Mistral withdraws it, Mistral subtitles stop with Mistral's error until an update ships.
- **Tags:** `v1.8.0` (annotated) points to `502ba34`, the merge of
  [#157](https://github.com/fmadore/Live-translation/pull/157). A `v1.8.0-rc.1` tag was deleted
  before its build made a draft.
- **Build:** the [release-commit CI](https://github.com/fmadore/Live-translation/actions/runs/39581397954)
  and all four [installer/MSIX/bundle jobs](https://github.com/fmadore/Live-translation/actions/runs/39581877713)
  passed.
- **Package checks:** all 36 passed on the published assets:
  - GitHub's digests match;
  - the bundle holds exactly both packages, byte for byte, with no signature;
  - the assigned identity and publisher, at 1.8.0.0;
  - native PE machine types;
  - only `runFullTrust` and `microphone`;
  - the rehearsal WAVs and licence notices, and no weights;
  - the 46 `Assets/` files byte-identical to the repository, with a single `resources.pri`.
- **Bundle:** 11,855,153 bytes, SHA-256
  `a8eaa3bd42e21cc10f344cf99b950adbaf914256994d2da6b4e4740102591121`.
- **Not recorded:** the maintainer's test of the Local Test build, the sideload test, WACK,
  the screenshot retake and the manual checks. They carried over to [1.9.0](#release-190-handoff).

### Release 1.7.1 handoff

**Live as 1.7.1.0**, confirmed by the maintainer on 21 October 2026; the submission and
certification dates were not recorded.
- **What it shipped:** it replaced 1.6.1 in the Store, bringing 1.7.0's app code (1.7.0 itself
  was not submitted) and the unplated taskbar icon
  ([#152](https://github.com/fmadore/Live-translation/pull/152)).
- **Tags:** `v1.7.1` (annotated) and `v1.7.1-rc.1` both point to `2b52552`, the merge of #152.
- **Build:** release-commit CI and all four installer/MSIX/bundle jobs passed.
- **Package checks:** all 50 passed on the published assets. They are the 40 checks of 1.7.0
  at 1.7.1.0. In each package, the 46 `Assets/` files are byte-identical to the repository,
  and a single `resources.pri` (9,296 bytes) indexes the 14 unplated sizes.
- **Bundle:** 11,838,755 bytes, SHA-256
  `286ec7b988192e9df680477d6df33d3636c812cbb520fd4c9ea09cdf9b5f2ac9`.
- **Checked:** the taskbar icon, on ARM64 with a dark taskbar. The EN/FR/DE screenshots were
  recaptured from the 1.7.1 ARM64 MSIX on 21 October 2026
  ([#154](https://github.com/fmadore/Live-translation/pull/154)).
- **Not recorded:** the icon in Start, in Alt+Tab and on a light taskbar, and the 1.7.0 manual
  checks. They carry over to [1.8.0](#release-180-handoff).

### Release 1.7.0 handoff

**Published on GitHub on 20 October 2026 and not submitted;** 1.7.1 took its app code to the
Store.
- **Tags:** `v1.7.0` (annotated) points to `5ad2a0f`, the `chore(release): 1.7.0` commit.
  `v1.7.0-rc.1` points to `bffcd31`.
- **Build:** release-commit CI and all four installer/MSIX/bundle jobs passed.
- **Package checks:** all 40 passed. The bundle holds exactly both packages, byte for byte,
  with no signature. Manifests report 1.7.0.0 with the assigned identity, PE machine types are
  native, the package declares only `runFullTrust` and `microphone`, and it carries the
  rehearsal WAVs and licence notices but no weights.
- **Bundle:** 11,424,375 bytes, SHA-256
  `550765859b61c447b65f040b6fec54a47f457019b27194839489312ce5fd042b`.
- **Maintainer test:** the Local Test build of `f6a2448` on ARM64, on 20 October 2026,
  including system audio on wasapi 0.26. No x64 run is recorded. The other manual checks were
  not itemised and carry over to [1.8.0](#release-180-handoff).

### Release 1.6.1 handoff

Live as 1.6.1.0, confirmed by the maintainer on 20 October 2026: a Store-signed
`49346FMadore.LiveTranslationSubtitles_1.6.1.0_arm64` (`SignatureKind` `Store`) was installed on
the maintainer's PC on 6 October 2026. The submission and certification dates were not
recorded. It replaced 1.6.0 in the Store; 1.7.0 carries its open checks. Tag `v1.6.1` points to
`d206091`. Release-commit CI and all four installer/MSIX/bundle jobs passed, and all 37 package
checks passed on the published assets: GitHub digests, a bundle holding exactly both packages
byte for byte, 1.6.1.0 manifests with the assigned identity, native PE machine types, only
`runFullTrust` and `microphone`, rehearsal WAVs and licence notices, no weights or signature,
and the four new tile images. The bundle was 11,195,249 bytes, SHA-256
`86ef814f0e53baae6ff821db8cc263efa745982a7dfea945d8fda7d69a6ccea0`. The icon check on both
architectures, the smoke test of the new styles, screenshots and Windows App Certification Kit
were not recorded as done before submission.

### Release 1.6.0 handoff

Live as 1.6.0.0, confirmed by the maintainer on 6 October 2026; the submission, certification
and go-live dates were not recorded. Tag `v1.6.0` points to `41cee07`; release-commit CI and
all installer/MSIX/bundle jobs passed, and each downloaded asset's SHA-256 matched GitHub's
digest. Native x64/ARM64 hardware tests, cloud language acceptance (#78), keyboard/Narrator,
enlarged-text and mixed-DPI checks, final screenshots and Windows App Certification Kit were
not recorded as done before submission.

### Release 1.5.1 handoff

Live as 1.5.1.0, confirmed 23 September 2026; it replaced 1.2.4 in the Store. Tag `v1.5.1`
points to `e6a4e18`. Keeping panic unwinding (review item D10) grew the executable; net of
shipping only `.woff2` fonts, the bundle went from 7.96 MB to 9.22 MB. Installed-package
checks, the OpenAI target-code probe, live non-EN/FR speech (#78), screenshots and Windows App
Certification Kit were not recorded as done before submission.

### Release 1.5.0 handoff

Not submitted; superseded by 1.5.1, which carried its open checks. Tag `v1.5.0` points to
`d1ad615`.

## Sources

- [Open a developer account](https://learn.microsoft.com/en-us/windows/apps/publish/partner-center/open-a-developer-account?tabs=individual)
- [Publish app updates to Microsoft Store with GitHub Actions](https://learn.microsoft.com/en-us/windows/apps/publish/msstore-dev-cli/github-actions)
- [Microsoft Store Developer CLI commands](https://learn.microsoft.com/en-us/windows/apps/publish/msstore-dev-cli/commands)
- [Package version numbering](https://learn.microsoft.com/en-us/windows/uwp/publish/package-version-numbering)
- [App screenshots, images, and trailers](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/screenshots-and-images)
