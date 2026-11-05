//! One widget, one pointer: the same kind of component gives the same shape
//! wherever it is drawn. A combobox of the issues pane and a combobox of
//! Settings are both the pointing hand; the commit details pane's editable
//! message block is the I-beam and lights as a whole, its info lines are the
//! hand and light as a line. Each shape is the `OSC 22` the program wrote,
//! read back from the raw output.
//!
//! Real binary through a pty; temp config dirs and fixture repositories.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::mock_host::flow::{self, FlowHost, FlowHostOptions};
use xtask::pty::Terminal;

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);
const CHEVRON: &str = "\u{25BE}";

/// Every `OSC 22` pointer shape the program asked for, in order ("" is the
/// reset).
fn pointer_shapes(term: &Terminal) -> Vec<String> {
    let raw = String::from_utf8_lossy(&term.raw_output()).to_string();
    raw.split("\x1b]22;")
        .skip(1)
        .map(|s| s.split("\x1b\\").next().unwrap_or("").to_string())
        .collect()
}

/// Moves onto `(col, row)` and waits for the shape `want` to be the last one
/// requested.
fn hover(term: &mut Terminal, what: &str, (col, row): (u16, u16), want: &str) {
    term.move_to(col, row).unwrap();
    let deadline = std::time::Instant::now() + STEP;
    while pointer_shapes(term).last().map(String::as_str) != Some(want)
        && std::time::Instant::now() < deadline
    {
        std::thread::sleep(Duration::from_millis(50));
    }
    assert_eq!(
        pointer_shapes(term).last().map(String::as_str),
        Some(want),
        "over {what}: {:?}\n{}",
        pointer_shapes(term),
        term.text()
    );
}

struct Rig {
    _repo: FixtureRepo,
    _host: FlowHost,
    _config: tempfile::TempDir,
    term: Terminal,
}

/// A repository whose PULL REQUESTS and ISSUES sources are the mock host.
fn hosting_rig(session: &str) -> Rig {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "pointer widgets probe")
        .unwrap()
        .with_origin()
        .unwrap()
        .push("main")
        .unwrap()
        .build();
    let host = flow::start(FlowHostOptions::new(repo.remote_path().unwrap())).expect("host");
    let state = repo.state_dir();
    std::fs::create_dir_all(&state).unwrap();
    std::fs::write(
        state.join("repo-state.toml"),
        "[issues_source]\nintegration = \"h\"\nproject = \"o/p\"\n\n\
         [pulls_source]\nintegration = \"h\"\nproject = \"o/p\"\n",
    )
    .unwrap();
    let config = tempfile::tempdir().expect("temp config dir");
    std::fs::write(
        config.path().join("config.toml"),
        format!(
            "[ui]\ngraph_renderer = \"text\"\n\n[[integrations]]\nid = \"h\"\n\
             kind = \"gitlab_self_managed\"\nurl = \"http://127.0.0.1:{}\"\n\
             token_ref = \"integration:h\"\nname = \"h\"\nuser = {{ login = \"mara\" }}\n",
            host.port
        ),
    )
    .unwrap();
    let term = common::launch(repo.root())
        .session(session)
        .config_dir(config.path())
        .size(204, 60)
        .no_exit_pause()
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GITPOULPI_TOKEN_H", "test-token")
        .mouse_capture()
        .spawn();
    Rig {
        _repo: repo,
        _host: host,
        _config: config,
        term,
    }
}

