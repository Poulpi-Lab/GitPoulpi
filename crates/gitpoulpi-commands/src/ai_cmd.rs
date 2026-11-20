//! The AI commands: `gitpoulpi commit --ai`, `gitpoulpi ai commit-message`,
//! `gitpoulpi ai explain` and `gitpoulpi ai resolve-conflict`.
//!
//! The diff is read through the daemon like everything else; the model is
//! called from *this* process (the API key never travels over IPC), using
//! the profile and instructions of the action, with the repository's override.

use gitpoulpi_ai::assist::{changed_paths, render_diff, Job};
use gitpoulpi_ai::limits::Part;
use gitpoulpi_ai::Cut;
use gitpoulpi_ai::{parse, prompts};
use gitpoulpi_config::AiAction;
use gitpoulpi_core::domain::{DiffTarget, FileDiff, MergeDocument, Oid, OperationKind};
use gitpoulpi_secrets::CredentialHelperTokens;
use serde_json::json;

use crate::cli::AiCommand;
use crate::client::Session;
use crate::commands::{emitted, Emitted};

/// What one `ai` run knows about its privacy: whether it only prints what
/// would be sent (`--print-sent`) and the report of the job it ran, which
/// becomes the `privacy` object of the answer.
#[derive(Default)]
pub(crate) struct Ctx {
    print_sent: bool,
    /// `--send-all`: nothing is truncated.
    send_all: bool,
    report: Option<std::sync::Arc<gitpoulpi_ai::privacy::Report>>,
    /// The job's cap decisions, to report what it truncated.
    limit: Option<gitpoulpi_ai::LimitPolicy>,
}

/// Adds `truncated: [{part, detail?, bytes, limit, setting}]` to `out` (only
/// when something was cut) and a line to the human text.
pub(crate) fn report_truncated(out: &mut serde_json::Value, human: &mut String, truncated: &[Cut]) {
    if truncated.is_empty() {
        return;
    }
    if let Some(map) = out.as_object_mut() {
        map.insert("truncated".into(), json!(truncated));
    }
    for cut in truncated {
        human.push_str(&format!(
            "\ntruncated: {}{} is {} bytes, over the {} byte limit (raise {} or pass --send-all)",
            cut_name(cut.part),
            if cut.detail.is_empty() {
                String::new()
            } else {
                format!(" ({})", cut.detail)
            },
            cut.bytes,
            cut.limit,
            cut.setting
        ));
    }
}

/// Where the CLI keeps its AI caches (the installations design, 1.5): a
/// cache, so one folder per environment (`<private>/ai/<env>/`) and two
/// installations never race on a file or trip on each other's format. The
/// flat layout keeps `<private>` itself.
fn ai_cache_dir(private: &std::path::Path, env: Option<&str>) -> std::path::PathBuf {
    match env {
        Some(env) => private.join("ai").join(env),
        None => private.to_path_buf(),
    }
}

fn cut_name(part: gitpoulpi_ai::Part) -> &'static str {
    use gitpoulpi_ai::Part;
    match part {
        Part::Diff => "the diff",
        Part::SplitHunks => "the commit's hunks",
        Part::ReviewDiff => "the review's diff",
        Part::ReviewFile => "a file's diff",
        Part::ReviewText => "the description",
        Part::ReviewThreads => "the existing comments",
        Part::ContextFile => "a context file",
        Part::ContextTotal => "the context files",
        Part::ReplyConversation => "the conversation",
    }
}

impl Ctx {
    /// [`job_for`], tracked by this run.
    fn job(&mut self, session: &Session, action: AiAction) -> Result<Job, String> {
        let job = job_for(session, action)?
            .with_print_only(self.print_sent)
            .with_send_all(self.send_all);
        self.report = Some(job.privacy_report());
        self.limit = Some(job.limit.clone());
        Ok(job)
    }

    /// The answer with its `privacy` object, or, for `--print-sent`, the
    /// prepared text instead of the provider's answer.
    fn finish(self, result: Result<Emitted, String>) -> Result<Emitted, String> {
        let summary = self.report.as_ref().map(|r| r.summary());
        let privacy = summary
            .map(|s| json!({ "redactions": s.redactions, "excluded_files": s.excluded_files }));
        match result {
            Err(e)
                if self.print_sent && e.contains(&gitpoulpi_ai::Error::PrintOnly.to_string()) =>
            {
                let sent = self.report.map(|r| r.sent()).unwrap_or_default();
                let human = sent
                    .iter()
                    .map(|(system, user)| format!("--- system ---\n{system}\n--- user ---\n{user}"))
                    .collect::<Vec<_>>()
                    .join("\n");
                let requests: Vec<serde_json::Value> = sent
                    .iter()
                    .map(|(system, user)| json!({ "system": system, "user": user }))
                    .collect();
                Ok(emitted(
                    json!({ "sent": requests, "privacy": privacy, "dry_run": true }),
                    human,
                ))
            }
            Ok(mut out) => {
                if let (Some(p), Some(map)) = (privacy, out.json.as_object_mut()) {
                    map.insert("privacy".into(), p);
                }
                if let Some(limit) = &self.limit {
                    // A command that reported its own (the review) has it already.
                    if out.json.get("truncated").is_none() {
                        report_truncated(&mut out.json, &mut out.human, &limit.truncated());
                    }
                }
                Ok(out)
            }
            other => other,
        }
    }
}

