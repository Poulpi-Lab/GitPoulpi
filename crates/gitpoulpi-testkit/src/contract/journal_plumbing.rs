use super::{head_oid, rev};
use crate::TestRepoBuilder;
use gitpoulpi_core::domain::{
    CheckoutTarget, CommitOptions, HeadState, InitOptions, LogOptions, Oid, RefScope, RefUpdate,
    StageTarget, StashId, UndoRefusal, WorktreeAddOptions, WorktreeCommit, WorktreeRemoveOptions,
    WorktreeStart,
};
use gitpoulpi_core::{Error, GitBackend};

/// Both backends must report **the main worktree too**, first and with
/// `is_main: true`. `git worktree list` includes it while
/// `git2::Repository::worktrees()` returns only linked worktrees, so the
/// git2 side has to synthesize the main entry.
pub fn check_worktrees_lists_main_and_linked(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new().with_initial();
    let (repo, linked_path) = repo.worktree("wt-feature", "feature");
    let repo = repo.build();
    let repo_ref = repo.repo_ref();

    let worktrees = backend.worktrees(&repo_ref).unwrap();
    assert_eq!(
        worktrees.len(),
        2,
        "expected the main worktree plus one linked worktree, got {worktrees:?}"
    );

    let main = &worktrees[0];
    assert!(main.is_main, "the main worktree must be listed first");
    assert_eq!(main.branch.as_deref(), Some("main"));
    assert_eq!(canonical(&main.path), canonical(repo.root()));
    assert!(!main.is_locked);

    let linked = &worktrees[1];
    assert!(!linked.is_main);
    assert_eq!(linked.branch.as_deref(), Some("feature"));
    assert_eq!(canonical(&linked.path), canonical(&linked_path));
    assert!(!linked.is_locked);
}

/// A repo with no `git worktree add` still *has* one worktree - its own
/// working directory. Returning an empty vec here would make the TUI
/// hide the pane for a perfectly normal repo.
pub fn check_worktrees_includes_main_when_there_are_no_linked_ones(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new().with_initial().build();
    let repo_ref = repo.repo_ref();

    let worktrees = backend.worktrees(&repo_ref).unwrap();
    assert_eq!(worktrees.len(), 1, "got {worktrees:?}");
    assert!(worktrees[0].is_main);
    assert_eq!(worktrees[0].branch.as_deref(), Some("main"));
    assert_eq!(canonical(&worktrees[0].path), canonical(repo.root()));
}

/// Temp dirs, `git worktree list` and libgit2 can each spell the same
/// directory differently on Windows (8.3 names, drive-letter case,
/// `\\?\` prefixes), so paths are only comparable once canonicalized on
/// both sides.
fn canonical(path: &std::path::Path) -> std::path::PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

/// The root and any path inside it must produce the *same* `RepoRef`:
/// `RepoId` is the daemon's subscription key, so a frontend that discovered
/// from a subdirectory and one that discovered from the root must end up
/// on one daemon entry, not two.
pub fn check_discover_from_the_root_and_from_inside_it(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .commit_file("src/deep/file.txt", "x", "initial")
        .build();
    let root = repo.root();

    let from_root = backend.discover(root).unwrap();
    assert_eq!(from_root.repo, repo.repo_ref());
    assert_eq!(from_root.name, "repo");
    assert!(!from_root.is_bare);
    assert!(!from_root.is_worktree);
    assert_eq!(from_root.head_branch.as_deref(), Some("main"));

    let from_subdir = backend.discover(&root.join("src").join("deep")).unwrap();
    assert_eq!(
        from_subdir.repo, from_root.repo,
        "a subdirectory must discover the same repository as its root"
    );

    let from_file = backend.discover(&root.join("src/deep/file.txt")).unwrap();
    assert_eq!(
        from_file.repo, from_root.repo,
        "a file inside the repo must discover the repository containing it"
    );
}

/// Both of the ways an "open repository" flow fails have to be an `Err`,
/// not a `RepoRef` pointing at nothing: a directory that is not in a
/// repository, and a path that is gone from disk - which is what a
/// stale recent-repositories entry looks like.
pub fn check_discover_rejects_what_is_not_a_repository(backend: &dyn GitBackend) {
    let plain = tempfile::tempdir().unwrap();
    assert!(
        backend.discover(plain.path()).is_err(),
        "a directory inside no repository must not discover one"
    );

    let missing = plain.path().join("gone");
    assert!(
        backend.discover(&missing).is_err(),
        "a path that does not exist must not discover a repository"
    );
}

/// A linked worktree's `.git` is a *file*, so a naive
/// `join(".git").exists()` walk reports it as an ordinary checkout.
/// Multi-repo tabs must tell them apart: opening one repository twice
/// through two worktrees is legitimate and must not be deduplicated.
pub fn check_discover_recognises_a_linked_worktree(backend: &dyn GitBackend) {
    let (builder, worktree_path) = TestRepoBuilder::new()
        .with_initial()
        .worktree("linked", "feature");
    let _repo = builder.build();

    let found = backend.discover(&worktree_path).unwrap();
    assert!(
        found.is_worktree,
        "a linked worktree must be reported as one"
    );
    assert_eq!(
        found.repo.root.canonicalize().unwrap(),
        worktree_path.canonicalize().unwrap(),
        "a linked worktree's root is the worktree, not the main checkout"
    );
    assert_eq!(found.head_branch.as_deref(), Some("feature"));
    assert!(!found.is_bare);
}

/// A detached HEAD has no branch. Reporting one anyway (`HEAD`, or the
/// last branch that pointed here) would put a branch name in a tab title
/// that no `checkout` could return to.
pub fn check_discover_reports_a_detached_head_as_no_branch(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .with_initial()
        .commit("second")
        .build();
    let repo_ref = repo.repo_ref();
    let parent = backend
        .log(&repo_ref, &LogOptions::default())
        .unwrap()
        .nodes[1]
        .id;
    backend
        .checkout(&repo_ref, &CheckoutTarget::Commit(parent))
        .unwrap();

    let found = backend.discover(repo.root()).unwrap();
    assert_eq!(
        found.head_branch, None,
        "a detached HEAD must report no branch"
    );
}

// -- init ----------------------------------------------------------------
//
// The other half of `discover`: it finds the repository a directory is in,
// `init` turns a directory into one. Both end in a `RepoId` the daemon keys
// on, so both backends must produce the same spelling of a root.

/// Give a freshly-initialized repository an identity, so a check can
/// commit into it without depending on the machine's global git config.
///
/// Written into the repository's own config file rather than via `git
/// config`, because it must work for a repository either backend made and
/// the trait has no "set a config key" operation.
fn give_it_an_identity(root: &std::path::Path) {
    let config = root.join(".git").join("config");
    let mut existing = std::fs::read_to_string(&config).unwrap_or_default();
    existing.push_str("\n[user]\n\tname = Contract Suite\n\temail = contract@example.invalid\n");
    std::fs::write(&config, existing).unwrap();
}

