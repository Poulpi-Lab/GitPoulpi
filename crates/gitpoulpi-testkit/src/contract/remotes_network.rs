use super::{block_on, head_oid};
use crate::TestRepoBuilder;
use gitpoulpi_core::domain::{
    CloneOptions, ClonePhase, CloneProgress, FetchOptions, LogOptions, PullMode, PullOptions,
    PullOutcome, PushOptions, PushRejectReason, RefKind, ResetMode, TagSelection, TagSync,
    REMOTE_TAG_MIRROR_PREFIX,
};
use gitpoulpi_core::GitBackend;

/// The clone has to be a repository the *same backend* can then read,
/// and the `RepoRef` it returns has to be the one every later request
/// will be keyed on.
pub fn check_clone_produces_a_usable_repository(backend: &dyn GitBackend) {
    let source = TestRepoBuilder::new()
        .with_initial()
        .commit("second")
        .build();
    let dest_dir = tempfile::tempdir().unwrap();
    let dest = dest_dir.path().join("clone");

    let cloned = block_on(backend.clone(
        &source.root().to_string_lossy(),
        &dest,
        &CloneOptions::default(),
    ))
    .unwrap();

    assert_eq!(
        cloned.root.canonicalize().unwrap(),
        dest.canonicalize().unwrap()
    );
    assert_eq!(
        cloned.id,
        gitpoulpi_core::domain::RepoId::from_path(&dest),
        "the clone's id must be the one `discover` would produce for it"
    );
    assert_eq!(
        backend.discover(&dest).unwrap().repo,
        cloned,
        "cloning and then discovering the destination must agree"
    );

    let log = backend.log(&cloned, &LogOptions::default()).unwrap();
    assert_eq!(log.nodes.len(), 2);
    assert_eq!(log.nodes[0].summary, "second");
    assert!(
        backend
            .remotes(&cloned)
            .unwrap()
            .iter()
            .any(|r| r.name == "origin"),
        "a clone must have an origin remote"
    );
}

/// The progress contract, which is what makes a progress bar drawable: at
/// least one report, the last one is `Done` at 100%, and no report ever
/// claims more than 100%.
///
/// Intermediate phases are deliberately *not* asserted: a local clone of a
/// two-commit repository can finish before git prints anything.
pub fn check_clone_reports_progress_that_ends_at_done(backend: &dyn GitBackend) {
    use std::sync::Mutex;

    let source = TestRepoBuilder::new().with_initial().build();
    let dest_dir = tempfile::tempdir().unwrap();
    let dest = dest_dir.path().join("clone");

    let seen: Mutex<Vec<CloneProgress>> = Mutex::new(Vec::new());
    let sink = |p: CloneProgress| seen.lock().unwrap().push(p);

    block_on(backend.clone_with_progress(
        &source.root().to_string_lossy(),
        &dest,
        &CloneOptions::default(),
        &sink,
    ))
    .unwrap();

    let seen = seen.into_inner().unwrap();
    assert!(!seen.is_empty(), "a clone must report at least once");
    let last = *seen.last().unwrap();
    assert_eq!(last.phase, ClonePhase::Done);
    assert_eq!(last.percent, 100);
    assert!(
        seen.iter().all(|p| p.percent <= 100),
        "no progress report may exceed 100%: {seen:?}"
    );
    assert!(
        seen.iter()
            .all(|p| p.current.zip(p.total).map_or(true, |(c, t)| c <= t)),
        "a count may never exceed its total: {seen:?}"
    );
}

/// `--branch` has to select what is checked out, not merely be accepted.
pub fn check_clone_honours_the_branch_option(backend: &dyn GitBackend) {
    let source = TestRepoBuilder::new()
        .with_initial()
        .branch("side")
        .commit("on side")
        .checkout("main")
        .build();
    let dest_dir = tempfile::tempdir().unwrap();
    let dest = dest_dir.path().join("clone");

    let opts = CloneOptions {
        branch: Some("side".to_string()),
        ..CloneOptions::default()
    };
    let cloned = block_on(backend.clone(&source.root().to_string_lossy(), &dest, &opts)).unwrap();

    assert_eq!(
        backend
            .discover(&cloned.root)
            .unwrap()
            .head_branch
            .as_deref(),
        Some("side")
    );
    assert_eq!(
        backend.log(&cloned, &LogOptions::default()).unwrap().nodes[0].summary,
        "on side"
    );
}

/// `remote.<name>` sections round trip like `branch.<name>` ones, a batch
/// read answers in the order asked, and `remote_dependents` names the
/// branches whose upstream is that remote.
pub fn check_remote_config_sections_and_dependents(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .with_initial()
        .branch("feature")
        .checkout("main")
        .with_origin()
        .push("main")
        .push("feature")
        .track("main", "origin")
        .track("feature", "origin")
        .build();
    let repo_ref = repo.repo_ref();

    let section = backend.config_section(&repo_ref, "remote.origin").unwrap();
    assert!(section.iter().any(|(k, _)| k == "remote.origin.url"));
    assert!(section.iter().any(|(k, _)| k == "remote.origin.fetch"));
    assert!(
        section.iter().all(|(k, _)| k.starts_with("remote.origin.")),
        "only the section's own keys: {section:?}"
    );

    let batch = backend
        .config_sections(
            &repo_ref,
            &[
                "branch.feature".to_string(),
                "remote.origin".to_string(),
                "remote.nobody".to_string(),
            ],
        )
        .unwrap();
    assert_eq!(batch.len(), 3);
    assert_eq!(batch[0].0, "branch.feature");
    assert_eq!(
        batch[0].1,
        backend.config_section(&repo_ref, "branch.feature").unwrap()
    );
    assert_eq!(batch[1].1, section);
    assert!(batch[2].1.is_empty());

    assert_eq!(
        backend.remote_dependents(&repo_ref, "origin").unwrap(),
        vec!["branch.feature".to_string(), "branch.main".to_string()]
    );
    assert!(backend
        .remote_dependents(&repo_ref, "nobody")
        .unwrap()
        .is_empty());

    // The whole section goes and comes back.
    backend
        .set_config_section(&repo_ref, "remote.origin", &[])
        .unwrap();
    assert!(backend
        .config_section(&repo_ref, "remote.origin")
        .unwrap()
        .is_empty());
    backend
        .set_config_section(&repo_ref, "remote.origin", &section)
        .unwrap();
    assert_eq!(
        backend.config_section(&repo_ref, "remote.origin").unwrap(),
        section
    );
}

pub fn check_remote_add_list_remove(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new().with_initial().build();
    let repo_ref = repo.repo_ref();

    assert!(backend.remotes(&repo_ref).unwrap().is_empty());

    backend
        .remote_add(&repo_ref, "origin", "https://example.invalid/repo.git")
        .unwrap();
    let remotes = backend.remotes(&repo_ref).unwrap();
    assert_eq!(remotes.len(), 1);
    assert_eq!(remotes[0].name, "origin");
    assert_eq!(remotes[0].fetch_url, "https://example.invalid/repo.git");

    backend.remote_remove(&repo_ref, "origin").unwrap();
    assert!(backend.remotes(&repo_ref).unwrap().is_empty());
}

/// A tag that has only ever existed here is unpushed, and says so.
///
/// `tag_sync: None` would mean *we did not look*, which the UI renders
/// differently from "no remote has it"; a backend that never populated the
/// field would otherwise pass every other check here.
pub fn check_a_new_tag_is_reported_as_unpushed(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new().with_initial().with_origin().build();
    let repo_ref = repo.repo_ref();
    let head = head_oid(backend, &repo_ref);

    backend
        .tag_create(&repo_ref, "v1", head, None)
        .expect("lightweight tag");

    let tag = find_tag(backend, &repo_ref, "v1");
    assert_eq!(
        tag.tag_sync,
        Some(TagSync::default()),
        "a tag nobody pushed must carry an empty, *present* sync state"
    );
    assert!(tag.is_unpushed_tag());
}

/// Pushing one tag puts it on the remote and takes its marker off,
/// immediately, without a fetch. A backend that pushes and leaves the
/// mirror alone is correct as far as git is concerned, but leaves an
/// "unpushed" badge on the tag just pushed.
pub fn check_tag_push_sends_one_tag_and_clears_its_marker(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new().with_initial().with_origin().build();
    let repo_ref = repo.repo_ref();
    let head = head_oid(backend, &repo_ref);
    backend.tag_create(&repo_ref, "v1", head, None).unwrap();

    let outcome = block_on(backend.tag_push(
        &repo_ref,
        "origin",
        &TagSelection::Named(vec!["v1".to_string()]),
    ))
    .expect("tag push");
    assert_eq!(outcome.pushed_refs, vec!["v1".to_string()]);

    // Asked of git, in the bare remote itself -- not of the backend that
    // just claimed to have done it.
    let on_remote = repo.git_in_remote(&["tag", "--list"]);
    assert_eq!(on_remote, "v1", "the remote must actually hold the tag");

    let tag = find_tag(backend, &repo_ref, "v1");
    assert_eq!(
        tag.tag_sync.as_ref().map(|s| s.pushed_to.as_slice()),
        Some(["origin".to_string()].as_slice()),
        "the push must be recorded locally, with no fetch in between"
    );
    assert!(!tag.is_unpushed_tag());
}

