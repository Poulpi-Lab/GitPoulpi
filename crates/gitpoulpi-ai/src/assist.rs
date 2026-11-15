//! The glue every frontend shares: resolve an action to a profile and its
//! instructions, fetch the API key, render a diff as prompt text, run it.
//!
//! The TUI and the CLI both go through here so a repository's override,
//! the key lookup and the prompt can never differ between them.

use gitpoulpi_config::{
    AiAction, AiActionSettings, AiConfig, AiProfile, AiResolveError, RepoAiInstructions,
    WorkspaceAi,
};
use std::collections::HashSet;

use gitpoulpi_core::domain::{
    validate_split, CommitNode, DiffLine, FileDiff, FileStatus, LineOrigin, Oid, SplitPart,
    SplitPiece, SplitSource, WholeReason,
};
use gitpoulpi_secrets::TokenStore;

mod rewrite_rules;
mod rewrite_script;
pub use rewrite_rules::{RewriteRulesOutcome, RewriteRulesRequest};
pub use rewrite_script::{RewriteScriptOutcome, RewriteScriptRequest, ScriptChecker};

use crate::parse::PlannedSplit;

use crate::limits::{Cut, LimitGate, LimitPolicy, Limits, Part};
use crate::review_context::{ContextSource, EffectiveReviewContext, Gathered};
use crate::vars::{self, PromptContext};
use crate::{prompts, AiClient, Error, Prepared, Privacy, Result, SendGate};

/// Everything needed to run one action.
#[derive(Debug, Clone)]
pub struct Job {
    pub action: AiAction,
    pub profile: AiProfile,
    /// The user's instructions for every action (may be empty).
    pub global_instructions: String,
    /// The repository's instructions for every action, and how they
    /// combine with the global ones (see [`Job::with_repo_instructions`]).
    pub repo_instructions: RepoAiInstructions,
    /// The workspace's instructions for every action and for this one (see
    /// [`Job::with_workspace`]); they sit between the global and the
    /// repository levels.
    pub workspace: WorkspaceAi,
    /// The user's instructions for this action, repository override
    /// applied (may be empty).
    pub action_instructions: String,
    /// `action_instructions` is the repository's override, which a
    /// workspace level never sits under.
    pub action_from_repo: bool,
    /// Loaded from `gitpoulpi-secrets`; `None` for a local agent.
    pub api_key: Option<String>,
    /// The English name of the language a ✦ Explain answer is written in
    /// ("French"), set by the caller from its UI language so this crate
    /// stays independent of it. `None` (the CLI, English) adds no line; other
    /// actions ignore it (see [`prompts::answer_language_line`]).
    pub answer_language: Option<String>,
    /// Values for the `${...}` variables of the instruction texts (see
    /// [`crate::vars`]); the default expands them to nothing.
    pub context: PromptContext,
    /// What is taken out of a request before it is sent, and what a frontend
    /// hooks to show or print it (see [`crate::privacy`]).
    pub privacy: Privacy,
    /// Most bytes of one merge conflict a "Suggest a resolution" may carry
    /// (`ai.limits.conflict_bytes`); `usize::MAX` once the user chose to send
    /// a large one anyway.
    pub conflict_limit: usize,
    /// The other input caps (`[ai.limits]`), whom to ask when an input is
    /// over one, and what was truncated (see [`crate::limits`]).
    pub limit: LimitPolicy,
    /// The user's own base prompt (`ai.hosting_reply.base_prompt`) for the
    /// two writing actions; `None` uses the built-in one. Every other
    /// action ignores it: their base prompts fix an answer format a parser
    /// relies on.
    pub base_override: Option<String>,
}

impl Job {
    /// Resolves `action` for a repository (whose override, if any, is
    /// `repo_override`) and loads the profile's key.
    ///
    /// Returns the reason as a user-facing string: it ends up in a toast
    /// or on stderr.
    pub fn prepare(
        config: &AiConfig,
        action: AiAction,
        repo_override: Option<&AiActionSettings>,
        tokens: &dyn TokenStore,
    ) -> std::result::Result<Self, String> {
        let resolved = config
            .resolve(action, repo_override)
            .map_err(|e: AiResolveError| e.to_string())?;
        let profile = resolved.profile;
        let api_key = if profile.kind.needs_key() && !profile.token_ref.is_empty() {
            tokens.load(&profile.token_ref).ok().flatten()
        } else {
            None
        };
        Ok(Self {
            action,
            profile,
            global_instructions: resolved.global_instructions,
            repo_instructions: RepoAiInstructions::default(),
            workspace: WorkspaceAi::default(),
            action_instructions: resolved.action_instructions,
            action_from_repo: resolved.action_from_repo,
            api_key,
            answer_language: None,
            context: PromptContext::default(),
            privacy: Privacy::from_config(
                &config.privacy,
                &gitpoulpi_core::privacy::default_sensitive_patterns(),
            ),
            conflict_limit: config.limits.conflict_bytes_clamped(),
            limit: LimitPolicy::new(Limits::from_config(&config.limits)),
            base_override: matches!(action, AiAction::HostingReply | AiAction::IssueBody)
                .then(|| config.hosting_reply.base_prompt.trim().to_string())
                .filter(|text| !text.is_empty()),
        })
    }

    /// Asks `gate` when an input is over a cap (`[ai.limits]`) instead of
    /// truncating it.
    pub fn with_limit_gate(mut self, gate: std::sync::Arc<dyn LimitGate>) -> Self {
        self.limit.gate = Some(gate);
        self
    }

    /// Sends every input whole, whatever the caps say (`--send-all`): for a
    /// frontend that cannot ask.
    pub fn with_send_all(mut self, send_all: bool) -> Self {
        self.limit.send_all = send_all;
        self
    }

    /// What this job truncated (or would have, when nothing could ask),
    /// shared with its clones: the `truncated` of the CLI/MCP answers.
    pub fn truncated(&self) -> Vec<Cut> {
        self.limit.truncated()
    }

    /// The byte budget of a diff of `bytes` bytes (`ai.limits.diff_bytes`),
    /// after asking about it when it is over (see [`LimitPolicy::decide`]).
    pub async fn diff_budget(&self, part: crate::limits::Part, bytes: usize) -> Result<usize> {
        self.limit.diff_budget(part, bytes).await
    }

    /// Sends a conflict over `ai.limits.conflict_bytes` as well: the user
    /// was warned and chose to. Everything else (redaction, excluded files)
    /// applies unchanged.
    pub fn without_conflict_limit(mut self) -> Self {
        self.conflict_limit = usize::MAX;
        self
    }

    /// Takes the privacy settings as they are now: `[ai.privacy]` and the
    /// patterns of `[privacy]` (a window's copy of the config can be older
    /// than the file).
    pub fn with_privacy(mut self, ai: &gitpoulpi_config::AiPrivacy, patterns: &[String]) -> Self {
        self.privacy.redact_secrets = ai.redact_secrets;
        self.privacy.exclude_sensitive_files = ai.exclude_sensitive_files;
        self.privacy.patterns = patterns.to_vec();
        self
    }

