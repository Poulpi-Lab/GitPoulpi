//! Headless tests of the Refs sidebar, the Log's paging and the diff pane on GPUI's test platform,
//! against fixture repositories and a temporary config directory.

use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

use gitpoulpi_app::graph::TotalShown;
use gitpoulpi_core::domain::{DiffLine, FileDiff, FileStatus, Hunk, HunkId, LineOrigin};
use gitpoulpi_desktop::app::{self, Overrides, Workspace};
use gitpoulpi_desktop::daemon::Backend;
use gitpoulpi_desktop::panels::diff::{DiffPane, DiffSource};
use gitpoulpi_desktop::panels::refs::SectionKind;
use gitpoulpi_testkit::{isolate_process_env, FixtureRepo, FixtureRepoBuilder};
use gpui_kit::{AnyWindowHandle, AppContext as _, Entity, TestAppContext};

fn three_commits() -> FixtureRepoBuilder {
    isolate_process_env();
    FixtureRepoBuilder::new()
        .unwrap()
        .with_initial()
        .unwrap()
        .commit_file("a.txt", "a", "second")
        .unwrap()
        .commit_file("b.txt", "b", "third")
        .unwrap()
}

/// Three commits on the default branch and one more, "side work", on the branch `side`.
fn with_side_branch() -> FixtureRepo {
    let repo = three_commits().build();
    let main = repo
        .git(&["rev-parse", "--abbrev-ref", "HEAD"])
        .trim()
        .to_string();
    repo.git(&["checkout", "-b", "side"]);
    std::fs::write(repo.root().join("s.txt"), "s").unwrap();
    repo.git(&["add", "s.txt"]);
    repo.git(&["commit", "-m", "side work"]);
    repo.git(&["checkout", &main]);
    repo
}

fn open(
    cx: &mut TestAppContext,
    repo: &Path,
    prefs_dir: &Path,
) -> (AnyWindowHandle, Entity<Workspace>) {
    let backend = Backend::start(repo).unwrap();
    cx.executor().allow_parking();
    cx.update(|cx| app::init(cx, Some(prefs_dir.to_path_buf()), &Overrides::default()).unwrap());
    cx.update(|cx| app::open_main_window(backend, cx).unwrap())
}

/// Runs the app until `done` holds (the daemon answers on its own runtime, so this polls).
fn wait_until(
    cx: &mut TestAppContext,
    what: &str,
    mut done: impl FnMut(&mut TestAppContext) -> bool,
) {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        cx.run_until_parked();
        if done(cx) {
            return;
        }
        assert!(Instant::now() < deadline, "timed out waiting for {what}");
        std::thread::sleep(Duration::from_millis(25));
    }
}

fn shown_commits(cx: &mut TestAppContext, ws: &Entity<Workspace>) -> usize {
    ws.read_with(cx, |ws, cx| ws.log.read(cx).loaded_commits())
}

fn frames(cx: &mut TestAppContext, window: AnyWindowHandle, n: usize) {
    for _ in 0..n {
        cx.update_window(window, |_, window, _| window.refresh())
            .ok();
        cx.run_until_parked();
    }
}

#[gpui_kit::test]
fn the_refs_sidebar_lists_the_terminal_uis_sections_and_folders(cx: &mut TestAppContext) {
    let repo = with_side_branch();
    repo.git(&["branch", "feature/login"]);
    repo.git(&["tag", "v1"]);
    let prefs = tempfile::tempdir().unwrap();
    let (_window, ws) = open(cx, repo.root(), prefs.path());
    wait_until(cx, "the refs", |cx| {
        !ws.read_with(cx, |ws, cx| ws.refs.read(cx).row_labels(SectionKind::Local))
            .is_empty()
    });
    let (titles, local, tags) = ws.read_with(cx, |ws, cx| {
        let r = ws.refs.read(cx);
        (
            r.section_titles(),
            r.row_labels(SectionKind::Local),
            r.row_labels(SectionKind::Tags),
        )
    });
    assert_eq!(titles.len(), 3, "LOCAL, REMOTE and TAGS: {titles:?}");
    assert!(
        titles[0].ends_with("(3)"),
        "three local branches: {titles:?}"
    );
    assert_eq!(local[0], "feature", "folders come first");
    assert!(local.contains(&"login".to_string()) && local.contains(&"side".to_string()));
    assert_eq!(tags, vec!["v1"]);
}

