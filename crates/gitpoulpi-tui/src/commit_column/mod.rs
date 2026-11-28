//! The commit column: a compact list of the Log's commits on the left of
//! the Diff, File View, Blame and Edit views of a commit's file.
//!
//! It is a second *view* of the Log's state, never a second state: the
//! rows are the Log's own layout (`graph::LogLayout`: same commits, same
//! order, same lane glyphs through `graph::text_line`), the selected row is
//! the Log's selection (`log_state`), and picking a row moves that
//! selection. The right panel (commit info and file list) therefore follows
//! for free, and [`follow`] re-points the open view at the picked commit
//! once that commit's file list has landed. The paging is the Log's
//! (`lazy_log`), asked from the frame, never from here.
//!
//! Not part of it: the File History (`history_view`), a separate feature
//! with its own list of one file's commits (it hides and reopens the same
//! way, `STRIP_COLS`, `paint_strip`, with a choice of its own). The column
//! is hidden while a History is open, and for views that are not of a
//! commit (the working tree's diffs, a comparison).

#[allow(unused_imports)]
use crate::log_table::LogColumnExt;
mod follow;
mod picture;
mod render;
mod selector;

pub(crate) use follow::{open_file, plan, viewed_commit, Retarget, ViewKind};
pub(crate) use picture::ColumnPicture;
pub(crate) use render::{paint_strip, render, render_strip, ColumnPaint};
pub(crate) use selector::{selector, words as selector_words};

use gitpoulpi_core::domain::Oid;
use ratatui::layout::Rect;

/// From this terminal width (columns) the column is shown on its own
/// (`Pref::Auto`); below, the user can still turn it on.
pub(crate) const AUTO_WIDTH: u16 = 140;
/// Its narrowest (cells, border included).
pub(crate) const MIN_COLS: u16 = 24;
/// The most of the central column it may take, in percent; the rest stays
/// the diff's (and `MIN_VIEW_COLS` below is the floor the view keeps either
/// way). The cap used to be a flat 100 cells: on a wide window that left the
/// diff well over half of the room while the list, now able to hold a Refs
/// part and a long message, scrolled sideways. A share of the window grows
/// with it, and 70% still leaves the diff a readable third.
pub(crate) const MAX_SHARE_PCT: u16 = 70;
/// The widest a saved or dragged part may be (cells).
pub(crate) const MAX_PART_COLS: u16 = 300;
/// What the view beside it keeps, whatever the column asks for: a column
/// that cannot leave this much is not drawn at all.
pub(crate) const MIN_VIEW_COLS: u16 = 30;
/// The cells the reopen strip takes beside the view while the column is
/// hidden: the pane's seam (its own column, kept as the resize grab) and the
/// strip itself.
pub(crate) const STRIP_COLS: u16 = 2;
/// One keyboard step of the width.
pub(crate) const STEP: u16 = 4;

/// The user's choice, remembered in the session file: follow the terminal's
/// width, or force it on or off.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Pref {
    Auto,
    On,
    Off,
}

impl Pref {
    pub(crate) fn from_saved(v: u8) -> Pref {
        match v {
            1 => Pref::On,
            2 => Pref::Off,
            _ => Pref::Auto,
        }
    }

    pub(crate) fn saved(self) -> u8 {
        match self {
            Pref::Auto => 0,
            Pref::On => 1,
            Pref::Off => 2,
        }
    }
}

mod vocab;
pub(crate) use vocab::{ColumnCmd, ColumnState, Part, Side};

/// Which list the column shows *when it is shown*: the bar's `File | All`
/// selector. Remembered apart from shown / hidden, so reopening the column
/// (the strip, `Alt+C`, a click on a segment) restores it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum ColumnMode {
    /// The commits of the open file (the File History's list).
    File,
    /// Every commit, with the Log's graph.
    All,
}

impl ColumnMode {
    /// The saved value: `0` (also anything unknown, and a session written
    /// before the mode was kept) is `All`, which is what the column always
    /// showed first.
    pub(crate) fn from_saved(v: u8) -> ColumnMode {
        if v == 1 {
            ColumnMode::File
        } else {
            ColumnMode::All
        }
    }

    pub(crate) fn saved(self) -> u8 {
        match self {
            ColumnMode::All => 0,
            ColumnMode::File => 1,
        }
    }

    /// The state that shows this mode.
    pub(crate) fn state(self) -> ColumnState {
        match self {
            ColumnMode::File => ColumnState::File,
            ColumnMode::All => ColumnState::Graph,
        }
    }
}

/// How many parts a row can have (`Part::ORDER.len()`).
pub(crate) const PARTS: usize = 7;

/// The parts shown when the user chose nothing, by `Part::slot` (lanes,
/// author, date, refs, message, changes, sha): Refs, Changes and Sha are
/// options.
pub(crate) const DEFAULT_SHOWN: [bool; PARTS] = [true, true, true, false, true, false, false];