    /// Uses `[privacy] sensitive_patterns` instead of the built-in list.
    pub fn with_sensitive_patterns(mut self, patterns: &[String]) -> Self {
        self.privacy.patterns = patterns.to_vec();
        self
    }

    /// Records the prepared request and sends nothing (`--print-sent`): the
    /// run fails with [`Error::PrintOnly`] and [`Job::privacy_report`] holds
    /// what would have gone out.
    pub fn with_print_only(mut self, print_only: bool) -> Self {
        self.privacy.print_only = print_only;
        self
    }

    /// Asks `gate` before the first request of a run is sent.
    pub fn with_send_gate(mut self, gate: std::sync::Arc<dyn SendGate>) -> Self {
        self.privacy.gate = Some(gate);
        self
    }

    /// What this job's requests were cleaned of (shared with its clones).
    pub fn privacy_report(&self) -> std::sync::Arc<crate::privacy::Report> {
        self.privacy.report()
    }

    /// What [`Job::run`] would send for `user`, after cleaning: the text the
    /// "What is sent" dialog shows.
    pub fn prepared(&self, user: &str) -> Prepared {
        let job = self.with_files_counted(user);
        crate::privacy::prepare(&job.system_prompt(), user, &self.privacy)
    }

    fn with_files_counted(&self, user: &str) -> Job {
        // `${files}`: what the prompt's diff touches, unless the caller said.
        let mut job = self.clone();
        if job.context.files.is_empty() {
            let files = user.lines().filter(|l| l.starts_with("diff -- ")).count();
            job.context.files = files.to_string();
        }
        job
    }

    /// Adds the repository's instructions for every action (they come
    /// from its `repo-state.toml`, `preferences.ai_instructions`).
    pub fn with_repo_instructions(mut self, repo: RepoAiInstructions) -> Self {
        self.repo_instructions = repo;
        self
    }

    /// Adds the instructions of the workspace the repository is used in
    /// (`Workspaces::ai_for`): between the global and the repository levels.
    pub fn with_workspace(mut self, workspace: WorkspaceAi) -> Self {
        self.workspace = workspace;
        self
    }

    /// Asks for the answer in `language` (an English name such as "French")
    /// when this is an Explain job.
    pub fn with_answer_language(mut self, language: Option<String>) -> Self {
        self.answer_language = language;
        self
    }

    /// Sets the values of the prompt variables (the caller's UI language,
    /// the repository's facts); the action's name is filled in here.
    pub fn with_context(mut self, mut context: PromptContext) -> Self {
        context.action = self.action.key().to_string();
        self.context = context;
        self
    }

    /// Runs a local agent in `workdir` for this job instead of the profile's
    /// own (`[ai.review]`'s `run_in_repository`); `None` and HTTP profiles
    /// change nothing.
    pub fn with_workdir(mut self, workdir: Option<gitpoulpi_config::AgentWorkdir>) -> Self {
        if let Some(workdir) = workdir {
            if self.profile.kind == gitpoulpi_config::AiProviderKind::LocalAgent {
                self.profile.workdir = workdir;
            }
        }
        self
    }

    /// The agent works inside the repository's checkout: a local agent
    /// whose working folder is the repository, with the repository known.
    pub fn runs_in_repository(&self) -> bool {
        self.profile.kind == gitpoulpi_config::AiProviderKind::LocalAgent
            && self.profile.workdir == gitpoulpi_config::AgentWorkdir::Repository
            && self.repo_path().is_some()
    }

    /// The repository the job runs for, when known (a local agent's
    /// `repository` working folder).
    pub fn repo_path(&self) -> Option<std::path::PathBuf> {
        Some(self.context.repo_root.as_str())
            .filter(|p| !p.is_empty())
            .map(std::path::PathBuf::from)
    }

    /// The instruction levels that reach the model, most general first, as
    /// `(level, text)`: `global`, `workspace`, `repository`, then `task`
    /// (the action's own, with the workspace's per-action text folded in).
    /// A level in "replace" mode has already dropped the ones above it.
    /// `${...}` variables are expanded.
    pub fn prompt_layers(&self) -> Vec<(&'static str, String)> {
        let ctx = &self.context;
        let expand = |i: &RepoAiInstructions| RepoAiInstructions {
            text: vars::expand(&i.text, ctx),
            mode: i.mode,
        };
        let task = prompts::task_layers(
            &vars::expand(&self.action_instructions, ctx),
            &expand(&self.workspace.for_action(self.action)),
            self.action_from_repo,
        );
        let mut layers: Vec<(&'static str, String)> = prompts::general_layers(
            &vars::expand(&self.global_instructions, ctx),
            &expand(&self.workspace.general),
            &expand(&self.repo_instructions),
        )
        .into_iter()
        .map(|(heading, text)| {
            let level = match heading {
                "Global instructions" => "global",
                "Workspace instructions" => "workspace",
                _ => "repository",
            };
            (level, text)
        })
        .collect();
        if !task.trim().is_empty() {
            layers.push(("task", task.trim().to_string()));
        }
        layers
    }

    /// The base prompt this job starts from: the built-in one of its action,
    /// or the user's own for the writing actions (see
    /// [`Job::base_override`]).
    pub fn base_prompt(&self) -> String {
        match &self.base_override {
            Some(text) => vars::expand(text, &self.context),
            None => prompts::base_prompt(self.action).to_string(),
        }
    }

    /// The system prompt this job sends: the base prompt (see
    /// [`Job::base_prompt`]), then the
    /// instructions from the most general to the most specific - global,
    /// workspace, repository - then the action's own (see
    /// [`prompts::system_prompt_layered`]). A level in "replace" mode drops
    /// the levels above it.
    pub fn system_prompt(&self) -> String {
        let ctx = &self.context;
        let expand = |i: &RepoAiInstructions| RepoAiInstructions {
            text: vars::expand(&i.text, ctx),
            mode: i.mode,
        };
        let task = prompts::task_layers(
            &vars::expand(&self.action_instructions, ctx),
            &expand(&self.workspace.for_action(self.action)),
            self.action_from_repo,
        );
        let mut out = prompts::system_prompt_layered_with_base(
            &self.base_prompt(),
            &vars::expand(&self.global_instructions, ctx),
            &expand(&self.workspace.general),
            &expand(&self.repo_instructions),
            &task,
        );
        if let Some(line) =
            prompts::answer_language_line(self.action, self.answer_language.as_deref())
        {
            out.push_str("\n\n");
            out.push_str(&line);
        }
        out
    }

    /// Runs [`Job::system_prompt`] plus `user` against the profile,
    /// over the real network transport (or the agent subprocess).
    #[cfg(feature = "network")]
    pub async fn run(&self, user: &str) -> Result<String> {
        let client = crate::client_for_network_in(
            &self.profile,
            self.api_key.clone(),
            self.repo_path().as_deref(),
            &self.privacy,
        )?;
        self.run_with(&*client, user).await
    }

    /// [`Job::run`] over a caller-supplied client (tests).
    pub async fn run_with(&self, client: &dyn AiClient, user: &str) -> Result<String> {
        let job = self.with_files_counted(user);
        client.complete(&job.system_prompt(), user).await
    }

