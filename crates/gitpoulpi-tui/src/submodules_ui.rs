//! The SUBMODULES section of the Refs sidebar: one row per submodule with
//! its state, and the helpers its actions share (open as a tab, the
//! selected path). The list comes from the daemon's `Submodules` read; the
//! actions (`Update`, `Init`) go back through it as `Action`s, so a
//! change made from the CLI or an agent shows up here too.
//!
//! Keyboard: `Enter` opens the selected submodule as its own tab (the
//! worktree rows' double-click twin); the palette reaches Update, Init,
//! Update all and Open without the mouse (`Refs: Submodule ...`).

use super::*;
use gitpoulpi_i18n::msg::submodules_ui as msg;

use gitpoulpi_core::domain::{Submodule, SubmoduleState};

/// The state's glyph and color: a tick when the submodule sits where the
/// repository records it, an arrow when it has moved, a ring when it is
/// not cloned yet, a cross on conflicts.
pub(crate) fn state_glyph(state: SubmoduleState) -> (&'static str, Color) {
    match state {
        SubmoduleState::UpToDate => ("\u{2713}", th().refs.local),
        SubmoduleState::Modified => ("\u{2191}", th().status.warning),
        SubmoduleState::Uninitialized => ("\u{25CB}", th().ui.muted),
        SubmoduleState::Conflicted => ("\u{2717}", th().status.danger),
    }
}

/// The section's rows: `<glyph> <path>  <short sha>`.
pub(crate) fn submodule_items(list: &[Submodule], width: u16) -> Vec<ListItem<'static>> {
    list.iter()
        .map(|s| {
            let (glyph, color) = state_glyph(s.state);
            let sha = s
                .head
                .or(s.expected)
                .map(|o| o.to_hex()[..7].to_string())
                .unwrap_or_default();
            let label = if sha.is_empty() {
                s.path.clone()
            } else {
                format!("{}  {sha}", s.path)
            };
            ListItem::new(Line::from(vec![
                Span::raw(" "),
                Span::styled(glyph, Style::default().fg(color)),
                Span::raw(" "),
                Span::styled(
                    truncate_with_ellipsis(&label, width.saturating_sub(6) as usize),
                    Style::default().fg(th().ui.text),
                ),
            ]))
        })
        .collect()
}

/// The path (repo-relative) of the selected row.
pub(crate) fn selected_path(list: &[Submodule], selected: Option<usize>) -> Option<String> {
    selected.and_then(|i| list.get(i)).map(|s| s.path.clone())
}

/// `Enter` / the palette / the menu's "Open in new tab": the submodule's
/// working tree as a repository tab of its own (focusing the one it
/// already has). A submodule that is not cloned has nothing to open.
pub(crate) fn open_submodule_in_tab(
    list: &[Submodule],
    selected: Option<usize>,
    root: &Path,
    ctx: &mut TabCtx,
) -> Option<String> {
    let Some(sub) = selected.and_then(|i| list.get(i)) else {
        return Some(msg::SELECT_FIRST.get().into());
    };
    if sub.state == SubmoduleState::Uninitialized {
        return Some(msg::not_cloned(&sub.path));
    }
    ctx.switcher_action(repo_switcher::SwitcherAction::Open(root.join(&sub.path)))
}

/// [`open_submodule_in_tab`] by path, for the context menu (which carries
/// the row's path, not its index): the folder to open, or why not.
pub(crate) fn open_target(list: &[Submodule], root: &Path, path: &str) -> Result<PathBuf, String> {
    match list.iter().find(|s| s.path == path) {
        None => Err(msg::no_longer(path)),
        Some(s) if s.state == SubmoduleState::Uninitialized => Err(msg::not_cloned(path)),
        Some(_) => Ok(root.join(path)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitpoulpi_core::domain::Oid;

    fn sub(path: &str, state: SubmoduleState) -> Submodule {
        Submodule {
            name: path.into(),
            path: path.into(),
            url: "../x".into(),
            branch: None,
            state,
            expected: Oid::from_hex(&"a".repeat(40)),
            head: (state != SubmoduleState::Uninitialized)
                .then(|| Oid::from_hex(&"b".repeat(40)))
                .flatten(),
        }
    }

    #[test]
    fn every_state_has_its_own_glyph() {
        let glyphs: Vec<&str> = [
            SubmoduleState::UpToDate,
            SubmoduleState::Modified,
            SubmoduleState::Uninitialized,
            SubmoduleState::Conflicted,
        ]
        .into_iter()
        .map(|s| state_glyph(s).0)
        .collect();
        let unique: std::collections::HashSet<_> = glyphs.iter().collect();
        assert_eq!(unique.len(), 4);
    }

    #[test]
    fn a_row_shows_the_checked_out_commit_or_else_the_recorded_one() {
        let rows = submodule_items(
            &[
                sub("libs/a", SubmoduleState::UpToDate),
                sub("libs/b", SubmoduleState::Uninitialized),
            ],
            40,
        );
        assert_eq!(rows.len(), 2);
    }

    #[test]
    fn only_a_cloned_submodule_can_be_opened() {
        let list = [
            sub("libs/a", SubmoduleState::UpToDate),
            sub("libs/b", SubmoduleState::Uninitialized),
        ];
        let root = Path::new("/repo");
        assert_eq!(
            open_target(&list, root, "libs/a").unwrap(),
            Path::new("/repo/libs/a")
        );
        assert!(open_target(&list, root, "libs/b")
            .unwrap_err()
            .contains("not cloned"));
        assert!(open_target(&list, root, "gone").is_err());
    }
}
