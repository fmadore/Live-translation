//! Mistral Voxtral Mini Transcribe Realtime 3. Unlike the translation providers, its
//! transcript is the audience caption itself: it goes in the accumulator's `text`, where a
//! translation would, and is exported through the existing caption/transcript path.

use std::time::Duration;

use anyhow::Result;
use tokio_tungstenite::tungstenite::handshake::client::Request;

use super::protocol::{InputAudioAppend, ServerEvent, SessionUpdate};
use crate::realtime::{
    bearer_request, parse_or_log, CaptionUpdate, MessageControl, MessageOutcome, RealtimeProtocol,
    Signal, TurnAccumulator,
};
use crate::types::Origin;

pub const DEFAULT_MISTRAL_MODEL: &str = "voxtral-mini-transcribe-realtime-3-0";
pub const DEFAULT_MISTRAL_HOST: &str = "api.mistral.ai";
/// A small context window keeps subtitles responsive while improving recognition quality.
pub const DEFAULT_TARGET_STREAMING_DELAY_MS: u32 = 480;
const FINALIZE_AFTER: Duration = Duration::from_millis(900);

#[derive(Clone)]
pub struct MistralConfig {
    pub api_key: String,
    pub model: String,
    pub host: String,
    pub target_streaming_delay_ms: u32,
    pub origin: Origin,
}

impl MistralConfig {
    /// Realtime 2602, the model before Realtime 3, still reachable through
    /// `MISTRAL_TRANSCRIBE_MODEL` for comparison. It rejects turn detection.
    fn is_realtime_2602(&self) -> bool {
        self.model.contains("2602")
    }

    /// The protocol the realtime runner drives for this source, with nothing heard yet.
    pub fn into_client(self) -> MistralClient {
        MistralClient {
            config: self,
            received_delta: false,
            mid_turn: false,
        }
    }

    fn ws_url(&self) -> String {
        format!(
            "wss://{}/v1/audio/transcriptions/realtime?model={}",
            self.host, self.model
        )
    }
}

/// One Mistral source: its configuration, plus what the provider has sent on the current
/// connection that a later event depends on. That part is the protocol's own business, so it
/// lives here rather than in a configuration the session has to fill in.
pub struct MistralClient {
    config: MistralConfig,
    /// Whether the provider's current session has streamed any transcript delta. Cleared by
    /// the `session.created` that opens each connection.
    received_delta: bool,
    /// Whether turn detection last said the speaker is talking: after `turn.start` or
    /// `turn.resume`, until `turn.eager_end` or `turn.end`. Never set on Realtime 2602.
    mid_turn: bool,
}

impl RealtimeProtocol for MistralClient {
    const NAME: &'static str = "Mistral";

    fn origin(&self) -> Origin {
        self.config.origin
    }

    fn connect_request(&self) -> Result<Request> {
        bearer_request(Self::NAME, &self.config.ws_url(), &self.config.api_key)
    }

    fn setup_json(&self) -> Result<String> {
        let update = SessionUpdate::pcm16(self.config.target_streaming_delay_ms);
        if self.config.is_realtime_2602() {
            return Ok(serde_json::to_string(&update)?);
        }
        Ok(serde_json::to_string(&update.with_turn_detection())?)
    }

    fn audio_json(&self, base64_pcm: String) -> Result<String> {
        Ok(serde_json::to_string(&InputAudioAppend::pcm16(base64_pcm))?)
    }

    fn closing_json(&self) -> Result<Vec<String>> {
        Ok(vec![
            r#"{"type":"input_audio.flush"}"#.to_string(),
            r#"{"type":"input_audio.end"}"#.to_string(),
        ])
    }

