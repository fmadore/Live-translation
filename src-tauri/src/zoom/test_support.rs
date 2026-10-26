//! A loopback stand-in for Zoom's caption endpoint, and a recorder for the statuses a share
//! reports, shared by the sender's tests and the share's.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use super::link::Link;
use super::sender::Timing;
use super::{Report, ZoomStatus};
use crate::sync::lock;

/// Production's timing on a scale a test can wait for.
pub(super) fn quick() -> Timing {
    Timing {
        request: Duration::from_secs(2),
        retry_for: Duration::from_millis(300),
        backoff: Duration::from_millis(10),
        max_age: Duration::from_secs(15),
    }
}

/// One request as the server read it.
#[derive(Debug, Clone)]
pub(super) struct Request {
    pub method: String,
    pub path: String,
    pub query: String,
    pub content_type: Option<String>,
    pub body: Vec<u8>,
}

impl Request {
    /// The `seq` this request was numbered with, if any.
    pub(super) fn seq(&self) -> Option<u64> {
        self.query
            .split('&')
            .find_map(|pair| pair.strip_prefix("seq="))
            .and_then(|seq| seq.parse().ok())
    }
}

/// What the server answers one request with.
pub(super) struct Answer {
    status: u16,
    body: String,
    /// How long it waits first: a slow or silent Zoom.
    delay: Duration,
}

impl Answer {
    pub(super) fn ok(body: &str) -> Self {
        Self {
            status: 200,
            body: body.to_owned(),
            delay: Duration::ZERO,
        }
    }

    pub(super) fn status(status: u16) -> Self {
        Self {
            status,
            body: String::new(),
            delay: Duration::ZERO,
        }
    }

    pub(super) fn after(self, delay: Duration) -> Self {
        Self { delay, ..self }
    }
}

type Script = dyn Fn(&Request) -> Answer + Send + Sync;

/// An HTTP server on a loopback port that answers each request as its script says and keeps
/// what it was sent. Plain threads, one per connection, so it keeps answering while the test's
/// runtime is busy with the client, and a slow answer holds up only its own request.
pub(super) struct Server {
    base: String,
    requests: Arc<Mutex<Vec<Request>>>,
}

impl Server {
    pub(super) fn start(script: impl Fn(&Request) -> Answer + Send + Sync + 'static) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let requests = Arc::new(Mutex::new(Vec::new()));
        let script: Arc<Script> = Arc::new(script);
        let recorded = Arc::clone(&requests);
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(stream) = stream else {
                    return;
                };
                let script = Arc::clone(&script);
                let recorded = Arc::clone(&recorded);
                std::thread::spawn(move || answer(stream, &*script, &recorded));
            }
        });
        Self { base, requests }
    }

    /// An address with nobody listening: a Zoom that cannot be reached at all.
    pub(super) fn closed() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        drop(listener);
        Self {
            base,
            requests: Arc::default(),
        }
    }

    /// A caption link to this server, shaped like Zoom's.
    pub(super) fn link(&self) -> Link {
        Link::unchecked(&format!(
            "{}/closedcaption?id=meeting&signature=secret",
            self.base
        ))
    }

    pub(super) fn requests(&self) -> Vec<Request> {
        lock(&self.requests).clone()
    }

    pub(super) fn posts(&self) -> Vec<Request> {
        self.requests()
            .into_iter()
            .filter(|request| request.method == "POST")
            .collect()
    }

    /// Wait until the server has read `count` requests.
    pub(super) async fn wait_for(&self, count: usize) {
        let deadline = Instant::now() + Duration::from_secs(10);
        while lock(&self.requests).len() < count {
            assert!(Instant::now() < deadline, "{:?}", self.requests());
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    }
}

/// Read one request, record it, and answer it as the script says; `Connection: close`, so
/// every request arrives on a connection of its own.
fn answer(mut stream: TcpStream, script: &Script, recorded: &Mutex<Vec<Request>>) {
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut line = String::new();
    if reader.read_line(&mut line).unwrap_or(0) == 0 {
        return;
    }
    let mut words = line.split_whitespace();
    let method = words.next().unwrap_or_default().to_owned();
    let target = words.next().unwrap_or_default().to_owned();
    let (path, query) = target.split_once('?').unwrap_or((target.as_str(), ""));
    let mut content_type = None;
    let mut length = 0;
    loop {
        line.clear();
        if reader.read_line(&mut line).unwrap_or(0) == 0 {
            return;
        }
        if line == "\r\n" {
            break;
        }
        if let Some((name, value)) = line.split_once(':') {
            let value = value.trim();
            match name.to_ascii_lowercase().as_str() {
                "content-type" => content_type = Some(value.to_owned()),
                "content-length" => length = value.parse().unwrap_or(0),
                _ => {}
            }
        }
    }
    let mut body = vec![0; length];
    if reader.read_exact(&mut body).is_err() {
        return;
    }
    let request = Request {
        method,
        path: path.to_owned(),
        query: query.to_owned(),
        content_type,
        body,
    };
    let reply = script(&request);
    lock(recorded).push(request);
    std::thread::sleep(reply.delay);
    let head = format!(
        "HTTP/1.1 {} Test\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        reply.status,
        reply.body.len()
    );
    let _ = stream
        .write_all(head.as_bytes())
        .and_then(|()| stream.write_all(reply.body.as_bytes()));
}

/// The statuses a share reported, in order.
#[derive(Clone, Default)]
pub(super) struct Recorded(Arc<Mutex<Vec<ZoomStatus>>>);

impl Recorded {
    pub(super) fn all(&self) -> Vec<ZoomStatus> {
        lock(&self.0).clone()
    }

    pub(super) fn last(&self) -> Option<ZoomStatus> {
        lock(&self.0).last().cloned()
    }

    /// The last status reported, once `ready` holds for it.
    pub(super) async fn until(&self, ready: impl Fn(&ZoomStatus) -> bool) -> ZoomStatus {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            if let Some(last) = self.last().filter(|status| ready(status)) {
                return last;
            }
            assert!(Instant::now() < deadline, "{:?}", self.all());
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    }
}

/// A `Report` that records into the returned list instead of emitting an event.
pub(super) fn recorder() -> (Recorded, Report) {
    let recorded = Recorded::default();
    let sink = Arc::clone(&recorded.0);
    let report: Report = Box::new(move |status: &ZoomStatus| lock(&sink).push(status.clone()));
    (recorded, report)
}
