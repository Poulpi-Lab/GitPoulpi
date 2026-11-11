//! Settings > Open in on the master-detail list: add a preset from the
//! `[+ Add]` dropdown, drag it to the top (the default star follows), remove
//! it behind the confirmation; `config.toml` follows every step. Temp config dir only; the
//! entries are never run.

#![cfg(unix)]

mod common;

use std::path::Path;
use std::time::{Duration, Instant};

use gitpoulpi_testkit::FixtureRepoBuilder;

const STEP: Duration = Duration::from_secs(30);

fn labels(config_dir: &Path) -> Vec<String> {
    gitpoulpi_config::Config::load(&config_dir.join("config.toml"))
        .ok()
        .and_then(|c| c.ui.open_in)
        .unwrap_or_default()
        .into_iter()
        .map(|e| e.label)
        .collect()
}

fn wait_labels(config_dir: &Path, what: &str, ok: impl Fn(&[String]) -> bool) -> Vec<String> {
    let deadline = Instant::now() + STEP;
    loop {
        let now = labels(config_dir);
        if ok(&now) {
            return now;
        }
        assert!(Instant::now() < deadline, "{what}: {now:?}");
        std::thread::sleep(Duration::from_millis(150));
    }
}

fn press(term: &mut xtask::pty::Terminal, key: &str, times: usize) {
    for _ in 0..times {
        term.send_keys(key).unwrap();
    }
}

/// Opens Settings on the Open in page, mouse enabled.
fn open_page(term: &mut xtask::pty::Terminal) {
    let (gc, gr) = term
        .wait_for_text("\u{2699}", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(gc, gr).unwrap();
    term.wait_for("Settings open", STEP, |s| {
        s.contains("Repo-specific settings")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    common::settings_page(term, "Open in");
    common::settings_tab(term, "Entries");
    term.wait_for("the Open in list", STEP, |s| {
        s.contains("+ Add") && s.contains("Alpha")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

/// The list rows (left pane) showing `label`, top to bottom.
fn list_rows(term: &xtask::pty::Terminal, label: &str) -> Vec<(u16, u16)> {
    let mut rows: Vec<(u16, u16)> = term
        .screen()
        .find_all(label)
        .into_iter()
        .filter(|(c, _)| *c < 70)
        .collect();
    rows.sort_by_key(|&(_, r)| r);
    rows
}

#[test]
fn a_preset_is_added_from_the_dropdown_dragged_to_the_top_starred_and_removed() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("open in settings probe")
        .unwrap()
        .build();
    let (mut term, config) = common::launch(repo.root())
        .session("settings-open-in")
        .config(
            "[[ui.open_in]]\nlabel = \"Alpha\"\ncommand = \"true ${cwd}\"\n\n\
             [[ui.open_in]]\nlabel = \"Beta\"\ncommand = \"true ${cwd}\"\n",
        )
        .size(200, 60)
        .mouse_capture()
        .ready("open in settings probe")
        .start();
    let dir = config.path();
    open_page(&mut term);

    // The first entry is the default: its row carries the star.
    let (_, alpha_row) = list_rows(&term, "Alpha")[0];
    assert!(term.screen().line(alpha_row).contains('\u{2605}'));

    // [+ Add] opens the dropdown; typing narrows it, Enter adds the preset.
    let (c, r) = term
        .wait_for_text("+ Add", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(c + 1, r).unwrap();
    term.send_text("gitk").unwrap();
    term.wait_for("the preset in the dropdown", STEP, |s| s.contains("gitk"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    let after_add = wait_labels(dir, "a preset was appended", |l| l.len() == 3);
    assert_eq!(&after_add[..2], ["Alpha", "Beta"]);
    let added = after_add[2].clone();

    // Drag it onto the first row: it becomes the first entry and the default.
    std::thread::sleep(Duration::from_millis(900));
    term.wait_for("the new row", STEP, |_| {
        !list_rows(&term, &added).is_empty()
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let from = *list_rows(&term, &added).last().unwrap();
    let to = list_rows(&term, "Alpha")[0];
    term.drag((from.0 + 1, from.1), (to.0 + 1, to.1)).unwrap();
    wait_labels(dir, "dragged to the top", |l| {
        l.first() == Some(&added) && l.len() == 3
    });
    term.wait_for("the star on the new first row", STEP, |s| {
        list_rows(&term, &added)
            .first()
            .is_some_and(|&(_, r)| s.line(r).contains('\u{2605}'))
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (_, alpha_row) = list_rows(&term, "Alpha")[0];
    assert!(
        !term.screen().line(alpha_row).contains('\u{2605}'),
        "Alpha lost the star:\n{}",
        term.text()
    );

    // It is the selected row: Delete asks, Remove confirms.
    term.send_keys("Delete").unwrap();
    let screen = term
        .wait_for("the removal question with its buttons", STEP, |s| {
            s.contains("Remove \"")
                && s.find("Cancel")
                    .is_some_and(|(_, row)| s.find_all("Remove").into_iter().any(|(_, r)| r == row))
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (_, row) = screen.find("Cancel").unwrap();
    let (col, _) = screen
        .find_all("Remove")
        .into_iter()
        .find(|&(_, r)| r == row)
        .expect("a Remove button beside Cancel");
    term.click(col + 1, row).unwrap();
    wait_labels(dir, "removed", |l| l == ["Alpha", "Beta"]);
    let _ = term.stop();
}

/// The `[+ Add]` dropdown names a category on every row and its search line
/// narrows to one (another OS's rows are covered by the unit tests).
#[test]
fn the_add_preset_picker_groups_presets_by_category() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("open in categories probe")
        .unwrap()
        .build();
    let (mut term, _config) = common::launch(repo.root())
        .session("settings-open-in-categories")
        .config("[[ui.open_in]]\nlabel = \"Alpha\"\ncommand = \"true ${cwd}\"\n")
        .size(200, 60)
        .mouse_capture()
        .ready("open in categories probe")
        .start();
    let (gc, gr) = term
        .wait_for_text("\u{2699}", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(gc, gr).unwrap();
    term.wait_for("Settings open", STEP, |s| {
        s.contains("Repo-specific settings")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    common::settings_page(&mut term, "Open in");
    common::settings_tab(&mut term, "Entries");
    term.wait_for("the Open in page", STEP, |s| s.contains("+ Add"))
        .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = term
        .wait_for_text("+ Add", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(c + 1, r).unwrap();
    term.send_text("Git tool: gitk").unwrap();
    term.wait_for("the Git tool category", STEP, |s| {
        s.contains("Git tool: gitk")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    press(&mut term, "BSpace", "Git tool: gitk".len());
    term.send_text("Web: Repository").unwrap();
    term.wait_for("the Web category", STEP, |s| {
        s.contains("Web: Repository page on the web")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let _ = term.stop();
}
