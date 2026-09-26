//! `hosting refs`: the issue / pull request references a message or a
//! branch holds, read through the repository's links.

use crate::common::*;

#[test]
#[cfg(feature = "jira")]
fn a_message_names_only_the_linked_jira_key_not_utf_8() {
    let cli = Cli::new(
        "refs-jira",
        &integration(
            "jira",
            "jira_cloud",
            "https://acme.atlassian.net",
            "account = \"me@x.io\"",
        ),
        &[],
    );
    let repo = fixture();
    write_state(
        &repo,
        "[issues_source]\nintegration = \"jira\"\nproject = \"ABC\"\n",
    );

    let out = cli.json(
        repo.root(),
        &[
            "hosting",
            "refs",
            "--message",
            "Fixes ABC-12, see UTF-8 and SHA-256",
        ],
    );
    let refs = out["refs"].as_array().unwrap();
    assert_eq!(refs.len(), 1, "{out}");
    assert_eq!(refs[0]["text"], "ABC-12");
    assert_eq!(refs[0]["kind"], "issue");
    assert_eq!(refs[0]["reference"], "ABC-12");
    assert_eq!(refs[0]["url"], "https://acme.atlassian.net/browse/ABC-12");

    let out = cli.json(
        repo.root(),
        &["hosting", "refs", "--branch", "feature/ABC-7-login"],
    );
    assert_eq!(out["refs"][0]["reference"], "ABC-7");
    assert_eq!(
        out["refs"][0]["url"],
        "https://acme.atlassian.net/browse/ABC-7"
    );

    // A branch of the repository: HEAD's commit message.
    repo.git(&["commit", "--allow-empty", "-q", "-m", "Closes ABC-99"]);
    let out = cli.json(repo.root(), &["hosting", "refs", "--commit", "HEAD"]);
    assert_eq!(out["refs"][0]["reference"], "ABC-99");
}

#[test]
fn nothing_linked_means_nothing_is_a_reference() {
    let cli = Cli::new("refs-unlinked", "", &[]);
    let repo = fixture();
    let out = cli.json(
        repo.root(),
        &["hosting", "refs", "--message", "Fixes ABC-12 and #7"],
    );
    assert_eq!(out["refs"].as_array().unwrap().len(), 0, "{out}");
}
