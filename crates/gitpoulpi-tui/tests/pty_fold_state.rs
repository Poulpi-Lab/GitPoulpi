//! The collapsed/expanded state of the foldable panes persists per
//! repository, across a tab switch (which rebuilds the window) and across
//! a restart. Real binary, temp config dir, two fixture repos.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);

fn exit(mut term: Terminal) {
    term.send_keys("q").unwrap();
    for _ in 0..100 {
        if term.exited() {
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    let _ = term.stop();
}

fn launch(repo: &std::path::Path, config: &std::path::Path) -> Terminal {
    common::launch(repo)
        .implicit_session()
        .config_dir(config)
        .no_exit_pause()
        .size(140, 40)
        .ready("alpha-commit")
        .spawn()
}

fn sidebar_header(term: &Terminal, header: &str) -> Option<(u16, u16)> {
    term.screen()
        .find_all(header)
        .into_iter()
        .find(|(col, _)| *col < 30)
}

#[test]
fn collapsed_sections_survive_a_tab_switch_and_a_restart() {
    let alpha = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "alpha-commit")
        .unwrap()
        .build();
    let bravo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("b.txt", "b\n", "bravo-commit")
        .unwrap()
        .build();
    common::git_ok(alpha.root(), &["tag", "v-fold-me"]);
    // A dirty file so the right pane has Unstaged Files.
    std::fs::write(alpha.root().join("a.txt"), "changed\n").unwrap();
    let config = tempfile::tempdir().unwrap();
    std::fs::write(
        config.path().join("recent.toml"),
        format!(
            "[[repos]]\npath = '{}'\nname = 'bravo-repo'\nlast_opened = 1\n",
            bravo.root().display()
        ),
    )
    .unwrap();

    let mut term = launch(alpha.root(), config.path());
    term.wait_for("the tag listed", STEP, |s| s.contains("v-fold-me"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.wait_for("unstaged pane", STEP, |s| s.contains("Unstaged Files"))
        .unwrap_or_else(|e| panic!("{e}"));

    // Fold TAGS and Unstaged Files with the mouse.
    let (c, r) = sidebar_header(&term, "TAGS (").expect("TAGS header");
    term.click(c + 1, r).unwrap();
    term.wait_for("tags folded", STEP, |s| !s.contains("v-fold-me"))
        .unwrap_or_else(|e| panic!("{e}"));
    let before = term.screen();
    let unstaged_rows_open = before.text().matches("a.txt").count();
    let (c, r) = before.find("Unstaged Files").expect("unstaged header");
    term.click(c + 2, r).unwrap();
    term.wait_for("unstaged folded", STEP, |s| {
        s.find("Unstaged Files").is_some() && !s.contains("changed") && !s.contains("a.txt\u{0}")
    })
    .ok();
    let folded_screen = term
        .wait_for_stable(Duration::from_millis(500), STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    let unstaged_rows_folded = folded_screen.text().matches("a.txt").count();

    assert!(
        unstaged_rows_folded < unstaged_rows_open,
        "folding Unstaged Files did not hide its rows:\n{}",
        folded_screen.text()
    );

    // Open bravo as a second tab, then go back to alpha.
    term.send_keys("C-o").unwrap();
    term.wait_for("the management tab", STEP, |s| {
        s.contains("Repository Management") && s.contains("bravo-repo")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("/").unwrap();
    term.send_text("bravo").unwrap();
    term.wait_for("the filter applied", STEP, |s| {
        s.contains("Enter open best match")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("bravo open", STEP, |s| {
        s.contains("bravo-commit") && !s.contains("Repository Management")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    for _ in 0..3 {
        term.send_text("\x1b[5;5~").unwrap();
        if term
            .wait_for("alpha back", Duration::from_secs(5), |s| {
                s.contains("alpha-commit") && !s.contains("bravo-commit")
            })
            .is_ok()
        {
            break;
        }
    }
    let back = term
        .wait_for("alpha back", STEP, |s| s.contains("alpha-commit"))
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        !back.contains("v-fold-me"),
        "TAGS is expanded again after the tab switch:\n{}",
        back.text()
    );
    assert!(back.contains("TAGS ("));
    let back = term
        .wait_for_stable(Duration::from_millis(500), STEP)
        .unwrap();
    assert_eq!(
        back.text().matches("a.txt").count(),
        unstaged_rows_folded,
        "Unstaged Files unfolded after the tab switch:\n{}",
        back.text()
    );
    exit(term);

    // Restart: still folded.
    let term = launch(alpha.root(), config.path());
    let screen = term
        .wait_for_stable(Duration::from_millis(800), STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(screen.contains("TAGS ("), "{}", screen.text());
    assert!(
        !screen.contains("v-fold-me"),
        "TAGS expanded after a restart:\n{}",
        screen.text()
    );
    assert_eq!(screen.text().matches("a.txt").count(), unstaged_rows_folded);
    exit(term);
}

/// Click the first on-screen `needle` (folder rows are `<chevron> name/`).
fn click_text(term: &mut Terminal, needle: &str) {
    let (c, r) = term
        .wait_for_text(needle, STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.click(c + 1, r).unwrap();
}

/// Select the WIP row (the Unstaged tree) / the HEAD commit (its files).
fn show_wip(term: &mut Terminal) {
    click_text(term, "\u{25CC}");
    term.wait_for("the unstaged tree", STEP, |s| s.contains("Unstaged Files"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

fn show_head(term: &mut Terminal) {
    click_text(term, "alpha-commit");
    term.wait_for("the commit files", STEP, |s| {
        s.contains("View all files") && !s.contains("Unstaged Files")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // The list mode is per window (Path by default): pick Tree.
    if !term.screen().contains("\u{25BE} alpha/") {
        click_text(term, "Tree  ");
    }
    term.wait_for("the commit tree", STEP, |s| s.contains("\u{25BE} alpha/"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

fn assert_folders_collapsed(term: &mut Terminal, when: &str) {
    show_wip(term);
    term.wait_for(&format!("Unstaged folder collapsed {when}"), STEP, |s| {
        s.contains("\u{25B8} gamma/")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    show_head(term);
    term.wait_for(&format!("commit folder collapsed {when}"), STEP, |s| {
        s.contains("\u{25B8} beta/") && s.contains("\u{25BE} alpha/")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

#[test]
fn collapsed_tree_folders_survive_a_tab_switch_and_a_restart() {
    let alpha = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("alpha/a.txt", "a\n", "base")
        .unwrap()
        .commit_file("beta/b.txt", "b\n", "alpha-commit")
        .unwrap()
        .commit_file("gamma/g.txt", "g\n", "gamma-base")
        .unwrap()
        .build();
    // Order matters for the first commit shown: put alpha-commit on top.
    common::git_ok(alpha.root(), &["reset", "--hard", "HEAD~2"]);
    std::fs::create_dir_all(alpha.root().join("alpha")).unwrap();
    std::fs::write(alpha.root().join("alpha/a.txt"), "a2\n").unwrap();
    std::fs::create_dir_all(alpha.root().join("gamma")).unwrap();
    std::fs::write(alpha.root().join("gamma/g.txt"), "g\n").unwrap();
    std::fs::create_dir_all(alpha.root().join("beta")).unwrap();
    std::fs::write(alpha.root().join("beta/b.txt"), "b\n").unwrap();
    common::git_ok(alpha.root(), &["add", "alpha/a.txt", "beta/b.txt"]);
    common::git_ok(alpha.root(), &["commit", "-q", "-m", "alpha-commit"]);
    std::fs::write(alpha.root().join("alpha/a.txt"), "a3\n").unwrap();
    // Untracked gamma/g.txt gives the Unstaged tree a folder of its own.
    let bravo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("b.txt", "b\n", "bravo-commit")
        .unwrap()
        .build();
    let config = tempfile::tempdir().unwrap();
    std::fs::write(
        config.path().join("recent.toml"),
        format!(
            "[[repos]]\npath = '{}'\nname = 'bravo-repo'\nlast_opened = 1\n",
            bravo.root().display()
        ),
    )
    .unwrap();

    let mut term = launch(alpha.root(), config.path());
    // Commit files tree: collapse beta/ (HEAD is selected on open).
    show_head(&mut term);
    click_text(&mut term, "\u{25BE} beta/");
    term.wait_for("beta collapsed", STEP, |s| s.contains("\u{25B8} beta/"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Unstaged tree: collapse gamma/.
    show_wip(&mut term);
    click_text(&mut term, "\u{25BE} gamma/");
    term.wait_for("gamma collapsed", STEP, |s| s.contains("\u{25B8} gamma/"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Let the tick persist, then switch tab and come back.
    std::thread::sleep(Duration::from_millis(500));

    term.send_keys("C-o").unwrap();
    term.wait_for("the management tab", STEP, |s| {
        s.contains("Repository Management") && s.contains("bravo-repo")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("/").unwrap();
    term.send_text("bravo").unwrap();
    term.wait_for("the filter applied", STEP, |s| {
        s.contains("Enter open best match")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("bravo open", STEP, |s| {
        s.contains("bravo-commit") && !s.contains("Repository Management")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    for _ in 0..3 {
        term.send_text("\x1b[5;5~").unwrap();
        if term
            .wait_for("alpha back", Duration::from_secs(5), |s| {
                s.contains("alpha-commit") && !s.contains("bravo-commit")
            })
            .is_ok()
        {
            break;
        }
    }
    term.wait_for("alpha back", STEP, |s| s.contains("alpha-commit"))
        .unwrap_or_else(|e| panic!("{e}"));
    assert_folders_collapsed(&mut term, "after the tab switch");
    exit(term);

    let mut term = launch(alpha.root(), config.path());
    assert_folders_collapsed(&mut term, "after a restart");
    exit(term);
}

#[test]
fn the_footer_names_the_context_menu_key() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "alpha-commit")
        .unwrap()
        .build();
    let config = tempfile::tempdir().unwrap();
    let term = common::launch(repo.root())
        .implicit_session()
        .config_dir(config.path())
        .no_exit_pause()
        .size(200, 40)
        .ready("alpha-commit")
        .spawn();
    term.wait_for("the menu hint before quit", STEP, |s| {
        s.contains("Ctrl+P: commands  Shift+F10: menu  q: quit")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    exit(term);
}
