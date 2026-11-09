//! The suggestion worker: resolves the `code_review` job (profile, key,
//! global + workspace + repository instructions), reads the request's whole
//! diff from the daemon, asks the model and turns the answer into this
//! view's [`Item`]s. One thread per run; the view polls the channel and
//! cancels through the flag (dropping the request also stops a local agent:
//! its process is killed on drop).

use std::sync::atomic::Ordering;
use std::sync::mpsc;
use std::time::Duration;

use gitpoulpi_ai::assist::ReviewOutcome;
use gitpoulpi_ai::prompts::{ReviewPrompt, ReviewThreadBrief};
use gitpoulpi_ai::review::{DraftVerdict, Severity, Side, Suggestion};
use gitpoulpi_config::AiAction;
use gitpoulpi_core::domain::FileDiff;
use gitpoulpi_hosting::{DiffSide, ReviewEvent};

use super::ai::{Answer, AskInput, Done};
use super::model::{Item, Place, Sev};
use crate::redraw::NotifySend as _;

fn side(s: Side) -> DiffSide {
    match s {
        Side::Old => DiffSide::Old,
        Side::New => DiffSide::New,
    }
}

fn severity(s: Severity) -> Sev {
    match s {
        Severity::Info => Sev::Info,
        Severity::Warning => Sev::Warning,
        Severity::Issue => Sev::Issue,
    }
}

/// A suggestion as the view holds it. `id` is given by the view.
fn item_of(s: &Suggestion) -> Item {
    let place = if s.anchored {
        Some(Place {
            side: side(s.side),
            line: s.line,
            other_line: s.other_line,
            start_line: s.start_line,
        })
    } else {
        // A comment about the file goes on its first changed line, saying
        // which line it was about.
        s.fallback.map(|a| Place {
            side: side(a.side),
            line: a.line,
            other_line: a.other_line,
            start_line: None,
        })
    };
    Item {
        id: 0,
        path: s.path.clone(),
        line: s.line,
        severity: severity(s.severity),
        body: s.body.clone(),
        anchored: s.anchored,
        place,
        prefix: if !s.anchored && s.line > 0 {
            gitpoulpi_i18n::msg::review_ai::about_prefix(s.line)
        } else {
            String::new()
        },
    }
}

/// Starts the run.
pub(super) fn start(input: AskInput) -> mpsc::Receiver<Answer> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let cancel = input.cancel.clone();
        let answer = run(input);
        if !cancel.load(Ordering::Relaxed) {
            let _ = tx.send_notify(answer);
        }
    });
    rx
}

/// The whole request's diff: where the suggestions are placed. From the
/// daemon for a clone, from the host's `pull_request_diff` without one.
fn read_files(input: &AskInput) -> Result<Vec<FileDiff>, String> {
    let reply = input.source.all_files(input.base, input.head);
    loop {
        if let Some(answer) = reply.take() {
            return answer;
        }
        if input.cancel.load(Ordering::Relaxed) {
            return Err(String::new());
        }
        std::thread::sleep(Duration::from_millis(4));
    }
}

fn run(input: AskInput) -> Answer {
    let files = read_files(&input)?;
    if files.is_empty() {
        return Err(gitpoulpi_i18n::msg::review_ai::WHY_NO_DIFF
            .get()
            .to_string());
    }
    #[cfg(test)]
    if let Some(fake) = &input.fake {
        return Ok(fake(&files));
    }
    let settings = gitpoulpi_config::Settings::resolve_ambient().map_err(|e| e.to_string())?;
    let job = match &input.repo_root {
        Some(root) => crate::ai_ui::prepare_for_root(
            &settings.config().ai,
            &input.session,
            root,
            AiAction::CodeReview,
        )?,
        None => crate::ai_ui::prepare_remote(&settings.config().ai, AiAction::CodeReview)?,
    };
    // What the run is given besides the diff: the global `[ai.review]`, the
    // repository's (or the project's) own values over it. `run_in_repository`
    // decides where a local agent works.
    let config = settings.config();
    let own = super::ai_context::own_values(
        input.repo_root.as_deref(),
        input.remote_file.as_deref(),
        &input.remote_key,
        &config.session.per_repo_state_dir,
    );
    let context = super::ai_context::effective(&config.ai.review, &own, &input.per_run);
    let workdir = context.run_in_repository.apply(job.profile.workdir);
    let job = job.with_workdir(Some(workdir));
    let source = super::ai_context::ViewContext {
        source: std::sync::Mutex::new(input.source.clone()),
        base: input.base,
        cancel: input.cancel.clone(),
        integrations: input.integrations.clone(),
    };
    let briefs: Vec<ReviewThreadBrief> = input
        .threads
        .iter()
        .map(|t| ReviewThreadBrief {
            path: t.path.clone(),
            line: t.line,
            resolved: t.resolved,
            comments: t.comments.clone(),
        })
        .collect();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| e.to_string())?;
    let cancel = input.cancel.clone();
    let outcome: Result<ReviewOutcome, String> = runtime.block_on(async {
        // The context files, the description, the comments and the diff are
        // read and capped together (`[ai.limits]`): over a cap the worker
        // waits for the answer of "Over the AI limit" (`ai_limit_dialog`).
        let ask = async {
            let prompt = ReviewPrompt {
                title: &input.title,
                description: &input.description,
                files: &files,
                threads: &briefs,
                ..ReviewPrompt::default()
            };
            job.suggest_review_with_context(&context, &source, &prompt)
                .await
        };
        let canceled = async {
            while !cancel.load(Ordering::Relaxed) {
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
        };
        tokio::select! {
            result = ask => result.map_err(|e| e.to_string()),
            _ = canceled => Err(String::new()),
        }
    });
    let outcome = outcome?;
    let mut notes: Vec<String> = outcome
        .context_problems
        .iter()
        .cloned()
        .map(gitpoulpi_i18n::msg::review_options::context_problem)
        .collect();
    notes.extend(
        outcome
            .context_cut
            .iter()
            .map(gitpoulpi_i18n::msg::review_options::context_cut),
    );
    Ok(Done {
        items: outcome
            .suggestions
            .suggestions
            .iter()
            .map(item_of)
            .collect(),
        verdict: match outcome.suggestions.verdict.unwrap_or_default() {
            DraftVerdict::Approve => ReviewEvent::Approve,
            DraftVerdict::RequestChanges => ReviewEvent::RequestChanges,
            DraftVerdict::Comment => ReviewEvent::Comment,
        },
        body: outcome.suggestions.body,
        summary: outcome.suggestions.summary,
        cut: outcome.cut.len(),
        left_out: outcome.left_out.len(),
        context_notes: notes,
    })
}
