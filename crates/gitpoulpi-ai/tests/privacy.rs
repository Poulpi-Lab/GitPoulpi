//! What an AI request is cleaned of before it leaves.

use std::sync::{Arc, Mutex};

use gitpoulpi_ai::privacy::prepare;
use gitpoulpi_ai::{
    client_for, AiClient, Error, Prepared, Privacy, PrivacyClient, Result, SendGate,
};
use gitpoulpi_config::{AiProfile, AiProviderKind, ConfigPaths};
use gitpoulpi_hosting::{RecordedTransport, TrustGate};

fn run<F: std::future::Future>(f: F) -> F::Output {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(f)
}

const TOKEN_BODY: &str = "q7Xk2LmZp9RtV4bN8wYcJ3hGfD5sAe6U1oIx";

fn token() -> String {
    format!("ghp_{TOKEN_BODY}")
}

/// A prompt shaped like `render_diff`'s: a source file, a `.env`, a note.
fn prompt() -> String {
    format!(
        "Write the commit message for this change.\n\n```diff\n\
         diff -- src/lib.rs (modified)\n@@ -1,1 +1,2 @@\n fn a() {{}}\n+// uses {t}\n\
         diff -- config/.env (new file (untracked))\n@@ -0,0 +1,1 @@\n+DB_PASSWORD=hunter2hunter2\n\
         diff -- docs/readme.md (modified)\n@@ -1,1 +1,2 @@\n a\n+b\n```",
        t = token()
    )
}

/// Records what it was asked; never reaches a network.
#[derive(Clone, Default)]
struct Scripted(Arc<Mutex<Vec<(String, String)>>>);

#[async_trait::async_trait]
impl AiClient for Scripted {
    async fn complete(&self, system: &str, user: &str) -> Result<String> {
        self.0
            .lock()
            .unwrap()
            .push((system.to_string(), user.to_string()));
        Ok("feat: done".into())
    }
}

#[test]
fn the_prompt_has_no_token_and_no_env_section_and_keeps_the_rest() {
    let scripted = Scripted::default();
    let privacy = Privacy::default();
    let report = privacy.report();
    let client = PrivacyClient::new(Box::new(scripted.clone()), privacy);

    run(client.complete("system", &prompt())).unwrap();

    let sent = scripted.0.lock().unwrap();
    let (_, user) = &sent[0];
    assert!(!user.contains(TOKEN_BODY), "{user}");
    assert!(user.contains("[REDACTED:github-token]"), "{user}");
    assert!(!user.contains(".env"), "{user}");
    assert!(!user.contains("hunter2"), "{user}");
    assert!(user.contains("diff -- src/lib.rs (modified)"), "{user}");
    assert!(user.contains("diff -- docs/readme.md (modified)"), "{user}");
    // The fence of the last diff survives a dropped section before it.
    assert!(user.trim_end().ends_with("```"), "{user}");

    let summary = report.summary();
    assert_eq!(summary.redactions, 1);
    assert_eq!(summary.excluded_files, vec!["config/.env".to_string()]);
    // What the report keeps is masked.
    let shown = format!("{:?}", report.redactions());
    assert!(!shown.contains(TOKEN_BODY), "{shown}");
}

#[test]
fn a_secret_in_the_system_prompt_is_redacted_too() {
    let p = prepare(&format!("Use {}", token()), "hi", &Privacy::default());
    assert!(!p.system.contains(TOKEN_BODY));
    assert_eq!(p.redactions.len(), 1);
}

#[test]
fn each_protection_can_be_turned_off_alone() {
    let mut keep_files = Privacy::default();
    keep_files.exclude_sensitive_files = false;
    let p = prepare("s", &prompt(), &keep_files);
    assert!(p.user.contains("config/.env"));
    assert!(p.excluded_files.is_empty());
    assert!(!p.user.contains(TOKEN_BODY));

    let mut keep_secrets = Privacy::default();
    keep_secrets.redact_secrets = false;
    let p = prepare("s", &prompt(), &keep_secrets);
    assert!(p.user.contains(TOKEN_BODY));
    assert!(!p.user.contains("config/.env"));
    assert!(p.redactions.is_empty());
}

