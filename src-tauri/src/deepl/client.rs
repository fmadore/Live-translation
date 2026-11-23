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
//!   alone, and any tentative text, the next sentence's start, opens the next turn: shown by
//!   the next target update even if that repeats it, drained for on a Pause or Stop, and
//!   closed as it is when the runner closes the turn itself (`before_finalize`);
//! - otherwise at the idle boundary (`FINALIZE_AFTER` without target text), held while DeepL
//!   still has tentative text pending, up to the runner's `MAX_HOLD`;
//! - at `end_of_stream`.
//!
//! A turn closed on a timer may have shown tentative text that DeepL concludes later. Segments
//! that end no later than what that final caption showed are dropped, so it is not printed
//! twice (`shown_until`), and each drop is logged with its times and length. Only the runner's
//! own closes need this: a turn closed on a concluded sentence showed concluded text alone,
//! which DeepL sends once. The filter lifts at the first concluded segment that ends later, so
//! times that are not strictly increasing cost at most the segments around one timer close.
//! Segment times are trusted within one connection only, and the source transcript is matched
//! against its own times the same way.

use std::time::Duration;

use anyhow::{Context, Result};
use reqwest::Url;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::handshake::client::Request;

use super::protocol::{
    END_OF_SOURCE_MEDIA, Segment, ServerMessage, SessionRequest, StreamError, TranscriptUpdate,
    deepl_target_code, source_media_chunk, source_media_frame,
};
use super::session::{self, Refusal};
use crate::realtime::{
    CaptionUpdate, Keepalive, MessageControl, MessageOutcome, PrepareError, RealtimeProtocol,
    TurnAccumulator, Verdict, classify, ends_sentence, idle_keepalive, parse_or_log,
};
use crate::types::{Origin, TargetLanguage};

pub const DEFAULT_DEEPL_HOST: &str = "api.deepl.com";
/// No new target text for this long closes the turn, unless DeepL still holds tentative text.
/// Most turns close earlier, on a concluded sentence.
const FINALIZE_AFTER: Duration = Duration::from_millis(1_000);
/// Whether a streaming URL may be plain `ws`. The token is a credential for the session, so a
/// release build refuses it; a debug build allows it for the loopback servers of the tests and
/// for whatever a developer's `DEEPL_API_HOST` hands out.
const ALLOW_PLAIN_WS: bool = cfg!(debug_assertions);

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
            resumable: false,
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
    /// Whether the session behind `token` can still be resumed.
    resumable: bool,
    turn: Turn,
}

/// The open turn as DeepL has described it.
#[derive(Default)]
struct Turn {
    /// The accumulator's turn this state belongs to. When the runner has moved on without us
    /// (the idle boundary, or a connection that ended), the turn we held went out as final.
    id: Option<u64>,
    target: Transcript,
    source: Transcript,
}

/// One transcript's part of the open turn: the text DeepL has concluded since the turn began,
/// then its current tentative text.
#[derive(Default)]
struct Transcript {
    concluded: String,
    tentative: String,
    /// The latest end time of the concluded and the tentative segments, in ms.
    concluded_end: f64,
    tentative_end: f64,
    /// Segments ending no later than this were shown in a final caption the runner closed.
    shown_until: Option<f64>,
    /// Where the next tentative text is assembled, kept to reuse its allocation.
    scratch: String,
}

/// What an update did to a transcript.
struct Change {
    /// It concluded new text.
    concluded: bool,
    /// Its text is not what it was.
    changed: bool,
}

impl Transcript {
    /// The runner closed the turn and showed all of this: remember how far, and start afresh.
    fn shown(&mut self) {
        let shown = self.concluded_end.max(self.tentative_end);
        if shown > 0.0 {
            self.shown_until = Some(shown);
        }
        self.concluded.clear();
        self.tentative.clear();
        self.concluded_end = 0.0;
        self.tentative_end = 0.0;
    }

    /// Whether a segment says something no final caption has shown yet. A segment without times
    /// is always kept: duplicated text is the lesser fault than lost text. So is everything once
    /// a concluded segment has ended past the final caption: DeepL has moved on from it.
    fn keep(
        &mut self,
        segment: &Segment,
        concluded: bool,
        transcript: &str,
        origin: Origin,
    ) -> bool {
        let Some(shown) = self.shown_until else {
            return true;
        };
        if segment.end_time <= 0.0 {
            return true;
        }
        if segment.end_time > shown {
            if concluded {
                self.shown_until = None;
            }
            return true;
        }
        // Times and length only: caption text does not go to the log.
        tracing::debug!(
            ?origin,
            transcript,
            concluded,
            end_ms = segment.end_time,
            shown_until_ms = shown,
            chars = segment.text.chars().count(),
            "DeepL segment already shown in a final caption; dropped"
        );
        false
    }

    fn apply(&mut self, update: &TranscriptUpdate, transcript: &str, origin: Origin) -> Change {
        let before = self.concluded.len();
        for segment in &update.concluded {
            if self.keep(segment, true, transcript, origin) {
                self.concluded.push_str(&segment.text);
                self.concluded_end = self.concluded_end.max(segment.end_time);
            }
        }
        let mut tentative = std::mem::take(&mut self.scratch);
        tentative.clear();
        let mut tentative_end = 0.0_f64;
        for segment in &update.tentative {
            if self.keep(segment, false, transcript, origin) {
                tentative.push_str(&segment.text);
                tentative_end = tentative_end.max(segment.end_time);
            }
        }
        let concluded = self.concluded.len() > before;
        let changed = concluded || tentative != self.tentative;
        self.scratch = std::mem::replace(&mut self.tentative, tentative);
        self.tentative_end = tentative_end;
        Change { concluded, changed }
    }

