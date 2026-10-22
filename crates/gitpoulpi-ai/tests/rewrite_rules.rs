//! The "Create Rules with AI" action: the prompt's vocabulary is the
//! workbench's own, an answer becomes rules and settings, bad rules are
//! reported while the good ones stay, and nothing but data comes out.

use std::sync::Mutex;

use gitpoulpi_ai::assist::{Job, RewriteRulesRequest};
use gitpoulpi_ai::prompts::{self, RulesContext, REWRITE_RULES_VOCABULARY};
use gitpoulpi_ai::{parse, AiClient, Error, Result};
use gitpoulpi_config::{AiAction, AiConfig};
use gitpoulpi_core::domain::rewrite::{
    validate_recipe, CommitterPolicy, Recipe, Rule, RuleKind, Scope,
};
use gitpoulpi_secrets::TokenStore;

/// Every `RuleKind::name()` but `script`, the one the AI may not create.
const KINDS: &[&str] = &[
    "identity",
    "mailmap",
    "anonymise",
    "date_shift",
    "date_timezone",
    "date_window",
    "committer_from_author",
    "message_replace",
    "message_delete_lines",
    "trailer",
    "content_replace",
    "remove_paths",
    "remove_blobs_over",
    "move_paths",
    "lfs_import",
    "lfs_export",
    "line_endings",
    "strip_exec",
    "reencode",
    "remove_signatures",
    "message_encoding",
];

#[test]
fn every_vocabulary_example_is_a_valid_workbench_rule() {
    for entry in REWRITE_RULES_VOCABULARY {
        let kind: RuleKind = serde_json::from_str(entry.example)
            .unwrap_or_else(|e| panic!("{}: {e}\n{}", entry.kind, entry.example));
        assert_eq!(kind.name(), entry.kind, "{}", entry.example);
        let recipe = Recipe::new("", Scope::AllLocal, vec![Rule::new(kind)]);
        validate_recipe(&recipe).unwrap_or_else(|e| panic!("{}: {e}", entry.kind));
    }
}

#[test]
fn the_vocabulary_covers_every_rule_kind_but_script() {
    let mut have: Vec<&str> = REWRITE_RULES_VOCABULARY.iter().map(|e| e.kind).collect();
    have.sort_unstable();
    let mut want = KINDS.to_vec();
    want.sort_unstable();
    assert_eq!(have, want);
    assert!(!have.contains(&"script"));
}

#[test]
fn the_system_prompt_carries_the_vocabulary_and_the_answer_shape() {
    let base = prompts::base_prompt(AiAction::RewriteRules);
    for kind in KINDS {
        assert!(base.contains(&format!("- {kind}:")), "{kind}");
    }
    for needle in [
        "\"scope\"",
        "all_local",
        "\"rules\"",
        "\"notes\"",
        "PREPARE",
    ] {
        assert!(base.contains(needle), "{needle}");
    }
}

const SAMPLE: &str = r#"Sure.
```json
{
  "scope": {"scope": "all_local"},
  "scope_options": {"include_tags": false},
  "options": {"committer": "me_now", "prune_empty": "never"},
  "rules": [
    {"label": "Old email", "kind": {"type": "identity",
      "when": {"email": "a@b.org"}, "set": {"email": "c@d.org"}}},
    {"kind": {"type": "remove_paths", "globs": ["secrets.txt"]}},
    {"kind": {"type": "date_shift", "by_seconds": 7200, "apply_to": "author_date"}}
  ],
  "notes": ["I cannot limit it to commits since March."]
}
```"#;

#[test]
fn a_sample_answer_becomes_rules_scope_and_settings() {
    let a = parse::rewrite_rules(SAMPLE).unwrap();
    assert_eq!(a.scope, Some(Scope::AllLocal));
    assert_eq!(a.settings.include_tags, Some(false));
    assert_eq!(a.settings.committer, Some(CommitterPolicy::MeNow));
    assert_eq!(a.rules.len(), 3);
    assert_eq!(a.rules[0].label.as_deref(), Some("Old email"));
    let names: Vec<&str> = a.rules.iter().map(|r| r.kind.name()).collect();
    assert_eq!(names, ["identity", "remove_paths", "date_shift"]);
    assert!(a.problems.is_empty(), "{:?}", a.problems);
    assert_eq!(a.notes, ["I cannot limit it to commits since March."]);
}

