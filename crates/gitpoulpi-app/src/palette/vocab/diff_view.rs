//! How a diff is laid out.
//!
//! Pure vocabulary: it names no terminal type, so a frontend other than the
//! terminal can share it. The module that owns the feature re-exports it.

/// The layout axis. The other axis, whole file or only the hunks, is
/// `DiffViewState::hunks_only`; every combination is valid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffViewMode {
    /// `+`/`-`/context interleaved in one column.
    Inline,
    /// Before and after side by side, deletes paired with adds.
    Split,
}

impl DiffViewMode {
    /// `m`: the other layout.
    pub fn next(self) -> Self {
        match self {
            DiffViewMode::Inline => DiffViewMode::Split,
            DiffViewMode::Split => DiffViewMode::Inline,
        }
    }
}

/// The config-side twin is `gitpoulpi_config::DiffLayout`; these two
/// conversions are the one seam between them.
impl From<gitpoulpi_config::DiffLayout> for DiffViewMode {
    fn from(layout: gitpoulpi_config::DiffLayout) -> Self {
        match layout {
            gitpoulpi_config::DiffLayout::Inline => DiffViewMode::Inline,
            gitpoulpi_config::DiffLayout::Split => DiffViewMode::Split,
        }
    }
}

impl From<DiffViewMode> for gitpoulpi_config::DiffLayout {
    fn from(mode: DiffViewMode) -> Self {
        match mode {
            DiffViewMode::Inline => gitpoulpi_config::DiffLayout::Inline,
            DiffViewMode::Split => gitpoulpi_config::DiffLayout::Split,
        }
    }
}
