//! The Images page: the viewer settings (a few plain rows, placed by
//! `pages::IMAGES`) and the editor list behind the image view's "Edit in"
//! combobox (`[[ui.image_editors]]`, `ui.image_editor_default`).
//!
//! The editor list is the Open in page's, over the image editors catalog
//! (`gitpoulpi_config::image_editors`): the list on screen is
//! `config.ui.image_editors` when the file has one, else what this machine has
//! installed (found by the background lookup `image_editor::install` starts).
//! The first edit turns that into a real `[[ui.image_editors]]` array and
//! every edit rewrites the whole array. It reuses the Open in page's
//! `OpenInTarget` / `OpenInButton` row kinds under its own `Target` /
//! `PrefAction` variants.

use super::list_editor::{AddKind, ListItem, ListOp, ListSpec, Mark};
use super::*;
use gitpoulpi_config::image_editors::{self, Detected};
use gitpoulpi_config::open_in::{OpenInEntry, Os};
use gitpoulpi_config::set_image_editors_in_file;
use gitpoulpi_i18n::msg::{
    settings_images as t, settings_open_in as open_in_t, settings_screen as screen,
};
use model::{OpenInButton, OpenInTarget};

/// The variable chips under the Command row.
pub(super) const IMAGE_VARIABLES: [&str; 4] = ["${file}", "${dir}", "${name}", "${repo}"];

/// The picture the Test button opens: a 4x4 RGBA PNG.
const SAMPLE_PNG: &[u8] = &[
    0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x04, 0x08, 0x06, 0x00, 0x00, 0x00, 0xa9, 0xf1, 0x9e,
    0x7e, 0x00, 0x00, 0x00, 0x29, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9c, 0x15, 0xc8, 0xa1, 0x01, 0x00,
    0x30, 0x0c, 0xc3, 0xb0, 0x3c, 0x99, 0xff, 0x86, 0xfd, 0x64, 0xe6, 0x02, 0x11, 0x25, 0x79, 0xab,
    0xd0, 0x94, 0xd4, 0x10, 0x5a, 0x2f, 0x30, 0x84, 0xc6, 0xc5, 0x0c, 0xa1, 0xe9, 0x03, 0xf7, 0xd4,
    0x29, 0xe1, 0x93, 0x9e, 0x4c, 0xd9, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4e, 0x44, 0xae, 0x42,
    0x60, 0x82,
];

impl PreferencesState {
    /// What the machine has: the page's own (tests), else the shared lookup.
    /// Until it answers only the OS default app is known.
    fn images_detected(&self) -> Option<&Detected> {
        self.image_detected
            .get()
            .or(crate::image_editor::detected_ready())
    }

    /// The background lookup has not answered yet. Starts it (a no-op once
    /// started) when the page is the one being shown.
    pub(super) fn images_detecting(&self) -> bool {
        if self.current_section() != PrefSection::Images {
            return false;
        }
        if self.images_detected().is_none() {
            crate::image_editor::install(&self.config.ui);
        }
        self.images_detected().is_none()
    }

    /// The list the page edits: the saved one, else the computed one.
    pub(super) fn images_list(&self) -> Vec<OpenInEntry> {
        if let Some(list) = &self.config.ui.image_editors {
            return list.clone();
        }
        match self.image_detected.get() {
            Some(found) => image_editors::effective_list(found, Os::current()),
            None => crate::image_editor::effective_entries_ready(&self.config.ui),
        }
    }

    fn images_selected(&self, list: &[OpenInEntry]) -> Option<usize> {
        (!list.is_empty()).then(|| self.image_index.min(list.len() - 1))
    }

    /// The Add preset options: a placeholder, then the presets of this OS,
    /// the detected ones first and marked. Entries already listed are left out.
    pub(super) fn images_preset_options(&self) -> Vec<(String, Option<OpenInEntry>)> {
        let os = Os::current();
        let placeholder = if self.images_detected().is_none() {
            open_in_t::DETECTING.get()
        } else {
            open_in_t::PRESET_PLACEHOLDER.get()
        };
        let mut out = vec![(placeholder.to_string(), None)];
        let mut seen = self.images_list();
        let none = Detected::default();
        let detected = self.images_detected().unwrap_or(&none);
        for preset in image_editors::picker_order(os, detected) {
            let entry = preset.entry();
            if seen.contains(&entry) {
                continue;
            }
            let mut text = String::new();
            if detected.has(preset.id) {
                text.push_str(open_in_t::GROUP_DETECTED.get());
                text.push_str(": ");
            }
            text.push_str(&entry.label);
            seen.push(entry.clone());
            out.push((text, Some(entry)));
        }
        out
    }

    /// Images > General: the viewer, the default editor, Restore.
    pub(super) fn images_general_fields(&self) -> Vec<Field> {
        self.images_general_fields_for(crate::build_features::IMAGE_VIEW)
    }

