use super::head_oid;
use crate::TestRepoBuilder;
use gitpoulpi_core::domain::{CheckoutTarget, LogOptions, Oid, RefKind};
use gitpoulpi_core::GitBackend;

pub fn check_status_and_log(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new().with_initial().build();
    let repo_ref = repo.repo_ref();
    std::fs::write(repo.root().join("new.txt"), "hello").unwrap();

    let status = backend.status(&repo_ref).unwrap();
    assert!(status
        .files
        .iter()
        .any(|f| f.path.to_str() == Some("new.txt")));

    let log = backend.log(&repo_ref, &LogOptions::default()).unwrap();
    assert_eq!(log.nodes.len(), 1);
    assert_eq!(log.nodes[0].summary, "initial");
}

pub fn check_commit_message_body(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .commit_with_body("summary line", "first paragraph.\n\nsecond paragraph.")
        .build();
    let repo_ref = repo.repo_ref();

    let log = backend.log(&repo_ref, &LogOptions::default()).unwrap();
    assert_eq!(log.nodes[0].summary, "summary line");
    assert_eq!(
        log.nodes[0].message,
        "summary line\n\nfirst paragraph.\n\nsecond paragraph."
    );
}

/// `LogOptions::all_refs` must behave like `git log --all`: it shows
/// commits from *every* branch tip, including one that is not an ancestor
/// of the checked-out branch. The default (`false`) stays HEAD-only, so
/// both branches of the assertion matter.
pub fn check_log_all_refs_includes_every_branch_tip(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .commit("on-main")
        .branch("feature")
        .commit("on-feature-only")
        .checkout("main")
        .build();
    let repo_ref = repo.repo_ref();

    // Plain (HEAD-only) log from main must NOT see feature's commit.
    let head_only = backend.log(&repo_ref, &LogOptions::default()).unwrap();
    assert!(head_only
        .nodes
        .iter()
        .all(|n| n.summary != "on-feature-only"));

    // `all_refs: true` must see it even though feature isn't checked out
    // and its tip isn't reachable from HEAD.
    let all = backend
        .log(
            &repo_ref,
            &LogOptions {
                all_refs: true,
                ..LogOptions::default()
            },
        )
        .unwrap();
    assert!(
        all.nodes.iter().any(|n| n.summary == "on-feature-only"),
        "all_refs log must include commits from every branch, not just HEAD's ancestry"
    );
    assert!(all.nodes.iter().any(|n| n.summary == "on-main"));
}

/// `LogOptions::skip` must page history like `git log --skip=<n>`: the
/// same walk with the first `n` commits dropped. The TUI's lazy loading
/// stands on two properties asserted here:
///
/// - **pages tile the full walk exactly**, with no gap and no overlap;
/// - **a skip past the end is empty, not an error**, which is how the
///   client learns history has ended.
///
/// Guarded on both backends because they reach it by different means
/// (`--skip` vs dropping revwalk entries); a backend that applied the
/// skip after the limit, or re-ordered the walk per page, would tile wrong.
pub fn check_log_skip_pages_history(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .commit("c1")
        .commit("c2")
        .commit("c3")
        .commit("c4")
        .commit("c5")
        .commit("c6")
        .build();
    let repo_ref = repo.repo_ref();

    let whole = backend.log(&repo_ref, &LogOptions::default()).unwrap();
    let all: Vec<Oid> = whole.nodes.iter().map(|n| n.id).collect();
    assert_eq!(all.len(), 6, "fixture should hold six commits");

    let page = |skip: u32, limit: u32| -> Vec<Oid> {
        backend
            .log(
                &repo_ref,
                &LogOptions {
                    limit: Some(limit),
                    skip,
                    ..LogOptions::default()
                },
            )
            .unwrap()
            .nodes
            .iter()
            .map(|n| n.id)
            .collect()
    };

    // Two-at-a-time paging must reconstruct the whole walk, in order.
    let mut tiled: Vec<Oid> = Vec::new();
    for p in 0..3u32 {
        let got = page(p * 2, 2);
        let lo = p as usize * 2;
        assert_eq!(
            got,
            all[lo..lo + 2].to_vec(),
            "page {p} of two must be the same slice of the unpaged walk"
        );
        tiled.extend(got);
    }
    assert_eq!(tiled, all, "pages must tile the full walk with no gap");

    // A partial last page, and a skip past the end. Both are what the
    // lazy loader meets at the root commit.
    assert_eq!(page(4, 10), all[4..].to_vec());
    assert!(
        page(6, 10).is_empty(),
        "a skip past the end of history is an empty page, not an error"
    );
    assert!(page(99, 10).is_empty());

    // `skip` composes with `all_refs`, which is the combination the graph
    // actually asks for.
    let all_refs_page = backend
        .log(
            &repo_ref,
            &LogOptions {
                limit: Some(2),
                skip: 2,
                all_refs: true,
                ..LogOptions::default()
            },
        )
        .unwrap();
    assert_eq!(all_refs_page.nodes.len(), 2);
}

