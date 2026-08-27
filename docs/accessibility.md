# Accessibility

What this app owes an operator who cannot rely on colour, a mouse, or small text, how much of
that is checked automatically, and what has to be checked by hand before a release.

The target is WCAG 2.2 AA for the operator window, plus the Windows-specific behaviour the
Microsoft Store accessibility checklist asks for: keyboard operation, Narrator, contrast
themes, and text scaling.

The overlay is treated differently on purpose. It is not a control surface — it is the caption
being projected into a room, over slides nobody in this app controls. Its contrast is measured
against the scrim it paints, not against the operator's system palette, and it opts out of
contrast themes (see *Contrast themes* below).

## What the code guarantees

| Concern | Where it lives |
| --- | --- |
| Text contrast | `src/app.css` — one text ramp, every step of which clears 4.5:1 on every surface token. `--faint` carries no text; it is the idle status dot at the 3:1 non-text target. |
| Text size | `src/app.css` — one type ramp, every step of which is a multiple of `--text-scale`. No component declares its own pixel size. |
| Windows text scaling | `src-tauri/src/textscale.rs` reads `UISettings.TextScaleFactor` and follows its change event; `src/lib/textScale.ts` writes it onto the document root. |
| Reflow | The operator's two columns are a container query in `em`, so the window stacks them and scrolls as one column once the operator's text no longer fits beside itself. |
| Focus | One `:focus-visible` ring in `src/app.css`, on every focusable element. A component may restyle it; none may remove it. |
| Section structure | The window has one `h1` (its name, in the title bar) and an `h2` per region, so Narrator's heading navigation walks the rail and the stage. |
| Announcements | Two `role="status"` regions in the operator window — session state and the last status message — plus one in the transcript panel for a completed save. They hold nothing that changes on a timer. |
| Progress | `aria-busy` on Start, Stop, the audio test, and both save buttons. |
| Level meters | `role="meter"`, with `aria-valuenow` rounded to a tenth so the attribute does not change twenty times a second. |
| Modal prompts | `role="dialog"`, `aria-modal`, a Tab trap, focus on the safe answer, Escape where a safe dismissal exists, and focus returned to the opener on close. |
| Motion | `prefers-reduced-motion` stops the sweep, the breathing status dot, the caret and the meter easing; a global safety net catches anything added later. |
| Contrast themes | `forced-colors: active` blocks next to the styles they correct — selection outlines, dropped gradients, the meter fill, the status dot. |

## What is checked automatically

- `npm test` → `src/lib/palette.test.ts` reads the tokens out of `app.css` and fails if any
  text token drops below 4.5:1 on any surface, or any non-text mark below 3:1. This is the
  regression guard: the contrast failures that prompted issue #24 arrived one shade at a time,
  and nothing could see them.
- `npm test` → `src/lib/typeScale.test.ts` reads the same stylesheets and fails if any
  component declares a bare `font-size` in pixels — a size Windows' text setting cannot reach.
  It is the same kind of guard for the same reason: the failure is invisible on a machine
  sitting at 100%. It also covers the clamp that stands between a settings event and every
  `calc()` in the stylesheet.
- `npm test` also covers the modal Tab trap and the save announcement.
- `npm run check` catches Svelte's own accessibility lints (missing labels, roles on the wrong
  element, click handlers without keyboard equivalents).

Text on a **tinted wash** — a selected engine's mint background, a warning chip — composites
two colours that the stylesheet never names together, so no static check can see it. That is
measured against the rendered window instead:

```bash
npm run dev
```

then, in the browser preview, walk the DOM comparing each element's computed colour against
its composited background. The last run of that audit checked 52 text elements in the idle
operator window with no failures; it is how the `/hr` unit on a selected engine was caught at
4.36:1.

## Release checklist (manual, on Windows)

Run these against the **Store MSIX**, not a dev build — package identity changes how Windows
treats the window.

