use super::{head_oid, pick, rev};
use crate::TestRepoBuilder;
use gitpoulpi_core::domain::{
    CheckoutTarget, CherryPickOptions, CherryPickOutcome, CommitOptions, ConflictResolution,
    ConflictSide, FastForwardOutcome, InteractiveRebase, LogOptions, MergeOptions, MergeOutcome,
    Oid, RebaseAction, RebaseOptions, RebaseOutcome, RebaseRefusal, RebaseStep, RefKind, ResetMode,
    RevertOutcome, RewordRefusal, StageTarget,
};
use gitpoulpi_core::GitBackend;

pub fn check_reset_hard(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new().with_initial().build();
    let repo_ref = repo.repo_ref();
    let first = head_oid(backend, &repo_ref);
    std::fs::write(repo.root().join("new.txt"), "hello").unwrap();
    backend
        .stage(&repo_ref, &StageTarget::File("new.txt".into()))
        .unwrap();
    backend
        .commit(&repo_ref, "second", &CommitOptions::default())
        .unwrap();

    backend.reset(&repo_ref, first, ResetMode::Hard).unwrap();

    let log = backend.log(&repo_ref, &LogOptions::default()).unwrap();
    assert_eq!(log.nodes.len(), 1);
    assert!(!repo.root().join("new.txt").exists());
}

pub fn check_merge_fast_forward(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new().with_initial().build();
    let repo_ref = repo.repo_ref();
    let base = head_oid(backend, &repo_ref);

    backend.branch_create(&repo_ref, "feature", base).unwrap();
    backend
        .checkout(&repo_ref, &CheckoutTarget::Branch("feature".into()))
        .unwrap();
    std::fs::write(repo.root().join("feature.txt"), "x").unwrap();
    backend
        .stage(&repo_ref, &StageTarget::File("feature.txt".into()))
        .unwrap();
    let tip = backend
        .commit(&repo_ref, "feature work", &CommitOptions::default())
        .unwrap();
    backend
        .checkout(&repo_ref, &CheckoutTarget::Branch("main".into()))
        .unwrap();

    let feature_ref = backend
        .refs(&repo_ref)
        .unwrap()
        .into_iter()
        .find(|r| matches!(&r.kind, RefKind::LocalBranch { name } if name == "feature"))
        .expect("feature branch should exist");

    let outcome = backend
        .merge(&repo_ref, &feature_ref, &MergeOptions::default())
        .unwrap();
    assert_eq!(outcome, MergeOutcome::FastForwarded(tip));
    assert_eq!(head_oid(backend, &repo_ref), tip);
}

// -- partial discard and line-level (un)staging ---------

pub fn check_revert_creates_an_inverse_commit(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new().with_initial().build();
    let repo_ref = repo.repo_ref();

    std::fs::write(repo.root().join("doomed.txt"), "doomed").unwrap();
    backend
        .stage(&repo_ref, &StageTarget::File("doomed.txt".into()))
        .unwrap();
    let doomed = backend
        .commit(&repo_ref, "add doomed.txt", &CommitOptions::default())
        .unwrap();

    let outcome = backend.revert(&repo_ref, doomed, None).unwrap();
    let RevertOutcome::Completed(revert_commit) = outcome else {
        panic!("a clean revert must complete, got {outcome:?}");
    };

    assert_eq!(head_oid(backend, &repo_ref), revert_commit);
    assert!(
        !repo.root().join("doomed.txt").exists(),
        "reverting the commit that added the file must remove it again"
    );
}

pub fn check_cherry_pick_copies_a_commit_onto_head(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new().with_initial().build();
    let repo_ref = repo.repo_ref();
    let base = head_oid(backend, &repo_ref);

    backend.branch_create(&repo_ref, "feature", base).unwrap();
    backend
        .checkout(&repo_ref, &CheckoutTarget::Branch("feature".into()))
        .unwrap();
    std::fs::write(repo.root().join("picked.txt"), "picked").unwrap();
    backend
        .stage(&repo_ref, &StageTarget::File("picked.txt".into()))
        .unwrap();
    let source = backend
        .commit(&repo_ref, "work to pick", &CommitOptions::default())
        .unwrap();

    backend
        .checkout(&repo_ref, &CheckoutTarget::Branch("main".into()))
        .unwrap();
    // main has to move before the pick, or the copy would land on the
    // very same parent with the very same tree and author -- git would
    // then hand back a commit with the *same* oid as the source, and the
    // "it really copied" assertion below would be meaningless.
    std::fs::write(repo.root().join("on-main.txt"), "main").unwrap();
    backend
        .stage(&repo_ref, &StageTarget::File("on-main.txt".into()))
        .unwrap();
    let main_tip = backend
        .commit(&repo_ref, "main moves on", &CommitOptions::default())
        .unwrap();

    let outcome = backend
        .cherry_pick(&repo_ref, &[source], &CherryPickOptions::default())
        .unwrap();

    let CherryPickOutcome::Completed(applied) = outcome else {
        panic!("a conflict-free cherry-pick must complete, got {outcome:?}");
    };
    assert_eq!(applied.len(), 1);
    // A cherry-pick *copies*: the new commit is a different object on a
    // different parent, so asserting `applied[0] == source` would be wrong.
    assert_ne!(applied[0], source);
    assert_eq!(head_oid(backend, &repo_ref), applied[0]);
    let log = backend.log(&repo_ref, &LogOptions::default()).unwrap();
    assert_eq!(log.nodes[0].parents, vec![main_tip]);
    assert!(repo.root().join("picked.txt").exists());
}

/// Two branches that both rewrite `c.txt`: returns the fixture, with
/// `main` checked out holding "ours" and `feature` holding "theirs"
/// (author "Other <other@example.com>").
fn diverged_on_c_txt() -> crate::fixture::FixtureRepo {
    let repo = TestRepoBuilder::new().with_initial().build();
    let commit_c = |text: &str, msg: &str, author: &str| {
        std::fs::write(repo.root().join("c.txt"), text).unwrap();
        repo.git(&["add", "c.txt"]);
        repo.git(&["commit", "-q", "--author", author, "-m", msg]);
    };
    commit_c("base", "base", "Base <base@example.com>");
    repo.git(&["branch", "feature"]);
    repo.git(&["checkout", "-q", "feature"]);
    commit_c("theirs", "theirs", "Other <other@example.com>");
    repo.git(&["checkout", "-q", "main"]);
    commit_c("ours", "ours", "Base <base@example.com>");
    repo
}

fn resolve_and_stage_c_txt(backend: &dyn GitBackend, repo: &crate::fixture::FixtureRepo) {
    std::fs::write(repo.root().join("c.txt"), "resolved").unwrap();
    backend
        .stage(&repo.repo_ref(), &StageTarget::File("c.txt".into()))
        .unwrap();
}

fn assert_no_sequencer_state(repo: &crate::fixture::FixtureRepo) {
    for f in ["MERGE_HEAD", "MERGE_MSG", "CHERRY_PICK_HEAD", "REVERT_HEAD"] {
        assert!(
            !repo.root().join(".git").join(f).exists(),
            "{f} must be cleaned up by the concluding commit"
        );
    }
    assert!(repo.git(&["status", "--porcelain"]).trim().is_empty());
}

pub fn check_commit_concludes_a_conflicted_merge(backend: &dyn GitBackend) {
    let repo = diverged_on_c_txt();
    let repo_ref = repo.repo_ref();
    let ours = head_oid(backend, &repo_ref);
    let theirs = backend.resolve(&repo_ref, "feature").unwrap();
    let feature = backend
        .refs(&repo_ref)
        .unwrap()
        .into_iter()
        .find(|r| matches!(&r.kind, RefKind::LocalBranch { name } if name == "feature"))
        .expect("feature ref");
    let outcome = backend
        .merge(&repo_ref, &feature, &MergeOptions::default())
        .unwrap();
    assert!(
        matches!(outcome, MergeOutcome::Conflicted(_)),
        "{outcome:?}"
    );
    resolve_and_stage_c_txt(backend, &repo);

    let merged = backend
        .commit(&repo_ref, "merge feature", &CommitOptions::default())
        .unwrap();
    let log = backend.log(&repo_ref, &LogOptions::default()).unwrap();
    assert_eq!(log.nodes[0].id, merged);
    assert_eq!(
        log.nodes[0].parents,
        vec![ours, theirs],
        "the concluding commit of a merge has both parents"
    );
    assert_no_sequencer_state(&repo);
}

pub fn check_commit_concludes_a_conflicted_cherry_pick(backend: &dyn GitBackend) {
    let repo = diverged_on_c_txt();
    let repo_ref = repo.repo_ref();
    let ours = head_oid(backend, &repo_ref);
    let source = backend.resolve(&repo_ref, "feature").unwrap();
    let outcome = backend
        .cherry_pick(&repo_ref, &[source], &CherryPickOptions::default())
        .unwrap();
    assert!(
        matches!(outcome, CherryPickOutcome::Conflicted { .. }),
        "{outcome:?}"
    );
    resolve_and_stage_c_txt(backend, &repo);

    let picked = backend
        .commit(&repo_ref, "theirs", &CommitOptions::default())
        .unwrap();
    let log = backend.log(&repo_ref, &LogOptions::default()).unwrap();
    assert_eq!(log.nodes[0].id, picked);
    assert_eq!(log.nodes[0].parents, vec![ours]);
    assert_eq!(
        repo.git(&["log", "-1", "--format=%an <%ae>"]).trim(),
        "Other <other@example.com>",
        "the picked commit's authorship is kept"
    );
    assert_no_sequencer_state(&repo);
}

