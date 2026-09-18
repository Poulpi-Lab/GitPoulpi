//! The three-pane split: minimum/collapsed widths, divider positions and
//! their drag arithmetic, the resize grab strips (pane dividers and Log
//! column boundaries) and persisting the layout to the session.

use super::*;

/// Minimum width, in percent of the panes row, any one of the three main
/// panes can be resized down to.
pub(crate) const MIN_PANE_PERCENT: u16 = 10;
/// Dragging a divider past this raw percentage (before `MIN_PANE_PERCENT`
/// clamps it) collapses the pane instead of leaving a sliver pinned at the
/// minimum.
pub(crate) const DRAG_COLLAPSE_PERCENT: u16 = 4;
/// The width, in columns, of a collapsed pane's restore strip.
pub(crate) const PANE_COLLAPSED_WIDTH: u16 = 1;
// `DRAG_COLLAPSE_PERCENT` has to sit strictly below `MIN_PANE_PERCENT`: a
// drag already clamped to the minimum for a while collapses the pane, and if
// the constants were equal every ordinary drag to the minimum would collapse
// it immediately.
const _: () = assert!(DRAG_COLLAPSE_PERCENT < MIN_PANE_PERCENT);
/// How long a second click has to land within, at the same cell, to
/// count as a double-click (checkout-by-double-click on a ref row).
pub(crate) const DOUBLE_CLICK_WINDOW: Duration = Duration::from_millis(400);

/// What a resize grab strip resizes: pane divider 1 (Refs/Log) or 2
/// (Log/Status), or the Log column whose right-hand edge is under the
/// pointer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ResizeGrab {
    PaneDivider(u8),
    LogColumn(LogColumn),
    /// The bottom border of a Refs sidebar section (resizes its height).
    SidebarSection(SidebarSection),
    /// The bottom border of the commit message block (resizes it).
    CommitMessage,
    /// The splitter between the File History list and its diff.
    HistorySplit,
    /// The splitter between the commit column and the view beside it.
    CommitColumn,
    /// A boundary of the commit column's header: sizes a part of its rows.
    CommitColumnPart(crate::commit_column::Boundary),
}

/// A pane divider's grab strip: its own column, over the full height of the
/// panes row (a divider *is* a full-height seam).
pub(crate) fn pane_divider_grab_rect(panes_area: Rect, divider_x: u16) -> Rect {
    Rect::new(divider_x, panes_area.y, 1, panes_area.height)
}

/// A Log column boundary's grab strip, `boundary_x` being the first cell of
/// the column after it (`log_area.y` is the pane's top border, `+ 1` the
/// header). `ui.column_resize = "header"`: that one cell on the header row,
/// like a spreadsheet's. `"line"`: the gap cell before it (the last cell of
/// the column on its left), over the header and every row down to the bottom
/// border, so the line lit on hover and the drag start are the same cells
/// (`column_resize`).
pub(crate) fn log_boundary_grab_rect(log_area: Rect, boundary_x: u16) -> Rect {
    crate::column_resize::grab_rect(
        crate::column_resize::mode(),
        crate::column_resize::Edge::before(boundary_x),
        log_area.y + 1,
        (log_area.y + log_area.height).saturating_sub(1),
    )
}

/// Paints a resize grab strip into the frame, over exactly the cells
/// `resize_grab_at` named and no others: the buffer half of the invariant,
/// and the half every terminal shows regardless of `OSC 22`.
///
/// A solid ACCENT bar, not the subtler `HOVER_BG` used for rows: a pane
/// divider column *is* the pane's own border, already ACCENT-on-default
/// when that pane has focus, so a dark tint would be invisible exactly where
/// it is needed. Inverting reads clearly against a focused and an unfocused
/// border. Styling the existing cells rather than writing a glyph keeps the
/// border character already there.
pub(crate) fn paint_resize_strip(buf: &mut Buffer, strip: Rect) {
    // Every cell of the rect: a pane divider is one column tall, but the
    // commit-message and sidebar-section splitters are one *row* wide.
    let visible = strip.intersection(buf.area);
    for y in visible.y..visible.y + visible.height {
        for x in visible.x..visible.x + visible.width {
            buf[(x, y)].set_style(Style::default().bg(th().ui.accent).fg(th().ui.on_accent));
        }
    }
}