    /// Whether there is nothing to show.
    fn is_blank(&self) -> bool {
        self.concluded.trim().is_empty() && self.tentative.trim().is_empty()
    }

    /// The concluded text, then the tentative, without leading space, into `out`'s buffer.
    fn render_into(&self, out: &mut String) {
        out.clear();
        let concluded = self.concluded.trim_start();
        out.push_str(concluded);
        out.push_str(if concluded.is_empty() {
            self.tentative.trim_start()
        } else {
            &self.tentative
        });
    }

    /// The concluded text alone into `out`, for a closed sentence; the tentative text stays.
    fn take_concluded_into(&mut self, out: &mut String) {
        out.clear();
        out.push_str(self.concluded.trim());
        self.concluded.clear();
        self.concluded_end = 0.0;
    }
}

impl Turn {
    /// Catch up with the runner. A turn it finalized itself showed everything we held, the
    /// tentative text included: remember how far that went, and start afresh.
    fn follow(&mut self, acc: &TurnAccumulator) {
        if self.id == Some(acc.id) {
            return;
        }
        if self.id.is_some() {
            self.target.shown();
            self.source.shown();
        }
        self.id = Some(acc.id);
    }

    fn render(&self, acc: &mut TurnAccumulator) {
        self.target.render_into(&mut acc.text);
        self.source.render_into(&mut acc.source);
    }

    fn target(
        &mut self,
        update: &TranscriptUpdate,
        acc: &mut TurnAccumulator,
        origin: Origin,
    ) -> MessageOutcome {
        let change = self.target.apply(update, "target", origin);
        if change.concluded && ends_sentence(&self.target.concluded) {
            // The sentence alone; the tentative text after it is the next turn's. Concluded
            // segments arrive once, so nothing of this line can come again: no `shown_until`.
            self.target.take_concluded_into(&mut acc.text);
            self.source.take_concluded_into(&mut acc.source);
            self.id = Some(acc.id + 1);
            return MessageOutcome::activity(CaptionUpdate::Final);
        }
        // The tentative start a closed sentence left behind is not on screen yet: an update
        // that repeats it still shows it.
        let unshown = acc.text.is_empty() && !self.target.is_blank();
        if !change.changed && !unshown {
            return MessageOutcome::default();
        }
        self.render(acc);
        MessageOutcome::activity(CaptionUpdate::Interim)
    }

    fn source(
        &mut self,
        update: &TranscriptUpdate,
        acc: &mut TurnAccumulator,
        origin: Origin,
    ) -> MessageOutcome {
        if !self.source.apply(update, "source", origin).changed {
            return MessageOutcome::default();
        }
        self.render(acc);
        // Not activity: the source leads the translation, and would close a turn before its
        // translation arrived.
        MessageOutcome::caption(CaptionUpdate::Interim)
    }

    /// DeepL holds tentative target text in this turn that it may still revise or conclude.
    fn pending(&self, acc: &TurnAccumulator) -> bool {
        self.id == Some(acc.id) && !self.target.tentative.trim().is_empty()
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
        self.resumable = true;
    }

    /// The session is over: the next socket needs a new one.
    fn session_over(&mut self) {
        self.resumable = false;
    }

    fn stream_error(&mut self, error: &StreamError) -> MessageOutcome {
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

        if self.resumable
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
        check_streaming_scheme(&url, ALLOW_PLAIN_WS)?;
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

    fn audio_frame(&self, pcm_le: &[u8]) -> Result<String> {
        Ok(source_media_frame(pcm_le))
    }

    /// DeepL ends a session after 30 seconds without audio. Silence goes out well before that.
    fn keepalive(&self) -> Option<Keepalive> {
        Some(idle_keepalive())
    }

    /// Ends the session: DeepL concludes what is tentative, then sends `end_of_stream`.
    fn closing_json(&mut self) -> Result<Vec<String>> {
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
            return self.turn.target(update, acc, self.config.origin);
        }
        if let Some(update) = &message.source_transcript_update {
            return self.turn.source(update, acc, self.config.origin);
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

    /// DeepL still has words it may revise: closing now would show them before it settles them,
    /// and a Pause or Stop drains for them, as `end_of_source_media` makes DeepL conclude them.
    fn keeps_turn_open(&self, acc: &TurnAccumulator) -> bool {
        self.turn.pending(acc)
    }

    /// The tentative start a concluded sentence left behind is not in the accumulator yet
    /// (`Turn::target`). When the runner closes the turn on its own (DeepL sent nothing more
    /// before the hold ran out, or the connection or source ended), it goes out with the rest of
    /// the turn rather than the runner closing an empty one. Otherwise the accumulator already
    /// holds the turn as rendered, and rendering it again changes nothing.
    fn before_finalize(&mut self, acc: &mut TurnAccumulator) {
        if self.turn.id == Some(acc.id) && !self.turn.target.is_blank() {
            self.turn.render(acc);
        }
    }
}

/// The token is a credential for the session: it goes over `wss`, or over plain `ws` only where
/// `allow_plain` says so (`ALLOW_PLAIN_WS`). No other scheme opens a socket.
fn check_streaming_scheme(url: &Url, allow_plain: bool) -> Result<()> {
    match url.scheme() {
        "wss" => Ok(()),
        "ws" if allow_plain => Ok(()),
        scheme => anyhow::bail!("DeepL answered with a streaming URL that is not wss ({scheme})"),
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
