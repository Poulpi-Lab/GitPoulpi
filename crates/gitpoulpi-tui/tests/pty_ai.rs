#![cfg(feature = "ai")]
//! AI end to end, driven through a pty by `xtask::pty`: the real binary,
//! the real daemon, real `git`, and a **fake local agent** (a script that
//! prints a canned answer) standing in for the model -- no socket is ever
//! opened.
//!
//! The rules are `pty_ui.rs`'s (see its header and
//! `docs/developer/visual-verification.md`): nothing is located by a
//! hard-coded coordinate, nothing sleeps, and every wait describes the
//! whole state it is waiting for.

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::fake_agent::{
    agent_calls, ai_config_toml, write_fake_agent, write_slow_fake_agent,
};
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);

/// One commit and one staged edit, so there is something to describe.
fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial commit")
        .unwrap()
        .write_worktree("a.txt", b"one\ntwo\n")
        .unwrap()
        .stage("a.txt")
        .unwrap()
        .build()
}

/// Launches the TUI with `config_toml` as its `config.toml` (or none).
fn launch(
    repo: &FixtureRepo,
    session: &str,
    config_toml: Option<String>,
) -> (Terminal, tempfile::TempDir) {
    common::launch(repo.root())
        .session(session)
        .config(config_toml.as_deref().unwrap_or_default())
        .size(200, 50)
        .ready_when("the fixture loaded, WIP row and all", |s| {
            s.contains("initial commit")
                && s.contains("// WIP")
                && s.contains("Amend previous commit")
        })
        .start()
}

/// The Generate sparkle: the `\u{2726}` on the commit message's Summary row.
fn generate_pos(s: &xtask::pty::Screen) -> Option<(u16, u16)> {
    s.find_all("\u{2726}")
        .into_iter()
        .find(|&(_, r)| s.line(r).contains("Summary"))
}

fn wait_generate(term: &Terminal) -> (u16, u16) {
    let screen = term
        .wait_for("the Generate sparkle on the Summary row", STEP, |s| {
            generate_pos(s).is_some()
        })
        .unwrap_or_else(|e| panic!("{e}"));
    generate_pos(&screen).unwrap()
}

fn ai_config(agent: &Path) -> Option<String> {
    Some(ai_config_toml(agent))
}