/// A boundary of the header, where a drag resizes: the right edge of the
/// part it names, which is the first cell of the next shown part (the Log's
/// rule: its boundary is the first letter of the next column's header).
pub(crate) type Boundary = Part;

/// The saved order's parts: unknown names and repeats ignored, the parts
/// the list leaves out following in default order (the Log's rule), so an
/// empty list is the default order.
pub(crate) fn order_from_names(names: &[String]) -> [Part; PARTS] {
    let mut order: Vec<Part> = Vec::with_capacity(PARTS);
    for part in names.iter().filter_map(|n| Part::from_name(n)) {
        if !order.contains(&part) {
            order.push(part);
        }
    }
    for part in Part::ORDER {
        if !order.contains(&part) {
            order.push(part);
        }
    }
    let mut out = Part::ORDER;
    out.copy_from_slice(&order);
    out
}

/// One keyboard step of a part's width.
pub(crate) const PART_STEP: u16 = 2;

/// The widest the column may be in a central column `column_width` wide
/// (`MAX_SHARE_PCT`), never under its least.
pub(crate) fn max_cols(column_width: u16) -> u16 {
    let share = u32::from(column_width) * u32::from(MAX_SHARE_PCT) / 100;
    u16::try_from(share).unwrap_or(u16::MAX).max(MIN_COLS)
}

/// The shown parts a saved list of Log column names stands for
/// (`DEFAULT_SHOWN` for an empty list; the message is always shown).
pub(crate) fn shown_from_names(names: &[String]) -> [bool; PARTS] {
    if names.is_empty() {
        return DEFAULT_SHOWN;
    }
    let mut shown = [false; PARTS];
    for part in Part::ORDER {
        shown[part.slot()] = !part.hideable()
            || names
                .iter()
                .any(|n| n.as_str() == part.log_column().config_name());
    }
    shown
}

/// The commit column's own part choices as `config.toml` holds them
/// (`ui.commit_column_columns_visible`, `ui.commit_column_columns_order`,
/// `ui.commit_column_columns_widths`), global like the Log's column layout and apart from it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ColumnPrefs {
    /// Shown parts by the Log's stable column names; empty = the default.
    pub(crate) visible: Vec<String>,
    /// The parts' order by name; empty = the default order.
    pub(crate) order: Vec<String>,
    /// Dragged widths by `Part::slot`; `0` = automatic.
    pub(crate) widths: [u16; PARTS],
}

impl ColumnPrefs {
    pub(crate) fn from_ui(ui: &gitpoulpi_config::UiConfig) -> Self {
        let mut widths = [0u16; PARTS];
        for part in Part::ORDER {
            if let Some(w) = ui
                .commit_column_columns_widths
                .get(part.log_column().config_name())
            {
                widths[part.slot()] = (*w).clamp(2, 300);
            }
        }
        Self {
            visible: ui.commit_column_columns_visible.clone(),
            order: ui.commit_column_columns_order.clone(),
            widths,
        }
    }

    /// What an older session file held (before these moved to the config).
    pub(crate) fn from_legacy(layout: &gitpoulpi_session::PaneLayout) -> Self {
        let mut widths = [0u16; PARTS];
        for (slot, w) in widths.iter_mut().zip(&layout.commit_column_widths) {
            *slot = *w;
        }
        Self {
            visible: layout.commit_column_visible.clone(),
            order: Vec::new(),
            widths,
        }
    }

    pub(crate) fn is_default(&self) -> bool {
        self.visible.is_empty() && self.order.is_empty() && self.widths == [0; PARTS]
    }

    /// The widths as the config stores them: part name -> cells, automatic
    /// parts left out.
    pub(crate) fn width_map(&self) -> std::collections::BTreeMap<String, u16> {
        Part::ORDER
            .iter()
            .filter(|p| self.widths[p.slot()] != 0)
            .map(|p| {
                (
                    p.log_column().config_name().to_string(),
                    self.widths[p.slot()],
                )
            })
            .collect()
    }

    /// Writes both keys to `config.toml` on a short-lived thread (the input
    /// loop never waits on the disk).
    pub(crate) fn save_in_background(&self, path: &std::path::Path) {
        let (visible, order, widths, path) = (
            self.visible.clone(),
            self.order.clone(),
            self.width_map(),
            path.to_path_buf(),
        );
        std::thread::spawn(move || {
            let _ = gitpoulpi_config::set_commit_column_in_file(&path, &visible, &order, &widths);
        });
    }
}

thread_local! {
    /// The state as painted this frame, for the bar's control to name
    /// (set by `draw_central` before the bar is drawn).
    static PAINTED: std::cell::Cell<ColumnState> = const { std::cell::Cell::new(ColumnState::Off) };
    /// The remembered mode, for the selector to show while the column is
    /// closed.
    static MODE: std::cell::Cell<ColumnMode> = const { std::cell::Cell::new(ColumnMode::All) };
}

pub(crate) fn note_mode(mode: ColumnMode) {
    MODE.with(|c| c.set(mode));
}

pub(crate) fn remembered() -> ColumnMode {
    MODE.with(std::cell::Cell::get)
}

