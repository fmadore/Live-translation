//! Gemini 3.5 Transcribe Live: same-language subtitles over the same `BidiGenerateContent`
//! socket the translate client uses, with the same 16 kHz PCM frame and the same API key.
//!
//! Unlike Live Translate, the transcript *is* the audience caption, so it goes in the
//! accumulator's `translated` field and reaches the export through the ordinary caption
//! path — the arrangement Mistral already uses.

use std::time::Duration;

use anyhow::{Context, Result};
use tauri::AppHandle;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::handshake::client::Request;

use super::protocol::{
    RealtimeInputMessage, ServerMessage, TranscribeSetupMessage, AUDIO_STREAM_END,
};
use crate::realtime::{
    emit_caption, MessageControl, MessageOutcome, RealtimeProtocol, TurnAccumulator,
};
use crate::types::Origin;

pub const DEFAULT_TRANSCRIBE_MODEL: &str = "gemini-3.5-transcribe-live";

/// Safety net, not the normal path: the server finalizes a segment when the speaker pauses,
/// and this only fires if an interim hypothesis is left stranded with no final behind it.
/// Deliberately longer than Mistral's 900 ms so it never races the server's own
/// finalization and splits one utterance across two transcript lines.
const FINALIZE_AFTER: Duration = Duration::from_secs(3);

#[derive(Clone)]
pub struct GeminiTranscribeConfig {
    pub api_key: String,
    pub model: String,
    pub host: String,
    pub origin: Origin,
}

impl GeminiTranscribeConfig {
    fn ws_url(&self) -> String {
        format!(
            "wss://{}/ws/google.ai.generativelanguage.v1beta.GenerativeService.BidiGenerateContent?key={}",
            self.host, self.api_key
        )
    }
}

impl RealtimeProtocol for GeminiTranscribeConfig {
    const NAME: &'static str = "Gemini Transcribe";

    fn origin(&self) -> Origin {
        self.origin
    }

    fn connect_request(&self) -> Result<Request> {
        self.ws_url()
            .into_client_request()
            .context("failed to build Gemini Transcribe request")
    }

    fn setup_json(&self) -> Result<String> {
        Ok(serde_json::to_string(
            &TranscribeSetupMessage::live_transcribe(&self.model),
        )?)
    }

    fn audio_json(&self, base64_pcm: String) -> Result<String> {
        Ok(serde_json::to_string(&RealtimeInputMessage::pcm16(
            base64_pcm,
        ))?)
    }

    fn wait_for_setup_complete(&self) -> bool {
        true
    }

    fn closing_json(&self) -> Result<Vec<String>> {
        Ok(vec![AUDIO_STREAM_END.to_string()])
    }

    fn handle_message(
        &mut self,
        app: &AppHandle,
        text: &str,
        acc: &mut TurnAccumulator,
    ) -> MessageOutcome {
        let msg: ServerMessage = match serde_json::from_str(text) {
            Ok(m) => m,
            Err(e) => {
                tracing::debug!("unparsed server message: {e} :: {text}");
                return MessageOutcome::default();
            }
        };

        if msg.setup_complete.is_some() {
            tracing::debug!(origin = ?self.origin, "Gemini Transcribe setup complete; streaming audio");
            return MessageOutcome::setup_complete();
        }
        // Live transcription sessions cap at 10 minutes, so a long room session reconnects
        // several times an hour. `goAway` gets us moving before the socket actually drops.
        if msg.go_away.is_some() {
            return MessageOutcome::control(MessageControl::Reconnect);
        }
        if let Some(error) = msg.error {
            return MessageOutcome::control(MessageControl::Fatal(format!(
                "Gemini Transcribe error: {error}"
            )));
        }

        let Some(content) = msg.server_content else {
            return MessageOutcome::default();
        };

        // Both fields describe the *same* segment, so each one replaces the buffer rather
        // than extending it. Appending would repeat every revised hypothesis on screen.
        if let Some(interim) = content
            .interim_input_transcription
            .as_ref()
            .filter(|t| !t.text.is_empty())
        {
            acc.translated.clear();
            acc.translated.push_str(&interim.text);
            emit_caption(app, self.origin, acc, false);
            return MessageOutcome::activity();
        }

        // A finalized segment is the model's authoritative reading of it, and ends the turn:
        // one segment is one transcript line, broken where the speaker paused.
        if let Some(final_text) = content
            .input_transcription
            .as_ref()
            .filter(|t| !t.text.is_empty())
        {
            acc.translated.clear();
            acc.translated.push_str(&final_text.text);
        }
        // `generationComplete` is the marker the server actually sends; `turnComplete` is
        // Live Translate's and is accepted here only for symmetry. It matters for the segment
        // that yields no final text — SMART mode dropping an all-filler utterance — where
        // without it the last speculative hypothesis would sit on screen until the idle timer
        // committed it to the transcript as though it had been confirmed.
        let closed = content.input_transcription.is_some()
            || content.generation_complete.unwrap_or(false)
            || content.turn_complete.unwrap_or(false);
        if closed {
            if !acc.is_empty() {
                emit_caption(app, self.origin, acc, true);
                acc.next_turn();
            }
            return MessageOutcome::activity();
        }

        MessageOutcome::default()
    }

