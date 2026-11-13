//! The right panel's commit detail: its layout and chrome, the View
//! Changes button, and what a click on a changed-file row does.

use super::*;

/// The sort toggle's labels (leading spaces keep its badge off the glyphs).
pub(crate) const SORT_LABEL: &str = "  A\u{2193}Z ";
pub(crate) const SORT_LABEL_DESC: &str = "  Z\u{2193}A ";

/// Fixed rows at the top of the changed-files pane, below its border and
/// above the list: the sort / Path|Tree / View-all row and the filter.
pub(crate) const FILES_HEADER_ROWS: u16 = 2;

/// The room the changed-files pane is always left, whatever the message
/// block asks for: its two borders, its options rows plus two list rows.
/// The message block gives way first (down to one row).
pub(crate) const MIN_FILES_HEIGHT: u16 = 2 + FILES_HEADER_ROWS + 2;

/// Rows of the details box that do not scroll or resize: the `commit:`
/// header, the two author lines, the counts row and the message block's two
/// borders. Add the message block's content rows and the box's own two
/// border rows to get its height (`commit_detail_height`).
const FIXED_INNER_ROWS: u16 = 1 + 2 + 1 + 2;

/// The most content rows the message block takes on its own (before the
/// user drags it taller).
pub(crate) const AUTO_MESSAGE_ROWS: u16 = 8;

/// Where each part of the commit-detail pane landed.
///
/// Every mouse site resolves clicks against these rects and the tests pin
/// the row arithmetic against them, so the geometry is computed once here
/// (as the Status pane's `status_section_areas` does).
///
/// Top to bottom, the details box: `commit: <sha>`; the author (avatar,
/// name, `authored <date>`, `parent: <sha>` on the right); the change
/// counts; a bordered block with the summary and the description. Then the
/// changed-files pane: the sort / Path|Tree / View-all-files row and the
/// filter, fixed at its top, and the file list below them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CommitDetailLayout {
    /// The framed "commit details" box. Doubles as the click guard that
    /// keeps a click up here from falling through to the Status branch.
    pub(crate) detail_area: Rect,
    /// The `commit: <short sha>` line.
    pub(crate) header_row: Rect,
    /// The message block, border included. Its bottom border row is the
    /// splitter that resizes it.
    pub(crate) message_area: Rect,
    /// Its content band (summary, description).
    pub(crate) message_inner: Rect,
    /// The two author lines (name + parent, `authored <date>`).
    pub(crate) author_area: Rect,
    /// The `✎ 2 modified ...` counts.
    pub(crate) counts_row: Rect,
    /// The `sort / Path / Tree / View all files` row: the files pane's first
    /// inner row.
    pub(crate) toolbar_row: Rect,
    /// The `Filter files:` row, right under it.
    pub(crate) filter_row: Rect,
    /// The changed-files sub-pane, border included.
    pub(crate) files_area: Rect,
    /// Its content band - what the `List` is rendered into. Below the
    /// border and the [`FILES_HEADER_ROWS`] options rows.
    pub(crate) files_inner: Rect,
}

/// Height of the framed commit-details box for `message_rows` rows of
/// message.
pub(crate) fn commit_detail_height(message_rows: u16) -> u16 {
    2 + FIXED_INNER_ROWS + message_rows
}

/// The most message rows a right column of `status_height` rows can give
/// the message block while still leaving the file list its minimum.
pub(crate) fn max_message_rows(status_height: u16) -> u16 {
    status_height
        .saturating_sub(MIN_FILES_HEIGHT + 2 + FIXED_INNER_ROWS)
        .max(1)
}

/// The message block's content rows: what the user dragged it to
/// (`user`, `0` for "never"), else what the text needs up to
/// [`AUTO_MESSAGE_ROWS`]; never below one row nor more than the column
/// can spare.
pub(crate) fn message_rows(status_height: u16, text_lines: usize, user: u16) -> u16 {
    let wanted = if user > 0 {
        user
    } else {
        (text_lines.min(AUTO_MESSAGE_ROWS as usize) as u16).max(1)
    };
    wanted.clamp(1, max_message_rows(status_height))
}

/// [`message_rows`] for a comparison's cards: what they need up to a third
/// of the column (the files keep most of the space) and at least four rows,
/// else what the user dragged the splitter to.
pub(crate) fn compare_card_rows(status_height: u16, card_lines: usize, user: u16) -> u16 {
    let wanted = if user > 0 {
        user
    } else {
        (card_lines.min((status_height / 3).max(4) as usize) as u16).max(1)
    };
    wanted.clamp(1, max_message_rows(status_height))
}

