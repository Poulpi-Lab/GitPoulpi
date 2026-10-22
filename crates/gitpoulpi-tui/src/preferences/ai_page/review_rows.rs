//! The Code review context rows of the AI page: where the local agent runs
//! during a review and which project files the model is given (`[ai.review]`,
//! slice S5b). Global rows write `config.toml`; the repository page shows the
//! same four with "use global" and writes `RepoPreferences::ai_review`.

use gitpoulpi_config::{RepoAiReview, RunInRepo};
use gitpoulpi_i18n::msg::review_options as msg;

use super::*;

/// One of the four settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::preferences) enum ReviewRow {
    Run,
    Claude,
    Agents,
    Extra,
    /// The repository page's "No extra file" switch.
    NoExtra,
}

impl ReviewRow {
    fn key(self) -> &'static str {
        match self {
            Self::Run => "ai.review.run_in_repository",
            Self::Claude => "ai.review.include_claude_md",
            Self::Agents => "ai.review.include_agents_md",
            Self::Extra => "ai.review.extra_file",
            Self::NoExtra => "ai.review.no_extra_file",
        }
    }
}

fn run_label(run: RunInRepo) -> &'static str {
    match run {
        RunInRepo::Profile => msg::RUN_PROFILE,
        RunInRepo::Yes => msg::RUN_YES,
        RunInRepo::No => msg::RUN_NO,
    }
    .get()
}

fn on_off(on: bool) -> &'static str {
    if on {
        msg::ON.get()
    } else {
        msg::OFF.get()
    }
}

impl PreferencesState {
    /// The four rows, for `scope`.
    pub(in crate::preferences) fn review_rows(&self, scope: AiScope) -> Vec<Field> {
        let global = &self.config.ai.review;
        let own = &self.repo_state.preferences.ai_review;
        let target = |row| Target::Ai(AiTarget::Review(scope, row));
        match scope {
            AiScope::Global => vec![
                Field::plain(
                    msg::RUN_LABEL.get(),
                    Control::Choice {
                        options: RunInRepo::ALL
                            .iter()
                            .map(|r| run_label(*r).to_string())
                            .collect(),
                        index: RunInRepo::ALL
                            .iter()
                            .position(|r| *r == global.run_in_repository)
                            .unwrap_or(0),
                    },
                    target(ReviewRow::Run),
                )
                .with_help(msg::RUN_HELP.get()),
                Field::plain(
                    msg::CLAUDE_LABEL.get(),
                    Control::Toggle(global.include_claude_md),
                    target(ReviewRow::Claude),
                )
                .with_help(msg::CLAUDE_HELP.get()),
                Field::plain(
                    msg::AGENTS_LABEL.get(),
                    Control::Toggle(global.include_agents_md),
                    target(ReviewRow::Agents),
                )
                .with_help(msg::AGENTS_HELP.get()),
                Field::plain(
                    msg::EXTRA_LABEL.get(),
                    Control::Text(global.extra_file.clone()),
                    target(ReviewRow::Extra),
                )
                .with_placeholder(msg::EXTRA_NONE.get())
                .with_help(msg::EXTRA_HELP.get()),
                Field::button(
                    gitpoulpi_i18n::msg::dialog::BROWSE.get(),
                    PrefAction::Ai(AiButton::BrowseExtra(AiScope::Global)),
                ),
            ],
            AiScope::Repo => {
                let tri = |value: Option<bool>, global: bool| Control::Choice {
                    options: vec![
                        msg_use_global(on_off(global)),
                        msg::ON.get().to_string(),
                        msg::OFF.get().to_string(),
                    ],
                    index: match value {
                        None => 0,
                        Some(true) => 1,
                        Some(false) => 2,
                    },
                };
                let mut run_options = vec![msg_use_global(run_label(global.run_in_repository))];
                run_options.extend(RunInRepo::ALL.iter().map(|r| run_label(*r).to_string()));
                let mut rows = vec![
                    Field::plain(
                        msg::RUN_LABEL.get(),
                        Control::Choice {
                            options: run_options,
                            index: own.run_in_repository.map_or(0, |r| {
                                1 + RunInRepo::ALL.iter().position(|x| *x == r).unwrap_or(0)
                            }),
                        },
                        target(ReviewRow::Run),
                    )
                    .with_help(msg::RUN_HELP.get()),
                    Field::plain(
                        msg::CLAUDE_LABEL.get(),
                        tri(own.include_claude_md, global.include_claude_md),
                        target(ReviewRow::Claude),
                    )
                    .with_help(msg::CLAUDE_HELP.get()),
                    Field::plain(
                        msg::AGENTS_LABEL.get(),
                        tri(own.include_agents_md, global.include_agents_md),
                        target(ReviewRow::Agents),
                    )
                    .with_help(msg::AGENTS_HELP.get()),
                ];
                let no_extra = own.no_extra_file == Some(true);
                let shown = if no_extra || global.extra_file.trim().is_empty() {
                    msg::EXTRA_NONE.get().to_string()
                } else {
                    global.extra_file.clone()
                };
                rows.push(
                    Field::plain(
                        msg::EXTRA_LABEL.get(),
                        Control::Text(own.extra_file.clone().unwrap_or_default()),
                        target(ReviewRow::Extra),
                    )
                    .with_placeholder(&shown)
                    .with_help(msg::EXTRA_HELP.get()),
                );
                rows.push(Field::button(
                    gitpoulpi_i18n::msg::dialog::BROWSE.get(),
                    PrefAction::Ai(AiButton::BrowseExtra(AiScope::Repo)),
                ));
                rows.push(
                    Field::plain(
                        msg::NO_EXTRA_LABEL.get(),
                        Control::Toggle(no_extra),
                        target(ReviewRow::NoExtra),
                    )
                    .with_help(msg::NO_EXTRA_HELP.get()),
                );
                if !own.is_empty() {
                    rows.push(Field::button(
                        gitpoulpi_i18n::msg::ai_page::RESET_TO_GLOBAL.get(),
                        PrefAction::Ai(AiButton::ResetReview),
                    ));
                }
                rows
            }
        }
    }

