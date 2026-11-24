# Store screenshots

Captured 24 November 2026 from a release build of **v1.16.1**: the tag's code, built for ARM64
and run unpackaged under the side-by-side *Local Test* identity, with the interface set to each
listing language and Settings › App › Theme set to **Dark**. The images show what the Store
package shows; the one difference, the question about checking GitHub for new versions, which
the Store package never asks, was answered *No thanks* before capturing and appears in none of
them. Display: 3440×1440 at 125% scaling; the window was sized to exactly 1920×1080 and captured
pixel for pixel. Upload each language's set in filename order, with the matching
[caption](../store-listing.md#screenshots). This set replaces the 1.7.1 one of 21 October 2026,
which predated the Live and Transcript tabs, *Zoom captions*, the graphics-processor switch and
the OpenAI and DeepL engine cards.

| File | Shows |
| --- | --- |
| `1-whisper.png` | Subtitles mode after a real Local Whisper session: system audio, spoken language set, Local Whisper chosen, all pre-flight checks green, transcript below. |
| `2-overlay.png` | The click-through overlay locked low over a neutral slide during a live session (Stable reading, white captions on a 95% black backing, 1500 px wide). Cropped to 2194×1234 to drop the screen edge. |
| `3-bilingual.png` | The Live tab of a Gemini live-translation session with two caption languages, the original speech shown above one of them. |
| `4-appearance.png` | Settings › Captions: presets, size, typeface, colours, contrast and the bright/dark slide previews. |
| `5-reading.png` | Settings › Reading: Stable reading and the editable filler-word list. |
| `6-history.png` | Settings › Transcript history: a titled session found by search, with copy/export controls and the transcript. |

## Content

All speech is a synthesized, neutral talk on community gardens (Windows' Zira, Hortense and
Hedda voices, at a slightly slow rate) played through the default output and captured as system
audio. The slide behind the overlay is a plain page made for these images. Nothing shows a real
meeting, API key, account or personal window. The script that speaks the talk must be saved as
UTF-8 *with* a byte-order mark: Windows PowerShell 5.1 reads a script without one as ANSI, and
the voices then read every accented letter as two symbols ("ü" as "Ã¼"), which Whisper and
Gemini transcribe as nonsense.

| Set | Whisper speech | Bilingual: speech → captions | Overlay: speech → caption |
| --- | --- | --- | --- |
| `en/` | English | French → English + German | English (Whisper) |
| `fr/` | French | English → French + German | English → French (Gemini) |
| `de/` | German | French → German + English | English → German (Gemini) |

Whisper ran on the Base model, on the processor, so its transcripts keep a few recognition
slips (for example "parler déjà d'un partage" in the French set). Gemini wording is as
returned, not edited ("Mais merci" heard as "But thank you" in the English bilingual image).

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
layouts. It is an illustration generated with OpenAI GPT Image (`gpt-image-2.6-flare`, high
quality, 24 November 2026), rendered at 2560×1440 and resized to 1920×1080; the prompt and
model are in `super-hero-art.png.json`, kept beside it. It replaced a glowing-tube rendering
of the same idea, chosen against a Gemini (`gemini-nano-banana-2.1`) rendering of the same
prompt.

It extends the icon's wave-to-words mark as a quiet studio object: graphite bars shaped like a
speech wave lie down at the centre into four rows of frosted-glass caption bars, one in the
accent mint. Microsoft's rules for this image exclude text, app UI, devices and flags, and ask
for the key content in the centre and out of the bottom third. It has no text, so one file
serves every language.

The same render makes the home page's hero (`docs/assets/hero.jpg`) and the art of the
repository's social preview (`.github/social-preview-art.jpg`): crops with the floor outside
the sculpture masked and the black point raised, so the ground is pure black and the page
blends it into the dark band with `mix-blend-mode: lighten`. The sidecar's `postprocess`
records the steps. To change the art, edit the prompt from the sidecar, draft at low quality,
render the final once, and regenerate all three files and the social preview together.

## Recapturing

Re-capture whenever the UI these show changes, from a sideloaded MSIX or a release build of the
release's tag rather than a dev build, and keep the same order and names in every language.
Launch the app on an unlocked desktop: an overlay created while Windows is locked can stay
invisible until it is moved and locked again. Since 1.9.0 the window follows Windows' light or
dark app mode by default, so set Settings › App › Theme to **Dark** before
capturing, whatever mode the capture PC is in: the whole set has to show one theme, and the
current shots are dark. Check the current Microsoft
[screenshot requirements](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/screenshots-and-images)
and the [Store rules](../store-updates.md#screenshots) first.
