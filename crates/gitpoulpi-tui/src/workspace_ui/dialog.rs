//! The New / Edit workspace form.
//!
//! ```text
//! ╭ New workspace ──────────────────────────────────────────────╮
//! │  Name           Acme█                                       │
//! │  Color          ● Teal                                  ▾   │
//! │  Description    the web stack                               │
//! │  Profile        Work                                    ▾   │
//! │  Repositories   2 selected                    [ Browse... ] │
//! │   [x] acme-api     ~/src/acme-api                           │
//! │   [x] acme-web     ~/src/acme-web                           │
//! │   [ ] notes        ~/src/notes                              │
//! │  Tab next · Space toggle · Enter save · Esc cancel          │
//! ╰──────────────────────────────────────────────────────────────╯
//! ```
//!
//! Pure like `init_dialog`: state, keys, clicks and one paint walk that
//! returns the hit rects with the glyphs. Saving is `workspace_ui::task`
//! (the origin URLs are a daemon read), started by `repo_dialogs` from
//! [`Outcome::Submit`]; Browse is the shared folder picker, also opened there.
//!
//! Colour and Profile are comboboxes (maintainer rule: never click-to-cycle);
//! the repository list is a multi-select over what the app knows (open tabs,
//! favorites, recent, the workspace's own), plus what Browse adds.

use std::path::{Path, PathBuf};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use gitpoulpi_config::WorkspaceAi;
use gitpoulpi_i18n::msg::workspace_ai as ai_m;
use gitpoulpi_i18n::msg::workspaces as m;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;

use super::ai_box::{self, AiBox, AiBoxOutcome};
use super::task::Draft;
use crate::combo::{Combo, ComboOption, ComboOutcome};
use crate::dialog_kit::{self, ButtonState};
use crate::keymap::chord::Key;
use crate::th;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Focus {
    Name,
    Color,
    Description,
    Profile,
    /// "✦ AI instructions": only while AI is on.
    Ai,
    Repos,
    Browse,
    Cancel,
    Save,
}

const ORDER: [Focus; 9] = [
    Focus::Name,
    Focus::Color,
    Focus::Description,
    Focus::Profile,
    Focus::Ai,
    Focus::Repos,
    Focus::Browse,
    Focus::Cancel,
    Focus::Save,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Hit {
    Part(Focus),
    /// A repository row, an index into [`WorkspaceDialog::repos`].
    Row(usize),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Outcome {
    Stay,
    Cancel,
    /// Open the folder picker to add a repository.
    Browse,
    /// Write this workspace.
    Submit(Draft),
}

/// One row of the repository list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RepoChoice {
    pub(crate) path: PathBuf,
    pub(crate) name: String,
    pub(crate) checked: bool,
    /// Gone from disk: listed (a workspace keeps them), marked.
    pub(crate) missing: bool,
}

pub(crate) struct WorkspaceDialog {
    id: Option<String>,
    name: String,
    description: String,
    color: Combo,
    profile: Combo,
    pub(crate) repos: Vec<RepoChoice>,
    cursor: usize,
    scroll: usize,
    /// The cursor the window last followed: a wheel turn or a bar drag moves
    /// the window away from it and stays until the cursor moves.
    followed: usize,
    /// The list's scrollbar as last painted, and the thumb held.
    bar: Option<crate::scrollbar::Track>,
    bar_drag: crate::scrollbar::BarDrag,
    list_rows: usize,
    focus: Focus,
    hover: Option<Hit>,
    hits: Vec<(Rect, Hit)>,
    close: dialog_kit::CloseHit,
    /// What the name, description and checked repositories held when the
    /// dialog opened (dirty = different).
    pristine: String,
    /// A save is running.
    pub(crate) busy: bool,
    pub(crate) message: Option<String>,
    /// Names the other workspaces carry (lower-cased), for the uniqueness check.
    taken: Vec<String>,
    edit: crate::text_edit::FieldEditor<Focus>,
    /// The workspace's AI instructions; `None` while AI is off (the row is
    /// not there and the workspace's are left as they are).
    ai: Option<WorkspaceAi>,
    ai_box: Option<AiBox>,
    /// A click that closed a combo popup must not also land on what was
    /// under it (the pull request form's rule).
    shield: Option<(Rect, std::time::Instant)>,
}

/// How long after a click closed a combo the area it covered ignores clicks.
const SHIELD: std::time::Duration = std::time::Duration::from_millis(300);

/// What the dialog is opened with.
pub(crate) struct Seed {
    /// The workspace being edited (`None` = new).
    pub(crate) id: Option<String>,
    pub(crate) name: String,
    pub(crate) color: Option<String>,
    pub(crate) description: String,
    pub(crate) profile: Option<String>,
    /// The workspace's own repositories (checked), missing ones included.
    pub(crate) members: Vec<PathBuf>,
    /// Everything else the app knows: (path, name).
    pub(crate) known: Vec<(PathBuf, String)>,
    /// `[[profiles]]`: (id, name).
    pub(crate) profiles: Vec<(String, String)>,
    /// Other workspaces' names.
    pub(crate) taken_names: Vec<String>,
    /// AI is on: the form shows the AI instructions row.
    pub(crate) ai_enabled: bool,
    /// The workspace's AI instructions.
    pub(crate) ai: WorkspaceAi,
}

fn name_of(path: &Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string())
}

impl WorkspaceDialog {
    /// What `Ctrl+N` badges: every field and button, and the first repository row.
    pub(crate) fn nav_targets(&self) -> Vec<(Rect, (u16, u16))> {
        let mut seen_row = false;
        dialog_kit::nav_cells(self.hits.iter().filter_map(|(r, hit)| match hit {
            Hit::Part(_) => Some(*r),
            Hit::Row(_) if !std::mem::replace(&mut seen_row, true) => Some(*r),
            Hit::Row(_) => None,
        }))
    }

