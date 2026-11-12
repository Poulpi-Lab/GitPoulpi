use super::*;

fn select(page: &mut IntegrationsPage, h: &H, kind: IntegrationKind) {
    let at = page.kind_index(kind).expect("kind is listed");
    page.select_kind(at, &h.config);
    page.add_profile(&h.config);
}

/// Hosting first, then issue trackers, each category one contiguous run
/// (the renderer starts a new heading at every category change).
#[test]
fn page_kinds_are_grouped_by_category() {
    use gitpoulpi_config::IntegrationCategory as C;
    let cats: Vec<C> = page_kinds().iter().map(|k| k.category()).collect();
    let first_tracker = cats.iter().position(|c| *c == C::IssueTracker);
    let last_hosting = cats.iter().rposition(|c| *c == C::Hosting);
    if let (Some(t), Some(h)) = (first_tracker, last_hosting) {
        assert!(h < t, "a Hosting kind follows an issue tracker: {cats:?}");
    }
    assert!(page_kinds().contains(&IntegrationKind::Gerrit));
}

#[test]
fn the_list_has_both_groups_in_order() {
    let mut page = IntegrationsPage::default();
    let h = H::new();
    let screen = text(&draw(&mut page, &h, 100, 34, None));
    let at = |s: &str| {
        screen
            .find(s)
            .unwrap_or_else(|| panic!("missing {s}:\n{screen}"))
    };
    assert!(at("Hosting") < at("Azure DevOps Server"));
    assert!(at("Azure DevOps Server") < at("Bitbucket"));
    assert!(at("Bitbucket") < at("Gitea"));
    assert!(at("Gitea") < at("Forgejo"));
    assert!(at("Forgejo") < at("Issue trackers"));
    assert!(at("Issue trackers") < at("Jira Cloud"));
    assert!(at("Jira Data Center") < at("Trello"));
}

#[test]
fn the_azure_form_asks_for_an_organization_and_links_to_its_token_page() {
    let h = H::new();
    let mut page = IntegrationsPage::default();
    select(&mut page, &h, IntegrationKind::AzureDevops);
    let screen = text(&draw(&mut page, &h, 100, 36, None));
    for want in [
        "Organization",
        "Code (Read & write)",
        "Personal access token",
    ] {
        assert!(screen.contains(want), "missing {want:?}:\n{screen}");
    }
    assert!(!screen.contains("Host domain"), "{screen}");
    assert_eq!(page.token_url(), None, "no organization yet");
    page.org = "contoso".into();
    assert_eq!(
        page.token_url().as_deref(),
        Some("https://dev.azure.com/contoso/_usersSettings/tokens")
    );
    page.org = "https://contoso.visualstudio.com/".into();
    assert_eq!(
        page.form_url().as_deref(),
        Some("https://dev.azure.com/contoso")
    );
    let screen = text(&draw(&mut page, &h, 100, 36, None));
    assert!(
        screen.contains("Generate a token on dev.azure.com"),
        "{screen}"
    );
}

#[test]
fn trello_has_two_links_and_the_token_one_waits_for_the_key() {
    let h = H::new();
    let mut page = IntegrationsPage::default();
    select(&mut page, &h, IntegrationKind::Trello);
    let screen = text(&draw(&mut page, &h, 100, 36, None));
    assert!(screen.contains(" Get an API key on Trello "), "{screen}");
    assert!(
        screen.contains(" Generate a token (enter the API key first) "),
        "{screen}"
    );
    assert_eq!(page.token_url(), None);
    page.account = "k3y".into();
    let url = page.token_url().unwrap();
    assert!(
        url.contains("trello.com/1/authorize") && url.contains("key=k3y"),
        "{url}"
    );
    let screen = text(&draw(&mut page, &h, 100, 36, None));
    assert!(
        screen.contains(" Generate a token on trello.com "),
        "{screen}"
    );
}

#[test]
fn jira_cloud_needs_a_site_an_email_and_a_token_and_fits_a_narrow_page() {
    let h = H::new();
    let mut page = IntegrationsPage::default();
    select(&mut page, &h, IntegrationKind::JiraCloud);
    let screen = text(&draw(&mut page, &h, 60, 40, None));
    for want in ["Site URL", "Atlassian account email"] {
        assert!(screen.contains(want), "missing {want:?}:\n{screen}");
    }
    assert!(!page.can_connect(&h.config, &h.presence));
    page.host = "acme.atlassian.net".into();
    page.account = "no-at-sign".into();
    page.token = "t".into();
    assert!(!page.can_connect(&h.config, &h.presence));
    page.account = "me@acme.com".into();
    assert!(page.can_connect(&h.config, &h.presence));
    // The Atlassian token page needs no host.
    page.host.clear();
    assert!(page
        .token_url()
        .unwrap()
        .starts_with("https://id.atlassian.com/"));
}

#[test]
fn a_jira_cloud_connect_keeps_the_account_on_the_profile() {
    let mut h = H::new();
    let (mut page, t) = page_with(RecordedTransport::new().on(
        "/rest/api/3/myself",
        200,
        r#"{"accountId":"5b10","displayName":"Me","emailAddress":"me@acme.com"}"#,
    ));
    select(&mut page, &h, IntegrationKind::JiraCloud);
    page.host = "acme.atlassian.net".into();
    page.account = " me@acme.com ".into();
    page.token = "tok".into();
    page.activate(Item::Connect, &mut h.env());
    wait_connected(&mut page, &mut h);
    assert_eq!(h.config.integrations.len(), 1);
    assert_eq!(h.config.integrations[0].account, "me@acme.com");
    assert_eq!(t.requests().len(), 1);
    let saved = std::fs::read_to_string(&h.path).unwrap();
    assert!(saved.contains("account = \"me@acme.com\""), "{saved}");
    assert!(!saved.contains("tok\""), "{saved}");
}

