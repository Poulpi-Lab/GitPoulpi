//! Client side of `gitpoulpi --add` / `gp -a`: open a folder as a new tab of a
//! GitPoulpi that is already running, instead of starting a second one.
//!
//! It lives in this crate because the TUI binary (what the `gp` command alias
//! starts) and the `gitpoulpi` CLI both need it and neither depends on the
//! other; both already depend on the daemon crate.
//!
//! Nothing here starts a daemon. A *session* is joinable when its daemon
//! answers and at least one TUI window is attached to it (a daemon that only
//! a script started has no window to show a tab in). The pieces are small and
//! pure where they can be: [`candidates`], [`plan`], [`parse_choice`],
//! [`resolve_repo`]; only [`scan_sessions_within`], [`join_session`] and
//! [`raise_window`] touch sockets.
//!
//! The tab is opened with the existing `UiShow(ShowTab)` request, tagged with
//! [`UiOrigin::add_command`] so the window applies it as the user's own choice
//! (no agent mark, no confirmation chip). A window whose user switched agent
//! view control off never announces itself to the daemon, so it is not found
//! here either (the same limit `gitpoulpi launch` has).

use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

use gitpoulpi_config::{list_session_port_files, ConfigPaths};
use gitpoulpi_core::domain::{OperationKind, RepoId, RepoRef};
use gitpoulpi_ipc::ui::{
    RaiseAnswer, UiIntent, UiOrigin, UiOutcome, UiQueryArgs, UiShowArgs, ViewerId, ViewerInfo,
    RAISE_ANSWER_TIMEOUT_MS,
};
use gitpoulpi_ipc::{Client, Request};

/// How long a daemon gets to list its windows. Short on purpose: the daemons
/// are on loopback, and `gp -a` must feel as fast as `gp`.
pub const PROBE_TIMEOUT: Duration = Duration::from_millis(400);

/// One live daemon's windows.
#[derive(Debug, Clone)]
pub struct SessionViewers {
    pub session: String,
    pub port: u16,
    pub viewers: Vec<ViewerInfo>,
}

/// The windows of every live session except `skip`. Dead port files are
/// skipped silently; nothing is started.
///
/// Every file is judged and probed at once, so the whole scan waits for the
/// slowest probe (at most `patience`), not the sum: a dozen leftover files
/// from crashed windows must not turn a click into seconds (on Windows every
/// refused loopback connect is slow). A file whose recorded process is gone
/// is deleted without a connect.
pub async fn scan_sessions_within(skip: Option<&str>, patience: Duration) -> Vec<SessionViewers> {
    let Some(paths) = ConfigPaths::discover() else {
        return Vec::new();
    };
    scan_dir(
        &paths.sessions_dir(),
        skip,
        patience,
        &gitpoulpi_ipc::discovery::pid_alive,
        probe_viewers,
    )
    .await
}

/// [`scan_sessions_within`] over one directory, with the process check and the
/// probe injectable.
async fn scan_dir<F, Fut>(
    sessions_dir: &Path,
    skip: Option<&str>,
    patience: Duration,
    pid_alive: &(dyn Fn(u32) -> bool + Sync),
    probe: F,
) -> Vec<SessionViewers>
where
    F: Fn(u16, Duration) -> Fut,
    Fut: std::future::Future<Output = Option<Vec<ViewerInfo>>> + Send + 'static,
{
    let mut probes = tokio::task::JoinSet::new();
    for (index, (session, port)) in list_session_port_files(sessions_dir)
        .into_iter()
        .enumerate()
    {
        if skip == Some(session.as_str()) {
            continue;
        }
        let port_file = sessions_dir.join(format!("{session}.port"));
        if gitpoulpi_ipc::discovery::remove_if_stale(&port_file, pid_alive) {
            continue;
        }
        let answer = probe(port, patience);
        probes.spawn(async move { (index, session, port, answer.await) });
    }
    let mut found = Vec::new();
    while let Some(done) = probes.join_next().await {
        if let Ok((index, session, port, Some(viewers))) = done {
            found.push((index, session, port, viewers));
        }
    }
    found.sort_by_key(|f| f.0);
    found
        .into_iter()
        .map(|(_, session, port, viewers)| SessionViewers {
            session,
            port,
            viewers,
        })
        .collect()
}

