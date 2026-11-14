use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// One entry of `git worktree list`: the repository's own working
/// directory plus every linked worktree added with `git worktree add`.
///
/// Note the deliberate contract both backends converge on: **the main
/// worktree is always included, first, with `is_main: true`**. That
/// isn't free - `git worktree list` reports it but
/// `git2::Repository::worktrees()` lists *only* linked worktrees, so the
/// git2 backend has to synthesize it. Listing it matters because a repo
/// with no linked worktrees still has one worktree, and a UI that hides
/// the main entry would show an empty pane for the very repo you're
/// looking at.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Worktree {
    pub path: PathBuf,
    /// Short branch name (`refs/heads/` stripped), or `None` when this
    /// worktree has a detached HEAD.
    pub branch: Option<String>,
    pub is_main: bool,
    pub is_locked: bool,
}

/// The folder-safe form of a branch name: letters, digits, `-`, `_` and
/// `.` stay, anything else (slashes first) becomes `-`; runs of `-` shrink
/// to one and the ends are trimmed.
pub fn worktree_safe(text: &str) -> String {
    let mut out = String::new();
    for c in text.trim().chars() {
        let c = if c.is_alphanumeric() || "-_.".contains(c) {
            c
        } else {
            '-'
        };
        if c == '-' && out.ends_with('-') {
            continue;
        }
        out.push(c);
    }
    out.trim_matches('-').to_string()
}

/// The worktree name for a branch: `worktree-<branch>` with the branch
/// made folder-safe (`feature/login` gives `worktree-feature-login`),
/// without doubling the word: a branch that already has `worktree` as a
/// word (`worktree-1a2b3c4d`, `feature/worktree-x`, `worktree`) is used as
/// it is, made folder-safe. An empty branch gives an empty name.
pub fn worktree_name_after_branch(branch: &str) -> String {
    let safe = worktree_safe(branch);
    if safe.is_empty() {
        return safe;
    }
    let has_word = safe
        .split(|c: char| !c.is_alphanumeric())
        .any(|w| w.eq_ignore_ascii_case("worktree"));
    if has_word {
        safe
    } else {
        format!("worktree-{safe}")
    }
}

/// `worktree-<id>`: the name (and new branch) a worktree dialog starts
/// with. `id` comes from [`worktree_id`].
pub fn worktree_generated_name(id: &str) -> String {
    format!("worktree-{id}")
}

/// A short id for a generated worktree name: 8 lowercase hex digits from
/// 4 random bytes (the caller supplies the randomness, this crate has no
/// I/O).
pub fn worktree_id(bytes: [u8; 4]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Whether `name` is of the form `worktree-<8 lowercase hex>`.
pub fn is_generated_worktree_name(name: &str) -> bool {
    name.strip_prefix("worktree-").is_some_and(|id| {
        id.len() == 8
            && id
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    })
}

#[cfg(test)]
mod name_tests {
    use super::*;

    #[test]
    fn the_id_is_eight_lowercase_hex_digits() {
        let id = worktree_id([0x0a, 0xff, 0x00, 0x1b]);
        assert_eq!(id, "0aff001b");
        assert!(is_generated_worktree_name(&worktree_generated_name(&id)));
        assert!(!is_generated_worktree_name("worktree-0AFF001B"));
        assert!(!is_generated_worktree_name("worktree-0aff001"));
        assert!(!is_generated_worktree_name("worktree-feature"));
    }

    #[test]
    fn a_name_after_a_branch_never_doubles_the_word() {
        for (branch, name) in [
            ("feature", "worktree-feature"),
            ("feature/login", "worktree-feature-login"),
            ("fix//a b", "worktree-fix-a-b"),
            ("worktree-1a2b3c4d", "worktree-1a2b3c4d"),
            ("worktree", "worktree"),
            ("feature/worktree-x", "feature-worktree-x"),
            ("Worktree_x", "Worktree_x"),
            // A word, not a substring.
            ("worktrees", "worktree-worktrees"),
            ("", ""),
        ] {
            assert_eq!(worktree_name_after_branch(branch), name, "{branch:?}");
        }
    }
}
