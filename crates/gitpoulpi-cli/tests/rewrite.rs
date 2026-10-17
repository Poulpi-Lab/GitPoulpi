//! `gitpoulpi rewrite`, end to end through the real binary, the daemon and
//! the backend. Assertions are on deserialized JSON and exit codes.
#![cfg(feature = "history-rewrite")]

use std::path::Path;

use assert_cmd::Command;
use gitpoulpi_core::domain::rewrite::{
    IdentityMatch, IdentityRule, IdentitySet, Recipe, RewriteOutcome, RewritePlan, RewriteTools,
    Rule, RuleKind, Scope, SignaturePolicy, SigningCheck, Who,
};
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use serde_json::Value;

const OLD_EMAIL: &str = "testkit@gitpoulpi.local";
const NEW_EMAIL: &str = "casey@example.com";

fn config_dir() -> &'static Path {
    static DIR: std::sync::OnceLock<tempfile::TempDir> = std::sync::OnceLock::new();
    DIR.get_or_init(|| tempfile::tempdir().unwrap()).path()
}

/// Runs the CLI; returns (exit code, parsed stdout).
fn run(repo: &FixtureRepo, session: &str, stdin: Option<&str>, args: &[&str]) -> (i32, Value) {
    let mut command = Command::cargo_bin("gitpoulpi").unwrap();
    command
        .env("GITPOULPI_CONFIG_DIR", config_dir())
        .args(["--json", "--session", session])
        .args(["--repo", &repo.root().to_string_lossy()])
        .args(args)
        .current_dir(repo.root());
    if let Some(input) = stdin {
        command.write_stdin(input.to_string());
    }
    let output = command.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let value = serde_json::from_str(&stdout).unwrap_or_else(|e| {
        panic!(
            "not JSON ({e}): {stdout}\n{}",
            String::from_utf8_lossy(&output.stderr)
        )
    });
    (output.status.code().unwrap_or(-1), value)
}

fn ok(repo: &FixtureRepo, session: &str, args: &[&str]) -> Value {
    let (code, value) = run(repo, session, None, args);
    assert_eq!(code, 0, "{args:?}: {value}");
    value
}

/// A standalone daemon for the session. Without one every command embeds a
/// short-lived daemon, and the previewed plan (which lives in the daemon)
/// would not survive to the next command. Stopped on drop.
struct Daemon {
    session: String,
    cwd: std::path::PathBuf,
}

impl Daemon {
    fn start(repo: &FixtureRepo, session: &str) -> Daemon {
        let daemon = Daemon {
            session: session.to_string(),
            cwd: repo.root().to_path_buf(),
        };
        let status = std::process::Command::new(env!("CARGO_BIN_EXE_gitpoulpi"))
            .env("GITPOULPI_CONFIG_DIR", config_dir())
            .current_dir(&daemon.cwd)
            .args(["-s", session, "daemon", "start", "--json"])
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .unwrap();
        assert!(status.success(), "daemon start");
        daemon
    }
}

impl Drop for Daemon {
    fn drop(&mut self) {
        let _ = std::process::Command::new(env!("CARGO_BIN_EXE_gitpoulpi"))
            .env("GITPOULPI_CONFIG_DIR", config_dir())
            .current_dir(&self.cwd)
            .args(["-s", &self.session, "daemon", "stop", "--json"])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
    }
}

/// Three commits on `main`, a topic branch on the first and a tag.
fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit("alpha")
        .unwrap()
        .branch("topic")
        .unwrap()
        .commit("beta")
        .unwrap()
        .tag("light")
        .unwrap()
        .commit("gamma")
        .unwrap()
        .build()
}

/// The fixture with `main` on a remote, so its ref counts as pushed.
fn pushed_fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit("alpha")
        .unwrap()
        .with_origin()
        .unwrap()
        .push("main")
        .unwrap()
        .track("main", "origin")
        .unwrap()
        .commit("beta")
        .unwrap()
        .build()
}

const CASEY: [&str; 6] = [
    "--match-email",
    OLD_EMAIL,
    "--set-name",
    "Casey Example",
    "--set-email",
    NEW_EMAIL,
];

