use crate::TestRepoBuilder;
use gitpoulpi_core::GitBackend;

/// `ahead_behind` counts each side of a fork, in both directions, and is
/// `(0, 0)` against itself.
pub fn check_ahead_behind_counts_both_sides_of_a_fork(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .with_initial()
        .branch("feature")
        .commit_file("f1.txt", "1", "f1")
        .commit_file("f2.txt", "2", "f2")
        .checkout("main")
        .commit_file("m1.txt", "1", "m1")
        .commit_file("m2.txt", "2", "m2")
        .commit_file("m3.txt", "3", "m3")
        .build();
    let r = repo.repo_ref();
    let feature = backend.resolve(&r, "feature").unwrap();
    let main = backend.resolve(&r, "main").unwrap();
    assert_eq!(backend.ahead_behind(&r, feature, main).unwrap(), (2, 3));
    assert_eq!(backend.ahead_behind(&r, main, feature).unwrap(), (3, 2));
    assert_eq!(backend.ahead_behind(&r, main, main).unwrap(), (0, 0));
    let fork = backend.resolve(&r, "main~3").unwrap();
    assert_eq!(
        backend.ahead_behind(&r, main, fork).unwrap(),
        (3, 0),
        "an ancestor is only behind"
    );
}

/// `config_entries` lists the repository-local entries under the asked
/// prefixes (keys as `config --list` spells them: variable lower-cased,
/// subsection as written), in file order, and nothing else.
pub fn check_config_entries_lists_local_keys_under_prefixes(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new().with_initial().build();
    repo.git(&["config", "gitflow.branch.master", "main"]);
    repo.git(&["config", "gitflow.prefix.feature", "feat/"]);
    repo.git(&["config", "git-town.main-branch", "main"]);
    repo.git(&["config", "branch.Feature/X.gitpoulpi-parent", "develop"]);
    repo.git(&["config", "user.name", "Someone"]);
    let r = repo.repo_ref();

    let flow = backend.config_entries(&r, &["gitflow."]).unwrap();
    assert_eq!(
        flow,
        [
            ("gitflow.branch.master".to_string(), "main".to_string()),
            ("gitflow.prefix.feature".to_string(), "feat/".to_string()),
        ]
    );
    let several = backend
        .config_entries(&r, &["git-town.", "branch."])
        .unwrap();
    assert!(
        several.contains(&("git-town.main-branch".to_string(), "main".to_string())),
        "{several:?}"
    );
    assert!(
        several.contains(&(
            "branch.Feature/X.gitpoulpi-parent".to_string(),
            "develop".to_string()
        )),
        "the subsection keeps its case: {several:?}"
    );
    assert!(several.iter().all(|(k, _)| !k.starts_with("user.")));
    assert!(backend
        .config_entries(&r, &["nothing."])
        .unwrap()
        .is_empty());
    assert!(backend.config_entries(&r, &[]).unwrap().is_empty());
}

/// `MergeOptions::no_ff` makes a merge commit where a fast-forward was
/// possible; `squash` lands the branch as one commit with the given message
/// and a single parent (the branch's history is not brought in).
pub fn check_merge_no_ff_and_squash_shape_the_result(backend: &dyn GitBackend) {
    use gitpoulpi_core::domain::{MergeOptions, MergeOutcome, RefKind};
    let repo = TestRepoBuilder::new()
        .with_initial()
        .branch("topic")
        .commit_file("t1.txt", "1", "t1")
        .commit_file("t2.txt", "2", "t2")
        .checkout("main")
        .branch("copy")
        .checkout("main")
        .build();
    let r = repo.repo_ref();
    let branch = |name: &str| {
        backend
            .refs(&r)
            .unwrap()
            .into_iter()
            .find(|x| matches!(&x.kind, RefKind::LocalBranch { name: n } if n == name))
            .unwrap()
    };
    let topic = branch("topic");
    let main_before = backend.resolve(&r, "main").unwrap();

    // Squash on main: one new commit, one parent, both files, no history.
    let outcome = backend
        .merge(
            &r,
            &topic,
            &MergeOptions {
                squash: true,
                message: Some("Squashed topic".into()),
                ..MergeOptions::default()
            },
        )
        .unwrap();
    let MergeOutcome::Merged(squashed) = outcome else {
        panic!("a squash makes a commit, got {outcome:?}");
    };
    assert_eq!(backend.resolve(&r, "main").unwrap(), squashed);
    assert_eq!(
        repo.git(&["log", "-1", "--format=%s", "main"]).trim(),
        "Squashed topic"
    );
    assert_eq!(
        repo.git(&["rev-list", "--parents", "-n1", "main"])
            .split_whitespace()
            .count(),
        2,
        "a squash commit has exactly one parent"
    );
    assert!(repo.root().join("t1.txt").exists() && repo.root().join("t2.txt").exists());
    assert_ne!(squashed, main_before);
    assert_ne!(squashed, topic.target);

    // --no-ff on a branch that could fast-forward: a merge commit.
    repo.git(&["checkout", "-q", "copy"]);
    let outcome = backend
        .merge(
            &r,
            &topic,
            &MergeOptions {
                no_ff: true,
                message: Some("Merge topic".into()),
                ..MergeOptions::default()
            },
        )
        .unwrap();
    let MergeOutcome::Merged(merged) = outcome else {
        panic!("--no-ff makes a merge commit, got {outcome:?}");
    };
    assert_eq!(
        repo.git(&["rev-list", "--parents", "-n1", "copy"])
            .split_whitespace()
            .count(),
        3,
        "a merge commit has two parents"
    );
    assert_eq!(backend.resolve(&r, "copy").unwrap(), merged);
    assert_eq!(
        repo.git(&["log", "-1", "--format=%s", "copy"]).trim(),
        "Merge topic"
    );

    // Merging what is already merged changes nothing.
    assert_eq!(
        backend
            .merge(
                &r,
                &topic,
                &MergeOptions {
                    no_ff: true,
                    ..MergeOptions::default()
                }
            )
            .unwrap(),
        MergeOutcome::UpToDate
    );
}

