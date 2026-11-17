//! Telling a copy installed from GitHub that a newer version exists (issue #217).
//!
//! Tell, don't install: one unauthenticated request for the latest published release, its tag
//! compared with the running version, and a notice in the operator window with a link to the
//! release page. Nothing is downloaded and nothing is installed.
//!
//! Only copies that nothing else keeps up to date ask. The Microsoft Store and the Mac App
//! Store update theirs, and a notice there would send people to an installer that puts a second
//! copy beside the store's (on the Mac, App Review Guideline 2.4.5(vii) forbids it outright).
//! So the check is limited to a Windows installation without package identity (the NSIS and
//! MSI installers; every MSIX, the Store's and the sideloaded test packages, has identity) and
//! to the Mac disk image, the copy that is not sandboxed. Everything else, Linux included, is
//! never eligible outside tests.
//!
//! The core alone decides whether a request goes out. Its state (the operator's answer to
//! *Check for new versions*, the launches counted until that answer, the last check's time, the
//! newest version seen and a skipped one) is a small file in the app's data folder, not webview
//! storage, and every command enforces eligibility, the switch, once a day and no session
//! itself, so the renderer cannot make a store build or a switched-off copy send anything.
//!
//! The release page opened is built here from the validated version number. Neither a URL from
//! GitHub's answer nor one from the renderer ever reaches the shell.

use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use reqwest::Client;
use reqwest::header::ACCEPT;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};
use tokio::sync::Mutex as AsyncMutex;

use crate::atomic_file;
use crate::errors::{AppError, id};
use crate::session::SessionManager;
use crate::sync::lock;

#[cfg(test)]
mod tests;

/// GitHub's REST API. A parameter of every check, so the tests can put a loopback server here.
const API_BASE: &str = "https://api.github.com";
/// The latest published release: GitHub leaves drafts and prereleases out of this one.
const LATEST_PATH: &str = "/repos/fmadore/Live-translation/releases/latest";
/// Where a release's notes are. The version is appended by `release_page`, from a parsed
/// version number only.
const RELEASES_PAGE: &str = "https://github.com/fmadore/Live-translation/releases/tag/v";
/// The state file, in the app's local data folder.
const STATE_FILE: &str = "update-check.json";
/// At most one request a day, failed ones included: a failure is tried again the next day.
const CHECK_INTERVAL_MS: u64 = 24 * 60 * 60 * 1000;
/// The whole request, connection included. A check nobody waits for can afford to give up.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(15);
/// More than any release's JSON, release notes included; anything longer is not an answer.
const MAX_BODY: usize = 1024 * 1024;
/// Launches before the operator is asked whether to check: the question waits for the second,
/// so a first launch is about the app and nothing else.
const ASK_FROM_LAUNCH: u32 = 2;

/// A release number, `MAJOR.MINOR.PATCH`, compared as semantic versions without prerelease
/// parts compare: field by field. Field order matters to the derived ordering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    major: u64,
    minor: u64,
    patch: u64,
}

/// What a release tag turned out to be.
#[derive(Debug, PartialEq, Eq)]
enum Tag {
    Release(Version),
    /// `v1.15.0-rc.1`: GitHub should not return one from `releases/latest`, and it is ignored.
    Prerelease,
}

impl Version {
    pub const fn new(major: u64, minor: u64, patch: u64) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    /// `1.2.3` exactly: three parts of ASCII digits, no leading zero, nothing before or after.
    fn parse(text: &str) -> Option<Self> {
        fn part(text: &str) -> Option<u64> {
            let leading_zero = text.len() > 1 && text.starts_with('0');
            if text.is_empty() || text.len() > 9 || leading_zero {
                return None;
            }
            if !text.bytes().all(|b| b.is_ascii_digit()) {
                return None;
            }
            text.parse().ok()
        }
        let mut parts = text.split('.');
        let version = Self::new(
            part(parts.next()?)?,
            part(parts.next()?)?,
            part(parts.next()?)?,
        );
        parts.next().is_none().then_some(version)
    }

