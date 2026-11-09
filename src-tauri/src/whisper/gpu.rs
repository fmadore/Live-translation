//! The Vulkan helper (`whisper-vulkan`), seen from the app: started when a session begins, sent
//! each window, stopped with the session. Every way it can fail — it cannot start (no Vulkan
//! loader on the PC), it finds no usable graphics processor, it stops answering, it dies — comes
//! back here as an error, and the caller decodes on the processor instead.

use std::collections::VecDeque;
use std::io::{self, BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, anyhow, bail};
use tokio_util::sync::CancellationToken;

use super::engine::{self, Decode, RawSegment, Request, Response};
use super::models::ModelLease;
use crate::sync::lock;

/// Loading includes building the device's compute pipelines, which a first run on a slow driver
/// can take a while over; a driver that hangs must not hold the session's start for longer.
const LOAD_TIMEOUT: Duration = Duration::from_secs(120);
/// A window takes well under a second on a graphics processor; one that takes a minute has hung.
const DECODE_TIMEOUT: Duration = Duration::from_secs(60);
const TOKENIZE_TIMEOUT: Duration = Duration::from_secs(10);
/// How often a wait for an answer looks at the discard token.
const POLL: Duration = Duration::from_millis(100);
/// How many of the helper's last lines on standard error are kept to say why it failed.
const RECENT_LINES: usize = 40;
/// How long to wait for the last of those lines once the helper has ended.
const LAST_WORDS: Duration = Duration::from_millis(300);
/// The longest reason shown to the operator.
const MAX_REASON: usize = 240;

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
    /// Its last lines on standard error, and the thread reading them.
    recent: Arc<Mutex<VecDeque<String>>>,
    log: Option<JoinHandle<()>>,
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
        #[cfg(windows)]
        end_with_the_app(&child);
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
        // whisper.cpp and ggml write what they are doing to standard error: kept in the log,
        // and the last of it kept here, for when it explains a failure.
        let recent = Arc::new(Mutex::new(VecDeque::with_capacity(RECENT_LINES)));
        let kept = recent.clone();
        let log = std::thread::Builder::new()
            .name("whisper-vulkan-log".into())
            .spawn(move || {
                for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                    tracing::debug!(target: "whisper_vulkan", "{line}");
                    let mut kept = lock(&kept);
                    if kept.len() == RECENT_LINES {
                        kept.pop_front();
                    }
                    kept.push_back(line);
                }
            })?;
        let mut worker = Self {
            child,
            stdin,
            answers,
            recent,
            log: Some(log),
            device: String::new(),
            text_context: 0,
        };
        let load = Request::Load {
            path: lease.path.clone(),
            bytes: lease.id().bytes(),
            sha256: lease.id().sha256().to_owned(),
            allow_software,
        };
        match worker.ask(&load, None, LOAD_TIMEOUT, None) {
            Ok(Response::Loaded {
                device,
                text_context,
            }) => {
                worker.device = device;
                worker.text_context = text_context;
                Ok(worker)
            }
            Ok(Response::Failed { message }) => Err(anyhow!(message)),
            Ok(other) => bail!("Unexpected answer from the graphics processor helper: {other:?}"),
            Err(error) => Err(error.context(worker.reason())),
        }
    }

    /// Why the helper failed, in the words of the line on its standard error that says so,
    /// when one does: what the app sees of a crash is only an exit code.
    pub fn reason(&mut self) -> String {
        if self.child.try_wait().is_ok_and(|status| status.is_some())
            && let Some(log) = self.log.take()
        {
            // The pipe is closed: its last lines are on their way to the reading thread.
            let deadline = Instant::now() + LAST_WORDS;
            while !log.is_finished() && Instant::now() < deadline {
                std::thread::sleep(Duration::from_millis(10));
            }
        }
        let recent: Vec<String> = lock(&self.recent).iter().cloned().collect();
        let exit = self.child.try_wait().ok().flatten();
        cause(&recent)
            .map(str::to_owned)
            .or_else(|| {
                exit.and_then(|status| status.code())
                    .and_then(windows_status)
            })
            .unwrap_or_default()
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
        match self.ask(&request, Some(samples), DECODE_TIMEOUT, Some(abort)) {
            Ok(Response::Decoded { segments, language }) => Ok((segments, language)),
            Ok(Response::Failed { message }) => Err(anyhow!(message)),
            Ok(other) => bail!("Unexpected answer from the graphics processor helper: {other:?}"),
            Err(error) if abort.is_cancelled() => Err(error),
            Err(error) => {
                let reason = self.reason();
                Err(if reason.is_empty() {
                    error
                } else {
                    error.context(reason)
                })
            }
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

/// Put the helper in a job object that ends it when the app's last handle to the job closes,
/// which Windows does when the app exits however it exits. The helper also ends on its own when
/// its input closes, but not while a driver call it is in has hung, and a helper left behind
/// would hold the graphics processor and its memory. Best effort: without the job, the helper
/// still ends with its input.
#[cfg(windows)]
fn end_with_the_app(child: &Child) {
    use std::os::windows::io::AsRawHandle;
    use std::sync::OnceLock;
    use windows_sys::Win32::System::JobObjects::{
        AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
        JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JobObjectExtendedLimitInformation,
        SetInformationJobObject,
    };

    /// The job's handle, as an address: a raw handle is neither `Send` nor `Sync`. Made once
    /// and never closed, so it lives exactly as long as the app.
    static JOB: OnceLock<Option<usize>> = OnceLock::new();
    let job = JOB.get_or_init(|| {
        // SAFETY: no security attributes and no name: an anonymous job only this process holds.
        let job = unsafe { CreateJobObjectW(std::ptr::null(), std::ptr::null()) };
        if job.is_null() {
            return None;
        }
        let mut limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
        limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        #[expect(
            clippy::cast_possible_truncation,
            reason = "the structure is a few dozen bytes"
        )]
        let size = std::mem::size_of_val(&limits) as u32;
        // SAFETY: `job` was just created and is valid; the pointer and size describe `limits`,
        // which outlives the call.
        let set = unsafe {
            SetInformationJobObject(
                job,
                JobObjectExtendedLimitInformation,
                std::ptr::from_ref(&limits).cast(),
                size,
            )
        };
        (set != 0).then_some(job as usize)
    });
    let Some(job) = *job else {
        tracing::debug!("no job object for the graphics processor helper");
        return;
    };
    // SAFETY: the job handle is never closed, and the child's handle is valid while `child`
    // is borrowed.
    let assigned = unsafe { AssignProcessToJobObject(job as _, child.as_raw_handle() as _) };
    if assigned == 0 {
        tracing::debug!("the graphics processor helper could not join its job object");
    }
}

