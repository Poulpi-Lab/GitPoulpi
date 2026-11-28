//! The menus behind the status bar's chips: encoding, line breaks,
//! indentation (what Notepad++ spreads over its Encoding menu and status
//! bar, VS Code puts on its status bar).
//!
//! Each is a [`Combo`] popup (maintainer rule: a dropdown, never click-to-cycle)
//! hung under its chip, or under the bar's right end when the menu was
//! opened from the palette and the chip is hidden. While one is open it owns
//! the keyboard and the pointer (`Editor::on_key`, `Editor::menu_mouse`,
//! and the event loop's mouse prelude, which also keeps everything behind it
//! from hovering). Choosing a row acts at once and closes the menu; the
//! encoding menu is two steps (reopen or convert, then which encoding).

use super::*;
use crate::combo::{Combo, ComboOption, ComboOutcome};
use crate::editor_charset::{self as charset, Charset, Eol};
use crossterm::event::MouseEvent;

pub(crate) use super::vocab::StatusMenu;

/// The open menu.
pub(super) struct OpenMenu {
    kind: StatusMenu,
    combo: Combo,
}

impl OpenMenu {
    /// The chip this menu hangs from (it stays lit while the menu is up).
    pub(super) fn chip_hit(&self) -> Option<EditorHit> {
        CHIPS
            .iter()
            .find(|(k, _)| k.chip_slot() == self.kind.chip_slot())
            .map(|(_, h)| *h)
    }
}

/// What the Save / Discard / Cancel prompt is about to do once answered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(super) enum Then {
    #[default]
    Close,
    /// Read the file again in this encoding.
    Reopen(Charset),
    /// Open the file just selected in the right pane instead.
    Follow,
}

/// The chips, in the order they sit on the bar.
pub(super) const CHIPS: [(StatusMenu, EditorHit); 3] = [
    (StatusMenu::Encoding, EditorHit::EncodingChip),
    (StatusMenu::LineEndings, EditorHit::EolChip),
    (StatusMenu::Indentation, EditorHit::IndentChip),
];

impl StatusMenu {
    fn chip_slot(self) -> usize {
        match self {
            StatusMenu::Encoding | StatusMenu::Reopen | StatusMenu::Convert => 0,
            StatusMenu::LineEndings => 1,
            StatusMenu::Indentation => 2,
        }
    }
}

fn eol_value(eol: Eol) -> &'static str {
    match eol {
        Eol::Lf => "lf",
        Eol::Crlf => "crlf",
        Eol::Cr => "cr",
        Eol::Mixed => "mixed",
    }
}

fn eol_label(eol: Eol) -> String {
    match eol {
        Eol::Mixed => msg::EOL_MIXED.get().to_string(),
        other => other.code().unwrap_or_default().to_string(),
    }
}

impl Editor {
    /// The eol chip's text: `LF`, `CRLF`, `CR` or the translated `Mixed`.
    pub(super) fn eol_chip_text(&self) -> String {
        eol_label(self.doc.eol())
    }

    /// Whether a status menu is up.
    pub(crate) fn menu_open(&self) -> bool {
        self.menu.is_some()
    }

    /// Opens a status menu (a click on its chip, or the palette), hung
    /// under the chip when the bar shows it.
    pub(crate) fn open_menu(&mut self, kind: StatusMenu) {
        let mut combo = self.menu_combo(kind);
        let anchor = self.chips[kind.chip_slot()].unwrap_or_else(|| {
            let bar = self.bar;
            Rect::new(bar.right().saturating_sub(16), bar.y, 16.min(bar.width), 1)
        });
        combo.set_anchor(anchor);
        combo.open();
        self.menu = Some(OpenMenu { kind, combo });
    }

