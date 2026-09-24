//! Session orchestration: one bounded audio pipeline and realtime client per source.

use std::sync::{Mutex, MutexGuard};
use std::thread::JoinHandle;
use std::time::Duration;

use anyhow::{Context, Result};
use futures_util::future::join_all;
use tauri::async_runtime::JoinHandle as AsyncJoinHandle;
use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::sync::Mutex as AsyncMutex;
use tokio_util::sync::CancellationToken;

use crate::audio::applications::SystemCapture;
use crate::audio::capture::{run_microphone, MicrophoneRuntimeError};
use crate::audio::fixture::run_rehearsal;
use crate::audio::loopback::run_system_loopback;
use crate::audio::AudioChunk;
use crate::errors::{id, AppError};
use crate::gemini::{
    GeminiConfig, GeminiTranscribeConfig, DEFAULT_HOST, DEFAULT_TRANSCRIBE_MODEL,
    DEFAULT_TRANSLATE_MODEL,
};
use crate::mistral::{
    MistralConfig, DEFAULT_MISTRAL_HOST, DEFAULT_MISTRAL_MODEL, DEFAULT_TARGET_STREAMING_DELAY_MS,
};
use crate::ondevice::{self, OnDeviceConfig};
use crate::openai::{
    OpenAiConfig, DEFAULT_OPENAI_HOST, DEFAULT_OPENAI_TRANSCRIBE_MODEL,
    DEFAULT_OPENAI_TRANSLATE_MODEL,
};
use crate::realtime::{run_session, RealtimeProtocol};
use crate::secrets;
use crate::timing::SessionClock;
use crate::types::{
    events, AudioLevel, AudioSource, AudioTestUpdate, Origin, OutputMode, Provider, SessionState,
    StartOptions, StatusUpdate, TargetLanguage,
};

/// At most half a second of 100 ms chunks. The realtime consumer coalesces queued chunks
/// to the newest one after a stall, favoring live latency over replaying stale speech.
const AUDIO_CHANNEL_CAPACITY: usize = 5;
const LEVEL_CHANNEL_CAPACITY: usize = 8;
const CLIENT_DRAIN_TIMEOUT: Duration = Duration::from_secs(5);
/// The preflight test throws its audio away, and `capture.rs` accumulates the level meter over
/// the mono signal *before* resampling, so this rate reaches nothing that can observe it. It
/// exists only because the capture path requires a target.
const TEST_SAMPLE_RATE: u32 = 16_000;

#[derive(Default)]
pub struct SessionManager {
    /// Serializes concurrent start/stop commands so an older request cannot tear down a
    /// newly-started session.
    lifecycle: AsyncMutex<()>,
    active: Mutex<Option<ActiveSession>>,
    /// Preflight level test. Shares `lifecycle` with the session, so the two can never be
    /// holding the same capture device at once.
    active_test: Mutex<Option<ActiveTest>>,
}

/// Level-only capture started from the preflight. No client tasks and no fixture tasks by
/// construction — that absence is the entire point of it.
struct ActiveTest {
    cancel: CancellationToken,
    capture_threads: Vec<JoinHandle<()>>,
}

struct ActiveSession {
    cancel: CancellationToken,
    sources: Vec<CancellationToken>,
    capture_threads: Vec<JoinHandle<()>>,
    /// Timer-driven audio producers used by commercial-provider rehearsal playback.
    fixture_tasks: Vec<AsyncJoinHandle<()>>,
    client_tasks: Vec<AsyncJoinHandle<()>>,
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn validate_provider(mode: OutputMode, provider: Provider) -> Result<()> {
    anyhow::ensure!(
        provider.can_translate() == (mode == OutputMode::Translate),
        "The selected provider does not support this output mode"
    );
    Ok(())
}

/// The selected provider's endpoint and model, with any environment overrides applied.
///
/// Only the selected provider's variables are read, so a malformed override for one backend
/// (say `MISTRAL_TARGET_STREAMING_DELAY_MS`) cannot stop another from starting.
#[derive(Debug, PartialEq)]
enum ProviderSettings {
    Gemini {
        host: String,
        model: String,
    },
    GeminiTranscribe {
        host: String,
        model: String,
    },
    OpenAi {
        host: String,
        model: String,
        transcribe_model: String,
    },
    Mistral {
        host: String,
        model: String,
        delay_ms: u32,
    },
    OnDevice,
}

impl ProviderSettings {
    fn resolve(provider: Provider) -> Result<Self> {
        Self::resolve_with(provider, |name| std::env::var(name).ok())
    }