pub fn check_commit_concludes_a_conflicted_revert(backend: &dyn GitBackend) {
    let repo = diverged_on_c_txt();
    let repo_ref = repo.repo_ref();
    // Reverting "base" on top of "ours" conflicts on c.txt.
    let base = backend.resolve(&repo_ref, "HEAD~1").unwrap();
    let ours = head_oid(backend, &repo_ref);
    let outcome = backend.revert(&repo_ref, base, None).unwrap();
    assert!(matches!(outcome, RevertOutcome::Conflicted), "{outcome:?}");
    resolve_and_stage_c_txt(backend, &repo);

    let reverted = backend
        .commit(&repo_ref, "Revert \"base\"", &CommitOptions::default())
        .unwrap();
    let log = backend.log(&repo_ref, &LogOptions::default()).unwrap();
    assert_eq!(log.nodes[0].id, reverted);
    assert_eq!(log.nodes[0].parents, vec![ours]);
    assert_no_sequencer_state(&repo);
}

pub fn check_rebase_replays_onto_the_target(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new().with_initial().build();
    let repo_ref = repo.repo_ref();
    let base = head_oid(backend, &repo_ref);

    // main moves on...
    std::fs::write(repo.root().join("on-main.txt"), "main").unwrap();
    backend
        .stage(&repo_ref, &StageTarget::File("on-main.txt".into()))
        .unwrap();
    let main_tip = backend
        .commit(&repo_ref, "main moves on", &CommitOptions::default())
        .unwrap();

    // ...while feature, branched at `base`, does its own work.
    backend.branch_create(&repo_ref, "feature", base).unwrap();
    backend
        .checkout(&repo_ref, &CheckoutTarget::Branch("feature".into()))
        .unwrap();
    std::fs::write(repo.root().join("on-feature.txt"), "feature").unwrap();
    backend
        .stage(&repo_ref, &StageTarget::File("on-feature.txt".into()))
        .unwrap();
    backend
        .commit(&repo_ref, "feature work", &CommitOptions::default())
        .unwrap();

    let main_ref = backend
        .refs(&repo_ref)
        .unwrap()
        .into_iter()
        .find(|r| matches!(&r.kind, RefKind::LocalBranch { name } if name == "main"))
        .expect("main should exist");

    let outcome = backend
        .rebase(&repo_ref, &main_ref, &RebaseOptions::default())
        .unwrap();
    assert_eq!(outcome, RebaseOutcome::Completed);

    let log = backend.log(&repo_ref, &LogOptions::default()).unwrap();
    assert_eq!(log.nodes[0].summary, "feature work");
    assert_eq!(
        log.nodes[0].parents,
        vec![main_tip],
        "the replayed commit must sit directly on main's tip"
    );
    // Both files are present only if the replay really happened on top of
    // main rather than the branch just being reported as rebased.
    assert!(repo.root().join("on-main.txt").exists());
    assert!(repo.root().join("on-feature.txt").exists());
}

// -- interactive rebase ----------------------------------------------

fn step(commit: Oid, action: RebaseAction) -> RebaseStep {
    RebaseStep {
        commit,
        action,
        message: None,
    }
}

fn reword_step(commit: Oid, message: &str) -> RebaseStep {
    RebaseStep {
        commit,
        action: RebaseAction::Reword,
        message: Some(message.to_string()),
    }
}

fn rebase_refused(
    result: gitpoulpi_core::Result<RebaseOutcome>,
) -> gitpoulpi_core::domain::RebaseRefusal {
    match result {
        Err(gitpoulpi_core::Error::RebaseRefused(r)) => r,
        other => panic!("expected a typed rebase refusal, got {other:?}"),
    }
}

pub fn check_rebase_plan_lists_the_range_oldest_first(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .commit("one")
        .commit("two")
        .commit("three")
        .build();
    let repo_ref = repo.repo_ref();
    let log = backend.log(&repo_ref, &LogOptions::default()).unwrap();
    let one = log.nodes[2].id;

    let draft = backend.rebase_plan(&repo_ref, Some(one)).unwrap();
    assert_eq!(draft.upstream, Some(one));
    assert_eq!(
        draft
            .commits
            .iter()
            .map(|c| c.summary.as_str())
            .collect::<Vec<_>>(),
        vec!["two", "three"]
    );

    let root_draft = backend.rebase_plan(&repo_ref, None).unwrap();
    assert_eq!(
        root_draft
            .commits
            .iter()
            .map(|c| c.summary.as_str())
            .collect::<Vec<_>>(),
        vec!["one", "two", "three"]
    );
}

pub fn check_rebase_plan_refuses_a_merge_in_range(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .commit("base")
        .branch("feature")
        .commit("on-feature")
        .checkout("main")
        .build();
    let repo_ref = repo.repo_ref();
    let base = backend.resolve(&repo_ref, "HEAD").unwrap();
    repo.git(&["merge", "--no-ff", "-m", "merge feature", "feature"]);

    let outcome = backend.rebase_plan(&repo_ref, Some(base));
    assert!(matches!(
        outcome,
        Err(gitpoulpi_core::Error::RebaseRefused(
            RebaseRefusal::MergeInRange
        ))
    ));
}

pub fn check_rebase_interactive_reorders_and_drops(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .commit("base")
        .commit("add-a")
        .commit("add-b")
        .commit("add-c")
        .build();
    let repo_ref = repo.repo_ref();
    let log = backend.log(&repo_ref, &LogOptions::default()).unwrap();
    let base = log.nodes[3].id;
    let a = log.nodes[2].id;
    let b = log.nodes[1].id;
    let c = log.nodes[0].id;

    // Reorder to c, a and drop b entirely.
    let outcome = backend
        .rebase_interactive(
            &repo_ref,
            &InteractiveRebase {
                upstream: Some(base),
                onto: None,
                steps: vec![
                    step(c, RebaseAction::Pick),
                    step(a, RebaseAction::Pick),
                    step(b, RebaseAction::Drop),
                ],
                autosquash: false,
                cherry_pick: false,
            },
        )
        .unwrap();
    assert_eq!(outcome, RebaseOutcome::Completed);

    let log = backend.log(&repo_ref, &LogOptions::default()).unwrap();
    assert_eq!(
        log.nodes
            .iter()
            .map(|n| n.summary.as_str())
            .collect::<Vec<_>>(),
        vec!["add-a", "add-c", "base"]
    );
    assert!(repo.root().join("add-a.txt").exists());
    assert!(!repo.root().join("add-b.txt").exists());
    assert!(repo.root().join("add-c.txt").exists());
}

pub fn check_rebase_interactive_squash_concatenates_and_fixup_keeps_first_message(
    backend: &dyn GitBackend,
) {
    let repo = TestRepoBuilder::new()
        .commit("base")
        .commit("first")
        .commit("second")
        .commit("third")
        .build();
    let repo_ref = repo.repo_ref();
    let log = backend.log(&repo_ref, &LogOptions::default()).unwrap();
    let base = log.nodes[3].id;
    let first = log.nodes[2].id;
    let second = log.nodes[1].id;
    let third = log.nodes[0].id;

    let outcome = backend
        .rebase_interactive(
            &repo_ref,
            &InteractiveRebase {
                upstream: Some(base),
                onto: None,
                steps: vec![
                    pick(first),
                    step(second, RebaseAction::Squash),
                    step(third, RebaseAction::Fixup),
                ],
                autosquash: false,
                cherry_pick: false,
            },
        )
        .unwrap();
    assert_eq!(outcome, RebaseOutcome::Completed);

    let log = backend.log(&repo_ref, &LogOptions::default()).unwrap();
    assert_eq!(log.nodes.len(), 2, "base + one combined commit");
    assert!(log.nodes[0].message.contains("first"));
    assert!(log.nodes[0].message.contains("second"));
    assert!(
        !log.nodes[0].message.contains("third"),
        "fixup discards its own message: {}",
        log.nodes[0].message
    );
    for f in ["first.txt", "second.txt", "third.txt"] {
        assert!(repo.root().join(f).exists(), "{f} must still be present");
    }
}

pub fn check_rebase_interactive_reword_normalizes_crlf_and_keeps_hash_lines(
    backend: &dyn GitBackend,
) {
    let repo = TestRepoBuilder::new()
        .commit("base")
        .commit("to reword")
        .build();
    let repo_ref = repo.repo_ref();
    let head = head_oid(backend, &repo_ref);

    let outcome = backend
        .rebase_interactive(
            &repo_ref,
            &InteractiveRebase {
                upstream: Some(backend.resolve(&repo_ref, "HEAD~1").unwrap()),
                onto: None,
                steps: vec![reword_step(head, "a\r\n\r\n# b\r\n")],
                autosquash: false,
                cherry_pick: false,
            },
        )
        .unwrap();
    assert_eq!(outcome, RebaseOutcome::Completed);

    let log = backend.log(&repo_ref, &LogOptions::default()).unwrap();
    // `log()` reports a message with no trailing newline (see
    // `check_commit_message_body`); the byte git actually stored still
    // ends in exactly one, which `check_rebase_interactive_...` cannot
    // observe through this trait alone.
    assert_eq!(log.nodes[0].message, "a\n\n# b");
}

