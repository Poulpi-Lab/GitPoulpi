use super::*;

/// The commit-menu row arithmetic, which is the part
/// most likely to be quietly wrong: the WIP row occupies display index
/// 0 on a dirty tree, so a menu anchored by hand opens over one commit
/// and acts on another.
#[test]
fn the_commit_menu_resolves_a_row_to_the_commit_actually_drawn_there() {
    let area = Rect::new(0, 0, 40, 20);
    let graph = Ok(gitpoulpi_core::domain::CommitGraph {
        nodes: vec![commit(1, &[]), commit(2, &[1]), commit(3, &[2])],
    });
    let dirty = graph::LogRows::new(true, Vec::new());
    let clean = graph::LogRows::default();

    // Dirty tree: display index 0 is the one-line WIP row, which is
    // "no commit", and the first commit is right under it.
    let len = 4; // WIP + 3 commits
    assert_eq!(
        log_commit_at_row(&graph, &dirty, area, 0, len, 2),
        None,
        "row 2 is the WIP row, which has no commit"
    );
    assert_eq!(
        log_commit_at_row(&graph, &dirty, area, 0, len, 3),
        Some(oid(1))
    );
    assert_eq!(
        log_commit_at_row(&graph, &dirty, area, 0, len, 4),
        Some(oid(2))
    );

    // Clean tree: no WIP row at all, so nothing shifts.
    assert_eq!(
        log_commit_at_row(&graph, &clean, area, 0, 3, 2),
        Some(oid(1))
    );
    assert_eq!(
        log_commit_at_row(&graph, &clean, area, 0, 3, 3),
        Some(oid(2))
    );

    // Border, header and past-the-end rows resolve to nothing.
    assert_eq!(log_commit_at_row(&graph, &clean, area, 0, 3, 0), None);
    assert_eq!(log_commit_at_row(&graph, &clean, area, 0, 3, 1), None);
    assert_eq!(log_commit_at_row(&graph, &clean, area, 0, 3, 9), None);

    // A stash row between commits 1 and 2: it is no commit, and every
    // commit below it is one row further down.
    let placed = graph::LogRows::new(
        true,
        vec![graph::StashSlot {
            before: 1,
            stash: 0,
        }],
    );
    let len = 5; // WIP, 1, stash, 2, 3
    assert_eq!(
        log_commit_at_row(&graph, &placed, area, 0, len, 3),
        Some(oid(1))
    );
    assert_eq!(
        log_commit_at_row(&graph, &placed, area, 0, len, 4),
        None,
        "row 4 is the stash row, which has no commit"
    );
    assert_eq!(
        log_commit_at_row(&graph, &placed, area, 0, len, 5),
        Some(oid(2))
    );
    assert_eq!(
        log_commit_at_row(&graph, &placed, area, 0, len, 6),
        Some(oid(3))
    );
}

/// Revert and `reset --hard` are the two that stop at the confirm
/// modal, and they submit the action the modal promises - a mismatch
/// here is a prompt that says one thing and does another.
#[test]
fn only_revert_and_reset_hard_go_through_the_confirm_modal() {
    use gitpoulpi_core::domain::ResetMode;
    let target = oid(1);

    for action in [
        LogContextAction::CherryPick,
        LogContextAction::Reset(ResetMode::Soft),
        LogContextAction::Reset(ResetMode::Mixed),
    ] {
        assert!(
            log_action_confirmation(action, target).is_none(),
            "{action:?} must not ask: it loses no work"
        );
    }

    let (prompt, pending) = log_action_confirmation(LogContextAction::Revert, target)
        .expect("revert rewrites the working tree");
    assert!(prompt.contains("Revert"), "got: {prompt}");
    assert!(matches!(
        pending.into_action(),
        Action::Revert { commit: o, mainline: None } if o == target
    ));

    let (prompt, pending) =
        log_action_confirmation(LogContextAction::Reset(ResetMode::Hard), target)
            .expect("reset --hard discards the working tree");
    assert!(prompt.contains("cannot be undone"), "got: {prompt}");
    assert!(matches!(
        pending.into_action(),
        Action::Reset { target: t, mode: ResetMode::Hard } if t == target
    ));
}