/// With AI off - the default - nothing on screen mentions AI: no
/// composer button, no palette entry (see `pty_ai_off.rs` for the sweep).
#[test]
fn with_ai_off_no_entry_point_is_visible() {
    let repo = fixture();
    let (mut term, _config) = launch(&repo, "pty-ai-off", None);
    assert!(
        generate_pos(&snap(&term)).is_none(),
        "no Generate sparkle while AI is off"
    );

    term.send_keys("C-p").expect("open the palette");
    term.send_text("generate").expect("type a query");
    let screen = term
        .wait_for("the query echoed in the palette", STEP, |s| {
            s.contains("generate")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        !screen.contains("AI: Generate Commit Message"),
        "the palette entry is hidden while AI is off"
    );
}

/// The Generate button fills the composer from the agent's answer; nothing
/// is committed until the user commits; the commit carries that message.
#[test]
fn the_generate_button_fills_the_composer_and_the_commit_uses_it() {
    let repo = fixture();
    let tools = tempfile::tempdir().unwrap();
    let agent = write_fake_agent(
        tools.path(),
        &["feat: add two", "", "Adds a second line to a.txt."],
    );
    let (mut term, _config) = launch(&repo, "pty-ai-button", ai_config(&agent));

    let (col, row) = wait_generate(&term);
    term.click(col + 1, row).expect("click Generate");

    term.wait_for("the composer filled from the agent", STEP, |s| {
        s.contains("feat: add two") && s.contains("Adds a second line to a.txt.")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(agent_calls(&agent), 1);
    assert_eq!(
        repo.git(&["log", "-1", "--format=%s"]),
        "initial commit",
        "generating a message commits nothing"
    );

    let (col, row) = term
        .wait_for_text("Commit changes to 1 file", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(col + 1, row).expect("click Commit");
    let deadline = std::time::Instant::now() + STEP;
    loop {
        if repo.git(&["log", "-1", "--format=%s"]) == "feat: add two" {
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "the commit never carried the generated message"
        );
        std::thread::sleep(Duration::from_millis(200));
    }
}

/// The keyboard twins of the button: the palette entry and `Ctrl+G i`.
#[test]
fn the_palette_and_the_leader_chord_generate_too() {
    let repo = fixture();
    let tools = tempfile::tempdir().unwrap();
    let agent = write_fake_agent(tools.path(), &["fix: from the palette"]);
    let (mut term, _config) = launch(&repo, "pty-ai-keys", ai_config(&agent));

    term.send_keys("C-p").expect("open the palette");
    term.send_text("generate commit").expect("type a query");
    term.wait_for("the AI entry offered", STEP, |s| {
        s.contains("AI: Generate Commit Message")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").expect("run it");
    term.wait_for("the composer filled by the palette", STEP, |s| {
        s.contains("fix: from the palette") && !s.contains("AI: Generate Commit Message")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(agent_calls(&agent), 1);

    // Again by chord: the answer is the same, the agent ran twice.
    term.send_keys("C-g").expect("Ctrl+G");
    term.send_keys("i").expect("i");
    let deadline = std::time::Instant::now() + STEP;
    while agent_calls(&agent) < 2 {
        assert!(
            std::time::Instant::now() < deadline,
            "Ctrl+G i never reached the agent"
        );
        std::thread::sleep(Duration::from_millis(200));
    }
}

/// Esc stops a running request and leaves the composer untouched.
#[test]
fn escape_cancels_a_running_request() {
    let repo = fixture();
    let tools = tempfile::tempdir().unwrap();
    let agent = write_slow_fake_agent(tools.path(), 20, &["feat: too late"]);
    let (mut term, _config) = launch(&repo, "pty-ai-cancel", ai_config(&agent));

    term.send_keys("C-g").expect("Ctrl+G");
    term.send_keys("i").expect("i");
    term.wait_for("the request running", STEP, |s| s.contains("working: "))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Escape").expect("Esc");
    let screen = term
        .wait_for("the cancellation reported", STEP, |s| {
            s.contains("canceled")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(!screen.contains("feat: too late"));
    assert_eq!(repo.git(&["log", "-1", "--format=%s"]), "initial commit");
}

/// Wait for whatever the last input did to finish repainting. Keys and
/// clicks resolve against render-time state, so a test that fires the next
/// one before the previous has landed races the event loop.
fn settled(term: &Terminal) -> xtask::pty::Screen {
    term.wait_for_stable(Duration::from_millis(150), STEP)
        .unwrap_or_else(|e| panic!("{e}"))
}

/// Settings > AI end to end: enable AI, add a profile (both written to
/// `config.toml`, no key in it), close Settings - the composer's Generate
/// button appears, and using it on a profile without a key says so instead
/// of calling anything.
#[test]
fn the_settings_page_turns_ai_on_and_adds_a_profile() {
    let repo = fixture();
    let (mut term, config) = launch(&repo, "pty-ai-settings", None);
    // The toolbar gear, as `pty_ui.rs` opens Preferences.
    let (gear_col, gear_row) = term
        .wait_for_text("\u{2699}", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(gear_col, gear_row).expect("click the gear");
    term.wait_for("Settings open", STEP, |s| {
        s.contains("Repo-specific settings")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    common::settings_page(&mut term, "AI");
    term.wait_for("the AI page", STEP, |s| s.contains("Enable AI"))
        .unwrap_or_else(|e| panic!("{e}"));

    let (col, row) = term.wait_for_text("Enable AI", STEP).unwrap();
    term.click(col + 1, row).expect("click the switch");
    let deadline = std::time::Instant::now() + STEP;
    let path = config.path().join("config.toml");
    while !std::fs::read_to_string(&path)
        .unwrap_or_default()
        .contains("enabled = true")
    {
        assert!(
            std::time::Instant::now() < deadline,
            "ai.enabled never saved. Screen was:\n{}",
            snap(&term).text()
        );
        std::thread::sleep(Duration::from_millis(200));
    }

    // The profile list's own "+ Add" button, on the list tab.
    common::settings_tab(&mut term, "Profiles (connections)");
    term.wait_for_text("+ Add", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    let (col, row) = snap(&term)
        .find_all("+ Add")
        .into_iter()
        .find(|(c, _)| *c < 40)
        .expect("the list's Add button (left pane)");
    term.click(col + 1, row).expect("press the button");
    let deadline = std::time::Instant::now() + STEP;
    while !std::fs::read_to_string(&path)
        .unwrap_or_default()
        .contains("[[ai.profiles]]")
    {
        assert!(
            std::time::Instant::now() < deadline,
            "the profile never saved. Screen was:\n{}",
            snap(&term).text()
        );
        std::thread::sleep(Duration::from_millis(200));
    }
    let raw = std::fs::read_to_string(&path).unwrap();
    assert!(raw.contains("token_ref = \"ai:profile\""), "{raw}");

    term.send_keys("Escape").expect("close Settings");
    term.send_keys("Escape")
        .expect("close Settings if a field had focus");
    let (col, row) = wait_generate(&term);
    term.click(col + 1, row).expect("click Generate");
    // The status line's error (no card); its entry has the cause.
    common::open_status_error(&mut term, "AI request failed");
    term.wait_for("the missing key reported, nothing called", STEP, |s| {
        s.contains("no API key")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(repo.git(&["log", "-1", "--format=%s"]), "initial commit");
}

/// Explain changes: the palette explains the working changes, the commit
/// menu's row explains a commit; the answer is a read-only popup that Esc
/// closes. With AI off the menu has no such row.
#[test]
fn explain_changes_opens_a_popup_from_the_palette_and_the_commit_menu() {
    let repo = fixture();
    let tools = tempfile::tempdir().unwrap();
    let agent = write_fake_agent(tools.path(), &["This change adds a second line to a.txt."]);
    let (mut term, _config) = launch(&repo, "pty-ai-explain", ai_config(&agent));

    term.send_keys("C-p").expect("open the palette");
    term.send_text("explain working").expect("type a query");
    term.wait_for("the entry offered", STEP, |s| {
        s.contains("AI: Explain Working Changes")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").expect("run it");
    term.wait_for("the explanation popup", STEP, |s| {
        s.contains("Explain working changes") && s.contains("adds a second line to a.txt")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(agent_calls(&agent), 1);
    term.send_keys("Escape").expect("close the popup");
    term.wait_for("the popup closed", STEP, |s| {
        !s.contains("Explain working changes")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // The commit's own menu.
    let (col, row) = term.wait_for_text("initial commit", STEP).unwrap();
    term.right_click(col + 1, row)
        .expect("right-click the commit");
    let menu = term
        .wait_for("the commit menu with its AI row", STEP, |s| {
            s.contains("Explain changes (AI)")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = menu.find("Explain changes (AI)").unwrap();
    term.click(c + 1, r).expect("click the AI row");
    term.wait_for("the commit's explanation", STEP, |s| {
        s.contains("Explain commit") && s.contains("adds a second line to a.txt")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(agent_calls(&agent), 2);
}

#[test]
fn with_ai_off_the_commit_menu_has_no_ai_row() {
    let repo = fixture();
    let (mut term, _config) = launch(&repo, "pty-ai-explain-off", None);
    let (col, row) = term.wait_for_text("initial commit", STEP).unwrap();
    term.right_click(col + 1, row)
        .expect("right-click the commit");
    let menu = term
        .wait_for("the commit menu", STEP, |s| {
            s.contains("Edit commit message...")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(!menu.contains("(AI)"), "no AI row while AI is off");
}

/// Stash with a message: `Ctrl+G m` opens the prompt (no AI needed); with
/// AI on, `Ctrl+G` inside it fills the message from the agent.
#[test]
fn the_stash_prompt_takes_a_typed_message_and_an_ai_one() {
    // Typed, AI off.
    let repo = fixture();
    let (mut term, _config) = launch(&repo, "pty-stash-typed", None);
    term.send_keys("C-g").unwrap();
    term.send_keys("m").unwrap();
    term.wait_for("the stash prompt, with no AI hint", STEP, |s| {
        s.contains("Stash message (optional)") && !s.contains("AI message")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_text("my typed stash").unwrap();
    term.send_keys("Enter").unwrap();
    wait_for_stash(&repo, "my typed stash");

    // AI on: Ctrl+G inside the prompt.
    let repo = fixture();
    let tools = tempfile::tempdir().unwrap();
    let agent = write_fake_agent(tools.path(), &["WIP: second line"]);
    let (mut term, _config) = launch(&repo, "pty-stash-ai", ai_config(&agent));
    term.send_keys("C-g").unwrap();
    term.send_keys("m").unwrap();
    term.wait_for("the stash prompt with the AI hint", STEP, |s| {
        s.contains("Stash message (optional)") && s.contains("AI message")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-g").unwrap();
    term.wait_for("the prompt filled by the agent", STEP, |s| {
        s.contains("WIP: second line")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(agent_calls(&agent), 1);
    term.send_keys("Enter").unwrap();
    wait_for_stash(&repo, "WIP: second line");
}

fn wait_for_stash(repo: &FixtureRepo, message: &str) {
    let deadline = std::time::Instant::now() + STEP;
    loop {
        if repo.git(&["stash", "list"]).contains(message) {
            return;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "no stash named {message:?}: {}",
            repo.git(&["stash", "list"])
        );
        std::thread::sleep(Duration::from_millis(200));
    }
}

/// One commit, then three uncommitted files: `a.txt` modified, `b.txt` and
/// `c.txt` new.
fn compose_fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial commit")
        .unwrap()
        .write_worktree("a.txt", b"one\ntwo\n")
        .unwrap()
        .write_worktree("b.txt", b"bee\n")
        .unwrap()
        .write_worktree("c.txt", b"sea\n")
        .unwrap()
        .build()
}

const COMPOSE_PLAN: &str =
    r#"[{"message":"feat: a","files":["a.txt"]},{"message":"fix: bc","files":["b.txt","c.txt"]}]"#;

fn open_composer(term: &mut Terminal) {
    term.send_keys("C-p").unwrap();
    term.send_text("compose commits").unwrap();
    term.wait_for("the entry offered", STEP, |s| {
        s.contains("AI: Compose Commits from Working Changes")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("the review popup with the plan", STEP, |s| {
        s.contains("feat: a") && s.contains("fix: bc") && s.contains("Create 2 commits")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

fn wait_for_head_subject(repo: &FixtureRepo, subject: &str) {
    let deadline = std::time::Instant::now() + STEP;
    while repo.git(&["log", "-1", "--format=%s"]) != subject {
        assert!(
            std::time::Instant::now() < deadline,
            "HEAD never became {subject:?}: {}",
            repo.git(&["log", "--format=%s"])
        );
        std::thread::sleep(Duration::from_millis(200));
    }
}

/// The commit composer, keyboard only: the model's plan opens as a review,
/// the user reorders (`J`), edits a message (`e`), and creates the commits
/// (`Ctrl+S`), each with exactly its files, the tree ending clean.
#[test]
fn the_commit_composer_review_reorders_edits_and_creates_the_commits() {
    let repo = compose_fixture();
    let tools = tempfile::tempdir().unwrap();
    let agent = write_fake_agent(tools.path(), &[COMPOSE_PLAN]);
    let (mut term, _config) = launch_wip(&repo, "pty-ai-compose-keys", ai_config(&agent));
    open_composer(&mut term);
    assert_eq!(agent_calls(&agent), 1);
    assert_eq!(
        repo.git(&["log", "-1", "--format=%s"]),
        "initial commit",
        "proposing a plan commits nothing"
    );

    term.send_keys("J").expect("move the first commit down");
    term.send_keys("e").expect("edit its message");
    term.wait_for("the reword popup", STEP, |s| {
        s.contains("Reword commit message")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_text("!").unwrap();
    term.send_keys("C-s").unwrap(); // applies the popup, not the plan
    term.wait_for("the edited message", STEP, |s| {
        s.contains("feat: a!") && !s.contains("Reword commit message")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-s").expect("create the commits");

    wait_for_head_subject(&repo, "feat: a!");
    assert_eq!(
        repo.git(&["log", "--format=%s", "-3"]),
        "feat: a!\nfix: bc\ninitial commit"
    );
    assert_eq!(
        repo.git(&["show", "--name-only", "--format=", "HEAD"]),
        "a.txt"
    );
    let earlier = repo.git(&["show", "--name-only", "--format=", "HEAD~1"]);
    assert!(
        earlier.contains("b.txt") && earlier.contains("c.txt"),
        "{earlier}"
    );
    assert_eq!(repo.git(&["status", "--porcelain"]), "");
}

/// The same with the mouse: click a commit, drop it, click Create - only
/// the kept commit is made and the dropped one's files stay uncommitted.
#[test]
fn the_commit_composer_review_drops_a_commit_by_mouse() {
    let repo = compose_fixture();
    let tools = tempfile::tempdir().unwrap();
    let agent = write_fake_agent(tools.path(), &[COMPOSE_PLAN]);
    let (mut term, _config) = launch_wip(&repo, "pty-ai-compose-mouse", ai_config(&agent));
    open_composer(&mut term);

    let (col, row) = term.wait_for_text("fix: bc", STEP).unwrap();
    term.click(col + 1, row).expect("select the second commit");
    // Wait until the *second* commit's row is the one offering the
    // controls (a frame later than the click), not the first's.
    let screen = term
        .wait_for("the second commit selected", STEP, |s| {
            matches!(
                (s.find("fix: bc"), s.find("\u{2715} drop")),
                (Some((_, a)), Some((_, b))) if a == b
            )
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (col, row) = screen.find("\u{2715} drop").unwrap();
    term.click(col + 1, row).expect("click drop");
    term.wait_for("one commit left to create", STEP, |s| {
        s.contains("Create 1 commit")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let (col, row) = term.wait_for_text("Create 1 commit", STEP).unwrap();
    term.click(col + 1, row).expect("click Create");

    wait_for_head_subject(&repo, "feat: a");
    assert_eq!(
        repo.git(&["log", "--format=%s", "-2"]),
        "feat: a\ninitial commit"
    );
    let status = repo.git(&["status", "--porcelain"]);
    assert!(
        status.contains("b.txt") && status.contains("c.txt"),
        "{status}"
    );
    assert!(!status.contains("a.txt"), "{status}");
}

/// Esc leaves the review with nothing done; an AI-off session has no entry
/// for it anywhere.
#[test]
fn the_commit_composer_review_cancels_and_is_hidden_with_ai_off() {
    let repo = compose_fixture();
    let tools = tempfile::tempdir().unwrap();
    let agent = write_fake_agent(tools.path(), &[COMPOSE_PLAN]);
    let (mut term, _config) = launch_wip(&repo, "pty-ai-compose-cancel", ai_config(&agent));
    open_composer(&mut term);
    term.send_keys("Escape").unwrap();
    term.wait_for("the review closed", STEP, |s| {
        !s.contains("Create 2 commits")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(repo.git(&["log", "-1", "--format=%s"]), "initial commit");

    let repo = compose_fixture();
    let (mut term, _config) = launch_wip(&repo, "pty-ai-compose-off", None);
    term.send_keys("C-p").unwrap();
    term.send_text("compose commits").unwrap();
    let screen = term
        .wait_for("the query echoed", STEP, |s| s.contains("compose commits"))
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(!screen.contains("AI: Compose Commits"));
}

/// Like [`launch`] for a fixture whose changes are all uncommitted files.
fn launch_wip(
    repo: &FixtureRepo,
    session: &str,
    config_toml: Option<String>,
) -> (Terminal, tempfile::TempDir) {
    common::launch(repo.root())
        .session(session)
        .config(config_toml.as_deref().unwrap_or_default())
        .size(200, 50)
        .ready_when("the fixture loaded, WIP row and all", |s| {
            s.contains("initial commit") && s.contains("// WIP")
        })
        .start()
}

/// A repository on `feature` (one commit ahead, pushed to a local bare
/// origin that is then re-pointed at a GitHub URL, so the form's
/// repository dropdowns resolve without any network access).
fn pull_request_fixture() -> FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial commit")
        .unwrap()
        .with_origin()
        .unwrap()
        .push("main")
        .unwrap()
        .track("main", "origin")
        .unwrap()
        .branch("feature")
        .unwrap()
        .checkout("feature")
        .unwrap()
        .commit_file("b.txt", "two\n", "add b file")
        .unwrap()
        .push("feature")
        .unwrap()
        .track("feature", "origin")
        .unwrap()
        .build();
    repo.git(&[
        "remote",
        "set-url",
        "origin",
        "https://github.com/acme/demo.git",
    ]);
    repo
}

/// The Create Pull Request form's `Generate title and description`: hidden
/// while AI is off, and with AI on it fills Title and Description from the
/// agent's answer (the diff and commits of the branch pair are read
/// locally; nothing is sent to GitHub).
#[test]
fn the_pull_request_form_generates_a_title_and_description() {
    let integration = "\n[[integrations]]\nid = \"gh\"\nkind = \"github\"\nurl = \"https://github.com\"\ntoken_ref = \"integration:gh\"\n";
    let open_form = |term: &mut Terminal| {
        term.send_keys("C-p").unwrap();
        term.send_text("create pull request").unwrap();
        term.wait_for("the entry offered", STEP, |s| {
            s.contains("Create Pull Request")
        })
        .unwrap_or_else(|e| panic!("{e}"));
        term.send_keys("Enter").unwrap();
        term.wait_for("the form", STEP, |s| {
            s.contains("From repo") && s.contains("Description") && s.contains("To repo")
        })
        .unwrap_or_else(|e| panic!("{e}"))
    };

    // AI off: no button.
    let repo = pull_request_fixture();
    let (mut term, _config) = launch_pr(&repo, "pty-ai-pr-off", integration.to_string());
    let screen = open_form(&mut term);
    assert!(!screen.contains("Generate title"), "hidden while AI is off");
    term.send_keys("Escape").unwrap();

    // AI on.
    let repo = pull_request_fixture();
    let tools = tempfile::tempdir().unwrap();
    let agent = write_fake_agent(
        tools.path(),
        &["Add the b file", "", "Adds b.txt with a greeting."],
    );
    let (mut term, _config) = launch_pr(
        &repo,
        "pty-ai-pr-on",
        format!("{}{integration}", ai_config_toml(&agent)),
    );
    open_form(&mut term);
    // The form opens before the host's answer: with no token the repository
    // listing fails and a modal "Hosting request failed" card appears over
    // it, whenever the daemon gets to it. A click on Generate that lands
    // while the card is up is swallowed by the card (the agent is never
    // called and the Description stays empty), and clicking outside it does
    // not dismiss it. So each attempt closes the card (its ✕) when one is up
    // and clicks Generate when none is, until the agent has answered: a click
    // on the button while the agent runs does nothing (the call count below
    // proves the agent ran once).
    //
    // (The credential helper's stderr used to be painted onto the terminal
    // here: its newline scrolled the screen under the TUI and every click
    // after it landed one row off. It is piped now, `gitpoulpi-secrets`'s
    // `helper_stderr` test pins that; the resize below only guarantees a
    // clean frame for the Title read.)
    term.act_until(
        "the Description filled by the agent",
        Duration::from_secs(6),
        STEP,
        |t| {
            let (col, row) = t.wait_for_text("Generate title and description", STEP)?;
            let screen = t.screen();
            let card_close = screen
                .find("Hosting request failed")
                .and_then(|(_, card_row)| {
                    screen
                        .find_all("\u{2715}")
                        .into_iter()
                        .find(|&(_, r)| r == card_row)
                });
            match card_close {
                Some((x, y)) => t.click(x, y),
                None => t.click(col + 1, row),
            }
        },
        |s| s.contains("Adds b.txt with a greeting."),
    )
    .unwrap_or_else(|e| panic!("{e}"));
    term.resize(201, 50).expect("force a repaint");
    term.wait_for("the Title filled by the agent", STEP, |s| {
        s.contains("Add the b file") && s.contains("Adds b.txt with a greeting.")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(agent_calls(&agent), 1);
}

/// Like [`launch`], with a `config.toml` and an environment in which
/// `git credential fill` cannot prompt or reach a credential manager.
fn launch_pr(
    repo: &FixtureRepo,
    session: &str,
    config_toml: String,
) -> (Terminal, tempfile::TempDir) {
    let config = tempfile::tempdir().expect("temp config dir");
    std::fs::write(config.path().join("config.toml"), config_toml).unwrap();
    let empty_git_config = config.path().join("empty.gitconfig");
    std::fs::write(&empty_git_config, "").unwrap();
    let term = common::launch(repo.root())
        .session(session)
        .config_dir(config.path())
        .size(200, 50)
        .env("GIT_CONFIG_GLOBAL", &empty_git_config)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GCM_INTERACTIVE", "never")
        .spawn();
    term.wait_for("the fixture loaded", STARTUP, |s| {
        s.contains("add b file") && s.contains("initial commit")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    (term, config)
}

/// Settings > AI, one action's block end to end: pick a profile for
/// Commit message generation through its combo, click a chip, type in the
/// action's textarea, and read it all back from `config.toml`.
#[test]
fn an_actions_profile_chip_and_textarea_reach_config_toml() {
    let repo = fixture();
    let config_toml = r#"
[ai]
enabled = true

[[ai.profiles]]
id = "alpha"
name = "Alpha"
kind = "local_agent"
command = "claude"
args = ["-p"]

[[ai.profiles]]
id = "beta"
name = "Beta"
kind = "local_agent"
command = "claude"
args = ["-p"]
"#;
    let (mut term, config) = launch(&repo, "pty-ai-actions", Some(config_toml.to_string()));
    let (gear_col, gear_row) = term
        .wait_for_text("\u{2699}", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(gear_col, gear_row).expect("click the gear");
    term.wait_for("Settings open", STEP, |s| {
        s.contains("Repo-specific settings")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    common::settings_page(&mut term, "AI");
    // The local-agent profile editor is long (type, command, options,
    // model, environment variables): walk down until the action blocks
    // scroll in.
    term.wait_for("the AI page", STEP, |s| s.contains("Enable AI"))
        .unwrap_or_else(|e| panic!("{e}"));
    // Sections -> the General tab's rows.
    term.send_keys("Enter").unwrap();
    settled(&term);
    for _ in 0..40 {
        let s = snap(&term);
        if s.contains("Commit message generation") && s.contains("generate commit messages") {
            break;
        }
        term.send_keys("Down").unwrap();
        settled(&term);
    }
    term.wait_for("the AI page with its action blocks", STEP, |s| {
        s.contains("Commit message generation") && s.contains("generate commit messages")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let path = config.path().join("config.toml");
    let read = || std::fs::read_to_string(&path).unwrap_or_default();
    let wait_for_file = |what: &str, ok: &dyn Fn(&str) -> bool| {
        let deadline = std::time::Instant::now() + STEP;
        while !ok(&read()) {
            assert!(std::time::Instant::now() < deadline, "{what}: {}", read());
            std::thread::sleep(Duration::from_millis(200));
        }
    };

    // The combo: its help line sits right under the combo's own row.
    let (col, row) = term
        .wait_for_text("The profile GitPoulpi will use to generate commit", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(col + 1, row - 1)
        .expect("open the profile combo");
    term.wait_for("the dropdown with the default entry", STEP, |s| {
        s.contains("Default (Alpha)") && s.contains("Beta (")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Down").unwrap(); // Default -> Alpha
    term.send_keys("Down").unwrap(); // Alpha -> Beta
    term.send_keys("Enter").unwrap();
    wait_for_file("ai.actions.commit_message.profile never saved", &|raw| {
        raw.contains("[ai.actions.commit_message]") && raw.contains("profile = \"beta\"")
    });

    // The next row is the action's Custom instructions: Enter opens the
    // editor, whose Quick insert row holds the chips.
    term.send_keys("Down").unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("the editor", STEP, |s| {
        s.contains("Commit message generation - Custom instructions")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let (col, row) = term
        .wait_for_text("+ Include issue numbers", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(col + 2, row).expect("click the chip");
    term.wait_for("the sentence inserted", STEP, |s| {
        s.contains("Include the related issue numbers")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_text("Answer in French. ").unwrap();
    term.send_keys("C-s").unwrap();
    wait_for_file("the typed text never saved", &|raw| {
        raw.contains("Answer in French.") && raw.contains("Include the related issue numbers")
    });
    let raw = read();
    assert!(raw.contains("profile = \"beta\""), "{raw}");
}

/// Settings > Repo-specific settings > AI: a profile pick and a chip
/// land in the repository's `repo-state.toml` and never in `config.toml`;
/// "Reset to global" clears them again.
#[test]
fn the_repo_ai_page_writes_overrides_to_repo_state_only() {
    let repo = fixture();
    let config_toml = r#"
[ai]
enabled = true

[[ai.profiles]]
id = "alpha"
name = "Alpha"
kind = "local_agent"
command = "claude"
args = ["-p"]

[[ai.profiles]]
id = "beta"
name = "Beta"
kind = "local_agent"
command = "claude"
args = ["-p"]
"#;
    let (mut term, config) = launch(&repo, "pty-ai-repo", Some(config_toml.to_string()));
    let global_path = config.path().join("config.toml");
    let global_before = std::fs::read_to_string(&global_path).unwrap();
    let state_path = repo.state_dir().join("repo-state.toml");
    let read = || std::fs::read_to_string(&state_path).unwrap_or_default();
    let wait_for_state = |what: &str, ok: &dyn Fn(&str) -> bool| {
        let deadline = std::time::Instant::now() + STEP;
        while !ok(&read()) {
            assert!(std::time::Instant::now() < deadline, "{what}: {}", read());
            std::thread::sleep(Duration::from_millis(200));
        }
    };
    let (gear_col, gear_row) = term
        .wait_for_text("\u{2699}", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(gear_col, gear_row).expect("click the gear");
    term.wait_for("Settings open", STEP, |s| {
        s.contains("Repo-specific settings")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    common::settings_page_nth(&mut term, "AI", 1); // the repo's AI page
    term.wait_for("the repo AI page", STEP, |s| {
        s.contains("Empty fields use the global AI settings.")
            && s.contains("The profile used in this repository to generate commit")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // The combo.
    let (col, row) = term
        .wait_for_text(
            "The profile used in this repository to generate commit",
            STEP,
        )
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(col + 1, row - 1).expect("open the combo");
    term.wait_for("Use global first, then the profiles", STEP, |s| {
        s.contains("Use global (Alpha)") && s.contains("Beta (")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Down").unwrap();
    term.send_keys("Down").unwrap();
    term.send_keys("Enter").unwrap();
    wait_for_state("the profile override never saved", &|raw| {
        raw.contains("[preferences.ai_overrides.commit_message]")
            && raw.contains("profile = \"beta\"")
    });

    // A chip, in the Custom instructions editor of the next row.
    term.send_keys("Down").unwrap();
    term.send_keys("Enter").unwrap();
    let (col, row) = term
        .wait_for_text("+ Include issue numbers", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(col + 2, row).expect("click the chip");
    term.send_keys("C-s").unwrap();
    wait_for_state("the chip never saved", &|raw| {
        raw.contains("Include the related issue numbers")
    });
    assert_eq!(
        std::fs::read_to_string(&global_path).unwrap(),
        global_before,
        "config.toml must not change"
    );

    // Reset to global.
    let (col, row) = term
        .wait_for_text("Reset to global", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(col + 1, row).expect("select the button");
    settled(&term);
    term.click(col + 1, row).expect("press the button");
    wait_for_state("the override was never cleared", &|raw| {
        !raw.contains("commit_message")
    });
    assert_eq!(
        std::fs::read_to_string(&global_path).unwrap(),
        global_before
    );
}

/// One commit and an unstaged edit: nothing staged.
fn unstaged_fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial commit")
        .unwrap()
        .write_worktree("a.txt", b"one\ntwo\n")
        .unwrap()
        .build()
}

/// Generate refuses (and never calls the agent) with nothing staged, and
/// works once something is staged.
#[test]
fn generate_refuses_with_nothing_staged_and_works_after_staging() {
    let repo = unstaged_fixture();
    let tools = tempfile::tempdir().unwrap();
    let agent = write_fake_agent(tools.path(), &["feat: staged now"]);
    let (mut term, _config) = launch(&repo, "pty-ai-gen-staged", ai_config(&agent));

    let (col, row) = wait_generate(&term);
    term.click(col + 1, row).expect("click Generate");
    term.wait_for("the refusal naming staging", STEP, |s| {
        s.contains("stage some changes first")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(agent_calls(&agent), 0, "a refused click calls nothing");

    // Stage everything (Ctrl+G a), then Generate works.
    term.send_keys("C-g").expect("Ctrl+G");
    term.send_keys("a").expect("a");
    term.wait_for("the file staged", STEP, |s| s.contains("Staged Files (1)"))
        .unwrap_or_else(|e| panic!("{e}"));
    let (col, row) = wait_generate(&term);
    term.click(col + 1, row).expect("click Generate again");
    term.wait_for("the composer filled", STEP, |s| {
        s.contains("feat: staged now")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(agent_calls(&agent), 1);
}

/// The header's red trash sits left of the title and opens the discard
/// confirmation; the AI Explain sparkle sits at the far right and runs
/// the explain action.
#[test]
fn the_header_trash_confirms_a_discard_and_the_explain_button_explains() {
    let repo = unstaged_fixture();
    let tools = tempfile::tempdir().unwrap();
    let agent = write_fake_agent(tools.path(), &["The header explanation."]);
    let (mut term, _config) = launch(&repo, "pty-ai-header", ai_config(&agent));

    let screen = term
        .wait_for("the header with both buttons", STEP, |s| {
            s.contains(" \u{2716}") && s.contains(" \u{2726} ") && s.contains("file change")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (tc, tr) = screen.find(" \u{2716}").unwrap();
    let (title_col, title_row) = screen.find("file change").unwrap();
    assert_eq!(tr, title_row, "same header row");
    assert!(tc < title_col, "the trash is left of the title");
    let (ec, er) = screen.find(" \u{2726} ").unwrap();
    assert_eq!(er, title_row);
    assert!(ec > title_col, "Explain is at the far right");

    term.click(tc + 1, tr).expect("click the trash");
    term.wait_for("the discard confirmation", STEP, |s| {
        s.contains("Discard 1 file? This cannot be undone")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Escape").expect("cancel");
    term.wait_for("the confirmation closed", STEP, |s| {
        !s.contains("Discard 1 file? This cannot be undone")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    term.click(ec + 1, er).expect("click Explain");
    term.wait_for("the explanation popup", STEP, |s| {
        s.contains("Explain working changes") && s.contains("The header explanation.")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(agent_calls(&agent), 1);
}

/// Keyboard: Enter opens the reword popup on a proposal (message
/// *and* description), applying it puts the text back in the plan, and the
/// commit made carries both. The right pane lists the selected proposal's
/// files.
#[test]
fn the_composer_reword_popup_edits_message_and_description_and_files_show_on_the_right() {
    let repo = compose_fixture();
    let tools = tempfile::tempdir().unwrap();
    let agent = write_fake_agent(tools.path(), &[COMPOSE_PLAN]);
    let (mut term, _config) = launch_wip(&repo, "pty-ai-compose-reword", ai_config(&agent));
    open_composer(&mut term);

    // The right pane follows the selection: commit 1 owns a.txt only.
    term.wait_for("commit 1's files in the right pane", STEP, |s| {
        s.contains("Commit 1 \u{b7} 1 file")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Down").unwrap();
    term.send_keys("Down").unwrap(); // second commit's row (its first file is between)
    term.wait_for("commit 2's files in the right pane", STEP, |s| {
        s.contains("Commit 2 \u{b7} 2 files")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Up").unwrap();
    term.send_keys("Up").unwrap();

    term.send_keys("Enter").unwrap();
    term.wait_for("the reword popup", STEP, |s| {
        s.contains("Reword commit message")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_text("!").unwrap();
    term.send_keys("Enter").unwrap(); // summary -> description
    term.send_text("why it matters").unwrap();
    term.wait_for("the description typed", STEP, |s| {
        s.contains("why it matters")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-s").unwrap();
    term.wait_for("popup closed, row marked as having a body", STEP, |s| {
        !s.contains("Reword commit message") && s.contains("feat: a! \u{2261}")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-s").unwrap();

    wait_for_head_subject(&repo, "fix: bc");
    let body = repo.git(&["log", "--format=%B", "--grep", "feat: a!", "-1"]);
    assert!(
        body.contains("feat: a!") && body.contains("why it matters"),
        "{body}"
    );
    assert_eq!(repo.git(&["status", "--porcelain"]), "");
}

/// Mouse: a double-click on a proposal row opens the same popup
/// (a single click only selects), and Esc closes it leaving the plan as it
/// was.
#[test]
fn double_clicking_a_composer_row_opens_the_reword_popup() {
    let repo = compose_fixture();
    let tools = tempfile::tempdir().unwrap();
    let agent = write_fake_agent(tools.path(), &[COMPOSE_PLAN]);
    let (mut term, _config) = launch_wip(&repo, "pty-ai-compose-dblclick", ai_config(&agent));
    open_composer(&mut term);

    let (col, row) = term.wait_for_text("fix: bc", STEP).unwrap();
    term.click(col + 1, row).unwrap();
    term.wait_for("the second commit selected", STEP, |s| {
        s.contains("Commit 2 \u{b7} 2 files")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        !snap(&term).contains("Reword commit message"),
        "a single click must not open the popup"
    );
    term.double_click(col + 1, row).unwrap();
    term.wait_for("the reword popup", STEP, |s| {
        s.contains("Reword commit message")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Escape").unwrap();
    term.wait_for("popup closed, plan intact", STEP, |s| {
        !s.contains("Reword commit message") && s.contains("Create 2 commits")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

/// Prompt variables in the user's instructions are expanded before the
/// prompt is sent: the (fake, prompt-capturing) agent receives the
/// repository's folder name, the UI language and the file count, not the
/// `${...}` text; an unknown variable and an escaped `$${` stay as written.
#[cfg(unix)]
#[test]
fn prompt_variables_are_expanded_in_what_the_agent_receives() {
    use gitpoulpi_testkit::fake_agent::{agent_prompt, write_capturing_fake_agent};
    let repo = fixture();
    let name = repo
        .root()
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    let tools = tempfile::tempdir().unwrap();
    let agent = write_capturing_fake_agent(tools.path(), &["feat: add two"]);
    let config = ai_config_toml(&agent).replacen(
        "enabled = true\n",
        "enabled = true\nglobal_instructions = \"Work on ${repo} in ${uilang}, ${files} file(s). ${nope} $${repo}\"\n",
        1,
    );
    let (mut term, _config) = launch(&repo, "pty-ai-vars", Some(config));
    let (col, row) = wait_generate(&term);
    term.click(col + 1, row).expect("click Generate");
    term.wait_for("the composer filled", STEP, |s| s.contains("feat: add two"))
        .unwrap_or_else(|e| panic!("{e}"));
    let prompt = agent_prompt(&agent);
    assert!(
        prompt.contains(&format!(
            "Work on {name} in English, 1 file(s). ${{nope}} ${{repo}}"
        )),
        "{prompt}"
    );
}

/// Repository instructions end to end: written in Settings > Repo-Specific
/// Preferences > AI, saved to the repository's `repo-state.toml`, and sent
/// to the (fake, prompt-capturing) agent by Generate - after the global
/// instructions in "Add to global" mode, instead of them in "Replace
/// global" mode.
#[cfg(unix)]
#[test]
fn the_repo_instructions_reach_the_prompt_in_both_modes() {
    use gitpoulpi_testkit::fake_agent::{agent_prompt, write_capturing_fake_agent};
    let repo = fixture();
    let tools = tempfile::tempdir().unwrap();
    let agent = write_capturing_fake_agent(tools.path(), &["feat: add two"]);
    let config = format!(
        "{}\n",
        ai_config_toml(&agent).replacen(
            "enabled = true\n",
            "enabled = true\nglobal_instructions = \"Global rule GG.\"\n",
            1
        )
    );
    let (mut term, _config) = launch(&repo, "pty-ai-repo-prompt", Some(config));

    let (gear_col, gear_row) = term
        .wait_for_text("\u{2699}", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(gear_col, gear_row).expect("click the gear");
    term.wait_for("Settings open", STEP, |s| {
        s.contains("Repo-specific settings")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    common::settings_page_nth(&mut term, "AI", 1); // the repository's AI page
    term.wait_for("the repository AI page", STEP, |s| {
        s.contains("Repository instructions") && s.contains("Combine with global")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // A click on the textarea opens the editor (Enter on the row does the
    // same); Ctrl+S saves.
    let (col, row) = term.wait_for_text("Custom instructions", STEP).unwrap();
    term.click(col + 1, row).expect("click the textarea");
    term.wait_for("the editor", STEP, |s| {
        s.contains("Repository instructions - Custom instructions")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_text("Repo rule RR.").unwrap();
    term.send_keys("C-s").unwrap();
    let state = repo.state_dir().join("repo-state.toml");
    wait_for_file_containing(&term, &state, "Repo rule RR.");

    let generate = |term: &mut Terminal| {
        term.send_keys("Escape").unwrap();
        term.send_keys("Escape").unwrap();
        // The keyboard twin of the sparkle button.
        term.send_keys("C-g").unwrap();
        term.send_keys("i").unwrap();
    };
    generate(&mut term);
    wait_for_agent_calls(&term, &agent, 1);
    let first = agent_prompt(&agent);
    let g = first
        .find("## Global instructions\nGlobal rule GG.")
        .unwrap();
    let r = first
        .find("## Repository instructions\nRepo rule RR.")
        .unwrap();
    assert!(g < r, "{first}");

    // Switch the combo to "Replace global".
    term.click(gear_col, gear_row).ok();
    term.wait_for("Settings open again", STEP, |s| {
        s.contains("Combine with global")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    // The click is retried until the dropdown is up (a click aimed at a torn
    // frame opens nothing); the keys go to a settled dropdown.
    common::retry_until(
        &mut term,
        "the dropdown with both modes",
        4,
        Duration::from_secs(10),
        |t| {
            let screen = t
                .wait_for_stable(Duration::from_millis(300), STEP)
                .unwrap_or_else(|e| panic!("{e}"));
            if !screen.contains("\u{2713} Add to global") {
                let (col, row) = screen.find("Combine with global").expect("the combo row");
                t.click(col + 1, row).expect("open the combo");
            }
        },
        |s| s.contains("\u{2713} Add to global") && s.contains("Replace global"),
    );
    term.wait_for_stable(Duration::from_millis(300), STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Down").unwrap(); // Add -> Replace
    term.wait_for_stable(Duration::from_millis(300), STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    wait_for_file_containing(&term, &state, "replace");

    generate(&mut term);
    wait_for_agent_calls(&term, &agent, 2);
    let second = agent_prompt(&agent);
    assert!(!second.contains("Global rule GG."), "{second}");
    assert!(
        second.contains("## Repository instructions\nRepo rule RR."),
        "{second}"
    );
}

#[cfg(unix)]
fn wait_for_file_containing(term: &Terminal, path: &Path, needle: &str) {
    let deadline = std::time::Instant::now() + STEP;
    while !std::fs::read_to_string(path)
        .unwrap_or_default()
        .contains(needle)
    {
        assert!(
            std::time::Instant::now() < deadline,
            "{} never contained {needle:?}. Screen was:\n{}",
            path.display(),
            snap(term).text()
        );
        std::thread::sleep(Duration::from_millis(200));
    }
}

#[cfg(unix)]
fn wait_for_agent_calls(term: &Terminal, agent: &Path, n: usize) {
    let deadline = std::time::Instant::now() + STEP;
    while agent_calls(agent) < n {
        assert!(
            std::time::Instant::now() < deadline,
            "the agent was never called {n} time(s). Screen was:\n{}",
            snap(term).text()
        );
        std::thread::sleep(Duration::from_millis(200));
    }
}

/// The screen once it has settled: a torn frame is not a place to locate a
/// cell or compare colors in.
fn snap(term: &xtask::pty::Terminal) -> xtask::pty::Screen {
    term.wait_for_stable(
        std::time::Duration::from_millis(300),
        std::time::Duration::from_secs(30),
    )
    .unwrap_or_else(|e| panic!("{e}"))
}
