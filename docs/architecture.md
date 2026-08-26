# Architecture

## Data flow

```text
live microphone (CPAL) ─┐
                        ├─ PCM16 bounded channel ── provider WebSocket ─┐
system audio (WASAPI) ──┘                                              │
                                                                        ├─ caption/status/level events
built-in demo ───────────── deterministic caption + level timeline ────┘
                                                                                 │
                                                                         operator + overlay
```

1. **Live capture.** Each live source owns a native capture thread. `CaptureState` downmixes,
   low-pass filters when downsampling, resamples to the provider rate, converts to mono PCM16,
   and forms roughly 100 ms chunks. Realtime callbacks write only to bounded channels.
2. **Provider sessions.** Gemini and OpenAI produce translated captions; Mistral produces
   same-language captions. One async client runs per selected source and owns setup, timeouts,
   backoff, reconnect classification, audio pumping, turn finalization, and graceful shutdown.
3. **Built-in demonstration.** The compatibility provider id `ondevice` starts
   `ondevice::run_session`, which opens no device and contacts no service. A deterministic
   English or French timeline emits the same session-status, audio-level, partial-caption, and
   final-caption events as a live provider. This tests the shipping UI, timer, overlay,
   transcript, and export paths identically on x64 and ARM64. It is explicitly presented as a
   demonstration, not speech recognition.
4. **Render/export.** Both windows receive caption events. Pending turns are keyed by
   `(origin, turnId)` and finalized lines remain available for plain-text or Markdown export.

## The transcript document

The transcript is an explicit document with a saved state, not a scrolling side effect.

- **No truncation.** The log is unbounded. It was previously capped at 1,000 lines and
  silently truncated, which discarded the beginning of exactly the long events worth keeping.
  Past `TRANSCRIPT_WARN_LINES` (`src/lib/document.ts`) an unsaved log is flagged on screen;
  nothing is ever dropped.
- **Saved vs unsaved.** `savedLineId` records the highest line id written to disk, so a second
  save of an unchanged document stays saved while one further line makes it unsaved again.
  `clearTranscript` resets the marker with the text, because line ids keep climbing and a
  stale marker would make a later run's first lines look as if they were already saved.
- **Optional recovery spool.** Off by default. When enabled, `src/routes/+page.svelte` writes
  the finalized lines to one file in the app's local data directory every few seconds while
  the document is unsaved. `recovery.rs` reads and writes that file as opaque UTF-8 and never
  interprets it; the format lives in `src/lib/document.ts` and carries caption fields only.
  It is deleted on save, clear, discard, disable, and once a startup recovery offer is
  answered. A malformed or truncated spool is deleted rather than shown.

## Provider contracts

| Provider | Mode | Input | Caption source | Graceful stop |
|---|---|---|---|---|
| Google Gemini Live | Translate | 16 kHz PCM16 | output transcription | WebSocket close |
| OpenAI Realtime | Translate | 24 kHz PCM16 | output transcript deltas | close and drain |
| Mistral Voxtral Realtime | Transcribe | 16 kHz PCM16 | transcription deltas | flush, end, drain |
| Built-in demo | Transcribe demo | bundled deterministic timeline | scripted partial/final events | cancellation token |

Mistral and the built-in demo are unavailable in translation mode. `session.rs` enforces the
mode/provider relationship through `Provider::can_translate`.

## Concurrency and shutdown

`SessionManager` serializes start and stop operations with a lifecycle mutex. A parent
`CancellationToken` owns the run and each live source gets a child token. A capture failure
cancels that source. Stop cancels producers, lets live providers flush and drain briefly, joins
capture threads, clears meters and current captions, and retains completed transcript lines.

The built-in demo observes the same cancellation token on every short delay, so Stop remains
responsive and cannot leave an audio or recognizer thread behind.

Closing the operator window runs the same shutdown, in order: `recovery::CloseGuard` holds the
close only while the front-end reports unsaved text or a live session, `prepareClose`
(`src/lib/quit.ts`) stops the session and waits on the bounded drain, the in-flight turn is
committed, and only then is the operator asked to save, discard, or cancel. A save that fails
keeps the app open. Two rules keep the guard from ever producing a window that refuses to close:
it defaults to off, and an interception the front-end does not acknowledge within `ACK_TIMEOUT`
is released. `WindowEvent::Destroyed` on the operator window exits the process, so the
undecorated always-on-top overlay cannot outlive its controls.

## Security and privacy

- The built-in demo opens no audio device, uses no network, and needs no credential.
- Each optional provider key has a separate Windows Credential Manager entry, with `.env`
  fallback for development.
- Keys never enter the Svelte renderer. Provider authentication happens in Rust.
- The optional recovery spool is local-only, off by default, holds finalized caption text and
  nothing else, and is deleted as soon as the transcript is saved or discarded.
- The developer operates no backend, relay, telemetry, analytics, or crash-reporting service.
- The webview content-security policy blocks direct renderer connections.

## Operational limits

- Windows is the supported release target; native x64 and ARM64 packages are built.
- CI contract-tests provider messages but does not call billable services.
- Live caption accuracy and availability depend on the selected third-party provider.
- The built-in demo verifies product presentation and workflow, not microphone recognition.
