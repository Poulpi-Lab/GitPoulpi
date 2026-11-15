use std::sync::{Arc, Mutex};

use gitpoulpi_config::{AiConfig, AiProfile, AiProviderKind};
use gitpoulpi_secrets::InMemoryTokens;

use super::prompt::tests::{comment, request};
use super::*;
use crate::limits::{LimitAnswer, LimitGate, Part};

/// Records every (system, user) it is asked and answers with a fixed text.
struct Spy {
    answer: String,
    seen: Mutex<Vec<(String, String)>>,
}

impl Spy {
    fn new(answer: &str) -> Self {
        Self {
            answer: answer.into(),
            seen: Mutex::new(Vec::new()),
        }
    }
}

#[async_trait::async_trait]
impl AiClient for Spy {
    async fn complete(&self, system: &str, user: &str) -> Result<String> {
        self.seen
            .lock()
            .unwrap()
            .push((system.to_string(), user.to_string()));
        Ok(self.answer.clone())
    }
}

fn config() -> AiConfig {
    let mut config = AiConfig {
        enabled: true,
        ..AiConfig::default()
    };
    config
        .profiles
        .push(AiProfile::new("p", "P", AiProviderKind::LocalAgent));
    config
}

fn job(config: &AiConfig, action: AiAction) -> Job {
    Job::prepare(config, action, None, &InMemoryTokens::default()).unwrap()
}

fn block_on<T>(f: impl std::future::Future<Output = T>) -> T {
    tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(f)
}

fn many(n: usize) -> Vec<Entry> {
    (0..n)
        .map(|i| {
            comment(
                &format!("c{i:03}"),
                "bob",
                &format!("note-{i:03} {}", "x".repeat(2_000)),
            )
        })
        .collect()
}

#[test]
fn a_draft_is_the_answer_text_and_the_request_has_the_base_prompt_and_the_conversation() {
    let spy = Spy::new("  Thanks, I will look tomorrow.  \n");
    let mut r = request(vec![comment("a", "bob", "Ping?")]);
    r.hint = "tomorrow".into();
    let draft = block_on(draft_with(
        &job(&config(), AiAction::HostingReply),
        &spy,
        &r,
    ))
    .unwrap();
    assert_eq!(draft.text, "Thanks, I will look tomorrow.");
    assert!(!draft.improved);
    assert_eq!(draft.context.entries_sent, 1);
    assert!(!draft.context.repository, "no repository access yet");
    let seen = spy.seen.lock().unwrap();
    assert_eq!(seen.len(), 1, "exactly one request, nothing else");
    assert!(seen[0].0.starts_with(BASE_PROMPT), "{}", seen[0].0);
    assert!(seen[0].1.contains("Ping?") && seen[0].1.contains("tomorrow"));
}

#[test]
fn an_improved_draft_says_so() {
    let spy = Spy::new("Better text.");
    let mut r = request(Vec::new());
    r.draft = Some("meh".into());
    let draft = block_on(draft_with(
        &job(&config(), AiAction::HostingReply),
        &spy,
        &r,
    ))
    .unwrap();
    assert!(draft.improved);
    assert!(spy.seen.lock().unwrap()[0].1.contains("meh"));
}

#[test]
fn a_request_for_the_other_action_is_refused_before_anything_is_sent() {
    let spy = Spy::new("x");
    let r = request(Vec::new());
    let result = block_on(draft_with(&job(&config(), AiAction::IssueBody), &spy, &r));
    assert!(matches!(result, Err(Error::Invalid(_))), "{result:?}");
    assert!(spy.seen.lock().unwrap().is_empty());
}

#[test]
fn an_empty_answer_is_an_error_not_an_empty_draft() {
    let spy = Spy::new("  \n ");
    let r = request(Vec::new());
    let result = block_on(draft_with(
        &job(&config(), AiAction::HostingReply),
        &spy,
        &r,
    ));
    assert_eq!(result, Err(Error::Empty));
}

#[test]
fn a_fence_around_the_whole_answer_is_taken_off_but_real_code_is_kept() {
    assert_eq!(parse_draft("```markdown\nHello\n```").unwrap(), "Hello");
    assert_eq!(parse_draft("```\nHello\n```").unwrap(), "Hello");
    // A reply that is code stays code: another language, or a fence inside.
    assert_eq!(
        parse_draft("```rust\nlet x = 1;\n```").unwrap(),
        "```rust\nlet x = 1;\n```"
    );
    assert_eq!(
        parse_draft("Try:\n```\nx\n```\nOK").unwrap(),
        "Try:\n```\nx\n```\nOK"
    );
    assert_eq!(
        parse_draft("```md\nTry:\n```\nx\n```\n```").unwrap(),
        "```md\nTry:\n```\nx\n```\n```"
    );
    assert_eq!(parse_draft(""), Err(Error::Empty));
}

