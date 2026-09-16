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