    /// A tag as the release workflow writes it, `v1.2.3`. A hyphenated suffix is a prerelease;
    /// anything else that is not exactly a version (build metadata, spaces, a branch name) is
    /// refused.
    fn parse_tag(tag: &str) -> Option<Tag> {
        let text = tag.strip_prefix('v').unwrap_or(tag);
        if let Some((core, suffix)) = text.split_once('-') {
            let suffix_ok = !suffix.is_empty()
                && suffix
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'-');
            return (suffix_ok && Self::parse(core).is_some()).then_some(Tag::Prerelease);
        }
        Self::parse(text).map(Tag::Release)
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// The release page of `version`. Built from the three numbers alone, so it can only ever be a
/// page of this repository's releases.
fn release_page(version: Version) -> String {
    format!("{RELEASES_PAGE}{version}")
}

/// The operator's answer to *Check for new versions*. Off until it is given: nothing is asked of
/// GitHub before the operator turns it on.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UpdateChecks {
    #[default]
    Unanswered,
    On,
    Off,
}

/// What the interface shows: whether this copy checks at all, the switch, whether to ask the
/// question, the last check and a newer version, if there is one it has not been told to skip.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateStatus {
    /// A copy installed from GitHub. False everywhere else, and then nothing else is shown.
    pub eligible: bool,
    pub checks: UpdateChecks,
    /// Show the one-line question on the start screen: unanswered, from the second launch on.
    pub ask: bool,
    /// When the last check was made, successful or not, in milliseconds since the Unix epoch.
    pub last_checked_ms: Option<u64>,
    /// A newer version than this one, as `X.Y.Z`, unless Not now was chosen for it.
    pub available: Option<String>,
}

/// The state file. Versions are kept as text and parsed again on every read, so a hand-edited
/// or damaged file cannot put anything but a version number into a status or a URL.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
struct Stored {
    checks: UpdateChecks,
    launches: u32,
    last_checked_ms: Option<u64>,
    latest: Option<String>,
    skipped: Option<String>,
}

impl Stored {
    fn latest(&self) -> Option<Version> {
        self.latest.as_deref().and_then(Version::parse)
    }

    fn skipped(&self) -> Option<Version> {
        self.skipped.as_deref().and_then(Version::parse)
    }

    /// The newest version seen, if it is newer than this one and newer than a skipped one.
    fn available(&self, running: Version) -> Option<Version> {
        self.latest()
            .filter(|&latest| latest > running)
            .filter(|&latest| self.skipped().is_none_or(|skipped| latest > skipped))
    }

    /// Whether a day has passed since the last check. A last check in the future means the
    /// clock was set back, and must not hold the next one off until that date.
    fn due(&self, now_ms: u64) -> bool {
        self.last_checked_ms
            .is_none_or(|last| now_ms < last || now_ms - last >= CHECK_INTERVAL_MS)
    }

    fn status(&self, running: Version) -> UpdateStatus {
        UpdateStatus {
            eligible: true,
            checks: self.checks,
            ask: self.checks == UpdateChecks::Unanswered && self.launches >= ASK_FROM_LAUNCH,
            last_checked_ms: self.last_checked_ms,
            available: self.available(running).map(|v| v.to_string()),
        }
    }
}

/// Where and how this copy checks. Built from the app for the commands, by hand in the tests.
pub(crate) struct Context {
    pub path: PathBuf,
    pub api_base: String,
    pub running: Version,
    pub eligible: bool,
    pub timeout: Duration,
}

/// The module's managed state: one check at a time, one file operation at a time, and the
/// launch counted once per process (a reloaded window is not a launch).
#[derive(Default)]
pub struct Updates {
    checking: AsyncMutex<()>,
    io: Mutex<()>,
    launch_counted: AtomicBool,
}