pub(crate) fn note_painted(state: ColumnState) {
    PAINTED.with(|c| c.set(state));
}

pub(crate) fn painted() -> ColumnState {
    PAINTED.with(std::cell::Cell::get)
}

/// What a row of the header's menu does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ColumnsAct {
    /// Shows or hides the part.
    Toggle(Part),
    /// Fits the part to its content.
    Fit(Part),
    /// Fits every shown part to its content.
    FitAll,
}

/// What a hit on the column means.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ColumnHit {
    /// A commit's row (by id: the rows are the Log's, whatever solo shows).
    Row(Oid),
    /// The splitter between the column and the view.
    Divider,
    /// The column's own close cross.
    Close,
    /// A boundary of the header: the grab of a part's width.
    Boundary(Boundary),
    /// A part's name in the header: a press may start dragging it to
    /// another place.
    Header(Part),
    /// A row of the header's parts menu (`CommitColumn::columns_menu_entries`).
    ColumnsMenuItem(usize),
    /// The cross in that menu's frame.
    ColumnsMenuClose,
    /// The strip left of the view while the column is hidden: brings it back.
    Reopen,
}

#[derive(Debug)]
pub(crate) struct CommitColumn {
    pub(crate) pref: Pref,
    /// Which list it shows when shown (remembered while hidden).
    pub(crate) mode: ColumnMode,
    /// The widths the user dragged the parts to, in cells, by `Part::slot`
    /// (lanes, author, date, refs, message); `0` = automatic.
    pub(crate) widths: [u16; PARTS],
    /// Which parts the rows show, by `Part::slot`: its own choice, apart
    /// from the Log's columns (`DEFAULT_SHOWN` until the menu is used).
    pub(crate) shown: [bool; PARTS],
    /// The parts' order, left to right (a hidden part keeps its place):
    /// its own, apart from the Log's column order.
    pub(crate) order: [Part; PARTS],
    /// The header being dragged to another place.
    pub(crate) dragging_header: Option<Part>,
    /// The pointer while a header is carried (`None` until it moved): the
    /// ghost follows it.
    pub(crate) header_drag_at: Option<(u16, u16)>,
    /// The part the carried one would be dropped onto, and the screen cell of
    /// the insertion bar. The order changes on release only.
    pub(crate) header_drop: Option<(Part, (u16, u16))>,
    /// The commits whose rows the Changes part shows this frame, for the
    /// numbers to be asked for (empty while it is hidden).
    pub(crate) visible_ids: Vec<Oid>,
    /// The header row as painted (empty when the column is not drawn): the
    /// right-click on it opens the parts menu.
    pub(crate) header: Rect,
    /// How far the rows are scrolled sideways, and the last painted extent
    /// of the row and of the room for it.
    pub(crate) h_offset: usize,
    pub(crate) content_w: usize,
    pub(crate) inner_w: usize,
    /// The part the keyboard sizes.
    pub(crate) part: Part,
    /// What each part needs to show the rows the last paint drew whole, by
    /// `Part::slot` (the header's own word included): what a fit reads.
    pub(crate) natural: [usize; PARTS],
    /// The press that armed a boundary drag pinned the message's width too
    /// (it was automatic): a double click on that boundary, the fit, gives it
    /// back.
    press_pinned_title: bool,
    /// The boundary being dragged.
    pub(crate) dragging_part: Option<Boundary>,
    /// Where each part was painted this frame: its screen columns
    /// (`x0..x1`), for the pointer to name the part under it.
    pub(crate) parts: Vec<(Part, u16, u16)>,
    /// The separator cell of each boundary of the header, as painted.
    pub(crate) boundaries: Vec<(Boundary, Rect)>,
    /// The kind of view the next follow step opens at the picked commit,
    /// whatever the view open now is (the column's menu rows).
    pub(crate) force: Option<ViewKind>,
    /// How the last painted row was shared between its parts (what a drag
    /// of a boundary measures against).
    plan: picture::Widths,
    /// The width the user dragged it to, in cells; `0` = the default share.
    pub(crate) cols: u16,
    /// First row shown, and whether it follows the selection (a key moved
    /// it) or stays where the wheel or the scrollbar left it.
    pub(crate) offset: usize,
    pub(crate) follow: bool,
    /// The keyboard is on the column (a click on it, the badge, the palette
    /// command): the arrows move its selection instead of the view's cursor.
    pub(crate) keys: bool,
    pub(crate) dragging: bool,
    /// Where it was painted this frame (empty when it was not), the rows
    /// inside it, and the central column it was cut out of.
    pub(crate) area: Rect,
    pub(crate) rows_area: Rect,
    pub(crate) column: Rect,
    /// The reopen strip, painted while the column is hidden but could be
    /// shown (empty otherwise).
    pub(crate) strip: Rect,
    /// What the view got of that column (all of it while no column is drawn).
    pub(crate) view: Rect,
    /// The rows of the list, as painted: what the scrollbar and the lazy
    /// loader measure.
    pub(crate) len: usize,
}

