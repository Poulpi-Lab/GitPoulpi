//! History rewrite, slice S11 (the history-rewrite design,
//! "External command"): scripts run by an interpreter outside GitPoulpi
//! (shell, Python, an installed Lua), their policy (switched off, trusted by
//! content hash) and what a hung or crashing script leaves behind.
//!
//! "Python" and "Lua" here are the testkit's own fixture program
//! (`script_fixture`), so no interpreter has to be installed; the shell
//! checks use the machine's `sh` and are skipped without one. Expected values
//! come from plain `git`. These checks need the `rewrite-scripts` feature of
//! the backend under test.

use std::process::Command;

use gitpoulpi_core::domain::rewrite::script::parse_script_failure;
use gitpoulpi_core::domain::rewrite::{
    script_hash, Recipe, RewriteRefusal, Rule, RuleKind, Scope, ScriptLanguage, ScriptPolicy,
    ScriptRef, ScriptSource, ScriptTrust, SignaturePolicy,
};
use gitpoulpi_core::{Error, GitBackend};

use super::rewrite::{alice, bob, chain, git_text, history, object_files, preview, run, run_dirs};
use crate::script_fixture;
use crate::FixtureRepo;

fn script(language: ScriptLanguage, text: &str) -> ScriptRef {
    ScriptRef {
        language,
        source: ScriptSource::Inline {
            text: text.to_string(),
        },
        params: Default::default(),
    }
}

/// Everything switched on and `text` trusted.
fn allow(text: &str) -> ScriptPolicy {
    ScriptPolicy {
        shell: true,
        python: true,
        lua: true,
        trusted: vec![script_hash(text)],
        ..ScriptPolicy::default()
    }
}

fn recipe(script: ScriptRef, policy: ScriptPolicy) -> Recipe {
    let mut r = Recipe::new(
        "external",
        Scope::default(),
        vec![Rule::new(RuleKind::Script(script))],
    );
    r.options.signatures = Some(SignaturePolicy::Strip);
    r.script_policy = policy;
    r
}

fn start() -> (FixtureRepo, Vec<String>) {
    let authors: Vec<(String, &str)> = (0..6)
        .map(|i| (if i % 2 == 0 { alice(i) } else { bob(i) }, ""))
        .collect();
    let (repo, _) = chain(&authors);
    let ids = history(&repo);
    (repo, ids)
}

fn lines(repo: &FixtureRepo, format: &str) -> Vec<String> {
    git_text(
        repo,
        &[
            "log",
            "--reverse",
            "--topo-order",
            &format!("--format={format}"),
            "HEAD",
        ],
    )
    .lines()
    .map(str::to_string)
    .collect()
}

fn snapshot(repo: &FixtureRepo) -> (String, Vec<String>, Vec<String>) {
    (
        git_text(repo, &["for-each-ref"]),
        object_files(repo),
        run_dirs(repo),
    )
}

fn try_preview(
    backend: &dyn GitBackend,
    repo: &FixtureRepo,
    recipe: &Recipe,
) -> gitpoulpi_core::Result<gitpoulpi_core::domain::rewrite::RewritePlan> {
    backend.rewrite_preview(&repo.repo_ref(), recipe, &mut |_| {
        std::ops::ControlFlow::Continue(())
    })
}

fn refusal_of<T: std::fmt::Debug>(result: gitpoulpi_core::Result<T>) -> (RewriteRefusal, String) {
    match result {
        Err(Error::RewriteRefused(refusal, detail)) => (refusal, detail),
        other => panic!("expected a refusal, got {other:?}"),
    }
}

/// Whether the process is still running (a zombie is not).
#[cfg(target_os = "linux")]
fn running(pid: u32) -> bool {
    match std::fs::read_to_string(format!("/proc/{pid}/stat")) {
        Ok(stat) => stat
            .rsplit_once(')')
            .is_some_and(|(_, rest)| !rest.trim_start().starts_with('Z')),
        Err(_) => false,
    }
}