    fn resolve_with(provider: Provider, var: impl Fn(&str) -> Option<String>) -> Result<Self> {
        let value = |name: &str, default: &str| var(name).unwrap_or_else(|| default.to_string());
        // Pointing a client at another server is a development tool. In a release build a
        // stray variable, or a `.env` in some parent directory, would otherwise be enough to
        // send the operator's API key somewhere else.
        let host = |name: &str, default: &str| {
            if cfg!(debug_assertions) {
                value(name, default)
            } else {
                default.to_string()
            }
        };
        Ok(match provider {
            Provider::Gemini => Self::Gemini {
                host: host("GEMINI_WS_HOST", DEFAULT_HOST),
                model: value("GEMINI_TRANSLATE_MODEL", DEFAULT_TRANSLATE_MODEL),
            },
            Provider::GeminiTranscribe => Self::GeminiTranscribe {
                host: host("GEMINI_WS_HOST", DEFAULT_HOST),
                model: value("GEMINI_TRANSCRIBE_MODEL", DEFAULT_TRANSCRIBE_MODEL),
            },
            Provider::OpenAi => Self::OpenAi {
                host: host("OPENAI_WS_HOST", DEFAULT_OPENAI_HOST),
                model: value("OPENAI_TRANSLATE_MODEL", DEFAULT_OPENAI_TRANSLATE_MODEL),
                transcribe_model: value("OPENAI_TRANSCRIBE_MODEL", DEFAULT_OPENAI_TRANSCRIBE_MODEL),
            },
            Provider::Mistral => Self::Mistral {
                host: host("MISTRAL_WS_HOST", DEFAULT_MISTRAL_HOST),
                model: value("MISTRAL_TRANSCRIBE_MODEL", DEFAULT_MISTRAL_MODEL),
                delay_ms: var("MISTRAL_TARGET_STREAMING_DELAY_MS")
                    .map(|v| v.parse::<u32>())
                    .transpose()
                    .context("MISTRAL_TARGET_STREAMING_DELAY_MS must be an integer")?
                    .unwrap_or(DEFAULT_TARGET_STREAMING_DELAY_MS),
            },
            Provider::OnDevice => Self::OnDevice,
        })
    }
}

/// Which failure this is, for a capture failure on one source.
///
/// The microphone gets its own id on purpose. Under package identity Windows gates the
/// microphone per app, so a blocked install fails at device open with an ordinary cpal error
/// and nothing pointing at the toggle — see gate 6 in `docs/microsoft-store.md`. The
/// interface's wording for `MIC_CAPTURE` therefore names the privacy setting; the id covers a
/// denied device and an absent one alike, because cpal reports both the same way. The
/// underlying error rides along as the detail, so the real cause is still visible.
fn source_failure(origin: Origin, error: &anyhow::Error) -> AppError {
    let detail = format!("{error:#}");
    match origin {
        Origin::Microphone if error.downcast_ref::<MicrophoneRuntimeError>().is_some() => {
            AppError::with(id::MIC_STREAM, detail)
        }
        Origin::Microphone => AppError::with(id::MIC_CAPTURE, detail),
        Origin::System => AppError::with(id::SYSTEM_CAPTURE, detail),
    }
}

/// Report a failure on the audio side of one source: log it, tell the operator which source
/// died, and stop that source's client. The other source, if any, stays live.
fn report_source_failure(
    app: &AppHandle,
    origin: Origin,
    error: &anyhow::Error,
    cancel: &CancellationToken,
) {
    tracing::error!(?origin, "capture failed: {error:#}");
    cancel.cancel();
    let message = source_failure(origin, error);
    let _ = app.emit(
        events::STATUS,
        StatusUpdate {
            state: SessionState::Error,
            message: Some(message),
            origin: Some(origin),
        },
    );
}

fn complete_probe(
    result: Result<()>,
    origin: Origin,
    cancel: &CancellationToken,
    publish: impl FnOnce(AudioTestUpdate),
) {
    if let Err(error) = result {
        tracing::error!(?origin, "audio test failed: {error:#}");
        cancel.cancel();
        publish(AudioTestUpdate {
            active: false,
            message: Some(source_failure(origin, &error)),
        });
    }
}

fn validate_start(options: &StartOptions) -> Result<()> {
    validate_provider(options.mode, options.provider)?;
    anyhow::ensure!(
        options.target_language.supported_by(options.provider),
        "{:?} does not support caption language {}",
        options.provider,
        options.target_language.bcp47()
    );
    if options.rehearsal.is_none()
        && options.provider != Provider::OnDevice
        && options.source != AudioSource::Microphone
    {
        crate::audio::applications::validate(&options.system_capture)?;
    }
    if options.provider == Provider::OnDevice && options.source != AudioSource::Microphone {
        anyhow::bail!("The built-in demonstration uses its bundled sample; select Demo audio")
    }
    if options.provider == Provider::OnDevice && options.rehearsal.is_some() {
        anyhow::bail!("The built-in demonstration already uses bundled content")
    }
    Ok(())
}

/// The capture devices a source selection opens, in a stable order.
fn live_origins(source: AudioSource) -> Vec<Origin> {
    let mut origins = Vec::new();
    if source.wants_mic() {
        origins.push(Origin::Microphone);
    }
    if source.wants_system() {
        origins.push(Origin::System);
    }
    origins
}

/// The sources a session runs. A rehearsal runs exactly one origin, System, off the bundled
/// fixture: `source` and the microphone selection are deliberately ignored, because the point
/// of the mode is to exercise the pipeline with no audio hardware involved at all.
fn session_origins(options: &StartOptions) -> Vec<Origin> {
    if options.rehearsal.is_some() {
        vec![Origin::System]
    } else {
        live_origins(options.source)
    }
}

/// Which devices a live source reads from. Sessions and the preflight test both go through
/// this, so the test opens exactly what a session would.
#[derive(Clone)]
struct CaptureTarget {
    mic_name: Option<String>,
    system_id: Option<String>,
    system_capture: SystemCapture,
}

impl CaptureTarget {
    fn for_session(options: &StartOptions) -> Self {
        Self {
            mic_name: options
                .mic_device_id
                .clone()
                .or(options.mic_device_name.clone()),
            system_id: options.system_device_id.clone(),
            system_capture: options.system_capture.clone(),
        }
    }

