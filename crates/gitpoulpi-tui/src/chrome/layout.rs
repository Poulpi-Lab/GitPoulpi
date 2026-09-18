//! Where the chrome's bands go: the repository tab bar, the toolbar (a row
//! above or below the panes, or a column beside them) and the status line
//! (the first or the last row). One pure function, so the frame, the tests
//! and the hit-test arithmetic read the same rectangles.
//!
//! The tab bar (`ui.tabs_position`) is a row under the window's title border
//! (or under the status line when that is on top), a row at the very bottom,
//! or a column at the left edge. It is the outermost band on its edge, so a
//! column toolbar and a column of tabs read tabs, toolbar, panes.

use ratatui::layout::{Constraint, Layout, Rect};

use gitpoulpi_config::{StatusbarPosition, TabsPosition, ToolbarAlign, ToolbarPosition, UiConfig};

/// A window narrower than this draws a `left` / `right` toolbar as a row at
/// the bottom: a column plus three panes does not fit in less.
pub(crate) const VERTICAL_MIN_WIDTH: u16 = 80;

/// Width of the tab column (`ui.tabs_position = "left"`), in cells: room for
/// `⌂`, a name of about sixteen characters and the close cross.
pub(crate) const TABS_COLUMN_WIDTH: u16 = 24;

/// The widest a toolbar column gets, in cells.
pub(crate) const COLUMN_MAX_WIDTH: u16 = 26;

/// The layout options, as `ui.toolbar_*` / `ui.statusbar_position` hold them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ChromeLayout {
    pub(crate) toolbar: ToolbarPosition,
    pub(crate) align: ToolbarAlign,
    /// The pull request button sits at the far end of the toolbar instead of
    /// flowing with the other buttons.
    pub(crate) pr_pinned: bool,
    pub(crate) status: StatusbarPosition,
    pub(crate) tabs: TabsPosition,
}

impl Default for ChromeLayout {
    fn default() -> Self {
        Self::from_ui(&UiConfig::default())
    }
}

impl ChromeLayout {
    pub(crate) fn from_ui(ui: &UiConfig) -> Self {
        Self {
            toolbar: ui.toolbar_position,
            align: ui.toolbar_align,
            pr_pinned: ui.toolbar_pr_pinned,
            status: ui.statusbar_position,
            tabs: ui.tabs_position,
        }
    }

    /// What is actually drawn in a window `width` cells wide. `home` is the
    /// Repository Management tab, which paints its own status bar at the
    /// bottom.
    pub(crate) fn effective(self, width: u16, home: bool) -> Self {
        let mut out = self;
        if out.tabs == TabsPosition::Left && width < VERTICAL_MIN_WIDTH {
            out.tabs = TabsPosition::Top;
        }
        let room = width.saturating_sub(out.tabs_column_width());
        if out.toolbar.is_vertical() && room < VERTICAL_MIN_WIDTH {
            out.toolbar = ToolbarPosition::Bottom;
        }
        if home {
            out.status = StatusbarPosition::Bottom;
        }
        out
    }

    pub(crate) fn is_vertical(self) -> bool {
        self.toolbar.is_vertical()
    }

    /// The width the tab column takes (0 unless the tabs are on the left).
    pub(crate) fn tabs_column_width(self) -> u16 {
        if self.tabs == TabsPosition::Left {
            TABS_COLUMN_WIDTH
        } else {
            0
        }
    }
}

/// The bands of one frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Regions {
    pub(crate) tab_bar: Rect,
    pub(crate) toolbar: Rect,
    pub(crate) panes: Rect,
    pub(crate) footer: Rect,
}

