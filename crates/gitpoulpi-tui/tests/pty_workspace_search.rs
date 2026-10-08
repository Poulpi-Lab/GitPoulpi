#![cfg(feature = "global-search")]
//! The workspace search popup (`Ctrl+Alt+F`): two repositories open as tabs,
//! a query typed in the first one finds refs, stashes and commits of the
//! second, and Enter switches to its tab and reveals the result (through the
//! agent-view intent machinery). Temp config dir, fixture repositories, no
//! network.

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_ipc::ui::UiState;
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);
const SESSION: &str = "wsearch";

struct Rig {
    alpha: FixtureRepo,
    bravo: FixtureRepo,
    config: tempfile::TempDir,
    term: Terminal,
}

/// `alpha` (active) and `bravo` as two tabs of one session. `bravo` has a
/// branch, a tag, a stash and an old commit nothing else points at.
fn rig() -> Rig {
    let alpha = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "alpha base commit")
        .unwrap()
        .build();
    let bravo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("b.txt", "b\n", "bravo base commit")
        .unwrap()
        .commit_file("c.txt", "c\n", "needle in an old bravo commit")
        .unwrap()
        .commit_file("d.txt", "d\n", "bravo newer commit")
        .unwrap()
        .commit_file("e.txt", "e\n", "bravo newest commit")
        .unwrap()
        .build();
    bravo.git(&["branch", "feature-zebra", "HEAD~2"]);
    bravo.git(&["tag", "release-zebra", "HEAD~1"]);
    std::fs::write(bravo.root().join("b.txt"), "changed\n").unwrap();
    bravo.git(&["stash", "push", "-m", "parked giraffe work"]);

    let config = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(config.path().join("sessions")).unwrap();
    std::fs::write(
        config.path().join("sessions").join(format!("{SESSION}.toml")),
        format!(
            "[[tabs]]\nrepo_path = '{}'\nactive = true\n\n[[tabs]]\nrepo_path = '{}'\nactive = false\n",
            alpha.root().canonicalize().unwrap().display(),
            bravo.root().canonicalize().unwrap().display(),
        ),
    )
    .unwrap();
    let term = common::launch(alpha.root())
        .session(SESSION)
        .config_dir(config.path())
        .size(180, 50)
        .mouse_capture()
        .ready("alpha base commit")
        .spawn();
    Rig {
        alpha,
        bravo,
        config,
        term,
    }
}

fn state(config: &Path) -> UiState {
    common::ui_state(config, SESSION)
}

fn selected_commit(config: &Path) -> Option<String> {
    state(config)
        .log
        .and_then(|l| l["selected_commit"].as_str().map(str::to_string))
}