impl Updates {
    /// Read the state file; a missing, unreadable or malformed one is the first launch's state.
    fn load(&self, path: &Path) -> Stored {
        let _io = lock(&self.io);
        read(path)
    }

    /// Change the state file. A failed write is logged and otherwise ignored: nothing about
    /// this feature is worth an error in front of the operator.
    fn modify(&self, path: &Path, change: impl FnOnce(&mut Stored)) -> Stored {
        let _io = lock(&self.io);
        let mut stored = read(path);
        let before = stored.clone();
        change(&mut stored);
        if stored != before {
            write(path, &stored);
        }
        stored
    }

    /// The status, counting this process's launch the first time it is asked for, until the
    /// question has been answered.
    pub(crate) fn status(&self, cx: &Context) -> UpdateStatus {
        if !cx.eligible {
            return UpdateStatus::default();
        }
        let stored = if self.launch_counted.swap(true, Ordering::SeqCst) {
            self.load(&cx.path)
        } else {
            self.modify(&cx.path, |stored| {
                if stored.checks == UpdateChecks::Unanswered {
                    stored.launches = stored.launches.saturating_add(1).min(ASK_FROM_LAUNCH);
                }
            })
        };
        stored.status(cx.running)
    }

    /// Turn the check on or off; an answer to the question either way.
    pub(crate) fn set_checks(&self, cx: &Context, on: bool) -> UpdateStatus {
        if !cx.eligible {
            return UpdateStatus::default();
        }
        self.modify(&cx.path, |stored| {
            stored.checks = if on {
                UpdateChecks::On
            } else {
                UpdateChecks::Off
            };
        })
        .status(cx.running)
    }

    /// Not now: hide the version on offer until a newer one is released.
    pub(crate) fn skip(&self, cx: &Context) -> UpdateStatus {
        if !cx.eligible {
            return UpdateStatus::default();
        }
        self.modify(&cx.path, |stored| {
            if let Some(available) = stored.available(cx.running) {
                stored.skipped = Some(available.to_string());
            }
        })
        .status(cx.running)
    }

    /// The release page of the version on offer, if there is one.
    pub(crate) fn release_page(&self, cx: &Context) -> Option<String> {
        if !cx.eligible {
            return None;
        }
        self.load(&cx.path).available(cx.running).map(release_page)
    }

    /// Ask GitHub for the latest release, if this copy may and it is time: eligible, switched
    /// on, no session running and a day since the last check. Anything else returns the status
    /// as it stands without a request. A failed request changes nothing but the last check's
    /// time, so it is tried again the next day.
    pub(crate) async fn check(
        &self,
        cx: &Context,
        now_ms: u64,
        session_running: bool,
    ) -> UpdateStatus {
        if !cx.eligible {
            return UpdateStatus::default();
        }
        // One at a time: a second caller waits, then finds the check already made today.
        let _checking = self.checking.lock().await;
        let stored = self.load(&cx.path);
        if stored.checks != UpdateChecks::On || session_running || !stored.due(now_ms) {
            return stored.status(cx.running);
        }
        let outcome = match client(cx.timeout) {
            Ok(http) => latest_release(&http, &cx.api_base).await,
            Err(error) => Err(Failure::Network(error)),
        };
        match &outcome {
            Ok(Tag::Release(version)) => tracing::info!("latest release on GitHub: {version}"),
            Ok(Tag::Prerelease) => tracing::info!("GitHub's latest release is a prerelease"),
            // Silent to the operator, and only info here: offline is not a fault.
            Err(failure) => tracing::info!("update check failed: {failure}"),
        }
        self.modify(&cx.path, |stored| {
            stored.last_checked_ms = Some(now_ms);
            if let Ok(Tag::Release(version)) = outcome {
                stored.latest = Some(version.to_string());
            }
        })
        .status(cx.running)
    }
}

