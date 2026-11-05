//! ✦ Write a script with AI (slice S12): the dialog over the Script editor
//! that asks for what the script must do, shows exactly what will be sent,
//! and hands the request to the window.
//!
//! **AI only on an explicit action.** Nothing here talks to a model: the
//! dialog sets a [`AiWish`] the window collects (`take_ai_wish`) and runs
//! through the usual `AiUi`, so the profile, the key and the jobs indicator
//! are the same as for every other ✦ action. The answer comes back through
//! [`RewriteView::put_ai_script`], which puts the script in the editor and
//! marks it "written by AI" until the user changes it. It is never applied,
//! previewed or tested on its own.
//!
//! **What is sent** is on screen before the button: the system prompt (the
//! fixed contract, then the user's own AI instructions), the request, and,
//! only when the box is ticked (off by default), author, date and subject
//! of up to [`SAMPLE_MAX`] commits. Never file contents.
//!
//! While the request runs the dialog stays open and modal (Esc cancels it),
//! so the rule the answer belongs to cannot change under it.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;

use gitpoulpi_i18n::msg::rewrite_ai as t;

use super::bars::BarId;
use super::render::{bad, bold, dim, plain, wrap, RewriteHit};
use super::script::ScriptFocus;
use super::{Notice, RewriteView};
use crate::dialog_kit::{self, button_width, ButtonState, CheckState};
use crate::text_edit::{Caret, KeyEdit};
use crate::text_field::{edit_line, offset_at, EditLook};
use crate::th;

/// How many commits the sample holds at most.
pub(crate) const SAMPLE_MAX: usize = 20;

/// One commit as the sample shows it: who, when, the subject. Nothing from
/// the commit's files or its message body.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct SampleRow {
    pub author_name: String,
    pub author_email: String,
    /// Unix seconds.
    pub timestamp: i64,
    pub subject: String,
}

/// What the dialog asks the window to do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum AiWish {
    /// Work out the system prompt and the profile that would be used.
    Describe,
    /// Send the request.
    Send {
        instruction: String,
        sample: Vec<SampleRow>,
    },
    /// Stop the running request.
    Cancel,
    /// "Create Rules with AI": work out the system prompt and the profile.
    DescribeRules,
    /// "Create Rules with AI": send the description.
    SendRules {
        description: String,
        context: super::RulesCtx,
    },
}

/// The system prompt and where it would go (the window's answer to
/// [`AiWish::Describe`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AiSystem {
    pub prompt: String,
    pub via: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AiFocus {
    Text,
    Sample,
    Sent,
    Send,
    Cancel,
}

/// The open dialog.
pub(crate) struct AiDialog {
    pub text: String,
    pub caret: Caret,
    /// "Send a sample of N commits": off unless the user ticks it.
    pub sample: bool,
    pub focus: AiFocus,
    /// First line of the "what is sent" pane shown.
    pub scroll: usize,
    /// `None` until the window answered; `Err` when AI cannot run.
    pub system: Option<Result<AiSystem, String>>,
    /// The request is out; the dialog waits for it.
    pub working: bool,
    pub note: Option<Notice>,
}

/// All the AI state of the Script editor.
#[derive(Default)]
pub(crate) struct AiState {
    /// `ai.enabled` and a build with AI (set by the window every tick).
    pub enabled: bool,
    /// A request is running (any AI request, not only ours).
    pub busy: bool,
    pub dialog: Option<AiDialog>,
    wish: Option<AiWish>,
    /// The key of the script the AI wrote: the banner shows while the
    /// script on screen is still that one.
    mark: Option<u64>,
    /// The sent-pane rows and the pane's height from the last paint.
    sent_rows: std::cell::Cell<usize>,
    sent_height: std::cell::Cell<usize>,
    text_area: std::cell::Cell<Rect>,
}

impl AiDialog {
    fn new() -> Self {
        Self {
            text: String::new(),
            caret: Caret::default(),
            sample: false,
            focus: AiFocus::Text,
            scroll: 0,
            system: None,
            working: false,
            note: None,
        }
    }
}

/// Paints the sparkle of `rect` in the AI purple (the button's label is one
/// style; the glyph is the only part that carries the color).
fn tint_spark(buf: &mut Buffer, rect: Rect) {
    for x in rect.x..rect.right() {
        if let Some(cell) = buf.cell_mut((x, rect.y)) {
            if cell.symbol() == crate::theme::AI_SPARK {
                cell.set_fg(th().status.ai);
            }
        }
    }
}

