//! What a history row says, in the UI language: the text lives in
//! `gitpoulpi_link::undo_label`. This file adds the terminal's icon column.

pub(crate) use gitpoulpi_link::undo_label::*;

/// Width-2 glyphs with default emoji presentation (no VS16), like the
/// sidebar's (`theme::TAG_ICON`, `STASH_ICON`): see "Emoji presentation"
/// in docs/developer/where-we-are.md.
pub(crate) fn glyph(family: Family) -> &'static str {
    match family {
        Family::Commit => "\u{1F4BE}", // floppy disk
        Family::Branch => "\u{1F331}", // seedling
        Family::Tag => crate::theme::TAG_ICON,
        Family::Stash => crate::theme::STASH_ICON,
        Family::Move => "\u{23EA}", // rewind
        Family::Merge => crate::theme::PULL_REQUEST_ICON,
        Family::Discard => "\u{1F525}", // fire
        Family::Edit => "\u{1F4DD}",    // memo
        Family::Staging => "\u{2795}",  // heavy plus
        Family::Config => "\u{1F527}",  // wrench
        Family::Outside => "\u{1F6AA}", // door
        Family::Other => "\u{00B7} ",   // middle dot, padded to the column
    }
}
