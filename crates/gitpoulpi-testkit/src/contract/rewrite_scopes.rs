//! History rewrite, slice S4 (the history-rewrite design):
//! scopes, tags, notes, linked worktrees, what is already pushed, protected
//! branches and the refusals. Same rule as `rewrite.rs`: expected values come
//! from plain `git`, never from the engine under test.

use std::collections::BTreeSet;

use gitpoulpi_core::domain::rewrite::{
    PushState, Recipe, RewriteField, RewriteRefusal, RewriteWarning, Rule, RuleKind, Scope,
};
use gitpoulpi_core::domain::{Oid, RepoRef};
use gitpoulpi_core::GitBackend;

use super::rewrite::{
    alice, bob, carol_for_alice, chain, chain_in, git_bytes, git_text, object_files, preview,
    refused, run, run_dirs,
};
use crate::{FixtureRepo, FixtureRepoBuilder};

fn alice_to_carol(scope: Scope) -> Recipe {
    let mut recipe = carol_for_alice();
    recipe.scope = scope;
    recipe
}

fn oid(hex: &str) -> Oid {
    Oid::from_hex(hex).expect("a 40 character id")
}

fn authors(repo: &FixtureRepo, rev: &str) -> Vec<String> {
    git_text(repo, &["log", "--reverse", "--format=%an", rev])
        .lines()
        .map(str::to_string)
        .collect()
}

fn rev(repo: &FixtureRepo, name: &str) -> String {
    git_text(repo, &["rev-parse", name])
}

fn four_alice_commits() -> (FixtureRepo, Vec<String>) {
    chain(&[
        (alice(0), ""),
        (alice(1), ""),
        (alice(2), ""),
        (alice(3), ""),
    ])
}

/// What a refused preview must leave alone: every ref, the object store and
/// the run directories.
fn snapshot(repo: &FixtureRepo) -> (String, Vec<String>, Vec<String>) {
    (
        git_text(repo, &["for-each-ref"]),
        object_files(repo),
        run_dirs(repo),
    )
}

fn warnings_of(plan: &gitpoulpi_core::domain::rewrite::RewritePlan) -> Vec<RewriteWarning> {
    plan.warnings.clone()
}

fn kept_old(plan: &gitpoulpi_core::domain::rewrite::RewritePlan) -> BTreeSet<String> {
    plan.warnings
        .iter()
        .filter_map(|w| match w {
            RewriteWarning::OtherBranchesKeepOldHistory { refs } => Some(refs.clone()),
            _ => None,
        })
        .flatten()
        .collect()
}

/// A selection or a range changes only its commits; the descendants are
/// replayed with new parents and keep their authors. `since` limits the walk.
pub fn check_rewrite_selection_range_and_since_scopes(backend: &dyn GitBackend) {
    // Selection: only the second commit of four is Carol's afterwards.
    let (repo, ids) = four_alice_commits();
    let before = git_text(&repo, &["rev-list", "--reverse", "main"]);
    let before: Vec<&str> = before.lines().collect();
    let (plan, _) = run(
        backend,
        &repo,
        &alice_to_carol(Scope::Selection {
            commits: vec![oid(&ids[1])],
        }),
    );
    let after = git_text(&repo, &["rev-list", "--reverse", "main"]);
    let after: Vec<&str> = after.lines().collect();
    assert_eq!(
        &after[..2],
        &before[..2],
        "init and the first commit keep ids"
    );
    assert_eq!(
        authors(&repo, "main"),
        [
            "GitPoulpi Testkit",
            "Alice Example",
            "Carol Example",
            "Alice Example",
            "Alice Example"
        ]
    );
    assert_eq!(
        rev(&repo, &format!("{}^", after[3])),
        after[2],
        "a descendant is replayed on the new parent"
    );
    assert_eq!(plan.counts.commits_changed, 3);
    assert!(plan.rows[0]
        .fields
        .iter()
        .any(|f| f.field == RewriteField::AuthorEmail));
    assert!(
        plan.rows[1].fields.is_empty() && plan.rows[2].fields.is_empty(),
        "a re-parented commit changes no field"
    );
    git_text(&repo, &["fsck", "--connectivity-only", "--no-dangling"]);

    // A selection that is not in the branch's history has nothing to do.
    let (repo, _) = four_alice_commits();
    let stranger = repo.git(&["commit-tree", "-m", "elsewhere", "HEAD^{tree}"]);
    let before = snapshot(&repo);
    refused(
        backend.rewrite_preview(
            &repo.repo_ref(),
            &alice_to_carol(Scope::Selection {
                commits: vec![oid(&stranger)],
            }),
            &mut |_| std::ops::ControlFlow::Continue(()),
        ),
        RewriteRefusal::EmptyScope,
    );
    assert_eq!(snapshot(&repo), before);

    // Range base..tip: the commits after the base up to the tip.
    let (repo, ids) = four_alice_commits();
    let (plan, _) = run(
        backend,
        &repo,
        &alice_to_carol(Scope::Range {
            base: oid(&ids[0]),
            tip: oid(&ids[2]),
        }),
    );
    assert_eq!(
        authors(&repo, "main"),
        [
            "GitPoulpi Testkit",
            "Alice Example",
            "Carol Example",
            "Carol Example",
            "Alice Example"
        ]
    );
    assert_eq!(plan.counts.commits_changed, 3);
    git_text(&repo, &["fsck", "--connectivity-only", "--no-dangling"]);

    // since: the walk stops there.
    let (repo, ids) = four_alice_commits();
    let (plan, _) = run(
        backend,
        &repo,
        &alice_to_carol(Scope::CurrentBranch {
            since: Some(oid(&ids[1])),
        }),
    );
    assert_eq!(plan.counts.commits_walked, 2);
    assert_eq!(plan.counts.commits_changed, 2);
    assert_eq!(
        authors(&repo, "main"),
        [
            "GitPoulpi Testkit",
            "Alice Example",
            "Alice Example",
            "Carol Example",
            "Carol Example"
        ]
    );
    assert_eq!(rev(&repo, "main~2"), ids[1]);
}

