//! History rewrite, slice S13 (the history-rewrite design,
//! "Already pushed"): pushing the rewritten refs with explicit leases to a
//! local bare remote, protected refs, the dry run, the maps and the record.
//! Expected values come from plain `git`, never from the engine under test.

use gitpoulpi_core::domain::rewrite::{
    note_refs, note_text, NoteRefKind, PushRefStatus, PushRewritten, PushRewrittenOutcome, Recipe,
    RewriteOutcome, RewritePlan, RewriteRefusal, Scope,
};
use gitpoulpi_core::GitBackend;

use super::block_on;
use super::rewrite::{alice, carol_for_alice, chain_in, git_text, refused, run};
use crate::{FixtureRepo, FixtureRepoBuilder};

fn repo_with_origin() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .with_initial()
        .unwrap()
        .with_origin()
        .unwrap()
        .build()
}

fn mark_main_protected(repo: &FixtureRepo) {
    let dir = repo.state_dir();
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("workflow.toml"),
        "[[base]]\nname = \"main\"\nprotected = true\n",
    )
    .unwrap();
}

fn all_local() -> Recipe {
    let mut recipe = carol_for_alice();
    recipe.scope = Scope::AllLocal;
    recipe
}

fn origin_rev(repo: &FixtureRepo, name: &str) -> String {
    let dir = repo.remote_path().expect("a fixture with an origin");
    repo.git(&["--git-dir", &dir.to_string_lossy(), "rev-parse", name])
}

fn origin_has(repo: &FixtureRepo, name: &str) -> bool {
    let dir = repo.remote_path().expect("a fixture with an origin");
    repo.try_git(&[
        "--git-dir",
        &dir.to_string_lossy(),
        "rev-parse",
        "--verify",
        "--quiet",
        name,
    ])
    .is_some()
}

fn local(repo: &FixtureRepo, name: &str) -> String {
    git_text(repo, &["rev-parse", name])
}

fn status_of(out: &PushRewrittenOutcome, name: &str) -> PushRefStatus {
    out.results
        .iter()
        .find(|r| r.name == name)
        .unwrap_or_else(|| panic!("no result for {name}: {out:?}"))
        .status
}

fn push(backend: &dyn GitBackend, repo: &FixtureRepo, req: &PushRewritten) -> PushRewrittenOutcome {
    block_on(backend.push_rewritten(&repo.repo_ref(), req)).expect("push_rewritten")
}

/// Two pushed branches `main` and `side` (one commit apart), a pushed tag
/// `v1` and an unpushed branch `local`, all with Alice as the author; the
/// whole thing is rewritten (Alice becomes Carol) and the run applied.
struct Pushed {
    repo: FixtureRepo,
    plan: RewritePlan,
    outcome: RewriteOutcome,
    old_main: String,
    old_side: String,
}

fn setup() -> FixtureRepo {
    let repo = repo_with_origin();
    let ids = chain_in(&repo, &[(alice(0), ""), (alice(1), "")]);
    repo.git(&["branch", "side", &ids[0]]);
    repo.git(&["tag", "v1", &ids[1]]);
    repo.git(&["push", "origin", "main", "side", "v1"]);
    repo.git(&[
        "update-ref",
        "refs/gitpoulpi/remote-tags/origin/v1",
        &ids[1],
    ]);
    repo.git(&["branch", "local", &ids[0]]);
    repo
}

fn apply(backend: &dyn GitBackend, repo: FixtureRepo, recipe: &Recipe) -> Pushed {
    let old_main = local(&repo, "main");
    let old_side = local(&repo, "side");
    let (plan, outcome) = run(backend, &repo, recipe);
    Pushed {
        repo,
        plan,
        outcome,
        old_main,
        old_side,
    }
}

fn rewritten(backend: &dyn GitBackend, recipe: &Recipe) -> Pushed {
    apply(backend, setup(), recipe)
}

