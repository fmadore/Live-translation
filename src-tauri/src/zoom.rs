//! Captions in Zoom: one caption language of the app's captions, posted to a meeting's
//! third-party caption URL so attendees see them in Zoom's own caption bar. See
//! `docs/zoom-captions.md`.
//!
//! Sharing is opt-in and independent of the caption session: the operator starts it with the
//! URL the host copied (Show Captions › Set up manual captioner › Copy the API token), it
//! carries on across Stop and Start, and it ends with Stop sharing or when the app quits.
//!
//! Captions reach it through `ZoomCaptions::offer`, called where every engine's captions leave
//! the core (`Events::caption` for `AppHandle`), so no provider, the demonstration or Whisper
//! knows about Zoom. The tap must never hold a caption up: it decides what is newly settled
//! (`publish`), cuts it into Zoom-sized captions and hands them to a bounded queue without
//! waiting. A sender task (`sender`) owns the other end, the HTTP client and Zoom's `seq`
//! counter. While nothing is shared the tap returns at once and nothing is kept.
//!
//! The URL is the meeting's caption credential (`link`). It lives in the share's sender and
//! nowhere else in the core: it is never logged, and no error or status carries it.

mod link;
mod publish;
mod sender;
#[cfg(test)]
mod test_support;

use std::sync::{Arc, Mutex};
use std::time::Instant;

use serde::Serialize;
use tauri::{AppHandle, Emitter, State};
use tokio::sync::{mpsc, Mutex as AsyncMutex};
use tokio_util::sync::CancellationToken;

use crate::errors::{id, AppError};
use crate::sync::lock;
use crate::types::{events, Caption, Origin, Provider};
use link::Link;
use publish::{chunk, Publisher, Selection};
use sender::{Queued, Reply, Sender, Timing};

/// Captions waiting for the sender. Past this the tap drops rather than waits: a Zoom that
/// cannot keep up must cost the meeting captions, never the room its overlay.
const QUEUE: usize = 64;

/// The longest test caption, in characters.
const TEST_CHARS: usize = 500;

/// The step-by-step setup guide the dialog links to: Zoom's settings are several menus deep,
/// and the host's part happens outside this app. Also in `src/lib/zoomCaptions.svelte.ts`,
/// which shows it; a test below keeps the two in step.
pub const GUIDE_URL: &str =
    "https://github.com/fmadore/Live-translation/blob/main/docs/zoom-captions.md";

/// What the operator is told about the share. Each state but `Off` follows Zoom's latest
/// answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ZoomState {
    /// Nothing is being shared.
    Off,
    /// Zoom accepted the start check or the last caption.
    Sharing,
    /// Zoom answered 400: the meeting has not started. Captions flow once it has.
    Waiting,
    /// Zoom answered 401, 403 or 404: the link has expired, been revoked or was copied
    /// incompletely.
    Refused,
    /// No connection, a timeout, 429 or a server error, after the retries.
    Unreachable,
}

/// The payload of `events::ZOOM_CAPTIONS`, and what `zoom_captions_status` answers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ZoomStatus {
    pub state: ZoomState,
    pub lane: u8,
    pub origins: Vec<Origin>,
    /// Captions Zoom accepted since this share started.
    pub sent: u64,
    /// Captions given up on: queue full, too old to be worth showing, or undeliverable.
    pub dropped: u64,
    /// The HTTP status of the last failure, when Zoom answered one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub http_status: Option<u16>,
}

impl ZoomStatus {
    pub fn off() -> Self {
        Self {
            state: ZoomState::Off,
            lane: 0,
            origins: Vec::new(),
            sent: 0,
            dropped: 0,
            http_status: None,
        }
    }
}

/// Where a share's status goes: the `zoom-captions` event in the app, a list in the tests.
type Report = Box<dyn Fn(&ZoomStatus) + Send + Sync>;

/// A share's status, and the one way it changes and is reported.
///
/// Every change is reported under the lock, after checking that the share has not ended. So
/// reports leave in the order the changes were made, and none can follow the `off` Stop
/// reports, or the first status of a share that replaced this one.
struct Ledger {
    status: Mutex<ZoomStatus>,
    /// Cancelled when the share ends. The sender watches it through every request and wait.
    cancel: CancellationToken,
    report: Report,
}

