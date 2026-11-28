//! Small UI primitives shared by every pane: popup placement and
//! frame clipping (`ClippedRender`, `anchored_popup`), display-width-aware
//! text truncation, list/table selection stepping, the row-to-index mapping
//! and timestamp formatting.

use super::*;

pub(crate) use crate::oid_text::short_oid;

/// A `percent_x` x `percent_y` box centered in `area`, for the help
/// overlay popup.
pub(crate) fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let [_, vertical, _] = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .areas(area);
    let [_, horizontal, _] = Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .areas(vertical);
    horizontal
}

/// Renders through the frame, but never outside it.
///
/// `ratatui::Frame::render_widget` does **not** clip: widgets index the
/// buffer by absolute coordinates through `Buffer::index_mut`, which panics
/// on a coordinate it does not own, so a rect one column past the edge
/// aborts the process mid-session.
///
/// Clipping to the frame turns that class of mistake into a widget that is a
/// little too short. The `debug_assert` keeps it honest: tests and debug
/// builds still fail loudly, naming the rect and the frame.
pub(crate) trait ClippedRender {
    fn render_clipped<W: Widget>(&mut self, widget: W, area: Rect);
    fn render_stateful_clipped<W: StatefulWidget>(
        &mut self,
        widget: W,
        area: Rect,
        state: &mut W::State,
    );
}

impl ClippedRender for Frame<'_> {
    fn render_clipped<W: Widget>(&mut self, widget: W, area: Rect) {
        let clipped = clip_to_frame(self.area(), area);
        if clipped.is_empty() {
            return;
        }
        self.render_widget(widget, clipped);
    }

    fn render_stateful_clipped<W: StatefulWidget>(
        &mut self,
        widget: W,
        area: Rect,
        state: &mut W::State,
    ) {
        let clipped = clip_to_frame(self.area(), area);
        if clipped.is_empty() {
            return;
        }
        self.render_stateful_widget(widget, clipped, state);
    }
}

/// `area` reduced to the part of it that lies inside `frame_area`.
pub(crate) fn clip_to_frame(frame_area: Rect, area: Rect) -> Rect {
    debug_assert!(
        frame_area.union(area) == frame_area || area.is_empty(),
        "a widget was asked to render at {area:?}, which is not inside the frame {frame_area:?}          - in a release build this is clipped, but it is a layout bug either way" // i18n-english
    );
    frame_area.intersection(area)
}

/// A popup of at most `width` x `height`, hanging just below
/// `(anchor_col, anchor_row)` and **guaranteed to lie inside
/// `frame_area`**.
///
/// Every anchored popup goes through this because getting it wrong aborts
/// the process: `Clear`, which each popup draws first, writes the rect cell
/// by cell through `Buffer::index_mut`, and a rect one column past the edge
/// panics.
///
/// Clamping only the *position* is not enough: when the popup is larger than
/// the frame, `saturating_sub` bottoms out at 0, `x`/`y` are pulled back to
/// the origin while `width`/`height` keep the popup's full size, and the
/// rect runs off the far edge (right-clicking in a window too small for the
/// menu killed the app). The size is clamped *before* the position.
pub(crate) fn anchored_popup(
    frame_area: Rect,
    anchor_col: u16,
    anchor_row: u16,
    width: u16,
    height: u16,
) -> Rect {
    let width = width.min(frame_area.width);
    let height = height.min(frame_area.height);
    // `anchor_row + 1`: a menu hangs *below* the row it was opened on, so it
    // never covers what was right-clicked. With no room below (a toolbar or a
    // status line at the bottom of the window) it flips: it ends on the
    // anchor row's top edge, above it, when there is room there. Saturating,
    // because an anchor on the last row is legitimate: with room nowhere the
    // clamp lifts the popup back over it, the only place left.
    let x = anchor_col.min(frame_area.x + frame_area.width.saturating_sub(width));
    let bottom = frame_area.y + frame_area.height;
    let below = anchor_row.saturating_add(1);
    let y = if below.saturating_add(height) > bottom
        && anchor_row >= frame_area.y.saturating_add(height)
        && anchor_row <= bottom
    {
        anchor_row - height
    } else {
        below.min(bottom.saturating_sub(height))
    };
    Rect::new(x.max(frame_area.x), y.max(frame_area.y), width, height)
}

/// Where a list opened by a menu row goes: a continuation of that row, like a
/// submenu. It starts at the parent's right edge on the row's own line (the
/// parent stays visible); with no room on the right it ends at the parent's
/// left edge; with room on neither side it covers the parent's right part, the
/// only place left. Always inside `bounds`.
pub(crate) fn flyout_rect(parent: Rect, row_y: u16, width: u16, height: u16, bounds: Rect) -> Rect {
    let width = width.min(bounds.width);
    let height = height.min(bounds.height);
    let x = if parent.right().saturating_add(width) <= bounds.right() {
        parent.right()
    } else if parent.x >= bounds.x.saturating_add(width) {
        parent.x - width
    } else {
        bounds.right().saturating_sub(width)
    };
    let y = row_y.min(bounds.bottom().saturating_sub(height));
    Rect::new(x.max(bounds.x), y.max(bounds.y), width, height)
}

pub(crate) fn move_selection(state: &mut ListState, len: usize, delta: i32) {
    state.select(move_index(state.selected(), len, delta));
}

/// Flips `value`'s membership in `set`: the show/hide and collapse/expand
/// toggle every pane runs from keyboard, mouse and the palette alike.
///
/// Inline, `if !set.remove(&v) { set.insert(v); }` as the only statement of
/// a `match` arm trips clippy's `collapsible_match`, which would have us fold
/// the *mutating* `remove` into a guard that silently makes the `match`
/// non-exhaustive.
pub(crate) fn toggle_in<T: Eq + std::hash::Hash>(set: &mut HashSet<T>, value: T) {
    if !set.remove(&value) {
        set.insert(value);
    }
}

pub(crate) fn move_table_selection(state: &mut TableState, len: usize, delta: i32) {
    state.select(move_index(state.selected(), len, delta));
}

/// Moves `state`'s offset by `delta` rows without touching `selected`,
/// clamped so the view cannot scroll past the last row or above the first.
///
/// `visible` is the number of content rows the viewport can show; without
/// it the clamp would let the offset run the whole list past the bottom
/// instead of stopping once the last row is on screen.
pub(crate) fn scroll_offset(offset: usize, len: usize, visible: usize, delta: i32) -> usize {
    let max_offset = len.saturating_sub(visible.max(1));
    (offset as i32 + delta).clamp(0, max_offset as i32) as usize
}

