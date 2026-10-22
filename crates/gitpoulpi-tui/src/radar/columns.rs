//! The Radar's sizable columns: the repository chip, the `#`, the author and the age. Widths
//! are automatic (fitted to the content, within a cap) until the user sizes a
//! column: by dragging its separator (double-click fits it again) or by key
//! (`C` picks, `[` / `]` size by the Log's own `KEY_STEP`, `0` resets), and
//! the choice is kept in `radar.toml`. The title is the flexible column.

use super::*;
use crate::log_table::KEY_STEP;

/// The narrowest a sizable column goes.
pub(crate) const MIN_WIDTH: usize = 4;
/// The widest a user-sized column goes.
pub(crate) const MAX_WIDTH: usize = 60;
/// What the automatic repository width stops at.
const AUTO_REPO_CAP: usize = 40;
/// The widest the `#` column goes.
const MAX_NUM_WIDTH: usize = 24;
/// What the automatic author width stops at.
const AUTO_AUTHOR_CAP: usize = 14;
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
        .clamp(MIN_WIDTH, MAX_WIDTH);
    let auto = content.clamp(
        MIN_WIDTH,
        AUTO_REPO_CAP.min((window as usize / 3).max(MIN_WIDTH)),
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
            _ => None,
        }
    }

    fn column_name(cell: Cell) -> &'static str {
        match cell {
            Cell::Author => msg::radar::COLUMN_AUTHOR.get(),
            Cell::Num => "#",
            Cell::Age => msg::radar::COLUMN_AGE.get(),
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
        let order = [Cell::Repo, Cell::Num, Cell::Author, Cell::Age];
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
        }
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
        Some(match key.code {
            KeyCode::Char('[') => self.column_command(ColumnCmd::Narrower),
            KeyCode::Char(']') => self.column_command(ColumnCmd::Wider),
            KeyCode::Char('0') => self.column_command(ColumnCmd::Reset),
            KeyCode::Tab | KeyCode::BackTab => self.column_command(ColumnCmd::Pick),
            KeyCode::Esc | KeyCode::Enter => {
                self.col_pick = None;
                Outcome::None
            }
            _ => return None,
        })
    }

    /// A press on a separator: a double-click fits the column again, else a
    /// drag begins.
    pub(super) fn divider_press(&mut self, cell: Cell, x: u16, double: bool) -> Outcome {
        // A mouse drag never leaves the column picked: only the keyboard's
        // column mode (`C`) keeps one lit until Esc.
        if double {
            self.col_drag = None;
            self.set_width(cell, None);
            self.save_state();
            return Outcome::None;
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
            Cell::Title => super::render::row_title(item),
            Cell::Author => item.author.clone(),
            Cell::Num => item.label.clone(),
            Cell::Age => item.updated_at.clone(),
            Cell::Labels => item.labels.join(" \u{00B7} "),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_repository_column_fits_its_content_within_a_cap() {
        assert_eq!(repo_width(22, None, 120).0, 22);
        assert_eq!(repo_width(80, None, 120).0, AUTO_REPO_CAP);
        // A narrow window caps it at a third.
        assert_eq!(repo_width(30, None, 60).0, 20);
        assert_eq!(repo_width(1, None, 120).0, MIN_WIDTH);
    }

    #[test]
    fn a_set_width_is_kept_within_the_window() {
        assert_eq!(repo_width(10, Some(30), 120).0, 30);
        assert_eq!(repo_width(10, Some(2), 120).0, MIN_WIDTH);
        let (w, limit) = repo_width(10, Some(200), 100);
        assert_eq!((w, limit), (64.min(MAX_WIDTH), 64.min(MAX_WIDTH)));
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