/// `refs` moves the named branches only (short or full names) and lists the
/// branches that keep the old history; `extra_refs` adds to any scope.
pub fn check_rewrite_refs_scope_lists_branches_that_keep_the_old_history(backend: &dyn GitBackend) {
    let (repo, ids) = chain(&[(alice(0), ""), (alice(1), "")]);
    repo.git(&["branch", "feat", "main"]);
    repo.git(&["branch", "old", &ids[0]]);
    let main_before = rev(&repo, "main");
    let old_before = rev(&repo, "old");

    let (plan, _) = run(
        backend,
        &repo,
        &alice_to_carol(Scope::Refs {
            refs: vec!["feat".into()],
        }),
    );
    assert_eq!(
        plan.refs
            .iter()
            .map(|r| r.name.as_str())
            .collect::<Vec<_>>(),
        ["refs/heads/feat"]
    );
    assert_ne!(rev(&repo, "feat"), main_before);
    assert_eq!(rev(&repo, "main"), main_before, "not in scope");
    assert_eq!(rev(&repo, "old"), old_before, "not in scope");
    assert_eq!(
        kept_old(&plan),
        BTreeSet::from(["refs/heads/main".to_string(), "refs/heads/old".to_string()])
    );
    assert_eq!(
        git_text(&repo, &["symbolic-ref", "HEAD"]),
        "refs/heads/main"
    );

    // Full names, and extra refs on top of the current branch.
    let (repo, ids) = chain(&[(alice(0), ""), (alice(1), "")]);
    repo.git(&["branch", "feat", "main"]);
    repo.git(&["branch", "old", &ids[0]]);
    let mut recipe = alice_to_carol(Scope::CurrentBranch { since: None });
    recipe.scope_options.extra_refs = vec!["refs/heads/feat".into(), "old".into()];
    let (plan, _) = run(backend, &repo, &recipe);
    let moved: BTreeSet<&str> = plan.refs.iter().map(|r| r.name.as_str()).collect();
    assert_eq!(
        moved,
        BTreeSet::from(["refs/heads/main", "refs/heads/feat", "refs/heads/old"])
    );
    assert_eq!(rev(&repo, "main"), rev(&repo, "feat"));
    assert_eq!(
        authors(&repo, "old"),
        ["GitPoulpi Testkit", "Carol Example"]
    );
    assert!(kept_old(&plan).is_empty());
    git_text(&repo, &["fsck", "--connectivity-only", "--no-dangling"]);

    // A name that is no branch or tag is an empty scope, nothing changes.
    let before = snapshot(&repo);
    refused(
        backend.rewrite_preview(
            &repo.repo_ref(),
            &alice_to_carol(Scope::Refs {
                refs: vec!["nosuchbranch".into()],
            }),
            &mut |_| std::ops::ControlFlow::Continue(()),
        ),
        RewriteRefusal::EmptyScope,
    );
    assert_eq!(snapshot(&repo), before);
}

