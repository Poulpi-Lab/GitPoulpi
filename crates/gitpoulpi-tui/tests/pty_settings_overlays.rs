//! The command palette, the theme picker, the workspace search and the About
//! window open over the Settings screen and close back to it with its state
//! intact (an unsaved Workflow draft, the selected section). The title line
//! wears the app's octopus and name, like the main window's. Temp config dir
//! only.

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::{Color, Screen, Terminal};

const STEP: Duration = Duration::from_secs(30);

const WORKFLOW: &str = r#"template = "custom"

[[base]]
name = "main"

[[topic]]
id = "feature"
label = "Feature"
pattern = "feature/{slug}"
parent = "main"
"#;

fn fixture() -> FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial commit")
        .unwrap()
        .build();
    std::fs::create_dir_all(repo.state_dir()).unwrap();
    std::fs::write(repo.state_dir().join("workflow.toml"), WORKFLOW).unwrap();
    repo
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

/// Settings on the Workflow page with an unsaved draft (a topic type added).
fn settings_with_draft(term: &mut Terminal) {
    let (c, r) = term
        .wait_for_text("\u{2699}", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(c, r).expect("click the gear");
    term.wait_for("Settings open", STEP, |s| {
        s.contains("Repo-specific settings")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    common::settings_page(term, "Workflow");
    common::settings_tab(term, "Rules");
    term.wait_for("the save bar", STEP, |s| s.contains("No changes"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (c, r) = term
        .wait_for_text("+ Add", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(c + 1, r).unwrap();
    term.wait_for("the add menu", STEP, |s| s.contains("Finish rule"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (c, r) = term
        .screen()
        .find_all("Topic type")
        .into_iter()
        .find(|&(_, row)| row > 10)
        .expect("the Topic type entry");
    term.click(c + 1, r).unwrap();
    term.wait_for("the dirty mark", STEP, |s| s.contains("Unsaved"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

/// Settings is back, with the draft and the page it was left on.
fn settings_intact(term: &Terminal, what: &str) {
    term.wait_for(&format!("Settings intact after {what}"), STEP, |s| {
        s.contains("Unsaved") && s.contains("Workflow - ") && s.contains("Settings - Tab")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

fn title_bg(screen: &Screen) -> Color {
    let (col, row) = screen.find("Settings - Tab").expect("the Settings title");
    screen.cell(col, row).expect("a cell").bg
}

#[test]
fn the_settings_title_line_wears_the_octopus_and_the_app_name() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), config.path(), "pty-ovl-title");
    settings_with_draft(&mut term);
    let screen = term.screen();
    let (col, row) = screen
        .find("GitPoulpi - Settings - Tab")
        .expect("the title");
    assert_eq!(row, 0, "on the top border");
    let (octopus, orow) = screen.find("\u{1F419}").expect("the octopus");
    assert_eq!(orow, 0);
    assert!(octopus < col, "the octopus comes before the name");
    // Clicking it opens About, like the main window's.
    term.click(octopus, 0).unwrap();
    term.wait_for("About over Settings", STEP, |s| {
        s.contains("About GitPoulpi") && s.contains("Settings - Tab")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Escape").unwrap();
    term.wait_for_text_to_go("About GitPoulpi", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    settings_intact(&term, "About by mouse");
    let _ = term.stop();
}

#[test]
fn the_palette_opens_over_settings_greys_what_leaves_it_and_closes_back() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), config.path(), "pty-ovl-palette");
    settings_with_draft(&mut term);
    term.send_keys("C-p").unwrap();
    term.send_text("change theme").unwrap();
    term.wait_for("the palette over Settings", STEP, |s| {
        s.contains("Change Theme") && s.contains("Settings - Tab")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // A command that acts on the panes behind Settings is refused.
    for _ in 0.."change theme".len() {
        term.send_keys("Backspace").unwrap();
    }
    term.send_text("toggle word wrap").unwrap();
    term.wait_for("the reason", STEP, |s| s.contains("close Settings first"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Escape").unwrap();
    term.wait_for_text_to_go("close Settings first", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    settings_intact(&term, "the palette");

    // A click outside closes it too.
    term.send_keys("C-p").unwrap();
    term.wait_for("the palette", STEP, |s| s.contains("Change Theme"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.click(1, 58).unwrap();
    term.wait_for_text_to_go("Change Theme", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    settings_intact(&term, "a click outside the palette");
    let _ = term.stop();
}

#[test]
fn a_palette_command_that_works_over_settings_runs_without_closing_it() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), config.path(), "pty-ovl-palette-run");
    settings_with_draft(&mut term);
    term.send_keys("C-p").unwrap();
    term.send_text("about gitpoulpi").unwrap();
    term.wait_for("the About command", STEP, |s| s.contains("About GitPoulpi"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    term.wait_for("About over Settings", STEP, |s| {
        s.contains("Version") && s.contains("Settings - Tab")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Escape").unwrap();
    term.wait_for_text_to_go("Version", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    settings_intact(&term, "About from the palette");
    let _ = term.stop();
}

#[test]
fn the_theme_picker_previews_on_settings_and_cancelling_restores_it() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), config.path(), "pty-ovl-theme");
    settings_with_draft(&mut term);
    let before = title_bg(&term.screen());
    term.send_keys("C-n").unwrap();
    term.send_text("y").unwrap();
    term.wait_for("the theme list over Settings", STEP, |s| {
        s.contains("System (follows terminal)") && s.contains("Settings - Tab")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_text("light").unwrap();
    let screen = term
        .wait_for("Settings repainted in the previewed theme", STEP, |s| {
            s.find("Settings - Tab").is_some() && title_bg(s) != before
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_ne!(title_bg(&screen), before);
    term.send_keys("Escape").unwrap();
    term.wait_for("the colours are back", STEP, |s| {
        !s.contains("System (follows terminal)") && title_bg(s) == before
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    settings_intact(&term, "the theme picker");
    let _ = term.stop();
}

#[test]
fn the_workspace_search_opens_over_settings() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), config.path(), "pty-ovl-search");
    settings_with_draft(&mut term);
    term.send_keys("M-C-f").unwrap();
    term.wait_for("the search over Settings", STEP, |s| {
        s.contains("Search open repositories") && s.contains("Settings - Tab")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Escape").unwrap();
    term.wait_for_text_to_go("Search open repositories", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    settings_intact(&term, "the workspace search");
    let _ = term.stop();
}

#[test]
fn about_opens_over_settings_by_key_and_a_chord_that_leaves_settings_is_refused() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), config.path(), "pty-ovl-about");
    settings_with_draft(&mut term);
    term.send_keys("C-n").unwrap();
    term.send_text("a").unwrap();
    term.wait_for("About over Settings", STEP, |s| {
        s.contains("About GitPoulpi") && s.contains("Settings - Tab")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Escape").unwrap();
    term.wait_for_text_to_go("About GitPoulpi", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    settings_intact(&term, "About");
    term.send_keys("C-n").unwrap();
    term.send_text("t").unwrap();
    term.wait_for("the refusal", STEP, |s| s.contains("close Settings first"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    settings_intact(&term, "a refused chord");
    let _ = term.stop();
}

#[test]
fn the_palette_and_the_picker_open_over_settings_opened_from_the_home_tab() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), config.path(), "pty-ovl-home");
    term.send_keys("C-o").expect("go Home");
    term.wait_for("Home", STEP, |s| s.contains("Repository Management"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-n").unwrap();
    term.send_text(",").unwrap();
    term.wait_for("Settings over Home", STEP, |s| {
        s.contains("Settings - Tab") && s.contains("Search settings")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("C-p").unwrap();
    term.wait_for("the palette over Settings", STEP, |s| {
        s.contains("Change Theme") && s.contains("Settings - Tab")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Escape").unwrap();
    term.wait_for_text_to_go("Change Theme", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-n").unwrap();
    term.send_text("y").unwrap();
    term.wait_for("the theme list over Settings", STEP, |s| {
        s.contains("System (follows terminal)") && s.contains("Settings - Tab")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Escape").unwrap();
    term.wait_for("back in Settings", STEP, |s| {
        !s.contains("System (follows terminal)") && s.contains("Search settings")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let _ = term.stop();
}
