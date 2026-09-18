//! Building a Log row: commit rows, the changes cell, stash rows, the ref connector.

use super::*;

/// A stash as the commit it is, for the right pane's commit detail: its
/// own commit, its base as the parent (so the file list is exactly what
/// the stash changed on top of its base), its message and date. The
/// "author" line names the stash, `stash@{n}`.
pub(crate) fn stash_as_commit(stash: &gitpoulpi_core::domain::Stash) -> Option<CommitNode> {
    Some(CommitNode {
        id: stash.commit?,
        parents: stash.base.into_iter().collect(),
        summary: stash.message.clone(),
        message: stash.message.clone(),
        author_name: format!("stash@{{{}}}", stash.id.0),
        author_email: String::new(),
        timestamp: stash.timestamp,
        refs: Vec::new(),
    })
}

/// Everything `commit_row_with_changes` reads to build one commit's row.
pub(crate) struct RowCtx<'a> {
    pub commit: &'a CommitNode,
    /// The commit's pills (`LogLayout::pills_for`).
    pub pills: &'a [RefPill],
    pub graph: Line<'static>,
    /// The Graph cell's node column (`graph::node_column`): the selection
    /// and hover band starts there. `None` bands the whole cell.
    pub graph_node: Option<usize>,
    pub lane_color: Color,
    pub widths: &'a LogColumnWidths,
    pub visible_order: &'a [LogColumn],
    pub is_selected: bool,
    pub is_hovered: bool,
    /// `Some(width)` only for the commit whose reveal popup is open: hides
    /// its `+n` badge AND pads its pill out to the popup's width so the two
    /// share a right edge.
    pub popup_fill_width: Option<usize>,
    /// True while a search is running and this commit is *not* a hit.
    /// Dimmed, never hidden - this is a search, not the Refs sidebar's
    /// solo filter, so the graph's shape and every lane stay intact.
    pub is_dimmed: bool,
    /// `ui.avatars` (`avatar::log_avatar_mode`): prefixes the Author cell
    /// with the author's badge - a one-character colored initial on a text
    /// terminal (`Glyph`), or a two-cell slot on an image terminal
    /// (`Picture`) holding the two-letter initials, which
    /// `avatar::LogAvatarImages` covers with the round picture once fetched.
    pub avatar_mode: avatar::LogAvatarMode,
    /// `ui.avatar_fallback`: identicon blocks or initials in that slot.
    pub avatar_fallback: gitpoulpi_config::AvatarFallback,
    /// `ui.ref_connector`, when it applies to this table at all (see
    /// `ref_connector_applies`): the thin line from the pill to the node,
    /// drawn only if this commit has a pill. See `RefConnector`.
    pub ref_connector: Option<RefConnector<'a>>,
    /// The "Changes" column's numbers once the daemon has answered; `None`
    /// draws a placeholder (the column never delays a paint).
    pub changes: Option<&'a gitpoulpi_core::domain::CommitStat>,
    /// The ref the commit is deduced to belong to, drawn as a faded pill when
    /// the commit has no pill of its own; `None` draws nothing.
    pub ghost: Option<&'a crate::graph::GhostRef>,
}

