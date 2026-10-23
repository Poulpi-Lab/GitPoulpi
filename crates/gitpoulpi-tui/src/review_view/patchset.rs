//! The patch-set combobox of a Gerrit review: a field on the view's top
//! border, next to the page tabs, that lists the change's patch sets
//! (newest first, the newest marked) and re-reads the review at the one
//! picked. Kept in its own file: it paints over the border after the view
//! has painted itself (`render_all`), so the view's own drawing code does
//! not know it exists.
//!
//! Keyboard twin: `P` and the palette row *Review: Choose Patch Set...*.
//! The hit rect is the one the field was painted in (`Combo::field_rect`).

use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use gitpoulpi_hosting::review::patch_set_of_ref;
use gitpoulpi_i18n::msg::review_view as msg;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::combo::{Combo, ComboOption, ComboOutcome};
use crate::dialog_kit;
use crate::display_width;

use super::ReviewView;

/// Columns of the field (the popup may be wider).
const FIELD_WIDTH: u16 = 24;
/// Columns of the "compare with" field, left of it.
const COMPARE_WIDTH: u16 = 26;

impl ReviewView {
    /// Builds the combobox from the host data that just arrived.
    pub(super) fn sync_patch_sets(&mut self) {
        let Some(h) = &self.host else {
            self.ps_combo = None;
            return;
        };
        let sets = &h.detail.patch_sets;
        if sets.is_empty() {
            self.ps_combo = None;
            return;
        }
        let latest = sets.iter().map(|p| p.number).max();
        let mut combo = Combo::new(msg::PATCH_SET_PLACEHOLDER.get());
        combo.set_options(
            sets.iter()
                .rev()
                .map(|p| {
                    let label = if Some(p.number) == latest {
                        msg::patch_set_latest(u64::from(p.number))
                    } else {
                        msg::patch_set_item(u64::from(p.number))
                    };
                    ComboOption::new(label, p.number.to_string())
                })
                .collect(),
        );
        combo.select(h.detail.shown_patch_set.map(|n| n.to_string()));
        self.ps_combo = Some(combo);
        self.sync_compare();
    }

    /// The "compare with" combobox: the base, or an older patch set than the
    /// one shown.
    fn sync_compare(&mut self) {
        let Some(h) = &self.host else {
            self.cmp_combo = None;
            return;
        };
        let shown = h.detail.shown_patch_set;
        let mut options = vec![ComboOption::new(msg::COMPARE_BASE.get(), "0")];
        for p in h.detail.patch_sets.iter().rev() {
            if Some(p.number) < shown {
                options.push(ComboOption::new(
                    msg::compare_item(u64::from(p.number)),
                    p.number.to_string(),
                ));
            }
        }
        let mut combo = Combo::new(msg::COMPARE_PLACEHOLDER.get());
        combo.set_options(options);
        combo.select(Some(
            self.target
                .compare_patch_set
                .map_or_else(|| "0".to_string(), |k| k.to_string()),
        ));
        self.cmp_combo = Some(combo);
    }

    /// The patch set the git side should show: the one picked, else the one
    /// the list named (a Gerrit change ref), else what the host reports.
    pub(super) fn wanted_patch_set(&self) -> Option<u32> {
        self.target
            .patch_set
            .or_else(|| patch_set_of_ref(&self.target.source_branch))
    }

    /// After the host's answer: the code must be the patch set the host
    /// described (the list may have been a patch set behind).
    pub(super) fn follow_shown_patch_set(&mut self) {
        let shown = self.host.as_ref().and_then(|h| h.detail.shown_patch_set);
        if let Some(shown) = shown {
            if self.git_ps != Some(shown) {
                self.git_ps = Some(shown);
                self.git_err = None;
                self.git = None;
                self.restart_git();
            }
        }
    }

    /// Reads the review again at patch set `number`.
    pub(super) fn pick_patch_set(&mut self, number: u32) {
        let shown = self.host.as_ref().and_then(|h| h.detail.shown_patch_set);
        if shown == Some(number) {
            return;
        }
        self.target.patch_set = Some(number);
        // The old side must stay older than the new one.
        if self.target.compare_patch_set.is_some_and(|k| k >= number) {
            self.target.compare_patch_set = None;
        }
        self.message = Some(msg::loading_patch_set(u64::from(number)));
        self.refresh();
    }

    /// Opens the list (the key and the palette row).
    pub(super) fn open_patch_sets(&mut self) {
        match self.ps_combo.as_mut() {
            Some(combo) => combo.open(),
            None => self.message = Some(msg::WHY_NO_PATCH_SETS.get().to_string()),
        }
    }

    /// Reads the review again with the diff starting from patch set `from`
    /// (`None`: the target branch).
    fn pick_compare(&mut self, from: Option<u32>) {
        if self.target.compare_patch_set == from {
            return;
        }
        self.target.compare_patch_set = from;
        self.message = Some(match from {
            Some(k) => msg::loading_compare(u64::from(k)),
            None => msg::REFRESHING.get().to_string(),
        });
        self.refresh();
    }

    /// Opens the "compare with" list (the key and the palette row).
    pub(super) fn open_compare(&mut self) {
        match self.cmp_combo.as_mut() {
            Some(combo) => combo.open(),
            None => self.message = Some(msg::WHY_NO_PATCH_SETS.get().to_string()),
        }
    }

    pub(super) fn patch_sets_open(&self) -> bool {
        self.ps_combo.as_ref().is_some_and(Combo::is_open)
            || self.cmp_combo.as_ref().is_some_and(Combo::is_open)
    }

    /// Why the palette row does nothing, if it does not.
    pub(super) fn has_patch_sets(&self) -> bool {
        self.ps_combo.is_some()
    }