impl Ledger {
    fn new(status: ZoomStatus, report: Report) -> Self {
        Self {
            status: Mutex::new(status),
            cancel: CancellationToken::new(),
            report,
        }
    }

    /// Change the status and report it, unless the share has ended.
    fn record(&self, change: impl FnOnce(&mut ZoomStatus)) {
        let mut status = lock(&self.status);
        if self.cancel.is_cancelled() {
            return;
        }
        change(&mut status);
        (self.report)(&status);
    }

    fn snapshot(&self) -> ZoomStatus {
        lock(&self.status).clone()
    }

    /// End the share. Under the lock, so a report already under way finishes first and none
    /// starts after.
    fn close(&self) {
        let _status = lock(&self.status);
        self.cancel.cancel();
    }
}

/// One meeting's captions, from `start_zoom_captions` until it is stopped or replaced.
struct Share {
    selection: Selection,
    queue: mpsc::Sender<Queued>,
    ledger: Arc<Ledger>,
}

impl Share {
    /// Cut `text` into captions and queue them without waiting. What finds the queue full is
    /// dropped; the ledger to count it in comes back with the number, to be recorded once the
    /// caller has let go of `ZoomCaptions::inner`.
    fn queue(&self, text: &str) -> Option<(Arc<Ledger>, u64)> {
        let at = Instant::now();
        let mut dropped = 0;
        for text in chunk(text) {
            if self.queue.try_send(Queued { text, at }).is_err() {
                dropped += 1;
            }
        }
        (dropped > 0).then(|| (Arc::clone(&self.ledger), dropped))
    }
}

fn record_dropped(full: Option<(Arc<Ledger>, u64)>) {
    if let Some((ledger, dropped)) = full {
        tracing::debug!(dropped, "the Zoom caption queue is full; dropping captions");
        ledger.record(|status| status.dropped += dropped);
    }
}

/// Managed state: the share, if there is one, and the caption session's progress through it.
#[derive(Default)]
pub struct ZoomCaptions {
    /// Serializes start and stop, so a stop cannot land between a start's check of the link
    /// and the share it then installs.
    lifecycle: AsyncMutex<()>,
    inner: Mutex<Inner>,
}

#[derive(Default)]
struct Inner {
    publisher: Publisher,
    share: Option<Share>,
}

impl ZoomCaptions {
    /// A caption session is starting with `provider`. Turn ids start again with it, so the
    /// progress kept for the last session's turns is forgotten; and whether interim captions
    /// may be shared before their turn ends depends on how this engine builds them.
    pub fn begin_session(&self, provider: Provider) {
        lock(&self.inner).publisher = Publisher::new(interims_only_grow(provider));
    }

    /// A caption on its way to the windows. Queues whatever it settles for Zoom, if a share
    /// wants it, and returns: no I/O, no waiting, one short lock.
    pub fn offer(&self, caption: &Caption<'_>) {
        let mut inner = lock(&self.inner);
        let Inner { publisher, share } = &mut *inner;
        let Some(share) = share
            .as_ref()
            .filter(|share| share.selection.wants(caption))
        else {
            return;
        };
        let Some(text) = publisher.take(caption) else {
            return;
        };
        let full = share.queue(text);
        drop(inner);
        record_dropped(full);
    }

    /// Check `link` with Zoom and share captions with it, in place of any share already
    /// running. A link Zoom refuses fails the start and leaves the current share as it was;
    /// one Zoom cannot be asked about yet (the meeting has not started, the network is down)
    /// starts anyway, in the state that says so, and recovers with the next caption.
    async fn start(
        &self,
        link: Link,
        selection: Selection,
        timing: Timing,
        report: Report,
    ) -> Result<ZoomStatus, AppError> {
        let _lifecycle = self.lifecycle.lock().await;
        let client = sender::client(&timing)
            .map_err(|error| AppError::with(id::TASK_FAILED, sender::describe(error)))?;
        // Zoom's count of the captions it has accepted: it shows nothing to attendees, and
        // tells a link that works from one that does not before the operator relies on it.
        let reply =
            sender::exchange(client.get(link.last_seq()), timing.request, link.host()).await;
        let opening = Opening::of(&reply)?;

        let status = ZoomStatus {
            state: opening.state,
            lane: selection.lane,
            origins: selection.origins.clone(),
            sent: 0,
            dropped: 0,
            http_status: opening.http_status,
        };
        tracing::info!(
            host = link.host(),
            lane = selection.lane,
            origins = ?selection.origins,
            state = ?status.state,
            "sharing captions with Zoom"
        );
        let ledger = Arc::new(Ledger::new(status.clone(), report));
        let (queue, receiver) = mpsc::channel(QUEUE);
        let sender = Sender::new(client, link, timing, Arc::clone(&ledger), opening.next);
        let previous = lock(&self.inner).share.replace(Share {
            selection,
            queue,
            ledger: Arc::clone(&ledger),
        });
        if let Some(previous) = previous {
            previous.ledger.close();
        }
        tauri::async_runtime::spawn(sender.run(receiver));
        ledger.record(|_| {});
        Ok(status)
    }

