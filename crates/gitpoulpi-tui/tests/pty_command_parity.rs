//! The same command reached from the toolbar, the `Ctrl+G` leader and the
//! command palette must do the same thing: New Branch, New Tag and Stash
//! Changes (each by all three surfaces, checked through `git`), and Stash
//! Pop, whose three surfaces differ today; those differences are pinned as
//! they are, so changing them is a visible, deliberate flip.
//!
//! Same rules as `pty_ui.rs`: nothing located by a hard-coded coordinate,
//! no sleeps, temp config dir and fixture repository.

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);

#[derive(Clone, Copy, Debug)]
enum Surface {
    Toolbar,
    Leader,
    Palette,
}

const SURFACES: [Surface; 3] = [Surface::Toolbar, Surface::Leader, Surface::Palette];

fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("notes.txt", "original\n", "add notes")
        .unwrap()
        .write_worktree("notes.txt", b"changed\n")
        .unwrap()
        .build()
}

fn launch(repo: &FixtureRepo, session: &str) -> (Terminal, tempfile::TempDir) {
    common::launch(repo.root())
        .session(session)
        .size(200, 50)
        .ready_when("the toolbar and the log", |s| {
            s.contains("Stash") && s.contains("Pop") && s.contains("add notes")
        })
        .mouse_capture()
        .start()
}

/// Runs the palette entry whose title contains `title`: filter to exactly
/// one row, then Enter.
fn palette(term: &mut Terminal, title: &str) {
    palette_n(term, title, 1);
}

