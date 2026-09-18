//! The Graph column's picture (and the per-row avatars and row tint) must
//! be gone whenever the Log is not the central view (Settings, the Diff
//! view, the Home tab). ratatui's diff skips a cell the new screen leaves
//! blank, so nothing erased the picture on iTerm2 / WezTerm, and a Kitty
//! picture was never deleted.
//!
//! Driven through the real binary with a pinned cell size, inspecting the
//! raw byte stream. Kitty: a delete (`a=d,d=I`) must be written. iTerm2: no
//! image escape may be written again while the Log is away. The cell
//! rewrite itself is pinned by unit tests in `graph::inline_image`.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(60);
const MESSAGE: &str = "graph hide probe commit";
const LOG_HEADER: &str = "Message";

fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("probe.txt", "one\n", "an older commit under the probe")
        .unwrap()
        .commit_file("probe.txt", "one\ntwo changed\n", MESSAGE)
        .unwrap()
        .build()
}

#[derive(Clone, Copy)]
enum Wire {
    Iterm2,
    Wezterm,
    Kitty,
}

#[derive(Clone, Copy)]
enum Away {
    Settings,
    Diff,
    Home,
}

fn count(haystack: &[u8], needle: &[u8]) -> usize {
    haystack
        .windows(needle.len())
        .filter(|w| *w == needle)
        .count()
}

/// Image escapes anchored left of column 100: the Log's side. The commit
/// detail's avatar (right panel, column 132 at this size) may legitimately
/// be redrawn beside the Settings popup.
fn log_side_images(stream: &[u8], marker: &[u8]) -> usize {
    let mut found = 0;
    for (i, w) in stream.windows(marker.len()).enumerate() {
        if w != marker {
            continue;
        }
        // The last `ESC [ row ; col H` before the escape is its anchor.
        let head = &stream[..i];
        let text = String::from_utf8_lossy(head);
        let col = text.rfind("\x1b[").and_then(|_| {
            text.rmatches('H')
                .next()
                .and(text.rfind('H'))
                .and_then(|h| {
                    let start = text[..h].rfind("\x1b[")?;
                    let (r, c) = text[start + 2..h].split_once(';')?;
                    // Row 1 is the title bar: its octopus picture is not the Log's.
                    if r == "1" {
                        return Some(u32::MAX);
                    }
                    c.parse::<u32>().ok()
                })
        });
        if col.map_or(true, |c| c <= 100) {
            found += 1;
        }
    }
    found
}

fn run(wire: Wire, away: Away, session: &str) {
    let repo = fixture();
    let protocol = match wire {
        Wire::Iterm2 | Wire::Wezterm => "iterm2",
        Wire::Kitty => "kitty",
    };
    let mut launch = common::launch(repo.root())
        .session(session)
        .config(format!(
            "[ui]\ngraph_renderer = \"image\"\nimage_protocol = \"{protocol}\"\n\
             avatars = \"graph\"\navatar_sources = \"hosting\"\navatar_fallback = \"identicon\"\n\
             graph_lane_background = \"row_tint\"\n"
        ))
        .terminal_image()
        .env("GITPOULPI_TERMINAL_CELL_PX", "9x21")
        .size(200, 60)
        .ready_when("the fixture loaded", |s| s.contains(MESSAGE));
    if let Wire::Wezterm = wire {
        launch = launch
            .env("TERM_PROGRAM", "WezTerm")
            .env("TERM", "xterm-256color")
            .env("HERDR_ENV", "")
            .env("HERDR_PANE_ID", "")
            .env("HERDR_SOCKET_PATH", "");
    }
    let (mut term, _config) = launch.start();
    let quiet = Duration::from_millis(800);
    term.wait_for_stable(quiet, STEP).unwrap();
    let marker: &[u8] = match wire {
        Wire::Iterm2 | Wire::Wezterm => b"]1337;File=",
        Wire::Kitty => b"_Ga=",
    };
    assert!(
        count(&term.raw_output(), marker) > 0,
        "{session}: the Log never sent a picture"
    );

    // The commit column beside a diff draws a Graph picture of its own (its
    // own tests: pty_commit_column), left of column 100 like the Log's:
    // turn it off first (Alt+C answers on the Log too), so this test keeps
    // asking only whether the Log's picture is gone.
    if matches!(away, Away::Diff) {
        term.send_keys("M-c").unwrap();
        term.wait_for_stable(quiet, STEP).unwrap();
    }
    let before = term.raw_output().len();
    leave_the_log(&mut term, away);
    term.wait_for_stable(quiet, STEP).unwrap();
    std::thread::sleep(Duration::from_millis(1200));
    let all = term.raw_output();
    let after = &all[before..];
    match wire {
        Wire::Kitty => assert!(
            count(after, b"_Ga=d,d=I") > 0,
            "{session}: no Kitty delete reached the terminal after leaving the Log"
        ),
        Wire::Iterm2 | Wire::Wezterm => assert_eq!(
            log_side_images(after, marker),
            0,
            "{session}: an image escape was written while the Log is away"
        ),
    }
    let _ = term.stop();
}

