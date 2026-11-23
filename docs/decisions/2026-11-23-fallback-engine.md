# 2026-11-23 · A one-click fallback engine for a source that ends mid-session

**Status:** agreed on 23 November 2026, with the answers under *Decisions taken* below
([#248](https://github.com/fmadore/Live-translation/issues/248), finding F7 of the
[16 November review](../app-review-2026-11-16.md)). Line numbers are for `ff374de`.

## Context

When a cloud source ends on a key, quota or billing error, the operator today has to Stop,
change the engine in the setup and Start again: a new run, a new history record, and a clock
that starts from zero. What the code does now:

| What | Where | Today |
| --- | --- | --- |
| A fatal provider end | `realtime/mod.rs:387-427` | `RunEnd::Fatal`, `Rejected`, `Malformed` or a fifth failed setup become `error.providerStopped` or `error.providerRejected`, reported per lane as `Error` (`realtime/mod.rs:473-488`). |
| A network outage | `realtime/policy.rs:192` | Never ends a source: `Failed` and `Reconnect` back off up to 16 s and retry for ever, so the source stays `Reconnecting`. |
| The capture behind it | `realtime/mod.rs:334`, `session/builder.rs:117-121` | With one caption language the client owns the source's token and its drop guard cancels it, which stops the capture thread. With two, each lane has a child token and `relay_audio` (`builder.rs:34-62`) stops the capture once both lanes have gone. |
| The core's session | `session/mod.rs:54-74` | Still held after every source has ended, until Stop or the next Start. `set_paused` refuses it (`mod.rs:197-200`). |
| What a running session holds | `session/mod.rs:54-74` | Parallel vectors (`sources`, `capture_threads`, `fixture_tasks`, `relay_tasks`, `client_tasks`) and one session-wide `local: bool`, which decides how Pause is reported (`mod.rs:204`) and how Stop drains (`mod.rs:278-310`). |
| Turn ids | `realtime/mod.rs:110-126`, `whisper/reconcile.rs:32`, `whisper/pipeline.rs:330` | Each client starts at 0: `TurnAccumulator::new` and `Reconciler::default`. |
| The interface | `src/lib/sessionStatus.ts:137-176` | When the last source ends by itself, `isRunning` turns false, the clock stops (`sessionStartedAt.set(null)`) and the history record is closed (`endTranscriptSession`). The page swaps to the setup sheet. |
| The "banner" | `src/lib/RunningStage.svelte:20-22`, `LiveActivity.svelte` | There is none: the failure is the status line (`error.providerStopped`, `en.ts:1102`) and the source's `error` activity row. The only recovery banner is `DeviceRecoveryBanner.svelte` for capture failures, driven by `deviceFailure.svelte.ts`, which stops and starts the whole session. |
| The engine in documents | `src/lib/history.ts:103-113`, `types.ts:407-424` | Not recorded anywhere: the history header has mode and languages, lines have origin, lane and language. |

## Decision

A source that ended on a provider failure can be continued with another engine from a banner,
in the same run: same core `ActiveSession`, same `SessionClock`, same transcript, markers and
history record. Only that source's producer and client are rebuilt. The first version offers
offline Whisper only, in a session with one caption language.

### 1. Lifecycle

**The command.** `switch_source_engine(request: SourceSwitch)`, with
`SourceSwitch { origin, from: Provider, to: Provider, whisper_model?, whisper_gpu? }` in
`types.rs` and `types.ts` (pinned by `contract.test.ts`). Called by the **operator window only**:
handler in `commands.rs` and `lib.rs` `generate_handler!`, name in `command_names.rs`,
`allow-switch-source-engine` in `capabilities/operator.json`. Not in `overlay.json`; the pinned
list in `command_names.rs:143-156` fails if it is ever added there. The interface calls it as
`api.switchSourceEngine` (`src/lib/tauri.ts`), through `sessionController.ts`'s queue, so it
cannot interleave with this window's Start or Stop.

**What a session holds** becomes one record per source, replacing the parallel vectors:

```rust
struct SourceSlot {
    origin: Origin,
    engine: Provider,               // replaces ActiveSession::local
    cancel: CancellationToken,      // child of ActiveSession::cancel
    capture: Option<JoinHandle<()>>,
    fixture: Option<AsyncJoinHandle<()>>,
    relay: Option<AsyncJoinHandle<()>>,
    clients: Vec<AsyncJoinHandle<()>>,
    turns: [TurnCounter; 2],        // one per lane, see 6
}
```

`ActiveSession` also keeps what a rebuild needs and `start` already has: the `StartOptions`, the
`SessionClock`, the level sender (`spawn_level_forwarder`) and the loaded Whisper model
(`Option<Arc<LoadedModel>>`, shared by both sources as at Start). `SessionBuilder`
(`builder.rs:94-105`) stops borrowing `&StartOptions` and gains
`build_source(origin, engine) -> Result<SourceSlot>`, used by `start` and by the switch alike.
`shutdown` (`mod.rs:250-311`) drains each slot by its own engine: local clients without a
timeout, cloud clients within `CLIENT_DRAIN_TIMEOUT`, concurrently. `set_paused` reports
`Paused` itself for local slots only (`mod.rs:204-216`).

**`SessionManager::switch_source(events, request)`**, in `session/mod.rs`:

1. Take `lifecycle`. Before waiting for it, record a `switching: CancellationToken` beside
   `local_abort`; `stop`, `start` and Quit cancel it before they wait for the lock, so a Stop
   never waits out a model load.
2. Find the slot by `origin`. Refuse with `error.sessionFallback` unless `slot.engine == from`
   (a compare-and-swap: a double click or a stale banner does nothing) and every client of the
   slot has finished. Phase 2 also accepts a slot that has been `Reconnecting` for 30 s: its
   token is cancelled first and its clients joined within `CLIENT_DRAIN_TIMEOUT`.
3. Join what the slot still holds, outside the `active` mutex: the capture thread through
   `join_threads` (3 s bound, `capture.rs:141`), the relay, the clients. The device is free
   before it is opened again at the new engine's rate (`Provider::input_sample_rate`: Whisper
   is 16 kHz, OpenAI 24 kHz).
