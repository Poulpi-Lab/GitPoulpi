//! The CLI's answer to a trust refusal.

use gitpoulpi_hosting::trust::CertInfo;
use gitpoulpi_hosting::{Account, TrustProblem, TrustSubject};

use super::*;

fn subject() -> TrustSubject {
    TrustSubject::Integration("t-answer".into())
}

fn cert(fp: &str) -> CertInfo {
    CertInfo {
        spki_sha256: fp.into(),
        subject: "CN=git.corp.example".into(),
        ..CertInfo::default()
    }
}

#[test]
fn every_change_kind_has_its_json_and_exit_code_2() {
    let cases = [
        (
            TrustProblem::OriginChanged {
                subject: subject(),
                old: "https://a".into(),
                new: "https://b".into(),
            },
            "origin",
            "--accept-origin https://b",
        ),
        (
            TrustProblem::AccountChanged {
                subject: subject(),
                old: Box::new(Account {
                    login: "ada".into(),
                    id: "1".into(),
                }),
                new: Box::new(Account {
                    login: "eve".into(),
                    id: "2".into(),
                }),
            },
            "account",
            "--accept-account 2",
        ),
        (
            TrustProblem::UntrustedCertificate {
                subject: subject(),
                new: Box::new(cert("aa")),
            },
            "untrusted_certificate",
            "--accept-certificate aa",
        ),
        (
            TrustProblem::CertificateChanged {
                subject: subject(),
                old: Box::new(cert("aa")),
                new: Box::new(cert("bb")),
            },
            "certificate",
            "--accept-certificate bb",
        ),
        (
            TrustProblem::PlainHttp {
                subject: subject(),
                origin: "http://nas".into(),
            },
            "plain_http",
            "--allow-insecure-http on",
        ),
    ];
    for (problem, change, fix_tail) in cases {
        let out = trust_refusal(&problem);
        assert_eq!(out.exit_code, 2, "{change}");
        assert_eq!(out.json["error"], "trust_check");
        assert_eq!(out.json["subject"], "t-answer");
        assert_eq!(out.json["change"], change);
        let fix = out.json["fix"][0].as_str().unwrap();
        assert!(fix.ends_with(fix_tail), "{fix}");
        assert!(fix.starts_with("gitpoulpi hosting integrations trust t-answer"));
        assert!(out.human.starts_with(TRUST_REFUSAL), "{}", out.human);
    }
}

#[test]
fn only_the_problem_the_error_carries_is_answered() {
    let mine = TrustProblem::PlainHttp {
        subject: TrustSubject::Integration("t-mine".into()),
        origin: "http://mine".into(),
    };
    let other = TrustProblem::PlainHttp {
        subject: TrustSubject::Integration("t-other".into()),
        origin: "http://other".into(),
    };
    gitpoulpi_hosting::trust::report(&mine);
    gitpoulpi_hosting::trust::report(&other);
    let answered = trust_answer(Err(format!("failed: {mine}"))).unwrap();
    assert_eq!(answered.exit_code, 2);
    assert_eq!(answered.json["subject"], "t-mine");
    // The other task's refusal is still queued for whoever owns it.
    let left = gitpoulpi_hosting::trust::peek_pending();
    assert!(left.contains(&other));
    gitpoulpi_hosting::trust::clear_pending(other.subject());
    // An ordinary failure stays one.
    assert_eq!(
        trust_answer(Err("no token".into())).unwrap_err(),
        "no token"
    );
}
