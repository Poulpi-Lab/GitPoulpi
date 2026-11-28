//! The Radar's sizable columns: the repository chip, the branches, the `#`, the author and the age. Widths
//! are automatic (fitted to the content, within a cap) until the user sizes a
//! column: by dragging its separator (double-click fits it again) or by key
//! (`C` picks, `[` / `]` size by the Log's own `KEY_STEP`, `0` resets), and
//! the choice is kept in `radar.toml`. The title is the flexible column.

use super::*;
use crate::log_table::KEY_STEP;

/// The narrowest a column the user sizes goes (drag, keys, stored width).
pub(crate) const MIN_WIDTH: usize = 1;
/// The narrowest an automatic (unsized) repository column is, and the least
/// room its sized limit keeps.
const AUTO_MIN: usize = 4;
/// The widest a user-sized column goes.
pub(crate) const MAX_WIDTH: usize = 60;
/// What the automatic repository width stops at.
const AUTO_REPO_CAP: usize = 40;
/// The widest the `#` column goes.
const MAX_NUM_WIDTH: usize = 24;
/// What the automatic author width stops at.
const AUTO_AUTHOR_CAP: usize = 14;
/// What the automatic branches width stops at.
const AUTO_BRANCHES_CAP: usize = 28;
/// Cells kept for the number, title and age beside a sized repository column.
const KEEP_FOR_TITLE: usize = 36;

/// A separator drag: which column, where the pointer was, how wide it was.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Drag {
    cell: Cell,
    start_x: u16,
    start_w: usize,
}

impl Drag {
    /// Whether this drag is sizing `cell`.
    pub(crate) fn is_for(&self, cell: Cell) -> bool {
        self.cell == cell
    }
}

/// The repository column's width: the user's, else the content's, within the
/// room the window gives it. Returns `(width, limit)`.
pub(crate) fn repo_width(content: usize, set: Option<u16>, window: u16) -> (usize, usize) {
    let limit = (window as usize)
        .saturating_sub(KEEP_FOR_TITLE)
        .clamp(AUTO_MIN, MAX_WIDTH);
    let auto = content.clamp(
        AUTO_MIN,
        AUTO_REPO_CAP.min((window as usize / 3).max(AUTO_MIN)),
    );
    let width = match set {
        Some(w) => (w as usize).clamp(MIN_WIDTH, limit),
        None => auto,
    };
    (width, limit)
}

/// The author column's width (0 = no author on any row).
pub(crate) fn author_width(content: usize, set: Option<u16>) -> usize {
    match set {
        Some(w) => (w as usize).clamp(MIN_WIDTH, MAX_WIDTH),
        None => content.min(AUTO_AUTHOR_CAP),
    }
}

/// The branches column's width (0 = no request names its branches).
pub(crate) fn branches_width(content: usize, set: Option<u16>) -> usize {
    match set {
        Some(w) if content > 0 => (w as usize).clamp(MIN_WIDTH, MAX_WIDTH),
        _ => content.min(AUTO_BRANCHES_CAP),
    }
}

/// A request's `source -> target` cut to `width` cells: the target stays whole
/// while the source gives way (an ellipsis), the target is cut only when the
/// source cannot keep three cells. Padded to `width`.
pub(crate) fn fit_branches(source: &str, target: Option<&str>, width: usize) -> String {
    const ARROW: &str = " \u{2192} ";
    let Some(target) = target else {
        return fit_pad(source, width);
    };
    let whole = format!("{source}{ARROW}{target}");
    if crate::display_width(&whole) <= width {
        return fit_pad(&whole, width);
    }
    let arrow = crate::display_width(ARROW);
    let target_w = crate::display_width(target);
    if width >= target_w + arrow + 3 {
        let cut = crate::fit(source, width - target_w - arrow);
        return fit_pad(&format!("{cut}{ARROW}{target}"), width);
    }
    // No room for a source worth reading: a lone ellipsis, then the target.
    if width >= arrow + 2 {
        let cut = crate::fit(target, width - arrow - 1);
        return fit_pad(&format!("\u{2026}{ARROW}{cut}"), width);
    }
    fit_pad(&whole, width)
}

