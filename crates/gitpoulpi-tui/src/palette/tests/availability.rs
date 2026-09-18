use super::*;

/// LFS commands that run the program say why they are grayed out --
/// with the install link when it is the program that is missing.
#[test]
fn the_lfs_commands_are_grayed_out_with_a_reason_when_off_or_when_git_lfs_is_missing() {
    let on = PaletteContext {
        lfs_enabled: true,
        lfs_program_ok: true,
        has_remotes: true,
        unstaged_file_selected: true,
        ..PaletteContext::default()
    };
    for command in [
        PaletteCommand::LfsInitialize,
        PaletteCommand::LfsTrackPattern,
        PaletteCommand::LfsFetch,
        PaletteCommand::LfsPull,
        PaletteCommand::LfsPush,
        PaletteCommand::LfsPrune,
        PaletteCommand::LfsShowLocks,
        PaletteCommand::LfsLockSelectedFile,
    ] {
        assert_eq!(unavailable_reason(command, on), None, "{command:?}");
    }
    let missing = PaletteContext {
        lfs_program_ok: false,
        ..on
    };
    let reason = unavailable_reason(PaletteCommand::LfsPull, missing).unwrap();
    assert!(reason.contains("git-lfs.com"), "{reason}");
    let off = PaletteContext {
        lfs_enabled: false,
        ..on
    };
    assert!(unavailable_reason(PaletteCommand::LfsPull, off)
        .unwrap()
        .contains("turned off"));
    // The install page is the way out of "missing": never grayed out.
    assert_eq!(
        unavailable_reason(PaletteCommand::LfsOpenInstallPage, missing),
        None
    );
    // Network commands need a remote; file commands a selection.
    let no_remote = PaletteContext {
        has_remotes: false,
        ..on
    };
    assert!(unavailable_reason(PaletteCommand::LfsFetch, no_remote).is_some());
    let no_file = PaletteContext {
        unstaged_file_selected: false,
        ..on
    };
    assert!(unavailable_reason(PaletteCommand::LfsLockSelectedFile, no_file).is_some());
}

#[test]
fn repository_commands_are_grayed_on_the_home_tab() {
    let home = PaletteContext {
        in_repository_management: true,
        ..PaletteContext::default()
    };
    let reason_of = |ctx: PaletteContext, label: &str| {
        palette_matches(label, ctx)
            .into_iter()
            .find(|m| m.entry.label.get() == label)
            .unwrap_or_else(|| panic!("{label} not found"))
            .unavailable
    };
    for label in [
        "Git: Fetch",
        "Refs: Check Out Selected Ref",
        "Navigation: Focus Log Pane",
    ] {
        if PALETTE_COMMANDS.iter().any(|e| e.label.get() == label) {
            assert_eq!(reason_of(home, label), Some("needs an open repository"));
        }
    }
    for label in [
        "App: Settings...",
        "Repository: Clone...",
        "Repositories: Search",
    ] {
        assert_eq!(reason_of(home, label), None, "{label}");
    }
    // Outside Home the same commands carry no such reason.
    assert_ne!(
        reason_of(PaletteContext::default(), "Navigation: Focus Log Pane"),
        Some("needs an open repository")
    );
    // Every repository-category command in the catalogue is grayed.
    let grayed = palette_matches("", home)
        .into_iter()
        .filter(|m| needs_repository(m.entry.label.get()))
        .all(|m| m.unavailable.is_some());
    assert!(grayed);
}

#[test]
fn management_commands_need_the_management_tab_and_a_selected_row() {
    use ManagementCommand as M;
    let repo_window = PaletteContext {
        open_repository_tabs: 2,
        ..PaletteContext::default()
    };
    let manager = PaletteContext {
        in_repository_management: true,
        open_repository_tabs: 2,
        ..PaletteContext::default()
    };
    let manager_on_row = PaletteContext {
        management_row_selected: true,
        ..manager
    };
    let open = PaletteCommand::Management(M::OpenSelected);
    assert!(unavailable_reason(open, repo_window).is_some());
    assert!(unavailable_reason(open, manager).is_some());
    assert_eq!(unavailable_reason(open, manager_on_row), None);
    let collapse = PaletteCommand::Management(M::CollapseAll);
    assert_eq!(unavailable_reason(collapse, manager), None);
    assert!(
        unavailable_reason(collapse, repo_window).is_some(),
        "a management command outside the management tab"
    );
    let copy = PaletteCommand::TabMenu(RepoMenuAction::CopyPath);
    assert_eq!(unavailable_reason(copy, repo_window), None);
    assert!(unavailable_reason(copy, manager).is_some());
    assert_eq!(unavailable_reason(copy, manager_on_row), None);
    assert!(unavailable_reason(PaletteCommand::OpenTabMenu, manager).is_some());
}