fn read(path: &Path) -> Stored {
    // An interrupted replacement never stands in for the last complete one.
    let _ = atomic_file::discard_staging(path);
    match std::fs::read(path) {
        Ok(bytes) => serde_json::from_slice(&bytes).unwrap_or_else(|error| {
            tracing::warn!("ignoring a malformed {STATE_FILE}: {error}");
            Stored::default()
        }),
        Err(error) => {
            if error.kind() != std::io::ErrorKind::NotFound {
                tracing::warn!("could not read {}: {error}", path.display());
            }
            Stored::default()
        }
    }
}

fn write(path: &Path, stored: &Stored) {
    let result = (|| {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let bytes = serde_json::to_vec(stored).map_err(std::io::Error::other)?;
        atomic_file::replace(path, &bytes)
    })();
    if let Err(error) = result {
        tracing::warn!("could not write {}: {error}", path.display());
    }
}

/// Why a check found nothing. Logged, never shown.
#[derive(Debug)]
enum Failure {
    /// No answer: offline, a proxy refusing, the timeout.
    Network(reqwest::Error),
    /// GitHub answered with this status: 403 or 429 for its rate limit, 404 before any release.
    Status(u16),
    /// An answer that is not a release with a version tag.
    Malformed(&'static str),
}

impl std::fmt::Display for Failure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Network(error) => write!(f, "{error:#}"),
            Self::Status(status) => write!(f, "GitHub answered HTTP {status}"),
            Self::Malformed(what) => write!(f, "malformed answer: {what}"),
        }
    }
}

/// The client for one check: the system proxy (reqwest's `system-proxy`), the platform's TLS,
/// a bounded request, no redirect, and a User-Agent that is the app's name and version and
/// nothing else. GitHub refuses a request without one.
fn client(timeout: Duration) -> reqwest::Result<Client> {
    let builder = Client::builder()
        // Plain HTTP exists only for the tests' loopback server.
        .https_only(cfg!(not(test)))
        .timeout(timeout)
        .redirect(reqwest::redirect::Policy::none())
        .user_agent(concat!("LiveTranslation/", env!("CARGO_PKG_VERSION")));
    // A developer's own proxy setting must not intercept the tests' loopback server.
    #[cfg(test)]
    let builder = builder.no_proxy();
    builder.build()
}

/// The fields of GitHub's release object that matter here.
#[derive(Deserialize)]
struct Release {
    tag_name: String,
    #[serde(default)]
    prerelease: bool,
    #[serde(default)]
    draft: bool,
}

async fn latest_release(http: &Client, api_base: &str) -> Result<Tag, Failure> {
    let mut response = http
        .get(format!("{api_base}{LATEST_PATH}"))
        .header(ACCEPT, "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .send()
        .await
        .map_err(|e| Failure::Network(e.without_url()))?;
    let status = response.status();
    if !status.is_success() {
        return Err(Failure::Status(status.as_u16()));
    }
    let mut body = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|e| Failure::Network(e.without_url()))?
    {
        if body.len() + chunk.len() > MAX_BODY {
            return Err(Failure::Malformed("the answer is too long"));
        }
        body.extend_from_slice(&chunk);
    }
    let release: Release =
        serde_json::from_slice(&body).map_err(|_| Failure::Malformed("not a release"))?;
    if release.draft || release.prerelease {
        return Ok(Tag::Prerelease);
    }
    Version::parse_tag(&release.tag_name).ok_or(Failure::Malformed("not a version tag"))
}

/// Whether this is a copy installed from GitHub, which nothing else keeps up to date.
#[cfg(windows)]
pub(crate) fn installed_from_github() -> bool {
    !windows_impl::has_package_identity()
}

/// The Mac disk image: every copy that is not the sandboxed Mac App Store build.
#[cfg(target_os = "macos")]
pub(crate) fn installed_from_github() -> bool {
    !crate::support::app_store_build()
}

