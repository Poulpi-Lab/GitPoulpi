//! The Open in page: the list behind the tab bar's "Open in" dropdown
//! (`[[ui.open_in]]`, `ui.open_in_default`).
//!
//! The list on screen is `config.ui.open_in` when the file has one, else the
//! computed list the tab bar shows while the key is absent (legacy terminal
//! entry, the OS explorer, what is detected). The first edit turns that into
//! a real `[[ui.open_in]]` array; every edit rewrites the whole array, the way
//! the Profiles page rewrites `[[profiles]]`.

use super::list_editor::{AddKind, ListItem, ListOp, ListSpec, Mark};
use super::*;
use gitpoulpi_config::open_in::{self, Detected, OpenInEntry, Os};
use gitpoulpi_config::set_open_in_in_file;
use gitpoulpi_i18n::msg::{settings_open_in as t, settings_screen as screen};
use model::{OpenInButton, OpenInTarget};

/// The variable chips under the Command row.
const VARIABLES: [&str; 5] = ["${cwd}", "${repo}", "${branch}", "${name}", "${web_url}"];

impl PreferencesState {
    /// What this machine runs: the process-wide lookup `tab_dir` runs once in
    /// the background (environment variables and `PATH`; nothing is spawned).
    /// Never waits: until it is done nothing is detected yet, and
    /// `open_in_detecting` says so.
    fn open_in_detected(&self) -> &Detected {
        static NONE: Detected = Detected {
            multiplexers: Vec::new(),
            terminals: Vec::new(),
            desktop: Vec::new(),
            editors: Vec::new(),
            installed: Vec::new(),
        };
        match (
            self.open_in_detected.get(),
            crate::tab_dir::detected_ready(),
        ) {
            (Some(own), _) => own,
            (None, Some(shared)) => shared,
            (None, None) => &NONE,
        }
    }

    /// The background lookup has not answered yet.
    pub(super) fn open_in_detecting(&self) -> bool {
        self.open_in_detected.get().is_none() && crate::tab_dir::detected_ready().is_none()
    }

    /// The list the page edits: the saved one, else the computed one.
    pub(super) fn open_in_list(&self) -> Vec<OpenInEntry> {
        if let Some(list) = &self.config.ui.open_in {
            return list.clone();
        }
        crate::tab_dir::effective_entries_ready(&self.config.ui)
    }

    /// The entry the rows edit (clamped; `None` for an empty list).
    fn open_in_selected(&self, list: &[OpenInEntry]) -> Option<usize> {
        (!list.is_empty()).then(|| self.open_in_index.min(list.len() - 1))
    }

    /// The Add preset options: a placeholder, then Detected, then the
    /// presets of this OS by group. Entries already in the list are left out.
    pub(super) fn open_in_preset_options(&self) -> Vec<(String, Option<OpenInEntry>)> {
        let os = Os::current();
        let have = self.open_in_list();
        let placeholder = if self.open_in_detecting() {
            t::DETECTING.get()
        } else {
            t::PRESET_PLACEHOLDER.get()
        };
        let mut out = vec![(placeholder.to_string(), None)];
        let mut seen: Vec<OpenInEntry> = have;
        let mut push = |group: &str, entry: OpenInEntry, out: &mut Vec<_>| {
            if seen.contains(&entry) {
                return;
            }
            let text = format!(
                "{}: {}",
                super::choice_labels::open_in_group(group),
                entry.label
            );
            seen.push(entry.clone());
            out.push((text, Some(entry)));
        };
        for entry in open_in::effective_list(None, self.open_in_detected(), os) {
            push("detected", entry, &mut out);
        }
        // The current OS's presets only, by category, the ones this machine
        // can use first.
        for preset in open_in::picker_order(os, self.open_in_detected()) {
            push(preset.group, preset.entry(), &mut out);
        }
        out
    }

