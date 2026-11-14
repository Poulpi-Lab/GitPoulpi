//! The Settings "Keyboard Shortcuts" page: every registry action, grouped
//! by context, with a search box, a context and a show filter, and a
//! capture popup that rebinds one.
//!
//! A child module of `preferences` (it reads that module's private
//! state: the config, its path, the message line). Saving goes through
//! `gitpoulpi-config`'s `[keybindings]` writers, then the file is
//! reloaded and the effective keymap rebuilt and installed, so the
//! palette, the help overlay and the footers change on the next frame.
//!
//! Keyboard twins of every pointer gesture: `/` the search, `Enter` the
//! capture popup (a double-click), `a` an alternative chord, `Delete`
//! removes the last chord, `u` unbinds, `r` resets the row (its Reset
//! button), `R` resets everything after a confirmation, `Tab` walks the
//! header controls. The popups own the pointer while open; their hit
//! rects come first and are pushed by the paint walk.

use std::time::{Duration, Instant};

use crossterm::event::{KeyEvent, MouseEvent};
use gitpoulpi_config::{
    reset_all_keybindings_in_file, reset_keybinding_in_file, set_keybinding_in_file,
};
use ratatui::buffer::Buffer;

use super::*;
use crate::combo::{Combo, ComboOption, ComboOutcome};
use crate::dialog_kit::{self, ButtonState};
use crate::keymap::conflict::{self, Analysis, Conflict, Level};
use crate::keymap::{self, Action as Bind, Ctx, KeyChord, Keymap, KeymapIssue, ACTIONS};
use gitpoulpi_i18n::msg::keys_page as msg;

const DOUBLE_CLICK: Duration = Duration::from_millis(500);

/// Which control of the page has the keyboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Zone {
    List,
    Search,
    Context,
    Show,
    ResetAll,
}

impl Zone {
    const ORDER: [Zone; 5] = [
        Zone::List,
        Zone::Search,
        Zone::Context,
        Zone::Show,
        Zone::ResetAll,
    ];

    fn step(self, delta: isize) -> Zone {
        let at = Self::ORDER.iter().position(|z| *z == self).unwrap_or(0) as isize;
        Self::ORDER[(at + delta).rem_euclid(Self::ORDER.len() as isize) as usize]
    }
}

/// The "show" filter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Show {
    All,
    Modified,
    Problems,
}

/// What a key or click asks of the screen around the page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Nav {
    Stay,
    Sections,
    Close,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Btn {
    Replace,
    Save,
    TypeIt,
    Cancel,
    ConfirmReset,
}

impl Btn {
    fn label(self) -> &'static str {
        match self {
            Btn::Replace => msg::BTN_REPLACE.get(),
            Btn::Save => msg::BTN_SAVE.get(),
            Btn::TypeIt => msg::BTN_TYPE_IT.get(),
            Btn::Cancel => msg::BTN_CANCEL.get(),
            Btn::ConfirmReset => msg::BTN_RESET_ALL.get(),
        }
    }
}

/// The capture popup's state.
#[derive(Debug, Clone)]
pub(super) struct Capture {
    /// Index into `ACTIONS`.
    action: usize,
    /// Add an alternative chord rather than replace the chords.
    add: bool,
    chord: Option<KeyChord>,
    /// The "Type it instead" field, while it is open.
    typing: Option<String>,
    error: Option<String>,
    analysis: Analysis,
    focus: usize,
}

