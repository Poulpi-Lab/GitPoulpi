use super::*;
use gitpoulpi_i18n::{msg::ai_page as msg, M};

impl AiSection {
    /// The quick-insert chips: (button text, sentence appended). The
    /// sentence is sent to the model, so it is read with `english()`.
    pub(in crate::preferences) fn chips(self) -> &'static [(M, M)] {
        match self {
            Self::Global => &[
                (msg::CHIP_RESPOND_LANGUAGE, msg::CHIP_RESPOND_LANGUAGE_TEXT),
                (msg::CHIP_CONCISE, msg::CHIP_CONCISE_TEXT),
                (msg::CHIP_IMPERATIVE, msg::CHIP_IMPERATIVE_TEXT),
                (msg::CHIP_TECHNICAL, msg::CHIP_TECHNICAL_TEXT),
                (msg::CHIP_CASUAL, msg::CHIP_CASUAL_TEXT),
            ],
            Self::Action(AiAction::CommitMessage) => &[
                (msg::CHIP_CONVENTIONAL, msg::CHIP_CONVENTIONAL_TEXT),
                (msg::CHIP_LIMIT_50, msg::CHIP_LIMIT_50_TEXT),
                (msg::CHIP_ISSUE_NUMBERS, msg::CHIP_ISSUE_NUMBERS_TEXT),
                (msg::CHIP_NO_BODY, msg::CHIP_NO_BODY_TEXT),
                (msg::CHIP_WHAT_WHY, msg::CHIP_WHAT_WHY_TEXT),
                (msg::CHIP_IMPERATIVE, msg::CHIP_IMPERATIVE_TEXT),
            ],
            Self::Action(AiAction::ExplainChanges) => &[
                (msg::CHIP_BUSINESS_IMPACT, msg::CHIP_BUSINESS_IMPACT_TEXT),
                (msg::CHIP_BULLETS, msg::CHIP_BULLETS_TEXT),
                (msg::CHIP_COMPONENTS, msg::CHIP_COMPONENTS_TEXT),
                (msg::CHIP_BEGINNER, msg::CHIP_BEGINNER_TEXT),
            ],
            Self::Action(AiAction::StashMessage) => &[
                (msg::CHIP_TASK_CONTEXT, msg::CHIP_TASK_CONTEXT_TEXT),
                (msg::CHIP_ACTION_VERBS, msg::CHIP_ACTION_VERBS_TEXT),
                (msg::CHIP_BRANCH_NAME, msg::CHIP_BRANCH_NAME_TEXT),
            ],
            Self::Action(AiAction::PullRequestDescription) => &[
                (msg::CHIP_TESTING, msg::CHIP_TESTING_TEXT),
                (msg::CHIP_BREAKING, msg::CHIP_BREAKING_TEXT),
                (msg::CHIP_REVIEWER, msg::CHIP_REVIEWER_TEXT),
            ],
            Self::Action(AiAction::ConflictResolve) => &[
                (msg::CHIP_MINIMAL, msg::CHIP_MINIMAL_TEXT),
                (msg::CHIP_KEEP_BOTH, msg::CHIP_KEEP_BOTH_TEXT),
            ],
            Self::Action(AiAction::CodeReview) => {
                use gitpoulpi_i18n::msg::review_ai as ra;
                &[
                    (ra::CHIP_SECURITY, ra::CHIP_SECURITY_TEXT),
                    (ra::CHIP_TESTS, ra::CHIP_TESTS_TEXT),
                    (ra::CHIP_NITS, ra::CHIP_NITS_TEXT),
                ]
            }
            Self::Action(AiAction::SplitCommit) => &[],
            Self::Action(AiAction::RewriteScript) => &[],
            Self::Action(AiAction::RewriteRules) => &[],
            Self::Action(AiAction::CommitComposer) => &[
                (msg::CHIP_SMALL_COMMITS, msg::CHIP_SMALL_COMMITS_TEXT),
                (msg::CHIP_GROUP_FEATURE, msg::CHIP_GROUP_FEATURE_TEXT),
                (
                    msg::CHIP_SEPARATE_REFACTORS,
                    msg::CHIP_SEPARATE_REFACTORS_TEXT,
                ),
                (msg::CHIP_CONVENTIONAL, msg::CHIP_CONVENTIONAL_TEXT),
            ],
        }
    }
}

