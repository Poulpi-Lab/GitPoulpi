//! Shared by the `pty_*` tests: one builder that launches the real
//! `gitpoulpi-tui` binary in a pty with an isolated config directory, and
//! the `git` helpers the fixtures use.
//!
//! ```ignore
//! let (mut term, _config) = common::launch(repo.root())
//!     .session("pty-example")
//!     .config("[ui]\ngraph_renderer = \"text\"\n")
//!     .size(200, 50)
//!     .ready("initial commit")
//!     .start();
//! ```

#![allow(dead_code)]

use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use xtask::pty::{Screen, Terminal};

/// The release key pair of the update tests. The app under test is a debug
/// build, which accepts one extra public key through `GITPOULPI_TEST_RELEASE_KEY`
/// (compiled out of release builds), so a fake release server can sign what
/// it serves. The project's real key is never involved.
#[allow(dead_code)]
pub fn release_signer() -> &'static gitpoulpi_testkit::signing::TestSigner {
    static SIGNER: std::sync::OnceLock<gitpoulpi_testkit::signing::TestSigner> =
        std::sync::OnceLock::new();
    SIGNER.get_or_init(gitpoulpi_testkit::signing::TestSigner::new)
}

/// The environment variable that hands [`release_signer`]'s public key to the app.
#[allow(dead_code)]
pub const RELEASE_KEY_ENV: &str = "GITPOULPI_TEST_RELEASE_KEY";

/// How long the first byte of output may take.
const FIRST_OUTPUT: Duration = Duration::from_secs(20);
/// How long the ready condition may take.
const STARTUP: Duration = Duration::from_secs(60);

type Ready = Box<dyn FnMut(&Screen) -> bool>;

/// A pty launch of the TUI. Start with [`launch`].
pub struct Launch {
    repo: PathBuf,
    session: String,
    implicit_session: bool,
    config: Option<String>,
    config_dir: Option<PathBuf>,
    size: (u16, u16),
    envs: Vec<(OsString, OsString)>,
    extra_args: Vec<OsString>,
    kitty_answers: bool,
    background: Option<String>,
    terminal_image: bool,
    mouse_capture: bool,
    startup: Duration,
    ready: Option<(String, Ready)>,
    /// Do not wait for the child's first output (see [`Launch::detached`]).
    detached: bool,
    /// Another executable than the built `gitpoulpi-tui` (a copy in a temp
    /// install folder, for the self-update tests).
    binary: Option<PathBuf>,
    /// Seeds the session so the commit column is off (see
    /// [`Launch::no_commit_column`]).
    no_commit_column: bool,
}

/// Launches the TUI with `repo` as its working directory.
pub fn launch(repo: impl AsRef<Path>) -> Launch {
    Launch {
        repo: repo.as_ref().to_path_buf(),
        session: "pty-test".into(),
        implicit_session: false,
        config: None,
        config_dir: None,
        size: (200, 50),
        envs: Vec::new(),
        extra_args: Vec::new(),
        kitty_answers: false,
        background: None,
        terminal_image: false,
        mouse_capture: false,
        startup: STARTUP,
        ready: None,
        detached: false,
        binary: None,
        no_commit_column: false,
    }
}

impl Launch {
    /// Runs this executable instead of the built `gitpoulpi-tui`.
    pub fn binary(mut self, path: impl AsRef<Path>) -> Self {
        self.binary = Some(path.as_ref().to_path_buf());
        self
    }

    /// Starts with the commit column turned off (what a user's "off" choice
    /// saves in the session file): the tests about the view bars at a width
    /// where the column would take the bar's room (it shows from 140
    /// columns) are not about the column.
    pub fn no_commit_column(mut self) -> Self {
        self.no_commit_column = true;
        self
    }

    /// The named session (`-s`); one per test, so daemons never collide.
    pub fn session(mut self, name: &str) -> Self {
        self.session = name.into();
        self
    }

