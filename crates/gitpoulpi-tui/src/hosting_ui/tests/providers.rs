//! Azure DevOps, Bitbucket, Jira and Trello in the sidebar (more-providers S10):
//! names and chip tags, how a request is checked out, branch names from an
//! issue, and the rows and query row of the ISSUES section.

use super::*;

fn with_kind(mut snap: HostingSnapshot, kind: IntegrationKind) -> HostingSnapshot {
    snap.kind = Some(kind);
    let profile = ProfileRef {
        id: "corp".into(),
        label: "corp".into(),
        kind,
    };
    snap.pulls_target.as_mut().unwrap().profile = profile.clone();
    snap.issues_target.as_mut().unwrap().profile = profile;
    snap
}

fn menu_labels(ui: &HostingUi) -> Vec<String> {
    let menu = ui.menu.as_ref().expect("a menu");
    (0..menu.row_count()).map(|r| menu.row_label(r)).collect()
}

#[test]
fn every_provider_has_a_name_and_a_chip_tag() {
    let cases = [
        (IntegrationKind::Github, "GitHub", "GH"),
        (IntegrationKind::GitlabSelfManaged, "GitLab", "GL"),
        (IntegrationKind::AzureDevops, "Azure DevOps", "AZ"),
        (IntegrationKind::AzureDevopsServer, "Azure DevOps", "AZ"),
        (IntegrationKind::Bitbucket, "Bitbucket", "BB"),
        (IntegrationKind::BitbucketDataCenter, "Bitbucket", "BB"),
        (IntegrationKind::JiraCloud, "Jira", "JI"),
        (IntegrationKind::JiraDataCenter, "Jira", "JI"),
        (IntegrationKind::Trello, "Trello", "TR"),
        (IntegrationKind::Gitea, "Gitea", "GT"),
        (IntegrationKind::Forgejo, "Forgejo", "GT"),
    ];
    for (kind, name, tag) in cases {
        assert_eq!(provider_name(kind), name);
        assert_eq!(provider_tag(Some(kind)), tag);
    }
}

#[test]
fn the_chip_of_an_azure_request_reads_az_bang_12() {
    let mut p = open_pr(12);
    p.iid_label = "!12".into();
    let (ui, _d) = ui_with(with_kind(
        snapshot(vec![p], vec![]),
        IntegrationKind::AzureDevops,
    ));
    assert_eq!(ui.chip().unwrap().text, "AZ !12");
}