/// Splits the right column into the details box and the changed-files
/// sub-pane below it. `message_rows` is the message block's content rows
/// (see [`message_rows`]).
pub(crate) fn commit_detail_layout(status_area: Rect, message_rows: u16) -> CommitDetailLayout {
    let [detail_area, files_area] = Layout::vertical([
        Constraint::Length(commit_detail_height(message_rows)),
        Constraint::Min(0),
    ])
    .areas(status_area);
    // Ask the block itself where its content goes instead of writing
    // `+1`/`-2` by hand, so no toolbar row is inset beside a border nobody
    // paints.
    let detail_inner = Block::default().borders(Borders::ALL).inner(detail_area);
    let [header_row, author_area, counts_row, message_area] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(2),
        Constraint::Length(1),
        Constraint::Length(message_rows.saturating_add(2)),
    ])
    .areas(detail_inner);
    let message_inner = Block::default().borders(Borders::ALL).inner(message_area);
    let files_content = Block::default().borders(Borders::ALL).inner(files_area);
    let [toolbar_row, filter_row, files_inner] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(0),
    ])
    .areas(files_content);
    CommitDetailLayout {
        detail_area,
        header_row,
        message_area,
        message_inner,
        author_area,
        counts_row,
        toolbar_row,
        filter_row,
        files_area,
        files_inner,
    }
}

/// The list's own bordered-equivalent rect inside the files pane: the pane
/// without its options rows. Row hit tests (`row_to_index`, top offset one)
/// and the scrollbar track take this, never `files_area` itself, or a click
/// lands [`FILES_HEADER_ROWS`] rows off.
pub(crate) fn files_list_body(files_area: Rect) -> Rect {
    let skip = FILES_HEADER_ROWS.min(files_area.height);
    Rect::new(
        files_area.x,
        files_area.y + skip,
        files_area.width,
        files_area.height - skip,
    )
}

/// Word-wraps `text` to `width` columns: each source line separately,
/// words kept whole, a word longer than the width broken. Blank source
/// lines survive (paragraph breaks in a commit body).
pub(crate) fn wrap_text(text: &str, width: usize) -> Vec<String> {
    wrap_marked(text, width)
        .into_iter()
        .map(|(line, _)| line)
        .collect()
}

/// [`wrap_text`] with, per line, whether a word broken at the line's end
/// goes on in the next one (an address wider than the pane).
pub(crate) fn wrap_marked(text: &str, width: usize) -> Vec<(String, bool)> {
    let width = width.max(1);
    let mut out: Vec<(String, bool)> = Vec::new();
    for source in text.lines() {
        if source.trim().is_empty() {
            out.push((String::new(), false));
            continue;
        }
        let mut line = String::new();
        let mut line_w = 0usize;
        for word in source.split_whitespace() {
            let mut word = word.to_string();
            let mut word_w = display_width(&word);
            // Break a word that cannot fit on a line of its own.
            while word_w > width {
                if line_w > 0 {
                    out.push((std::mem::take(&mut line), false));
                    line_w = 0;
                }
                let mut head = String::new();
                let mut head_w = 0;
                let mut rest = String::new();
                for ch in word.chars() {
                    let w = display_width(&ch.to_string());
                    if rest.is_empty() && head_w + w <= width {
                        head.push(ch);
                        head_w += w;
                    } else {
                        rest.push(ch);
                    }
                }
                out.push((head, true));
                word = rest;
                word_w = display_width(&word);
            }
            if line_w == 0 {
                line = word;
                line_w = word_w;
            } else if line_w + 1 + word_w <= width {
                line.push(' ');
                line.push_str(&word);
                line_w += 1 + word_w;
            } else {
                out.push((std::mem::take(&mut line), false));
                line = word;
                line_w = word_w;
            }
        }
        out.push((line, false));
    }
    out
}

/// A commit message as the right panel's message block shows it: the
/// summary (first line) wrapped and flagged, a blank line, then the
/// description wrapped. `(text, is_summary)` per display line.
#[cfg(test)]
pub(crate) fn message_display_lines(message: &str, width: usize) -> Vec<(String, bool)> {
    message_display_rows(message, width)
        .into_iter()
        .map(|(text, summary, _)| (text, summary))
        .collect()
}

