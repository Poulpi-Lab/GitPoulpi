//! "Copy as Mermaid gitGraph...": the popup that turns a run of Log commits
//! into a Mermaid `gitGraph` script (`gitpoulpi_core::domain::mermaid_git_graph`,
//! the generator the CLI's `log --format mermaid` runs too).
//!
//! Opened from the commit context menu, the Log key (`Shift+Y`) and the
//! palette: all one function, `apply_log_context_action`. It holds three
//! comboboxes (which commits, how much detail, which direction), the fence
//! checkbox and a scrollable live preview of the exact text that `Enter`
//! copies. Detail, direction and fence are remembered (`ui.mermaid_*`).
//!
//! Modal like `revert_mainline`: it owns the keyboard and the mouse while
//! open, a click away closes it, nothing behind it hovers. Every mouse
//! gesture has a key: click a field = `Enter` / `Space` on it, the wheel and
//! the scrollbar = `Up` / `Down` / `PageUp` / `PageDown`, the Copy button =
//! `Enter`, a click away = `Esc`. Hit rects are kept from the paint walk.

use std::path::PathBuf;
use std::sync::Mutex;

use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};
use gitpoulpi_core::domain::{
    attach_refs, commits_to_tip, mermaid_git_graph, CommitGraph, CommitNode, MermaidDetail,
    MermaidOptions, MermaidOrientation, Oid, Ref,
};
use gitpoulpi_i18n::msg::mermaid as msg;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, BorderType, Borders, Widget};
use ratatui::Frame;

use crate::combo::{Combo, ComboOption, ComboOutcome};
use crate::dialog_kit::{self, ButtonState, CheckState};
use crate::live_settings::{self, MermaidPrefs};
use crate::scrollbar::{self, BarDrag, Track};
use crate::th;

/// Which commits the script covers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Scope {
    /// Exactly the selected commits.
    Selection,
    /// From the oldest selected commit up to HEAD (the loaded graph's answer).
    ToHead,
}

impl Scope {
    fn value(self) -> &'static str {
        match self {
            Scope::Selection => "selection",
            Scope::ToHead => "to_head",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Field {
    Scope,
    Detail,
    Orientation,
    Fence,
    Preview,
}

const FIELDS: [Field; 5] = [
    Field::Scope,
    Field::Detail,
    Field::Orientation,
    Field::Fence,
    Field::Preview,
];

/// What the popup asks of the event loop.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Outcome {
    /// Nothing for the caller (the popup may have closed or changed).
    None,
    /// Put this text on the clipboard; the popup has closed.
    Copy(String),
}

#[derive(Default)]
struct Rects {
    outer: Rect,
    preview: Rect,
    fence: Rect,
    copy: Rect,
    close: Rect,
}

struct Open {
    /// The loaded graph with the refs attached to its commits.
    nodes: Vec<CommitNode>,
    head: Option<Oid>,
    /// The selected commits that are loaded, as ids.
    selected: Vec<Oid>,
    scope: Scope,
    prefs: MermaidPrefs,
    scope_combo: Combo,
    detail_combo: Combo,
    orient_combo: Combo,
    focus: Field,
    text: String,
    lines: Vec<String>,
    commits: usize,
    top: usize,
    rows: usize,
    pointer: Option<(u16, u16)>,
    bar: BarDrag,
    track: Option<Track>,
    rects: Rects,
    cross: dialog_kit::CloseHit,
}

static POPUP: Mutex<Option<Open>> = Mutex::new(None);

fn detail_label(d: MermaidDetail) -> &'static str {
    match d {
        MermaidDetail::Structure => msg::DETAIL_STRUCTURE.get(),
        MermaidDetail::Refs => msg::DETAIL_REFS.get(),
        MermaidDetail::Subjects => msg::DETAIL_SUBJECTS.get(),
        MermaidDetail::Full => msg::DETAIL_FULL.get(),
    }
}

fn orientation_label(o: MermaidOrientation) -> &'static str {
    match o {
        MermaidOrientation::Lr => msg::ORIENTATION_LR.get(),
        MermaidOrientation::Tb => msg::ORIENTATION_TB.get(),
    }
}

impl Open {
    fn scope_commits(&self, scope: Scope) -> Vec<CommitNode> {
        let picked: Vec<CommitNode> = self
            .nodes
            .iter()
            .filter(|n| self.selected.contains(&n.id))
            .cloned()
            .collect();
        match (scope, self.head) {
            (Scope::ToHead, Some(head)) => commits_to_tip(&self.nodes, &self.selected, head),
            _ => picked,
        }
    }