impl CommitColumn {
    pub(crate) fn new(saved_pref: u8, saved_cols: u16) -> CommitColumn {
        CommitColumn {
            pref: Pref::from_saved(saved_pref),
            mode: ColumnMode::All,
            widths: [0; PARTS],
            shown: DEFAULT_SHOWN,
            order: Part::ORDER,
            dragging_header: None,
            header_drag_at: None,
            header_drop: None,
            visible_ids: Vec::new(),
            header: Rect::default(),
            h_offset: 0,
            content_w: 0,
            inner_w: 0,
            part: Part::Graph,
            natural: [0; PARTS],
            press_pinned_title: false,
            dragging_part: None,
            parts: Vec::new(),
            boundaries: Vec::new(),
            force: None,
            plan: picture::Widths::default(),
            cols: saved_cols,
            offset: 0,
            follow: true,
            keys: false,
            dragging: false,
            area: Rect::default(),
            rows_area: Rect::default(),
            column: Rect::default(),
            strip: Rect::default(),
            view: Rect::default(),
            len: 0,
        }
    }

    /// The mode and the part widths of a saved session (`commit_column_mode`,
    /// `commit_column_widths`); a missing or short list is automatic.
    pub(crate) fn with_saved(mut self, mode: u8, widths: &[u16]) -> CommitColumn {
        self.mode = ColumnMode::from_saved(mode);
        for (slot, w) in self.widths.iter_mut().zip(widths) {
            *slot = *w;
        }
        self
    }

    /// The parts a saved session shows (`commit_column_visible`, the Log
    /// columns' stable names); nothing saved is `DEFAULT_SHOWN`. The message
    /// is always shown.
    pub(crate) fn with_visible(mut self, names: &[String]) -> CommitColumn {
        self.shown = shown_from_names(names);
        self
    }

    /// The parts' order from a saved list of names (`order_from_names`).
    pub(crate) fn with_order(mut self, names: &[String]) -> CommitColumn {
        self.order = order_from_names(names);
        self
    }

    /// The names to save for the order: empty while it is the default.
    pub(crate) fn order_names(&self) -> Vec<String> {
        if self.order == Part::ORDER {
            return Vec::new();
        }
        self.order
            .iter()
            .map(|p| p.log_column().config_name().to_string())
            .collect()
    }

    /// Puts `dragged` in the place of `target`, as the Log's drag does (the
    /// parts between shift by one). Dragged to the right it lands after the
    /// target, so the last place can be reached. `false` when nothing moved.
    pub(crate) fn move_onto(&mut self, dragged: Part, target: Part) -> bool {
        let (Some(from), Some(to)) = (
            self.order.iter().position(|p| *p == dragged),
            self.order.iter().position(|p| *p == target),
        ) else {
            return false;
        };
        if from == to {
            return false;
        }
        if from < to {
            self.order[from..=to].rotate_left(1);
        } else {
            self.order[to..=from].rotate_right(1);
        }
        true
    }

    /// The row as the widths and the order now say it (what the next paint
    /// draws), so a drag reads positions that are never a frame behind.
    fn fresh_plan(&self) -> picture::Widths {
        picture::plan_custom(
            self.inner_w,
            self.plan.graph,
            self.widths,
            self.shown,
            self.order,
        )
    }

    /// A drag of `dragged`'s header to the pointer at screen cell `(col,
    /// row)`: finds the part it would take the place of and the cell of the
    /// insertion bar, without moving anything (the order changes on release,
    /// `drop_header`). A move to the right waits until the pointer is where
    /// the dragged part would then begin, so the target does not flicker.
    pub(crate) fn drag_header(&mut self, dragged: Part, col: u16, row: u16) {
        self.header_drag_at = Some((col, row));
        self.header_drop = None;
        let x = self.content_x(col);
        let spans = self.fresh_plan().spans();
        let Some((_, dragged_at, _)) = spans.iter().copied().find(|(p, _, _)| *p == dragged) else {
            return;
        };
        let Some((target, at, target_w)) = spans
            .iter()
            .copied()
            .find(|(_, at, w)| x >= *at && x < at + w)
        else {
            return;
        };
        let right = x > dragged_at;
        if target == dragged || (right && x < dragged_at + target_w + 1) {
            return;
        }
        let edge = if right { at + target_w } else { at };
        let screen = self.rows_area.x as i32 + edge as i32 - self.h_offset as i32;
        let header_row = self.rows_area.y.saturating_sub(1);
        // The air cell between two parts, so no letter is covered: after the
        // target going right, before it going left.
        let sx = if right { screen } else { screen - 1 }.max(self.rows_area.x as i32);
        self.header_drop = Some((target, (sx as u16, header_row)));
    }

    /// The carried header was released: it takes the place of the part it was
    /// over. `true` when the order changed.
    pub(crate) fn drop_header(&mut self) -> bool {
        let dragged = self.dragging_header.take();
        let drop = self.header_drop.take();
        self.header_drag_at = None;
        match (dragged, drop) {
            (Some(d), Some((target, _))) => self.move_onto(d, target),
            _ => false,
        }
    }

