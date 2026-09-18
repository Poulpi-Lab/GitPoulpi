use super::*;

/// The WIP row's menu offers Create patch and Apply patch by name, and
/// gates everything but Apply patch on there being changes to act on:
/// applying a patch needs no existing changes, so it alone stays live
/// on a clean tree.
#[test]
fn wip_menu_entries_gate_every_row_but_apply_patch_on_has_wip() {
    // Every row is a live `Action`, none `Disabled`, when the tree is
    // dirty.
    let dirty = wip_menu_entries(true, false);
    assert!(
        !dirty
            .iter()
            .any(|e| matches!(e, context_menu::Entry::Disabled { .. })),
        "a dirty tree must not gray out any WIP menu row: {dirty:?}"
    );
    for (action, needle) in [
        (WipContextAction::CreatePatch, "Create patch from WIP"),
        (WipContextAction::ApplyPatch, "Apply patch"),
        (WipContextAction::StashAll, "Stash all"),
        (WipContextAction::StageAll, "Stage all"),
        (WipContextAction::DiscardAll, "Discard all"),
    ] {
        assert!(
            dirty.iter().any(|e| matches!(e,
                context_menu::Entry::Action { label, action: a, .. }
                if *a == action && label.contains(needle)
            )),
            "{action:?} must be a live row containing {needle:?}: {dirty:?}"
        );
    }

    // Everything but Apply patch grays out on a clean tree, with a
    // reason; Apply patch alone stays live.
    let clean = wip_menu_entries(false, false);
    let disabled_count = clean
        .iter()
        .filter(|e| matches!(e, context_menu::Entry::Disabled { .. }))
        .count();
    assert_eq!(
        disabled_count, 4,
        "CreatePatch/StashAll/StageAll/DiscardAll must disable on a clean tree: {clean:?}"
    );
    for e in &clean {
        if let context_menu::Entry::Disabled { label } = e {
            assert!(
                label.contains("no uncommitted changes"),
                "a disabled WIP row must say why: {label}"
            );
        }
    }
    assert!(
        clean.iter().any(|e| matches!(
            e,
            context_menu::Entry::Action {
                action: WipContextAction::ApplyPatch,
                ..
            }
        )),
        "Apply patch must stay live on a clean tree: {clean:?}"
    );
}

/// Same painted-rect-matches-its-own-label guard as the commit menu's
/// own test: every row the WIP menu draws must be clickable exactly
/// where its label is drawn.
#[test]
fn every_row_the_wip_menu_paints_is_hit_testable_at_its_own_label() {
    use ratatui::backend::TestBackend;

    for has_wip in [true, false] {
        let entries = wip_menu_entries(has_wip, false);
        let is_separator: Vec<bool> = entries
            .iter()
            .map(|e| matches!(e, context_menu::Entry::Separator))
            .collect();
        let menu = ContextMenu::new("uncommitted changes", entries, 4, 6);
        let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
        let mut hits: Vec<(Rect, HitTarget)> = Vec::new();
        terminal
            .draw(|frame| {
                hits = menu
                    .render(frame, None)
                    .into_iter()
                    .enumerate()
                    .map(|(i, r)| (r, HitTarget::WipContextMenuItem(i)))
                    .collect();
            })
            .unwrap();
        let buf = terminal.backend().buffer().clone();

        assert_eq!(hits.len(), menu.row_count());
        for i in 0..menu.row_count() {
            let (rect, target) = &hits[i];
            assert_eq!(*target, HitTarget::WipContextMenuItem(i));
            let painted: String = (rect.x..rect.right())
                .map(|x| {
                    buf.cell((x, rect.y))
                        .expect("cell in bounds")
                        .symbol()
                        .to_string()
                })
                .collect();
            let label = menu.row_label(i);
            if is_separator[i] {
                assert!(
                    painted.chars().all(|c| c == '\u{2500}'),
                    "row {i} is a separator but painted {painted:?}"
                );
            } else {
                // Both live and disabled rows paint their own label --
                // a disabled row is still hit-tested, so clicking it
                // must resolve to the row that explains why it's
                // inert, not to whatever is underneath.
                assert!(
                    painted.trim_end() == label,
                    "row {i} (has_wip={has_wip}) registers over {painted:?}, not over its \
                     own label {label:?}"
                );
            }
        }
    }
}