    /// Open in > General: the rows that are not an entry's own.
    pub(super) fn open_in_general_fields(&self) -> Vec<Field> {
        let list = self.open_in_list();
        let labels: Vec<String> = list.iter().map(|e| e.label.clone()).collect();
        let mut fields = self.picked(pages::OPEN_IN);
        if !list.is_empty() {
            fields.push(
                Field::global(
                    t::DEFAULT_ENTRY.get(),
                    "ui.open_in_default",
                    Control::Choice {
                        options: labels.clone(),
                        index: open_in::default_index(&list, &self.config.ui.open_in_default),
                    },
                )
                .with_help(t::DEFAULT_ENTRY_HELP.get()),
            );
        }
        // The Restore row is page-level; the list's `+ Add` button holds the
        // presets and Custom, and the list tab edits the selected entry.
        fields.push(
            Field::button(t::RESTORE.get(), PrefAction::OpenIn(OpenInButton::Restore))
                .with_help(t::RESTORE_HELP.get()),
        );
        fields
    }

    /// Open in > the list tab: the rows of the selected entry.
    pub(super) fn open_in_item_fields(&self) -> Vec<Field> {
        let list = self.open_in_list();
        let mut fields = Vec::new();
        if let Some(sel) = self.open_in_selected(&list) {
            let entry = &list[sel];
            fields.push(Field::heading(&t::editing_entry(sel + 1, list.len())));
            fields.push(
                Field::plain(
                    t::LABEL.get(),
                    Control::Text(entry.label.clone()),
                    Target::OpenIn(OpenInTarget::Label),
                )
                .with_help(t::LABEL_HELP.get()),
            );
            fields.push(
                Field::plain(
                    t::COMMAND.get(),
                    Control::Text(entry.command.clone()),
                    Target::OpenIn(OpenInTarget::Command),
                )
                .with_help(t::COMMAND_HELP.get()),
            );
            fields.push(Field::plain(
                t::INSERT_VARIABLE.get(),
                Control::Chips(VARIABLES.iter().map(|v| v.to_string()).collect()),
                Target::OpenIn(OpenInTarget::Chips),
            ));
            fields.push(
                Field::button(t::TEST.get(), PrefAction::OpenIn(OpenInButton::Test))
                    .with_help(t::TEST_HELP.get()),
            );
        } else {
            fields.push(Field::heading(t::HEADING_NO_ENTRIES.get()));
        }
        fields
    }

