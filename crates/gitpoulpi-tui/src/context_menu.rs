//! A right-click popup menu with submenus, driven identically by the
//! mouse and by the keyboard.
//!
//! # Drill-in, not a flyout
//!
//! A submenu can be a *flyout* (a second popup hanging off the parent row)
//! or a *drill-in*, where choosing `Reset >` replaces the list in place and
//! a `< Back` row (or `Esc`/`Left`) returns. This is a drill-in, for reasons
//! specific to a terminal:
//!
//! 1. **A flyout has to choose a side, and can be wrong.** It opens right
//!    unless that runs off the screen, then flips left, which in an
//!    80-column terminal with a menu anchored wherever the user right-clicked
//!    is wrong often. A drill-in is one rect in the place the menu already
//!    was, sized by `anchored_popup` like the flat menu.
//! 2. **A flyout needs hover tracking to stay honest.** Real flyouts open on
//!    hover, close when the pointer leaves, and need a "triangle" grace area
//!    for diagonal moves. A mechanism whose correctness depends on pointer
//!    *paths* is the wrong thing to add to an app whose mouse affordances
//!    have repeatedly highlighted without acting (`where-we-are.md`).
//! 3. **The hit registration stays one flat list.** `find_hit_target`
//!    returns the first rect containing a point, so two overlapping popups
//!    would need an ordering rule; one list of rows needs none.
//!
//! The price is that the parent list is hidden while a submenu is open. The
//! `< Back` row and the title (`a1b2c3d > Reset`) pay it, and both are mouse-
//! *and* keyboard-reachable (principle 5).
//!
//! # The contract with the caller
//!
//! `Menu` is pure: it holds rows and a selection, answers keys and row
//! activations, and renders itself. It never touches the daemon, the
//! clipboard, or `last_message`. Every outcome the caller must act on
//! comes back as a `MenuOutcome`, so the mouse path and the keyboard path
//! run the *same* `activate`, and cannot drift -- the bug class this
//! module exists to prevent.

use crate::th;
use crate::wide_edge::Clear;
use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};
use ratatui::Frame;

mod scroll;
pub(crate) use scroll::{begin_frame, on_mouse};

use crate::{anchored_popup, display_width, ClippedRender};

/// The `>` at the end of a row that opens a submenu, and the `<` on the
/// row that leaves one.
///
/// U+203A / U+2039 rather than the geometric triangles (U+25B8 and friends):
/// they are General Punctuation, text presentation by definition, and
/// present in every monospace font a Windows terminal falls back to (a
/// character chosen for its shape has sheared the Graph column once).
const SUBMENU_MARK: &str = "\u{203A}";
const BACK_MARK: &str = "\u{2039}";

/// A row's text with its leading glyph (the glyph is code, the text a message).
pub(crate) fn glyph_label(glyph: &str, text: impl std::fmt::Display) -> String {
    format!("{glyph} {text}")
}

/// Whether this key press opens a context menu here: this app's own `.`, or
/// one of the two platform-standard aliases, `Shift+F10` and the Menu/Apps
/// key, the three a mouse right-click is equivalent to everywhere a menu
/// exists.
///
/// Every call site widens its `KeyCode::Char('.')` pattern to
/// `KeyCode::Char('.') | KeyCode::Menu | KeyCode::F(10)` and guards it with
/// `is_menu_key(key.code, key.modifiers)`: widening lets `F(10)` reach the
/// guard, and the guard tells a plain F10 apart from `Shift+F10`.
///
/// **`KeyCode::Menu` is close to unreachable on Windows.** crossterm's
/// Windows console backend has no `VK_APPS` arm, so the physical Apps/Menu
/// key produces no event under classic `conhost`. The ANSI/kitty parser
/// decodes it only once the kitty keyboard protocol is negotiated with
/// `PushKeyboardEnhancementFlags`. The arm is kept for terminals where it
/// does arrive (WezTerm and other kitty-protocol terminals). `Shift+F10`
/// works everywhere: crossterm maps `VK_F10` to `KeyCode::F(10)` and reports
/// `Shift` as an ordinary modifier bit.
pub(crate) fn is_menu_key(code: KeyCode, modifiers: KeyModifiers) -> bool {
    matches!(code, KeyCode::Char('.') | KeyCode::Menu)
        || (code == KeyCode::F(10) && modifiers.contains(KeyModifiers::SHIFT))
}

#[cfg(test)]
mod menu_key_tests {
    use super::*;

    #[test]
    fn dot_shift_f10_and_the_menu_key_are_recognized_plain_f10_is_not() {
        assert!(is_menu_key(KeyCode::Char('.'), KeyModifiers::NONE));
        assert!(is_menu_key(KeyCode::F(10), KeyModifiers::SHIFT));
        assert!(is_menu_key(KeyCode::Menu, KeyModifiers::NONE));
        assert!(!is_menu_key(KeyCode::F(10), KeyModifiers::NONE));
        assert!(!is_menu_key(KeyCode::F(9), KeyModifiers::SHIFT));
        assert!(!is_menu_key(KeyCode::Char('x'), KeyModifiers::NONE));
    }
}

/// Tells two menus apart without making equal menus unequal: ids never
/// take part in `==`.
#[derive(Debug, Clone, Copy)]
struct MenuId(u64);

impl MenuId {
    fn next() -> Self {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        Self(NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed))
    }
}

