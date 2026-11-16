//! The exchange given back to a model that does not remember it: the exact
//! request bodies (messages, cache markers, trimming), over the recorded
//! transport.

use std::sync::{Arc, Mutex};

use gitpoulpi_config::{AiConfig, AiProfile, AiProviderKind};
use gitpoulpi_core::domain::Oid;
use gitpoulpi_hosting::RecordedTransport;
use gitpoulpi_secrets::InMemoryTokens;
use serde_json::{json, Value};

use super::prompt::tests::{comment, request};
use super::prompt::{build, follow_up};
use super::*;
use crate::{AnthropicClient, OpenAiClient, Request};

fn block_on<T>(f: impl std::future::Future<Output = T>) -> T {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(f)
}

fn job() -> Job {
    let mut config = AiConfig {
        enabled: true,
        ..AiConfig::default()
    };
    config
        .profiles
        .push(AiProfile::new("p", "P", AiProviderKind::LocalAgent));
    Job::prepare(
        &config,
        AiAction::HostingReply,
        None,
        &InMemoryTokens::default(),
    )
    .unwrap()
}

fn ex(hint: &str, draft: Option<&str>, answer: &str) -> Exchange {
    Exchange {
        hint: hint.into(),
        draft: draft.map(str::to_string),
        answer: answer.into(),
    }
}

/// The request of the third sitting step: `earlier` already answered.
fn next(hint: &str, draft: Option<&str>, earlier: Vec<Exchange>) -> ReplyRequest {
    let mut r = request(vec![comment("a", "bob", "Can you look at it?")]);
    r.hint = hint.into();
    r.draft = draft.map(str::to_string);
    r.exchange = earlier;
    r
}

fn body_of(transport: &RecordedTransport, n: usize) -> Value {
    serde_json::from_str(transport.requests()[n].body.as_ref().unwrap()).unwrap()
}

fn openai(transport: &Arc<RecordedTransport>) -> OpenAiClient {
    OpenAiClient::new(transport.clone(), "http://x/v1", "m", None)
}

const OPENAI_OK: &str = r#"{"choices":[{"message":{"content":"Draft."}}]}"#;
const ANTHROPIC_OK: &str = r#"{"content":[{"type":"text","text":"Draft."}]}"#;

#[test]
fn an_openai_follow_up_sends_the_exchange_as_chat_messages_in_a_stable_order() {
    let job = job();
    let transport = Arc::new(RecordedTransport::new().on("/chat/completions", 200, OPENAI_OK));
    let client = openai(&transport);
    // The first request, then a follow-up and a second one.
    block_on(draft_with(&job, &client, &next("be polite", None, vec![]))).unwrap();
    let first_ex = ex("be polite", None, "Draft one.");
    block_on(draft_with(
        &job,
        &client,
        &next("shorter", None, vec![first_ex.clone()]),
    ))
    .unwrap();
    let second_ex = ex("shorter", None, "Draft two.");
    let current = "Draft two, with my edit.";
    block_on(draft_with(
        &job,
        &client,
        &next("add a thanks", Some(current), vec![first_ex, second_ex]),
    ))
    .unwrap();

    let (a, b, c) = (
        body_of(&transport, 0),
        body_of(&transport, 1),
        body_of(&transport, 2),
    );
    let first_message = build(&next("be polite", None, vec![]), usize::MAX).user;
    assert_eq!(
        a["messages"],
        json!([
            {"role": "system", "content": job.system_prompt()},
            {"role": "user", "content": first_message},
        ])
    );
    assert_eq!(
        b["messages"],
        json!([
            {"role": "system", "content": job.system_prompt()},
            {"role": "user", "content": first_message},
            {"role": "assistant", "content": "Draft one."},
            {"role": "user", "content": follow_up("shorter", None)},
        ])
    );
    assert_eq!(
        c["messages"],
        json!([
            {"role": "system", "content": job.system_prompt()},
            {"role": "user", "content": first_message},
            {"role": "assistant", "content": "Draft one."},
            {"role": "user", "content": follow_up("shorter", None)},
            {"role": "assistant", "content": "Draft two."},
            {"role": "user", "content": follow_up("add a thanks", Some(current))},
        ])
    );
    // Each request only appends to the one before: a provider caches the prefix.
    let (a, b) = (
        a["messages"].as_array().unwrap(),
        b["messages"].as_array().unwrap(),
    );
    assert_eq!(a[..], b[..a.len()]);
    // The follow-up is only what is new, not the conversation again.
    let last = c["messages"][5]["content"].as_str().unwrap();
    assert!(last.contains("add a thanks") && last.contains(current));
    assert!(!last.contains("Can you look at it?"), "{last}");
}