/// The repository has to exist, be discoverable, and answer reads - and
/// the `RepoRef` `init` hands back has to be the one `discover` would
/// produce for the same path, or the tab that opens next is a different
/// repository to the daemon than the one just created.
pub fn check_init_creates_a_repository_discover_agrees_with(backend: &dyn GitBackend) {
    let parent = tempfile::tempdir().unwrap();
    // Two levels down, and neither exists: `git init <path>` creates the
    // whole chain, and a picker that makes the user create the folder
    // first cannot create anything.
    let path = parent.path().join("new").join("project");

    let made = backend.init(&path, &InitOptions::default()).unwrap();

    assert!(path.join(".git").exists(), "a .git directory must exist");
    assert_eq!(
        made.root.canonicalize().unwrap(),
        path.canonicalize().unwrap()
    );
    assert_eq!(
        made,
        backend.discover(&path).unwrap().repo,
        "initializing and then discovering the same path must agree"
    );
    assert_eq!(
        made.id,
        gitpoulpi_core::domain::RepoId::from_path(&made.root),
        "the id must be the one every later request will be keyed on"
    );

    // An empty repository answers reads rather than failing: this is
    // the state the TUI opens the new tab in.
    assert!(backend.status(&made).unwrap().files.is_empty());
    assert!(backend.refs(&made).unwrap().is_empty());
    assert!(
        backend.discover(&path).unwrap().head_branch.is_none(),
        "an unborn HEAD is no branch, not a failure"
    );
}

/// `initial_branch` has to name the branch the first commit lands on,
/// not merely be accepted. This is the one thing `general.default_
/// branch_name` does, so a backend that ignored the option would make
/// that setting a lie.
pub fn check_init_honours_the_initial_branch_name(backend: &dyn GitBackend) {
    let parent = tempfile::tempdir().unwrap();
    let path = parent.path().join("trunked");

    let made = backend
        .init(
            &path,
            &InitOptions {
                initial_branch: Some("trunk".into()),
                ..InitOptions::default()
            },
        )
        .unwrap();
    give_it_an_identity(&made.root);

    std::fs::write(made.root.join("a.txt"), "one").unwrap();
    backend
        .stage(&made, &StageTarget::File("a.txt".into()))
        .unwrap();
    backend
        .commit(&made, "first", &CommitOptions::default())
        .unwrap();

    assert_eq!(
        backend.discover(&path).unwrap().head_branch.as_deref(),
        Some("trunk"),
        "the first commit must land on the branch `init` was asked for"
    );
}

/// Re-initializing a repository is refused: `git init` and
/// `git2::Repository::init` both happily re-run, so a mistyped `Create`
/// path would report "created" and open the repository already there. It
/// is a rule GitPoulpi adds, so the suite enforces it on both backends.
pub fn check_init_refuses_a_path_that_is_already_a_repository(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new().with_initial().build();

    assert!(
        backend.init(repo.root(), &InitOptions::default()).is_err(),
        "an existing repository root must not be re-initialized"
    );
    // ...and the repository it refused is untouched.
    assert_eq!(
        backend
            .log(&repo.repo_ref(), &LogOptions::default())
            .unwrap()
            .nodes
            .len(),
        1
    );

    // A directory *inside* a repository is a different matter: a nested
    // repository is legitimate, and refusing it would be inventing a
    // rule git does not have.
    let nested = repo.root().join("vendor").join("thing");
    let made = backend.init(&nested, &InitOptions::default()).unwrap();
    assert_eq!(
        made.root.canonicalize().unwrap(),
        nested.canonicalize().unwrap(),
        "a nested repository is its own root, not the enclosing one"
    );
}

// -- clone ---------------------------------------------------------------
//
// Cloned from a local fixture over a filesystem path, so the suite stays
// runnable with no network (see the mission's network constraint).

fn oid_of(repo: &crate::FixtureRepo, rev: &str) -> Oid {
    Oid::from_hex(&repo.git(&["rev-parse", rev])).unwrap()
}

fn is_ref_moved(result: gitpoulpi_core::Result<()>) -> bool {
    matches!(
        result,
        Err(Error::UndoRefused(UndoRefusal::RefMoved { .. }))
    )
}

/// The undo journal's source of truth: raw values, so an annotated tag
/// is its tag *object* (restoring it must give back the same object, not
/// a new lightweight tag at the peeled commit).
pub fn check_ref_snapshot_reports_raw_tag_objects(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .with_initial()
        .branch("feature")
        .checkout("main")
        .annotated_tag("v1")
        .tag("light")
        .build();
    let repo_ref = repo.repo_ref();
    let snap = backend
        .ref_snapshot(&repo_ref, &RefScope::default())
        .unwrap();
    let commit = oid_of(&repo, "main");
    assert_eq!(snap.head, HeadState::Branch("main".into()));
    assert_eq!(snap.refs["refs/heads/main"], commit);
    assert_eq!(snap.refs["refs/heads/feature"], commit);
    assert_eq!(snap.refs["refs/tags/light"], commit);
    let tag_object = oid_of(&repo, "refs/tags/v1");
    assert_ne!(tag_object, commit, "an annotated tag has its own object");
    assert_eq!(snap.refs["refs/tags/v1"], tag_object, "raw, not peeled");

    let heads = backend.ref_snapshot(&repo_ref, &RefScope::heads()).unwrap();
    assert!(heads.refs.keys().all(|n| n.starts_with("refs/heads/")));
    assert_eq!(heads.refs.len(), 2);
}

pub fn check_ref_snapshot_reports_head_unborn_and_detached(backend: &dyn GitBackend) {
    let unborn = TestRepoBuilder::new().build();
    let snap = backend
        .ref_snapshot(&unborn.repo_ref(), &RefScope::default())
        .unwrap();
    assert_eq!(snap.head, HeadState::Branch("main".into()));
    assert!(snap.refs.is_empty());

    let repo = TestRepoBuilder::new()
        .with_initial()
        .commit("second")
        .build();
    let repo_ref = repo.repo_ref();
    let first = oid_of(&repo, "HEAD~1");
    repo.git(&["checkout", "--detach", "HEAD~1"]);
    let snap = backend
        .ref_snapshot(&repo_ref, &RefScope::default())
        .unwrap();
    assert_eq!(snap.head, HeadState::Detached(first));
    assert!(snap.refs.contains_key("refs/heads/main"));
}

/// The compare-and-swap transaction the undo journal moves refs with:
/// a stale expectation anywhere leaves *every* ref as it was.
pub fn check_update_refs_is_all_or_nothing(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .with_initial()
        .branch("a")
        .checkout("main")
        .branch("b")
        .checkout("main")
        .commit("second")
        .build();
    let repo_ref = repo.repo_ref();
    let c1 = oid_of(&repo, "a");
    let c2 = oid_of(&repo, "main");
    assert_ne!(c1, c2);

    // 1st update is valid, 2nd expects a stale value: neither applies.
    let result = backend.update_refs(
        &repo_ref,
        &[
            RefUpdate::Update {
                name: "refs/heads/a".into(),
                old: c1,
                new: c2,
            },
            RefUpdate::Update {
                name: "refs/heads/b".into(),
                old: c2,
                new: c1,
            },
        ],
        "gitpoulpi: test",
    );
    assert!(is_ref_moved(result));
    assert_eq!(oid_of(&repo, "a"), c1, "the valid update must not apply");
    assert_eq!(oid_of(&repo, "b"), c1);

    // Creating a ref that exists, deleting with a stale old, and a failed
    // verify are all refusals that change nothing.
    for bad in [
        RefUpdate::Create {
            name: "refs/heads/a".into(),
            new: c2,
        },
        RefUpdate::Delete {
            name: "refs/heads/a".into(),
            old: c2,
        },
        RefUpdate::Verify {
            name: "refs/heads/a".into(),
            old: c2,
        },
        RefUpdate::Update {
            name: "refs/heads/missing".into(),
            old: c1,
            new: c2,
        },
    ] {
        let result = backend.update_refs(
            &repo_ref,
            &[
                RefUpdate::Create {
                    name: "refs/heads/fresh".into(),
                    new: c1,
                },
                bad,
            ],
            "gitpoulpi: test",
        );
        assert!(is_ref_moved(result));
        assert!(
            repo.git(&["for-each-ref", "refs/heads/fresh"]).is_empty(),
            "the create must roll back with the refused update"
        );
    }
    assert_eq!(oid_of(&repo, "a"), c1);

    // The valid transaction applies every update at once.
    backend
        .update_refs(
            &repo_ref,
            &[
                RefUpdate::Update {
                    name: "refs/heads/a".into(),
                    old: c1,
                    new: c2,
                },
                RefUpdate::Delete {
                    name: "refs/heads/b".into(),
                    old: c1,
                },
                RefUpdate::Create {
                    name: "refs/heads/c".into(),
                    new: c1,
                },
                RefUpdate::Verify {
                    name: "refs/heads/main".into(),
                    old: c2,
                },
            ],
            "gitpoulpi: undo test",
        )
        .unwrap();
    assert_eq!(oid_of(&repo, "a"), c2);
    assert_eq!(oid_of(&repo, "c"), c1);
    assert!(repo.git(&["for-each-ref", "refs/heads/b"]).is_empty());
    assert!(
        repo.git(&["reflog", "show", "refs/heads/a"])
            .contains("gitpoulpi: undo test"),
        "the reflog explains the move"
    );
}