    fn rebuild(&mut self) {
        let commits = self.scope_commits(self.scope);
        self.commits = commits.len();
        self.text = mermaid_git_graph(
            &commits,
            self.head,
            &MermaidOptions {
                detail: self.prefs.detail,
                orientation: self.prefs.orientation,
                fence: self.prefs.fence,
                subject_width: self.prefs.subject_width,
            },
        );
        self.lines = self.text.lines().map(str::to_string).collect();
        self.top = self.top.min(self.max_top());
    }

    fn max_top(&self) -> usize {
        self.lines.len().saturating_sub(self.rows.max(1))
    }

    fn scroll_to(&mut self, top: usize) {
        self.top = top.min(self.max_top());
    }

    fn scroll_by(&mut self, delta: isize) {
        self.scroll_to((self.top as isize + delta).max(0) as usize);
    }

    fn combo_mut(&mut self, field: Field) -> Option<&mut Combo> {
        match field {
            Field::Scope => Some(&mut self.scope_combo),
            Field::Detail => Some(&mut self.detail_combo),
            Field::Orientation => Some(&mut self.orient_combo),
            Field::Fence | Field::Preview => None,
        }
    }

    fn open_combo(&mut self) -> Option<Field> {
        [Field::Scope, Field::Detail, Field::Orientation]
            .into_iter()
            .find(|f| self.combo_mut(*f).is_some_and(|c| c.is_open()))
    }

    fn pick(&mut self, field: Field, value: &str, config_path: &Option<PathBuf>) {
        match field {
            Field::Scope => {
                self.scope = if value == Scope::ToHead.value() {
                    Scope::ToHead
                } else {
                    Scope::Selection
                };
            }
            Field::Detail => {
                if let Some(d) = MermaidDetail::parse(value) {
                    self.prefs.detail = d;
                    persist("ui.mermaid_detail", d.as_str(), self.prefs, config_path);
                }
            }
            Field::Orientation => {
                if let Some(o) = MermaidOrientation::parse(value) {
                    self.prefs.orientation = o;
                    persist(
                        "ui.mermaid_orientation",
                        o.as_str(),
                        self.prefs,
                        config_path,
                    );
                }
            }
            Field::Fence | Field::Preview => {}
        }
        self.rebuild();
    }

    fn toggle_fence(&mut self, config_path: &Option<PathBuf>) {
        self.prefs.fence = !self.prefs.fence;
        persist(
            "ui.mermaid_fence",
            if self.prefs.fence { "true" } else { "false" },
            self.prefs,
            config_path,
        );
        self.rebuild();
    }

    /// Opens the combo of `field` (the others stay closed).
    fn open_field(&mut self, field: Field) {
        self.focus = field;
        if let Some(c) = self.combo_mut(field) {
            c.open();
        }
    }
}

/// Publishes the remembered choice and writes the one key that changed.
fn persist(key: &str, value: &str, prefs: MermaidPrefs, config_path: &Option<PathBuf>) {
    live_settings::set_mermaid(prefs);
    if let Some(path) = config_path {
        let _ = gitpoulpi_config::set_in_file(path, key, value);
    }
}

fn detail_options() -> Vec<ComboOption> {
    MermaidDetail::ALL
        .into_iter()
        .map(|d| ComboOption::new(detail_label(d), d.as_str()))
        .collect()
}

fn orientation_options() -> Vec<ComboOption> {
    MermaidOrientation::ALL
        .into_iter()
        .map(|o| ComboOption::new(orientation_label(o), o.as_str()))
        .collect()
}