#[test]
fn an_anthropic_follow_up_keeps_the_stable_prefix_under_cache_control() {
    let job = job();
    let transport = Arc::new(RecordedTransport::new().on("/messages", 200, ANTHROPIC_OK));
    let client = AnthropicClient::new(transport.clone(), "http://x/v1", "m", "k".into());
    let earlier = vec![
        ex("be polite", None, "Draft one."),
        ex("shorter", None, "Two."),
    ];
    block_on(draft_with(
        &job,
        &client,
        &next("add a thanks", None, earlier),
    ))
    .unwrap();
    let body = body_of(&transport, 0);
    let mark = json!({"type": "ephemeral"});
    let first_message = build(&next("be polite", None, vec![]), usize::MAX).user;
    assert_eq!(
        body["system"],
        json!([{"type": "text", "text": job.system_prompt(), "cache_control": mark}])
    );
    assert_eq!(
        body["messages"],
        json!([
            {"role": "user", "content": [
                {"type": "text", "text": first_message, "cache_control": mark}]},
            {"role": "assistant", "content": "Draft one."},
            {"role": "user", "content": follow_up("shorter", None)},
            {"role": "assistant", "content": [
                {"type": "text", "text": "Two.", "cache_control": mark}]},
            {"role": "user", "content": follow_up("add a thanks", None)},
        ])
    );
}

#[test]
fn with_repository_context_the_markers_stay_within_the_four_the_api_allows() {
    let transport = Arc::new(RecordedTransport::new().on("/messages", 200, ANTHROPIC_OK));
    let client = AnthropicClient::new(transport.clone(), "http://x/v1", "m", "k".into());
    let context = ContextBlock {
        text: "REPO CONTEXT".into(),
        summary: ContextSummary::default(),
    };
    let history = [
        Turn {
            user: "conversation".into(),
            assistant: "one".into(),
        },
        Turn {
            user: "shorter".into(),
            assistant: "two".into(),
        },
    ];
    block_on(AiClient::complete_request(
        &client,
        &Request {
            system: "BASE",
            context: Some(&context),
            history: &history,
            user: "now",
        },
    ))
    .unwrap();
    let text = transport.requests()[0].body.clone().unwrap();
    assert_eq!(text.matches("cache_control").count(), 4, "{text}");
    let body = body_of(&transport, 0);
    // Instructions, context, the conversation message, the newest answer.
    assert_eq!(body["system"][0]["cache_control"]["type"], "ephemeral");
    assert_eq!(body["system"][1]["text"], "REPO CONTEXT");
    assert_eq!(body["system"][1]["cache_control"]["type"], "ephemeral");
    assert_eq!(
        body["messages"][0]["content"][0]["cache_control"]["type"],
        "ephemeral"
    );
    assert_eq!(
        body["messages"][3]["content"][0]["cache_control"]["type"],
        "ephemeral"
    );
    // What changes stays outside every marker.
    assert_eq!(body["messages"][4]["content"], "now");
}

#[test]
fn a_first_request_is_unchanged() {
    let transport = Arc::new(RecordedTransport::new().on("/messages", 200, ANTHROPIC_OK));
    let client = AnthropicClient::new(transport.clone(), "http://x/v1", "m", "k".into());
    block_on(draft_with(&job(), &client, &next("hi", None, vec![]))).unwrap();
    let body = body_of(&transport, 0);
    assert!(body["system"].is_string(), "no markers without a context");
    assert!(body["messages"][0]["content"].is_string());
    assert_eq!(body["messages"].as_array().unwrap().len(), 1);
}

#[test]
fn a_local_agent_that_cannot_resume_reads_the_history_as_a_transcript() {
    struct Spy(Mutex<Vec<String>>);
    #[async_trait::async_trait]
    impl AiClient for Spy {
        async fn complete(&self, _system: &str, user: &str) -> Result<String> {
            self.0.lock().unwrap().push(user.to_string());
            Ok("Draft.".into())
        }
    }
    let spy = Spy(Mutex::new(Vec::new()));
    let draft = block_on(draft_with(
        &job(),
        &spy,
        &next(
            "shorter",
            Some("my text"),
            vec![ex("be polite", None, "Draft one.")],
        ),
    ))
    .unwrap();
    let sent = spy.0.lock().unwrap()[0].clone();
    let first_message = build(&next("be polite", None, vec![]), usize::MAX).user;
    assert!(sent.starts_with(&first_message), "{sent}");
    assert!(sent.contains("Your draft so far:\nDraft one."), "{sent}");
    assert!(sent.contains("The user now asks:\nWhat the user wants now:\nshorter"));
    assert!(sent.contains("my text"));
    assert_eq!(draft.context.history_turns, 1);
}

