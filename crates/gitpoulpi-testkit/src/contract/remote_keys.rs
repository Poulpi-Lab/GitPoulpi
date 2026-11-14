//! GitPoulpi's per-branch remote keys (`gitpoulpi-push-to`,
//! `gitpoulpi-pull-from`) live in the branch's config section, so they have
//! to follow it: a branch rename or delete, a remote rename or removal.
//! Both backends, one expectation.

use crate::TestRepoBuilder;
use gitpoulpi_core::GitBackend;

fn e(k: &str, v: &str) -> (String, String) {
    (k.to_string(), v.to_string())
}

fn push_to(
    backend: &dyn GitBackend,
    repo: &gitpoulpi_core::domain::RepoRef,
    b: &str,
) -> Vec<String> {
    let key = format!("branch.{b}.gitpoulpi-push-to");
    backend
        .config_section(repo, &format!("branch.{b}"))
        .unwrap()
        .into_iter()
        .filter(|(k, _)| *k == key)
        .map(|(_, v)| v)
        .collect()
}

/// `git branch -m` moves the whole section, `git branch -d` drops it: the
/// keys need no code of ours.
pub fn check_branch_rename_and_delete_carry_gitpoulpi_remote_keys(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .with_initial()
        .with_origin()
        .with_remote("gitlab")
        .with_remote("backup")
        .branch("feature")
        .track("feature", "origin")
        .push_targets("feature", &["gitlab", "backup:x"])
        .checkout("main")
        .build();
    let repo_ref = repo.repo_ref();

    backend
        .branch_rename(&repo_ref, "feature", "feature2")
        .unwrap();
    assert!(
        push_to(backend, &repo_ref, "feature").is_empty(),
        "the old section is gone"
    );
    assert_eq!(
        push_to(backend, &repo_ref, "feature2"),
        vec!["gitlab".to_string(), "backup:x".to_string()],
        "renamed with the branch, order kept"
    );
    // The upstream travelled too.
    assert!(backend
        .config_section(&repo_ref, "branch.feature2")
        .unwrap()
        .contains(&e("branch.feature2.remote", "origin")));

    backend.branch_delete(&repo_ref, "feature2").unwrap();
    assert!(
        backend
            .config_section(&repo_ref, "branch.feature2")
            .unwrap()
            .is_empty(),
        "the section goes with the branch"
    );
}