/// [`message_display_lines`] with the "word goes on in the next line" flag
/// of [`wrap_marked`]: `(text, is_summary, continues)`.
pub(crate) fn message_display_rows(message: &str, width: usize) -> Vec<(String, bool, bool)> {
    let message = message.trim_end();
    let (summary, body) = match message.split_once('\n') {
        Some((s, b)) => (s.trim(), b.trim_matches('\n')),
        None => (message.trim(), ""),
    };
    let mut lines: Vec<(String, bool, bool)> = wrap_marked(summary, width)
        .into_iter()
        .map(|(l, c)| (l, true, c))
        .collect();
    if !body.trim().is_empty() {
        lines.push((String::new(), false, false));
        lines.extend(
            wrap_marked(body, width)
                .into_iter()
                .map(|(l, c)| (l, false, c)),
        );
    }
    lines
}

/// How many files of each kind a commit changed, in the order the counts
/// row shows them: `(modified, added, deleted, renamed)`.
pub(crate) fn change_counts(
    statuses: impl IntoIterator<Item = gitpoulpi_core::domain::FileStatus>,
) -> (usize, usize, usize, usize) {
    use gitpoulpi_core::domain::FileStatus as F;
    let mut c = (0, 0, 0, 0);
    for status in statuses {
        match status {
            F::Added | F::Untracked => c.1 += 1,
            F::Deleted => c.2 += 1,
            F::Renamed => c.3 += 1,
            F::Modified | F::Staged | F::Conflicted => c.0 += 1,
        }
    }
    c
}

/// Everything the commit-detail chrome needs painted, already resolved to
/// lines by the caller. Keeping the data lookups in the draw closure and the
/// framing here lets a `TestBackend` test assert the frame is unbroken
/// without a daemon.
pub(crate) struct CommitDetailChrome<'a> {
    pub(crate) title: String,
    pub(crate) border_style: Style,
    pub(crate) header: Line<'a>,
    /// The visible slice of the message (already scrolled).
    pub(crate) message: Vec<Line<'a>>,
    pub(crate) author: Vec<Line<'a>>,
    pub(crate) counts: Line<'a>,
    pub(crate) toolbar: Line<'a>,
    pub(crate) filter: Line<'a>,
    pub(crate) files_title: String,
}

/// Paints the two boxes of the commit-detail pane and their static
/// contents, and hands back the geometry.
///
/// The changed-files list is its own bordered sub-pane rather than the
/// bottom half of one tall frame, the shape the Status pane uses for
/// `Unstaged Files` / `Staged Files`, so panes 1 and 3 read alike.
pub(crate) fn render_commit_detail_chrome(
    frame: &mut Frame,
    status_area: Rect,
    message_rows: u16,
    chrome: &CommitDetailChrome<'_>,
) -> CommitDetailLayout {
    let layout = commit_detail_layout(status_area, message_rows);

    let detail_block = Block::default()
        .title(chrome.title.clone())
        .border_type(BorderType::Rounded)
        .borders(Borders::ALL)
        .border_style(chrome.border_style);
    frame.render_clipped(detail_block, layout.detail_area);
    frame.render_clipped(Paragraph::new(chrome.header.clone()), layout.header_row);
    frame.render_clipped(
        Block::default()
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(th().ui.muted)),
        layout.message_area,
    );
    frame.render_clipped(Paragraph::new(chrome.message.clone()), layout.message_inner);
    frame.render_clipped(Paragraph::new(chrome.author.clone()), layout.author_area);
    frame.render_clipped(Paragraph::new(chrome.counts.clone()), layout.counts_row);

    frame.render_clipped(
        Block::default()
            .title(chrome.files_title.clone())
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL)
            .border_style(chrome.border_style),
        layout.files_area,
    );
    // Part of the files pane: its first two inner rows, above the list.
    frame.render_clipped(Paragraph::new(chrome.toolbar.clone()), layout.toolbar_row);
    frame.render_clipped(Paragraph::new(chrome.filter.clone()), layout.filter_row);
    layout
}

/// What a left click on a row of the changed-files sub-pane does.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CommitFileClick {
    /// A folder row - collapse it, or expand it again.
    ToggleFolder(PathBuf),
    /// Open this file's diff in the central column.
    OpenDiff(usize),
    /// Put the log table back in the central column.
    CloseDiff,
    Nothing,
}

/// What a click on a file row of one of the right pane's file lists does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FileRowAction {
    /// Open the file (diff, or its content).
    Open,
    /// Back out to the Log - unless something is unsaved.
    Close,
    /// Do nothing: the second click of a double click.
    Ignore,
}

/// What a click on a file row does.
///
/// A single click toggles: on a file that is not showing it opens it; a single click on
/// the file already showing goes back out to the Log (the diff bar's ✕).
/// `second_of_double` is the second click of a double click on the same
/// cell: the first click already acted (opened, or closed), so this one is
/// ignored and a double click can never flip-flop.
///
/// All the right pane's file lists share this rule: the commit's changed
/// files, the working tree's Unstaged/Staged halves and View all files.
pub(crate) fn file_row_click(already_showing: bool, second_of_double: bool) -> FileRowAction {
    match (second_of_double, already_showing) {
        (true, _) => FileRowAction::Ignore,
        (false, true) => FileRowAction::Close,
        (false, false) => FileRowAction::Open,
    }
}