/// Builds one commit's table row in `visible_order` - the SHA/date/author
/// are deliberately the smaller, secondary columns (per the target
/// layout), the commit message is the primary one. Column order is a
/// runtime, user-draggable setting (see `HitTarget::LogColumnHeader`), so
/// this builds cells by column *identity*, not a fixed positional list.
pub(crate) fn commit_row_with_changes(ctx: RowCtx<'_>) -> Row<'static> {
    let RowCtx {
        commit,
        pills,
        graph,
        graph_node,
        lane_color,
        widths,
        visible_order,
        is_selected,
        is_hovered,
        popup_fill_width,
        is_dimmed,
        avatar_mode,
        avatar_fallback,
        ref_connector,
        changes,
        ghost,
    } = ctx;
    let ref_connector = ref_connector.filter(|_| !pills.is_empty());
    let graph = match ref_connector.and_then(|c| c.graph_row) {
        Some(row) => graph::with_ref_connector(graph, row),
        None => graph,
    };
    let mut graph = Some(graph); // Graph is rendered at most once; `Cell::from` needs it by value.
                                 // Applied to every column except Ref: the table's own
                                 // `row_highlight_style` sets no background, so a selected
                                 // row's ref pills keep their lane color. Hover uses the
                                 // subtler `HOVER_BG`.
    let sel_bg = |style: Style| {
        if is_selected {
            style.bg(th().ui.selection)
        } else if is_hovered {
            style.bg(th().ui.hover)
        } else {
            style
        }
    };
    let cells: Vec<Cell> = visible_order
        .iter()
        .map(|&col| {
            let w = widths.width_of(col) as usize;
            let cell = match col {
                LogColumn::Ref => {
                    let mut spans = ref_pill_spans(pills, w, lane_color, popup_fill_width);
                    if let (Some(ghost), true) = (ghost, pills.is_empty()) {
                        spans = ghost_pill_spans(
                            &ghost.pill,
                            w,
                            lane_color,
                            ghost.deleted_in.is_some(),
                        );
                    }
                    // The connector's Ref-column half: `─` from the end of
                    // the pill (or its `+n` badge) to the column's edge,
                    // where the Graph column's half picks it up. Nothing
                    // when the pill already fills the column.
                    if ref_connector.is_some() {
                        let used: usize = spans.iter().map(|s| display_width(&s.content)).sum();
                        if used < w {
                            spans.push(graph::ref_connector_span(lane_color, w - used));
                        }
                    }
                    Cell::from(Line::from(spans))
                }
                LogColumn::Graph => graph::banded_graph_cell(
                    graph
                        .take()
                        .unwrap_or_else(|| Line::from(Vec::<Span>::new())),
                    graph_node,
                    w,
                    sel_bg(Style::default()).bg,
                ),
                // One cell of the width is kept empty as a gutter: the
                // columns tile the pane exactly (see `log_table` on why
                // the table itself must not add spacing), so text allowed
                // to run to the last cell butts straight up against the
                // next column's first character -- `08:09` `341e7cb` read
                // as one word.
                LogColumn::Message => Cell::from(Span::styled(
                    truncate_with_ellipsis(&commit.summary, w.saturating_sub(1)),
                    // A non-matching row must read dimmed even where the
                    // terminal ignores `DIM` next to `BOLD` (Windows Terminal
                    // / conhost fold the two into one intensity, so the
                    // bold Message cell stayed bright until a hover repainted
                    // it): drop the bold and use the muted color outright.
                    if is_dimmed {
                        sel_bg(Style::default().fg(th().ui.muted))
                    } else {
                        sel_bg(Style::default().add_modifier(Modifier::BOLD))
                    },
                )),
                LogColumn::Author => {
                    if avatar_mode != avatar::LogAvatarMode::Off {
                        let (glyph, badge_color) = avatar::fallback_text(
                            avatar_fallback,
                            &commit.author_name,
                            &commit.author_email,
                            if avatar_mode == avatar::LogAvatarMode::Picture {
                                usize::from(avatar::SLOT_COLS)
                            } else {
                                1
                            },
                        );
                        // glyph + one gutter space before the name, same
                        // budgeting rule as every other column's trailing
                        // gutter cell (see the comment on `Message` above).
                        let name_budget = w
                            .saturating_sub(display_width(&glyph) + 1)
                            .saturating_sub(1);
                        Cell::from(Line::from(vec![
                            Span::styled(
                                glyph,
                                sel_bg(
                                    Style::default()
                                        .fg(badge_color)
                                        .add_modifier(Modifier::BOLD),
                                ),
                            ),
                            Span::styled(" ", sel_bg(Style::default())),
                            Span::styled(
                                truncate_with_ellipsis(&commit.author_name, name_budget),
                                sel_bg(Style::default().fg(th().ui.muted)),
                            ),
                        ]))
                    } else {
                        Cell::from(Span::styled(
                            truncate_with_ellipsis(&commit.author_name, w.saturating_sub(1)),
                            sel_bg(Style::default().fg(th().ui.muted)),
                        ))
                    }
                }
                LogColumn::Date => Cell::from(Span::styled(
                    truncate_with_ellipsis(
                        &format_timestamp(commit.timestamp),
                        w.saturating_sub(1),
                    ),
                    sel_bg(Style::default().fg(th().ui.muted)),
                )),
                LogColumn::Changes => Cell::from(Line::from(changes_spans(
                    changes,
                    w.saturating_sub(1),
                    sel_bg(Style::default().fg(th().ui.muted)),
                ))),
                LogColumn::Sha => Cell::from(Span::styled(
                    commit.id.to_hex()[..w.min(7)].to_string(),
                    sel_bg(Style::default().fg(th().ui.muted)),
                )),
            };
            // The cell's own background covers the column past the end of
            // its text, so a short message or author leaves no hole in the
            // selected/hovered row. Ref keeps its pills' lane color.
            if matches!(col, LogColumn::Ref | LogColumn::Graph) {
                cell
            } else {
                cell.style(sel_bg(Style::default()))
            }
        })
        .collect();
    // Built as ONE style: `Row::style` replaces rather than merges, so
    // chaining two calls would silently drop the first modifier.
    //
    // `DIM` is applied at the Row level rather than per-cell so it reaches
    // the Ref column too -- that one deliberately bypasses `sel_bg` to
    // keep its lane color, and a pill left at full brightness on a
    // non-matching row is precisely what makes a dimmed table look broken.
    let mut row_style = Style::default();
    if is_dimmed {
        row_style = row_style.add_modifier(Modifier::DIM);
    }
    if is_selected && !is_dimmed {
        row_style = row_style.add_modifier(Modifier::BOLD);
    }
    let row = Row::new(cells);
    if row_style == Style::default() {
        row
    } else {
        row.style(row_style)
    }
}

/// Most commits the "Changes" cache holds before it starts over; a few
/// bytes each, so this is a memory bound, not a freshness one.
const CHANGES_CACHE_MAX: usize = 20_000;

/// The Log "Changes" column's per-frame bookkeeping, run after the paint:
/// collects the daemon's answers, then asks for the visible commits that are
/// neither held nor already asked for. Costs nothing while the column is
/// hidden (`visible` is empty then), and never blocks: the request goes to
/// the link's own task.
pub(crate) fn request_visible_changes(
    link: &DaemonLink,
    held: &mut std::collections::HashMap<Oid, gitpoulpi_core::domain::CommitStat>,
    asked: &mut std::collections::HashSet<Oid>,
    visible: &[Oid],
) {
    while let Ok(outcome) = link.commit_stats_rx.try_recv() {
        // A failed batch stays marked as asked: retrying it every frame
        // would hammer the daemon, and the cells keep their placeholder.
        if let Ok(stats) = outcome.result {
            if held.len() + stats.len() > CHANGES_CACHE_MAX {
                held.clear();
                asked.clear();
            }
            held.extend(stats.into_iter().map(|s| (s.id, s)));
        }
    }
    let wanted: Vec<Oid> = visible
        .iter()
        .copied()
        .filter(|id| !held.contains_key(id) && !asked.contains(id))
        .collect();
    if !wanted.is_empty() {
        asked.extend(wanted.iter().copied());
        link.request_commit_stats(wanted);
    }
}

/// [`commit_row_with_changes`] without any "Changes" numbers: the
/// placeholder in that column, nothing else different.
#[cfg(test)]
#[allow(clippy::too_many_arguments)]
pub(crate) fn commit_row(
    commit: &CommitNode,
    refs: &[gitpoulpi_core::domain::Ref],
    graph: Line<'static>,
    lane_color: Color,
    widths: &LogColumnWidths,
    visible_order: &[LogColumn],
    is_selected: bool,
    is_hovered: bool,
    popup_fill_width: Option<usize>,
    is_dimmed: bool,
    avatar_mode: avatar::LogAvatarMode,
    avatar_fallback: gitpoulpi_config::AvatarFallback,
    ref_connector: Option<RefConnector<'_>>,
) -> Row<'static> {
    commit_row_with_changes(RowCtx {
        commit,
        pills: &ref_pills_for_commit(commit.id, refs),
        graph,
        graph_node: None,
        lane_color,
        widths,
        visible_order,
        is_selected,
        is_hovered,
        popup_fill_width,
        is_dimmed,
        avatar_mode,
        avatar_fallback,
        ref_connector,
        changes: None,
        ghost: None,
    })
}