/// The age column's width (0 = no age on any row).
pub(crate) fn age_width(content: usize, set: Option<u16>) -> usize {
    match set {
        Some(w) if content > 0 => (w as usize).clamp(MIN_WIDTH, MAX_WIDTH),
        _ => content,
    }
}

/// The `#` column's width: the user's, else the widest label plus the room for
/// the checks / conflict marks. Returns 0 when no row has a label.
pub(crate) fn num_width(label: usize, status: usize, set: Option<u16>) -> usize {
    if label == 0 {
        return 0;
    }
    match set {
        Some(w) => (w as usize).clamp(MIN_WIDTH, MAX_NUM_WIDTH),
        None => label + status,
    }
}

/// `text` cut to `width` cells (an ellipsis when cut) and padded to it.
pub(crate) fn fit_pad(text: &str, width: usize) -> String {
    let mut out = crate::fit(text, width);
    let used = crate::display_width(&out);
    out.push_str(&" ".repeat(width.saturating_sub(used)));
    out
}

/// `text` broken into lines of at most `width` cells, for a tooltip.
pub(crate) fn wrap(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::new();
    let mut used = 0;
    for c in text.chars() {
        let w = crate::display_width(&c.to_string());
        if used + w > width && !line.is_empty() {
            lines.push(std::mem::take(&mut line));
            used = 0;
        }
        line.push(c);
        used += w;
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}

impl RadarUi {
    fn slot(cell: Cell) -> Option<usize> {
        match cell {
            Cell::Repo => Some(0),
            Cell::Author => Some(1),
            Cell::Num => Some(2),
            Cell::Age => Some(3),
            Cell::Branches => Some(4),
            _ => None,
        }
    }

    fn column_name(cell: Cell) -> &'static str {
        match cell {
            Cell::Author => msg::radar::COLUMN_AUTHOR.get(),
            Cell::Num => "#",
            Cell::Age => msg::radar::COLUMN_AGE.get(),
            Cell::Branches => msg::radar::COLUMN_BRANCHES.get(),
            _ => msg::radar::COLUMN_REPOSITORY.get(),
        }
    }

    fn limit_of(&self, cell: Cell) -> usize {
        match cell {
            Cell::Repo => self.repo_max,
            Cell::Num => MAX_NUM_WIDTH,
            _ => MAX_WIDTH,
        }
    }

    /// Sets a column's width (clamped), `None` = automatic.
    fn set_width(&mut self, cell: Cell, width: Option<usize>) {
        let limit = self.limit_of(cell);
        let value = width.map(|w| w.clamp(MIN_WIDTH, limit) as u16);
        match cell {
            Cell::Repo => self.state.columns.repo = value,
            Cell::Author => self.state.columns.author = value,
            Cell::Age => self.state.columns.age = value,
            Cell::Num => self.state.columns.number = value,
            Cell::Branches => self.state.columns.branches = value,
            _ => {}
        }
    }

    fn width_of(&self, cell: Cell) -> usize {
        Self::slot(cell).map_or(0, |i| self.col_eff[i])
    }

    /// The palette's and the keys' column commands.
    pub(super) fn column_command(&mut self, cmd: ColumnCmd) -> Outcome {
        let step = usize::from(KEY_STEP);
        // Left to right on screen; a column with no width (no `#`, no author) is skipped.
        let order = [
            Cell::Repo,
            Cell::Branches,
            Cell::Num,
            Cell::Author,
            Cell::Age,
        ];
        let picked = self.col_pick.filter(|c| Self::slot(*c).is_some());
        match cmd {
            ColumnCmd::Pick => {
                let from = picked.and_then(|p| order.iter().position(|c| *c == p));
                let next = (1..=order.len())
                    .map(|k| order[(from.unwrap_or(order.len() - 1) + k) % order.len()])
                    .find(|c| *c == Cell::Repo || self.width_of(*c) > 0)
                    .unwrap_or(Cell::Repo);
                self.col_pick = Some(next);
                let width = self.width_of(next) as u64;
                self.say(msg::radar::column_picked(Self::column_name(next), width))
            }
            ColumnCmd::Wider | ColumnCmd::Narrower => {
                let cell = picked.unwrap_or(Cell::Repo);
                self.col_pick = Some(cell);
                let now = self.width_of(cell);
                let next = if cmd == ColumnCmd::Wider {
                    now + step
                } else {
                    now.saturating_sub(step)
                };
                self.set_width(cell, Some(next));
                self.save_state();
                // What the next paint will use; the message reports it.
                let shown = next.clamp(MIN_WIDTH, self.limit_of(cell));
                if let Some(i) = Self::slot(cell) {
                    self.col_eff[i] = shown;
                }
                self.say(msg::radar::column_width(
                    Self::column_name(cell),
                    shown as u64,
                ))
            }
            ColumnCmd::Reset => {
                self.state.columns = Default::default();
                self.save_state();
                self.say(msg::radar::COLUMN_RESET.get().to_string())
            }
            ColumnCmd::Fit => self.fit_column(picked.unwrap_or(Cell::Repo)),
            ColumnCmd::FitAll => self.fit_all_columns(),
        }
    }

    /// Fits `cell` to its content: the width that shows its widest entry
    /// whole, within what the column can be (`limit_of`). The twin of
    /// double-clicking its separator.
    pub(super) fn fit_column(&mut self, cell: Cell) -> Outcome {
        let Some(slot) = Self::slot(cell) else {
            return Outcome::None;
        };
        self.col_pick = self.col_pick.map(|_| cell);
        let shown = self.fit_one(cell, slot);
        self.save_state();
        self.say(msg::radar::column_width(
            Self::column_name(cell),
            shown as u64,
        ))
    }

    /// One column's fit: unsized again when its automatic width already shows
    /// every entry, else the width of the widest one (within the limit).
    fn fit_one(&mut self, cell: Cell, slot: usize) -> usize {
        let content = self.col_content[slot];
        let shown = if self.col_auto[slot] >= content {
            self.set_width(cell, None);
            self.col_auto[slot]
        } else {
            let w = content.clamp(MIN_WIDTH, self.limit_of(cell));
            self.set_width(cell, Some(w));
            w
        };
        self.col_eff[slot] = shown;
        shown
    }

    /// Fits every sizable column to its content.
    pub(super) fn fit_all_columns(&mut self) -> Outcome {
        for cell in [
            Cell::Repo,
            Cell::Branches,
            Cell::Num,
            Cell::Author,
            Cell::Age,
        ] {
            let Some(slot) = Self::slot(cell) else {
                continue;
            };
            // A column no row has (no `#`, no author) stays out.
            if self.col_content[slot] == 0 {
                continue;
            }
            self.fit_one(cell, slot);
        }
        self.save_state();
        self.say(msg::log_table::COLUMNS_FITTED_ALL.get().to_string())
    }

    /// The header menu: fits the column whose title was right-clicked (or the
    /// picked one, from the keyboard), or all of them.
    pub(crate) fn open_header_menu(&mut self, cell: Option<Cell>, col: u16, row: u16) {
        let mut entries: Vec<Entry<Act>> = Vec::new();
        if let Some(cell) = cell.filter(|c| Self::slot(*c).is_some()) {
            entries.push(Entry::action(
                msg::log_draw::column_menu_fit(Self::column_name(cell)),
                Act::FitColumn(SizedColumn::of(cell)),
            ));
        }
        entries.push(Entry::action(
            msg::log_draw::COLUMN_MENU_FIT_ALL.get(),
            Act::FitAllColumns,
        ));
        self.close_popups();
        self.menu = Some(Menu::new(
            msg::commit_column::COLUMNS_MENU_TITLE.get().to_string(),
            entries,
            col,
            row,
        ));
    }

    /// A key while a column is picked: `[` `]` `0` `Tab`, `Esc`/`Enter` leave.
    /// `None` = not a column key (it goes on to the list).
    pub(super) fn column_key(&mut self, key: KeyEvent) -> Option<Outcome> {
        if key
            .modifiers
            .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT)
        {
            return None;
        }
        // The header menu's keyboard twin: the picked column's own fit row
        // and the fit of all of them.
        if crate::context_menu::is_menu_key(key.code, key.modifiers) {
            let picked = self.col_pick;
            let at = self
                .hits
                .iter()
                .find_map(|(r, h)| (picked.is_some() && *h == Hit::Header(picked?)).then_some(*r));
            let (x, y) = at.map_or((0, 0), |r| (r.x, r.y + 1));
            self.open_header_menu(picked, x, y);
            return Some(Outcome::None);
        }
        Some(match key.code {
            KeyCode::Char('[') => self.column_command(ColumnCmd::Narrower),
            KeyCode::Char(']') => self.column_command(ColumnCmd::Wider),
            KeyCode::Char('0') => self.column_command(ColumnCmd::Reset),
            KeyCode::Char('=') => self.column_command(ColumnCmd::Fit),
            KeyCode::Char('+') => self.column_command(ColumnCmd::FitAll),
            KeyCode::Tab | KeyCode::BackTab => self.column_command(ColumnCmd::Pick),
            KeyCode::Esc | KeyCode::Enter => {
                self.col_pick = None;
                Outcome::None
            }
            _ => return None,
        })
    }

    /// A press on a separator: a double-click fits the column to its content, else a
    /// drag begins.
    pub(super) fn divider_press(&mut self, cell: Cell, x: u16, double: bool) -> Outcome {
        // A mouse drag never leaves the column picked: only the keyboard's
        // column mode (`C`) keeps one lit until Esc.
        if double {
            self.col_drag = None;
            return self.fit_column(cell);
        }
        self.col_drag = Some(Drag {
            cell,
            start_x: x,
            start_w: self.width_of(cell),
        });
        Outcome::None
    }

    /// The pointer moved with the button down on a separator.
    pub(super) fn divider_drag(&mut self, x: u16) {
        let Some(drag) = self.col_drag else {
            return;
        };
        let delta = i32::from(x) - i32::from(drag.start_x);
        // The author and the age sit at the right: their separator is on the
        // left edge.
        let delta = if matches!(drag.cell, Cell::Author | Cell::Age) {
            -delta
        } else {
            delta
        };
        let width = (drag.start_w as i32 + delta).max(MIN_WIDTH as i32) as usize;
        self.set_width(drag.cell, Some(width));
    }

    /// The button came up: saves a finished drag. `true` when one ended.
    pub(super) fn divider_release(&mut self) -> bool {
        let was = self.col_drag.take().is_some();
        if was {
            self.save_state();
        }
        was
    }

    /// The tooltip text of a cut cell.
    pub(super) fn cut_text(item: &Item, cell: Cell) -> String {
        match cell {
            Cell::Repo if !item.project.is_empty() => item.project.clone(),
            Cell::Repo => item.repo_name.clone(),
            // A title whose Branches column is hidden or cut still has them.
            Cell::Title => match item.branches_text() {
                b if b.is_empty() => super::render::row_title(item),
                b => format!("{} ({b})", super::render::row_title(item)),
            },
            Cell::Author => item.author.clone(),
            Cell::Num => item.label.clone(),
            Cell::Age => item.updated_at.clone(),
            Cell::Labels => item.labels.join(" \u{00B7} "),
            Cell::Branches => item.branches_text(),
        }
    }
}

