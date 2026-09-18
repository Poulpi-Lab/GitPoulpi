//! The Home page's context menus: a workspace's, and one of its
//! repositories'. They ride on the repository menu's machinery
//! (`RepoMenuState`, one menu slot, one set of key and mouse handlers), so
//! their rows are `RepoMenuAction::Ws(..)`.

use gitpoulpi_i18n::msg::workspaces as m;

use crate::context_menu::Entry;
use crate::repo_menu::{RepoMenuAction, WsMenu};

/// Rows for a workspace (`repo: None`) or for one of its repositories
/// (`Some((present, has_remote))`).
pub(crate) fn entries(has_missing: bool, repo: Option<(bool, bool)>) -> Vec<Entry<RepoMenuAction>> {
    use WsMenu as W;
    let row = |label: &str, w: W| Entry::action(label, RepoMenuAction::Ws(w));
    match repo {
        None => {
            let mut rows = vec![
                row(m::MENU_OPEN.get(), W::Open),
                row(m::MENU_EDIT.get(), W::Edit),
                Entry::Separator,
                row(m::MENU_FETCH_ALL.get(), W::FetchAll),
                row(m::MENU_PULL_ALL.get(), W::PullAll),
            ];
            if has_missing {
                rows.push(row(m::MENU_CLONE_MISSING.get(), W::CloneMissing));
            }
            rows.push(Entry::Separator);
            rows.push(Entry::danger(
                m::MENU_DELETE.get(),
                RepoMenuAction::Ws(W::Delete),
            ));
            rows
        }
        Some((present, has_remote)) => {
            let mut rows = Vec::new();
            if present {
                rows.push(row(gitpoulpi_i18n::msg::home::ROW_OPEN.get(), W::OpenRepo));
            } else {
                rows.push(row(m::MENU_LOCATE.get(), W::Locate));
                if has_remote {
                    rows.push(row(m::MENU_CLONE_REPO.get(), W::CloneRepo));
                }
            }
            rows.push(Entry::Separator);
            rows.push(row(m::MENU_REMOVE_FROM.get(), W::RemoveRepo));
            rows
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn labels(entries: &[Entry<RepoMenuAction>]) -> Vec<String> {
        entries
            .iter()
            .filter_map(|e| match e {
                Entry::Action { label, .. } => Some(label.clone()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn a_workspace_menu_offers_clone_missing_only_when_something_is_missing() {
        let with = labels(&entries(true, None));
        let without = labels(&entries(false, None));
        assert!(with.contains(&"Clone missing".to_string()), "{with:?}");
        assert!(
            !without.contains(&"Clone missing".to_string()),
            "{without:?}"
        );
        assert_eq!(with.last().map(String::as_str), Some("Delete..."));
    }

    #[test]
    fn a_missing_repository_offers_locate_and_clone_only_with_a_recorded_remote() {
        let cloneable = labels(&entries(true, Some((false, true))));
        assert!(cloneable.iter().any(|l| l == "Locate..."), "{cloneable:?}");
        assert!(cloneable.iter().any(|l| l == "Clone"), "{cloneable:?}");
        let no_remote = labels(&entries(true, Some((false, false))));
        assert!(!no_remote.iter().any(|l| l == "Clone"), "{no_remote:?}");
        let present = labels(&entries(false, Some((true, true))));
        assert!(present.iter().any(|l| l == "Open"), "{present:?}");
        assert!(!present.iter().any(|l| l == "Locate..."), "{present:?}");
    }
}
