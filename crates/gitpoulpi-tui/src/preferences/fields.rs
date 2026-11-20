//! The Preferences pages' field builders. Every plain `config.toml` row is
//! built once in the pools ([`PreferencesState::row_pool`]: `general_rows`,
//! `ui_rows`, `editor_rows`) and the page tables in `pages.rs` pick them by
//! key, so where a setting lives is one table, not scattered builder code.

use super::*;
use gitpoulpi_i18n::msg;

impl PreferencesState {
    /// One "Start tab" dropdown. `radar` is only offered while the
    /// Radar is built and enabled; a stored `radar` then reads as
    /// Home, which is what the start falls back to (`startup_tab::decide`).
    fn startup_tab_field(
        &self,
        label: &'static str,
        key: &'static str,
        choices: &[gitpoulpi_config::StartupTab],
        current: gitpoulpi_config::StartupTab,
    ) -> Field {
        use gitpoulpi_config::StartupTab;
        let radar = crate::build_features::offered(Some(crate::build_features::Feature::Radar));
        let options: Vec<String> = choices
            .iter()
            .filter(|t| radar || **t != StartupTab::Radar)
            .map(|t| t.as_str().to_string())
            .collect();
        let shown = if current == StartupTab::Radar && !radar {
            StartupTab::Home
        } else {
            current
        };
        let index = options
            .iter()
            .position(|o| o == shown.as_str())
            .unwrap_or(0);
        Field::global(label, key, Control::Choice { options, index })
    }

    /// The MCP server's switch and port. The port is a virtual key
    /// (`mcp.port`): it edits the port of `mcp.bind` and keeps its host.
    #[cfg(feature = "mcp")]
    pub(super) fn mcp_rows(&self) -> Vec<Field> {
        let mcp = &self.mcp_in_force();
        let (_, port) = mcp.bind_parts();
        let mut port_row = Field::global(
            msg::settings_general::MCP_PORT.get(),
            "mcp.port",
            Control::Number {
                value: i64::from(port.unwrap_or(gitpoulpi_config::MCP_DEFAULT_PORT)),
                min: 1,
                max: 65_535,
            },
        )
        .with_help(msg::settings_general::MCP_PORT_HELP.get())
        .live();
        if port.is_none() {
            port_row =
                port_row.with_warning(&msg::settings_general::mcp_bind_unreadable(mcp.bind.trim()));
        }
        vec![
            Field::global(
                msg::settings_general::MCP_SERVER.get(),
                "mcp.enabled",
                Control::Toggle(mcp.enabled),
            )
            .with_help(msg::settings_general::MCP_SERVER_HELP.get()),
            port_row,
            self.mcp_suggest_row(),
        ]
    }

