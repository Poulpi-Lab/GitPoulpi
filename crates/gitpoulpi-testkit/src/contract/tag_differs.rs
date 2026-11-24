//! A tag that `differs` (here and on a remote at two commits) and the two
//! ways out: take the remote's version (no network, from the tag mirror), or
//! force-push ours (leased on what the mirror says the remote has).
//!
//! Every check asks the *bare remote itself* and git, never only the backend.

use super::{block_on, head_oid};
use crate::{FixtureRepo, TestRepoBuilder};
use gitpoulpi_core::domain::{Oid, Ref, RepoRef, TagSelection, TagWhere};
use gitpoulpi_core::{Error, GitBackend};

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

fn local_ref(repo: &FixtureRepo, name: &str) -> String {
    repo.git(&["rev-parse", &format!("refs/tags/{name}")])
}

fn remote_ref(repo: &FixtureRepo, name: &str) -> String {
    repo.git_in_remote(&["rev-parse", &format!("refs/tags/{name}")])
}

fn mirror_ref(repo: &FixtureRepo, name: &str) -> String {
    repo.git(&[
        "rev-parse",
        &format!("refs/gitpoulpi/remote-tags/origin/{name}"),
    ])
}

/// A repository whose remote holds `name` (made by `make_remote_side`, at
/// the first commit) while the local tag is re-made at a second commit.
/// Returns the first commit.
fn differing(
    backend: &dyn GitBackend,
    name: &str,
    annotated_on_remote: bool,
) -> (FixtureRepo, Oid) {
    let repo = TestRepoBuilder::new()
        .with_initial()
        .with_origin()
        .push("main")
        .build();
    let r = repo.repo_ref();
    let first = head_oid(backend, &r);
    let message = annotated_on_remote.then_some("release notes");
    backend.tag_create(&r, name, first, message).unwrap();
    push(backend, &r, name);
    repo.git(&["commit", "--allow-empty", "-m", "second"]);
    repo.git(&["tag", "-f", name]);
    assert_eq!(where_of(backend, &r, name), Some(TagWhere::Differs));
    (repo, first)
}

/// "Use the remote's version" of a lightweight tag: the local tag moves to
/// the remote's commit, from the mirror alone (the remote is not asked), and
/// the tag then reads as on both.
pub fn check_take_remote_moves_a_lightweight_tag(backend: &dyn GitBackend) {
    let (repo, first) = differing(backend, "v1", false);
    let r = repo.repo_ref();
    backend
        .tag_take_remote(&r, "origin", "v1")
        .expect("take the remote's tag");
    assert_eq!(local_ref(&repo, "v1"), first.to_hex());
    assert_eq!(where_of(backend, &r, "v1"), Some(TagWhere::Both));
    assert_eq!(
        repo.git_in_remote(&["tag", "--list"]),
        "v1",
        "the remote was not touched"
    );
}

/// The remote's tag is annotated: the local tag becomes that very tag
/// object (still annotated, same id), not a lightweight tag at its commit.
pub fn check_take_remote_keeps_an_annotated_tag_object(backend: &dyn GitBackend) {
    let (repo, _) = differing(backend, "v2", true);
    let r = repo.repo_ref();
    assert_eq!(
        mirror_ref(&repo, "v2"),
        remote_ref(&repo, "v2"),
        "the mirror holds the tag object the remote has"
    );
    backend.tag_take_remote(&r, "origin", "v2").unwrap();
    assert_eq!(repo.git(&["cat-file", "-t", "refs/tags/v2"]), "tag");
    assert_eq!(local_ref(&repo, "v2"), remote_ref(&repo, "v2"));
    assert_eq!(where_of(backend, &r, "v2"), Some(TagWhere::Both));
}

/// There is nothing to take from a remote that is not known to have the tag
/// (no mirror), and no local tag to move for a remote-only one; neither
/// changes anything.
pub fn check_take_remote_refuses_what_it_cannot_take(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .with_initial()
        .with_origin()
        .push("main")
        .build();
    let r = repo.repo_ref();
    let head = head_oid(backend, &r);
    backend.tag_create(&r, "mine", head, None).unwrap();
    backend.tag_create(&r, "gone", head, None).unwrap();
    push(backend, &r, "gone");
    backend.tag_delete(&r, "gone").unwrap();

    assert!(backend.tag_take_remote(&r, "origin", "mine").is_err());
    assert!(
        backend.tag_take_remote(&r, "origin", "gone").is_err(),
        "a remote-only tag is fetched, not taken"
    );
    assert!(backend.tag_take_remote(&r, "nowhere", "mine").is_err());
    assert_eq!(local_ref(&repo, "mine"), head.to_hex());
    assert!(repo
        .try_git(&["rev-parse", "--verify", "-q", "refs/tags/gone"])
        .is_none());
}