impl RewriteView {
    // ------------------------------------------------------ the window's side

    /// The window tells the view whether AI is on and whether a request runs.
    pub(crate) fn set_ai(&mut self, enabled: bool, busy: bool) {
        self.script.ai.enabled = enabled;
        self.script.ai.busy = busy;
        self.rules_ai.enabled = enabled;
        self.rules_ai.busy = busy;
    }

    /// What the dialog wants done, once.
    pub(crate) fn take_ai_wish(&mut self) -> Option<AiWish> {
        self.script
            .ai
            .wish
            .take()
            .or_else(|| self.take_rules_wish())
    }

    /// The window's answer to [`AiWish::Describe`].
    pub(crate) fn set_ai_system(&mut self, system: Result<AiSystem, String>) {
        if let Some(d) = self.script.ai.dialog.as_mut() {
            d.system = Some(system);
        }
    }

    /// The model's script: into the editor, marked, never applied.
    pub(crate) fn put_ai_script(&mut self, script: String, problems: bool) {
        self.script.ai.dialog = None;
        if self.script_text().is_none() {
            return;
        }
        self.set_script_text(script);
        if let Some(ed) = self.script.editor.as_mut() {
            ed.caret = Caret::default();
            ed.focus = ScriptFocus::Editor;
        }
        self.script.ai.mark = self.script_key();
        let (text, error) = if problems {
            (t::DONE_PROBLEMS.get(), true)
        } else {
            (t::DONE.get(), false)
        };
        self.script_note(text.to_string(), error);
    }

    /// The request failed or was canceled: back to the dialog, with why.
    pub(crate) fn ai_stopped(&mut self, why: String) {
        if let Some(d) = self.script.ai.dialog.as_mut() {
            d.working = false;
            d.note = Some(Notice {
                text: why,
                error: true,
            });
        }
    }

    /// The script on screen is the one the AI wrote and nobody changed.
    pub(crate) fn ai_mark_visible(&self) -> bool {
        self.script.ai.mark.is_some() && self.script.ai.mark == self.script_key()
    }

    /// Whether the editor offers the button: AI is on and the script is an
    /// embedded Lua one with text in the rule (not a file, not external).
    pub(super) fn ai_offered(&self) -> bool {
        self.script.ai.enabled
            && self.script_text().is_some()
            && self.script_ref().is_some_and(|s| !s.language.is_external())
    }

    // ----------------------------------------------------------------- open

    /// ✦ "Write with AI...": the dialog. Refused with a notice when AI is
    /// off or another request runs.
    pub(crate) fn open_ai_dialog(&mut self) {
        if !self.script.ai.enabled {
            return self.script_note(t::DLG_OFF.get().to_string(), true);
        }
        if self.script.ai.busy {
            return self.script_note(t::BUSY.get().to_string(), true);
        }
        if !self.ai_offered() {
            return;
        }
        self.script.ai.dialog = Some(AiDialog::new());
        self.script.ai.wish = Some(AiWish::Describe);
    }

    /// The workbench opened by the palette's ✦ row: a new Script rule, its
    /// editor, and the dialog on top.
    pub(crate) fn with_script_ai(self, enabled: bool) -> Self {
        let mut view = self.with_rule(super::RuleTemplate::Script);
        view.script.ai.enabled = enabled;
        view.open_ai_dialog();
        view
    }

    fn close_ai_dialog(&mut self) {
        if self.script.ai.dialog.as_ref().is_some_and(|d| d.working) {
            self.script.ai.wish = Some(AiWish::Cancel);
        }
        self.script.ai.dialog = None;
    }

    // ------------------------------------------------------------- queries

    /// The rows the sample holds: empty unless ticked.
    fn ai_sample(&self) -> Vec<SampleRow> {
        match &self.script.ai.dialog {
            Some(d) if d.sample => self
                .scope_ctx
                .sample
                .iter()
                .take(SAMPLE_MAX)
                .cloned()
                .collect(),
            _ => Vec::new(),
        }
    }

    /// Whether there is anything to put in a sample.
    fn ai_sample_available(&self) -> bool {
        !self.scope_ctx.sample.is_empty()
    }