/// Resolves `action` for the session's repository into a runnable [`Job`].
pub(crate) fn job_for(session: &Session, action: AiAction) -> Result<Job, String> {
    job_for_root(session.repo_root(), action)
}

/// [`job_for`] for a repository root known to the caller, or none (a
/// command that names its project instead of running in a repository).
pub(crate) fn job_for_root(
    root: Option<&std::path::Path>,
    action: AiAction,
) -> Result<Job, String> {
    let settings = gitpoulpi_config::Settings::resolve_ambient().map_err(|e| e.to_string())?;
    let config = &settings.config().ai;
    let patterns = settings.config().privacy.sensitive_patterns.clone();
    let state = root
        .map(crate::commands::repo_state_path)
        .and_then(|p| gitpoulpi_session::RepoState::load(&p).ok());
    let repo_override = state
        .as_ref()
        .and_then(|s| s.preferences.ai_override(action.key(), config));
    let repo_instructions = state
        .map(|s| s.preferences.ai_instructions)
        .unwrap_or_default();
    #[cfg(feature = "workspaces")]
    let workspace = root.map(crate::workspace_cmd::ai_for).unwrap_or_default();
    #[cfg(not(feature = "workspaces"))]
    let workspace = Default::default();
    Job::prepare(
        config,
        action,
        repo_override.as_ref(),
        &CredentialHelperTokens::new(),
    )
    .map(|job| {
        let mut context = root
            .map(gitpoulpi_ai::vars::PromptContext::from_repo)
            .unwrap_or_default()
            .with_now();
        context.uilang = "English".to_string();
        job.with_repo_instructions(repo_instructions)
            .with_workspace(workspace)
            .with_context(context)
            .with_sensitive_patterns(&patterns)
    })
}

