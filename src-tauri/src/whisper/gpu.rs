//! The Vulkan helper (`whisper-vulkan`), seen from the app: started when a session begins, sent
//! each window, stopped with the session. Every way it can fail — it cannot start (no Vulkan
//! loader on the PC), it finds no usable graphics processor, it stops answering, it dies — comes
//! back here as an error, and the caller decodes on the processor instead.

use std::io::{self, BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::time::{Duration, Instant};

use anyhow::{Context, Result, anyhow, bail};
use tokio_util::sync::CancellationToken;

use super::engine::{self, Decode, RawSegment, Request, Response};
use super::models::ModelLease;

/// Loading includes building the device's compute pipelines, which a first run on a slow driver
/// can take a while over; a driver that hangs must not hold the session's start for longer.
const LOAD_TIMEOUT: Duration = Duration::from_secs(120);
/// A window takes well under a second on a graphics processor; one that takes a minute has hung.
const DECODE_TIMEOUT: Duration = Duration::from_secs(60);
const TOKENIZE_TIMEOUT: Duration = Duration::from_secs(10);
/// How often a wait for an answer looks at the discard token.
const POLL: Duration = Duration::from_millis(100);

/// The helper beside the app's own executable, where the installer puts it, when there is one.
/// None on macOS, where Whisper uses the graphics processor through Metal in the app itself.
pub fn helper_path() -> Option<PathBuf> {
    if cfg!(target_os = "macos") {
        return None;
    }
    let name = format!("whisper-vulkan{}", std::env::consts::EXE_SUFFIX);
    let path = std::env::current_exe().ok()?.with_file_name(name);
    path.is_file().then_some(path)
}

pub struct Worker {
    child: Child,
    stdin: ChildStdin,
    /// What the helper answers, read on a thread of its own so every wait can time out.
    answers: Receiver<io::Result<Response>>,
    /// The graphics processor, as its driver names it.
    pub device: String,
    /// The model's text context, for the names-and-terms prompt's budget.
    pub text_context: i32,
}

impl Worker {
    /// Start `program` and load the leased model on the graphics processor it chooses.
    pub fn start(program: &Path, lease: &ModelLease, allow_software: bool) -> Result<Self> {
        let mut command = Command::new(program);
        command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            // CREATE_NO_WINDOW: the helper is a console program; no console may flash up.
            command.creation_flags(0x0800_0000);
        }
        let mut child = command
            .spawn()
            .with_context(|| format!("Could not start {}", program.display()))?;
        let (stdin, stdout, stderr) =
            match (child.stdin.take(), child.stdout.take(), child.stderr.take()) {
                (Some(stdin), Some(stdout), Some(stderr)) => (stdin, stdout, stderr),
                _ => {
                    let _ = child.kill();
                    bail!("The graphics processor helper started without its pipes");
                }
            };
        let (sender, answers) = mpsc::channel();
        std::thread::Builder::new()
            .name("whisper-vulkan-out".into())
            .spawn(move || {
                let mut stdout = BufReader::new(stdout);
                loop {
                    match engine::receive::<Response>(&mut stdout) {
                        Ok(Some(answer)) => {
                            if sender.send(Ok(answer)).is_err() {
                                return;
                            }
                        }
                        Ok(None) => return,
                        Err(error) => {
                            let _ = sender.send(Err(error));
                            return;
                        }
                    }
                }
            })?;
        // whisper.cpp and ggml write what they are doing to standard error: kept in the log.
        std::thread::Builder::new()
            .name("whisper-vulkan-log".into())
            .spawn(move || {
                for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                    tracing::debug!(target: "whisper_vulkan", "{line}");
                }
            })?;
        let mut worker = Self {
            child,
            stdin,
            answers,
            device: String::new(),
            text_context: 0,
        };
        let load = Request::Load {
            path: lease.path.clone(),
            bytes: lease.id().bytes(),
            sha256: lease.id().sha256().to_owned(),
            allow_software,
        };
        match worker.ask(&load, None, LOAD_TIMEOUT, None)? {
            Response::Loaded {
                device,
                text_context,
            } => {
                worker.device = device;
                worker.text_context = text_context;
                Ok(worker)
            }
            Response::Failed { message } => Err(anyhow!(message)),
            other => bail!("Unexpected answer from the graphics processor helper: {other:?}"),
        }
    }

    /// Send `request` (and `samples`, for a window) and wait for its answer. The helper is
    /// stopped when the wait times out or `abort` fires, so it can never answer late.
    fn ask(
        &mut self,
        request: &Request,
        samples: Option<&[f32]>,
        timeout: Duration,
        abort: Option<&CancellationToken>,
    ) -> Result<Response> {
        let sent = engine::send(&mut self.stdin, request).and_then(|()| match samples {
            Some(samples) => {
                engine::write_frame(&mut self.stdin, &engine::samples_to_bytes(samples))
            }
            None => Ok(()),
        });
        if let Err(error) = sent {
            bail!(
                "The graphics processor helper stopped ({}): {error}",
                self.status()
            );
        }
        let deadline = Instant::now() + timeout;
        loop {
            if abort.is_some_and(CancellationToken::is_cancelled) {
                self.stop();
                bail!("Discarded");
            }
            match self.answers.recv_timeout(POLL) {
                Ok(answer) => {
                    return answer.context("The graphics processor helper's answer was unreadable");
                }
                Err(RecvTimeoutError::Timeout) if Instant::now() < deadline => {}
                Err(RecvTimeoutError::Timeout) => {
                    self.stop();
                    bail!("The graphics processor helper stopped answering");
                }
                Err(RecvTimeoutError::Disconnected) => {
                    bail!("The graphics processor helper stopped ({})", self.status())
                }
            }
        }
    }

    /// `text` as the model's tokens; `None` when the tokenizer refuses it or the helper fails.
    pub fn tokenize(&mut self, text: &str) -> Option<Vec<i32>> {
        let request = Request::Tokenize { text: text.into() };
        match self.ask(&request, None, TOKENIZE_TIMEOUT, None) {
            Ok(Response::Tokens { tokens }) => tokens,
            _ => None,
        }
    }

    /// Decode one window: its segments and the language whisper.cpp decoded it in.
    pub fn decode(
        &mut self,
        decode: &Decode,
        samples: &[f32],
        abort: &CancellationToken,
    ) -> Result<(Vec<RawSegment>, i32)> {
        let request = Request::Decode(decode.clone());
        match self.ask(&request, Some(samples), DECODE_TIMEOUT, Some(abort))? {
            Response::Decoded { segments, language } => Ok((segments, language)),
            Response::Failed { message } => Err(anyhow!(message)),
            other => bail!("Unexpected answer from the graphics processor helper: {other:?}"),
        }
    }

    fn stop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }

    /// How the helper ended, for the log, once it has.
    fn status(&mut self) -> String {
        match self.child.try_wait() {
            Ok(Some(status)) => status.to_string(),
            _ => "still running".into(),
        }
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        self.stop();
    }
}