/// Decides what a click on a changed-file row of the commit-detail
/// sub-pane means. See `file_row_click` for the rule itself.
pub(crate) fn commit_file_click(
    row: Option<&TreeRow>,
    open_file_index: Option<usize>,
    second_of_double: bool,
) -> CommitFileClick {
    match row {
        Some(TreeRow::Folder { path, .. }) => CommitFileClick::ToggleFolder(path.clone()),
        Some(TreeRow::File { index, .. }) => {
            match file_row_click(open_file_index == Some(*index), second_of_double) {
                FileRowAction::Open => CommitFileClick::OpenDiff(*index),
                FileRowAction::Close => CommitFileClick::CloseDiff,
                FileRowAction::Ignore => CommitFileClick::Nothing,
            }
        }
        None => CommitFileClick::Nothing,
    }
}

// Which row is "already showing" is `file_panel::commit_open_path_index`
// by path, in the list the rows were built from.

/// Whether the Diff view is currently showing this working-tree file --
/// matched on `(source, path)`, never on `file_index`, because staging a
/// hunk changes which files are in each half and an index goes stale the
/// moment the view's own button is pressed.
pub(crate) fn working_diff_is_showing(
    state: Option<&DiffViewState>,
    section: StatusSection,
    path: &Path,
) -> bool {
    let wanted = match section {
        StatusSection::Unstaged => DiffSource::Unstaged,
        StatusSection::Staged => DiffSource::Staged,
    };
    state.is_some_and(|s| s.source == wanted && s.path == path)
}