impl PartialEq for MenuId {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl Eq for MenuId {}

/// One row of a menu: something to do, a list to drill into, or a rule
/// between groups.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Entry<A> {
    Action {
        label: String,
        action: A,
        /// Painted red, and by convention the rows that stop at a confirm
        /// modal. Purely presentational here - the caller decides what
        /// actually confirms.
        destructive: bool,
        /// The registry action this row is the mouse twin of: its
        /// effective binding is drawn right-aligned on the row.
        key: Option<crate::keymap::Action>,
    },
    Submenu {
        label: String,
        items: Vec<Entry<A>>,
    },
    /// A horizontal rule grouping the rows above and below it. It occupies a
    /// screen row (drawn, and the caller registers a rect for it) but can
    /// never be selected or run: `activate` answers `Stay` and the arrows
    /// step over it, since a separator the keyboard can land on is a row that
    /// looks selected and does nothing.
    Separator,
    /// A row that exists but cannot run *here*, grayed out, its label saying
    /// why ("Fast-forward feature to main (diverged, ...)"). Hiding it would
    /// leave the user wondering where the entry went; offering it live would
    /// be a row whose only outcome is an error. Like a separator it can never
    /// be selected or run: the arrows step over it and a click on it is
    /// consumed.
    Disabled {
        label: String,
    },
}

impl<A> Entry<A> {
    pub(crate) fn action(label: impl Into<String>, action: A) -> Self {
        Entry::Action {
            label: label.into(),
            action,
            destructive: false,
            key: None,
        }
    }

    /// Shows `key`'s effective binding at the right of an action row
    /// (nothing when it is unbound; no effect on other rows).
    pub(crate) fn with_key(mut self, action: crate::keymap::Action) -> Self {
        if let Entry::Action { key, .. } = &mut self {
            *key = Some(action);
        }
        self
    }

    pub(crate) fn danger(label: impl Into<String>, action: A) -> Self {
        Entry::Action {
            label: label.into(),
            action,
            destructive: true,
            key: None,
        }
    }

    pub(crate) fn submenu(label: impl Into<String>, items: Vec<Entry<A>>) -> Self {
        Entry::Submenu {
            label: label.into(),
            items,
        }
    }

    /// A grayed-out row whose label says why it cannot run.
    pub(crate) fn disabled(label: impl Into<String>, why: &str) -> Self {
        Entry::Disabled {
            label: format!("{} ({why})", label.into()),
        }
    }

    fn is_selectable(&self) -> bool {
        !matches!(self, Entry::Separator | Entry::Disabled { .. })
    }
}

/// Drops a leading, a trailing or a doubled separator. Menus are built by
/// concatenating optional groups, and an empty group would leave its rule
/// behind; running this once at construction means no builder has to reason
/// about which neighbors happened to be empty.
pub(crate) fn tidy_separators<A>(entries: Vec<Entry<A>>) -> Vec<Entry<A>> {
    let mut out: Vec<Entry<A>> = Vec::with_capacity(entries.len());
    for e in entries {
        if matches!(e, Entry::Separator)
            && (out.is_empty() || matches!(out.last(), Some(Entry::Separator)))
        {
            continue;
        }
        out.push(e);
    }
    while matches!(out.last(), Some(Entry::Separator)) {
        out.pop();
    }
    out
}

/// What the caller must do about a key or a click.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum MenuOutcome<A> {
    /// Handled entirely inside the menu; it stays open.
    Stay,
    /// Run this, then close the menu.
    Run(A),
    /// Close the menu, having done nothing.
    Close,
    /// Not a menu key - the caller's other handlers should see it.
    NotHandled,
}

/// Where the last painted menu was and which of its rows was activated, so a
/// list opened by that row (the "Open with" programs) can hang beside it like a
/// submenu. The menu closes the moment its action runs; this outlives it.
static PAINTED: std::sync::Mutex<Option<Painted>> = std::sync::Mutex::new(None);

#[derive(Debug, Clone, Copy)]
struct Painted {
    popup: Rect,
    /// Rows scrolled out of view above the first drawn one.
    offset: usize,
    /// The screen row last activated.
    row: usize,
}

/// The menu the last frame painted and the row of it that was activated:
/// `(the menu's rect, the screen y of that row)`. `None` before any menu was
/// painted. Read it when the activated row's action runs; it is the parent a
/// flyout list hangs off ([`crate::ui_kit::flyout_rect`]).
pub(crate) fn activated_row() -> Option<(Rect, u16)> {
    let p = (*PAINTED.lock().ok()?)?;
    let y = p.popup.y + 1 + p.row.saturating_sub(p.offset) as u16;
    Some((p.popup, y.min(p.popup.bottom().saturating_sub(2))))
}

/// An open context menu: its rows, where the user is inside them, and
/// where on screen it was opened.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Menu<A> {
    /// What the menu is about - a short SHA, a ref name. A menu whose
    /// rows all say "here" has to say where "here" is.
    caption: String,
    entries: Vec<Entry<A>>,
    /// Which submenu is open, as a path of indices from the root; empty is
    /// the top level. A path rather than an `Option<usize>` means "one level
    /// only" is not an invariant anyone can break by nesting two deep.
    path: Vec<usize>,
    /// The row highlighted for the keyboard, counting the `Back` row when
    /// there is one. Always in range: every mutation clamps.
    selected: usize,
    pub(crate) anchor_col: u16,
    pub(crate) anchor_row: u16,
    /// Longer explanations of grayed-out rows, by the start of their label:
    /// the row itself carries a few words, the status line the rest.
    notes: Vec<(String, String)>,
    /// Whether the frame carries a close cross (`closable`), and where the
    /// last paint put it.
    closable: bool,
    cross: std::cell::Cell<Option<Rect>>,
    /// Which menu this is, for the scroll state of a tall one (`scroll`).
    id: MenuId,
}

