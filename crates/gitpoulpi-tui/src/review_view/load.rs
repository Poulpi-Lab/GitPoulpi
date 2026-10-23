//! The review view's worker calls. Plain threads over one-shot channels,
//! drained by the render loop with `try_recv`; nothing here is awaited on
//! the input thread (the `meta_picker::load` shape).
//!
//! Two loads run side by side: the host's half (details, files, threads,
//! conversation, submitted reviews, metadata) and the repository's half
//! (fetch the request's head and base into `refs/gitpoulpi/pr/<n>`, the
//! merge base, the commits). The repository's half talks to the daemon
//! through the window's [`Requester`] and waits for each answer on its own
//! thread, so the render loop never does.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::time::Duration;

use gitpoulpi_core::domain::{CommitGraph, FetchOutcome, LogOptions, Oid, OperationKind};
use gitpoulpi_hosting::{
    review_base_ref, review_base_refspec, review_compare_ref, review_compare_refspec,
    review_head_ref, review_head_refspec, review_head_refspec_at, HostingClient, MetaKind,
    ReviewComment, ReviewOutcome, ReviewSubmission,
};
use serde_json::json;

use super::model::{GitData, HostData, ReviewTarget};
use crate::daemon_link::{Reply, Requester};
use crate::hosting_task::{error_text, request_task_failed, ClientFactory};
use crate::redraw::NotifySend as _;

/// How many commits of a request the Commits page lists at most.
const COMMIT_LIMIT: u32 = 300;

pub(super) fn runtime() -> Result<tokio::runtime::Runtime, String> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(request_task_failed)
}

// -- the host's half -----------------------------------------------------------

/// Starts reading the request from the host.
pub(super) fn start_host(
    factory: Arc<dyn ClientFactory>,
    target: ReviewTarget,
) -> mpsc::Receiver<Result<HostData, String>> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let result = runtime().and_then(|rt| rt.block_on(host_load(&*factory, &target)));
        let _ = tx.send_notify(result);
    });
    rx
}

async fn host_load(factory: &dyn ClientFactory, target: &ReviewTarget) -> Result<HostData, String> {
    let client = factory
        .client(&target.integration, &target.all)
        .map_err(|e| error_text(&e))?;
    let (project, number) = (target.project.as_str(), target.number);
    let patch_set = target.patch_set;
    let (detail, files, threads, conversation, reviews, meta) = tokio::join!(
        client.pull_request_detail_at(project, number, patch_set),
        client.pull_request_files_against(project, number, patch_set, target.compare_patch_set),
        client.pull_request_threads_at(project, number, patch_set),
        client.pull_request_conversation(project, number),
        client.pull_request_reviews(project, number),
        client.item_meta(project, number, MetaKind::PullRequest),
    );
    Ok(HostData {
        detail: detail.map_err(|e| error_text(&e))?,
        files: files.map_err(|e| error_text(&e))?,
        threads: threads.map_err(|e| error_text(&e))?,
        conversation: conversation.map_err(|e| error_text(&e))?,
        // Nice to have: a host that cannot list them, or a token that may
        // not read them, still gets a review view.
        reviews: reviews.unwrap_or_default(),
        meta: meta.ok(),
    })
}

// -- the repository's half -----------------------------------------------------

/// Starts fetching the request's refs and reading its commits. `cancel`
/// stops the thread when the view is gone.
pub(super) fn start_git(
    requester: Requester,
    target: &ReviewTarget,
    patch_set: Option<u32>,
    cancel: Arc<AtomicBool>,
) -> mpsc::Receiver<Result<GitData, String>> {
    let (tx, rx) = mpsc::channel();
    let target = target.clone();
    std::thread::spawn(move || {
        let result = git_load(&requester, &target, patch_set, &cancel);
        if !cancel.load(Ordering::Relaxed) {
            let _ = tx.send_notify(result);
        }
    });
    rx
}

/// Waits for one daemon answer. A poll loop, not a block on the render
/// thread: this runs on a worker.
fn wait<T>(reply: Reply<T>, cancel: &AtomicBool) -> Result<T, String> {
    loop {
        if let Some(answer) = reply.take() {
            return answer.map_err(|e| e.to_string());
        }
        if cancel.load(Ordering::Relaxed) {
            return Err(String::new());
        }
        std::thread::sleep(Duration::from_millis(4));
    }
}

