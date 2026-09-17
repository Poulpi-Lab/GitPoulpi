//! `gitpoulpi launch [-- <command>...]`: what every shortcut runs.
//!
//! A click on a GitPoulpi shortcut should show the window of a GitPoulpi that
//! is already open instead of piling up another terminal. So the shortcut
//! runs this first: it looks (over loopback, never starting a daemon) for a
//! live TUI window in any session, asks the most recently focused one to
//! bring its terminal window forward and exits. The window answers whether it
//! managed to (it cannot on every desktop); when there is none, the setting
//! `ui.reuse_window` is off, the daemon does not answer within a few hundred
//! milliseconds, or the window could not raise itself, the command after `--`
//! (the terminal that runs GitPoulpi, exactly what the shortcut used to be)
//! is started instead.
//!
//! The decision is pure ([`decide`], [`most_recent_window`]); only the
//! sockets and the spawn are I/O. On Windows the shortcut runs the
//! console-less `gitpoulpi-launch.exe` (`crates/gitpoulpi-cli/src/bin`),
//! which calls [`run`] with `quiet`, so a click never flashes a console.

use std::time::Duration;

use gitpoulpi_config::{ConfigPaths, Overrides, Settings};
use gitpoulpi_ipc::ui::{RaiseAnswer, ViewerInfo};
use serde_json::{json, Value};

/// How long the launcher waits for a daemon to list its windows. A click
/// must never feel slower than starting a terminal: past this the answer is
/// "no window".
pub const PROBE_TIMEOUT: Duration = Duration::from_millis(300);

/// A live window and where to reach its daemon.
#[derive(Debug, Clone, PartialEq)]
pub struct Window {
    pub session: String,
    pub port: u16,
    pub viewer: ViewerInfo,
}

/// What a launch does.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decision {
    /// Ask this window to bring itself forward.
    Raise,
    /// Run the command, because ...
    Start(StartReason),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartReason {
    /// `ui.reuse_window` is off.
    Disabled,
    /// No GitPoulpi window is open (or no daemon answered in time).
    NoWindow,
    /// A window is open but could not bring its terminal forward.
    CouldNotRaise,
    /// A new window was asked for: `--new-window`, or Shift held on the click.
    NewWindow,
}

impl StartReason {
    pub fn as_str(self) -> &'static str {
        match self {
            StartReason::Disabled => "disabled",
            StartReason::NoWindow => "no_window",
            StartReason::CouldNotRaise => "could_not_raise",
            StartReason::NewWindow => "new_window",
        }
    }
}

/// Reuse an open window, or start a new one: the whole policy. `new_window`
/// (`--new-window`, or Shift held when the shortcut was clicked) always wins:
/// it is the explicit "I want another one".
pub fn decide(reuse_window: bool, new_window: bool, windows: &[Window]) -> Decision {
    if new_window {
        Decision::Start(StartReason::NewWindow)
    } else if !reuse_window {
        Decision::Start(StartReason::Disabled)
    } else if windows.is_empty() {
        Decision::Start(StartReason::NoWindow)
    } else {
        Decision::Raise
    }
}

/// The window a click should show: the one that gained focus last, across
/// sessions; ties keep the order given (a daemon lists its own windows most
/// recently focused first).
pub fn most_recent_window(windows: &[Window]) -> Option<&Window> {
    let mut best: Option<&Window> = None;
    for w in windows {
        let newer = best.map_or(true, |b| {
            w.viewer.last_focus_at.unwrap_or(0) > b.viewer.last_focus_at.unwrap_or(0)
        });
        if newer {
            best = Some(w);
        }
    }
    best
}

/// Splits the arguments after `launch`: a leading `--` is dropped, the rest
/// is the command to start when no window is reused.
pub fn command_of(args: &[String]) -> Vec<String> {
    match args.split_first() {
        Some((first, rest)) if first == "--" => rest.to_vec(),
        _ => args.to_vec(),
    }
}

/// Whether a `GetAsyncKeyState` result says the key is down (the high bit).
#[cfg_attr(not(windows), allow(dead_code))]
fn key_down(state: i16) -> bool {
    state < 0
}

/// Windows: is Shift held right now? A click on the taskbar button or a
/// shortcut with Shift held means "a new window", like in every other app.
/// Read first thing, before anything slow lets the key go.
#[cfg(windows)]
fn shift_held() -> bool {
    #[link(name = "user32")]
    extern "system" {
        fn GetAsyncKeyState(vkey: i32) -> i16;
    }
    const VK_SHIFT: i32 = 0x10;
    // SAFETY: plain value argument, no pointers.
    key_down(unsafe { GetAsyncKeyState(VK_SHIFT) })
}

