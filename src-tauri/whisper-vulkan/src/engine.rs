//! What the app (`src/whisper/`) and the Vulkan helper (`whisper-vulkan`) share: how a window is
//! decoded, what whisper.cpp reports of each segment, and the messages between the two
//! processes. Compiled into both from this one file (the app includes it by path), so a window
//! decoded on the graphics processor is asked for exactly as one decoded on the processor.
//!
//! The messages are frames: a 32-bit little-endian length, then that many bytes. A request or a
//! response is JSON; a `Decode` request is followed by one more frame holding the window's
//! samples as 32-bit little-endian floats.

use std::io::{self, Read, Write};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use whisper_rs::{FullParams, SamplingStrategy, WhisperSegment, WhisperState};

/// Far more than a window can hold (28 s of samples is under 2 MiB): anything larger is a
/// broken stream, not a request.
const MAX_FRAME: usize = 64 << 20;

/// How one window is decoded. `language` is the language it was spoken in, or `None` to detect
/// it; `translate` turns the task from transcription into translation into English; `prompt` is
/// the session's names and terms as tokens, empty when there are none.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Decode {
    pub language: Option<String>,
    pub translate: bool,
    pub threads: i32,
    pub prompt: Vec<i32>,
}

/// whisper.cpp's parameters for `decode`.
pub fn params(decode: &Decode) -> FullParams<'_, '_> {
    decode_params(
        decode.language.as_deref(),
        decode.translate,
        decode.threads,
        &decode.prompt,
    )
}

/// How one window is decoded. Translating still starts from the spoken language — it is what
/// Whisper translates from — so detection and the operator's choice mean the same either way.
pub fn decode_params<'a, 'b>(
    spoken: Option<&'a str>,
    translate: bool,
    threads: i32,
    prompt: &'b [i32],
) -> FullParams<'a, 'b> {
    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
    params.set_language(spoken);
    // `no_context` below drops the text of earlier windows, not this: whisper.cpp clears its
    // history first, then puts the prompt in front of every decoding pass of the window.
    if !prompt.is_empty() {
        params.set_tokens(prompt);
    }
    // `detect_language=true` asks whisper.cpp to return after detection. Leaving it false with
    // language=None performs detection followed by the task.
    params.set_translate(translate);
    params.set_no_context(true);
    params.set_n_threads(threads);
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_timestamps(false);
    params.set_print_special(false);
    params.set_suppress_blank(true);
    params.set_suppress_nst(true);
    params
}

/// One segment as whisper.cpp reported it, before the app decides whether to caption it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RawSegment {
    /// Decoded leniently: a token split mid-character must not end the source.
    pub text: String,
    /// Centiseconds from the start of the window, as whisper.cpp counts them.
    pub start_cs: i64,
    pub end_cs: i64,
    /// One value for the whole window: whisper.cpp asks once, before decoding.
    pub no_speech: f32,
    /// Mean log-probability of the segment's tokens.
    pub mean_logprob: f32,
}

/// Every segment of the window `state` last decoded. A segment whisper.cpp cannot describe is
/// skipped, never fatal to the source.
pub fn segments(state: &WhisperState) -> Vec<RawSegment> {
    state.as_iter().filter_map(|s| raw_segment(&s)).collect()
}

fn raw_segment(segment: &WhisperSegment<'_>) -> Option<RawSegment> {
    // Lossy: a token boundary inside a multi-byte character must cost one character, not the
    // rest of the meeting's transcript.
    let text = segment.to_str_lossy().ok()?.into_owned();
    let tokens = segment.n_tokens();
    let mean_logprob = if tokens > 0 {
        (0..tokens)
            .filter_map(|i| segment.get_token(i))
            .map(|token| token.token_data().plog)
            .sum::<f32>()
            / tokens as f32
    } else {
        0.0
    };
    Some(RawSegment {
        text,
        start_cs: segment.start_timestamp(),
        end_cs: segment.end_timestamp(),
        no_speech: segment.no_speech_probability(),
        mean_logprob,
    })
}

/// What the app asks of the helper.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Request {
    /// Load the model at `path`, provided it is `bytes` long with this SHA-256, on the graphics
    /// processor the helper chooses. `allow_software` lets it choose a software renderer, which
    /// only the tests want.
    #[serde(rename_all = "camelCase")]
    Load {
        path: PathBuf,
        bytes: u64,
        sha256: String,
        allow_software: bool,
    },
    /// The text as the model's tokens, for the names-and-terms prompt.
    Tokenize { text: String },
    /// Decode the samples in the frame that follows.
    Decode(Decode),
}