pub fn check_update_refs_set_head(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .with_initial()
        .branch("feature")
        .checkout("main")
        .commit("second")
        .build();
    let repo_ref = repo.repo_ref();
    let c1 = oid_of(&repo, "feature");
    let c2 = oid_of(&repo, "main");
    let main = HeadState::Branch("main".into());
    let feature = HeadState::Branch("feature".into());
    let tracked = repo.root().join("second.txt");
    assert!(tracked.exists());

    // A stale `old` refuses and moves nothing.
    let stale = backend.update_refs(
        &repo_ref,
        &[RefUpdate::SetHead {
            old: feature.clone(),
            new: main.clone(),
        }],
        "gitpoulpi: test",
    );
    assert!(matches!(
        stale,
        Err(Error::UndoRefused(UndoRefusal::HeadMoved { .. }))
    ));
    assert_eq!(repo.git(&["symbolic-ref", "HEAD"]), "refs/heads/main");

    // Branch -> branch: HEAD moves, the files do not.
    backend
        .update_refs(
            &repo_ref,
            &[RefUpdate::SetHead {
                old: main.clone(),
                new: feature.clone(),
            }],
            "gitpoulpi: test",
        )
        .unwrap();
    assert_eq!(repo.git(&["symbolic-ref", "HEAD"]), "refs/heads/feature");
    assert!(tracked.exists(), "SetHead never touches files");

    // Branch -> detached, then detached -> detached, all in one
    // transaction with a ref update.
    backend
        .update_refs(
            &repo_ref,
            &[RefUpdate::SetHead {
                old: feature.clone(),
                new: HeadState::Detached(c2),
            }],
            "gitpoulpi: test",
        )
        .unwrap();
    assert_eq!(oid_of(&repo, "HEAD"), c2);
    assert!(repo.git(&["branch", "--show-current"]).is_empty());
    backend
        .update_refs(
            &repo_ref,
            &[
                RefUpdate::SetHead {
                    old: HeadState::Detached(c2),
                    new: HeadState::Detached(c1),
                },
                RefUpdate::Create {
                    name: "refs/heads/made".into(),
                    new: c2,
                },
            ],
            "gitpoulpi: test",
        )
        .unwrap();
    assert_eq!(oid_of(&repo, "HEAD"), c1);
    assert_eq!(oid_of(&repo, "made"), c2);
    assert!(tracked.exists());
}

/// `SetSymbolic` creates, repoints and deletes an alias such as
/// `refs/remotes/origin/HEAD`, after the rest of its transaction, and
/// `ref_snapshot` lists it in `symrefs`.
pub fn check_update_refs_set_symbolic(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new().with_initial().build();
    let repo_ref = repo.repo_ref();
    let tip = oid_of(&repo, "main");
    let alias = "refs/remotes/origin/HEAD".to_string();
    let main = "refs/remotes/origin/main".to_string();
    let other = "refs/remotes/origin/other".to_string();

    let scope = RefScope {
        prefixes: vec!["refs/remotes/".into(), "refs/heads/".into()],
    };
    let symrefs = || backend.ref_snapshot(&repo_ref, &scope).unwrap().symrefs;
    assert!(symrefs().is_empty(), "no alias yet, and HEAD is not listed");

    // The alias is made in the same transaction as its target.
    backend
        .update_refs(
            &repo_ref,
            &[
                RefUpdate::Create {
                    name: main.clone(),
                    new: tip,
                },
                RefUpdate::Create {
                    name: other.clone(),
                    new: tip,
                },
                RefUpdate::SetSymbolic {
                    name: alias.clone(),
                    old: None,
                    new: Some(main.clone()),
                },
            ],
            "gitpoulpi: test",
        )
        .unwrap();
    assert_eq!(repo.git(&["symbolic-ref", &alias]), main);
    assert_eq!(
        symrefs().get(&alias),
        Some(&main),
        "ref_snapshot lists the alias with its target, and only the alias"
    );
    assert_eq!(symrefs().len(), 1);
    assert_eq!(oid_of(&repo, &alias), tip, "it resolves like any alias");

    // Repointed, then deleted; the inverse of each puts the other back.
    let repoint = RefUpdate::SetSymbolic {
        name: alias.clone(),
        old: Some(main.clone()),
        new: Some(other.clone()),
    };
    backend
        .update_refs(&repo_ref, std::slice::from_ref(&repoint), "gitpoulpi: test")
        .unwrap();
    assert_eq!(repo.git(&["symbolic-ref", &alias]), other);
    backend
        .update_refs(&repo_ref, &[repoint.inverse()], "gitpoulpi: test")
        .unwrap();
    assert_eq!(repo.git(&["symbolic-ref", &alias]), main);
    backend
        .update_refs(
            &repo_ref,
            &[
                RefUpdate::Delete {
                    name: main.clone(),
                    old: tip,
                },
                RefUpdate::SetSymbolic {
                    name: alias.clone(),
                    old: Some(main.clone()),
                    new: None,
                },
            ],
            "gitpoulpi: test",
        )
        .unwrap();
    assert!(symrefs().is_empty());
    assert!(!repo
        .git(&["for-each-ref", "refs/remotes/origin/"])
        .contains("HEAD"));
    assert!(repo
        .git(&["for-each-ref", "refs/remotes/origin/"])
        .contains("other"));
}

/// A pin naming an object the repository does not have is refused and
/// writes nothing. Outside detection relies on it: after a history rewrite
/// (`git filter-repo`, which also prunes the old objects) the refs snapshot
/// still holds the old ids, and pinning them would leave refs that `git
/// fsck` and `git branch -d` report as invalid.
pub fn check_update_refs_refuses_pin_to_missing_object(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new().with_initial().build();
    let repo_ref = repo.repo_ref();
    let head = oid_of(&repo, "HEAD");
    let missing = Oid::from_hex("230a508d2bc0c451c4081cce0f40f06ad9b22734").unwrap();
    assert!(
        repo.try_git(&["cat-file", "-e", &missing.to_string()])
            .is_none(),
        "the fixture must not have the object"
    );
    for updates in [
        vec![RefUpdate::Create {
            name: "refs/gitpoulpi/undo/1/0".into(),
            new: missing,
        }],
        vec![
            RefUpdate::Create {
                name: "refs/gitpoulpi/undo/1/0".into(),
                new: head,
            },
            RefUpdate::Create {
                name: "refs/gitpoulpi/undo/1/1".into(),
                new: missing,
            },
        ],
    ] {
        assert!(
            backend
                .update_refs(&repo_ref, &updates, "gitpoulpi: pin")
                .is_err(),
            "pinning a missing object must fail"
        );
        assert_eq!(
            repo.git(&["for-each-ref", "refs/gitpoulpi/"]),
            "",
            "a refused pin batch must write nothing"
        );
    }
}