/// The text of a "Changes" cell that fits `width` cells: `3 files +12 -4`,
/// shortened to `3f +12 -4`, then `3f`, as the column narrows. `None`
/// (not loaded yet) is the muted placeholder `…`. Binary-only and empty
/// commits read `0 files`/`1 file` with no counts.
pub(crate) fn changes_spans(
    stat: Option<&gitpoulpi_core::domain::CommitStat>,
    width: usize,
    base: Style,
) -> Vec<Span<'static>> {
    let Some(stat) = stat else {
        return vec![Span::styled("\u{2026}", base)];
    };
    let files = |long: bool| match (long, stat.files) {
        (true, n) => gitpoulpi_i18n::msg::log_table::changes_files(n as u64),
        (false, n) => gitpoulpi_i18n::msg::log_table::changes_files_short(n),
    };
    let lines = stat.insertions > 0 || stat.deletions > 0;
    let plus = format!("+{}", stat.insertions);
    let minus = format!("-{}", stat.deletions);
    let candidates = [files(true), files(false)];
    for head in &candidates {
        let full = display_width(head)
            + if lines {
                1 + plus.len() + 1 + minus.len()
            } else {
                0
            };
        if full <= width {
            let mut spans = vec![Span::styled(head.clone(), base)];
            if lines {
                spans.push(Span::styled(" ", base));
                spans.push(Span::styled(plus, base.fg(th().chrome.stat_add)));
                spans.push(Span::styled(" ", base));
                spans.push(Span::styled(minus, base.fg(th().chrome.stat_del)));
            }
            return spans;
        }
    }
    vec![Span::styled(
        truncate_with_ellipsis(&candidates[1], width),
        base,
    )]
}

