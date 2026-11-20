# Roadmap

GitHub milestones are the source of truth for active work. This file holds the current status,
the open work and a short record of what each release shipped; the
[release notes](docs/release-notes.md), the [Store handoffs](docs/store-updates.md) and git
history keep the detail.

## Current status — 1.14.0 in the Store, 1.15.0 tagged

[**1.15.0**](https://github.com/fmadore/Live-translation/releases/tag/v1.15.0) was tagged and
published on GitHub on 17 November 2026; its Store submission is pending and no 1.15 Mac build
has been uploaded ([release notes](docs/release-1.15.0.md),
[handoff](docs/store-updates.md#release-1150-handoff)). It ships the work of the [16 November app review](docs/app-review-2026-11-16.md)
([#227](https://github.com/fmadore/Live-translation/pull/227)): for the person running the
captions, a spending guard for cloud engines, markers and Find in the transcript, a notice on the
overlay for breaks and opt-in system-wide Pause and overlay keys; a new-version notice for copies
installed from GitHub ([#217](https://github.com/fmadore/Live-translation/issues/217)); Mistral keeping a quiet source open
([#230](https://github.com/fmadore/Live-translation/pull/230)); several saved sessions deleted or saved at once ([#231](https://github.com/fmadore/Live-translation/pull/231)); and the
review's fixes (four overlay rows that each show a line, DeepL keeping concluded text, the echo
filter by session time, the pre-flight's stream count). None of it had run on Windows hardware or
a Mac when it was merged; the review lists what to check by hand. The
[20 November audit](#the-20-november-audit) found defects in this code, filed as issues.

The Store serves [**1.14.0**](https://github.com/fmadore/Live-translation/releases/tag/v1.14.0)
(13 November 2026, MSIX 1.14.0.0), read with `Get-AppxPackage` on the maintainer's ARM64 PC on
17 November 2026 (SignatureKind Store); the submission and certification dates were not
recorded ([handoff](docs/store-updates.md#release-1140-handoff),
[release notes](docs/release-1.14.0.md)): DeepL as an experimental fourth cloud translation
engine, for paid DeepL API plans only
([#220](https://github.com/fmadore/Live-translation/pull/220)); Mistral subtitles that reconnect
after a transcription timeout (code 3804); and overlay rows that each show a line when both
sources share a strip sized for one ([#221](https://github.com/fmadore/Live-translation/pull/221)).
On the Mac, 1.13.1 is still in App Review (17 November 2026); 1.14.0 was not uploaded, and 1.15.0
is the next Mac build.

Before it, the Store served
[**1.13.1**](https://github.com/fmadore/Live-translation/releases/tag/v1.13.1)
(11 November 2026, MSIX 1.13.1.0), read with `Get-AppxPackage` on 13 November 2026
([handoff](docs/store-updates.md#release-1131-handoff),
[release notes](docs/release-1.13.1.md)). It carried the fixes from the Mac app's first run on a
real Mac on top of 1.13.0, which was tagged on 10 November 2026 and published on GitHub only
([release notes](docs/release-1.13.0.md),
[handoff](docs/store-updates.md#release-1130-handoff)): the Mac
app, Whisper on the graphics processor, speech detection, Large v3 Turbo, names and terms for
Gemini Transcribe and Whisper, the prompt before audio first goes to a cloud provider, and the
switch that leaves out the call heard by the room microphone with Both sources. The
[10 November app review](docs/app-review-2026-11-10.md) lists the findings fixed before the tag
([#213](https://github.com/fmadore/Live-translation/pull/213)) and what is still to check by hand.

Before it, the Store served
[**1.12.0**](https://github.com/fmadore/Live-translation/releases/tag/v1.12.0)
(27 October 2026, MSIX 1.12.0.0), live as reported by the maintainer on 10 November 2026; the
submission and certification dates were not recorded
([handoff](docs/store-updates.md#release-1120-handoff), [release notes](docs/release-1.12.0.md)).
1.12.0 replaced 1.11.0 and added:
- [#189](https://github.com/fmadore/Live-translation/pull/189): captions in Zoom, one caption
  language sent to a Zoom meeting's own caption bar through the host's caption API token
  ([guide](docs/zoom-captions.md)), with a toolbar button labelled *Zoom captions* and a generic
  captions mark rather than Zoom's logo.
- [#188](https://github.com/fmadore/Live-translation/pull/188): no flashes or layout shifts in
  setup and the transcript; [#190](https://github.com/fmadore/Live-translation/pull/190): the
  pre-flight says why Start waits for an application.

Before it, the Store served
[**1.11.0**](https://github.com/fmadore/Live-translation/releases/tag/v1.11.0) (26 October 2026,
MSIX 1.11.0.0), accepted as reported by the maintainer the day it was tagged
([handoff](docs/store-updates.md#release-1110-handoff), [release notes](docs/release-1.11.0.md)).
1.11.0 replaced 1.10.0 (batches 4–6 of the
[22 October app review](docs/app-review-2026-10-22.md), which also lists what is left) and
added:
- [#182](https://github.com/fmadore/Live-translation/pull/182): an optional Buy Me a Coffee
  link in Settings › App and on the start screen, which unlocks nothing, with a README section
  and the repository's Sponsor button; and no empty band at the bottom of the operator window.
  It closed [#180](https://github.com/fmadore/Live-translation/issues/180). The Store
  supporter add-on that issue also proposed is not planned: the Store cannot sell a pure
  donation, and a paid cosmetic extra would contradict "unlocks nothing".

Store copy leads with offline captions and optional cloud translation; the scripted demo stays
available as a setup-free display check. Fresh installs select Whisper with Base and automatic
detection, while upgrades preserve saved setups.

## Open verification

The code for these has landed; what remains is evidence from Windows hardware, an installed
package or a native speaker, kept distinct from unit and fixture tests. The
[1.15.0 handoff](docs/store-updates.md#release-1150-handoff) carries the open checks, with
[1.14.0's](docs/store-updates.md#release-1140-handoff),
[1.13.1's](docs/store-updates.md#release-1131-handoff) and
[1.13.0's](docs/store-updates.md#release-1130-handoff).

- **DeepL Voice** ([notes](docs/deepl-voice-api.md)): a first live session with a paid key, which
  confirms the line breaks, segment times, the target codes `en-US`, `zh-Hans` and `nb`, the
  keepalive and the token lifetime; until then the engine picker calls DeepL experimental.

- **Local Whisper:** live hardware, language and Store-package checks
  ([local Whisper](docs/local-whisper.md)), now including Small translating real speech into
  English, the speed of the SIMD builds on x64 and ARM64, an x64 PC without AVX2, a model
  download behind a proxy and quitting with a large backlog.
- **Cloud session resilience** (6 October review, batches 1–2): a live handover, an OpenAI
  session past 60 minutes, Gemini Translate's answer to `audioStreamEnd`, pong replies from
  every provider and Mistral's real `error` payloads.
- **Voxtral Realtime 3** ([#153](https://github.com/fmadore/Live-translation/issues/153)):
  - Real French and English conference speech against 2602 at 240, 480 and 960 ms.
  - Whether the glossary fixes names the model gets wrong.
  - Whether lines break at turn ends and sentence ends on a live speaker and in a Q&A.

  The [21 October check](docs/mistral-realtime-api.md#live-check-21-october-2026) used the
  synthetic rehearsal recordings: both models were word-perfect, and every caption closed on a
  whole sentence.
- **Cloud caption languages** ([#78](https://github.com/fmadore/Live-translation/issues/78)):
  live endpoint and speech acceptance ([language coverage](docs/language-coverage.md)).
- **Accessibility and tray** ([#24](https://github.com/fmadore/Live-translation/issues/24),
  [#22](https://github.com/fmadore/Live-translation/issues/22)): the Narrator, contrast-theme
  and text-scaling walks in [accessibility](docs/accessibility.md#release-checklist-manual-on-windows),
  and keyboard and Narrator operation of the tray menu. Text scaling at the 980 × 660 minimum
  has been verified only in a browser preview at a forced factor, not with the Windows slider.
- **French and German Store listings** (French:
  [#23](https://github.com/fmadore/Live-translation/issues/23)): final-package screenshots and
  a native-speaker review of the catalog and Store copy ([localization](docs/localization.md)).
- **Audio** ([#27](https://github.com/fmadore/Live-translation/issues/27),
  [#28](https://github.com/fmadore/Live-translation/issues/28)): the application isolation
  matrix and the [hardware matrix](docs/audio-device-testing.md) — Teams/Zoom, browser child
  processes, device changes, sleep/wake and mixed-DPI displays. System capture on wasapi 0.27
  has been tested on ARM64 hardware but not on x64, and a 24-bit microphone has not been
  tried.
- **Export and responsive captions** ([#26](https://github.com/fmadore/Live-translation/issues/26),
  [#77](https://github.com/fmadore/Live-translation/issues/77)): the remaining
  [export checks](docs/transcript-export.md); user feedback, packaged Windows checks and fresh
  screenshots for the [caption layout](docs/caption-layout.md), including the overlay's restore
  across launches with a 150% laptop and a 100% projector and its recovery when a display is
  unplugged mid-session.

## Planning conventions

- A milestone is a release outcome, not a promised date.
- Each active issue owns one independently testable change and carries its acceptance criteria.
- High-priority correctness, data-safety, and build work lands before optional features in the
  same milestone.
- Windows behavior is verified in an installed MSIX as well as browser/Tauri development; audio
  changes also require real hardware and a meeting-app test.
- Experimental Windows APIs remain outside Store builds until Microsoft documents them as stable
  and Store-eligible.

## Next

### The 20 November audit

A whole-repository audit of v1.15.0 (`cd0289b`) was checked finding by finding against the code
on 20 November 2026. No structural change is needed; what it found is filed in the milestone
[1.16 — Audit fixes](https://github.com/fmadore/Live-translation/milestone/4), each issue with
its files, the fix and its acceptance criteria. Suggested order:

1. **Before the next Mac upload and Store submission:** [#232](https://github.com/fmadore/Live-translation/issues/232) (the System Audio Recording
   hint returning after a pause in a call; to try on the Mac), [#235](https://github.com/fmadore/Live-translation/issues/235) (a failed History delete
   blocks the session; the batch confirmation survives a filter change) and [#242](https://github.com/fmadore/Live-translation/issues/242) (interface
   and German and French strings).
2. **Captions and sessions:** [#233](https://github.com/fmadore/Live-translation/issues/233) (echo filter and source-only interims), [#234](https://github.com/fmadore/Live-translation/issues/234) (DeepL
   and Mistral turn closing), [#243](https://github.com/fmadore/Live-translation/issues/243) (one punctuation table for turn ends and Zoom), [#237](https://github.com/fmadore/Live-translation/issues/237)
   (Whisper overlap and loops in Chinese and Japanese), [#238](https://github.com/fmadore/Live-translation/issues/238) (microphone unplugged
   mid-session; Test audio and driver panics).
3. **Overlay and setup:** [#236](https://github.com/fmadore/Live-translation/issues/236) (clamp `undo_overlay_move`; a strip left off an unplugged
   display), [#241](https://github.com/fmadore/Live-translation/issues/241) (reduced motion and the overlay's fit), [#240](https://github.com/fmadore/Live-translation/issues/240) (stale Whisper model
   list), [#239](https://github.com/fmadore/Live-translation/issues/239) (Vulkan helper crash report).
4. **Tests, release workflow and cleanups:** [#244](https://github.com/fmadore/Live-translation/issues/244), [#245](https://github.com/fmadore/Live-translation/issues/245) (check all six artefacts before
   publishing) and [#246](https://github.com/fmadore/Live-translation/issues/246).

Where the audit was wrong or overstated, the issue says so: the Vulkan helper is killed on
purpose and stays so ([#239](https://github.com/fmadore/Live-translation/issues/239)); the placement check is generous by design ([#236](https://github.com/fmadore/Live-translation/issues/236)); the
allocation, cache and memory savings are unmeasured ([#243](https://github.com/fmadore/Live-translation/issues/243), [#245](https://github.com/fmadore/Live-translation/issues/245), [#246](https://github.com/fmadore/Live-translation/issues/246)).

Features and research it raised, outside the milestone: [#248](https://github.com/fmadore/Live-translation/issues/248) (fallback engine), [#247](https://github.com/fmadore/Live-translation/issues/247)
(one DeepL session for two caption languages), [#249](https://github.com/fmadore/Live-translation/issues/249) (provider settings), [#250](https://github.com/fmadore/Live-translation/issues/250) (Soniox),
[#251](https://github.com/fmadore/Live-translation/issues/251) (OpenAI subtitles) and [#252](https://github.com/fmadore/Live-translation/issues/252) (a public home page on GitHub Pages).

### Earlier

- **Deferred from the [16 November app review](docs/app-review-2026-11-16.md):** a one-click
  fallback engine when a source ends on a key or quota error (it restarts a source inside a
  running session; [#248](https://github.com/fmadore/Live-translation/issues/248)), and saving the transcript to a chosen folder at Stop (the Mac App Store
  sandbox needs a security-scoped bookmark for a folder kept across launches).

[#187](https://github.com/fmadore/Live-translation/issues/187) was triaged on 26 October 2026:
captions in Zoom first, then Whisper's names-and-terms prompt and speech detection; OBS output
and a display-only corrections list only if needed; Teams CART, the LAN audience viewer, richer
glossaries, live corrections and offline text translation are not planned.

- **Captions in Zoom:** sharing one caption language with a Zoom meeting's own caption bar
  ([guide](docs/zoom-captions.md)) shipped in 1.12.0, after the maintainer's test in a real
  meeting on 27 October 2026. The finer live meeting checks listed in the guide are still to be
  recorded.

- **macOS ([#195](https://github.com/fmadore/Live-translation/issues/195)):** implemented for
  Apple silicon, macOS 15 and later: system audio and one-application capture through Core Audio
  taps, device notifications, the overlay by window level, the Save panel, the Keychain, ⌘Q and
  the Dock, macOS wording and ⌘ shortcuts in all three languages, a macOS CI lane, a test build
  on every push, a notarised disk image built by the release workflow from 1.13.0 on and a free Mac App Store build with its upload workflow
  ([macOS notes](docs/macos.md), [App Store notes](docs/mac-app-store.md)). That workflow
  uploaded a build to App Store Connect on 5 November 2026, which was not submitted; 1.13.1 was,
  and is still in App Review (17 November 2026); 1.15.0 is the next build to upload. First run on a real Mac (an M2 Mac mini on macOS 26, direct and sandboxed builds) on
  10–11 November 2026, whose fixes shipped in 1.13.1
  ([macOS notes](docs/macos.md#first-run-1011-november-2026)). The paid trial proposed in the issue was dropped: the Mac version is
  free.

- **DeepL Voice as a fourth cloud translation engine** (merged in
  [#220](https://github.com/fmadore/Live-translation/pull/220), in 1.14.0, marked experimental
  until a live session has run): DeepL's
  realtime Voice API transcribes and translates in one stream, into 42 of the catalog's caption
  languages, from a company based in the EU ([DeepL notes](docs/deepl-voice-api.md)). Each
  session first requests a single-use token over HTTPS, then streams 16 kHz PCM over a WebSocket.
  It needs a paid DeepL API plan: on 11 November 2026 the Voice route answered 404 on the free
  API host and 403 on the paid one, without credentials, so a free key can never open a session.
  Still to settle: the rate (about $1.85 an hour from third-party sources, not
  from DeepL's own page), the target codes `en-US`, `zh-Hans` and `nb`, and the real latency and
  line breaks on French speech. Later: one DeepL session can carry up to five caption languages,
  so sharing it between the session's two caption languages would halve the cost ([#247](https://github.com/fmadore/Live-translation/issues/247)).

- **The [10 November app review](docs/app-review-2026-11-10.md):** its fixes, the release
  workflow's checks among them, shipped in 1.13.0
  ([#213](https://github.com/fmadore/Live-translation/pull/213)); the review tracks each
  finding's status, what is left for the maintainer and what is still to check by hand.

- **Names and terms beyond Voxtral:** the setup's list also goes to Gemini Transcribe as
  `customVocabulary` and to local Whisper as its decoder prompt, cut to the terms that fit
  ([Whisper notes](docs/local-whisper.md#names-and-terms)).

- **Whisper's speech detection, graphics processor and Turbo:** Silero's voice-activity model,
  bundled, skips windows without speech; loops and subtitle credits are cut from captions;
  Whisper runs on the graphics processor through Metal on a Mac and a Vulkan helper on Windows,
  falling back to the processor; Large v3 Turbo is offered for subtitles
  ([Whisper notes](docs/local-whisper.md#speech-detection)). Speeds on real graphics
  processors are still to be measured.
  The translation models document no vocabulary field, so translated captions take no list.

- **One voice captioned once with Both sources:** with a switch under Both, off by default, a
  room-microphone line repeating what the system audio was captioned with at the same moment is
  left out, judged on the text
  ([architecture](docs/architecture.md#one-voice-captioned-twice)); the setup and the stage say
  where Whisper decoded and why the graphics processor could not be used; on Windows the Vulkan
  helper ends with the app even if a driver call hangs. Learnt from OpenWhispr's meeting
  pipeline (November 2026). A real room with loudspeakers, a call and a local speaker talking
  over it is still to be tried; once it has been, the switch could default to on.

## Unscheduled and research

- [#32 — Windows AI speech-recognition prototype](https://github.com/fmadore/Live-translation/issues/32)
  was closed as not planned on 20 November 2026: local Whisper fills the role on every PC, the
  API is still experimental and so barred from Store builds, and it crashed natively on the
  target ARM64 Surface during the 1.0.x certification attempts. To reopen if Microsoft ships it
  in a stable, Store-eligible channel.
- [#12 — automatic FR ⇄ EN direction](https://github.com/fmadore/Live-translation/issues/12)
  needs a measured provider-switching design before it becomes a release commitment. Two
  caption languages already serve the bilingual room: both directions run side by side, at
  twice the cost, with no switching to get wrong. Leads for one stream: Soniox's two-way mode
  ([#250](https://github.com/fmadore/Live-translation/issues/250)) and one DeepL session for both languages ([#247](https://github.com/fmadore/Live-translation/issues/247)).
- Multi-monitor overlay presets: named placements to switch between in a running session, such
  as "bottom of the projector" and "top of the laptop". The overlay already reopens where it was
  last placed on each display arrangement, clamped to a connected display, and a meeting
  profile already carries a placement with its setup, and since 1.7.0 a display unplugged or
  reconnected mid-session moves the overlay off it and back; what remains is choosing among
  several without a profile and without move mode.
- Make rate-card verification dates visible: apart from Mistral's, whose source and date are in
  the [Mistral API notes](docs/mistral-realtime-api.md#scope), the rates in `providers.ts` have
  no in-repo source or date
  ([6 October review](docs/app-review-2026-10-06.md#5-dependencies-and-provider-drift)).
- The other feature candidates of the
  [6 October review](docs/app-review-2026-10-06.md#6-feature-candidates), beyond #12 (its F3):
  an OBS mode (F4), only if livestreaming needs it; small provider adoptions (F6; [#249](https://github.com/fmadore/Live-translation/issues/249)); and an opt-in
  transcript polish after the session (F7). Per-origin labels (F5) already show on the overlay
  whenever both sources are on screen; captions on attendees' phones over the LAN (F4) and local
  text translation (F8) were set aside in the
  [#187](https://github.com/fmadore/Live-translation/issues/187) triage.
- Streaming local recognition: NVIDIA's cache-aware streaming models (Nemotron Speech
  Streaming, English; Nemotron 3.5 ASR Streaming, 15 languages with detection) and Parakeet TDT
  v3 (25 languages) through sherpa-onnx, as OpenWhispr runs them, would give interim captions
  offline, which Whisper's ten-second windows cannot. Lessons from their Windows packaging:
  rename the bundled ONNX Runtime and rewrite its importers, because Windows 11 ships an older
  `onnxruntime.dll` in System32 that some loader configurations pick first; and use sherpa's
  `-no-tts` archives, whose TTS builds link espeak-ng (GPL-3.0). whisper.cpp 1.9.0 added
  Parakeet support upstream, which would avoid a second inference runtime, but whisper-rs 0.16
  still vendors 1.8.3 and has no binding for it (checked 20 November 2026). Research only.
- More engines ([#250](https://github.com/fmadore/Live-translation/issues/250), [#251](https://github.com/fmadore/Live-translation/issues/251)): Soniox returns transcript and translation on one stream
  at a fraction of the current rates, one target language per session; OpenAI's
  `gpt-live-transcribe` ($0.017 a minute, with keyword hints and several language hints) would
  give OpenAI-only users subtitles that take the names-and-terms list. Both need their
  documentation read, privacy terms and a billable trial before a decision.
- A public home page ([#252](https://github.com/fmadore/Live-translation/issues/252)): GitHub Pages already serves `docs/`, whose home is the
  developers' index; a small page with the Store screenshots and the user guides would give the
  Stores a support URL.
- A billable provider smoke workflow: an explicitly manual run of live credentials against a
  golden audio fixture. It is intentionally not automatic because it costs money and CI secrets
  are not available to forked pull requests.

## Shipped

- **1.15.0** (17 November 2026; published on GitHub, Store submission pending): a spending guard for
  cloud engines, markers and Find in the transcript, a notice on the overlay for breaks, opt-in
  system-wide Pause and overlay keys, a new-version notice for copies installed from GitHub
  ([#217](https://github.com/fmadore/Live-translation/issues/217)), Mistral keeping a quiet source open
  ([#230](https://github.com/fmadore/Live-translation/pull/230)), History multi-select ([#231](https://github.com/fmadore/Live-translation/pull/231)), and the fixes of the
  [16 November app review](docs/app-review-2026-11-16.md)
  ([#227](https://github.com/fmadore/Live-translation/pull/227)).
  [Release body](docs/release-1.15.0.md).
- **1.14.0** (13 November 2026; live in the Store, read 17 November 2026): DeepL as an experimental
  fourth cloud translation engine, paid DeepL API plans only
  ([#220](https://github.com/fmadore/Live-translation/pull/220)); Mistral reconnects after a
  transcription timeout (code 3804); every overlay row shows a line when both sources share a
  shallow strip ([#221](https://github.com/fmadore/Live-translation/pull/221)).
  [Release body](docs/release-1.14.0.md).
- **1.13.1** (11 November 2026; live in the Store, read 13 November 2026, replaced by 1.14.0; in Mac App Review): the fixes from the Mac app's
  first run on a real Mac: the overlay over full-screen apps, Stop while System Audio Recording
  is unanswered, a refused permission flagged, plainer capture messages on both systems, and
  the Mac App Store screenshots
  ([#218](https://github.com/fmadore/Live-translation/pull/218)).
  [Release body](docs/release-1.13.1.md).
- **1.13.0** (10 November 2026; published on GitHub, never submitted to the Store): the Mac app for Apple silicon
  ([#202](https://github.com/fmadore/Live-translation/pull/202)); Whisper on the graphics
  processor, speech detection, loop and credit filters and Large v3 Turbo
  ([#208](https://github.com/fmadore/Live-translation/pull/208)); names and terms for Gemini
  Transcribe and Whisper; the question before audio first goes to a cloud provider; the switch
  that leaves out the call heard by the room microphone
  ([#212](https://github.com/fmadore/Live-translation/pull/212)); and the fixes of the
  [10 November app review](docs/app-review-2026-11-10.md)
  ([#213](https://github.com/fmadore/Live-translation/pull/213)).
  [Release body](docs/release-1.13.0.md).
- **1.12.0** (27 October 2026; live in the Store, reported 10 November 2026, replaced by 1.13.1): captions in Zoom, one caption
  language in a Zoom meeting's own caption bar
  ([#189](https://github.com/fmadore/Live-translation/pull/189)); no flashes or layout shifts in
  setup and the transcript ([#188](https://github.com/fmadore/Live-translation/pull/188)); and a
  pre-flight that says why Start waits for an application.
  [Release body](docs/release-1.12.0.md).
- **1.11.0** (26 October 2026; accepted in the Store, reported 26 October 2026, replaced by 1.12.0): an optional Buy Me a Coffee link that unlocks
  nothing, and no empty band at the bottom of the operator window
  ([#182](https://github.com/fmadore/Live-translation/pull/182)).
  [Release body](docs/release-1.11.0.md).
- **1.10.0** (26 October 2026; live in the Store the same day, replaced by 1.11.0): batches 4–6 of the
  [22 October app review](docs/app-review-2026-10-22.md)
  ([#171](https://github.com/fmadore/Live-translation/pull/171)), no process handles outside
  One application capture ([#177](https://github.com/fmadore/Live-translation/pull/177)) and
  move mode left on Alt+F4 ([#179](https://github.com/fmadore/Live-translation/pull/179)).
  [Release body](docs/release-1.10.0.md).
- **1.9.0** (22 October 2026; live in the Store from 25 October 2026, replaced by 1.10.0): a light theme that follows
  Windows' app mode ([#159](https://github.com/fmadore/Live-translation/pull/159)), and batches
  1–3 of the [22 October app review](docs/app-review-2026-10-22.md)
  ([#160](https://github.com/fmadore/Live-translation/pull/160),
  [#162](https://github.com/fmadore/Live-translation/pull/162)).
  [Release body](docs/release-1.9.0.md).
- **1.8.0** (21 October 2026; live in the Store, confirmed 21 October 2026): a Transcript tab for reading
  and copying the transcript during a session
  ([#155](https://github.com/fmadore/Live-translation/pull/155)); Mistral subtitles on Voxtral
  Realtime 3, with lines closed at the speaker's turn and a names-and-terms list
  ([#156](https://github.com/fmadore/Live-translation/pull/156)).
  [Release body](docs/release-1.8.0.md).
- **1.7.1** (20 October 2026; live in the Store, confirmed 21 October 2026): 1.7.0's app code
  with unplated taskbar icon sizes and a resource index in the package, so the taskbar and Start
  no longer draw the icon on an accent-coloured plate. [Release body](docs/release-1.7.1.md).
- **1.7.0** (20 October 2026, GitHub only; superseded in the Store by 1.7.1): the six batches
  of the [6 October app review](docs/app-review-2026-10-06.md)
  ([#116](https://github.com/fmadore/Live-translation/pull/116)–[#121](https://github.com/fmadore/Live-translation/pull/121)),
  with offline Whisper translation into English (F1) and an overlay that reopens where it was
  placed (F2); Mistral reconnects after error 3803
  ([#122](https://github.com/fmadore/Live-translation/pull/122)); overlay recovery when a
  display is disconnected ([#139](https://github.com/fmadore/Live-translation/pull/139));
  SvelteKit 3 ([#138](https://github.com/fmadore/Live-translation/pull/138)).
  [Release body](docs/release-1.7.0.md).
- **1.6.1** (6 October 2026, Store): the wave-to-words icon
  ([#111](https://github.com/fmadore/Live-translation/pull/111)) and the 5 October design
  review ([#112](https://github.com/fmadore/Live-translation/pull/112)).
  [Release body](docs/release-1.6.1.md).
- **1.6.0** (4 October 2026, Store): local Whisper transcription
  ([#99](https://github.com/fmadore/Live-translation/pull/99)), suggested by @valentinrabot in
  [#98](https://github.com/fmadore/Live-translation/issues/98); two translation targets,
  original speech in exports and optionally in the overlay, and Pause/Resume (#90); an
  editable filler-word list (#89 / #85); review batches 3–4 (#87/#88), lighter history and
  caption updates, improved Gemini reconnection. [Release body](docs/release-1.6.0.md).
- **1.5.1** (23 September 2026, Store): batches 1–2 of the
  [22 September app review](docs/app-review-2026-09-22.md)
  ([#86](https://github.com/fmadore/Live-translation/pull/86)). [Release body](docs/release-1.5.1.md).
- **1.5.0** (22 September 2026): searchable caption-language selector with favourites (#78):
  78 Gemini and 13 OpenAI targets, including German
  ([#56](https://github.com/fmadore/Live-translation/issues/56)).
  [Release body](docs/release-1.5.0.md).
- **1.4.2** (21 September 2026): a persistent Start/Stop bar below the app header.
- **1.4.1** (20 September 2026): the [operator UI audit](docs/ui-audit-implementation.md) —
  tabbed Settings, profile picker, history browser, presets and swatches, localized tray labels
  and date fields.
- **1.4.0** (19 September 2026): [meeting profiles, reading controls](docs/usability.md),
  history titles and search, live input status, keyboard shortcuts and Gemini subtitle
  corrections.
- **1.3.0** (19 September 2026): opt-in [transcript history](docs/transcript-history.md) (#81),
  Stable reading, the overlay filler-word filter and the German interface.
- **1.2.3–1.2.4:** native Save As with SRT/VTT export (#26), application audio capture (#27)
  and the responsive caption layout (#77).
- **1.2.1–1.2.2:** device lifecycle and output selection (#28), Jump to latest, shorter
  transcript paragraphs, per-source handling of fatal provider errors and atomic recovery
  snapshots.
- **1.2.0:** the French interface (#23); operator-chosen caption typeface, colours and width
  ([#54](https://github.com/fmadore/Live-translation/issues/54),
  [#55](https://github.com/fmadore/Live-translation/issues/55)) with a composite contrast
  readout; a settings panel; the accessibility pass, including Windows text scaling (#24);
  caption timestamps; and separate operator and overlay capabilities
  ([#31](https://github.com/fmadore/Live-translation/issues/31)).
- **1.1.0** (27 August 2026, the first Store update): Gemini Transcribe Live as a second
  subtitle engine; the transcript as a document with an optional recovery spool
  ([#25](https://github.com/fmadore/Live-translation/issues/25)); a system tray with safe
  close and quit (#22); a truthful audio preflight
  ([#20](https://github.com/fmadore/Live-translation/issues/20)); F2 no longer promises a
  mid-session direction flip ([#21](https://github.com/fmadore/Live-translation/issues/21));
  no Tauri IPC calls in browser previews
  ([#29](https://github.com/fmadore/Live-translation/issues/29)).
- **1.0.5** (first Store certification): MSIX packaging, the privacy policy and a keyless
  default path. Store policy 10.8.3 bars individual accounts from requiring API keys for primary
  functionality, and policy 10.3 requires certification to be able to test the app. On-device
  recognizers failed on unconfigured review hardware, so 1.0.5 replaced them with a
  deterministic bundled demonstration and became native x64 and ARM64. The lesson stands for
  the next attempt: a keyless path that depends on the reviewer's hardware, language packs or
  privacy settings is not a keyless path. macOS support was dropped then; it returned for Apple
  silicon in [#195](https://github.com/fmadore/Live-translation/issues/195), for 1.13.0. The
  Linux CI lane is a compile check only. See [Microsoft Store submission](docs/microsoft-store.md).
- **0.4.0** (August 2026): Mistral Voxtral subtitles, plain-text and Markdown export, current
  provider contracts, a serialized start/stop lifecycle, bounded audio channels, frontend
  tests, Windows and Linux Rust CI with npm/RustSec audits.
- **0.3.0:** fixes from the July and August 2026 reviews — per-origin status and captions,
  capture-error reporting, a shared realtime session runner with backoff reset and fail-fast
  handshakes, stale audio dropped before reconnecting, async commands, overlay move mode and
  the first CI workflow.
