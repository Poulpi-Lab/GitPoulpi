//! Whose choice the left pane's show/hide set is: this repository's own, or
//! the global default (`[ui] sidebar_sections_hidden`).
//!
//! The window keeps one effective hidden set (what is painted); this struct
//! remembers where it came from and what was last written, so the tick can
//! save a change to the right place: the repository's state file for a
//! per-repository choice, `config.toml` only after "Use for all
//! repositories". A repository with no choice of its own follows the default.
//!
//! Under `ui.sidebar_sections_scope = global` the window follows the default
//! only and every toggle writes it; the repository's own choice is kept (in
//! memory and in its state file) for when the setting is switched back.

use crate::refs_sidebar::{hidden_from_names, hidden_to_names, SidebarSection};
use gitpoulpi_session::RepoState;
use std::collections::HashSet;
use std::path::Path;

#[derive(Debug, Clone, Default)]
pub(crate) struct SectionsScope {
    /// The global default, as loaded from config or set by "Use for all" (and,
    /// under the `global` scope, by every toggle).
    pub(crate) default: HashSet<SidebarSection>,
    /// This repository's own choice, when it has one. Kept (and saved) while
    /// the `global` scope is on, so switching back loses nothing.
    own_set: Option<HashSet<SidebarSection>>,
    /// `ui.sidebar_sections_scope = global`: the window follows `default`.
    global: bool,
    saved_default: HashSet<SidebarSection>,
    saved_own: Option<Vec<String>>,
}

impl SectionsScope {
    /// The effective hidden set and the scope for a repository: its own
    /// stored choice when it has one, else the default; always the default
    /// under the `global` scope.
    pub(crate) fn resolve(
        default: HashSet<SidebarSection>,
        own: Option<&[String]>,
        global: bool,
    ) -> (HashSet<SidebarSection>, Self) {
        let scope = Self {
            saved_default: default.clone(),
            default,
            own_set: own.map(hidden_from_names),
            global,
            saved_own: own.map(|names| hidden_to_names(&hidden_from_names(names))),
        };
        (scope.active(), scope)
    }

    /// The set the window shows under the current scope.
    fn active(&self) -> HashSet<SidebarSection> {
        if self.global {
            self.default.clone()
        } else {
            self.own_set.clone().unwrap_or_else(|| self.default.clone())
        }
    }

    /// Whether this repository has a choice of its own.
    #[cfg(test)]
    pub(crate) fn has_own(&self) -> bool {
        self.own_set.is_some()
    }

    /// Whether the `global` scope is on.
    pub(crate) fn is_global(&self) -> bool {
        self.global
    }

    /// A toggle was made: the set now on screen is stored where the scope
    /// says (the default when global, else this repository's own).
    pub(crate) fn touch(&mut self, hidden: &HashSet<SidebarSection>) {
        if self.global {
            self.default.clone_from(hidden);
        } else {
            self.own_set = Some(hidden.clone());
        }
    }

    /// "Reset to default": drop this repository's choice (under the `global`
    /// scope, show every section).
    pub(crate) fn reset(&mut self, hidden: &mut HashSet<SidebarSection>) {
        if self.global {
            hidden.clear();
            self.default.clear();
        } else {
            self.own_set = None;
            hidden.clone_from(&self.default);
        }
    }

    /// "Use for all repositories": the current set becomes the default.
    pub(crate) fn use_for_all(&mut self, hidden: &HashSet<SidebarSection>) {
        self.default.clone_from(hidden);
        if !self.global {
            self.own_set = Some(hidden.clone());
        }
    }

    /// The setting flipped while the window is open: swap in the other set.
    /// Nothing is dropped, each slot keeps its own value.
    pub(crate) fn follow_scope(&mut self, hidden: &mut HashSet<SidebarSection>, global: bool) {
        if global != self.global {
            self.global = global;
            *hidden = self.active();
        }
    }

    /// What changed since the last save: `(default to write, own to write)`.
    /// The own choice is `Some(None)` when it was dropped.
    #[allow(clippy::type_complexity)]
    pub(crate) fn take_changes(
        &mut self,
    ) -> (Option<HashSet<SidebarSection>>, Option<Option<Vec<String>>>) {
        let default = (self.default != self.saved_default).then(|| {
            self.saved_default.clone_from(&self.default);
            self.default.clone()
        });
        let own_now = self.own_set.as_ref().map(hidden_to_names);
        let own = (own_now != self.saved_own).then(|| {
            self.saved_own.clone_from(&own_now);
            own_now
        });
        (default, own)
    }
}