/// `ai prompt`: the layers of the system prompt `action` sends here. Needs no
/// model and no key, so it works with AI turned off.
fn prompt(session: &Session, action: &str) -> Result<Emitted, String> {
    let action = AiAction::from_key(action)
        .filter(|a| crate::build_features::ai_action_offered(*a))
        .ok_or_else(|| {
            format!(
                "unknown AI action `{action}` (one of: {})",
                AiAction::ALL
                    .into_iter()
                    .filter(|a| crate::build_features::ai_action_offered(*a))
                    .map(|a| a.key())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        })?;
    let settings = gitpoulpi_config::Settings::resolve_ambient().map_err(|e| e.to_string())?;
    let mut config = settings.config().ai.clone();
    let enabled = config.enabled;
    // The prompt is the same on or off; resolving needs it on.
    config.enabled = true;
    let state = session
        .repo_root()
        .map(crate::commands::repo_state_path)
        .and_then(|p| gitpoulpi_session::RepoState::load(&p).ok());
    let repo_override = state
        .as_ref()
        .and_then(|s| s.preferences.ai_override(action.key(), &config));
    let repo_instructions = state
        .map(|s| s.preferences.ai_instructions)
        .unwrap_or_default();
    #[cfg(feature = "workspaces")]
    let workspace = session
        .repo_root()
        .map(crate::workspace_cmd::ai_for)
        .unwrap_or_default();
    #[cfg(not(feature = "workspaces"))]
    let workspace = Default::default();
    let mut context = session
        .repo_root()
        .map(gitpoulpi_ai::vars::PromptContext::from_repo)
        .unwrap_or_default()
        .with_now();
    context.uilang = "English".to_string();
    let job = Job::prepare(
        &config,
        action,
        repo_override.as_ref(),
        &gitpoulpi_secrets::InMemoryTokens::default(),
    )?
    .with_repo_instructions(repo_instructions)
    .with_workspace(workspace)
    .with_context(context);
    let layers: Vec<serde_json::Value> = std::iter::once(("base", job.base_prompt()))
        .chain(job.prompt_layers())
        .map(|(level, text)| json!({ "level": level, "text": text }))
        .collect();
    let system = job.system_prompt();
    let mut out = json!({
        "action": action.key(),
        "ai_enabled": enabled,
        "layers": layers,
        "system_prompt": system,
    });
    let mut human = system.clone();
    if action == AiAction::CodeReview {
        // Not part of the system prompt: it joins the request, before the
        // diff, so it is its own entry, not a layer.
        let context = crate::ai_review_ctx::effective(
            session.repo_root(),
            None,
            &gitpoulpi_config::RepoAiReview::default(),
        )?;
        let text = context.describe();
        let mut shown = crate::ai_review_ctx::effective_json(&context);
        shown["text"] = json!(text);
        out["review_context"] = shown;
        human.push_str(&format!("\n\n[review_context] {text}"));
    }
    Ok(emitted(out, human))
}

/// The diff a commit message is generated from: what is staged, or, when
/// nothing is, everything that changed.
pub(crate) async fn diff_to_commit(session: &mut Session) -> Result<Vec<FileDiff>, String> {
    let staged: Vec<FileDiff> = session
        .call(
            OperationKind::Diff,
            json!({ "target": DiffTarget::IndexVsHead }),
        )
        .await?;
    if !staged.is_empty() {
        return Ok(staged);
    }
    session
        .call(
            OperationKind::Diff,
            json!({ "target": DiffTarget::WorkingTreeVsIndex }),
        )
        .await
}

/// Generates a commit message for the staged changes.
pub(crate) async fn generate_commit_message(
    session: &mut Session,
    send_all: bool,
) -> Result<(parse::CommitMessage, Vec<Cut>), String> {
    let job = job_for(session, AiAction::CommitMessage)?.with_send_all(send_all);
    let message = commit_message_with(session, &job).await?;
    Ok((message, job.truncated()))
}

async fn commit_message_with(
    session: &mut Session,
    job: &Job,
) -> Result<parse::CommitMessage, String> {
    let diffs = diff_to_commit(session).await?;
    if diffs.is_empty() {
        return Err("there are no changes to describe".into());
    }
    let diff = render_diff(&diffs);
    let budget = job
        .diff_budget(Part::Diff, diff.len())
        .await
        .map_err(|e| e.to_string())?;
    let prompt = prompts::commit_message(&diff, budget);
    let answer = job.run(&prompt).await.map_err(|e| e.to_string())?;
    parse::commit_message(&answer).map_err(|e| e.to_string())
}

/// Generates a one-line stash name for the working changes (staged and
/// unstaged together).
pub(crate) async fn generate_stash_message(
    session: &mut Session,
    send_all: bool,
) -> Result<(String, Vec<Cut>), String> {
    let job = job_for(session, AiAction::StashMessage)?.with_send_all(send_all);
    let staged: Vec<FileDiff> = session
        .call(
            OperationKind::Diff,
            json!({ "target": DiffTarget::IndexVsHead }),
        )
        .await?;
    let unstaged: Vec<FileDiff> = session
        .call(
            OperationKind::Diff,
            json!({ "target": DiffTarget::WorkingTreeVsIndex }),
        )
        .await?;
    let all: Vec<FileDiff> = staged.into_iter().chain(unstaged).collect();
    if all.is_empty() {
        return Err("there are no changes to stash".into());
    }
    let diff = render_diff(&all);
    let budget = job
        .diff_budget(Part::Diff, diff.len())
        .await
        .map_err(|e| e.to_string())?;
    let prompt = prompts::stash_message(&diff, budget);
    let answer = job.run(&prompt).await.map_err(|e| e.to_string())?;
    let message = parse::stash_message(&answer).map_err(|e| e.to_string())?;
    Ok((message, job.truncated()))
}

/// The AI's suggestion for conflict `conflict` (1-based) of `path`.
async fn resolve_conflict(
    session: &mut Session,
    ctx: &mut Ctx,
    path: &str,
    conflict: Option<usize>,
) -> Result<Emitted, String> {
    let doc: MergeDocument = session
        .call(OperationKind::ConflictShow, json!({ "path": path }))
        .await?;
    let total = doc.conflict_count();
    if total == 0 {
        return Err(format!(
            "{path} has no text conflict regions to suggest a resolution for"
        ));
    }
    let n = match conflict {
        Some(n) if (1..=total).contains(&n) => n,
        Some(n) => {
            return Err(format!(
                "there is no conflict {n} in {path} (it has {total})"
            ))
        }
        None if total == 1 => 1,
        None => {
            return Err(format!(
                "{path} has {total} conflicts: pass --conflict N (1 to {total})"
            ))
        }
    };
    let context = prompts::ConflictContext::from_document(&doc, n - 1)
        .ok_or_else(|| format!("there is no conflict {n} in {path}"))?;
    let job = ctx.job(session, AiAction::ConflictResolve)?;
    // A conflict is refused over its cap, never cut: `--send-all` sends it.
    let job = if ctx.send_all {
        job.without_conflict_limit()
    } else {
        job
    };
    let lines = job
        .resolve_conflict(&context)
        .await
        .map_err(|e| e.to_string())?;
    let text = lines.join("\n");
    Ok(emitted(
        json!({
            "path": path,
            "conflict": n,
            "conflicts": total,
            "resolution": lines,
            "applied": false,
        }),
        text,
    ))
}

/// Checks a script through the daemon, for the AI's retry.
#[cfg(feature = "history-rewrite")]
struct DaemonChecker<'a>(tokio::sync::Mutex<&'a mut Session>);

#[cfg(feature = "history-rewrite")]
#[async_trait::async_trait]
impl gitpoulpi_ai::assist::ScriptChecker for DaemonChecker<'_> {
    async fn check(&self, script: &str) -> Result<Vec<String>, String> {
        let check = check_script(&mut **self.0.lock().await, script).await?;
        Ok(check.problems)
    }
}

