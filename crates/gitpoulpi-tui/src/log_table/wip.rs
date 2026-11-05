//! The WIP row: its counters, inline message field, selection and what it shows.

use super::*;

/// Whether the Log's selection is on the WIP row.
pub(crate) fn wip_row_selected(
    status_result: &gitpoulpi_core::Result<WorkingTree>,
    log_state: &TableState,
) -> bool {
    has_wip_row(status_result) && log_state.selected() == Some(0)
}

/// What the right pane shows for the Log's current selection.
///
/// **The WIP row always gets the working-changes view.** It is not a
/// commit - there is no message, author or diff to detail - and what the
/// user wants from it is the Changed files
/// and the commit composer. Every place that moves the Log's selection sets
/// `CommitDetail`; this is applied once per frame on top of all of them,
/// so no path (arrows, PageUp, Home, a click, a sidebar jump, the palette)
/// can land on the WIP row and show an empty commit detail.
pub(crate) fn right_panel_for_log_selection(current: RightPanel, wip_selected: bool) -> RightPanel {
    if wip_selected {
        RightPanel::Status
    } else {
        current
    }
}

/// How many files the WIP row's counters report: `✎ n  + n`.
///
/// Counted per *file*, not per index/worktree half: a file both staged
/// and further modified is one change. A deletion on either side wins
/// (the file is going away), then an addition (new or untracked), and
/// everything else - modified, renamed, conflicted - is a modification.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct WipCounts {
    pub modified: usize,
    pub added: usize,
    pub deleted: usize,
}

pub(crate) fn wip_change_counts(tree: &WorkingTree) -> WipCounts {
    use gitpoulpi_core::domain::FileStatus;
    let mut counts = WipCounts::default();
    for entry in &tree.files {
        let sides = [entry.index, entry.worktree];
        if sides.contains(&Some(FileStatus::Deleted)) {
            counts.deleted += 1;
        } else if sides
            .iter()
            .any(|s| matches!(s, Some(FileStatus::Added | FileStatus::Untracked)))
        {
            counts.added += 1;
        } else {
            counts.modified += 1;
        }
    }
    counts
}

/// The counters as spans: `✎ 4  + 8  − 1`, each only when non-zero, each
/// in the color the status list uses for that kind of change.
///
/// `✎` (U+270E) is the maintainer's explicit ask and already appears in the ref
/// context menu. Measured 2026-09-24 it is in **none** of the terminal
/// fonts (Cascadia Code/Mono, CaskaydiaCove NFM, JetBrainsMono NFM,
/// Consolas), so the terminal draws it from a fallback font; it is a
/// plain single-width symbol with no emoji presentation and no combining
/// mark, so that costs its look, not the layout. `−` is U+2212 MINUS SIGN,
/// present in all of them.
pub(crate) fn wip_counter_spans(counts: WipCounts) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    for (n, glyph, color) in [
        (counts.modified, "\u{270E}", th().status.warning),
        (counts.added, "+", th().refs.local),
        (counts.deleted, "\u{2212}", th().chrome.wip_del),
    ] {
        if n == 0 {
            continue;
        }
        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            format!("{glyph} {n}"),
            Style::default().fg(color),
        ));
    }
    spans
}

/// The placeholder the WIP row's message field shows while it is empty --
/// The conventional wording.
pub(crate) const WIP_PLACEHOLDER: &str = "// WIP";

/// The columns of the WIP row's Message cell that are the summary field
/// itself: the cell minus the counters and a gutter.
///
/// The field's width is chrome, not content (`text_field::field_window`):
/// it is whatever the column has left after the counters and a gutter, so
/// the counters never slide as the summary grows.
pub(crate) fn wip_field_width(counts: WipCounts, width: usize) -> usize {
    let counters_width: usize = wip_counter_spans(counts)
        .iter()
        .map(|s| display_width(&s.content))
        .sum();
    // One trailing gutter cell, like every other Log column.
    width.saturating_sub(counters_width + 1)
}

/// The WIP row's Message cell: an inline commit-summary field, then the
/// change counters.
///
/// **The field is the commit composer's Summary** - the same `String`,
/// not a copy - so typing in either shows up in the other on the very
/// next frame, with nothing to keep in sync. `editing` is whether it has
/// the keyboard (a caret, and the field's own background); an empty
/// summary shows [`WIP_PLACEHOLDER`], dimmed and italic, unless it is
/// being edited.
pub(crate) fn wip_message_line(
    summary: &str,
    editing: bool,
    caret: Option<&text_edit::Caret>,
    counts: WipCounts,
    width: usize,
) -> Line<'static> {
    let counters = wip_counter_spans(counts);
    let field_width = wip_field_width(counts, width);
    let mut spans: Vec<Span<'static>> = Vec::new();
    if editing {
        // The shared editor's rendering: the caret can
        // sit anywhere and a selection shows.
        let field = Style::default()
            .fg(th().ui.text)
            .bg(crate::input_focus_bg());
        let look =
            text_field::EditLook::new(field, "_").caret_style(field.add_modifier(Modifier::BOLD));
        spans.extend(text_field::edit_line(summary, caret, field_width, true, look).spans);
    } else {
        let (shown, style) = if summary.is_empty() {
            (
                truncate_with_ellipsis(WIP_PLACEHOLDER, field_width),
                Style::default()
                    .fg(th().ui.muted)
                    .add_modifier(Modifier::ITALIC),
            )
        } else {
            (
                truncate_with_ellipsis(summary, field_width),
                Style::default()
                    .fg(th().ui.text)
                    .add_modifier(Modifier::BOLD),
            )
        };
        // Padded out to the field's width, so the counters sit where they
        // sit while editing too, and never slide as the text changes.
        let pad = field_width.saturating_sub(display_width(&shown));
        spans.push(Span::styled(shown, style));
        spans.push(Span::raw(" ".repeat(pad)));
    }
    // Counters only when they fit whole beside at least a sliver of the
    // field; a narrow column keeps the field, which is the thing you type
    // into.
    if field_width > 0 {
        spans.extend(counters);
    }
    Line::from(spans)
}