    /// Why Send is not possible now.
    fn ai_send_blocker(&self) -> Option<String> {
        let d = self.script.ai.dialog.as_ref()?;
        if d.working {
            return Some(String::new());
        }
        if d.text.trim().is_empty() {
            return Some(t::DLG_NEED_TEXT.get().to_string());
        }
        match &d.system {
            Some(Err(why)) => Some(why.clone()),
            Some(Ok(_)) => None,
            None => Some(String::new()),
        }
    }

    /// The script in the editor has work in it the AI's answer would replace.
    fn ai_replaces(&self) -> bool {
        self.script_text().is_some_and(|t| {
            let t = t.trim();
            !t.is_empty()
                && t != super::rules_form::starter_for(
                    gitpoulpi_core::domain::rewrite::ScriptLanguage::Lua,
                )
                .trim()
        })
    }

    fn ai_foci(&self) -> Vec<AiFocus> {
        let mut order = vec![AiFocus::Text];
        if self.ai_sample_available() {
            order.push(AiFocus::Sample);
        }
        order.extend([AiFocus::Sent, AiFocus::Send, AiFocus::Cancel]);
        order
    }

    fn ai_step_focus(&mut self, delta: isize) {
        let order = self.ai_foci();
        let Some(d) = self.script.ai.dialog.as_mut() else {
            return;
        };
        let at = order.iter().position(|f| *f == d.focus).unwrap_or(0) as isize;
        d.focus = order[(at + delta).rem_euclid(order.len() as isize) as usize];
    }

    /// The "what is sent" pane as lines: (text, heading).
    fn ai_sent_lines(&self, width: usize) -> Vec<(String, bool)> {
        let Some(d) = &self.script.ai.dialog else {
            return Vec::new();
        };
        let mut out: Vec<(String, bool)> = vec![(t::DLG_SENT_SYSTEM.get().to_string(), true)];
        if let Some(Ok(system)) = &d.system {
            for line in system.prompt.lines() {
                if line.trim().is_empty() {
                    out.push((String::new(), false));
                } else {
                    out.extend(wrap(line, width).into_iter().map(|l| (l, false)));
                }
            }
        }
        out.push((String::new(), false));
        out.push((t::DLG_SENT_USER.get().to_string(), true));
        let user = crate::ai_ui::rewrite_user_message(d.text.trim(), &self.ai_sample());
        for line in user.lines() {
            if line.trim().is_empty() {
                out.push((String::new(), false));
            } else {
                out.extend(wrap(line, width).into_iter().map(|l| (l, false)));
            }
        }
        out
    }

    // ---------------------------------------------------------------- send

    fn ai_send(&mut self) {
        if let Some(why) = self.ai_send_blocker() {
            if !why.is_empty() {
                if let Some(d) = self.script.ai.dialog.as_mut() {
                    d.note = Some(Notice {
                        text: why,
                        error: true,
                    });
                }
            }
            return;
        }
        let sample = self.ai_sample();
        let Some(d) = self.script.ai.dialog.as_mut() else {
            return;
        };
        d.working = true;
        d.note = None;
        self.script.ai.wish = Some(AiWish::Send {
            instruction: d.text.trim().to_string(),
            sample,
        });
    }

    // ---------------------------------------------------------------- keys

