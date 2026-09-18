//! An agent's view intents (`ui show diff|file`, `ui highlight`, `ui state
//! --text`) end to end: the real binary and its embedded daemon in a pty,
//! and a plain IPC client standing in for the `gitpoulpi ui ...` commands
//! (`common::ui_show`).
//!
//! Every wait names the state it waits for.

mod common;

use std::path::{Path, PathBuf};
use std::time::Duration;

use common::{git_ok, git_trimmed};
use gitpoulpi_core::domain::{Oid, RepoId};
use gitpoulpi_ipc::ui::{
    DiffAt, DiffLayout, DiffSide, FileAt, FileMode, LineRange, UiIntent, UiOutcome, UiState,
};
use xtask::pty::{Color, Terminal};

const STEP: Duration = Duration::from_secs(30);
/// The tint of an agent's highlight band (`diff_view::HIGHLIGHT_BG`).
const HIGHLIGHT: Color = Color::Rgb(92, 74, 18);

struct Rig {
    term: Terminal,
    config: tempfile::TempDir,
    session: String,
    root: PathBuf,
    _repo: tempfile::TempDir,
}

fn launch(
    session: &str,
    repo: tempfile::TempDir,
    ready: &'static str,
    hold_ms: Option<u64>,
) -> Rig {
    launch_env(session, repo, ready, hold_ms, &[])
}

fn launch_env(
    session: &str,
    repo: tempfile::TempDir,
    ready: &'static str,
    hold_ms: Option<u64>,
    envs: &[(&str, &str)],
) -> Rig {
    let root = repo.path().to_path_buf();
    let mut builder = common::launch(&root)
        .session(session)
        .size(200, 50)
        .no_exit_pause()
        .ready(ready);
    if let Some(ms) = hold_ms {
        builder = builder.env("GITPOULPI_UI_REVEAL_HOLD_MS", ms.to_string());
    }
    for (key, value) in envs {
        builder = builder.env(key, value);
    }
    let (term, config) = builder.start();
    common::wait_for_viewer(config.path(), session);
    // Past the "the user just pressed a key" deferral window.
    std::thread::sleep(Duration::from_millis(1700));
    Rig {
        term,
        config,
        session: session.into(),
        root,
        _repo: repo,
    }
}

impl Rig {
    fn repo(&self) -> RepoId {
        RepoId::from_path(&self.root)
    }

    fn show(&self, intent: UiIntent) -> UiOutcome {
        common::ui_show(self.config.path(), &self.session, intent)
    }

    fn state(&self) -> UiState {
        common::ui_state(self.config.path(), &self.session)
    }

    fn diff(&self, target: DiffAt, line: Option<u32>) -> UiIntent {
        UiIntent::ShowDiff {
            repo: self.repo(),
            target,
            path: "src/lib.rs".into(),
            line,
            side: DiffSide::New,
            layout: None,
            whole_file: None,
            highlight: None,
        }
    }

    fn file(&self, at: FileAt, mode: FileMode, path: &str, line: Option<u32>) -> UiIntent {
        UiIntent::ShowFile {
            repo: self.repo(),
            path: path.into(),
            at,
            mode,
            line,
            column: None,
            highlight: None,
        }
    }

    fn wait_for(&self, what: &str, f: impl Fn(&str) -> bool) {
        self.term
            .wait_for(what, STEP, |s| f(&s.text()))
            .unwrap_or_else(|e| panic!("{e}"));
    }

    /// The background of the first cell showing `needle`.
    fn bg_of(&self, needle: &str) -> Option<Color> {
        let screen = self.term.screen();
        let (col, row) = screen.find(needle)?;
        Some(screen.cell(col, row)?.bg)
    }
}

fn oid(hex: &str) -> Oid {
    Oid::from_hex(hex).expect("a full object id")
}

fn applied(out: UiOutcome) -> UiState {
    match out {
        UiOutcome::Applied { state } => state,
        other => panic!("expected Applied, got {other:?}"),
    }
}

fn init(dir: &Path) {
    git_ok(dir, &["init", "-q", "-b", "main"]);
    git_ok(dir, &["config", "user.name", "Test User"]);
    git_ok(dir, &["config", "user.email", "t@example.com"]);
}

fn write(dir: &Path, rel: &str, text: &str) {
    let path = dir.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, text).unwrap();
}

fn numbered(n: usize, changed: &[usize]) -> String {
    (1..=n)
        .map(|i| {
            if changed.contains(&i) {
                format!("line {i} changed\n")
            } else {
                format!("line {i}\n")
            }
        })
        .collect()
}

