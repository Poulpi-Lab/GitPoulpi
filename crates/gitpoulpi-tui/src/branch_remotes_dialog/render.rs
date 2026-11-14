//! Painting the table of "Remotes of <branch>", handing back its hit rects
//! from the same walk (inner-first: a checkbox or the Remove button before
//! the row band that covers it).

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};

use super::{RemoteRelation, RemotesPane, Row};
use crate::dialog_kit::{self, button, checkbox, put, ButtonState, CheckState};
use crate::prompt_dialog::PromptPart;
use crate::theme::th;
use crate::ui_kit::{display_width, truncate_with_ellipsis};
use gitpoulpi_i18n::msg::branch_remotes as msg;

/// The widest a remote or branch column grows before it truncates.
const NAME_MAX: usize = 18;
const GAP: u16 = 2;

/// The state cell of a row: its text and what colour says about it.
fn state_cell(row: &Row) -> (String, Color) {
    let relation = match row.relation {
        Some(r) if !row.gone => r,
        _ => return (msg::STATE_NONE.get().to_string(), th().ui.muted),
    };
    match relation {
        RemoteRelation::UpToDate => (msg::STATE_IN_SYNC.get().to_string(), th().status.success),
        RemoteRelation::LocalAhead(n) => (format!("\u{2191}{n}"), th().status.warning),
        RemoteRelation::RemoteAhead(n) => (format!("\u{2193}{n}"), th().status.warning),
        RemoteRelation::Diverged { ahead, behind } => {
            (format!("\u{2191}{ahead} \u{2193}{behind}"), th().status.danger)
        }
        RemoteRelation::Absent => (msg::STATE_NEW.get().to_string(), th().ui.muted),
        RemoteRelation::Unrelated => (msg::STATE_UNRELATED.get().to_string(), th().status.danger),
    }
}

impl RemotesPane {
    /// Rows of the table the dialog wants (all of them, the dialog caps it).
    pub(crate) fn wanted_rows(&self) -> usize {
        self.rows.len().max(1)
    }

