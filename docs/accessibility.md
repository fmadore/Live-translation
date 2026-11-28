# Accessibility

How Live Translation & Subtitles supports operators who rely on the keyboard, a screen reader,
high-contrast themes or enlarged text—along with what is verified automatically and what requires
manual checks before release.

The operator window targets WCAG 2.2 AA and the Microsoft Store Windows accessibility checklist:
full keyboard navigation, Narrator support, Windows contrast themes and live text scaling, with
a text size of its own in Settings › App on Windows and macOS alike.

The caption overlay is handled differently by design. It is projected content shown over slides
and calls rather than an operator control surface, so its contrast is measured against its own
scrim over bright and dark backgrounds, and it intentionally opts out of Windows contrast themes
and operator text scaling (see [Windows text scaling](#windows-text-scaling)).

## What the code guarantees

| Concern | Where it lives |
| --- | --- |
| Text contrast | `src/app.css` — four text levels (bright, body, secondary, muted), every one of which clears 4.5:1 on each of the three surfaces; the muted level also clears it on every tinted wash the stylesheet names. `--faint` carries no text; it is the idle status dot at the 3:1 non-text target. No component paints with a hex literal, so every colour on screen is one the checks have seen. The light theme (`:root[data-theme='light']`, chosen in Settings → App → Theme or followed from Windows' app mode) re-declares every colour and is held to the same floors. |
| Text size | `src/app.css` — seven type roles (caption 11, small 12, body 13, label 14, title 17, heading 21, display 27px at 100%), each a multiple of `--text-scale`. Spacing is a six-step `rem` scale, so the space around text grows with it too. No component declares its own pixel size or spacing. |
| Windows text scaling | `src-tauri/src/textscale.rs` reads `UISettings.TextScaleFactor` and follows its change event; `src/lib/textScale.ts` multiplies it by Settings › App › Text size (`src/lib/textSize.ts`) and writes the product onto the document root. |
| Reflow | The operator's two columns are a container query in `em`, so the window stacks them and scrolls as one column once the operator's text no longer fits beside itself. |
| Caption reflow | Fit window measures rendered text against the overlay width and height, keeping the newest words at the chosen font size. Compact keeps a separate width control. Both layouts refit on resizing and font changes; see [caption layout](caption-layout.md). |
| Caption colours | `src/lib/captionColour.ts` — the operator picks the ink and the scrim, and every ratio is computed on the **composite**: the scrim as it is thinned under the text, over both a white and a black slide, with the halo that rings the glyph. One function settles how dim each step is painted *and* what the readout says, so the two cannot disagree. A single control puts the whole overlay appearance back to what it ships with, because a way into an unreadable palette needs a way out of it. |
| Caption typeface | `src/lib/captionFont.ts` offers faces this machine actually has, checked with a canvas width probe rather than assumed from the Windows SKU. Every stack ends in the bundled Archivo, and every face offered has a real weight at 600 — a synthesised bold at projector size is a legibility failure, not a cosmetic one. |
| Reaching the controls | The caption appearance controls are one snippet rendered in two places — the running rail and the settings panel — and the panel is opened by the one button that is in the same place in every state, so no control is discoverable only in a state the operator has to reach first. |
| Language | `src/lib/i18n/index.ts` sets `<html lang>` from the active locale, in both windows. The overlay's captions are not in the interface language, so they carry their own `lang`, pushed as `OverlayConfig.captionLanguage`; while a subtitle engine is auto-detecting it is `lang=""`, which is how HTML says the language is unknown. The operator window's live turns and transcript paragraphs carry their caption language the same way, with `dir` set right to left for Arabic, Hebrew, Persian, Urdu and Sindhi; the original speech, and a line restored from a recovery file, are `lang=""` with the direction taken from the text. |
| Focus | One `:focus-visible` ring in `src/app.css`, on every focusable element. A component may restyle it; none may remove it. Start and Rehearse make way for Stop and Pause as a run begins, and back as it ends: when the keyboard was on one of them, focus moves to the counterpart (Stop after a start, Start after a stop) instead of falling to the page. The pre-flight Whisper row's Download hands focus on the same way: to the row's sentence while the download starts (Cancel is enabled once the core reports the download), to Cancel or the sentence when that goes in turn, and back to Download after a cancel or a failure. Add marker (or Ctrl+Shift+M) puts focus in its field, and Enter or Escape gives it back to where it was; Find (or Ctrl+F) does the same with its field, and Escape there closes find and returns focus. The overlay notice's panel opens on its field and Escape returns to its button. |
| Section structure | The window has one `h1` (its name — visually hidden, because the Windows frame already prints it above the toolbar) and an `h2` per region, so Narrator's heading navigation walks the rail and the stage. |
| Announcements | A live region says that something changed state, never how far something has got, and none holds anything that changes on a timer. Two are always present in the operator window: session state and the last status message. The running rail's input labels are not live, because they flip with every pause in speech; a region beside them speaks only when a source stops with an error or has sent audio but no captions for 15 seconds, and is empty again once it recovers. Local Whisper's waiting-audio strip is not live either, because its seconds move with every progress report; it appears only once Whisper falls behind (more than 21 s waiting, two of its 10-second windows) or while it finishes, and a region beside it speaks once when finishing starts and once when a discard is confirmed. The Whisper model's status says that the model is missing, downloading or ready, once each; the download's percentage is its progress bar's value. The pre-flight checklist's Whisper row offers the same download with its own bar and Cancel, but is not a second region: the model step's status speaks for both. Under the rail's caption size, a status says once that the overlay draws captions smaller than chosen, has hidden the original speech to make room, or has no room at all, without the size it draws them at, which a resize moves; the size is in the visible notice beside it. The transcript panel announces a completed save and a copy, and has a status for the long-session warning and the missing-timing note. Background saving that keeps failing is said once per run of failures — a recovery copy on the status line, history saving in the panel's own region — and stays written beside the save badge until it works again. The rest answer the operator's own edits where they happen: glossary and filler-word changes, profile and history notices, the language picker's error and empty filter, the setup's language error, the capture picker's notes, and each stepper's new value (an `<output>`). A spending-guard status, mounted with each session (and so minutes before it can speak), says why the guard paused a cloud session and that a session has passed the chosen estimate or length, once each; the notices above the stage are its copy for the eye, with Resume and Dismiss outside the region. **Add marker** has a status that announces each marker as it goes in (*Marker added at 0:42:10: Talk 2 starts*); the transcript reader's find bar has one, mounted with the bar before any count, that says the match count after each search (*2 of 7*, *No matches*), not as the operator steps. The rail's overlay notice says what the room now reads and, once it is gone, that the captions are back. Captions in Zoom says its state (sharing, waiting, refused, unreachable) in a status; the counts of captions sent and skipped sit beside it, outside the region, because they climb with every caption. The toolbar's Zoom button prints the state as a word beside its dot and carries it in its name. On a copy installed from GitHub, the start screen's new-version line is a status mounted, empty, a render before its sentence (*Version X.Y.Z is available*, or the question whether to check), so it is said once when it arrives; its buttons sit outside it, and the same notice in Settings › App is not live. A failure that needs an answer — a rejected entry, a failed profile or history action, the discard question, a failed save when quitting — is `role="alert"`. |
| Toggle buttons | A button whose name changes with its state — Pause and Resume, Move and Done — is a plain button, not also `aria-pressed`: Narrator read "Resume, toggle button, pressed", which says the opposite of what is happening. While moving, Done is drawn as the primary action instead of a pressed one. `aria-pressed` stays on selections whose name does not change: the source, engine, interface-language and theme cards, presets, swatches and pinned languages. |
| Progress | `aria-busy` on Start, Rehearse, Stop, the audio test, and the save buttons. While a start or stop is under way, Start, Rehearse and Stop are `aria-disabled` rather than `disabled`, so the button keeps the focus it holds until the swap hands it on; a press is ignored, and a contrast theme greys them as it greys a disabled button. |
| Level meters | `role="meter"`, with `aria-valuenow` rounded to a tenth so the attribute does not change twenty times a second. |
| Modal prompts | `role="dialog"`, `aria-modal`, a Tab trap, focus on the safe answer, Escape where a safe dismissal exists, and focus returned to the opener on close. A panel whose first control is the corner × names what it opens on instead (`initialFocus`): Settings its selected tab, Captions in Zoom its link field. Escape on the first explanation of hiding to the tray closes it and leaves the window open; it used to quit. Written once in `src/lib/ModalPrompt.svelte` and shared by the two decisions, the settings panel and Captions in Zoom, so the chrome an accessible dialog is judged on cannot drift between them. When dialogs stack — a quit prompt over Settings — only the topmost handles Escape and Tab, and the one beneath gets the keyboard back when it closes. |
| Shortcuts | One table in `src/lib/shortcuts.ts`: the listener matches it, `aria-keyshortcuts` on Start and Stop, Pause, Add marker, Find and the find field announces it, and the key caps print it in the interface language's key names (`Strg+Umschalt+Leertaste`). The printed copy inside a button is `aria-hidden`, so it never becomes part of the button's name. |
| Targets and names | Every button is at least 32px tall at 100%. The colour swatches are named (Mint, Navy), not read out as hex, and a chosen swatch has its own inset ring rather than the focus ring's outline. The profile picker is named by its visible label (WCAG 2.5.3). Icons are sized in `rem`, as the type is, so at 225% an icon still matches the label beside it rather than staying 14px. |
| Motion | `prefers-reduced-motion` stops the sweep, the breathing status dot, the caret and the meter easing; a global safety net catches anything added later. |
| Contrast themes | `forced-colors: active` blocks next to the styles they correct — selection outlines, dropped gradients, the meter fill, the status dot. |

## What is checked automatically

- `npm test` → `src/lib/palette.test.ts` reads the tokens out of `app.css`, for the dark and
  the light theme alike, and fails if the light block leaves a colour undeclared or if any
  text token drops below 4.5:1 on any surface, if the muted level drops below it on any tinted
  wash, or if any non-text mark drops below 3:1. In each theme the selected edge
  (`--accent-border`) has to stand out more than the grey edge of a card under the pointer, so
  hovering never makes the wrong card look chosen. It pins the ramps (four text levels, three
  surfaces, three lines), and it reads every component stylesheet: a hex literal, a text
  colour that is not a token, or a `prefers-color-scheme` rule fails it, and so does an overlay
  toolbar that a white slide would lift above `--surface-2`. It exists because the contrast
  failures behind issue #24 arrived one shade at a time, and nothing could see them.
  `src/lib/theme.test.ts` goes further for `prefers-color-scheme`: the theme sets it on the
  WebView2 profile the overlay shares, so no file in `src` may name it except `theme.ts` and
  `app.html`, which read it to resolve System.
- `npm run test:style` records the operator window's computed styles in the light theme as
  well as the dark one, checks that a light desktop gets a light first frame before the app's
  scripts have loaded, and that the overlay keeps the dark tokens when Light is chosen.
- `npm test` → `src/lib/captionColour.test.ts` is the same guard for the overlay, which
  `palette.test.ts` cannot reach because its colours are no longer in the stylesheet. It fails
  if the shipped default drops below 4.5:1 at any step over either slide, if the default stops
  reproducing the overlay's original literals, or if a colour is written back into the
  overlay's stylesheet where the check cannot see it.
- `npm test` → `src/lib/typeScale.test.ts` reads the same stylesheets and fails if any
  component declares a bare `font-size` in pixels — a size Windows' text setting cannot reach,
  and a failure invisible on a machine at 100%. It also fails on a size outside the seven
  roles, on a small-capital label not tracked at `--tracking-caps`, on a line-height outside
  the three `--leading-*` steps (caption text excepted), and covers the clamps that stand
  between a settings event, the stored Text size and every `calc()` in the stylesheet.
- `npm run test:style` → `e2e/style/overflow.spec.ts` walks every state in all three languages
  at the minimum window at 100, 150 and 225%, and once more at the 90% Text size.
- `npm test` → `src/lib/spacing.test.ts` does the same for spacing, corners and control boxes:
  every padding, margin and gap is a `--space-*` step (or an optical nudge of a few pixels),
  every radius is a `--radius-*` token at Windows 11 geometry (4px in the page, 8px for
  dialogs and flyouts), and every control box is one of the three `--control-*` heights.
- `npm test` also covers the modal Tab trap, stacked dialogs and the save announcement.
- `npm run check` catches Svelte's own accessibility lints (missing labels, roles on the wrong
  element, click handlers without keyboard equivalents).

Text on a **tinted wash** that the stylesheet names (`--accent-wash`, `--warn-chip` and the
rest: every hue has the same 8% `-wash` and 14% `-chip`) is checked by `palette.test.ts` for the
muted level, which bounds every grey above it. Anything else composited — a coloured label on
its own wash (amber on amber), or text over a gradient — is measured against the rendered
window instead: run `npm run dev`, then walk the DOM in the browser preview, comparing each
element's computed colour against its composited background.

## Release checklist (manual, on Windows)

Run these against the **Store MSIX**, not a dev build — package identity changes how Windows
treats the window.

1. **Keyboard only.** Unplug the mouse. Reach every control in the rail, the checklist, the
   transcript panel and the tray menu. Start a session, place the overlay, stop it, save the
   transcript, and quit. Focus order follows the visual order and the ring is always visible.
2. **Narrator.** Same walk. Heading navigation works (`H`). The session state, status messages and a
   completed save are each announced once, not repeatedly. The level meters do not chatter.
   The input status is silent while captions flow and speaks once when a source goes stale or
   stops with an error. Pause, Resume, Move and Done read as plain buttons.
3. **Contrast theme.** Settings → Accessibility → Contrast themes → Aquatic and Desert. The
   selected mode, source, language and engine stay distinguishable; the Start button is
   readable; the level meter is visible; the status pill keeps an edge. The overlay's audience
   view keeps its own colours — that is deliberate.
   Repeat the pass with Settings → App → Theme on **Light**, and once on **System** while
   switching Windows between light and dark app mode: the operator window and its title bar
   follow, and the overlay's move-mode toolbar stays dark. In Light, the selected mode, source,
   engine and preset are as easy to pick out as in Dark, and a card under the pointer never
   looks more chosen than the selected one. Under a contrast theme Windows reports light app
   mode, so System resolves to Light; the contrast theme repaints the window either way.
4. **Caption colours over real slides.** With the overlay placed, put a **bright** deck
   behind it and then a **dark** one, and read the captions from the back of the room in both.
   Include a finished line and a trailing previous line, which are the dimmest text on screen
   and the steps the check is decided by. The reading beside the control is computed against
   exactly these two extremes; this is the step that confirms the model matches the room.
5. **Text scaling.** Settings → Accessibility → Text size. Walk 100%, 150% and 225% with the
   window at its 980 × 660 minimum and again maximised. The window follows the slider without
   a restart. Nothing is clipped or overlapping; the rail's cards keep their proportions; the
   two columns stack into one scrolling column when they no longer fit, and come back when the
   window is widened. The overlay does not change — see below.
   Then Settings → App → Text size, with the slider back at 100%: walk 90% and 150%, and once
   at 150% with the slider at 150% (225%, the ceiling) and at 200% (still 225%). The window
   follows each choice at once, and the choice is still there after a restart. At 90%, the
   smallest text (chips, key caps, kickers) is still readable at the operator's distance.
   **On a Mac**, which has no slider for the app, walk every Text size choice the same way,
   at the minimum size and maximised.
6. **Display scaling.** 150% and 200%, and a mixed-DPI two-monitor setup with the overlay on
   the second display.
7. **Reduced motion.** Settings → Accessibility → Visual effects → Animation effects off. No
   sweep, no breathing dot, no blinking caret.
8. **Accessibility Insights for Windows** on the operator window, and **Accessibility Insights
   for Web** (or `axe` DevTools) against `npm run dev` for the DOM-level rules.
9. **Minimum window size.** Resize to 980 × 660. Nothing overlaps and nothing is clipped; both
   columns scroll rather than compress.
10. **Responsive captions.** Follow the [caption layout matrix](caption-layout.md#release-verification):
    narrow/wide and short/tall windows, 20/38/96px fonts, both origins, long words, and
    Fit window / Compact switching. Verify the selector’s keyboard operation and Narrator
    label in English and French. Reset must return to Fit window; the hidden measurement
    paragraph must not be announced. If the window cannot fit a whole line at the chosen
    font size, enlarge it or reduce the font size.

## Windows text scaling

Windows' *Make text bigger* setting does not reach web content inside WebView2 — it is
[an open WebView2 request](https://github.com/MicrosoftEdge/WebView2Feedback/issues/1662),
not something this app gets for free. Display scaling works, because that scales the whole
window. So the core reads the factor and the operator window applies it:

1. `src-tauri/src/textscale.rs` reads `UISettings.TextScaleFactor`. The operator window asks
   for it once as it boots — a command rather than only an event, so someone who needs 225%
   text is not shown a frame of 9.5px type before the window could subscribe. The module then
   subscribes to `TextScaleFactorChanged` and emits every later change, so moving the slider
   moves the window without a restart.
2. `src/lib/textScale.ts` clamps the factor to the slider's own 1 … 2.25, multiplies it by
   the operator's [Text size](#text-size-in-the-app), clamps the product to 0.9 … 2.25 and
   writes it to `--text-scale` on the document root. Both ends clamp: the value crosses a JSON
   event boundary, and a `NaN` there would invalidate every `calc()` in the stylesheet at once.
   Each input is checked on its own, so a corrupt read still leaves the other one in force.
3. `src/app.css` multiplies the whole type ramp by it, and scales the root font size by it as
   well. The ramp is what makes the text grow; the root size is what makes the *layout* grow
   with it, because every gutter and width that has to hold text is expressed in `em`.

Honouring a text-size setting is not only a question of type: a 225% caption inside a 380px
rail is not accessible, it is clipped. So the rail, the meter labels, the transcript's gutter,
the key field and the dialog are all measured in `em`, and the two-column layout is a container
query — `@container window (min-width: 53.75em)`, the 380px rail plus the narrowest stage that
still shows a caption line. Being in `em`, that threshold rises with the text, so the columns
stack into one scrolling column where they would otherwise start clipping, and return when the
operator widens or maximises the window. At 100% it resolves to 860px, inside the 980px minimum
window, so the normal layout is untouched.

**The overlay opts out**, for the same reason it opts out of contrast themes: its captions are
not chrome on the operator's screen, they are projected content whose size the operator sets
for the room, with its own control and its own `--fs`. An accessibility setting on the
operator's PC has no business resizing what an audience is reading, and neither has the
app's own Text size.

Verified in a browser preview at 980 × 660 — the window's minimum — with the factor forced to
every step of the Windows slider: no horizontal overflow, no clipped box, and no overlapping
region at 225%. The native walk is item 5 of the release checklist.

### Text size in the app

Settings › App › **Text size** offers 90%, 100% (the default), 115%, 130% and 150%
([#283](https://github.com/fmadore/Live-translation/issues/283)). It is a percentage *of the
system's* text size, not a replacement for it: on Windows the window is drawn at the slider's
factor times the choice, so 130% with the slider at 150% is 195%. That is why there is no
separate *System* choice: 100% is exactly the window as Windows sizes it, and a second entry
for the same window would only ask the operator to tell two identical things apart. On macOS,
whose webview reports no text size, the choice is the only factor there is. It applies at
once, is kept in the operator window's local storage (`appearance.textSize`), and does not
reach the overlay, whose captions keep their own size control; it is not webview zoom, which
would also scale the window's chrome and pointer targets.

The top stays at 2.25, the range the `em` breakpoints are verified at, so 150% of 150% is the
ceiling and anything past it is held there. The one step below 100% was checked against the
layout before it was offered. Everything that holds text is in `em` or `rem`, so at 0.9 the
whole window shrinks in proportion: the stacking threshold falls to 774px (two columns at
every window size the app allows), spacing and control heights shrink with the type, and the
smallest control box is still 28.8px, above WCAG 2.2's 24px target minimum. What does not
shrink is drawn in pixels on purpose (borders, corner radii, the focus ring), and only gains
room. The type ramp's caption step renders at 9.9px, which is why the floor is 90% and not
lower. `overflow.spec.ts` walks every state at 90% in all three languages.

## Feature checks

[Operator shortcuts](usability.md#keyboard-controls) apply only with the operator window
focused, outside editable inputs and dialogs. The opt-in system-wide Pause and Show/hide
overlay keys ([usability](usability.md#system-wide-pause-and-overlay-keys)) are listed under
their switch in Settings › App with `aria-keyshortcuts` on each key cap; a status region, in
the page before anything is in it and empty otherwise, says which combination another
application holds. Native screen-reader, contrast-theme, 225%
Windows text scaling and mixed-DPI acceptance for the final package are tracked in the
[current release checklist](store-updates.md#release-1120-handoff).

### Settings and date fields

Settings uses a roving tab stop with Left/Right, Home and End navigation. The dialog
frame and tab bar stay in place while the selected tab's body scrolls; the body can also take
keyboard focus. Opening Settings puts focus on the selected tab. Switching tabs starts the new
panel at its top. The close control stays outside the scrolling body. Modal Tab cycling excludes negative-tabindex controls.

Date fields have explicit labels, localised format hints and `aria-invalid` with a linked
localised error for invalid input. Calendar buttons have translated accessible names;
the implementation's hidden native date input is excluded from the accessibility tree
and normal Tab order. Packaged WebView calendar and screen-reader acceptance are still pending.

### Caption language selector (#78)

The translation selector uses a combobox and filtered listbox with active-descendant,
selected and disabled states. Keyboard and pin interaction are documented in
[language coverage](language-coverage.md#keyboard-and-accessibility). Test in all three UI
locales, including an unsupported pinned language after a provider change. The list has
bounded scrolling and scrolls into the operator viewport on opening; the Start/Stop bar
remains reachable. Confirm the active option and the disabled reason with Narrator in the
native build before release. Automated keyboard tests are not a substitute for that check.

Caption paragraphs inherit their selected language and explicit RTL/LTR direction; unknown
subtitle languages use auto direction. Stable reading uses logical start alignment so Arabic
starts at the right edge. Verify Japanese and Arabic glyph fallback on Windows; the
bundled Archivo face falls back to the system for uncovered scripts.

### Markers, find and the overlay notice

Markers are `h3` headings inside the transcript log and the History reader, so Narrator's
heading navigation moves between talks. Find matches are drawn with the CSS Custom Highlight
API, which adds no element: Narrator reads the text as before, and selection and Copy are
unchanged. In a contrast theme the matches take the system's `Mark`/`MarkText` colours and the
current one `Highlight`/`HighlightText`. Check with Narrator that the count is spoken after
typing pauses, that Enter and Shift+Enter move the current match without speaking every step,
and that Escape lands back on Find or wherever Ctrl+F was pressed. The overlay's notice, like
its captions, is projected content and has no live region.

### Whisper controls and Pause

Test keyboard access to Whisper's spoken-language and model selectors, Download/Cancel/Remove,
and the model's ready/error status. Verify that the pending-audio duration and finishing state
are readable, that Narrator says "Finishing the transcript" and "Downloading model" once rather
than with every second or percent, and that the discard confirmation keeps focus inside the
active dialog. Check the
Pause/Resume labels and Ctrl+Shift+P without losing access to Stop. These native checks remain
part of final-package acceptance, including Narrator and enlarged Windows text.