/// "Push mine (force)": the remote's tag moves to ours (lightweight or
/// annotated, the object as is), the mirror follows and the tag reads as on
/// both.
pub fn check_tag_force_push_moves_the_remote_tag(backend: &dyn GitBackend) {
    let (repo, first) = differing(backend, "v1", false);
    let r = repo.repo_ref();
    assert_eq!(remote_ref(&repo, "v1"), first.to_hex());
    block_on(backend.tag_force_push(&r, "origin", "v1")).expect("force push with a good lease");
    assert_eq!(remote_ref(&repo, "v1"), local_ref(&repo, "v1"));
    assert_eq!(mirror_ref(&repo, "v1"), local_ref(&repo, "v1"));
    assert_eq!(where_of(backend, &r, "v1"), Some(TagWhere::Both));

    // An annotated tag of ours goes as the tag object.
    let second = repo.git(&["rev-parse", "HEAD"]);
    backend.tag_delete(&r, "v1").unwrap();
    let at = Oid::from_hex(&second).unwrap();
    backend.tag_create(&r, "v1", at, Some("mine")).unwrap();
    repo.git(&["commit", "--allow-empty", "-m", "third"]);
    let third = Oid::from_hex(&repo.git(&["rev-parse", "HEAD"])).unwrap();
    backend.tag_delete(&r, "v1").unwrap();
    backend
        .tag_create(&r, "v1", third, Some("mine again"))
        .unwrap();
    assert_eq!(where_of(backend, &r, "v1"), Some(TagWhere::Differs));
    block_on(backend.tag_force_push(&r, "origin", "v1")).unwrap();
    assert_eq!(
        repo.git_in_remote(&["cat-file", "-t", "refs/tags/v1"]),
        "tag"
    );
    assert_eq!(remote_ref(&repo, "v1"), local_ref(&repo, "v1"));
    assert_eq!(mirror_ref(&repo, "v1"), local_ref(&repo, "v1"));
}

/// The remote moved since the last fetch: the lease is stale, the push is
/// refused (typed, `StaleLease`) and the remote keeps what it has. After a
/// fetch the new value is known and the push goes.
pub fn check_tag_force_push_refuses_a_stale_lease(backend: &dyn GitBackend) {
    let (repo, _) = differing(backend, "v1", false);
    let r = repo.repo_ref();
    // Someone else moves the tag on the remote to a commit we never saw.
    repo.push_from_elsewhere("main", "theirs");
    repo.git_in_remote(&["tag", "-f", "v1", "main"]);
    let theirs = remote_ref(&repo, "v1");
    let mine = local_ref(&repo, "v1");
    let mirror = mirror_ref(&repo, "v1");

    let err = block_on(backend.tag_force_push(&r, "origin", "v1"))
        .expect_err("a stale lease must refuse");
    assert!(
        matches!(
            &err,
            Error::PushRejected(p) if p.reason == gitpoulpi_core::domain::PushRejectReason::StaleLease
        ),
        "expected a StaleLease rejection, got {err:?}"
    );
    assert_eq!(remote_ref(&repo, "v1"), theirs, "nothing was overwritten");
    assert_eq!(local_ref(&repo, "v1"), mine);
    assert_eq!(mirror_ref(&repo, "v1"), mirror);

    block_on(backend.fetch(
        &r,
        "origin",
        &gitpoulpi_core::domain::FetchOptions::default(),
    ))
    .unwrap();
    assert_eq!(mirror_ref(&repo, "v1"), theirs, "the fetch learned it");
    block_on(backend.tag_force_push(&r, "origin", "v1")).expect("lease honoured after a fetch");
    assert_eq!(remote_ref(&repo, "v1"), mine);
}

/// A tag the mirror does not show on that remote is not force-pushed (that is
/// a plain push), nor is one that already matches.
pub fn check_tag_force_push_refuses_what_is_not_differing(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .with_initial()
        .with_origin()
        .push("main")
        .build();
    let r = repo.repo_ref();
    let head = head_oid(backend, &r);
    backend.tag_create(&r, "new", head, None).unwrap();
    backend.tag_create(&r, "same", head, None).unwrap();
    push(backend, &r, "same");
    assert!(block_on(backend.tag_force_push(&r, "origin", "new")).is_err());
    assert!(block_on(backend.tag_force_push(&r, "origin", "same")).is_err());
    assert!(!repo.git_in_remote(&["tag", "--list"]).contains("new"));
}