1. **Keyboard only.** Unplug the mouse. Reach every control in the rail, the checklist, the
   transcript panel and the tray menu. Start a session, place the overlay, stop it, save the
   transcript, and quit. Focus order follows the visual order and the ring is always visible.
2. **Narrator.** Same walk. Headings navigate (`H`). The session state, status messages and a
   completed save are each announced once, not repeatedly. The level meters do not chatter.
3. **Contrast theme.** Settings → Accessibility → Contrast themes → Aquatic and Desert. The
   selected mode, source, language and engine stay distinguishable; the Start button is
   readable; the level meter is visible; the status pill keeps an edge. The overlay's audience
   view keeps its own colours — that is deliberate.
4. **Text scaling.** Settings → Accessibility → Text size. Walk 100%, 150% and 225% with the
   window at its 980 × 660 minimum and again maximized. The window follows the slider without
   a restart. Nothing is clipped or overlapping; the rail's cards keep their proportions; the
   two columns stack into one scrolling column when they no longer fit, and come back when the
   window is widened. The overlay does not change — see below.
5. **Display scaling.** 150% and 200%, and a mixed-DPI two-monitor setup with the overlay on
   the second display.
6. **Reduced motion.** Settings → Accessibility → Visual effects → Animation effects off. No
   sweep, no breathing dot, no blinking caret.
7. **Accessibility Insights for Windows** on the operator window, and **Accessibility Insights
   for Web** (or `axe` DevTools) against `npm run dev` for the DOM-level rules.
8. **Minimum window size.** Resize to 980 × 660. Nothing overlaps and nothing is clipped; both
   columns scroll rather than compress.

## Windows text scaling

Windows' *Make text bigger* setting does not reach web content inside WebView2 — it is
[an open WebView2 request](https://github.com/MicrosoftEdge/WebView2Feedback/issues/1662),
not something this app gets for free. Display scaling works, because that scales the whole
window. So the core reads the factor and the operator window applies it:

1. `src-tauri/src/textscale.rs` reads `UISettings.TextScaleFactor`. The operator window asks
   for it once as it boots — a command rather than only an event, because a window has to lay
   itself out before it could have subscribed to anything, and someone who needs 225% text
   should not be shown a frame of 9.5px type first. The module then subscribes to
   `TextScaleFactorChanged` and emits every later change, so moving the slider moves the
   window without a restart.
2. `src/lib/textScale.ts` clamps the factor to the slider's own 1 … 2.25 and writes it to
   `--text-scale` on the document root. Both ends clamp: the value crosses a JSON event
   boundary, and a `NaN` there would invalidate every `calc()` in the stylesheet at once.
3. `src/app.css` multiplies the whole type ramp by it, and scales the root font size by it as
   well. The ramp is what makes the text grow; the root size is what makes the *layout* grow
   with it, because every gutter and width that has to hold text is expressed in `em`.

That third point is the part worth keeping: honouring a text-size setting is not only a
question of type. A 225% caption inside a 380px rail is not accessible, it is clipped. So the
rail, the meter labels, the transcript's gutter, the key field and the dialog are all measured
in `em`, and the two-column layout is a container query — `@container window (min-width:
53.75em)`, which is the 380px rail plus the narrowest stage that still shows a caption line.
Being in `em`, that threshold rises with the text, so the columns stack into one scrolling
column at the point where they would otherwise start clipping, and return when the operator
widens or maximizes the window. At 100% it resolves to 860px, comfortably inside the 980px
minimum window, so the normal layout is untouched.

**The overlay opts out**, for the same reason it opts out of contrast themes: its captions are
not chrome on the operator's screen, they are projected content whose size the operator sets
for the room, with its own control and its own `--fs`. An accessibility setting on the
operator's PC has no business resizing what an audience is reading.

Verified at 980 × 660 — the window's minimum — at every step of the Windows slider: no
horizontal overflow, no clipped box, and no overlapping region at 225%.
