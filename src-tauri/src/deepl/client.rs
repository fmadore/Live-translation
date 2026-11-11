//! DeepL Voice API: speech in, auto-detected, and a translation into the caption language out.
//! See `docs/deepl-voice-api.md`.
//!
//! Unlike the other realtime engines, a connection starts over REST. `prepare` asks for a
//! session (or resumes the one whose socket dropped) and gets a single-use streaming URL and
//! token; the socket then opens on them and audio follows at once, with no setup message.
//!
//! Which session the next socket belongs to:
//! - a socket that dropped resumes its session with the latest token (`GET`), falling back to a
//!   new session when DeepL says that one is gone;
//! - after DeepL's `error`, its `end_of_stream`, or our `end_of_source_media` (every Pause and
//!   Stop), that session is over, and the next socket gets a new one (`POST`).
//!
//! Captions. `text` is the target transcript, `source` the source transcript for the
//! operator's monitor. A turn holds the target text DeepL has concluded since it began, then
//! the current tentative text, which each update replaces. Each target update is an interim,
//! and counts as activity for the idle boundary; source updates are interims that do not. A
//! turn closes as final:
//! - when DeepL concludes target text that ends a sentence: the caption is the concluded text
//!   alone, and any tentative text, the next sentence's start, opens the next turn;
//! - otherwise at the idle boundary (`FINALIZE_AFTER` without target text), held while DeepL
//!   still has tentative text pending, up to the runner's `MAX_HOLD`;
//! - at `end_of_stream`.
//!
//! A turn closed on a timer may have shown tentative text that DeepL concludes later. Target
//! segments that end no later than what a final caption already showed are dropped, so it is
//! not printed twice (`shown_until`). Segment times are trusted within one connection only.

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use anyhow::{Context, Result};
use reqwest::Url;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::handshake::client::Request;

use super::protocol::{
    END_OF_SOURCE_MEDIA, Segment, ServerMessage, SessionRequest, StreamError, TranscriptUpdate,
    deepl_target_code, source_media_chunk,
};
use super::session::{self, Refusal};
use crate::audio::AudioChunk;
use crate::realtime::{
    CaptionUpdate, Keepalive, MessageControl, MessageOutcome, PrepareError, RealtimeProtocol,
    TurnAccumulator, Verdict, classify, ends_sentence, parse_or_log,
};
use crate::types::{Origin, TargetLanguage};

pub const DEFAULT_DEEPL_HOST: &str = "api.deepl.com";
/// No new target text for this long closes the turn, unless DeepL still holds tentative text.
/// Most turns close earlier, on a concluded sentence.
const FINALIZE_AFTER: Duration = Duration::from_millis(1_000);
/// DeepL ends a session after 30 seconds without audio. Silence goes out well before that.
const KEEPALIVE_AFTER: Duration = Duration::from_secs(10);
/// 100 ms of 16 kHz mono PCM16, like every chunk capture sends.
const SILENCE_BYTES: usize = 3_200;

#[derive(Clone)]
pub struct DeepLConfig {
    pub api_key: String,
    /// `https://api.deepl.com`, or a loopback server in the tests.
    pub api_base: String,
    pub target: TargetLanguage,
    pub origin: Origin,
}

impl DeepLConfig {
    /// The protocol the realtime runner drives for this source, with no session yet.
    pub fn into_client(self) -> DeepLClient {
        DeepLClient {
            config: self,
            http: None,
            streaming_url: None,
            token: None,
            resumable: AtomicBool::new(false),
            turn: Turn::default(),
        }
    }
}

/// One DeepL source: its configuration, its session, and the turn being captioned.
pub struct DeepLClient {
    config: DeepLConfig,
    /// Built on the first connection and kept for every later one.
    http: Option<reqwest::Client>,
    /// Where the next socket connects: from the latest session answer.
    streaming_url: Option<String>,
    /// The latest token DeepL issued: it opens the next socket, and afterwards is the one that
    /// may resume that socket's session.
    token: Option<String>,
    /// Whether the session behind `token` can still be resumed. Atomic only because
    /// `closing_json`, which ends the session, takes `&self`.
    resumable: AtomicBool,
    turn: Turn,
}

/// The open turn as DeepL has described it.
#[derive(Default)]
struct Turn {
    /// The accumulator's turn this state belongs to. When the runner has moved on without us
    /// (the idle boundary, or a connection that ended), the turn we held went out as final.
    id: Option<u64>,
    concluded: String,
    tentative: String,
    /// The latest end time of the concluded and the tentative target segments, in ms.
    concluded_end: f64,
    tentative_end: f64,
    source_concluded: String,
    source_tentative: String,
    /// Target segments ending no later than this have already been shown in a final caption.
    shown_until: Option<f64>,
}

