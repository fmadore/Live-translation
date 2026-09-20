# Store screenshots

The Microsoft Store listing is per-language, so each language needs its own screenshots. They
live here rather than only in Partner Center, so that "do these still match the build?" is a
question the repository can answer instead of one somebody has to remember.

Both the English and French captures were refreshed during the September 2026
installed-MSIX testing. On 8 September, the user confirmed that all screenshots
had been updated for the 1.2.2 Store submission. The current sets contain five
screenshots per language. They are historical captures, not ready for the 1.4.1 submission.
Re-capture the changed overlay and appearance controls in both languages from the final MSIX;
review the idle, running and contrast captures against that package too.
The unpackaged Local Test app does not replace MSIX screenshot verification.

`de/` does not exist yet and should not be created until there is something true to put in it:
the German captures come from the final MSIX of the release that ships the German interface,
with the interface set to German. Same five filenames, same order — the order belongs to the
listing, not to the language.

```text
en/    English listing
fr/    French listing
de/    German listing — not captured yet
```

Five per language, named for the order they are uploaded in — that order is what a visitor
scrolls through, so it belongs to the listing rather than to whoever captured them:

| File | What it shows |
| --- | --- |
| `1-idle.png` | The pre-flight screen while idle: provider key, microphone, overlay placement, hourly cost. |
| `2-running.png` | A running demo: Demo status, a moving meter, elapsed time, and a caption. |
| `3-overlay.png` | The overlay in placement mode: drag handles, the size readout, and the stand-in caption set in the chosen appearance. |
| `4-appearance.png` | Caption appearance: Fit window / Compact, size, typeface, caption colour and backing; line width appears only in Compact. |
| `5-contrast.png` | The contrast readout, caught naming a step that falls below its target rather than sitting on "Readable". |

Two of these describe the product and three argue for it. Shots 4 and 5 exist because the
appearance panel and its contrast readout are the part of this app that has no equivalent in
the category, and a listing that stops at "it shows captions" never says so. Shot 3 is the
placement pass, because deciding where the captions land before the room fills is the part of
running an event this app is actually for.

Two shots came out of this set rather than in. A saved transcript states its file path and
line count in the description already, and a list of finished lines makes a dull picture. A
second provider screen repeated the first one’s layout at a single field’s difference, which
is a slot spent showing the same thing twice.

**One current set, overwritten in place.** Re-capture over the existing files rather than adding
a folder per version: the history keeps every past set, and the working tree should contain the current verified set. It currently holds historical
1.2.2 captures. A folder per version would grow the repository by a few megabytes every
release to preserve something nobody looks at twice.

Capture from a **sideloaded MSIX** on Windows, not a dev build — package identity changes how
Windows draws the window — at the display scaling the app is actually used at.

Store rules that decide how these are framed, not just whether they are accepted:

- **1366×768 minimum**, `.png`, under 50 MB. 4K (3840×2160) is supported.
- **Keep anything that matters in the top two-thirds.** The Store draws its own text overlays
  across the bottom third, so a caption or a status row parked down there is the part that
  gets covered.
- **No logos, wordmarks or marketing text burned into the image.** Each screenshot carries a
  Description field in Partner Center instead — 200 characters, and it doubles as the alt text
  a screen reader announces. The wording for all ten is in
  [`../store-listing.md`](../store-listing.md#screenshots).
- Ten desktop screenshots are the ceiling and Microsoft recommends five to eight, so five
  sits at the low end of the recommendation with room to grow.

The French set is captured with the interface language set to French, which is a setting in the
app rather than a Windows one: title bar → gear → **Langue de l'interface**. The whole run is
captured that way, rather than one French frame inside an otherwise English set. The caption
language is independent of the interface language — that is what screenshot 2's description
says in both listings, since a set captured entirely in one language cannot show it.

Full rules, and where these sit in a submission, are in
[`../store-updates.md`](../store-updates.md#screenshots).

For 1.4.1, recapture the new Stable reading and Hide filler words controls and add a saved-session history image in each listing language. Capture only disposable demo transcripts, with no private meeting text. The existing images have not been refreshed by this release preparation.

For 1.4.1 also capture Meeting profiles, the expanded appearance preview/presets, and a titled
history session with search filters. Use disposable demo content. Impeccable browser fixtures
verify layout but are not Store screenshots or evidence of native provider behavior.

For the 1.4.1 interface, capture the profile picker above step 01, Captions with presets and
bright/dark previews, Reading controls, and the History list/detail view. Verify the App tab
and localized tray separately. Do not use browser-preview captures for the Store.
