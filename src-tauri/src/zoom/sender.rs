//! The task that posts queued captions to Zoom, one at a time, in order.
//!
//! It follows Zoom's own contract (support article KB0060368): every new caption carries a
//! `seq` one higher than the last, a retry keeps its number, and a failure Zoom calls
//! transient is retried with randomized binary exponential backoff for about five seconds
//! before the caption is given up. It owns the queue's receiving end, the HTTP client and the
//! counter; it reports through the share's `Ledger`, so it runs in the tests without Tauri.
//!
//! Nothing here may log the caption URL: its query is the meeting's credential, and a
//! `reqwest::Error` prints the URL it failed on unless it is stripped first (`describe`).

use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hasher};
use std::sync::Arc;
use std::time::{Duration, Instant};

use reqwest::header::CONTENT_TYPE;
use reqwest::{Client, RequestBuilder};
use tokio::sync::mpsc;

use super::link::Link;
use super::{Ledger, ZoomState, ZoomStatus};

/// How long the sender waits on Zoom and on its own queue. One struct so the tests can run
/// the whole retry and expiry logic in milliseconds.
#[derive(Debug, Clone, Copy)]
pub(super) struct Timing {
    /// Each request, connection included.
    pub request: Duration,
    /// How long one caption is retried, from its first attempt, before it is given up.
    pub retry_for: Duration,
    /// The first backoff's upper bound; each later one doubles it.
    pub backoff: Duration,
    /// The oldest a queued caption may be when its turn comes. Past it, the meeting has moved
    /// on: after an outage attendees see current captions, not a replay of the backlog.
    pub max_age: Duration,
}

impl Default for Timing {
    fn default() -> Self {
        Self {
            request: Duration::from_secs(3),
            retry_for: Duration::from_secs(5),
            backoff: Duration::from_millis(100),
            max_age: Duration::from_secs(15),
        }
    }
}

/// The client every request of a share goes through.
pub(super) fn client(timing: &Timing) -> reqwest::Result<Client> {
    let client = Client::builder()
        // Plain HTTP exists only for the tests' loopback server.
        .https_only(cfg!(not(test)))
        .connect_timeout(timing.request)
        // Zoom answers a caption itself. A redirect would turn the POST into a GET elsewhere,
        // taking the token along in the URL, and lose the caption on the way.
        .redirect(reqwest::redirect::Policy::none());
    // A developer's own proxy setting must not intercept the tests' loopback server.
    #[cfg(test)]
    let client = client.no_proxy();
    client.build()
}

/// One caption waiting for the sender.
#[derive(Debug)]
pub(super) struct Queued {
    pub text: String,
    /// When it was published, for `Timing::max_age`.
    pub at: Instant,
}

/// What Zoom made of one request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Reply {
    /// A 2xx, with its body.
    Accepted(String),
    /// Any other status.
    Status(u16),
    /// No answer at all: no connection, a timeout, a reset.
    Failed,
}

impl Reply {
    /// Zoom's list of answers worth another try, plus 429 and the network failing.
    fn worth_retrying(&self) -> bool {
        match self {
            Self::Accepted(_) => false,
            Self::Status(status) => {
                matches!(status, 400 | 403 | 405 | 408 | 429 | 500 | 502 | 503 | 504)
            }
            Self::Failed => true,
        }
    }

    /// What the operator is told this answer means.
    pub(super) fn state(&self) -> ZoomState {
        match self {
            Self::Accepted(_) => ZoomState::Sharing,
            // Zoom's documented answer for a meeting that has not started.
            Self::Status(400) => ZoomState::Waiting,
            // The link has expired, been revoked, or was copied incompletely.
            Self::Status(401 | 403 | 404) => ZoomState::Refused,
            Self::Status(_) | Self::Failed => ZoomState::Unreachable,
        }
    }

    pub(super) fn http_status(&self) -> Option<u16> {
        match self {
            Self::Status(status) => Some(*status),
            Self::Accepted(_) | Self::Failed => None,
        }
    }
}

/// Send one request and read its answer whole, within `timeout`. `host` is for the log line;
/// the URL never is.
pub(super) async fn exchange(request: RequestBuilder, timeout: Duration, host: &str) -> Reply {
    let answer = async {
        let response = request.timeout(timeout).send().await?;
        let status = response.status();
        // Read even when it is not wanted, so the connection can be used again.
        let body = response.text().await?;
        Ok::<_, reqwest::Error>((status, body))
    };
    match answer.await {
        Ok((status, body)) if status.is_success() => Reply::Accepted(body),
        Ok((status, _)) => Reply::Status(status.as_u16()),
        Err(error) => {
            tracing::debug!(host, error = %describe(error), "Zoom caption request failed");
            Reply::Failed
        }
    }
}