impl Turn {
    /// Catch up with the runner. A turn it finalized itself showed everything we held, the
    /// tentative text included: remember how far that went, and start afresh.
    fn follow(&mut self, acc: &TurnAccumulator) {
        if self.id == Some(acc.id) {
            return;
        }
        if self.id.is_some() {
            let shown = self.concluded_end.max(self.tentative_end);
            if shown > 0.0 {
                self.shown_until = Some(shown);
            }
            self.concluded.clear();
            self.tentative.clear();
            self.concluded_end = 0.0;
            self.tentative_end = 0.0;
            self.source_concluded.clear();
            self.source_tentative.clear();
        }
        self.id = Some(acc.id);
    }

    /// Whether a target segment says something no final caption has shown yet. A segment
    /// without times is always kept: duplicated text is the lesser fault than lost text.
    fn is_new(&self, segment: &Segment) -> bool {
        segment.end_time <= 0.0
            || self
                .shown_until
                .is_none_or(|shown| segment.end_time > shown)
    }

    fn render(&self, acc: &mut TurnAccumulator) {
        acc.text = format!("{}{}", self.concluded, self.tentative)
            .trim_start()
            .to_string();
        acc.source = format!("{}{}", self.source_concluded, self.source_tentative)
            .trim_start()
            .to_string();
    }

    fn target(&mut self, update: &TranscriptUpdate, acc: &mut TurnAccumulator) -> MessageOutcome {
        let mut concluded = String::new();
        let mut concluded_end = self.concluded_end;
        for segment in update.concluded.iter().filter(|s| self.is_new(s)) {
            concluded.push_str(&segment.text);
            concluded_end = concluded_end.max(segment.end_time);
        }
        self.concluded_end = concluded_end;
        let new_segments: Vec<&Segment> =
            update.tentative.iter().filter(|s| self.is_new(s)).collect();
        let tentative: String = new_segments.iter().map(|s| s.text.as_str()).collect();
        self.tentative_end = new_segments.iter().map(|s| s.end_time).fold(0.0, f64::max);
        let changed = !concluded.is_empty() || tentative != self.tentative;
        self.concluded.push_str(&concluded);
        self.tentative = tentative;

        if !concluded.is_empty() && ends_sentence(&self.concluded) {
            // The sentence alone; the tentative text after it is the next turn's.
            acc.text = std::mem::take(&mut self.concluded).trim().to_string();
            acc.source = std::mem::take(&mut self.source_concluded)
                .trim()
                .to_string();
            if self.concluded_end > 0.0 {
                self.shown_until = Some(self.concluded_end);
            }
            self.concluded_end = 0.0;
            self.id = Some(acc.id + 1);
            return MessageOutcome::activity(CaptionUpdate::Final);
        }
        if !changed {
            return MessageOutcome::default();
        }
        self.render(acc);
        MessageOutcome::activity(CaptionUpdate::Interim)
    }

    fn source(&mut self, update: &TranscriptUpdate, acc: &mut TurnAccumulator) -> MessageOutcome {
        let concluded: String = update.concluded.iter().map(|s| s.text.as_str()).collect();
        let tentative: String = update.tentative.iter().map(|s| s.text.as_str()).collect();
        if concluded.is_empty() && tentative == self.source_tentative {
            return MessageOutcome::default();
        }
        self.source_concluded.push_str(&concluded);
        self.source_tentative = tentative;
        self.render(acc);
        // Not activity: the source leads the translation, and would close a turn before its
        // translation arrived.
        MessageOutcome::caption(CaptionUpdate::Interim)
    }
}

impl DeepLClient {
    fn http(&mut self) -> Result<reqwest::Client, PrepareError> {
        if let Some(http) = &self.http {
            return Ok(http.clone());
        }
        let http = session::client()
            .context("failed to start the DeepL HTTP client")
            .map_err(PrepareError::Failed)?;
        self.http = Some(http.clone());
        Ok(http)
    }

    fn accept(&mut self, ticket: super::protocol::Ticket) {
        self.streaming_url = Some(ticket.streaming_url);
        self.token = Some(ticket.token);
        self.resumable.store(true, Ordering::Relaxed);
    }

    /// The session is over: the next socket needs a new one.
    fn session_over(&self) {
        self.resumable.store(false, Ordering::Relaxed);
    }