/// `src/lib.rs` of 300 lines; the second commit changes lines 10 and 120.
fn diff_repo() -> (tempfile::TempDir, String) {
    let dir = tempfile::tempdir().expect("temp repo");
    init(dir.path());
    write(dir.path(), "src/lib.rs", &numbered(300, &[]));
    git_ok(dir.path(), &["add", "."]);
    git_ok(dir.path(), &["commit", "-q", "-m", "first version"]);
    write(dir.path(), "src/lib.rs", &numbered(300, &[10, 120]));
    git_ok(
        dir.path(),
        &["commit", "-q", "-am", "tweak ten and one-twenty"],
    );
    let head = git_trimmed(dir.path(), &["rev-parse", "HEAD"]);
    (dir, head)
}

#[test]
fn show_diff_at_a_line_in_split_layout_tints_its_row() {
    let (repo, head) = diff_repo();
    let rig = launch("pty-ui-look-split", repo, "tweak ten", None);
    let mut intent = rig.diff(DiffAt::Commit(oid(&head)), Some(120));
    if let UiIntent::ShowDiff { layout, .. } = &mut intent {
        *layout = Some(DiffLayout::Split);
    }
    let state = applied(rig.show(intent));
    let diff = state.diff.expect("the diff in the state");
    assert_eq!(diff["layout"], "split", "{diff}");
    assert_eq!(diff["cursor_line"], 120, "{diff}");
    assert_eq!(diff["path"], "src/lib.rs");
    assert_eq!(diff["target"]["commit"], head.as_str());
    assert_eq!(diff["highlight"]["lines"], serde_json::json!([120, 120]));
    rig.wait_for("line 120 in the split diff", |t| {
        t.contains("line 120 changed")
    });
    assert_eq!(
        rig.bg_of("line 120 changed"),
        Some(HIGHLIGHT),
        "the row is tinted"
    );
    // `ui state --text`: the visible text holds the highlighted line.
    let text = common::ui_state_text(rig.config.path(), &rig.session)
        .text
        .expect("the text of the central pane");
    assert!(
        text.iter().any(|l| l.contains("line 120 changed")),
        "{text:#?}"
    );
}

#[test]
fn a_line_outside_the_hunks_switches_to_the_whole_file() {
    let (repo, head) = diff_repo();
    let rig = launch("pty-ui-look-whole", repo, "tweak ten", None);
    let state = applied(rig.show(rig.diff(DiffAt::Commit(oid(&head)), Some(200))));
    let diff = state.diff.expect("diff");
    assert_eq!(diff["whole_file"], true, "{diff}");
    assert_eq!(diff["cursor_line"], 200, "{diff}");
    rig.wait_for("line 200 on screen", |t| t.contains("line 200"));
    assert_eq!(rig.bg_of("line 200"), Some(HIGHLIGHT));
    // A hunks-only request cannot reach it: Partial, not a lie.
    let mut intent = rig.diff(DiffAt::Commit(oid(&head)), Some(200));
    if let UiIntent::ShowDiff { whole_file, .. } = &mut intent {
        *whole_file = Some(false);
    }
    let out = rig.show(intent);
    assert!(
        matches!(&out, UiOutcome::Partial { note, .. } if note.contains("not in a hunk")),
        "{out:?}"
    );
}

#[test]
fn show_diff_unstaged_on_a_modified_file() {
    let (repo, _head) = diff_repo();
    write(repo.path(), "src/lib.rs", &numbered(300, &[10, 120, 50]));
    let rig = launch("pty-ui-look-unstaged", repo, "tweak ten", None);
    let state = applied(rig.show(rig.diff(DiffAt::Unstaged, Some(50))));
    let diff = state.diff.expect("diff");
    assert_eq!(diff["target"], "unstaged", "{diff}");
    assert_eq!(diff["cursor_line"], 50, "{diff}");
    rig.wait_for("the unstaged title and the line", |t| {
        t.contains("unstaged") && t.contains("line 50 changed")
    });
    assert_eq!(rig.bg_of("line 50 changed"), Some(HIGHLIGHT));
    // Highlight on the open diff, moving the band.
    let out = rig.show(UiIntent::Highlight {
        range: LineRange {
            start_line: 52,
            end_line: 52,
            start_col: None,
            end_col: None,
        },
        scroll: true,
    });
    let state = applied(out);
    assert_eq!(
        state.diff.unwrap()["highlight"]["lines"],
        serde_json::json!([52, 52])
    );
    rig.wait_for("line 52 tinted", |t| t.contains("line 52"));
    assert_eq!(rig.bg_of("line 52"), Some(HIGHLIGHT));
}

#[test]
fn a_key_during_the_reveal_cancels_it() {
    let (repo, head) = diff_repo();
    let mut rig = launch("pty-ui-look-took-over", repo, "tweak ten", Some(3000));
    let (config, session) = (rig.config.path().to_path_buf(), rig.session.clone());
    let intent = rig.diff(DiffAt::Commit(oid(&head)), Some(120));
    let asked = std::thread::spawn(move || common::ui_show(&config, &session, intent));
    std::thread::sleep(Duration::from_millis(600));
    rig.term.send_keys("Down").unwrap();
    let out = asked.join().unwrap();
    assert_eq!(out, UiOutcome::refused("user took over"), "{out:?}");
    assert!(rig.state().diff.is_none(), "no diff was opened");
}