/// Splits `inner` (the window inside its border). `footer_rows` is what the
/// status line needs (more than one while a long key hint wraps);
/// `column_width` is what a vertical toolbar needs. `layout` must already be
/// [`ChromeLayout::effective`].
pub(crate) fn split(
    inner: Rect,
    footer_rows: u16,
    column_width: u16,
    layout: ChromeLayout,
) -> Regions {
    let footer_rows = footer_rows.max(1);
    let status_top = layout.status == StatusbarPosition::Top;
    let row_top = layout.toolbar == ToolbarPosition::Top;
    let row_bottom = layout.toolbar == ToolbarPosition::Bottom;
    let tabs_top = layout.tabs == TabsPosition::Top;
    let tabs_bottom = layout.tabs == TabsPosition::Bottom;

    // The tab column is carved first: it spans the window's whole height.
    let (tab_col, inner) = if layout.tabs == TabsPosition::Left {
        let w = TABS_COLUMN_WIDTH.min(inner.width);
        (
            Some(Rect::new(inner.x, inner.y, w, inner.height)),
            Rect::new(inner.x + w, inner.y, inner.width - w, inner.height),
        )
    } else {
        (None, inner)
    };

    let mut constraints = Vec::with_capacity(6);
    if status_top {
        constraints.push(Constraint::Length(footer_rows));
    }
    if tabs_top {
        constraints.push(Constraint::Length(1));
    }
    if row_top {
        constraints.push(Constraint::Length(1));
    }
    constraints.push(Constraint::Min(0));
    if row_bottom {
        constraints.push(Constraint::Length(1));
    }
    if !status_top {
        constraints.push(Constraint::Length(footer_rows));
    }
    if tabs_bottom {
        constraints.push(Constraint::Length(1));
    }
    let areas = Layout::vertical(constraints).split(inner);
    let mut it = areas.iter().copied();
    let mut next = || it.next().unwrap_or_default();

    let footer_first = status_top.then(&mut next);
    let tab_top = tabs_top.then(&mut next);
    let toolbar_top = row_top.then(&mut next);
    let body = next();
    let toolbar_bottom = row_bottom.then(&mut next);
    let footer_last = (!status_top).then(&mut next);
    let tab_bottom = tabs_bottom.then(&mut next);
    let tab_bar = tab_top.or(tab_bottom).or(tab_col).unwrap_or_default();

    let (toolbar, panes) = match layout.toolbar {
        ToolbarPosition::Top => (toolbar_top.unwrap_or_default(), body),
        ToolbarPosition::Bottom => (toolbar_bottom.unwrap_or_default(), body),
        ToolbarPosition::Left => {
            let w = column_width.min(body.width);
            (
                Rect::new(body.x, body.y, w, body.height),
                Rect::new(body.x + w, body.y, body.width - w, body.height),
            )
        }
        ToolbarPosition::Right => {
            let w = column_width.min(body.width);
            (
                Rect::new(body.right() - w, body.y, w, body.height),
                Rect::new(body.x, body.y, body.width - w, body.height),
            )
        }
    };
    Regions {
        tab_bar,
        toolbar,
        panes,
        footer: footer_first.or(footer_last).unwrap_or_default(),
    }
}