    /// Capture `origin` on the calling thread until `cancel` fires or the device fails.
    fn run(
        self,
        origin: Origin,
        target_rate: u32,
        level_tx: Sender<AudioLevel>,
        audio_tx: Sender<AudioChunk>,
        cancel: CancellationToken,
    ) -> Result<()> {
        match origin {
            Origin::Microphone => {
                run_microphone(self.mic_name, target_rate, level_tx, audio_tx, cancel)
            }
            Origin::System => run_system_loopback(
                self.system_id,
                self.system_capture,
                target_rate,
                level_tx,
                audio_tx,
                cancel,
            ),
        }
    }
}

/// Forward meter readings to the interface until every producer has dropped its sender.
fn spawn_level_forwarder(app: &AppHandle) -> Sender<AudioLevel> {
    let (level_tx, mut level_rx) = channel::<AudioLevel>(LEVEL_CHANNEL_CAPACITY);
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        while let Some(level) = level_rx.recv().await {
            let _ = app.emit(events::LEVEL, &level);
        }
    });
    level_tx
}

/// Join capture threads without blocking the async runtime. `what` names them in the log.
async fn join_threads(handles: Vec<JoinHandle<()>>, what: &'static str) {
    let joined = tauri::async_runtime::spawn_blocking(move || {
        for handle in handles {
            if handle.join().is_err() {
                tracing::warn!("{what} thread panicked while stopping");
            }
        }
    })
    .await;
    if let Err(error) = joined {
        tracing::warn!("{what} join task failed: {error}");
    }
}

