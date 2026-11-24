//! Following the active tab's folder (`ui.tab_dir_follow`).
//!
//! A program cannot change its parent shell's directory, so GitPoulpi offers
//! two ways to get from the active tab to a shell in its folder:
//!
//! - **Quit and cd here**: the wrapper `gitpoulpi shell-init` prints runs
//!   `gitpoulpi --cwd-file <tmp>`; on quit this module writes the active
//!   tab's folder into that file and the wrapper `cd`s there. Only the
//!   button (or its palette twin) asks for it, unless
//!   `ui.tab_dir_follow = "auto"`, where every quit does.
//! - **Open terminal here**: a terminal or multiplexer pane running the
//!   user's own shell in the folder (`ui.terminal_launch`).
//!
//! The state is process-wide (statics, like the other `install_*` settings)
//! because the buttons are drawn and handled in places that have no config in
//! hand and the TUI restarts its window on every tab switch.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

use gitpoulpi_config::launch::{self, LaunchVars};
use gitpoulpi_config::open_in::{self, OpenInEntry};
use gitpoulpi_config::{TabDirFollow, UiConfig};
use gitpoulpi_i18n::msg;

use crate::external_tools;

/// The `quit_how` of a quit asked with "Quit and cd here": `finish` turns it
/// into [`request_quit_cd`] only when the window really ends (an unsaved
/// editor cancels the quit and clears `quit_how`).
pub(crate) const QUIT_CD_HOW: &str = "quit-and-cd-button";

/// The two controls.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TabDirButton {
    QuitCd,
    Terminal,
}

impl TabDirButton {
    /// The text on the tab bar.
    pub(crate) fn label(self) -> String {
        match self {
            TabDirButton::QuitCd => format!(" {} ", msg::tab_dir::QUIT_CD.get()),
            TabDirButton::Terminal => format!(" {} ", msg::tab_dir::TERMINAL.get()),
        }
    }
}

static CWD_FILE: OnceLock<PathBuf> = OnceLock::new();
static LAUNCH_DIRS: OnceLock<Vec<PathBuf>> = OnceLock::new();
static AUTO: AtomicBool = AtomicBool::new(false);
static QUIT_CD: AtomicBool = AtomicBool::new(false);
static TERMINAL: Mutex<TerminalSettings> = Mutex::new(TerminalSettings::new());

#[derive(Debug, Clone)]
struct TerminalSettings {
    launch: String,
    command: String,
    ui_launch: String,
    ui_launch_command: String,
}

impl TerminalSettings {
    const fn new() -> Self {
        Self {
            launch: String::new(),
            command: String::new(),
            ui_launch: String::new(),
            ui_launch_command: String::new(),
        }
    }
}

/// `--cwd-file`: where the TUI writes the folder on quit.
pub(crate) fn set_cwd_file(file: Option<PathBuf>) {
    if let Some(file) = file {
        let _ = CWD_FILE.set(file);
    }
}

pub(crate) fn has_cwd_file() -> bool {
    CWD_FILE.get().is_some()
}

fn normalized(p: &Path) -> PathBuf {
    std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf())
}

/// The folder GitPoulpi was started in, and the repository enclosing it: a tab
/// on either is "here" and gets no buttons.
pub(crate) fn set_launch_dir(cwd: &Path, repo_root: Option<&Path>) {
    let mut dirs = vec![normalized(cwd)];
    if let Some(root) = repo_root {
        dirs.push(normalized(root));
    }
    let _ = LAUNCH_DIRS.set(dirs);
}

static LAUNCH_NOTICE: Mutex<Option<String>> = Mutex::new(None);

/// A message the first window shows on its status line (a bad `--repo`).
pub(crate) fn set_launch_notice(message: String) {
    if let Ok(mut n) = LAUNCH_NOTICE.lock() {
        *n = Some(message);
    }
}

/// Takes the launch notice: shown once, by the first window.
pub(crate) fn take_launch_notice() -> Option<String> {
    LAUNCH_NOTICE.lock().ok().and_then(|mut n| n.take())
}

/// Whether `tab` is the launch folder (or its repository). With no launch
/// folder recorded nothing is "here".
pub(crate) fn is_launch_folder(tab: &Path) -> bool {
    let tab = normalized(tab);
    LAUNCH_DIRS.get().is_some_and(|dirs| dirs.contains(&tab))
}

/// Publishes the live settings: the follow mode and the terminal choice.
pub(crate) fn install(ui: &UiConfig) {
    AUTO.store(ui.tab_dir_follow == TabDirFollow::Auto, Ordering::Relaxed);
    DETECTED.warm(detect_machine);
    install_open_in(ui);
    if let Ok(mut t) = TERMINAL.lock() {
        *t = TerminalSettings {
            launch: ui.terminal_launch.clone(),
            command: ui.terminal_launch_command.clone(),
            ui_launch: ui.launch.clone(),
            ui_launch_command: ui.launch_command.clone(),
        };
    }
}

/// `ui.tab_dir_follow = "auto"`.
pub(crate) fn auto() -> bool {
    AUTO.load(Ordering::Relaxed)
}

/// The buttons the tab bar shows for the active tab at `path`.
///
/// "Open in" is on every repository tab, the launch folder's and `auto` mode
/// included (it opens editors and explorers too). "Quit & cd here" is hidden
/// in `auto` mode (every quit already cds) and on the launch folder (nothing
/// to cd to), and when GitPoulpi was not started through the wrapper
/// (`--cwd-file`): a button that can only say "set something up first" would
/// sit in the bar of everyone who never uses it; the palette entry stays and
/// explains the setup.
pub(crate) fn visible_buttons(path: Option<&Path>) -> Vec<TabDirButton> {
    let Some(path) = path else {
        return Vec::new();
    };
    let mut out = Vec::new();
    if has_cwd_file() && !auto() && !is_launch_folder(path) {
        out.push(TabDirButton::QuitCd);
    }
    out.push(TabDirButton::Terminal);
    out
}

/// "Quit and cd here" was asked: remember it for the quit that follows.
pub(crate) fn request_quit_cd() {
    QUIT_CD.store(true, Ordering::Relaxed);
}

/// On the final quit: writes `root` into `--cwd-file` when the user asked
/// for it (or `auto` is on). Nothing is written otherwise, so the wrapper
/// leaves the shell where it was.
pub(crate) fn write_cwd_on_quit(root: &Path) -> std::io::Result<bool> {
    let Some(file) = CWD_FILE.get() else {
        return Ok(false);
    };
    if !(QUIT_CD.load(Ordering::Relaxed) || auto()) {
        return Ok(false);
    }
    std::fs::write(file, root.to_string_lossy().as_bytes())?;
    Ok(true)
}

