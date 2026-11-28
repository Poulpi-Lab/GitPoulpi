//! The dropdowns of the workbench: what each one lists, where its cursor is
//! and what picking a row does. Keys and mouse both end in
//! [`RewriteView::popup_pick`], so they cannot drift apart. Painting is
//! `render.rs`; nothing here draws.

#[allow(unused_imports)]
use crate::rewrite_view::RuleTemplateExt;
use gitpoulpi_i18n::{msg::rewrite_view as t, M};
use gitpoulpi_ipc::rewrite::RowFilter;
use gitpoulpi_session::RecipePlace;

use super::{FieldValue, Focus, Popup, RewriteView, RuleGroup, RuleTemplate};

/// The filters of the Commits tab, in menu order.
pub(crate) const ROW_FILTERS: [(RowFilter, M); 4] = [
    (RowFilter::Changed, t::FILTER_CHANGED),
    (RowFilter::Dropped, t::FILTER_DROPPED),
    (RowFilter::Pushed, t::FILTER_PUSHED),
    (RowFilter::All, t::FILTER_ALL),
];

/// One row of an open dropdown.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PopupRow {
    pub label: String,
    /// Dim text after the label (where a saved recipe lives).
    pub note: Option<M>,
    /// A checklist row: ticked or not.
    pub checked: Option<bool>,
    /// A heading drawn above this row (the Add menu's groups).
    pub heading: Option<M>,
    /// The row is the value the field holds now.
    pub current: bool,
    /// Why the row cannot be picked (it is drawn dim, with this beside it).
    pub disabled: Option<M>,
}

impl PopupRow {
    fn plain(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            note: None,
            checked: None,
            heading: None,
            current: false,
            disabled: None,
        }
    }
}

impl RewriteView {
    pub(crate) fn open_scope_menu(&mut self) {
        let kinds = self.scope_kinds();
        let at = kinds
            .iter()
            .position(|k| *k == self.scope_kind())
            .unwrap_or(0);
        self.popup = Some(Popup::Scope { cursor: at });
    }

    /// The branches and tags checklist (only for a refs scope).
    pub(crate) fn open_refs_menu(&mut self) {
        if !self.ref_choices().is_empty() && self.scope_kind() == super::ScopeKind::Refs {
            self.popup = Some(Popup::Refs { cursor: 0 });
        }
    }

    pub(crate) fn open_save_menu(&mut self) {
        self.popup = Some(Popup::Save { cursor: 0 });
    }

    pub(crate) fn open_filter_menu(&mut self) {
        let at = ROW_FILTERS
            .iter()
            .position(|(f, _)| *f == self.rows.filter)
            .unwrap_or(0);
        self.popup = Some(Popup::Filter { cursor: at });
    }

    /// Opens the dropdown of a choice field, types into a text field, flips
    /// a toggle: the one entry for Enter, Space and a click on a field.
    pub(crate) fn open_field(&mut self, at: usize) {
        match self.detail.fields.get(at).map(|f| &f.value) {
            Some(FieldValue::Choice { index, .. }) => {
                self.popup = Some(Popup::Choice {
                    field: at,
                    cursor: *index,
                });
            }
            Some(_) => self.activate_field(at),
            None => {}
        }
    }

    /// "Only commits after X": the scope starts after the commit the Log menu
    /// named, or goes back to every commit.
    pub(crate) fn toggle_since(&mut self) {
        let next = match super::scope::since(&self.recipe) {
            Some(_) => None,
            None => self.scope_ctx.since,
        };
        if super::scope::set_since(&mut self.recipe, next) {
            self.touch();
        }
    }

    /// Whether the since checkbox is offered, and whether it is ticked.
    pub(crate) fn since_offer(&self) -> Option<(gitpoulpi_core::domain::Oid, bool)> {
        let id = self.scope_ctx.since?;
        (self.scope_kind() == super::ScopeKind::CurrentBranch)
            .then(|| (id, super::scope::since(&self.recipe).is_some()))
    }

    pub(crate) fn popup_len(&self) -> usize {
        match &self.popup {
            Some(Popup::AddRule { .. }) => RuleTemplate::ALL.len(),
            Some(Popup::Load { items, .. }) => items.len(),
            Some(Popup::Choice { field, .. }) => match self.detail.fields.get(*field) {
                Some(f) => match &f.value {
                    FieldValue::Choice { options, .. } => options.len(),
                    _ => 0,
                },
                None => 0,
            },
            Some(Popup::Scope { .. }) => self.scope_kinds().len(),
            Some(Popup::Save { .. }) => 2,
            Some(Popup::Filter { .. }) => ROW_FILTERS.len(),
            Some(Popup::Refs { .. }) => self.ref_choices().len(),
            None => 0,
        }
    }

