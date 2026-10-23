//! Split-plan prompt, parser and runner, driven by fixture answers.

use std::path::PathBuf;
use std::sync::Mutex;

use gitpoulpi_ai::assist::{split_pieces, to_parts, Job};
use gitpoulpi_ai::{parse, prompts, AiClient, Error, Result};
use gitpoulpi_config::{AiAction, AiConfig};
use gitpoulpi_core::domain::{
    validate_split, DiffLine, FileStatus, Hunk, HunkId, LineOrigin, Oid, SplitFile, SplitPiece,
    SplitSource, WholeReason,
};
use gitpoulpi_secrets::TokenStore;

fn fixture(name: &str) -> String {
    let path = format!(
        "{}/tests/fixtures/split_commit/{name}.txt",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

fn hunk(id: u32, start: u32, lines: &[(LineOrigin, &str)]) -> Hunk {
    Hunk {
        id: HunkId(id),
        old_start: start,
        old_lines: 3,
        new_start: start,
        new_lines: 5,
        lines: lines
            .iter()
            .map(|(origin, content)| DiffLine {
                origin: origin.clone(),
                content: format!("{content}\n"),
                staged: false,
            })
            .collect(),
    }
}

/// parse.rs: two hunks (H1, H2); cli.rs: one (H3); logo.png: whole (F4).
fn source() -> SplitSource {
    use LineOrigin::{Add, Context};
    SplitSource {
        commit: Oid([1; 20]),
        parent: Some(Oid([2; 20])),
        message: "feat: add parser and --strict flag".into(),
        sign_default: false,
        files: vec![
            SplitFile {
                path: PathBuf::from("src/parse.rs"),
                status: FileStatus::Modified,
                whole_only: None,
                hunks: vec![
                    hunk(0, 10, &[(Context, "fn parse() {"), (Add, "    lex();")]),
                    hunk(1, 40, &[(Add, "/// docs"), (Context, "fn lex() {}")]),
                ],
            },
            SplitFile {
                path: PathBuf::from("src/cli.rs"),
                status: FileStatus::Modified,
                whole_only: None,
                hunks: vec![hunk(0, 1, &[(Add, "    strict: bool,")])],
            },
            SplitFile {
                path: PathBuf::from("assets/logo.png"),
                status: FileStatus::Added,
                whole_only: Some(WholeReason::Binary),
                hunks: vec![],
            },
        ],
    }
}

fn ids() -> Vec<String> {
    split_pieces(&source())
        .1
        .into_iter()
        .map(|(id, _)| id)
        .collect()
}

#[test]
fn pieces_are_numbered_across_files_and_whole_files_are_f() {
    let (prompt, pieces) = split_pieces(&source());
    assert_eq!(ids(), ["H1", "H2", "H3", "F4"]);
    assert_eq!(prompt.len(), 4);
    assert_eq!(
        pieces[2].1,
        SplitPiece::Hunk {
            path: PathBuf::from("src/cli.rs"),
            hunk: HunkId(0)
        }
    );
    assert_eq!(
        pieces[3].1,
        SplitPiece::File {
            path: PathBuf::from("assets/logo.png")
        }
    );
    let text = prompts::split_commit(&source().message, &prompt, 10_000);
    assert!(
        text.contains("### H1 src/parse.rs hunk 1/2 @@ -10,3 +10,5 @@"),
        "{text}"
    );
    assert!(text.contains("+    lex();"), "{text}");
    assert!(
        text.contains("### F4 assets/logo.png (binary, whole file)"),
        "{text}"
    );
}

#[test]
fn good_answers_parse_in_every_shape() {
    for name in ["ok_plain", "ok_fenced", "prose_around", "lowercase_ids"] {
        let plan =
            parse::split_plan(&fixture(name), &ids()).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!(plan.len(), 2, "{name}");
        assert_eq!(plan[0].message, "feat: parser", "{name}");
        assert_eq!(plan[0].pieces, ["H1", "H2"], "{name}");
        assert_eq!(plan[1].pieces, ["H3", "F4"], "{name}");
        let (_, map) = split_pieces(&source());
        let parts = to_parts(&plan, &map);
        assert_eq!(validate_split(&source(), &parts), Ok(()), "{name}");
    }
}

#[test]
fn bad_answers_are_rejected_with_a_reason() {
    for (name, needle) in [
        ("missing_piece", "leaves out: F4"),
        ("duplicate", "more than one commit"),
        ("unknown_id", "H9"),
        ("single_commit", "at least two"),
        ("empty_message", "no message"),
    ] {
        match parse::split_plan(&fixture(name), &ids()) {
            Err(Error::Invalid(m)) => assert!(m.contains(needle), "{name}: {m}"),
            other => panic!("{name}: {other:?}"),
        }
    }
    assert!(matches!(
        parse::split_plan(&fixture("malformed"), &ids()),
        Err(Error::Decode(_))
    ));
}

struct Fixed;
impl TokenStore for Fixed {
    fn store(&self, _: &str, _: &str) -> gitpoulpi_secrets::Result<()> {
        Ok(())
    }
    fn load(&self, _: &str) -> gitpoulpi_secrets::Result<Option<String>> {
        Ok(None)
    }
    fn delete(&self, _: &str) -> gitpoulpi_secrets::Result<()> {
        Ok(())
    }
}

struct Script(Mutex<Vec<String>>, Mutex<Vec<String>>);
#[async_trait::async_trait]
impl AiClient for Script {
    async fn complete(&self, _system: &str, user: &str) -> Result<String> {
        self.1.lock().unwrap().push(user.to_string());
        Ok(self.0.lock().unwrap().remove(0))
    }
}

fn job() -> Job {
    let config = AiConfig {
        enabled: true,
        ..AiConfig::default()
    };
    Job::prepare(&config, AiAction::SplitCommit, None, &Fixed).unwrap()
}

fn run(answers: &[&str]) -> (Result<Vec<gitpoulpi_core::domain::SplitPart>>, Vec<String>) {
    let client = Script(
        Mutex::new(answers.iter().map(|a| a.to_string()).collect()),
        Mutex::default(),
    );
    let out = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(job().plan_split_with(&client, &source()));
    let sent = client.1.lock().unwrap().clone();
    (out, sent)
}

#[test]
fn a_good_first_answer_needs_one_call() {
    let (out, sent) = run(&[&fixture("ok_fenced")]);
    let parts = out.unwrap();
    assert_eq!(parts.len(), 2);
    assert_eq!(parts[0].pieces.len(), 2);
    assert_eq!(sent.len(), 1);
}

#[test]
fn a_rejected_answer_is_retried_once_with_the_reason() {
    let (out, sent) = run(&[&fixture("missing_piece"), &fixture("ok_plain")]);
    assert_eq!(out.unwrap().len(), 2);
    assert_eq!(sent.len(), 2);
    assert!(
        sent[1].contains("previous answer was rejected"),
        "{}",
        sent[1]
    );
    assert!(sent[1].contains("leaves out: F4"), "{}", sent[1]);
}

#[test]
fn two_bad_answers_give_the_second_error() {
    let (out, sent) = run(&[&fixture("malformed"), &fixture("unknown_id")]);
    assert!(matches!(out, Err(Error::Invalid(m)) if m.contains("H9")));
    assert_eq!(sent.len(), 2);
}
