//! The central Diff view: everything that turns one `FileDiff` into the
//! rows the central pane draws, plus its own top button bar.
//!
//! Layout of the module: `model` (the row model and `render_diff_with`),
//! `layout` (the Inline/Split emitters, wrap, tabs, scroll extents),
//! `interaction` (row and body buttons, the visible window, selection,
//! copy, cursor) and this file (colors, toggles, bar buttons,
//! `DiffViewState`).
//!
//! Everything below the bar is a pure `FileDiff -> Vec<Line>` function
//! (`render_diff_with`), so both layouts, Hunks-only, word wrap and the
//! whitespace toggle are unit-testable without a terminal, a daemon or a
//! repo. `lib.rs` keeps the state transitions and the mouse/key wiring;
//! `diff_content.rs` fetches the whole file and caches the render.
//!
//! One walk produces the rows *and* the `hunk_starts` offsets the
//! next-hunk buttons scroll to: two walks over the same hunks would
//! drift, and a button that lands one row off is worse than none.
//!
//! The bar's File View / Blame / History buttons open the File Insight
//! overlay (`file_view.rs`), not another mode of this view; this module
//! only decides which revision and path they open on (see
//! `diff_insight_revision` in `lib.rs`).
//!
//! `DiffBarButton::pending_reason` is `None` for every button; it stays
//! as the way an unbacked button explains itself instead of eating the
//! click.

use gitpoulpi_i18n::msg::diff_view as msg;
use std::path::PathBuf;

use gitpoulpi_core::domain::{FileDiff, Oid};
#[cfg(test)]
use ratatui::prelude::*;

mod interaction;
mod layout;
mod model;

#[cfg(test)]
pub(crate) use interaction::hunk_button;
use layout::expand_tabs;
#[cfg(test)]
pub(crate) use model::render_diff;

use crate::text_select::TextSelection;
pub(crate) use interaction::{
    copy_text, diff_row_at, move_cursor, paint_text_selection, ruler_marks, text_pos_at,
    text_side_at, visible_window, yank, DiffBodyButton, DiffCopy,
};
pub(crate) use layout::marker as line_marker;
pub(crate) use layout::{h_scroll_max, h_track_content, minimap_row};
pub(crate) use model::{
    diff_bar, hunk_at, render_diff_with, selected_lines, DiffBarRender, DiffRender, DiffRenderOpts,
    DiffRow,
};
// What the review's diff pane builds on (the hook that feeds its rows into
// this render, and the pieces of the paint walk it shares).
#[cfg(all(feature = "hosting", feature = "review"))]
pub(crate) use interaction::{apply_cursor, change_marks, paint_band};
#[cfg(all(feature = "hosting", feature = "review"))]
pub(crate) use layout::annot_region;
#[cfg(all(feature = "hosting", feature = "review"))]
pub(crate) use model::{
    diff_bar_of, render_diff_annotated, AnnotBlock, AnnotRow, AnnotSide, AnnotTag, Annotator,
    RowChange,
};

/// The layout axis. The other axis, whole file or only the hunks, is
/// `DiffViewState::hunks_only`; every combination is valid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DiffViewMode {
    /// `+`/`-`/context interleaved in one column.
    Inline,
    /// Before and after side by side, deletes paired with adds.
    Split,
}

