//! Prompt variables in the instruction editor: the "Variables" dropdown,
//! the `${` autocomplete popup, the tooltip line and the unknown-variable
//! warning. The names and the expansion live in `gitpoulpi_ai::vars`.

use super::*;
use crate::combo::{Combo, ComboOption, ComboOutcome};
use gitpoulpi_ai::vars::{self, VARIABLES};

/// What the user's caret sits in: a `${prefix` being typed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Completion {
    /// Byte offset of the `$`.
    pub(super) start: usize,
    pub(super) prefix: String,
}

/// The `${prefix` the caret is right after, when there is one (not an
/// escaped `$${`).
pub(super) fn completion_at(text: &str, cursor: usize) -> Option<Completion> {
    let before = &text[..cursor];
    let word = before
        .rfind(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .map_or(0, |i| i + 1);
    let head = &before[..word];
    let dollar = head.strip_suffix('{')?;
    if !dollar.ends_with('$') {
        return None;
    }
    let start = dollar.len() - 1;
    // `$${` is the escape for a literal `${`.
    if dollar[..start].ends_with('$') {
        return None;
    }
    Some(Completion {
        start,
        prefix: before[word..].to_string(),
    })
}

/// The variables a prefix completes to: those starting with it first, then
/// those containing it.
pub(super) fn matching(prefix: &str) -> Vec<&'static str> {
    let p = prefix.to_ascii_lowercase();
    let names = VARIABLES.iter().map(|v| v.name);
    let mut out: Vec<&'static str> = names.clone().filter(|n| n.starts_with(&p)).collect();
    out.extend(names.filter(|n| !n.starts_with(&p) && n.contains(&p)));
    out
}

/// The sentence describing variable `name`.
pub(super) fn description(name: &str) -> &'static str {
    match name {
        "uilang" => msg::VAR_UILANG,
        "repo" => msg::VAR_REPO,
        "branch" => msg::VAR_BRANCH,
        "ref" => msg::VAR_REF,
        "commit" => msg::VAR_COMMIT,
        "commit_subject" => msg::VAR_COMMIT_SUBJECT,
        "date" => msg::VAR_DATE,
        "time" => msg::VAR_TIME,
        "datetime" => msg::VAR_DATETIME,
        "user" => msg::VAR_USER,
        "email" => msg::VAR_EMAIL,
        "remote" => msg::VAR_REMOTE,
        "host" => msg::VAR_HOST,
        "files" => msg::VAR_FILES,
        _ => msg::VAR_ACTION,
    }
    .get()
}

/// "description (e.g. example)" for variable `name`.
pub(super) fn hint(name: &str) -> String {
    let example = VARIABLES
        .iter()
        .find(|v| v.name == name)
        .map_or("", |v| v.example);
    msg::variable_hint(description(name), example)
}

/// The Variables dropdown, the autocomplete popup and their hit rects.
pub(super) struct VarUi {
    pub(super) combo: Combo,
    /// Highlighted row of the autocomplete popup.
    pub(super) ac_cursor: usize,
    /// Esc closed the popup for the `${` at this offset.
    pub(super) dismissed: Option<usize>,
    /// The popup's rows as painted: (rect, index into the matches).
    pub(super) ac_hits: Vec<(Rect, usize)>,
}

impl VarUi {
    pub(super) fn new() -> Self {
        let mut combo = Combo::new(msg::VARIABLES_BUTTON.get());
        combo.set_options(
            VARIABLES
                .iter()
                .map(|v| {
                    ComboOption::new(format!("${{{}}}  {}", v.name, description(v.name)), v.name)
                })
                .collect(),
        );
        Self {
            combo,
            ac_cursor: 0,
            dismissed: None,
            ac_hits: Vec::new(),
        }
    }
}

impl PromptEdit {
    /// The caret's byte offset in the text.
    pub(super) fn cursor(&self) -> usize {
        self.field
            .caret(())
            .map_or(self.text.len(), |c| c.cursor(&self.text))
    }