#[derive(Debug, Clone)]
pub(super) enum Popup {
    Capture(Capture),
    ConfirmReset { focus: usize },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Hit {
    Popup,
    Button(Btn),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ListLine {
    Heading(Ctx),
    Action(usize),
}

pub(super) struct KeybindingsPage {
    pub(super) zone: Zone,
    pub(super) query: String,
    context: Combo,
    show: Combo,
    /// Index into `ACTIONS`.
    pub(super) selected: usize,
    scroll: usize,
    /// The list's scrollbar as last painted, and its thumb drag.
    bar: Option<crate::scrollbar::Track>,
    bar_drag: crate::scrollbar::BarDrag,
    pub(super) popup: Option<Popup>,
    /// The effective keymap this page shows, rebuilt after every write.
    pub(super) km: Keymap,
    conflicts: Vec<Conflict>,
    issues: Vec<KeymapIssue>,
    /// Publish saved keymaps process-wide (always, except in unit tests).
    pub(super) live: bool,
    /// What the last write said (`... saved to <path>`): Settings paints
    /// over the status line, so the page shows it itself.
    notice: Option<String>,
    list_height: usize,
    // Painted this frame: the paint walk's hit rects.
    rows: Vec<(Rect, usize)>,
    resets: Vec<(Rect, usize)>,
    /// The rows' `!` problem marks as last painted: a click shows the
    /// conflicts of that row's action.
    marks: Vec<(Rect, usize)>,
    /// "Show the conflicts of this action" (`c`, or a click on its `!`): the
    /// list holds that action and every action it clashes with, whatever
    /// the other filters say. Esc clears it.
    pub(super) conflicts_of: Option<Bind>,
    search_rect: Rect,
    reset_all_rect: Rect,
    popup_hits: Vec<(Rect, Hit)>,
    /// The close cross of the open popup (confirm / capture).
    popup_close: dialog_kit::CloseHit,
    last_click: Option<(usize, Instant)>,
    /// The `gitpoulpi_i18n::epoch()` the combos' option labels were built for.
    labels_epoch: u64,
}

impl std::fmt::Debug for KeybindingsPage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KeybindingsPage")
            .field("zone", &self.zone)
            .field("selected", &self.selected)
            .finish()
    }
}

impl Default for KeybindingsPage {
    fn default() -> Self {
        Self::new(Keymap::defaults())
    }
}

fn split_id(id: &str) -> (&str, &str) {
    id.split_once('.').unwrap_or(("", id))
}

fn ctx_title(ctx: Ctx) -> &'static str {
    ctx.title().get()
}

/// The translated text of a chord parse error (`ChordError`'s own `Display`
/// stays English: it is also what `config.toml` diagnostics say).
fn chord_error_text(error: &keymap::chord::ChordError) -> String {
    use keymap::chord::ChordError;
    match error {
        ChordError::Empty => msg::CHORD_EMPTY.get().to_string(),
        ChordError::UnknownKey(k) => msg::chord_unknown_key(k),
        ChordError::AltGr => msg::CHORD_ALTGR.get().to_string(),
    }
}

impl KeybindingsPage {
    pub(super) fn new(km: Keymap) -> Self {
        let (context, show) = Self::combos(None, None);
        let mut page = KeybindingsPage {
            zone: Zone::List,
            query: String::new(),
            context,
            show,
            selected: 0,
            scroll: 0,
            bar: None,
            bar_drag: Default::default(),
            popup: None,
            km,
            conflicts: Vec::new(),
            issues: Vec::new(),
            live: !cfg!(test),
            notice: None,
            list_height: 10,
            rows: Vec::new(),
            resets: Vec::new(),
            marks: Vec::new(),
            conflicts_of: None,
            search_rect: Rect::default(),
            reset_all_rect: Rect::default(),
            popup_hits: Vec::new(),
            popup_close: dialog_kit::CloseHit::default(),
            last_click: None,
            labels_epoch: gitpoulpi_i18n::epoch(),
        };
        page.refresh();
        page.selected = page.visible().first().copied().unwrap_or(0);
        page
    }

    /// The two filter dropdowns, labeled in the current language, with the
    /// given values selected (`all` when none).
    fn combos(context: Option<String>, show: Option<String>) -> (Combo, Combo) {
        let mut context_combo = Combo::new(msg::ALL_CONTEXTS.get());
        let mut options = vec![ComboOption::new(msg::ALL_CONTEXTS.get(), "all")];
        options.extend(
            Ctx::ALL
                .iter()
                .map(|c| ComboOption::new(ctx_title(*c), c.name())),
        );
        context_combo.set_options(options);
        context_combo.select(Some(context.unwrap_or_else(|| "all".into())));
        let mut show_combo = Combo::new(msg::SHOW_ALL.get());
        show_combo.set_options(vec![
            ComboOption::new(msg::SHOW_ALL.get(), "all"),
            ComboOption::new(msg::SHOW_MODIFIED.get(), "modified"),
            ComboOption::new(msg::SHOW_PROBLEMS.get(), "problems"),
        ]);
        show_combo.select(Some(show.unwrap_or_else(|| "all".into())));
        (context_combo, show_combo)
    }

    /// After a language or vocabulary switch: relabel the dropdowns (they
    /// store the stable value, so the filters stay as they were).
    pub(super) fn relabel(&mut self) {
        let epoch = gitpoulpi_i18n::epoch();
        if self.labels_epoch == epoch || self.combo_open() {
            return;
        }
        self.labels_epoch = epoch;
        let context = self.context.selected().map(str::to_string);
        let show = self.show.selected().map(str::to_string);
        (self.context, self.show) = Self::combos(context, show);
    }

