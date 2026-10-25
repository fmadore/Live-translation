# Live Translation & Subtitles 1.10.0

Long cloud sessions now hold on to their words through the provider's routine reconnects, and
system audio keeps up when the PC is busy. Choosing system audio no longer touches any other
running program, which matters for games with anti-cheat. The caption overlay can do less
than before, and tells the operator window when it has been hidden. This release completes the
fixes from a review of the whole app.

## Cloud sessions and capture

- **No lost words at a Gemini handover.** Gemini asks the app to move to a new connection
  several times an hour. The app used to drop the old one at once, losing the last second or more
  of speech and cutting the line mid-sentence. It now lets the old connection finish what it
  was transcribing, keeps the audio captured meanwhile, and sends it on the new one.
- **A brief provider hiccup no longer ends a source.** A timeout or a "try again later" from
  Gemini, OpenAI or Mistral while a session was connecting used to stop that source until you
  restarted. The app now retries it, as it already did mid-session; five failures in a row
  still stop it, and a rejected key or an exhausted quota still stops it at once.
- **System audio keeps up under load.** Capturing the PC's audio while local Whisper is busy
  could fall behind for good and drop audio. The capture now uses a longer buffer, reads
  everything waiting each time it wakes, and turns the silence Windows reports into real
  silence.
- **System audio leaves other programs alone.** To list applications for capture, the app
  opens a lightweight handle on each program with a window, and kernel anti-cheat watches for
  handles on a game. The list is now built only when you choose **One application**, and
  Refresh rebuilds it only then, so System audio and the microphone open nothing on any other
  process ([#177](https://github.com/fmadore/Live-translation/pull/177)).

## The caption overlay

- **The overlay window says when it is hidden.** After Alt+F4 hides the overlay, the Show
  overlay toggle and the tray menu say so, and one click brings it back.
- **Two-language labels after a slow start.** If the overlay finished loading after the
  operator window, as on a cold start, its language labels could stay empty. It now reports
  when it is ready and receives the full settings.
- **Adjusting the overlay mid-session no longer disturbs the captions.** Dragging a colour or
  opacity slider used to keep finished lines on screen and release paced text early.
- **A strip across two displays stays across two displays.** A placement stretched over two
  projectors on purpose was shrunk to one on the next launch; it is now restored as placed.
- **Locking the overlay sticks.** A late update from the operator window can no longer put the
  overlay back in move mode just after you locked it.

## The operator window

- **Start waits for the session.** Start stays blocked, showing "Starting…", until the
  session reports that it is running, so a quick second click or the shortcut can no longer
  restart the session you just began.
- **The transcript keeps its place.** The time window chosen beside Copy, the scroll position
  and a save under way now survive Start and Stop.
- **Crash recovery can be switched on before a first session**, in Settings › Transcript
  history next to the history switch, as well as under the transcript.
- **Saving problems stay visible.** If the background recovery copy cannot be written, the app
  keeps trying, and "Recovery copy failing" or "History not saving" stays beside the save
  badge until it works again.
- **Right-to-left captions in the operator window.** Arabic, Hebrew, Persian and Urdu captions
  on the Live and Transcript tabs now run right to left, and Narrator reads them in their own
  language rather than the interface's.
- **Smaller fixes:** saving or removing a key while switching engine credits the right engine;
  the Mistral model name breaks between its words in a narrow window; the notes for an
  unavailable application or audio device re-word when you switch the interface language.

Under the surface: the overlay's permissions are cut to listening for events and its own
commands (report its state, start a drag, nudge, snap and undo). It can no longer send events
to the rest of the app or move the operator window, and a test pins the list. One error
classifier now serves all three cloud providers. Operator pushes to the overlay are coalesced
to one per frame. Errors keep their whole chain of causes everywhere. The operator window's
page is split into smaller components. New tests cover the Credential Manager store, the
system-audio decoder, the tray, Whisper's pipeline and the session manager. Dependabot updated
`keyring-core` to 1.0.1 and two development tools. The MSIX capabilities are the same as in
1.9.0. The
[22 October review](https://github.com/fmadore/Live-translation/blob/main/docs/app-review-2026-10-22.md)
lists what was changed and what is left
([#171](https://github.com/fmadore/Live-translation/pull/171)).

Nothing about what the app sends or stores has changed. Cloud translation and cloud subtitles
still require your own provider account and API key, and may incur usage charges.

For the Microsoft Store, upload
[Live.Translation_1.10.0.msixbundle](https://github.com/fmadore/Live-translation/releases/download/v1.10.0/Live.Translation_1.10.0.msixbundle),
containing native x64 and ARM64 packages at version **1.10.0.0**. The Store signs accepted packages.
Separate MSIX files are provided for per-architecture testing; EXE/MSI installers are for
installation outside the Store.