/// Clone and open are available from any state, including a window with
/// no tabs, which is the state a first clone happens in (ADR-0006).
#[test]
fn clone_and_open_are_available_from_every_state() {
    for ctx in [
        PaletteContext::default(),
        PaletteContext {
            open_repository_tabs: 3,
            selected_local_branch: true,
            selected_commit: true,
            has_conflicts: true,
            conflicted_file_selected: true,
            ..PaletteContext::default()
        },
    ] {
        for command in [
            PaletteCommand::CloneRepository,
            PaletteCommand::OpenRepository,
        ] {
            assert_eq!(
                unavailable_reason(command, ctx),
                None,
                "{command:?} must always be reachable"
            );
        }
    }
}

/// Tab navigation needs somewhere else to go, and closing needs
/// something to close - and both reasons have to name the state so
/// the user knows what would make the command work.
#[test]
fn the_tab_commands_gray_out_when_there_is_nothing_to_do() {
    let none = PaletteContext::default();
    let one = PaletteContext {
        open_repository_tabs: 1,
        ..PaletteContext::default()
    };
    let two = PaletteContext {
        open_repository_tabs: 2,
        ..PaletteContext::default()
    };

    for forward in [true, false] {
        let cycle = PaletteCommand::CycleRepositoryTab(forward);
        assert!(unavailable_reason(cycle, none).is_some());
        assert!(
            unavailable_reason(cycle, one).is_some_and(|r| r.contains("only one repository")),
            "cycling with one tab must say why"
        );
        assert_eq!(unavailable_reason(cycle, two), None);
    }

    assert!(unavailable_reason(PaletteCommand::CloseRepositoryTab, none).is_some());
    assert_eq!(
        unavailable_reason(PaletteCommand::CloseRepositoryTab, one),
        None
    );
}

/// The commit-targeted trio keys off `selected_commit`, not "some row
/// is selected": the Log's pinned WIP row is a rendering of the
/// working tree and has no `Oid` behind it.
#[test]
fn the_commit_operations_need_a_commit_not_just_a_selected_row() {
    let wip = PaletteContext::default();
    let on_commit = PaletteContext {
        selected_commit: true,
        ..PaletteContext::default()
    };
    for cmd in [
        PaletteCommand::CherryPickSelectedCommit,
        PaletteCommand::RevertSelectedCommit,
        PaletteCommand::ResetToSelectedCommit(ResetMode::Soft),
        PaletteCommand::ResetToSelectedCommit(ResetMode::Mixed),
        PaletteCommand::ResetToSelectedCommit(ResetMode::Hard),
    ] {
        assert!(
            unavailable_reason(cmd, wip).is_some_and(|r| r.contains("WIP")),
            "{cmd:?} should name the WIP row"
        );
        assert_eq!(unavailable_reason(cmd, on_commit), None, "{cmd:?}");
    }
}

/// The WIP row's inline message field is reachable from the palette
/// (principle 5), and grayed out - with the reason - when there is
/// no WIP row to type into.
#[test]
fn editing_the_wip_message_needs_uncommitted_changes() {
    assert!(PALETTE_COMMANDS
        .iter()
        .any(|e| e.command == PaletteCommand::EditWipMessage));
    let clean = PaletteContext::default();
    let dirty = PaletteContext {
        has_wip: true,
        ..PaletteContext::default()
    };
    assert!(unavailable_reason(PaletteCommand::EditWipMessage, clean)
        .is_some_and(|r| r.contains("uncommitted")));
    assert_eq!(
        unavailable_reason(PaletteCommand::EditWipMessage, dirty),
        None
    );
}

/// "There are no conflicts" and "there are, but you haven't pointed at
/// one" send the user to two different places, so they are two
/// different reasons.
#[test]
fn conflict_resolution_distinguishes_no_conflicts_from_no_selection() {
    let clean = PaletteContext::default();
    let unselected = PaletteContext {
        has_conflicts: true,
        ..PaletteContext::default()
    };
    let ready = PaletteContext {
        has_conflicts: true,
        conflicted_file_selected: true,
        ..PaletteContext::default()
    };
    for side in [ConflictSide::Ours, ConflictSide::Theirs, ConflictSide::Base] {
        let cmd = PaletteCommand::ResolveConflictWith(side);
        assert!(unavailable_reason(cmd, clean).is_some_and(|r| r.contains("no unresolved")));
        assert!(unavailable_reason(cmd, unselected).is_some_and(|r| r.contains("select")));
        assert_eq!(unavailable_reason(cmd, ready), None);
    }
}

