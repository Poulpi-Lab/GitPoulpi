//! Branching workflows (plan `branching-workflows.md`, slice W3) through a
//! pty: the real binary, daemon and git, a temp config dir, no network.
//!
//! - Settings > Repository > Workflow: pick Git Flow, Detect, save the shared
//!   file (the file's content is asserted);
//! - the sidebar groups LOCAL by branch type and the `Flow ▾` button appears;
//! - the Start Feature form builds the right name and calls the fake
//!   `FlowActions` (`GITPOULPI_FLOW_FAKE=<log file>`; the daemon requests of
//!   slice W2 are wired separately).
//!
//! The rules are `pty_ui.rs`'s: nothing is located by a hard-coded
//! coordinate, nothing sleeps, and every wait describes the whole state it
//! is waiting for.

mod common;

use std::path::Path;
use std::time::{Duration, Instant};

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);
const ATTEMPT: Duration = Duration::from_secs(3);

fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial commit")
        .unwrap()
        .branch("develop")
        .unwrap()
        .branch("feature/login")
        .unwrap()
        .branch("feature/search")
        .unwrap()
        .branch("hotfix/1.0.1")
        .unwrap()
        .branch("wip-experiment")
        .unwrap()
        .tag("v1.4.2")
        .unwrap()
        .build()
}

const GIT_FLOW: &str = r#"template = "git-flow"

[[base]]
name = "main"
names = ["master"]
protected = true

[[base]]
name = "develop"
names = ["dev"]
parent = "main"
merge = "merge"

[[topic]]
id = "feature"
label = "Feature"
pattern = "feature/{slug}"
parent = "develop"
finish = [{ into = "develop", merge = "merge" }]
delete_after_finish = true

[[topic]]
id = "hotfix"
label = "Hotfix"
pattern = "hotfix/{version}"
parent = "main"
finish = [{ into = "main", merge = "merge", tag = "v{version}" }, { into = "develop", merge = "merge" }]
"#;

fn shared_file(repo: &Path) -> std::path::PathBuf {
    repo.join(".git").join("gitpoulpi").join("workflow.toml")
}

fn write_workflow(repo: &Path) {
    std::fs::create_dir_all(repo.join(".git").join("gitpoulpi")).unwrap();
    std::fs::write(shared_file(repo), GIT_FLOW).unwrap();
}

fn launch(repo: &FixtureRepo, session: &str, fake_log: Option<&Path>) -> Terminal {
    let config = tempfile::tempdir().expect("temp config dir");
    std::fs::write(
        config.path().join("config.toml"),
        "[ui]\ngraph_renderer = \"text\"\n",
    )
    .unwrap();
    let mut l = common::launch(repo.root())
        .session(session)
        .config_dir(config.path())
        .size(200, 60)
        .no_exit_pause()
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .mouse_capture();
    if let Some(log) = fake_log {
        l = l.env("GITPOULPI_FLOW_FAKE", log);
    }
    // The config dir lives as long as the test: leak it with the terminal.
    std::mem::forget(config);
    l.spawn()
}

fn palette(term: &mut Terminal, query: &str, entry: &str) {
    term.send_keys("C-p").unwrap();
    term.wait_for_text("Command Palette", STEP).unwrap();
    term.send_text(query).unwrap();
    term.wait_for("the palette entry", STEP, |s| s.contains(entry))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
}

fn wait_file(path: &Path, what: &str) -> String {
    let until = Instant::now() + xtask::pty::scaled(STEP);
    loop {
        if let Ok(text) = std::fs::read_to_string(path) {
            if !text.is_empty() {
                return text;
            }
        }
        assert!(
            Instant::now() < until,
            "{what}: {} never appeared",
            path.display()
        );
        std::thread::sleep(Duration::from_millis(100));
    }
}