/// The fast-forward policies of a merge (the owner's scenario: a branch made
/// from `main`, two commits on it, `main` untouched). `no_ff` makes a merge
/// commit whose first parent is the old `main`; no flag fast-forwards;
/// `fast_forward_only` fast-forwards when it can and is the typed
/// `NotFastForward` (nothing moves) once the branches have diverged.
pub fn check_merge_fast_forward_policies(backend: &dyn GitBackend) {
    use gitpoulpi_core::domain::{MergeOptions, MergeOutcome, RefKind};
    let repo = TestRepoBuilder::new()
        .with_initial()
        .branch("topic")
        .commit_file("t1.txt", "1", "t1")
        .commit_file("t2.txt", "2", "t2")
        .checkout("main")
        .branch("plain")
        .checkout("main")
        .branch("strict")
        .checkout("main")
        .build();
    let r = repo.repo_ref();
    let branch = |name: &str| {
        backend
            .refs(&r)
            .unwrap()
            .into_iter()
            .find(|x| matches!(&x.kind, RefKind::LocalBranch { name: n } if n == name))
            .unwrap()
    };
    let topic = branch("topic");
    let old_main = backend.resolve(&r, "main").unwrap();

    // no_ff: a merge commit, first parent the old main, second the topic tip.
    let outcome = backend
        .merge(
            &r,
            &topic,
            &MergeOptions {
                no_ff: true,
                ..MergeOptions::default()
            },
        )
        .unwrap();
    let MergeOutcome::Merged(merged) = outcome else {
        panic!("--no-ff makes a merge commit, got {outcome:?}");
    };
    let parents = repo.git(&["rev-list", "--parents", "-n1", "main"]);
    let parents: Vec<&str> = parents.split_whitespace().collect();
    assert_eq!(parents.len(), 3, "two parents: {parents:?}");
    assert_eq!(parents[0], merged.to_hex());
    assert_eq!(
        parents[1],
        old_main.to_hex(),
        "first parent is the old main"
    );
    assert_eq!(parents[2], topic.target.to_hex());
    assert!(
        repo.git(&["log", "-1", "--format=%s", "main"])
            .starts_with("Merge branch 'topic'"),
        "the default message of a merge"
    );

    // No flag: a fast-forward.
    repo.git(&["checkout", "-q", "plain"]);
    let outcome = backend.merge(&r, &topic, &MergeOptions::default()).unwrap();
    assert_eq!(outcome, MergeOutcome::FastForwarded(topic.target));

    // fast_forward_only: a fast-forward when it can...
    repo.git(&["checkout", "-q", "strict"]);
    let outcome = backend
        .merge(
            &r,
            &topic,
            &MergeOptions {
                fast_forward_only: true,
                ..MergeOptions::default()
            },
        )
        .unwrap();
    assert_eq!(outcome, MergeOutcome::FastForwarded(topic.target));

    // ...and the typed refusal once the branch has diverged from the topic.
    repo.git(&["checkout", "-q", "-b", "other", &old_main.to_hex()]);
    std::fs::write(repo.root().join("o.txt"), "o").unwrap();
    repo.git(&["add", "o.txt"]);
    repo.git(&["commit", "-q", "-m", "other"]);
    let before = backend.resolve(&r, "other").unwrap();
    let err = backend
        .merge(
            &r,
            &topic,
            &MergeOptions {
                fast_forward_only: true,
                ..MergeOptions::default()
            },
        )
        .unwrap_err();
    assert!(
        matches!(err, gitpoulpi_core::Error::NotFastForward),
        "typed refusal, got {err:?}"
    );
    assert_eq!(
        backend.resolve(&r, "other").unwrap(),
        before,
        "nothing moved"
    );
}

