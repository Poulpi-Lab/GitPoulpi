//! The left pane's show/hide checklist: one checkable row per section, then
//! "Use for all repositories" and "Reset to default" (the choice is per
//! repository, see `sections_scope`). It is a `RefMenu` keyed by [`MENU_KEY`] (so the
//! hit targets, hover, keyboard and click handlers are the ref menu's own),
//! whose two actions run through `apply_sidebar_sections_action`: the box
//! flips and the menu stays open, the checklist convention.

use crate::context_menu::Entry;
use crate::refs_sidebar::{can_hide, SidebarSection};
use crate::RefContextAction;
use gitpoulpi_i18n::msg;
use std::collections::HashSet;

/// The `RefMenu` row identity that marks the checklist.
pub(crate) const MENU_KEY: &str = "\u{1}sidebar-sections";

/// Whether the action belongs to the checklist (it must not close the menu).
pub(crate) fn is_sections_action(action: RefContextAction) -> bool {
    matches!(
        action,
        RefContextAction::ToggleSidebarSection(_)
            | RefContextAction::ResetSidebarSections
            | RefContextAction::UseSidebarSectionsForAll
    )
}

/// The rows: every section with its box, then a rule and the reset row. The
/// only section still shown cannot be unchecked, so it is a grayed row.
pub(crate) fn entries(
    hidden: &HashSet<SidebarSection>,
    available: &[SidebarSection],
    global: bool,
) -> Vec<Entry<RefContextAction>> {
    let mut rows: Vec<Entry<RefContextAction>> = SidebarSection::ORDER
        .iter()
        .filter(|s| s.compiled_in())
        .map(|&section| {
            let checked = !hidden.contains(&section);
            let glyph = if checked { "\u{2611}" } else { "\u{2610}" };
            let label = format!("{glyph} {}", section.display_title());
            if checked && !can_hide(section, hidden, available) {
                Entry::Disabled {
                    label: format!("{label} {}", msg::log_draw::ALWAYS_SHOWN.get()),
                }
            } else {
                Entry::action(label, RefContextAction::ToggleSidebarSection(section))
            }
        })
        .collect();
    rows.push(Entry::Separator);
    // Under the global scope every toggle already applies to all of them.
    if !global {
        rows.push(Entry::action(
            msg::sidebar::SECTIONS_USE_FOR_ALL.get(),
            RefContextAction::UseSidebarSectionsForAll,
        ));
    }
    rows.push(Entry::action(
        msg::sidebar::SECTIONS_RESET.get(),
        RefContextAction::ResetSidebarSections,
    ));
    rows
}

#[cfg(test)]
mod tests {
    use super::*;

    fn labels(rows: &[Entry<RefContextAction>]) -> Vec<String> {
        rows.iter()
            .map(|r| match r {
                Entry::Action { label, .. } | Entry::Disabled { label } => label.clone(),
                _ => "---".into(),
            })
            .collect()
    }

    #[test]
    fn the_global_scope_has_no_use_for_all_row() {
        let available = [SidebarSection::Local, SidebarSection::Remote];
        let per_repo = labels(&entries(&HashSet::new(), &available, false));
        let global = labels(&entries(&HashSet::new(), &available, true));
        assert_eq!(global.len() + 1, per_repo.len());
        assert!(!global.contains(&msg::sidebar::SECTIONS_USE_FOR_ALL.get().to_string()));
    }

    #[test]
    fn every_section_is_a_checkbox_row_then_the_two_scope_rows() {
        let available = [SidebarSection::Local, SidebarSection::Remote];
        let rows = entries(&HashSet::from([SidebarSection::Tags]), &available, false);
        let text = labels(&rows);
        let shown = SidebarSection::ORDER
            .iter()
            .filter(|s| s.compiled_in())
            .count();
        assert_eq!(text.len(), shown + 3);
        assert!(text[0].starts_with('\u{2611}'), "{text:?}");
        let tags = SidebarSection::ORDER
            .iter()
            .filter(|s| s.compiled_in())
            .position(|s| *s == SidebarSection::Tags)
            .unwrap();
        assert!(
            text[tags].starts_with('\u{2610}'),
            "tags is hidden: {text:?}"
        );
        assert_eq!(text[shown], "---");
    }

    #[test]
    fn the_last_shown_section_is_a_grayed_row_that_cannot_run() {
        let available = [SidebarSection::Local, SidebarSection::Remote];
        let hidden = HashSet::from([SidebarSection::Remote]);
        let rows = entries(&hidden, &available, false);
        assert!(matches!(rows[0], Entry::Disabled { .. }), "{rows:?}");
        assert!(matches!(rows[1], Entry::Action { .. }));
    }
}
