//! End-to-end tests for the multi-repository CLI surface: `repo
//! discover`, `repo open/close/list`, `repo recent`, and `clone`.
//!
//! Same discipline as `commands.rs` (real binary, real fixture repository,
//! assertions on deserialized JSON, a per-test `--session`), plus one extra
//! isolation: every case gets **its own** `GITPOULPI_CONFIG_DIR`, because
//! the session file and the recent-repositories file are written there and
//! a shared one would have cases overwriting each other's tab list.

use std::path::Path;

use assert_cmd::Command;
use gitpoulpi_core::domain::{DiscoveredRepo, RepoRef};
use gitpoulpi_testkit::FixtureRepoBuilder;
use serde_json::Value;

struct Cli {
    config_dir: tempfile::TempDir,
    session: String,
}

impl Cli {
    fn new(session: &str) -> Self {
        Self {
            config_dir: tempfile::tempdir().expect("temp config dir"),
            session: session.to_string(),
        }
    }

    fn command(&self, cwd: &Path, args: &[&str]) -> Command {
        let mut cmd = Command::cargo_bin("gitpoulpi").unwrap();
        cmd.env("GITPOULPI_CONFIG_DIR", self.config_dir.path())
            .arg("--json")
            .args(["--session", &self.session])
            .args(args)
            .current_dir(cwd);
        cmd
    }

    fn json(&self, cwd: &Path, args: &[&str]) -> Value {
        let output = self.command(cwd, args).output().expect("run gitpoulpi");
        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        assert!(
            output.status.success(),
            "`gitpoulpi {}` failed\nstdout: {stdout}\nstderr: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr)
        );
        serde_json::from_str(&stdout)
            .unwrap_or_else(|e| panic!("output was not JSON ({e}): {stdout}"))
    }

    /// The JSON body of a command that is expected to *fail*. The error
    /// path emits JSON too when `--json` is passed, which is the whole
    /// point of the contract - a scripting caller must never have to
    /// parse a human sentence.
    fn json_err(&self, cwd: &Path, args: &[&str]) -> Value {
        let output = self.command(cwd, args).output().expect("run gitpoulpi");
        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        assert!(
            !output.status.success(),
            "`gitpoulpi {}` unexpectedly succeeded: {stdout}",
            args.join(" ")
        );
        serde_json::from_str(&stdout)
            .unwrap_or_else(|e| panic!("error output was not JSON ({e}): {stdout}"))
    }
}

fn parse<T: serde::de::DeserializeOwned>(value: Value) -> T {
    serde_json::from_value(value).expect("payload did not match its domain type")
}

/// Paths cross a process boundary here, so they are compared through
/// `canonicalize` - macOS puts temp dirs under a `/var` symlink to
/// `/private/var`, and Windows hands back both short (8.3) and long
/// spellings of the same directory.
fn same_path(a: &Path, b: &Path) -> bool {
    match (a.canonicalize(), b.canonicalize()) {
        (Ok(a), Ok(b)) => a == b,
        _ => a == b,
    }
}

#[test]
fn discover_answers_the_same_repository_from_the_root_and_from_inside_it() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("src/deep/file.txt", "x", "initial")
        .unwrap()
        .build();
    let cli = Cli::new("multirepo-discover");

    let from_root: DiscoveredRepo = parse(cli.json(repo.root(), &["repo", "discover"]));
    assert!(same_path(&from_root.repo.root, repo.root()));
    assert_eq!(from_root.name, "repo");
    assert_eq!(from_root.head_branch.as_deref(), Some("main"));
    assert!(!from_root.is_bare);
    assert!(!from_root.is_worktree);

    let subdir = repo.root().join("src").join("deep");
    let from_subdir: DiscoveredRepo = parse(cli.json(&subdir, &["repo", "discover"]));
    assert_eq!(
        from_subdir.repo.id, from_root.repo.id,
        "a subdirectory and the root must be one repo to the daemon"
    );
}

