//! The Activity Log's Scope box: opened from a repository tab it shows that
//! repository's operations ("This repository"), opened from Home it shows
//! "All repositories"; the box (a combobox) changes the scope by mouse and by
//! key, for both tabs, and the Application tab's lines follow it.
//!
//! Real binary, daemon and system git through a pty; the failing remote is a
//! closed local port (no network), temp config dirs only.

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::{Screen, Terminal};

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);

struct Setup {
    term: Terminal,
    repo: FixtureRepo,
    name: String,
    _config: tempfile::TempDir,
    _root: tempfile::TempDir,
}

/// A repository whose `origin` cannot be reached, with an unpushed commit, in
/// a window of its own.
fn launch(session: &str) -> Setup {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("scope base")
        .unwrap()
        .with_origin()
        .unwrap()
        .push("main")
        .unwrap()
        .track("main", "origin")
        .unwrap()
        .commit("scope unpushed")
        .unwrap()
        .build();
    repo.git(&["remote", "set-url", "origin", "http://127.0.0.1:1/r.git"]);
    let name = repo
        .root()
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    let root = tempfile::tempdir().unwrap();
    let config = tempfile::tempdir().unwrap();
    std::fs::write(
        config.path().join("config.toml"),
        format!("[repos]\nclone_root = '{}'\n", root.path().display()),
    )
    .unwrap();
    let term = common::launch(Path::new(repo.root()))
        .session(session)
        .config_dir(config.path())
        .size(200, 50)
        .no_exit_pause()
        .mouse_capture()
        .env("GITPOULPI_CLIPBOARD", "memory")
        .startup(STARTUP)
        .ready("scope unpushed")
        .spawn();
    Setup {
        term,
        repo,
        name,
        _config: config,
        _root: root,
    }
}

