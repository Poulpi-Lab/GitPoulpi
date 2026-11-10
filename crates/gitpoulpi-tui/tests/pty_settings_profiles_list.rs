//! Settings > Profiles on the master-detail layout, end to end through a pty:
//! add two profiles from the list's toolbar, reorder with Alt+Down and with a
//! mouse drag, edit one, remove it behind the confirmation. `config.toml`
//! follows every step. Temp config dir only.

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);

fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial commit")
        .unwrap()
        .build()
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

fn open_settings(term: &mut Terminal) {
    let (c, r) = term
        .wait_for_text("\u{2699}", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(c, r).expect("click the gear");
    term.wait_for("Settings open", STEP, |s| {
        s.contains("Repo-specific settings")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

/// The profile ids in `config.toml`, in file order.
fn saved_ids(config: &Path) -> Vec<String> {
    std::fs::read_to_string(config.join("config.toml"))
        .unwrap_or_default()
        .lines()
        .filter_map(|l| l.strip_prefix("id = \""))
        .map(|l| l.trim_end_matches('"').to_string())
        .collect()
}

fn wait_for_ids(term: &Terminal, config: &Path, what: &str, want: &[&str]) {
    term.wait_for(what, STEP, |_| saved_ids(config) == want)
        .unwrap_or_else(|e| {
            panic!(
                "{e}\nwanted {want:?}, config has {:?}\n{}",
                saved_ids(config),
                term.text()
            )
        });
}

/// The list rows labelled `New profile` (the list pane, not the detail's
/// "Profile name" value), top to bottom.
fn list_rows(term: &Terminal) -> Vec<(u16, u16)> {
    let mut rows: Vec<(u16, u16)> = term
        .screen()
        .find_all("New profile")
        .into_iter()
        .filter(|(c, _)| *c < 66)
        .collect();
    rows.sort_by_key(|&(_, r)| r);
    rows
}

#[test]
fn profiles_are_added_reordered_dragged_and_removed_from_the_list() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), config.path(), "pty-profiles-list");
    open_settings(&mut term);
    common::settings_page(&mut term, "Profiles");
    common::settings_tab(&mut term, "Profiles");
    term.wait_for("the list and its detail", STEP, |s| {
        s.contains("+ Add") && s.contains("No profiles yet")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Two profiles through the toolbar button.
    for n in 1..=2 {
        let (c, r) = term
            .wait_for_text("+ Add", STEP)
            .unwrap_or_else(|e| panic!("{e}"));
        term.click(c + 1, r).unwrap();
        let want: Vec<&str> = ["profile", "profile-2"][..n].to_vec();
        wait_for_ids(&term, config.path(), "a profile added", &want);
    }
    term.wait_for("both rows listed", STEP, |_| list_rows(&term).len() == 2)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Select the first row and move it down with Alt+Down.
    let (c, r) = list_rows(&term)[0];
    term.click(c + 1, r).unwrap();
    term.send_keys("M-Down").unwrap();
    wait_for_ids(
        &term,
        config.path(),
        "moved down",
        &["profile-2", "profile"],
    );

    // Drag the first row onto the second: the order comes back. (Wait out the
    // double-click window of the earlier click on that row first.)
    std::thread::sleep(Duration::from_millis(900));
    term.wait_for("the list repainted", STEP, |_| list_rows(&term).len() == 2)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let rows = list_rows(&term);
    term.drag((rows[0].0 + 1, rows[0].1), (rows[1].0 + 1, rows[1].1))
        .unwrap();
    wait_for_ids(&term, config.path(), "dragged", &["profile", "profile-2"]);

    // Edit the selected profile so removing it asks, then remove it.
    let (c, r) = term
        .wait_for_text("Author email", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(c, r).unwrap();
    term.wait_for("the field open", STEP, |s| s.contains("\u{2588}"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_text("jo@example.test").unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("the email saved", STEP, |_| {
        std::fs::read_to_string(config.path().join("config.toml"))
            .unwrap_or_default()
            .contains("jo@example.test")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // The edited one is the second row (the drag selected it); its row now
    // shows the email instead of the warning.
    let (c, r) = rows[1];
    term.wait_for("the row shows the email", STEP, |s| {
        s.line(r).contains("jo@") && !s.line(r).contains("No email")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.click(c + 1, r).unwrap();
    term.send_keys("Delete").unwrap();
    confirm_list_removal(&mut term);
    wait_for_ids(&term, config.path(), "removed", &["profile"]);
}

/// Answers the list's own "Remove "<name>"?" question with its Remove button
/// (the one on the row of Cancel).
fn confirm_list_removal(term: &mut Terminal) {
    let screen = term
        .wait_for("the list's removal question with its buttons", STEP, |s| {
            s.contains("Remove \"New profile\"?")
                && s.find("Cancel")
                    .is_some_and(|(_, row)| s.find_all("Remove").into_iter().any(|(_, r)| r == row))
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (_, row) = screen.find("Cancel").unwrap();
    let (col, _) = screen
        .find_all("Remove")
        .into_iter()
        .find(|&(_, r)| r == row)
        .expect("a Remove button beside Cancel");
    term.click(col + 1, row).unwrap();
}