/// As [`palette`] when `rows` entries match and the first is the one run.
fn palette_n(term: &mut Terminal, title: &str, rows: usize) {
    term.send_keys("C-p").unwrap();
    term.send_text(title).unwrap();
    term.wait_for("one palette row left", STEP, |s| {
        s.contains(&format!("Command Palette ({rows}/")) && s.contains(title)
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for_text_to_go("Command Palette", STEP)
        .unwrap_or_else(|e| panic!("the palette should close: {e}"));
}

fn leader(term: &mut Terminal, key: &str) {
    term.send_keys("C-g").unwrap();
    term.wait_for_text("Ctrl+G:", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_text(key).unwrap();
}

fn toolbar(term: &mut Terminal, label: &str) {
    let (c, r) = term.wait_for_text(label, STEP).unwrap();
    term.click(c + 1, r).unwrap();
}

fn stashes(root: &Path) -> Option<usize> {
    let out = std::process::Command::new("git")
        .args(["stash", "list"])
        .current_dir(root)
        .output()
        .ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).lines().count())
}

fn wait_stashes(root: &Path, n: usize) {
    for _ in 0..300 {
        if stashes(root) == Some(n) {
            return;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    panic!("expected {n} stashes, found {:?}", stashes(root));
}

fn branch_exists(root: &Path, name: &str) -> bool {
    common::git(root, &["branch", "--list", name])
        .trim()
        .ends_with(name)
}

fn tag_exists(root: &Path, name: &str) -> bool {
    common::git(root, &["tag", "--list", name]).trim() == name
}

fn wait_until(what: &str, mut ok: impl FnMut() -> bool) {
    for _ in 0..300 {
        if ok() {
            return;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    panic!("timed out waiting for {what}");
}

/// New Branch from the toolbar, the leader (`Ctrl+G b`) and the palette:
/// each opens the name prompt, and Enter creates the branch.
#[test]
fn new_branch_is_the_same_from_the_toolbar_the_leader_and_the_palette() {
    let repo = fixture();
    let (mut term, _c) = launch(&repo, "pty-parity-branch");
    for (i, surface) in SURFACES.into_iter().enumerate() {
        let name = format!("br-{i}");
        match surface {
            Surface::Toolbar => toolbar(&mut term, "Branch"),
            Surface::Leader => leader(&mut term, "b"),
            Surface::Palette => palette(&mut term, "New Branch at HEAD"),
        }
        term.wait_for("the branch-name prompt", STEP, |s| {
            s.contains("enter branch name") || s.contains("New branch name")
        })
        .unwrap_or_else(|e| panic!("{surface:?}: {e}"));
        term.send_text(&name).unwrap();
        term.send_keys("Enter").unwrap();
        wait_until(&format!("{surface:?} to create {name}"), || {
            branch_exists(repo.root(), &name)
        });
        term.wait_for("the prompt to close", STEP, |s| {
            !s.contains("enter branch name") && !s.contains("New branch name")
        })
        .unwrap_or_else(|e| panic!("{surface:?}: {e}"));
    }
    term.stop().unwrap();
}

/// New Tag, the same three ways.
#[test]
fn new_tag_is_the_same_from_the_toolbar_the_leader_and_the_palette() {
    let repo = fixture();
    let (mut term, _c) = launch(&repo, "pty-parity-tag");
    for (i, surface) in SURFACES.into_iter().enumerate() {
        let name = format!("tg-{i}");
        match surface {
            Surface::Toolbar => toolbar(&mut term, "Tag"),
            Surface::Leader => leader(&mut term, "t"),
            Surface::Palette => palette(&mut term, "New Tag at HEAD"),
        }
        term.wait_for("the tag-name prompt", STEP, |s| {
            s.contains("enter tag name") || s.contains("New tag name")
        })
        .unwrap_or_else(|e| panic!("{surface:?}: {e}"));
        term.send_text(&name).unwrap();
        term.send_keys("Enter").unwrap();
        wait_until(&format!("{surface:?} to create {name}"), || {
            tag_exists(repo.root(), &name)
        });
        term.wait_for("the prompt to close", STEP, |s| {
            !s.contains("enter tag name") && !s.contains("New tag name")
        })
        .unwrap_or_else(|e| panic!("{surface:?}: {e}"));
    }
    term.stop().unwrap();
}

/// Stash Changes three ways: each stashes the dirty file (so the stash
/// count goes 1, 2, 3 and the tree is clean again), then Pop three ways
/// takes them back one by one.
#[test]
fn stash_and_pop_work_from_every_surface() {
    let repo = fixture();
    let file = repo.root().join("notes.txt");
    let (mut term, _c) = launch(&repo, "pty-parity-stash");
    for (i, surface) in SURFACES.into_iter().enumerate() {
        std::fs::write(&file, format!("edit {i}\n")).unwrap();
        wait_until("the edit to be visible to git", || {
            !common::git(repo.root(), &["status", "--porcelain"])
                .trim()
                .is_empty()
        });
        match surface {
            Surface::Toolbar => toolbar(&mut term, "Stash"),
            Surface::Leader => leader(&mut term, "T"),
            Surface::Palette => palette_n(&mut term, "Stash All Changes", 2),
        }
        wait_stashes(repo.root(), i + 1);
        wait_until(&format!("{surface:?} to clean the tree"), || {
            std::fs::read_to_string(&file).is_ok_and(|t| t == "original\n")
        });
    }
    // Pop, newest first, one surface each.
    for (i, surface) in SURFACES.into_iter().enumerate() {
        match surface {
            Surface::Toolbar => toolbar(&mut term, "Pop"),
            Surface::Leader => leader(&mut term, "o"),
            Surface::Palette => palette(&mut term, "Stash Pop"),
        }
        wait_stashes(repo.root(), 2 - i);
        // Each pop restores a file the next stash then conflicts with, so
        // put it back to the committed content before the next one.
        if i < 2 {
            common::git(repo.root(), &["checkout", "--", "notes.txt"]);
        }
    }
    term.stop().unwrap();
}

/// Stash Pop with no stash: the toolbar and the leader both say "no stash
/// to pop"; the palette refuses the row up front instead (grayed with
/// Pinned as a known divergence.
#[test]
fn pop_without_a_stash_answers_differently_per_surface_today() {
    let repo = fixture();
    // One terminal per surface, so the status message cannot be a stale
    // one from the previous surface.
    for surface in [Surface::Toolbar, Surface::Leader] {
        let (mut term, _c) = launch(&repo, &format!("pty-parity-nopop-{surface:?}"));
        match surface {
            Surface::Toolbar => toolbar(&mut term, "Pop"),
            _ => leader(&mut term, "o"),
        }
        term.wait_for_text("no stash to pop", STEP)
            .unwrap_or_else(|e| panic!("{surface:?}: {e}"));
        term.stop().unwrap();
    }
    let (mut term, _c) = launch(&repo, "pty-parity-nopop-palette");
    term.send_keys("C-p").unwrap();
    term.send_text("Stash Pop").unwrap();
    // The reason is part of the wait: the row can be drawn a frame before the
    // stash list has arrived and the row is grayed (load flake).
    let screen = term
        .wait_for("the palette graying Stash Pop with its reason", STEP, |s| {
            s.contains("Command Palette (1/")
                && s.contains("Git: Stash Pop")
                && s.contains("there are no stashes")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        screen.contains("there are no stashes"),
        "the palette grays Pop with its reason:\n{}",
        screen.text()
    );
    term.stop().unwrap();
}