impl DiffViewMode {
    /// `m`: the other layout.
    pub(crate) fn next(self) -> Self {
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

/// One of the Diff view's display toggles: layout, hunks only, word wrap,
/// whitespace-only changes. `apply` is the only place any of them is
/// mutated: keys, bar clicks and palette commands all build one and hand
/// it over, and `lib.rs`'s `persist_diff_view_toggle` writes the
/// `(key, value)` pair `apply` returns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DiffViewToggle {
    Mode(DiffViewMode),
    /// Whole file <-> only the hunks.
    ToggleHunksOnly,
    ToggleWordWrap,
    ToggleWhitespace,
}

impl DiffViewToggle {
    /// Mutates `state`, resets its position (see
    /// `DiffViewState::reset_position`), and returns the dotted
    /// `config.toml` key plus the value to persist under it.
    pub(crate) fn apply(self, state: &mut DiffViewState) -> (&'static str, String) {
        let result = match self {
            DiffViewToggle::Mode(m) => {
                state.mode = m;
                (
                    "ui.diff_layout",
                    gitpoulpi_config::DiffLayout::from(m).as_str().to_string(),
                )
            }
            DiffViewToggle::ToggleHunksOnly => {
                state.hunks_only = !state.hunks_only;
                ("ui.diff_hunks_only", state.hunks_only.to_string())
            }
            DiffViewToggle::ToggleWordWrap => {
                state.word_wrap = !state.word_wrap;
                ("ui.diff_word_wrap", state.word_wrap.to_string())
            }
            DiffViewToggle::ToggleWhitespace => {
                state.show_whitespace = !state.show_whitespace;
                ("ui.diff_show_whitespace", state.show_whitespace.to_string())
            }
        };
        state.reset_position();
        result
    }
}

/// One button in the Diff view's top bar. A
/// button GitPoulpi cannot back yet is present but disabled rather than
/// absent, so the bar does not change shape once it lands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DiffBarButton {
    FileView,
    DiffView,
    Blame,
    /// The bar item of the commit column's `File | All` selector. It is
    /// drawn as its two segments ([`DiffBarButton::ColumnFile`], `ColumnAll`),
    /// which are what a click hits; this is what the bar's fitting measures,
    /// and what the `...` menu offers when the bar is too narrow (a click:
    /// hide or show the column).
    History,
    /// The selector's `File` segment: the column lists this file's commits.
    ColumnFile,
    /// The selector's `All` segment: the column lists every commit.
    ColumnAll,
    PrevHunk,
    NextHunk,
    Mode(DiffViewMode),
    ToggleHunksOnly,
    ToggleWhitespace,
    ToggleWordWrap,
    /// The minimap (`ui.minimap`, `M`): a global preference, so it is
    /// "active" from `live_settings::minimap()` rather than from the view state.
    ToggleMinimap,
    /// Edit in Working Directory: the built-in editor on the working-tree
    /// copy of the file shown (a diff or File View, of the working tree or
    /// of a commit). Always drawn; `edit_target` refuses when the path is
    /// not on disk (deleted, renamed away, or a deletion in the commit).
    EditWorkingFile,
    /// The encoding dropdown: how the file's bytes are read (`diff_recode`).
    /// Opens a combobox; its label is the choice.
    Encoding,
}