/// "Not a git repo" has to be a clean, JSON-shaped failure - the same
/// error the open-repository flow shows the user.
#[test]
fn discover_rejects_a_directory_that_is_not_a_repository() {
    let plain = tempfile::tempdir().unwrap();
    let cli = Cli::new("multirepo-not-a-repo");

    let error = cli.json_err(plain.path(), &["repo", "discover"]);
    assert!(
        error["error"].is_string(),
        "the error path must emit JSON too: {error}"
    );
}

#[test]
fn opening_two_repositories_puts_both_on_one_session() {
    let first = FixtureRepoBuilder::new()
        .unwrap()
        .commit("a")
        .unwrap()
        .build();
    let second = FixtureRepoBuilder::new()
        .unwrap()
        .commit("b")
        .unwrap()
        .build();
    let cli = Cli::new("multirepo-two");

    cli.json(
        first.root(),
        &["repo", "open", &first.root().to_string_lossy()],
    );
    let opened = cli.json(
        first.root(),
        &["repo", "open", &second.root().to_string_lossy()],
    );
    assert_eq!(opened["tab_index"], 1);

    let listed = cli.json(first.root(), &["repo", "list"]);
    let tabs = listed["tabs"].as_array().expect("tabs array");
    assert_eq!(tabs.len(), 2, "both repositories must be open: {listed}");
    assert_eq!(
        tabs.iter().filter(|t| t["active"] == true).count(),
        1,
        "exactly one tab is focused"
    );
    assert_eq!(
        tabs[1]["active"], true,
        "the most recently opened repository is the focused one"
    );

    // Closing one leaves the other, and the repository itself is
    // untouched.
    cli.json(
        first.root(),
        &["repo", "close", &second.root().to_string_lossy()],
    );
    let listed = cli.json(first.root(), &["repo", "list"]);
    assert_eq!(listed["tabs"].as_array().unwrap().len(), 1);
    assert!(second.root().join(".git").exists());
}

/// Reopening must move the entry, not add a second tab for the same
/// repository.
#[test]
fn reopening_a_repository_focuses_its_tab_instead_of_duplicating_it() {
    let first = FixtureRepoBuilder::new()
        .unwrap()
        .commit("a")
        .unwrap()
        .build();
    let second = FixtureRepoBuilder::new()
        .unwrap()
        .commit("b")
        .unwrap()
        .build();
    let cli = Cli::new("multirepo-reopen");

    cli.json(
        first.root(),
        &["repo", "open", &first.root().to_string_lossy()],
    );
    cli.json(
        first.root(),
        &["repo", "open", &second.root().to_string_lossy()],
    );
    let again = cli.json(
        first.root(),
        &["repo", "open", &first.root().to_string_lossy()],
    );

    assert_eq!(again["tab_index"], 0, "must reuse the existing tab");
    let listed = cli.json(first.root(), &["repo", "list"]);
    assert_eq!(listed["tabs"].as_array().unwrap().len(), 2);
    assert_eq!(listed["tabs"][0]["active"], true);
}

#[test]
fn opening_a_repository_records_it_in_the_recent_list() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("a")
        .unwrap()
        .build();
    let cli = Cli::new("multirepo-recent");

    assert!(
        cli.json(repo.root(), &["repo", "recent"])["repos"]
            .as_array()
            .unwrap()
            .is_empty(),
        "a fresh config dir has no recents"
    );

    cli.json(
        repo.root(),
        &["repo", "open", &repo.root().to_string_lossy()],
    );

    let recents = cli.json(repo.root(), &["repo", "recent"]);
    let repos = recents["repos"].as_array().unwrap();
    assert_eq!(repos.len(), 1);
    assert_eq!(repos[0]["name"], "repo");
    assert_eq!(repos[0]["branch"], "main");
    assert_eq!(repos[0]["exists"], true);
}

