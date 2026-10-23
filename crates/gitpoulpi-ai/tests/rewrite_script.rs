//! The history-rewrite script action: parser fixtures, the retry with a
//! compile error, and what is (not) sent.

use std::sync::Mutex;

use gitpoulpi_ai::assist::{Job, RewriteScriptRequest, ScriptChecker};
use gitpoulpi_ai::{parse, AiClient, Error, Result};
use gitpoulpi_config::{AiAction, AiConfig};
use gitpoulpi_core::domain::{CommitNode, Oid};
use gitpoulpi_secrets::TokenStore;

fn fixture(name: &str) -> String {
    let path = format!(
        "{}/tests/fixtures/rewrite_script/{name}.txt",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

const PLAIN: &str = "--! name: Night commits to the morning\n--! kinds: commit\n--! window: 0\n\
-- Moves commits made between 22:00 and 06:00 one hour later.\n\
function rewrite_commit(c, ctx)\n  local h = hour(c.author.time, c.author.offset_minutes)\n  \
if h >= 22 or h < 6 then\n    return { author = { time = c.author.time + 3600 } }\n  end\nend\n";

#[test]
fn good_answers_parse_in_every_shape() {
    for name in ["ok_plain", "ok_fenced", "prose_around"] {
        let script =
            parse::rewrite_script(&fixture(name)).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!(script, PLAIN, "{name}");
    }
}

#[test]
fn a_script_without_header_gets_the_default_one() {
    let script = parse::rewrite_script(&fixture("no_header")).unwrap();
    assert!(
        script.starts_with("--! name: AI script\n--! kinds: commit\nfunction rewrite_commit"),
        "{script}"
    );
    assert!(gitpoulpi_core::domain::rewrite::script::parse_header(&script).is_ok());
}

#[test]
fn an_answer_with_no_script_is_rejected() {
    assert!(matches!(
        parse::rewrite_script("I cannot do that."),
        Err(Error::Invalid(_))
    ));
    assert_eq!(parse::rewrite_script("  "), Err(Error::Empty));
    // A code block that is not the script is not taken for it.
    assert!(parse::rewrite_script("```sh\ngit rebase\n```").is_err());
}

#[test]
fn an_unfenced_script_with_a_lead_in_line_is_found() {
    let script = parse::rewrite_script(&format!("Sure, here it is:\n{PLAIN}")).unwrap();
    assert_eq!(script, PLAIN);
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

/// Answers in order and records every user message.
struct Script(Mutex<Vec<String>>, Mutex<Vec<String>>);
#[async_trait::async_trait]
impl AiClient for Script {
    async fn complete(&self, _system: &str, user: &str) -> Result<String> {
        self.1.lock().unwrap().push(user.to_string());
        Ok(self.0.lock().unwrap().remove(0))
    }
}

/// "Compiles" a script unless it still has the unbalanced brace the
/// fixture plants (a stand-in for the daemon's `rewrite_script_check`).
struct Checker(Mutex<Vec<String>>);
#[async_trait::async_trait]
impl ScriptChecker for Checker {
    async fn check(&self, script: &str) -> std::result::Result<Vec<String>, String> {
        self.0.lock().unwrap().push(script.to_string());
        let open = script.matches('{').count();
        let close = script.matches('}').count();
        Ok(if open == close {
            Vec::new()
        } else {
            vec!["syntax error: script:6: '}' expected".to_string()]
        })
    }
}

fn job() -> Job {
    let config = AiConfig {
        enabled: true,
        ..AiConfig::default()
    };
    Job::prepare(&config, AiAction::RewriteScript, None, &Fixed).unwrap()
}

fn node(name: &str, email: &str, subject: &str) -> CommitNode {
    CommitNode {
        id: Oid([1; 20]),
        parents: vec![],
        summary: subject.into(),
        message: format!("{subject}\n\nthe body stays home"),
        author_name: name.into(),
        author_email: email.into(),
        timestamp: 1_700_000_000,
        refs: vec![],
    }
}

fn run(
    answers: &[String],
    request: &RewriteScriptRequest,
) -> (
    Result<gitpoulpi_ai::assist::RewriteScriptOutcome>,
    Vec<String>,
    Vec<String>,
) {
    let client = Script(Mutex::new(answers.to_vec()), Mutex::default());
    let checker = Checker(Mutex::default());
    let out = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(job().write_rewrite_script_with(&client, request, &checker));
    let sent = client.1.lock().unwrap().clone();
    let checked = checker.0.lock().unwrap().clone();
    (out, sent, checked)
}

fn ask(text: &str) -> RewriteScriptRequest {
    RewriteScriptRequest::new(text, &[], false, 0)
}

#[test]
fn a_good_first_answer_needs_one_call_and_one_check() {
    let (out, sent, checked) = run(&[fixture("ok_fenced")], &ask("night to morning"));
    let out = out.unwrap();
    assert_eq!(out.script, PLAIN);
    assert!(out.problems.is_empty() && out.checked && !out.retried);
    assert_eq!(sent.len(), 1);
    assert_eq!(checked.len(), 1);
}

#[test]
fn a_compile_error_is_sent_back_once_and_the_fix_is_kept() {
    let both = fixture("compile_error_then_fixed");
    let (first, second) = both.split_once("=====\n").unwrap();
    let (out, sent, checked) = run(&[first.to_string(), second.to_string()], &ask("old email"));
    let out = out.unwrap();
    assert!(out.retried);
    assert!(out.problems.is_empty(), "{:?}", out.problems);
    assert!(
        out.script.contains("email = \"new@example.org\" } }"),
        "{}",
        out.script
    );
    assert_eq!(sent.len(), 2, "exactly one retry");
    assert!(
        sent[1].contains("previous answer was rejected"),
        "{}",
        sent[1]
    );
    assert!(
        sent[1].contains("'}' expected"),
        "the error goes back\n{}",
        sent[1]
    );
    assert!(
        sent[1].contains("old@example.org"),
        "and the script\n{}",
        sent[1]
    );
    assert_eq!(checked.len(), 2);
}

#[test]
fn a_second_failure_is_returned_with_its_problems_not_retried_again() {
    let both = fixture("compile_error_then_fixed");
    let (first, _) = both.split_once("=====\n").unwrap();
    let (out, sent, _) = run(&[first.to_string(), first.to_string()], &ask("old email"));
    let out = out.unwrap();
    assert_eq!(sent.len(), 2);
    assert!(out.retried);
    assert_eq!(
        out.problems.len(),
        1,
        "the editor shows what is still wrong"
    );
}

#[test]
fn an_answer_without_a_script_is_retried_once() {
    let (out, sent, _) = run(
        &["Sorry, I cannot.".to_string(), fixture("ok_plain")],
        &ask("x"),
    );
    assert_eq!(out.unwrap().script, PLAIN);
    assert_eq!(sent.len(), 2);
    assert!(sent[1].contains("defines rewrite_commit"), "{}", sent[1]);
}

#[test]
fn an_empty_instruction_asks_nothing() {
    let (out, sent, _) = run(&[], &ask("   "));
    assert!(matches!(out, Err(Error::Invalid(_))));
    assert!(sent.is_empty());
}

#[test]
fn with_the_sample_off_no_commit_data_is_sent() {
    let commits = [
        node("Ada Lovelace", "ada@secret.example", "fix: the parser"),
        node("Bob", "bob@secret.example", "feat: login"),
    ];
    let request = RewriteScriptRequest::new("night to morning", &commits, false, 20);
    let (out, sent, _) = run(&[fixture("ok_plain")], &request);
    out.unwrap();
    let all = sent.join("\n");
    for needle in [
        "Ada",
        "secret.example",
        "fix: the parser",
        "login",
        "sample",
        "body stays",
    ] {
        assert!(!all.contains(needle), "{needle:?} was sent\n{all}");
    }
    assert!(all.contains("night to morning"));
    // The dialog shows the same text the model gets.
    let (_, shown) = job().rewrite_script_messages(&request);
    assert_eq!(shown, sent[0]);
}

#[test]
fn with_the_sample_on_only_headers_of_the_first_n_commits_are_sent() {
    let commits = [
        node("Ada Lovelace", "ada@secret.example", "fix: the parser"),
        node("Bob", "bob@secret.example", "feat: login"),
    ];
    let request = RewriteScriptRequest::new("night to morning", &commits, true, 1);
    let (_, sent, _) = run(&[fixture("ok_plain")], &request);
    let all = sent.join("\n");
    assert!(all.contains("Ada Lovelace <ada@secret.example>"), "{all}");
    assert!(all.contains("2023-11-14 22:13:20 UTC"), "{all}");
    assert!(all.contains("fix: the parser"), "{all}");
    assert!(
        !all.contains("Bob") && !all.contains("login"),
        "only N commits\n{all}"
    );
    assert!(!all.contains("body stays"), "never the message body\n{all}");
}

#[test]
fn the_system_prompt_carries_the_contract() {
    let system = job().system_prompt();
    for needle in [
        "rewrite_commit(c, ctx)",
        "--! kinds: commit",
        "regex_replace",
        "no io",
        "10 million",
    ] {
        assert!(system.contains(needle), "{needle}");
    }
}
