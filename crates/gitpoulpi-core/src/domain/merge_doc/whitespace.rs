//! Whitespace-only conflicts that resolve themselves (plan, maintainer answer
//! 4: automatic when unambiguous, a button for what this cannot decide).
//!
//! A conflict is *whitespace only* when the two sides, with every
//! whitespace character (and blank lines) removed, are the same. It is
//! *unambiguous* when one of these holds:
//!
//! * a side is exactly the base: the other side is the change, take it;
//! * the sides differ only in trailing whitespace or a stray `\r` on
//!   lines that otherwise match one for one: keep our side.
//!
//! Indentation changes, re-wrapped lines and added or removed blank lines
//! are whitespace-only too but have no obviously right answer, so they
//! are left for the person (the editor shows a "whitespace only" badge).
//!
//! A file is resolved by this pass only if *every* one of its conflicts
//! is unambiguous: all or nothing, never half a file.

use super::{
    BlobContent, ConflictKind, ConflictPicks, ConflictSide, MergeDocument, MergeRegion,
    WorktreeFileState,
};

fn squeezed(lines: &[String]) -> Vec<String> {
    lines
        .iter()
        .map(|l| l.chars().filter(|c| !c.is_whitespace()).collect::<String>())
        .filter(|l| !l.is_empty())
        .collect()
}

/// Whether the conflict's two sides differ in whitespace only (and do
/// differ).
pub fn is_whitespace_only(ours: &[String], theirs: &[String]) -> bool {
    ours != theirs && squeezed(ours) == squeezed(theirs)
}

/// The side to take for one conflict, when the choice is unambiguous.
pub fn whitespace_choice(
    base: &[String],
    ours: &[String],
    theirs: &[String],
) -> Option<ConflictSide> {
    if !is_whitespace_only(ours, theirs) {
        return None;
    }
    if !base.is_empty() && base == ours {
        return Some(ConflictSide::Theirs);
    }
    if !base.is_empty() && base == theirs {
        return Some(ConflictSide::Ours);
    }
    let trailing_only = ours.len() == theirs.len()
        && ours
            .iter()
            .zip(theirs)
            .all(|(a, b)| a.trim_end() == b.trim_end());
    trailing_only.then_some(ConflictSide::Ours)
}

/// The picks that resolve every conflict of `doc`, if all of them are
/// whitespace-only and unambiguous; `None` otherwise (and for anything
/// that is not a plain text content conflict whose file on disk still has
/// git's markers, i.e. that nobody touched).
pub fn whitespace_resolution(doc: &MergeDocument) -> Option<Vec<ConflictPicks>> {
    if !matches!(
        doc.entry.kind,
        ConflictKind::BothModified | ConflictKind::BothAdded
    ) || doc.entry.mode_changed()
        || doc.entry.is_symlink()
        || doc.entry.is_submodule()
        || doc.worktree != WorktreeFileState::Markers
        || doc.conflict_count() == 0
    {
        return None;
    }
    let text = |b: &Option<super::SideBlob>| {
        b.as_ref()
            .is_none_or(|b| matches!(b.content, BlobContent::Text { .. }))
    };
    if !(text(&doc.base) && text(&doc.ours) && text(&doc.theirs)) {
        return None;
    }
    let mut picks = Vec::new();
    for region in &doc.regions {
        if let MergeRegion::Conflict { base, ours, theirs } = region {
            let side = whitespace_choice(base, ours, theirs)?;
            picks.push(ConflictPicks::take(region, &[side]));
        }
    }
    Some(picks)
}
