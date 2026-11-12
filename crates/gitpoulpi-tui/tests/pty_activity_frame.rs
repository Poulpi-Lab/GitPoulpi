//! The Activity Log window opened from the status bar's button (the way a
//! user opens it): the title drag moves it, the bottom corner resizes it and
//! its list shows more rows, `Ctrl+N m` arranges it. Whole-state waits.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::Screen;

const STEP: Duration = Duration::from_secs(30);
const ATTEMPT: Duration = Duration::from_secs(5);

fn title_at(s: &Screen, title: &str) -> Option<(u16, u16)> {
    s.find_all(title).into_iter().min_by_key(|(c, r)| (*r, *c))
}

fn grip(s: &Screen) -> Option<(u16, u16)> {
    s.find("\u{25e2}")
}

/// The rows of the window between its title and its bottom border that hold
/// text (what the list shows).
fn height_of_window(s: &Screen) -> Option<u16> {
    let (_, tr) = title_at(s, "Activity Log")?;
    let (_, gy) = grip(s)?;
    Some(gy - tr)
}

#[test]
fn the_activity_window_opened_from_the_status_button_moves_and_resizes() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "frames-commit")
        .unwrap()
        .build();
    let config = tempfile::tempdir().unwrap();
    let mut term = common::launch(repo.root())
        .session("pty-activity-frame")
        .config_dir(config.path())
        .size(140, 40)
        .no_exit_pause()
        .ready("frames-commit")
        .mouse_capture()
        .spawn();
    // The status bar's button (the last row's right end).
    let (bx, by) = term
        .screen()
        .find_all("Activity")
        .into_iter()
        .max_by_key(|(c, r)| (*r, *c))
        .expect("the Activity button");
    term.act_until(
        "the Activity window with its grips",
        ATTEMPT,
        STEP,
        |t| t.click(bx + 1, by),
        |s| title_at(s, "Activity Log").is_some_and(|(_, r)| r < by) && grip(s).is_some(),
    )
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let s = term.screen();
    let (tc, tr) = title_at(&s, "Activity Log").unwrap();
    let (gx, gy) = grip(&s).unwrap();
    let h0 = height_of_window(&s).unwrap();

    // Shrink first (the default fills the screen), then drag the title.
    term.act_until(
        "the window narrowed by its corner",
        ATTEMPT,
        STEP,
        |t| {
            if grip(&t.screen()) == Some((gx, gy)) {
                t.drag((gx, gy), (gx - 20, gy - 8))?;
            }
            Ok(())
        },
        |s| grip(s) == Some((gx - 20, gy - 8)) && title_at(s, "Activity Log") == Some((tc, tr)),
    )
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (dx, dy) = (6u16, 3u16);
    term.act_until(
        "the window moved by its title",
        ATTEMPT,
        STEP,
        |t| {
            if let Some((c, r)) = title_at(&t.screen(), "Activity Log") {
                if (c, r) == (tc, tr) {
                    t.drag((c + 1, r), (c + 1 + dx, r + dy))?;
                }
            }
            Ok(())
        },
        |s| {
            title_at(s, "Activity Log") == Some((tc + dx, tr + dy))
                && grip(s) == Some((gx - 20 + dx, gy - 8 + dy))
        },
    )
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // The corner again: the window grows and the content follows its height.
    let g = (gx - 20 + dx, gy - 8 + dy);
    term.act_until(
        "the window taller by its corner",
        ATTEMPT,
        STEP,
        |t| {
            if grip(&t.screen()) == Some(g) {
                t.drag(g, (g.0 + 6, g.1 + 4))?;
            }
            Ok(())
        },
        |s| grip(s) == Some((g.0 + 6, g.1 + 4)),
    )
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let h1 = height_of_window(&term.screen()).unwrap();
    assert!(h1 + 8 - 4 > h0 - 8 && h1 > h0 - 8, "grew: {h0} -> {h1}");

    // `Ctrl+N m`: the arrow moves it.
    term.send_keys("C-n").unwrap();
    term.send_text("m").unwrap();
    term.wait_for("the arrange banner", STEP, |s| s.contains("Arrange:"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Left").unwrap();
    term.wait_for("two columns left", STEP, |s| {
        title_at(s, "Activity Log") == Some((tc + dx - 2, tr + dy)) && s.contains("Arrange:")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Escape").unwrap();
    term.stop().unwrap();
}

/// The window opened by the palette (the Global one too) at a wide terminal:
/// the right border narrows it, then the title drag moves it (it must not
/// close it).
/// A drag the way Windows delivers it: one report per write, with the loop
/// free to draw between them.
fn paced(t: &mut xtask::pty::Terminal, from: (u16, u16), to: (u16, u16)) -> xtask::pty::Result<()> {
    t.press(from.0, from.1)?;
    std::thread::sleep(Duration::from_millis(60));
    let (mut c, mut r) = from;
    while (c, r) != to {
        c = if c < to.0 {
            c + 1
        } else if c > to.0 {
            c - 1
        } else {
            c
        };
        r = if r < to.1 {
            r + 1
        } else if r > to.1 {
            r - 1
        } else {
            r
        };
        t.drag_to(c, r)?;
        std::thread::sleep(Duration::from_millis(25));
    }
    t.release(to.0, to.1)
}

/// The window's title and grip once two reads 150 ms apart agree: a torn
/// frame (a half-drawn screen) must not pin the coordinates the drags start from.
fn settled_frame(term: &xtask::pty::Terminal) -> ((u16, u16), (u16, u16)) {
    let read = |t: &xtask::pty::Terminal| {
        let s = t.screen();
        title_at(&s, "Activity Log").zip(grip(&s))
    };
    let deadline = std::time::Instant::now() + STEP;
    let mut last = read(term);
    loop {
        std::thread::sleep(Duration::from_millis(150));
        let now = read(term);
        if let (Some(a), Some(b)) = (last, now) {
            if a == b {
                return a;
            }
        }
        assert!(
            std::time::Instant::now() < deadline,
            "the window never settled\n{}",
            term.text()
        );
        last = now;
    }
}

fn open_by_palette_and_frame(entry: &str, session: &str, home: bool) {
    open_with_config(entry, session, home, "")
}

fn open_with_config(entry: &str, session: &str, home: bool, toml: &str) {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "frames-commit")
        .unwrap()
        .build();
    let config = tempfile::tempdir().unwrap();
    if !toml.is_empty() {
        std::fs::write(config.path().join("config.toml"), toml).unwrap();
    }
    let mut term = common::launch(repo.root())
        .session(session)
        .config_dir(config.path())
        .size(142, 45)
        .no_exit_pause()
        .ready("frames-commit")
        .mouse_capture()
        .spawn();
    if home {
        let (hx, hy) = term.screen().find("Radar").expect("the Home tab");
        term.act_until(
            "Home in front",
            ATTEMPT,
            STEP,
            |t| t.click(hx, hy),
            |s| !s.contains("frames-commit"),
        )
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    }
    term.send_keys("C-p").unwrap();
    term.wait_for_text("Command Palette", STEP).unwrap();
    term.send_text(entry).unwrap();
    term.wait_for("the entry", STEP, |s| s.contains(entry))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    term.wait_for("the window", STEP, |s| {
        s.contains("Git operations") && title_at(s, "Activity Log").is_some() && grip(s).is_some()
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let ((tc, tr), (gx, gy)) = settled_frame(&term);

    term.act_until(
        "the window narrowed by its right border",
        ATTEMPT,
        STEP,
        |t| {
            if grip(&t.screen()) == Some((gx, gy)) {
                t.move_to(gx, gy - 5)?;
                paced(t, (gx, gy - 5), (gx - 30, gy - 5))?;
            }
            Ok(())
        },
        |s| grip(s) == Some((gx - 30, gy)),
    )
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (dx, dy) = (4u16, 2u16);
    term.act_until(
        "the window moved by its title",
        ATTEMPT,
        STEP,
        |t| {
            if title_at(&t.screen(), "Activity Log") == Some((tc, tr)) {
                t.move_to(tc + 1, tr)?;
                paced(t, (tc + 1, tr), (tc + 1 + dx, tr + dy))?;
            }
            Ok(())
        },
        |s| {
            title_at(s, "Activity Log") == Some((tc + dx, tr + dy))
                && grip(s) == Some((gx - 30 + dx, gy + dy))
        },
    )
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Escape").unwrap();
    term.stop().unwrap();
}

#[test]
fn the_activity_window_opened_by_the_palette_moves_and_narrows() {
    open_by_palette_and_frame("View: Activity Log", "pty-activity-frame-pal", false);
}

#[test]
fn the_global_activity_window_opened_by_the_palette_moves_and_narrows() {
    open_by_palette_and_frame(
        "View: Global Activity Log",
        "pty-activity-frame-glob",
        false,
    );
}

#[test]
fn the_activity_window_over_home_moves_and_narrows() {
    open_by_palette_and_frame("View: Activity Log", "pty-activity-frame-home", true);
}

#[test]
fn the_activity_window_moves_and_narrows_with_never() {
    open_with_config(
        "View: Activity Log",
        "pty-af-never",
        false,
        "[ui]\ndialog_geometry = \"never\"\n",
    );
}

#[test]
fn the_activity_window_moves_and_narrows_with_saved() {
    open_with_config(
        "View: Activity Log",
        "pty-af-saved",
        false,
        "[ui]\ndialog_geometry = \"saved\"\n",
    );
}
