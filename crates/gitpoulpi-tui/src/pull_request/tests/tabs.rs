use super::*;

#[test]
fn only_supported_providers_get_a_tab() {
    // Bitbucket has a client and a tab; Jira is an issue tracker and never
    // gets one.
    let kinds = all_kinds();
    let tabs = pr_tabs(&kinds);
    let labels: Vec<&str> = tabs.iter().map(|t| t.label.as_str()).collect();
    assert_eq!(labels, vec!["GitHub", "GitLab", "Bitbucket"], "no Jira");
    // Two of a kind are told apart by name.
    let mut two = kinds.clone();
    two.push(integration(
        "corp2",
        IntegrationKind::GitlabSelfManaged,
        "https://git.other.example",
    ));
    let labels: Vec<String> = pr_tabs(&two).into_iter().map(|t| t.label).collect();
    assert_eq!(labels[0], "GitHub");
    assert!(labels[1].starts_with("GitLab \u{00B7} ") && labels[3].contains("other"));
    assert!(build_seed(
        "x",
        &[],
        &[],
        &[integration(
            "jira",
            IntegrationKind::JiraCloud,
            "https://x.atlassian.net"
        )]
    )
    .unwrap_err()
    .contains("connect an integration first"));
}

#[test]
fn switching_tabs_reloads_the_repositories_of_that_provider() {
    let mut factory = FakeFactory::new();
    factory.projects = vec![project("me/one"), project("me/two")];
    let seed = build_seed("feature", &seed_refs(), &remotes(), &all_kinds()).unwrap();
    let mut d = PullRequestDialog::with_factory(seed, "x", std::sync::Arc::new(factory));
    // The home tab starts with the remote's repository.
    assert_eq!(d.tab, 1);
    // Left/Right on the tab strip switch the provider.
    d.focus = PrFocus::Tabs;
    d.on_key(key(KeyCode::Left));
    assert_eq!(d.tab, 0);
    assert_eq!(
        d.from_repo.selected(),
        None,
        "another provider starts empty"
    );
    assert_eq!(d.plan().unwrap_err(), "choose the repository to merge into");
    let deadline = Instant::now() + std::time::Duration::from_secs(10);
    while d.projects_rx.iter().any(|_| true) {
        d.poll();
        assert!(Instant::now() < deadline, "the project list never landed");
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    let names: Vec<String> = d.to_repo.options.iter().map(|o| o.label.clone()).collect();
    assert_eq!(names, vec!["me/one", "me/two"]);
    assert_eq!(
        d.seed.tabs[d.tab].integration.id, "hub",
        "the tab strip follows the provider"
    );
}

#[test]
fn create_from_the_header_opens_over_the_supported_providers_whatever_the_branch_state() {
    let mut seed = None;
    // Checked out and pushed.
    let msg = create_pull_request_now(&seed_refs(), &remotes(), &mut seed);
    // No configuration is read in this test: an empty list of tabs is
    // a clear message, not a form over nothing.
    if seed.is_none() {
        assert!(msg.contains("connect an integration first"), "{msg}");
    }
    // With profiles, a never-pushed branch still opens a form, saying
    // what is missing.
    let mut unpushed = local_branch_at("wip", oid(9));
    unpushed.is_head = true;
    let seed = build_seed("wip", &[unpushed], &remotes(), &all_kinds()).unwrap();
    assert_eq!(seed.source_branch, None);
    let d = PullRequestDialog::with_factory(seed, "", std::sync::Arc::new(FakeFactory::new()));
    assert_eq!(d.title, "wip");
    assert!(d.plan().unwrap_err().contains("branch to merge from"));
}