    /// Whether a click at the point is a control: every button and field the
    /// last paint put on screen, not only the badged ones. A repository row
    /// keeps the arrow (its checkbox mark is a hand, registered by the
    /// paint); the text fields are answered ahead of this (`pointer_map`).
    pub(crate) fn pointer_hand_at(&self, col: u16, row: u16) -> bool {
        matches!(dialog_kit::hit_at(&self.hits, col, row), Some(Hit::Part(_)))
    }

    pub(crate) fn new(seed: Seed) -> Self {
        let mut color = Combo::new("");
        color.set_options(
            super::COLORS
                .iter()
                .map(|(id, label)| {
                    ComboOption::new(label.get(), *id).with_swatch(vec![super::color_of(Some(id))])
                })
                .collect(),
        );
        color.select(Some(seed.color.clone().unwrap_or_else(|| "accent".into())));
        let mut profile = Combo::new("");
        let mut options = vec![ComboOption::new(m::PROFILE_NONE.get(), "")];
        options.extend(
            seed.profiles
                .iter()
                .map(|(id, name)| ComboOption::new(name.clone(), id.clone())),
        );
        // A profile id that names nothing stays selectable as itself, so
        // editing a workspace never drops it silently.
        if let Some(id) = seed.profile.as_deref() {
            if !seed.profiles.iter().any(|(p, _)| p == id) {
                options.push(ComboOption::new(m::profile_gone(id), id));
            }
        }
        profile.set_options(options);
        profile.select(Some(seed.profile.clone().unwrap_or_default()));

        let mut repos: Vec<RepoChoice> = seed
            .members
            .iter()
            .map(|p| RepoChoice {
                path: p.clone(),
                name: name_of(p),
                checked: true,
                missing: !p.exists(),
            })
            .collect();
        for (path, name) in seed.known {
            if !repos
                .iter()
                .any(|r| gitpoulpi_session::same_repo_path(&r.path, &path))
            {
                repos.push(RepoChoice {
                    missing: !path.exists(),
                    path,
                    name,
                    checked: false,
                });
            }
        }
        let mut dialog = Self {
            id: seed.id,
            name: seed.name,
            description: seed.description,
            color,
            profile,
            repos,
            cursor: 0,
            scroll: 0,
            followed: usize::MAX,
            bar: None,
            bar_drag: Default::default(),
            list_rows: 3,
            focus: Focus::Name,
            hover: None,
            hits: Vec::new(),
            busy: false,
            message: None,
            taken: seed
                .taken_names
                .iter()
                .map(|n| n.trim().to_lowercase())
                .collect(),
            edit: Default::default(),
            ai: seed.ai_enabled.then_some(seed.ai),
            ai_box: None,
            shield: None,
            close: Default::default(),
            pristine: String::new(),
        };
        dialog.pristine = dialog.signature();
        dialog
    }

    fn signature(&self) -> String {
        let checked: Vec<String> = self
            .repos
            .iter()
            .filter(|r| r.checked)
            .map(|r| r.path.display().to_string())
            .collect();
        format!(
            "{}\n{}\n{}",
            self.name,
            self.description,
            checked.join("\n")
        )
    }

    /// The Tab order: the AI row only exists while AI is on.
    fn order(&self) -> Vec<Focus> {
        ORDER
            .iter()
            .copied()
            .filter(|f| *f != Focus::Ai || self.ai.is_some())
            .collect()
    }

    fn open_ai_box(&mut self) {
        if let Some(ai) = &self.ai {
            self.ai_box = Some(AiBox::new(ai.clone()));
        }
    }

    fn on_ai_box(&mut self, outcome: AiBoxOutcome) -> Outcome {
        match outcome {
            AiBoxOutcome::Stay => {}
            AiBoxOutcome::Cancel => self.ai_box = None,
            AiBoxOutcome::Apply(ai) => {
                self.ai = Some(ai);
                self.ai_box = None;
                self.message = None;
            }
        }
        Outcome::Stay
    }

    fn selected_count(&self) -> usize {
        self.repos.iter().filter(|r| r.checked).count()
    }

    /// A repository chosen with Browse: added (or just checked) and shown.
    pub(crate) fn add_repo(&mut self, path: &Path) {
        match self
            .repos
            .iter()
            .position(|r| gitpoulpi_session::same_repo_path(&r.path, path))
        {
            Some(i) => {
                self.repos[i].checked = true;
                self.cursor = i;
            }
            None => {
                self.repos.push(RepoChoice {
                    path: path.to_path_buf(),
                    name: name_of(path),
                    checked: true,
                    missing: false,
                });
                self.cursor = self.repos.len() - 1;
            }
        }
        self.focus = Focus::Repos;
        self.message = None;
    }

    /// The draft the form describes, or why it cannot be saved.
    pub(crate) fn draft(&self) -> Result<Draft, String> {
        let name = self.name.trim();
        if name.is_empty() {
            return Err(m::NAME_EMPTY.get().into());
        }
        if self.taken.iter().any(|t| *t == name.to_lowercase()) {
            return Err(m::name_taken(name));
        }
        let color = self
            .color
            .selected()
            .filter(|c| *c != "accent")
            .map(str::to_string);
        let profile = self
            .profile
            .selected()
            .filter(|p| !p.is_empty())
            .map(str::to_string);
        Ok(Draft {
            id: self.id.clone(),
            name: name.to_string(),
            color,
            description: self.description.clone(),
            profile,
            repos: self
                .repos
                .iter()
                .filter(|r| r.checked)
                .map(|r| r.path.clone())
                .collect(),
            ai: self.ai.clone(),
        })
    }

    fn submit(&mut self) -> Outcome {
        if self.busy {
            return Outcome::Stay;
        }
        match self.draft() {
            Ok(draft) => Outcome::Submit(draft),
            Err(why) => {
                self.message = Some(why);
                Outcome::Stay
            }
        }
    }