    /// Suggests the resolution of one conflict ([`AiAction::ConflictResolve`]
    /// job): builds the prompt, asks, parses. An answer the parser rejects
    /// is sent back once with what was wrong with it. Returns the resolved
    /// lines. Over the real network transport (or the agent subprocess).
    #[cfg(feature = "network")]
    pub async fn resolve_conflict(
        &self,
        conflict: &prompts::ConflictContext,
    ) -> Result<Vec<String>> {
        let client = crate::client_for_network_in(
            &self.profile,
            self.api_key.clone(),
            self.repo_path().as_deref(),
            &self.privacy,
        )?;
        self.resolve_conflict_with(&*client, conflict).await
    }

    /// [`Job::resolve_conflict`] over a caller-supplied client (tests).
    pub async fn resolve_conflict_with(
        &self,
        client: &dyn AiClient,
        conflict: &prompts::ConflictContext,
    ) -> Result<Vec<String>> {
        let prompt = prompts::conflict_resolve(conflict, self.conflict_limit)?;
        let answer = self.run_with(client, &prompt).await?;
        match prompts::conflict_resolution(&answer) {
            Ok(lines) => Ok(lines),
            Err(first) => {
                let retry = format!(
                    "{prompt}\n\nYour previous answer was rejected: {first}. Answer again with \
                     the resolved text in one code block only."
                );
                let answer = self.run_with(client, &retry).await?;
                prompts::conflict_resolution(&answer)
            }
        }
    }

    /// Suggests review comments for a pull request ([`AiAction::CodeReview`]
    /// job): builds the prompt, asks, parses. `files` is the request's whole
    /// diff, which is also what suggestions are placed on. Over the real
    /// network transport (or the agent subprocess).
    #[cfg(feature = "network")]
    pub async fn suggest_review(&self, input: &prompts::ReviewPrompt<'_>) -> Result<ReviewOutcome> {
        let client = crate::client_for_network_in(
            &self.profile,
            self.api_key.clone(),
            self.repo_path().as_deref(),
            &self.privacy,
        )?;
        self.suggest_review_with(&*client, input).await
    }

    /// [`Job::suggest_review`] over a caller-supplied client (tests).
    pub async fn suggest_review_with(
        &self,
        client: &dyn AiClient,
        input: &prompts::ReviewPrompt<'_>,
    ) -> Result<ReviewOutcome> {
        // The job knows where its agent runs, whatever the caller set.
        let input = &prompts::ReviewPrompt {
            in_repository: input.in_repository || self.runs_in_repository(),
            ..input.clone()
        };
        let built = prompts::code_review(input);
        self.review_outcome(client, input.files, built, Gathered::default())
            .await
    }

    /// [`Job::suggest_review`] that also reads the project context files
    /// (`ctx`, from `source`), so every input cap of a review (the context
    /// files, the description, the existing comments, each file's diff and
    /// the whole diff) is decided together: one question, one answer. The
    /// caps are the job's own (`[ai.limits]`); `input.context` and the
    /// budgets of `input` are ignored.
    #[cfg(feature = "network")]
    pub async fn suggest_review_with_context(
        &self,
        ctx: &EffectiveReviewContext,
        source: &dyn ContextSource,
        input: &prompts::ReviewPrompt<'_>,
    ) -> Result<ReviewOutcome> {
        let client = crate::client_for_network_in(
            &self.profile,
            self.api_key.clone(),
            self.repo_path().as_deref(),
            &self.privacy,
        )?;
        self.suggest_review_with_context_with(&*client, ctx, source, input)
            .await
    }

    /// [`Job::suggest_review_with_context`] over a caller-supplied client.
    pub async fn suggest_review_with_context_with(
        &self,
        client: &dyn AiClient,
        ctx: &EffectiveReviewContext,
        source: &dyn ContextSource,
        input: &prompts::ReviewPrompt<'_>,
    ) -> Result<ReviewOutcome> {
        let build = |gathered: &Gathered, l: &Limits| {
            prompts::code_review(&prompts::ReviewPrompt {
                context: &gathered.files,
                in_repository: input.in_repository || self.runs_in_repository(),
                budget: l.diff,
                file_budget: l.review_file,
                text_budget: l.review_text,
                threads_budget: l.review_threads,
                ..input.clone()
            })
        };
        let l = self.limit.limits;
        let mut gathered =
            crate::review_context::gather_within(ctx, source, l.context_file, l.context_total)
                .await;
        let mut built = build(&gathered, &l);
        let mut found = gathered.cuts.clone();
        found.extend(built.cuts.iter().cloned());
        if self.limit.decide(found).await? {
            let none = Limits::unlimited();
            gathered = crate::review_context::gather_within(
                ctx,
                source,
                none.context_file,
                none.context_total,
            )
            .await;
            built = build(&gathered, &none);
        }
        self.review_outcome(client, input.files, built, gathered)
            .await
    }

    async fn review_outcome(
        &self,
        client: &dyn AiClient,
        files: &[FileDiff],
        built: prompts::BuiltReviewPrompt,
        gathered: Gathered,
    ) -> Result<ReviewOutcome> {
        let answer = self.run_with(client, &built.text).await?;
        if answer.trim().is_empty() {
            return Err(Error::Empty);
        }
        let index = crate::review::DiffIndex::new(files);
        Ok(ReviewOutcome {
            suggestions: crate::review::parse_suggestions(&answer, &index),
            cut: built.cut,
            left_out: built.left_out,
            context_problems: gathered.problems,
            context_given: gathered.files.iter().map(|f| f.label.clone()).collect(),
            context_cut: gathered
                .files
                .iter()
                .filter(|f| f.cut)
                .map(|f| f.label.clone())
                .collect(),
            truncated: self.truncated(),
        })
    }

    /// Proposes how to split one commit ([`AiAction::SplitCommit`] job):
    /// numbers the pieces, asks, parses and checks the answer with
    /// `validate_split`. An answer that is rejected is sent back once with
    /// what was wrong with it. Returns the parts, oldest first, ready for
    /// the split editor or `SplitCommit::parts`. Over the real network
    /// transport (or the agent subprocess).
    #[cfg(feature = "network")]
    pub async fn plan_split(&self, source: &SplitSource) -> Result<Vec<SplitPart>> {
        let client = crate::client_for_network_in(
            &self.profile,
            self.api_key.clone(),
            self.repo_path().as_deref(),
            &self.privacy,
        )?;
        self.plan_split_with(&*client, source).await
    }

