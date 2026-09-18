//! End to end through a pty: the TUI must not end while the user browses
//! View all files and the diffs it opens, and every end of the process
//! leaves a record in `gitpoulpi-tui-exit.log`; a file opened from View all
//! files that the commit did not change shows its content in the central
//! File View, not a diff stuck on "waiting".

mod common;

use std::path::Path;
use std::time::{Duration, Instant};

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);
const LOG_HEADER: &str = "Message";

fn launch(repo: &Path, session: &str, cols: u16, rows: u16) -> (Terminal, tempfile::TempDir) {
    common::launch(repo)
        .no_commit_column()
        .session(session)
        .size(cols, rows)
        .no_exit_pause()
        .ready_when("the commit detail of HEAD", |s| {
            s.contains("View all files") && s.contains(LOG_HEADER)
        })
        .mouse_capture()
        .start()
}

/// Two commits: the first adds `count` files spread over folders, the
/// second changes only `d0/s0/f0.txt`.
fn many_files(count: usize) -> gitpoulpi_testkit::FixtureRepo {
    let mut b = FixtureRepoBuilder::new().unwrap();
    for i in 0..count {
        b = b
            .write_worktree(
                &format!("d{}/s{}/f{i}.txt", i % 7, i % 3),
                format!("content of file {i}\n").as_bytes(),
            )
            .unwrap();
    }
    b.stage(".")
        .unwrap()
        .commit("many files")
        .unwrap()
        .commit_file("d0/s0/f0.txt", "changed\n", "one change")
        .unwrap()
        .build()
}

fn wait_for_exit(term: &Terminal, timeout: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if term.exited() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    term.exited()
}

fn exit_log(config: &Path) -> String {
    std::fs::read_to_string(config.join("gitpoulpi-tui-exit.log")).unwrap_or_default()
}