/// What the Home (Repository Management) view covers: everything the tab
/// bar leaves, status line row included (Home paints its own).
pub(crate) fn home_body(inner: Rect, tab_bar: Rect, tabs: TabsPosition) -> Rect {
    match tabs {
        TabsPosition::Top => {
            let top = tab_bar.y + 1;
            Rect::new(
                inner.x,
                top,
                inner.width,
                inner.bottom().saturating_sub(top),
            )
        }
        TabsPosition::Bottom => Rect::new(
            inner.x,
            inner.y,
            inner.width,
            tab_bar.y.saturating_sub(inner.y),
        ),
        TabsPosition::Left => Rect::new(
            tab_bar.right(),
            inner.y,
            inner.right().saturating_sub(tab_bar.right()),
            inner.height,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn layout(toolbar: ToolbarPosition, status: StatusbarPosition) -> ChromeLayout {
        ChromeLayout {
            toolbar,
            align: ToolbarAlign::Start,
            pr_pinned: true,
            status,
            tabs: TabsPosition::Top,
        }
    }

    const INNER: Rect = Rect {
        x: 1,
        y: 1,
        width: 100,
        height: 30,
    };

    fn covers(r: &Regions, inner: Rect) {
        // Every cell of `inner` belongs to exactly one band.
        let bands = [r.tab_bar, r.toolbar, r.panes, r.footer];
        for y in inner.y..inner.bottom() {
            for x in inner.x..inner.right() {
                let n = bands
                    .iter()
                    .filter(|b| x >= b.x && x < b.right() && y >= b.y && y < b.bottom())
                    .count();
                assert_eq!(n, 1, "({x},{y}) in {n} bands: {r:?}");
            }
        }
    }

    #[test]
    fn the_default_is_a_toolbar_row_and_the_status_line_at_the_bottom() {
        let l = ChromeLayout::default();
        assert_eq!(l.toolbar, ToolbarPosition::Bottom);
        assert_eq!(l.status, StatusbarPosition::Bottom);
        let r = split(INNER, 1, 12, l);
        assert_eq!(r.tab_bar, Rect::new(1, 1, 100, 1));
        assert_eq!(r.panes, Rect::new(1, 2, 100, 27));
        assert_eq!(r.toolbar, Rect::new(1, 29, 100, 1));
        assert_eq!(r.footer, Rect::new(1, 30, 100, 1));
        covers(&r, INNER);
    }

    #[test]
    fn a_top_toolbar_sits_between_the_tabs_and_the_panes() {
        let r = split(
            INNER,
            1,
            12,
            layout(ToolbarPosition::Top, StatusbarPosition::Bottom),
        );
        assert_eq!(r.tab_bar.y, 1);
        assert_eq!(r.toolbar, Rect::new(1, 2, 100, 1));
        assert_eq!(r.panes, Rect::new(1, 3, 100, 27));
        assert_eq!(r.footer, Rect::new(1, 30, 100, 1));
        covers(&r, INNER);
    }

    #[test]
    fn a_column_takes_its_width_from_the_panes_on_the_chosen_side() {
        let left = split(
            INNER,
            1,
            12,
            layout(ToolbarPosition::Left, StatusbarPosition::Bottom),
        );
        assert_eq!(left.toolbar, Rect::new(1, 2, 12, 28));
        assert_eq!(left.panes, Rect::new(13, 2, 88, 28));
        covers(&left, INNER);
        let right = split(
            INNER,
            1,
            12,
            layout(ToolbarPosition::Right, StatusbarPosition::Bottom),
        );
        assert_eq!(right.panes, Rect::new(1, 2, 88, 28));
        assert_eq!(right.toolbar, Rect::new(89, 2, 12, 28));
        covers(&right, INNER);
    }

    fn with_tabs(toolbar: ToolbarPosition, tabs: TabsPosition) -> ChromeLayout {
        ChromeLayout {
            tabs,
            ..layout(toolbar, StatusbarPosition::Bottom)
        }
    }

    #[test]
    fn bottom_tabs_are_the_last_row_below_the_status_line() {
        let r = split(
            INNER,
            1,
            12,
            with_tabs(ToolbarPosition::Bottom, TabsPosition::Bottom),
        );
        assert_eq!(r.tab_bar, Rect::new(1, 30, 100, 1));
        assert_eq!(r.footer, Rect::new(1, 29, 100, 1));
        assert_eq!(r.toolbar, Rect::new(1, 28, 100, 1));
        assert_eq!(r.panes, Rect::new(1, 1, 100, 27));
        covers(&r, INNER);
    }

    #[test]
    fn left_tabs_are_a_full_height_column_outside_a_left_toolbar() {
        let r = split(
            INNER,
            1,
            12,
            with_tabs(ToolbarPosition::Left, TabsPosition::Left),
        );
        assert_eq!(r.tab_bar, Rect::new(1, 1, TABS_COLUMN_WIDTH, 30));
        assert_eq!(r.toolbar.x, 1 + TABS_COLUMN_WIDTH);
        assert_eq!(r.panes.x, 1 + TABS_COLUMN_WIDTH + 12);
        assert_eq!(r.footer.x, 1 + TABS_COLUMN_WIDTH);
        assert_eq!(r.footer.right(), INNER.right());
        covers(&r, INNER);
    }

    #[test]
    fn every_tabs_toolbar_status_combination_tiles_the_window() {
        for tabs in TabsPosition::ALL {
            for toolbar in ToolbarPosition::ALL {
                for status in StatusbarPosition::ALL {
                    for rows in [1, 3] {
                        let mut l = layout(*toolbar, *status);
                        l.tabs = *tabs;
                        let r = split(INNER, rows, 14, l);
                        covers(&r, INNER);
                        assert!(r.panes.height > 0 && r.panes.width > 0, "{l:?}");
                    }
                }
            }
        }
    }

    #[test]
    fn left_tabs_need_eighty_columns_and_a_side_toolbar_needs_eighty_beside_them() {
        let mut l = with_tabs(ToolbarPosition::Top, TabsPosition::Left);
        assert_eq!(l.effective(79, false).tabs, TabsPosition::Top);
        assert_eq!(l.effective(80, false).tabs, TabsPosition::Left);
        l.toolbar = ToolbarPosition::Right;
        let narrow = VERTICAL_MIN_WIDTH + TABS_COLUMN_WIDTH - 1;
        assert_eq!(l.effective(narrow, false).toolbar, ToolbarPosition::Bottom);
        assert_eq!(l.effective(narrow, false).tabs, TabsPosition::Left);
        assert_eq!(
            l.effective(narrow + 1, false).toolbar,
            ToolbarPosition::Right
        );
    }

    #[test]
    fn the_home_view_covers_what_the_tab_bar_leaves() {
        for tabs in TabsPosition::ALL {
            let mut l = layout(ToolbarPosition::Bottom, StatusbarPosition::Bottom);
            l.tabs = *tabs;
            let r = split(INNER, 1, 12, l);
            let body = home_body(INNER, r.tab_bar, *tabs);
            for y in INNER.y..INNER.bottom() {
                for x in INNER.x..INNER.right() {
                    let on_bar = r.tab_bar.contains((x, y).into());
                    let in_body = body.contains((x, y).into());
                    assert_ne!(on_bar, in_body, "{tabs:?} ({x},{y})");
                }
            }
        }
    }

    #[test]
    fn the_status_line_on_top_takes_the_first_row_and_may_wrap() {
        for rows in [1, 3] {
            for toolbar in ToolbarPosition::ALL {
                let r = split(INNER, rows, 10, layout(*toolbar, StatusbarPosition::Top));
                assert_eq!(r.footer, Rect::new(1, 1, 100, rows), "{toolbar:?}");
                assert_eq!(r.tab_bar.y, 1 + rows, "{toolbar:?}");
                covers(&r, INNER);
            }
        }
    }

    #[test]
    fn every_combination_tiles_the_window_without_overlap() {
        for toolbar in ToolbarPosition::ALL {
            for status in StatusbarPosition::ALL {
                for rows in [1, 2] {
                    let r = split(INNER, rows, 14, layout(*toolbar, *status));
                    covers(&r, INNER);
                    assert!(r.panes.height > 0 && r.panes.width > 0);
                }
            }
        }
    }

    #[test]
    fn a_narrow_window_draws_a_column_as_a_row_and_home_keeps_the_bottom_status() {
        let mut l = layout(ToolbarPosition::Left, StatusbarPosition::Top);
        assert_eq!(l.effective(79, false).toolbar, ToolbarPosition::Bottom);
        assert_eq!(l.effective(80, false).toolbar, ToolbarPosition::Left);
        assert_eq!(l.effective(200, false).status, StatusbarPosition::Top);
        assert_eq!(l.effective(200, true).status, StatusbarPosition::Bottom);
        l.toolbar = ToolbarPosition::Top;
        assert_eq!(l.effective(20, false).toolbar, ToolbarPosition::Top);
    }

    #[test]
    fn a_window_too_small_for_everything_still_yields_valid_rects() {
        for h in 0..6 {
            for w in 0..6 {
                for toolbar in ToolbarPosition::ALL {
                    let inner = Rect::new(0, 0, w, h);
                    let r = split(inner, 1, 12, layout(*toolbar, StatusbarPosition::Top));
                    for b in [r.tab_bar, r.toolbar, r.panes, r.footer] {
                        assert!(b.right() <= inner.right() && b.bottom() <= inner.bottom());
                    }
                }
            }
        }
    }
}