/// Which list/table a scrollbar drawn belongs to: the id `ListBarRegion`
/// carries so one mouse handler serves them all (the diff and File Insight
/// bars have their own `scrollbar::ScrollBar` ids, which also carry a
/// horizontal axis).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ListBar {
    Log,
    CommitFiles,
    /// The commit-detail pane's message block.
    CommitMessage,
    /// The About window's crate list (driven by `about.rs` itself).
    AboutCrates,
    /// The Activity Log window (driven by `activity_log/panel.rs` itself).
    ActivityLog,
    /// The jobs panel (driven by `jobs_ui/panel.rs` itself).
    JobsPanel,
    /// The Undo history / Reflog / Lost work panel's list.
    SafetyPanel,
    WipAll,
    /// The commit composer's Description box (focused, overflowing).
    CommitDescription,
    /// The File History list.
    History,
    /// The commit column left of the Diff / File View / Blame / Edit views.
    CommitColumn,
    Refs(SidebarSection),
    Status(StatusSection),
}

/// One scrollable list's bar geometry, resolved from the same numbers the
/// render pass drew it from: `area` is the list's **outer (bordered)**
/// rect (the track lives in its right border, see `scrollbar.rs`), `len`
/// its row count and `visible` how many rows the viewport really shows
/// (which is not always `area.height - 2`: the Log spends a row on its
/// header).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ListBarRegion {
    pub(crate) bar: ListBar,
    pub(crate) area: Rect,
    pub(crate) len: usize,
    pub(crate) visible: usize,
}

/// What a mouse-down on a list's scrollbar does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ListBarPress {
    /// Where within the thumb the pointer holds it (see
    /// `scrollbar::Track::drag_offset`) - carried into the drag.
    pub(crate) grab: i32,
    /// `Some(offset)` when the press landed on the *track* outside the
    /// thumb: the view jumps so the thumb is centered under the pointer
    /// (click-to-jump), and the same press then keeps
    /// dragging. `None` on the thumb itself: nothing moves until the
    /// pointer does.
    pub(crate) jump_to: Option<usize>,
    /// The press hit an end arrow: `jump_to` is the one-row step and no
    /// drag should start.
    pub(crate) arrow: bool,
}

impl ListBarRegion {
    fn track(&self, offset: usize) -> Option<scrollbar::Track> {
        scrollbar::Track::vertical(self.area, self.len, offset)
    }
}

/// A mouse-down at `(col, row)`: `None` when it is not on the bar (or
/// the list fits and draws none), else what to do about it.
pub(crate) fn list_bar_press(
    region: ListBarRegion,
    offset: usize,
    col: u16,
    row: u16,
) -> Option<ListBarPress> {
    let track = region.track(offset)?;
    if !track.rect.contains((col, row).into()) {
        return None;
    }
    let pos = (row - track.rect.y) as i32;
    if let Some(step) = track.arrow_step(pos as u16) {
        let max = region.len.saturating_sub(region.visible.max(1));
        return Some(ListBarPress {
            grab: 0,
            jump_to: Some((offset as isize + step).clamp(0, max as isize) as usize),
            arrow: true,
        });
    }
    if track.on_thumb(pos as u16) {
        return Some(ListBarPress {
            grab: pos - track.thumb_start as i32,
            jump_to: None,
            arrow: false,
        });
    }
    let grab = (track.thumb_len / 2) as i32;
    Some(ListBarPress {
        grab,
        jump_to: Some(list_bar_drag(region, offset, grab, row)),
        arrow: false,
    })
}

/// The offset a drag holding the thumb at `grab` puts the list at while
/// the pointer is on screen row `row` (which may be above or below the
/// track: it is clamped). Recomputed from the live region every event, so
/// content that reflowed mid-drag cannot be dragged against stale
/// geometry.
pub(crate) fn list_bar_drag(region: ListBarRegion, offset: usize, grab: i32, row: u16) -> usize {
    let Some(track) = region.track(offset) else {
        return 0;
    };
    let pos = row as i32 - track.rect.y as i32;
    let visible = region.visible.max(1);
    let new = track.drag_offset(region.len, visible, grab, pos);
    new.min(region.len.saturating_sub(visible))
}

/// Whether `index` lies inside the window `[offset, offset + visible)` --
/// i.e. whether it would actually be drawn this frame at that offset.
fn in_view(index: usize, offset: usize, visible: usize) -> bool {
    index >= offset && index < offset + visible.max(1)
}

/// A `ListState` to hand to the widget for *this frame's render only*, which
/// never lets ratatui's "keep the selection visible" logic
/// (`List::render`'s `get_items_bounds`) drag a wheel-scrolled offset back to
/// the selection.
///
/// `follow == true` (a keyboard move or a click just happened) passes
/// straight through: ratatui's default snaps the offset to the selection.
///
/// `follow == false` (the last thing was a wheel scroll) keeps
/// `state.offset()` where the wheel left it. `selected` still rides along,
/// and is still drawn, whenever it already lies inside that window; it is
/// hidden from the widget for this one frame otherwise, which only stops
/// ratatui dragging the *offset* back to reveal it.
///
/// The caller copies `.offset()` off the returned state into the persisted
/// one afterwards (needed for `follow == true`; a no-op copy otherwise).
pub(crate) fn list_render_state(state: &ListState, follow: bool, visible: usize) -> ListState {
    let mut render = *state;
    if !follow {
        if let Some(sel) = state.selected() {
            if !in_view(sel, state.offset(), visible) {
                // NOT `render.select(None)`: ratatui's `select` zeroes
                // `offset` whenever the new selection is `None` (see
                // `ListState::select`), the frozen offset this function
                // exists to protect. `selected_mut()` changes only `selected`.
                *render.selected_mut() = None;
            }
        }
    }
    render
}

/// The `TableState` counterpart of `list_render_state`, for the Log.
pub(crate) fn table_render_state(state: &TableState, follow: bool, visible: usize) -> TableState {
    let mut render = *state;
    if !follow {
        if let Some(sel) = state.selected() {
            if !in_view(sel, state.offset(), visible) {
                // See `list_render_state`'s comment: `select(None)` would
                // zero `offset` too.
                *render.selected_mut() = None;
            }
        }
    }
    render
}

pub(crate) fn move_index(current: Option<usize>, len: usize, delta: i32) -> Option<usize> {
    if len == 0 {
        return None;
    }
    let current = current.unwrap_or(0) as i32;
    Some((current + delta).clamp(0, (len as i32 - 1).max(0)) as usize)
}

/// A string's terminal-column width (via ratatui/unicode-width), not its
/// character count: the icon glyphs used in ref pills (⎇/☁/🏷) render wider
/// than one column in many terminals and fonts, and char-counting them
/// undercounts a pill's width, letting it overflow into the next column.
pub(crate) fn display_width(s: &str) -> usize {
    Span::raw(s).width()
}