    /// Every `general.*` / `repos.*` / `mcp.*` / launch row, unplaced.
    pub(super) fn general_rows(&self) -> Vec<Field> {
        let g = &self.config.general;
        vec![
            self.auto_fetch_field(),
            Field::global(
                msg::settings_general::FETCH_ON_TAB_OPEN.get(),
                "general.fetch_on_tab_open",
                Control::Toggle(g.fetch_on_tab_open),
            )
            .with_help(msg::settings_general::FETCH_ON_TAB_OPEN_HELP.get()),
            Field::global(
                msg::settings_general::AUTO_PRUNE.get(),
                "general.auto_prune",
                Control::Toggle(g.auto_prune),
            )
            .with_help(msg::settings_general::AUTO_PRUNE_HELP.get()),
            Field::global(
                msg::settings_general::AUTO_PRUNE_TAGS.get(),
                "general.auto_prune_tags",
                Control::Toggle(g.auto_prune_tags),
            )
            .with_help(msg::settings_general::AUTO_PRUNE_TAGS_HELP.get()),
            Field::global(
                msg::settings_general::KEEP_SUBMODULES_UP_TO_DATE.get(),
                "general.keep_submodules_up_to_date",
                Control::Toggle(g.keep_submodules_up_to_date),
            ),
            Field::global(
                msg::settings_general::DEFAULT_BRANCH_NAME.get(),
                "general.default_branch_name",
                Control::Text(g.default_branch_name.clone()),
            )
            .with_help(msg::settings_general::DEFAULT_BRANCH_NAME_HELP.get()),
            Field::global(
                msg::conflict_prevention::GLOBAL_ENABLE.get(),
                "general.conflict_detection",
                Control::Toggle(g.conflict_detection),
            )
            .with_help(msg::conflict_prevention::GLOBAL_ENABLE_HELP.get()),
            Field::global(
                msg::settings_general::DELETE_ORIG_FILES_AFTER_MERGE.get(),
                "general.delete_orig_files_after_merge",
                Control::Toggle(g.delete_orig_files_after_merge),
            )
            .with_help(msg::settings_general::DELETE_ORIG_FILES_AFTER_MERGE_HELP.get()),
            Field::global(
                msg::settings_general::INITIAL_COMMITS_IN_GRAPH.get(),
                "general.initial_commits_in_graph",
                Control::Number {
                    value: i64::from(g.initial_commits_in_graph),
                    min: i64::from(MIN_INITIAL_COMMITS_IN_GRAPH),
                    max: 1_000_000,
                },
            )
            .with_help(msg::settings_general::INITIAL_COMMITS_IN_GRAPH_HELP.get())
            .with_warning(msg::settings_general::INITIAL_COMMITS_IN_GRAPH_WARNING.get()),
            Field::global(
                msg::settings_general::LAZY_LOAD_COMMITS_IN_GRAPH.get(),
                "general.lazy_load_commits_in_graph",
                Control::Toggle(g.lazy_load_commits_in_graph),
            )
            .with_help(msg::settings_general::LAZY_LOAD_COMMITS_IN_GRAPH_HELP.get()),
            Field::global(
                msg::settings_general::COMMITS_PER_LAZY_PAGE.get(),
                "general.commits_per_lazy_page",
                Control::Number {
                    value: i64::from(g.commits_per_lazy_page),
                    min: i64::from(MIN_COMMITS_PER_LAZY_PAGE),
                    max: 100_000,
                },
            )
            .with_help(msg::settings_general::COMMITS_PER_LAZY_PAGE_HELP.get()),
            Field::global(
                msg::settings_general::MAX_COMMITS_IN_GRAPH.get(),
                "general.max_commits_in_graph",
                Control::Number {
                    value: i64::from(g.max_commits_in_graph),
                    min: 0,
                    max: 10_000_000,
                },
            )
            .with_help(msg::settings_general::MAX_COMMITS_IN_GRAPH_HELP.get())
            .with_warning(msg::settings_general::MAX_COMMITS_IN_GRAPH_WARNING.get()),
            Field::global(
                msg::settings_general::UNDO_ENABLED.get(),
                "general.undo_enabled",
                Control::Toggle(g.undo_enabled),
            )
            .with_help(msg::settings_general::UNDO_ENABLED_HELP.get()),
            {
                let depth = Field::global(
                    msg::settings_general::UNDO_DEPTH.get(),
                    "general.undo_depth",
                    Control::Number {
                        value: i64::from(g.undo_depth),
                        min: 0,
                        max: 1000,
                    },
                )
                .with_help(msg::settings_general::UNDO_DEPTH_HELP.get());
                if g.undo_enabled {
                    depth
                } else {
                    depth.coming(msg::settings_general::UNDO_OFF_NOTE.get())
                }
            },
            Field::global(
                msg::settings_general::JOURNAL_OUTSIDE.get(),
                "general.journal_outside",
                Control::Toggle(g.journal_outside),
            )
            .with_help(msg::settings_general::JOURNAL_OUTSIDE_HELP.get()),
            Field::global(
                msg::settings_general::LOG_LEVEL.get(),
                "log.level",
                Control::Choice {
                    options: gitpoulpi_config::LogLevel::ALL
                        .iter()
                        .map(|l| l.as_str().into())
                        .collect(),
                    index: gitpoulpi_config::LogLevel::ALL
                        .iter()
                        .position(|l| *l == self.config.log.level)
                        .unwrap_or(0),
                },
            ),
            self.retention_field(),
            self.log_max_file_field(),
            Field::global(
                msg::settings_general::LOG_TIMINGS.get(),
                "log.timings",
                Control::Toggle(self.config.log.timings),
            )
            .with_help(msg::settings_general::LOG_TIMINGS_HELP.get()),
            Field::global(
                msg::settings_general::LOG_EXTENDED.get(),
                "log.extended",
                Control::Toggle(self.config.log.extended),
            )
            .with_help(msg::settings_general::LOG_EXTENDED_HELP.get()),
            Field::global(
                msg::settings_general::COMMIT_COUNT.get(),
                "log.commit_count",
                Control::Choice {
                    options: gitpoulpi_config::CommitCountMethod::ALL
                        .iter()
                        .map(|m| m.as_str().into())
                        .collect(),
                    index: gitpoulpi_config::CommitCountMethod::ALL
                        .iter()
                        .position(|m| *m == self.config.log.commit_count)
                        .unwrap_or(0),
                },
            )
            .with_help(msg::settings_general::COMMIT_COUNT_HELP.get()),
            Field::global(
                msg::settings_general::USE_MAILMAP.get(),
                "log.use_mailmap",
                Control::Toggle(self.config.log.use_mailmap),
            )
            .with_help(msg::settings_general::USE_MAILMAP_HELP.get()),
            Field::global(
                msg::settings_general::REMEMBER_TABS.get(),
                "general.remember_tabs",
                Control::Toggle(g.remember_tabs),
            )
            .with_help(msg::settings_general::REMEMBER_TABS_HELP.get()),
            Field::global(
                msg::settings_general::RADAR_ENABLED.get(),
                "general.radar_enabled",
                Control::Toggle(g.radar_enabled),
            )
            .with_help(msg::settings_general::RADAR_ENABLED_HELP.get()),
            self.startup_tab_field(
                msg::settings_general::STARTUP_TAB_IN_REPO.get(),
                "general.startup_tab_in_repo",
                gitpoulpi_config::StartupTab::ALL,
                g.startup_tab_in_repo,
            )
            .with_help(msg::settings_general::STARTUP_TAB_IN_REPO_HELP.get()),
            self.startup_tab_field(
                msg::settings_general::STARTUP_TAB_OUTSIDE_REPO.get(),
                "general.startup_tab_outside_repo",
                gitpoulpi_config::StartupTab::WITHOUT_REPO,
                g.startup_tab_outside_repo,
            )
            .with_help(msg::settings_general::STARTUP_TAB_OUTSIDE_REPO_HELP.get()),
            self.startup_tab_field(
                msg::settings_general::STARTUP_TAB_LAUNCHER.get(),
                "general.startup_tab_launcher",
                gitpoulpi_config::StartupTab::WITHOUT_REPO,
                g.startup_tab_launcher,
            )
            .with_help(msg::settings_general::STARTUP_TAB_LAUNCHER_HELP.get()),
            Field::global(
                msg::settings_general::GROUP_WORKTREE_TABS.get(),
                "general.group_worktree_tabs",
                Control::Toggle(g.group_worktree_tabs),
            )
            .with_help(msg::settings_general::GROUP_WORKTREE_TABS_HELP.get()),
            Field::global(
                msg::settings_general::ALLOW_AGENT_VIEW_CONTROL.get(),
                "ui.allow_agent_view_control",
                Control::Toggle(self.config.ui.allow_agent_view_control),
            )
            .with_help(msg::settings_general::ALLOW_AGENT_VIEW_CONTROL_HELP.get()),
            Field::global(
                msg::settings_general::ALLOW_UI_PROPOSE.get(),
                "mcp.allow_ui_propose",
                Control::Toggle(self.config.mcp.allow_ui_propose),
            )
            .with_help(msg::settings_general::ALLOW_UI_PROPOSE_HELP.get()),
            Field::global(
                msg::settings_general::CLONE_ROOT.get(),
                "repos.clone_root",
                Control::Text(self.config.repos.clone_root.clone()),
            )
            .with_help(msg::settings_general::CLONE_ROOT_HELP.get()),
            self.clone_start_source_field(),
            Field::global(
                msg::settings_general::BROWSE_ROOT.get(),
                "repos.browse_root",
                Control::Text(self.config.repos.browse_root.clone()),
            )
            .with_help(msg::settings_general::BROWSE_ROOT_HELP.get()),
            Field::global(
                msg::settings_general::REPO_DATA_LOCATION.get(),
                "session.per_repo_state_dir",
                Control::Combo {
                    options: vec![
                        (
                            msg::settings_general::REPO_DATA_IN_GIT_DIR.get().into(),
                            "in_git_dir".into(),
                        ),
                        (
                            msg::settings_general::REPO_DATA_IN_WORKTREE.get().into(),
                            "in_worktree".into(),
                        ),
                        (
                            msg::settings_general::REPO_DATA_USER_DATA.get().into(),
                            "user_data".into(),
                        ),
                    ],
                    value: gitpoulpi_config::RepoDataLocation::parse(
                        &self.config.session.per_repo_state_dir,
                    )
                    .as_str()
                    .into(),
                    custom: false,
                },
            )
            .with_help(msg::settings_general::REPO_DATA_LOCATION_HELP.get()),
            Field::global(
                msg::settings_general::AUTO_STASH.get(),
                "general.auto_stash",
                Control::Choice {
                    options: vec!["ask".into(), "always".into(), "never".into()],
                    index: match g.auto_stash {
                        gitpoulpi_config::AutoStashMode::Ask => 0,
                        gitpoulpi_config::AutoStashMode::Always => 1,
                        gitpoulpi_config::AutoStashMode::Never => 2,
                    },
                },
            )
            .with_help(msg::settings_general::AUTO_STASH_HELP.get()),
            Field::global(
                msg::settings_general::INCLUDE_UNTRACKED_IN_AUTOSTASH.get(),
                "general.include_untracked_in_autostash",
                Control::Toggle(g.include_untracked_in_autostash),
            )
            .with_help(msg::settings_general::INCLUDE_UNTRACKED_IN_AUTOSTASH_HELP.get()),
            self.updates_mode_field(),
            self.updates_channel_field(),
            self.updates_installed_with_field(),
            self.hosting_refresh_field(),
            Field::global(
                msg::settings_general::HOSTING_POST_AS.get(),
                "general.hosting_post_as",
                Control::Text(g.hosting_post_as.clone()),
            )
            .with_help(msg::settings_general::HOSTING_POST_AS_HELP.get()),
            self.script_timeout_field(),
            self.script_total_timeout_field(),
            Field::global(
                msg::settings_general::PR_TERMINOLOGY.get(),
                "ui.pr_terminology",
                Control::Choice {
                    options: gitpoulpi_config::PrTerminology::ALL
                        .iter()
                        .map(|t| t.as_str().into())
                        .collect(),
                    index: gitpoulpi_config::PrTerminology::ALL
                        .iter()
                        .position(|t| *t == self.config.ui.pr_terminology)
                        .unwrap_or(0),
                },
            )
            .with_help(msg::settings_general::PR_TERMINOLOGY_HELP.get()),
            Field::global(
                msg::settings_general::HIDE_FINISHED_PR_CHIP.get(),
                "ui.hide_finished_pr_chip",
                Control::Toggle(self.config.ui.hide_finished_pr_chip),
            )
            .with_help(msg::settings_general::HIDE_FINISHED_PR_CHIP_HELP.get()),
            Field::global(
                msg::settings_general::EXTERNAL_EDITOR.get(),
                "general.external_editor",
                Control::Text(g.external_editor.clone()),
            )
            .with_placeholder(msg::settings_general::EXTERNAL_EDITOR_PLACEHOLDER.get())
            .with_help(msg::settings_general::EXTERNAL_EDITOR_HELP.get()),
            Field::global(
                msg::settings_general::EXTERNAL_DIFF_TOOL.get(),
                "general.external_diff_tool",
                Control::Text(g.external_diff_tool.clone()),
            )
            .with_placeholder(msg::settings_general::EXTERNAL_DIFF_TOOL_PLACEHOLDER.get())
            .with_help(msg::settings_general::EXTERNAL_DIFF_TOOL_HELP.get()),
            Field::global(
                msg::settings_general::EXTERNAL_MERGE_TOOL.get(),
                "general.external_merge_tool",
                Control::Text(g.external_merge_tool.clone()),
            )
            .with_placeholder(msg::settings_general::EXTERNAL_MERGE_TOOL_PLACEHOLDER.get())
            .with_help(msg::settings_general::EXTERNAL_MERGE_TOOL_HELP.get()),
            Field::global(
                msg::settings_general::FILE_EXPLORER.get(),
                "general.file_explorer",
                Control::Text(g.file_explorer.clone()),
            )
            .with_help(msg::settings_general::FILE_EXPLORER_HELP.get()),
            {
                // The presets this OS offers, plus whatever the config holds
                // (a file shared between machines keeps its value visible).
                let mut options = gitpoulpi_config::launch::choices_for_this_os();
                let current = self.config.ui.launch.clone();
                let index = match options.iter().position(|o| *o == current) {
                    Some(i) => i,
                    None => {
                        options.insert(1, current);
                        1
                    }
                };
                Field::global(
                    msg::settings_general::LAUNCH.get(),
                    "ui.launch",
                    Control::Choice { options, index },
                )
                .with_help(msg::settings_general::LAUNCH_HELP.get())
            },
            Field::global(
                msg::settings_general::LAUNCH_COMMAND.get(),
                "ui.launch_command",
                Control::Text(self.config.ui.launch_command.clone()),
            )
            .with_help(msg::settings_general::LAUNCH_COMMAND_HELP.get()),
            Field::global(
                msg::settings_general::TAB_DIR_FOLLOW.get(),
                "ui.tab_dir_follow",
                Control::Choice {
                    options: TabDirFollow::ALL
                        .iter()
                        .map(|f| f.as_str().into())
                        .collect(),
                    index: TabDirFollow::ALL
                        .iter()
                        .position(|f| *f == self.config.ui.tab_dir_follow)
                        .unwrap_or(0),
                },
            )
            .with_help(msg::settings_general::TAB_DIR_FOLLOW_HELP.get()),
        ]
    }

