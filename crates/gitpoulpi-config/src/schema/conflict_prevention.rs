//! The `conflict_prevention` part of the config schema: named lists of "Base
//! branches" patterns and which one is the default.
//!
//! Which branches proactive conflict detection may compare the current branch
//! with is a list of patterns (`gitpoulpi_core::domain::conflict_prevention`).
//! A repository chooses where its list comes from (`RepoPreferences`:
//! `conflict_base_source`, `conflict_base_list`, `conflict_base_branches`);
//! [`ConflictPreventionConfig::resolve`] is the one place that turns that
//! choice into patterns, used by the daemon and the Settings pages alike.

use std::collections::BTreeMap;

use gitpoulpi_core::domain::conflict_prevention::{parse_base_globs, DEFAULT_BASE_GLOBS};
use serde::{Deserialize, Serialize};

/// A repository follows the global default list.
pub const BASE_SOURCE_DEFAULT: &str = "default";
/// A repository follows a named list (`conflict_base_list`).
pub const BASE_SOURCE_LIST: &str = "list";
/// A repository uses its own patterns (`conflict_base_branches`).
pub const BASE_SOURCE_OWN: &str = "own";

/// One named list of "Base branches" patterns.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct BaseList {
    /// The patterns, one per entry (`**`, `origin/main`, `origin/feature/**`,
    /// `!origin/abc`). An empty list allows every branch, like `**`.
    pub patterns: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ConflictPreventionConfig {
    /// Live: the name of the list a repository uses when it has no list of
    /// its own. Empty, or a name no list has, means the built-in `**`.
    pub default_list: String,
    /// Live: the named lists (`[conflict_prevention.lists.<name>]`). Editing
    /// one applies to every repository that follows it.
    pub lists: BTreeMap<String, BaseList>,
}

/// Where a repository's patterns came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BaseOrigin {
    /// The repository's own patterns.
    Own,
    /// A named list the repository follows.
    List(String),
    /// The global default list.
    DefaultList(String),
    /// Nothing set anywhere: the built-in `**`.
    BuiltIn,
}

/// The patterns in force and where they came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedBases {
    pub patterns: Vec<String>,
    pub origin: BaseOrigin,
}

impl ConflictPreventionConfig {
    /// The built-in default list: every branch.
    pub fn built_in() -> Vec<String> {
        vec![DEFAULT_BASE_GLOBS.to_string()]
    }

    /// The patterns of the list `name`, cleaned (blank lines and `#` comments
    /// dropped); `None` when there is no such list.
    pub fn list_patterns(&self, name: &str) -> Option<Vec<String>> {
        self.lists
            .get(name)
            .map(|l| parse_base_globs(&l.patterns.join("\n")))
    }

    /// The default list's patterns and its name, or the built-in `**`.
    pub fn default_patterns(&self) -> ResolvedBases {
        match self.list_patterns(&self.default_list) {
            Some(patterns) => ResolvedBases {
                patterns,
                origin: BaseOrigin::DefaultList(self.default_list.clone()),
            },
            None => ResolvedBases {
                patterns: Self::built_in(),
                origin: BaseOrigin::BuiltIn,
            },
        }
    }

    /// The patterns a repository uses, by precedence: its own list, then the
    /// named list it follows, then the global default list, then `**`.
    ///
    /// `source` is the repository's explicit choice (`"default"`, `"list"`,
    /// `"own"`; see the `BASE_SOURCE_*` constants). Absent (a file written
    /// before the choice existed) it is inferred: own patterns when there are
    /// any, else the named list, else the default. A named list that no longer
    /// exists falls back to the default; empty own patterns do too.
    pub fn resolve(
        &self,
        source: Option<&str>,
        list: Option<&str>,
        own: Option<&str>,
    ) -> ResolvedBases {
        let own_patterns = own.map(parse_base_globs).filter(|p| !p.is_empty());
        let named = list.and_then(|name| {
            self.list_patterns(name).map(|patterns| ResolvedBases {
                patterns,
                origin: BaseOrigin::List(name.to_string()),
            })
        });
        let from_own = |patterns: Vec<String>| ResolvedBases {
            patterns,
            origin: BaseOrigin::Own,
        };
        let chosen = match source {
            Some(BASE_SOURCE_OWN) => own_patterns.map(from_own),
            Some(BASE_SOURCE_LIST) => named,
            Some(BASE_SOURCE_DEFAULT) => None,
            _ => own_patterns.map(from_own).or(named),
        };
        chosen.unwrap_or_else(|| self.default_patterns())
    }