/// A `Reword` step followed by `Squash`/`Fixup` steps: the reword
/// message is the whole final message (the caller has already combined
/// the squashed messages into it), so the folded commits' own messages
/// are not appended a second time.
pub fn check_rebase_interactive_reword_receiving_squash_keeps_exactly_its_message(
    backend: &dyn GitBackend,
) {
    let repo = TestRepoBuilder::new()
        .commit("base")
        .commit("first")
        .commit("second")
        .commit("third")
        .commit("fourth")
        .build();
    let repo_ref = repo.repo_ref();
    let log = backend.log(&repo_ref, &LogOptions::default()).unwrap();
    let base = log.nodes[4].id;
    let first = log.nodes[3].id;
    let second = log.nodes[2].id;
    let third = log.nodes[1].id;
    let fourth = log.nodes[0].id;

    let outcome = backend
        .rebase_interactive(
            &repo_ref,
            &InteractiveRebase {
                upstream: Some(base),
                onto: None,
                steps: vec![
                    reword_step(first, "combined\n\nfirst\n\nsecond"),
                    step(second, RebaseAction::Squash),
                    step(third, RebaseAction::Fixup),
                    pick(fourth),
                ],
                autosquash: false,
                cherry_pick: false,
            },
        )
        .unwrap();
    assert_eq!(outcome, RebaseOutcome::Completed);

    let log = backend.log(&repo_ref, &LogOptions::default()).unwrap();
    assert_eq!(log.nodes.len(), 3, "base + combined + fourth");
    assert_eq!(log.nodes[0].message, "fourth");
    assert_eq!(log.nodes[1].message, "combined\n\nfirst\n\nsecond");
    for f in ["first.txt", "second.txt", "third.txt", "fourth.txt"] {
        assert!(repo.root().join(f).exists(), "{f} must still be present");
    }
    assert!(backend.rebase_state(&repo_ref).unwrap().is_none());
}

/// Same rule in a cherry-pick plan.
pub fn check_cherry_pick_plan_reword_receiving_squash_keeps_exactly_its_message(
    backend: &dyn GitBackend,
) {
    let (repo, [c1, c2, c3]) = cherry_pick_plan_fixture();
    let repo_ref = repo.repo_ref();

    let outcome = backend
        .rebase_interactive(
            &repo_ref,
            &InteractiveRebase {
                upstream: None,
                onto: None,
                steps: vec![
                    reword_step(c1, "c1 and c2\n\ndetails of both"),
                    step(c2, RebaseAction::Squash),
                    pick(c3),
                ],
                autosquash: false,
                cherry_pick: true,
            },
        )
        .unwrap();
    assert_eq!(outcome, RebaseOutcome::Completed);

    let log = backend.log(&repo_ref, &LogOptions::default()).unwrap();
    assert_eq!(log.nodes[0].summary, "c3");
    assert_eq!(log.nodes[1].message, "c1 and c2\n\ndetails of both");
    for f in ["c1.txt", "c2.txt", "c3.txt", "main.txt"] {
        assert!(repo.root().join(f).exists(), "{f} must be present");
    }
}

pub fn check_rebase_interactive_edit_stops_then_continue_completes(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .commit("base")
        .commit("to edit")
        .commit("after")
        .build();
    let repo_ref = repo.repo_ref();
    let log = backend.log(&repo_ref, &LogOptions::default()).unwrap();
    let base = log.nodes[2].id;
    let to_edit = log.nodes[1].id;
    let after = log.nodes[0].id;

    let outcome = backend
        .rebase_interactive(
            &repo_ref,
            &InteractiveRebase {
                upstream: Some(base),
                onto: None,
                steps: vec![step(to_edit, RebaseAction::Edit), pick(after)],
                autosquash: false,
                cherry_pick: false,
            },
        )
        .unwrap();
    let RebaseOutcome::Stopped { remaining, .. } = outcome else {
        panic!("an edit step must stop the rebase, got {outcome:?}");
    };
    assert_eq!(remaining, vec![after]);

    let state = backend.rebase_state(&repo_ref).unwrap();
    assert!(state.is_some(), "a stopped rebase must report a state");
    assert!(state.unwrap().interactive);

    std::fs::write(repo.root().join("extra.txt"), "extra").unwrap();
    repo.git(&["add", "extra.txt"]);
    repo.git(&["commit", "--amend", "--no-edit"]);

    let outcome = backend.rebase_continue(&repo_ref).unwrap();
    assert_eq!(outcome, RebaseOutcome::Completed);
    assert!(backend.rebase_state(&repo_ref).unwrap().is_none());
}

pub fn check_rebase_interactive_conflict_resolve_then_continue(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .commit_file("f.txt", "line1\n", "base")
        .commit_file("f.txt", "line1\nline2\n", "adds line2")
        .commit_file("f.txt", "line1\nline2\nline3\n", "adds line3")
        .build();
    let repo_ref = repo.repo_ref();
    let log = backend.log(&repo_ref, &LogOptions::default()).unwrap();
    let base = log.nodes[2].id;
    let adds_line2 = log.nodes[1].id;
    let adds_line3 = log.nodes[0].id;

    let outcome = backend
        .rebase_interactive(
            &repo_ref,
            &InteractiveRebase {
                upstream: Some(base),
                onto: None,
                // `adds_line2` is dropped, so `adds_line3`'s patch --
                // which assumes line2 is already there -- is applied
                // directly onto `base` and must conflict. It is also the
                // last step, so nothing is left to apply afterwards.
                steps: vec![step(adds_line2, RebaseAction::Drop), pick(adds_line3)],
                autosquash: false,
                cherry_pick: false,
            },
        )
        .unwrap();
    let RebaseOutcome::Conflicted { remaining, .. } = outcome else {
        panic!("expected a conflict, got {outcome:?}");
    };
    assert_eq!(remaining, Vec::<Oid>::new());

    let state = backend.rebase_state(&repo_ref).unwrap().unwrap();
    assert!(state.conflicted);

    std::fs::write(repo.root().join("f.txt"), "line1\nline2\nline3\n").unwrap();
    repo.git(&["add", "f.txt"]);

    let outcome = backend.rebase_continue(&repo_ref).unwrap();
    assert_eq!(outcome, RebaseOutcome::Completed);
    let log = backend.log(&repo_ref, &LogOptions::default()).unwrap();
    assert_eq!(log.nodes[0].summary, "adds line3");
}

pub fn check_rebase_interactive_conflict_then_skip(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .commit_file("f.txt", "line1\n", "base")
        .commit_file("f.txt", "line1\nline2\n", "adds line2")
        .commit_file("f.txt", "line1\nline2\nline3\n", "adds line3")
        .build();
    let repo_ref = repo.repo_ref();
    let log = backend.log(&repo_ref, &LogOptions::default()).unwrap();
    let base = log.nodes[2].id;
    let adds_line2 = log.nodes[1].id;
    let adds_line3 = log.nodes[0].id;

    let outcome = backend
        .rebase_interactive(
            &repo_ref,
            &InteractiveRebase {
                upstream: Some(base),
                onto: None,
                steps: vec![pick(adds_line3), pick(adds_line2)],
                autosquash: false,
                cherry_pick: false,
            },
        )
        .unwrap();
    assert!(matches!(outcome, RebaseOutcome::Conflicted { .. }));

    let outcome = backend.rebase_skip(&repo_ref).unwrap();
    assert_eq!(outcome, RebaseOutcome::Completed);
    // Skipping the conflicted `adds_line3` step (applied first, since the
    // plan swapped it before `adds_line2`) leaves only `adds_line2`
    // applied.
    let log = backend.log(&repo_ref, &LogOptions::default()).unwrap();
    assert_eq!(log.nodes[0].summary, "adds line2");
}

pub fn check_rebase_abort_restores_branch_and_head(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .commit_file("f.txt", "line1\n", "base")
        .commit_file("f.txt", "line1\nline2\n", "adds line2")
        .commit_file("f.txt", "line1\nline2\nline3\n", "adds line3")
        .build();
    let repo_ref = repo.repo_ref();
    let log = backend.log(&repo_ref, &LogOptions::default()).unwrap();
    let base = log.nodes[2].id;
    let adds_line2 = log.nodes[1].id;
    let adds_line3 = log.nodes[0].id;
    let original_head = head_oid(backend, &repo_ref);

    let outcome = backend
        .rebase_interactive(
            &repo_ref,
            &InteractiveRebase {
                upstream: Some(base),
                onto: None,
                steps: vec![pick(adds_line3), pick(adds_line2)],
                autosquash: false,
                cherry_pick: false,
            },
        )
        .unwrap();
    assert!(matches!(outcome, RebaseOutcome::Conflicted { .. }));

    backend.rebase_abort(&repo_ref).unwrap();
    assert_eq!(head_oid(backend, &repo_ref), original_head);
    assert!(backend.rebase_state(&repo_ref).unwrap().is_none());
    assert_eq!(repo.git(&["rev-parse", "main"]), original_head.to_hex());

    // A plain (non-interactive) rebase's conflict must abort the same way.
    let repo2 = TestRepoBuilder::new()
        .commit_file("g.txt", "line1\n", "base")
        .branch("feature")
        .commit_file("g.txt", "line1\nfrom-feature\n", "feature change")
        .checkout("main")
        .commit_file("g.txt", "line1\nfrom-main\n", "main change")
        .checkout("feature")
        .build();
    let repo2_ref = repo2.repo_ref();
    let feature_head = head_oid(backend, &repo2_ref);
    let main_ref = backend
        .refs(&repo2_ref)
        .unwrap()
        .into_iter()
        .find(|r| matches!(&r.kind, RefKind::LocalBranch { name } if name == "main"))
        .unwrap();
    let outcome = backend
        .rebase(&repo2_ref, &main_ref, &RebaseOptions::default())
        .unwrap();
    assert!(matches!(outcome, RebaseOutcome::Conflicted { .. }));
    backend.rebase_abort(&repo2_ref).unwrap();
    assert_eq!(head_oid(backend, &repo2_ref), feature_head);
}

