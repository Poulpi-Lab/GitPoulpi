//! The integration form and card around the security check: the token's
//! rights and expiry, the pinning and plain http toggles, an address change,
//! and an account change. Every page here keeps its trust records in a
//! private temporary directory (`IntegrationsPage::trust_paths`).

use super::*;
use gitpoulpi_hosting::trust::{CertInfo, TlsMode};
use gitpoulpi_hosting::{Account, TrustGate, TrustProblem, TrustSubject};

const GH_USER: &str =
    r#"{"login":"octo","id":9,"name":"Octo Cat","html_url":"https://github.com/octo"}"#;

fn github_page(headers: &[(&str, &str)]) -> (IntegrationsPage, Arc<RecordedTransport>) {
    page_with(RecordedTransport::new().on_with_headers("/user", 200, GH_USER, headers))
}

/// Connects a GitHub profile with `token` and returns the page on its card.
fn connect_github(page: &mut IntegrationsPage, h: &mut H, token: &str) {
    let kind = page.kind_index(IntegrationKind::Github).unwrap();
    page.select_kind(kind, &h.config);
    page.add_profile(&h.config);
    type_str(page, h, token);
    page.activate(Item::Connect, &mut h.env());
    wait_connected(page, h);
}

fn gitea_ok() -> RecordedTransport {
    RecordedTransport::new().on(
        "/api/v1/user",
        200,
        r#"{"id":7,"login":"kai","full_name":"Kai G","html_url":"https://gitlab.example.com/kai"}"#,
    )
}

fn gate_of(page: &IntegrationsPage) -> TrustGate {
    TrustGate::load(page.trust_paths.as_ref().unwrap())
}

#[test]
fn a_token_with_extra_rights_shows_them_with_the_expiry_and_the_advice() {
    let mut h = H::new();
    let (mut page, _) = github_page(&[
        (
            "X-OAuth-Scopes",
            "repo, read:user, admin:public_key, gist, delete_repo",
        ),
        ("github-authentication-token-expiration", "2026-12-01 10:00:00 UTC"),
    ]);
    connect_github(&mut page, &mut h, "ghp_secret");
    let screen = text(&draw(&mut page, &h, 90, 30, None));
    for want in [
        "This token has more rights than GitPoulpi needs:",
        "delete_repo",
        "gist",
        "Prefer a fine-grained token with an expiry date",
        "Expires",
        "2026-12-01",
    ] {
        assert!(screen.contains(want), "missing {want:?}:\n{screen}");
    }
    // The sensitive scope is drawn in the danger colour, the other is not.
    let buf = draw(&mut page, &h, 90, 30, None);
    let colour_of = |word: &str| {
        (0..buf.area.height).find_map(|y| {
            let line: String = (0..buf.area.width).map(|x| buf[(x, y)].symbol()).collect();
            line.find(word).map(|x| buf[(x as u16, y)].fg)
        })
    };
    assert_eq!(colour_of("delete_repo"), Some(crate::th().status.danger));
    assert_ne!(colour_of("gist"), Some(crate::th().status.danger));
}

#[test]
fn exact_rights_and_no_expiry_leave_the_card_quiet() {
    let mut h = H::new();
    let (mut page, _) = github_page(&[("X-OAuth-Scopes", "repo, read:user, admin:public_key")]);
    connect_github(&mut page, &mut h, "0123456789abcdef");
    let screen = text(&draw(&mut page, &h, 90, 30, None));
    for gone in ["more rights", "Prefer a fine-grained", "Expires", "expiry"] {
        assert!(!screen.contains(gone), "{gone:?} shown:\n{screen}");
    }
}

#[test]
fn a_classic_token_gets_the_advice_even_with_nothing_extra() {
    let mut h = H::new();
    let (mut page, _) = github_page(&[("X-OAuth-Scopes", "repo, read:user, admin:public_key")]);
    connect_github(&mut page, &mut h, "ghp_exactscopes");
    let screen = text(&draw(&mut page, &h, 90, 30, None));
    assert!(screen.contains("Prefer a fine-grained token"), "{screen}");
    assert!(!screen.contains("more rights"), "{screen}");
}