/// The line among the helper's last ones on standard error that says why it failed: the most
/// specific kind first, then any line with an error word, then whisper.cpp's own report that
/// loading failed, which says that it did but never why. Cut to [`MAX_REASON`] characters.
/// After whisper.cpp's and ggml's formats, as OpenWhispr reads the same libraries' output.
fn cause(lines: &[String]) -> Option<&str> {
    let found = specific(lines)
        .or_else(|| {
            lines
                .iter()
                .map(String::as_str)
                .find(|line| has_error_word(line) && !is_recovered(line) && !is_load_echo(line))
        })
        .or_else(|| lines.iter().map(String::as_str).find(|line| is_abort(line)))
        .or_else(|| {
            lines
                .iter()
                .map(String::as_str)
                .find(|line| is_load_echo(line))
        })?;
    let found = found.trim();
    Some(match found.char_indices().nth(MAX_REASON) {
        Some((at, _)) => &found[..at],
        None => found,
    })
}

/// Lines whose kind names the cause, most specific first: an exception whisper.cpp caught
/// while loading (`vk::DeviceLostError` creating the device), ggml-vulkan's own check of a
/// Vulkan call before it exits, the Vulkan library's exception text, and this helper's last
/// word before it stopped.
fn specific(lines: &[String]) -> Option<&str> {
    const LOAD_EXCEPTION: &str = "exception during model load: ";
    const HELPER: &str = "whisper-vulkan: ";
    let find = |test: &dyn Fn(&str) -> Option<usize>| {
        lines
            .iter()
            .find_map(|line| test(line).map(|at| &line[at..]))
    };
    find(&|line| {
        line.find(LOAD_EXCEPTION)
            .map(|at| at + LOAD_EXCEPTION.len())
    })
    .or_else(|| {
        find(&|line| {
            line.find("ggml_vulkan: ")
                .filter(|_| line.contains("error Error"))
        })
    })
    .or_else(|| find(&|line| line.find("vk::").filter(|_| line.contains(": Error"))))
    .or_else(|| {
        find(&|line| {
            line.starts_with(HELPER)
                .then_some(HELPER.len())
                .filter(|&at| !line[at..].trim().is_empty())
        })
    })
}

fn has_error_word(line: &str) -> bool {
    line.split(|c: char| !c.is_ascii_alphabetic()).any(|word| {
        [
            "error",
            "failed",
            "failure",
            "exception",
            "abort",
            "aborted",
        ]
        .iter()
        .any(|w| word.eq_ignore_ascii_case(w))
    })
}

/// Warnings the backends log and carry on from, though they carry error words: a buffer in
/// ordinary memory instead of pinned memory, the Vulkan loader skipping one of several drivers.
fn is_recovered(line: &str) -> bool {
    (line.contains("pinned memory") && !line.contains("non-pinned"))
        || line.starts_with("WARNING:")
        || line.contains("[Loader Message]")
        || line.contains("Failed getMemoryHostPointerPropertiesEXT")
        || line.contains("Failed ggml_vk_create_buffer")
}

/// What whisper.cpp prints after any failed load: that it failed, not why.
fn is_load_echo(line: &str) -> bool {
    line.contains("failed to load model") || line.contains("failed to initialize whisper context")
}

