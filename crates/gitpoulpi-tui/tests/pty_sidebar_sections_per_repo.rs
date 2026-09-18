//! The left pane's section choice is per repository: hiding TAGS in one repo
//! leaves another repo's TAGS alone (immediately, on a tab switch), the choice
//! survives a restart, "Use for all repositories" turns it into the global
//! default and "Reset to default" drops the repository's own choice. Real
//! binary, daemon and system git in a pty; temp config dirs only.

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(30);

fn repo(commit: &str, tag: &str) -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("f.txt", "f\n", commit)
        .unwrap()
        .tag(tag)
        .unwrap()
        .build()
}

fn launch(repo: &Path, config: &Path) -> Terminal {
    common::launch(repo)
        .implicit_session()
        .config_dir(config)
        .no_exit_pause()
        .size(200, 60)
        .ready("alpha-commit")
        .spawn()
}

fn exit(mut term: Terminal) {
    term.send_keys("q").unwrap();
    for _ in 0..100 {
        if term.exited() {
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    let _ = term.stop();
}

/// A left-pane section header on screen (not a checklist row).
fn has_header(screen: &Screen, name: &str) -> bool {
    screen
        .text()
        .lines()
        .any(|l| l.contains(name) && !l.contains('\u{2611}') && !l.contains('\u{2610}'))
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_default()
}

fn wait_file(path: &Path, what: &str, ok: impl Fn(&str) -> bool) {
    let deadline = std::time::Instant::now() + STEP;
    while !ok(&read(path)) {
        assert!(
            std::time::Instant::now() < deadline,
            "{what}: {}",
            read(path)
        );
        std::thread::sleep(Duration::from_millis(200));
    }
}

fn open_checklist(term: &mut Terminal) {
    let (c, r) = term
        .screen()
        .find("TAGS")
        .unwrap_or_else(|| panic!("no TAGS header\n{}", term.text()));
    term.right_click(c + 1, r).expect("right-click the header");
    term.wait_for("the checklist", STEP, |s| {
        s.contains("Use for all repositories") && s.contains("Reset to default")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

fn click_checkbox(term: &mut Terminal, name: &str) {
    let screen = term.screen();
    let (c, r) = screen
        .text()
        .lines()
        .enumerate()
        .find_map(|(r, l)| {
            l.find('\u{2611}').filter(|_| l.contains(name)).map(|byte| {
                let col = l[..byte].chars().count();
                (col as u16, r as u16)
            })
        })
        .unwrap_or_else(|| panic!("no checked {name} row\n{}", term.text()));
    term.click(c + 2, r).expect("click the checklist row");
}

fn switch_tab(term: &mut Terminal, to: &str, gone: &str) {
    for _ in 0..3 {
        term.send_text("\x1b[5;5~").unwrap();
        if term
            .wait_for("tab switched", Duration::from_secs(5), |s| {
                s.contains(to) && !s.contains(gone)
            })
            .is_ok()
        {
            return;
        }
    }
    panic!("never reached {to}\n{}", term.text());
}

#[test]
fn the_sections_choice_is_per_repository_and_survives_a_restart() {
    let alpha = repo("alpha-commit", "v-alpha");
    let bravo = repo("bravo-commit", "v-bravo");
    let config = tempfile::tempdir().unwrap();
    std::fs::write(
        config.path().join("recent.toml"),
        format!(
            "[[repos]]\npath = '{}'\nname = 'bravo-repo'\nlast_opened = 1\n",
            bravo.root().display()
        ),
    )
    .unwrap();
    let alpha_state = alpha.state_dir().join("repo-state.toml");
    let config_file = config.path().join("config.toml");

    let mut term = launch(alpha.root(), config.path());
    term.wait_for("TAGS", STEP, |s| has_header(s, "TAGS"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Hide TAGS in alpha only.
    open_checklist(&mut term);
    click_checkbox(&mut term, "TAGS");
    term.send_keys("Escape").unwrap();
    term.wait_for("TAGS gone in alpha", STEP, |s| {
        !has_header(s, "TAGS") && has_header(s, "LOCAL")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    wait_file(&alpha_state, "alpha's own choice", |t| {
        t.contains("sidebar_sections_hidden") && t.contains("tags")
    });
    assert!(
        !read(&config_file).contains("tags"),
        "a per-repo choice must not touch the global default: {}",
        read(&config_file)
    );

    // Bravo has no choice of its own: it shows TAGS, right away.
    term.send_keys("C-o").unwrap();
    term.wait_for("the management tab", STEP, |s| {
        s.contains("Repository Management") && s.contains("bravo-repo")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("/").unwrap();
    term.send_text("bravo").unwrap();
    term.wait_for("the filter applied", STEP, |s| {
        s.contains("Enter open best match")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("bravo open with TAGS", STEP, |s| {
        s.contains("bravo-commit") && has_header(s, "TAGS") && !s.contains("Repository Management")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Back to alpha: still hidden.
    switch_tab(&mut term, "alpha-commit", "bravo-commit");
    term.wait_for("alpha without TAGS", STEP, |s| {
        !has_header(s, "TAGS") && has_header(s, "LOCAL")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    exit(term);

    // A restart: alpha still hides TAGS.
    let mut term = launch(alpha.root(), config.path());
    term.wait_for("alpha without TAGS after a restart", STEP, |s| {
        !has_header(s, "TAGS") && has_header(s, "LOCAL")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // "Use for all repositories" writes the global default.
    term.send_keys("C-p").unwrap();
    term.send_text("Apply These Sections").unwrap();
    term.wait_for("the palette entry", STEP, |s| {
        s.contains("Apply These Sections to All Repositories")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    wait_file(&config_file, "the global default", |t| t.contains("tags"));

    // Reset drops alpha's own choice: it follows the (now TAGS-less) default.
    term.send_keys("C-p").unwrap();
    term.send_text("Reset Sections").unwrap();
    term.wait_for("the palette entry", STEP, |s| {
        s.contains("Reset Sections to Default")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    wait_file(&alpha_state, "alpha's own choice dropped", |t| {
        !t.contains("sidebar_sections_hidden")
    });
    term.wait_for("still no TAGS (the default hides it)", STEP, |s| {
        !has_header(s, "TAGS") && has_header(s, "LOCAL")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    exit(term);
}

/// `ui.sidebar_sections_scope = "global"`: one set for every repository. The
/// repository's own choice is ignored (but kept), a toggle writes the global
/// default, and the checklist drops "Use for all repositories". Switching the
/// setting back shows the repository's own choice again.
#[test]
fn the_global_scope_ignores_but_keeps_the_repository_choice() {
    let alpha = repo("alpha-commit", "v-alpha");
    let config = tempfile::tempdir().unwrap();
    let config_file = config.path().join("config.toml");
    std::fs::write(&config_file, "[ui]\nsidebar_sections_scope = \"global\"\n").unwrap();
    let alpha_state = alpha.state_dir().join("repo-state.toml");
    let own = gitpoulpi_session::RepoState {
        sidebar_sections_hidden: Some(vec!["remote".to_string()]),
        ..Default::default()
    };
    own.save(&alpha_state).unwrap();

    let mut term = launch(alpha.root(), config.path());
    term.wait_for("global: TAGS and REMOTE both shown", STEP, |s| {
        has_header(s, "TAGS") && has_header(s, "REMOTE")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    let (c, r) = term.screen().find("TAGS").expect("TAGS header");
    term.right_click(c + 1, r).unwrap();
    term.wait_for("the checklist", STEP, |s| s.contains("Reset to default"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(
        !term.text().contains("Use for all repositories"),
        "no per-repository scope to widen under global\n{}",
        term.text()
    );
    click_checkbox(&mut term, "TAGS");
    term.send_keys("Escape").unwrap();
    term.wait_for("TAGS gone", STEP, |s| {
        !has_header(s, "TAGS") && has_header(s, "REMOTE")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    wait_file(&config_file, "the global default", |t| t.contains("tags"));
    assert!(
        read(&alpha_state).contains("remote") && !read(&alpha_state).contains("tags"),
        "the repository's own choice is untouched: {}",
        read(&alpha_state)
    );
    exit(term);

    // Back to per repository: the own choice (REMOTE hidden) is intact.
    let config_text = read(&config_file).replace("\"global\"", "\"repository\"");
    std::fs::write(&config_file, config_text).unwrap();
    let term = launch(alpha.root(), config.path());
    term.wait_for("per repository: REMOTE hidden, TAGS shown", STEP, |s| {
        has_header(s, "TAGS") && !has_header(s, "REMOTE")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    exit(term);
}