/// Every pushed ref the rewrite moved goes to the remote with its lease: a
/// branch, a second branch and a tag; a local-only branch is not sent.
pub fn check_rewrite_push_sends_every_pushed_ref_with_a_lease(backend: &dyn GitBackend) {
    let f = rewritten(backend, &all_local());
    let new_main = local(&f.repo, "main");
    assert_ne!(new_main, f.old_main);
    assert_eq!(origin_rev(&f.repo, "refs/heads/main"), f.old_main);

    let out = push(backend, &f.repo, &PushRewritten::new(f.plan.run.clone()));

    assert!(out.is_clean(), "{out:?}");
    assert_eq!(out.results.len(), 3, "{out:?}");
    for name in ["refs/heads/main", "refs/heads/side", "refs/tags/v1"] {
        assert_eq!(status_of(&out, name), PushRefStatus::Pushed, "{name}");
    }
    assert_eq!(origin_rev(&f.repo, "refs/heads/main"), new_main);
    assert_eq!(
        origin_rev(&f.repo, "refs/heads/side"),
        local(&f.repo, "side")
    );
    assert_ne!(origin_rev(&f.repo, "refs/heads/side"), f.old_side);
    assert_eq!(origin_rev(&f.repo, "refs/tags/v1"), local(&f.repo, "v1"));
    assert!(
        !origin_has(&f.repo, "refs/heads/local"),
        "a branch that was never pushed stays local"
    );
    // The lease of each result is what the preview saw on the remote.
    let main = out
        .results
        .iter()
        .find(|r| r.name == "refs/heads/main")
        .unwrap();
    assert_eq!(main.expected_old.to_hex(), f.old_main);

    // A second push finds nothing to do and does not fail.
    let again = push(backend, &f.repo, &PushRewritten::new(f.plan.run.clone()));
    assert!(again.is_clean(), "{again:?}");
}

/// A remote that moved after the preview is rejected for that ref; with
/// `--atomic` nothing else is half pushed; without it only that ref stays.
pub fn check_rewrite_push_rejects_a_remote_that_moved_and_atomic_pushes_nothing(
    backend: &dyn GitBackend,
) {
    for atomic in [true, false] {
        let f = rewritten(backend, &all_local());
        let rival = collaborator_moves_main(&f);
        let mut req = PushRewritten::new(f.plan.run.clone());
        req.atomic = atomic;

        let out = push(backend, &f.repo, &req);

        assert_eq!(
            status_of(&out, "refs/heads/main"),
            PushRefStatus::RejectedRemoteMoved,
            "atomic={atomic}: {out:?}"
        );
        assert_eq!(
            origin_rev(&f.repo, "refs/heads/main"),
            rival,
            "the collaborator's commit is still there"
        );
        if atomic {
            assert_eq!(
                origin_rev(&f.repo, "refs/heads/side"),
                f.old_side,
                "nothing half pushed"
            );
            assert_eq!(status_of(&out, "refs/heads/side"), PushRefStatus::NotPushed);
            assert_eq!(status_of(&out, "refs/tags/v1"), PushRefStatus::NotPushed);
        } else {
            assert_eq!(status_of(&out, "refs/heads/side"), PushRefStatus::Pushed);
            assert_eq!(
                origin_rev(&f.repo, "refs/heads/side"),
                local(&f.repo, "side")
            );
        }
        assert!(!out.is_clean());
    }
}

/// Someone else pushes a commit on `main` after the preview: the remote's
/// `main` is no longer where the run recorded it. Returns the new remote id.
fn collaborator_moves_main(f: &Pushed) -> String {
    let tree = git_text(&f.repo, &["rev-parse", &format!("{}^{{tree}}", f.old_main)]);
    let rival = git_text(
        &f.repo,
        &[
            "commit-tree",
            &tree,
            "-p",
            &f.old_main,
            "-m",
            "collaborator",
        ],
    );
    // Objects reach the remote through a scratch ref, then main moves there
    // (not through a push of main).
    f.repo
        .git(&["push", "origin", &format!("{rival}:refs/heads/collab")]);
    let dir = f.repo.remote_path().unwrap().to_string_lossy().to_string();
    f.repo
        .git(&["--git-dir", &dir, "update-ref", "refs/heads/main", &rival]);
    // An auto-fetch learns of it: the tracking ref now says the same as the
    // remote, so a lease taken from the tracking ref would let the push
    // overwrite the collaborator. The run's recorded lease must not.
    f.repo
        .git(&["update-ref", "refs/remotes/origin/main", &rival]);
    rival
}

