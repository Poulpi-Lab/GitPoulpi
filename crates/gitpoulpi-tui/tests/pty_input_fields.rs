//! Text inputs that look like inputs, a commit Description that scrolls,
//! and the Clone dialog's editable folder name -- end to end through a
//! pty (`xtask::pty`): the real binary, the real daemon, real `git`.
//!
//! The rules are `pty_ui.rs`'s: nothing is located by a hard-coded
//! coordinate, nothing sleeps, every wait describes the whole state it is
//! waiting for, and every config dir and repository is temporary.

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::{Color, Terminal};

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);

/// `theme.rs`'s `INPUT_BG` and the scrollbar thumb color of `scrollbar.rs`.
const INPUT_BG: Color = Color::Rgb(42, 42, 46);
const BLUE_BG: Color = Color::Rgb(28, 34, 52);
const THUMB: Color = Color::Rgb(90, 96, 130);

fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial commit")
        .unwrap()
        .write_worktree("a.txt", b"one\ntwo\n")
        .unwrap()
        .stage("a.txt")
        .unwrap()
        .build()
}

fn launch(
    repo: &Path,
    session: &str,
    config_toml: &str,
    ready: &str,
) -> (Terminal, tempfile::TempDir) {
    let config = tempfile::tempdir().expect("temp config dir");
    if !config_toml.is_empty() {
        std::fs::write(config.path().join("config.toml"), config_toml).unwrap();
    }
    let term = common::launch(repo)
        .session(session)
        .config_dir(config.path())
        .size(200, 50)
        .spawn();
    let ready = ready.to_string();
    term.wait_for("the first window", STARTUP, move |s| s.contains(&ready))
        .unwrap_or_else(|e| panic!("{e}"));
    (term, config)
}

/// Forward slashes: typing a literal `\` through ConPTY's synthetic input
/// does not reach the app (see `pty_ui.rs`), and Windows accepts `/`.
fn typed(path: &Path) -> String {
    path.display().to_string().replace('\\', "/")
}