fn flags<'a>(head: &[&'a str], tail: &[&'a str]) -> Vec<&'a str> {
    head.iter()
        .chain(CASEY.iter())
        .chain(tail)
        .copied()
        .collect()
}

fn recipe() -> Recipe {
    let rule = Rule::new(RuleKind::Identity(IdentityRule {
        when: IdentityMatch::email(OLD_EMAIL),
        set: IdentitySet {
            name: Some("Casey Example".into()),
            email: Some(NEW_EMAIL.into()),
        },
        apply_to: Who::Author,
        include_taggers: false,
        map_trailers: false,
    }));
    let mut recipe = Recipe::new("casey", Scope::AllLocal, vec![rule]);
    recipe.options.signatures = Some(SignaturePolicy::Strip);
    recipe
}

fn rev_parse(repo: &FixtureRepo, rev: &str) -> String {
    repo.git(&["rev-parse", rev]).trim().to_string()
}

fn authors(repo: &FixtureRepo) -> Vec<String> {
    let mut all: Vec<String> = repo
        .git(&["log", "--branches", "--tags", "--format=%ae"])
        .lines()
        .map(str::to_string)
        .collect();
    all.sort();
    all.dedup();
    all
}

fn run_dirs(repo: &FixtureRepo) -> usize {
    std::fs::read_dir(repo.root().join(".git/gitpoulpi/rewrite"))
        .map(|d| d.count())
        .unwrap_or(0)
}

#[test]
fn tools_lists_the_checks() {
    let repo = fixture();
    let tools: RewriteTools =
        serde_json::from_value(ok(&repo, "cli-rw-tools", &["rewrite", "tools"])).unwrap();
    assert!(!tools.tools.is_empty());
    let fresh = ok(&repo, "cli-rw-tools", &["rewrite", "tools", "--fresh"]);
    assert!(fresh.get("tools").is_some());
}

#[test]
fn preview_plans_with_flags_and_changes_nothing() {
    let repo = fixture();
    let session = "cli-rw-preview";
    let head = rev_parse(&repo, "HEAD");
    let value = ok(
        &repo,
        session,
        &flags(
            &["rewrite", "preview", "--all-local", "--signatures", "strip"],
            &["--limit", "2"],
        ),
    );
    let total = value["rows_total"].as_u64().unwrap();
    assert_eq!(total, 3);
    assert_eq!(value["rows"].as_array().unwrap().len(), 2);
    let plan: RewritePlan = serde_json::from_value(value).unwrap();
    assert_eq!(plan.counts.commits_changed, 3);
    assert!(!plan.run.0.is_empty() && !plan.digest.0.is_empty());
    assert_eq!(rev_parse(&repo, "HEAD"), head, "a preview moves nothing");
    assert_eq!(authors(&repo), [OLD_EMAIL]);

    // `--all` prints every changed commit.
    let all = ok(
        &repo,
        session,
        &flags(
            &["rewrite", "preview", "--all-local", "--signatures", "strip"],
            &["--all"],
        ),
    );
    assert_eq!(all["rows"].as_array().unwrap().len(), 3);
}

#[test]
fn rows_pages_the_previewed_run_and_filters() {
    let repo = fixture();
    let session = "cli-rw-rows";
    let _daemon = Daemon::start(&repo, session);
    let plan: RewritePlan = serde_json::from_value(ok(
        &repo,
        session,
        &flags(
            &["rewrite", "preview", "--all-local", "--signatures", "strip"],
            &["--limit", "1"],
        ),
    ))
    .unwrap();
    let run = plan.run.0.as_str();
    let first = ok(
        &repo,
        session,
        &["rewrite", "rows", "--run", run, "--limit", "2"],
    );
    assert_eq!(first["total"], 3);
    assert_eq!(first["rows"].as_array().unwrap().len(), 2);
    let rest = ok(
        &repo,
        session,
        &[
            "rewrite", "rows", "--run", run, "--offset", "2", "--limit", "2",
        ],
    );
    assert_eq!(rest["rows"].as_array().unwrap().len(), 1);
    let dropped = ok(
        &repo,
        session,
        &["rewrite", "rows", "--run", run, "--filter", "dropped"],
    );
    assert_eq!(dropped["total"], 0);
    let pushed = ok(
        &repo,
        session,
        &["rewrite", "rows", "--run", run, "--filter", "pushed"],
    );
    assert_eq!(pushed["total"], 0);
}

#[test]
fn apply_rewrites_once_and_undo_puts_it_back() {
    let repo = fixture();
    let session = "cli-rw-apply";
    let _daemon = Daemon::start(&repo, session);
    let head = rev_parse(&repo, "HEAD");
    let refs_before = repo.git(&[
        "for-each-ref",
        "--format=%(refname) %(objectname)",
        "refs/heads",
        "refs/tags",
    ]);
    let plan: RewritePlan = serde_json::from_value(ok(
        &repo,
        session,
        &flags(
            &["rewrite", "preview", "--all-local", "--signatures", "strip"],
            &[],
        ),
    ))
    .unwrap();

    // A digest that is not the reviewed plan's is refused, typed, exit 2.
    let (code, value) = run(
        &repo,
        session,
        None,
        &[
            "rewrite",
            "apply",
            "--run",
            &plan.run.0,
            "--digest",
            "not-the-digest",
        ],
    );
    assert_eq!(code, 2, "{value}");
    assert_eq!(value["refused"]["reason"], "stale");
    assert_eq!(rev_parse(&repo, "HEAD"), head);

    let outcome: RewriteOutcome = serde_json::from_value(ok(
        &repo,
        session,
        &[
            "rewrite",
            "apply",
            "--run",
            &plan.run.0,
            "--digest",
            &plan.digest.0,
        ],
    ))
    .unwrap();
    assert_eq!(outcome.commits_changed, 3);
    assert_ne!(rev_parse(&repo, "HEAD"), head);
    assert_eq!(authors(&repo), [NEW_EMAIL]);

    // The spent run is gone: applying it again is stale.
    let (code, value) = run(
        &repo,
        session,
        None,
        &[
            "rewrite",
            "apply",
            "--run",
            &plan.run.0,
            "--digest",
            &plan.digest.0,
        ],
    );
    assert_eq!(code, 2, "{value}");
    assert_eq!(value["refused"]["reason"], "stale");

    ok(&repo, session, &["undo"]);
    assert_eq!(rev_parse(&repo, "HEAD"), head);
    assert_eq!(
        repo.git(&[
            "for-each-ref",
            "--format=%(refname) %(objectname)",
            "refs/heads",
            "refs/tags"
        ]),
        refs_before
    );
    assert_eq!(authors(&repo), [OLD_EMAIL]);
}

#[test]
fn a_ref_that_moved_after_the_preview_is_refused_and_nothing_changes() {
    let repo = fixture();
    let session = "cli-rw-moved";
    let _daemon = Daemon::start(&repo, session);
    let plan: RewritePlan = serde_json::from_value(ok(
        &repo,
        session,
        &flags(
            &["rewrite", "preview", "--all-local", "--signatures", "strip"],
            &[],
        ),
    ))
    .unwrap();
    repo.git(&["commit", "-q", "--allow-empty", "-m", "moved on"]);
    let head = rev_parse(&repo, "HEAD");
    let (code, value) = run(
        &repo,
        session,
        None,
        &[
            "rewrite",
            "apply",
            "--run",
            &plan.run.0,
            "--digest",
            &plan.digest.0,
        ],
    );
    assert_eq!(code, 2, "{value}");
    assert_eq!(value["refused"]["reason"], "stale");
    assert_eq!(rev_parse(&repo, "HEAD"), head);
}

#[test]
fn run_previews_and_applies_a_recipe_from_stdin() {
    let repo = fixture();
    let session = "cli-rw-run";
    let head = rev_parse(&repo, "HEAD");
    let json = serde_json::to_string(&recipe()).unwrap();
    let (code, value) = run(
        &repo,
        session,
        Some(&json),
        &["rewrite", "run", "--recipe", "-"],
    );
    assert_eq!(code, 0, "{value}");
    let plan: RewritePlan = serde_json::from_value(value["plan"].clone()).unwrap();
    let outcome: RewriteOutcome = serde_json::from_value(value["outcome"].clone()).unwrap();
    assert_eq!(plan.counts.commits_changed, 3);
    assert_eq!(outcome.commits_changed, 3);
    assert_eq!(plan.run, outcome.run);
    assert_eq!(authors(&repo), [NEW_EMAIL]);
    ok(&repo, session, &["undo"]);
    assert_eq!(rev_parse(&repo, "HEAD"), head);
}

#[test]
fn a_recipe_file_in_toml_and_inline_json_work_and_flags_add_rules() {
    let repo = fixture();
    let session = "cli-rw-recipe";
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("casey.toml");
    std::fs::write(&file, toml::to_string(&recipe()).unwrap()).unwrap();
    let plan = ok(
        &repo,
        session,
        &["rewrite", "preview", "--recipe", &file.to_string_lossy()],
    );
    assert_eq!(plan["counts"]["commits_changed"], 3);

    let json = serde_json::to_string(&recipe()).unwrap();
    let plan = ok(
        &repo,
        session,
        &["rewrite", "preview", "--recipe-json", &json],
    );
    assert_eq!(plan["counts"]["commits_changed"], 3);

    // A flag rule goes after the file's: the message rule changes no author
    // but the commits still count once.
    let plan = ok(
        &repo,
        session,
        &[
            "rewrite",
            "preview",
            "--recipe-json",
            &json,
            "--replace-message",
            "gamma=delta",
        ],
    );
    assert_eq!(plan["per_rule"].as_array().unwrap().len(), 2);
}

#[test]
fn discard_drops_the_run_and_twice_is_fine() {
    let repo = fixture();
    let session = "cli-rw-discard";
    let _daemon = Daemon::start(&repo, session);
    let plan: RewritePlan = serde_json::from_value(ok(
        &repo,
        session,
        &flags(
            &["rewrite", "preview", "--all-local", "--signatures", "strip"],
            &[],
        ),
    ))
    .unwrap();
    assert_eq!(run_dirs(&repo), 1);
    let value = ok(
        &repo,
        session,
        &["rewrite", "discard", "--run", &plan.run.0],
    );
    assert_eq!(value["discarded"], plan.run.0.as_str());
    assert_eq!(run_dirs(&repo), 0);
    ok(
        &repo,
        session,
        &["rewrite", "discard", "--run", &plan.run.0],
    );
    let (code, value) = run(
        &repo,
        session,
        None,
        &["rewrite", "rows", "--run", &plan.run.0],
    );
    assert_eq!(code, 2, "{value}");
    assert_eq!(value["refused"]["reason"], "stale");
}

#[test]
fn pushed_refs_need_yes_to_apply_and_to_run_exit_3() {
    let repo = pushed_fixture();
    let session = "cli-rw-pushed";
    let _daemon = Daemon::start(&repo, session);
    let head = rev_parse(&repo, "HEAD");
    // `main` is protected by default (`rewrite.protected_branches`).
    let preview = flags(
        &[
            "rewrite",
            "preview",
            "--signatures",
            "strip",
            "--include-protected",
            "main",
        ],
        &[],
    );
    let plan: RewritePlan = serde_json::from_value(ok(&repo, session, &preview)).unwrap();
    let apply = [
        "rewrite",
        "apply",
        "--run",
        &plan.run.0,
        "--digest",
        &plan.digest.0,
    ];
    let (code, value) = run(&repo, session, None, &apply);
    assert_eq!(code, 3, "{value}");
    assert!(value["needs_confirmation"]["prompt"].is_string());
    assert_eq!(rev_parse(&repo, "HEAD"), head, "nothing changed");

    // `run` without --yes: exit 3, nothing changed, no run left behind.
    ok(
        &repo,
        session,
        &["rewrite", "discard", "--run", &plan.run.0],
    );
    let run_args = flags(
        &[
            "rewrite",
            "run",
            "--signatures",
            "strip",
            "--include-protected",
            "main",
        ],
        &[],
    );
    let (code, value) = run(&repo, session, None, &run_args);
    assert_eq!(code, 3, "{value}");
    assert!(value["needs_confirmation"].is_object());
    assert_eq!(rev_parse(&repo, "HEAD"), head);
    assert_eq!(run_dirs(&repo), 0);

    let mut with_yes = run_args.clone();
    with_yes.push("--yes");
    let (code, value) = run(&repo, session, None, &with_yes);
    assert_eq!(code, 0, "{value}");
    assert_ne!(rev_parse(&repo, "HEAD"), head);
    // Nothing was pushed: the remote still has the old tip.
    assert_eq!(
        repo.git_in_remote(&["rev-parse", "main"]).trim(),
        rev_parse(&repo, "origin/main")
    );
    assert_ne!(rev_parse(&repo, "HEAD"), rev_parse(&repo, "origin/main"));
}

#[test]
fn refusals_and_bad_input_have_their_exit_codes() {
    let repo = fixture();
    let session = "cli-rw-refuse";

    // A recipe with no rule: refused by the shared validation, exit 2.
    let (code, value) = run(&repo, session, None, &["rewrite", "preview", "--all-local"]);
    assert_eq!(code, 2, "{value}");
    assert_eq!(value["refused"]["reason"], "no_rule");
    let (code, value) = run(&repo, session, None, &["rewrite", "run", "--all-local"]);
    assert_eq!(code, 2, "{value}");

    // An invalid rule (an email with '<'): refused, exit 2.
    let (code, value) = run(
        &repo,
        session,
        None,
        &["rewrite", "preview", "--set-email", "a<b@example.com"],
    );
    assert_eq!(code, 2, "{value}");
    assert_eq!(value["refused"]["reason"], "invalid_rule");

    // Bad usage and unreadable input are plain errors, exit 1.
    let (code, value) = run(
        &repo,
        session,
        None,
        &["rewrite", "preview", "--match-email", "x@example.com"],
    );
    assert_eq!(code, 1, "{value}");
    assert!(value["error"].is_string());
    let (code, value) = run(
        &repo,
        session,
        None,
        &["rewrite", "preview", "--recipe", "/nonexistent/recipe.toml"],
    );
    assert_eq!(code, 1, "{value}");
    let (code, value) = run(
        &repo,
        session,
        Some("{ not json"),
        &["rewrite", "preview", "--recipe", "-"],
    );
    assert_eq!(code, 1, "{value}");
    assert_eq!(run_dirs(&repo), 0);
}

#[test]
fn secrets_files_stay_out_of_answers_and_need_a_secrets_rule() {
    let repo = fixture();
    let session = "cli-rw-secrets";
    let dir = tempfile::tempdir().unwrap();
    let secrets = dir.path().join("secrets.txt");
    std::fs::write(&secrets, "hunter2-not-a-real-secret\n").unwrap();
    let secrets_arg = secrets.to_string_lossy().into_owned();

    // No rule that searches for secrets: an error that does not repeat them.
    let json = serde_json::to_string(&recipe()).unwrap();
    let (code, value) = run(
        &repo,
        session,
        None,
        &[
            "rewrite",
            "preview",
            "--recipe-json",
            &json,
            "--secrets-file",
            &secrets_arg,
        ],
    );
    assert_eq!(code, 1, "{value}");
    assert!(!value.to_string().contains("hunter2"));

    // A missing file and an empty one are errors too.
    let (code, value) = run(
        &repo,
        session,
        None,
        &[
            "rewrite",
            "preview",
            "--recipe-json",
            &json,
            "--secrets-file",
            "/nonexistent/secrets",
        ],
    );
    assert_eq!(code, 1, "{value}");
    let empty = dir.path().join("empty.txt");
    std::fs::write(&empty, "\n").unwrap();
    let (code, _) = run(
        &repo,
        session,
        None,
        &[
            "rewrite",
            "preview",
            "--recipe-json",
            &json,
            "--secrets-file",
            &empty.to_string_lossy(),
        ],
    );
    assert_eq!(code, 1);

    // With a secrets rule the literals reach the daemon (the content engine
    // may refuse the rule for now); whatever comes back never echoes them.
    let rule = serde_json::json!({
        "type": "content_replace", "find": "secrets", "with": "***"
    });
    let recipe = serde_json::json!({
        "scope": {"scope": "all_local"},
        "rules": [{"kind": rule}],
        "options": {"signatures": "strip"}
    });
    let (_, value) = run(
        &repo,
        session,
        None,
        &[
            "rewrite",
            "preview",
            "--recipe-json",
            &recipe.to_string(),
            "--secrets-file",
            &secrets_arg,
        ],
    );
    assert!(!value.to_string().contains("hunter2"));
}

/// A preview outlives its (embedded, short-lived) daemon: preview, apply and
/// undo are three separate invocations and no `daemon start`.
#[test]
fn preview_then_apply_in_two_invocations_without_a_daemon_then_undo() {
    let repo = fixture();
    let session = "cli-rw-two-shots";
    let head = rev_parse(&repo, "HEAD");
    let plan: RewritePlan = serde_json::from_value(ok(
        &repo,
        session,
        &flags(
            &["rewrite", "preview", "--all-local", "--signatures", "strip"],
            &[],
        ),
    ))
    .unwrap();
    let outcome: RewriteOutcome = serde_json::from_value(ok(
        &repo,
        session,
        &[
            "rewrite",
            "apply",
            "--run",
            &plan.run.0,
            "--digest",
            &plan.digest.0,
        ],
    ))
    .unwrap();
    assert_eq!(outcome.commits_changed, 3);
    assert_eq!(authors(&repo), [NEW_EMAIL]);
    ok(&repo, session, &["undo"]);
    assert_eq!(rev_parse(&repo, "HEAD"), head);
    assert_eq!(authors(&repo), [OLD_EMAIL]);
}

/// A hand-edited `plan.json` is never applied: stale, nothing moves.
#[test]
fn a_tampered_plan_file_is_refused_stale_by_a_later_invocation() {
    let repo = fixture();
    let session = "cli-rw-tampered";
    let head = rev_parse(&repo, "HEAD");
    let plan: RewritePlan = serde_json::from_value(ok(
        &repo,
        session,
        &flags(
            &["rewrite", "preview", "--all-local", "--signatures", "strip"],
            &[],
        ),
    ))
    .unwrap();
    let file = repo
        .root()
        .join(".git/gitpoulpi/rewrite")
        .join(&plan.run.0)
        .join("plan.json");
    let mut stored: Value = serde_json::from_slice(&std::fs::read(&file).unwrap()).unwrap();
    // Point the first ref somewhere else, keeping the digest.
    stored["refs"][0]["new"] = Value::String(head.clone());
    std::fs::write(&file, serde_json::to_vec(&stored).unwrap()).unwrap();
    let (code, value) = run(
        &repo,
        session,
        None,
        &[
            "rewrite",
            "apply",
            "--run",
            &plan.run.0,
            "--digest",
            &plan.digest.0,
        ],
    );
    assert_eq!(code, 2, "{value}");
    assert_eq!(value["refused"]["reason"], "stale");
    assert_eq!(rev_parse(&repo, "HEAD"), head);
    assert_eq!(authors(&repo), [OLD_EMAIL]);
}

/// Every file under `dir` that holds `needle`.
fn files_holding(dir: &Path, needle: &str) -> Vec<std::path::PathBuf> {
    let mut found = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return found;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            found.extend(files_holding(&path, needle));
        } else if std::fs::read(&path)
            .map(|bytes| String::from_utf8_lossy(&bytes).contains(needle))
            .unwrap_or(false)
        {
            found.push(path);
        }
    }
    found
}