/// `AllUnpushed` sends every tag no remote has, in one operation, and
/// leaves the already-pushed ones alone. Running it again is a *success*
/// reporting nothing: "everything is already pushed" is an answer, not a
/// failure.
pub fn check_tag_push_all_unpushed_sends_every_one(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new().with_initial().with_origin().build();
    let repo_ref = repo.repo_ref();
    let head = head_oid(backend, &repo_ref);
    for name in ["v1", "v2", "v3"] {
        backend.tag_create(&repo_ref, name, head, None).unwrap();
    }

    block_on(backend.tag_push(
        &repo_ref,
        "origin",
        &TagSelection::Named(vec!["v1".to_string()]),
    ))
    .unwrap();

    let outcome = block_on(backend.tag_push(&repo_ref, "origin", &TagSelection::AllUnpushed))
        .expect("push all unpushed");
    let mut pushed = outcome.pushed_refs.clone();
    pushed.sort();
    assert_eq!(
        pushed,
        vec!["v2".to_string(), "v3".to_string()],
        "only the tags the remote lacked, and all of them"
    );

    let mut on_remote: Vec<String> = repo
        .git_in_remote(&["tag", "--list"])
        .lines()
        .map(str::to_string)
        .collect();
    on_remote.sort();
    assert_eq!(on_remote, vec!["v1", "v2", "v3"]);

    let unpushed: Vec<String> = backend
        .refs(&repo_ref)
        .unwrap()
        .into_iter()
        .filter(|r| r.is_unpushed_tag())
        .filter_map(|r| r.tag_name().map(str::to_string))
        .collect();
    assert!(unpushed.is_empty(), "still marked unpushed: {unpushed:?}");

    let again = block_on(backend.tag_push(&repo_ref, "origin", &TagSelection::AllUnpushed))
        .expect("pushing nothing is a success");
    assert!(again.pushed_refs.is_empty());
}

/// A tag the remote already had (pushed by someone else) stops being
/// "unpushed" after a fetch, with no extra gesture. The fixture pushes the
/// tag itself, so the backend under test has no local record until it
/// fetches.
pub fn check_a_fetch_learns_the_tags_the_remote_already_had(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .with_initial()
        .with_origin()
        .tag("v1")
        .push("main")
        .push("refs/tags/v1")
        .build();
    let repo_ref = repo.repo_ref();

    assert!(
        find_tag(backend, &repo_ref, "v1").is_unpushed_tag(),
        "before any fetch, nothing local knows the remote has it"
    );

    block_on(backend.fetch(&repo_ref, "origin", &FetchOptions::default())).expect("fetch");

    let tag = find_tag(backend, &repo_ref, "v1");
    assert_eq!(
        tag.tag_sync.as_ref().map(|s| s.pushed_to.as_slice()),
        Some(["origin".to_string()].as_slice()),
        "a fetch must refresh the remote-tag mirror"
    );
}

/// Deleting a tag from the remote keeps the local one, and puts its
/// "unpushed" marker back.
pub fn check_tag_delete_remote_keeps_the_local_tag(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new().with_initial().with_origin().build();
    let repo_ref = repo.repo_ref();
    let head = head_oid(backend, &repo_ref);
    backend.tag_create(&repo_ref, "v1", head, None).unwrap();
    block_on(backend.tag_push(
        &repo_ref,
        "origin",
        &TagSelection::Named(vec!["v1".to_string()]),
    ))
    .unwrap();

    block_on(backend.tag_delete_remote(&repo_ref, "origin", "v1")).expect("delete remote tag");

    assert_eq!(
        repo.git_in_remote(&["tag", "--list"]),
        "",
        "the remote must no longer have it"
    );
    let tag = find_tag(backend, &repo_ref, "v1");
    assert!(
        tag.is_unpushed_tag(),
        "the local tag survives, and is unpushed again"
    );
}

/// Deleting a branch from the remote removes it there and drops the
/// remote-tracking ref, while the local branch of the same name survives.
pub fn check_branch_delete_remote_removes_it_from_the_remote(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new().with_initial().with_origin().build();
    let repo_ref = repo.repo_ref();
    let head = head_oid(backend, &repo_ref);
    backend.branch_create(&repo_ref, "topic", head).unwrap();
    repo.git(&["push", "origin", "topic"]);
    assert!(
        repo.git_in_remote(&["branch", "--list", "topic"])
            .contains("topic"),
        "fixture: the remote must have the branch"
    );

    block_on(backend.branch_delete_remote(&repo_ref, "origin", "topic"))
        .expect("delete remote branch");

    assert_eq!(
        repo.git_in_remote(&["branch", "--list", "topic"]).trim(),
        "",
        "the remote must no longer have it"
    );
    let refs = backend.refs(&repo_ref).unwrap();
    assert!(
        refs.iter()
            .any(|r| matches!(&r.kind, RefKind::LocalBranch { name } if name == "topic")),
        "the local branch survives"
    );
    assert!(
        !refs.iter().any(|r| matches!(
            &r.kind,
            RefKind::RemoteBranch { name, .. } if name == "topic"
        )),
        "the remote-tracking ref is gone"
    );
}

/// The mirror image: deleting a tag *locally* must not reach the remote.
/// The two deletions are separate menu entries, and conflating them would
/// take a published tag away from everyone who had fetched it.
pub fn check_tag_delete_is_local_only(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new().with_initial().with_origin().build();
    let repo_ref = repo.repo_ref();
    let head = head_oid(backend, &repo_ref);
    backend.tag_create(&repo_ref, "v1", head, None).unwrap();
    block_on(backend.tag_push(
        &repo_ref,
        "origin",
        &TagSelection::Named(vec!["v1".to_string()]),
    ))
    .unwrap();

    backend.tag_delete(&repo_ref, "v1").unwrap();

    assert!(
        backend
            .refs(&repo_ref)
            .unwrap()
            .iter()
            .filter(|r| r.tag_name() == Some("v1"))
            .all(|r| r.is_remote_only_tag()),
        "the local tag is gone: what is left is the remote's copy, listed as remote only"
    );
    assert!(
        repo.try_git(&["rev-parse", "--verify", "-q", "refs/tags/v1"])
            .is_none(),
        "no local ref"
    );
    assert_eq!(
        repo.git_in_remote(&["tag", "--list"]),
        "v1",
        "a local delete must leave the remote's tag alone"
    );
}

/// `tag_create` with no message makes a *lightweight* tag, and
/// `tag_annotate` turns it into an annotated one **without moving it**.
///
/// The object type is asked of git, because `refs()` peels annotated tags
/// and both kinds look identical through the trait.
pub fn check_tag_annotate_rewrites_a_tag_in_place(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .with_initial()
        .commit("second")
        .build();
    let repo_ref = repo.repo_ref();
    // Deliberately *not* HEAD: a backend that annotated by re-creating
    // the tag at a commit it resolved itself would silently move it.
    let older = backend
        .log(&repo_ref, &LogOptions::default())
        .unwrap()
        .nodes[1]
        .id;
    backend.tag_create(&repo_ref, "v1", older, None).unwrap();
    assert_eq!(
        repo.git(&["cat-file", "-t", "refs/tags/v1"]),
        "commit",
        "no message means a lightweight tag"
    );

    backend
        .tag_annotate(&repo_ref, "v1", "the first one")
        .expect("annotate");

    assert_eq!(
        repo.git(&["cat-file", "-t", "refs/tags/v1"]),
        "tag",
        "annotating must produce a real tag object"
    );
    assert!(repo
        .git(&["tag", "-l", "--format=%(contents)", "v1"])
        .contains("the first one"));
    assert_eq!(
        find_tag(backend, &repo_ref, "v1").target,
        older,
        "annotating must not move the tag"
    );
}

/// Annotating a tag that does not exist is an error, not a quiet
/// creation. "Annotate v9" inventing a tag at HEAD is worse than
/// refusing.
pub fn check_tag_annotate_refuses_an_unknown_tag(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new().with_initial().build();
    let repo_ref = repo.repo_ref();

    assert!(backend.tag_annotate(&repo_ref, "nope", "x").is_err());
    assert!(!backend
        .refs(&repo_ref)
        .unwrap()
        .iter()
        .any(|r| r.tag_name() == Some("nope")));
}