#[test]
fn an_azure_request_is_fetched_by_its_branch_into_pr_n() {
    let (mut ui, _d) = ui_with(with_kind(
        snapshot(vec![open_pr(12)], vec![]),
        IntegrationKind::AzureDevopsServer,
    ));
    let pull = first_pull(&ui);
    ui.open_row_menu(HostingList::Pulls, pull, 3, 3);
    let labels = menu_labels(&ui);
    assert!(
        labels.iter().any(|l| l.contains("as a local branch")),
        "{labels:?}"
    );
    let mut clip = crate::clipboard::Clipboard::new();
    match ui.run(
        HostingAction::CheckoutPullRequestHead { number: 12 },
        &mut clip,
    ) {
        HostingEffect::CheckoutPullRequestHead {
            remote_ref, branch, ..
        } => {
            assert_eq!(remote_ref, "refs/heads/feature");
            assert_eq!(branch, "pr/12");
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn an_azure_or_bitbucket_fork_request_is_refused_with_the_reason() {
    for kind in [IntegrationKind::AzureDevops, IntegrationKind::Bitbucket] {
        let mut fork = open_pr(7);
        fork.source_project = Some("someone/p".into());
        fork.web_url = "https://bitbucket.org/ws/p/pull-requests/7".into();
        let (mut ui, _d) = ui_with(with_kind(snapshot(vec![fork], vec![]), kind));
        let pull = first_pull(&ui);
        ui.open_row_menu(HostingList::Pulls, pull, 3, 3);
        let labels = menu_labels(&ui);
        assert!(
            labels
                .iter()
                .any(|l| l.contains("add it as a remote first")),
            "{kind:?}: {labels:?}"
        );
        let mut clip = crate::clipboard::Clipboard::new();
        match ui.run(
            HostingAction::CheckoutPullRequestHead { number: 7 },
            &mut clip,
        ) {
            HostingEffect::Message(m) => {
                assert!(m.contains("someone/p") && m.contains("feature"), "{m}")
            }
            other => panic!("{kind:?}: {other:?}"),
        }
    }
}

#[test]
fn a_bitbucket_data_center_request_keeps_its_head_ref() {
    let mut dc = open_pr(5);
    dc.web_url = "https://bb.corp/projects/KEY/repos/slug/pull-requests/5".into();
    dc.source_project = Some("KEY/fork".into());
    let (mut ui, _d) = ui_with(with_kind(
        snapshot(vec![dc], vec![]),
        IntegrationKind::BitbucketDataCenter,
    ));
    let mut clip = crate::clipboard::Clipboard::new();
    match ui.run(
        HostingAction::CheckoutPullRequestHead { number: 5 },
        &mut clip,
    ) {
        HostingEffect::CheckoutPullRequestHead {
            remote_ref, branch, ..
        } => {
            assert_eq!(remote_ref, "refs/pull-requests/5/from");
            assert_eq!(branch, "pr/5");
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn branch_from_issue_names_follow_the_provider() {
    let mut i = issue(123);
    i.title = "Fix login".into();
    i.reference = "ABC-123".into();
    assert_eq!(
        issue_branch_for(Some(gitpoulpi_config::ApiFamily::Jira), &i),
        "ABC-123-fix-login"
    );
    for family in [
        gitpoulpi_config::ApiFamily::AzureDevops,
        gitpoulpi_config::ApiFamily::Github,
        gitpoulpi_config::ApiFamily::Trello,
    ] {
        assert_eq!(issue_branch_for(Some(family), &i), "123-fix-login");
    }
    i.title = "  ".into();
    assert_eq!(
        issue_branch_for(Some(gitpoulpi_config::ApiFamily::Jira), &i),
        "ABC-123"
    );
}

#[test]
fn linear_and_youtrack_branches_come_from_their_key_or_linears_own_name() {
    let mut i = issue(12);
    i.title = "Fix login".into();
    i.reference = "ENG-12".into();
    for family in [
        gitpoulpi_config::ApiFamily::Linear,
        gitpoulpi_config::ApiFamily::YouTrack,
    ] {
        assert_eq!(issue_branch_for(Some(family), &i), "ENG-12-fix-login");
    }
    i.branch_name = Some("kai/eng-12-fix-login".into());
    assert_eq!(
        issue_branch_for(Some(gitpoulpi_config::ApiFamily::Linear), &i),
        "kai/eng-12-fix-login",
        "Linear's branchName wins"
    );
}

#[test]
fn only_jira_azure_trello_and_youtrack_issue_sources_get_a_query_row() {
    for (kind, query) in [
        (IntegrationKind::JiraCloud, true),
        (IntegrationKind::AzureDevops, true),
        (IntegrationKind::Trello, true),
        (IntegrationKind::Youtrack, true),
        (IntegrationKind::Linear, false),
        (IntegrationKind::GitlabSelfManaged, false),
    ] {
        let (ui, _d) = ui_with(with_kind(snapshot(vec![], vec![issue(1)]), kind));
        assert_eq!(
            ui.rows(HostingList::Issues)
                .contains(&HostingRow::IssueQuery),
            query,
            "{kind:?}"
        );
    }
}

#[test]
fn the_query_row_edits_the_jql_and_saves_it_on_enter_or_on_leaving() {
    let (mut ui, d) = ui_with(with_kind(
        snapshot(vec![], vec![issue(1)]),
        IntegrationKind::JiraCloud,
    ));
    ui.issues_source = Some(gitpoulpi_session::IssuesSource {
        integration: "corp".into(),
        project: "g/p".into(),
        filter: None,
    });
    let row = ui
        .rows(HostingList::Issues)
        .iter()
        .position(|r| *r == HostingRow::IssueQuery)
        .unwrap();
    let mut clip = crate::clipboard::Clipboard::new();
    ui.activate_row(HostingList::Issues, row, &mut clip);
    assert!(ui.editing_query && ui.editing == Some(HostingList::Issues));
    for c in "component = UI".chars() {
        assert!(ui.on_search_key(key(KeyCode::Char(c))));
    }
    ui.on_search_key(key(KeyCode::Enter));
    assert!(!ui.editing_query && ui.editing.is_none());
    let saved =
        gitpoulpi_session::RepoState::load(&crate::remotes_ui::repo_state_path(d.path())).unwrap();
    assert_eq!(
        saved.issues_source.unwrap().filter.as_deref(),
        Some("component = UI")
    );
    assert_eq!(ui.saved_query(), "component = UI");

    // Esc drops what was typed; a click elsewhere keeps it.
    ui.activate_row(HostingList::Issues, row, &mut clip);
    ui.on_search_key(key(KeyCode::Char('x')));
    ui.on_search_key(key(KeyCode::Esc));
    assert_eq!(ui.saved_query(), "component = UI");
    ui.activate_row(HostingList::Issues, row, &mut clip);
    ui.on_search_key(key(KeyCode::Char('!')));
    ui.editing = None;
    ui.tick(Instant::now());
    assert_eq!(ui.saved_query(), "component = UI!");
}

#[test]
fn trello_lists_toggle_in_and_out_of_the_saved_query() {
    use super::super::picker::{chosen_lists, toggle_list};
    assert_eq!(toggle_list("", "Todo").as_deref(), Some("Todo"));
    assert_eq!(toggle_list("Todo", "Doing").as_deref(), Some("Todo, Doing"));
    assert_eq!(toggle_list("Todo, Doing", "todo").as_deref(), Some("Doing"));
    assert_eq!(toggle_list("Doing", "doing"), None);
    assert_eq!(toggle_list("Todo, Doing", ""), None, "All lists clears");
    assert_eq!(chosen_lists("Todo, ,Doing"), vec!["Todo", "Doing"]);
}

#[test]
fn issue_rows_carry_the_providers_dim_words_when_the_sidebar_has_room() {
    let mut i = issue(12);
    i.iid_label = "ABC-12".into();
    i.kind_label = Some("Bug".into());
    i.status_label = Some("To Do".into());
    let (ui, _d) = ui_with(with_kind(
        snapshot(vec![], vec![i]),
        IntegrationKind::JiraCloud,
    ));
    let (wide, ..) = draw_section(&ui, HostingList::Issues, 70, 12, None);
    assert!(wide.contains("Bug \u{00B7} To Do"), "{wide}");
    let (narrow, ..) = draw_section(&ui, HostingList::Issues, 30, 12, None);
    assert!(
        !narrow.contains("To Do"),
        "the title keeps the room:\n{narrow}"
    );
}

#[test]
fn gerrit_has_a_name_and_a_chip_tag() {
    assert_eq!(provider_name(IntegrationKind::Gerrit), "Gerrit");
    assert_eq!(provider_tag(Some(IntegrationKind::Gerrit)), "GR");
    assert_eq!(provider_name(IntegrationKind::Redmine), "Redmine");
    assert_eq!(provider_tag(Some(IntegrationKind::Redmine)), "RM");
}

fn gerrit_ui() -> (HostingUi, tempfile::TempDir) {
    let mut a = open_pr(7);
    a.target_branch = "master".into();
    let mut b = open_pr(8);
    b.target_branch = "stable".into();
    let mut c = open_pr(9);
    c.target_branch = "stable".into();
    ui_with(with_kind(
        snapshot(vec![a, b, c], vec![]),
        IntegrationKind::Gerrit,
    ))
}

#[test]
fn on_gerrit_the_create_button_and_the_palette_row_offer_a_push_for_review() {
    let (mut ui, _d) = gerrit_ui();
    let mut clip = crate::clipboard::Clipboard::new();
    assert!(ui
        .command_unavailable(HostingCommand::PushForReview)
        .is_none());
    // The header's `+` (Create Pull Request) is the same flow: a menu of the
    // branches the listed changes target, the most used first.
    for cmd in [
        HostingCommand::CreatePullRequest,
        HostingCommand::PushForReview,
    ] {
        ui.menu = None;
        assert_eq!(ui.run_command(cmd, (3, 3), &mut clip), HostingEffect::None);
        assert_eq!(menu_labels(&ui), ["stable", "master"], "{cmd:?}");
    }
    // The pick pushes to that branch's review ref on the snapshot's remote.
    let effect = ui.run(
        HostingAction::PushForReview {
            branch: "stable".into(),
        },
        &mut clip,
    );
    assert_eq!(
        effect,
        HostingEffect::PushForReview {
            remote: "origin".into(),
            branch: "stable".into()
        }
    );
}

#[test]
fn outside_gerrit_the_push_for_review_row_explains_itself() {
    let (mut ui, _d) = ui_with(with_kind(
        snapshot(vec![open_pr(12)], vec![]),
        IntegrationKind::GitlabSelfManaged,
    ));
    let why = ui
        .command_unavailable(HostingCommand::PushForReview)
        .expect("not on GitLab");
    assert!(why.contains("Gerrit"), "{why}");
    let mut clip = crate::clipboard::Clipboard::new();
    assert_eq!(
        ui.run_command(HostingCommand::CreatePullRequest, (3, 3), &mut clip),
        HostingEffect::CreatePullRequest,
        "GitLab keeps its form"
    );
}
