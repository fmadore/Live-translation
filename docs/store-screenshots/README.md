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

## What has changed since this set

The set predates the Live and Transcript tabs of 1.8.0 and the *Zoom captions* toolbar button
of 1.12.0. 1.13.0 changes what some of the images would show:

- `1-whisper.png`: the setup gains *Use the graphics processor* under the model, with a line
  under it saying where the latest session decoded, and the model list gains Large v3 Turbo.
  With *Both* as the source, *Leave out the call heard by the room mic* appears under it.
- Any image showing Start while it cannot be pressed: it now has a neutral surface, a border
  and muted text at full opacity instead of a faded green.
- `4-appearance.png`: each custom-colour button, labelled *Custom…* (*Personnaliser…*,
  *Benutzerdefiniert…*), now sits at the end of its own swatches and holds one line in French
  and German.

None of this makes the current images wrong about what the app does. Whether to retake them
for 1.13.0 is the maintainer's decision, recorded in the
[1.13.0 handoff](../store-updates.md#release-1130-handoff); a retake keeps any Zoom window and
Zoom's artwork out of the images.

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

## Mac App Store (English)

`mac/en/` holds the Mac App Store set, captured on 11 November 2026 on an M2 Mac mini (macOS
26.7.3) from the **sandboxed build** of the `fix/macos-first-run` branch (the store build's
entitlements, ad hoc signed), so no support link appears, as App Review requires. The listing is
in English only (see [mac-app-store.md](../mac-app-store.md#the-listing-english-us)). Upload in
filename order.

**1280 × 800**, one of the four sizes App Store Connect takes for a Mac (1280 × 800, 1440 × 900,
2560 × 1600, 2880 × 1800): the display is 1920 × 1080 at 1×, and macOS would not make the window
taller than the space above the Dock, so 1440 × 900 could not be captured pixel for pixel. The
window was sized to exactly 1280 × 800 and captured as it is drawn; the images are then flattened
onto white, because App Store Connect refuses screenshots with an alpha channel and a window
capture has transparent rounded corners.

| File | Shows |
| --- | --- |
| `1-whisper.png` | Subtitles after a real local Whisper session on system audio: spoken language English, Base model ready, every pre-flight check green, the transcript below. |
| `2-overlay.png` | The locked, click-through overlay over a neutral slide during that session (Fit window). A full-screen capture cropped to 16:10 (1728 × 1080) and scaled to 1280 × 800. |
| `3-bilingual.png` | A Gemini live-translation session into English and German, with the original speech shown above the English. |
| `4-appearance.png` | Settings › Captions: presets, size, typeface, colours, backing and the bright and dark slide previews. |
| `5-reading.png` | Settings › Reading: Stable reading and the editable filler-word list. |
| `6-history.png` | Settings › Transcript history: a titled session found by search, with copy and export controls and its transcript. |

**Content.** The speech is a synthesized talk on community gardens, the subject of the Windows
set, made with macOS's voices (Samantha in English for Whisper, Thomas in French for Gemini) and
played through the speakers with `afplay`, captured as system audio. The slide is a plain
illustration made for these images. Whisper ran on the Base model, so its transcript keeps a few
slips ("Gardner's Eag"); Gemini's wording is as returned. Nothing shows a real meeting, API key,
account or personal window. In `6-history.png` macOS capitalised the search word as it was typed.

## Super hero art

`super-hero-art.png` is the listing's 16:9 Super hero art, uploaded under **Trailers and
additional assets**, where the Store shows it at the top of the listing and in promotional
layouts. It is an illustration generated with OpenAI GPT Image (`gpt-image-2.5-flare`, high
quality, 28 October 2026), rendered at 2560×1440 and resized to 1920×1080; the prompt and
model are in `super-hero-art.png.json`, kept beside it.

It extends the icon's wave-to-words mark: a speech wave splits at the centre into five
caption lines, each tinted for a different language. Microsoft's rules for this image
exclude text, app UI, devices and flags, and ask for the key content in the centre and out
of the bottom third. It has no text, so one file serves every language. To change it, edit
the prompt from the sidecar, draft at low quality, and keep the new sidecar with the image.

## Recapturing

Re-capture whenever the UI these show changes, from a sideloaded MSIX rather than a dev
build, and keep the same order and names in every language. Since 1.9.0 the window follows
Windows' light or dark app mode by default, so set Settings › App › Theme to **Dark** before
capturing, whatever mode the capture PC is in: the whole set has to show one theme, and the
current shots are dark. Check the current Microsoft
[screenshot requirements](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/screenshots-and-images)
and the [Store rules](../store-updates.md#screenshots) first.
