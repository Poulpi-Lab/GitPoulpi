//! The Conflict prevention page (Settings > Repository; plan
//! `repo-settings.md`, slice 3) and the global "Conflict prevention" group of
//! Settings > General: the switches, and where the Base branches come from.
//!
//! A repository's Base branches are one of three things, chosen on its page:
//! the global **Default** (`conflict_prevention.default_list`, else `**`), a
//! named **List** shared by every repository that picks it, or its **Own
//! list**. The choice is in `repo-state.toml` (`RepoPreferences`:
//! `conflict_base_source`, `conflict_base_list`, `conflict_base_branches`),
//! the named lists in `config.toml` (`[conflict_prevention.lists.<name>]`);
//! `ConflictPreventionConfig::resolve` turns them into patterns, for the
//! daemon and for this page alike.
//!
//! Nothing here applies anything itself: the daemon reads the settings with
//! every check, and the window asks again when Settings closes
//! (`conflict_badge`), so an edited list or default reaches open repositories
//! without a restart. The General group edits the lists: create, rename, edit,
//! delete, make default, reset.

use gitpoulpi_config::{
    BaseList, BaseOrigin, ConflictPreventionConfig, BASE_SOURCE_DEFAULT, BASE_SOURCE_LIST,
    BASE_SOURCE_OWN,
};
use gitpoulpi_core::domain::conflict_prevention::parse_base_globs;
use gitpoulpi_i18n::msg::conflict_prevention as t;

use super::repo_pages::{RpButton, RpTarget};
use super::*;

/// The pattern table's rows: a pattern and what it matches.
type Example = (&'static str, fn() -> &'static str);
const EXAMPLES: [Example; 4] = [
    ("origin/main", || t::EX_ONE.get()),
    ("origin/**", || t::EX_REMOTE.get()),
    ("origin/feature/**", || t::EX_PREFIX.get()),
    ("!origin/abc", || t::EX_EXCLUDE.get()),
];

/// One entry of the "Base branches from" choice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum SourceChoice {
    Default,
    List(String),
    Own,
}

impl PreferencesState {
    fn lists(&self) -> &ConflictPreventionConfig {
        &self.config.conflict_prevention
    }

    /// The sorted names of the named lists.
    fn list_names(&self) -> Vec<String> {
        self.lists().lists.keys().cloned().collect()
    }

    /// What this repository follows now: its explicit choice, else what its
    /// older settings imply (own patterns when set).
    pub(super) fn source_choice(&self) -> SourceChoice {
        let prefs = &self.repo_state.preferences;
        let list = prefs
            .conflict_base_list
            .as_deref()
            .filter(|n| self.lists().lists.contains_key(*n))
            .map(str::to_string);
        let own_set = prefs
            .conflict_base_branches
            .as_deref()
            .is_some_and(|t| !parse_base_globs(t).is_empty());
        match prefs.conflict_base_source.as_deref() {
            Some(BASE_SOURCE_OWN) => SourceChoice::Own,
            Some(BASE_SOURCE_LIST) => list.map_or(SourceChoice::Default, SourceChoice::List),
            Some(BASE_SOURCE_DEFAULT) => SourceChoice::Default,
            _ if own_set => SourceChoice::Own,
            _ => list.map_or(SourceChoice::Default, SourceChoice::List),
        }
    }

    /// The choice's entries with their labels: Default, each list, Own list.
    fn source_options(&self) -> Vec<(String, SourceChoice)> {
        let default = match self.lists().default_patterns().origin {
            BaseOrigin::DefaultList(name) => t::source_default_list(&name),
            _ => t::SOURCE_DEFAULT_BUILT_IN.get().to_string(),
        };
        let mut options = vec![(default, SourceChoice::Default)];
        for name in self.list_names() {
            options.push((t::source_list(&name), SourceChoice::List(name)));
        }
        options.push((t::SOURCE_OWN.get().to_string(), SourceChoice::Own));
        options
    }

    /// The patterns this repository uses now.
    fn patterns_in_force(&self) -> Vec<String> {
        let prefs = &self.repo_state.preferences;
        self.lists()
            .resolve(
                prefs.conflict_base_source.as_deref(),
                prefs.conflict_base_list.as_deref(),
                prefs.conflict_base_branches.as_deref(),
            )
            .patterns
    }

    /// The Own list's text: what was saved, else the patterns in force (a
    /// start to edit from).
    pub(super) fn base_branches_text(&self) -> String {
        match &self.repo_state.preferences.conflict_base_branches {
            Some(text) if !parse_base_globs(text).is_empty() => text.clone(),
            _ => self.patterns_in_force().join("\n"),
        }
    }