    /// No `-s`: the session is derived from the launch folder, as a
    /// plain `gitpoulpi` would. Isolation still holds through the config dir.
    pub fn implicit_session(mut self) -> Self {
        self.implicit_session = true;
        self
    }

    /// Seeds the fresh config directory with this `config.toml` (none when empty).
    pub fn config(mut self, toml: impl AsRef<str>) -> Self {
        self.config = Some(toml.as_ref().into());
        self
    }

    /// Uses a config directory the test owns instead of a fresh one.
    pub fn config_dir(mut self, dir: &Path) -> Self {
        self.config_dir = Some(dir.to_path_buf());
        self
    }

    pub fn size(mut self, cols: u16, rows: u16) -> Self {
        self.size = (cols, rows);
        self
    }

    pub fn env(mut self, key: impl AsRef<OsStr>, value: impl AsRef<OsStr>) -> Self {
        self.envs
            .push((key.as_ref().to_owned(), value.as_ref().to_owned()));
        self
    }

    /// An extra command-line argument for the TUI binary (after `-s`).
    pub fn arg(mut self, value: impl AsRef<OsStr>) -> Self {
        self.extra_args.push(value.as_ref().to_owned());
        self
    }

    /// Skips the "press a key to close" pause when the TUI exits.
    pub fn no_exit_pause(self) -> Self {
        self.env("GITPOULPI_NO_EXIT_PAUSE", "1")
    }

    /// Answers the kitty-keyboard query the way a kitty-protocol terminal does.
    pub fn kitty_answers(mut self) -> Self {
        self.kitty_answers = true;
        self
    }

    /// Answers the OSC 11 background query with `rgb` (`rgb:rrrr/gggg/bbbb`),
    /// as most terminals do; silent by default.
    pub fn answer_background(mut self, rgb: &str) -> Self {
        self.background = Some(rgb.to_string());
        self
    }

    /// Leaves `GITPOULPI_TERMINAL_IMAGE` unset, for tests of the image graph.
    pub fn terminal_image(mut self) -> Self {
        self.terminal_image = true;
        self
    }

    /// Also waits for the child to turn mouse capture on.
    pub fn mouse_capture(mut self) -> Self {
        self.mouse_capture = true;
        self
    }

    /// Overrides the ready-wait timeout.
    pub fn startup(mut self, timeout: Duration) -> Self {
        self.startup = timeout;
        self
    }

    /// Ready once the screen shows `text`.
    pub fn ready(self, text: &str) -> Self {
        let needle = text.to_string();
        self.ready_when(&format!("{text:?} drawn"), move |s| s.contains(&needle))
    }

    /// Ready once the screen shows every one of `texts`.
    pub fn ready_all(self, texts: &[&str]) -> Self {
        let needles: Vec<String> = texts.iter().map(|t| t.to_string()).collect();
        self.ready_when(&format!("{texts:?} drawn"), move |s| {
            needles.iter().all(|n| s.contains(n))
        })
    }

    /// Ready once `predicate` holds; `what` names it in the timeout message.
    pub fn ready_when(
        mut self,
        what: &str,
        predicate: impl FnMut(&Screen) -> bool + 'static,
    ) -> Self {
        self.ready = Some((what.into(), Box::new(predicate)));
        self
    }

    /// Returns at once, with no wait for the child to draw. For a child that
    /// does its job and exits (`gp -a` handing a folder to a running window):
    /// a ConPTY can lose the output of a process that exits straight away,
    /// so "it drew something" is not a promise the driver can keep for it.
    pub fn detached(mut self) -> Self {
        self.detached = true;
        self
    }

    /// Spawns against a config directory the test owns ([`config_dir`](Self::config_dir)).
    pub fn spawn(self) -> Terminal {
        let dir = self
            .config_dir
            .clone()
            .expect("spawn() needs config_dir(); use start() for a fresh one");
        self.run(&dir)
    }