/// Drawn through the shipped renderer, not a copy of its geometry:
/// every row the menu paints must be clickable, and the click must
/// land on the row whose label is under the pointer. Driven at the
/// top level *and* inside the Reset submenu, because the drill-in is
/// where a second, hand-rolled geometry would have crept in.
#[test]
fn every_row_the_commit_menu_paints_is_hit_testable_at_its_own_label() {
    use ratatui::backend::TestBackend;

    let ctx = MenuContext {
        current_branch: Some("main".into()),
        ..Default::default()
    };
    let submenus: Vec<usize> = commit_menu_entries(&ctx, false)
        .iter()
        .enumerate()
        .filter(|(_, e)| matches!(e, context_menu::Entry::Submenu { .. }))
        .map(|(i, _)| i)
        .collect();
    assert_eq!(
        submenus.len(),
        2 + usize::from(crate::build_features::HISTORY_REWRITE),
        "Reset, Copy and Rewrite history must be submenus"
    );
    let mut depths: Vec<Option<usize>> = vec![None];
    depths.extend(submenus.into_iter().map(Some));
    for drill_into in depths {
        let mut menu = ContextMenu::new(
            &oid(1).to_hex()[..SHORT_SHA_LEN],
            commit_menu_entries(&ctx, false),
            4,
            6,
        );
        if let Some(row) = drill_into {
            assert_eq!(
                menu.activate(row),
                MenuOutcome::Stay,
                "row {row} is a submenu"
            );
        }
        let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
        let mut hits: Vec<(Rect, HitTarget)> = Vec::new();
        terminal
            .draw(|frame| {
                hits = menu
                    .render(frame, None)
                    .into_iter()
                    .enumerate()
                    .map(|(i, r)| (r, HitTarget::LogContextMenuItem(i)))
                    .collect();
            })
            .unwrap();
        let buf = terminal.backend().buffer().clone();

        assert_eq!(hits.len(), menu.row_count());
        for i in 0..menu.row_count() {
            let (rect, target) = &hits[i];
            assert_eq!(*target, HitTarget::LogContextMenuItem(i));
            let painted: String = (rect.x..rect.right())
                .map(|x| {
                    buf.cell((x, rect.y))
                        .expect("cell in bounds")
                        .symbol()
                        .to_string()
                })
                .collect();
            let label = menu.row_label(i);
            if menu.row_is_selectable(i) {
                // The label, then (when the row has a key twin) the
                // effective key right-aligned at the row's end.
                let hint = menu.row_hint(i);
                let ok = if hint.is_empty() {
                    painted.trim_end() == label
                } else {
                    painted.starts_with(&label) && painted.trim_end().ends_with(&hint)
                };
                assert!(
                    ok,
                    "row {i} registers over {painted:?}, not over its own label {label:?}"
                );
            } else {
                // A separator: a rule to the popup's inner width, and
                // clicking it must leave the menu exactly as it was.
                assert!(
                    painted.chars().all(|c| c == '\u{2500}'),
                    "row {i} is a separator but painted {painted:?}"
                );
                let before = menu.row_count();
                assert_eq!(menu.clone().activate(i), MenuOutcome::Stay);
                assert_eq!(menu.row_count(), before);
            }
            // And the click really resolves there, through the same
            // first-match lookup the event loop uses.
            assert_eq!(
                find_hit_target(&hits, rect.x, rect.y),
                Some(HitTarget::LogContextMenuItem(i))
            );
        }

        // The caption names the commit every row acts on, at every
        // depth -- the drill-in hides the parent list, not the
        // subject.
        let title: String = (0..80)
            .map(|x| buf.cell((x, hits[0].0.y - 1)).unwrap().symbol().to_string())
            .collect();
        assert!(
            title.contains(&oid(1).to_hex()[..SHORT_SHA_LEN]),
            "the menu must say which commit it acts on; got {title:?}"
        );
    }
}

