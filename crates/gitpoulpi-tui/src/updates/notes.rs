//! The Release notes tab: a view of the Home window (like the Radar), held
//! in `RepoManager::notes`, that shows the release body of a newer version on
//! top when one is known, then the changelog compiled into this build, scrolled
//! to the running version (to `Unreleased` for a development build).
//!
//! Buttons (each with a key and a palette command): Download (the release page
//! in the browser), Update now / Restart to update / Revert to <previous>
//! (a build that replaces its own files, or Run update for a package-manager
//! install, see `auto`), Update options (the update panel, for every other
//! install), Skip this version, Check now. The buttons run the same actions
//! as the update panel's (`updates::run`). Hit rects come out of the
//! paint walk; the body scrolls with the wheel, the keys and a draggable bar.

use crate::wide_edge::Clear;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::widgets::Widget;
use ratatui::Frame;

use gitpoulpi_config::UpdateMode;
use gitpoulpi_i18n::msg::updates as msg;

use gitpoulpi_update::plan::Procedure;

use super::auto::Phase;
use super::{Snapshot, UpdatesCommand};
use crate::dialog_kit::put;
use crate::markdown_lite::{self, Look};
use crate::scrollbar::{BarDrag, Track};
use crate::{button_bg, display_width, th};

/// `CHANGELOG.md` of this build.
const CHANGELOG: &str = include_str!("../../../../CHANGELOG.md");

/// Rows above the body: heading, buttons, rule.
const CHROME_TOP: u16 = 3;
/// Rows below it: the hint line.
const CHROME_BOTTOM: u16 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Hit {
    Download,
    UpdateNow,
    Restart,
    Revert,
    Skip,
    Check,
    /// The update panel.
    Panel,
}

/// What the event loop must do after the tab handled an event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum NotesEvent {
    None,
    /// Back to the Home list.
    Close,
    /// A line for the status bar.
    Message(String),
}

/// The slot `RepoManager` holds: the tab when it is showing.
#[derive(Default)]
pub(crate) struct ReleaseNotesSlot(Option<Box<NotesUi>>);

impl Clone for ReleaseNotesSlot {
    fn clone(&self) -> Self {
        Self(None)
    }
}

impl std::fmt::Debug for ReleaseNotesSlot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(if self.0.is_some() {
            "ReleaseNotesSlot(open)"
        } else {
            "ReleaseNotesSlot(closed)"
        })
    }
}

impl ReleaseNotesSlot {
    pub(crate) fn is_open(&self) -> bool {
        self.0.is_some()
    }

    pub(crate) fn open(&mut self) {
        if self.0.is_none() {
            self.0 = Some(Box::default());
        }
    }

    pub(crate) fn close(&mut self) {
        self.0 = None;
    }

    pub(crate) fn render(&mut self, frame: &mut Frame, body: Rect) {
        if let Some(ui) = self.0.as_mut() {
            ui.render(frame, body);
        }
    }

    /// Whether a click at the point does something: one of the buttons the
    /// last paint put on screen. The pointing hand.
    pub(crate) fn pointer_hand_at(&self, col: u16, row: u16) -> bool {
        self.0
            .as_ref()
            .is_some_and(|ui| ui.hit_at(col, row).is_some())
    }

    pub(crate) fn on_key(&mut self, key: KeyEvent) -> NotesEvent {
        self.0
            .as_mut()
            .map_or(NotesEvent::None, |ui| ui.on_key(key))
    }

    pub(crate) fn on_mouse(&mut self, mouse: MouseEvent) -> NotesEvent {
        self.0
            .as_mut()
            .map_or(NotesEvent::None, |ui| ui.on_mouse(mouse))
    }
}

/// One line of the body.
#[derive(Clone)]
enum Row {
    Md(markdown_lite::Line),
    Rule,
}

#[derive(Default)]
struct NotesUi {
    offset: usize,
    /// The rows laid out for `layout_key`.
    rows: Vec<Row>,
    layout_key: Option<(u16, Snapshot)>,
    /// The first row of the running version's section (the initial offset
    /// when nothing newer is known).
    anchor: usize,
    /// The first frame places the view; later ones keep where the user is.
    placed: bool,
    hits: Vec<(Rect, Hit)>,
    hover: Option<Hit>,
    body: Rect,
    bar_body: Rect,
    bar: BarDrag,
}

