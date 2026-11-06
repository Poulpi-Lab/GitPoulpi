//! `gitpoulpi hosting review show|threads|comment|reply|resolve|submit|fetch|diff|suggest` -- in-app code review of a pull/merge request.
//!
//! With `--project` (see [`super::review_api`]) every action but `fetch`,
//! `push` and `suggest`'s local diff works without a repository.
//!
//! The host half is `gitpoulpi_hosting::review` (called here, in the
//! client process, like every other hosting call); the one thing that
//! touches the repository, `review fetch`, goes through the daemon as
//! `OperationKind::FetchRefspec`.

use gitpoulpi_core::domain::OperationKind;
use gitpoulpi_hosting::review::{
    review_base_refspec, review_head_refspec, review_head_refspec_at, DiffSide, PendingComment,
    ReviewEvent, ReviewSubmission, ReviewVote,
};
use gitpoulpi_hosting::{review_base_ref, review_head_ref, HostingClient};
use serde_json::{json, Value};

use crate::cli::ReviewEventArg;
use crate::client::Session;
use crate::repo_cmd::Emitted;

use super::*;

fn client_for_target(
    env: &HostingEnv,
    target: &RepoTarget,
) -> Result<Box<dyn HostingClient>, String> {
    env.client(&target.integration, &target.integrations)
}

/// [`super::link`]'s target description, with `source: "project"` when the
/// target was named by `--project` instead of resolved from a repository.
pub(super) fn target_json(target: &RepoTarget) -> Value {
    let mut out = super::link::target_json(target);
    if super::review_api::is_project_target(target) {
        out["source"] = json!("project");
    }
    out
}

fn json_of<T: serde::Serialize>(value: &T) -> Result<Value, String> {
    serde_json::to_value(value).map_err(|e| format!("failed to encode output: {e}"))
}

/// `review show`: the request, its capabilities and its changed files.
pub(super) async fn show(
    env: &HostingEnv,
    target: &RepoTarget,
    number: u64,
    patch_set: Option<u32>,
    compare_with: Option<u32>,
) -> Result<Emitted, String> {
    let client = client_for_target(env, target)?;
    let detail = client
        .pull_request_detail_at(&target.project, number, patch_set)
        .await
        .map_err(|e| e.to_string())?;
    // The files of the patch set the detail describes (the newest by
    // default), as the difference from `compare_with` when it is given.
    let files = client
        .pull_request_files_against(
            &target.project,
            number,
            patch_set.or(detail.shown_patch_set),
            compare_with,
        )
        .await
        .map_err(|e| e.to_string())?;
    let mut human = format!(
        "{} {}  ({} -> {}, {})\n{}",
        detail.summary.iid_label,
        detail.summary.title,
        detail.summary.source_branch,
        detail.summary.target_branch,
        detail.summary.author,
        detail.summary.web_url
    );
    for f in &files {
        human.push_str(&format!(
            "\n  {:<9} +{} -{}  {}",
            format!("{:?}", f.status).to_lowercase(),
            f.additions,
            f.deletions,
            f.path
        ));
    }
    let mut out = merge(target_json(target), "number", json!(number));
    out["detail"] = json_of(&detail)?;
    out["files"] = json_of(&files)?;
    out["head_ref"] = json!(review_head_ref(number));
    out["base_ref"] = json!(review_base_ref(number));
    if let Some(k) = compare_with {
        out["compare_with"] = json!(k);
        human.push_str(&format!("\nfiles compared with patch set {k}"));
    }
    if !detail.patch_sets.is_empty() {
        human.push_str(&format!(
            "\npatch sets: {} (showing {})",
            detail
                .patch_sets
                .iter()
                .map(|p| p.number.to_string())
                .collect::<Vec<_>>()
                .join(", "),
            detail.shown_patch_set.unwrap_or_default()
        ));
    }
    for label in &detail.vote_labels {
        human.push_str(&format!(
            "\nvote {}: {} (now {:+})",
            label.name,
            label
                .values
                .iter()
                .map(|v| format!("{:+}", v.value))
                .collect::<Vec<_>>()
                .join(" "),
            label.current
        ));
    }
    Ok(Emitted {
        exit_code: 0,
        json: out,
        human,
    })
}

