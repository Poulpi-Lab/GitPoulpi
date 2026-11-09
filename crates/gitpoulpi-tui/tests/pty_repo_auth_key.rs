//! The SSH & credentials page, end to end through a pty: the palette opens it,
//! a key file is picked from the home folder's `.ssh`, Save stores the
//! profile through the daemon, and Test connection over a stub `ssh` (a
//! local directory standing in for the server, no network) shows the key
//! the server accepted in the results panel. The results panel's scrollbar
//! is dragged with the mouse; Esc closes it.
//!
//! Nothing is located by a hard-coded coordinate. The home folder, the
//! global git config naming the stub and the config directory are all
//! temporary: never the developer's own.
#![cfg(unix)]

mod common;

use std::path::Path;
use std::time::{Duration, Instant};

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder, StubSsh};
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(40);

struct Rig {
    repo: FixtureRepo,
    stub: StubSsh,
    home: tempfile::TempDir,
    gitconfig: std::path::PathBuf,
}

fn rig(remotes: usize) -> Rig {
    let mut builder = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial commit")
        .unwrap()
        .with_origin()
        .unwrap()
        .push("main")
        .unwrap();
    for i in 1..remotes {
        builder = builder.with_remote(&format!("extra{i}")).unwrap();
    }
    let repo = builder.build();
    let bare = repo.remote_path().unwrap().to_path_buf();
    repo.git(&[
        "remote",
        "set-url",
        "origin",
        &StubSsh::url_for(&bare),
    ]);
    for i in 1..remotes {
        repo.git(&[
            "remote",
            "set-url",
            &format!("extra{i}"),
            &StubSsh::url_for(&bare),
        ]);
    }
    let stub = StubSsh::new();
    let home = tempfile::tempdir().unwrap();
    let ssh = home.path().join(".ssh");
    std::fs::create_dir_all(&ssh).unwrap();
    std::fs::write(ssh.join("id_work"), "not a real key\n").unwrap();
    std::fs::write(ssh.join("id_work.pub"), "ssh-ed25519 AAAA work\n").unwrap();
    // The program a key file choice builds on is the global `core.sshCommand`.
    let gitconfig = home.path().join("gitconfig");
    std::fs::write(
        &gitconfig,
        format!(
            "[core]\n\tsshCommand = \"{}\"\n[user]\n\tname = T\n\temail = t@example.com\n",
            stub.command().replace('\\', "\\\\").replace('"', "\\\"")
        ),
    )
    .unwrap();
    Rig {
        repo,
        stub,
        home,
        gitconfig,
    }
}

fn launch(rig: &Rig, session: &str) -> (Terminal, tempfile::TempDir) {
    common::launch(rig.repo.root())
        .session(session)
        .size(200, 90)
        .env("HOME", rig.home.path())
        .env("GIT_CONFIG_GLOBAL", &rig.gitconfig)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .no_exit_pause()
        .ready_when("the fixture", |s| s.contains("initial commit"))
        .mouse_capture()
        .start()
}