fn leave_the_log(term: &mut Terminal, away: Away) {
    match away {
        Away::Settings => {
            let (col, row) = term
                .wait_for_text("\u{2699}", STEP)
                .unwrap_or_else(|e| panic!("{e}"));
            term.click(col, row).expect("click the gear");
            term.wait_for("Settings open", STEP, |s| {
                s.contains("Repo-specific settings")
            })
            .unwrap_or_else(|e| panic!("{e}"));
        }
        Away::Diff => {
            let (c, r) = term
                .wait_for_text("probe.txt", STEP)
                .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
            term.click(c, r).unwrap();
            term.wait_for("the diff in the center", STEP, |s| {
                s.contains("two changed") && !s.contains(LOG_HEADER)
            })
            .unwrap_or_else(|e| panic!("{e}"));
        }
        Away::Home => {
            term.send_keys("C-o").expect("go Home");
            term.wait_for("Home", STEP, |s| s.contains("Repository Management"))
                .unwrap_or_else(|e| panic!("{e}"));
        }
    }
}

macro_rules! case {
    ($name:ident, $wire:expr, $away:expr) => {
        #[test]
        fn $name() {
            run($wire, $away, stringify!($name));
        }
    };
    // The same, for a wire whose escapes the Windows pty does not hand over
    // as the app wrote them.
    ($name:ident, $wire:expr, $away:expr, $why:literal) => {
        #[test]
        #[cfg_attr(windows, ignore = $why)]
        fn $name() {
            run($wire, $away, stringify!($name));
        }
    };
}

case!(
    kitty_settings,
    Wire::Kitty,
    Away::Settings,
    "ConPTY drops the kitty graphics APC from the stream the driver reads"
);
case!(
    kitty_diff,
    Wire::Kitty,
    Away::Diff,
    "ConPTY drops the kitty graphics APC from the stream the driver reads"
);
case!(
    kitty_home,
    Wire::Kitty,
    Away::Home,
    "ConPTY drops the kitty graphics APC from the stream the driver reads"
);
case!(iterm2_settings, Wire::Iterm2, Away::Settings, "ConPTY re-anchors the picture escapes, so the cursor positions the test parses are not the app's");
case!(iterm2_diff, Wire::Iterm2, Away::Diff);
case!(iterm2_home, Wire::Iterm2, Away::Home, "ConPTY re-anchors the picture escapes, so the cursor positions the test parses are not the app's");
case!(wezterm_settings, Wire::Wezterm, Away::Settings, "ConPTY re-anchors the picture escapes, so the cursor positions the test parses are not the app's");
case!(wezterm_diff, Wire::Wezterm, Away::Diff);
case!(wezterm_home, Wire::Wezterm, Away::Home, "ConPTY re-anchors the picture escapes, so the cursor positions the test parses are not the app's");

