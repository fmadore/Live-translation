//! Session orchestration: one bounded audio pipeline and realtime client per source.
//!
//! This module owns the lifecycle — start, pause, stop — and what a running session holds.
//! `options` validates a start and decides its sources and caption languages, `settings`
//! resolves the provider and spawns its clients, `builder` wires each source's producer to
//! them, `capture` covers the devices, and `preflight` is the level-only test that shares the
//! lifecycle lock.

mod builder;
mod capture;
mod options;
mod preflight;
mod settings;

use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

use anyhow::{Context, Result};
use futures_util::future::join_all;
use tauri::async_runtime::JoinHandle as AsyncJoinHandle;
use tauri::{AppHandle, Manager};
use tokio::sync::Mutex as AsyncMutex;
use tokio::sync::mpsc::Sender;
use tokio::sync::watch;
use tokio_util::sync::CancellationToken;

use crate::errors::AppError;
use crate::realtime::Events;
use crate::secrets;
use crate::sync::lock;
use crate::timing::SessionClock;
use crate::types::{AudioLevel, Origin, Provider, SessionState, StartOptions, StatusUpdate};
use crate::whisper::{self, LoadedModel};
use builder::SessionBuilder;
use capture::{CaptureTarget, join_threads, spawn_level_forwarder};
use options::{is_live, session_origins, validate_start};
use preflight::ActiveTest;
use settings::ProviderSettings;

const CLIENT_DRAIN_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Default)]
pub struct SessionManager {
    /// Serializes concurrent start/stop commands so an older request cannot tear down a
    /// newly-started session.
    lifecycle: AsyncMutex<()>,
    active: Mutex<Option<ActiveSession>>,
    /// Preflight level test. Shares `lifecycle` with the session, so the two can never be
    /// holding the same capture device at once.
    active_test: Mutex<Option<ActiveTest>>,
    local_abort: Mutex<Option<CancellationToken>>,
}

struct ActiveSession {
    cancel: CancellationToken,
    failure: Arc<Mutex<Option<AppError>>>,
    abort: CancellationToken,
    /// Every client holds a receiver; see `SessionManager::set_paused`.
    pause: watch::Sender<bool>,
    /// One per source, in the order `start` added them.
    slots: Vec<SourceSlot>,
    /// What a source is built from, kept for as long as the session runs.
    options: StartOptions,
    /// One clock for the session, copied into every source, so the microphone and system
    /// timelines agree in a transcript that interleaves them. See `timing::SessionClock`.
    clock: SessionClock,
    capture: CaptureTarget,
    level_tx: Sender<AudioLevel>,
    /// The Whisper model a local source decodes with, loaded once for the session.
    model: Option<Arc<LoadedModel>>,
}

/// Everything one source of a session runs: what feeds its audio channel and the clients that
/// read it, with the engine they speak and the token that stops them.
struct SourceSlot {
    origin: Origin,
    engine: Provider,
    /// A child of the session's token; cancelled when the source ends, by Stop or by itself.
    cancel: CancellationToken,
    /// Cancelled when Stop gives up on a capture thread that did not stop in time. Such a
    /// thread still holds its source's audio channel open, so a local transcriber waiting for
    /// the channel to close would hold Stop forever; this tells it to finish without the end.
    abandoned: CancellationToken,
    capture: Option<JoinHandle<()>>,
    /// The timer-driven audio producer of a commercial-provider rehearsal.
    fixture: Option<AsyncJoinHandle<()>>,
    /// With a second caption language, the relay that copies the audio to both clients.
    relay: Option<AsyncJoinHandle<()>>,
    clients: Vec<AsyncJoinHandle<()>>,
}

impl SourceSlot {
    fn new(origin: Origin, engine: Provider, cancel: CancellationToken) -> Self {
        Self {
            origin,
            engine,
            cancel,
            abandoned: CancellationToken::new(),
            capture: None,
            fixture: None,
            relay: None,
            clients: Vec::new(),
        }
    }

    /// Whisper has no connection to close, so a pause and Stop are the manager's to handle.
    fn is_local(&self) -> bool {
        self.engine.is_local_whisper()
    }

    fn is_running(&self) -> bool {
        !self.cancel.is_cancelled()
    }
}

