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
4. **Text scaling.** Settings → Accessibility → Text size → 225%. See the caveat below.
5. **Display scaling.** 150% and 200%, and a mixed-DPI two-monitor setup with the overlay on
   the second display.
6. **Reduced motion.** Settings → Accessibility → Visual effects → Animation effects off. No
   sweep, no breathing dot, no blinking caret.
7. **Accessibility Insights for Windows** on the operator window, and **Accessibility Insights
   for Web** (or `axe` DevTools) against `npm run dev` for the DOM-level rules.
8. **Minimum window size.** Resize to 980 × 660. Nothing overlaps and nothing is clipped; both
   columns scroll rather than compress.

## Known gap: Windows text scaling

Windows' *Make text bigger* setting does not reach web content inside WebView2 — it is
[an open WebView2 request](https://github.com/MicrosoftEdge/WebView2Feedback/issues/1662),
not something this app can pick up for free. Display scaling works, because that scales the
whole window.

Honouring it means reading `UISettings.TextScaleFactor` on the Rust side, handing it to the
front end, and having a type scale that can respond — which means moving the windows off fixed
pixel sizes. That is tracked as the remaining part of issue #24 and is not in the pass that
introduced this file.

Until then, an operator who needs larger text has display scaling and the overlay's own
caption-size control, and the pre-flight checklist and Start button stay reachable at 225%
display scaling because the window's minimum size is expressed in logical pixels.