/// Starts the "Quit and cd here" the palette and the button share: the status
/// line's answer when the wrapper is missing, else `None` and the caller
/// quits.
pub(crate) fn quit_cd_blocked_reason() -> Option<&'static str> {
    (!has_cwd_file()).then(|| msg::tab_dir::SETUP_HINT.get())
}

/// What a button press decided.
pub(crate) enum Outcome {
    /// Say this on the status line.
    Message(String),
    /// End the window; the quit carries [`QUIT_CD_HOW`].
    QuitNow,
}

/// Runs `button` for the active tab at `path`. Shared by the tab bar's click
/// and the palette's twin, so the two cannot drift.
pub(crate) fn run(button: TabDirButton, path: Option<&Path>) -> Outcome {
    let Some(path) = path else {
        return Outcome::Message(msg::tab_dir::NO_TAB.get().into());
    };
    match button {
        TabDirButton::QuitCd => match quit_cd_blocked_reason() {
            Some(reason) => Outcome::Message(reason.to_string()),
            None => Outcome::QuitNow,
        },
        TabDirButton::Terminal => {
            Outcome::Message(open_terminal(path).unwrap_or_else(|error| error))
        }
    }
}

/// The template "Open terminal here" runs, or `None` for the platform's own
/// terminal.
fn terminal_template() -> Result<Option<String>, String> {
    let t = TERMINAL
        .lock()
        .map(|t| t.clone())
        .unwrap_or_else(|e| e.into_inner().clone());
    launch::resolve_terminal_template(&t.launch, &t.command, &t.ui_launch, &t.ui_launch_command)
}

/// The argv of "Open terminal here" for `path`, `None` when the platform's
/// own terminal applies. Split out so it is testable without spawning.
pub(crate) fn terminal_argv(path: &Path) -> Result<Option<Vec<String>>, String> {
    let Some(template) = terminal_template()? else {
        return Ok(None);
    };
    let repo = path.to_string_lossy().to_string();
    launch::expand(
        &template,
        &LaunchVars {
            repo: &repo,
            session: "",
            gitpoulpi: "",
        },
    )
    .map(Some)
}

/// Opens a terminal or pane in `path`, running the user's shell. Never waits
/// for the terminal itself: a launcher that fails at once (tmux with no
/// server) is reported later on the status line, a program that cannot be
/// found right away.
pub(crate) fn open_terminal(path: &Path) -> Result<String, String> {
    if !path.exists() {
        return Err(msg::tab_dir::gone_from_disk(path.display()));
    }
    let (program, args) = match terminal_argv(path)? {
        Some(argv) => (argv[0].clone(), argv[1..].to_vec()),
        None => crate::repo_menu::terminal_command(
            path,
            crate::repo_menu::on_path,
            std::env::var("TERMINAL").ok(),
        ),
    };
    let label = msg::tab_dir::TERMINAL.get().trim().to_string();
    let launch = Launch {
        label: &label,
        program: &program,
        args: &args,
        raw_args: None,
        cwd: path,
        info: &launch_repo_info(path),
    };
    start(&launch, external_tools::report_late_error)?;
    Ok(msg::tab_dir::opened_terminal(path.display()))
}

// ---------------------------------------------------------------- Open in
//
// The "Open in" launcher (see the open-in design): the list,
// the facts cache and the runner. The tab bar still shows the old "Terminal
// here" button; the chrome slice switches it to these.

/// The list `ui.open_in` configures, or, while the key is absent, the list
/// computed from the legacy terminal keys and what this machine runs.
#[derive(Debug, Clone, Default)]
struct OpenInState {
    entries: Vec<OpenInEntry>,
    default_label: String,
}

static OPEN_IN: Mutex<OpenInState> = Mutex::new(OpenInState {
    entries: Vec::new(),
    default_label: String::new(),
});
static DETECTED: Warmable<open_in::Detected> = Warmable::new();

/// A value computed once per process that can be asked for without waiting:
/// `warm` starts the computation on a background thread, `get` answers only
/// once it is done, `get_or_init` is the blocking form. One computation
/// whichever way it is reached.
pub(crate) struct Warmable<T> {
    cell: OnceLock<T>,
    started: std::sync::atomic::AtomicBool,
}

impl<T: Send + Sync + 'static> Warmable<T> {
    pub(crate) const fn new() -> Self {
        Self {
            cell: OnceLock::new(),
            started: std::sync::atomic::AtomicBool::new(false),
        }
    }

    /// The value, when it is already there. Never waits.
    pub(crate) fn get(&self) -> Option<&T> {
        self.cell.get()
    }

    pub(crate) fn get_or_init(&self, init: impl FnOnce() -> T) -> &T {
        self.cell.get_or_init(init)
    }

    /// Starts computing on a background thread, once.
    pub(crate) fn warm(&'static self, init: fn() -> T) {
        if !self.started.swap(true, Ordering::AcqRel) {
            std::thread::spawn(move || {
                self.cell.get_or_init(init);
            });
        }
    }
}

fn detect_machine() -> open_in::Detected {
    open_in::detect(
        &|k| std::env::var(k).ok(),
        open_in::Os::current(),
        &crate::repo_menu::path_lookup(),
    )
}

/// What the machine runs, looked up once per process: environment variables
/// and `PATH` (each directory listed once), no spawn. Waits for the lookup.
fn detected() -> &'static open_in::Detected {
    DETECTED.get_or_init(detect_machine)
}

/// What the machine runs, if the background lookup (started by `install`)
/// has finished. The Settings page asks this one, so it never waits for a
/// `PATH` listing on the input thread.
pub(crate) fn detected_ready() -> Option<&'static open_in::Detected> {
    DETECTED.get()
}

/// The effective list without waiting for detection: until it is ready the
/// detected entries are simply not in it yet.
pub(crate) fn effective_entries_ready(ui: &UiConfig) -> Vec<OpenInEntry> {
    entries_with(
        ui,
        detected_ready().unwrap_or(&open_in::Detected::default()),
    )
}

/// The effective list for `ui`: its own `open_in`, else the migration of the
/// legacy keys plus detection.
pub(crate) fn effective_entries(ui: &UiConfig) -> Vec<OpenInEntry> {
    entries_with(ui, detected())
}

fn entries_with(ui: &UiConfig, detected: &open_in::Detected) -> Vec<OpenInEntry> {
    if let Some(list) = &ui.open_in {
        return list.clone();
    }
    let os = open_in::Os::current();
    let legacy =
        if ui.terminal_launch.trim().is_empty() || ui.terminal_launch == launch::TERMINAL_AUTO {
            None
        } else {
            open_in::migrate_terminal(
                &ui.terminal_launch,
                &ui.terminal_launch_command,
                &ui.launch,
                &ui.launch_command,
                os,
            )
            .ok()
            .flatten()
        };
    open_in::effective_list(legacy, detected, os)
}