fn secrets_recipe() -> String {
    use gitpoulpi_core::domain::rewrite::{ContentFind, ContentReplace};
    let rule = Rule::new(RuleKind::ContentReplace(ContentReplace {
        paths: Vec::new(),
        find: ContentFind::Secrets {
            literals: Vec::new(),
            scan: false,
        },
        with: String::new(),
        max_blob_bytes: None,
    }));
    let mut recipe = Recipe::new("secrets", Scope::default(), vec![rule]);
    recipe.options.signatures = Some(SignaturePolicy::Strip);
    serde_json::to_string(&recipe).unwrap()
}

/// A secret from `--secrets-file` is replaced in every commit and in the
/// working tree, the secret appears in no answer, no log and no file the
/// rewrite leaves behind, the blob diff hides it, and undo brings it back.
#[test]
fn a_secrets_file_replacement_works_end_to_end_and_undo_restores_it() {
    const SECRET: &str = "EXAMPLE_TOKEN_123";
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("alpha")
        .unwrap()
        .commit_file("config.txt", &format!("token = {SECRET}\n"), "add config")
        .unwrap()
        .commit_file(
            "config.txt",
            &format!("token = {SECRET}\nmore\n"),
            "edit config",
        )
        .unwrap()
        .build();
    let session = "cli-rw-secrets-e2e";
    let dir = tempfile::tempdir().unwrap();
    let secrets = dir.path().join("secrets.txt");
    std::fs::write(&secrets, format!("{SECRET}\n")).unwrap();
    let secrets_arg = secrets.to_string_lossy().into_owned();
    let recipe = secrets_recipe();
    let head = rev_parse(&repo, "HEAD");
    let first = rev_parse(&repo, "HEAD~1");

    // Preview, then the diff of the first commit's file.
    let (code, value) = run(
        &repo,
        session,
        None,
        &[
            "rewrite",
            "preview",
            "--recipe-json",
            &recipe,
            "--secrets-file",
            &secrets_arg,
        ],
    );
    assert_eq!(code, 0, "{value}");
    assert!(!value.to_string().contains(SECRET));
    let plan: RewritePlan = serde_json::from_value(value).unwrap();
    assert_eq!(plan.counts.commits_changed, 2);
    assert_eq!(plan.counts.blobs_changed, 2);
    let (code, diff) = run(
        &repo,
        session,
        None,
        &[
            "rewrite",
            "diff",
            "--run",
            &plan.run.0,
            "--commit",
            &first,
            "--path",
            "config.txt",
        ],
    );
    assert_eq!(code, 0, "{diff}");
    assert!(!diff.to_string().contains(SECRET));
    let change: gitpoulpi_core::domain::rewrite::BlobChange = serde_json::from_value(diff).unwrap();
    assert!(change.redacted);
    assert_eq!(
        change.kind,
        gitpoulpi_core::domain::rewrite::BlobChangeKind::Modified
    );
    ok(
        &repo,
        session,
        &["rewrite", "discard", "--run", &plan.run.0],
    );

    // Preview and apply in one call.
    let (code, value) = run(
        &repo,
        session,
        None,
        &[
            "rewrite",
            "run",
            "--recipe-json",
            &recipe,
            "--secrets-file",
            &secrets_arg,
        ],
    );
    assert_eq!(code, 0, "{value}");
    assert!(!value.to_string().contains(SECRET));
    // The branches, not `--all`: the undo pins keep the old history alive
    // until a purge (S15).
    let log = repo.git(&["log", "-p", "--branches", "--tags"]);
    assert!(!log.contains(SECRET), "gone from every commit");
    let config = repo.root().join("config.txt");
    assert_eq!(
        std::fs::read_to_string(&config).unwrap(),
        "token = ***REMOVED***\nmore\n"
    );
    assert_eq!(repo.git(&["status", "--porcelain"]), "");
    // Nothing the tool wrote holds it: not the application log, not the
    // stored plan, not the maps.
    assert_eq!(
        files_holding(config_dir(), SECRET),
        Vec::<std::path::PathBuf>::new()
    );
    assert_eq!(
        files_holding(&repo.root().join(".git/gitpoulpi"), SECRET),
        Vec::<std::path::PathBuf>::new()
    );

    // Undo puts the commits, and the file, back.
    ok(&repo, session, &["undo"]);
    assert_eq!(rev_parse(&repo, "HEAD"), head);
    assert_eq!(
        std::fs::read_to_string(&config).unwrap(),
        format!("token = {SECRET}\nmore\n")
    );
    assert_eq!(repo.git(&["status", "--porcelain"]), "");
}