#[test]
fn invalid_rules_are_reported_and_the_valid_ones_kept() {
    let answer = r#"{"rules": [
      {"kind": {"type": "remove_paths", "globs": ["a.txt"]}},
      {"kind": {"type": "teleport"}},
      {"label": "Zero", "kind": {"type": "date_shift", "by_seconds": 0}},
      {"kind": {"type": "script", "language": "lua", "source": {"from": "inline", "text": "x"}}},
      {"kind": {"type": "content_replace", "find": "secrets", "literals": ["hunter2"], "scan": false}},
      {"kind": {"type": "remove_blobs_over", "bytes": 1024}}
    ]}"#;
    let a = parse::rewrite_rules(answer).unwrap();
    let kept: Vec<&str> = a.rules.iter().map(|r| r.kind.name()).collect();
    assert_eq!(kept, ["remove_paths", "remove_blobs_over"]);
    let at: Vec<usize> = a.problems.iter().map(|p| p.position).collect();
    assert_eq!(at, [2, 3, 4, 5]);
    assert!(a.problems[1].reason.contains("zero"), "{:?}", a.problems);
    assert_eq!(a.problems[1].what, "Zero");
    assert!(a.problems[2].reason.contains("runs code"));
    assert!(a.problems[3].reason.contains("never handed"));
}

#[test]
fn scopes_the_workbench_cannot_hold_are_refused_not_guessed() {
    let a = parse::rewrite_rules(
        r#"{"scope": {"scope": "range", "base": "0000000000000000000000000000000000000001", "tip": "0000000000000000000000000000000000000002"}, "rules": []}"#,
    )
    .unwrap();
    assert_eq!(a.scope, None);
    assert_eq!(a.problems.len(), 1);
    assert_eq!(a.problems[0].position, 0);
    let a = parse::rewrite_rules(r#"{"scope": {"scope": "refs", "refs": ["main"]}}"#).unwrap();
    assert_eq!(
        a.scope,
        Some(Scope::Refs {
            refs: vec!["main".into()]
        })
    );
}

#[test]
fn signatures_are_never_taken_from_the_model() {
    let a = parse::rewrite_rules(r#"{"options": {"signatures": "strip"}, "rules": []}"#).unwrap();
    assert!(a.settings.is_empty());
    assert!(a.problems[0].reason.contains("asked when you Apply"));
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

/// Answers in order and records every system and user message.
struct Script(Mutex<Vec<String>>, Mutex<Vec<(String, String)>>);
#[async_trait::async_trait]
impl AiClient for Script {
    async fn complete(&self, system: &str, user: &str) -> Result<String> {
        self.1.lock().unwrap().push((system.into(), user.into()));
        Ok(self.0.lock().unwrap().remove(0))
    }
}

fn run(
    answers: &[&str],
    request: &RewriteRulesRequest,
) -> (
    Result<gitpoulpi_ai::assist::RewriteRulesOutcome>,
    Vec<(String, String)>,
) {
    let config = AiConfig {
        enabled: true,
        ..AiConfig::default()
    };
    let job = Job::prepare(&config, AiAction::RewriteRules, None, &Fixed).unwrap();
    let client = Script(
        Mutex::new(answers.iter().map(|s| s.to_string()).collect()),
        Mutex::default(),
    );
    let out = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(job.create_rewrite_rules_with(&client, request));
    let sent = client.1.lock().unwrap().clone();
    (out, sent)
}

fn request(text: &str) -> RewriteRulesRequest {
    RewriteRulesRequest {
        description: text.into(),
        context: RulesContext::default(),
    }
}

#[test]
fn the_job_asks_once_and_sends_the_vocabulary_and_only_the_description() {
    let (out, sent) = run(&[SAMPLE], &request("replace a@b.org by c@d.org"));
    let out = out.unwrap();
    assert!(!out.retried);
    assert_eq!(out.answer.rules.len(), 3);
    assert_eq!(sent.len(), 1);
    assert!(sent[0].0.contains("- remove_paths:"), "{}", sent[0].0);
    assert_eq!(
        sent[0].1,
        "What the history rewrite must do:\nreplace a@b.org by c@d.org\n"
    );
}

#[test]
fn the_context_goes_along_only_when_given() {
    let mut r = request("x");
    r.context = RulesContext {
        scope: "the current branch".into(),
        current_branch: Some("main".into()),
        refs: vec!["main".into()],
        authors: vec![("Ada".into(), "ada@example.org".into())],
    };
    let (_, sent) = run(&[SAMPLE], &r);
    assert!(sent[0].1.contains("Ada <ada@example.org>"));
    let (_, sent) = run(&[SAMPLE], &request("x"));
    assert!(!sent[0].1.contains("ada@example.org"));
}

#[test]
fn an_unreadable_answer_is_asked_again_once_then_fails() {
    let (out, sent) = run(&["no idea", SAMPLE], &request("x"));
    assert!(out.unwrap().retried);
    assert_eq!(sent.len(), 2);
    assert!(sent[1].1.contains("Your previous answer was rejected"));
    let (out, sent) = run(&["no idea", "still none"], &request("x"));
    assert!(matches!(out, Err(Error::Invalid(_))));
    assert_eq!(sent.len(), 2);
}

#[test]
fn an_empty_description_is_refused_before_any_call() {
    let (out, sent) = run(&[], &request("   "));
    assert!(matches!(out, Err(Error::Invalid(_))));
    assert!(sent.is_empty());
}
