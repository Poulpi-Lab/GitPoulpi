//! The metadata picker: one popup that chooses assignees, reviewers or
//! labels (a checkbox list with a fuzzy filter) or the milestone (a
//! combobox), for a project's pull / merge request or issue on GitHub or
//! GitLab.
//!
//! Used in four places that share this code and nothing else:
//!
//! - the create-pull-request composer and the create-issue form (the choices
//!   ride along with the new request: [`MetaFields`]);
//! - an existing request or issue, from the sidebar row menus, the Radar
//!   and the palette (the choices are sent as an edit: [`MetaEdit`]).
//!
//! Keys: type to filter, `Up`/`Down` move, `Space` toggles, `Enter` applies,
//! `Esc` cancels, `Tab` reaches the buttons. Mouse: a click on a row toggles
//! it, the wheel scrolls, the buttons click. Everything loads on a worker
//! thread; the popup opens at once and shows a spinner row after
//! [`SPIN_AFTER`] (maintainer rule: feedback after about 150 ms).
//!
//! Candidates and the item's current values are asked of the host when the
//! popup opens, never earlier: hosting is opt-in and a form nobody opens
//! costs no request.

mod edit;
mod fields;
mod load;
mod render;
#[cfg(test)]
mod tests;

pub(crate) use edit::{MetaEdit, MetaEditOutcome};
pub(crate) use fields::MetaFields;
pub(crate) use load::MetaTarget;

use std::sync::mpsc;
use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use gitpoulpi_hosting::ItemMeta;
use ratatui::layout::Rect;
use ratatui::style::Color;

use crate::combo::{Combo, ComboOption, ComboOutcome};
use crate::dialog_kit;
use load::Loaded;

/// A request in flight for longer than this shows a spinner.
pub(crate) const SPIN_AFTER: Duration = Duration::from_millis(150);

/// Whether GitPoulpi can edit assignees, reviewers, labels and milestones
/// on this provider. GitHub, GitLab, Gitea / Forgejo, Linear, YouTrack, Redmine and
/// the local Markdown source: nothing about metadata shows for any other
/// provider.
pub(crate) fn supports(kind: gitpoulpi_config::IntegrationKind) -> bool {
    matches!(
        kind.family(),
        gitpoulpi_config::ApiFamily::Github
            | gitpoulpi_config::ApiFamily::Gitlab
            | gitpoulpi_config::ApiFamily::Gitea
            | gitpoulpi_config::ApiFamily::Linear
            | gitpoulpi_config::ApiFamily::YouTrack
            | gitpoulpi_config::ApiFamily::Redmine
            | gitpoulpi_config::ApiFamily::LocalMarkdown
    )
}

/// Whether `field` can be edited on this provider: everything that takes
/// metadata does, except that Redmine has no labels (the picker is hidden,
/// not offered empty).
pub(crate) fn supports_field(kind: gitpoulpi_config::IntegrationKind, field: MetaField) -> bool {
    supports(kind)
        && !(field == MetaField::Labels && kind.family() == gitpoulpi_config::ApiFamily::Redmine)
}

/// What a picker chooses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum MetaField {
    Assignees,
    Reviewers,
    Labels,
    Milestone,
}

impl MetaField {
    /// The popup's title.
    pub(crate) fn title(self) -> &'static str {
        match self {
            MetaField::Assignees => gitpoulpi_i18n::msg::meta_picker::TITLE_ASSIGNEES.get(),
            MetaField::Reviewers => gitpoulpi_i18n::msg::meta_picker::TITLE_REVIEWERS.get(),
            MetaField::Labels => gitpoulpi_i18n::msg::meta_picker::TITLE_LABELS.get(),
            MetaField::Milestone => gitpoulpi_i18n::msg::meta_picker::TITLE_MILESTONE.get(),
        }
    }

    /// The short name on a form's chip.
    pub(crate) fn chip(self) -> &'static str {
        match self {
            MetaField::Assignees => gitpoulpi_i18n::msg::meta_picker::CHIP_ASSIGNEES.get(),
            MetaField::Reviewers => gitpoulpi_i18n::msg::meta_picker::CHIP_REVIEWERS.get(),
            MetaField::Labels => gitpoulpi_i18n::msg::meta_picker::CHIP_LABELS.get(),
            MetaField::Milestone => gitpoulpi_i18n::msg::meta_picker::CHIP_MILESTONE.get(),
        }
    }

    pub(crate) fn is_single(self) -> bool {
        self == MetaField::Milestone
    }
}