    /// Saves the edited Own list. An empty one is stored as "not set".
    pub(super) fn set_base_branches(&mut self, text: &str) {
        let lines = parse_base_globs(text);
        let value = (!lines.is_empty()).then(|| lines.join("\n"));
        if value != self.repo_state.preferences.conflict_base_branches {
            self.repo_state.preferences.conflict_base_branches = value;
            self.write_repo_state();
        }
    }

    /// The base branches of this repository's branching workflow as patterns:
    /// each name as a local branch and as a branch of any remote.
    fn workflow_patterns(&self) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for base in &self.wf_draft().bases {
            for name in base.all_names() {
                for pattern in [name.to_string(), format!("*/{name}")] {
                    if !out.contains(&pattern) {
                        out.push(pattern);
                    }
                }
            }
        }
        out
    }

    // -- the repository page ----------------------------------------------------

    pub(super) fn conflict_fields(&self) -> Vec<Field> {
        let prefs = &self.repo_state.preferences;
        let choice = self.source_choice();
        let options = self.source_options();
        let index = options.iter().position(|(_, c)| *c == choice).unwrap_or(0);
        let mut fields = vec![
            Field::repo(
                t::REPO_ENABLE.get(),
                RepoField::ConflictDetection,
                Control::Toggle(prefs.conflict_detection.unwrap_or(true)),
            )
            .live()
            .with_help(&t::repo_enable_help(&self.repo_label)),
            Field::plain(
                t::SOURCE.get(),
                Control::Choice {
                    options: options.into_iter().map(|(label, _)| label).collect(),
                    index,
                },
                Target::RepoPage(RpTarget::BaseSource),
            )
            .with_help(t::SOURCE_HELP.get()),
        ];
        if choice == SourceChoice::Own {
            fields.push(
                Field::plain(
                    t::BASES.get(),
                    Control::Area(self.base_branches_text()),
                    Target::RepoPage(RpTarget::BaseBranches),
                )
                .with_placeholder(t::BASES_PLACEHOLDER.get())
                .with_help(t::BASES_HELP.get()),
            );
        } else {
            fields.push(
                Field::plain(
                    t::BASES.get(),
                    Control::Info(self.patterns_in_force().join("  ")),
                    Target::None,
                )
                .with_help(t::FOLLOWS_NOTE.get()),
            );
        }
        fields.push(
            Field::plain(
                t::SAVE_AS_LIST.get(),
                Control::Text(String::new()),
                Target::RepoPage(RpTarget::SaveAsList),
            )
            .with_help(t::SAVE_AS_LIST_HELP.get()),
        );
        let nothing_to_reset = prefs.conflict_base_source.is_none()
            && prefs.conflict_base_list.is_none()
            && prefs.conflict_base_branches.is_none();
        let mut reset = Btn::new(
            t::RESET_REPO.get(),
            PrefAction::RepoPage(RpButton::ConflictResetRepo),
        );
        reset.enabled = !nothing_to_reset;
        let mut buttons = vec![reset];
        if !self.workflow_patterns().is_empty() {
            buttons.push(Btn::new(
                t::USE_WORKFLOW.get(),
                PrefAction::RepoPage(RpButton::ConflictUseWorkflow),
            ));
        }
        fields.push(Field::buttons("", buttons).with_help(&format!(
            "{} {}",
            t::RESET_REPO_HELP.get(),
            t::USE_WORKFLOW_HELP.get()
        )));
        fields.push(Field::plain(
            t::EXPLAIN.get(),
            Control::Info(String::new()),
            Target::None,
        ));
        for (pattern, what) in EXAMPLES {
            fields.push(Field::plain(
                pattern,
                Control::Info(what().to_string()),
                Target::None,
            ));
        }
        fields
    }

    // -- the global group (Settings > General) ---------------------------------

    /// The rows after the global switch: the default list and the lists.
    pub(super) fn conflict_global_fields(&self) -> Vec<Field> {
        let names = self.list_names();
        let default = &self.lists().default_list;
        let mut options = vec![t::BUILT_IN.get().to_string()];
        options.extend(names.iter().cloned());
        let index = names.iter().position(|n| n == default).map_or(0, |i| i + 1);
        let mut fields = vec![Field::plain(
            t::DEFAULT_LIST.get(),
            Control::Choice { options, index },
            Target::RepoPage(RpTarget::ConflictDefaultList),
        )
        .with_help(t::DEFAULT_LIST_HELP.get())];
        let mut reset = Btn::new(
            t::RESET_DEFAULT.get(),
            PrefAction::RepoPage(RpButton::ConflictResetGlobal),
        );
        reset.enabled = !default.is_empty();
        fields.push(Field::buttons("", vec![reset]).with_help(t::RESET_DEFAULT_HELP.get()));
        fields.push(Field::heading(t::LISTS_HEADING.get()));
        if names.is_empty() {
            fields.push(Field::plain(
                t::LISTS_NONE.get(),
                Control::Info(String::new()),
                Target::None,
            ));
        }
        for (i, name) in names.iter().enumerate() {
            let is_default = name == default;
            fields.push(Field::heading(&if is_default {
                t::list_title_default(name)
            } else {
                name.clone()
            }));
            fields.push(
                Field::plain(
                    t::LIST_NAME.get(),
                    Control::Text(name.clone()),
                    Target::RepoPage(RpTarget::ConflictListName(i)),
                )
                .with_help(t::LIST_NAME_HELP.get()),
            );
            let text = self.lists().lists[name].patterns.join("\n");
            fields.push(
                Field::plain(
                    t::LIST_PATTERNS.get(),
                    Control::Area(text),
                    Target::RepoPage(RpTarget::ConflictListPatterns(i)),
                )
                .with_placeholder(t::BASES_PLACEHOLDER.get())
                .with_help(t::LIST_PATTERNS_HELP.get()),
            );
            let mut make = Btn::new(
                t::LIST_MAKE_DEFAULT.get(),
                PrefAction::RepoPage(RpButton::ConflictListDefault(i)),
            );
            make.enabled = !is_default;
            fields.push(Field::buttons(
                "",
                vec![
                    make,
                    Btn::new(
                        t::LIST_DELETE.get(),
                        PrefAction::RepoPage(RpButton::ConflictListDelete(i)),
                    ),
                ],
            ));
        }
        fields.push(
            Field::plain(
                t::LIST_NEW.get(),
                Control::Text(String::new()),
                Target::RepoPage(RpTarget::ConflictListNew),
            )
            .with_help(t::LIST_NEW_HELP.get()),
        );
        fields
    }

    /// The text of a list's patterns, for its editor.
    pub(super) fn conflict_list_text(&self, index: usize) -> String {
        self.list_names()
            .get(index)
            .and_then(|n| self.lists().lists.get(n))
            .map(|l| l.patterns.join("\n"))
            .unwrap_or_default()
    }

    /// Stores the edited patterns of the list at `index`.
    pub(super) fn set_conflict_list_patterns(&mut self, index: usize, text: &str) {
        let Some(name) = self.list_names().get(index).cloned() else {
            return;
        };
        let patterns = parse_base_globs(text);
        if let Some(list) = self.config.conflict_prevention.lists.get_mut(&name) {
            if list.patterns != patterns {
                list.patterns = patterns;
                self.write_conflict_lists(&t::list_saved(&name));
            }
        }
    }

    /// Writes the lists to `config.toml`; `done` is the status line.
    fn write_conflict_lists(&mut self, done: &str) {
        let Some(path) = self.config_file() else {
            self.message = Some(
                gitpoulpi_i18n::msg::settings_screen::NO_CONFIG_DIR
                    .get()
                    .into(),
            );
            return;
        };
        match gitpoulpi_config::set_conflict_prevention_in_file(
            &path,
            &self.config.conflict_prevention,
        ) {
            Ok(()) => self.message = Some(done.to_string()),
            Err(err) => {
                let text = format!("conflict_prevention: {err}");
                crate::activity_log::report_problem(
                    crate::activity_log::ProblemSource::Save,
                    &text,
                );
                self.message = Some(text);
            }
        }
    }

    /// A list name typed by the user: trimmed, not empty.
    fn clean_list_name(&mut self, draft: &str) -> Option<String> {
        let name = draft.trim().to_string();
        if name.is_empty() {
            self.message = Some(t::LIST_NAME_EMPTY.get().into());
            return None;
        }
        Some(name)
    }

    /// Creates the list `name` with these patterns; `false` (with the
    /// message) when the name is taken.
    fn create_list(&mut self, name: &str, patterns: Vec<String>) -> bool {
        if self.lists().lists.contains_key(name) {
            self.message = Some(t::list_exists(name));
            return false;
        }
        self.config
            .conflict_prevention
            .lists
            .insert(name.to_string(), BaseList { patterns });
        self.write_conflict_lists(&t::list_saved(name));
        true
    }

    /// The typed text of the rows of these pages. `false`: refused, the
    /// message says why and the field stays open.
    pub(super) fn conflict_commit_text(&mut self, target: RpTarget, draft: &str) -> bool {
        match target {
            RpTarget::ConflictListNew => {
                let Some(name) = self.clean_list_name(draft) else {
                    return false;
                };
                self.create_list(&name, ConflictPreventionConfig::built_in())
            }
            RpTarget::SaveAsList => {
                let Some(name) = self.clean_list_name(draft) else {
                    return false;
                };
                let patterns = self.patterns_in_force();
                if !self.create_list(&name, patterns) {
                    return false;
                }
                let prefs = &mut self.repo_state.preferences;
                prefs.conflict_base_source = Some(BASE_SOURCE_LIST.into());
                prefs.conflict_base_list = Some(name);
                self.write_repo_state();
                true
            }
            RpTarget::ConflictListName(index) => {
                let Some(old) = self.list_names().get(index).cloned() else {
                    return true;
                };
                let Some(name) = self.clean_list_name(draft) else {
                    return false;
                };
                if name == old {
                    return true;
                }
                if self.lists().lists.contains_key(&name) {
                    self.message = Some(t::list_exists(&name));
                    return false;
                }
                let section = &mut self.config.conflict_prevention;
                if let Some(list) = section.lists.remove(&old) {
                    section.lists.insert(name.clone(), list);
                }
                if section.default_list == old {
                    section.default_list = name.clone();
                }
                self.write_conflict_lists(&t::list_saved(&name));
                // This repository keeps following it; others fall back (the
                // help under the name says so).
                if self.repo_state.preferences.conflict_base_list.as_deref() == Some(old.as_str()) {
                    self.repo_state.preferences.conflict_base_list = Some(name);
                    self.write_repo_state();
                }
                true
            }
            _ => true,
        }
    }

    /// A pick of the two choices: the default list, or this repository's source.
    pub(super) fn conflict_commit_choice(&mut self, target: RpTarget, next: usize) {
        match target {
            RpTarget::ConflictDefaultList => {
                let name = next
                    .checked_sub(1)
                    .and_then(|i| self.list_names().get(i).cloned())
                    .unwrap_or_default();
                if self.config.conflict_prevention.default_list != name {
                    self.config.conflict_prevention.default_list = name.clone();
                    self.write_conflict_lists(&t::list_saved(&name));
                }
            }
            RpTarget::BaseSource => {
                let Some((_, choice)) = self.source_options().into_iter().nth(next) else {
                    return;
                };
                // Own list starts as a copy of what was in force.
                if choice == SourceChoice::Own {
                    let text = self.base_branches_text();
                    self.repo_state.preferences.conflict_base_branches =
                        Some(parse_base_globs(&text).join("\n"));
                }
                let prefs = &mut self.repo_state.preferences;
                match choice {
                    SourceChoice::Default => {
                        prefs.conflict_base_source = Some(BASE_SOURCE_DEFAULT.into())
                    }
                    SourceChoice::List(name) => {
                        prefs.conflict_base_source = Some(BASE_SOURCE_LIST.into());
                        prefs.conflict_base_list = Some(name);
                    }
                    SourceChoice::Own => prefs.conflict_base_source = Some(BASE_SOURCE_OWN.into()),
                }
                self.write_repo_state();
            }
            _ => {}
        }
    }

    /// The buttons of these pages.
    pub(super) fn conflict_button(&mut self, button: RpButton) {
        match button {
            RpButton::ConflictResetGlobal => {
                self.config.conflict_prevention.default_list.clear();
                self.write_conflict_lists(&t::RESET_DEFAULT.get().to_string());
            }
            RpButton::ConflictListDefault(index) => {
                if let Some(name) = self.list_names().get(index).cloned() {
                    self.config.conflict_prevention.default_list = name.clone();
                    self.write_conflict_lists(&t::list_saved(&name));
                }
            }
            RpButton::ConflictListDelete(index) => {
                let Some(name) = self.list_names().get(index).cloned() else {
                    return;
                };
                let section = &mut self.config.conflict_prevention;
                section.lists.remove(&name);
                if section.default_list == name {
                    section.default_list.clear();
                }
                self.write_conflict_lists(&t::list_deleted(&name));
            }
            RpButton::ConflictResetRepo => {
                let prefs = &mut self.repo_state.preferences;
                prefs.conflict_base_source = None;
                prefs.conflict_base_list = None;
                prefs.conflict_base_branches = None;
                self.write_repo_state();
            }
            RpButton::ConflictUseWorkflow => {
                let patterns = self.workflow_patterns();
                if patterns.is_empty() {
                    return;
                }
                let prefs = &mut self.repo_state.preferences;
                prefs.conflict_base_branches = Some(patterns.join("\n"));
                prefs.conflict_base_source = Some(BASE_SOURCE_OWN.into());
                self.write_repo_state();
            }
            _ => {}
        }
    }

    /// The question before a destructive button of these pages (`None`: the
    /// generic one).
    pub(super) fn conflict_confirm_text(&self, action: PrefAction) -> Option<String> {
        let PrefAction::RepoPage(RpButton::ConflictListDelete(index)) = action else {
            return None;
        };
        let name = self.list_names().get(index).cloned()?;
        let follows = self.repo_state.preferences.conflict_base_list.as_deref()
            == Some(name.as_str())
            && self.source_choice() == SourceChoice::List(name.clone());
        Some(if follows {
            t::list_delete_confirm_here(&name)
        } else {
            t::list_delete_confirm(&name)
        })
    }
}
