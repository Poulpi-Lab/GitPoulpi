use super::*;

#[test]
fn the_form_opens_on_the_remotes_provider_repository_and_branches() {
    let d = dialog();
    assert_eq!(d.seed.tabs[d.tab].integration.id, "corp");
    assert_eq!(d.from_repo.selected(), Some("g/p"));
    assert_eq!(d.to_repo.selected(), Some("g/p"));
    assert_eq!(d.from_branch.selected(), Some("feature"));
    assert_eq!(d.to_branch.selected(), Some("main"));
    assert_eq!(d.title, "Add the thing");
    let plan = d.plan().unwrap();
    assert_eq!(plan.project, "g/p");
    assert_eq!(plan.source_project, None, "same repository: no fork");
    assert_eq!(
        (plan.source_branch.as_str(), plan.target_branch.as_str()),
        ("feature", "main")
    );
    assert!(!plan.draft);
}

#[test]
fn the_form_hides_the_ai_hook() {
    let mut d = dialog();
    let (text, rect) = paint(&mut d, 110, 34);
    for want in [
        "Create Pull Request",
        "GitHub",
        "GitLab",
        "From repo",
        "To repo",
        "Branch",
        "Title",
        "Description",
        "Submit as draft",
        "Cancel",
        "g/p",
        "feature",
        "main",
    ] {
        assert!(text.contains(want), "{want}\n{text}");
    }
    assert!(
        text.contains("Bitbucket") && !text.contains("Jira"),
        "every provider with a pull-request client has a tab, an issue tracker none"
    );
    assert!(
        !text.contains("Generate title"),
        "the AI hook stays hidden until it exists"
    );
    for (r, _) in &d.hits {
        assert!(rect.contains((r.x, r.y).into()), "{r:?}");
    }
    // It also fits a small terminal without panicking.
    for (w, h) in [(60u16, 20u16), (40, 16), (30, 10)] {
        let mut d = dialog();
        let _ = paint(&mut d, w, h);
    }
}

#[test]
fn keys_walk_the_form_pick_from_the_combos_and_create() {
    let mut d = dialog();
    // Focus starts on Title; the description takes Enter as a newline.
    assert_eq!(d.focus, PrFocus::Title);
    d.on_key(key(KeyCode::Enter)); // Title: Enter moves on
    assert_eq!(d.focus, PrFocus::Body);
    for c in "line one".chars() {
        d.on_key(key(KeyCode::Char(c)));
    }
    d.on_key(key(KeyCode::Enter));
    for c in "line two".chars() {
        d.on_key(key(KeyCode::Char(c)));
    }
    assert_eq!(d.body, "line one\nline two");
    // Up moves the caret inside the description, it does not leave it.
    d.on_key(key(KeyCode::Up));
    assert_eq!(d.focus, PrFocus::Body);
    // Tab walks the metadata buttons (GitHub, GitLab), then Draft, toggled
    // with Space.
    for want in [
        PrFocus::MetaAssignees,
        PrFocus::MetaReviewers,
        PrFocus::MetaLabels,
        PrFocus::MetaMilestone,
        PrFocus::Draft,
    ] {
        d.on_key(key(KeyCode::Tab));
        assert_eq!(d.focus, want);
    }
    d.on_key(key(KeyCode::Char(' ')));
    assert!(d.draft);
    // Back to the To branch: it opens on "main", two Ups reach "develop".
    d.focus = PrFocus::ToBranch;
    d.on_key(key(KeyCode::Enter));
    assert!(d.to_branch.is_open());
    d.on_key(key(KeyCode::Up));
    d.on_key(key(KeyCode::Up));
    d.on_key(key(KeyCode::Enter));
    assert!(!d.to_branch.is_open());
    assert_eq!(d.to_branch.selected(), Some("develop"));

    // Ctrl+S creates, with everything the form holds.
    let out = d.on_key(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL));
    match out {
        PullRequestOutcome::Submit { integration, req } => {
            assert_eq!(integration.id, "corp");
            assert!(req.draft);
            assert_eq!(req.body, "line one\nline two");
            assert_eq!(req.title, "Add the thing");
        }
        other => panic!("{other:?}"),
    }
    // The Description was typed: Esc asks, Discard closes.
    assert_eq!(d.on_key(key(KeyCode::Esc)), PullRequestOutcome::Stay);
    d.on_key(key(KeyCode::Right));
    assert_eq!(d.on_key(key(KeyCode::Enter)), PullRequestOutcome::Cancel);
}

