//! The update check against a loopback stand-in for GitHub's API. Nothing here reaches GitHub.

use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use super::*;

const RUNNING: Version = Version::new(1, 14, 0);
const DAY: u64 = CHECK_INTERVAL_MS;
/// Some moment well after the epoch, so "a day earlier" is still a valid time.
const NOW: u64 = 1_800_000_000_000;

/// One request as the server read it.
#[derive(Debug, Clone)]
struct Request {
    method: String,
    path: String,
    user_agent: Option<String>,
    accept: Option<String>,
}

/// How the server answers.
#[derive(Clone)]
struct Answer {
    status: u16,
    body: String,
    delay: Duration,
}

impl Answer {
    fn release(tag: &str) -> Self {
        Self::ok(&format!(
            r#"{{"tag_name":"{tag}","name":"Live Translation {tag}","draft":false,"prerelease":false,"html_url":"https://example.invalid/elsewhere","body":"Notes"}}"#
        ))
    }

    fn ok(body: &str) -> Self {
        Self {
            status: 200,
            body: body.to_owned(),
            delay: Duration::ZERO,
        }
    }

    fn status(status: u16) -> Self {
        Self {
            status,
            body: r#"{"message":"API rate limit exceeded"}"#.to_owned(),
            delay: Duration::ZERO,
        }
    }

    fn after(self, delay: Duration) -> Self {
        Self { delay, ..self }
    }
}

/// A loopback HTTP server that answers every request the same way and keeps what it read.
struct Server {
    base: String,
    requests: Arc<Mutex<Vec<Request>>>,
}

impl Server {
    fn start(answer: Answer) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let requests = Arc::new(Mutex::new(Vec::new()));
        let recorded = Arc::clone(&requests);
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(stream) = stream else { return };
                let answer = answer.clone();
                let recorded = Arc::clone(&recorded);
                std::thread::spawn(move || serve(stream, &answer, &recorded));
            }
        });
        Self { base, requests }
    }

    fn requests(&self) -> Vec<Request> {
        lock(&self.requests).clone()
    }
}

fn serve(mut stream: TcpStream, answer: &Answer, recorded: &Mutex<Vec<Request>>) {
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut line = String::new();
    if reader.read_line(&mut line).unwrap_or(0) == 0 {
        return;
    }
    let mut words = line.split_whitespace();
    let mut request = Request {
        method: words.next().unwrap_or_default().to_owned(),
        path: words.next().unwrap_or_default().to_owned(),
        user_agent: None,
        accept: None,
    };
    loop {
        line.clear();
        if reader.read_line(&mut line).unwrap_or(0) == 0 || line == "\r\n" {
            break;
        }
        if let Some((name, value)) = line.split_once(':') {
            let value = Some(value.trim().to_owned());
            match name.to_ascii_lowercase().as_str() {
                "user-agent" => request.user_agent = value,
                "accept" => request.accept = value,
                _ => {}
            }
        }
    }
    lock(recorded).push(request);
    std::thread::sleep(answer.delay);
    let head = format!(
        "HTTP/1.1 {} Test\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        answer.status,
        answer.body.len()
    );
    let _ = stream
        .write_all(head.as_bytes())
        .and_then(|()| stream.write_all(answer.body.as_bytes()));
}

/// A context on a fresh data folder, pointed at `server`, eligible unless a test says not.
struct Fixture {
    _dir: tempfile::TempDir,
    cx: Context,
    updates: Updates,
}

impl Fixture {
    fn new(server: &Server) -> Self {
        let dir = tempfile::tempdir().unwrap();
        let cx = Context {
            path: dir.path().join(STATE_FILE),
            api_base: server.base.clone(),
            running: RUNNING,
            eligible: true,
            timeout: Duration::from_secs(5),
        };
        Self {
            _dir: dir,
            cx,
            updates: Updates::default(),
        }
    }

    /// Switched on, as after the operator's Turn on.
    fn switched_on(server: &Server) -> Self {
        let fixture = Self::new(server);
        fixture.updates.set_checks(&fixture.cx, true);
        fixture
    }

