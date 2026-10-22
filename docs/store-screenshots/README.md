# Store screenshots

Captured 21 October 2026 from the sideloaded **1.7.1.0 ARM64 MSIX**
(`49346FMadore.LiveTranslationSubtitles`), with the interface set to each listing language.
Display: 3440×1440 at 100% scaling; the window was sized to exactly 1920×1080 and captured
pixel for pixel. Upload each language's set in filename order, with the matching
[caption](../store-listing.md#screenshots).

| File | Shows |
| --- | --- |
| `1-whisper.png` | Subtitles mode after a real Local Whisper session: system audio, spoken language set, Base model ready, all pre-flight checks green, transcript below. |
| `2-overlay.png` | The click-through overlay locked low over a neutral slide during a live session (Stable reading). Cropped to 2194×1234 to drop the screen edge. |
| `3-bilingual.png` | A Gemini live-translation session with two target languages, the original speech shown above one of them. |
| `4-appearance.png` | Settings › Captions: presets, size, typeface, colours, contrast and the bright/dark slide previews. |
| `5-reading.png` | Settings › Reading: Stable reading and the editable filler-word list. |
| `6-history.png` | Settings › Transcript history: a titled session found by search, with copy/export controls and the transcript. |

## Content

All speech is a synthesized, neutral talk on community gardens (Windows' Zira, Hortense and
Hedda voices) played through the default output and captured as system audio. Nothing shows
a real meeting, API key, account or personal window.

| Set | Whisper speech | Bilingual: speech → captions | Overlay: speech → caption |
| --- | --- | --- | --- |
| `en/` | English | French → English + German | English (Whisper) |
| `fr/` | French | English → French + German | English → French (Gemini) |
| `de/` | German | French → German + English | English → German (Gemini) |

Whisper ran on the Base model, so its transcripts keep a few recognition slips (for example
in the German history detail). Gemini wording is as returned, not edited.

## Recapturing

Re-capture whenever the UI these show changes, from a sideloaded MSIX rather than a dev
build, and keep the same order and names in every language. Since 1.9.0 the window follows
Windows' light or dark app mode by default, so set Settings › App › Theme to **Dark** before
capturing, whatever mode the capture PC is in: the whole set has to show one theme, and the
current shots are dark. Check the current Microsoft
[screenshot requirements](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/screenshots-and-images)
and the [Store rules](../store-updates.md#screenshots) first.
