//! Linking a provider and a repository: the source dropdowns, proposals and unlinking.

use super::*;

// -- the ISSUES section: provider and repository per repository ---------

#[test]
fn the_issues_section_offers_only_supported_providers() {
    let mut snap = snapshot(vec![], vec![issue(12)]);
    snap.issue_profiles = vec![
        ProfileRef {
            id: "corp".into(),
            label: "corp".into(),
            kind: IntegrationKind::GitlabSelfManaged,
        },
        ProfileRef {
            id: "hub".into(),
            label: "github.com".into(),
            kind: IntegrationKind::Github,
        },
    ];
    let (mut ui, _d) = ui_with(snap);
    let mut clip = crate::clipboard::Clipboard::new();
    let provider = row_of(&ui, HostingList::Issues, HostingRow::IssueProvider);
    ui.activate_row(HostingList::Issues, provider, &mut clip);
    let labels: Vec<String> = ui
        .picker
        .as_ref()
        .unwrap()
        .combo
        .options
        .iter()
        .map(|o| o.label.clone())
        .collect();
    assert_eq!(
        labels,
        vec![
            "GitLab \u{00B7} corp".to_string(),
            "GitHub \u{00B7} github.com".to_string()
        ],
        "GitHub and GitLab profiles, nothing else"
    );
}

#[test]
fn picking_a_provider_and_a_repository_is_remembered_for_this_repo() {
    let mut snap = snapshot(vec![], vec![issue(12)]);
    snap.issues_target = None;
    snap.issues.clear();
    snap.profile_id = None;
    let (mut ui, dir) = ui_with(snap);
    let mut clip = crate::clipboard::Clipboard::new();
    // Nothing chosen: only the two dropdowns, both saying Select...
    assert_eq!(
        ui.rows(HostingList::Issues),
        vec![HostingRow::IssueProvider, HostingRow::IssueRepo]
    );
    let (screen, _, _) = draw_section(&ui, HostingList::Issues, 44, 6, None);
    assert!(screen.contains("Select provider..."), "{screen}");
    assert!(screen.contains("Select repository..."), "{screen}");
    // The repository dropdown waits for a provider.
    let repo = row_of(&ui, HostingList::Issues, HostingRow::IssueRepo);
    assert_eq!(
        ui.activate_row(HostingList::Issues, repo, &mut clip),
        HostingEffect::Message("choose a provider for issues first".into())
    );

    let provider = row_of(&ui, HostingList::Issues, HostingRow::IssueProvider);
    ui.activate_row(HostingList::Issues, provider, &mut clip);
    ui.on_picker_key(key(KeyCode::Enter));
    let path = crate::remotes_ui::repo_state_path(dir.path());
    let state = gitpoulpi_session::RepoState::load(&path).unwrap();
    let source = state.issues_source.expect("saved for this repository");
    assert_eq!(
        (source.integration.as_str(), source.project.as_str()),
        ("corp", "")
    );
    assert_eq!(ui.section_title(HostingList::Issues), "GITLAB ISSUES");

    // The repository: typed, since no list is fetched in this test.
    ui.activate_row(HostingList::Issues, repo, &mut clip);
    for c in "team/app".chars() {
        ui.on_picker_key(key(KeyCode::Char(c)));
    }
    ui.on_picker_key(key(KeyCode::Enter));
    let state = gitpoulpi_session::RepoState::load(&path).unwrap();
    assert_eq!(state.issues_source.unwrap().project, "team/app");
    let rows = ui.rows(HostingList::Issues);
    assert!(rows.contains(&HostingRow::Search) && rows.contains(&HostingRow::Filter));

    // A new window on the same repository starts from what was saved.
    let next = HostingUi::with_parts(dir.path(), None, None, profiles, None);
    assert_eq!(
        next.issues_source,
        Some(gitpoulpi_session::IssuesSource {
            integration: "corp".into(),
            project: "team/app".into(),
            filter: None,
        })
    );
}

#[test]
fn the_repository_dropdown_lists_the_providers_projects_when_they_land() {
    use crate::hosting_task::fake::project;
    let mut factory = FakeFactory::new();
    factory.projects = vec![project("team/app"), project("team/tool")];
    let mut snap = snapshot(vec![], vec![]);
    snap.issues.clear();
    let (mut ui, _d) = ui_with(snap);
    ui.set_factory(Arc::new(factory));
    let mut clip = crate::clipboard::Clipboard::new();
    let repo = row_of(&ui, HostingList::Issues, HostingRow::IssueRepo);
    ui.activate_row(HostingList::Issues, repo, &mut clip);
    assert!(
        ui.picker.as_ref().unwrap().combo.loading,
        "asked, not here yet"
    );
    let deadline = Instant::now() + Duration::from_secs(10);
    while !ui.poll() {
        assert!(Instant::now() < deadline, "the list never landed");
        std::thread::sleep(Duration::from_millis(5));
    }
    let combo = &ui.picker.as_ref().unwrap().combo;
    assert!(!combo.loading);
    let names: Vec<&str> = combo.options.iter().map(|o| o.label.as_str()).collect();
    assert_eq!(names, vec!["team/app", "team/tool"]);
    // Choosing one points the section at it.
    ui.on_picker_key(key(KeyCode::Down));
    ui.on_picker_key(key(KeyCode::Enter));
    assert_eq!(
        ui.snapshot()
            .unwrap()
            .issues_target
            .as_ref()
            .unwrap()
            .project,
        "team/tool"
    );
}

