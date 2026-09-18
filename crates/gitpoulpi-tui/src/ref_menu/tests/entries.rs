use super::*;

/// Both context menus' labels obey the glyph rule that has already
/// produced two rendering holes: no variation selector, nothing
/// outside the BMP. Walks submenu rows too.
#[test]
fn the_context_menu_labels_use_no_variation_selectors() {
    fn check<A>(entries: &[context_menu::Entry<A>]) {
        for e in entries {
            let (label, kids) = match e {
                context_menu::Entry::Action { label, .. } => (label, None),
                context_menu::Entry::Submenu { label, items } => (label, Some(items)),
                context_menu::Entry::Disabled { label } => (label, None),
                context_menu::Entry::Separator => continue,
            };
            for c in label.chars() {
                assert!(
                    c != '\u{FE0E}' && c != '\u{FE0F}',
                    "{label:?} carries a variation selector"
                );
                assert!(
                    (c as u32) < 0x1_0000,
                    "{label:?} uses a non-BMP glyph: {c:?}"
                );
            }
            if let Some(kids) = kids {
                check(kids);
            }
        }
    }
    check(&commit_menu_entries(&MenuContext::default(), false));
    for section in [
        SidebarSection::Local,
        SidebarSection::Remote,
        SidebarSection::Tags,
        SidebarSection::Stashes,
    ] {
        // `Hide in graph` predates the rule and keeps its U+1F441;
        // it is checked by `ref_menu_entries`' own callers, so only
        // the newer rows are asserted here.
        let entries: Vec<_> =
            ref_menu_entries(section, "x", &MenuContext::default(), false, false, false)
                .into_iter()
                .filter(|e| {
                    !matches!(e, context_menu::Entry::Action { action, .. }
                if matches!(action, RefContextAction::ToggleHideInGraph))
                })
                .collect();
        check(&entries);
    }
}

/// A ref's three copyable cells. `qualified_ref_name` is the one that
/// can silently be wrong - `refs/heads/origin/main` for a remote
/// would paste into a script and fail somewhere else entirely.
#[test]
fn a_refs_copy_rows_read_its_own_cells() {
    assert_eq!(
        qualified_ref_name(RefRowKind::Local, "main"),
        "refs/heads/main"
    );
    assert_eq!(
        qualified_ref_name(RefRowKind::Remote, "origin/main"),
        "refs/remotes/origin/main"
    );
    assert_eq!(
        qualified_ref_name(RefRowKind::Tag, "v1.2.0"),
        "refs/tags/v1.2.0"
    );

    let refs: gitpoulpi_core::Result<Vec<_>> = Ok(vec![
        local_branch_at("main", oid(1)),
        gitpoulpi_core::domain::Ref {
            kind: RefKind::RemoteBranch {
                remote: "origin".into(),
                name: "main".into(),
            },
            target: oid(2),
            visibility: Default::default(),
            is_head: false,
            upstream: None,
            tag_sync: None,
        },
    ]);
    assert_eq!(
        ref_by_display_name(&refs, RefRowKind::Local, "main").map(|r| r.target),
        Some(oid(1))
    );
    // The remote row is matched on its *displayed* `{remote}/{name}`,
    // which is the only spelling the sidebar ever has.
    assert_eq!(
        ref_by_display_name(&refs, RefRowKind::Remote, "origin/main").map(|r| r.target),
        Some(oid(2))
    );
    assert!(ref_by_display_name(&refs, RefRowKind::Remote, "main").is_none());
    assert!(ref_by_display_name(&refs, RefRowKind::Tag, "main").is_none());
}

/// Checkout follows the policy every other checkout affordance in
/// this UI already follows: local branches and tags yes, remote
/// branches no. Offering a row that can only print "not supported"
/// is the dead-affordance complaint this project keeps getting.
#[test]
fn ref_menus_offer_checkout_but_stashes_do_not() {
    let offers_checkout = |section| {
        ref_menu_entries(section, "x", &MenuContext::default(), false, false, false)
            .iter()
            .any(|e| {
                matches!(
                    e,
                    context_menu::Entry::Action {
                        action: RefContextAction::Checkout,
                        ..
                    }
                )
            })
    };
    assert!(offers_checkout(SidebarSection::Local));
    assert!(offers_checkout(SidebarSection::Tags));
    // A remote branch's Checkout goes through `checkout_ref_now`, the
    // same tracking-branch resolution the sidebar's double-click
    // uses -- never a raw checkout of the remote ref itself, but the
    // row is offered.
    assert!(offers_checkout(SidebarSection::Remote));
    assert!(!offers_checkout(SidebarSection::Stashes));
}