fn click_push(term: &mut Terminal) {
    let push = term.wait_for_text("\u{21E7} Push", STEP).unwrap();
    term.click(push.0 + 2, push.1).expect("click Push");
    term.wait_for("the failure in the status line", STEP, |s| {
        s.contains("Push failed")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

fn open_activity(term: &mut Terminal) {
    // Under load a lone Ctrl+N can be lost (read together with the Esc
    // before it, the pair is one Alt chord), and an "A" sent before the
    // chord prompt is up is lost too: retry until the prompt shows.
    let mut up = false;
    for _ in 0..4 {
        term.send_keys("C-n").unwrap();
        if term
            .wait_for("the Ctrl+N chord prompt", Duration::from_secs(5), |s| {
                s.contains("t=theme")
            })
            .is_ok()
        {
            up = true;
            break;
        }
    }
    assert!(
        up,
        "the Ctrl+N chord prompt never came up:\n{}",
        term.text()
    );
    term.send_text("A").unwrap();
}

/// The Scope row of the window: `Scope:  <what it says> ...`.
fn scope_row(s: &Screen) -> Option<String> {
    s.text()
        .lines()
        .find(|l| l.contains("Scope:"))
        .map(str::to_string)
}

fn scope_says(s: &Screen, text: &str) -> bool {
    scope_row(s).is_some_and(|r| r.contains(text))
}

/// The row and text of the box's list row for `name`: under the Scope row,
/// the row text the list paints.
fn list_row(s: &Screen, name: &str) -> Option<(u16, String)> {
    let text = s.text();
    let lines: Vec<&str> = text.lines().collect();
    let scope = lines.iter().position(|l| l.contains("Scope:"))?;
    lines
        .iter()
        .enumerate()
        .skip(scope + 1)
        .find(|(_, l)| l.contains(name))
        .map(|(i, l)| (i as u16, (*l).to_string()))
}

/// A failed clone from Home: a repository the repository tab does not know.
fn failing_clone(term: &mut Terminal, name: &str) {
    term.send_keys("C-o").expect("the management tab");
    term.wait_for("the management tab", STEP, |s| {
        s.contains("Repository Management") && s.contains("Clone")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("c").expect("Clone");
    term.wait_for("the Clone dialog", STEP, |s| {
        s.contains("Clone a repository") && s.contains("Clone with URL")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_text(&format!("git://127.0.0.1:1/{name}.git"))
        .expect("the URL");
    let n = name.to_string();
    term.wait_for("the full path", STEP, move |s| {
        s.contains("Full path:") && s.contains(&n)
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("the failure in the status line", STEP, |s| {
        s.contains("Clone failed")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

/// From a repository tab: "This repository (name)" and its operations (the
/// failed push); by mouse the box lists the choices and "All repositories"
/// replaces it; by key `s`, Down, Enter does the same the other way round;
/// the box is on the Application tab too, and the lines the daemon and the
/// window wrote about this repository show there, app-wide ones marked.
#[test]
fn opened_from_a_repository_tab_it_is_scoped_to_that_repository_and_the_box_switches() {
    let mut s = launch("pty-scope-repo");
    click_push(&mut s.term);
    s.term.send_keys("Escape").unwrap();
    s.term
        .wait_for("the status line cleared by Esc", STEP, |x| {
            !x.contains("Push failed")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    open_activity(&mut s.term);
    let this = format!("This repository ({})", s.name);
    let t = this.clone();
    let screen = s
        .term
        .wait_for("the log scoped to this repository", STEP, move |x| {
            scope_says(x, &t) && x.contains("Git operations") && x.contains("Push failed")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    assert!(!screen.contains(" Repository  "), "{}", screen.text());

    // The Application tab keeps the scope: this repository's lines, and the
    // app-wide ones marked.
    s.term.send_keys("Tab").unwrap();
    let screen = s
        .term
        .wait_for("the application log, scoped", STEP, |x| {
            x.contains("Level:") && x.contains("Push failed (") && x.contains("[app-wide]")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    assert!(scope_says(&screen, &this), "{}", screen.text());
    s.term.send_keys("Tab").unwrap();

    // By mouse: a click opens the list (it never cycles), a click on the
    // row picks it.
    let (col, row) = s.term.screen().find("Scope:").unwrap();
    s.term.click(col + 12, row).unwrap();
    let screen = s
        .term
        .wait_for("the scope list", STEP, |x| {
            x.text()
                .lines()
                .any(|l| l.contains("All repositories") && !l.contains("Scope:"))
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    assert!(scope_says(&screen, &this), "a click only opens the list");
    let (col, row) = screen
        .find_all("All repositories")
        .into_iter()
        .find(|(_, r)| *r != screen.find("Scope:").unwrap().1)
        .expect("the All repositories row");
    s.term.click(col + 1, row).unwrap();
    s.term
        .wait_for("all repositories", STEP, |x| {
            scope_says(x, "All repositories") && !scope_says(x, "This repository")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));

    // By key: `s` opens the list on the current row, Up reaches the first
    // (this repository), Enter picks it.
    s.term.send_text("s").unwrap();
    s.term
        .wait_for("the list by key", STEP, |x| {
            x.find_all("All repositories").len() >= 2
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    s.term.send_keys("Up").unwrap();
    s.term.send_keys("Enter").unwrap();
    s.term
        .wait_for("back to this repository", STEP, |x| scope_says(x, &this))
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    s.term.send_keys("Escape").unwrap();
    s.term.stop().unwrap();
}

/// From Home there is no repository of its own: "All repositories", and no
/// "This repository" row in the box.
#[test]
fn opened_from_home_it_shows_all_repositories() {
    let mut s = launch("pty-scope-home");
    s.term.send_keys("C-o").unwrap();
    s.term
        .wait_for("Home", STEP, |x| x.contains("Repository Management"))
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    open_activity(&mut s.term);
    let screen = s
        .term
        .wait_for("the log on all repositories", STEP, |x| {
            scope_says(x, "All repositories") && x.contains("Git operations")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    assert!(!screen.contains("This repository"), "{}", screen.text());
    s.term.send_text("s").unwrap();
    let screen = s
        .term
        .wait_for("the list", STEP, |x| {
            x.find_all("All repositories").len() >= 2
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    assert!(
        !screen.contains("This repository"),
        "Home has no such row:\n{}",
        screen.text()
    );
    s.term.send_keys("Escape").unwrap();
    s.term.stop().unwrap();
}

/// The Application tab's lines follow the scope: a push failure of the
/// repository and a clone failure of another one are both there under "All
/// repositories", and picking the repository drops the other's lines.
#[test]
fn the_application_tab_is_filtered_by_the_scope() {
    let mut s = launch("pty-scope-app");
    click_push(&mut s.term);
    s.term.send_keys("Escape").unwrap();
    s.term
        .wait_for("the status line cleared by Esc", STEP, |x| {
            !x.contains("Push failed")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    failing_clone(&mut s.term, "elsewhere-gone");
    // The status line says it too: Esc clears it.
    s.term.send_keys("Escape").unwrap();
    s.term
        .wait_for("the status line cleared by Esc", STEP, |x| {
            !x.contains("Clone failed")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    open_activity(&mut s.term);
    // The window in front is still the repository's tab: "This repository".
    s.term
        .wait_for("the log on this repository", STEP, |x| {
            scope_says(x, "This repository") && x.contains("Git operations")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    s.term.send_text("s").unwrap();
    s.term
        .wait_for("the list", STEP, |x| {
            !x.find_all("All repositories").is_empty()
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    s.term.send_keys("Down").unwrap();
    s.term.send_keys("Enter").unwrap();
    s.term
        .wait_for("the log on all repositories", STEP, |x| {
            scope_says(x, "All repositories")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    s.term.send_keys("Tab").unwrap();
    let name = s.name.clone();
    s.term
        .wait_for("both repositories' lines", STEP, move |x| {
            x.contains("Level:")
                && x.contains(&format!("[{name}] Push failed ("))
                && x.contains("[elsewhere-gone] Clone failed")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));

    // Pick the repository by its row in the list (below the Scope row: the
    // window's tab bar behind also says the name).
    s.term.send_text("s").unwrap();
    let screen = s
        .term
        .wait_for("the scope list", STEP, |x| {
            list_row(x, "This repository (").is_some()
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    let (row, line) = list_row(&screen, "This repository (").expect("the row");
    let col = line.find("This repository (").unwrap();
    let col = line[..col].chars().count() as u16;
    s.term.click(col + 1, row).unwrap();
    let name = format!("This repository ({})", s.name);
    let screen = s
        .term
        .wait_for("only the repository's lines", STEP, move |x| {
            scope_says(x, &name)
                && x.contains("Push failed (")
                && !x.contains("Clone failed")
                && x.contains("[app-wide]")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    assert!(!screen.contains("[elsewhere-gone]"), "{}", screen.text());
    let _ = &s.repo;
    s.term.send_keys("Escape").unwrap();
    s.term.stop().unwrap();
}