pub fn check_rebase_interactive_from_root(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new().commit("one").commit("two").build();
    let repo_ref = repo.repo_ref();
    let log = backend.log(&repo_ref, &LogOptions::default()).unwrap();
    let one = log.nodes[1].id;
    let two = log.nodes[0].id;

    let outcome = backend
        .rebase_interactive(
            &repo_ref,
            &InteractiveRebase {
                upstream: None,
                onto: None,
                steps: vec![pick(two), pick(one)],
                autosquash: false,
                cherry_pick: false,
            },
        )
        .unwrap();
    assert_eq!(outcome, RebaseOutcome::Completed);

    let log = backend.log(&repo_ref, &LogOptions::default()).unwrap();
    assert_eq!(
        log.nodes
            .iter()
            .map(|n| n.summary.as_str())
            .collect::<Vec<_>>(),
        vec!["one", "two"]
    );
    assert!(
        log.nodes[1].parents.is_empty(),
        "the root commit must stay parentless"
    );
}

/// A repo on `main` (`base`, `main work`) with a `topic` branch holding
/// three commits (`c1`, `c2`, `c3`, each adding its own file) that are
/// *not* in main's history. Returns the repo and `[c1, c2, c3]`.
fn cherry_pick_plan_fixture() -> (crate::FixtureRepo, [Oid; 3]) {
    let repo = TestRepoBuilder::new()
        .commit_file("base.txt", "base\n", "base")
        .branch("topic")
        .commit_file("c1.txt", "1\n", "c1")
        .commit_file("c2.txt", "2\n", "c2")
        .commit_file("c3.txt", "3\n", "c3")
        .checkout("main")
        .commit_file("main.txt", "m\n", "main work")
        .build();
    let ids: Vec<Oid> = ["topic~2", "topic~1", "topic"]
        .iter()
        .map(|r| Oid::from_hex(&repo.git(&["rev-parse", r])).unwrap())
        .collect();
    (repo, [ids[0], ids[1], ids[2]])
}

pub fn check_cherry_pick_plan_applies_reordered_and_squashed(backend: &dyn GitBackend) {
    let (repo, [c1, c2, c3]) = cherry_pick_plan_fixture();
    let repo_ref = repo.repo_ref();
    let main_tip = head_oid(backend, &repo_ref);

    let outcome = backend
        .rebase_interactive(
            &repo_ref,
            &InteractiveRebase {
                upstream: None,
                onto: None,
                // c3 first, then c1 with c2 squashed into it.
                steps: vec![pick(c3), pick(c1), step(c2, RebaseAction::Squash)],
                autosquash: false,
                cherry_pick: true,
            },
        )
        .unwrap();
    assert_eq!(outcome, RebaseOutcome::Completed);

    let log = backend.log(&repo_ref, &LogOptions::default()).unwrap();
    // main work <- c3 <- (c1 + c2 squashed)
    assert_eq!(log.nodes[2].id, main_tip);
    assert_eq!(log.nodes[1].summary, "c3");
    assert!(log.nodes[0].summary.starts_with("c1"));
    let head_message = repo.git(&["log", "-1", "--format=%B"]);
    assert!(head_message.contains("c1") && head_message.contains("c2"));
    for f in ["c1.txt", "c2.txt", "c3.txt", "main.txt"] {
        assert!(repo.root().join(f).exists(), "{f} must be present");
    }
    assert_eq!(repo.git(&["symbolic-ref", "--short", "HEAD"]), "main");
    assert!(backend.rebase_state(&repo_ref).unwrap().is_none());
}

/// `main` and `topic` both rewrite `f.txt`'s second line; picking topic's
/// commit onto main conflicts.
fn cherry_pick_conflict_fixture() -> (crate::FixtureRepo, Oid) {
    let repo = TestRepoBuilder::new()
        .commit_file("f.txt", "line1\nline2\n", "base")
        .branch("topic")
        .commit_file("f.txt", "line1\nfrom-topic\n", "topic change")
        .checkout("main")
        .commit_file("f.txt", "line1\nfrom-main\n", "main change")
        .build();
    let topic = Oid::from_hex(&repo.git(&["rev-parse", "topic"])).unwrap();
    (repo, topic)
}

fn conflicting_cherry_pick_plan(topic: Oid) -> InteractiveRebase {
    InteractiveRebase {
        upstream: None,
        onto: None,
        steps: vec![pick(topic)],
        autosquash: false,
        cherry_pick: true,
    }
}

pub fn check_cherry_pick_plan_conflict_then_continue(backend: &dyn GitBackend) {
    let (repo, topic) = cherry_pick_conflict_fixture();
    let repo_ref = repo.repo_ref();

    let outcome = backend
        .rebase_interactive(&repo_ref, &conflicting_cherry_pick_plan(topic))
        .unwrap();
    assert!(
        matches!(outcome, RebaseOutcome::Conflicted { .. }),
        "{outcome:?}"
    );
    assert!(backend.rebase_state(&repo_ref).unwrap().unwrap().conflicted);

    std::fs::write(repo.root().join("f.txt"), "line1\nresolved\n").unwrap();
    repo.git(&["add", "f.txt"]);
    let outcome = backend.rebase_continue(&repo_ref).unwrap();
    assert_eq!(outcome, RebaseOutcome::Completed);
    let log = backend.log(&repo_ref, &LogOptions::default()).unwrap();
    assert_eq!(log.nodes[0].summary, "topic change");
    assert_eq!(log.nodes[1].summary, "main change");
    assert!(backend.rebase_state(&repo_ref).unwrap().is_none());
}

pub fn check_cherry_pick_plan_conflict_then_abort(backend: &dyn GitBackend) {
    let (repo, topic) = cherry_pick_conflict_fixture();
    let repo_ref = repo.repo_ref();
    let before = head_oid(backend, &repo_ref);

    let outcome = backend
        .rebase_interactive(&repo_ref, &conflicting_cherry_pick_plan(topic))
        .unwrap();
    assert!(
        matches!(outcome, RebaseOutcome::Conflicted { .. }),
        "{outcome:?}"
    );

    backend.rebase_abort(&repo_ref).unwrap();
    assert_eq!(head_oid(backend, &repo_ref), before);
    assert!(backend.rebase_state(&repo_ref).unwrap().is_none());
    assert_eq!(repo.git(&["symbolic-ref", "--short", "HEAD"]), "main");
}

pub fn check_cherry_pick_plan_refuses_duplicates_and_leading_squash(backend: &dyn GitBackend) {
    let (repo, [c1, c2, _c3]) = cherry_pick_plan_fixture();
    let repo_ref = repo.repo_ref();
    let before = head_oid(backend, &repo_ref);
    let req = |steps| InteractiveRebase {
        upstream: None,
        onto: None,
        steps,
        autosquash: false,
        cherry_pick: true,
    };

    let err = backend
        .rebase_interactive(&repo_ref, &req(vec![pick(c1), pick(c1)]))
        .unwrap_err();
    assert!(
        matches!(
            err,
            gitpoulpi_core::Error::RebaseRefused(RebaseRefusal::PlanMismatch)
        ),
        "{err:?}"
    );
    let err = backend
        .rebase_interactive(
            &repo_ref,
            &req(vec![step(c1, RebaseAction::Squash), pick(c2)]),
        )
        .unwrap_err();
    assert!(
        matches!(
            err,
            gitpoulpi_core::Error::RebaseRefused(RebaseRefusal::SquashWithoutPredecessor)
        ),
        "{err:?}"
    );
    let err = backend
        .rebase_interactive(&repo_ref, &req(vec![]))
        .unwrap_err();
    assert!(
        matches!(
            err,
            gitpoulpi_core::Error::RebaseRefused(RebaseRefusal::PlanMismatch)
        ),
        "{err:?}"
    );
    assert_eq!(head_oid(backend, &repo_ref), before);
    assert!(backend.rebase_state(&repo_ref).unwrap().is_none());
}

pub fn check_rebase_interactive_dropping_everything_lands_on_upstream(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .commit("base")
        .commit("one")
        .commit("two")
        .build();
    let repo_ref = repo.repo_ref();
    let log = backend.log(&repo_ref, &LogOptions::default()).unwrap();
    let base = log.nodes[2].id;
    let one = log.nodes[1].id;
    let two = log.nodes[0].id;

    let outcome = backend
        .rebase_interactive(
            &repo_ref,
            &InteractiveRebase {
                upstream: Some(base),
                onto: None,
                steps: vec![step(one, RebaseAction::Drop), step(two, RebaseAction::Drop)],
                autosquash: false,
                cherry_pick: false,
            },
        )
        .unwrap();
    assert_eq!(outcome, RebaseOutcome::Completed);
    assert_eq!(head_oid(backend, &repo_ref), base);
}