    /// The autocomplete in effect now: the completion at the caret and its
    /// matches, unless Esc closed it or nothing matches.
    pub(super) fn completion(&self) -> Option<(Completion, Vec<&'static str>)> {
        let c = completion_at(&self.text, self.cursor())?;
        if self.vars.dismissed == Some(c.start) {
            return None;
        }
        let m = matching(&c.prefix);
        (!m.is_empty()).then_some((c, m))
    }

    /// Inserts `${name}` at the caret.
    pub(super) fn insert_variable(&mut self, name: &str) {
        self.field
            .caret_mut(())
            .insert(&mut self.text, &format!("${{{name}}}"));
        self.vars.dismissed = None;
    }

    /// Replaces the `${prefix` being typed by `${name}` (and a `}` that
    /// already follows the caret).
    pub(super) fn accept_completion(&mut self, name: &str) {
        let Some((c, _)) = self.completion() else {
            return;
        };
        let cursor = self.cursor();
        let end = if self.text[cursor..].starts_with('}') {
            cursor + 1
        } else {
            cursor
        };
        let full = format!("${{{name}}}");
        self.text.replace_range(c.start..end, &full);
        self.field
            .caret_mut(())
            .set_cursor(&self.text, c.start + full.len(), false);
        self.vars.ac_cursor = 0;
    }

    /// Unknown `${names}` in the text (for the warning line).
    pub(super) fn unknown(&self) -> Vec<String> {
        vars::unknown_in(&self.text)
    }

    /// A key while the Variables dropdown or the autocomplete is up.
    /// `true` when it was theirs.
    pub(super) fn vars_key(&mut self, key: &KeyEvent) -> bool {
        if self.vars.combo.is_open() {
            if let ComboOutcome::Picked(name) = self.vars.combo.on_key(*key) {
                self.insert_variable(&name);
            }
            return true;
        }
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        if ctrl && matches!(key.code, KeyCode::Char('t' | 'T' | ' ')) {
            self.vars.combo.open();
            return true;
        }
        let Some((_, matches)) = self.completion() else {
            return false;
        };
        let last = matches.len() - 1;
        let at = self.vars.ac_cursor.min(last);
        match key.code {
            KeyCode::Down => self.vars.ac_cursor = (at + 1).min(last),
            KeyCode::Up => self.vars.ac_cursor = at.saturating_sub(1),
            KeyCode::Enter | KeyCode::Tab => self.accept_completion(matches[at]),
            KeyCode::Esc => {
                self.vars.dismissed = self.completion().map(|(c, _)| c.start);
            }
            _ => return false,
        }
        true
    }

    /// Resets the highlight when the typed prefix changed what matches.
    pub(super) fn after_edit(&mut self) {
        self.vars.dismissed = self.vars.dismissed.filter(|d| self.completion_start_is(*d));
        self.vars.ac_cursor = 0;
    }

    fn completion_start_is(&self, start: usize) -> bool {
        completion_at(&self.text, self.cursor()).is_some_and(|c| c.start == start)
    }
}

impl AiPage {
    /// A mouse event for the dropdown and the popup. `true` when handled.
    pub(super) fn vars_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        let Some(edit) = self.prompt.as_mut() else {
            return false;
        };
        let event = crossterm::event::MouseEvent {
            kind,
            column: col,
            row,
            modifiers: KeyModifiers::NONE,
        };
        if edit.vars.combo.is_open() {
            if let ComboOutcome::Picked(name) = edit.vars.combo.on_mouse(event) {
                edit.insert_variable(&name);
            }
            return true;
        }
        if kind == MouseEventKind::Down(MouseButton::Left) {
            let hit = crate::dialog_kit::hit_at(&edit.vars.ac_hits, col, row);
            if let Some(i) = hit {
                if let Some((_, matches)) = edit.completion() {
                    if let Some(name) = matches.get(i) {
                        edit.accept_completion(name);
                    }
                }
                return true;
            }
        }
        edit.vars.combo.on_mouse(event) != ComboOutcome::Ignored
    }
}
