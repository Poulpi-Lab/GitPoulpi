//! Settings > Sequences end to end through a pty: a keyboard-only pass (add
//! the Backup preset, reorder its steps, set Never on the AI step, bind a
//! key through the capture popup, see the palette row with that key) and a
//! mouse pass (add a step through the picker, the cross of the picker, the
//! back button of the steps). `config.toml` follows every step. A temp
//! config dir per test; nothing is located by a hard-coded coordinate.

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);

fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial commit")
        .unwrap()
        .build()
}

fn launch(repo: &Path, config: &Path, session: &str) -> Terminal {
    common::launch(repo)
        .session(session)
        .config_dir(config)
        .size(200, 60)
        .no_exit_pause()
        .ready_when("the log", |s| s.contains("Message"))
        .mouse_capture()
        .spawn()
}

fn stored(config: &Path) -> String {
    std::fs::read_to_string(config.join("config.toml")).unwrap_or_default()
}

fn parsed(config: &Path) -> toml::Table {
    stored(config).parse().unwrap_or_default()
}

fn wait_stored(term: &Terminal, config: &Path, what: &str, ok: impl Fn(&toml::Table) -> bool) {
    term.wait_for(what, STEP, |_| ok(&parsed(config)))
        .unwrap_or_else(|e| panic!("{e}\n{}\n{}", stored(config), term.text()));
}