impl PreferencesState {
    pub(in crate::preferences) fn ai_general_fields(&self) -> Vec<Field> {
        let ai = &self.config.ai;
        let mut fields = vec![Field::global(
            msg::ENABLE_AI.get(),
            "ai.enabled",
            Control::Toggle(ai.enabled),
        )
        .with_help(msg::ENABLE_AI_HELP.get())];

        let profile_names: Vec<String> = ai.profiles.iter().map(profile_option).collect();
        let default_name = ai
            .profile(&ai.default_profile)
            .or_else(|| ai.profiles.first())
            .map(|p| p.display_name().to_string())
            .unwrap_or_else(|| msg::NO_PROFILE.get().to_string());

        // -- The default profile (the list on the left holds the profiles) --
        if !ai.profiles.is_empty() {
            let index = ai
                .profiles
                .iter()
                .position(|p| p.id == ai.default_profile)
                .unwrap_or(0);
            fields.push(
                Field::plain(
                    msg::DEFAULT_PROFILE.get(),
                    Control::Choice {
                        options: profile_names.clone(),
                        index,
                    },
                    Target::Ai(AiTarget::DefaultProfile),
                )
                .with_help(msg::DEFAULT_PROFILE_HELP.get()),
            );
        }
        // -- Global instructions -------------------------------------------
        let global = AiSection::Global;
        fields.push(Field::heading(global.title()));
        fields.push(
            Field::plain(
                msg::CUSTOM_INSTRUCTIONS.get(),
                Control::Area(ai.global_instructions.clone()),
                Target::Ai(AiTarget::Instructions(AiScope::Global, global)),
            )
            .with_help(msg::GLOBAL_HELP.get()),
        );

        // -- One block per action ------------------------------------------
        for action in AiAction::ALL {
            if !crate::preferences::ai_action_built(action) {
                continue;
            }
            let section = AiSection::Action(action);
            let settings = ai.actions.get(action);
            fields.push(Field::heading(section.title()));
            let mut options = vec![msg::default_option(&default_name)];
            options.extend(profile_names.iter().cloned());
            let index = ai
                .profiles
                .iter()
                .position(|p| !settings.profile.is_empty() && p.id == settings.profile)
                .map(|i| i + 1)
                .unwrap_or(0);
            fields.push(
                Field::plain(
                    msg::PROFILE.get(),
                    Control::Choice { options, index },
                    Target::Ai(AiTarget::ActionProfile(AiScope::Global, action)),
                )
                .with_help(AiSection::profile_help(action, false)),
            );
            fields.push(
                Field::plain(
                    msg::CUSTOM_INSTRUCTIONS.get(),
                    Control::Area(settings.instructions.clone()),
                    Target::Ai(AiTarget::Instructions(AiScope::Global, section)),
                )
                .with_help(msg::ACTION_INSTRUCTIONS_HELP.get()),
            );
            if action == AiAction::CodeReview {
                fields.extend(self.review_rows(AiScope::Global));
            }
        }

        // -- The archive of the old presets --------------------------------
        if !ai.legacy_presets.is_empty() {
            fields.push(Field::heading(msg::LEGACY_HEADING.get()));
            fields.push(
                Field::button(
                    if self.ai.legacy_open {
                        msg::HIDE_LEGACY.get()
                    } else {
                        msg::SHOW_LEGACY.get()
                    },
                    PrefAction::Ai(AiButton::ToggleLegacy),
                )
                .with_help(&msg::legacy_help(ai.legacy_presets.len() as u64)),
            );
            if self.ai.legacy_open {
                for preset in &ai.legacy_presets {
                    let name = if preset.name.is_empty() {
                        preset.id.as_str()
                    } else {
                        preset.name.as_str()
                    };
                    fields.push(Field::plain(
                        &format!("{name} ({})", action_label(preset.action)),
                        Control::Info(first_line_preview(&preset.system_prompt)),
                        Target::None,
                    ));
                }
            }
            fields.push(Field::button(
                msg::DELETE_ARCHIVE.get(),
                PrefAction::Ai(AiButton::DeleteLegacy),
            ));
        }
        fields
    }