#[cfg(feature = "history-rewrite")]
async fn check_script(
    session: &mut Session,
    script: &str,
) -> Result<gitpoulpi_core::domain::rewrite::script::ScriptCheck, String> {
    use gitpoulpi_core::domain::rewrite::{ScriptLanguage, ScriptRef, ScriptSource};
    let args = gitpoulpi_ipc::rewrite::ScriptCheckArgs {
        script: ScriptRef {
            language: ScriptLanguage::Lua,
            source: ScriptSource::Inline {
                text: script.to_string(),
            },
            params: Default::default(),
        },
        vouched: Vec::new(),
    };
    session
        .call(
            OperationKind::RewriteScriptCheck,
            serde_json::to_value(args).map_err(|e| e.to_string())?,
        )
        .await
}

/// `ai rewrite-script`: the script the model writes, and the check of it.
#[cfg(feature = "history-rewrite")]
async fn rewrite_script(
    session: &mut Session,
    ctx: &mut Ctx,
    instruction: &str,
    sample: Option<usize>,
) -> Result<Emitted, String> {
    use gitpoulpi_ai::assist::RewriteScriptRequest;
    let job = ctx.job(session, AiAction::RewriteScript)?;
    let commits = match sample.filter(|n| *n > 0) {
        Some(n) => {
            let graph: gitpoulpi_core::domain::CommitGraph = session
                .call(
                    OperationKind::Log,
                    json!({ "opts": gitpoulpi_core::domain::LogOptions {
                        limit: Some(n.min(prompts::MAX_SAMPLE) as u32),
                        ..Default::default()
                    } }),
                )
                .await?;
            graph.nodes
        }
        None => Vec::new(),
    };
    let request = RewriteScriptRequest::new(
        instruction,
        &commits,
        sample.is_some_and(|n| n > 0),
        sample.unwrap_or(0),
    );
    let outcome = {
        let checker = DaemonChecker(tokio::sync::Mutex::new(&mut *session));
        job.write_rewrite_script(&request, &checker)
            .await
            .map_err(|e| e.to_string())?
    };
    // The full check (header, functions) for the answer.
    let check = check_script(session, &outcome.script).await?;
    let human = if check.ok() {
        outcome.script.clone()
    } else {
        format!(
            "{}\n-- check: {}",
            outcome.script,
            check.problems.join("; ")
        )
    };
    let mut answer = emitted(
        json!({
            "script": outcome.script,
            "check": check,
            "retried": outcome.retried,
            "sample": request.sample.len(),
            "applied": false,
        }),
        human,
    );
    if !check.ok() {
        answer.exit_code = 2;
    }
    Ok(answer)
}

/// `ai rewrite-rules`: the rules and settings the model proposes, as the
/// recipe JSON. Data only: this function has no path to a preview or an
/// apply, and says `applied: false` so a caller can rely on it.
#[cfg(feature = "history-rewrite")]
async fn rewrite_rules(
    session: &mut Session,
    ctx: &mut Ctx,
    description: &str,
    context: bool,
) -> Result<Emitted, String> {
    use gitpoulpi_ai::assist::RewriteRulesRequest;
    use gitpoulpi_core::domain::rewrite::{Recipe, Scope};
    use gitpoulpi_core::domain::{Ref, RefKind};
    let job = ctx.job(session, AiAction::RewriteRules)?;
    let mut request = RewriteRulesRequest {
        description: description.to_string(),
        context: prompts::RulesContext::default(),
    };
    if context {
        let refs: Vec<Ref> = session.call(OperationKind::Refs, json!({})).await?;
        let c = &mut request.context;
        c.scope = "the current branch".into();
        for r in &refs {
            match &r.kind {
                RefKind::LocalBranch { name } => {
                    if r.is_head {
                        c.current_branch = Some(name.clone());
                    }
                    c.refs.push(name.clone());
                }
                RefKind::Tag { name } => c.refs.push(name.clone()),
                RefKind::RemoteBranch { .. } => {}
            }
        }
        let graph: gitpoulpi_core::domain::CommitGraph = session
            .call(
                OperationKind::Log,
                json!({ "opts": gitpoulpi_core::domain::LogOptions {
                    limit: Some(200),
                    ..Default::default()
                } }),
            )
            .await?;
        for n in &graph.nodes {
            let who = (n.author_name.clone(), n.author_email.clone());
            if !c.authors.contains(&who) {
                c.authors.push(who);
            }
        }
    }
    let outcome = job
        .create_rewrite_rules(&request)
        .await
        .map_err(|e| e.to_string())?;
    let answer = &outcome.answer;
    let mut recipe = Recipe::new("AI proposal", Scope::default(), answer.rules.clone());
    if let Some(scope) = &answer.scope {
        recipe.scope = scope.clone();
    }
    if let Some(v) = answer.settings.include_tags {
        recipe.scope_options.include_tags = v;
    }
    if let Some(v) = answer.settings.committer {
        recipe.options.committer = v;
    }
    if let Some(v) = answer.settings.prune_empty {
        recipe.options.prune_empty = v;
    }
    if let Some(v) = answer.settings.update_message_hashes {
        recipe.options.update_message_hashes = v;
    }
    if let Some(v) = answer.settings.keep_backup_refs {
        recipe.options.keep_backup_refs = v;
    }
    let mut human = format!("{} rule(s) proposed", answer.rules.len());
    for p in &answer.problems {
        human.push_str(&format!("\nleft out ({}): {}", p.what, p.reason));
    }
    for n in &answer.notes {
        human.push_str(&format!("\nnote: {n}"));
    }
    let mut out = emitted(
        json!({
            "recipe": recipe,
            "problems": answer.problems,
            "notes": answer.notes,
            "retried": outcome.retried,
            "context": request.context != prompts::RulesContext::default(),
            "applied": false,
        }),
        human,
    );
    if !answer.problems.is_empty() {
        out.exit_code = 2;
    }
    Ok(out)
}