    fn on_ps_outcome(&mut self, outcome: ComboOutcome) {
        if let ComboOutcome::Picked(value) = outcome {
            if let Ok(number) = value.parse::<u32>() {
                self.pick_patch_set(number);
            }
        }
    }

    fn on_cmp_outcome(&mut self, outcome: ComboOutcome) {
        if let ComboOutcome::Picked(value) = outcome {
            if let Ok(number) = value.parse::<u32>() {
                self.pick_compare((number != 0).then_some(number));
            }
        }
    }

    pub(super) fn ps_key(&mut self, key: crossterm::event::KeyEvent) {
        if let Some(combo) = self.cmp_combo.as_mut().filter(|c| c.is_open()) {
            let outcome = combo.on_key(key);
            self.on_cmp_outcome(outcome);
        } else if let Some(combo) = self.ps_combo.as_mut() {
            let outcome = combo.on_key(key);
            self.on_ps_outcome(outcome);
        }
    }

    /// The mouse while a list is open, or on a field. `true`: used.
    pub(super) fn ps_mouse(&mut self, mouse: MouseEvent) -> bool {
        let ps_open = self.ps_combo.as_ref().is_some_and(Combo::is_open);
        let cmp_open = self.cmp_combo.as_ref().is_some_and(Combo::is_open);
        if !cmp_open {
            if let Some(combo) = self.ps_combo.as_mut() {
                if let Some(outcome) = drive(combo, self.ps_field, &mut self.ps_hover, mouse) {
                    self.on_ps_outcome(outcome);
                    return true;
                }
            }
        }
        if !ps_open {
            if let Some(combo) = self.cmp_combo.as_mut() {
                if let Some(outcome) = drive(combo, self.cmp_field, &mut self.cmp_hover, mouse) {
                    self.on_cmp_outcome(outcome);
                    return true;
                }
            }
        }
        false
    }

    /// [`Self::render`], then the combobox over the top border. The popup
    /// is painted last, over everything.
    pub(crate) fn render_all(&mut self, area: Rect, buf: &mut Buffer) {
        self.render(area, buf);
        self.ps_field = Rect::default();
        self.cmp_field = Rect::default();
        let popups = self.compose.is_some()
            || self.finish.is_some()
            || self.options.is_some()
            || self.confirm.is_some()
            || self.meta.is_some();
        let hover = self.ps_hover && !popups;
        let cmp_hover = self.cmp_hover && !popups;
        let Some(combo) = self.ps_combo.as_mut() else {
            return;
        };
        if popups {
            combo.close();
            if let Some(c) = self.cmp_combo.as_mut() {
                c.close();
            }
            return;
        }
        let r = dialog_kit::centered(
            area,
            area.width.saturating_sub(2),
            area.height.saturating_sub(2),
        );
        if r.width < 56 || r.height < 14 {
            return;
        }
        // Where the page tabs start (the same arithmetic as the border's).
        let tabs: u16 = [
            msg::TAB_OVERVIEW.get(),
            msg::TAB_FILES.get(),
            msg::TAB_COMMITS.get(),
        ]
        .iter()
        .map(|l| display_width(l) as u16 + 2)
        .sum();
        let close_x = r.right().saturating_sub(5);
        let tabs_x = close_x.saturating_sub(tabs + 1);
        // The title ` Review #12 ` sits at the left of the border.
        let title_end = r.x + 2 + display_width(&msg::title(&self.target.iid_label)) as u16 + 2;
        let width = FIELD_WIDTH.min(tabs_x.saturating_sub(title_end + 1));
        if width < 14 {
            return;
        }
        let rect = Rect::new(tabs_x - 1 - width, r.y, width, 1);
        combo.render_field(buf, rect, false, hover);
        self.ps_field = rect;
        // The "compare with" field to its left, when the border has room.
        let room = rect.x.saturating_sub(title_end + 2);
        let cmp_width = COMPARE_WIDTH.min(room);
        if let Some(c) = self.cmp_combo.as_mut() {
            if cmp_width >= 14 {
                let r = Rect::new(rect.x - 1 - cmp_width, rect.y, cmp_width, 1);
                c.render_field(buf, r, false, cmp_hover);
                self.cmp_field = r;
            } else {
                c.close();
            }
        }
        if let Some(c) = self.ps_combo.as_mut() {
            c.render_popup(buf, area);
        }
        if let Some(c) = self.cmp_combo.as_mut() {
            c.render_popup(buf, area);
        }
    }

    /// Asks before merging or abandoning the change (outward, no undo).
    pub(super) fn ask_change(&mut self, what: super::ConfirmFor) {
        let (prompt, yes) = match what {
            super::ConfirmFor::SubmitChange => (
                msg::submit_prompt(&self.target.iid_label),
                msg::SUBMIT_YES.get(),
            ),
            _ => (
                msg::abandon_prompt(&self.target.iid_label),
                msg::ABANDON_YES.get(),
            ),
        };
        // The title is the verb: "Discard review" over a submit is a lie.
        self.confirm = Some((what, super::popups::Confirm::new(yes, prompt, yes)));
    }
}

/// One combobox's mouse: `Some` when it used the event (a list is open, or
/// the press was on its field), with what the combobox decided.
fn drive(
    combo: &mut Combo,
    field: Rect,
    hover: &mut bool,
    mouse: MouseEvent,
) -> Option<ComboOutcome> {
    let on_field = field.contains((mouse.column, mouse.row).into()) && field.width > 0;
    if combo.is_open() {
        return Some(combo.on_mouse(mouse));
    }
    match mouse.kind {
        MouseEventKind::Moved => {
            *hover = on_field;
            None
        }
        MouseEventKind::Down(MouseButton::Left) if on_field => Some(combo.on_mouse(mouse)),
        _ => None,
    }
}