#[test]
fn the_toggles_follow_the_address_in_the_form() {
    let mut h = H::new();
    let mut page = IntegrationsPage::default();
    page.select_kind(GITLAB_SM, &h.config);
    page.add_profile(&h.config);
    type_str(&mut page, &mut h, "http://gitlab.example.com");
    let items = page.items(&h.config);
    assert!(items.contains(&Item::AllowPlain), "{items:?}");
    assert!(!items.contains(&Item::StrictPin), "{items:?}");
    let screen = text(&draw(&mut page, &h, 90, 40, None));
    assert!(screen.contains("Allow plain http (not encrypted)"), "{screen}");
    assert!(screen.contains("Warning: the token and everything else"), "{screen}");

    // The warning is red.
    let buf = draw(&mut page, &h, 90, 40, None);
    let warn_fg = (0..buf.area.height).find_map(|y| {
        let line: String = (0..buf.area.width).map(|x| buf[(x, y)].symbol()).collect();
        line.find("Warning: the token").map(|x| buf[(x as u16, y)].fg)
    });
    assert_eq!(warn_fg, Some(crate::th().status.danger));

    // This machine's own address is fine in clear text: no toggle.
    let mut page = IntegrationsPage::default();
    page.select_kind(GITLAB_SM, &h.config);
    page.add_profile(&h.config);
    type_str(&mut page, &mut h, "http://localhost:3000");
    let items = page.items(&h.config);
    assert!(!items.contains(&Item::AllowPlain), "{items:?}");

    // https: pinning is offered, plain http is not.
    let mut page = IntegrationsPage::default();
    page.select_kind(GITLAB_SM, &h.config);
    page.add_profile(&h.config);
    type_str(&mut page, &mut h, "gitlab.example.com");
    let items = page.items(&h.config);
    assert!(items.contains(&Item::StrictPin) && !items.contains(&Item::AllowPlain));
    let screen = text(&draw(&mut page, &h, 90, 40, None));
    assert!(screen.contains("[ ] Strict certificate pinning"), "{screen}");
    page.activate(Item::StrictPin, &mut h.env());
    let screen = text(&draw(&mut page, &h, 90, 40, None));
    assert!(screen.contains("[x] Strict certificate pinning"), "{screen}");
}

#[test]
fn a_remote_http_address_is_refused_without_the_toggle_and_sends_nothing() {
    let mut h = H::new();
    let (mut page, transport) = page_with(gitlab_ok());
    fill_sm_form(&mut page, &mut h, "http://gitlab.example.com", "glpat-GOOD");
    page.activate(Item::Connect, &mut h.env());
    wait_connected(&mut page, &mut h);
    assert_eq!(page.mode, Mode::Form(None), "the form stays");
    let error = page.error.clone().unwrap_or_default();
    assert!(error.contains("not encrypted"), "{error}");
    assert!(transport.requests().is_empty(), "nothing was sent");
    assert!(h.config.integrations.is_empty());
    assert_eq!(h.tokens.load("integration:gitlab-self-managed").unwrap(), None);
}

#[test]
fn the_plain_http_toggle_lets_a_remote_http_address_connect_and_is_remembered() {
    let mut h = H::new();
    let (mut page, transport) = page_with(gitlab_ok());
    fill_sm_form(&mut page, &mut h, "http://gitlab.example.com", "glpat-GOOD");
    page.activate(Item::AllowPlain, &mut h.env());
    page.activate(Item::Connect, &mut h.env());
    wait_connected(&mut page, &mut h);
    assert_eq!(page.view(&h.config), View::Connected(0));
    assert!(!transport.requests().is_empty());
    let id = h.config.integrations[0].id.clone();
    let record = gate_of(&page).record_of(&id).unwrap();
    assert!(record.allow_plain_http);
    assert_eq!(record.origin, "http://gitlab.example.com");
    assert_eq!(record.account_id, "7");
}

#[test]
fn a_token_of_another_account_is_refused_before_it_is_stored_and_queued_for_the_dialog() {
    let mut h = H::new();
    let id = h.connected(IntegrationKind::Gitea, "kai");
    let integration = h.config.integrations[0].clone();
    // The integration was trusted for account 99; the check answers id 7.
    let (mut page, _) = page_with(gitea_ok());
    gate_of(&page)
        .trust_account(
            &integration,
            &Account {
                login: "old".into(),
                id: "99".into(),
            },
        )
        .unwrap();
    let kind = page.kind_index(IntegrationKind::Gitea).unwrap();
    page.select_kind(kind, &h.config);
    page.begin_form(&h.config, Some(0));
    page.token = "glpat-NEW".into();
    page.activate(Item::Connect, &mut h.env());
    wait_connected(&mut page, &mut h);
    assert_eq!(page.mode, Mode::Form(Some(id.clone())));
    assert_eq!(
        h.tokens.load(&integration.token_ref).unwrap().as_deref(),
        Some("glpat-secret"),
        "the typed token was not stored"
    );
    let queued = gitpoulpi_hosting::trust::peek_pending()
        .into_iter()
        .find(|p| p.subject() == &TrustSubject::Integration(id.clone()));
    assert!(
        matches!(queued, Some(TrustProblem::AccountChanged { .. })),
        "{queued:?}"
    );
    gitpoulpi_hosting::trust::clear_pending(&TrustSubject::Integration(id));
}