#[test]
fn the_metadata_buttons_open_the_picker_and_what_it_chose_rides_along() {
    let mut d = dialog();
    let (text, _) = paint(&mut d, 110, 36);
    for want in [
        "Assignees: none",
        "Reviewers: none",
        "Labels: none",
        "Milestone: none",
    ] {
        assert!(text.contains(want), "{want}\n{text}");
    }
    // Enter on a button opens the picker; it owns every key until it closes.
    d.focus = PrFocus::MetaLabels;
    assert_eq!(d.on_key(key(KeyCode::Enter)), PullRequestOutcome::Stay);
    assert!(d.meta.is_open());
    assert_eq!(d.on_key(key(KeyCode::Esc)), PullRequestOutcome::Stay);
    assert!(!d.meta.is_open(), "Esc closed the picker, not the form");
    // What the picker chose reaches the request.
    d.sync_meta_scope();
    d.meta.selection.apply(crate::meta_picker::Applied::List(
        MetaField::Assignees,
        vec!["ann".into()],
    ));
    d.meta.selection.apply(crate::meta_picker::Applied::List(
        MetaField::Reviewers,
        vec!["bob".into()],
    ));
    d.meta.selection.apply(crate::meta_picker::Applied::List(
        MetaField::Labels,
        vec!["bug".into()],
    ));
    d.meta
        .selection
        .apply(crate::meta_picker::Applied::Milestone(Some(
            crate::meta_picker::MilestoneRef {
                id: 7,
                title: "v1".into(),
            },
        )));
    let plan = d.plan().unwrap();
    assert_eq!(plan.assignees, ["ann"]);
    assert_eq!(plan.reviewers, ["bob"]);
    assert_eq!(plan.labels, ["bug"]);
    assert_eq!(plan.milestone, Some(7));
    // Another target repository starts the choices over.
    d.to_repo.select(Some("other/q".into()));
    d.sync_meta_scope();
    let plan = d.plan().unwrap();
    assert!(plan.assignees.is_empty() && plan.milestone.is_none());
}

#[test]
fn a_provider_without_metadata_shows_none_of_it() {
    let mut d = dialog();
    let bitbucket = d
        .seed
        .tabs
        .iter()
        .position(|t| t.integration.kind == gitpoulpi_hosting::IntegrationKind::Bitbucket)
        .expect("a Bitbucket tab");
    d.tab = bitbucket;
    let (text, _) = paint(&mut d, 110, 36);
    assert!(!text.contains("Assignees"), "{text}");
    assert!(!d.order().contains(&PrFocus::MetaAssignees));
}

#[test]
fn an_incomplete_form_says_why_and_never_submits() {
    let mut d = dialog();
    d.title.clear();
    assert_eq!(d.plan().unwrap_err(), "choose a title");
    assert_eq!(
        d.on_key(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL)),
        PullRequestOutcome::Stay
    );
    assert_eq!(d.message.as_deref(), Some("choose a title"));
    let mut d = dialog();
    d.to_branch.select(Some("feature".into()));
    assert!(d.plan().unwrap_err().contains("nothing to merge"));
    let mut d = dialog();
    d.from_repo.select(None);
    assert!(d.plan().unwrap_err().contains("source branch"));
}

#[test]
fn a_different_source_repository_is_a_fork_request() {
    let mut d = dialog();
    d.from_repo.select(Some("me/p".into()));
    let plan = d.plan().unwrap();
    assert_eq!(plan.project, "g/p");
    assert_eq!(plan.source_project.as_deref(), Some("me/p"));
}

#[test]
fn the_form_says_it_is_loading_until_the_repository_list_lands() {
    let mut d = dialog();
    let id = d.current_tab().unwrap().integration.id.clone();
    d.projects.insert(id.clone(), ProjectsState::Loading);
    d.message = None;
    assert!(d.is_loading());
    let (text, _) = paint(&mut d, 110, 34);
    assert!(text.contains(LOADING_TEXT.get()), "{text}");
    d.projects.insert(id, ProjectsState::Ready(Vec::new()));
    assert!(!d.is_loading());
    let (text, _) = paint(&mut d, 110, 34);
    assert!(!text.contains(LOADING_TEXT.get()), "{text}");
}