/// Pushing a tag that does not exist is an error, refused **here**, before
/// the remote is contacted.
///
/// The message assertion is the point: `git push origin refs/tags/nope`
/// errors on its own (`src refspec ... does not match any`), so an
/// `is_err()` check would pass with no existence check at all. Demanding
/// *our* sentence pins that the refusal is local.
pub fn check_tag_push_refuses_an_unknown_tag(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new().with_initial().with_origin().build();
    let repo_ref = repo.repo_ref();

    let error = block_on(backend.tag_push(
        &repo_ref,
        "origin",
        &TagSelection::Named(vec!["nope".to_string()]),
    ))
    .expect_err("pushing a tag that does not exist must fail");
    let message = error.to_string();
    assert!(
        message.contains("no tag named") && message.contains("nope"),
        "the refusal must name the tag, in our words, not git's: {message}"
    );
}

/// The mirror namespace is bookkeeping and must not show up in the UI, as a
/// ref or as a line in the graph.
///
/// `LogOptions::all_refs` means branches, tags and remotes, not "every ref
/// under `refs/`". The check pushes a tag (creating mirror refs) and
/// demands the ref list and the all-refs walk come out without them.
pub fn check_the_remote_tag_mirror_never_surfaces(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .commit("one")
        .commit("two")
        .with_origin()
        .build();
    let repo_ref = repo.repo_ref();
    let log = backend.log(&repo_ref, &LogOptions::default()).unwrap();
    let (second, first) = (log.nodes[0].id, log.nodes[1].id);

    backend.tag_create(&repo_ref, "v1", second, None).unwrap();
    block_on(backend.tag_push(
        &repo_ref,
        "origin",
        &TagSelection::Named(vec!["v1".to_string()]),
    ))
    .unwrap();

    // The state that makes this check discriminate: the remote still has
    // the tag, the local tag is gone and the branch has been rewound past
    // the commit it named. `second` is then reachable from *nothing* except
    // the mirror ref, so a walk that includes the mirror shows a commit on
    // no branch and no tag, and one that does not shows exactly one commit.
    // Without the rewind the check is vacuous: the mirror points at a commit
    // HEAD already reaches, so `--all` passes as happily as `--branches
    // --tags --remotes`.
    backend.tag_delete(&repo_ref, "v1").unwrap();
    backend.reset(&repo_ref, first, ResetMode::Hard).unwrap();

    assert!(
        repo.git(&[
            "for-each-ref",
            "--format=%(refname)",
            REMOTE_TAG_MIRROR_PREFIX
        ])
        .contains("origin/v1"),
        "the mirror ref must still be there, or this check proves nothing"
    );

    let refs = backend.refs(&repo_ref).unwrap();
    // The tag is listed once, as the remote's copy (`local == false`):
    // the mirror's own refname never becomes a row of its own.
    let tags: Vec<_> = refs.iter().filter(|r| r.tag_name().is_some()).collect();
    assert_eq!(tags.len(), 1, "one row for the remote-only tag: {refs:?}");
    assert!(tags[0].is_remote_only_tag(), "{refs:?}");
    assert!(
        refs.iter()
            .all(|r| !format!("{:?}", r.kind).contains("gitpoulpi/remote-tags")),
        "the mirror must not appear as a ref: {refs:?}"
    );

    let all_refs = LogOptions {
        all_refs: true,
        ..LogOptions::default()
    };
    let walked: Vec<String> = backend
        .log(&repo_ref, &all_refs)
        .unwrap()
        .nodes
        .into_iter()
        .map(|n| n.summary)
        .collect();
    assert_eq!(
        walked,
        vec!["one".to_string()],
        "the mirror must not add a line to the all-refs walk"
    );
    assert!(second != first);
}

/// A remote named with a slash (`git remote add grp/sub <url>` is legal)
/// must not corrupt the tag-sync mirror.
///
/// The mirror ref is `refs/gitpoulpi/remote-tags/<remote>/<tag>`; splitting
/// on the *first* `/` reads remote `grp/sub` + tag `v1` as remote `grp` +
/// tag `sub/v1`. This pins the write side (`tag_push`'s mirror update) and
/// the read side (a `fetch`'s `refresh_mirror`), with a tag name that itself
/// contains a slash (`release/1.0`) so the two ambiguous slashes cannot
/// cancel out.
pub fn check_tag_sync_survives_a_remote_whose_name_contains_a_slash(backend: &dyn GitBackend) {
    let source = TestRepoBuilder::new().commit("on the remote").build();
    let repo = TestRepoBuilder::new().with_initial().build();
    let repo_ref = repo.repo_ref();
    let head = head_oid(backend, &repo_ref);

    backend
        .remote_add(&repo_ref, "grp/sub", &source.root().to_string_lossy())
        .unwrap();
    backend.tag_create(&repo_ref, "v1", head, None).unwrap();
    backend
        .tag_create(&repo_ref, "release/1.0", head, None)
        .unwrap();

    let outcome = block_on(backend.tag_push(
        &repo_ref,
        "grp/sub",
        &TagSelection::Named(vec!["v1".to_string(), "release/1.0".to_string()]),
    ))
    .expect("tag push");
    assert_eq!(
        {
            let mut pushed = outcome.pushed_refs;
            pushed.sort();
            pushed
        },
        vec!["release/1.0".to_string(), "v1".to_string()]
    );

    // Asked of git, in the remote itself -- not of the backend that just
    // claimed to have done it.
    let mut on_remote: Vec<String> = source
        .git(&["tag", "--list"])
        .lines()
        .map(str::to_string)
        .collect();
    on_remote.sort();
    assert_eq!(
        on_remote,
        vec!["release/1.0".to_string(), "v1".to_string()],
        "the remote must actually hold both tags"
    );

    for name in ["v1", "release/1.0"] {
        let tag = find_tag(backend, &repo_ref, name);
        assert_eq!(
            tag.tag_sync.as_ref().map(|s| s.pushed_to.as_slice()),
            Some(["grp/sub".to_string()].as_slice()),
            "`{name}` must report as pushed to the exact remote `grp/sub`, \
             not truncated at the first `/` - got {tag:?}"
        );
        assert!(!tag.is_unpushed_tag());
    }

    // The read side too: a fetch re-derives the same mirror from
    // `refresh_mirror`'s own refspec, independently of `tag_push`'s
    // `record_pushed`. Both paths must parse the mirror ref identically.
    block_on(backend.fetch(&repo_ref, "grp/sub", &FetchOptions::default())).expect("fetch");
    for name in ["v1", "release/1.0"] {
        let tag = find_tag(backend, &repo_ref, name);
        assert_eq!(
            tag.tag_sync.as_ref().map(|s| s.pushed_to.as_slice()),
            Some(["grp/sub".to_string()].as_slice()),
            "after a fetch, `{name}` must still report as pushed to `grp/sub` - got {tag:?}"
        );
    }
}

/// The one tag of this name, or a panic naming what was actually there -
/// an `unwrap()` on a `find` says nothing about why it failed.
fn find_tag(
    backend: &dyn GitBackend,
    repo: &gitpoulpi_core::domain::RepoRef,
    name: &str,
) -> gitpoulpi_core::domain::Ref {
    let refs = backend.refs(repo).unwrap();
    refs.iter()
        .find(|r| r.tag_name() == Some(name))
        .unwrap_or_else(|| panic!("no tag `{name}` among {refs:?}"))
        .clone()
}

// `tag_push`, `tag_delete_remote` and `fetch` are async for the same
// reason `clone` is, and are driven from these synchronous checks by the
// same `block_on` above.

// -- remote-tracking branches ---------------------------------------------
//
// Three facts the Refs sidebar's REMOTE section is built on: what a local
// branch tracks, how `refs/remotes/<...>` splits into a remote and a
// branch, and what "check this remote branch out" does.

/// A cloned repository's checked-out branch tracks the remote it came
/// from, and every other ref reports no upstream at all.
///
/// The spelling matters: `upstream` must be the same string
/// `RefKind::RemoteBranch` produces for that ref (`origin/main`), because
/// the only consumer joins the two by equality.
pub fn check_refs_report_a_local_branchs_upstream(backend: &dyn GitBackend) {
    let source = TestRepoBuilder::new().with_initial().build();
    let dest_dir = tempfile::tempdir().unwrap();
    let dest = dest_dir.path().join("clone");
    let cloned = block_on(backend.clone(
        &source.root().to_string_lossy(),
        &dest,
        &CloneOptions::default(),
    ))
    .unwrap();

    let refs = backend.refs(&cloned).unwrap();
    let head = refs
        .iter()
        .find(|r| r.is_head)
        .expect("a fresh clone has a checked-out branch");
    let RefKind::LocalBranch { name } = &head.kind else {
        panic!("HEAD must be a local branch");
    };
    assert_eq!(
        head.upstream.as_deref(),
        Some(format!("origin/{name}").as_str()),
        "a cloned branch tracks origin/<its own name>"
    );
    assert!(
        refs.iter()
            .filter(|r| !matches!(r.kind, RefKind::LocalBranch { .. }))
            .all(|r| r.upstream.is_none()),
        "only local branches can carry an upstream"
    );

    // A local branch created off HEAD has no upstream of its own, which
    // is what makes "no local branch corresponds to this remote one" a
    // real state rather than something the backend invents.
    let at = head.target;
    backend.branch_create(&cloned, "solo", at).unwrap();
    let solo = backend
        .refs(&cloned)
        .unwrap()
        .into_iter()
        .find(|r| matches!(&r.kind, RefKind::LocalBranch { name } if name == "solo"))
        .unwrap();
    assert_eq!(solo.upstream, None);
}

