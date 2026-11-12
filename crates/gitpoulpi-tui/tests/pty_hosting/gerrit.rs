//! Slice T3 end to end: change `#7` of the shared `xtask mock-host`'s Gerrit
//! flavour, reviewed in the app. The repository is real - its `origin` holds
//! the change's two patch sets as `refs/changes/07/7/1` and `/2` - so the
//! view really fetches the refs and asks the daemon for the diff, and the
//! host only answers the REST calls. Covered: the patch-set combobox (the
//! latest first, another read on pick), the Finish dialog's vote comboboxes
//! (only the values the server permits) and the review reaching the server
//! on the patch set that is shown.

use crate::support::*;
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::mock_host::{self, Options};
use xtask::pty::Terminal;

use super::review::wait_until;

fn config(port: u16) -> String {
    format!(
        "[general]\nhosting_refresh_minutes = 0\n\n[ui]\npr_terminology = \"pull_request\"\n\n[[integrations]]\nid = \"gr\"\nkind = \"gerrit\"\nurl = \"http://127.0.0.1:{port}\"\ntoken_ref = \"integration:gr\"\naccount = \"mara\"\nname = \"gr\"\nuser = {{ login = \"mara\" }}\n"
    )
}

/// `main` with a 30-line `src/planets.rs`; patch set 1 replaces line 12 by
/// two lines, patch set 2 also adds `docs/moons.md`.
fn gerrit_fixture() -> FixtureRepo {
    let base: String = (1..=30).map(|n| format!("// line {n}\n")).collect();
    let moons = base.replace("// line 12\n", "let moons = Vec::new();\nmoons.push(1);\n");
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("src/planets.rs", &base, "Start the planet table")
        .unwrap()
        .with_origin()
        .unwrap()
        .push("main")
        .unwrap()
        .branch("feature")
        .unwrap()
        .commit_file("src/planets.rs", &moons, "Add moons to the table")
        .unwrap()
        .push("feature:refs/changes/07/7/1")
        .unwrap()
        .commit_file("docs/moons.md", "# Moons\ntext\n", "Document the moons")
        .unwrap()
        .push("feature:refs/changes/07/7/2")
        .unwrap()
        .build();
    let dir = repo.state_dir();
    std::fs::create_dir_all(&dir).expect("state dir");
    std::fs::write(
        dir.join("repo-state.toml"),
        "[pulls_source]\nintegration = \"gr\"\nproject = \"sandbox\"\n",
    )
    .expect("seed repo-state.toml");
    repo
}

