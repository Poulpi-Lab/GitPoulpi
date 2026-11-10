//! Button contrast on a light blue-gray theme (maintainer report 2026-10-08):
//! the merge editor bar's inactive buttons and the Unsaved-changes dialog's
//! Discard / Cancel painted dark text on the dark `ui.button` fill. The pty
//! has no pixels, so the painted cell colors are what is compared.
//! `iceberg-light` (bg #e8e9ec) is the preset closest to the screenshots.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::ConflictFixture;
use xtask::pty::{Color, Screen};

const STEP: Duration = Duration::from_secs(30);

fn lum((r, g, b): (u8, u8, u8)) -> f64 {
    let ch = |v: u8| {
        let v = f64::from(v) / 255.0;
        if v <= 0.03928 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * ch(r) + 0.7152 * ch(g) + 0.0722 * ch(b)
}

/// Label-on-fill contrast of the first cell of `needle`.
fn label_ratio(screen: &Screen, needle: &str) -> f64 {
    let (c, r) = screen
        .find(needle)
        .unwrap_or_else(|| panic!("no {needle:?} on\n{}", screen.text()));
    let cell = screen.cell(c, r).unwrap();
    let (Color::Rgb(fr, fg, fb), Color::Rgb(br, bg, bb)) = (cell.fg, cell.bg) else {
        panic!(
            "{needle:?}: fg {:?} bg {:?} are not both RGB",
            cell.fg, cell.bg
        );
    };
    let (a, b) = (lum((fr, fg, fb)), lum((br, bg, bb)));
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}

#[test]
fn the_merge_editor_bar_and_the_unsaved_dialog_read_on_a_light_theme() {
    let fixture = ConflictFixture::multi_conflict();
    let (mut term, _config) = common::launch(fixture.root())
        .session("pty-button-contrast")
        .config("[ui]\ntheme = \"iceberg-light\"\n")
        .size(200, 50)
        .no_exit_pause()
        .ready_when("the conflicted file in the Status pane", |s| {
            s.contains("Unstaged") && s.contains("Merge in progress")
        })
        .mouse_capture()
        .start();
    let (c, r) = term.wait_for_text("a.txt", STEP).unwrap();
    term.click(c, r).unwrap();
    term.send_keys("Enter").unwrap();
    let screen = term
        .wait_for("the merge editor with its whole bar", STEP, |s| {
            s.contains("file 1 of 3")
                && ["A Current", "B Incoming", "Reset", "Full file", "Save"]
                    .iter()
                    .all(|l| s.contains(l))
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    for label in ["A Current", "B Incoming", "Reset", "Full file", "Save"] {
        let k = label_ratio(&screen, label);
        // `Save` is disabled until something is picked: muted but readable.
        let min = if label == "Save" { 3.0 } else { 4.5 };
        assert!(
            k >= min,
            "{label:?} reads {k:.2} < {min}\n{}",
            screen.text()
        );
    }

    // An unsaved pick, then Alt+PageDown asks Save / Discard / Cancel.
    term.send_keys("a").unwrap();
    term.send_text("\x1b[6;3~").unwrap();
    let screen = term
        .wait_for("the unsaved-changes box", STEP, |s| {
            s.contains("Unsaved changes")
                && ["Save (", "Discard (", "Cancel ("]
                    .iter()
                    .all(|l| s.contains(l))
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    for label in ["Save (", "Discard (", "Cancel ("] {
        let k = label_ratio(&screen, label);
        assert!(k >= 4.5, "{label:?} reads {k:.2} < 4.5\n{}", screen.text());
    }
    term.stop().unwrap();
}