/// Opens the popup for `selected` (the Log's multi-selection, newest first)
/// or, with fewer than two, for the commit under the cursor. `Err` is the
/// status-line sentence when there is nothing to copy.
pub(crate) fn open(
    graph: &CommitGraph,
    refs: &[Ref],
    selected: &[Oid],
    cursor: Oid,
) -> Result<(), String> {
    let mut nodes = graph.nodes.clone();
    attach_refs(&mut nodes, refs);
    let wanted: Vec<Oid> = if selected.len() > 1 {
        selected.to_vec()
    } else {
        vec![cursor]
    };
    let selected: Vec<Oid> = wanted
        .into_iter()
        .filter(|id| nodes.iter().any(|n| n.id == *id))
        .collect();
    if selected.is_empty() {
        return Err(msg::NO_COMMITS.get().into());
    }
    let head = crate::refs_sidebar::head_oid(refs);
    let mut open = Open {
        nodes,
        head,
        selected,
        scope: Scope::Selection,
        prefs: live_settings::mermaid(),
        scope_combo: Combo::new(""),
        detail_combo: Combo::new(""),
        orient_combo: Combo::new(""),
        focus: Field::Preview,
        text: String::new(),
        lines: Vec::new(),
        commits: 0,
        top: 0,
        rows: 10,
        pointer: None,
        bar: BarDrag::default(),
        track: None,
        rects: Rects::default(),
        cross: Default::default(),
    };
    let to_head = open.scope_commits(Scope::ToHead).len();
    let selection = open.scope_commits(Scope::Selection).len();
    let mut scope_options = vec![ComboOption::new(
        msg::scope_selection(selection as u64),
        Scope::Selection.value(),
    )];
    if head.is_some() && to_head > selection {
        scope_options.push(ComboOption::new(
            msg::scope_to_head(to_head as u64),
            Scope::ToHead.value(),
        ));
        // One commit alone is a dot: the useful default is the run to HEAD.
        if selection == 1 {
            open.scope = Scope::ToHead;
        }
    }
    open.scope_combo.set_options(scope_options);
    open.scope_combo
        .select(Some(open.scope.value().to_string()));
    open.detail_combo.set_options(detail_options());
    open.detail_combo
        .select(Some(open.prefs.detail.as_str().to_string()));
    open.orient_combo.set_options(orientation_options());
    open.orient_combo
        .select(Some(open.prefs.orientation.as_str().to_string()));
    open.rebuild();
    if let Ok(mut slot) = POPUP.lock() {
        *slot = Some(open);
    }
    Ok(())
}

pub(crate) fn is_open() -> bool {
    POPUP.lock().is_ok_and(|p| p.is_some())
}

fn close(slot: &mut Option<Open>) {
    *slot = None;
}

fn copy_out(slot: &mut Option<Open>) -> Outcome {
    match slot.take() {
        Some(open) => Outcome::Copy(open.text),
        None => Outcome::None,
    }
}

/// A key while the popup is open (it swallows them all).
pub(crate) fn on_key(key: KeyEvent, config_path: &Option<PathBuf>) -> Outcome {
    let Ok(mut slot) = POPUP.lock() else {
        return Outcome::None;
    };
    let Some(open) = slot.as_mut() else {
        return Outcome::None;
    };
    if let Some(field) = open.open_combo() {
        let outcome = open
            .combo_mut(field)
            .map_or(ComboOutcome::Ignored, |c| c.on_key(key));
        if let ComboOutcome::Picked(value) = outcome {
            open.pick(field, &value, config_path);
        }
        return Outcome::None;
    }
    let step = |from: Field, back: bool| {
        let at = FIELDS.iter().position(|f| *f == from).unwrap_or(0);
        let next = if back {
            (at + FIELDS.len() - 1) % FIELDS.len()
        } else {
            (at + 1) % FIELDS.len()
        };
        FIELDS[next]
    };
    let page = open.rows.max(2) - 1;
    match key.code {
        KeyCode::Esc => close(&mut slot),
        KeyCode::Tab => open.focus = step(open.focus, false),
        KeyCode::BackTab => open.focus = step(open.focus, true),
        KeyCode::Up if open.focus == Field::Preview => open.scroll_by(-1),
        KeyCode::Down if open.focus == Field::Preview => open.scroll_by(1),
        KeyCode::Char('k') if open.focus == Field::Preview => open.scroll_by(-1),
        KeyCode::Char('j') if open.focus == Field::Preview => open.scroll_by(1),
        KeyCode::PageUp => open.scroll_by(-(page as isize)),
        KeyCode::PageDown => open.scroll_by(page as isize),
        KeyCode::Home if open.focus == Field::Preview => open.scroll_to(0),
        KeyCode::End if open.focus == Field::Preview => open.scroll_to(usize::MAX),
        KeyCode::Char(' ') if open.focus == Field::Fence => open.toggle_fence(config_path),
        // A combo field opens; anywhere else Enter copies, from any field.
        KeyCode::Enter | KeyCode::Char(' ') | KeyCode::Down
            if matches!(
                open.focus,
                Field::Scope | Field::Detail | Field::Orientation
            ) =>
        {
            let field = open.focus;
            open.open_field(field);
        }
        KeyCode::Enter => return copy_out(&mut slot),
        _ => {}
    }
    Outcome::None
}

