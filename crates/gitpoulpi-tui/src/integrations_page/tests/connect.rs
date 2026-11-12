use super::*;
use crate::th;

#[test]
fn generate_link_opens_the_hosts_prefilled_token_page() {
    let mut h = H::new();
    let opened = Arc::new(Mutex::new(Vec::<String>::new()));
    let sink = opened.clone();
    let mut page = IntegrationsPage {
        opener: Box::new(move |url| {
            sink.lock().unwrap().push(url.to_string());
            Ok(())
        }),
        ..Default::default()
    };
    page.select_kind(GITLAB_SM, &h.config);
    page.add_profile(&h.config);
    // No host yet: nothing opens.
    page.activate(Item::Generate, &mut h.env());
    assert!(opened.lock().unwrap().is_empty());
    type_str(&mut page, &mut h, "gitlab.example.com");
    page.activate(Item::Generate, &mut h.env());
    let urls = opened.lock().unwrap().clone();
    assert_eq!(
        urls,
        ["https://gitlab.example.com/-/user_settings/personal_access_tokens?name=GitPoulpi&scopes=api,read_user"]
    );
}

#[test]
fn connecting_checks_the_token_then_stores_it_and_saves_the_profile() {
    let mut h = H::new();
    let (mut page, transport) = page_with(gitlab_ok());
    fill_sm_form(&mut page, &mut h, "gitlab.example.com", "glpat-GOOD");
    page.activate(Item::Connect, &mut h.env());
    assert_eq!(page.mode, Mode::Connecting(None));
    let screen = text(&draw(&mut page, &h, 80, 26, None));
    assert!(
        screen.contains("Connecting to gitlab.example.com"),
        "{screen}"
    );
    wait_connected(&mut page, &mut h);

    let saved = &h.config.integrations[0];
    assert_eq!(saved.kind, IntegrationKind::GitlabSelfManaged);
    assert_eq!(saved.url, "https://gitlab.example.com");
    assert_eq!(saved.user.as_ref().unwrap().login, "kai");
    assert_eq!(
        h.tokens.load(&saved.token_ref).unwrap().as_deref(),
        Some("glpat-GOOD")
    );
    let raw = std::fs::read_to_string(&h.path).unwrap();
    assert!(raw.contains("token_ref"), "{raw}");
    assert!(!raw.contains("glpat-GOOD"), "{raw}");
    assert!(!transport.requests().is_empty());

    let screen = text(&draw(&mut page, &h, 80, 26, None));
    assert_eq!(page.view(&h.config), View::Connected(0));
    assert!(page.pending.is_none());
    for gone in [
        "Host domain",
        "[ Connect ]",
        "[ Cancel ]",
        "Scopes:",
        "Connecting to",
    ] {
        assert!(
            !screen.contains(gone),
            "{gone:?} still shown:
{screen}"
        );
    }
    assert!(
        page.items(&h.config).iter().all(|i| !matches!(
            i,
            Item::Token | Item::Host | Item::Connect | Item::CancelForm
        )),
        "form items still focusable"
    );
    for want in [
        "Kai G (@kai)",
        "Connected",
        "stored",
        "[ Edit ]",
        "[ Disconnect ]",
        "[ Remove ]",
        "SSH",
    ] {
        assert!(screen.contains(want), "missing {want:?}:\n{screen}");
    }
    assert!(!screen.contains("glpat"), "{screen}");
}

#[test]
fn a_rejected_token_returns_to_the_form_with_the_reason_and_stores_nothing() {
    let mut h = H::new();
    let (mut page, _) = page_with(RecordedTransport::new().on("/api/v4/user", 401, "{}"));
    fill_sm_form(&mut page, &mut h, "gitlab.example.com", "glpat-BAD");
    page.activate(Item::Connect, &mut h.env());
    wait_connected(&mut page, &mut h);
    assert!(h.config.integrations.is_empty());
    assert!(matches!(page.mode, Mode::Form(None)));
    assert!(h
        .tokens
        .load("integration:gitlab-self-managed")
        .unwrap()
        .is_none());
    let screen = text(&draw(&mut page, &h, 80, 30, None));
    assert!(screen.contains("rejected"), "{screen}");
}

#[test]
fn missing_scopes_are_listed_in_red() {
    let mut h = H::new();
    let (mut page, _) = page_with(
        RecordedTransport::new()
            .on(
                "/personal_access_tokens/self",
                200,
                r#"{"scopes":["read_user"],"expires_at":null}"#,
            )
            .on(
                "/api/v4/user",
                200,
                r#"{"id":1,"username":"kai","name":""}"#,
            ),
    );
    fill_sm_form(&mut page, &mut h, "gitlab.example.com", "glpat-x");
    page.activate(Item::Connect, &mut h.env());
    wait_connected(&mut page, &mut h);
    let buf = draw(&mut page, &h, 80, 26, None);
    let screen = text(&buf);
    assert!(screen.contains("Token lacks: api"), "{screen}");
    let row = screen
        .lines()
        .position(|l| l.contains("Token lacks"))
        .unwrap() as u16;
    let col = screen
        .lines()
        .nth(row as usize)
        .unwrap()
        .find("Token lacks")
        .unwrap();
    // `find` is a byte offset; the row's earlier cells are ASCII or
    // single-width, so it is the column here.
    let col = screen.lines().nth(row as usize).unwrap()[..col]
        .chars()
        .count() as u16;
    assert_eq!(buf[(col, row)].fg, th().status.danger);
}