/// `wide.txt`: 400 lines, line 300 has `NEEDLE` at column 151; `blame.txt`
/// is blamed on another author; `hist.txt` has five commits.
fn file_repo() -> (tempfile::TempDir, Vec<String>) {
    let dir = tempfile::tempdir().expect("temp repo");
    init(dir.path());
    let wide: String = (1..=400)
        .map(|i| {
            if i == 300 {
                format!("{}NEEDLE tail\n", "a".repeat(150))
            } else {
                format!("row {i}\n")
            }
        })
        .collect();
    write(dir.path(), "wide.txt", &wide);
    git_ok(dir.path(), &["add", "."]);
    git_ok(dir.path(), &["commit", "-q", "-m", "wide file"]);
    write(dir.path(), "blame.txt", &numbered(50, &[]));
    git_ok(dir.path(), &["add", "."]);
    git_ok(
        dir.path(),
        &[
            "-c",
            "user.name=Zed Blamer",
            "commit",
            "-q",
            "-m",
            "blame target",
        ],
    );
    let mut hist = Vec::new();
    for n in 1..=5 {
        write(dir.path(), "hist.txt", &format!("version {n}\n"));
        git_ok(dir.path(), &["add", "."]);
        git_ok(dir.path(), &["commit", "-q", "-m", &format!("hist-{n}")]);
        hist.push(git_trimmed(dir.path(), &["rev-parse", "HEAD"]));
    }
    (dir, hist)
}

#[test]
fn show_file_at_a_line_and_column_scrolls_sideways_and_tints_the_line() {
    let (repo, _) = file_repo();
    let head = git_trimmed(repo.path(), &["rev-parse", "HEAD"]);
    let rig = launch("pty-ui-look-file", repo, "hist-5", None);
    let mut intent = rig.file(
        FileAt::Rev(oid(&head)),
        FileMode::Content,
        "wide.txt",
        Some(300),
    );
    if let UiIntent::ShowFile { column, .. } = &mut intent {
        *column = Some(151);
    }
    let state = applied(rig.show(intent));
    let file = state.file.expect("the file in the state");
    assert_eq!(file["mode"], "content", "{file}");
    assert_eq!(file["selected_line"], 300, "{file}");
    assert_eq!(file["h_offset"], 143, "{file}");
    rig.wait_for("the column on screen", |t| t.contains("NEEDLE tail"));
    assert_eq!(
        rig.bg_of("NEEDLE tail"),
        Some(HIGHLIGHT),
        "the line is tinted"
    );
    let text = common::ui_state_text(rig.config.path(), &rig.session)
        .text
        .expect("text");
    assert!(text.iter().any(|l| l.contains("NEEDLE tail")), "{text:#?}");
}

#[test]
fn show_blame_at_a_line_shows_the_author_on_that_row() {
    let (repo, _) = file_repo();
    let head = git_trimmed(repo.path(), &["rev-parse", "HEAD"]);
    let rig = launch("pty-ui-look-blame", repo, "hist-5", None);
    let state = applied(rig.show(rig.file(
        FileAt::Rev(oid(&head)),
        FileMode::Blame,
        "blame.txt",
        Some(30),
    )));
    assert_eq!(state.file.expect("file")["mode"], "blame");
    // One 50-line hunk: its author is printed once, on the first visible row
    // of the hunk (the annotation repeats there when the hunk's first line is
    // scrolled off), and the requested line is on screen, tinted.
    rig.wait_for("the blame row", |t| {
        t.contains("Zed Blamer") && t.lines().any(|l| l.contains("line 30"))
    });
    assert_eq!(rig.bg_of("line 30"), Some(HIGHLIGHT));
}

#[test]
fn show_history_with_a_rev_selects_that_commit_row() {
    let (repo, hist) = file_repo();
    let head = git_trimmed(repo.path(), &["rev-parse", "HEAD"]);
    let rig = launch("pty-ui-look-history", repo, "hist-5", None);
    let _ = head;
    let state = applied(rig.show(rig.file(
        FileAt::Rev(oid(&hist[2])),
        FileMode::History,
        "hist.txt",
        None,
    )));
    let file = state.file.expect("file");
    assert_eq!(file["mode"], "history", "{file}");
    // Newest first: hist-5, hist-4, hist-3.
    assert_eq!(file["selected_line"], 3, "{file}");
    rig.wait_for("the history rows", |t| t.contains("hist-3"));
}