    /// General > "Clone dialog starts on": Last used, Clone with URL, or a
    /// profile the dialog could list (its own filter).
    pub(super) fn clone_start_source_field(&self) -> Field {
        let mut options: Vec<(String, String)> = vec![
            (
                msg::settings_general::CLONE_START_LAST.get().into(),
                gitpoulpi_config::CLONE_START_LAST.into(),
            ),
            (
                msg::clone::SOURCE_URL.get().into(),
                gitpoulpi_config::CLONE_START_URL.into(),
            ),
        ];
        options.extend(
            crate::clone_dialog::hosted_sources(&self.config.integrations)
                .into_iter()
                .map(|h| (h.label, h.profile_id)),
        );
        let stored = self.config.repos.clone_start_source.trim();
        let value = if stored.is_empty() || stored.eq_ignore_ascii_case("last") {
            gitpoulpi_config::CLONE_START_LAST.to_string()
        } else if stored.eq_ignore_ascii_case("url") {
            gitpoulpi_config::CLONE_START_URL.to_string()
        } else {
            stored.to_string()
        };
        Field::global(
            msg::settings_general::CLONE_START_SOURCE.get(),
            "repos.clone_start_source",
            Control::Combo {
                options,
                value,
                custom: false,
            },
        )
        .with_help(msg::settings_general::CLONE_START_SOURCE_HELP.get())
    }