/// **The** resize predicate: is `(col, row)` on a grab strip, and if so
/// which one and over exactly which cells?
///
/// One function, one region, three consumers: the hover highlight
/// (`hovered_resize_strip`), the OS pointer shape (`desired_cursor_shape`)
/// and the mouse-down drag arming. They must not be three pieces of
/// arithmetic, or the highlight appears while the pointer merely
/// *approaches* a strip, promising a drag that does not start.
///
/// **The invariant: if it highlights, dragging it works; if dragging it
/// works, it highlights.** There is deliberately no grab *margin*: the
/// pointer has to be on the strip itself.
///
/// The column half is read back out of `hit_targets` rather than
/// recomputed, so it inherits the registration order `find_hit_target`
/// depends on (boundaries before header cells) and cannot drift from the
/// rect that was actually registered.
#[allow(clippy::too_many_arguments)]
pub(crate) fn resize_grab_at(
    hit_targets: &[(Rect, HitTarget)],
    col: u16,
    row: u16,
    panes_area: Rect,
    divider1_x: u16,
    divider2_x: u16,
    left_collapsed: bool,
    right_collapsed: bool,
) -> Option<(ResizeGrab, Rect)> {
    if let Some((rect, HitTarget::LogColumnBoundary(c))) = find_hit_entry(hit_targets, col, row) {
        return Some((ResizeGrab::LogColumn(c), rect));
    }
    if let Some((rect, HitTarget::SidebarSectionBoundary(s))) =
        find_hit_entry(hit_targets, col, row)
    {
        return Some((ResizeGrab::SidebarSection(s), rect));
    }
    if let Some((rect, HitTarget::CommitMessageBoundary)) = find_hit_entry(hit_targets, col, row) {
        return Some((ResizeGrab::CommitMessage, rect));
    }
    if let Some((rect, HitTarget::History(crate::history_view::HistoryHit::Divider))) =
        find_hit_entry(hit_targets, col, row)
    {
        return Some((ResizeGrab::HistorySplit, rect));
    }
    if let Some((rect, HitTarget::CommitColumn(crate::commit_column::ColumnHit::Divider))) =
        find_hit_entry(hit_targets, col, row)
    {
        return Some((ResizeGrab::CommitColumn, rect));
    }
    if let Some((
        rect,
        HitTarget::CommitColumn(crate::commit_column::ColumnHit::Boundary(boundary)),
    )) = find_hit_entry(hit_targets, col, row)
    {
        return Some((ResizeGrab::CommitColumnPart(boundary), rect));
    }
    // A collapsed pane has no divider to drag: its former grab strip is the
    // thin restore strip, handled by `HitTarget::RestorePane`. Offering a
    // drag there too would let the pointer resize a divider that is not
    // drawn.
    [
        (1u8, divider1_x, left_collapsed),
        (2u8, divider2_x, right_collapsed),
    ]
    .into_iter()
    .filter(|(_, _, collapsed)| !collapsed)
    .map(|(which, x, _)| (which, pane_divider_grab_rect(panes_area, x)))
    .find(|(_, rect)| rect.contains((col, row).into()))
    .map(|(which, rect)| (ResizeGrab::PaneDivider(which), rect))
}

/// The raw divider percentage implied by the pointer's column, before any
/// clamp. Shared by `compute_divider1_pct`/`compute_divider2_pct` and by
/// the collapse-on-drag check in the mouse-drag handler, which needs the
/// *unclamped* value: the clamped one cannot tell "dragged to exactly the
/// minimum" apart from "dragged well past it and pinned there", and only
/// the second should collapse the pane.
pub(crate) fn raw_divider_pct(col: u16, panes_area_x: u16, panes_area_width: u16) -> u16 {
    ((col.saturating_sub(panes_area_x)) as u32 * 100 / panes_area_width.max(1) as u32) as u16
}

/// New `divider1_pct` while dragging the Refs/Log divider, clamped so
/// Refs never shrinks below `MIN_PANE_PERCENT` and never crosses into
/// Log's own minimum. Pulled out of the mouse-drag handler for the same
/// reason as `find_hit_target`.
pub(crate) fn compute_divider1_pct(
    col: u16,
    panes_area_x: u16,
    panes_area_width: u16,
    divider2_pct: u16,
) -> u16 {
    let pct = raw_divider_pct(col, panes_area_x, panes_area_width);
    pct.clamp(
        MIN_PANE_PERCENT,
        divider2_pct.saturating_sub(MIN_PANE_PERCENT),
    )
}

/// The keyboard's way to drag a pane divider: one notch of the Refs or the
/// Status pane's width (`PANE_NUDGE` percent of the window).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum PaneResize {
    RefsWider,
    RefsNarrower,
    StatusWider,
    StatusNarrower,
}

/// What one notch changes a pane's width by, in percent of the window.
pub(crate) const PANE_NUDGE: u16 = 3;

impl PaneResize {
    /// The two dividers after the notch, each kept inside the same limits a
    /// drag keeps them in (`compute_divider1_pct` / `compute_divider2_pct`).
    pub(crate) fn apply(self, divider1_pct: u16, divider2_pct: u16) -> (u16, u16) {
        match self {
            PaneResize::RefsWider => (
                (divider1_pct + PANE_NUDGE).min(divider2_pct.saturating_sub(MIN_PANE_PERCENT)),
                divider2_pct,
            ),
            PaneResize::RefsNarrower => (
                divider1_pct
                    .saturating_sub(PANE_NUDGE)
                    .max(MIN_PANE_PERCENT),
                divider2_pct,
            ),
            // The Status pane is what is right of divider 2.
            PaneResize::StatusWider => (
                divider1_pct,
                divider2_pct
                    .saturating_sub(PANE_NUDGE)
                    .max(divider1_pct + MIN_PANE_PERCENT),
            ),
            PaneResize::StatusNarrower => (
                divider1_pct,
                (divider2_pct + PANE_NUDGE).min(100 - MIN_PANE_PERCENT),
            ),
        }
    }
}

/// New `divider2_pct` while dragging the Log/Status divider - see
/// `compute_divider1_pct`.
pub(crate) fn compute_divider2_pct(
    col: u16,
    panes_area_x: u16,
    panes_area_width: u16,
    divider1_pct: u16,
) -> u16 {
    let pct = raw_divider_pct(col, panes_area_x, panes_area_width);
    pct.clamp(divider1_pct + MIN_PANE_PERCENT, 100 - MIN_PANE_PERCENT)
}