#[test]
fn a_template_env_file_is_not_excluded_and_other_patterns_apply() {
    let user = "```diff\ndiff -- .env.example (modified)\n+A=1\ndiff -- keys/server.pem (modified)\n+x\ndiff --git a/id_rsa b/id_rsa\n+y\n```";
    let p = prepare("s", user, &Privacy::default());
    assert!(p.user.contains(".env.example"), "{}", p.user);
    assert!(!p.user.contains("server.pem"));
    assert_eq!(
        p.excluded_files,
        vec!["keys/server.pem".to_string(), "id_rsa".to_string()]
    );
}

#[test]
fn a_split_piece_header_of_a_sensitive_file_is_dropped_with_its_diff() {
    let user = "Pieces:\n### H1 src/a.rs hunk 1/1 @@ -1 +1 @@\n```diff\n+a\n```\n### H2 .env hunk 1/1 @@ -1 +1 @@\n```diff\n+B=2\n```\n";
    let p = prepare("s", user, &Privacy::default());
    assert!(p.user.contains("H1 src/a.rs"));
    assert!(!p.user.contains("H2"), "{}", p.user);
    assert!(!p.user.contains("B=2"));
}

#[test]
fn print_only_records_the_prepared_text_and_sends_nothing() {
    let privacy = Privacy::default();
    let report = privacy.report();
    let client = PrivacyClient::print_only(privacy);
    let err = run(client.complete("system", &prompt())).unwrap_err();
    assert_eq!(err, Error::PrintOnly);
    let sent = report.sent();
    assert_eq!(sent.len(), 1);
    assert!(!sent[0].1.contains(TOKEN_BODY));
}

#[test]
fn print_only_needs_no_key_and_no_trust() {
    let dir = tempfile::tempdir().unwrap();
    let gate = TrustGate::load(&ConfigPaths::for_dir(dir.path()));
    let profile = AiProfile::new("p", "P", AiProviderKind::OpenAi);
    let mut privacy = Privacy::default();
    privacy.print_only = true;
    let t = Arc::new(RecordedTransport::new());
    let client =
        gitpoulpi_ai::client_for_in(&profile, None, t.clone(), None, &gate, &privacy).unwrap();
    assert_eq!(run(client.complete("s", "u")).unwrap_err(), Error::PrintOnly);
    assert!(t.requests().is_empty());
}

struct Answer(bool, Arc<Mutex<usize>>);

#[async_trait::async_trait]
impl SendGate for Answer {
    async fn confirm(&self, sent: &Prepared) -> bool {
        assert!(!sent.user.contains(TOKEN_BODY), "the gate sees cleaned text");
        *self.1.lock().unwrap() += 1;
        self.0
    }
}

#[test]
fn a_declined_request_is_not_sent_and_an_approved_one_is_asked_once() {
    let asked = Arc::new(Mutex::new(0));
    let mut privacy = Privacy::default();
    privacy.gate = Some(Arc::new(Answer(false, asked.clone())));
    let scripted = Scripted::default();
    let client = PrivacyClient::new(Box::new(scripted.clone()), privacy);
    assert_eq!(
        run(client.complete("s", &prompt())).unwrap_err(),
        Error::Declined
    );
    assert!(scripted.0.lock().unwrap().is_empty());

    let asked = Arc::new(Mutex::new(0));
    let mut privacy = Privacy::default();
    privacy.gate = Some(Arc::new(Answer(true, asked.clone())));
    let client = PrivacyClient::new(Box::new(scripted.clone()), privacy);
    run(client.complete("s", &prompt())).unwrap();
    run(client.complete("s", &prompt())).unwrap();
    assert_eq!(*asked.lock().unwrap(), 1);
    assert_eq!(scripted.0.lock().unwrap().len(), 2);
}

#[test]
fn the_request_that_reaches_the_server_is_clean() {
    let dir = tempfile::tempdir().unwrap();
    let gate = TrustGate::load(&ConfigPaths::for_dir(dir.path()));
    let mut profile = AiProfile::new("work", "Work", AiProviderKind::OpenAiCompatible);
    profile.base_url = "https://llm.corp.example/v1".into();
    let t = Arc::new(RecordedTransport::new().on(
        "chat/completions",
        200,
        r#"{"choices":[{"message":{"content":"OK"}}]}"#,
    ));
    let client = client_for(&profile, Some("sk-key".into()), t.clone(), &gate).unwrap();
    assert_eq!(run(client.complete("s", &prompt())).unwrap(), "OK");
    let body = t.requests()[0].body.clone().unwrap();
    assert!(!body.contains(TOKEN_BODY), "{body}");
    assert!(!body.contains("hunter2"), "{body}");
    assert!(body.contains("REDACTED"), "{body}");
}