/// Where the WIP row's change counters are drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WipCountsAt {
    /// Inside the Message cell, after the summary field.
    Message,
    /// In the Changes column's cell, like a commit's changes.
    ChangesColumn,
    /// Nowhere.
    Nowhere,
}

/// The Changes column, when visible, owns the WIP counters; otherwise they
/// ride in the message when `ui.log_wip_counts_in_message` is on.
pub(crate) fn wip_counts_at(visible_order: &[LogColumn], in_message_setting: bool) -> WipCountsAt {
    if visible_order.contains(&LogColumn::Changes) {
        WipCountsAt::ChangesColumn
    } else if in_message_setting {
        WipCountsAt::Message
    } else {
        WipCountsAt::Nowhere
    }
}

/// The counters the Message cell shows (zero unless they ride there).
pub(crate) fn wip_message_counts(counts: WipCounts, at: WipCountsAt) -> WipCounts {
    if at == WipCountsAt::Message {
        counts
    } else {
        WipCounts::default()
    }
}

/// The WIP counters in the Changes column's cell: the same `✎ n + n − n`
/// glyphs and colors as the message, truncated to the column.
fn wip_changes_cell(counts: WipCounts, width: usize) -> Line<'static> {
    let mut spans = wip_counter_spans(counts);
    if let Some(first) = spans.first_mut() {
        first.content = "".into();
    }
    let mut used = 0usize;
    let budget = width.saturating_sub(1);
    let mut out: Vec<Span<'static>> = Vec::new();
    for span in spans {
        let w = display_width(&span.content);
        if used + w > budget {
            break;
        }
        used += w;
        out.push(span);
    }
    Line::from(out)
}

/// The WIP row itself: one terminal line like every commit, its Graph
/// cell the dotted marker on HEAD's lane, its Message cell
/// [`wip_message_line`], every other column blank.
#[allow(clippy::too_many_arguments)] // one call site, mirroring `commit_row`
pub(crate) fn wip_row(
    graph: Line<'static>,
    graph_node: Option<usize>,
    widths: &LogColumnWidths,
    visible_order: &[LogColumn],
    summary: &str,
    editing: bool,
    caret: Option<&text_edit::Caret>,
    counts: WipCounts,
    counts_at: WipCountsAt,
    is_selected: bool,
    is_hovered: bool,
) -> Row<'static> {
    let message_counts = wip_message_counts(counts, counts_at);
    let bg = |style: Style| {
        if is_selected {
            style.bg(th().ui.selection)
        } else if is_hovered {
            style.bg(th().ui.hover)
        } else {
            style
        }
    };
    let mut graph = Some(graph);
    let cells: Vec<Cell> = visible_order
        .iter()
        .map(|&col| match col {
            LogColumn::Graph => graph::banded_graph_cell(
                graph
                    .take()
                    .unwrap_or_else(|| Line::from(Vec::<Span>::new())),
                graph_node,
                widths.width_of(col) as usize,
                bg(Style::default()).bg,
            ),
            LogColumn::Message => Cell::from(wip_message_line(
                summary,
                editing,
                caret,
                message_counts,
                widths.width_of(col) as usize,
            ))
            .style(bg(Style::default())),
            LogColumn::Changes if counts_at == WipCountsAt::ChangesColumn => {
                Cell::from(wip_changes_cell(counts, widths.width_of(col) as usize))
                    .style(bg(Style::default()))
            }
            _ => Cell::from("").style(bg(Style::default())),
        })
        .collect();
    Row::new(cells)
}

/// Puts the Log's selection on `commit` and reports whether it could.
///
/// A node's index in `CommitGraph` is not its index in the table: the
/// pinned WIP row occupies index 0 whenever there are uncommitted
/// changes, and every stash row above the commit shifts it one further.
/// Getting this wrong selects a row above the one asked for, so it goes
/// through [`graph::LogRows`] and nowhere else.
pub(crate) fn select_log_commit(
    log_result: &gitpoulpi_core::Result<gitpoulpi_core::domain::CommitGraph>,
    rows: &graph::LogRows,
    log_state: &mut TableState,
    commit: Oid,
) -> bool {
    let Ok(graph) = log_result.as_ref() else {
        return false;
    };
    let Some(index) = graph.nodes.iter().position(|n| n.id == commit) else {
        return false;
    };
    if !rows.is_shown(index) {
        return false;
    }
    log_state.select(Some(rows.commit_display_index(index)));
    true
}