    /// Esc: the carried header is let go and nothing moves.
    pub(crate) fn cancel_header_drag(&mut self) -> bool {
        let was = self.header_drag_at.is_some();
        self.dragging_header = None;
        self.header_drop = None;
        self.header_drag_at = None;
        was
    }

    /// Moves `part` one place toward `side` among the shown parts (the
    /// keyboard's twin of the drag); `false` at the end of the row.
    pub(crate) fn move_part(&mut self, part: Part, side: Side) -> bool {
        let shown: Vec<Part> = self
            .order
            .iter()
            .copied()
            .filter(|p| self.shown[p.slot()])
            .collect();
        let Some(at) = shown.iter().position(|p| *p == part) else {
            return false;
        };
        let neighbor = match side {
            Side::Left => at.checked_sub(1),
            Side::Right => Some(at + 1),
        }
        .and_then(|i| shown.get(i).copied());
        neighbor.is_some_and(|n| self.move_onto(part, n))
    }

    /// The names to save for the shown parts: empty while they are the
    /// default.
    pub(crate) fn visible_names(&self) -> Vec<String> {
        if self.shown == DEFAULT_SHOWN {
            return Vec::new();
        }
        Part::ORDER
            .iter()
            .filter(|p| self.shown[p.slot()])
            .map(|p| p.log_column().config_name().to_string())
            .collect()
    }

    pub(crate) fn is_shown(&self, part: Part) -> bool {
        self.shown[part.slot()]
    }

    /// Shows or hides `part`; `false` when it cannot be hidden (the message).
    pub(crate) fn toggle_shown(&mut self, part: Part) -> bool {
        if !part.hideable() {
            return false;
        }
        let slot = part.slot();
        self.shown[slot] = !self.shown[slot];
        if self.part == part && !self.shown[slot] {
            self.part = Part::Title;
        }
        true
    }

    /// The rows of the header's parts menu: one per part, a box and the
    /// Log's own column word; the message is listed, not selectable. Then
    /// the fits: the part the menu was opened on (`target`, when there is
    /// one) and all of them.
    pub(crate) fn columns_menu_entries(
        &self,
        target: Option<Part>,
    ) -> Vec<crate::context_menu::Entry<ColumnsAct>> {
        use crate::context_menu::Entry;
        use crate::keymap::Action;
        let mut rows: Vec<Entry<ColumnsAct>> = Part::ORDER
            .iter()
            .map(|&part| {
                let glyph = crate::dialog_kit::checkbox_mark(self.is_shown(part));
                let label = crate::context_menu::glyph_label(glyph, part.title());
                if part.hideable() {
                    Entry::action(label, ColumnsAct::Toggle(part))
                } else {
                    Entry::disabled(
                        label,
                        gitpoulpi_i18n::msg::log_draw::ALWAYS_SHOWN
                            .get()
                            .trim_matches(['(', ')', '\u{ff08}', '\u{ff09}']),
                    )
                }
            })
            .collect();
        rows.push(Entry::Separator);
        if let Some(part) = target.filter(|p| self.is_shown(*p)) {
            rows.push(
                Entry::action(
                    gitpoulpi_i18n::msg::log_draw::column_menu_fit(part.title()),
                    ColumnsAct::Fit(part),
                )
                .with_key(Action::CommitColumnFit),
            );
        }
        rows.push(
            Entry::action(
                gitpoulpi_i18n::msg::log_draw::COLUMN_MENU_FIT_ALL.get(),
                ColumnsAct::FitAll,
            )
            .with_key(Action::CommitColumnFitAll),
        );
        rows
    }

    /// Whether `(col, row)` is on the painted header row.
    pub(crate) fn header_contains(&self, col: u16, row: u16) -> bool {
        self.header.contains((col, row).into())
    }

    /// Whether the user (or the terminal's width) wants the column.
    pub(crate) fn wanted(&self, terminal_width: u16) -> bool {
        match self.pref {
            Pref::Auto => terminal_width >= AUTO_WIDTH,
            Pref::On => true,
            Pref::Off => false,
        }
    }

    /// The width it takes out of a central column `column_width` wide, or
    /// `None` when the view beside it would be left under `MIN_VIEW_COLS`.
    pub(crate) fn width_in(&self, column_width: u16) -> Option<u16> {
        let room = column_width.checked_sub(MIN_VIEW_COLS)?;
        if room < MIN_COLS {
            return None;
        }
        let wanted = if self.cols == 0 {
            (column_width / 2).clamp(MIN_COLS + 6, 46)
        } else {
            self.cols
        };
        Some(wanted.clamp(MIN_COLS, max_cols(column_width)).min(room))
    }

    /// The column's rect and the view's, cut out of `column`.
    pub(crate) fn split(&self, column: Rect) -> Option<(Rect, Rect)> {
        let w = self.width_in(column.width)?;
        Some((
            Rect::new(column.x, column.y, w, column.height),
            Rect::new(column.x + w, column.y, column.width - w, column.height),
        ))
    }

