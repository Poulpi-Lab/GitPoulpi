//! The board's worker calls: plain threads over one-shot channels, drained
//! by the render loop with `try_recv`; nothing here runs on the input thread
//! (the `meta_picker::load` and `review_view::load` shape).

use std::sync::{mpsc, Arc};

use gitpoulpi_hosting::board::{BoardColumn, BoardSource, BoardSpec};
use gitpoulpi_hosting::{HostingClient, IssueFilter, IssueSummary, StateFilter};

use super::BoardTarget;
use crate::hosting_task::{error_text, request_task_failed, ClientFactory};
use crate::redraw::NotifySend as _;

/// How many closed issues a host board lists at most: the Closed column is a
/// recent history, not the whole archive.
pub(super) const CLOSED_LIMIT: usize = 60;

/// What a load brings back.
pub(super) struct Loaded {
    pub(super) issues: Vec<IssueSummary>,
    /// The provider's declared column names (`issue_statuses`).
    pub(super) statuses: Vec<String>,
    pub(super) source: BoardSource,
}

fn runtime() -> Result<tokio::runtime::Runtime, String> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(request_task_failed)
}

fn client(
    factory: &dyn ClientFactory,
    target: &BoardTarget,
) -> Result<Box<dyn HostingClient>, String> {
    factory
        .client(&target.seed.integration, &target.all)
        .map_err(|e| error_text(&e))
}

/// Starts reading the issues (every state) and the declared columns.
pub(super) fn start_load(
    factory: Arc<dyn ClientFactory>,
    target: &BoardTarget,
) -> mpsc::Receiver<Result<Loaded, String>> {
    let (tx, rx) = mpsc::channel();
    let target = target.clone();
    std::thread::spawn(move || {
        let result = runtime().and_then(|rt| rt.block_on(load(&*factory, &target)));
        let _ = tx.send_notify(result);
    });
    rx
}

async fn load(factory: &dyn ClientFactory, target: &BoardTarget) -> Result<Loaded, String> {
    let client = client(factory, target)?;
    let project = target.seed.project.as_str();
    let source = client.board_source();
    let mut filter = IssueFilter {
        query: target.query.clone(),
        ..IssueFilter::default()
    };
    // A notes folder is cheap to read whole; a host is asked for the open
    // issues and the recently closed ones.
    let local = target.seed.integration.kind.capabilities().local_folder;
    filter.state = if local {
        StateFilter::All
    } else {
        StateFilter::Open
    };
    let mut issues = client
        .list_issues(project, &filter)
        .await
        .map_err(|e| error_text(&e))?;
    if !local {
        filter.state = StateFilter::Closed;
        // Nice to have: a board without its Closed column is still a board.
        if let Ok(mut closed) = client.list_issues(project, &filter).await {
            closed.truncate(CLOSED_LIMIT);
            issues.extend(closed);
        }
    }
    let statuses = client.issue_statuses(project).await.unwrap_or_default();
    Ok(Loaded {
        issues,
        statuses,
        source,
    })
}

/// Starts moving `issue` to `to`; the answer is `Ok(())` once the provider
/// took it.
pub(super) fn start_move(
    factory: Arc<dyn ClientFactory>,
    target: &BoardTarget,
    issue: IssueSummary,
    spec: BoardSpec,
    to: BoardColumn,
) -> mpsc::Receiver<Result<(), String>> {
    let (tx, rx) = mpsc::channel();
    let target = target.clone();
    std::thread::spawn(move || {
        let result = runtime().and_then(|rt| {
            rt.block_on(async {
                let client = client(&*factory, &target)?;
                client
                    .move_issue(&target.seed.project, &issue, &spec, &to)
                    .await
                    .map_err(|e| error_text(&e))?;
                crate::hosting_notify::notify(
                    &target.seed.integration.id,
                    &target.seed.project,
                    gitpoulpi_ipc::HostingItemKind::Issue,
                    issue.number,
                    "",
                    gitpoulpi_ipc::HostingChange::StateChanged,
                );
                Ok(())
            })
        });
        let _ = tx.send_notify(result);
    });
    rx
}