    async fn check(&self, now: u64) -> UpdateStatus {
        self.updates.check(&self.cx, now, false).await.unwrap()
    }

    fn stored(&self) -> Stored {
        read(&self.cx.path)
    }
}

// --- Versions -------------------------------------------------------------------------------

#[test]
fn versions_parse_exactly() {
    assert_eq!(Version::parse("1.14.0"), Some(Version::new(1, 14, 0)));
    assert_eq!(Version::parse("0.0.0"), Some(Version::new(0, 0, 0)));
    for bad in [
        "",
        "1",
        "1.2",
        "1.2.3.4",
        "01.2.3",
        "1.02.3",
        "1.2.x",
        " 1.2.3",
        "1.2.3 ",
        "1..3",
        "+1.2.3",
        "-1.2.3",
        "1.2.3+build",
        "1234567890.0.0",
        "１.2.3",
    ] {
        assert_eq!(Version::parse(bad), None, "{bad:?}");
    }
}

#[test]
fn tags_are_versions_or_prereleases_or_nothing() {
    assert_eq!(
        Version::parse_tag("v1.15.0"),
        Some(Tag::Release(Version::new(1, 15, 0)))
    );
    assert_eq!(
        Version::parse_tag("1.15.0"),
        Some(Tag::Release(Version::new(1, 15, 0)))
    );
    assert_eq!(Version::parse_tag("v1.15.0-rc.1"), Some(Tag::Prerelease));
    assert_eq!(Version::parse_tag("v1.15.0-beta"), Some(Tag::Prerelease));
    for bad in [
        "v1.15",
        "vv1.15.0",
        "v1.15.0-",
        "v1.15.0+build",
        "latest",
        "v1.15.0-rc/../x",
        "V1.15.0",
    ] {
        assert_eq!(Version::parse_tag(bad), None, "{bad:?}");
    }
}

#[test]
fn versions_compare_as_numbers() {
    assert!(Version::new(1, 10, 0) > Version::new(1, 9, 9));
    assert!(Version::new(2, 0, 0) > Version::new(1, 99, 99));
    assert!(Version::new(1, 14, 1) > Version::new(1, 14, 0));
    assert_eq!(Version::new(1, 14, 0), RUNNING);
}

#[test]
fn the_release_page_is_built_from_the_version_alone() {
    assert_eq!(
        release_page(Version::new(1, 15, 0)),
        "https://github.com/fmadore/Live-translation/releases/tag/v1.15.0"
    );
}

// --- Checks against the loopback server ------------------------------------------------------

#[tokio::test]
async fn a_newer_release_is_offered() {
    let server = Server::start(Answer::release("v1.15.0"));
    let fixture = Fixture::switched_on(&server);

    let status = fixture.check(NOW).await;

    assert_eq!(status.available.as_deref(), Some("1.15.0"));
    assert_eq!(status.last_checked_ms, Some(NOW));
    assert_eq!(
        fixture.updates.release_page(&fixture.cx).as_deref(),
        Some("https://github.com/fmadore/Live-translation/releases/tag/v1.15.0"),
        "the page is built from the version, never the answer's html_url"
    );
    let requests = server.requests();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].method, "GET");
    assert_eq!(
        requests[0].path,
        "/repos/fmadore/Live-translation/releases/latest"
    );
    assert_eq!(
        requests[0].user_agent.as_deref(),
        Some(concat!("LiveTranslation/", env!("CARGO_PKG_VERSION"))),
        "the app's name and version, and nothing that identifies anyone"
    );
    assert_eq!(
        requests[0].accept.as_deref(),
        Some("application/vnd.github+json")
    );
}

#[tokio::test]
async fn the_same_or_an_older_release_offers_nothing() {
    for tag in ["v1.14.0", "v1.13.2", "v0.9.0"] {
        let server = Server::start(Answer::release(tag));
        let fixture = Fixture::switched_on(&server);

        let status = fixture.check(NOW).await;

        assert_eq!(status.available, None, "{tag}");
        assert_eq!(status.last_checked_ms, Some(NOW), "{tag}");
        assert_eq!(fixture.updates.release_page(&fixture.cx), None, "{tag}");
    }
}

