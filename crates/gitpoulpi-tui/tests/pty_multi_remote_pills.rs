//! A branch with extra push targets wears a remotes badge: `⇉3 ↑1` in the
//! Refs row (a hit target that opens the branch's remotes dialog, with a
//! keyboard twin on the row) and `⇉3` on its Log pill, green when every
//! remote is in sync, yellow when some is ahead or has no copy yet, red when
//! one diverged. The Git settings page carries the repository's "Extra push
//! targets for new branches" checkboxes and the global preflight switch.
//!
//! The rules are `pty_ui.rs`'s: nothing is located by a hard-coded
//! coordinate and every wait describes the whole state it waits for.
//! Falsified: with the badge's hit target removed the click opens nothing;
//! with the colour mapping broken the three states share one colour.

mod common;

use std::time::Duration;

use gitpoulpi_config::theme::{preset, ThemeColor};
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::{Color, Screen, Terminal};

const STEP: Duration = Duration::from_secs(30);
const GLYPH: &str = "\u{21c9}";

#[derive(Clone, Copy, PartialEq, Eq)]
enum State {
    /// Every remote holds the branch tip.
    InSync,
    /// origin and gitlab are one commit behind; backup has no copy.
    Pending,
    /// The branch was rewritten after it was pushed to every remote.
    Diverged,
}

/// `feat` is checked out, tracks `origin` and also pushes to `gitlab` and
/// `backup` (`gitpoulpi-push-to`).
fn fixture(state: State) -> FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("base commit")
        .unwrap()
        .with_origin()
        .unwrap()
        .with_remote("gitlab")
        .unwrap()
        .with_remote("backup")
        .unwrap()
        .push("main")
        .unwrap()
        .build();
    repo.git(&["checkout", "-q", "-b", "feat"]);
    repo.git(&["commit", "--allow-empty", "-q", "-m", "feat work"]);
    repo.git(&["push", "-q", "-u", "origin", "feat"]);
    repo.git(&["push", "-q", "gitlab", "feat"]);
    if state != State::Pending {
        repo.git(&["push", "-q", "backup", "feat"]);
    }
    repo.git(&["config", "--add", "branch.feat.gitpoulpi-push-to", "gitlab"]);
    repo.git(&["config", "--add", "branch.feat.gitpoulpi-push-to", "backup"]);
    match state {
        State::InSync => {}
        State::Pending => {
            repo.git(&["commit", "--allow-empty", "-q", "-m", "one more"]);
        }
        State::Diverged => {
            repo.git(&["reset", "-q", "--hard", "HEAD~1"]);
            repo.git(&["commit", "--allow-empty", "-q", "-m", "rewritten"]);
        }
    }
    repo
}

fn launch(repo: &FixtureRepo, session: &str, rows: u16) -> (Terminal, tempfile::TempDir) {
    let config = tempfile::tempdir().unwrap();
    std::fs::write(
        config.path().join("config.toml"),
        "[ui]\ntheme = \"vscode-dark\"\n",
    )
    .unwrap();
    let term = common::launch(repo.root())
        .session(session)
        .config_dir(config.path())
        .size(170, rows)
        .no_commit_column()
        .no_exit_pause()
        .ready("base commit")
        .spawn();
    (term, config)
}

fn rgb(c: ThemeColor) -> Color {
    match c {
        ThemeColor::Rgb(r, g, b) => Color::Rgb(r, g, b),
        other => panic!("{other:?}"),
    }
}

/// The badge glyphs on screen, left to right: the Refs row's, then the Log
/// pill's.
fn badges(s: &Screen) -> Vec<(u16, u16)> {
    let mut found = s.find_all(GLYPH);
    found.sort();
    found
}

fn wait_badges(term: &Terminal, what: &str, count: usize) -> Screen {
    term.wait_for(what, STEP, |s| badges(s).len() >= count)
        .unwrap_or_else(|e| panic!("{e}"))
}

fn dialog_open(s: &Screen) -> bool {
    s.contains("upstream of feat") || s.contains("Remotes of feat")
}

#[test]
fn the_badge_shows_in_the_refs_row_and_the_log_pill_with_the_states_colour() {
    let spec = preset("vscode-dark").unwrap();
    let want = [
        (State::InSync, rgb(spec.status.success)),
        (State::Pending, rgb(spec.status.warning)),
        (State::Diverged, rgb(spec.status.danger)),
    ];
    let mut seen = Vec::new();
    for (i, (state, colour)) in want.into_iter().enumerate() {
        let repo = fixture(state);
        let (term, _config) = launch(&repo, &format!("pty-mr-pills-{i}"), 40);
        // The Log pill drops its badge first when the name and the other chips
        // already fill the Ref column: in sync, `+2` other remotes' tips share
        // the commit, so only the Refs row is guaranteed one.
        let pill_count = if state == State::InSync { 1 } else { 2 };
        let screen = wait_badges(&term, "the badges", pill_count);
        let found = badges(&screen);
        let (refs_col, refs_row) = found[0];
        let row_text = screen.line(refs_row);
        assert!(row_text.contains("feat"), "{row_text}");
        assert!(row_text.contains(&format!("{GLYPH}3")), "{row_text}");
        // The Refs row paints the glyph in the state's colour, the Log pill
        // puts it on a chip of that colour.
        let cell = screen.cell(refs_col, refs_row).unwrap();
        assert_eq!(cell.fg, colour, "Refs badge colour for state {i}: {row_text}");
        if let Some(&(log_col, log_row)) = found.get(1) {
            assert!(refs_col < log_col, "Refs row left of the Log pill: {found:?}");
            let pill = screen.cell(log_col, log_row).unwrap();
            assert_eq!(pill.bg, colour, "Log pill chip colour for state {i}");
        }
        seen.push(cell.fg);
        let _ = term.stop();
    }
    assert!(
        seen[0] != seen[1] && seen[1] != seen[2] && seen[0] != seen[2],
        "three states, three colours: {seen:?}"
    );
}

