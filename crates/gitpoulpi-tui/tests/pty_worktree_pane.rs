//! The WORKTREES pane: the header `+` (create from the current branch, the
//! selected commit, or a chosen branch), the row's context menu (open in a
//! tab, Open in, merge, remove with the dirty-worktree choices), the empty
//! section shown on demand. Real binary, daemon and system git in a pty;
//! temp config dirs only.

mod common;

use std::path::{Path, PathBuf};
use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(30);

struct Fx {
    _repo: FixtureRepo,
    root: PathBuf,
    /// The linked worktree on `feature-x`, in a folder named `wt-folder`.
    wt: PathBuf,
    _dir: tempfile::TempDir,
    config: tempfile::TempDir,
}

fn repo() -> (FixtureRepo, PathBuf) {
    let fx = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "base commit")
        .unwrap()
        .branch("other")
        .unwrap()
        .checkout("main")
        .unwrap()
        .build();
    let root = fx.root().to_path_buf();
    (fx, root)
}

fn fixture(config_toml: &str) -> Fx {
    let (repo, root) = repo();
    let dir = tempfile::tempdir().unwrap();
    let wt = dir.path().join("wt-folder");
    common::git_ok(
        &root,
        &[
            "worktree",
            "add",
            "-q",
            "-b",
            "feature-x",
            &wt.to_string_lossy(),
        ],
    );
    common::git_ok(&wt, &["config", "user.name", "T"]);
    common::git_ok(&wt, &["config", "user.email", "t@example.com"]);
    std::fs::write(wt.join("f.txt"), "feature\n").unwrap();
    common::git_ok(&wt, &["add", "f.txt"]);
    common::git_ok(&wt, &["commit", "-q", "-m", "feature work"]);
    let config = tempfile::tempdir().unwrap();
    std::fs::write(config.path().join("config.toml"), config_toml).unwrap();
    Fx {
        _repo: repo,
        root,
        wt,
        _dir: dir,
        config,
    }
}

fn launch(root: &Path, session: &str, config: &Path) -> Terminal {
    common::launch(root)
        .session(session)
        .config_dir(config)
        .size(200, 60)
        .ready("base commit")
        .spawn()
}

fn worktree_list(root: &Path) -> String {
    common::git(root, &["worktree", "list", "--porcelain"])
}

fn wait_until(what: &str, mut ok: impl FnMut() -> bool) {
    let deadline = std::time::Instant::now() + xtask::pty::scaled(STEP);
    while !ok() {
        assert!(std::time::Instant::now() < deadline, "timed out: {what}");
        std::thread::sleep(Duration::from_millis(200));
    }
}

/// The cell of `needle` on a row below the WORKTREES header.
fn in_worktrees(screen: &Screen, needle: &str) -> Option<(u16, u16)> {
    let text = screen.text();
    let header = text.lines().position(|l| l.contains("WORKTREES"))?;
    text.lines()
        .enumerate()
        .skip(header + 1)
        .find_map(|(r, l)| {
            l.find(needle)
                .map(|b| (l[..b].chars().count() as u16, r as u16))
        })
}