impl DiffBarButton {
    /// `Some(reason)` for a button that's deliberately inert. Clicking it
    /// reports the reason rather than doing nothing - a button that
    /// swallows clicks in silence is worse than no button at all.
    pub(crate) fn pending_reason(self) -> Option<&'static str> {
        // Nothing on this bar is inert today; the mechanism stays for the
        // next unbacked button.
        let _ = self;
        None
    }

    pub(crate) fn enabled(self) -> bool {
        self.pending_reason().is_none()
    }

    /// How long the button survives a narrowing bar (0 = kept longest;
    /// `button_bar.rs`): the hunk arrows, then the layouts and the Hunks
    /// toggle, then wrap and whitespace, then the buttons that leave the
    /// diff. Diff View goes first: it is always active, so a click does
    /// nothing.
    pub(crate) fn priority(self) -> u8 {
        match self {
            DiffBarButton::PrevHunk => 1,
            DiffBarButton::NextHunk => 2,
            DiffBarButton::Mode(DiffViewMode::Inline) => 3,
            DiffBarButton::Mode(DiffViewMode::Split) => 4,
            DiffBarButton::ToggleHunksOnly => 5,
            DiffBarButton::ToggleWordWrap => 6,
            DiffBarButton::ToggleWhitespace => 7,
            DiffBarButton::ToggleMinimap => 7,
            DiffBarButton::Encoding => 7,
            DiffBarButton::EditWorkingFile => 8,
            DiffBarButton::Blame => 8,
            // The way to and from the column: the last to fold.
            DiffBarButton::History | DiffBarButton::ColumnFile | DiffBarButton::ColumnAll => 0,
            DiffBarButton::FileView => 10,
            DiffBarButton::DiffView => 11,
        }
    }

    /// The registry action this button is the mouse twin of, if any.
    fn key_action(self) -> Option<crate::keymap::Action> {
        use crate::keymap::Action as K;
        Some(match self {
            DiffBarButton::PrevHunk => K::DiffPrevHunk,
            DiffBarButton::NextHunk => K::DiffNextHunk,
            DiffBarButton::Mode(_) => K::DiffToggleLayout,
            DiffBarButton::ToggleHunksOnly => K::DiffToggleHunksOnly,
            DiffBarButton::ToggleWhitespace => K::DiffToggleWhitespace,
            DiffBarButton::ToggleWordWrap => K::DiffToggleWrap,
            DiffBarButton::ToggleMinimap => K::DiffToggleMinimap,
            DiffBarButton::EditWorkingFile => K::DiffEdit,
            DiffBarButton::Encoding => K::DiffChooseEncoding,
            DiffBarButton::History | DiffBarButton::ColumnFile | DiffBarButton::ColumnAll => {
                K::GlobalCommitColumn
            }
            _ => return None,
        })
    }

    /// The text on the bar button, without the padding spaces: glyphs stay
    /// in code, the word comes from the catalog.
    pub(crate) fn bar_label(self) -> String {
        match self {
            DiffBarButton::FileView => msg::B_FILE_VIEW.get().to_string(),
            DiffBarButton::DiffView => msg::B_DIFF_VIEW.get().to_string(),
            DiffBarButton::Blame => msg::B_BLAME.get().to_string(),
            // The selector of the column left of the view: two words and a
            // rule, whatever the column does (its width never changes).
            DiffBarButton::History => {
                let (file, all) = crate::commit_column::selector_words();
                format!("{file} \u{2502} {all}")
            }
            DiffBarButton::ColumnFile => crate::commit_column::selector_words().0.to_string(),
            DiffBarButton::ColumnAll => crate::commit_column::selector_words().1.to_string(),
            DiffBarButton::EditWorkingFile => msg::B_EDIT.get().to_string(),
            DiffBarButton::PrevHunk => "\u{2191}".to_string(),
            DiffBarButton::NextHunk => "\u{2193}".to_string(),
            DiffBarButton::Mode(DiffViewMode::Inline) => {
                format!("\u{25A5} {}", msg::B_INLINE.get())
            }
            DiffBarButton::Mode(DiffViewMode::Split) => format!("\u{25A6} {}", msg::B_SPLIT.get()),
            DiffBarButton::ToggleHunksOnly => format!("\u{25A4} {}", msg::B_HUNKS.get()),
            DiffBarButton::ToggleWhitespace => format!("\u{00B6} {}", msg::B_WS.get()),
            DiffBarButton::ToggleWordWrap => format!("\u{21B5} {}", msg::B_WRAP.get()),
            DiffBarButton::ToggleMinimap => format!("\u{25A7} {}", msg::B_MAP.get()),
            DiffBarButton::Encoding => format!("{} \u{25BE}", msg::B_ENCODING.get()),
        }
    }

    /// [`bar_label`](Self::bar_label), but the encoding button names the
    /// encoding the view is reading in.
    pub(crate) fn label_for(self, state: &DiffViewState) -> String {
        match self {
            DiffBarButton::Encoding => format!(
                "{} \u{25BE}",
                state
                    .encoding
                    .map_or_else(|| msg::B_ENCODING_AUTO.get(), |c| c.name())
            ),
            other => other.bar_label(),
        }
    }

    /// The hover tooltip: what the button does, and its effective key
    /// when it has one.
    pub(crate) fn tooltip(self) -> String {
        let what = match self {
            DiffBarButton::FileView => msg::T_FILE_VIEW.get(),
            DiffBarButton::DiffView => msg::T_DIFF_VIEW.get(),
            DiffBarButton::Blame => msg::T_BLAME.get(),
            DiffBarButton::ColumnFile => msg::T_COLUMN_FILE.get(),
            DiffBarButton::History | DiffBarButton::ColumnAll => msg::T_COLUMN_ALL.get(),
            DiffBarButton::PrevHunk => gitpoulpi_i18n::msg::button_bar::PREV_HUNK.get(),
            DiffBarButton::NextHunk => gitpoulpi_i18n::msg::button_bar::NEXT_HUNK.get(),
            DiffBarButton::Mode(_) => msg::T_LAYOUT.get(),
            DiffBarButton::ToggleHunksOnly => msg::T_HUNKS_ONLY.get(),
            DiffBarButton::ToggleWhitespace => msg::T_WHITESPACE.get(),
            DiffBarButton::ToggleWordWrap => msg::T_WRAP.get(),
            DiffBarButton::ToggleMinimap => msg::T_MINIMAP.get(),
            DiffBarButton::EditWorkingFile => msg::T_EDIT.get(),
            DiffBarButton::Encoding => msg::T_ENCODING.get(),
        };
        match self.key_action() {
            Some(a) => format!(
                "{what}{}",
                crate::keymap::paren(&crate::keymap::current(), a)
            ),
            None => what.to_string(),
        }
    }

    /// Whether the button should render as "currently on" (the active view
    /// mode, or an engaged toggle).
    fn is_active(self, state: &DiffViewState) -> bool {
        match self {
            // The File History's right side is the History view: its tab
            // is the lit one, and Diff View is somewhere to go to.
            DiffBarButton::DiffView => !crate::history_view::HistoryView::owns(state),
            DiffBarButton::ColumnFile => {
                crate::commit_column::painted() == crate::commit_column::ColumnState::File
            }
            DiffBarButton::ColumnAll => {
                crate::commit_column::painted() == crate::commit_column::ColumnState::Graph
            }
            DiffBarButton::Mode(m) => state.mode == m,
            DiffBarButton::ToggleHunksOnly => state.hunks_only,
            DiffBarButton::ToggleWhitespace => state.show_whitespace,
            DiffBarButton::ToggleWordWrap => state.word_wrap,
            DiffBarButton::ToggleMinimap => crate::live_settings::minimap(),
            DiffBarButton::Encoding => state.encoding.is_some(),
            _ => false,
        }
    }
}