/// `ai worktree-name`: a worktree name and a branch name for a task.
#[cfg(feature = "worktree")]
async fn worktree_name(
    session: &mut Session,
    ctx: &mut Ctx,
    describe: &str,
) -> Result<Emitted, String> {
    use gitpoulpi_core::domain::{Ref, RefKind};
    if describe.trim().is_empty() {
        return Err("describe the task with --describe".into());
    }
    let job = ctx.job(session, AiAction::WorktreeName)?;
    let refs: Vec<Ref> = session.call(OperationKind::Refs, json!({})).await?;
    let branches: Vec<String> = refs
        .iter()
        .filter_map(|r| match &r.kind {
            RefKind::LocalBranch { name } => Some(name.clone()),
            _ => None,
        })
        .collect();
    let names = gitpoulpi_ai::worktree_name::suggest(&job, describe, &branches)
        .await
        .map_err(|e| e.to_string())?;
    let worktree = gitpoulpi_core::domain::worktree_name_after_branch(&names.name);
    Ok(emitted(
        json!({ "name": names.name, "worktree": worktree, "branch": names.branch }),
        format!("{worktree}  (branch {})", names.branch),
    ))
}

/// The conversation of `ai hosting-reply`: a file, `-` (standard input) or
/// the JSON itself.
#[cfg(feature = "hosting")]
fn read_conversation(
    source: &str,
) -> Result<gitpoulpi_hosting::conversation::TimelinePage, String> {
    use std::io::Read;
    let text = if source.trim_start().starts_with('{') {
        source.to_string()
    } else if source == "-" {
        let mut text = String::new();
        std::io::stdin()
            .read_to_string(&mut text)
            .map_err(|e| format!("could not read the conversation from standard input: {e}"))?;
        text
    } else {
        std::fs::read_to_string(source)
            .map_err(|e| format!("could not read the conversation {source}: {e}"))?
    };
    crate::ai_conversation::parse(&text)
}

/// `--item`: the conversation read from the host itself, through the same
/// path as `hosting issue comments` (an issue) or `hosting review
/// conversation` (a reference starting with `!` is a pull request).
#[cfg(feature = "hosting")]
async fn fetch_item(
    session: &mut Session,
    reference: &str,
) -> Result<gitpoulpi_hosting::conversation::TimelinePage, String> {
    let printed = crate::hosting_cmd::read_conversation_json(session, reference).await?;
    crate::ai_conversation::parse(&printed.to_string())
}

/// What `ai hosting-reply` was asked.
#[cfg(feature = "hosting")]
pub(crate) struct ReplyArgs {
    pub conversation: Option<String>,
    pub item: Option<String>,
    pub reply_to: Option<String>,
    pub path: Option<String>,
    pub line: Option<u32>,
    pub review_summary: bool,
    pub issue_title: Option<String>,
    pub hint: String,
    pub draft: Option<String>,
    pub flavor: Option<String>,
    pub read_repo: bool,
    pub continue_exchange: bool,
    pub refresh_context: bool,
    pub pr_base: Option<String>,
    pub pr_head: Option<String>,
}

