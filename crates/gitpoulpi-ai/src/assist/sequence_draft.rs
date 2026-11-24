//! [`AiAction::SequenceDraft`](gitpoulpi_config::AiAction): asks for a new
//! action sequence, or a changed one, and parses it for the caller to show.
//!
//! **Data only, structurally.** This module has no daemon, no backend and no
//! config writer: its one output, [`SequenceDraftOutcome`], is a definition
//! plus what was left out. There is no code path from here to a save or to a
//! run; the user validates the proposal and the caller saves it.

use gitpoulpi_config::sequences::SequenceDef;

use super::Job;
use crate::parse::SequenceProposal;
use crate::prompts::{self, SequenceContext};
use crate::{AiClient, Error, Result};

/// What the user asked for.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SequenceDraftRequest {
    /// What to create, or how to change `edit`.
    pub description: String,
    /// Edit mode: the sequence to change. The proposal keeps its id, scope
    /// and owner.
    pub edit: Option<SequenceDef>,
    /// Ids already in use (a new proposal gets a different one).
    pub taken_ids: Vec<String>,
    /// Empty unless the user allowed sending names of the installation.
    pub context: SequenceContext,
}

/// The proposal, and whether it took a second ask.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct SequenceDraftOutcome {
    pub proposal: SequenceProposal,
    /// The first answer held nothing usable and the model was asked again
    /// with the reasons.
    pub retried: bool,
}

/// Why an answer is not usable, for the retry.
fn unusable(parsed: &Result<SequenceProposal>) -> Option<String> {
    match parsed {
        Err(e) => Some(e.to_string()),
        Ok(p) if p.def.steps.is_empty() => {
            let why: Vec<String> = p
                .problems
                .iter()
                .map(|x| format!("step {} ({}): {}", x.position, x.what, x.reason))
                .collect();
            Some(if why.is_empty() {
                "the sequence has no steps".to_string()
            } else {
                format!("no step was usable. {}", why.join(". "))
            })
        }
        Ok(_) => None,
    }
}

/// Edit mode keeps what the model must not change.
fn keep_identity(mut proposal: SequenceProposal, edit: &SequenceDef) -> SequenceProposal {
    let def = &mut proposal.def;
    def.id = edit.id.clone();
    def.scope = edit.scope;
    def.owner = edit.owner.clone();
    def.extra = edit.extra.clone();
    if def.description.is_none() {
        def.description = edit.description.clone();
    }
    proposal
}

impl Job {
    /// The system prompt and the user message a draft request sends, for the
    /// "what is sent" pane and the CLI.
    pub fn sequence_draft_messages(&self, request: &SequenceDraftRequest) -> (String, String) {
        (
            self.system_prompt(),
            prompts::sequence_draft(
                &request.description,
                request.edit.as_ref(),
                &request.context,
            ),
        )
    }

    /// Over the real network transport (or the agent subprocess).
    #[cfg(feature = "network")]
    pub async fn draft_sequence(
        &self,
        request: &SequenceDraftRequest,
    ) -> Result<SequenceDraftOutcome> {
        let client = crate::client_for_network_in(
            &self.profile,
            self.api_key.clone(),
            self.repo_path().as_deref(),
            &self.privacy,
        )?;
        self.draft_sequence_with(&*client, request).await
    }

    /// [`Job::draft_sequence`] over a caller-supplied client (tests).
    pub async fn draft_sequence_with(
        &self,
        client: &dyn AiClient,
        request: &SequenceDraftRequest,
    ) -> Result<SequenceDraftOutcome> {
        if request.description.trim().is_empty() {
            return Err(Error::Invalid(
                "say what the sequence must do, or how to change it".into(),
            ));
        }
        let (_, prompt) = self.sequence_draft_messages(request);
        let mut taken = request.taken_ids.clone();
        if let Some(edit) = &request.edit {
            taken.retain(|id| *id != edit.id);
        }
        let finish = |parsed: Result<SequenceProposal>| {
            parsed.map(|p| match &request.edit {
                Some(edit) => keep_identity(p, edit),
                None => p,
            })
        };
        let answer = self.run_with(client, &prompt).await?;
        let first = crate::parse::sequence_draft(&answer, &taken);
        let Some(reason) = unusable(&first) else {
            return Ok(SequenceDraftOutcome {
                proposal: finish(first)?,
                retried: false,
            });
        };
        let retry = prompts::sequence_draft_retry(&prompt, &reason);
        let answer = self.run_with(client, &retry).await?;
        Ok(SequenceDraftOutcome {
            proposal: finish(crate::parse::sequence_draft(&answer, &taken))?,
            retried: true,
        })
    }
}