/// A chosen milestone: what an edit sends and what a chip shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MilestoneRef {
    pub(crate) id: u64,
    pub(crate) title: String,
}

/// What a picker applied.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Applied {
    /// Logins (assignees or reviewers) or label names.
    List(MetaField, Vec<String>),
    /// `None` clears the milestone.
    Milestone(Option<MilestoneRef>),
}

/// What a create form carries besides its text.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct MetaSelection {
    pub(crate) assignees: Vec<String>,
    pub(crate) reviewers: Vec<String>,
    pub(crate) labels: Vec<String>,
    pub(crate) milestone: Option<MilestoneRef>,
}

impl MetaSelection {
    pub(crate) fn apply(&mut self, applied: Applied) {
        match applied {
            Applied::List(MetaField::Assignees, list) => self.assignees = list,
            Applied::List(MetaField::Reviewers, list) => self.reviewers = list,
            Applied::List(_, list) => self.labels = list,
            Applied::Milestone(m) => self.milestone = m,
        }
    }

    pub(crate) fn list(&self, field: MetaField) -> &[String] {
        match field {
            MetaField::Assignees => &self.assignees,
            MetaField::Reviewers => &self.reviewers,
            _ => &self.labels,
        }
    }

    /// What a form chip shows after its name: the choices, or "none".
    pub(crate) fn summary(&self, field: MetaField) -> String {
        if field == MetaField::Milestone {
            return match &self.milestone {
                Some(m) => m.title.clone(),
                None => gitpoulpi_i18n::msg::meta_picker::NONE.get().to_string(),
            };
        }
        let list = self.list(field);
        match list {
            [] => gitpoulpi_i18n::msg::meta_picker::NONE.get().to_string(),
            [only] => only.clone(),
            [first, rest @ ..] => format!("{first} +{}", rest.len()),
        }
    }
}

/// One candidate row.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Item {
    /// What is sent: a login, a label name or a milestone id.
    pub(crate) value: String,
    pub(crate) label: String,
    /// A person's name, a label's description, a milestone's due date.
    pub(crate) detail: String,
    /// A label's colour swatch.
    pub(crate) color: Option<Color>,
    /// A person's initials badge (the avatar fallback, no request).
    pub(crate) badge: Option<(String, Color)>,
}