fn install_open_in(ui: &UiConfig) {
    let state = OpenInState {
        entries: effective_entries(ui),
        default_label: ui.open_in_default.clone(),
    };
    if let Ok(mut s) = OPEN_IN.lock() {
        *s = state;
    }
}

fn open_in_state() -> OpenInState {
    OPEN_IN
        .lock()
        .map(|s| s.clone())
        .unwrap_or_else(|e| e.into_inner().clone())
}

/// The labels of the "Open in" entries, in order.
#[allow(dead_code)] // the chrome slice draws the dropdown from this
pub(crate) fn entries() -> Vec<String> {
    open_in_state()
        .entries
        .into_iter()
        .map(|e| e.label)
        .collect()
}

/// The "Open in" entries that may open a worktree (the entry's "Can open a
/// worktree" box), as (index, label): what a worktree row's Open in menu lists.
#[cfg(feature = "worktree")]
pub(crate) fn worktree_entries() -> Vec<(usize, String)> {
    open_in_state()
        .entries
        .iter()
        .enumerate()
        .filter(|(_, e)| e.can_open_worktree())
        .map(|(i, e)| (i, e.label.clone()))
        .collect()
}

/// The index of the default entry (`ui.open_in_default`, else the first).
#[allow(dead_code)] // the chrome slice marks it and runs it from the palette
pub(crate) fn default_index() -> usize {
    let s = open_in_state();
    open_in::default_index(&s.entries, &s.default_label)
}

// ------------------------------------------------------ repository facts cache
//
// `${repo}` and `${branch}` come from here, never from a git call on the run
// path: the facts are read from the `.git` files (no process spawned) and
// cached per tab folder. The chrome calls [`tab_activated`] when a tab becomes
// the active one and [`head_changed`] when its HEAD moves; a run that finds the
// cache empty reads the two tiny files itself (microseconds) and stores them.

/// What `${repo}` and `${branch}` stand for on one tab.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct RepoInfo {
    /// The repository root (differs from the tab folder for a worktree).
    pub(crate) repo: String,
    /// The checked-out branch, empty when detached or unknown.
    pub(crate) branch: String,
    /// The origin's web page when it is a GitHub / GitLab project, else empty.
    pub(crate) web_url: String,
}

static REPO_INFO: Mutex<BTreeMap<PathBuf, RepoInfo>> = Mutex::new(BTreeMap::new());

/// Removes `.`/`..` components without touching the disk.
fn lexical_normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for c in path.components() {
        match c {
            std::path::Component::ParentDir => {
                out.pop();
            }
            std::path::Component::CurDir => {}
            other => out.push(other.as_os_str()),
        }
    }
    out
}

/// Reads the facts from the files under `.git`: a plain repository, or a
/// linked worktree whose `.git` is a `gitdir: ...` file.
pub(crate) fn read_repo_info(path: &Path) -> RepoInfo {
    let folder = path.to_string_lossy().to_string();
    let dot_git = path.join(".git");
    let (git_dir, repo) = if dot_git.is_dir() {
        (dot_git, folder.clone())
    } else if let Ok(text) = std::fs::read_to_string(&dot_git) {
        let Some(dir) = text.trim().strip_prefix("gitdir:").map(str::trim) else {
            return RepoInfo {
                repo: folder,
                ..RepoInfo::default()
            };
        };
        let git_dir = lexical_normalize(&path.join(dir));
        // `commondir` points from `.git/worktrees/<n>` back to the shared
        // `.git`; the repository root is its parent.
        let repo = std::fs::read_to_string(git_dir.join("commondir"))
            .ok()
            .map(|c| lexical_normalize(&git_dir.join(c.trim())))
            .and_then(|common| common.parent().map(|p| p.to_string_lossy().to_string()))
            .unwrap_or_else(|| folder.clone());
        (git_dir, repo)
    } else {
        return RepoInfo {
            repo: folder,
            ..RepoInfo::default()
        };
    };
    let branch = std::fs::read_to_string(git_dir.join("HEAD"))
        .ok()
        .and_then(|head| {
            head.trim()
                .strip_prefix("ref:")
                .map(|r| r.trim().trim_start_matches("refs/heads/").to_string())
        })
        .unwrap_or_default();
    let common = std::fs::read_to_string(git_dir.join("commondir"))
        .ok()
        .map(|c| lexical_normalize(&git_dir.join(c.trim())))
        .unwrap_or_else(|| git_dir.clone());
    let web_url = origin_url(&common.join("config"))
        .and_then(|url| web_url_of(&url))
        .unwrap_or_default();
    RepoInfo {
        repo,
        branch,
        web_url,
    }
}

/// The `url` of `[remote "origin"]` in a git config file.
fn origin_url(config: &Path) -> Option<String> {
    let text = std::fs::read_to_string(config).ok()?;
    let mut in_origin = false;
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_origin = line.replace(' ', "") == "[remote\"origin\"]"; // i18n-english
        } else if in_origin {
            if let Some(v) = line.strip_prefix("url") {
                if let Some(v) = v.trim_start().strip_prefix('=') {
                    return Some(v.trim().to_string());
                }
            }
        }
    }
    None
}

/// `https://host/owner/repo` for a GitHub or GitLab (well-known or configured
/// as an integration) remote URL; `None` for any other host.
fn web_url_of(remote_url: &str) -> Option<String> {
    let (base, project) = gitpoulpi_hosting::remote_web_base(remote_url)?;
    #[cfg(feature = "hosting")]
    let configured = gitpoulpi_hosting::resolve_for_repo(
        &[gitpoulpi_hosting::ResolveRemote::new(
            "origin".to_string(),
            remote_url.to_string(),
        )],
        &crate::pull_request::load_integrations(),
        None,
    )
    .is_some();
    #[cfg(not(feature = "hosting"))]
    let configured = crate::pull_request::integration_for_url(
        remote_url,
        &crate::pull_request::load_integrations(),
    )
    .is_some();
    let known = gitpoulpi_hosting::well_known_kind(&base).is_some() || configured;
    known.then(|| format!("{base}/{project}"))
}

/// The cached facts for the tab at `path`, read now when not cached yet.
pub(crate) fn repo_info(path: &Path) -> RepoInfo {
    if let Some(info) = REPO_INFO
        .lock()
        .ok()
        .and_then(|cache| cache.get(path).cloned())
    {
        return info;
    }
    let info = read_repo_info(path);
    if let Ok(mut cache) = REPO_INFO.lock() {
        cache.insert(path.to_path_buf(), info.clone());
    }
    info
}

/// The facts a launch substitutes (`${repo}`, `${branch}`), read fresh:
/// nothing calls `head_changed` yet, and a launch is rare while reading
/// `.git/HEAD` is a couple of small files (no git process).
pub(crate) fn launch_repo_info(path: &Path) -> RepoInfo {
    invalidate_repo_info(path);
    repo_info(path)
}