/// `all-local` moves every branch and tag, keeps HEAD on its branch.
pub fn check_rewrite_all_local_moves_every_branch_and_tag(backend: &dyn GitBackend) {
    let (repo, ids) = chain(&[(alice(0), ""), (alice(1), "")]);
    repo.git(&["branch", "feat", "main"]);
    repo.git(&["tag", "v1", &ids[0]]);
    let (plan, _) = run(backend, &repo, &alice_to_carol(Scope::AllLocal));
    let moved: BTreeSet<&str> = plan.refs.iter().map(|r| r.name.as_str()).collect();
    assert_eq!(
        moved,
        BTreeSet::from(["refs/heads/main", "refs/heads/feat", "refs/tags/v1"])
    );
    assert_eq!(plan.counts.refs_moved, 2);
    assert_eq!(plan.counts.tags_moved, 1);
    assert_eq!(rev(&repo, "main"), rev(&repo, "feat"));
    assert_eq!(rev(&repo, "v1"), rev(&repo, "main~"));
    assert_eq!(
        git_text(&repo, &["symbolic-ref", "HEAD"]),
        "refs/heads/main"
    );
    assert!(kept_old(&plan).is_empty());
    git_text(&repo, &["fsck", "--connectivity-only", "--no-dangling"]);
}

/// Lightweight tags move, annotated tags are re-created with the same tagger
/// and message (a tag of a tag follows its target), a tag signature is dropped
/// and counted, tags on untouched commits stay, `include_tags = false` leaves
/// them all.
pub fn check_rewrite_moves_lightweight_and_annotated_tags(backend: &dyn GitBackend) {
    let (repo, ids) = chain(&[(alice(0), ""), (alice(1), ""), (alice(2), "")]);
    let init = rev(&repo, &format!("{}^", ids[0]));
    repo.git(&["tag", "light", &ids[0]]);
    repo.git(&["tag", "stay", &init]);
    repo.git(&["tag", "-a", "ann", "-m", "annotated message", &ids[1]]);
    let ann_old = rev(&repo, "ann");
    repo.git(&["tag", "-a", "outer", "-m", "outer message", "refs/tags/ann"]);
    let outer_old = rev(&repo, "outer");
    assert!(
        git_text(&repo, &["cat-file", "-p", "outer"])
            .contains(&format!("object {ann_old}\ntype tag")),
        "the fixture must hold a tag of a tag"
    );
    let signed_body = format!(
        "object {}\ntype commit\ntag signed\ntagger T Agger <t@example.com> 100 +0000\n\nsigned message\n-----BEGIN PGP SIGNATURE-----\n\niQEzBAAB\n-----END PGP SIGNATURE-----\n",
        ids[2]
    );
    let signed = String::from_utf8(git_bytes(
        &repo,
        &["hash-object", "-t", "tag", "-w", "--stdin", "--literally"],
        &[],
        signed_body.as_bytes(),
    ))
    .unwrap()
    .trim()
    .to_string();
    repo.git(&["update-ref", "refs/tags/signed", &signed]);
    let tagger_old = git_text(
        &repo,
        &[
            "for-each-ref",
            "--format=%(taggername) %(taggeremail) %(taggerdate:raw)",
            "refs/tags/ann",
        ],
    );

    let (plan, _) = run(backend, &repo, &carol_for_alice());

    assert_eq!(rev(&repo, "light"), rev(&repo, "main~2"));
    assert_eq!(rev(&repo, "stay"), init);
    let ann_new = rev(&repo, "ann");
    assert_ne!(ann_new, ann_old);
    assert_eq!(git_text(&repo, &["cat-file", "-t", "ann"]), "tag");
    assert_eq!(rev(&repo, "ann^{commit}"), rev(&repo, "main~1"));
    assert_eq!(
        git_text(
            &repo,
            &[
                "for-each-ref",
                "--format=%(taggername) %(taggeremail) %(taggerdate:raw)",
                "refs/tags/ann"
            ]
        ),
        tagger_old,
        "tagger and date are kept"
    );
    assert_eq!(
        git_text(
            &repo,
            &["for-each-ref", "--format=%(contents)", "refs/tags/ann"]
        ),
        "annotated message"
    );
    // The tag of a tag points at the new inner tag object.
    assert_ne!(rev(&repo, "outer"), outer_old);
    let outer = git_text(&repo, &["cat-file", "-p", "outer"]);
    assert!(
        outer.contains(&format!("object {ann_new}\ntype tag")),
        "{outer}"
    );
    assert_eq!(rev(&repo, "outer^{commit}"), rev(&repo, "main~1"));
    // The signature is gone, the message is not.
    let signed_new = git_text(&repo, &["cat-file", "-p", "signed"]);
    assert!(signed_new.contains("signed message"));
    assert!(!signed_new.contains("BEGIN PGP"), "{signed_new}");
    assert_eq!(plan.counts.tags_moved, 4);
    assert!(warnings_of(&plan)
        .iter()
        .any(|w| matches!(w, RewriteWarning::SignedTagsDropped { count: 1 })));
    git_text(&repo, &["fsck", "--connectivity-only", "--no-dangling"]);

    // include_tags = false: no tag moves.
    let (repo, ids) = chain(&[(alice(0), "")]);
    repo.git(&["tag", "light", &ids[0]]);
    repo.git(&["tag", "-a", "ann", "-m", "m", &ids[0]]);
    let (light, ann) = (rev(&repo, "light"), rev(&repo, "ann"));
    let mut recipe = carol_for_alice();
    recipe.scope_options.include_tags = false;
    let (plan, _) = run(backend, &repo, &recipe);
    assert_eq!(rev(&repo, "light"), light);
    assert_eq!(rev(&repo, "ann"), ann);
    assert_eq!(plan.counts.tags_moved, 0);
}