    fn menu_combo(&self, kind: StatusMenu) -> Combo {
        let mut combo = Combo::new("").with_click_picks();
        match kind {
            StatusMenu::Encoding => {
                combo.set_options(vec![
                    ComboOption::new(msg::ENC_REOPEN.get(), "reopen"),
                    ComboOption::new(msg::ENC_CONVERT.get(), "convert"),
                ]);
            }
            StatusMenu::Reopen | StatusMenu::Convert => {
                combo = combo.with_search();
                combo.set_options(
                    charset::ALL
                        .iter()
                        .map(|c| ComboOption::plain(c.name()))
                        .collect(),
                );
                combo.select(Some(self.doc.charset().name().to_string()));
            }
            StatusMenu::LineEndings => {
                combo.set_options(vec![
                    ComboOption::new(msg::EOL_TO_LF.get(), eol_value(Eol::Lf)),
                    ComboOption::new(msg::EOL_TO_CRLF.get(), eol_value(Eol::Crlf)),
                    ComboOption::new(msg::EOL_TO_CR.get(), eol_value(Eol::Cr)),
                ]);
                combo.select(Some(eol_value(self.doc.eol()).to_string()));
            }
            StatusMenu::Indentation => {
                let mut options = vec![
                    ComboOption::new(msg::IND_SPACES.get(), "spaces"),
                    ComboOption::new(msg::IND_TABS.get(), "tabs"),
                ];
                options.extend(
                    (1..=8).map(|n| ComboOption::new(msg::ind_size(n), format!("size:{n}"))),
                );
                options.push(ComboOption::new(msg::IND_TO_SPACES.get(), "to_spaces"));
                options.push(ComboOption::new(msg::IND_TO_TABS.get(), "to_tabs"));
                combo.set_options(options);
                let tabs = matches!(self.doc.indent(), Indent::Tabs);
                combo.set_marks(vec![
                    if tabs { "tabs" } else { "spaces" }.to_string(),
                    format!("size:{}", self.doc.tab_cols()),
                ]);
            }
        }
        combo
    }

    /// A key while a menu is up: the combo takes it all.
    pub(super) fn menu_key(&mut self, key: &KeyEvent) -> EditorOutcome {
        let Some(menu) = self.menu.as_mut() else {
            return EditorOutcome::default();
        };
        let kind = menu.kind;
        let outcome = menu.combo.on_key(*key);
        self.menu_outcome(kind, outcome)
    }

    /// A mouse event while a menu is up (the event loop sends all of them,
    /// the pointer's moves included, so the rows light up under it).
    pub(crate) fn menu_mouse(&mut self, mouse: MouseEvent) -> EditorOutcome {
        let Some(menu) = self.menu.as_mut() else {
            return EditorOutcome::default();
        };
        let kind = menu.kind;
        let outcome = menu.combo.on_mouse(mouse);
        self.menu_outcome(kind, outcome)
    }

    fn menu_outcome(&mut self, kind: StatusMenu, outcome: ComboOutcome) -> EditorOutcome {
        // Closed by Esc, a click away or a pick: the menu is gone.
        if self.menu.as_ref().is_some_and(|m| !m.combo.is_open()) {
            self.menu = None;
        }
        match outcome {
            ComboOutcome::Picked(value) => self.pick(kind, &value),
            _ => EditorOutcome::default(),
        }
    }

    /// Paints the open menu over everything else.
    pub(super) fn render_menu(&mut self, frame: &mut Frame, bounds: Rect) {
        let Some(menu) = self.menu.as_mut() else {
            return;
        };
        let slot = menu.kind.chip_slot();
        if let Some(rect) = self.chips[slot] {
            menu.combo.set_anchor(rect);
        }
        menu.combo.render_popup(frame.buffer_mut(), bounds);
    }