    /// The headings and the visible rows inside `rect` (`rect.height - 1`
    /// rows fit). `focused`: the table has the keyboard.
    pub(crate) fn paint(
        &self,
        buf: &mut Buffer,
        rect: Rect,
        focused: bool,
        hovered: Option<PromptPart>,
    ) -> Vec<(Rect, PromptPart)> {
        let mut hits = Vec::new();
        self.bar.set(None);
        let panel = th().ui.panel;
        let muted = Style::default().fg(th().ui.muted).bg(panel);
        if rect.height < 2 || rect.width < 24 {
            return hits;
        }
        if !self.is_ready() {
            let text = match self.failure() {
                Some(why) => (why.to_string(), th().status.danger),
                None => (msg::LOADING.get().to_string(), th().ui.muted),
            };
            put(
                buf,
                rect.x,
                rect.y + 1,
                &truncate_with_ellipsis(&text.0, rect.width as usize),
                rect.width,
                Style::default().fg(text.1).bg(panel),
            );
            return hits;
        }
        let view = (rect.height - 1) as usize;
        self.viewport.set(view);
        let n = self.rows.len();
        let top = self.top.get().min(n.saturating_sub(view));
        self.top.set(top);
        let bar_w = u16::from(n > view);
        let width = rect.width - bar_w;

        // Column widths: the heading or the longest cell, capped.
        let widest = |f: &dyn Fn(&Row) -> usize, head: &str| -> u16 {
            self.rows
                .iter()
                .map(f)
                .chain([display_width(head)])
                .max()
                .unwrap_or(0)
                .min(NAME_MAX) as u16
        };
        let mut rw = widest(&|r| display_width(&r.remote), &msg::COL_REMOTE.get());
        let mut bw = widest(
            &|r| display_width(self.dest_shown(r)),
            &msg::COL_BRANCH.get(),
        );
        let pw = (display_width(&msg::COL_PUSH.get()) as u16).max(4);
        let lw = (display_width(&msg::COL_PULL.get()) as u16)
            .max(display_width(&msg::PULL_UPSTREAM.get()) as u16)
            .max(4);
        let fixed = |rw: u16, bw: u16| 2 + rw + GAP + bw + GAP + pw + GAP + lw + GAP;
        let state_min = 10u16;
        while fixed(rw, bw) + state_min > width && (rw > 6 || bw > 6) {
            if bw >= rw && bw > 6 {
                bw -= 1;
            } else if rw > 6 {
                rw -= 1;
            }
        }
        let x_remote = rect.x + 2;
        let x_branch = x_remote + rw + GAP;
        let x_push = x_branch + bw + GAP;
        let x_pull = x_push + pw + GAP;
        let x_state = x_pull + lw + GAP;
        let state_w = (rect.x + width).saturating_sub(x_state);

        // Headings.
        let mut head = |x: u16, w: u16, text: &str| {
            put(buf, x, rect.y, &truncate_with_ellipsis(text, w as usize), w, muted);
        };
        head(x_remote, rw, &msg::COL_REMOTE.get());
        head(x_branch, bw, &msg::COL_BRANCH.get());
        head(x_push, pw, &msg::COL_PUSH.get());
        head(x_pull, lw, &msg::COL_PULL.get());
        head(x_state, state_w, &msg::COL_STATE.get());

        let hovered_at = |part: PromptPart| hovered == Some(part);
        // Visible rows by index (no list window helper: the bar below owns the offset).
        let last = (top + view).min(n);
        for i in top..last {
            let row = &self.rows[i];
            let y = rect.y + 1 + (i - top) as u16;
            let band = Rect::new(rect.x, y, width, 1);
            let selected = focused && i == self.cursor;
            let idx = i.min(u8::MAX as usize) as u8;
            dialog_kit::row_band(
                buf,
                band,
                dialog_kit::row_bg(selected, hovered_at(PromptPart::Row(idx))),
            );
            let bg = dialog_kit::row_bg(selected, hovered_at(PromptPart::Row(idx)));
            let text = Style::default().fg(th().ui.text).bg(bg);
            let dim = Style::default().fg(th().ui.muted).bg(bg);
            if i == self.cursor {
                let color = if focused { th().ui.accent } else { th().ui.muted };
                put(buf, rect.x, y, "\u{258C}", 1, Style::default().fg(color).bg(bg));
            }
            let primary = self.is_primary(row);
            let name_style = if row.gone {
                Style::default().fg(th().status.warning).bg(bg)
            } else {
                text
            };
            put(
                buf,
                x_remote,
                y,
                &truncate_with_ellipsis(&row.remote, rw as usize),
                rw,
                name_style,
            );
            if row.gone {
                put(
                    buf,
                    x_push,
                    y,
                    &truncate_with_ellipsis(
                        &msg::GONE.get(),
                        (x_state + state_w).saturating_sub(x_push) as usize,
                    ),
                    (x_state + state_w).saturating_sub(x_push),
                    Style::default().fg(th().status.warning).bg(bg),
                );
                let label = msg::REMOVE_BUTTON.get();
                let w = dialog_kit::button_width(&label);
                if width >= w + 2 {
                    let rect_b = button(
                        buf,
                        rect.x + width - w,
                        y,
                        &label,
                        ButtonState::Normal {
                            hovered: hovered_at(PromptPart::Remove(idx)),
                            focused: false,
                        },
                    );
                    hits.push((rect_b, PromptPart::Remove(idx)));
                }
                hits.push((band, PromptPart::Row(idx)));
                continue;
            }
            put(
                buf,
                x_branch,
                y,
                &truncate_with_ellipsis(self.dest_shown(row), bw as usize),
                bw,
                if primary { dim } else { text },
            );
            // Push: the upstream is always pushed, so its box is inert.
            let push_state = if primary {
                CheckState::Disabled
            } else {
                CheckState::Enabled {
                    hovered: hovered_at(PromptPart::Push(idx)),
                    focused: false,
                }
            };
            let push_rect = checkbox(buf, x_push, y, pw, "", row.push || primary, push_state);
            if !primary {
                hits.push((push_rect, PromptPart::Push(idx)));
            }
            if primary {
                put(buf, x_pull, y, &msg::PULL_UPSTREAM.get(), lw, dim);
            } else {
                let pull_rect = checkbox(
                    buf,
                    x_pull,
                    y,
                    lw,
                    "",
                    row.pull,
                    CheckState::Enabled {
                        hovered: hovered_at(PromptPart::Pull(idx)),
                        focused: false,
                    },
                );
                hits.push((pull_rect, PromptPart::Pull(idx)));
            }
            let (state, color) = state_cell(row);
            put(
                buf,
                x_state,
                y,
                &truncate_with_ellipsis(&state, state_w as usize),
                state_w,
                Style::default().fg(color).bg(bg),
            );
            hits.push((band, PromptPart::Row(idx)));
        }
        if bar_w > 0 {
            let column = Rect::new(rect.right() - 1, rect.y + 1, 1, view as u16);
            if let Some(track) = crate::scrollbar::Track::column(column, n, view, top) {
                crate::scrollbar::render_buf(buf, track, true, false);
                self.bar.set(Some(track));
            }
        }
        hits
    }
}