fn wait_section(term: &mut Terminal) {
    term.wait_for("the WORKTREES section", STEP, |s| s.contains("WORKTREES"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

/// Clicks the header's `+`.
fn click_plus(term: &mut Terminal) {
    common::retry_until(
        term,
        "the + menu",
        6,
        Duration::from_secs(5),
        |t| {
            let s = t.screen();
            let text = s.text();
            if let Some((r, line)) = text
                .lines()
                .enumerate()
                .find(|(_, l)| l.contains("WORKTREES"))
            {
                let title = line.find("WORKTREES").unwrap();
                let after: String = line.chars().skip(line[..title].chars().count()).collect();
                if let Some(off) = after.find('+') {
                    let col = line[..title].chars().count() + after[..off].chars().count();
                    t.click(col as u16, r as u16).unwrap();
                }
            }
        },
        |s| s.contains("From the current branch"),
    );
}

fn palette(term: &mut Terminal, query: &str, label: &str) {
    term.send_keys("C-p").unwrap();
    term.send_text(query).unwrap();
    term.wait_for("the palette entry", STEP, |s| s.contains(label))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
}

fn click_text(term: &mut Terminal, needle: &str, what: &str, pred: impl FnMut(&Screen) -> bool) {
    common::click_text_until(term, needle, what, pred);
}

/// Clicks `needle` on a row below the title bar (the title bar has its own
/// "Open in").
fn click_below_title(
    term: &mut Terminal,
    needle: &str,
    what: &str,
    pred: impl FnMut(&Screen) -> bool,
) {
    let mut pred = pred;
    common::retry_until(
        term,
        what,
        6,
        Duration::from_secs(5),
        |t| {
            let s = t.screen();
            if pred(&s) {
                return;
            }
            let text = s.text();
            if let Some((r, c)) = text.lines().enumerate().skip(2).find_map(|(r, l)| {
                l.match_indices(needle)
                    .find(|(b, _)| !l[*b..].starts_with("Open in a new tab"))
                    .map(|(b, _)| (r, l[..b].chars().count()))
            }) {
                t.click(c as u16 + 1, r as u16).unwrap();
            }
        },
        |s| s.contains("Mark A"),
    );
}

fn row_menu(term: &mut Terminal, name: &str) {
    common::retry_until(
        term,
        "the row menu",
        8,
        Duration::from_secs(4),
        |t| {
            if let Some((c, r)) = in_worktrees(&t.screen(), name) {
                t.right_click(c + 1, r).unwrap();
            }
        },
        |s| s.contains("Open in a new tab"),
    );
}

/// The generated `worktree-<8 hex>` names on `text`, in reading order.
fn generated_names(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    for (at, _) in text.match_indices("worktree-") {
        let id: String = text[at + 9..].chars().take(8).collect();
        let after = text[at + 9..].chars().nth(8);
        if id.len() == 8
            && id
                .chars()
                .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c))
            && !after.is_some_and(|c| c.is_ascii_alphanumeric())
        {
            out.push(format!("worktree-{id}"));
        }
    }
    out
}

#[test]
fn plus_menu_creates_from_the_current_branch_with_default_name_and_folder() {
    let fx = fixture("");
    let mut term = launch(&fx.root, "pty-wt-plus", fx.config.path());
    wait_section(&mut term);
    click_plus(&mut term);
    click_text(&mut term, "From the current branch", "the dialog", |s| {
        s.contains("Create worktree from main")
    });
    // The checked-out branch needs a new branch: the name and the new branch
    // are the same generated `worktree-<id>`, and the folder follows the name.
    let repo_name = fx.root.file_name().unwrap().to_string_lossy().into_owned();
    let names = generated_names(&term.text());
    assert!(names.len() >= 2, "{}", term.text());
    let name = names[0].clone();
    assert!(names.iter().all(|n| *n == name), "{names:?}");
    term.send_keys("Enter").unwrap();
    let created = fx
        .root
        .parent()
        .unwrap()
        .join(format!("{repo_name}-{name}"));
    wait_until("the worktree is created", || {
        worktree_list(&fx.root).contains(&format!("{repo_name}-{name}"))
    });
    assert!(worktree_list(&fx.root).contains(&format!("branch refs/heads/{name}")));
    let _ = std::fs::remove_dir_all(&created);
}

#[test]
fn the_palette_creates_from_the_current_branch_and_chooses_a_branch() {
    let fx = fixture("");
    let mut term = launch(&fx.root, "pty-wt-palette", fx.config.path());
    wait_section(&mut term);
    palette(
        &mut term,
        "Create Worktree Choosing",
        "Refs: Create Worktree Choosing a Branch",
    );
    // The Worktree branch list is open on the branches: pick `other`.
    term.wait_for("the branch list", STEP, |s| {
        s.contains("Create worktree from main") && s.contains("other")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let name = generated_names(&term.text())[0].clone();
    term.send_text("other").unwrap();
    term.send_keys("Enter").unwrap();
    let repo_name = fx.root.file_name().unwrap().to_string_lossy().into_owned();
    term.wait_for("the existing branch is the worktree's", STEP, |s| {
        s.contains("checks out this branch") && s.contains(" on other")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Tab to Create (the keyboard twin of the button): `other` is checked
    // out in the new worktree, whose folder keeps the generated name.
    for _ in 0..3 {
        term.send_keys("Tab").unwrap();
    }
    term.send_keys("Enter").unwrap();
    wait_until("the worktree on other", || {
        worktree_list(&fx.root).contains("branch refs/heads/other")
    });
    let _ = std::fs::remove_dir_all(
        fx.root
            .parent()
            .unwrap()
            .join(format!("{repo_name}-{name}")),
    );
}

#[test]
fn the_selected_commit_starts_a_worktree_on_a_generated_branch() {
    let fx = fixture("");
    let mut term = launch(&fx.root, "pty-wt-commit", fx.config.path());
    wait_section(&mut term);
    // Select the base commit's row in the Log.
    common::click_text_until(&mut term, "base commit", "the commit selected", |s| {
        s.contains("base commit")
    });
    click_plus(&mut term);
    click_text(&mut term, "From the selected commit", "the dialog", |s| {
        s.contains("Create worktree from")
    });
    term.send_keys("Enter").unwrap();
    // The dialog proposes a new branch `worktree-<id>` on the commit.
    wait_until("a worktree on a generated branch", || {
        worktree_list(&fx.root).contains("branch refs/heads/worktree-")
    });
}

#[test]
fn the_context_menu_opens_the_worktree_as_a_tab() {
    let fx = fixture("");
    let mut term = launch(&fx.root, "pty-wt-tab", fx.config.path());
    wait_section(&mut term);
    row_menu(&mut term, "feature-x");
    click_text(&mut term, "Open in a new tab", "the worktree's tab", |s| {
        s.contains("wt-folder")
    });
}

#[test]
fn the_open_in_submenu_lists_only_the_entries_flagged_for_worktrees() {
    let marker = tempfile::tempdir().unwrap();
    let file = marker.path().join("marker.txt");
    let config = format!(
        "[[ui.open_in]]\nlabel = \"Mark A\"\ncommand = \"sh -c 'echo ${{cwd}} > {}'\"\n\n[[ui.open_in]]\nlabel = \"Skip B\"\ncommand = \"true ${{cwd}}\"\nworktree = false\n",
        file.display()
    );
    let fx = fixture(&config);
    let mut term = launch(&fx.root, "pty-wt-open-in", fx.config.path());
    wait_section(&mut term);
    row_menu(&mut term, "feature-x");
    click_below_title(&mut term, "Open in", "the submenu", |s| {
        s.contains("Mark A")
    });
    let screen = term.screen();
    assert!(
        screen.find("Skip B").is_none(),
        "an entry that cannot open a worktree is listed:\n{}",
        term.text()
    );
    click_text(&mut term, "Mark A", "the entry ran", |_| file.exists());
    wait_until("the marker has the folder", || {
        std::fs::read_to_string(&file)
            .map(|t| t.contains("wt-folder"))
            .unwrap_or(false)
    });
}

#[test]
fn removing_a_clean_worktree_asks_and_removes_it() {
    let fx = fixture("");
    // Clean it: the feature commit is committed already.
    let mut term = launch(&fx.root, "pty-wt-remove", fx.config.path());
    wait_section(&mut term);
    row_menu(&mut term, "feature-x");
    click_text(&mut term, "Remove worktree...", "the confirmation", |s| {
        s.contains("Remove worktree wt-folder") && s.contains("is not merged")
    });
    // Cancel keeps it.
    click_text(&mut term, "Cancel", "closed", |s| {
        !s.contains("Remove worktree wt-folder")
    });
    assert!(fx.wt.exists(), "Cancel removed it:\n{}", term.text());
    row_menu(&mut term, "feature-x");
    click_text(&mut term, "Remove worktree...", "the confirmation", |s| {
        s.contains("Remove worktree wt-folder") && s.contains("is not merged")
    });
    // The branch is not merged into main: no checkbox.
    assert!(term.text().contains("is not merged"), "{}", term.text());
    assert!(!term.text().contains("Also delete branch"));
    click_text(&mut term, " Remove  ", "removed", |_| !fx.wt.exists());
    assert!(!worktree_list(&fx.root).contains("wt-folder"));
    assert!(
        common::git(&fx.root, &["branch", "--list", "feature-x"]).contains("feature-x"),
        "the branch stays"
    );
}

#[test]
fn a_dirty_worktree_offers_commit_stash_open_and_a_forced_removal() {
    let fx = fixture("");
    std::fs::write(fx.wt.join("scratch.txt"), "scratch\n").unwrap();
    let mut term = launch(&fx.root, "pty-wt-dirty", fx.config.path());
    wait_section(&mut term);
    row_menu(&mut term, "feature-x");
    click_text(&mut term, "Remove worktree...", "the choices", |s| {
        s.contains("1 uncommitted change")
            && s.contains("Commit and remove")
            && s.contains("Stash and remove")
            && s.contains("Open to commit")
            && s.contains("Remove anyway")
            && s.contains("scratch.txt")
    });
    // Remove anyway needs a second, explicit confirmation.
    click_text(&mut term, "Remove anyway", "the second confirmation", |s| {
        s.contains("and discard its")
    });
    assert!(fx.wt.exists(), "nothing is removed before the confirmation");
    click_text(&mut term, "Back", "back", |s| {
        s.contains("Commit and remove")
    });
    // Stash and remove: the stash holds the untracked file.
    click_text(&mut term, "Stash and remove", "removed", |_| {
        !fx.wt.exists()
    });
    wait_until("the stash exists", || {
        common::git(&fx.root, &["stash", "list"]).contains("WIP on wt-folder")
    });
    assert_eq!(
        common::git_trimmed(&fx.root, &["ls-tree", "-r", "--name-only", "stash@{0}^3"]),
        "scratch.txt"
    );
}

#[test]
fn commit_and_remove_leaves_the_commit_on_the_branch() {
    let fx = fixture("");
    std::fs::write(fx.wt.join("scratch.txt"), "scratch\n").unwrap();
    let mut term = launch(&fx.root, "pty-wt-commit-remove", fx.config.path());
    wait_section(&mut term);
    row_menu(&mut term, "feature-x");
    click_text(&mut term, "Remove worktree...", "the choices", |s| {
        s.contains("Commit and remove")
    });
    // The message field has the keyboard.
    term.send_text("park the scratch").unwrap();
    term.wait_for("the message typed", STEP, |s| {
        s.contains("park the scratch")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(
        !common::status_line_text(&term.screen()).contains("park the scratch"),
        "the status line never types"
    );
    click_text(&mut term, "Commit and remove", "removed", |_| {
        !fx.wt.exists()
    });
    wait_until("the commit is on the branch", || {
        common::git_trimmed(&fx.root, &["log", "-1", "--format=%s", "feature-x"])
            == "park the scratch"
    });
}

#[test]
fn open_to_commit_opens_the_tab_with_the_message_field() {
    let fx = fixture("");
    std::fs::write(fx.wt.join("scratch.txt"), "scratch\n").unwrap();
    let mut term = launch(&fx.root, "pty-wt-open-commit", fx.config.path());
    wait_section(&mut term);
    row_menu(&mut term, "feature-x");
    click_text(&mut term, "Remove worktree...", "the choices", |s| {
        s.contains("Open to commit")
    });
    click_text(&mut term, "Open to commit", "the worktree tab", |s| {
        s.contains("wt-folder")
    });
    assert!(fx.wt.exists(), "it is not removed");
}

#[test]
fn merging_a_worktree_branch_into_main_lands_on_main() {
    let fx = fixture("");
    let mut term = launch(&fx.root, "pty-wt-merge", fx.config.path());
    wait_section(&mut term);
    row_menu(&mut term, "feature-x");
    click_text(&mut term, "Merge feature-x into...", "the picker", |s| {
        s.contains("Merge feature-x into:") && s.contains("the merge runs there")
    });
    click_text(&mut term, " Merge  ", "merged", |_| {
        common::git_try(
            &fx.root,
            &["merge-base", "--is-ancestor", "feature-x", "main"],
        )
        .is_some()
    });
    assert!(fx.wt.exists(), "merging does not remove the worktree");
}

#[test]
fn removing_a_merged_worktree_can_delete_its_branch_and_undo_brings_it_back() {
    let (repo, root) = repo();
    let dir = tempfile::tempdir().unwrap();
    let wt = dir.path().join("wt-done");
    common::git_ok(
        &root,
        &["worktree", "add", "-q", "-b", "done", &wt.to_string_lossy()],
    );
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(&root, "pty-wt-delete-branch", config.path());
    wait_section(&mut term);
    row_menu(&mut term, "done");
    click_text(&mut term, "Remove worktree...", "the box", |s| {
        s.contains("Also delete branch done")
    });
    // Checked by default: just Remove.
    assert!(
        term.text().contains("[x] Also delete branch done"),
        "{}",
        term.text()
    );
    click_text(&mut term, " Remove  ", "removed", |_| !wt.exists());
    wait_until("the branch is deleted", || {
        common::git(&root, &["branch", "--list", "done"])
            .trim()
            .is_empty()
    });
    // Undo brings the branch back (the folder stays gone). The journal entry
    // can land a moment after the disk changed: press again until it is back.
    for _ in 0..12 {
        if !common::git(&root, &["branch", "--list", "done"])
            .trim()
            .is_empty()
        {
            break;
        }
        term.send_keys("C-z").unwrap();
        std::thread::sleep(Duration::from_millis(900));
        if term.text().contains("Undo") && term.text().contains("Cancel") {
            term.send_keys("Enter").unwrap();
            std::thread::sleep(Duration::from_millis(600));
        }
    }
    wait_until("the branch is back", || {
        !common::git(&root, &["branch", "--list", "done"])
            .trim()
            .is_empty()
    });
    assert!(!wt.exists());
    drop(repo);
}

#[test]
fn the_section_is_hidden_without_worktrees_and_shown_empty_on_demand() {
    let (_repo, root) = repo();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(&root, "pty-wt-empty", config.path());
    term.wait_for("the sidebar", STEP, |s| s.contains("LOCAL"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(
        !term.text().contains("WORKTREES"),
        "hidden when the repository has no worktree:\n{}",
        term.text()
    );
    palette(
        &mut term,
        "Show Worktrees",
        "Sidebar: Show Worktrees Section",
    );
    term.wait_for("the empty section", STEP, |s| {
        s.contains("WORKTREES") && s.contains("No worktree yet: + to create one")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    drop(term);
    // Shown for this repository from now on: a new session sees it.
    let again = launch(&root, "pty-wt-empty-2", config.path());
    again
        .wait_for("still shown", STEP, |s| s.contains("No worktree yet"))
        .unwrap_or_else(|e| panic!("{e}\n{}", again.text()));
    drop(again);
    // Another repository without worktrees starts hidden.
    let (_other, other_root) = repo();
    let fresh = launch(&other_root, "pty-wt-empty-3", config.path());
    std::thread::sleep(Duration::from_millis(500));
    assert!(!fresh.text().contains("WORKTREES"), "{}", fresh.text());
}