    /// Profiles > General: the one setting that is not a profile's own.
    pub(super) fn profile_general_fields(&self) -> Vec<Field> {
        let mut options: Vec<String> =
            vec![msg::settings_profiles::NO_PROFILE_OPTION.get().to_string()];
        options.extend(
            self.config
                .profiles
                .iter()
                .map(|p| format!("{} <{}>", p.name, p.author_email)),
        );
        let active = self
            .config
            .profiles
            .iter()
            .position(|p| p.id == self.config.general.active_profile)
            .map(|i| i + 1)
            .unwrap_or(0);

        vec![Field::global(
            msg::settings_profiles::ACTIVE_PROFILE.get(),
            "general.active_profile",
            Control::Choice {
                options,
                index: active,
            },
        )
        .with_help(msg::settings_profiles::ACTIVE_PROFILE_HELP.get())]
    }

    /// Profiles > the list tab: the rows of the selected profile.
    pub(super) fn profile_item_fields(&self) -> Vec<Field> {
        let mut fields = Vec::new();
        // The list (left) adds and removes; the detail edits its selection.
        if self.config.profiles.is_empty() {
            fields.push(Field::heading(
                msg::settings_profiles::HEADING_NO_PROFILES_YET.get(),
            ));
            return fields;
        }

        let index = self.profile_index.min(self.config.profiles.len() - 1);
        let profile = &self.config.profiles[index];
        fields.push(Field::heading(&msg::settings_profiles::editing_profile(
            index + 1,
            self.config.profiles.len(),
        )));
        fields.push(
            Field::plain(
                msg::settings_profiles::PROFILE_ID.get(),
                Control::Text(profile.id.clone()),
                Target::ProfileField(index, TextSlot::Id),
            )
            .with_help(msg::settings_profiles::PROFILE_ID_HELP.get()),
        );
        fields.push(Field::plain(
            msg::settings_profiles::PROFILE_NAME.get(),
            Control::Text(profile.name.clone()),
            Target::ProfileField(index, TextSlot::Name),
        ));
        fields.push(Field::plain(
            msg::settings_profiles::AUTHOR_NAME.get(),
            Control::Text(profile.author_name.clone()),
            Target::ProfileField(index, TextSlot::AuthorName),
        ));
        fields.push(Field::plain(
            msg::settings_profiles::AUTHOR_EMAIL.get(),
            Control::Text(profile.author_email.clone()),
            Target::ProfileField(index, TextSlot::AuthorEmail),
        ));
        fields
    }