/// A repository deleted from disk stays in the list, flagged, until
/// something prunes it - so the user sees why the entry is grayed out
/// rather than finding it silently gone.
#[test]
fn a_repository_gone_from_disk_is_flagged_and_prunable() {
    let dir = tempfile::tempdir().unwrap();
    let cwd = dir.path().to_path_buf();
    let repo_dir = tempfile::tempdir().unwrap();
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("a")
        .unwrap()
        .build();
    let cli = Cli::new("multirepo-missing");

    cli.json(
        repo.root(),
        &["repo", "open", &repo.root().to_string_lossy()],
    );
    drop(repo);
    drop(repo_dir);

    let recents = cli.json(&cwd, &["repo", "recent"]);
    let repos = recents["repos"].as_array().unwrap();
    assert_eq!(repos.len(), 1, "a missing repo is kept, not forgotten");
    assert_eq!(repos[0]["exists"], false);

    let pruned = cli.json(&cwd, &["repo", "recent", "--prune"]);
    assert_eq!(pruned["pruned"], 1);
    assert!(pruned["repos"].as_array().unwrap().is_empty());
}

/// `repo recent` must answer outside any repository - it is how the
/// open-repository flow is reached from a home directory.
#[test]
fn recent_works_outside_a_repository() {
    let plain = tempfile::tempdir().unwrap();
    let cli = Cli::new("multirepo-outside");
    let recents = cli.json(plain.path(), &["repo", "recent"]);
    assert!(recents["repos"].as_array().unwrap().is_empty());
}

/// Clone end to end over a filesystem path - no network - and then
/// `discover` the result, which is the whole clone-then-open-a-tab
/// sequence the TUI performs.
#[test]
fn clone_produces_a_repository_that_discover_then_agrees_with() {
    let source = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .commit("second")
        .unwrap()
        .build();
    let dest_dir = tempfile::tempdir().unwrap();
    let dest = dest_dir.path().join("cloned");
    let cli = Cli::new("multirepo-clone");

    let cloned: RepoRef = parse(cli.json(
        dest_dir.path(),
        &[
            "clone",
            &source.root().to_string_lossy(),
            "--dest",
            &dest.to_string_lossy(),
        ],
    ));
    assert!(same_path(&cloned.root, &dest));

    let found: DiscoveredRepo = parse(cli.json(&dest, &["repo", "discover"]));
    assert_eq!(
        found.repo.id, cloned.id,
        "clone and discover must produce the same RepoId, or the new tab \
         would key a different daemon entry than the clone did"
    );
    assert_eq!(found.head_branch.as_deref(), Some("main"));

    // And it is a usable repository through the same session.
    let log = cli.json(&dest, &["log"]);
    assert_eq!(log["nodes"].as_array().unwrap().len(), 2);
}

/// `--branch` has to select what is checked out, through the whole
/// stack, not merely be accepted by clap.
#[test]
fn clone_branch_selects_what_is_checked_out() {
    let source = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .branch("side")
        .unwrap()
        .commit("on side")
        .unwrap()
        .checkout("main")
        .unwrap()
        .build();
    let dest_dir = tempfile::tempdir().unwrap();
    let dest = dest_dir.path().join("cloned");
    let cli = Cli::new("multirepo-clone-branch");

    cli.json(
        dest_dir.path(),
        &[
            "clone",
            &source.root().to_string_lossy(),
            "--dest",
            &dest.to_string_lossy(),
            "--branch",
            "side",
        ],
    );

    let found: DiscoveredRepo = parse(cli.json(&dest, &["repo", "discover"]));
    assert_eq!(found.head_branch.as_deref(), Some("side"));
}

