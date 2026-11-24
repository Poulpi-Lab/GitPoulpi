//! Where a tag lives (here, on a remote, both, or at two commits), and
//! the deletes and fetches that move it between those states.
//!
//! git keeps no remote-tracking refs for tags; `refs()` answers from the
//! remote-tag mirror (see `REMOTE_TAG_MIRROR_PREFIX`). Every check here
//! asks the *bare remote itself* what it holds, never only the backend.

use super::{block_on, head_oid};
use crate::TestRepoBuilder;
use gitpoulpi_core::domain::{FetchOptions, Ref, RepoRef, TagSelection, TagWhere};
use gitpoulpi_core::GitBackend;

fn tag(backend: &dyn GitBackend, repo: &RepoRef, name: &str) -> Option<Ref> {
    backend
        .refs(repo)
        .unwrap()
        .into_iter()
        .find(|r| r.tag_name() == Some(name))
}

fn where_of(backend: &dyn GitBackend, repo: &RepoRef, name: &str) -> Option<TagWhere> {
    tag(backend, repo, name).and_then(|r| r.tag_where())
}

fn push(backend: &dyn GitBackend, repo: &RepoRef, name: &str) {
    block_on(backend.tag_push(repo, "origin", &TagSelection::Named(vec![name.to_string()])))
        .unwrap_or_else(|e| panic!("push {name}: {e}"));
}

fn remote_tags(repo: &crate::FixtureRepo) -> Vec<String> {
    repo.git_in_remote(&["tag", "--list"])
        .lines()
        .map(str::to_string)
        .collect()
}

/// The four states, told apart: a tag only here is `local`, one pushed is
/// `both`, one deleted here but still on the remote is `remote` (a row
/// with `tag_sync.local == false`, pointing at the remote's commit), and
/// one the remote holds at another commit is `differs`.
pub fn check_a_tag_reports_where_it_lives(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new().with_initial().with_origin().build();
    let r = repo.repo_ref();
    let first = head_oid(backend, &r);
    for name in ["only-here", "shared", "gone-here", "moved"] {
        backend.tag_create(&r, name, first, None).unwrap();
    }
    for name in ["shared", "gone-here", "moved"] {
        push(backend, &r, name);
    }
    // `gone-here` is deleted locally only; `moved` is re-made locally at
    // a newer commit (the remote keeps the first).
    backend.tag_delete(&r, "gone-here").unwrap();
    repo.git(&["commit", "--allow-empty", "-m", "second"]);
    repo.git(&["tag", "-f", "moved"]);

    assert_eq!(where_of(backend, &r, "only-here"), Some(TagWhere::Local));
    assert_eq!(where_of(backend, &r, "shared"), Some(TagWhere::Both));
    assert_eq!(where_of(backend, &r, "moved"), Some(TagWhere::Differs));
    let ghost = tag(backend, &r, "gone-here").expect("a remote-only tag is still listed");
    assert_eq!(ghost.tag_where(), Some(TagWhere::Remote));
    assert!(ghost.is_remote_only_tag());
    assert_eq!(ghost.target, first, "it points at the remote's commit");
    assert!(
        !ghost.is_unpushed_tag(),
        "remote-only is not 'unpushed': the remote has it"
    );
    let sync = tag(backend, &r, "moved").unwrap().tag_sync.unwrap();
    assert_eq!(sync.differs_on, vec!["origin".to_string()]);
    assert!(sync.pushed_to.is_empty());
    assert!(
        repo.try_git(&["rev-parse", "--verify", "-q", "refs/tags/gone-here"])
            .is_none(),
        "the remote-only row must not create a local ref"
    );
}