#[tokio::test]
async fn a_prerelease_is_ignored() {
    for body in [
        Answer::release("v1.15.0-rc.1").body,
        r#"{"tag_name":"v1.15.0","prerelease":true}"#.to_owned(),
        r#"{"tag_name":"v1.15.0","draft":true}"#.to_owned(),
    ] {
        let server = Server::start(Answer::ok(&body));
        let fixture = Fixture::switched_on(&server);

        let status = fixture.check(NOW).await;

        assert_eq!(status.available, None, "{body}");
        assert_eq!(fixture.stored().latest, None, "{body}");
        assert_eq!(status.last_checked_ms, Some(NOW), "{body}");
    }
}

#[tokio::test]
async fn a_malformed_answer_is_silent_and_changes_nothing_else() {
    for body in [
        "",
        "not json",
        "[]",
        r#"{"name":"v1.15.0"}"#,
        r#"{"tag_name":"v1.15"}"#,
        r#"{"tag_name":"v1.15.0+evil"}"#,
        r#"{"tag_name":"https://example.invalid/v9.9.9"}"#,
        r#"{"tag_name":42}"#,
    ] {
        let server = Server::start(Answer::ok(body));
        let fixture = Fixture::switched_on(&server);
        // A version found earlier stays on offer.
        fixture.updates.modify(&fixture.cx.path, |stored| {
            stored.latest = Some("1.14.5".to_owned());
        });

        let status = fixture.check(NOW).await;

        assert_eq!(status.available.as_deref(), Some("1.14.5"), "{body}");
        assert_eq!(status.last_checked_ms, Some(NOW), "{body}");
        assert_eq!(server.requests().len(), 1, "{body}");
    }
}

#[tokio::test]
async fn an_oversized_answer_is_refused() {
    let padding = "x".repeat(MAX_BODY);
    let body = format!(r#"{{"tag_name":"v9.0.0","body":"{padding}"}}"#);
    let server = Server::start(Answer::ok(&body));
    let fixture = Fixture::switched_on(&server);

    let status = fixture.check(NOW).await;

    assert_eq!(status.available, None);
    assert_eq!(status.last_checked_ms, Some(NOW));
}

#[tokio::test]
async fn a_rate_limit_or_a_refusal_is_silent_and_retried_the_next_day() {
    for status in [403, 429, 404, 500] {
        let server = Server::start(Answer::status(status));
        let fixture = Fixture::switched_on(&server);

        let first = fixture.check(NOW).await;
        assert_eq!(first.available, None, "{status}");
        assert_eq!(first.last_checked_ms, Some(NOW), "{status}");

        // Not again the same day…
        fixture.check(NOW + DAY - 1).await;
        assert_eq!(server.requests().len(), 1, "{status}");
        // …but the next.
        let next = fixture.check(NOW + DAY).await;
        assert_eq!(server.requests().len(), 2, "{status}");
        assert_eq!(next.last_checked_ms, Some(NOW + DAY), "{status}");
    }
}

#[tokio::test]
async fn a_server_that_does_not_answer_times_out_silently() {
    let server = Server::start(Answer::release("v1.15.0").after(Duration::from_secs(3)));
    let mut fixture = Fixture::switched_on(&server);
    fixture.cx.timeout = Duration::from_millis(200);

    let started = std::time::Instant::now();
    let status = fixture.check(NOW).await;

    assert!(started.elapsed() < Duration::from_secs(2));
    assert_eq!(status.available, None);
    assert_eq!(status.last_checked_ms, Some(NOW));
}

#[tokio::test]
async fn nobody_listening_is_silent() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    drop(listener);
    let server = Server {
        base,
        requests: Arc::default(),
    };
    let fixture = Fixture::switched_on(&server);

    let status = fixture.check(NOW).await;

    assert_eq!(status.available, None);
    assert_eq!(status.last_checked_ms, Some(NOW));
}

