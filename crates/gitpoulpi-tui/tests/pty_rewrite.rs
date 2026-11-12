//! The history rewrite workbench end to end through a pty (`xtask::pty`): the
//! real binary, daemon and git. The chord opens it, a rule is added and filled
//! in by keys, Preview and Apply rewrite the history, Undo puts it back; the
//! same path by mouse through the dropdowns; a rule whose tool is missing is
//! greyed; the signature question appears on a signed fixture and blocks the
//! rewrite until it is answered.
//!
//! The rules are `pty_ui.rs`'s: nothing is located by a hard-coded
//! coordinate, nothing sleeps, every wait describes the state it waits for.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);

/// `git commit` as "Old Name <old@example.com>".
fn old_commit(repo: &FixtureRepo, message: &str, extra: &[&str]) {
    let mut args = vec![
        "-c",
        "user.name=Old Name",
        "-c",
        "user.email=old@example.com",
        "commit",
        "--allow-empty",
        "-q",
        "-m",
        message,
    ];
    args.extend_from_slice(extra);
    repo.git(&args);
}

/// `base` by the fixture's own identity, then two commits by old@example.com.
fn fixture() -> FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "base")
        .unwrap()
        .build();
    old_commit(&repo, "first by old", &[]);
    old_commit(&repo, "second by old", &[]);
    repo
}

fn emails(repo: &FixtureRepo) -> Vec<String> {
    repo.git(&["log", "--format=%ae"])
        .lines()
        .map(str::to_string)
        .collect()
}