impl<A: Clone> Menu<A> {
    pub(crate) fn new(
        caption: impl Into<String>,
        entries: Vec<Entry<A>>,
        anchor_col: u16,
        anchor_row: u16,
    ) -> Self {
        let mut menu = Self {
            caption: caption.into(),
            entries: tidy_separators(entries),
            path: Vec::new(),
            selected: 0,
            anchor_col,
            anchor_row,
            notes: Vec::new(),
            closable: false,
            cross: std::cell::Cell::new(None),
            id: MenuId::next(),
        };
        menu.selected = menu.edge(false);
        menu
    }

    /// Gives the frame the app's close cross (a checklist that stays open
    /// until it is dismissed).
    pub(crate) fn closable(mut self) -> Self {
        self.closable = true;
        self
    }

    /// Where the last paint put the close cross: the click target, which
    /// exists exactly when a cross was drawn.
    pub(crate) fn cross_rect(&self) -> Option<Rect> {
        self.cross.get()
    }

    /// The rows, to carry them over to an open menu (`replace_entries`).
    pub(crate) fn into_entries(self) -> Vec<Entry<A>> {
        self.entries
    }

    /// Swaps the rows for `entries` keeping the cursor where it is (clamped):
    /// a checklist redraws its boxes after a click without closing. An open
    /// submenu stays open when the new rows still have it at the same place
    /// under the same label (a row refreshed by a late answer must not shut
    /// the submenu the user just opened).
    pub(crate) fn replace_entries(&mut self, entries: Vec<Entry<A>>) {
        let open: Vec<String> = {
            let mut here = &self.entries;
            let mut labels = Vec::new();
            for &i in &self.path {
                match here.get(i) {
                    Some(Entry::Submenu { label, items }) => {
                        labels.push(label.clone());
                        here = items;
                    }
                    _ => break,
                }
            }
            labels
        };
        self.entries = tidy_separators(entries);
        let mut here = &self.entries;
        let still_open = self.path.len() == open.len()
            && self
                .path
                .iter()
                .zip(&open)
                .all(|(&i, want)| match here.get(i) {
                    Some(Entry::Submenu { label, items }) if label == want => {
                        here = items;
                        true
                    }
                    _ => false,
                });
        if !still_open {
            self.path.clear();
        }
        self.selected = self.selected.min(self.row_count().saturating_sub(1));
    }

    /// Explains the grayed-out rows whose label starts with `label_start`
    /// in full: [`Menu::disabled_note`] hands the text to the status line
    /// when such a row is clicked.
    #[cfg_attr(not(feature = "hosting"), allow(dead_code))]
    pub(crate) fn with_note(
        mut self,
        label_start: impl Into<String>,
        note: impl Into<String>,
    ) -> Self {
        self.notes.push((label_start.into(), note.into()));
        self
    }

    /// The full explanation of grayed-out row `row`, when it has one.
    pub(crate) fn disabled_note(&self, row: usize) -> Option<&str> {
        if !self.row_is_disabled(row) {
            return None;
        }
        let label = self.row_label_raw(row);
        self.notes
            .iter()
            .find(|(start, _)| label.starts_with(start.as_str()))
            .map(|(_, note)| note.as_str())
    }

    /// The entry list currently on screen - the root, or whichever
    /// submenu `path` points into.
    fn level(&self) -> &[Entry<A>] {
        let mut here = &self.entries;
        for &i in &self.path {
            match here.get(i) {
                Some(Entry::Submenu { items, .. }) => here = items,
                // Unreachable by construction: `path` only ever grows via
                // `activate` on a `Submenu`. Degrading to the level we
                // reached beats panicking in a draw closure.
                _ => break,
            }
        }
        here
    }

    /// True while a submenu is open, which is exactly when a `Back` row
    /// is drawn as row 0.
    fn has_back(&self) -> bool {
        !self.path.is_empty()
    }

    /// Rows on screen, `Back` included. Never zero: an empty submenu
    /// still shows its way out.
    pub(crate) fn row_count(&self) -> usize {
        self.level().len() + usize::from(self.has_back())
    }

    /// The entry a screen row maps to, or `None` for the `Back` row.
    fn entry_at(&self, row: usize) -> Option<&Entry<A>> {
        let offset = usize::from(self.has_back());
        if self.has_back() && row == 0 {
            return None;
        }
        self.level().get(row - offset)
    }

    /// The text of one screen row, submenu marker and all. A separator
    /// has no text of its own - `render` draws its rule at the row's
    /// full width, which `row_label` cannot know.
    pub(crate) fn row_label(&self, row: usize) -> String {
        crate::terminology::tr(&self.row_label_raw(row)).into_owned()
    }

    /// The action a screen row runs, `None` for `Back`, separators, grayed
    /// rows and submenus.
    pub(crate) fn row_action(&self, row: usize) -> Option<&A> {
        match self.entry_at(row) {
            Some(Entry::Action { action, .. }) => Some(action),
            _ => None,
        }
    }

    /// The effective key hint of one row, `""` when it has none.
    pub(crate) fn row_hint(&self, row: usize) -> String {
        match self.entry_at(row) {
            Some(Entry::Action { key: Some(a), .. }) => crate::keymap::current().shown(*a),
            _ => String::new(),
        }
    }

    fn row_label_raw(&self, row: usize) -> String {
        match self.entry_at(row) {
            None => format!("{BACK_MARK} {}", gitpoulpi_i18n::msg::menu_ctx::BACK.get()),
            Some(Entry::Action { label, .. } | Entry::Disabled { label }) => label.clone(),
            Some(Entry::Submenu { label, .. }) => format!("{label} {SUBMENU_MARK}"),
            Some(Entry::Separator) => String::new(),
        }
    }

    /// Whether the keyboard and the mouse may land on this row. False
    /// only for separators; the `Back` row is selectable.
    pub(crate) fn row_is_selectable(&self, row: usize) -> bool {
        match self.entry_at(row) {
            None => true,
            Some(e) => e.is_selectable(),
        }
    }

