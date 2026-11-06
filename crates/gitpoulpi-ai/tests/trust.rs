//! The AI key is bound to the origin it was stored for.

use std::sync::Arc;

use gitpoulpi_ai::{client_for, Error};
use gitpoulpi_config::{AiProfile, AiProviderKind, ConfigPaths};
use gitpoulpi_hosting::{RecordedTransport, TrustGate, TrustProblem, TrustSubject};

fn run<F: std::future::Future>(f: F) -> F::Output {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(f)
}

const ANSWER: &str = r#"{"choices":[{"message":{"content":"OK"}}]}"#;

fn profile(base_url: &str) -> AiProfile {
    let mut p = AiProfile::new("work", "Work", AiProviderKind::OpenAiCompatible);
    p.base_url = base_url.to_string();
    p
}

#[test]
fn a_changed_base_url_is_refused_and_nothing_is_sent() {
    let dir = tempfile::tempdir().unwrap();
    let gate = TrustGate::load(&ConfigPaths::for_dir(dir.path()));
    let t = Arc::new(RecordedTransport::new().on("chat/completions", 200, ANSWER));

    let trusted = client_for(
        &profile("https://llm.corp.example/v1"),
        Some("sk-key".into()),
        t.clone(),
        &gate,
    )
    .unwrap();
    assert_eq!(run(trusted.complete("s", "u")).unwrap(), "OK");
    assert!(t.requests()[0].url.starts_with("https://llm.corp.example/"));

    // The profile now points at another server: the key would go there.
    let err = client_for(
        &profile("https://llm.evil.example/v1"),
        Some("sk-key".into()),
        t.clone(),
        &gate,
    )
    .err()
    .expect("refused");
    match err {
        Error::Trust(TrustProblem::OriginChanged { subject, old, new }) => {
            assert_eq!(subject, TrustSubject::Ai("work".into()));
            assert_eq!(old, "https://llm.corp.example/v1");
            assert_eq!(new, "https://llm.evil.example/v1");
        }
        other => panic!("{other:?}"),
    }
    assert_eq!(t.requests().len(), 1, "nothing was sent to the new address");

    // Accepted by a person (what `gitpoulpi ai trust` does).
    gate.trust_ai_origin("work", "https://llm.evil.example/v1")
        .unwrap();
    assert!(client_for(
        &profile("https://llm.evil.example/v1"),
        Some("sk-key".into()),
        t,
        &gate
    )
    .is_ok());
}

#[test]
fn plain_http_is_for_this_machine_only_unless_allowed() {
    let dir = tempfile::tempdir().unwrap();
    let gate = TrustGate::load(&ConfigPaths::for_dir(dir.path()));
    let t = Arc::new(RecordedTransport::new());
    // Ollama on loopback: no key, plain http, no flag needed.
    assert!(client_for(
        &profile("http://localhost:11434/v1"),
        None,
        t.clone(),
        &gate
    )
    .is_ok());
    let err = client_for(
        &profile("http://10.0.0.5:11434/v1"),
        Some("k".into()),
        t,
        &gate,
    )
    .err()
    .expect("refused");
    assert!(matches!(err, Error::Trust(TrustProblem::PlainHttp { .. })));
}

#[test]
fn a_local_agent_has_no_address_to_bind() {
    let dir = tempfile::tempdir().unwrap();
    let gate = TrustGate::load(&ConfigPaths::for_dir(dir.path()));
    let t = Arc::new(RecordedTransport::new());
    let agent = AiProfile::new("agent", "Agent", AiProviderKind::LocalAgent);
    assert!(client_for(&agent, None, t, &gate).is_ok());
    assert!(gate.ai_record_of("agent").is_none());
}