// --- When a request goes out ----------------------------------------------------------------

#[tokio::test]
async fn at_most_once_a_day() {
    let server = Server::start(Answer::release("v1.15.0"));
    let fixture = Fixture::switched_on(&server);

    fixture.check(NOW).await;
    fixture.check(NOW + 1).await;
    fixture.check(NOW + DAY / 2).await;
    assert_eq!(server.requests().len(), 1);

    fixture.check(NOW + DAY).await;
    assert_eq!(server.requests().len(), 2);
}

#[tokio::test]
async fn a_clock_set_back_does_not_hold_the_check_off() {
    let server = Server::start(Answer::release("v1.15.0"));
    let fixture = Fixture::switched_on(&server);

    fixture.check(NOW).await;
    fixture.check(NOW - DAY * 30).await;

    assert_eq!(server.requests().len(), 2);
}

#[tokio::test]
async fn two_checks_at_once_make_one_request() {
    let server = Server::start(Answer::release("v1.15.0").after(Duration::from_millis(100)));
    let fixture = Fixture::switched_on(&server);

    let (a, b) = tokio::join!(fixture.check(NOW), fixture.check(NOW));

    assert_eq!(server.requests().len(), 1);
    assert_eq!(a, b);
}

#[tokio::test]
async fn nothing_is_asked_until_the_switch_is_on() {
    let server = Server::start(Answer::release("v1.15.0"));
    let fixture = Fixture::new(&server);

    // Unanswered, then answered No thanks.
    let unanswered = fixture.check(NOW).await;
    assert_eq!(unanswered.checks, UpdateChecks::Unanswered);
    fixture.updates.set_checks(&fixture.cx, false);
    let off = fixture.check(NOW).await;

    assert_eq!(off.checks, UpdateChecks::Off);
    assert_eq!(off.last_checked_ms, None);
    assert!(server.requests().is_empty());

    // And a switch turned off again stops it.
    fixture.updates.set_checks(&fixture.cx, true);
    fixture.check(NOW).await;
    fixture.updates.set_checks(&fixture.cx, false);
    fixture.check(NOW + DAY * 2).await;
    assert_eq!(server.requests().len(), 1);
}

#[tokio::test]
async fn a_copy_that_is_not_eligible_never_asks_or_shows_anything() {
    let server = Server::start(Answer::release("v1.15.0"));
    let mut fixture = Fixture::new(&server);
    fixture.cx.eligible = false;

    // Even with a state file that says on and has a version waiting.
    write(
        &fixture.cx.path,
        &Stored {
            checks: UpdateChecks::On,
            launches: 5,
            last_checked_ms: None,
            latest: Some("2.0.0".to_owned()),
            skipped: None,
        },
    );
    let before = fixture.stored();

    assert_eq!(fixture.updates.status(&fixture.cx), UpdateStatus::default());
    assert_eq!(
        fixture.updates.set_checks(&fixture.cx, true),
        UpdateStatus::default()
    );
    assert_eq!(fixture.updates.skip(&fixture.cx), UpdateStatus::default());
    assert_eq!(fixture.check(NOW).await, UpdateStatus::default());
    assert_eq!(fixture.updates.release_page(&fixture.cx), None);

    assert!(server.requests().is_empty());
    assert_eq!(fixture.stored(), before, "the state file is not touched");
    assert!(!UpdateStatus::default().eligible);
}

#[tokio::test]
async fn no_request_while_a_session_runs() {
    let server = Server::start(Answer::release("v1.15.0"));
    let fixture = Fixture::switched_on(&server);

    let status = fixture.updates.check(&fixture.cx, NOW, true).await.unwrap();

    assert!(server.requests().is_empty());
    assert_eq!(status.last_checked_ms, None, "nothing was checked");

    // Once it has stopped, the check is still due.
    fixture.check(NOW + 1).await;
    assert_eq!(server.requests().len(), 1);
}