    /// The rows of the selected connection profile (the list tab's detail).
    pub(in crate::preferences) fn ai_item_fields(&self) -> Vec<Field> {
        let ai = &self.config.ai;
        let mut fields: Vec<Field> = Vec::new();
        // -- The selected profile ---------------------------------------------
        if ai.profiles.is_empty() {
            fields.push(Field::plain(
                msg::NO_PROFILE_YET.get(),
                Control::Info(msg::NO_PROFILE_YET_INFO.get().into()),
                Target::None,
            ));
        } else {
            let index = self.ai.profile.min(ai.profiles.len() - 1);
            let profile = &ai.profiles[index];
            fields.push(Field::plain(
                msg::NAME.get(),
                Control::Text(profile.name.clone()),
                Target::Ai(AiTarget::ProfileName),
            ));
            fields.push(Field::plain(
                msg::PROVIDER.get(),
                Control::Choice {
                    options: AiProviderKind::ALL
                        .iter()
                        .map(|k| provider_label(*k).to_string())
                        .collect(),
                    index: AiProviderKind::ALL
                        .iter()
                        .position(|k| *k == profile.kind)
                        .unwrap_or(0),
                },
                Target::Ai(AiTarget::ProfileKind),
            ));
            if profile.kind == AiProviderKind::LocalAgent {
                use gitpoulpi_ai::presets::{self, Harness};
                let harness = presets::harness_of(profile);
                // The type of agent (what the suggestions follow) and the
                // exact command (free text) are two separate rows.
                let mut kinds: Vec<(String, String)> = presets::HARNESSES
                    .iter()
                    .map(|h| {
                        (
                            format!("{}  ({})", h.name, h.command),
                            h.command.to_string(),
                        )
                    })
                    .collect();
                kinds.push((
                    msg::HARNESS_OTHER.get().to_string(),
                    presets::OTHER_HARNESS.to_string(),
                ));
                fields.push(
                    Field::plain(
                        msg::AGENT_TYPE.get(),
                        Control::Combo {
                            options: kinds,
                            value: harness.map_or_else(
                                || presets::OTHER_HARNESS.to_string(),
                                |h| h.command.to_string(),
                            ),
                            custom: false,
                        },
                        Target::Ai(AiTarget::ProfileHarness),
                    )
                    .with_help(msg::AGENT_TYPE_HELP.get()),
                );
                fields.push(
                    Field::plain(
                        msg::COMMAND.get(),
                        Control::Text(profile.command.clone()),
                        Target::Ai(AiTarget::ProfileCommand),
                    )
                    .with_help(msg::COMMAND_HELP.get()),
                );
                let mode = profile.run_mode;
                let terminal_capable = harness.is_some_and(|h| h.terminal().is_some());
                if terminal_capable {
                    let terminal = mode == gitpoulpi_config::AgentRunMode::Terminal;
                    fields.push(
                        Field::plain(
                            msg::RUN_MODE.get(),
                            Control::Combo {
                                options: vec![
                                    (msg::RUN_MODE_HEADLESS.get().to_string(), "headless".into()),
                                    (msg::RUN_MODE_TERMINAL.get().to_string(), "terminal".into()),
                                ],
                                value: if terminal { "terminal" } else { "headless" }.into(),
                                custom: false,
                            },
                            Target::Ai(AiTarget::ProfileRunMode),
                        )
                        .with_help(msg::RUN_MODE_HELP.get()),
                    );
                }
                fields.push(
                    Field::plain(
                        msg::ARGUMENTS.get(),
                        Control::Text(join_args(profile.active_args())),
                        Target::Ai(AiTarget::ProfileArgs),
                    )
                    .with_help(msg::ARGUMENTS_HELP.get()),
                );
                if let Some(h) = harness {
                    let h_options = h.options_for(mode);
                    if !h_options.is_empty() {
                        let active: Vec<String> = h_options
                            .iter()
                            .filter(|o| Harness::option_active(o, profile.active_args()))
                            .map(|o| o.label())
                            .collect();
                        let options = h_options
                            .iter()
                            .enumerate()
                            .map(|(n, o)| {
                                let on = Harness::option_active(o, profile.active_args());
                                (
                                    format!(
                                        "[{}] {}  -  {}",
                                        if on { "x" } else { " " },
                                        o.label(),
                                        arg_purpose_text(o.purpose)
                                    ),
                                    n.to_string(),
                                )
                            })
                            .collect();
                        fields.push(
                            Field::plain(
                                msg::ARGUMENT_OPTIONS.get(),
                                Control::Combo {
                                    options,
                                    value: if active.is_empty() {
                                        msg::OPTIONS_NONE.get().to_string()
                                    } else {
                                        active.join(", ")
                                    },
                                    custom: false,
                                },
                                Target::Ai(AiTarget::ProfileOptions),
                            )
                            .with_help(msg::ARGUMENT_OPTIONS_HELP.get()),
                        );
                    }
                    if !h.model_flags.is_empty() {
                        let mut models =
                            vec![(msg::MODEL_DEFAULT_NO_FLAG.get().to_string(), String::new())];
                        models.extend(h.models.iter().map(|m| (m.to_string(), m.to_string())));
                        fields.push(
                            Field::plain(
                                msg::MODEL.get(),
                                Control::Combo {
                                    options: models,
                                    value: h.model_in(profile.active_args()).unwrap_or_default(),
                                    custom: true,
                                },
                                Target::Ai(AiTarget::ProfileAgentModel),
                            )
                            .with_help(msg::AGENT_MODEL_HELP.get()),
                        );
                    }
                }
                // -- Where it runs, how long it may take ------------------
                {
                    use gitpoulpi_config::AgentWorkdir;
                    let key = |w: AgentWorkdir| match w {
                        AgentWorkdir::Default => "default",
                        AgentWorkdir::Dedicated => "dedicated",
                        AgentWorkdir::Repository => "repository",
                    };
                    let options = vec![
                        (msg::WORKDIR_DEFAULT.get().to_string(), "default".into()),
                        (msg::WORKDIR_DEDICATED.get().to_string(), "dedicated".into()),
                        (
                            msg::WORKDIR_REPOSITORY.get().to_string(),
                            "repository".into(),
                        ),
                    ];
                    fields.push(
                        Field::plain(
                            msg::WORKDIR.get(),
                            Control::Combo {
                                options,
                                value: key(profile.workdir).into(),
                                custom: false,
                            },
                            Target::Ai(AiTarget::ProfileWorkdir),
                        )
                        .with_help(msg::WORKDIR_HELP.get()),
                    );
                }
                fields.push(
                    Field::plain(
                        msg::TIMEOUT.get(),
                        Control::Text(if profile.timeout_secs == 0 {
                            String::new()
                        } else {
                            profile.timeout_secs.to_string()
                        }),
                        Target::Ai(AiTarget::ProfileTimeout),
                    )
                    .with_placeholder(&msg::timeout_default(profile.effective_timeout_secs()))
                    .with_help(msg::TIMEOUT_HELP.get()),
                );
                if terminal_capable {
                    fields.push(
                        Field::button(msg::OPEN_AGENT.get(), PrefAction::Ai(AiButton::OpenAgent))
                            .with_help(msg::OPEN_AGENT_HELP.get()),
                    );
                }
                // -- Environment variables of that process only ----------
                for (n, (name, value)) in profile.env.iter().enumerate() {
                    let shown = if presets::env_value_is_secret(name, value) {
                        ENV_MASK.to_string()
                    } else {
                        value.clone()
                    };
                    let help = harness
                        .and_then(|h| h.env.iter().find(|e| e.name == name))
                        .map(|e| env_purpose_text(e.purpose))
                        .unwrap_or_else(|| msg::ENV_VALUE_HELP.get());
                    let label = format!("{} {name}", msg::ENV_PREFIX.get());
                    // A small table: `KEY  [value]  [x]`. The value button
                    // types a new value; the row is a plain text field for
                    // as long as that edit lasts.
                    let typing = self.edit.is_some() && self.ai.env_edit == Some((n, fields.len()));
                    if typing {
                        fields.push(
                            Field::plain(
                                &label,
                                Control::Text(shown),
                                Target::Ai(AiTarget::ProfileEnv(n)),
                            )
                            .with_help(help),
                        );
                    } else {
                        let shown = if shown.is_empty() {
                            "\u{2026}".to_string()
                        } else {
                            shown
                        };
                        fields.push(
                            Field::buttons(
                                &label,
                                vec![
                                    Btn::new(&shown, PrefAction::Ai(AiButton::EnvEdit(n)))
                                        .filling(),
                                    Btn::new("\u{2715}", PrefAction::Ai(AiButton::EnvRemove(n))),
                                ],
                            )
                            .with_help(help),
                        );
                    }
                }
                // Suggested names not set yet, each with what it is for.
                let suggestions: Vec<(String, String)> = harness
                    .map(|h| h.env)
                    .unwrap_or(&[])
                    .iter()
                    .filter(|e| !profile.env.contains_key(e.name))
                    .map(|e| {
                        (
                            format!("{}  -  {}", e.name, env_purpose_text(e.purpose)),
                            e.name.to_string(),
                        )
                    })
                    .collect();
                fields.push(
                    Field::plain(
                        msg::ENV_ADD.get(),
                        Control::Combo {
                            options: suggestions,
                            value: String::new(),
                            custom: true,
                        },
                        Target::Ai(AiTarget::ProfileEnvAdd),
                    )
                    .with_help(msg::ENV_ADD_HELP.get()),
                );
            } else {
                let preset = gitpoulpi_ai::presets::provider_for(profile);
                fields.push(
                    Field::plain(
                        msg::PROVIDER_PRESET.get(),
                        Control::Combo {
                            options: gitpoulpi_ai::presets::PROVIDERS
                                .iter()
                                .map(|p| (p.name.to_string(), p.name.to_string()))
                                .collect(),
                            value: preset
                                .map(|p| p.name.to_string())
                                .unwrap_or_else(|| msg::PRESET_CUSTOM.get().to_string()),
                            custom: false,
                        },
                        Target::Ai(AiTarget::ProfilePreset),
                    )
                    .with_help(msg::PROVIDER_PRESET_HELP.get()),
                );
                let mut models: Vec<(String, String)> = match preset {
                    Some(p) => p
                        .models
                        .iter()
                        .map(|m| (m.to_string(), m.to_string()))
                        .collect(),
                    None => vec![(
                        profile.kind.default_model().to_string(),
                        profile.kind.default_model().to_string(),
                    )],
                };
                models.retain(|(_, v)| !v.is_empty());
                fields.push(
                    Field::plain(
                        msg::MODEL.get(),
                        Control::Combo {
                            options: models,
                            value: profile.model.clone(),
                            custom: true,
                        },
                        Target::Ai(AiTarget::ProfileModel),
                    )
                    .with_help(&msg::model_help(profile.kind.default_model())),
                );
                fields.push(
                    Field::plain(
                        msg::BASE_URL.get(),
                        Control::Text(profile.base_url.clone()),
                        Target::Ai(AiTarget::ProfileBaseUrl),
                    )
                    .with_help(&msg::base_url_help(profile.kind.default_base_url())),
                );
                // No entry at all: never probed (AI is off, so Settings does
                // not spend a credential-helper call on every open).
                let present = self.ai.key_present.get(&profile.token_ref).copied();
                fields.push(
                    Field::plain(
                        msg::API_KEY.get(),
                        Control::Text(match present {
                            Some(Some(true)) => msg::KEY_STORED.get().to_string(),
                            Some(Some(false)) => msg::KEY_NOT_SET.get().to_string(),
                            Some(None) => msg::KEY_CHECKING.get().to_string(),
                            None => msg::KEY_NOT_CHECKED.get().to_string(),
                        }),
                        Target::Ai(AiTarget::ProfileKey),
                    )
                    .with_help(&msg::api_key_help(
                        crate::dialog_kit::key_name(crate::keymap::chord::Key::Enter),
                    )),
                );
            }
            fields.push(Field::button(
                msg::TEST_CONNECTION.get(),
                PrefAction::Ai(AiButton::Test),
            ));
            let test = match self.ai.tests.get(&profile.id) {
                None => None,
                Some(TestState::Running) => Some(msg::TEST_RUNNING.get().to_string()),
                Some(TestState::Ok) => Some(msg::TEST_OK.get().to_string()),
                Some(TestState::Failed(why)) => Some(msg::test_failed(why)),
            };
            if let Some(test) = test {
                fields.push(Field::plain(
                    msg::LAST_TEST.get(),
                    Control::Info(test),
                    Target::None,
                ));
            }
        }

        fields
    }