/// The issues pane's comboboxes and Settings' comboboxes are both the
/// pointing hand; an item row of the pane keeps the arrow.
#[test]
fn a_combobox_of_the_issues_pane_and_one_of_settings_are_both_the_hand() {
    let mut rig = hosting_rig("pty-pointer-widgets-combo");
    let term = &mut rig.term;
    let screen = term
        .wait_for("the sidebar sections", STARTUP, |s| {
            s.contains("REQUESTS") && s.contains("ISSUES") && s.contains(CHEVRON)
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // A combobox row of the sidebar: its `v` sits well right of the section
    // chevrons at the left edge.
    let combo = screen
        .find_all(CHEVRON)
        .into_iter()
        .find(|(c, r)| *c > 15 && *c < 60 && *r > 3)
        .unwrap_or_else(|| panic!("no combobox in the sidebar\n{}", term.text()));
    // The hand on a combobox of the pane, the arrow back on a Log cell.
    // The Log's row (the one line holding both the ref and the message; the
    // details pane repeats the message, as the I-beam).
    let log_cell = screen
        .find_all("pointer widgets pro")
        .into_iter()
        .find(|(_, r)| screen.line(*r).contains("main"))
        .unwrap_or_else(|| panic!("no Log row\n{}", term.text()));
    hover(
        term,
        "an issues pane combobox",
        (combo.0 - 4, combo.1),
        "pointer",
    );
    // Settings: its combobox is the same shape.
    hover(term, "a Log cell again", (log_cell.0 + 2, log_cell.1), "");
    let (gear_col, gear_row) = term
        .wait_for_text("\u{2699}", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(gear_col, gear_row).expect("click the gear");
    term.wait_for("Settings open", STEP, |s| {
        s.contains("Repo-specific settings") && s.contains("Search settings")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let screen = term.screen();
    let combo = screen
        .find_all(CHEVRON)
        .into_iter()
        .find(|(c, r)| *c > 40 && *r > 3)
        .unwrap_or_else(|| panic!("no combobox in Settings\n{}", term.text()));
    hover(
        term,
        "a Settings combobox",
        (combo.0 - 1, combo.1),
        "pointer",
    );
    rig.term.stop().unwrap();
}

/// The commit details pane: the message block is editable (the I-beam, the
/// whole block lit), the commit id line is a control (the hand, the line lit),
/// the stats line has no action (the arrow).
#[test]
fn the_commit_details_message_is_an_i_beam_and_its_info_lines_are_hands_and_light() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file(
            "a.txt",
            "a\n",
            "pointer details subject\n\nthe long description body of it",
        )
        .unwrap()
        .build();
    let config = tempfile::tempdir().unwrap();
    let mut term = common::launch(repo.root())
        .no_commit_column()
        .session("pty-pointer-details")
        .config_dir(config.path())
        .size(200, 45)
        .no_exit_pause()
        .mouse_capture()
        .spawn();
    let screen = term
        .wait_for("the details pane", STARTUP, |s| {
            s.contains("Changed files") && s.contains("the long description body")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let bg = |term: &Terminal, (c, r): (u16, u16)| term.screen().cell(c, r).unwrap().bg;
    let wait_lit = |term: &mut Terminal, at: (u16, u16), rest| {
        let deadline = std::time::Instant::now() + STEP;
        while bg(term, at) == rest && std::time::Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(50));
        }
    };
    // The description line of the pane (the right half of the window).
    let body = screen
        .find_all("the long description body")
        .into_iter()
        .find(|(c, _)| *c > 100)
        .expect("the description in the details pane");
    let summary = screen
        .find_all("pointer details subject")
        .into_iter()
        .find(|(c, r)| *c > 100 && *r < body.1)
        .expect("the summary in the details pane");
    let far = (body.0 + 45, body.1);
    term.move_to(1, 30).unwrap();
    std::thread::sleep(Duration::from_millis(400));
    let rest = bg(&term, far);
    let rest_summary = bg(&term, (far.0, summary.1));
    hover(&mut term, "the message", (body.0 + 1, body.1), "text");
    wait_lit(&mut term, far, rest);
    assert_ne!(
        bg(&term, far),
        rest,
        "the message block lights\n{}",
        term.text()
    );
    assert_ne!(
        bg(&term, (far.0, summary.1)),
        rest_summary,
        "the whole block lights, summary line too"
    );
    // The commit id line: the hand and its own light.
    let id = term
        .screen()
        .find_all("commit: ")
        .into_iter()
        .find(|(c, _)| *c > 100)
        .unwrap_or_else(|| panic!("no id line\n{}", term.text()));
    let id_far = (id.0 + 30, id.1);
    term.move_to(1, 30).unwrap();
    std::thread::sleep(Duration::from_millis(400));
    let id_rest = bg(&term, id_far);
    hover(&mut term, "the commit id line", (id.0 + 2, id.1), "pointer");
    wait_lit(&mut term, id_far, id_rest);
    assert_ne!(
        bg(&term, id_far),
        id_rest,
        "the id line lights\n{}",
        term.text()
    );
    // The authored-date line is a control too (a double click amends it).
    let date = term
        .screen()
        .find_all("authored on")
        .into_iter()
        .find(|(c, _)| *c > 100)
        .expect("the authored date line");
    hover(&mut term, "the date line", (date.0 + 2, date.1), "pointer");
    // The stats line has no action: the arrow.
    let stats = term
        .screen()
        .find_all("added")
        .into_iter()
        .find(|(c, _)| *c > 100)
        .expect("the stats line");
    hover(&mut term, "the stats line", (stats.0 + 1, stats.1), "");
    term.stop().unwrap();
}