    /// Steps `selected` by `delta`, wrapping, and keeps stepping over
    /// separators. Gives up after a full lap, so a menu of nothing but
    /// separators cannot spin forever.
    fn step(&mut self, delta: isize) {
        let rows = self.row_count();
        if rows == 0 {
            return;
        }
        for _ in 0..rows {
            let next = (self.selected as isize + delta).rem_euclid(rows as isize) as usize;
            self.selected = next;
            if self.row_is_selectable(next) {
                return;
            }
        }
    }

    /// The first/last selectable row, for Home/End and for the initial
    /// cursor - a menu that opens on its own separator would be a
    /// highlight sitting on a rule.
    fn edge(&self, from_end: bool) -> usize {
        let rows = self.row_count();
        let it = 0..rows;
        let found = if from_end {
            it.rev().find(|&r| self.row_is_selectable(r))
        } else {
            it.clone().find(|&r| self.row_is_selectable(r))
        };
        found.unwrap_or(0)
    }

    fn row_is_separator(&self, row: usize) -> bool {
        matches!(self.entry_at(row), Some(Entry::Separator))
    }

    fn row_is_disabled(&self, row: usize) -> bool {
        matches!(self.entry_at(row), Some(Entry::Disabled { .. }))
    }

    fn row_is_destructive(&self, row: usize) -> bool {
        matches!(
            self.entry_at(row),
            Some(Entry::Action {
                destructive: true,
                ..
            })
        )
    }

    /// `a1b2c3d` at the top level, `a1b2c3d > Reset` inside a submenu, so
    /// the drill-in never loses the thing the menu is about.
    pub(crate) fn title(&self) -> String {
        let mut title = self.caption.clone();
        let mut here = &self.entries;
        for &i in &self.path {
            match here.get(i) {
                Some(Entry::Submenu { label, items }) => {
                    title.push_str(&format!(" {SUBMENU_MARK} {label}"));
                    here = items;
                }
                _ => break,
            }
        }
        crate::terminology::tr(&title).into_owned()
    }

    #[cfg(test)]
    pub(crate) fn selected(&self) -> usize {
        self.selected
    }

    /// Runs the row the *mouse* clicked or the *keyboard* selected. Both
    /// paths land here, which is what keeps them from disagreeing.
    pub(crate) fn activate(&mut self, row: usize) -> MenuOutcome<A> {
        if let Ok(mut p) = PAINTED.lock() {
            if let Some(p) = p.as_mut() {
                p.row = row;
            }
        }
        if self.has_back() && row == 0 {
            self.go_back();
            return MenuOutcome::Stay;
        }
        let offset = usize::from(self.has_back());
        match self.level().get(row.wrapping_sub(offset)) {
            Some(Entry::Action { action, .. }) => MenuOutcome::Run(action.clone()),
            Some(Entry::Submenu { .. }) => {
                self.path.push(row - offset);
                // Row 0 in a submenu is `Back`; landing on it would make
                // Enter-Enter a no-op round trip. Start on the first real
                // entry, unless the submenu is empty.
                self.selected = if self.row_count() > 1 {
                    let mut probe = self.clone_cursor(1);
                    while probe < self.row_count() && !self.row_is_selectable(probe) {
                        probe += 1;
                    }
                    probe.min(self.row_count() - 1)
                } else {
                    0
                };
                MenuOutcome::Stay
            }
            // A separator, a grayed-out row, or a row past the end:
            // consume the click and leave the menu exactly as it was.
            Some(Entry::Separator | Entry::Disabled { .. }) | None => MenuOutcome::Stay,
        }
    }

    /// `min(n, last row)`, spelled out so `activate`'s probe reads.
    fn clone_cursor(&self, n: usize) -> usize {
        n.min(self.row_count().saturating_sub(1))
    }

    /// Leaves the current submenu, restoring the selection to the row
    /// that opened it so `Enter`, `Esc` lands where it started.
    fn go_back(&mut self) {
        if let Some(came_from) = self.path.pop() {
            self.selected = came_from + usize::from(self.has_back());
        }
    }

    /// Whether the selected row opens a submenu - what `Right` acts on.
    fn selected_is_submenu(&self) -> bool {
        matches!(self.entry_at(self.selected), Some(Entry::Submenu { .. }))
    }

