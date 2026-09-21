use super::*;

/// Create worktree is on every branch and tag menu (the checked-out
/// branch included: the dialog then asks for a new branch), never on
/// a stash.
#[test]
fn create_worktree_is_offered_for_branches_and_tags() {
    let ctx = MenuContext {
        current_branch: Some("main".into()),
        ..Default::default()
    };
    for (section, name) in [
        (SidebarSection::Local, "main"),
        (SidebarSection::Local, "feature"),
        (SidebarSection::Remote, "origin/feature"),
        (SidebarSection::Tags, "v1.0"),
    ] {
        let entries = ref_menu_entries(section, name, &ctx, false, false, false);
        assert_eq!(
            row_state(&entries, "Create worktree from"),
            Some(Ok(RefContextAction::CreateWorktree)),
            "{section:?} {name}"
        );
    }
    let stash = ref_menu_entries(SidebarSection::Stashes, "0", &ctx, false, false, false);
    assert!(row_state(&stash, "Create worktree").is_none());
}

/// "Start a pull request" is a Local-only row, offered when
/// `ctx.pr_reason` is `None` and disabled with its reason (the
/// caller's own "push it first"/"remote host not recognized") when
/// it is not - a remote branch or a tag never offers it at all.
#[test]
#[cfg(feature = "hosting")]
fn start_pull_request_needs_an_upstream() {
    let blocked = MenuContext {
        current_branch: Some("feature".into()),
        pr_reason: Some("push feature first".into()),
        ..Default::default()
    };
    let entries = ref_menu_entries(
        SidebarSection::Local,
        "feature",
        &blocked,
        false,
        false,
        false,
    );
    let reason = row_state(&entries, "Start a pull request");
    assert!(
        matches!(&reason, Some(Err(label)) if label.contains("push feature first")),
        "a blocked row must report its reason, not be runnable: {reason:?}"
    );

    let ready = MenuContext {
        current_branch: Some("feature".into()),
        pr_reason: None,
        ..Default::default()
    };
    let entries = ref_menu_entries(
        SidebarSection::Local,
        "feature",
        &ready,
        false,
        false,
        false,
    );
    assert_eq!(
        row_state(&entries, "Start a pull request"),
        Some(Ok(RefContextAction::StartPullRequest))
    );

    // Remote branches and tags never offer it at all, blocked or not.
    for (section, name) in [
        (SidebarSection::Remote, "origin/feature"),
        (SidebarSection::Tags, "v1.0"),
    ] {
        let entries = ref_menu_entries(section, name, &ready, false, false, false);
        assert!(
            row_state(&entries, "Start a pull request").is_none(),
            "{section:?}"
        );
    }
}