    fn open_combo(&mut self) -> Option<&mut Combo> {
        [&mut self.color, &mut self.profile]
            .into_iter()
            .find(|c| c.is_open())
    }

    fn combo_of(&mut self, focus: Focus) -> Option<&mut Combo> {
        match focus {
            Focus::Color => Some(&mut self.color),
            Focus::Profile => Some(&mut self.profile),
            _ => None,
        }
    }

    fn text_of(&mut self, focus: Focus) -> Option<&mut String> {
        match focus {
            Focus::Name => Some(&mut self.name),
            Focus::Description => Some(&mut self.description),
            _ => None,
        }
    }

    fn toggle(&mut self, row: usize) {
        if let Some(r) = self.repos.get_mut(row) {
            r.checked = !r.checked;
            self.message = None;
        }
    }

    fn keep_cursor_visible(&mut self) {
        let rows = self.list_rows.max(1);
        if self.cursor < self.scroll {
            self.scroll = self.cursor;
        } else if self.cursor >= self.scroll + rows {
            self.scroll = self.cursor + 1 - rows;
        }
    }

    pub(crate) fn paste(&mut self, text: &str) {
        if let Some(b) = self.ai_box.as_mut() {
            b.paste(text);
            return;
        }
        if let Some(c) = self.open_combo() {
            c.paste(text);
            return;
        }
        let clean: String = text.chars().filter(|c| !c.is_control()).collect();
        let focus = self.focus;
        self.edit.sync(Some(focus));
        if let Some(field) = self.text_of(focus) {
            let mut value = std::mem::take(field);
            self.edit.caret_mut(focus).insert(&mut value, &clean);
            *self.text_of(focus).expect("same field") = value;
        }
    }

    pub(crate) fn on_key(&mut self, key: KeyEvent) -> Outcome {
        // The AI instructions popup is modal over the form.
        if let Some(b) = self.ai_box.as_mut() {
            let outcome = b.on_key(key);
            return self.on_ai_box(outcome);
        }
        // An open popup is modal: it takes every key.
        if let Some(c) = self.open_combo() {
            c.on_key(key);
            self.message = None;
            return Outcome::Stay;
        }
        let focus = self.focus;
        let shift = key.modifiers.contains(KeyModifiers::SHIFT);
        // The text fields' editing keys, through the shared core.
        let typing = matches!(focus, Focus::Name | Focus::Description);
        self.edit.sync(typing.then_some(focus));
        if typing {
            let mut value = std::mem::take(self.text_of(focus).expect("a text field"));
            let result = self.edit.dialog_key(focus, &mut value, &key);
            *self.text_of(focus).expect("same field") = value;
            if let Some((edit, copied)) = result {
                if edit == crate::text_edit::KeyEdit::Edited {
                    self.message = None;
                }
                if copied.is_some() {
                    self.message = copied;
                }
                return Outcome::Stay;
            }
        }
        // A focused, closed combo opens on Enter / Space / Down.
        if let Some(c) = self.combo_of(focus) {
            if !matches!(key.code, KeyCode::Tab | KeyCode::BackTab | KeyCode::Up)
                && c.on_key(key) == ComboOutcome::Handled
            {
                self.message = None;
                return Outcome::Stay;
            }
        }
        if focus == Focus::Repos {
            match key.code {
                KeyCode::Up if self.cursor > 0 => {
                    self.cursor -= 1;
                    self.keep_cursor_visible();
                    return Outcome::Stay;
                }
                KeyCode::Down if self.cursor + 1 < self.repos.len() => {
                    self.cursor += 1;
                    self.keep_cursor_visible();
                    return Outcome::Stay;
                }
                KeyCode::Char(' ') => {
                    self.toggle(self.cursor);
                    return Outcome::Stay;
                }
                KeyCode::Home => {
                    self.cursor = 0;
                    self.keep_cursor_visible();
                    return Outcome::Stay;
                }
                KeyCode::End => {
                    self.cursor = self.repos.len().saturating_sub(1);
                    self.keep_cursor_visible();
                    return Outcome::Stay;
                }
                _ => {}
            }
        }
        match key.code {
            KeyCode::Esc => return Outcome::Cancel,
            KeyCode::Tab | KeyCode::BackTab | KeyCode::Down | KeyCode::Up => {
                let forward =
                    matches!(key.code, KeyCode::Down) || (key.code == KeyCode::Tab && !shift);
                let order = self.order();
                let at = order.iter().position(|f| *f == focus).unwrap_or(0);
                let next = if forward {
                    (at + 1) % order.len()
                } else {
                    (at + order.len() - 1) % order.len()
                };
                self.focus = order[next];
            }
            _ if crate::keymap::current().is(crate::keymap::Action::DialogBrowse, &key) => {
                return Outcome::Browse
            }
            KeyCode::Enter => {
                return match focus {
                    Focus::Browse => Outcome::Browse,
                    Focus::Cancel => Outcome::Cancel,
                    Focus::Ai => {
                        self.open_ai_box();
                        Outcome::Stay
                    }
                    Focus::Repos => {
                        self.toggle(self.cursor);
                        Outcome::Stay
                    }
                    _ => self.submit(),
                }
            }
            KeyCode::Char(' ')
                if matches!(
                    focus,
                    Focus::Browse | Focus::Cancel | Focus::Save | Focus::Ai
                ) =>
            {
                return match focus {
                    Focus::Browse => Outcome::Browse,
                    Focus::Cancel => Outcome::Cancel,
                    Focus::Ai => {
                        self.open_ai_box();
                        Outcome::Stay
                    }
                    _ => self.submit(),
                }
            }
            _ => {}
        }
        Outcome::Stay
    }