/// The flag rules for content: a folder removed from all history, and the
/// commit it emptied is dropped.
#[test]
fn content_flags_remove_paths_and_prune_the_commits_they_empty() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("alpha")
        .unwrap()
        .commit_file("secrets/key.pem", "k\n", "add key")
        .unwrap()
        .commit_file("app.txt", "a\n", "add app")
        .unwrap()
        .build();
    let session = "cli-rw-content-flags";
    let value = ok(
        &repo,
        session,
        &[
            "rewrite",
            "run",
            "--remove-path",
            "secrets/",
            "--signatures",
            "strip",
        ],
    );
    let plan: RewritePlan = serde_json::from_value(value["plan"].clone()).unwrap();
    assert_eq!(plan.counts.commits_dropped, 1);
    assert_eq!(repo.git(&["log", "--format=%s"]).lines().count(), 2);
    assert!(!repo.root().join("secrets").exists());
    assert!(repo.root().join("app.txt").exists());
}

#[cfg(feature = "rewrite-scripts")]
const SHOUT_LUA: &str = r#"--! name: Shout
--! param suffix: text "Added to every subject" = "!"
function rewrite_commit(c, ctx)
  log('seen ' .. ctx.index)
  return { message = string.upper((c.message:gsub('\n+$', ''))) .. ctx.params.suffix .. '\n' }
end
"#;

