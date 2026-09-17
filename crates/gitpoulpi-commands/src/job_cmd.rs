//! `gitpoulpi job list|cancel|wait`: the registry lives in the daemon
//! (`gitpoulpi-daemon/src/jobs.rs`); these only ask it.

use std::time::{Duration, Instant};

use gitpoulpi_core::domain::OperationKind;
use gitpoulpi_ipc::{JobInfo, JobState};
use serde_json::json;

use crate::cli::JobAction;
use crate::client::Session;
use crate::repo_cmd::Emitted;

/// How often `job wait` asks the daemon.
const POLL: Duration = Duration::from_millis(100);

pub async fn run_via_daemon(ipc: &mut Session, action: JobAction) -> Result<Emitted, String> {
    match action {
        JobAction::List => {
            let jobs = list(ipc).await?;
            let human = if jobs.is_empty() {
                "no jobs".to_string()
            } else {
                jobs.iter().map(line).collect::<Vec<_>>().join("\n")
            };
            Ok(emitted(serde_json::to_value(&jobs).map_err(err)?, human, 0))
        }
        JobAction::Cancel { id } => {
            let job: JobInfo = ipc
                .call_without_repo(OperationKind::JobCancel, json!({ "id": id }))
                .await?;
            let human = format!("cancel requested: {}", line(&job));
            Ok(emitted(serde_json::to_value(&job).map_err(err)?, human, 0))
        }
        JobAction::Wait { id, timeout } => {
            let deadline = timeout.map(|s| Instant::now() + Duration::from_secs(s));
            let mut seen = false;
            loop {
                let jobs = list(ipc).await?;
                match jobs.into_iter().find(|j| j.id == id) {
                    Some(job) if job.state.is_finished() => {
                        let code = i32::from(job.state != JobState::Done);
                        let human = line(&job);
                        return Ok(emitted(
                            serde_json::to_value(&job).map_err(err)?,
                            human,
                            code,
                        ));
                    }
                    Some(_) => seen = true,
                    // It ended and was dropped from the list (30 s) before
                    // this call looked, or it never existed.
                    None => {
                        return Err(if seen {
                            format!("job {id} left the list before its result was read")
                        } else {
                            format!("no such job: {id}")
                        });
                    }
                }
                if deadline.is_some_and(|d| Instant::now() >= d) {
                    let still = list(ipc).await?.into_iter().find(|j| j.id == id);
                    let human = format!("timed out waiting for job {id}");
                    return Ok(emitted(
                        json!({ "timed_out": true, "job": still }),
                        human,
                        1,
                    ));
                }
                tokio::time::sleep(POLL).await;
            }
        }
    }
}

async fn list(ipc: &mut Session) -> Result<Vec<JobInfo>, String> {
    ipc.call_without_repo(OperationKind::JobList, json!({}))
        .await
}

fn err(e: serde_json::Error) -> String {
    e.to_string()
}

fn emitted(json: serde_json::Value, human: String, exit_code: i32) -> Emitted {
    Emitted {
        json,
        human,
        exit_code,
    }
}

/// One job as a line of text.
fn line(job: &JobInfo) -> String {
    let mut text = format!("{} [{:?}] {}", job.id, job.state, job.label);
    if let Some(phase) = &job.phase {
        text.push_str(&format!(" ({phase})"));
    }
    if let Some(percent) = job.progress {
        text.push_str(&format!(" {percent}%"));
    }
    if let Some(reason) = job.not_cancellable.filter(|_| !job.state.is_finished()) {
        text.push_str(&format!(" - cannot be canceled: {}", reason.message()));
    }
    if let Some(error) = &job.error {
        text.push_str(&format!(" - {error}"));
    }
    text
}