#[cfg(not(windows))]
fn shift_held() -> bool {
    false
}

/// For `gitpoulpi-launch`, which has no argument parser: a leading
/// `--new-window` (before the `--` that starts the command) and the rest.
pub fn split_new_window(args: &[String]) -> (bool, &[String]) {
    match args.split_first() {
        Some((first, rest)) if first == "--new-window" => (true, rest),
        _ => (false, args),
    }
}

/// `ui.reuse_window` from the resolved config; on, when it cannot be read.
fn reuse_window_setting(config: Option<&gitpoulpi_config::Config>) -> bool {
    config.map_or(true, |c| c.ui.reuse_window)
}

fn resolved_config() -> Option<gitpoulpi_config::Config> {
    Settings::resolve(ConfigPaths::discover().as_ref(), &Overrides::default())
        .ok()
        .map(|s| s.config().clone())
}

/// Dev mode with `ui.launcher_build = "newest"`: a click starts the most
/// recently built GitPoulpi of the checkout, decided now, not when the
/// shortcut was written. Every GitPoulpi program of `target/dist` or
/// `target/release` in `command` is swapped for its newest build
/// ([`gitpoulpi_config::dev_mode::resolve_command`]); anything else, and
/// everything outside newest mode, runs as written.
pub fn command_for_click(
    config: Option<&gitpoulpi_config::Config>,
    this_exe: &std::path::Path,
    command: Vec<String>,
) -> Vec<String> {
    match config {
        Some(c) if gitpoulpi_config::dev_mode::launcher_newest(c, this_exe) => {
            gitpoulpi_config::dev_mode::resolve_command(&command)
        }
        _ => command,
    }
}

/// Every live window of every session, from the port files. Reads only.
async fn open_windows() -> Vec<Window> {
    let mut windows = Vec::new();
    for found in crate::ui_launch::scan_sessions_within(None, PROBE_TIMEOUT).await {
        for viewer in found.viewers {
            windows.push(Window {
                session: found.session.clone(),
                port: found.port,
                viewer,
            });
        }
    }
    windows
}

async fn ask_to_raise(window: &Window) -> Option<RaiseAnswer> {
    gitpoulpi_daemon::join::raise_window(window.port, &window.viewer.id).await
}

/// Starts `command` detached from this process.
pub fn spawn_detached(command: &[String]) -> Result<(), String> {
    let (program, args) = command
        .split_first()
        .ok_or("nothing to start: give the command after `--`")?;
    let mut cmd = std::process::Command::new(program);
    cmd.args(args);
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        cmd.stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .process_group(0);
    }
    // Windows: the standard handles stay as they are. This process has no
    // console of its own (`gitpoulpi-launch.exe`), so a console program
    // (the plain-console shortcut) gets a fresh one, which a redirection to
    // NUL would break.
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0000_0200); // CREATE_NEW_PROCESS_GROUP
    }
    cmd.spawn()
        .map(|_| ())
        .map_err(|e| format!("could not start `{program}`: {e}"))
}

/// What happened, for the JSON line and the exit code.
#[derive(Debug, Clone, PartialEq)]
pub enum Outcome {
    Raised { session: String, viewer: String },
    Started { reason: StartReason },
    Failed(String),
}

impl Outcome {
    pub fn exit_code(&self) -> i32 {
        match self {
            Outcome::Raised { .. } | Outcome::Started { .. } => 0,
            Outcome::Failed(_) => 1,
        }
    }

    pub fn to_json(&self, command: &[String]) -> Value {
        match self {
            Outcome::Raised { session, viewer } => {
                json!({ "action": "raised", "session": session, "viewer": viewer })
            }
            Outcome::Started { reason } => {
                json!({ "action": "started", "reason": reason.as_str(), "command": command })
            }
            Outcome::Failed(message) => json!({ "action": "failed", "error": message }),
        }
    }
}