/// `rewrite script check` / `test`, the `--script` flag and the typed
/// `ScriptFailed` answer, through the real binary.
#[cfg(feature = "rewrite-scripts")]
#[test]
fn scripts_are_checked_tested_run_and_fail_with_commit_and_reason() {
    let repo = fixture();
    let session = "cli-rw-scripts";
    let script = repo.root().join("shout.lua");
    std::fs::write(&script, SHOUT_LUA).unwrap();
    let file = script.to_string_lossy().into_owned();
    let head = repo.git(&["rev-parse", "HEAD"]);

    // check: header, functions, no problems.
    let check = ok(&repo, session, &["rewrite", "script", "check", &file]);
    assert_eq!(check["header"]["name"], "Shout");
    assert_eq!(check["header"]["params"][0]["name"], "suffix");
    assert_eq!(check["functions"], serde_json::json!(["rewrite_commit"]));
    assert_eq!(check["problems"], serde_json::json!([]));

    // check: a broken script is exit 2 with the problem in the answer.
    let broken = repo.root().join("broken.lua");
    std::fs::write(&broken, "function rewrite_commit(\n").unwrap();
    let (code, value) = run(
        &repo,
        session,
        None,
        &["rewrite", "script", "check", &broken.to_string_lossy()],
    );
    assert_eq!(code, 2);
    assert!(value["problems"][0].as_str().unwrap().contains("script:"));

    // test: what it would change and log, nothing written.
    let value = ok(
        &repo,
        session,
        &[
            "rewrite", "script", "test", &file, "--param", "suffix=?", "--commit", "HEAD",
        ],
    );
    let rows = value["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["commit"], head);
    assert_eq!(rows[0]["log"][0], "seen 0");
    let message = rows[0]["fields"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["field"] == "message")
        .unwrap();
    assert_eq!(message["new"], "GAMMA?\n");
    assert_eq!(repo.git(&["rev-parse", "HEAD"]), head);

    // run with --script: the history changes, as one undo entry.
    let value = ok(
        &repo,
        session,
        &[
            "rewrite",
            "run",
            "--script",
            &file,
            "--script-param",
            "suffix=.",
            "--signatures",
            "strip",
        ],
    );
    let plan: RewritePlan = serde_json::from_value(value["plan"].clone()).unwrap();
    assert_eq!(plan.counts.commits_changed, 3);
    assert!(repo
        .git(&["log", "--format=%s"])
        .lines()
        .all(|s| s.ends_with('.') && s == s.to_uppercase()));

    // A failing script: exit 2, the commit and the reason are fields.
    let failing = repo.root().join("failing.lua");
    std::fs::write(
        &failing,
        "function rewrite_commit(c, ctx) error('nope ' .. ctx.index) end\n",
    )
    .unwrap();
    let (code, value) = run(
        &repo,
        session,
        None,
        &[
            "rewrite",
            "preview",
            "--script",
            &failing.to_string_lossy(),
            "--signatures",
            "strip",
        ],
    );
    assert_eq!(code, 2, "{value}");
    assert_eq!(value["refused"]["reason"], "script_failed");
    let first = repo.git(&["rev-list", "--max-parents=0", "HEAD"]);
    assert_eq!(value["refused"]["script"]["commit"], first);
    assert!(value["refused"]["script"]["reason"]
        .as_str()
        .unwrap()
        .contains("nope 0"));

    // A parameter without a script is a usage error, not silence.
    let (code, _) = run(
        &repo,
        session,
        None,
        &["rewrite", "preview", "--script-param", "a=b"],
    );
    assert_ne!(code, 0);
}

/// `rewrite scan-secrets` lists a planted token with its history and a
/// redacted preview, `--remove-finding` removes it by id, and the value
/// shows up in no answer, no log and no file the tool wrote.
#[test]
fn scan_secrets_lists_redacted_findings_and_remove_finding_removes_by_id() {
    use gitpoulpi_core::domain::rewrite::SecretScan;

    let token = format!("ghp_{}", "Zk9xQ2mL7vB4nR8tY1wC5dF3hJ6pS0aEgUiX");
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("alpha")
        .unwrap()
        .commit_file("deploy.env", &format!("GITHUB_TOKEN={token}\n"), "add env")
        .unwrap()
        .commit_file("README.md", "docs\n", "add readme")
        .unwrap()
        .build();
    let session = "cli-rw-scan-secrets";
    let first = rev_parse(&repo, "HEAD~1");
    let head = rev_parse(&repo, "HEAD");

    let value = ok(&repo, session, &["rewrite", "scan-secrets"]);
    assert!(!value.to_string().contains(&token), "{value}");
    let scan: SecretScan = serde_json::from_value(value).unwrap();
    assert_eq!(scan.findings.len(), 1, "{scan:?}");
    let f = &scan.findings[0];
    assert_eq!(f.rule, "github-token");
    assert_eq!(f.path, "deploy.env");
    assert_eq!(f.first_commit.to_hex(), first);
    assert_eq!(f.last_commit.to_hex(), head);
    assert_eq!((f.commits, f.blobs), (2, 1));
    assert_eq!(f.preview, format!("ghp_****{}", &token[token.len() - 4..]));

    // A value typed where an id belongs is refused and never echoed.
    let output = Command::cargo_bin("gitpoulpi")
        .unwrap()
        .env("GITPOULPI_CONFIG_DIR", config_dir())
        .args(["--json", "--session", session])
        .args(["--repo", &repo.root().to_string_lossy()])
        .args(["rewrite", "preview", "--remove-finding", &token])
        .current_dir(repo.root())
        .output()
        .unwrap();
    assert!(!output.status.success());
    let shown = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!shown.contains(&token), "{shown}");

    // Removed by id; the run answers with no value either.
    let (code, value) = run(
        &repo,
        session,
        None,
        &[
            "rewrite",
            "run",
            "--remove-finding",
            &f.id,
            "--signatures",
            "strip",
        ],
    );
    assert_eq!(code, 0, "{value}");
    assert!(!value.to_string().contains(&token));
    let log = repo.git(&["log", "-p", "--branches", "--tags"]);
    assert!(!log.contains(&token), "gone from every commit");
    assert!(log.contains("GITHUB_TOKEN=***REMOVED***"));
    let after: SecretScan =
        serde_json::from_value(ok(&repo, session, &["rewrite", "scan-secrets"])).unwrap();
    assert!(after.findings.is_empty());

    // Nothing the tool wrote holds the value: not the log, not the plan.
    assert_eq!(
        files_holding(config_dir(), &token),
        Vec::<std::path::PathBuf>::new()
    );
    assert_eq!(
        files_holding(&repo.root().join(".git/gitpoulpi"), &token),
        Vec::<std::path::PathBuf>::new()
    );
    // Undo brings it back and the scan finds it again with the same id.
    ok(&repo, session, &["undo"]);
    let again: SecretScan =
        serde_json::from_value(ok(&repo, session, &["rewrite", "scan-secrets"])).unwrap();
    assert_eq!(again.findings, scan.findings);
}

// ------------------------------------------------------ push, note, maps (S13)

fn origin_main(repo: &FixtureRepo) -> String {
    repo.git_in_remote(&["rev-parse", "main"])
        .trim()
        .to_string()
}

fn status_of(value: &Value, name: &str) -> String {
    value["results"]
        .as_array()
        .and_then(|r| r.iter().find(|r| r["name"] == name))
        .unwrap_or_else(|| panic!("no result for {name}: {value}"))["status"]
        .as_str()
        .unwrap()
        .to_string()
}

