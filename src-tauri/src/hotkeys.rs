//! System-wide Pause and Show/hide overlay keys, opt-in.
//!
//! A presenter running slides full screen on the same computer cannot reach the operator
//! window's own shortcuts (`src/lib/shortcuts.ts`) without switching to it. With *Use shortcuts
//! while other apps are in front* on, two combinations are registered with the system: one
//! pauses or resumes the captions, one shows or hides the overlay. Never one that starts or
//! stops a session: a stray key press must not start something billable or end a meeting's
//! transcript.
//!
//! Held only while the switch is on and a session is running or paused, so an idle app holds no
//! key at all and another application can have the combination. The front-end pushes both
//! facts through `set_global_shortcuts` whenever either changes, as it pushes the tray's state,
//! and gets back which keys are held and which the system refused, for the Settings row.
//!
//! A key press does what the tray menu does: it emits a `tray-command`, and the operator
//! window carries it out through the same function its own shortcut and button call. So the
//! pause goes through `pause_session` and the session's lifecycle lock like any other, and the
//! button labels and the tray menu follow, because they read the state that function changes.
//!
//! Registered from Rust only, through `tauri-plugin-global-shortcut` (`RegisterHotKey` on
//! Windows, Carbon's `RegisterEventHotKey` on macOS). Neither window is granted the plugin's
//! own commands. The app is told about these combinations and nothing else: no keyboard hook,
//! no event tap, nothing about any other key.

use std::sync::Mutex;
use std::sync::atomic::{AtomicU8, Ordering};
use std::time::{Duration, Instant};

use serde::Serialize;
use tauri::plugin::TauriPlugin;
use tauri::{AppHandle, Manager, State, Wry};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutEvent, ShortcutState};

use crate::sync::lock;
use crate::tray::{self, TrayCommand};

/// A system-wide shortcut. Mirrors `GlobalShortcut` in `src/lib/types.ts`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum GlobalShortcut {
    TogglePause,
    ToggleOverlay,
}

/// Two presses of the same combination closer together than this are one press. Windows is
/// asked for no auto-repeat (`MOD_NOREPEAT`), but a quiet period also covers a system that
/// repeats and a key that bounces: every press restarts it, so holding the keys acts once.
const QUIET: Duration = Duration::from_millis(400);

impl GlobalShortcut {
    pub const ALL: [Self; 2] = [Self::TogglePause, Self::ToggleOverlay];

    /// The combination, in the plugin's syntax. The operator window's own P and O with Alt
    /// added (Option on a Mac), and Control rather than Command on a Mac: three modifiers that
    /// PowerPoint, Keynote, Zoom, Teams and the browsers leave alone, where Ctrl+Shift+P opens
    /// a private browser window and ⇧⌘P is taken in many Mac apps. `src/lib/globalShortcuts.ts`
    /// prints the same keys; `contract.test.ts` holds the two together.
    pub const fn accelerator(self) -> &'static str {
        match self {
            Self::TogglePause => "Control+Alt+Shift+KeyP",
            Self::ToggleOverlay => "Control+Alt+Shift+KeyO",
        }
    }

    fn shortcut(self) -> Result<Shortcut, String> {
        self.accelerator()
            .parse::<Shortcut>()
            .map_err(|error| error.to_string())
    }

    /// What the operator window is asked to do: the same request as the tray's.
    const fn command(self) -> TrayCommand {
        match self {
            Self::TogglePause => TrayCommand::TogglePause,
            Self::ToggleOverlay => TrayCommand::ToggleOverlay,
        }
    }

    const fn bit(self) -> u8 {
        match self {
            Self::TogglePause => 1,
            Self::ToggleOverlay => 2,
        }
    }

    fn of(shortcut: &Shortcut) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|key| key.shortcut().is_ok_and(|own| own == *shortcut))
    }
}

/// Whether the keys should be held: the switch on and a session running or paused.
pub const fn should_hold(enabled: bool, session_active: bool) -> bool {
    enabled && session_active
}

/// What the Settings row shows: the combinations held now, and the ones the system refused
/// when they were last asked for, usually because another application holds them.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct GlobalShortcutStatus {
    pub registered: Vec<GlobalShortcut>,
    pub failed: Vec<GlobalShortcut>,
}

