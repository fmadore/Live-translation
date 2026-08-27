# Localization

How the app says things, in which language, and what has to happen for a new one.

The rule the whole design follows: **the interface language is not the caption language.** An
operator running a French-language event may be working in English, or the other way round.
Nothing here touches `options.targetLanguage`, and the language selector is deliberately in
the rail's app-level section rather than in the numbered setup sheet.

## Where the words live

| | |
| --- | --- |
| `src/lib/i18n/en.ts` | The English catalog, and the shape every other catalog has to match. `Messages` is `typeof en`. |
| `src/lib/i18n/fr.ts` | French. Typed as `Messages`, so a missing, extra or wrongly-shaped key fails `npm run check`. |
| `src/lib/i18n/index.ts` | The `locale` store, `t` (the active catalog), `localeTag`, detection and persistence. |
| `src-tauri/src/errors.rs` | The ids the core reports failures under. It never writes a sentence. |
| `src/lib/errors.ts` | Turns an id plus its technical detail into one line the operator can read. |

Components read `$t.some.key`. Anything with a parameter is a function — `lines(n)`,
`isDefault(name)` — so word order and plural rules stay the translator's decision rather than
being frozen into a `{0}` placeholder by English.

## What the core does and does not say

A command that fails rejects with `{ id, detail? }`, not with a sentence:

```rust
AppError::with(id::MIC_CAPTURE, format!("{error:#}"))
```

The catalog owns the wording for `error.micCapture`; the detail is the Windows or provider
text, which is never translated and is appended in parentheses. An id the catalog has never
heard of falls back to the detail alone, so a core that has learned a new failure still says
something an operator can act on.

Status events carry the same shape, and the status store keeps it structured until render
time — which is what lets a message already on screen re-word itself when the language
changes.

## Choosing the language

First run follows Windows (`navigator.languages`, matched on the primary subtag, so `fr-CA`
and `fr-FR` are both French). After that the explicit choice wins and is persisted in
`localStorage` under `ui.locale`.

The overlay is a **separate webview**, so the operator pushes the choice to it through the
existing overlay-config event rather than relying on a `storage` event crossing two windows.

## What is checked automatically

- `npm run check` — the type system is the completeness check. `fr` is `Messages`.
- `npm test`:
  - `i18n.test.ts` — key-for-key parity, matching parameter counts, nothing blank, each
    language named in its own language, detection and persistence, and that `t` follows the
    store.
  - `errors.test.ts` — reads the ids out of `src-tauri/src/errors.rs` and fails if a failure
    the core can report has no sentence in either catalog.
  - `transcript.test.ts` — the saved file's headings and date follow the language it is given.

## Adding a language

1. Copy `fr.ts`, rename the export, set `locale.name` (in that language) and `locale.tag`.
2. Add it to `Locale`, `LOCALES` and `CATALOGS` in `index.ts`, and to `detectLocale`.
3. Widen `OverlayConfig.locale` in `src/lib/types.ts` and `src-tauri/src/types.rs`.
4. Run `npm run check` and `npm test`; both will name anything missing.
5. Walk the operator window and the overlay at the minimum window size — a translation is
   routinely 20–30% longer than its English source, and this UI is dense.

## Still to do for French (issue #23)

- `fr.ts` carries the English wording. Translating it is the second half of the issue.
- The French Store copy in `docs/store-listing.md` predates the translated UI and has to be
  reviewed against it, with French screenshots.
- A native French speaker reviews the app and the Store copy before publication.
