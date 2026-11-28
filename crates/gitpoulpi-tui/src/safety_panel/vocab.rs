//! The Safety panel tabs.
//!
//! Pure vocabulary: it names no terminal type, so a frontend other than the
//! terminal can share it. The module that owns the feature re-exports it.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Tab {
    History,
    Reflog,
    Lost,
}
