//! Row menus, the effects the loop acts on, and the palette commands.

use super::*;

#[test]
fn the_new_issue_command_needs_a_linked_source_and_seeds_the_form_from_it() {
    let (mut ui, _d) = ui_with(snapshot(vec![], vec![]));
    let mut clip = crate::clipboard::Clipboard::new();
    assert!(ui.issue_seed().is_none());
    assert!(matches!(
        ui.run_command(HostingCommand::CreateIssue, (0, 0), &mut clip),
        HostingEffect::Message(_)
    ));
    let none = HostingAvailability::default();
    assert!(command_unavailable_from(HostingCommand::CreateIssue, none).is_some());
    let linked = HostingAvailability {
        issue_repo: true,
        ..none
    };
    assert!(command_unavailable_from(HostingCommand::CreateIssue, linked).is_none());

    ui.set_issues_source("corp", "g/p".into());
    let seed = ui.issue_seed().expect("linked");
    assert_eq!(seed.project, "g/p");
    assert_eq!(seed.integration.id, "corp");
    assert_eq!(
        ui.run_command(HostingCommand::CreateIssue, (0, 0), &mut clip),
        HostingEffect::NewIssue(Box::new(seed))
    );
}

#[test]
fn edit_on_the_host_opens_the_edit_view_where_there_is_one() {
    use gitpoulpi_config::IntegrationKind as K;
    assert_eq!(
        edit_url(
            Some(K::GitlabSelfManaged),
            "https://h/g/p/-/merge_requests/53"
        ),
        "https://h/g/p/-/merge_requests/53/edit"
    );
    assert_eq!(
        edit_url(Some(K::Github), "https://github.com/o/r/pull/7"),
        "https://github.com/o/r/pull/7"
    );
    // And the menu entry opens that URL (the browser is never launched
    // with GITPOULPI_NO_BROWSER; the status line reports the URL).
    let (mut ui, _d) = ui_with(snapshot(vec![open_pr(53)], vec![]));
    let url = format!("{}/edit", open_pr(53).web_url);
    let mut clip = crate::clipboard::Clipboard::new();
    std::env::set_var("GITPOULPI_NO_BROWSER", "1");
    let effect = ui.run(
        HostingAction::OpenUrl(edit_url(Some(K::GitlabSelfManaged), &open_pr(53).web_url)),
        &mut clip,
    );
    assert_eq!(
        effect,
        HostingEffect::Message(format!("opened {url} in your browser"))
    );
}

#[test]
fn the_pull_request_menu_offers_the_five_actions_and_grays_out_forks() {
    let (mut ui, _d) = ui_with(snapshot(vec![open_pr(53)], vec![]));
    let pull = first_pull(&ui);
    assert!(ui.open_row_menu(HostingList::Pulls, pull, 3, 3));
    // The controls have no menu of their own.
    assert!(!ui.open_row_menu(HostingList::Pulls, 0, 3, 3));
    assert!(ui.open_row_menu(HostingList::Pulls, pull, 3, 3));
    let menu = ui.menu.as_ref().unwrap();
    let labels: Vec<String> = (0..menu.row_count()).map(|r| menu.row_label(r)).collect();
    for want in [
        "Open in browser",
        "Edit on GitLab",
        "Copy URL",
        "Copy number",
        "Check out source branch",
        "View source-branch commits",
    ] {
        assert!(
            labels.iter().any(|l| l.contains(want)),
            "{want}: {labels:?}"
        );
    }

    let mut fork = open_pr(7);
    fork.source_project = Some("someone/p".into());
    let (mut ui, _d) = ui_with(snapshot(vec![fork], vec![]));
    let pull = first_pull(&ui);
    ui.open_row_menu(HostingList::Pulls, pull, 3, 3);
    let menu = ui.menu.as_ref().unwrap();
    let labels: Vec<String> = (0..menu.row_count()).map(|r| menu.row_label(r)).collect();
    assert!(
        labels.iter().any(|l| l.contains("from a fork")),
        "{labels:?}"
    );
    let view_row = labels
        .iter()
        .position(|l| l.contains("View source-branch commits"))
        .unwrap_or_else(|| panic!("{labels:?}"));
    assert!(
        !menu.row_is_selectable(view_row),
        "a fork's branch is not here"
    );
    assert!(
        labels
            .iter()
            .any(|l| l.contains("Check out pull request (fork)")),
        "{labels:?}"
    );
}