fn git_load(
    requester: &Requester,
    target: &ReviewTarget,
    patch_set: Option<u32>,
    cancel: &AtomicBool,
) -> Result<GitData, String> {
    let kind = target.integration.kind;
    let number = target.number;
    // Gerrit keeps every patch set in its own ref; the others have one head ref.
    let head_spec = match patch_set {
        Some(ps) => review_head_refspec_at(kind, number, ps),
        None => None,
    }
    .or_else(|| review_head_refspec(kind, number))
    .ok_or_else(|| {
        gitpoulpi_i18n::msg::review_view::no_head_ref(crate::hosting_ui::provider_name(kind))
    })?;
    let base_spec = review_base_refspec(&target.target_branch, number);
    let mut specs = vec![head_spec, base_spec];
    // "Compare with patch set k" (Gerrit): that patch set is the diff's old side.
    if let Some(k) = target.compare_patch_set {
        specs.extend(review_compare_refspec(kind, number, k));
    }
    for spec in specs {
        wait(
            requester.call::<FetchOutcome>(
                OperationKind::FetchRefspec,
                json!({ "remote": target.remote, "refspec": spec }),
            ),
            cancel,
        )?;
    }
    let head_ref = review_head_ref(number);
    let head = wait(
        requester.call::<Oid>(OperationKind::Resolve, json!({ "rev": head_ref })),
        cancel,
    )?;
    let base = wait(
        requester.call::<Oid>(
            OperationKind::Resolve,
            json!({ "rev": review_base_ref(number) }),
        ),
        cancel,
    )?;
    let merge_base = wait(
        requester.call::<Option<Oid>>(OperationKind::MergeBase, json!({ "a": base, "b": head })),
        cancel,
    )?
    // Unrelated histories: diff against the target's tip.
    .unwrap_or(base);
    let opts = LogOptions {
        limit: Some(COMMIT_LIMIT),
        start_ref: Some(head_ref),
        ..LogOptions::default()
    };
    let graph = wait(
        requester.call::<CommitGraph>(OperationKind::Log, json!({ "opts": opts })),
        cancel,
    )?;
    let mut commits: Vec<_> = graph
        .nodes
        .into_iter()
        .take_while(|c| c.id != merge_base)
        .collect();
    let compare = match target.compare_patch_set {
        Some(k) => Some(wait(
            requester.call::<Oid>(
                OperationKind::Resolve,
                json!({ "rev": review_compare_ref(number, k) }),
            ),
            cancel,
        )?),
        None => None,
    };
    if let Some(old) = compare {
        // The head patch set is one commit; against the older patch set its
        // "parent" is that patch set, so the Commits page shows the
        // difference between the two.
        commits.truncate(1);
        if let Some(c) = commits.first_mut() {
            c.parents = vec![old];
        }
    }
    Ok(GitData {
        head,
        diff_base: compare.unwrap_or(merge_base),
        comparing: compare.is_some(),
        commits,
    })
}

// -- writes -----------------------------------------------------------------

/// A call that changes something on the host.
#[derive(Debug, Clone)]
pub(super) enum Call {
    /// A comment in the conversation, posted at once.
    Comment(String),
    /// A reply to a thread, posted at once.
    Reply {
        thread_id: String,
        body: String,
    },
    Resolve {
        thread_id: String,
        resolved: bool,
    },
    /// The pending review: comments and the verdict, in one go.
    Submit(ReviewSubmission),
    /// Merge the change on the server (Gerrit). Not undoable.
    SubmitChange,
    /// Abandon the change on the server (Gerrit). Not undoable.
    AbandonChange,
}

/// What a call did, for the view to apply.
#[derive(Debug, Clone)]
pub(super) enum Done {
    Comment(ReviewComment),
    Reply {
        thread_id: String,
        comment: ReviewComment,
    },
    Resolved {
        thread_id: String,
        resolved: bool,
    },
    Submitted(ReviewOutcome),
    ChangeSubmitted,
    ChangeAbandoned,
}

pub(super) fn start_call(
    factory: Arc<dyn ClientFactory>,
    target: ReviewTarget,
    call: Call,
) -> mpsc::Receiver<Result<Done, String>> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let result = runtime().and_then(|rt| rt.block_on(run_call(&*factory, &target, call)));
        let _ = tx.send_notify(result);
    });
    rx
}

async fn run_call(
    factory: &dyn ClientFactory,
    target: &ReviewTarget,
    call: Call,
) -> Result<Done, String> {
    let client = factory
        .client(&target.integration, &target.all)
        .map_err(|e| error_text(&e))?;
    let (project, number) = (target.project.as_str(), target.number);
    perform(&*client, project, number, call)
        .await
        .map_err(|e| error_text(&e))
}

async fn perform(
    client: &dyn HostingClient,
    project: &str,
    number: u64,
    call: Call,
) -> gitpoulpi_hosting::Result<Done> {
    Ok(match call {
        Call::Comment(body) => Done::Comment(client.add_comment(project, number, &body).await?),
        Call::Reply { thread_id, body } => Done::Reply {
            comment: client
                .reply_to_thread(project, number, &thread_id, &body)
                .await?,
            thread_id,
        },
        Call::Resolve {
            thread_id,
            resolved,
        } => {
            client
                .resolve_thread(project, number, &thread_id, resolved)
                .await?;
            Done::Resolved {
                thread_id,
                resolved,
            }
        }
        Call::Submit(review) => {
            Done::Submitted(client.submit_review(project, number, &review).await?)
        }
        Call::SubmitChange => {
            client.submit_change(project, number).await?;
            Done::ChangeSubmitted
        }
        Call::AbandonChange => {
            client.abandon_change(project, number, None).await?;
            Done::ChangeAbandoned
        }
    })
}
