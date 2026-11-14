//! What a refresh's refs read does to the list the window holds.
//!
//! A failed read must not wipe a good list: every branch and tag pill, the
//! Refs pane and the checked-out name would vanish for one bad poll (the
//! repository mid-write, a `git` killed). The last good list stays and the
//! status line says why; the next good read replaces it.

use gitpoulpi_core::domain::Ref;
use gitpoulpi_core::Result;

pub(super) enum RefsUpdate {
    /// Take the new result (a good list, or the first error with nothing to keep).
    Replace(Result<Vec<Ref>>),
    /// Keep the held list; `why` is the first line of the error.
    Keep { why: String },
}

pub(super) fn refs_update(held: &Result<Vec<Ref>>, new: Result<Vec<Ref>>) -> RefsUpdate {
    match (&new, held) {
        (Err(error), Ok(_)) => RefsUpdate::Keep {
            why: error
                .to_string()
                .lines()
                .next()
                .unwrap_or_default()
                .to_string(),
        },
        _ => RefsUpdate::Replace(new),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitpoulpi_core::domain::{Oid, RefKind, RefVisibility};
    use gitpoulpi_core::Error;

    fn branch(name: &str) -> Ref {
        Ref {
            kind: RefKind::LocalBranch { name: name.into() },
            target: Oid([1; 20]),
            visibility: RefVisibility::Shown,
            is_head: false,
            upstream: None,
            tag_sync: None,
            multi_remote: None,
        }
    }

    fn failure() -> Result<Vec<Ref>> {
        Err(Error::Backend(
            "git for-each-ref failed: boom\nsecond line".into(),
        ))
    }

    #[test]
    fn a_failed_read_keeps_the_good_list_and_names_the_error() {
        let held = Ok(vec![branch("main")]);
        match refs_update(&held, failure()) {
            RefsUpdate::Keep { why } => {
                assert!(why.contains("boom"), "{why}");
                assert!(!why.contains("second line"), "one line only: {why}");
            }
            RefsUpdate::Replace(_) => panic!("a good list was replaced by an error"),
        }
    }

    #[test]
    fn a_good_read_replaces_whatever_is_held() {
        for held in [Ok(vec![branch("old")]), failure()] {
            match refs_update(&held, Ok(vec![branch("new")])) {
                RefsUpdate::Replace(Ok(list)) => assert_eq!(list.len(), 1),
                _ => panic!("a good read must replace the held result"),
            }
        }
    }

    #[test]
    fn an_error_with_nothing_good_to_keep_is_shown_as_the_error() {
        assert!(matches!(
            refs_update(&failure(), failure()),
            RefsUpdate::Replace(Err(_))
        ));
    }
}