#[cfg(feature = "gitea")]
#[test]
fn forgejo_starts_on_codeberg_and_gitea_asks_for_a_host() {
    let h = H::new();
    let mut page = IntegrationsPage::default();
    select(&mut page, &h, IntegrationKind::Forgejo);
    assert_eq!(
        page.host, "https://codeberg.org",
        "prefilled, still editable"
    );
    assert_eq!(
        page.token_url().as_deref(),
        Some("https://codeberg.org/user/settings/applications")
    );
    let screen = text(&draw(&mut page, &h, 100, 36, None));
    assert!(screen.contains("codeberg.org"), "{screen}");
    page.host = "forgejo.example.com".into();
    assert_eq!(
        page.form_url().as_deref(),
        Some("https://forgejo.example.com")
    );

    let mut page = IntegrationsPage::default();
    select(&mut page, &h, IntegrationKind::Gitea);
    assert_eq!(page.host, "", "no public instance to start on");
    assert_eq!(page.token_url(), None);
    page.host = "gitea.example.com".into();
    assert_eq!(
        page.token_url().as_deref(),
        Some("https://gitea.example.com/user/settings/applications")
    );
}

#[cfg(feature = "gitea")]
#[test]
fn a_gitea_connect_checks_the_token_with_the_token_scheme_and_saves_the_profile() {
    let mut h = H::new();
    let (mut page, t) = page_with(RecordedTransport::new().on(
        "/api/v1/user",
        200,
        r#"{"id":1,"login":"poulpi","full_name":"Poulpi"}"#,
    ));
    select(&mut page, &h, IntegrationKind::Gitea);
    page.host = "git.example.com".into();
    page.token = "tok".into();
    page.activate(Item::Connect, &mut h.env());
    wait_connected(&mut page, &mut h);
    assert_eq!(h.config.integrations.len(), 1);
    let profile = &h.config.integrations[0];
    assert_eq!(profile.kind, IntegrationKind::Gitea);
    assert_eq!(profile.url, "https://git.example.com");
    assert_eq!(profile.user.as_ref().unwrap().login, "poulpi");
    let request = &t.requests()[0];
    assert_eq!(request.url, "https://git.example.com/api/v1/user");
    assert!(request
        .headers
        .iter()
        .any(|(k, v)| k == "Authorization" && v == "token tok"));
}

#[test]
fn a_local_markdown_folder_has_a_folder_field_a_browse_button_and_no_token() {
    let mut h = H::new();
    let mut page = IntegrationsPage::default();
    select(&mut page, &h, IntegrationKind::LocalMarkdown);
    let screen = text(&draw(&mut page, &h, 100, 36, None));
    for want in ["Notes folder", " Browse... ", " Add folder "] {
        assert!(screen.contains(want), "missing {want:?}:\n{screen}");
    }
    for gone in ["Personal access token", "Generate a token", "Host domain"] {
        assert!(!screen.contains(gone), "{gone:?} in:\n{screen}");
    }
    // Nothing typed: the folder is the missing input.
    assert!(page.missing_input().is_some());

    // Browse opens the folder picker; choosing a folder fills the field.
    page.activate(Item::FolderBrowse, &mut h.env());
    assert!(page.ssh.picker.is_some() && page.ssh.picking_folder);
    page.picker_outcome(PickerOutcome::Choose(PathBuf::from("/tmp")));
    assert_eq!(page.host, "/tmp");
    assert!(page.ssh.picker.is_none() && !page.ssh.picking_folder);

    // A relative folder is kept as typed; there is no token to check.
    page.host = "docs/issues".into();
    assert!(page.missing_input().is_none());
    page.activate(Item::Connect, &mut h.env());
    for _ in 0..200 {
        page.poll(&mut h.env());
        if page.pending.is_none() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    let saved = &h.config.integrations[0];
    assert_eq!(saved.kind, IntegrationKind::LocalMarkdown);
    assert_eq!(saved.url, "docs/issues");
    assert_eq!(page.view(&h.config), View::Connected(0));
    let screen = text(&draw(&mut page, &h, 100, 36, None));
    assert!(
        screen.contains("docs/issues") && screen.contains(" Edit "),
        "{screen}"
    );
    assert!(!screen.contains("Disconnect"), "{screen}");

    // An absolute folder that is not there is refused with its name.
    let mut page = IntegrationsPage::default();
    select(&mut page, &h, IntegrationKind::LocalMarkdown);
    // Absolute on every platform: "/x" has no drive letter on Windows.
    let missing = std::env::temp_dir()
        .join("gitpoulpi-definitely-not-a-folder")
        .display()
        .to_string();
    page.host = missing.clone();
    page.activate(Item::Connect, &mut h.env());
    assert!(
        page.error.as_deref().is_some_and(|e| e.contains(&missing)),
        "{:?}",
        page.error
    );
    assert_eq!(h.config.integrations.len(), 1);
}