    fn handle_message(&mut self, text: &str, acc: &mut TurnAccumulator) -> MessageOutcome {
        let Some(event) = parse_or_log::<ServerEvent>(Self::NAME, text) else {
            return MessageOutcome::default();
        };

        match event.kind.as_str() {
            "session.created" | "session.updated" => {
                self.received_delta = false;
                self.mid_turn = false;
                tracing::debug!(origin = ?self.config.origin, event = %event.kind, "Mistral session ready");
            }
            "turn.start" | "turn.resume" => self.mid_turn = true,
            // Most pauses between sentences send one, followed by `turn.resume`.
            "turn.eager_end" => self.mid_turn = false,
            "transcription.text.delta" => {
                if let Some(delta) = event.text.as_deref().filter(|delta| !delta.is_empty()) {
                    self.received_delta = true;
                    acc.text.push_str(delta);
                    return MessageOutcome::activity(CaptionUpdate::Interim);
                }
            }
            // The speaker finished. Their last words are written a streaming delay after
            // they were spoken, so they can still follow this event; anything spoken after
            // it arrives later than that. The turn closes once the delay has passed.
            "turn.end" => {
                self.mid_turn = false;
                tracing::debug!(origin = ?self.config.origin, reason = ?event.end_reason, "Mistral end of turn");
                return MessageOutcome {
                    signal: Signal::EndOfTurn(Duration::from_millis(u64::from(
                        self.config.target_streaming_delay_ms,
                    ))),
                    ..MessageOutcome::default()
                };
            }
            // The drain's end after `input_audio.end`. Mid-stream the final caption still goes
            // out, and the runner moves to a new session rather than ending the source.
            "transcription.done" => {
                // `done.text` contains the full session transcript. Only use it when the
                // server sent no deltas; otherwise idle-finalized turns would be duplicated.
                if !self.received_delta && acc.text.is_empty() {
                    if let Some(full_text) = event.text {
                        acc.text = full_text;
                    }
                }
                let caption = if acc.is_empty() {
                    CaptionUpdate::None
                } else {
                    CaptionUpdate::Final
                };
                return MessageOutcome {
                    caption,
                    control: MessageControl::Closed,
                    ..MessageOutcome::default()
                };
            }
            "error" => {
                if event.error.as_ref().is_some_and(recoverable) {
                    tracing::warn!(
                        origin = ?self.config.origin,
                        error = ?event.error,
                        "Mistral reported a transient error; reconnecting"
                    );
                    return MessageOutcome::control(MessageControl::Reconnect);
                }
                return MessageOutcome::control(MessageControl::Fatal(
                    event
                        .error
                        .map(|error| error.to_string())
                        .unwrap_or_else(|| "Mistral realtime transcription error".to_string()),
                ));
            }
            _ => {}
        }

        MessageOutcome::default()
    }

    fn finalize_after(&self) -> Option<Duration> {
        Some(FINALIZE_AFTER)
    }

    /// Text can stall for longer than the idle boundary in the middle of a sentence. When turn
    /// detection says the speaker is still talking and the text has not ended a sentence, the
    /// line stays open; a sentence that has ended still closes at the boundary, so a speaker
    /// who never pauses for a turn end does not make one endless line.
    fn keeps_turn_open(&self, acc: &TurnAccumulator) -> bool {
        self.mid_turn && !ends_sentence(&acc.text)
    }
}

/// Whether the text ends a sentence, closing quotes and brackets aside, and the (often
/// non-breaking) space French sets before them.
fn ends_sentence(text: &str) -> bool {
    text.trim_end_matches(|c: char| {
        c.is_whitespace() || matches!(c, '"' | '\'' | '”' | '’' | '»' | ')' | ']')
    })
    .ends_with(['.', '?', '!', '…', '。', '？', '！', '؟', '।'])
}

/// Wording that marks a failure inside Mistral's backend rather than in the request: gRPC's
/// UNAVAILABLE and DEADLINE_EXCEEDED, and a connection reset between its services. Seen live
/// on 7 October 2026 as code 3803, "gRPC connection error: <AioRpcError … StatusCode.UNAVAILABLE
/// … Connection reset by peer …>"; a new session reaches a backend that is up. Specific on
/// purpose: a bare "unavailable" could also describe a model the key cannot use.
const TRANSIENT_BACKEND: [&str; 5] = [
    "statuscode.unavailable",
    "grpc_status:14",
    "statuscode.deadline_exceeded",
    "grpc connection error",
    "connection reset by peer",
];