#[test]
fn the_badge_opens_the_branch_remotes_by_click_and_by_key() {
    let repo = fixture(State::Pending);
    let (mut term, _config) = launch(&repo, "pty-mr-pills-click", 40);
    let screen = wait_badges(&term, "the badges", 1);
    let (col, row) = badges(&screen)[0];
    // Pending: origin and gitlab are one ahead, backup has no copy.
    assert!(screen.line(row).contains(&format!("{GLYPH}3 \u{2191}1")));
    term.click(col, row).expect("click the badge");
    term.wait_for("the branch's remotes dialog", STEP, dialog_open)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Esc closes the combo's list first, then the dialog.
    common::retry_until(
        &mut term,
        "the dialog closed",
        4,
        Duration::from_secs(5),
        |t| t.send_keys("Escape").unwrap(),
        |s| !dialog_open(s),
    );

    // The keyboard twin: select the row (a click on the name), then `u`.
    let screen = term.screen();
    let (name_col, name_row) = screen
        .find_all("feat")
        .into_iter()
        .find(|(c, r)| *c < 30 && screen.line(*r).contains(GLYPH))
        .expect("the branch name in the Refs row");
    term.click(name_col, name_row).unwrap();
    term.send_keys("u").unwrap();
    term.wait_for("the dialog opened from the key", STEP, dialog_open)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let _ = term.stop();
}

/// The repository's saved extra push targets for new branches.
fn saved_targets(repo: &FixtureRepo) -> Vec<String> {
    gitpoulpi_session::RepoState::load(&repo.root().join(".git/gitpoulpi/repo-state.toml"))
        .map(|s| s.preferences.default_push_targets)
        .unwrap_or_default()
}

fn open_git_page(term: &mut Terminal) {
    let (c, r) = term
        .wait_for_text("\u{2699}", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(c, r).expect("click the gear");
    term.wait_for("Settings open", STEP, |s| {
        s.contains("Repo-specific settings")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    common::settings_page(term, "Git & encoding");
    term.wait_for("the Git page", STEP, |s| {
        s.contains("Extra push targets for new branches") && s.contains("gitlab")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

/// Clicks the row labelled `label` (right of the sidebar) and waits for its
/// checkbox to read `mark`.
fn click_row(term: &mut Terminal, label: &str, mark: &str) {
    let screen = term.screen();
    let (c, r) = screen
        .find_all(label)
        .into_iter()
        .find(|(c, _)| *c >= 28)
        .unwrap_or_else(|| panic!("no `{label}` row:\n{}", screen.text()));
    term.click(c, r).unwrap();
    term.wait_for(&format!("{label} reads {mark}"), STEP, |s| {
        s.find_all(label)
            .into_iter()
            .any(|(c, r)| c >= 28 && s.line(r).contains(mark))
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

#[test]
fn the_git_page_ticks_extra_push_targets_and_the_preflight_and_saves_them() {
    let repo = fixture(State::InSync);
    let (mut term, config) = launch(&repo, "pty-mr-pills-settings", 120);
    open_git_page(&mut term);

    // Per repository: one checkbox per remote, saved in the repo data.
    assert!(saved_targets(&repo).is_empty());
    click_row(&mut term, "gitlab", "[x]");
    click_row(&mut term, "backup", "[x]");
    let deadline = std::time::Instant::now() + STEP;
    // Saved in the order of the remotes (the push order), not of the clicks.
    while saved_targets(&repo) != ["backup", "gitlab"] {
        assert!(
            std::time::Instant::now() < deadline,
            "not saved: {:?}",
            saved_targets(&repo)
        );
        std::thread::sleep(Duration::from_millis(200));
    }
    click_row(&mut term, "gitlab", "[ ]");
    let deadline = std::time::Instant::now() + STEP;
    while saved_targets(&repo) != ["backup"] {
        assert!(
            std::time::Instant::now() < deadline,
            "not saved: {:?}",
            saved_targets(&repo)
        );
        std::thread::sleep(Duration::from_millis(200));
    }

    // Global: the preflight switch is on by default and writes config.toml.
    let stored = |c: &tempfile::TempDir| {
        std::fs::read_to_string(c.path().join("config.toml")).unwrap_or_default()
    };
    assert!(!stored(&config).contains("push_group_preflight"));
    click_row(&mut term, "Check every remote", "[ ]");
    let deadline = std::time::Instant::now() + STEP;
    while !stored(&config).contains("push_group_preflight = false") {
        assert!(
            std::time::Instant::now() < deadline,
            "not saved: {}",
            stored(&config)
        );
        std::thread::sleep(Duration::from_millis(200));
    }
    let _ = term.stop();
}