    /// The page with or without the viewer rows (`viewer` is whether the
    /// build has `image-view`).
    pub(super) fn images_general_fields_for(&self, viewer: bool) -> Vec<Field> {
        let list = self.images_list();
        let labels: Vec<String> = list.iter().map(|e| e.label.clone()).collect();
        let mut fields = self.picked(if viewer {
            pages::IMAGES
        } else {
            pages::IMAGES_EDITORS_ONLY
        });
        if !list.is_empty() {
            fields.push(
                Field::global(
                    open_in_t::DEFAULT_ENTRY.get(),
                    "ui.image_editor_default",
                    Control::Choice {
                        options: labels.clone(),
                        index: image_editors::default_index(
                            &list,
                            &self.config.ui.image_editor_default,
                        ),
                    },
                )
                .with_help(t::DEFAULT_ENTRY_HELP.get()),
            );
        }
        fields.push(
            Field::button(
                open_in_t::RESTORE.get(),
                PrefAction::ImageEditor(OpenInButton::Restore),
            )
            .with_help(t::RESTORE_HELP.get()),
        );
        fields
    }

    /// Images > the list tab: the rows of the selected editor.
    pub(super) fn images_item_fields(&self) -> Vec<Field> {
        let list = self.images_list();
        let mut fields = Vec::new();
        if let Some(sel) = self.images_selected(&list) {
            let entry = &list[sel];
            let editor = |label: &str, control: Control, which: OpenInTarget| {
                Field::plain(label, control, Target::ImageEditor(which))
            };
            fields.push(Field::heading(&open_in_t::editing_entry(
                sel + 1,
                list.len(),
            )));
            fields.push(
                editor(
                    open_in_t::LABEL.get(),
                    Control::Text(entry.label.clone()),
                    OpenInTarget::Label,
                )
                .with_help(open_in_t::LABEL_HELP.get()),
            );
            fields.push(
                editor(
                    open_in_t::COMMAND.get(),
                    Control::Text(entry.command.clone()),
                    OpenInTarget::Command,
                )
                .with_help(t::COMMAND_HELP.get()),
            );
            fields.push(editor(
                open_in_t::INSERT_VARIABLE.get(),
                Control::Chips(IMAGE_VARIABLES.iter().map(|v| v.to_string()).collect()),
                OpenInTarget::Chips,
            ));
            fields.push(
                Field::button(t::TEST.get(), PrefAction::ImageEditor(OpenInButton::Test))
                    .with_help(t::TEST_HELP.get()),
            );
        } else {
            fields.push(Field::heading(open_in_t::HEADING_NO_ENTRIES.get()));
        }
        fields
    }

    /// The master list: one row per editor, the default one starred.
    pub(super) fn images_list_spec(&self) -> ListSpec {
        let list = self.images_list();
        let default = image_editors::default_index(&list, &self.config.ui.image_editor_default);
        let items = list
            .iter()
            .enumerate()
            .map(|(i, e)| ListItem {
                label: e.label.clone(),
                sub: None,
                mark: if i == default {
                    Mark::Default
                } else {
                    Mark::None
                },
                pristine: e.label == open_in_t::NEW_ENTRY_LABEL.get() && e.command.is_empty(),
                ..ListItem::default()
            })
            .collect();
        let mut menu: Vec<String> = self
            .images_preset_options()
            .into_iter()
            .skip(1)
            .map(|(label, _)| label)
            .collect();
        menu.push(open_in_t::ADD_CUSTOM.get().to_string());
        ListSpec {
            title: t::LIST_TITLE.get().to_string(),
            items,
            add: AddKind::Menu(menu),
            reorder: true,
            ..ListSpec::default()
        }
    }

    /// Applies one list gesture to the editors.
    pub(super) fn images_op(&mut self, op: ListOp) {
        let mut list = self.images_list();
        let len = list.len();
        match op {
            ListOp::Add(Some(pick)) => {
                let presets = self.images_preset_options();
                match presets.into_iter().skip(1).nth(pick).and_then(|(_, e)| e) {
                    Some(entry) => self.images_add(entry),
                    None => {
                        self.images_add(OpenInEntry::new(open_in_t::NEW_ENTRY_LABEL.get(), ""));
                        self.focus_detail_target(|t| {
                            matches!(t, Target::ImageEditor(OpenInTarget::Command))
                        });
                    }
                }
            }
            ListOp::Remove(i) if i < len => {
                let removed = list.remove(i);
                self.image_index = i.min(len.saturating_sub(2));
                self.images_apply(list);
                if self.config.ui.image_editor_default == removed.label {
                    self.write_global("ui.image_editor_default", "");
                }
                self.field = self.first_selectable();
            }
            ListOp::Move { from, to } if from < len && to < len => {
                let item = list.remove(from);
                list.insert(to, item);
                self.image_index = to;
                self.images_apply(list);
            }
            _ => {}
        }
    }

