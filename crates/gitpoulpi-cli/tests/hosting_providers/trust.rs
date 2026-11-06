//! The integration security check from the CLI: a change since the trusted
//! state is a JSON refusal with exit code 2 (nothing sent to the new
//! address), and `integrations trust` accepts exactly what it is told to.

use std::path::Path;

use serde_json::Value;

use crate::common::*;

const USER: &str = r#"{"id":1,"username":"mara","name":"Mara"}"#;

fn host() -> Mock {
    Mock::start(vec![
        route("GET", "/api/v4/user", USER),
        route("GET", "/api/v4/projects", "[]"),
    ])
}

fn config(url: &str) -> String {
    integration(
        "gl",
        "gitlab_self_managed",
        url,
        "user = { login = \"mara\", id = \"1\" }",
    )
}

fn rewrite_url(cli: &Cli, url: &str) {
    std::fs::write(cli.config.path().join("config.toml"), config(url)).unwrap();
}

fn trust_file(cli: &Cli) -> std::path::PathBuf {
    cli.config
        .path()
        .join("data")
        .join("private")
        .join("trust.toml")
}

fn here() -> &'static Path {
    Path::new(".")
}

fn exit_code(cli: &Cli, args: &[&str]) -> (i32, Value) {
    // `output` reports success only; a refusal is exit code 2 with JSON.
    let (ok, json) = cli.output(here(), args);
    (if ok { 0 } else { 2 }, json)
}

#[test]
fn a_moved_integration_is_refused_before_any_request_and_accepted_by_trust() {
    let old = host();
    let new = host();
    let cli = Cli::new("trust-origin", &config(&old.url()), &[("gl", "tok")]);
    cli.json(here(), &["hosting", "projects", "--id", "gl"]);
    assert!(
        trust_file(&cli).is_file(),
        "the first use recorded the trust"
    );

    // The config is edited (by a person, a script or something worse).
    rewrite_url(&cli, &new.url());
    let (code, json) = exit_code(&cli, &["hosting", "projects", "--id", "gl"]);
    assert_eq!(code, 2, "{json}");
    assert_eq!(json["error"], "trust_check");
    assert_eq!(json["subject"], "gl");
    assert_eq!(json["subject_kind"], "integration");
    assert_eq!(json["change"], "origin");
    assert_eq!(json["old"], old.url());
    assert_eq!(json["new"], new.url());
    let fix = json["fix"][0].as_str().unwrap();
    assert!(
        fix.contains("integrations trust gl --accept-origin") && fix.contains(&new.url()),
        "{fix}"
    );
    assert!(
        new.requests().is_empty(),
        "nothing may reach the new address: {:?}",
        new.requests()
    );

    // The wrong address to accept is refused too.
    let (ok, json) = cli.output(
        here(),
        &[
            "hosting",
            "integrations",
            "trust",
            "gl",
            "--accept-origin",
            "http://127.0.0.1:1",
        ],
    );
    assert!(!ok, "{json}");
    assert!(new.requests().is_empty());

    // The exact one is accepted, and then the integration works.
    let accepted = cli.json(
        here(),
        &[
            "hosting",
            "integrations",
            "trust",
            "gl",
            "--accept-origin",
            &new.url(),
        ],
    );
    assert!(accepted["accepted"][0]
        .as_str()
        .unwrap()
        .contains("address"));
    cli.json(here(), &["hosting", "projects", "--id", "gl"]);
    assert!(!new.requests().is_empty());
}

#[test]
fn plain_http_off_this_machine_is_refused_until_allowed() {
    let cli = Cli::new(
        "trust-plain",
        &config("http://nas.invalid:3000"),
        &[("gl", "tok")],
    );
    let (code, json) = exit_code(&cli, &["hosting", "projects", "--id", "gl"]);
    assert_eq!(code, 2, "{json}");
    assert_eq!(json["error"], "trust_check");
    assert_eq!(json["change"], "plain_http");
    assert_eq!(json["new"], "http://nas.invalid:3000");
    let shown = cli.json(
        here(),
        &[
            "hosting",
            "integrations",
            "trust",
            "gl",
            "--allow-insecure-http",
            "on",
        ],
    );
    assert_eq!(shown["trusted"]["allow_plain_http"], true);
}

#[test]
fn a_token_of_another_account_is_refused() {
    let mock = host();
    let cli = Cli::new("trust-account", &config(&mock.url()), &[("gl", "tok")]);
    // Trusted for account 99; the host says the token is account 1.
    cli.json(
        here(),
        &[
            "hosting",
            "integrations",
            "trust",
            "gl",
            "--accept-account",
            "99",
        ],
    );
    let (code, json) = exit_code(&cli, &["hosting", "projects", "--id", "gl"]);
    assert_eq!(code, 2, "{json}");
    assert_eq!(json["error"], "trust_check");
    assert_eq!(json["change"], "account");
    assert_eq!(json["new"], "1");
    assert!(json["old"].as_str().unwrap().contains("99"));
    assert!(
        mock.to("/projects").is_empty(),
        "the listing was never asked for: {:?}",
        mock.requests()
    );
    // Accepting the account that really owns the token lets it through.
    cli.json(
        here(),
        &[
            "hosting",
            "integrations",
            "trust",
            "gl",
            "--accept-account",
            "1",
        ],
    );
    cli.json(here(), &["hosting", "projects", "--id", "gl"]);
}

#[test]
fn a_certificate_pin_is_recorded_by_its_exact_fingerprint() {
    let mock = host();
    let cli = Cli::new("trust-cert", &config(&mock.url()), &[("gl", "tok")]);
    let (ok, json) = cli.output(
        here(),
        &[
            "hosting",
            "integrations",
            "trust",
            "gl",
            "--accept-certificate",
            "abcd",
        ],
    );
    assert!(!ok, "a short fingerprint is not accepted: {json}");
    let fp = "ab".repeat(32);
    let shown = cli.json(
        here(),
        &[
            "hosting",
            "integrations",
            "trust",
            "gl",
            "--accept-certificate",
            &fp,
        ],
    );
    assert_eq!(shown["trusted"]["tls"]["spki_sha256"], fp);
    assert_eq!(shown["trusted"]["tls"]["mode"], "tofu");
    let strict = cli.json(
        here(),
        &[
            "hosting",
            "integrations",
            "trust",
            "gl",
            "--strict-tls",
            "on",
        ],
    );
    assert_eq!(strict["trusted"]["tls"]["mode"], "strict");
}

#[test]
fn removing_an_integration_forgets_its_trust() {
    let mock = host();
    let cli = Cli::new("trust-forget", &config(&mock.url()), &[("gl", "tok")]);
    cli.json(here(), &["hosting", "projects", "--id", "gl"]);
    assert!(std::fs::read_to_string(trust_file(&cli))
        .unwrap()
        .contains("gl"));
    cli.json(here(), &["hosting", "integrations", "remove", "gl"]);
    let left = std::fs::read_to_string(trust_file(&cli)).unwrap_or_default();
    assert!(!left.contains("id = \"gl\""), "{left}");
}

#[test]
fn the_trust_status_names_what_is_trusted() {
    let mock = host();
    let cli = Cli::new("trust-status", &config(&mock.url()), &[("gl", "tok")]);
    cli.json(here(), &["hosting", "projects", "--id", "gl"]);
    let shown = cli.json(here(), &["hosting", "integrations", "trust", "gl"]);
    assert_eq!(shown["trusted"]["origin"], mock.url());
    assert_eq!(shown["trusted"]["account_id"], "1");
    assert_eq!(shown["current_origin"], mock.url());
}
