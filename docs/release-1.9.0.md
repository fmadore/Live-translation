# Live Translation & Subtitles 1.9.0

The operator window now comes in a light theme as well as the dark one, and follows Windows'
light or dark app mode unless you choose one yourself. This release also fixes a set of smaller
problems found in a review of the whole app.

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

## Fixes

- **The caption overlay can no longer be closed by accident.** Alt+F4 while placing it used to
  remove it until the app was restarted; it now hides it, and Show overlay brings it back.
  A double-click while placing it no longer maximises it.
- **A mistyped API key fails once.** A key pasted with a space or an invisible character in it
  is refused when you save it, with the character and its position named. A key saved that
  way earlier now stops its source with "check the API key" instead of reconnecting forever.
- **Mistral lines keep their last words.** A line could close a moment before Mistral's end of
  turn, leaving the final words or the full stop on a line of their own.
- **Rehearse after a failed session** now plays the sample recording, with its meter and cost,
  instead of showing the source that had just failed and its old error.
- **No false "no captions" warning after a pause.** Resuming after a break longer than
  15 seconds no longer raises it, or has Narrator read it, before the provider has had a chance.
- **Narrator** no longer repeats the Whisper backlog every second or the model download
  percentage; it says when finishing starts and when audio is discarded. Keyboard focus moves
  to Stop as a session starts and back to Start as it ends, instead of falling to the page.
- **Errors keep their cause**: a reconnect, a model download or a Credential Manager failure
  now says which, such as a proxy or certificate problem.
- **Wording:** sums of money follow the interface language (`0,48 $` in French and German),
  history durations over an hour read as hours, a failed action in Transcript history has its
  own message, the crash-recovery prompt agrees with a single line, and messages re-word when
  you switch the interface language.
- **Closing with unsaved text:** once you choose Save or Discard, the other buttons and Escape
  wait until it has finished.
- In Transcript history, the filter controls stay lined up when a label wraps onto two lines,
  as French and German labels do at larger text sizes
  ([#158](https://github.com/fmadore/Live-translation/pull/158)).

Under the surface: the light theme is one set of colour tokens re-declared for a white page
(`src/app.css`), so no component paints a colour of its own. The contrast tests run for both
themes and check the selected edge against the hover edge. The computed-style snapshots gained a
light pass, and a check that the first frame is light on a light desktop. The operator window
gains one permission, `core:window:allow-set-theme`, for its own title bar; the overlay's
permissions are unchanged ([#159](https://github.com/fmadore/Live-translation/pull/159)). The
content security policy also forbids plugins, `<base>`, form targets and frames; a trace log
never records the WebSocket handshake, which carries the provider key; model downloads write to
disk off the async runtime; a capture thread that panics reports a capture error. Tauri moves
to 2.12.4 (the `tauri` crate and `@tauri-apps/api`), and `tauri-plugin-single-instance` to
2.5.4. Two other Rust dependencies changed in the lockfile only: `sha2` back to a single
0.10 copy, and `chacha20` from a yanked 0.10.1 to 0.10.2. The MSIX capabilities are the same as in 1.8.0. The
[22 October review](https://github.com/fmadore/Live-translation/blob/main/docs/app-review-2026-10-22.md)
lists what was changed and what is left.

Nothing about what the app sends or stores has changed: the theme is a local preference, like
the interface language. Cloud translation and cloud subtitles still require your own provider
account and API key, and may incur usage charges.

For the Microsoft Store, upload
[Live.Translation_1.9.0.msixbundle](https://github.com/fmadore/Live-translation/releases/download/v1.9.0/Live.Translation_1.9.0.msixbundle),
containing native x64 and ARM64 packages at version **1.9.0.0**. The Store signs accepted packages.
Separate MSIX files are provided for per-architecture testing; EXE/MSI installers are for
installation outside the Store.