    // -- persisting -----------------------------------------------------

    /// Writes `config.ui.image_editors` (the whole array, or the key's
    /// absence) and republishes the live list.
    fn write_image_editors(&mut self) {
        let Some(path) = self.config_file() else {
            self.message = Some(screen::NO_CONFIG_DIR.get().into());
            return;
        };
        match set_image_editors_in_file(&path, self.config.ui.image_editors.as_deref()) {
            Ok(()) => self.message = Some(t::SAVED.get().into()),
            Err(err) => self.message = Some(format!("ui.image_editors: {err}")),
        }
        crate::image_editor::install(&self.config.ui);
    }

    fn images_apply(&mut self, list: Vec<OpenInEntry>) {
        self.config.ui.image_editors = Some(list);
        self.write_image_editors();
    }

    fn images_unique_label(list: &[OpenInEntry], base: &str) -> String {
        let taken = |c: &str| list.iter().any(|e| e.label == c);
        if !taken(base) {
            return base.to_string();
        }
        (2..)
            .map(|n| format!("{base} ({n})"))
            .find(|c| !taken(c))
            .expect("the range is unbounded")
    }

    /// Appends `entry` (its label made unique) and selects it.
    pub(super) fn images_add(&mut self, mut entry: OpenInEntry) {
        let mut list = self.images_list();
        entry.label = Self::images_unique_label(&list, &entry.label);
        list.push(entry);
        self.image_index = list.len() - 1;
        self.images_apply(list);
    }

    /// Commits a typed Label or Command. `false`: refused (the message says
    /// why, the field keeps its text).
    pub(super) fn images_commit_text(&mut self, target: OpenInTarget, draft: &str) -> bool {
        let mut list = self.images_list();
        let Some(sel) = self.images_selected(&list) else {
            return true;
        };
        let value = draft.trim();
        match target {
            OpenInTarget::Label => {
                if value.is_empty() {
                    self.message = Some(open_in_t::LABEL_EMPTY.get().into());
                    return false;
                }
                if list
                    .iter()
                    .enumerate()
                    .any(|(i, e)| i != sel && e.label == value)
                {
                    self.message = Some(open_in_t::label_taken(value));
                    return false;
                }
                if list[sel].label == value {
                    return true;
                }
                let old = std::mem::replace(&mut list[sel].label, value.to_string());
                self.images_apply(list);
                if self.config.ui.image_editor_default == old {
                    self.write_global("ui.image_editor_default", value);
                }
            }
            OpenInTarget::Command => {
                if let Err(err) = image_editors::validate(value) {
                    self.message = Some(open_in_t::command_invalid(err));
                    return false;
                }
                if list[sel].command == value {
                    return true;
                }
                list[sel].command = value.to_string();
                self.images_apply(list);
            }
            _ => {}
        }
        true
    }

    /// A variable chip: appended to the selected editor's command.
    pub(super) fn images_insert_variable(&mut self, chip: usize) {
        let mut list = self.images_list();
        let (Some(sel), Some(var)) = (self.images_selected(&list), IMAGE_VARIABLES.get(chip))
        else {
            return;
        };
        let command = &mut list[sel].command;
        if !command.is_empty() && !command.ends_with(' ') {
            command.push(' ');
        }
        command.push_str(var);
        self.images_apply(list);
    }

    pub(super) fn images_button(&mut self, button: OpenInButton) {
        let list = self.images_list();
        let sel = self.images_selected(&list);
        match (button, sel) {
            (OpenInButton::Restore, _) => {
                self.config.ui.image_editors = None;
                self.image_index = 0;
                self.write_image_editors();
                if !self.config.ui.image_editor_default.is_empty() {
                    self.write_global("ui.image_editor_default", "");
                }
            }
            (OpenInButton::Test, Some(i)) => {
                let label = list[i].label.clone();
                crate::image_editor::install(&self.config.ui);
                let sample = crate::image_editor::materialise(
                    SAMPLE_PNG,
                    "settings-test",
                    "gitpoulpi-sample.png",
                );
                self.message = Some(match sample {
                    Ok(path) => match crate::image_editor::run(i, &path) {
                        Ok(_) => t::tested(&label),
                        Err(err) => crate::activity_log::report_problem_clear(
                            crate::activity_log::ProblemSource::Tool,
                            &open_in_t::test_failed(&label, err),
                        ),
                    },
                    Err(err) => crate::activity_log::report_problem_clear(
                        crate::activity_log::ProblemSource::Tool,
                        &open_in_t::test_failed(&label, err),
                    ),
                });
            }
            _ => {}
        }
    }
}