    /// [`Job::plan_split`] over a caller-supplied client (tests).
    pub async fn plan_split_with(
        &self,
        client: &dyn AiClient,
        source: &SplitSource,
    ) -> Result<Vec<SplitPart>> {
        let (pieces, ids) = split_pieces(source);
        let hunk_bytes: usize = pieces
            .iter()
            .filter_map(|p| p.diff.as_ref())
            .map(String::len)
            .sum();
        let budget = self.diff_budget(Part::SplitHunks, hunk_bytes).await?;
        let prompt = prompts::split_commit(&source.message, &pieces, budget);
        let id_names: Vec<String> = ids.iter().map(|(id, _)| id.clone()).collect();
        let check = |answer: &str| -> Result<Vec<SplitPart>> {
            let planned = crate::parse::split_plan(answer, &id_names)?;
            let parts = to_parts(&planned, &ids);
            validate_split(source, &parts)
                .map_err(|r| Error::Invalid(format!("the plan is not valid: {r:?}")))?;
            Ok(parts)
        };
        let answer = self.run_with(client, &prompt).await?;
        match check(&answer) {
            Ok(parts) => Ok(parts),
            Err(first) => {
                let retry = format!(
                    "{prompt}\n\nYour previous answer was rejected: {first}. Answer again with \
                     the JSON array only."
                );
                let answer = self.run_with(client, &retry).await?;
                check(&answer)
            }
        }
    }

    /// A one-line description for a status line: "Commit message via
    /// Claude Code".
    pub fn describe(&self) -> String {
        format!(
            "{} via {}",
            self.action.label(),
            self.profile.display_name()
        )
    }
}

/// What a review run produced, and what the model did not get to see.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ReviewOutcome {
    pub suggestions: crate::review::ReviewSuggestions,
    /// Files whose diff was shortened in the prompt.
    pub cut: Vec<String>,
    /// Files left out of the prompt (over the total budget).
    pub left_out: Vec<String>,
    /// What the project context files could not give (an unreadable extra
    /// file, one that no longer fit the total), when they were read by
    /// [`Job::suggest_review_with_context`].
    pub context_problems: Vec<String>,
    /// The context files given to the model, by label.
    pub context_given: Vec<String>,
    /// The context files that were shortened.
    pub context_cut: Vec<String>,
    /// Every input truncated for this run (the report of CLI/MCP; in the TUI
    /// the user chose it).
    pub truncated: Vec<Cut>,
}

/// The unified-diff-ish text a prompt carries. Not byte-for-byte `git
/// diff` (the backend hands hunks, not patches) but the same shape the
/// models are trained on.
pub fn render_diff(files: &[FileDiff]) -> String {
    let mut out = String::new();
    for file in files {
        let path = file.path.to_string_lossy().replace('\\', "/");
        let status = match file.status {
            FileStatus::Untracked => "new file (untracked)",
            FileStatus::Added => "new file",
            FileStatus::Deleted => "deleted file",
            FileStatus::Renamed => "renamed",
            FileStatus::Conflicted => "conflicted",
            FileStatus::Modified | FileStatus::Staged => "modified",
        };
        out.push_str(&format!("diff -- {path} ({status})\n"));
        for hunk in &file.hunks {
            out.push_str(&format!(
                "@@ -{},{} +{},{} @@\n",
                hunk.old_start, hunk.old_lines, hunk.new_start, hunk.new_lines
            ));
            for DiffLine {
                origin, content, ..
            } in &hunk.lines
            {
                let sign = match origin {
                    LineOrigin::Add => '+',
                    LineOrigin::Delete => '-',
                    LineOrigin::Context => ' ',
                };
                out.push(sign);
                out.push_str(content.trim_end_matches(['\n', '\r']));
                out.push('\n');
            }
        }
    }
    out
}

/// Numbers the pieces of a commit to split for the prompt: `H<n>` for each
/// hunk of a splittable text file, `F<n>` for each whole-only file, one
/// counter across files in file order. Returns the prompt pieces and, in the
/// same order, each id with the [`SplitPiece`] it stands for.
pub fn split_pieces(
    source: &SplitSource,
) -> (Vec<prompts::SplitPromptPiece>, Vec<(String, SplitPiece)>) {
    let mut prompt = Vec::new();
    let mut ids = Vec::new();
    let mut n = 0;
    for file in &source.files {
        let path = file.path.to_string_lossy().replace('\\', "/");
        if let Some(reason) = file.whole_only {
            n += 1;
            let id = format!("F{n}");
            let why = match reason {
                WholeReason::Binary => "binary",
                WholeReason::ModeOnly => "mode change",
                WholeReason::Added => "added",
                WholeReason::Deleted => "deleted",
                WholeReason::Submodule => "submodule",
                WholeReason::Lfs => "LFS",
            };
            prompt.push(prompts::SplitPromptPiece {
                id: id.clone(),
                header: format!("{path} ({why}, whole file)"),
                diff: None,
            });
            ids.push((
                id,
                SplitPiece::File {
                    path: file.path.clone(),
                },
            ));
            continue;
        }
        let total = file.hunks.len();
        for (i, hunk) in file.hunks.iter().enumerate() {
            n += 1;
            let id = format!("H{n}");
            let mut body = String::new();
            for DiffLine {
                origin, content, ..
            } in &hunk.lines
            {
                body.push(match origin {
                    LineOrigin::Add => '+',
                    LineOrigin::Delete => '-',
                    LineOrigin::Context => ' ',
                });
                body.push_str(content.trim_end_matches(['\n', '\r']));
                body.push('\n');
            }
            prompt.push(prompts::SplitPromptPiece {
                id: id.clone(),
                header: format!(
                    "{path} hunk {}/{total} @@ -{},{} +{},{} @@",
                    i + 1,
                    hunk.old_start,
                    hunk.old_lines,
                    hunk.new_start,
                    hunk.new_lines
                ),
                diff: Some(body),
            });
            ids.push((
                id,
                SplitPiece::Hunk {
                    path: file.path.clone(),
                    hunk: hunk.id,
                },
            ));
        }
    }
    (prompt, ids)
}

/// Turns a parsed split plan into parts, resolving each piece id through
/// `ids` (from [`split_pieces`]). An id not in `ids` is skipped; the parser
/// has already rejected those.
pub fn to_parts(planned: &[PlannedSplit], ids: &[(String, SplitPiece)]) -> Vec<SplitPart> {
    planned
        .iter()
        .map(|commit| SplitPart {
            message: commit.message.trim().to_string(),
            pieces: commit
                .pieces
                .iter()
                .filter_map(|id| ids.iter().find(|(k, _)| k == id).map(|(_, p)| p.clone()))
                .collect(),
        })
        .collect()
}

/// What a branch adds over its target: the subjects of the commits of
/// `source` (newest first, as the log walks them) up to the first one that
/// `target_ids` already holds, and that first one - the merge base as far
/// as the walked history can tell (`None` when the walk never met the
/// target: the range is then unknown).
pub fn branch_range(
    source: &[CommitNode],
    target_ids: &HashSet<Oid>,
) -> (Vec<String>, Option<Oid>) {
    let mut subjects = Vec::new();
    for node in source {
        if target_ids.contains(&node.id) {
            return (subjects, Some(node.id));
        }
        subjects.push(node.summary.clone());
    }
    (subjects, None)
}

/// The paths a diff touches, `/`-separated.
pub fn changed_paths(files: &[FileDiff]) -> Vec<String> {
    files
        .iter()
        .map(|f| f.path.to_string_lossy().replace('\\', "/"))
        .collect()
}

