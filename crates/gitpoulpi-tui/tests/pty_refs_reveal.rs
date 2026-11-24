//! The Refs sidebar's menus and Reveal in graph, end to end through a pty
//! (`xtask::pty`): the real binary, the real daemon, real `git`.
//!
//! The rules are `pty_ui.rs`'s: nothing is located by a hard-coded
//! coordinate, every wait describes the whole state it waits for, and the
//! config dir is a temp dir.

mod common;

use common::git_trimmed as git;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(20);
const COLS: u16 = 200;
const ROWS: u16 = 50;
/// The Refs pane is 20% of 200 columns.
const SIDEBAR_RIGHT: u16 = 40;
/// `HIDDEN_IN_GRAPH_MARK` in `gitpoulpi_tui`.
const HIDDEN: &str = "\u{2298}";

/// Two commits on `main`, a remote `gh` holding `main` and a `side` branch.
fn fixture() -> gitpoulpi_testkit::FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial")
        .unwrap()
        .commit_file("b.txt", "two\n", "second")
        .unwrap()
        .build();
    let root = repo.root().to_path_buf();
    for args in [
        &["remote", "add", "gh", "../nowhere-gh"][..],
        &["update-ref", "refs/remotes/gh/main", "HEAD"],
        &["update-ref", "refs/remotes/gh/side", "HEAD~1"],
    ] {
        git(&root, args);
    }
    repo
}

fn launch(repo: &Path, session: &str, config: &Path) -> Terminal {
    common::launch(repo)
        .session(session)
        .config_dir(config)
        .size(COLS, ROWS)
        .ready_when("the fixture's log and its REMOTE tree", |s| {
            s.contains("second") && s.contains("initial") && s.contains("gh")
        })
        .spawn()
}

/// The first sidebar occurrence of `needle` strictly below row `from`.
fn sidebar_below(screen: &Screen, needle: &str, from: u16) -> Option<(u16, u16)> {
    screen
        .find_all(needle)
        .into_iter()
        .filter(|(col, row)| *col < SIDEBAR_RIGHT && *row > from)
        .min_by_key(|(_, row)| *row)
}

fn header_row(screen: &Screen, header: &str) -> Option<u16> {
    sidebar_below(screen, header, 0).map(|(_, row)| row)
}

/// The sidebar row of `label` under the section `header`.
fn label_under(screen: &Screen, header: &str, label: &str) -> Option<(u16, u16)> {
    sidebar_below(screen, label, header_row(screen, header)?)
}

fn wait_label(term: &Terminal, header: &str, label: &str) -> (u16, u16) {
    let screen = term
        .wait_for(&format!("{label:?} under {header:?}"), STEP, |s| {
            label_under(s, header, label).is_some()
        })
        .unwrap_or_else(|e| panic!("{e}"));
    label_under(&screen, header, label).unwrap()
}

const REMOTE_MENU: &str = "Copy link to remote";
const REF_MENU: &str = "Hide in graph";

fn right_click_remote(term: &mut Terminal) {
    let (col, row) = wait_label(term, "REMOTE (", "gh");
    term.right_click(col + 1, row).unwrap();
}

fn right_click_local_main(term: &mut Terminal) {
    let (col, row) = wait_label(term, "LOCAL (", "main");
    term.right_click(col + 1, row).unwrap();
}

