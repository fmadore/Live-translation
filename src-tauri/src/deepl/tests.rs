//! The DeepL client: its turns and errors through the runner's caption path, its handshake,
//! and whole sessions against a loopback REST endpoint and WebSocket server. Nothing here
//! reaches DeepL except `live_probe`, which is ignored unless asked for.

use std::time::Duration;

use base64::Engine;
use futures_util::{SinkExt, StreamExt};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio::time::timeout;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{WebSocketStream, accept_hdr_async};
use tokio_util::sync::CancellationToken;

use super::*;
use crate::errors::id;
use crate::realtime::Signal;
use crate::realtime::test_support::{Emitted, Harness, handshake};
use crate::types::{Caption, SessionState, StatusUpdate};

fn config(api_base: &str) -> DeepLConfig {
    DeepLConfig {
        api_key: "deepl-test".into(),
        api_base: api_base.into(),
        target: TargetLanguage::Es,
        origin: Origin::Microphone,
    }
}

fn harness() -> Harness<DeepLClient> {
    Harness::new(config("https://api.deepl.com").into_client())
}

/// A target update, with each segment as `(text, end_time)`.
fn target(concluded: &[(&str, u32)], tentative: &[(&str, u32)]) -> String {
    let segments = |list: &[(&str, u32)]| -> Vec<serde_json::Value> {
        list.iter()
            .map(|(text, end)| serde_json::json!({"text": text, "start_time": 0, "end_time": end}))
            .collect()
    };
    serde_json::json!({"target_transcript_update": {
        "language": "es",
        "concluded": segments(concluded),
        "tentative": segments(tentative),
    }})
    .to_string()
}

fn source(concluded: &[&str], tentative: &[&str]) -> String {
    let segments = |list: &[&str]| -> Vec<serde_json::Value> {
        list.iter()
            .map(|text| serde_json::json!({"language": "en", "text": text, "start_time": 0, "end_time": 0}))
            .collect()
    };
    serde_json::json!({"source_transcript_update": {
        "concluded": segments(concluded),
        "tentative": segments(tentative),
    }})
    .to_string()
}

/// DeepL's own examples, then the rest of the sentence concluded.
#[test]
fn a_concluded_sentence_closes_the_turn() {
    let mut h = harness();
    let outcome = h.send(
        r#"{"source_transcript_update":{"concluded":[{"language":"en","text":"Hello, how are you","start_time":0,"end_time":1500}],"tentative":[{"language":"en","text":" today?","start_time":1500,"end_time":2000}]}}"#,
    );
    assert_eq!(outcome.caption, CaptionUpdate::Interim);
    assert_eq!(outcome.signal, Signal::None, "source text is not activity");
    h.send(&source(&[" today?"], &[]));
    let outcome = h.send(
        r#"{"target_transcript_update":{"language":"es","concluded":[{"text":"Hola, ¿cómo estás","start_time":0,"end_time":1500}],"tentative":[{"text":" hoy?","start_time":1500,"end_time":2000}]}}"#,
    );
    assert_eq!(outcome.signal, Signal::TranscriptActivity);
    let outcome = h.send(&target(&[(" hoy?", 2000)], &[]));
    assert_eq!(outcome.caption, CaptionUpdate::Final);
    let heard = "Hello, how are you today?";
    assert_eq!(
        h.captions,
        [
            Emitted::interim(0, "", "Hello, how are you today?"),
            Emitted::interim(0, "", heard),
            Emitted::interim(0, "Hola, ¿cómo estás hoy?", heard),
            Emitted::final_(0, "Hola, ¿cómo estás hoy?", heard),
        ]
    );
}

/// The tentative text after a concluded sentence is the next sentence's start: not part of the
/// closed line, and shown in the next one.
#[test]
fn the_tentative_tail_of_a_closed_sentence_opens_the_next_turn() {
    let mut h = harness();
    h.send(&target(
        &[("Buenos días a todos.", 1200)],
        &[(" Hoy", 1500)],
    ));
    h.send(&target(&[], &[(" Hoy vamos", 1800)]));
    h.send(&target(&[(" Hoy vamos a", 2000)], &[(" empezar", 2300)]));
    assert_eq!(
        h.captions,
        [
            Emitted::final_(0, "Buenos días a todos.", ""),
            Emitted::interim(1, "Hoy vamos", ""),
            Emitted::interim(1, "Hoy vamos a empezar", ""),
        ]
    );
    // An update that changes nothing is no caption.
    h.send(&target(&[], &[(" empezar", 2300)]));
    assert_eq!(h.captions.len(), 3);
}