/// The runs of the first sequence's steps.
fn runs(doc: &toml::Table) -> Vec<String> {
    doc.get("sequences")
        .and_then(|s| s.get(0))
        .and_then(|s| s.get("steps"))
        .and_then(|s| s.as_array())
        .map(|steps| {
            steps
                .iter()
                .filter_map(|s| s.get("run").and_then(|r| r.as_str()).map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

/// Steps down the sidebar with `Down` until the Sequences page is in front,
/// then hands the keyboard to its list.
fn open_sequences_by_keyboard(term: &mut Terminal) {
    term.send_keys("C-,").unwrap();
    term.wait_for("Settings open", STEP, |s| {
        s.contains("Repo-specific settings")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    for _ in 0..30 {
        if term.text().contains("Sequences - global") {
            break;
        }
        term.send_keys("Down").unwrap();
        std::thread::sleep(Duration::from_millis(120));
    }
    term.wait_for("the Sequences page", STEP, |s| {
        s.contains("Sequences - global")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
}

#[test]
fn the_backup_preset_is_added_reordered_paused_and_bound_by_keyboard_alone() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), config.path(), "pty-sequence-settings-kb");
    open_sequences_by_keyboard(&mut term);
    term.wait_for("an empty list", STEP, |s| s.contains("No sequences yet"))
        .unwrap_or_else(|e| panic!("{e}"));

    // Add > From preset > Backup.
    term.send_keys("Insert").unwrap();
    term.wait_for("the Add menu", STEP, |s| s.contains("Empty sequence"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Down").unwrap();
    term.send_keys("Enter").unwrap();
    wait_stored(&term, config.path(), "the Backup preset", |d| {
        runs(d) == ["stage_all", "ai_generate_commit_message", "commit", "push"]
    });
    let doc = parsed(config.path());
    let seq = &doc["sequences"][0];
    assert_eq!(seq["id"].as_str(), Some("backup"));
    assert_eq!(seq["scope"].as_str(), Some("installation"));
    assert!(seq.get("owner").is_some(), "this installation owns it");

    // To the steps: Enter on the list reaches Name, four Downs the Shortcut
    // row, one more the buttons row; Enter presses Edit steps.
    term.send_keys("Enter").unwrap();
    for _ in 0..5 {
        term.send_keys("Down").unwrap();
    }
    term.send_keys("Enter").unwrap();
    term.wait_for("the steps face", STEP, |s| s.contains("Steps of Backup"))
        .unwrap_or_else(|e| panic!("{e}"));

    // Reorder: select Push (3 Downs) and move it up one place.
    for _ in 0..3 {
        term.send_keys("Down").unwrap();
    }
    term.send_keys("M-Up").unwrap();
    wait_stored(&term, config.path(), "push before commit", |d| {
        runs(d) == ["stage_all", "ai_generate_commit_message", "push", "commit"]
    });

    // Never on the AI step: select it (Up), Enter reaches Pause, Enter opens
    // the dropdown, Down twice picks Never.
    term.send_keys("Up").unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("the Pause row", STEP, |s| s.contains("Pause"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.send_keys("Down").unwrap();
    term.send_keys("Down").unwrap();
    term.send_keys("Enter").unwrap();
    wait_stored(&term, config.path(), "pause never on the AI step", |d| {
        d["sequences"][0]["steps"][1]
            .get("pause")
            .and_then(|p| p.as_str())
            == Some("never")
    });

    // Esc goes back to the sequences; the Shortcut button opens the capture.
    term.send_keys("Esc").unwrap();
    term.wait_for("back on the sequences", STEP, |s| {
        s.contains("Editing sequence 1/1")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    for _ in 0..4 {
        term.send_keys("Down").unwrap();
    }
    term.send_keys("Enter").unwrap();
    term.wait_for("the capture popup", STEP, |s| {
        s.contains("Press the new key")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("M-b").unwrap();
    term.wait_for("the captured key", STEP, |s| s.contains("Alt+B"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    wait_stored(
        &term,
        config.path(),
        "the key in [keybindings.sequence]",
        |d| {
            d.get("keybindings")
                .and_then(|k| k.get("sequence"))
                .and_then(|k| k.get("backup"))
                .is_some_and(|v| v.to_string().contains("Alt+B"))
        },
    );

    // The palette lists the row with its key (Settings closes first).
    term.send_keys("Esc").unwrap();
    term.send_keys("Esc").unwrap();
    term.send_keys("Esc").unwrap();
    term.wait_for("the main window", STEP, |s| s.contains("Message"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-p").unwrap();
    term.send_text("Sequence: Backup").unwrap();
    term.wait_for("the row with its key", STEP, |s| {
        s.contains("Sequence: Backup") && s.contains("Alt+B")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

#[test]
fn a_step_is_added_through_the_picker_and_both_crosses_close_by_mouse() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), config.path(), "pty-sequence-settings-mouse");
    let (c, r) = term
        .wait_for_text("\u{2699}", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(c, r).expect("click the gear");
    term.wait_for("Settings open", STEP, |s| {
        s.contains("Repo-specific settings")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    common::settings_page(&mut term, "Sequences");

    // + Add > Empty sequence.
    let (c, r) = term
        .wait_for_text("+ Add", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(c + 1, r).expect("click + Add");
    let (c, r) = term
        .wait_for_text("Empty sequence", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(c + 1, r).expect("click Empty sequence");
    wait_stored(&term, config.path(), "the empty sequence", |d| {
        d.get("sequences").is_some()
    });

    // Edit steps (button), then + Add opens the picker.
    let (c, r) = term
        .wait_for_text("Edit steps", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(c + 1, r).expect("click Edit steps");
    term.wait_for("the steps face", STEP, |s| {
        s.contains("Steps of New sequence")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = term
        .wait_for_text("+ Add", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(c + 1, r).expect("click + Add");
    term.wait_for("the step picker", STEP, |s| s.contains("Add a step"))
        .unwrap_or_else(|e| panic!("{e}"));

    // The picker's own cross closes it and adds nothing.
    let screen = term
        .wait_for("the picker title", STEP, |s| s.contains("Add a step"))
        .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = screen.find_all("Add a step")[0];
    let cross = screen
        .find_all("\u{2715}")
        .into_iter()
        .find(|(cc, rr)| *rr == r && *cc > c)
        .expect("the picker's cross on its title row");
    term.click(cross.0, cross.1).expect("click the cross");
    term.wait_for("the picker closed", STEP, |s| !s.contains("Add a step"))
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(runs(&parsed(config.path())).is_empty());

    // Open it again, type, click the Fetch row.
    let (c, r) = term
        .wait_for_text("+ Add", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(c + 1, r).expect("click + Add");
    term.wait_for("the step picker", STEP, |s| s.contains("Add a step"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_text("fetch").unwrap();
    // The list is re-filtered per keystroke: let the last one land before
    // reading where the row is.
    std::thread::sleep(Duration::from_millis(600));
    let screen = term
        .wait_for("the Fetch row", STEP, |s| {
            s.find_all("Fetch").iter().any(|(c, _)| *c > 30)
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (c, r) = screen
        .find_all("Fetch")
        .into_iter()
        .find(|(c, r)| *c > 30 && *r > 5)
        .expect("a Fetch row");
    term.click(c + 1, r).expect("click the Fetch row");
    wait_stored(&term, config.path(), "the fetch step", |d| {
        runs(d) == ["fetch"]
    });

    // The Back button (a cross and its label) returns to the sequences.
    let (c, r) = term
        .wait_for_text("Back to sequences", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(c + 1, r).expect("click Back");
    term.wait_for("the sequences face", STEP, |s| {
        s.contains("Editing sequence 1/1")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}
