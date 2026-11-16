//! "What is sent": the dialog shown before an AI request leaves the machine.
//!
//! The request is built and cleaned in the worker (secrets replaced, sensitive
//! files left out: `gitpoulpi_ai::privacy`), and the worker's [`Gate`] asks
//! this dialog before anything is sent. The dialog shows the cleaned system
//! and user text in the shared "What is sent" pane (`sent_pane`), what was
//! replaced or left out (a rule and a masked preview per secret, a path per
//! file: never a value), a "Don't show again for <action>" box that writes
//! `[ai.sent_preview] <action> = false`, and Send / Cancel.
//!
//! **No default for the dangerous direction.** Send is the focused button
//! (it is the action the user asked for), but nothing is sent until it is
//! pressed, and Esc, Cancel, the close cross and a click outside all cancel.
//! Every mouse action has a key: Tab / Shift+Tab move between the pane, the
//! box and the buttons, Space toggles the box, Enter presses a button,
//! PageUp / PageDown / Home / End scroll the pane from anywhere (and the
//! arrows while it has the focus), Alt+W shows or hides it.
//!
//! Process-wide state, like the other small modals: the window restarts on
//! every tab switch and the question comes from a worker thread. A request
//! canceled elsewhere (the jobs list) closes its dialog by itself.

use std::sync::{Arc, Mutex};

use crossterm::event::MouseEventKind;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent};
use gitpoulpi_ai::{Prepared, SendGate};
use gitpoulpi_config::AiAction;
use gitpoulpi_i18n::msg::ai_privacy as msg;
use gitpoulpi_i18n::msg::rewrite_ai as pane_msg;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::Frame;

/// The "what is sent" window resizes: the pane that shows the prompt takes
/// the rows it gains, and its lines wrap to the width it has.
const WIN: crate::dialog_kit::Win = crate::dialog_kit::Win::resizable("ai_sent", 50, 12);

use crate::dialog_kit::{self, ButtonState, CheckState};
use crate::sent_pane::{wrap_width, SentPane, MAX_WIDTH};
use crate::th;
use crate::ui_kit::display_width;