#[test]
fn a_pushed_main_is_protected_until_asked_and_the_push_is_leased_confirmed_and_noted() {
    let repo = pushed_fixture();
    let session = "cli-rw-push";
    let _daemon = Daemon::start(&repo, session);
    let old_remote = origin_main(&repo);
    let old_local = rev_parse(&repo, "main");

    // Protected by default: the preview is refused, typed.
    let plain = flags(&["rewrite", "preview", "--signatures", "strip"], &[]);
    let (code, value) = run(&repo, session, None, &plain);
    assert_eq!(code, 2, "{value}");
    assert_eq!(value["refused"]["reason"], "protected_ref", "{value}");

    // Asked for: rewritten and applied (it is on a remote, so --yes).
    let go = flags(
        &[
            "rewrite",
            "run",
            "--signatures",
            "strip",
            "--include-protected",
            "main",
        ],
        &["--yes"],
    );
    let applied = ok(&repo, session, &go);
    let run_id = applied["outcome"]["run"].as_str().unwrap().to_string();
    let new_main = rev_parse(&repo, "main");
    assert_ne!(new_main, old_remote);

    // Protected again at the push: nothing to confirm, nothing sent.
    let value = ok(&repo, session, &["rewrite", "push", "--run", &run_id]);
    assert_eq!(status_of(&value, "refs/heads/main"), "skipped_protected");
    assert_eq!(origin_main(&repo), old_remote);

    // Named: the dry run in front of the confirmation lists it.
    let ask = ["rewrite", "push", "--include-protected", "main"];
    let (code, value) = run(&repo, session, None, &ask);
    assert_eq!(code, 3, "{value}");
    let would = &value["needs_confirmation"]["would_push"];
    assert_eq!(status_of(would, "refs/heads/main"), "would_push");
    assert_eq!(would["results"][0]["expected_old"], old_remote.as_str());
    assert_eq!(origin_main(&repo), old_remote, "nothing was sent");

    // --dry-run answers and exits 0, still sending nothing.
    let mut dry = ask.to_vec();
    dry.push("--dry-run");
    let value = ok(&repo, session, &dry);
    assert_eq!(value["dry_run"], true);
    assert_eq!(status_of(&value, "refs/heads/main"), "would_push");
    assert_eq!(origin_main(&repo), old_remote);

    // The note and the maps do not need the push to have happened.
    let note = ok(&repo, session, &["rewrite", "note"]);
    assert_eq!(note["run"], run_id.as_str());
    assert_eq!(note["refs"][0]["short"], "main");
    assert_eq!(note["refs"][0]["kind"], "branch");
    assert!(note["text"]
        .as_str()
        .unwrap()
        .contains("git reset --hard origin/main"));
    let out = tempfile::tempdir().unwrap();
    let dest = out.path().join("maps");
    let maps = ok(
        &repo,
        session,
        &["rewrite", "maps", "--out", &dest.to_string_lossy()],
    );
    assert_eq!(maps["copied"], true);
    let commit_map = std::fs::read_to_string(dest.join("commit-map")).unwrap();
    assert!(commit_map.starts_with("old"), "{commit_map}");
    assert!(commit_map.contains(&old_remote), "{commit_map}");
    assert!(commit_map.contains(&new_main), "{commit_map}");
    assert!(std::fs::read_to_string(dest.join("ref-map"))
        .unwrap()
        .contains(&format!("{old_local} {new_main} refs/heads/main")));

    // --yes sends it.
    let mut yes = ask.to_vec();
    yes.push("--yes");
    let value = ok(&repo, session, &yes);
    assert_eq!(status_of(&value, "refs/heads/main"), "pushed");
    assert_eq!(origin_main(&repo), new_main);
}

#[test]
fn a_remote_that_moved_since_the_preview_is_rejected_with_exit_4() {
    let repo = pushed_fixture();
    let session = "cli-rw-push-moved";
    let _daemon = Daemon::start(&repo, session);
    let old_remote = origin_main(&repo);
    let go = flags(
        &[
            "rewrite",
            "run",
            "--signatures",
            "strip",
            "--include-protected",
            "main",
        ],
        &["--yes"],
    );
    ok(&repo, session, &go);

    // A collaborator pushes to main, and our tracking ref learns of it (an
    // auto-fetch): only the run's recorded lease still knows the difference.
    let tree = repo.git(&["rev-parse", &format!("{old_remote}^{{tree}}")]);
    let rival = repo.git(&["commit-tree", tree.trim(), "-p", &old_remote, "-m", "rival"]);
    let rival = rival.trim();
    repo.git(&[
        "push",
        "-q",
        "origin",
        &format!("{rival}:refs/heads/collab"),
    ]);
    let remote = repo.remote_path().unwrap().to_string_lossy().into_owned();
    repo.git(&["--git-dir", &remote, "update-ref", "refs/heads/main", rival]);
    repo.git(&["update-ref", "refs/remotes/origin/main", rival]);

    let (code, value) = run(
        &repo,
        session,
        None,
        &["rewrite", "push", "--include-protected", "main", "--yes"],
    );
    assert_eq!(code, 4, "{value}");
    assert_eq!(
        status_of(&value, "refs/heads/main"),
        "rejected_remote_moved"
    );
    assert_eq!(origin_main(&repo), rival, "the collaborator's commit stays");
}

// ----------------------------------------------------------------- purge (S15)

#[test]
fn purge_asks_first_is_blocked_by_a_stash_and_then_deletes_the_old_history_for_good() {
    let repo = fixture();
    let session = "cli-rw-purge";
    let _daemon = Daemon::start(&repo, session);
    let old_tip = rev_parse(&repo, "main");
    // A stash made before the rewrite sits on the old history.
    std::fs::write(repo.root().join("wip.txt"), "wip\n").unwrap();
    repo.git(&["add", "wip.txt"]);
    repo.git(&["stash", "push", "-q", "-m", "wip"]);

    let go = flags(
        &["rewrite", "run", "--all-local", "--signatures", "strip"],
        &[],
    );
    ok(&repo, session, &go);
    assert_ne!(rev_parse(&repo, "main"), old_tip);

    // Something still holds the old history: exit 4, typed, nothing changed.
    let (code, value) = run(&repo, session, None, &["rewrite", "purge", "--yes"]);
    assert_eq!(code, 4, "{value}");
    assert_eq!(value["outcome"]["status"], "blocked", "{value}");
    assert_eq!(value["outcome"]["blockers"][0]["kind"], "stash", "{value}");
    assert!(repo.try_git(&["cat-file", "-e", &old_tip]).is_some());
    repo.git(&["stash", "drop", "-q"]);

    // No --yes: exit 3 with the confirmation, which names the entry and what
    // is lost. Nothing is deleted.
    let (code, value) = run(&repo, session, None, &["rewrite", "purge"]);
    assert_eq!(code, 3, "{value}");
    let confirm = &value["needs_confirmation"];
    assert!(
        confirm["prompt"]
            .as_str()
            .unwrap()
            .contains("Undo will no longer be possible for"),
        "{value}"
    );
    let entries = confirm["check"]["entries"].as_array().unwrap();
    assert!(
        entries
            .iter()
            .any(|e| e["rewrite"] == true
                && e["label"].as_str().unwrap().starts_with("Rewrite history")),
        "{value}"
    );
    assert_eq!(confirm["check"]["outcome"]["status"], "ready", "{value}");
    assert!(repo.try_git(&["cat-file", "-e", &old_tip]).is_some());

    // With --yes: gone.
    let (code, value) = run(&repo, session, None, &["rewrite", "purge", "--yes"]);
    assert_eq!(code, 0, "{value}");
    assert_eq!(value["outcome"]["status"], "purged", "{value}");
    assert_eq!(value["outcome"]["remaining"], 0, "{value}");
    assert!(repo.try_git(&["cat-file", "-e", &old_tip]).is_none());
    repo.git(&["fsck", "--full", "--strict"]);

    // Undo is refused, typed: the entry is purged.
    let (code, value) = run(&repo, session, None, &["undo"]);
    assert_eq!(code, 2, "{value}");
    assert!(
        value["refused"]["reason"]["Purged"]["entry"].is_u64(),
        "{value}"
    );

    // Purging again: nothing to purge.
    let (code, value) = run(&repo, session, None, &["rewrite", "purge", "--yes"]);
    assert_eq!(code, 2, "{value}");
    assert_eq!(value["refused"]["reason"], "stale", "{value}");

    // The listing marks it.
    let (_, listing) = run(&repo, session, None, &["undo", "--list"]);
    assert!(listing.to_string().contains("Purged"), "{listing}");
}