/// A mirror ref left by a remote that was removed outside GitPoulpi (a
/// plain `git remote remove` run by hand cannot know about it) is stale:
/// its tags are not "on" that remote and no row may name it. A tag only
/// that mirror had reads as local only; a tag the live remote also has
/// keeps only the live remote.
pub fn check_a_mirror_of_a_removed_remote_is_ignored(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new().with_initial().with_origin().build();
    let r = repo.repo_ref();
    let first = head_oid(backend, &r);
    for name in ["only-archive", "shared"] {
        backend.tag_create(&r, name, first, None).unwrap();
    }
    push(backend, &r, "shared");
    let head = repo.git(&["rev-parse", "HEAD"]);
    for name in ["only-archive", "shared"] {
        repo.git(&[
            "update-ref",
            &format!("refs/gitpoulpi/remote-tags/archive/{name}"),
            &head,
        ]);
    }
    // A remote-only ghost recorded just by the stale mirror must not be listed.
    repo.git(&[
        "update-ref",
        "refs/gitpoulpi/remote-tags/archive/ghost",
        &head,
    ]);

    assert_eq!(where_of(backend, &r, "only-archive"), Some(TagWhere::Local));
    assert!(tag(backend, &r, "only-archive").unwrap().is_unpushed_tag());
    assert_eq!(where_of(backend, &r, "shared"), Some(TagWhere::Both));
    let sync = tag(backend, &r, "shared").unwrap().tag_sync.unwrap();
    assert_eq!(sync.pushed_to, vec!["origin".to_string()]);
    assert!(tag(backend, &r, "ghost").is_none());
}

/// Delete locally keeps the remote's copy (the tag is then remote only);
/// delete on the remote keeps the local one (then local only); both leaves
/// neither - asked of git on both sides.
pub fn check_tag_delete_scopes_are_independent(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new().with_initial().with_origin().build();
    let r = repo.repo_ref();
    let head = head_oid(backend, &r);
    for name in ["a", "b", "c"] {
        backend.tag_create(&r, name, head, None).unwrap();
        push(backend, &r, name);
    }

    backend.tag_delete(&r, "a").unwrap();
    assert_eq!(where_of(backend, &r, "a"), Some(TagWhere::Remote));
    assert!(remote_tags(&repo).contains(&"a".to_string()));

    block_on(backend.tag_delete_remote(&r, "origin", "b")).unwrap();
    assert_eq!(where_of(backend, &r, "b"), Some(TagWhere::Local));
    assert!(!remote_tags(&repo).contains(&"b".to_string()));
    assert!(tag(backend, &r, "b").unwrap().is_unpushed_tag());

    backend.tag_delete(&r, "c").unwrap();
    block_on(backend.tag_delete_remote(&r, "origin", "c")).unwrap();
    assert!(tag(backend, &r, "c").is_none(), "gone from the listing");
    assert!(!remote_tags(&repo).contains(&"c".to_string()));
}

/// The maintainer's report: delete a tag locally, fetch, it comes back (git
/// follows the remote's tags). Deleted on the remote too, a fetch leaves
/// it gone. Both halves, so the combined delete is proven to stick.
pub fn check_a_fetch_brings_a_tag_back_unless_deleted_on_the_remote(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .with_initial()
        .with_origin()
        .push("main")
        .build();
    let r = repo.repo_ref();
    let head = head_oid(backend, &r);
    backend.tag_create(&r, "v1", head, None).unwrap();
    push(backend, &r, "v1");

    backend.tag_delete(&r, "v1").unwrap();
    block_on(backend.fetch(&r, "origin", &FetchOptions::default())).unwrap();
    assert_eq!(
        where_of(backend, &r, "v1"),
        Some(TagWhere::Both),
        "a local-only delete is undone by the next fetch"
    );

    backend.tag_delete(&r, "v1").unwrap();
    block_on(backend.tag_delete_remote(&r, "origin", "v1")).unwrap();
    block_on(backend.fetch(&r, "origin", &FetchOptions::default())).unwrap();
    assert!(
        tag(backend, &r, "v1").is_none(),
        "deleted both places: the fetch must not resurrect it"
    );
}