/// A commit that only a `refs/gitpoulpi/undo/` pin keeps alive.
fn pinned_orphan(backend: &dyn GitBackend) -> (crate::FixtureRepo, Oid) {
    let repo = TestRepoBuilder::new()
        .with_initial()
        .branch("side")
        .checkout("side")
        .commit("pinned only")
        .checkout("main")
        .build();
    let repo_ref = repo.repo_ref();
    let orphan = oid_of(&repo, "side");
    backend
        .update_refs(
            &repo_ref,
            &[
                RefUpdate::Create {
                    name: "refs/gitpoulpi/undo/1/0".into(),
                    new: orphan,
                },
                RefUpdate::Delete {
                    name: "refs/heads/side".into(),
                    old: orphan,
                },
            ],
            "gitpoulpi: pin",
        )
        .unwrap();
    assert_eq!(oid_of(&repo, "refs/gitpoulpi/undo/1/0"), orphan);
    (repo, orphan)
}

pub fn check_refs_hide_gitpoulpi_namespace(backend: &dyn GitBackend) {
    let (repo, orphan) = pinned_orphan(backend);
    let repo_ref = repo.repo_ref();
    let refs = backend.refs(&repo_ref).unwrap();
    assert!(
        refs.iter().all(|r| r.target != orphan),
        "a pin must not show as a branch or tag"
    );
    let snap = backend
        .ref_snapshot(&repo_ref, &RefScope::default())
        .unwrap();
    assert!(snap.refs.keys().all(|n| !n.starts_with("refs/gitpoulpi/")));
}

pub fn check_log_all_refs_skips_gitpoulpi_namespace(backend: &dyn GitBackend) {
    let (repo, orphan) = pinned_orphan(backend);
    let repo_ref = repo.repo_ref();
    let log = backend
        .log(
            &repo_ref,
            &LogOptions {
                all_refs: true,
                ..LogOptions::default()
            },
        )
        .unwrap();
    assert!(
        log.nodes.iter().all(|n| n.id != orphan),
        "pinned commits must never appear in the graph"
    );
    let log = backend.log(&repo_ref, &LogOptions::default()).unwrap();
    assert!(log.nodes.iter().all(|n| n.id != orphan));
}

pub fn check_config_section_roundtrip(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .with_initial()
        .branch("feature")
        .checkout("main")
        .build();
    let repo_ref = repo.repo_ref();
    assert!(backend
        .config_section(&repo_ref, "branch.feature")
        .unwrap()
        .is_empty());

    let entries = vec![
        ("branch.feature.remote".to_string(), "origin".to_string()),
        (
            "branch.feature.merge".to_string(),
            "refs/heads/feature".to_string(),
        ),
    ];
    backend
        .set_config_section(&repo_ref, "branch.feature", &entries)
        .unwrap();
    assert_eq!(repo.git(&["config", "branch.feature.remote"]), "origin");
    let mut read = backend.config_section(&repo_ref, "branch.feature").unwrap();
    read.sort();
    let mut want = entries.clone();
    want.sort();
    assert_eq!(read, want);

    // Replacing drops keys that are not in the new list.
    backend
        .set_config_section(&repo_ref, "branch.feature", &entries[..1])
        .unwrap();
    assert_eq!(
        backend.config_section(&repo_ref, "branch.feature").unwrap(),
        entries[..1].to_vec()
    );
    // Empty removes the section.
    backend
        .set_config_section(&repo_ref, "branch.feature", &[])
        .unwrap();
    assert!(backend
        .config_section(&repo_ref, "branch.feature")
        .unwrap()
        .is_empty());

    // Deleting a branch drops its section: the journal records the
    // after-state as empty without asking.
    backend
        .set_config_section(&repo_ref, "branch.feature", &entries)
        .unwrap();
    backend.branch_delete(&repo_ref, "feature").unwrap();
    assert!(backend
        .config_section(&repo_ref, "branch.feature")
        .unwrap()
        .is_empty());

    // Only `branch.<name>` sections are accepted.
    assert!(backend.config_section(&repo_ref, "core").is_err());
    assert!(backend
        .set_config_section(
            &repo_ref,
            "core.editor",
            &[("core.editor.x".into(), "x".into())]
        )
        .is_err());
}

/// `stash_store` puts a dropped stash back, the very same commit, on top;
/// `stash_paths` names what a stash holds, untracked files included.
pub fn check_stash_store_and_paths(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new().with_initial().build();
    let repo_ref = repo.repo_ref();
    std::fs::write(repo.root().join("initial.txt"), "dirty").unwrap();
    std::fs::write(repo.root().join("fresh.txt"), "untracked").unwrap();

    let id = backend
        .stash_push(&repo_ref, Some("wip"), true)
        .unwrap()
        .expect("a stash");
    let stash = backend
        .stashes(&repo_ref)
        .unwrap()
        .into_iter()
        .find(|s| s.id == id)
        .unwrap();
    let commit = stash.commit.expect("a stash names its commit");
    assert_eq!(
        backend.stash_paths(&repo_ref, commit).unwrap(),
        vec!["fresh.txt".to_string(), "initial.txt".to_string()]
    );

    backend.stash_drop(&repo_ref, id).unwrap();
    assert!(backend.stashes(&repo_ref).unwrap().is_empty());

    backend
        .stash_store(&repo_ref, commit, &stash.message)
        .unwrap();
    let back = backend.stashes(&repo_ref).unwrap();
    assert_eq!(back.len(), 1);
    assert_eq!(back[0].id, StashId(0));
    assert_eq!(back[0].commit, Some(commit), "the very same commit");
    assert_eq!(back[0].message, stash.message);
    assert_eq!(back[0].base, stash.base);

    // Stored on top of another stash, it is stash@{0}; the other moves down.
    std::fs::write(repo.root().join("second.txt"), "x").unwrap();
    backend.stash_push(&repo_ref, Some("second"), true).unwrap();
    let top = backend.stashes(&repo_ref).unwrap()[0].commit.unwrap();
    backend.stash_drop(&repo_ref, StashId(0)).unwrap();
    backend.stash_store(&repo_ref, top, "second").unwrap();
    let list = backend.stashes(&repo_ref).unwrap();
    assert_eq!(list[0].commit, Some(top));
    assert_eq!(list[1].commit, Some(commit));

    // Anything that is not a stash commit is refused and changes nothing.
    let head = head_oid(backend, &repo_ref);
    assert!(backend.stash_store(&repo_ref, head, "nope").is_err());
    assert_eq!(backend.stashes(&repo_ref).unwrap().len(), 2);
}

