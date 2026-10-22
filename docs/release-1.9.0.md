# Live Translation & Subtitles 1.9.0

The operator window now comes in a light theme as well as the dark one, and follows Windows'
light or dark app mode unless you choose one yourself.

## A light theme

- **System, Light or Dark.** Settings › App › Theme. **System is the default**: it follows the
  light or dark app mode set in Windows, and switches with it while the app is open. **If your
  Windows is in light mode, the app opens light after this update; choose Dark to keep the
  look you had.**
- **The title bar follows.** On Light or Dark the window's title bar takes the same mode; on
  System it is left to Windows.
- **The caption overlay is unchanged.** Its captions keep the colours you set for the room,
  and its move-mode toolbar stays dark over whatever slide it sits on, in every theme.
- **No dark flash on a light desktop.** The theme is applied before the window draws its first
  frame, so a light desktop never sees the dark window first.
- **Held to the same contrast as the dark theme.** Every text colour reads at 4.5:1 or better
  on every surface it sits on, and every status mark at 3:1. A selected card, source or engine
  is marked by an edge that stays stronger than the edge of a card under the pointer, so the
  card you chose cannot be confused with the one you are hovering. Windows contrast themes
  still take over completely.

## Also in this release

- In Transcript history, the filter controls stay lined up when a label wraps onto two lines,
  as French and German labels do at larger text sizes
  ([#158](https://github.com/fmadore/Live-translation/pull/158)).

Under the surface: the light theme is one set of colour tokens re-declared for a white page
(`src/app.css`), so no component paints a colour of its own. The contrast tests run for both
themes and check the selected edge against the hover edge. The computed-style snapshots gained a
light pass, and a check that the first frame is light on a light desktop. The operator window
gains one permission, `core:window:allow-set-theme`, for its own title bar; the overlay's
permissions are unchanged. No dependency changed, and the MSIX capabilities are the same as in
1.8.0 ([#159](https://github.com/fmadore/Live-translation/pull/159)).

Nothing about what the app sends or stores has changed: the theme is a local preference, like
the interface language. Cloud translation and cloud subtitles still require your own provider
account and API key, and may incur usage charges.

For the Microsoft Store, upload
[Live.Translation_1.9.0.msixbundle](https://github.com/fmadore/Live-translation/releases/download/v1.9.0/Live.Translation_1.9.0.msixbundle),
containing native x64 and ARM64 packages at version **1.9.0.0**. The Store signs accepted packages.
Separate MSIX files are provided for per-architecture testing; EXE/MSI installers are for
installation outside the Store.
