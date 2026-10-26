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
use tokio::sync::watch;
use tokio::sync::Mutex as AsyncMutex;
use tokio_util::sync::CancellationToken;

use crate::errors::AppError;
use crate::realtime::Events;
use crate::secrets;
use crate::sync::lock;
use crate::timing::SessionClock;
use crate::types::{Origin, SessionState, StartOptions, StatusUpdate};
use crate::whisper;
use builder::SessionBuilder;
use capture::{join_threads, spawn_level_forwarder, CaptureTarget};
use options::{session_origins, validate_start};
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
    local: bool,
    failure: Arc<Mutex<Option<AppError>>>,
    abort: CancellationToken,
    /// One per source, in the order `start` added them, so `set_paused` can zip the two.
    origins: Vec<Origin>,
    /// Every client holds a receiver; see `SessionManager::set_paused`.
    pause: watch::Sender<bool>,
    sources: Vec<CancellationToken>,
    capture_threads: Vec<JoinHandle<()>>,
    /// Timer-driven audio producers used by commercial-provider rehearsal playback.
    fixture_tasks: Vec<AsyncJoinHandle<()>>,
    /// With a second caption language, one relay per source copies its audio to both clients.
    relay_tasks: Vec<AsyncJoinHandle<()>>,
    client_tasks: Vec<AsyncJoinHandle<()>>,
}

/// Publish a pause or resume to every client. False when it changes nothing: an unchanged
/// value must wake no client, or a repeated Resume would cut a reconnect backoff short.
fn signal_pause(pause: &watch::Sender<bool>, paused: bool) -> bool {
    pause.send_if_modified(|current| std::mem::replace(current, paused) != paused)
}

impl SessionManager {
    pub async fn start(&self, app: &AppHandle, options: StartOptions) -> Result<()> {
        let _lifecycle = self.lifecycle.lock().await;
        self.stop_active(app).await;
        // A preflight test is holding the very devices this session is about to open.
        self.stop_test_active(app).await;
        validate_start(&options)?;
        // Turn ids start again with this session, and how much of an interim caption a Zoom
        // share may send depends on the engine.
        app.state::<crate::zoom::ZoomCaptions>()
            .begin_session(options.provider);

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
        let settings = ProviderSettings::resolve(provider)?.with_glossary(&options.glossary);
        let local_model = if provider.is_local_whisper() {
            let app = app.clone();
            let model = options.whisper_model;
            Some(
                tauri::async_runtime::spawn_blocking(move || whisper::load(&app, model))
                    .await
                    .context("Whisper model loading stopped unexpectedly")??,
            )
        } else {
            None
        };

        let cancel = CancellationToken::new();
        let cancel_guard = cancel.clone().drop_guard();
        let abort = CancellationToken::new();
        if provider.is_local_whisper() {
            *lock(&self.local_abort) = Some(abort.clone());
        }
        let origins = session_origins(&options);
        let mut builder = SessionBuilder {
            app,
            options: &options,
            settings,
            api_key,
            capture: CaptureTarget::for_session(&options),
            target_rate: provider.input_sample_rate(),
            // One clock for the session, copied into every source, so the microphone and
            // system timelines agree in a transcript that interleaves them. See
            // `timing::SessionClock`.
            clock: SessionClock::start(),
            level_tx: spawn_level_forwarder(app),
            session: ActiveSession {
                cancel,
                local: provider.is_local_whisper(),
                failure: Arc::new(Mutex::new(None)),
                abort,
                origins: origins.clone(),
                pause: watch::Sender::new(false),
                sources: Vec::new(),
                capture_threads: Vec::new(),
                fixture_tasks: Vec::new(),
                relay_tasks: Vec::new(),
                client_tasks: Vec::new(),
            },
            local_model,
        };
        for &origin in &origins {
            if let Err(error) = builder.add_source(origin) {
                // The sources before this one are already capturing and connecting. Release
                // them as Stop would, so a failed start leaves no thread, task or token behind.
                // No Idle: the start reports its own failure, and a late Idle would clear it.
                self.shutdown(builder.session).await;
                return Err(error);
            }
        }

        cancel_guard.disarm();
        *lock(&self.active) = Some(builder.session);
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
            .filter(|session| session.sources.iter().any(|source| !source.is_cancelled()))
            .context("No session is running")?;
        if !signal_pause(&session.pause, paused) {
            return Ok(());
        }
        if session.local {
            for (origin, source) in session.origins.iter().zip(&session.sources) {
                if source.is_cancelled() {
                    continue;
                }
                let state = if paused {
                    SessionState::Paused
                } else {
                    SessionState::Running
                };
                events.status(StatusUpdate::source(*origin, state, None));
            }
        }
        Ok(())
    }

    /// Available while Stop owns the lifecycle lock and drains a long local backlog.
    pub fn discard_local_pending(&self) {
        if let Some(abort) = lock(&self.local_abort).as_ref() {
            abort.cancel();
        }
        if let Some(session) = lock(&self.active).as_ref().filter(|s| s.local) {
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
    async fn shutdown(&self, mut session: ActiveSession) {
        session.cancel.cancel();
        join_threads(session.capture_threads, "capture").await;

        // Rehearsal playback holds the producer end of its audio channel, and the client
        // below only sees the stream end once that is dropped — so drain it here, in the
        // same place the capture threads are joined.
        // A relay ends once its producer has: the capture threads are joined and the
        // rehearsal tasks are next, so this cannot wait on anything still running.
        for result in join_all(
            session
                .fixture_tasks
                .iter_mut()
                .chain(session.relay_tasks.iter_mut()),
        )
        .await
        {
            if let Err(error) = result {
                tracing::warn!("rehearsal playback task failed: {error}");
            }
        }

        // Providers may emit their last transcript while flushing. Do not report Idle
        // (or start a replacement session) until that bounded drain has completed.
        if session.local {
            // Local Stop is an EOF, not cancellation of inference. Do not truncate the
            // transcript at the cloud client's five-second shutdown deadline.
            for result in join_all(session.client_tasks.iter_mut()).await {
                if let Err(error) = result {
                    tracing::warn!("local transcription task failed: {error}");
                }
            }
            *lock(&self.local_abort) = None;
        } else if tokio::time::timeout(
            CLIENT_DRAIN_TIMEOUT,
            join_all(session.client_tasks.iter_mut()),
        )
        .await
        .is_err()
        {
            tracing::warn!("realtime clients did not finish graceful shutdown in time");
            for task in &session.client_tasks {
                task.abort();
            }
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
        let sources: Vec<_> = origins.iter().map(|_| cancel.child_token()).collect();
        ActiveSession {
            local,
            failure: Arc::new(Mutex::new(None)),
            abort: CancellationToken::new(),
            origins: origins.to_vec(),
            pause: watch::Sender::new(false),
            client_tasks: sources.iter().cloned().map(client).collect(),
            sources,
            cancel,
            capture_threads: Vec::new(),
            fixture_tasks: Vec::new(),
            relay_tasks: Vec::new(),
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
        ended.sources[0].cancel();
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
        active.sources[1].cancel();
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
        let source = active.sources[0].clone();
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