4. For Whisper: `whisper::cpu::ensure_supported()` (as `commands.rs:97`),
   `whisper::validate_language(options.spoken_language)`, and `whisper::load` on
   `spawn_blocking` if the session holds no model or another one, with the session's `abort`
   token (so Discard and Quit cut it short) raced against `switching`.
5. `build_source(origin, to)` with a new child of `session.cancel`, the slot's `TurnCounter`s
   and the session's `failure`. `EchoGuard::lengthen_hold(HOLD_LOCAL)` when `to` is local
   (see 6).
6. Put the slot back under the `active` mutex. If the session is paused at that moment, the new
   client sees it through its own `pause.subscribe()`; a Whisper slot reports `Paused` instead
   of `Running` (`drive` reports `Running` unconditionally today, `pipeline.rs:194`, so
   `Pipeline` gains the pause receiver for its first status).

**Races.**

| Race | Outcome |
| --- | --- |
| Pause or Resume during the switch | `set_paused` takes no lifecycle lock. A pause before step 6 is read by the new slot when it subscribes; one after it reaches it as any client. A local slot reports its own first status from the pause value (step 6). |
| Stop during the switch | Stop cancels `switching`, then waits for the lock. The switch returns at its next await (model load, joins), shuts down any half-built slot through the same per-slot drain, and answers `error.sessionFallback` with no status; Stop then reports its one Idle. |
| Start during the switch | As Stop. The interface's queue already orders them; the core's compare-and-swap refuses a switch that lands on a newer session whose slot does not match `from`. |
| The other source fails meanwhile | Its status arrives as usual and joins the banner; a second call waits for the lock and switches it next. Nothing of the first switch touches the other slot. |
| The replacement fails at once | Model load or CPU check: the command answers `error.whisperModel` or `error.whisperCpu` (mapped like `start_error`, `commands.rs:118-127`), the slot stays ended, the banner drops Whisper. A capture that fails to open reports through `report_source_failure` (`capture.rs:56`) like any capture failure. |
| The replacement fails later | An ordinary `Error` for that source. The banner may offer again, but never an engine already tried for that origin in this run: no loops. |