    /// The whole keyboard model, in one place.
    ///
    /// `j`/`k` join the arrows because every other list in this UI takes
    /// them. `Home`/`End` clamp rather than wrap (the repository
    /// overlay's rule); the arrows wrap, because a six-row popup is
    /// exactly where wrapping saves keystrokes.
    pub(crate) fn on_key(&mut self, code: KeyCode) -> MenuOutcome<A> {
        let rows = self.row_count();
        if rows == 0 {
            // Cannot happen for a menu built with entries, but a caller
            // that hands over an empty list must still be able to escape.
            return match code {
                KeyCode::Esc => MenuOutcome::Close,
                _ => MenuOutcome::NotHandled,
            };
        }
        match code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.step(-1);
                MenuOutcome::Stay
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.step(1);
                MenuOutcome::Stay
            }
            KeyCode::Home => {
                self.selected = self.edge(false);
                MenuOutcome::Stay
            }
            KeyCode::End => {
                self.selected = self.edge(true);
                MenuOutcome::Stay
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                let row = self.selected;
                self.activate(row)
            }
            // Right only ever opens: on an action row it would otherwise
            // fire the action, and a stray arrow key must not commit a
            // reset.
            KeyCode::Right | KeyCode::Char('l') if self.selected_is_submenu() => {
                let row = self.selected;
                self.activate(row)
            }
            KeyCode::Left | KeyCode::Char('h') | KeyCode::Backspace | KeyCode::Esc => {
                if self.has_back() {
                    self.go_back();
                    MenuOutcome::Stay
                } else {
                    MenuOutcome::Close
                }
            }
            _ => MenuOutcome::NotHandled,
        }
    }

    /// Draws the menu and returns one rect per screen row, in order, so the
    /// caller registers hit targets from the same walk that painted the
    /// glyphs (the `log_table` rule: a second walk recomputing the geometry
    /// is how clicks land a cell off).
    ///
    /// `hovered` is the row the mouse is over; it gets the subtler `HOVER_BG`
    /// while the keyboard/click-driven `self.selected` keeps the stronger
    /// `SELECTION_BG` and wins when both coincide.
    pub(crate) fn render(&self, frame: &mut Frame, hovered: Option<usize>) -> Vec<Rect> {
        let title = self.title();
        let rows = self.row_count();
        let width = (0..rows)
            .map(|r| {
                let hint = self.row_hint(r);
                let gap = if hint.is_empty() {
                    0
                } else {
                    display_width(&hint) + 2
                };
                (display_width(&self.row_label(r)) + gap) as u16
            })
            .max()
            .unwrap_or(0)
            .max(display_width(&title) as u16 + if self.closable { 6 } else { 0 })
            + 2;
        let height = rows as u16 + 2;
        let popup = anchored_popup(
            frame.area(),
            self.anchor_col,
            self.anchor_row,
            width,
            height,
        );
        frame.render_clipped(Clear, popup);
        let inner_width = popup.width.saturating_sub(2) as usize;
        let lines: Vec<Line> = (0..rows)
            .map(|r| {
                if self.row_is_separator(r) {
                    // A rule, drawn to the popup's inner width in the
                    // border's own color so it reads as part of the
                    // frame rather than as a row of text.
                    return Line::from(Span::styled(
                        "\u{2500}".repeat(inner_width),
                        Style::default().fg(th().ui.accent),
                    ));
                }
                let mut label = self.row_label(r);
                let hint = self.row_hint(r);
                if !hint.is_empty() {
                    // Right-aligned: pad between label and hint.
                    let gap = inner_width
                        .saturating_sub(display_width(&label) + display_width(&hint))
                        .max(1);
                    label.push_str(&" ".repeat(gap));
                    label.push_str(&hint);
                }
                // Pad so the selection band spans the whole row rather
                // than stopping at the end of the text -- the repository
                // overlay's rule, and the difference between "selected"
                // reading as a highlight and as a smudge.
                let pad = inner_width.saturating_sub(display_width(&label));
                label.push_str(&" ".repeat(pad));
                if self.row_is_disabled(r) {
                    // Never highlighted: it cannot be selected, and a
                    // hover band would say "clickable".
                    return Line::from(Span::styled(label, Style::default().fg(th().ui.disabled)));
                }
                let mut style = if self.row_is_destructive(r) {
                    Style::default().fg(th().status.danger)
                } else {
                    Style::default().fg(th().ui.text)
                };
                if r == self.selected {
                    style = style.bg(th().ui.selection);
                } else if Some(r) == hovered {
                    style = style.bg(th().ui.hover);
                }
                Line::from(Span::styled(label, style))
            })
            .collect();
        // A menu taller than the screen (the merged ref+commit menu is ~35
        // rows) shows a window of its rows with a scrollbar in the right
        // border: the selection is followed, the wheel and the bar move it.
        let visible = popup.height.saturating_sub(2) as usize;
        let (offset, bar) =
            scroll::frame(self.id.0, &self.path, rows, visible, self.selected, popup);
        if let Ok(mut p) = PAINTED.lock() {
            *p = Some(Painted {
                popup,
                offset,
                row: self.selected,
            });
        }
        let lines: Vec<Line> = lines.into_iter().skip(offset).take(visible).collect();
        frame.render_clipped(
            Paragraph::new(lines).block(
                Block::default()
                    .title(title)
                    .border_type(BorderType::Rounded)
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(th().ui.accent))
                    .style(Style::default().bg(th().ui.panel)),
            ),
            popup,
        );
        if let Some(track) = bar {
            crate::scrollbar::render(frame, track, true, false);
        }
        self.cross.set(if self.closable {
            crate::chrome::close_cross::paint(frame.buffer_mut(), popup, false)
        } else {
            None
        });
        // One rect per row, in row order, so callers can `enumerate()`
        // them into hit targets: a row scrolled out of view gets an empty
        // rect no pointer can be inside.
        (0..rows)
            .map(|i| {
                if i < offset || i >= offset + visible {
                    return Rect::new(0, 0, 0, 0);
                }
                Rect::new(
                    popup.x + 1,
                    popup.y + 1 + (i - offset) as u16,
                    popup.width.saturating_sub(2),
                    1,
                )
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum A {
        One,
        Two,
        Soft,
        Hard,
    }

    fn menu() -> Menu<A> {
        Menu::new(
            "a1b2c3d",
            vec![
                Entry::action("One", A::One),
                Entry::action("Two", A::Two),
                Entry::submenu(
                    "Reset",
                    vec![
                        Entry::action("soft", A::Soft),
                        Entry::danger("hard", A::Hard),
                    ],
                ),
            ],
            4,
            6,
        )
    }

    /// A late refresh of the rows keeps the submenu the user opened, and
    /// closes it only when that submenu is gone from the new rows.
    #[test]
    fn replacing_the_rows_keeps_an_open_submenu_that_still_exists() {
        let mut m = menu();
        assert_eq!(m.activate(2), MenuOutcome::Stay);
        assert!(m.has_back(), "Reset is open");
        m.replace_entries(menu().into_entries());
        assert!(m.has_back(), "same rows: still open");
        assert_eq!(m.row_label(1), "soft");
        m.replace_entries(vec![Entry::action("One", A::One)]);
        assert!(!m.has_back(), "the submenu is gone: back to the top level");
    }

    /// A grayed-out row shows its reason, is stepped over by the arrows
    /// and does nothing when clicked - never a row that looks live.
    #[test]
    fn a_disabled_row_is_shown_with_its_reason_but_never_selected_or_run() {
        let mut m = Menu::new(
            "x",
            vec![
                Entry::action("One", A::One),
                Entry::disabled("Two", "not now"),
                Entry::action("Soft", A::Soft),
            ],
            0,
            0,
        );
        assert_eq!(m.row_label(1), "Two (not now)");
        assert!(!m.row_is_selectable(1));
        m.on_key(KeyCode::Down);
        assert_eq!(m.selected(), 2, "Down steps over the disabled row");
        assert_eq!(m.activate(1), MenuOutcome::Stay);
        assert_eq!(m.on_key(KeyCode::Enter), MenuOutcome::Run(A::Soft));
    }

    /// A long reason stays off the row: the row carries a few words and
    /// the status line gets the whole sentence when the row is clicked.
    #[test]
    fn a_grayed_row_keeps_a_short_label_and_hands_its_full_note_to_the_status_line() {
        let m = Menu::new(
            "x",
            vec![
                Entry::action("One", A::One),
                Entry::disabled("Open in Obsidian", "no vault"),
            ],
            0,
            0,
        )
        .with_note(
            "Open in Obsidian",
            "This folder is not inside an Obsidian vault",
        );
        assert_eq!(m.row_label(1), "Open in Obsidian (no vault)");
        assert_eq!(
            m.disabled_note(1),
            Some("This folder is not inside an Obsidian vault")
        );
        assert_eq!(m.disabled_note(0), None, "a live row has no note");
    }

    #[test]
    fn a_tall_menu_scrolls_to_keep_the_selection_visible() {
        use super::scroll::follow;
        assert_eq!(follow(0, 9, 10), 0);
        assert_eq!(follow(0, 10, 10), 1);
        assert_eq!(follow(0, 29, 10), 20);
        assert_eq!(follow(20, 25, 10), 20, "inside the window: it stays");
        assert_eq!(follow(20, 5, 10), 5);
    }

    #[test]
    fn enter_on_an_action_row_runs_it() {
        let mut m = menu();
        assert_eq!(m.on_key(KeyCode::Down), MenuOutcome::Stay);
        assert_eq!(m.on_key(KeyCode::Enter), MenuOutcome::Run(A::Two));
    }

    #[test]
    fn enter_on_a_submenu_row_drills_in_and_esc_comes_back_to_it() {
        let mut m = menu();
        m.on_key(KeyCode::End);
        assert_eq!(m.selected(), 2);
        assert_eq!(m.on_key(KeyCode::Enter), MenuOutcome::Stay);
        // Back + two entries, and the cursor skipped Back.
        assert_eq!(m.row_count(), 3);
        assert_eq!(m.selected(), 1);
        assert_eq!(m.row_label(0), format!("{BACK_MARK} Back"));
        assert_eq!(m.title(), format!("a1b2c3d {SUBMENU_MARK} Reset"));
        assert_eq!(m.on_key(KeyCode::Esc), MenuOutcome::Stay);
        assert_eq!(m.row_count(), 3);
        assert_eq!(
            m.selected(),
            2,
            "Esc must restore the cursor to the row that opened the submenu"
        );
    }

    #[test]
    fn esc_at_the_top_level_closes_but_esc_inside_a_submenu_does_not() {
        let mut m = menu();
        assert_eq!(m.on_key(KeyCode::Esc), MenuOutcome::Close);
        let mut m = menu();
        m.activate(2);
        assert_eq!(m.on_key(KeyCode::Esc), MenuOutcome::Stay);
        assert_eq!(m.on_key(KeyCode::Esc), MenuOutcome::Close);
    }

    /// The mouse's only job: a click on the `Back` row must leave the
    /// submenu, not close the menu and not run anything. Without a
    /// clickable Back row a mouse user who drills in is stuck.
    #[test]
    fn clicking_the_back_row_leaves_the_submenu() {
        let mut m = menu();
        m.activate(2);
        assert_eq!(m.activate(0), MenuOutcome::Stay);
        assert!(!m.has_back());
        assert_eq!(m.row_count(), 3);
    }

    #[test]
    fn clicking_a_submenu_row_opens_it_rather_than_running_anything() {
        let mut m = menu();
        assert_eq!(m.activate(2), MenuOutcome::Stay);
        assert_eq!(m.activate(2), MenuOutcome::Run(A::Hard));
    }

    /// `Right` is an "open" key, never a "run" key: pressing it on an
    /// action row must not commit a reset.
    #[test]
    fn right_opens_a_submenu_and_does_nothing_on_an_action_row() {
        let mut m = menu();
        assert_eq!(m.on_key(KeyCode::Right), MenuOutcome::NotHandled);
        m.on_key(KeyCode::End);
        assert_eq!(m.on_key(KeyCode::Right), MenuOutcome::Stay);
        assert!(m.has_back());
    }

    #[test]
    fn the_arrows_wrap_and_home_end_clamp() {
        let mut m = menu();
        m.on_key(KeyCode::Up);
        assert_eq!(m.selected(), 2);
        m.on_key(KeyCode::Down);
        assert_eq!(m.selected(), 0);
        m.on_key(KeyCode::End);
        assert_eq!(m.selected(), 2);
        m.on_key(KeyCode::End);
        assert_eq!(m.selected(), 2);
        m.on_key(KeyCode::Home);
        assert_eq!(m.selected(), 0);
        m.on_key(KeyCode::Home);
        assert_eq!(m.selected(), 0);
    }

    /// `menu()`'s fixture has no separator, so it cannot tell a real
    /// skip-the-separator implementation from one that naively steps by
    /// one row - with only actions and a submenu, every row is
    /// selectable and the two behave identically. A leading/trailing
    /// separator is auto-stripped by `tidy_separators` (that is its own
    /// job, exercised elsewhere), so the only way to keep one in the
    /// fixture is between two actions - which is exactly the row
    /// Up/Down/wrap/Home/End all have to step past.
    #[test]
    fn the_arrows_skip_a_separator_between_two_actions() {
        let mut m: Menu<A> = Menu::new(
            "a1b2c3d",
            vec![
                Entry::action("One", A::One),
                Entry::Separator,
                Entry::action("Two", A::Two),
            ],
            4,
            6,
        );
        assert_eq!(m.selected(), 0, "opens on the first action, not the rule");
        m.on_key(KeyCode::Down);
        assert_eq!(m.selected(), 2, "stepping down must skip the rule at 1");
        m.on_key(KeyCode::Down);
        assert_eq!(
            m.selected(),
            0,
            "wrapping down from the last row must skip the rule and land on the first action"
        );
        m.on_key(KeyCode::Up);
        assert_eq!(
            m.selected(),
            2,
            "wrapping up from the first row must skip the rule and land on the last action"
        );
        m.on_key(KeyCode::Home);
        assert_eq!(m.selected(), 0);
        m.on_key(KeyCode::End);
        assert_eq!(m.selected(), 2);
    }

    #[test]
    fn a_submenu_row_is_marked_and_an_action_row_is_not() {
        let m = menu();
        assert_eq!(m.row_label(0), "One");
        assert_eq!(m.row_label(2), format!("Reset {SUBMENU_MARK}"));
    }

    #[test]
    fn a_key_the_menu_does_not_own_falls_through() {
        let mut m = menu();
        assert_eq!(m.on_key(KeyCode::Char('q')), MenuOutcome::NotHandled);
        assert_eq!(m.on_key(KeyCode::Tab), MenuOutcome::NotHandled);
    }

    /// Every row is drawn *and* registered, in the same order, from one
    /// walk - including the `Back` row, whose rect is what makes the
    /// drill-in mouse-operable at all.
    #[test]
    fn render_returns_one_rect_per_row_in_order() {
        use ratatui::backend::TestBackend;
        use ratatui::Terminal;
        let mut m = menu();
        m.activate(2);
        let mut terminal = Terminal::new(TestBackend::new(60, 20)).unwrap();
        let mut rects = Vec::new();
        terminal
            .draw(|f| {
                rects = m.render(f, None);
            })
            .unwrap();
        assert_eq!(rects.len(), m.row_count());
        for pair in rects.windows(2) {
            assert_eq!(pair[1].y, pair[0].y + 1);
            assert_eq!(pair[1].x, pair[0].x);
        }
        let buffer = terminal.backend().buffer().clone();
        let text: String = buffer.content().iter().map(|c| c.symbol()).collect();
        assert!(text.contains("Back"), "no Back row was drawn");
        assert!(text.contains("hard"), "the submenu's rows were not drawn");
        assert!(
            !text.contains("One"),
            "the parent level is still drawn: this is a flyout, not a drill-in"
        );
    }

    fn hinted() -> Menu<A> {
        Menu::new(
            "x",
            vec![
                Entry::action("Pull", A::One).with_key(crate::keymap::Action::GitPull),
                Entry::action("Plain", A::Two),
                Entry::action("Unbound", A::Soft).with_key(crate::keymap::Action::GitPush),
            ],
            0,
            0,
        )
    }

    fn drawn(m: &Menu<A>) -> Vec<String> {
        use ratatui::backend::TestBackend;
        use ratatui::Terminal;
        let mut terminal = Terminal::new(TestBackend::new(60, 12)).unwrap();
        terminal.draw(|f| drop(m.render(f, None))).unwrap();
        let buf = terminal.backend().buffer().clone();
        (0..buf.area.height)
            .map(|y| (0..buf.area.width).map(|x| buf[(x, y)].symbol()).collect())
            .collect()
    }

    /// A row that is the mouse twin of a registry action shows the
    /// effective binding at its right edge; rows without a twin do not.
    #[test]
    fn a_closable_menu_paints_a_cross_on_its_frame_and_hands_back_its_rect() {
        use ratatui::backend::TestBackend;
        let mut terminal = ratatui::Terminal::new(TestBackend::new(60, 12)).unwrap();
        let plain = menu();
        terminal
            .draw(|f| {
                plain.render(f, None);
            })
            .unwrap();
        assert!(plain.cross_rect().is_none());
        let closable = menu().closable();
        terminal
            .draw(|f| {
                closable.render(f, None);
            })
            .unwrap();
        let cross = closable.cross_rect().expect("a cross rect");
        let buf = terminal.backend().buffer().clone();
        let at: String = (cross.x..cross.x + cross.width)
            .map(|x| buf[(x, cross.y)].symbol().to_string())
            .collect();
        assert!(at.contains('\u{2715}'), "{at:?}");
    }

    #[test]
    fn a_row_shows_the_effective_binding_of_its_action_right_aligned() {
        let m = hinted();
        assert_eq!(m.row_hint(0), "Ctrl+G p");
        assert_eq!(m.row_hint(1), "");
        let rows = drawn(&m);
        let pull = rows.iter().find(|r| r.contains("Pull")).expect("row");
        let plain = rows.iter().find(|r| r.contains("Plain")).expect("row");
        assert!(pull.contains("Ctrl+G p"), "{pull}");
        assert!(!plain.contains("Ctrl"), "{plain}");
        // Right-aligned: the hint ends where the border begins.
        assert!(pull.trim_end().ends_with("Ctrl+G p\u{2502}"), "{pull:?}");
    }

    /// Rebinding the action changes the hint; unbinding drops it.
    #[test]
    fn the_hint_follows_a_rebinding_and_disappears_when_unbound() {
        let km = crate::keymap::test_keymap(&[
            ("git_prefix", "pull", &["z"]),
            ("git_prefix", "push", &[]),
        ]);
        crate::keymap::with_test_keymap(km, || {
            let m = hinted();
            assert_eq!(m.row_hint(0), "Ctrl+G z");
            assert_eq!(m.row_hint(2), "");
            let rows = drawn(&m);
            assert!(rows
                .iter()
                .any(|r| r.contains("Pull") && r.contains("Ctrl+G z")));
            assert!(!rows.iter().any(|r| r.contains("Ctrl+G p")));
        });
    }

    /// The real menus tag their rows: the ref menu's Push carries Push's
    /// binding, the file menu's Stage carries Stage's.
    #[test]
    fn the_shipped_menus_tag_their_key_twins() {
        use crate::log_menu::MenuContext;
        use crate::ref_menu::ref_menu_entries;
        use crate::refs_sidebar::SidebarSection;
        let entries = ref_menu_entries(
            SidebarSection::Local,
            "feature",
            &MenuContext::default(),
            false,
            false,
            false,
        );
        let keyed: Vec<_> = entries
            .iter()
            .filter_map(|e| match e {
                Entry::Action { key: Some(k), .. } => Some(*k),
                _ => None,
            })
            .collect();
        assert!(keyed.contains(&crate::keymap::Action::GitPush), "{keyed:?}");
        assert!(
            keyed.contains(&crate::keymap::Action::RefsSolo),
            "{keyed:?}"
        );
    }

    /// 30 rows `r00`..`r29` in a 60x12 screen: 10 fit.
    fn tall() -> Menu<A> {
        Menu::new(
            "tall",
            (0..30)
                .map(|i| Entry::action(format!("r{i:02}"), A::One))
                .collect(),
            0,
            0,
        )
    }

    fn paint(m: &Menu<A>) -> ratatui::buffer::Buffer {
        use ratatui::backend::TestBackend;
        let mut terminal = ratatui::Terminal::new(TestBackend::new(60, 12)).unwrap();
        crate::scrollbar::begin_frame();
        begin_frame();
        terminal.draw(|f| drop(m.render(f, None))).unwrap();
        terminal.backend().buffer().clone()
    }

    fn mouse(
        kind: crossterm::event::MouseEventKind,
        col: u16,
        row: u16,
    ) -> crossterm::event::MouseEvent {
        crossterm::event::MouseEvent {
            kind,
            column: col,
            row,
            modifiers: KeyModifiers::NONE,
        }
    }

    fn text_of(buf: &ratatui::buffer::Buffer) -> String {
        (0..buf.area.height)
            .map(|y| {
                (0..buf.area.width)
                    .map(|x| buf[(x, y)].symbol())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn a_menu_taller_than_the_screen_draws_a_bar_and_one_that_fits_does_not() {
        let m = tall();
        let buf = paint(&m);
        let track = scroll::track().expect("a bar");
        assert_eq!(buf[(track.rect.x, track.rect.y)].symbol(), "\u{2191}");
        assert_eq!(
            buf[(track.rect.x, track.rect.bottom() - 1)].symbol(),
            "\u{2193}"
        );
        paint(&menu());
        assert!(scroll::track().is_none(), "a menu that fits has no bar");
    }

    #[test]
    fn the_bar_drags_the_wheel_scrolls_and_the_selection_is_followed() {
        use crossterm::event::{MouseButton, MouseEventKind};
        let mut m = tall();
        paint(&m);
        let track = scroll::track().unwrap();
        let x = track.rect.x;
        // The thumb dragged past the bottom: the window is at the end and the
        // selection did not move.
        let down = track.rect.y + track.thumb_start;
        assert!(on_mouse(mouse(
            MouseEventKind::Down(MouseButton::Left),
            x,
            down
        )));
        assert!(on_mouse(mouse(
            MouseEventKind::Drag(MouseButton::Left),
            x,
            track.rect.bottom() + 2
        )));
        assert!(on_mouse(mouse(
            MouseEventKind::Up(MouseButton::Left),
            x,
            track.rect.bottom() + 2
        )));
        let buf = paint(&m);
        assert_eq!(scroll::top(), 20);
        assert!(text_of(&buf).contains("r29"));
        assert_eq!(m.selected(), 0);
        // The wheel over the popup scrolls it back; off the popup it is not ours.
        assert!(on_mouse(mouse(MouseEventKind::ScrollUp, 5, 5)));
        assert_eq!(scroll::top(), 17);
        assert!(!on_mouse(mouse(MouseEventKind::ScrollUp, 59, 11)));
        // A key moves the selection and the window comes to it.
        paint(&m);
        for _ in 0..3 {
            m.on_key(KeyCode::Up);
        }
        assert_eq!(m.selected(), 27);
        let buf = paint(&m);
        assert!(text_of(&buf).contains("r27"), "{}", text_of(&buf));
        m.on_key(KeyCode::Home);
        paint(&m);
        assert_eq!(scroll::top(), 0);
    }

    #[test]
    fn a_closed_menu_takes_no_click() {
        use crossterm::event::{MouseButton, MouseEventKind};
        let m = tall();
        paint(&m);
        let track = scroll::track().unwrap();
        // The next frame paints no menu: the registry is empty.
        begin_frame();
        assert!(!on_mouse(mouse(
            MouseEventKind::Down(MouseButton::Left),
            track.rect.x,
            track.rect.y + 2
        )));
    }
}
