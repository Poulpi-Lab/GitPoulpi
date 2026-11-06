//! The update panel: what the title bar's version chip opens (the
//! install-channels design). A modal window with the running version, how
//! GitPoulpi was installed, where the update stands, and the buttons of the
//! install channel's procedure:
//!
//! - `Swap` (archive, the Inno installers): **Update and restart**.
//! - `Run` (Chocolatey, winget, Scoop, Homebrew): **Run update** and the
//!   command it runs.
//! - `Show` (apt, AUR, cargo): the exact command with **Copy command** and
//!   **Open in terminal**; the app never runs these itself.
//! - `Announce` (managed installs, containers, a folder this user cannot
//!   write, an administrator's policy): the reason, the administrator's
//!   message, the commands worth knowing.
//!
//! Every button has a key (listed in the hint line) and a palette command
//! (`Updates: ...`); the chip's click and `Ctrl+N` `u` open the panel.
//! Modal while open: `Esc`, `q`, the close cross or a click outside closes it
//! and nothing behind it hovers. The body scrolls with the wheel, the keys
//! and a draggable bar. Hit rects come out of the walk that paints
//! (`render`), registered inner-first (the cross and the buttons; the window
//! itself is only the "outside" test).
//!
//! Process-wide state, because the window restarts on every tab switch.
//! Nothing here blocks: the actions start threads (`auto`, `terminal`).

use std::sync::Mutex;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::Frame;

use gitpoulpi_config::UpdateMode;
use gitpoulpi_i18n::msg::updates as msg;
use gitpoulpi_update::plan::{
    display_command, AnnounceReason, Elevation, NothingReason, Procedure,
};

use super::auto::Phase;
use super::{Snapshot, UpdatesCommand};
use crate::dialog_kit::{self, ButtonState};
use crate::scrollbar::{BarDrag, Track};
use crate::th;

/// A button of the panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Act {
    /// Update and restart (a swap), or run the package manager.
    Primary,
    Copy,
    Terminal,
    Revert,
    /// The release page in the browser.
    Download,
    Notes,
    Skip,
    Check,
    Close,
}

/// What the event loop does after the panel handled an event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Event {
    None,
    /// A line for the status bar.
    Message(String),
    /// Open the Release notes tab (the panel has closed).
    ReleaseNotes,
}

#[derive(Default)]
struct State {
    open: bool,
    window: Rect,
    close: Rect,
    hits: Vec<(Rect, Act)>,
    /// The scrolling body as last painted.
    body: Rect,
    /// The bordered stand-in the bar is drawn over.
    bar_body: Rect,
    lines: usize,
    offset: usize,
    bar: BarDrag,
    pointer: Option<(u16, u16)>,
    notice: Option<String>,
}

static STATE: Mutex<Option<State>> = Mutex::new(None);

fn with<R>(f: impl FnOnce(&mut State) -> R) -> R {
    let mut guard = STATE.lock().unwrap_or_else(|e| e.into_inner());
    f(guard.get_or_insert_with(State::default))
}

pub(crate) fn is_open() -> bool {
    with(|s| s.open)
}

pub(crate) fn open() {
    crate::scrollbar::set_pointer(None);
    with(|s| {
        *s = State {
            open: true,
            ..State::default()
        }
    });
    crate::redraw::poke();
}

pub(crate) fn close() {
    with(|s| *s = State::default());
    crate::redraw::poke();
}

// ---- what the procedure offers (pure) ----

/// The commands a procedure shows: `Show` and `Announce` list theirs, `Run`
/// shows what it runs.
pub(crate) fn commands(procedure: &Procedure) -> Vec<String> {
    match procedure {
        Procedure::Show { commands } | Procedure::Announce { commands, .. } => commands.clone(),
        Procedure::Run { argv, .. } => vec![display_command(argv)],
        Procedure::Swap { .. } | Procedure::Nothing(_) => Vec::new(),
    }
}

