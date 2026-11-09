//! Writing a Preferences edit to `config.toml`, the profiles file or the repo state.

use super::fields_ui::{encoding_options, git_config_values};
use super::*;
use gitpoulpi_i18n::msg::settings_screen as screen;

/// `profile`, `profile-2`, `profile-3`, ... - the first candidate the
/// predicate does not already claim.
pub(super) fn unique_id(stem: &str, taken: impl Fn(String) -> bool) -> String {
    if !taken(stem.to_string()) {
        return stem.to_string();
    }
    (2..)
        .map(|n| format!("{stem}-{n}"))
        .find(|candidate| !taken(candidate.clone()))
        .expect("the range is unbounded")
}

impl PreferencesState {
    // ---------------------------------------------------------------
    // Committing
    // ---------------------------------------------------------------

    pub(super) fn config_file(&self) -> Option<PathBuf> {
        self.config_path.clone()
    }

    /// Writes one dotted key and reloads the file layer, so what the
    /// screen shows next frame is what is actually on disk.
    /// `false` when the file refused the value (the message says why).
    pub(super) fn write_global(&mut self, key: &str, value: &str) -> bool {
        let Some(path) = self.config_file() else {
            self.message = Some(screen::NO_CONFIG_DIR.get().into());
            return false;
        };
        // `mcp.port` is not a stored key: it is the port of `mcp.bind`.
        let bind;
        let (key, value) = if key == "mcp.port" {
            let Ok(port) = value.parse::<u16>() else {
                self.message = Some(screen::not_whole_number(key, format!("{value:?}")));
                return false;
            };
            bind = self.config.mcp.bind_with_port(port);
            ("mcp.bind", bind.as_str())
        } else {
            (key, value)
        };
        match set_in_file(&path, key, value) {
            Ok(stored) => {
                self.config = Config::load(&path).unwrap_or_else(|_| self.config.clone());
                if matches!(
                    key,
                    "ui.language" | "ui.translate_git_terms" | "ui.git_terms_english"
                ) {
                    install_language(&self.config.ui);
                }
                if matches!(
                    key,
                    "ui.date_format"
                        | "ui.date_locale"
                        | "ui.date_time_format"
                        | "ui.date_verbose_format"
                ) {
                    install_date_patterns(&self.config.ui);
                }
                if matches!(
                    key,
                    "ui.diff_layout"
                        | "ui.diff_hunks_only"
                        | "ui.diff_show_whitespace"
                        | "ui.diff_word_wrap"
                ) {
                    install_diff_view_prefs(DiffViewPrefs {
                        layout: self.config.ui.diff_layout,
                        hunks_only: self.config.ui.diff_hunks_only,
                        show_whitespace: self.config.ui.diff_show_whitespace,
                        word_wrap: self.config.ui.diff_word_wrap,
                    });
                }
                if key.starts_with("editor.") {
                    crate::editor_prefs::install(&self.config.editor);
                }
                if key == "ui.log_wip_counts_in_message" {
                    install_log_wip_counts_in_message(self.config.ui.log_wip_counts_in_message);
                }
                if key == "ui.reveal_on_ref_click" {
                    crate::live_settings::install_reveal_on_ref_click(
                        self.config.ui.reveal_on_ref_click,
                    );
                }
                if key == "ui.hide_finished_pr_chip" {
                    crate::live_settings::install_hide_finished_pr_chip(
                        self.config.ui.hide_finished_pr_chip,
                    );
                }
                if key == "ui.graph_ghost_ref" {
                    crate::live_settings::install_graph_ghost_ref(self.config.ui.graph_ghost_ref);
                }
                if key == "general.group_worktree_tabs" {
                    crate::live_settings::install_group_worktree_tabs(
                        self.config.general.group_worktree_tabs,
                    );
                }
                if matches!(key, "ui.blame_side" | "ui.blame_width") {
                    crate::live_settings::install_blame(&self.config.ui);
                }
                if key == "ui.column_resize" {
                    crate::live_settings::install_column_resize(self.config.ui.column_resize);
                }
                if key == "ui.log_columns_layout" {
                    crate::live_settings::install_log_columns_layout(
                        self.config.ui.log_columns_layout,
                    );
                }
                if key == "ui.sidebar_sections_scope" {
                    crate::live_settings::install_sidebar_sections_global(
                        self.config.ui.sidebar_sections_scope
                            == gitpoulpi_config::SidebarSectionsScope::Global,
                    );
                }
                if matches!(
                    key,
                    "ui.toolbar_position"
                        | "ui.toolbar_align"
                        | "ui.toolbar_pr_pinned"
                        | "ui.statusbar_position"
                        | "ui.tabs_position"
                ) {
                    crate::live_settings::install_chrome_layout(&self.config.ui);
                }
                if key == "ui.minimap" {
                    install_minimap(self.config.ui.minimap);
                }
                if matches!(
                    key,
                    "ui.tab_dir_follow" | "ui.open_in_default" | "ui.launch" | "ui.launch_command"
                ) {
                    crate::tab_dir::install(&self.config.ui);
                }
                if key == "ui.image_editor_default" {
                    crate::image_editor::install(&self.config.ui);
                }
                if key == "ui.pr_terminology" {
                    crate::terminology::install(self.config.ui.pr_terminology);
                }
                if key == "ui.theme" {
                    self.install_stored_theme();
                }
                if key == "ui.input_color" || key == "ui.input_color_custom" {
                    crate::install_input_color(
                        &self.config.ui.input_color,
                        &self.config.ui.input_color_custom,
                    );
                }
                if key == "general.auto_prune" {
                    install_auto_prune(self.config.general.auto_prune);
                }
                if key == "general.auto_prune_tags" {
                    crate::live_settings::install_auto_prune_tags(
                        self.config.general.auto_prune_tags,
                    );
                }
                if key == "general.radar_enabled" {
                    crate::build_features::install_radar_enabled(self.config.general.radar_enabled);
                }
                if key == "general.delete_orig_files_after_merge" {
                    install_delete_orig_after_merge(
                        self.config.general.delete_orig_files_after_merge,
                    );
                }
                if key == "log.commit_count" {
                    crate::live_settings::install_commit_count(self.config.log.commit_count);
                }
                if key.starts_with("updates.") {
                    crate::updates::configure(&self.config, None);
                }
                if key == "general.active_profile" {
                    install_active_profile(&self.config);
                }
                if key.starts_with("general.external_") || key == "general.file_explorer" {
                    crate::external_tools::install(&self.config);
                }
                if key.starts_with("log.") {
                    crate::activity_log::install_timings(&self.config.log);
                    crate::activity_log::apply_log_settings(&self.config.log);
                }
                if key == "ai.enabled" {
                    self.ai_probe_keys();
                }
                self.message = Some(format!("{key} = {stored}"));
                true
            }
            Err(err) => {
                let text = format!("{key}: {err}");
                crate::activity_log::report_problem(
                    crate::activity_log::ProblemSource::Save,
                    &text,
                );
                self.message = Some(text);
                false
            }
        }
    }

