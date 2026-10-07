//! Local Markdown issues through the CLI: a folder of notes is the issue
//! source (no token, no network). Assertions are on deserialized JSON and on
//! the notes on disk.

use crate::common::*;

fn cli(name: &str) -> Cli {
    Cli::new(name, "[general]\n", &[])
}

#[test]
fn a_folder_is_added_linked_listed_created_edited_and_closed() {
    let cli = cli("local-flow");
    let repo = fixture();
    let notes = repo.root().join("docs").join("issues");
    std::fs::create_dir_all(&notes).unwrap();
    // An existing note with a property GitPoulpi does not know.
    std::fs::write(
        notes.join("3-old-bug.md"),
        "---\nid: 3\ntitle: Old bug\nstatus: doing\nassignees: [ana]\nlabels: [bug]\ncssclasses: wide\n---\nThe body.\n",
    )
    .unwrap();

    // --path is the folder; a kind with no host needs no URL and no token.
    let added = cli.json(
        std::path::Path::new("."),
        &[
            "hosting",
            "integrations",
            "add",
            "--kind",
            "local_markdown",
            "--path",
            "docs/issues",
            "--id",
            "notes",
        ],
    );
    assert_eq!(added["added"]["kind"], "local_markdown");
    assert_eq!(added["added"]["category"], "issue_tracker");
    assert_eq!(added["added"]["url"], "docs/issues");
    let missing = cli.error(
        std::path::Path::new("."),
        &["hosting", "integrations", "add", "--kind", "local_markdown"],
    );
    assert!(missing.contains("--path"), "{missing}");

    // Linking without --project points at the profile's folder.
    let linked = cli.json(repo.root(), &["hosting", "link", "issues", "--id", "notes"]);
    assert_eq!(linked["project"], "docs/issues");
    assert!(read_state(&repo).contains("docs/issues"));

    let listed = cli.json(repo.root(), &["hosting", "issues"]);
    let rows = listed["issues"].as_array().unwrap();
    assert_eq!(rows.len(), 1, "{listed}");
    assert_eq!(rows[0]["number"], 3);
    assert_eq!(rows[0]["reference"], "#3");
    assert_eq!(rows[0]["status"], "doing");
    assert_eq!(rows[0]["labels"][0], "bug");

    // Create: ids are max + 1, the note is named <id>-<slug>.md.
    let created = cli.json(
        repo.root(),
        &[
            "hosting",
            "issue",
            "create",
            "--title",
            "Crash on empty name",
            "--body",
            "Steps.",
            "--label",
            "bug,ui",
            "--assignee",
            "kai",
            "--list",
            "todo",
        ],
    );
    assert_eq!(created["number"], 4);
    assert_eq!(created["reference"], "#4");
    let note = notes.join("4-crash-on-empty-name.md");
    let text = std::fs::read_to_string(&note).unwrap();
    assert!(text.starts_with("---\nid: 4\ntitle: Crash on empty name\nstatus: todo\nassignees: [kai]\nlabels: [bug, ui]\n"), "{text}");
    assert!(text.ends_with("Steps.\n"), "{text}");

    // Edit metadata: unknown properties and the body stay byte for byte.
    let edited = cli.json(
        repo.root(),
        &[
            "hosting",
            "issue",
            "edit",
            "3",
            "--add-label",
            "ui",
            "--remove-assignee",
            "ana",
        ],
    );
    assert_eq!(edited["labels"], serde_json::json!(["bug", "ui"]));
    assert_eq!(edited["assignees"], serde_json::json!([]));
    let old = std::fs::read_to_string(notes.join("3-old-bug.md")).unwrap();
    assert!(old.starts_with("---\nid: 3\ntitle: Old bug\nstatus: doing\nassignees: []\nlabels: [bug, ui]\ncssclasses: wide\nupdated: "), "{old}");
    assert!(old.ends_with("---\nThe body.\n"), "{old}");

    // Close and reopen: status moves to the done status and back.
    let closed = cli.json(repo.root(), &["hosting", "issue", "close", "3"]);
    assert_eq!(
        (closed["status"].as_str(), closed["state"].as_str()),
        (Some("done"), Some("closed"))
    );
    let open_only = cli.json(repo.root(), &["hosting", "issues"]);
    assert_eq!(open_only["issues"].as_array().unwrap().len(), 1);
    let all = cli.json(repo.root(), &["hosting", "issues", "--state", "all"]);
    assert_eq!(all["issues"].as_array().unwrap().len(), 2);
    let reopened = cli.json(repo.root(), &["hosting", "issue", "reopen", "3"]);
    assert_eq!(reopened["status"], "todo");
    let moved = cli.json(
        repo.root(),
        &["hosting", "issue", "move", "3", "--status", "DOING"],
    );
    assert_eq!(moved["status"], "doing", "the known spelling is kept");

    // `issue options` exposes the statuses (the board's columns).
    let options = cli.json(repo.root(), &["hosting", "issue", "options"]);
    let lists: Vec<&str> = options["lists"]
        .as_array()
        .unwrap()
        .iter()
        .map(|l| l["id"].as_str().unwrap())
        .collect();
    assert_eq!(lists, ["todo", "doing", "done"]);

    // References: `#3` in a message and `3-slug` as a branch resolve to the note.
    let refs = cli.json(
        repo.root(),
        &["hosting", "refs", "--message", "fixes #3 today"],
    );
    assert_eq!(refs["issues_linked"], true);
    assert_eq!(refs["refs"][0]["reference"], "#3");
    assert!(
        refs["refs"][0]["url"]
            .as_str()
            .unwrap()
            .ends_with("/3-old-bug.md"),
        "{refs}"
    );
    let branch = cli.json(repo.root(), &["hosting", "refs", "--branch", "4-crash-fix"]);
    assert_eq!(branch["refs"][0]["reference"], "#4");

    // `open issue` names the note.
    let opened = cli.json(repo.root(), &["hosting", "open", "issue", "4"]);
    assert!(
        opened["url"]
            .as_str()
            .unwrap()
            .ends_with("/4-crash-on-empty-name.md"),
        "{opened}"
    );
}