#[test]
fn closing_preferences_mid_check_still_saves_the_profile() {
    let mut h = H::new();
    let (mut page, _transport) = page_with(gitlab_ok());
    fill_sm_form(&mut page, &mut h, "gitlab.example.com", "glpat-GOOD");
    page.activate(Item::Connect, &mut h.env());
    assert!(page.pending.is_some());
    // Preferences closes: the page is told, then dropped.
    page.detach_pending();
    drop(page);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        let saved = Config::load(&h.path)
            .map(|c| c.integrations)
            .unwrap_or_default();
        if let Some(profile) = saved.first() {
            assert_eq!(profile.user.as_ref().unwrap().login, "kai");
            assert!(profile.enabled);
            break;
        }
        assert!(std::time::Instant::now() < deadline, "profile never saved");
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}

#[test]
fn canceling_a_check_saves_nothing() {
    let mut h = H::new();
    let (mut page, _transport) = page_with(gitlab_ok());
    fill_sm_form(&mut page, &mut h, "gitlab.example.com", "glpat-GOOD");
    page.activate(Item::Connect, &mut h.env());
    page.activate(Item::CancelConnect, &mut h.env());
    drop(page);
    std::thread::sleep(std::time::Duration::from_millis(200));
    assert!(Config::load(&h.path).unwrap().integrations.is_empty());
}

#[test]
fn a_legacy_profile_with_a_stored_token_connects_without_retyping_it() {
    let mut h = H::new();
    let legacy = Integration::new(
        "corp",
        IntegrationKind::GitlabSelfManaged,
        "https://gitlab.example.com",
    );
    h.tokens.store(&legacy.token_ref, "glpat-OLD").unwrap();
    h.presence.insert(legacy.token_ref.clone(), Some(true));
    h.config.integrations.push(legacy);
    let (mut page, _) = page_with(gitlab_ok());
    page.select_kind(GITLAB_SM, &h.config);
    // Unverified profile: opens straight into its form, host prefilled.
    assert_eq!(page.view(&h.config), View::Form);
    assert!(page.can_connect(&h.config, &h.presence));
    page.activate(Item::Connect, &mut h.env());
    wait_connected(&mut page, &mut h);
    assert_eq!(h.config.integrations.len(), 1);
    assert_eq!(h.config.integrations[0].user.as_ref().unwrap().login, "kai");
    assert_eq!(
        h.tokens.load("integration:corp").unwrap().as_deref(),
        Some("glpat-OLD")
    );
}

#[test]
fn hosts_are_normalized_or_refused() {
    assert_eq!(
        normalize_host("gitlab.example.com").as_deref(),
        Some("https://gitlab.example.com")
    );
    assert_eq!(
        normalize_host(" https://gitlab.example.com/ ").as_deref(),
        Some("https://gitlab.example.com")
    );
    assert_eq!(
        normalize_host("http://git.corp:8080/gl").as_deref(),
        Some("http://git.corp:8080/gl")
    );
    assert_eq!(
        normalize_host("localhost:3000").as_deref(),
        Some("https://localhost:3000")
    );
    for bad in [
        "",
        "gitlab",
        "https://gitlab",
        "ftp://x.y",
        "a b.com",
        "me@x.com",
        "https://",
        "gitea:",
        "gitea:abc",
        "gitea:123456789",
        "mailto:x",
        ".:80",
        "[::1",
        "[]:80",
        "[zz]:80",
        "http://",
        "http://:80",
    ] {
        assert_eq!(normalize_host(bad), None, "{bad:?}");
    }
}

/// A self-hosted server in a Docker container or on the LAN is often a
/// dotless name: it needs a port or an explicit `http://`.
#[test]
fn plain_docker_and_lan_hosts_are_accepted() {
    for (typed, want) in [
        ("gitea:3000", "https://gitea:3000"),
        ("http://nas:8080", "http://nas:8080"),
        ("http://nas", "http://nas"),
        ("192.168.1.10:3000", "https://192.168.1.10:3000"),
        ("http://192.168.1.10:3000/", "http://192.168.1.10:3000"),
        ("[::1]:3000", "https://[::1]:3000"),
        ("http://[::1]:3000", "http://[::1]:3000"),
        ("http://localhost:8080", "http://localhost:8080"),
        ("git_lab-1:80/gl", "https://git_lab-1:80/gl"),
    ] {
        assert_eq!(normalize_host(typed).as_deref(), Some(want), "{typed:?}");
    }
}

/// A token the OS credential store refused is held for the session only:
/// the confirmation says so, with the store's error, instead of a bare
/// "connected". The fake store never touches the real one.
#[test]
fn connecting_says_when_the_token_is_kept_for_this_session_only() {
    struct SessionOnly(InMemoryTokens);
    impl TokenStore for SessionOnly {
        fn store(&self, r: &str, t: &str) -> gitpoulpi_secrets::Result<()> {
            self.0.store(r, t)
        }
        fn load(&self, r: &str) -> gitpoulpi_secrets::Result<Option<String>> {
            self.0.load(r)
        }
        fn delete(&self, r: &str) -> gitpoulpi_secrets::Result<()> {
            self.0.delete(r)
        }
        fn session_only_reason(&self, _: &str) -> Option<String> {
            Some("no helper".into())
        }
    }
    let mut h = H::new();
    h.tokens = Arc::new(SessionOnly(InMemoryTokens::default()));
    let (mut page, _transport) = page_with(gitlab_ok());
    fill_sm_form(&mut page, &mut h, "gitlab.example.com", "glpat-GOOD");
    page.activate(Item::Connect, &mut h.env());
    wait_connected(&mut page, &mut h);
    let said = h.message.clone().unwrap_or_default();
    assert!(said.contains("connected to"), "{said}");
    assert!(said.contains("this session only"), "{said}");
    assert!(said.contains("no helper"), "{said}");
}
