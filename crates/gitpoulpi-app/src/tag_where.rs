//! Where a tag lives: local only, on a remote only, on both, or at different
//! commits, and the words that say so.
//!
//! git keeps no remote-tracking refs for tags, so GitPoulpi learns each
//! remote's tags from the last fetch or tag push and `refs()` reports, per
//! tag, a [`TagWhere`]. This module turns that list into a lookup
//! ([`snapshot_of`]) and into the tooltip text ([`tooltip_lines`]), so every
//! frontend tells the same story. The marker (glyph, icon) and its colour
//! are the frontend's: it maps each [`TagWhere`] to its own.

use std::collections::HashMap;

use gitpoulpi_core::domain::{Ref, TagWhere};
use gitpoulpi_i18n::msg::tag_where as msg;

/// What the snapshot knows about one tag.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagInfo {
    pub place: TagWhere,
    /// Remotes that have the tag at the same commit.
    pub remotes: Vec<String>,
    /// Remotes that have it at another commit.
    pub differs_on: Vec<String>,
}

impl TagInfo {
    /// Every remote that has a tag of this name, same commit or not.
    pub fn all_remotes(&self) -> Vec<String> {
        let mut all: Vec<String> = self
            .remotes
            .iter()
            .chain(self.differs_on.iter())
            .cloned()
            .collect();
        all.sort();
        all.dedup();
        all
    }
}

/// The snapshot of `refs`' tags, by tag name. A ref that is not a tag, or
/// carries no sync information, is not in it.
pub fn snapshot_of(refs: &[Ref]) -> HashMap<String, TagInfo> {
    refs.iter()
        .filter_map(|r| {
            let name = r.tag_name()?.to_string();
            let sync = r.tag_sync.as_ref()?;
            Some((
                name,
                TagInfo {
                    place: sync.where_(),
                    remotes: sync.pushed_to.clone(),
                    differs_on: sync.differs_on.clone(),
                },
            ))
        })
        .collect()
}

/// The tooltip of tag `name`: its name, where it lives in words, and what a
/// fetch does with it.
pub fn tooltip_lines(name: &str, info: &TagInfo) -> Vec<String> {
    let remotes = info.all_remotes().join(", ");
    let mut lines = vec![name.to_string()];
    lines.push(match info.place {
        TagWhere::Local => msg::TIP_LOCAL.get().to_string(),
        TagWhere::Remote => msg::tip_remote(&remotes),
        TagWhere::Both => msg::tip_both(&remotes),
        TagWhere::Differs => msg::tip_differs(&remotes),
    });
    if info.place == TagWhere::Differs {
        lines.push(msg::TIP_DIFFERS_ACTIONS.get().to_string());
    }
    // What a fetch does with it matters exactly when a remote has it.
    if info.place != TagWhere::Local {
        lines.push(msg::TIP_FETCH_NOTE.get().to_string());
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitpoulpi_core::domain::{Oid, RefKind, RefVisibility, TagSync};

    fn tag(name: &str, local: bool, pushed: &[&str], differs: &[&str]) -> Ref {
        Ref {
            kind: RefKind::Tag { name: name.into() },
            target: Oid([1; 20]),
            visibility: RefVisibility::Shown,
            is_head: false,
            upstream: None,
            tag_sync: Some(TagSync {
                pushed_to: pushed.iter().map(|s| s.to_string()).collect(),
                differs_on: differs.iter().map(|s| s.to_string()).collect(),
                local,
            }),
            multi_remote: None,
        }
    }

    fn four() -> Vec<Ref> {
        vec![
            tag("only-here", true, &[], &[]),
            tag("only-there", false, &["origin"], &[]),
            tag("both", true, &["origin"], &[]),
            tag("moved", true, &[], &["origin"]),
        ]
    }

    #[test]
    fn each_tag_is_told_apart_by_where_it_lives() {
        let snap = snapshot_of(&four());
        assert_eq!(snap["only-here"].place, TagWhere::Local);
        assert_eq!(snap["only-there"].place, TagWhere::Remote);
        assert_eq!(snap["both"].place, TagWhere::Both);
        assert_eq!(snap["moved"].place, TagWhere::Differs);
        assert_eq!(snap["moved"].all_remotes(), vec!["origin".to_string()]);
    }

    #[test]
    fn a_ref_without_sync_information_is_not_in_the_snapshot() {
        let mut plain = tag("plain", true, &[], &[]);
        plain.tag_sync = None;
        let branch = Ref {
            kind: RefKind::LocalBranch { name: "b".into() },
            ..tag("x", true, &[], &[])
        };
        assert!(snapshot_of(&[plain, branch]).is_empty());
    }

    #[test]
    fn the_tooltip_says_where_in_words_and_what_a_fetch_does() {
        let snap = snapshot_of(&four());
        let tip = |n: &str| tooltip_lines(n, &snap[n]).join("\n");
        let local = tip("only-here");
        assert!(local.contains("Only here"), "{local}");
        assert!(!local.contains("brings back"), "{local}");
        let remote = tip("only-there");
        assert!(remote.contains("Only on origin"), "{remote}");
        assert!(remote.contains("brings back"), "{remote}");
        let both = tip("both");
        assert!(
            both.contains("Here and on origin, at the same commit"),
            "{both}"
        );
        assert!(both.contains("Auto-prune"), "{both}");
        assert!(tip("moved").contains("different commits"));
    }
}