/// Whether an `error` event is one a new session can get past. The official SDK types it as
/// `{ message, code }`: `message` a string or an object, `code` an integer it calls an
/// internal code for debugging. Codes in HTTP's transient range (timeout, rate limit, server
/// errors) reconnect, as do the transient `type`/`code` names Mistral's HTTP API uses, should
/// `message` be such an object. Everything else — authentication, permission, a bad request
/// or model, an exhausted quota, an unknown code — stays fatal, so a persistent failure
/// reports itself instead of reconnecting in a loop. One exception by wording: Mistral's own
/// backend failing to reach an internal service, which it reports under an internal code with
/// the gRPC failure as the message (`TRANSIENT_BACKEND`).
fn recoverable(error: &serde_json::Value) -> bool {
    let code = error
        .get("code")
        .and_then(|code| code.as_u64().or_else(|| code.as_str()?.parse().ok()));
    let message = error.get("message");
    let names: Vec<&str> = [error, message.unwrap_or(&serde_json::Value::Null)]
        .into_iter()
        .flat_map(|value| ["type", "code"].map(|key| value.get(key)))
        .filter_map(|name| name?.as_str())
        .collect();
    let text = message.map(|message| message.to_string().to_lowercase());
    // A 429 is also how an exhausted quota or plan limit is reported; no reconnect fixes that.
    if text.as_deref().is_some_and(|text| {
        ["quota", "billing", "payment", "credit"]
            .iter()
            .any(|w| text.contains(w))
    }) {
        return false;
    }
    if text
        .as_deref()
        .is_some_and(|text| TRANSIENT_BACKEND.iter().any(|w| text.contains(w)))
    {
        return true;
    }
    matches!(code, Some(408 | 429 | 500 | 502 | 503 | 504))
        || names.iter().any(|name| {
            matches!(
                *name,
                "server_error"
                    | "rate_limit_error"
                    | "rate_limited"
                    | "timeout"
                    | "session_expired"
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::realtime::test_support::{handshake, Emitted, Harness};
    use crate::realtime::Signal;

    fn config(api_key: &str) -> MistralConfig {
        MistralConfig {
            api_key: api_key.to_string(),
            model: DEFAULT_MISTRAL_MODEL.to_string(),
            host: DEFAULT_MISTRAL_HOST.to_string(),
            target_streaming_delay_ms: DEFAULT_TARGET_STREAMING_DELAY_MS,
            origin: Origin::Microphone,
        }
    }

    fn harness() -> Harness<MistralClient> {
        Harness::new(config("").into_client())
    }

    fn setup(config: MistralConfig) -> serde_json::Value {
        serde_json::from_str(&config.into_client().setup_json().unwrap()).unwrap()
    }

    #[test]
    fn realtime_3_detects_turns_and_2602_does_not() {
        let mut config = config("");
        let session = &setup(config.clone())["session"];
        assert_eq!(session["turn_detection"], serde_json::json!({}));

        // As Mistral answered on 21 October 2026: "Model '…-2602' does not support turn
        // detection.", code 3051.
        config.model = "voxtral-mini-transcribe-realtime-2602".into();
        let session = &setup(config)["session"];
        assert!(session.get("turn_detection").is_none());
    }

    #[test]
    fn the_end_of_a_turn_finalizes_after_the_streaming_delay() {
        let mut h = harness();
        h.send(r#"{"type":"turn.start","time_s":0.0}"#);
        h.send(r#"{"type":"transcription.text.delta","text":"Ceci est en direct"}"#);
        for kind in ["turn.eager_end", "turn.resume"] {
            let outcome = h.send(&format!(r#"{{"type":"{kind}","time_s":4.72}}"#));
            assert_eq!(outcome.signal, Signal::None, "{kind}");
        }
        let outcome = h.send(r#"{"type":"turn.end","time_s":5.28,"end_reason":"threshold"}"#);
        assert_eq!(outcome.caption, CaptionUpdate::None);
        assert_eq!(
            outcome.signal,
            Signal::EndOfTurn(Duration::from_millis(480))
        );
        // The turn's last word, after its end (seen live on 21 October 2026).
        h.send(r#"{"type":"transcription.text.delta","text":"."}"#);
        h.finalize_idle();
        assert_eq!(
            h.captions.last(),
            Some(&Emitted::final_(0, "Ceci est en direct.", ""))
        );
    }

    /// Seen live on 21 October 2026: text stalls for over a second inside a sentence ("Une |
    /// oratrice", "Utilisez cet | enregistrement"), long enough for the idle boundary to cut it.
    #[test]
    fn a_sentence_stays_open_while_the_speaker_is_mid_turn() {
        let mut h = harness();
        let open = |h: &Harness<MistralClient>| h.proto.keeps_turn_open(&h.acc);
        h.send(r#"{"type":"transcription.text.delta","text":" Une"}"#);
        assert!(!open(&h), "no turn detection, nothing held");

        h.send(r#"{"type":"turn.start","time_s":5.44}"#);
        assert!(open(&h));
        h.send(r#"{"type":"transcription.text.delta","text":" oratrice s'adresse au public."}"#);
        assert!(!open(&h), "a finished sentence closes at the boundary");
        h.send(r#"{"type":"transcription.text.delta","text":" Utilisez cet"}"#);
        assert!(open(&h));
        h.send(r#"{"type":"turn.eager_end","time_s":20.1}"#);
        assert!(!open(&h), "the speaker has probably finished");
        h.send(r#"{"type":"turn.resume","time_s":20.2}"#);
        assert!(open(&h));
        h.send(r#"{"type":"turn.end","time_s":20.9,"end_reason":"timeout"}"#);
        assert!(!open(&h));

        h.send(r#"{"type":"turn.start","time_s":21.0}"#);
        h.send(r#"{"type":"session.created"}"#);
        assert!(!open(&h), "a new session starts with no turn");
    }

    #[test]
    fn sentence_ends_include_closing_quotes_and_other_scripts() {
        for text in [
            "direct\u{a0}?",
            "« Bonjour.\u{a0}»",
            "(fin.)",
            "完了。",
            "انتهى؟",
            "Ende!  ",
        ] {
            assert!(ends_sentence(text), "{text}");
        }
        for text in ["Utilisez cet", "à l'", "Dr", "", "and,"] {
            assert!(!ends_sentence(text), "{text}");
        }
    }

    #[test]
    fn the_handshake_authenticates_with_a_bearer_header() {
        let request = config("mistral-test")
            .into_client()
            .connect_request()
            .unwrap();
        assert_eq!(
            handshake(request),
            "wss://api.mistral.ai/v1/audio/transcriptions/realtime?model=voxtral-mini-transcribe-realtime-3-0\n\
             GET /v1/audio/transcriptions/realtime?model=voxtral-mini-transcribe-realtime-3-0 HTTP/1.1\r\n\
             Host: api.mistral.ai\r\n\
             Connection: Upgrade\r\n\
             Upgrade: websocket\r\n\
             Sec-WebSocket-Version: 13\r\n\
             Sec-WebSocket-Key: <key>\r\n\
             authorization: Bearer mistral-test\r\n\
             \r\n"
        );
    }

    /// The operator reads these as the reconnecting status's detail.
    #[test]
    fn a_request_that_cannot_be_built_says_which_part_failed() {
        let error = config("bad\nkey")
            .into_client()
            .connect_request()
            .unwrap_err();
        assert_eq!(
            error.to_string(),
            "Mistral API key is not a valid header value"
        );
        let mut config = config("mistral-test");
        config.host = "not a host".to_string();
        let error = config.into_client().connect_request().unwrap_err();
        assert_eq!(error.to_string(), "failed to build Mistral request");
    }

    #[test]
    fn deltas_are_interim_activity() {
        let mut h = harness();
        let outcome = h.send(r#"{"type":"transcription.text.delta","text":"Hello "}"#);
        assert_eq!(outcome.caption, CaptionUpdate::Interim);
        assert_eq!(outcome.signal, Signal::TranscriptActivity);
        h.send(r#"{"type":"transcription.text.delta","text":""}"#);
        h.send(r#"{"type":"transcription.text.delta","text":"world"}"#);
        assert_eq!(
            h.captions,
            [
                Emitted::interim(0, "Hello ", ""),
                Emitted::interim(0, "Hello world", ""),
            ]
        );
    }

    /// `done` repeats the whole session. After deltas, idle finalization has already
    /// committed those turns, so replaying `done.text` would print every line twice.
    #[test]
    fn done_after_deltas_does_not_replay_the_session() {
        let mut h = harness();
        h.send(r#"{"type":"transcription.text.delta","text":"First line."}"#);
        h.finalize_idle();
        let outcome = h.send(r#"{"type":"transcription.done","text":"First line."}"#);
        assert!(matches!(outcome.control, MessageControl::Closed));
        assert_eq!(
            h.captions,
            [
                Emitted::interim(0, "First line.", ""),
                Emitted::final_(0, "First line.", ""),
            ]
        );
    }

    #[test]
    fn done_without_deltas_supplies_the_whole_transcript() {
        let mut h = harness();
        h.send(r#"{"type":"session.created"}"#);
        h.send(r#"{"type":"transcription.done","text":"Only the summary."}"#);
        assert_eq!(h.captions, [Emitted::final_(0, "Only the summary.", "")]);
    }

    #[test]
    fn a_new_session_forgets_earlier_deltas() {
        let mut h = harness();
        h.send(r#"{"type":"transcription.text.delta","text":"Before reconnect"}"#);
        h.finalize_idle();
        h.send(r#"{"type":"session.updated"}"#);
        h.send(r#"{"type":"transcription.done","text":"After reconnect"}"#);
        assert_eq!(
            h.captions.last(),
            Some(&Emitted::final_(1, "After reconnect", ""))
        );
    }

    #[test]
    fn transient_errors_reconnect_and_the_rest_stop_the_source() {
        let mut h = harness();
        for (error, reconnects) in [
            (r#"{"message":"Internal server error","code":500}"#, true),
            (r#"{"message":"Service unavailable","code":503}"#, true),
            (r#"{"message":"Rate limit exceeded","code":429}"#, true),
            (
                r#"{"message":{"type":"server_error","detail":"retry"},"code":3000}"#,
                true,
            ),
            // As Mistral sent it on 7 October 2026, mid-session.
            (
                concat!(
                    r#"{"code":3803,"message":"gRPC connection error: <AioRpcError of RPC that "#,
                    r#"terminated with:\n\tstatus = StatusCode.UNAVAILABLE\n\tdetails = \"failed "#,
                    r#"to connect to all addresses; last error: UNAVAILABLE: "#,
                    r#"ipv4:10.235.138.223:50052: recvmsg:Connection reset by peer\"\n\t"#,
                    r#"debug_error_string = \"UNKNOWN:Error received from peer {grpc_status:14, "#,
                    r#"grpc_message:\"failed to connect to all addresses; last error: "#,
                    r#"UNAVAILABLE: ipv4:10.235.138.223:50052: recvmsg:Connection reset by "#,
                    r#"peer\"}\"\n>"}"#
                ),
                true,
            ),
            (
                r#"{"message":"Model voxtral-mini-transcribe is unavailable for this key","code":3001}"#,
                false,
            ),
            // Session configuration errors, as Mistral documents them for Realtime 3.
            (
                r#"{"message":"Session fields cannot be changed after audio has started.","code":3051}"#,
                false,
            ),
            (
                r#"{"message":"target_streaming_delay_ms must be one of the trained values: ['80-1200', '2400'] (got 1300).","code":3051}"#,
                false,
            ),
            (r#"{"message":"Monthly quota exceeded","code":429}"#, false),
            (r#"{"message":"Unauthorized","code":401}"#, false),
            (r#"{"message":"Forbidden","code":403}"#, false),
            (
                r#"{"message":"Invalid model: voxtral-unknown","code":400}"#,
                false,
            ),
            (
                r#"{"message":{"detail":"something new"},"code":3001}"#,
                false,
            ),
            (r#"{"message":"invalid key"}"#, false),
        ] {
            let frame = format!(r#"{{"type":"error","error":{error}}}"#);
            let control = h.send(&frame).control;
            assert_eq!(
                matches!(control, MessageControl::Reconnect),
                reconnects,
                "{error} -> {control:?}"
            );
            if !reconnects {
                assert!(matches!(control, MessageControl::Fatal(_)), "{error}");
            }
        }
        assert!(matches!(
            h.send(r#"{"type":"error"}"#).control,
            MessageControl::Fatal(_)
        ));
    }

    /// Records what `run_session` reports, with when, for the live check below.
    struct Live(std::sync::mpsc::Sender<(std::time::Instant, String, bool)>);

    impl crate::realtime::Events for Live {
        fn caption(&self, caption: crate::types::Caption<'_>) {
            let event = (
                std::time::Instant::now(),
                caption.text.to_string(),
                caption.final_,
            );
            let _ = self.0.send(event);
        }

        fn status(&self, update: crate::types::StatusUpdate) {
            if let Some(error) = update.message {
                let detail = format!("{:?}: {}", update.state, error.detail.unwrap_or_default());
                let _ = self.0.send((std::time::Instant::now(), detail, false));
            }
        }
    }

    /// The French recording through the real runner, with Realtime 3 accepting turn detection.
    /// Its text stalls mid-sentence for longer than the idle boundary, yet every caption must
    /// close on a whole sentence; and the first sentence, the one followed by a detected end of
    /// turn, sooner than the idle boundary would close it.
    #[tokio::test]
    #[ignore = "streams the bundled fixture to Mistral and bills the key; run with --ignored"]
    async fn realtime_3_closes_a_sentence_at_its_detected_end() {
        use crate::audio::fixture::load_fixture;
        use crate::audio::{chunk_samples, resample::f32_to_pcm16_le, AudioChunk};

        let _ = dotenvy::dotenv();
        let config = config(
            &std::env::var("MISTRAL_API_KEY")
                .expect("set MISTRAL_API_KEY, or copy .env.example to .env at the repo root"),
        );
        let samples = load_fixture(std::path::Path::new("resources/fixtures/rehearsal-fr.wav"))
            .expect("the bundled rehearsal fixture should load");

        let (events, recorded) = std::sync::mpsc::channel();
        let (audio, audio_rx) = tokio::sync::mpsc::channel(5);
        let (_pause, pause_rx) = tokio::sync::watch::channel(false);
        let cancel = tokio_util::sync::CancellationToken::new();
        let session = tokio::spawn(crate::realtime::run_session(
            Live(events),
            config.into_client(),
            audio_rx,
            cancel.clone(),
            crate::timing::SessionClock::start(),
            pause_rx,
            0,
        ));
        // At the pace of speech: a recording sent at once is not what a room sounds like.
        for chunk in samples.chunks(chunk_samples(16_000)) {
            let mut pcm_le = Vec::new();
            f32_to_pcm16_le(chunk, &mut pcm_le);
            audio.send(AudioChunk { pcm_le }).await.unwrap();
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        cancel.cancel();
        session.await.unwrap();

        let events: Vec<_> = recorded.try_iter().collect();
        let finals: Vec<_> = events.iter().filter(|(_, _, final_)| *final_).collect();
        for (at, text, _) in &finals {
            println!("{:>12?} {text}", at.duration_since(events[0].0));
        }
        assert!(
            finals.len() >= 2,
            "expected the first sentence as a caption of its own: {events:#?}"
        );
        for (_, text, _) in &finals {
            assert!(ends_sentence(text), "cut mid-sentence: {text}");
        }
        let (closed, first, _) = finals[0];
        assert!(first.contains("en direct"), "{first}");
        let (spoken, _, _) = events
            .iter()
            .rfind(|(at, text, final_)| !final_ && at < closed && text == first)
            .expect("the sentence was shown before it was closed");
        let after = closed.duration_since(*spoken);
        println!("closed {after:?} after its last word");
        assert!(
            after < FINALIZE_AFTER,
            "closed by the idle boundary instead"
        );
    }
}