/// The six cells the user asked for, read off the commit the Log
/// already holds. A field that silently copied the wrong cell is
/// indistinguishable from a working one until it is pasted.
#[test]
fn every_commit_field_copies_its_own_cell() {
    let node = gitpoulpi_core::domain::CommitNode {
        id: oid(0xAB),
        parents: vec![],
        summary: "a summary".into(),
        message: "a summary\n\nand a body".into(),
        author_name: "Ada Lovelace".into(),
        author_email: "ada@example.com".into(),
        timestamp: 0,
        refs: vec![],
    };
    assert_eq!(
        CommitField::Message.value(&node),
        "a summary\n\nand a body",
        "Copy Message must take the whole message, not the one-line summary"
    );
    assert_eq!(CommitField::AuthorName.value(&node), "Ada Lovelace");
    assert_eq!(CommitField::AuthorEmail.value(&node), "ada@example.com");
    assert_eq!(
        CommitField::Date.value(&node),
        format_timestamp_full(node.timestamp)
    );
    let full = node.id.to_hex();
    assert_eq!(CommitField::LongSha.value(&node), full);
    assert_eq!(CommitField::ShortSha.value(&node), full[..SHORT_SHA_LEN]);
    assert_ne!(CommitField::ShortSha.value(&node), full);
}

/// The names involved are spelled into the row.
/// A menu that says "Merge into current" makes the user check which
/// branch they are on before clicking, which is exactly the moment a
/// merge goes to the wrong place.
#[test]
fn menu_labels_name_the_branches_actually_involved() {
    let ctx = MenuContext {
        current_branch: Some("main".into()),
        ..Default::default()
    };
    fn labels<A>(entries: Vec<context_menu::Entry<A>>) -> Vec<String> {
        entries
            .iter()
            .filter_map(|e| match e {
                context_menu::Entry::Action { label, .. }
                | context_menu::Entry::Submenu { label, .. } => Some(label.clone()),
                context_menu::Entry::Separator | context_menu::Entry::Disabled { .. } => None,
            })
            .collect()
    }

    let commit = labels(commit_menu_entries(&ctx, false));
    assert!(
        commit
            .iter()
            .any(|l| l.contains("Reset main to this commit")),
        "the reset submenu must name the branch it would move: {commit:?}"
    );
    assert!(
        commit.iter().any(|l| l.contains("Cherry-pick onto main")),
        "{commit:?}"
    );

    let branch = labels(ref_menu_entries(
        SidebarSection::Local,
        "feature",
        &ctx,
        false,
        false,
        false,
    ));
    assert!(
        branch.iter().any(|l| l.contains("Merge feature into main")),
        "{branch:?}"
    );
    assert!(
        branch
            .iter()
            .any(|l| l.contains("Rebase main onto feature")),
        "{branch:?}"
    );
    assert!(
        branch.iter().any(|l| l.contains("Delete feature")),
        "{branch:?}"
    );

    // A detached HEAD has no branch name, and must say `HEAD` rather
    // than an empty gap or a name it made up.
    let detached = labels(commit_menu_entries(&MenuContext::default(), false));
    assert!(
        detached
            .iter()
            .any(|l| l.contains("Reset HEAD to this commit")),
        "{detached:?}"
    );
}