/// `rewrite analyze` ranks a deleted large file first, with its commits and
/// state, folders and totals, `--top` caps the lists, and it changes no ref.
#[test]
fn analyze_lists_the_largest_blobs_folders_and_deleted_files() {
    use gitpoulpi_core::domain::rewrite::{BlobState, SizeAnalysis};

    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("alpha")
        .unwrap()
        .commit_file("old/dump.sql", &"d".repeat(9000), "add dump")
        .unwrap()
        .commit_file("live/data.bin", &"l".repeat(4000), "add data")
        .unwrap()
        .build();
    repo.git(&["rm", "-q", "old/dump.sql"]);
    repo.git(&["commit", "-q", "-m", "drop dump"]);
    let session = "cli-rw-analyze";
    let added = rev_parse(&repo, "HEAD~2");
    let dropped = rev_parse(&repo, "HEAD");
    let tip = dropped.clone();

    let value = ok(&repo, session, &["rewrite", "analyze"]);
    let found: SizeAnalysis = serde_json::from_value(value).unwrap();
    let first = &found.blobs[0];
    assert_eq!((first.path.as_str(), first.size), ("old/dump.sql", 9000));
    assert_eq!(first.state, BlobState::Deleted);
    assert_eq!(first.first_commit.to_hex(), added);
    assert_eq!(first.last_commit.to_hex(), dropped);
    assert_eq!(found.blobs[1].path, "live/data.bin");
    assert_eq!(found.blobs[1].state, BlobState::InHead);
    assert_eq!(found.folders[0].path, "old");
    assert!(!found.folders[0].in_head);
    assert_eq!(found.deleted[0].path, "old/dump.sql");
    assert_eq!(found.bytes_history_only, 9000);

    let capped: SizeAnalysis =
        serde_json::from_value(ok(&repo, session, &["rewrite", "analyze", "--top", "1"])).unwrap();
    assert_eq!(capped.blobs.len(), 1);
    assert_eq!(capped.blobs_cut, found.blobs_total - 1);
    assert_eq!(rev_parse(&repo, "HEAD"), tip, "a read");
}

/// A repository whose commits are signed with a generated ssh key (a test
/// double for the maintainer's setup); `None` without `ssh-keygen`.
fn signed_fixture(dir: &Path) -> Option<FixtureRepo> {
    let key = dir.join("key");
    let made = std::process::Command::new("ssh-keygen")
        .args(["-q", "-t", "ed25519", "-N", "", "-C", "cli-test", "-f"])
        .arg(&key)
        .output();
    if !matches!(&made, Ok(out) if out.status.success()) {
        eprintln!("SKIPPED: ssh-keygen is not available");
        return None;
    }
    let repo = fixture();
    let public = std::fs::read_to_string(dir.join("key.pub")).unwrap();
    std::fs::write(dir.join("allowed"), format!("* {public}")).unwrap();
    repo.git(&["config", "gpg.format", "ssh"]);
    repo.git(&["config", "user.signingkey", key.to_str().unwrap()]);
    repo.git(&[
        "config",
        "gpg.ssh.allowedSignersFile",
        dir.join("allowed").to_str().unwrap(),
    ]);
    repo.git(&["commit", "-q", "--allow-empty", "-S", "-m", "signed one"]);
    repo.git(&["commit", "-q", "--allow-empty", "-S", "-m", "signed two"]);
    Some(repo)
}

#[test]
fn check_signing_then_resign_and_remove_signatures_end_to_end() {
    let dir = tempfile::tempdir().unwrap();
    let Some(repo) = signed_fixture(dir.path()) else {
        return;
    };
    let session = "cli-rw-sign";

    let value = ok(&repo, session, &["rewrite", "check-signing"]);
    let check: SigningCheck = serde_json::from_value(value).unwrap();
    assert!(check.is_ok(), "{check:?}");
    assert!(check.millis > 0);

    // Signed commits change: no answer is a typed refusal, exit 2.
    let mut base = vec!["rewrite", "run"];
    base.extend(CASEY);
    let (code, value) = run(&repo, session, None, &base);
    assert_eq!(code, 2, "{value}");
    assert_eq!(value["refused"]["reason"], "signature_choice_needed");

    // Re-sign signed: the two commits come out signed and verify.
    base.extend(["--signatures", "resign-signed"]);
    let (code, value) = run(&repo, session, None, &base);
    assert_eq!(code, 0, "{value}");
    let plan: RewritePlan = serde_json::from_value(value["plan"].clone()).unwrap();
    assert_eq!(plan.counts.resigned, 2);
    assert!(repo.try_git(&["verify-commit", "HEAD"]).is_some());
    assert!(repo.try_git(&["verify-commit", "HEAD~1"]).is_some());

    // The rule needs no answer and leaves nothing signed.
    let (code, value) = run(
        &repo,
        session,
        None,
        &["rewrite", "run", "--remove-signatures"],
    );
    assert_eq!(code, 0, "{value}");
    assert!(repo.try_git(&["verify-commit", "HEAD"]).is_none());
    assert!(!repo.git(&["cat-file", "commit", "HEAD"]).contains("gpgsig"));

    // A broken key: the smoke test refuses with its typed reason.
    repo.git(&["config", "user.signingkey", "/nonexistent/key"]);
    let (code, value) = run(&repo, session, None, &["rewrite", "check-signing"]);
    assert_eq!(code, 2, "{value}");
    assert_eq!(value["refused"]["reason"], "signing_failed");
}

/// The trailer flags and `--update-message-hashes`, through the real binary:
/// the plan's counts and the messages `git log` shows.
#[test]
fn trailer_flags_edit_messages_and_hash_references_follow() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("alpha")
        .unwrap()
        .commit_file(
            "a.txt",
            "a\n",
            "add a\n\nSigned-off-by: Old Al <al@old.org>",
        )
        .unwrap()
        .build();
    let first = rev_parse(&repo, "HEAD");
    std::fs::write(repo.root().join("b.txt"), "b\n").unwrap();
    repo.git(&["add", "b.txt"]);
    repo.git(&["commit", "-q", "-m", &format!("fix {}", &first[..9])]);
    let session = "cli-rw-trailers";
    let value = ok(
        &repo,
        session,
        &[
            "rewrite",
            "run",
            "--trailer-add",
            "Reviewed-by=Rae <rae@example.com>",
            "--trailer-map",
            "Signed-off-by:@old.org=@new.org",
            "--trailer-remove",
            "Nope",
            "--update-message-hashes",
            "--signatures",
            "strip",
        ],
    );
    let plan: RewritePlan = serde_json::from_value(value["plan"].clone()).unwrap();
    assert_eq!(plan.counts.message_hashes_updated, 1);
    assert_eq!(plan.counts.message_hashes_left, 0);
    // add: every commit; remove of a missing key: none; map: one.
    let per_rule: Vec<u64> = plan.per_rule.iter().map(|r| r.commits).collect();
    assert_eq!(per_rule, [3, 0, 1]);
    let log = repo.git(&["log", "--reverse", "--format=%B%x00"]);
    let messages: Vec<&str> = log.split('\0').map(str::trim).collect();
    assert!(messages[1].contains("Signed-off-by: Old Al <al@new.org>"));
    assert!(messages[1].ends_with("Reviewed-by: Rae <rae@example.com>"));
    let new_first = rev_parse(&repo, "HEAD~1");
    assert!(
        messages[2].starts_with(&format!("fix {}", &new_first[..9])),
        "{}",
        messages[2]
    );
    assert_ne!(first, new_first);
}

