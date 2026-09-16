//! `commit_messages`: the workspace search's commit read.

use crate::TestRepoBuilder;
use gitpoulpi_core::domain::{CommitMessage, CommitMessagesQuery};
use gitpoulpi_core::GitBackend;

fn subjects(commits: &[CommitMessage]) -> Vec<String> {
    let mut s: Vec<String> = commits.iter().map(|c| c.subject.clone()).collect();
    s.sort();
    s
}

/// Every commit of every branch, remote-tracking branch, tag and HEAD, with
/// git's subject and body, the author and the time; never a stash commit.
pub fn check_commit_messages_cover_every_ref_and_split_subject_from_body(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .commit_with_body("on main", "the body\n\nsecond paragraph")
        .branch("feature")
        .commit("only on feature")
        .checkout("main")
        .commit("main again")
        .tag("v1")
        .with_origin()
        .build();
    let repo_ref = repo.repo_ref();
    // A detached HEAD commit no branch reaches, and a stash (whose commits
    // are not history).
    repo.git(&["checkout", "-q", "--detach"]);
    std::fs::write(repo.root().join("loose.txt"), "x").unwrap();
    repo.git(&["add", "loose.txt"]);
    repo.git(&["commit", "-q", "-m", "detached work"]);
    // A commit only a tag of a tag reaches: peeled all the way.
    repo.git(&["commit", "-q", "--allow-empty", "-m", "tag only"]);
    repo.git(&["tag", "-a", "inner", "-m", "inner"]);
    repo.git(&["tag", "-a", "outer", "-m", "outer", "inner"]);
    repo.git(&["tag", "-d", "inner"]);
    repo.git(&["checkout", "-q", "--detach", "HEAD~1"]);
    std::fs::write(repo.root().join("loose.txt"), "changed").unwrap();
    repo.git(&["stash", "push", "-q", "-m", "parked"]);

    let all = backend
        .commit_messages(&repo_ref, &CommitMessagesQuery::default())
        .unwrap();
    assert_eq!(
        subjects(&all.commits),
        vec![
            "detached work",
            "main again",
            "on main",
            "only on feature",
            "tag only"
        ]
    );
    assert!(!all.replace && !all.truncated);

    let on_main = all.commits.iter().find(|c| c.subject == "on main").unwrap();
    assert_eq!(on_main.body, "the body\n\nsecond paragraph");
    assert!(!on_main.author.is_empty());
    assert!(on_main.time > 1_000_000_000);
    let plain = all
        .commits
        .iter()
        .find(|c| c.subject == "main again")
        .unwrap();
    assert_eq!(plain.body, "");
    assert_eq!(
        plain.id.to_hex(),
        repo.git(&["rev-parse", "main"]),
        "the id is the commit's"
    );

    // The tips: the branches, the remote-tracking branch, the tag (peeled)
    // and the detached HEAD - sorted, without duplicates.
    let expect_tip = |rev: &str| {
        let oid = gitpoulpi_core::domain::Oid::from_hex(&repo.git(&["rev-parse", rev])).unwrap();
        assert!(all.tips.contains(&oid), "{rev} is a tip");
    };
    for rev in ["main", "feature", "v1", "HEAD"] {
        expect_tip(rev);
    }
    let mut sorted = all.tips.clone();
    sorted.sort_by_key(|o| o.0);
    sorted.dedup();
    assert_eq!(sorted, all.tips, "tips are sorted and unique");
    // The stash commit is not a tip.
    let stash = repo.git(&["rev-parse", "refs/stash"]);
    assert!(all.tips.iter().all(|t| t.to_hex() != stash));

    // Newest first: the list is non-increasing in time.
    assert!(all.commits.windows(2).all(|w| w[0].time >= w[1].time));
}

/// Asked with the previous answer's tips, only what is new comes back; a
/// repository with nothing new answers an empty list and the same tips.
pub fn check_commit_messages_with_known_tips_return_only_new_commits(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .commit("one")
        .commit("two")
        .branch("side")
        .commit("side one")
        .checkout("main")
        .build();
    let repo_ref = repo.repo_ref();
    let first = backend
        .commit_messages(&repo_ref, &CommitMessagesQuery::default())
        .unwrap();
    assert_eq!(first.commits.len(), 3);

    let again = backend
        .commit_messages(
            &repo_ref,
            &CommitMessagesQuery {
                known_tips: first.tips.clone(),
                limit: 0,
            },
        )
        .unwrap();
    assert!(again.commits.is_empty() && !again.replace);
    assert_eq!(again.tips, first.tips);

    // A commit on each branch, and a new branch pointing at an old commit.
    repo.git(&["commit", "-q", "--allow-empty", "-m", "three"]);
    repo.git(&["checkout", "-q", "side"]);
    repo.git(&["commit", "-q", "--allow-empty", "-m", "side two"]);
    repo.git(&["branch", "old-pointer", "main~1"]);
    let next = backend
        .commit_messages(
            &repo_ref,
            &CommitMessagesQuery {
                known_tips: first.tips.clone(),
                limit: 0,
            },
        )
        .unwrap();
    assert_eq!(subjects(&next.commits), vec!["side two", "three"]);
    assert!(!next.replace);
    assert_ne!(next.tips, first.tips);
}

/// A known tip no ref reaches any more (amend, reset, deleted branch) makes
/// the answer a full replacement.
pub fn check_commit_messages_after_a_rewrite_replace_everything(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .commit("kept")
        .commit("to be amended")
        .build();
    let repo_ref = repo.repo_ref();
    let first = backend
        .commit_messages(&repo_ref, &CommitMessagesQuery::default())
        .unwrap();
    repo.git(&["commit", "-q", "--amend", "-m", "amended"]);
    let after = backend
        .commit_messages(
            &repo_ref,
            &CommitMessagesQuery {
                known_tips: first.tips.clone(),
                limit: 0,
            },
        )
        .unwrap();
    assert!(after.replace);
    assert_eq!(subjects(&after.commits), vec!["amended", "kept"]);
}

/// `limit` keeps the newest commits and says it cut; a repository with no
/// commit answers empty, not an error.
pub fn check_commit_messages_limit_and_unborn_repository(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .commit("a")
        .commit("b")
        .commit("c")
        .build();
    let repo_ref = repo.repo_ref();
    let two = backend
        .commit_messages(
            &repo_ref,
            &CommitMessagesQuery {
                known_tips: Vec::new(),
                limit: 2,
            },
        )
        .unwrap();
    assert_eq!(two.commits.len(), 2);
    assert!(two.truncated);
    let all = backend
        .commit_messages(&repo_ref, &CommitMessagesQuery::default())
        .unwrap();
    assert_eq!(all.commits.len(), 3);
    assert!(!all.truncated);

    let empty = TestRepoBuilder::new().build();
    let none = backend
        .commit_messages(&empty.repo_ref(), &CommitMessagesQuery::default())
        .unwrap();
    assert!(none.commits.is_empty() && none.tips.is_empty() && !none.replace);
}
