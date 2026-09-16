//! History rewrite, slice S19 (the history-rewrite design):
//! re-signing and stripping signatures. Signatures are made by the
//! repository's own setup: `gpg.format=ssh` and a generated ed25519 key, and
//! checked by `git verify-commit`. Skipped without `ssh-keygen`, like
//! split's signing check.

use std::ops::ControlFlow;
use std::path::PathBuf;

use gitpoulpi_core::domain::rewrite::{
    RewritePhase, RewriteRefusal, Rule, RuleKind, SignaturePolicy,
};
use gitpoulpi_core::{Error, GitBackend};

use super::rewrite::{carol_for_alice, git_text, history, object_files, refused, run, run_dirs};
use crate::{FixtureRepo, FixtureRepoBuilder};

/// A generated key and the directory that keeps it alive.
struct Key {
    _dir: tempfile::TempDir,
    private: PathBuf,
    allowed: PathBuf,
}

fn generate_key() -> Option<Key> {
    let dir = tempfile::tempdir().unwrap();
    let private = dir.path().join("key");
    let made = std::process::Command::new("ssh-keygen")
        .args(["-q", "-t", "ed25519", "-N", "", "-C", "rewrite-test", "-f"])
        .arg(&private)
        .output();
    if !matches!(&made, Ok(out) if out.status.success()) {
        eprintln!("SKIPPED: ssh-keygen is not available");
        return None;
    }
    let public = std::fs::read_to_string(dir.path().join("key.pub")).unwrap();
    let allowed = dir.path().join("allowed_signers");
    std::fs::write(&allowed, format!("* {public}")).unwrap();
    Some(Key {
        _dir: dir,
        private,
        allowed,
    })
}

fn use_key(repo: &FixtureRepo, key: &Key) {
    repo.git(&["config", "gpg.format", "ssh"]);
    repo.git(&["config", "user.signingkey", key.private.to_str().unwrap()]);
    repo.git(&[
        "config",
        "gpg.ssh.allowedSignersFile",
        key.allowed.to_str().unwrap(),
    ]);
}

/// A repository whose history is the initial commit, then one commit per
/// entry: `(author is alice, signed)`. Returns the ids, oldest first.
fn history_of(key: &Key, commits: &[(bool, bool)]) -> (FixtureRepo, Vec<String>) {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .with_initial()
        .unwrap()
        .build();
    use_key(&repo, key);
    let mut ids = Vec::new();
    for (i, (alice, signed)) in commits.iter().enumerate() {
        let author = if *alice {
            "Alice Example <alice@example.com>"
        } else {
            "Bob Example <bob@example.com>"
        };
        let message = format!("commit {i}");
        repo.git(&[
            "commit",
            "-q",
            "--allow-empty",
            "--author",
            author,
            if *signed { "-S" } else { "--no-gpg-sign" },
            "-m",
            &message,
        ]);
        ids.push(git_text(&repo, &["rev-parse", "HEAD"]));
    }
    (repo, ids)
}

fn verifies(repo: &FixtureRepo, id: &str) -> bool {
    repo.try_git(&["verify-commit", id]).is_some()
}

fn is_signed(repo: &FixtureRepo, id: &str) -> bool {
    git_text(repo, &["cat-file", "commit", id]).contains("\ngpgsig ")
}

fn recipe(policy: Option<SignaturePolicy>) -> gitpoulpi_core::domain::rewrite::Recipe {
    let mut recipe = carol_for_alice();
    recipe.options.signatures = policy;
    recipe
}