#[test]
fn show_file_of_the_worktree_and_a_missing_line() {
    let (repo, _) = file_repo();
    let rig = launch("pty-ui-look-worktree", repo, "hist-5", None);
    let out = rig.show(rig.file(FileAt::Worktree, FileMode::Content, "blame.txt", Some(500)));
    let UiOutcome::Partial { note, state } = out else {
        panic!("expected Partial, got {out:?}");
    };
    assert!(note.contains("past the end"), "{note}");
    assert_eq!(state.file.expect("file")["at"], "worktree");
}

#[test]
fn back_closes_the_diff_an_intent_opened() {
    let (repo, head) = diff_repo();
    let rig = launch("pty-ui-look-back", repo, "tweak ten", None);
    let state = applied(rig.show(rig.diff(DiffAt::Commit(oid(&head)), Some(120))));
    assert_eq!(state.central.as_deref(), Some("diff"));
    // Past the dwell between two applied intents.
    std::thread::sleep(Duration::from_millis(600));
    let state = applied(rig.show(UiIntent::Back));
    assert_eq!(state.central.as_deref(), Some("log"), "{state:?}");
    assert!(state.diff.is_none());
}

fn highlight(line: u32) -> UiIntent {
    UiIntent::Highlight {
        range: LineRange {
            start_line: line,
            end_line: line,
            start_col: None,
            end_col: None,
        },
        scroll: true,
    }
}

#[test]
fn a_highlight_outside_the_hunks_widens_to_the_whole_file() {
    let (repo, head) = diff_repo();
    let rig = launch("pty-ui-look-hl-widen", repo, "tweak ten", None);
    let state = applied(rig.show(rig.diff(DiffAt::Commit(oid(&head)), Some(120))));
    assert_eq!(state.diff.expect("diff")["whole_file"], false);
    std::thread::sleep(Duration::from_millis(600));
    // Line 200 is in no hunk: the same widening as `ShowDiff`, no `Partial`.
    let state = applied(rig.show(highlight(200)));
    let diff = state.diff.expect("diff");
    assert_eq!(diff["whole_file"], true, "{diff}");
    assert_eq!(diff["cursor_line"], 200, "{diff}");
    assert_eq!(diff["highlight"]["lines"], serde_json::json!([200, 200]));
    rig.wait_for("line 200 on screen", |t| t.contains("line 200"));
    assert_eq!(rig.bg_of("line 200"), Some(HIGHLIGHT));
}

#[test]
fn the_highlight_band_goes_when_the_user_moves_the_cursor() {
    let (repo, head) = diff_repo();
    let mut rig = launch("pty-ui-look-hl-clear", repo, "tweak ten", None);
    let state = applied(rig.show(rig.diff(DiffAt::Commit(oid(&head)), Some(120))));
    assert_eq!(
        state.diff.expect("diff")["highlight"]["lines"],
        serde_json::json!([120, 120])
    );
    rig.wait_for("line 120 tinted", |t| t.contains("line 120 changed"));
    assert_eq!(rig.bg_of("line 120 changed"), Some(HIGHLIGHT));
    rig.term.send_keys("Down").unwrap();
    rig.wait_for("the band is gone from the state", |_| {
        rig.state().diff.is_some_and(|d| d["highlight"].is_null())
    });
    assert_ne!(rig.bg_of("line 120 changed"), Some(HIGHLIGHT));
}

#[test]
fn an_intent_that_waited_too_long_is_refused_as_stale() {
    let (repo, head) = diff_repo();
    // Everything older than 100 ms when it would apply is stale; the 400 ms
    // dwell between two applied intents makes the second one so.
    let rig = launch_env(
        "pty-ui-look-stale",
        repo,
        "tweak ten",
        None,
        &[("GITPOULPI_UI_STALE_MS", "100")],
    );
    applied(rig.show(rig.diff(DiffAt::Commit(oid(&head)), Some(120))));
    let out = rig.show(rig.diff(DiffAt::Commit(oid(&head)), Some(10)));
    assert_eq!(out, UiOutcome::refused("stale"), "{out:?}");
}

#[test]
fn the_tab_an_agent_changed_carries_a_star_until_the_user_acts() {
    let (repo, head) = diff_repo();
    let mut rig = launch("pty-ui-look-tab-mark", repo, "tweak ten", None);
    let top = |rig: &Rig| {
        (0..3)
            .map(|r| rig.term.screen().line(r))
            .collect::<String>()
    };
    assert!(!top(&rig).contains('\u{2726}'), "no marker before");
    applied(rig.show(rig.diff(DiffAt::Commit(oid(&head)), Some(120))));
    rig.wait_for("the star on the tab bar", |_| {
        top(&rig).contains('\u{2726}')
    });
    rig.term.send_keys("Down").unwrap();
    rig.wait_for("the star gone after a key", |_| {
        !top(&rig).contains('\u{2726}')
    });
}