    /// Spawns against a fresh config directory, returned so it outlives the run.
    pub fn start(self) -> (Terminal, tempfile::TempDir) {
        let dir = tempfile::tempdir().expect("temp config dir");
        if let Some(toml) = self.config.as_ref().filter(|t| !t.is_empty()) {
            std::fs::write(dir.path().join("config.toml"), toml).expect("seed config.toml");
        }
        let term = self.run(dir.path());
        (term, dir)
    }

    fn run(self, config: &Path) -> Terminal {
        if self.no_commit_column {
            let dir = config.join("sessions");
            std::fs::create_dir_all(&dir).expect("sessions dir");
            let file = dir.join(format!("{}.toml", self.session));
            if !file.exists() {
                std::fs::write(&file, "[layout]\ncommit_column = 2\n").expect("seed the session");
            }
        }
        let program = self
            .binary
            .clone()
            .unwrap_or_else(|| PathBuf::from(env!("CARGO_BIN_EXE_gitpoulpi-tui")));
        let mut builder = Terminal::builder(program)
            .args(if self.implicit_session {
                Vec::new()
            } else {
                vec!["-s", self.session.as_str()]
            })
            .args(
                self.extra_args
                    .iter()
                    .map(|a| a.to_string_lossy().into_owned()),
            )
            .cwd(&self.repo)
            .env("GITPOULPI_CONFIG_DIR", config)
            .size(self.size.0, self.size.1);
        // Not optional by default: a pty inherits this process' environment,
        // so under WezTerm the child would draw the Graph column as an
        // inline image into a cell grid that has no pixels.
        if !self.terminal_image {
            builder = builder.env("GITPOULPI_TERMINAL_IMAGE", "off");
        }
        // Background fetching is on by default (fetch on tab open, every
        // 5 minutes); tests that are not about it must not race it. A test
        // about it overrides these with `.env(..)`.
        builder = builder
            .env("GITPOULPI_GENERAL_FETCH_ON_TAB_OPEN", "false")
            .env("GITPOULPI_GENERAL_AUTO_FETCH_INTERVAL_MINUTES", "0")
            // The once-a-day update check asks a release server: only the
            // tests about it (`pty_update_check`) turn it back on.
            .env("GITPOULPI_UPDATES_MODE", "off")
            // `ui.theme = system` falls back to COLORFGBG when the terminal
            // is silent: pin it, so a developer's own light terminal does
            // not repaint every test's expectations.
            .env("COLORFGBG", "15;0")
            // Changing the logo color rewrites the icons of installed
            // shortcuts: never the developer's own launcher entry.
            .env("XDG_DATA_HOME", config.join("xdg-data"))
            // "Open in" asks the desktop which apps open a file type
            // (`open_with`): never the developer's own installed applications.
            .env("XDG_DATA_DIRS", config.join("xdg-data-dirs"))
            .env("XDG_CONFIG_DIRS", config.join("xdg-config-dirs"))
            // A terminal program is opened in `$TERMINAL`: never the
            // developer's own terminal emulator.
            .env("TERMINAL", "")
            .env("GITPOULPI_DESKTOP_NO_HOOKS", "1");
        // The "What is sent" dialog stands between an AI action and its
        // request: every test about an AI answer would stop at it. Only
        // `pty_ai_sent_dialog` turns it back on, with `.env(..)` of these
        // keys (`gitpoulpi_config::AiSentPreview`).
        for action in [
            "COMMIT_MESSAGE",
            "EXPLAIN_CHANGES",
            "STASH_MESSAGE",
            "PULL_REQUEST_DESCRIPTION",
            "COMMIT_COMPOSER",
            "CONFLICT_RESOLVE",
            "CODE_REVIEW",
            "SPLIT_COMMIT",
            "REWRITE_SCRIPT",
            "REWRITE_RULES",
        ] {
            builder = builder.env(format!("GITPOULPI_AI_SENT_PREVIEW_{action}"), "false");
        }
        // (Alone in its ConPTY the Windows build would relaunch itself in
        // Windows Terminal and count as a launcher start: the PTY driver says
        // a shell shares the console, `GITPOULPI_TEST_SHARED_CONSOLE`.)
        for (key, value) in &self.envs {
            builder = builder.env(key, value);
        }
        if self.kitty_answers {
            builder = builder.answer_kitty_keyboard();
        }
        if let Some(rgb) = &self.background {
            builder = builder.answer_background(rgb);
        }
        let term = builder.spawn().expect("spawn the TUI in a pty");
        if self.detached {
            return term;
        }
        term.wait_for_first_output(FIRST_OUTPUT)
            .unwrap_or_else(|e| panic!("the TUI drew nothing: {e}"));
        if let Some((what, predicate)) = self.ready {
            term.wait_for(&what, self.startup, predicate)
                .unwrap_or_else(|e| panic!("{e}"));
        }
        if self.mouse_capture {
            term.wait_for_mouse_capture(self.startup)
                .unwrap_or_else(|e| panic!("{e}"));
        }
        term
    }
}