    fn field_mouse(&mut self, mouse: MouseEvent) -> bool {
        let focus = self.focus;
        let typing = matches!(focus, Focus::Name | Focus::Description);
        self.edit.sync(typing.then_some(focus));
        if typing {
            let value = std::mem::take(self.text_of(focus).expect("a text field"));
            let used =
                self.edit
                    .dialog_mouse(Some(focus), &value, mouse.kind, mouse.column, mouse.row);
            *self.text_of(focus).expect("same field") = value;
            used
        } else {
            self.edit
                .dialog_mouse(None, "", mouse.kind, mouse.column, mouse.row)
        }
    }

    pub(crate) fn on_mouse(&mut self, mouse: MouseEvent) -> Outcome {
        if let Some(b) = self.ai_box.as_mut() {
            let outcome = b.on_mouse(mouse);
            return self.on_ai_box(outcome);
        }
        // An open popup is modal: it takes every mouse event.
        if let Some(c) = self.open_combo() {
            let covered = c.popup_rect();
            c.on_mouse(mouse);
            let closed = self.color.is_open() || self.profile.is_open();
            if matches!(mouse.kind, MouseEventKind::Down(_)) && !closed {
                self.shield = Some((covered, std::time::Instant::now()));
            }
            return Outcome::Stay;
        }
        if let Some((shield, at)) = self.shield {
            let inside = shield.contains((mouse.column, mouse.row).into());
            match mouse.kind {
                _ if at.elapsed() > SHIELD => self.shield = None,
                MouseEventKind::Down(_) | MouseEventKind::Up(_) | MouseEventKind::Drag(_)
                    if inside =>
                {
                    return Outcome::Stay
                }
                _ if inside => {}
                _ => self.shield = None,
            }
        }
        // The list's scrollbar takes its presses and the drag of its thumb.
        if let Some(top) = self
            .bar_drag
            .on_mouse(self.bar, mouse, self.repos.len(), self.scroll)
        {
            self.scroll = top;
            return Outcome::Stay;
        }
        let hit = dialog_kit::hit_at(&self.hits, mouse.column, mouse.row);
        if !matches!(mouse.kind, MouseEventKind::Down(_)) && self.field_mouse(mouse) {
            return Outcome::Stay;
        }
        match mouse.kind {
            MouseEventKind::Moved | MouseEventKind::Drag(_) => self.hover = hit,
            MouseEventKind::ScrollDown if hit.is_some() => {
                let max = self.repos.len().saturating_sub(self.list_rows);
                self.scroll = (self.scroll + 3).min(max);
            }
            MouseEventKind::ScrollUp if hit.is_some() => {
                self.scroll = self.scroll.saturating_sub(3);
            }
            MouseEventKind::Down(MouseButton::Left) => match hit {
                Some(Hit::Row(i)) => {
                    self.focus = Focus::Repos;
                    self.cursor = i;
                    self.toggle(i);
                }
                Some(Hit::Part(part)) => {
                    self.focus = part;
                    match part {
                        Focus::Color | Focus::Profile => {
                            if let Some(c) = self.combo_of(part) {
                                c.on_mouse(mouse);
                            }
                        }
                        Focus::Name | Focus::Description => {
                            self.field_mouse(mouse);
                        }
                        Focus::Browse => return Outcome::Browse,
                        Focus::Cancel => return Outcome::Cancel,
                        Focus::Save => return self.submit(),
                        Focus::Ai => self.open_ai_box(),
                        Focus::Repos => {}
                    }
                }
                None => {}
            },
            _ => {}
        }
        Outcome::Stay
    }