    /// End the share, discarding what is still queued, and report `off` through `report`.
    /// Reports `off` with nothing to stop too, so a window that lost track hears it.
    async fn stop(&self, report: impl FnOnce(&ZoomStatus)) {
        let _lifecycle = self.lifecycle.lock().await;
        let share = lock(&self.inner).share.take();
        if let Some(share) = share {
            share.ledger.close();
            tracing::info!("stopped sharing captions with Zoom");
        }
        report(&ZoomStatus::off());
    }

    /// Queue `text` as it is, whatever the share's language and sources, so the operator can
    /// see that captions reach the meeting. Blank text sends nothing.
    fn send_test(&self, text: &str) -> Result<(), AppError> {
        let inner = lock(&self.inner);
        let share = inner.share.as_ref().ok_or_else(|| {
            AppError::with(id::ZOOM_NOT_SHARING, "no caption link is being shared")
        })?;
        let text: String = text.trim().chars().take(TEST_CHARS).collect();
        if text.is_empty() {
            return Ok(());
        }
        let full = share.queue(&text);
        drop(inner);
        record_dropped(full);
        Ok(())
    }

    fn status(&self) -> ZoomStatus {
        lock(&self.inner)
            .share
            .as_ref()
            .map_or_else(ZoomStatus::off, |share| share.ledger.snapshot())
    }
}

/// Whether `provider` builds an interim caption only by appending to it, so its start never
/// changes and can be shared before the turn ends. Listed rather than `_`, so a new provider
/// has to say.
fn interims_only_grow(provider: Provider) -> bool {
    match provider {
        // Each appends transcript deltas to the turn.
        Provider::Gemini | Provider::OpenAi | Provider::Mistral => true,
        // Transcribe Live replaces its hypothesis; the demonstration rewrites its partial
        // line; Whisper sends finals only.
        Provider::GeminiTranscribe
        | Provider::OnDevice
        | Provider::Whisper
        | Provider::WhisperTranslate => false,
    }
}

/// Where a share starts, from Zoom's answer to the start probe.
#[derive(Debug, PartialEq, Eq)]
struct Opening {
    state: ZoomState,
    http_status: Option<u16>,
    /// The first caption's `seq`, when Zoom said what it last accepted.
    next: Option<u64>,
}

impl Opening {
    /// A refusal is the only answer that fails the start: nothing the meeting does later can
    /// make that link work.
    fn of(reply: &Reply) -> Result<Self, AppError> {
        let next = match reply {
            Reply::Accepted(body) => sender::last_seq(body).map(|last| last.saturating_add(1)),
            Reply::Status(status @ (401 | 403 | 404)) => {
                return Err(AppError::with(id::ZOOM_REFUSED, format!("HTTP {status}")));
            }
            Reply::Status(_) | Reply::Failed => None,
        };
        Ok(Self {
            state: reply.state(),
            http_status: reply.http_status(),
            next,
        })
    }
}

/// The status of a share as it changes, to the operator window.
fn report_to(app: AppHandle) -> Report {
    Box::new(move |status: &ZoomStatus| {
        // Broadcast, like every other core event; see the comment in `lib.rs`.
        let _ = app.emit(events::ZOOM_CAPTIONS, status);
    })
}

/// Share captions in `lane` from `origins` with the Zoom meeting whose caption URL is `url`,
/// replacing any share already running. Operator-only.
#[tauri::command]
pub async fn start_zoom_captions(
    app: AppHandle,
    zoom: State<'_, ZoomCaptions>,
    url: String,
    lane: u8,
    origins: Vec<Origin>,
) -> Result<ZoomStatus, AppError> {
    // The rule that failed, never the text: the URL is a credential.
    let link = Link::parse(&url).map_err(|rule| AppError::with(id::ZOOM_LINK, rule))?;
    zoom.start(
        link,
        Selection { lane, origins },
        Timing::default(),
        report_to(app),
    )
    .await
}