    pub(crate) fn popup_cursor(&self) -> usize {
        match &self.popup {
            Some(
                Popup::AddRule { cursor }
                | Popup::Load { cursor, .. }
                | Popup::Choice { cursor, .. }
                | Popup::Scope { cursor }
                | Popup::Save { cursor }
                | Popup::Filter { cursor }
                | Popup::Refs { cursor },
            ) => *cursor,
            None => 0,
        }
    }

    pub(crate) fn popup_set_cursor(&mut self, to: usize) {
        if let Some(
            Popup::AddRule { cursor }
            | Popup::Load { cursor, .. }
            | Popup::Choice { cursor, .. }
            | Popup::Scope { cursor }
            | Popup::Save { cursor }
            | Popup::Filter { cursor }
            | Popup::Refs { cursor },
        ) = &mut self.popup
        {
            *cursor = to;
        }
    }

    /// The rows of the open dropdown, ready to draw.
    pub(crate) fn popup_rows(&self) -> Vec<PopupRow> {
        match &self.popup {
            Some(Popup::AddRule { .. }) => {
                let mut last: Option<RuleGroup> = None;
                RuleTemplate::ALL
                    .iter()
                    .map(|template| {
                        let mut row = PopupRow::plain(template.label().get());
                        row.disabled = template.disabled_reason();
                        if last != Some(template.group()) {
                            row.heading = Some(template.group().label());
                            last = Some(template.group());
                        }
                        row
                    })
                    .collect()
            }
            Some(Popup::Load { items, .. }) => items
                .iter()
                .map(|item| {
                    let mut row = PopupRow::plain(item.name.clone());
                    row.note = Some(match item.place {
                        RecipePlace::Shared => t::PLACE_SHARED,
                        RecipePlace::Mine => t::PLACE_MINE,
                    });
                    row
                })
                .collect(),
            Some(Popup::Choice { field, .. }) => match self.detail.fields.get(*field) {
                Some(f) => match &f.value {
                    FieldValue::Choice { options, index } => options
                        .iter()
                        .enumerate()
                        .map(|(i, m)| {
                            let mut row = PopupRow::plain(m.get());
                            row.current = i == *index;
                            if f.id == super::FieldId::ScriptLanguage {
                                row.disabled = self.language_row_blocker(i);
                            }
                            row
                        })
                        .collect(),
                    _ => Vec::new(),
                },
                None => Vec::new(),
            },
            Some(Popup::Scope { .. }) => self
                .scope_kinds()
                .into_iter()
                .map(|k| {
                    let mut row = PopupRow::plain(k.label().get());
                    row.current = k == self.scope_kind();
                    row
                })
                .collect(),
            Some(Popup::Save { .. }) => vec![
                PopupRow::plain(t::SAVE_SHARED.get()),
                PopupRow::plain(t::SAVE_MINE.get()),
            ],
            Some(Popup::Filter { .. }) => ROW_FILTERS
                .iter()
                .map(|(f, m)| {
                    let mut row = PopupRow::plain(m.get());
                    row.current = *f == self.rows.filter;
                    row
                })
                .collect(),
            Some(Popup::Refs { .. }) => self
                .ref_choices()
                .into_iter()
                .map(|c| {
                    let mut row = PopupRow::plain(c.name);
                    row.checked = Some(c.checked);
                    row
                })
                .collect(),
            None => Vec::new(),
        }
    }

    /// Picks row `index` of the open dropdown (Enter, or a click).
    pub(crate) fn popup_pick(&mut self, index: usize) {
        let Some(popup) = self.popup.clone() else {
            return;
        };
        if index >= self.popup_len() {
            return;
        }
        match popup {
            Popup::Refs { .. } => {
                let name = self.ref_choices()[index].name.clone();
                self.toggle_ref(&name);
                self.popup_set_cursor(index);
            }
            Popup::AddRule { .. } => {
                let template = RuleTemplate::ALL[index];
                if let Some(why) = template.disabled_reason() {
                    self.popup = None;
                    self.notice = Some(super::Notice {
                        text: format!("{}: {}", template.label().get(), why.get()),
                        error: true,
                    });
                    return;
                }
                self.popup = None;
                self.add_rule(template);
                self.focus = Focus::Detail;
                if template == RuleTemplate::Script {
                    self.open_script_editor();
                }
            }
            Popup::Load { items, .. } => {
                self.popup = None;
                self.load(&items[index]);
            }
            Popup::Choice { field, .. } => {
                self.popup = None;
                self.edit_field(field, super::FieldInput::Choice(index));
            }
            Popup::Scope { .. } => {
                self.popup = None;
                let kinds = self.scope_kinds();
                self.set_scope_kind(kinds[index]);
            }
            Popup::Save { .. } => {
                self.popup = None;
                self.save(if index == 0 {
                    RecipePlace::Shared
                } else {
                    RecipePlace::Mine
                });
            }
            Popup::Filter { .. } => {
                self.popup = None;
                self.set_row_filter(ROW_FILTERS[index].0);
            }
        }
    }
}