    /// Whether this is what a fresh config holds (an unset section is not
    /// written).
    pub fn is_default(&self) -> bool {
        *self == Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> ConflictPreventionConfig {
        let mut c = ConflictPreventionConfig::default();
        c.lists.insert(
            "release".into(),
            BaseList {
                patterns: vec!["origin/release/**".into(), "# note".into()],
            },
        );
        c.lists.insert(
            "trunk".into(),
            BaseList {
                patterns: vec!["origin/main".into()],
            },
        );
        c
    }

    #[test]
    fn nothing_set_is_the_built_in_everything() {
        let r = ConflictPreventionConfig::default().resolve(None, None, None);
        assert_eq!(r.patterns, ["**"]);
        assert_eq!(r.origin, BaseOrigin::BuiltIn);
    }

    #[test]
    fn the_default_list_replaces_the_built_in() {
        let mut c = config();
        c.default_list = "trunk".into();
        let r = c.resolve(None, None, None);
        assert_eq!(r.patterns, ["origin/main"]);
        assert_eq!(r.origin, BaseOrigin::DefaultList("trunk".into()));
        // A default that names no list is the built-in.
        c.default_list = "gone".into();
        assert_eq!(c.resolve(None, None, None).origin, BaseOrigin::BuiltIn);
    }

    #[test]
    fn own_beats_named_beats_default_beats_built_in() {
        let mut c = config();
        c.default_list = "trunk".into();
        let all = c.resolve(None, Some("release"), Some("origin/dev"));
        assert_eq!(all.patterns, ["origin/dev"]);
        assert_eq!(all.origin, BaseOrigin::Own);
        let named = c.resolve(None, Some("release"), None);
        assert_eq!(named.patterns, ["origin/release/**"], "comments dropped");
        assert_eq!(named.origin, BaseOrigin::List("release".into()));
        assert_eq!(c.resolve(None, None, None).patterns, ["origin/main"]);
    }

    #[test]
    fn an_explicit_choice_wins_over_what_else_is_stored() {
        let mut c = config();
        c.default_list = "trunk".into();
        let own = Some("origin/dev");
        let list = Some("release");
        assert_eq!(c.resolve(Some("own"), list, own).origin, BaseOrigin::Own);
        assert_eq!(
            c.resolve(Some("list"), list, own).origin,
            BaseOrigin::List("release".into())
        );
        assert_eq!(
            c.resolve(Some("default"), list, own).origin,
            BaseOrigin::DefaultList("trunk".into())
        );
    }

    #[test]
    fn a_deleted_list_or_empty_own_patterns_fall_back_to_the_default() {
        let mut c = config();
        c.default_list = "trunk".into();
        let r = c.resolve(Some("list"), Some("deleted"), None);
        assert_eq!(r.origin, BaseOrigin::DefaultList("trunk".into()));
        let r = c.resolve(Some("own"), None, Some("\n# nothing\n"));
        assert_eq!(r.origin, BaseOrigin::DefaultList("trunk".into()));
    }

    #[test]
    fn the_section_round_trips_through_toml() {
        let mut c = config();
        c.default_list = "trunk".into();
        let text = toml::to_string(&c).unwrap();
        assert!(text.contains("[lists.release]"), "{text}");
        let back: ConflictPreventionConfig = toml::from_str(&text).unwrap();
        assert_eq!(back, c);
    }
}
