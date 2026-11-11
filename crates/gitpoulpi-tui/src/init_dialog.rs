//! The Init dialog: create a new, empty repository and
//! open it as a tab.
//!
//! ```text
//! ╭ Initialize a repository ─────────────────────────────────────╮
//! │  Initialize in   C:\Projets█                    [ Browse… ]  │
//! │  Name            my-project                                  │
//! │  Initial branch  main                                        │
//! │                  general.default_branch_name; empty = git's  │
//! │                                                              │
//! │  Will create C:\Projets\my-project                           │
//! │  Tab next · Enter create               [ Cancel ] [ Create ] │
//! ╰──────────────────────────────────────────────────────────────╯
//! ```
//!
//! Pure: validation, keys, clicks and drawing. Creating the repository
//! is `init_task` (the daemon's `GitBackend::init`, off the input loop),
//! started by the owner (`repo_dialogs`) from [`InitOutcome::Submit`];
//! Browse is the shared folder picker, also opened by the owner.

use crate::th;
use std::path::{Path, PathBuf};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use gitpoulpi_core::domain::InitOptions;
use gitpoulpi_i18n::msg;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;

use crate::dialog_kit::{self, ButtonState};
use crate::keymap::chord::Key;

/// The dialog's focusable parts, in `Tab` order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InitFocus {
    Parent,
    Browse,
    Name,
    Branch,
    Create,
    Cancel,
}