    fn pick(&mut self, kind: StatusMenu, value: &str) -> EditorOutcome {
        let shown = self.shown();
        match kind {
            StatusMenu::Encoding => {
                self.open_menu(if value == "reopen" {
                    StatusMenu::Reopen
                } else {
                    StatusMenu::Convert
                });
                EditorOutcome::default()
            }
            StatusMenu::Reopen => match Charset::from_name(value) {
                Some(cs) => self.reopen_as(cs),
                None => EditorOutcome::default(),
            },
            StatusMenu::Convert => match Charset::from_name(value) {
                Some(cs) => match self.doc.convert_charset(cs) {
                    Ok(true) => EditorOutcome::say(msg::enc_converted(&shown, cs.name())),
                    Ok(false) => EditorOutcome::say(msg::enc_already(&shown, cs.name())),
                    Err(why) => EditorOutcome::say(msg::enc_unencodable(
                        cs.name(),
                        why.count,
                        format!("{:?}", why.ch),
                        why.line,
                    )),
                },
                None => EditorOutcome::default(),
            },
            StatusMenu::LineEndings => {
                let to = match value {
                    "crlf" => Eol::Crlf,
                    "cr" => Eol::Cr,
                    _ => Eol::Lf,
                };
                self.follow = true;
                if self.doc.convert_eol(to) {
                    EditorOutcome::say(msg::eol_converted(eol_label(to)))
                } else {
                    EditorOutcome::say(msg::eol_already(eol_label(to)))
                }
            }
            StatusMenu::Indentation => {
                self.follow = true;
                match value {
                    "spaces" => self.doc.set_indent_tabs(false),
                    "tabs" => self.doc.set_indent_tabs(true),
                    "to_spaces" | "to_tabs" => {
                        let n = self.doc.convert_indent(value == "to_tabs");
                        return EditorOutcome::say(if n == 0 {
                            msg::IND_NOTHING.get().to_string()
                        } else {
                            msg::ind_converted(n)
                        });
                    }
                    other => {
                        if let Some(n) = other.strip_prefix("size:").and_then(|n| n.parse().ok()) {
                            self.doc.set_tab_size(n);
                        }
                    }
                }
                EditorOutcome::say(msg::ind_set(self.doc.indent_label()))
            }
        }
    }

    /// Reads the file's bytes again in `cs`. With unsaved changes it asks
    /// first (Save / Discard / Cancel), since the changes would be lost.
    pub(super) fn reopen_as(&mut self, cs: Charset) -> EditorOutcome {
        if self.doc.is_dirty() {
            self.prompt = true;
            self.then = Then::Reopen(cs);
            return EditorOutcome::default();
        }
        self.reopen_now(cs)
    }

    pub(super) fn reopen_now(&mut self, cs: Charset) -> EditorOutcome {
        let shown = self.shown();
        let bytes = match std::fs::read(&self.abs) {
            Ok(bytes) => bytes,
            Err(e) => return EditorOutcome::say(msg::cannot_read(&shown, e)),
        };
        match Doc::decode_as(&bytes, Some(cs)) {
            Ok(mut doc) => {
                doc.continue_from(&self.doc);
                let mut at = self.doc.cursor().min(doc.text().len());
                while !doc.text().is_char_boundary(at) {
                    at -= 1;
                }
                doc.set_cursor(at, false);
                let name = doc.charset().name();
                self.doc = doc;
                self.stamp = stamp(&self.abs);
                self.overwrite_armed = false;
                self.longest = (u64::MAX, 0);
                self.ruler_cache = None;
                self.find = None;
                self.follow = true;
                EditorOutcome::say(msg::enc_reopened(&shown, name))
            }
            Err(why) => EditorOutcome::say(why.message(&shown)),
        }
    }

    /// The question the prompt asks.
    pub(super) fn prompt_question(&self) -> String {
        match self.then {
            Then::Close | Then::Follow => msg::prompt_question(self.shown()),
            Then::Reopen(cs) => msg::prompt_reopen(self.shown(), cs.name()),
        }
    }

    /// Discard: drop the changes and do what was asked.
    pub(super) fn prompt_discard(&mut self) -> EditorOutcome {
        match std::mem::take(&mut self.then) {
            Then::Close => EditorOutcome::close(),
            Then::Reopen(cs) => {
                self.prompt = false;
                self.reopen_now(cs)
            }
            Then::Follow => {
                self.prompt = false;
                self.follow_answer = Some(FollowAnswer::Switch);
                EditorOutcome::default()
            }
        }
    }

    /// Cancel: forget what was asked.
    pub(super) fn prompt_cancel(&mut self) -> EditorOutcome {
        self.prompt = false;
        if self.then == Then::Follow {
            self.follow_answer = Some(FollowAnswer::Stay);
        }
        self.then = Then::Close;
        EditorOutcome::default()
    }
}