/// Notes follow their commits in the same transaction, notes on untouched
/// commits stay.
pub fn check_rewrite_notes_follow_the_rewritten_commits(backend: &dyn GitBackend) {
    let (repo, ids) = chain(&[(alice(0), ""), (alice(1), "")]);
    let init = rev(&repo, &format!("{}^", ids[0]));
    repo.git(&["notes", "add", "-m", "on init", &init]);
    repo.git(&["notes", "add", "-m", "on zero", &ids[0]]);
    repo.git(&["notes", "add", "-m", "on one", &ids[1]]);
    let notes_before = rev(&repo, "refs/notes/commits");

    let (plan, _) = run(backend, &repo, &carol_for_alice());

    let (zero, one) = (rev(&repo, "main~1"), rev(&repo, "main"));
    assert_eq!(git_text(&repo, &["notes", "show", &zero]), "on zero");
    assert_eq!(git_text(&repo, &["notes", "show", &one]), "on one");
    assert_eq!(git_text(&repo, &["notes", "show", &init]), "on init");
    let listed: BTreeSet<String> = git_text(&repo, &["notes", "list"])
        .lines()
        .map(|l| l.split(' ').nth(1).unwrap().to_string())
        .collect();
    assert_eq!(listed, BTreeSet::from([init, zero, one]));
    assert_ne!(rev(&repo, "refs/notes/commits"), notes_before);
    assert!(plan
        .refs
        .iter()
        .any(|r| r.name == "refs/notes/commits" && r.old.to_hex() == notes_before));
    git_text(&repo, &["fsck", "--connectivity-only", "--no-dangling"]);

    // Nothing rewritten, no notes move.
    let (repo, ids) = chain(&[(bob(0), "")]);
    repo.git(&["notes", "add", "-m", "n", &ids[0]]);
    let notes = rev(&repo, "refs/notes/commits");
    let (plan, _) = run(backend, &repo, &carol_for_alice());
    assert!(plan.refs.is_empty());
    assert_eq!(rev(&repo, "refs/notes/commits"), notes);
}