    fn finalize_after(&self) -> Option<Duration> {
        Some(FINALIZE_AFTER)
    }
}

/// Opt-in probe against the real endpoint. Ignored by default, so CI never runs it and it
/// never bills anyone by accident:
///
/// ```text
/// GEMINI_API_KEY=... cargo test -p live-translation --lib live_probe -- --ignored --nocapture
/// ```
///
/// Serialization tests pin this client to the *documented* shape. They cannot tell us the
/// documentation is right — and it demonstrably is not everywhere, since the live translate
/// guide still shows the transcription sidecars nested under `generationConfig`, a placement
/// the server rejects outright. This closes that gap by putting the production setup message,
/// audio frame and response types in front of Google's servers with the bundled rehearsal
/// recording as input. Re-run it before an event, as `docs/gemini-live-api.md` asks.
#[cfg(test)]
mod live_probe {
    use std::time::Duration;

    use base64::Engine;
    use futures_util::{SinkExt, StreamExt};
    use tokio_tungstenite::connect_async;
    use tokio_tungstenite::tungstenite::Message;

    use super::*;
    use crate::audio::fixture::load_fixture;
    use crate::audio::{chunk_samples, resample::f32_to_pcm16_le};
    use crate::gemini::protocol::AUDIO_STREAM_END;
    use crate::gemini::DEFAULT_HOST;

    /// The Live API replies in **binary** frames carrying UTF-8 JSON, not text frames —
    /// undocumented, and the reason this probe reads them the way `realtime.rs` does rather
    /// than matching on `Message::Text`. A text-only reader sees the socket go silent.
    fn frame_json(message: &Message) -> Option<String> {
        match message {
            Message::Text(text) => Some(text.to_string()),
            Message::Binary(bytes) => String::from_utf8(bytes.to_vec()).ok(),
            _ => None,
        }
    }

    const FIXTURE: &str = "resources/fixtures/rehearsal-en.wav";
    const RATE: u32 = 16_000;
    /// The recording is about twenty seconds; the rest is head-room for the closing drain.
    const DEADLINE: Duration = Duration::from_secs(60);