/// Refs / Log / Status, split by the two divider percentages. The one place
/// the horizontal split is computed: the rendered pane edges and the hit
/// geometry that advertises them must come from the same solver.
///
/// `left_collapsed`/`right_collapsed` shrink Refs and/or Status to
/// `PANE_COLLAPSED_WIDTH` and hand every freed column to the Log pane. The
/// divider percentages are untouched while collapsed (nothing here
/// overwrites `divider1_pct`/`divider2_pct`), which is what makes restoring
/// bring back the previous width.
///
/// The uncollapsed case goes through ratatui's `Layout` solver (which
/// rounds, unlike a hand-rolled `width * pct / 100`, which floors; see
/// `pane_divider_xs`). Only the collapsed branch does its own arithmetic,
/// and it is exact by construction (three rects whose widths sum to
/// `panes_area.width`).
pub(crate) fn pane_areas(
    panes_area: Rect,
    divider1_pct: u16,
    divider2_pct: u16,
    left_collapsed: bool,
    right_collapsed: bool,
) -> [Rect; 3] {
    let uncollapsed: [Rect; 3] = Layout::horizontal([
        Constraint::Percentage(divider1_pct),
        Constraint::Percentage(divider2_pct - divider1_pct),
        Constraint::Percentage(100 - divider2_pct),
    ])
    .areas(panes_area);
    if !left_collapsed && !right_collapsed {
        return uncollapsed;
    }
    let [raw_refs, _, raw_status] = uncollapsed;
    let left_w = if left_collapsed {
        PANE_COLLAPSED_WIDTH.min(panes_area.width)
    } else {
        raw_refs.width
    };
    let right_w = if right_collapsed {
        PANE_COLLAPSED_WIDTH.min(panes_area.width.saturating_sub(left_w))
    } else {
        raw_status.width
    };
    let refs = Rect::new(panes_area.x, panes_area.y, left_w, panes_area.height);
    let status_x = (panes_area.x + panes_area.width).saturating_sub(right_w);
    let status = Rect::new(status_x, panes_area.y, right_w, panes_area.height);
    let log_x = panes_area.x + left_w;
    let log_w = status_x.saturating_sub(log_x);
    let log = Rect::new(log_x, panes_area.y, log_w, panes_area.height);
    [refs, log, status]
}

/// The screen columns of the two pane dividers: the seams where one pane's
/// block ends and the next one's begins, i.e. the Log pane's left border
/// and the Status pane's left border.
///
/// Asked of `pane_areas` rather than computed as
/// `panes_area.x + panes_area.width * pct / 100`: ratatui's solver *rounds*
/// where that formula *floors*, so they part company by a cell at most
/// widths (at a 193-column panes row with a 20/65 split, 38 against the
/// drawn seam at 39), which would put the grab zone, the resize pointer and
/// the highlight strip a column off the divider. `log_table` solves the same
/// defect for the Log's column edges.
pub(crate) fn pane_divider_xs(
    panes_area: Rect,
    divider1_pct: u16,
    divider2_pct: u16,
    left_collapsed: bool,
    right_collapsed: bool,
) -> (u16, u16) {
    let [_, log, status] = pane_areas(
        panes_area,
        divider1_pct,
        divider2_pct,
        left_collapsed,
        right_collapsed,
    );
    (log.x, status.x)
}

/// The strip to light while a pane divider is being dragged: the divider's
/// own column, recomputed from the percentage the drag has just written.
///
/// A held mouse button reports as `Drag`, never `Moved`, so the hover path
/// that normally lights the strip never runs mid-drag; without this the
/// highlight freezes at the column the drag started from.
pub(crate) fn dragged_divider_strip(
    panes_area: Rect,
    which: u8,
    divider1_pct: u16,
    divider2_pct: u16,
    left_collapsed: bool,
    right_collapsed: bool,
) -> Option<Rect> {
    let (d1x, d2x) = pane_divider_xs(
        panes_area,
        divider1_pct,
        divider2_pct,
        left_collapsed,
        right_collapsed,
    );
    let x = match which {
        1 => d1x,
        2 => d2x,
        _ => return None,
    };
    // Same builder the hover/arming predicate uses, so a dragged strip and
    // a hovered one are the same region.
    Some(pane_divider_grab_rect(panes_area, x))
}

/// Whether the Log's WIP row *becoming* selected should auto-restore a
/// collapsed Status pane: selecting it (click, `c`, keyboard, palette) is
/// meaningless with the pane that draws the changed files and composer
/// collapsed.
///
/// Edge-triggered on purpose: `was_selected` is the previous frame's
/// answer, so this fires once on the transition. A level-triggered "always
/// expand while selected" would re-expand a pane the user deliberately
/// collapsed again and the collapse button would look broken.
pub(crate) fn wip_selection_should_expand_status(
    wip_selected: bool,
    was_selected: bool,
    right_pane_collapsed: bool,
) -> bool {
    wip_selected && !was_selected && right_pane_collapsed
}

/// Opening a diff or a file view in the center collapses the Refs pane to
/// give the content room, and closing it brings the pane back, unless the
/// user toggled the pane meanwhile. The automatic collapse is never
/// persisted: `persisted` is what every `persist_pane_layout` call writes
/// for the left pane, so a save made while the pane is auto-collapsed still
/// records the user's own state.
///
/// Edge-triggered on the center's content opening and closing (`sync`, once
/// per frame). The user's toggle is *detected*: while auto-collapsed the
/// pane can only become expanded by the user, so "expanded while `saved` is
/// held" means the user took over, and no toggle site has to report it.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct AutoCollapse {
    /// The pane's state before the automatic collapse, while it holds.
    saved: Option<bool>,
    /// Whether a diff / file view was in the center last frame.
    was_open: bool,
    /// The editor's own "show the panels" is on: the pane is shown for now
    /// and the user's state is still restored when the center closes.
    peeking: bool,
    /// The pane's state in the Log, held while a non-Log view is in the
    /// center (see `enter_view` / `leave_view`).
    view_saved: Option<bool>,
}