/// The three ways to start a worktree: on a new branch, on an existing
/// branch, detached at a commit - each returned exactly as `worktrees()`
/// lists it, with the commit's files checked out there.
pub fn check_worktree_add_on_a_new_branch_an_existing_branch_and_a_commit(
    backend: &dyn GitBackend,
) {
    let repo = TestRepoBuilder::new()
        .with_initial()
        .branch("feature")
        .commit_file("feature.txt", "feature\n", "feature work")
        .checkout("main")
        .build();
    let repo_ref = repo.repo_ref();
    let initial = rev(&repo, "main");
    let base = repo.root().parent().unwrap().to_path_buf();

    // A new branch, at a commit, in a folder whose parent is missing.
    let new_path = base.join("nested").join("wt-new");
    let added = backend
        .worktree_add(
            &repo_ref,
            &new_path,
            &WorktreeAddOptions {
                start: WorktreeStart::Commit(initial),
                new_branch: Some("topic".into()),
            },
        )
        .unwrap();
    assert_eq!(added.branch.as_deref(), Some("topic"));
    assert!(!added.is_main);
    assert_eq!(canonical(&added.path), canonical(&new_path));
    assert_eq!(rev(&repo, "topic"), initial);
    assert!(new_path.join("initial.txt").exists());

    // An existing branch, checked out there.
    let feature_path = base.join("wt-feature");
    let added = backend
        .worktree_add(
            &repo_ref,
            &feature_path,
            &WorktreeAddOptions {
                start: WorktreeStart::Branch("feature".into()),
                new_branch: None,
            },
        )
        .unwrap();
    assert_eq!(added.branch.as_deref(), Some("feature"));
    assert!(feature_path.join("feature.txt").exists());

    // A commit, detached.
    let detached_path = base.join("wt-detached");
    let added = backend
        .worktree_add(
            &repo_ref,
            &detached_path,
            &WorktreeAddOptions {
                start: WorktreeStart::Commit(initial),
                new_branch: None,
            },
        )
        .unwrap();
    assert_eq!(added.branch, None);
    assert!(!detached_path.join("feature.txt").exists());

    let listed = backend.worktrees(&repo_ref).unwrap();
    assert_eq!(listed.len(), 4, "{listed:?}");
    assert_eq!(
        repo.git(&["symbolic-ref", "--short", "HEAD"]),
        "main",
        "the main worktree stays where it was"
    );
}

/// A branch already checked out, a new-branch name already taken, a
/// start branch that does not exist and a non-empty folder are refused,
/// and none of them leaves a worktree behind.
pub fn check_worktree_add_refusals_leave_no_worktree(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .with_initial()
        .branch("feature")
        .checkout("main")
        .build();
    let repo_ref = repo.repo_ref();
    let base = repo.root().parent().unwrap().to_path_buf();
    let occupied = base.join("occupied");
    std::fs::create_dir(&occupied).unwrap();
    std::fs::write(occupied.join("keep.txt"), "mine").unwrap();

    let cases: [(std::path::PathBuf, WorktreeAddOptions); 4] = [
        (
            base.join("wt-main"),
            WorktreeAddOptions {
                start: WorktreeStart::Branch("main".into()),
                new_branch: None,
            },
        ),
        (
            base.join("wt-taken"),
            WorktreeAddOptions {
                start: WorktreeStart::Branch("main".into()),
                new_branch: Some("feature".into()),
            },
        ),
        (
            base.join("wt-unknown"),
            WorktreeAddOptions {
                start: WorktreeStart::Branch("no-such-branch".into()),
                new_branch: None,
            },
        ),
        (
            occupied.clone(),
            WorktreeAddOptions {
                start: WorktreeStart::Branch("feature".into()),
                new_branch: None,
            },
        ),
    ];
    for (path, opts) in cases {
        assert!(
            backend.worktree_add(&repo_ref, &path, &opts).is_err(),
            "{opts:?} at {path:?} should be refused"
        );
    }
    assert_eq!(backend.worktrees(&repo_ref).unwrap().len(), 1);
    assert_eq!(
        std::fs::read_to_string(occupied.join("keep.txt")).unwrap(),
        "mine"
    );
}

// -- create patch / apply patch ----------------

/// `move_head_keep` is a compare-and-swap on the checked-out branch that
/// carries the files along and keeps unrelated local changes.
pub fn check_move_head_keep_cas(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .with_initial()
        .branch("other")
        .checkout("main")
        .commit_file("second.txt", "two\n", "second")
        .build();
    let repo_ref = repo.repo_ref();
    let c1 = oid_of(&repo, "other");
    let c2 = oid_of(&repo, "main");
    std::fs::write(repo.root().join("untouched.txt"), "mine\n").unwrap();

    // A stale `from` and a branch that is not checked out both refuse.
    assert!(matches!(
        backend.move_head_keep(&repo_ref, "main", c1, c2),
        Err(Error::UndoRefused(UndoRefusal::RefMoved { .. }))
    ));
    assert!(matches!(
        backend.move_head_keep(&repo_ref, "other", c1, c2),
        Err(Error::UndoRefused(UndoRefusal::HeadMoved { .. }))
    ));
    assert_eq!(oid_of(&repo, "main"), c2);
    assert!(repo.root().join("second.txt").exists());

    // Back: the branch, the index and the files follow; the unrelated
    // untracked file stays.
    backend.move_head_keep(&repo_ref, "main", c2, c1).unwrap();
    assert_eq!(oid_of(&repo, "main"), c1);
    assert_eq!(oid_of(&repo, "HEAD"), c1);
    assert!(!repo.root().join("second.txt").exists());
    assert!(repo.root().join("untouched.txt").exists());
    // And forward again.
    backend.move_head_keep(&repo_ref, "main", c1, c2).unwrap();
    assert_eq!(oid_of(&repo, "main"), c2);
    assert!(repo.root().join("second.txt").exists());
    assert_eq!(repo.git(&["symbolic-ref", "HEAD"]), "refs/heads/main");
}

/// A local edit the move would overwrite refuses it, typed, and nothing
/// changes: not the branch, not the edit.
pub fn check_move_head_keep_refuses_overwrite(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .commit_file("a.txt", "one\n", "first")
        .commit_file("a.txt", "two\n", "second")
        .build();
    let repo_ref = repo.repo_ref();
    let c1 = oid_of(&repo, "HEAD~1");
    let c2 = oid_of(&repo, "HEAD");
    std::fs::write(repo.root().join("a.txt"), "my local edit\n").unwrap();

    let result = backend.move_head_keep(&repo_ref, "main", c2, c1);
    assert!(
        matches!(
            result,
            Err(Error::UndoRefused(UndoRefusal::WouldOverwrite { .. }))
        ),
        "{result:?}"
    );
    assert_eq!(oid_of(&repo, "main"), c2, "the branch did not move");
    assert_eq!(
        std::fs::read_to_string(repo.root().join("a.txt")).unwrap(),
        "my local edit\n",
        "the local edit survived"
    );
}

/// The index's tree id is stable, and `index_restore` puts the index back
/// exactly without touching the working tree.
pub fn check_index_tree_and_restore(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .commit_file("a.txt", "one\n", "first")
        .build();
    let repo_ref = repo.repo_ref();
    let clean = backend.index_tree(&repo_ref).unwrap();
    assert_eq!(
        clean,
        oid_of(&repo, "HEAD^{tree}"),
        "a clean index is HEAD's tree"
    );

    std::fs::write(repo.root().join("a.txt"), "two\n").unwrap();
    std::fs::write(repo.root().join("b.txt"), "bee\n").unwrap();
    backend
        .stage(&repo_ref, &StageTarget::File("a.txt".into()))
        .unwrap();
    let one = backend.index_tree(&repo_ref).unwrap();
    assert_ne!(one, clean);
    assert_eq!(one, backend.index_tree(&repo_ref).unwrap(), "stable");
    backend
        .stage(&repo_ref, &StageTarget::File("b.txt".into()))
        .unwrap();
    let two = backend.index_tree(&repo_ref).unwrap();
    assert_ne!(two, one);

    backend.index_restore(&repo_ref, one).unwrap();
    assert_eq!(backend.index_tree(&repo_ref).unwrap(), one);
    assert_eq!(
        repo.git(&["diff", "--cached", "--name-only"]).trim(),
        "a.txt"
    );
    assert_eq!(
        std::fs::read_to_string(repo.root().join("b.txt")).unwrap(),
        "bee\n",
        "the working tree is untouched"
    );
    backend.index_restore(&repo_ref, clean).unwrap();
    assert_eq!(repo.git(&["diff", "--cached", "--name-only"]).trim(), "");
    assert_eq!(
        std::fs::read_to_string(repo.root().join("a.txt")).unwrap(),
        "two\n"
    );
    backend.index_restore(&repo_ref, two).unwrap();
    assert_eq!(backend.index_tree(&repo_ref).unwrap(), two);
}