/// The Refs sidebar's Filter box is an input: its hint text and the rest
/// of its width are painted on the input background.
#[test]
fn the_filter_input_is_painted_on_the_input_background() {
    let repo = fixture();
    let (term, _config) = launch(repo.root(), "pty-input-bg", "", "initial commit");
    let screen = term
        .wait_for("the Refs filter box with its hint", STEP, |s| {
            s.contains("(click or press / to filter)")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (col, row) = screen.find("(click or press / to filter)").unwrap();
    let hint_w = "(click or press / to filter)".chars().count() as u16;
    for c in [col, col + hint_w / 2, col + hint_w - 1, col + hint_w + 1] {
        let cell = screen.cell(c, row).unwrap();
        assert_eq!(cell.bg, INPUT_BG, "cell ({c}, {row}) {:?}", cell.text);
    }
}

/// A 30-line Description overflows its box: the standard scrollbar shows a
/// thumb, the caret stays visible while typing, and dragging the thumb to
/// the top brings the first line back.
#[test]
fn a_long_description_shows_a_scrollbar_thumb_that_drags() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-desc-scrollbar", "", "// WIP");
    term.send_keys("c").expect("press c in the Log");
    term.wait_for("the Summary field focused", STEP, |s| {
        s.contains("Staged Files") && !s.contains("// WIP")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_text("the subject").expect("type the summary");
    term.send_keys("Enter").expect("on to the Description");
    for n in 1..=30 {
        term.send_text(&format!("dsc{n:02}")).expect("type a line");
        if n < 30 {
            term.send_keys("Enter").expect("new line");
        }
    }
    term.wait_for(
        "the last line with the caret, the first scrolled off",
        STEP,
        |s| s.contains("dsc30_") && !s.contains("dsc01"),
    )
    .unwrap_or_else(|e| panic!("{e}"));
    // Settled: the thumb and the arrows are read off a whole frame.
    let screen = snap(&term);
    let (text_col, _) = screen.find("dsc30_").unwrap();
    let desc_rows: Vec<u16> = (0..screen.rows())
        .filter(|r| screen.line(*r).contains("dsc"))
        .collect();
    assert!(desc_rows.len() >= 2, "the description rows: {desc_rows:?}");
    let thumb: Vec<(u16, u16)> = desc_rows
        .iter()
        .flat_map(|r| (text_col..screen.cols()).map(move |c| (c, *r)))
        .filter(|(c, r)| screen.cell(*c, *r).is_some_and(|x| x.bg == THUMB))
        .collect();
    assert!(!thumb.is_empty(), "no scrollbar thumb beside the text");
    let bar_col = thumb[0].0;
    // At the end of the text the thumb is at the bottom of its track,
    // just above the down arrow (the last row).
    let last = *desc_rows.last().unwrap();
    assert!(thumb.contains(&(bar_col, last - 1)), "{thumb:?}");
    assert_eq!(
        screen
            .cell(bar_col, last)
            .map(|c| c.text.clone())
            .as_deref(),
        Some("\u{2193}")
    );

    // The wheel scrolls it...
    term.scroll(false, text_col, desc_rows[0])
        .expect("wheel up over the description");
    term.wait_for("the view scrolled up", STEP, |s| {
        s.contains("dsc2") && !s.contains("dsc30")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    // The end arrows scroll one row: ↓ moves the window down a line, ↑
    // brings it back.
    let first_row = |t: &xtask::pty::Terminal| {
        let s = t.screen();
        let line = s.line(desc_rows[0]);
        line.split("dsc").nth(1).map(|x| x[..2].to_string())
    };
    let top = first_row(&term).expect("a description row");
    term.click(bar_col, last).expect("click the down arrow");
    term.wait_for("the window one line lower", STEP, |s| {
        let line = s.line(desc_rows[0]);
        line.split("dsc")
            .nth(1)
            .is_some_and(|x| x[..2].parse::<u32>().ok() == top.parse::<u32>().ok().map(|n| n + 1))
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.click(bar_col, desc_rows[0])
        .expect("click the up arrow");
    term.wait_for("the window back", STEP, |s| {
        s.line(desc_rows[0])
            .split("dsc")
            .nth(1)
            .is_some_and(|x| x[..2] == top)
    })
    .unwrap_or_else(|e| panic!("{e}"));
    // ...and dragging the thumb to the top of the track reaches line 1.
    let screen = snap(&term);
    let thumb_row = desc_rows
        .iter()
        .copied()
        .find(|r| screen.cell(bar_col, *r).is_some_and(|x| x.bg == THUMB))
        .expect("a thumb after the wheel");
    term.drag((bar_col, thumb_row), (bar_col, desc_rows[0]))
        .expect("drag the thumb to the top");
    term.wait_for("the first line back in view", STEP, |s| s.contains("dsc01"))
        .unwrap_or_else(|e| panic!("{e}"));
}

/// Clone with URL: the folder name is prefilled from the URL, editing it
/// changes the Full path preview, and the clone lands in the edited folder.
#[test]
fn clone_into_an_edited_folder_name() {
    let source = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("s.txt", "s\n", "clone-source-only-commit")
        .unwrap()
        .build();
    let start = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "clone-name-start-commit")
        .unwrap()
        .build();
    let root = tempfile::tempdir().unwrap();
    let (mut term, _config) = launch(
        start.root(),
        "pty-clone-folder-name",
        &format!("[repos]\nclone_root = '{}'\n", root.path().display()),
        "clone-name-start-commit",
    );
    term.send_keys("C-o").expect("go to the management tab");
    term.wait_for("the management tab", STEP, |s| {
        s.contains("Repository Management") && s.contains("Clone")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("c").expect("Clone");
    term.wait_for("the Clone dialog", STEP, |s| {
        s.contains("Clone a repository") && s.contains("Folder name")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    term.send_text(&typed(source.root())).expect("type the URL");
    let source_name = source
        .root()
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    let expect = source_name.clone();
    term.wait_for("the preview with the URL's name", STEP, move |s| {
        s.contains("Full path:") && s.contains(&expect) && !s.contains("enter the URL")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // Tab to the folder name, replace it.
    term.send_keys("Tab").expect("to the folder name");
    term.send_keys("C-u").expect("clear it");
    term.send_text("edited-clone").expect("type the new name");
    term.wait_for("the preview ends with the edited name", STEP, |s| {
        s.find("Full path:").is_some_and(|(_, row)| {
            let line = s.line(row);
            line.contains("edited-clone") && !line.contains(&source_name)
        })
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").expect("clone it");
    term.wait_for("the clone open as a tab", STARTUP, |s| {
        s.contains("clone-source-only-commit") && !s.contains("Clone a repository")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        root.path().join("edited-clone").join(".git").is_dir(),
        "no clone in the edited folder"
    );
    assert!(
        !root.path().join(&source_name).exists(),
        "the URL's own name must not be used"
    );
}

/// Settings > Appearance > Input color: picking Blue from the
/// dropdown repaints the inputs on the next frame (no restart) and lands
/// in config.toml; a custom color in the config file is honoured too.
#[test]
fn the_input_color_setting_applies_live_and_a_custom_hex_is_honoured() {
    let repo = fixture();
    let (mut term, config) = launch(repo.root(), "pty-input-color", "", "initial commit");
    let hint = "(click or press / to filter)";
    let filter_bg = |term: &Terminal| {
        let screen = snap(term);
        let (col, row) = screen.find(hint).expect("the Refs filter hint");
        screen.cell(col + 1, row).unwrap().bg
    };
    term.wait_for("the filter hint", STEP, |s| s.contains(hint))
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(filter_bg(&term), INPUT_BG, "the theme's gray by default");

    let (gear_col, gear_row) = term.wait_for_text("\u{2699}", STEP).unwrap();
    term.click(gear_col, gear_row).expect("click the gear");
    term.wait_for("Settings open", STEP, |s| {
        s.contains("Repo-specific settings")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    common::settings_page(&mut term, "Appearance");
    // The page is taller than the window: wheel down over it until the
    // Input color row is on screen.
    let (body_col, body_row) = term
        .wait_for_text("Appearance - global", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    let deadline = std::time::Instant::now() + STEP;
    while common::choice_at(&snap(&term), "Follow theme").is_none() {
        assert!(
            std::time::Instant::now() < deadline,
            "the Input color row never scrolled into view:\n{}",
            term.text()
        );
        term.scroll(true, body_col + 10, body_row + 10).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(150));
    }
    // Retried: a click aimed at a torn frame opens nothing.
    let screen = common::retry_until(
        &mut term,
        "the popup",
        4,
        Duration::from_secs(10),
        |t| {
            let (col, row) = common::wait_for_choice(t, "Follow theme", STEP);
            t.click(col + 1, row).expect("open the combo");
        },
        |s| s.contains("\u{2713} Follow theme"),
    );
    let (tick_col, tick_row) = screen.find("\u{2713} Follow theme").unwrap();
    // Follow theme (ticked), Gray, Blue.
    term.click(tick_col + 2, tick_row + 2).expect("pick Blue");
    let path = config.path().join("config.toml");
    let deadline = std::time::Instant::now() + STEP;
    while !std::fs::read_to_string(&path)
        .unwrap_or_default()
        .contains("input_color = \"blue\"")
    {
        assert!(std::time::Instant::now() < deadline, "never stored");
        std::thread::sleep(Duration::from_millis(200));
    }
    term.wait_for("the combo shows Blue", STEP, |s| {
        common::choice_at(s, "Blue").is_some() && !s.contains("\u{2713}")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Escape").unwrap();
    term.wait_for("back on the Log", STEP, |s| s.contains(hint))
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(filter_bg(&term), BLUE_BG, "applied live, no restart");
    drop(term);

    // A custom hex from the file.
    let (term, _config) = launch(
        repo.root(),
        "pty-input-color-custom",
        "[ui]\ninput_color = \"custom\"\ninput_color_custom = \"#102030\"\n",
        "initial commit",
    );
    term.wait_for("the filter hint", STEP, |s| s.contains(hint))
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(filter_bg(&term), Color::Rgb(16, 32, 48));
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
