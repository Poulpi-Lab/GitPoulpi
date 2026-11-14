//! The Create worktree dialog's names through a pty (`xtask::pty`): the
//! generated `worktree-<id>` Name and branch, "Regenerate", "Name after
//! branch", an existing branch checked out, and the ✦ AI naming with a
//! **scripted local agent** (a program printing a canned answer): no socket
//! is opened. Real binary, daemon and system git; temp config dirs only.
//! Same rules as `pty_worktree_pane.rs`: no coordinates for keys, no sleeps,
//! every wait names the state it waits for.

mod common;

use std::path::{Path, PathBuf};
use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(30);

struct Fx {
    _repo: FixtureRepo,
    root: PathBuf,
    config: tempfile::TempDir,
}

fn fixture(config_toml: &str) -> Fx {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "base commit")
        .unwrap()
        .branch("other")
        .unwrap()
        .checkout("main")
        .unwrap()
        .build();
    let root = repo.root().to_path_buf();
    let config = tempfile::tempdir().unwrap();
    std::fs::write(config.path().join("config.toml"), config_toml).unwrap();
    Fx {
        _repo: repo,
        root,
        config,
    }
}

fn launch(fx: &Fx, session: &str) -> Terminal {
    common::launch(&fx.root)
        .session(session)
        .config_dir(fx.config.path())
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

/// Opens the dialog from the current branch (`main`, checked out: a new
/// branch is needed) through the palette: the WORKTREES section is hidden
/// while the repository has no linked worktree.
fn open_dialog(term: &mut Terminal) -> Screen {
    term.send_keys("C-p").unwrap();
    term.send_text("Create Worktree from Current").unwrap();
    term.wait_for("the palette entry", STEP, |s| {
        s.contains("Refs: Create Worktree from Current Branch")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    term.wait_for("the dialog", STEP, |s| {
        s.contains("Create worktree from main") && s.contains("Worktree branch")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()))
}

/// The Name field's and the branch field's names, from the dialog's screen.
fn name_and_branch(screen: &Screen) -> (String, String) {
    let line_of = |label: &str| {
        screen
            .text()
            .lines()
            .find(|l| l.contains(label))
            .map(str::to_string)
            .unwrap_or_else(|| panic!("no {label} row\n{}", screen.text()))
    };
    let first = |line: String| {
        generated_names(&line)
            .into_iter()
            .next()
            .unwrap_or_else(|| panic!("no generated name in {line:?}"))
    };
    (first(line_of("Name   ")), first(line_of("Worktree branch")))
}

fn wait_dialog_text(term: &Terminal, what: &str, needle: &str) {
    term.wait_for(what, STEP, |s| s.contains(needle))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

#[test]
fn the_dialog_opens_with_the_same_generated_name_and_branch_and_regenerate_changes_both() {
    let fx = fixture("");
    let mut term = launch(&fx, "pty-wt-names-gen");
    let screen = open_dialog(&mut term);
    let (name, branch) = name_and_branch(&screen);
    assert_eq!(name, branch, "the same id for the Name and the branch");
    let repo_name = fx.root.file_name().unwrap().to_string_lossy().into_owned();
    assert!(
        screen.contains(&format!("{repo_name}-{name}")),
        "the Folder follows the Name\n{}",
        screen.text()
    );

    // Regenerate (the button): a new id for both.
    common::click_text_until(&mut term, "Regenerate", "a new name", |s| {
        !s.contains(&name)
    });
    let (name2, branch2) = name_and_branch(&term.screen());
    assert_ne!(name2, name);
    assert_eq!(name2, branch2);

    // Its keyboard twin: Alt+R.
    term.send_keys("M-r").unwrap();
    term.wait_for("another new name", STEP, |s| !s.contains(&name2))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (name3, branch3) = name_and_branch(&term.screen());
    assert_eq!(name3, branch3);

    // Create (Tab, Tab from the Regenerate button the click focused): the
    // folder and the branch carry the last generated id.
    term.send_keys("Tab").unwrap(); // Open as a tab
    term.send_keys("Tab").unwrap(); // Create
    term.send_keys("Enter").unwrap();
    wait_until("the worktree is created", || {
        worktree_list(&fx.root).contains(&format!("branch refs/heads/{name3}"))
    });
    assert!(worktree_list(&fx.root).contains(&format!("{repo_name}-{name3}")));
    let _ = std::fs::remove_dir_all(
        fx.root
            .parent()
            .unwrap()
            .join(format!("{repo_name}-{name3}")),
    );
}

#[test]
fn an_existing_branch_is_checked_out_and_name_after_branch_names_the_worktree_after_it() {
    let fx = fixture("");
    let mut term = launch(&fx, "pty-wt-names-existing");
    open_dialog(&mut term);
    // Tab to the Worktree branch field (Name, Name after branch, Folder,
    // Browse, Ref, Branch) and pick `other` from its list.
    for _ in 0..5 {
        term.send_keys("Tab").unwrap();
    }
    term.send_keys("Enter").unwrap();
    wait_dialog_text(
        &term,
        "the branch list",
        "required: the branch is already checked out",
    );
    term.send_text("other").unwrap();
    term.send_keys("Enter").unwrap();
    wait_dialog_text(
        &term,
        "the existing branch is chosen",
        "checks out this branch",
    );
    // The Name did not follow: still generated until we ask.
    assert!(!generated_names(&term.text()).is_empty());

    // "Name after branch" (Alt+N): `worktree-other`.
    term.send_keys("M-n").unwrap();
    term.wait_for("the name after the branch", STEP, |s| {
        s.text()
            .lines()
            .any(|l| l.contains("Name   ") && l.contains("worktree-other"))
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let repo_name = fx.root.file_name().unwrap().to_string_lossy().into_owned();
    wait_dialog_text(
        &term,
        "the folder follows",
        &format!("{repo_name}-worktree-other"),
    );

    // Create: `other` is checked out in the new worktree.
    term.send_keys("Tab").unwrap(); // Regenerate
    term.send_keys("Tab").unwrap(); // Open as a tab
    term.send_keys("Tab").unwrap(); // Create
    term.send_keys("Enter").unwrap();
    wait_until("the worktree on other", || {
        let list = worktree_list(&fx.root);
        list.contains("branch refs/heads/other")
            && list.contains(&format!("{repo_name}-worktree-other"))
    });
    let _ = std::fs::remove_dir_all(
        fx.root
            .parent()
            .unwrap()
            .join(format!("{repo_name}-worktree-other")),
    );
}

#[test]
fn a_branch_that_already_says_worktree_keeps_its_name_without_doubling_it() {
    let fx = fixture("");
    let mut term = launch(&fx, "pty-wt-names-no-double");
    open_dialog(&mut term);
    for _ in 0..5 {
        term.send_keys("Tab").unwrap();
    }
    // Typing on the branch field opens its list; a name that is not a
    // branch is offered as a new one.
    term.send_text("worktree-foo").unwrap();
    wait_dialog_text(&term, "the own-name row", "worktree-foo");
    term.send_keys("Enter").unwrap();
    term.wait_for("the new branch is chosen", STEP, |s| {
        s.contains("new branch from main")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("M-n").unwrap();
    term.wait_for(
        "the name is the branch, not worktree-worktree-foo",
        STEP,
        |s| {
            s.text()
                .lines()
                .any(|l| l.contains("Name   ") && l.contains("worktree-foo"))
        },
    )
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(
        !term.text().contains("worktree-worktree-foo"),
        "{}",
        term.text()
    );
    for _ in 0..3 {
        term.send_keys("Tab").unwrap(); // Regenerate, Open as a tab, Create
    }
    term.send_keys("Enter").unwrap();
    let repo_name = fx.root.file_name().unwrap().to_string_lossy().into_owned();
    wait_until("the new branch", || {
        worktree_list(&fx.root).contains("branch refs/heads/worktree-foo")
    });
    let _ = std::fs::remove_dir_all(
        fx.root
            .parent()
            .unwrap()
            .join(format!("{repo_name}-worktree-foo")),
    );
}

#[cfg(feature = "ai")]
mod ai {
    use super::*;
    use gitpoulpi_testkit::fake_agent::{agent_calls, ai_config_toml, write_fake_agent};

    /// What the agent answers. Plain words only (the Windows stand-in echoes
    /// it through `cmd`).
    const ANSWER: &[&str] =
        &["{\"name\": \"Empty Config Crash\", \"branch\": \"fix/empty-config\"}"];

    #[test]
    fn the_ai_button_names_the_worktree_and_the_branch_from_a_description() {
        let tools = tempfile::tempdir().unwrap();
        let agent = write_fake_agent(tools.path(), ANSWER);
        let fx = fixture(&ai_config_toml(&agent));
        let mut term = launch(&fx, "pty-wt-names-ai");
        open_dialog(&mut term);
        // The button (and its Alt+A twin) opens the box inside the dialog.
        wait_dialog_text(&term, "the AI button", "Name with AI");
        term.send_keys("M-a").unwrap();
        wait_dialog_text(
            &term,
            "the description box",
            "Describe the feature, bug or task",
        );
        // Typed text (the status line is never typed into); Tab to "Name it".
        term.send_text("fix the crash when the config is empty")
            .unwrap();
        wait_dialog_text(&term, "the description typed", "the config is empty");
        term.send_keys("Tab").unwrap();
        term.send_keys("Enter").unwrap();
        term.wait_for("both fields filled by the agent", STEP, |s| {
            let text = s.text();
            text.lines()
                .any(|l| l.contains("Name   ") && l.contains("worktree-empty-config-crash"))
                && text
                    .lines()
                    .any(|l| l.contains("Worktree branch") && l.contains("fix/empty-config"))
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
        assert_eq!(agent_calls(&agent), 1);
        let repo_name = fx.root.file_name().unwrap().to_string_lossy().into_owned();
        wait_dialog_text(
            &term,
            "the folder follows",
            &format!("{repo_name}-worktree-empty-config-crash"),
        );
        // Create with them (Enter on the Name field is the default button).
        term.send_keys("Enter").unwrap();
        wait_until("the worktree on the AI's branch", || {
            let list = worktree_list(&fx.root);
            list.contains("branch refs/heads/fix/empty-config")
                && list.contains(&format!("{repo_name}-worktree-empty-config-crash"))
        });
        let _ = std::fs::remove_dir_all(
            fx.root
                .parent()
                .unwrap()
                .join(format!("{repo_name}-worktree-empty-config-crash")),
        );
    }
}