#[test]
fn old_follow_ups_are_trimmed_first_and_the_newest_draft_stays() {
    let mut job = job();
    job.limit.limits.reply_context = 2_000;
    let long = |c: char| c.to_string().repeat(700);
    let earlier = vec![
        ex("h0", None, &long('a')),
        ex("h1", None, &long('b')),
        ex("h2", None, &long('c')),
        ex("h3", None, &long('d')),
    ];
    let built = block_on(prepare(&job, &next("now", None, earlier))).unwrap();
    let answers: Vec<&str> = built.history.iter().map(|t| t.assistant.as_str()).collect();
    // The first turn (it holds the conversation) and the newest draft stay;
    // the middle ones went, oldest first, until the rest fits.
    assert_eq!(answers.first(), Some(&long('a').as_str()));
    assert_eq!(answers.last(), Some(&long('d').as_str()));
    assert_eq!(built.history.len(), 2, "{answers:?}");
    assert_eq!(built.history_omitted, 2);
    assert!(built.history[0].user.contains("Can you look at it?"));
    assert_eq!(built.user, follow_up("now", None));
}

#[test]
fn the_newest_draft_stays_even_when_nothing_fits() {
    let mut job = job();
    job.limit.limits.reply_context = 10;
    let built = block_on(prepare(
        &job,
        &next("now", None, vec![ex("h0", None, "A"), ex("h1", None, "B")]),
    ))
    .unwrap();
    assert_eq!(built.history.last().unwrap().assistant, "B");
}

fn agent_request(session: AgentSession) -> ReplyRequest {
    let mut r = next("shorter", None, vec![ex("be polite", None, "Draft one.")]);
    r.repo = RepoAccess::AgentReadOnly {
        root: "/repo".into(),
        head: Oid([1; 20]),
        tools: "Read, Grep, Glob".into(),
        session: Some(session),
    };
    r
}

#[test]
fn a_resumed_agent_session_is_not_told_again_but_a_new_one_is() {
    let job = job();
    // Resumed: it remembers, the request is the whole message as before.
    let resumed = agent_request(AgentSession::Continued);
    let built = block_on(prepare(&job, &resumed)).unwrap();
    assert!(built.history.is_empty());
    assert_eq!(built.user, build(&resumed, usize::MAX).user);
    // Started (a new HEAD, a refresh, the setting off): it knows nothing.
    let started = agent_request(AgentSession::Started);
    let built = block_on(prepare(&job, &started)).unwrap();
    assert_eq!(built.history.len(), 1);
    assert_eq!(built.history[0].assistant, "Draft one.");
}

#[test]
fn the_repository_context_selection_stays_the_one_of_the_first_request() {
    use super::plan::plan_context;
    let tree = vec!["src/a.rs".to_string(), "src/b.rs".to_string()];
    let limits = repo_context::ContextLimits::from_config(&gitpoulpi_config::AiLimits::default());
    let mut first = next("look at src/a.rs", None, vec![]);
    first.entries.clear();
    let mut follow = next(
        "shorter, and mention src/b.rs",
        None,
        vec![ex("look at src/a.rs", None, "x")],
    );
    follow.entries.clear();
    assert_eq!(
        plan_context(&first, &tree, &limits).hash(),
        plan_context(&follow, &tree, &limits).hash(),
        "a new hint does not change the context, so its cache and the provider's hold"
    );
}

#[test]
fn earlier_turns_are_cleaned_like_the_rest_of_the_request() {
    use crate::privacy::{prepare_request, Privacy};
    const TOKEN: &str = "ghp_q7Xk2LmZp9RtV4bN8wYcJ3hGfD5sAe6U1oIx";
    let history = [Turn {
        user: "conversation".into(),
        assistant: format!("Use {TOKEN} to log in."),
    }];
    let prepared = prepare_request(
        &Request {
            system: "S",
            context: None,
            history: &history,
            user: "shorter",
        },
        &Privacy::default(),
    );
    assert!(!prepared.history[0].assistant.contains(TOKEN));
    assert!(prepared.history[0].assistant.contains("[REDACTED"));
    assert!(!prepared.redactions.is_empty());
    // The dialog and print-only show the whole of what goes.
    let shown = prepared.user_text();
    assert!(shown.contains("conversation") && shown.contains("shorter"));
    assert!(!shown.contains(TOKEN));
}
