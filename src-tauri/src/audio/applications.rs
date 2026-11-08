//! Explicit process selection. Creation time prevents a recycled PID capturing another app.
//!
//! Windows captures the selected process and its child processes. macOS has no process tree for
//! audio; it captures the selected application's own process and its helpers, the processes
//! whose bundle identifier extends the application's (`com.google.Chrome.helper` for
//! `com.google.Chrome`), which is where browsers and Electron apps play their sound.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProcessIdentity {
    pub pid: u32,
    pub created_at: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum SystemCapture {
    #[default]
    Output,
    Application {
        process: Option<ProcessIdentity>,
    },
}

#[derive(Debug, Serialize)]
pub struct Application {
    pub process: ProcessIdentity,
    pub name: String,
}
#[derive(Debug, Serialize)]
pub struct ApplicationList {
    pub supported: bool,
    pub applications: Vec<Application>,
}

pub fn validate(capture: &SystemCapture) -> anyhow::Result<()> {
    if let SystemCapture::Application { process } = capture {
        let identity = process
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("select an application before starting capture"))?;
        #[cfg(any(windows, target_os = "macos"))]
        let _ = ProcessGuard::selected(identity)?;
        #[cfg(not(any(windows, target_os = "macos")))]
        {
            let _ = identity;
            anyhow::bail!("application capture requires Windows or macOS");
        }
    }
    Ok(())
}

#[cfg(not(any(windows, target_os = "macos")))]
#[allow(
    clippy::unnecessary_wraps,
    reason = "the Windows build's signature, which can fail"
)]
pub fn list() -> anyhow::Result<ApplicationList> {
    Ok(ApplicationList {
        supported: false,
        applications: vec![],
    })
}

#[cfg(windows)]
pub use platform::{ProcessGuard, list};

#[cfg(target_os = "macos")]
pub use mac::{ProcessGuard, audio_processes, list};

/// Whether a process with bundle identifier `process` belongs to the application `app`: the
/// application itself, or one of its helpers, whose identifiers extend the application's.
/// `com.google.Chrome.helper` belongs to `com.google.Chrome`; `com.google.Chromecast` does not.
#[cfg(any(target_os = "macos", test))]
fn belongs_to(process: &str, app: &str) -> bool {
    !app.is_empty()
        && process
            .strip_prefix(app)
            .is_some_and(|rest| rest.is_empty() || rest.starts_with('.'))
}

#[cfg(target_os = "macos")]
mod mac {
    use super::{Application, ApplicationList, ProcessIdentity, belongs_to};
    use crate::audio::tap::{get_ids, get_property, get_string};
    use objc2_app_kit::{NSApplicationActivationPolicy, NSRunningApplication, NSWorkspace};
    use objc2_core_audio::{
        AudioObjectID, kAudioHardwarePropertyProcessObjectList, kAudioObjectSystemObject,
        kAudioProcessPropertyBundleID, kAudioProcessPropertyPID,
    };

    /// The selected application, checked by its start time so that a recycled PID never stands
    /// in for it.
    pub struct ProcessGuard {
        identity: ProcessIdentity,
    }

    impl ProcessGuard {
        pub fn selected(identity: &ProcessIdentity) -> anyhow::Result<Self> {
            let guard = Self {
                identity: identity.clone(),
            };
            anyhow::ensure!(
                guard.running(),
                "selected application has closed; select it again"
            );
            Ok(guard)
        }

        /// Still the same process: alive, and started when it was when selected.
        pub fn running(&self) -> bool {
            started_at(self.identity.pid).as_deref() == Some(self.identity.created_at.as_str())
        }
    }

    /// When the process started, as `seconds.microseconds`, or `None` if there is no such
    /// process (or it is not ours to ask about).
    fn started_at(pid: u32) -> Option<String> {
        let pid = i32::try_from(pid).ok()?;
        let size = i32::try_from(size_of::<libc::proc_bsdinfo>()).ok()?;
        // SAFETY: plain data, for which all zeroes is a valid value.
        let mut info: libc::proc_bsdinfo = unsafe { std::mem::zeroed() };
        // SAFETY: `info` has room for `size` bytes, which is what PROC_PIDTBSDINFO writes.
        let written = unsafe {
            libc::proc_pidinfo(pid, libc::PROC_PIDTBSDINFO, 0, (&raw mut info).cast(), size)
        };
        (written == size).then(|| format!("{}.{:06}", info.pbi_start_tvsec, info.pbi_start_tvusec))
    }

