# Privacy Policy — Live Translation & Subtitles

**Effective date:** 23 November 2026 (adds OpenAI subtitles, sending audio, entered names and
terms, and the chosen spoken language to OpenAI; clarifies that names and terms are sent to
Mistral, Gemini Transcribe and OpenAI subtitles and used locally by Whisper)

Previous version: 16 November 2026 (added the optional new-version check for GitHub installs,
optional system-wide shortcuts, and operator transcript markers)

## What this app is

Live Translation & Subtitles is a desktop application for Windows and macOS with three modes of
operation:

- local multilingual speech transcription, or translation into English, using Whisper on your
  computer;
- optional cloud captioning or translation through a provider you choose and configure: Google
  Gemini, OpenAI, Mistral or DeepL; and
- a scripted demonstration of the caption display, overlay, timer, level meter, transcript and
  export workflow.

With any of these, you can optionally share one caption language with a Zoom meeting's built-in
caption bar, as described under [Optional captions in Zoom](#optional-captions-in-zoom).

The built-in demonstration is clearly labelled as a demonstration. It does not capture or
recognise speech.

## Audio

The built-in demonstration never opens a microphone or system-audio device. Its simulated level
data and English/French caption scripts are bundled inside the application and stay on your
device.

When a live provider is selected, the app captures only the audio source you choose: a microphone,
the computer's system audio, or both. With a cloud provider, audio is processed in memory and
streamed directly to that provider for the duration of the session; nothing is written to disk.
System capture can listen to all audio on an output device or to a single application (its child
processes on Windows, or its helper processes on macOS). Single-app capture includes that
application's notifications and may include multiple browser tabs. On macOS, system audio uses a
Core Audio process tap, which macOS grants only after you allow *System Audio Recording*; it
captures sound only, never screen content. Application titles and process identities stay local to
select the source and are never sent to a speech provider.

With **local Whisper**, audio is processed on your computer and is never uploaded. Pending audio
is buffered in a temporary file so a slower recogniser can catch up without dropping speech. This
file stays local, is not a saved recording, and is deleted automatically when its handle closes,
including when the app exits. It is neither an encrypted archive nor a recoverable recording.
Stopping a session finishes any remaining audio; discarding the backlog, or closing the app before
it finishes, keeps captions already produced but leaves the transcript incomplete. Pausing
excludes new audio from recognition and temporary storage while keeping input meters active.

If a cloud source stops mid-session, you can continue it with local Whisper; from that point on,
that source's audio stays on your computer.

On Windows, Whisper can run in `whisper-vulkan`, a bundled helper that uses your graphics
processor. Audio reaches it only through local memory, never over a network, and it keeps nothing
once the session ends. The speech-detection model that skips silence before Whisper is bundled
with the app and is never downloaded.

Whisper model files download only when you click **Download model**. Downloads contact Hugging
Face and its file-delivery infrastructure, which receive ordinary network request details such as
your IP address, with no captured audio, captions or API keys. If a proxy server is configured in
Windows Settings, macOS network settings, or `HTTPS_PROXY`/`HTTP_PROXY`, downloads go through it
and the proxy sees the same request details. While downloading, the file is saved as `<model>.part`
in the models folder and deleted if the download fails, is cancelled, or is superseded after an
interrupted launch. The app verifies each model's size and SHA-256 checksum before installation
and before use. Models stay in the application's local-data `whisper-models` folder until you
remove them. Once a model is installed, local transcription and translation need no internet
connection.

## Captions and transcripts

Caption text is held in memory while the app runs, and the full session is kept intact: the
transcript is never truncated or discarded behind your back. A transcript is written to disk only
when you explicitly save it, or if you turn on the optional recovery copy or session history
described below. Saved transcripts are never uploaded automatically. Markers you add to the
transcript (such as a talk's start or a speaker's name) are stored and exported with it; break
notices shown on the overlay stay in memory only.

The overlay keeps a bounded amount of recent reading context in memory so a taller window can show
more lines. Stable reading drops the oldest hidden lines once its buffer fills. Text cleared from
the overlay by resizing or its idle timeout stays in the full session transcript. Layout choices
create no extra disk history or network requests.

Clicking **Copy** in the live transcript or a saved session puts the selected text on your system
clipboard as plain text. Other apps can then read it. If clipboard history or cross-device sync is
enabled in Windows Settings, Windows keeps or syncs it like any copied text; on macOS, Universal
Clipboard may offer it to your nearby Apple devices. The app writes to the clipboard but never
reads from it.

Closing the app with unsaved captions asks whether to save or discard them first.

## Optional local recovery copy

Recovery is **off by default**. If you turn on *Keep a local recovery copy while captioning*, the
app periodically saves finalised caption lines from the current session to a single file in its
local application-data folder:

`%LOCALAPPDATA%\io.github.fmadore.live-translation\recovery\transcript.json`

- It stores only finalised caption text and any transcript markers (text and timestamp), including
  markers added before the first caption. It never stores audio, API keys, provider identities or
  device names.
- It is never uploaded, transmitted or shared; the developer operates no service that could receive
  it.
- Only one file exists, with no history: each write replaces the previous one.
- It is deleted automatically as soon as you save the transcript, clear the log, answer the
  unsaved-transcript prompt, turn recovery off, or restore or decline it at the next launch.
- After a clean exit, no recovery file remains. It appears at startup only after a crash or power
  cut, when the app asks whether to restore or delete it.

You can delete the file manually at any time without affecting the app.

## Optional transcript history

History is **off by default**. Turning on *Automatically save sessions locally* progressively saves
finalised raw caption lines from that point onward, one file per session, under:

`%LOCALAPPDATA%\io.github.fmadore.live-translation\history\<session-id>.json`

Each file stores its optional title, start time, duration through the last save, output mode,
known language details, raw translated and source text with cue timing, and any transcript markers
(text and timestamp). A session containing only markers is also saved. Auto-detected source
languages are marked as such rather than guessed. No audio, credentials, application identity or
device name is saved.

History remains after you export, clear the active transcript, close the app or turn history off.
Turning history off stops future saves; delete unwanted sessions permanently (with confirmation)
inside Transcript history. Files are local, unencrypted, never uploaded automatically and never
expire on their own. An interrupted session stays readable up to its last completed save. Clearing
the recovery copy does not affect history.

*Hide filler words* is an optional overlay display filter. Its word list is edited by you and
stored only on this device. Raw text in the transcript, history, recovery copy and exports stays
untouched, and the filter makes no network requests.

## Optional captions in Zoom

Sharing captions with Zoom is **off until you start it** for a meeting. Once you paste the caption
link (API token) copied by the meeting's host and start sharing, the app sends finished caption
text in your chosen language and from your chosen sources directly to Zoom, which displays it to
attendees. This includes captions generated locally by Whisper: once sharing is active, their text
leaves your computer, though the audio never does. No audio, API key, device name or transcript
file is sent, and nothing passes through a developer server. Because captions reflect what was
said, they may contain names or other personal information, visible to anyone in the meeting who
turns captions on. **Send test caption** sends one fixed sentence in the interface language ("Test
caption from Live Translation." in English). Sharing stops when you click **Stop sharing** or quit
the app, and stays active across Stop and Start of caption sessions until then.

The caption link stays in memory only until the app quits, so you can stop and restart sharing
without pasting it again. It is never saved in preferences, meeting profiles, history, the recovery
copy or logs. Zoom receives ordinary network request details, such as your IP address, with each
caption. How Zoom handles those captions, including whether they enter a meeting transcript or
recording, is governed by Zoom's terms and the host's settings.

## New-version check (copies installed from GitHub)

Microsoft Store and Mac App Store builds update automatically and never run this check or display
any setting for it, nor do the MSIX test packages attached to GitHub releases. Copies installed
directly from GitHub (the Windows installer or MSI, or the Mac disk image) can optionally check
for newer releases.

- **Off until you turn it on.** From the second launch, the start screen asks once: *Check for new
  versions? This asks GitHub once a day.*, with **Turn on** and **No thanks**. Nothing is sent
  until you click **Turn on** or enable *Check for new versions* under Settings › App › Updates,
  where you can also turn it off.
- **What is sent.** One standard HTTPS request to GitHub's API (`api.github.com`) for the
  repository's latest release, with no account or key. Its User-Agent header names only the app
  and version (for example `LiveTranslation/1.14.0`). No user, device or installation identifier
  is sent, and no audio, captions, transcripts, settings or keys leave your device. GitHub
  receives ordinary network request details such as your IP address; if a system proxy is set, the
  request uses it.
- **When.** At most once a day, a few seconds after launch and never during a session. Failed
  checks (offline, refused or rate-limited) are silent and retry the next day.
- **What is shown.** When a newer release exists, *Version X.Y.Z is available* appears on the
  start screen and under Settings › App, with **Release notes** (opening GitHub in your default
  browser) and **Not now** (hiding the notice until a newer version arrives). The app never
  downloads or installs updates itself.
- **What is kept.** Your choice, the launch count until you answer, the last check time, the
  newest version seen and any dismissed version are stored locally in `update-check.json` inside
  the app's local application-data folder
  (`%LOCALAPPDATA%\io.github.fmadore.live-translation\` on Windows,
  `~/Library/Application Support/io.github.fmadore.live-translation/` on macOS). It is never
  uploaded.

The developer receives nothing from this check: the request goes straight to GitHub.

## API keys

Local Whisper and the built-in demonstration need no API key. Optional provider keys are stored in
Windows Credential Manager or the macOS Keychain and read only by the native application core.
Each key is sent solely to the provider that issued it to authenticate live sessions. The
developer never receives, copies, proxies or stores your keys.

## Preferences stored on the device

The app stores ordinary interface preferences locally on your device:

- **Setup and engines:** the last mode, provider, Whisper model and spoken language, audio source,
  caption languages, names and terms entered for Mistral, Gemini Transcribe, OpenAI subtitles and
  local Whisper, and which cloud providers you have agreed to send audio to.
- **Overlay and reading:** overlay position and size for up to eight display arrangements
  (recorded as display positions, resolutions and scaling so it reopens in the same spot), caption
  size, layout (Fit window, Compact or Stable reading), compact line width, caption hold time and
  update pace, filler-word cleanup and its word list, and whether a break notice clears itself
  when captions resume (the notice text itself is never stored).
- **Session and app behaviour:** cloud spending-guard choices, whether history or crash recovery
  is enabled, the new-version check state on GitHub installs
  ([above](#new-version-check-copies-installed-from-github)), whether a running session opens on
  Live or Transcript, whether closing the window keeps the app in the notification area (or macOS
  menu bar), whether system-wide shortcuts are enabled, the last export folder and format, and
  whether system capture listens to an output device or a single application.

Selected application process identities are never saved (choose the application again after
relaunch), and Zoom caption links are never stored. Named meeting profiles save your setup
(including names and terms), audio device identifiers, appearance and overlay rectangle in local
webview storage; they contain no API keys, audio, caption text, application process identities or
rehearsal mode, and stay until you delete them in Meeting profiles.

Raw transcripts preserve the text returned by the provider. Gemini Transcribe returns a
word-for-word transcript, hesitations included; the local overlay filter hides them on screen
without changing the transcript.

## Running in the notification area or the menu bar

The app never starts with your computer and does not run as a background service. It appears in
the Windows notification area (or macOS menu bar) only while running. Closing the window quits the
app unless you turn on *Keep running in the tray when I close this window* (*in the menu bar* on
macOS). When that option is on, the app tells you the first time the window is hidden, and
quitting from the icon stops any active capture before exiting.

## Optional system-wide shortcuts

Settings › App › *Use shortcuts while other apps are in front* is off by default. When enabled,
and only while a session is running or paused, the app registers two fixed combinations with the
operating system: Ctrl+Alt+Shift+P (pause or resume) and Ctrl+Alt+Shift+O (show or hide the
overlay), or ⌃⌥⇧P and ⌃⌥⇧O on macOS. It uses the system's own hot-key API (`RegisterHotKey` on
Windows, `RegisterEventHotKey` on macOS), not a keyboard hook or event tap. The app is notified
only when one of those two combinations is pressed (and released on Windows); it never sees other
keystrokes, what you type or which app is in front, and nothing is recorded or sent anywhere. Both
shortcuts are released when the session stops, when the option is turned off or when the app
quits. macOS requests no Accessibility or Input Monitoring permission for this.

## Support link

Settings › App and the foot of the start screen include an optional *Buy me a coffee* link to
`https://buymeacoffee.com/fmadore`. Clicking it simply opens that fixed address in your default
browser: the app sends no information about you, your device or your usage, and contacts no
payment service itself. Anything you enter on that site is governed by the
[Buy Me a Coffee privacy policy](https://buymeacoffee.com/privacy-policy). Supporting is optional
and changes nothing in the app. The Mac App Store build omits this link.

## macOS

On macOS, the app requests Microphone and System Audio Recording permissions the first time a
session needs them; either can be revoked in System Settings › Privacy & Security. Its data folder
(Whisper models, optional history and recovery copy) lives at
`~/Library/Application Support/io.github.fmadore.live-translation`, or inside the sandbox
container (`~/Library/Containers/io.github.fmadore.live-translation`) for the Mac App Store build.
On first launch of the Mac App Store version, macOS migrates any existing disk-image data folder
into that container. Distribution through Apple (notarisation, App Store and TestFlight) involves
Apple receiving the app package from the developer, never any user data; App Store downloads are
governed by Apple's privacy policy.

## What the developer collects

The developer operates no server, relay, account system, telemetry, advertising, analytics or
crash-reporting service for this app, and receives no audio, captions, transcripts, API keys,
device identifiers or usage data. The only requests the app makes on its own—outside the speech
providers, Zoom meetings or Hugging Face model downloads you explicitly choose—are the optional
daily version checks to GitHub on copies installed from GitHub.

## Third-party processing

Starting a cloud session streams audio directly to the provider you chose under your own account
and terms. Before your first session or rehearsal with each provider, the app asks for
confirmation: it names the provider, explains that captured audio goes there under your own
account, API key and terms, and links to this policy. Nothing is sent unless you agree, and your
answer is remembered locally for that provider. Deleting a provider's API key stops all traffic to
it.

With Mistral, Gemini or OpenAI subtitles, any names and terms you entered are sent to that
provider at session start alongside the audio to guide spelling; enter only names you are
comfortable sharing with that provider. With OpenAI subtitles, your chosen spoken language (if set)
is sent too, using the same API key as OpenAI translation. Local Whisper uses names and terms on
your computer alone, and cloud translation engines do not receive them.

With **DeepL** (translation only), audio from your chosen sources and the target caption language
are streamed to DeepL SE (Cologne, Germany) while the session runs, and DeepL returns the source
transcript and translation. Your DeepL key stays in Windows Credential Manager or the macOS
Keychain and goes only to DeepL. DeepL Voice requires a paid DeepL API plan; the app refuses
free-plan keys without sending audio.

Each provider's own terms and privacy policy apply:

- [Google Privacy Policy](https://policies.google.com/privacy)
- [OpenAI Privacy Policy](https://openai.com/policies/privacy-policy/)
- [Mistral AI Privacy Policy](https://mistral.ai/terms/#privacy-policy)
- [DeepL Privacy Policy](https://www.deepl.com/privacy)

The built-in demonstration and local Whisper (both subtitles and English translation) contact none
of these services. Captions shared with Zoom are governed by the
[Zoom Privacy Statement](https://www.zoom.com/en/trust/privacy/privacy-statement/) and the meeting
host's settings. Model downloads are covered by the
[Hugging Face privacy policy](https://huggingface.co/privacy). GitHub version checks are covered
by the
[GitHub General Privacy Statement](https://docs.github.com/en/site-policy/privacy-policies/github-general-privacy-statement).

## Children

The app is not directed to children, and the developer knowingly collects no personal information
from children.

## Changes to this policy

Material changes are published with a new effective date. The policy matching each release appears
in its Store listing and repository documentation.

## Contact

For privacy questions, use the publisher support contact listed on the Microsoft Store page for
Live Translation & Subtitles (Product ID `9PFB8LR3RR9X`).