#[test]
fn checking_out_a_fork_pull_request_fetches_its_head_ref() {
    let mut fork = open_pr(7);
    fork.source_project = Some("someone/p".into());
    let (mut ui, _d) = ui_with(snapshot(vec![fork], vec![]));
    let mut clip = crate::clipboard::Clipboard::new();
    let effect = ui.run(
        HostingAction::CheckoutPullRequestHead { number: 7 },
        &mut clip,
    );
    match effect {
        HostingEffect::CheckoutPullRequestHead {
            remote_ref, branch, ..
        } => {
            assert!(
                remote_ref == "refs/pull/7/head" || remote_ref == "refs/merge-requests/7/head",
                "{remote_ref}"
            );
            assert!(branch == "pr/7" || branch == "mr/7", "{branch}");
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn menu_actions_return_the_effects_the_loop_acts_on() {
    let (mut ui, _d) = ui_with(snapshot(vec![open_pr(53)], vec![issue(12)]));
    let mut clip = crate::clipboard::Clipboard::new();
    assert_eq!(
        ui.run(
            HostingAction::CheckoutBranch {
                branch: "feature".into(),
                remote: "origin".into()
            },
            &mut clip
        ),
        HostingEffect::Checkout {
            branch: "feature".into(),
            remote: "origin".into()
        }
    );
    assert_eq!(
        ui.run(
            HostingAction::NewBranchFromIssue("12-fix".into()),
            &mut clip
        ),
        HostingEffect::NewBranch("12-fix".into())
    );
    assert!(ui.menu.is_none(), "running an action closes the menu");
}

#[test]
fn issue_branch_names_are_slugged_and_bounded() {
    assert_eq!(issue_branch_name(12, "Fix the Thing!"), "12-fix-the-thing");
    assert_eq!(issue_branch_name(3, "  -  "), "3");
    assert_eq!(issue_branch_name(5, "\u{00E9}t\u{00E9} 2026"), "5-t-2026");
    let long = issue_branch_name(1, &"word ".repeat(30));
    assert!(long.len() <= 43, "{long}");
    assert!(!long.ends_with('-'));
}

#[test]
fn a_source_branch_resolves_to_local_first_then_the_remote_one() {
    let refs = vec![
        local_branch_at("feature", oid(1)),
        remote_branch_at("origin", "feature", oid(1)),
        remote_branch_at("origin", "other", oid(2)),
    ];
    assert_eq!(
        branch_target(&refs, "origin", "feature").as_deref(),
        Some("feature")
    );
    assert_eq!(
        branch_target(&refs, "origin", "other").as_deref(),
        Some("origin/other")
    );
    assert_eq!(branch_target(&refs, "origin", "missing"), None);
    assert_eq!(branch_target(&refs, "upstream", "other"), None);
}

#[test]
fn palette_commands_are_gated_and_the_choosers_open_a_menu() {
    let (mut ui, _d) = ui_with(snapshot(vec![open_pr(53)], vec![issue(12)]));
    for cmd in [
        HostingCommand::ChoosePullRequestFilter,
        HostingCommand::ChooseIssueFilter,
        HostingCommand::SearchPullRequests,
        HostingCommand::SearchIssues,
        HostingCommand::ChooseIssueProvider,
        HostingCommand::ChooseIssueRepository,
        HostingCommand::CreatePullRequest,
        HostingCommand::OpenLinkedPr,
        HostingCommand::CopyLinkedPrUrl,
        HostingCommand::RefreshHosting,
        HostingCommand::CheckoutPullRequest,
        HostingCommand::OpenIssue,
        HostingCommand::IntegrationsPreferences,
        HostingCommand::UseProfile,
        HostingCommand::ConnectIntegration,
    ] {
        assert_eq!(ui.command_unavailable(cmd), None, "{cmd:?}");
    }
    let mut clip = crate::clipboard::Clipboard::new();
    assert_eq!(
        ui.run_command(HostingCommand::IntegrationsPreferences, (0, 0), &mut clip),
        HostingEffect::Preferences(Some("corp".into()))
    );
    assert_eq!(
        ui.run_command(HostingCommand::ConnectIntegration, (0, 0), &mut clip),
        HostingEffect::Preferences(None)
    );
    ui.run_command(HostingCommand::CheckoutPullRequest, (2, 2), &mut clip);
    let menu = ui.menu.as_ref().unwrap();
    assert!(menu.row_label(0).contains("!53"));
    ui.run_command(HostingCommand::UseProfile, (2, 2), &mut clip);
    let menu = ui.menu.as_ref().unwrap();
    assert!(menu.row_label(0).contains("corp (current)"));

    // Unlinked: the linked-PR commands say why.
    let dir = tempfile::tempdir().unwrap();
    let ui = HostingUi::with_parts(dir.path(), None, None, profiles, None);
    assert!(ui
        .command_unavailable(HostingCommand::OpenLinkedPr)
        .is_some());
    assert!(ui.command_unavailable(HostingCommand::OpenIssue).is_some());
    assert!(ui
        .command_unavailable(HostingCommand::ConnectIntegration)
        .is_none());
}

#[cfg(feature = "board")]
#[test]
fn the_board_command_needs_a_linked_source_and_opens_on_it() {
    let (mut ui, _d) = ui_with(snapshot(vec![], vec![]));
    let mut clip = crate::clipboard::Clipboard::new();
    assert!(matches!(
        ui.run_command(HostingCommand::OpenBoard, (0, 0), &mut clip),
        HostingEffect::Message(_)
    ));
    let none = HostingAvailability::default();
    assert!(command_unavailable_from(HostingCommand::OpenBoard, none).is_some());
    ui.set_issues_source("corp", "g/p".into());
    match ui.run_command(HostingCommand::OpenBoard, (0, 0), &mut clip) {
        HostingEffect::Board(target) => {
            assert_eq!(target.seed.project, "g/p");
            assert_eq!(target.seed.integration.id, "corp");
            assert!(target.labels.is_empty());
        }
        other => panic!("{other:?}"),
    }
}
