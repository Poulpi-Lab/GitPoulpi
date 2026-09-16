//! The `editor` part of the config schema.

use serde::{Deserialize, Serialize};

config_enum! {
/// How the built-in editor writes line endings when it saves:
/// `editor.eol`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorEol {
    /// Keep what the file already uses (a new file gets LF).
    #[default]
    Auto => "auto",
    Lf => "lf",
    Crlf => "crlf",
}
}

impl EditorEol {
    /// The label the Settings combo shows.
    pub fn label(self) -> &'static str {
        match self {
            EditorEol::Auto => "Auto (keep the file's)",
            EditorEol::Lf => "LF",
            EditorEol::Crlf => "CRLF",
        }
    }

    pub fn from_label(label: &str) -> Option<EditorEol> {
        Self::ALL.iter().copied().find(|e| e.label() == label)
    }
}

pub const DEFAULT_EDITOR_TAB_SIZE: u8 = 4;

pub const MIN_EDITOR_TAB_SIZE: u8 = 1;

pub const MAX_EDITOR_TAB_SIZE: u8 = 8;

/// Default for `merge.side_by_side_min_width`.
pub const DEFAULT_MERGE_SIDE_BY_SIDE_MIN_WIDTH: u16 = 100;

/// Smallest width `merge.side_by_side_min_width` takes (below the merge
/// editor shows its notice anyway); `0` is the "always" switch.
pub const MIN_MERGE_SIDE_BY_SIDE_MIN_WIDTH: u16 = 40;

pub const MAX_MERGE_SIDE_BY_SIDE_MIN_WIDTH: u16 = 400;

/// The `[merge]` table: the merge conflict editor's layout.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct MergeConfig {
    /// Live: the width, in columns, of the central column from which the
    /// merge editor shows A and B side by side (long lines scroll
    /// sideways); narrower, it shows one side at a time behind tabs.
    /// `0` means always side by side. Read through
    /// [`MergeConfig::side_by_side_min_width_clamped`].
    pub side_by_side_min_width: u16,
}

impl Default for MergeConfig {
    fn default() -> Self {
        Self {
            side_by_side_min_width: DEFAULT_MERGE_SIDE_BY_SIDE_MIN_WIDTH,
        }
    }
}

impl MergeConfig {
    /// `side_by_side_min_width` with a hand-edited out-of-range value
    /// brought back: `0` stays "always", `1` to `39` become 40, and
    /// anything above 400 is 400.
    pub fn side_by_side_min_width_clamped(&self) -> u16 {
        match self.side_by_side_min_width {
            0 => 0,
            w => w.clamp(
                MIN_MERGE_SIDE_BY_SIDE_MIN_WIDTH,
                MAX_MERGE_SIDE_BY_SIDE_MIN_WIDTH,
            ),
        }
    }
}

/// The `[editor]` table: what makes sense in a terminal. Word wrap for
/// the diff views stays `ui.diff_word_wrap` (the Editor page shows it).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct EditorConfig {
    /// Live: columns between tab stops in the editor and the diff views.
    pub tab_size: u8,
    /// Live: the line ending the editor saves with.
    pub eol: EditorEol,
    /// Live: syntax colors in the editor.
    pub syntax_highlighting: bool,
    /// Live: the line-number gutter of the editor and the diff views.
    pub line_numbers: bool,
    /// Live: while the editor is open, selecting another file in the right
    /// pane opens that file in the editor too (asking to save first).
    pub follow_selection: bool,
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            tab_size: DEFAULT_EDITOR_TAB_SIZE,
            eol: EditorEol::Auto,
            syntax_highlighting: true,
            line_numbers: true,
            follow_selection: true,
        }
    }
}

impl EditorConfig {
    /// `tab_size` clamped to `1..=8`, so a hand-edited `0` or `99` cannot
    /// break the column arithmetic.
    pub fn tab_size_clamped(&self) -> u8 {
        self.tab_size
            .clamp(MIN_EDITOR_TAB_SIZE, MAX_EDITOR_TAB_SIZE)
    }
}
