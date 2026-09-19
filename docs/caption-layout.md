# Caption layout

Prepared for **1.2.4**, following [issue #77](https://github.com/fmadore/Live-translation/issues/77).
Thanks to **@valentinrabot for the feedback and responsive-layout suggestion**.
This acknowledgment is for feedback, not implementation.

## Fit window

Open Settings → Caption layout → **Fit window**. In French, use
**Disposition des sous-titres → Adapter à la fenêtre**. The same control is available
in the live session's caption appearance controls.

Fit window is the default when no layout preference has been saved. Widening the overlay
allows more words per line; increasing its height allows more recent caption context.
The selected font size stays unchanged. Captions remain bottom-aligned with margins,
and the newest words take priority when the available space shrinks.

Use **Place the overlay** to move or resize it, then **Lock into place** to see the
audience view again. Placement mode shows positioning controls instead of live captions.
The locked overlay remains click-through. **Align to bottom / Aligner en bas** also resets
the height to a shallow strip sized for roughly two lines per currently visible source
(160 logical pixels at the default font with one source). New overlays start at 160 pixels
high. Drag the edges taller when you deliberately want more reading context. Resizing does not create text: a short caption
or a new session with little history can still leave empty space.

When both sources have visible captions, system audio appears above the microphone,
with labels, and they share the available height. A single visible source uses the
available height without an origin label. If the window is too short to fit even one
complete line at the selected font size, enlarge it or reduce the font size.

## Compact

Choose **Compact** to retain a limited line width. **Line width / Largeur des lignes**
appears only in this mode. The default is 30ch, adjustable from 20ch to 60ch; the
character budget remains proportional to this width. A smaller window can further
limit the text that fits.

Switching layouts keeps the chosen font, colours and compact width. The layout preference
survives relaunch and is shared by the settings panel and live rail. **Reset appearance**
restores Fit window and the other appearance defaults; it does not move the overlay.

## Stable reading

Choose **Stable reading** for a teleprompter-style display ([issue #79](https://github.com/fmadore/Live-translation/issues/79)).
Text is left-aligned and anchored to the top of each source's allocated area. New words wrap
naturally. Once the area fills, the viewport advances one complete rendered line at a time;
there is no continuous centering, animated scrolling, or moving caret. Earlier turns retain
the same colour so finishing a turn does not visually relocate the reading point.

This mode retains the session's caption context in memory, including during pauses. Stop
clears it. The two sources remain independent and share the window. Resizing, changing fonts,
adding a second source, or provider corrections can still reflow text. The backing is even
across the reading area so top-aligned text is readable over light content. Fit window and
Compact retain their existing alignment, colour treatment, and idle expiry.

## Hide filler words

**Hide filler words** is off by default ([issue #80](https://github.com/fmadore/Live-translation/issues/80)).
It removes exact hesitation tokens `um`, `uh`, `erm`, `hmm`, `euh`, and `heu` from the overlay
and repairs adjacent punctuation. Meaningful words such as “so”, “well”, “like”, and “oh”
remain. Substrings, uppercase abbreviations, quoted tokens, and unfinished streamed tokens
are preserved. This is a conservative text filter, not semantic speech analysis: it cannot
reliably distinguish every intentional hesitation from other uses in every language.

The operator transcript, saved history, recovery copy, and exports retain the original text.
No provider setting or additional request is involved. Both caption options persist across
relaunch, synchronize with the live controls, and reset with the other appearance settings.

## Recent context and the transcript

Fit window retains a bounded recent history in memory for each source. Completed text is
dimmed ahead of the current turn. Growing the window can reveal more of that available
history; shrinking it removes older words from the audience view first. Very long unbroken
tokens can be shortened so their newest characters still fit.

In Fit window and Compact, the existing fade-out timers remain: captions clear 4 seconds after the last final update,
or 3 seconds after a stalled interim update. Clearing also removes that source's reading
context. This is a live-caption view, not a scrolling transcript viewer. The full operator
transcript and exported files are unaffected by overlay trimming or expiry. No additional
audio capture, provider request, disk history or telemetry is introduced.

## Release verification

Completed on 14 September 2026:

- 287 frontend tests, Svelte checks, formatting and the production frontend build passed.
- Browser checks covered long captions at 600 × 260, 1200 × 600 and 1200 × 850,
  showing more text as width/height increased at the same font size.
- Compact mode, EN/FR settings and large captions were checked in the browser.
- The optimized native ARM64 executable built and launched successfully as a separate
  **Live Translation Local Test** install. The updated executable reports 1.2.4 and includes the shallow bottom-alignment preset.

User confirmation of caption behavior in the installed app is pending. Before publishing
1.2.4, run this matrix against the final x64 and ARM64 MSIX packages:

| Check | Expected result |
| --- | --- |
| Clean preferences | Fit window is selected; compact width is hidden. |
| Wider, same height and font | Longer lines use the available width without horizontal clipping. |
| Taller, same width and font | More available recent context is visible; newest words remain visible. |
| Shrink and enlarge again | Oldest words disappear first; retained context returns when space permits. |
| Short turns and long continuous speech | Recent short turns accumulate; long text remains bounded and responsive. |
| Microphone, system, then Both | Each source works alone; simultaneous rows remain labelled and fit together. |
| One source expires | Its row clears; the other source gains the available space. |
| Long words, URLs, punctuation and emoji | No horizontal overflow or broken Unicode in shortened tokens. |
| Caption fonts at 20, 38 and 96px | Text refits after size/typeface changes; no partly clipped lines. |
| Windows scaling | Check 100%, 150%, 200% display scaling and moving between monitors with different DPI. |
| Compact and reset | Width is adjustable only in Compact; switching retains it; reset returns to Fit window. |
| Relaunch | Layout and appearance persist; the fresh session does not resurrect old captions. |
| Placement and locking | Move/resize, Enter, Escape and click-through still work. |
| Idle and export | Existing fade-out works; full transcript remains available and exports correctly. |
| English and French | Labels, keyboard access and layout choice work in both the rail and settings. |
| Light/dark content | Current text and dimmed history remain readable over slides and video calls. |

The built-in demo provides a free first check. Use live speech for continuous long-turn,
two-source and meeting tests; those use the operator's chosen provider and account.
See [accessibility](accessibility.md#release-checklist-manual-on-windows) and the
[release handoff](store-updates.md#release-124-handoff) for the remaining release gates.

## Reading pace and preview

Fit window and Compact keep finished captions for a configurable **2–30 seconds** (default
4 seconds). This is a per-source reading pause after its latest completed caption, not a
minimum display time that delays new speech. Unfinished, stalled text still expires after
3 seconds. Stable reading retains its existing session-long context behavior.

**Immediate** remains the default. **Steadier** presents the most recent interim hypothesis
every 450 ms per source, without waiting indefinitely for silence. Final text and turn
transitions flush promptly; it can still revise already displayed words. This setting only
changes presentation: it does not buffer audio, delay transcription storage or add requests.

The expandable **Appearance preview** shows sample text at the selected font size over bright
and dark slides. Standard, Large room and High contrast presets change appearance; the Reset
button also restores the default reading pace, hold time and filler filter. The real overlay
may have different dimensions, so finish placement on the presentation display.

Gemini subtitles already request Google's Smart transcription. The optional local filler
filter is additional and cannot restore fillers removed by the provider. Raw text means the
provider's returned text, not a guaranteed verbatim record of speech.