#[test]
fn the_base_is_written_once_and_a_vault_is_not_required() {
    let cli = cli("local-base");
    let repo = fixture();
    let notes = repo.root().join("issues");
    std::fs::create_dir_all(&notes).unwrap();
    cli.json(
        std::path::Path::new("."),
        &[
            "hosting",
            "integrations",
            "add",
            "--kind",
            "local_markdown",
            "--path",
            "issues",
            "--id",
            "notes",
        ],
    );
    cli.json(repo.root(), &["hosting", "link", "issues", "--id", "notes"]);
    let made = cli.json(repo.root(), &["hosting", "issue", "base"]);
    let base = std::path::PathBuf::from(made["base"].as_str().unwrap());
    // Canonical on both sides: macOS temp dirs live behind the
    // `/var` -> `/private/var` symlink.
    assert_eq!(
        base.canonicalize().unwrap(),
        notes.join("issues.base").canonicalize().unwrap()
    );
    let text = std::fs::read_to_string(&base).unwrap();
    assert!(
        text.contains("type: kanban") && text.contains("property: note.status"),
        "{text}"
    );
    // Never over an existing file.
    std::fs::write(&base, "mine").unwrap();
    let again = cli.error(repo.root(), &["hosting", "issue", "base"]);
    assert!(again.contains("already exists"), "{again}");
    assert_eq!(std::fs::read_to_string(&base).unwrap(), "mine");
}

#[test]
fn an_absolute_folder_and_the_statuses_of_the_config_are_honoured() {
    let vault = tempfile::tempdir().unwrap();
    let notes = vault.path().join("Issues");
    std::fs::create_dir_all(&notes).unwrap();
    let config = format!(
        "[general]\n\n[[integrations]]\nid = \"v\"\nkind = \"local_markdown\"\nurl = \"{}\"\ntoken_ref = \"integration:v\"\n\n[integrations.local]\nstatuses = [\"backlog\", \"review\", \"shipped\"]\ndone_status = \"shipped\"\nreference_prefix = \"LOC\"\nme = \"kai\"\n",
        notes.display().to_string().replace('\\', "/")
    );
    let cli = Cli::new("local-abs", &config, &[]);
    let repo = fixture();
    cli.json(repo.root(), &["hosting", "link", "issues", "--id", "v"]);
    let created = cli.json(
        repo.root(),
        &[
            "hosting",
            "issue",
            "create",
            "--title",
            "First",
            "--assignee",
            "kai",
        ],
    );
    assert_eq!(created["reference"], "LOC-1");
    let text = std::fs::read_to_string(notes.join("1-first.md")).unwrap();
    assert!(
        text.contains("status: backlog\n"),
        "first configured status: {text}"
    );
    let mine = cli.json(repo.root(), &["hosting", "issues", "--assigned"]);
    assert_eq!(mine["issues"].as_array().unwrap().len(), 1);
    cli.json(repo.root(), &["hosting", "issue", "close", "1"]);
    assert!(std::fs::read_to_string(notes.join("1-first.md"))
        .unwrap()
        .contains("status: shipped\n"));
    let reopened = cli.json(repo.root(), &["hosting", "issue", "reopen", "1"]);
    assert_eq!(reopened["status"], "backlog");
}