fn launch_gerrit(repo: &FixtureRepo, session: &str, port: u16) -> (Terminal, tempfile::TempDir) {
    let dir = tempfile::tempdir().expect("temp config dir");
    std::fs::write(dir.path().join("config.toml"), config(port)).expect("seed config.toml");
    let term = crate::common::launch(repo.root())
        .session(session)
        .config_dir(dir.path())
        .size(COLS, ROWS)
        .env("GITPOULPI_NO_BROWSER", "1")
        .env("GITPOULPI_TOKEN_GR", "http-password")
        .spawn();
    term.wait_for("the change listed", STARTUP, |s| {
        s.contains("PULL REQUESTS (1)")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    (term, dir)
}

/// Opens the review of `#7` with Enter on its row.
fn open_review(term: &mut Terminal) {
    let screen = term
        .wait_for("the change listed", STARTUP, |s| {
            s.contains("All Pull Requests")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    if !screen.contains("#7  Add moons") {
        let (c, r) = screen.find("All Pull Requests").unwrap();
        term.click(c + 2, r).expect("unfold the All group");
    }
    let screen = term
        .wait_for("the #7 row", STEP, |s| s.contains("#7  Add moons"))
        .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = screen.find("#7  Add moons").unwrap();
    term.click(c + 1, r).expect("select the row");
    term.send_keys("Enter").expect("review");
    term.wait_for("the review view", STEP, |s| {
        s.contains("Review #7") && s.contains("Add moons to the table")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

#[test]
fn a_change_is_reviewed_at_a_patch_set_and_the_vote_reaches_gerrit() {
    let mock = mock_host::start(Options::default()).expect("the mock host");
    let repo = gerrit_fixture();
    let (mut term, _config) = launch_gerrit(&repo, "pty-gerrit-flow", mock.port);
    open_review(&mut term);

    // The patch-set combobox starts on the newest patch set.
    let screen = term
        .wait_for("the patch-set combobox", STEP, |s| {
            s.contains("Patch set 2 (latest)") && s.contains("main \u{2190} refs/changes/07/7/2")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));
    let _ = screen;

    // Files at patch set 2 list both files; the code of planets.rs carries
    // Gerrit's thread under its line.
    term.send_keys("M-2").expect("Alt+2");
    term.wait_for("the files of patch set 2", STEP, |s| {
        s.contains("docs/moons.md") && s.contains("src/planets.rs")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));
    term.send_keys("Down").expect("select planets.rs");
    let screen = term
        .wait_for("the diff and its thread", STEP, |s| {
            s.contains("let moons = Vec::new();") && s.contains("Why a Vec here and not a slice?")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));
    let (_, code) = screen.find("let moons = Vec::new();").unwrap();
    let (_, thread) = screen.find("Why a Vec here and not a slice?").unwrap();
    assert!(
        thread > code && thread <= code + 3,
        "the thread sits under its line:\n{}",
        screen.text()
    );

    // `P` opens the list (the keyboard twin of a click on the field); pick patch set 1.
    term.send_keys("P").expect("patch sets");
    let screen = term
        .wait_for("the list of patch sets", STEP, |s| {
            s.contains("Patch set 1") && s.contains("Patch set 2 (latest)")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));
    assert!(screen.contains("Patch set 1"), "{}", screen.text());
    term.send_keys("Down Enter").expect("choose patch set 1");
    let screen = term
        .wait_for("patch set 1 read", STEP, |s| {
            s.contains("Patch set 1")
                && !s.contains("docs/moons.md")
                && s.contains("let moons = Vec::new();")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));
    assert!(
        screen.contains("main \u{2190} refs/changes/07/7/1")
            || !screen.contains("(latest)\u{25be}"),
        "{}",
        screen.text()
    );
    assert!(
        mock.requests()
            .iter()
            .any(|r| r.method == "GET" && r.target.contains("/revisions/1/files/")),
        "the files of patch set 1 were asked for"
    );

    // Finish: one combobox per label, only the permitted values.
    term.send_keys("f").expect("finish");
    let screen = term
        .wait_for("the finish dialog with its votes", STEP, |s| {
            s.contains("Code-Review") && s.contains("Verified")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));
    assert!(!screen.contains("Request changes"), "{}", screen.text());
    // Summary, then Code-Review: open it and take +1.
    term.send_text("Looks fine to me").unwrap();
    term.send_keys("Tab Enter").expect("open Code-Review");
    let screen = term
        .wait_for("the Code-Review values", STEP, |s| {
            s.contains("This shall not be submitted") && s.contains("+2")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));
    assert!(screen.contains("-1"), "{}", screen.text());
    // The list opens on the current vote (0); Down moves to +1.
    term.send_keys("Down Enter").expect("pick +1");
    term.wait_for("+1 chosen", STEP, |s| {
        s.text()
            .lines()
            .any(|l| l.contains("Code-Review") && l.contains("+1"))
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));
    term.send_keys("C-s").expect("submit");
    wait_until("the server got the review", || {
        mock.requests().iter().any(|r| {
            r.method == "POST" && r.target.contains("/revisions/") && r.target.ends_with("/review")
        })
    });
    let post = mock
        .requests()
        .into_iter()
        .find(|r| r.method == "POST" && r.target.ends_with("/review"))
        .unwrap();
    assert!(
        post.target.contains(&format!(
            "/a/changes/sandbox~7/revisions/{}/review",
            mock_host::gerrit::sha_of(1)
        )),
        "sent on the patch set that was read: {}",
        post.target
    );
    assert!(post.body.contains("\"Code-Review\":1"), "{}", post.body);
    assert!(
        !post.body.contains("Verified"),
        "an unchanged label is not sent: {}",
        post.body
    );
    assert!(post.body.contains("Looks fine to me"), "{}", post.body);
    assert!(
        post.headers.to_lowercase().contains("authorization: basic"),
        "HTTP basic: {}",
        post.headers
    );
    term.wait_for("the review reported", STEP, |s| {
        s.contains("review submitted")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));
    term.stop().ok();
}

#[test]
fn the_palette_twin_opens_the_patch_set_list() {
    let mock = mock_host::start(Options::default()).expect("the mock host");
    let repo = gerrit_fixture();
    let (mut term, _config) = launch_gerrit(&repo, "pty-gerrit-palette", mock.port);
    open_review(&mut term);
    term.wait_for("the patch-set combobox", STEP, |s| {
        s.contains("Patch set 2 (latest)")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-p").expect("the palette");
    term.wait_for("the palette", STEP, |s| s.contains("Command Palette"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_text("Review: Choose Patch Set").unwrap();
    term.wait_for("the command", STEP, |s| {
        s.contains("Review: Choose Patch Set...")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").expect("run it");
    term.wait_for("the list of patch sets", STEP, |s| {
        s.contains("Patch set 1") && s.contains("Patch set 2 (latest)")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));
    term.send_keys("Escape").expect("close the list");
    term.stop().ok();
}

/// The palette row (and the section's `+`, which is the same flow) pushes
/// `HEAD` to the review ref of the picked branch through the daemon.
#[test]
fn push_for_review_sends_head_to_the_review_ref_of_the_picked_branch() {
    let mock = mock_host::start(Options::default()).expect("the mock host");
    let repo = gerrit_fixture();
    let (mut term, _config) = launch_gerrit(&repo, "pty-gerrit-push", mock.port);
    // The row needs the listed changes to know their target branch.
    term.wait_for("the change listed", STARTUP, |s| {
        s.contains("All Pull Requests")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-p").expect("the palette");
    term.wait_for("the palette", STEP, |s| s.contains("Command Palette"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_text("Hosting: Push for Review").unwrap();
    term.wait_for("the command", STEP, |s| {
        s.contains("Hosting: Push for Review...")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").expect("run it");
    term.wait_for("the target branches", STEP, |s| {
        s.contains("Push HEAD for review on branch")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));
    // The change listed targets `main`: the only row.
    term.send_keys("Enter").expect("pick main");
    let head = repo.git(&["rev-parse", "HEAD"]);
    wait_until("the review ref on the server", || {
        std::process::Command::new("git")
            .args(["rev-parse", "--verify", "-q", "refs/for/main"])
            .current_dir(repo.remote_path().expect("an origin"))
            .output()
            .ok()
            .filter(|o| o.status.success())
            .is_some_and(|o| String::from_utf8_lossy(&o.stdout).trim() == head)
    });
    term.stop().ok();
}

/// "Compare with" an older patch set: the Files page lists only what patch
/// set 2 changed since patch set 1 (the server's `files/?base=1`), the diff is
/// read from the two fetched patch sets, and the Commits page shows that one
/// difference. `D` and the palette are the keyboard twins of the combobox.
#[test]
fn comparing_two_patch_sets_shows_only_what_changed_between_them() {
    let mock = mock_host::start(Options::default()).expect("the mock host");
    let repo = gerrit_fixture();
    let (mut term, _config) = launch_gerrit(&repo, "pty-gerrit-compare", mock.port);
    open_review(&mut term);
    term.wait_for("the compare combobox", STEP, |s| {
        s.contains("Patch set 2 (latest)") && s.contains("Compare: base")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));

    // `D` opens the list: the base, then the older patch sets only.
    term.send_keys("D").expect("compare");
    let screen = term
        .wait_for("the compare list", STEP, |s| {
            s.contains("Compare: patch set 1")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));
    assert!(
        !screen.contains("Compare: patch set 2"),
        "{}",
        screen.text()
    );
    term.send_keys("Down Enter").expect("pick patch set 1");
    term.wait_for("the difference read", STEP, |s| {
        s.contains("Compare: patch set 1")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));

    term.send_keys("M-2").expect("Alt+2");
    let screen = term
        .wait_for("only the file patch set 2 added", STEP, |s| {
            s.contains("docs/moons.md") && !s.contains("src/planets.rs")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));
    assert!(
        mock.requests()
            .iter()
            .any(|r| r.target.ends_with("/files/?base=1")),
        "the server was asked for the difference: {:?}",
        mock.requests()
            .iter()
            .map(|r| &r.target)
            .collect::<Vec<_>>()
    );
    let _ = screen;
    term.wait_for("the diff between the two patch sets", STEP, |s| {
        s.contains("# Moons")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));

    // The Commits page: the head patch set against patch set 1.
    term.send_keys("M-3").expect("Alt+3");
    let screen = term
        .wait_for("the head commit alone", STEP, |s| {
            s.contains("Commits (1)") && s.contains("Document the moons")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));
    // Patch set 1's commit is the old side, not a row of the list: its
    // subject is only the change's title in the header.
    assert_eq!(
        screen.find_all("Add moons to the table").len(),
        1,
        "{}",
        screen.text()
    );
    term.stop().ok();
}

#[test]
fn the_palette_twin_opens_the_compare_list() {
    let mock = mock_host::start(Options::default()).expect("the mock host");
    let repo = gerrit_fixture();
    let (mut term, _config) = launch_gerrit(&repo, "pty-gerrit-compare-palette", mock.port);
    open_review(&mut term);
    term.wait_for("the compare combobox", STEP, |s| {
        s.contains("Compare: base")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-p").expect("the palette");
    term.wait_for("the palette", STEP, |s| s.contains("Command Palette"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_text("Review: Compare Patch Sets").unwrap();
    term.wait_for("the command", STEP, |s| {
        s.contains("Review: Compare Patch Sets...")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").expect("run it");
    term.wait_for("the compare list", STEP, |s| {
        s.contains("Compare: patch set 1")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));
    term.send_keys("Escape").expect("close the list");
    term.stop().ok();
}

fn posts_to(mock: &mock_host::MockHost, tail: &str) -> usize {
    mock.requests()
        .iter()
        .filter(|r| r.method == "POST" && r.target.ends_with(tail))
        .count()
}

/// Submit and Abandon: header buttons (the mock says the change is
/// submittable and abandonable), each behind a confirmation that says it
/// cannot be undone; declining sends nothing.
#[test]
fn submit_and_abandon_ask_first_and_reach_the_server_only_when_confirmed() {
    let mock = mock_host::start(Options::default()).expect("the mock host");
    let repo = gerrit_fixture();
    let (mut term, _config) = launch_gerrit(&repo, "pty-gerrit-submit", mock.port);
    open_review(&mut term);
    term.wait_for("the Submit and Abandon buttons", STEP, |s| {
        s.contains("Submit") && s.contains("Abandon")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));

    // `U`: the question; Esc declines, nothing is sent.
    term.send_keys("U").expect("submit");
    let screen2 = term
        .wait_for("the confirmation", STEP, |s| {
            s.contains("Submit #7 on the server?") && s.contains("cannot undo it")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));
    let _ = screen2;
    term.send_keys("Escape").expect("decline");
    term.wait_for("the question gone", STEP, |s| {
        !s.contains("Submit #7 on the server?")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(posts_to(&mock, "/submit"), 0);

    // Confirmed: Tab to the Submit button, Enter.
    term.send_keys("U").expect("submit again");
    term.wait_for("the confirmation", STEP, |s| {
        s.contains("Submit #7 on the server?")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Tab Enter").expect("confirm");
    wait_until("the server got the submit", || {
        posts_to(&mock, "/submit") == 1
    });
    term.wait_for("the result", STEP, |s| s.contains("#7 submitted"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));

    // Abandon with the header button (a click), then confirm with a click.
    // (Located now, once the header has settled after the submit: the
    // buttons shift when the change's state does, and the screen read at the
    // start of the test is stale by then.)
    term.wait_for_settled_text("Abandon", std::time::Duration::from_millis(150), STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));
    // The confirm button is part of the wait: it is drawn with the dialog,
    // not necessarily in the frame that has the question's title.
    let confirm_below = |s: &xtask::pty::Screen| {
        let (_, question) = s.find("Abandon #7 on the server?")?;
        s.find_all("Abandon")
            .into_iter()
            .filter(|(_, row)| *row > question)
            .max_by_key(|(_, row)| *row)
    };
    // A click on a torn frame can be lost: re-locate and retry until asked.
    crate::common::retry_until(
        &mut term,
        "the confirmation and its button",
        6,
        std::time::Duration::from_secs(5),
        |t| {
            let screen = t.screen();
            if screen.contains("Abandon #7 on the server?") {
                return;
            }
            if let Some((c, r)) = screen.find("Abandon") {
                t.click(c + 1, r).expect("click Abandon");
            }
        },
        |s| confirm_below(s).is_some(),
    );
    assert_eq!(posts_to(&mock, "/abandon"), 0);
    crate::common::retry_until(
        &mut term,
        "the abandon sent",
        6,
        std::time::Duration::from_secs(5),
        |t| {
            if posts_to(&mock, "/abandon") == 1 {
                return;
            }
            if let Some((yc, yr)) = confirm_below(&t.screen()) {
                t.click(yc + 1, yr).expect("confirm the abandon");
            }
        },
        |_| posts_to(&mock, "/abandon") == 1,
    );
    wait_until("the server got the abandon", || {
        posts_to(&mock, "/abandon") == 1
    });
    term.wait_for("the result", STEP, |s| s.contains("#7 abandoned"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));
    term.stop().ok();
}

#[test]
fn the_palette_twins_of_submit_and_abandon_ask_the_same_questions() {
    let mock = mock_host::start(Options::default()).expect("the mock host");
    let repo = gerrit_fixture();
    let (mut term, _config) = launch_gerrit(&repo, "pty-gerrit-submit-palette", mock.port);
    open_review(&mut term);
    term.wait_for("the buttons", STEP, |s| s.contains("Abandon"))
        .unwrap_or_else(|e| panic!("{e}"));
    for (command, asked) in [
        ("Review: Submit Change", "Submit #7 on the server?"),
        ("Review: Abandon Change", "Abandon #7 on the server?"),
    ] {
        term.send_keys("C-p").expect("the palette");
        term.wait_for("the palette", STEP, |s| s.contains("Command Palette"))
            .unwrap_or_else(|e| panic!("{e}"));
        term.send_text(command).unwrap();
        term.wait_for("the command", STEP, |s| {
            s.contains(&format!("{command}..."))
        })
        .unwrap_or_else(|e| panic!("{e}"));
        term.send_keys("Enter").expect("run it");
        term.wait_for("the question", STEP, |s| s.contains(asked))
            .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));
        term.send_keys("Escape").expect("decline");
        term.wait_for("the question gone", STEP, |s| !s.contains(asked))
            .unwrap_or_else(|e| panic!("{e}"));
    }
    assert_eq!(posts_to(&mock, "/submit") + posts_to(&mock, "/abandon"), 0);
    term.stop().ok();
}