    /// A pick in one of the choice rows.
    pub(super) fn review_commit_choice(&mut self, scope: AiScope, row: ReviewRow, next: usize) {
        match scope {
            AiScope::Global => {
                if let (ReviewRow::Run, Some(run)) = (row, RunInRepo::ALL.get(next)) {
                    self.write_global(row.key(), run.key());
                }
            }
            AiScope::Repo => {
                let flag = match next {
                    0 => None,
                    1 => Some(true),
                    _ => Some(false),
                };
                self.edit_repo_review(|o| match row {
                    ReviewRow::Run => {
                        o.run_in_repository = next
                            .checked_sub(1)
                            .and_then(|i| RunInRepo::ALL.get(i).copied())
                    }
                    ReviewRow::Claude => o.include_claude_md = flag,
                    ReviewRow::Agents => o.include_agents_md = flag,
                    ReviewRow::Extra | ReviewRow::NoExtra => {}
                });
            }
        }
    }

    /// A toggle of the global rows.
    pub(super) fn review_commit_bool(&mut self, scope: AiScope, row: ReviewRow, value: bool) {
        if scope == AiScope::Global && matches!(row, ReviewRow::Claude | ReviewRow::Agents) {
            self.write_global(row.key(), if value { "true" } else { "false" });
        }
        // The switch and a file of the repository exclude each other.
        if scope == AiScope::Repo && row == ReviewRow::NoExtra {
            self.edit_repo_review(|o| {
                o.no_extra_file = value.then_some(true);
                if value {
                    o.extra_file = None;
                }
            });
        }
    }

    /// The extra file field.
    pub(super) fn review_commit_text(&mut self, scope: AiScope, draft: &str) {
        let value = draft.trim().to_string();
        match scope {
            AiScope::Global => {
                self.write_global(ReviewRow::Extra.key(), &value);
            }
            // Empty follows the global value.
            AiScope::Repo => self.edit_repo_review(|o| {
                if !value.is_empty() {
                    o.no_extra_file = None;
                }
                o.extra_file = (!value.is_empty()).then_some(value);
            }),
        }
    }

    /// Back to the global values: the repository keeps none of its own.
    pub(super) fn review_reset_repo(&mut self) {
        self.edit_repo_review(|o| *o = RepoAiReview::default());
        self.message = Some(gitpoulpi_i18n::msg::ai_page::back_to_global(
            gitpoulpi_i18n::msg::review_ai::TITLE_CODE_REVIEW.get(),
        ));
    }

    fn edit_repo_review(&mut self, change: impl FnOnce(&mut RepoAiReview)) {
        change(&mut self.repo_state.preferences.ai_review);
        self.write_repo_state();
    }
}

fn msg_use_global(name: &str) -> String {
    gitpoulpi_i18n::msg::ai_page::use_global(name)
}