### 2. Which engines are offered

Worked out in the interface, from data it already has; nothing new is asked of the core except
key presence, through the existing `has_api_key` (`tauri.ts:97`).

| Engine | Offered when | Data used |
| --- | --- | --- |
| Offline Whisper (v1) | The processor runs it, a model is installed and not downloading, the run is not a rehearsal or the demo, and the source is not already local | `preflight.whisperCpu?.supported` (`preflightController.svelte.ts:103`), the `whisperModels` store, `launch.rehearsalRunning` |
| Another cloud engine (phase 2) | It needs a key under a **different** key name (both Gemini engines share one key, which just failed), `providerCanTranslate(p)` matches the mode, `unsupportedCaptionLanguage({ ...options, provider: p })` is undefined, and `api.hasApiKey(p)` is true | `providers.ts` (`providerKeyName`, `providerCanTranslate`), `languages.ts:31` |

Whisper's model: the setup's `whisperModel` if installed (and able to translate when that is
needed, see 3), else the installed model that fits, preferring one that translates. Its
graphics setting is the setup's `whisperGpu`. The setup itself is not changed: the next Start
uses the engine the operator chose.

Which failures offer it: `error.providerStopped` and `error.providerRejected` on a status with an
origin. Capture failures keep `DeviceRecoveryBanner`. A source still `Reconnecting` after 30 s
(a network outage) is phase 2 (open question 3).

### 3. Mode mismatch

Whisper translates into English only, and Large v3 Turbo not at all (`models.rs:43`,
`options.rs:22-25`).

| Session | Caption language | Engine switched to | Button | Line under it |
| --- | --- | --- | --- | --- |
| Translate | English, model translates | `whisper-translate` | Continue with offline Whisper | Offline Whisper will keep translating into English. |
| Translate | English, only Turbo installed | `whisper` | Continue in the spoken language | Large v3 Turbo cannot translate: captions will continue in the spoken language. |
| Translate | Not English | `whisper` | Continue in the spoken language | Offline Whisper cannot translate into French: captions will continue in the spoken language. |
| Transcribe | (spoken) | `whisper` | Continue with offline Whisper | none |

The consequence is on the button itself, before the click, because the audience's overlay
changes language. The renderer needs a per-track engine (`trackEngines`, beside `originStates`
in `sessionStatus.ts`) so that a line committed after the switch is not noted as being in the
caption language: `transcriptLog.ts` `commit` and `toLine` (`:74-114`), `LiveTurns.svelte:42-51`
and `TranscriptLog.svelte:344` read `laneLanguage(setup, lane)` today and must read "unknown"
for a track whose engine detects the language (`providerDetectsLanguage`).

### 4. Two caption languages

**v1: not offered.** A key or quota failure takes both lanes down together (one key), and
Whisper has no lanes (`options.rs:52-57`, `pipeline.rs` emits `lane: 0`).

Phase 2:

- **Whisper.** Offered once every lane of the source has ended. Whisper captions go to the lane
  whose language is English when the model translates, else to lane 0 as subtitles; the other
  lane stays ended. The banner says `Captions in German stop: offline Whisper writes one
  language.`
- **Cloud.** Offered only for an engine that writes both languages. If one lane still runs, only
  an engine with the same input rate can join it, through a new `RelayHandle::attach(Sender)` on
  `relay_audio`; otherwise the producer would be reopened under the running lane.

### 5. Spending guard, consent and estimate

- **Consent.** Whisper sends nothing (`needsAiConsent` is false, `aiConsent.ts:43`), so v1 asks
  nothing. Phase 2: the click runs the same question as Start. `LaunchController.#consent`
  (`launchController.svelte.ts:79-85`) becomes a public `consent(provider)`, and the command is
  called only after *Send*; *Cancel* or Escape switches nothing. A provider not yet agreed to
  therefore takes two clicks, never one. As at Start, the core does not enforce it
  ([2026-11-05](2026-11-05-mac-app-store.md)): the command is operator-only and carries only
  the interface's choice.
