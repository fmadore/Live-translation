//! Opt-in check of the input noise filter against the real endpoint. Ignored by default, so CI
//! never runs it and it never bills anyone by accident. It takes the key the app saved, in
//! process, and never prints it:
//!
//! ```text
//! cargo test --manifest-path src-tauri/Cargo.toml --lib noise_reduction_by_value -- --ignored --nocapture
//! ```
//!
//! Each run opens one translation session per `noise_reduction` value, sends the production
//! `session.update`, prints the server's `session.updated`, then streams the bundled French
//! rehearsal recording at the pace of speech and prints the source transcript and the English
//! translation. One clean studio recording shows that the server takes each value and whether
//! the words change; it says nothing about a reverberant room.

use std::time::Duration;

use base64::Engine;
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

use super::protocol::{InputAudioAppend, NoiseReduction, SessionUpdate};
use super::{
    DEFAULT_OPENAI_HOST, DEFAULT_OPENAI_TRANSCRIBE_MODEL, DEFAULT_OPENAI_TRANSLATE_MODEL,
    OpenAiConfig,
};
use crate::audio::fixture::load_fixture;
use crate::audio::resample::{Resampler, f32_to_pcm16_le};
use crate::realtime::RealtimeProtocol;
use crate::types::Origin;

const FIXTURE: &str = "resources/fixtures/rehearsal-fr.wav";
const FIXTURE_RATE: u32 = 16_000;
const RATE: u32 = 24_000;
/// About twenty seconds of speech, then the close drain.
const DEADLINE: Duration = Duration::from_secs(60);

fn frame_json(message: &Message) -> Option<String> {
    match message {
        Message::Text(text) => Some(text.to_string()),
        Message::Binary(bytes) => String::from_utf8(bytes.to_vec()).ok(),
        _ => None,
    }
}

struct Outcome {
    updated: serde_json::Value,
    source: String,
    translation: String,
}

async fn translate_fixture(noise_reduction: Option<NoiseReduction>) -> anyhow::Result<Outcome> {
    let config = OpenAiConfig {
        api_key: crate::secrets::resolve_api_key(crate::types::Provider::OpenAi)?,
        model: DEFAULT_OPENAI_TRANSLATE_MODEL.to_string(),
        transcribe_model: DEFAULT_OPENAI_TRANSCRIBE_MODEL.to_string(),
        host: DEFAULT_OPENAI_HOST.to_string(),
        target_language_code: "en".to_string(),
        origin: Origin::Microphone,
    };
    let (socket, _) = tokio::time::timeout(
        Duration::from_secs(20),
        connect_async(config.connect_request()?),
    )
    .await??;
    let (mut write, mut read) = socket.split();
    let update = SessionUpdate::translate(
        &config.target_language_code,
        &config.transcribe_model,
        noise_reduction,
    );
    write
        .send(Message::Text(serde_json::to_string(&update)?.into()))
        .await?;

    let deadline = tokio::time::Instant::now() + DEADLINE;
    let updated = loop {
        let message = tokio::time::timeout_at(deadline, read.next())
            .await?
            .ok_or_else(|| anyhow::anyhow!("closed before session.updated"))??;
        let Some(json) = frame_json(&message) else {
            continue;
        };
        let event: serde_json::Value = serde_json::from_str(&json)?;
        match event["type"].as_str() {
            Some("session.updated") => break event,
            Some("error") => anyhow::bail!("rejected: {}", event["error"]),
            _ => {}
        }
    };

    let samples = load_fixture(std::path::Path::new(FIXTURE))?;
    let mut resampled = Vec::new();
    Resampler::new(FIXTURE_RATE, RATE).process(&samples, &mut resampled);
    let sender = tokio::spawn(async move {
        let mut pcm = Vec::new();
        for chunk in resampled.chunks(crate::audio::chunk_samples(RATE)) {
            pcm.clear();
            f32_to_pcm16_le(chunk, &mut pcm);
            let data = base64::engine::general_purpose::STANDARD.encode(&pcm);
            let frame = serde_json::to_string(&InputAudioAppend::pcm16(data)).unwrap();
            if write.send(Message::Text(frame.into())).await.is_err() {
                return write;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        let _ = write
            .send(Message::Text(r#"{"type":"session.close"}"#.into()))
            .await;
        write
    });

    let mut source = String::new();
    let mut translation = String::new();
    while let Ok(Some(Ok(message))) = tokio::time::timeout_at(deadline, read.next()).await {
        let Some(json) = frame_json(&message) else {
            continue;
        };
        let event: serde_json::Value = serde_json::from_str(&json)?;
        let kind = event["type"].as_str().unwrap_or_default();
        if kind == "error" {
            anyhow::bail!("provider error mid-stream: {}", event["error"]);
        }
        if kind == "session.closed" {
            break;
        }
        if let Some(delta) = event["delta"].as_str() {
            if kind.ends_with("input_transcript.delta") {
                source.push_str(delta);
            } else if kind.ends_with("output_transcript.delta") {
                translation.push_str(delta);
            }
        }
    }
    drop(sender.await?);
    Ok(Outcome {
        updated,
        source,
        translation,
    })
}

#[tokio::test]
#[ignore = "streams the bundled fixture to OpenAI three times and bills the key; run with --ignored"]
async fn noise_reduction_by_value() -> anyhow::Result<()> {
    for (label, value) in [
        ("near_field", Some(NoiseReduction::NEAR_FIELD)),
        ("far_field", Some(NoiseReduction::FAR_FIELD)),
        ("null", None),
    ] {
        let outcome = translate_fixture(value).await?;
        println!("=== noise_reduction: {label}");
        println!("session.updated: {}", outcome.updated);
        println!("source:      {}", outcome.source.trim());
        println!("translation: {}", outcome.translation.trim());
        let echoed = &outcome.updated["session"]["audio"]["input"]["noise_reduction"];
        match value {
            Some(sent) => assert_eq!(echoed["type"], serde_json::to_value(sent)?["type"]),
            None => assert!(echoed.is_null(), "expected the filter off, got {echoed}"),
        }
        assert!(
            !outcome.translation.trim().is_empty(),
            "no translation came back with {label}"
        );
    }
    Ok(())
}
