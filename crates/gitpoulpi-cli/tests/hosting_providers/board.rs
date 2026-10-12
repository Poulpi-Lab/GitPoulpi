//! `hosting board` and `hosting board move` through the CLI: a notes folder
//! (no network) and Jira over a loopback mock. Assertions are on
//! deserialized JSON, the notes on disk and the requests the mock saw.

use crate::common::*;
use serde_json::Value;

fn names(board: &Value) -> Vec<String> {
    board["columns"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["name"].as_str().unwrap().to_string())
        .collect()
}

/// The references in a column, sorted (the cards come most recently changed first).
fn cards(board: &Value, column: &str) -> Vec<String> {
    let mut refs: Vec<String> = board["columns"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == column)
        .unwrap_or_else(|| panic!("no column {column} in {board}"))["cards"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["reference"].as_str().unwrap().to_string())
        .collect();
    refs.sort();
    refs
}

#[cfg(feature = "local-issues")]
#[test]
fn a_notes_folder_board_lists_its_columns_and_moves_a_card() {
    let cli = Cli::new("board-local", "[general]\n", &[]);
    let repo = fixture();
    let notes = repo.root().join("docs").join("issues");
    std::fs::create_dir_all(&notes).unwrap();
    for (id, status) in [(1, "todo"), (2, "doing"), (3, "todo")] {
        std::fs::write(
            notes.join(format!("{id}-note-{id}.md")),
            format!("---\nid: {id}\ntitle: Note {id}\nstatus: {status}\n---\nBody.\n"),
        )
        .unwrap();
    }
    cli.json(
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
    cli.json(repo.root(), &["hosting", "link", "issues", "--id", "notes"]);

    let board = cli.json(repo.root(), &["hosting", "board"]);
    assert_eq!(board["source"], "status");
    assert_eq!(board["movable"], true);
    assert_eq!(names(&board), ["todo", "doing", "done"]);
    assert_eq!(cards(&board, "todo"), ["#1", "#3"]);
    assert_eq!(cards(&board, "doing"), ["#2"]);
    assert!(cards(&board, "done").is_empty());
    let doing = &board["columns"][1]["cards"][0];
    assert_eq!(doing["title"], "Note 2");
    assert_eq!(doing["status"], "doing");
    assert_eq!(doing["state"], "open");
    // `board show` is the same thing.
    assert_eq!(cli.json(repo.root(), &["hosting", "board", "show"]), board);

    let moved = cli.json(repo.root(), &["hosting", "board", "move", "#3", "DOING"]);
    assert_eq!(moved["reference"], "#3");
    assert_eq!(moved["from"], "todo");
    assert_eq!(moved["column"], "doing");
    let note = std::fs::read_to_string(notes.join("3-note-3.md")).unwrap();
    assert!(note.contains("status: doing"), "{note}");
    let again = cli.json(repo.root(), &["hosting", "board"]);
    assert_eq!(cards(&again, "doing"), ["#2", "#3"]);

    // The number alone names the card too; the done column closes it.
    cli.json(repo.root(), &["hosting", "board", "move", "1", "done"]);
    let done = cli.json(repo.root(), &["hosting", "board"]);
    assert_eq!(cards(&done, "done"), ["#1"]);
    assert_eq!(done["columns"][2]["cards"][0]["state"], "closed");

    let unknown_column = cli.error(repo.root(), &["hosting", "board", "move", "1", "nowhere"]);
    assert!(
        unknown_column.contains("no column `nowhere`") && unknown_column.contains("todo, doing"),
        "{unknown_column}"
    );
    let unknown_card = cli.error(repo.root(), &["hosting", "board", "move", "99", "done"]);
    assert!(unknown_card.contains("no card `99`"), "{unknown_card}");
}

#[cfg(feature = "jira")]
mod jira {
    use super::*;

    const ISSUE: &str = r#"{"issues":[{"id":"10001","key":"ABC-12","fields":{"summary":"Crash on start",
        "status":{"name":"In Progress","statusCategory":{"key":"indeterminate"}},
        "issuetype":{"name":"Bug"},"labels":[],"updated":"2026-10-01T10:00:00.000+0000"}}],"isLast":true}"#;
    const STATUSES: &str = r#"[{"name":"Bug","statuses":[
        {"id":"1","name":"To Do","statusCategory":{"key":"new"}},
        {"id":"2","name":"In Progress","statusCategory":{"key":"indeterminate"}},
        {"id":"3","name":"Done","statusCategory":{"key":"done"}}]}]"#;
    const TRANSITIONS: &str = r#"{"transitions":[
        {"id":"31","name":"Finish","to":{"id":"3","name":"Done"}}]}"#;

    fn setup(name: &str) -> (Mock, Cli, gitpoulpi_testkit::FixtureRepo) {
        let mock = Mock::start(vec![
            route("GET", "/rest/api/3/search/jql", ISSUE),
            route("GET", "/rest/api/3/project/ABC/statuses", STATUSES),
            route("GET", "/rest/api/3/issue/ABC-12/transitions", TRANSITIONS),
            route("POST", "/rest/api/3/issue/ABC-12/transitions", "{}"),
        ]);
        let cli = Cli::new(
            name,
            &integration("jira", "jira_cloud", &mock.url(), "account = \"me@x.io\""),
            &[("jira", "tok")],
        );
        let repo = fixture();
        cli.json(
            repo.root(),
            &[
                "hosting",
                "link",
                "issues",
                "--id",
                "jira",
                "--project",
                "ABC",
            ],
        );
        (mock, cli, repo)
    }

    #[test]
    fn the_board_has_the_project_statuses_and_a_move_runs_the_transition() {
        let (mock, cli, repo) = setup("board-jira");
        let board = cli.json(repo.root(), &["hosting", "board"]);
        assert_eq!(names(&board), ["To Do", "In Progress", "Done"]);
        assert_eq!(cards(&board, "In Progress"), ["ABC-12"], "{board}");
        assert_eq!(board["columns"][1]["cards"][0]["kind"], "Bug");

        let moved = cli.json(repo.root(), &["hosting", "board", "move", "ABC-12", "done"]);
        assert_eq!(moved["from"], "In Progress");
        assert_eq!(moved["column"], "Done");
        let posts: Vec<_> = mock
            .to("/transitions")
            .into_iter()
            .filter(|r| r.method == "POST")
            .collect();
        assert_eq!(posts.len(), 1);
        let body: Value = serde_json::from_str(&posts[0].body).unwrap();
        assert_eq!(body["transition"]["id"], "31");
    }

    #[test]
    fn a_column_without_a_transition_fails_and_posts_nothing() {
        let (mock, cli, repo) = setup("board-jira-none");
        let err = cli.error(
            repo.root(),
            &["hosting", "board", "move", "ABC-12", "To Do"],
        );
        assert!(
            err.contains("no transition from In Progress to To Do"),
            "{err}"
        );
        assert!(mock.to("/transitions").iter().all(|r| r.method == "GET"));
    }
}