/// The Diff view's bar, in drawing order; the text of each comes from
/// [`DiffBarButton::bar_label`].
pub(crate) const DIFF_BAR_BUTTONS: &[DiffBarButton] = &[
    DiffBarButton::FileView,
    DiffBarButton::DiffView,
    DiffBarButton::Blame,
    DiffBarButton::History,
    DiffBarButton::EditWorkingFile,
    DiffBarButton::PrevHunk,
    DiffBarButton::NextHunk,
    DiffBarButton::Mode(DiffViewMode::Inline),
    DiffBarButton::Mode(DiffViewMode::Split),
    DiffBarButton::ToggleHunksOnly,
    DiffBarButton::ToggleWhitespace,
    DiffBarButton::ToggleWordWrap,
    DiffBarButton::ToggleMinimap,
    DiffBarButton::Encoding,
];

/// The Diff view's staging affordances, which live in the body rather
/// than the top bar: the per-hunk button, the line selection, and
/// applying it.
///
/// Matched exhaustively by `palette.rs`'s reachability guard, like
/// `DiffBarButton`, so a new way to stage from this view cannot exist
/// without a palette command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DiffStageAction {
    /// The whole hunk under the cursor (Space, or its header button).
    Hunk,
    /// The selected line range, or the line under the cursor (Enter).
    Lines,
    /// Start or clear the selection (`v`). Direction-independent, but
    /// listed here so the guard covers the whole affordance.
    SelectLines,
    /// Revert the hunk under the cursor in the working tree (`x`, or its
    /// header's `Discard hunk`) - Unstaged half only, always confirmed.
    DiscardHunk,
    /// Revert the selected lines, or the line under the cursor (`X`, or a
    /// hovered row's `✗ Discard`) - Unstaged half only, confirmed.
    DiscardLines,
}

/// Which diff the view is showing, and therefore what staging a hunk out
/// of it means.
///
/// `StageTarget::{Hunk,Line}` is resolved by the backend against a diff it
/// fetches itself (`WorkingTreeVsIndex` to stage, `IndexVsHead` to
/// unstage; see `gitpoulpi-cli-backend`'s `staging.rs`), so staging is
/// offered only on those two diffs. A commit's diff has no index to move
/// a hunk into, and its hunk id would mean something else.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DiffSource {
    /// A commit's diff against its parent - read-only.
    Commit,
    /// The comparison of the Log's multi-select (`compare`): two commits,
    /// a commit and the working tree, a stash and a commit, a run of
    /// commits. Read-only like a commit's; the target is in
    /// `DiffViewState::compare`.
    Compare,
    /// `DiffTarget::WorkingTreeVsIndex`, the Unstaged half of the third
    /// pane. A hunk here is *staged*.
    Unstaged,
    /// `DiffTarget::IndexVsHead`, the Staged half. A hunk here is
    /// *unstaged*.
    Staged,
}

impl DiffSource {
    /// `true` for the half whose staging action moves changes *into* the
    /// index, `false` for the one that takes them back out, `None` when
    /// there is no staging action at all.
    pub(crate) fn staging(self) -> Option<bool> {
        match self {
            DiffSource::Commit | DiffSource::Compare => None,
            DiffSource::Unstaged => Some(true),
            DiffSource::Staged => Some(false),
        }
    }