/// Forgets the cached facts of `path` (its HEAD moved).
pub(crate) fn invalidate_repo_info(path: &Path) {
    if let Ok(mut cache) = REPO_INFO.lock() {
        cache.remove(path);
    }
}

/// Recomputes the facts of `path` off the UI thread.
#[allow(dead_code)] // the chrome calls it when a tab activates / HEAD moves
pub(crate) fn tab_activated(path: &Path) {
    let path = path.to_path_buf();
    std::thread::spawn(move || {
        let info = read_repo_info(&path);
        if let Ok(mut cache) = REPO_INFO.lock() {
            cache.insert(path, info);
        }
    });
}

/// HEAD of the tab at `path` moved (checkout, branch switch): the cached
/// branch is stale.
#[allow(dead_code)] // the chrome calls it when the daemon reports a HEAD change
pub(crate) fn head_changed(path: &Path) {
    invalidate_repo_info(path);
    tab_activated(path);
}

// ------------------------------------------------------------------ the runner

/// How long the watcher thread waits for an early failure.
const WATCH: Duration = Duration::from_millis(1500);
/// How much of the launcher's stderr is kept.
const STDERR_KEEP: usize = 300;

/// One launch: what to run, in which folder, with which facts.
pub(crate) struct Launch<'a> {
    pub(crate) label: &'a str,
    pub(crate) program: &'a str,
    pub(crate) args: &'a [String],
    /// The command line after the program, passed verbatim on Windows when a
    /// program parses it itself (`--single-argument`); `args` otherwise.
    pub(crate) raw_args: Option<&'a str>,
    pub(crate) cwd: &'a Path,
    pub(crate) info: &'a RepoInfo,
}

fn program_stem(program: &str) -> String {
    let name = program.rsplit(['/', '\\']).next().unwrap_or(program);
    let lower = name.to_ascii_lowercase();
    lower.strip_suffix(".exe").unwrap_or(&lower).to_string()
}

/// `cmd /C start ...` opens its own window: it must not get
/// `CREATE_NO_WINDOW`.
#[cfg_attr(not(windows), allow(dead_code))] // only Windows spawns with it
fn is_cmd_start(program: &str, args: &[String]) -> bool {
    program_stem(program) == "cmd"
        && args
            .iter()
            .skip(1)
            .find(|a| !a.starts_with('/') || a.len() > 2)
            .is_some_and(|a| a.eq_ignore_ascii_case("start"))
}

/// Whether `program args` is a bare console shell (`cmd`, `pwsh`,
/// `powershell`, `wsl [--cd <dir>]`, `cmd /K <command>`): on Windows it is
/// started with `CREATE_NEW_CONSOLE` so it opens a window of its own in the
/// launch directory. Anything with other arguments (`cmd /C dir`) is a
/// command to run, not a terminal to show.
pub(crate) fn wants_new_console(program: &str, args: &[String]) -> bool {
    match program_stem(program).as_str() {
        "cmd" => args.is_empty() || args[0].eq_ignore_ascii_case("/K"),
        "pwsh" | "powershell" => args.is_empty(),
        "wsl" => args.is_empty() || (args.len() == 2 && args[0].eq_ignore_ascii_case("--cd")),
        _ => false,
    }
}

/// The late-error message for a launcher that ended with `code`, or `None`
/// when that is a success. `explorer.exe` always exits 1 even when it worked.
pub(crate) fn exit_failure(
    label: &str,
    program: &str,
    code: Option<i32>,
    stderr: &[u8],
) -> Option<String> {
    let Some(code) = code else {
        return Some(msg::open_in_run::stopped(label));
    };
    if code == 0 || (code == 1 && program_stem(program) == "explorer") {
        return None;
    }
    let text = String::from_utf8_lossy(stderr);
    let detail = text
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .unwrap_or_default();
    Some(if detail.is_empty() {
        msg::open_in_run::failed_silent(label, code)
    } else {
        msg::open_in_run::failed(label, code, detail)
    })
}

/// Spawns the launcher and returns at once. Immediate errors: the program is
/// not found, or the system refuses to start it. Later ones (a non-zero exit
/// within [`WATCH`]) go to `report`; the child is reaped either way.
pub(crate) fn start(launch: &Launch, report: fn(String)) -> Result<(), String> {
    use std::process::{Command, Stdio};
    let resolved = crate::repo_menu::resolve_program(
        launch.program,
        std::env::var_os("PATH").as_deref(),
        std::env::var("PATHEXT").ok().as_deref(),
        cfg!(windows),
        launch.cwd,
    )
    .ok_or_else(|| msg::open_in_run::program_not_found(launch.label, launch.program))?;
    let mut command = Command::new(&resolved);
    command.current_dir(launch.cwd);
    match launch.raw_args {
        #[cfg(windows)]
        Some(raw) => {
            use std::os::windows::process::CommandExt;
            command.raw_arg(raw);
        }
        _ => {
            command.args(launch.args);
        }
    }
    command
        .env("GITPOULPI_CWD", launch.cwd)
        .env("GITPOULPI_REPO", &launch.info.repo)
        .env("GITPOULPI_BRANCH", &launch.info.branch);
    // A console shell opened "here" gets its own console window, with the
    // folder as its working directory: no `cmd /C start` in between, which
    // would parse the folder name as a command line.
    let own_console = cfg!(windows) && wants_new_console(launch.program, launch.args);
    if own_console {
        // Null handles would be kept by the new console and end the shell at
        // once; inherited console handles are replaced by the new console's.
        command
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());
    } else {
        command
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped());
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        if own_console {
            command.creation_flags(0x0000_0010); // CREATE_NEW_CONSOLE
        } else if !is_cmd_start(launch.program, launch.args) {
            command.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
        }
    }
    let mut child = command
        .spawn()
        .map_err(|err| msg::open_in_run::could_not_start(launch.label, launch.program, err))?;

    // Keep the first bytes of stderr, drain the rest so the child never blocks
    // on a full pipe (a GUI program holds it open for as long as it lives).
    let captured = Arc::new(Mutex::new(Vec::<u8>::new()));
    let drained = Arc::new(AtomicBool::new(false));
    if let Some(mut stderr) = child.stderr.take() {
        let (captured, drained) = (captured.clone(), drained.clone());
        std::thread::spawn(move || {
            use std::io::Read;
            let mut buf = [0u8; 4096];
            while let Ok(n) = stderr.read(&mut buf) {
                if n == 0 {
                    break;
                }
                if let Ok(mut kept) = captured.lock() {
                    let room = STDERR_KEEP.saturating_sub(kept.len());
                    kept.extend_from_slice(&buf[..n.min(room)]);
                }
            }
            drained.store(true, Ordering::Relaxed);
        });
    } else {
        drained.store(true, Ordering::Relaxed);
    }

    let (label, program) = (launch.label.to_string(), launch.program.to_string());
    std::thread::spawn(move || {
        let deadline = Instant::now() + WATCH;
        loop {
            match child.try_wait() {
                Ok(Some(status)) => {
                    // Give the stderr reader a moment to see the end.
                    let until = Instant::now() + Duration::from_millis(150);
                    while !drained.load(Ordering::Relaxed) && Instant::now() < until {
                        std::thread::sleep(Duration::from_millis(10));
                    }
                    let kept = captured.lock().map(|k| k.clone()).unwrap_or_default();
                    if let Some(message) = exit_failure(&label, &program, status.code(), &kept) {
                        report(message);
                    }
                    return;
                }
                Ok(None) if Instant::now() < deadline => {
                    std::thread::sleep(Duration::from_millis(20));
                }
                Ok(None) => break,
                Err(_) => return,
            }
        }
        // Still running: a terminal or editor that stays up. Reap it
        // whenever it ends.
        let _ = child.wait();
    });
    Ok(())
}