    pub(super) fn ssh_fields(&self) -> Vec<Field> {
        let ssh = &self.config.ssh;
        vec![
            Field::global(
                msg::settings_ssh::PRIVATE_KEY_PATH.get(),
                "ssh.private_key_path",
                Control::Text(ssh.private_key_path.clone().unwrap_or_default()),
            )
            .with_help(msg::settings_ssh::PRIVATE_KEY_PATH_HELP.get()),
            Field::global(
                msg::settings_ssh::PUBLIC_KEY_PATH.get(),
                "ssh.public_key_path",
                Control::Text(ssh.public_key_path.clone().unwrap_or_default()),
            )
            .with_help(msg::settings_ssh::PUBLIC_KEY_PATH_HELP.get()),
            Field::global(
                msg::settings_ssh::KEY_TYPE.get(),
                "ssh.key_type",
                Control::Choice {
                    options: vec!["ed25519".into(), "rsa".into(), "ecdsa".into()],
                    index: match ssh.key_type.as_str() {
                        "rsa" => 1,
                        "ecdsa" => 2,
                        _ => 0,
                    },
                },
            ),
            Field::heading(msg::settings_ssh::HEADING_HTTPS_CREDENTIALS.get()),
            Field::global(
                msg::settings_ssh::HTTPS_MODE.get(),
                "credentials.https_mode",
                Control::Choice {
                    options: gitpoulpi_config::HttpsMode::ALL
                        .iter()
                        .map(|m| m.as_str().into())
                        .collect(),
                    index: gitpoulpi_config::HttpsMode::ALL
                        .iter()
                        .position(|m| *m == self.config.credentials.https_mode)
                        .unwrap_or(0),
                },
            )
            .with_help(msg::settings_ssh::HTTPS_MODE_HELP.get()),
            Field::button(
                msg::settings_general::FORGET_ALL_USERNAMES_AND_PASSWORDS.get(),
                PrefAction::ForgetCredentials,
            )
            .with_help(msg::settings_general::FORGET_ALL_USERNAMES_AND_PASSWORDS_HELP.get()),
            Field::heading(msg::settings_ssh::HEADING_KEYS.get()),
            Field::button(
                msg::settings_ssh::GENERATE_KEY_PAIR.get(),
                PrefAction::GenerateSshKey,
            )
            .with_help(msg::settings_ssh::GENERATE_KEY_PAIR_HELP.get()),
        ]
    }