    /// The daemon `DiffTarget` this source is, for the two halves that
    /// have one: how an arriving `WorkingDiffOutcome` is matched against
    /// the open view.
    pub(crate) fn diff_target(self) -> Option<gitpoulpi_core::domain::DiffTarget> {
        use gitpoulpi_core::domain::DiffTarget;
        match self {
            DiffSource::Commit | DiffSource::Compare => None,
            DiffSource::Unstaged => Some(DiffTarget::WorkingTreeVsIndex),
            DiffSource::Staged => Some(DiffTarget::IndexVsHead),
        }
    }

    /// Whether this half offers Discard hunk / Discard lines. Only the
    /// Unstaged one: discarding reverts the working tree to the index,
    /// which is what its diff is against. A staged change is unstaged
    /// first.
    pub(crate) fn discards(self) -> bool {
        self == DiffSource::Unstaged
    }

    /// Label of the per-hunk button, worded exactly like the status
    /// pane's own row buttons (`Stage file` / `Unstage file`).
    pub(crate) fn hunk_label(self) -> Option<&'static str> {
        match self {
            DiffSource::Commit | DiffSource::Compare => None,
            DiffSource::Unstaged => Some(gitpoulpi_i18n::msg::diff_actions::HUNK_STAGE.get()),
            DiffSource::Staged => Some(gitpoulpi_i18n::msg::diff_actions::HUNK_UNSTAGE.get()),
        }
    }
}

/// Which file is open in the central Diff view and how it is shown.
/// `file_index` indexes the same `Vec<FileDiff>` the commit-detail pane's
/// changed-files list is built from, so opening a diff needs no second
/// daemon round trip.
#[derive(Debug, Clone)]
pub(crate) struct DiffViewState {
    pub(crate) source: DiffSource,
    /// The target of a `DiffSource::Compare` diff; `None` otherwise.
    pub(crate) compare: Option<gitpoulpi_core::domain::DiffTarget>,
    /// The commit this diff was opened against. Checked against the
    /// incoming `CommitFilesOutcome` before indexing: a refresh can swap
    /// `commit_files` for another commit's files, and position alone
    /// would show the wrong file. Meaningless for the working-tree
    /// sources.
    pub(crate) commit: Oid,
    pub(crate) file_index: usize,
    /// The file's path: what every `StageTarget` names, and how the
    /// working-tree diffs are looked up (staging a hunk changes which
    /// files each half lists, so an index would go stale).
    pub(crate) path: PathBuf,
    /// The file's other name, for a diff that crosses a rename (the History
    /// view): asked together with `path`, answered as one `Renamed` file.
    pub(crate) also_path: Option<PathBuf>,
    pub(crate) mode: DiffViewMode,
    /// `true`: only the hunks with git's context lines around them.
    /// `false`: the whole file (its new side, fetched by
    /// `diff_content.rs`), the hunks highlighted in place.
    pub(crate) hunks_only: bool,
    pub(crate) word_wrap: bool,
    /// When false, a `-`/`+` pair differing only in trailing whitespace
    /// collapses to a single context line, and a hunk left with no real
    /// change disappears entirely.
    pub(crate) show_whitespace: bool,
    pub(crate) scroll: usize,
    /// First character column shown of each row's content span, never the
    /// fixed gutter. Forced back to 0 while `word_wrap` is on.
    pub(crate) h_offset: usize,
    /// The keyboard cursor, as an index into the rendered rows (not the
    /// file's lines): the one numbering the mouse and the keyboard share.
    pub(crate) cursor: usize,
    /// The other end of a visual-style line selection, or `None` when
    /// only the cursor row is current. Also an index into rendered rows.
    pub(crate) selection_anchor: Option<usize>,
    /// A mouse text selection, independent of the staging line selection
    /// above: anchored in the text (see `text_select`), copied by
    /// `Ctrl+C` / `y` / Copy, never staged.
    pub(crate) text_sel: Option<TextSelection>,
    /// A Git LFS file's diff shows a card (sizes, short oids, whether the
    /// new object is stored locally) instead of the three lines of
    /// pointer text; `r` flips this to show the raw pointer diff.
    pub(crate) lfs_raw: bool,
    /// Whether the new side's LFS object is in the local store (`None`:
    /// unknown). Republished every frame, so a `Fetch` shows in the card.
    pub(crate) lfs_local: Option<bool>,
    /// The encoding the file's two sides are read in; `None` is
    /// Auto-detect (what the backends decode). Set from the dropdown or
    /// the remembered choice for the path (`diff_recode`). While it is set
    /// the hunks are `diff_recode`'s, so staging is off.
    pub(crate) encoding: Option<crate::editor_charset::Charset>,
    /// The band an agent's `ui show diff` / `ui highlight` tinted, drawn
    /// over the cursor's band. Gone with the view.
    pub(crate) highlight: Option<DiffHighlight>,
    /// Set by the draw once the rows drawn are this state's real diff
    /// (the file, and the whole-file content when asked for). An agent's
    /// reveal waits on it. Cleared by whatever changes the layout.
    pub(crate) ready: bool,
    /// A pane jump (`Ctrl+N <badge>`, a click on a list, the palette's focus
    /// commands) handed the navigation keys (`j`/`k`, arrows, paging, `h`/`l`)
    /// to a list while the diff stays open beside it. A freshly opened diff
    /// is `false`: it owns them. Living on the state means closing the diff
    /// or opening another one resets it, with nothing to remember.
    pub(crate) lists_have_keys: bool,
}

