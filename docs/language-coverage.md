# Caption languages

Which spoken and translation languages each engine supports (current for 1.16.0). The dated
verification notes below record the latest live checks rather than fresh endpoint probes.

| Engine | Caption language control |
| --- | --- |
| Gemini Live Translate | 78 searchable translation targets |
| OpenAI Realtime Translate | 13 searchable targets; endpoint code probe pending |
| DeepL Voice | 42 searchable targets; spoken language detected automatically |
| Local Whisper (subtitles) | 99 spoken languages or automatic detection; same-language subtitles |
| Local Whisper (translation) | English only, translated from 99 spoken languages or automatic detection |
| Built-in demo | English/French script buttons only |
| Gemini Transcribe | 71 of Whisper's spoken languages or automatic detection; same-language subtitles |
| Mistral | Auto-detection hint, no language selector |
| OpenAI transcription | Optional spoken-language hint from the 99 Whisper languages, or automatic detection; same-language subtitles |

Choosing a caption language never changes the English, French or German interface; see
[localisation](localization.md).

Unsupported favourites stay visible in the list, disabled with a brief explanation. Searching also
turns up languages the current engine does not offer, listed below supported matches and disabled
with the same explanation—so typing "zul" under DeepL explains why rather than showing "No
matching languages". Switching engines preserves your chosen target: Start and Rehearse stay
blocked until you pick a supported language, and the Rust core rejects unsupported targets as
well. Cloud subtitle auto-detection ignores the saved translation target, while Whisper and Gemini
Transcribe use the spoken-language selector; switching back to translation restores your earlier
target. Gemini Transcribe offers the 71 Whisper languages listed in Google's table (see
[the spoken-language hint](gemini-live-api.md#the-spoken-language-hint)); a Whisper language
outside that set stays saved for Whisper and falls back to automatic detection under Gemini.

F2 swaps your first two pinned favourites while translation is stopped (or selects the first if
neither is active). With fewer than two pins, or if either pin is unsupported by the current
engine, F2 does nothing. Rehearsal plays French for an English target and English for all other
targets; demo scripts and rehearsal audio are English and French only.

## Catalogue

`src/lib/languages.json` is the single source of truth for English fallback names, endonyms,
provider memberships, source URLs and verification dates. After editing it, run
`npm run generate:languages` to regenerate the TypeScript union and Rust enum checks;
`npm run check:languages` (run by `npm run verify` and CI) catches drift. Norwegian uses `no`,
with `nb` as a search alias rather than a duplicate row. Portuguese regional variants and Chinese
script tags stay separate.

Display names come from `Intl.DisplayNames` in the active interface locale, falling back to the
catalogue's English name rather than a bare code. Search ignores combining accents and case,
matching codes, English or localised names, endonyms and aliases, with prefix matches ranked
ahead of substrings. An empty search lists pinned favourites first in pin order; the star beside
each row toggles pinning. Favourites live in `language.favourites`, independent of session
options, and persist across restarts. UI catalogues contain only the selector's own copy.

Overlay captions carry the selected BCP-47 code and their own text direction. Arabic, Hebrew,
Persian, Urdu and Sindhi are RTL; unknown auto-detected subtitles use `dir="auto"`, and Stable
reading aligns to the start edge. Font stacks keep the system fallback after bundled Archivo
for scripts the bundled font does not cover.

## Keyboard and accessibility

- Focus/click opens the combobox and selects its displayed text for replacement.
- Type to filter; Up/Down navigate, Home/End jump, Enter selects a supported row.
- Escape discards the query and restores the selected value. Clicking outside or moving
  focus out closes the list. Tab does not trap focus.
- Tab from the input reaches the active row's pin button; Enter/Space toggles it. Other pin
  buttons remain clickable without adding 78 stops to the keyboard sequence.
- The input exposes expanded/controlled/active-descendant state. Options expose selected
  and disabled state; unsupported reasons are part of the option name. Pin buttons are
  siblings of the listbox, never nested inside its options.

Follow the [Windows accessibility checklist](accessibility.md) for Narrator and packaged
contrast/theme checks. Automated DOM assertions cannot establish what Narrator speaks.

## Verification status

- 2026-09-22: Gemini's table rechecked (78 languages, `no`/`nb` on one row; see
  [Gemini API notes](gemini-live-api.md#target-language-catalog)). OpenAI's cookbook still
  lists 13; the configuration probe found no saved key and a rerun with user-assisted
  credentials is pending (see [OpenAI API notes](openai-realtime-api.md#target-language-catalog)).
- 2026-10-04: local ARM64 Whisper fixture inference passed in English with detection and in
  French with an explicit language. That does not establish accuracy in all 99 languages;
  see the [Whisper acceptance plan](local-whisper.md#validation).
- 2026-10-21: Mistral subtitles moved from Voxtral Realtime 2602 (13 languages) to Realtime 3,
  which Mistral counts at 45 languages and dialects without publishing the list. Realtime 3
  accepts a two-letter `language` pin that the app does not send yet; see the
  [Mistral API notes](mistral-realtime-api.md#session-features-not-used-yet).
- 2026-11-23: OpenAI transcription (`gpt-live-transcribe`) added for subtitles. OpenAI
  publishes no language list for it. The spoken-language selector's choice goes as the one
  entry of `languages` (Whisper's `jw` as ISO `jv`); OpenAI rejects codes it does not support,
  which stops the source with its message, and no code beyond `en` and `fr` has been tried.
  See the [OpenAI transcription notes](openai-transcribe-api.md).
- Tests cover generated catalogue parity, persistence and defaults, search ranking, favourites,
  F2, keyboard controls, unsupported options, history and caption direction. Rust tests
  verify provider memberships and every demo script's packaged WAV.
- Browser checks at 980×660 in English, French and German: the open selector fits the rail,
  pinned Swahili survives the Gemini → OpenAI switch with a disabled reason, and the selected
  value is preserved. Japanese and Arabic preview glyphs render, and Arabic stable reading aligns
  at the right edge with `dir="rtl"`. Reproduce with `/overlay?language=ja` or
  `/overlay?language=ar`; `he`, `fa` and `ur` previews are also available.
- **Pending:** real speech into a non-EN/FR target on both engines, native Narrator
  announcements and final MSIX checks. Keep #78 open and complete the live code probe and
  real-speech checks before Store submission.
- 2026-11-11: DeepL Voice added for translation with 42 of DeepL's 48 Voice targets, from
  [DeepL's supported-languages page](https://developers.deepl.com/docs/voice/supported-voice-languages)
  (`sources.deepl` in the catalogue). Not in the catalogue yet: Irish, Maltese, Tagalog, and the
  Canadian French, Latin American Spanish and British English variants. DeepL detects the
  spoken language automatically among the languages it transcribes itself; spoken languages it
  transcribes through a partner are not auto-detected. Real speech through DeepL has not been
  tried yet; see the [DeepL Voice API notes](deepl-voice-api.md).

## Live acceptance procedure

1. Save OpenAI and Gemini keys in the app; never put keys in screenshots or test reports.
2. Run the [OpenAI code probe](openai-realtime-api.md#target-language-catalog). Record every
   accepted and rejected code, including all Portuguese and Chinese variants, and regenerate the
   catalogue if its memberships change.
3. In a native development build, choose Live translation → German → Gemini → microphone.
   Start, speak a new English or French sentence, confirm German in the overlay and transcript,
   then Stop. Repeat with OpenAI. Record the input sentence, output and provider/date; a
   fixture or synthetic speech run does not satisfy this check.
4. Choose Japanese, then Arabic on Gemini. Check actual glyph fallback, wrapping and stable
   reading's right-hand start edge for Arabic. Repeat in Fit window and Compact layouts.
   Verify Hebrew/Persian/Urdu direction. The system font fallback must render readable glyphs.
5. At 980×660, check the open selector in English/French/German. Navigate and pin with only
   the keyboard, switch providers with an unsupported target, and confirm Start is blocked.
6. Run Narrator through the combobox; verify active option, selected/disabled state and
   the unsupported reason. Complete the existing packaged accessibility checklist.