/// WezTerm (transparent iTerm2 pictures): Settings leaves most of the Log's
/// rows blank below its last section. A blank cell equals the blank the
/// picture sat on, so ratatui's diff writes nothing there and the picture
/// stays, or is even sent again under the page. A hundred-odd commits make
/// the Graph column much taller than Settings' text, so under half of its
/// cells differ from the plain reservation. Red before the reservation
/// mark: the picture is written again after Settings opens.
#[test]
#[cfg_attr(
    windows,
    ignore = "ConPTY re-anchors the picture escapes, so the cursor positions the test parses are not the app's"
)]
fn wezterm_settings_rewrites_the_blank_rows_under_its_last_section() {
    let mut builder = FixtureRepoBuilder::new().unwrap();
    for i in 0..115 {
        builder = builder
            .commit_file(
                "probe.txt",
                &format!("line {i}\n"),
                &format!("commit {i} {MESSAGE}"),
            )
            .unwrap();
    }
    let repo = builder.build();
    let (mut term, _config) = common::launch(repo.root())
        .session("wezterm-blank-rows")
        .config(
            "[ui]
graph_renderer = \"image\"
image_protocol = \"iterm2\"
             avatars = \"graph\"
avatar_sources = \"hosting\"
avatar_fallback = \"identicon\"
             graph_lane_background = \"row_tint\"
",
        )
        .terminal_image()
        .env("GITPOULPI_TERMINAL_CELL_PX", "9x21")
        .env("TERM_PROGRAM", "WezTerm")
        .env("TERM", "xterm-256color")
        .env("HERDR_ENV", "")
        .env("HERDR_PANE_ID", "")
        .env("HERDR_SOCKET_PATH", "")
        .size(200, 130)
        .ready_when("the fixture loaded", |s| s.contains(MESSAGE))
        .start();
    let quiet = Duration::from_millis(800);
    term.wait_for_stable(quiet, STEP).unwrap();
    // The Graph picture's column: where the first picture was anchored.
    let first = term.raw_output();
    let log_text = String::from_utf8_lossy(&first).into_owned();
    // Past the title bar's octopus (anchored on row 1).
    let at = log_text
        .match_indices("]1337;File=")
        .map(|(i, _)| i)
        .find(|&i| {
            let head = &log_text[..i];
            !head[head.rfind("\x1b[").unwrap() + 2..head.rfind('H').unwrap()].starts_with("1;")
        })
        .expect("the Log sent a picture");
    let head = &log_text[..at];
    let cup = &head[head.rfind("\x1b[").unwrap() + 2..head.rfind('H').unwrap()];
    let graph_col: u32 = cup.split_once(';').unwrap().1.parse().unwrap();
    // Opened from the keyboard (Ctrl+N then `,`): a mouse move to the gear
    // would repaint the Log first, and a Log frame may resend its picture.
    // A first prefix press and its cancel settle whatever the first key
    // repaints; only then is the stream measured.
    //
    // Each key waits for the screen to show it was taken (the prefix hint
    // replaces the footer's `?: help`), not for a quiet period: under load
    // the TUI can be a long way behind a screen that looks settled, and an
    // Escape still unread when the next Ctrl+N arrives is read with it as
    // Alt+Ctrl+N.
    term.send_keys("C-n").unwrap();
    term.wait_for("the prefix hint", STEP, |s| !s.contains("?: help"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Escape").unwrap();
    term.wait_for("the prefix cancelled", STEP, |s| s.contains("?: help"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.wait_for_stable(quiet, STEP).unwrap();
    let before = term.raw_output().len();
    term.send_keys("C-n").unwrap();
    term.wait_for("the prefix hint", STEP, |s| !s.contains("?: help"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys(",").unwrap();
    term.wait_for("Settings open", STEP, |s| {
        s.contains("Repo-specific settings")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.wait_for_stable(quiet, STEP).unwrap();
    let all = term.raw_output();
    let after = &all[before..];
    assert_eq!(
        log_side_images(after, b"]1337;File="),
        0,
        "the Graph picture was sent again under a Settings page that is blank below its text"
    );
    // Its cells down there, where Settings paints nothing, must be
    // rewritten (a move to the Graph column on a deep row): that is what
    // makes the terminal drop the picture.
    let text = String::from_utf8_lossy(after);
    let rewritten = text.split("\x1b[").skip(1).any(|chunk| {
        chunk.split_once('H').is_some_and(|(pos, _)| {
            pos.split_once(';').is_some_and(|(r, c)| {
                c.parse::<u32>().ok() == Some(graph_col)
                    && r.parse::<u16>().is_ok_and(|r| (100..=120).contains(&r))
            })
        })
    });
    assert!(
        rewritten,
        "the Graph column's blank rows were never rewritten"
    );
    let _ = term.stop();
}