/// A reqwest error with every cause, as `errors::AppError` keeps them, and without the URL its
/// `Display` would otherwise include.
pub(super) fn describe(error: reqwest::Error) -> String {
    format!("{:#}", anyhow::Error::new(error.without_url()))
}

/// The last `seq` Zoom accepted, from the body of `/closedcaption/seq`. Empty means none yet.
pub(super) fn last_seq(body: &str) -> Option<u64> {
    let body = body.trim();
    if body.is_empty() {
        Some(0)
    } else {
        body.parse().ok()
    }
}

/// A random wait in `[0, limit]`. No crate for it: the standard library's hasher keys are
/// random per process, and each `RandomState::new` moves them on, which is all the spread a
/// backoff needs.
fn jitter(limit: Duration, salt: u32) -> Duration {
    let mut hasher = RandomState::new().build_hasher();
    hasher.write_u32(salt);
    let limit_ms = u64::try_from(limit.as_millis()).unwrap_or(u64::MAX);
    Duration::from_millis(hasher.finish() % limit_ms.saturating_add(1))
}

/// The share was stopped while a request or a wait was in flight. Nothing more may be
/// reported for it.
struct Cancelled;

/// What became of one caption.
enum Delivery {
    Accepted,
    /// Given up, on this last answer.
    GaveUp(Reply),
}

pub(super) struct Sender {
    client: Client,
    link: Link,
    timing: Timing,
    ledger: Arc<Ledger>,
    /// The `seq` the next caption goes out with, once known.
    next: Option<u64>,
    /// The `seq` of the caption given up on last. Zoom never accepted it, so it is the number
    /// to try again when Zoom cannot say what it last accepted.
    last_failed: Option<u64>,
}

impl Sender {
    /// `next` is known when the start probe read it from Zoom already.
    pub(super) fn new(
        client: Client,
        link: Link,
        timing: Timing,
        ledger: Arc<Ledger>,
        next: Option<u64>,
    ) -> Self {
        Self {
            client,
            link,
            timing,
            ledger,
            next,
            last_failed: None,
        }
    }

    /// Post captions until the share stops: its token is cancelled, or its queue closed.
    pub(super) async fn run(mut self, mut queue: mpsc::Receiver<Queued>) {
        let cancel = self.ledger.cancel.clone();
        loop {
            let queued = tokio::select! {
                biased;
                () = cancel.cancelled() => return,
                queued = queue.recv() => match queued {
                    Some(queued) => queued,
                    None => return,
                },
            };
            let (fresh, stale) = self.skip_stale(queued, &mut queue);
            let delivery = match fresh {
                Some(caption) => match self.deliver(&caption.text).await {
                    Ok(delivery) => Some(delivery),
                    Err(Cancelled) => return,
                },
                None => None,
            };
            // One report per caption, stale ones skipped on the way included.
            self.ledger.record(|status| {
                status.dropped += stale;
                match delivery {
                    Some(Delivery::Accepted) => accepted(status),
                    Some(Delivery::GaveUp(reply)) => gave_up(status, &reply),
                    None => {}
                }
            });
        }
    }

    /// The first caption in the queue still young enough to show, from `queued` on, and how
    /// many older ones were skipped to reach it.
    fn skip_stale(
        &self,
        mut queued: Queued,
        queue: &mut mpsc::Receiver<Queued>,
    ) -> (Option<Queued>, u64) {
        let mut stale = 0;
        while queued.at.elapsed() > self.timing.max_age {
            stale += 1;
            match queue.try_recv() {
                Ok(next) => queued = next,
                Err(_) => return (None, stale),
            }
        }
        if stale > 0 {
            tracing::info!(stale, "skipped captions that waited too long for Zoom");
        }
        (Some(queued), stale)
    }