/// The three Reset rows moved into a submenu, and the top level kept
/// the rows the user reaches for. A regression here is the menu
/// silently going flat again.
#[test]
fn the_resets_live_in_a_submenu_and_the_top_level_stays_short() {
    let entries = commit_menu_entries(&MenuContext::default(), false);
    // 18 since Split commit joined Recover lost commits, Create annotated
    // tag and Copy link to commit: sixteen rows and two rules. The three
    // resets would make it twenty-one.
    // 19 with the Rewrite history submenu, 20 with Amend.
    assert!(
        entries.len() <= 19 + usize::from(crate::build_features::HISTORY_REWRITE),
        "the top level grew back to {} rows",
        entries.len()
    );
    assert!(
        !entries.iter().any(|e| matches!(
            e,
            context_menu::Entry::Action {
                action: LogContextAction::Reset(_),
                ..
            }
        )),
        "a Reset row is still flat at the top level"
    );
    let resets = entries
        .iter()
        .filter_map(|e| match e {
            context_menu::Entry::Submenu { items, .. } => Some(items),
            _ => None,
        })
        .flatten()
        .filter(|e| {
            matches!(
                e,
                context_menu::Entry::Action {
                    action: LogContextAction::Reset(_),
                    ..
                }
            )
        })
        .count();
    assert_eq!(resets, 3, "all three reset modes must stay reachable");
}

/// Create patch from this commit is disabled on a merge (its
/// `PatchSource::Commits` would refuse it); Create patch to HEAD is
/// disabled on a merge and on a detached HEAD (no branch to reach).
#[test]
fn create_patch_is_disabled_on_a_merge_and_off_head() {
    fn label_of(
        entries: &[context_menu::Entry<LogContextAction>],
        wanted: LogContextAction,
    ) -> Option<String> {
        entries.iter().find_map(|e| match e {
            context_menu::Entry::Action { label, action, .. } if *action == wanted => {
                Some(label.clone())
            }
            context_menu::Entry::Disabled { label }
                if label.starts_with(match wanted {
                    LogContextAction::CreatePatch => "Create patch from this commit",
                    LogContextAction::CreatePatchToHead => "Create patch from here to",
                    _ => unreachable!(),
                }) =>
            {
                Some(label.clone())
            }
            _ => None,
        })
    }

    let attached = MenuContext {
        current_branch: Some("main".into()),
        ..Default::default()
    };
    let detached = MenuContext::default();

    // Not a merge, HEAD attached: both are live rows.
    let entries = commit_menu_entries(&attached, false);
    assert!(matches!(
        entries
            .iter()
            .find(|e| label_of(std::slice::from_ref(e), LogContextAction::CreatePatch).is_some()),
        Some(context_menu::Entry::Action { .. })
    ));
    assert!(matches!(
        entries.iter().find(|e| label_of(
            std::slice::from_ref(e),
            LogContextAction::CreatePatchToHead
        )
        .is_some()),
        Some(context_menu::Entry::Action { .. })
    ));

    // A merge commit: both rows disable, and say why.
    let merge_entries = commit_menu_entries(&attached, true);
    let disabled_here =
        label_of(&merge_entries, LogContextAction::CreatePatch).expect("row exists");
    assert!(disabled_here.contains("merge"), "{disabled_here}");
    let disabled_to_head =
        label_of(&merge_entries, LogContextAction::CreatePatchToHead).expect("row exists");
    assert!(disabled_to_head.contains("merge"), "{disabled_to_head}");

    // Detached HEAD, not a merge: only the range row disables.
    let detached_entries = commit_menu_entries(&detached, false);
    assert!(matches!(
        detached_entries.iter().find(|e| label_of(
            std::slice::from_ref(e),
            LogContextAction::CreatePatch
        )
        .is_some()),
        Some(context_menu::Entry::Action { .. })
    ));
    let detached_to_head =
        label_of(&detached_entries, LogContextAction::CreatePatchToHead).expect("row exists");
    assert!(detached_to_head.contains("detached"), "{detached_to_head}");
}