#[test]
fn a_changed_address_needs_a_token_typed_for_it() {
    let mut h = H::new();
    let id = h.connected(IntegrationKind::GitlabSelfManaged, "kai");
    let (mut page, transport) = page_with(gitlab_ok());
    // Trusted for the old address.
    let integration = h.config.integrations[0].clone();
    gate_of(&page)
        .record_after_check(
            &integration,
            &Account {
                login: "kai".into(),
                id: "7".into(),
            },
        )
        .unwrap();
    page.select_kind(GITLAB_SM, &h.config);
    page.begin_form(&h.config, Some(0));
    page.host = "https://elsewhere.example.com".into();
    // No token typed: the saved one must not travel there.
    page.activate(Item::Connect, &mut h.env());
    assert_eq!(page.mode, Mode::Form(Some(id.clone())));
    assert!(
        page.error
            .clone()
            .unwrap_or_default()
            .contains("enter the token again"),
        "{:?}",
        page.error
    );
    assert!(transport.requests().is_empty());

    // With a typed token the new address is the person's own choice.
    page.token = "glpat-NEW".into();
    page.activate(Item::Connect, &mut h.env());
    wait_connected(&mut page, &mut h);
    assert_eq!(page.view(&h.config), View::Connected(0));
    assert_eq!(h.config.integrations[0].url, "https://elsewhere.example.com");
    let record = gate_of(&page).record_of(&id).unwrap();
    assert_eq!(record.origin, "https://elsewhere.example.com");
    assert!(transport.requests().iter().all(|r| r.url.starts_with("https://elsewhere.example.com")));
}

#[test]
fn trusting_a_new_address_opens_the_form_on_the_token_with_the_hint() {
    let mut h = H::new();
    let id = h.connected(IntegrationKind::GitlabSelfManaged, "kai");
    let mut page = IntegrationsPage::default();
    page.begin_retoken(&h.config, &id);
    assert_eq!(page.view(&h.config), View::Form);
    assert_eq!(page.zone, Zone::Body);
    assert_eq!(page.current_item(&h.config), Some(Item::Token));
    assert!(page.text_focused(&h.config));
    let screen = text(&draw(&mut page, &h, 90, 40, None));
    assert!(
        screen.contains("Re-entering the token is recommended."),
        "{screen}"
    );
    // Any other form opens without it.
    page.begin_form(&h.config, Some(0));
    let screen = text(&draw(&mut page, &h, 90, 40, None));
    assert!(!screen.contains("Re-entering"), "{screen}");
}

#[test]
fn the_connected_card_shows_the_pinned_fingerprint() {
    let mut h = H::new();
    let id = h.connected(IntegrationKind::GitlabSelfManaged, "kai");
    let mut page = IntegrationsPage::default();
    let cert = CertInfo {
        spki_sha256: "ab12cd34".repeat(8),
        subject: "CN=gitlab".into(),
        ..Default::default()
    };
    gate_of(&page)
        .trust_certificate(&h.config.integrations[0], &cert, TlsMode::Strict)
        .unwrap();
    page.select_kind(GITLAB_SM, &h.config);
    let screen = text(&draw(&mut page, &h, 100, 30, None));
    assert!(screen.contains("Pinned"), "{screen}");
    assert!(screen.contains("(strict)"), "{screen}");
    assert!(screen.contains("ab12 cd34 ab12 cd34"), "{screen}");
    // The form's toggle starts from the record.
    page.begin_form(&h.config, Some(0));
    assert!(page.strict);
    let _ = id;
}

#[test]
fn disconnect_and_remove_forget_what_was_trusted() {
    let mut h = H::new();
    let id = h.connected(IntegrationKind::Forgejo, "kai");
    let mut page = IntegrationsPage::default();
    gate_of(&page)
        .record_after_check(
            &h.config.integrations[0],
            &Account {
                login: "kai".into(),
                id: "7".into(),
            },
        )
        .unwrap();
    assert!(gate_of(&page).record_of(&id).is_some());
    let kind = page.kind_index(IntegrationKind::Forgejo).unwrap();
    page.select_kind(kind, &h.config);
    page.activate(Item::Disconnect, &mut h.env());
    assert!(gate_of(&page).record_of(&id).is_none(), "disconnect forgets");
}