/// The buttons for `snap`, main one first: `(action, label)`.
pub(crate) fn actions(snap: &Snapshot) -> Vec<(Act, String)> {
    let install = &snap.install;
    let known = snap.available.is_some() || snap.skipped.is_some();
    let off = snap.mode == UpdateMode::Off;
    let busy = matches!(install.phase, Phase::Downloading(_) | Phase::Installing(_));
    let mut out: Vec<(Act, String)> = Vec::new();
    match &install.procedure {
        Procedure::Swap { .. } if install.updatable && !off && !busy => {
            if known || matches!(install.phase, Phase::Ready(_)) {
                out.push((Act::Primary, msg::BTN_UPDATE_RESTART.get().to_string()));
            }
        }
        Procedure::Run { .. } if known && !off && !busy => {
            out.push((Act::Primary, msg::BTN_RUN_UPDATE.get().to_string()));
        }
        _ => {}
    }
    if known && !off {
        match &install.procedure {
            Procedure::Show { commands } if !commands.is_empty() => {
                out.push((Act::Copy, msg::BTN_COPY.get().to_string()));
                out.push((Act::Terminal, msg::BTN_TERMINAL.get().to_string()));
            }
            Procedure::Run { .. } => out.push((Act::Copy, msg::BTN_COPY.get().to_string())),
            Procedure::Announce { commands, .. } if !commands.is_empty() => {
                out.push((Act::Copy, msg::BTN_COPY.get().to_string()));
            }
            _ => {}
        }
    }
    if let Some(previous) = &install.revert_to {
        if !busy {
            out.push((Act::Revert, msg::btn_revert(previous)));
        }
    }
    if known
        && !off
        && !matches!(
            install.procedure,
            Procedure::Swap { .. } | Procedure::Run { .. }
        )
    {
        out.push((Act::Download, msg::BTN_DOWNLOAD.get().to_string()));
    }
    out.push((Act::Notes, msg::BTN_NOTES.get().to_string()));
    if snap.available.is_some() && !off {
        out.push((Act::Skip, msg::BTN_SKIP.get().to_string()));
    }
    if !off {
        out.push((Act::Check, msg::BTN_CHECK.get().to_string()));
    }
    out.push((Act::Close, msg::BTN_CLOSE.get().to_string()));
    out
}

/// How a body row looks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Look {
    Plain,
    Muted,
    Heading,
    Accent,
    Danger,
    Command,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Row {
    pub(crate) text: String,
    pub(crate) look: Look,
}

fn push_wrapped(rows: &mut Vec<Row>, text: &str, look: Look, width: usize, indent: &str) {
    let room = width.saturating_sub(indent.chars().count()).max(8);
    for line in crate::ui_kit::wrap_words(text, room) {
        rows.push(Row {
            text: format!("{indent}{line}"),
            look,
        });
    }
}

fn blank(rows: &mut Vec<Row>) {
    rows.push(Row {
        text: String::new(),
        look: Look::Plain,
    });
}

/// Where the update stands, as one line and its look.
fn status(snap: &Snapshot, progress: Option<(u64, Option<u64>)>) -> (String, Look) {
    let install = &snap.install;
    match &install.phase {
        Phase::Downloading(version) => (
            match progress {
                Some((done, Some(total))) if total > 0 => {
                    msg::head_downloading(version, done * 100 / total)
                }
                _ => msg::head_downloading_unknown(version),
            },
            Look::Accent,
        ),
        Phase::Installing(version) => (msg::head_installing(version), Look::Accent),
        Phase::Ready(version) => (msg::head_ready(version), Look::Accent),
        Phase::Failed(reason) => (msg::head_failed(reason), Look::Danger),
        Phase::Idle => {
            if snap.mode == UpdateMode::Off {
                (msg::HEAD_OFF.get().to_string(), Look::Muted)
            } else if snap.checking {
                (msg::HEAD_CHECKING.get().to_string(), Look::Muted)
            } else if let Some(r) = &snap.available {
                (msg::head_available(&r.version), Look::Accent)
            } else if let Some(r) = &snap.skipped {
                (msg::head_skipped(&r.version), Look::Plain)
            } else {
                (msg::head_current(super::RUNNING), Look::Plain)
            }
        }
    }
}

