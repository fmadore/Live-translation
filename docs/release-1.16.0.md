# Live Translation & Subtitles 1.16.0

OpenAI joins the subtitles engines, and a session no longer has to end when a cloud engine
does: the app offers to carry on with offline Whisper in the same session. Gemini subtitles stop
repeating earlier speech, and the first words after Start are no longer lost. It also carries the
fixes of the 20 November audit.

## New

- **OpenAI subtitles.** OpenAI's `gpt-live-transcribe` is a fifth engine under Live subtitles,
  beside Whisper, Gemini, Mistral and the demo. It uses the same OpenAI key as OpenAI
  translation, takes the spoken language and the names-and-terms list as hints, and costs about
  $1.02 an hour (OpenAI's price, read 23 November 2026). Someone with only an OpenAI key now gets
  subtitles as well as translation
  ([#251](https://github.com/fmadore/Live-translation/issues/251)).
- **Continue with offline Whisper.** When a cloud engine ends a source during a session, on a
  key, quota or billing error, or has been reconnecting for 30 seconds, a banner offers
  **Continue with offline Whisper**. One click continues every waiting source in the same
  session: the clock, transcript and history entry carry on, and a marker such as "System audio:
  offline Whisper from here" records the switch. **Not now** leaves the sources as they are. The
  banner picks the setup's Whisper model when it is installed, otherwise the best installed one;
  when that model cannot translate into the caption language, it offers to continue in the
  spoken language and says why. It is not offered in a rehearsal, with the demo or with two
  caption languages, without an installed model, or on a processor that cannot run Whisper. The
  cost estimate stops counting a stream once it ends, and Whisper time costs nothing
  ([#248](https://github.com/fmadore/Live-translation/issues/248)).
- **A spoken language for Gemini subtitles.** The setup now offers the spoken-language picker
  for Gemini Transcribe (71 languages, *Detect automatically* by default). A wrong choice biases
  recognition rather than forcing it.
- **OpenAI noise reduction by source:** the microphone is treated as a room microphone
  (far field) and system audio gets none
  ([#249](https://github.com/fmadore/Live-translation/issues/249)).

## Fixes

- **Gemini subtitles repeated everything said before.** Since 1.4.0, every Gemini Transcribe
  caption after the first repeated the whole session so far, because Gemini's default mode sends
  the cleaned transcript of everything heard each time. The app now captions only what each new
  transcript adds. *Correction, 24 November 2026:* this line first said 1.16.0 asked Gemini for a
  verbatim transcript, with captions about a second after a pause; that change was left out of
  1.16.0 and ships in [1.16.1](release-1.16.1.md).
- **The first words after Start or Resume** are no longer lost with cloud engines: audio captured
  while the connection opens is held and sent in order.
- **Stop while Whisper is loading** after a switch no longer waits for the model to finish
  loading, and a failed switch on a reconnecting source ends that source instead of leaving it
  stuck while the other one runs on.
- **Transcripts with only markers** can be saved (text, Markdown or WebVTT), are kept by crash
  recovery and history, and are offered for saving on close.
- **The cost estimate** stops counting a paused source even when the other one has failed, and
  prices each source by the engine it is actually on.
- **The overlay** uses each source's own caption language, and so its text direction, after a
  switch to Whisper.
- **Echo filter:** a microphone echo that carries only source text, before its translation
  arrives, is now withdrawn like any other echo.
- **DeepL** (still experimental) shows a concluded sentence's last words without waiting for the
  next sentence. **Mistral:** Pause and Stop no longer wait after Mistral closed a connection.
  Sentence ends such as `。」`, `؟` and `।` are recognised in captions and in Zoom.
- **Whisper in Chinese, Japanese and Thai:** overlaps between windows and repeated loops are now
  removed in scripts written without spaces, while short genuine repetitions are kept. A crashed
  graphics-processor helper is reported with its exit code.
- **Audio:** an unplugged microphone says that it was disconnected, and a driver crash during
  Test audio stops the test and reports it instead of leaving it running.
- **Overlay placement:** an overlay left mostly off screen by an unplugged projector is brought
  back on the next start; a window deliberately stretched across two displays is left alone.
- **macOS:** the System Audio Recording hint no longer comes back after a long pause in a call.
- **History:** a session whose delete failed stays in the list and can still be renamed; a
  changed filter disarms the batch Delete confirmation, and every delete button is marked as
  dangerous.
- **Interface:** the Whisper model list no longer misses a download that ended during a refresh;
  reduced motion now removes transitions entirely; only the button pressed shows "Starting…";
  the German interface says *Overlay* throughout, and French spacing before colons is fixed.
- **Security:** `url` 2.5.9 and `cookie` 0.18.3, for RustSec advisories RUSTSEC-2026-0592 and
  RUSTSEC-2026-0594.

## Under the surface

- One new command, `switch_source_engine`, granted to the operator window only (a test pins
  that the overlay never gets it), and one new error id, `error.sessionFallback`. A new provider
  id, `openai-transcribe`, shares the one OpenAI key. Status updates from a local engine name
  the engine, so the cost ledger and the overlay follow a switch at once.
- The release workflow checks all six assets by name before it publishes, and CI runs
  `check:languages`.
- The [privacy policy](https://github.com/fmadore/Live-translation/blob/main/docs/privacy.md),
  effective 23 November 2026, names OpenAI as a recipient of subtitle audio and of the
  names-and-terms list, and says which engines use that list. Continuing with Whisper sends
  nothing new anywhere. No permission or capability is added.
- The project has a public home page at <https://fmadore.github.io/Live-translation/>
  ([#252](https://github.com/fmadore/Live-translation/issues/252)).

None of the engine changes has been tried in a live session with a real provider failure, an
outage of more than 30 seconds or a 60-minute OpenAI handover, and none has run on a Mac; the
pull requests list what is still to be checked by hand.

## Downloads

- **Windows:** for the Microsoft Store, upload
  [Live.Translation_1.16.0.msixbundle](https://github.com/fmadore/Live-translation/releases/download/v1.16.0/Live.Translation_1.16.0.msixbundle),
  with native x64 and ARM64 packages at version **1.16.0.0**; the Store signs accepted packages.
  The separate MSIX files are for per-architecture testing, and the EXE and MSI installers for
  installing outside the Store (they are unsigned).
- **macOS 15 or later, Apple silicon:** `Live.Translation_1.16.0_aarch64.dmg`, signed with the
  developer's Developer ID and notarised by Apple.

The [release handoff](https://github.com/fmadore/Live-translation/blob/main/docs/store-updates.md#release-1160-handoff)
records the package checks and the remaining Store steps.