    /// The master list: one row per entry, the default one starred.
    pub(super) fn open_in_list_spec(&self) -> ListSpec {
        let list = self.open_in_list();
        let default = open_in::default_index(&list, &self.config.ui.open_in_default);
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
                pristine: e.label == t::NEW_ENTRY_LABEL.get() && e.command.is_empty(),
                ..ListItem::default()
            })
            .collect();
        let mut menu: Vec<String> = self
            .open_in_preset_options()
            .into_iter()
            .skip(1)
            .map(|(label, _)| label)
            .collect();
        menu.push(t::ADD_CUSTOM.get().to_string());
        ListSpec {
            title: t::LIST_TITLE.get().to_string(),
            items,
            add: AddKind::Menu(menu),
            reorder: true,
            ..ListSpec::default()
        }
    }

    /// Applies one list gesture to the entries.
    pub(super) fn open_in_op(&mut self, op: ListOp) {
        let mut list = self.open_in_list();
        let len = list.len();
        match op {
            ListOp::Add(Some(pick)) => {
                let presets = self.open_in_preset_options();
                match presets.into_iter().skip(1).nth(pick).and_then(|(_, e)| e) {
                    Some(entry) => self.open_in_add(entry),
                    None => {
                        self.open_in_add(OpenInEntry::new(t::NEW_ENTRY_LABEL.get(), ""));
                        self.focus_detail_target(|t| {
                            matches!(t, Target::OpenIn(OpenInTarget::Command))
                        });
                    }
                }
            }
            ListOp::Remove(i) if i < len => {
                let removed = list.remove(i);
                self.open_in_index = i.min(len.saturating_sub(2));
                self.open_in_apply(list);
                if self.config.ui.open_in_default == removed.label {
                    self.write_global("ui.open_in_default", "");
                }
                self.field = self.first_selectable();
            }
            ListOp::Move { from, to } if from < len && to < len => {
                let item = list.remove(from);
                list.insert(to, item);
                self.open_in_index = to;
                self.open_in_apply(list);
            }
            _ => {}
        }
    }

    // -- persisting -----------------------------------------------------

    /// Writes `config.ui.open_in` (the whole array, or the key's absence)
    /// and republishes the live list.
    fn write_open_in(&mut self) {
        let Some(path) = self.config_file() else {
            self.message = Some(screen::NO_CONFIG_DIR.get().into());
            return;
        };
        match set_open_in_in_file(&path, self.config.ui.open_in.as_deref()) {
            Ok(()) => self.message = Some(t::SAVED.get().into()),
            Err(err) => self.message = Some(format!("ui.open_in: {err}")),
        }
        crate::tab_dir::install(&self.config.ui);
    }

    /// Replaces the list (turning an absent key into a saved one).
    fn open_in_apply(&mut self, list: Vec<OpenInEntry>) {
        self.config.ui.open_in = Some(list);
        self.write_open_in();
    }

    /// `base`, `base (2)`, ... not yet used as a label in `list`.
    fn open_in_unique_label(list: &[OpenInEntry], base: &str) -> String {
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
    pub(super) fn open_in_add(&mut self, mut entry: OpenInEntry) {
        let mut list = self.open_in_list();
        entry.label = Self::open_in_unique_label(&list, &entry.label);
        list.push(entry);
        self.open_in_index = list.len() - 1;
        self.open_in_apply(list);
    }

    /// Commits a typed Label or Command. `false`: refused (the message says
    /// why, the field keeps its text).
    pub(super) fn open_in_commit_text(&mut self, target: OpenInTarget, draft: &str) -> bool {
        let mut list = self.open_in_list();
        let Some(sel) = self.open_in_selected(&list) else {
            return true;
        };
        let value = draft.trim();
        match target {
            OpenInTarget::Label => {
                if value.is_empty() {
                    self.message = Some(t::LABEL_EMPTY.get().into());
                    return false;
                }
                if list
                    .iter()
                    .enumerate()
                    .any(|(i, e)| i != sel && e.label == value)
                {
                    self.message = Some(t::label_taken(value));
                    return false;
                }
                if list[sel].label == value {
                    return true;
                }
                let old = std::mem::replace(&mut list[sel].label, value.to_string());
                self.open_in_apply(list);
                if self.config.ui.open_in_default == old {
                    self.write_global("ui.open_in_default", value);
                }
            }
            OpenInTarget::Command => {
                if let Err(err) = open_in::validate(value) {
                    self.message = Some(t::command_invalid(err));
                    return false;
                }
                if list[sel].command == value {
                    return true;
                }
                list[sel].command = value.to_string();
                self.open_in_apply(list);
            }
            _ => {}
        }
        true
    }

    /// A variable chip: appended to the selected entry's command.
    pub(super) fn open_in_insert_variable(&mut self, chip: usize) {
        let mut list = self.open_in_list();
        let (Some(sel), Some(var)) = (self.open_in_selected(&list), VARIABLES.get(chip)) else {
            return;
        };
        let command = &mut list[sel].command;
        if !command.is_empty() && !command.ends_with(' ') {
            command.push(' ');
        }
        command.push_str(var);
        self.open_in_apply(list);
    }

    pub(super) fn open_in_button(&mut self, button: OpenInButton) {
        let list = self.open_in_list();
        let sel = self.open_in_selected(&list);
        match (button, sel) {
            (OpenInButton::Restore, _) => {
                self.config.ui.open_in = None;
                self.open_in_index = 0;
                self.write_open_in();
                if !self.config.ui.open_in_default.is_empty() {
                    self.write_global("ui.open_in_default", "");
                }
            }
            (OpenInButton::Test, Some(i)) => {
                let label = list[i].label.clone();
                crate::tab_dir::install(&self.config.ui);
                self.message = Some(match crate::tab_dir::run_open_in(i, &self.repo_root) {
                    Ok(_) => t::tested(&label),
                    Err(err) => crate::activity_log::report_problem_clear(
                        crate::activity_log::ProblemSource::Tool,
                        &t::test_failed(&label, err),
                    ),
                });
            }
            _ => {}
        }
    }
}