/// The commit menu offers Edit commit message, and choosing it opens
/// the popup on the commit's full message - knowing it is pushed when
/// it is on the upstream - or explains why it cannot.
#[test]
fn edit_commit_message_opens_on_the_full_message_or_says_why_not() {
    let entries = commit_menu_entries(&MenuContext::default(), false);
    assert!(entries.iter().any(|e| matches!(
        e,
        context_menu::Entry::Action {
            action: LogContextAction::EditMessage,
            ..
        }
    )));

    // 1 <- 2 <- 3 (main, HEAD, upstream origin/main at 2); 9 off 1.
    let mut two = commit(2, &[1]);
    two.message = "commit 2\n\nits body".into();
    let graph = Ok(gitpoulpi_core::domain::CommitGraph {
        nodes: vec![commit(3, &[2]), commit(9, &[1]), two, commit(1, &[])],
    });
    let mut main = local_branch_at("main", oid(3));
    main.is_head = true;
    main.upstream = Some("origin/main".into());
    let refs = Ok(vec![main, remote_branch_at("origin", "main", oid(2))]);

    let form = reword_form_for(&graph, &refs, oid(2)).expect("2 is on main");
    assert_eq!(form.text(), "commit 2\n\nits body");
    assert_eq!(form.pushed_to(), Some("origin/main"));
    let head = reword_form_for(&graph, &refs, oid(3)).expect("HEAD is on main");
    assert_eq!(head.pushed_to(), None, "3 is not on origin/main yet");

    let off = reword_form_for(&graph, &refs, oid(9)).unwrap_err();
    assert!(off.contains("not on the current branch"), "{off}");
    let gone = reword_form_for(&graph, &refs, oid(42)).unwrap_err();
    assert!(gone.contains("no longer in the loaded graph"), "{gone}");

    // The off-branch question's list: the other local branches whose
    // history holds it, sorted; remote branches and the current one are not.
    let mut topic = local_branch_at("topic", oid(9));
    topic.is_head = false;
    let mut main = local_branch_at("main", oid(3));
    main.is_head = true;
    let refs = vec![
        main,
        topic,
        local_branch_at("alpha", oid(9)),
        local_branch_at("other", oid(3)),
        remote_branch_at("origin", "topic", oid(9)),
    ];
    let held = crate::reword_ui::other_branches_holding(graph.as_ref().unwrap(), &refs, oid(9));
    assert_eq!(held, vec!["alpha".to_string(), "topic".to_string()]);
    let none = crate::reword_ui::other_branches_holding(graph.as_ref().unwrap(), &refs, oid(42));
    assert!(none.is_empty(), "{none:?}");
}

/// Regression: "Interactively rebase from here" listed the commits
/// *after* the clicked one but sent the clicked commit's *parent* as
/// the upstream, so the backend's range (one commit longer than the
/// plan) refused every Start with "the plan is not exactly a
/// reordering". The upstream is the clicked commit itself.
#[test]
fn rebase_from_here_uses_the_clicked_commit_as_the_base() {
    let graph = Ok(gitpoulpi_core::domain::CommitGraph {
        nodes: vec![
            commit(4, &[3]),
            commit(3, &[2]),
            commit(2, &[1]),
            commit(1, &[]),
        ],
    });
    let mut main = local_branch_at("main", oid(4));
    main.is_head = true;
    let refs = Ok(vec![main]);
    let form = rebase_form_for(&graph, &refs, oid(2)).expect("2 is on main");
    assert_eq!(form.upstream, Some(oid(2)));
    let planned: Vec<Oid> = form.steps().iter().map(|s| s.commit).collect();
    assert_eq!(planned, vec![oid(3), oid(4)], "oldest first, base excluded");
}