- **Estimate.** Today `estimateSessionCost($options.provider, streamedMs, streams)`
  (`LiveRail.svelte:131`, `providers.ts:147`) prices the whole run at the setup's engine and
  keeps billing a stream that has ended. It becomes a ledger, in a new `billing.ts`:
  segments `{ track, provider, from, to? }` on the `streamedMs` timeline, closed when a track
  ends and opened when it runs again; cost = Σ `hourlyEstimate` × length, so time after a switch
  to Whisper costs nothing. This also stops an ended stream from adding to the estimate, which
  changes today's figure.
- **Guard.** `spendingGuard.svelte.ts:65` checks `isBilled(setup.current.provider)`; it becomes
  "some open segment is billed". The cost warning reads the ledger's total, so spending before
  the switch still counts; the idle pause stops applying once nothing billed is open; the
  duration warning follows the run's clock, which goes on.

### 6. Turn ids and the echo filter

Ids restarting at 0 would break three things keyed on them: the echo filter's remembered system
turns (`remember`, `echo.rs:621-647`, matches by `turn_id`) and its microphone turn
(`echo.rs:461-466`), the Zoom publisher's stream (`zoom/publish.rs:125`), and the interface's
pending turn per track (`transcriptLog.ts:148-176`).

- **`TurnCounter`** (`Arc<AtomicU64>`), one per source and lane, made by `build_source` at Start
  and kept by the slot. `TurnAccumulator::new(clock, lane, counter)` starts at the counter and
  advances it in `next_turn`; `Reconciler` takes it the same way through `LocalSession`. Only one
  client per track runs at a time (step 3 joins the old one first), so ids keep increasing across
  any number of switches, and an aborted client has still advanced the counter.
- **Echo.** `EchoGuard::lengthen_hold` makes the hold `HOLD_LOCAL` (30 s) once any source is
  local, as `start` does (`mod.rs:139-146`); it never shortens a hold and keeps the lanes'
  history. A microphone line in a source's spoken language against system captions translated
  into another language cannot match (`repeats` compares text with text and source with source,
  `echo.rs:652-672`), so after a switch on one source only, the filter lets such echoes through.
  That loses no words; comparing across is deferred.
- **Zoom.** Whisper sends finals only, which `Publisher` always passes, so v1 changes nothing.
  Phase 2 cloud to cloud: `ZoomCaptions::engine_changed(provider)` keeps
  `interims_only_grow` true only if both engines grow their interims (`zoom.rs:331-345`).

### 7. Transcript, history and export

Nothing records the engine today. v1 adds an **automatic marker** at the switch, labelled in the
interface language of the moment (`Room: offline Whisper from here`), through `addMarker`
(`transcriptLog.ts:179`). Markers already reach the history log, the recovery copy and every
export, so a reader sees why a translation turned into a transcript, with no format change.
The history record is the same record: no new `begin`.

Held run. When the last source ends with a failure that has an offer, `applyStatus`
(`sessionStatus.ts:167-176`) does not stop the clock or close the record; the page keeps the
running layout (`$isRunning || $heldForFallback`) and the banner offers the switch and **Not
now**. Not now, Stop or Start end the run as today, and the record's end time is the failure's
(`sessionHistory.finish(at)`, `history.ts:448`), so the duration does not run on, as
[history](../transcript-history.md) promises.

### 8. Errors and strings

One new error id, `error.sessionFallback` (`id::SESSION_FALLBACK` in `errors.rs`), for a switch
refused or cut short. Whisper failures keep `error.whisperModel` and `error.whisperCpu`.

New keys in `en.ts`, `fr.ts` and `de.ts` (French and German written with the code):