pub fn check_head_ref_flag(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new().with_initial().build();
    let repo_ref = repo.repo_ref();
    let head = head_oid(backend, &repo_ref);
    backend.branch_create(&repo_ref, "feature", head).unwrap();

    // Still on main: main is HEAD, feature is not.
    let refs = backend.refs(&repo_ref).unwrap();
    assert!(refs
        .iter()
        .any(|r| matches!(&r.kind, RefKind::LocalBranch { name } if name == "main") && r.is_head));
    assert!(refs.iter().any(
        |r| matches!(&r.kind, RefKind::LocalBranch { name } if name == "feature") && !r.is_head
    ));

    backend
        .checkout(&repo_ref, &CheckoutTarget::Branch("feature".into()))
        .unwrap();
    let refs = backend.refs(&repo_ref).unwrap();
    assert!(refs.iter().any(
        |r| matches!(&r.kind, RefKind::LocalBranch { name } if name == "feature") && r.is_head
    ));
    assert!(refs
        .iter()
        .any(|r| matches!(&r.kind, RefKind::LocalBranch { name } if name == "main") && !r.is_head));
    assert!(!refs
        .iter()
        .any(|r| r.is_head && !matches!(&r.kind, RefKind::LocalBranch { .. })));
}

/// An annotated tag's own ref points at a *tag object*, so `refs()` must
/// peel it: returning the tag object's oid as `target` would make a lookup
/// of `r.target == commit` miss and the tag's pill never render.
pub fn check_tag_target_resolves_to_the_tagged_commit(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .with_initial()
        .annotated_tag("v1.0.0")
        .build();
    let repo_ref = repo.repo_ref();
    let head = head_oid(backend, &repo_ref);

    let refs = backend.refs(&repo_ref).unwrap();
    let tag = refs
        .iter()
        .find(|r| matches!(&r.kind, RefKind::Tag { name } if name == "v1.0.0"))
        .expect("annotated tag should be listed");
    assert_eq!(
        tag.target, head,
        "an annotated tag's target must be the commit it tags, not its own tag-object oid"
    );
}

/// A ref whose object is gone (a repository being deleted, a half-written
/// clone) must not take `refs()` down: the system `git` aborts (SIGABRT) when
/// asked for `%(*objectname)` of such a ref. The valid refs are still listed,
/// the annotated tag still peeled, and the broken branch keeps its raw id.
pub fn check_refs_survive_a_ref_to_a_missing_object(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .with_initial()
        .branch("a")
        .annotated_tag("v1")
        .build();
    let repo_ref = repo.repo_ref();
    let head = head_oid(backend, &repo_ref);

    // Written straight into the ref store, as a half-deleted repository
    // would be. Each sorts after a valid ref (`target` after `a`/`main`,
    // `zz` after `v1`), otherwise git only exits 128.
    let missing = "deadbeefdeadbeefdeadbeefdeadbeefdeadbeef";
    // A valid mirror ref sorting before the broken one (`aa` < `zz`): git
    // lists every ref name before it sorts, and only dies on the missing
    // object when it formats one; with a broken ref first it just exits 128.
    let first = repo
        .root()
        .join(".git/refs/gitpoulpi/remote-tags/origin/aa");
    std::fs::create_dir_all(first.parent().unwrap()).unwrap();
    std::fs::write(first, format!("{head}\n")).unwrap();
    for broken in [
        "refs/heads/target",
        "refs/tags/zz",
        "refs/gitpoulpi/remote-tags/origin/zz",
    ] {
        let path = repo.root().join(".git").join(broken);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, format!("{missing}\n")).unwrap();
    }

    let refs = backend
        .refs(&repo_ref)
        .expect("refs() answers despite a ref to a missing object");
    let tag = refs
        .iter()
        .find(|r| matches!(&r.kind, RefKind::Tag { name } if name == "v1"))
        .expect("the annotated tag is listed");
    assert_eq!(tag.target, head, "the annotated tag is still peeled");
    let target = refs
        .iter()
        .find(|r| matches!(&r.kind, RefKind::LocalBranch { name } if name == "target"))
        .expect("the broken branch is listed");
    assert_eq!(Some(target.target), Oid::from_hex(missing));
}