    pub(crate) fn render(&mut self, rect: Rect, buf: &mut Buffer) {
        self.edit
            .sync(matches!(self.focus, Focus::Name | Focus::Description).then_some(self.focus));
        self.edit.begin_frame();
        let title = if self.id.is_some() {
            m::DIALOG_TITLE_EDIT.get()
        } else {
            m::DIALOG_TITLE_NEW.get()
        };
        self.close.framed(buf, rect, title, &WIN, true);
        let mut hits: Vec<(Rect, Hit)> = Vec::new();
        let inner = Rect::new(
            rect.x + 2,
            rect.y + 1,
            rect.width.saturating_sub(4),
            rect.height.saturating_sub(2),
        );
        if inner.width < 36 || inner.height < 13 {
            self.hits = hits;
            return;
        }
        let muted = Style::default().fg(th().ui.muted).bg(th().ui.panel);
        let label_w: u16 = [
            m::FIELD_NAME,
            m::FIELD_COLOR,
            m::FIELD_DESCRIPTION,
            m::FIELD_PROFILE,
            m::FIELD_REPOSITORIES,
            ai_m::FIELD,
        ]
        .iter()
        .map(|msg| crate::display_width(msg.get()) as u16 + 2)
        .max()
        .unwrap_or(14)
        .max(14);
        let field_x = inner.x + label_w;
        let field_w = inner.width.saturating_sub(label_w);

        let mut y = inner.y;
        dialog_kit::put(buf, inner.x, y, m::FIELD_NAME.get(), label_w, muted);
        let name = Rect::new(field_x, y, field_w, 1);
        dialog_kit::text_input(
            buf,
            name,
            &self.name,
            m::NAME_HINT.get(),
            self.focus == Focus::Name,
            self.hover == Some(Hit::Part(Focus::Name)),
            (&self.edit, Focus::Name),
        );
        hits.push((name, Hit::Part(Focus::Name)));

        y += 1;
        dialog_kit::put(buf, inner.x, y, m::FIELD_COLOR.get(), label_w, muted);
        let combo_w = field_w.min(28);
        let color = Rect::new(field_x, y, combo_w, 1);
        let (focused, hovered) = (
            self.focus == Focus::Color,
            self.hover == Some(Hit::Part(Focus::Color)),
        );
        self.color.render_field(buf, color, focused, hovered);
        hits.push((color, Hit::Part(Focus::Color)));

        y += 1;
        dialog_kit::put(buf, inner.x, y, m::FIELD_DESCRIPTION.get(), label_w, muted);
        let description = Rect::new(field_x, y, field_w, 1);
        dialog_kit::text_input(
            buf,
            description,
            &self.description,
            m::DESCRIPTION_HINT.get(),
            self.focus == Focus::Description,
            self.hover == Some(Hit::Part(Focus::Description)),
            (&self.edit, Focus::Description),
        );
        hits.push((description, Hit::Part(Focus::Description)));

        y += 1;
        dialog_kit::put(buf, inner.x, y, m::FIELD_PROFILE.get(), label_w, muted);
        let profile = Rect::new(field_x, y, combo_w, 1);
        let (focused, hovered) = (
            self.focus == Focus::Profile,
            self.hover == Some(Hit::Part(Focus::Profile)),
        );
        self.profile.render_field(buf, profile, focused, hovered);
        hits.push((profile, Hit::Part(Focus::Profile)));

        // The AI instructions: a button and how many texts there are.
        if let Some(ai) = &self.ai {
            y += 1;
            dialog_kit::put(
                buf,
                inner.x,
                y,
                &format!("{} {}", crate::theme::AI_SPARK, ai_m::FIELD.get()),
                label_w,
                muted,
            );
            let edit_label = ai_m::EDIT.get();
            let edit = dialog_kit::button(
                buf,
                field_x,
                y,
                edit_label,
                ButtonState::Normal {
                    hovered: self.hover == Some(Hit::Part(Focus::Ai)),
                    focused: self.focus == Focus::Ai,
                },
            );
            let sets = ai_box::count_sets(ai);
            let summary = if sets == 0 {
                ai_m::NONE.get().to_string()
            } else {
                ai_m::sets(sets as u64)
            };
            dialog_kit::put(
                buf,
                edit.right() + 2,
                y,
                &summary,
                inner.right().saturating_sub(edit.right() + 2),
                muted,
            );
            hits.push((edit, Hit::Part(Focus::Ai)));
        }

        // Repositories: the heading, Browse on its right, then the list.
        y += 2;
        dialog_kit::put(buf, inner.x, y, m::FIELD_REPOSITORIES.get(), label_w, muted);
        dialog_kit::put(
            buf,
            field_x,
            y,
            &m::selected_count(self.selected_count() as u64),
            field_w,
            Style::default().fg(th().ui.text).bg(th().ui.panel),
        );
        let browse_w = dialog_kit::button_width(m::BROWSE_ADD.get());
        let browse = dialog_kit::button(
            buf,
            inner.right().saturating_sub(browse_w),
            y,
            m::BROWSE_ADD.get(),
            ButtonState::Normal {
                hovered: self.hover == Some(Hit::Part(Focus::Browse)),
                focused: self.focus == Focus::Browse,
            },
        );
        hits.push((browse, Hit::Part(Focus::Browse)));

        let footer = inner.bottom().saturating_sub(1);
        let message_y = footer.saturating_sub(1);
        let list_top = y + 1;
        let list_rows = message_y.saturating_sub(list_top) as usize;
        self.list_rows = list_rows.max(1);
        self.scroll = self
            .scroll
            .min(self.repos.len().saturating_sub(self.list_rows));
        if self.followed != self.cursor {
            self.followed = self.cursor;
            self.keep_cursor_visible();
        }
        // The bar rides the dialog's right border, beside the rows.
        self.bar = crate::scrollbar::Track::column(
            Rect::new(rect.right() - 1, list_top, 1, list_rows as u16),
            self.repos.len(),
            list_rows,
            self.scroll,
        );
        if let Some(track) = self.bar {
            crate::scrollbar::render_buf(buf, track, true, false);
        }
        if self.repos.is_empty() {
            dialog_kit::put(
                buf,
                inner.x + 1,
                list_top,
                m::REPOS_EMPTY.get(),
                inner.width,
                muted,
            );
        }
        for (offset, (i, repo)) in self
            .repos
            .iter()
            .enumerate()
            .skip(self.scroll)
            .take(list_rows)
            .enumerate()
        {
            let row_y = list_top + offset as u16;
            let on_cursor = self.focus == Focus::Repos && i == self.cursor;
            let hovered = self.hover == Some(Hit::Row(i));
            let bg = dialog_kit::row_bg(on_cursor, hovered);
            dialog_kit::row_band(buf, Rect::new(inner.x, row_y, inner.width, 1), bg);
            let mark = if repo.checked { "[x]" } else { "[ ]" };
            let name_w = (inner.width / 3).clamp(10, 28);
            let fg = if repo.missing {
                th().ui.muted
            } else {
                th().ui.text
            };
            let mut x = inner.x + 1;
            let mark_rect = dialog_kit::put(
                buf,
                x,
                row_y,
                &format!("{mark} "),
                4,
                Style::default().fg(th().ui.accent).bg(bg),
            );
            crate::pointer_map::hand(Rect::new(mark_rect.x, mark_rect.y, 3, 1));
            x += mark_rect.width;
            let shown = dialog_kit::put(
                buf,
                x,
                row_y,
                &crate::fit(&repo.name, name_w as usize),
                name_w,
                Style::default().fg(fg).bg(bg),
            );
            x = shown.right().max(x + name_w) + 2;
            let tail = if repo.missing {
                format!("{} ({})", repo.path.display(), m::MISSING.get())
            } else {
                repo.path.display().to_string()
            };
            let room = inner.right().saturating_sub(x);
            dialog_kit::put(
                buf,
                x,
                row_y,
                &crate::tail_fit(&tail, room as usize),
                room,
                Style::default().fg(th().ui.muted).bg(bg),
            );
            hits.push((Rect::new(inner.x, row_y, inner.width, 1), Hit::Row(i)));
        }

        if let Some(message) = &self.message {
            dialog_kit::put(
                buf,
                inner.x,
                message_y,
                message,
                inner.width,
                Style::default().fg(th().status.danger).bg(th().ui.panel),
            );
        }

        let save_label = if self.busy {
            m::SAVING.get()
        } else {
            m::BTN_SAVE.get()
        };
        let save_w = dialog_kit::button_width(save_label);
        let cancel_w = dialog_kit::button_width(gitpoulpi_i18n::msg::dialog::CANCEL.get());
        let save_x = inner.right().saturating_sub(save_w);
        let cancel_x = save_x.saturating_sub(cancel_w + 1);
        let cancel = dialog_kit::button(
            buf,
            cancel_x,
            footer,
            gitpoulpi_i18n::msg::dialog::CANCEL.get(),
            ButtonState::Normal {
                hovered: self.hover == Some(Hit::Part(Focus::Cancel)),
                focused: self.focus == Focus::Cancel,
            },
        );
        let save = dialog_kit::button(
            buf,
            save_x,
            footer,
            save_label,
            if self.busy {
                ButtonState::Disabled
            } else {
                ButtonState::Primary {
                    hovered: self.hover == Some(Hit::Part(Focus::Save)),
                    focused: self.focus == Focus::Save,
                }
            },
        );
        // Buttons first, then the rows and fields they never overlap.
        hits.insert(0, (cancel, Hit::Part(Focus::Cancel)));
        hits.insert(0, (save, Hit::Part(Focus::Save)));
        let tab = dialog_kit::key_name(Key::Tab);
        let space = dialog_kit::key_name(Key::Space);
        let enter = dialog_kit::key_name(Key::Enter);
        let browse_key = dialog_kit::key_label(crate::keymap::Action::DialogBrowse);
        dialog_kit::hints(
            buf,
            inner.x,
            footer,
            cancel_x.saturating_sub(inner.x + 1),
            &[
                (tab.as_str(), gitpoulpi_i18n::msg::dialog::HINT_NEXT.get()),
                (
                    space.as_str(),
                    gitpoulpi_i18n::msg::dialog::HINT_TOGGLE.get(),
                ),
                (enter.as_str(), m::HINT_SAVE.get()),
                (
                    browse_key.as_str(),
                    gitpoulpi_i18n::msg::dialog::HINT_BROWSE.get(),
                ),
            ],
        );
        self.hits = hits;
        // The open popup last, over everything else.
        let bounds = buf.area;
        self.color.render_popup(buf, bounds);
        self.profile.render_popup(buf, bounds);
        if let Some(b) = self.ai_box.as_mut() {
            b.render(bounds, buf);
        }
    }
}