/// A branch checked out in a linked worktree moves (metadata only: same
/// trees), is reported with that worktree, and the worktree stays clean. The
/// rewrite also runs from the linked worktree itself.
pub fn check_rewrite_moves_a_branch_checked_out_in_a_linked_worktree(backend: &dyn GitBackend) {
    let (repo, _) = chain(&[(alice(0), ""), (alice(1), "")]);
    let linked = repo.root().with_file_name("linked-side");
    repo.git(&[
        "worktree",
        "add",
        "-b",
        "side",
        &linked.to_string_lossy(),
        "main",
    ]);

    let (plan, _) = run(backend, &repo, &alice_to_carol(Scope::AllLocal));
    let side = plan
        .refs
        .iter()
        .find(|r| r.name == "refs/heads/side")
        .expect("side moves");
    assert_eq!(
        side.checked_out_in
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str()),
        Some("linked-side")
    );
    let main = plan
        .refs
        .iter()
        .find(|r| r.name == "refs/heads/main")
        .unwrap();
    assert_eq!(main.checked_out_in, None);
    assert_eq!(rev(&repo, "side"), rev(&repo, "main"));
    assert_eq!(
        repo.git_at(&linked, &["rev-parse", "HEAD"]),
        rev(&repo, "main")
    );
    assert_eq!(repo.git_at(&linked, &["status", "--porcelain"]), "");
    git_text(&repo, &["fsck", "--connectivity-only", "--no-dangling"]);

    // From the linked worktree: its branch moves, main keeps the old history.
    let (repo, _) = chain(&[(alice(0), ""), (alice(1), "")]);
    let linked = repo.root().with_file_name("linked-side");
    repo.git(&[
        "worktree",
        "add",
        "-b",
        "side",
        &linked.to_string_lossy(),
        "main",
    ]);
    let main_before = rev(&repo, "main");
    let from_linked = RepoRef::from_root(&linked);
    let plan = backend
        .rewrite_preview(&from_linked, &carol_for_alice(), &mut |_| {
            std::ops::ControlFlow::Continue(())
        })
        .expect("preview from a linked worktree");
    backend.rewrite_apply(&from_linked, &plan).expect("apply");
    assert_ne!(rev(&repo, "side"), main_before);
    assert_eq!(rev(&repo, "main"), main_before);
    assert_eq!(repo.git_at(&linked, &["status", "--porcelain"]), "");
    assert!(kept_old(&plan).contains("refs/heads/main"));
}

fn repo_with_origin() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .with_initial()
        .unwrap()
        .with_origin()
        .unwrap()
        .build()
}

/// What is on a remote is told per commit and per ref; a tag is pushed when
/// GitPoulpi's mirror says so; a repository without a remote is all local.
pub fn check_rewrite_classifies_what_is_already_pushed(backend: &dyn GitBackend) {
    let repo = repo_with_origin();
    let ids = chain_in(&repo, &[(alice(0), ""), (alice(1), "")]);
    repo.git(&["push", "origin", "main"]);
    assert_eq!(rev(&repo, "origin/main"), ids[1]);
    let more = chain_in(&repo, &[(alice(2), "")]);
    repo.git(&["tag", "v1", &ids[1]]);
    repo.git(&["tag", "v2", &more[0]]);
    repo.git(&[
        "update-ref",
        "refs/gitpoulpi/remote-tags/origin/v1",
        &ids[1],
    ]);

    let plan = preview(backend, &repo, &carol_for_alice());

    assert_eq!(plan.rows.len(), 3);
    assert_eq!(
        plan.rows.iter().map(|r| r.pushed).collect::<Vec<_>>(),
        [true, true, false]
    );
    let by_name = |name: &str| plan.refs.iter().find(|r| r.name == name).unwrap();
    assert_eq!(
        by_name("refs/heads/main").pushed,
        PushState::OnRemote {
            remote: "origin".into(),
            remote_oid: oid(&ids[1]),
            rewritten_on_remote: 2,
        }
    );
    assert_eq!(
        by_name("refs/tags/v1").pushed,
        PushState::OnRemote {
            remote: "origin".into(),
            remote_oid: oid(&ids[1]),
            rewritten_on_remote: 1,
        }
    );
    assert_eq!(by_name("refs/tags/v2").pushed, PushState::LocalOnly);
    assert!(warnings_of(&plan)
        .iter()
        .any(|w| matches!(w, RewriteWarning::PushedRefs { count: 2 })));
    backend
        .rewrite_discard(&repo.repo_ref(), &plan.run)
        .unwrap();

    // Only unpushed commits: the remote's commits are left out of the walk.
    let plan = preview(
        backend,
        &repo,
        &alice_to_carol(Scope::CurrentBranch {
            since: Some(oid(&ids[1])),
        }),
    );
    assert_eq!(plan.rows.len(), 1);
    assert!(!plan.rows[0].pushed);
    assert_eq!(
        by_name_of(&plan, "refs/heads/main").pushed,
        PushState::LocalOnly
    );
    backend
        .rewrite_discard(&repo.repo_ref(), &plan.run)
        .unwrap();

    // No remote at all.
    let (repo, _) = chain(&[(alice(0), "")]);
    let plan = preview(backend, &repo, &carol_for_alice());
    assert!(plan.rows.iter().all(|r| !r.pushed));
    assert_eq!(plan.refs[0].pushed, PushState::LocalOnly);
    assert!(!warnings_of(&plan)
        .iter()
        .any(|w| matches!(w, RewriteWarning::PushedRefs { .. })));
}