fn snapshot_fixture() -> (crate::FixtureRepo, Vec<String>, Vec<u8>) {
    let mut binary: Vec<u8> = (0..=255u8).collect();
    binary.extend_from_slice(b"\r\nCRLF\r\n\0tail");
    let repo = TestRepoBuilder::new()
        .commit_file("tracked.txt", "one\ntwo\n", "first")
        .commit_file("sub/deep.txt", "deep\n", "deep")
        .build();
    // Tracked, modified and staged, then modified again.
    std::fs::write(repo.root().join("tracked.txt"), "staged\n").unwrap();
    repo.git(&["add", "tracked.txt"]);
    std::fs::write(repo.root().join("tracked.txt"), "staged\nunstaged\n").unwrap();
    // Untracked: binary with CRLF, in a new directory.
    std::fs::create_dir_all(repo.root().join("new/dir")).unwrap();
    std::fs::write(repo.root().join("new/dir/blob.bin"), &binary).unwrap();
    std::fs::write(repo.root().join("crlf.txt"), b"a\r\nb\r\n").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::write(repo.root().join("run.sh"), "#!/bin/sh\necho hi\n").unwrap();
        std::fs::set_permissions(
            repo.root().join("run.sh"),
            std::fs::Permissions::from_mode(0o755),
        )
        .unwrap();
        std::os::unix::fs::symlink("tracked.txt", repo.root().join("link")).unwrap();
    }
    #[cfg_attr(not(unix), allow(unused_mut))]
    let mut paths = vec![
        "tracked.txt".to_string(),
        "new".to_string(),
        "crlf.txt".to_string(),
        "sub/deep.txt".to_string(),
    ];
    #[cfg(unix)]
    {
        paths.push("run.sh".to_string());
        paths.push("link".to_string());
    }
    (repo, paths, binary)
}

/// Discard-style destruction, then restore: untracked, binary, CRLF,
/// executable and symlink files come back byte for byte, and so does the
/// index side (staged content differs from the working copy).
pub fn check_snapshot_round_trips_untracked_binary_exec_and_links(backend: &dyn GitBackend) {
    let (repo, paths, binary) = snapshot_fixture();
    let repo_ref = repo.repo_ref();
    let snap = backend.snapshot_create(&repo_ref, &paths).unwrap();
    let index_before = repo.git(&["ls-files", "-s"]);

    // Destroy it all.
    std::fs::remove_dir_all(repo.root().join("new")).unwrap();
    std::fs::remove_file(repo.root().join("crlf.txt")).unwrap();
    std::fs::write(repo.root().join("tracked.txt"), "clobbered\n").unwrap();
    std::fs::remove_file(repo.root().join("sub/deep.txt")).unwrap();
    repo.git(&["reset", "-q", "--hard"]);
    #[cfg(unix)]
    {
        std::fs::remove_file(repo.root().join("run.sh")).unwrap();
        std::fs::remove_file(repo.root().join("link")).unwrap();
    }

    backend
        .snapshot_restore(&repo_ref, snap.tree, &paths)
        .unwrap();

    assert_eq!(
        std::fs::read(repo.root().join("new/dir/blob.bin")).unwrap(),
        binary
    );
    assert_eq!(
        std::fs::read(repo.root().join("crlf.txt")).unwrap(),
        b"a\r\nb\r\n"
    );
    assert_eq!(
        std::fs::read_to_string(repo.root().join("tracked.txt")).unwrap(),
        "staged\nunstaged\n"
    );
    assert_eq!(
        std::fs::read_to_string(repo.root().join("sub/deep.txt")).unwrap(),
        "deep\n"
    );
    assert_eq!(
        repo.git(&["ls-files", "-s"]),
        index_before,
        "the index side"
    );
    assert_eq!(repo.git(&["show", ":tracked.txt"]), "staged");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(repo.root().join("run.sh"))
            .unwrap()
            .permissions()
            .mode();
        assert_ne!(mode & 0o111, 0, "the executable bit");
        assert_eq!(
            std::fs::read_link(repo.root().join("link")).unwrap(),
            std::path::PathBuf::from("tracked.txt")
        );
    }
    assert_eq!(
        backend.path_hashes(&repo_ref, &paths).unwrap(),
        snap.hashes,
        "the restored state hashes as the snapshot did"
    );
}

/// Taking a snapshot writes objects only.
pub fn check_snapshot_touches_neither_index_nor_worktree(backend: &dyn GitBackend) {
    let (repo, paths, _) = snapshot_fixture();
    let repo_ref = repo.repo_ref();
    let status = repo.git(&["status", "--porcelain=v2"]);
    let index = repo.git(&["ls-files", "-s"]);
    let tree = repo.git(&["write-tree"]);
    let snap = backend.snapshot_create(&repo_ref, &paths).unwrap();
    assert_eq!(repo.git(&["status", "--porcelain=v2"]), status);
    assert_eq!(repo.git(&["ls-files", "-s"]), index);
    assert_eq!(repo.git(&["write-tree"]), tree);
    // The tree exists and is the snapshot's own.
    assert_eq!(
        repo.git(&["cat-file", "-t", &snap.tree.to_hex()]).trim(),
        "tree"
    );
    assert!(snap
        .hashes
        .iter()
        .any(|h| h.path == "new/dir/blob.bin" && h.index.is_none() && h.worktree.is_some()));
    // A second snapshot of the same state is the same tree.
    assert_eq!(
        backend.snapshot_create(&repo_ref, &paths).unwrap().tree,
        snap.tree
    );
}

/// A path the snapshot does not hold is removed from the index and the
/// disk on restore (undoing "created and staged" state).
pub fn check_snapshot_restore_removes_paths_absent_in_the_snapshot(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new().with_initial().build();
    let repo_ref = repo.repo_ref();
    let paths = vec!["ghost.txt".to_string(), "ghosts".to_string()];
    let snap = backend.snapshot_create(&repo_ref, &paths).unwrap();
    assert!(snap.hashes.is_empty());

    std::fs::write(repo.root().join("ghost.txt"), "boo\n").unwrap();
    std::fs::create_dir_all(repo.root().join("ghosts")).unwrap();
    std::fs::write(repo.root().join("ghosts/one.txt"), "1\n").unwrap();
    repo.git(&["add", "ghost.txt"]);

    backend
        .snapshot_restore(&repo_ref, snap.tree, &paths)
        .unwrap();
    assert!(!repo.root().join("ghost.txt").exists());
    assert!(!repo.root().join("ghosts/one.txt").exists());
    assert_eq!(repo.git(&["ls-files", "--", "ghost.txt"]).trim(), "");
}