/// The system's side of it. A trait so the bookkeeping is tested without a display server:
/// the tests never reach the plugin, X11 or a window message loop.
trait Registrar {
    fn register(&self, key: GlobalShortcut) -> Result<(), String>;
    fn unregister(&self, key: GlobalShortcut) -> Result<(), String>;
}

/// Bring the held keys to what is wanted. A refused key is reported, never fatal, and asked
/// for again at the next push. A key whose release the system refused is forgotten all the
/// same, so a press that still arrives does nothing; the system frees it when the app exits.
fn reconcile(
    held: &mut Vec<GlobalShortcut>,
    want: bool,
    os: &impl Registrar,
) -> GlobalShortcutStatus {
    let mut failed = Vec::new();
    if want {
        for key in GlobalShortcut::ALL {
            if held.contains(&key) {
                continue;
            }
            match os.register(key) {
                Ok(()) => held.push(key),
                Err(error) => {
                    tracing::warn!("could not register the system-wide {key:?} shortcut: {error}");
                    failed.push(key);
                }
            }
        }
    } else {
        for key in held.drain(..) {
            if let Err(error) = os.unregister(key) {
                tracing::warn!("could not release the system-wide {key:?} shortcut: {error}");
            }
        }
    }
    let registered = GlobalShortcut::ALL
        .into_iter()
        .filter(|key| held.contains(key))
        .collect();
    GlobalShortcutStatus { registered, failed }
}

/// The quiet period for each combination.
#[derive(Default)]
struct Debounce {
    last: [Option<Instant>; 2],
}

impl Debounce {
    /// Whether a press at `now` acts. Every press, acted on or not, restarts the quiet period.
    fn allow(&mut self, key: GlobalShortcut, now: Instant) -> bool {
        let slot = &mut self.last[usize::from(key.bit() - 1)];
        let quiet = slot.is_none_or(|last| now.saturating_duration_since(last) >= QUIET);
        *slot = Some(now);
        quiet
    }
}

/// The keys this app holds. `held` is the command's bookkeeping; `live` is what the key
/// handler reads, an atomic so a press never waits on, or deadlocks against, a registration
/// under way on the same main thread.
#[derive(Default)]
pub struct Hotkeys {
    held: Mutex<Vec<GlobalShortcut>>,
    live: AtomicU8,
    debounce: Mutex<Debounce>,
}

impl Hotkeys {
    fn apply(&self, want: bool, os: &impl Registrar) -> GlobalShortcutStatus {
        let mut held = lock(&self.held);
        if !want {
            // Before the system is asked, so a press racing the release does nothing.
            self.live.store(0, Ordering::Relaxed);
        }
        let status = reconcile(&mut held, want, os);
        let live = held.iter().fold(0, |mask, key| mask | key.bit());
        self.live.store(live, Ordering::Relaxed);
        status
    }

    fn holds(&self, key: GlobalShortcut) -> bool {
        self.live.load(Ordering::Relaxed) & key.bit() != 0
    }

    /// Whether a press should be acted on: one of ours, held, and not a repeat.
    fn accept(&self, key: GlobalShortcut, state: ShortcutState, now: Instant) -> bool {
        state == ShortcutState::Pressed && self.holds(key) && lock(&self.debounce).allow(key, now)
    }
}

/// The plugin, through which every registration goes.
struct Os<'a>(&'a AppHandle);

impl Registrar for Os<'_> {
    fn register(&self, key: GlobalShortcut) -> Result<(), String> {
        if self
            .0
            .try_state::<tauri_plugin_global_shortcut::GlobalShortcut<Wry>>()
            .is_none()
        {
            return Err("the global-shortcut plugin is not running".into());
        }
        self.0
            .global_shortcut()
            .register(key.shortcut()?)
            .map_err(|error| error.to_string())
    }

    fn unregister(&self, key: GlobalShortcut) -> Result<(), String> {
        if self
            .0
            .try_state::<tauri_plugin_global_shortcut::GlobalShortcut<Wry>>()
            .is_none()
        {
            return Err("the global-shortcut plugin is not running".into());
        }
        self.0
            .global_shortcut()
            .unregister(key.shortcut()?)
            .map_err(|error| error.to_string())
    }
}