/// Publish a pause or resume to every client. False when it changes nothing: an unchanged
/// value must wake no client, or a repeated Resume would cut a reconnect backoff short.
fn signal_pause(pause: &watch::Sender<bool>, paused: bool) -> bool {
    pause.send_if_modified(|current| std::mem::replace(current, paused) != paused)
}

impl SessionManager {
    /// Whether a session has been started and not yet stopped. The update check (`update.rs`)
    /// asks, so that no request of its own goes out during an event.
    pub fn is_active(&self) -> bool {
        lock(&self.active).is_some()
    }

    pub async fn start(&self, app: &AppHandle, options: StartOptions) -> Result<()> {
        let _lifecycle = self.lifecycle.lock().await;
        self.stop_active(app).await;
        // A preflight test is holding the very devices this session is about to open.
        self.stop_test_active(app).await;
        validate_start(&options)?;
        // Turn ids start again with this session, and how much of an interim caption a Zoom
        // share may send depends on the engine. A rehearsal or the demonstration is never
        // shared: a share left running must not show the meeting sample captions.
        app.state::<crate::zoom::ZoomCaptions>()
            .begin_session(options.provider, is_live(&options));

        let provider = options.provider;
        // The built-in demo and local Whisper start with no credential.
        // Credential Manager is a blocking call, and this holds the lifecycle lock: keep it off
        // the async workers that are pumping the other windows' events meanwhile.
        let api_key = if provider.requires_api_key() {
            tauri::async_runtime::spawn_blocking(move || secrets::resolve_api_key(provider))
                .await
                .context("keychain lookup did not complete")??
        } else {
            String::new()
        };
        // A key from the environment never went through `set_api_key`'s check.
        secrets::check_plan(provider, &api_key)?;
        let settings = ProviderSettings::resolve(provider)?
            .with_glossary(&options.glossary)
            .with_spoken_language(options.spoken_language.as_deref());
        // Made before the model loads, so discarding (and quitting) also ends a load that hangs.
        let abort = CancellationToken::new();
        let local_model = if provider.is_local_whisper() {
            *lock(&self.local_abort) = Some(abort.clone());
            let (app, abort) = (app.clone(), abort.clone());
            let (model, graphics) = (options.whisper_model, options.whisper_gpu);
            Some(
                tauri::async_runtime::spawn_blocking(move || {
                    whisper::load(&app, model, graphics, &abort)
                })
                .await
                .context("Whisper model loading stopped unexpectedly")??,
            )
        } else {
            None
        };

        let cancel = CancellationToken::new();
        let cancel_guard = cancel.clone().drop_guard();
        let origins = session_origins(&options);
        // Only a live session hearing both the room and the computer can caption one voice
        // twice, and only the operator decides to drop the copy; the demonstration scripts its
        // two sources apart.
        let both = origins.contains(&Origin::Microphone) && origins.contains(&Origin::System);
        app.state::<crate::echo::EchoGuard>().begin(
            options.drop_mic_echo && both && provider != Provider::OnDevice,
            if provider.is_local_whisper() {
                crate::echo::HOLD_LOCAL
            } else {
                crate::echo::HOLD_CLOUD
            },
        );
        let mut session = ActiveSession {
            cancel,
            failure: Arc::new(Mutex::new(None)),
            abort,
            pause: watch::Sender::new(false),
            slots: Vec::with_capacity(origins.len()),
            capture: CaptureTarget::for_session(&options),
            options,
            clock: SessionClock::start(),
            level_tx: spawn_level_forwarder(app),
            model: local_model,
        };
        let builder = SessionBuilder {
            app,
            settings,
            api_key,
        };
        for origin in origins {
            let mut slot = SourceSlot::new(origin, provider, session.cancel.child_token());
            let built = builder.build_source(&session, &mut slot);
            session.slots.push(slot);
            if let Err(error) = built {
                // The sources before this one are already capturing and connecting. Release
                // them as Stop would, so a failed start leaves no thread, task or token behind.
                // No Idle: the start reports its own failure, and a late Idle would clear it.
                self.shutdown(session).await;
                return Err(error);
            }
        }

        cancel_guard.disarm();
        *lock(&self.active) = Some(session);
        Ok(())
    }