| Key | English |
| --- | --- |
| `error.sessionFallback` | The source could not be continued with another engine |
| `fallback.region` | Continue the session |
| `fallback.ended(source, engine)` | `${source}: ${engine} stopped captioning.` |
| `fallback.whisper` | Continue with offline Whisper |
| `fallback.spoken` | Continue in the spoken language |
| `fallback.keepsEnglish` | Offline Whisper will keep translating into English. |
| `fallback.cannotTranslate(language)` | `Offline Whisper cannot translate into ${language}: captions will continue in the spoken language.` |
| `fallback.turboCannotTranslate` | Large v3 Turbo cannot translate: captions will continue in the spoken language. |
| `fallback.bothSources` | Both sources continue with it. |
| `fallback.switching` | Switching engine… |
| `fallback.continued(source, engine)` | `${source} continues with ${engine}.` |
| `fallback.marker(source, engine)` | `${source}: ${engine} from here` |
| `fallback.notNow` | Not now |
| `fallback.engine(engine)` (phase 2) | `Continue with ${engine}` |
| `fallback.secondLaneStops(language)` (phase 2) | `Captions in ${language} stop: offline Whisper writes one language.` |

Mac wording needs no override: none of these names a Windows setting.

### 9. Privacy

- **v1 (Whisper):** no new recipient; the source's audio stops going to the provider and stays on
  the computer from the switch, with Whisper's temporary file from then on. `privacy.md` gains
  one sentence under *Audio*, with a new effective date: *If a cloud source stops during a
  session, the user can continue it with local Whisper; from then on that source's audio stays
  on the computer.*
- **Phase 2 (another cloud engine):** a new provider receives audio mid-session. `privacy.md`
  must say so under *Third-party processing*, with a new effective date: *If a cloud source
  stops during a session, the user can continue it with another provider whose key is saved.
  That provider receives the source's audio from then on, after the same question if the user
  has not yet agreed to it.* `store-listing-next.md` advertises the feature in either phase.

### 10. Tests

Rust, with no billable call: loopback servers and the Whisper stand-in that `drive` already takes
(`pipeline.rs:180-183`).

| Test | Where | Proves |
| --- | --- | --- |
| `a_fatal_end_then_a_switch_continues_the_source_with_increasing_turn_ids` | `realtime/tests.rs` | Server A sends two finals then a quota error: `Error(providerStopped)`. A second `run_session` on server B with the same `TurnCounter` and clock: `Connecting`, `Running`, finals whose ids are greater than A's and whose `start_ms` do not go back. `Recorder` gains the turn id. |
| `the_other_source_captions_through_a_switch` | `realtime/tests.rs` | Microphone on server M keeps its captions and ids, with no status, while System on server S fails and is replaced. |
| `a_local_replacement_continues_the_turn_ids` | `whisper/tests.rs` | `drive` with the stand-in and a counter at N emits ids from N. |
| `a_replacement_started_while_paused_reports_paused` | `whisper/tests.rs` | No `Running` before `Paused`. |
| `stop_during_a_switch_does_not_wait_for_the_model` | `session/mod.rs` tests | A stand-in load waiting on its token: Stop returns at once, one Idle. |
| `a_switch_is_refused_unless_the_source_ended_on_the_expected_engine` | `session/mod.rs` tests | A second call, a live slot, or a newer session: `error.sessionFallback`, nothing touched. |
| `a_mixed_session_drains_each_source_by_its_engine` | `session/mod.rs` tests | Stop waits for the local client and cuts off the cloud one after `CLIENT_DRAIN_TIMEOUT`. |
| `lengthening_the_hold_keeps_what_was_heard` | `echo.rs` tests | A held line still waits; remembered system turns survive. |
| command registration | `command_names.rs` tests | Registered, granted to the operator, absent from the overlay. |

Interface (Vitest and component tests):

- `sourceFallback.test.ts`: eligible ids and origins; the table in 3 row by row; Turbo only; no
  model; processor refused; rehearsal, demo and two languages hidden; tried engines dropped;
  Both sources continued by one click, one call each, in order; a refused call removes the offer.
- `SourceFallbackBanner.svelte.test.ts`: the region's accessible name, the button and line per
  row of 3, disabled while switching, Not now ends the held run.
