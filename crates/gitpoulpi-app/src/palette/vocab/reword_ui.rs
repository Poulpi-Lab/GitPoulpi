//! What an amend rewrites.
//!
//! Pure vocabulary: it names no terminal type, so a frontend other than the
//! terminal can share it. The module that owns the feature re-exports it.

/// Which entry point opened the popup. `Message` is the Edit commit
/// message popup (summary + description only); every other scope opens the
/// complete Amend dialog (`GitBackend::amend_commit`, every field shown) and
/// only decides which field the caret starts in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AmendScope {
    /// The Amend dialog, caret in the summary.
    All,
    /// Summary and description only: the message popup.
    Message,
    /// The Amend dialog, caret in the summary.
    Summary,
    /// The Amend dialog, caret in the description.
    Description,
    /// The Amend dialog, caret in the author's name.
    Author,
    /// The Amend dialog, caret in the author's date.
    Date,
}