/// The commit-detail changed-files list's menu actions, opened by
/// `.`/`Shift+F10`/Menu or a right click.
///
#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::*;

    /// The "Commit details" meta block is a sub-rect of `status_area`, so a
    /// click landing in it must not satisfy the `in_status` test and flip the
    /// whole right panel back to Status. The two halves must be disjoint and
    /// the meta half checked first.
    #[test]
    fn the_commit_detail_halves_partition_the_right_column_without_overlapping() {
        let status_area = Rect::new(80, 3, 40, 20);
        let layout = commit_detail_layout(status_area, 3);
        let (meta_area, files_area) = (layout.detail_area, layout.files_area);

        assert!(status_area.contains((85, 4).into()));
        assert!(meta_area.contains((85, 4).into()));
        assert!(
            !files_area.contains((85, 4).into()),
            "a point in the meta half must NOT also read as the files half"
        );
        // Every row of the column belongs to exactly one half.
        for y in status_area.y..status_area.y + status_area.height {
            let p = (85, y).into();
            assert_ne!(
                meta_area.contains(p),
                files_area.contains(p),
                "row {y} belongs to both halves or neither"
            );
        }
    }

    /// Renders the commit-detail chrome exactly the way the draw closure
    /// does, into a `TestBackend`, so the assertions below are about the
    /// real widget tree and not about a second copy of the layout maths.
    fn draw_commit_detail(area: Rect) -> ratatui::buffer::Buffer {
        let mut terminal = ratatui::Terminal::new(ratatui::backend::TestBackend::new(
            area.x + area.width,
            area.y + area.height,
        ))
        .expect("test backend");
        let chrome = CommitDetailChrome {
            title: " Commit details ".into(),
            border_style: Style::default(),
            header: Line::from("commit: fa477cb"),
            message: vec![Line::from("fix: a border with a hole in it")],
            author: vec![
                Line::from("Kai                      parent: 8a5f52a"),
                Line::from("authored 2026-09-19 08:52"),
            ],
            counts: Line::from("2 modified"),
            toolbar: Line::from("5 changed   Path  Tree   [ ] View all files"),
            filter: Line::from("Filter files: "),
            files_title: " Changed files (5)  click: diff, j/k browse, Esc: back ".into(),
        };
        terminal
            .draw(|frame| {
                render_commit_detail_chrome(frame, area, 3, &chrome);
            })
            .expect("draw");
        terminal.backend().buffer().clone()
    }

    fn symbol_at(buf: &ratatui::buffer::Buffer, x: u16, y: u16) -> String {
        buf.cell((x, y))
            .expect("cell in bounds")
            .symbol()
            .to_string()
    }

    /// The toolbar and `Filter files:` rows sit *between* two blocks (one
    /// bordered TOP|LEFT|RIGHT, one BOTTOM|LEFT|RIGHT) and have none of their
    /// own, so nothing painted a border on those two rows. The content is
    /// correctly inset by one column, so it looked like a hole rather than
    /// overdrawn text.
    #[test]
    fn the_commit_detail_pane_paints_an_unbroken_border_down_both_edges() {
        let area = Rect::new(0, 0, 60, 22);
        let buf = draw_commit_detail(area);
        for y in area.y..area.bottom() {
            for x in [area.x, area.right() - 1] {
                let s = symbol_at(&buf, x, y);
                assert!(
                    s.chars().all(|c| ('\u{2500}'..='\u{257F}').contains(&c)),
                    "row {y}, column {x} holds {s:?}, not a border glyph"
                );
            }
        }
    }

    /// The changed-files list is its own bordered sub-pane, the shape the
    /// Status pane gives `Unstaged Files` / `Staged Files`.
    #[test]
    fn the_changed_files_list_is_its_own_closed_sub_pane() {
        let area = Rect::new(0, 0, 60, 22);
        let layout = commit_detail_layout(area, 3);
        let buf = draw_commit_detail(area);

        // The details box closes on its own last row...
        let last = layout.detail_area.bottom() - 1;
        assert_eq!(symbol_at(&buf, area.x, last), "\u{2570}");
        assert_eq!(symbol_at(&buf, area.right() - 1, last), "\u{256F}");
        // ...and the file list opens its own directly underneath.
        assert_eq!(layout.files_area.y, last + 1);
        assert_eq!(symbol_at(&buf, area.x, layout.files_area.y), "\u{256D}");
        assert_eq!(
            symbol_at(&buf, area.right() - 1, layout.files_area.y),
            "\u{256E}"
        );
        // Its title says which sub-pane it is.
        let title: String = (area.x + 1..area.right() - 1)
            .map(|x| symbol_at(&buf, x, layout.files_area.y))
            .collect();
        assert!(
            title.contains("Changed files"),
            "the sub-pane must name itself; got {title:?}"
        );
    }

    /// The sub-pane has a top border and two fixed options rows, so every
    /// list row is three lower. `row_to_index`'s `top_offset` is an absolute
    /// offset from `area.y`, and getting it wrong (`where-we-are.md`'s
    /// repeat offender) makes every click land on the row above or below
    /// the pointer.
    #[test]
    fn a_click_in_the_changed_files_sub_pane_resolves_through_border_and_options_rows() {
        let status_area = Rect::new(80, 3, 40, 24);
        let layout = commit_detail_layout(status_area, 3);
        let top = 1 + FILES_HEADER_ROWS;
        assert_eq!(layout.toolbar_row.y, layout.files_area.y + 1);
        assert_eq!(layout.filter_row.y, layout.files_area.y + 2);
        assert_eq!(
            layout.files_inner.y,
            layout.files_area.y + top,
            "the border and the two options rows own the first rows"
        );
        let len = 5;
        // The border, the toolbar and the filter rows select nothing.
        for y in layout.files_area.y..layout.files_inner.y {
            assert_eq!(row_to_index(layout.files_area, 0, top, y, len), None);
        }
        // The first content row is the first file, and it is the row the
        // list widget actually paints into.
        assert_eq!(
            row_to_index(layout.files_area, 0, top, layout.files_inner.y, len),
            Some(0)
        );
        assert_eq!(
            row_to_index(layout.files_area, 0, top, layout.files_inner.y + 1, len),
            Some(1)
        );
        // Scrolled, the same row means a later file.
        assert_eq!(
            row_to_index(layout.files_area, 3, top, layout.files_inner.y, 20),
            Some(3)
        );
        // The options rows are inside the files pane, not the details box.
        assert!(layout
            .files_area
            .contains((85, layout.toolbar_row.y).into()));
        assert!(layout.files_area.contains((85, layout.filter_row.y).into()));
        assert!(!layout
            .detail_area
            .contains((85, layout.toolbar_row.y).into()));
        // The scrollbar's body starts where the list does.
        assert_eq!(
            files_list_body(layout.files_area).y + 1,
            layout.files_inner.y
        );
    }

    /// A single click opens the diff.
    #[test]
    fn a_single_click_on_a_changed_file_opens_its_diff() {
        let row = TreeRow::File {
            depth: 0,
            label: "src/lib.rs".into(),
            index: 2,
        };
        assert_eq!(
            commit_file_click(Some(&row), None, false),
            CommitFileClick::OpenDiff(2)
        );
        // With another file's diff already open, a single click switches to
        // this one.
        assert_eq!(
            commit_file_click(Some(&row), Some(7), false),
            CommitFileClick::OpenDiff(2)
        );
    }

    /// A single click on the row already on screen goes back to the Log; the
    /// second click of a double click is ignored, so it can't flip-flop.
    #[test]
    fn a_single_click_on_the_file_already_showing_closes_the_diff() {
        let row = TreeRow::File {
            depth: 0,
            label: "src/lib.rs".into(),
            index: 2,
        };
        assert_eq!(
            commit_file_click(Some(&row), Some(2), false),
            CommitFileClick::CloseDiff
        );
        assert_eq!(
            commit_file_click(Some(&row), Some(2), true),
            CommitFileClick::Nothing
        );
        assert_eq!(
            commit_file_click(Some(&row), None, true),
            CommitFileClick::Nothing
        );
    }

    /// The Diff view serves three sources (`Commit`, `Unstaged`, `Staged`)
    /// and `file_index` means something different in each, so a click in the
    /// changed-files list must not treat a working-tree diff at the same
    /// position as "this row's diff" and close it. The row is found by *path*
    /// in the list the rows were built from
    /// (`file_panel::commit_open_path_index`), the whole tree under View all
    /// files.
    #[test]
    fn only_this_commits_own_diff_counts_as_the_row_already_showing() {
        let commit = oid(9);
        let paths: Vec<PathBuf> = ["x.rs", "y.rs", "z.rs", "a.rs"]
            .iter()
            .map(PathBuf::from)
            .collect();
        let open = |s: &DiffViewState, c: Option<Oid>| {
            commit_open_path_index(Some(s), None, CentralView::Diff, c, &paths)
        };
        let mut shown = DiffViewState::for_commit(commit, 0, PathBuf::from("a.rs"));
        assert_eq!(open(&shown, Some(commit)), Some(3));
        // Another commit's files are in the list now.
        assert_eq!(open(&shown, Some(oid(10))), None);
        assert_eq!(open(&shown, None), None);
        // A working-tree diff of the same path is a different thing.
        shown.source = DiffSource::Unstaged;
        assert_eq!(open(&shown, Some(commit)), None);
        assert_eq!(
            commit_open_path_index(None, None, CentralView::Diff, Some(commit), &paths),
            None
        );
    }

    /// The working-changes halves share the commit list's rule.
    #[test]
    fn a_single_click_on_a_working_tree_file_opens_then_closes_its_diff() {
        let path = PathBuf::from("src/lib.rs");
        let showing = DiffViewState::for_working(DiffSource::Unstaged, path.clone());
        let is_showing = working_diff_is_showing(Some(&showing), StatusSection::Unstaged, &path);

        assert_eq!(file_row_click(false, false), FileRowAction::Open);
        assert_eq!(file_row_click(is_showing, false), FileRowAction::Close);
        assert_eq!(file_row_click(is_showing, true), FileRowAction::Ignore);
        assert_eq!(file_row_click(false, true), FileRowAction::Ignore);
    }

    /// Matched on `(source, path)`, never on `file_index`: staging a hunk
    /// moves a file between the halves, so the same path in the other half
    /// is a different diff and an index would go stale.
    #[test]
    fn the_working_diff_is_matched_by_half_and_path_not_by_position() {
        let path = PathBuf::from("src/lib.rs");
        let unstaged = DiffViewState::for_working(DiffSource::Unstaged, path.clone());
        assert!(working_diff_is_showing(
            Some(&unstaged),
            StatusSection::Unstaged,
            &path
        ));
        // Same path, other half.
        assert!(!working_diff_is_showing(
            Some(&unstaged),
            StatusSection::Staged,
            &path
        ));
        // Same half, other path.
        assert!(!working_diff_is_showing(
            Some(&unstaged),
            StatusSection::Unstaged,
            Path::new("src/other.rs")
        ));
        // A commit's diff is never one of the working halves.
        let commit = DiffViewState::for_commit(oid(4), 0, path.clone());
        assert!(!working_diff_is_showing(
            Some(&commit),
            StatusSection::Unstaged,
            &path
        ));
        assert!(!working_diff_is_showing(None, StatusSection::Staged, &path));
    }

    #[test]
    fn a_click_on_a_folder_row_still_collapses_it_rather_than_opening_a_diff() {
        let row = TreeRow::Folder {
            depth: 0,
            path: PathBuf::from("crates/gitpoulpi-tui"),
            label: "gitpoulpi-tui".into(),
            collapsed: false,
        };
        for double in [false, true] {
            assert_eq!(
                commit_file_click(Some(&row), Some(2), double),
                CommitFileClick::ToggleFolder(PathBuf::from("crates/gitpoulpi-tui"))
            );
        }
        assert_eq!(
            commit_file_click(None, None, false),
            CommitFileClick::Nothing
        );
    }

    /// After a tooltip/context menu that briefly covered part of the
    /// changed-files sub-pane closes, stale `|` border fragments and tinted
    /// patches must not be left in the pane's empty area below the last file.
    ///
    /// The cause is general: `ratatui::widgets::List` (like `Block` and
    /// `Paragraph`) only *re-styles* cells it has no item for
    /// (`buf.set_style(area, self.style)`) and never resets their *symbol*,
    /// so whatever last wrote a glyph there stays until something touches
    /// that cell again. The fix is a `Clear` immediately before the `List`
    /// every frame; this test reproduces the same two widgets in the same
    /// order to pin it independently of the draw closure.
    #[test]
    fn a_closed_popup_over_the_changed_files_list_leaves_no_stale_cells() {
        use ratatui::backend::TestBackend;
        use ratatui::Terminal;

        let area = Rect::new(0, 0, 60, 22);

        // Only two files: with a files pane many rows tall, most of
        // `files_inner` is empty -- exactly the area a tooltip can leave
        // stale glyphs in.
        let items = vec![
            ListItem::new("  modified  src/lib.rs"),
            ListItem::new("  added     src/new.rs"),
        ];

        let draw = |show_popup: bool, terminal: &mut Terminal<TestBackend>| {
            terminal
                .draw(|frame| {
                    let chrome = CommitDetailChrome {
                        title: " 2 file changes ".into(),
                        border_style: Style::default(),
                        header: Line::from("commit: abc123"),
                        message: vec![Line::from("a commit message")],
                        author: vec![Line::from("Kai"), Line::from("authored today")],
                        counts: Line::from("2 modified"),
                        toolbar: Line::from("2 changed   Path  Tree"),
                        filter: Line::from("Filter files: "),
                        files_title: " Changed files (2) ".into(),
                    };
                    let layout = render_commit_detail_chrome(frame, area, 3, &chrome);

                    if show_popup {
                        // A popup anchored against the *whole frame*, landing
                        // inside the files pane's empty tail instead of being
                        // bounded to the pane it covers.
                        let popup =
                            Rect::new(layout.files_inner.x + 2, layout.files_inner.y + 5, 20, 3);
                        frame.render_clipped(Clear, popup);
                        frame.render_clipped(
                            Paragraph::new("a commit message")
                                .block(Block::default().borders(Borders::ALL)),
                            popup,
                        );
                    }

                    // The fix under test: `Clear` immediately before the
                    // `List`, matching `event_loop.rs`.
                    frame.render_clipped(Clear, layout.files_inner);
                    frame.render_stateful_clipped(
                        List::new(items.clone()),
                        layout.files_inner,
                        &mut ListState::default(),
                    );
                })
                .unwrap();
        };

        let mut live = Terminal::new(TestBackend::new(area.width, area.height)).unwrap();
        draw(false, &mut live); // before the popup ever opened
        draw(true, &mut live); // popup open, over the empty tail
        draw(false, &mut live); // popup closed

        let mut expected = Terminal::new(TestBackend::new(area.width, area.height)).unwrap();
        draw(false, &mut expected);

        assert_eq!(
            live.backend().buffer(),
            expected.backend().buffer(),
            "cells under the closed popup were not fully repainted"
        );
    }

    fn buffer_text(buf: &ratatui::buffer::Buffer) -> Vec<String> {
        (0..buf.area.height)
            .map(|y| {
                (0..buf.area.width)
                    .map(|x| symbol_at(buf, x, y))
                    .collect::<String>()
            })
            .collect()
    }

    #[test]
    fn wrap_text_keeps_words_whole_breaks_long_ones_and_keeps_blank_lines() {
        assert_eq!(wrap_text("a b c", 10), vec!["a b c"]);
        assert_eq!(wrap_text("aaa bbb ccc", 7), vec!["aaa bbb", "ccc"]);
        assert_eq!(wrap_text("abcdefghij", 4), vec!["abcd", "efgh", "ij"]);
        assert_eq!(wrap_text("one\n\ntwo", 10), vec!["one", "", "two"]);
        assert_eq!(wrap_text("", 10), Vec::<String>::new());
    }

    #[test]
    fn a_message_is_summary_blank_line_then_description() {
        let lines = message_display_lines("fix: a long summary line\n\nBody one.\nBody two.\n", 12);
        let texts: Vec<&str> = lines.iter().map(|(t, _)| t.as_str()).collect();
        assert_eq!(
            texts,
            vec!["fix: a long", "summary line", "", "Body one.", "Body two."]
        );
        assert!(lines[0].1 && lines[1].1, "the summary lines are flagged");
        assert!(!lines[3].1, "the description is not");
        // No description, no blank line.
        assert_eq!(message_display_lines("just a summary", 40).len(), 1);
    }

    #[test]
    fn the_message_block_is_as_tall_as_its_text_until_capped_or_dragged() {
        // Automatic: the text's height, at least one row, at most the cap.
        assert_eq!(message_rows(40, 0, 0), 1);
        assert_eq!(message_rows(40, 3, 0), 3);
        assert_eq!(message_rows(40, 50, 0), AUTO_MESSAGE_ROWS);
        // Dragged: the user's height, but the file list keeps its room.
        assert_eq!(message_rows(40, 3, 12), 12);
        assert_eq!(message_rows(20, 3, 30), max_message_rows(20));
        assert_eq!(
            commit_detail_height(max_message_rows(20)) + MIN_FILES_HEIGHT,
            20
        );
    }

    #[test]
    fn the_details_box_stacks_header_author_counts_message_and_the_files_pane_holds_its_options() {
        let status_area = Rect::new(80, 3, 60, 40);
        let l = commit_detail_layout(status_area, 5);
        let inner = l.detail_area.inner(Margin::new(1, 1));
        assert_eq!(l.header_row.y, inner.y);
        assert_eq!(l.author_area.y, l.header_row.y + 1);
        assert_eq!(l.author_area.height, 2);
        assert_eq!(l.counts_row.y, l.author_area.bottom());
        assert_eq!(l.message_area.y, l.counts_row.y + 1);
        assert_eq!(l.message_area.height, 5 + 2);
        assert_eq!(l.message_inner.y, l.message_area.y + 1);
        assert_eq!(l.detail_area.bottom(), l.message_area.bottom() + 1);
        assert_eq!(l.files_area.y, l.detail_area.bottom());
        assert_eq!(l.toolbar_row.y, l.files_area.y + 1);
        assert_eq!(l.filter_row.y, l.toolbar_row.y + 1);
        assert_eq!(l.files_inner.y, l.filter_row.bottom());
    }

    #[test]
    fn change_counts_bucket_each_status() {
        use gitpoulpi_core::domain::FileStatus as F;
        assert_eq!(
            change_counts([F::Modified, F::Modified, F::Added, F::Deleted, F::Renamed]),
            (2, 1, 1, 1)
        );
    }

    /// The whole details box drawn through a `TestBackend`, in the documented
    /// order.
    #[test]
    fn the_details_box_renders_in_the_documented_order() {
        let area = Rect::new(0, 0, 50, 24);
        let mut terminal = ratatui::Terminal::new(ratatui::backend::TestBackend::new(50, 24))
            .expect("test backend");
        let chrome = CommitDetailChrome {
            title: " Commit details ".into(),
            border_style: Style::default(),
            header: Line::from("commit: 561f80"),
            message: message_display_lines(
                "fix: prevent mic re-muting and improve recording overlay placement\n\nSome telephony applications can re-mute the microphone.",
                46,
            )
            .into_iter()
            .take(5)
            .map(|(t, _)| Line::from(t))
            .collect(),
            author: vec![
                Line::from("Morgane Lefebvre                  parent: d4431f"),
                Line::from("authored 14.09.2026 @ 08:03"),
            ],
            counts: Line::from("\u{270E} 2 modified"),
            toolbar: Line::from("  A\u{2193}Z    Path  Tree   [ ] View all files"),
            filter: Line::from("Filter files: "),
            files_title: " Changed files (2) ".into(),
        };
        terminal
            .draw(|frame| {
                render_commit_detail_chrome(frame, area, 5, &chrome);
            })
            .expect("draw");
        let text = buffer_text(terminal.backend().buffer());
        for line in &text {
            println!("{line}");
        }
        let rows: Vec<&str> = text.iter().map(String::as_str).collect();
        let at = |needle: &str| rows.iter().position(|r| r.contains(needle)).expect(needle);
        assert!(at("commit: 561f80") < at("Morgane Lefebvre"));
        assert!(rows[at("Morgane Lefebvre")].contains("parent: d4431f"));
        assert_eq!(at("authored 14.09.2026"), at("Morgane Lefebvre") + 1);
        assert!(at("authored") < at("2 modified"));
        assert!(at("2 modified") < at("fix: prevent"));
        // The options are the files pane's first rows, under its title.
        assert!(at("fix: prevent") < at("Changed files"));
        assert!(at("Changed files") < at("Path"));
        assert_eq!(at("Filter files:"), at("Path") + 1);
    }
}