    /// Recomputes the conflicts and the config issues of the effective
    /// keymap.
    pub(super) fn refresh(&mut self) {
        self.conflicts = self.km.conflicts();
        self.issues = self.km.issues().to_vec();
    }

    pub(super) fn text_focused(&self) -> bool {
        self.popup.is_some() || self.zone == Zone::Search || self.combo_open()
    }

    fn combo_open(&self) -> bool {
        self.context.is_open() || self.show.is_open()
    }

    fn context_filter(&self) -> Option<Ctx> {
        let value = self.context.selected()?;
        Ctx::ALL.iter().copied().find(|c| c.name() == value)
    }

    fn show_filter(&self) -> Show {
        match self.show.selected() {
            Some("modified") => Show::Modified,
            Some("problems") => Show::Problems,
            _ => Show::All,
        }
    }

    fn problem_level(&self, action: Bind) -> Option<Level> {
        self.conflicts
            .iter()
            .filter(|c| (c.action == action || c.other == Some(action)) && c.level != Level::Info)
            .map(|c| c.level)
            .min()
    }

    /// The actions `action` clashes with (errors and warnings, not notes).
    pub(super) fn clashing_with(&self, action: Bind) -> Vec<Bind> {
        let mut out = Vec::new();
        for c in self.conflicts.iter().filter(|c| c.level != Level::Info) {
            let other = if c.action == action {
                c.other
            } else if c.other == Some(action) {
                Some(c.action)
            } else {
                None
            };
            if let Some(o) = other.filter(|o| !out.contains(o)) {
                out.push(o);
            }
        }
        out
    }

    /// Shows the conflicts of the action at `ACTIONS[index]` (or says it has
    /// none).
    pub(super) fn show_conflicts_of(&mut self, index: usize) {
        let action = ACTIONS[index].action;
        if self.problem_level(action).is_none() {
            self.notice = Some(msg::NO_CONFLICTS.get().to_string());
            return;
        }
        self.conflicts_of = Some(action);
        self.selected = index;
        self.scroll = 0;
        self.reselect();
    }

    /// `ACTIONS` indices that pass the filters, grouped by context in the
    /// registry's context order.
    pub(super) fn visible(&self) -> Vec<usize> {
        if let Some(action) = self.conflicts_of {
            let parties = self.clashing_with(action);
            return ACTIONS
                .iter()
                .enumerate()
                .filter(|(_, d)| d.built() && (d.action == action || parties.contains(&d.action)))
                .map(|(i, _)| i)
                .collect();
        }
        let ctx_filter = self.context_filter();
        let show = self.show_filter();
        let mut out = Vec::new();
        for ctx in Ctx::ALL {
            if ctx_filter.is_some_and(|c| c != *ctx) {
                continue;
            }
            for (index, def) in ACTIONS
                .iter()
                .enumerate()
                .filter(|(_, d)| d.ctx == *ctx && d.built())
            {
                let keep = match show {
                    Show::All => true,
                    Show::Modified => self.km.is_modified(def.action),
                    Show::Problems => self.problem_level(def.action).is_some(),
                };
                if keep && matches_query(&self.query, def, &self.km) {
                    out.push(index);
                }
            }
        }
        out
    }

    fn lines(&self) -> Vec<ListLine> {
        let mut out = Vec::new();
        let mut last = None;
        for index in self.visible() {
            let ctx = ACTIONS[index].ctx;
            if last != Some(ctx) {
                out.push(ListLine::Heading(ctx));
                last = Some(ctx);
            }
            out.push(ListLine::Action(index));
        }
        out
    }

    fn move_selection(&mut self, delta: isize) {
        let visible = self.visible();
        if visible.is_empty() {
            return;
        }
        let at = visible
            .iter()
            .position(|i| *i == self.selected)
            .unwrap_or(0) as isize;
        let next = (at + delta).clamp(0, (visible.len() as isize - 1).max(0)) as usize;
        self.selected = visible[next];
        self.keep_in_view();
    }

    fn keep_in_view(&mut self) {
        let lines = self.lines();
        let Some(at) = lines
            .iter()
            .position(|l| *l == ListLine::Action(self.selected))
        else {
            return;
        };
        let height = self.list_height.max(1);
        // The heading above the first action of a group stays with it.
        let top = if at > 0 && matches!(lines[at - 1], ListLine::Heading(_)) {
            at - 1
        } else {
            at
        };
        if top < self.scroll {
            self.scroll = top;
        } else if at >= self.scroll + height {
            self.scroll = at + 1 - height;
        }
    }