fn announce_text(reason: AnnounceReason) -> &'static str {
    match reason {
        AnnounceReason::PolicyBlocksSelfUpdate => msg::ANNOUNCE_POLICY.get(),
        AnnounceReason::Managed => msg::ANNOUNCE_MANAGED.get(),
        AnnounceReason::Container => msg::ANNOUNCE_CONTAINER.get(),
        AnnounceReason::LegacyReadOnly => msg::ANNOUNCE_LEGACY_READ_ONLY.get(),
        AnnounceReason::AptWithoutRepository => msg::ANNOUNCE_APT.get(),
        AnnounceReason::NoRevert => msg::ANNOUNCE_NO_REVERT.get(),
    }
}

/// The body for `snap`, wrapped to `width` cells.
pub(crate) fn content(
    snap: &Snapshot,
    width: usize,
    progress: Option<(u64, Option<u64>)>,
) -> Vec<Row> {
    let install = &snap.install;
    let known = snap.available.is_some() || snap.skipped.is_some();
    let mut rows = Vec::new();
    rows.push(Row {
        text: msg::panel_version(super::running_label()),
        look: Look::Heading,
    });
    let via = match install.channel {
        Some(channel) => msg::panel_installed_with(super::plans::channel_label(channel)),
        None => msg::PANEL_DETECTING.get().to_string(),
    };
    push_wrapped(&mut rows, &via, Look::Muted, width, "");
    blank(&mut rows);
    let (line, look) = status(snap, progress);
    push_wrapped(&mut rows, &line, look, width, "");
    if let Some(message) = &install.message {
        blank(&mut rows);
        push_wrapped(
            &mut rows,
            &msg::panel_admin_message(message),
            Look::Plain,
            width,
            "",
        );
    }
    let off = snap.mode == UpdateMode::Off;
    let commands_now = |rows: &mut Vec<Row>, list: &[String]| {
        for command in list {
            push_wrapped(rows, command, Look::Command, width, "  ");
        }
    };
    match &install.procedure {
        Procedure::Nothing(NothingReason::NoInstallRecord) => {
            blank(&mut rows);
            push_wrapped(
                &mut rows,
                msg::PANEL_NO_RECORD.get(),
                Look::Muted,
                width,
                "",
            );
        }
        Procedure::Nothing(NothingReason::ModeOff) => {}
        _ if off => {}
        Procedure::Swap { elevation, .. } => {
            if known || matches!(install.phase, Phase::Ready(_)) {
                blank(&mut rows);
                let note = if cfg!(windows) {
                    msg::PANEL_NOTE_SWAP_WINDOWS.get()
                } else {
                    msg::PANEL_NOTE_SWAP.get()
                };
                push_wrapped(&mut rows, note, Look::Plain, width, "");
                if *elevation == Elevation::Uac {
                    push_wrapped(&mut rows, msg::PANEL_NOTE_UAC.get(), Look::Muted, width, "");
                }
            }
        }
        Procedure::Run { argv, elevation } => {
            if known {
                blank(&mut rows);
                let note = if cfg!(windows) {
                    msg::PANEL_NOTE_RUN_WINDOWS.get()
                } else {
                    msg::PANEL_NOTE_RUN.get()
                };
                push_wrapped(&mut rows, note, Look::Plain, width, "");
                push_wrapped(
                    &mut rows,
                    msg::PANEL_RUN_COMMAND.get(),
                    Look::Muted,
                    width,
                    "",
                );
                commands_now(&mut rows, &[display_command(argv)]);
                if *elevation == Elevation::Uac {
                    push_wrapped(&mut rows, msg::PANEL_NOTE_UAC.get(), Look::Muted, width, "");
                }
            }
        }
        Procedure::Show { commands } => {
            if known {
                blank(&mut rows);
                push_wrapped(
                    &mut rows,
                    msg::PANEL_SHOW_INTRO.get(),
                    Look::Plain,
                    width,
                    "",
                );
                commands_now(&mut rows, commands);
            }
        }
        Procedure::Announce {
            reason, commands, ..
        } => {
            blank(&mut rows);
            push_wrapped(&mut rows, announce_text(*reason), Look::Plain, width, "");
            if known {
                commands_now(&mut rows, commands);
            }
        }
    }
    rows
}