fn open_search(term: &mut Terminal) {
    term.send_keys("M-C-f").unwrap();
    term.wait_for("the search popup", STEP, |s| {
        s.contains("Search open repositories (2)")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

fn on_repo(config: &Path, repo: &FixtureRepo) {
    let want = repo
        .root()
        .canonicalize()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    let deadline = std::time::Instant::now() + STEP;
    loop {
        let now = state(config).repo;
        if now.as_deref() == Some(want.as_str()) {
            return;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "still on {now:?}, wanted {want}"
        );
        std::thread::sleep(Duration::from_millis(100));
    }
}

fn wait_selected(config: &Path, oid: &str) {
    let deadline = std::time::Instant::now() + STEP;
    loop {
        let now = selected_commit(config);
        if now.as_deref() == Some(oid) {
            return;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "selected {now:?}, wanted {oid}"
        );
        std::thread::sleep(Duration::from_millis(100));
    }
}

/// Enter on the highlighted result, until the window is on `repo`. An Enter
/// that reached the popup before its results were in (or before the
/// highlight was on one) does nothing, so it is only repeated while the
/// popup is still up; the state is what proves the jump.
fn go_to(term: &mut Terminal, config: &Path, repo: &FixtureRepo) {
    let want = repo
        .root()
        .canonicalize()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    term.act_until(
        "the window on the other repository",
        Duration::from_secs(6),
        STEP,
        |t| {
            if t.screen().contains("Search open repositories") {
                t.send_keys("Enter")?;
            }
            Ok(())
        },
        |_| state(config).repo.as_deref() == Some(want.as_str()),
    )
    .unwrap_or_else(|e| panic!("{e}"));
}

#[test]
fn a_branch_of_the_other_repository_switches_tab_and_selects_it() {
    let mut rig = rig();
    common::wait_for_viewer(rig.config.path(), SESSION);
    on_repo(rig.config.path(), &rig.alpha);
    open_search(&mut rig.term);
    rig.term.send_text("zebra").unwrap();
    rig.term
        .wait_for("the branch and the tag of bravo", STEP, |s| {
            s.contains("feature-zebra") && s.contains("release-zebra") && s.contains("Branches")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    // The first result is the branch (kinds are grouped, repositories first,
    // and no repository matches "zebra").
    go_to(&mut rig.term, rig.config.path(), &rig.bravo);
    let tip = rig.bravo.git(&["rev-parse", "feature-zebra"]);
    wait_selected(rig.config.path(), tip.trim());
    let _ = rig.term.stop();
}

#[test]
fn a_commit_subject_finds_and_reveals_the_commit_in_the_log() {
    let mut rig = rig();
    common::wait_for_viewer(rig.config.path(), SESSION);
    open_search(&mut rig.term);
    rig.term.send_text("c: needle").unwrap();
    rig.term
        // Only the answer to the whole query: an earlier, shorter one ("c: ")
        // lists every commit too, and Enter on it goes to whichever is
        // highlighted when the list is replaced.
        .wait_for("the commit alone", STEP, |s| {
            s.contains("needle in an old bravo commit")
                && s.contains("Commits")
                && !s.contains("bravo base commit")
                && !s.contains("bravo newest commit")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    go_to(&mut rig.term, rig.config.path(), &rig.bravo);
    let oid = rig.bravo.git(&["rev-parse", "HEAD~2"]);
    wait_selected(rig.config.path(), oid.trim());
    let _ = rig.term.stop();
}

#[test]
fn a_stash_message_finds_the_stash_and_selects_its_sidebar_row() {
    let mut rig = rig();
    common::wait_for_viewer(rig.config.path(), SESSION);
    open_search(&mut rig.term);
    rig.term.send_text("s: giraffe").unwrap();
    rig.term
        .wait_for("the stash", STEP, |s| s.contains("parked giraffe work"))
        .unwrap_or_else(|e| panic!("{e}"));
    go_to(&mut rig.term, rig.config.path(), &rig.bravo);
    let deadline = std::time::Instant::now() + STEP;
    while state(rig.config.path()).focus.as_deref() != Some("refs") {
        assert!(
            std::time::Instant::now() < deadline,
            "the Refs pane never got the focus"
        );
        std::thread::sleep(Duration::from_millis(100));
    }
    let _ = rig.term.stop();
}

#[test]
fn a_prefix_narrows_to_its_kind_and_a_chip_turns_a_kind_off() {
    let mut rig = rig();
    common::wait_for_viewer(rig.config.path(), SESSION);
    open_search(&mut rig.term);
    rig.term.send_text("zebra").unwrap();
    rig.term
        .wait_for("branch and tag", STEP, |s| {
            s.contains("feature-zebra") && s.contains("release-zebra")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    // `t:` keeps tags only.
    // (Home and End move the list, Ctrl+A is the field's caret key.)
    rig.term.send_keys("C-a").unwrap();
    rig.term.send_text("t: ").unwrap();
    rig.term
        .wait_for("tags only", STEP, |s| {
            s.contains("release-zebra") && !s.contains("feature-zebra")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    // Without the prefix both are back; Alt+3 turns the Tags chip off.
    for _ in 0..3 {
        rig.term.send_keys("BSpace").unwrap();
    }
    rig.term
        .wait_for("both again", STEP, |s| {
            s.contains("feature-zebra") && s.contains("release-zebra")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    rig.term.send_keys("M-3").unwrap();
    rig.term
        .wait_for("no tag", STEP, |s| {
            s.contains("feature-zebra") && !s.contains("release-zebra")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    // Esc closes and nothing moved.
    rig.term.send_keys("Escape").unwrap();
    rig.term
        .wait_for_text_to_go("Search open repositories", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    on_repo(rig.config.path(), &rig.alpha);
    let _ = rig.term.stop();
}

/// The mouse twins: a click on a kind chip turns the kind off, a double click
/// on a result goes there, and a click outside the popup closes it.
#[test]
fn the_mouse_toggles_chips_double_click_goes_and_outside_closes() {
    let mut rig = rig();
    common::wait_for_viewer(rig.config.path(), SESSION);
    open_search(&mut rig.term);
    rig.term.send_text("zebra").unwrap();
    let screen = rig
        .term
        .wait_for("branch and tag", STEP, |s| {
            s.contains("feature-zebra") && s.contains("release-zebra")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    // The Tags chip sits on the line that also holds the Branches chip.
    let (branches_col, chips_row) = screen
        .find_all("Branches")
        .into_iter()
        .find(|(_, row)| screen.find_all("Tags").iter().any(|(_, r)| r == row))
        .expect("the chip row");
    let (tags_col, _) = screen
        .find_all("Tags")
        .into_iter()
        .find(|(_, r)| *r == chips_row)
        .unwrap();
    assert!(tags_col > branches_col);
    rig.term.click(tags_col + 1, chips_row).unwrap();
    rig.term
        .wait_for("the tag gone", STEP, |s| {
            s.contains("feature-zebra") && !s.contains("release-zebra")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    // A double click on the branch goes there. The row is taken once it has
    // held still and located afresh for every attempt (a double click is two
    // events the TUI times itself), and only while the popup is up.
    let want = rig
        .bravo
        .root()
        .canonicalize()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    let config = rig.config.path().to_path_buf();
    rig.term
        .act_until(
            "the branch opened by a double click",
            Duration::from_secs(5),
            STEP,
            |t| {
                if !t.screen().contains("Search open repositories") {
                    return Ok(());
                }
                if let Some((col, row)) = t
                    .screen()
                    .find_all("feature-zebra")
                    .into_iter()
                    .find(|(_, r)| *r > chips_row)
                {
                    t.double_click(col + 1, row)?;
                }
                Ok(())
            },
            |_| state(&config).repo.as_deref() == Some(want.as_str()),
        )
        .unwrap_or_else(|e| panic!("{e}"));
    on_repo(rig.config.path(), &rig.bravo);
    let tip = rig.bravo.git(&["rev-parse", "feature-zebra"]);
    wait_selected(rig.config.path(), tip.trim());
    // And a click outside the popup closes it.
    open_search(&mut rig.term);
    rig.term.click(0, 49).unwrap();
    rig.term
        .wait_for_text_to_go("Search open repositories", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    let _ = rig.term.stop();
}