/// `ai hosting-reply`: a draft for a conversation on a host. Drafts only:
/// nothing here reaches a hosting client, so nothing is posted.
#[cfg(feature = "hosting")]
async fn hosting_reply(
    session: &mut Session,
    ctx: &mut Ctx,
    args: ReplyArgs,
) -> Result<Emitted, String> {
    use gitpoulpi_ai::hosting_reply::{convert, draft, ReplyRequest, ReplyTarget};
    use gitpoulpi_hosting::conversation::MarkupFlavor;

    let target = match (&args.issue_title, &args.reply_to, &args.path) {
        (Some(title), ..) => ReplyTarget::IssueBody {
            title: title.trim().to_string(),
        },
        (_, Some(thread), _) => ReplyTarget::ThreadReply {
            thread: thread.clone(),
        },
        (_, _, Some(path)) => ReplyTarget::LineComment {
            path: path.clone(),
            line: args.line,
            hunk: None,
        },
        _ if args.review_summary => ReplyTarget::ReviewSummary,
        _ => ReplyTarget::Comment,
    };
    let mut request = ReplyRequest::new(target);
    request.hint = args.hint;
    request.draft = args.draft;
    let mut flavor = MarkupFlavor::default();
    let fetched = match &args.item {
        Some(reference) => Some(fetch_item(session, reference).await?),
        None => None,
    };
    let source_page = match (&fetched, &args.conversation) {
        (Some(page), _) => Some(page.clone()),
        (None, Some(source)) => Some(read_conversation(source)?),
        (None, None) => None,
    };
    if let Some(page) = source_page {
        flavor = page.header.body.flavor;
        request.item = Some(convert::item_digest(&page.header));
        request.entries = convert::entries(&page.entries);
    }
    if let Some(named) = &args.flavor {
        flavor = serde_json::from_value(json!(named.trim()))
            .map_err(|_| format!("unknown markup flavor `{named}`"))?;
    }
    request.flavor = convert::flavor_name(flavor).to_string();

    if let (Some(base), Some(head)) = (&args.pr_base, &args.pr_head) {
        request.diff = Some(gitpoulpi_ai::hosting_reply::DiffRange {
            base: base.clone(),
            head: head.clone(),
        });
    }

    // The exchange: what an earlier call for this same target asked and
    // answered, given back to a model that would not remember it.
    let store = gitpoulpi_config::ConfigPaths::discover().map(|paths| {
        let root = session
            .repo_root()
            .map(|r| r.display().to_string())
            .unwrap_or_default();
        let (thread, place) = match &request.target {
            ReplyTarget::ThreadReply { thread } => (thread.clone(), String::new()),
            ReplyTarget::LineComment { path, line, .. } => (
                String::new(),
                format!("{path}:{}", line.map(|l| l.to_string()).unwrap_or_default()),
            ),
            ReplyTarget::IssueBody { title } => (String::new(), title.clone()),
            _ => (String::new(), String::new()),
        };
        let key = gitpoulpi_ai::hosting_reply::exchange::key_of(&[
            &root,
            request.target.name(),
            &thread,
            &place,
            args.item.as_deref().unwrap_or_default(),
        ]);
        gitpoulpi_ai::hosting_reply::exchange::ExchangeStore::on_disk(
            &paths.private_dir().join("ai-exchange"),
            &key,
        )
    });
    let mut exchange = if args.continue_exchange {
        let store = store.as_ref().ok_or_else(|| {
            "--continue: could not determine this OS's data directory".to_string()
        })?;
        let earlier = store.load();
        if earlier.is_empty() {
            return Err("--continue: no earlier draft for this target to continue".into());
        }
        earlier
    } else {
        Vec::new()
    };
    request.exchange = exchange.clone();
    let (asked_hint, asked_draft) = (request.hint.clone(), request.draft.clone());

    let action = request.target.action();
    let job = ctx.job(session, action)?;
    let settings = gitpoulpi_config::Settings::resolve_ambient().map_err(|e| e.to_string())?;
    let repo_prefs = session
        .repo_root()
        .map(crate::commands::repo_state_path)
        .and_then(|p| gitpoulpi_session::RepoState::load(&p).ok())
        .map(|s| s.preferences.ai_hosting_reply)
        .unwrap_or_default();
    let reply_settings = settings.config().ai.hosting_reply.with_repo(&repo_prefs);
    // Off unless asked for: the flag, or the setting the user turned on.
    let read_repo = args.read_repo || reply_settings.read_repository;
    let result = if read_repo {
        let root = session
            .repo_root()
            .map(std::path::Path::to_path_buf)
            .ok_or_else(|| "--read-repo needs a repository".to_string())?;
        if let Some(why) = gitpoulpi_ai::hosting_reply::repo_access::blocked_reason(&job.profile) {
            return Err(format!("--read-repo: {why}"));
        }
        let paths = gitpoulpi_config::ConfigPaths::discover()
            .ok_or_else(|| "could not determine this OS's data directory".to_string())?;
        let private = ai_cache_dir(&paths.private_dir(), paths.env());
        let ttl = reply_settings.context_ttl_minutes_clamped();
        let mut cache = gitpoulpi_ai::hosting_reply::cache::ContextCache::on_disk(
            &private.join("ai-context"),
            &root,
            ttl,
        );
        let mut sessions = gitpoulpi_ai::hosting_reply::cache::SessionStore::on_disk(
            &private.join("ai-sessions.toml"),
        );
        let reader = crate::ai_repo::SessionReader::new(session);
        let mut setup = gitpoulpi_ai::hosting_reply::repo_access::RepoSetup {
            reader: &reader,
            root: &root,
            cache: &mut cache,
            sessions: &mut sessions,
            reuse_session: reply_settings.reuse_agent_session,
            refresh: args.refresh_context,
            limits: gitpoulpi_ai::hosting_reply::repo_context::ContextLimits::from_config(
                &settings.config().ai.limits,
            ),
            privacy: job.privacy.clone(),
        };
        gitpoulpi_ai::hosting_reply::repo_access::draft_reading(job, request.clone(), &mut setup)
            .await
            .map_err(|e| e.to_string())?
    } else {
        draft(&job, &request).await.map_err(|e| e.to_string())?
    };
    if let Some(store) = &store {
        gitpoulpi_ai::hosting_reply::exchange::push_capped(
            &mut exchange,
            gitpoulpi_ai::hosting_reply::Exchange {
                hint: asked_hint,
                draft: asked_draft,
                answer: result.text.clone(),
            },
        );
        store.save(&exchange);
    }
    Ok(emitted(
        json!({
            "action": action.key(),
            "target": request.target.name(),
            "draft": result.text,
            "improved": result.improved,
            "context": result.context,
            // Said on purpose: this command never posts, edits or deletes.
            "posted": false,
        }),
        result.text,
    ))
}