    /// Post one caption, retrying as Zoom asks, under one `seq`.
    async fn deliver(&mut self, text: &str) -> Result<Delivery, Cancelled> {
        let seq = match self.next {
            Some(seq) => seq,
            None => self.resync().await?,
        };
        let first = Instant::now();
        let mut attempt = 0u32;
        loop {
            let request = self
                .client
                .post(self.link.caption(seq))
                .header(CONTENT_TYPE, "text/plain; charset=utf-8")
                .body(text.to_owned());
            let reply = self.exchange(request).await?;
            if let Reply::Accepted(_) = reply {
                self.next = Some(seq.saturating_add(1));
                return Ok(Delivery::Accepted);
            }
            let elapsed = first.elapsed();
            if !reply.worth_retrying() || elapsed >= self.timing.retry_for {
                tracing::info!(
                    host = self.link.host(),
                    seq,
                    status = ?reply.http_status(),
                    attempts = attempt + 1,
                    "Zoom did not take a caption; skipping it"
                );
                // Zoom may have taken it after all, its answer lost on the way back: ask what
                // it last accepted before numbering the next one.
                self.next = None;
                self.last_failed = Some(seq);
                return Ok(Delivery::GaveUp(reply));
            }
            // Binary exponential: [0, 100] ms, then [0, 200], [0, 400]…, never past the budget.
            let limit = self.timing.backoff.saturating_mul(1 << attempt.min(10));
            let wait = jitter(limit, attempt).min(self.timing.retry_for.saturating_sub(elapsed));
            tokio::select! {
                biased;
                () = self.ledger.cancel.cancelled() => return Err(Cancelled),
                () = tokio::time::sleep(wait) => {}
            }
            attempt += 1;
        }
    }

    /// The `seq` for the next caption when it is not known: one past the last Zoom accepted,
    /// or, when Zoom cannot say, the number of the caption it last refused (1 at first).
    async fn resync(&self) -> Result<u64, Cancelled> {
        let request = self.client.get(self.link.last_seq());
        let reply = self.exchange(request).await?;
        let seq = match &reply {
            Reply::Accepted(body) => last_seq(body).map(|last| last.saturating_add(1)),
            Reply::Status(_) | Reply::Failed => None,
        };
        Ok(seq.or(self.last_failed).unwrap_or(1))
    }

    async fn exchange(&self, request: RequestBuilder) -> Result<Reply, Cancelled> {
        tokio::select! {
            biased;
            () = self.ledger.cancel.cancelled() => Err(Cancelled),
            reply = exchange(request, self.timing.request, self.link.host()) => Ok(reply),
        }
    }
}

fn accepted(status: &mut ZoomStatus) {
    status.sent += 1;
    status.state = ZoomState::Sharing;
    status.http_status = None;
}