async fn probe_viewers(port: u16, patience: Duration) -> Option<Vec<ViewerInfo>> {
    let work = async {
        let mut client = Client::connect_port(port).await.ok()?;
        let request = Request {
            id: 1,
            session: String::new(),
            repo: None,
            command: OperationKind::UiQuery,
            args: serde_json::to_value(UiQueryArgs::Viewers).ok()?,
        };
        let response = client.call(&request).await.ok()?;
        serde_json::from_value(response.result.ok()?).ok()
    };
    tokio::time::timeout(patience, work).await.ok().flatten()
}

/// A running session with at least one TUI window: somewhere to add a tab.
#[derive(Debug, Clone, PartialEq)]
pub struct Candidate {
    pub session: String,
    pub port: u16,
    /// Most recently focused first (the order the daemon lists them in).
    pub viewers: Vec<ViewerInfo>,
}

impl Candidate {
    /// Every repository open in a tab of any window, each once.
    pub fn tabs(&self) -> Vec<RepoId> {
        let mut tabs: Vec<RepoId> = Vec::new();
        for viewer in &self.viewers {
            for tab in &viewer.tabs {
                if !tabs.contains(tab) {
                    tabs.push(tab.clone());
                }
            }
        }
        tabs
    }

    /// The window that gets `repo`: the one showing it, else the one that has
    /// it in a tab, else the most recently focused.
    pub fn best_window(&self, repo: &RepoId) -> Option<&ViewerInfo> {
        self.viewers
            .iter()
            .find(|v| v.active_repo.as_ref() == Some(repo))
            .or_else(|| self.viewers.iter().find(|v| v.tabs.contains(repo)))
            .or_else(|| self.viewers.first())
    }
}

/// The joinable sessions of a scan, by name.
pub fn candidates(found: Vec<SessionViewers>) -> Vec<Candidate> {
    let mut out: Vec<Candidate> = found
        .into_iter()
        .filter(|s| !s.viewers.is_empty())
        .map(|s| Candidate {
            session: s.session,
            port: s.port,
            viewers: s.viewers,
        })
        .collect();
    out.sort_by(|a, b| a.session.cmp(&b.session));
    out
}

/// A repository's folder name, for lists.
pub fn repo_name(id: &RepoId) -> String {
    Path::new(&id.0)
        .file_name()
        .map_or_else(|| id.0.clone(), |n| n.to_string_lossy().into_owned())
}

/// `work  (3 tabs: api, web, docs)`: how a session is told apart in the menu.
pub fn describe(candidate: &Candidate) -> String {
    const SHOWN: usize = 4;
    let tabs = candidate.tabs();
    let names: Vec<String> = tabs.iter().take(SHOWN).map(repo_name).collect();
    let more = tabs.len().saturating_sub(SHOWN);
    let windows = if candidate.viewers.len() > 1 {
        format!(", {} windows", candidate.viewers.len())
    } else {
        String::new()
    };
    format!(
        "{}  ({} tab{}{}{}{})",
        candidate.session,
        tabs.len(),
        if tabs.len() == 1 { "" } else { "s" },
        if names.is_empty() { "" } else { ": " },
        if more > 0 {
            format!("{}, +{more}", names.join(", "))
        } else {
            names.join(", ")
        },
        windows
    )
}

/// The numbered menu, one line per session.
pub fn menu(candidates: &[Candidate]) -> String {
    candidates
        .iter()
        .enumerate()
        .map(|(i, c)| format!("  {}) {}", i + 1, describe(c)))
        .collect::<Vec<_>>()
        .join("\n")
}

/// What `--add` does with the sessions it found.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Plan {
    /// Nothing to join: start GitPoulpi normally.
    Start,
    /// Join `candidates[i]`.
    Join(usize),
    /// Several: the user picks.
    Ask,
}

/// `wanted` is `--session NAME` (already sanitised): that session when it is
/// running, otherwise a normal start in it. Without it: none -> start, one ->
/// join it, several -> ask.
pub fn plan(candidates: &[Candidate], wanted: Option<&str>) -> Plan {
    match wanted {
        Some(name) => candidates
            .iter()
            .position(|c| c.session == name)
            .map_or(Plan::Start, Plan::Join),
        None => match candidates.len() {
            0 => Plan::Start,
            1 => Plan::Join(0),
            _ => Plan::Ask,
        },
    }
}