    /// Repo-Specific Settings > AI: one block per action, each row an
    /// override of the global page (empty inherits).
    pub(in crate::preferences) fn repo_ai_fields(&self) -> Vec<Field> {
        let ai = &self.config.ai;
        let mut fields = vec![Field::heading(&msg::repo_overrides(&self.repo_label))];
        if !ai.enabled {
            // AI off: no per-repository AI control at all, only where to
            // turn it on.
            fields.push(Field::plain(
                gitpoulpi_i18n::msg::settings_general::AI_ASSISTANT.get(),
                Control::Info(gitpoulpi_i18n::msg::merge::MSG_AI_OFF.get().into()),
                Target::None,
            ));
            return fields;
        }
        let profile_names: Vec<String> = ai.profiles.iter().map(profile_option).collect();
        let repo_ai = &self.repo_state.preferences.ai_instructions;
        fields.push(Field::heading(msg::REPO_INSTRUCTIONS_HEADING.get()));
        fields.push(
            Field::plain(
                msg::COMBINE.get(),
                Control::Choice {
                    options: RepoInstructionsMode::ALL
                        .iter()
                        .map(|m| mode_label(*m).to_string())
                        .collect(),
                    index: RepoInstructionsMode::ALL
                        .iter()
                        .position(|m| *m == repo_ai.mode)
                        .unwrap_or(0),
                },
                Target::Ai(AiTarget::RepoMode),
            )
            .with_help(msg::COMBINE_HELP.get()),
        );
        let mut repo_area = Field::plain(
            msg::CUSTOM_INSTRUCTIONS.get(),
            Control::Area(repo_ai.text.clone()),
            Target::Ai(AiTarget::Instructions(AiScope::Repo, AiSection::Global)),
        )
        .with_help(msg::REPO_INSTRUCTIONS_HELP.get());
        if repo_ai.text.is_empty() {
            repo_area = repo_area.with_placeholder(msg::NO_REPO_INSTRUCTIONS.get());
        }
        fields.push(repo_area);
        for action in AiAction::ALL {
            if !crate::preferences::ai_action_built(action) {
                continue;
            }
            let section = AiSection::Action(action);
            let global = ai.actions.get(action);
            let over = self.repo_override(action);
            fields.push(Field::heading(section.title()));
            // What the global settings resolve this action to.
            let global_name = ai
                .profile(&global.profile)
                .or_else(|| ai.profile(&ai.default_profile))
                .or_else(|| ai.profiles.first())
                .map(|p| p.display_name().to_string())
                .unwrap_or_else(|| msg::NO_PROFILE.get().to_string());
            let mut options = vec![msg::use_global(&global_name)];
            options.extend(profile_names.iter().cloned());
            let index = ai
                .profiles
                .iter()
                .position(|p| !over.profile.is_empty() && p.id == over.profile)
                .map(|i| i + 1)
                .unwrap_or(0);
            fields.push(
                Field::plain(
                    msg::PROFILE.get(),
                    Control::Choice { options, index },
                    Target::Ai(AiTarget::ActionProfile(AiScope::Repo, action)),
                )
                .with_help(AiSection::profile_help(action, true)),
            );
            let mut area = Field::plain(
                msg::CUSTOM_INSTRUCTIONS.get(),
                Control::Area(over.instructions.clone()),
                Target::Ai(AiTarget::Instructions(AiScope::Repo, section)),
            )
            .with_help(msg::REPO_ACTION_HELP.get());
            if over.instructions.is_empty() {
                let shown = if global.instructions.is_empty() {
                    msg::NO_GLOBAL_INSTRUCTIONS.get()
                } else {
                    global.instructions.as_str()
                };
                area = area.with_placeholder(shown);
            }
            fields.push(area);
            if !over.is_empty() {
                fields.push(Field::button(
                    msg::RESET_TO_GLOBAL.get(),
                    PrefAction::Ai(AiButton::ResetRepo(action)),
                ));
            }
            if action == AiAction::CodeReview {
                fields.extend(self.review_rows(AiScope::Repo));
            }
        }
        fields
    }
}