#[gpui_kit::test]
fn stashes_and_linked_worktrees_get_their_sections_in_the_shared_order(cx: &mut TestAppContext) {
    let repo = three_commits().build();
    repo.git(&["tag", "v1"]);
    std::fs::write(repo.root().join("a.txt"), "changed").unwrap();
    repo.git(&["stash", "push", "-m", "parked"]);
    let linked = repo.root().with_extension("linked");
    repo.git(&["worktree", "add", linked.to_str().unwrap(), "-b", "other"]);
    let prefs = tempfile::tempdir().unwrap();
    let (_window, ws) = open(cx, repo.root(), prefs.path());
    wait_until(cx, "the stash and worktree sections", |cx| {
        ws.read_with(cx, |ws, cx| ws.refs.read(cx).section_titles().len() == 5)
    });
    let titles = ws.read_with(cx, |ws, cx| ws.refs.read(cx).section_titles());
    let names: Vec<&str> = titles
        .iter()
        .map(|t| t.split(" (").next().unwrap_or(""))
        .collect();
    use gitpoulpi_app::palette::vocab::refs_sidebar::SidebarSection as S;
    assert_eq!(
        names,
        vec![
            S::Local.display_title(),
            S::Remote.display_title(),
            S::Worktrees.display_title(),
            S::Stashes.display_title(),
            S::Tags.display_title(),
        ],
        "the shared order, no placeholder for pull requests or issues: {titles:?}"
    );
    let _ = std::fs::remove_dir_all(&linked);
}

#[gpui_kit::test]
fn hiding_a_ref_leaves_its_commits_out_of_the_log_and_is_remembered(cx: &mut TestAppContext) {
    let repo = with_side_branch();
    let prefs = tempfile::tempdir().unwrap();
    let (_window, ws) = open(cx, repo.root(), prefs.path());
    wait_until(cx, "four commits", |cx| shown_commits(cx, &ws) == 4);

    ws.update(cx, |ws, cx| {
        ws.refs.update(cx, |r, cx| r.toggle_hidden("side", cx))
    });
    wait_until(cx, "the Log without the hidden branch", |cx| {
        shown_commits(cx, &ws) == 3
    });
    assert!(ws.read_with(cx, |ws, cx| ws.refs.read(cx).hidden().contains("side")));
    // Written where the terminal UI reads it (through the daemon, so it lands a moment later).
    wait_until(cx, "the hidden set on disk", |_| {
        gitpoulpi_app::refs_tree::load_ref_visibility(repo.root())
            .0
            .contains("side")
    });

    ws.update(cx, |ws, cx| {
        ws.refs.update(cx, |r, cx| r.show_all_hidden(cx))
    });
    wait_until(cx, "the Log with every ref again", |cx| {
        shown_commits(cx, &ws) == 4
    });
    wait_until(cx, "the hidden set cleared on disk", |_| {
        gitpoulpi_app::refs_tree::load_ref_visibility(repo.root())
            .0
            .is_empty()
    });
}

#[gpui_kit::test]
fn soloing_a_ref_shows_only_what_it_reaches(cx: &mut TestAppContext) {
    let repo = with_side_branch();
    let prefs = tempfile::tempdir().unwrap();
    let (_window, ws) = open(cx, repo.root(), prefs.path());
    wait_until(cx, "four commits", |cx| shown_commits(cx, &ws) == 4);
    let main = repo
        .git(&["rev-parse", "--abbrev-ref", "HEAD"])
        .trim()
        .to_string();

    ws.update(cx, |ws, cx| {
        ws.refs.update(cx, |r, cx| r.toggle_solo(&main, cx))
    });
    wait_until(cx, "the soloed Log", |cx| shown_commits(cx, &ws) == 3);
    wait_until(cx, "the solo set on disk", |_| {
        gitpoulpi_app::refs_tree::load_ref_visibility(repo.root())
            .1
            .contains(&main)
    });
    ws.update(cx, |ws, cx| ws.refs.update(cx, |r, cx| r.clear_solo(cx)));
    wait_until(cx, "the whole Log", |cx| shown_commits(cx, &ws) == 4);
}

