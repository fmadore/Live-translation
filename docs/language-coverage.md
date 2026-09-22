# Caption language coverage (issue #78)

This describes version 1.5.0. The older 1.4.2 installers have English/French caption buttons.
GitHub publication was requested with the live verification gaps below explicitly retained.

| Engine | Caption language control |
| --- | --- |
| Gemini Live Translate | 78 searchable translation targets |
| OpenAI Realtime Translate | 13 searchable targets; endpoint code probe pending |
| Built-in demo | English/French script buttons only |
| Mistral / Gemini Transcribe | Auto-detection hint, no language selector |

Names use the interface's locale without changing that locale. Search accepts English,
localized and native names, codes and aliases; accents/case are ignored. Prefixes rank
before substrings. Empty search groups favourites first in pin order. Pin/unpin uses the
star next to each row. Preferences survive restart independently of session options.

An unsupported favourite stays visible and disabled with an explanation. Switching engines
does not change the chosen target: Start and Rehearse remain blocked until a supported
language is selected. The Rust core rejects unsupported targets too. Subtitle auto-detection
does not consult the retained target. Changing back to translation keeps the operator's choice.

F2 swaps the first two favourites while translation is stopped. From outside that pair it
selects the first. With fewer than two pins, or either pin unsupported, it does nothing.
Rehearsal uses French for an English translation target and English for other targets.
Both demo scripts and rehearsal recordings remain statically restricted to English/French.

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

## Verification status — 2026-09-22

- Gemini's live table was rechecked today: 78 languages, with `no`/`nb` on one row. The app
  sends `no`, and searches `nb` as an alias. Full mirror: [Gemini API notes](gemini-live-api.md).
- OpenAI's cookbook still lists 13 languages. The reproducible configuration probe is in
  `src-tauri/src/openai/language_probe.rs`; see [OpenAI API notes](openai-realtime-api.md).
  Its first run found no saved key. User-assisted credential setup and rerun are pending.
- Validation: 356 frontend tests and 74 Rust tests pass (2 network probes remain opt-in).
  Type checking, production build, generated-catalog parity, Prettier, Rust formatting and
  Clippy pass. Windows enumeration tests require running outside the sandbox.
- Tests cover generated catalog parity, persistence/defaults, search ranking, favourites,
  F2, keyboard controls, unsupported options, history and caption direction. Rust tests
  verify provider memberships and every demo script's packaged WAV.
- Browser checks passed at 980×660 in English, French and German: the open selector fits the rail,
  pinned Swahili survives the Gemini → OpenAI switch with a disabled reason, and the
  selected value is preserved. Japanese/Arabic browser preview glyphs render, and Arabic
  stable reading aligns at the right edge with `dir="rtl"`. Reproduce with `/overlay?language=ja`
  or `/overlay?language=ar`; `he`, `fa`, `ur` previews are also available.
- Real speech into a non-EN/FR target on both engines, native Narrator announcements, and
  final MSIX checks are **pending**. Keep #78 open and complete the live code probe and real-speech checks before Store submission.

## Live acceptance procedure

1. Save OpenAI and Gemini keys in the app; never put keys in screenshots or test reports.
2. Run the OpenAI code probe documented above. Record every accepted/rejected code, including
   all Portuguese/Chinese variants, and regenerate the catalog if its memberships change.
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