fn wait(term: &Terminal, what: &str, ok: impl FnMut(&xtask::pty::Screen) -> bool) {
    term.wait_for(what, STEP, ok)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

/// Opens the page from the palette (the keyboard twin of the sidebar entry).
fn open_page(term: &mut Terminal) {
    common::retry_until(
        term,
        "the SSH & credentials page",
        4,
        Duration::from_secs(10),
        |t| {
            t.send_keys("C-p").unwrap();
            t.send_text("ssh & credentials").unwrap();
            t.send_keys("Enter").unwrap();
        },
        |s| s.contains("SSH & credentials - repo-specific"),
    );
}

/// Every column of `row` that draws `glyph`.
fn glyph_cols(s: &xtask::pty::Screen, row: u16, glyph: &str) -> Vec<u16> {
    (0..s.cols())
        .filter(|&c| s.cell(c, row).is_some_and(|cell| cell.display() == glyph))
        .collect()
}

/// `(first row, last row, bar column)` of the results panel's bar: the up
/// arrow below the title and the down arrow, in the same column, further down.
fn bar_ends(s: &xtask::pty::Screen) -> Option<(u16, u16, u16)> {
    let (_, title) = s.find("Connection test")?;
    let first = (title + 1..s.rows()).find(|&r| !glyph_cols(s, r, "\u{2191}").is_empty())?;
    let col = *glyph_cols(s, first, "\u{2191}").last()?;
    let last = (first + 1..s.rows()).find(|&r| glyph_cols(s, r, "\u{2193}").contains(&col))?;
    Some((first, last, col))
}

fn click_button(term: &mut Terminal, label: &str) {
    let needle = format!(" {label} ");
    let (col, row) = common::scroll_settings_to(term, &needle, STEP);
    term.click(col + 1, row).unwrap();
}

fn profile_file(config: &Path) -> String {
    let paths = gitpoulpi_config::ConfigPaths::for_dir(config);
    std::fs::read_to_string(paths.private_dir().join("repo-auth.toml")).unwrap_or_default()
}

#[test]
fn a_picked_key_is_saved_and_the_connection_test_names_the_accepted_key() {
    let rig = rig(1);
    let (mut term, config) = launch(&rig, "pty-repo-auth-key");
    open_page(&mut term);
    // The page has read the repository's remotes (the rows below do not
    // shift while the dropdowns are used).
    wait(&term, "the page loaded", |s| {
        s.contains("origin") && s.contains("system ssh (agent") && !s.contains("Reading the sign-in")
    });

    // SSH: System -> Key file, through the dropdown.
    let (col, row) = common::scroll_settings_to_choice(&mut term, "System (ssh's own setup)", STEP);
    term.click(col + 1, row).unwrap();
    wait(&term, "the SSH choices", |s| s.contains("Account key"));
    common::click_text_until(&mut term, "Key file", "the key file row", |s| {
        s.contains("Private key file")
    });

    // The key combo lists the keys of ~/.ssh: pick id_work.
    let key = rig.home.path().join(".ssh/id_work");
    let key_text = key.to_string_lossy().into_owned();
    let (_, r) = common::scroll_settings_to(&mut term, "Private key file", STEP);
    // The empty value sits on the label's row.
    let (c, r) = term
        .screen()
        .find_all("(not set)")
        .into_iter()
        .find(|(_, row)| *row == r)
        .expect("the key combo's value");
    term.click(c + 1, r).unwrap();
    wait(&term, "the key list", |s| s.contains("id_work"));
    // A searchable list: a click selects the entry, Enter (or a double
    // click) picks it.
    let (c, r) = term.screen().find("id_work").unwrap();
    term.click(c + 1, r).unwrap();
    std::thread::sleep(Duration::from_millis(400));
    term.send_keys("Enter").unwrap();
    wait(&term, "the key in the field", |s| {
        // The label's row now shows the key, and the list is closed.
        s.find("Private key file")
            .is_some_and(|(_, row)| s.line(row).contains("id_work"))
            && s.find_all("id_work").len() == 1
    });

    // Save: the page says so, and the private file holds the key.
    click_button(&mut term, "Save");
    wait(&term, "the saved status", |s| {
        s.contains("Saved. This repository now signs in as chosen.")
    });
    let deadline = Instant::now() + STEP;
    while !profile_file(config.path()).contains(&key_text) {
        assert!(
            Instant::now() < deadline,
            "the profile never reached the private file: {}",
            profile_file(config.path())
        );
        std::thread::sleep(Duration::from_millis(100));
    }

    // Test connection: both checks run over the stub and the results name the key.
    click_button(&mut term, "Test connection");
    let screen = term
        .wait_for("the results panel", STEP, |s| s.contains("Connection test"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(screen.contains("accepted key"), "{}", term.text());
    assert!(screen.contains(&key_text) || screen.contains("id_work"), "{}", term.text());
    assert!(screen.contains("Fetch: ok") && screen.contains("Push: ok"), "{}", term.text());
    assert!(
        rig.stub.identity_files().iter().any(|f| *f == key_text),
        "ssh was run with the chosen key: {:?}",
        rig.stub.identity_files()
    );
    // Esc closes the panel.
    term.send_keys("Escape").unwrap();
    wait(&term, "the panel closed", |s| !s.contains("Connection test"));
}

#[test]
fn the_results_scrollbar_drags_and_the_cross_closes_the_panel() {
    // Enough remotes that the results do not fit one screen.
    let rig = rig(14);
    let (mut term, _config) = common::launch(rig.repo.root())
        .session("pty-repo-auth-results")
        .size(120, 30)
        .env("HOME", rig.home.path())
        .env("GIT_CONFIG_GLOBAL", &rig.gitconfig)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .no_exit_pause()
        .ready_when("the fixture", |s| s.contains("initial commit"))
        .mouse_capture()
        .start();
    common::retry_until(
        &mut term,
        "the results panel",
        4,
        Duration::from_secs(20),
        |t| {
            t.send_keys("C-p").unwrap();
            t.send_text("test connection").unwrap();
            t.send_keys("Enter").unwrap();
        },
        |s| s.contains("Connection test"),
    );
    // The panel is long: it has a scrollbar (arrows at both ends).
    let screen = term.screen();
    assert!(screen.contains("Fetch: ok"), "{}", term.text());
    // Keys scroll the pane.
    term.send_keys("PageDown").unwrap();
    wait(&term, "the pane scrolled by PageDown", |s| {
        !s.contains("Tested with the saved choices.")
    });
    term.send_keys("Home").unwrap();
    wait(&term, "back at the top", |s| {
        s.contains("Tested with the saved choices.")
    });
    // The wheel scrolls it too.
    let (c, r) = term.screen().find("Tested with the saved choices.").unwrap();
    term.scroll(true, c + 2, r).unwrap();
    wait(&term, "the pane scrolled by the wheel", |s| {
        !s.contains("Tested with the saved choices.")
    });
    term.send_keys("Home").unwrap();
    wait(&term, "back at the top again", |s| {
        s.contains("Tested with the saved choices.")
    });
    // Drag the thumb (right under the up arrow) past the bottom.
    let start = term.screen();
    let (first, last, col) = bar_ends(&start).expect("the results bar");
    term.drag((col, first + 1), (col, last + 2)).unwrap();
    wait(&term, "the drag scrolled the pane", |s| {
        !s.contains("Tested with the saved choices.")
    });
    assert!(
        term.screen().contains("origin  (ssh,"),
        "dragged to the end: {}",
        term.text()
    );
    // The close cross.
    // (Settings has its own cross on the top row: the panel's is on the row
    // of its title.)
    let screen = term.screen();
    let (_, title_row) = screen.find("Connection test").expect("the panel title");
    let cross = screen
        .find_all("\u{2715}")
        .into_iter()
        .find(|(_, r)| *r == title_row)
        .expect("the panel's cross");
    term.click(cross.0, cross.1).unwrap();
    wait(&term, "the panel closed", |s| !s.contains("Connection test"));
}