/// The idle boundary closes a turn whose last words were still tentative. When DeepL concludes
/// them afterwards, they have already been shown, and are not printed again.
#[test]
fn text_shown_by_an_idle_close_is_not_repeated_when_concluded() {
    let mut h = harness();
    h.send(&target(&[("Muchas", 800)], &[(" gracias", 1400)]));
    assert!(h.proto.keeps_turn_open(&h.acc), "tentative text is pending");
    h.finalize_idle();
    assert!(
        !h.proto.keeps_turn_open(&h.acc),
        "what was pending went out with the final"
    );

    let outcome = h.send(&target(&[(" gracias", 1400)], &[]));
    assert_eq!(outcome.caption, CaptionUpdate::None);
    h.send(&target(&[(" ¿Y usted?", 2600)], &[]));
    assert_eq!(
        h.captions,
        [
            Emitted::interim(0, "Muchas gracias", ""),
            Emitted::final_(0, "Muchas gracias", ""),
            Emitted::final_(1, "¿Y usted?", ""),
        ]
    );
}

/// Without segment times nothing can be matched up, and nothing is dropped.
#[test]
fn segments_without_times_are_never_dropped() {
    let mut h = harness();
    h.send(&target(&[("Uno", 0)], &[(" dos", 0)]));
    h.finalize_idle();
    h.send(&target(&[(" dos.", 0)], &[]));
    assert_eq!(h.captions.last(), Some(&Emitted::final_(1, "dos.", "")));
}

/// A source update, with each segment as `(text, end_time)`.
fn timed_source(concluded: &[(&str, u32)], tentative: &[(&str, u32)]) -> String {
    let segments = |list: &[(&str, u32)]| -> Vec<serde_json::Value> {
        list.iter()
            .map(|(text, end)| serde_json::json!({"language": "en", "text": text, "start_time": 0, "end_time": end}))
            .collect()
    };
    serde_json::json!({"source_transcript_update": {
        "concluded": segments(concluded),
        "tentative": segments(tentative),
    }})
    .to_string()
}

/// What the client logs while `f` runs, as the app's log would hold it.
#[derive(Clone, Default)]
struct Log(std::sync::Arc<std::sync::Mutex<Vec<u8>>>);

impl std::io::Write for Log {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Log {
    fn subscriber(&self) -> impl tracing::Subscriber + Send + Sync + use<> {
        // tracing caches for the whole process whether a callsite is enabled. While a single
        // subscriber is registered it asks only the current thread's, so a callsite another
        // test's thread reached first would be cached as disabled here. A second, idle one,
        // kept for the life of the process, makes it ask each thread's subscriber per event.
        static IDLE: std::sync::OnceLock<tracing::Dispatch> = std::sync::OnceLock::new();
        IDLE.get_or_init(|| tracing::Dispatch::new(tracing::subscriber::NoSubscriber::default()));
        let log = self.clone();
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .with_ansi(false)
            .with_writer(move || log.clone())
            .finish()
    }

