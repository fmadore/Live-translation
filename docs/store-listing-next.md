# Next Store submission — 1.16.0

**1.16.0 tagged on 23 November 2026; Store submission pending.** The Store serves
**1.15.0.0**, read from the installed package with `Get-AppxPackage` on the maintainer's ARM64
PC on 23 November 2026 (SignatureKind Store); the submission and certification dates were not
recorded. Everything 1.15.0 brought to the listing is in Partner Center. The next submission is
**1.16.0.0**, replacing it: one unsigned bundle with native x64 and ARM64 packages, at
`https://github.com/fmadore/Live-translation/releases/download/v1.16.0/Live.Translation_1.16.0.msixbundle`
once the tag is built.

What's new and the certification notes' first paragraph cover the changes since 1.15.0. What
1.15.0, 1.14.0, 1.13.1 and 1.12.0 brought to the listing is already in Partner Center and stays.

The positioning is unchanged. Lead the listing with offline Whisper captions, the transparent
overlay and optional cloud translation; the scripted demo is a secondary way to try the display
controls. New installations open on Whisper / Base / automatic detection, and existing saved
setups are preserved. Model download and capture remain explicit user actions. The Store
listing is for Windows: 1.13 and later also ship a Mac app, which the listing does not mention.

## On main after 1.16.0, for the next listing

Nothing yet.

## What 1.16.0 adds

From the [release notes](release-1.16.0.md), user-visible:

- **OpenAI subtitles** ([#251](https://github.com/fmadore/Live-translation/issues/251),
  [#266](https://github.com/fmadore/Live-translation/pull/266)): OpenAI's `gpt-live-transcribe`
  (Realtime transcription) joins Whisper, Mistral (Voxtral) and Gemini under Live subtitles, on
  the same OpenAI key as its translation (no second key), at $0.017 a minute, **$1.02 an hour**
  (OpenAI's pricing, read 23 November 2026). It takes the names and terms (as `keywords`) and the
  chosen spoken language as hints. Sessions last 60 minutes and hand over. Both OpenAI engines
  share one answer to the question asked before audio first goes to a provider.
- **Continue with offline Whisper** ([#248](https://github.com/fmadore/Live-translation/issues/248),
  [#271](https://github.com/fmadore/Live-translation/pull/271)): when a cloud engine ends a source
  mid-session on a key, quota or billing error, or has been reconnecting for 30 seconds, a banner
  offers **Continue with offline Whisper** (or **Continue in the spoken language** when the
  installed model cannot translate or the caption language is not English). The same session
  goes on: same clock, transcript and history entry, with an automatic marker at the switch. One
  click continues every waiting source; **Not now** leaves them. It needs an installed Whisper
  model and a processor that can run Whisper, and is not offered in a rehearsal, the demo or with
  two caption languages. Whisper time costs nothing in the estimate, and nothing new is sent
  anywhere.
- **Gemini subtitles fixed** ([#268](https://github.com/fmadore/Live-translation/pull/268)):
  since 1.4.0, every Gemini Transcribe caption after the first repeated everything said before
  it, because Google's SMART mode sends cumulative finals. The app now uses VERBATIM: each
  caption arrives about a second after the speaker pauses, word for word, fillers included (the
  app's own optional filler filter can still hide them on the overlay). Gemini subtitles now take
  a **spoken language** (71 languages, default Detect automatically).
- **OpenAI noise reduction by source** ([#249](https://github.com/fmadore/Live-translation/issues/249)):
  far-field for the microphone, none for system audio, with the OpenAI translation engine.
- **The first words at Start or Resume** are no longer lost on cloud engines: audio captured
  while the connection opens is buffered and sent
  ([#266](https://github.com/fmadore/Live-translation/pull/266)).
- **Fixes** from the 20 November audit and
  [#272](https://github.com/fmadore/Live-translation/pull/272): the echo filter catches
  microphone echoes that only have source text; DeepL (experimental) sends a finished
  sentence's last words without waiting for the next one; Mistral's Pause and Stop no longer
  wait after a forced close; Whisper removes overlaps and loops in Chinese, Japanese and Thai; a
  crashed graphics-processor helper is reported with its cause; an unplugged microphone says so;
  a driver crash in Test audio no longer leaves the test stuck; the overlay comes back when an
  unplugged projector leaves most of it off screen; History restores sessions whose delete
  failed; Stop no longer waits for a Whisper model to finish loading; transcripts holding only
  markers can be saved, recovered and kept in history; the cost estimate stops counting a paused
  track; the overlay uses each track's own caption language (right-to-left included) after a
  switch; reduced motion is fully respected; the German interface says "Overlay" throughout.
- **Security updates**: `url` 2.5.9 and `cookie` 0.18.3 (RustSec advisories).

[store-listing.md](store-listing.md) replaces What's new in each language (OpenAI subtitles
and the offline Whisper fallback first, then the Gemini fix, the first words, and a short list of
fixes) and the certification notes' first paragraph. Each description changes in its fourth
paragraph only: Gemini, Mistral and OpenAI offer cloud subtitles, OpenAI on the same key as its
translation; OpenAI joins the engines that take names and terms; and one new sentence says the
app offers to carry on with offline Whisper in the same session when a cloud engine stops. Two
feature lines change in place, the first (offline Whisper can carry on a session) and the fifth
(cloud subtitles with OpenAI), so the list stays at twenty. The certification notes'
*Optional cloud features* paragraph adds OpenAI subtitles, the consent answer both OpenAI engines
share and the names and terms sent to OpenAI. No capability or product declaration changes: the
MSIX capabilities are still `runFullTrust` and `microphone`, and `AppxManifest.xml.template` and
`bundle.config.json` are unchanged since `v1.15.0`. The privacy policy's effective date is
**23 November 2026**: it names OpenAI as a recipient of subtitle audio, names and terms and the
chosen spoken language, and corrects where names and terms go (Mistral `context_bias`, Gemini
Transcribe `customVocabulary`, OpenAI subtitles `keywords`, local Whisper on the device). The
fallback to offline Whisper sends nothing new.

## What 1.15.0 adds

From the [release notes](release-1.15.0.md) and the
[16 November app review](app-review-2026-11-16.md)
([#227](https://github.com/fmadore/Live-translation/pull/227)), user-visible:

- **Spending guard for cloud engines** (Settings › App › Cloud spending, all off by default):
  pause a session nothing has been captioned in for 5–30 minutes, through the ordinary Pause,
  and say once when the estimated cost or the session's length passes a chosen value. Never
  stops a session.
- **Markers in the transcript**: Add marker (Ctrl+Shift+M) puts a talk's start, a speaker's
  name or a numbered section break into the transcript, its exports and history. Not shown on
  the overlay.
- **Find in the live transcript** (Ctrl+F on the Transcript tab), with highlighted matches and
  the History search's matching.
- **A notice on the overlay for breaks** ("Back soon"), in place of the captions until cleared
  or until captions resume after a pause.
- **System-wide Pause and Show/hide overlay keys** (Settings › App, off by default):
  Ctrl+Alt+Shift+P and Ctrl+Alt+Shift+O work with slides or a meeting app in front, only while
  a session runs. No Start or Stop key. `RegisterHotKey` needs no MSIX capability.
- **Several saved sessions at once** in Transcript history
  ([#231](https://github.com/fmadore/Live-translation/pull/231)): select sessions (or every one
  the search shows), then save them all to a folder or delete them after one confirmation.
  Files go only to the folder the user picks; nothing new is sent or kept, so the privacy
  policy is unchanged.
- **The review's fixes**: four overlay rows that each show a line, with a notice when captions
  are drawn smaller than chosen; DeepL keeping concluded text; the echo filter by session time;
  the pre-flight counting both sources, with a Download button on a first run; and smaller ones
  listed in the release notes.

Not for the Store listing: the **new-version notice** ([#217](https://github.com/fmadore/Live-translation/issues/217))
is for copies installed from GitHub only. Every MSIX has package identity, and the core never
checks from a copy that has one (`src-tauri/src/update.rs`), so a Store copy never shows the
question and never contacts GitHub. The certification notes say so in one sentence, because the
privacy policy describes the check.

[store-listing.md](store-listing.md) replaces What's new in each language and the certification
notes' first paragraph, adds the operator tools to the descriptions' fifth and sixth paragraphs,
and changes three feature lines in place (the fourth, twelfth and fourteenth), so the list stays
at twenty. No capability or product declaration changes: the MSIX capabilities are still
`runFullTrust` and `microphone`, pinned by `src/lib/msixIdentity.test.ts`, and
`AppxManifest.xml.template` and `bundle.config.json` are unchanged since `v1.14.0`. The privacy
policy's effective date is **23 November 2026**. The 16 November version added the check for
new versions (GitHub copies only), the system-wide keys (the app is told of these two
combinations only) and the markers kept with the transcript; the 23 November one only corrects
where the names and terms are used (Gemini Transcribe and local Whisper as well as Mistral),
with nothing new sent, so the listing needs no matching line. 1.14.0 was in the Store before that change was published,
which is harmless: the policy describes features the Store copy either lacks or does not yet have.

## What 1.14.0 adds

From the [release notes](release-1.14.0.md):

- **DeepL** (DeepL Voice API, by DeepL SE, Cologne) as a fourth cloud engine, beside Google
  Gemini, OpenAI and Mistral ([#220](https://github.com/fmadore/Live-translation/pull/220),
  [DeepL notes](deepl-voice-api.md)). It serves live translation only: DeepL hears which
  language is spoken and writes the caption language the operator chooses, in 42 of the app's
  caption languages ([coverage](language-coverage.md)). It needs a paid DeepL API plan, and the
  app refuses a free key (`error.deeplPaidPlan`). The engine picker marks it **experimental**:
  it has not yet run in a live session.
- **Mistral reconnects after a transcription timeout** (code 3804) instead of ending the
  session.
- **The overlay with both sources** in a strip sized for one shows a line in every row instead
  of the Room and Remote labels over empty rows
  ([#221](https://github.com/fmadore/Live-translation/pull/221)), reported from a real Zoom
  meeting with Mistral subtitles.

[store-listing.md](store-listing.md) replaces What's new in each language and the certification
notes' first paragraph, and adds DeepL as planned below. No capability or product declaration
changes: the MSIX capabilities are still `runFullTrust` and `microphone`, and
`AppxManifest.xml.template` and `bundle.config.json` are unchanged since `v1.13.1`.

### DeepL in the listing

Planned with #220, done in the 1.14.0 copy and in the Store since 1.14.0:

- **Descriptions.** The opening paragraph names DeepL with Gemini and OpenAI, under the user's
  own DeepL API key, and says DeepL needs a **paid DeepL API plan**: Voice is not on DeepL's free
  API, and the app refuses a free key (one ending in `:fx`) with its own message. The cloud
  paragraph gives DeepL's 42 languages and says it is experimental. Nothing in the copy suggests
  a free DeepL tier.
- **Features.** The fourth line, on cloud translation, is replaced by one naming Gemini, OpenAI
  or DeepL with the user's own key, with DeepL's paid plan and experimental status; the list
  stays at Partner Center's maximum of twenty.
- **Privacy policy.** Its effective date is **11 November 2026**: DeepL is named as a
  recipient (the audio of the chosen sources while a DeepL session runs, and the caption
  language), with DeepL's privacy policy linked. The question before audio first goes to a
  provider asks about DeepL on its own. 1.13.1 was in the Store before this change was
  published, so the published page names DeepL a few days before the Store package contains it;
  that is harmless.
- **Certification notes.** The first paragraph and the *Optional cloud features* paragraph add
  DeepL: a tester needs a key from a paid DeepL API plan to exercise it, and can see the refusal
  of a free key without one.
- **Keywords.** Not changed: all seven slots are taken. "DeepL" would be named descriptively, as
  the other providers are, if a slot is freed later.

## What 1.13.1 adds

From the [release notes](release-1.13.1.md): mostly the Mac app's fixes from its first run on a
real Mac, which the Windows listing does not mention. Two reach Windows: Stop no longer waits on
local Whisper when an audio device stops answering, and a closed application, an unplugged
output or a missing microphone each get a plain message (`error.applicationClosed`,
`error.outputDisconnected`, `error.micNone`). [store-listing.md](store-listing.md) adds one
sentence on them to the end of What's new in each language and to the certification notes' first
paragraph, and changes the version there to 1.13.1.0. No other field, capability or declaration
changes.

## What 1.13.0 adds

From the [release notes](release-1.13.0.md): Whisper on the graphics processor (a Vulkan helper
on Windows), bundled speech detection, loop and credit filters and Large v3 Turbo
([#208](https://github.com/fmadore/Live-translation/pull/208),
[local Whisper notes](local-whisper.md#graphics-processor)); names and terms for Gemini
Transcribe and Whisper ([#193](https://github.com/fmadore/Live-translation/pull/193)); a
question asked once for each cloud provider before audio is first sent to it (in
[#202](https://github.com/fmadore/Live-translation/pull/202)); the switch that leaves out the
call heard by the room microphone with Both sources
([#212](https://github.com/fmadore/Live-translation/pull/212)); and the fixes of the
[10 November app review](app-review-2026-11-10.md)
([#213](https://github.com/fmadore/Live-translation/pull/213)). The macOS port in the same
release does not reach the Microsoft Store.

## What the submission needs

In the Store since 1.12.0, and kept:

- **Privacy policy (Store Policy 10.5, personal information).** Caption text can contain
  personal information, and Captions in Zoom sends it to Zoom once the user starts sharing. The
  published [privacy policy](privacy.md) describes that flow in *Optional captions in Zoom* and
  in the Zoom line under *Third-party processing*. Sharing is opt-in by construction: nothing is
  sent until the user pastes a link and chooses **Start sharing**, and the dialog says, before
  that button, that caption text goes to Zoom even when Whisper makes it.
- **Notes for certification.** The second paragraph of the notes in
  [store-listing.md](store-listing.md#notes-for-certification) says what Captions in Zoom sends
  and where, that it is off until the user starts it, and how a tester can exercise it (a Zoom
  meeting they host; a free Basic account is enough, confirmed by the maintainer on 27 October
  2026) or skip it (pasting a non-Zoom address, which the app refuses without sending anything).
- **Trademark (Policies 10.1 and 11.2).** Zoom is named only descriptively; the app shows no Zoom
  logo, and neither the screenshots nor the logo use Zoom's artwork. Each description ends with
  "Zoom is a trademark of Zoom Communications, Inc. Live Translation & Subtitles is not affiliated
  with or endorsed by Zoom." in its language. Zoom is not added to the keywords.
- **Descriptions and features.** The Zoom sentences in the overlay, privacy and requirements
  paragraphs, and the twentieth feature line, the most Partner Center allows.

In the Store since 1.13.1 (new in 1.13.0), and kept:

- **A second executable in the package.** `whisper-vulkan.exe` sits beside the app's
  executable. It is part of the same full-trust app, started and stopped only by it, opens no
  network connection, and needs no new capability: it reaches the graphics processor through
  the graphics driver already installed. The certification notes say so.
- **A bundled model.** `models/ggml-silero-v5.1.2.bin` (864 KiB, Silero VAD, MIT) ships in the
  package. The Whisper speech models are still downloaded only at the user's request; Large v3
  Turbo is a new one, about 547 MiB.
- **The question before audio first goes to a cloud provider.** It names the provider, says
  what is sent and under whose terms, gives the privacy policy's address and waits for **Send**;
  **Cancel**, which has the focus, starts nothing. The answer is kept on the device for that
  provider. A certification tester with cloud credentials meets it on the first cloud session,
  so the notes' *Optional cloud features* paragraph describes it.
- **Names and terms to Gemini.** The list now goes to Gemini Transcribe as well as Mistral;
  Whisper keeps it on the PC. The certification notes say so.
- **Privacy policy.** Its effective date was **5 November 2026** for 1.13.0 (now 11 November
  2026, for DeepL), and it covers the macOS
  app, the question before audio first goes to each provider, names and terms for Gemini, and
  the Vulkan helper, which uses only the computer's own memory. It also keeps 1.12.0's Zoom
  sections, so 1.13.0 changed nothing in it.
- **Screenshots.** The current images are from 1.7.1. 1.13.0 changes what they would show: the
  setup gains *Use the graphics processor* under the model and, with Both sources, *Leave out the
  call heard by the room mic*; a Start that cannot be pressed has a neutral look; the
  custom-colour chips in Settings › Captions moved. The toolbar has had *Zoom captions* since
  1.12.0. See the [screenshot plan](store-screenshots/README.md).

No product declaration changes: the purchases-outside-Microsoft-commerce box ticked for 1.11.0
stays ticked, and neither Zoom captions nor anything in 1.13.0 involves a payment. The MSIX
capabilities are unchanged (`runFullTrust` and `microphone`); `AppxManifest.xml.template` and
`bundle.config.json` are unchanged since `v1.12.0`.

## The copy

[store-listing.md](store-listing.md) is the 1.16.0 copy, with every block in English, French and
German: What's new, the descriptions' new sentences, two feature lines and the certification
notes. The French and German sentences new in 1.16.0 need the maintainer's review.

- [EN/FR/DE descriptions, What's new and certification notes](store-listing.md)
- [Release notes](release-1.16.0.md)
- [Build evidence and acceptance checklist](store-updates.md#release-1160-handoff)
- [Captions in Zoom guide](zoom-captions.md), which the app's dialog opens
- [Screenshot plan](store-screenshots/README.md)
- [Partner Center walkthrough](partner-center-walkthrough.md)

Before submission, complete:

- a test of the 1.16.0 package on Windows hardware: at least an OpenAI subtitles session with an
  OpenAI key (names and terms, a chosen spoken language), Gemini subtitles arriving once each
  without earlier text and with a chosen spoken language, the first words after Start and Resume
  on a cloud engine, and Continue with offline Whisper after a cloud engine stops (a revoked key,
  or the network cut for 30 seconds), with the marker in the transcript and one history entry;
- the final-package checks, including `whisper-vulkan.exe` and the bundled model in both
  packages, and the checks still open from earlier releases
  ([1.15.0's](store-updates.md#release-1150-handoff),
  [1.14.0's](store-updates.md#release-1140-handoff),
  [1.13.1's](store-updates.md#release-1131-handoff) and
  [1.13.0's](store-updates.md#release-1130-handoff));
- the screenshot decision (1.16.0 adds OpenAI as a fifth engine card in Subtitles);
- the German and French copy review of the 1.16.0 sentences;
- Windows App Certification Kit on the final packages.

Use the normal Store identity: the side-by-side Local Test installation is for feedback, not the
Store submission package.