/// A fixture with a branch `a`, an annotated tag `v1`, and (written straight
/// into the ref store, as a half-deleted repository would be) a broken
/// branch `target` and a broken tag `zz` pointing at an object that is gone.
/// Each sorts after a valid ref of its kind: git only dies on the missing
/// object when it formats a row after a valid one, otherwise it exits 128.
fn repo_with_broken_refs() -> crate::FixtureRepo {
    let repo = TestRepoBuilder::new()
        .with_initial()
        .branch("a")
        .annotated_tag("v1")
        .build();
    let missing = "deadbeefdeadbeefdeadbeefdeadbeefdeadbeef";
    for broken in ["refs/heads/target", "refs/tags/zz"] {
        let path = repo.root().join(".git").join(broken);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, format!("{missing}\n")).unwrap();
    }
    repo
}

/// The filtered walk (`LogOptions::hidden`) lists tips with a `for-each-ref`
/// and must survive a ref to a missing object: the valid refs still count.
pub fn check_log_hidden_survives_a_ref_to_a_missing_object(backend: &dyn GitBackend) {
    let repo = repo_with_broken_refs();
    let repo_ref = repo.repo_ref();
    let opts = LogOptions {
        all_refs: true,
        hidden: gitpoulpi_core::domain::RefFilter {
            names: vec!["a".into()],
            ..Default::default()
        },
        ..LogOptions::default()
    };
    let graph = backend
        .log(&repo_ref, &opts)
        .expect("a filtered log answers despite a ref to a missing object");
    assert_eq!(graph.nodes.len(), 1, "the initial commit is still reached");
}

/// `commit_messages` lists its tips with a `for-each-ref` and must survive a
/// ref to a missing object: the valid commits are still answered.
pub fn check_commit_messages_survive_a_ref_to_a_missing_object(backend: &dyn GitBackend) {
    let repo = repo_with_broken_refs();
    let repo_ref = repo.repo_ref();
    let answer = backend
        .commit_messages(
            &repo_ref,
            &gitpoulpi_core::domain::CommitMessagesQuery::default(),
        )
        .expect("commit_messages answers despite a ref to a missing object");
    assert_eq!(
        answer.commits.len(),
        1,
        "the initial commit is still listed"
    );
}

/// The history rewrite lists refs with a `for-each-ref` and must survive a
/// ref to a missing object (the size analysis walks every listed ref).
pub fn check_rewrite_analyze_survives_a_ref_to_a_missing_object(backend: &dyn GitBackend) {
    let repo = repo_with_broken_refs();
    let repo_ref = repo.repo_ref();
    backend
        .rewrite_analyze(
            &repo_ref,
            &gitpoulpi_core::domain::rewrite::Scope::default(),
            5,
            &mut |_| std::ops::ControlFlow::Continue(()),
        )
        .expect("analyze answers despite a ref to a missing object");
}