const ORDER: [InitFocus; 6] = [
    InitFocus::Parent,
    InitFocus::Browse,
    InitFocus::Name,
    InitFocus::Branch,
    InitFocus::Create,
    InitFocus::Cancel,
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum InitOutcome {
    Stay,
    Cancel,
    /// Open the folder picker, starting here.
    Browse(PathBuf),
    /// Create the repository at this path.
    Submit(PathBuf, InitOptions),
}

#[derive(Debug, Clone)]
pub(crate) struct InitDialog {
    /// The folder the repository goes in (or *is*, with no name).
    pub parent: String,
    /// The new folder's name; empty initializes `parent` itself.
    pub name: String,
    /// `general.default_branch_name` to start with; empty means git's
    /// own `init.defaultBranch`.
    pub branch: String,
    pub focus: InitFocus,
    /// `git init` is running.
    pub busy: bool,
    /// The backend's answer when it failed, or any other notice.
    pub message: Option<String>,
    pub hover: Option<InitFocus>,
    hits: Vec<(Rect, InitFocus)>,
    close: dialog_kit::CloseHit,
    /// What the fields held when the dialog opened (dirty = different).
    pristine: String,
    /// The three fields' caret and selection.
    edit: crate::text_edit::FieldEditor<InitFocus>,
}

impl InitDialog {
    pub(crate) fn new(parent: &Path, default_branch: &str) -> Self {
        let mut dialog = Self {
            parent: parent.display().to_string(),
            name: String::new(),
            branch: default_branch.trim().to_string(),
            focus: InitFocus::Name,
            busy: false,
            message: None,
            hover: None,
            hits: Vec::new(),
            close: Default::default(),
            pristine: String::new(),
            edit: Default::default(),
        };
        dialog.pristine = dialog.signature();
        dialog
    }

    fn signature(&self) -> String {
        format!("{}\n{}\n{}", self.parent, self.name, self.branch)
    }

    /// Where the repository would be created, or why it cannot be.
    pub(crate) fn plan(&self) -> Result<(PathBuf, InitOptions), String> {
        let parent = self.parent.trim();
        if parent.is_empty() {
            return Err(msg::init::CHOOSE_FOLDER.get().into());
        }
        let parent = PathBuf::from(parent);
        if !parent.is_absolute() {
            return Err(msg::init::NOT_ABSOLUTE.get().into());
        }
        let name = self.name.trim();
        if let Some(reason) = folder_name_error(name) {
            return Err(reason);
        }
        let branch = self.branch.trim();
        if !branch.is_empty() {
            if let Some(reason) = branch_name_error(branch) {
                return Err(reason);
            }
        }
        let path = if name.is_empty() {
            parent
        } else {
            parent.join(name)
        };
        Ok((
            path,
            InitOptions {
                bare: false,
                initial_branch: (!branch.is_empty()).then(|| branch.to_string()),
            },
        ))
    }

    /// The shared editor, and the focused text field with its text when
    /// a text field has the keyboard - split so both can be borrowed.
    fn field_parts(
        &mut self,
    ) -> (
        &mut crate::text_edit::FieldEditor<InitFocus>,
        Option<(InitFocus, &mut String)>,
    ) {
        let focus = self.focus;
        let field = match focus {
            InitFocus::Parent => Some(&mut self.parent),
            InitFocus::Name => Some(&mut self.name),
            InitFocus::Branch => Some(&mut self.branch),
            _ => None,
        };
        self.edit.sync(field.is_some().then_some(focus));
        (&mut self.edit, field.map(|f| (focus, f)))
    }

    /// A folder chosen in the picker.
    pub(crate) fn set_parent(&mut self, dir: &Path) {
        self.parent = dir.display().to_string();
        self.focus = InitFocus::Name;
        self.message = None;
    }

    pub(crate) fn paste(&mut self, text: &str) {
        let clean: String = text.chars().filter(|c| !c.is_control()).collect();
        if let (edit, Some((id, field))) = self.field_parts() {
            edit.caret_mut(id).insert(field, &clean);
        }
    }

    fn submit(&mut self) -> InitOutcome {
        if self.busy {
            return InitOutcome::Stay;
        }
        match self.plan() {
            Ok((path, opts)) => InitOutcome::Submit(path, opts),
            Err(reason) => {
                self.message = Some(reason);
                InitOutcome::Stay
            }
        }
    }

    fn browse(&self) -> InitOutcome {
        InitOutcome::Browse(PathBuf::from(self.parent.trim()))
    }

    fn activate(&mut self, part: InitFocus) -> InitOutcome {
        match part {
            InitFocus::Browse => self.browse(),
            InitFocus::Cancel => InitOutcome::Cancel,
            _ => self.submit(),
        }
    }

    pub(crate) fn on_key(&mut self, key: KeyEvent) -> InitOutcome {
        let shift = key.modifiers.contains(KeyModifiers::SHIFT);
        // The fields' editing keys, through the shared core: caret,
        // selection, Ctrl+A/U/K, Ctrl+C/X.
        if let (edit, Some((id, field))) = self.field_parts() {
            if let Some((result, copied)) = edit.dialog_key(id, field, &key) {
                if result == crate::text_edit::KeyEdit::Edited {
                    self.message = None;
                }
                if copied.is_some() {
                    self.message = copied;
                }
                return InitOutcome::Stay;
            }
        }
        match key.code {
            KeyCode::Esc => return InitOutcome::Cancel,
            KeyCode::Tab | KeyCode::BackTab | KeyCode::Down | KeyCode::Up => {
                let forward =
                    matches!(key.code, KeyCode::Down) || (key.code == KeyCode::Tab && !shift);
                let at = ORDER.iter().position(|f| *f == self.focus).unwrap_or(0);
                let next = if forward {
                    (at + 1) % ORDER.len()
                } else {
                    (at + ORDER.len() - 1) % ORDER.len()
                };
                self.focus = ORDER[next];
            }
            _ if crate::keymap::current().is(crate::keymap::Action::DialogBrowse, &key) => {
                return self.browse()
            }
            KeyCode::Enter => return self.activate(self.focus),
            KeyCode::Char(' ') if self.field_parts().1.is_none() => {
                return self.activate(self.focus)
            }
            _ => {}
        }
        InitOutcome::Stay
    }

    /// Hands a mouse event to the fields' editor; see
    /// `FieldEditor::dialog_mouse`.
    fn field_mouse(&mut self, mouse: MouseEvent) -> bool {
        match self.field_parts() {
            (edit, Some((id, field))) => {
                edit.dialog_mouse(Some(id), field, mouse.kind, mouse.column, mouse.row)
            }
            (edit, None) => edit.dialog_mouse(None, "", mouse.kind, mouse.column, mouse.row),
        }
    }

    pub(crate) fn on_mouse(&mut self, mouse: MouseEvent) -> InitOutcome {
        let hit = dialog_kit::hit_at(&self.hits, mouse.column, mouse.row);
        // A drag selecting in a field, and its release.
        if !matches!(mouse.kind, MouseEventKind::Down(_)) && self.field_mouse(mouse) {
            return InitOutcome::Stay;
        }
        match mouse.kind {
            MouseEventKind::Moved | MouseEventKind::Drag(_) => self.hover = hit,
            MouseEventKind::Down(MouseButton::Left) => {
                if let Some(part) = hit {
                    self.focus = part;
                    if matches!(
                        part,
                        InitFocus::Browse | InitFocus::Create | InitFocus::Cancel
                    ) {
                        return self.activate(part);
                    }
                    self.field_mouse(mouse);
                }
            }
            _ => {}
        }
        InitOutcome::Stay
    }

    /// What `Ctrl+N` badges: every field and button.
    pub(crate) fn nav_targets(&self) -> Vec<(Rect, (u16, u16))> {
        dialog_kit::nav_cells(self.hits.iter().map(|(r, _)| *r))
    }

    pub(crate) fn render(&mut self, rect: Rect, buf: &mut Buffer) {
        self.field_parts();
        self.edit.begin_frame();
        self.close
            .framed(buf, rect, msg::init::TITLE.get(), &WIN, true);
        let mut hits = Vec::new();
        let inner = Rect::new(
            rect.x + 2,
            rect.y + 1,
            rect.width.saturating_sub(4),
            rect.height.saturating_sub(2),
        );
        if inner.width < 30 || inner.height < 8 {
            self.hits = hits;
            return;
        }
        // The widest label plus a gap (16 at least), whatever the language.
        let label_w: u16 = [
            msg::init::INITIALIZE_IN,
            msg::init::NAME,
            msg::init::INITIAL_BRANCH,
        ]
        .iter()
        .map(|m| crate::display_width(m.get()) as u16 + 2)
        .max()
        .unwrap_or(16)
        .max(16);
        let label = |buf: &mut Buffer, y: u16, text: &str| {
            dialog_kit::put(
                buf,
                inner.x,
                y,
                text,
                label_w,
                Style::default().fg(th().ui.muted).bg(th().ui.panel),
            );
        };
        let field_x = inner.x + label_w;
        let browse_w = dialog_kit::button_width(msg::dialog::BROWSE.get());
        let field_w = inner.width.saturating_sub(label_w);

        let mut y = inner.y;
        label(buf, y, msg::init::INITIALIZE_IN.get());
        let parent = Rect::new(field_x, y, field_w.saturating_sub(browse_w + 1), 1);
        dialog_kit::text_input(
            buf,
            parent,
            &self.parent,
            msg::init::PARENT_HINT.get(),
            self.focus == InitFocus::Parent,
            self.hover == Some(InitFocus::Parent),
            (&self.edit, InitFocus::Parent),
        );
        let browse = dialog_kit::button(
            buf,
            parent.right() + 1,
            y,
            msg::dialog::BROWSE.get(),
            ButtonState::Normal {
                hovered: self.hover == Some(InitFocus::Browse),
                focused: self.focus == InitFocus::Browse,
            },
        );
        hits.push((browse, InitFocus::Browse));
        hits.push((parent, InitFocus::Parent));

        y += 2;
        label(buf, y, msg::init::NAME.get());
        let name = Rect::new(field_x, y, field_w, 1);
        dialog_kit::text_input(
            buf,
            name,
            &self.name,
            msg::init::NAME_HINT.get(),
            self.focus == InitFocus::Name,
            self.hover == Some(InitFocus::Name),
            (&self.edit, InitFocus::Name),
        );
        hits.push((name, InitFocus::Name));

        y += 2;
        label(buf, y, msg::init::INITIAL_BRANCH.get());
        let branch = Rect::new(field_x, y, field_w, 1);
        dialog_kit::text_input(
            buf,
            branch,
            &self.branch,
            msg::init::BRANCH_HINT.get(),
            self.focus == InitFocus::Branch,
            self.hover == Some(InitFocus::Branch),
            (&self.edit, InitFocus::Branch),
        );
        hits.push((branch, InitFocus::Branch));
        dialog_kit::put(
            buf,
            field_x + 1,
            y + 1,
            msg::init::BRANCH_NOTE.get(),
            field_w.saturating_sub(1),
            Style::default().fg(th().ui.muted).bg(th().ui.panel),
        );

        // What Create would do, or why it cannot.
        let bottom = inner.y + inner.height;
        let plan_y = bottom.saturating_sub(3);
        let plan = self.plan();
        match &plan {
            Ok((path, _)) => {
                let lead = if self.busy {
                    msg::init::CREATING_PATH.get()
                } else {
                    msg::init::WILL_CREATE.get()
                };
                let x = inner.x
                    + dialog_kit::put(
                        buf,
                        inner.x,
                        plan_y,
                        lead,
                        inner.width,
                        Style::default().fg(th().ui.muted).bg(th().ui.panel),
                    )
                    .width;
                let room = inner.right().saturating_sub(x);
                dialog_kit::put(
                    buf,
                    x,
                    plan_y,
                    &crate::tail_fit(&path.display().to_string(), room as usize),
                    room,
                    Style::default().fg(th().ui.text).bg(th().ui.panel),
                );
            }
            Err(reason) => {
                dialog_kit::put(
                    buf,
                    inner.x,
                    plan_y,
                    reason,
                    inner.width,
                    Style::default().fg(th().status.warning).bg(th().ui.panel),
                );
            }
        }
        if let Some(message) = &self.message {
            dialog_kit::put(
                buf,
                inner.x,
                plan_y + 1,
                message,
                inner.width,
                Style::default().fg(th().status.danger).bg(th().ui.panel),
            );
        }

        let footer = bottom.saturating_sub(1);
        let create_label = if self.busy {
            msg::init::CREATING.get()
        } else {
            msg::dialog::CREATE.get()
        };
        let create_w = dialog_kit::button_width(create_label);
        let cancel_w = dialog_kit::button_width(msg::dialog::CANCEL.get());
        let create_x = inner.right().saturating_sub(create_w);
        let cancel_x = create_x.saturating_sub(cancel_w + 1);
        let cancel = dialog_kit::button(
            buf,
            cancel_x,
            footer,
            msg::dialog::CANCEL.get(),
            ButtonState::Normal {
                hovered: self.hover == Some(InitFocus::Cancel),
                focused: self.focus == InitFocus::Cancel,
            },
        );
        let create = dialog_kit::button(
            buf,
            create_x,
            footer,
            create_label,
            if plan.is_err() || self.busy {
                ButtonState::Disabled
            } else {
                ButtonState::Primary {
                    hovered: self.hover == Some(InitFocus::Create),
                    focused: self.focus == InitFocus::Create,
                }
            },
        );
        hits.insert(0, (cancel, InitFocus::Cancel));
        hits.insert(0, (create, InitFocus::Create));
        let browse = dialog_kit::key_label(crate::keymap::Action::DialogBrowse);
        let tab = dialog_kit::key_name(Key::Tab);
        let enter = dialog_kit::key_name(Key::Enter);
        let clear = dialog_kit::ctrl_key('u');
        dialog_kit::hints(
            buf,
            inner.x,
            footer,
            cancel_x.saturating_sub(inner.x + 1),
            &[
                (tab.as_str(), msg::dialog::HINT_NEXT.get()),
                (enter.as_str(), msg::dialog::HINT_CREATE.get()),
                (browse.as_str(), msg::dialog::HINT_BROWSE.get()),
                (clear.as_str(), msg::dialog::HINT_CLEAR_FIELD.get()),
            ],
        );
        self.hits = hits;
    }
}

impl dialog_kit::Dismiss for InitDialog {
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

/// The Init dialog's size in a frame.
pub(crate) fn init_rect(frame: Rect) -> Rect {
    WIN.place(
        frame,
        dialog_kit::centered(frame, frame.width.saturating_sub(4).min(90), 14),
    )
}

const WIN: dialog_kit::Win = dialog_kit::Win::movable("init");

/// Why `name` cannot be a single new folder's name, if it cannot.
pub(crate) fn folder_name_error(name: &str) -> Option<String> {
    if name.is_empty() {
        return None;
    }
    if name == "." || name == ".." {
        return Some(msg::init::not_a_folder_name(format!("{name:?}")));
    }
    if name.contains(['/', '\\']) {
        return Some(msg::init::NAME_IS_ONE_FOLDER.get().into());
    }
    if name
        .chars()
        .any(|c| "<>:\"|?*".contains(c) || c.is_control())
    {
        return Some(msg::init::bad_character(format!("{name:?}")));
    }
    // Windows: a trailing dot or space is silently stripped by the OS
    // (so the folder you asked for is not the one you get), and the MS-DOS
    // device names are reserved whatever the extension -- `NUL`, `NUL.txt`,
    // both name the null device, never a folder.
    if name.ends_with('.') || name.ends_with(' ') {
        return Some(msg::init::trailing(format!("{name:?}")));
    }
    let stem = name.split('.').next().unwrap_or(name);
    const RESERVED: [&str; 22] = [
        "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
        "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ];
    if RESERVED.iter().any(|r| stem.eq_ignore_ascii_case(r)) {
        return Some(msg::init::reserved(format!("{name:?}")));
    }
    None
}

/// Why `name` is not a valid branch name, if it is not - the rules of
/// `git check-ref-format --branch` a user can actually break by typing.
pub(crate) fn branch_name_error(name: &str) -> Option<String> {
    let bad = name.is_empty()
        || name == "@"
        || name.starts_with('-')
        || name.starts_with('/')
        || name.ends_with('/')
        || name.ends_with('.')
        || name.ends_with(".lock")
        || name.contains("..")
        || name.contains("//")
        || name.contains("@{")
        || name.split('/').any(|part| part.starts_with('.'))
        || name
            .chars()
            .any(|c| c.is_whitespace() || c.is_control() || "~^:?*[\\".contains(c));
    bad.then(|| msg::init::bad_branch(format!("{name:?}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn abs(p: &str) -> PathBuf {
        if cfg!(windows) {
            PathBuf::from(format!("C:\\{p}"))
        } else {
            PathBuf::from(format!("/{p}"))
        }
    }

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn it_plans_parent_plus_name_and_carries_the_branch() {
        let mut dialog = InitDialog::new(&abs("work"), "main");
        for c in "proj".chars() {
            dialog.on_key(key(KeyCode::Char(c)));
        }
        let (path, opts) = dialog.plan().unwrap();
        assert_eq!(path, abs("work").join("proj"));
        assert_eq!(opts.initial_branch.as_deref(), Some("main"));
        assert_eq!(
            dialog.on_key(key(KeyCode::Enter)),
            InitOutcome::Submit(abs("work").join("proj"), opts)
        );
    }

    #[test]
    fn an_empty_name_initializes_the_folder_itself_and_an_empty_branch_means_gits_default() {
        let mut dialog = InitDialog::new(&abs("work"), "");
        let (path, opts) = dialog.plan().unwrap();
        assert_eq!(path, abs("work"));
        assert_eq!(opts.initial_branch, None);
        dialog.branch = "  ".into();
        assert_eq!(dialog.plan().unwrap().1.initial_branch, None);
    }

    #[test]
    fn invalid_input_is_refused_with_the_reason_and_submits_nothing() {
        let cases: [(&str, &str, &str, &str); 5] = [
            ("", "x", "main", "choose the folder"),
            ("relative", "x", "main", "full path"),
            ("ABS", "a/b", "main", "single folder"),
            ("ABS", "x", "bad name", "not a valid branch name"),
            ("ABS", "x", "feature..x", "not a valid branch name"),
        ];
        for (parent, name, branch, reason) in cases {
            let mut dialog = InitDialog::new(Path::new(""), branch);
            dialog.parent = if parent == "ABS" {
                abs("work").display().to_string()
            } else {
                parent.to_string()
            };
            dialog.name = name.into();
            let err = dialog.plan().unwrap_err();
            assert!(err.contains(reason), "{parent}/{name}/{branch}: {err}");
            assert_eq!(dialog.on_key(key(KeyCode::Enter)), InitOutcome::Stay);
            assert!(dialog.message.as_deref().unwrap().contains(reason));
        }
    }

    /// The "New folder" field in the picker reuses this
    /// validator, so Windows' reserved device names and the trailing
    /// dot/space the OS silently strips must be caught here too.
    #[test]
    fn windows_reserved_names_and_trailing_dot_or_space_are_refused() {
        for bad in ["CON", "con", "NUL.txt", "COM1", "LPT9", "name.", "name "] {
            assert!(folder_name_error(bad).is_some(), "{bad}");
        }
        for good in ["Console", "NULs", "my.folder", "COM10"] {
            assert!(folder_name_error(good).is_none(), "{good}");
        }
    }

    #[test]
    fn branch_names_follow_gits_rules() {
        for good in ["main", "feature/x", "v1.2", "fix-3"] {
            assert_eq!(branch_name_error(good), None, "{good}");
        }
        for bad in ["-x", "a..b", "a b", "x.lock", "a/.b", "a~1", "@", "end/"] {
            assert!(branch_name_error(bad).is_some(), "{bad}");
        }
    }

    #[test]
    fn tab_walks_every_part_and_ctrl_b_browses_from_the_folder() {
        let mut dialog = InitDialog::new(&abs("work"), "main");
        let mut seen = vec![dialog.focus];
        for _ in 0..ORDER.len() - 1 {
            dialog.on_key(key(KeyCode::Tab));
            seen.push(dialog.focus);
        }
        assert_eq!(seen.len(), ORDER.len());
        assert!(ORDER.iter().all(|f| seen.contains(f)));
        assert_eq!(
            dialog.on_key(KeyEvent::new(KeyCode::Char('b'), KeyModifiers::CONTROL)),
            InitOutcome::Browse(abs("work"))
        );
        dialog.focus = InitFocus::Cancel;
        assert_eq!(dialog.on_key(key(KeyCode::Enter)), InitOutcome::Cancel);
        assert_eq!(dialog.on_key(key(KeyCode::Esc)), InitOutcome::Cancel);
    }

    /// `dialog.browse = "Alt+b"`: the new chord browses, Ctrl+B no longer
    /// does, and the footer names it.
    #[test]
    fn a_rebound_browse_chord_browses_and_shows_in_the_footer() {
        let km = crate::keymap::test_keymap(&[("dialog", "browse", &["Alt+b"])]);
        crate::keymap::with_test_keymap(km, || {
            let mut dialog = InitDialog::new(&abs("work"), "main");
            assert_eq!(
                dialog.on_key(KeyEvent::new(KeyCode::Char('b'), KeyModifiers::CONTROL)),
                InitOutcome::Stay
            );
            assert_eq!(
                dialog.on_key(KeyEvent::new(KeyCode::Char('b'), KeyModifiers::ALT)),
                InitOutcome::Browse(abs("work"))
            );
            let (_, text) = draw(&mut dialog, 100, 30);
            assert!(text.contains("Alt+B browse"), "{text}");
        });
    }

    fn draw(dialog: &mut InitDialog, width: u16, height: u16) -> (Rect, String) {
        let mut buf = Buffer::empty(Rect::new(0, 0, width, height));
        let rect = init_rect(buf.area);
        dialog.render(rect, &mut buf);
        (rect, dialog_kit::buffer_text(&buf))
    }

    #[test]
    fn it_moves_and_arranges_but_does_not_resize() {
        let mut dialog = InitDialog::new(&abs("work"), "main");
        crate::dialog_kit::frame::testing::check_gestures(false, || {
            draw(&mut dialog, 160, 50);
        });
    }

    #[test]
    fn it_renders_at_two_widths_and_every_button_is_where_it_is_hit() {
        for width in [72u16, 140] {
            let mut dialog = InitDialog::new(&abs("work"), "main");
            dialog.name = "proj".into();
            let (rect, text) = draw(&mut dialog, width, 30);
            for needle in [
                "Initialize a repository",
                "Initialize in",
                "Name",
                "Initial branch",
                "Will create",
                "proj",
                "Browse",
                "Create",
                "Cancel",
            ] {
                assert!(text.contains(needle), "{width}: {needle}\n{text}");
            }
            for (hit, _) in &dialog.hits {
                assert!(rect.contains((hit.x, hit.y).into()));
            }
            // A click on Browse browses, on Cancel cancels.
            let at = |dialog: &InitDialog, part| {
                let (r, _) = dialog.hits.iter().find(|(_, p)| *p == part).unwrap();
                MouseEvent {
                    kind: MouseEventKind::Down(MouseButton::Left),
                    column: r.x + 1,
                    row: r.y,
                    modifiers: KeyModifiers::NONE,
                }
            };
            assert_eq!(
                dialog.on_mouse(at(&dialog, InitFocus::Browse)),
                InitOutcome::Browse(abs("work"))
            );
            assert!(matches!(
                dialog.on_mouse(at(&dialog, InitFocus::Create)),
                InitOutcome::Submit(..)
            ));
            assert_eq!(
                dialog.on_mouse(at(&dialog, InitFocus::Cancel)),
                InitOutcome::Cancel
            );
        }
    }

    #[test]
    fn an_invalid_form_says_why_and_grays_the_create_button() {
        let mut dialog = InitDialog::new(&abs("work"), "bad name");
        let (_, text) = draw(&mut dialog, 100, 30);
        assert!(text.contains("not a valid branch name"), "{text}");
    }
}