    /// After a filter change: keep the selection on a visible row.
    fn reselect(&mut self) {
        let visible = self.visible();
        if !visible.contains(&self.selected) {
            self.selected = visible.first().copied().unwrap_or(self.selected);
        }
        self.scroll = 0;
        self.keep_in_view();
    }

    fn selected_def(&self) -> Option<&'static crate::keymap::action::ActionDef> {
        let def = ACTIONS.get(self.selected)?;
        self.visible().contains(&self.selected).then_some(def)
    }

    fn open_capture(&mut self, add: bool) {
        if self.selected_def().is_none() {
            return;
        }
        self.popup = Some(Popup::Capture(Capture {
            action: self.selected,
            add,
            chord: None,
            typing: None,
            error: None,
            analysis: Analysis::default(),
            focus: 0,
        }));
    }
}

/// Whether `def` matches the search words: each must occur in the label,
/// the id or one of its chords (case-insensitively), so `ctrl+g` finds
/// everything under the prefix and `stage selected` finds the action.
pub(super) fn matches_query(
    query: &str,
    def: &crate::keymap::action::ActionDef,
    km: &Keymap,
) -> bool {
    let words: Vec<String> = query.split_whitespace().map(str::to_lowercase).collect();
    if words.is_empty() {
        return true;
    }
    let chords: Vec<String> = km
        .labels(def.action)
        .into_iter()
        .chain(km.chords_of(def.action).iter().map(|c| c.to_string()))
        .collect();
    // The label in the current language and in English: a French user can
    // still search with the English words (and docs, agents).
    let hay = format!(
        "{} {} {} {}",
        def.label.get(),
        def.label.english(),
        def.id,
        chords.join(" ")
    )
    .to_lowercase();
    words.iter().all(|w| hay.contains(w))
}

impl Capture {
    fn def(&self) -> &'static crate::keymap::action::ActionDef {
        &ACTIONS[self.action]
    }

    /// The chords the action would have after Save.
    fn new_chords(&self, km: &Keymap) -> Vec<KeyChord> {
        let Some(chord) = self.chord else {
            return Vec::new();
        };
        let mut chords = if self.add {
            km.chords_of(self.def().action).to_vec()
        } else {
            Vec::new()
        };
        if !chords.iter().any(|c| c.class() == chord.class()) {
            chords.push(chord);
        }
        chords
    }

    fn set_chord(&mut self, chord: KeyChord, km: &Keymap) {
        self.chord = Some(chord);
        self.error = None;
        self.focus = 0;
        let new = self.new_chords(km);
        self.analysis =
            conflict::analyse(km, conflict::Platform::current(), self.def().action, &new);
    }

    fn buttons(&self) -> Vec<Btn> {
        let mut out = Vec::new();
        if self.chord.is_some() && self.analysis.refusal.is_none() {
            out.push(if self.analysis.replace.is_some() {
                Btn::Replace
            } else {
                Btn::Save
            });
        }
        out.push(Btn::TypeIt);
        out.push(Btn::Cancel);
        out
    }
}

fn chord_texts(chords: &[KeyChord]) -> Vec<String> {
    chords.iter().map(|c| c.to_string()).collect()
}

impl PreferencesState {
    /// Lands on the Keyboard Shortcuts page with the keyboard in its list
    /// (the palette's `App: Keybindings...`).
    pub(crate) fn show_keybindings(&mut self, reset_all: bool) {
        self.section = PrefSection::ALL
            .iter()
            .position(|s| *s == PrefSection::Keybindings)
            .unwrap_or(0);
        self.focus = PrefFocus::Fields;
        self.strip = false;
        self.remember_sub_tab(PrefSection::Keybindings, SubTab::List);
        self.keys = KeybindingsPage::new(Keymap::from_config(&self.config.keybindings));
        if reset_all {
            self.keys.popup = Some(Popup::ConfirmReset { focus: 1 });
        }
    }

    /// The Keyboard & mouse page filtered on `action` (the palette's key
    /// button and Alt+K), its row selected.
    pub(crate) fn show_keybindings_for(&mut self, action: Bind) {
        self.show_keybindings(false);
        if let Some((index, def)) = ACTIONS.iter().enumerate().find(|(_, d)| d.action == action) {
            self.keys.query = def.id.to_string();
            self.keys.selected = index;
            self.keys.reselect();
        }
    }

    /// The keybindings tab is in front.
    pub(super) fn keys_shown(&self) -> bool {
        self.current_section() == PrefSection::Keybindings && self.sub_tab() == SubTab::List
    }