pub fn check_rebase_interactive_refusals_are_typed_and_change_nothing(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new().commit("base").commit("one").build();
    let repo_ref = repo.repo_ref();
    let log = backend.log(&repo_ref, &LogOptions::default()).unwrap();
    let base = log.nodes[1].id;
    let one = log.nodes[0].id;
    let original_head = one;

    let assert_untouched = || {
        assert_eq!(repo.git(&["rev-parse", "HEAD"]), original_head.to_hex());
        assert!(!repo.root().join(".git").join("gitpoulpi-rebase").exists());
    };

    // Non-ancestor upstream.
    let elsewhere = TestRepoBuilder::new().commit("unrelated").build();
    let elsewhere_head = head_oid(backend, &elsewhere.repo_ref());
    assert_eq!(
        rebase_refused(backend.rebase_interactive(
            &repo_ref,
            &InteractiveRebase {
                upstream: Some(elsewhere_head),
                onto: None,
                steps: vec![pick(one)],
                autosquash: false,
                cherry_pick: false,
            },
        )),
        RebaseRefusal::NotOnCurrentBranch
    );
    assert_untouched();

    // Plan mismatch (missing a commit from the range).
    assert_eq!(
        rebase_refused(backend.rebase_interactive(
            &repo_ref,
            &InteractiveRebase {
                upstream: Some(base),
                onto: None,
                steps: vec![],
                autosquash: false,
                cherry_pick: false,
            },
        )),
        RebaseRefusal::PlanMismatch
    );
    assert_untouched();

    // Squash first.
    assert_eq!(
        rebase_refused(backend.rebase_interactive(
            &repo_ref,
            &InteractiveRebase {
                upstream: Some(base),
                onto: None,
                steps: vec![step(one, RebaseAction::Squash)],
                autosquash: false,
                cherry_pick: false,
            },
        )),
        RebaseRefusal::SquashWithoutPredecessor
    );
    assert_untouched();

    // Empty reword message.
    assert_eq!(
        rebase_refused(backend.rebase_interactive(
            &repo_ref,
            &InteractiveRebase {
                upstream: Some(base),
                onto: None,
                steps: vec![reword_step(one, "  \r\n ")],
                autosquash: false,
                cherry_pick: false,
            },
        )),
        RebaseRefusal::EmptyRewordMessage
    );
    assert_untouched();

    // Dirty working tree -- a *tracked* file modified but not committed.
    // An untracked scratch file must not trip this (matching plain `git
    // rebase`, which tolerates untracked files): `is_dirty` deliberately
    // asks with `-uno`.
    std::fs::write(repo.root().join("one.txt"), "modified").unwrap();
    assert_eq!(
        rebase_refused(backend.rebase_interactive(
            &repo_ref,
            &InteractiveRebase {
                upstream: Some(base),
                onto: None,
                steps: vec![pick(one)],
                autosquash: false,
                cherry_pick: false,
            },
        )),
        RebaseRefusal::UncommittedChanges
    );
    repo.git(&["checkout", "--", "one.txt"]);
    assert_untouched();

    // Operation already in progress.
    std::fs::write(
        repo.root().join(".git").join("MERGE_HEAD"),
        format!("{}\n", base.to_hex()),
    )
    .unwrap();
    assert_eq!(
        rebase_refused(backend.rebase_interactive(
            &repo_ref,
            &InteractiveRebase {
                upstream: Some(base),
                onto: None,
                steps: vec![pick(one)],
                autosquash: false,
                cherry_pick: false,
            },
        )),
        RebaseRefusal::OperationInProgress
    );
    std::fs::remove_file(repo.root().join(".git").join("MERGE_HEAD")).unwrap();
    assert_untouched();
}

pub fn check_rebase_continue_without_rebase_is_a_typed_refusal(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new().commit("base").build();
    let repo_ref = repo.repo_ref();

    assert_eq!(
        rebase_refused(backend.rebase_continue(&repo_ref)),
        RebaseRefusal::NoRebaseInProgress
    );
    assert_eq!(
        rebase_refused(backend.rebase_skip(&repo_ref)),
        RebaseRefusal::NoRebaseInProgress
    );
    match backend.rebase_abort(&repo_ref) {
        Err(gitpoulpi_core::Error::RebaseRefused(RebaseRefusal::NoRebaseInProgress)) => {}
        other => panic!("expected a typed refusal, got {other:?}"),
    }
}

pub fn check_rebase_interactive_ignores_user_editor_and_autosquash_config(
    backend: &dyn GitBackend,
) {
    let repo = TestRepoBuilder::new()
        .commit("base")
        .commit("first")
        .commit("second")
        .config("sequence.editor", "false")
        .config("core.editor", "false")
        .config("rebase.autoSquash", "true")
        .build();
    let repo_ref = repo.repo_ref();
    let log = backend.log(&repo_ref, &LogOptions::default()).unwrap();
    let base = log.nodes[2].id;
    let first = log.nodes[1].id;
    let second = log.nodes[0].id;

    let outcome = backend
        .rebase_interactive(
            &repo_ref,
            &InteractiveRebase {
                upstream: Some(base),
                onto: None,
                steps: vec![
                    step(second, RebaseAction::Pick),
                    step(first, RebaseAction::Pick),
                ],
                autosquash: false,
                cherry_pick: false,
            },
        )
        .unwrap();
    assert_eq!(outcome, RebaseOutcome::Completed);
    let log = backend.log(&repo_ref, &LogOptions::default()).unwrap();
    // The plan swapped the two commits' order (second replayed before
    // first), so the new history is first (newest), second, base.
    assert_eq!(
        log.nodes
            .iter()
            .map(|n| n.summary.as_str())
            .collect::<Vec<_>>(),
        vec!["first", "second", "base"]
    );
}

pub fn check_rebase_interactive_in_a_path_with_spaces(backend: &dyn GitBackend) {
    // A real, uniquely-named temp dir (not a hand-rolled name keyed on
    // the process id) -- both backends' contract suites run this check
    // concurrently in the same process, and a shared name raced two
    // `git init`s onto the same directory.
    let parent = tempfile::Builder::new()
        .prefix("gitpoulpi rebase test ")
        .tempdir()
        .unwrap();
    let root = parent.path().join("repo with spaces");
    std::fs::create_dir_all(&root).unwrap();

    let run_git = |args: &[&str]| {
        let out = std::process::Command::new("git")
            .args(args)
            .current_dir(&root)
            .env("GIT_CONFIG_GLOBAL", "")
            .env("GIT_CONFIG_SYSTEM", "")
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .output()
            .expect("spawn git");
        assert!(
            out.status.success(),
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr)
        );
    };
    run_git(&["init", "--initial-branch=main"]);
    run_git(&["config", "user.name", "GitPoulpi Testkit"]);
    run_git(&["config", "user.email", "testkit@gitpoulpi.local"]);
    std::fs::write(root.join("one.txt"), "one").unwrap();
    run_git(&["add", "one.txt"]);
    run_git(&["commit", "-m", "one"]);
    std::fs::write(root.join("two.txt"), "two").unwrap();
    run_git(&["add", "two.txt"]);
    run_git(&["commit", "-m", "two"]);

    let repo_ref = gitpoulpi_core::domain::RepoRef::from_root(&root);
    let log = backend.log(&repo_ref, &LogOptions::default()).unwrap();
    let one = log.nodes[1].id;
    let two = log.nodes[0].id;

    let outcome = backend
        .rebase_interactive(
            &repo_ref,
            &InteractiveRebase {
                upstream: None,
                onto: None,
                steps: vec![pick(two), pick(one)],
                autosquash: false,
                cherry_pick: false,
            },
        )
        .unwrap();
    assert_eq!(outcome, RebaseOutcome::Completed);
}

/// A conflicted merge has to be *readable* (all three stages) and
/// resolvable through the trait, since the TUI's merge UI and the CLI's
/// `conflicts` commands are both built on nothing but these two calls.
pub fn check_conflicts_are_listed_and_resolvable(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new().with_initial().build();
    let repo_ref = repo.repo_ref();
    let base = head_oid(backend, &repo_ref);

    backend.branch_create(&repo_ref, "feature", base).unwrap();
    backend
        .checkout(&repo_ref, &CheckoutTarget::Branch("feature".into()))
        .unwrap();
    std::fs::write(repo.root().join("shared.txt"), "theirs\n").unwrap();
    backend
        .stage(&repo_ref, &StageTarget::File("shared.txt".into()))
        .unwrap();
    backend
        .commit(&repo_ref, "theirs", &CommitOptions::default())
        .unwrap();

    backend
        .checkout(&repo_ref, &CheckoutTarget::Branch("main".into()))
        .unwrap();
    std::fs::write(repo.root().join("shared.txt"), "ours\n").unwrap();
    backend
        .stage(&repo_ref, &StageTarget::File("shared.txt".into()))
        .unwrap();
    backend
        .commit(&repo_ref, "ours", &CommitOptions::default())
        .unwrap();

    let feature_ref = backend
        .refs(&repo_ref)
        .unwrap()
        .into_iter()
        .find(|r| matches!(&r.kind, RefKind::LocalBranch { name } if name == "feature"))
        .expect("feature should exist");
    let outcome = backend
        .merge(&repo_ref, &feature_ref, &MergeOptions::default())
        .unwrap();
    assert_eq!(
        outcome,
        MergeOutcome::Conflicted(vec!["shared.txt".to_string()])
    );

    let conflicts = backend.conflicts(&repo_ref).unwrap();
    assert_eq!(conflicts.len(), 1, "got {conflicts:?}");
    assert_eq!(conflicts[0].path.to_str(), Some("shared.txt"));
    assert_eq!(conflicts[0].ours.as_deref(), Some("ours\n"));
    assert_eq!(conflicts[0].theirs.as_deref(), Some("theirs\n"));

    backend
        .resolve_conflict(
            &repo_ref,
            std::path::Path::new("shared.txt"),
            ConflictResolution::TakeSide(ConflictSide::Ours),
            &gitpoulpi_core::domain::ResolveOptions::default(),
        )
        .unwrap();
    assert!(
        backend.conflicts(&repo_ref).unwrap().is_empty(),
        "a resolved path must leave the conflict list"
    );
    assert_eq!(
        std::fs::read_to_string(repo.root().join("shared.txt")).unwrap(),
        "ours\n"
    );
}

