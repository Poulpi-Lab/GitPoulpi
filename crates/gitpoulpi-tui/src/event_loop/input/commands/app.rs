//! Repository tabs, undo/redo, view toggles, settings, AI and the app itself.

use crate::event_loop::*;

use super::{shared::*, CommandCtx};

pub(super) fn run(cx: &mut CommandCtx<'_>, cmd: PaletteCommand) -> bool {
    match cmd {
        // -- Repository ------------------------
        PaletteCommand::OpenRepository => {
            go_to_management(ManagementEntry::Show, &mut tab_ctx!(cx))
        }
        PaletteCommand::BrowseRepository => {
            go_to_management(ManagementEntry::Browse, &mut tab_ctx!(cx))
        }
        PaletteCommand::CreateRepository => {
            go_to_management(ManagementEntry::Init, &mut tab_ctx!(cx))
        }
        PaletteCommand::CloneRepository => {
            go_to_management(ManagementEntry::Clone, &mut tab_ctx!(cx))
        }
        PaletteCommand::Management(command) => {
            *cx.last_message = run_management_command(command, &mut tab_ctx!(cx));
            if let (Some(m), Some(message)) = (cx.management.as_mut(), cx.last_message.as_ref()) {
                m.message = Some(message.clone());
            }
            *cx.should_quit |= cx.switch_to.is_some() || *cx.close_window;
        }
        PaletteCommand::Workspace(command) => {
            *cx.last_message = crate::workspace_ui::apply::run_command(command, &mut tab_ctx!(cx));
            if let (Some(m), Some(message)) = (cx.management.as_mut(), cx.last_message.as_ref()) {
                m.message = Some(message.clone());
            }
            *cx.should_quit |=
                cx.switch_to.is_some() || *cx.close_window || cx.show_management.is_some();
        }
        PaletteCommand::Radar(crate::radar::RadarCommand::Open) => {
            go_to_management(ManagementEntry::Radar, &mut tab_ctx!(cx));
            *cx.should_quit |= cx.show_management.is_some();
        }
        PaletteCommand::Radar(crate::radar::RadarCommand::CloseTab) => {
            close_radar_tab(&mut tab_ctx!(cx));
        }
        PaletteCommand::Radar(command) => {
            let outcome = cx
                .management
                .as_mut()
                .map_or(crate::radar::Outcome::None, |m| m.radar.command(command));
            apply_radar_outcome(cx, outcome);
        }
        PaletteCommand::Updates(crate::updates::UpdatesCommand::ReleaseNotes) => {
            go_to_management(ManagementEntry::ReleaseNotes, &mut tab_ctx!(cx));
            *cx.should_quit |= cx.show_management.is_some();
        }
        PaletteCommand::Updates(crate::updates::UpdatesCommand::CloseTab) => {
            close_notes_tab(&mut tab_ctx!(cx));
        }
        PaletteCommand::Updates(command) => {
            if let Some(message) = crate::updates::run(command) {
                *cx.last_message = Some(message);
            }
        }
        PaletteCommand::NewTab => {
            go_to_management(ManagementEntry::NewTab, &mut tab_ctx!(cx));
            *cx.should_quit |= cx.show_management.is_some();
        }
        PaletteCommand::OpenNewMenu => {
            // Under the `+` when the bar drew it, else the bar's left end.
            let anchor = cx
                .hit_targets
                .iter()
                .find(|(_, target)| matches!(target, HitTarget::NewTabButton))
                .map(|(rect, _)| (rect.x, rect.y + 1))
                .unwrap_or((0, 1));
            open_new_menu(anchor, &mut tab_ctx!(cx));
        }
        PaletteCommand::OpenTabMenu => {
            let anchor = cx
                .tab_hits
                .iter()
                .find(|h| h.index == *cx.active_tab)
                .map(|h| (h.tab.x, h.tab.y + 1))
                .unwrap_or((0, 1));
            if let Some(message) = open_current_tab_menu(anchor, &mut tab_ctx!(cx)) {
                *cx.last_message = Some(message);
            }
        }
        PaletteCommand::TabMenu(action) => {
            *cx.last_message = run_tab_menu_command(action, &mut tab_ctx!(cx));
            *cx.should_quit |= cx.switch_to.is_some() || *cx.close_window;
        }
        PaletteCommand::TabOpenIn => {
            // `Tab: Open in...`: the tab bar's popup, focused.
            crate::open_in_bar::open();
        }
        PaletteCommand::TabOpenAgentHere => {
            let active = cx.repo_tabs.get(*cx.active_tab).map(|t| t.path.clone());
            match active {
                None => *cx.last_message = Some(gitpoulpi_i18n::msg::tab_dir::NO_TAB.get().into()),
                Some(path) => match crate::agent_launch::palette_step(&path) {
                    crate::agent_launch::PaletteStep::Done(done) => {
                        *cx.last_message = Some(done.unwrap_or_else(|e| e));
                    }
                    // Several agents: the same popup as "Open in", one row each.
                    crate::agent_launch::PaletteStep::Choose(agents) => {
                        crate::open_in_bar::open_agents(&agents);
                    }
                },
            }
        }
        PaletteCommand::AboutGitPoulpi => crate::about::open(),
        PaletteCommand::AboutLicenses => crate::about::open_licenses(),
        PaletteCommand::TrustReview => {
            if !crate::trust_dialog::open_review() {
                *cx.last_message =
                    Some(gitpoulpi_i18n::msg::trust_dialog::NONE_WAITING.get().into());
            }
        }
        PaletteCommand::OpenActivityLog => crate::activity_log::open_panel(),
        PaletteCommand::OpenGlobalLog => crate::activity_log::open_global_panel(),
        PaletteCommand::Jobs(cmd) => crate::jobs_ui::run(cmd),
        PaletteCommand::CopyDiagnostics => {
            *cx.last_message = Some(crate::activity_log::copy_diagnostics());
        }
        PaletteCommand::ActivityLevel(cmd) => crate::activity_log::level_command(cmd),
        PaletteCommand::OpenLastError => crate::activity_log::status_error::open_entry(),
        PaletteCommand::CopyLastError => {
            if let Some(command) = crate::activity_log::copy_last_error() {
                super::super::apply_card_command(cx, command);
            }
        }
        PaletteCommand::RunLastErrorRemedy => {
            if let Some(command) = crate::activity_log::last_error_command() {
                super::super::apply_card_command(cx, command);
            }
        }
        PaletteCommand::ChangeProfile => {
            if crate::profile_bar::visible() {
                crate::profile_bar::open();
            } else {
                *cx.last_message = Some(gitpoulpi_i18n::msg::profile_bar::NOT_ENOUGH.get().into());
            }
        }
        PaletteCommand::TabOpenInDefault => {
            let active = cx.repo_tabs.get(*cx.active_tab).map(|t| t.path.clone());
            *cx.last_message = Some(match active {
                Some(path) => crate::open_in_bar::run_default(&path),
                None => gitpoulpi_i18n::msg::tab_dir::NO_TAB.get().into(),
            });
        }
        PaletteCommand::TabDir(button) => {
            let active = cx.repo_tabs.get(*cx.active_tab).map(|t| t.path.clone());
            match crate::tab_dir::run(button, active.as_deref()) {
                crate::tab_dir::Outcome::Message(message) => *cx.last_message = Some(message),
                crate::tab_dir::Outcome::QuitNow => {
                    *cx.should_quit = true;
                    *cx.quit_how = Some(crate::tab_dir::QUIT_CD_HOW);
                }
            }
        }
        PaletteCommand::CycleRepositoryTab(forward) => {
            if let Some(message) = cycle_repository_tab(
                forward,
                cx.session,
                cx.link,
                cx.repo,
                &mut *cx.repo_dialogs,
                &mut *cx.repo_tabs,
                &mut *cx.active_tab,
                &mut *cx.switch_to,
                &mut *cx.close_window,
            ) {
                *cx.last_message = Some(message);
            }
            *cx.should_quit |= cx.switch_to.is_some() || *cx.close_window;
        }
        PaletteCommand::MoveRepositoryTab(forward) => {
            let from = *cx.active_tab;
            *cx.last_message = match tab_move_neighbour(from, cx.repo_tabs.len(), forward) {
                Some(to) => reorder_tab(
                    cx.session,
                    &mut *cx.repo_tabs,
                    &mut *cx.active_tab,
                    from,
                    to,
                ),
                None => Some(
                    if forward {
                        gitpoulpi_i18n::msg::palette::WHY_TAB_ALREADY_LAST
                    } else {
                        gitpoulpi_i18n::msg::palette::WHY_TAB_ALREADY_FIRST
                    }
                    .get()
                    .to_string(),
                ),
            };
        }
        PaletteCommand::CloseRepositoryTab
            if cx
                .management
                .as_ref()
                .is_some_and(|m| m.page == crate::repo_manager::HomePage::Workspaces) =>
        {
            close_workspaces_tab(&mut tab_ctx!(cx));
        }
        PaletteCommand::CloseRepositoryTab
            if cx.management.as_ref().is_some_and(|m| m.radar.is_open()) =>
        {
            // On the Radar tab, "close tab" closes it (no question: it
            // holds nothing, and reopening is one command away).
            close_radar_tab(&mut tab_ctx!(cx));
        }
        PaletteCommand::CloseRepositoryTab => {
            // Confirmed, like every other
            // way to close a tab -- the
            // palette must not be the one
            // path that skips the question.
            if let Some(prompt) = close_tab_prompt(&*cx.repo_tabs, *cx.active_tab) {
                *cx.input_mode = InputMode::Confirm {
                    prompt,
                    action: ConfirmAction::CloseTab(*cx.active_tab),
                };
            }
        }
        // Not offered without the feature (tagged in `command_requires`).
        #[cfg(not(feature = "global-search"))]
        PaletteCommand::WorkspaceSearch => {}
        #[cfg(feature = "global-search")]
        PaletteCommand::WorkspaceSearch => {
            *cx.input_mode = InputMode::WorkspaceSearch(Box::new(
                crate::workspace_search::WorkspaceSearch::open(
                    cx.session,
                    &*cx.repo_tabs,
                    cx.link.activity_handle().raw.requester(),
                ),
            ));
        }
        PaletteCommand::SelectTab => {
            *cx.input_mode = crate::tabs::open_tab_picker(&*cx.repo_tabs, *cx.active_tab);
        }

        // Same path as the toolbar buttons and
        // Ctrl+Z / Ctrl+Y; the unavailable ones
        // were refused before reaching here.
        PaletteCommand::Undo => {
            *cx.last_message = Some(
                cx.link
                    .start_undo_redo(gitpoulpi_core::domain::Direction::Undo),
            );
        }
        PaletteCommand::UndoForget => {
            let state = cx.link.journal_state();
            match crate::undo_ui::forget_prompt(&state) {
                Some(prompt) => {
                    *cx.input_mode = InputMode::Confirm {
                        prompt,
                        action: ConfirmAction::UndoForget {
                            expect: crate::undo_ui::top_id(
                                &state,
                                gitpoulpi_core::domain::Direction::Undo,
                            ),
                        },
                    };
                }
                None => *cx.last_message = Some(crate::undo_ui::nothing_to_undo().to_string()),
            }
        }
        PaletteCommand::UndoHistory => {
            cx.repo_dialogs
                .open_safety(cx.link, crate::safety_panel::Tab::History, None);
        }
        PaletteCommand::ReflogHead => {
            cx.repo_dialogs.open_safety(
                cx.link,
                crate::safety_panel::Tab::Reflog,
                Some("HEAD".to_string()),
            );
        }
        PaletteCommand::LostCommits => {
            cx.repo_dialogs
                .open_safety(cx.link, crate::safety_panel::Tab::Lost, None);
        }
        PaletteCommand::Safety(crate::safety_panel::SafetyCmd::ArrangeDialog) => {
            *cx.last_message = Some(if crate::dialog_kit::frame::arrange_top() {
                gitpoulpi_i18n::msg::dialog_frame::ARRANGING.get().into()
            } else {
                gitpoulpi_i18n::msg::dialog_frame::WHY_NO_DIALOG
                    .get()
                    .into()
            });
        }
        PaletteCommand::Safety(crate::safety_panel::SafetyCmd::ResetDialog) => {
            *cx.last_message = Some(if crate::dialog_kit::frame::reset_top() {
                gitpoulpi_i18n::msg::dialog_frame::RESET_DONE.get().into()
            } else {
                gitpoulpi_i18n::msg::dialog_frame::WHY_NO_DIALOG
                    .get()
                    .into()
            });
        }
        PaletteCommand::Safety(cmd) => {
            let state = cx.link.journal_state();
            let (effect, why) = cx.repo_dialogs.run_safety(cmd, &state);
            if let Some(why) = why {
                *cx.last_message = Some(why);
            }
            if let Some(effect) = effect {
                super::super::menu_actions::apply_effect(cx, effect);
            }
        }
        PaletteCommand::Board(cmd) => {
            #[cfg(feature = "hosting")]
            if let Some(effect) = cx.repo_dialogs.run_board(cmd) {
                super::super::menu_actions::apply_effect(cx, effect);
            }
            #[cfg(not(feature = "hosting"))]
            let _ = cmd;
        }
        PaletteCommand::Composer(cmd) => {
            let said = if cx
                .management
                .as_ref()
                .is_some_and(|m| m.radar.hosts_review() || m.radar.hosts_item())
            {
                cx.management
                    .as_mut()
                    .and_then(|m| m.radar.run_composer(cmd))
            } else {
                cx.repo_dialogs.run_composer(cmd)
            };
            if let Some(said) = said {
                *cx.last_message = Some(said);
            }
        }
        PaletteCommand::Item(cmd) => {
            // The conversation hosted over the Radar answers before the
            // dialogs do.
            if cx.management.as_ref().is_some_and(|m| m.radar.hosts_item()) {
                let outcome = cx
                    .management
                    .as_mut()
                    .map_or(crate::radar::Outcome::None, |m| m.radar.run_item(cmd));
                apply_radar_outcome(cx, outcome);
            } else {
                #[cfg(feature = "hosting")]
                if let Some(effect) = cx.repo_dialogs.run_item(cmd) {
                    super::super::menu_actions::apply_effect(cx, effect);
                }
                #[cfg(not(feature = "hosting"))]
                let _ = cmd;
            }
        }
        // The window opens the dialog, menu or page at its next notice pass
        // (it owns the seed's data: refs, tags, issues).
        PaletteCommand::Flow(cmd) => crate::flow::request(cmd),
        PaletteCommand::Review(cmd) => {
            // The review hosted over the Radar answers before the dialogs do.
            if cx
                .management
                .as_ref()
                .is_some_and(|m| m.radar.hosts_review())
            {
                let outcome = cx
                    .management
                    .as_mut()
                    .map_or(crate::radar::Outcome::None, |m| m.radar.run_review(cmd));
                apply_radar_outcome(cx, outcome);
            } else {
                #[cfg(feature = "hosting")]
                if let Some(effect) = cx.repo_dialogs.run_review(cmd) {
                    super::super::menu_actions::apply_effect(cx, effect);
                }
                #[cfg(not(feature = "hosting"))]
                let _ = cmd;
            }
        }
        PaletteCommand::UndoSequence => {
            *cx.last_message = Some(cx.link.start_undo_sequence());
        }
        PaletteCommand::UndoOffer => {
            let state = cx.link.journal_state();
            match crate::safety_panel::offer_of(&state) {
                Some((name, oid, _)) => {
                    *cx.input_mode = InputMode::Confirm {
                        prompt: gitpoulpi_i18n::msg::undo_safety::restore_confirm_ref(&name),
                        action: ConfirmAction::ReflogRestore { refname: name, oid },
                    };
                }
                None => {
                    *cx.last_message = Some(
                        gitpoulpi_i18n::msg::undo_safety::WHY_NO_OFFER
                            .get()
                            .to_string(),
                    )
                }
            }
        }
        PaletteCommand::Redo => {
            *cx.last_message = Some(
                cx.link
                    .start_undo_redo(gitpoulpi_core::domain::Direction::Redo),
            );
        }

        // -- View ------------------------------
        PaletteCommand::Refresh if cx.link_failure.is_some() => {
            *cx.retry_link_requested = true;
        }
        PaletteCommand::Refresh => {
            cx.link.request_refresh();
            cx.hosting.demand();
        }
        PaletteCommand::RefreshAll if cx.link_failure.is_some() => {
            *cx.retry_link_requested = true;
        }
        PaletteCommand::RefreshAll => {
            // Everything in the background: the fetch is a daemon action
            // (in-flight feedback after 150 ms), the reload and the
            // hosting re-check are requests answered by their workers.
            cx.link.submit_action(Action::FetchAll);
            cx.link.request_refresh();
            cx.hosting.demand();
            crate::event_loop::fetch_policy::note_fetch(cx.repo_root);
        }
        PaletteCommand::ToggleHelp => *cx.show_help = !*cx.show_help,
        PaletteCommand::Sequence(op) => {
            crate::sequence_run::start(op.slot(), cx.repo_root, cx.last_message);
        }
        PaletteCommand::Setting(op) => match palette::run_setting(op) {
            palette::SettingOutcome::Message(message) => *cx.last_message = Some(message),
            palette::SettingOutcome::EditOnPage(key) => {
                // Settings already open (the palette over it): reuse it.
                let mut state = cx
                    .preferences
                    .take()
                    .unwrap_or_else(|| PreferencesState::open(cx.repo_root));
                if let Err(why) = state.edit_setting(key) {
                    *cx.last_message = Some(why);
                }
                *cx.preferences = Some(state);
            }
            palette::SettingOutcome::OpenPage(key) => {
                let mut state = PreferencesState::open(cx.repo_root);
                if let Err(why) = state.show_setting(key) {
                    *cx.last_message = Some(why);
                }
                *cx.preferences = Some(state);
            }
        },
        PaletteCommand::OpenPreferences => {
            *cx.preferences = Some(PreferencesState::open(cx.repo_root));
        }
        PaletteCommand::RepoAuthOpen => {
            let mut state = PreferencesState::open(cx.repo_root);
            state.open_repo_auth_page();
            *cx.preferences = Some(state);
        }
        PaletteCommand::RepoAuthTest => {
            let mut state = PreferencesState::open(cx.repo_root);
            state.repo_auth_test_command();
            *cx.preferences = Some(state);
        }
        PaletteCommand::RepoAuthDefaults => {
            let mut state = PreferencesState::open(cx.repo_root);
            state.repo_auth_defaults_command();
            *cx.preferences = Some(state);
        }
        PaletteCommand::SearchSettings => {
            let mut state = PreferencesState::open(cx.repo_root);
            state.open_search();
            *cx.preferences = Some(state);
        }
        PaletteCommand::Keybindings => {
            let mut state = PreferencesState::open(cx.repo_root);
            state.show_keybindings(false);
            *cx.preferences = Some(state);
        }
        PaletteCommand::Theme => {
            let mut state = PreferencesState::open(cx.repo_root);
            let _ = state.show_page(None, Some("ui.theme"));
            *cx.preferences = Some(state);
        }
        PaletteCommand::ConflictLists => {
            let mut state = PreferencesState::open(cx.repo_root);
            let _ = state.show_page(
                Some(gitpoulpi_ipc::ui::SettingsPage::ConflictPreventionGlobal),
                None,
            );
            *cx.preferences = Some(state);
        }
        PaletteCommand::ConflictRepoPage => {
            let mut state = PreferencesState::open(cx.repo_root);
            let _ = state.show_page(
                Some(gitpoulpi_ipc::ui::SettingsPage::ConflictPrevention),
                None,
            );
            *cx.preferences = Some(state);
        }
        PaletteCommand::ChangeTheme => {
            *cx.input_mode =
                InputMode::ThemePicker(Box::new(crate::theme_picker::ThemePicker::open()));
        }
        PaletteCommand::ChangeLanguage => {
            *cx.input_mode =
                InputMode::LanguagePicker(Box::new(crate::language_picker::LanguagePicker::open()));
        }
        PaletteCommand::LauncherCreate => {
            let mut state = PreferencesState::open(cx.repo_root);
            state.launcher_command(false, false);
            *cx.preferences = Some(state);
        }
        PaletteCommand::LauncherCreateDesktop => {
            let mut state = PreferencesState::open(cx.repo_root);
            state.launcher_command(true, false);
            *cx.preferences = Some(state);
        }
        PaletteCommand::LauncherRemove => {
            let mut state = PreferencesState::open(cx.repo_root);
            state.launcher_command(false, true);
            *cx.preferences = Some(state);
        }
        PaletteCommand::McpSuggestPort => {
            #[cfg_attr(not(feature = "mcp"), allow(unused_mut))]
            let mut state = PreferencesState::open(cx.repo_root);
            #[cfg(feature = "mcp")]
            {
                state.mcp_suggest_command();
                *cx.last_message = state.take_message();
            }
            *cx.preferences = Some(state);
        }
        PaletteCommand::LauncherAliasCreate
        | PaletteCommand::LauncherAliasRemove
        | PaletteCommand::LauncherPathAdd
        | PaletteCommand::LauncherPathRemove => {
            use crate::preferences::ToolsButton as B;
            let button = match cmd {
                PaletteCommand::LauncherAliasCreate => B::AliasCreate,
                PaletteCommand::LauncherAliasRemove => B::AliasRemove,
                PaletteCommand::LauncherPathAdd => B::PathAdd,
                _ => B::PathRemove,
            };
            let mut state = PreferencesState::open(cx.repo_root);
            state.tools_command(button);
            *cx.last_message = state.take_message();
            *cx.preferences = Some(state);
        }
        PaletteCommand::LauncherFolderMenuInstall | PaletteCommand::LauncherFolderMenuRemove => {
            let mut state = PreferencesState::open(cx.repo_root);
            state.folder_menu_command(cmd == PaletteCommand::LauncherFolderMenuRemove);
            *cx.last_message = state.take_message();
            *cx.preferences = Some(state);
        }
        PaletteCommand::LauncherBecomePrimary
        | PaletteCommand::LauncherRemoveLegacy
        | PaletteCommand::LauncherForgetGone => {
            use crate::preferences::InstallButton as B;
            let button = match cmd {
                PaletteCommand::LauncherBecomePrimary => B::BecomePrimary,
                PaletteCommand::LauncherRemoveLegacy => B::RemoveLegacy,
                _ => B::Forget(0),
            };
            let mut state = PreferencesState::open(cx.repo_root);
            state.installation_command(button);
            *cx.last_message = state.take_message();
            *cx.preferences = Some(state);
        }
        PaletteCommand::ChangeGraphColorTheme => {
            *cx.input_mode = InputMode::LanguagePicker(Box::new(
                crate::language_picker::LanguagePicker::open_graph_color_theme(),
            ));
        }
        PaletteCommand::ToggleGraphRowTints => {
            *cx.last_message = Some(crate::language_picker::toggle_row_tints(
                &mut *cx.ui_lane_background,
            ));
        }
        PaletteCommand::KeybindingsResetAll => {
            let mut state = PreferencesState::open(cx.repo_root);
            state.show_keybindings(true);
            *cx.preferences = Some(state);
        }
        // The agent's view change is applied by the window's next tick
        // (`ui_intent`): these only say what the user chose.
        PaletteCommand::AgentShowPending => cx.ui_viewer.accept_pending(),
        PaletteCommand::AgentDismissPending => cx.ui_viewer.dismiss_pending(),
        PaletteCommand::AgentBack => cx.ui_viewer.request_back(),
        PaletteCommand::SequenceCancel => crate::sequence_run::request_cancel(),
        PaletteCommand::SequenceContinue => crate::sequence_run::request_continue(),
        PaletteCommand::AiExplainSelectedCommit => {
            if let Some(commit) =
                selected_log_commit(&*cx.log_result, &*cx.log_state, &*cx.log_order)
            {
                explain_commit(cx, commit);
            }
        }
        PaletteCommand::AiSplitSelectedCommit => {
            if let Some(commit) =
                selected_log_commit(&*cx.log_result, &*cx.log_state, &*cx.log_order)
            {
                split_commit_with_ai(cx, commit);
            }
        }
        PaletteCommand::AiExplainWip => explain_wip(cx),
        PaletteCommand::AiInsertPromptVariable => {
            let mut state = PreferencesState::open(cx.repo_root);
            state.open_prompt_variables();
            *cx.preferences = Some(state);
        }
        PaletteCommand::AiOpenAgent => {
            let mut state = PreferencesState::open(cx.repo_root);
            state.open_ai_profile_list();
            state.ai_open_default_agent();
            *cx.preferences = Some(state);
        }
        PaletteCommand::AiEditProfile => {
            let mut state = PreferencesState::open(cx.repo_root);
            state.open_ai_profile_list();
            *cx.preferences = Some(state);
        }
        PaletteCommand::ComposerMoveFile => {
            *cx.compose_move_requested = true;
        }
        PaletteCommand::AiComposeCommits => {
            super::start_ai(cx, AiRequest::Compose);
        }
        PaletteCommand::StageGuardShowInFile => {
            crate::stage_guard_dialog::show_in_file();
        }
        PaletteCommand::StashPushWithMessage => {
            *cx.input_mode = InputMode::Prompt(Box::new(
                crate::prompt_dialog::PromptDialog::stash(cx.ai_ui.enabled()),
            ));
        }
        // The command itself is the explicit action: it opens the dialog and
        // starts the request, which shows "writing..." in the field.
        PaletteCommand::AiGenerateStashMessage => {
            let mut dialog = crate::prompt_dialog::PromptDialog::stash(cx.ai_ui.enabled());
            cx.ai_ui.start_into(
                cx.session,
                cx.repo,
                AiRequest::StashMessage,
                &mut *cx.last_message,
            );
            dialog.ai_running = cx.ai_ui.running();
            *cx.input_mode = InputMode::Prompt(Box::new(dialog));
        }
        PaletteCommand::AiGenerateCommitMessage => {
            generate_commit_message(cx);
        }
        PaletteCommand::Hosting(command) => {
            let anchor = (cx.refs_area.x + 2, cx.refs_area.y + 2);
            let effect = cx.hosting.run_command(command, anchor, &mut *cx.clipboard);
            apply_hosting_effect(cx, effect);
        }
        PaletteCommand::ToggleColumn(c) => {
            toggle_in(&mut *cx.log_column_hidden, c);
        }
        PaletteCommand::LogColumn(cmd) => super::log_columns::run(cx, cmd),

        // -- App -------------------------------
        PaletteCommand::Quit => {
            *cx.should_quit = true;
            *cx.quit_how = Some("the palette's Quit command");
        }
        _ => return false,
    }
    true
}