/// How a commit row draws the ref-to-commit connector (`ui.ref_connector`).
/// Passed as `Some` only when the setting is on and
/// [`ref_connector_applies`] to the table; `commit_row` then draws it on
/// the rows that carry a pill and leaves every other row alone.
#[derive(Debug, Clone, Copy)]
pub(crate) struct RefConnector<'a> {
    /// The row's graph parts, for the Graph column's half - `Some` under
    /// the text renderer only. Under the image renderer the Graph column
    /// is blanks under a picture, and the picture draws its own one-pixel
    /// line (`graph::RasterRow::ref_connector`); the Ref column's `─`
    /// padding is text in both modes.
    pub(crate) graph_row: Option<&'a graph::GraphRow>,
}

/// Whether the connector can be drawn at all in this column order: only
/// when the Ref column sits **immediately left of** the Graph column (the
/// default order). Columns are user-reorderable; with anything between
/// the two, or either one hidden, a line from the pill would have to run
/// through another column's text, so none is drawn.
pub(crate) fn ref_connector_applies(enabled: bool, visible_order: &[LogColumn]) -> bool {
    enabled
        && visible_order
            .windows(2)
            .any(|w| w == [LogColumn::Ref, LogColumn::Graph])
}

/// One stash's Log row: its Graph cell is the box on its
/// own lane (laid out by `graph::layout`, like every other row's), its
/// Message the stash's own message (`WIP on main: 1a2b3c4 ...`) in muted
/// italics so it never reads as a commit, its Author the `stash@{n}`
/// handle, then the date and the stash commit's short SHA. No pills: a
/// stash is not a ref.
pub(crate) fn stash_log_row(
    stash: &gitpoulpi_core::domain::Stash,
    graph: Line<'static>,
    graph_node: Option<usize>,
    widths: &LogColumnWidths,
    visible_order: &[LogColumn],
    is_selected: bool,
    is_hovered: bool,
) -> Row<'static> {
    let sel_bg = |style: Style| {
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
        .map(|&col| {
            let w = widths.width_of(col) as usize;
            let cell = match col {
                LogColumn::Ref => Cell::from(""),
                LogColumn::Graph => graph::banded_graph_cell(
                    graph
                        .take()
                        .unwrap_or_else(|| Line::from(Vec::<Span>::new())),
                    graph_node,
                    w,
                    sel_bg(Style::default()).bg,
                ),
                LogColumn::Message => Cell::from(Span::styled(
                    truncate_with_ellipsis(&stash.message, w.saturating_sub(1)),
                    sel_bg(
                        Style::default()
                            .fg(th().ui.muted)
                            .add_modifier(Modifier::ITALIC),
                    ),
                )),
                LogColumn::Author => Cell::from(Span::styled(
                    truncate_with_ellipsis(
                        &format!("stash@{{{}}}", stash.id.0),
                        w.saturating_sub(1),
                    ),
                    sel_bg(Style::default().fg(th().ui.muted)),
                )),
                LogColumn::Date => Cell::from(Span::styled(
                    truncate_with_ellipsis(&format_timestamp(stash.timestamp), w.saturating_sub(1)),
                    sel_bg(Style::default().fg(th().ui.muted)),
                )),
                LogColumn::Changes => Cell::from(""),
                LogColumn::Sha => Cell::from(Span::styled(
                    stash
                        .commit
                        .map(|c| c.to_hex()[..w.min(7)].to_string())
                        .unwrap_or_default(),
                    sel_bg(Style::default().fg(th().ui.muted)),
                )),
            };
            // Same as `commit_row`: no hole after short text.
            if matches!(col, LogColumn::Ref | LogColumn::Graph) {
                cell
            } else {
                cell.style(sel_bg(Style::default()))
            }
        })
        .collect();
    let row = Row::new(cells);
    if is_selected {
        row.style(Style::default().add_modifier(Modifier::BOLD))
    } else {
        row
    }
}
