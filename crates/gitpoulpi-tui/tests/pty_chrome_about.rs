//! The title bar's octopus and the About window, the toolbar's per-repository
//! profile combobox (two profiles or more), and the confirmation that offers
//! to write the chosen identity into the repository's git config.
//!
//! Temp config dir and fixture repo only: the identity is written into the
//! fixture's own `.git/config`.

#![cfg(unix)]

mod common;

use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);

const PROFILES: &str = "[ui]\ngraph_renderer = \"text\"\n\
    [general]\nactive_profile = \"home\"\n\
    [[profiles]]\nid = \"home\"\nname = \"Home\"\nauthor_name = \"Home Me\"\nauthor_email = \"me@home.test\"\n\
    [[profiles]]\nid = \"work\"\nname = \"Work\"\nauthor_name = \"Work Me\"\nauthor_email = \"me@work.test\"\n";

fn start(
    session: &str,
    config: &str,
) -> (Terminal, tempfile::TempDir, gitpoulpi_testkit::FixtureRepo) {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "first-commit")
        .unwrap()
        .build();
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("config.toml"), config).unwrap();
    let term = common::launch(repo.root())
        .session(session)
        .config_dir(dir.path())
        .size(200, 50)
        .no_exit_pause()
        .ready("first-commit")
        .spawn();
    (term, dir, repo)
}

/// Clicks the octopus until the About window is up, then lets it be drawn
/// whole: the octopus can be drawn before its hit target is registered, and a
/// torn frame hides parts of the window.
fn click_octopus(term: &mut Terminal) {
    common::retry_until(
        term,
        "the About window",
        6,
        Duration::from_secs(10),
        |t| {
            if let Some((col, row)) = t.screen().find("\u{1F419}") {
                t.click(col, row).unwrap();
            }
        },
        |s| s.contains("About GitPoulpi"),
    );
    settled(term);
}

fn settled(term: &Terminal) -> xtask::pty::Screen {
    term.wait_for_stable(Duration::from_millis(300), STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()))
}

