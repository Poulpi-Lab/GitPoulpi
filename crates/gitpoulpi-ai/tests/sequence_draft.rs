//! The "Sequence draft" action: the prompt's vocabulary is generated from the
//! config crate's own step vocabulary, an answer becomes a definition, bad
//! steps are reported while the good ones stay, and nothing but data comes
//! out.

use std::sync::Mutex;

use gitpoulpi_ai::assist::{Job, SequenceDraftRequest};
use gitpoulpi_ai::prompts::{self, SequenceContext};
use gitpoulpi_ai::{parse, AiClient, Error, Result};
use gitpoulpi_config::sequences::{
    step_kind, validate, PauseMode, Scope, SequenceDef, StepSpec, EXCLUDED_REASON, MAX_STEPS,
    STEP_VOCABULARY,
};
use gitpoulpi_config::{AiAction, AiConfig};
use gitpoulpi_secrets::TokenStore;

#[test]
fn the_prompt_vocabulary_covers_every_step_and_lists_no_excluded_one() {
    let base = prompts::base_prompt(AiAction::SequenceDraft);
    for kind in STEP_VOCABULARY {
        assert!(base.contains(&format!("- {}:", kind.id)), "{}", kind.id);
        assert!(base.contains(kind.meaning_en), "{}", kind.id);
        let example = prompts::sequence_draft_example_step(kind).to_string();
        assert!(base.contains(&example), "{example}");
    }
    for (id, _) in EXCLUDED_REASON {
        assert!(!base.contains(id), "{id} must not be in the prompt");
    }
    // The pause rules come from the vocabulary, not from the prose.
    assert!(base.contains("never use pause \"never\""));
    assert!(base.contains("required when pause is \"never\""));
    for needle in ["\"steps\"", "\"notes\"", "\"shortcut\"", "PROPOSE"] {
        assert!(base.contains(needle), "{needle}");
    }
    assert!(base.contains(&format!("at most {MAX_STEPS}")));
}

#[test]
fn every_example_step_parses_and_validates_alone() {
    for kind in STEP_VOCABULARY {
        let example = prompts::sequence_draft_example_step(kind);
        let answer = serde_json::json!({ "name": "X", "id": "x", "steps": [example] });
        let p = parse::sequence_draft(&answer.to_string(), &[]).unwrap();
        assert!(p.problems.is_empty(), "{}: {:?}", kind.id, p.problems);
        assert_eq!(p.def.steps.len(), 1, "{}", kind.id);
        assert!(validate(&p.def).is_empty(), "{}", kind.id);
    }
}

const SAMPLE: &str = r#"Here is a draft:
```json
{
  "name": "Backup",
  "id": "Backup Now!",
  "description": "Stage, message, commit, push",
  "scope": "machine",
  "steps": [
    {"run": "stage_all"},
    {"run": "ai_generate_commit_message", "pause": "always"},
    {"run": "commit"},
    {"run": "push"}
  ],
  "shortcut": "Ctrl+G b",
  "notes": ["Review the message before it commits."]
}
```
Enjoy."#;

#[test]
fn a_fenced_answer_with_prose_becomes_a_definition() {
    let p = parse::sequence_draft(SAMPLE, &[]).unwrap();
    assert!(p.problems.is_empty(), "{:?}", p.problems);
    assert_eq!(p.def.name, "Backup");
    assert_eq!(p.def.id, "backup-now");
    assert_eq!(p.def.scope, Scope::Machine);
    assert_eq!(p.def.owner, None);
    let runs: Vec<&str> = p.def.steps.iter().map(|s| s.run.as_str()).collect();
    assert_eq!(
        runs,
        ["stage_all", "ai_generate_commit_message", "commit", "push"]
    );
    assert_eq!(p.def.steps[1].pause, PauseMode::Always);
    assert_eq!(p.suggested_shortcut.as_deref(), Some("Ctrl+G b"));
    assert_eq!(p.notes, ["Review the message before it commits."]);
    assert!(validate(&p.def).is_empty());
}

#[test]
fn an_unfenced_object_inside_prose_parses_and_the_id_is_made_unique() {
    let p = parse::sequence_draft(
        r#"I propose {"name": "Sync", "id": "sync", "steps": [{"run": "fetch"}]} as asked."#,
        &["sync".into(), "sync-2".into()],
    )
    .unwrap();
    assert_eq!(p.def.id, "sync-3");
    assert_eq!(p.def.scope, Scope::Installation);
}