/// `refs/remotes/<rest>` must be split against the repository's actual
/// remote list, not on the first `/`: a remote named `grp/sub` is legal and
/// its branch `main` lives at `refs/remotes/grp/sub/main`. Splitting on the
/// first separator reports remote `grp`, branch `sub/main`, and every
/// "which local branch tracks this" join misses.
pub fn check_refs_split_a_remote_whose_name_contains_a_slash(backend: &dyn GitBackend) {
    let source = TestRepoBuilder::new().with_initial().build();
    let repo = TestRepoBuilder::new().commit("local").build();
    let repo_ref = repo.repo_ref();

    backend
        .remote_add(&repo_ref, "grp/sub", &source.root().to_string_lossy())
        .unwrap();
    block_on(backend.fetch(&repo_ref, "grp/sub", &FetchOptions::default())).unwrap();

    let remote_branches: Vec<(String, String)> = backend
        .refs(&repo_ref)
        .unwrap()
        .into_iter()
        .filter_map(|r| match r.kind {
            RefKind::RemoteBranch { remote, name } => Some((remote, name)),
            _ => None,
        })
        .collect();
    assert!(
        !remote_branches.is_empty(),
        "the fetch produced no remote-tracking branch at all"
    );
    assert!(
        remote_branches
            .iter()
            .all(|(remote, _)| remote == "grp/sub"),
        "every fetched ref belongs to the remote that is actually configured: {remote_branches:?}"
    );
    assert!(
        remote_branches.iter().all(|(_, name)| !name.contains('/')),
        "the remote's own name must not be left inside the branch name: {remote_branches:?}"
    );
}

/// `branch_create_tracking` creates the branch, sets its upstream and
/// leaves it checked out - all three, because the caller's gesture was
/// "check this remote branch out" and any one of them missing looks like
/// success while being wrong.
pub fn check_branch_create_tracking_creates_sets_upstream_and_checks_out(backend: &dyn GitBackend) {
    let source = TestRepoBuilder::new()
        .with_initial()
        .branch("feature/login")
        .commit("on the feature")
        // Back on `main` before the clone: `FixtureRepoBuilder::branch`
        // is `checkout -b`, so leaving it checked out would make the
        // clone's own HEAD `feature/login` and the local branch this
        // check creates would already exist.
        .checkout("main")
        .build();
    let dest_dir = tempfile::tempdir().unwrap();
    let dest = dest_dir.path().join("clone");
    let cloned = block_on(backend.clone(
        &source.root().to_string_lossy(),
        &dest,
        &CloneOptions::default(),
    ))
    .unwrap();

    backend
        .branch_create_tracking(&cloned, "feature/login", "origin/feature/login")
        .unwrap();

    let refs = backend.refs(&cloned).unwrap();
    let created = refs
        .iter()
        .find(|r| matches!(&r.kind, RefKind::LocalBranch { name } if name == "feature/login"))
        .expect("the branch must exist afterwards");
    assert!(created.is_head, "the new branch must be checked out");
    assert_eq!(
        created.upstream.as_deref(),
        Some("origin/feature/login"),
        "the new branch must track the remote branch it was made from"
    );
    let remote_tip = refs
        .iter()
        .find(|r| {
            matches!(&r.kind, RefKind::RemoteBranch { remote, name }
                if remote == "origin" && name == "feature/login")
        })
        .unwrap()
        .target;
    assert_eq!(
        created.target, remote_tip,
        "the new branch must start at the remote branch's tip"
    );
}

/// An already-taken local name is an error, not a silent checkout. Both
/// `git branch` and `git2` refuse already, but the contract must say so:
/// the UI's third case (name taken by a branch tracking something else)
/// is built on it.
pub fn check_branch_create_tracking_refuses_an_existing_local_name(backend: &dyn GitBackend) {
    let source = TestRepoBuilder::new()
        .with_initial()
        .branch("side")
        // See the check above: `branch` checks out, and the clone would
        // otherwise start on `side` already.
        .checkout("main")
        .build();
    let dest_dir = tempfile::tempdir().unwrap();
    let dest = dest_dir.path().join("clone");
    let cloned = block_on(backend.clone(
        &source.root().to_string_lossy(),
        &dest,
        &CloneOptions::default(),
    ))
    .unwrap();

    let head = backend
        .refs(&cloned)
        .unwrap()
        .into_iter()
        .find(|r| r.is_head)
        .unwrap();
    backend.branch_create(&cloned, "side", head.target).unwrap();

    assert!(
        backend
            .branch_create_tracking(&cloned, "side", "origin/side")
            .is_err(),
        "creating a branch over an existing name must fail"
    );
    let side = backend
        .refs(&cloned)
        .unwrap()
        .into_iter()
        .find(|r| matches!(&r.kind, RefKind::LocalBranch { name } if name == "side"))
        .unwrap();
    assert_eq!(
        side.upstream, None,
        "a refused create must not have touched the existing branch's upstream"
    );
}

// -- remotes: configuration, upstreams, push / pull / fetch all ----------
//
// Every push goes to a bare repository inside the fixture's temp dir
// (`with_origin` / `with_remote`), and "someone else pushed" is
// `FixtureRepo::push_from_elsewhere`. No network, no credentials.

/// The `(name, fetch_url, push_url)` of one remote, as the backend lists it.
fn remote_named(
    backend: &dyn GitBackend,
    repo: &gitpoulpi_core::domain::RepoRef,
    name: &str,
) -> Option<gitpoulpi_core::domain::Remote> {
    backend
        .remotes(repo)
        .unwrap()
        .into_iter()
        .find(|r| r.name == name)
}

fn local_branch(
    backend: &dyn GitBackend,
    repo: &gitpoulpi_core::domain::RepoRef,
    name: &str,
) -> gitpoulpi_core::domain::Ref {
    let refs = backend.refs(repo).unwrap();
    refs.iter()
        .find(|r| matches!(&r.kind, RefKind::LocalBranch { name: n } if n == name))
        .unwrap_or_else(|| panic!("no local branch `{name}` among {refs:?}"))
        .clone()
}

/// `core.sshCommand` is the opt-in, per-repository way to pin the SSH key
/// a profile generated. Setting it is visible to the VCS itself, is local
/// to this repository, and clearing it (even twice) restores "unset".
pub fn check_ssh_command_is_set_and_cleared_per_repo(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new().with_initial().build();
    let repo_ref = repo.repo_ref();
    assert_eq!(backend.ssh_command(&repo_ref).unwrap(), None);

    let cmd = "ssh -i /home/me/.ssh/gitpoulpi_x_ed25519 -o IdentitiesOnly=yes";
    backend.set_ssh_command(&repo_ref, Some(cmd)).unwrap();
    assert_eq!(
        backend.ssh_command(&repo_ref).unwrap().as_deref(),
        Some(cmd)
    );
    let asked = std::process::Command::new("git")
        .args(["config", "--local", "--get", "core.sshCommand"])
        .current_dir(&repo_ref.root)
        .output()
        .unwrap();
    assert_eq!(String::from_utf8_lossy(&asked.stdout).trim(), cmd);

    backend.set_ssh_command(&repo_ref, None).unwrap();
    assert_eq!(backend.ssh_command(&repo_ref).unwrap(), None);
    backend
        .set_ssh_command(&repo_ref, None)
        .expect("clearing twice must succeed");
}

