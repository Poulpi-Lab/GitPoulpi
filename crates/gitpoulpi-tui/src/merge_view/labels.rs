//! The side labels and the title description, translated at the display edge.
//!
//! The core builds them in English (`SideLabels`: `Current · main`,
//! `merging x into main`) and the CLI / MCP keep that text. The editor
//! recognises the fixed frames and renders them from the catalog; a text it
//! does not recognise is shown as it came.

use gitpoulpi_core::domain::ConflictSide;
use gitpoulpi_i18n::msg::merge as tr;
use gitpoulpi_i18n::msg::merge_view as mv;

const SEP: &str = " \u{b7} ";
const BASE: &str = "Base"; // i18n-core-frame
const CURRENT: &str = "Current \u{b7} "; // i18n-core-frame
const WORKTREE: &str = "working tree"; // i18n-core-frame
const ONTO: &str = "onto "; // i18n-core-frame
const INCOMING: &str = "Incoming \u{b7} "; // i18n-core-frame
const STASH: &str = "stash"; // i18n-core-frame
const REVERT_OF: &str = "revert of "; // i18n-core-frame
const YOUR_COMMIT: &str = "your commit "; // i18n-core-frame
const MERGING: &str = "merging "; // i18n-core-frame
const INTO: &str = " into "; // i18n-core-frame
const CHERRY: &str = "cherry-picking "; // i18n-core-frame
const REVERTING: &str = "reverting "; // i18n-core-frame
const REBASING: &str = "rebasing "; // i18n-core-frame
const ONTO_SP: &str = " onto "; // i18n-core-frame
const STEP: &str = " \u{b7} step "; // i18n-core-frame

/// The label of one pane, e.g. `Current · main`, in the UI language. An
/// empty core label gives the plain role word.
pub(crate) fn side(label: &str, side: ConflictSide) -> String {
    if label.is_empty() {
        return match side {
            ConflictSide::Ours => tr::CURRENT.get(),
            ConflictSide::Theirs => tr::INCOMING.get(),
            ConflictSide::Base => tr::BASE.get(),
        }
        .to_string();
    }
    match side {
        ConflictSide::Base => {
            if label == BASE {
                tr::BASE.get().to_string()
            } else {
                label.to_string()
            }
        }
        ConflictSide::Ours => match label.strip_prefix(CURRENT) {
            Some(WORKTREE) => mv::SIDE_CURRENT_WORKTREE.get().to_string(),
            Some(r) if r.starts_with(ONTO) => mv::side_current_onto(&r[ONTO.len()..]),
            Some(r) => mv::side_current(r),
            None => label.to_string(),
        },
        ConflictSide::Theirs => match label.strip_prefix(INCOMING) {
            Some(STASH) => mv::SIDE_INCOMING_STASH.get().to_string(),
            Some(r) if r.starts_with(REVERT_OF) => mv::side_incoming_revert(&r[REVERT_OF.len()..]),
            Some(r) if r.starts_with(YOUR_COMMIT) => {
                mv::side_incoming_rebase(&r[YOUR_COMMIT.len()..])
            }
            Some(r) => mv::side_incoming(r),
            None => label.to_string(),
        },
    }
}

/// The one-line description of the running operation (title bar).
pub(crate) fn description(desc: &str) -> String {
    if let Some(r) = desc.strip_prefix(MERGING) {
        if let Some((name, head)) = r.rsplit_once(INTO) {
            return mv::desc_merging(name, head);
        }
    } else if let Some(r) = desc.strip_prefix(CHERRY) {
        return mv::desc_cherry_pick(r);
    } else if let Some(r) = desc.strip_prefix(REVERTING) {
        return mv::desc_revert(r);
    } else if let Some(r) = desc.strip_prefix(REBASING) {
        if let Some((branch, tail)) = r.split_once(ONTO_SP) {
            let (onto, step) = match tail.split_once(STEP) {
                Some((o, s)) => (o, Some(s)),
                None => (tail, None),
            };
            if let Some((k, n)) = step.and_then(|s| s.split_once('/')) {
                return mv::desc_rebasing_step(branch, onto, k, n);
            }
            if step.is_none() {
                return mv::desc_rebasing(branch, onto);
            }
        }
    }
    desc.to_string()
}

/// A tab label: the role letter, then the label squeezed to `A Current·main`.
pub(crate) fn tab(label: &str, side_: ConflictSide) -> String {
    side(label, side_).replace(SEP, "\u{b7}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitpoulpi_i18n::{override_git_terms, override_lang, Lang};

    #[test]
    fn english_is_returned_unchanged() {
        for (l, s) in [
            ("Current \u{b7} main", ConflictSide::Ours),
            ("Current \u{b7} onto 1a2b3c4", ConflictSide::Ours),
            ("Current \u{b7} working tree", ConflictSide::Ours),
            ("Incoming \u{b7} feature/x", ConflictSide::Theirs),
            (
                "Incoming \u{b7} revert of 1a2b3c4 oops",
                ConflictSide::Theirs,
            ),
            (
                "Incoming \u{b7} your commit 1a2b3c4 wip",
                ConflictSide::Theirs,
            ),
            ("Incoming \u{b7} stash", ConflictSide::Theirs),
            ("Base", ConflictSide::Base),
            ("Base \u{b7} merge-base 9f8e7d6", ConflictSide::Base),
        ] {
            assert_eq!(side(l, s), l);
        }
        for d in [
            "merging feature/x into main",
            "cherry-picking 1a2b3c4",
            "reverting 1a2b3c4",
            "rebasing topic onto 1a2b3c4",
            "rebasing topic onto 1a2b3c4 \u{b7} step 3/5",
            "something else",
        ] {
            assert_eq!(description(d), d);
        }
    }

    #[test]
    fn french_labels_and_descriptions() {
        let _l = override_lang(Lang::Fr);
        assert_eq!(
            side("Current \u{b7} main", ConflictSide::Ours),
            "Courant \u{b7} main"
        );
        assert_eq!(
            side("Current \u{b7} onto 1a2b3c4", ConflictSide::Ours),
            "Courant \u{b7} sur 1a2b3c4"
        );
        assert_eq!(
            side("Incoming \u{b7} feature/x", ConflictSide::Theirs),
            "Entrant \u{b7} feature/x"
        );
        assert_eq!(side("", ConflictSide::Theirs), "Entrant");
        assert_eq!(
            tab("Current \u{b7} main", ConflictSide::Ours),
            "Courant\u{b7}main"
        );
        assert_eq!(
            description("merging feature/x into main"),
            "merge de feature/x dans main"
        );
        assert_eq!(
            description("rebasing topic onto 1a2b3c4 \u{b7} step 3/5"),
            "rebase de topic sur 1a2b3c4 \u{b7} \u{e9}tape 3/5"
        );
        {
            let _t = override_git_terms(true);
            assert_eq!(
                description("merging feature/x into main"),
                "fusion de feature/x dans main"
            );
            assert_eq!(
                side(
                    "Incoming \u{b7} your commit 1a2b3c4 wip",
                    ConflictSide::Theirs
                ),
                "Entrant \u{b7} votre validation 1a2b3c4 wip"
            );
            assert_eq!(
                side("Incoming \u{b7} stash", ConflictSide::Theirs),
                "Entrant \u{b7} remise"
            );
        }
    }
}