#[test]
fn a_missing_id_comes_from_the_name() {
    let p = parse::sequence_draft(
        r#"{"name": "Fetch and pull", "steps": [{"run": "fetch"}]}"#,
        &[],
    )
    .unwrap();
    assert_eq!(p.def.id, "fetch-and-pull");
}

#[test]
fn a_destructive_step_is_refused_with_its_reason_and_the_rest_kept() {
    let p = parse::sequence_draft(
        r#"{"name":"X","id":"x","steps":[
            {"run":"stage_all"},
            {"run":"discard_all_changes"},
            {"run":"push"}]}"#,
        &[],
    )
    .unwrap();
    let runs: Vec<&str> = p.def.steps.iter().map(|s| s.run.as_str()).collect();
    assert_eq!(runs, ["stage_all", "push"]);
    assert_eq!(p.problems.len(), 1);
    assert_eq!(p.problems[0].position, 2);
    assert_eq!(p.problems[0].what, "discard_all_changes");
    assert!(
        p.problems[0].reason.contains("throws work away"),
        "{:?}",
        p.problems
    );
}

#[test]
fn unknown_nested_and_forced_never_steps_are_refused() {
    let p = parse::sequence_draft(
        r#"{"name":"X","id":"x","steps":[
            {"run":"teleport"},
            {"run":"sequence_run"},
            {"run":"force_push_with_lease","pause":"never"},
            {"run":"force_push_with_lease","pause":"always"},
            {"run":"new_branch","pause":"never"},
            {"run":"push","pause":"sometimes"},
            {"run":"push","options":{"nope":1}},
            {"nothing":true}]}"#,
        &[],
    )
    .unwrap();
    let kept: Vec<(&str, PauseMode)> = p
        .def
        .steps
        .iter()
        .map(|s| (s.run.as_str(), s.pause))
        .collect();
    assert_eq!(kept, [("force_push_with_lease", PauseMode::Always)]);
    let at: Vec<usize> = p.problems.iter().map(|x| x.position).collect();
    assert_eq!(at, [1, 2, 3, 5, 6, 7, 8]);
    assert!(p.problems[0].reason.contains("not a command"));
    assert!(p.problems[1].reason.contains("another sequence"));
    assert!(p.problems[2].reason.contains("always asks"));
    assert!(p.problems[3].reason.contains("needs the option"));
    assert!(p.problems[4].reason.contains("pause"));
    assert!(p.problems[5].reason.contains("no option"));
}