fn git_local(root: &std::path::Path, key: &str) -> String {
    let out = std::process::Command::new("git")
        .args(["config", "--local", "--get", key])
        .current_dir(root)
        .output()
        .unwrap();
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

/// `core.autocrlf` / `core.eol` are written to (and read back from) the
/// repository's own git config; `None` removes them; nothing else is
/// reachable and a value the key does not take is refused.
pub fn check_repo_config_sets_unsets_and_reads_back(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new().with_initial().build();
    let repo_ref = repo.repo_ref();
    // The fixture pins `core.autocrlf=false`: read it back, then clear it.
    assert_eq!(
        backend
            .repo_config(&repo_ref, "core.autocrlf")
            .unwrap()
            .as_deref(),
        Some("false")
    );
    for key in ["core.autocrlf", "core.eol"] {
        backend.set_repo_config(&repo_ref, key, None).unwrap();
        assert_eq!(backend.repo_config(&repo_ref, key).unwrap(), None, "{key}");
    }
    for (key, value) in [("core.autocrlf", "input"), ("core.eol", "crlf")] {
        backend
            .set_repo_config(&repo_ref, key, Some(value))
            .unwrap();
        assert_eq!(
            backend.repo_config(&repo_ref, key).unwrap().as_deref(),
            Some(value)
        );
        assert_eq!(git_local(repo.root(), key), value, "git itself sees {key}");
    }
    // Replaced, not duplicated.
    backend
        .set_repo_config(&repo_ref, "core.autocrlf", Some("true"))
        .unwrap();
    assert_eq!(git_local(repo.root(), "core.autocrlf"), "true");
    backend
        .set_repo_config(&repo_ref, "core.autocrlf", None)
        .unwrap();
    assert_eq!(
        backend.repo_config(&repo_ref, "core.autocrlf").unwrap(),
        None
    );
    assert_eq!(git_local(repo.root(), "core.autocrlf"), "");
    // The other key is untouched; clearing twice is fine.
    assert_eq!(
        backend
            .repo_config(&repo_ref, "core.eol")
            .unwrap()
            .as_deref(),
        Some("crlf")
    );
    backend
        .set_repo_config(&repo_ref, "core.autocrlf", None)
        .expect("clearing twice must succeed");
    // Allow-list and value validation.
    assert!(backend.repo_config(&repo_ref, "user.signingkey").is_err());
    assert!(backend
        .set_repo_config(&repo_ref, "core.sshCommand", Some("x"))
        .is_err());
    assert!(backend
        .set_repo_config(&repo_ref, "core.eol", Some("true"))
        .is_err());
    assert_eq!(
        backend
            .repo_config(&repo_ref, "core.eol")
            .unwrap()
            .as_deref(),
        Some("crlf"),
        "a refused value changes nothing"
    );
}

/// A repository's chosen profile is written as its local `user.name` /
/// `user.email` through the same call: free text, replaced not duplicated,
/// blank refused, and the commit author follows.
pub fn check_repo_config_writes_the_commit_identity(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new().with_initial().build();
    let repo_ref = repo.repo_ref();
    backend
        .set_repo_config(&repo_ref, "user.name", Some("Ada Lovelace"))
        .unwrap();
    backend
        .set_repo_config(&repo_ref, "user.email", Some("ada@example.org"))
        .unwrap();
    backend
        .set_repo_config(&repo_ref, "user.name", Some("Ada L."))
        .unwrap();
    assert_eq!(git_local(repo.root(), "user.name"), "Ada L.");
    assert_eq!(git_local(repo.root(), "user.email"), "ada@example.org");
    assert_eq!(
        backend
            .repo_config(&repo_ref, "user.email")
            .unwrap()
            .as_deref(),
        Some("ada@example.org")
    );
    assert!(backend
        .set_repo_config(&repo_ref, "user.name", Some(" "))
        .is_err());
    assert_eq!(git_local(repo.root(), "user.name"), "Ada L.");
    backend
        .set_repo_config(&repo_ref, "user.name", None)
        .unwrap();
    assert_eq!(git_local(repo.root(), "user.name"), "");
}

/// The undo journal restores these keys as one-variable config sections:
/// reading, replacing and emptying `core.autocrlf` that way works, and
/// leaves `core.filemode` and the other key alone.
pub fn check_repo_config_key_sections_roundtrip_for_undo(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new().with_initial().build();
    let repo_ref = repo.repo_ref();
    backend
        .set_repo_config(&repo_ref, "core.eol", Some("lf"))
        .unwrap();
    backend
        .set_repo_config(&repo_ref, "core.autocrlf", None)
        .unwrap();
    assert!(backend
        .config_section(&repo_ref, "core.autocrlf")
        .unwrap()
        .is_empty());
    backend
        .set_config_section(
            &repo_ref,
            "core.autocrlf",
            &[("core.autocrlf".into(), "input".into())],
        )
        .unwrap();
    assert_eq!(
        backend.config_section(&repo_ref, "core.autocrlf").unwrap(),
        vec![("core.autocrlf".to_string(), "input".to_string())]
    );
    assert_eq!(git_local(repo.root(), "core.autocrlf"), "input");
    backend
        .set_config_section(&repo_ref, "core.autocrlf", &[])
        .unwrap();
    assert_eq!(git_local(repo.root(), "core.autocrlf"), "");
    assert_eq!(git_local(repo.root(), "core.eol"), "lf");
    assert!(backend.config_section(&repo_ref, "core").is_err());
}

/// A fetch URL and a push URL are two settings, and clearing the push
/// URL makes pushes go to the fetch URL again - which is what
/// `Remote::push_url` must then report, since it is "the URL a push
/// uses", not "the raw `pushurl` key".
pub fn check_remote_set_url_and_push_url(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new().with_initial().build();
    let repo_ref = repo.repo_ref();
    backend
        .remote_add(&repo_ref, "origin", "https://example.invalid/a.git")
        .unwrap();

    backend
        .remote_set_url(&repo_ref, "origin", "git@example.invalid:team/b.git")
        .unwrap();
    let r = remote_named(backend, &repo_ref, "origin").unwrap();
    assert_eq!(r.fetch_url, "git@example.invalid:team/b.git");
    assert_eq!(
        r.push_url.as_deref(),
        Some("git@example.invalid:team/b.git"),
        "with no push URL of its own, a remote pushes where it fetches"
    );

    backend
        .remote_set_push_url(
            &repo_ref,
            "origin",
            Some("ssh://git@example.invalid/push.git"),
        )
        .unwrap();
    let r = remote_named(backend, &repo_ref, "origin").unwrap();
    assert_eq!(r.fetch_url, "git@example.invalid:team/b.git");
    assert_eq!(
        r.push_url.as_deref(),
        Some("ssh://git@example.invalid/push.git")
    );
    // Asked of git itself: the key really is `pushurl`, not a second `url`.
    assert_eq!(
        repo.git(&["config", "--get", "remote.origin.pushurl"]),
        "ssh://git@example.invalid/push.git"
    );

    backend
        .remote_set_push_url(&repo_ref, "origin", None)
        .unwrap();
    let r = remote_named(backend, &repo_ref, "origin").unwrap();
    assert_eq!(
        r.push_url.as_deref(),
        Some("git@example.invalid:team/b.git")
    );
    // ...and clearing an already-clear push URL is not an error.
    backend
        .remote_set_push_url(&repo_ref, "origin", None)
        .expect("clearing twice must succeed");

    assert!(
        backend
            .remote_set_url(&repo_ref, "nope", "https://example.invalid/x.git")
            .is_err(),
        "an unknown remote is an error, not a new remote"
    );
}

/// Renaming a remote carries everything that names it: the
/// remote-tracking branches, a local branch's upstream, and the tag
/// mirror - the last one being ours, and the one `git remote rename`
/// alone would leave behind.
pub fn check_remote_rename_carries_branches_upstreams_and_tag_mirror(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .with_initial()
        .with_origin()
        .tag("v1")
        .push("main")
        .push("refs/tags/v1")
        .track("main", "origin")
        .build();
    let repo_ref = repo.repo_ref();
    // Learn the remote's tags, so there is a mirror to carry.
    block_on(backend.fetch(&repo_ref, "origin", &FetchOptions::default())).unwrap();
    assert!(!find_tag(backend, &repo_ref, "v1").is_unpushed_tag());

    backend
        .remote_rename(&repo_ref, "origin", "upstream")
        .expect("rename");

    let names: Vec<String> = backend
        .remotes(&repo_ref)
        .unwrap()
        .into_iter()
        .map(|r| r.name)
        .collect();
    assert_eq!(names, vec!["upstream".to_string()]);
    assert_eq!(
        local_branch(backend, &repo_ref, "main").upstream.as_deref(),
        Some("upstream/main"),
        "a branch that tracked origin/main must track upstream/main"
    );
    assert!(backend.refs(&repo_ref).unwrap().iter().any(|r| matches!(
        &r.kind,
        RefKind::RemoteBranch { remote, name } if remote == "upstream" && name == "main"
    )));
    let tag = find_tag(backend, &repo_ref, "v1");
    assert_eq!(
        tag.tag_sync.as_ref().map(|s| s.pushed_to.clone()),
        Some(vec!["upstream".to_string()]),
        "the tag mirror must follow the rename, or every tag reads unpushed"
    );

    assert!(backend
        .remote_rename(&repo_ref, "missing", "other")
        .is_err());
}

/// Removing a remote forgets everything it named, *including our tag
/// mirror*: a tag pushed only to a removed remote is unpushed again, and
/// no `refs/gitpoulpi/remote-tags/<removed>/...` ref is left for a later
/// remote of the same name to inherit.
pub fn check_remote_remove_forgets_its_tag_mirror(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .with_initial()
        .with_origin()
        .tag("v1")
        .push("main")
        .push("refs/tags/v1")
        .build();
    let repo_ref = repo.repo_ref();
    block_on(backend.fetch(&repo_ref, "origin", &FetchOptions::default())).unwrap();
    assert!(!find_tag(backend, &repo_ref, "v1").is_unpushed_tag());

    backend.remote_remove(&repo_ref, "origin").expect("remove");

    assert!(backend.remotes(&repo_ref).unwrap().is_empty());
    assert!(
        find_tag(backend, &repo_ref, "v1").is_unpushed_tag(),
        "a tag only the removed remote had is unpushed again"
    );
    assert_eq!(
        repo.git(&["for-each-ref", REMOTE_TAG_MIRROR_PREFIX]),
        "",
        "no mirror ref may outlive its remote"
    );
    assert!(backend.remote_remove(&repo_ref, "origin").is_err());
}

/// Adding a remote under a name that is taken is an error, and leaves
/// the existing remote's URL alone.
pub fn check_remote_add_refuses_a_taken_name(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new().with_initial().build();
    let repo_ref = repo.repo_ref();
    backend
        .remote_add(&repo_ref, "origin", "https://example.invalid/a.git")
        .unwrap();
    assert!(backend
        .remote_add(&repo_ref, "origin", "https://example.invalid/b.git")
        .is_err());
    assert_eq!(
        remote_named(backend, &repo_ref, "origin")
            .unwrap()
            .fetch_url,
        "https://example.invalid/a.git"
    );
}

/// A URL or remote name that would be read as an option, or that names a
/// remote helper (`ext::`), is refused by every network entry point, and
/// nothing is written or run.
pub fn check_option_lookalike_and_helper_urls_are_refused(backend: &dyn GitBackend) {
    let source = TestRepoBuilder::new().with_initial().build();
    let marker = source.root().join("pwned");
    let upload_pack = format!("--upload-pack=touch {}", marker.display());
    let ext = format!("ext::sh -c touch% {}", marker.display());
    let dest_dir = tempfile::tempdir().unwrap();

    for (i, bad) in [upload_pack.as_str(), "-x", ext.as_str()]
        .into_iter()
        .enumerate()
    {
        let dest = dest_dir.path().join(format!("clone{i}"));
        assert!(
            block_on(backend.clone(bad, &dest, &CloneOptions::default())).is_err(),
            "clone of {bad:?}"
        );
        assert!(!dest.exists(), "a refused clone leaves nothing behind");

        let repo = TestRepoBuilder::new().with_initial().build();
        let repo_ref = repo.repo_ref();
        assert!(
            backend.remote_add(&repo_ref, "origin", bad).is_err(),
            "{bad:?}"
        );
        assert!(backend
            .remote_add(&repo_ref, bad, "https://example.invalid/a.git")
            .is_err());
        backend
            .remote_add(&repo_ref, "origin", "https://example.invalid/a.git")
            .unwrap();
        assert!(backend.remote_set_url(&repo_ref, "origin", bad).is_err());
        assert!(backend
            .remote_set_push_url(&repo_ref, "origin", Some(bad))
            .is_err());
        assert!(block_on(backend.fetch(&repo_ref, bad, &FetchOptions::default())).is_err());
        assert!(block_on(backend.push(&repo_ref, bad, &PushOptions::default())).is_err());
        assert_eq!(
            remote_named(backend, &repo_ref, "origin")
                .unwrap()
                .fetch_url,
            "https://example.invalid/a.git",
            "a refused change writes nothing"
        );
    }
    assert!(!marker.exists(), "no command was run");
}

/// Setting an upstream points the branch at an existing remote branch;
/// `None` removes it; a remote branch nobody fetched is refused.
pub fn check_branch_set_upstream_sets_and_unsets(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .with_initial()
        .with_origin()
        .push("main")
        .branch("feature")
        .push("feature")
        .build();
    let repo_ref = repo.repo_ref();
    assert_eq!(local_branch(backend, &repo_ref, "feature").upstream, None);

    backend
        .branch_set_upstream(&repo_ref, "feature", Some("origin/main"))
        .expect("set upstream");
    assert_eq!(
        local_branch(backend, &repo_ref, "feature")
            .upstream
            .as_deref(),
        Some("origin/main"),
        "the upstream need not share the branch's name"
    );

    backend
        .branch_set_upstream(&repo_ref, "feature", None)
        .expect("unset upstream");
    assert_eq!(local_branch(backend, &repo_ref, "feature").upstream, None);
    backend
        .branch_set_upstream(&repo_ref, "feature", None)
        .expect("unsetting twice is not an error");

    assert!(
        backend
            .branch_set_upstream(&repo_ref, "feature", Some("origin/never-fetched"))
            .is_err(),
        "an upstream that does not exist must be refused"
    );
    assert_eq!(local_branch(backend, &repo_ref, "feature").upstream, None);
}

/// Fetch all reaches every remote, and `prune` drops a remote-tracking
/// branch whose branch was deleted on the remote.
pub fn check_fetch_all_reaches_every_remote_and_prunes(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .with_initial()
        .with_origin()
        .with_remote("mirror")
        .push("main")
        .push("main:gone")
        .push_to("mirror", "main:side")
        .build();
    let repo_ref = repo.repo_ref();
    // The fixture's pushes left `refs/remotes/origin/gone` behind; delete
    // the branch on the remote so only a pruning fetch removes it.
    repo.git_in_remote(&["update-ref", "-d", "refs/heads/gone"]);

    let has = |remote: &str, name: &str| {
        backend.refs(&repo_ref).unwrap().iter().any(|r| {
            matches!(&r.kind, RefKind::RemoteBranch { remote: rm, name: n }
                if rm == remote && n == name)
        })
    };
    assert!(has("origin", "gone"));

    block_on(backend.fetch_all(
        &repo_ref,
        &FetchOptions {
            prune: true,
            ..Default::default()
        },
    ))
    .expect("fetch all");

    assert!(
        has("mirror", "side"),
        "the second remote must be fetched too"
    );
    assert!(has("origin", "main"));
    assert!(
        !has("origin", "gone"),
        "prune must drop a branch the remote deleted"
    );
}

/// A pull request's head lives in a ref a normal fetch never brings
/// (`refs/pull/<n>/head`, possibly a fork's commits). Fetching it as a
/// branch creates a local branch tracking it, and a second fetch after the
/// PR moved fast-forwards that branch.
pub fn check_fetch_ref_as_branch_tracks_a_pull_ref(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .with_initial()
        .with_origin()
        .push("main")
        .branch("work")
        .checkout("work")
        .commit_file("pr.txt", "one", "pr commit")
        .push("work")
        .checkout("main")
        .build();
    let repo_ref = repo.repo_ref();
    let tip = repo.git(&["rev-parse", "work"]);
    repo.git_in_remote(&["update-ref", "refs/pull/7/head", &tip]);

    block_on(backend.fetch_ref_as_branch(&repo_ref, "origin", "refs/pull/7/head", "pr/7"))
        .expect("fetch the pull ref");
    assert_eq!(repo.git(&["rev-parse", "refs/heads/pr/7"]), tip);
    assert_eq!(
        repo.git(&["config", "branch.pr/7.remote"]),
        "origin",
        "the branch must track the fetched ref"
    );
    assert!(local_branch(backend, &repo_ref, "pr/7").upstream.is_some());

    // The PR moves; a second fetch fast-forwards the local branch.
    repo.git(&["checkout", "-q", "work"]);
    repo.git(&["commit", "-q", "--allow-empty", "-m", "more"]);
    let moved = repo.git(&["rev-parse", "work"]);
    repo.git(&["push", "-q", "origin", "work"]);
    repo.git(&["checkout", "-q", "main"]);
    repo.git_in_remote(&["update-ref", "refs/pull/7/head", &moved]);
    block_on(backend.fetch_ref_as_branch(&repo_ref, "origin", "refs/pull/7/head", "pr/7"))
        .expect("refetch");
    assert_eq!(repo.git(&["rev-parse", "refs/heads/pr/7"]), moved);

    // A ref outside refs/ and a bad branch name are refused.
    assert!(
        block_on(backend.fetch_ref_as_branch(&repo_ref, "origin", "pull/7/head", "x")).is_err()
    );
    assert!(block_on(backend.fetch_ref_as_branch(
        &repo_ref,
        "origin",
        "refs/pull/7/head",
        "bad name"
    ))
    .is_err());
}

/// `fetch_refspec` brings a pull request's head into a namespaced ref
/// (`refs/gitpoulpi/pr/<n>`) without creating a branch, checking anything
/// out or touching the remote-tracking refs; a refetch after the request
/// moved force-updates it (the request may have been force-pushed), and a
/// destination outside `refs/gitpoulpi/` or a source outside `refs/` is
/// refused with nothing fetched.
pub fn check_fetch_refspec_lands_in_the_gitpoulpi_namespace(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .with_initial()
        .with_origin()
        .push("main")
        .branch("work")
        .checkout("work")
        .commit_file("pr.txt", "one", "pr commit")
        .push("work")
        .checkout("main")
        .build();
    let repo_ref = repo.repo_ref();
    let tip = repo.git(&["rev-parse", "work"]);
    repo.git_in_remote(&["update-ref", "refs/pull/7/head", &tip]);
    let refs_before = repo.git(&["for-each-ref", "refs/heads", "refs/remotes"]);

    let outcome = block_on(backend.fetch_refspec(
        &repo_ref,
        "origin",
        "+refs/pull/7/head:refs/gitpoulpi/pr/7",
    ))
    .expect("fetch the pull ref");
    assert_eq!(
        outcome.updated_refs,
        vec!["refs/gitpoulpi/pr/7".to_string()]
    );
    assert_eq!(repo.git(&["rev-parse", "refs/gitpoulpi/pr/7"]), tip);
    assert_eq!(
        repo.git(&["for-each-ref", "refs/heads", "refs/remotes"]),
        refs_before,
        "no branch and no remote-tracking ref may appear"
    );
    assert_eq!(repo.git(&["symbolic-ref", "--short", "HEAD"]), "main");

    // The request is force-pushed to something unrelated: the ref follows.
    repo.git(&["checkout", "-q", "--orphan", "other"]);
    repo.git(&["commit", "-q", "--allow-empty", "-m", "rewritten"]);
    let rewritten = repo.git(&["rev-parse", "other"]);
    repo.git(&["push", "-q", "origin", "other"]);
    repo.git(&["checkout", "-q", "-f", "main"]);
    repo.git_in_remote(&["update-ref", "refs/pull/7/head", &rewritten]);
    block_on(backend.fetch_refspec(&repo_ref, "origin", "+refs/pull/7/head:refs/gitpoulpi/pr/7"))
        .expect("refetch");
    assert_eq!(repo.git(&["rev-parse", "refs/gitpoulpi/pr/7"]), rewritten);

    // Refused: a destination that is a branch, a tag or anywhere else, a
    // source that is not a full ref, a spec with no destination, and one
    // that smuggles a second spec or an option.
    for bad in [
        "refs/pull/7/head:refs/heads/pr-7",
        "refs/pull/7/head:refs/tags/v1",
        "refs/pull/7/head:refs/remotes/origin/x",
        "refs/pull/7/head:refs/gitpoulpi/../heads/x",
        "pull/7/head:refs/gitpoulpi/pr/7",
        "refs/pull/7/head",
        "refs/pull/7/head:refs/gitpoulpi/a refs/pull/7/head:refs/heads/b",
        "--upload-pack=x:refs/gitpoulpi/pr/7",
    ] {
        assert!(
            block_on(backend.fetch_refspec(&repo_ref, "origin", bad)).is_err(),
            "{bad} must be refused"
        );
    }
    assert!(
        !repo.git(&["for-each-ref", "refs/heads"]).contains("pr-7"),
        "a refused spec fetched nothing"
    );
}

/// The first push of a new branch creates it on the remote *named* --
/// not necessarily `origin` - and `set_upstream` makes the local branch
/// track it; `branch` pushes a branch other than the checked-out one.
pub fn check_push_to_a_chosen_remote_sets_the_upstream(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .with_initial()
        .with_origin()
        .with_remote("fork")
        .branch("feature")
        .commit("on feature")
        .checkout("main")
        .build();
    let repo_ref = repo.repo_ref();

    let outcome = block_on(backend.push(
        &repo_ref,
        "fork",
        &PushOptions {
            set_upstream: true,
            branch: Some("feature".into()),
            ..PushOptions::default()
        },
    ))
    .expect("push");
    assert_eq!(outcome.pushed_refs, vec!["feature".to_string()]);

    let feature_tip = repo.git(&["rev-parse", "feature"]);
    assert_eq!(
        repo.git_in_named_remote("fork", &["rev-parse", "refs/heads/feature"]),
        feature_tip,
        "the named remote must hold the branch"
    );
    assert_eq!(
        local_branch(backend, &repo_ref, "feature")
            .upstream
            .as_deref(),
        Some("fork/feature")
    );
    assert!(
        repo.git_in_remote(&["for-each-ref", "refs/heads"])
            .is_empty(),
        "origin was not asked for anything"
    );
}

/// `push_for_review` pushes `HEAD` to `refs/for/<branch>` - how a Gerrit
/// change is created - optionally with a `%topic=` option, never creates a
/// branch on the remote or a remote-tracking ref, and refuses names that
/// could smuggle a second option or refspec before git runs. (A plain git
/// remote accepts the ref name, which is all the contract needs.)
pub fn check_push_for_review_pushes_head_to_the_review_ref(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .with_initial()
        .with_origin()
        .push("main")
        .branch("work")
        .checkout("work")
        .commit_file("change.txt", "one", "a change to review")
        .build();
    let repo_ref = repo.repo_ref();
    let tip = repo.git(&["rev-parse", "HEAD"]);
    let branches_before = repo.git_in_remote(&["for-each-ref", "refs/heads"]);

    let outcome = block_on(backend.push_for_review(&repo_ref, "origin", "main", None))
        .expect("push for review");
    assert_eq!(outcome.pushed_refs, vec!["refs/for/main".to_string()]);
    assert_eq!(
        repo.git_in_remote(&["rev-parse", "refs/for/main"]),
        tip,
        "HEAD (the checked-out `work`) is what the review ref holds"
    );
    assert_eq!(
        repo.git_in_remote(&["for-each-ref", "refs/heads"]),
        branches_before,
        "no branch appears on the remote"
    );
    assert_eq!(repo.git(&["symbolic-ref", "--short", "HEAD"]), "work");
    assert!(
        repo.git(&["for-each-ref", "refs/remotes/origin/work"])
            .is_empty(),
        "no remote-tracking ref for the work branch"
    );

    // A topic rides as a Gerrit option on the pushed ref.
    repo.git(&["commit", "-q", "--allow-empty", "-m", "second"]);
    let outcome = block_on(backend.push_for_review(&repo_ref, "origin", "main", Some("moons")))
        .expect("push with a topic");
    assert_eq!(
        outcome.pushed_refs,
        vec!["refs/for/main%topic=moons".to_string()]
    );
    assert_eq!(
        repo.git_in_remote(&["rev-parse", "refs/for/main%topic=moons"]),
        repo.git(&["rev-parse", "HEAD"])
    );

    // Refused before git runs.
    for (branch, topic) in [
        ("", None),
        ("-f", None),
        ("main%topic=x", None),
        ("a b", None),
        ("../main", None),
        ("main", Some("x y")),
        ("main", Some("x%y")),
        ("main", Some("--force")),
    ] {
        assert!(
            block_on(backend.push_for_review(&repo_ref, "origin", branch, topic)).is_err(),
            "{branch:?} {topic:?} must be refused"
        );
    }
    assert!(
        block_on(backend.push_for_review(&repo_ref, "nowhere", "main", None)).is_err(),
        "an unknown remote is an error"
    );
}

/// A push the remote refuses because it moved on is the typed
/// `PushRejected { NonFastForward }` - the value the TUI's "pull first /
/// force push / cancel" dialog is keyed on - and changes nothing.
pub fn check_a_diverged_push_is_a_typed_rejection(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .with_initial()
        .with_origin()
        .push("main")
        .track("main", "origin")
        .commit("mine")
        .build();
    let repo_ref = repo.repo_ref();
    repo.push_from_elsewhere("main", "theirs");
    let remote_before = repo.git_in_remote(&["rev-parse", "main"]);

    let err = block_on(backend.push(&repo_ref, "origin", &PushOptions::default()))
        .expect_err("a diverged push must fail");
    match err {
        gitpoulpi_core::Error::PushRejected(rejection) => {
            assert_eq!(rejection.remote, "origin");
            assert_eq!(rejection.branch, "main");
            assert_eq!(rejection.reason, PushRejectReason::NonFastForward);
        }
        other => panic!("expected a typed rejection, got {other:?}"),
    }
    assert_eq!(repo.git_in_remote(&["rev-parse", "main"]), remote_before);
}

/// Force push is `--force-with-lease`: it overwrites a remote we have
/// seen, and refuses - typed, `StaleLease` - one that moved since our
/// last fetch. A bare `--force` would pass the second half and lose the
/// other user's commit.
pub fn check_force_push_is_always_with_lease(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .with_initial()
        .with_origin()
        .push("main")
        .track("main", "origin")
        .commit("mine")
        .build();
    let repo_ref = repo.repo_ref();
    let force = PushOptions {
        force_with_lease: true,
        ..PushOptions::default()
    };

    // Someone pushed, and we have *not* fetched: the lease must hold.
    repo.push_from_elsewhere("main", "theirs");
    let theirs = repo.git_in_remote(&["rev-parse", "main"]);
    let err = block_on(backend.push(&repo_ref, "origin", &force))
        .expect_err("a stale lease must refuse the force push");
    assert!(
        matches!(
            &err,
            gitpoulpi_core::Error::PushRejected(r) if r.reason == PushRejectReason::StaleLease
        ),
        "expected a StaleLease rejection, got {err:?}"
    );
    assert_eq!(repo.git_in_remote(&["rev-parse", "main"]), theirs);

    // Having fetched, we have seen their commit: the force push goes.
    block_on(backend.fetch(&repo_ref, "origin", &FetchOptions::default())).unwrap();
    block_on(backend.push(&repo_ref, "origin", &force)).expect("lease honoured after fetch");
    assert_eq!(
        repo.git_in_remote(&["rev-parse", "main"]),
        repo.git(&["rev-parse", "main"])
    );
}

/// The three pull modes against a diverged branch: fast-forward only
/// refuses (typed, nothing changed), merge makes a merge commit, rebase
/// makes history linear.
pub fn check_pull_modes_on_a_diverged_branch(backend: &dyn GitBackend) {
    let build = || {
        let repo = TestRepoBuilder::new()
            .with_initial()
            .with_origin()
            .push("main")
            .track("main", "origin")
            .commit("mine")
            .build();
        repo.push_from_elsewhere("main", "theirs");
        repo
    };
    let pull = |repo: &crate::FixtureRepo, mode: PullMode| {
        block_on(backend.pull(&repo.repo_ref(), &PullOptions { mode, remote: None }))
    };

    let repo = build();
    let before = repo.git(&["rev-parse", "HEAD"]);
    let err = pull(&repo, PullMode::FastForwardOnly).expect_err("ff-only must refuse");
    assert!(
        matches!(err, gitpoulpi_core::Error::NotFastForward),
        "expected NotFastForward, got {err:?}"
    );
    assert_eq!(repo.git(&["rev-parse", "HEAD"]), before);

    let repo = build();
    let outcome = pull(&repo, PullMode::Merge).expect("merge pull");
    assert!(matches!(outcome, PullOutcome::Merged(_)), "{outcome:?}");
    assert_eq!(
        repo.git(&["rev-list", "--parents", "-n", "1", "HEAD"])
            .split(' ')
            .count(),
        3,
        "a merge pull of a diverged branch makes a two-parent commit"
    );

    let repo = build();
    pull(&repo, PullMode::Rebase).expect("rebase pull");
    assert_eq!(
        repo.git(&["rev-list", "--merges", "--count", "HEAD"]),
        "0",
        "a rebase pull leaves no merge commit"
    );
    assert_eq!(repo.git(&["log", "-1", "--format=%s"]), "mine");
    assert_eq!(repo.git(&["log", "-1", "--format=%s", "HEAD~1"]), "theirs");
}

/// A fast-forwardable pull fast-forwards in every mode, and a branch
/// with no upstream can still pull when told which remote to use.
pub fn check_pull_fast_forwards_and_accepts_an_explicit_remote(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .with_initial()
        .with_origin()
        .push("main")
        .build();
    repo.push_from_elsewhere("main", "theirs");
    let theirs = repo.git_in_remote(&["rev-parse", "main"]);

    // No upstream is configured: plain `git pull` would refuse.
    let outcome = block_on(backend.pull(
        &repo.repo_ref(),
        &PullOptions {
            mode: PullMode::FastForwardOnly,
            remote: Some("origin".into()),
        },
    ))
    .expect("pull from an explicit remote");
    assert!(
        matches!(outcome, PullOutcome::FastForwarded(_)),
        "{outcome:?}"
    );
    assert_eq!(repo.git(&["rev-parse", "HEAD"]), theirs);
}

// -- reword (edit a commit's message) --------------------------------------
//
// Assertions about the rewritten history use the fixture's own `git`, never
// the backend under test: two backends that wrote the same wrong commits
// would agree with themselves.

/// A rejected push keeps the typed rejection the dialog is keyed on *and*
/// the facts the Activity Log shows: the command that ran, its exit code and
/// git's own words.
pub fn check_a_rejected_push_carries_the_command_and_stderr(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .with_initial()
        .with_origin()
        .push("main")
        .track("main", "origin")
        .commit("mine")
        .build();
    let repo_ref = repo.repo_ref();
    repo.push_from_elsewhere("main", "theirs");

    let err = block_on(backend.push(&repo_ref, "origin", &PushOptions::default()))
        .expect_err("a diverged push must fail");
    let detail = err.detail().expect("a rejection carries its detail");
    let command = detail.command.as_deref().expect("the command line");
    assert!(command.starts_with("git push"), "{command}");
    assert!(command.contains("origin"), "{command}");
    assert!(
        detail.exit_code.is_some_and(|c| c != 0),
        "{:?}",
        detail.exit_code
    );
    assert!(
        detail.stderr.to_lowercase().contains("rejected"),
        "git's own words must be kept: {:?}",
        detail.stderr
    );
}

/// A push or pull that cannot reach its remote is an error whose detail
/// holds the command, a non-zero exit code and git's stderr, and nothing in
/// the error, its message or its detail holds the password that was in the
/// remote's URL.
pub fn check_a_failed_network_operation_carries_detail_without_secrets(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new().with_initial().build();
    repo.git(&[
        "remote",
        "add",
        "origin",
        "http://me:topsecret@127.0.0.1:1/r.git",
    ]);
    let repo_ref = repo.repo_ref();

    let push = block_on(backend.push(&repo_ref, "origin", &PushOptions::default()))
        .expect_err("pushing to a closed port must fail");
    let pull = block_on(backend.pull(
        &repo_ref,
        &PullOptions {
            mode: PullMode::Merge,
            remote: Some("origin".into()),
        },
    ))
    .expect_err("pulling from a closed port must fail");
    for (what, err) in [("push", push), ("pull", pull)] {
        let detail = err.detail().unwrap_or_else(|| panic!("{what}: {err:?}"));
        let command = detail.command.as_deref().expect("the command line");
        assert!(command.starts_with(&format!("git {what}")), "{command}");
        assert!(detail.exit_code.is_some_and(|c| c != 0), "{what}");
        assert!(
            !detail.stderr.trim().is_empty(),
            "{what}: git's stderr must be kept"
        );
        let everything = format!("{err} {err:?} {detail:?}");
        assert!(
            !everything.contains("topsecret"),
            "{what} leaked the password: {everything}"
        );
    }
}

/// `commit.template` and `core.commentChar` are readable and writable
/// repository-local keys (the Message template settings page): free text,
/// replaced not duplicated, cleared with `None`, and the system git sees the
/// value.
pub fn check_repo_config_reads_and_writes_the_commit_template(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new().with_initial().build();
    let repo_ref = repo.repo_ref();
    assert_eq!(
        backend.repo_config(&repo_ref, "commit.template").unwrap(),
        None
    );
    backend
        .set_repo_config(&repo_ref, "commit.template", Some("/tmp/one.txt"))
        .unwrap();
    backend
        .set_repo_config(&repo_ref, "commit.template", Some("/tmp/two.txt"))
        .unwrap();
    assert_eq!(
        backend
            .repo_config(&repo_ref, "commit.template")
            .unwrap()
            .as_deref(),
        Some("/tmp/two.txt")
    );
    assert_eq!(git_local(repo.root(), "commit.template"), "/tmp/two.txt");
    backend
        .set_repo_config(&repo_ref, "core.commentChar", Some(";"))
        .unwrap();
    assert_eq!(
        backend
            .repo_config(&repo_ref, "core.commentChar")
            .unwrap()
            .as_deref(),
        Some(";")
    );
    backend
        .set_repo_config(&repo_ref, "commit.template", None)
        .unwrap();
    assert_eq!(
        backend.repo_config(&repo_ref, "commit.template").unwrap(),
        None
    );
    assert_eq!(git_local(repo.root(), "commit.template"), "");
    assert_eq!(
        backend
            .repo_config(&repo_ref, "core.commentChar")
            .unwrap()
            .as_deref(),
        Some(";"),
        "the other key is untouched"
    );
}
