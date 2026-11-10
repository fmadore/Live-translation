//! The Vulkan helper (`whisper-vulkan`), seen from the app: started when a session begins, sent
//! each window, stopped with the session. Every way it can fail — it cannot start (no Vulkan
//! loader on the PC), it finds no usable graphics processor, it stops answering, it dies — comes
//! back here as an error, and the caller decodes on the processor instead.

use std::collections::VecDeque;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, anyhow, bail};
use tokio_util::sync::CancellationToken;

use super::engine::{self, Decode, RawSegment, Request, Response};
use super::models::ModelLease;
use crate::sync::lock;

/// How long each kind of request may take before the helper is taken to have hung.
#[derive(Debug, Clone, Copy)]
struct Timeouts {
    load: Duration,
    decode: Duration,
    tokenize: Duration,
}

const TIMEOUTS: Timeouts = Timeouts {
    // Loading includes building the device's compute pipelines, which a first run on a slow
    // driver can take a while over; a driver that hangs must not hold the session's start for
    // longer, and the session's discard token ends the wait sooner.
    load: Duration::from_secs(120),
    // A window takes well under a second on a graphics processor; one that takes a minute has
    // hung.
    decode: Duration::from_secs(60),
    tokenize: Duration::from_secs(10),
};
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
    /// The helper's process; `None` for the tests' helper, which is a thread.
    child: Option<Child>,
    stdin: Box<dyn Write + Send>,
    /// What the helper answers, read on a thread of its own so every wait can time out.
    answers: Receiver<io::Result<Response>>,
    /// Its last lines on standard error, and the thread reading them.
    recent: Arc<Mutex<VecDeque<String>>>,
    log: Option<JoinHandle<()>>,
    timeouts: Timeouts,
    /// The graphics processor, as its driver names it.
    pub device: String,
    /// The model's text context, for the names-and-terms prompt's budget.
    pub text_context: i32,
}

/// What became of one request for a window: decoded, or answered as failed by a helper that is
/// still running and can take another (`Ok(Err)`). `Err` is the helper itself failing: it could
/// not be written to, its answer was unreadable, it stopped answering or it ended.
type Attempt = Result<Result<(Vec<RawSegment>, i32), String>>;

impl Worker {
    /// Start `program` and load the leased model on the graphics processor it chooses. `abort`
    /// ends the wait for the load, and the helper with it: the session's start holds the
    /// lifecycle lock meanwhile.
    pub fn start(
        program: &Path,
        lease: &ModelLease,
        allow_software: bool,
        abort: &CancellationToken,
    ) -> Result<Self> {
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
        let mut worker = Self::attach(Some(child), stdin, stdout, stderr, TIMEOUTS)?;
        worker.load(
            &Request::Load {
                path: lease.path.clone(),
                bytes: lease.id().bytes(),
                sha256: lease.id().sha256().to_owned(),
                allow_software,
            },
            abort,
        )?;
        Ok(worker)
    }

