//! Settings text fields and editable combos fill the room right of the value
//! column instead of a fixed 40/36 cells: through a pty, a long text value
//! shows whole and a click on the far right of the widened band lands in the
//! field. A pick-only choice fits its options (it does not stretch) and its
//! open popup starts at the field. The AI environment-variable value fills
//! its row, the remove cross stays at the right end.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);
/// Longer than the old 40-cell band, shorter than the new one at 200 columns.
const LONG: &str = "/srv/projects/a/rather/long/clone/folder/that/used/to/be/cut/off/END";

fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial commit")
        .unwrap()
        .build()
}

fn launch(repo: &FixtureRepo, session: &str, config: &str) -> (Terminal, tempfile::TempDir) {
    common::launch(repo.root())
        .session(session)
        .config(config)
        .size(200, 50)
        .ready_when("the fixture loaded", |s| s.contains("initial commit"))
        .start()
}

fn open_settings(term: &mut Terminal) {
    let (gear_col, gear_row) = term
        .wait_for_text("\u{2699}", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(gear_col, gear_row).expect("click the gear");
    term.wait_for("Settings open", STEP, |s| {
        s.contains("Repo-specific settings")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

#[test]
fn a_wide_text_field_shows_a_long_value_whole_and_takes_clicks_on_its_far_right() {
    let repo = fixture();
    let config = format!("[repos]\nclone_root = \"{LONG}\"\n");
    let (mut term, _config) = launch(&repo, "pty-wide-text", &config);
    open_settings(&mut term);
    common::scroll_settings_to(&mut term, "Default clone folder", STEP);
    let screen = term
        .wait_for("the whole long value", STEP, |s| s.contains(LONG))
        .unwrap_or_else(|e| panic!("{e}"));
    let (col, _) = screen.find(LONG).unwrap();
    // The far right of the band, well past where the old 40-cell band ended.
    let far = col + LONG.len() as u16 + 6;
    assert!(far > col + 40, "the click must be past the old band");
    // Retried (the field located afresh): a click on a torn frame lands
    // nowhere.
    common::retry_until(
        &mut term,
        "the field in edit mode (caret drawn)",
        4,
        STEP,
        |t| {
            let s = t.screen();
            if !s.contains(&format!("{LONG}\u{2588}")) {
                if let Some((col, row)) = s.find(LONG) {
                    t.click(col + LONG.len() as u16 + 6, row)
                        .expect("click the far right of the field");
                }
            }
        },
        |s| s.contains(&format!("{LONG}\u{2588}")),
    );
}

#[test]
fn a_pick_only_choice_fits_its_options_and_its_popup_starts_at_the_field() {
    let repo = fixture();
    let (mut term, _config) = launch(&repo, "pty-wide-choice", "");
    open_settings(&mut term);
    common::settings_page(&mut term, "Git & encoding");
    let (col, row) = common::wait_for_choice(&term, "direct", STEP);
    let line: Vec<char> = term.screen().line(row).chars().collect();
    let arrow = line.iter().position(|c| *c == '\u{25BE}').unwrap() as u16;
    // Band: one blank, text, padding, arrow, one blank; it starts one cell
    // left of the text.
    let band_w = arrow + 2 - (col - 1);
    assert!(
        (20..40).contains(&band_w),
        "a short-option choice must fit its content, got {band_w}\n{}",
        term.text()
    );
    term.click(col + 1, row).expect("click the choice");
    let screen = term
        .wait_for("the popup", STEP, |s| s.contains("\u{2713} direct"))
        .unwrap_or_else(|e| panic!("{e}"));
    let (tick_col, tick_row) = screen.find("\u{2713} direct").unwrap();
    let popup: Vec<char> = screen.line(tick_row).chars().collect();
    let border = |c: &char| matches!(c, '\u{2502}' | '\u{2503}');
    let left = popup[..tick_col as usize].iter().rposition(border);
    let right = popup[tick_col as usize..]
        .iter()
        .position(border)
        .map(|i| i + tick_col as usize);
    let (left, right) = (left.expect("left border"), right.expect("right border"));
    assert!(
        (right - left + 1) as u16 >= band_w,
        "popup at least as wide as the field\n{}",
        term.text()
    );
    assert_eq!(left as u16, col - 1, "popup starts at the field");
}

#[cfg(feature = "ai")]
mod env_rows {
    use super::*;

    const LONG_ENV: &str = "/srv/claude/profiles/a/rather/long/config/dir/END";

    fn config() -> String {
        format!(
            "[ai]\nenabled = true\ndefault_profile = \"alpha\"\n\n[[ai.profiles]]\nid = \"alpha\"\nname = \"Alpha\"\nkind = \"local_agent\"\ncommand = \"claude\"\nargs = [\"-p\"]\n\n[ai.profiles.env]\nCLAUDE_CONFIG_DIR = \"{LONG_ENV}\"\nMY_FLAG = \"on\"\n"
        )
    }

    #[test]
    fn an_env_value_fills_its_row_takes_clicks_far_right_and_its_cross_still_removes() {
        let repo = fixture();
        let (mut term, config_dir) = launch(&repo, "pty-wide-env", &config());
        open_settings(&mut term);
        common::settings_page(&mut term, "AI");
        common::settings_tab(&mut term, "Profiles (connections)");
        // The run mode rows push the env rows below the fold.
        common::scroll_settings_to(&mut term, "Env MY_FLAG", STEP);
        let screen = term
            .wait_for("the env rows, whole", STEP, |s| {
                s.contains("Env CLAUDE_CONFIG_DIR")
                    && s.contains(LONG_ENV)
                    && s.find("Env MY_FLAG").is_some_and(|(_, r)| {
                        s.line(r).contains('\u{2715}') && s.line(r).contains(" on ")
                    })
            })
            .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
        // The long value shows whole.
        assert!(screen.contains(LONG_ENV), "{}", term.text());
        // The short value's field runs to the cross at the right end.
        let (_, row) = screen.find("Env MY_FLAG").unwrap();
        let line: Vec<char> = screen.line(row).chars().collect();
        let cross = line.iter().rposition(|c| *c == '\u{2715}').unwrap() as u16;
        let (value_col, _) = screen.find(" on ").unwrap();
        assert!(
            cross > value_col + 60,
            "the cross must sit at the far right, value at {value_col}, cross at {cross}\n{}",
            term.text()
        );
        // A click far right inside the field edits the value.
        common::retry_until(
            &mut term,
            "the value in edit mode (caret drawn)",
            4,
            STEP,
            |t| {
                let s = t.screen();
                if !s.contains("on\u{2588}") {
                    // Located afresh: the page can still be settling.
                    if let Some((_, row)) = s.find("Env MY_FLAG") {
                        if let Some(cross) = s
                            .line(row)
                            .chars()
                            .collect::<Vec<_>>()
                            .iter()
                            .rposition(|c| *c == '\u{2715}')
                        {
                            t.click(cross as u16 - 6, row)
                                .expect("click the value field");
                        }
                    }
                }
            },
            |s| s.contains("on\u{2588}"),
        );
        term.send_keys("Escape").unwrap();
        // The cross removes the variable (the row drawn whole again).
        let screen = term
            .wait_for("edit closed", STEP, |s| {
                !s.contains("on\u{2588}")
                    && s.find("Env MY_FLAG")
                        .is_some_and(|(_, r)| s.line(r).contains('\u{2715}'))
            })
            .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
        let (_, row) = screen.find("Env MY_FLAG").unwrap();
        let cross = screen
            .line(row)
            .chars()
            .collect::<Vec<_>>()
            .iter()
            .rposition(|c| *c == '\u{2715}')
            .unwrap();
        term.click(cross as u16, row).unwrap();
        term.wait_for("MY_FLAG removed", STEP, |s| !s.contains("Env MY_FLAG"))
            .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
        let stored = std::fs::read_to_string(config_dir.path().join("config.toml")).unwrap();
        assert!(!stored.contains("MY_FLAG"), "{stored}");
        assert!(stored.contains(LONG_ENV), "{stored}");
        // The editable "Add an environment variable" combo fills the room.
        let screen = term
            .wait_for("the add row's combo", STEP, |s| {
                s.find("Add an environment variable")
                    .is_some_and(|(_, r)| s.line(r).contains('\u{25BE}'))
            })
            .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
        let (_, row) = screen
            .find("Add an environment variable")
            .expect("the add row");
        let arrow = screen
            .line(row)
            .chars()
            .position(|c| c == '\u{25BE}')
            .expect("the combo arrow");
        assert!(
            arrow > 150,
            "an editable combo fills the room, arrow at {arrow}"
        );
    }
}