/// A range of file lines on one side of a diff, tinted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DiffHighlight {
    /// `0`: the old side's numbers, `1`: the new side's.
    pub(crate) side: usize,
    pub(crate) from: u32,
    pub(crate) to: u32,
}

impl DiffHighlight {
    fn covers(&self, row: &DiffRow) -> bool {
        row.numbers[self.side].is_some_and(|n| (self.from..=self.to).contains(&n))
    }
}

/// The first rendered row showing `line` on `side` (`0` old, `1` new).
pub(crate) fn row_of_line(rows: &[DiffRow], side: usize, line: u32) -> Option<usize> {
    rows.iter().position(|r| r.numbers[side] == Some(line))
}

/// The file lines (new side, else old) the rows `first..last` show, as
/// `(first, last)`: what `ui state` reports as `visible_lines`.
pub(crate) fn line_span(rows: &[DiffRow], first: usize, last: usize) -> Option<(u32, u32)> {
    let numbered = |side: usize| {
        let slice = rows.get(first..last.min(rows.len()))?;
        let mut it = slice.iter().filter_map(|r| r.numbers[side]);
        let lo = it.next()?;
        Some((lo, slice.iter().rev().find_map(|r| r.numbers[side])?))
    };
    numbered(1).or_else(|| numbered(0))
}

impl DiffViewState {
    /// A commit's diff against its parent, opened from the commit-detail
    /// pane's changed-files list.
    pub(crate) fn for_commit(commit: Oid, file_index: usize, path: PathBuf) -> Self {
        Self {
            source: DiffSource::Commit,
            compare: None,
            commit,
            file_index,
            path,
            also_path: None,
            mode: DiffViewMode::Inline,
            hunks_only: true,
            word_wrap: false,
            show_whitespace: true,
            scroll: 0,
            h_offset: 0,
            cursor: 0,
            selection_anchor: None,
            text_sel: None,
            lfs_raw: false,
            lfs_local: None,
            encoding: None,
            highlight: None,
            ready: false,
            lists_have_keys: false,
        }
    }

    /// A comparison's diff, opened from the right pane's file list. `key` is
    /// the comparison's synthetic id (`CompareSpec::id`), which the file list
    /// is published under.
    pub(crate) fn for_compare(
        key: Oid,
        target: gitpoulpi_core::domain::DiffTarget,
        file_index: usize,
        path: PathBuf,
    ) -> Self {
        Self {
            source: DiffSource::Compare,
            compare: Some(target),
            ..Self::for_commit(key, file_index, path)
        }
    }

    /// The daemon `DiffTarget` this view shows, when it is not a plain
    /// commit's: a working-tree half or a comparison.
    pub(crate) fn diff_target(&self) -> Option<gitpoulpi_core::domain::DiffTarget> {
        match self.source {
            DiffSource::Compare => self.compare.clone(),
            other => other.diff_target(),
        }
    }

    /// One half of the working tree, opened from the third pane. This is
    /// the only diff hunk- and line-level staging can act on.
    pub(crate) fn for_working(source: DiffSource, path: PathBuf) -> Self {
        Self {
            source,
            ..Self::for_commit(Oid([0u8; 20]), 0, path)
        }
    }