/// The launch itself. `reuse`: the setting, already read.
pub async fn launch(command: &[String], reuse: bool, new_window: bool) -> Outcome {
    let windows = if reuse && !new_window {
        open_windows().await
    } else {
        Vec::new()
    };
    let mut reason = match decide(reuse, new_window, &windows) {
        Decision::Raise => None,
        Decision::Start(reason) => Some(reason),
    };
    if reason.is_none() {
        let Some(window) = most_recent_window(&windows) else {
            return start(command, StartReason::NoWindow);
        };
        gitpoulpi_daemon::join::allow_foreground(window.viewer.pid);
        match ask_to_raise(window).await {
            Some(answer) if answer.raised => {
                return Outcome::Raised {
                    session: window.session.clone(),
                    viewer: window.viewer.id.to_string(),
                }
            }
            _ => reason = Some(StartReason::CouldNotRaise),
        }
    }
    start(command, reason.unwrap_or(StartReason::NoWindow))
}

fn start(command: &[String], reason: StartReason) -> Outcome {
    match spawn_detached(command) {
        Ok(()) => Outcome::Started { reason },
        Err(e) => Outcome::Failed(e),
    }
}

/// The entry point of `gitpoulpi launch` and `gitpoulpi-launch`: prints one
/// JSON line (`json`) or one plain line (neither when `quiet`) and returns
/// the exit code. `args` are the words after `launch`; `new_window` is
/// `--new-window` (Shift held on Windows counts too, read right here).
pub fn run(args: &[String], json: bool, quiet: bool, new_window: bool) -> i32 {
    let new_window = shift_held() || new_window;
    let this_exe = std::env::current_exe().unwrap_or_default();
    // Read once: both the command and the reuse setting come from it.
    let config = resolved_config();
    let command = command_for_click(config.as_ref(), &this_exe, command_of(args));
    let runtime = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            report(&Outcome::Failed(e.to_string()), &command, json, quiet);
            return 1;
        }
    };
    let outcome = runtime.block_on(launch(
        &command,
        reuse_window_setting(config.as_ref()),
        new_window,
    ));
    report(&outcome, &command, json, quiet);
    outcome.exit_code()
}