#[test]
fn using_a_profile_pins_it_in_the_repo_state_and_automatic_clears_it() {
    let (mut ui, dir) = ui_with(snapshot(vec![], vec![]));
    let mut clip = crate::clipboard::Clipboard::new();
    ui.run(HostingAction::UseProfile("other".into()), &mut clip);
    let path = crate::remotes_ui::repo_state_path(dir.path());
    let state = gitpoulpi_session::RepoState::load(&path).unwrap();
    assert_eq!(state.hosting_integration.as_deref(), Some("other"));
    ui.run(HostingAction::AutomaticProfile, &mut clip);
    let state = gitpoulpi_session::RepoState::load(&path).unwrap();
    assert_eq!(state.hosting_integration, None);
}

fn suggesting_snapshot() -> HostingSnapshot {
    let mut snap = snapshot(vec![], vec![]);
    snap.profile_id = None;
    snap.can_pulls = false;
    snap.pulls_target = None;
    snap.issues_target = None;
    snap.suggestions = vec![crate::hosting_task::SuggestionRef {
        profile: ProfileRef {
            id: "corp".into(),
            label: "corp".into(),
            kind: IntegrationKind::GitlabSelfManaged,
        },
        remote: "origin".into(),
        project: "g/p".into(),
    }];
    snap
}

fn option_labels(ui: &HostingUi) -> Vec<String> {
    ui.picker
        .as_ref()
        .unwrap()
        .combo
        .options
        .iter()
        .map(|o| o.label.clone())
        .collect()
}

#[test]
fn a_fresh_repository_is_unlinked_and_the_dropdown_only_proposes() {
    let (mut ui, dir) = ui_with(suggesting_snapshot());
    let mut clip = crate::clipboard::Clipboard::new();
    for list in [HostingList::Pulls, HostingList::Issues] {
        assert_eq!(ui.rows(list).len(), 2, "just the two dropdowns");
        assert_eq!(ui.item_count(list), 0);
    }
    assert!(ui.linked_snapshot().is_none() && ui.chip().is_none());
    let provider = row_of(&ui, HostingList::Pulls, HostingRow::PullProvider);
    ui.activate_row(HostingList::Pulls, provider, &mut clip);
    let labels = option_labels(&ui);
    assert!(labels[0].starts_with("g/p"), "{labels:?}");
    assert!(labels[0].contains("origin"), "{labels:?}");
    assert_eq!(labels.len(), 2, "the proposal, then the plain provider");
    // Opening and dismissing it links nothing and writes nothing.
    ui.on_picker_key(key(KeyCode::Esc));
    let state = gitpoulpi_session::RepoState::load(&crate::remotes_ui::repo_state_path(dir.path()))
        .unwrap_or_default();
    assert_eq!((state.pulls_source, state.issues_source), (None, None));
    assert!(ui.linked_snapshot().is_none());
}

#[test]
fn picking_a_proposal_links_it_once_and_unlinking_sticks_for_both_sections() {
    for list in [HostingList::Pulls, HostingList::Issues] {
        let (mut ui, dir) = ui_with(suggesting_snapshot());
        let mut clip = crate::clipboard::Clipboard::new();
        let state_path = crate::remotes_ui::repo_state_path(dir.path());
        let row = match list {
            HostingList::Pulls => HostingRow::PullProvider,
            HostingList::Issues => HostingRow::IssueProvider,
        };
        let at = row_of(&ui, list, row.clone());
        ui.activate_row(list, at, &mut clip);
        ui.on_picker_key(key(KeyCode::Enter));
        let state = gitpoulpi_session::RepoState::load(&state_path).unwrap();
        let saved = match list {
            HostingList::Pulls => state.pulls_source,
            HostingList::Issues => state.issues_source,
        }
        .expect("the pick is persisted");
        assert_eq!(
            (saved.integration.as_str(), saved.project.as_str()),
            ("corp", "g/p")
        );
        // A new window reads the choice back.
        let reopened = HostingUi::with_parts(dir.path(), None, None, profiles, None);
        assert!(match list {
            HostingList::Pulls => reopened.pulls_source.is_some(),
            HostingList::Issues => reopened.issues_source.is_some(),
        });
        // Unlink: the first row of the dropdown.
        let at = row_of(&ui, list, row.clone());
        ui.activate_row(list, at, &mut clip);
        assert_eq!(option_labels(&ui)[0], unlink_label());
        for _ in 0..4 {
            ui.on_picker_key(key(KeyCode::Up));
        }
        ui.on_picker_key(key(KeyCode::Enter));
        let state = gitpoulpi_session::RepoState::load(&state_path).unwrap();
        assert_eq!((state.pulls_source, state.issues_source), (None, None));
        // A worker answer (the remote still matches) never re-links.
        let mut again = suggesting_snapshot();
        again.fetched_at = 2_000;
        ui.apply(again);
        assert_eq!(ui.rows(list).len(), 2);
        assert!(ui.linked_snapshot().is_none());
        let reopened = HostingUi::with_parts(dir.path(), None, None, profiles, None);
        assert!(reopened.pulls_source.is_none() && reopened.issues_source.is_none());
    }
}