// ---- actions ----

/// `Updates: Copy update command`.
pub(crate) fn copy_command() -> Option<String> {
    let commands = commands(&super::snapshot().install.procedure);
    if commands.is_empty() {
        return Some(msg::WHY_NO_COMMAND.get().to_string());
    }
    Some(crate::clipboard::copy_shared(
        msg::COPY_WHAT.get(),
        &commands.join("\n"),
    ))
}

/// `Updates: Run update command in a terminal`: only for a `Show` procedure.
pub(crate) fn run_in_terminal() -> Option<String> {
    let procedure = super::snapshot().install.procedure;
    let Procedure::Show { commands } = procedure else {
        return Some(msg::WHY_NO_TERMINAL.get().to_string());
    };
    if commands.is_empty() {
        return Some(msg::WHY_NO_TERMINAL.get().to_string());
    }
    super::terminal::open(commands.join(" && "));
    None
}

fn perform(act: Act) -> Event {
    let said = |line: Option<String>| line.map_or(Event::None, Event::Message);
    match act {
        Act::Primary => said(super::auto::update_and_restart()),
        Act::Copy => said(copy_command()),
        Act::Terminal => said(run_in_terminal()),
        Act::Revert => said(super::auto::revert_now()),
        Act::Download => said(super::run(UpdatesCommand::Download)),
        Act::Skip => said(super::skip()),
        Act::Check => said(super::check_now()),
        Act::Notes => {
            close();
            Event::ReleaseNotes
        }
        Act::Close => {
            close();
            Event::None
        }
    }
}

/// Runs an action and shows its answer on the panel too (the status line is
/// under the dialog's dim, so the panel keeps its own notice row).
fn act(act: Act) -> Event {
    let event = perform(act);
    if let Event::Message(line) = &event {
        with(|s| s.notice = Some(line.clone()));
    }
    event
}

// ---- keys ----

fn key_action(key: &KeyEvent) -> Option<Act> {
    if key
        .modifiers
        .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT)
    {
        return None;
    }
    Some(match key.code {
        KeyCode::Enter | KeyCode::Char('u') => Act::Primary,
        KeyCode::Char('c') => Act::Copy,
        KeyCode::Char('t') => Act::Terminal,
        KeyCode::Char('v') => Act::Revert,
        KeyCode::Char('d') => Act::Download,
        KeyCode::Char('n') => Act::Notes,
        KeyCode::Char('s') => Act::Skip,
        KeyCode::Char('r') => Act::Check,
        KeyCode::Esc | KeyCode::Char('q') => Act::Close,
        _ => return None,
    })
}

fn scroll_by(delta: isize) {
    with(|s| {
        let max = s.lines.saturating_sub(usize::from(s.body.height).max(1));
        s.offset = s.offset.saturating_add_signed(delta).min(max);
    });
}

