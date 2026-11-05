//! What each Log column would need to show its content untruncated.
//!
//! The paint walk feeds every row it builds into one [`NaturalWidths`]
//! (`WindowState::log_natural_widths`); "fit this column to its content"
//! (double-click on its boundary, `=`) reads it back. Measuring in the walk
//! that paints keeps the numbers the width of what is drawn, wide glyphs
//! included (`display_width`, never a char count).

use super::*;

/// The widest content of each column among the rows and the header the last
/// paint drew, in cells and including the one gutter cell every text column
/// keeps (so a fitted column shows its text without an ellipsis).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct NaturalWidths {
    cells: [u16; 7],
}

impl Default for NaturalWidths {
    fn default() -> Self {
        Self::new()
    }
}

fn slot(col: LogColumn) -> usize {
    LogColumn::ALL
        .iter()
        .position(|c| *c == col)
        .unwrap_or_default()
}

/// The `ui_kit::display_width` of every span, summed.
fn spans_width(spans: &[Span<'_>]) -> usize {
    spans.iter().map(|s| display_width(&s.content)).sum()
}

impl NaturalWidths {
    /// Starts from the header titles: a column is never fitted narrower than
    /// its own name.
    pub(crate) fn new() -> Self {
        let mut me = Self { cells: [0; 7] };
        for col in LogColumn::ALL {
            me.grow(col, display_width(col.title()));
        }
        me
    }

    /// Notes that `col` has content `cells` wide in some row.
    pub(crate) fn grow(&mut self, col: LogColumn, cells: usize) {
        let i = slot(col);
        let wanted = u16::try_from(cells.saturating_add(1)).unwrap_or(u16::MAX);
        self.cells[i] = self.cells[i].max(wanted);
    }

    /// The width that shows `col`'s widest content, clamped to what a column
    /// can be at all.
    pub(crate) fn of(&self, col: LogColumn) -> u16 {
        let floor = match col {
            // Wider than the compact picture, so it still draws lanes.
            LogColumn::Graph => crate::graph::COMPACT_W as u16 + 1,
            other => other.drag_min_width(),
        };
        self.cells[slot(col)].clamp(floor, MAX_COLUMN_CELLS)
    }

    /// One commit row.
    pub(crate) fn commit_row(
        &mut self,
        commit: &CommitNode,
        pills: &[RefPill],
        graph_cells: usize,
        avatar_mode: avatar::LogAvatarMode,
        avatar_fallback: gitpoulpi_config::AvatarFallback,
        changes: Option<&gitpoulpi_core::domain::CommitStat>,
    ) {
        self.grow(
            LogColumn::Ref,
            spans_width(&ref_pill_spans(pills, usize::MAX / 2, th().ui.accent, None)),
        );
        self.grow(LogColumn::Graph, graph_cells);
        self.grow(LogColumn::Message, display_width(&commit.summary));
        self.grow(
            LogColumn::Author,
            author_width(
                &commit.author_name,
                &commit.author_email,
                avatar_mode,
                avatar_fallback,
            ),
        );
        self.grow(
            LogColumn::Date,
            display_width(&format_timestamp(commit.timestamp)),
        );
        self.grow(
            LogColumn::Changes,
            spans_width(&changes_spans(changes, usize::MAX / 2, Style::default())),
        );
        self.grow(LogColumn::Sha, 7);
    }

    /// The WIP row: the summary (or its placeholder) and the counters it
    /// draws in the Message cell.
    pub(crate) fn wip_row(
        &mut self,
        summary: &str,
        counts_in_message: WipCounts,
        graph_cells: usize,
    ) {
        let text = if summary.is_empty() {
            WIP_PLACEHOLDER
        } else {
            summary
        };
        self.grow(LogColumn::Graph, graph_cells);
        self.grow(
            LogColumn::Message,
            display_width(text) + spans_width(&wip_counter_spans(counts_in_message)),
        );
    }

    /// One stash row.
    pub(crate) fn stash_row(&mut self, stash: &gitpoulpi_core::domain::Stash, graph_cells: usize) {
        self.grow(LogColumn::Graph, graph_cells);
        self.grow(LogColumn::Message, display_width(&stash.message));
        self.grow(
            LogColumn::Author,
            display_width(&format!("stash@{{{}}}", stash.id.0)),
        );
        self.grow(
            LogColumn::Date,
            display_width(&format_timestamp(stash.timestamp)),
        );
        self.grow(LogColumn::Sha, 7);
    }
}

/// The Author cell's content width: the avatar badge, a space and the name
/// when avatars are on, the name alone otherwise (what `commit_row` draws).
pub(crate) fn author_width(
    name: &str,
    email: &str,
    mode: avatar::LogAvatarMode,
    fallback: gitpoulpi_config::AvatarFallback,
) -> usize {
    if mode == avatar::LogAvatarMode::Off {
        return display_width(name);
    }
    let (glyph, _) = avatar::fallback_text(
        fallback,
        name,
        email,
        if mode == avatar::LogAvatarMode::Picture {
            usize::from(avatar::SLOT_COLS)
        } else {
            1
        },
    );
    display_width(&glyph) + 1 + display_width(name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::log_table::tests::sample_commit;

    fn measured(name: &str, summary: &str) -> NaturalWidths {
        let mut commit = sample_commit(1, summary);
        commit.author_name = name.to_string();
        let mut n = NaturalWidths::new();
        n.commit_row(
            &commit,
            &[],
            5,
            avatar::LogAvatarMode::Off,
            gitpoulpi_config::AvatarFallback::Initials,
            None,
        );
        n
    }

    /// Wide glyphs count two cells (the emoji presentation and CJK both),
    /// plus the gutter. Red when `chars().count()` is used.
    #[test]
    fn cjk_authors_and_emoji_summaries_count_their_cells() {
        let n = measured("\u{5C71}\u{7530}\u{592A}\u{90CE}", "\u{1F419} octopus");
        assert_eq!(n.of(LogColumn::Author), 8 + 1);
        assert_eq!(n.of(LogColumn::Message), 2 + 1 + 7 + 1);
    }

    /// A header wider than every row still sets the width.
    #[test]
    fn a_header_wider_than_its_rows_wins() {
        let n = measured("A", "x");
        assert_eq!(
            n.of(LogColumn::Author),
            (display_width(LogColumn::Author.title()) + 1) as u16
        );
        assert_eq!(
            n.of(LogColumn::Ref),
            (display_width(LogColumn::Ref.title()) + 1)
                .max(LogColumn::Ref.drag_min_width() as usize) as u16
        );
    }

    #[test]
    fn avatars_add_the_badge_and_a_space() {
        let off = author_width(
            "Ada",
            "a@x",
            avatar::LogAvatarMode::Off,
            gitpoulpi_config::AvatarFallback::Initials,
        );
        let glyph = author_width(
            "Ada",
            "a@x",
            avatar::LogAvatarMode::Glyph,
            gitpoulpi_config::AvatarFallback::Initials,
        );
        let picture = author_width(
            "Ada",
            "a@x",
            avatar::LogAvatarMode::Picture,
            gitpoulpi_config::AvatarFallback::Initials,
        );
        assert_eq!((off, glyph, picture), (3, 5, 6));
    }

    #[test]
    fn graph_is_never_fitted_into_the_compact_picture() {
        let n = measured("A", "x");
        assert!(usize::from(n.of(LogColumn::Graph)) > crate::graph::COMPACT_W);
    }
}