fn gave_up(status: &mut ZoomStatus, reply: &Reply) {
    status.dropped += 1;
    status.state = reply.state();
    status.http_status = reply.http_status();
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::super::test_support::{quick, recorder, Answer, Recorded, Server};
    use super::*;
    use crate::types::Origin;

    struct Running {
        queue: mpsc::Sender<Queued>,
        ledger: Arc<Ledger>,
        reports: Recorded,
        task: tokio::task::JoinHandle<()>,
    }

    impl Running {
        fn send(&self, text: &str) {
            self.queue
                .try_send(Queued {
                    text: text.to_owned(),
                    at: Instant::now(),
                })
                .unwrap();
        }

        async fn until(&self, ready: impl Fn(&ZoomStatus) -> bool) -> ZoomStatus {
            self.reports.until(ready).await
        }
    }

    fn start(server: &Server, timing: Timing, next: Option<u64>) -> Running {
        let (reports, report) = recorder();
        let ledger = Arc::new(Ledger::new(
            ZoomStatus {
                state: ZoomState::Sharing,
                lane: 0,
                origins: vec![Origin::Microphone],
                sent: 0,
                dropped: 0,
                http_status: None,
            },
            report,
        ));
        let (queue, receiver) = mpsc::channel(8);
        let sender = Sender::new(
            client(&timing).unwrap(),
            server.link(),
            timing,
            Arc::clone(&ledger),
            next,
        );
        Running {
            queue,
            ledger,
            reports,
            task: tokio::spawn(sender.run(receiver)),
        }
    }

    #[tokio::test]
    async fn captions_are_numbered_on_from_the_last_seq_zoom_reports() {
        let server = Server::start(|request| match request.path.as_str() {
            "/closedcaption/seq" => Answer::ok("41\n"),
            _ => Answer::ok(""),
        });
        let running = start(&server, quick(), None);
        // Accents, typographic quotes and an emoji: the body is the UTF-8 text, as is.
        let texts = [
            "Première ligne — « déjà » vue.",
            "Second line 🎉",
            "三行目です。",
        ];
        for text in texts {
            running.send(text);
        }
        let status = running.until(|s| s.sent == 3).await;
        assert_eq!(status.state, ZoomState::Sharing);
        assert_eq!(status.dropped, 0);

        let requests = server.requests();
        assert_eq!(requests[0].method, "GET");
        assert_eq!(requests[0].path, "/closedcaption/seq");
        assert_eq!(requests[0].query, "id=meeting&signature=secret");
        let posts = server.posts();
        assert_eq!(
            posts.iter().map(|p| p.seq()).collect::<Vec<_>>(),
            [Some(42), Some(43), Some(44)]
        );
        for (post, text) in posts.iter().zip(texts) {
            assert_eq!(post.path, "/closedcaption");
            assert_eq!(
                post.content_type.as_deref(),
                Some("text/plain; charset=utf-8")
            );
            assert_eq!(post.body, text.as_bytes());
        }
        // Asked once: after an accepted caption the next number is known.
        assert_eq!(requests.iter().filter(|r| r.method == "GET").count(), 1);
    }

    #[tokio::test]
    async fn a_retried_caption_keeps_its_number() {
        let posts = Arc::new(AtomicUsize::new(0));
        let counted = Arc::clone(&posts);
        let server = Server::start(move |request| {
            if request.method == "GET" {
                return Answer::ok("7");
            }
            match counted.fetch_add(1, Ordering::SeqCst) {
                0 => Answer::status(503),
                _ => Answer::ok(""),
            }
        });
        let running = start(&server, quick(), None);
        running.send("Once, though it took two tries.");
        running.send("And the next one.");
        let status = running.until(|s| s.sent == 2).await;
        assert_eq!((status.dropped, status.http_status), (0, None));
        assert_eq!(
            server.posts().iter().map(|p| p.seq()).collect::<Vec<_>>(),
            [Some(8), Some(8), Some(9)]
        );
    }

    #[tokio::test]
    async fn a_caption_still_failing_after_the_budget_is_given_up_and_the_count_resynced() {
        let server = Server::start(|request| {
            if request.method == "GET" {
                return Answer::ok("7");
            }
            if request.body == b"Lost." {
                Answer::status(503)
            } else {
                Answer::ok("")
            }
        });
        let running = start(&server, quick(), None);
        running.send("Lost.");
        let status = running.until(|s| s.dropped == 1).await;
        assert_eq!(status.state, ZoomState::Unreachable);
        assert_eq!(status.http_status, Some(503));
        let lost = server.posts();
        assert!(lost.len() >= 2, "retried: {}", lost.len());
        assert!(lost.iter().all(|p| p.seq() == Some(8)));

        running.send("Found.");
        let status = running.until(|s| s.sent == 1).await;
        assert_eq!(
            (status.state, status.http_status),
            (ZoomState::Sharing, None)
        );
        // Zoom was asked again, still says 7, so the next caption reuses 8.
        let requests = server.requests();
        assert_eq!(requests.iter().filter(|r| r.method == "GET").count(), 2);
        assert_eq!(server.posts().last().unwrap().seq(), Some(8));
    }

    #[tokio::test]
    async fn without_an_answer_on_the_count_the_refused_number_is_tried_again() {
        let server = Server::start(|request| {
            if request.method == "GET" {
                return Answer::status(500);
            }
            if request.body == b"Lost." {
                Answer::status(413)
            } else {
                Answer::ok("")
            }
        });
        let running = start(&server, quick(), None);
        running.send("Lost.");
        running.send("Found.");
        let status = running.until(|s| s.sent == 1).await;
        assert_eq!(status.dropped, 1);
        // 413 is not worth retrying: one attempt. No count from Zoom: 1, then 1 again.
        assert_eq!(
            server.posts().iter().map(|p| p.seq()).collect::<Vec<_>>(),
            [Some(1), Some(1)]
        );
    }

    #[tokio::test]
    async fn a_meeting_that_has_not_started_is_waiting() {
        let server = Server::start(|_| Answer::status(400));
        let running = start(&server, quick(), Some(1));
        running.send("Too early.");
        let status = running.until(|s| s.dropped == 1).await;
        assert_eq!(status.state, ZoomState::Waiting);
        assert_eq!(status.http_status, Some(400));
    }

    #[tokio::test]
    async fn an_expired_or_revoked_link_is_refused() {
        // 403 is on Zoom's list to retry, so it is, until the budget runs out.
        let server = Server::start(|_| Answer::status(403));
        let running = start(&server, quick(), Some(1));
        running.send("Expired.");
        let status = running.until(|s| s.dropped == 1).await;
        assert_eq!(
            (status.state, status.http_status),
            (ZoomState::Refused, Some(403))
        );
        assert!(server.posts().len() >= 2);

        // 401 is not: one attempt.
        let server = Server::start(|_| Answer::status(401));
        let running = start(&server, quick(), Some(1));
        running.send("Revoked.");
        let status = running.until(|s| s.dropped == 1).await;
        assert_eq!(
            (status.state, status.http_status),
            (ZoomState::Refused, Some(401))
        );
        assert_eq!(server.posts().len(), 1);
    }

    #[tokio::test]
    async fn no_connection_or_no_answer_in_time_is_unreachable() {
        // A short timeout: Windows takes a while to give up on a refused loopback connection.
        let timing = Timing {
            request: Duration::from_millis(200),
            ..quick()
        };
        let closed = Server::closed();
        let running = start(&closed, timing, Some(1));
        running.send("Nobody there.");
        let status = running.until(|s| s.dropped == 1).await;
        assert_eq!(
            (status.state, status.http_status),
            (ZoomState::Unreachable, None)
        );

        let slow = Server::start(|_| Answer::ok("").after(Duration::from_millis(500)));
        let timing = Timing {
            request: Duration::from_millis(100),
            ..quick()
        };
        let running = start(&slow, timing, Some(1));
        running.send("Too slow.");
        let status = running.until(|s| s.dropped == 1).await;
        assert_eq!(
            (status.state, status.http_status),
            (ZoomState::Unreachable, None)
        );
    }

    #[tokio::test]
    async fn captions_that_waited_too_long_are_skipped_not_replayed() {
        let server = Server::start(|_| Answer::ok("0"));
        let timing = Timing {
            max_age: Duration::from_millis(200),
            ..quick()
        };
        let running = start(&server, timing, Some(1));
        let old = Instant::now() - Duration::from_secs(1);
        for text in ["Stale one.", "Stale two."] {
            running
                .queue
                .try_send(Queued {
                    text: text.to_owned(),
                    at: old,
                })
                .unwrap();
        }
        running.send("Current.");
        let status = running.until(|s| s.sent == 1).await;
        assert_eq!(status.dropped, 2);
        let posts = server.posts();
        assert_eq!(posts.len(), 1);
        assert_eq!(posts[0].body, b"Current.");
        // Skipped and sent in one report, not one per caption.
        assert_eq!(running.reports.all().len(), 1);
    }

    #[tokio::test]
    async fn stopping_ends_a_request_in_flight_at_once_and_reports_nothing_more() {
        let server = Server::start(|_| Answer::ok("").after(Duration::from_secs(5)));
        let timing = Timing {
            request: Duration::from_secs(4),
            ..quick()
        };
        let running = start(&server, timing, Some(1));
        running.send("Never answered.");
        server.wait_for(1).await;

        let stopped = Instant::now();
        running.ledger.close();
        tokio::time::timeout(Duration::from_secs(1), running.task)
            .await
            .expect("the sender did not stop")
            .unwrap();
        assert!(stopped.elapsed() < Duration::from_secs(1));
        assert!(running.reports.all().is_empty());
    }

    #[test]
    fn the_count_zoom_reports_is_read_leniently() {
        assert_eq!(last_seq(" 12\r\n"), Some(12));
        assert_eq!(last_seq(""), Some(0));
        assert_eq!(last_seq("not a number"), None);
    }

    #[test]
    fn a_backoff_stays_within_its_bound() {
        for salt in 0..200 {
            assert!(jitter(Duration::from_millis(100), salt) <= Duration::from_millis(100));
        }
        assert_eq!(jitter(Duration::ZERO, 1), Duration::ZERO);
        // Spread out, not one fixed wait.
        let waits: std::collections::HashSet<_> = (0..50)
            .map(|salt| jitter(Duration::from_millis(1000), salt))
            .collect();
        assert!(waits.len() > 10, "{waits:?}");
    }

    #[test]
    fn each_answer_maps_to_what_the_operator_is_told() {
        let state = |reply: Reply| (reply.state(), reply.http_status(), reply.worth_retrying());
        assert_eq!(
            state(Reply::Accepted(String::new())),
            (ZoomState::Sharing, None, false)
        );
        assert_eq!(
            state(Reply::Status(400)),
            (ZoomState::Waiting, Some(400), true)
        );
        assert_eq!(
            state(Reply::Status(403)),
            (ZoomState::Refused, Some(403), true)
        );
        assert_eq!(
            state(Reply::Status(401)),
            (ZoomState::Refused, Some(401), false)
        );
        assert_eq!(
            state(Reply::Status(404)),
            (ZoomState::Refused, Some(404), false)
        );
        assert_eq!(
            state(Reply::Status(429)),
            (ZoomState::Unreachable, Some(429), true)
        );
        assert_eq!(
            state(Reply::Status(502)),
            (ZoomState::Unreachable, Some(502), true)
        );
        assert_eq!(
            state(Reply::Status(413)),
            (ZoomState::Unreachable, Some(413), false)
        );
        assert_eq!(state(Reply::Failed), (ZoomState::Unreachable, None, true));
    }
}