    /// Pause or resume the running session. While paused every client closes its provider
    /// connection — so nothing is streamed or billed — and capture keeps metering, so the
    /// operator can see when the room starts talking again. Not behind the lifecycle lock: a
    /// pause is a signal to the clients, and must not wait out a start or stop. `events` is the
    /// app, or the tests' recorder.
    pub fn set_paused(&self, events: &impl Events, paused: bool) -> Result<()> {
        let active = lock(&self.active);
        let session = active
            .as_ref()
            .filter(|session| session.slots.iter().any(SourceSlot::is_running))
            .context("No session is running")?;
        if !signal_pause(&session.pause, paused) {
            return Ok(());
        }
        for slot in &session.slots {
            if !slot.is_local() || !slot.is_running() {
                continue;
            }
            let state = if paused {
                SessionState::Paused
            } else {
                SessionState::Running
            };
            events.status(StatusUpdate::source(slot.origin, state, None));
        }
        Ok(())
    }

    /// Available while Stop owns the lifecycle lock and drains a long local backlog.
    pub fn discard_local_pending(&self) {
        if let Some(abort) = lock(&self.local_abort).as_ref() {
            abort.cancel();
        }
        if let Some(session) = lock(&self.active)
            .as_ref()
            .filter(|s| s.slots.iter().any(SourceSlot::is_local))
        {
            session.cancel.cancel();
        }
    }

    pub async fn stop(&self, events: &impl Events) {
        let _lifecycle = self.lifecycle.lock().await;
        self.stop_active(events).await;
    }

    async fn stop_active(&self, events: &impl Events) {
        let session = lock(&self.active).take();
        if let Some(session) = session {
            let failure = session.failure.clone();
            self.shutdown(session).await;
            events.status(StatusUpdate::session(
                SessionState::Idle,
                lock(&failure).clone(),
            ));
            tracing::info!("session stopped");
        }
    }

    /// Cancel a session and wait for everything it started: capture, playback, relays and
    /// clients. Shared by Stop and by a start that failed part-way.
    async fn shutdown(&self, session: ActiveSession) {
        session.cancel.cancel();
        let local = session.slots.iter().any(SourceSlot::is_local);
        drain_slots(session.slots).await;
        if local {
            *lock(&self.local_abort) = None;
        }
    }
}

/// Wait for everything some cancelled sources started, each source by its own engine and all
/// of them at once: first what feeds them, then their clients.
async fn drain_slots(slots: Vec<SourceSlot>) {
    join_all(slots.into_iter().map(drain_slot)).await;
}