/// The advanced rewrite sits in one drill-in submenu: one commit offers "from
/// this commit to HEAD" and the workbench, a multi-selection "Selected
/// commits" and the workbench. Each row opens the workbench with its scope.
#[test]
fn the_rewrite_history_submenu_names_the_scope_each_row_opens() {
    if !crate::build_features::HISTORY_REWRITE {
        return;
    }
    use crate::log_menu::RewriteFrom;
    fn rewrite_rows(
        entries: Vec<context_menu::Entry<LogContextAction>>,
    ) -> Vec<(String, LogContextAction)> {
        entries
            .into_iter()
            .find_map(|e| match e {
                context_menu::Entry::Submenu { label, items }
                    if label.contains("Rewrite history") =>
                {
                    Some(items)
                }
                _ => None,
            })
            .expect("a Rewrite history submenu")
            .into_iter()
            .filter_map(|e| match e {
                context_menu::Entry::Action { label, action, .. } => Some((label, action)),
                _ => None,
            })
            .collect()
    }
    let one = rewrite_rows(commit_menu_entries(&MenuContext::default(), false));
    assert_eq!(
        one.iter().map(|(_, a)| *a).collect::<Vec<_>>(),
        vec![
            LogContextAction::RewriteHistory(RewriteFrom::FromHere),
            LogContextAction::RewriteHistory(RewriteFrom::Workbench)
        ]
    );
    assert!(one[0].0.contains("to HEAD"), "{:?}", one[0].0);
    let many = rewrite_rows(commit_menu_entries(
        &MenuContext::default().with_selected(3),
        false,
    ));
    assert_eq!(
        many[0].1,
        LogContextAction::RewriteHistory(RewriteFrom::Selected)
    );
    assert!(many[0].0.contains("Selected"), "{:?}", many[0].0);
}

/// The commit menu offers Amend, and choosing it opens the dialog with the
/// commit's message and author filled in - or says why it cannot.
#[test]
fn amend_opens_the_dialog_prefilled_with_the_message_and_the_author() {
    let entries = commit_menu_entries(&MenuContext::default(), false);
    assert!(entries.iter().any(|e| matches!(
        e,
        context_menu::Entry::Action {
            action: LogContextAction::Amend(AmendScope::All),
            ..
        }
    )));

    let mut two = commit(2, &[1]);
    two.message = "commit 2\n\nits body".into();
    two.author_name = "Ada".into();
    two.author_email = "ada@example.org".into();
    let graph = Ok(gitpoulpi_core::domain::CommitGraph {
        nodes: vec![commit(3, &[2]), commit(9, &[1]), two, commit(1, &[])],
    });
    let mut main = local_branch_at("main", oid(3));
    main.is_head = true;
    let refs = Ok(vec![main]);

    let form = amend_form_for(&graph, &refs, oid(2), AmendScope::All).expect("2 is on main");
    let author = amend_form_for(&graph, &refs, oid(2), AmendScope::Author).unwrap();
    assert_eq!(author.focus, RewordPart::AuthorName);
    assert_eq!(form.text(), "commit 2\n\nits body");
    // Off the branch the backend would refuse: the Log says so first.
    let why = amend_form_for(&graph, &refs, oid(9), AmendScope::All).unwrap_err();
    assert!(why.contains("not on the current branch"), "{why}");
}

/// A right click on a property of the commit-detail pane offers that
/// property's Amend row first, the whole popup last.
#[test]
fn the_detail_pane_menu_leads_with_the_property_it_was_opened_on() {
    let actions = |scope| -> Vec<LogContextAction> {
        detail_part_menu_entries(scope)
            .into_iter()
            .filter_map(|e| match e {
                context_menu::Entry::Action { action, .. } => Some(action),
                _ => None,
            })
            .collect()
    };
    assert_eq!(
        actions(AmendScope::Summary),
        vec![
            LogContextAction::Amend(AmendScope::Summary),
            LogContextAction::EditMessage,
            LogContextAction::Amend(AmendScope::All),
        ]
    );
    assert_eq!(
        actions(AmendScope::Description)[0],
        LogContextAction::Amend(AmendScope::Description)
    );
    assert_eq!(
        actions(AmendScope::Author),
        vec![
            LogContextAction::Amend(AmendScope::Author),
            LogContextAction::Amend(AmendScope::All),
        ]
    );
}