/// A branch's recorded parent is one more key in its `branch.<name>` section:
/// writing the section back keeps the others, and `config_entries` reads it.
pub fn check_a_recorded_parent_is_a_branch_config_key(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .with_initial()
        .branch("feature/x")
        .checkout("main")
        .build();
    repo.git(&["config", "branch.feature/x.remote", "origin"]);
    repo.git(&["config", "branch.feature/x.merge", "refs/heads/feature/x"]);
    let r = repo.repo_ref();
    let section = "branch.feature/x";
    let mut entries = backend.config_section(&r, section).unwrap();
    entries.push((
        "branch.feature/x.gitpoulpi-parent".to_string(),
        "develop".to_string(),
    ));
    backend.set_config_section(&r, section, &entries).unwrap();
    let back = backend.config_section(&r, section).unwrap();
    assert!(
        back.iter()
            .any(|(k, v)| k == "branch.feature/x.gitpoulpi-parent" && v == "develop"),
        "{back:?}"
    );
    assert!(back
        .iter()
        .any(|(k, v)| k == "branch.feature/x.remote" && v == "origin"));
    let listed = backend.config_entries(&r, &["branch."]).unwrap();
    assert!(listed.contains(&(
        "branch.feature/x.gitpoulpi-parent".to_string(),
        "develop".to_string()
    )));
    // Removing the key leaves the rest.
    let without: Vec<_> = back
        .into_iter()
        .filter(|(k, _)| !k.ends_with(".gitpoulpi-parent"))
        .collect();
    backend.set_config_section(&r, section, &without).unwrap();
    let back = backend.config_section(&r, section).unwrap();
    assert!(
        back.iter().all(|(k, _)| !k.ends_with(".gitpoulpi-parent")),
        "{back:?}"
    );
    assert!(back.iter().any(|(k, _)| k == "branch.feature/x.merge"));
}

/// `MergeOptions { squash, no_commit }`: the other branch's changes are
/// staged and no commit is made (HEAD stays), with no `MERGE_HEAD` left
/// behind (a squash is not a merge in progress). This is the Merge options
/// setting's "Squash" mode.
pub fn check_merge_squash_without_commit_stages_the_changes(backend: &dyn GitBackend) {
    use gitpoulpi_core::domain::{MergeOptions, MergeOutcome, RefKind};
    let repo = TestRepoBuilder::new()
        .with_initial()
        .branch("topic")
        .commit_file("t1.txt", "1", "t1")
        .commit_file("t2.txt", "2", "t2")
        .checkout("main")
        .build();
    let r = repo.repo_ref();
    let topic = backend
        .refs(&r)
        .unwrap()
        .into_iter()
        .find(|x| matches!(&x.kind, RefKind::LocalBranch { name } if name == "topic"))
        .unwrap();
    let head_before = backend.resolve(&r, "main").unwrap();

    let outcome = backend
        .merge(
            &r,
            &topic,
            &MergeOptions {
                squash: true,
                no_commit: true,
                ..MergeOptions::default()
            },
        )
        .unwrap();
    assert!(
        matches!(outcome, MergeOutcome::Merged(_)),
        "a staged squash reports Merged, got {outcome:?}"
    );
    assert_eq!(backend.resolve(&r, "main").unwrap(), head_before);
    assert_eq!(
        repo.git(&["diff", "--cached", "--name-only"]).trim(),
        "t1.txt\nt2.txt"
    );
    assert!(repo.root().join("t1.txt").exists());
    assert!(
        !repo.root().join(".git/MERGE_HEAD").exists(),
        "a squash leaves no merge in progress"
    );
    assert_eq!(backend.operation_state(&r).unwrap(), None);
    assert!(
        repo.root().join(".git/SQUASH_MSG").exists(),
        "the squash message is left for the commit panel"
    );
}
