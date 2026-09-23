# Release notes

Paste-ready text for the **GitHub release** body, which is a different audience from the Store
listing and wants a different register.

Store copy is written for someone deciding whether to install; a release body is read by
someone who already has the app, or who is looking at the source. It can name versions, link
issues and say what changed under the surface. It should still not be a commit log — the
commit log is one click away and nobody wants it twice.

Written before tagging, pasted into the release GitHub creates. The
[`Release installers`](../.github/workflows/release.yml) workflow does not fill the body in.
Set it once the workflow has finished, so no build job recreates the release first:
`gh release edit vX.Y.Z --notes-file <body>.md`. The v1.4.0–v1.5.1 bodies below were applied
this way on 23 September 2026; earlier releases without prepared text here stay empty.

---

## v1.5.1

23 September 2026. [Release body](release-1.5.1.md) and
[Store handoff](store-updates.md#release-151-handoff). Supersedes the unsubmitted 1.5.0 as the
Store candidate.

## v1.5.0

22 September 2026. [Release body](release-1.5.0.md) and
[Store handoff](store-updates.md#release-150-handoff).

## v1.4.2

GitHub release: [v1.4.2](https://github.com/fmadore/Live-translation/releases/tag/v1.4.2),
21 September 2026. See the [Store handoff](store-updates.md#release-142-handoff).

```markdown
Start and Stop stay within reach throughout a meeting.

- Session controls now sit in a persistent bar directly below the app header.
- Start and Stop occupy the same position; setup, live captions and the completed transcript scroll below them.
- The controls remain visible in narrow windows, with Rehearse beside Start and the existing keyboard shortcut nearby.
- Readiness checks, audio capture, translation and transcript storage behavior are unchanged.

Includes the interface, profile, history and localized date improvements from 1.4.1. Thanks to @valentinrabot for the meeting feedback that informed these improvements.

For Microsoft Store submission, use Live.Translation_1.4.2.msixbundle (x64 + ARM64), not the EXE or MSI installer. Native acceptance and Store certification are tracked separately from GitHub publication.
```

## v1.4.1

GitHub release: [v1.4.1](https://github.com/fmadore/Live-translation/releases/tag/v1.4.1),
20 September 2026. Installer builds and native/Store acceptance are tracked separately
in the [1.4.1 handoff](store-updates.md#release-141-handoff).

```markdown
A clearer operator interface for setting up and running meetings.

- Settings is organized into Captions, Reading, Transcript history and App, with keyboard navigation and an explicit note that changes apply immediately.
- Meeting profiles sit above setup. Save a new setup, load a profile, rename it or confirm deletion from its row.
- Browse saved sessions beside their transcript, with aligned filters, Clear filters and grouped copy/export/delete actions.
- Choose visible appearance presets and curated colour swatches, with bright/dark previews and contrast feedback.
- Consistent controls, readable small type, explicit units and overlay labels make the app easier to scan. The tray menu follows the English, French or German interface and shows session status.
- Corrected accented labels and retained the existing opt-in history and local caption-cleanup behavior.
- Settings keeps its window and tab bar stationary when switching tabs; only the content scrolls. Profile controls now share aligned edges and heights.
- History date fields use explicit English, French and German format hints, with calendar selection and validation of real dates, including leap years.

Includes the meeting profiles, reading controls, history search and Gemini finalization improvements from 1.4.0. Thanks to @valentinrabot for the meeting feedback and suggestions that informed this work.

For Partner Center, use Live.Translation_1.4.1.msixbundle with x64 and ARM64 packages after native acceptance and release workflow verification. GitHub publication does not imply Store certification.
```

## v1.4.0

GitHub release: [v1.4.0](https://github.com/fmadore/Live-translation/releases/tag/v1.4.0),
19 September 2026. Installer build and native/Store acceptance are tracked separately in the
[release handoff](store-updates.md#release-140-handoff).

```markdown
Set up recurring meetings faster and make live captions easier to follow.

- **Meeting profiles:** save setup, caption appearance and overlay placement locally. Load while stopped, with device checks and safe placement on connected displays. Keys and application process IDs are excluded.
- **Reading controls:** keep completed Fit window/Compact captions for 2–30 seconds, optionally group interim updates every 450 ms, and preview appearance on bright/dark backgrounds. Standard, Large room and High contrast presets are available.
- **Find saved meetings:** name sessions and search titles, captions and source text; filter by dates and caption language. Titles update without replacing newer recorded lines. Long titles wrap at enlarged text sizes.
- **Live feedback and shortcuts:** distinguish recent audio, captions, quiet inputs and reconnecting sources. Use Ctrl+Shift+Space to start/stop, Ctrl+Shift+O to show/hide the overlay, and Ctrl+Shift+Up/Down to resize captions while the operator window has focus. Input fields and dialogs retain their normal keys.
- **Gemini subtitle corrections:** Smart transcription was already enabled. Final text now takes precedence over simultaneous interim text, and empty finals retract speculative filler-only captions. The optional local overlay filler filter is separate; saved text is the text returned by the provider.

Thanks again to **@valentinrabot** for the detailed meeting feedback and suggestions in [#79](https://github.com/fmadore/Live-translation/issues/79), [#80](https://github.com/fmadore/Live-translation/issues/80), and [#81](https://github.com/fmadore/Live-translation/issues/81), which informed these improvements. This acknowledges feedback, not implementation.

The custom filter-list proposal remains open in [#85](https://github.com/fmadore/Live-translation/issues/85). This update adds no new scroll-back or return-to-live controls.

Implementation verification: 331 frontend tests, 73 Rust tests, formatting, Svelte/TypeScript checks, Clippy and production build passed; the paid provider test remains opt-in. Impeccable browser checks covered compact/desktop layouts, French/German, enlarged text, profile/history/live controls and the two-source overlay. Native package acceptance and Store certification are tracked separately.

The expected Partner Center artifact is **Live.Translation_1.4.0.msixbundle**, containing x64 and ARM64 packages. Verify its manifests and digest after the installer workflow completes.
```

## v1.3.0

Published on 19 September 2026: [GitHub release](https://github.com/fmadore/Live-translation/releases/tag/v1.3.0). See [release handoff](store-updates.md#release-130-handoff).

```markdown
This release makes long meetings easier to read and keeps optional local copies of past sessions.

- **Transcript history:** opt in to progressively save finalized captions locally. Browse previous sessions by date, duration and language information, read raw captions and available source text, copy, export Markdown/text/SRT/VTT, or delete a session. History stays until deleted and is separate from the recovery copy.
- **Stable reading:** left-aligned captions stay at the top of their area and advance by whole lines as it fills. Fit window and Compact remain available.
- **Hide filler words:** optional, conservative cleanup of hesitation sounds in the overlay. Raw transcripts, history and exports remain unchanged. No extra provider requests are made.
- **German interface:** choose Deutsch, English or Français independently of the caption language. Live translation remains English–French.
- Updated privacy documentation, English/French/German controls and Store submission copy. Updated the compatible devalue dependency to 5.9.4 to resolve a moderate security advisory.

Thanks to **@valentinrabot** for the detailed meeting feedback and suggestions in [#79](https://github.com/fmadore/Live-translation/issues/79), [#80](https://github.com/fmadore/Live-translation/issues/80), and [#81](https://github.com/fmadore/Live-translation/issues/81). This acknowledgment is for feedback, not implementation.

Validation: 314 frontend tests and 69 Rust tests passed locally, alongside formatting, Svelte/TypeScript checks, Clippy, the production build and browser layout checks. The billable provider test remains opt-in. Final packaged-app acceptance and Microsoft Store certification are separate checks; this GitHub release is not a claim of Store submission or certification.

For Partner Center, use **Live.Translation_1.3.0.msixbundle**, containing x64 and ARM64 packages. Individual unsigned MSIX files are for local signing and sideload testing.
```

## v1.2.4

GitHub release copy. See [release handoff](store-updates.md#release-124-handoff) for
package testing and publication status. The local test executable reports 1.2.4; final MSIX verification is pending.

```markdown
Captions now adapt to the overlay window. Widen it to fit more words per line, or make
it taller to show more recent caption context. Font size stays under your control.
New overlays start as a shallow strip, and Align to bottom resets an enlarged overlay
to roughly two lines per visible speaker.
Choose Compact in the caption appearance controls to retain a limited line width.

Development tooling now uses Vitest 5.0.0; all 287 frontend tests pass.

Thanks to @valentinrabot for the feedback and responsive-layout suggestion in
[#77](https://github.com/fmadore/Live-translation/issues/77).
```

## v1.2.3

The developer-signed ARM64 MSIX **1.2.3.0** was installed on 8 September 2026.
The user confirmed Save As and application selection. The later layout correction
was checked in English and French. Full audio isolation, device-transition and
packaged x64/NSIS matrices remain unverified; see the
[release handoff](store-updates.md#release-123-handoff).

```markdown
Save transcripts to your chosen folder with the native Windows Save As dialog. The last
successful folder is remembered. Export plain text, Markdown, SubRip (SRT), or WebVTT (VTT).
Cancelling a save keeps the transcript available and cancelling from the quit prompt keeps
the app open. Captions received during a save remain marked as unsaved.

For system audio, choose an output device or one open application. Application capture
includes its child processes and notifications; browsers may include multiple tabs.
Refresh the application list before selecting a source. If the application closes, its
capture stops without switching to all system audio. Select it again after restarting it.

The start and rehearsal buttons now share an aligned row, with readable explanations below.

SRT/VTT timing follows the caption session clock, including provider latency, rather than
alignment to a recording. Older recovered transcripts without timing can still be saved
as text or Markdown. Application selection must be repeated after relaunching the app.

Implements [#26](https://github.com/fmadore/Live-translation/issues/26) and
[#27](https://github.com/fmadore/Live-translation/issues/27).
```

## v1.2.2

The user confirmed both fixes on the installed 1.2.2.0 ARM64 MSIX on 8 September 2026.
The broader hardware matrix in `audio-device-testing.md` remains pending.

```markdown
This release brings a bilingual interface, caption appearance controls, and more reliable audio capture and transcript recovery.

- Use the interface in English or French, independently of the caption language.
- Adjust caption typeface, size, colours, backing, and line width, with a contrast readout and an overlay placement preview.
- Refresh audio devices without restarting and choose the output used for system-audio capture. Device changes update the lists automatically; disconnected sources offer an explicit retry or default-device fallback.
- Read earlier transcript lines without incoming captions pulling you to the bottom. Jump to latest resumes following, and long transcripts use shorter paragraphs on screen and in exports.
- Fatal provider failures release audio capture and finalize the last caption while a healthy second source can continue.
- Audio tests no longer accumulate unused buffers, and recovery snapshots are saved atomically with ordered writes and deletion.
- Overlay controls report failed appearance updates and preserve the confirmed placement mode.

- Refreshing audio devices no longer fails when a worker thread already uses another COM apartment mode.
- A recoverable microphone buffer underrun or overrun no longer ends an audio test or live session. Disconnections and invalid streams still stop the affected source.

The two reported Windows audio errors were retested successfully with the installed 1.2.2.0 ARM64 MSIX. Automated verification: 263 frontend tests and 63 Rust tests passed; the billable live-provider test remains opt-in.

For Microsoft Store submission, use `Live.Translation_1.2.2.msixbundle`, which contains both x64 and ARM64 packages. The individual `.msix` files are unsigned packages for local signing and sideload testing.
```

## v1.2.1

Do not tag or submit this release until the desktop checks in
[`app-review-2026-09-05.md`](archive/app-review-2026-09-05.md#verification-and-remaining-work)
have been completed.

```markdown
This update improves transcript reading and fixes audio and recovery failures.

- Refresh audio devices without restarting and choose which output system-audio capture uses. Device changes update the lists automatically; a disconnected source reports an error and offers an explicit session retry or default-device fallback.
- Read earlier transcript lines without new captions pulling you to the bottom. Jump to latest resumes following the conversation.
- Long transcripts use shorter paragraphs on screen and in text and Markdown exports, without dropping any text.
- A fatal provider error now releases its audio capture and finalizes the last caption. A healthy second audio source can continue.
- Audio tests no longer accumulate unused audio buffers. Microphone failures end the test and display its error correctly.
- Optional recovery snapshots are replaced atomically, and queued writes cannot recreate a snapshot after it has been discarded.
- Overlay controls report failed appearance updates and preserve the confirmed move mode when a command fails.

The operator's session, preflight, quit/tray, recovery, and overlay logic now have separate controllers, with shared caption appearance controls. Regression coverage includes 263 frontend tests and 61 Rust tests; the billable live-provider test remains opt-in.
```

## v1.2.0

```markdown
The interface speaks French, captions can be styled for the room they are read in, and both are reachable from a settings panel that does not require a running session.

## The app is bilingual

The whole interface is available in French as well as English, and the interface language is independent of the caption language — a francophone operator can run an English event, or the reverse. It follows Windows' own language on first run and remembers an explicit choice after that.

## Captions can be styled

Typeface, text colour, the colour and strength of the backing behind it, text size, and how long a line runs before it wraps.

Beside the colour controls is a contrast reading, and it is measured the way the audience actually sees a caption: the slide shows through the backing, so the ratio is computed on the composite — the slide at both extremes, the backing as it is thinned under the text, the halo that rings the glyph, and the ink. A ratio measured against the backing's own swatch would be a number that flatters every palette and predicts nothing.

Two things fell out of writing that check. A fixed black halo is what swallows dark ink rather than what rescues it, so the halo follows the ink. And sRGB alpha is not perceptually uniform, so a fixed dimming table would have warned on every dark palette — the steps are floors now, dimming as far as the design asks or as far as the 4.5:1 bar allows, whichever is less dim.

Defaults are unchanged to the digit, so an existing setup looks exactly as it did.

## Settings

A gear in the title bar opens a panel holding the caption appearance, the interface language and the notification-area preference. The appearance controls stay in the session rail too, because raising the size mid-session is the thing operators actually do — they are the same controls over the same settings, not a copy.

Placement mode doubles as the preview: the stand-in caption the overlay shows while being positioned is set in whatever the panel is choosing, so a palette gets judged on the projector rather than on a swatch.

## Accessibility

The operator window has had a pass against WCAG 2.2 AA and the Windows accessibility checklist: full keyboard operation with a visible focus ring everywhere, Narrator support with headings that navigate and announcements that do not chatter, Windows contrast themes, and it now follows the Windows text-size setting up to 225%. Captions carry their own language attribute, separate from the interface, so a screen reader does not read French in an English voice.

## Also

- Caption lines have a configurable measure, and the amount of a long streaming turn that is shown moves with it — widening a caption no longer quietly changes how much of the slide it covers.
- Every caption is stamped with the time it happened at.
- The overlay window is no longer trusted with the whole application's command surface.
- The bundle identifier is `io.github.fmadore.live-translation`. It used to be `org.stias.live-translation`, which named the venue of the workshop the app was first built for as though it were the developer — it never was. Nothing is asked of you: a saved provider key is moved to the new name on first use and the old credential removed, and a crash-recovery file left by a previous version is still found and still offered.

**Windows 11 with the Microsoft Edge WebView2 Runtime.** [Get it from the Microsoft Store](https://apps.microsoft.com/detail/9PFB8LR3RR9X) for a signed build that updates itself. The installers below are unsigned and meet a SmartScreen warning on first launch — choose More info, then Run anyway.
```