fn report(outcome: &Outcome, command: &[String], json: bool, quiet: bool) {
    if quiet {
        return;
    }
    if json {
        println!("{}", outcome.to_json(command));
        return;
    }
    match outcome {
        Outcome::Raised { session, .. } => println!("showed the open window (session {session})"),
        Outcome::Started { reason } => println!("started a new window ({})", reason.as_str()),
        Outcome::Failed(message) => eprintln!("gitpoulpi launch: {message}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitpoulpi_ipc::ui::ViewerId;

    fn fake_checkout(root: &std::path::Path) -> (std::path::PathBuf, std::path::PathBuf) {
        std::fs::create_dir_all(root.join("crates/gitpoulpi-tui")).unwrap();
        std::fs::write(root.join("Cargo.toml"), "").unwrap();
        let mk = |profile: &str, age: u64| {
            let dir = root.join("target").join(profile);
            std::fs::create_dir_all(&dir).unwrap();
            let p = dir.join("gitpoulpi-tui");
            let f = std::fs::File::create(&p).unwrap();
            f.set_modified(std::time::SystemTime::now() - Duration::from_secs(age))
                .unwrap();
            p
        };
        (mk("dist", 100), mk("release", 1))
    }

    #[test]
    fn a_click_starts_the_newest_build_only_in_dev_mode_with_newest() {
        let tmp = tempfile::tempdir().unwrap();
        let (dist, release) = fake_checkout(tmp.path());
        let launch = dist.with_file_name("gitpoulpi-launch");
        let command: Vec<String> = vec![
            "kitty".into(),
            dist.to_string_lossy().into(),
            "--launcher".into(),
        ];
        let mut config = gitpoulpi_config::Config::default();
        // Default: as written.
        assert_eq!(
            command_for_click(Some(&config), &launch, command.clone()),
            command
        );
        config.ui.launcher_build = gitpoulpi_config::LauncherBuild::Newest;
        let swapped = command_for_click(Some(&config), &launch, command.clone());
        assert_eq!(swapped[0], "kitty");
        assert_eq!(swapped[1], release.to_string_lossy());
        assert_eq!(swapped[2], "--launcher");
        // Dev mode explicitly off, or no config: as written.
        config.general.dev_mode = Some(false);
        assert_eq!(
            command_for_click(Some(&config), &launch, command.clone()),
            command
        );
        assert_eq!(command_for_click(None, &launch, command.clone()), command);
    }

    fn window(session: &str, id: &str, focus: Option<i64>) -> Window {
        Window {
            session: session.into(),
            port: 1,
            viewer: ViewerInfo {
                id: ViewerId(id.into()),
                pid: 7,
                active_repo: None,
                tabs: Vec::new(),
                focused: None,
                agent_control: true,
                last_focus_at: focus,
            },
        }
    }

    #[test]
    fn reuse_means_raise_only_when_enabled_and_a_window_is_open() {
        let open = [window("s", "a", None)];
        assert_eq!(decide(true, false, &open), Decision::Raise);
        assert_eq!(
            decide(false, false, &open),
            Decision::Start(StartReason::Disabled)
        );
        assert_eq!(
            decide(true, false, &[]),
            Decision::Start(StartReason::NoWindow)
        );
        assert_eq!(
            decide(false, false, &[]),
            Decision::Start(StartReason::Disabled)
        );
    }

    #[test]
    fn a_new_window_asked_for_beats_an_open_window() {
        let open = [window("s", "a", None)];
        let new = Decision::Start(StartReason::NewWindow);
        assert_eq!(decide(true, true, &open), new);
        assert_eq!(decide(true, true, &[]), new);
        assert_eq!(decide(false, true, &open), new);
        assert_eq!(StartReason::NewWindow.as_str(), "new_window");
    }

    #[test]
    fn the_shift_key_state_is_down_when_its_high_bit_is_set() {
        assert!(key_down(i16::MIN)); // 0x8000
        assert!(key_down(-32767)); // 0x8001: down and pressed since the last call
        assert!(!key_down(0));
        assert!(!key_down(1)); // pressed since the last call, up now
    }

    #[test]
    fn the_helper_reads_a_leading_new_window_flag() {
        let words = |a: &[&str]| a.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        let a = words(&["--new-window", "--", "wt.exe", "gp"]);
        let (flag, rest) = split_new_window(&a);
        assert!(flag);
        assert_eq!(command_of(rest), words(&["wt.exe", "gp"]));
        let b = words(&["--", "wt.exe", "--new-window"]);
        let (flag, rest) = split_new_window(&b);
        assert!(!flag, "after `--` it belongs to the command");
        assert_eq!(command_of(rest), words(&["wt.exe", "--new-window"]));
    }

    #[test]
    fn the_window_to_show_is_the_one_focused_last_across_sessions() {
        let all = [
            window("s1", "a", Some(100)),
            window("s2", "b", Some(300)),
            window("s2", "c", Some(200)),
        ];
        assert_eq!(most_recent_window(&all).unwrap().viewer.id.0, "b");
        // Never focused: the first listed (a daemon lists its newest first).
        let none = [window("s1", "x", None), window("s2", "y", None)];
        assert_eq!(most_recent_window(&none).unwrap().viewer.id.0, "x");
        assert!(most_recent_window(&[]).is_none());
    }

    #[test]
    fn the_command_is_what_follows_the_double_dash() {
        let words = |a: &[&str]| a.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert_eq!(
            command_of(&words(&["--", "kitty", "gp", "--launcher"])),
            words(&["kitty", "gp", "--launcher"])
        );
        assert_eq!(command_of(&words(&["kitty"])), words(&["kitty"]));
        assert!(command_of(&[]).is_empty());
        // Only the first `--` is the separator.
        assert_eq!(
            command_of(&words(&["--", "wezterm", "start", "--", "gp"])),
            words(&["wezterm", "start", "--", "gp"])
        );
    }

    #[test]
    fn outcomes_say_what_happened_and_failures_exit_nonzero() {
        let cmd = vec!["kitty".to_string()];
        let started = Outcome::Started {
            reason: StartReason::NoWindow,
        };
        assert_eq!(
            started.to_json(&cmd),
            json!({"action": "started", "reason": "no_window", "command": ["kitty"]})
        );
        assert_eq!(started.exit_code(), 0);
        let raised = Outcome::Raised {
            session: "s".into(),
            viewer: "tui-1".into(),
        };
        assert_eq!(
            raised.to_json(&cmd),
            json!({"action": "raised", "session": "s", "viewer": "tui-1"})
        );
        assert_eq!(Outcome::Failed("x".into()).exit_code(), 1);
    }

    #[test]
    fn starting_with_no_command_fails_instead_of_doing_nothing() {
        assert!(spawn_detached(&[]).is_err());
        let missing = vec!["/nonexistent/gitpoulpi-test-program".to_string()];
        assert!(spawn_detached(&missing).is_err());
    }
}