- `sessionStatus` tests: an eligible last failure holds the run (clock, record open); Not now
  closes the record at the failure's time; the replacement's `Running` resumes the run.
- `transcriptLog`/`lanes.test.ts`: a line after a switch to subtitles is not noted as the
  caption language; the marker is added once.
- `billing.test.ts`: the ledger closes and opens segments; Whisper time is free.
- `contract.test.ts` for `SourceSwitch`; the catalogue parity test for the new keys;
  `npm run test:style` for the banner.

### 11. Scope and phasing

**v1, the smallest useful version:**

- Offline Whisper only, single caption language, live sessions (no rehearsal, no demo).
- Offered on `error.providerStopped` and `error.providerRejected`, and on a source that has been
  `Reconnecting` for 30 s (decision 3); one click continues every source in either state.
- `SourceSlot` refactor, `switch_source_engine`, `TurnCounter`, the held run, the automatic
  marker, the cost ledger, `error.sessionFallback` and the strings in three languages.
- `privacy.md` sentence and `store-listing-next.md` line.

**Deferred:** another cloud engine (with its consent step, Zoom rule and privacy paragraph); two
caption languages; a fallback from Whisper or the demo; comparing
across translation and transcript in the echo filter; recording the engine in the history
header.

**Platforms.** Nothing differs per system: the switch reopens the device through the same
`CaptureTarget` as Start and device recovery. On a Mac, reopening a process tap mid-session is
untried and must be checked once by hand.

**Rough size (v1).** Rust about 700 lines, half of them tests (slot refactor 200, switch 200,
counter and pause 60); interface about 600 lines, half of them tests (controller 150, banner 80,
held run 60, ledger 60); docs 40. Three to four days, best as two pull requests: the
`SourceSlot` refactor with no change in behaviour, then the feature.

### Decisions taken

The maintainer agreed the proposal on 23 November 2026, with these answers to the questions
that follow:

1. **Held run:** yes. The clock and the history record stay open while the banner offers the
   switch after the last source ended. If the operator declines (Stop, or dismissing the
   banner), the run's end time is set back to the moment of the failure.
2. **Engine change in documents:** the automatic marker. The engine in the history header is
   deferred.
3. **Network outages:** in v1. After a source has been `Reconnecting` for 30 s, the banner offers
   the switch beside the reconnection, which carries on until the operator clicks; the click
   cancels that source's reconnection and starts Whisper in its place.
4. **Spoken language instead of a translation:** offer Whisper, with the warning on the button
   that the captions will be in the spoken language.
5. **Whisper model:** the setup's model, else the best installed one that fits.
6. **Cost estimate:** accepted. A stream that has ended stops adding to the estimate.
7. **Phase 2 (another cloud engine):** not now. Revisit once v1 has been used at an event.

### Open questions for the maintainer (as asked)

1. **Held run.** Keep the clock and history record open while the banner offers a switch after
   the last source ended (proposed), with the end time set back to the failure if the operator
   declines?
2. **Engine change in documents.** An automatic marker (proposed), or a record of its own and
   the engine in the history header?
3. **Network outages.** Offer the switch after a source has been `Reconnecting` for 30 s,
   cancelling it, in v1 rather than phase 2?
4. **Spoken language instead of a translation.** Offer Whisper with the warning on the button
   (proposed), or never when the room reads a translation not in English?
5. **Whisper model.** The setup's model, else the best installed one that fits (proposed), or
   always the smallest for speed?
6. **Cost estimate.** Accept that an ended stream stops adding to the estimate, which changes
   today's figure?
7. **Phase 2.** Is continuing with another cloud provider wanted at all, given the consent step
   and the privacy change?

## Consequences

- A key or quota failure no longer costs the event its clock, transcript continuity or history
  record.
- `ActiveSession` holds one `SourceSlot` per source, which makes a session with a cloud and a
  local source possible; Stop, Pause and Discard follow each slot's engine.
- Turn ids increase across a source's whole run, whatever engine produces them.
