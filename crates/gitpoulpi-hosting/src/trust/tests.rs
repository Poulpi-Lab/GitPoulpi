use std::sync::Arc;

use gitpoulpi_config::{ConfigPaths, ConnectedUser, Integration, IntegrationKind};

use super::*;
use crate::client::{client_for, HostingClient};
use crate::http::{HttpRequest, HttpTransport, RecordedTransport};

fn block_on<F: std::future::Future>(f: F) -> F::Output {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(f)
}

fn gitlab(id: &str, url: &str) -> Integration {
    Integration::new(id, IntegrationKind::GitlabSelfManaged, url)
}

fn connected(mut i: Integration, login: &str, id: &str) -> Integration {
    i.user = Some(ConnectedUser {
        login: login.into(),
        name: String::new(),
        id: id.into(),
    });
    i
}

fn gate_in(dir: &tempfile::TempDir) -> (ConfigPaths, TrustGate) {
    let paths = ConfigPaths::for_dir(dir.path());
    let gate = TrustGate::load(&paths);
    (paths, gate)
}

fn pending_for(id: &str) -> Vec<TrustProblem> {
    peek_pending()
        .into_iter()
        .filter(|p| p.subject().name() == id)
        .collect()
}

#[test]
fn a_changed_address_is_refused_before_anything_is_sent() {
    let dir = tempfile::tempdir().unwrap();
    let (_paths, gate) = gate_in(&dir);
    let t =
        Arc::new(RecordedTransport::new().on("/api/v4/user", 200, r#"{"id":1,"username":"ada"}"#));

    let trusted = connected(gitlab("chg-origin", "https://git.corp.example"), "ada", "1");
    assert!(client_for(trusted.clone(), "tok".into(), t.clone(), &gate).is_ok());

    // The config was edited to point elsewhere.
    let moved = connected(gitlab("chg-origin", "https://git.evil.example"), "ada", "1");
    let err = client_for(moved, "tok".into(), t.clone(), &gate)
        .err()
        .expect("refused");
    match err {
        Error::Trust(TrustProblem::OriginChanged { old, new, subject }) => {
            assert_eq!(old, "https://git.corp.example");
            assert_eq!(new, "https://git.evil.example");
            assert_eq!(subject, TrustSubject::Integration("chg-origin".into()));
        }
        other => panic!("{other:?}"),
    }
    assert!(t.requests().is_empty(), "no request may leave on refusal");
    assert_eq!(pending_for("chg-origin").len(), 1);

    // Accepting the new address (what `trust --accept-origin` does) lets it through.
    let moved = connected(gitlab("chg-origin", "https://git.evil.example"), "ada", "1");
    gate.trust_origin(&moved).unwrap();
    assert!(client_for(moved, "tok".into(), t, &gate).is_ok());
    assert!(pending_for("chg-origin").is_empty());
}

#[test]
fn equivalent_spellings_of_the_address_are_not_a_change() {
    let dir = tempfile::tempdir().unwrap();
    let (_p, gate) = gate_in(&dir);
    let t = Arc::new(RecordedTransport::new());
    let a = connected(gitlab("spell", "https://Git.Corp.Example:443/"), "a", "1");
    assert!(client_for(a, "t".into(), t.clone(), &gate).is_ok());
    let b = connected(gitlab("spell", "https://git.corp.example"), "a", "1");
    assert!(client_for(b, "t".into(), t, &gate).is_ok());
}

#[test]
fn a_request_off_the_integrations_origins_is_refused_unsent() {
    let inner = Arc::new(RecordedTransport::new().on("example", 200, "{}"));
    let guard = GuardedTransport::new(
        inner.clone(),
        TrustSubject::Integration("guarded".into()),
        "https://git.corp.example".into(),
        vec!["https://git.corp.example".into()],
        None,
    );
    block_on(guard.send(HttpRequest::get("https://git.corp.example/api/v4/user"))).unwrap();
    for url in [
        "https://evil.example/api/v4/user",
        "http://git.corp.example/api/v4/user",
        "https://git.corp.example:8443/x",
        "https://git.corp.example.evil.test/x",
    ] {
        let err = block_on(guard.send(HttpRequest::get(url))).unwrap_err();
        assert!(
            matches!(err, Error::Trust(TrustProblem::OriginChanged { .. })),
            "{url}"
        );
    }
    assert_eq!(inner.requests().len(), 1);
}

#[test]
fn saas_clients_reach_their_api_host_and_a_client_cannot_leave_it() {
    let dir = tempfile::tempdir().unwrap();
    let (_p, gate) = gate_in(&dir);
    let t = Arc::new(RecordedTransport::new().on(
        "api.github.com/user",
        200,
        r#"{"login":"a","id":3}"#,
    ));
    let gh = connected(
        Integration::new("saas", IntegrationKind::Github, "https://github.com"),
        "a",
        "3",
    );
    let client = client_for(gh, "tok".into(), t.clone(), &gate).unwrap();
    assert_eq!(block_on(client.current_user()).unwrap().id, "3");
    assert_eq!(t.requests().len(), 1);
}

#[test]
fn plain_http_needs_a_flag_except_on_loopback() {
    let dir = tempfile::tempdir().unwrap();
    let (_p, gate) = gate_in(&dir);
    let t = Arc::new(RecordedTransport::new());
    let nas = connected(gitlab("plain", "http://nas.local:3000"), "a", "1");
    let err = client_for(nas.clone(), "t".into(), t.clone(), &gate)
        .err()
        .unwrap();
    assert!(
        matches!(err, Error::Trust(TrustProblem::PlainHttp { .. })),
        "{err:?}"
    );
    gate.set_allow_plain_http(&nas, true).unwrap();
    assert!(client_for(nas, "t".into(), t.clone(), &gate).is_ok());
    let local = connected(gitlab("plain-loop", "http://127.0.0.1:3000"), "a", "1");
    assert!(client_for(local, "t".into(), t, &gate).is_ok());
}

fn github_client(
    gate: &TrustGate,
    id: &str,
    answer: &str,
) -> (Integration, Box<dyn HostingClient>) {
    let t = Arc::new(RecordedTransport::new().on("/user", 200, answer));
    let i = connected(
        Integration::new(id, IntegrationKind::Github, "https://github.com"),
        "ada",
        "7",
    );
    let c = client_for(i.clone(), "tok".into(), t, gate).unwrap();
    (i, c)
}

#[test]
fn a_token_that_now_belongs_to_another_account_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let (_p, gate) = gate_in(&dir);
    let (i, same) = github_client(&gate, "acct", r#"{"login":"ada","id":7}"#);
    gate.adopt(&i).unwrap();
    block_on(gate.verify_account(&i, same.as_ref(), VerifyReason::TokenStored)).unwrap();

    // The login is the same, the id is not: another person took the name.
    let (_, other) = github_client(&gate, "acct", r#"{"login":"ada","id":99}"#);
    let err =
        block_on(gate.verify_account(&i, other.as_ref(), VerifyReason::TokenStored)).unwrap_err();
    match err {
        Error::Trust(TrustProblem::AccountChanged { old, new, .. }) => {
            assert_eq!((old.id.as_str(), new.id.as_str()), ("7", "99"));
        }
        other => panic!("{other:?}"),
    }
    // A rename with the same id is fine.
    let (_, renamed) = github_client(&gate, "acct", r#"{"login":"ada2","id":7}"#);
    block_on(gate.verify_account(&i, renamed.as_ref(), VerifyReason::TokenStored)).unwrap();
    assert_eq!(gate.record_of("acct").unwrap().account_login, "ada2");
}

#[test]
fn the_account_check_runs_when_due_only() {
    let dir = tempfile::tempdir().unwrap();
    let (_p, gate) = gate_in(&dir);
    let (i, c) = github_client(&gate, "due", r#"{"login":"ada","id":7}"#);
    gate.adopt(&i).unwrap();
    // No record check yet (checked_at is 0): periodic is due and records.
    block_on(gate.verify_account(&i, c.as_ref(), VerifyReason::Periodic)).unwrap();
    let t = Arc::new(RecordedTransport::new());
    let quiet = client_for(i.clone(), "tok".into(), t.clone(), &gate).unwrap();
    // Just checked: a periodic check asks nobody (the transport has no answer).
    block_on(gate.verify_account(&i, quiet.as_ref(), VerifyReason::Periodic)).unwrap();
    assert!(t.requests().is_empty());
    // A stored token is always checked, and the missing answer shows it asked.
    assert!(block_on(gate.verify_account(&i, quiet.as_ref(), VerifyReason::TokenStored)).is_err());
}

#[test]
fn an_integration_connected_before_the_upgrade_is_recorded_silently() {
    let dir = tempfile::tempdir().unwrap();
    let (paths, gate) = gate_in(&dir);
    let t = Arc::new(RecordedTransport::new());
    let old = connected(gitlab("legacy", "https://git.corp.example"), "ada", "42");
    assert!(client_for(old, "t".into(), t.clone(), &gate).is_ok());
    let rec = TrustStore::load(&TrustStore::file_in(&paths))
        .unwrap()
        .integration("legacy")
        .cloned()
        .expect("recorded on first use");
    assert_eq!(rec.origin, "https://git.corp.example");
    assert_eq!(
        (rec.account_id.as_str(), rec.account_login.as_str()),
        ("42", "ada")
    );

    // One that was never connected is held in memory until a check passes.
    let fresh = gitlab("never", "https://git.other.example");
    assert!(client_for(fresh.clone(), "t".into(), t, &gate).is_ok());
    assert!(gate.record_of("never").is_none());
    gate.record_after_check(
        &fresh,
        &Account {
            login: "bob".into(),
            id: "5".into(),
        },
    )
    .unwrap();
    assert_eq!(gate.record_of("never").unwrap().account_id, "5");
}

#[test]
fn the_store_is_in_the_private_directory_not_the_config() {
    let dir = tempfile::tempdir().unwrap();
    let (paths, gate) = gate_in(&dir);
    gate.adopt(&gitlab("where", "https://git.corp.example"))
        .unwrap();
    assert!(paths.private_dir().join("trust.toml").is_file());
    assert!(!paths.config_file().exists());
}

#[test]
fn a_corrupt_store_refuses_instead_of_re_trusting() {
    let dir = tempfile::tempdir().unwrap();
    let (paths, gate) = gate_in(&dir);
    let file = TrustStore::file_in(&paths);
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::write(&file, "[[integration]\nnope").unwrap();
    let i = connected(gitlab("corrupt", "https://git.corp.example"), "a", "1");
    let err = gate.admit(&i).unwrap_err();
    assert!(matches!(err, Error::TrustStore(_)), "{err:?}");
}

#[test]
fn forget_removes_the_record_and_every_cache_of_the_integration() {
    let dir = tempfile::tempdir().unwrap();
    let (paths, gate) = gate_in(&dir);
    let mine = connected(gitlab("gone", "https://git.corp.example"), "ada", "1");
    let other = connected(gitlab("kept", "https://git.other.example"), "bob", "2");
    gate.adopt(&mine).unwrap();
    gate.adopt(&other).unwrap();

    // The Clone dialog's saved lists.
    let lists = paths.cache_dir().join("clone-lists");
    for (i, login) in [(&mine, "ada"), (&mine, "ada2"), (&other, "bob")] {
        let key = crate::list_cache::ListKey::of(i, Some(login)).unwrap();
        let list = crate::list_cache::CachedList::new(key, 1, Vec::new());
        crate::list_cache::store(&lists, &list).unwrap();
    }
    // The Radar account cache.
    let account = crate::account::cache_path(paths.config_dir());
    let mut cache = crate::account::AccountCache::default();
    for id in ["gone", "kept"] {
        cache.entries.insert(
            id.into(),
            serde_json::from_str(
                r#"{"fetched_at":1,"query_hash":"q","items":{"review_requests":[],"open_requests":[],"assigned_issues":[],"notes":[]}}"#,
            )
            .unwrap_or_else(|e| panic!("{e}")),
        );
    }
    cache.save(&account).unwrap();
    // The remote review drafts of the host.
    let drafts = paths.config_dir().join("remote-reviews.toml");
    std::fs::write(
        &drafts,
        "[drafts.\"git.corp.example/g/p\".k]\ncomments = []\n[drafts.\"git.other.example/g/p\".k]\ncomments = []\n[ai_review.\"git.corp.example/g/p\"]\n",
    )
    .unwrap();

    let report = forget(&paths, &mine);
    assert!(report.trust_record);
    assert_eq!(report.list_cache_files, 2);
    assert!(report.account_cache);
    assert_eq!(report.review_drafts, 2);

    assert!(gate.record_of("gone").is_none());
    assert!(gate.record_of("kept").is_some());
    let left: Vec<_> = std::fs::read_dir(&lists).unwrap().flatten().collect();
    assert_eq!(left.len(), 1);
    let cache = crate::account::AccountCache::load(&account);
    assert!(cache.entries.contains_key("kept") && !cache.entries.contains_key("gone"));
    let text = std::fs::read_to_string(&drafts).unwrap();
    assert!(
        text.contains("git.other.example") && !text.contains("git.corp.example"),
        "{text}"
    );
}

#[test]
fn ai_profiles_are_bound_to_their_origin() {
    let dir = tempfile::tempdir().unwrap();
    let (_p, gate) = gate_in(&dir);
    gate.admit_ai("ai-bound", "https://api.openai.com/v1", true)
        .unwrap();
    let err = gate
        .admit_ai("ai-bound", "https://api.evil.example/v1", true)
        .unwrap_err();
    match err {
        Error::Trust(TrustProblem::OriginChanged { subject, .. }) => {
            assert_eq!(subject, TrustSubject::Ai("ai-bound".into()));
        }
        other => panic!("{other:?}"),
    }
    // Loopback http (Ollama) needs no flag; elsewhere it does.
    gate.admit_ai("ai-local", "http://localhost:11434/v1", true)
        .unwrap();
    assert!(matches!(
        gate.admit_ai("ai-lan", "http://10.0.0.5:11434/v1", true)
            .unwrap_err(),
        Error::Trust(TrustProblem::PlainHttp { .. })
    ));
    gate.trust_ai_origin("ai-bound", "https://api.evil.example/v1")
        .unwrap();
    gate.admit_ai("ai-bound", "https://api.evil.example/v1", true)
        .unwrap();
}

#[test]
fn the_problem_spellings_a_script_reads_are_stable() {
    let p = TrustProblem::OriginChanged {
        subject: TrustSubject::Integration("x".into()),
        old: "https://a".into(),
        new: "https://b".into(),
    };
    assert_eq!(p.change(), "origin");
    assert_eq!(
        p.fix(),
        vec!["gitpoulpi hosting integrations trust x --accept-origin https://b"]
    );
    let ai = p.clone().with_subject(TrustSubject::Ai("prof".into()));
    assert_eq!(
        ai.fix(),
        vec!["gitpoulpi ai trust prof --accept-origin https://b"]
    );
    let kinds = [
        TrustProblem::AccountChanged {
            subject: TrustSubject::Integration("x".into()),
            old: Account::default(),
            new: Account::default(),
        }
        .change(),
        TrustProblem::UntrustedCertificate {
            subject: TrustSubject::Integration("x".into()),
            new: CertInfo::default(),
        }
        .change(),
        TrustProblem::CertificateChanged {
            subject: TrustSubject::Integration("x".into()),
            old: CertInfo::default(),
            new: CertInfo::default(),
        }
        .change(),
        TrustProblem::PlainHttp {
            subject: TrustSubject::Integration("x".into()),
            origin: String::new(),
        }
        .change(),
    ];
    assert_eq!(
        kinds,
        [
            "account",
            "untrusted_certificate",
            "certificate",
            "plain_http"
        ]
    );
}