/// "Fetch this tag": a remote-only tag becomes a real local tag at the
/// remote's commit; a missing tag is an error, and a local tag at another
/// commit is never overwritten.
pub fn check_tag_fetch_brings_one_remote_only_tag(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new().with_initial().with_origin().build();
    let r = repo.repo_ref();
    let head = head_oid(backend, &r);
    backend.tag_create(&r, "v1", head, None).unwrap();
    push(backend, &r, "v1");
    backend.tag_delete(&r, "v1").unwrap();
    assert_eq!(where_of(backend, &r, "v1"), Some(TagWhere::Remote));

    block_on(backend.tag_fetch(&r, "origin", "v1")).expect("fetch the tag");
    let fetched = tag(backend, &r, "v1").unwrap();
    assert_eq!(fetched.tag_where(), Some(TagWhere::Both));
    assert_eq!(fetched.target, head);
    assert!(repo
        .try_git(&["rev-parse", "--verify", "-q", "refs/tags/v1"])
        .is_some());

    assert!(
        block_on(backend.tag_fetch(&r, "origin", "nope")).is_err(),
        "a tag the remote lacks is an error"
    );

    repo.git(&["commit", "--allow-empty", "-m", "second"]);
    repo.git(&["tag", "-f", "v1"]);
    let moved = tag(backend, &r, "v1").unwrap().target;
    assert!(
        block_on(backend.tag_fetch(&r, "origin", "v1")).is_err(),
        "a tag that differs is refused, not overwritten"
    );
    assert_eq!(tag(backend, &r, "v1").unwrap().target, moved);
}

/// Someone else deletes a tag on the remote. Even a plain fetch (no
/// `--prune`) must stop calling it published, or the badge lies; the local
/// tag stays, now unpushed. With `prune_tags` the local tag goes too, but
/// only a tag that *was* on the remote: one never pushed is untouched.
pub fn check_a_fetch_notices_a_tag_deleted_on_the_remote(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .with_initial()
        .with_origin()
        .push("main")
        .build();
    let r = repo.repo_ref();
    let head = head_oid(backend, &r);
    for name in ["pushed", "pushed2", "mine"] {
        backend.tag_create(&r, name, head, None).unwrap();
    }
    push(backend, &r, "pushed");
    push(backend, &r, "pushed2");

    repo.git_in_remote(&["tag", "-d", "pushed"]);
    let out = block_on(backend.fetch(&r, "origin", &FetchOptions::default())).unwrap();
    assert!(out.pruned_tags.is_empty(), "off by default");
    assert_eq!(
        where_of(backend, &r, "pushed"),
        Some(TagWhere::Local),
        "the mirror was pruned, the local tag was not"
    );
    assert_eq!(where_of(backend, &r, "pushed2"), Some(TagWhere::Both));

    repo.git_in_remote(&["tag", "-d", "pushed2"]);
    let out = block_on(backend.fetch(
        &r,
        "origin",
        &FetchOptions {
            prune_tags: true,
            ..Default::default()
        },
    ))
    .unwrap();
    assert_eq!(out.pruned_tags, vec!["pushed2".to_string()]);
    assert!(tag(backend, &r, "pushed2").is_none());
    assert!(
        tag(backend, &r, "mine").is_some(),
        "a tag that was never on the remote is never pruned"
    );
    assert!(
        tag(backend, &r, "pushed").is_some(),
        "already unpublished before this fetch: not a candidate"
    );
}

// -- moving a tag, and merging a branch into one --

fn tag_object_type(repo: &crate::FixtureRepo, name: &str) -> String {
    repo.git(&["cat-file", "-t", &format!("refs/tags/{name}")])
}

fn write_commit(repo: &crate::FixtureRepo, file: &str, body: &str, message: &str) {
    std::fs::write(repo.root().join(file), body).unwrap();
    repo.git(&["add", file]);
    repo.git(&["commit", "-q", "-m", message]);
}