#[test]
fn settings_workflow_pick_git_flow_detect_and_save_the_shared_file() {
    let repo = fixture();
    let mut term = launch(&repo, "pty-workflow-settings", None);
    term.wait_for("the log", STEP, |s| s.contains("initial commit"))
        .unwrap_or_else(|e| panic!("{e}"));

    // Settings on the Workflow page (a repo-specific one).
    let (col, row) = term.wait_for_text("\u{2699}", STEP).unwrap();
    term.click(col, row).unwrap();
    term.wait_for("Settings open", STEP, |s| {
        s.contains("Repo-specific settings")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    common::settings_page(&mut term, "Workflow");
    term.wait_for("the template picker", STEP, |s| {
        s.contains("Workflow template") && s.contains("Detect from the branches")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Pick Git Flow in the template dropdown (a combobox, never click-to-cycle).
    let (tc, tr) = common::wait_for_choice(&term, "Custom", STEP);
    term.click(tc + 1, tr).unwrap();
    term.wait_for("the template list", STEP, |s| {
        s.contains("GitHub Flow") && s.contains("Trunk-based")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // The list opens on the current template; Home goes to the first row,
    // Git Flow, and Enter picks it.
    term.send_keys("Home Enter").unwrap();
    term.wait_for("Git Flow picked", STEP, |s| {
        common::choice_at(s, "Git Flow").is_some()
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Its bases are rules, on the other tab; General again for the rest.
    common::settings_tab(&mut term, "Rules");
    term.wait_for("the template's bases listed", STEP, |s| {
        s.contains("develop") && s.contains("Base branches")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    common::settings_tab(&mut term, "General");
    assert!(
        !shared_file(repo.root()).exists(),
        "picking a template saves nothing"
    );

    // Detect: the daemon's report brings the branch names; the page shows the
    // score and the names it found. Idempotent, so repeated until the report
    // has arrived.
    let (dc, dr) = term
        .wait_for_text("Detect from the branches", STEP)
        .unwrap();
    let screen = term
        .act_until(
            "Detect's findings",
            ATTEMPT,
            STEP,
            |t| {
                // A button needs a selecting click, then the running one.
                t.click(dc + 1, dr)?;
                t.click(dc + 1, dr)
            },
            |s| s.contains("Closest template: Git Flow (score"),
        )
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(
        screen.contains("main, develop"),
        "the bases it found:\n{}",
        screen.text()
    );
    assert!(
        screen.contains("feature: 2 branches (feature/login, feature/search)"),
        "the topics it found:\n{}",
        screen.text()
    );

    // The save bar is pinned under the page: no scrolling to reach it.
    let screen = term
        .wait_for("the shared save button", STEP, |s| {
            s.contains("Save to the shared file")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (sc, sr) = screen.find("Save to the shared file").unwrap();
    term.click(sc + 1, sr).unwrap();
    term.click(sc + 1, sr).unwrap();
    let text = wait_file(&shared_file(repo.root()), "the shared workflow file");
    assert!(text.contains("template = \"git-flow\""), "{text}");
    assert!(text.contains("pattern = \"feature/{slug}\""), "{text}");
    assert!(text.contains("name = \"main\""), "{text}");
}

#[test]
fn the_sidebar_groups_local_branches_by_type_and_the_flow_button_appears() {
    let repo = fixture();
    write_workflow(repo.root());
    let mut term = launch(&repo, "pty-workflow-sidebar", None);
    let screen = term
        .wait_for("the grouped LOCAL section", STEP, |s| {
            s.contains("Feature (2)")
                && s.contains("Hotfix (1)")
                && s.contains("Other (1)")
                && s.contains("[base, protected]")
                && s.contains("develop  [base]")
                // The toolbar is the bottom row of the same frame: a snapshot
                // can hold the sidebar rows of a frame whose last rows have
                // not been parsed yet, so the button is part of "ready".
                && s.contains("Flow \u{25BE}")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Base branches come first, then the folders, in order.
    let main = screen.find("main  [base, protected]").unwrap().1;
    let develop = screen.find("develop  [base]").unwrap().1;
    let feature = screen.find("Feature (2)").unwrap().1;
    let hotfix = screen.find("Hotfix (1)").unwrap().1;
    let other = screen.find("Other (1)").unwrap().1;
    assert!(
        main < develop && develop < feature && feature < hotfix && hotfix < other,
        "{}",
        screen.text()
    );
    // The members are under their folder.
    assert!(screen.contains("feature/login") && screen.contains("wip-experiment"));
    // The toolbar has the Flow button (only with a workflow).
    assert!(screen.contains("Flow \u{25BE}"), "{}", screen.text());

    // Collapsing a group hides its members (a click on the folder row).
    let (fc, fr) = screen.find("Feature (2)").unwrap();
    term.click(fc + 2, fr).unwrap();
    term.wait_for("the Feature group collapsed", STEP, |s| {
        s.contains("Feature (2)") && !s.contains("feature/login")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // The toggle (a palette command, also in the Flow menu) gives the plain
    // tree back, and again the groups.
    palette(&mut term, "Toggle Grouped", "Flow: Toggle Grouped Branches");
    term.wait_for("the plain tree", STEP, |s| {
        !s.contains("Feature (2)") && s.contains("feature")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    palette(&mut term, "Toggle Grouped", "Flow: Toggle Grouped Branches");
    term.wait_for("the groups again", STEP, |s| s.contains("Feature (2)"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

#[test]
fn the_start_feature_form_builds_the_name_and_calls_the_fake() {
    let repo = fixture();
    write_workflow(repo.root());
    let log = tempfile::NamedTempFile::new().unwrap();
    let log_path = log.path().to_path_buf();
    std::fs::write(&log_path, "").unwrap();
    let mut term = launch(&repo, "pty-workflow-start", Some(&log_path));
    let screen = term
        .wait_for("the Flow button", STEP, |s| s.contains("Flow \u{25BE}"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // The dropdown: one Start row per type, then the fixed rows.
    let (bc, br) = screen.find("Flow \u{25BE}").unwrap();
    term.act_until(
        "the Flow dropdown",
        ATTEMPT,
        STEP,
        |t| t.click(bc + 1, br),
        |s| s.contains("Start Feature...") && s.contains("Finish current branch..."),
    )
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let screen = term.screen();
    for want in [
        "Start Feature...",
        "Start Hotfix...",
        "Sync with parent...",
        "Set parent...",
        "Group branches by type",
        "Workflow settings...",
    ] {
        assert!(screen.contains(want), "{want}:\n{}", screen.text());
    }
    let (rc, rr) = screen.find("Start Feature...").unwrap();
    term.click(rc + 2, rr).unwrap();

    // The form: the type, the slug field with the focus, the start point.
    term.wait_for("the Start Feature form", STEP, |s| {
        s.contains("Slug") && s.contains("Start from") && s.contains("develop")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_text("Add a login page").unwrap();
    term.wait_for("the name preview", STEP, |s| {
        s.contains("feature/add-a-login-page")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Enter previews (through the fake), Enter again runs it.
    term.send_keys("Enter").unwrap();
    term.wait_for("the step list", STEP, |s| {
        s.contains("This will:") && s.contains("Create the branch")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    term.wait_for("the run done", STEP, |s| {
        s.contains("[x] Create the branch") && s.contains("[x] Check it out")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // What the fake was asked.
    let text = wait_file(&log_path, "the fake's call log");
    let calls: Vec<serde_json::Value> = text
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    let execute = calls
        .iter()
        .find(|c| c["call"] == "execute")
        .unwrap_or_else(|| panic!("no execute call: {text}"));
    assert_eq!(execute["op"], "start");
    assert_eq!(execute["type"], "feature");
    assert_eq!(execute["name"], "feature/add-a-login-page");
    assert_eq!(execute["start_point"], "develop");
    assert_eq!(execute["publish"], false);
    assert_eq!(execute["run_hooks"], false);
    assert!(
        calls.iter().any(|c| c["call"] == "preview"),
        "the preview came first: {text}"
    );
}

#[test]
fn the_palette_twin_of_start_feature_opens_the_same_form() {
    let repo = fixture();
    write_workflow(repo.root());
    let mut term = launch(&repo, "pty-workflow-palette", None);
    term.wait_for("the Flow button", STEP, |s| s.contains("Flow \u{25BE}"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    palette(&mut term, "Start Feature", "Flow: Start Feature...");
    term.wait_for("the Start Feature form", STEP, |s| {
        s.contains("Slug") && s.contains("Start from")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Esc closes it.
    term.send_keys("Esc").unwrap();
    term.wait_for_text_to_go("Start from", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
}