    #[tokio::test]
    #[ignore = "streams the bundled fixture to Google and bills the key; run with --ignored"]
    async fn the_documented_shape_is_the_shape_the_server_accepts() {
        let _ = dotenvy::dotenv();
        let api_key = std::env::var("GEMINI_API_KEY")
            .expect("set GEMINI_API_KEY, or copy .env.example to .env at the repo root");

        let config = GeminiTranscribeConfig {
            api_key,
            model: DEFAULT_TRANSCRIBE_MODEL.to_string(),
            host: DEFAULT_HOST.to_string(),
            origin: Origin::Microphone,
        };

        let samples = load_fixture(std::path::Path::new(FIXTURE))
            .expect("the bundled rehearsal fixture should load");
        let per_chunk = chunk_samples(RATE);
        println!(
            "probing {} with {:.1}s of bundled speech",
            config.model,
            samples.len() as f32 / RATE as f32
        );

        let (socket, _) = connect_async(config.connect_request().unwrap())
            .await
            .expect("the Live API should accept the connection");
        let (mut write, mut read) = socket.split();

        // Failure here is the one this probe exists for: a setup field in the wrong place
        // comes back as `Unknown name ... Cannot find field`, not as a bad transcript.
        write
            .send(Message::Text(config.setup_json().unwrap().into()))
            .await
            .unwrap();
        let deadline = tokio::time::Instant::now() + DEADLINE;
        loop {
            let message = tokio::time::timeout_at(deadline, read.next())
                .await
                .expect("timed out waiting for setupComplete")
                .expect("the server closed before confirming setup")
                .unwrap();
            if let Message::Close(frame) = &message {
                panic!("the server rejected setup: {frame:?}");
            }
            if let Some(json) = frame_json(&message) {
                let parsed: ServerMessage = serde_json::from_str(&json)
                    .unwrap_or_else(|e| panic!("unparsed setup reply: {e} :: {json}"));
                assert!(
                    parsed.error.is_none(),
                    "the server rejected setup: {:?}",
                    parsed.error
                );
                if parsed.setup_complete.is_some() {
                    println!("setup accepted");
                    break;
                }
            }
        }

        // Pace the fixture at wall-clock speed. Dumping twenty seconds of audio at once is a
        // different thing from a live room, and this probe is only worth what its realism is.
        let sender = tokio::spawn(async move {
            let mut pcm = Vec::new();
            for chunk in samples.chunks(per_chunk) {
                pcm.clear();
                f32_to_pcm16_le(chunk, &mut pcm);
                let data = base64::engine::general_purpose::STANDARD.encode(&pcm);
                if write
                    .send(Message::Text(config.audio_json(data).unwrap().into()))
                    .await
                    .is_err()
                {
                    return write;
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            let _ = write.send(Message::Text(AUDIO_STREAM_END.into())).await;
            write
        });

        let mut interim = Vec::new();
        let mut finals: Vec<String> = Vec::new();
        let mut closes = 0usize;
        while let Ok(Some(Ok(message))) = tokio::time::timeout_at(deadline, read.next()).await {
            let Some(json) = frame_json(&message) else {
                continue;
            };
            let parsed: ServerMessage = serde_json::from_str(&json)
                .unwrap_or_else(|e| panic!("unparsed server message: {e} :: {json}"));
            assert!(parsed.error.is_none(), "provider error: {:?}", parsed.error);
            let Some(content) = parsed.server_content else {
                continue;
            };
            if let Some(t) = content.interim_input_transcription {
                println!("  interim: {}", t.text);
                interim.push(t.text);
            }
            if let Some(t) = content.input_transcription {
                println!("  FINAL:   {}", t.text);
                finals.push(t.text);
            }
            if content.generation_complete.unwrap_or(false) {
                println!("  [generationComplete]");
                closes += 1;
            }
            // The fixture is one short passage; one finalized segment is enough to prove the
            // field names, and waiting for the socket to close would only cost wall clock.
            if !finals.is_empty() && sender.is_finished() {
                break;
            }
        }

        assert!(
            !finals.iter().any(String::is_empty) && !finals.is_empty(),
            "expected at least one non-empty inputTranscription; interim seen: {}",
            interim.len()
        );
        assert!(
            closes > 0,
            "expected generationComplete to close a segment; the client relies on it to \
             finalize a segment that yields no transcript"
        );
        // Interim is a revision of the segment in progress, never an increment of it. If this
        // ever stops holding, the client's replace-don't-append rule is wrong.
        assert!(
            interim.len() > finals.len(),
            "expected many speculative updates per finalized segment, got {} and {}",
            interim.len(),
            finals.len()
        );
        println!(
            "ok — {} interim update(s), {} finalized segment(s), {} segment close(s)",
            interim.len(),
            finals.len(),
            closes
        );
    }
}