/// Runs `git` in `repo` and returns its stdout.
pub fn git(repo: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .args(args)
        .current_dir(repo)
        .output()
        .expect("run git");
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// Runs `git` in `repo`, panics unless it succeeds, returns trimmed stdout.
pub fn git_trimmed(repo: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .args(args)
        .current_dir(repo)
        .output()
        .expect("run git");
    assert!(
        out.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

/// Runs `git` in `repo` and panics, with its stderr, unless it succeeds.
pub fn git_ok(repo: &Path, args: &[&str]) {
    let out = Command::new("git")
        .args(args)
        .current_dir(repo)
        .output()
        .expect("run git");
    assert!(
        out.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// Repeats an idempotent `act` until `predicate` holds, for effects the TUI
/// can lose to a race the test cannot see (a status message wiped by a late
/// daemon refresh, a key landing mid-action). Only a timeout retries; a dead
/// child fails at once. Not a blind sleep: it returns the moment the result
/// shows.
pub fn retry_until(
    term: &mut Terminal,
    what: &str,
    tries: usize,
    per_try: Duration,
    mut act: impl FnMut(&mut Terminal),
    mut predicate: impl FnMut(&Screen) -> bool,
) -> Screen {
    let mut last = None;
    for _ in 0..tries {
        act(term);
        match term.wait_for(what, per_try, &mut predicate) {
            Ok(screen) => return screen,
            Err(e @ xtask::pty::Error::Timeout { .. }) => last = Some(e),
            Err(e) => panic!("{e}"),
        }
    }
    panic!("{}", last.expect("tries must be at least 1"));
}

/// Right-clicks the text `needle` (one cell in) until `predicate` holds.
///
/// A right-click sent as soon as the row is drawn can be dropped: the row is
/// drawn before its hit target is registered. The row is re-located on every
/// attempt.
pub fn right_click_text_until(
    term: &mut Terminal,
    needle: &str,
    what: &str,
    predicate: impl FnMut(&Screen) -> bool,
) -> Screen {
    retry_until(
        term,
        what,
        8,
        Duration::from_secs(4),
        |t| {
            if let Some((col, row)) = t.screen().find(needle) {
                t.right_click(col + 1, row).unwrap();
            }
        },
        predicate,
    )
}

/// Left-clicks the text `needle` (one cell in) until `predicate` holds.
///
/// A click sent as soon as a button is drawn can be dropped on a loaded
/// machine: the cell is painted before its hit target is registered. The text
/// is re-located on every attempt, and nothing is clicked once the predicate
/// already holds, so a button that toggles (Details) is never toggled back by
/// a retry that raced a slow first click.
pub fn click_text_until(
    term: &mut Terminal,
    needle: &str,
    what: &str,
    mut predicate: impl FnMut(&Screen) -> bool,
) -> Screen {
    let mut last = None;
    for _ in 0..6 {
        let screen = term.screen();
        if predicate(&screen) {
            return screen;
        }
        if let Some((col, row)) = screen.find(needle) {
            term.click(col + 1, row).unwrap();
        }
        match term.wait_for(what, Duration::from_secs(10), &mut predicate) {
            Ok(screen) => return screen,
            Err(e @ xtask::pty::Error::Timeout { .. }) => last = Some(e),
            Err(e) => panic!("{e}"),
        }
    }
    panic!("{}\n{}", last.expect("at least one try"), term.text());
}

/// Waits for the status line's error `error: {title}` (a failed operation has
/// no card), asserts no card is drawn, and clicks it: the Activity Log opens
/// on the entry. Returns the screen the error was found on.
pub fn open_status_error(term: &mut Terminal, title: &str) -> Screen {
    let needle = format!("error: {title}");
    let screen = term
        .wait_for(
            "the failure in the status line",
            Duration::from_secs(30),
            |s| s.contains(&needle),
        )
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(
        !screen.contains(" Details ") && !screen.contains(&format!("{title} \u{2500}")),
        "a failure has no card:\n{}",
        screen.text()
    );
    let (row, line) = screen
        .text()
        .lines()
        .enumerate()
        .find(|(_, l)| l.contains(&needle))
        .map(|(i, l)| (i, l.to_string()))
        .expect("the error line");
    let col = line[..line.find("error:").unwrap()].chars().count();
    term.click(col as u16 + 2, row as u16).expect("click it");
    screen
}

/// Runs `git` in `repo`; `Some(trimmed stdout)` only if it succeeded.
///
/// For polling a repository the TUI is writing to: git can fail for an
/// instant mid-operation (`git stash list` during a pop), which a poll must
/// treat as "not yet" rather than panic.
pub fn git_try(repo: &Path, args: &[&str]) -> Option<String> {
    let out = Command::new("git")
        .args(args)
        .current_dir(repo)
        .output()
        .ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// One daemon request on a fresh connection to `session`'s daemon, as the
/// CLI and MCP tools send it. `config` is the TUI's `GITPOULPI_CONFIG_DIR`.
pub fn daemon_call(
    config: &Path,
    session: &str,
    command: gitpoulpi_core::domain::OperationKind,
    args: impl serde::Serialize,
) -> serde_json::Value {
    let port_file = config.join("sessions").join(format!("{session}.port"));
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        let mut client = gitpoulpi_ipc::Client::connect(
            &gitpoulpi_ipc::discovery::endpoint(&port_file)
                .expect("the session published its discovery files"),
        )
        .await
        .expect("connect to the session's daemon");
        let response = client
            .call(&gitpoulpi_ipc::Request {
                id: 1,
                session: session.into(),
                repo: None,
                command,
                args: serde_json::to_value(args).unwrap(),
            })
            .await
            .expect("call");
        match response.result {
            Ok(v) => v,
            Err(e) => serde_json::json!({ "error": e.message }),
        }
    })
}

/// What `gitpoulpi ui show ...` does: sends `intent` to `session`'s daemon
/// (which routes it to the attached TUI window) in the agent's `control`
/// mode and returns the window's answer.
pub fn ui_show_as(
    config: &Path,
    session: &str,
    intent: gitpoulpi_ipc::ui::UiIntent,
    control: gitpoulpi_core::domain::UiControl,
) -> gitpoulpi_ipc::ui::UiOutcome {
    let v = daemon_call(
        config,
        session,
        gitpoulpi_core::domain::OperationKind::UiShow,
        gitpoulpi_ipc::ui::UiShowArgs {
            viewer: None,
            intent,
            origin: gitpoulpi_ipc::ui::UiOrigin {
                client: "claude-code".into(),
                label: None,
                control,
            },
            timeout_ms: Some(10_000),
        },
    );
    serde_json::from_value(v.clone()).unwrap_or_else(|_| panic!("an outcome, got {v}"))
}

/// [`ui_show_as`] in `auto` mode: the agent lets the window move at once.
pub fn ui_show(
    config: &Path,
    session: &str,
    intent: gitpoulpi_ipc::ui::UiIntent,
) -> gitpoulpi_ipc::ui::UiOutcome {
    ui_show_as(
        config,
        session,
        intent,
        gitpoulpi_core::domain::UiControl::Auto,
    )
}

/// `ui state`: what the window reports about itself.
pub fn ui_state(config: &Path, session: &str) -> gitpoulpi_ipc::ui::UiState {
    let v = daemon_call(
        config,
        session,
        gitpoulpi_core::domain::OperationKind::UiQuery,
        gitpoulpi_ipc::ui::UiQueryArgs::State {
            viewer: None,
            include_text: false,
        },
    );
    serde_json::from_value(v.clone()).unwrap_or_else(|_| panic!("a state, got {v}"))
}

/// `ui state --text`: the state with the visible text of the central pane.
pub fn ui_state_text(config: &Path, session: &str) -> gitpoulpi_ipc::ui::UiState {
    let v = daemon_call(
        config,
        session,
        gitpoulpi_core::domain::OperationKind::UiQuery,
        gitpoulpi_ipc::ui::UiQueryArgs::State {
            viewer: None,
            include_text: true,
        },
    );
    serde_json::from_value(v.clone()).unwrap_or_else(|_| panic!("a state, got {v}"))
}

/// Waits until a TUI window is attached to `session`'s daemon.
pub fn wait_for_viewer(config: &Path, session: &str) {
    let deadline = std::time::Instant::now() + Duration::from_secs(30);
    loop {
        let v = daemon_call(
            config,
            session,
            gitpoulpi_core::domain::OperationKind::UiQuery,
            gitpoulpi_ipc::ui::UiQueryArgs::Viewers,
        );
        if v.as_array().is_some_and(|a| !a.is_empty()) {
            return;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "no window attached to {session}"
        );
        std::thread::sleep(Duration::from_millis(100));
    }
}

/// With Settings open, clicks the sidebar entry titled `title` and waits for
/// the body to show it (`<title> - global` / ` - repo-specific`). Navigating
/// by title, not by a count of `Down` presses, keeps a test valid when the
/// page list changes. `nth` picks among several entries with the same title
/// (the AI page is listed twice: global, then repo-specific).
pub fn settings_page_nth(term: &mut Terminal, title: &str, nth: usize) {
    let step = Duration::from_secs(20);
    let screen = term
        .wait_for(&format!("the {title} entry in the sidebar"), step, |s| {
            s.find_all(title)
                .iter()
                .filter(|(c, r)| *c < 28 && *r >= 3)
                .count()
                > nth
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (col, row) = screen
        .find_all(title)
        .into_iter()
        .filter(|(c, r)| *c < 28 && *r >= 3)
        .nth(nth)
        .unwrap();
    term.click(col + 1, row).expect("click the sidebar entry");
    term.wait_for(&format!("the {title} page"), step, |s| {
        s.contains(&format!("{title} - global")) || s.contains(&format!("{title} - repo-specific"))
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

/// [`settings_page_nth`] for a title that is listed once.
pub fn settings_page(term: &mut Terminal, title: &str) {
    settings_page_nth(term, title, 0);
}

/// Clicks the sub-tab `label` (`General`, or the list tab's name) of the
/// master-detail Settings page in front. The strip is the row under the page
/// title, right of the sidebar.
pub fn settings_tab(term: &mut Terminal, label: &str) {
    let step = Duration::from_secs(20);
    let screen = term
        .wait_for(&format!("the {label} sub-tab"), step, |s| {
            tab_cell(s, label).is_some()
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (col, row) = tab_cell(&screen, label).expect("the predicate matched");
    term.click(col + 1, row).expect("click the sub-tab");
    // Let the page repaint (and a double-click window pass) before going on.
    std::thread::sleep(Duration::from_millis(300));
}

/// Where the strip's tab `label` starts: on the row of the `General` tab,
/// right of the 28-column sidebar.
fn tab_cell(s: &Screen, label: &str) -> Option<(u16, u16)> {
    let (_, strip) = s
        .find_all(" General ")
        .into_iter()
        .find(|(c, r)| *c >= 28 && *r >= 2)?;
    s.find_all(&format!(" {label} "))
        .into_iter()
        .find(|(c, r)| *c >= 28 && *r == strip)
}

/// Where a Settings choice showing `value` is drawn: the value, padded
/// inside its band, then the `▾` arrow (`direct      ▾`). Returns the cell of
/// the value's first letter, or `None` when no such choice is on screen.
/// The band pads the value, so `"direct ▾"` as one string never appears.
pub fn choice_at(screen: &Screen, value: &str) -> Option<(u16, u16)> {
    screen.find_all(value).into_iter().find(|&(_, row)| {
        let line = screen.line(row);
        line.find(value).is_some_and(|i| {
            let rest = &line[i + value.len()..];
            rest.starts_with(' ') && rest.trim_start_matches(' ').starts_with('\u{25BE}')
        })
    })
}

/// [`choice_at`] as a wait predicate target: waits until the choice shows.
pub fn wait_for_choice(term: &Terminal, value: &str, timeout: Duration) -> (u16, u16) {
    let screen = term
        .wait_for(&format!("a choice showing {value:?}"), timeout, |s| {
            choice_at(s, value).is_some()
        })
        .unwrap_or_else(|e| panic!("{e}"));
    choice_at(&screen, value).expect("predicate just matched")
}

/// Scrolls the Settings body with the wheel until `text` is on screen, and
/// returns where it is. Every row now carries its wrapped help, so pages are
/// taller than the terminal and a row can sit below the fold: a test that
/// needs one asks for it by name instead of assuming it is drawn.
pub fn scroll_settings_to(term: &mut Terminal, text: &str, timeout: Duration) -> (u16, u16) {
    let screen = scroll_settings_until(term, &format!("{text:?}"), timeout, |s| s.contains(text));
    screen.find(text).expect("predicate just matched")
}

/// [`scroll_settings_to`] for a choice showing `value` (see [`choice_at`]).
pub fn scroll_settings_to_choice(
    term: &mut Terminal,
    value: &str,
    timeout: Duration,
) -> (u16, u16) {
    let screen =
        scroll_settings_until(term, &format!("a choice showing {value:?}"), timeout, |s| {
            choice_at(s, value).is_some()
        });
    choice_at(&screen, value).expect("predicate just matched")
}

fn scroll_settings_until(
    term: &mut Terminal,
    what: &str,
    timeout: Duration,
    found: impl Fn(&Screen) -> bool,
) -> Screen {
    term.act_until(
        &format!("{what} scrolled into view"),
        Duration::from_millis(400),
        timeout,
        |t| {
            let s = t.screen();
            if !found(&s) {
                for _ in 0..2 {
                    t.scroll(true, 100, s.rows() / 2)?;
                }
            }
            Ok(())
        },
        |s| found(s),
    )
    .unwrap_or_else(|e| panic!("{e}"))
}

/// Confirms a removal dialog ("... This cannot be undone.", buttons
/// `Cancel` and the confirm button to its right) by clicking the confirm
/// button, whatever its label.
pub fn confirm_removal(term: &mut Terminal, timeout: Duration) {
    let screen = term
        .wait_for("the removal confirmation", timeout, |s| {
            // "This cannot": a long label wraps the sentence before "be".
            s.contains("This cannot") && s.contains("Cancel")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (cancel_col, row) = screen.find("Cancel").expect("the Cancel button");
    let mut col = cancel_col + "Cancel".len() as u16;
    while screen
        .cell(col, row)
        .is_some_and(|c| c.text.trim().is_empty())
    {
        col += 1;
    }
    term.click(col + 1, row).expect("click the confirm button");
}