/// No other system has a release to be told about.
#[cfg(not(any(windows, target_os = "macos")))]
pub(crate) fn installed_from_github() -> bool {
    false
}

#[cfg(windows)]
mod windows_impl {
    use windows_sys::Win32::Foundation::APPMODEL_ERROR_NO_PACKAGE;
    use windows_sys::Win32::Storage::Packaging::Appx::GetCurrentPackageFullName;

    /// Whether this process runs with package identity: any MSIX, the Store's and the
    /// sideloaded test packages alike. Anything but "no package" counts as identity, so a
    /// surprise answer leaves the copy not checking.
    pub(super) fn has_package_identity() -> bool {
        let mut length = 0u32;
        // SAFETY: a zero length with a null buffer is the documented way to ask for the length
        // alone; the function writes only `length`, which outlives the call.
        let result = unsafe { GetCurrentPackageFullName(&raw mut length, std::ptr::null_mut()) };
        result != APPMODEL_ERROR_NO_PACKAGE
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| {
            u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX)
        })
}

/// This copy's context. Without a data folder there is nowhere to keep the state that decides
/// whether a request goes out, so such a copy is treated as one that never checks.
fn context(app: &AppHandle) -> Context {
    let running = &app.package_info().version;
    let path = app
        .path()
        .app_local_data_dir()
        .map(|dir| dir.join(STATE_FILE));
    Context {
        eligible: installed_from_github() && path.is_ok(),
        path: path.unwrap_or_default(),
        api_base: API_BASE.to_owned(),
        running: Version::new(running.major, running.minor, running.patch),
        timeout: REQUEST_TIMEOUT,
    }
}

/// Run a file-bound step off the async runtime.
async fn blocking<T: Send + 'static>(
    app: AppHandle,
    step: impl FnOnce(&Updates, &Context) -> T + Send + 'static,
) -> Result<T, AppError> {
    tauri::async_runtime::spawn_blocking(move || step(&app.state::<Updates>(), &context(&app)))
        .await
        .map_err(|e| AppError::with(id::TASK_FAILED, e))
}

/// Where the check stands, for the start screen and Settings › App. The first call of a
/// process counts the launch.
#[tauri::command]
pub async fn update_status(app: AppHandle) -> Result<UpdateStatus, AppError> {
    blocking(app, |updates, cx| updates.status(cx)).await
}

/// Check now if it is time. The core decides: a copy that is not eligible, a switch that is not
/// on, a running session or a check made in the last day all return the status unchanged
/// without a request.
#[tauri::command]
pub async fn check_for_update(
    app: AppHandle,
    updates: State<'_, Updates>,
    sessions: State<'_, SessionManager>,
) -> Result<UpdateStatus, AppError> {
    let cx = context(&app);
    Ok(updates.check(&cx, now_ms(), sessions.is_active()).await)
}

/// *Check for new versions*, on or off.
#[tauri::command]
pub async fn set_update_checks(app: AppHandle, on: bool) -> Result<UpdateStatus, AppError> {
    blocking(app, move |updates, cx| updates.set_checks(cx, on)).await
}

/// Not now: hide the version on offer until a newer one.
#[tauri::command]
pub async fn skip_update_version(app: AppHandle) -> Result<UpdateStatus, AppError> {
    blocking(app, |updates, cx| updates.skip(cx)).await
}

/// Open the release page of the version on offer in the default browser. No argument, for the
/// reason `open_support_page` takes none: the page is built here, from the version number.
#[tauri::command]
pub async fn open_release_page(app: AppHandle) -> Result<(), AppError> {
    let page = blocking(app, |updates, cx| updates.release_page(cx)).await?;
    let Some(page) = page else {
        return Err(AppError::with(
            id::OPEN_LINK,
            "no newer version is on offer",
        ));
    };
    tauri::async_runtime::spawn_blocking(move || crate::support::open(&page))
        .await
        .map_err(|e| AppError::with(id::TASK_FAILED, e))?
}