/// S24: `--fix-message-encoding` through the real binary: the header goes,
/// the message is UTF-8, `encoding_left` counts what cannot be decided, and
/// `--fix-name-encoding` needs the first flag.
#[test]
fn fix_message_encoding_flags_rewrite_latin1_messages_to_utf8() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("alpha")
        .unwrap()
        .build();
    repo.commit_raw_message(b"caf\xe9 au lait\n", Some("ISO-8859-1"));
    repo.commit_raw_message(b"no header \xe9\n", None);
    let session = "cli-rw-encoding";
    let (code, _) = run(
        &repo,
        session,
        None,
        &["rewrite", "preview", "--fix-name-encoding"],
    );
    assert_eq!(code, 1, "the names flag needs the message flag");
    let value = ok(
        &repo,
        session,
        &[
            "rewrite",
            "run",
            "--fix-message-encoding",
            "--signatures",
            "strip",
        ],
    );
    let plan: RewritePlan = serde_json::from_value(value["plan"].clone()).unwrap();
    // The fixed commit and its child, which only gets the new parent.
    assert_eq!(plan.counts.commits_changed, 2);
    assert_eq!(plan.per_rule[0].commits, 1);
    assert_eq!(plan.counts.encoding_left, 1);
    assert_eq!(
        repo.git(&["log", "-1", "--format=%s", "HEAD~1"]).trim(),
        "caf\u{e9} au lait"
    );
    assert!(!repo
        .git(&["cat-file", "commit", "HEAD~1"])
        .contains("encoding"));
}

#[test]
fn trailer_flags_with_bad_values_are_errors_with_exit_1() {
    let repo = fixture();
    for args in [
        &["rewrite", "preview", "--trailer-add", "NoEquals"][..],
        &["rewrite", "preview", "--trailer-map", "Key:NoEquals"][..],
        &["rewrite", "preview", "--map-trailers"][..],
    ] {
        let (code, _) = run(&repo, "cli-rw-trailer-errors", None, args);
        assert_eq!(code, 1, "{args:?}");
    }
}

/// S21: `--lfs-import` / `--lfs-min-size` / `--lfs-export` through the real
/// binary: the counts in the JSON plan, a pointer in the commit, the real file
/// on disk, and the export giving the original blob back.
#[test]
fn lfs_flags_import_then_export_and_a_missing_object_is_refused() {
    gitpoulpi_testkit::require_git_lfs!("lfs_flags_import_then_export");
    let big: Vec<u8> = (0..4000u32).map(|i| (i * 13) as u8).collect();
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("alpha")
        .unwrap()
        .commit_bytes("art.bin", &big, "add art")
        .unwrap()
        .commit_bytes("tiny.bin", b"x\0y", "add tiny")
        .unwrap()
        .build();
    let before = rev_parse(&repo, "HEAD:art.bin");
    let session = "cli-rw-lfs";
    let value = ok(
        &repo,
        session,
        &[
            "rewrite",
            "run",
            "--lfs-import",
            "*.bin",
            "--lfs-min-size",
            "1000",
            "--signatures",
            "strip",
        ],
    );
    let plan: RewritePlan = serde_json::from_value(value["plan"].clone()).unwrap();
    assert_eq!(plan.counts.lfs_blobs, 1);
    assert_eq!(plan.counts.lfs_bytes, 4000);
    assert_eq!(plan.counts.lfs_objects, 1);
    assert!(plan.counts.lfs_run);
    let pointer = repo.git(&["show", "HEAD:art.bin"]);
    assert!(pointer.starts_with("version https://git-lfs.github.com/spec/v1\noid sha256:"));
    assert!(pointer.ends_with("size 4000"));
    assert_eq!(std::fs::read(repo.root().join("art.bin")).unwrap(), big);
    assert_eq!(rev_parse(&repo, "HEAD:tiny.bin").len(), 40);
    assert!(repo
        .git(&["show", "HEAD:.gitattributes"])
        .contains("*.bin filter=lfs"));

    // The object goes missing: the export is refused with the typed reason.
    let store = repo.root().join(".git/lfs/objects");
    let kept = repo.root().join(".git/lfs-kept");
    std::fs::rename(&store, &kept).unwrap();
    let (code, value) = run(
        &repo,
        session,
        None,
        &[
            "rewrite",
            "run",
            "--lfs-export",
            "*.bin",
            "--signatures",
            "strip",
        ],
    );
    assert_ne!(code, 0);
    assert_eq!(value["refused"]["reason"], "lfs_objects_missing", "{value}");
    std::fs::rename(&kept, &store).unwrap();

    let value = ok(
        &repo,
        session,
        &[
            "rewrite",
            "run",
            "--lfs-export",
            "*.bin",
            "--signatures",
            "strip",
        ],
    );
    let plan: RewritePlan = serde_json::from_value(value["plan"].clone()).unwrap();
    assert_eq!(plan.counts.lfs_restored, 1);
    assert_eq!(rev_parse(&repo, "HEAD:art.bin"), before);
}

/// S22 and S24bis through the flags: line endings, the exec bit and the file
/// encoding, with their path filters, and the typed counts.
#[test]
fn text_flags_convert_line_endings_the_exec_bit_and_the_encoding() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("alpha")
        .unwrap()
        .commit_bytes("win.txt", b"a\r\nb\r\n", "add win")
        .unwrap()
        .commit_bytes("old.txt", b"caf\xE9\n", "add latin")
        .unwrap()
        .commit_bytes("data.bin", b"\x00\r\n\x00", "add binary")
        .unwrap()
        .commit_bytes("run.sh", b"#!/bin/sh\n", "add script")
        .unwrap()
        .build();
    repo.git(&["update-index", "--chmod=+x", "run.sh"]);
    repo.git(&["commit", "-q", "-m", "make it executable"]);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(
            repo.root().join("run.sh"),
            std::fs::Permissions::from_mode(0o755),
        )
        .unwrap();
    }
    // The chmod above moved the file's ctime; git compares whole seconds, so
    // on a loaded machine (the commit and the chmod in different seconds) the
    // index entry is stale and the rewrite's checkout says "not uptodate".
    repo.git(&["update-index", "--refresh"]);
    let session = "cli-rw-text-flags";
    let value = ok(
        &repo,
        session,
        &[
            "rewrite",
            "run",
            "--eol",
            "lf",
            "--eol-paths",
            "*.txt",
            "*.bin",
            "--strip-exec",
            "*.sh",
            "--reencode",
            "utf-8",
            "--encoding-paths",
            "old.txt",
            "--signatures",
            "strip",
        ],
    );
    let plan: RewritePlan = serde_json::from_value(value["plan"].clone()).unwrap();
    assert_eq!(plan.counts.blobs_skipped_binary, 1, "the .bin with NULs");
    assert_eq!(plan.counts.blobs_skipped_undecodable, 0);
    assert_eq!(plan.counts.blobs_changed, 2, "win.txt and old.txt");
    assert_eq!(repo.git(&["show", "HEAD:win.txt"]), "a\nb");
    assert_eq!(repo.git(&["show", "HEAD:old.txt"]), "caf\u{e9}");
    assert!(repo
        .git(&["ls-tree", "HEAD", "run.sh"])
        .starts_with("100644"));
    assert_eq!(
        repo.git(&["cat-file", "blob", "HEAD:data.bin"]).len(),
        4,
        "the binary file is untouched"
    );
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