    /// The Diff & editor page's editor rows: only what makes sense in a terminal, each row read
    /// by the views it names (`editor_prefs`).
    pub(super) fn editor_rows(&self) -> Vec<Field> {
        let ed = &self.config.editor;
        let eol_options: Vec<String> = gitpoulpi_config::EditorEol::ALL
            .iter()
            .map(|e| e.as_str().to_string())
            .collect();
        vec![
            Field::global(
                msg::settings_editor::TAB_SIZE.get(),
                "editor.tab_size",
                Control::Choice {
                    options: (1..=8).map(|n| n.to_string()).collect(),
                    index: usize::from(ed.tab_size_clamped()) - 1,
                },
            )
            .with_help(msg::settings_editor::TAB_SIZE_HELP.get()),
            Field::global(
                msg::settings_editor::EOL.get(),
                "editor.eol",
                Control::Choice {
                    options: eol_options,
                    index: gitpoulpi_config::EditorEol::ALL
                        .iter()
                        .position(|e| *e == ed.eol)
                        .unwrap_or(0),
                },
            )
            .with_help(msg::settings_editor::EOL_HELP.get()),
            Field::global(
                msg::settings_editor::SYNTAX_HIGHLIGHTING.get(),
                "editor.syntax_highlighting",
                Control::Toggle(ed.syntax_highlighting),
            )
            .with_help(msg::settings_editor::SYNTAX_HIGHLIGHTING_HELP.get()),
            Field::global(
                msg::settings_editor::LINE_NUMBERS.get(),
                "editor.line_numbers",
                Control::Toggle(ed.line_numbers),
            )
            .with_help(msg::settings_editor::LINE_NUMBERS_HELP.get()),
            Field::global(
                msg::settings_editor::FOLLOW_SELECTION.get(),
                "editor.follow_selection",
                Control::Toggle(ed.follow_selection),
            )
            .with_help(msg::settings_editor::FOLLOW_SELECTION_HELP.get()),
            Field::global(
                msg::settings_editor::DIFF_WORD_WRAP.get(),
                "ui.diff_word_wrap",
                Control::Toggle(self.config.ui.diff_word_wrap),
            )
            .with_help(msg::settings_editor::DIFF_WORD_WRAP_HELP.get()),
            // The merge editor's settings come last, so the editor and
            // diff rows above stay together.
            Field::global(
                msg::settings_editor::MERGE_SIDE_BY_SIDE_MIN_WIDTH.get(),
                "merge.side_by_side_min_width",
                Control::Number {
                    value: i64::from(self.config.merge.side_by_side_min_width),
                    min: 0,
                    max: i64::from(gitpoulpi_config::MAX_MERGE_SIDE_BY_SIDE_MIN_WIDTH),
                },
            )
            .with_help(msg::settings_editor::MERGE_SIDE_BY_SIDE_MIN_WIDTH_HELP.get()),
        ]
    }
}