/// Where the section for `running` starts in `changelog` (byte offset): its
/// `## [x.y.z]` heading, else `## [Unreleased]`, else the top.
pub(crate) fn section_start(changelog: &str, running: &str, dev: bool) -> usize {
    let find = |needle: &str| -> Option<usize> {
        let mut at = 0;
        for line in changelog.split_inclusive('\n') {
            let t = line.trim_start();
            if t.starts_with("## ")
                && t[3..]
                    .trim_start_matches('[')
                    .to_ascii_lowercase()
                    .starts_with(needle)
            {
                return Some(at);
            }
            at += line.len();
        }
        None
    };
    let release = (!dev)
        .then(|| find(&running.to_ascii_lowercase()))
        .flatten();
    release.or_else(|| find("unreleased")).unwrap_or(0)
}

impl NotesUi {
    fn viewport(&self) -> usize {
        usize::from(self.body.height).max(1)
    }

    fn max_offset(&self) -> usize {
        self.rows.len().saturating_sub(self.viewport())
    }

    fn scroll_by(&mut self, delta: isize) {
        self.offset = self
            .offset
            .saturating_add_signed(delta)
            .min(self.max_offset());
    }

    /// Lays the body out for `width` and `snapshot`; keeps the old rows when
    /// neither changed.
    fn layout(&mut self, width: u16, snapshot: &Snapshot) {
        if self.layout_key.as_ref() == Some(&(width, snapshot.clone())) {
            return;
        }
        let text_w = usize::from(width.saturating_sub(3)).max(8);
        let mut rows: Vec<Row> = Vec::new();
        let release = snapshot.available.as_ref().or(snapshot.skipped.as_ref());
        if let Some(release) = release {
            let note = match &snapshot.install.procedure {
                // A build with no install record: say why there is no button.
                Procedure::Nothing(_) => Some(msg::NOTE_NOT_UPDATABLE.get()),
                Procedure::Swap { .. } | Procedure::Run { .. } => None,
                Procedure::Show { .. } | Procedure::Announce { .. } => {
                    Some(msg::NOTE_SEE_PANEL.get())
                }
            };
            if let Some(note) = note {
                rows.extend(markdown_lite::render(note, text_w).into_iter().map(Row::Md));
                rows.push(Row::Md(markdown_lite::Line::default()));
            }
            let body = if release.notes.trim().is_empty() {
                msg::NOTES_EMPTY.get().to_string()
            } else {
                release.notes.clone()
            };
            rows.extend(
                markdown_lite::render(&body, text_w)
                    .into_iter()
                    .map(Row::Md),
            );
            rows.push(Row::Md(markdown_lite::Line::default()));
            rows.push(Row::Rule);
            rows.push(Row::Md(markdown_lite::Line::default()));
        }
        let start = section_start(CHANGELOG, super::RUNNING, !super::GIT_HASH.is_empty());
        let before = markdown_lite::render(&CHANGELOG[..start], text_w).len();
        self.anchor = rows.len() + before;
        rows.extend(
            markdown_lite::render(CHANGELOG, text_w)
                .into_iter()
                .map(Row::Md),
        );
        self.rows = rows;
        self.layout_key = Some((width, snapshot.clone()));
        if !self.placed {
            // A newer release's notes come first; otherwise the running version.
            self.offset = if release.is_some() { 0 } else { self.anchor };
            self.placed = true;
        }
        self.offset = self.offset.min(self.max_offset());
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let snapshot = super::snapshot();
        let buf = frame.buffer_mut();
        Clear.render(area, buf);
        self.hits.clear();
        let body = Rect::new(
            area.x + 1,
            area.y + CHROME_TOP,
            area.width.saturating_sub(2),
            area.height.saturating_sub(CHROME_TOP + CHROME_BOTTOM),
        );
        self.body = body;
        self.layout(body.width, &snapshot);
        self.paint_header(buf, area, &snapshot);
        self.paint_body(buf, body);
        let hint = if snapshot.install.updatable {
            format!(" {}  {}", msg::TAB_HINT.get(), msg::TAB_HINT_UPDATE.get())
        } else {
            format!(" {}", msg::TAB_HINT.get())
        };
        put(
            buf,
            area.x,
            area.bottom().saturating_sub(1),
            &hint,
            area.width,
            Style::default().fg(th().ui.muted),
        );
    }