    /// The splitter's cells (the view's left border, beside the column: the
    /// column starts under the view's bar row) for the width the column has
    /// now, `None` when it does not fit.
    pub(crate) fn divider_rect(&self) -> Option<Rect> {
        let (list, _) = self.split(self.column)?;
        Some(Rect::new(
            list.right(),
            self.column.y,
            1,
            self.column.height,
        ))
    }

    /// The width a pointer on the splitter at `col` stands for.
    pub(crate) fn cols_at(&self, col: u16) -> u16 {
        col.saturating_sub(self.column.x)
            .clamp(MIN_COLS, max_cols(self.column.width))
    }

    /// The width now (what `split` gave it, or the saved one).
    pub(crate) fn current_cols(&self) -> u16 {
        if self.area.width > 0 {
            self.area.width
        } else {
            self.width_in(self.column.width).unwrap_or(0)
        }
    }

    pub(crate) fn set_cols(&mut self, cols: u16) {
        self.cols = cols.clamp(MIN_COLS, max_cols(self.column.width));
    }

    /// Fixes every shown part at the width it has now, so that sizing one
    /// never moves the others: the first drag (or key) leaves the automatic
    /// share. A part with a width of its own is left alone, so it is safe to
    /// call before every resize.
    pub(crate) fn pin_widths(&mut self) {
        self.pin_widths_except_title(false);
    }

    /// [`Self::pin_widths`], the message left automatic when `flexible`: it
    /// keeps what the other parts leave, so a part fitted wider takes its
    /// cells from the message instead of pushing the row past the pane.
    fn pin_widths_except_title(&mut self, flexible: bool) {
        let p = self.plan;
        let cells = |w: usize| u16::try_from(w).unwrap_or(MAX_PART_COLS);
        let now = |part: Part| match part {
            Part::Graph => p.graph,
            Part::Author => p.author.max(picture::AUTHOR_W),
            Part::Date => p.date.max(picture::DATE_W),
            Part::Refs => p.refs.max(picture::REFS_W),
            Part::Title => p.title.max(picture::MIN_MESSAGE),
            Part::Changes => p.changes.max(picture::CHANGES_W),
            Part::Sha => p.sha.max(picture::SHA_W),
        };
        for part in Part::ORDER {
            let slot = part.slot();
            if flexible && part == Part::Title {
                continue;
            }
            if self.shown[slot] && self.widths[slot] == 0 {
                self.widths[slot] = cells(now(part));
            }
        }
    }

    /// Where the pointer at screen column `col` is in the row, scrolled
    /// sideways included.
    fn content_x(&self, col: u16) -> usize {
        usize::from(col.saturating_sub(self.rows_area.x)) + self.h_offset
    }

    /// A drag of `boundary` (the right edge of that part) to the pointer at
    /// `col`: the part takes the width that puts the next part's first
    /// letter under the pointer. Every other part keeps its width, so the
    /// parts after it move and the row grows past the pane (it scrolls
    /// sideways) instead of squeezing anything. The last part has no next
    /// letter: its end handle is the cell right after it, which the pointer
    /// then covers.
    pub(crate) fn drag_boundary(&mut self, boundary: Boundary, col: u16) {
        self.pin_widths();
        let x = self.content_x(col);
        let spans = self.plan.spans();
        let Some((_, start, _)) = spans.iter().copied().find(|(p, _, _)| *p == boundary) else {
            return;
        };
        let last = spans.last().is_some_and(|(p, _, _)| *p == boundary);
        // In `line` mode the pointer holds the separator, a cell before the
        // next part's first letter.
        let x = x + usize::from(crate::column_resize::pointer_to_edge(
            crate::column_resize::mode(),
            !last,
        ));
        let w = x
            .saturating_sub(start + usize::from(!last))
            .max(boundary.min_width());
        self.widths[boundary.slot()] = u16::try_from(w).unwrap_or(MAX_PART_COLS).min(MAX_PART_COLS);
    }

    /// The header cell of `boundary` for the widths now: the first letter of
    /// the next part, where the next paint will put it (a drag lights it
    /// before that paint). For the last part, the cell right after it (the
    /// column's right border when the row fills the room).
    pub(crate) fn boundary_rect(&self, boundary: Boundary) -> Option<Rect> {
        let plan = picture::plan_custom(
            self.inner_w,
            self.plan.graph,
            self.widths,
            self.shown,
            self.order,
        );
        let spans = plan.spans();
        let at = spans.iter().position(|(p, _, _)| *p == boundary)?;
        let (rel, last) = match spans.get(at + 1) {
            Some((_, rel, _)) => (*rel, false),
            None => (spans[at].1 + spans[at].2, true),
        };
        let x = rel.checked_sub(self.h_offset)?;
        // A last part dragged wider than the room has its end past the
        // column: the light stays on the column's border cell.
        let x = if last { x.min(self.inner_w) } else { x };
        (x <= self.inner_w).then(|| {
            let sx = self.rows_area.x + x as u16;
            let edge = if last {
                crate::column_resize::Edge::after(sx)
            } else {
                crate::column_resize::Edge::before(sx)
            };
            crate::column_resize::grab_rect(
                crate::column_resize::mode(),
                edge,
                self.rows_area.y.saturating_sub(1),
                self.rows_area.y + self.rows_area.height,
            )
        })
    }