/// A protected pushed ref the run did rewrite is not pushed unless the
/// request lists it (or asks for all of them); the others go.
pub fn check_rewrite_push_leaves_protected_refs_out_unless_asked(backend: &dyn GitBackend) {
    let build = || {
        let repo = setup();
        mark_main_protected(&repo);
        let mut recipe = all_local();
        recipe.scope_options.include_protected = vec!["main".into()];
        apply(backend, repo, &recipe)
    };

    let f = build();
    assert!(f
        .plan
        .refs
        .iter()
        .any(|r| r.name == "refs/heads/main" && r.protected));
    let out = push(backend, &f.repo, &PushRewritten::new(f.plan.run.clone()));
    assert_eq!(
        status_of(&out, "refs/heads/main"),
        PushRefStatus::SkippedProtected
    );
    assert_eq!(status_of(&out, "refs/heads/side"), PushRefStatus::Pushed);
    assert_eq!(origin_rev(&f.repo, "refs/heads/main"), f.old_main);
    assert!(
        out.is_clean(),
        "a skipped protected ref is the user's choice"
    );

    // Asked for by name: it goes, with the same lease.
    let mut req = PushRewritten::new(f.plan.run.clone());
    req.include_protected = vec!["main".into()];
    let out = push(backend, &f.repo, &req);
    assert_eq!(status_of(&out, "refs/heads/main"), PushRefStatus::Pushed);
    assert_eq!(
        origin_rev(&f.repo, "refs/heads/main"),
        local(&f.repo, "main")
    );

    // Asked for with "all protected".
    let f = build();
    let mut req = PushRewritten::new(f.plan.run.clone());
    req.include_all_protected = true;
    let out = push(backend, &f.repo, &req);
    assert_eq!(status_of(&out, "refs/heads/main"), PushRefStatus::Pushed);

    // The ticks of a list: only the refs named are sent and answered for;
    // an unticked one is not even a row, and stays as the remote has it.
    let f = build();
    let old_tag = origin_rev(&f.repo, "refs/tags/v1");
    let mut req = PushRewritten::new(f.plan.run.clone());
    req.refs = Some(vec!["side".into()]);
    let out = push(backend, &f.repo, &req);
    assert_eq!(out.results.len(), 1, "{out:?}");
    assert_eq!(status_of(&out, "refs/heads/side"), PushRefStatus::Pushed);
    assert_eq!(
        origin_rev(&f.repo, "refs/heads/side"),
        local(&f.repo, "side")
    );
    assert_eq!(origin_rev(&f.repo, "refs/tags/v1"), old_tag);
}

/// A dry run asks the remote and sends nothing; a push of a ref that was
/// undone locally is refused rather than sent.
pub fn check_rewrite_push_dry_run_sends_nothing(backend: &dyn GitBackend) {
    let f = rewritten(backend, &all_local());
    let mut req = PushRewritten::new(f.plan.run.clone());
    req.dry_run = true;

    let out = push(backend, &f.repo, &req);

    assert!(out.dry_run);
    assert_eq!(out.count(PushRefStatus::WouldPush), 3, "{out:?}");
    assert_eq!(origin_rev(&f.repo, "refs/heads/main"), f.old_main);
    assert_eq!(origin_rev(&f.repo, "refs/heads/side"), f.old_side);
    assert!(
        f.repo
            .try_git(&["cat-file", "-e", "refs/gitpoulpi/pushed"])
            .is_none(),
        "no record of a push that did not happen"
    );
    let pushed_file = f.repo.state_dir().join("rewrite").join("pushed.json");
    assert!(!pushed_file.exists(), "a dry run leaves no push record");

    // The branch is put back by hand (what an undo does): its rewritten
    // commit is not on it any more, and the push says so instead of sending.
    f.repo.git(&["branch", "-f", "side", &f.old_side]);
    let out = push(backend, &f.repo, &PushRewritten::new(f.plan.run.clone()));
    assert_eq!(status_of(&out, "refs/heads/side"), PushRefStatus::NotPushed);
    assert_eq!(origin_rev(&f.repo, "refs/heads/side"), f.old_side);
    assert_eq!(status_of(&out, "refs/heads/main"), PushRefStatus::Pushed);
}

/// A push needs an applied run; a made-up or unapplied one is `Stale`.
pub fn check_rewrite_push_needs_an_applied_run(backend: &dyn GitBackend) {
    let fixture = setup();
    let repo = fixture.repo_ref();
    let req = PushRewritten::new(gitpoulpi_core::domain::rewrite::RunId("nope".into()));
    refused(
        block_on(backend.push_rewritten(&repo, &req)),
        RewriteRefusal::Stale,
    );

    // Previewed but not applied: nothing to push yet.
    let plan = backend
        .rewrite_preview(&repo, &all_local(), &mut |_| {
            std::ops::ControlFlow::Continue(())
        })
        .unwrap();
    let req = PushRewritten::new(plan.run.clone());
    refused(
        block_on(backend.push_rewritten(&repo, &req)),
        RewriteRefusal::Stale,
    );
    refused(
        backend.rewrite_record(&repo, &plan.run),
        RewriteRefusal::Stale,
    );
}

