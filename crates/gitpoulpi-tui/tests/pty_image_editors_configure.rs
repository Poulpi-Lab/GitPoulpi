//! Settings > Images: toggle the viewer, add a preset, edit its command,
//! reorder and remove entries, all by keyboard, and `config.toml` follows every
//! step. Temp config dir only; the entries use the harmless `true` program and
//! no real editor is ever launched.
//!
//! Not covered here (needs slice S4, the File View image mode): with the
//! viewer checkbox off the File View shows the old binary notice again. Until
//! then only the `ui.image_view` key flipping is checked.

#![cfg(unix)]

mod common;

use std::path::Path;
use std::time::{Duration, Instant};

use gitpoulpi_testkit::FixtureRepoBuilder;

const STEP: Duration = Duration::from_secs(30);

fn config_of(dir: &Path) -> gitpoulpi_config::Config {
    gitpoulpi_config::Config::load(&dir.join("config.toml")).unwrap()
}

fn entries(dir: &Path) -> Vec<(String, String)> {
    config_of(dir)
        .ui
        .image_editors
        .unwrap_or_default()
        .into_iter()
        .map(|e| (e.label, e.command))
        .collect()
}

fn wait<T>(what: &str, probe: impl Fn() -> Option<T>) -> T {
    let deadline = Instant::now() + STEP;
    loop {
        if let Some(found) = probe() {
            return found;
        }
        assert!(Instant::now() < deadline, "timed out: {what}");
        std::thread::sleep(Duration::from_millis(150));
    }
}

fn wait_labels(dir: &Path, what: &str, ok: impl Fn(&[String]) -> bool) -> Vec<(String, String)> {
    wait(what, || {
        let now = entries(dir);
        let labels: Vec<String> = now.iter().map(|(l, _)| l.clone()).collect();
        ok(&labels).then_some(now)
    })
}

fn press(term: &mut xtask::pty::Terminal, key: &str, times: usize) {
    for _ in 0..times {
        term.send_keys(key).unwrap();
    }
}

#[test]
fn the_images_page_edits_the_viewer_keys_and_the_editor_list() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("image settings probe")
        .unwrap()
        .build();
    let (mut term, config) = common::launch(repo.root())
        .session("settings-images")
        .config(
            "[[ui.image_editors]]\nlabel = \"Alpha\"\ncommand = \"true ${file}\"\n\n\
             [[ui.image_editors]]\nlabel = \"Beta\"\ncommand = \"true ${file}\"\n",
        )
        .size(200, 60)
        .mouse_capture()
        .ready("image settings probe")
        .start();
    let dir = config.path();

    let (gc, gr) = term
        .wait_for_text("\u{2699}", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(gc, gr).unwrap();
    term.wait_for("Settings open", STEP, |s| {
        s.contains("Repo-specific settings")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    common::settings_page(&mut term, "Images");
    term.wait_for("the Images page", STEP, |s| {
        s.contains("Image editors") && s.contains("Preview images")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // The viewer checkbox is a live key (click its row: the list has the
    // keyboard on entering the page).
    assert!(config_of(dir).ui.image_view, "on by default");
    let toggle = |term: &mut xtask::pty::Terminal| {
        let (c, r) = snap(&term).find("Preview images").unwrap();
        term.click(c + 1, r).unwrap();
    };
    toggle(&mut term);
    wait("image_view off", || {
        (!config_of(dir).ui.image_view).then_some(())
    });
    toggle(&mut term);
    wait("image_view on again", || {
        config_of(dir).ui.image_view.then_some(())
    });

    // The list is on its own tab; `[+ Add]` opens the dropdown of presets and
    // Enter takes the first one.
    common::settings_tab(&mut term, "Editors");
    let (c, r) = snap(&term).find("+ Add").unwrap();
    term.click(c + 1, r).unwrap();
    term.wait_for("the dropdown", STEP, |s| s.contains("Add custom"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    let after_add = wait_labels(dir, "a preset was appended", |l| l.len() == 3);
    assert_eq!(after_add[0].0, "Alpha");
    assert_eq!(after_add[1].0, "Beta");

    // The new entry is selected: Enter moves into its rows (Label first).
    term.send_keys("Enter").unwrap();
    press(&mut term, "Down", 1);
    term.send_keys("Enter").unwrap(); // edit Command
    term.send_keys("C-a").unwrap();
    term.send_text("true --edited ${file}").unwrap();
    term.send_keys("Enter").unwrap();
    wait("the command was stored", || {
        (entries(dir).get(2).map(|e| e.1.as_str()) == Some("true --edited ${file}")).then_some(())
    });
    press(&mut term, "Up", 1);
    term.send_keys("Enter").unwrap(); // edit Label
    term.send_keys("C-a").unwrap();
    term.send_text("Gamma").unwrap();
    term.send_keys("Enter").unwrap();
    wait_labels(dir, "renamed", |l| l == ["Alpha", "Beta", "Gamma"]);

    // Test runs the fake `true` command on a sample image: nothing to assert on
    // disk, the page just keeps answering.
    press(&mut term, "Down", 3);
    term.send_keys("Enter").unwrap();

    // Back on the list: move up, move down, then remove (it asks first).
    let (c, r) = snap(&term)
        .find_all("Gamma")
        .into_iter()
        .find(|(c, _)| *c < 70)
        .expect("the Gamma row");
    term.click(c + 1, r).unwrap();
    term.send_keys("M-Up").unwrap();
    wait_labels(dir, "moved up", |l| l == ["Alpha", "Gamma", "Beta"]);
    term.send_keys("M-Down").unwrap();
    wait_labels(dir, "moved back down", |l| l == ["Alpha", "Beta", "Gamma"]);
    term.send_keys("Delete").unwrap();
    let screen = term
        .wait_for("the removal question", STEP, |s| {
            s.contains("Cancel") && s.contains("Remove \"")
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

/// The screen once it has settled: a torn frame is not a place to locate a
/// cell or compare colors in.
fn snap(term: &xtask::pty::Terminal) -> xtask::pty::Screen {
    term.wait_for_stable(
        std::time::Duration::from_millis(300),
        std::time::Duration::from_secs(30),
    )
    .unwrap_or_else(|e| panic!("{e}"))
}