/// `q` over the central diff closes the diff; only `q` over the graph quits,
/// and that quit is recorded. Red without the fix: `q` over a diff opened
/// from View all files closed the whole window with nothing in the log.
#[test]
fn q_over_a_diff_closes_the_diff_and_only_q_over_the_graph_quits() {
    let repo = many_files(12);
    let (mut term, config) = launch(repo.root(), "pty-q-closes-diff", 160, 40);

    // Open the changed file's diff from the commit-detail list.
    let (c, r) = term
        .wait_for_text("d0/s0/f0.txt", STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.click(c, r).unwrap();
    term.wait_for("the diff to replace the Log", STEP, |s| {
        s.contains("\u{2715}") && s.contains("changed") && !s.contains(LOG_HEADER)
    })
    .unwrap_or_else(|e| panic!("{e}"));

    term.send_keys("q").unwrap();
    term.wait_for("q to close the diff, back to the graph", STEP, |s| {
        s.contains(LOG_HEADER) && !s.contains("\u{2715}")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert!(!term.exited(), "q over a diff ended the process");

    // Over the graph, q still quits -- and says so in the exit log.
    term.send_keys("q").unwrap();
    assert!(wait_for_exit(&term, STEP), "q over the graph should quit");
    let log = exit_log(config.path());
    assert!(
        log.contains("user quit (q key)") && log.contains("Log in the center"),
        "the quit was not recorded: {log:?}"
    );
}

/// In View all files, a file the commit did not change
/// opens its content at that commit, in the central column (not a popup,
/// not a diff stuck on "waiting"), and Esc goes back to the graph.
#[test]
fn an_unchanged_file_in_view_all_files_opens_as_a_central_file_view() {
    let repo = many_files(12);
    let (mut term, _config) = launch(repo.root(), "pty-vaf-unchanged", 160, 40);
    let (c, r) = term.wait_for_text("View all files", STEP).unwrap();
    term.click(c + 6, r).unwrap();
    // 12 files + the fixture's own `many files.txt`.
    term.wait_for("the whole tree", STEP, |s| {
        s.contains("13 files") && s.contains("All files (13)")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let (fc, fr) = term.wait_for_text("d5/s2/f5.txt", STEP).unwrap();
    term.click(fc, fr).unwrap();
    term.wait_for("the file's content in the central column", STEP, |s| {
        s.contains("File View - d5/s2/f5.txt @")
            && s.contains("content of file 5")
            && s.contains("\u{2715}")
            && !s.contains(LOG_HEADER)
            && !s.contains("waiting for this commit's diff")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    // The right pane is still there, beside it -- not covered by a popup.
    assert!(term.text().contains("All files (13)"));

    term.send_keys("Escape").unwrap();
    term.wait_for("Esc back to the graph", STEP, |s| {
        s.contains(LOG_HEADER) && !s.contains("content of file 5")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.stop().unwrap();
}

/// The diff bar's Blame opens in the central column, with the
/// same bar; its Diff View goes back to the diff; Esc to the graph.
#[test]
fn blame_from_the_diff_bar_takes_the_central_column_and_diff_view_returns() {
    let repo = many_files(4);
    let (mut term, _config) = launch(repo.root(), "pty-central-blame", 200, 40);
    let (c, r) = term.wait_for_text("d0/s0/f0.txt", STEP).unwrap();
    term.click(c, r).unwrap();
    let (bc, br) = term
        .wait_for_text(" Blame ", STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.click(bc + 2, br).unwrap();
    term.wait_for("Blame in the central column", STEP, |s| {
        s.contains("Blame - d0/s0/f0.txt @") && s.contains("Diff View") && !s.contains(LOG_HEADER)
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let (dc, dr) = term.wait_for_text("Diff View", STEP).unwrap();
    term.click(dc + 2, dr).unwrap();
    term.wait_for("Diff View back to the diff", STEP, |s| {
        !s.contains("Blame -") && s.contains("changed") && s.contains("\u{2715}")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Escape").unwrap();
    term.wait_for("Esc back to the graph", STEP, |s| s.contains(LOG_HEADER))
        .unwrap_or_else(|e| panic!("{e}"));
    term.stop().unwrap();
}

/// View all files on the WIP row lists the whole working tree,
/// the changes marked; an unchanged file opens its bytes on disk in the
/// File View, a changed one its diff.
#[test]
fn view_all_files_on_the_wip_row_lists_the_working_tree() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .write_worktree("a.txt", b"committed a\n")
        .unwrap()
        .write_worktree("b.txt", b"committed b\n")
        .unwrap()
        .stage(".")
        .unwrap()
        .commit("base")
        .unwrap()
        .write_worktree("b.txt", b"edited b\n")
        .unwrap()
        .write_worktree("new.txt", b"brand new\n")
        .unwrap()
        .build();
    let (mut term, _config) = launch(repo.root(), "pty-wip-view-all", 200, 40);
    // The WIP row is selected on a dirty tree; its toolbar has the box.
    term.wait_for("the working-changes pane", STEP, |s| {
        s.contains("Unstaged") && s.contains("[ ] View all files")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = term.wait_for_text("[ ] View all files", STEP).unwrap();
    term.click(c + 1, r).unwrap();
    // a.txt, b.txt, base.txt (the fixture's), new.txt.
    term.wait_for("every working-tree file", STEP, |s| {
        s.contains("All files (4)") && s.contains("a.txt") && s.contains("new.txt")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    let (ac, ar) = term.screen().find("a.txt").unwrap();
    term.click(ac, ar).unwrap();
    term.wait_for("a.txt's bytes on disk, centrally", STEP, |s| {
        s.contains("File View - a.txt @ WIP")
            && s.contains("committed a")
            && !s.contains(LOG_HEADER)
    })
    .unwrap_or_else(|e| panic!("{e}"));

    let (bc, br) = term.screen().find(" b.txt").unwrap();
    term.click(bc + 1, br).unwrap();
    term.wait_for("b.txt's unstaged diff", STEP, |s| {
        s.contains("b.txt - unstaged") && s.contains("edited b")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.stop().unwrap();
}

/// Scrolling View all files hard - wheel past both ends, keys, clicks on
/// rows while the tree is still loading, and the diffs/file views those
/// clicks open - never ends the process.
#[test]
fn hammering_view_all_files_never_ends_the_process() {
    let repo = many_files(300);
    for (cols, rows) in [(160u16, 40u16), (130, 30)] {
        let (mut term, config) = launch(repo.root(), &format!("pty-vaf-hammer-{cols}"), cols, rows);
        let (c, r) = term.wait_for_text("View all files", STEP).unwrap();
        term.click(c + 6, r).unwrap();
        // Straight away: the tree is still loading.
        let (tc, tr) = term.screen().find("files (").unwrap_or((c, r + 3));
        for _ in 0..100 {
            term.scroll(true, tc + 2, tr + 2).unwrap();
        }
        term.wait_for("the whole tree listed", STEP, |s| s.contains("301 files"))
            .unwrap_or_else(|e| panic!("{e}"));
        for k in 0..12u16 {
            term.send_keys("Escape").unwrap();
            let (tc, tr) = term
                .wait_for_text("All files (", STEP)
                .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
            term.click(tc + 4, tr + 1 + (k % 8)).unwrap();
            for _ in 0..60 {
                term.scroll(k % 2 == 0, tc + 4, tr + 3).unwrap();
            }
            for _ in 0..60 {
                term.scroll(k % 2 == 1, cols / 2, rows / 2).unwrap();
            }
            term.send_keys("j j j End Home PageDown PageUp").unwrap();
            assert!(!term.exited(), "exited at step {k}:\n{}", term.text());
        }
        term.wait_for_stable(Duration::from_millis(300), STEP)
            .unwrap();
        assert!(!term.exited(), "exited:\n{}", term.text());
        assert_eq!(exit_log(config.path()), "", "nothing ended, nothing logged");
        term.stop().unwrap();
    }
}

// ---- The WIP's own files (uncommitted): resolved against the working
// copy and the index, never against HEAD's tree. ----

const NOT_IN_COMMIT: &str = "not in commit";

fn wip_fixture() -> gitpoulpi_testkit::FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .write_worktree("tracked.txt", b"alpha line\nold beta line\n")
        .unwrap()
        .stage(".")
        .unwrap()
        .commit("base")
        .unwrap()
        .write_worktree("tracked.txt", b"alpha line\nnew beta line\n")
        .unwrap()
        .write_worktree("TODO.md", b"todo item one\n")
        .unwrap()
        .build()
}

/// Clicks the WIP row's marker, then `name` in the Unstaged list, and
/// waits for that file's unstaged diff showing `shown`.
fn open_unstaged_diff(term: &mut Terminal, name: &str, shown: &str) {
    let (col, row) = term
        .wait_for_text("\u{25CC}", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(col, row).unwrap();
    let (col, row) = term
        .wait_for_text(name, STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.click(col + 1, row).unwrap();
    let title = format!("{name} - unstaged");
    let shown = shown.to_string();
    term.wait_for("the file's unstaged diff", STEP, move |s| {
        s.contains(&title) && s.contains(&shown)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

/// Clicks the bar button `label` and waits for `title` plus every `shown`
/// text, failing on any sign of the old HEAD-resolution error.
fn open_bar_view(term: &mut Terminal, label: &str, title: &str, shown: &[&str]) {
    let (c, r) = term
        .wait_for_text(label, STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.click(c + 2, r).unwrap();
    let title = title.to_string();
    let wanted: Vec<String> = shown.iter().map(|s| s.to_string()).collect();
    term.wait_for("the view of the WIP file", STEP, move |s| {
        s.contains(&title) && wanted.iter().all(|t| s.contains(t.as_str()))
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let text = term.text();
    assert!(!text.contains(NOT_IN_COMMIT), "{text}");
    assert!(!text.contains("backend error"), "{text}");
}

/// An untracked file opened from the Unstaged list: File View is its
/// content from disk titled WIP, Diff View is all added, Blame says "Not
/// committed yet", History is empty - and no tab shows the HEAD error.
#[test]
fn an_untracked_wip_file_opens_in_every_tab_without_a_backend_error() {
    let repo = wip_fixture();
    let (mut term, _config) = launch(repo.root(), "pty-wip-untracked", 200, 40);
    open_unstaged_diff(&mut term, "TODO.md", "todo item one");

    open_bar_view(
        &mut term,
        " File View ",
        "File View - TODO.md @ WIP",
        &["todo item one"],
    );
    open_bar_view(
        &mut term,
        " Blame ",
        "Blame - TODO.md @ WIP",
        &["Not committed yet", "todo item one"],
    );
    // The History is the `File` segment of the column's selector.
    open_bar_view(
        &mut term,
        " File \u{2502} All ",
        "History - TODO.md",
        &["no commits touch this file from here"],
    );
    assert!(!term.text().contains(NOT_IN_COMMIT));
    term.stop().unwrap();
}

/// A modified tracked file: File View shows what is on disk, Blame marks
/// the edited line and keeps HEAD's commit on the unchanged one, History
/// is the file's commits up to HEAD, and the Diff view is the change.
#[test]
fn a_modified_tracked_wip_file_shows_its_working_content_and_blame() {
    let repo = wip_fixture();
    let (mut term, _config) = launch(repo.root(), "pty-wip-modified", 200, 40);
    open_unstaged_diff(&mut term, "tracked.txt", "new beta line");
    assert!(term.text().contains("old beta line"), "{}", term.text());

    open_bar_view(
        &mut term,
        " File View ",
        "File View - tracked.txt @ WIP",
        &["alpha line", "new beta line"],
    );
    assert!(
        !term.text().contains("old beta line"),
        "File View is the disk's content, not HEAD's:\n{}",
        term.text()
    );
    open_bar_view(
        &mut term,
        " Blame ",
        "Blame - tracked.txt @ WIP",
        &["Not committed yet", "new beta line", "alpha line"],
    );
    open_bar_view(
        &mut term,
        " File \u{2502} All ",
        "History - tracked.txt",
        &["base"],
    );
    term.stop().unwrap();
}

/// A staged file reads the index: the title says so.
#[test]
fn a_staged_wip_file_opens_its_index_content() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .write_worktree("tracked.txt", b"alpha line\n")
        .unwrap()
        .stage(".")
        .unwrap()
        .commit("base")
        .unwrap()
        .write_worktree("staged.txt", b"staged line\n")
        .unwrap()
        .stage("staged.txt")
        .unwrap()
        .build();
    let (mut term, _config) = launch(repo.root(), "pty-wip-staged", 200, 40);
    let (col, row) = term
        .wait_for_text("\u{25CC}", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(col, row).unwrap();
    let (col, row) = term
        .wait_for_text("staged.txt", STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.click(col + 1, row).unwrap();
    term.wait_for("the staged diff", STEP, |s| {
        s.contains("staged.txt - staged") && s.contains("staged line")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    open_bar_view(
        &mut term,
        " File View ",
        "File View - staged.txt @ WIP (staged)",
        &["staged line"],
    );
    term.stop().unwrap();
}

/// "Treat as text" decides the File View too: a file with a NUL in it that a
/// `.gitattributes` rule marks as text shows its lines, while the same bytes
/// without the rule show the binary notice.
#[test]
fn the_file_view_honours_a_treat_as_text_rule() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .write_worktree(".gitattributes", b"forced.dat diff\n")
        .unwrap()
        .write_worktree("forced.dat", b"alpha line\nsecond\0part\nomega line\n")
        .unwrap()
        .write_worktree("plain.bin", b"alpha line\nsecond\0part\nomega line\n")
        .unwrap()
        .stage(".")
        .unwrap()
        .commit("attributes and data")
        .unwrap()
        .commit_file("other.txt", "other\n", "unrelated change")
        .unwrap()
        .build();
    let (mut term, _config) = launch(repo.root(), "pty-fileview-treat-text", 160, 40);
    let (c, r) = term.wait_for_text("View all files", STEP).unwrap();
    term.click(c + 6, r).unwrap();
    term.wait_for("the whole tree", STEP, |s| {
        s.contains("forced.dat") && s.contains("plain.bin")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    let (fc, fr) = term.screen().find("plain.bin").unwrap();
    term.click(fc, fr).unwrap();
    term.wait_for("the binary notice for the file without a rule", STEP, |s| {
        s.contains("File View - plain.bin @") && s.contains("nothing to show")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(!term.text().contains("omega line"));

    let (fc, fr) = term.screen().find("forced.dat").unwrap();
    term.click(fc, fr).unwrap();
    term.wait_for("the forced-text file's lines", STEP, |s| {
        s.contains("File View - forced.dat @")
            && s.contains("alpha line")
            && s.contains("omega line")
            && !s.contains("nothing to show")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}