impl PreferencesState {
    /// "Keep log files for": a number and a unit on one line. The config
    /// stores days (`log.retention_days`) and the unit it is shown in
    /// (`log.retention_unit`).
    fn retention_field(&self) -> Field {
        use gitpoulpi_config::RetentionUnit;
        let (n, current) = self.config.log.retention_quantity();
        let units: Vec<QuantityUnit> = RetentionUnit::CHOICES
            .iter()
            .map(|u| QuantityUnit {
                label: retention_unit_word(*u, u64::from(n)),
                key: u.as_str(),
                weight: i64::from(u.days()),
            })
            .collect();
        let unit = RetentionUnit::CHOICES
            .iter()
            .position(|u| *u == current)
            .unwrap_or(0);
        Field::global(
            msg::settings_general::LOG_RETENTION_DAYS.get(),
            "log.retention_days",
            Control::Quantity {
                value: i64::from(n),
                unit,
                units,
                max_total: i64::from(gitpoulpi_config::MAX_LOG_RETENTION_DAYS),
                allow_zero: false,
                unit_key: "log.retention_unit",
            },
        )
        .with_help(msg::settings_general::LOG_RETENTION_HELP.get())
    }
}

/// The unit's name for `n` ("week", "weeks"): the catalog's plural text with
/// the number cut out, so the dropdown shows only the word.
fn retention_unit_word(unit: gitpoulpi_config::RetentionUnit, n: u64) -> String {
    use gitpoulpi_config::RetentionUnit as U;
    let text = match unit {
        U::Weeks => msg::settings_general::retention_unit_weeks(n),
        U::Months => msg::settings_general::retention_unit_months(n),
        U::Years => msg::settings_general::retention_unit_years(n),
        U::Days | U::Auto => msg::settings_general::retention_unit_days(n),
    };
    let word = text.replacen(&n.to_string(), "", 1);
    let word = word.trim();
    if word.is_empty() {
        text
    } else {
        word.to_string()
    }
}