    fn stream_error(&self, error: &StreamError) -> MessageOutcome {
        self.session_over();
        let message = error.error_message.as_deref().unwrap_or_default();
        let verdict = classify(error.error_code, &[], message);
        if verdict != Verdict::Fatal {
            let origin = self.config.origin;
            tracing::warn!(
                ?origin,
                error = %error.describe(),
                request = error.request_type.as_deref().unwrap_or("unknown"),
                "DeepL ended the session; requesting a new one"
            );
        }
        MessageOutcome::control(verdict.control(|| error.describe()))
    }
}

impl RealtimeProtocol for DeepLClient {
    const NAME: &'static str = "DeepL";

    fn origin(&self) -> Origin {
        self.config.origin
    }

    async fn prepare(&mut self) -> Result<(), PrepareError> {
        // A new connection: segment times start again, and the runner has finalized the turn
        // the last one left open.
        self.turn = Turn::default();
        let target = deepl_target_code(self.config.target).ok_or_else(|| {
            PrepareError::Malformed(anyhow::anyhow!(
                "DeepL does not translate into {}",
                self.config.target.bcp47()
            ))
        })?;
        let auth = session::auth_header(&self.config.api_key).map_err(PrepareError::Malformed)?;
        let http = self.http()?;
        let api_base = self.config.api_base.clone();

        if self.resumable.load(Ordering::Relaxed)
            && let Some(token) = self.token.clone()
        {
            match session::reconnect(&http, &api_base, &auth, &token).await {
                Ok(ticket) => {
                    tracing::info!(origin = ?self.config.origin, "DeepL session resumed");
                    self.accept(ticket);
                    return Ok(());
                }
                Err(refusal) if refusal.session_gone() => {
                    tracing::info!(origin = ?self.config.origin, ?refusal, "DeepL session could not be resumed; requesting a new one");
                }
                Err(refusal) => return Err(refusal.into_prepare_error()),
            }
        }
        self.session_over();
        let ticket = session::request(&http, &api_base, &auth, &SessionRequest::translate(target))
            .await
            .map_err(Refusal::into_prepare_error)?;
        self.accept(ticket);
        Ok(())
    }

    fn connect_request(&self) -> Result<Request> {
        let (Some(streaming_url), Some(token)) = (&self.streaming_url, &self.token) else {
            anyhow::bail!("no DeepL session to connect to");
        };
        let mut url =
            Url::parse(streaming_url).context("DeepL answered with an invalid streaming URL")?;
        // The token is a credential for the session: never over plain WebSocket, except to the
        // loopback servers of the tests and of a debug build's `DEEPL_API_HOST`.
        anyhow::ensure!(
            cfg!(debug_assertions) || url.scheme() == "wss",
            "DeepL answered with a streaming URL that is not wss"
        );
        url.query_pairs_mut().append_pair("token", token);
        url.as_str()
            .into_client_request()
            .context("failed to build DeepL request")
    }

    fn setup_json(&self) -> Result<Option<String>> {
        Ok(None)
    }

    fn audio_json(&self, base64_pcm: String) -> Result<String> {
        Ok(source_media_chunk(&base64_pcm))
    }

    fn keepalive(&self) -> Option<Keepalive> {
        Some(Keepalive {
            after: KEEPALIVE_AFTER,
            silence: AudioChunk {
                pcm_le: vec![0; SILENCE_BYTES],
            },
        })
    }

    /// Ends the session: DeepL concludes what is tentative, then sends `end_of_stream`.
    fn closing_json(&self) -> Result<Vec<String>> {
        self.session_over();
        Ok(vec![END_OF_SOURCE_MEDIA.to_string()])
    }

    fn handle_message(&mut self, text: &str, acc: &mut TurnAccumulator) -> MessageOutcome {
        let Some(message) = parse_or_log::<ServerMessage>(Self::NAME, text) else {
            return MessageOutcome::default();
        };
        self.turn.follow(acc);

        if let Some(error) = &message.error {
            return self.stream_error(error);
        }
        if let Some(update) = &message.target_transcript_update {
            return self.turn.target(update, acc);
        }
        if let Some(update) = &message.source_transcript_update {
            return self.turn.source(update, acc);
        }
        // The last message: the drain's end after `end_of_source_media`. Mid-stream the runner
        // moves the source to a new session.
        if message.end_of_stream.is_some() {
            self.session_over();
            self.turn.render(acc);
            self.turn = Turn::default();
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
        MessageOutcome::default()
    }

    fn finalize_after(&self) -> Option<Duration> {
        Some(FINALIZE_AFTER)
    }

    /// DeepL still has words it may revise: closing now would show them before it settles them.
    fn keeps_turn_open(&self, _acc: &TurnAccumulator) -> bool {
        !self.turn.tentative.is_empty()
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