/// The per-source plumbing every client gets, whichever provider it speaks.
struct ClientIo {
    app: AppHandle,
    origin: Origin,
    audio_rx: Receiver<AudioChunk>,
    cancel: CancellationToken,
    clock: SessionClock,
}

impl ClientIo {
    fn spawn_realtime<P: RealtimeProtocol + Send + 'static>(self, proto: P) -> AsyncJoinHandle<()> {
        tauri::async_runtime::spawn(run_session(
            self.app,
            proto,
            self.audio_rx,
            self.cancel,
            self.clock,
        ))
    }
}

impl ProviderSettings {
    fn spawn_client(
        &self,
        io: ClientIo,
        api_key: &str,
        target: TargetLanguage,
    ) -> Result<AsyncJoinHandle<()>> {
        let origin = io.origin;
        let api_key = api_key.to_string();
        let target_language_code = target.bcp47().to_string();
        Ok(match self {
            Self::Gemini { host, model } => io.spawn_realtime(GeminiConfig {
                api_key,
                model: model.clone(),
                host: host.clone(),
                target_language_code,
                origin,
            }),
            Self::GeminiTranscribe { host, model } => io.spawn_realtime(GeminiTranscribeConfig {
                api_key,
                model: model.clone(),
                host: host.clone(),
                origin,
            }),
            Self::OpenAi {
                host,
                model,
                transcribe_model,
            } => io.spawn_realtime(OpenAiConfig {
                api_key,
                model: model.clone(),
                transcribe_model: transcribe_model.clone(),
                host: host.clone(),
                target_language_code,
                origin,
            }),
            Self::Mistral {
                host,
                model,
                delay_ms,
            } => io.spawn_realtime(MistralConfig {
                api_key,
                model: model.clone(),
                host: host.clone(),
                target_streaming_delay_ms: *delay_ms,
                origin,
                received_delta: false,
            }),
            Self::OnDevice => {
                let config = OnDeviceConfig {
                    origin,
                    language: target.try_into()?,
                };
                tauri::async_runtime::spawn(ondevice::run_session(
                    io.app,
                    config,
                    io.audio_rx,
                    io.cancel,
                    io.clock,
                ))
            }
        })
    }
}

/// Collects one session's sources while `start` spawns them.
struct SessionBuilder<'a> {
    app: &'a AppHandle,
    options: &'a StartOptions,
    settings: ProviderSettings,
    api_key: String,
    capture: CaptureTarget,
    target_rate: u32,
    clock: SessionClock,
    level_tx: Sender<AudioLevel>,
    session: ActiveSession,
}

impl SessionBuilder<'_> {
    fn add_source(&mut self, origin: Origin) -> Result<()> {
        let (audio_tx, audio_rx) = channel::<AudioChunk>(AUDIO_CHANNEL_CAPACITY);
        let cancel = self.session.cancel.child_token();
        self.session.sources.push(cancel.clone());
        self.spawn_producer(origin, audio_tx, &cancel)?;

        let io = ClientIo {
            app: self.app.clone(),
            origin,
            audio_rx,
            cancel,
            clock: self.clock,
        };
        let client = self
            .settings
            .spawn_client(io, &self.api_key, self.options.target_language)?;
        self.session.client_tasks.push(client);
        Ok(())
    }

    /// Start whatever feeds one source's audio channel.
    fn spawn_producer(
        &mut self,
        origin: Origin,
        audio_tx: Sender<AudioChunk>,
        cancel: &CancellationToken,
    ) -> Result<()> {
        if self.options.provider == Provider::OnDevice {
            // The deterministic demo emits its own level/caption timeline and never opens a
            // capture device. Close the unused producer immediately.
            drop(audio_tx);
            return Ok(());
        }

        let app = self.app.clone();
        let level_tx = self.level_tx.clone();
        let target_rate = self.target_rate;
        let cancel = cancel.clone();
        match self.options.rehearsal {
            // Rehearsal swaps the capture device for a bundled recording and changes nothing
            // else: same channel, same chunk shape, same engine below it.
            Some(language) => {
                self.session
                    .fixture_tasks
                    .push(tauri::async_runtime::spawn(async move {
                        let result = run_rehearsal(
                            &app,
                            language,
                            target_rate,
                            level_tx,
                            audio_tx,
                            cancel.clone(),
                        )
                        .await;
                        if let Err(error) = result {
                            report_source_failure(&app, origin, &error, &cancel);
                        }
                    }));
            }
            None => {
                let capture = self.capture.clone();
                let handle = std::thread::Builder::new()
                    .name(format!("capture-{origin:?}"))
                    .spawn(move || {
                        let result =
                            capture.run(origin, target_rate, level_tx, audio_tx, cancel.clone());
                        if let Err(error) = result {
                            report_source_failure(&app, origin, &error, &cancel);
                        }
                    })
                    .context("failed to spawn capture thread")?;
                self.session.capture_threads.push(handle);
            }
        }
        Ok(())
    }
}