impl dialog_kit::Dismiss for WorkspaceDialog {
    fn is_dirty(&self) -> bool {
        self.signature() != self.pristine
    }

    fn close_hit(&self) -> &dialog_kit::CloseHit {
        &self.close
    }

    fn close_hit_mut(&mut self) -> &mut dialog_kit::CloseHit {
        &mut self.close
    }
}

/// The form's size in a frame.
pub(crate) fn rect(frame: Rect) -> Rect {
    WIN.place(
        frame,
        dialog_kit::centered(frame, frame.width.saturating_sub(4).min(86), 22),
    )
}

const WIN: dialog_kit::Win = dialog_kit::Win::movable("workspace");

#[cfg(test)]
mod tests {
    use super::super::ai_box::Part;
    use super::*;
    use crossterm::event::KeyEventKind;

    fn press(code: KeyCode) -> KeyEvent {
        let mut e = KeyEvent::new(code, KeyModifiers::NONE);
        e.kind = KeyEventKind::Press;
        e
    }

    fn type_text(d: &mut WorkspaceDialog, text: &str) {
        for c in text.chars() {
            d.on_key(press(KeyCode::Char(c)));
        }
    }

    fn seed() -> Seed {
        Seed {
            id: None,
            name: String::new(),
            color: None,
            description: String::new(),
            profile: None,
            members: Vec::new(),
            known: vec![
                (PathBuf::from("/nonexistent-gp/api"), "api".into()),
                (PathBuf::from("/nonexistent-gp/web"), "web".into()),
            ],
            profiles: vec![("work".into(), "Work".into())],
            taken_names: vec!["Taken".into()],
            ai_enabled: false,
            ai: WorkspaceAi::default(),
        }
    }