    pub(super) fn keys_active(&self) -> bool {
        self.keys_shown() && self.focus == PrefFocus::Fields
    }

    /// Rebuilds the effective keymap from the reloaded config and, live,
    /// installs it.
    fn keys_reload(&mut self, path: &Path) {
        if let Ok(config) = Config::load(path) {
            self.config.keybindings = config.keybindings;
        }
        self.keys.km = Keymap::from_config(&self.config.keybindings);
        self.keys.refresh();
        if self.keys.live {
            keymap::install(Keymap::from_config(&self.config.keybindings));
        }
        self.keys.reselect_keep();
    }

    /// Writes `chords` for `action` (resetting instead when they are the
    /// defaults), and says so.
    fn keys_write(&mut self, action: Bind, chords: &[KeyChord]) -> Result<(), String> {
        let Some(path) = self.config_file() else {
            return Err(msg::NO_CONFIG_DIR.get().to_string());
        };
        let def = ACTIONS
            .iter()
            .find(|d| d.action == action)
            .expect("registered");
        let (ctx, name) = split_id(def.id);
        let defaults: Vec<KeyChord> = def
            .defaults
            .iter()
            .filter_map(|t| KeyChord::parse(t).ok())
            .collect();
        let class = |l: &[KeyChord]| l.iter().map(|c| c.class()).collect::<Vec<_>>();
        let result = if class(chords) == class(&defaults) {
            reset_keybinding_in_file(&path, ctx, name)
        } else {
            set_keybinding_in_file(&path, ctx, name, &chord_texts(chords))
        };
        result.map_err(|e| msg::write_failed(def.id, e))?;
        self.keys_reload(&path);
        Ok(())
    }

    fn keys_saved(&mut self, what: String) {
        let path = self
            .config_file()
            .map(|p| p.display().to_string())
            .unwrap_or_default();
        let text = msg::saved(what, path);
        self.keys.notice = Some(text.clone());
        self.message = Some(text);
    }

    fn keys_save_capture(&mut self, replace: bool) {
        let Some(Popup::Capture(cap)) = self.keys.popup.clone() else {
            return;
        };
        let new = cap.new_chords(&self.keys.km);
        if new.is_empty() || cap.analysis.refusal.is_some() {
            return;
        }
        let def = cap.def();
        if replace {
            if let Some(other) = cap.analysis.replace {
                let remaining: Vec<KeyChord> = self
                    .keys
                    .km
                    .chords_of(other)
                    .iter()
                    .copied()
                    .filter(|c| !new.iter().any(|n| n.class() == c.class()))
                    .collect();
                if let Err(e) = self.keys_write(other, &remaining) {
                    self.message = Some(e);
                    return;
                }
            }
        }
        match self.keys_write(def.action, &new) {
            Ok(()) => {
                self.keys.popup = None;
                let shown = chord_texts(&new).join(", ");
                self.keys_saved(msg::saved_chords(def.label.get(), shown));
            }
            Err(e) => self.message = Some(e),
        }
    }

    fn keys_reset_row(&mut self) {
        let Some(def) = self.keys.selected_def() else {
            return;
        };
        if !self.keys.km.is_modified(def.action) {
            self.message = Some(msg::already_default(def.label.get()));
            return;
        }
        let defaults: Vec<KeyChord> = def
            .defaults
            .iter()
            .filter_map(|t| KeyChord::parse(t).ok())
            .collect();
        match self.keys_write(def.action, &defaults) {
            Ok(()) => self.keys_saved(msg::reset_one(def.label.get())),
            Err(e) => self.message = Some(e),
        }
    }

    fn keys_reset_all(&mut self) {
        let Some(path) = self.config_file() else {
            self.message = Some(msg::NO_CONFIG_DIR.get().to_string());
            return;
        };
        match reset_all_keybindings_in_file(&path) {
            Ok(()) => {
                self.keys_reload(&path);
                self.keys_saved(msg::ALL_RESET.get().to_string());
            }
            Err(e) => self.message = Some(msg::keybindings_error(e)),
        }
    }

    /// `Delete` (the last chord) and `u` (all of them).
    fn keys_unbind(&mut self, all: bool) {
        let Some(def) = self.keys.selected_def() else {
            return;
        };
        let mut chords = self.keys.km.chords_of(def.action).to_vec();
        if chords.is_empty() {
            self.message = Some(msg::no_key(def.label.get()));
            return;
        }
        if all {
            chords.clear();
        } else {
            chords.pop();
        }
        match self.keys_write(def.action, &chords) {
            Ok(()) => self.keys_saved(msg::unbound_done(def.label.get())),
            Err(e) => self.message = Some(e),
        }
    }