/// A lightweight tag moves forward to a descendant commit, and stays
/// lightweight. Moving it onto the commit it already names is a no-op.
pub fn check_tag_move_fast_forwards_a_lightweight_tag(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .with_initial()
        .commit("second")
        .build();
    let r = repo.repo_ref();
    let tip = head_oid(backend, &r);
    let first = backend
        .resolve(&r, "HEAD~1")
        .expect("the first commit resolves");
    backend.tag_create(&r, "v1", first, None).unwrap();

    backend.tag_move(&r, "v1", first, false).expect("no-op");
    backend
        .tag_move(&r, "v1", tip, false)
        .expect("fast-forward");

    assert_eq!(tag(backend, &r, "v1").unwrap().target, tip);
    assert_eq!(tag_object_type(&repo, "v1"), "commit", "still lightweight");
    assert!(backend.tag_move(&r, "nope", tip, false).is_err());
}

/// Moving to a commit that is not a descendant is refused with the typed
/// error and changes nothing, unless `force` says the owner chose it.
pub fn check_tag_move_refuses_a_non_fast_forward_unless_forced(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .with_initial()
        .commit("second")
        .build();
    let r = repo.repo_ref();
    let tip = head_oid(backend, &r);
    let first = backend.resolve(&r, "HEAD~1").unwrap();
    backend.tag_create(&r, "v1", tip, None).unwrap();

    // Backwards: `first` is an ancestor of the tag's commit, not a descendant.
    let refused = backend.tag_move(&r, "v1", first, false);
    assert!(
        matches!(refused, Err(gitpoulpi_core::Error::NotFastForward)),
        "{refused:?}"
    );
    assert_eq!(tag(backend, &r, "v1").unwrap().target, tip, "unchanged");

    backend.tag_move(&r, "v1", first, true).expect("forced");
    assert_eq!(tag(backend, &r, "v1").unwrap().target, first);
}

/// An annotated tag is re-created annotated with its message and tagger
/// line verbatim; only the commit it points at changes. The old tag object
/// stays in the repository (an undo restores it).
pub fn check_tag_move_keeps_an_annotated_tag_annotated(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .with_initial()
        .annotated_tag("v1")
        .commit("second")
        .build();
    let r = repo.repo_ref();
    let tip = head_oid(backend, &r);
    let old_object = repo.git(&["rev-parse", "refs/tags/v1"]);
    let old_raw = repo.git(&["cat-file", "tag", "refs/tags/v1"]);

    backend.tag_move(&r, "v1", tip, false).expect("move");

    assert_eq!(tag_object_type(&repo, "v1"), "tag", "still annotated");
    assert_ne!(repo.git(&["rev-parse", "refs/tags/v1"]), old_object);
    assert_eq!(tag(backend, &r, "v1").unwrap().target, tip);
    let new_raw = repo.git(&["cat-file", "tag", "refs/tags/v1"]);
    let after_object = |raw: &str| raw.split_once('\n').unwrap().1.to_string();
    assert_eq!(
        after_object(&new_raw),
        after_object(&old_raw),
        "type, name, tagger and message are carried over verbatim"
    );
    assert!(new_raw.starts_with(&format!("object {}", tip.to_hex())));
    assert_eq!(repo.git(&["cat-file", "-t", &old_object]), "tag");
}

/// Moving a pushed tag leaves the remote at the old commit: the tag now
/// reports `differs`, and nothing was pushed on its own.
pub fn check_tag_move_leaves_the_remote_copy_and_reports_differs(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new().with_initial().with_origin().build();
    let r = repo.repo_ref();
    let first = head_oid(backend, &r);
    backend.tag_create(&r, "v1", first, None).unwrap();
    push(backend, &r, "v1");
    repo.git(&["commit", "-q", "--allow-empty", "-m", "second"]);
    let tip = head_oid(backend, &r);
    assert_eq!(where_of(backend, &r, "v1"), Some(TagWhere::Both));

    backend.tag_move(&r, "v1", tip, false).unwrap();

    assert_eq!(where_of(backend, &r, "v1"), Some(TagWhere::Differs));
    assert_eq!(
        repo.git_in_remote(&["rev-parse", "refs/tags/v1"]),
        first.to_hex(),
        "the remote still holds the old position"
    );
}