/// `commit-map` and `ref-map` are git-filter-repo's format: a header line,
/// then `old new` per line; a dropped commit maps to the null id. They can
/// be copied out, and the record and the note follow from the same run.
pub fn check_rewrite_maps_and_note_follow_the_applied_run(backend: &dyn GitBackend) {
    let f = rewritten(backend, &all_local());
    let repo = f.repo.repo_ref();

    let maps = backend.rewrite_maps(&repo, &f.plan.run, None).unwrap();
    assert!(!maps.copied);
    let commit_map = std::fs::read_to_string(&maps.commit_map).unwrap();
    let mut lines = commit_map.lines();
    let header: Vec<&str> = lines.next().unwrap().split_whitespace().collect();
    assert_eq!(header, ["old", "new"]);
    let pairs: Vec<(String, String)> = lines
        .map(|l| {
            let mut f = l.split(' ');
            (f.next().unwrap().to_string(), f.next().unwrap().to_string())
        })
        .collect();
    assert!(!pairs.is_empty());
    for (old, new) in &pairs {
        assert_eq!((old.len(), new.len()), (40, 40), "{old} {new}");
    }
    assert!(
        pairs.iter().any(|(old, _)| *old == f.old_main),
        "the old main tip is mapped"
    );
    let new_main = local(&f.repo, "main");
    assert!(
        pairs
            .iter()
            .any(|(old, new)| *old == f.old_main && *new == new_main),
        "and maps to the new one"
    );

    let ref_map = std::fs::read_to_string(&maps.ref_map).unwrap();
    let mut lines = ref_map.lines();
    let header: Vec<&str> = lines.next().unwrap().split_whitespace().collect();
    assert_eq!(header, ["old", "new", "ref"]);
    assert!(lines.any(|l| l == format!("{} {} refs/heads/main", f.old_main, new_main)));

    let out = tempfile::tempdir().unwrap();
    let dest = out.path().join("maps");
    let copied = backend
        .rewrite_maps(&repo, &f.plan.run, Some(&dest))
        .unwrap();
    assert!(copied.copied);
    assert_eq!(copied.dir, dest);
    assert_eq!(
        std::fs::read_to_string(dest.join("commit-map")).unwrap(),
        commit_map
    );
    assert_eq!(
        std::fs::read_to_string(dest.join("ref-map")).unwrap(),
        ref_map
    );

    let record = backend.rewrite_record(&repo, &f.plan.run).unwrap();
    assert_eq!(record.refs.len(), f.outcome.moved.len());
    let refs = note_refs(&record);
    assert_eq!(
        refs.iter()
            .filter(|r| r.kind == NoteRefKind::Branch)
            .count(),
        2,
        "the unpushed branch is not in the note: {refs:?}"
    );
    assert!(refs
        .iter()
        .any(|r| r.short == "v1" && r.kind == NoteRefKind::Tag));
    let text = note_text(&refs);
    assert!(text.contains("git reset --hard origin/main"), "{text}");
    assert!(!text.contains("Branch local"), "{text}");
}

/// After a real push the record `pushed.json` names the refs, so a later
/// undo of the rewrite can warn that the remote has the new history.
pub fn check_rewrite_push_leaves_a_record_for_the_undo_warning(backend: &dyn GitBackend) {
    let f = rewritten(backend, &all_local());
    push(backend, &f.repo, &PushRewritten::new(f.plan.run.clone()));

    let file = f.repo.state_dir().join("rewrite").join("pushed.json");
    let text = std::fs::read_to_string(&file).expect("the push record");
    let record: Vec<gitpoulpi_core::domain::rewrite::PushedRef> =
        serde_json::from_str(&text).unwrap();
    let main = record
        .iter()
        .find(|r| r.name == "refs/heads/main")
        .expect("main is recorded");
    assert_eq!(main.remote, "origin");
    assert_eq!(main.new.to_hex(), local(&f.repo, "main"));
}