    /// The inclusive row range a line-level action covers: the selection
    /// when one is active, the cursor row alone otherwise.
    pub(crate) fn selected_rows(&self) -> (usize, usize) {
        match self.selection_anchor {
            Some(anchor) => (anchor.min(self.cursor), anchor.max(self.cursor)),
            None => (self.cursor, self.cursor),
        }
    }

    /// Overwrites the layout/whitespace/wrap the constructor set with the
    /// user's last choices from `crate::live_settings::diff_view_prefs()`.
    ///
    /// Kept out of `for_commit`/`for_working` so those stay pure
    /// constructors: the tests here are then immune to what another test
    /// in the shared binary published to that global. The call sites in
    /// `lib.rs` opt in.
    pub(crate) fn resume_last_used(mut self) -> Self {
        let prefs = crate::live_settings::diff_view_prefs();
        self.mode = prefs.layout.into();
        self.hunks_only = prefs.hunks_only;
        self.show_whitespace = prefs.show_whitespace;
        self.word_wrap = prefs.word_wrap;
        self.encoding = crate::diff_recode::remembered(&self.path);
        self
    }

    /// Resets everything positional. Called whenever the layout changes:
    /// the cursor and selection are row numbers of *that* layout, and
    /// carrying them across would stage lines the user never pointed at.
    pub(crate) fn reset_position(&mut self) {
        self.scroll = 0;
        self.h_offset = 0;
        self.cursor = 0;
        self.selection_anchor = None;
        self.text_sel = None;
    }

    /// Scrolls the content column sideways, independent of `scroll` and
    /// the cursor. Clamps only the lower bound; the render loop clamps
    /// the upper one against `longest_line(file)` every frame.
    pub(crate) fn scroll_h(&mut self, delta: isize) {
        if self.word_wrap {
            return;
        }
        let next = self.h_offset as isize + delta;
        self.h_offset = next.max(0) as usize;
    }
}

/// Longest raw line in the diff, in terminal cells: what `scroll_h` clamps
/// against. Measured over the whole file, not the rows on screen, so the
/// maximum does not move while scrolling vertically. `full_file` is the
/// whole-file content when that is what is drawn (`diff_content.rs`
/// caches the answer per file).
pub(crate) fn longest_line(file: &FileDiff, full_file: Option<&[String]>) -> usize {
    let width = |s: &str| crate::cell_text::cells(&expand_tabs(s));
    let hunks = file
        .hunks
        .iter()
        .flat_map(|h| h.lines.iter())
        .map(|l| width(&l.content));
    let full = full_file.unwrap_or(&[]).iter().map(|l| width(l));
    hunks.chain(full).max().unwrap_or(0)
}

/// Where the ↑/↓ buttons, `n`/`p` and the palette land: the next (or
/// previous) hunk header after (before) the cursor row. By the cursor,
/// not the scroll offset: near the end of a file the scroll is clamped
/// short of the last headers, and a jump measured from it would find the
/// same header again and again.
pub(crate) fn hunk_jump(starts: &[usize], cursor: usize, forward: bool) -> Option<usize> {
    if forward {
        starts.iter().copied().find(|&h| h > cursor)
    } else {
        starts.iter().rev().copied().find(|&h| h < cursor)
    }
}

/// Moves the cursor (and the view) onto the next/previous hunk header, or
/// says why it cannot. The cursor lands on the header so the next Space
/// stages the hunk just jumped to.
///
/// Measured from the cursor while it is on screen, else from the top of
/// the view ("next" after what the user is looking at). `view_height` is
/// the inner height (rows of diff drawn).
pub(crate) fn jump_to_hunk(
    state: &mut DiffViewState,
    starts: &[usize],
    forward: bool,
    view_height: u16,
) -> Result<(), &'static str> {
    let visible = state.scroll..state.scroll + view_height.max(1) as usize;
    let anchor = if visible.contains(&state.cursor) {
        state.cursor
    } else {
        state.scroll
    };
    match hunk_jump(starts, anchor, forward) {
        Some(row) => {
            state.cursor = row;
            state.scroll = row;
            Ok(())
        }
        None if forward => Err(msg::NO_FURTHER_HUNK.get()),
        None => Err(msg::FIRST_HUNK.get()),
    }
}

#[cfg(test)]
mod tests;