    /// The applications with a Dock presence, apart from this one, by name.
    #[allow(
        clippy::unnecessary_wraps,
        reason = "the Windows build's signature, which can fail"
    )]
    pub fn list() -> anyhow::Result<ApplicationList> {
        let own = std::process::id();
        let mut applications = Vec::new();
        for app in NSWorkspace::sharedWorkspace().runningApplications().iter() {
            if app.activationPolicy() != NSApplicationActivationPolicy::Regular
                || app.isTerminated()
            {
                continue;
            }
            let Ok(pid) = u32::try_from(app.processIdentifier()) else {
                continue;
            };
            if pid == own {
                continue;
            }
            let Some(created_at) = started_at(pid) else {
                continue;
            };
            let name = app
                .localizedName()
                .map_or_else(|| format!("PID {pid}"), |name| name.to_string());
            applications.push(Application {
                process: ProcessIdentity { pid, created_at },
                name,
            });
        }
        applications.sort_by_key(|app| app.name.to_lowercase());
        Ok(ApplicationList {
            supported: true,
            applications,
        })
    }

    /// The Core Audio process objects to tap for the application with this PID: its own and
    /// its helpers'. A process appears only once it has used audio, so this is asked again
    /// while capturing.
    pub fn audio_processes(pid: u32) -> anyhow::Result<Vec<AudioObjectID>> {
        let bundle = i32::try_from(pid)
            .ok()
            .and_then(NSRunningApplication::runningApplicationWithProcessIdentifier)
            .and_then(|app| app.bundleIdentifier())
            .map(|id| id.to_string());
        let system = AudioObjectID::try_from(kAudioObjectSystemObject).unwrap_or(1);
        let mut objects = Vec::new();
        for object in get_ids(system, kAudioHardwarePropertyProcessObjectList)? {
            let own = get_property::<i32>(object, kAudioProcessPropertyPID)
                .ok()
                .and_then(|p| u32::try_from(p).ok())
                == Some(pid);
            let helper = bundle.as_deref().is_some_and(|app| {
                get_string(object, kAudioProcessPropertyBundleID)
                    .is_ok_and(|process| belongs_to(&process, app))
            });
            if own || helper {
                objects.push(object);
            }
        }
        objects.sort_unstable();
        Ok(objects)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn this_process_is_recognised_and_a_stale_identity_refused() {
            let pid = std::process::id();
            let created_at = started_at(pid).expect("this process has a start time");
            let guard = ProcessGuard::selected(&ProcessIdentity {
                pid,
                created_at: created_at.clone(),
            })
            .unwrap();
            assert!(guard.running());
            assert!(
                ProcessGuard::selected(&ProcessIdentity {
                    pid,
                    created_at: "0.000000".into()
                })
                .is_err()
            );
            // Listing must not fail on a machine with no window server session either.
            let _ = list().unwrap();
            // A process that is not playing anything simply has no objects.
            let _ = audio_processes(pid).unwrap();
        }
    }
}

#[cfg(windows)]
mod platform {
    use super::*;
    use windows_sys::Win32::Foundation::{
        CloseHandle, FILETIME, HANDLE, HWND, LPARAM, WAIT_TIMEOUT,
    };
    use windows_sys::Win32::System::Threading::{
        GetProcessTimes, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SYNCHRONIZE,
        WaitForSingleObject,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetWindowTextW, GetWindowThreadProcessId, IsWindowVisible,
    };

