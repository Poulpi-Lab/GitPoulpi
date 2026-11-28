//! The name rules of a new branch or tag, a rename and an upstream.

use gitpoulpi_core::domain::{Ref, RefKind};
use gitpoulpi_i18n::msg;

/// A new branch or a new tag.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NewRef {
    Branch,
    Tag,
}

/// `git check-ref-format`'s rules, as far as a short human message goes.
fn check_ref_format(name: &str) -> Result<(), String> {
    use msg::menu_inline as m;
    if name == "@" {
        return Err(m::NAME_AT.get().into());
    }
    if name.starts_with('/') || name.ends_with('/') || name.contains("//") {
        return Err(m::NAME_SLASH.get().into());
    }
    if name.ends_with('.') {
        return Err(m::NAME_DOT_END.get().into());
    }
    if name.contains("..") {
        return Err(m::NAME_DOTDOT.get().into());
    }
    if name.contains("@{") {
        return Err(m::NAME_AT_BRACE.get().into());
    }
    if name.starts_with('-') {
        return Err(m::NAME_DASH_START.get().into());
    }
    if let Some(bad) = name
        .chars()
        .find(|c| c.is_whitespace() || c.is_control() || "~^:?*[\\".contains(*c))
    {
        return Err(if bad.is_whitespace() {
            m::NAME_SPACES.get().to_string()
        } else {
            m::name_bad_char(bad.escape_default())
        });
    }
    for part in name.split('/') {
        if part.starts_with('.') {
            return Err(m::NAME_PART_DOT.get().into());
        }
        if part.ends_with(".lock") {
            return Err(m::NAME_PART_LOCK.get().into());
        }
    }
    Ok(())
}

/// The trimmed name if it can be created, else the message to show: git's format rules, an existing ref
/// of the same kind, and a directory/file clash (`a` next to `a/b`).
pub fn validate_new_ref(kind: NewRef, raw: &str, refs: &[Ref]) -> Result<String, String> {
    use msg::menu_inline as m;
    let name = raw.trim();
    if name.is_empty() {
        return Err(match kind {
            NewRef::Branch => m::ENTER_BRANCH.get().to_string(),
            NewRef::Tag => m::ENTER_TAG.get().to_string(),
        });
    }
    check_ref_format(name)?;
    let existing = refs.iter().filter_map(|r| match (&r.kind, kind) {
        (RefKind::LocalBranch { name }, NewRef::Branch) => Some(name.as_str()),
        (RefKind::Tag { name }, NewRef::Tag) => Some(name.as_str()),
        _ => None,
    });
    for other in existing {
        if other == name {
            return Err(match kind {
                NewRef::Branch => m::exists_branch(name),
                NewRef::Tag => m::exists_tag(name),
            });
        }
        if other.starts_with(&format!("{name}/")) || name.starts_with(&format!("{other}/")) {
            return Err(m::clashes(name, other));
        }
    }
    Ok(name.to_string())
}

/// [`validate_new_ref`] for a rename: the branch's own old name is not taken by itself.
pub fn validate_rename(old: &str, raw: &str, refs: &[Ref]) -> Result<String, String> {
    let others: Vec<Ref> = refs
        .iter()
        .filter(|r| !matches!(&r.kind, RefKind::LocalBranch { name } if name == old))
        .cloned()
        .collect();
    validate_new_ref(NewRef::Branch, raw, &others)
}

/// The upstream typed in the Set upstream form: empty unsets it, otherwise it must be a remote-tracking
/// branch the repository has (`origin/main`).
pub fn validate_upstream(raw: &str, refs: &[Ref]) -> Result<Option<String>, String> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Ok(None);
    }
    let known = refs.iter().any(|r| match &r.kind {
        RefKind::RemoteBranch { remote, name } => format!("{remote}/{name}") == raw,
        _ => false,
    });
    if known {
        Ok(Some(raw.to_string()))
    } else {
        Err(msg::git_actions::no_remote_branch(raw))
    }
}