/// A second frontend of the same session sets the hidden and soloed refs through the daemon.
fn set_visibility_elsewhere(session: &str, repo: &Path, hidden: &[&str], solo: &[&str]) {
    let link = gitpoulpi_link::start(
        session.to_string(),
        gitpoulpi_core::domain::RepoRef::from_root(repo),
        gitpoulpi_desktop::daemon::DesktopHost::detached(),
    );
    link.submit_action(gitpoulpi_link::Action::SetRefVisibility {
        hidden: hidden.iter().map(|s| s.to_string()).collect(),
        solo: solo.iter().map(|s| s.to_string()).collect(),
        hidden_remotes: Vec::new(),
    });
    // Queued on the link's thread: wait for the write to land before the link is dropped.
    let want_hidden = hidden.iter().map(|s| s.to_string()).collect();
    let deadline = Instant::now() + Duration::from_secs(20);
    while gitpoulpi_app::refs_tree::load_ref_visibility(repo).0 != want_hidden {
        assert!(
            Instant::now() < deadline,
            "the other client's write never landed"
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}

#[gpui_kit::test]
fn hiding_a_ref_from_another_frontend_updates_the_refs_panel_and_the_log(cx: &mut TestAppContext) {
    let repo = with_side_branch();
    let prefs = tempfile::tempdir().unwrap();
    let backend = Backend::start(repo.root()).unwrap();
    let session = backend.session().to_string();
    cx.executor().allow_parking();
    cx.update(|cx| app::init(cx, Some(prefs.path().to_path_buf()), &Overrides::default()).unwrap());
    let (_window, ws) = cx.update(|cx| app::open_main_window(backend, cx).unwrap());
    wait_until(cx, "four commits", |cx| shown_commits(cx, &ws) == 4);

    set_visibility_elsewhere(&session, repo.root(), &["side"], &[]);
    wait_until(cx, "the Log without the hidden branch", |cx| {
        shown_commits(cx, &ws) == 3
    });
    assert!(ws.read_with(cx, |ws, cx| ws.refs.read(cx).hidden().contains("side")));

    set_visibility_elsewhere(&session, repo.root(), &[], &[]);
    wait_until(cx, "the Log with every ref again", |cx| {
        shown_commits(cx, &ws) == 4
    });
    assert!(ws.read_with(cx, |ws, cx| ws.refs.read(cx).hidden().is_empty()));
}

#[gpui_kit::test]
fn a_visibility_event_does_not_undo_a_change_whose_write_is_still_out(cx: &mut TestAppContext) {
    let repo = with_side_branch();
    let prefs = tempfile::tempdir().unwrap();
    let (_window, ws) = open(cx, repo.root(), prefs.path());
    wait_until(cx, "four commits", |cx| shown_commits(cx, &ws) == 4);

    // The daemon's event arrives while this window's own hide has not been stored yet: reading the
    // stored sets now would bring "side" back.
    ws.update(cx, |ws, cx| {
        ws.refs.update(cx, |r, cx| {
            r.toggle_hidden("side", cx);
            r.on_visibility_event(cx);
            assert!(
                r.visibility_event_deferred(),
                "the event waits for this window's own write"
            );
        })
    });
    wait_until(cx, "the hidden set on disk", |_| {
        gitpoulpi_app::refs_tree::load_ref_visibility(repo.root())
            .0
            .contains("side")
    });
    wait_until(cx, "the Log without the hidden branch", |cx| {
        shown_commits(cx, &ws) == 3
    });
    // Let any stale read land, then check nothing was undone.
    for _ in 0..20 {
        cx.run_until_parked();
        std::thread::sleep(Duration::from_millis(25));
    }
    assert!(ws.read_with(cx, |ws, cx| ws.refs.read(cx).hidden().contains("side")));
    assert!(!ws.read_with(cx, |ws, cx| ws.refs.read(cx).visibility_event_deferred()));
    assert_eq!(shown_commits(cx, &ws), 3);
}

#[gpui_kit::test]
fn the_keyboard_walks_the_rows_and_enter_reveals_the_ref_in_the_log(cx: &mut TestAppContext) {
    let repo = with_side_branch();
    let prefs = tempfile::tempdir().unwrap();
    let (_window, ws) = open(cx, repo.root(), prefs.path());
    wait_until(cx, "four commits", |cx| shown_commits(cx, &ws) == 4);
    wait_until(cx, "the refs", |cx| {
        !ws.read_with(cx, |ws, cx| ws.refs.read(cx).row_labels(SectionKind::Local))
            .is_empty()
    });
    let before = ws.read_with(cx, |ws, cx| ws.log.read(cx).selected_row());
    assert_eq!(
        before, 0,
        "the Log starts on its newest commit, the side branch's"
    );
    let main = repo
        .git(&["rev-parse", "--abbrev-ref", "HEAD"])
        .trim()
        .to_string();
    // The rows of LOCAL are the default branch then `side`: walk down to it and reveal it.
    for _ in 0..4 {
        ws.update(cx, |ws, cx| {
            ws.refs.update(cx, |r, cx| r.move_selection(1, cx))
        });
        let label = ws.read_with(cx, |ws, cx| ws.refs.read(cx).selected_label());
        if label.as_deref() == Some(main.as_str()) {
            break;
        }
    }
    assert_eq!(
        ws.read_with(cx, |ws, cx| ws.refs.read(cx).selected_label())
            .as_deref(),
        Some(main.as_str())
    );
    ws.update(cx, |ws, cx| ws.refs.update(cx, |r, cx| r.activate(cx)));
    cx.run_until_parked();
    let after = ws.read_with(cx, |ws, cx| ws.log.read(cx).selected_row());
    assert_eq!(
        after, 1,
        "Enter moved the Log to the commit the branch points at"
    );
}

/// The config directory is process-wide: tests that write `config.toml` take turns.
static CONFIG: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn config_turn() -> std::sync::MutexGuard<'static, ()> {
    CONFIG.lock().unwrap_or_else(|e| e.into_inner())
}

fn write_config(text: &str) {
    let dir = isolate_process_env().join("config");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("config.toml"), text).unwrap();
}