/// A script maps the people of every commit: one process, the contract's
/// request on stdin and answer on stdout, whatever the language.
pub fn check_rewrite_external_script_runs_the_contract_and_changes_people(
    backend: &dyn GitBackend,
) {
    script_fixture::install();
    for language in [ScriptLanguage::Python, ScriptLanguage::LuaExternal] {
        let (repo, _) = start();
        let text = "mode=upper\n";
        let before = lines(&repo, "%an");
        let r = recipe(script(language, text), allow(text));
        run(backend, &repo, &r);
        let after = lines(&repo, "%an");
        assert_eq!(
            after,
            before.iter().map(|n| n.to_uppercase()).collect::<Vec<_>>(),
            "{language:?}"
        );
    }
}

/// The state travels in the requests: each answer sees what the last one set.
pub fn check_rewrite_external_script_carries_state_between_commits(backend: &dyn GitBackend) {
    script_fixture::install();
    let (repo, ids) = start();
    let text = "mode=count\n";
    let r = recipe(script(ScriptLanguage::Python, text), allow(text));
    run(backend, &repo, &r);
    let expected: Vec<String> = (0..ids.len()).map(|i| format!("n{i}")).collect();
    assert_eq!(lines(&repo, "%an"), expected);
}

/// A script nobody trusted does not run; trust is the content hash, so a
/// change asks again; a script file in the repository is the same.
pub fn check_rewrite_external_script_is_refused_until_trusted_and_again_after_a_change(
    backend: &dyn GitBackend,
) {
    script_fixture::install();
    let (repo, _) = start();
    let r = repo.repo_ref();
    let text = "mode=upper\n";
    let untrusted = ScriptPolicy {
        trusted: Vec::new(),
        ..allow(text)
    };
    let before = snapshot(&repo);

    // Preview, check and test all refuse; the refusal carries the hash.
    let (refusal, detail) = refusal_of(try_preview(
        backend,
        &repo,
        &recipe(script(ScriptLanguage::Python, text), untrusted.clone()),
    ));
    assert_eq!(refusal, RewriteRefusal::ScriptNotTrusted);
    assert_eq!(detail, script_hash(text));
    let (refusal, _) = refusal_of(backend.rewrite_script_test(
        &r,
        &script(ScriptLanguage::Python, text),
        &untrusted,
        &[],
    ));
    assert_eq!(refusal, RewriteRefusal::ScriptNotTrusted);
    let check = backend
        .rewrite_script_check(&r, &script(ScriptLanguage::Python, text), &untrusted)
        .expect("an untrusted script is an answer, not an error");
    assert_eq!(check.trust, ScriptTrust::Untrusted);
    assert_eq!(check.hash.as_deref(), Some(script_hash(text).as_str()));
    assert_eq!(snapshot(&repo), before);

    // Trusting the hash is enough, and the same text from a file of the
    // repository has the same hash.
    std::fs::create_dir_all(repo.root().join(".gitpoulpi/rewrite")).unwrap();
    std::fs::write(repo.root().join(".gitpoulpi/rewrite/upper.py"), text).unwrap();
    let from_file = ScriptRef {
        language: ScriptLanguage::Python,
        source: ScriptSource::File {
            path: ".gitpoulpi/rewrite/upper.py".into(),
        },
        params: Default::default(),
    };
    preview(backend, &repo, &recipe(from_file.clone(), allow(text)));

    // A changed file is a different hash: refused again, with the new hash.
    let changed = "mode=upper\n# edited\n";
    std::fs::write(repo.root().join(".gitpoulpi/rewrite/upper.py"), changed).unwrap();
    let (refusal, detail) =
        refusal_of(try_preview(backend, &repo, &recipe(from_file, allow(text))));
    assert_eq!(refusal, RewriteRefusal::ScriptNotTrusted);
    assert_eq!(detail, script_hash(changed));
    assert_ne!(script_hash(changed), script_hash(text));
}