    pub struct ProcessGuard(HANDLE);
    impl Drop for ProcessGuard {
        fn drop(&mut self) {
            unsafe {
                CloseHandle(self.0);
            }
        }
    }
    impl ProcessGuard {
        fn open(pid: u32) -> anyhow::Result<Self> {
            anyhow::ensure!(
                pid != 0 && pid != std::process::id(),
                "invalid capture application"
            );
            let handle = unsafe {
                OpenProcess(
                    PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SYNCHRONIZE,
                    0,
                    pid,
                )
            };
            anyhow::ensure!(!handle.is_null(), "selected application is unavailable");
            Ok(Self(handle))
        }
        fn identity(&self, pid: u32) -> anyhow::Result<ProcessIdentity> {
            let mut created = FILETIME::default();
            let mut exit = FILETIME::default();
            let mut kernel = FILETIME::default();
            let mut user = FILETIME::default();
            anyhow::ensure!(
                unsafe { GetProcessTimes(self.0, &mut created, &mut exit, &mut kernel, &mut user) }
                    != 0,
                "cannot identify application"
            );
            Ok(ProcessIdentity {
                pid,
                created_at: ((u64::from(created.dwHighDateTime) << 32)
                    | u64::from(created.dwLowDateTime))
                .to_string(),
            })
        }
        pub fn selected(identity: &ProcessIdentity) -> anyhow::Result<Self> {
            anyhow::ensure!(
                supported(),
                "application capture requires Windows build 20348 or later"
            );
            let guard = Self::open(identity.pid)?;
            anyhow::ensure!(
                guard.identity(identity.pid)? == *identity && guard.running(),
                "selected application has closed or restarted; select it again"
            );
            Ok(guard)
        }
        pub fn running(&self) -> bool {
            unsafe { WaitForSingleObject(self.0, 0) == WAIT_TIMEOUT }
        }
    }
    fn supported() -> bool {
        use windows_sys::Win32::System::SystemInformation::OSVERSIONINFOW;
        #[expect(
            clippy::cast_possible_truncation,
            reason = "a Win32 structure of a few hundred bytes"
        )]
        let mut version = OSVERSIONINFOW {
            dwOSVersionInfoSize: std::mem::size_of::<OSVERSIONINFOW>() as u32,
            ..Default::default()
        };
        unsafe {
            windows_sys::Wdk::System::SystemServices::RtlGetVersion(&mut version) >= 0
                && version.dwBuildNumber >= 20348
        }
    }
    unsafe extern "system" fn visit(window: HWND, data: LPARAM) -> i32 {
        // SAFETY: `window` is a top-level window handle `EnumWindows` passes for this call.
        if unsafe { IsWindowVisible(window) } == 0 {
            return 1;
        }
        let mut title = [0u16; 512];
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_possible_wrap,
            reason = "the buffer is 512 units long"
        )]
        // SAFETY: the same handle, and `title` is writable for the length passed.
        let length = unsafe { GetWindowTextW(window, title.as_mut_ptr(), title.len() as i32) };
        // Zero for an untitled window or a failure; never documented as negative.
        let Some(length) = usize::try_from(length).ok().filter(|&n| n > 0) else {
            return 1;
        };
        let mut pid = 0;
        // SAFETY: the same handle, and `pid` is a valid place for the id.
        unsafe { GetWindowThreadProcessId(window, &mut pid) };
        // SAFETY: `data` is the `&mut Vec<Application>` that `list` passes to `EnumWindows`,
        // borrowed by nothing else until `EnumWindows` returns.
        let apps = unsafe { &mut *(data as *mut Vec<Application>) };
        if apps.iter().any(|app| app.process.pid == pid) {
            return 1;
        }
        if let Ok(guard) = ProcessGuard::open(pid)
            && let Ok(process) = guard.identity(pid)
            && guard.running()
        {
            apps.push(Application {
                process,
                name: String::from_utf16_lossy(&title[..length]),
            });
        }
        1
    }
    pub fn list() -> anyhow::Result<ApplicationList> {
        if !supported() {
            return Ok(ApplicationList {
                supported: false,
                applications: vec![],
            });
        }
        let mut applications = Vec::<Application>::new();
        anyhow::ensure!(
            unsafe { EnumWindows(Some(visit), &mut applications as *mut _ as LPARAM) } != 0,
            "cannot list applications"
        );
        applications.sort_by_key(|app| app.name.to_lowercase());
        Ok(ApplicationList {
            supported: true,
            applications,
        })
    }

    #[cfg(test)]
    #[test]
    fn native_process_client_starts_without_endpoint_queries_and_rejects_pid_reuse() {
        use std::os::windows::process::CommandExt;
        use std::process::{Command, Stdio};
        if !supported() {
            return;
        }
        struct Child(std::process::Child);
        impl Drop for Child {
            fn drop(&mut self) {
                let _ = self.0.kill();
                let _ = self.0.wait();
            }
        }
        let executable = std::path::PathBuf::from(std::env::var_os("SystemRoot").unwrap())
            .join("System32/cmd.exe");
        // A silent, hidden process owned by this test; no user's application is captured.
        let child = Child(
            Command::new(executable)
                .args(["/C", "pause"])
                .creation_flags(0x08000000)
                .stdin(Stdio::piped())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .unwrap(),
        );
        let guard = ProcessGuard::open(child.0.id()).unwrap();
        let identity = guard.identity(child.0.id()).unwrap();
        assert!(
            ProcessGuard::selected(&ProcessIdentity {
                created_at: "0".into(),
                ..identity.clone()
            })
            .is_err()
        );
        let cancel = tokio_util::sync::CancellationToken::new();
        cancel.cancel();
        let (levels, _) = tokio::sync::mpsc::channel(1);
        let (chunks, _) = tokio::sync::mpsc::channel(1);
        let result = std::thread::spawn(move || {
            crate::audio::loopback::run_system_loopback(
                None,
                SystemCapture::Application {
                    process: Some(identity),
                },
                16000,
                levels,
                chunks.into(),
                &cancel,
            )
        })
        .join()
        .unwrap();
        result.unwrap();
        drop(child);
        assert!(!guard.running());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn application_without_selection_never_means_output() {
        let target: SystemCapture =
            serde_json::from_str(r#"{"kind":"application","process":null}"#).unwrap();
        assert!(matches!(
            target,
            SystemCapture::Application { process: None }
        ));
        assert!(validate(&target).is_err());
        assert!(validate(&SystemCapture::Output).is_ok());
        assert!(serde_json::from_str::<SystemCapture>(r#"{"kind":"unknown"}"#).is_err());
    }
    #[test]
    fn helpers_belong_to_their_application_and_nothing_else_does() {
        assert!(belongs_to("com.google.Chrome", "com.google.Chrome"));
        assert!(belongs_to("com.google.Chrome.helper", "com.google.Chrome"));
        assert!(belongs_to(
            "com.google.Chrome.helper.Renderer",
            "com.google.Chrome"
        ));
        assert!(!belongs_to("com.google.Chromecast", "com.google.Chrome"));
        assert!(!belongs_to("com.apple.WebKit.GPU", "com.apple.Safari"));
        assert!(!belongs_to("anything", ""));
    }

    #[cfg(windows)]
    #[test]
    fn enumeration_and_stale_identity_are_safe() {
        let _ = list().unwrap();
        assert!(
            ProcessGuard::selected(&ProcessIdentity {
                pid: std::process::id(),
                created_at: "0".into()
            })
            .is_err()
        );
    }
}