/// What the user typed at the menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Choice {
    /// The 0-based index.
    Pick(usize),
    /// An empty line, or Esc (the terminal sends it before Enter).
    Cancel,
    /// Not a number of the menu: ask again.
    Invalid,
}

/// Reads one line of menu input for `count` sessions.
pub fn parse_choice(input: &str, count: usize) -> Choice {
    let text = input.trim_matches(|c: char| c.is_whitespace());
    if text.is_empty() || text.starts_with('\u{1b}') {
        return Choice::Cancel;
    }
    match text.parse::<usize>() {
        Ok(n) if (1..=count).contains(&n) => Choice::Pick(n - 1),
        _ => Choice::Invalid,
    }
}

/// The repository that encloses `path` (the current directory when none).
pub fn resolve_repo(path: Option<&Path>) -> Result<RepoId, String> {
    let cwd = || {
        std::env::current_dir().map_err(|e| format!("could not read the current directory: {e}"))
    };
    let start = match path {
        Some(p) if p.is_absolute() => p.to_path_buf(),
        Some(p) => cwd()?.join(p),
        None => cwd()?,
    };
    let start = gitpoulpi_core::paths::canonicalize_display(&start);
    let root = find_repo_root(&start)
        .ok_or_else(|| format!("{} is not inside a git repository", start.display()))?;
    Ok(RepoRef::from_root(&root).id)
}

fn find_repo_root(start: &Path) -> Option<PathBuf> {
    let mut dir = start;
    loop {
        if dir.join(".git").exists() {
            return Some(dir.to_path_buf());
        }
        dir = dir.parent()?;
    }
}

/// Windows only: lets `pid` take the foreground. A process may only do so when
/// the process that has the foreground right allows it, and the command the
/// user just typed is what gave us that right.
#[cfg(windows)]
pub fn allow_foreground(pid: u32) {
    #[link(name = "user32")]
    extern "system" {
        fn AllowSetForegroundWindow(process_id: u32) -> i32;
    }
    // SAFETY: plain value argument, no pointers.
    unsafe {
        AllowSetForegroundWindow(pid);
    }
}

#[cfg(not(windows))]
pub fn allow_foreground(_pid: u32) {}

/// Asks a window to bring its terminal window forward (best effort: not every
/// desktop lets a terminal raise itself). `None`: the daemon did not answer.
pub async fn raise_window(port: u16, viewer: &ViewerId) -> Option<RaiseAnswer> {
    let work = async {
        let mut client = Client::connect_port(port).await.ok()?;
        let request = Request {
            id: 1,
            session: String::new(),
            repo: None,
            command: OperationKind::UiQuery,
            args: serde_json::to_value(UiQueryArgs::Raise {
                viewer: Some(viewer.clone()),
            })
            .ok()?,
        };
        let response = client.call(&request).await.ok()?;
        serde_json::from_value(response.result.ok()?).ok()
    };
    let wait = Duration::from_millis(RAISE_ANSWER_TIMEOUT_MS) + PROBE_TIMEOUT;
    tokio::time::timeout(wait, work).await.ok().flatten()
}

/// What joining a session did.
#[derive(Debug, Clone, PartialEq)]
pub struct Joined {
    pub session: String,
    pub repo: RepoId,
    /// The repository was already a tab of the window: it was shown, not added.
    pub already_open: bool,
    /// The window brought its terminal forward.
    pub raised: bool,
    pub outcome: UiOutcome,
}

impl Joined {
    /// One plain line for the terminal.
    pub fn line(&self) -> String {
        let name = repo_name(&self.repo);
        let session = &self.session;
        match &self.outcome {
            UiOutcome::Applied { .. } | UiOutcome::Partial { .. } => {
                if self.already_open {
                    format!("showed the open tab {name} in session `{session}`")
                } else {
                    format!("opened {name} as a new tab in session `{session}`")
                }
            }
            UiOutcome::Deferred { .. } => format!(
                "session `{session}` is busy: it offers {name} on its footer chip, \
                 accept it there"
            ),
            UiOutcome::Refused { reason } => format!("session `{session}` refused: {reason}"),
            UiOutcome::NotFound { what } => {
                format!("session `{session}` could not open it: {what}")
            }
            UiOutcome::NoViewer { message } => message.clone(),
            UiOutcome::Proposed { .. } => format!("session `{session}` answered"),
        }
    }
}