/// The cap is `ai.limits.hosting_reply_context_bytes`: with nobody to ask,
/// the conversation is held to it and the cut is recorded, never silent.
#[test]
fn over_the_cap_the_conversation_is_cut_and_the_job_records_it() {
    let mut config = config();
    config.limits.hosting_reply_context_bytes = 8_000;
    let job = job(&config, AiAction::HostingReply);
    let spy = Spy::new("ok");
    let draft = block_on(draft_with(&job, &spy, &request(many(30)))).unwrap();
    assert!(draft.context.entries_omitted > 0);
    let user = spy.seen.lock().unwrap()[0].1.clone();
    assert!(user.contains("note-029") && !user.contains("note-000"));
    let cuts = job.truncated();
    assert!(!cuts.is_empty());
    assert!(cuts.iter().all(|c| c.part == Part::ReplyConversation));
    assert_eq!(cuts[0].setting, "ai.limits.hosting_reply_context_bytes");
}

#[test]
fn send_all_lifts_the_cap() {
    let mut config = config();
    config.limits.hosting_reply_context_bytes = 8_000;
    let job = job(&config, AiAction::HostingReply).with_send_all(true);
    let spy = Spy::new("ok");
    let draft = block_on(draft_with(&job, &spy, &request(many(30)))).unwrap();
    assert_eq!(draft.context.entries_omitted, 0);
    assert!(spy.seen.lock().unwrap()[0].1.contains("note-000"));
    assert!(job.truncated().is_empty());
}

struct Gate(LimitAnswer, Mutex<usize>);
#[async_trait::async_trait]
impl LimitGate for Gate {
    async fn ask(&self, _: &[Cut]) -> LimitAnswer {
        *self.1.lock().unwrap() += 1;
        self.0
    }
}

#[test]
fn a_frontend_is_asked_and_cancel_sends_nothing() {
    let mut config = config();
    config.limits.hosting_reply_context_bytes = 8_000;
    let gate = Arc::new(Gate(LimitAnswer::Cancel, Mutex::new(0)));
    let job = job(&config, AiAction::HostingReply).with_limit_gate(gate.clone());
    let spy = Spy::new("ok");
    let result = block_on(draft_with(&job, &spy, &request(many(30))));
    assert_eq!(result, Err(Error::Declined));
    assert_eq!(*gate.1.lock().unwrap(), 1);
    assert!(spy.seen.lock().unwrap().is_empty(), "nothing was sent");
}

#[test]
fn under_the_cap_nobody_is_asked() {
    let gate = Arc::new(Gate(LimitAnswer::Cancel, Mutex::new(0)));
    let job = job(&config(), AiAction::HostingReply).with_limit_gate(gate.clone());
    let spy = Spy::new("ok");
    block_on(draft_with(&job, &spy, &request(many(2)))).unwrap();
    assert_eq!(*gate.1.lock().unwrap(), 0);
}

/// Owner requirement: the user can customise the base prompt. Blank keeps
/// the built-in one; the instruction levels still follow it; the other
/// actions never take it.
#[test]
fn the_base_prompt_is_the_users_when_they_wrote_one() {
    let mut config = config();
    config.global_instructions = "Be kind.".into();
    let built_in = job(&config, AiAction::HostingReply).system_prompt();
    assert!(built_in.starts_with(BASE_PROMPT));

    config.hosting_reply.base_prompt = "  Reply like a pirate.  ".into();
    for action in [AiAction::HostingReply, AiAction::IssueBody] {
        let system = job(&config, action).system_prompt();
        assert_eq!(
            system, "Reply like a pirate.\n\n## Global instructions\nBe kind.",
            "{action:?}"
        );
    }
    assert_eq!(
        job(&config, AiAction::CommitMessage).base_prompt(),
        crate::prompts::base_prompt(AiAction::CommitMessage),
        "the other actions keep theirs"
    );

    config.hosting_reply.base_prompt = "   ".into();
    assert!(job(&config, AiAction::HostingReply)
        .system_prompt()
        .starts_with(BASE_PROMPT));
}

#[test]
fn the_edited_base_prompt_is_what_is_sent() {
    let mut config = config();
    config.hosting_reply.base_prompt = "Reply like a pirate.".into();
    let spy = Spy::new("Arr.");
    block_on(draft_with(
        &job(&config, AiAction::HostingReply),
        &spy,
        &request(Vec::new()),
    ))
    .unwrap();
    assert!(spy.seen.lock().unwrap()[0]
        .0
        .starts_with("Reply like a pirate."));
}
