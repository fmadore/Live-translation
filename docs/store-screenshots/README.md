# Store screenshots

The Microsoft Store listing is per-language, so each language needs its own screenshots. They
live here rather than only in Partner Center, so that "do these still match the build?" is a
question the repository can answer instead of one somebody has to remember.

```text
en/    English listing
fr/    French listing
```

Four per language, named for the order they are uploaded in — that order is what a visitor
scrolls through, so it belongs to the listing rather than to whoever captured them:

| File | What it shows |
| --- | --- |
| `1-idle.png` | The idle screen: Built-in demo, Demo audio, Free, and the Start demo button. |
| `2-running-english.png` | A running English demo: Demo status, a moving meter, elapsed time, and an overlay caption. |
| `3-running-french.png` | A running French demo, showing a French caption. |
| `4-provider.png` | Optional. A live provider's configuration, showing its key requirement and estimated cost. |

**One current set, overwritten in place.** Re-capture over the existing files rather than adding
a folder per version: the history keeps every past set, and the working tree keeps exactly what
is on the listing now. A folder per version would grow the repository by a few megabytes every
release to preserve something nobody looks at twice.

Capture from a **sideloaded MSIX** on Windows, not a dev build — package identity changes how
Windows draws the window — at the display scaling the app is actually used at. The Store
minimum is 1366×768.

The French set is captured with the interface language set to French, which is a setting in the
app rather than a Windows one: title bar → gear → **Langue de l'interface**. The caption
language is independent of it, so screenshot 3 wants a French *caption*, which comes from the
demo language, not from the interface.

Full rules, and where these sit in a submission, are in
[`../store-updates.md`](../store-updates.md#screenshots).