/// Opens `repo` as a tab of `candidate`'s window and brings that window
/// forward when `raise` (best effort).
pub async fn join_session(
    candidate: &Candidate,
    repo: &RepoId,
    raise: bool,
) -> Result<Joined, String> {
    let window = candidate
        .best_window(repo)
        .ok_or_else(|| format!("session `{}` has no open window", candidate.session))?;
    let already_open = window.tabs.contains(repo);
    let mut raised = false;
    if raise {
        allow_foreground(window.pid);
        raised = raise_window(candidate.port, &window.id)
            .await
            .is_some_and(|answer| answer.raised);
    }
    let args = UiShowArgs {
        viewer: Some(window.id.clone()),
        intent: UiIntent::ShowTab { repo: repo.clone() },
        origin: UiOrigin::add_command(),
        timeout_ms: None,
    };
    let mut client = Client::connect_port(candidate.port)
        .await
        .map_err(|e| format!("session `{}`: {e}", candidate.session))?;
    let request = Request {
        id: 1,
        session: candidate.session.clone(),
        repo: None,
        command: OperationKind::UiShow,
        args: serde_json::to_value(&args).map_err(|e| e.to_string())?,
    };
    let response = client
        .call(&request)
        .await
        .map_err(|e| format!("session `{}`: {e}", candidate.session))?;
    let value = response.result.map_err(|e| e.message)?;
    let outcome: UiOutcome = serde_json::from_value(value).map_err(|e| e.to_string())?;
    Ok(Joined {
        session: candidate.session.clone(),
        repo: repo.clone(),
        already_open,
        raised,
        outcome,
    })
}

/// The command line of `--add`.
#[derive(Debug, Clone, Default)]
pub struct AddArgs {
    /// The folder to open; the current directory when none.
    pub path: Option<PathBuf>,
    /// `--session NAME`, sanitised: skips the question.
    pub session: Option<String>,
    /// Do not try to bring the window forward.
    pub no_raise: bool,
}

/// How `--add` ended.
#[derive(Debug, Clone, PartialEq)]
pub enum AddOutcome {
    Joined(Box<Joined>),
    /// No session to join: the caller starts GitPoulpi normally here.
    Start,
    /// The user left the menu.
    Cancelled,
}

/// `--add` over the sessions already `found`. `interactive`: stdin is a
/// terminal, so several sessions are resolved by asking on `out`/`input`.
pub async fn add_with(
    args: &AddArgs,
    found: Vec<SessionViewers>,
    interactive: bool,
    input: &mut dyn BufRead,
    out: &mut dyn Write,
) -> Result<AddOutcome, String> {
    let cands = candidates(found);
    let chosen = match plan(&cands, args.session.as_deref()) {
        Plan::Start => return Ok(AddOutcome::Start),
        Plan::Join(i) => i,
        Plan::Ask => {
            let names: Vec<&str> = cands.iter().map(|c| c.session.as_str()).collect();
            if !interactive {
                return Err(format!(
                    "several GitPoulpi sessions are open ({}): pass --session NAME to choose one",
                    names.join(", ")
                ));
            }
            match ask(&cands, input, out) {
                Some(i) => i,
                None => return Ok(AddOutcome::Cancelled),
            }
        }
    };
    let repo = resolve_repo(args.path.as_deref())?;
    join_session(&cands[chosen], &repo, !args.no_raise)
        .await
        .map(|joined| AddOutcome::Joined(Box::new(joined)))
}

/// The menu: numbers, Enter on a number picks it; an empty line or Esc leaves.
fn ask(cands: &[Candidate], input: &mut dyn BufRead, out: &mut dyn Write) -> Option<usize> {
    let _ = writeln!(
        out,
        "Several GitPoulpi sessions are open. Join which one?\n{}",
        menu(cands)
    );
    loop {
        let _ = write!(out, "Number then Enter (just Enter or Esc to cancel): ");
        let _ = out.flush();
        let mut line = String::new();
        match input.read_line(&mut line) {
            Ok(0) | Err(_) => return None,
            Ok(_) => {}
        }
        match parse_choice(&line, cands.len()) {
            Choice::Pick(i) => return Some(i),
            Choice::Cancel => return None,
            Choice::Invalid => {
                let _ = writeln!(out, "Type a number from 1 to {}.", cands.len());
            }
        }
    }
}