impl AutoCollapse {
    /// Call once per frame, after the central view is known and before any
    /// automatic collapse: the first frame outside the Log records the
    /// pane's state as the Log had it.
    pub(crate) fn enter_view(&mut self, in_log: bool, collapsed: bool) {
        if !in_log && self.view_saved.is_none() {
            self.view_saved = Some(collapsed);
        }
    }

    /// Call once per frame, after every automatic collapse has been
    /// resolved: back in the Log, the recorded state returns whatever the
    /// pane did meanwhile. Returns the state to restore, if any.
    pub(crate) fn leave_view(&mut self, in_log: bool) -> Option<bool> {
        if in_log {
            self.view_saved.take()
        } else {
            None
        }
    }

    /// Call once per frame with whether a diff or a file view is in the
    /// center now. Returns `true` when it changed `left_collapsed` (the
    /// caller moves focus off a pane that just collapsed).
    pub(crate) fn sync(&mut self, content_open: bool, left_collapsed: &mut bool) -> bool {
        // The user expanded the pane while it was auto-collapsed: theirs.
        if self.saved.is_some() && !*left_collapsed && !self.peeking {
            self.saved = None;
        }
        let changed = match (self.was_open, content_open) {
            (false, true) => {
                self.saved = Some(*left_collapsed);
                let changed = !*left_collapsed;
                *left_collapsed = true;
                changed
            }
            (true, false) => match self.close_saved() {
                Some(before) => {
                    let changed = *left_collapsed != before;
                    *left_collapsed = before;
                    changed
                }
                None => false,
            },
            _ => false,
        };
        self.was_open = content_open;
        changed
    }

    /// The state saved before the automatic collapse, taken as the center
    /// closes (which also ends any peek).
    fn close_saved(&mut self) -> Option<bool> {
        self.peeking = false;
        self.saved.take()
    }

    /// Shows or hides the pane for now without making it the user's own
    /// choice (the merge editor's "show or hide the side panels"): while the
    /// automatic collapse holds, the state saved before it is still what
    /// comes back when the center closes.
    pub(crate) fn peek(&mut self, show: bool, collapsed: &mut bool) {
        if self.saved.is_some() {
            self.peeking = show;
        }
        *collapsed = !show;
    }

    /// The left pane's state to *save*: the user's own, not the automatic
    /// collapse. `left_collapsed` still collapsed while `saved` holds is
    /// the automatic state; anything else is the user's (a toggle at this
    /// very moment, before the next `sync` has seen it, included).
    pub(crate) fn persisted(&self, left_collapsed: bool) -> bool {
        if let Some(in_log) = self.view_saved {
            return in_log;
        }
        match self.saved {
            Some(before) if left_collapsed || self.peeking => before,
            _ => left_collapsed,
        }
    }
}