    fn text(&self) -> String {
        String::from_utf8_lossy(&self.0.lock().unwrap()).into_owned()
    }
}

/// A turn closed on a concluded sentence showed concluded text alone, which DeepL sends once:
/// what follows is kept even when its times do not increase, as DeepL does not promise they do.
#[test]
fn text_after_a_concluded_sentence_is_kept_whatever_its_times() {
    let mut h = harness();
    h.send(&target(&[("Buenos días.", 1200)], &[]));
    h.send(&target(&[(" Hoy", 1200)], &[]));
    h.send(&target(&[(" empezamos.", 1100)], &[]));
    assert_eq!(
        h.captions,
        [
            Emitted::final_(0, "Buenos días.", ""),
            Emitted::interim(1, "Hoy", ""),
            Emitted::final_(1, "Hoy empezamos.", ""),
        ]
    );
}

/// After the idle boundary closed a turn, a segment ending at or before what it showed is
/// dropped, concluded or tentative; each drop is logged with its times and length, never its
/// text.
#[test]
fn a_segment_an_idle_close_already_showed_is_dropped_and_logged_without_its_text() {
    let log = Log::default();
    let mut h = harness();
    tracing::subscriber::with_default(log.subscriber(), || {
        h.send(&target(&[("Muchas", 800)], &[(" gracias", 1400)]));
        h.finalize_idle();
        let outcome = h.send(&target(&[(" gracias", 1400)], &[(" señora", 900)]));
        assert_eq!(outcome.caption, CaptionUpdate::None);
    });
    assert_eq!(h.captions.len(), 2);
    assert!(
        !h.proto.keeps_turn_open(&h.acc),
        "the dropped tentative is not held"
    );
    let log = log.text();
    assert_eq!(
        log.matches("already shown in a final caption").count(),
        2,
        "{log}"
    );
    assert!(
        log.contains("end_ms=1400") && log.contains("shown_until_ms=1400"),
        "{log}"
    );
    assert!(log.contains("chars=8"), "{log}");
    assert!(
        !log.contains("gracias") && !log.contains("señora"),
        "caption text reached the log: {log}"
    );
}

/// The filter lifts at the first concluded segment past what the final showed: DeepL has moved
/// on, and a later segment with an earlier time is kept rather than lost.
#[test]
fn the_filter_lifts_once_deepl_concludes_past_the_idle_close() {
    let mut h = harness();
    h.send(&target(&[("Muchas", 800)], &[(" gracias", 1400)]));
    h.finalize_idle();
    h.send(&target(&[(" gracias", 1400), (" a", 1600)], &[]));
    h.send(&target(&[(" todos", 1500)], &[]));
    assert_eq!(
        h.captions,
        [
            Emitted::interim(0, "Muchas gracias", ""),
            Emitted::final_(0, "Muchas gracias", ""),
            Emitted::interim(1, "a", ""),
            Emitted::interim(1, "a todos", ""),
        ]
    );
}

/// The tentative start that a concluded sentence leaves behind is not on screen: an update that
/// repeats it unchanged shows it, and DeepL holds it, so a Pause or Stop drains for it.
#[test]
fn the_tail_of_a_closed_sentence_shows_on_the_next_update_even_unchanged() {
    let mut h = harness();
    h.send(&target(
        &[("Buenos días a todos.", 1200)],
        &[(" Hoy", 1500)],
    ));
    assert!(h.acc.is_empty());
    assert!(h.proto.keeps_turn_open(&h.acc), "the tail is still to come");
    h.send(&target(&[], &[(" Hoy", 1500)]));
    h.send(&target(&[], &[(" Hoy", 1500)]));
    assert_eq!(
        h.captions,
        [
            Emitted::final_(0, "Buenos días a todos.", ""),
            Emitted::interim(1, "Hoy", ""),
        ]
    );
}

/// The source is matched against its own times as the target is: source words an idle close
/// showed as tentative are not repeated under the next line when DeepL concludes them.
#[test]
fn source_text_an_idle_close_showed_is_not_repeated() {
    let mut h = harness();
    h.send(&timed_source(&[("Thank", 700)], &[(" you", 1400)]));
    h.send(&target(&[("Muchas", 800)], &[(" gracias", 1400)]));
    h.finalize_idle();
    let outcome = h.send(&timed_source(&[(" you", 1400)], &[]));
    assert_eq!(outcome.caption, CaptionUpdate::None);
    h.send(&timed_source(&[(" And", 1900)], &[]));
    assert_eq!(
        h.captions,
        [
            Emitted::interim(0, "", "Thank you"),
            Emitted::interim(0, "Muchas gracias", "Thank you"),
            Emitted::final_(0, "Muchas gracias", "Thank you"),
            Emitted::interim(1, "", "And"),
        ]
    );
}

/// What stays a limitation: a line closed on a concluded sentence takes all the source DeepL
/// has concluded so far, which can run into the next sentence; the source's tentative text
/// goes to the next line. Source and target times are not matched against each other.
#[test]
fn a_closed_sentence_takes_the_source_concluded_so_far() {
    let mut h = harness();
    h.send(&source(&["Good morning everyone.", " Today"], &[" we"]));
    h.send(&target(&[("Buenos días a todos.", 1200)], &[]));
    h.send(&target(&[], &[(" Hoy", 1500)]));
    assert_eq!(
        &h.captions[1..],
        [
            Emitted::final_(0, "Buenos días a todos.", "Good morning everyone. Today"),
            Emitted::interim(1, "Hoy", "we"),
        ]
    );
}

/// The session token never goes over plain `ws`, except where a debug build allows it for
/// loopback servers; no other scheme opens a socket.
#[test]
fn a_streaming_url_that_is_not_wss_is_refused() {
    let url = |url: &str| Url::parse(url).unwrap();
    assert!(
        check_streaming_scheme(&url("wss://api.deepl.com/v3/voice/realtime/connect"), false)
            .is_ok()
    );
    for refused in [
        "ws://api.deepl.com/v3/voice/realtime/connect",
        "https://api.deepl.com/v3/voice/realtime/connect",
        "http://127.0.0.1:9000/connect",
    ] {
        let error = check_streaming_scheme(&url(refused), false).unwrap_err();
        assert!(error.to_string().contains("not wss"), "{refused}: {error}");
    }
    assert!(check_streaming_scheme(&url("ws://127.0.0.1:9000/connect"), true).is_ok());
    assert!(check_streaming_scheme(&url("http://127.0.0.1:9000/connect"), true).is_err());
    assert_eq!(
        ALLOW_PLAIN_WS,
        cfg!(debug_assertions),
        "release builds refuse ws"
    );
}

#[test]
fn the_end_of_the_stream_closes_the_open_turn_and_the_session() {
    let mut h = harness();
    h.proto.resumable = true;
    h.send(&target(&[("Hasta", 600)], &[(" luego", 900)]));
    let outcome = h.send(r#"{"end_of_stream":{}}"#);
    assert!(matches!(outcome.control, MessageControl::Closed));
    assert_eq!(
        h.captions.last(),
        Some(&Emitted::final_(0, "Hasta luego", ""))
    );
    assert!(!h.proto.resumable);

    let outcome = h.send(r#"{"end_of_stream":{}}"#);
    assert_eq!(outcome.caption, CaptionUpdate::None);
    assert!(matches!(outcome.control, MessageControl::Closed));
    for ignored in [
        r#"{"end_of_source_transcript":{}}"#,
        r#"{"end_of_target_transcript":{"language":"es"}}"#,
        "not json",
    ] {
        let outcome = h.send(ignored);
        assert_eq!(outcome.caption, CaptionUpdate::None, "{ignored}");
        assert!(matches!(outcome.control, MessageControl::Continue));
    }
}

/// Every `error` ends DeepL's session; a transient one moves the source to a new session.
#[test]
fn transient_errors_reconnect_and_the_rest_stop_the_source() {
    let mut h = harness();
    for (code, message, reconnects) in [
        (429, "Too many requests", true),
        (500, "Internal error", true),
        (503, "Temporarily unavailable", true),
        (400, "Audio format not supported", false),
        (456, "Quota exceeded", false),
        (403, "Forbidden", false),
    ] {
        h.proto.resumable = true;
        let frame = serde_json::json!({"error": {
            "request_type": "source_media_chunk",
            "error_code": code,
            "reason_code": 4_000_403,
            "error_message": message,
        }})
        .to_string();
        let control = h.send(&frame).control;
        assert!(!h.proto.resumable, "{code}");
        if reconnects {
            assert!(matches!(control, MessageControl::Reconnect), "{code}");
        } else {
            let MessageControl::Fatal(detail) = control else {
                panic!("{code} should stop the source");
            };
            assert_eq!(detail, format!("{message} (error {code}, reason 4000403)"));
        }
    }
}

/// The socket carries no key, only the session's single-use token, encoded for the query.
#[test]
fn the_handshake_carries_only_the_session_token() {
    let mut client = config("https://api.deepl.com").into_client();
    assert!(client.connect_request().is_err(), "no session yet");
    client.streaming_url = Some("wss://api.deepl.com/v3/voice/realtime/connect".into());
    client.token = Some("VGhpcyBp+/=".into());
    assert_eq!(
        handshake(client.connect_request().unwrap()),
        "wss://api.deepl.com/v3/voice/realtime/connect?token=VGhpcyBp%2B%2F%3D\n\
         GET /v3/voice/realtime/connect?token=VGhpcyBp%2B%2F%3D HTTP/1.1\r\n\
         Host: api.deepl.com\r\n\
         Connection: Upgrade\r\n\
         Upgrade: websocket\r\n\
         Sec-WebSocket-Version: 13\r\n\
         Sec-WebSocket-Key: <key>\r\n\
         \r\n"
    );
    assert_eq!(client.setup_json().unwrap(), None);
}

#[test]
fn audio_goes_out_as_source_media_chunks_and_silence_keeps_a_quiet_session() {
    let mut client = config("https://api.deepl.com").into_client();
    assert_eq!(
        client.audio_json("AAA=".into()).unwrap(),
        r#"{"source_media_chunk":{"data":"AAA="}}"#
    );
    let keepalive = client.keepalive().unwrap();
    assert!(keepalive.after < Duration::from_secs(30));
    assert_eq!(keepalive.silence.pcm_le.len(), 3_200);
    assert!(keepalive.silence.pcm_le.iter().all(|&byte| byte == 0));

    assert_eq!(
        client.audio_frame(&[0, 0, 0]).unwrap(),
        r#"{"source_media_chunk":{"data":"AAAA"}}"#
    );

    client.resumable = true;
    assert_eq!(
        client.closing_json().unwrap(),
        [r#"{"end_of_source_media":{}}"#]
    );
    assert!(!client.resumable, "ending the input ends the session");
}

// --- Whole sessions against loopback servers -------------------------------------------------

/// Long enough for any step, short enough that a hang fails in seconds.
const GUARD: Duration = Duration::from_secs(30);

#[derive(Debug, PartialEq, Eq)]
enum Event {
    Status(SessionState, Option<(&'static str, String)>),
    Caption(String, bool),
}

struct Recorder(mpsc::UnboundedSender<Event>);

impl crate::realtime::Events for Recorder {
    fn caption(&self, caption: Caption<'_>) {
        let _ = self
            .0
            .send(Event::Caption(caption.text.to_string(), caption.final_));
    }

    fn status(&self, update: StatusUpdate) {
        let message = update
            .message
            .map(|error| (error.id, error.detail.unwrap_or_default()));
        let _ = self.0.send(Event::Status(update.state, message));
    }
}

fn status(state: SessionState) -> Event {
    Event::Status(state, None)
}

/// A socket DeepL closed as soon as it opened, before sending anything: with no setup
/// acknowledgement to wait for, that counts as a failed setup, and says so.
fn closed_at_once() -> Event {
    Event::Status(
        SessionState::Reconnecting,
        Some((
            id::PROVIDER_RECONNECTING,
            "DeepL closed the connection as soon as it opened".into(),
        )),
    )
}

struct Client {
    events: mpsc::UnboundedReceiver<Event>,
    audio: mpsc::Sender<AudioChunk>,
    pause: tokio::sync::watch::Sender<bool>,
    cancel: CancellationToken,
    task: tokio::task::JoinHandle<()>,
}

impl Client {
    fn start(api_base: &str) -> Self {
        let (recorder, events) = mpsc::unbounded_channel();
        let (audio, audio_rx) = mpsc::channel(5);
        let (pause, pause_rx) = tokio::sync::watch::channel(false);
        let cancel = CancellationToken::new();
        let task = tokio::spawn(crate::realtime::run_session(
            Recorder(recorder),
            config(api_base).into_client(),
            audio_rx,
            cancel.clone(),
            crate::timing::SessionClock::start(),
            pause_rx,
            0,
        ));
        Self {
            events,
            audio,
            pause,
            cancel,
            task,
        }
    }

    async fn expect(&mut self, expected: Event) {
        assert_eq!(self.next().await, expected);
    }

    async fn next(&mut self) -> Event {
        timeout(GUARD, self.events.recv())
            .await
            .expect("still waiting for an event")
            .expect("the runner ended")
    }

    async fn finished(mut self) {
        timeout(GUARD, self.task)
            .await
            .expect("the runner did not end")
            .unwrap();
        if let Ok(event) = self.events.try_recv() {
            panic!("unexpected {event:?}");
        }
    }
}

/// A loopback stand-in for `api.deepl.com`'s session endpoint, one request per connection.
struct Rest {
    listener: TcpListener,
    base: String,
}

/// One request as the server read it.
struct Request {
    /// `POST /v3/voice/realtime HTTP/1.1`, as sent.
    line: String,
    /// Every header line, lower-cased.
    headers: String,
    body: String,
}

impl Rest {
    async fn bind() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        Self { listener, base }
    }

    /// Answer the next request with `status` and a JSON `body`.
    async fn answer(&self, status: &str, body: &str) -> Request {
        let (mut stream, request) = self.receive().await;
        let response = format!(
            "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\
             X-Trace-ID: trace-test\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        stream.write_all(response.as_bytes()).await.unwrap();
        let _ = stream.shutdown().await;
        request
    }

    /// Read the next request and close the connection without answering it.
    async fn hang_up(&self) -> Request {
        self.receive().await.1
    }

    async fn receive(&self) -> (TcpStream, Request) {
        let (mut stream, _) = timeout(GUARD, self.listener.accept())
            .await
            .expect("no session request came")
            .unwrap();
        let mut read = Vec::new();
        let mut buffer = [0u8; 4096];
        let (head_len, content_length) = loop {
            let n = stream.read(&mut buffer).await.unwrap();
            assert!(n > 0, "the request ended early");
            read.extend_from_slice(&buffer[..n]);
            if let Some(end) = read.windows(4).position(|w| w == b"\r\n\r\n") {
                let head = String::from_utf8_lossy(&read[..end]).to_lowercase();
                let length = head
                    .lines()
                    .find_map(|line| line.strip_prefix("content-length:"))
                    .map_or(0, |value| value.trim().parse().unwrap());
                break (end + 4, length);
            }
        };
        while read.len() < head_len + content_length {
            let n = stream.read(&mut buffer).await.unwrap();
            assert!(n > 0, "the request body ended early");
            read.extend_from_slice(&buffer[..n]);
        }
        let head = String::from_utf8_lossy(&read[..head_len - 4]).to_string();
        let (line, headers) = head.split_once("\r\n").unwrap_or((&head, ""));
        let request = Request {
            line: line.to_string(),
            headers: headers.to_lowercase(),
            body: String::from_utf8_lossy(&read[head_len..]).to_string(),
        };
        (stream, request)
    }
}

/// A loopback stand-in for the streaming URL DeepL hands out.
struct Stream {
    listener: TcpListener,
    url: String,
}

impl Stream {
    async fn bind() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!(
            "ws://{}/v3/voice/realtime/connect",
            listener.local_addr().unwrap()
        );
        Self { listener, url }
    }

    fn ticket(&self, token: &str) -> String {
        serde_json::json!({"streaming_url": self.url, "token": token, "session_id": "s"})
            .to_string()
    }

    /// The next socket, and the path and query it asked for.
    async fn accept(&self) -> (String, Peer) {
        let (stream, _) = timeout(GUARD, self.listener.accept())
            .await
            .expect("the client did not connect")
            .unwrap();
        stream.set_nodelay(true).unwrap();
        let mut uri = String::new();
        #[expect(
            clippy::result_large_err,
            reason = "the signature of tungstenite's handshake callback"
        )]
        let ws = accept_hdr_async(
            stream,
            |request: &tokio_tungstenite::tungstenite::handshake::server::Request, response| {
                uri = request.uri().to_string();
                Ok(response)
            },
        )
        .await
        .unwrap();
        (uri, Peer(ws))
    }
}

struct Peer(WebSocketStream<TcpStream>);

impl Peer {
    /// The next text frame the client sent.
    async fn recv(&mut self) -> String {
        loop {
            let message = timeout(GUARD, self.0.next())
                .await
                .expect("the client sent nothing")
                .expect("the socket ended")
                .unwrap();
            match message {
                Message::Text(text) => return text.to_string(),
                Message::Close(_) => return "<close>".into(),
                _ => {}
            }
        }
    }

    async fn send(&mut self, text: &str) {
        self.0.send(Message::text(text)).await.unwrap();
    }
}

fn audio_of(frame: &str) -> Vec<u8> {
    let value: serde_json::Value = serde_json::from_str(frame).unwrap();
    let data = value["source_media_chunk"]["data"]
        .as_str()
        .unwrap_or_else(|| panic!("not an audio chunk: {frame}"));
    base64::engine::general_purpose::STANDARD
        .decode(data)
        .unwrap()
}

/// A whole session: requested over REST, opened on its token with audio straight after the
/// handshake, resumed with the latest token after the socket drops, and ended on Stop with
/// `end_of_source_media`, its last caption and `end_of_stream`.
#[tokio::test]
async fn a_session_is_requested_resumed_after_a_drop_and_drained_on_stop() {
    let rest = Rest::bind().await;
    let stream = Stream::bind().await;
    let mut client = Client::start(&rest.base);
    client.expect(status(SessionState::Connecting)).await;

    let request = rest.answer("200 OK", &stream.ticket("first+/=")).await;
    assert_eq!(request.line, "POST /v3/voice/realtime HTTP/1.1");
    assert!(
        request
            .headers
            .contains("authorization: deepl-auth-key deepl-test"),
        "{}",
        request.headers
    );
    assert_eq!(
        request.body,
        r#"{"source_media_content_type":"audio/pcm;encoding=s16le;rate=16000","message_format":"json","source_language_mode":"auto","target_languages":["es"]}"#
    );
    let (uri, mut peer) = stream.accept().await;
    assert_eq!(uri, "/v3/voice/realtime/connect?token=first%2B%2F%3D");
    client.expect(status(SessionState::Running)).await;

    // No setup message: the first frame is audio.
    client
        .audio
        .send(AudioChunk {
            pcm_le: vec![1, 0, 2, 0],
        })
        .await
        .unwrap();
    assert_eq!(audio_of(&peer.recv().await), [1, 0, 2, 0]);

    peer.send(&target(&[("Hola, ¿cómo estás", 1500)], &[(" hoy?", 2000)]))
        .await;
    client
        .expect(Event::Caption("Hola, ¿cómo estás hoy?".into(), false))
        .await;
    peer.send(&target(&[(" hoy?", 2000)], &[])).await;
    client
        .expect(Event::Caption("Hola, ¿cómo estás hoy?".into(), true))
        .await;

    // The socket goes, the session does not: it is resumed with the latest token.
    peer.0.close(None).await.unwrap();
    drop(peer);
    client.expect(status(SessionState::Reconnecting)).await;
    let request = rest.answer("200 OK", &stream.ticket("second")).await;
    assert_eq!(
        request.line,
        "GET /v3/voice/realtime?token=first%2B%2F%3D HTTP/1.1"
    );
    assert!(
        request
            .headers
            .contains("authorization: deepl-auth-key deepl-test")
    );
    let (uri, mut peer) = stream.accept().await;
    assert_eq!(uri, "/v3/voice/realtime/connect?token=second");
    client.expect(status(SessionState::Running)).await;

    client.cancel.cancel();
    assert_eq!(peer.recv().await, r#"{"end_of_source_media":{}}"#);
    peer.send(&target(&[("Adiós.", 3000)], &[])).await;
    peer.send(r#"{"end_of_target_transcript":{"language":"es"}}"#)
        .await;
    peer.send(r#"{"end_of_stream":{}}"#).await;
    client.expect(Event::Caption("Adiós.".into(), true)).await;
    assert_eq!(peer.recv().await, "<close>");
    client.finished().await;
}

/// After DeepL's `error` the session is over, so the next socket gets a new one; after a drop
/// whose session DeepL no longer knows, the resume falls back to a new one; and an error no
/// retry fixes ends the source with DeepL's own words.
#[tokio::test]
async fn an_ended_session_is_replaced_and_a_refusal_stops_the_source() {
    let rest = Rest::bind().await;
    let stream = Stream::bind().await;
    let mut client = Client::start(&rest.base);
    client.expect(status(SessionState::Connecting)).await;
    rest.answer("200 OK", &stream.ticket("one")).await;
    let (_, mut peer) = stream.accept().await;
    client.expect(status(SessionState::Running)).await;

    peer.send(
        r#"{"error":{"request_type":"source_media_chunk","error_code":503,"reason_code":5030000,"error_message":"Temporarily unavailable"}}"#,
    )
    .await;
    client.expect(status(SessionState::Reconnecting)).await;
    let request = rest.answer("200 OK", &stream.ticket("two")).await;
    assert_eq!(request.line, "POST /v3/voice/realtime HTTP/1.1");
    drop(peer);
    let (uri, mut peer) = stream.accept().await;
    assert_eq!(uri, "/v3/voice/realtime/connect?token=two");
    client.expect(status(SessionState::Running)).await;

    peer.0.close(None).await.unwrap();
    drop(peer);
    client.expect(closed_at_once()).await;
    client.expect(status(SessionState::Reconnecting)).await;
    let request = rest
        .answer("400 Bad Request", r#"{"message":"Token outdated"}"#)
        .await;
    assert_eq!(request.line, "GET /v3/voice/realtime?token=two HTTP/1.1");
    let request = rest.answer("200 OK", &stream.ticket("three")).await;
    assert_eq!(request.line, "POST /v3/voice/realtime HTTP/1.1");
    let (uri, mut peer) = stream.accept().await;
    assert_eq!(uri, "/v3/voice/realtime/connect?token=three");
    client.expect(status(SessionState::Running)).await;

    peer.send(
        r#"{"error":{"request_type":"source_media_chunk","error_code":400,"reason_code":4000403,"error_message":"Audio format not supported"}}"#,
    )
    .await;
    client
        .expect(Event::Status(
            SessionState::Error,
            Some((
                id::PROVIDER_STOPPED,
                "DeepL — Audio format not supported (error 400, reason 4000403)".into(),
            )),
        ))
        .await;
    client.finished().await;
}

/// A refused session request ends the source as a refused handshake would: the key or plan
/// as a rejection, a spent quota in DeepL's words.
#[tokio::test]
async fn refused_session_requests_end_the_source() {
    for (status_line, body, expected) in [
        (
            "403 Forbidden",
            r#"{"message":"Forbidden"}"#,
            (id::PROVIDER_REJECTED, "DeepL — HTTP 403"),
        ),
        (
            "456 Quota Exceeded",
            r#"{"message":"Quota exceeded","code":"quota_exceeded"}"#,
            (id::PROVIDER_STOPPED, "DeepL — HTTP 456: Quota exceeded"),
        ),
    ] {
        let rest = Rest::bind().await;
        let mut client = Client::start(&rest.base);
        client.expect(status(SessionState::Connecting)).await;
        rest.answer(status_line, body).await;
        client
            .expect(Event::Status(
                SessionState::Error,
                Some((expected.0, expected.1.into())),
            ))
            .await;
        client.finished().await;
    }
}

/// A rate limit is retried, with the status as the Reconnecting detail.
#[tokio::test]
async fn a_rate_limited_session_request_is_retried() {
    let rest = Rest::bind().await;
    let stream = Stream::bind().await;
    let mut client = Client::start(&rest.base);
    client.expect(status(SessionState::Connecting)).await;
    rest.answer(
        "429 Too Many Requests",
        r#"{"error":{"message":"Slow down"}}"#,
    )
    .await;
    client
        .expect(Event::Status(
            SessionState::Reconnecting,
            Some((
                id::PROVIDER_RECONNECTING,
                "session request answered HTTP 429: Slow down".into(),
            )),
        ))
        .await;
    client.expect(status(SessionState::Reconnecting)).await;
    let request = rest.answer("200 OK", &stream.ticket("again")).await;
    assert_eq!(request.line, "POST /v3/voice/realtime HTTP/1.1");
    let (_, peer) = stream.accept().await;
    client.expect(status(SessionState::Running)).await;
    client.cancel.cancel();
    drop(peer);
    client.finished().await;
}

/// A resume whose request fails says why in the Reconnecting detail. The resume's URL carries
/// the session token, and neither that detail nor the log may.
#[tokio::test]
async fn a_failed_resume_never_shows_or_logs_the_token() {
    const TOKEN: &str = "resume-secret-token";
    let log = Log::default();
    let _log = tracing::subscriber::set_default(log.subscriber());
    let rest = Rest::bind().await;
    let stream = Stream::bind().await;
    let mut client = Client::start(&rest.base);
    client.expect(status(SessionState::Connecting)).await;
    rest.answer("200 OK", &stream.ticket(TOKEN)).await;
    let (_, mut peer) = stream.accept().await;
    client.expect(status(SessionState::Running)).await;

    peer.0.close(None).await.unwrap();
    drop(peer);
    client.expect(closed_at_once()).await;
    client.expect(status(SessionState::Reconnecting)).await;
    let request = rest.hang_up().await;
    assert_eq!(
        request.line,
        format!("GET /v3/voice/realtime?token={TOKEN} HTTP/1.1")
    );
    let Event::Status(SessionState::Reconnecting, Some((error, detail))) = client.next().await
    else {
        panic!("expected a Reconnecting with a reason");
    };
    assert_eq!(error, id::PROVIDER_RECONNECTING);
    assert!(
        detail.starts_with("DeepL session reconnect failed"),
        "{detail}"
    );
    assert!(!detail.contains(TOKEN), "{detail}");

    client.cancel.cancel();
    client.finished().await;
    let log = log.text();
    assert!(log.contains("session reconnect failed"), "{log}");
    assert!(!log.contains(TOKEN), "{log}");
}

/// Every Pause ends DeepL's session with `end_of_source_media`, so the Resume requests a new
/// one rather than resuming the old.
#[tokio::test]
async fn resume_after_a_pause_requests_a_new_session() {
    let rest = Rest::bind().await;
    let stream = Stream::bind().await;
    let mut client = Client::start(&rest.base);
    client.expect(status(SessionState::Connecting)).await;
    rest.answer("200 OK", &stream.ticket("before")).await;
    let (_, mut peer) = stream.accept().await;
    client.expect(status(SessionState::Running)).await;

    client.pause.send_replace(true);
    assert_eq!(peer.recv().await, r#"{"end_of_source_media":{}}"#);
    peer.send(r#"{"end_of_stream":{}}"#).await;
    assert_eq!(peer.recv().await, "<close>");
    client.expect(status(SessionState::Paused)).await;

    client.pause.send_replace(false);
    client.expect(status(SessionState::Connecting)).await;
    let request = rest.answer("200 OK", &stream.ticket("after")).await;
    assert_eq!(request.line, "POST /v3/voice/realtime HTTP/1.1");
    let (uri, peer) = stream.accept().await;
    assert_eq!(uri, "/v3/voice/realtime/connect?token=after");
    client.expect(status(SessionState::Running)).await;

    client.cancel.cancel();
    drop(peer);
    client.finished().await;
}

/// Records what `run_session` reports, for the live probe below.
struct Live(std::sync::mpsc::Sender<(std::time::Instant, String, String, bool)>);

impl crate::realtime::Events for Live {
    fn caption(&self, caption: Caption<'_>) {
        let _ = self.0.send((
            std::time::Instant::now(),
            caption.text.to_string(),
            caption.source_text.to_string(),
            caption.final_,
        ));
    }

    fn status(&self, update: StatusUpdate) {
        if let Some(error) = update.message {
            let detail = format!("{:?}: {}", update.state, error.detail.unwrap_or_default());
            let _ = self
                .0
                .send((std::time::Instant::now(), detail, String::new(), false));
        }
    }
}

/// The bundled English rehearsal through the production client and runner, against the real
/// Voice API, translated into German (or `DEEPL_PROBE_TARGET`, a catalog code). Prints every
/// final caption with its source text and when it closed, and any status with a reason.
/// Ignored: it bills the key, and CI never runs it.
///
/// ```text
/// DEEPL_API_KEY=... cargo test --manifest-path src-tauri/Cargo.toml --lib live_probe -- --ignored --nocapture
/// ```
#[tokio::test]
#[ignore = "streams the bundled fixture to DeepL and bills the key; run with --ignored"]
async fn live_probe() {
    use crate::audio::fixture::load_fixture;
    use crate::audio::{chunk_samples, resample::f32_to_pcm16_le};

    let _ = dotenvy::dotenv();
    let api_key = std::env::var("DEEPL_API_KEY")
        .expect("set DEEPL_API_KEY, or copy .env.example to .env at the repo root");
    assert!(
        crate::secrets::check_plan(crate::types::Provider::DeepL, &api_key).is_ok(),
        "a DeepL API Free key (:fx) cannot reach the Voice API"
    );
    let target: TargetLanguage = serde_json::from_value(serde_json::Value::String(
        std::env::var("DEEPL_PROBE_TARGET").unwrap_or_else(|_| "de".into()),
    ))
    .expect("DEEPL_PROBE_TARGET must be a catalog language code");
    let config = DeepLConfig {
        api_key,
        api_base: format!("https://{DEFAULT_DEEPL_HOST}"),
        target,
        origin: Origin::Microphone,
    };
    let samples = load_fixture(std::path::Path::new("resources/fixtures/rehearsal-en.wav"))
        .expect("the bundled rehearsal fixture should load");

    let (events, recorded) = std::sync::mpsc::channel();
    let (audio, audio_rx) = mpsc::channel(5);
    let (_pause, pause_rx) = tokio::sync::watch::channel(false);
    let cancel = CancellationToken::new();
    let session = tokio::spawn(crate::realtime::run_session(
        Live(events),
        config.into_client(),
        audio_rx,
        cancel.clone(),
        crate::timing::SessionClock::start(),
        pause_rx,
        0,
    ));
    // At the pace of speech: DeepL caps input at twice real time.
    for chunk in samples.chunks(chunk_samples(16_000)) {
        let mut pcm_le = Vec::new();
        f32_to_pcm16_le(chunk, &mut pcm_le);
        audio.send(AudioChunk { pcm_le }).await.unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    // The room going quiet for a moment, then Stop and its drain.
    tokio::time::sleep(Duration::from_secs(3)).await;
    cancel.cancel();
    session.await.unwrap();

    let events: Vec<_> = recorded.try_iter().collect();
    let start = events.first().map(|(at, ..)| *at);
    for (at, text, source, final_) in &events {
        let at = start
            .map(|start| at.duration_since(start))
            .unwrap_or_default();
        if *final_ {
            println!("{at:>12?} {text}\n{:>12} ({source})", "");
        } else if source.is_empty() && text.contains(": ") {
            println!("{at:>12?} status {text}");
        }
    }
    let finals = events.iter().filter(|(.., final_)| *final_).count();
    assert!(finals >= 2, "expected several final captions: {events:#?}");
}