/// The index side and the working-tree side are restored independently.
pub fn check_snapshot_index_and_worktree_sides_restore_independently(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .commit_file("a.txt", "base\n", "first")
        .build();
    let repo_ref = repo.repo_ref();
    std::fs::write(repo.root().join("a.txt"), "staged\n").unwrap();
    repo.git(&["add", "a.txt"]);
    std::fs::write(repo.root().join("a.txt"), "worktree\n").unwrap();
    let paths = vec!["a.txt".to_string()];
    let snap = backend.snapshot_create(&repo_ref, &paths).unwrap();

    // Both sides change differently.
    repo.git(&["checkout", "HEAD", "--", "a.txt"]);
    std::fs::write(repo.root().join("a.txt"), "other\n").unwrap();
    backend
        .snapshot_restore(&repo_ref, snap.tree, &paths)
        .unwrap();
    assert_eq!(repo.git(&["show", ":a.txt"]), "staged");
    assert_eq!(
        std::fs::read_to_string(repo.root().join("a.txt")).unwrap(),
        "worktree\n"
    );
}

/// `path_hashes` is what `snapshot_create` recorded, and notices an edit
/// on either side.
pub fn check_path_hashes_match_the_snapshot_and_notice_changes(backend: &dyn GitBackend) {
    let (repo, paths, _) = snapshot_fixture();
    let repo_ref = repo.repo_ref();
    let snap = backend.snapshot_create(&repo_ref, &paths).unwrap();
    let now = backend.path_hashes(&repo_ref, &paths).unwrap();
    assert_eq!(now, snap.hashes);
    assert!(
        now.windows(2).all(|w| w[0].path < w[1].path),
        "sorted by path"
    );

    std::fs::write(repo.root().join("crlf.txt"), b"a\nb\n").unwrap();
    assert_ne!(backend.path_hashes(&repo_ref, &paths).unwrap(), now);
    std::fs::write(repo.root().join("crlf.txt"), b"a\r\nb\r\n").unwrap();
    assert_eq!(backend.path_hashes(&repo_ref, &paths).unwrap(), now);
    repo.git(&["add", "crlf.txt"]);
    assert_ne!(
        backend.path_hashes(&repo_ref, &paths).unwrap(),
        now,
        "staging changes the index side"
    );
    // Paths that do not exist hash to nothing; nothing outside the repository.
    assert!(backend
        .path_hashes(&repo_ref, &["nope.txt".to_string()])
        .unwrap()
        .is_empty());
    assert!(backend
        .path_hashes(&repo_ref, &["../outside".to_string()])
        .is_err());
}

/// `delete_files` removes from disk only; `patch_files` names every file
/// a patch touches without applying it.
pub fn check_delete_files_and_patch_files(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .commit_file("keep.txt", "keep\n", "first")
        .commit_file("gone.txt", "gone\n", "second")
        .commit_file("old-name.txt", "renamed\nbody\nmore\nlines\n", "third")
        .build();
    let repo_ref = repo.repo_ref();

    // A patch that edits one file, adds one and renames one.
    std::fs::write(repo.root().join("keep.txt"), "keep\nplus\n").unwrap();
    std::fs::write(repo.root().join("added.txt"), "new\n").unwrap();
    repo.git(&["mv", "old-name.txt", "new-name.txt"]);
    repo.git(&["add", "-A"]);
    let patch = repo.git(&["diff", "--cached", "-M", "--binary"]);
    repo.git(&["reset", "-q", "--hard"]);
    let patch_path = repo.root().join(".git").join("contract.patch");
    std::fs::write(&patch_path, format!("{patch}\n")).unwrap();
    let mut files = backend.patch_files(&repo_ref, &patch_path).unwrap();
    files.sort();
    assert_eq!(
        files,
        vec!["added.txt", "keep.txt", "new-name.txt", "old-name.txt"]
    );
    assert_eq!(
        repo.git(&["status", "--porcelain"]).trim(),
        "",
        "not applied"
    );
    let _ = std::fs::remove_file(&patch_path);

    // Delete a tracked file and an untracked directory.
    std::fs::create_dir_all(repo.root().join("junk/inner")).unwrap();
    std::fs::write(repo.root().join("junk/inner/x"), "x").unwrap();
    backend
        .delete_files(
            &repo_ref,
            &[
                "gone.txt".to_string(),
                "junk".to_string(),
                "never-existed".to_string(),
            ],
        )
        .unwrap();
    assert!(!repo.root().join("gone.txt").exists());
    assert!(!repo.root().join("junk").exists());
    assert_eq!(
        repo.git(&["status", "--porcelain"]).trim(),
        "D gone.txt",
        "the index is untouched: a tracked file is a deleted change"
    );
    assert!(backend
        .delete_files(&repo_ref, &[".git/config".to_string()])
        .is_err());
    assert!(repo.root().join(".git/config").exists());
}

/// `convert_files` rewrites text files in place (encoding, line breaks,
/// indentation), reports what it left alone, never touches the index, and
/// refuses paths outside the repository or inside `.git`.
pub fn check_convert_files(backend: &dyn GitBackend) {
    use gitpoulpi_core::textconv::{Charset, ConvertOp, ConvertSkip, Eol};
    let repo = TestRepoBuilder::new()
        .commit_file("crlf.txt", "a\nb\n", "first")
        .commit_file("latin.txt", "x\n", "second")
        .commit_file("tabs.txt", "x\n", "third")
        .commit_file("bin.dat", "x\n", "fourth")
        .build();
    let repo_ref = repo.repo_ref();
    let root = repo.root();
    std::fs::write(root.join("crlf.txt"), b"a\r\nb\r\n").unwrap();
    std::fs::write(root.join("latin.txt"), b"caf\xe9\n").unwrap();
    std::fs::write(root.join("tabs.txt"), b"\tx\n\t\ty\n").unwrap();
    std::fs::write(root.join("bin.dat"), b"a\0b\r\n").unwrap();
    let index_before = repo.git(&["ls-files", "-s"]);
    let all: Vec<String> = ["crlf.txt", "latin.txt", "tabs.txt", "bin.dat", "gone.txt"]
        .iter()
        .map(|s| s.to_string())
        .collect();

    let report = backend
        .convert_files(&repo_ref, &all, ConvertOp::Eol(Eol::Lf))
        .unwrap();
    assert_eq!(report.converted, vec!["crlf.txt"]);
    assert_eq!(report.unchanged, vec!["latin.txt", "tabs.txt"]);
    assert_eq!(
        report.skipped,
        vec![
            ("bin.dat".to_string(), ConvertSkip::Binary),
            ("gone.txt".to_string(), ConvertSkip::Missing)
        ]
    );
    assert_eq!(std::fs::read(root.join("crlf.txt")).unwrap(), b"a\nb\n");
    assert_eq!(std::fs::read(root.join("bin.dat")).unwrap(), b"a\0b\r\n");

    let report = backend
        .convert_files(&repo_ref, &all, ConvertOp::Encoding(Charset::Utf8))
        .unwrap();
    assert_eq!(report.converted, vec!["latin.txt"]);
    assert_eq!(
        std::fs::read(root.join("latin.txt")).unwrap(),
        "caf\u{e9}\n".as_bytes()
    );
    // And back: Windows-1252 holds it, Shift_JIS-less ASCII-only targets do not.
    let report = backend
        .convert_files(
            &repo_ref,
            &["latin.txt".to_string()],
            ConvertOp::Encoding(Charset::Windows1252),
        )
        .unwrap();
    assert_eq!(report.converted, vec!["latin.txt"]);
    std::fs::write(root.join("latin.txt"), "\u{4e2d}\n").unwrap();
    let report = backend
        .convert_files(
            &repo_ref,
            &["latin.txt".to_string()],
            ConvertOp::Encoding(Charset::Windows1252),
        )
        .unwrap();
    assert!(matches!(
        report.skipped[0].1,
        ConvertSkip::Unencodable { count: 1, .. }
    ));
    assert_eq!(
        std::fs::read(root.join("latin.txt")).unwrap(),
        "\u{4e2d}\n".as_bytes(),
        "an unencodable file is left as it was"
    );

    let report = backend
        .convert_files(
            &repo_ref,
            &["tabs.txt".to_string()],
            ConvertOp::Indent {
                tabs: false,
                size: 4,
            },
        )
        .unwrap();
    assert_eq!(report.converted, vec!["tabs.txt"]);
    assert_eq!(
        std::fs::read(root.join("tabs.txt")).unwrap(),
        b"    x\n        y\n"
    );

    assert_eq!(
        repo.git(&["ls-files", "-s"]),
        index_before,
        "index untouched"
    );
    for bad in [".git/config", "../outside"] {
        assert!(backend
            .convert_files(&repo_ref, &[bad.to_string()], ConvertOp::Eol(Eol::Lf))
            .is_err());
    }
}

