# Live Translation & Subtitles 1.12.0

Live Translation & Subtitles can now show its captions in a Zoom meeting's own caption bar, so
attendees turn them on in Zoom and install nothing. This release also stops several flashes and
layout shifts in the setup and the transcript, and says why Start waits when one application is
to be captured and none is chosen.

## Captions in Zoom

- **One caption language in Zoom's caption bar.** Choose **Zoom captions** in the toolbar,
  paste the caption link (API token) that the meeting's host copied in Zoom with *Show Captions ›
  Set up manual captioner › Copy the API token*, choose the caption language (with two caption
  languages) and the sources (with the microphone and system audio both captured), and **Start
  sharing**. Attendees turn captions on with **Show Captions** in their own meeting controls
  ([#189](https://github.com/fmadore/Live-translation/pull/189), part of
  [#187](https://github.com/fmadore/Live-translation/issues/187)).
- **What the host sets up.** Once, in the Zoom web portal, the host or an account admin turns on
  *Manual captions* and allows the caption API token for third-party closed-captioning services.
  In each meeting, the host or a co-host copies a new API token: a token belongs to one meeting.
  The dialog's **Step-by-step setup guide** opens the [full guide](zoom-captions.md) in the
  browser.
- **Caption text only.** Sharing sends finished caption text, and nothing else, straight from
  the app to Zoom's caption address. This includes captions made on this PC with Whisper: once
  sharing is started, their text leaves the computer, although the audio does not. It opens no
  second provider session, so it adds no provider cost. Lines that an engine may still revise are
  held back until they are final, and each Zoom caption is at most 120 characters.
- **Independent of the session.** Sharing can start before or during a caption session and
  carries on across Stop and Start; it ends with **Stop sharing** or when the app quits. The
  button shows where it stands: *sharing*, *waiting* (Zoom says the meeting has not started),
  *refused* (the link has expired, been revoked or was copied incompletely) or *unreachable*
  (no connection or a Zoom error; each new caption tries again). The dialog counts the captions
  sent and any skipped because they could not be delivered in time.
- **Send test caption** shows one line, "Test caption from Live Translation." (in the interface language), to everyone in
  the meeting, to confirm the link works before anyone speaks.
- **The link is treated like a password.** It is kept in memory until the app quits, never
  written to the saved setup, a meeting profile, history, the recovery copy or a log, and its
  field is masked in case the operator's screen is projected.
- **Zoom only, if you prefer.** Untick *Show captions on this screen too* in the dialog to hide
  the overlay and caption the meeting through Zoom alone. It is the overlay's usual show and
  hide, so the overlay's button, the tray menu and Ctrl+Shift+O bring it back.
- **Works on a free Zoom account.** The caption API token is available on a Basic account as
  well as a licensed one.
- **A plain name.** The button reads *Zoom captions* (*Sous-titres Zoom*, *Zoom-Untertitel*)
  with a generic captions mark, so it cannot be mistaken for magnification. Zoom is named only
  to say what the feature works with; the app does not use Zoom's logo and is not affiliated
  with Zoom.

Captions in Zoom has been tested in a real meeting on a free account. Reading pace, delay and
interaction with Zoom's own captions have not yet been measured one by one; breakout rooms and Zoom Rooms are untested. The
[guide's limits](zoom-captions.md#limits) list what is known.

## Fixes

- **Start says why it waits for an application.** With system capture set to one application
  and none chosen, or the chosen one closed, Start and Test audio were disabled while the
  pre-flight still read as ready. The audio row now asks for the application and is marked as
  needing attention.
- **No more flashes and jumps in setup and the transcript**
  ([#188](https://github.com/fmadore/Live-translation/pull/188)):
  - *Jump to latest* floats over the foot of the transcript instead of pushing the panel below
    it down, and the copy confirmation floats under **Copy** instead of a row that appeared and
    vanished above the log.
  - Switching between Gemini and OpenAI no longer flashes the API key form.
  - **Refresh devices** shows itself busy only when you asked for the refresh.
  - Local Whisper's waiting-audio strip no longer blinks, and moves the stage, at its threshold.
  - A stored microphone, output or application is no longer labelled missing before the first
    device or application list has arrived.
  - The second caption language uses the same searchable picker as the first, with *None* at the
    top and the first language left out.

Under the surface: the operator window gains five commands, `start_zoom_captions`,
`stop_zoom_captions`, `send_zoom_test_caption`, `zoom_captions_status` and `open_zoom_guide`; the
caption overlay can call none of them. The core accepts only an `https` caption link on
`zoom.us` or `zoomgov.com` and refuses anything else before sending a byte, and
`open_zoom_guide`, like `open_support_page`, takes no address from the interface. Sending runs
on its own task with a bounded queue, so a slow or failing Zoom never delays capture, the
overlay, the transcript or exports. No crate was added, and the MSIX capabilities are the same as
in 1.11.0.

Nothing is sent to Zoom unless you start sharing, and nothing passes through a server of the
developer's: the app posts caption text directly to the address in the host's link, and Zoom
receives ordinary request information such as your IP address. The
[privacy policy](privacy.md#optional-captions-in-zoom) describes it. Cloud translation and cloud
subtitles still require your own provider account and API key, and may incur usage charges.

For the Microsoft Store, upload
[Live.Translation_1.12.0.msixbundle](https://github.com/fmadore/Live-translation/releases/download/v1.12.0/Live.Translation_1.12.0.msixbundle),
containing native x64 and ARM64 packages at version **1.12.0.0**. The Store signs accepted packages.
Separate MSIX files are provided for per-architecture testing; EXE/MSI installers are for
installation outside the Store.
