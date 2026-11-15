//! The rows of the writing help on a host (`[ai.hosting_reply]`), shown under
//! the "Replies on a host" action block of the AI page: the editable base
//! prompt (global only; a repository changes the prompt through its own
//! instructions), then the "Repository access" group.
//!
//! The AI only ever drafts: none of these rows lets it post. Letting it read
//! the repository is **off by default**: the group sets what the per-use
//! choice in the composer starts from.

use gitpoulpi_i18n::msg::ai_hosting_reply as msg;

use super::*;

impl PreferencesState {
    /// The base prompt row and the repository access group.
    pub(in crate::preferences) fn hosting_reply_rows(&self) -> Vec<Field> {
        let hr = &self.config.ai.hosting_reply;
        let mut base = Field::plain(
            msg::BASE_PROMPT_LABEL.get(),
            Control::Area(hr.base_prompt.clone()),
            Target::Ai(AiTarget::Instructions(
                AiScope::Global,
                AiSection::HostingBase,
            )),
        )
        .with_help(msg::BASE_PROMPT_HELP.get());
        if hr.base_prompt.is_empty() {
            base = base.with_placeholder(gitpoulpi_ai::hosting_reply::BASE_PROMPT);
        }
        vec![
            base,
            Field::heading(msg::REPO_HEADING.get()),
            Field::global(
                msg::READ_REPOSITORY_LABEL.get(),
                "ai.hosting_reply.read_repository",
                Control::Toggle(hr.read_repository),
            )
            .with_help(msg::READ_REPOSITORY_HELP.get()),
            Field::global(
                msg::REUSE_SESSION_LABEL.get(),
                "ai.hosting_reply.reuse_agent_session",
                Control::Toggle(hr.reuse_agent_session),
            )
            .with_help(msg::REUSE_SESSION_HELP.get()),
            Field::global(
                msg::CONTEXT_TTL_LABEL.get(),
                "ai.hosting_reply.context_ttl_minutes",
                Control::Number {
                    value: i64::from(hr.context_ttl_minutes_clamped()),
                    min: 1,
                    max: i64::from(gitpoulpi_config::MAX_CONTEXT_TTL_MINUTES),
                },
            )
            .with_help(msg::CONTEXT_TTL_HELP.get()),
        ]
    }
}