pub(crate) fn on_key(key: KeyEvent) -> Event {
    let page = with(|s| usize::from(s.body.height).saturating_sub(1).max(1)) as isize;
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => scroll_by(-1),
        KeyCode::Down | KeyCode::Char('j') => scroll_by(1),
        KeyCode::PageUp => scroll_by(-page),
        KeyCode::PageDown | KeyCode::Char(' ') => scroll_by(page),
        KeyCode::Home | KeyCode::Char('g') => with(|s| s.offset = 0),
        KeyCode::End | KeyCode::Char('G') => scroll_by(isize::MAX / 2),
        _ => {
            // A key whose button is not on screen still answers: the action
            // says why it does nothing (the same line the palette gives).
            if let Some(wanted) = key_action(&key) {
                return act(wanted);
            }
        }
    }
    Event::None
}

// ---- mouse ----

/// The mouse, before the panes: the open panel takes every event. `None`
/// when the panel is closed.
pub(crate) fn route_mouse(mouse: MouseEvent) -> Option<Event> {
    if !is_open() {
        return None;
    }
    Some(on_mouse(mouse))
}

fn on_mouse(mouse: MouseEvent) -> Event {
    with(|s| s.pointer = Some((mouse.column, mouse.row)));
    // The bar first: it sits where no button is.
    let consumed = with(|s| {
        let track = Track::vertical(s.bar_body, s.lines, s.offset);
        s.bar.on_mouse(track, mouse, s.lines, s.offset).map(|top| {
            let max = s.lines.saturating_sub(usize::from(s.body.height).max(1));
            s.offset = top.min(max);
        })
    });
    if consumed.is_some() {
        return Event::None;
    }
    match mouse.kind {
        MouseEventKind::ScrollUp => scroll_by(-3),
        MouseEventKind::ScrollDown => scroll_by(3),
        MouseEventKind::Down(MouseButton::Left) => {
            let at = (mouse.column, mouse.row).into();
            let (on_close, inside, hit) = with(|s| {
                (
                    s.close.contains(at),
                    s.window.contains(at),
                    s.hits.iter().find(|(r, _)| r.contains(at)).map(|(_, a)| *a),
                )
            });
            if on_close || !inside {
                close();
            } else if let Some(hit) = hit {
                return act(hit);
            }
        }
        _ => {}
    }
    Event::None
}

/// Whether a click at the point does something: the cross or a button.
pub(crate) fn pointer_hand(col: u16, row: u16) -> bool {
    let at = (col, row).into();
    with(|s| s.close.contains(at) || s.hits.iter().any(|(r, _)| r.contains(at)))
}

// ---- painting ----

fn style_of(look: Look) -> Style {
    let bg = th().ui.panel;
    match look {
        Look::Plain => Style::default().fg(th().ui.text).bg(bg),
        Look::Muted => Style::default().fg(th().ui.muted).bg(bg),
        Look::Heading => Style::default()
            .fg(th().ui.text)
            .bg(bg)
            .add_modifier(Modifier::BOLD),
        Look::Accent => Style::default()
            .fg(th().ui.accent)
            .bg(bg)
            .add_modifier(Modifier::BOLD),
        Look::Danger => Style::default()
            .fg(th().status.danger)
            .bg(bg)
            .add_modifier(Modifier::BOLD),
        Look::Command => Style::default().fg(th().ui.accent).bg(bg),
    }
}

/// Lays `labels` out in rows no wider than `width`: the index range of each.
fn button_rows(labels: &[String], width: u16) -> Vec<std::ops::Range<usize>> {
    let mut rows = Vec::new();
    let (mut start, mut used) = (0, 0u16);
    for (i, label) in labels.iter().enumerate() {
        let w = dialog_kit::button_width(label);
        let need = if used == 0 { w } else { used + 1 + w };
        if used > 0 && need > width {
            rows.push(start..i);
            start = i;
            used = w;
        } else {
            used = need;
        }
    }
    if start < labels.len() {
        rows.push(start..labels.len());
    }
    rows
}