/// `--add` on the real terminal: scans the running sessions, asks on stderr
/// and reads stdin when it has to.
pub async fn add(args: &AddArgs) -> Result<AddOutcome, String> {
    use std::io::IsTerminal;
    let found = scan_sessions_within(None, PROBE_TIMEOUT).await;
    let interactive = std::io::stdin().is_terminal();
    let stdin = std::io::stdin();
    let mut input = stdin.lock();
    let mut err = std::io::stderr();
    add_with(args, found, interactive, &mut input, &mut err).await
}

/// What a binary does after [`run_add`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddExit {
    /// Nothing to join: the binary starts GitPoulpi normally.
    Start,
    /// The command is over: exit with this code.
    Exit(i32),
}

/// `--add` start to finish for a binary's `main`: scans, asks, joins, prints
/// the one line (or the JSON object with `json`) and says what to do next.
/// Errors go to stderr (a JSON object on stdout with `json`) and exit 1.
pub fn run_add(args: &AddArgs, json: bool) -> AddExit {
    let runtime = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => return fail(&e.to_string(), json),
    };
    match runtime.block_on(add(args)) {
        Ok(AddOutcome::Start) => AddExit::Start,
        Ok(outcome) => {
            if json {
                println!("{}", to_json(&outcome));
            } else {
                match &outcome {
                    AddOutcome::Joined(j) => println!("{}", j.line()),
                    _ => eprintln!("cancelled"),
                }
            }
            AddExit::Exit(exit_code(&outcome))
        }
        Err(message) => fail(&message, json),
    }
}

fn fail(message: &str, json: bool) -> AddExit {
    if json {
        println!("{}", serde_json::json!({ "error": message }));
    } else {
        eprintln!("gitpoulpi: {message}");
    }
    AddExit::Exit(1)
}

/// `gitpoulpi --add` as JSON.
pub fn to_json(outcome: &AddOutcome) -> serde_json::Value {
    match outcome {
        AddOutcome::Joined(j) => serde_json::json!({
            "action": "joined",
            "session": j.session,
            "repo": j.repo.0,
            "already_open": j.already_open,
            "raised": j.raised,
            "outcome": serde_json::to_value(&j.outcome).unwrap_or(serde_json::Value::Null),
        }),
        AddOutcome::Start => serde_json::json!({ "action": "start" }),
        AddOutcome::Cancelled => serde_json::json!({ "action": "cancelled" }),
    }
}

