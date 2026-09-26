use crate::mock::*;
use crate::support::*;

/// GitHub Flow style: a feature finishes by merge request. The daemon pushes
/// the branch (to a local bare repository: `pushurl` keeps the real push off
/// the mock's address), then the client opens the existing Create Pull
/// Request form prefilled: target = the parent, title from the branch.
#[test]
fn a_merge_request_finish_pushes_then_opens_the_prefilled_pull_request_form() {
    let mock = start_mock();
    let repo = fixture(mock.port);
    let bare = repo.remote_path().expect("the bare origin").to_path_buf();
    repo.git(&[
        "config",
        "remote.origin.pushurl",
        bare.to_str().expect("utf-8 path"),
    ]);
    repo.git(&["checkout", "-q", "-b", "work/add-login", "main"]);
    std::fs::write(repo.root().join("login.txt"), "x\n").unwrap();
    repo.git(&["add", "login.txt"]);
    repo.git(&[
        "-c",
        "user.name=Test",
        "-c",
        "user.email=test@example.com",
        "commit",
        "-q",
        "-m",
        "Add the login form",
    ]);
    std::fs::create_dir_all(repo.state_dir()).unwrap();
    std::fs::write(
        repo.state_dir().join("workflow.toml"),
        r#"template = "github-flow"

[[base]]
name = "main"
protected = true

[[topic]]
id = "feature"
label = "Feature"
pattern = "work/{slug}"
parent = "main"
finish = [{ into = "main", merge = "squash" }]
finish_mode = "merge-request"
delete_after_finish = true
"#,
    )
    .unwrap();
    let (mut term, _config) = launch(
        &repo,
        "pty-hosting-flow-mr",
        &config_toml(mock.port, ""),
        COLS,
        "Add the login",
    );
    term.wait_for("the Flow button", STEP, |s| s.contains("Flow \u{25BE}"))
        .unwrap_or_else(|e| panic!("{e}"));

    term.send_keys("C-p").unwrap();
    term.wait_for_text("Command Palette", STEP).unwrap();
    term.send_text("Finish Current").unwrap();
    term.wait_for("the palette entry", STEP, |s| {
        s.contains("Flow: Finish Current Branch...")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();

    // The preview lists the push and the request, highlights what Undo
    // cannot take back, and the form of the request comes after the run.
    let preview = term
        .wait_for("the preview of the merge request finish", STEP, |s| {
            s.contains("This will:")
                && s.contains("Push work/add-login to origin and track it")
                && s.contains("Open a merge request from work/add-login into main")
                && s.contains("Undo will not take back:")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(preview.contains("! "), "{}", preview.text());
    term.send_keys("Enter").unwrap();

    let form = term
        .wait_for("the prefilled Create Pull Request form", STEP, |s| {
            s.contains("Create Pull Request")
                && s.contains("From repo")
                && s.contains("Add login")
                && s.contains("work/add-login")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(
        form.contains("main"),
        "the target is the parent:\n{}",
        form.text()
    );
    // The daemon pushed before the form opened.
    assert!(
        !repo
            .git_in_remote(&["branch", "--list", "work/add-login"])
            .is_empty(),
        "the branch is on the remote"
    );
    term.send_keys("Esc").unwrap();
    term.wait_for_text_to_go("From repo", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
}