    /// The keyboard's step: the part chosen (`part`) `delta` cells wider.
    pub(crate) fn resize_part(&mut self, delta: i32) {
        self.pin_widths();
        let part = if self.is_shown(self.part) {
            self.part
        } else {
            Part::Title
        };
        let slot = part.slot();
        let now = i32::from(self.widths[slot]);
        // A part the plan has no cells for (no lanes yet) stays automatic.
        if now == 0 {
            return;
        }
        self.widths[slot] =
            (now + delta).clamp(part.min_width() as i32, i32::from(MAX_PART_COLS)) as u16;
    }

    /// A press on a boundary: fixes every part where it is (so that moving one
    /// boundary moves nothing else) and arms the drag.
    pub(crate) fn grab_boundary(&mut self, boundary: Boundary) {
        self.press_pinned_title = self.widths[Part::Title.slot()] == 0;
        self.pin_widths();
        self.dragging_part = Some(boundary);
    }

    /// The second press of a double click on `boundary`: the drag the first
    /// press armed is dropped and the message is automatic again if that press
    /// was what pinned it, so the fit gives the part the cells and the message
    /// keeps what is left.
    pub(crate) fn release_for_fit(&mut self) {
        self.dragging_part = None;
        if std::mem::take(&mut self.press_pinned_title) {
            self.widths[Part::Title.slot()] = 0;
        }
    }

    /// The width that shows `part`'s widest painted content whole: never under
    /// its least, never over the pane (a part wider than the pane could not be
    /// read at all) nor `MAX_PART_COLS`.
    pub(crate) fn natural_width(&self, part: Part) -> u16 {
        let least = part.min_width();
        let room = self.inner_w.max(least);
        let wanted = self.natural[part.slot()].max(least).min(room);
        u16::try_from(wanted)
            .unwrap_or(MAX_PART_COLS)
            .min(MAX_PART_COLS)
    }

    /// Fits `part` to its content (the twin of double-clicking its boundary).
    /// The other parts keep the width they have. Returns the cells it got.
    pub(crate) fn fit_part(&mut self, part: Part) -> u16 {
        self.pin_widths_except_title(part != Part::Title);
        let w = self.natural_width(part);
        self.widths[part.slot()] = w;
        w
    }

    /// Fits every shown part to its content.
    pub(crate) fn fit_all(&mut self) {
        self.pin_widths();
        for part in Part::ORDER {
            if self.shown[part.slot()] {
                self.widths[part.slot()] = self.natural_width(part);
            }
        }
        // The message takes what the others leave when that already shows it
        // whole (the row then fits the pane): automatic again.
        let others: usize = Part::ORDER
            .iter()
            .filter(|p| **p != Part::Title && self.shown[p.slot()])
            .map(|p| usize::from(self.widths[p.slot()]) + 1)
            .sum();
        if self.inner_w.saturating_sub(others) >= self.natural[Part::Title.slot()] {
            self.widths[Part::Title.slot()] = 0;
        }
    }

    /// Every part back to its automatic width, and the rows back at the left.
    pub(crate) fn reset_widths(&mut self) {
        self.widths = [0; PARTS];
        self.h_offset = 0;
    }

    /// The part of a row under screen column `col`, when it is over one.
    pub(crate) fn part_at(&self, col: u16) -> Option<Part> {
        self.parts
            .iter()
            .find(|(_, x0, x1)| col >= *x0 && col < *x1)
            .map(|(p, _, _)| *p)
    }

    /// How far the rows can scroll sideways: what the row has beyond the
    /// room for it.
    pub(crate) fn max_h_offset(&self) -> usize {
        self.content_w.saturating_sub(self.inner_w)
    }

    /// Scrolls the rows sideways by `delta` cells, inside what there is.
    pub(crate) fn scroll_h(&mut self, delta: isize) {
        let to = (self.h_offset as isize + delta).max(0) as usize;
        self.h_offset = to.min(self.max_h_offset());
    }

    /// The width the part has now: the one dragged, else what the automatic
    /// plan gave it.
    pub(crate) fn part_width(&self, part: Part) -> u16 {
        match self.widths[part.slot()] {
            0 => {
                let p = self.plan;
                let auto = match part {
                    Part::Graph => p.graph,
                    Part::Author => p.author,
                    Part::Date => p.date,
                    Part::Refs => p.refs,
                    Part::Title => p.title,
                    Part::Changes => p.changes,
                    Part::Sha => p.sha,
                };
                u16::try_from(auto).unwrap_or(0)
            }
            w => w,
        }
    }

    /// Rows the list can show.
    pub(crate) fn viewport(&self) -> usize {
        self.rows_area.height as usize
    }