    pub(super) fn write_profiles(&mut self) {
        let Some(path) = self.config_file() else {
            self.message = Some(screen::NO_CONFIG_DIR.get().into());
            return;
        };
        match set_list_in_file(&path, "profiles", &self.config.profiles) {
            Ok(()) => self.message = Some(screen::PROFILES_SAVED.get().into()),
            Err(err) => {
                let text = format!("profiles: {err}");
                crate::activity_log::report_problem(
                    crate::activity_log::ProblemSource::Save,
                    &text,
                );
                self.message = Some(text);
            }
        }
        // A profile's own name/email may just have changed under the
        // active profile's id -- reread, not just the id-to-id case
        // `write_global("general.active_profile", ..)` handles.
        install_active_profile(&self.config);
    }

    pub(super) fn write_repo_state(&mut self) {
        match self.repo_state.save(&self.repo_state_path) {
            Ok(()) => {
                self.message = Some(screen::saved_to(self.repo_state_path.display()));
            }
            Err(err) => {
                let text = screen::repo_settings_error(err);
                crate::activity_log::report_problem(
                    crate::activity_log::ProblemSource::Save,
                    &text,
                );
                self.message = Some(text);
            }
        }
    }

    pub(super) fn commit_bool(&mut self, target: &Target, value: bool) {
        match target {
            Target::Global(key) => {
                let key = *key;
                self.write_global(key, if value { "true" } else { "false" });
            }
            Target::Repo(RepoField::Lfs) => {
                self.repo_state.preferences.lfs_enabled = Some(value);
                self.write_repo_state();
            }
            Target::Repo(field) => self.commit_repo_toggle(*field, value),
            Target::Workflow(t) => self.wf_commit_bool(*t, value),
            Target::RepoAuth(t) => self.ra_commit_bool(*t, value),
            Target::Ai(t) => self.ai_commit_bool(*t, value),
            _ => {}
        }
    }