impl SessionManager {
    pub async fn start(&self, app: &AppHandle, options: StartOptions) -> Result<()> {
        let _lifecycle = self.lifecycle.lock().await;
        self.stop_active(app).await;
        // A preflight test is holding the very devices this session is about to open.
        self.stop_test_active(app).await;
        validate_start(&options)?;

        let provider = options.provider;
        // The built-in demonstration is the one backend that starts with no credential.
        // Credential Manager is a blocking call, and this holds the lifecycle lock: keep it off
        // the async workers that are pumping the other windows' events meanwhile.
        let api_key = if provider.requires_api_key() {
            tauri::async_runtime::spawn_blocking(move || secrets::resolve_api_key(provider))
                .await
                .context("keychain lookup did not complete")??
        } else {
            String::new()
        };
        let settings = ProviderSettings::resolve(provider)?;

        let cancel = CancellationToken::new();
        let cancel_guard = cancel.clone().drop_guard();
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
                sources: Vec::new(),
                capture_threads: Vec::new(),
                fixture_tasks: Vec::new(),
                client_tasks: Vec::new(),
            },
        };
        for origin in session_origins(&options) {
            builder.add_source(origin)?;
        }

        cancel_guard.disarm();
        *lock(&self.active) = Some(builder.session);
        Ok(())
    }

    /// Level-only capture for the preflight, so an operator can confirm the room microphone or
    /// the loopback is actually producing sound before committing to a session.
    ///
    /// It opens the same devices a session would and then throws every sample away. There is no
    /// provider client, no caption, and nothing written anywhere, so it cannot bill and cannot
    /// leak room audio. Dropping the audio receiver is the whole mechanism: `capture.rs` sees a
    /// closed channel, discards the chunk and carries on, while the level channel keeps flowing
    /// because levels are accumulated before the chunk is ever sent.
    pub async fn start_test(
        &self,
        app: &AppHandle,
        source: AudioSource,
        mic_device_name: Option<String>,
        system_device_id: Option<String>,
        system_capture: SystemCapture,
    ) -> Result<()> {
        let _lifecycle = self.lifecycle.lock().await;
        if lock(&self.active)
            .as_ref()
            .is_some_and(|session| session.sources.iter().any(|source| !source.is_cancelled()))
        {
            anyhow::bail!("A session is already running, so its meters are already live");
        }
        self.stop_test_active(app).await;
        // A failed provider may have left completed handles to join, but no live client.
        self.stop_active(app).await;

        if source != AudioSource::Microphone {
            crate::audio::applications::validate(&system_capture)?;
        }

        let cancel = CancellationToken::new();
        let cancel_guard = cancel.clone().drop_guard();
        let level_tx = spawn_level_forwarder(app);
        let capture = CaptureTarget {
            mic_name: mic_device_name,
            system_id: system_device_id,
            system_capture,
        };
        let mut capture_threads = Vec::new();
        let mut starters = Vec::new();

        for origin in live_origins(source) {
            let (audio_tx, audio_rx) = channel::<AudioChunk>(AUDIO_CHANNEL_CAPACITY);
            // The receiver is dropped immediately and deliberately: that is what makes this
            // level-only rather than a silent session.
            drop(audio_rx);

            let probe_app = app.clone();
            // A preflight is one test: failure on either source stops both devices.
            let probe_cancel = cancel.clone();
            let level_tx = level_tx.clone();
            let capture = capture.clone();
            let (start_tx, start_rx) = std::sync::mpsc::channel();
            let handle = std::thread::Builder::new()
                .name(format!("audio-test-{origin:?}"))
                .spawn(move || {
                    // Publish active before a fast device-open error can publish inactive.
                    // A failed thread spawn drops the senders and releases earlier workers.
                    if start_rx.recv().is_err() || probe_cancel.is_cancelled() {
                        return;
                    }
                    let result = capture.run(
                        origin,
                        TEST_SAMPLE_RATE,
                        level_tx,
                        audio_tx,
                        probe_cancel.clone(),
                    );
                    complete_probe(result, origin, &probe_cancel, |update| {
                        let _ = probe_app.emit(events::AUDIO_TEST, update);
                    });
                })
                .context("failed to spawn audio test thread")?;
            capture_threads.push(handle);
            starters.push(start_tx);
        }

        let cancel = cancel_guard.disarm();
        *lock(&self.active_test) = Some(ActiveTest {
            cancel,
            capture_threads,
        });
        let _ = app.emit(
            events::AUDIO_TEST,
            AudioTestUpdate {
                active: true,
                message: None,
            },
        );
        for starter in starters {
            let _ = starter.send(());
        }
        tracing::info!(?source, "audio test started");
        Ok(())
    }

    pub async fn stop_test(&self, app: &AppHandle) {
        let _lifecycle = self.lifecycle.lock().await;
        self.stop_test_active(app).await;
    }

    /// Releases the capture devices and joins the probe threads. Callers already hold
    /// `lifecycle`.
    async fn stop_test_active(&self, app: &AppHandle) {
        let test = lock(&self.active_test).take();
        if let Some(test) = test {
            test.cancel.cancel();
            join_threads(test.capture_threads, "audio test").await;
            let _ = app.emit(
                events::AUDIO_TEST,
                AudioTestUpdate {
                    active: false,
                    message: None,
                },
            );
            tracing::info!("audio test stopped");
        }
    }

    pub async fn stop(&self, app: &AppHandle) {
        let _lifecycle = self.lifecycle.lock().await;
        self.stop_active(app).await;
    }

    async fn stop_active(&self, app: &AppHandle) {
        let session = lock(&self.active).take();
        if let Some(mut session) = session {
            session.cancel.cancel();
            join_threads(session.capture_threads, "capture").await;

            // Rehearsal playback holds the producer end of its audio channel, and the client
            // below only sees the stream end once that is dropped — so drain it here, in the
            // same place the capture threads are joined.
            for result in join_all(session.fixture_tasks.iter_mut()).await {
                if let Err(error) = result {
                    tracing::warn!("rehearsal playback task failed: {error}");
                }
            }

            // Providers may emit their last transcript while flushing. Do not report Idle
            // (or start a replacement session) until that bounded drain has completed.
            if tokio::time::timeout(
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
            let _ = app.emit(
                events::STATUS,
                StatusUpdate {
                    state: SessionState::Idle,
                    message: None,
                    origin: None,
                },
            );
            tracing::info!("session stopped");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_preflight_runtime_failure_stops_both_sources_and_reports_test_status() {
        let test = CancellationToken::new();
        let system = test.child_token();
        let error = anyhow::Error::new(MicrophoneRuntimeError(cpal::Error::new(
            cpal::ErrorKind::DeviceNotAvailable,
        )));
        let mut updates = Vec::new();
        complete_probe(Err(error), Origin::Microphone, &test, |update| {
            updates.push(update)
        });
        assert!(test.is_cancelled());
        assert!(system.is_cancelled());
        assert_eq!(updates.len(), 1);
        assert!(!updates[0].active);
        assert_eq!(updates[0].message.as_ref().unwrap().id, id::MIC_STREAM);
    }

    #[test]
    fn device_open_failure_keeps_the_microphone_privacy_guidance() {
        let error = anyhow::Error::new(cpal::Error::new(cpal::ErrorKind::PermissionDenied));
        assert_eq!(
            source_failure(Origin::Microphone, &error).id,
            id::MIC_CAPTURE
        );
    }

    #[test]
    fn a_clean_preflight_stop_does_not_publish_a_failure() {
        let test = CancellationToken::new();
        complete_probe(Ok(()), Origin::System, &test, |_| {
            panic!("unexpected failure")
        });
    }

    #[test]
    fn every_provider_is_rejected_in_the_wrong_mode() {
        for (provider, translates) in [
            (Provider::Gemini, true),
            (Provider::OpenAi, true),
            (Provider::GeminiTranscribe, false),
            (Provider::Mistral, false),
            (Provider::OnDevice, false),
        ] {
            assert_eq!(
                validate_provider(OutputMode::Translate, provider).is_ok(),
                translates
            );
            assert_eq!(
                validate_provider(OutputMode::Transcribe, provider).is_ok(),
                !translates
            );
        }
    }

    fn options(source: &str, rehearsal: Option<&str>) -> StartOptions {
        serde_json::from_value(serde_json::json!({
            "source": source,
            "targetLanguage": "fr",
            "provider": "gemini",
            "rehearsal": rehearsal,
        }))
        .unwrap()
    }

    #[test]
    fn a_session_opens_the_selected_sources_in_order() {
        assert_eq!(
            session_origins(&options("both", None)),
            [Origin::Microphone, Origin::System]
        );
        assert_eq!(
            session_origins(&options("microphone", None)),
            [Origin::Microphone]
        );
        assert_eq!(session_origins(&options("system", None)), [Origin::System]);
    }

    #[test]
    fn a_rehearsal_plays_one_system_source_whatever_is_selected() {
        for source in ["microphone", "system", "both"] {
            assert_eq!(
                session_origins(&options(source, Some("en"))),
                [Origin::System]
            );
        }
    }

    #[test]
    fn a_session_uses_the_device_id_before_the_device_name() {
        let mut start = options("microphone", None);
        start.mic_device_name = Some("Room mic".into());
        assert_eq!(
            CaptureTarget::for_session(&start).mic_name.as_deref(),
            Some("Room mic")
        );
        start.mic_device_id = Some("{0.0.1.00000000}".into());
        assert_eq!(
            CaptureTarget::for_session(&start).mic_name.as_deref(),
            Some("{0.0.1.00000000}")
        );
    }

    #[test]
    fn another_providers_malformed_override_does_not_block_a_session() {
        let env =
            |name: &str| (name == "MISTRAL_TARGET_STREAMING_DELAY_MS").then(|| "soon".to_string());
        assert!(ProviderSettings::resolve_with(Provider::Gemini, env).is_ok());
        assert!(ProviderSettings::resolve_with(Provider::Mistral, env).is_err());
    }

    #[test]
    fn model_overrides_apply_and_defaults_fill_the_rest() {
        let env = |name: &str| (name == "OPENAI_TRANSLATE_MODEL").then(|| "pinned".to_string());
        let ProviderSettings::OpenAi {
            model,
            transcribe_model,
            ..
        } = ProviderSettings::resolve_with(Provider::OpenAi, env).unwrap()
        else {
            panic!("wrong provider settings")
        };
        assert_eq!(model, "pinned");
        assert_eq!(transcribe_model, DEFAULT_OPENAI_TRANSCRIBE_MODEL);
    }

    #[test]
    fn host_overrides_are_honoured_only_in_debug_builds() {
        let env = |name: &str| (name == "GEMINI_WS_HOST").then(|| "localhost:9000".to_string());
        let ProviderSettings::Gemini { host, .. } =
            ProviderSettings::resolve_with(Provider::Gemini, env).unwrap()
        else {
            panic!("wrong provider settings")
        };
        let expected = if cfg!(debug_assertions) {
            "localhost:9000"
        } else {
            DEFAULT_HOST
        };
        assert_eq!(host, expected);
    }
}