/// 520 commits: 20 past the smallest first page the config allows.
fn long_history() -> FixtureRepo {
    let mut b = FixtureRepoBuilder::new().unwrap().with_initial().unwrap();
    for i in 1..520 {
        b = b
            .commit_file("f.txt", &i.to_string(), &format!("c{i}"))
            .unwrap();
    }
    b.build()
}

#[gpui_kit::test]
fn the_log_pages_by_the_configured_sizes_and_shows_the_total(cx: &mut TestAppContext) {
    let _turn = config_turn();
    isolate_process_env();
    write_config(
        "[general]\ninitial_commits_in_graph = 500\ncommits_per_lazy_page = 50\n\
         lazy_load_commits_in_graph = true\n",
    );
    let repo = long_history();
    let prefs = tempfile::tempdir().unwrap();
    let (window, ws) = open(cx, repo.root(), prefs.path());
    wait_until(cx, "the first page", |cx| shown_commits(cx, &ws) >= 500);
    assert_eq!(
        shown_commits(cx, &ws),
        500,
        "the configured first page, not a fixed one"
    );
    wait_until(cx, "the total", |cx| {
        ws.read_with(cx, |ws, cx| ws.log.read(cx).commit_total()) == TotalShown::Of(520)
    });
    // Scrolling to the end asks for the next page of 50: the 20 left arrive and the walk is exhausted.
    ws.update(cx, |ws, cx| ws.log.update(cx, |l, cx| l.select(499, cx)));
    for _ in 0..200 {
        frames(cx, window, 1);
        if shown_commits(cx, &ws) == 520 {
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(shown_commits(cx, &ws), 520);
    wait_until(cx, "the end of history", |cx| {
        ws.read_with(cx, |ws, cx| ws.log.read(cx).is_exhausted())
    });
    assert_eq!(
        ws.read_with(cx, |ws, cx| ws.log.read(cx).commit_total()),
        TotalShown::Whole
    );
    write_config("");
}

#[gpui_kit::test]
fn with_lazy_loading_off_the_log_stops_at_the_first_page(cx: &mut TestAppContext) {
    let _turn = config_turn();
    isolate_process_env();
    write_config("[general]\ninitial_commits_in_graph = 500\nlazy_load_commits_in_graph = false\n");
    let repo = long_history();
    let prefs = tempfile::tempdir().unwrap();
    let (window, ws) = open(cx, repo.root(), prefs.path());
    wait_until(cx, "the first page", |cx| shown_commits(cx, &ws) >= 500);
    ws.update(cx, |ws, cx| ws.log.update(cx, |l, cx| l.select(499, cx)));
    for _ in 0..30 {
        frames(cx, window, 1);
        std::thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(
        shown_commits(cx, &ws),
        500,
        "nothing past the first page without lazy loading"
    );
    write_config("");
}

// -- the diff pane -----------------------------------------------------------

fn diff_line(origin: LineOrigin, text: &str) -> DiffLine {
    DiffLine {
        origin,
        content: format!("{text}\n"),
        staged: false,
    }
}

fn file_with(lines: Vec<DiffLine>) -> FileDiff {
    let old = lines.iter().filter(|l| l.origin != LineOrigin::Add).count() as u32;
    let new = lines
        .iter()
        .filter(|l| l.origin != LineOrigin::Delete)
        .count() as u32;
    FileDiff {
        path: "src/lib.rs".into(),
        status: FileStatus::Modified,
        renamed_from: None,
        similarity: None,
        hunks: vec![Hunk {
            id: HunkId(0),
            old_start: 1,
            old_lines: old,
            new_start: 1,
            new_lines: new,
            lines,
        }],
    }
}

fn open_diff(cx: &mut TestAppContext) -> (AnyWindowHandle, Entity<DiffPane>) {
    let prefs = tempfile::tempdir().unwrap();
    cx.update(|cx| app::init(cx, Some(prefs.path().to_path_buf()), &Overrides::default()).unwrap());
    cx.update(|cx| {
        gpui_kit::open_window(app::window_options(cx), cx, |window, cx| {
            cx.new(|cx| DiffPane::new(window, cx))
        })
        .unwrap()
    })
}

#[gpui_kit::test]
fn the_diff_pane_builds_both_layouts_and_follows_the_toggles(cx: &mut TestAppContext) {
    let (window, pane) = open_diff(cx);
    let file = file_with(vec![
        diff_line(LineOrigin::Context, "fn main() {"),
        diff_line(LineOrigin::Delete, "    let a = 1;"),
        diff_line(LineOrigin::Add, "    let a = 2;"),
        diff_line(LineOrigin::Context, "}"),
    ]);
    pane.update(cx, |p, cx| {
        p.show(Arc::new(vec![file]), 0, DiffSource::Commit, cx)
    });
    wait_until(cx, "the rows", |cx| {
        pane.read_with(cx, |p, _| p.row_count()) > 0
    });
    // Header + context + a paired change + context.
    assert_eq!(pane.read_with(cx, |p, _| p.row_count()), 4);
    frames(cx, window, 3);

    pane.update(cx, |p, cx| p.toggle_layout(cx));
    wait_until(cx, "the Inline rows", |cx| {
        pane.read_with(cx, |p, _| p.row_count()) == 5
    });
    assert!(!pane.read_with(cx, |p, _| p.is_split()));
    pane.update(cx, |p, cx| p.toggle_wrap(cx));
    assert!(pane.read_with(cx, |p, _| p.wraps()));
    frames(cx, window, 3);

    // Whitespace off: a whitespace-only change folds into context and the pane says there is nothing.
    let ws_only = file_with(vec![
        diff_line(LineOrigin::Delete, "x  "),
        diff_line(LineOrigin::Add, "x"),
    ]);
    pane.update(cx, |p, cx| {
        p.show(Arc::new(vec![ws_only]), 0, DiffSource::Commit, cx);
        p.toggle_whitespace(cx);
    });
    wait_until(cx, "the notice", |cx| {
        pane.read_with(cx, |p, _| p.row_count()) == 1
    });
    assert!(!pane.read_with(cx, |p, _| p.shows_whitespace()));
    frames(cx, window, 2);
}

#[gpui_kit::test]
fn a_huge_diff_builds_off_the_ui_thread_and_paints_in_bounded_time(cx: &mut TestAppContext) {
    let (window, pane) = open_diff(cx);
    let mut lines = Vec::new();
    for i in 0..40_000 {
        lines.push(diff_line(
            LineOrigin::Delete,
            &format!("old line {i} with some text"),
        ));
        lines.push(diff_line(
            LineOrigin::Add,
            &format!("new line {i} with some text"),
        ));
    }
    pane.update(cx, |p, cx| {
        p.show(Arc::new(vec![file_with(lines)]), 0, DiffSource::Commit, cx)
    });
    wait_until(cx, "the rows", |cx| {
        pane.read_with(cx, |p, _| p.row_count()) > 1
    });
    assert_eq!(pane.read_with(cx, |p, _| p.row_count()), 40_001);
    let start = Instant::now();
    frames(cx, window, 3);
    assert!(
        start.elapsed() < Duration::from_secs(10),
        "three frames of a 40k-row diff took {:?}: the list is not virtualised",
        start.elapsed()
    );
}