/// Stop sharing, discarding queued captions. Idempotent.
#[tauri::command]
pub async fn stop_zoom_captions(
    app: AppHandle,
    zoom: State<'_, ZoomCaptions>,
) -> Result<(), AppError> {
    zoom.stop(|off| {
        let _ = app.emit(events::ZOOM_CAPTIONS, off);
    })
    .await;
    Ok(())
}

/// Show `text` in the meeting through the running share, cut and queued like any caption.
#[tauri::command]
pub async fn send_zoom_test_caption(
    zoom: State<'_, ZoomCaptions>,
    text: String,
) -> Result<(), AppError> {
    zoom.send_test(&text)
}

/// The share's status as it stands, for an operator window that has just loaded.
#[tauri::command]
pub async fn zoom_captions_status(zoom: State<'_, ZoomCaptions>) -> Result<ZoomStatus, AppError> {
    Ok(zoom.status())
}

/// Open the setup guide in the default browser. It takes no argument, for the reason
/// `open_support_page` takes none: the interface must not be able to hand the shell a URL.
#[tauri::command]
pub async fn open_zoom_guide() -> Result<(), AppError> {
    tauri::async_runtime::spawn_blocking(|| crate::support::open(GUIDE_URL))
        .await
        .map_err(|e| AppError::with(id::TASK_FAILED, e))?
}

#[cfg(test)]
mod tests {
    use super::test_support::{quick, recorder, Answer, Server};
    use super::*;
    use std::time::Duration;