/// The process exit code of an outcome: like `ui show` (3 when the window
/// deferred it to its chip), 1 when it did not happen.
pub fn exit_code(outcome: &AddOutcome) -> i32 {
    match outcome {
        AddOutcome::Joined(j) => j.outcome.exit_code(),
        AddOutcome::Start => 0,
        AddOutcome::Cancelled => 130,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn viewer(id: &str, active: &str, tabs: &[&str]) -> ViewerInfo {
        ViewerInfo {
            id: ViewerId(id.into()),
            pid: 1,
            active_repo: Some(RepoId(active.into())),
            tabs: tabs.iter().map(|t| RepoId((*t).into())).collect(),
            focused: None,
            agent_control: true,
            last_focus_at: None,
        }
    }

    fn found(name: &str, port: u16, viewers: Vec<ViewerInfo>) -> SessionViewers {
        SessionViewers {
            session: name.into(),
            port,
            viewers,
        }
    }

    #[tokio::test]
    async fn twenty_sleeping_probes_take_one_wait_not_twenty() {
        let dir = tempfile::tempdir().unwrap();
        for i in 0..20 {
            std::fs::write(
                dir.path().join(format!("s{i:02}.port")),
                (40000 + i).to_string(),
            )
            .unwrap();
        }
        let wait = Duration::from_millis(300);
        let started = std::time::Instant::now();
        let found = scan_dir(
            dir.path(),
            None,
            wait,
            &|_| true,
            |_, patience| async move {
                tokio::time::sleep(patience).await;
                None
            },
        )
        .await;
        assert!(found.is_empty());
        assert!(
            started.elapsed() < wait * 2,
            "20 probes of {wait:?} took {:?}",
            started.elapsed()
        );
    }

    #[tokio::test]
    async fn a_scan_skips_dead_process_files_and_keeps_the_order() {
        let dir = tempfile::tempdir().unwrap();
        for (name, port) in [("a", 1u16), ("b", 2), ("c", 3), ("d", 4)] {
            std::fs::write(dir.path().join(format!("{name}.port")), port.to_string()).unwrap();
        }
        std::fs::write(dir.path().join("b.host"), "111").unwrap();
        let probed = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let log = probed.clone();
        let found = scan_dir(
            dir.path(),
            Some("d"),
            Duration::from_millis(100),
            &|pid| pid != 111,
            move |port, _| {
                log.lock().unwrap().push(port);
                // The first sessions answer last: the order must not follow it.
                let delay = 40 - u64::from(port) * 10;
                async move {
                    tokio::time::sleep(Duration::from_millis(delay)).await;
                    Some(Vec::new())
                }
            },
        )
        .await;
        let names: Vec<_> = found.iter().map(|f| f.session.as_str()).collect();
        assert_eq!(names, ["a", "c"]);
        assert!(
            !dir.path().join("b.port").exists(),
            "the dead one is deleted"
        );
        let mut probed = probed.lock().unwrap().clone();
        probed.sort();
        assert_eq!(probed, [1, 3], "no probe for the dead or the skipped one");
    }

    fn two() -> Vec<Candidate> {
        candidates(vec![
            found(
                "work",
                2,
                vec![viewer("a", "/p/api", &["/p/api", "/p/web"])],
            ),
            found("home", 1, vec![viewer("b", "/h/notes", &["/h/notes"])]),
        ])
    }

    #[test]
    fn only_sessions_with_a_window_are_candidates_and_they_are_sorted() {
        let c = candidates(vec![
            found("z", 3, vec![viewer("a", "/a", &["/a"])]),
            found("no-window", 4, vec![]),
            found("a", 5, vec![viewer("b", "/b", &["/b"])]),
        ]);
        let names: Vec<_> = c.iter().map(|c| c.session.as_str()).collect();
        assert_eq!(names, ["a", "z"]);
    }

    #[test]
    fn the_plan_follows_how_many_sessions_run() {
        let one = &two()[..1];
        assert_eq!(plan(&[], None), Plan::Start);
        assert_eq!(plan(one, None), Plan::Join(0));
        assert_eq!(plan(&two(), None), Plan::Ask);
    }

    #[test]
    fn a_named_session_skips_the_question_or_starts_when_it_is_not_running() {
        let c = two();
        assert_eq!(plan(&c, Some("work")), Plan::Join(1));
        assert_eq!(plan(&c, Some("home")), Plan::Join(0));
        assert_eq!(plan(&c, Some("other")), Plan::Start);
        assert_eq!(plan(&[], Some("work")), Plan::Start);
    }

    #[test]
    fn menu_input_is_a_number_of_the_menu_or_a_way_out() {
        assert_eq!(parse_choice("1\n", 3), Choice::Pick(0));
        assert_eq!(parse_choice(" 3 \r\n", 3), Choice::Pick(2));
        assert_eq!(parse_choice("4", 3), Choice::Invalid);
        assert_eq!(parse_choice("0", 3), Choice::Invalid);
        assert_eq!(parse_choice("work", 3), Choice::Invalid);
        assert_eq!(parse_choice("-1", 3), Choice::Invalid);
        assert_eq!(parse_choice("\n", 3), Choice::Cancel);
        assert_eq!(parse_choice("", 3), Choice::Cancel);
        assert_eq!(parse_choice("\u{1b}\n", 3), Choice::Cancel);
    }

    #[test]
    fn a_session_is_described_by_its_tab_count_and_repo_names() {
        let c = two();
        assert_eq!(describe(&c[0]), "home  (1 tab: notes)");
        assert_eq!(describe(&c[1]), "work  (2 tabs: api, web)");
        let many = candidates(vec![found(
            "big",
            1,
            vec![
                viewer("a", "/r/a", &["/r/a", "/r/b", "/r/c"]),
                viewer("b", "/r/d", &["/r/d", "/r/e", "/r/a"]),
            ],
        )]);
        assert_eq!(
            describe(&many[0]),
            "big  (5 tabs: a, b, c, d, +1, 2 windows)"
        );
        assert_eq!(
            menu(&c),
            "  1) home  (1 tab: notes)\n  2) work  (2 tabs: api, web)"
        );
    }

    #[test]
    fn the_window_that_gets_the_tab_shows_the_repo_else_has_it_else_is_newest() {
        let c = candidates(vec![found(
            "s",
            1,
            vec![
                viewer("recent", "/x", &["/x"]),
                viewer("has", "/y", &["/y", "/z"]),
                viewer("shows", "/z", &["/z"]),
            ],
        )]);
        let id = |r: &str| c[0].best_window(&RepoId(r.into())).unwrap().id.0.clone();
        assert_eq!(id("/z"), "shows");
        assert_eq!(id("/y"), "has");
        assert_eq!(id("/new"), "recent");
    }

    #[test]
    fn a_subfolder_resolves_to_its_repository_and_a_plain_folder_is_refused() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("repo");
        std::fs::create_dir_all(root.join(".git")).unwrap();
        std::fs::create_dir_all(root.join("src/deep")).unwrap();
        let want = RepoRef::from_root(&root).id;
        assert_eq!(resolve_repo(Some(&root)).unwrap(), want);
        assert_eq!(resolve_repo(Some(&root.join("src/deep"))).unwrap(), want);
        let plain = tmp.path().join("plain");
        std::fs::create_dir_all(&plain).unwrap();
        // The temp dir itself may sit inside a repository on a dev machine:
        // only the message is pinned, when it fails.
        if let Err(message) = resolve_repo(Some(&plain)) {
            assert!(
                message.contains("is not inside a git repository"),
                "{message}"
            );
        }
    }

    #[test]
    fn the_exit_code_is_the_window_answer_and_json_names_the_action() {
        let joined = AddOutcome::Joined(Box::new(Joined {
            session: "s".into(),
            repo: RepoId("/r".into()),
            already_open: false,
            raised: true,
            outcome: UiOutcome::Deferred {
                reason: gitpoulpi_ipc::ui::DeferReason::InputBusy,
            },
        }));
        assert_eq!(exit_code(&joined), 3);
        assert_eq!(to_json(&joined)["action"], "joined");
        assert_eq!(to_json(&joined)["raised"], true);
        assert_eq!(exit_code(&AddOutcome::Start), 0);
        assert_eq!(to_json(&AddOutcome::Cancelled)["action"], "cancelled");
    }

    #[tokio::test]
    async fn several_sessions_without_a_terminal_say_to_pass_the_session() {
        let found = vec![
            found("a", 1, vec![viewer("v", "/a", &["/a"])]),
            found("b", 2, vec![viewer("w", "/b", &["/b"])]),
        ];
        let mut input = std::io::empty();
        let mut out = Vec::new();
        let err = add_with(&AddArgs::default(), found, false, &mut input, &mut out)
            .await
            .unwrap_err();
        assert!(
            err.contains("--session NAME") && err.contains("a, b"),
            "{err}"
        );
    }

    #[tokio::test]
    async fn the_menu_loops_on_a_bad_number_and_cancels_on_enter_or_end_of_input() {
        let found = || {
            vec![
                found("a", 1, vec![viewer("v", "/a", &["/a"])]),
                found("b", 2, vec![viewer("w", "/b", &["/b"])]),
            ]
        };
        let args = AddArgs::default();
        let mut out = Vec::new();
        let mut input = std::io::Cursor::new(b"9\nx\n\n".to_vec());
        let r = add_with(&args, found(), true, &mut input, &mut out)
            .await
            .unwrap();
        assert_eq!(r, AddOutcome::Cancelled);
        let shown = String::from_utf8(out).unwrap();
        assert!(shown.contains("1) a") && shown.contains("2) b"), "{shown}");
        assert_eq!(shown.matches("Type a number from 1 to 2.").count(), 2);
        let mut eof = std::io::empty();
        let r = add_with(&args, found(), true, &mut eof, &mut Vec::new())
            .await
            .unwrap();
        assert_eq!(r, AddOutcome::Cancelled);
    }

    #[tokio::test]
    async fn no_running_session_means_start_normally() {
        let mut input = std::io::empty();
        let r = add_with(
            &AddArgs::default(),
            vec![],
            true,
            &mut input,
            &mut Vec::new(),
        )
        .await
        .unwrap();
        assert_eq!(r, AddOutcome::Start);
    }
}