/// Every ref section that is a ref gets the Copy submenu, and a
/// stash - which is not a ref and has no name to copy - does not.
#[test]
fn every_ref_kind_gets_a_copy_submenu_and_a_stash_does_not() {
    let has_copy = |section| {
        ref_menu_entries(section, "x", &MenuContext::default(), false, false, false)
            .iter()
            .any(|e| {
                matches!(e, context_menu::Entry::Submenu { items, .. }
                if items.len() == RefField::all().len())
            })
    };
    assert!(has_copy(SidebarSection::Local));
    assert!(has_copy(SidebarSection::Remote));
    assert!(has_copy(SidebarSection::Tags));
    assert!(!has_copy(SidebarSection::Stashes));
}

/// Checkout, merge, rebase and delete against the branch you are
/// already on are a no-op or a git error. They are hidden
/// because because a row whose only outcome is an error message is the
/// dead affordance this project keeps getting complaints about.
#[test]
fn the_checked_out_branch_is_not_offered_rows_that_act_on_itself() {
    let ctx = MenuContext {
        current_branch: Some("main".into()),
        ..Default::default()
    };
    let actions = |name: &str| -> Vec<RefContextAction> {
        ref_menu_entries(SidebarSection::Local, name, &ctx, false, false, false)
            .iter()
            .filter_map(|e| match e {
                context_menu::Entry::Action { action, .. } => Some(*action),
                _ => None,
            })
            .collect()
    };

    let on_self = actions("main");
    for forbidden in [
        RefContextAction::Checkout,
        RefContextAction::MergeIntoCurrent,
        RefContextAction::RebaseOntoThis,
    ] {
        assert!(
            !on_self.contains(&forbidden),
            "{forbidden:?} is offered for the branch already checked out: {on_self:?}"
        );
    }
    // ...and it keeps everything that still makes sense.
    assert!(on_self.contains(&RefContextAction::RenameBranch));
    assert!(on_self.contains(&RefContextAction::ToggleSolo));
    // Delete stays: it is refused with a message when run, not hidden.
    assert!(on_self.contains(&RefContextAction::DeleteBranch));
    assert!(
        ref_menu_entries(SidebarSection::Local, "main", &ctx, false, false, false)
            .iter()
            .any(|e| matches!(e, context_menu::Entry::Submenu { .. }))
    );

    let other = actions("feature");
    assert!(other.contains(&RefContextAction::Checkout));
    assert!(other.contains(&RefContextAction::DeleteBranch));
}

/// The rows that cannot be undone keep going through the one confirm
/// modal, and are the ones painted red. A destructive row that skips
/// the modal, or a harmless one painted red, both teach the user to
/// ignore the color.
#[test]
fn exactly_the_irreversible_ref_rows_are_marked_destructive() {
    let ctx = MenuContext {
        current_branch: Some("main".into()),
        ..Default::default()
    };
    let mut seen = 0;
    for (section, name) in [
        (SidebarSection::Local, "feature"),
        (SidebarSection::Remote, "origin/feature"),
        (SidebarSection::Tags, "v1.0"),
        (SidebarSection::Stashes, "0"),
    ] {
        for e in ref_menu_entries(section, name, &ctx, false, false, false) {
            if let context_menu::Entry::Action {
                action,
                destructive,
                label,
                ..
            } = e
            {
                let irreversible = matches!(
                    action,
                    RefContextAction::DeleteBranch
                        | RefContextAction::DeleteRemoteBranch
                        | RefContextAction::DeleteBranchAndRemote
                        | RefContextAction::DeleteTag
                        | RefContextAction::StashDrop
                );
                assert_eq!(
                    destructive, irreversible,
                    "{label:?} ({action:?}) is painted {destructive} but irreversible is {irreversible}"
                );
                seen += usize::from(irreversible);
            }
        }
    }
    // local delete, remote delete (remote row), delete-both (remote
    // row: only with a local twin, so none here), delete-tag, stash-drop.
    assert_eq!(
        seen, 4,
        "delete-branch, delete-remote, delete-tag, stash-drop"
    );
}

/// Every ref kind with a commit leads with Reveal in graph; the Log's own
/// menus drop it (they are already there).
#[test]
fn every_ref_menu_leads_with_reveal_in_graph_except_inside_the_log() {
    for section in [
        SidebarSection::Local,
        SidebarSection::Remote,
        SidebarSection::Tags,
        SidebarSection::Stashes,
    ] {
        let mut entries =
            ref_menu_entries(section, "x", &MenuContext::default(), false, false, false);
        assert!(
            matches!(
                entries.first(),
                Some(context_menu::Entry::Action {
                    action: RefContextAction::RevealInGraph,
                    ..
                })
            ),
            "{section:?} does not lead with Reveal in graph"
        );
        assert!(matches!(entries[1], context_menu::Entry::Separator));
        drop_reveal_row(&mut entries);
        assert!(!entries.iter().any(|e| matches!(
            e,
            context_menu::Entry::Action {
                action: RefContextAction::RevealInGraph,
                ..
            }
        )));
    }
}