    pub(super) fn keys_key(&mut self, key: &KeyEvent) -> bool {
        self.keys.notice = None;
        match self.keys_key_nav(key) {
            Nav::Stay => {}
            Nav::Sections => self.focus = PrefFocus::Sections,
            Nav::Close => return false,
        }
        true
    }

    fn keys_key_nav(&mut self, key: &KeyEvent) -> Nav {
        if self.keys.popup.is_some() {
            self.keys_popup_key(key);
            return Nav::Stay;
        }
        for which in 0..2 {
            let combo = if which == 0 {
                &mut self.keys.context
            } else {
                &mut self.keys.show
            };
            if combo.is_open() {
                let outcome = combo.on_key(*key);
                self.keys_finish_combo(which, outcome);
                return Nav::Stay;
            }
        }
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        if self.keys.zone == Zone::Search {
            match key.code {
                KeyCode::Esc | KeyCode::Enter | KeyCode::Down => self.keys.zone = Zone::List,
                KeyCode::Tab => self.keys.zone = self.keys.zone.step(1),
                KeyCode::BackTab => self.keys.zone = self.keys.zone.step(-1),
                KeyCode::Backspace => {
                    self.keys.query.pop();
                    self.keys.reselect();
                }
                KeyCode::Char('u') if ctrl => {
                    self.keys.query.clear();
                    self.keys.reselect();
                }
                KeyCode::Char(c) if !ctrl => {
                    self.keys.query.push(c);
                    self.keys.reselect();
                }
                _ => {}
            }
            return Nav::Stay;
        }
        // Ctrl+F is find here too.
        if ctrl && matches!(key.code, KeyCode::Char('f' | 'F')) {
            self.keys.zone = Zone::Search;
            return Nav::Stay;
        }
        match key.code {
            KeyCode::Tab => {
                self.keys.zone = self.keys.zone.step(1);
                return Nav::Stay;
            }
            KeyCode::BackTab => {
                self.keys.zone = self.keys.zone.step(-1);
                return Nav::Stay;
            }
            _ => {}
        }
        match self.keys.zone {
            Zone::Context | Zone::Show => {
                let which = usize::from(self.keys.zone == Zone::Show);
                let combo = if which == 0 {
                    &mut self.keys.context
                } else {
                    &mut self.keys.show
                };
                match key.code {
                    KeyCode::Esc | KeyCode::Up => self.keys.zone = Zone::List,
                    KeyCode::Left | KeyCode::Char('h') => {
                        self.keys.zone = self.keys.zone.step(-1);
                    }
                    KeyCode::Right | KeyCode::Char('l') => {
                        self.keys.zone = self.keys.zone.step(1);
                    }
                    _ => {
                        let outcome = combo.on_key(*key);
                        self.keys_finish_combo(which, outcome);
                    }
                }
                Nav::Stay
            }
            Zone::ResetAll => {
                match key.code {
                    KeyCode::Esc | KeyCode::Up => self.keys.zone = Zone::List,
                    KeyCode::Enter | KeyCode::Char(' ') => {
                        self.keys.popup = Some(Popup::ConfirmReset { focus: 1 });
                    }
                    KeyCode::Left | KeyCode::Char('h') => {
                        self.keys.zone = self.keys.zone.step(-1);
                    }
                    _ => {}
                }
                Nav::Stay
            }
            Zone::Search => Nav::Stay,
            Zone::List => self.keys_list_key(key),
        }
    }

    fn keys_list_key(&mut self, key: &KeyEvent) -> Nav {
        let page = self.keys.list_height.max(1) as isize;
        match key.code {
            KeyCode::Char('/') => self.keys.zone = Zone::Search,
            KeyCode::Down | KeyCode::Char('j') => self.keys.move_selection(1),
            KeyCode::Up | KeyCode::Char('k') => self.keys.move_selection(-1),
            KeyCode::PageDown => self.keys.move_selection(page),
            KeyCode::PageUp => self.keys.move_selection(-page),
            KeyCode::Home => self.keys.move_selection(isize::MIN / 2),
            KeyCode::End => self.keys.move_selection(isize::MAX / 2),
            KeyCode::Enter => self.keys.open_capture(false),
            KeyCode::Char('a') => self.keys.open_capture(true),
            KeyCode::Delete | KeyCode::Backspace => self.keys_unbind(false),
            KeyCode::Char('u') => self.keys_unbind(true),
            KeyCode::Char('r') => self.keys_reset_row(),
            KeyCode::Char('R') => {
                self.keys.popup = Some(Popup::ConfirmReset { focus: 1 });
            }
            KeyCode::Char('c') => self.keys.show_conflicts_of(self.keys.selected),
            KeyCode::Left | KeyCode::Char('h') => return Nav::Sections,
            KeyCode::Esc if self.keys.conflicts_of.is_some() => {
                self.keys.conflicts_of = None;
                self.keys.reselect();
            }
            KeyCode::Esc if !self.keys.query.is_empty() => {
                self.keys.query.clear();
                self.keys.reselect();
            }
            KeyCode::Esc | KeyCode::Char('q') => return Nav::Close,
            _ => {}
        }
        Nav::Stay
    }