async fn drain_slot(mut slot: SourceSlot) {
    if let Some(thread) = slot.capture.take()
        && !join_threads(vec![thread], "capture").await
    {
        // macOS can do this while it waits for the operator to answer the System Audio
        // Recording question: every Core Audio call of the app's waits with it.
        slot.abandoned.cancel();
    }

    // Rehearsal playback holds the producer end of its audio channel, and the client below
    // only sees the stream end once that is dropped — so drain it here, in the same place the
    // capture thread is joined.
    // A relay ends once its producer has: the capture thread is joined and the rehearsal task
    // is next, so this cannot wait on anything still running.
    for task in [slot.fixture.take(), slot.relay.take()]
        .into_iter()
        .flatten()
    {
        if let Err(error) = task.await {
            tracing::warn!("rehearsal playback task failed: {error}");
        }
    }

    // Providers may emit their last transcript while flushing. Do not report Idle (or start a
    // replacement session) until that bounded drain has completed.
    if slot.is_local() {
        // Local Stop is an EOF, not cancellation of inference. Do not truncate the transcript
        // at the cloud client's five-second shutdown deadline.
        for result in join_all(slot.clients.iter_mut()).await {
            if let Err(error) = result {
                tracing::warn!("local transcription task failed: {error}");
            }
        }
    } else {
        let aborts: Vec<_> = slot
            .clients
            .iter()
            .map(|task| task.inner().abort_handle())
            .collect();
        // One `JoinAll`, polled again after the timeout: it never polls a finished handle
        // twice, which a `JoinHandle` does not allow.
        let joined = join_all(slot.clients.iter_mut());
        tokio::pin!(joined);
        if tokio::time::timeout(CLIENT_DRAIN_TIMEOUT, &mut joined)
            .await
            .is_err()
        {
            tracing::warn!("realtime clients did not finish graceful shutdown in time");
            for abort in &aborts {
                abort.abort();
            }
            // A client cut off finalizes its open turn as it is dropped (`run_session`). Wait
            // for that, so its last line comes before the Idle and not into the next session.
            joined.await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::id;

    /// The statuses a session reports, recorded in place of the app's events. Shared by the
    /// submodules' tests.
    #[derive(Default)]
    pub(super) struct Statuses(Mutex<Vec<StatusUpdate>>);

    impl Statuses {
        pub(super) fn take(&self) -> Vec<StatusUpdate> {
            std::mem::take(&mut lock(&self.0))
        }
    }

    impl Events for Statuses {
        fn caption(&self, _: crate::types::Caption<'_>) {}

        fn status(&self, update: StatusUpdate) {
            lock(&self.0).push(update);
        }
    }

    /// A running session as `start` leaves one, minus its devices: one source per origin, each
    /// with the client task `client` makes from its token.
    fn session(
        local: bool,
        origins: &[Origin],
        client: impl Fn(CancellationToken) -> AsyncJoinHandle<()>,
    ) -> ActiveSession {
        let cancel = CancellationToken::new();
        let engine = if local {
            Provider::Whisper
        } else {
            Provider::Gemini
        };
        let slots = origins
            .iter()
            .map(|&origin| {
                let mut slot = SourceSlot::new(origin, engine, cancel.child_token());
                slot.clients = vec![client(slot.cancel.clone())];
                slot
            })
            .collect();
        let options = options("microphone", None);
        ActiveSession {
            failure: Arc::new(Mutex::new(None)),
            abort: CancellationToken::new(),
            pause: watch::Sender::new(false),
            slots,
            cancel,
            capture: CaptureTarget::for_session(&options),
            options,
            clock: SessionClock::start(),
            level_tx: tokio::sync::mpsc::channel(1).0,
            model: None,
        }
    }

    /// A client that ends with its source, as `run_session` does after its drain.
    fn ends_with_its_source(source: CancellationToken) -> AsyncJoinHandle<()> {
        AsyncJoinHandle::Tokio(tokio::spawn(async move { source.cancelled().await }))
    }

    fn manager_with(session: ActiveSession) -> SessionManager {
        let manager = SessionManager::default();
        *lock(&manager.active) = Some(session);
        manager
    }

    /// Every source has ended on its own — say each failed — but the session is still the
    /// active one until Stop. A pause has nothing left to pause.
    #[tokio::test]
    async fn a_session_whose_sources_have_all_ended_cannot_be_paused() {
        let ended = session(false, &[Origin::Microphone], ends_with_its_source);
        ended.slots[0].cancel.cancel();
        let manager = manager_with(ended);
        let events = Statuses::default();
        assert!(manager.set_paused(&events, true).is_err());
        assert!(events.take().is_empty());
    }

    /// Whisper has no connection to close, so the manager reports a local pause itself — for
    /// the sources still running, and only when the value changes.
    #[tokio::test]
    async fn a_local_pause_is_reported_for_each_source_still_running() {
        let active = session(
            true,
            &[Origin::Microphone, Origin::System],
            ends_with_its_source,
        );
        active.slots[1].cancel.cancel();
        let manager = manager_with(active);
        let events = Statuses::default();

        manager.set_paused(&events, true).unwrap();
        manager.set_paused(&events, true).unwrap();
        manager.set_paused(&events, false).unwrap();
        let reported: Vec<_> = events
            .take()
            .into_iter()
            .map(|update| (update.origin, update.state))
            .collect();
        assert_eq!(
            reported,
            [
                (Some(Origin::Microphone), SessionState::Paused),
                (Some(Origin::Microphone), SessionState::Running),
            ]
        );
    }

    /// Stop reports Idle once, for the whole session, after every client has drained, and
    /// carries the failure that ended it so the operator still sees why.
    #[tokio::test]
    async fn stop_reports_one_idle_with_the_failure_that_ended_the_session() {
        let active = session(true, &[Origin::Microphone], ends_with_its_source);
        let source = active.slots[0].cancel.clone();
        *lock(&active.failure) = Some(AppError::with(id::WHISPER_SESSION, "disk full"));
        let manager = manager_with(active);
        let events = Statuses::default();

        manager.stop(&events).await;
        assert!(source.is_cancelled());
        assert!(lock(&manager.active).is_none());
        let [idle] = &events.take()[..] else {
            panic!("expected exactly one status");
        };
        assert_eq!(idle.state, SessionState::Idle);
        assert_eq!(idle.origin, None);
        assert_eq!(
            idle.message.as_ref().map(|error| error.id),
            Some(id::WHISPER_SESSION)
        );

        // Nothing is left to stop.
        manager.stop(&events).await;
        assert!(events.take().is_empty());
    }

    /// A cloud client that is still draining after `CLIENT_DRAIN_TIMEOUT` is cut off, so Stop,
    /// and the next start behind it, cannot wait on a provider that never answers.
    #[tokio::test(start_paused = true)]
    async fn stop_cuts_off_a_cloud_client_that_does_not_drain_in_time() {
        let (dropped_tx, dropped) = tokio::sync::oneshot::channel::<()>();
        let dropped_tx = Mutex::new(Some(dropped_tx));
        let stuck = |_| {
            let guard = lock(&dropped_tx).take();
            AsyncJoinHandle::Tokio(tokio::spawn(async move {
                let _guard = guard;
                std::future::pending::<()>().await;
            }))
        };
        let manager = manager_with(session(false, &[Origin::System], stuck));
        let events = Statuses::default();

        let started = tokio::time::Instant::now();
        manager.stop(&events).await;
        assert!(started.elapsed() >= CLIENT_DRAIN_TIMEOUT);
        // The aborted task dropped what it held.
        assert!(dropped.await.is_err());
        assert_eq!(events.take().len(), 1, "Idle, once");
    }

    /// Each source is drained by its own engine: Stop waits out a local transcriber's whole
    /// backlog, however long, and cuts a cloud client off after `CLIENT_DRAIN_TIMEOUT`, in the
    /// same session and at the same time.
    #[tokio::test(start_paused = true)]
    async fn a_mixed_session_drains_each_source_by_its_engine() {
        let backlog = CLIENT_DRAIN_TIMEOUT * 3;
        let mut active = session(true, &[Origin::Microphone], |source| {
            AsyncJoinHandle::Tokio(tokio::spawn(async move {
                source.cancelled().await;
                tokio::time::sleep(backlog).await;
            }))
        });
        let (dropped_tx, dropped) = tokio::sync::oneshot::channel::<()>();
        let mut cloud = SourceSlot::new(
            Origin::System,
            Provider::Gemini,
            active.cancel.child_token(),
        );
        cloud.clients = vec![AsyncJoinHandle::Tokio(tokio::spawn(async move {
            let _guard = dropped_tx;
            std::future::pending::<()>().await;
        }))];
        active.slots.push(cloud);
        let manager = manager_with(active);
        let events = Statuses::default();

        let started = tokio::time::Instant::now();
        manager.stop(&events).await;
        assert!(
            started.elapsed() >= backlog,
            "the local backlog was cut short"
        );
        assert!(
            started.elapsed() < backlog + CLIENT_DRAIN_TIMEOUT,
            "drained one after the other"
        );
        assert!(dropped.await.is_err(), "the cloud client was cut off");
        assert_eq!(events.take().len(), 1, "Idle, once");
    }

    /// A capture thread stuck in a driver call (macOS does this while the System Audio
    /// Recording question is open) holds its source's audio channel, so a local transcriber
    /// waiting for that channel to close would hold Stop forever. Stop gives up on the thread
    /// after `CAPTURE_JOIN_TIMEOUT` and cancels `abandoned`, which ends the transcriber, so Stop
    /// completes and reports Idle once.
    #[tokio::test(flavor = "multi_thread")]
    async fn stop_completes_past_a_wedged_capture_thread_by_abandoning_its_input() {
        let (release_tx, release) = std::sync::mpsc::channel::<()>();
        let mut active = session(true, &[Origin::System], ends_with_its_source);
        // Deaf to cancellation, as a thread inside a blocked Core Audio call is.
        active.slots[0].capture = Some(std::thread::spawn(move || {
            let _ = release.recv();
        }));
        let abandoned = active.slots[0].abandoned.clone();
        // Ends only on `abandoned`, as the Whisper source does while its input is still open.
        active.slots[0].clients = vec![AsyncJoinHandle::Tokio(tokio::spawn(
            abandoned.clone().cancelled_owned(),
        ))];
        let manager = manager_with(active);
        let events = Statuses::default();

        let started = std::time::Instant::now();
        tokio::time::timeout(Duration::from_secs(20), manager.stop(&events))
            .await
            .expect("Stop waited on the wedged capture thread");
        assert!(started.elapsed() >= Duration::from_secs(3));
        assert!(abandoned.is_cancelled());
        let [idle] = &events.take()[..] else {
            panic!("expected exactly one status");
        };
        assert_eq!(idle.state, SessionState::Idle);
        let _ = release_tx.send(());
    }

    /// Threads that stop in time are joined and nothing is abandoned.
    #[tokio::test]
    async fn stop_abandons_nothing_when_the_capture_threads_stop() {
        let mut active = session(false, &[Origin::Microphone], ends_with_its_source);
        let source = active.slots[0].cancel.clone();
        active.slots[0].capture = Some(std::thread::spawn(move || {
            while !source.is_cancelled() {
                std::thread::sleep(Duration::from_millis(5));
            }
        }));
        let abandoned = active.slots[0].abandoned.clone();
        let manager = manager_with(active);
        manager.stop(&Statuses::default()).await;
        assert!(!abandoned.is_cancelled());
    }

    /// A start that fails part-way releases what it had started through `shutdown`: every
    /// source is cancelled, its threads joined and its clients drained, the local abort is
    /// dropped, and nothing becomes the active session. `shutdown` reports no status at all, so
    /// no Idle can clear the failure the start reports itself.
    #[tokio::test]
    async fn a_failed_start_is_shut_down_without_an_idle_or_an_active_session() {
        let mut partial = session(true, &[Origin::Microphone], ends_with_its_source);
        let source = partial.slots[0].cancel.clone();
        let thread_source = source.clone();
        let (joined_tx, joined) = std::sync::mpsc::channel::<()>();
        partial.slots[0].capture = Some(std::thread::spawn(move || {
            while !thread_source.is_cancelled() {
                std::thread::sleep(Duration::from_millis(5));
            }
            let _ = joined_tx.send(());
        }));
        let abandoned = partial.slots[0].abandoned.clone();
        let manager = SessionManager::default();
        *lock(&manager.local_abort) = Some(CancellationToken::new());

        manager.shutdown(partial).await;
        assert!(source.is_cancelled());
        assert!(joined.try_recv().is_ok(), "the capture thread was joined");
        assert!(!abandoned.is_cancelled());
        assert!(lock(&manager.local_abort).is_none());
        assert!(lock(&manager.active).is_none());
        // With nothing active, a Stop behind the failed start has nothing to report either.
        let events = Statuses::default();
        manager.stop(&events).await;
        assert!(events.take().is_empty());
    }

    /// A Gemini start captioning in French from `source`, shared by the submodules' tests.
    pub(super) fn options(source: &str, rehearsal: Option<&str>) -> StartOptions {
        serde_json::from_value(serde_json::json!({
            "source": source,
            "targetLanguage": "fr",
            "provider": "gemini",
            "rehearsal": rehearsal,
        }))
        .unwrap()
    }

    #[test]
    fn an_unchanged_pause_wakes_no_client() {
        let pause = watch::Sender::new(false);
        let mut client = pause.subscribe();
        assert!(!signal_pause(&pause, false));
        assert!(
            !client.has_changed().unwrap(),
            "a redundant Resume woke a client"
        );

        assert!(signal_pause(&pause, true));
        assert!(client.has_changed().unwrap());
        assert!(*client.borrow_and_update());
        assert!(!signal_pause(&pause, true));
        assert!(!client.has_changed().unwrap());
    }
}