/// `remote_dependents` lists the branches whose keys name a remote, rename
/// rewrites the values (keeping a `:branch` part), remove drops them, and
/// a remote name with a slash behaves the same.
pub fn check_remote_rename_and_remove_rewrite_push_targets_and_pull_sources(
    backend: &dyn GitBackend,
) {
    let repo = TestRepoBuilder::new()
        .with_initial()
        .with_origin()
        .with_remote("gitlab")
        .with_remote("backup")
        .with_remote("grp/sub")
        .branch("feature")
        .track("feature", "origin")
        .push_targets("feature", &["gitlab", "backup:x", "grp/sub:topic"])
        .branch("other")
        .push_targets("other", &["backup"])
        .checkout("main")
        .build();
    let repo_ref = repo.repo_ref();
    backend
        .set_config_section(
            &repo_ref,
            "branch.other",
            &[
                e("branch.other.gitpoulpi-push-to", "backup"),
                e("branch.other.gitpoulpi-pull-from", "gitlab:g"),
            ],
        )
        .unwrap();

    assert_eq!(
        backend.remote_dependents(&repo_ref, "gitlab").unwrap(),
        vec!["branch.feature".to_string(), "branch.other".to_string()]
    );
    assert_eq!(
        backend.remote_dependents(&repo_ref, "backup").unwrap(),
        vec!["branch.feature".to_string(), "branch.other".to_string()]
    );
    assert_eq!(
        backend.remote_dependents(&repo_ref, "grp/sub").unwrap(),
        vec!["branch.feature".to_string()]
    );
    assert_eq!(
        backend.remote_dependents(&repo_ref, "origin").unwrap(),
        vec!["branch.feature".to_string()]
    );

    backend.remote_rename(&repo_ref, "gitlab", "gl").unwrap();
    assert_eq!(
        push_to(backend, &repo_ref, "feature"),
        vec![
            "gl".to_string(),
            "backup:x".to_string(),
            "grp/sub:topic".to_string()
        ],
        "renamed in place, order kept"
    );
    let other = backend.config_section(&repo_ref, "branch.other").unwrap();
    assert!(other.contains(&e("branch.other.gitpoulpi-pull-from", "gl:g")));
    assert!(other.contains(&e("branch.other.gitpoulpi-push-to", "backup")));
    assert!(backend
        .remote_dependents(&repo_ref, "gitlab")
        .unwrap()
        .is_empty());
    assert_eq!(
        backend.remote_dependents(&repo_ref, "gl").unwrap(),
        vec!["branch.feature".to_string(), "branch.other".to_string()]
    );

    backend.remote_rename(&repo_ref, "grp/sub", "grp2").unwrap();
    assert_eq!(
        push_to(backend, &repo_ref, "feature")[2],
        "grp2:topic".to_string()
    );

    backend.remote_remove(&repo_ref, "backup").unwrap();
    assert_eq!(
        push_to(backend, &repo_ref, "feature"),
        vec!["gl".to_string(), "grp2:topic".to_string()],
        "the removed remote's values are dropped, the others keep their order"
    );
    assert!(push_to(backend, &repo_ref, "other").is_empty());
    assert!(backend
        .config_section(&repo_ref, "branch.other")
        .unwrap()
        .contains(&e("branch.other.gitpoulpi-pull-from", "gl:g")));
    // git's own upstream was not ours to touch.
    assert!(backend
        .config_section(&repo_ref, "branch.feature")
        .unwrap()
        .contains(&e("branch.feature.remote", "origin")));
}

/// A repeated key is a multi-valued variable: written, it comes back in
/// the order given, on both backends.
pub fn check_set_config_section_keeps_multivalued_keys_in_order(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .with_initial()
        .branch("feature")
        .checkout("main")
        .build();
    let repo_ref = repo.repo_ref();
    let entries = vec![
        e("branch.feature.gitpoulpi-push-to", "zeta"),
        e("branch.feature.remote", "origin"),
        e("branch.feature.gitpoulpi-push-to", "alpha:b"),
        e("branch.feature.merge", "refs/heads/feature"),
        e("branch.feature.gitpoulpi-push-to", "mid/dle"),
        e("branch.feature.gitpoulpi-pull-from", "zeta"),
        e("branch.feature.gitpoulpi-pull-from", "alpha"),
    ];
    backend
        .set_config_section(&repo_ref, "branch.feature", &entries)
        .unwrap();
    let read = backend.config_section(&repo_ref, "branch.feature").unwrap();
    let only = |key: &str| -> Vec<String> {
        read.iter()
            .filter(|(k, _)| k == key)
            .map(|(_, v)| v.clone())
            .collect()
    };
    assert_eq!(
        only("branch.feature.gitpoulpi-push-to"),
        vec!["zeta", "alpha:b", "mid/dle"]
    );
    assert_eq!(
        only("branch.feature.gitpoulpi-pull-from"),
        vec!["zeta", "alpha"]
    );
    assert_eq!(only("branch.feature.remote"), vec!["origin"]);

    // Replacing with fewer values drops the rest, order still kept.
    backend
        .set_config_section(
            &repo_ref,
            "branch.feature",
            &[
                e("branch.feature.gitpoulpi-push-to", "b"),
                e("branch.feature.gitpoulpi-push-to", "a"),
            ],
        )
        .unwrap();
    assert_eq!(push_to(backend, &repo_ref, "feature"), vec!["b", "a"]);
    assert_eq!(
        backend
            .config_section(&repo_ref, "branch.feature")
            .unwrap()
            .len(),
        2
    );
}