/// The setting decides which external languages exist at all.
pub fn check_rewrite_external_script_languages_switched_off_are_refused(backend: &dyn GitBackend) {
    script_fixture::install();
    let (repo, _) = start();
    let r = repo.repo_ref();
    let text = "mode=upper\n";
    let before = snapshot(&repo);
    for (language, off) in [
        (
            ScriptLanguage::Python,
            ScriptPolicy {
                python: false,
                ..allow(text)
            },
        ),
        (
            ScriptLanguage::LuaExternal,
            ScriptPolicy {
                lua: false,
                ..allow(text)
            },
        ),
        (
            ScriptLanguage::Shell,
            ScriptPolicy {
                shell: false,
                ..allow(text)
            },
        ),
    ] {
        let s = script(language, text);
        let (refusal, detail) =
            refusal_of(try_preview(backend, &repo, &recipe(s.clone(), off.clone())));
        assert_eq!(refusal, RewriteRefusal::ScriptsDisabled, "{language:?}");
        assert!(
            detail.contains("rewrite.external_scripts"),
            "the reason names the setting: {detail}"
        );
        let (refusal, _) = refusal_of(backend.rewrite_script_check(&r, &s, &off));
        assert_eq!(refusal, RewriteRefusal::ScriptsDisabled);
        let (refusal, _) = refusal_of(backend.rewrite_script_test(&r, &s, &off, &[]));
        assert_eq!(refusal, RewriteRefusal::ScriptsDisabled);
    }
    assert_eq!(snapshot(&repo), before);
}

