use super::*;

/// AI rows exist only while AI is on: hidden, not grayed out.
#[test]
fn the_ai_explain_rows_appear_only_when_ai_is_enabled() {
    fn has_ai<A>(entries: &[context_menu::Entry<A>]) -> bool {
        entries.iter().any(
            |e| matches!(e, context_menu::Entry::Action { label, .. } if label.contains("(AI)")),
        )
    }
    let off = MenuContext::default();
    let on = MenuContext::default().with_ai(true);
    assert!(!has_ai(&commit_menu_entries(&off, false)));
    assert!(has_ai(&commit_menu_entries(&on, false)));
    assert!(!has_ai(&wip_menu_entries(true, false)));
    assert!(has_ai(&wip_menu_entries(true, true)));
}

/// Maintainer rule: every menu entry that triggers AI carries the sparkle.
#[test]
fn every_ai_menu_entry_carries_the_sparkle() {
    use context_menu::Entry;
    let commit = commit_menu_entries(&MenuContext::default().with_ai(true), false);
    let wip = wip_menu_entries(true, true);
    let mut seen = 0;
    for e in &commit {
        if let Entry::Action {
            label,
            action: LogContextAction::AiExplain | LogContextAction::AiSplitCommit,
            ..
        } = e
        {
            assert!(label.starts_with(crate::theme::AI_SPARK), "{label}");
            seen += 1;
        }
    }
    for e in &wip {
        if let Entry::Action {
            label,
            action: WipContextAction::AiExplain | WipContextAction::AiCompose,
            ..
        } = e
        {
            assert!(label.starts_with(crate::theme::AI_SPARK), "{label}");
            seen += 1;
        }
    }
    assert_eq!(seen, 4);
}