    pub(super) fn commit_choice(&mut self, target: &Target, next: usize) {
        match target.clone() {
            Target::Global("general.active_profile") => {
                let id = if next == 0 {
                    String::new()
                } else {
                    self.config
                        .profiles
                        .get(next - 1)
                        .map(|p| p.id.clone())
                        .unwrap_or_default()
                };
                self.profile_index = next.saturating_sub(1);
                self.write_global("general.active_profile", &id);
            }
            Target::Global(key) => {
                // Every other choice field's option labels *are* the
                // values the schema accepts, which is why they are
                // rendered from the enum's own `as_str`.
                let fields = self.fields();
                if let Some(Control::Choice { options, .. }) =
                    fields.get(self.field).map(|f| &f.control)
                {
                    if let Some(value) = options.get(next) {
                        let value = if key == "ui.language" {
                            super::fields_ui::language_choices(self.config.ui.language)
                                .get(next)
                                .map_or_else(|| value.clone(), |l| l.as_str().to_string())
                        } else if key == "ui.input_color" {
                            value.trim_end_matches('\u{2026}').to_ascii_lowercase()
                        } else {
                            value.clone()
                        };
                        self.write_global(key, &value);
                    }
                }
            }
            Target::Ai(t) => self.ai_commit_choice(t, next),
            Target::RepoAuth(t) => self.ra_commit_choice(t, next),
            Target::Repo(field) => {
                // Index 0 is "(inherited)", i.e. clear the override.
                // `core.autocrlf` / `core.eol` are git's own repository
                // config, not `repo-state.toml`: the event loop sends the
                // pick to the daemon (journaled), the next refresh confirms.
                if let Some(key) = field.git_config_key() {
                    let value = next
                        .checked_sub(1)
                        .and_then(|i| git_config_values(key).get(i))
                        .map(|v| (*v).to_string());
                    self.repo_config.insert(key.to_string(), value.clone());
                    self.repo_config_requests.push((key.to_string(), value));
                    return;
                }
                let value = if next == 0 {
                    None
                } else {
                    match field {
                        RepoField::Encoding => {
                            let current = self.repo_state.preferences.encoding.clone();
                            encoding_options(current.as_deref().unwrap_or(""))
                                .get(next - 1)
                                .cloned()
                        }
                        _ => None,
                    }
                };
                if field == RepoField::Encoding {
                    self.repo_state.preferences.encoding = value;
                }
                self.write_repo_state();
            }
            _ => {}
        }
    }

    /// Commits a typed value. `false` means it was refused (a number out of
    /// range, a bad color): the message says why and the caller keeps the
    /// field open with its text.
    pub(super) fn commit_text(&mut self, index: usize, draft: &str) -> bool {
        let fields = self.fields();
        let Some(field) = fields.get(index) else {
            return true;
        };
        let target = field.target.clone();
        match (&field.control, target) {
            (Control::Quantity { .. }, Target::Global(_)) => {
                return self.commit_quantity_number(index, draft);
            }
            (Control::Number { min, max, .. }, Target::Global(key)) => {
                match draft.trim().parse::<i64>() {
                    Ok(value) if value >= *min && value <= *max => {
                        self.write_global(key, &value.to_string());
                    }
                    Ok(value) => {
                        self.message = Some(screen::value_outside(key, value, min, max));
                        return false;
                    }
                    Err(_) => {
                        self.message = Some(screen::not_whole_number(key, format!("{draft:?}")));
                        return false;
                    }
                }
            }
            (_, Target::Global("ui.input_color_custom")) => {
                if crate::parse_hex_color(draft).is_some() {
                    return self.write_global("ui.input_color_custom", draft.trim());
                } else {
                    self.message = Some(screen::not_hex_color(
                        "ui.input_color_custom",
                        format!("{:?}", draft.trim()),
                    ));
                    return false;
                }
            }
            (_, Target::Global(key)) => return self.write_global(key, draft.trim()),
            (_, Target::Ai(t)) => self.ai_commit_text(t, draft),
            (_, Target::OpenIn(t)) => return self.open_in_commit_text(t, draft),
            (_, Target::Workflow(t)) => return self.wf_commit_text(t, draft),
            (_, Target::RepoPage(t)) => return self.rp_commit_text(t, draft),
            (_, Target::RepoAuth(t)) => return self.ra_commit_text(t, draft),
            (_, Target::ImageEditor(t)) => return self.images_commit_text(t, draft),
            (_, Target::ProfileField(i, slot)) => {
                let value = draft.trim().to_string();
                if let Some(profile) = self.config.profiles.get_mut(i) {
                    match slot {
                        TextSlot::Id if value.is_empty() => {}
                        TextSlot::Id => profile.id = value,
                        TextSlot::Name => profile.name = value,
                        TextSlot::AuthorName => profile.author_name = value,
                        TextSlot::AuthorEmail => profile.author_email = value,
                    }
                }
                self.write_profiles();
            }
            _ => {}
        }
        true
    }
}
