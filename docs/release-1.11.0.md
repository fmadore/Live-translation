# Live Translation & Subtitles 1.11.0

Live Translation & Subtitles stays free, with every captioning, translation and export feature
included. This release adds an optional way to support its development, and fixes an empty band
that could appear at the bottom of the operator window.

## Support development

- **A Buy Me a Coffee link.** If the app is useful to you, you can help fund its development
  on [Buy Me a Coffee](https://buymeacoffee.com/fmadore). The link sits at the end of
  Settings › App, under *Support development*, and in a short credit line at the foot of the
  start screen. Clicking it opens the page in your default browser, where Buy Me a Coffee
  handles any payment.
- **Entirely optional.** Supporting unlocks nothing and changes nothing in the app: no feature,
  limit or reminder depends on it. The link is never shown during a session, and the app never
  prompts you to support it. Cloud providers still bill their own usage separately.
- The link is a real link for Narrator and the keyboard, and says that it opens in your browser,
  in English, French and German
  ([#182](https://github.com/fmadore/Live-translation/pull/182), part of
  [#180](https://github.com/fmadore/Live-translation/issues/180)).

## Fixes

- **No empty band at the bottom of the window.** With a long setup column (for example with the
  names-and-terms list shown for Mistral), the whole operator window could scroll by a few lines,
  pushing the top bar out of view and leaving an empty strip under both columns. Text read only by
  screen readers was being placed against the window rather than its column; it no longer is.

Under the surface: the operator window gains one command, `open_support_page`, which opens
the Buy Me a Coffee page and nothing else, as it takes no address from the interface; the caption
overlay cannot call it. The MSIX capabilities are the same as in 1.10.0. Dependabot updated a
development tool (zod). The repository gained a Sponsor button that points to the same page.

Nothing about what the app sends or stores has changed: the support link hands the address to
your browser, and the app itself contacts no payment service. Cloud translation and cloud
subtitles still require your own provider account and API key, and may incur usage charges.

For the Microsoft Store, upload
[Live.Translation_1.11.0.msixbundle](https://github.com/fmadore/Live-translation/releases/download/v1.11.0/Live.Translation_1.11.0.msixbundle),
containing native x64 and ARM64 packages at version **1.11.0.0**. The Store signs accepted packages.
Separate MSIX files are provided for per-architecture testing; EXE/MSI installers are for
installation outside the Store.
