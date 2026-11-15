//! The "Write with AI" worker: resolves the job (profile, key, global +
//! workspace + repository instructions), asks the model through
//! `gitpoulpi_ai::hosting_reply` (or, for a pull request description, reads
//! the commits and the diff from the daemon first) and sends back the text.
//!
//! One thread per run with its own small runtime; the panel polls the
//! channel. Stop sets the flag: the `select!` drops the request, which kills a
//! local agent's process. The worker never touches a host: the answer is a
//! text and nothing else. The "What is sent" dialog (`ai_sent_dialog`) is the
//! job's send gate, so it appears here like for every other AI request.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::time::Duration;

use gitpoulpi_config::AiAction;
use gitpoulpi_core::domain::RepoRef;

use super::ai::{AiScope, AiTask, Answer};
use crate::redraw::NotifySend as _;

pub(super) struct Input {
    pub(super) scope: AiScope,
    pub(super) task: AiTask,
    pub(super) cancel: Arc<AtomicBool>,
    /// "Let the AI read the repository" was ticked for this draft.
    pub(super) read_repo: bool,
    /// Where the worker says what it read (the panel shows it).
    pub(super) context: Arc<std::sync::Mutex<Option<String>>>,
    #[cfg(test)]
    pub(super) fake: Option<super::ai::Fake>,
}

fn action_of(task: &AiTask) -> AiAction {
    match task {
        AiTask::Reply(request) => request.target.action(),
        AiTask::PullRequest { .. } => AiAction::PullRequestDescription,
    }
}

/// Starts the run; the jobs list shows it (with its Cancel) until it ends.
pub(super) fn start(input: Input) -> mpsc::Receiver<Answer> {
    let (tx, rx) = mpsc::channel();
    let label = format!(
        "{} {}",
        crate::theme::AI_SPARK,
        gitpoulpi_i18n::msg::md_editor::AI_JOB.get()
    );
    let flag = input.cancel.clone();
    let job =
        crate::jobs_ui::local_start(label, Arc::new(move || flag.store(true, Ordering::Relaxed)));
    std::thread::spawn(move || {
        let cancel = input.cancel.clone();
        let answer = run(input);
        let state = match &answer {
            _ if cancel.load(Ordering::Relaxed) => gitpoulpi_ipc::JobState::Canceled,
            Ok(_) => gitpoulpi_ipc::JobState::Done,
            Err(_) => gitpoulpi_ipc::JobState::Failed,
        };
        crate::jobs_ui::local_finish(job, state);
        if !cancel.load(Ordering::Relaxed) {
            let _ = tx.send_notify(answer);
        }
    });
    rx
}

fn run(input: Input) -> Answer {
    #[cfg(test)]
    if let Some(fake) = &input.fake {
        return fake(&input.task);
    }
    let settings = gitpoulpi_config::Settings::resolve_ambient().map_err(|e| e.to_string())?;
    let config = settings.config().ai.clone();
    if !config.enabled {
        return Err(gitpoulpi_i18n::msg::ai_ui::OFF.get().to_string());
    }
    let action = action_of(&input.task);
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| e.to_string())?;
    let cancel = input.cancel.clone();
    let scope = input.scope;
    let task = input.task;
    let read_repo = input.read_repo;
    let context = input.context.clone();
    let answer = runtime.block_on(async {
        let ask = async {
            let job = match &scope.root {
                Some(root) => crate::ai_ui::prepare_for_root(&config, &scope.session, root, action),
                None => crate::ai_ui::prepare_remote(&config, action),
            }?;
            match task {
                // The repository is read only when the box was ticked, and the
                // AI still only drafts.
                AiTask::Reply(request) if read_repo => {
                    let draft =
                        super::ai_context::draft_reading(job, *request, &scope, &config).await?;
                    if let Ok(mut slot) = context.lock() {
                        *slot = super::ai_context::line(&draft.context);
                    }
                    Ok(draft.text)
                }
                AiTask::Reply(request) => gitpoulpi_ai::hosting_reply::draft(&job, &request)
                    .await
                    .map(|d| d.text)
                    .map_err(|e| e.to_string()),
                AiTask::PullRequest {
                    source,
                    target,
                    hint,
                    draft,
                } => {
                    let Some(root) = scope.root.as_deref() else {
                        return Err(gitpoulpi_i18n::msg::md_editor::AI_NEEDS_REPOSITORY
                            .get()
                            .to_string());
                    };
                    crate::ai_ui::pull_request_hinted(
                        &config,
                        &scope.session,
                        &RepoRef::from_root(root),
                        &source,
                        &target,
                        &hint,
                        draft.as_deref(),
                    )
                    .await
                    .map(|t| t.description)
                }
            }
        };
        let canceled = async {
            while !cancel.load(Ordering::Relaxed) {
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
        };
        tokio::select! {
            result = ask => result,
            _ = canceled => Err(String::new()),
        }
    });
    runtime.shutdown_background();
    answer
}
