# Capturing one application

Capture audio from a single open application instead of everything playing on your computer.

For **System** or **Both** sources, choose **System capture → One application**, select an open
window, and run the level-only audio test before starting a live session. Click **Refresh
applications** to update the list. The other option, **All audio on the selected output**, listens
to your default or chosen output device as usual. Rehearsal and the built-in demonstration use
their own bundled audio and never capture applications. Local Whisper receives the same selected
process tree as any cloud engine; see [local Whisper](local-whisper.md) for offline buffering and
setup.

The list shows accessible processes with visible top-level windows, identified by window title and
PID. Multiple windows belonging to one process share a single entry; background-only and
inaccessible processes are omitted.

Building the list opens a `PROCESS_QUERY_LIMITED_INFORMATION | SYNCHRONIZE` handle on each
listed process to read its start time and check whether it is still running; that handle cannot
read memory or run code. Handles are opened only while **One application** is selected (when you
choose that mode or click **Refresh applications**), so capturing the full output never opens a
handle on another process—the safer choice for games whose anti-cheat software monitors process
handles.

## Capture scope and lifecycle

- WASAPI captures only the selected process and its child processes.
- Notifications from that application are included. In a browser, one process may host several
  tabs; individual tabs cannot be isolated.
- The capture mode is remembered across launches, but the chosen process is not: select the
  application again after restarting the app. Selecting a process does not start capture.
- The app checks both the process ID and its creation timestamp and keeps a live handle open, so
  PID reuse cannot silently redirect capture to a different program.
- If the selected application closes mid-session, that source stops while a working microphone
  keeps running. The recovery action stops and drains the session, preserves the transcript and
  prompts you to pick a new source.
- A missing selection, unsupported Windows version or capture failure never falls back to all
  system audio automatically; only you can switch modes.
- Requires Windows build 20348 or later. Process loopback runs independently of a render endpoint,
  using explicit 48 kHz stereo float audio and a 20 ms shared/event buffer.
- The existing resampling, metering, bounded queues and provider clients are reused. Process
  enumeration never runs inside the real-time audio callback.
- Only the operator window can enumerate applications; the overlay has no permission to invoke it.

## Verification

Automated tests cover mode preservation, missing, unsupported or stale selections, pre-flight
forwarding, selector behaviour and command capability boundaries. Windows tests enumerate
application windows, open a process-loopback client against a silent child process, and verify
process exit and stale creation times.

Manual acceptance testing (introduced in 1.2.3, issue #27) remains open for full isolation and
device-change behaviour in development, NSIS and MSIX builds. Run the isolation and exit checks
with both local Whisper and a cloud engine, letting Whisper's backlog finish draining after Stop
before inspecting the transcript.

| Scenario | Expected result |
| --- | --- |
| Teams/Zoom plus unrelated media | Only the selected application's process tree is captured |
| Browser with multiple tabs | Scope is explained; verify which tabs share the selected tree |
| Microphone + application | Both meters and captions work; an app failure leaves the mic running |
| App exit and restart | Capture stops; explicit reselection required; transcript preserved |
| Default-output or Bluetooth/dock change | Application scope stays selected; verify capture continuity |
| Minimised app, multiple windows | Selection remains stable; no duplicate entry for one PID |
| No application audio | Silence, without a false disconnection |
| Unsupported Windows | Explanatory message; all-output capture only after explicit selection |
| Demo and rehearsal | Bundled fixture behaviour remains unchanged |

The silent-process smoke test does not replace live isolation checks in Teams, Zoom or browsers.