pub(crate) async fn run(
    session: &mut Session,
    action: AiCommand,
    print_sent: bool,
    send_all: bool,
) -> Result<Emitted, String> {
    let mut ctx = Ctx {
        print_sent,
        send_all,
        report: None,
        limit: None,
    };
    let result = run_action(session, &mut ctx, action).await;
    ctx.finish(result)
}

async fn run_action(
    session: &mut Session,
    ctx: &mut Ctx,
    action: AiCommand,
) -> Result<Emitted, String> {
    match action {
        AiCommand::CommitMessage => {
            let job = ctx.job(session, AiAction::CommitMessage)?;
            let message = commit_message_with(session, &job).await?;
            Ok(emitted(
                json!({ "summary": message.summary, "description": message.description }),
                message.full(),
            ))
        }
        AiCommand::Prompt { action } => prompt(session, &action),
        #[cfg(feature = "hosting")]
        AiCommand::HostingReply {
            conversation,
            item,
            reply_to,
            path,
            line,
            review_summary,
            issue_title,
            hint,
            draft,
            flavor,
            read_repo,
            continue_exchange,
            refresh_context,
            pr_base,
            pr_head,
        } => {
            hosting_reply(
                session,
                ctx,
                ReplyArgs {
                    conversation,
                    item,
                    reply_to,
                    path,
                    line,
                    review_summary,
                    issue_title,
                    hint,
                    draft,
                    flavor,
                    read_repo,
                    continue_exchange,
                    refresh_context,
                    pr_base,
                    pr_head,
                },
            )
            .await
        }
        #[cfg(feature = "worktree")]
        AiCommand::WorktreeName { describe } => worktree_name(session, ctx, &describe).await,
        // Answered from files, before a daemon is involved (`is_local`).
        AiCommand::ReviewContext { .. } => Err("`ai review-context` runs without a daemon".into()),
        AiCommand::Trust {
            profile,
            accept_origin,
            accept_certificate,
            allow_insecure_http,
        } => {
            // Trusting is a person's decision (the MCP server cannot).
            if !session.may_trust() {
                return Err(TRUST_REFUSAL.to_string());
            }
            trust(
                &profile,
                accept_origin.as_deref(),
                accept_certificate.as_deref(),
                allow_insecure_http.map(|o| o == crate::cli::OnOff::On),
            )
        }
        #[cfg(feature = "history-rewrite")]
        AiCommand::RewriteScript {
            instruction,
            sample,
        } => rewrite_script(session, ctx, &instruction, sample).await,
        #[cfg(feature = "history-rewrite")]
        AiCommand::RewriteRules {
            description,
            context,
        } => rewrite_rules(session, ctx, &description, context).await,
        AiCommand::ResolveConflict { path, conflict } => {
            resolve_conflict(session, ctx, &path, conflict).await
        }
        AiCommand::SplitPlan { rev } => {
            let oid: Oid = session
                .call(OperationKind::Resolve, json!({ "rev": rev }))
                .await?;
            let source: gitpoulpi_core::domain::SplitSource = session
                .call(OperationKind::SplitSource, json!({ "commit": oid }))
                .await?;
            let job = ctx.job(session, AiAction::SplitCommit)?;
            let parts = job.plan_split(&source).await.map_err(|e| e.to_string())?;
            let human = parts
                .iter()
                .enumerate()
                .map(|(i, p)| {
                    format!(
                        "{}. {} ({} piece(s))",
                        i + 1,
                        p.message.lines().next().unwrap_or(""),
                        p.pieces.len()
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");
            Ok(emitted(
                json!({ "commit": oid, "parts": parts, "applied": false }),
                human,
            ))
        }
        AiCommand::Explain { rev, wip } => {
            let job = ctx.job(session, AiAction::ExplainChanges)?;
            let (subject, message, diffs) = if wip {
                let staged = diff_to_commit(session).await?;
                ("the working tree changes".to_string(), None, staged)
            } else {
                let rev = rev.unwrap_or_else(|| "HEAD".to_string());
                let oid: Oid = session
                    .call(OperationKind::Resolve, json!({ "rev": rev }))
                    .await?;
                let diffs: Vec<FileDiff> = session
                    .call(
                        OperationKind::Diff,
                        json!({ "target": DiffTarget::CommitVsParent(oid) }),
                    )
                    .await?;
                (format!("the commit {rev}"), None::<String>, diffs)
            };
            if diffs.is_empty() {
                return Err("there is nothing to explain".into());
            }
            let diff = render_diff(&diffs);
            let budget = job
                .diff_budget(Part::Diff, diff.len())
                .await
                .map_err(|e| e.to_string())?;
            let prompt = prompts::explain_changes(&subject, message.as_deref(), &diff, budget);
            let answer = job.run(&prompt).await.map_err(|e| e.to_string())?;
            let files = changed_paths(&diffs);
            Ok(emitted(
                json!({ "explanation": answer.trim(), "files": files }),
                answer.trim().to_string(),
            ))
        }
    }
}

/// What an agent reads when a security check needs the person.
const TRUST_REFUSAL: &str =
    "This AI profile's security check needs the user: open GitPoulpi or run `gitpoulpi ai trust`";

/// `ai trust`: show or accept what binds a profile's API key to its address.
/// Each `--accept-*` names the exact value to accept.
pub(crate) fn trust(
    profile: &str,
    accept_origin: Option<&str>,
    accept_certificate: Option<&str>,
    allow_insecure_http: Option<bool>,
) -> Result<Emitted, String> {
    use gitpoulpi_hosting::trust::{origin, CertInfo, TlsMode, TlsRecord};
    let settings = gitpoulpi_config::Settings::resolve_ambient().map_err(|e| e.to_string())?;
    let found = settings
        .config()
        .ai
        .profile(profile)
        .cloned()
        .ok_or_else(|| format!("no AI profile `{profile}`"))?;
    if found.kind == gitpoulpi_config::AiProviderKind::LocalAgent {
        return Err(format!(
            "`{profile}` is a local agent: it has no address to trust"
        ));
    }
    let base = found.effective_base_url().to_string();
    let current = gitpoulpi_hosting::trust::normalize(&base)
        .ok_or_else(|| format!("`{base}` is not an address"))?;
    let gate = gitpoulpi_hosting::TrustGate::ambient();
    let err = |e: gitpoulpi_hosting::Error| e.to_string();
    let mut accepted: Vec<String> = Vec::new();
    if let Some(url) = accept_origin {
        let given = gitpoulpi_hosting::trust::normalize(url)
            .ok_or_else(|| format!("`{url}` is not an address"))?;
        if given != current {
            return Err(format!(
                "`{profile}` points to {current} now, not to {given}: nothing was accepted"
            ));
        }
        gate.trust_ai_origin(profile, &base).map_err(err)?;
        accepted.push(format!("address {current}"));
    }
    if let Some(on) = allow_insecure_http {
        gate.update_ai_record(profile, &base, |r| r.allow_plain_http = on)
            .map_err(err)?;
        accepted.push(format!(
            "plain http {}",
            if on { "allowed" } else { "refused" }
        ));
    }
    if let Some(fingerprint) = accept_certificate {
        let fp = fingerprint.trim().to_ascii_lowercase().replace(' ', "");
        if fp.len() != 64 || !fp.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(
                "--accept-certificate takes the SHA-256 fingerprint (64 hex digits)".into(),
            );
        }
        gate.update_ai_record(profile, &base, |r| {
            r.tls = Some(TlsRecord {
                mode: TlsMode::Tofu,
                cert: CertInfo {
                    spki_sha256: fp.clone(),
                    ..CertInfo::default()
                },
            });
        })
        .map_err(err)?;
        accepted.push(format!("certificate {fp}"));
    }
    let record = gate.ai_record_of(profile);
    let human = if accepted.is_empty() {
        match &record {
            Some(r) => format!("`{profile}` is trusted for {}", r.origin),
            None => format!("`{profile}` has no trust record yet; its key is bound on first use"),
        }
    } else {
        format!("`{profile}`: accepted {}", accepted.join(", "))
    };
    Ok(emitted(
        json!({
            "profile": profile,
            "accepted": accepted,
            "current_origin": current,
            "trusted": record.as_ref().map(|r| json!({
                "origin": r.origin,
                "allow_plain_http": r.allow_plain_http,
                "certificate": r.tls.as_ref().map(|t| t.cert.spki_sha256.clone()),
            })),
            "plain_http_remote": origin::is_plain_http_remote(&current),
        }),
        human,
    ))
}

#[cfg(test)]
mod env_cache_tests {
    use super::ai_cache_dir;
    use std::path::Path;

    #[test]
    fn the_ai_cache_is_per_env_and_flat_without_one() {
        let private = Path::new("/p");
        assert_eq!(ai_cache_dir(private, None), private);
        assert_eq!(ai_cache_dir(private, Some("a")), Path::new("/p/ai/a"));
        assert_ne!(
            ai_cache_dir(private, Some("a")),
            ai_cache_dir(private, Some("b"))
        );
    }
}