    fn keys_finish_combo(&mut self, which: usize, outcome: ComboOutcome) {
        if let ComboOutcome::Picked(value) = outcome {
            let combo = if which == 0 {
                &mut self.keys.context
            } else {
                &mut self.keys.show
            };
            combo.select(Some(value));
            self.keys.reselect();
        }
    }

    fn keys_popup_key(&mut self, key: &KeyEvent) {
        match self.keys.popup.clone() {
            Some(Popup::ConfirmReset { focus }) => match key.code {
                KeyCode::Esc | KeyCode::Char('n' | 'N') => self.keys.popup = None,
                KeyCode::Char('y' | 'Y') => {
                    self.keys.popup = None;
                    self.keys_reset_all();
                }
                KeyCode::Tab | KeyCode::BackTab | KeyCode::Left | KeyCode::Right => {
                    self.keys.popup = Some(Popup::ConfirmReset { focus: 1 - focus });
                }
                KeyCode::Enter => {
                    self.keys.popup = None;
                    if focus == 0 {
                        self.keys_reset_all();
                    }
                }
                _ => {}
            },
            Some(Popup::Capture(mut cap)) => {
                let km_snapshot = self.keys.km.clone();
                let mut done = false;
                if let Some(text) = cap.typing.as_mut() {
                    match key.code {
                        KeyCode::Esc => cap.typing = None,
                        KeyCode::Enter => match KeyChord::parse(text) {
                            Ok(chord) => {
                                cap.typing = None;
                                cap.set_chord(chord, &km_snapshot);
                            }
                            Err(e) => cap.error = Some(msg::bad_chord(text, chord_error_text(&e))),
                        },
                        KeyCode::Backspace => {
                            text.pop();
                        }
                        KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                            text.push(c);
                        }
                        _ => {}
                    }
                } else if key.code == KeyCode::Esc {
                    done = true;
                    self.keys.popup = None;
                } else if cap.chord.is_some()
                    && matches!(
                        key.code,
                        KeyCode::Enter
                            | KeyCode::Tab
                            | KeyCode::BackTab
                            | KeyCode::Left
                            | KeyCode::Right
                    )
                {
                    let buttons = cap.buttons();
                    match key.code {
                        KeyCode::Enter => {
                            let btn = buttons[cap.focus.min(buttons.len() - 1)];
                            self.keys.popup = Some(Popup::Capture(cap));
                            self.keys_activate(btn);
                            return;
                        }
                        KeyCode::Tab | KeyCode::Right => {
                            cap.focus = (cap.focus + 1) % buttons.len()
                        }
                        _ => cap.focus = (cap.focus + buttons.len() - 1) % buttons.len(),
                    }
                } else if let Some(chord) = KeyChord::from_event(key) {
                    cap.set_chord(chord, &km_snapshot);
                }
                if !done {
                    self.keys.popup = Some(Popup::Capture(cap));
                }
            }
            None => {}
        }
    }

    fn keys_activate(&mut self, btn: Btn) {
        match btn {
            Btn::Save => self.keys_save_capture(false),
            Btn::Replace => self.keys_save_capture(true),
            Btn::Cancel => self.keys.popup = None,
            Btn::TypeIt => {
                if let Some(Popup::Capture(cap)) = self.keys.popup.as_mut() {
                    cap.typing = Some(String::new());
                }
            }
            Btn::ConfirmReset => {
                self.keys.popup = None;
                self.keys_reset_all();
            }
        }
    }

    pub(super) fn keys_paste(&mut self, text: &str) {
        let clean: String = text.chars().filter(|c| !c.is_control()).collect();
        if let Some(Popup::Capture(cap)) = self.keys.popup.as_mut() {
            if let Some(typing) = cap.typing.as_mut() {
                typing.push_str(&clean);
            }
        } else if self.keys.zone == Zone::Search {
            self.keys.query.push_str(&clean);
            self.keys.reselect();
        }
    }

    /// Whether the page wants this pointer event (the popup owns all of
    /// them while it is open).
    pub(super) fn keys_wants_mouse(&self, col: u16, row: u16) -> bool {
        self.keys.popup.is_some()
            || self.keys.combo_open()
            || (self.body_area.contains((col, row).into())
                && !self.strip_rect.contains((col, row).into()))
    }

    pub(super) fn keys_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        let event = MouseEvent {
            kind,
            column: col,
            row,
            modifiers: KeyModifiers::NONE,
        };
        // The popups first: their hit rects are the topmost.
        if self.keys.popup.is_some() {
            if kind == MouseEventKind::Down(MouseButton::Left) {
                let hit = dialog_kit::hit_at(&self.keys.popup_hits, col, row);
                // The close cross: Esc's way out (the popup is dropped).
                if self.keys.popup_close.hit(col, row) {
                    self.keys.popup = None;
                    return true;
                }
                match hit {
                    Some(Hit::Button(btn)) => self.keys_activate(btn),
                    Some(Hit::Popup) => {}
                    None => self.keys.popup = None,
                }
            }
            return true;
        }
        for which in 0..2 {
            let combo = if which == 0 {
                &mut self.keys.context
            } else {
                &mut self.keys.show
            };
            if combo.is_open() {
                let outcome = combo.on_mouse(event);
                self.keys_finish_combo(which, outcome);
                return true;
            }
        }
        match kind {
            MouseEventKind::ScrollDown => {
                self.keys.scroll = (self.keys.scroll + 3).min(
                    self.keys
                        .lines()
                        .len()
                        .saturating_sub(self.keys.list_height),
                );
            }
            MouseEventKind::ScrollUp => self.keys.scroll = self.keys.scroll.saturating_sub(3),
            MouseEventKind::Down(MouseButton::Left) => {
                let at = (col, row).into();
                if self.keys.search_rect.contains(at) {
                    self.keys.zone = Zone::Search;
                } else if self.keys.reset_all_rect.contains(at) {
                    self.keys.zone = Zone::ResetAll;
                    self.keys.popup = Some(Popup::ConfirmReset { focus: 1 });
                } else if self.keys.context.on_mouse(event) != ComboOutcome::Ignored {
                    self.keys.zone = Zone::Context;
                } else if self.keys.show.on_mouse(event) != ComboOutcome::Ignored {
                    self.keys.zone = Zone::Show;
                } else if let Some((_, index)) = self
                    .keys
                    .marks
                    .iter()
                    .find(|(r, _)| r.contains(at))
                    .copied()
                {
                    self.keys.zone = Zone::List;
                    self.keys.show_conflicts_of(index);
                } else if let Some((_, index)) = self
                    .keys
                    .resets
                    .iter()
                    .find(|(r, _)| r.contains(at))
                    .copied()
                {
                    self.keys.selected = index;
                    self.keys.zone = Zone::List;
                    self.keys_reset_row();
                } else if let Some((_, index)) =
                    self.keys.rows.iter().find(|(r, _)| r.contains(at)).copied()
                {
                    self.keys.selected = index;
                    self.keys.zone = Zone::List;
                    let double = self
                        .keys
                        .last_click
                        .is_some_and(|(i, t)| i == index && t.elapsed() < DOUBLE_CLICK);
                    self.keys.last_click = Some((index, Instant::now()));
                    if double {
                        self.keys.last_click = None;
                        self.keys.open_capture(false);
                    }
                }
            }
            _ => {}
        }
        true
    }
}

impl KeybindingsPage {
    /// After a reload: keep the selection on something that exists.
    fn reselect_keep(&mut self) {
        if !self.visible().contains(&self.selected) {
            self.reselect();
        }
    }
}

mod render;

#[cfg(test)]
mod tests;

impl KeybindingsPage {
    /// The list scrollbar's share of a mouse event: `Some(top)` when the bar
    /// took it (the list is scrolled there already).
    pub(super) fn bar_mouse(&mut self, event: MouseEvent) -> Option<usize> {
        if self.popup.is_some() || self.combo_open() {
            return None;
        }
        let content = self.lines().len();
        let top = self
            .bar_drag
            .on_mouse(self.bar, event, content, self.scroll)?;
        self.scroll = top;
        Some(top)
    }
}