/// `review threads`: code threads plus the conversation.
pub(super) async fn threads(
    env: &HostingEnv,
    target: &RepoTarget,
    number: u64,
    unresolved_only: bool,
    patch_set: Option<u32>,
) -> Result<Emitted, String> {
    let client = client_for_target(env, target)?;
    let mut threads = client
        .pull_request_threads_at(&target.project, number, patch_set)
        .await
        .map_err(|e| e.to_string())?;
    if unresolved_only {
        threads.retain(|t| !t.resolved);
    }
    let conversation = client
        .pull_request_conversation(&target.project, number)
        .await
        .map_err(|e| e.to_string())?;
    // The verdicts already given: best effort (a token that cannot list
    // them still gets the threads).
    let reviews = client
        .pull_request_reviews(&target.project, number)
        .await
        .unwrap_or_default();
    let mut human = Vec::new();
    for t in &threads {
        let place = match t.anchor() {
            Some((side, line)) => format!("{}:{line} ({})", t.path, side.as_str()),
            None => format!("{} (outdated)", t.path),
        };
        let state = if t.resolved { "resolved" } else { "open" };
        let first = t.comments.first();
        human.push(format!(
            "[{state}] {place}  {}  {}: {}",
            t.id,
            first.map_or("", |c| c.author.as_str()),
            first.map_or("", |c| c.body.lines().next().unwrap_or(""))
        ));
    }
    for c in &conversation {
        human.push(format!(
            "(conversation) {}: {}",
            c.author,
            c.body.lines().next().unwrap_or("")
        ));
    }
    for r in &reviews {
        human.push(format!(
            "(review) {}: {:?} {}",
            r.author,
            r.state,
            r.body.lines().next().unwrap_or("")
        ));
    }
    let mut out = merge(target_json(target), "number", json!(number));
    out["threads"] = json_of(&threads)?;
    out["conversation"] = json_of(&conversation)?;
    out["reviews"] = json_of(&reviews)?;
    Ok(Emitted {
        exit_code: 0,
        json: out,
        human: human.join("\n"),
    })
}

/// `review comment`.
pub(super) async fn comment(
    env: &HostingEnv,
    target: &RepoTarget,
    number: u64,
    body: String,
) -> Result<Emitted, String> {
    let posted = client_for_target(env, target)?
        .add_comment(&target.project, number, &body)
        .await
        .map_err(|e| e.to_string())?;
    let mut out = merge(target_json(target), "number", json!(number));
    out["comment"] = json_of(&posted)?;
    Ok(Emitted {
        exit_code: 0,
        json: out,
        human: format!("commented on {}", target_label(target, number)),
    })
}

/// `review reply`.
pub(super) async fn reply(
    env: &HostingEnv,
    target: &RepoTarget,
    number: u64,
    thread: &str,
    body: String,
) -> Result<Emitted, String> {
    let posted = client_for_target(env, target)?
        .reply_to_thread(&target.project, number, thread, &body)
        .await
        .map_err(|e| e.to_string())?;
    let mut out = merge(target_json(target), "number", json!(number));
    out["thread"] = json!(thread);
    out["comment"] = json_of(&posted)?;
    Ok(Emitted {
        exit_code: 0,
        json: out,
        human: format!("replied in thread {thread}"),
    })
}

/// `review resolve`.
pub(super) async fn resolve(
    env: &HostingEnv,
    target: &RepoTarget,
    number: u64,
    thread: &str,
    resolved: bool,
) -> Result<Emitted, String> {
    client_for_target(env, target)?
        .resolve_thread(&target.project, number, thread, resolved)
        .await
        .map_err(|e| e.to_string())?;
    let mut out = merge(target_json(target), "number", json!(number));
    out["thread"] = json!(thread);
    out["resolved"] = json!(resolved);
    Ok(Emitted {
        exit_code: 0,
        json: out,
        human: format!(
            "{} thread {thread}",
            if resolved { "resolved" } else { "reopened" }
        ),
    })
}

