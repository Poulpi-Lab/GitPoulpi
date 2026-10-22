//! [`AiAction::RewriteScript`]: asks for a Lua script, parses it, has it
//! checked (the caller's [`ScriptChecker`], which is the daemon's
//! `rewrite_script_check`) and sends a compile error back once.

use super::Job;
use crate::prompts::{self, SampleCommit};
use crate::{parse, AiClient, Error, Result};

/// What the user asked for.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RewriteScriptRequest {
    pub instruction: String,
    /// Commit headers sent along. Empty unless the user ticked "Send a
    /// sample"; nothing else about the history is ever sent.
    pub sample: Vec<SampleCommit>,
}

impl RewriteScriptRequest {
    /// A request for `instruction`. The first `n` of `commits` go along as a
    /// sample only when `send_sample` is true (the user's tick, off by
    /// default); otherwise no commit data is kept at all.
    pub fn new(
        instruction: &str,
        commits: &[gitpoulpi_core::domain::CommitNode],
        send_sample: bool,
        n: usize,
    ) -> Self {
        let sample = if send_sample {
            commits
                .iter()
                .take(n.min(prompts::MAX_SAMPLE))
                .map(SampleCommit::from_node)
                .collect()
        } else {
            Vec::new()
        };
        Self {
            instruction: instruction.to_string(),
            sample,
        }
    }
}

/// The script and what the check said about it.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct RewriteScriptOutcome {
    pub script: String,
    /// Check problems left in `script` (empty = it compiles).
    pub problems: Vec<String>,
    /// The check ran (false when the daemon could not be asked).
    pub checked: bool,
    /// The first answer was rejected and the model was asked again.
    pub retried: bool,
}

/// Compiles a script without running it: the daemon's
/// `rewrite_script_check`, behind a seam so this crate stays free of IPC.
/// `Ok(problems)` (empty = fine); `Err` = the check itself could not run.
#[async_trait::async_trait]
pub trait ScriptChecker: Send + Sync {
    async fn check(&self, script: &str) -> std::result::Result<Vec<String>, String>;
}

impl Job {
    /// The system prompt and the user message a script request sends, for
    /// the "what is sent" dialog and the CLI.
    pub fn rewrite_script_messages(&self, request: &RewriteScriptRequest) -> (String, String) {
        (
            self.system_prompt(),
            prompts::rewrite_script(&request.instruction, &request.sample),
        )
    }

    /// Over the real network transport (or the agent subprocess).
    #[cfg(feature = "network")]
    pub async fn write_rewrite_script(
        &self,
        request: &RewriteScriptRequest,
        checker: &dyn ScriptChecker,
    ) -> Result<RewriteScriptOutcome> {
        let client = crate::client_for_network_in(
            &self.profile,
            self.api_key.clone(),
            self.repo_path().as_deref(),
        )?;
        self.write_rewrite_script_with(&*client, request, checker)
            .await
    }

    /// [`Job::write_rewrite_script`] over a caller-supplied client (tests).
    pub async fn write_rewrite_script_with(
        &self,
        client: &dyn AiClient,
        request: &RewriteScriptRequest,
        checker: &dyn ScriptChecker,
    ) -> Result<RewriteScriptOutcome> {
        if request.instruction.trim().is_empty() {
            return Err(Error::Invalid("say what the script must do".into()));
        }
        let (_, prompt) = self.rewrite_script_messages(request);
        let answer = self.run_with(client, &prompt).await?;
        let reason = match evaluate(&answer, checker).await {
            Ok(done) if done.problems.is_empty() => return Ok(done),
            Ok(done) => format!(
                "the script does not compile: {}\nThe script was:\n```lua\n{}```",
                done.problems.join("; "),
                done.script
            ),
            Err(e) => e.to_string(),
        };
        let retry = prompts::rewrite_script_retry(&prompt, &reason);
        let answer = self.run_with(client, &retry).await?;
        let mut done = evaluate(&answer, checker).await?;
        done.retried = true;
        Ok(done)
    }
}

async fn evaluate(answer: &str, checker: &dyn ScriptChecker) -> Result<RewriteScriptOutcome> {
    let script = parse::rewrite_script(answer)?;
    let (problems, checked) = match checker.check(&script).await {
        Ok(problems) => (problems, true),
        Err(_) => (Vec::new(), false),
    };
    Ok(RewriteScriptOutcome {
        script,
        problems,
        checked,
        retried: false,
    })
}