/// A remote's menu, then a branch's: only the branch's is painted.
#[test]
fn right_clicking_a_branch_closes_the_remote_menu_that_was_open() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), "reveal-remote-then-branch", config.path());

    right_click_remote(&mut term);
    term.wait_for("the remote's menu", STEP, |s| s.contains(REMOTE_MENU))
        .unwrap_or_else(|e| panic!("{e}"));
    right_click_local_main(&mut term);
    let screen = term
        .wait_for("only the branch's menu", STEP, |s| {
            s.contains(REF_MENU) && !s.contains(REMOTE_MENU)
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(!screen.contains("Copy link to remote"), "{}", term.text());

    // The surviving menu is hit-testable: Hide in graph hides `main`.
    let (c, r) = screen.find(REF_MENU).unwrap();
    term.click(c + 1, r).unwrap();
    term.wait_for("main hidden and no menu", STEP, |s| {
        !s.contains(REF_MENU)
            && label_under(s, "LOCAL (", "main")
                .is_some_and(|(_, row)| s.line(row).contains(HIDDEN))
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

/// A branch's menu, then a remote's: only the remote's is painted.
#[test]
fn right_clicking_a_remote_closes_the_branch_menu_that_was_open() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), "reveal-branch-then-remote", config.path());

    // The remote's row is below LOCAL, so a menu opened on `main` hangs
    // over it; open the branch menu on the REMOTE's own branch instead,
    // which sits below the remote row.
    let (col, row) = wait_label(&term, "REMOTE (", "side");
    term.right_click(col + 1, row).unwrap();
    term.wait_for("the branch's menu", STEP, |s| s.contains(REF_MENU))
        .unwrap_or_else(|e| panic!("{e}"));
    right_click_remote(&mut term);
    let screen = term
        .wait_for("only the remote's menu", STEP, |s| {
            s.contains(REMOTE_MENU) && !s.contains(REF_MENU)
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Hit-testable: Copy link to remote runs and closes the menu.
    let (c, r) = screen.find(REMOTE_MENU).unwrap();
    term.click(c + 1, r).unwrap();
    term.wait_for("the remote menu gone", STEP, |s| {
        !s.contains(REMOTE_MENU) && !s.contains(REF_MENU)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

/// Right-clicking a row of the open menu is not a right-click on the pane
/// behind it: the menu stays.
#[test]
fn right_clicking_a_row_of_the_open_menu_leaves_it_alone() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), "reveal-menu-owns-pointer", config.path());

    right_click_remote(&mut term);
    let screen = term
        .wait_for("the remote's menu", STEP, |s| s.contains(REMOTE_MENU))
        .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = screen.find(REMOTE_MENU).unwrap();
    term.right_click(c + 1, r).unwrap();
    // A later frame proving nothing changed: hover another row and wait.
    term.wait_for("still only the remote's menu", STEP, |s| {
        s.contains(REMOTE_MENU) && !s.contains(REF_MENU)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

// ---------------------------------------------------------------------------
// Reveal in graph
// ---------------------------------------------------------------------------

use std::io::Write;
use std::process::{Command, Stdio};

const COMMITS: usize = 1200;
const TAG: &str = "old-tag";

/// `COMMITS` commits (`commit-0001` oldest) built with one `git fast-import`,
/// a tag on the oldest and a branch `side` on the 600th. The Log loads 500
/// at first, so both are far outside the first page.
fn deep_repo() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("temp repo");
    common::git_ok(dir.path(), &["init", "-q", "-b", "main"]);
    let mut stream = String::new();
    for n in 1..=COMMITS {
        let msg = format!("commit-{n:04}");
        stream.push_str(&format!(
            "commit refs/heads/main\nmark :{n}\ncommitter T <t@example.com> {} +0000\ndata {}\n{msg}\n",
            1_600_000_000 + n * 60,
            msg.len()
        ));
        if n > 1 {
            stream.push_str(&format!("from :{}\n", n - 1));
        }
        stream.push_str(&format!(
            "M 100644 inline f.txt\ndata {}\n{n}\n\n",
            n.to_string().len()
        ));
    }
    let mut child = Command::new("git")
        .args(["fast-import", "--quiet"])
        .current_dir(dir.path())
        .stdin(Stdio::piped())
        .spawn()
        .expect("spawn fast-import");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(stream.as_bytes())
        .unwrap();
    assert!(child.wait().unwrap().success());
    common::git_ok(dir.path(), &["reset", "-q", "--hard", "main"]);
    let root = common::git_trimmed(dir.path(), &["rev-list", "--max-parents=0", "main"]);
    common::git_ok(dir.path(), &["tag", TAG, &root]);
    common::git_ok(dir.path(), &["branch", "side", "main~600"]);
    dir
}

fn launch_deep(repo: &Path, session: &str, config: &Path, ui_toml: &str) -> Terminal {
    std::fs::write(
        config.join("config.toml"),
        format!(
            "[general]\ninitial_commits_in_graph = 500\nlazy_load_commits_in_graph = true\n\
             commits_per_lazy_page = 50\n{ui_toml}"
        ),
    )
    .unwrap();
    let term = common::launch(repo)
        .session(session)
        .config_dir(config)
        .size(COLS, ROWS)
        .spawn();
    term.wait_for(
        "HEAD in the Log and the TAGS section",
        Duration::from_secs(60),
        |s| s.contains("commit-1200") && label_under(s, "TAGS (", TAG).is_some(),
    )
    .unwrap_or_else(|e| panic!("{e}"));
    term
}

/// Where the Log's `commit-NNNN` message is drawn (not the sidebar).
fn log_row(screen: &Screen, needle: &str) -> Option<(u16, u16)> {
    screen
        .find_all(needle)
        .into_iter()
        .find(|(col, _)| *col > SIDEBAR_RIGHT && *col < 120)
}

fn click_tag(term: &mut Terminal) {
    let (col, row) = wait_label(term, "TAGS (", TAG);
    term.click(col + 1, row).unwrap();
}

/// A single click on a tag whose commit is 1,200 rows down: the Log loads
/// the history, scrolls to `commit-0001`, flashes the row, and the Log's
/// selection (HEAD, still shown in the commit details) does not move.
#[test]
fn a_single_click_on_a_tag_loads_and_scrolls_the_log_to_its_commit() {
    let repo = deep_repo();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch_deep(repo.path(), "reveal-click-tag", config.path(), "");
    wait_label(&term, "TAGS (", TAG);
    // The click can be dropped while the app is still settling (the label is
    // drawn before its target is live); repeat it until the reveal shows. A
    // repeated reveal is harmless: it scrolls to the same row.
    let screen = common::retry_until(
        &mut term,
        "commit-0001 in view and flashing",
        6,
        Duration::from_secs(8),
        |t| {
            if let Some((col, row)) = label_under(&t.screen(), "TAGS (", TAG) {
                t.click(col + 1, row).unwrap();
            }
        },
        |s| {
            log_row(s, "commit-0001").is_some_and(|(col, row)| {
                let flashing = s.cell(col, row).map(|c| c.bg);
                let plain = s.cell(col, row - 1).map(|c| c.bg);
                flashing.is_some() && flashing != plain
            })
        },
    );
    // The selection stayed on HEAD: its message is in the commit details.
    assert!(screen.contains("commit-1200"), "{}", term.text());
    // The flash ends by itself.
    assert!(log_row(&screen, "commit-0001").is_some());
    // Wherever the row sits by then (late history rows can shift the view on
    // a loaded machine, even off screen): it is not drawn flashing any more.
    term.wait_for("the flash gone", STEP * 3, |s| {
        log_row(s, "commit-0001")
            .is_none_or(|(c, r)| s.cell(c, r).map(|x| x.bg) == s.cell(c, r - 1).map(|x| x.bg))
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

/// With `ui.reveal_on_ref_click = false` a click only selects the row; the
/// ref's menu still has Reveal in graph, and it reveals.
#[test]
fn reveal_in_graph_in_the_menu_works_with_the_click_setting_off() {
    let repo = deep_repo();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch_deep(
        repo.path(),
        "reveal-menu",
        config.path(),
        "[ui]\nreveal_on_ref_click = false\n",
    );
    click_tag(&mut term);
    // Let the click and any tick land: the row is selected (focus is Refs).
    term.wait_for("the Refs pane focused", STEP, |s| s.contains("Refs"))
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        log_row(&term.screen(), "commit-0001").is_none(),
        "the click revealed with the setting off: {}",
        term.text()
    );
    wait_label(&term, "TAGS (", TAG);
    // Three steps, each retried on its own: a click sent as the menu or its
    // entry is drawn can be dropped (the hit target is registered a frame
    // later). The reveal itself pages a deep history, which can take seconds
    // under load: it is awaited without touching the menu again.
    common::retry_until(
        &mut term,
        "the tag's menu",
        8,
        Duration::from_secs(3),
        |t| {
            if !t.screen().contains("Reveal in graph") {
                if let Some((col, row)) = label_under(&t.screen(), "TAGS (", TAG) {
                    t.right_click(col + 1, row).unwrap();
                }
            }
        },
        |s| s.contains("Reveal in graph"),
    );
    common::retry_until(
        &mut term,
        "the menu closed by Reveal in graph",
        8,
        Duration::from_secs(3),
        |t| {
            if let Some((c, r)) = t.screen().find("Reveal in graph") {
                t.click(c + 1, r).unwrap();
            }
        },
        |s| !s.contains("Reveal in graph"),
    );
    term.wait_for("commit-0001 in the Log", STEP, |s| {
        log_row(s, "commit-0001").is_some()
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

/// The Refs pane's key (`g`) is the keyboard twin: select the row, press it.
#[test]
fn the_g_key_in_the_refs_pane_reveals_the_selected_branch() {
    let repo = deep_repo();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch_deep(
        repo.path(),
        "reveal-key",
        config.path(),
        "[ui]\nreveal_on_ref_click = false\n",
    );
    let (col, row) = wait_label(&term, "LOCAL (", "side");
    term.click(col + 1, row).unwrap();
    term.send_keys("g").unwrap();
    // `side` is `main~600`: commit-0600.
    term.wait_for("commit-0600 in the Log", STEP, |s| {
        log_row(s, "commit-0600").is_some()
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

/// A ref hidden from the graph is not revealed: the footer says why.
#[test]
fn a_hidden_ref_says_so_in_the_footer_instead_of_revealing() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), "reveal-hidden", config.path());
    let (col, row) = wait_label(&term, "REMOTE (", "side");
    // The eye gutter hides it...
    term.click(col - 2, row).unwrap();
    term.wait_for("side hidden", STEP, |s| {
        s.cell(col - 2, row).is_some_and(|c| c.text == HIDDEN)
    })
    .unwrap_or_else(|e| panic!("{e}"));
    // ...and a click on its name says why nothing scrolled.
    term.click(col + 1, row).unwrap();
    term.wait_for("the hidden note", STEP, |s| {
        s.contains("gh/side is hidden from the graph")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

/// A local branch and the remote-tracking branch of the same name on
/// different commits (an unpushed `main`): each reveals its own commit, not
/// the other's.
#[test]
fn a_local_and_a_remote_branch_of_the_same_name_reveal_their_own_commits() {
    let repo = deep_repo();
    common::git_ok(repo.path(), &["remote", "add", "gh", "../nowhere-gh"]);
    common::git_ok(
        repo.path(),
        &["update-ref", "refs/remotes/gh/main", "main~600"],
    );
    let config = tempfile::tempdir().unwrap();
    let mut term = launch_deep(repo.path(), "reveal-same-name", config.path(), "");
    // The remote's `main` (commit-0600) first: far below the first screen.
    let (col, row) = wait_label(&term, "REMOTE (", "main");
    term.click(col + 1, row).unwrap();
    term.wait_for("commit-0600 in the Log", STEP, |s| {
        log_row(s, "commit-0600").is_some()
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Then the local `main` (commit-1200): back at the top.
    let (col, row) = wait_label(&term, "LOCAL (", "main");
    term.click(col + 1, row).unwrap();
    term.wait_for("commit-1200 back in the Log", STEP, |s| {
        log_row(s, "commit-1200").is_some() && log_row(s, "commit-0600").is_none()
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // And the remote's again.
    let (col, row) = wait_label(&term, "REMOTE (", "main");
    term.click(col + 1, row).unwrap();
    term.wait_for("commit-0600 in the Log again", STEP, |s| {
        log_row(s, "commit-0600").is_some()
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

/// The background of the Log cell where `needle` starts.
fn row_bg(s: &Screen, needle: &str) -> Option<xtask::pty::Color> {
    log_row(s, needle).and_then(|(col, row)| s.cell(col, row).map(|c| c.bg))
}

/// Clicks the sidebar `label` under `header` and waits until `own` flashes
/// (its background leaves its resting color) while `other` keeps its own.
fn click_and_see_flash(term: &mut Terminal, header: &str, label: &str, own: &str, other: &str) {
    let rest = term.screen();
    let (own_rest, other_rest) = (row_bg(&rest, own), row_bg(&rest, other));
    let (col, row) = wait_label(term, header, label);
    term.click(col + 1, row).unwrap();
    term.wait_for(&format!("{own} flashing, {other} not"), STEP, |s| {
        row_bg(s, own).is_some_and(|bg| Some(bg) != own_rest) && row_bg(s, other) == other_rest
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Let the flash end so the next click starts from rest.
    term.wait_for(&format!("{own} back at rest"), STEP, |s| {
        row_bg(s, own) == own_rest
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

/// Both commits already on screen (`main` five commits ahead of `gh/main`):
/// nothing scrolls, so the flash is what tells them apart - each click
/// flashes its own ref's row.
#[test]
fn same_name_refs_on_one_screen_flash_their_own_rows() {
    let repo = deep_repo();
    common::git_ok(repo.path(), &["remote", "add", "gh", "../nowhere-gh"]);
    common::git_ok(
        repo.path(),
        &["update-ref", "refs/remotes/gh/main", "main~5"],
    );
    let config = tempfile::tempdir().unwrap();
    let mut term = launch_deep(repo.path(), "reveal-same-screen", config.path(), "");
    click_and_see_flash(&mut term, "REMOTE (", "main", "commit-1195", "commit-1200");
    click_and_see_flash(&mut term, "LOCAL (", "main", "commit-1200", "commit-1195");
}

/// The other way round (a pull not done yet): `gh/main` five commits ahead
/// of the local `main`. Each click flashes its own ref's row.
#[test]
fn a_remote_ahead_of_its_local_namesake_flashes_its_own_row() {
    let repo = deep_repo();
    common::git_ok(repo.path(), &["remote", "add", "gh", "../nowhere-gh"]);
    common::git_ok(repo.path(), &["update-ref", "refs/remotes/gh/main", "main"]);
    common::git_ok(repo.path(), &["reset", "-q", "--hard", "main~5"]);
    let config = tempfile::tempdir().unwrap();
    let mut term = common::launch(repo.path())
        .session("reveal-remote-ahead")
        .config_dir(config.path())
        .size(COLS, ROWS)
        .spawn();
    term.wait_for("both mains in the Log", Duration::from_secs(60), |s| {
        log_row(s, "commit-1200").is_some() && log_row(s, "commit-1195").is_some()
    })
    .unwrap_or_else(|e| panic!("{e}"));
    click_and_see_flash(&mut term, "LOCAL (", "main", "commit-1195", "commit-1200");
    click_and_see_flash(&mut term, "REMOTE (", "main", "commit-1200", "commit-1195");
}

/// Where the Log's pill `label` is drawn on the row of `commit` (not the
/// sidebar): the cell a drag is dropped on.
fn log_pill(screen: &Screen, commit: &str, label: &str) -> Option<(u16, u16)> {
    let (_, row) = log_row(screen, commit)?;
    screen
        .find_all(label)
        .into_iter()
        .find(|(col, r)| *r == row && *col > SIDEBAR_RIGHT)
}

/// Pressing a sidebar ref to carry it must not make the Log follow it: the
/// reveal belongs to the release of a click, so the graph stays where the
/// user scrolled it and the drop lands on the pill they can see. `side`'s
/// commit (`commit-0600`) is far outside the first screen, `main`'s pill is
/// on the top row.
#[test]
fn dragging_a_ref_from_the_sidebar_leaves_the_log_where_it_is() {
    let repo = deep_repo();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch_deep(repo.path(), "reveal-drag-no-jump", config.path(), "");
    let target = log_pill(&term.screen(), "commit-1200", "main")
        .unwrap_or_else(|| panic!("no main pill on the top row\n{}", term.text()));
    let (col, row) = wait_label(&term, "LOCAL (", "side");

    term.press(col + 1, row).unwrap();
    term.drag_to(col + 3, row).unwrap();
    term.drag_to(target.0, target.1).unwrap();
    term.wait_for("the drag banner naming the target", STEP, |s| {
        s.contains("main - release to drop")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // A reveal pages a deep history in; give a wrongly started one the time.
    std::thread::sleep(Duration::from_millis(2500));
    let mid = term.screen();
    assert!(
        log_row(&mid, "commit-1200").is_some() && log_row(&mid, "commit-0600").is_none(),
        "the Log scrolled while the ref was being carried:\n{}",
        term.text()
    );

    term.release(target.0, target.1).unwrap();
    let screen = term
        .wait_for("the drop menu for side on main", STEP, |s| {
            s.contains("Merge side into main")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    std::thread::sleep(Duration::from_millis(1500));
    let after = term.screen();
    assert!(
        log_row(&after, "commit-1200").is_some() && log_row(&after, "commit-0600").is_none(),
        "the Log scrolled on the drop:\n{}",
        term.text()
    );
    assert!(screen.contains("Rebase side onto main"), "{}", term.text());
}