#[derive(Debug, Clone, PartialEq)]
enum Status {
    Loading,
    Ready,
    Failed(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Focus {
    /// The filter and the list: typing filters, arrows move.
    List,
    Combo,
    Apply,
    Cancel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Hit {
    Filter,
    Row(usize),
    Combo,
    Apply,
    Cancel,
}

/// What a key or click did to the picker.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum PickerOutcome {
    Stay,
    Cancel,
    Apply(Applied),
}

pub(crate) struct MetaPicker {
    field: MetaField,
    target: MetaTarget,
    rx: Option<mpsc::Receiver<Loaded>>,
    since: Instant,
    status: Status,
    items: Vec<Item>,
    /// The checked values, in the order they were checked.
    checked: Vec<String>,
    /// The user has toggled something: the loaded current values no longer
    /// replace the choice.
    touched: bool,
    query: String,
    edit: crate::text_edit::FieldEditor<()>,
    /// Indexes into `items` that match the filter, best first.
    shown: Vec<usize>,
    /// A local Markdown folder takes values nobody used yet: the typed text
    /// is offered as a row of its own.
    free_entry: bool,
    /// The index in `items` of that row, while it is on offer.
    free: Option<usize>,
    cursor: usize,
    offset: usize,
    focus: Focus,
    /// The milestone choice.
    combo: Combo,
    hover: Option<Hit>,
    hits: Vec<(Rect, Hit)>,
    bar: Option<crate::scrollbar::Track>,
    bar_drag: crate::scrollbar::BarDrag,
    rows_visible: usize,
    message: Option<String>,
    /// The frame and its close cross as the last render painted them.
    frame: Rect,
    close: dialog_kit::CloseHit,
}

impl MetaPicker {
    /// Opens the picker for `field` of `target` and starts loading its
    /// candidates (and, for an existing item, its current values).
    /// `preselected` is what a create form has chosen so far.
    pub(crate) fn open(
        factory: std::sync::Arc<dyn crate::hosting_task::ClientFactory>,
        target: MetaTarget,
        field: MetaField,
        preselected: &MetaSelection,
    ) -> Self {
        let free_entry = target.integration.kind.capabilities().local_folder && !field.is_single();
        let rx = load::start_load(factory, &target, field);
        let mut combo =
            Combo::new(gitpoulpi_i18n::msg::meta_picker::MILESTONE_PLACEHOLDER.get()).with_search();
        combo.loading = true;
        let checked = match field {
            MetaField::Milestone => preselected
                .milestone
                .iter()
                .map(|m| m.id.to_string())
                .collect(),
            other => preselected.list(other).to_vec(),
        };
        Self {
            free_entry,
            field,
            target,
            rx: Some(rx),
            since: Instant::now(),
            status: Status::Loading,
            items: Vec::new(),
            checked,
            touched: false,
            query: String::new(),
            edit: Default::default(),
            free: None,
            shown: Vec::new(),
            cursor: 0,
            offset: 0,
            focus: if field.is_single() {
                Focus::Combo
            } else {
                Focus::List
            },
            combo,
            hover: None,
            hits: Vec::new(),
            bar: None,
            bar_drag: Default::default(),
            rows_visible: 8,
            message: None,
            frame: Rect::default(),
            close: Default::default(),
        }
    }

    /// Waiting for the host (drives the spinner's redraws).
    pub(crate) fn busy(&self) -> bool {
        self.rx.is_some()
    }

    /// The values checked now (a test and the milestone's id).
    #[cfg(test)]
    pub(crate) fn checked(&self) -> &[String] {
        &self.checked
    }

    /// Drains the load. True when the popup changed.
    pub(crate) fn poll(&mut self) -> bool {
        let Some(rx) = &self.rx else {
            return false;
        };
        let Ok(loaded) = rx.try_recv() else {
            return false;
        };
        self.rx = None;
        match loaded.items {
            Ok(items) => {
                self.items = items;
                self.free = None;
                self.status = Status::Ready;
                if !self.touched {
                    if let Some(current) = loaded.current {
                        self.checked = current_values(self.field, &current);
                    }
                }
                self.refilter();
                if self.field.is_single() {
                    self.fill_combo();
                }
            }
            Err(why) => {
                self.message = Some(why.clone());
                self.status = Status::Failed(why);
                self.combo.loading = false;
            }
        }
        true
    }

    fn fill_combo(&mut self) {
        let mut options = vec![ComboOption::new(
            gitpoulpi_i18n::msg::meta_picker::NO_MILESTONE.get(),
            "",
        )];
        options.extend(self.items.iter().map(|i| {
            let label = if i.detail.is_empty() {
                i.label.clone()
            } else {
                format!("{}  ({})", i.label, i.detail)
            };
            ComboOption::new(label, i.value.clone())
        }));
        self.combo.set_options(options);
        self.combo.loading = false;
        // The checked milestone, else "no milestone" when there is a
        // current value that is none.
        let current = self.checked.first().cloned();
        self.combo.select(Some(current.unwrap_or_default()));
        self.combo.open();
    }

    /// Re-ranks the rows for the filter text.
    fn refilter(&mut self) {
        // The previous offer of the typed text: dropped, unless it was
        // ticked (then it is an ordinary row from now on).
        if let Some(i) = self.free.take() {
            if let Some(item) = self.items.get_mut(i) {
                if self.checked.contains(&item.value) {
                    item.label = item.value.clone();
                    item.detail.clear();
                } else {
                    self.items.remove(i);
                }
            }
        }
        let typed = self.query.trim().to_string();
        if self.free_entry
            && !typed.is_empty()
            && !self
                .items
                .iter()
                .any(|i| i.value.eq_ignore_ascii_case(&typed))
        {
            self.items.push(Item {
                value: typed.clone(),
                label: gitpoulpi_i18n::msg::local_issues::add_value(&typed),
                detail: String::new(),
                color: None,
                badge: None,
            });
            self.free = Some(self.items.len() - 1);
        }
        let q = self.query.trim().to_lowercase();
        let mut scored: Vec<(i32, usize)> = self
            .items
            .iter()
            .enumerate()
            .filter_map(|(i, item)| {
                if q.is_empty() {
                    return Some((0, i));
                }
                let hay = format!("{} {}", item.label, item.detail).to_lowercase();
                gitpoulpi_core::fuzzy::score_lower(&hay, &q, gitpoulpi_core::fuzzy::Anchor::Whole)
                    .map(|s| (s, i))
            })
            .collect();
        if !q.is_empty() {
            scored.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
        }
        self.shown = scored.into_iter().map(|(_, i)| i).collect();
        if let Some(free) = self.free {
            self.shown.retain(|i| *i != free);
            self.shown.insert(0, free);
        }
        self.cursor = self.cursor.min(self.shown.len().saturating_sub(1));
        self.offset = 0;
        self.keep_visible();
    }

    fn keep_visible(&mut self) {
        let rows = self.rows_visible.max(1);
        if self.cursor < self.offset {
            self.offset = self.cursor;
        } else if self.cursor >= self.offset + rows {
            self.offset = self.cursor + 1 - rows;
        }
    }

    fn toggle_at(&mut self, shown_index: usize) {
        let Some(&i) = self.shown.get(shown_index) else {
            return;
        };
        self.cursor = shown_index;
        self.touched = true;
        let value = self.items[i].value.clone();
        if let Some(at) = self.checked.iter().position(|c| *c == value) {
            self.checked.remove(at);
        } else {
            self.checked.push(value);
        }
        self.message = None;
    }

    /// What `Apply` sends.
    fn applied(&self) -> Applied {
        if self.field.is_single() {
            let chosen = self.combo.selected().filter(|v| !v.is_empty());
            return Applied::Milestone(chosen.and_then(|v| {
                let id = v.parse::<u64>().ok()?;
                let title = self
                    .items
                    .iter()
                    .find(|i| i.value == v)
                    .map(|i| i.label.clone())?;
                Some(MilestoneRef { id, title })
            }));
        }
        // Keep the host's own order for what was loaded, then anything
        // checked that the list does not know (a label typed elsewhere).
        let mut out: Vec<String> = self
            .items
            .iter()
            .filter(|i| self.checked.contains(&i.value))
            .map(|i| i.value.clone())
            .collect();
        for c in &self.checked {
            if !out.contains(c) {
                out.push(c.clone());
            }
        }
        Applied::List(self.field, out)
    }

    /// Whether the user may apply now: the host answered (or failed with
    /// something an empty choice can still be applied over is not offered).
    fn can_apply(&self) -> bool {
        matches!(self.status, Status::Ready)
    }

    fn apply(&mut self) -> PickerOutcome {
        if !self.can_apply() {
            self.message = Some(match &self.status {
                Status::Loading => gitpoulpi_i18n::msg::meta_picker::STILL_LOADING.get().into(),
                Status::Failed(why) => why.clone(),
                Status::Ready => String::new(),
            });
            return PickerOutcome::Stay;
        }
        PickerOutcome::Apply(self.applied())
    }

    pub(crate) fn paste(&mut self, text: &str) {
        if self.field.is_single() {
            self.combo.paste(text);
            return;
        }
        let clean: String = text.chars().filter(|c| !c.is_control()).collect();
        self.edit.sync(Some(()));
        let mut query = std::mem::take(&mut self.query);
        self.edit.caret_mut(()).insert(&mut query, &clean);
        self.query = query;
        self.refilter();
    }

    fn order(&self) -> [Focus; 3] {
        [
            if self.field.is_single() {
                Focus::Combo
            } else {
                Focus::List
            },
            Focus::Apply,
            Focus::Cancel,
        ]
    }

    fn step_focus(&mut self, forward: bool) {
        let order = self.order();
        let at = order.iter().position(|f| *f == self.focus).unwrap_or(0);
        let next = if forward {
            (at + 1) % order.len()
        } else {
            (at + order.len() - 1) % order.len()
        };
        self.focus = order[next];
    }

    pub(crate) fn on_key(&mut self, key: KeyEvent) -> PickerOutcome {
        let shift = key.modifiers.contains(KeyModifiers::SHIFT);
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        // The milestone's dropdown is modal while it is open.
        if self.field.is_single() && self.combo.is_open() {
            let outcome = self.combo.on_key(key);
            if let ComboOutcome::Picked(_) = outcome {
                self.focus = Focus::Apply;
            }
            return PickerOutcome::Stay;
        }
        match key.code {
            KeyCode::Esc => return PickerOutcome::Cancel,
            KeyCode::Tab => {
                self.step_focus(true);
                return PickerOutcome::Stay;
            }
            KeyCode::BackTab => {
                self.step_focus(false);
                return PickerOutcome::Stay;
            }
            _ => {}
        }
        match self.focus {
            Focus::Cancel => match key.code {
                KeyCode::Enter | KeyCode::Char(' ') => return PickerOutcome::Cancel,
                KeyCode::Left | KeyCode::Up => self.focus = Focus::Apply,
                _ => {}
            },
            Focus::Apply => match key.code {
                KeyCode::Enter | KeyCode::Char(' ') => return self.apply(),
                KeyCode::Right | KeyCode::Down => self.focus = Focus::Cancel,
                KeyCode::Up | KeyCode::Left => self.focus = self.order()[0],
                _ => {}
            },
            Focus::Combo => {
                if key.code == KeyCode::Enter && !ctrl {
                    // Enter on the field opens the list; a second Enter on
                    // Apply applies.
                    self.combo.open();
                } else {
                    self.combo.on_key(key);
                }
            }
            Focus::List => {
                if let Some(to) =
                    crate::popup_list::nav(&key, self.cursor, self.shown.len(), self.rows_visible)
                {
                    if !self.shown.is_empty() {
                        self.cursor = to;
                        self.keep_visible();
                    }
                    return PickerOutcome::Stay;
                }
                match key.code {
                    KeyCode::Enter => {
                        // Enter on the offer of the typed text takes it.
                        if let Some(free) = self.free {
                            let on_it = self.shown.get(self.cursor) == Some(&free);
                            if on_it && !self.checked.contains(&self.items[free].value) {
                                self.toggle_at(self.cursor);
                            }
                        }
                        return self.apply();
                    }
                    KeyCode::Char(' ') if !ctrl => self.toggle_at(self.cursor),
                    _ => {
                        self.edit.sync(Some(()));
                        let mut query = std::mem::take(&mut self.query);
                        let handled = self.edit.dialog_key((), &mut query, &key);
                        self.query = query;
                        if handled.is_some() {
                            self.refilter();
                        }
                    }
                }
            }
        }
        let _ = shift;
        PickerOutcome::Stay
    }

    pub(crate) fn on_mouse(&mut self, mouse: MouseEvent) -> PickerOutcome {
        if self.field.is_single() && self.combo.is_open() {
            if let ComboOutcome::Picked(_) = self.combo.on_mouse(mouse) {
                self.focus = Focus::Apply;
            }
            return PickerOutcome::Stay;
        }
        // The list's scrollbar: thumb drag, track jump, end arrows.
        if let Some(top) = self
            .bar_drag
            .on_mouse(self.bar, mouse, self.shown.len(), self.offset)
        {
            self.offset = top.min(self.shown.len().saturating_sub(self.rows_visible));
            return PickerOutcome::Stay;
        }
        let hit = dialog_kit::hit_at(&self.hits, mouse.column, mouse.row);
        if mouse.kind == MouseEventKind::Down(MouseButton::Left) {
            // The cross and a click outside ask the one close rule: outside
            // is swallowed (the cross flashes) once the selection changed.
            let kind = if self.close.hit(mouse.column, mouse.row) {
                Some(dialog_kit::DismissKind::Cross)
            } else if !self.frame.contains((mouse.column, mouse.row).into()) {
                Some(dialog_kit::DismissKind::Outside)
            } else {
                None
            };
            if let Some(kind) = kind {
                return match dialog_kit::dismiss(&*self, kind) {
                    dialog_kit::DismissAction::Swallow => {
                        self.close.request_flash();
                        PickerOutcome::Stay
                    }
                    _ => PickerOutcome::Cancel,
                };
            }
        }
        match mouse.kind {
            MouseEventKind::Moved | MouseEventKind::Drag(_) => {
                self.close.set_hover(mouse.column, mouse.row);
                if !self.edit.is_dragging() {
                    self.hover = hit;
                } else {
                    self.edit.dialog_mouse(
                        Some(()),
                        &self.query,
                        mouse.kind,
                        mouse.column,
                        mouse.row,
                    );
                }
            }
            MouseEventKind::ScrollUp => self.scroll(-(crate::popup_list::WHEEL_STEP as isize)),
            MouseEventKind::ScrollDown => self.scroll(crate::popup_list::WHEEL_STEP as isize),
            MouseEventKind::Up(MouseButton::Left) => {
                self.edit
                    .dialog_mouse(Some(()), &self.query, mouse.kind, mouse.column, mouse.row);
            }
            MouseEventKind::Down(MouseButton::Left) => match hit {
                Some(Hit::Row(i)) => {
                    self.focus = Focus::List;
                    self.toggle_at(i);
                }
                Some(Hit::Filter) => {
                    self.focus = Focus::List;
                    self.edit.sync(Some(()));
                    self.edit.dialog_mouse(
                        Some(()),
                        &self.query,
                        mouse.kind,
                        mouse.column,
                        mouse.row,
                    );
                }
                Some(Hit::Combo) => {
                    self.focus = Focus::Combo;
                    self.combo.open();
                }
                Some(Hit::Apply) => {
                    self.focus = Focus::Apply;
                    return self.apply();
                }
                Some(Hit::Cancel) => return PickerOutcome::Cancel,
                // A click inside the frame but on no control does nothing.
                None => {}
            },
            _ => {}
        }
        PickerOutcome::Stay
    }

    fn scroll(&mut self, delta: isize) {
        let max = self.shown.len().saturating_sub(self.rows_visible);
        self.offset = (self.offset as isize + delta).clamp(0, max as isize) as usize;
    }
}

impl dialog_kit::Dismiss for MetaPicker {
    /// The selection was changed since the picker opened.
    fn is_dirty(&self) -> bool {
        self.touched
    }

    fn close_hit(&self) -> &dialog_kit::CloseHit {
        &self.close
    }

    fn close_hit_mut(&mut self) -> &mut dialog_kit::CloseHit {
        &mut self.close
    }
}

/// The values an item currently has for `field`.
fn current_values(field: MetaField, current: &ItemMeta) -> Vec<String> {
    match field {
        MetaField::Assignees => current.assignees.clone(),
        MetaField::Reviewers => current.reviewers.clone(),
        MetaField::Labels => current.labels.clone(),
        MetaField::Milestone => current.milestone.iter().map(|m| m.id.to_string()).collect(),
    }
}