/// With nothing configured, the hosting surface must teach the shape of
/// the table rather than merely reporting its absence - and must not
/// need a daemon or a repository to say so.
#[test]
fn hosting_with_no_integrations_explains_how_to_configure_one() {
    let plain = tempfile::tempdir().unwrap();
    let cli = Cli::new("multirepo-hosting-empty");

    let listed = cli.json(plain.path(), &["hosting", "list"]);
    assert!(listed["integrations"].as_array().unwrap().is_empty());

    let error = cli.json_err(plain.path(), &["hosting", "projects"]);
    let message = error["error"].as_str().expect("a JSON error string");
    for expected in ["[[integrations]]", "token_ref", "GITPOULPI_TOKEN_"] {
        assert!(
            message.contains(expected),
            "missing {expected} in:\n{message}"
        );
    }
}

/// Configured integrations are listed with their derived API base and
/// **without** their tokens - the config carries a reference, and the
/// output must not upgrade that into a secret in shell history.
#[test]
fn hosting_list_reports_the_derived_api_base_and_never_a_token() {
    let plain = tempfile::tempdir().unwrap();
    let cli = Cli::new("multirepo-hosting-list");
    std::fs::write(
        cli.config_dir.path().join("config.toml"),
        r#"
[[integrations]]
id = "corp"
kind = "gitlab"
url = "https://gitlab.example.com"
token_ref = "integration:corp"

[[integrations]]
id = "gh"
kind = "github"
url = "https://github.com"
token_ref = "integration:gh"
"#,
    )
    .unwrap();

    let listed = cli.json(plain.path(), &["hosting", "list"]);
    let rows = listed["integrations"].as_array().unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0]["api_base"], "https://gitlab.example.com/api/v4");
    assert_eq!(rows[1]["api_base"], "https://api.github.com");
    assert_eq!(rows[0]["token_ref"], "integration:corp");

    let raw = serde_json::to_string(&listed).unwrap();
    assert!(
        !raw.contains("\"token\""),
        "a token field must never appear in this output: {raw}"
    );

    // With two configured, an unnamed one is an error rather than a
    // silent pick.
    let error = cli.json_err(plain.path(), &["hosting", "projects"]);
    assert!(error["error"].as_str().unwrap().contains("--id"));
}

/// `hosting integrations add|list|remove` manage `[[integrations]]`
/// through the CLI, with no daemon, no repository and no token needed.
#[test]
fn hosting_integrations_add_list_remove_round_trip() {
    let plain = tempfile::tempdir().unwrap();
    let cli = Cli::new("multirepo-hosting-integrations");

    let added = cli.json(
        plain.path(),
        &[
            "hosting",
            "integrations",
            "add",
            "--kind",
            "gitlab_self_managed",
            "--url",
            "gitlab.example.com/",
            "--id",
            "corp",
            "--name",
            "Corp GitLab",
        ],
    );
    assert_eq!(added["added"]["id"], "corp");
    assert_eq!(added["added"]["url"], "https://gitlab.example.com");
    assert_eq!(
        added["added"]["api_base"],
        "https://gitlab.example.com/api/v4"
    );

    cli.json(
        plain.path(),
        &["hosting", "integrations", "add", "--kind", "github"],
    );

    let listed = cli.json(plain.path(), &["hosting", "integrations", "list"]);
    let rows = listed["integrations"].as_array().unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0]["name"], "Corp GitLab");
    assert_eq!(rows[0]["token_ref"], "integration:corp");
    assert_eq!(rows[1]["id"], "github");
    assert_eq!(rows[1]["api_base"], "https://api.github.com");
    assert!(!serde_json::to_string(&listed)
        .unwrap()
        .contains("\"token\""));

    // The written file is what the rest of GitPoulpi reads.
    let raw = std::fs::read_to_string(cli.config_dir.path().join("config.toml")).unwrap();
    assert!(raw.contains("gitlab_self_managed"), "{raw}");

    let duplicate = cli.json_err(
        plain.path(),
        &[
            "hosting",
            "integrations",
            "add",
            "--kind",
            "github",
            "--id",
            "corp",
        ],
    );
    assert!(duplicate["error"]
        .as_str()
        .unwrap()
        .contains("already exists"));

    let removed = cli.json(plain.path(), &["hosting", "integrations", "remove", "corp"]);
    assert_eq!(removed["removed"], "corp");
    let listed = cli.json(plain.path(), &["hosting", "integrations", "list"]);
    assert_eq!(listed["integrations"].as_array().unwrap().len(), 1);
}

