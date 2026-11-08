//! Live Translation's Whisper on a graphics processor, through Vulkan.
//!
//! The app starts this program beside itself when a local Whisper session begins on Windows,
//! sends it the model to load, then each window to decode, over standard input and output
//! (`engine.rs` has the messages). It answers each request in turn and ends when the app closes
//! its input. Anything that goes wrong here — no Vulkan loader on the PC, so this program cannot
//! even start; no usable graphics processor; the device running out of memory; a driver fault —
//! ends this program, or fails the request, and the app decodes on the processor instead.

mod device;
// Shared word for word with the app (`src/whisper/mod.rs`); some of it is the app's half.
#[allow(dead_code)]
mod engine;

use std::io::{self, Read, Write};
use std::path::Path;
use std::process::ExitCode;

use sha2::{Digest, Sha256};
use whisper_rs::{WhisperContext, WhisperContextParameters, WhisperState};

use engine::{Request, Response};

fn main() -> ExitCode {
    let mut input = io::stdin().lock();
    let mut output = io::stdout().lock();
    match serve(&mut input, &mut output) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("whisper-vulkan: {error}");
            ExitCode::FAILURE
        }
    }
}

struct Loaded {
    context: WhisperContext,
    state: WhisperState,
}

/// Answer requests until the input ends. An error here is the stream itself breaking; a request
/// that fails is answered with `Failed`.
fn serve(input: &mut impl Read, output: &mut impl Write) -> io::Result<()> {
    let mut loaded: Option<Loaded> = None;
    while let Some(request) = engine::receive::<Request>(input)? {
        let response = match request {
            Request::Load {
                path,
                bytes,
                sha256,
                allow_software,
            } => {
                // Release the previous model first: two may not fit on the device.
                loaded = None;
                match load(&path, bytes, &sha256, allow_software) {
                    Ok((model, device)) => {
                        let text_context = model.context.n_text_ctx();
                        loaded = Some(model);
                        Response::Loaded {
                            device,
                            text_context,
                        }
                    }
                    Err(message) => Response::Failed { message },
                }
            }
            Request::Tokenize { text } => Response::Tokens {
                // Every token stands for at least one byte, so the text's length is room
                // enough: whisper-rs mishandles whisper.cpp's "needs more room" answer.
                tokens: loaded
                    .as_ref()
                    .and_then(|model| model.context.tokenize(&text, text.len() + 1).ok()),
            },
            Request::Decode(decode) => {
                let samples = engine::read_frame(input)?.ok_or_else(|| {
                    io::Error::new(io::ErrorKind::UnexpectedEof, "a window without samples")
                })?;
                let samples = engine::bytes_to_samples(&samples)?;
                match loaded.as_mut() {
                    None => Response::Failed {
                        message: "No model is loaded".into(),
                    },
                    Some(model) => match model.state.full(engine::params(&decode), &samples) {
                        Ok(()) => Response::Decoded {
                            segments: engine::segments(&model.state),
                            language: model.state.full_lang_id_from_state(),
                        },
                        Err(error) => Response::Failed {
                            message: format!("Whisper failed: {error}"),
                        },
                    },
                }
            }
        };
        engine::send(output, &response)?;
    }
    Ok(())
}

/// The model at `path` on the chosen graphics processor, and that device's name.
fn load(
    path: &Path,
    bytes: u64,
    sha256: &str,
    allow_software: bool,
) -> Result<(Loaded, String), String> {
    // Read through Rust's filesystem API, which takes any Windows path, and checked again here:
    // a file replaced since the app checked it must never reach whisper.cpp.
    let data = std::fs::read(path).map_err(|e| format!("Could not read the model: {e}"))?;
    let digest: String = Sha256::digest(&data)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    if data.len() as u64 != bytes || digest != sha256 {
        return Err("Whisper model integrity check failed".into());
    }
    let devices: Vec<device::Device> = whisper_rs::vulkan::list_devices()
        .into_iter()
        .map(|d| device::Device {
            name: d.name,
            memory: d.vram.total,
        })
        .collect();
    let index = device::choose(&devices, allow_software).ok_or_else(|| {
        let names: Vec<&str> = devices.iter().map(|d| d.name.as_str()).collect();
        format!("No usable graphics processor (found: {names:?})")
    })?;
    // whisper.cpp numbers the graphics processors it can use in ggml's order, which for Vulkan,
    // the only graphics backend built in here, is the order listed above.
    let gpu_device = i32::try_from(index).map_err(|e| e.to_string())?;
    let mut params = WhisperContextParameters::default();
    params.use_gpu(true).flash_attn(true).gpu_device(gpu_device);
    let context = WhisperContext::new_from_buffer_with_params(&data, params)
        .map_err(|e| format!("Could not load the model: {e}"))?;
    let state = context
        .create_state()
        .map_err(|e| format!("Could not prepare the model: {e}"))?;
    Ok((Loaded { context, state }, devices[index].name.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A request that needs no graphics processor, through the real loop: decoding before any
    /// model is loaded fails and leaves the helper running for the next request.
    #[test]
    fn requests_before_a_model_is_loaded_are_answered_not_fatal() {
        let mut input = Vec::new();
        engine::send(&mut input, &Request::Decode(engine::Decode::default())).unwrap();
        engine::write_frame(&mut input, &engine::samples_to_bytes(&[0.0; 16])).unwrap();
        engine::send(
            &mut input,
            &Request::Tokenize {
                text: "IWAC".into(),
            },
        )
        .unwrap();
        let mut output = Vec::new();
        serve(&mut input.as_slice(), &mut output).unwrap();
        let mut answers = output.as_slice();
        assert!(matches!(
            engine::receive::<Response>(&mut answers).unwrap(),
            Some(Response::Failed { .. })
        ));
        assert_eq!(
            engine::receive::<Response>(&mut answers).unwrap(),
            Some(Response::Tokens { tokens: None })
        );
        assert_eq!(engine::receive::<Response>(&mut answers).unwrap(), None);
    }

    #[test]
    fn a_model_that_is_not_the_pinned_file_is_refused_before_whisper_reads_it() {
        let dir = std::env::temp_dir().join(format!("whisper-vulkan-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("model.bin");
        std::fs::write(&path, b"not a model").unwrap();
        let refused = load(&path, 11, "0000", false).err().unwrap();
        assert!(refused.contains("integrity"), "{refused}");
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