/// An assertion or `ggml_abort`: `<source file>:<line>: <message>`, often with no error word.
fn is_abort(line: &str) -> bool {
    let line = line.strip_prefix("WHISPER_ASSERT: ").unwrap_or(line);
    let Some(first) = line.split_whitespace().next() else {
        return false;
    };
    let Some(rest) = first.strip_suffix(':') else {
        return false;
    };
    let Some((file, number)) = rest.rsplit_once(':') else {
        return false;
    };
    !number.is_empty()
        && number.bytes().all(|b| b.is_ascii_digit())
        && [".c", ".cpp", ".cu", ".cuh", ".h"]
            .iter()
            .any(|ext| file.ends_with(ext))
}

/// What a Windows exit code says, when it is one of the statuses a helper that cannot even
/// start ends with. The app sees only the number.
fn windows_status(code: i32) -> Option<String> {
    #[expect(
        clippy::cast_sign_loss,
        reason = "an NTSTATUS is an unsigned 32-bit value that Windows hands back as an i32"
    )]
    let status = code as u32;
    let name = match status {
        0xC000_0135 => "a library it needs is missing (STATUS_DLL_NOT_FOUND)",
        0xC000_0139 => "a library it needs is too old (STATUS_ENTRYPOINT_NOT_FOUND)",
        0xC000_007B => "a library it needs is for another processor (STATUS_INVALID_IMAGE_FORMAT)",
        0xC000_0005 => "it crashed (STATUS_ACCESS_VIOLATION)",
        0xC000_001D => "the processor lacks an instruction it uses (STATUS_ILLEGAL_INSTRUCTION)",
        0xC000_0409 => "it stopped itself (STATUS_STACK_BUFFER_OVERRUN)",
        _ => return None,
    };
    Some(format!("{name}, exit code {status:#010x}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(text: &str) -> Vec<String> {
        text.lines().map(str::to_owned).collect()
    }

    #[test]
    fn the_most_specific_line_explains_the_failure() {
        let log = lines(concat!(
            "whisper_init_from_file_with_params_no_state: loading model\n",
            "ggml_vulkan: Found 1 Vulkan devices:\n",
            "WARNING: failed to allocate pinned memory, falling back\n",
            "ggml_vulkan: error ErrorDeviceLost at ggml-vulkan.cpp:2209\n",
            "whisper_init_with_params_no_state: failed to load model\n",
        ));
        assert_eq!(
            cause(&log),
            Some("ggml_vulkan: error ErrorDeviceLost at ggml-vulkan.cpp:2209")
        );
        let log = lines(concat!(
            "whisper_init_with_params_no_state: exception during model load: ",
            "vk::Device::createBuffer: ErrorOutOfDeviceMemory\n",
            "whisper_init_with_params_no_state: failed to load model\n",
        ));
        assert_eq!(
            cause(&log),
            Some("vk::Device::createBuffer: ErrorOutOfDeviceMemory")
        );
        assert_eq!(
            cause(&lines(
                "terminate called after throwing\n  what():  vk::Queue::submit: ErrorDeviceLost"
            )),
            Some("vk::Queue::submit: ErrorDeviceLost")
        );
    }

    #[test]
    fn warnings_the_backend_recovers_from_never_explain_it() {
        let log = lines(concat!(
            "ggml_vulkan: Failed ggml_vk_create_buffer, retrying in host memory\n",
            "[Loader Message] loader skipped a driver: error code -9\n",
            "/src/ggml-vulkan.cpp:8412: Requested preallocation size is too large\n",
        ));
        assert_eq!(
            cause(&log),
            Some("/src/ggml-vulkan.cpp:8412: Requested preallocation size is too large")
        );
        assert_eq!(cause(&lines("all is well\nloaded")), None);
        // Loading failed, and nothing else says why: that much is still worth saying.
        assert_eq!(
            cause(&lines(
                "whisper_init_with_params_no_state: failed to load model"
            )),
            Some("whisper_init_with_params_no_state: failed to load model")
        );
        // The helper's own last word, when it had one.
        assert_eq!(
            cause(&lines("whisper-vulkan: a window without samples")),
            Some("a window without samples")
        );
    }

    #[test]
    fn a_long_reason_is_cut_on_a_character() {
        let long = format!("ggml_vulkan: error ErrorUnknown {}", "é".repeat(400));
        let log = vec![long];
        assert_eq!(cause(&log).unwrap().chars().count(), MAX_REASON);
    }

    #[test]
    fn a_helper_that_could_not_start_is_explained_by_its_exit_code() {
        #[expect(clippy::cast_possible_wrap, reason = "how Windows reports the status")]
        let missing = 0xC000_0135_u32 as i32;
        assert_eq!(
            windows_status(missing).as_deref(),
            Some("a library it needs is missing (STATUS_DLL_NOT_FOUND), exit code 0xc0000135")
        );
        assert_eq!(windows_status(1), None);
    }
}