/// A token is never accepted as an argument: `login` without
/// `--token-stdin` refuses and says why.
#[test]
fn hosting_login_requires_the_token_on_stdin() {
    let plain = tempfile::tempdir().unwrap();
    let cli = Cli::new("multirepo-hosting-login");
    cli.json(
        plain.path(),
        &["hosting", "integrations", "add", "--kind", "github"],
    );
    let error = cli.json_err(plain.path(), &["hosting", "login"]);
    assert!(error["error"].as_str().unwrap().contains("--token-stdin"));
}

/// Generating a key must work with no host configured at all, and must
/// produce a real OpenSSH pair on disk.
#[test]
fn an_ssh_key_can_be_generated_without_uploading_it() {
    let dir = tempfile::tempdir().unwrap();
    let key_path = dir.path().join("keys").join("id_ed25519");
    let cli = Cli::new("multirepo-sshkey");

    let generated = cli.json(
        dir.path(),
        &[
            "hosting",
            "ssh-key",
            "create",
            "--no-upload",
            "--path",
            &key_path.to_string_lossy(),
            "--title",
            "gitpoulpi@test",
        ],
    );

    assert!(generated["public_key"]
        .as_str()
        .unwrap()
        .starts_with("ssh-ed25519 "));
    assert!(generated["fingerprint"]
        .as_str()
        .unwrap()
        .starts_with("SHA256:"));
    assert!(generated["uploaded"].is_null());
    assert!(key_path.exists());
    assert!(key_path.with_extension("pub").exists());

    // Running it again must refuse rather than revoke the first key.
    let error = cli.json_err(
        dir.path(),
        &[
            "hosting",
            "ssh-key",
            "create",
            "--no-upload",
            "--path",
            &key_path.to_string_lossy(),
        ],
    );
    assert!(error["error"].as_str().unwrap().contains("already exists"));
}

/// `repo init` creates the repository, opens it as a tab of this
/// session, and reports the `RepoRef` every later request will be keyed
/// on - the same three things the TUI's Create form does, through the
/// same daemon operation.
#[test]
fn init_creates_a_repository_and_opens_it_as_a_tab() {
    let work = tempfile::tempdir().unwrap();
    let dest = work.path().join("brand").join("new");
    let cli = Cli::new("multirepo-init");

    let made = cli.json(
        work.path(),
        &[
            "repo",
            "init",
            dest.to_str().unwrap(),
            "--initial-branch",
            "trunk",
        ],
    );
    let repo: RepoRef = parse(made["repo"].clone());
    assert!(same_path(&repo.root, &dest));
    assert!(dest.join(".git").exists(), "a repository was made");

    // Discovering it afterwards must agree, or the tab that opens next
    // is a different repository to the daemon than the one just made.
    let found: DiscoveredRepo = parse(cli.json(&dest, &["repo", "discover"]));
    assert_eq!(found.repo.id, repo.id);

    // ...and it is a tab of this session.
    let tabs = cli.json(work.path(), &["repo", "list"]);
    assert!(
        tabs.to_string().contains("new"),
        "the new repository is a tab: {tabs}"
    );
}

/// Initialising over an existing repository is refused rather than
/// silently re-initializing it - the contract both backends carry,
/// reaching the CLI as a JSON error.
#[test]
fn init_refuses_a_path_that_is_already_a_repository() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    let cli = Cli::new("multirepo-init-existing");

    let error = cli.json_err(
        repo.root(),
        &["repo", "init", repo.root().to_str().unwrap()],
    );
    assert!(
        error["error"]
            .as_str()
            .is_some_and(|e| e.contains("already a git repository")),
        "{error}"
    );
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