/// How `gitpoulpi_config::open_in::expand` words an unknown variable.
const UNKNOWN_VARIABLE_PREFIX: &str = "unknown variable "; // i18n-english

/// A run, prepared: the entry, its expanded argv and the facts.
struct Prepared {
    label: String,
    argv: Vec<String>,
    info: RepoInfo,
}

fn prepare(index: usize, path: &Path) -> Result<Prepared, String> {
    let state = open_in_state();
    let entry = state
        .entries
        .get(index)
        .ok_or_else(|| msg::open_in_run::no_entry(index + 1))?;
    let label = entry.label.clone();
    let info = launch_repo_info(path);
    let cwd = path.to_string_lossy().to_string();
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    // A saved config may still hold an old `cmd /C start` preset text.
    let command = open_in::modernized_command(&entry.command).unwrap_or(&entry.command);
    let argv = open_in::expand(
        command,
        &open_in::OpenInVars {
            cwd: &cwd,
            repo: &info.repo,
            name: &name,
            branch: &info.branch,
            web_url: &info.web_url,
        },
        &|k| std::env::var(k).ok(),
    )
    .map_err(|e| match e.strip_prefix(UNKNOWN_VARIABLE_PREFIX) {
        Some(var) => msg::open_in_run::unknown_variable(&label, var),
        None => msg::open_in_run::bad_command(&label, e),
    })?;
    if info.web_url.is_empty() && entry.command.contains("${web_url}") {
        return Err(msg::open_in_run::no_web_url(&label));
    }
    Ok(Prepared { label, argv, info })
}

/// The argv entry `index` runs for `path`, without spawning.
#[cfg(test)]
pub(crate) fn open_in_argv(index: usize, path: &Path) -> Result<Vec<String>, String> {
    prepare(index, path).map(|p| p.argv)
}

/// Runs "Open in" entry `index` for the tab at `path`: the status line's
/// answer, at once ("Opening in <label>..."). Errors that can be known before
/// spawning come back as `Err`; a launcher that fails afterwards reports
/// through `external_tools::take_late_errors`. Never waits, never calls git.
#[allow(dead_code)] // the chrome and palette slices call it
pub(crate) fn run_open_in(index: usize, path: &Path) -> Result<String, String> {
    run_open_in_with(index, path, external_tools::report_late_error)
}

fn run_open_in_with(index: usize, path: &Path, report: fn(String)) -> Result<String, String> {
    let label = open_in_state()
        .entries
        .get(index)
        .map(|e| e.label.clone())
        .ok_or_else(|| msg::open_in_run::no_entry(index + 1))?;
    if !path.is_dir() {
        return Err(msg::open_in_run::folder_gone(&label, path.display()));
    }
    let run = prepare(index, path)?;
    start(
        &Launch {
            label: &run.label,
            program: &run.argv[0],
            args: &run.argv[1..],
            raw_args: None,
            cwd: path,
            info: &run.info,
        },
        report,
    )?;
    Ok(msg::open_in_run::opening(&run.label))
}

/// The position of the entry labelled `label` (trimmed, exact, first match):
/// the label is the stable name, the position changes when the user
/// reorders the list.
pub(crate) fn index_of_label(label: &str) -> Result<usize, String> {
    let wanted = label.trim();
    open_in_state()
        .entries
        .iter()
        .position(|e| e.label.trim() == wanted)
        .ok_or_else(|| msg::open_in_run::no_label(wanted))
}

/// "Tab: Open In" with an entry's label (a sequence's `label` option): runs
/// that entry on the tab at `path`. An unknown label is an error naming it.
pub(crate) fn open_in_by_label(label: &str, path: &Path) -> Result<String, String> {
    run_open_in(index_of_label(label)?, path)
}

/// `ui.tab_dir_follow = "auto"`: the entry a brand-new tab runs, if any.
pub(crate) fn auto_target() -> Option<usize> {
    (auto() && !open_in_state().entries.is_empty()).then(default_index)
}

/// Runs the default entry on a new tab when `auto` is on. The window restarts
/// on the new tab, so a failure surfaces through the late-error queue.
pub(crate) fn auto_open(path: &Path) -> Option<Result<String, String>> {
    auto_target().map(|index| run_open_in(index, path))
}

/// The statics are process-wide: unit tests that publish settings through
/// them (here and in `preferences`) take turns.
#[cfg(test)]
pub(crate) static TEST_SERIAL: Mutex<()> = Mutex::new(());

