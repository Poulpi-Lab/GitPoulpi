//! [`AiAction::RewriteRules`]: asks for the rules and settings of a history
//! rewrite and parses them for the workbench.
//!
//! **Data only, structurally.** This module has no checker, no daemon and no
//! backend: its one output, [`RewriteRulesOutcome`], is a list of rules, a
//! scope and settings for the caller to put in a form. There is no code path
//! from here to a preview, an apply or any write.

use super::Job;
use crate::prompts::{self, RulesContext};
use crate::{AiClient, Error, Result, RulesAnswer};

/// What the user asked for.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RewriteRulesRequest {
    pub description: String,
    /// Empty unless the user ticked "Send the repository context".
    pub context: RulesContext,
}

/// The proposal, and whether it took a second ask.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct RewriteRulesOutcome {
    pub answer: RulesAnswer,
    /// The first answer could not be read and the model was asked again.
    pub retried: bool,
}

impl Job {
    /// The system prompt and the user message a rules request sends, for the
    /// "what is sent" pane and the CLI.
    pub fn rewrite_rules_messages(&self, request: &RewriteRulesRequest) -> (String, String) {
        (
            self.system_prompt(),
            prompts::rewrite_rules(&request.description, &request.context),
        )
    }

    /// Over the real network transport (or the agent subprocess).
    #[cfg(feature = "network")]
    pub async fn create_rewrite_rules(
        &self,
        request: &RewriteRulesRequest,
    ) -> Result<RewriteRulesOutcome> {
        let client = crate::client_for_network_in(
            &self.profile,
            self.api_key.clone(),
            self.repo_path().as_deref(),
        )?;
        self.create_rewrite_rules_with(&*client, request).await
    }

    /// [`Job::create_rewrite_rules`] over a caller-supplied client (tests).
    pub async fn create_rewrite_rules_with(
        &self,
        client: &dyn AiClient,
        request: &RewriteRulesRequest,
    ) -> Result<RewriteRulesOutcome> {
        if request.description.trim().is_empty() {
            return Err(Error::Invalid(
                "say how the history must be rewritten".into(),
            ));
        }
        let (_, prompt) = self.rewrite_rules_messages(request);
        let answer = self.run_with(client, &prompt).await?;
        match crate::parse::rewrite_rules(&answer) {
            Ok(answer) => Ok(RewriteRulesOutcome {
                answer,
                retried: false,
            }),
            Err(e) => {
                let retry = prompts::rewrite_rules_retry(&prompt, &e.to_string());
                let answer = self.run_with(client, &retry).await?;
                Ok(RewriteRulesOutcome {
                    answer: crate::parse::rewrite_rules(&answer)?,
                    retried: true,
                })
            }
        }
    }
}