pub(crate) trait SizedColumnExt: Sized {
    fn of(cell: Cell) -> Self;
    fn cell(self) -> Cell;
}

impl SizedColumnExt for SizedColumn {
    fn of(cell: Cell) -> Self {
        match cell {
            Cell::Num => SizedColumn::Num,
            Cell::Branches => SizedColumn::Branches,
            Cell::Author => SizedColumn::Author,
            Cell::Age => SizedColumn::Age,
            _ => SizedColumn::Repo,
        }
    }
    fn cell(self) -> Cell {
        match self {
            SizedColumn::Repo => Cell::Repo,
            SizedColumn::Branches => Cell::Branches,
            SizedColumn::Num => Cell::Num,
            SizedColumn::Author => Cell::Author,
            SizedColumn::Age => Cell::Age,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn branches_are_cut_target_first_kept_then_source_then_target() {
        let cut = |w| fit_branches("feature/login-form", Some("main"), w);
        assert_eq!(cut(40).trim_end(), "feature/login-form \u{2192} main");
        assert_eq!(crate::display_width(&cut(40)), 40);
        let c = cut(15);
        assert!(c.ends_with("\u{2192} main"), "{c}");
        assert!(c.contains('\u{2026}'), "{c}");
        assert_eq!(crate::display_width(&c), 15);
        // No room for a source: an ellipsis and the target.
        let c = cut(8);
        assert!(c.starts_with('\u{2026}') && c.contains("\u{2192}"), "{c}");
        assert_eq!(crate::display_width(&c), 8);
        assert_eq!(
            fit_branches("alice:feature/x", None, 6).trim_end(),
            "alice\u{2026}"
        );
        for w in 1..30 {
            assert_eq!(crate::display_width(&cut(w)), w, "{w}");
        }
    }

    #[test]
    fn the_branches_width_is_automatic_capped_or_sized_and_absent_without_content() {
        assert_eq!(branches_width(0, Some(20)), 0);
        assert_eq!(branches_width(0, None), 0);
        assert_eq!(branches_width(50, None), AUTO_BRANCHES_CAP);
        assert_eq!(branches_width(12, None), 12);
        assert_eq!(branches_width(12, Some(30)), 30);
        assert_eq!(branches_width(12, Some(0)), 1);
        assert_eq!(branches_width(12, Some(500)), MAX_WIDTH);
    }

    #[test]
    fn the_repository_column_fits_its_content_within_a_cap() {
        assert_eq!(repo_width(22, None, 120).0, 22);
        assert_eq!(repo_width(80, None, 120).0, AUTO_REPO_CAP);
        // A narrow window caps it at a third.
        assert_eq!(repo_width(30, None, 60).0, 20);
        assert_eq!(repo_width(1, None, 120).0, AUTO_MIN);
    }

    #[test]
    fn a_set_width_is_kept_within_the_window() {
        assert_eq!(repo_width(10, Some(30), 120).0, 30);
        assert_eq!(repo_width(10, Some(2), 120).0, 2);
        assert_eq!(repo_width(10, Some(1), 120).0, 1);
        assert_eq!(repo_width(10, Some(0), 120).0, 1);
        let (w, limit) = repo_width(10, Some(200), 100);
        assert_eq!((w, limit), (64.min(MAX_WIDTH), 64.min(MAX_WIDTH)));
    }

    #[test]
    fn every_sized_column_goes_down_to_one_cell_but_automatic_widths_keep_their_floor() {
        assert_eq!(author_width(9, Some(1)), 1);
        assert_eq!(author_width(9, Some(0)), 1);
        assert_eq!(age_width(5, Some(1)), 1);
        assert_eq!(num_width(3, 0, Some(1)), 1);
        assert_eq!(num_width(3, 0, Some(0)), 1);
        assert_eq!(author_width(2, None), 2);
        assert_eq!(age_width(3, None), 3);
        assert_eq!(num_width(3, 3, None), 6);
    }

    #[test]
    fn a_cell_cut_to_one_or_two_cells_never_overflows_or_splits_a_wide_glyph() {
        for text in ["日本語", "\u{1F680}x", "abc", "a日"] {
            for width in 1..=3 {
                let cut = fit_pad(text, width);
                assert_eq!(
                    crate::display_width(&cut),
                    width,
                    "{text:?} at {width}: {cut:?}"
                );
            }
        }
        assert_eq!(fit_pad("日本語", 1), "\u{2026}");
        assert_eq!(fit_pad("日本語", 2), "\u{2026} ");
        assert_eq!(fit_pad("abc", 1), "\u{2026}");
        assert_eq!(fit_pad("a", 1), "a");
    }

    #[test]
    fn a_cut_cell_ends_in_an_ellipsis_and_pads_to_its_width() {
        let cut = fit_pad("group/subgroup/project", 10);
        assert_eq!(crate::display_width(&cut), 10);
        assert!(cut.contains('\u{2026}'), "{cut}");
        assert_eq!(fit_pad("ab", 5), "ab   ");
    }

    #[test]
    fn wrap_keeps_every_character() {
        let lines = wrap("group/subgroup/project-with-a-long-name", 12);
        assert!(lines.iter().all(|l| crate::display_width(l) <= 12));
        assert_eq!(lines.concat(), "group/subgroup/project-with-a-long-name");
    }
}