fn local_config(repo: &std::path::Path, key: &str) -> String {
    let out = std::process::Command::new("git")
        .args(["config", "--local", "--get", key])
        .current_dir(repo)
        .output()
        .unwrap();
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

fn palette(term: &mut Terminal, query: &str, entry: &str) {
    term.send_keys("C-p").unwrap();
    term.wait_for_text("Command Palette", STEP).unwrap();
    term.send_text(query).unwrap();
    term.wait_for("the entry", STEP, |s| s.contains(entry))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
}

#[test]
fn the_octopus_opens_about_and_the_palette_twin_does_too() {
    let (mut term, _dir, _repo) = start("about-window", "[ui]\ngraph_renderer = \"text\"\n");
    let screen = snap(&term);
    let (col, row) = screen.find("\u{1F419}").expect("the octopus");
    assert_eq!(row, 0, "{}", term.text());
    let (title_col, title_row) = screen.find("GitPoulpi").expect("the title");
    assert_eq!(title_row, 0);
    assert!(col < title_col, "the octopus comes before the title");
    term.click(col, row).unwrap();
    term.wait_for("the About window", STEP, |s| {
        s.contains("About GitPoulpi")
            && s.contains("Version ")
            && s.contains("License: MIT")
            && s.contains("github.com/Poulpi-Lab/GitPoulpi")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Escape").unwrap();
    term.wait_for_text_to_go("License: MIT", STEP).unwrap();
    // The keyboard twin.
    palette(&mut term, "About", "Help: About GitPoulpi");
    term.wait_for_text("License: MIT", STEP).unwrap();
    // A click closes it too.
    term.click(2, 20).unwrap();
    term.wait_for_text_to_go("License: MIT", STEP).unwrap();
    term.stop().unwrap();
}

/// With an image protocol the octopus is the pixel sprite, not the emoji; the
/// same two cells still open About.
#[test]
fn with_an_image_protocol_the_octopus_is_a_picture_that_still_opens_about() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "first-commit")
        .unwrap()
        .build();
    let (mut term, _dir) = common::launch(repo.root())
        .session("about-picture")
        .config("[ui]\ngraph_renderer = \"image\"\nimage_protocol = \"iterm2\"\n")
        .terminal_image()
        .env("GITPOULPI_TERMINAL_CELL_PX", "9x21")
        .size(200, 50)
        .ready_when("the fixture loaded", |s| s.contains("first-commit"))
        .start();
    let screen = snap(&term);
    assert!(
        screen.find("\u{1F419}").is_none(),
        "the emoji is replaced by the picture:\n{}",
        term.text()
    );
    let (title_col, row) = screen.find("GitPoulpi").expect("the title");
    assert_eq!(row, 0);
    assert_eq!(title_col, 4, "the title does not move: {}", term.text());
    term.click(1, 0).unwrap();
    term.wait_for_text("License: MIT", STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

#[test]
fn no_profile_combobox_without_two_profiles() {
    let (mut term, _dir, _repo) = start(
        "profile-none",
        "[ui]\ngraph_renderer = \"text\"\n[[profiles]]\nid = \"home\"\nname = \"Home\"\nauthor_name = \"H\"\nauthor_email = \"h@x.test\"\n",
    );
    assert!(!snap(&term).contains("Default ("), "{}", term.text());
    palette(&mut term, "Change Profile", "Repository: Change Profile");
    term.wait_for_text("add a second profile", STEP).unwrap();
    term.stop().unwrap();
}

#[test]
fn the_profile_combobox_stores_the_choice_and_offers_the_git_config() {
    let (mut term, dir, repo) = start("profile-pick", PROFILES);
    let baseline = local_config(repo.root(), "user.email");
    // Shown at the toolbar's right end, on the default profile.
    let (col, row) = term
        .wait_for_text("Default (Home)", STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (_, toolbar_row) = snap(&term).find("Undo").expect("the toolbar");
    assert_eq!(row, toolbar_row, "on the toolbar row:\n{}", term.text());
    assert!(col > 150, "at the right end: {col}");
    term.click(col + 1, row).unwrap();
    term.wait_for("the list", STEP, |s| s.contains("Work"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (wc, wr) = snap(&term).find("Work").unwrap();
    term.click(wc, wr).unwrap();
    term.wait_for("the question", STEP, |s| {
        s.contains("Work Me <me@work.test>") && s.contains("Write to git config")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Not yet written: the dialog is the choice.
    assert_eq!(local_config(repo.root(), "user.email"), baseline);
    // The choice is already shown and persisted per repository.
    let state =
        std::fs::read_to_string(repo.state_dir().join("repo-state.toml")).unwrap_or_default();
    assert!(state.contains("profile = \"work\""), "{state}");
    // Esc keeps it GitPoulpi-only.
    term.send_keys("Escape").unwrap();
    term.wait_for_text_to_go("Write to git config", STEP)
        .unwrap();
    assert_eq!(local_config(repo.root(), "user.email"), baseline);
    assert!(snap(&term).contains("Work"), "{}", term.text());
    // The palette twin opens the same list; picking the default again asks
    // for Home's identity, and confirming writes it.
    palette(&mut term, "Change Profile", "Repository: Change Profile");
    term.wait_for("the list", STEP, |s| s.contains("Default (Home)"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Up").unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for_text("Write to git config", STEP).unwrap();
    let (bc, br) = snap(&term).find("Write to git config").unwrap();
    term.click(bc + 1, br).unwrap();
    for _ in 0..100 {
        if local_config(repo.root(), "user.email") == "me@home.test" {
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    assert_eq!(local_config(repo.root(), "user.email"), "me@home.test");
    assert_eq!(local_config(repo.root(), "user.name"), "Home Me");
    let _ = dir;
    term.stop().unwrap();
}

#[test]
fn clicking_the_logo_spins_to_the_next_color_and_the_list_picks_one() {
    let (mut term, dir, _repo) = start("about-logo", "[ui]\ngraph_renderer = \"text\"\n");
    click_octopus(&mut term);
    term.wait_for("the About window", STEP, |s| s.contains("License: MIT"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // The hint line names both keys.
    assert!(snap(&term).contains("c: color"), "{}", term.text());
    // The logo is drawn as half-blocks here (no pixel protocol in the pty).
    let (lc, lr) = settled(&term).find("\u{2580}").expect("the logo");
    // A click on it spins (does not close) and lands on the next color.
    term.click(lc + 2, lr + 2).unwrap();
    let saved = || std::fs::read_to_string(dir.path().join("config.toml")).unwrap_or_default();
    for _ in 0..100 {
        if saved().contains("logo_color") {
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    // Lavender (the default) is followed by Sun.
    assert!(saved().contains("logo_color = \"sun\""), "{}", saved());
    assert!(settled(&term).contains("License: MIT"), "still open");
    // `c` opens the color list; typing narrows it; Enter picks.
    term.send_text("c").unwrap();
    term.wait_for("the color list", STEP, |s| {
        s.contains("Lavender") && s.contains("Mint")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_text("Mint").unwrap();
    term.send_keys("Enter").unwrap();
    for _ in 0..100 {
        if saved().contains("logo_color = \"mint\"") {
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    assert!(saved().contains("logo_color = \"mint\""), "{}", saved());
    term.send_keys("Escape").unwrap();
    term.wait_for_text_to_go("License: MIT", STEP).unwrap();
    term.stop().unwrap();
}

/// The color list opened by a right click on the logo is a menu: one
/// click on a row picks it.
#[test]
fn the_color_list_rows_are_clickable() {
    let (mut term, dir, _repo) = start("about-list-click", "[ui]\ngraph_renderer = \"text\"\n");
    click_octopus(&mut term);
    term.wait_for("the About window", STEP, |s| s.contains("License: MIT"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (lc, lr) = settled(&term).find("\u{2580}").expect("the logo");
    term.right_click(lc + 2, lr + 2).unwrap();
    let screen = term
        .wait_for("the color list", STEP, |s| s.contains("Mint"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (mc, mr) = screen.find("Mint").expect("the Mint row");
    term.click(mc + 1, mr).unwrap();
    let saved = || std::fs::read_to_string(dir.path().join("config.toml")).unwrap_or_default();
    for _ in 0..50 {
        if saved().contains("logo_color = \"mint\"") {
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    assert!(
        saved().contains("logo_color = \"mint\""),
        "{}\n{}",
        saved(),
        term.text()
    );
    term.wait_for("the list closed, About still open", STEP, |s| {
        !s.contains("Mint") && s.contains("License: MIT")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// The license, source and docs lines are links: a click opens them and
/// keeps the window open, `s` / `d` are their keys, the addresses show in
/// full, and `l` (or a click on its line) lists the third-party crates.
#[test]
fn the_links_open_and_the_crate_list_scrolls() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "first-commit")
        .unwrap()
        .build();
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("config.toml"),
        "[ui]\ngraph_renderer = \"text\"\n",
    )
    .unwrap();
    let log = tempfile::NamedTempFile::new().unwrap();
    let mut term = common::launch(repo.root())
        .session("about-links")
        .config_dir(dir.path())
        .size(200, 50)
        .env("GITPOULPI_OPEN_LOG", log.path())
        .no_exit_pause()
        .ready("first-commit")
        .spawn();
    click_octopus(&mut term);
    let docs = "https://github.com/Poulpi-Lab/GitPoulpi/tree/main/docs/user";
    // The docs address is on the screen the wait describes, not asserted
    // after it (the window is drawn over more than one write).
    let screen = term
        .wait_for(
            "the About window with the docs address in full",
            STEP,
            |s| s.contains("License: MIT") && s.contains(docs),
        )
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let opened = |n: usize| {
        for _ in 0..100 {
            let lines: Vec<String> = std::fs::read_to_string(log.path())
                .unwrap_or_default()
                .lines()
                .map(str::to_string)
                .collect();
            if lines.len() >= n {
                return lines;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        panic!("{n} URLs not opened")
    };
    let (lc, lr) = screen.find("License: MIT").unwrap();
    term.click(lc + 2, lr).unwrap();
    assert!(
        opened(1)[0].ends_with("/blob/main/LICENSE"),
        "{:?}",
        opened(1)
    );
    let (dc, dr) = snap(&term).find(docs).unwrap();
    term.click(dc + 4, dr).unwrap();
    assert_eq!(opened(2)[1], docs);
    term.send_text("s").unwrap();
    assert_eq!(opened(3)[2], "https://github.com/Poulpi-Lab/GitPoulpi");
    // The license has its own key, and every link shows its access key.
    term.send_text("i").unwrap();
    assert!(
        opened(4)[3].ends_with("/blob/main/LICENSE"),
        "{:?}",
        opened(4)
    );
    term.wait_for("every link's key, the window still open", STEP, |s| {
        [
            "[s] Source code",
            "[d] Documentation",
            "[i] License",
            "[l] Third-party",
            "License: MIT",
        ]
        .iter()
        .all(|k| s.contains(k))
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // The crate list, by key, then back with Esc (which does not close).
    term.send_text("l").unwrap();
    // Each crate shows its license (read from the registry at build time).
    term.wait_for("the crate list, arboard with its license", STEP, |s| {
        s.find("arboard")
            .is_some_and(|(_, r)| s.line(r).contains("MIT OR Apache-2.0"))
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("End").unwrap();
    term.wait_for("scrolled to the end", STEP, |s| {
        !s.contains("arboard ") && s.contains("tracing ")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Escape").unwrap();
    term.wait_for("back to the facts", STEP, |s| s.contains("License: MIT"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // A click on the crate line opens the list too.
    common::click_text_until(
        &mut term,
        "Third-party crates:",
        "the crate list by click",
        |s| s.contains("arboard "),
    );
    // The mouse way back: the list's title is a back link ("‹ Third-party
    // crates (n)"), lit under the pointer like every link.
    common::click_text_until(
        &mut term,
        "\u{2039} Third-party crates (",
        "back to the facts by click",
        |s| s.contains("License: MIT"),
    );
    // The ✕ in the top border closes the window.
    common::click_text_until(&mut term, "\u{2715}", "the window closed", |s| {
        !s.contains("About GitPoulpi")
    });
    term.stop().unwrap();
}

/// The About window's close cross lights up under the pointer, like every
/// other window's, and a click on it closes the window.
#[test]
fn the_close_cross_highlights_under_the_pointer() {
    let (mut term, _dir, _repo) = start("about-cross-hover", "[ui]\ngraph_renderer = \"text\"\n");
    click_octopus(&mut term);
    let screen = term
        .wait_for("the About window", STEP, |s| s.contains("License: MIT"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (xc, xr) = screen.find("\u{2715}").expect("the close cross");
    let at_rest = screen.cell(xc, xr).unwrap().bg;
    // Away from it first, so the pointer has been somewhere else.
    term.move_to(xc.saturating_sub(30), xr + 5).unwrap();
    term.move_to(xc, xr).unwrap();
    let lit = term
        .wait_for("the cross highlighted", STEP, |s| {
            s.cell(xc, xr).is_some_and(|c| c.bg != at_rest)
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(lit.contains("About GitPoulpi"));
    term.move_to(xc.saturating_sub(30), xr + 5).unwrap();
    term.wait_for("the cross back at rest", STEP, |s| {
        s.cell(xc, xr).is_some_and(|c| c.bg == at_rest)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// Every half-block cell of the About logo, as `(col, row, fg, bg)`.
fn logo_cells(
    screen: &xtask::pty::Screen,
) -> Vec<(u16, u16, xtask::pty::Color, xtask::pty::Color)> {
    let mut out = Vec::new();
    for row in 0..screen.rows() {
        for col in 0..screen.cols() {
            if let Some(c) = screen.cell(col, row) {
                if c.text == "\u{2580}" {
                    out.push((col, row, c.fg, c.bg));
                }
            }
        }
    }
    out
}

#[test]
fn the_octopus_in_about_follows_the_mouse_with_its_pupils() {
    let (mut term, _dir, _repo) = start("about-eyes", "[ui]\ngraph_renderer = \"text\"\n");
    click_octopus(&mut term);
    term.wait_for_text("License: MIT", STEP).unwrap();
    // Far top left, then far bottom right: the pupils sit in other cells.
    term.move_to(1, 1).unwrap();
    term.move_to(2, 2).unwrap();
    let top_left = logo_cells(&settled(&term));
    assert!(!top_left.is_empty(), "no half-block logo:\n{}", term.text());
    term.move_to(198, 48).unwrap();
    term.wait_for("the pupils moved", STEP, |s| logo_cells(s) != top_left)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let bottom_right = logo_cells(&settled(&term));
    // Back to the first corner: the same picture as before.
    term.move_to(1, 1).unwrap();
    term.move_to(2, 2).unwrap();
    term.wait_for("looking top left again", STEP, |s| {
        logo_cells(s) == top_left
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_ne!(top_left, bottom_right);
    term.stop().unwrap();
}

#[test]
fn ctrl_n_a_opens_about_and_the_flag_sits_under_the_octopus() {
    let (mut term, _dir, _repo) = start("about-nav-a", "[ui]\ngraph_renderer = \"text\"\n");
    let (col, row) = snap(&term).find("\u{1F419}").expect("the octopus");
    assert_eq!(row, 0);
    let flag = |s: &xtask::pty::Screen| {
        s.cell(col + 1, row + 1)
            .is_some_and(|c| c.text == "a" && c.bold)
    };
    assert!(!flag(&snap(&term)), "flag shown before the prefix");
    term.send_keys("C-n").unwrap();
    term.wait_for("the flag under the octopus", STEP, flag)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_text("a").unwrap();
    term.wait_for_text("License: MIT", STEP).unwrap();
    term.send_keys("Escape").unwrap();
    term.wait_for_text_to_go("License: MIT", STEP).unwrap();
    term.stop().unwrap();
}

/// `(column, up arrow row, down arrow row)` of the crate list's bar: the
/// arrow pair right under the list's title.
fn crates_bar(screen: &xtask::pty::Screen) -> Option<(u16, u16, u16)> {
    let (_, title) = screen.find("Third-party crates (")?;
    let (col, up) = screen
        .find_all("\u{2191}")
        .into_iter()
        .find(|(_, r)| *r == title + 1)?;
    let down = screen
        .find_all("\u{2193}")
        .into_iter()
        .filter(|(c, r)| *c == col && *r > up)
        .map(|(_, r)| r)
        .min()?;
    Some((col, up, down))
}

/// The owner could not drag the crate list's scrollbar: the thumb follows
/// the pointer (even off the track), the track jumps and the arrows step.
#[test]
fn the_crate_list_scrollbar_drags_jumps_and_steps() {
    let (mut term, _dir, _repo) = start("about-crates-bar", "[ui]\ngraph_renderer = \"text\"\n");
    click_octopus(&mut term);
    term.wait_for("the About window", STEP, |s| s.contains("License: MIT"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_text("l").unwrap();
    let screen = term
        .wait_for("the crate list with its bar", STEP, |s| {
            s.contains("arboard ") && crates_bar(s).is_some()
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.wait_for_mouse_capture(STEP).unwrap();
    let (col, up, down) = crates_bar(&screen).expect("the bar");
    // Hold the thumb (first cell under the up arrow), pull it past the
    // track's end and out sideways: the list ends up at its tail.
    term.press(col, up + 1).unwrap();
    term.drag_to(col - 20, down + 3).unwrap();
    term.wait_for("the tail of the crate list", STEP, |s| {
        !s.contains("arboard ") && s.contains("tracing ")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.release(col - 20, down + 3).unwrap();
    assert!(snap(&term).contains("License: MIT") || snap(&term).contains("Third-party"));
    // Back to the top by dragging the thumb up.
    let (col, up, down) = crates_bar(&snap(&term)).expect("the bar again");
    term.press(col, down - 1).unwrap();
    term.drag_to(col, up).unwrap();
    term.wait_for("the head of the crate list", STEP, |s| {
        s.contains("arboard ")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.release(col, up).unwrap();
    // The down arrow steps a row: the first crate scrolls away.
    let first = snap(&term).line(up).trim().to_string();
    term.click(col, down).unwrap();
    term.wait_for("one row down", STEP, |s| s.line(up).trim() != first)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// Every row of the crate list links to its crate's page: a click opens it
/// (and selects the row), Enter opens the selected one, Down moves on. The
/// browser is the `GITPOULPI_OPEN_LOG` stub: no browser ever starts.
#[test]
fn a_crate_row_opens_its_page_by_click_and_by_enter() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "first-commit")
        .unwrap()
        .build();
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("config.toml"),
        "[ui]\ngraph_renderer = \"text\"\n",
    )
    .unwrap();
    let log = tempfile::NamedTempFile::new().unwrap();
    let mut term = common::launch(repo.root())
        .session("about-crate-links")
        .config_dir(dir.path())
        .size(200, 50)
        .env("GITPOULPI_OPEN_LOG", log.path())
        .no_exit_pause()
        .ready("first-commit")
        .spawn();
    click_octopus(&mut term);
    term.wait_for("the About window", STEP, |s| s.contains("License: MIT"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_text("l").unwrap();
    term.wait_for("the crate list", STEP, |s| s.contains("arboard "))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Page down until ratatui's row shows (the list is longer than the page).
    // (Each page is given time to be drawn whole before the next PageDown.)
    let mut found = None;
    for _ in 0..40 {
        let screen = term.wait_for(
            "ratatui's row with its link",
            Duration::from_millis(1500),
            |s| {
                s.find("ratatui ").is_some_and(|(_, r)| {
                    s.line(r)
                        .split_whitespace()
                        .any(|w| w.contains('/') && w.contains('.') && w.contains("ratatui"))
                })
            },
        );
        if let Ok(screen) = screen {
            found = Some(screen);
            break;
        }
        term.send_keys("PageDown").unwrap();
    }
    let screen = found.unwrap_or_else(|| panic!("ratatui row:\n{}", term.text()));
    let (rc, rr) = screen.find("ratatui ").unwrap();
    let line = screen.line(rr);
    let shown = line
        .split_whitespace()
        .find(|w| w.contains('/') && w.contains('.'))
        .unwrap_or_else(|| panic!("the link shown on the row: {line:?}"))
        .to_string();
    assert!(shown.contains("ratatui"), "{shown}\n{}", term.text());
    // The link column shrinks as the longest crate name grows and then ends
    // in an ellipsis: the opened address must still start with what is shown.
    let expected = format!("https://{}", shown.trim_end_matches('\u{2026}'));
    let opened = |n: usize| {
        for _ in 0..100 {
            let lines: Vec<String> = std::fs::read_to_string(log.path())
                .unwrap_or_default()
                .lines()
                .map(str::to_string)
                .collect();
            if lines.len() >= n {
                return lines;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        panic!("{n} URLs not opened")
    };
    term.click(rc + 1, rr).unwrap();
    let first = opened(1)[0].clone();
    assert!(first.starts_with(&expected), "{first} vs {expected}");
    // The click selected the row: Enter opens the same page again.
    term.send_keys("Enter").unwrap();
    assert_eq!(opened(2)[1], first);
    // Down moves the selection to the next crate, whose page differs.
    term.send_keys("Down").unwrap();
    term.send_keys("Enter").unwrap();
    let third = opened(3)[2].clone();
    assert!(third.starts_with("https://") && third != first, "{third}");
    assert!(
        snap(&term).contains("Enter: open"),
        "the hint names the key:\n{}",
        term.text()
    );
    term.stop().unwrap();
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