fn by_name_of<'a>(
    plan: &'a gitpoulpi_core::domain::rewrite::RewritePlan,
    name: &str,
) -> &'a gitpoulpi_core::domain::rewrite::RefChange {
    plan.refs.iter().find(|r| r.name == name).unwrap()
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

/// A pushed protected base is left out unless it is listed in
/// `include_protected`; when it is the whole scope the preview is refused;
/// an unpushed protected base is no problem.
pub fn check_rewrite_leaves_pushed_protected_branches_out_unless_listed(backend: &dyn GitBackend) {
    let setup = || {
        let repo = repo_with_origin();
        let ids = chain_in(&repo, &[(alice(0), ""), (alice(1), "")]);
        repo.git(&["push", "origin", "main"]);
        repo.git(&["branch", "feat", "main"]);
        mark_main_protected(&repo);
        (repo, ids)
    };

    // All local branches: main stays, feat moves, both are reported.
    let (repo, _) = setup();
    let main_before = rev(&repo, "main");
    let (plan, _) = run(backend, &repo, &alice_to_carol(Scope::AllLocal));
    assert_eq!(rev(&repo, "main"), main_before);
    assert_ne!(rev(&repo, "feat"), main_before);
    assert!(plan.refs.iter().all(|r| r.name != "refs/heads/main"));
    assert!(warnings_of(&plan).iter().any(|w| matches!(
        w,
        RewriteWarning::ProtectedExcluded { refs } if refs == &["refs/heads/main".to_string()]
    )));
    assert!(kept_old(&plan).contains("refs/heads/main"));
    git_text(&repo, &["fsck", "--connectivity-only", "--no-dangling"]);

    // Listed: it moves like the others.
    let (repo, _) = setup();
    let mut recipe = alice_to_carol(Scope::AllLocal);
    recipe.scope_options.include_protected = vec!["main".into()];
    let (plan, _) = run(backend, &repo, &recipe);
    assert_eq!(rev(&repo, "main"), rev(&repo, "feat"));
    assert!(plan
        .refs
        .iter()
        .any(|r| r.name == "refs/heads/main" && r.protected));
    assert!(!warnings_of(&plan)
        .iter()
        .any(|w| matches!(w, RewriteWarning::ProtectedExcluded { .. })));

    // The current branch is the protected, pushed one: refused, nothing moves.
    let (repo, ids) = setup();
    let before = snapshot(&repo);
    refused(
        backend.rewrite_preview(&repo.repo_ref(), &carol_for_alice(), &mut |_| {
            std::ops::ControlFlow::Continue(())
        }),
        RewriteRefusal::ProtectedRef,
    );
    assert_eq!(snapshot(&repo), before);

    // Only the unpushed commits of it: nothing pushed is touched.
    chain_in(&repo, &[(alice(5), "")]);
    let (plan, _) = run(
        backend,
        &repo,
        &alice_to_carol(Scope::CurrentBranch {
            since: Some(oid(&ids[1])),
        }),
    );
    assert_eq!(plan.counts.commits_changed, 1);
    assert_eq!(rev(&repo, "main~"), ids[1]);

    // Not pushed at all: protected or not, it moves.
    let (repo, _) = chain(&[(alice(0), "")]);
    mark_main_protected(&repo);
    let (plan, _) = run(backend, &repo, &carol_for_alice());
    assert_eq!(plan.refs.len(), 1);
}

/// Stashes, remote-tracking refs and GitPoulpi's own refs are never moved;
/// a stash on rewritten commits is reported.
pub fn check_rewrite_never_moves_stashes_remote_tracking_or_gitpoulpi_refs(
    backend: &dyn GitBackend,
) {
    let repo = repo_with_origin();
    let ids = chain_in(&repo, &[(alice(0), ""), (alice(1), "")]);
    repo.git(&["push", "origin", "main"]);
    repo.git(&["update-ref", "refs/gitpoulpi/pin", &ids[1]]);
    let tracked = repo.git(&["ls-files"]);
    let file = tracked.lines().next().unwrap();
    std::fs::write(repo.root().join(file), "stashed edit\n").unwrap();
    repo.git(&["stash"]);
    let (stash, tracking, pin) = (
        rev(&repo, "refs/stash"),
        rev(&repo, "refs/remotes/origin/main"),
        rev(&repo, "refs/gitpoulpi/pin"),
    );
    let mut recipe = alice_to_carol(Scope::AllLocal);
    recipe.scope_options.include_protected = vec!["main".into()];

    let (plan, _) = run(backend, &repo, &recipe);

    assert_eq!(rev(&repo, "refs/stash"), stash);
    assert_eq!(rev(&repo, "refs/remotes/origin/main"), tracking);
    assert_eq!(rev(&repo, "refs/gitpoulpi/pin"), pin);
    assert!(plan.refs.iter().all(|r| r.name.starts_with("refs/heads/")
        || r.name.starts_with("refs/tags/")
        || r.name.starts_with("refs/notes/")));
    assert!(warnings_of(&plan)
        .iter()
        .any(|w| matches!(w, RewriteWarning::StashesKeepOldHistory { count: 1 })));
    assert_eq!(
        repo.git(&["log", "-1", "--format=%an", "refs/stash^"]),
        "Alice Example",
        "the stash still sits on the old history"
    );
}