/// A mouse event while the popup is open (it swallows them all).
pub(crate) fn on_mouse(mouse: MouseEvent, config_path: &Option<PathBuf>) -> Outcome {
    let Ok(mut slot) = POPUP.lock() else {
        return Outcome::None;
    };
    let Some(open) = slot.as_mut() else {
        return Outcome::None;
    };
    if let Some(field) = open.open_combo() {
        let outcome = open
            .combo_mut(field)
            .map_or(ComboOutcome::Ignored, |c| c.on_mouse(mouse));
        if let ComboOutcome::Picked(value) = outcome {
            open.pick(field, &value, config_path);
        }
        return Outcome::None;
    }
    let at: ratatui::layout::Position = (mouse.column, mouse.row).into();
    open.pointer = Some((mouse.column, mouse.row));
    if matches!(mouse.kind, MouseEventKind::Moved | MouseEventKind::Drag(_)) {
        open.cross.set_hover(mouse.column, mouse.row);
    }
    // The scrollbar first: its thumb drags even off the track.
    if let Some(top) = open
        .bar
        .on_mouse(open.track, mouse, open.lines.len(), open.top)
    {
        open.scroll_to(top);
        return Outcome::None;
    }
    match mouse.kind {
        MouseEventKind::ScrollUp => open.scroll_by(-3),
        MouseEventKind::ScrollDown => open.scroll_by(3),
        MouseEventKind::Down(MouseButton::Left) => {
            if open.rects.copy.contains(at) {
                return copy_out(&mut slot);
            }
            if open.rects.close.contains(at)
                || open.cross.hit(mouse.column, mouse.row)
                || !open.rects.outer.contains(at)
            {
                close(&mut slot);
                return Outcome::None;
            }
            if open.rects.fence.contains(at) {
                open.focus = Field::Fence;
                open.toggle_fence(config_path);
                return Outcome::None;
            }
            for field in [Field::Scope, Field::Detail, Field::Orientation] {
                let opened = open
                    .combo_mut(field)
                    .is_some_and(|c| c.on_mouse(mouse) == ComboOutcome::Handled);
                if opened {
                    open.focus = field;
                    return Outcome::None;
                }
            }
            if open.rects.preview.contains(at) {
                open.focus = Field::Preview;
            }
        }
        _ => {}
    }
    Outcome::None
}

fn keyword_style() -> Style {
    Style::default()
        .fg(th().ui.accent)
        .bg(th().ui.panel)
        .add_modifier(Modifier::BOLD)
}

/// One preview line: the statement's keyword in the accent color.
fn put_script_line(buf: &mut Buffer, x: u16, y: u16, width: u16, line: &str) {
    let text = Style::default().fg(th().ui.text).bg(th().ui.panel);
    let body = line.trim_start();
    let indent = line.len() - body.len();
    let (word, rest) = body.split_once(' ').unwrap_or((body, ""));
    let is_keyword = matches!(
        word,
        "commit" | "branch" | "checkout" | "merge" | "gitGraph" | "cherry-pick"
    );
    if !is_keyword {
        dialog_kit::put(buf, x, y, line, width, text);
        return;
    }
    let lead = indent as u16;
    let used = dialog_kit::put(
        buf,
        x + lead,
        y,
        word,
        width.saturating_sub(lead),
        keyword_style(),
    );
    let after = lead + used.width;
    dialog_kit::put(
        buf,
        x + after,
        y,
        &format!(" {rest}"),
        width.saturating_sub(after),
        text,
    );
}