/// Removing a worktree: a clean one goes; the main one, an unknown path
/// and a dirty one (no way to save the changes) are refused and leave
/// everything where it was; `force` discards the changes.
pub fn check_worktree_remove_clean_dirty_and_refusals(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .with_initial()
        .branch("feature")
        .checkout("main")
        .build();
    let repo_ref = repo.repo_ref();
    let base = repo.root().parent().unwrap().to_path_buf();
    let add = |name: &str, branch: &str| {
        let path = base.join(name);
        backend
            .worktree_add(
                &repo_ref,
                &path,
                &WorktreeAddOptions {
                    start: WorktreeStart::Branch(branch.into()),
                    new_branch: None,
                },
            )
            .unwrap();
        path
    };
    let clean = add("wt-clean", "feature");
    assert!(backend
        .worktree_changes(&repo_ref, &clean)
        .unwrap()
        .is_empty());
    let removed = backend
        .worktree_remove(&repo_ref, &clean, &Default::default())
        .unwrap();
    assert_eq!(removed.commit, None);
    assert!(!removed.stashed);
    assert!(!clean.exists());
    assert_eq!(backend.worktrees(&repo_ref).unwrap().len(), 1);
    assert_eq!(
        rev(&repo, "feature"),
        rev(&repo, "main"),
        "the branch stays"
    );

    // The main worktree and a stranger are refused.
    assert!(backend
        .worktree_remove(&repo_ref, repo.root(), &Default::default())
        .is_err());
    assert!(backend
        .worktree_remove(&repo_ref, &base.join("nope"), &Default::default())
        .is_err());
    assert!(repo.root().exists());

    // Dirty: a tracked edit and an untracked file.
    let dirty = add("wt-dirty", "feature");
    std::fs::write(dirty.join("initial.txt"), "edited\n").unwrap();
    std::fs::write(dirty.join("new.txt"), "new\n").unwrap();
    let changes = backend.worktree_changes(&repo_ref, &dirty).unwrap();
    let names: Vec<(&str, bool)> = changes
        .iter()
        .map(|c| (c.path.as_str(), c.untracked))
        .collect();
    assert_eq!(names, vec![("initial.txt", false), ("new.txt", true)]);
    let refused = backend.worktree_remove(&repo_ref, &dirty, &Default::default());
    assert!(
        matches!(refused, Err(gitpoulpi_core::Error::DirtyWorkingTree { .. })),
        "{refused:?}"
    );
    assert!(dirty.join("new.txt").exists(), "a refusal changes nothing");

    // Force discards them.
    backend
        .worktree_remove(
            &repo_ref,
            &dirty,
            &WorktreeRemoveOptions {
                force: true,
                ..Default::default()
            },
        )
        .unwrap();
    assert!(!dirty.exists());
    assert!(backend.stashes(&repo_ref).unwrap().is_empty());
}

/// Commit and remove: every change (untracked too) lands as one commit on
/// the worktree's branch, visible from the main repository, and the folder
/// is gone. A detached worktree needs a branch name for it.
pub fn check_worktree_remove_commits_first(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .with_initial()
        .branch("feature")
        .checkout("main")
        .build();
    let repo_ref = repo.repo_ref();
    let base = repo.root().parent().unwrap().to_path_buf();
    let path = base.join("wt-commit");
    backend
        .worktree_add(
            &repo_ref,
            &path,
            &WorktreeAddOptions {
                start: WorktreeStart::Branch("feature".into()),
                new_branch: None,
            },
        )
        .unwrap();
    std::fs::write(path.join("initial.txt"), "edited\n").unwrap();
    std::fs::write(path.join("new.txt"), "new\n").unwrap();
    let removed = backend
        .worktree_remove(
            &repo_ref,
            &path,
            &WorktreeRemoveOptions {
                commit: Some(WorktreeCommit {
                    message: "save work".into(),
                    branch: None,
                }),
                ..Default::default()
            },
        )
        .unwrap();
    assert!(!path.exists());
    let tip = rev(&repo, "feature");
    assert_eq!(removed.commit, Some(tip));
    assert_eq!(
        repo.git(&["log", "-1", "--format=%s", "feature"]),
        "save work"
    );
    assert_eq!(repo.git(&["show", "feature:new.txt"]), "new");

    // Detached: no branch to commit on.
    let detached = base.join("wt-detached");
    backend
        .worktree_add(
            &repo_ref,
            &detached,
            &WorktreeAddOptions {
                start: WorktreeStart::Commit(rev(&repo, "main")),
                new_branch: None,
            },
        )
        .unwrap();
    std::fs::write(detached.join("d.txt"), "d\n").unwrap();
    let commit = |branch: Option<&str>| WorktreeRemoveOptions {
        commit: Some(WorktreeCommit {
            message: "d".into(),
            branch: branch.map(str::to_string),
        }),
        ..Default::default()
    };
    assert!(backend
        .worktree_remove(&repo_ref, &detached, &commit(None))
        .is_err());
    assert!(detached.join("d.txt").exists(), "refused before any change");
    backend
        .worktree_remove(&repo_ref, &detached, &commit(Some("rescued")))
        .unwrap();
    assert!(!detached.exists());
    assert_eq!(repo.git(&["show", "rescued:d.txt"]), "d");
}

/// Stash and remove: the stash (untracked file included) stays in the
/// shared stash list, the folder is gone.
pub fn check_worktree_remove_stashes_first(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .with_initial()
        .branch("feature")
        .checkout("main")
        .build();
    let repo_ref = repo.repo_ref();
    let path = repo.root().parent().unwrap().join("wt-stash");
    backend
        .worktree_add(
            &repo_ref,
            &path,
            &WorktreeAddOptions {
                start: WorktreeStart::Branch("feature".into()),
                new_branch: None,
            },
        )
        .unwrap();
    std::fs::write(path.join("initial.txt"), "edited\n").unwrap();
    std::fs::write(path.join("untracked.txt"), "u\n").unwrap();
    let removed = backend
        .worktree_remove(
            &repo_ref,
            &path,
            &WorktreeRemoveOptions {
                stash: Some("WIP on wt-stash".into()),
                ..Default::default()
            },
        )
        .unwrap();
    assert!(removed.stashed);
    assert!(!path.exists());
    let stashes = backend.stashes(&repo_ref).unwrap();
    assert_eq!(stashes.len(), 1);
    assert!(
        stashes[0].message.contains("WIP on wt-stash"),
        "{stashes:?}"
    );
    assert_eq!(
        repo.git(&["ls-tree", "-r", "--name-only", "stash@{0}^3"]),
        "untracked.txt"
    );
}