/// Every refusal is typed and changes nothing: no ref, no object, no run.
pub fn check_rewrite_scope_refusals_are_typed_and_change_nothing(backend: &dyn GitBackend) {
    let try_preview = |repo: &FixtureRepo, recipe: &Recipe| {
        backend.rewrite_preview(&repo.repo_ref(), recipe, &mut |_| {
            std::ops::ControlFlow::Continue(())
        })
    };

    // Shallow.
    let (repo, ids) = chain(&[(alice(0), ""), (alice(1), "")]);
    std::fs::write(repo.root().join(".git/shallow"), format!("{}\n", ids[0])).unwrap();
    let before = snapshot(&repo);
    refused(
        try_preview(&repo, &carol_for_alice()),
        RewriteRefusal::Shallow,
    );
    refused(
        try_preview(&repo, &alice_to_carol(Scope::AllLocal)),
        RewriteRefusal::Shallow,
    );
    assert_eq!(snapshot(&repo), before);

    // An operation in progress, of each kind the markers know.
    let (repo, ids) = chain(&[(alice(0), "")]);
    for marker in ["MERGE_HEAD", "CHERRY_PICK_HEAD", "REVERT_HEAD"] {
        let path = repo.root().join(".git").join(marker);
        std::fs::write(&path, format!("{}\n", ids[0])).unwrap();
        let before = snapshot(&repo);
        refused(
            try_preview(&repo, &alice_to_carol(Scope::AllLocal)),
            RewriteRefusal::OperationInProgress,
        );
        assert_eq!(snapshot(&repo), before, "{marker}");
        std::fs::remove_file(&path).unwrap();
    }
    std::fs::create_dir(repo.root().join(".git/rebase-merge")).unwrap();
    refused(
        try_preview(&repo, &carol_for_alice()),
        RewriteRefusal::OperationInProgress,
    );
    std::fs::remove_dir(repo.root().join(".git/rebase-merge")).unwrap();

    // A partial clone, for a rule that reads file content.
    let (repo, _) = chain(&[(alice(0), "")]);
    repo.git(&["config", "extensions.partialclone", "origin"]);
    repo.git(&["config", "remote.origin.promisor", "true"]);
    let content = Recipe::new(
        "drop",
        Scope::default(),
        vec![Rule::new(RuleKind::RemovePaths {
            globs: vec!["secrets/**".into()],
        })],
    );
    let before = snapshot(&repo);
    refused(try_preview(&repo, &content), RewriteRefusal::PartialClone);
    assert_eq!(snapshot(&repo), before);
    // Metadata rules are fine in a partial clone: every commit is present.
    let (plan, _) = run(backend, &repo, &carol_for_alice());
    assert_eq!(plan.counts.commits_changed, 1);

    // SHA-256 (skipped when this git cannot make such a repository).
    let dir = tempfile::tempdir().unwrap();
    let init = std::process::Command::new("git")
        .args(["init", "-q", "--object-format=sha256"])
        .arg(dir.path())
        .output()
        .unwrap();
    if init.status.success() {
        let commit = std::process::Command::new("git")
            .current_dir(dir.path())
            .args([
                "-c",
                "user.name=T",
                "-c",
                "user.email=t@example.com",
                "commit",
                "-q",
                "--allow-empty",
                "-m",
                "x",
            ])
            .output()
            .unwrap();
        assert!(commit.status.success());
        let r = RepoRef::from_root(dir.path());
        let result = backend.rewrite_preview(&r, &carol_for_alice(), &mut |_| {
            std::ops::ControlFlow::Continue(())
        });
        refused(result, RewriteRefusal::UnsupportedObjectFormat);
        assert!(!dir.path().join(".git/gitpoulpi").exists());
    }
}