/// "Re-sign signed" signs only the rewritten commits that were signed; the
/// unchanged prefix keeps its ids and its signatures.
pub fn check_rewrite_resign_signed_signs_only_the_signed_ones(backend: &dyn GitBackend) {
    let Some(key) = generate_key() else { return };
    // bob (signed, untouched), then alice: signed, plain, signed.
    let (repo, ids) = history_of(
        &key,
        &[(false, true), (true, true), (true, false), (true, true)],
    );
    let (plan, _) = run(backend, &repo, &recipe(Some(SignaturePolicy::ResignSigned)));
    let after = history(&repo);
    assert_eq!(after[1], ids[0], "the untouched prefix keeps its id");
    assert!(verifies(&repo, &after[1]));
    assert!(verifies(&repo, &after[2]), "a rewritten signed commit");
    assert!(
        !is_signed(&repo, &after[3]),
        "a commit that was plain stays plain"
    );
    assert!(verifies(&repo, &after[4]));
    assert_eq!(plan.counts.commits_changed, 3);
    assert_eq!(plan.counts.signed_changed, 2);
    assert_eq!(plan.counts.resigned, 2);
    assert_eq!(
        git_text(&repo, &["log", "--format=%an", "-1", &after[4]]),
        "Carol Example"
    );
}

/// "Re-sign all" signs every rewritten commit, the plain ones too.
pub fn check_rewrite_resign_all_signs_every_rewritten_commit(backend: &dyn GitBackend) {
    let Some(key) = generate_key() else { return };
    let (repo, ids) = history_of(&key, &[(false, false), (true, true), (true, false)]);
    let (plan, _) = run(backend, &repo, &recipe(Some(SignaturePolicy::ResignAll)));
    let after = history(&repo);
    assert_eq!(after[1], ids[0]);
    assert!(
        !is_signed(&repo, &after[1]),
        "an unchanged commit is not signed"
    );
    for new in &after[2..] {
        assert!(verifies(&repo, new), "{new} is not signed");
    }
    assert_eq!(plan.counts.resigned, 2);
    assert!(plan.counts.signing_millis > 0 || cfg!(windows));
    // A number for the perf notes (`--nocapture`): ssh-keygen spawn per commit.
    eprintln!(
        "signing: {} ms for {} signatures",
        plan.counts.signing_millis, plan.counts.resigned
    );
}

/// "Strip" removes the signatures of rewritten commits and signs nothing.
pub fn check_rewrite_strip_removes_signatures_and_signs_nothing(backend: &dyn GitBackend) {
    let Some(key) = generate_key() else { return };
    let (repo, ids) = history_of(&key, &[(false, true), (true, true)]);
    let (plan, _) = run(backend, &repo, &recipe(Some(SignaturePolicy::Strip)));
    let after = history(&repo);
    assert_eq!(after[1], ids[0]);
    assert!(verifies(&repo, &after[1]));
    assert!(!is_signed(&repo, &after[2]));
    assert_eq!(plan.counts.resigned, 0);
    assert_eq!(plan.counts.signed_changed, 1);
}