/// Writes the three-pane split to `session`'s file, whenever a divider drag
/// ends or a pane is collapsed/restored, never per drag frame. The
/// dividers are saved untouched by a collapse so restoring brings back the
/// previous width.
pub(crate) fn persist_pane_layout(
    session: &str,
    divider1_pct: u16,
    divider2_pct: u16,
    left_collapsed: bool,
    right_collapsed: bool,
) {
    // Read-modify-write: the sidebar's section heights live in the same
    // record and are saved by `session_store::save_sidebar_heights`.
    let mut layout = session_store::layout(session);
    layout.divider1_pct = divider1_pct;
    layout.divider2_pct = divider2_pct;
    layout.left_collapsed = left_collapsed;
    layout.right_collapsed = right_collapsed;
    session_store::save_layout(session, layout);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_log_keeps_its_right_pane_state_across_a_view() {
        let mut a = AutoCollapse::default();
        a.enter_view(true, false);
        assert_eq!(a.leave_view(true), None);
        a.enter_view(false, false);
        a.enter_view(false, true);
        assert!(!a.persisted(true), "the file keeps the Log's state");
        assert_eq!(a.leave_view(false), None, "view to view keeps the state");
        assert_eq!(a.leave_view(true), Some(false));
        assert_eq!(a.leave_view(true), None);
        a.enter_view(false, true);
        assert_eq!(a.leave_view(true), Some(true));
    }

    /// The left pane runs the same machine: the Log's state survives a view
    /// whatever the diff's own auto-collapse and the user did meanwhile.
    #[test]
    fn the_log_keeps_its_left_pane_state_across_the_auto_collapse() {
        let mut a = AutoCollapse::default();
        let mut collapsed = false;
        a.enter_view(false, collapsed);
        assert!(a.sync(true, &mut collapsed), "the diff collapses it");
        collapsed = false; // the user expands it in the diff
        a.sync(true, &mut collapsed);
        collapsed = true; // and collapses it again
        a.sync(true, &mut collapsed);
        assert!(!a.persisted(collapsed), "the file keeps the Log's state");
        a.sync(false, &mut collapsed);
        assert_eq!(a.leave_view(true), Some(false));
    }

    /// Selecting the WIP row while the Status pane is collapsed restores
    /// it, once, on the transition - not on every frame it stays
    /// selected (which would fight a deliberate re-collapse), and not at
    /// all when it was already open.
    #[test]
    fn selecting_the_wip_row_restores_a_collapsed_status_pane_once() {
        // Not selected -> selected, pane collapsed: expand.
        assert!(wip_selection_should_expand_status(true, false, true));
        // Still selected next frame: no repeat trigger.
        assert!(!wip_selection_should_expand_status(true, true, true));
        // Selected but the pane was never collapsed: nothing to do.
        assert!(!wip_selection_should_expand_status(true, false, false));
        // Not selected at all: nothing to do, collapsed or not.
        assert!(!wip_selection_should_expand_status(false, false, true));
        assert!(!wip_selection_should_expand_status(false, true, true));
    }

    /// Geometry for the resize tests: a Log pane inside a panes row, with
    /// its header hit targets registered exactly as `draw` registers them.
    struct ResizeGeometry {
        panes_area: Rect,
        log_area: Rect,
        hits: Vec<(Rect, HitTarget)>,
        offsets: Vec<(LogColumn, u16, u16)>,
    }

    fn resize_geometry() -> ResizeGeometry {
        let panes_area = Rect::new(0, 3, 100, 18);
        let log_area = Rect::new(20, 3, 40, 18);
        let widths = LogColumnWidths::for_area_width(
            log_area.width,
            &HashSet::new(),
            &std::collections::HashMap::new(),
        );
        let order = LogColumn::ALL.to_vec();
        let offsets = column_x_offsets(log_area.x, &order, &widths);
        let hits = log_header_hit_targets(log_area, &offsets);
        ResizeGeometry {
            panes_area,
            log_area,
            hits,
            offsets,
        }
    }

    /// The in-buffer half of the resize affordance, which is the half that
    /// works on every terminal regardless of `OSC 22` support.
    ///
    /// A column boundary lights **one header cell**; a pane divider lights
    /// its own column over the full height of the panes row.
    #[test]
    fn hovered_resize_strip_lights_column_boundaries_and_pane_dividers() {
        // The `header` mode (the first letter of the next name is the grab).
        crate::live_settings::install_column_resize(gitpoulpi_config::ColumnResize::Header);
        let ResizeGeometry {
            panes_area,
            log_area,
            hits,
            offsets,
        } = resize_geometry();
        let header_row = log_area.y + 1;
        let bx = log_boundary_x(&offsets, LogColumn::Graph).unwrap();

        // Over a Log column boundary, on the header row: one cell.
        assert_eq!(
            resize_grab_at(&hits, bx, header_row, panes_area, 18, 70, false, false),
            Some((
                ResizeGrab::LogColumn(LogColumn::Graph),
                Rect::new(bx, header_row, 1, 1)
            ))
        );
        // The same column one row down is a commit row: nothing at all.
        assert_eq!(
            resize_grab_at(&hits, bx, header_row + 1, panes_area, 18, 70, false, false),
            None
        );

        // On a pane divider: the full height of the panes row, unchanged.
        for (which, d) in [(1u8, 18u16), (2u8, 70u16)] {
            assert_eq!(
                resize_grab_at(&hits, d, 10, panes_area, 18, 70, false, false),
                Some((
                    ResizeGrab::PaneDivider(which),
                    Rect::new(d, panes_area.y, 1, panes_area.height)
                ))
            );
        }

        // One cell beside a divider is not "close enough": the pointer has
        // to be ON the strip. Nor is a divider's column outside the panes
        // row (over the toolbar).
        assert_eq!(
            resize_grab_at(&hits, 19, 10, panes_area, 18, 70, false, false),
            None
        );
        assert_eq!(
            resize_grab_at(&hits, 17, 10, panes_area, 18, 70, false, false),
            None
        );
        assert_eq!(
            resize_grab_at(&hits, 45, 10, panes_area, 18, 70, false, false),
            None
        );
        assert_eq!(
            resize_grab_at(&hits, 18, 0, panes_area, 18, 70, false, false),
            None
        );
    }

    /// The same thing again, but read off the **rendered buffer** rather
    /// than off the predicate's return value - the assertion a user could
    /// make by looking at the screen.
    ///
    /// (a) The cell under a column boundary in a *commit* row is not
    /// highlighted, while the cell at the same x in the header row is.
    /// (b) A pane divider still highlights over its full height.
    /// (c) The cell that highlights arms a drag; one cell away, neither.
    #[test]
    fn the_rendered_highlight_covers_the_header_cell_and_the_whole_divider_and_nothing_else() {
        // The `header` mode (the first letter of the next name is the grab).
        crate::live_settings::install_column_resize(gitpoulpi_config::ColumnResize::Header);
        use ratatui::backend::TestBackend;
        use ratatui::Terminal;

        let ResizeGeometry {
            panes_area,
            log_area,
            hits,
            offsets,
        } = resize_geometry();
        let (d1, d2) = (18u16, 70u16);
        let header_row = log_area.y + 1;
        let bx = log_boundary_x(&offsets, LogColumn::Graph).unwrap();

        // Renders the strip the predicate names for a pointer at
        // `(col, row)`, and reports whether a given cell came out lit.
        let lit_cells = |col: u16, row: u16| -> Vec<(u16, u16)> {
            let mut terminal =
                Terminal::new(TestBackend::new(100, panes_area.y + panes_area.height)).unwrap();
            terminal
                .draw(|f| {
                    if let Some((_, strip)) =
                        resize_grab_at(&hits, col, row, panes_area, d1, d2, false, false)
                    {
                        paint_resize_strip(f.buffer_mut(), strip);
                    }
                })
                .unwrap();
            let buf = terminal.backend().buffer().clone();
            let mut lit = Vec::new();
            for y in buf.area.y..buf.area.y + buf.area.height {
                for x in buf.area.x..buf.area.x + buf.area.width {
                    if buf[(x, y)].style().bg == Some(th().ui.accent) {
                        lit.push((x, y));
                    }
                }
            }
            lit
        };

        // (a) Hovering the boundary on the header row lights exactly one
        // cell -- the header one. The commit row below it stays dark, and
        // hovering that commit row lights nothing at all.
        assert_eq!(lit_cells(bx, header_row), vec![(bx, header_row)]);
        assert!(!lit_cells(bx, header_row).contains(&(bx, header_row + 1)));
        assert!(lit_cells(bx, header_row + 1).is_empty());
        assert!(lit_cells(bx, log_area.y + log_area.height - 2).is_empty());

        // (b) A pane divider lights its whole column, over the full height
        // of the panes row -- unchanged.
        let divider: Vec<(u16, u16)> = (panes_area.y..panes_area.y + panes_area.height)
            .map(|y| (d1, y))
            .collect();
        assert_eq!(lit_cells(d1, 10), divider);

        // (c) One cell either side of the divider: nothing lit, and
        // nothing armed either -- the two answers come from one call.
        for beside in [d1 - 1, d1 + 1] {
            assert!(lit_cells(beside, 10).is_empty());
            assert!(resize_grab_at(&hits, beside, 10, panes_area, d1, d2, false, false).is_none());
        }
        assert!(resize_grab_at(&hits, d1, 10, panes_area, d1, d2, false, false).is_some());
        assert!(resize_grab_at(&hits, bx, header_row, panes_area, d1, d2, false, false).is_some());
        assert!(
            resize_grab_at(&hits, bx + 1, header_row, panes_area, d1, d2, false, false).is_none()
        );
    }

    /// The invariant, swept over every cell of the panes row: **if it
    /// highlights, dragging it works; if dragging it works, it highlights**,
    /// and the OS pointer shape says the same at the same cells. One
    /// predicate, so the three cannot drift.
    #[test]
    fn what_highlights_is_exactly_what_can_be_dragged_and_what_shows_the_resize_cursor() {
        // The `header` mode (the first letter of the next name is the grab).
        crate::live_settings::install_column_resize(gitpoulpi_config::ColumnResize::Header);
        let ResizeGeometry {
            panes_area,
            log_area,
            hits,
            offsets,
        } = resize_geometry();
        let (d1, d2) = (18u16, 70u16);
        let boundaries: Vec<u16> = LogColumn::ALL
            .iter()
            .filter_map(|c| log_boundary_x(&offsets, *c))
            .collect();
        let header_row = log_area.y + 1;

        for row in 0..panes_area.y + panes_area.height + 2 {
            for col in 0..100u16 {
                let grab = resize_grab_at(&hits, col, row, panes_area, d1, d2, false, false);
                // What the render pass paints...
                let lit = grab.is_some();
                // ...what a mouse-down arms, from the same value...
                let arms = matches!(
                    grab,
                    Some((ResizeGrab::LogColumn(_) | ResizeGrab::PaneDivider(_), _))
                );
                // ...and what the OS pointer asks for.
                let resize_cursor = desired_cursor_shape(
                    find_hit_target(&hits, col, row).as_ref(),
                    None,
                    None,
                    None,
                    false,
                    grab.map(|(g, _)| g),
                ) == CursorShape::ColResize;
                assert_eq!(lit, arms, "lit/armed disagree at {col},{row}");
                assert_eq!(lit, resize_cursor, "lit/cursor disagree at {col},{row}");

                // And the region itself: a divider's own column inside the
                // panes row, or a boundary cell on the header row. Nothing
                // adjacent, nothing below the header.
                let expected = (row >= panes_area.y
                    && row < panes_area.y + panes_area.height
                    && (col == d1 || col == d2))
                    || (row == header_row && boundaries.contains(&col));
                assert_eq!(lit, expected, "wrong region at {col},{row}");
            }
        }
    }

    #[test]
    fn a_pane_notch_moves_one_divider_and_stays_inside_the_drag_limits() {
        use PaneResize as R;
        assert_eq!(R::RefsWider.apply(20, 70), (20 + PANE_NUDGE, 70));
        assert_eq!(R::RefsNarrower.apply(20, 70), (20 - PANE_NUDGE, 70));
        assert_eq!(
            R::RefsNarrower.apply(MIN_PANE_PERCENT, 70).0,
            MIN_PANE_PERCENT
        );
        assert_eq!(R::RefsWider.apply(58, 70).0, 70 - MIN_PANE_PERCENT);
        assert_eq!(R::StatusWider.apply(20, 70), (20, 70 - PANE_NUDGE));
        assert_eq!(R::StatusNarrower.apply(20, 70), (20, 70 + PANE_NUDGE));
        assert_eq!(R::StatusWider.apply(20, 30).1, 20 + MIN_PANE_PERCENT);
        assert_eq!(R::StatusNarrower.apply(20, 89).1, 100 - MIN_PANE_PERCENT);
    }

    #[test]
    fn compute_divider1_pct_tracks_the_mouse_but_respects_minimums() {
        // Dragging to the far left still leaves Refs at MIN_PANE_PERCENT,
        // never 0 or negative.
        assert_eq!(compute_divider1_pct(0, 0, 100, 65), MIN_PANE_PERCENT);
        // Mid-drag: roughly proportional to mouse position.
        assert_eq!(compute_divider1_pct(30, 0, 100, 65), 30);
        // Dragging past divider2 clamps to divider2's own minimum gap.
        assert_eq!(compute_divider1_pct(90, 0, 100, 65), 65 - MIN_PANE_PERCENT);
    }

    /// The resize affordance for a pane divider has to sit on the column
    /// ratatui actually drew the divider at, not one derived by hand from
    /// the same percentage: the solver rounds, the hand-rolled formula
    /// floors, and the two part company by a cell at most widths.
    #[test]
    fn pane_divider_xs_land_on_the_rendered_pane_edges() {
        for width in [60u16, 80, 100, 120, 137, 193, 200, 251, 300] {
            for (d1, d2) in [(20u16, 65u16), (10, 90), (33, 66), (45, 55), (15, 72)] {
                let panes = Rect::new(0, 3, width, 18);
                let [refs, log, status] = pane_areas(panes, d1, d2, false, false);
                assert_eq!(
                    pane_divider_xs(panes, d1, d2, false, false),
                    (log.x, status.x),
                    "divider columns must be the rendered pane edges \
                     at width {width}, {d1}%/{d2}%"
                );
                // Precondition for the sweep being worth anything: the
                // panes really do tile the row, so `log.x`/`status.x` are
                // the only two seams there are.
                assert_eq!(refs.x + refs.width, log.x);
                assert_eq!(log.x + log.width, status.x);
            }
        }
        // At a 193-column panes row the hand-rolled formula answers 38 where
        // ratatui draws the seam at 39.
        let panes = Rect::new(0, 3, 193, 18);
        assert_eq!(pane_divider_xs(panes, 20, 65, false, false).0, 39);
        assert_ne!(
            panes.x + panes.width * 20 / 100,
            39,
            "precondition: the formula this replaced really does disagree here"
        );
    }

    /// Collapsing a side pane hands its width to the Log pane and shrinks
    /// it to exactly `PANE_COLLAPSED_WIDTH`, but leaves the *other* side
    /// pane's width untouched - and every rect still tiles the row with
    /// no gap and no overlap, swept across a range of widths and
    /// percentages the way `pane_divider_xs_land_on_the_rendered_pane_edges`
    /// sweeps the uncollapsed case.
    #[test]
    fn a_collapsed_pane_shrinks_to_the_restore_strip_and_the_center_absorbs_the_rest() {
        for width in [40u16, 60, 80, 100, 137, 193, 251] {
            for (d1, d2) in [(20u16, 65u16), (10, 90), (33, 66), (45, 55)] {
                let panes = Rect::new(2, 3, width, 18);
                let [ref_normal, log_normal, status_normal] =
                    pane_areas(panes, d1, d2, false, false);

                // Left collapsed: Refs shrinks to the strip, Status is
                // untouched, Log absorbs exactly what Refs gave up.
                let [refs, log, status] = pane_areas(panes, d1, d2, true, false);
                assert_eq!(refs.width, PANE_COLLAPSED_WIDTH, "width {width} {d1}/{d2}");
                assert_eq!(refs.x, panes.x);
                assert_eq!(status, status_normal);
                assert_eq!(refs.x + refs.width, log.x);
                assert_eq!(log.x + log.width, status.x);
                assert_eq!(
                    log.width,
                    log_normal.width + ref_normal.width - PANE_COLLAPSED_WIDTH,
                    "the Log pane must absorb exactly what Refs gave up"
                );
                assert_eq!(refs.width + log.width + status.width, panes.width);

                // Right collapsed: the mirror image.
                let [refs, log, status] = pane_areas(panes, d1, d2, false, true);
                assert_eq!(status.width, PANE_COLLAPSED_WIDTH);
                assert_eq!(status.x + status.width, panes.x + panes.width);
                assert_eq!(refs, ref_normal);
                assert_eq!(refs.x + refs.width, log.x);
                assert_eq!(log.x + log.width, status.x);
                assert_eq!(refs.width + log.width + status.width, panes.width);

                // Both collapsed: the center takes everything but the two
                // one-column strips.
                let [refs, log, status] = pane_areas(panes, d1, d2, true, true);
                assert_eq!(refs.width, PANE_COLLAPSED_WIDTH);
                assert_eq!(status.width, PANE_COLLAPSED_WIDTH);
                assert_eq!(log.width, panes.width - 2 * PANE_COLLAPSED_WIDTH);
                assert_eq!(refs.x + refs.width, log.x);
                assert_eq!(log.x + log.width, status.x);
            }
        }
    }

    /// A collapsed pane's divider is gone: only the restore strip remains
    /// (its own hit target, not a `ResizeGrab`). Red without the
    /// `left_collapsed`/`right_collapsed` filter in `resize_grab_at`.
    #[test]
    fn a_collapsed_panes_divider_no_longer_offers_a_resize_grab() {
        let ResizeGeometry {
            panes_area, hits, ..
        } = resize_geometry();
        let (d1, d2) = pane_divider_xs(panes_area, 18, 70, false, false);

        assert!(resize_grab_at(&hits, d1, 10, panes_area, d1, d2, false, false).is_some());
        assert!(
            resize_grab_at(&hits, d1, 10, panes_area, d1, d2, true, false).is_none(),
            "divider 1 must not be grabbable once the Refs pane is collapsed"
        );
        assert!(resize_grab_at(&hits, d2, 10, panes_area, d1, d2, false, false).is_some());
        assert!(
            resize_grab_at(&hits, d2, 10, panes_area, d1, d2, false, true).is_none(),
            "divider 2 must not be grabbable once the Status pane is collapsed"
        );
    }

    /// Dragging a divider must carry its highlight with it. crossterm
    /// reports a held button as `Drag`, never `Moved`, so the strip cannot
    /// be left to the hover path: this is the function the drag arm calls,
    /// and it has to answer with the divider's new column every step.
    #[test]
    fn a_dragged_divider_carries_its_highlight_strip() {
        let panes = Rect::new(0, 3, 193, 18);
        let d1 = 20u16;
        let mut seen: Vec<u16> = Vec::new();
        for pointer in [80u16, 100, 120, 140, 160] {
            let d2 = compute_divider2_pct(pointer, panes.x, panes.width, d1);
            let strip =
                dragged_divider_strip(panes, 2, d1, d2, false, false).expect("divider 2 exists");
            assert_eq!(strip, Rect::new(strip.x, panes.y, 1, panes.height));
            assert_eq!(
                strip.x,
                pane_divider_xs(panes, d1, d2, false, false).1,
                "the strip must sit on the divider, not the pointer"
            );
            assert!(
                strip.x.abs_diff(pointer) <= 2,
                "the divider must follow the pointer: pointer {pointer}, strip {}",
                strip.x
            );
            seen.push(strip.x);
        }
        // Falsifies "the strip is recomputed at all": a frozen highlight
        // would report the same column for every step of the drag.
        assert!(
            seen.windows(2).all(|w| w[0] < w[1]),
            "the highlight must move with every step of the drag: {seen:?}"
        );
    }

    #[test]
    fn compute_divider2_pct_tracks_the_mouse_but_respects_minimums() {
        assert_eq!(compute_divider2_pct(0, 0, 100, 20), 20 + MIN_PANE_PERCENT);
        assert_eq!(compute_divider2_pct(70, 0, 100, 20), 70);
        assert_eq!(
            compute_divider2_pct(100, 0, 100, 20),
            100 - MIN_PANE_PERCENT
        );
    }

    // ---- the diff collapses the Refs pane ------------

    /// Open collapses, close restores, and what would be saved meanwhile
    /// is the user's own state. Red without `persisted`'s `saved` branch:
    /// the save while open writes `true`.
    #[test]
    fn opening_a_diff_collapses_the_refs_pane_and_closing_restores_it() {
        let mut auto = AutoCollapse::default();
        let mut left = false;
        assert!(!auto.sync(false, &mut left), "nothing open, nothing done");
        assert!(auto.sync(true, &mut left), "opening collapses");
        assert!(left);
        assert!(!auto.persisted(left), "the automatic collapse is not saved");
        assert!(!auto.sync(true, &mut left), "still open: no change");
        assert!(left, "stays collapsed while open (Diff <-> File View too)");
        assert!(auto.sync(false, &mut left), "closing restores");
        assert!(!left);
        assert!(!auto.persisted(left));
    }

    /// The pane already collapsed by the user stays collapsed after the
    /// diff closes - "restore the previous state", not "expand".
    #[test]
    fn a_pane_the_user_had_collapsed_stays_collapsed_after_the_diff() {
        let mut auto = AutoCollapse::default();
        let mut left = true;
        auto.sync(true, &mut left);
        assert!(left);
        assert!(auto.persisted(left));
        auto.sync(false, &mut left);
        assert!(left);
    }

    /// The user expands the pane while the diff is open: their choice
    /// stands when it closes, and it is what gets saved - both at the
    /// moment of the toggle (before `sync` has seen it) and after.
    /// Red without the "expanded while saved" detection in `sync`.
    #[test]
    fn toggling_the_pane_while_the_diff_is_open_wins_over_the_restore() {
        let mut auto = AutoCollapse::default();
        let mut left = false;
        auto.sync(true, &mut left);
        // The user restores it (strip / Ctrl+N [) ...
        left = false;
        assert!(!auto.persisted(left), "saved at the toggle: expanded");
        auto.sync(true, &mut left);
        // ... then collapses it again, deliberately.
        left = true;
        assert!(auto.persisted(left), "their collapse is saved");
        assert!(!auto.sync(false, &mut left), "closing leaves it alone");
        assert!(left, "the user's collapse stands");

        // And the other way round: collapsed by the user before, expanded
        // by the user while open, stays expanded after.
        let mut auto = AutoCollapse::default();
        let mut left = true;
        auto.sync(true, &mut left);
        left = false;
        auto.sync(true, &mut left);
        auto.sync(false, &mut left);
        assert!(!left, "the user's expand stands");
    }

    /// The merge editor's "show or hide the side panels": the pane comes
    /// and goes for now, and the state the user had before the editor is
    /// what returns on close (and what gets saved meanwhile).
    #[test]
    fn peeking_at_the_panels_while_the_editor_is_open_does_not_change_what_comes_back() {
        for before in [false, true] {
            let mut auto = AutoCollapse::default();
            let mut pane = before;
            auto.sync(true, &mut pane);
            assert!(pane, "the editor hides it");
            auto.peek(true, &mut pane);
            assert!(!pane, "shown for now");
            auto.sync(true, &mut pane);
            assert_eq!(
                auto.persisted(pane),
                before,
                "the user's own state is saved"
            );
            auto.peek(false, &mut pane);
            assert!(pane);
            auto.peek(true, &mut pane);
            auto.sync(false, &mut pane);
            assert_eq!(pane, before, "closing restores the user's state");
        }
    }
}