/// A script that never answers is stopped at the per-request timeout, its
/// process is gone, its working directory is gone, and nothing moved.
pub fn check_rewrite_external_script_timeout_kills_the_process_and_leaves_nothing(
    backend: &dyn GitBackend,
) {
    script_fixture::install();
    let (repo, ids) = start();
    let scratch = tempfile::tempdir().unwrap();
    let pidfile = scratch.path().join("pid");
    let cwdfile = scratch.path().join("cwd");
    let text = format!(
        "mode=hang\npidfile={}\ncwdfile={}\n",
        pidfile.display(),
        cwdfile.display()
    );
    let policy = ScriptPolicy {
        // Windows can take over a second to start a freshly built program
        // (antivirus scan); a script killed before it ran writes no pid file.
        timeout_ms: if cfg!(windows) { 8000 } else { 1000 },
        ..allow(&text)
    };
    let before = snapshot(&repo);
    let started = std::time::Instant::now();
    let result = try_preview(
        backend,
        &repo,
        &recipe(script(ScriptLanguage::Python, &text), policy),
    );
    let (refusal, detail) = refusal_of(result);
    assert_eq!(refusal, RewriteRefusal::ScriptFailed);
    let (commit, reason) = parse_script_failure(&detail);
    assert_eq!(commit.map(|c| c.to_hex()), Some(ids[0].clone()));
    assert!(reason.contains("did not answer"), "{reason}");
    assert!(started.elapsed() < std::time::Duration::from_secs(20));
    assert_eq!(snapshot(&repo), before, "a timeout changes nothing");

    let pid: u32 = std::fs::read_to_string(&pidfile)
        .expect("the script started")
        .trim()
        .parse()
        .unwrap();
    #[cfg(target_os = "linux")]
    {
        let waited = std::time::Instant::now();
        while running(pid) {
            assert!(
                waited.elapsed() < std::time::Duration::from_secs(5),
                "process {pid} is still running after the timeout"
            );
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
    }
    let _ = pid;
    let cwd = std::path::PathBuf::from(std::fs::read_to_string(&cwdfile).unwrap().trim());
    assert!(!cwd.exists(), "the working directory {cwd:?} is removed");
}

/// A script that dies in the middle of the run: the failure names the commit
/// and the exit status, and no ref, object or run directory was touched.
pub fn check_rewrite_external_script_crash_mid_run_changes_nothing(backend: &dyn GitBackend) {
    script_fixture::install();
    let (repo, ids) = start();
    let text = "mode=crash\n";
    let before = snapshot(&repo);
    let (refusal, detail) = refusal_of(try_preview(
        backend,
        &repo,
        &recipe(script(ScriptLanguage::Python, text), allow(text)),
    ));
    assert_eq!(refusal, RewriteRefusal::ScriptFailed);
    let (commit, reason) = parse_script_failure(&detail);
    assert_eq!(commit.map(|c| c.to_hex()), Some(ids[2].clone()));
    assert!(reason.contains("exited"), "{reason}");
    assert!(reason.contains("fixture crash"), "{reason}");
    assert_eq!(snapshot(&repo), before);
}

/// Bad output is the script's fault, with a reason: not JSON, a field the
/// engine owns, an `error` the script reports itself.
pub fn check_rewrite_external_script_bad_answers_are_typed_failures(backend: &dyn GitBackend) {
    script_fixture::install();
    let (repo, ids) = start();
    let before = snapshot(&repo);
    for (mode, needle) in [
        ("junk", "not JSON"),
        ("bad_field", "belongs to the engine"),
        ("error", "nope"),
    ] {
        let text = format!("mode={mode}\n");
        let (refusal, detail) = refusal_of(try_preview(
            backend,
            &repo,
            &recipe(script(ScriptLanguage::Python, &text), allow(&text)),
        ));
        assert_eq!(refusal, RewriteRefusal::ScriptFailed, "{mode}");
        let (commit, reason) = parse_script_failure(&detail);
        assert_eq!(commit.map(|c| c.to_hex()), Some(ids[0].clone()), "{mode}");
        assert!(reason.contains(needle), "{mode}: {reason}");
    }
    assert_eq!(snapshot(&repo), before);
}

/// The script's environment holds nothing of the caller's: a planted variable
/// is not there.
pub fn check_rewrite_external_script_environment_is_cleared(backend: &dyn GitBackend) {
    script_fixture::install();
    std::env::set_var("GITPOULPI_PLANTED_SECRET", "hunter2");
    let (repo, ids) = start();
    let text = "mode=env\n";
    let r = recipe(script(ScriptLanguage::Python, text), allow(text));
    run(backend, &repo, &r);
    assert_eq!(
        lines(&repo, "%s"),
        vec!["secret=unset"; ids.len()],
        "the script saw the planted variable"
    );
}

/// `rewrite_script_check` for an external script: the interpreter, the hash,
/// the trust and a dry handshake (not run for an untrusted script).
pub fn check_rewrite_external_script_check_reports_trust_and_does_a_handshake(
    backend: &dyn GitBackend,
) {
    script_fixture::install();
    let (repo, _) = start();
    let r = repo.repo_ref();
    let scratch = tempfile::tempdir().unwrap();

    let text = "mode=upper\n";
    let check = backend
        .rewrite_script_check(&r, &script(ScriptLanguage::Python, text), &allow(text))
        .unwrap();
    assert!(check.ok(), "{:?}", check.problems);
    assert_eq!(check.trust, ScriptTrust::Trusted);
    assert_eq!(check.hash.as_deref(), Some(script_hash(text).as_str()));
    assert!(check.interpreter.is_some());

    for mode in ["junk", "bad_field"] {
        let text = format!("mode={mode}\n");
        let check = backend
            .rewrite_script_check(&r, &script(ScriptLanguage::Python, &text), &allow(&text))
            .unwrap();
        assert!(
            check.problems.iter().any(|p| p.contains("handshake")),
            "{mode}: {:?}",
            check.problems
        );
    }

    // Untrusted: no process is started, not even for a handshake.
    let pidfile = scratch.path().join("pid");
    let text = format!("mode=upper\npidfile={}\n", pidfile.display());
    let untrusted = ScriptPolicy {
        trusted: Vec::new(),
        ..allow(&text)
    };
    let check = backend
        .rewrite_script_check(&r, &script(ScriptLanguage::Python, &text), &untrusted)
        .unwrap();
    assert_eq!(check.trust, ScriptTrust::Untrusted);
    assert!(!pidfile.exists(), "an untrusted script was started");
}

fn have_sh() -> bool {
    Command::new("sh")
        .args(["-c", ":"])
        .status()
        .is_ok_and(|s| s.success())
}

/// A shell script, with the machine's own `sh` (skipped without one): the
/// same contract, no interpreter to install, trusted by hash like the rest.
pub fn check_rewrite_shell_script_runs_with_the_machines_shell(backend: &dyn GitBackend) {
    if !have_sh() {
        eprintln!("skipped: no sh on this machine");
        return;
    }
    let (repo, ids) = start();
    let text = r#"#!/bin/sh
while IFS= read -r line; do
  printf '{"message":"from shell\\n"}\n'
done
"#;
    let untrusted = ScriptPolicy {
        trusted: Vec::new(),
        ..ScriptPolicy::default()
    };
    let (refusal, _) = refusal_of(try_preview(
        backend,
        &repo,
        &recipe(script(ScriptLanguage::Shell, text), untrusted),
    ));
    assert_eq!(refusal, RewriteRefusal::ScriptNotTrusted);

    let r = recipe(script(ScriptLanguage::Shell, text), allow(text));
    run(backend, &repo, &r);
    assert_eq!(lines(&repo, "%s"), vec!["from shell"; ids.len()]);
}