    pub(super) fn on_ai_key(&mut self, key: KeyEvent) {
        let Some(d) = self.script.ai.dialog.as_mut() else {
            return;
        };
        if d.working {
            if key.code == KeyCode::Esc {
                self.script.ai.wish = Some(AiWish::Cancel);
            }
            return;
        }
        let focus = d.focus;
        let plain_key = !key
            .modifiers
            .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT);
        match key.code {
            KeyCode::Esc => return self.close_ai_dialog(),
            KeyCode::Tab => return self.ai_step_focus(1),
            KeyCode::BackTab => return self.ai_step_focus(-1),
            _ => {}
        }
        match focus {
            AiFocus::Text => match key.code {
                KeyCode::Enter => {
                    if let Some(d) = self.script.ai.dialog.as_mut() {
                        d.focus = AiFocus::Send;
                    }
                }
                _ => {
                    if let Some(d) = self.script.ai.dialog.as_mut() {
                        d.note = None;
                        if let KeyEdit::Copy(s) | KeyEdit::Cut(s) =
                            d.caret.on_key(&mut d.text, &key, false)
                        {
                            let msg = crate::clipboard::copy_shared(t::DLG_PROMPT_LABEL.get(), &s);
                            d.note = Some(Notice {
                                text: msg,
                                error: false,
                            });
                        }
                        d.scroll = 0;
                    }
                }
            },
            AiFocus::Sample => match key.code {
                KeyCode::Enter | KeyCode::Char(' ') if plain_key => {
                    if let Some(d) = self.script.ai.dialog.as_mut() {
                        d.sample = !d.sample;
                        d.scroll = 0;
                    }
                }
                KeyCode::Down | KeyCode::Right if plain_key => self.ai_step_focus(1),
                KeyCode::Up | KeyCode::Left if plain_key => self.ai_step_focus(-1),
                _ => {}
            },
            AiFocus::Sent => {
                let page = self.script.ai.sent_height.get().max(2) as isize - 1;
                let step = match key.code {
                    KeyCode::Up | KeyCode::Char('k') => -1,
                    KeyCode::Down | KeyCode::Char('j') => 1,
                    KeyCode::PageUp => -page,
                    KeyCode::PageDown | KeyCode::Char(' ') => page,
                    KeyCode::Home => isize::MIN / 2,
                    KeyCode::End => isize::MAX / 2,
                    KeyCode::Enter => return self.ai_step_focus(1),
                    _ => return,
                };
                self.ai_scroll(step);
            }
            AiFocus::Send | AiFocus::Cancel => match key.code {
                KeyCode::Enter | KeyCode::Char(' ') if plain_key => {
                    if focus == AiFocus::Send {
                        self.ai_send();
                    } else {
                        self.close_ai_dialog();
                    }
                }
                KeyCode::Right | KeyCode::Down if plain_key => self.ai_step_focus(1),
                KeyCode::Left | KeyCode::Up if plain_key => self.ai_step_focus(-1),
                _ => {}
            },
        }
    }

    fn ai_scroll(&mut self, delta: isize) {
        let rows = self.script.ai.sent_rows.get();
        let height = self.script.ai.sent_height.get();
        let max = rows.saturating_sub(height) as isize;
        if let Some(d) = self.script.ai.dialog.as_mut() {
            d.scroll = (d.scroll as isize + delta).clamp(0, max) as usize;
        }
    }

    /// Pasted text goes into the request field.
    pub(super) fn paste_ai(&mut self, pasted: &str) {
        if let Some(d) = self.script.ai.dialog.as_mut() {
            if !d.working && d.focus == AiFocus::Text {
                let one = pasted.replace(['\r', '\n'], " ");
                d.caret.insert(&mut d.text, &one);
                d.note = None;
            }
        }
    }

    pub(crate) fn on_ai_wheel(&mut self, delta: isize) {
        self.ai_scroll(delta);
    }

    // --------------------------------------------------------------- mouse

    pub(super) fn on_ai_press(&mut self, hit: RewriteHit, col: u16) {
        let working = self.script.ai.dialog.as_ref().is_some_and(|d| d.working);
        match hit {
            RewriteHit::AiClose | RewriteHit::AiShield | RewriteHit::AiCancel => {
                self.close_ai_dialog()
            }
            RewriteHit::AiPanel => {}
            _ if working => {}
            RewriteHit::AiText => {
                let area = self.script.ai.text_area.get();
                if let Some(d) = self.script.ai.dialog.as_mut() {
                    d.focus = AiFocus::Text;
                    let at = offset_at(
                        &d.text,
                        &d.caret,
                        area.width as usize,
                        false,
                        col.saturating_sub(area.x) as usize,
                        0,
                    );
                    let text = d.text.clone();
                    d.caret.set_cursor(&text, at, false);
                }
            }
            RewriteHit::AiSample => {
                if let Some(d) = self.script.ai.dialog.as_mut() {
                    d.focus = AiFocus::Sample;
                    d.sample = !d.sample;
                    d.scroll = 0;
                }
            }
            RewriteHit::AiSent => {
                if let Some(d) = self.script.ai.dialog.as_mut() {
                    d.focus = AiFocus::Sent;
                }
            }
            RewriteHit::AiSend => {
                if let Some(d) = self.script.ai.dialog.as_mut() {
                    d.focus = AiFocus::Send;
                }
                self.ai_send();
            }
            _ => {}
        }
    }

    // --------------------------------------------------------------- paint

    /// "✦ written by AI: read it before Preview", for the editor's title,
    /// while the script on screen is the one the AI wrote.
    pub(super) fn ai_title_suffix(&self) -> Option<String> {
        self.ai_mark_visible()
            .then(|| format!("{} {}", crate::theme::AI_SPARK, t::BANNER.get()))
    }

    /// The mark's sparkle in the AI purple (the title is one style).
    pub(super) fn tint_title_spark(&self, buf: &mut Buffer, rect: Rect) {
        if self.ai_mark_visible() {
            tint_spark(buf, Rect::new(rect.x, rect.y, rect.width, 1));
        }
    }

    /// The editor's ✦ button: the sparkle in purple, the label after it.
    pub(super) fn paint_ai_button(
        &self,
        buf: &mut Buffer,
        x: u16,
        y: u16,
        focused: bool,
        hovered: bool,
    ) -> Rect {
        let label = format!("{} {}", crate::theme::AI_SPARK, t::BTN_WRITE.get());
        let state = if self.script.ai.busy {
            ButtonState::Disabled
        } else {
            ButtonState::Normal { hovered, focused }
        };
        let r = dialog_kit::button(buf, x, y, &label, state);
        if !self.script.ai.busy {
            tint_spark(buf, r);
        }
        r
    }

    /// The width the ✦ button takes (for the row's layout).
    pub(super) fn ai_button_width() -> u16 {
        button_width(&format!(
            "{} {}",
            crate::theme::AI_SPARK,
            t::BTN_WRITE.get()
        ))
    }

    /// Paints the dialog and returns its click targets, inner-first.
    pub(super) fn paint_ai(
        &self,
        buf: &mut Buffer,
        bounds: Rect,
        hovered: Option<RewriteHit>,
    ) -> Vec<(Rect, RewriteHit)> {
        let mut hits = Vec::new();
        let Some(d) = &self.script.ai.dialog else {
            return hits;
        };
        let rect = Rect::new(
            bounds.x + 4,
            bounds.y + 2,
            bounds.width.saturating_sub(8),
            bounds.height.saturating_sub(4),
        );
        let title = format!("{} {}", crate::theme::AI_SPARK, t::DLG_TITLE.get());
        dialog_kit::panel(buf, rect, &title);
        if let Some(cross) =
            crate::chrome::close_cross::paint(buf, rect, hovered == Some(RewriteHit::AiClose))
        {
            hits.push((cross, RewriteHit::AiClose));
        }
        let inner = Rect::new(
            rect.x + 1,
            rect.y + 1,
            rect.width.saturating_sub(2),
            rect.height.saturating_sub(2),
        );
        if inner.width < 40 || inner.height < 12 {
            dialog_kit::put(
                buf,
                inner.x + 1,
                inner.y,
                t::DLG_TOO_SMALL.get(),
                inner.width.saturating_sub(1),
                dim(),
            );
            hits.push((rect, RewriteHit::AiPanel));
            hits.push((bounds, RewriteHit::AiShield));
            return hits;
        }
        let wide = inner.width.saturating_sub(2);
        let mut y = inner.y;

        // The request.
        dialog_kit::put(
            buf,
            inner.x + 1,
            y,
            t::DLG_PROMPT_LABEL.get(),
            wide,
            bold(d.focus == AiFocus::Text),
        );
        y += 1;
        let field = Rect::new(inner.x + 1, y, wide, 1);
        let focus = d.focus == AiFocus::Text && !d.working;
        let hit = RewriteHit::AiText;
        let bg = crate::input_bg(focus, hovered == Some(hit));
        dialog_kit::fill(buf, field, Style::default().bg(bg));
        let body = Rect::new(field.x + 1, y, field.width.saturating_sub(2), 1);
        self.script.ai.text_area.set(body);
        if d.text.is_empty() && !focus {
            dialog_kit::put(
                buf,
                body.x,
                y,
                t::DLG_EXAMPLE.get(),
                body.width,
                Style::default().fg(th().ui.muted).bg(bg),
            );
        } else {
            let line = edit_line(
                &d.text,
                Some(&d.caret),
                body.width.saturating_sub(1) as usize,
                focus,
                EditLook::new(Style::default().fg(th().ui.text).bg(bg), "\u{2588}"),
            );
            buf.set_line(body.x, y, &line, body.width);
        }
        hits.push((field, hit));
        y += 2;

        // The sample.
        let available = self.ai_sample_available();
        let n = self.scope_ctx.sample.len().min(SAMPLE_MAX) as u64;
        let label = if available {
            t::dlg_sample(n)
        } else {
            t::DLG_SAMPLE_NONE.get().to_string()
        };
        let state = if available && !d.working {
            CheckState::Enabled {
                hovered: hovered == Some(RewriteHit::AiSample),
                focused: d.focus == AiFocus::Sample,
            }
        } else {
            CheckState::Disabled
        };
        let r = dialog_kit::checkbox(buf, inner.x + 1, y, wide, &label, d.sample, state);
        if available {
            hits.push((Rect::new(r.x, r.y, r.width.max(3), 1), RewriteHit::AiSample));
        }
        y += 1;
        if !d.sample {
            dialog_kit::put(
                buf,
                inner.x + 4,
                y,
                t::DLG_SAMPLE_OFF.get(),
                wide - 3,
                dim(),
            );
        }
        y += 1;
        let mut facts = Vec::new();
        if let Some(Ok(system)) = &d.system {
            facts.push(t::dlg_profile(&system.via));
        }
        if self.ai_replaces() {
            facts.push(t::DLG_REPLACES.get().to_string());
        }
        dialog_kit::put(buf, inner.x + 1, y, &facts.join("  "), wide, dim());
        y += 2;

        // What is sent.
        dialog_kit::put(
            buf,
            inner.x + 1,
            y,
            t::DLG_SENT_LABEL.get(),
            wide,
            bold(d.focus == AiFocus::Sent),
        );
        y += 1;
        let buttons_y = inner.bottom().saturating_sub(1);
        let note_y = buttons_y.saturating_sub(1);
        let pane_h = note_y.saturating_sub(y) as usize;
        let lines = self.ai_sent_lines(wide.saturating_sub(2) as usize);
        self.script.ai.sent_rows.set(lines.len());
        self.script.ai.sent_height.set(pane_h);
        let top = d.scroll.min(lines.len().saturating_sub(pane_h));
        let pane = Rect::new(inner.x + 1, y, wide, pane_h as u16);
        for (i, (text, heading)) in lines.iter().skip(top).take(pane_h).enumerate() {
            let style = if *heading { bold(false) } else { plain() };
            dialog_kit::put(buf, pane.x + 1, y + i as u16, text, pane.width - 1, style);
        }
        // The bar sits on the panel's border, beside the pane.
        self.paint_bar_on(
            buf,
            &mut hits,
            hovered == Some(RewriteHit::Bar(BarId::AiSent)),
            BarId::AiSent,
            (inner.right(), y, pane_h as u16),
            lines.len(),
            pane_h,
            top,
            None,
        );
        hits.push((pane, RewriteHit::AiSent));

        // The note: what went wrong, or the wait.
        if d.working {
            let line = format!(
                "{} {}",
                crate::theme::AI_SPARK,
                gitpoulpi_i18n::msg::ai_ui::working_start(t::DLG_TITLE.get())
            );
            let r = dialog_kit::put(buf, inner.x + 1, note_y, &line, wide, plain());
            tint_spark(buf, r);
        } else if let Some(note) = &d.note {
            let style = if note.error { bad() } else { plain() };
            dialog_kit::put(buf, inner.x + 1, note_y, &note.text, wide, style);
        } else if let Some(why) = self.ai_send_blocker().filter(|w| !w.is_empty()) {
            dialog_kit::put(buf, inner.x + 1, note_y, &why, wide, dim());
        }

        // The buttons.
        let send_label = format!("{} {}", crate::theme::AI_SPARK, t::DLG_SEND.get());
        let blocked = self.ai_send_blocker().is_some();
        let state = if blocked {
            ButtonState::Disabled
        } else {
            ButtonState::Primary {
                hovered: hovered == Some(RewriteHit::AiSend),
                focused: d.focus == AiFocus::Send,
            }
        };
        let r = dialog_kit::button(buf, inner.x + 1, buttons_y, &send_label, state);
        if !blocked {
            tint_spark(buf, r);
        }
        hits.push((r, RewriteHit::AiSend));
        let cancel = dialog_kit::button(
            buf,
            r.right() + 1,
            buttons_y,
            t::DLG_CANCEL.get(),
            ButtonState::Normal {
                hovered: hovered == Some(RewriteHit::AiCancel),
                focused: d.focus == AiFocus::Cancel,
            },
        );
        hits.push((cancel, RewriteHit::AiCancel));
        hits.push((rect, RewriteHit::AiPanel));
        hits.push((bounds, RewriteHit::AiShield));
        hits
    }
}