/// Paints the popup, last, over the panes.
pub(crate) fn render(frame: &mut Frame) {
    let Ok(mut slot) = POPUP.lock() else {
        return;
    };
    let Some(open) = slot.as_mut() else {
        return;
    };
    let area = frame.area();
    if area.width < 30 || area.height < 12 {
        return;
    }
    let outer = dialog_kit::centered(
        area,
        100.min(area.width.saturating_sub(2)),
        32.min(area.height.saturating_sub(2)),
    );
    let buf = frame.buffer_mut();
    open.cross.panel(buf, outer, msg::TITLE.get());
    open.rects = Rects {
        outer,
        ..Rects::default()
    };
    let inner_x = outer.x + 2;
    let inner_w = outer.width.saturating_sub(4);
    let muted = Style::default().fg(th().ui.muted).bg(th().ui.panel);
    let label_w = [
        msg::SCOPE_LABEL.get(),
        msg::DETAIL_LABEL.get(),
        msg::ORIENTATION_LABEL.get(),
    ]
    .iter()
    .map(|l| crate::display_width(l))
    .max()
    .unwrap_or(8) as u16
        + 2;
    let field_x = inner_x + label_w;
    let field_w = inner_w.saturating_sub(label_w);
    let hovered = |r: Rect, p: Option<(u16, u16)>| p.is_some_and(|p| r.contains(p.into()));
    let pointer = open.pointer;
    let focus = open.focus;
    let rows = [
        (Field::Scope, msg::SCOPE_LABEL.get()),
        (Field::Detail, msg::DETAIL_LABEL.get()),
        (Field::Orientation, msg::ORIENTATION_LABEL.get()),
    ];
    for (i, (field, label)) in rows.into_iter().enumerate() {
        let y = outer.y + 1 + i as u16;
        dialog_kit::put(buf, inner_x, y, label, label_w, muted);
        let rect = Rect::new(field_x, y, field_w, 1);
        let is_hovered = hovered(rect, pointer);
        if let Some(c) = open.combo_mut(field) {
            c.render_field(buf, rect, focus == field, is_hovered);
        }
    }
    let fence_y = outer.y + 4;
    open.rects.fence = dialog_kit::checkbox(
        buf,
        inner_x,
        fence_y,
        inner_w,
        msg::FENCE.get(),
        open.prefs.fence,
        CheckState::Enabled {
            hovered: hovered(Rect::new(inner_x, fence_y, inner_w, 1), pointer),
            focused: focus == Field::Fence,
        },
    );
    // Make the whole row clickable, not just the label's own width.
    open.rects.fence = Rect::new(inner_x, fence_y, inner_w, 1);

    let preview = Rect::new(
        outer.x + 1,
        outer.y + 5,
        outer.width.saturating_sub(2),
        outer.height.saturating_sub(5 + 4),
    );
    open.rects.preview = preview;
    open.rows = (preview.height as usize).saturating_sub(2).max(1);
    open.top = open.top.min(open.max_top());
    let title = format!(" {} ", msg::PREVIEW.get());
    Block::default()
        .title(title)
        .title_style(Style::default().fg(th().ui.muted))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(if focus == Field::Preview {
            th().ui.accent
        } else {
            th().ui.muted
        }))
        .style(Style::default().bg(th().ui.panel))
        .render(preview, buf);
    let text_w = preview.width.saturating_sub(3);
    for (i, line) in open.lines.iter().skip(open.top).take(open.rows).enumerate() {
        put_script_line(buf, preview.x + 1, preview.y + 1 + i as u16, text_w, line);
    }
    open.track = Track::vertical(preview, open.lines.len(), open.top);
    if let Some(track) = open.track {
        scrollbar::render_buf(buf, track, true, false);
    }

    let by = outer.bottom().saturating_sub(3);
    let copy_label = msg::COPY_BUTTON.get();
    let close_label = msg::CLOSE_BUTTON.get();
    let copy_rect = Rect::new(inner_x, by, dialog_kit::button_width(copy_label), 1);
    let close_rect = Rect::new(
        copy_rect.right() + 2,
        by,
        dialog_kit::button_width(close_label),
        1,
    );
    open.rects.copy = dialog_kit::button(
        buf,
        copy_rect.x,
        by,
        copy_label,
        ButtonState::Primary {
            hovered: hovered(copy_rect, pointer),
            focused: false,
        },
    );
    open.rects.close = dialog_kit::button(
        buf,
        close_rect.x,
        by,
        close_label,
        ButtonState::Normal {
            hovered: hovered(close_rect, pointer),
            focused: false,
        },
    );
    dialog_kit::hints(
        buf,
        inner_x,
        outer.bottom().saturating_sub(2),
        inner_w,
        &[
            ("Tab", msg::HINT_NEXT.get()),
            ("Enter", msg::HINT_COPY.get()),
            ("\u{2191}/\u{2193}", msg::HINT_SCROLL.get()),
        ],
    );
    // The open dropdown last, over the preview.
    open.scope_combo.render_popup(buf, area);
    open.detail_combo.render_popup(buf, area);
    open.orient_combo.render_popup(buf, area);
}

#[cfg(test)]
mod tests;
