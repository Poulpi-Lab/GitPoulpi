//! Both backends' `reword` must write the *same commit objects*, byte for
//! byte, save the committer line (which carries "now").
//!
//! The contract suite checks each backend against what a reword must
//! keep; this checks them against each other, on one repository: reword
//! with libgit2, record the objects, put the branch back, reword again
//! with system git, compare. A difference in message normalisation,
//! author-date handling or parent mapping shows up here as a diff.

use gitpoulpi_cli_backend::SystemGitBackend;
use gitpoulpi_core::GitBackend;
use gitpoulpi_git2_backend::Git2Backend;
use gitpoulpi_testkit::FixtureRepoBuilder;

fn objects_without_committer(repo: &gitpoulpi_testkit::FixtureRepo) -> Vec<String> {
    let oids = repo.git(&["rev-list", "--topo-order", "HEAD"]);
    oids.lines()
        .map(|oid| {
            // A parent's oid carries *its* committer time, so it is
            // compared by tree instead; the contract suite pins the
            // parent structure itself.
            repo.git(&["cat-file", "commit", oid])
                .lines()
                .filter(|l| !l.starts_with("committer "))
                .map(|l| match l.strip_prefix("parent ") {
                    Some(p) => format!(
                        "parent-tree {}",
                        repo.git(&["rev-parse", &format!("{p}^{{tree}}")])
                    ),
                    None => l.to_string(),
                })
                .collect::<Vec<_>>()
                .join("\n")
        })
        .collect()
}

#[test]
fn both_backends_write_the_same_commits() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("base")
        .unwrap()
        .commit("target")
        .unwrap()
        .commit("child")
        .unwrap()
        .build();
    let repo_ref = repo.repo_ref();
    let old_head = repo.git(&["rev-parse", "HEAD"]);
    let target = Git2Backend::new().resolve(&repo_ref, "HEAD~1").unwrap();
    let message = "  \r\nnew subject\r\n\r\nnew body  \r\n\n";

    Git2Backend::new()
        .reword(&repo_ref, target, message)
        .unwrap();
    let by_git2 = objects_without_committer(&repo);

    repo.git(&["update-ref", "HEAD", &old_head]);
    SystemGitBackend::new()
        .reword(&repo_ref, target, message)
        .unwrap();
    let by_system_git = objects_without_committer(&repo);

    assert_eq!(by_git2, by_system_git);
    assert!(
        by_git2[1].ends_with("\n\nnew subject\n\nnew body"),
        "{by_git2:?}"
    );
}

/// Amend: with the committer forced too, even the committer line agrees, so
/// the objects are compared whole (parents by tree, as above).
#[test]
fn both_backends_write_the_same_amended_commits() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("base")
        .unwrap()
        .commit("target")
        .unwrap()
        .commit("child")
        .unwrap()
        .build();
    let repo_ref = repo.repo_ref();
    let old_head = repo.git(&["rev-parse", "HEAD"]);
    let target = Git2Backend::new().resolve(&repo_ref, "HEAD~1").unwrap();
    let amend = gitpoulpi_core::domain::CommitAmend {
        message: Some("\r\nnew subject\r\n\r\nnew body  \r\n".into()),
        author_name: Some("Ada Lovelace".into()),
        author_email: Some("ada@example.org".into()),
        author_date: Some("2001-02-03T04:05:06-07:00".into()),
        committer_name: Some("Grace Hopper".into()),
        committer_email: Some("grace@example.org".into()),
        committer_date: Some("1262401445 +0200".into()),
    };

    Git2Backend::new()
        .amend_commit(&repo_ref, target, &amend)
        .unwrap();
    let by_git2 = objects_without_committer(&repo);
    let committer_git2 = repo.git(&["cat-file", "commit", "HEAD~1"]);

    repo.git(&["update-ref", "HEAD", &old_head]);
    SystemGitBackend::new()
        .amend_commit(&repo_ref, target, &amend)
        .unwrap();
    let by_system_git = objects_without_committer(&repo);
    let committer_system = repo.git(&["cat-file", "commit", "HEAD~1"]);

    assert_eq!(by_git2, by_system_git);
    assert_eq!(committer_git2, committer_system, "committer line included");
    assert!(committer_git2.contains("author Ada Lovelace <ada@example.org> 981198306 -0700"));
    assert!(committer_git2.contains("committer Grace Hopper <grace@example.org> 1262401445 +0200"));
}