    fn paint_header(&mut self, buf: &mut Buffer, area: Rect, snapshot: &Snapshot) {
        let text = Style::default().fg(th().ui.text);
        let accent = Style::default()
            .fg(th().ui.accent)
            .add_modifier(Modifier::BOLD);
        let (title, style) = if let Phase::Downloading(version) = &snapshot.install.phase {
            let text = match super::auto::progress() {
                Some((done, Some(total))) if total > 0 => {
                    msg::head_downloading(version, done * 100 / total)
                }
                _ => msg::head_downloading_unknown(version),
            };
            (text, accent)
        } else if let Phase::Installing(version) = &snapshot.install.phase {
            (msg::head_installing(version), accent)
        } else if let Phase::Ready(version) = &snapshot.install.phase {
            (msg::head_ready(version), accent)
        } else if let Phase::Failed(reason) = &snapshot.install.phase {
            (
                msg::head_failed(reason),
                Style::default()
                    .fg(th().status.danger)
                    .add_modifier(Modifier::BOLD),
            )
        } else if snapshot.checking {
            (
                msg::HEAD_CHECKING.get().to_string(),
                Style::default().fg(th().ui.muted),
            )
        } else if let Some(r) = &snapshot.available {
            (
                msg::head_available(&r.version),
                Style::default()
                    .fg(th().ui.accent)
                    .add_modifier(Modifier::BOLD),
            )
        } else if let Some(r) = &snapshot.skipped {
            (
                msg::head_skipped(&r.version),
                text.add_modifier(Modifier::BOLD),
            )
        } else {
            (
                msg::head_current(super::RUNNING),
                text.add_modifier(Modifier::BOLD),
            )
        };
        put(
            buf,
            area.x + 1,
            area.y,
            &title,
            area.width.saturating_sub(2),
            style,
        );
        let off = snapshot.mode == UpdateMode::Off;
        if off && snapshot.install.revert_to.is_none() {
            put(
                buf,
                area.x + 1,
                area.y + 1,
                msg::HEAD_OFF.get(),
                area.width.saturating_sub(2),
                Style::default().fg(th().ui.muted),
            );
        } else {
            let mut x = area.x + 1;
            let known = snapshot.available.is_some() || snapshot.skipped.is_some();
            let install = &snapshot.install;
            let mut buttons: Vec<(Hit, String)> = Vec::new();
            if known && !off {
                buttons.push((Hit::Download, msg::BTN_DOWNLOAD.get().to_string()));
            }
            match &install.phase {
                Phase::Ready(_) => {
                    buttons.push((Hit::Restart, msg::BTN_RESTART.get().to_string()));
                }
                Phase::Idle | Phase::Failed(_)
                    if install.updatable && snapshot.available.is_some() && !off =>
                {
                    buttons.push((Hit::UpdateNow, msg::BTN_UPDATE_NOW.get().to_string()));
                }
                Phase::Idle | Phase::Failed(_)
                    if matches!(install.procedure, Procedure::Run { .. })
                        && snapshot.available.is_some()
                        && !off =>
                {
                    buttons.push((Hit::UpdateNow, msg::BTN_RUN_UPDATE.get().to_string()));
                }
                _ => {}
            }
            if known
                && !off
                && matches!(
                    install.procedure,
                    Procedure::Show { .. } | Procedure::Announce { .. }
                )
            {
                buttons.push((Hit::Panel, msg::BTN_PANEL.get().to_string()));
            }
            if let Some(previous) = &install.revert_to {
                if !matches!(install.phase, Phase::Downloading(_) | Phase::Installing(_)) {
                    buttons.push((Hit::Revert, msg::btn_revert(previous)));
                }
            }
            if snapshot.available.is_some() && !off {
                buttons.push((Hit::Skip, msg::BTN_SKIP.get().to_string()));
            }
            if !off {
                buttons.push((Hit::Check, msg::BTN_CHECK.get().to_string()));
            }
            for (hit, label) in buttons {
                let label = format!(" {label} ");
                let w = display_width(&label) as u16;
                if x + w > area.right() {
                    break;
                }
                let hovered = self.hover == Some(hit);
                let style = Style::default()
                    .fg(th().ui.on_button)
                    .bg(button_bg(hovered));
                let rect = put(buf, x, area.y + 1, &label, w, style);
                self.hits.push((rect, hit));
                x += w + 1;
            }
        }
        let rule = "\u{2500}".repeat(usize::from(area.width.saturating_sub(2)));
        put(
            buf,
            area.x + 1,
            area.y + 2,
            &rule,
            area.width.saturating_sub(2),
            Style::default().fg(th().ui.muted),
        );
    }

    fn paint_body(&mut self, buf: &mut Buffer, body: Rect) {
        let height = usize::from(body.height);
        for (i, row) in self.rows.iter().skip(self.offset).take(height).enumerate() {
            let y = body.y + i as u16;
            match row {
                Row::Rule => {
                    let rule = "\u{2500}".repeat(usize::from(body.width));
                    put(
                        buf,
                        body.x,
                        y,
                        &rule,
                        body.width,
                        Style::default().fg(th().ui.muted),
                    );
                }
                Row::Md(line) => {
                    let mut x = body.x;
                    for span in &line.spans {
                        let room = body.right().saturating_sub(x);
                        let r = put(buf, x, y, &span.text, room, md_style(span.look));
                        x += r.width;
                    }
                }
            }
        }
        // The bar takes the right edge; the track lives in the border of a
        // bordered box, so hand it a stand-in two cells wide (its right column is the bar) and one cell taller on each side.
        self.bar_body = Rect::new(
            body.right().saturating_sub(1),
            body.y.saturating_sub(1),
            2,
            body.height + 2,
        );
        if let Some(track) = Track::vertical(self.bar_body, self.rows.len(), self.offset) {
            crate::scrollbar::render_buf(buf, track, true, false);
        }
    }