/// One line per commit reachable from `rev`, newest first, with
/// everything a reword must keep: tree, author name/email/date (raw, so
/// the timezone counts), the number of parents - and the subject, which
/// the caller compares separately. Oids are deliberately absent: every
/// rewritten commit gets a new one.
fn reword_history(repo: &crate::FixtureRepo, rev: &str) -> Vec<(String, String)> {
    let raw = repo.git(&[
        "log",
        "--topo-order",
        "--date=raw",
        "--format=%T|%an|%ae|%ad|%P%x01%s",
        rev,
    ]);
    raw.lines()
        .map(|line| {
            let (kept, subject) = line.split_once('\u{1}').expect("separator");
            let (fixed, parents) = kept.rsplit_once('|').expect("parents field");
            let parent_count = parents.split_whitespace().count();
            (
                format!("{fixed}|parents={parent_count}"),
                subject.to_string(),
            )
        })
        .collect()
}

/// On HEAD a reword is a message-only amend: same tree, same parent, same
/// author and author date, and the index is *not* folded in - a staged
/// file stays staged, where `git commit --amend` would have committed it.
/// The branch stays checked out, and `ORIG_HEAD` names the old tip.
pub fn check_reword_head_changes_only_the_message(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .commit("one")
        .commit("two")
        .write_worktree("staged.txt", b"staged")
        .stage("staged.txt")
        .write_worktree("one.txt", b"dirty")
        .build();
    let repo_ref = repo.repo_ref();
    let old_head = head_oid(backend, &repo_ref);
    let before = reword_history(&repo, "HEAD");
    let status_before = repo.git(&["status", "--porcelain"]);

    let outcome = backend
        .reword(&repo_ref, old_head, "two, reworded\n\nwith a body\n")
        .unwrap();

    assert_eq!(outcome.previous_head, old_head);
    assert_eq!(outcome.new_head, outcome.reworded);
    assert_eq!(outcome.replayed, 0);
    assert_ne!(outcome.reworded, old_head);
    assert_eq!(repo.git(&["rev-parse", "HEAD"]), outcome.new_head.to_hex());
    assert_eq!(repo.git(&["symbolic-ref", "HEAD"]), "refs/heads/main");
    assert_eq!(
        repo.git(&["log", "-1", "--format=%B", "HEAD"]),
        "two, reworded\n\nwith a body"
    );
    let after = reword_history(&repo, "HEAD");
    assert_eq!(after[0].0, before[0].0, "tree/author/parents of HEAD");
    assert_eq!(after[0].1, "two, reworded");
    assert_eq!(after[1..], before[1..], "the parent is untouched");
    assert_eq!(
        repo.git(&["rev-parse", "HEAD~1"]),
        repo.git(&["rev-parse", &format!("{}~1", old_head.to_hex())])
    );
    assert_eq!(repo.git(&["status", "--porcelain"]), status_before);
    assert_eq!(repo.git(&["rev-parse", "ORIG_HEAD"]), old_head.to_hex());
}

/// An older commit: its message changes, and every descendant up to HEAD -
/// including a merge, kept as a merge -- is re-created with its own
/// tree, author, author date and message. A dirty working tree is left
/// exactly as it was, because no tree changes.
pub fn check_reword_older_commit_replays_descendants_and_merges(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new().commit("base").build();
    // The target gets an explicit, odd-timezone author date, so a backend
    // that re-stamped authors with "now" cannot pass by luck.
    std::fs::write(repo.root().join("target.txt"), "t").unwrap();
    repo.git(&["add", "target.txt"]);
    repo.git(&[
        "commit",
        "--date=2001-02-03T04:05:06+0530",
        "-m",
        "target",
        "-m",
        "old body",
    ]);
    repo.git(&["checkout", "-b", "side"]);
    std::fs::write(repo.root().join("side.txt"), "s").unwrap();
    repo.git(&["add", "side.txt"]);
    repo.git(&["commit", "-m", "on side"]);
    repo.git(&["checkout", "main"]);
    std::fs::write(repo.root().join("main.txt"), "m").unwrap();
    repo.git(&["add", "main.txt"]);
    repo.git(&["commit", "-m", "on main"]);
    repo.git(&["merge", "--no-ff", "--no-edit", "side"]);
    std::fs::write(repo.root().join("last.txt"), "l").unwrap();
    repo.git(&["add", "last.txt"]);
    repo.git(&["commit", "-m", "last"]);
    std::fs::write(repo.root().join("base.txt"), "dirty").unwrap();

    let repo_ref = repo.repo_ref();
    let old_head = head_oid(backend, &repo_ref);
    let target = backend.resolve(&repo_ref, "HEAD~3").unwrap();
    assert_eq!(
        repo.git(&["log", "-1", "--format=%s", &target.to_hex()]),
        "target"
    );
    let before = reword_history(&repo, "HEAD");
    let status_before = repo.git(&["status", "--porcelain"]);

    let outcome = backend
        .reword(&repo_ref, target, "target, reworded")
        .unwrap();

    assert_eq!(outcome.previous_head, old_head);
    assert_eq!(outcome.replayed, 4, "on side, on main, the merge, last");
    assert_eq!(repo.git(&["rev-parse", "HEAD"]), outcome.new_head.to_hex());
    assert_eq!(repo.git(&["symbolic-ref", "HEAD"]), "refs/heads/main");
    assert_eq!(
        repo.git(&["log", "-1", "--format=%B", &outcome.reworded.to_hex()]),
        "target, reworded"
    );
    let after = reword_history(&repo, "HEAD");
    assert_eq!(after.len(), before.len());
    for (i, (a, b)) in after.iter().zip(&before).enumerate() {
        assert_eq!(a.0, b.0, "commit #{i}: tree/author/date/parent count");
        if b.1 == "target" {
            assert_eq!(a.1, "target, reworded");
        } else {
            assert_eq!(a.1, b.1, "commit #{i}: a descendant keeps its message");
        }
    }
    // The merge's side parent is the *rewritten* side commit (a
    // descendant of the target), and the base below the target is kept.
    let merge_side = repo.git(&["rev-parse", "HEAD~1^2"]);
    assert_ne!(merge_side, repo.git(&["rev-parse", "side"]));
    assert_eq!(
        repo.git(&["rev-parse", &format!("{merge_side}~1")]),
        outcome.reworded.to_hex()
    );
    assert_eq!(
        repo.git(&["rev-parse", &format!("{}~1", outcome.reworded.to_hex())]),
        repo.git(&["rev-parse", &format!("{}~1", target.to_hex())])
    );
    assert_eq!(repo.git(&["status", "--porcelain"]), status_before);
    assert_eq!(repo.git(&["rev-parse", "ORIG_HEAD"]), old_head.to_hex());
    assert!(
        repo.git(&["reflog", "-1", "--format=%gs", "main"])
            .contains("reword"),
        "the branch's reflog records the rewrite"
    );
}

/// Refusals are typed and change nothing: a commit on another branch, an
/// empty message, and a merge in progress.
pub fn check_reword_refusals_are_typed_and_change_nothing(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .commit("base")
        .branch("feature")
        .commit("elsewhere")
        .checkout("main")
        .commit("head")
        .build();
    let repo_ref = repo.repo_ref();
    let head = head_oid(backend, &repo_ref);
    let off_branch = backend.resolve(&repo_ref, "feature").unwrap();

    let refused = |result: gitpoulpi_core::Result<_>| match result {
        Err(gitpoulpi_core::Error::RewordRefused(r)) => r,
        other => panic!("expected a typed refusal, got {other:?}"),
    };

    assert_eq!(
        refused(backend.reword(&repo_ref, off_branch, "nope")),
        RewordRefusal::NotOnCurrentBranch
    );
    assert_eq!(
        refused(backend.reword(&repo_ref, head, " \r\n\n  ")),
        RewordRefusal::EmptyMessage
    );
    // A merge in progress, as git records it.
    std::fs::write(
        repo.root().join(".git").join("MERGE_HEAD"),
        format!("{}\n", off_branch.to_hex()),
    )
    .unwrap();
    assert_eq!(
        refused(backend.reword(&repo_ref, head, "fine")),
        RewordRefusal::OperationInProgress
    );
    assert_eq!(repo.git(&["rev-parse", "HEAD"]), head.to_hex());
    assert_eq!(repo.git(&["rev-parse", "feature"]), off_branch.to_hex());
}