#[test]
fn more_than_the_maximum_steps_keep_the_first_ones() {
    let steps: Vec<String> = (0..MAX_STEPS + 3)
        .map(|_| r#"{"run":"fetch"}"#.to_string())
        .collect();
    let answer = format!(
        r#"{{"name":"Many","id":"many","steps":[{}]}}"#,
        steps.join(",")
    );
    let p = parse::sequence_draft(&answer, &[]).unwrap();
    assert_eq!(p.def.steps.len(), MAX_STEPS);
    assert_eq!(p.problems.len(), 3);
    assert_eq!(p.problems[0].position, MAX_STEPS + 1);
    assert!(p.problems[0].reason.contains("at most"));
}

#[test]
fn an_answer_without_steps_or_json_is_an_error() {
    assert!(matches!(
        parse::sequence_draft("I cannot.", &[]),
        Err(Error::Invalid(_))
    ));
    assert_eq!(parse::sequence_draft("  ", &[]), Err(Error::Empty));
    assert!(parse::sequence_draft(r#"{"name":"x"}"#, &[]).is_err());
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
    request: &SequenceDraftRequest,
) -> (
    Result<gitpoulpi_ai::assist::SequenceDraftOutcome>,
    Vec<(String, String)>,
) {
    let config = AiConfig {
        enabled: true,
        ..AiConfig::default()
    };
    let job = Job::prepare(&config, AiAction::SequenceDraft, None, &Fixed).unwrap();
    let client = Script(
        Mutex::new(answers.iter().map(|s| s.to_string()).collect()),
        Mutex::default(),
    );
    let out = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(job.draft_sequence_with(&client, request));
    let sent = client.1.lock().unwrap().clone();
    (out, sent)
}

fn request(text: &str) -> SequenceDraftRequest {
    SequenceDraftRequest {
        description: text.into(),
        ..SequenceDraftRequest::default()
    }
}

#[test]
fn the_job_asks_once_and_sends_the_vocabulary_and_only_the_description() {
    let (out, sent) = run(&[SAMPLE], &request("back up everything"));
    let out = out.unwrap();
    assert!(!out.retried);
    assert_eq!(out.proposal.def.steps.len(), 4);
    assert_eq!(sent.len(), 1);
    assert!(sent[0].0.contains("- stage_all:"), "{}", sent[0].0);
    assert_eq!(sent[0].1, "The sequence to create:\nback up everything\n");
}

#[test]
fn nothing_valid_asks_again_with_the_problems_then_keeps_the_second_answer() {
    let bad = r#"{"name":"X","id":"x","steps":[{"run":"discard_all_changes"}]}"#;
    let (out, sent) = run(&[bad, SAMPLE], &request("x"));
    let out = out.unwrap();
    assert!(out.retried);
    assert_eq!(out.proposal.def.steps.len(), 4);
    assert_eq!(sent.len(), 2);
    assert!(sent[1].1.contains("Your previous answer was rejected"));
    assert!(sent[1].1.contains("discard_all_changes"), "{}", sent[1].1);
    // An unreadable first answer is asked again too; two bad ones fail.
    let (out, sent) = run(&["no idea", SAMPLE], &request("x"));
    assert!(out.unwrap().retried);
    assert_eq!(sent.len(), 2);
    let (out, sent) = run(&["no idea", "still none"], &request("x"));
    assert!(matches!(out, Err(Error::Invalid(_))));
    assert_eq!(sent.len(), 2);
    // Still nothing valid after the retry: the outcome says so, one retry only.
    let (out, sent) = run(&[bad, bad], &request("x"));
    let out = out.unwrap();
    assert!(out.retried && out.proposal.def.steps.is_empty());
    assert_eq!(out.proposal.problems.len(), 1);
    assert_eq!(sent.len(), 2);
}

fn current() -> SequenceDef {
    SequenceDef {
        id: "mine".into(),
        name: "Mine".into(),
        description: Some("old".into()),
        scope: Scope::Machine,
        owner: None,
        steps: vec![
            StepSpec::new("stage_all"),
            StepSpec::new("push").with_pause(PauseMode::Always),
        ],
        ..SequenceDef::default()
    }
}

#[test]
fn edit_mode_sends_the_current_sequence_and_keeps_its_identity() {
    let mut r = request("add a fetch first");
    r.edit = Some(current());
    r.taken_ids = vec!["mine".into(), "other".into()];
    let answer = r#"{"name":"Renamed","id":"something-else","steps":[
        {"run":"fetch"},{"run":"stage_all"},{"run":"push","pause":"always"}]}"#;
    let (out, sent) = run(&[answer], &r);
    let user = &sent[0].1;
    assert!(user.contains("The sequence to change"), "{user}");
    assert!(user.contains("\"id\": \"mine\""), "{user}");
    assert!(user.contains("\"run\": \"stage_all\""), "{user}");
    assert!(user.contains("\"pause\": \"always\""), "{user}");
    assert!(user.contains("add a fetch first"));
    assert!(user.contains("keep what is not mentioned"));
    let def = out.unwrap().proposal.def;
    assert_eq!((def.id.as_str(), def.scope), ("mine", Scope::Machine));
    assert_eq!(def.description.as_deref(), Some("old"));
    assert_eq!(def.steps.len(), 3);
    // Creating sends no current sequence.
    let (_, sent) = run(&[SAMPLE], &request("x"));
    assert!(!sent[0].1.contains("The sequence to change"));
}

#[test]
fn the_context_goes_along_only_when_given() {
    let mut r = request("push to every remote");
    r.context = SequenceContext {
        remotes: vec!["origin".into(), "backup-nas".into()],
        ai_profiles: vec!["claude".into()],
        open_in_labels: vec![],
    };
    let (_, sent) = run(&[SAMPLE], &r);
    assert!(sent[0].1.contains("  - backup-nas\n") && sent[0].1.contains("  - claude\n"));
    let (_, sent) = run(&[SAMPLE], &request("push to every remote"));
    assert!(!sent[0].1.contains("origin") && !sent[0].1.contains("backup-nas"));
    assert!(!sent[0].1.contains("installation (sent"));
}

#[test]
fn an_empty_description_is_refused_before_any_call() {
    let (out, sent) = run(&[], &request("   "));
    assert!(matches!(out, Err(Error::Invalid(_))));
    assert!(sent.is_empty());
}

#[test]
fn the_action_has_a_key_a_label_and_settings() {
    assert_eq!(AiAction::SequenceDraft.key(), "sequence_draft");
    assert_eq!(AiAction::SequenceDraft.label(), "Sequence draft");
    assert_eq!(
        AiAction::from_key("sequence_draft"),
        Some(AiAction::SequenceDraft)
    );
    assert!(step_kind("stage_all").is_some());
}