/// Writes (or drops, with `None`) the repository's own choice, keeping the
/// rest of its state file.
pub(crate) fn save_own(state_path: &Path, own: Option<Vec<String>>) {
    let mut state = RepoState::load(state_path).unwrap_or_default();
    state.sidebar_sections_hidden = own;
    let _ = state.save(state_path);
}

/// The repository's stored choice, if it made one.
pub(crate) fn load_own(state_path: &Path) -> Option<Vec<String>> {
    RepoState::load(state_path)
        .ok()
        .and_then(|s| s.sidebar_sections_hidden)
}

#[cfg(test)]
mod tests {
    use super::*;
    use SidebarSection::{Remote, Tags};

    fn set(s: &[SidebarSection]) -> HashSet<SidebarSection> {
        s.iter().copied().collect()
    }

    #[test]
    fn no_choice_of_its_own_follows_the_default() {
        let (hidden, scope) = SectionsScope::resolve(set(&[Tags]), None, false);
        assert_eq!(hidden, set(&[Tags]));
        assert!(!scope.has_own());
    }

    #[test]
    fn an_own_choice_overrides_the_default_even_when_empty() {
        let (hidden, scope) =
            SectionsScope::resolve(set(&[Tags]), Some(&["remote".to_string()]), false);
        assert_eq!(hidden, set(&[Remote]));
        assert!(scope.has_own());
        let (hidden, _) = SectionsScope::resolve(set(&[Tags]), Some(&[]), false);
        assert!(hidden.is_empty());
    }

    #[test]
    fn reset_drops_the_own_choice_and_returns_to_the_default() {
        let (mut hidden, mut scope) =
            SectionsScope::resolve(set(&[Tags]), Some(&["remote".to_string()]), false);
        scope.reset(&mut hidden);
        assert_eq!(hidden, set(&[Tags]));
        let (default, own) = scope.take_changes();
        assert_eq!(default, None);
        assert_eq!(own, Some(None), "the stored choice is dropped");
    }

    #[test]
    fn use_for_all_sets_the_default_from_the_current_set() {
        let (mut hidden, mut scope) = SectionsScope::resolve(HashSet::new(), None, false);
        hidden.insert(Tags);
        scope.touch(&hidden);
        scope.use_for_all(&hidden);
        let (default, own) = scope.take_changes();
        assert_eq!(default, Some(set(&[Tags])));
        assert_eq!(own, Some(Some(vec!["tags".to_string()])));
        assert_eq!(scope.take_changes(), (None, None), "saved once");
    }

    #[test]
    fn a_toggle_alone_writes_only_the_repository_choice() {
        let (mut hidden, mut scope) = SectionsScope::resolve(HashSet::new(), None, false);
        hidden.insert(Tags);
        scope.touch(&hidden);
        let (default, own) = scope.take_changes();
        assert_eq!(default, None);
        assert_eq!(own, Some(Some(vec!["tags".to_string()])));
    }

    #[test]
    fn the_global_scope_shows_the_default_and_a_toggle_writes_only_the_default() {
        let (mut hidden, mut scope) =
            SectionsScope::resolve(set(&[Tags]), Some(&["remote".to_string()]), true);
        assert_eq!(
            hidden,
            set(&[Tags]),
            "the repository's own choice is not used"
        );
        assert!(scope.is_global());
        hidden.insert(Remote);
        scope.touch(&hidden);
        let (default, own) = scope.take_changes();
        assert_eq!(default, Some(set(&[Tags, Remote])));
        assert_eq!(own, None, "the repository's own choice is untouched");
    }

    #[test]
    fn switching_the_scope_live_swaps_the_sets_and_loses_neither() {
        let (mut hidden, mut scope) =
            SectionsScope::resolve(set(&[Tags]), Some(&["remote".to_string()]), false);
        assert_eq!(hidden, set(&[Remote]));
        scope.follow_scope(&mut hidden, true);
        assert_eq!(hidden, set(&[Tags]));
        hidden.clear();
        scope.touch(&hidden);
        scope.follow_scope(&mut hidden, false);
        assert_eq!(hidden, set(&[Remote]), "the own choice came back intact");
        scope.follow_scope(&mut hidden, true);
        assert!(hidden.is_empty(), "and so did the global one");
    }

    #[test]
    fn the_own_choice_survives_a_write_and_a_read() {
        let dir = tempfile::tempdir().unwrap();
        let path = RepoState::path_for(dir.path(), ".gitpoulpi");
        assert_eq!(load_own(&path), None);
        save_own(&path, Some(vec!["tags".into()]));
        assert_eq!(load_own(&path), Some(vec!["tags".to_string()]));
        save_own(&path, None);
        assert_eq!(load_own(&path), None);
    }
}