// -- fast-forward and worktree add -----------------------

/// A branch that is not checked out moves by its ref alone: HEAD, the
/// index and the working tree stay exactly where they were. Asking again -
/// or asking to "fast-forward" onto an older commit -- is `UpToDate`
/// and moves nothing (moving it *back* would be a reset, not a
/// fast-forward).
pub fn check_fast_forward_moves_a_branch_that_is_not_checked_out(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .with_initial()
        .branch("feature")
        .checkout("main")
        .commit("second")
        .build();
    let repo_ref = repo.repo_ref();
    let initial = rev(&repo, "feature");
    let main = rev(&repo, "main");
    std::fs::write(repo.root().join("dirty.txt"), "uncommitted").unwrap();

    assert_eq!(
        backend.fast_forward(&repo_ref, "feature", main).unwrap(),
        FastForwardOutcome::FastForwarded {
            from: initial,
            to: main
        }
    );
    assert_eq!(rev(&repo, "feature"), main);
    assert_eq!(repo.git(&["symbolic-ref", "--short", "HEAD"]), "main");
    assert_eq!(
        std::fs::read_to_string(repo.root().join("dirty.txt")).unwrap(),
        "uncommitted",
        "moving another branch must not touch the working tree"
    );

    assert_eq!(
        backend.fast_forward(&repo_ref, "feature", main).unwrap(),
        FastForwardOutcome::UpToDate
    );
    assert_eq!(
        backend.fast_forward(&repo_ref, "main", initial).unwrap(),
        FastForwardOutcome::UpToDate,
        "a target behind the branch is already contained in it"
    );
    assert_eq!(rev(&repo, "main"), main);
}

/// The checked-out branch fast-forwards like `git merge --ff-only`: the
/// ref moves *and* the files the new commits added appear on disk, with
/// nothing left staged or modified.
pub fn check_fast_forward_of_the_checked_out_branch_updates_the_working_tree(
    backend: &dyn GitBackend,
) {
    let repo = TestRepoBuilder::new()
        .with_initial()
        .branch("feature")
        .commit_file("feature.txt", "from feature\n", "feature work")
        .checkout("main")
        .build();
    let repo_ref = repo.repo_ref();
    let initial = rev(&repo, "main");
    let tip = rev(&repo, "feature");
    assert!(!repo.root().join("feature.txt").exists());

    assert_eq!(
        backend.fast_forward(&repo_ref, "main", tip).unwrap(),
        FastForwardOutcome::FastForwarded {
            from: initial,
            to: tip
        }
    );
    assert_eq!(rev(&repo, "HEAD"), tip);
    assert_eq!(repo.git(&["symbolic-ref", "--short", "HEAD"]), "main");
    assert_eq!(
        std::fs::read_to_string(repo.root().join("feature.txt"))
            .unwrap()
            .replace('\r', ""),
        "from feature\n"
    );
    assert!(
        backend.status(&repo_ref).unwrap().files.is_empty(),
        "a fast-forward leaves nothing staged or modified"
    );
}

/// Diverged history is the typed `NotFastForward`, and a branch checked
/// out in another linked worktree is refused - both with nothing moved.
pub fn check_fast_forward_refuses_diverged_and_elsewhere_checked_out_branches(
    backend: &dyn GitBackend,
) {
    let (builder, _wt) = TestRepoBuilder::new()
        .with_initial()
        .worktree("wt-linked", "linked");
    let repo = builder
        .branch("feature")
        .commit("on feature")
        .checkout("main")
        .commit("on main")
        .build();
    let repo_ref = repo.repo_ref();
    let main = rev(&repo, "main");
    let feature = rev(&repo, "feature");
    let linked = rev(&repo, "linked");

    assert!(matches!(
        backend.fast_forward(&repo_ref, "feature", main),
        Err(gitpoulpi_core::Error::NotFastForward)
    ));
    assert!(matches!(
        backend.fast_forward(&repo_ref, "main", feature),
        Err(gitpoulpi_core::Error::NotFastForward)
    ));
    assert_eq!(rev(&repo, "feature"), feature);
    assert_eq!(rev(&repo, "main"), main);

    // `linked` sits at `initial`, an ancestor of main: a genuine
    // fast-forward, refused only because another worktree has it.
    assert!(backend.fast_forward(&repo_ref, "linked", main).is_err());
    assert_eq!(rev(&repo, "linked"), linked);

    assert!(
        backend
            .fast_forward(&repo_ref, "no-such-branch", main)
            .is_err(),
        "an unknown branch is an error, not a branch created at the target"
    );
}

/// Fast-forwarding the checked-out branch over a local change the move
/// would overwrite is refused, and neither the ref nor the change moves.
pub fn check_fast_forward_of_the_checked_out_branch_keeps_local_changes(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .commit_file("shared.txt", "one\n", "initial")
        .branch("feature")
        .commit_file("shared.txt", "two\n", "feature edits shared")
        .checkout("main")
        .build();
    let repo_ref = repo.repo_ref();
    let main = rev(&repo, "main");
    let feature = rev(&repo, "feature");
    std::fs::write(repo.root().join("shared.txt"), "my local edit\n").unwrap();

    assert!(backend.fast_forward(&repo_ref, "main", feature).is_err());
    assert_eq!(rev(&repo, "main"), main);
    assert_eq!(
        std::fs::read_to_string(repo.root().join("shared.txt")).unwrap(),
        "my local edit\n"
    );
}

/// The refusal of a fast-forward over local changes reads as "local changes
/// are in the way" (`gitpoulpi_core::failure::LOCAL_CHANGES_NEEDLES`) on every
/// backend: the daemon's auto-stash recognises it by those phrases, so a
/// backend wording it differently would silently disable the auto-stash of
/// "Pull all branches".
pub fn check_fast_forward_over_local_changes_reads_as_local_changes(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .commit_file("shared.txt", "one\n", "initial")
        .branch("feature")
        .commit_file("shared.txt", "two\n", "feature edits shared")
        .checkout("main")
        .build();
    let repo_ref = repo.repo_ref();
    let feature = rev(&repo, "feature");
    std::fs::write(repo.root().join("shared.txt"), "my local edit\n").unwrap();

    let error = backend
        .fast_forward(&repo_ref, "main", feature)
        .expect_err("the move would overwrite the local edit");
    assert!(
        gitpoulpi_core::failure::is_local_changes_message(&error.to_string()),
        "not read as local changes: {error}"
    );
}

/// A merge commit of `feature` (adds `feature.txt`) into `main` (adds
/// `main.txt`), made with `--no-ff`.
fn repo_with_a_merge_commit() -> crate::fixture::FixtureRepo {
    let repo = TestRepoBuilder::new().with_initial().build();
    repo.git(&["checkout", "-q", "-b", "feature"]);
    std::fs::write(repo.root().join("feature.txt"), "feature").unwrap();
    repo.git(&["add", "feature.txt"]);
    repo.git(&["commit", "-q", "-m", "feature work"]);
    repo.git(&["checkout", "-q", "main"]);
    std::fs::write(repo.root().join("main.txt"), "main").unwrap();
    repo.git(&["add", "main.txt"]);
    repo.git(&["commit", "-q", "-m", "main work"]);
    repo.git(&["merge", "-q", "--no-ff", "-m", "merge feature", "feature"]);
    repo
}

pub fn check_revert_of_a_merge_commit_takes_the_mainline_parent(backend: &dyn GitBackend) {
    let repo = repo_with_a_merge_commit();
    let repo_ref = repo.repo_ref();
    let merge = head_oid(backend, &repo_ref);

    // No mainline: a merge commit cannot be reverted, and nothing changes.
    assert!(backend.revert(&repo_ref, merge, None).is_err());
    assert_eq!(head_oid(backend, &repo_ref), merge);
    assert_no_sequencer_state(&repo);

    // Mainline 1 (main): the feature's changes go away, main's stay.
    let outcome = backend.revert(&repo_ref, merge, Some(1)).unwrap();
    let RevertOutcome::Completed(reverted) = outcome else {
        panic!("a clean revert must complete, got {outcome:?}");
    };
    assert_eq!(head_oid(backend, &repo_ref), reverted);
    assert!(!repo.root().join("feature.txt").exists());
    assert!(repo.root().join("main.txt").exists());
    let log = backend.log(&repo_ref, &LogOptions::default()).unwrap();
    assert_eq!(log.nodes[0].parents, vec![merge]);

    // Mainline 2 (feature): the other way round.
    repo.git(&["reset", "-q", "--hard", &merge.to_hex()]);
    backend.revert(&repo_ref, merge, Some(2)).unwrap();
    assert!(repo.root().join("feature.txt").exists());
    assert!(!repo.root().join("main.txt").exists());
}

// -- amend (change a commit's message, author, dates) -----------------------

/// `%an|%ae|%ad|%cn|%ce|%T|%s` per commit reachable from `rev`, newest
/// first, with raw author dates. The committer date is "now" for a
/// re-created commit, so it is read separately where a test pins it.
fn amend_history(repo: &crate::FixtureRepo, rev: &str) -> Vec<String> {
    repo.git(&[
        "log",
        "--topo-order",
        "--date=raw",
        "--format=%an|%ae|%ad|%cn|%ce|%T|%s",
        rev,
    ])
    .lines()
    .map(str::to_string)
    .collect()
}