    /// A worker talking to a helper over these streams: its input, its answers and its log.
    fn attach(
        child: Option<Child>,
        stdin: impl Write + Send + 'static,
        stdout: impl Read + Send + 'static,
        stderr: impl Read + Send + 'static,
        timeouts: Timeouts,
    ) -> Result<Self> {
        let (sender, answers) = mpsc::channel();
        let reader = std::thread::Builder::new()
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
            });
        // whisper.cpp and ggml write what they are doing to standard error: kept in the log,
        // and the last of it kept here, for when it explains a failure.
        let recent = Arc::new(Mutex::new(VecDeque::with_capacity(RECENT_LINES)));
        let kept = recent.clone();
        let log = reader.and_then(|_| {
            std::thread::Builder::new()
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
                })
        });
        let mut worker = Self {
            child,
            stdin: Box::new(stdin),
            answers,
            recent,
            log: None,
            timeouts,
            device: String::new(),
            text_context: 0,
        };
        // Returning the error drops the worker, which ends the helper.
        worker.log = Some(log?);
        Ok(worker)
    }

    /// Ask the helper to load a model; `abort` ends the wait, and the helper.
    fn load(&mut self, load: &Request, abort: &CancellationToken) -> Result<()> {
        match self.ask(load, None, self.timeouts.load, Some(abort)) {
            Ok(Response::Loaded {
                device,
                text_context,
            }) => {
                self.device = device;
                self.text_context = text_context;
                Ok(())
            }
            Ok(Response::Failed { message }) => Err(anyhow!(message)),
            Ok(other) => bail!("Unexpected answer from the graphics processor helper: {other:?}"),
            Err(error) if abort.is_cancelled() => Err(error),
            Err(error) => Err(self.explained(error)),
        }
    }

    /// Why the helper failed, in the words of the line on its standard error that says so,
    /// when one does: what the app sees of a crash is only an exit code.
    pub fn reason(&mut self) -> String {
        let ended = self
            .child
            .as_mut()
            .is_some_and(|child| child.try_wait().is_ok_and(|status| status.is_some()));
        if ended && let Some(log) = self.log.take() {
            // The pipe is closed: its last lines are on their way to the reading thread.
            let deadline = Instant::now() + LAST_WORDS;
            while !log.is_finished() && Instant::now() < deadline {
                std::thread::sleep(Duration::from_millis(10));
            }
        }
        let recent: Vec<String> = lock(&self.recent).iter().cloned().collect();
        let exit = self
            .child
            .as_mut()
            .and_then(|child| child.try_wait().ok().flatten());
        cause(&recent)
            .map(str::to_owned)
            .or_else(|| {
                exit.and_then(|status| status.code())
                    .and_then(windows_status)
            })
            .unwrap_or_default()
    }

    /// `error`, with the helper's own reason in front of it when it gave one.
    fn explained(&mut self, error: anyhow::Error) -> anyhow::Error {
        let reason = self.reason();
        if reason.is_empty() {
            error
        } else {
            error.context(reason)
        }
    }

    /// Send `request` (and `samples`, for a window) and wait for its answer. The helper is
    /// stopped when the wait times out, its answer is unreadable or `abort` fires, so it can
    /// never answer late.
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
                self.end();
                bail!("Discarded");
            }
            match self.answers.recv_timeout(POLL) {
                Ok(Ok(answer)) => return Ok(answer),
                Ok(Err(error)) => {
                    // The stream is out of step: nothing it says after this can be trusted.
                    self.end();
                    return Err(anyhow::Error::new(error)
                        .context("The graphics processor helper's answer was unreadable"));
                }
                Err(RecvTimeoutError::Timeout) if Instant::now() < deadline => {}
                Err(RecvTimeoutError::Timeout) => {
                    self.end();
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
        match self.ask(&request, None, self.timeouts.tokenize, None) {
            Ok(Response::Tokens { tokens }) => tokens,
            _ => None,
        }
    }

    /// Decode one window: its segments and the language whisper.cpp decoded it in.
    ///
    /// A window the helper answers as failed is asked for once more: the helper is still
    /// running, and a failure that passes (memory the driver could not find at that moment)
    /// should not cost the rest of the session the graphics processor. A second failure of the
    /// same window is an error, as is any failure of the helper itself, which is never retried.
    pub fn decode(
        &mut self,
        decode: &Decode,
        samples: &[f32],
        abort: &CancellationToken,
    ) -> Result<(Vec<RawSegment>, i32)> {
        let message = match self.attempt(decode, samples, abort)? {
            Ok(decoded) => return Ok(decoded),
            Err(message) => message,
        };
        if abort.is_cancelled() {
            bail!("Discarded");
        }
        tracing::warn!("Whisper's graphics processor failed a window; trying it again: {message}");
        match self.attempt(decode, samples, abort)? {
            Ok(decoded) => Ok(decoded),
            Err(again) => Err(anyhow!(again).context("The window failed twice")),
        }
    }

    fn attempt(&mut self, decode: &Decode, samples: &[f32], abort: &CancellationToken) -> Attempt {
        let request = Request::Decode(decode.clone());
        match self.ask(&request, Some(samples), self.timeouts.decode, Some(abort)) {
            Ok(Response::Decoded { segments, language }) => Ok(Ok((segments, language))),
            Ok(Response::Failed { message }) => Ok(Err(message)),
            Ok(other) => {
                // Answers out of step with the requests: the helper cannot be trusted further.
                self.end();
                bail!("Unexpected answer from the graphics processor helper: {other:?}")
            }
            Err(error) if abort.is_cancelled() => Err(error),
            Err(error) => Err(self.explained(error)),
        }
    }

    /// Stop the helper now, freeing the graphics processor and the memory it holds.
    pub fn end(&mut self) {
        if let Some(child) = self.child.as_mut() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }

    /// How the helper ended, for the log, once it has.
    fn status(&mut self) -> String {
        match self.child.as_mut().map(Child::try_wait) {
            Some(Ok(Some(status))) => status.to_string(),
            Some(_) => "still running".into(),
            None => "ended".into(),
        }
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        self.end();
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

    /// What the tests' helper does with each request it reads, in order.
    enum Reply {
        Answer(Response),
        /// A frame that is not a message.
        Garbage,
        /// Read on and never answer, as a helper stuck in a driver call.
        Silence,
        /// End, as a helper that crashed while working on the request.
        Die,
    }

    /// A worker whose helper is a thread over in-memory pipes, following `script`, and a count
    /// of the requests that helper read. Past the end of the script it ends.
    fn scripted(script: Vec<Reply>, timeouts: Timeouts) -> (Worker, Arc<Mutex<usize>>) {
        let (mut requests, app_out) = io::pipe().unwrap();
        let (app_in, mut answers) = io::pipe().unwrap();
        let (log_in, log_out) = io::pipe().unwrap();
        let asked = Arc::new(Mutex::new(0));
        let counted = asked.clone();
        std::thread::spawn(move || {
            let _log = log_out;
            for reply in script {
                let Ok(Some(request)) = engine::receive::<Request>(&mut requests) else {
                    return;
                };
                if matches!(request, Request::Decode(_))
                    && !matches!(engine::read_frame(&mut requests), Ok(Some(_)))
                {
                    return;
                }
                *counted.lock().unwrap() += 1;
                match reply {
                    Reply::Answer(response) => engine::send(&mut answers, &response).unwrap(),
                    Reply::Garbage => engine::write_frame(&mut answers, b"{not json").unwrap(),
                    Reply::Silence => {
                        while let Ok(Some(_)) = engine::read_frame(&mut requests) {}
                        return;
                    }
                    Reply::Die => return,
                }
            }
        });
        let worker = Worker::attach(None, app_out, app_in, log_in, timeouts).unwrap();
        (worker, asked)
    }

    const QUICK: Timeouts = Timeouts {
        load: Duration::from_secs(30),
        decode: Duration::from_millis(300),
        tokenize: Duration::from_millis(300),
    };

    fn decoded(text: &str) -> Response {
        Response::Decoded {
            segments: vec![RawSegment {
                text: text.into(),
                start_cs: 0,
                end_cs: 100,
                no_speech: 0.0,
                mean_logprob: -0.1,
            }],
            language: 2,
        }
    }

    fn failed() -> Response {
        Response::Failed {
            message: "Whisper failed: -6".into(),
        }
    }

    fn window(worker: &mut Worker) -> Result<(Vec<RawSegment>, i32)> {
        worker.decode(
            &Decode::default(),
            &[0.0; 16_000],
            &CancellationToken::new(),
        )
    }

    #[test]
    fn a_window_decodes_through_the_helper() {
        let (mut worker, asked) = scripted(vec![Reply::Answer(decoded("hello"))], QUICK);
        let (segments, language) = window(&mut worker).unwrap();
        assert_eq!((segments[0].text.as_str(), language), ("hello", 2));
        assert_eq!(*asked.lock().unwrap(), 1);
    }

    /// B7: a window the helper answers as failed is tried once more on the helper, which is
    /// still running; the session keeps the graphics processor when that succeeds.
    #[test]
    fn a_failed_window_is_tried_again_on_the_helper() {
        let (mut worker, asked) = scripted(
            vec![Reply::Answer(failed()), Reply::Answer(decoded("again"))],
            QUICK,
        );
        let (segments, _) = window(&mut worker).unwrap();
        assert_eq!(segments[0].text, "again");
        assert_eq!(*asked.lock().unwrap(), 2);
        // And the helper still takes the next window.
        let (mut worker, _) = scripted(
            vec![
                Reply::Answer(failed()),
                Reply::Answer(decoded("again")),
                Reply::Answer(decoded("next")),
            ],
            QUICK,
        );
        window(&mut worker).unwrap();
        assert_eq!(window(&mut worker).unwrap().0[0].text, "next");
    }

    #[test]
    fn a_window_that_fails_twice_is_an_error() {
        let (mut worker, asked) = scripted(
            vec![Reply::Answer(failed()), Reply::Answer(failed())],
            QUICK,
        );
        let error = format!("{:#}", window(&mut worker).unwrap_err());
        assert!(error.contains("failed twice"), "{error}");
        assert!(error.contains("Whisper failed: -6"), "{error}");
        assert_eq!(*asked.lock().unwrap(), 2);
    }

    /// The helper itself failing is never retried: the caller moves to the processor at once.
    #[test]
    fn a_helper_that_dies_mid_window_is_an_error_without_a_retry() {
        let (mut worker, asked) = scripted(vec![Reply::Die, Reply::Answer(decoded("x"))], QUICK);
        let error = format!("{:#}", window(&mut worker).unwrap_err());
        assert!(error.contains("helper stopped"), "{error}");
        assert_eq!(*asked.lock().unwrap(), 1);
    }

    #[test]
    fn a_helper_that_stops_answering_times_out() {
        let (mut worker, asked) = scripted(vec![Reply::Silence], QUICK);
        let started = Instant::now();
        let error = format!("{:#}", window(&mut worker).unwrap_err());
        assert!(error.contains("stopped answering"), "{error}");
        assert!(started.elapsed() < Duration::from_secs(5));
        assert_eq!(*asked.lock().unwrap(), 1);
    }

    #[test]
    fn an_unreadable_answer_is_an_error_without_a_retry() {
        let (mut worker, asked) =
            scripted(vec![Reply::Garbage, Reply::Answer(decoded("x"))], QUICK);
        let error = format!("{:#}", window(&mut worker).unwrap_err());
        assert!(error.contains("unreadable"), "{error}");
        assert_eq!(*asked.lock().unwrap(), 1);
        // Nothing more is read from a stream out of step.
        assert!(window(&mut worker).is_err());
    }

    #[test]
    fn a_tokenizer_that_fails_leaves_the_prompt_empty() {
        let (mut worker, _) = scripted(vec![Reply::Silence], QUICK);
        assert_eq!(worker.tokenize("IWAC"), None);
    }

    fn load_request() -> Request {
        Request::Load {
            path: PathBuf::from("model.bin"),
            bytes: 1,
            sha256: "00".into(),
            allow_software: false,
        }
    }

    #[test]
    fn a_load_reports_the_device_or_the_helpers_refusal() {
        let (mut worker, _) = scripted(
            vec![Reply::Answer(Response::Loaded {
                device: "Radeon".into(),
                text_context: 448,
            })],
            QUICK,
        );
        worker
            .load(&load_request(), &CancellationToken::new())
            .unwrap();
        assert_eq!(
            (worker.device.as_str(), worker.text_context),
            ("Radeon", 448)
        );
        let (mut worker, _) = scripted(
            vec![Reply::Answer(Response::Failed {
                message: "No usable graphics processor".into(),
            })],
            QUICK,
        );
        let error = worker
            .load(&load_request(), &CancellationToken::new())
            .unwrap_err();
        assert_eq!(error.to_string(), "No usable graphics processor");
    }

    /// B10: the session's discard token ends a load stuck in the driver long before its
    /// timeout, rather than holding the session's start for two minutes.
    #[test]
    fn discarding_ends_a_load_that_hangs() {
        let (mut worker, _) = scripted(vec![Reply::Silence], QUICK);
        let abort = CancellationToken::new();
        let cancel = abort.clone();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(200));
            cancel.cancel();
        });
        let started = Instant::now();
        let error = worker.load(&load_request(), &abort).unwrap_err();
        assert_eq!(error.to_string(), "Discarded");
        assert!(started.elapsed() < Duration::from_secs(5));
    }
}