pub fn check_branch_create_checkout_delete(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new().with_initial().build();
    let repo_ref = repo.repo_ref();
    let head = head_oid(backend, &repo_ref);

    backend.branch_create(&repo_ref, "feature", head).unwrap();
    backend
        .checkout(&repo_ref, &CheckoutTarget::Branch("feature".into()))
        .unwrap();
    let refs = backend.refs(&repo_ref).unwrap();
    assert!(refs
        .iter()
        .any(|r| matches!(&r.kind, RefKind::LocalBranch { name } if name == "feature")));

    backend
        .checkout(&repo_ref, &CheckoutTarget::Branch("main".into()))
        .unwrap();
    backend.branch_delete(&repo_ref, "feature").unwrap();
    let refs = backend.refs(&repo_ref).unwrap();
    assert!(!refs
        .iter()
        .any(|r| matches!(&r.kind, RefKind::LocalBranch { name } if name == "feature")));
}

pub fn check_tag_create_delete(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new().with_initial().build();
    let repo_ref = repo.repo_ref();
    let head = head_oid(backend, &repo_ref);

    backend
        .tag_create(&repo_ref, "v1", head, Some("release v1"))
        .unwrap();
    let refs = backend.refs(&repo_ref).unwrap();
    assert!(refs
        .iter()
        .any(|r| matches!(&r.kind, RefKind::Tag { name } if name == "v1")));

    backend.tag_delete(&repo_ref, "v1").unwrap();
    let refs = backend.refs(&repo_ref).unwrap();
    assert!(!refs
        .iter()
        .any(|r| matches!(&r.kind, RefKind::Tag { name } if name == "v1")));
}

/// Every revision spelling a human or an agent can type at the CLI has to
/// land on the same commit through either backend.
///
/// The CLI/MCP surfaces receive `String`s (`main`, `HEAD~1`, a short sha)
/// while mutating operations take an `Oid`, and nothing else may shell out
/// to `git rev-parse` to bridge the two (architecture principle 2).
pub fn check_resolve_revision_spellings(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .with_initial()
        .commit("second")
        .annotated_tag("v1")
        .build();
    let repo_ref = repo.repo_ref();

    let log = backend.log(&repo_ref, &LogOptions::default()).unwrap();
    let tip = log.nodes[0].id;
    let parent = log.nodes[1].id;

    assert_eq!(backend.resolve(&repo_ref, "HEAD").unwrap(), tip);
    assert_eq!(backend.resolve(&repo_ref, "main").unwrap(), tip);
    assert_eq!(backend.resolve(&repo_ref, &tip.to_hex()).unwrap(), tip);
    assert_eq!(backend.resolve(&repo_ref, &tip.to_hex()[..8]).unwrap(), tip);
    assert_eq!(backend.resolve(&repo_ref, "HEAD~1").unwrap(), parent);
    assert_eq!(backend.resolve(&repo_ref, "HEAD^").unwrap(), parent);

    // An *annotated* tag's ref points at a tag object, not at a commit.
    // Resolving has to peel it, or every `--at v1` would hand a tag oid to
    // an operation expecting a commit -- the same bug `refs()` once had.
    assert_eq!(backend.resolve(&repo_ref, "v1").unwrap(), tip);

    assert!(
        backend.resolve(&repo_ref, "no-such-ref").is_err(),
        "an unknown revision must be an error, not a silent fallback to HEAD"
    );
}

/// `merge_base` is what the code review view diffs from: the commit a
/// branch forked at, not the tip the target has moved on to.
pub fn check_merge_base_is_the_fork_point(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .with_initial()
        .branch("feature")
        .checkout("feature")
        .commit_file("f.txt", "f\n", "on feature")
        .checkout("main")
        .commit_file("m.txt", "m\n", "on main")
        .build();
    let repo_ref = repo.repo_ref();
    let fork = backend.resolve(&repo_ref, "main~1").unwrap();
    let main = backend.resolve(&repo_ref, "main").unwrap();
    let feature = backend.resolve(&repo_ref, "feature").unwrap();
    assert_eq!(
        backend.merge_base(&repo_ref, main, feature).unwrap(),
        Some(fork)
    );
    assert_eq!(
        backend.merge_base(&repo_ref, feature, main).unwrap(),
        Some(fork),
        "the order does not matter"
    );
    assert_eq!(
        backend.merge_base(&repo_ref, fork, main).unwrap(),
        Some(fork),
        "an ancestor is its own merge base with a descendant"
    );
}