/// `path:line[:side]:text` of `review submit --comment`. `line` is `N` or a
/// range `N-M`; `side` is `new` (default) or `old`, recognised only as the
/// exact words so a text that starts with another word is not eaten.
pub fn parse_comment_spec(spec: &str) -> Result<PendingComment, String> {
    let bad = || {
        format!(
            "`{spec}` is not a comment: expected path:line[:side]:text (side is `new` or `old`)"
        )
    };
    let (path, rest) = spec.split_once(':').ok_or_else(bad)?;
    let (lines, rest) = rest.split_once(':').ok_or_else(bad)?;
    let number = |t: &str| t.trim().parse::<u32>().ok().filter(|n| *n > 0);
    let (start_line, line) = match lines.split_once('-') {
        Some((a, b)) => (Some(number(a).ok_or_else(bad)?), number(b).ok_or_else(bad)?),
        None => (None, number(lines).ok_or_else(bad)?),
    };
    let (side, text) = match rest.split_once(':') {
        Some((word, tail)) if word.eq_ignore_ascii_case("old") => (DiffSide::Old, tail),
        Some((word, tail)) if word.eq_ignore_ascii_case("new") => (DiffSide::New, tail),
        _ => (DiffSide::New, rest),
    };
    let text = text.trim_start();
    if path.is_empty() || text.trim().is_empty() {
        return Err(bad());
    }
    if start_line.is_some_and(|s| s > line) {
        return Err(format!("`{spec}`: the range runs backwards"));
    }
    Ok(PendingComment {
        path: path.to_string(),
        previous_path: None,
        line,
        side,
        start_line,
        other_line: None,
        body: text.to_string(),
    })
}

fn event_of(arg: ReviewEventArg) -> ReviewEvent {
    match arg {
        ReviewEventArg::Comment => ReviewEvent::Comment,
        ReviewEventArg::Approve => ReviewEvent::Approve,
        ReviewEventArg::RequestChanges => ReviewEvent::RequestChanges,
    }
}