fn launch_with(
    repo: &FixtureRepo,
    session: &str,
    envs: &[(&str, &str)],
) -> (Terminal, tempfile::TempDir) {
    let config = tempfile::tempdir().expect("temp config dir");
    let mut launch = common::launch(repo.root())
        .session(session)
        .config_dir(config.path())
        .size(200, 50);
    for (key, value) in envs {
        launch = launch.env(key, value);
    }
    let term = launch.spawn();
    term.wait_for("the fixture loaded", STARTUP, |s| {
        s.contains("second by old") && s.contains("first by old")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    (term, config)
}

fn launch(repo: &FixtureRepo, session: &str) -> (Terminal, tempfile::TempDir) {
    launch_with(repo, session, &[])
}

/// `Ctrl+G Shift+H`.
fn open_by_chord(term: &mut Terminal) {
    term.send_keys("C-g").unwrap();
    term.send_text("H").unwrap();
    term.wait_for("the workbench", STEP, |s| {
        s.contains("Rewrite history") && s.contains("Add a rule to start")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

/// An identity rule filled in by keys: `old@example.com` becomes
/// `new@example.org`.
fn fill_identity_by_keys(term: &mut Terminal) {
    term.send_text("a").unwrap();
    term.wait_for("the Add menu", STEP, |s| {
        s.contains("Replace an author") && s.contains("Authors")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("the rule's fields", STEP, |s| {
        s.contains("Email is") && s.contains("New email")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    // Field 1 is "Email is", field 4 "New email".
    term.send_keys("Down").unwrap();
    term.send_keys("Enter").unwrap();
    term.send_text("old@example.com").unwrap();
    term.send_keys("Enter").unwrap();
    for _ in 0..3 {
        term.send_keys("Down").unwrap();
    }
    term.send_keys("Enter").unwrap();
    term.send_text("new@example.org").unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("both emails in the fields", STEP, |s| {
        s.contains("old@example.com") && s.contains("new@example.org")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

fn wanted_owned(wanted: &[&str]) -> Vec<String> {
    wanted.iter().map(|s| s.to_string()).collect()
}

fn wait_for_emails(repo: &FixtureRepo, wanted: &[&str]) {
    let deadline = std::time::Instant::now() + STEP;
    // `try_git`: a ref swap in flight can fail one `git log`.
    let now = || {
        repo.try_git(&["log", "--format=%ae"])
            .map(|t| t.lines().map(str::to_string).collect::<Vec<_>>())
    };
    while now().as_deref() != Some(wanted_owned(wanted).as_slice()) {
        assert!(
            std::time::Instant::now() < deadline,
            "authors are {:?}, wanted {wanted:?}",
            now()
        );
        std::thread::sleep(Duration::from_millis(50));
    }
}

/// Chord, rule by keys, Preview, Apply, then Undo.
#[test]
fn a_rule_is_added_previewed_applied_and_undone_by_keys() {
    let repo = fixture();
    let original = emails(&repo);
    let tip = repo.git(&["rev-parse", "HEAD"]);
    let (mut term, _config) = launch(&repo, "pty-rewrite-keys");
    open_by_chord(&mut term);
    fill_identity_by_keys(&mut term);
    // Nothing has been touched by typing.
    assert_eq!(emails(&repo), original);
    term.send_keys("F5").unwrap();
    term.wait_for("the summary: two of the three commits change", STEP, |s| {
        s.contains("keep their id") && s.contains("2 of 3")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(emails(&repo), original, "the preview changes nothing");
    // Apply with Ctrl+S.
    term.send_keys("C-s").unwrap();
    wait_for_emails(&repo, &["new@example.org", "new@example.org", &original[2]]);
    term.wait_for("the result panel", STEP, |s| s.contains("Rewrote 2"))
        .unwrap_or_else(|e| panic!("{e}"));
    assert_ne!(repo.git(&["rev-parse", "HEAD"]), tip);
    // Ctrl+Z in the result panel is the journal's Undo.
    term.send_keys("C-z").unwrap();
    wait_for_emails(
        &repo,
        &original.iter().map(String::as_str).collect::<Vec<_>>(),
    );
    assert_eq!(repo.git(&["rev-parse", "HEAD"]), tip);
}

/// The palette opens the workbench with the rule in place; the dropdown of a
/// field is opened and picked with the mouse, never cycled by a click.
#[test]
fn the_palette_row_and_the_mouse_reach_a_rule_through_its_dropdown() {
    let repo = fixture();
    let (mut term, _config) = launch(&repo, "pty-rewrite-mouse");
    term.send_keys("C-p").unwrap();
    term.send_text("replace an author").unwrap();
    term.wait_for("the palette row", STEP, |s| {
        s.contains("Rewrite: Replace an Author...")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    let screen = term
        .wait_for("the rule's fields", STEP, |s| {
            s.contains("Rewrite history")
                && s.contains("Apply to")
                && s.contains("Author and committer")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (_, r) = screen.find("Author and committer").unwrap();
    // A click on the field opens the list; the value did not change.
    common::click_text_until(&mut term, "Author and committer", "the dropdown", |s| {
        s.contains("Committer")
            && s.contains("Author")
            && s.find_all("Author and committer").len() >= 2
    });
    // Pick the plain "Committer" row (the list's own row, below the field).
    common::retry_until(
        &mut term,
        "the field shows the pick and the list is gone",
        6,
        Duration::from_secs(10),
        |t| {
            if let Some((pc, pr)) = t
                .screen()
                .find_all("Committer")
                .into_iter()
                .filter(|&(_, row)| row > r)
                .min_by_key(|&(_, row)| row)
            {
                t.click(pc + 1, pr).expect("pick Committer");
            }
        },
        |s| s.find_all("Author and committer").is_empty() && s.contains("Committer"),
    );
    // The Add rule combo: a click opens the menu, a click picks a kind.
    common::retry_until(
        &mut term,
        "the Add menu",
        6,
        Duration::from_secs(10),
        |t| {
            if let Some((ac, ar)) = t.screen().find("+ Add rule") {
                t.click(ac + 2, ar).expect("click Add rule");
            }
        },
        |s| s.contains("Search and replace in messages"),
    );
    common::click_text_until(
        &mut term,
        "Search and replace in messages",
        "the second rule's fields",
        |s| s.contains("Replace with") && s.contains("Match case"),
    );
    // The cross closes the workbench.
    common::retry_until(
        &mut term,
        "the workbench closed",
        6,
        Duration::from_secs(10),
        |t| {
            if let Some((xc, xr)) = t.screen().find(" \u{2715} ") {
                t.click(xc + 1, xr).expect("click the cross");
            }
        },
        |s| !s.contains("Rewrite history"),
    );
}

/// A rule whose program is not installed is greyed, says what it needs and
/// refuses to preview.
#[test]
fn a_rule_that_needs_a_missing_tool_is_greyed_and_refuses_to_preview() {
    let repo = fixture();
    let (mut term, _config) = launch_with(
        &repo,
        "pty-rewrite-tool",
        &[("GITPOULPI_TOOL_GIT_LFS", "missing")],
    );
    term.send_keys("C-p").unwrap();
    term.send_text("migrate files to lfs").unwrap();
    term.wait_for("the palette row", STEP, |s| {
        s.contains("Rewrite: Migrate Files to LFS...")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("the reason and the hint", STEP, |s| {
        s.contains("git-lfs is needed and was not found")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    // Its pattern, then Preview: refused with the tool's name, nothing runs.
    term.send_keys("Enter").unwrap();
    term.send_text("*.bin").unwrap();
    term.send_keys("Enter").unwrap();
    term.send_keys("F5").unwrap();
    term.wait_for("the refusal in the footer", STEP, |s| {
        s.contains("git-lfs is needed and was not found") && !s.contains("keep their id")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

/// A fake `gpg.program`: git believes the commit was signed.
fn fake_gpg(dir: &std::path::Path) -> std::path::PathBuf {
    // An extensionless `sh` script: Git for Windows starts it through its own `sh`.
    let path = dir.join("fake-gpg");
    common::write_sh_script(
        &path,
        "#!/bin/sh\ncat >/dev/null\necho '[GNUPG:] SIG_CREATED ' >&2\nprintf -- '-----BEGIN PGP SIGNATURE-----\\n\\nZmFrZQ==\\n-----END PGP SIGNATURE-----\\n'\n",
    );
    path
}

/// A commit by old@example.com, signed by the fake program.
fn signed_commit(repo: &FixtureRepo, gpg: &std::path::Path, message: &str) {
    repo.git(&[
        "-c",
        "user.name=Old Name",
        "-c",
        "user.email=old@example.com",
        "-c",
        &format!(
            "gpg.program={}",
            gpg.display().to_string().replace('\\', "/")
        ),
        "-c",
        "user.signingkey=ABCDEF0123456789",
        "commit",
        "--allow-empty",
        "-q",
        "-S",
        "-m",
        message,
    ]);
}

fn signed_fixture(scratch: &std::path::Path) -> FixtureRepo {
    let repo = fixture();
    signed_commit(&repo, &fake_gpg(scratch), "signed tip");
    assert!(
        repo.git(&["cat-file", "-p", "HEAD"]).contains("gpgsig"),
        "the fixture's tip is signed"
    );
    repo
}

/// Signed commits that change: the daemon refuses to preview until the
/// choice is made; the question has no default and nothing runs behind it.
#[test]
fn the_signature_question_blocks_the_rewrite_until_it_is_answered() {
    let scratch = tempfile::tempdir().unwrap();
    let repo = signed_fixture(scratch.path());
    let original = emails(&repo);
    let (mut term, _config) = launch(&repo, "pty-rewrite-sign");
    open_by_chord(&mut term);
    fill_identity_by_keys(&mut term);
    term.send_keys("F5").unwrap();
    term.wait_for("the signature question with its choices", STEP, |s| {
        s.contains("Signed commits change")
            && s.contains("Remove the signatures")
            && s.contains("Re-sign only the commits that were signed")
            && s.contains("Re-sign every rewritten commit")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    // Apply while the question is open does nothing: no preview exists and
    // the question keeps the keyboard.
    term.send_keys("C-s").unwrap();
    term.wait_for("the question still open", STEP, |s| {
        s.contains("Signed commits change") && !s.contains("keep their id")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(emails(&repo), original);
    // Answer 1: strip. The preview runs, then Apply rewrites.
    term.send_text("1").unwrap();
    term.wait_for("the summary after the answer", STEP, |s| {
        s.contains("keep their id") && !s.contains("Signed commits change")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(emails(&repo), original, "the preview changes nothing");
    term.send_keys("C-s").unwrap();
    wait_for_emails(
        &repo,
        &[
            "new@example.org",
            "new@example.org",
            "new@example.org",
            &original[3],
        ],
    );
    assert!(
        !repo.git(&["cat-file", "-p", "HEAD"]).contains("gpgsig"),
        "the signature was removed as chosen"
    );
}

/// Re-signing needs gpg: with it missing the choice is refused and nothing is
/// previewed.
#[test]
fn choosing_to_re_sign_without_gpg_says_what_is_missing() {
    let scratch = tempfile::tempdir().unwrap();
    let repo = signed_fixture(scratch.path());
    let (mut term, _config) = launch_with(
        &repo,
        "pty-rewrite-nogpg",
        &[("GITPOULPI_TOOL_GPG", "missing")],
    );
    open_by_chord(&mut term);
    fill_identity_by_keys(&mut term);
    term.send_keys("F5").unwrap();
    term.wait_for("the signature question", STEP, |s| {
        s.contains("Signed commits change")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_text("2").unwrap();
    term.wait_for("gpg named as missing, no preview", STEP, |s| {
        s.contains("gpg is needed and was not found")
            && !s.contains("Signed commits change")
            && !s.contains("keep their id")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

/// The Log's commit menu: a "Rewrite history" submenu whose "From this commit
/// to HEAD" row opens the workbench with the scope starting after the
/// commit's parent.
#[test]
fn the_log_menu_opens_the_workbench_from_the_clicked_commit() {
    let repo = fixture();
    let (mut term, _config) = launch(&repo, "pty-rewrite-menu");
    common::right_click_text_until(&mut term, "first by old", "the commit menu", |s| {
        s.contains("Rewrite history")
    });
    common::click_text_until(&mut term, "Rewrite history", "the submenu rows", |s| {
        s.contains("From this commit to HEAD...") && s.contains("Open the workbench...")
    });
    common::click_text_until(
        &mut term,
        "From this commit to HEAD...",
        "the workbench scoped after the parent",
        |s| s.contains("Rewrite history") && s.contains("Only commits after"),
    );
}

/// A repository whose tip (by old@example.com) is signed with a generated
/// ssh key through the repository's own `gpg.format=ssh`; `None` without
/// `ssh-keygen`.
fn ssh_signed_fixture(scratch: &std::path::Path) -> Option<FixtureRepo> {
    let key = scratch.join("key");
    let made = std::process::Command::new("ssh-keygen")
        .args(["-q", "-t", "ed25519", "-N", "", "-C", "pty-test", "-f"])
        .arg(&key)
        .output();
    if !matches!(&made, Ok(out) if out.status.success()) {
        eprintln!("SKIPPED: ssh-keygen is not available");
        return None;
    }
    let repo = fixture();
    let public = std::fs::read_to_string(scratch.join("key.pub")).unwrap();
    std::fs::write(scratch.join("allowed"), format!("* {public}")).unwrap();
    repo.git(&["config", "gpg.format", "ssh"]);
    repo.git(&["config", "user.signingkey", key.to_str().unwrap()]);
    repo.git(&[
        "config",
        "gpg.ssh.allowedSignersFile",
        scratch.join("allowed").to_str().unwrap(),
    ]);
    old_commit(&repo, "signed tip", &["-S"]);
    Some(repo)
}

/// The question measures one signature and estimates the run ("Check
/// signing"), "re-sign only the signed" signs for real, and the history
/// shows a good signature afterwards.
#[test]
fn re_signing_the_signed_commits_checks_signing_and_leaves_good_signatures() {
    let scratch = tempfile::tempdir().unwrap();
    let Some(repo) = ssh_signed_fixture(scratch.path()) else {
        return;
    };
    let original = emails(&repo);
    let (mut term, _config) = launch(&repo, "pty-rewrite-resign");
    open_by_chord(&mut term);
    fill_identity_by_keys(&mut term);
    term.send_keys("F5").unwrap();
    term.wait_for("the signature question", STEP, |s| {
        s.contains("Signed commits change") && s.contains("Check signing")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    // Row 4 checks that signing works and estimates the time.
    term.send_text("4").unwrap();
    term.wait_for("the measured time and the estimate", STEP, |s| {
        s.contains("Signing works with ssh-keygen") && s.contains("Estimated")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(emails(&repo), original, "the check changes nothing");
    // Row 2: re-sign only the signed. The preview signs and says so.
    term.send_text("2").unwrap();
    term.wait_for("the summary with the signed commit", STEP, |s| {
        s.contains("keep their id") && s.contains("signed again: 1")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(emails(&repo), original, "the preview changes nothing");
    term.send_keys("C-s").unwrap();
    wait_for_emails(
        &repo,
        &[
            "new@example.org",
            "new@example.org",
            "new@example.org",
            &original[3],
        ],
    );
    let shown = repo.git(&["log", "-1", "--show-signature", "--format=%H"]);
    assert!(
        shown.contains("Good \"git\" signature"),
        "the tip carries a good signature:\n{shown}"
    );
    assert!(
        repo.try_git(&["verify-commit", "HEAD"]).is_some(),
        "git verifies the new tip"
    );
}