/// The smoke test: a working setup answers with a time, a broken key is a
/// typed refusal before anything is walked, and nothing is left behind.
pub fn check_rewrite_signing_smoke_test_refuses_a_broken_key(backend: &dyn GitBackend) {
    let Some(key) = generate_key() else { return };
    let (repo, _) = history_of(&key, &[(true, true), (true, false)]);
    let ok = backend.rewrite_check_signing(&repo.repo_ref()).unwrap();
    assert_eq!(ok.reason, None, "{}", ok.detail);
    assert!(ok.millis > 0);
    assert_eq!(
        ok.format,
        gitpoulpi_core::domain::rewrite::SigningFormat::Ssh
    );

    repo.git(&[
        "config",
        "user.signingkey",
        repo.root().join("no-such-key").to_str().unwrap(),
    ]);
    let started = std::time::Instant::now();
    let broken = backend.rewrite_check_signing(&repo.repo_ref()).unwrap();
    // Windows' OpenSSH ssh-keygen never exits on a missing key when its
    // output is a pipe: the refusal then only came after the 120 s timeout.
    assert!(
        started.elapsed() < std::time::Duration::from_secs(30),
        "a missing key is refused at once, not after a timeout"
    );
    assert_eq!(broken.reason, Some(RewriteRefusal::SigningFailed));
    assert!(!broken.detail.is_empty());

    let before = (
        git_text(&repo, &["for-each-ref"]),
        object_files(&repo),
        run_dirs(&repo),
    );
    for policy in [SignaturePolicy::ResignSigned, SignaturePolicy::ResignAll] {
        let result = backend.rewrite_preview(&repo.repo_ref(), &recipe(Some(policy)), &mut |_| {
            ControlFlow::Continue(())
        });
        refused(result, RewriteRefusal::SigningFailed);
    }
    // Early: even where no rewritten commit would need a signature, a run
    // that re-signs starts only with a setup that works.
    let (plain, _) = history_of(&key, &[(true, false)]);
    plain.git(&[
        "config",
        "user.signingkey",
        plain.root().join("no-such-key").to_str().unwrap(),
    ]);
    refused(
        backend.rewrite_preview(
            &plain.repo_ref(),
            &recipe(Some(SignaturePolicy::ResignSigned)),
            &mut |_| ControlFlow::Continue(()),
        ),
        RewriteRefusal::SigningFailed,
    );
    // Stripping needs no signature, so a broken key does not matter.
    backend
        .rewrite_preview(
            &repo.repo_ref(),
            &recipe(Some(SignaturePolicy::Strip)),
            &mut |_| ControlFlow::Continue(()),
        )
        .unwrap();
    let after = (
        git_text(&repo, &["for-each-ref"]),
        object_files(&repo),
        run_dirs(&repo),
    );
    assert_eq!(before.0, after.0);
    assert_eq!(before.1, after.1);
    assert_eq!(before.2.len(), 0);
    assert_eq!(after.2.len(), 1, "only the strip preview keeps its run");
}

/// Cancelling between two signatures leaves no ref, no object and no run
/// directory behind, and the progress counted the signed commits.
pub fn check_rewrite_cancel_between_signatures_leaves_nothing(backend: &dyn GitBackend) {
    let Some(key) = generate_key() else { return };
    let (repo, _) = history_of(&key, &[(true, true), (true, true), (true, true)]);
    let before = (
        git_text(&repo, &["for-each-ref"]),
        object_files(&repo),
        run_dirs(&repo),
    );
    let mut seen = Vec::new();
    let result = backend.rewrite_preview(
        &repo.repo_ref(),
        &recipe(Some(SignaturePolicy::ResignSigned)),
        &mut |p| {
            if p.phase == RewritePhase::Signing {
                seen.push(p.done);
                if p.done >= 1 {
                    return ControlFlow::Break(());
                }
            }
            ControlFlow::Continue(())
        },
    );
    assert!(matches!(result, Err(Error::Canceled { .. })), "{result:?}");
    assert_eq!(seen, vec![0, 1], "the progress counts signed commits");
    let after = (
        git_text(&repo, &["for-each-ref"]),
        object_files(&repo),
        run_dirs(&repo),
    );
    assert_eq!(before, after);
}

/// The rule strips every signature in scope even where nothing else changes,
/// and asks nothing about re-signing.
pub fn check_rewrite_remove_signatures_rule_strips_the_scope(backend: &dyn GitBackend) {
    let Some(key) = generate_key() else { return };
    let (repo, ids) = history_of(&key, &[(true, true), (false, false), (false, true)]);
    let mut recipe = recipe(None);
    recipe.rules = vec![Rule::new(RuleKind::RemoveSignatures)];
    let (plan, _) = run(backend, &repo, &recipe);
    let after = history(&repo);
    assert_eq!(after.len(), 4);
    for (new, old) in after[1..].iter().zip(&ids) {
        assert!(!is_signed(&repo, new), "{new} (was {old}) is still signed");
    }
    assert_eq!(plan.counts.commits_changed, 3, "from the first signed one");
    assert_eq!(plan.per_rule[0].commits, 2);
    assert_eq!(plan.counts.signed_changed, 0);
    assert_eq!(plan.counts.resigned, 0);
}