#[cfg(test)]
pub(crate) fn reset_for_tests() {
    AUTO.store(false, Ordering::Relaxed);
    QUIT_CD.store(false, Ordering::Relaxed);
    if let Ok(mut t) = TERMINAL.lock() {
        *t = TerminalSettings::new();
    }
    if let Ok(mut s) = OPEN_IN.lock() {
        *s = OpenInState::default();
    }
    if let Ok(mut c) = REPO_INFO.lock() {
        c.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::TEST_SERIAL as SERIAL;
    use super::*;

    static SLOW: Warmable<u32> = Warmable::new();
    static RUNS: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

    fn slow_value() -> u32 {
        RUNS.fetch_add(1, Ordering::SeqCst);
        std::thread::sleep(Duration::from_millis(300));
        7
    }

    #[test]
    fn warm_returns_at_once_the_value_arrives_later_and_is_computed_once() {
        let t0 = Instant::now();
        SLOW.warm(slow_value);
        SLOW.warm(slow_value);
        assert!(t0.elapsed() < Duration::from_millis(100), "warm blocked");
        assert_eq!(SLOW.get(), None, "not ready yet");
        // The blocking form shares the one computation (whoever got there first).
        assert_eq!(*SLOW.get_or_init(slow_value), 7);
        assert_eq!(SLOW.get(), Some(&7));
        assert_eq!(RUNS.load(Ordering::SeqCst), 1);
    }

    fn ui(follow: TabDirFollow, terminal: &str, command: &str) -> UiConfig {
        UiConfig {
            tab_dir_follow: follow,
            terminal_launch: terminal.into(),
            terminal_launch_command: command.into(),
            ..UiConfig::default()
        }
    }

    #[test]
    fn web_url_comes_from_a_github_or_gitlab_origin_only() {
        let dir = tempfile::tempdir().unwrap();
        let git = dir.path().join(".git");
        std::fs::create_dir(&git).unwrap();
        let write = |url: &str| {
            std::fs::write(
                git.join("config"),
                format!("[core]\n\tbare = false\n[remote \"upstream\"]\n\turl = https://github.com/x/y.git\n[remote \"origin\"]\n\turl = {url}\n"),
            )
            .unwrap();
            read_repo_info(dir.path()).web_url
        };
        assert_eq!(write("git@github.com:o/r.git"), "https://github.com/o/r");
        assert_eq!(
            write("https://gitlab.com/g/s/p.git"),
            "https://gitlab.com/g/s/p"
        );
        assert_eq!(write("https://example.org/o/r.git"), "");
        assert_eq!(write("/srv/git/r.git"), "");
    }

    #[test]
    fn open_in_shows_everywhere_and_quit_cd_keeps_its_hiding_rules() {
        let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
        reset_for_tests();
        let here = tempfile::tempdir().unwrap();
        let other = tempfile::tempdir().unwrap();
        set_launch_dir(here.path(), None);
        install(&ui(TabDirFollow::Buttons, "auto", ""));
        // No --cwd-file in a unit test: only the terminal button.
        assert_eq!(
            visible_buttons(Some(other.path())),
            vec![TabDirButton::Terminal]
        );
        assert_eq!(
            visible_buttons(Some(here.path())),
            vec![TabDirButton::Terminal]
        );
        assert!(visible_buttons(None).is_empty());
        install(&ui(TabDirFollow::Auto, "auto", ""));
        assert_eq!(
            visible_buttons(Some(other.path())),
            vec![TabDirButton::Terminal]
        );
        reset_for_tests();
    }

    #[test]
    fn the_custom_template_becomes_an_argv_with_the_folder() {
        let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
        reset_for_tests();
        install(&ui(TabDirFollow::Buttons, "custom", "my-term --cd {repo}"));
        let argv = terminal_argv(Path::new("/work/my repo")).unwrap().unwrap();
        assert_eq!(argv, ["my-term", "--cd", "/work/my repo"]);
        install(&ui(TabDirFollow::Buttons, "kitty", ""));
        let argv = terminal_argv(Path::new("/w")).unwrap().unwrap();
        assert_eq!(argv, ["kitty", "--directory", "/w"]);
        // auto + ui.launch = tmux follows the launch preset.
        let mut config = ui(TabDirFollow::Buttons, "auto", "");
        config.launch = "tmux".into();
        install(&config);
        let argv = terminal_argv(Path::new("/w")).unwrap().unwrap();
        assert_eq!(argv, ["tmux", "new-window", "-c", "/w"]);
        // auto with launch off: the platform's own terminal.
        install(&ui(TabDirFollow::Buttons, "auto", ""));
        assert_eq!(terminal_argv(Path::new("/w")).unwrap(), None);
        reset_for_tests();
    }

    #[test]
    fn open_in_uses_the_configured_list_and_expands_the_folder() {
        let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
        reset_for_tests();
        let mut config = ui(TabDirFollow::Buttons, "auto", "");
        config.open_in = Some(vec![
            OpenInEntry::new("A", "a ${cwd}"),
            OpenInEntry::new("B", "b --name ${name} \"${cwd}/x\""),
        ]);
        config.open_in_default = "B".into();
        install(&config);
        assert_eq!(entries(), ["A", "B"]);
        assert_eq!(default_index(), 1);
        let argv = open_in_argv(1, Path::new("/w/my repo")).unwrap();
        assert_eq!(argv, ["b", "--name", "my repo", "/w/my repo/x"]);
        assert!(open_in_argv(2, Path::new("/w")).is_err());
        config.open_in_default = "gone".into();
        install(&config);
        assert_eq!(default_index(), 0);
        // Absent: computed, and the legacy custom terminal heads the list.
        let mut config = ui(TabDirFollow::Buttons, "custom", "my-term --cd {repo}");
        config.open_in = None;
        install(&config);
        assert_eq!(entries()[0], "Terminal (custom)");
        let argv = open_in_argv(0, Path::new("/w/r")).unwrap();
        assert_eq!(argv, ["my-term", "--cd", "/w/r"]);
        reset_for_tests();
    }

    #[cfg(unix)]
    #[test]
    fn open_terminal_runs_the_launcher_with_the_folder() {
        use std::os::unix::fs::PermissionsExt;
        let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
        reset_for_tests();
        let dir = tempfile::tempdir().unwrap();
        let log = dir.path().join("args.log");
        let script = dir.path().join("fake-term.sh");
        std::fs::write(
            &script,
            format!("#!/bin/sh\necho \"$@\" > '{}'\n", log.display()),
        )
        .unwrap();
        std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
        let repo = tempfile::tempdir().unwrap();
        install(&ui(
            TabDirFollow::Buttons,
            "custom",
            &format!("{} --cwd {{repo}}", script.display()),
        ));
        let message = open_terminal(repo.path()).unwrap();
        assert!(message.contains("opened a terminal"), "{message}");
        for _ in 0..100 {
            if log.exists() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        assert_eq!(
            std::fs::read_to_string(&log).unwrap().trim(),
            format!("--cwd {}", repo.path().display())
        );
        // A missing folder and a missing program are immediate errors.
        assert!(open_terminal(&dir.path().join("gone")).is_err());
        install(&ui(
            TabDirFollow::Buttons,
            "custom",
            "no-such-launcher-xyz {repo}",
        ));
        let err = open_terminal(repo.path()).unwrap_err();
        assert!(err.contains("no-such-launcher-xyz"), "{err}");
        reset_for_tests();
    }

    fn open_in_config(entries: &[(&str, &str)], follow: TabDirFollow) -> UiConfig {
        let mut config = ui(follow, "auto", "");
        config.open_in = Some(
            entries
                .iter()
                .map(|(l, c)| OpenInEntry::new(*l, *c))
                .collect(),
        );
        config
    }

    #[test]
    fn explorer_exit_code_one_is_success_and_other_failures_say_what_happened() {
        assert_eq!(exit_failure("Explorer", "explorer.exe", Some(1), b""), None);
        assert_eq!(
            exit_failure("Explorer", r"C:\Windows\Explorer.EXE", Some(1), b""),
            None
        );
        assert_eq!(exit_failure("X", "x", Some(0), b""), None);
        // Another program exiting 1 is a failure; explorer exiting 2 too.
        assert!(exit_failure("X", "x", Some(1), b"").is_some());
        assert!(exit_failure("Explorer", "explorer", Some(2), b"").is_some());
        // The message: label, exit code and the first non-empty stderr line.
        assert_eq!(
            exit_failure(
                "tmux: new window",
                "tmux",
                Some(1),
                b"\nno server running\nmore\n"
            ),
            Some("tmux: new window failed (exit 1): no server running".into())
        );
        assert_eq!(
            exit_failure("Z", "z", Some(7), b"  \n"),
            Some("Z failed (exit 7)".into())
        );
        assert!(exit_failure("Z", "z", None, b"")
            .unwrap()
            .contains("stopped"));
    }

    #[test]
    fn console_shells_get_their_own_window_and_commands_do_not() {
        let a = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert!(wants_new_console("cmd", &[]));
        assert!(wants_new_console(r"C:\Windows\System32\cmd.exe", &[]));
        assert!(wants_new_console("cmd", &a(&["/K", "claude"])));
        assert!(wants_new_console("pwsh.exe", &[]));
        assert!(wants_new_console("wsl", &a(&["--cd", r"C:\a&b"])));
        assert!(!wants_new_console("cmd", &a(&["/C", "dir"])));
        assert!(!wants_new_console("pwsh", &a(&["-c", "ls"])));
        assert!(!wants_new_console("code", &[]));
    }

    #[test]
    fn the_cmd_window_preset_never_puts_the_folder_in_a_command_line() {
        let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
        reset_for_tests();
        let hostile = r"C:\x&calc|^%PATH%!";
        for old in [
            "cmd /C start \"\" /D ${cwd} cmd.exe",
            open_in::preset("cmd-window").unwrap().command,
        ] {
            install(&open_in_config(&[("Cmd", old)], TabDirFollow::Buttons));
            let state = open_in_state();
            let command = open_in::modernized_command(&state.entries[0].command)
                .unwrap_or(&state.entries[0].command);
            let argv = open_in::expand(
                command,
                &open_in::OpenInVars {
                    cwd: hostile,
                    ..Default::default()
                },
                &|_| None,
            )
            .unwrap();
            assert_eq!(argv, ["cmd.exe"], "{old}");
            assert!(wants_new_console(&argv[0], &argv[1..]));
        }
    }

    #[test]
    fn cmd_start_is_the_only_launcher_that_keeps_its_window() {
        let a = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert!(is_cmd_start("cmd", &a(&["/C", "start", "", "x"])));
        assert!(is_cmd_start(
            r"C:\Windows\System32\CMD.EXE",
            &a(&["/c", "START"])
        ));
        assert!(!is_cmd_start("cmd", &a(&["/C", "dir"])));
        assert!(!is_cmd_start("code", &a(&["/C", "start"])));
    }

    #[test]
    fn an_open_in_entry_is_found_by_its_label_whatever_its_position() {
        let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
        reset_for_tests();
        install(&open_in_config(
            &[("First", "tool ${cwd}"), ("Second", "tool2 ${cwd}")],
            TabDirFollow::Buttons,
        ));
        assert_eq!(index_of_label("Second"), Ok(1));
        assert_eq!(index_of_label("  First "), Ok(0));
        // Reordered: the label still finds its entry.
        install(&open_in_config(
            &[("Second", "tool2 ${cwd}"), ("First", "tool ${cwd}")],
            TabDirFollow::Buttons,
        ));
        assert_eq!(index_of_label("Second"), Ok(0));
        let err = index_of_label("Nope").unwrap_err();
        assert!(err.contains("Nope") && err.contains("Open in"), "{err}");
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(open_in_by_label("Nope", dir.path()).unwrap_err(), err);
        // A known label runs through the same runner (a bad command is its error).
        install(&open_in_config(
            &[("Typo", "tool ${nope}")],
            TabDirFollow::Buttons,
        ));
        let err = open_in_by_label("Typo", dir.path()).unwrap_err();
        assert!(err.contains("unknown variable ${nope}"), "{err}");
    }

    #[test]
    fn unknown_variables_bad_quotes_and_missing_things_are_named_at_once() {
        let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
        reset_for_tests();
        install(&open_in_config(
            &[
                ("Typo", "tool ${nope}"),
                ("Quote", "tool \"unterminated"),
                ("Ghost", "no-such-program-xyz ${cwd}"),
            ],
            TabDirFollow::Buttons,
        ));
        let dir = tempfile::tempdir().unwrap();
        let err = run_open_in(0, dir.path()).unwrap_err();
        assert!(err.starts_with("Typo: "), "{err}");
        assert!(err.contains("unknown variable ${nope}"), "{err}");
        let err = run_open_in(1, dir.path()).unwrap_err();
        assert!(err.starts_with("Quote: cannot read its command"), "{err}");
        // The program and the entry are both named.
        let err = run_open_in(2, dir.path()).unwrap_err();
        assert!(
            err.contains("Ghost") && err.contains("no-such-program-xyz"),
            "{err}"
        );
        let gone = dir.path().join("gone");
        let err = run_open_in(2, &gone).unwrap_err();
        assert!(
            err.contains("Ghost") && err.contains("gone from disk"),
            "{err}"
        );
        assert!(run_open_in(9, dir.path())
            .unwrap_err()
            .contains("number 10"));
        reset_for_tests();
    }

    #[test]
    fn auto_mode_runs_the_default_entry_and_buttons_mode_does_not() {
        let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
        reset_for_tests();
        let mut config = open_in_config(
            &[("A", "a-prog ${cwd}"), ("B", "b-prog ${cwd}")],
            TabDirFollow::Auto,
        );
        config.open_in_default = "B".into();
        install(&config);
        assert_eq!(auto_target(), Some(1));
        // The default entry's program is missing: the error names it, proving
        // that entry (and no other) was run.
        let dir = tempfile::tempdir().unwrap();
        let err = auto_open(dir.path()).unwrap().unwrap_err();
        assert!(err.starts_with("B: ") && err.contains("b-prog"), "{err}");
        install(&open_in_config(&[("A", "a")], TabDirFollow::Buttons));
        assert_eq!(auto_target(), None);
        assert!(auto_open(dir.path()).is_none());
        reset_for_tests();
    }

    fn git_dir_with_head(root: &Path, head: &str) {
        std::fs::create_dir_all(root.join(".git")).unwrap();
        std::fs::write(root.join(".git").join("HEAD"), head).unwrap();
    }

    #[test]
    fn the_branch_is_cached_until_it_is_invalidated() {
        let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
        reset_for_tests();
        let repo = tempfile::tempdir().unwrap();
        git_dir_with_head(repo.path(), "ref: refs/heads/main\n");
        let info = repo_info(repo.path());
        assert_eq!(info.branch, "main");
        assert_eq!(info.repo, repo.path().to_string_lossy());
        // HEAD moves: the cache still answers (the run path reads nothing).
        git_dir_with_head(repo.path(), "ref: refs/heads/feature/x\n");
        assert_eq!(repo_info(repo.path()).branch, "main");
        invalidate_repo_info(repo.path());
        assert_eq!(repo_info(repo.path()).branch, "feature/x");
        // Detached HEAD: empty.
        git_dir_with_head(repo.path(), "0123456789abcdef0123456789abcdef01234567\n");
        invalidate_repo_info(repo.path());
        assert_eq!(repo_info(repo.path()).branch, "");
        // The background refresh lands in the cache.
        git_dir_with_head(repo.path(), "ref: refs/heads/dev\n");
        head_changed(repo.path());
        for _ in 0..100 {
            if repo_info(repo.path()).branch == "dev" {
                break;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        assert_eq!(repo_info(repo.path()).branch, "dev");
        reset_for_tests();
    }

    #[test]
    fn a_launch_sees_a_head_that_moved_since_it_was_cached() {
        let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
        reset_for_tests();
        let repo = tempfile::tempdir().unwrap();
        git_dir_with_head(repo.path(), "ref: refs/heads/main\n");
        assert_eq!(repo_info(repo.path()).branch, "main");
        // A checkout elsewhere moves HEAD; nothing invalidated the cache.
        git_dir_with_head(repo.path(), "ref: refs/heads/feature/x\n");
        assert_eq!(launch_repo_info(repo.path()).branch, "feature/x");
        reset_for_tests();
    }

    #[test]
    fn a_linked_worktree_reports_the_main_repository_as_repo() {
        let main = tempfile::tempdir().unwrap();
        let wt = tempfile::tempdir().unwrap();
        let admin = main.path().join(".git").join("worktrees").join("w1");
        std::fs::create_dir_all(&admin).unwrap();
        std::fs::write(admin.join("HEAD"), "ref: refs/heads/topic\n").unwrap();
        std::fs::write(admin.join("commondir"), "../..\n").unwrap();
        std::fs::write(
            wt.path().join(".git"),
            format!("gitdir: {}\n", admin.display()),
        )
        .unwrap();
        let info = read_repo_info(wt.path());
        assert_eq!(info.branch, "topic");
        assert_eq!(Path::new(&info.repo), main.path());
        assert_ne!(Path::new(&info.repo), wt.path());
    }

    #[cfg(unix)]
    fn script(dir: &Path, name: &str, body: &str) -> PathBuf {
        use std::os::unix::fs::PermissionsExt;
        let path = dir.join(name);
        std::fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
        path
    }

    #[cfg(unix)]
    fn wait_for(file: &Path) -> String {
        // A generous ceiling: on macOS the first launch of a new executable
        // can take seconds. Polls, so the common case stays fast.
        let deadline = std::time::Instant::now() + Duration::from_secs(60);
        while std::time::Instant::now() < deadline {
            if let Ok(text) = std::fs::read_to_string(file) {
                if text.ends_with('\n') {
                    return text;
                }
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        panic!("{} never written", file.display());
    }

    #[cfg(unix)]
    #[test]
    fn open_in_by_label_runs_the_labelled_entry_on_the_folder() {
        let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
        reset_for_tests();
        let tools = tempfile::tempdir().unwrap();
        let log = tools.path().join("by-label.log");
        let rec = |name: &str, tag: &str| {
            script(
                tools.path(),
                name,
                &format!("echo \"{tag}|$1\" > '{}'", log.display()),
            )
        };
        let first = rec("first-tool", "first");
        let second = rec("second-tool", "second");
        install(&open_in_config(
            &[
                ("Editor", &format!("{} ${{cwd}}", first.display())),
                ("Files", &format!("{} ${{cwd}}", second.display())),
            ],
            TabDirFollow::Buttons,
        ));
        let repo = tempfile::tempdir().unwrap();
        git_dir_with_head(repo.path(), "ref: refs/heads/main\n");
        assert_eq!(
            open_in_by_label("Files", repo.path()).unwrap(),
            "opening in Files..."
        );
        let text = wait_for(&log);
        assert_eq!(
            text.trim(),
            format!("second|{}", repo.path().display()),
            "the entry labelled Files ran, not the one at position 0"
        );
        reset_for_tests();
    }

    #[cfg(unix)]
    #[test]
    fn open_in_passes_the_folder_the_environment_and_reports_a_failure_late() {
        let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
        reset_for_tests();
        let tools = tempfile::tempdir().unwrap();
        let log = tools.path().join("out.log");
        let ok = script(
            tools.path(),
            "fake-launcher",
            &format!(
                "echo \"$1|$GITPOULPI_CWD|$GITPOULPI_REPO|$GITPOULPI_BRANCH|$(pwd)\" > '{}'",
                log.display()
            ),
        );
        let bad = script(
            tools.path(),
            "failing-launcher",
            "echo 'no server running' >&2\nexit 3",
        );
        install(&open_in_config(
            &[
                ("Fake", &format!("{} ${{cwd}}", ok.display())),
                ("Failing", &format!("{} ${{cwd}}", bad.display())),
            ],
            TabDirFollow::Buttons,
        ));
        let repo = tempfile::tempdir().unwrap();
        git_dir_with_head(repo.path(), "ref: refs/heads/main\n");
        let message = run_open_in(0, repo.path()).unwrap();
        assert_eq!(message, "opening in Fake...");
        let dir = repo.path().display();
        // `pwd` is the process's own view of its directory: the physical
        // one (macOS temp dirs sit behind the `/var` -> `/private/var` link).
        let physical = repo.path().canonicalize().unwrap();
        assert_eq!(
            wait_for(&log).trim(),
            format!("{dir}|{dir}|{dir}|main|{}", physical.display())
        );
        // A failing entry: "Opening in" now, the error from the watcher.
        assert_eq!(
            run_open_in(1, repo.path()).unwrap(),
            "opening in Failing..."
        );
        let mut late = Vec::new();
        let deadline = std::time::Instant::now() + Duration::from_secs(60);
        while std::time::Instant::now() < deadline {
            late.extend(
                external_tools::take_late_errors()
                    .into_iter()
                    .filter(|m| m.starts_with("Failing")),
            );
            if !late.is_empty() {
                break;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        assert_eq!(late, ["Failing failed (exit 3): no server running"]);
        reset_for_tests();
    }
}