/// `spans` cut to at most `max_width` display columns (whole characters,
/// styles kept), for a row whose tail is about to be covered by a button.
pub(crate) fn clip_spans(spans: Vec<Span<'_>>, max_width: usize) -> Vec<Span<'_>> {
    let mut used = 0usize;
    let mut out = Vec::with_capacity(spans.len());
    for span in spans {
        if used >= max_width {
            break;
        }
        let w = display_width(&span.content);
        if used + w <= max_width {
            used += w;
            out.push(span);
            continue;
        }
        let mut kept = String::new();
        for ch in span.content.chars() {
            let cw = display_width(&ch.to_string());
            if used + cw > max_width {
                break;
            }
            used += cw;
            kept.push(ch);
        }
        out.push(Span::styled(kept, span.style));
        break;
    }
    out
}

pub(crate) use gitpoulpi_app::dates::civil_from_days;

/// The Log's Date column, rendered through `ui.date_time_format`
/// (`live_settings::date_time_pattern`, republished when Preferences
/// changes it, so an edit shows on the next frame).
///
/// A pattern wide enough to overflow the column is truncated by the table,
/// not by this function (`LogColumn::Date`'s preferred width is 17); a clamp
/// here would put two truncation rules in one column.
pub(crate) fn format_timestamp(ts: i64) -> String {
    live_settings::format_date_time(ts)
}

/// The commit-detail Date line, the Date column's hover tooltip and
/// "Copy > Date", rendered through `ui.date_verbose_format`
/// (`live_settings::date_verbose_pattern`).
pub(crate) fn format_timestamp_full(ts: i64) -> String {
    live_settings::format_date(&live_settings::date_verbose_pattern(), ts)
}

/// Truncates `s` to at most `max_width` display **cells** (CJK and
/// emoji-presentation glyphs are 2, combining marks 0), ending in an
/// ellipsis when it does not fit. A wide glyph is never split: when it
/// would straddle the cut it is dropped whole, so the result can be one
/// cell short of `max_width`, never over.
pub(crate) fn truncate_with_ellipsis(s: &str, max_width: usize) -> String {
    if display_width(s) <= max_width {
        return s.to_string();
    }
    if max_width == 0 {
        return String::new();
    }
    let mut out = take_width(s, max_width - 1);
    out.push('\u{2026}');
    out
}

/// The longest prefix of `s` that fits in `max_width` display cells,
/// whole characters only (a zero-width mark stays with its base).
pub(crate) fn take_width(s: &str, max_width: usize) -> String {
    let mut out = String::new();
    let mut used = 0;
    for c in s.chars() {
        let mut buf = [0u8; 4];
        let w = display_width(c.encode_utf8(&mut buf));
        if used + w > max_width {
            break;
        }
        out.push(c);
        used += w;
    }
    out
}

/// Exactly what fits in `width` columns, ellipsised when cut.
pub(crate) fn fit(text: &str, width: usize) -> String {
    truncate_with_ellipsis(text, width)
}

/// Keeps the **end** of an over-long string - right for a path, whose
/// last segments say where you are.
pub(crate) fn tail_fit(text: &str, width: usize) -> String {
    if display_width(text) <= width || width == 0 {
        return text.to_string();
    }
    let mut out = Vec::new();
    let mut used = 0usize;
    for c in text.chars().rev() {
        let w = display_width(&c.to_string());
        if used + w + 1 > width {
            break;
        }
        out.push(c);
        used += w;
    }
    out.reverse();
    format!("\u{2026}{}", out.into_iter().collect::<String>())
}

/// Fits `text` in `width` columns by cutting its **middle** (`/data/Pro...ulpi`):
/// the start and the end of a path both say where it is.
pub(crate) fn fit_middle(text: &str, width: usize) -> String {
    if display_width(text) <= width {
        return text.to_string();
    }
    if width <= 1 {
        return "\u{2026}".repeat(width);
    }
    let room = width - 1;
    let (head_w, tail_w) = (room.div_ceil(2), room / 2);
    let mut head = String::new();
    let mut used = 0usize;
    for c in text.chars() {
        let w = display_width(&c.to_string());
        if used + w > head_w {
            break;
        }
        head.push(c);
        used += w;
    }
    let mut tail: Vec<char> = Vec::new();
    let mut used = 0usize;
    for c in text.chars().rev() {
        let w = display_width(&c.to_string());
        if used + w > tail_w {
            break;
        }
        tail.push(c);
        used += w;
    }
    tail.reverse();
    format!("{head}\u{2026}{}", tail.into_iter().collect::<String>())
}

/// Greedy word wrap, in terminal columns.
pub(crate) fn wrap_words(text: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return vec![text.to_string()];
    }
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        let candidate_width = display_width(&current) + display_width(word) + 1;
        if !current.is_empty() && candidate_width > width {
            lines.push(std::mem::take(&mut current));
        }
        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(word);
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

/// Per-section "does the offset still follow the selection on render"
/// flags (see `list_render_state`), indexed by section. All `true` --
/// ratatui's own behavior - until a mouse-wheel scroll over that section
/// clears one; a keyboard move or click sets it back.
pub(crate) struct FollowFlags<const N: usize>([bool; N]);

impl<const N: usize> Default for FollowFlags<N> {
    fn default() -> Self {
        FollowFlags([true; N])
    }
}

impl<const N: usize> FollowFlags<N> {
    pub(crate) fn get(&self, slot: usize) -> bool {
        self.0[slot]
    }

    pub(crate) fn get_mut(&mut self, slot: usize) -> &mut bool {
        &mut self.0[slot]
    }
}

/// Maps a click's screen row to a list index, given the list's current
/// scroll offset.
///
/// `top_offset` is how many rows sit between `area.y` and the widget's
/// first *content* row: 1 for a bordered and/or titled list, 2 for the Log
/// table (chrome row + column header). It is an explicit parameter because
/// "1" stood for two different things that only accidentally agreed.
///
/// Careful with blocks that have no `Borders::TOP`: the commit-detail
/// pane's changed-files list is `Borders::BOTTOM | LEFT | RIGHT` and shares
/// one frame with the "Commit details" block above it. It is still `1`, not
/// `0`: `Block::inner` reserves the top row when the block has a top border
/// **or a title**, and that block has a title. Reasoning from the border
/// set alone gives 0 and shifts every row by one.
pub(crate) fn row_to_index(
    area: Rect,
    offset: usize,
    top_offset: u16,
    row: u16,
    len: usize,
) -> Option<usize> {
    let top = area.y + top_offset;
    if row < top || row >= area.y + area.height - 1 {
        return None; // border and header rows
    }
    let index = (row - top) as usize + offset;
    (index < len).then_some(index)
}

/// How long a repository is allowed to look like it is merely loading before
/// the Log/right pane/Refs sidebar say it is taking a while.
///
/// Past this, `loading_lines` switches its first line from "Loading
/// repository..." to "Still loading... (large repository?)"; see
/// `where-we-are.md`, "Loading state, not an error".
pub(crate) const LOADING_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(15);

/// The lines the Log, the right pane and the Refs sidebar all show while
/// `still_loading` is true, so the three panes can never disagree about
/// what is happening. One function rather than three copies of the same
/// wording drifting apart.
pub(crate) fn loading_lines(root: &std::path::Path, elapsed: std::time::Duration) -> Vec<String> {
    let path = root.display().to_string();
    if elapsed >= LOADING_TIMEOUT {
        vec![
            gitpoulpi_i18n::msg::pane_notice::STILL_LOADING
                .get()
                .to_string(),
            path,
        ]
    } else {
        // A cheap ellipsis animation: the event loop already redraws on every
        // input poll (at worst every 33ms), so it forces no extra redraw.
        let dots = match (elapsed.as_millis() / 400) % 4 {
            1 => ".",
            2 => "..",
            3 => "...",
            _ => "",
        };
        vec![gitpoulpi_i18n::msg::pane_notice::loading(dots), path]
    }
}

/// What the Log, the right pane and the Refs sidebar say when a daemon link
/// has stopped (see `DaemonLink::failure_rx`): the real reason, and how to
/// get out of it. The key named here is the one the event loop turns into
/// a link restart while a failure is showing.
pub(crate) fn link_failure_lines(reason: &str) -> Vec<String> {
    vec![
        gitpoulpi_i18n::msg::pane_notice::LINK_LOST
            .get()
            .to_string(),
        reason.to_string(),
        gitpoulpi_i18n::msg::pane_notice::RETRY.get().to_string(),
    ]
}

/// The one message the three panes show instead of their content, if any,
/// and whether it is an error.
///
/// A stopped link wins over "loading": a link that died before its first
/// answer leaves `still_loading` true forever, and loading would be all the
/// window could ever say.
pub(crate) fn pane_notice(
    link_failure: Option<&str>,
    still_loading: bool,
    root: &std::path::Path,
    elapsed: std::time::Duration,
) -> Option<(Vec<String>, bool)> {
    match link_failure {
        Some(reason) => Some((link_failure_lines(reason), true)),
        None if still_loading => Some((loading_lines(root, elapsed), false)),
        None => None,
    }
}

/// Draws `lines`, vertically and horizontally centered inside `area`.
///
/// Used for the loading and "log error" states instead of a single narrow
/// table cell, which clips a long message to its first word.
pub(crate) fn render_centered_message(
    frame: &mut Frame,
    area: Rect,
    lines: &[String],
    style: Style,
) {
    if area.width == 0 || area.height == 0 || lines.is_empty() {
        return;
    }
    // Wrapped at word boundaries to the pane's width (a long reason, such as
    // a build mismatch, in the narrow Refs pane otherwise loses its end), and
    // centered as a block by its wrapped height.
    let rendered: Vec<Line> = lines
        .iter()
        .flat_map(|l| crate::commit_detail::wrap_text(l, usize::from(area.width)))
        .map(Line::from)
        .collect();
    let height = (rendered.len() as u16).min(area.height);
    let msg_area = Rect::new(
        area.x,
        area.y + (area.height.saturating_sub(height)) / 2,
        area.width,
        height,
    );
    frame.render_clipped(
        Paragraph::new(rendered)
            .alignment(Alignment::Center)
            .style(style),
        msg_area,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_flyout_hangs_beside_its_parent_row_and_stays_on_screen() {
        let bounds = Rect::new(0, 0, 100, 30);
        let parent = Rect::new(10, 5, 20, 12);
        // Room on the right: it starts at the parent's right edge, on the row.
        assert_eq!(
            flyout_rect(parent, 8, 25, 6, bounds),
            Rect::new(30, 8, 25, 6)
        );
        // No room on the right: it ends at the parent's left edge.
        let right = Rect::new(70, 5, 25, 12);
        assert_eq!(
            flyout_rect(right, 8, 25, 6, bounds),
            Rect::new(45, 8, 25, 6)
        );
        // Taller than the room below the row: lifted to fit.
        assert_eq!(
            flyout_rect(parent, 28, 25, 10, bounds),
            Rect::new(30, 20, 25, 10)
        );
        // Room on neither side: clamped inside the screen, never off it.
        let narrow = Rect::new(0, 0, 60, 30);
        let wide = Rect::new(5, 5, 50, 10);
        let r = flyout_rect(wide, 6, 40, 4, narrow);
        assert!(r.right() <= narrow.right() && r.x >= narrow.x, "{r:?}");
        // Never at the origin for a parent that is not there.
        assert_ne!(flyout_rect(parent, 8, 25, 6, bounds).x, 0);
    }

    #[test]
    fn follow_flags_start_true_and_clear_per_slot() {
        let mut flags = FollowFlags::<3>::default();
        assert!((0..3).all(|i| flags.get(i)));
        *flags.get_mut(1) = false;
        assert_eq!(
            [flags.get(0), flags.get(1), flags.get(2)],
            [true, false, true]
        );
    }

    #[test]
    fn wrap_words_pins_ascii_cjk_and_emoji() {
        assert_eq!(wrap_words("aaa bbb ccc", 7), vec!["aaa bbb", "ccc"]);
        assert_eq!(wrap_words("", 5), Vec::<String>::new());
        assert_eq!(wrap_words("abc  def", 0), vec!["abc  def"]);
        assert_eq!(wrap_words("abcdefgh", 3), vec!["abcdefgh"]);
        assert_eq!(wrap_words("日本 語テ キ", 5), vec!["日本", "語テ", "キ"]);
        assert_eq!(wrap_words("😀 😀 😀", 5), vec!["😀 😀", "😀"]);
    }

    #[test]
    fn fit_pins_ascii_cjk_and_emoji() {
        assert_eq!(fit("ab", 2), "ab");
        assert_eq!(fit("abcdef", 4), "abc\u{2026}");
        assert_eq!(fit("abcdef", 1), "\u{2026}");
        assert_eq!(fit("abcdef", 0), "");
        assert_eq!(fit("日本語テキスト", 7), "日本語\u{2026}");
        assert_eq!(fit("a😀b😀c", 4), "a😀\u{2026}");
    }

    #[test]
    fn tail_fit_keeps_the_end_of_a_path() {
        assert_eq!(tail_fit("C:\\Users\\me\\work", 8), "\u{2026}me\\work");
        assert_eq!(tail_fit("short", 8), "short");
        assert_eq!(tail_fit("abc", 0), "abc");
        assert_eq!(tail_fit("abcdef", 1), "\u{2026}");
        assert_eq!(tail_fit("abcdef", 4), "\u{2026}def");
        assert_eq!(tail_fit("日本語テキスト", 7), "\u{2026}キスト");
        assert_eq!(tail_fit("a😀b😀c", 4), "\u{2026}😀c");
    }
    use crate::test_support::*;

    fn bar_region(len: usize, visible: usize) -> ListBarRegion {
        // Outer rect 20 tall -> an 18-cell track starting at row 6.
        ListBarRegion {
            bar: ListBar::Log,
            area: Rect::new(0, 5, 30, 20),
            len,
            visible,
        }
    }

    #[test]
    fn a_press_on_the_track_jumps_and_a_press_on_the_thumb_only_grabs() {
        let region = bar_region(54, 18);
        let track_x = 29;
        // Row 6 is the up arrow; at the top the thumb starts at row 7; a
        // press far below it jumps
        // the list down, with a centered grab for the drag that follows.
        let press = list_bar_press(region, 0, track_x, 6 + 16).expect("on the track");
        let jump = press.jump_to.expect("off the thumb: a jump");
        assert!(jump > 0, "jumped toward the click, got {jump}");
        // A press on the thumb moves nothing.
        let press = list_bar_press(region, 0, track_x, 7).expect("on the thumb");
        assert_eq!(press.jump_to, None);
        assert_eq!(press.grab, 0);
        // A press deeper inside the thumb remembers where it holds it.
        let thumb_len = scrollbar::Track::vertical(region.area, 54, 0)
            .unwrap()
            .thumb_len;
        assert!(thumb_len >= 2, "thumb_len {thumb_len}");
        let press = list_bar_press(region, 0, track_x, 8).unwrap();
        assert_eq!((press.grab, press.jump_to), (1, None));
        // The arrows step one row and never start a drag.
        let down = list_bar_press(region, 5, track_x, 6 + 17).unwrap();
        assert_eq!((down.jump_to, down.arrow), (Some(6), true));
        let up = list_bar_press(region, 0, track_x, 6).unwrap();
        assert_eq!((up.jump_to, up.arrow), (Some(0), true));
    }

    #[test]
    fn a_press_off_the_bar_or_on_a_list_that_fits_is_not_ours() {
        let region = bar_region(180, 18);
        assert_eq!(list_bar_press(region, 0, 28, 10), None, "content column");
        assert_eq!(list_bar_press(region, 0, 29, 5), None, "top corner glyph");
        assert_eq!(
            list_bar_press(region, 0, 29, 24),
            None,
            "bottom corner glyph"
        );
        let fits = bar_region(10, 18);
        assert_eq!(list_bar_press(fits, 0, 29, 10), None);
    }

    #[test]
    fn dragging_the_thumb_to_either_end_reaches_the_first_and_last_row_and_clamps() {
        let region = bar_region(180, 18);
        let max = 180 - 18;
        assert_eq!(list_bar_drag(region, 0, 0, 6), 0);
        // Far past the bottom of the track: exactly the last window.
        assert_eq!(list_bar_drag(region, 0, 0, 500), max);
        // Above the track clamps to the first row.
        assert_eq!(list_bar_drag(region, max, 0, 0), 0);
    }

    #[test]
    fn dragging_is_monotonic_and_follows_the_pointer() {
        let region = bar_region(500, 17);
        let mut last = 0;
        for row in 6..24 {
            let off = list_bar_drag(region, 0, 0, row);
            assert!(off >= last, "row {row}: {off} < {last}");
            last = off;
        }
        assert!(
            last > 400,
            "the bottom of the track is near the end: {last}"
        );
        // The thumb's own row for an offset drags back to (about) it.
        let track = scrollbar::Track::vertical(region.area, 500, 250).unwrap();
        let row = region.area.y + 1 + track.thumb_start;
        let back = list_bar_drag(region, 250, 0, row);
        assert!(back.abs_diff(250) <= 500 / 16, "round trip {back}");
    }

    #[test]
    fn a_header_row_shortens_the_last_window_not_the_track() {
        // A Log spends a row on its header, so `visible` is one less than
        // the track: the last window must use `visible`.
        let region = bar_region(100, 17);
        assert_eq!(list_bar_drag(region, 0, 0, 100), 100 - 17);
    }

    #[test]
    fn short_oid_is_gits_own_seven_characters() {
        assert_eq!(short_oid(oid(0xab)), "ababababab"[..7].to_string());
        assert_eq!(short_oid(oid(0xab)).len(), 7);
    }

    /// The changed-files list is `Borders::BOTTOM | LEFT | RIGHT` (no top
    /// border) but DOES have a title, and `Block::inner` reserves the top row
    /// for `borders.intersects(TOP) || has_title_at_position(Top)`, so its
    /// content starts at `area.y + 1`. Pins the real rule against reasoning
    /// from the border set alone (which gives 0).
    #[test]
    fn a_titled_block_reserves_its_top_row_even_without_a_top_border() {
        let area = Rect::new(0, 10, 30, 6);
        let block = Block::default()
            .title("Changed files")
            .borders(Borders::BOTTOM | Borders::LEFT | Borders::RIGHT);
        let inner = block.inner(area);
        assert_eq!(
            inner.y,
            area.y + 1,
            "a titled block reserves its top row; row_to_index's offset must match"
        );
        let top_offset = inner.y - area.y;
        assert_eq!(row_to_index(area, 0, top_offset, 10, 5), None); // title row
        assert_eq!(row_to_index(area, 0, top_offset, 11, 5), Some(0));
        assert_eq!(row_to_index(area, 0, top_offset, 12, 5), Some(1));
        // The bottom border is still excluded.
        assert_eq!(row_to_index(area, 0, top_offset, 15, 5), None);
    }

    /// The genuinely-untitled, unbordered case, so the parameter's meaning
    /// stays honest even though no widget uses 0.
    #[test]
    fn a_bare_area_with_no_chrome_resolves_its_very_first_row() {
        let area = Rect::new(0, 10, 30, 6);
        assert_eq!(row_to_index(area, 0, 0, 10, 5), Some(0));
        assert_eq!(row_to_index(area, 0, 0, 11, 5), Some(1));
    }

    #[test]
    fn a_bordered_list_and_the_log_table_still_skip_their_own_chrome() {
        let area = Rect::new(0, 10, 30, 6);
        // Plain bordered list: row 10 is the border, 11 is content 0.
        assert_eq!(row_to_index(area, 0, 1, 10, 5), None);
        assert_eq!(row_to_index(area, 0, 1, 11, 5), Some(0));
        // Log table: border + column header, so content starts at 12.
        assert_eq!(row_to_index(area, 0, 2, 11, 5), None);
        assert_eq!(row_to_index(area, 0, 2, 12, 5), Some(0));
        // The scroll offset still shifts the mapping.
        assert_eq!(row_to_index(area, 7, 2, 12, 20), Some(7));
    }

    #[test]
    fn an_anchored_popup_always_fits_inside_the_frame() {
        let frame = Rect::new(0, 0, 80, 24);
        let cases = [
            ("ordinary", 10, 5, 30, 7),
            ("anchored at the last column", 79, 5, 30, 7),
            ("anchored at the last row", 10, 23, 30, 7),
            ("anchored at the far corner", 79, 23, 30, 7),
            ("menu wider than the frame", 10, 5, 200, 7),
            ("menu taller than the frame", 10, 5, 30, 100),
            ("menu bigger than the frame", 10, 5, 200, 100),
            ("zero-sized frame request", 0, 0, 0, 0),
        ];
        for (name, col, row, w, h) in cases {
            let popup = anchored_popup(frame, col, row, w, h);
            assert!(
                popup.x >= frame.x
                    && popup.y >= frame.y
                    && popup.right() <= frame.right()
                    && popup.bottom() <= frame.bottom(),
                "{name}: {popup:?} escapes {frame:?}"
            );
        }
    }

    /// The degenerate frames a terminal really does hand you mid-resize,
    /// where every `saturating_sub` bottoms out at once.
    #[test]
    fn an_anchored_popup_survives_a_frame_with_no_room_at_all() {
        for frame in [
            Rect::new(0, 0, 1, 1),
            Rect::new(0, 0, 0, 0),
            Rect::new(0, 0, 80, 1),
            Rect::new(0, 0, 1, 24),
        ] {
            let popup = anchored_popup(frame, 40, 12, 30, 7);
            assert!(
                popup.right() <= frame.right() && popup.bottom() <= frame.bottom(),
                "{popup:?} escapes {frame:?}"
            );
        }
    }

    /// A popup that fits is still placed *below* its anchor, so it never
    /// covers the row that was right-clicked. Without this, "clamp harder"
    /// would pass the test above by pinning everything to the origin.
    #[test]
    fn an_anchored_popup_that_fits_hangs_just_below_its_anchor() {
        let frame = Rect::new(0, 0, 80, 24);
        let popup = anchored_popup(frame, 10, 5, 30, 7);
        assert_eq!((popup.x, popup.y), (10, 6));
        assert_eq!((popup.width, popup.height), (30, 7));
    }

    /// No room below the anchor (a bottom toolbar): the popup ends where its
    /// anchor row starts, so it covers neither the row nor what is under it.
    #[test]
    fn an_anchored_popup_without_room_below_flips_above_its_anchor() {
        let frame = Rect::new(0, 0, 80, 24);
        let popup = anchored_popup(frame, 10, 22, 30, 7);
        assert_eq!((popup.x, popup.y), (10, 15));
        assert_eq!(popup.bottom(), 22, "ends on the anchor row's top edge");
        // Room below still wins.
        assert_eq!(anchored_popup(frame, 10, 10, 30, 7).y, 11);
        // Room nowhere: clamped inside the frame like before.
        let short = Rect::new(0, 0, 80, 8);
        let popup = anchored_popup(short, 10, 5, 30, 7);
        assert!(popup.bottom() <= short.bottom());
    }

    #[test]
    fn truncate_with_ellipsis_leaves_short_text_alone() {
        assert_eq!(truncate_with_ellipsis("main", 10), "main");
        assert_eq!(truncate_with_ellipsis("main", 4), "main");
    }

    #[test]
    fn truncate_with_ellipsis_cuts_long_text_with_an_indicator() {
        assert_eq!(
            truncate_with_ellipsis("bugfix/math-overflow", 12),
            "bugfix/math\u{2026}"
        );
        assert_eq!(
            truncate_with_ellipsis("bugfix/math-overflow", 1),
            "\u{2026}"
        );
        assert_eq!(truncate_with_ellipsis("bugfix/math-overflow", 0), "");
    }

    #[test]
    fn truncate_with_ellipsis_counts_cells_not_chars() {
        // CJK: 2 cells each. 4 cells for text + the ellipsis = 5.
        assert_eq!(truncate_with_ellipsis("日本語です", 6), "日本\u{2026}");
        assert_eq!(truncate_with_ellipsis("日本語です", 5), "日本\u{2026}");
        // A wide glyph that would straddle the cut is dropped whole.
        assert_eq!(truncate_with_ellipsis("日本語です", 4), "日\u{2026}");
        assert_eq!(truncate_with_ellipsis("日本語です", 2), "\u{2026}");
        assert_eq!(truncate_with_ellipsis("日本語です", 1), "\u{2026}");
        assert_eq!(truncate_with_ellipsis("日本語です", 0), "");
        // Exact fit is left alone, one cell short is cut.
        assert_eq!(truncate_with_ellipsis("日本", 4), "日本");
        assert_eq!(truncate_with_ellipsis("日本", 3), "日\u{2026}");
        assert_eq!(truncate_with_ellipsis("abcde", 5), "abcde");
    }

    #[test]
    fn truncate_with_ellipsis_handles_emoji_and_combining_marks() {
        // Default-emoji-presentation glyphs are 2 cells.
        assert_eq!(
            truncate_with_ellipsis("\u{1F680}\u{1F680}\u{1F680}", 5),
            "\u{1F680}\u{1F680}\u{2026}"
        );
        assert_eq!(
            truncate_with_ellipsis("\u{1F680}\u{1F680}\u{1F680}", 4),
            "\u{1F680}\u{2026}"
        );
        // A combining mark is zero-width and stays with its base.
        let e = "e\u{301}";
        assert_eq!(
            truncate_with_ellipsis(&format!("{e}{e}{e}{e}{e}"), 4),
            format!("{e}{e}{e}\u{2026}")
        );
        assert_eq!(
            truncate_with_ellipsis(&format!("{e}{e}{e}"), 3),
            format!("{e}{e}{e}")
        );
    }

    #[test]
    fn truncate_with_ellipsis_never_exceeds_the_width() {
        for text in [
            "日本語のテキスト",
            "a日b本c",
            "\u{1F680}x\u{1F680}y",
            "plain ascii text",
        ] {
            for w in 0..=20 {
                let out = truncate_with_ellipsis(text, w);
                assert!(display_width(&out) <= w, "{text:?} at {w}: {out:?}");
            }
        }
    }

    /// Guards the stale-index hazard: dropping a stash shortens the
    /// list under a selection pointing past its new end.
    #[test]
    fn selection_is_clamped_when_a_section_shrinks() {
        let mut state = ListState::default();
        state.select(Some(4));
        move_selection(&mut state, 2, 0);
        assert_eq!(state.selected(), Some(1));
        // Emptied entirely -> no phantom selection left behind.
        move_selection(&mut state, 0, 0);
        assert_eq!(state.selected(), None);
    }

    /// The loading state, not `Err`: a fresh window shows the repository
    /// path and never the word "backend".
    #[test]
    fn loading_lines_names_the_repository_and_never_says_backend() {
        let root = std::path::Path::new("C:/src/notes");
        let lines = loading_lines(root, std::time::Duration::from_secs(0));
        assert!(lines[0].starts_with("Loading repository"), "{lines:?}");
        assert!(lines.iter().any(|l| l.contains("notes")), "{lines:?}");
        assert!(
            !lines.iter().any(|l| l.contains("backend")),
            "the loading message must never mention the old placeholder: {lines:?}"
        );
    }

    /// Past `LOADING_TIMEOUT` the message says the wait is unusual instead of
    /// animating forever.
    #[test]
    fn loading_lines_says_so_once_it_has_taken_a_while() {
        let root = std::path::Path::new("/tmp/huge-repo");
        let lines = loading_lines(root, LOADING_TIMEOUT);
        assert!(
            lines[0].contains("Still loading"),
            "expected the timeout wording, got {lines:?}"
        );
        let before = loading_lines(root, LOADING_TIMEOUT - std::time::Duration::from_millis(1));
        assert!(
            !before[0].contains("Still loading"),
            "the timeout wording fired one tick early: {before:?}"
        );
    }

    /// The ellipsis is a pure function of elapsed time, so it cycles
    /// deterministically and never grows without bound.
    #[test]
    fn loading_lines_ellipsis_cycles_without_growing_forever() {
        let root = std::path::Path::new("/r");
        let at = |ms: u64| loading_lines(root, std::time::Duration::from_millis(ms))[0].clone();
        assert_eq!(at(0), "Loading repository");
        assert_eq!(at(400), "Loading repository.");
        assert_eq!(at(800), "Loading repository..");
        assert_eq!(at(1200), "Loading repository...");
        // Cycles back rather than growing a fifth dot.
        assert_eq!(at(1600), "Loading repository");
    }

    /// A link that stopped before its first answer must be shown as the
    /// failure it is, with its reason and a way out, even though the window
    /// is still "loading". Red if `pane_notice` checks `still_loading` before
    /// `link_failure`: the failure is masked forever.
    #[test]
    fn a_stopped_link_is_shown_as_an_error_not_as_loading() {
        let root = std::path::Path::new("/repo");
        let (lines, is_error) = pane_notice(
            Some("the session daemon closed the connection"),
            true,
            root,
            LOADING_TIMEOUT * 4,
        )
        .expect("a stopped link must say something");
        assert!(is_error);
        let text = lines.join("\n");
        assert!(!text.contains("Loading"), "{text}");
        assert!(
            text.contains("the session daemon closed the connection"),
            "{text}"
        );
        assert!(text.contains("retry"), "{text}");

        let (lines, is_error) = pane_notice(None, true, root, std::time::Duration::ZERO).unwrap();
        assert!(!is_error);
        assert!(lines[0].starts_with("Loading repository"));
        assert!(pane_notice(None, false, root, std::time::Duration::ZERO).is_none());
    }

    /// `render_centered_message` centers: a one-line message in a tall, wide
    /// area lands away from every edge. Red if it renders at `area.y`/`area.x`.
    #[test]
    fn render_centered_message_lands_away_from_every_edge() {
        use ratatui::backend::TestBackend;

        let mut terminal = Terminal::new(TestBackend::new(40, 11)).unwrap();
        terminal
            .draw(|frame| {
                render_centered_message(
                    frame,
                    Rect::new(0, 0, 40, 11),
                    &["Loading repository".to_string()],
                    Style::default(),
                )
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        // Nothing on the very first or very last row: a corner-pinned render
        // would fail this.
        let top: String = (0..40).map(|x| buffer[(x, 0)].symbol()).collect();
        let bottom: String = (0..40).map(|x| buffer[(x, 10)].symbol()).collect();
        assert!(top.trim().is_empty(), "top row not empty: {top:?}");
        assert!(bottom.trim().is_empty(), "bottom row not empty: {bottom:?}");
        // The message is on the middle row (5 of 0..=10), and it is not
        // flush against the left edge either.
        let middle: String = (0..40).map(|x| buffer[(x, 5)].symbol()).collect();
        assert!(middle.contains("Loading repository"), "{middle:?}");
        assert!(
            middle.starts_with(' '),
            "text is flush against the left edge, not centered: {middle:?}"
        );
    }

    /// A genuine log error is shown too, distinguishably from loading and
    /// never truncated to a single word.
    #[test]
    fn render_centered_message_shows_the_whole_error_text() {
        use ratatui::backend::TestBackend;

        let mut terminal = Terminal::new(TestBackend::new(60, 5)).unwrap();
        let message = "log error: backend error: the disk is full".to_string();
        terminal
            .draw(|frame| {
                render_centered_message(
                    frame,
                    Rect::new(0, 0, 60, 5),
                    std::slice::from_ref(&message),
                    Style::default(),
                )
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let row: String = (0..60).map(|x| buffer[(x, 2)].symbol()).collect();
        assert!(
            row.contains("log error: backend error: the disk is full"),
            "the error text was clipped: {row:?}"
        );
    }

    /// A long reason in a narrow pane (the Refs sidebar) wraps at words
    /// instead of losing its end. Red without the `wrap_text` in
    /// `render_centered_message`.
    #[test]
    fn render_centered_message_wraps_a_long_line_in_a_narrow_pane() {
        use ratatui::backend::TestBackend;

        let mut terminal = Terminal::new(TestBackend::new(20, 12)).unwrap();
        let message = "the session daemon is running a different build - retry later".to_string();
        terminal
            .draw(|frame| {
                render_centered_message(
                    frame,
                    Rect::new(0, 0, 20, 12),
                    std::slice::from_ref(&message),
                    Style::default(),
                )
            })
            .unwrap();
        let buffer = terminal.backend().buffer();
        let text: String = (0..12)
            .flat_map(|y| (0..20).map(move |x| (x, y)))
            .map(|(x, y)| buffer[(x, y)].symbol().to_string())
            .collect();
        for word in message.split_whitespace() {
            assert!(text.contains(word), "{word:?} was clipped: {text:?}");
        }
    }

    /// A wide per-column Log tooltip is bounded to `log_area`, not the whole
    /// frame (`anchored_popup(frame.area(), ...)`): anchored to the frame, a
    /// long one opened near the Log pane's right edge was clamped leftward
    /// past the pane's border into the Refs sidebar, an area this popup's
    /// open/close accounting never touches, leaving stale cells.
    ///
    /// Simulates a bordered Refs sidebar (with a "Worktrees" section row) left
    /// of a bordered Log pane and a tooltip at the Log pane's right edge wide
    /// enough to spill over the sidebar, then closed. The sidebar's cells must
    /// read identically before the tooltip opens, while it is open, and after
    /// it closes.
    #[test]
    fn a_closed_log_tooltip_leaves_no_stale_cells_over_the_sidebar() {
        use crate::wide_edge::Clear;
        use ratatui::backend::TestBackend;
        use ratatui::widgets::BorderType;
        use ratatui::Terminal;

        let sidebar_area = Rect::new(0, 0, 20, 12);
        let log_area = Rect::new(20, 0, 40, 12);

        let draw_background = |frame: &mut Frame| {
            frame.render_widget(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title("Refs"),
                sidebar_area,
            );
            // The "Worktrees" section row: a plain text row, styled like a
            // real sidebar row.
            frame.render_widget(
                Paragraph::new("Worktrees"),
                Rect::new(sidebar_area.x + 1, sidebar_area.y + 1, 9, 1),
            );
            frame.render_widget(
                Paragraph::new("  main"),
                Rect::new(sidebar_area.x + 1, sidebar_area.y + 2, 9, 1),
            );
            frame.render_widget(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title("Log"),
                log_area,
            );
        };

        let draw = |show_tooltip: bool, terminal: &mut Terminal<TestBackend>| {
            terminal
                .draw(|frame| {
                    draw_background(frame);
                    if show_tooltip {
                        // A long commit message, anchored near the Log
                        // pane's own right edge.
                        let lines = vec![Line::from(
                            "a commit message long enough to force a wide tooltip",
                        )];
                        let width = lines[0].width() as u16 + 2;
                        let height = 3u16;
                        let anchor_col = log_area.x + log_area.width - 2;
                        let anchor_row = log_area.y + 3;
                        let popup = anchored_popup(log_area, anchor_col, anchor_row, width, height);
                        // The popup never leaves the pane it belongs to.
                        assert!(
                            log_area.union(popup) == log_area,
                            "the tooltip {popup:?} escaped its own pane {log_area:?}"
                        );
                        frame.render_clipped(Clear, popup);
                        frame.render_clipped(
                            Paragraph::new(lines).block(
                                Block::default()
                                    .borders(Borders::ALL)
                                    .style(Style::default()),
                            ),
                            popup,
                        );
                    }
                })
                .unwrap();
        };

        let mut before = Terminal::new(TestBackend::new(60, 12)).unwrap();
        draw(false, &mut before);

        let mut live = Terminal::new(TestBackend::new(60, 12)).unwrap();
        draw(false, &mut live);
        // Sidebar reads the same before the tooltip has ever opened...
        for y in sidebar_area.top()..sidebar_area.bottom() {
            for x in sidebar_area.left()..sidebar_area.right() {
                assert_eq!(
                    before.backend().buffer()[(x, y)],
                    live.backend().buffer()[(x, y)]
                );
            }
        }
        draw(true, &mut live); // open
                               // ...and while it's open, since it can no longer reach the sidebar.
        for y in sidebar_area.top()..sidebar_area.bottom() {
            for x in sidebar_area.left()..sidebar_area.right() {
                assert_eq!(
                    before.backend().buffer()[(x, y)],
                    live.backend().buffer()[(x, y)],
                    "sidebar cell ({x}, {y}) changed while the Log tooltip was open"
                );
            }
        }
        draw(false, &mut live); // close
                                // ...and, the bug this falsifies, after it closes: every cell the
                                // sidebar owns must be back to exactly what a run that never
                                // showed a tooltip would have drawn -- no blank/stale leftover.
        let mut expected = Terminal::new(TestBackend::new(60, 12)).unwrap();
        draw(false, &mut expected);
        assert_eq!(
            live.backend().buffer(),
            expected.backend().buffer(),
            "cells under the closed tooltip were not fully repainted"
        );
    }

    // The mouse wheel scrolls a list's viewport, never its selection:
    // `scroll_offset` is the shared primitive every wheel handler goes through.

    #[test]
    fn scroll_offset_moves_by_delta_and_clamps_to_the_scrollable_range() {
        // 10 rows, 4 visible: the offset can run from 0 to 6 (so the
        // last row, index 9, is still on screen at offset 6).
        assert_eq!(scroll_offset(0, 10, 4, 2), 2);
        assert_eq!(scroll_offset(2, 10, 4, 2), 4);
        // Past the bottom: clamped, not left free to scroll the list
        // entirely past its own content.
        assert_eq!(scroll_offset(5, 10, 4, 10), 6);
        // Past the top: clamped at 0, never negative.
        assert_eq!(scroll_offset(1, 10, 4, -5), 0);
    }

    #[test]
    fn scroll_offset_is_a_no_op_when_everything_already_fits() {
        assert_eq!(scroll_offset(0, 3, 10, 5), 0);
        assert_eq!(scroll_offset(0, 0, 10, 5), 0);
    }

    #[test]
    fn list_render_state_passes_through_unchanged_while_following() {
        let mut state = ListState::default();
        state.select(Some(7));
        *state.offset_mut() = 7; // deliberately NOT containing the selection
        let render = list_render_state(&state, true, 3);
        // `follow == true` is ratatui's own default behavior: never
        // touched here, so `Table`/`List::render` still snaps it.
        assert_eq!(render.selected(), Some(7));
        assert_eq!(render.offset(), 7);
    }

    #[test]
    fn list_render_state_hides_the_selection_only_when_the_wheel_scrolled_it_out_of_view() {
        // Offset deliberately non-zero: `ListState::select(None)` resets
        // `offset` to 0 as a side effect, and an offset of 0 would hide that
        // bug by coincidence.
        let mut state = ListState::default();
        state.select(Some(7));
        *state.offset_mut() = 2; // the wheel scrolled up; row 7 is off screen
        let render = list_render_state(&state, false, 3);
        assert_eq!(
            render.selected(),
            None,
            "row 7 isn't in [2, 5) - hidden so ratatui can't drag the offset back to it"
        );
        assert_eq!(
            render.offset(),
            2,
            "the frozen offset itself must survive hiding the selection"
        );

        // Scroll back until row 7 is inside the window again: it must
        // reappear, still highlighted, with the offset still exactly
        // where the wheel put it -- not snapped to put 7 at the top.
        *state.offset_mut() = 5;
        let render = list_render_state(&state, false, 3);
        assert_eq!(render.selected(), Some(7));
        assert_eq!(render.offset(), 5);
    }

    #[test]
    fn list_render_state_never_mutates_the_persisted_state() {
        let mut state = ListState::default();
        state.select(Some(7));
        *state.offset_mut() = 0;
        let _ = list_render_state(&state, false, 3);
        assert_eq!(state.selected(), Some(7), "the real selection is untouched");
        assert_eq!(state.offset(), 0, "the real offset is untouched");
    }

    #[test]
    fn table_render_state_matches_list_render_state() {
        let mut state = TableState::default();
        state.select(Some(4));
        *state.offset_mut() = 9; // non-zero, so a select(None)-reset-to-0 bug can't hide
        let render = table_render_state(&state, false, 2);
        assert_eq!(render.selected(), None);
        assert_eq!(
            render.offset(),
            9,
            "hiding the selection must not move the offset"
        );
        *state.offset_mut() = 4;
        assert_eq!(table_render_state(&state, false, 2).selected(), Some(4));
        assert_eq!(table_render_state(&state, true, 2).selected(), Some(4));
    }

    #[test]
    fn clip_spans_cuts_whole_characters_and_keeps_styles() {
        let red = Style::default().fg(ratatui::style::Color::Red);
        let spans = vec![Span::raw("ab"), Span::styled("cdef", red)];
        let clipped = clip_spans(spans.clone(), 4);
        let text: String = clipped.iter().map(|s| s.content.as_ref()).collect();
        assert_eq!(text, "abcd");
        assert_eq!(clipped[1].style, red);
        assert!(clip_spans(spans.clone(), 0).is_empty());
        let all: String = clip_spans(spans, 99)
            .iter()
            .map(|s| s.content.as_ref())
            .collect();
        assert_eq!(all, "abcdef");
    }
}
