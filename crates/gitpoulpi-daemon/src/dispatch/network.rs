use std::path::PathBuf;

use super::*;
use gitpoulpi_core::domain::{
    CloneOptions, CloneProgress, FetchOptions, LfsFetchOptions, LfsProgress, LfsPushOptions,
    OperationKind, PullOptions, PushOptions, RemoteAccessOptions, TagSelection,
};
use gitpoulpi_core::GitBackend;
use gitpoulpi_ipc::{Event, Request};
use serde_json::Value;
use tokio::sync::broadcast;

pub(super) async fn dispatch_network_plain(
    backend: &dyn GitBackend,
    request: &Request,
    events: &broadcast::Sender<Event>,
) -> Dispatched {
    let args = &request.args;

    // Clone is the one operation with no repo to target -- it is what
    // *produces* one -- so it has to be answered before `repo_ref`, which
    // would otherwise reject the request for having no `repo`.
    if request.command == OperationKind::Clone {
        let url: String = arg_or_err!(args, "url", request.id);
        let dest: PathBuf = arg_or_err!(args, "dest", request.id);
        let opts: CloneOptions = args
            .get("opts")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default();

        // The sink is a plain closure over the broadcast sender, which is
        // exactly why `CloneProgressSink` is a callback rather than a
        // channel: `gitpoulpi-core` names no runtime, and the adapter
        // that does is three lines long and lives here.
        let sink_url = url.clone();
        let sink_dest = dest.clone();
        let sink_events = events.clone();
        let sink = move |progress: CloneProgress| {
            let _ = sink_events.send(Event::CloneProgress {
                url: sink_url.clone(),
                dest: sink_dest.clone(),
                progress,
            });
        };

        let outcome = backend.clone_with_progress(&url, &dest, &opts, &sink).await;
        let _ = events.send(Event::CloneFinished {
            url: url.clone(),
            dest: dest.clone(),
            result: outcome
                .as_ref()
                .map(|repo| repo.id.clone())
                .map_err(ToString::to_string),
        });
        return match outcome {
            Ok(repo) => match serde_json::to_value(repo) {
                Ok(json) => ok(request.id, json, false),
                Err(e) => err(request.id, format!("failed to encode result: {e}"), false),
            },
            Err(e) => err(request.id, e.to_string(), false),
        };
    }

    let repo = match repo_ref(request) {
        Ok(r) => r,
        Err(message) => return err(request.id, message, false),
    };

    macro_rules! to_json {
        ($result:expr) => {
            match $result {
                Ok(value) => match serde_json::to_value(value) {
                    Ok(json) => Ok(json),
                    Err(e) => Err((format!("failed to encode result: {e}"), None, None)),
                },
                Err(e) => Err((e.to_string(), e.typed(), e.detail().cloned())),
            }
        };
    }

    let outcome: Result<Value, Failure> = match request.command {
        OperationKind::Fetch => {
            let remote: String = arg_or_err!(args, "remote", request.id);
            let opts: FetchOptions = args
                .get("opts")
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_default();
            to_json!(backend.fetch(&repo, &remote, &opts).await)
        }
        OperationKind::FetchAll => {
            let opts: FetchOptions = args
                .get("opts")
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_default();
            to_json!(backend.fetch_all(&repo, &opts).await)
        }
        OperationKind::RemoteAccessCheck => {
            let remote: String = arg_or_err!(args, "remote", request.id);
            let opts: RemoteAccessOptions = args
                .get("opts")
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_default();
            to_json!(backend.remote_access_check(&repo, &remote, &opts).await)
        }
        OperationKind::FetchRefspec => {
            let remote: String = arg_or_err!(args, "remote", request.id);
            let refspec: String = arg_or_err!(args, "refspec", request.id);
            to_json!(backend.fetch_refspec(&repo, &remote, &refspec).await)
        }
        OperationKind::FetchRefAsBranch => {
            let remote: String = arg_or_err!(args, "remote", request.id);
            let remote_ref: String = arg_or_err!(args, "remote_ref", request.id);
            let branch: String = arg_or_err!(args, "branch", request.id);
            to_json!(
                backend
                    .fetch_ref_as_branch(&repo, &remote, &remote_ref, &branch)
                    .await
            )
        }
        OperationKind::Pull => {
            let opts: PullOptions = args
                .get("opts")
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_default();
            // Download, then integrate: the job can be stopped until the
            // integrate starts (`jobs::Policy::UntilPoint`).
            let pulled = async {
                // The phase first: the test hook's sleep stands for a slow
                // download and must happen *inside* the "fetch" phase, or a
                // test polling for that phase races the real (millisecond)
                // download instead of finding the stretch it asked for.
                crate::jobs::set_phase("fetch");
                crate::test_hook::pull_fetch_delay().await;
                backend.pull_fetch(&repo, &opts).await?;
                crate::jobs::enter_point("integrate").await;
                crate::test_hook::integrate_delay().await;
                let outcome = backend.pull_integrate(&repo, &opts).await;
                if outcome.is_err() {
                    // Refused before changing anything (local changes in the
                    // way): the auto-stash's second download may be stopped.
                    crate::jobs::leave_point("fetch");
                }
                outcome
            }
            .await;
            to_json!(pulled)
        }
        OperationKind::Push => {
            let remote: String = arg_or_err!(args, "remote", request.id);
            let opts: PushOptions = args
                .get("opts")
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_default();
            to_json!(backend.push(&repo, &remote, &opts).await)
        }
        // A rewrite pushed back: every ref with its own recorded lease, one
        // atomic push per remote. A refusal is a per-ref status in the
        // answer, so the answer is what the frontends show.
        OperationKind::RewritePush => {
            let push: gitpoulpi_core::domain::rewrite::PushRewritten =
                match serde_json::from_value(args.clone()) {
                    Ok(parsed) => parsed,
                    Err(e) => return err(request.id, format!("invalid arguments: {e}"), false),
                };
            crate::jobs::set_phase("push");
            to_json!(backend.push_rewritten(&repo, &push).await)
        }
        OperationKind::PushForReview => {
            let remote: String = arg_or_err!(args, "remote", request.id);
            let branch: String = arg_or_err!(args, "branch", request.id);
            let topic: Option<String> = args
                .get("topic")
                .and_then(|v| serde_json::from_value(v.clone()).ok());
            to_json!(
                backend
                    .push_for_review(&repo, &remote, &branch, topic.as_deref())
                    .await
            )
        }
        // `selection` carries the `Named`/`AllUnpushed` choice verbatim,
        // so "push every unpushed tag" stays one request, one push and
        // one broadcast -- no frontend ever loops over tags.
        OperationKind::TagPush => {
            let remote: String = arg_or_err!(args, "remote", request.id);
            let selection: TagSelection = arg_or_err!(args, "selection", request.id);
            to_json!(backend.tag_push(&repo, &remote, &selection).await)
        }
        OperationKind::TagDeleteRemote => {
            let remote: String = arg_or_err!(args, "remote", request.id);
            let name: String = arg_or_err!(args, "name", request.id);
            let deleted = match leased(args) {
                Some(expect) => {
                    backend
                        .remote_ref_update(
                            &repo,
                            &remote,
                            &format!("refs/tags/{name}"),
                            None,
                            Some(expect),
                        )
                        .await
                }
                None => backend.tag_delete_remote(&repo, &remote, &name).await,
            };
            // "Delete here and on the remote": the local half only once the
            // remote one happened.
            let deleted = match (deleted, local_twin(args, &name)) {
                (Ok(()), Some(local)) => backend.tag_delete(&repo, &local),
                (other, _) => other,
            };
            to_json!(deleted)
        }
        OperationKind::TagFetch => {
            let remote: String = arg_or_err!(args, "remote", request.id);
            let name: String = arg_or_err!(args, "name", request.id);
            to_json!(backend.tag_fetch(&repo, &remote, &name).await)
        }
        // Progress is a plain closure over the broadcast sender, like the
        // clone sink above; `operation` lets one event serve all three.
        OperationKind::LfsCheckout => {
            let paths: Vec<PathBuf> = args
                .get("paths")
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_default();
            let sink_repo = repo.id.clone();
            let sink_events = events.clone();
            let sink = move |progress: LfsProgress| {
                let _ = sink_events.send(Event::LfsProgress {
                    repo: sink_repo.clone(),
                    operation: OperationKind::LfsCheckout,
                    progress,
                });
            };
            to_json!(backend.lfs_checkout(&repo, &paths, &sink).await)
        }
        OperationKind::LfsFetch | OperationKind::LfsPull | OperationKind::LfsPush => {
            let remote: String = arg_or_err!(args, "remote", request.id);
            let operation = request.command;
            let sink_repo = repo.id.clone();
            let sink_events = events.clone();
            let sink = move |progress: LfsProgress| {
                let _ = sink_events.send(Event::LfsProgress {
                    repo: sink_repo.clone(),
                    operation,
                    progress,
                });
            };
            match operation {
                OperationKind::LfsFetch => {
                    let opts: LfsFetchOptions = args
                        .get("opts")
                        .and_then(|v| serde_json::from_value(v.clone()).ok())
                        .unwrap_or_default();
                    to_json!(backend.lfs_fetch(&repo, &remote, &opts, &sink).await)
                }
                // `git lfs pull` is `git lfs fetch` then `git lfs checkout`;
                // the download can be stopped, the checkout rewrites files.
                OperationKind::LfsPull => {
                    let pulled = async {
                        crate::jobs::set_phase("download");
                        backend
                            .lfs_fetch(&repo, &remote, &LfsFetchOptions::default(), &sink)
                            .await?;
                        crate::jobs::enter_point("checkout").await;
                        crate::test_hook::integrate_delay().await;
                        backend.lfs_checkout(&repo, &[], &sink).await
                    }
                    .await;
                    to_json!(pulled)
                }
                _ => {
                    let opts: LfsPushOptions = args
                        .get("opts")
                        .and_then(|v| serde_json::from_value(v.clone()).ok())
                        .unwrap_or_default();
                    to_json!(backend.lfs_push(&repo, &remote, &opts, &sink).await)
                }
            }
        }
        OperationKind::LfsLocks => {
            let remote: String = arg_or_err!(args, "remote", request.id);
            to_json!(backend.lfs_locks(&repo, &remote).await)
        }
        OperationKind::LfsLock => {
            let remote: String = arg_or_err!(args, "remote", request.id);
            let path: PathBuf = arg_or_err!(args, "path", request.id);
            to_json!(backend.lfs_lock(&repo, &remote, &path).await)
        }
        OperationKind::LfsUnlock => {
            let remote: String = arg_or_err!(args, "remote", request.id);
            let path: PathBuf = arg_or_err!(args, "path", request.id);
            let force: bool = args
                .get("force")
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or(false);
            to_json!(backend.lfs_unlock(&repo, &remote, &path, force).await)
        }
        OperationKind::BranchDeleteRemote => {
            let remote: String = arg_or_err!(args, "remote", request.id);
            let name: String = arg_or_err!(args, "name", request.id);
            let deleted = match leased(args) {
                Some(expect) => {
                    backend
                        .remote_ref_update(
                            &repo,
                            &remote,
                            &format!("refs/heads/{name}"),
                            None,
                            Some(expect),
                        )
                        .await
                }
                None => backend.branch_delete_remote(&repo, &remote, &name).await,
            };
            let deleted = match (deleted, local_twin(args, &name)) {
                (Ok(()), Some(local)) => backend.branch_delete(&repo, &local),
                (other, _) => other,
            };
            to_json!(deleted)
        }
        // The workflow actions: git steps, hooks and pushes in one run.
        OperationKind::FlowRun => crate::flow::run(backend, &repo, args).await,
        other => {
            return err(
                request.id,
                format!("{other:?} is not a network operation"),
                true,
            )
        }
    };

    // Listing locks reads the server and changes nothing here, so it must
    // not broadcast `RepoChanged` (a client refreshing on that event would
    // answer its own poll with another refresh).
    let mutated = !matches!(
        request.command,
        OperationKind::LfsLocks | OperationKind::RemoteAccessCheck
    );
    match outcome {
        Ok(value) => {
            if mutated {
                tracing::info!(
                    target: "daemon",
                    repo = request.repo.as_ref().map_or("", |r| r.0.as_str()),
                    "{:?} completed",
                    request.command
                );
            } else {
                tracing::debug!(
                    target: "daemon",
                    repo = request.repo.as_ref().map_or("", |r| r.0.as_str()),
                    "{:?} completed",
                    request.command
                );
            }
            ok(request.id, value, mutated)
        }
        Err((message, kind, detail)) => failed(request, message, kind, detail),
    }
}

/// The value a remote deletion is leased on (`expect`, set by the journaled
/// path after asking the server).
fn leased(args: &Value) -> Option<gitpoulpi_core::domain::Oid> {
    serde_json::from_value(args.get("expect")?.clone()).ok()
}

/// The local branch or tag the request deletes together with the remote
/// one: `and_local: true` is the one of the same name, a string names it
/// (a local branch may track a remote branch of another name).
pub(crate) fn local_twin(args: &Value, name: &str) -> Option<String> {
    match args.get("and_local")? {
        Value::Bool(true) => Some(name.to_string()),
        Value::String(local) => Some(local.clone()),
        _ => None,
    }
}