    fn caption(origin: Origin, lane: u8, turn_id: u64, text: &str, final_: bool) -> Caption<'_> {
        Caption {
            turn_id,
            text,
            source_text: "",
            final_,
            origin,
            lane,
            start_ms: 0,
            end_ms: 0,
        }
    }

    fn selection(lane: u8, origins: &[Origin]) -> Selection {
        Selection {
            lane,
            origins: origins.to_vec(),
        }
    }

    /// Zoom with a meeting under way that accepts every caption.
    fn meeting() -> Server {
        Server::start(|request| match request.path.as_str() {
            "/closedcaption/seq" => Answer::ok("0"),
            _ => Answer::ok(""),
        })
    }

    async fn wait_for_posts(server: &Server, count: usize) -> Vec<String> {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            let posts = server.posts();
            if posts.len() >= count {
                return posts
                    .into_iter()
                    .map(|post| String::from_utf8(post.body).unwrap())
                    .collect();
            }
            assert!(Instant::now() < deadline, "{posts:?}");
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    }

    #[tokio::test]
    async fn shares_only_the_chosen_language_and_sources_until_stopped() {
        let server = meeting();
        let zoom = ZoomCaptions::default();
        zoom.begin_session(Provider::Gemini);
        let (reports, report) = recorder();
        let status = zoom
            .start(
                server.link(),
                selection(1, &[Origin::System]),
                quick(),
                report,
            )
            .await
            .unwrap();
        assert_eq!(status.state, ZoomState::Sharing);
        assert_eq!(
            (status.lane, status.origins.clone()),
            (1, vec![Origin::System])
        );
        assert_eq!(reports.all(), [status], "the start is reported too");

        zoom.offer(&caption(Origin::Microphone, 1, 1, "Not this source.", true));
        zoom.offer(&caption(Origin::System, 0, 1, "Not this language.", true));
        let first = "This one is long enough to share.";
        zoom.offer(&caption(
            Origin::System,
            1,
            1,
            &format!("{first} And"),
            false,
        ));
        let whole = format!("{first} And the rest.");
        zoom.offer(&caption(Origin::System, 1, 1, &whole, true));

        assert_eq!(wait_for_posts(&server, 2).await, [first, "And the rest."]);
        let sent = reports.until(|status| status.sent == 2).await;
        assert_eq!(zoom.status(), sent);
        assert_eq!(
            server
                .posts()
                .iter()
                .map(|post| post.seq())
                .collect::<Vec<_>>(),
            [Some(1), Some(2)]
        );

        let mut off = Vec::new();
        zoom.stop(|status| off.push(status.clone())).await;
        assert_eq!(off, [ZoomStatus::off()]);
        assert_eq!(zoom.status(), ZoomStatus::off());
        let reported = reports.all().len();
        zoom.offer(&caption(Origin::System, 1, 2, "After the end.", true));
        assert_eq!(
            zoom.send_test("Anyone there?"),
            Err(AppError::with(
                id::ZOOM_NOT_SHARING,
                "no caption link is being shared"
            ))
        );
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert_eq!(server.posts().len(), 2);
        assert_eq!(reports.all().len(), reported, "nothing reported after Stop");
    }

    #[tokio::test]
    async fn a_refused_link_does_not_start_and_leaves_the_current_share_alone() {
        let current = meeting();
        let zoom = ZoomCaptions::default();
        let (_reports, report) = recorder();
        let status = zoom
            .start(
                current.link(),
                selection(0, &[Origin::Microphone]),
                quick(),
                report,
            )
            .await
            .unwrap();

        for refusal in [401, 403, 404] {
            let expired = Server::start(move |_| Answer::status(refusal));
            let (reports, report) = recorder();
            let error = zoom
                .start(
                    expired.link(),
                    selection(1, &[Origin::System]),
                    quick(),
                    report,
                )
                .await
                .unwrap_err();
            assert_eq!(
                error,
                AppError::with(id::ZOOM_REFUSED, format!("HTTP {refusal}"))
            );
            assert!(reports.all().is_empty());
            assert_eq!(zoom.status(), status);
        }
        zoom.stop(|_| {}).await;
    }

    #[tokio::test]
    async fn a_meeting_not_started_or_out_of_reach_still_starts() {
        let zoom = ZoomCaptions::default();
        let early = Server::start(|_| Answer::status(400));
        let (_reports, report) = recorder();
        let status = zoom
            .start(
                early.link(),
                selection(0, &[Origin::Microphone]),
                quick(),
                report,
            )
            .await
            .unwrap();
        assert_eq!(
            (status.state, status.http_status),
            (ZoomState::Waiting, Some(400))
        );

        let busy = Server::start(|_| Answer::status(503));
        let (_reports, report) = recorder();
        let status = zoom
            .start(
                busy.link(),
                selection(0, &[Origin::Microphone]),
                quick(),
                report,
            )
            .await
            .unwrap();
        assert_eq!(
            (status.state, status.http_status),
            (ZoomState::Unreachable, Some(503))
        );

        let timing = Timing {
            request: Duration::from_millis(200),
            ..quick()
        };
        let (_reports, report) = recorder();
        let status = zoom
            .start(
                Server::closed().link(),
                selection(0, &[Origin::Microphone]),
                timing,
                report,
            )
            .await
            .unwrap();
        assert_eq!(
            (status.state, status.http_status),
            (ZoomState::Unreachable, None)
        );
        zoom.stop(|_| {}).await;
    }

    /// The probe already read Zoom's count, so the first caption is numbered from it without
    /// asking again.
    #[tokio::test]
    async fn the_first_caption_follows_the_count_the_start_read() {
        let server = Server::start(|request| match request.path.as_str() {
            "/closedcaption/seq" => Answer::ok("17"),
            _ => Answer::ok(""),
        });
        let zoom = ZoomCaptions::default();
        let (reports, report) = recorder();
        zoom.start(
            server.link(),
            selection(0, &[Origin::Microphone]),
            quick(),
            report,
        )
        .await
        .unwrap();
        zoom.send_test("  Can everyone read this?\n").unwrap();
        // Blank: nothing to send, and no error.
        zoom.send_test(" \n ").unwrap();
        assert_eq!(
            wait_for_posts(&server, 1).await,
            ["Can everyone read this?"]
        );
        reports.until(|status| status.sent == 1).await;
        let requests = server.requests();
        assert_eq!(requests.iter().filter(|r| r.method == "GET").count(), 1);
        assert_eq!(server.posts()[0].seq(), Some(18));
        zoom.stop(|_| {}).await;
    }

    #[tokio::test]
    async fn a_test_caption_is_cut_to_five_hundred_characters_and_into_captions() {
        let server = meeting();
        let zoom = ZoomCaptions::default();
        let (_reports, report) = recorder();
        zoom.start(
            server.link(),
            selection(1, &[Origin::System]),
            quick(),
            report,
        )
        .await
        .unwrap();
        zoom.send_test(&"word ".repeat(200)).unwrap();
        let posts = wait_for_posts(&server, 5).await;
        assert!(posts.iter().all(|post| post.chars().count() <= 120));
        assert_eq!(posts.concat().replace(' ', "").len(), 400, "{posts:?}");
        zoom.stop(|_| {}).await;
    }

    #[tokio::test]
    async fn a_full_queue_drops_what_does_not_fit_and_counts_it() {
        let (reports, report) = recorder();
        let ledger = Arc::new(Ledger::new(ZoomStatus::off(), report));
        let (queue, _receiver) = mpsc::channel(2);
        let share = Share {
            selection: selection(0, &[Origin::Microphone]),
            queue,
            ledger,
        };
        // Three captions' worth into room for two.
        let sentence = format!("{}.", "x".repeat(100));
        let full = share.queue(&[sentence.as_str(); 3].join(" "));
        assert_eq!(full.as_ref().map(|(_, dropped)| *dropped), Some(1));
        record_dropped(full);
        assert_eq!(reports.last().map(|status| status.dropped), Some(1));
        assert!(share.queue("Still full.").is_some());
    }

    #[test]
    fn engines_that_append_may_share_interim_text_and_the_others_may_not() {
        for provider in [Provider::Gemini, Provider::OpenAi, Provider::Mistral] {
            assert!(interims_only_grow(provider), "{provider:?}");
        }
        for provider in [
            Provider::GeminiTranscribe,
            Provider::OnDevice,
            Provider::Whisper,
            Provider::WhisperTranslate,
        ] {
            assert!(!interims_only_grow(provider), "{provider:?}");
        }
    }

    /// A new session forgets the last one's turns: its turn 1 is not the old turn 1.
    #[test]
    fn a_new_session_starts_every_stream_afresh() {
        let zoom = ZoomCaptions::default();
        zoom.begin_session(Provider::Gemini);
        let shared = "The first session shares this sentence. And";
        let mut inner = lock(&zoom.inner);
        assert!(inner
            .publisher
            .take(&caption(Origin::Microphone, 0, 1, shared, false))
            .is_some());
        drop(inner);
        zoom.begin_session(Provider::Gemini);
        inner = lock(&zoom.inner);
        assert_eq!(
            inner
                .publisher
                .take(&caption(Origin::Microphone, 0, 1, "Short again.", true)),
            Some("Short again.")
        );
    }

    #[test]
    fn the_guide_is_https_and_the_page_the_front_end_shows() {
        assert!(GUIDE_URL.starts_with("https://"));
        assert!(
            include_str!("../../src/lib/zoomCaptions.svelte.ts")
                .contains(&format!("'{GUIDE_URL}'")),
            "src/lib/zoomCaptions.svelte.ts shows a different guide from the one this opens"
        );
    }

    #[test]
    fn statuses_serialize_as_the_interface_reads_them() {
        assert_eq!(
            serde_json::to_string(&ZoomStatus::off()).unwrap(),
            r#"{"state":"off","lane":0,"origins":[],"sent":0,"dropped":0}"#
        );
        let waiting = ZoomStatus {
            state: ZoomState::Waiting,
            lane: 1,
            origins: vec![Origin::Microphone, Origin::System],
            sent: 3,
            dropped: 1,
            http_status: Some(400),
        };
        assert_eq!(
            serde_json::to_string(&waiting).unwrap(),
            r#"{"state":"waiting","lane":1,"origins":["microphone","system"],"sent":3,"dropped":1,"httpStatus":400}"#
        );
    }

    #[test]
    fn the_start_probe_decides_where_a_share_begins() {
        assert_eq!(
            Opening::of(&Reply::Accepted("41".into())),
            Ok(Opening {
                state: ZoomState::Sharing,
                http_status: None,
                next: Some(42),
            })
        );
        assert_eq!(
            Opening::of(&Reply::Accepted("not a count".into())).map(|o| o.next),
            Ok(None)
        );
        assert_eq!(
            Opening::of(&Reply::Status(400)).map(|o| (o.state, o.http_status)),
            Ok((ZoomState::Waiting, Some(400)))
        );
        assert_eq!(
            Opening::of(&Reply::Status(403)),
            Err(AppError::with(id::ZOOM_REFUSED, "HTTP 403"))
        );
        assert_eq!(
            Opening::of(&Reply::Failed).map(|o| (o.state, o.http_status)),
            Ok((ZoomState::Unreachable, None))
        );
    }
}