/// The most rows the list of replaced secrets and of left-out files takes
/// each; the rest is counted in the heading.
const LIST_ROWS: usize = 4;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Focus {
    Pane,
    Dont,
    Send,
    Cancel,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Hit {
    Head,
    Pane,
    Bar,
    Dont,
    Send,
    Cancel,
}

struct Open {
    action: AiAction,
    via: String,
    action_label: String,
    sent: Prepared,
    dont_show: bool,
    focus: Focus,
    hover: Option<Hit>,
    pane: SentPane,
    hits: Vec<(Rect, Hit)>,
    frame: Rect,
    close: dialog_kit::CloseHit,
    reply: Option<tokio::sync::oneshot::Sender<bool>>,
}

static POPUP: Mutex<Option<Open>> = Mutex::new(None);
/// Questions that arrived while one was open.
static WAITING: Mutex<Vec<Open>> = Mutex::new(Vec::new());

/// The worker's side: asks the dialog and waits for the answer.
pub(crate) struct Gate {
    action: AiAction,
    via: String,
    action_label: String,
}

impl Gate {
    pub(crate) fn shared(action: AiAction, via: String, action_label: String) -> Arc<dyn SendGate> {
        Arc::new(Self {
            action,
            via,
            action_label,
        })
    }
}

#[async_trait::async_trait]
impl SendGate for Gate {
    async fn confirm(&self, sent: &Prepared) -> bool {
        let (tx, rx) = tokio::sync::oneshot::channel();
        ask(Open {
            action: self.action,
            via: self.via.clone(),
            action_label: self.action_label.clone(),
            sent: sent.clone(),
            dont_show: false,
            focus: Focus::Send,
            hover: None,
            pane: {
                let mut pane = SentPane::default();
                pane.open = true;
                pane
            },
            hits: Vec::new(),
            frame: Rect::default(),
            close: Default::default(),
            reply: Some(tx),
        });
        // A dropped window or a canceled request drops the sender.
        rx.await.unwrap_or(false)
    }
}

fn ask(open: Open) {
    let mut slot = POPUP.lock().unwrap_or_else(|e| e.into_inner());
    if slot.is_none() {
        *slot = Some(open);
    } else {
        WAITING.lock().unwrap_or_else(|e| e.into_inner()).push(open);
    }
    drop(slot);
    crate::redraw::poke();
}

/// Whether the dialog is up. A question whose request was canceled meanwhile
/// is dropped here, so the dialog never outlives what it asks about.
pub(crate) fn is_open() -> bool {
    let Ok(mut slot) = POPUP.lock() else {
        return false;
    };
    while slot
        .as_ref()
        .is_some_and(|o| !o.reply.as_ref().is_some_and(|r| !r.is_closed()))
    {
        finish(&mut slot);
    }
    slot.is_some()
}

/// Closes the open question and brings up the next live one.
fn finish(slot: &mut Option<Open>) {
    let mut waiting = WAITING.lock().unwrap_or_else(|e| e.into_inner());
    waiting.retain(|o| o.reply.as_ref().is_some_and(|r| !r.is_closed()));
    *slot = if waiting.is_empty() {
        None
    } else {
        Some(waiting.remove(0))
    };
}

/// Answers the open question and writes "Don't show again" when it was ticked
/// on a Send.
fn resolve(slot: &mut Option<Open>, send: bool) {
    if let Some(open) = slot.as_mut() {
        if send && open.dont_show {
            remember_off(open.action);
        }
        if let Some(reply) = open.reply.take() {
            let _ = reply.send(send);
        }
    }
    finish(slot);
}

/// `[ai.sent_preview] <action> = false` in `config.toml`.
fn remember_off(action: AiAction) {
    if let Some(path) = gitpoulpi_config::ConfigPaths::discover().map(|p| p.config_file()) {
        remember_off_in(&path, action);
    }
}

fn remember_off_in(path: &std::path::Path, action: AiAction) {
    let _ =
        gitpoulpi_config::set_in_file(path, &format!("ai.sent_preview.{}", action.key()), "false");
}

/// A control character is shown as `\x1b`, never sent to the terminal: the
/// text under review must not be able to repaint the screen it is judged on.
fn visible(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '\t' => out.push_str("    "),
            c if c.is_control() => out.push_str(&format!("\\x{:02x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

/// Cuts `text` into pieces of at most `width` columns, anywhere.
fn wrap_hard(text: &str, width: usize) -> Vec<String> {
    let width = width.max(1);
    let mut out = vec![String::new()];
    let mut used = 0usize;
    for ch in text.chars() {
        let w = display_width(&ch.to_string());
        if used > 0 && used + w > width {
            out.push(String::new());
            used = 0;
        }
        if let Some(last) = out.last_mut() {
            last.push(ch);
        }
        used += w;
    }
    out
}

/// Greedy word wrap on spaces, by display width.
fn wrap_words(text: &str, width: usize) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut line = String::new();
    for word in text.split(' ') {
        if !line.is_empty() && display_width(&line) + 1 + display_width(word) > width {
            out.push(std::mem::take(&mut line));
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(word);
    }
    if !line.is_empty() {
        out.push(line);
    }
    out
}

/// The pane's lines: the system text, then the user text, each under its
/// heading and cut to `width`.
fn pane_lines(sent: &Prepared, width: usize) -> Vec<(String, bool)> {
    let mut lines: Vec<(String, bool)> = Vec::new();
    let section = |lines: &mut Vec<(String, bool)>, heading: &str, text: &str| {
        lines.push((heading.to_string(), true));
        for line in text.split('\n') {
            for piece in wrap_hard(&visible(line), width) {
                lines.push((piece, false));
            }
        }
    };
    section(&mut lines, pane_msg::DLG_SENT_SYSTEM.get(), &sent.system);
    // The repository text that goes after the instructions, when the user let
    // the AI read the repository and GitPoulpi built it.
    if let Some(context) = sent.context.as_ref().filter(|c| !c.text.is_empty()) {
        lines.push((String::new(), false));
        section(&mut lines, msg::CTX_PANE.get(), &context.text);
    }
    lines.push((String::new(), false));
    section(&mut lines, pane_msg::DLG_SENT_USER.get(), &sent.user_text());
    lines
}

impl Open {
    fn step_focus(&mut self, delta: i32) {
        let order = [Focus::Pane, Focus::Dont, Focus::Send, Focus::Cancel];
        let at = order.iter().position(|f| *f == self.focus).unwrap_or(0) as i32;
        self.focus = order[(at + delta).rem_euclid(order.len() as i32) as usize];
    }
}

/// A key while the dialog is open (it swallows them all).
pub(crate) fn on_key(key: KeyEvent) {
    if key.kind != KeyEventKind::Press {
        return;
    }
    let Ok(mut slot) = POPUP.lock() else {
        return;
    };
    let Some(open) = slot.as_mut() else {
        return;
    };
    let plain = !key
        .modifiers
        .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT);
    if key.modifiers.contains(KeyModifiers::ALT)
        && !key.modifiers.contains(KeyModifiers::CONTROL)
        && matches!(key.code, KeyCode::Char('w') | KeyCode::Char('W'))
    {
        open.pane.toggle();
        return;
    }
    match key.code {
        KeyCode::Esc => return resolve(&mut slot, false),
        KeyCode::Tab => return open.step_focus(1),
        KeyCode::BackTab => return open.step_focus(-1),
        KeyCode::PageUp | KeyCode::PageDown | KeyCode::Home | KeyCode::End if open.pane.open => {
            if let Some(step) = open.pane.key_step(key.code) {
                open.pane.scroll_by(step);
            }
            return;
        }
        _ => {}
    }
    match open.focus {
        Focus::Pane => {
            if !open.pane.open {
                if matches!(
                    key.code,
                    KeyCode::Enter | KeyCode::Char(' ') | KeyCode::Right
                ) && plain
                {
                    open.pane.toggle();
                }
                return;
            }
            match key.code {
                KeyCode::Enter | KeyCode::Left => open.pane.toggle(),
                code => {
                    if let Some(step) = open.pane.key_step(code) {
                        open.pane.scroll_by(step);
                    }
                }
            }
        }
        Focus::Dont => match key.code {
            KeyCode::Enter | KeyCode::Char(' ') if plain => open.dont_show = !open.dont_show,
            KeyCode::Down | KeyCode::Right if plain => open.step_focus(1),
            KeyCode::Up | KeyCode::Left if plain => open.step_focus(-1),
            _ => {}
        },
        Focus::Send | Focus::Cancel => match key.code {
            KeyCode::Enter | KeyCode::Char(' ') if plain => {
                let send = open.focus == Focus::Send;
                resolve(&mut slot, send);
            }
            KeyCode::Right | KeyCode::Down if plain => open.step_focus(1),
            KeyCode::Left | KeyCode::Up if plain => open.step_focus(-1),
            _ => {}
        },
    }
}

/// A mouse event while the dialog is open (it swallows them all).
pub(crate) fn on_mouse(mouse: MouseEvent) {
    let Ok(mut slot) = POPUP.lock() else {
        return;
    };
    let Some(open) = slot.as_mut() else {
        return;
    };
    // The pane's bar first: its press, its drag (even off the track) and its
    // release.
    if open.pane.bar_mouse(mouse) {
        open.focus = Focus::Pane;
        return;
    }
    let at = ratatui::layout::Position::new(mouse.column, mouse.row);
    match mouse.kind {
        MouseEventKind::ScrollUp if open.frame.contains(at) && open.pane.open => {
            open.pane.scroll_by(-3)
        }
        MouseEventKind::ScrollDown if open.frame.contains(at) && open.pane.open => {
            open.pane.scroll_by(3)
        }
        MouseEventKind::Moved | MouseEventKind::Drag(_) => {
            open.close.set_hover(mouse.column, mouse.row);
            open.hover = dialog_kit::hit_at(&open.hits, mouse.column, mouse.row);
        }
        MouseEventKind::Down(MouseButton::Left) => {
            if open.close.hit(mouse.column, mouse.row) {
                return resolve(&mut slot, false);
            }
            match dialog_kit::hit_at(&open.hits, mouse.column, mouse.row) {
                Some(Hit::Head) => {
                    open.focus = Focus::Pane;
                    open.pane.toggle();
                }
                Some(Hit::Pane | Hit::Bar) => open.focus = Focus::Pane,
                Some(Hit::Dont) => {
                    open.focus = Focus::Dont;
                    open.dont_show = !open.dont_show;
                }
                Some(Hit::Send) => resolve(&mut slot, true),
                Some(Hit::Cancel) => resolve(&mut slot, false),
                None => {
                    // A click away cancels the request and is consumed.
                    if !open.frame.contains(at) {
                        resolve(&mut slot, false);
                    }
                }
            }
        }
        _ => {}
    }
}

fn tint_spark(buf: &mut Buffer, rect: Rect) {
    for x in rect.x..rect.right() {
        if let Some(cell) = buf.cell_mut((x, rect.y)) {
            if cell.symbol() == crate::theme::AI_SPARK {
                cell.set_fg(th().status.ai);
            }
        }
    }
}

/// Paints the dialog, last, over the panes.
pub(crate) fn render(frame: &mut Frame) {
    let Ok(mut slot) = POPUP.lock() else {
        return;
    };
    let Some(open) = slot.as_mut() else {
        return;
    };
    let area = frame.area();
    let room = Rect::new(
        area.x + 2,
        area.y + 1,
        area.width.saturating_sub(4),
        area.height.saturating_sub(2),
    );
    if room.width < 20 || room.height < 8 {
        let title = format!(
            "{} {}",
            crate::theme::AI_SPARK,
            msg::title(open.via.as_str())
        );
        open.hits.clear();
        open.frame = dialog_kit::too_small::paint_closable(
            frame.buffer_mut(),
            area,
            &title,
            &mut open.close,
        );
        return;
    }
    let width = room.width.min(MAX_WIDTH);
    let wide = width.saturating_sub(4);
    let lines = pane_lines(&open.sent, wrap_width(wide));
    let intro = wrap_words(msg::INTRO.get(), wide as usize);
    let summary = summary_rows(open);
    // Rows that are not the pane: the intro, a gap, the summary, a gap, the
    // pane's head, a gap under the pane, the box, the buttons and the hint.
    let fixed = |summary_rows: usize| 2 + intro.len() + 1 + summary_rows + 1 + 1 + 1 + 3;
    let budget = room.height as usize;
    let shown_summary = if fixed(summary.len()) + 4 <= budget {
        summary.len()
    } else {
        summary.len().min(1)
    };
    let pane_rows = open.pane.body_rows(
        lines.len(),
        budget.saturating_sub(fixed(shown_summary)) as u16,
    );
    let height = (fixed(shown_summary) as u16 + pane_rows).min(room.height);
    let outer = WIN.place(
        room,
        Rect::new(
            room.x + (room.width - width) / 2,
            room.y + (room.height - height) / 2,
            width,
            height,
        ),
    );
    // The text follows the width the window has (the pane's rows follow its
    // height below).
    let wide = outer.width.saturating_sub(4);
    let lines = if outer.width == width {
        lines
    } else {
        pane_lines(&open.sent, wrap_width(wide))
    };
    let intro = if outer.width == width {
        intro
    } else {
        wrap_words(msg::INTRO.get(), wide as usize)
    };
    open.frame = outer;
    open.hits.clear();
    let buf = frame.buffer_mut();
    let title = format!(
        "{} {}",
        crate::theme::AI_SPARK,
        msg::title(open.via.as_str())
    );
    open.close.framed(buf, outer, &title, &WIN, true);
    let inner = Rect::new(
        outer.x + 1,
        outer.y + 1,
        outer.width.saturating_sub(2),
        outer.height.saturating_sub(2),
    );
    let x = inner.x + 1;
    let panel = th().ui.panel;
    let normal = Style::default().fg(th().ui.text).bg(panel);
    let muted = Style::default().fg(th().ui.muted).bg(panel);
    let bold = normal.add_modifier(Modifier::BOLD);
    let mut y = inner.y;
    for line in &intro {
        dialog_kit::put(buf, x, y, line, wide, normal);
        y += 1;
    }
    y += 1;
    for (text, heading) in summary.iter().take(shown_summary) {
        let style = if *heading { bold } else { muted };
        dialog_kit::put(buf, x, y, text, wide, style);
        y += 1;
    }
    y += 1;
    let head = Rect::new(x, y, wide, 1);
    open.pane.paint_head(
        buf,
        head,
        open.focus == Focus::Pane,
        open.hover == Some(Hit::Head),
    );
    open.hits.push((head, Hit::Head));
    y += 1;
    let hint_y = inner.bottom().saturating_sub(1);
    let buttons_y = hint_y.saturating_sub(1);
    let check_y = buttons_y.saturating_sub(1);
    open.pane.paint_none();
    if open.pane.open {
        let pane = Rect::new(x, y, wide, check_y.saturating_sub(1).saturating_sub(y));
        let mut hits = Vec::new();
        open.pane
            .paint_body(buf, pane, &lines, true, &mut hits, (Hit::Bar, Hit::Pane));
        open.hits.extend(hits);
    }
    // The box.
    let r = dialog_kit::checkbox(
        buf,
        x,
        check_y,
        wide,
        &msg::dont_show(open.action_label.as_str()),
        open.dont_show,
        CheckState::Enabled {
            hovered: open.hover == Some(Hit::Dont),
            focused: open.focus == Focus::Dont,
        },
    );
    open.hits
        .push((Rect::new(r.x, r.y, r.width.max(3), 1), Hit::Dont));
    // The buttons.
    let send_label = format!("{} {}", crate::theme::AI_SPARK, msg::SEND.get());
    let send = dialog_kit::button(
        buf,
        x,
        buttons_y,
        &send_label,
        ButtonState::Primary {
            hovered: open.hover == Some(Hit::Send),
            focused: open.focus == Focus::Send,
        },
    );
    tint_spark(buf, send);
    open.hits.push((send, Hit::Send));
    let cancel = dialog_kit::button(
        buf,
        send.right() + 1,
        buttons_y,
        msg::CANCEL.get(),
        ButtonState::Normal {
            hovered: open.hover == Some(Hit::Cancel),
            focused: open.focus == Focus::Cancel,
        },
    );
    open.hits.push((cancel, Hit::Cancel));
    dialog_kit::put(buf, x, hint_y, msg::HINT.get(), wide, muted);
}

/// The rows above the pane: what was replaced and what was left out, or that
/// nothing was. `true` marks a heading.
fn summary_rows(open: &Open) -> Vec<(String, bool)> {
    let sent = &open.sent;
    let mut rows: Vec<(String, bool)> = Vec::new();
    if sent.redactions.is_empty() && sent.excluded_files.is_empty() {
        rows.push((msg::NOTHING_REMOVED.get().to_string(), false));
        context_rows(open, &mut rows);
        return rows;
    }
    if !sent.redactions.is_empty() {
        rows.push((msg::redactions_heading(sent.redactions.len() as u64), true));
        for r in sent.redactions.iter().take(LIST_ROWS) {
            rows.push((
                format!("  {}", msg::redaction_row(&r.rule, &r.preview)),
                false,
            ));
        }
    }
    if !sent.excluded_files.is_empty() {
        rows.push((
            msg::excluded_heading(sent.excluded_files.len() as u64),
            true,
        ));
        for f in sent.excluded_files.iter().take(LIST_ROWS) {
            rows.push((format!("  {}", visible(f)), false));
        }
    }
    context_rows(open, &mut rows);
    rows
}

/// "KB" with one decimal, like the settings rows say sizes.
fn size_text(bytes: usize) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    }
}

/// The repository context of the request: each part with its size and
/// whether it was kept from an earlier draft, the total against the limit,
/// what was cut or left out, or, for a local agent, what it reads with; and
/// the reminder that the AI only drafts.
fn context_rows(open: &Open, rows: &mut Vec<(String, bool)>) {
    let draft_only = matches!(open.action, AiAction::HostingReply | AiAction::IssueBody);
    if let Some(context) = &open.sent.context {
        let summary = &context.summary;
        rows.push((msg::CTX_HEADING.get().to_string(), true));
        if let Some(agent) = &summary.agent {
            rows.push((format!("  {}", msg::ctx_agent(&agent.tools)), false));
            rows.push((
                format!("  {}", msg::ctx_folder(visible(&agent.folder))),
                false,
            ));
            match agent.session.as_str() {
                "started" => rows.push((format!("  {}", msg::CTX_SESSION_STARTED.get()), false)),
                "continued" => {
                    rows.push((format!("  {}", msg::CTX_SESSION_CONTINUED.get()), false))
                }
                _ => {}
            }
        } else {
            rows.push((
                format!(
                    "  {}",
                    msg::ctx_size(size_text(summary.total), size_text(summary.budget))
                ),
                false,
            ));
            for part in summary.parts.iter().take(LIST_ROWS) {
                let label = visible(&part.label);
                let size = size_text(part.bytes);
                let row = if part.reused {
                    msg::ctx_part_reused(label, size)
                } else {
                    msg::ctx_part(label, size)
                };
                rows.push((format!("  {row}"), false));
            }
            let more = summary.parts.len().saturating_sub(LIST_ROWS);
            if more > 0 {
                rows.push((format!("  {}", msg::ctx_more(more as u64)), false));
            }
            for cut in summary.cuts.iter().take(2) {
                rows.push((format!("  {}", msg::ctx_cut(visible(&cut.what))), false));
            }
            if !summary.left_out.is_empty() {
                let files: Vec<String> = summary.left_out.iter().map(|f| visible(f)).collect();
                rows.push((format!("  {}", msg::ctx_left_out(files.join(", "))), false));
            }
        }
    }
    if draft_only {
        rows.push((msg::CTX_DRAFT_ONLY.get().to_string(), false));
    }
}

#[cfg(test)]
mod tests;