    /// Settles the offset: onto the selection when it follows it, inside the
    /// list either way.
    pub(crate) fn settle(&mut self, selected: Option<usize>, len: usize, viewport: usize) {
        if self.follow {
            if let Some(sel) = selected {
                if sel < self.offset {
                    self.offset = sel;
                } else if viewport > 0 && sel >= self.offset + viewport {
                    self.offset = sel + 1 - viewport;
                }
            }
        }
        self.offset = self.offset.min(len.saturating_sub(viewport.max(1)));
    }

    /// Scrolls the viewport only (the wheel, the scrollbar).
    pub(crate) fn scroll_to(&mut self, to: usize) {
        self.offset = to.min(self.len.saturating_sub(self.viewport().max(1)));
        self.follow = false;
    }

    /// Forgets what was painted: the column is not on screen this frame.
    pub(crate) fn hide(&mut self) {
        self.parts.clear();
        self.header = Rect::default();
        self.boundaries.clear();
        self.area = Rect::default();
        self.rows_area = Rect::default();
        self.strip = Rect::default();
        self.len = 0;
        self.visible_ids.clear();
    }
}

pub(crate) trait PartExt: Sized {
    /// Every part, in the order a row shows them.
    const ORDER: [Part; PARTS];
    /// The header's word: the Log's own column titles.
    fn title(self) -> &'static str;
    /// The part the keyboard sizes after this one, among the shown ones.
    fn next_sized(self, shown: [bool; PARTS]) -> Part;
    /// Its slot in the saved widths and in the shown flags.
    fn slot(self) -> usize;
    /// The Log column this part mirrors: its name in the saved list of
    /// shown parts (`LogColumn::config_name`) and its header word.
    fn log_column(self) -> crate::log_table::LogColumn;
    /// The part a saved column name stands for.
    fn from_name(name: &str) -> Option<Part>;
    /// Whether the menu may hide it: the message never goes, as in the Log.
    fn hideable(self) -> bool;
    /// The least width a part keeps.
    fn min_width(self) -> usize;
}

impl PartExt for Part {
    /// Every part, in the order a row shows them.
    const ORDER: [Part; PARTS] = [
        Part::Refs,
        Part::Graph,
        Part::Title,
        Part::Author,
        Part::Date,
        Part::Changes,
        Part::Sha,
    ];
    /// The header's word: the Log's own column titles.
    fn title(self) -> &'static str {
        use crate::log_table::LogColumn;
        match self {
            Part::Refs => LogColumn::Ref.title(),
            Part::Graph => LogColumn::Graph.title(),
            Part::Title => LogColumn::Message.title(),
            Part::Author => LogColumn::Author.title(),
            Part::Date => LogColumn::Date.title(),
            Part::Changes => LogColumn::Changes.title(),
            Part::Sha => LogColumn::Sha.title(),
        }
    }
    /// The part the keyboard sizes after this one, among the shown ones.
    fn next_sized(self, shown: [bool; PARTS]) -> Part {
        let at = Part::ORDER.iter().position(|p| *p == self).unwrap_or(0);
        (1..=Part::ORDER.len())
            .map(|i| Part::ORDER[(at + i) % Part::ORDER.len()])
            .find(|p| shown[p.slot()])
            .unwrap_or(self)
    }
    /// Its slot in the saved widths and in the shown flags.
    fn slot(self) -> usize {
        match self {
            Part::Graph => 0,
            Part::Author => 1,
            Part::Date => 2,
            Part::Refs => 3,
            Part::Title => 4,
            Part::Changes => 5,
            Part::Sha => 6,
        }
    }
    /// The Log column this part mirrors: its name in the saved list of
    /// shown parts (`LogColumn::config_name`) and its header word.
    fn log_column(self) -> crate::log_table::LogColumn {
        use crate::log_table::LogColumn;
        match self {
            Part::Refs => LogColumn::Ref,
            Part::Graph => LogColumn::Graph,
            Part::Title => LogColumn::Message,
            Part::Author => LogColumn::Author,
            Part::Date => LogColumn::Date,
            Part::Changes => LogColumn::Changes,
            Part::Sha => LogColumn::Sha,
        }
    }
    /// The part a saved column name stands for.
    fn from_name(name: &str) -> Option<Part> {
        Part::ORDER
            .iter()
            .copied()
            .find(|p| p.log_column().config_name() == name)
    }
    /// Whether the menu may hide it: the message never goes, as in the Log.
    fn hideable(self) -> bool {
        self != Part::Title
    }
    /// The least width a part keeps.
    fn min_width(self) -> usize {
        match self {
            Part::Refs => picture::MIN_REFS,
            Part::Graph => picture::min_lanes(),
            Part::Title => picture::MIN_MESSAGE,
            Part::Author => picture::MIN_AUTHOR,
            Part::Date => picture::MIN_DATE,
            Part::Changes => picture::MIN_CHANGES,
            Part::Sha => picture::MIN_SHA,
        }
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod render_tests;