pub(crate) fn render(frame: &mut Frame) {
    if !is_open() {
        return;
    }
    let area = frame.area();
    if area.width < 40 || area.height < 11 {
        return;
    }
    let snap = super::snapshot();
    let buttons = actions(&snap);
    let labels: Vec<String> = buttons.iter().map(|(_, l)| l.clone()).collect();
    let want_w = area.width.saturating_sub(4).min(92);
    let inner_w = want_w.saturating_sub(4);
    let text_w = usize::from(inner_w.saturating_sub(1));
    let rows = content(&snap, text_w, super::auto::progress());
    let button_lines = button_rows(&labels, inner_w);
    // Border, blank, body, blank, buttons, notice, hint, border.
    let chrome = 6 + button_lines.len() as u16;
    let want_h = (rows.len() as u16 + chrome).clamp(chrome + 3, area.height.saturating_sub(2));
    let rect = dialog_kit::centered(area, want_w, want_h);
    let buf = frame.buffer_mut();
    dialog_kit::panel(buf, rect, msg::PANEL_TITLE.get());

    let pointer = with(|s| s.pointer);
    let over = |r: Rect| pointer.is_some_and(|(x, y)| r.contains((x, y).into()));
    let close_spot = Rect::new(rect.right().saturating_sub(5), rect.y, 3, 1);
    let close = crate::chrome::close_cross::paint(buf, rect, over(close_spot)).unwrap_or_default();

    let x0 = rect.x + 2;
    let body = Rect::new(
        x0,
        rect.y + 2,
        inner_w.saturating_sub(1),
        rect.height.saturating_sub(chrome).max(1),
    );
    let offset = with(|s| {
        let max = rows.len().saturating_sub(usize::from(body.height));
        s.offset = s.offset.min(max);
        s.offset
    });
    for (n, row) in rows
        .iter()
        .skip(offset)
        .take(usize::from(body.height))
        .enumerate()
    {
        dialog_kit::put(
            buf,
            body.x,
            body.y + n as u16,
            &row.text,
            body.width,
            style_of(row.look),
        );
    }
    // The track lives in the border of a bordered box: hand it a stand-in two
    // cells wide (its right column is the bar) and one taller on each side.
    let bar_body = Rect::new(
        body.right().saturating_sub(1),
        body.y.saturating_sub(1),
        2,
        body.height + 2,
    );
    if let Some(track) = Track::vertical(bar_body, rows.len(), offset) {
        crate::scrollbar::with_pointer(pointer, || {
            crate::scrollbar::render_buf(buf, track, true, false)
        });
    }

    // Buttons, below the body; the main one is the accent button.
    let mut hits: Vec<(Rect, Act)> = Vec::new();
    let first_y = rect.bottom().saturating_sub(3 + button_lines.len() as u16);
    for (line, range) in button_lines.iter().enumerate() {
        let mut x = x0;
        for i in range.clone() {
            let (act, label) = &buttons[i];
            let w = dialog_kit::button_width(label);
            let r = Rect::new(x, first_y + line as u16, w, 1);
            let hovered = over(r);
            let state = if *act == Act::Primary {
                ButtonState::Primary {
                    hovered,
                    focused: false,
                }
            } else {
                ButtonState::Normal {
                    hovered,
                    focused: false,
                }
            };
            let drawn = dialog_kit::button(buf, r.x, r.y, label, state);
            hits.push((drawn, *act));
            x += w + 1;
        }
    }
    if let Some(notice) = with(|s| s.notice.clone()) {
        dialog_kit::put(
            buf,
            x0,
            rect.bottom().saturating_sub(3),
            &notice,
            inner_w,
            Style::default().fg(th().ui.accent).bg(th().ui.panel),
        );
    }
    dialog_kit::put(
        buf,
        x0,
        rect.bottom().saturating_sub(2),
        msg::PANEL_HINT.get(),
        inner_w,
        Style::default().fg(th().ui.muted).bg(th().ui.panel),
    );
    with(|s| {
        s.window = rect;
        s.close = close;
        s.hits = hits;
        s.body = body;
        s.bar_body = bar_body;
        s.lines = rows.len();
    });
}

#[cfg(test)]
mod tests;