/// Turns a client-side failure into the sentence a toast shows.
pub fn describe_error(error: &Error) -> String {
    error.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitpoulpi_core::domain::{Hunk, HunkId};
    use std::path::PathBuf;

    fn file(path: &str, lines: &[(LineOrigin, &str)]) -> FileDiff {
        FileDiff {
            path: PathBuf::from(path),
            status: FileStatus::Modified,
            hunks: vec![Hunk {
                id: HunkId(0),
                old_start: 1,
                old_lines: 1,
                new_start: 1,
                new_lines: 1,
                lines: lines
                    .iter()
                    .map(|(o, c)| DiffLine {
                        origin: o.clone(),
                        content: (*c).to_string(),
                        staged: false,
                    })
                    .collect(),
            }],
        }
    }

    #[test]
    fn a_diff_renders_with_signs_and_slash_paths() {
        let diff = file(
            "src\\a.rs",
            &[(LineOrigin::Delete, "old\n"), (LineOrigin::Add, "new")],
        );
        let text = render_diff(std::slice::from_ref(&diff));
        assert!(text.contains("diff -- src/a.rs (modified)"), "{text}");
        assert!(text.contains("@@ -1,1 +1,1 @@\n-old\n+new\n"), "{text}");
        assert_eq!(changed_paths(&[diff]), ["src/a.rs"]);
    }

    struct Fixed(Option<String>);
    impl TokenStore for Fixed {
        fn store(&self, _: &str, _: &str) -> gitpoulpi_secrets::Result<()> {
            Ok(())
        }
        fn load(&self, _: &str) -> gitpoulpi_secrets::Result<Option<String>> {
            Ok(self.0.clone())
        }
        fn delete(&self, _: &str) -> gitpoulpi_secrets::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn prepare_loads_the_key_only_for_profiles_that_need_one() {
        use gitpoulpi_config::AiProviderKind;
        let mut config = AiConfig {
            enabled: true,
            ..AiConfig::default()
        };
        config
            .profiles
            .push(AiProfile::new("gpt", "GPT", AiProviderKind::OpenAi));
        let job = Job::prepare(
            &config,
            AiAction::CommitMessage,
            None,
            &Fixed(Some("sk-1".into())),
        )
        .unwrap();
        assert_eq!(job.api_key.as_deref(), Some("sk-1"));
        config.profiles[0] = AiProfile::new("cc", "CC", AiProviderKind::LocalAgent);
        let job = Job::prepare(
            &config,
            AiAction::CommitMessage,
            None,
            &Fixed(Some("sk-1".into())),
        )
        .unwrap();
        assert_eq!(job.api_key, None);
        assert_eq!(job.describe(), "Commit message via CC");
    }

    #[test]
    fn a_branch_range_stops_at_the_first_commit_the_target_holds() {
        let node = |n: u8, summary: &str| CommitNode {
            id: Oid::from_hex(&format!("{n:02x}").repeat(20)).unwrap(),
            parents: Vec::new(),
            summary: summary.to_string(),
            message: summary.to_string(),
            author_name: String::new(),
            author_email: String::new(),
            timestamp: 0,
            refs: Vec::new(),
        };
        let source = [node(3, "third"), node(2, "second"), node(1, "shared")];
        let target: HashSet<Oid> = [source[2].id].into_iter().collect();
        let (subjects, base) = branch_range(&source, &target);
        assert_eq!(subjects, ["third", "second"]);
        assert_eq!(base, Some(source[2].id));
        let (subjects, base) = branch_range(&source, &HashSet::new());
        assert_eq!(subjects.len(), 3);
        assert_eq!(base, None);
    }

    #[test]
    fn a_job_sends_base_global_and_action_instructions_with_the_repo_override() {
        use std::sync::Mutex;
        struct Spy(Mutex<Vec<String>>);
        #[async_trait::async_trait]
        impl AiClient for Spy {
            async fn complete(&self, system: &str, _user: &str) -> Result<String> {
                self.0.lock().unwrap().push(system.to_string());
                Ok("ok".into())
            }
        }
        let mut config = AiConfig {
            enabled: true,
            global_instructions: "Be kind.".into(),
            ..AiConfig::default()
        };
        config.actions.commit_message.instructions = "Use gitmoji.".into();
        let spy = Spy(Mutex::new(Vec::new()));
        let run = |repo: Option<&AiActionSettings>| {
            let job = Job::prepare(&config, AiAction::CommitMessage, repo, &Fixed(None)).unwrap();
            tokio::runtime::Builder::new_current_thread()
                .build()
                .unwrap()
                .block_on(job.run_with(&spy, "diff"))
        };
        run(None).unwrap();
        let repo = AiActionSettings {
            profile: String::new(),
            instructions: "Use tickets.".into(),
        };
        run(Some(&repo)).unwrap();
        let sent = spy.0.lock().unwrap().clone();
        let base = prompts::base_prompt(AiAction::CommitMessage);
        assert_eq!(
            sent[0],
            format!(
                "{base}\n\n## Global instructions\nBe kind.\n\n## Instructions for this task\nUse gitmoji."
            )
        );
        assert!(sent[1].ends_with("Use tickets."), "{}", sent[1]);
        assert!(!sent[1].contains("gitmoji"), "{}", sent[1]);
    }

    #[test]
    fn a_job_applies_the_repo_instructions_to_every_action() {
        use gitpoulpi_config::RepoInstructionsMode;
        let config = AiConfig {
            enabled: true,
            global_instructions: "Be kind.".into(),
            ..AiConfig::default()
        };
        for action in AiAction::ALL {
            let repo = |mode| RepoAiInstructions {
                text: "Repo rule.".into(),
                mode,
            };
            let job = Job::prepare(&config, action, None, &Fixed(None)).unwrap();
            let add = job
                .clone()
                .with_repo_instructions(repo(RepoInstructionsMode::Add))
                .system_prompt();
            assert!(add.contains("Be kind.") && add.contains("Repo rule."));
            let replace = job
                .with_repo_instructions(repo(RepoInstructionsMode::Replace))
                .system_prompt();
            assert!(!replace.contains("Be kind.") && replace.contains("Repo rule."));
        }
    }

    #[test]
    fn a_job_layers_global_workspace_repository_and_task_for_every_action() {
        use gitpoulpi_config::{RepoInstructionsMode as M, WorkspaceAi};
        let mut config = AiConfig {
            enabled: true,
            global_instructions: "GLOBAL".into(),
            ..AiConfig::default()
        };
        for action in AiAction::ALL {
            config.actions.get_mut(action).instructions = "GTASK".into();
        }
        let ins = |text: &str, mode| RepoAiInstructions {
            text: text.into(),
            mode,
        };
        for action in AiAction::ALL {
            let ws = |mode, task_mode| WorkspaceAi {
                general: ins("WSALL", mode),
                actions: [(action.key().to_string(), ins("WSTASK", task_mode))].into(),
            };
            let job = |repo: Option<&AiActionSettings>| {
                Job::prepare(&config, action, repo, &Fixed(None)).unwrap()
            };
            // Add everywhere: base, global, workspace, repository, task.
            let all = job(None)
                .with_workspace(ws(M::Add, M::Add))
                .with_repo_instructions(ins("REPO", M::Add))
                .system_prompt();
            let at = |t: &str| all.find(t).unwrap_or_else(|| panic!("{t} in {all}"));
            assert!(
                at("GLOBAL") < at("WSALL") && at("WSALL") < at("REPO"),
                "{all}"
            );
            assert!(
                at("REPO") < at("GTASK") && at("GTASK") < at("WSTASK"),
                "{all}"
            );
            // A replacing workspace drops the global text, and the global task text.
            let rep = job(None)
                .with_workspace(ws(M::Replace, M::Replace))
                .with_repo_instructions(ins("REPO", M::Add))
                .system_prompt();
            assert!(!rep.contains("GLOBAL") && !rep.contains("GTASK"), "{rep}");
            assert!(rep.contains("WSALL") && rep.contains("REPO") && rep.contains("WSTASK"));
            // A replacing repository drops the workspace's general text too.
            let repl = job(None)
                .with_workspace(ws(M::Add, M::Add))
                .with_repo_instructions(ins("REPO", M::Replace))
                .system_prompt();
            assert!(
                !repl.contains("GLOBAL") && !repl.contains("WSALL"),
                "{repl}"
            );
            // The repository's own action override is the last word on the task.
            let over = AiActionSettings {
                profile: String::new(),
                instructions: "RTASK".into(),
            };
            let by_repo = job(Some(&over))
                .with_workspace(ws(M::Add, M::Replace))
                .system_prompt();
            assert!(
                by_repo.contains("RTASK")
                    && !by_repo.contains("GTASK")
                    && !by_repo.contains("WSTASK"),
                "{by_repo}"
            );
            // No workspace, no change.
            assert_eq!(
                job(None).system_prompt(),
                job(None)
                    .with_workspace(WorkspaceAi::default())
                    .system_prompt()
            );
        }
    }

    #[test]
    fn a_review_job_asks_once_and_places_the_suggestions() {
        use gitpoulpi_core::domain::{DiffLine, FileStatus, Hunk, HunkId};
        let files = [FileDiff {
            path: "src/a.rs".into(),
            status: FileStatus::Modified,
            hunks: vec![Hunk {
                id: HunkId(0),
                old_start: 1,
                old_lines: 0,
                new_start: 1,
                new_lines: 1,
                lines: vec![DiffLine {
                    origin: LineOrigin::Add,
                    content: "let x = 1;\n".into(),
                    staged: false,
                }],
            }],
        }];
        let client = Script(
            vec![r#"{"summary":"fine","suggestions":[{"path":"src/a.rs","line":1,"severity":"warning","body":"unused"},{"path":"src/a.rs","line":99,"body":"elsewhere"}]}"#].into(),
            Default::default(),
        );
        let out = block_on(job_for(AiAction::CodeReview).suggest_review_with(
            &client,
            &prompts::ReviewPrompt {
                title: "T",
                files: &files,
                ..prompts::ReviewPrompt::default()
            },
        ))
        .unwrap();
        assert_eq!(out.suggestions.summary, "fine");
        assert_eq!(out.suggestions.suggestions.len(), 2);
        assert!(out.suggestions.suggestions[0].anchored);
        assert!(
            !out.suggestions.suggestions[1].anchored,
            "kept, as file-level"
        );
        let sent = client.1.lock().unwrap().clone();
        assert_eq!(sent.len(), 1);
        assert!(sent[0].contains("    1 +let x = 1;"), "{}", sent[0]);
        let empty = Script(vec!["  "].into(), Default::default());
        assert!(block_on(
            job_for(AiAction::CodeReview)
                .suggest_review_with(&empty, &prompts::ReviewPrompt::default())
        )
        .is_err());
    }

    fn agent_job(workdir: gitpoulpi_config::AgentWorkdir, repo: &str) -> Job {
        let mut job = job_for(AiAction::CodeReview);
        job.profile = AiProfile::new(
            "agent",
            "Agent",
            gitpoulpi_config::AiProviderKind::LocalAgent,
        );
        job.profile.workdir = workdir;
        job.with_context(PromptContext {
            repo_root: repo.into(),
            ..PromptContext::default()
        })
    }

    #[test]
    fn a_review_can_override_where_a_local_agent_runs_but_not_an_http_profile() {
        use gitpoulpi_config::AgentWorkdir as W;
        // The override replaces the profile's folder for this job only.
        let job = agent_job(W::Default, "/r").with_workdir(Some(W::Repository));
        assert_eq!(job.profile.workdir, W::Repository);
        assert!(job.runs_in_repository());
        // None keeps the profile's.
        let kept = agent_job(W::Dedicated, "/r").with_workdir(None);
        assert_eq!(kept.profile.workdir, W::Dedicated);
        assert!(!kept.runs_in_repository());
        // Without a repository there is nothing to run in.
        assert!(!agent_job(W::Repository, "").runs_in_repository());
        // An HTTP profile has no working folder to change.
        let http = job_for(AiAction::CodeReview).with_workdir(Some(W::Repository));
        assert_eq!(http.profile.workdir, W::Default);
        assert!(!http.runs_in_repository());
    }

    #[test]
    fn the_prompt_of_an_agent_in_the_repository_says_the_diff_is_authoritative() {
        use gitpoulpi_config::AgentWorkdir as W;
        let files = [FileDiff {
            path: "a.rs".into(),
            status: FileStatus::Modified,
            hunks: vec![],
        }];
        let ask = |job: Job| {
            let client = Script(
                vec![r#"{"summary":"ok","suggestions":[]}"#].into(),
                Default::default(),
            );
            block_on(job.suggest_review_with(
                &client,
                &prompts::ReviewPrompt {
                    title: "T",
                    files: &files,
                    ..prompts::ReviewPrompt::default()
                },
            ))
            .unwrap();
            let sent = client.1.lock().unwrap().clone();
            sent[0].clone()
        };
        assert!(ask(agent_job(W::Repository, "/r")).contains("authoritative"));
        assert!(!ask(agent_job(W::Dedicated, "/r")).contains("authoritative"));
        assert!(!ask(job_for(AiAction::CodeReview)).contains("authoritative"));
    }

    struct Script(
        std::sync::Mutex<Vec<&'static str>>,
        std::sync::Mutex<Vec<String>>,
    );
    #[async_trait::async_trait]
    impl AiClient for Script {
        async fn complete(&self, _system: &str, user: &str) -> Result<String> {
            self.1.lock().unwrap().push(user.to_string());
            Ok(self.0.lock().unwrap().remove(0).to_string())
        }
    }

    fn conflict() -> prompts::ConflictContext {
        prompts::ConflictContext {
            path: "a.rs".into(),
            index: 1,
            total: 1,
            base: vec![],
            has_base: true,
            ours: vec!["a".into()],
            theirs: vec!["b".into()],
            ours_label: String::new(),
            theirs_label: String::new(),
            before: vec![],
            after: vec![],
        }
    }

    fn job() -> Job {
        let config = AiConfig {
            enabled: true,
            ..AiConfig::default()
        };
        Job::prepare(&config, AiAction::ConflictResolve, None, &Fixed(None)).unwrap()
    }

    fn job_for(action: AiAction) -> Job {
        let config = AiConfig {
            enabled: true,
            ..AiConfig::default()
        };
        Job::prepare(&config, action, None, &Fixed(None)).unwrap()
    }

    #[test]
    fn an_explain_prompt_ends_with_the_answer_language_and_others_do_not() {
        let plain = job_for(AiAction::ExplainChanges).system_prompt();
        assert!(!plain.contains("Answer in"), "{plain}");
        let french = job_for(AiAction::ExplainChanges)
            .with_answer_language(Some("French".into()))
            .system_prompt();
        assert_eq!(french, format!("{plain}\n\nAnswer in French."));
        for action in [
            AiAction::CommitMessage,
            AiAction::StashMessage,
            AiAction::PullRequestDescription,
            AiAction::CommitComposer,
            AiAction::ConflictResolve,
            AiAction::SplitCommit,
        ] {
            let base = job_for(action).system_prompt();
            let with = job_for(action)
                .with_answer_language(Some("French".into()))
                .system_prompt();
            assert_eq!(base, with, "{action:?} ignores the UI language");
        }
    }

    fn block_on<T>(f: impl std::future::Future<Output = T>) -> T {
        tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap()
            .block_on(f)
    }

    #[test]
    fn a_conflict_resolution_is_parsed_from_the_answer() {
        let client = Script(vec!["```\nab\n```"].into(), Default::default());
        let lines = block_on(job().resolve_conflict_with(&client, &conflict())).unwrap();
        assert_eq!(lines, ["ab"]);
    }

    #[test]
    fn a_junk_answer_is_retried_once_with_the_reason() {
        let client = Script(
            vec!["Here is the fix:\nab", "```\nab\n```"].into(),
            Default::default(),
        );
        let lines = block_on(job().resolve_conflict_with(&client, &conflict())).unwrap();
        assert_eq!(lines, ["ab"]);
        let sent = client.1.lock().unwrap().clone();
        assert!(
            sent[1].contains("previous answer was rejected"),
            "{}",
            sent[1]
        );
        let client = Script(vec!["sorry, no", "sorry, no"].into(), Default::default());
        assert!(block_on(job().resolve_conflict_with(&client, &conflict())).is_err());
    }

    /// `ai.limits.conflict_bytes` reaches the job; over it nothing is sent
    /// and the error is the typed one the frontends turn into a warning;
    /// "send anyway" lifts it for that job only.
    #[test]
    fn a_conflict_over_the_configured_limit_is_held_back_until_sent_anyway() {
        let mut config = AiConfig {
            enabled: true,
            ..AiConfig::default()
        };
        config.limits.conflict_bytes = 1_000;
        let job = Job::prepare(&config, AiAction::ConflictResolve, None, &Fixed(None)).unwrap();
        assert_eq!(job.conflict_limit, 1_000);
        let mut big = conflict();
        big.ours = vec!["x".repeat(1_500)];
        let client = Script(vec!["```\nab\n```"].into(), Default::default());
        let err = block_on(job.resolve_conflict_with(&client, &big)).unwrap_err();
        assert!(
            matches!(err, Error::TooLarge { limit: 1_000, .. }),
            "{err:?}"
        );
        assert!(client.1.lock().unwrap().is_empty(), "nothing was sent");
        // The same job, past the warning: it goes out.
        let lines = block_on(
            job.clone()
                .without_conflict_limit()
                .resolve_conflict_with(&client, &big),
        )
        .unwrap();
        assert_eq!(lines, ["ab"]);
        assert_eq!(client.1.lock().unwrap().len(), 1);
        // The default is today's 40000.
        assert_eq!(self::job().conflict_limit, 40_000);
    }

    #[test]
    fn prepare_says_why_ai_cannot_run() {
        let config = AiConfig::default();
        let err = Job::prepare(&config, AiAction::CommitMessage, None, &Fixed(None)).unwrap_err();
        assert!(err.contains("turned off"), "{err}");
    }

    // -- the input caps (`[ai.limits]`) -----------------------------------------

    struct Answers(crate::limits::LimitAnswer, std::sync::Mutex<Vec<Vec<Cut>>>);
    #[async_trait::async_trait]
    impl LimitGate for Answers {
        async fn ask(&self, cuts: &[Cut]) -> crate::limits::LimitAnswer {
            self.1.lock().unwrap().push(cuts.to_vec());
            self.0
        }
    }

    struct Context;
    #[async_trait::async_trait]
    impl ContextSource for Context {
        async fn repo_file(&self, path: &str) -> std::result::Result<Option<Vec<u8>>, String> {
            Ok((path == "CLAUDE.md").then(|| "c".repeat(9_000).into_bytes()))
        }
        async fn link_file(&self, _: &str) -> std::result::Result<Option<Vec<u8>>, String> {
            Ok(None)
        }
    }

    fn review_files(lines: usize) -> Vec<FileDiff> {
        use gitpoulpi_core::domain::{DiffLine, Hunk, HunkId};
        vec![FileDiff {
            path: "src/big.rs".into(),
            status: FileStatus::Modified,
            hunks: vec![Hunk {
                id: HunkId(0),
                old_start: 1,
                old_lines: 0,
                new_start: 1,
                new_lines: lines as u32,
                lines: (0..lines)
                    .map(|i| DiffLine {
                        origin: LineOrigin::Add,
                        content: format!("let value_{i} = {i};\n"),
                        staged: false,
                    })
                    .collect(),
            }],
        }]
    }

    fn review_context() -> EffectiveReviewContext {
        EffectiveReviewContext {
            run_in_repository: gitpoulpi_config::RunInRepo::No,
            include_claude_md: true,
            include_agents_md: false,
            extra_file: String::new(),
        }
    }

    const REVIEW_ANSWER: &str = r#"{"summary":"ok","suggestions":[]}"#;

    /// Four inputs over their caps are decided together: one question that
    /// names each of them with its size, limit and setting.
    #[test]
    fn a_review_asks_once_about_every_input_over_its_cap() {
        let files = review_files(2_000);
        let description = "d".repeat(5_000);
        let gate = std::sync::Arc::new(Answers(
            crate::limits::LimitAnswer::SendTruncated,
            Default::default(),
        ));
        let client = Script(vec![REVIEW_ANSWER].into(), Default::default());
        let job = job_for(AiAction::CodeReview).with_limit_gate(gate.clone());
        let out = block_on(job.suggest_review_with_context_with(
            &client,
            &review_context(),
            &Context,
            &prompts::ReviewPrompt {
                title: "T",
                description: &description,
                files: &files,
                ..prompts::ReviewPrompt::default()
            },
        ))
        .unwrap();
        let asked = gate.1.lock().unwrap().clone();
        assert_eq!(asked.len(), 1, "one question for the whole request");
        let settings: Vec<&str> = asked[0].iter().map(|c| c.setting).collect();
        assert_eq!(
            settings,
            [
                "ai.limits.review_context_file_bytes",
                "ai.limits.review_text_bytes",
                "ai.limits.review_file_bytes"
            ]
        );
        assert_eq!(asked[0][0].detail, "CLAUDE.md");
        assert_eq!((asked[0][0].bytes, asked[0][0].limit), (9_000, 8_192));
        // Truncated: what was sent is today's cut, and the run says so.
        let sent = client.1.lock().unwrap()[0].clone();
        assert!(sent.contains("[file cut: showing"), "{sent}");
        assert!(sent.contains("[text cut]"), "{sent}");
        assert!(
            sent.contains("[CLAUDE.md cut: showing 8192 of 9000"),
            "{sent}"
        );
        assert_eq!(out.truncated, asked[0]);
        assert_eq!(out.context_cut, ["CLAUDE.md"]);
    }

    #[test]
    fn send_all_lifts_every_review_cap_and_cancel_sends_nothing() {
        let files = review_files(2_000);
        let description = "d".repeat(5_000);
        let run = |answer| {
            let gate = std::sync::Arc::new(Answers(answer, Default::default()));
            let client = Script(vec![REVIEW_ANSWER].into(), Default::default());
            let job = job_for(AiAction::CodeReview).with_limit_gate(gate);
            let out = block_on(job.suggest_review_with_context_with(
                &client,
                &review_context(),
                &Context,
                &prompts::ReviewPrompt {
                    title: "T",
                    description: &description,
                    files: &files,
                    ..prompts::ReviewPrompt::default()
                },
            ));
            let sent = client.1.lock().unwrap().clone();
            (out, sent)
        };
        let (out, sent) = run(crate::limits::LimitAnswer::SendAll);
        let out = out.unwrap();
        let sent = &sent[0];
        assert!(!sent.contains("cut"), "nothing is cut: {sent}");
        assert!(sent.contains("value_1999"), "the whole file");
        assert!(sent.contains(&"d".repeat(5_000)), "the whole description");
        assert!(sent.contains(&"c".repeat(9_000)), "the whole context file");
        assert!(out.truncated.is_empty() && out.context_cut.is_empty());

        let (out, sent) = run(crate::limits::LimitAnswer::Cancel);
        assert_eq!(out.unwrap_err(), Error::Declined);
        assert!(sent.is_empty(), "nothing is sent after Cancel");
    }

    /// Nobody to ask (CLI, MCP): today's truncation, reported; `send_all`
    /// sends everything.
    #[test]
    fn without_a_gate_a_review_truncates_and_reports_or_sends_all() {
        let files = review_files(2_000);
        let ask = |job: Job| {
            let client = Script(vec![REVIEW_ANSWER].into(), Default::default());
            let out = block_on(job.suggest_review_with_context_with(
                &client,
                &review_context(),
                &Context,
                &prompts::ReviewPrompt {
                    title: "T",
                    files: &files,
                    ..prompts::ReviewPrompt::default()
                },
            ))
            .unwrap();
            let sent = client.1.lock().unwrap()[0].clone();
            (out, sent)
        };
        let (out, sent) = ask(job_for(AiAction::CodeReview));
        assert!(sent.contains("[file cut: showing"), "{sent}");
        let parts: Vec<crate::limits::Part> = out.truncated.iter().map(|c| c.part).collect();
        assert_eq!(
            parts,
            [
                crate::limits::Part::ContextFile,
                crate::limits::Part::ReviewFile
            ]
        );
        let (out, sent) = ask(job_for(AiAction::CodeReview).with_send_all(true));
        assert!(!sent.contains("[file cut"), "{sent}");
        assert!(out.truncated.is_empty());
    }

    /// The configured caps reach the job, and a raised one avoids the cut.
    #[test]
    fn the_configured_caps_reach_the_job() {
        let mut config = AiConfig {
            enabled: true,
            ..AiConfig::default()
        };
        config.limits.diff_bytes = 90_000;
        config.limits.review_file_bytes = 30_000;
        let job = Job::prepare(&config, AiAction::CodeReview, None, &Fixed(None)).unwrap();
        assert_eq!(job.limit.limits.diff, 90_000);
        assert_eq!(job.limit.limits.review_file, 30_000);
        assert_eq!(
            job_for(AiAction::CommitMessage).limit.limits.diff,
            60_000,
            "the default is today's value"
        );
        let files = review_files(800);
        let client = Script(vec![REVIEW_ANSWER].into(), Default::default());
        let out = block_on(job.suggest_review_with_context_with(
            &client,
            &review_context(),
            &Context,
            &prompts::ReviewPrompt {
                title: "T",
                files: &files,
                ..prompts::ReviewPrompt::default()
            },
        ))
        .unwrap();
        let sent = client.1.lock().unwrap()[0].clone();
        assert!(sent.contains("value_799"), "a raised cap keeps the file");
        // Only the context file (its own cap was not raised) is cut.
        assert_eq!(out.truncated.len(), 1);
    }

    /// A diff-style prompt: the budget is the cap, asked about when over.
    #[test]
    fn a_diff_over_the_cap_is_asked_about_before_it_is_cut() {
        let gate = std::sync::Arc::new(Answers(
            crate::limits::LimitAnswer::SendAll,
            Default::default(),
        ));
        let job = job_for(AiAction::CommitMessage).with_limit_gate(gate.clone());
        assert_eq!(
            block_on(job.diff_budget(Part::Diff, 60_000)),
            Ok(60_000),
            "at the cap: no question"
        );
        assert!(gate.1.lock().unwrap().is_empty());
        assert_eq!(
            block_on(job.diff_budget(Part::Diff, 60_001)),
            Ok(usize::MAX),
            "send all lifts the cap"
        );
        assert_eq!(
            gate.1.lock().unwrap()[0],
            [Cut::new(Part::Diff, "", 60_001, 60_000)]
        );
    }
}

#[cfg(test)]
mod vars_tests {
    use super::*;
    use crate::vars::PromptContext;
    use gitpoulpi_config::{AiConfig, RepoInstructionsMode};
    use gitpoulpi_secrets::InMemoryTokens;

    #[test]
    fn every_instruction_text_is_expanded_in_the_system_prompt() {
        let mut config = AiConfig {
            enabled: true,
            global_instructions: "Global for ${repo}.".into(),
            ..AiConfig::default()
        };
        config.actions.commit_message.instructions = "Action ${action} by ${user}.".into();
        let job = Job::prepare(
            &config,
            AiAction::CommitMessage,
            None,
            &InMemoryTokens::default(),
        )
        .unwrap()
        .with_repo_instructions(RepoAiInstructions {
            text: "Repo ${branch} ${nope} $${repo}".into(),
            mode: RepoInstructionsMode::Add,
        })
        .with_context(PromptContext {
            repo: "poulpi".into(),
            user: "Ada".into(),
            branch: "main".into(),
            ..PromptContext::default()
        });
        let prompt = job.system_prompt();
        assert!(prompt.contains("Global for poulpi."), "{prompt}");
        assert!(prompt.contains("Action commit_message by Ada."), "{prompt}");
        assert!(prompt.contains("Repo main ${nope} ${repo}"), "{prompt}");
    }
}