/// Merging a branch into a tag makes a merge commit whose parents are the
/// tag's old commit and the branch tip, moves the tag onto it, and moves
/// nothing else: no branch, not HEAD, not the working tree.
pub fn check_tag_merge_branch_makes_a_merge_and_moves_only_the_tag(backend: &dyn GitBackend) {
    for annotated in [false, true] {
        let repo = TestRepoBuilder::new().with_initial().build();
        let r = repo.repo_ref();
        let default_branch = repo.git(&["symbolic-ref", "--short", "HEAD"]);
        repo.git(&["checkout", "-q", "-b", "side"]);
        write_commit(&repo, "side.txt", "side\n", "side work");
        repo.git(&["checkout", "-q", &default_branch]);
        write_commit(&repo, "main.txt", "main\n", "main work");
        let main_tip = head_oid(backend, &r);
        let side_tip = backend.resolve(&r, "side").unwrap();
        backend
            .tag_create(&r, "v1", main_tip, annotated.then_some("release"))
            .unwrap();

        let merged = backend
            .tag_merge_branch(&r, "v1", "side", None)
            .expect("merge into tag");

        assert_eq!(tag(backend, &r, "v1").unwrap().target, merged);
        assert_eq!(
            repo.git(&["rev-list", "--parents", "-n1", &merged.to_hex()]),
            format!(
                "{} {} {}",
                merged.to_hex(),
                main_tip.to_hex(),
                side_tip.to_hex()
            ),
            "parents: the tag's old commit, then the branch tip"
        );
        assert_eq!(
            tag_object_type(&repo, "v1"),
            if annotated { "tag" } else { "commit" },
            "the tag keeps its kind"
        );
        assert_eq!(
            head_oid(backend, &r),
            main_tip,
            "HEAD's branch did not move"
        );
        assert_eq!(
            repo.git(&["symbolic-ref", "--short", "HEAD"]),
            default_branch
        );
        assert_eq!(backend.resolve(&r, "side").unwrap(), side_tip);
        assert!(repo.git(&["status", "--porcelain"]).is_empty());
        assert_eq!(
            repo.git(&["show", &format!("{}:side.txt", merged.to_hex())]),
            "side"
        );
        assert_eq!(
            repo.git(&["show", &format!("{}:main.txt", merged.to_hex())]),
            "main",
            "both sides' files are in the merge"
        );
        // Merging again has nothing left to merge.
        assert!(backend.tag_merge_branch(&r, "v1", "side", None).is_err());
    }
}

/// A conflicting merge is refused and nothing at all changes: the tag, the
/// branches, the index and the working tree.
pub fn check_tag_merge_branch_conflict_changes_nothing(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .commit_file("a.txt", "base\n", "base")
        .build();
    let r = repo.repo_ref();
    let default_branch = repo.git(&["symbolic-ref", "--short", "HEAD"]);
    repo.git(&["checkout", "-q", "-b", "side"]);
    write_commit(&repo, "a.txt", "two\n", "side edit");
    repo.git(&["checkout", "-q", &default_branch]);
    write_commit(&repo, "a.txt", "one\n", "main edit");
    let main_tip = head_oid(backend, &r);
    backend.tag_create(&r, "v1", main_tip, None).unwrap();
    let side_tip = backend.resolve(&r, "side").unwrap();

    let refused = backend.tag_merge_branch(&r, "v1", "side", None);

    assert!(
        matches!(refused, Err(gitpoulpi_core::Error::Conflict { .. })),
        "{refused:?}"
    );
    assert_eq!(tag(backend, &r, "v1").unwrap().target, main_tip);
    assert_eq!(head_oid(backend, &r), main_tip);
    assert_eq!(backend.resolve(&r, "side").unwrap(), side_tip);
    assert!(repo.git(&["status", "--porcelain"]).is_empty());
    assert!(!repo.root().join(".git/MERGE_HEAD").exists());
}