/// The plugin with no shortcut of its own: nothing is registered until the front-end asks.
pub fn plugin() -> TauriPlugin<Wry> {
    tauri_plugin_global_shortcut::Builder::new()
        .with_handler(on_shortcut)
        .build()
}

fn on_shortcut(app: &AppHandle, shortcut: &Shortcut, event: ShortcutEvent) {
    let Some(key) = GlobalShortcut::of(shortcut) else {
        return;
    };
    if app
        .state::<Hotkeys>()
        .accept(key, event.state, Instant::now())
    {
        tray::emit(app, key.command());
    }
}

/// Let go of every key as the app exits. The system would free them with the process; this
/// makes it explicit, and keeps a slow shutdown from acting on a press.
pub fn release(app: &AppHandle) {
    if let Some(hotkeys) = app.try_state::<Hotkeys>() {
        hotkeys.apply(false, &Os(app));
    }
}

/// Hold or release the keys. Pushed by the operator window whenever the switch or the
/// session's state changes, and once on mount. Answers which keys are held and which the
/// system refused.
///
/// Synchronous, so it runs on the main thread, where macOS requires hot keys to be registered;
/// the plugin then makes the call directly rather than as a round trip.
#[tauri::command]
#[expect(
    clippy::needless_pass_by_value,
    reason = "Tauri commands receive their arguments by value"
)]
pub fn set_global_shortcuts(
    app: AppHandle,
    hotkeys: State<'_, Hotkeys>,
    enabled: bool,
    session_active: bool,
) -> GlobalShortcutStatus {
    hotkeys.apply(should_hold(enabled, session_active), &Os(&app))
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use super::*;

    /// A system that records each call and refuses the keys it is told to.
    #[derive(Default)]
    struct Fake {
        refuse: Vec<GlobalShortcut>,
        refuse_release: bool,
        calls: RefCell<Vec<(&'static str, GlobalShortcut)>>,
    }

    impl Registrar for Fake {
        fn register(&self, key: GlobalShortcut) -> Result<(), String> {
            self.calls.borrow_mut().push(("register", key));
            if self.refuse.contains(&key) {
                Err("already registered".into())
            } else {
                Ok(())
            }
        }

        fn unregister(&self, key: GlobalShortcut) -> Result<(), String> {
            self.calls.borrow_mut().push(("unregister", key));
            if self.refuse_release {
                Err("refused".into())
            } else {
                Ok(())
            }
        }
    }

    use GlobalShortcut::{ToggleOverlay, TogglePause};

    #[test]
    fn held_only_while_the_switch_is_on_and_a_session_runs() {
        assert!(should_hold(true, true));
        assert!(!should_hold(true, false));
        assert!(!should_hold(false, true));
        assert!(!should_hold(false, false));
    }

    /// Parsing is pure: no display server is needed to check the combinations are valid.
    #[test]
    fn every_combination_parses_to_control_alt_shift_and_a_letter() {
        use tauri_plugin_global_shortcut::{Code, Modifiers};
        let expected = [Code::KeyP, Code::KeyO];
        for (key, code) in GlobalShortcut::ALL.into_iter().zip(expected) {
            let shortcut = key.shortcut().expect("the accelerator parses");
            assert_eq!(
                shortcut,
                Shortcut::new(
                    Some(Modifiers::CONTROL | Modifiers::ALT | Modifiers::SHIFT),
                    code
                )
            );
            assert_eq!(GlobalShortcut::of(&shortcut), Some(key));
        }
        let other = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyP);
        assert_eq!(GlobalShortcut::of(&other), None);
    }

    /// Neither key starts or stops a session: each asks for what the tray's entries ask for.
    #[test]
    fn a_press_asks_for_pause_or_the_overlay_never_start_or_stop() {
        assert!(matches!(TogglePause.command(), TrayCommand::TogglePause));
        assert!(matches!(
            ToggleOverlay.command(),
            TrayCommand::ToggleOverlay
        ));
    }

    #[test]
    fn switching_on_registers_both_and_off_releases_both() {
        let hotkeys = Hotkeys::default();
        let os = Fake::default();
        let on = hotkeys.apply(true, &os);
        assert_eq!(on.registered, [TogglePause, ToggleOverlay]);
        assert!(on.failed.is_empty());
        assert!(hotkeys.holds(TogglePause) && hotkeys.holds(ToggleOverlay));

        let off = hotkeys.apply(false, &os);
        assert_eq!(off, GlobalShortcutStatus::default());
        assert!(!hotkeys.holds(TogglePause) && !hotkeys.holds(ToggleOverlay));
        assert_eq!(
            *os.calls.borrow(),
            [
                ("register", TogglePause),
                ("register", ToggleOverlay),
                ("unregister", TogglePause),
                ("unregister", ToggleOverlay),
            ]
        );
    }

    /// The front-end pushes on mount and on every change: a second push asks for nothing new.
    #[test]
    fn a_repeated_push_registers_nothing_twice() {
        let hotkeys = Hotkeys::default();
        let os = Fake::default();
        hotkeys.apply(true, &os);
        hotkeys.apply(true, &os);
        assert_eq!(os.calls.borrow().len(), 2);
        // And an idle app that was never on releases nothing.
        let idle = Hotkeys::default();
        let os = Fake::default();
        idle.apply(false, &os);
        assert!(os.calls.borrow().is_empty());
    }

    /// Another application holds one combination: the other still works, the refusal is
    /// reported, and it is asked for again at the next push.
    #[test]
    fn a_refused_key_is_reported_and_tried_again() {
        let hotkeys = Hotkeys::default();
        let os = Fake {
            refuse: vec![ToggleOverlay],
            ..Fake::default()
        };
        let status = hotkeys.apply(true, &os);
        assert_eq!(status.registered, [TogglePause]);
        assert_eq!(status.failed, [ToggleOverlay]);
        assert!(hotkeys.holds(TogglePause));
        assert!(!hotkeys.holds(ToggleOverlay));

        let freed = Fake::default();
        let status = hotkeys.apply(true, &freed);
        assert_eq!(status.registered, [TogglePause, ToggleOverlay]);
        assert!(status.failed.is_empty());
        assert_eq!(*freed.calls.borrow(), [("register", ToggleOverlay)]);
    }

    /// Off means off, even if the system will not say it let go.
    #[test]
    fn a_refused_release_still_stops_presses_from_acting() {
        let hotkeys = Hotkeys::default();
        let os = Fake {
            refuse_release: true,
            ..Fake::default()
        };
        hotkeys.apply(true, &os);
        let status = hotkeys.apply(false, &os);
        assert!(status.registered.is_empty());
        assert!(!hotkeys.accept(TogglePause, ShortcutState::Pressed, Instant::now()));
    }

    #[test]
    fn only_a_press_of_a_held_key_acts() {
        let hotkeys = Hotkeys::default();
        let now = Instant::now();
        assert!(!hotkeys.accept(TogglePause, ShortcutState::Pressed, now));
        hotkeys.apply(true, &Fake::default());
        assert!(!hotkeys.accept(TogglePause, ShortcutState::Released, now));
        assert!(hotkeys.accept(TogglePause, ShortcutState::Pressed, now));
    }

    /// Holding the keys, or a bounce, acts once; a deliberate second press acts again. Each
    /// combination has its own quiet period.
    #[test]
    fn repeats_inside_the_quiet_period_are_ignored() {
        let mut debounce = Debounce::default();
        let start = Instant::now();
        assert!(debounce.allow(TogglePause, start));
        // Auto-repeat every 33 ms for a second: none of it acts, because each one restarts
        // the quiet period.
        for step in 1..=30 {
            assert!(!debounce.allow(TogglePause, start + Duration::from_millis(33 * step)));
        }
        // The other combination is not held back by the first.
        assert!(debounce.allow(ToggleOverlay, start + Duration::from_millis(100)));
        let released = start + Duration::from_millis(33 * 30);
        assert!(debounce.allow(TogglePause, released + QUIET));
    }
}