/// On HEAD: the chosen author name, email and date and the body change at
/// once; the summary, tree, parent, staged files and the rest stay.
pub fn check_amend_head_changes_only_the_chosen_properties(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .commit("one")
        .commit("two")
        .write_worktree("staged.txt", b"staged")
        .stage("staged.txt")
        .build();
    let repo_ref = repo.repo_ref();
    let old_head = head_oid(backend, &repo_ref);
    let before = amend_history(&repo, "HEAD");
    let status_before = repo.git(&["status", "--porcelain"]);

    let amend = gitpoulpi_core::domain::CommitAmend {
        message: Some("two\n\nthe new body\n".into()),
        author_name: Some("Grace Hopper".into()),
        author_email: Some("grace@example.org".into()),
        author_date: Some("2001-02-03T04:05:06+05:30".into()),
        ..Default::default()
    };
    let outcome = backend.amend_commit(&repo_ref, old_head, &amend).unwrap();

    assert_eq!(outcome.previous_head, old_head);
    assert_eq!(outcome.new_head, outcome.reworded);
    assert_eq!(outcome.replayed, 0);
    assert_ne!(outcome.reworded, old_head);
    assert_eq!(repo.git(&["rev-parse", "HEAD"]), outcome.new_head.to_hex());
    assert_eq!(repo.git(&["symbolic-ref", "HEAD"]), "refs/heads/main");
    assert_eq!(
        repo.git(&["log", "-1", "--format=%B", "HEAD"]),
        "two\n\nthe new body"
    );
    let after = amend_history(&repo, "HEAD");
    let parts: Vec<&str> = after[0].split('|').collect();
    assert_eq!(parts[0], "Grace Hopper");
    assert_eq!(parts[1], "grace@example.org");
    assert_eq!(parts[2], "981153306 +0530");
    let old_parts: Vec<&str> = before[0].split('|').collect();
    assert_eq!(parts[5], old_parts[5], "same tree");
    assert_eq!(parts[6], "two", "same summary");
    assert_eq!(after[1..], before[1..], "the parent is untouched");
    assert_eq!(repo.git(&["status", "--porcelain"]), status_before);
    assert_eq!(repo.git(&["rev-parse", "ORIG_HEAD"]), old_head.to_hex());
    assert!(repo
        .git(&["reflog", "-1", "--format=%gs", "main"])
        .contains("amend"));
}

/// An older commit with a merge among its descendants: only the target's
/// author changes (the message stays when none is given), every descendant
/// keeps its own author and date, and a committer override lands on the
/// target alone.
pub fn check_amend_older_commit_replays_descendants_with_their_own_authors(
    backend: &dyn GitBackend,
) {
    let repo = TestRepoBuilder::new().commit("base").build();
    std::fs::write(repo.root().join("target.txt"), "t").unwrap();
    repo.git(&["add", "target.txt"]);
    repo.git(&[
        "commit",
        "--date=2001-02-03T04:05:06+0530",
        "-m",
        "target",
        "-m",
        "old body",
    ]);
    repo.git(&["checkout", "-b", "side"]);
    std::fs::write(repo.root().join("side.txt"), "s").unwrap();
    repo.git(&["add", "side.txt"]);
    repo.git(&[
        "commit",
        "--author=Side Author <side@example.org>",
        "--date=2005-06-07T08:09:10-0700",
        "-m",
        "on side",
    ]);
    repo.git(&["checkout", "main"]);
    std::fs::write(repo.root().join("main.txt"), "m").unwrap();
    repo.git(&["add", "main.txt"]);
    repo.git(&["commit", "-m", "on main"]);
    repo.git(&["merge", "--no-ff", "--no-edit", "side"]);
    std::fs::write(repo.root().join("last.txt"), "l").unwrap();
    repo.git(&["add", "last.txt"]);
    repo.git(&["commit", "-m", "last"]);
    std::fs::write(repo.root().join("base.txt"), "dirty").unwrap();

    let repo_ref = repo.repo_ref();
    let old_head = head_oid(backend, &repo_ref);
    let target = backend.resolve(&repo_ref, "HEAD~3").unwrap();
    let before = amend_history(&repo, "HEAD");
    let status_before = repo.git(&["status", "--porcelain"]);

    let amend = gitpoulpi_core::domain::CommitAmend {
        author_name: Some("Ada Lovelace".into()),
        committer_name: Some("Committer Override".into()),
        committer_email: Some("committer@example.org".into()),
        committer_date: Some("2010-01-02T03:04:05Z".into()),
        ..Default::default()
    };
    let outcome = backend.amend_commit(&repo_ref, target, &amend).unwrap();

    assert_eq!(outcome.previous_head, old_head);
    assert_eq!(outcome.replayed, 4);
    assert_eq!(repo.git(&["rev-parse", "HEAD"]), outcome.new_head.to_hex());
    let after = amend_history(&repo, "HEAD");
    assert_eq!(after.len(), before.len());
    for (i, (a, b)) in after.iter().zip(&before).enumerate() {
        let (a, b): (Vec<&str>, Vec<&str>) = (a.split('|').collect(), b.split('|').collect());
        if b[6] == "target" {
            assert_eq!(a[0], "Ada Lovelace");
            assert_eq!(a[1], b[1], "email kept");
            assert_eq!(a[2], b[2], "author date kept");
            assert_eq!(
                (a[3], a[4]),
                ("Committer Override", "committer@example.org")
            );
            assert_eq!(a[5..], b[5..]);
        } else {
            assert_eq!(a[..3], b[..3], "commit #{i}: author and date");
            assert_eq!(a[5..], b[5..], "commit #{i}: tree and summary");
        }
    }
    assert_eq!(
        repo.git(&["log", "-1", "--format=%B", &outcome.reworded.to_hex()]),
        "target\n\nold body"
    );
    assert_eq!(
        repo.git(&[
            "log",
            "-1",
            "--date=raw",
            "--format=%cd",
            &outcome.reworded.to_hex()
        ]),
        "1262401445 +0000"
    );
    assert_ne!(
        repo.git(&["log", "-1", "--format=%cn", "HEAD"]),
        "Committer Override",
        "a descendant's committer is the current identity"
    );
    assert_eq!(repo.git(&["status", "--porcelain"]), status_before);
    assert_eq!(repo.git(&["rev-parse", "ORIG_HEAD"]), old_head.to_hex());
}

/// The refusals are typed and change nothing.
pub fn check_amend_refusals_are_typed_and_change_nothing(backend: &dyn GitBackend) {
    use gitpoulpi_core::domain::CommitAmend;
    let repo = TestRepoBuilder::new()
        .commit("base")
        .branch("feature")
        .commit("elsewhere")
        .checkout("main")
        .commit("head")
        .build();
    let repo_ref = repo.repo_ref();
    let head = head_oid(backend, &repo_ref);
    let off_branch = backend.resolve(&repo_ref, "feature").unwrap();
    let refused = |result: gitpoulpi_core::Result<_>| match result {
        Err(gitpoulpi_core::Error::RewordRefused(r)) => r,
        other => panic!("expected a typed refusal, got {other:?}"),
    };
    let name = |n: &str| CommitAmend {
        author_name: Some(n.into()),
        ..Default::default()
    };

    assert_eq!(
        refused(backend.amend_commit(&repo_ref, off_branch, &name("Ann"))),
        RewordRefusal::NotOnCurrentBranch
    );
    assert_eq!(
        refused(backend.amend_commit(&repo_ref, head, &CommitAmend::message_only(" \n"))),
        RewordRefusal::EmptyMessage
    );
    assert_eq!(
        refused(backend.amend_commit(&repo_ref, head, &name("  "))),
        RewordRefusal::InvalidIdentity
    );
    assert_eq!(
        refused(backend.amend_commit(
            &repo_ref,
            head,
            &CommitAmend {
                author_email: Some("not an email".into()),
                ..Default::default()
            }
        )),
        RewordRefusal::InvalidIdentity
    );
    assert_eq!(
        refused(backend.amend_commit(
            &repo_ref,
            head,
            &CommitAmend {
                author_date: Some("last tuesday".into()),
                ..Default::default()
            }
        )),
        RewordRefusal::InvalidDate
    );
    std::fs::write(
        repo.root().join(".git").join("MERGE_HEAD"),
        format!("{}\n", off_branch.to_hex()),
    )
    .unwrap();
    assert_eq!(
        refused(backend.amend_commit(&repo_ref, head, &name("Ann"))),
        RewordRefusal::OperationInProgress
    );
    assert_eq!(repo.git(&["rev-parse", "HEAD"]), head.to_hex());
}

/// A commit node's timestamp is the *author* date (what the detail pane
/// calls "authored" and the Amend popup prefills), not the committer's.
pub fn check_log_timestamp_is_the_author_date(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new().commit("base").build();
    std::fs::write(repo.root().join("late.txt"), "l").unwrap();
    repo.git(&["add", "late.txt"]);
    repo.git(&[
        "commit",
        "--date=2001-02-03T04:05:06+0530",
        "-m",
        "authored long ago",
    ]);
    let repo_ref = repo.repo_ref();
    let log = backend
        .log(&repo_ref, &gitpoulpi_core::domain::LogOptions::default())
        .unwrap();
    assert_eq!(log.nodes[0].summary, "authored long ago");
    assert_eq!(log.nodes[0].timestamp, 981_153_306);
}