#[test]
fn this_platform_is_eligible_only_where_a_github_release_exists() {
    if cfg!(not(any(windows, target_os = "macos"))) {
        assert!(!installed_from_github());
    }
}

// --- Not now, the question and the state file -------------------------------------------------

#[tokio::test]
async fn not_now_hides_a_version_until_a_newer_one() {
    let server = Server::start(Answer::release("v1.15.0"));
    let fixture = Fixture::switched_on(&server);
    fixture.check(NOW).await;

    let skipped = fixture.updates.skip(&fixture.cx);
    assert_eq!(skipped.available, None);
    assert_eq!(fixture.updates.release_page(&fixture.cx), None);
    assert_eq!(fixture.stored().skipped.as_deref(), Some("1.15.0"));

    // The same release the next day stays hidden.
    let again = fixture.check(NOW + DAY).await;
    assert_eq!(again.available, None);

    // A newer one is shown.
    fixture.updates.modify(&fixture.cx.path, |stored| {
        stored.latest = Some("1.15.1".to_owned());
    });
    let newer = fixture.updates.status(&fixture.cx);
    assert_eq!(newer.available.as_deref(), Some("1.15.1"));
}

#[test]
fn a_running_version_that_has_caught_up_hides_the_notice() {
    let stored = Stored {
        latest: Some("1.15.0".to_owned()),
        ..Stored::default()
    };
    assert_eq!(
        stored.status(Version::new(1, 14, 0)).available.as_deref(),
        Some("1.15.0")
    );
    assert_eq!(stored.status(Version::new(1, 15, 0)).available, None);
    assert_eq!(stored.status(Version::new(1, 16, 0)).available, None);
}

#[test]
fn the_question_waits_for_the_second_launch_and_ends_with_an_answer() {
    let server = Server {
        base: String::new(),
        requests: Arc::default(),
    };
    let fixture = Fixture::new(&server);

    // First launch: counted, not asked. A reloaded window is not a launch.
    assert!(!fixture.updates.status(&fixture.cx).ask);
    assert!(!fixture.updates.status(&fixture.cx).ask);
    assert_eq!(fixture.stored().launches, 1);

    // Second launch: a new process, the same data folder.
    let second = Updates::default();
    let status = second.status(&fixture.cx);
    assert!(status.ask);
    assert_eq!(status.checks, UpdateChecks::Unanswered);

    // Answered, either way, it is not asked again.
    let answered = second.set_checks(&fixture.cx, false);
    assert!(!answered.ask);
    let third = Updates::default();
    assert!(!third.status(&fixture.cx).ask);
    assert_eq!(third.status(&fixture.cx).checks, UpdateChecks::Off);
}

#[test]
fn a_damaged_state_file_reads_as_a_first_launch() {
    let server = Server {
        base: String::new(),
        requests: Arc::default(),
    };
    let fixture = Fixture::new(&server);
    std::fs::write(&fixture.cx.path, b"{ not json").unwrap();
    assert_eq!(fixture.stored(), Stored::default());

    // A version that does not parse is never offered, nor put into a page.
    std::fs::write(
        &fixture.cx.path,
        br#"{"checks":"on","latest":"9.9.9/../../evil","skipped":"x"}"#,
    )
    .unwrap();
    let status = fixture.updates.status(&fixture.cx);
    assert_eq!(status.checks, UpdateChecks::On);
    assert_eq!(status.available, None);
    assert_eq!(fixture.updates.release_page(&fixture.cx), None);
}

#[test]
fn the_status_is_camel_case_for_the_interface() {
    let status = UpdateStatus {
        eligible: true,
        checks: UpdateChecks::On,
        ask: false,
        last_checked_ms: Some(5),
        available: Some("1.15.0".to_owned()),
    };
    assert_eq!(
        serde_json::to_value(&status).unwrap(),
        serde_json::json!({
            "eligible": true,
            "checks": "on",
            "ask": false,
            "lastCheckedMs": 5,
            "available": "1.15.0",
        })
    );
}