/// `review submit`: reads the head first, so the comments are pinned to the
/// commit the caller could have read, and the host refuses (or marks them
/// outdated) if the request moved since.
#[allow(clippy::too_many_arguments)]
pub(super) async fn review(
    env: &HostingEnv,
    target: &RepoTarget,
    number: u64,
    event: ReviewEventArg,
    body: String,
    comments: &[String],
    votes: &[String],
    patch_set: Option<u32>,
) -> Result<Emitted, String> {
    let event = event_of(event);
    let votes: Vec<ReviewVote> = votes
        .iter()
        .map(|v| parse_vote_spec(v))
        .collect::<Result<_, _>>()?;
    let comments: Vec<PendingComment> = comments
        .iter()
        .map(|c| parse_comment_spec(c))
        .collect::<Result<_, _>>()?;
    let client = client_for_target(env, target)?;
    if !client.review_capabilities().allows(event) {
        return Err(format!(
            "{} does not offer `{}` reviews",
            target.integration.kind.label(),
            event.as_str()
        ));
    }
    let detail = client
        .pull_request_detail_at(&target.project, number, patch_set)
        .await
        .map_err(|e| e.to_string())?;
    // A vote the server does not permit is refused here with the values it
    // does, instead of by an opaque 409.
    for vote in &votes {
        let label = detail
            .vote_labels
            .iter()
            .find(|l| l.name.eq_ignore_ascii_case(&vote.label))
            .ok_or_else(|| {
                format!(
                    "you cannot vote on `{}` here (labels: {})",
                    vote.label,
                    detail
                        .vote_labels
                        .iter()
                        .map(|l| l.name.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            })?;
        if !label.values.iter().any(|v| v.value == vote.value) {
            return Err(format!(
                "`{}` does not permit {:+} for you on this change (permitted: {})",
                label.name,
                vote.value,
                label
                    .values
                    .iter()
                    .map(|v| format!("{:+}", v.value))
                    .collect::<Vec<_>>()
                    .join(" ")
            ));
        }
    }
    let votes: Vec<ReviewVote> = votes
        .into_iter()
        .map(|v| ReviewVote {
            label: detail
                .vote_labels
                .iter()
                .find(|l| l.name.eq_ignore_ascii_case(&v.label))
                .map_or(v.label.clone(), |l| l.name.clone()),
            value: v.value,
        })
        .collect();
    let voted = votes.len();
    let outcome = client
        .submit_review(
            &target.project,
            number,
            &ReviewSubmission {
                votes,
                event,
                body,
                comments,
                head_sha: Some(detail.head_sha),
            },
        )
        .await
        .map_err(|e| e.to_string())?;
    let mut out = merge(target_json(target), "number", json!(number));
    out["event"] = json!(outcome.event.as_str());
    out["comments"] = json!(outcome.comments);
    out["review_id"] = json!(outcome.review_id);
    out["votes"] = json!(voted);
    out["patch_set"] = json!(detail.shown_patch_set);
    Ok(Emitted {
        exit_code: 0,
        json: out,
        human: format!(
            "submitted a `{}` review of {} with {} comment(s)",
            event.as_str(),
            target_label(target, number),
            outcome.comments
        ),
    })
}

/// `Label=value`: `Code-Review=+1`, `Verified=-1`, `Code-Review=0`.
fn parse_vote_spec(spec: &str) -> Result<ReviewVote, String> {
    let (label, value) = spec
        .split_once('=')
        .ok_or_else(|| format!("`{spec}`: expected Label=value (like Code-Review=+1)"))?;
    let value: i32 = value
        .trim()
        .trim_start_matches('+')
        .parse()
        .map_err(|_| format!("`{spec}`: `{}` is not a vote value", value.trim()))?;
    if label.trim().is_empty() {
        return Err(format!("`{spec}`: the label is empty"));
    }
    Ok(ReviewVote {
        label: label.trim().to_string(),
        value,
    })
}

/// `review fetch`: the head and the target branch into `refs/gitpoulpi/pr/<n>`
/// and `...-base`.
pub(super) async fn fetch(
    env: &HostingEnv,
    ipc: &mut Session,
    target: &RepoTarget,
    number: u64,
    remote: Option<String>,
    patch_set: Option<u32>,
    compare_with: Option<u32>,
) -> Result<Emitted, String> {
    let detail = client_for_target(env, target)?
        .pull_request_detail_at(&target.project, number, patch_set)
        .await
        .map_err(|e| e.to_string())?;
    // Gerrit keeps each patch set in its own ref; the others have one head ref.
    let head_spec = match detail.shown_patch_set {
        Some(ps) => review_head_refspec_at(target.integration.kind, number, ps),
        None => review_head_refspec(target.integration.kind, number),
    }
    .ok_or_else(|| {
        format!(
            "{} keeps no head ref to review from",
            target.integration.kind.label()
        )
    })?;
    let remote = match remote {
        Some(r) => r,
        None if !target.remote.is_empty() => target.remote.clone(),
        None => default_remote_name(ipc).await?,
    };
    let base_spec = review_base_refspec(&detail.summary.target_branch, number);
    let mut specs = vec![head_spec, base_spec];
    if let Some(k) = compare_with {
        specs.push(
            gitpoulpi_hosting::review_compare_refspec(target.integration.kind, number, k)
                .ok_or_else(|| "--compare-with is for Gerrit patch sets".to_string())?,
        );
    }
    for spec in &specs {
        ipc.call::<gitpoulpi_core::domain::FetchOutcome>(
            OperationKind::FetchRefspec,
            json!({ "remote": remote, "refspec": spec }),
        )
        .await?;
    }
    let mut out = merge(target_json(target), "number", json!(number));
    if let Some(k) = compare_with {
        out["compare_ref"] = json!(gitpoulpi_hosting::review_compare_ref(number, k));
    }
    out["remote"] = json!(remote);
    out["head_ref"] = json!(review_head_ref(number));
    out["base_ref"] = json!(review_base_ref(number));
    out["head_sha"] = json!(detail.head_sha);
    out["patch_set"] = json!(detail.shown_patch_set);
    Ok(Emitted {
        exit_code: 0,
        json: out,
        human: format!(
            "fetched {} and {} from {remote}",
            review_head_ref(number),
            review_base_ref(number)
        ),
    })
}

/// `review submit-change`: merges a Gerrit change. Outward, not undoable.
pub(super) async fn submit_change(
    env: &HostingEnv,
    target: &RepoTarget,
    number: u64,
) -> Result<Emitted, String> {
    client_for_target(env, target)?
        .submit_change(&target.project, number)
        .await
        .map_err(|e| e.to_string())?;
    let mut out = merge(target_json(target), "number", json!(number));
    out["submitted"] = json!(true);
    out["undoable"] = json!(false);
    Ok(Emitted {
        exit_code: 0,
        json: out,
        human: format!(
            "submitted {} (not undoable from GitPoulpi)",
            target_label(target, number)
        ),
    })
}

/// `review abandon`: abandons a Gerrit change. Outward, not undoable here.
pub(super) async fn abandon(
    env: &HostingEnv,
    target: &RepoTarget,
    number: u64,
    message: Option<String>,
) -> Result<Emitted, String> {
    client_for_target(env, target)?
        .abandon_change(&target.project, number, message.as_deref())
        .await
        .map_err(|e| e.to_string())?;
    let mut out = merge(target_json(target), "number", json!(number));
    out["abandoned"] = json!(true);
    out["undoable"] = json!(false);
    Ok(Emitted {
        exit_code: 0,
        json: out,
        human: format!(
            "abandoned {} (not undoable from GitPoulpi: restore it in Gerrit)",
            target_label(target, number)
        ),
    })
}

/// `review push`: `HEAD` to `refs/for/<branch>` through the daemon (a
/// Gerrit change is created by pushing, never by an API call). Not
/// undoable from here: the change lives on the server.
pub(super) async fn push(
    env: &HostingEnv,
    ipc: &mut Session,
    target: &RepoTarget,
    branch: &str,
    topic: Option<String>,
    remote: Option<String>,
) -> Result<Emitted, String> {
    let _ = env;
    if target.integration.kind.family() != gitpoulpi_config::ApiFamily::Gerrit {
        return Err(format!(
            "{} has no review ref to push to (this is for Gerrit: use `hosting pr` there)",
            target.integration.kind.label()
        ));
    }
    let remote = match remote {
        Some(r) => r,
        None if !target.remote.is_empty() => target.remote.clone(),
        None => default_remote_name(ipc).await?,
    };
    let pushed: gitpoulpi_core::domain::PushOutcome = ipc
        .call(
            OperationKind::PushForReview,
            json!({ "remote": remote, "branch": branch, "topic": topic }),
        )
        .await?;
    let mut out = target_json(target);
    out["remote"] = json!(remote);
    out["branch"] = json!(branch);
    out["pushed_refs"] = json!(pushed.pushed_refs);
    out["undoable"] = json!(false);
    Ok(Emitted {
        exit_code: 0,
        json: out,
        human: format!(
            "pushed HEAD to {} on {remote} (not undoable from GitPoulpi: abandon the change in Gerrit)",
            pushed.pushed_refs.join(", ")
        ),
    })
}

/// `review diff`: the request's files with their hunks. From the clone
/// (fetching its refs) when there is one and `--api` is not given, else
/// from the host's API.
pub(super) async fn diff(
    env: &HostingEnv,
    ipc: Option<&mut Session>,
    target: &RepoTarget,
    number: u64,
    file: Option<String>,
    remote: Option<String>,
) -> Result<Emitted, String> {
    let (files, source) = match ipc {
        Some(ipc) => (local_diff(env, ipc, target, number, remote).await?, "git"),
        None => {
            let client = client_for_target(env, target)?;
            let files = client
                .pull_request_diff(&target.project, number, None, None)
                .await
                .map_err(|e| e.to_string())?;
            (files, "api")
        }
    };
    super::review_api::diff_output(target, number, source, files, file.as_deref())
}

/// The clone's diff in the API's shape.
async fn local_diff(
    env: &HostingEnv,
    ipc: &mut Session,
    target: &RepoTarget,
    number: u64,
    remote: Option<String>,
) -> Result<Vec<gitpoulpi_hosting::review::ApiFileDiff>, String> {
    let files = local_files(env, ipc, target, number, remote).await?;
    Ok(files
        .into_iter()
        .map(super::review_api::api_shape)
        .collect())
}

/// The request's diff from the clone: fetches its refs (like `review
/// fetch`), then `merge-base..head` through the daemon.
async fn local_files(
    env: &HostingEnv,
    ipc: &mut Session,
    target: &RepoTarget,
    number: u64,
    remote: Option<String>,
) -> Result<Vec<gitpoulpi_core::domain::FileDiff>, String> {
    use gitpoulpi_core::domain::{DiffTarget, FileDiff, Oid};

    fetch(env, ipc, target, number, remote, None, None).await?;
    let head: Oid = ipc
        .call(
            OperationKind::Resolve,
            json!({ "rev": review_head_ref(number) }),
        )
        .await?;
    let base: Oid = ipc
        .call(
            OperationKind::Resolve,
            json!({ "rev": review_base_ref(number) }),
        )
        .await?;
    let merge_base: Option<Oid> = ipc
        .call(OperationKind::MergeBase, json!({ "a": base, "b": head }))
        .await?;
    let files: Vec<FileDiff> = ipc
        .call(
            OperationKind::Diff,
            json!({ "target": DiffTarget::CommitRange(merge_base.unwrap_or(base), head) }),
        )
        .await?;
    Ok(files)
}

/// `review suggest`: the AI's suggested review comments, printed, never
/// posted. With a repository (`ipc`) it fetches the request's refs first
/// (like `review fetch`) and diffs `merge-base..head` through the daemon;
/// with `--project` (no `ipc`) it reads the diff from the host's API. The
/// model gets the diff, the description and the threads that exist already.
#[cfg(feature = "ai")]
pub(super) async fn suggest(
    env: &HostingEnv,
    mut ipc: Option<&mut Session>,
    target: &RepoTarget,
    number: u64,
    remote: Option<String>,
    run: gitpoulpi_config::RepoAiReview,
) -> Result<Emitted, String> {
    use gitpoulpi_ai::prompts::{ReviewPrompt, ReviewThreadBrief};
    use gitpoulpi_ai::review_context::gather_report;

    let root = ipc
        .as_deref()
        .and_then(|s| s.repo_root())
        .map(std::path::Path::to_path_buf);
    // What the review is given: the global defaults, the repository's (or the
    // project's) over them, this run's flags over both.
    let project_key =
        gitpoulpi_session::RemoteReviews::project_key(&target.integration.url, &target.project);
    let context = crate::ai_review_ctx::effective(
        root.as_deref(),
        root.is_none().then_some(project_key.as_str()),
        &run,
    )?;
    if root.is_none() && context.run_in_repository == gitpoulpi_config::RunInRepo::Yes {
        return Err(
            "--run-in-repo yes needs a clone: with --project there is no checkout to run in".into(),
        );
    }
    // Fail before any network call when the AI cannot run.
    let job = crate::ai_cmd::job_for_root(root.as_deref(), gitpoulpi_config::AiAction::CodeReview)?;
    let workdir = context.run_in_repository.apply(job.profile.workdir);
    let job = job.with_workdir(Some(workdir));
    let client = client_for_target(env, target)?;
    let (files, omitted, source) = match ipc.as_deref_mut() {
        Some(ipc) => (
            local_files(env, ipc, target, number, remote).await?,
            Vec::new(),
            "git",
        ),
        None => {
            let (files, omitted) = super::review_api::api_files(&*client, target, number).await?;
            (files, omitted, "api")
        }
    };
    let detail = client
        .pull_request_detail(&target.project, number)
        .await
        .map_err(|e| e.to_string())?;
    let threads = client
        .pull_request_threads(&target.project, number)
        .await
        .map_err(|e| e.to_string())?;
    if files.is_empty() {
        return Err("the request changes no file, so there is nothing to review".into());
    }
    let briefs: Vec<ReviewThreadBrief> = threads
        .iter()
        .map(|t| ReviewThreadBrief {
            path: t.path.clone(),
            line: t.line.or(t.original_line),
            resolved: t.resolved,
            comments: t
                .comments
                .iter()
                .map(|c| (c.author.clone(), c.body.clone()))
                .collect(),
        })
        .collect();
    // The project conventions, read at the branch the request targets: the
    // base ref of the clone, or the target branch on the host.
    let base = match ipc {
        Some(ipc) => {
            let base: gitpoulpi_core::domain::Oid = ipc
                .call(
                    OperationKind::Resolve,
                    json!({ "rev": review_base_ref(number) }),
                )
                .await?;
            super::review_ctx::Base::Daemon {
                session: tokio::sync::Mutex::new(ipc),
                base,
            }
        }
        None => super::review_ctx::Base::Api {
            client: &*client,
            project: target.project.clone(),
            branch: detail.summary.target_branch.clone(),
        },
    };
    let gathered = gather_report(
        &context,
        &super::review_ctx::SuggestSource {
            base,
            env,
            integrations: &target.integrations,
        },
    )
    .await;
    let outcome = job
        .suggest_review(&ReviewPrompt {
            title: &detail.summary.title,
            description: &detail.body,
            files: &files,
            threads: &briefs,
            context: &gathered.files,
            in_repository: false,
            budget: 0,
        })
        .await
        .map_err(|e| e.to_string())?;
    let mut human = Vec::new();
    if !outcome.suggestions.summary.is_empty() {
        human.push(outcome.suggestions.summary.clone());
    }
    for s in &outcome.suggestions.suggestions {
        let place = if s.anchored {
            format!("{}:{} ({})", s.path, s.line, s.side.as_str())
        } else {
            format!("{} (not on a line of the diff)", s.path)
        };
        human.push(format!(
            "[{}] {place}: {}",
            s.severity.key(),
            s.body.lines().next().unwrap_or("")
        ));
    }
    let mut out = merge(target_json(target), "number", json!(number));
    out["summary"] = json!(outcome.suggestions.summary);
    out["verdict"] = json!(outcome.suggestions.verdict.unwrap_or_default().key());
    out["body"] = json!(outcome.suggestions.body);
    out["suggestions"] = json_of(&outcome.suggestions.suggestions)?;
    out["cut"] = json!(outcome.cut);
    out["left_out"] = json!(outcome.left_out);
    out["posted"] = json!(false);
    out["context"] = json!({
        "effective": crate::ai_review_ctx::effective_json(&context),
        "files": gathered
            .files
            .iter()
            .map(|f| json!({ "label": f.label, "cut": f.cut }))
            .collect::<Vec<_>>(),
        "problems": gathered.problems,
        "agent_in_repository": job.runs_in_repository(),
    });
    let privacy = job.privacy_report().summary();
    out["privacy"] = json!({
        "redactions": privacy.redactions,
        "excluded_files": privacy.excluded_files,
    });
    out["diff_source"] = json!(source);
    // Files the host would not show are not in the prompt either.
    out["omitted_files"] = json!(omitted);
    Ok(Emitted {
        exit_code: 0,
        json: out,
        human: human.join("\n"),
    })
}

/// `review suggest` in a build without the `ai` feature.
#[cfg(not(feature = "ai"))]
pub(super) async fn suggest(
    _env: &HostingEnv,
    _ipc: Option<&mut Session>,
    _target: &RepoTarget,
    _number: u64,
    _remote: Option<String>,
    _run: gitpoulpi_config::RepoAiReview,
) -> Result<Emitted, String> {
    Err(gitpoulpi_core::Error::NotBuilt { feature: "ai" }.to_string())
}

pub(super) fn target_label(target: &RepoTarget, number: u64) -> String {
    format!("{}#{number}", target.project)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn comment_specs_carry_a_line_a_side_and_text_that_may_hold_colons() {
        let c = parse_comment_spec("src/a.rs:12:why: not here").unwrap();
        assert_eq!(
            (c.path.as_str(), c.line, c.side),
            ("src/a.rs", 12, DiffSide::New)
        );
        assert_eq!(c.body, "why: not here");
        assert_eq!(c.start_line, None);

        let old = parse_comment_spec("b.rs:4:old:gone").unwrap();
        assert_eq!(
            (old.side, old.line, old.body.as_str()),
            (DiffSide::Old, 4, "gone")
        );

        let range = parse_comment_spec("c.rs:10-14:new:extract").unwrap();
        assert_eq!((range.start_line, range.line), (Some(10), 14));

        // A text that starts with another word is not a side.
        assert_eq!(parse_comment_spec("a:1:head: x").unwrap().body, "head: x");
    }

    #[test]
    fn malformed_comment_specs_are_refused() {
        for bad in [
            "a.rs",
            "a.rs:x:text",
            "a.rs:0:text",
            "a.rs:3:",
            ":3:text",
            "a.rs:9-3:t",
        ] {
            assert!(parse_comment_spec(bad).is_err(), "{bad}");
        }
    }
}