/// The helper's answer to each request, in order.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Response {
    /// The model is loaded on `device`, as its driver names it.
    #[serde(rename_all = "camelCase")]
    Loaded { device: String, text_context: i32 },
    /// `None` when the model's tokenizer refused the text.
    Tokens { tokens: Option<Vec<i32>> },
    /// The window's segments, and the language whisper.cpp decoded it in.
    Decoded {
        segments: Vec<RawSegment>,
        language: i32,
    },
    /// The request failed; the helper is still running and can take another.
    Failed { message: String },
}

fn invalid(message: impl std::fmt::Display) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.to_string())
}

/// Write one frame and flush it: the other side is waiting for it.
pub fn write_frame(output: &mut impl Write, bytes: &[u8]) -> io::Result<()> {
    let length = u32::try_from(bytes.len()).map_err(|_| invalid("frame too large"))?;
    output.write_all(&length.to_le_bytes())?;
    output.write_all(bytes)?;
    output.flush()
}

/// The next frame, or `None` once the other side has closed the stream between frames.
pub fn read_frame(input: &mut impl Read) -> io::Result<Option<Vec<u8>>> {
    let mut length = [0u8; 4];
    match input.read_exact(&mut length) {
        Ok(()) => {}
        Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(error) => return Err(error),
    }
    let length = usize::try_from(u32::from_le_bytes(length)).unwrap_or(usize::MAX);
    if length > MAX_FRAME {
        return Err(invalid("frame too large"));
    }
    let mut bytes = vec![0; length];
    input.read_exact(&mut bytes)?;
    Ok(Some(bytes))
}

/// Write `message` as a JSON frame.
pub fn send<T: Serialize>(output: &mut impl Write, message: &T) -> io::Result<()> {
    write_frame(output, &serde_json::to_vec(message).map_err(invalid)?)
}

/// Read a JSON frame as `T`; `None` once the stream has ended.
pub fn receive<T: for<'de> Deserialize<'de>>(input: &mut impl Read) -> io::Result<Option<T>> {
    read_frame(input)?
        .map(|bytes| serde_json::from_slice(&bytes).map_err(invalid))
        .transpose()
}

pub fn samples_to_bytes(samples: &[f32]) -> Vec<u8> {
    samples.iter().flat_map(|s| s.to_le_bytes()).collect()
}

pub fn bytes_to_samples(bytes: &[u8]) -> io::Result<Vec<f32>> {
    let (samples, rest) = bytes.as_chunks::<4>();
    if !rest.is_empty() {
        return Err(invalid("a window's samples are 4 bytes each"));
    }
    Ok(samples.iter().map(|b| f32::from_le_bytes(*b)).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frames_round_trip_and_a_closed_stream_ends_between_them() {
        let mut stream = Vec::new();
        let request = Request::Decode(Decode {
            language: Some("fr".into()),
            translate: true,
            threads: 4,
            prompt: vec![50, 51],
        });
        send(&mut stream, &request).unwrap();
        write_frame(&mut stream, &samples_to_bytes(&[0.25, -1.0])).unwrap();
        let mut reader = stream.as_slice();
        assert_eq!(receive::<Request>(&mut reader).unwrap(), Some(request));
        let samples = read_frame(&mut reader).unwrap().unwrap();
        assert_eq!(bytes_to_samples(&samples).unwrap(), [0.25, -1.0]);
        assert_eq!(read_frame(&mut reader).unwrap(), None);
    }

    #[test]
    fn broken_streams_are_errors_not_requests() {
        // A length far beyond any window.
        let huge = u32::MAX.to_le_bytes();
        assert!(read_frame(&mut huge.as_slice()).is_err());
        // A frame cut short.
        let mut cut = Vec::new();
        write_frame(&mut cut, b"abcdef").unwrap();
        cut.truncate(7);
        assert!(read_frame(&mut cut.as_slice()).is_err());
        // Samples that are not whole floats.
        assert!(bytes_to_samples(&[0, 0, 0]).is_err());
    }

    #[test]
    fn the_messages_read_as_both_sides_write_them() {
        let load = Request::Load {
            path: PathBuf::from("réunion/ggml-base-q5_1.bin"),
            bytes: 3,
            sha256: "abc".into(),
            allow_software: false,
        };
        let json = serde_json::to_string(&load).unwrap();
        assert!(json.contains(r#""type":"load""#), "{json}");
        assert!(json.contains(r#""allowSoftware":false"#), "{json}");
        let decoded: Response =
            serde_json::from_str(r#"{"type":"failed","message":"out of memory"}"#).unwrap();
        assert_eq!(
            decoded,
            Response::Failed {
                message: "out of memory".into()
            }
        );
    }
}