    fn hit_at(&self, col: u16, row: u16) -> Option<Hit> {
        self.hits
            .iter()
            .find(|(r, _)| r.contains((col, row).into()))
            .map(|(_, h)| *h)
    }

    fn command(&mut self, hit: Hit) -> NotesEvent {
        let command = match hit {
            Hit::Download => UpdatesCommand::Download,
            Hit::UpdateNow => UpdatesCommand::UpdateNow,
            Hit::Restart => UpdatesCommand::Restart,
            Hit::Revert => UpdatesCommand::Revert,
            Hit::Skip => UpdatesCommand::Skip,
            Hit::Check => UpdatesCommand::CheckNow,
            Hit::Panel => UpdatesCommand::ShowPanel,
        };
        super::run(command).map_or(NotesEvent::None, NotesEvent::Message)
    }

    fn on_key(&mut self, key: KeyEvent) -> NotesEvent {
        if key
            .modifiers
            .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT)
        {
            return NotesEvent::None;
        }
        let page = self.viewport().saturating_sub(1).max(1) as isize;
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => return NotesEvent::Close,
            KeyCode::Char('d') => return self.command(Hit::Download),
            KeyCode::Char('u') => return self.command(Hit::UpdateNow),
            KeyCode::Char('R') => return self.command(Hit::Restart),
            KeyCode::Char('v') => return self.command(Hit::Revert),
            KeyCode::Char('s') => return self.command(Hit::Skip),
            KeyCode::Char('r') => return self.command(Hit::Check),
            KeyCode::Char('p') => return self.command(Hit::Panel),
            KeyCode::Up | KeyCode::Char('k') => self.scroll_by(-1),
            KeyCode::Down | KeyCode::Char('j') => self.scroll_by(1),
            KeyCode::PageUp => self.scroll_by(-page),
            KeyCode::PageDown | KeyCode::Char(' ') => self.scroll_by(page),
            KeyCode::Home | KeyCode::Char('g') => self.offset = 0,
            KeyCode::End | KeyCode::Char('G') => self.offset = self.max_offset(),
            _ => {}
        }
        NotesEvent::None
    }

    fn on_mouse(&mut self, mouse: MouseEvent) -> NotesEvent {
        let (col, row) = (mouse.column, mouse.row);
        let track = Track::vertical(self.bar_body, self.rows.len(), self.offset);
        if let Some(top) = self
            .bar
            .on_mouse(track, mouse, self.rows.len(), self.offset)
        {
            self.offset = top.min(self.max_offset());
            return NotesEvent::None;
        }
        match mouse.kind {
            MouseEventKind::Moved => {
                self.hover = self.hit_at(col, row);
                NotesEvent::None
            }
            MouseEventKind::ScrollDown => {
                self.scroll_by(3);
                NotesEvent::None
            }
            MouseEventKind::ScrollUp => {
                self.scroll_by(-3);
                NotesEvent::None
            }
            MouseEventKind::Down(MouseButton::Left) => match self.hit_at(col, row) {
                Some(hit) => self.command(hit),
                None => NotesEvent::None,
            },
            _ => NotesEvent::None,
        }
    }
}

fn md_style(look: Look) -> Style {
    let base = Style::default().fg(th().ui.text);
    match look {
        Look::Plain => base,
        Look::Bold => base.add_modifier(Modifier::BOLD),
        Look::Italic => base.add_modifier(Modifier::ITALIC),
        Look::BoldItalic => base.add_modifier(Modifier::BOLD | Modifier::ITALIC),
        Look::Strike => base.add_modifier(Modifier::CROSSED_OUT),
        Look::Code => Style::default().fg(th().ui.accent),
        Look::Link => Style::default()
            .fg(th().ui.accent)
            .add_modifier(Modifier::UNDERLINED),
        Look::Heading => Style::default()
            .fg(th().ui.accent)
            .add_modifier(Modifier::BOLD),
        Look::Quote => Style::default()
            .fg(th().ui.muted)
            .add_modifier(Modifier::ITALIC),
        Look::Marker => Style::default().fg(th().ui.accent),
    }
}