/// An annotated tag by `tagger` on `target`, written with `hash-object`.
fn tag_by(repo: &FixtureRepo, name: &str, target: &str, tagger: &str) {
    let body =
        format!("object {target}\ntype commit\ntag {name}\ntagger {tagger}\n\nmessage of {name}\n");
    let id = String::from_utf8(git_bytes(
        repo,
        &["hash-object", "-t", "tag", "-w", "--stdin", "--literally"],
        &[],
        body.as_bytes(),
    ))
    .unwrap()
    .trim()
    .to_string();
    repo.git(&["update-ref", &format!("refs/tags/{name}"), &id]);
}

fn tagger_of(repo: &FixtureRepo, name: &str) -> String {
    git_text(
        repo,
        &[
            "for-each-ref",
            "--format=%(taggername) %(taggeremail) %(taggerdate:raw)",
            &format!("refs/tags/{name}"),
        ],
    )
}

/// `include_taggers` (off by default) makes identity and anonymise rules also
/// rewrite the tagger of the annotated tags that are re-created; the date
/// stays, a tagger the rule does not match stays, and an anonymised tagger gets
/// the same placeholder as the same person's commits.
pub fn check_rewrite_taggers_follow_identity_rules_only_when_asked(backend: &dyn GitBackend) {
    use gitpoulpi_core::domain::rewrite::AnonymiseRule;
    let fixture = || {
        let (repo, ids) = chain(&[(alice(0), ""), (alice(1), "")]);
        tag_by(
            &repo,
            "by-alice",
            &ids[0],
            "Alice Example <alice@example.com> 100 +0300",
        );
        tag_by(
            &repo,
            "by-bob",
            &ids[1],
            "Bob Example <bob@example.com> 200 -0100",
        );
        repo
    };
    let set_taggers = |recipe: &mut Recipe| match &mut recipe.rules[0].kind {
        RuleKind::Identity(rule) => rule.include_taggers = true,
        RuleKind::Anonymise(rule) => rule.include_taggers = true,
        _ => unreachable!(),
    };
    let alice_tag = "Alice Example <alice@example.com> 100 +0300";
    let bob_tag = "Bob Example <bob@example.com> 200 -0100";

    // Off by default: the commits change, the taggers do not.
    let repo = fixture();
    run(backend, &repo, &carol_for_alice());
    assert_eq!(tagger_of(&repo, "by-alice"), alice_tag);
    assert_eq!(tagger_of(&repo, "by-bob"), bob_tag);
    assert_eq!(
        git_text(&repo, &["log", "-1", "--format=%an", "by-alice^{commit}"]),
        "Carol Example"
    );

    // On: Alice's tagger becomes Carol, date and offset kept; Bob stays.
    let repo = fixture();
    let mut recipe = carol_for_alice();
    set_taggers(&mut recipe);
    run(backend, &repo, &recipe);
    assert_eq!(
        tagger_of(&repo, "by-alice"),
        "Carol Example <carol@example.com> 100 +0300"
    );
    assert_eq!(tagger_of(&repo, "by-bob"), bob_tag);
    assert_eq!(
        git_text(
            &repo,
            &["for-each-ref", "--format=%(contents)", "refs/tags/by-alice"]
        ),
        "message of by-alice"
    );
    git_text(&repo, &["fsck", "--connectivity-only", "--no-dangling"]);

    // Anonymise: Alice (author and tagger) is one contributor, Bob another.
    let repo = fixture();
    let mut recipe = Recipe::new(
        "anonymise",
        Scope::default(),
        vec![Rule::new(RuleKind::Anonymise(AnonymiseRule::default()))],
    );
    recipe.options.signatures = carol_for_alice().options.signatures;
    set_taggers(&mut recipe);
    run(backend, &repo, &recipe);
    let author = git_text(
        &repo,
        &["log", "-1", "--format=%an <%ae>", "by-alice^{commit}"],
    );
    assert!(author.starts_with("Contributor "), "{author}");
    let tagger = tagger_of(&repo, "by-alice");
    assert!(tagger.starts_with(&author), "{tagger} vs {author}");
    assert!(tagger.ends_with(" 100 +0300"), "{tagger}");
    let bob = tagger_of(&repo, "by-bob");
    assert!(
        bob.starts_with("Contributor ") && bob.ends_with(" 200 -0100"),
        "{bob}"
    );
    assert!(!bob.starts_with(&author), "{bob}");
}
