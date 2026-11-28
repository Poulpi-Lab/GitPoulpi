//! The editor status bar menus.
//!
//! Pure vocabulary: it names no terminal type, so a frontend other than the
//! terminal can share it. The module that owns the feature re-exports it.

/// Which menu: also the palette's `Editor: ...` commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum StatusMenu {
    /// Reopen with an encoding, or convert to one.
    Encoding,
    /// Read the file's bytes again in the chosen encoding.
    Reopen,
    /// Keep the text, save it in the chosen encoding.
    Convert,
    LineEndings,
    Indentation,
}