    #[test]
    fn a_name_is_required_and_unique_and_the_draft_carries_what_was_picked() {
        let mut d = WorkspaceDialog::new(seed());
        assert_eq!(d.on_key(press(KeyCode::Enter)), Outcome::Stay);
        assert!(d.message.as_deref().unwrap().contains("name"));
        type_text(&mut d, "taken");
        assert_eq!(d.on_key(press(KeyCode::Enter)), Outcome::Stay);
        assert!(d.message.as_deref().unwrap().contains("taken"));
        for _ in 0..5 {
            d.on_key(press(KeyCode::Backspace));
        }
        type_text(&mut d, "Acme");
        // Down to the Repositories list (Name, Color, Description, Profile,
        // Repos), then check the second row.
        for _ in 0..4 {
            d.on_key(press(KeyCode::Tab));
        }
        assert_eq!(d.focus, Focus::Repos);
        d.on_key(press(KeyCode::Down));
        d.on_key(press(KeyCode::Char(' ')));
        match d.on_key(press(KeyCode::Enter)) {
            Outcome::Stay => {}
            other => panic!("Enter on a repository row toggles it: {other:?}"),
        }
        // Back to Save through Tab and press it.
        d.focus = Focus::Save;
        match d.on_key(press(KeyCode::Enter)) {
            Outcome::Submit(draft) => {
                assert_eq!(draft.name, "Acme");
                assert_eq!(draft.id, None);
                // Space checked `web`, Enter on the row unchecked it again.
                assert!(draft.repos.is_empty(), "{draft:?}");
                assert_eq!(draft.color, None, "the default accent is stored as none");
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn the_color_and_profile_are_comboboxes_and_a_pick_lands_in_the_draft() {
        let mut d = WorkspaceDialog::new(seed());
        type_text(&mut d, "Acme");
        d.focus = Focus::Profile;
        // Enter opens the popup, Down moves to Work, Enter picks it.
        d.on_key(press(KeyCode::Enter));
        assert!(d.profile.is_open());
        d.on_key(press(KeyCode::Down));
        d.on_key(press(KeyCode::Enter));
        assert!(!d.profile.is_open());
        d.focus = Focus::Color;
        d.on_key(press(KeyCode::Enter));
        d.on_key(press(KeyCode::Down));
        d.on_key(press(KeyCode::Enter));
        let draft = d.draft().unwrap();
        assert_eq!(draft.profile.as_deref(), Some("work"));
        assert_eq!(draft.color.as_deref(), Some("green"));
    }

    #[test]
    fn editing_keeps_the_members_checked_and_lists_missing_ones() {
        let mut s = seed();
        s.id = Some("acme".into());
        s.name = "Acme".into();
        s.members = vec![PathBuf::from("/nonexistent-gp/gone")];
        s.profile = Some("deleted".into());
        let mut d = WorkspaceDialog::new(s);
        assert_eq!(d.repos[0].name, "gone");
        assert!(d.repos[0].checked && d.repos[0].missing);
        let draft = d.draft().unwrap();
        assert_eq!(draft.id.as_deref(), Some("acme"));
        assert_eq!(draft.repos, vec![PathBuf::from("/nonexistent-gp/gone")]);
        assert_eq!(draft.profile.as_deref(), Some("deleted"));
        // Browse adds a repository, checked, and a known one is only checked.
        d.add_repo(Path::new("/nonexistent-gp/new"));
        d.add_repo(Path::new("/nonexistent-gp/api"));
        assert_eq!(d.selected_count(), 3);
    }

    #[test]
    fn the_ai_row_exists_only_while_ai_is_on_and_leaves_the_workspace_alone_otherwise() {
        let mut off = WorkspaceDialog::new(seed());
        type_text(&mut off, "Acme");
        let area = Rect::new(0, 0, 100, 30);
        let mut buf = Buffer::empty(area);
        off.render(rect(area), &mut buf);
        let text = dialog_kit::buffer_text(&buf);
        assert!(!text.contains("AI instructions"), "{text}");
        assert!(
            off.draft().unwrap().ai.is_none(),
            "untouched when AI is off"
        );
        assert!(!off.order().contains(&Focus::Ai));

        let mut on = WorkspaceDialog::new(Seed {
            ai_enabled: true,
            ..seed()
        });
        type_text(&mut on, "Acme");
        let mut buf = Buffer::empty(area);
        on.render(rect(area), &mut buf);
        let text = dialog_kit::buffer_text(&buf);
        assert!(
            text.contains("AI instructions") && text.contains("Edit..."),
            "{text}"
        );
        assert!(text.contains("none"), "no instructions yet:\n{text}");
        assert!(on.hits.iter().any(|(_, h)| *h == Hit::Part(Focus::Ai)));
        assert_eq!(on.draft().unwrap().ai, Some(WorkspaceAi::default()));
    }

    #[test]
    fn the_ai_box_edits_the_general_text_and_each_action_and_apply_puts_them_in_the_draft() {
        use gitpoulpi_config::RepoInstructionsMode;
        let mut d = WorkspaceDialog::new(Seed {
            ai_enabled: true,
            ..seed()
        });
        type_text(&mut d, "Acme");
        d.focus = Focus::Ai;
        d.on_key(press(KeyCode::Enter));
        assert!(d.ai_box.is_some(), "Enter on the row opens the popup");
        // The text area has the focus: type the general instructions.
        type_text(&mut d, "Use the Acme glossary.");
        // Tab to the mode list (Scope, Mode) and pick Replace.
        let b = d.ai_box.as_mut().unwrap();
        b.focus = Part::Mode;
        d.on_key(press(KeyCode::Enter));
        d.on_key(press(KeyCode::Down));
        d.on_key(press(KeyCode::Enter));
        // Switch the scope to Code review (the last action) and type its text.
        d.ai_box.as_mut().unwrap().focus = Part::Scope;
        d.on_key(press(KeyCode::Enter));
        for _ in 0..7 {
            d.on_key(press(KeyCode::Down));
        }
        d.on_key(press(KeyCode::Enter));
        d.ai_box.as_mut().unwrap().focus = Part::Text;
        type_text(&mut d, "Flag missing tests.");
        // The box applies and the form keeps the whole set.
        d.ai_box.as_mut().unwrap().focus = Part::Apply;
        d.on_key(press(KeyCode::Enter));
        assert!(d.ai_box.is_none());
        let ai = d.draft().unwrap().ai.unwrap();
        assert_eq!(ai.general.text, "Use the Acme glossary.");
        assert_eq!(ai.general.mode, RepoInstructionsMode::Replace);
        let review = &ai.actions["code_review"];
        assert_eq!(review.text, "Flag missing tests.");
        assert_eq!(review.mode, RepoInstructionsMode::Add);
        assert_eq!(ai_box::count_sets(&ai), 2);
    }

    #[test]
    fn cancelling_the_ai_box_changes_nothing() {
        let mut d = WorkspaceDialog::new(Seed {
            ai_enabled: true,
            ..seed()
        });
        d.focus = Focus::Ai;
        d.on_key(press(KeyCode::Enter));
        type_text(&mut d, "scrap this");
        d.on_key(press(KeyCode::Esc));
        assert!(d.ai_box.is_none());
        assert_eq!(d.ai, Some(WorkspaceAi::default()));
    }

    /// The pointing hand over every repository row (not only the first, which
    /// is the only one `Ctrl+N` badges), the buttons and the colour box; the
    /// arrow over the text fields.
    #[test]
    fn the_rows_and_buttons_are_hands_and_the_text_fields_are_arrows() {
        use crate::pointer_map::{self, Kind};
        let mut d = WorkspaceDialog::new(seed());
        let area = Rect::new(0, 0, 100, 30);
        let mut buf = Buffer::empty(area);
        pointer_map::begin_frame();
        d.render(rect(area), &mut buf);
        let part = |focus| {
            d.hits
                .iter()
                .find(|(_, h)| *h == Hit::Part(focus))
                .map(|(r, _)| *r)
                .unwrap()
        };
        for field in [Focus::Name, Focus::Description] {
            let r = part(field);
            for dx in [0, 1, r.width / 2] {
                assert_eq!(
                    pointer_map::at(r.x + dx, r.y),
                    Some(Kind::Field),
                    "{field:?}"
                );
            }
        }
        let save = part(Focus::Save);
        assert_eq!(pointer_map::at(save.x + 1, save.y), Some(Kind::Hand));
        let color = part(Focus::Color);
        assert_eq!(pointer_map::at(color.x + 1, color.y), Some(Kind::Hand));
        let second_row = d
            .hits
            .iter()
            .filter(|(_, h)| matches!(h, Hit::Row(_)))
            .nth(1)
            .map(|(r, _)| *r)
            .unwrap();
        // The row is the arrow and lit; its checkbox mark is the hand.
        assert!(!d.pointer_hand_at(second_row.x + 8, second_row.y));
        assert_eq!(
            pointer_map::at(second_row.x + 8, second_row.y),
            Some(Kind::Plain)
        );
        assert_eq!(
            pointer_map::at(second_row.x + 2, second_row.y),
            Some(Kind::Hand),
            "the row's checkbox"
        );
        assert!(!d.pointer_hand_at(0, 0));
    }

    #[test]
    fn the_form_paints_its_fields_and_registers_a_hit_for_each() {
        let mut d = WorkspaceDialog::new(seed());
        let area = Rect::new(0, 0, 100, 30);
        let mut buf = Buffer::empty(area);
        d.render(rect(area), &mut buf);
        let text = dialog_kit::buffer_text(&buf);
        for needle in [
            "New workspace",
            "Name",
            "Color",
            "Description",
            "Profile",
            "Repositories",
            "[ ] api",
            "[ ] web",
            "Save",
            "Cancel",
        ] {
            assert!(text.contains(needle), "{needle}\n{text}");
        }
        for focus in [Focus::Name, Focus::Color, Focus::Profile, Focus::Save] {
            assert!(
                d.hits.iter().any(|(_, h)| *h == Hit::Part(focus)),
                "{focus:?}"
            );
        }
        assert_eq!(
            d.hits
                .iter()
                .filter(|(_, h)| matches!(h, Hit::Row(_)))
                .count(),
            2
        );
    }

    fn many_repos(n: usize) -> WorkspaceDialog {
        let mut s = seed();
        s.known = (0..n)
            .map(|i| {
                (
                    PathBuf::from(format!("/nonexistent-gp/r{i:02}")),
                    format!("r{i:02}"),
                )
            })
            .collect();
        WorkspaceDialog::new(s)
    }

    fn mouse(kind: MouseEventKind, col: u16, row: u16) -> MouseEvent {
        MouseEvent {
            kind,
            column: col,
            row,
            modifiers: KeyModifiers::NONE,
        }
    }

    #[test]
    fn the_repository_list_draws_a_bar_only_while_it_overflows() {
        let area = Rect::new(0, 0, 100, 30);
        let mut few = many_repos(2);
        let mut buf = Buffer::empty(area);
        few.render(rect(area), &mut buf);
        assert!(few.bar.is_none());

        let mut long = many_repos(60);
        let mut buf = Buffer::empty(area);
        long.render(rect(area), &mut buf);
        let track = long.bar.expect("60 repositories overflow");
        assert_eq!(buf[(track.rect.x, track.rect.y)].symbol(), "\u{2191}");
        assert_eq!(
            buf[(track.rect.x, track.rect.bottom() - 1)].symbol(),
            "\u{2193}"
        );
    }

    #[test]
    fn the_bar_drags_the_wheel_scrolls_and_the_window_stays_until_the_cursor_moves() {
        let area = Rect::new(0, 0, 100, 30);
        let mut d = many_repos(60);
        let mut buf = Buffer::empty(area);
        d.render(rect(area), &mut buf);
        let track = d.bar.unwrap();
        let x = track.rect.x;
        let max = 60 - d.list_rows;
        d.on_mouse(mouse(
            MouseEventKind::Down(MouseButton::Left),
            x,
            track.rect.y + track.thumb_start,
        ));
        d.on_mouse(mouse(
            MouseEventKind::Drag(MouseButton::Left),
            x,
            track.rect.bottom() + 3,
        ));
        d.on_mouse(mouse(MouseEventKind::Up(MouseButton::Left), x, 0));
        assert_eq!(d.scroll, max, "dragged to the end");
        assert_eq!(d.cursor, 0, "the cursor did not move");
        // The next paints keep the window where it was put.
        d.render(rect(area), &mut Buffer::empty(area));
        d.render(rect(area), &mut Buffer::empty(area));
        assert_eq!(d.scroll, max);
        // The wheel over the list scrolls it back.
        let row = d
            .hits
            .iter()
            .find(|(_, h)| matches!(h, Hit::Row(_)))
            .unwrap()
            .0;
        d.on_mouse(mouse(MouseEventKind::ScrollUp, row.x + 2, row.y));
        d.render(rect(area), &mut Buffer::empty(area));
        assert_eq!(d.scroll, max - 3);
        // A key that moves the cursor brings the window back to it.
        d.focus = Focus::Repos;
        d.on_key(press(KeyCode::Down));
        d.render(rect(area), &mut Buffer::empty(area));
        assert!(d.scroll <= d.cursor && d.cursor < d.scroll + d.list_rows);
    }
}