/// Rebase asks for the same thing Merge does - a local branch in the
/// sidebar - and must not become available a frame earlier or later
/// than it does.
#[test]
fn rebase_onto_a_ref_is_available_exactly_when_merge_is() {
    for ctx in [
        PaletteContext::default(),
        PaletteContext {
            selected_local_branch: true,
            ..PaletteContext::default()
        },
        PaletteContext {
            selected_remote_branch: true,
            ..PaletteContext::default()
        },
        PaletteContext {
            selected_tag: true,
            ..PaletteContext::default()
        },
    ] {
        assert_eq!(
            unavailable_reason(PaletteCommand::RebaseOntoSelectedRef, ctx),
            unavailable_reason(PaletteCommand::MergeSelectedRef, ctx),
        );
    }
}

/// Availability is contextual, and the reason is always populated when
/// a command is unavailable - a grayed row with no explanation would
/// be worse than hiding it.
#[test]
fn contextual_commands_explain_why_they_are_unavailable() {
    let idle = PaletteContext::default();
    assert!(unavailable_reason(PaletteCommand::ToggleSolo, idle).is_some());
    assert!(unavailable_reason(PaletteCommand::CloseDiff, idle).is_some());
    assert!(unavailable_reason(PaletteCommand::StashPop(StashTarget::Selected), idle).is_some());

    let busy = PaletteContext {
        selected_local_branch: true,
        in_diff_view: true,
        has_stash: true,
        ..PaletteContext::default()
    };
    assert_eq!(unavailable_reason(PaletteCommand::ToggleSolo, busy), None);
    assert_eq!(unavailable_reason(PaletteCommand::CloseDiff, busy), None);
    assert_eq!(
        unavailable_reason(PaletteCommand::StashPop(StashTarget::Selected), busy),
        None
    );
    assert_eq!(
        unavailable_reason(PaletteCommand::MergeSelectedRef, busy),
        None
    );
}

/// A remote branch can be soloed/hidden but not checked out, and the
/// message has to say which - this is the one ref case where "a ref
/// is selected" isn't enough.
#[test]
fn a_remote_branch_cannot_be_checked_out_or_renamed() {
    let remote = PaletteContext {
        selected_remote_branch: true,
        ..PaletteContext::default()
    };
    assert!(
        unavailable_reason(PaletteCommand::CheckoutSelectedRef, remote)
            .is_some_and(|r| r.contains("remote"))
    );
    assert!(unavailable_reason(PaletteCommand::RenameSelectedBranch, remote).is_some());
    assert_eq!(unavailable_reason(PaletteCommand::ToggleSolo, remote), None);
}

/// The editor's status menus exist only while the editor is the central
/// view, and each is in the catalogue (the palette reaches 100%).
#[test]
fn the_editor_menus_are_offered_only_while_the_editor_is_open() {
    use crate::editor::StatusMenu;
    let menus = [
        StatusMenu::Encoding,
        StatusMenu::Reopen,
        StatusMenu::Convert,
        StatusMenu::LineEndings,
        StatusMenu::Indentation,
    ];
    let open = PaletteContext {
        editor_open: true,
        ..PaletteContext::default()
    };
    for menu in menus {
        let command = PaletteCommand::EditorMenu(menu);
        assert_eq!(unavailable_reason(command, open), None, "{menu:?}");
        let reason = unavailable_reason(command, PaletteContext::default())
            .unwrap_or_else(|| panic!("{menu:?} is grayed out without the editor"));
        assert!(reason.contains("editor"), "{reason}");
        assert!(
            PALETTE_COMMANDS.iter().any(|e| e.command == command),
            "{menu:?} is in the catalogue"
        );
    }
}

/// While Settings is open only the app-wide popups run; every other command
/// is grayed out with the one reason, and none of them is when it is closed.
#[test]
fn only_the_app_wide_commands_work_over_the_settings_screen() {
    let open = PaletteContext {
        settings_open: true,
        ..PaletteContext::default()
    };
    let closed = PaletteContext::default();
    for command in [
        PaletteCommand::ChangeTheme,
        PaletteCommand::ChangeLanguage,
        PaletteCommand::WorkspaceSearch,
        PaletteCommand::AboutGitPoulpi,
        PaletteCommand::ToggleHelp,
    ] {
        assert_eq!(unavailable_reason(command, open), None, "{command:?}");
    }
    for command in [
        PaletteCommand::Refresh,
        PaletteCommand::SelectTab,
        PaletteCommand::OpenDiff,
        PaletteCommand::OpenPreferences,
    ] {
        assert_eq!(
            unavailable_reason(command, open),
            Some(gitpoulpi_i18n::msg::palette::WHY_CLOSE_SETTINGS_FIRST.get()),
            "{command:?}"
        );
    }
    assert_ne!(
        unavailable_reason(PaletteCommand::SelectTab, closed),
        Some(gitpoulpi_i18n::msg::palette::WHY_CLOSE_SETTINGS_FIRST.get())
    );
}
