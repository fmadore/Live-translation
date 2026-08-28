# Release notes

Paste-ready text for the **GitHub release** body, which is a different audience from the Store
listing and wants a different register.

Store copy is written for someone deciding whether to install; a release body is read by
someone who already has the app, or who is looking at the source. It can name versions, link
issues and say what changed under the surface. It should still not be a commit log — the
commit log is one click away and nobody wants it twice.

Every previous release published with an empty body. That was a small, steady loss: the
releases page carries the unsigned installers the README points people at, and it said nothing
about what they were installing.

Written before tagging, pasted into the release GitHub creates. The
[`Release installers`](../.github/workflows/release.yml) workflow does not fill the body in.

---

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
