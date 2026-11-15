//! The Markdown composer through a pty by `xtask::pty`: the real binary and
//! daemon, the New issue form over the mock host.
//!
//! - typing Markdown in the description, switching to Preview (Alt+P, then
//!   the Write button) shows it rendered, not as typed;
//! - nothing typed there reaches the status line, in either mode;
//! - the discard question of the form still guards the text.
//!
//! Falsified: with the Preview painting the raw text (the `md_render::paint`
//! loop of `paint_preview` skipped) the rendered-text assertions go red.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::mock_host::flow::{self, FlowHost, FlowHostOptions};
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);

struct Rig {
    _repo: FixtureRepo,
    _host: FlowHost,
    term: Terminal,
}

/// A repository whose ISSUES source is the mock host (no network).
fn issue_rig(session: &str) -> Rig {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial commit")
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
        .size(200, 60)
        .no_exit_pause()
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GITPOULPI_TOKEN_H", "test-token")
        .mouse_capture()
        .spawn();
    std::mem::forget(config);
    Rig {
        _repo: repo,
        _host: host,
        term,
    }
}

fn open_issue_form(term: &mut Terminal) {
    term.wait_for("the log", STEP, |s| s.contains("initial commit"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let mut typed = false;
    common::retry_until(
        term,
        "the New issue form",
        30,
        Duration::from_secs(3),
        |t| {
            let s = t.screen();
            if s.contains("New issue") {
                return;
            }
            if s.contains("Command Palette") {
                if !typed {
                    t.send_text("Hosting: New Issue").unwrap();
                    typed = true;
                } else if s.contains("Hosting: New Issue...") {
                    t.send_keys("Enter").unwrap();
                    typed = false;
                }
            } else {
                typed = false;
                t.send_keys("C-p").unwrap();
            }
        },
        |s| s.contains("New issue"),
    );
}

#[test]
fn typed_markdown_is_shown_rendered_in_the_preview() {
    let mut rig = issue_rig("pty-md-editor");
    let term = &mut rig.term;
    open_issue_form(term);
    // Title, then Enter moves to the description.
    term.send_text("A title").unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("the Write / Preview buttons", STEP, |s| {
        s.contains("Write") && s.contains("Preview")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    for (i, line) in ["# Big heading", "plain **bold** text", "- item one"]
        .iter()
        .enumerate()
    {
        if i > 0 {
            term.send_keys("Enter").unwrap();
        }
        term.send_text(line).unwrap();
    }
    let typed = term
        .wait_for("the raw Markdown", STEP, |s| {
            s.contains("# Big heading") && s.contains("**bold**")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(
        !common::status_line_text(&typed).contains("Big heading"),
        "the typed text reached the status line:\n{}",
        typed.text()
    );
    // Alt+P: the same text, rendered.
    term.send_keys("M-p").unwrap();
    let rendered = term
        .wait_for("the rendered Preview", STEP, |s| {
            s.contains("Big heading") && !s.contains("# Big heading")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let text = rendered.text();
    assert!(text.contains("plain bold text"), "{text}");
    assert!(!text.contains("**bold**"), "{text}");
    assert!(text.contains("\u{2022} item one"), "{text}");
    assert!(
        !common::status_line_text(&rendered).contains("Big heading"),
        "{text}"
    );
    // The Write button (a real button, by mouse) goes back to the raw text.
    let (col, row) = rendered
        .find("Write")
        .unwrap_or_else(|| panic!("no Write button:\n{text}"));
    term.click(col + 1, row).unwrap();
    term.wait_for("the raw Markdown again", STEP, |s| {
        s.contains("# Big heading") && s.contains("**bold**")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // The form still guards the typed text.
    term.send_keys("Escape").unwrap();
    term.wait_for("the discard question", STEP, |s| {
        s.contains("Discard changes?") && s.contains("Keep editing")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}
