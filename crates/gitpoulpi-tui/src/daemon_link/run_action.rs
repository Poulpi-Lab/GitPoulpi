use super::*;
use crate::redraw::NotifySend as _;
use gitpoulpi_i18n::msg::link as l;

pub(super) async fn run_action(
    conn: &LinkConn,
    action: Action,
    action_error_tx: &std_mpsc::Sender<String>,
    action_message_tx: &std_mpsc::Sender<String>,
    push_rejected_tx: &std_mpsc::Sender<PushRejection>,
    autostash_tx: &std_mpsc::Sender<AutoStashNeeded>,
    activity: &Arc<crate::activity::Activity>,
) -> bool {
    // The Activity Log's timing (`log.timings`): from here to the daemon's
    // answer, test delay included.
    let started = std::time::Instant::now();
    // Test hook: makes every action take at least this long, so a test can
    // watch the in-flight feedback of an operation that is normally fast.
    if let Some(ms) = std::env::var("GITPOULPI_TEST_ACTION_DELAY_MS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
    {
        tokio::time::sleep(Duration::from_millis(ms)).await;
    }
    let original = action.clone();
    let result = dispatch(
        conn,
        action,
        action_error_tx,
        action_message_tx,
        push_rejected_tx,
        activity,
    )
    .await;
    match result {
        Err(CoreError::PushRejected(rejection)) => {
            crate::activity_log::report_rejection(
                &original,
                &rejection,
                &conn.repo.root,
                started.elapsed(),
            );
            let _ = push_rejected_tx.send_notify(rejection);
            false
        }
        // `general.auto_stash = ask`: nothing changed, the render loop asks.
        Err(CoreError::NeedsAutoStash { operation })
            if !matches!(original, Action::AutoStashed(_)) =>
        {
            let _ = autostash_tx.send_notify(AutoStashNeeded {
                operation,
                action: original,
            });
            false
        }
        // Sensitive files in a stage: nothing was staged, the render loop
        // lists them and asks (`stage_guard_dialog`); no error line.
        Err(CoreError::SensitiveStage { files })
            if !matches!(original, Action::AllowSensitive(_)) =>
        {
            crate::stage_guard_dialog::ask(conn.repo.root.clone(), original, files);
            false
        }
        // A pending auto-stash: point at the banner, not at CLI commands.
        Err(CoreError::PendingAutoStash {
            operation, stash, ..
        }) => {
            let _ = action_error_tx.send_notify(error_text::pending_autostash(&operation, stash.0));
            false
        }
        Err(e) => {
            crate::activity_log::report_failure(&original, &e, &conn.repo.root, started.elapsed());
            let _ = action_error_tx.send_notify(error_text::error_text(&e));
            false
        }
        Ok(()) => {
            crate::activity_log::report_success(&original, &conn.repo.root, started.elapsed());
            // Say, every time, that it can be taken back.
            if let Some(done) = original.undoable_done_text() {
                let _ = action_message_tx.send_notify(done);
            }
            true
        }
    }
}

tokio::task_local! {
    /// Set while an [`Action::AutoStashed`] runs: every request it sends
    /// carries `autostash: "always"` (see [`autostash_args`]).
    static AUTOSTASH: bool;
    /// Set while an [`Action::AllowSensitive`] runs: its stage request
    /// carries `allow_sensitive: true`.
    static ALLOW_SENSITIVE: bool;
}

/// `args` as the daemon should get them: with `autostash: "always"` when
/// this action is the answer to the auto-stash dialog. The daemon only
/// reads the key for checkout, merge, rebase and pull.
fn autostash_args(mut args: serde_json::Value) -> serde_json::Value {
    if AUTOSTASH.try_with(|on| *on).unwrap_or(false) {
        args["autostash"] = serde_json::json!("always");
    }
    // The daemon only reads this key for the three stage operations.
    if ALLOW_SENSITIVE.try_with(|on| *on).unwrap_or(false) {
        args["allow_sensitive"] = serde_json::json!(true);
    }
    args
}

/// Sends one action's request(s) and turns the answers into status lines.
/// The plain one-request actions are `Action::request`; everything here
/// reads the answer, chains requests or reports an outcome.
async fn dispatch(
    conn: &LinkConn,
    action: Action,
    action_error_tx: &std_mpsc::Sender<String>,
    action_message_tx: &std_mpsc::Sender<String>,
    push_rejected_tx: &std_mpsc::Sender<PushRejection>,
    activity: &Arc<crate::activity::Activity>,
) -> GitResult<()> {
    if let Action::AutoStashed(inner) = action {
        return AUTOSTASH
            .scope(
                true,
                Box::pin(dispatch(
                    conn,
                    *inner,
                    action_error_tx,
                    action_message_tx,
                    push_rejected_tx,
                    activity,
                )),
            )
            .await;
    }
    if let Action::AllowSensitive(inner) = action {
        return ALLOW_SENSITIVE
            .scope(
                true,
                Box::pin(dispatch(
                    conn,
                    *inner,
                    action_error_tx,
                    action_message_tx,
                    push_rejected_tx,
                    activity,
                )),
            )
            .await;
    }
    // Push on every commit (Settings > Commit): the commit first, and only
    // when it landed the push, as the usual push request (a tracked transfer).
    // A rejection takes the push-rejected path like any push.
    if let Action::Commit { push: true, .. } = &action {
        let (kind, args) = action.request().expect("a commit is one request");
        conn.call::<gitpoulpi_core::domain::Oid>(kind, args).await?;
        return push_after_commit(conn, action_message_tx).await;
    }
    if let Action::TreatAs {
        paths,
        scope,
        treat,
        file,
    } = &action
    {
        let done = conn
            .call::<gitpoulpi_core::domain::AttributesWrite>(
                OperationKind::GitattributesTreat,
                serde_json::json!({
                    "paths": paths, "scope": scope, "treat": treat, "file": file,
                }),
            )
            .await?;
        // The status line already said what was being written; this says
        // what the write did not achieve.
        let note = if !done.effective {
            Some(l::treat_overridden(&done.file))
        } else if !done.changed {
            Some(l::treat_unchanged(&done.file))
        } else {
            None
        };
        if let Some(note) = note {
            let _ = action_message_tx.send_notify(note);
        }
        return Ok(());
    }
    if let Action::ConvertFiles { paths, op } = &action {
        let report = conn
            .call::<gitpoulpi_core::textconv::ConvertReport>(
                OperationKind::ConvertFiles,
                serde_json::json!({ "paths": paths, "op": op }),
            )
            .await?;
        let _ = action_message_tx.send_notify(super::convert_summary::text(&report));
        // Files that could not be read or written are a failure (the card);
        // the others were converted and stay so (one Ctrl+Z puts all back).
        return match report.failed.first() {
            Some((path, why)) => Err(CoreError::Backend(l::convert_failed(
                report.failed.len() as u64,
                path,
                why,
            ))),
            None => Ok(()),
        };
    }
    if let Some((command @ OperationKind::StashPush, args)) = action.request() {
        // `null`: nothing to stash, so no stash was made and there is no undo.
        let stash = conn
            .call::<Option<gitpoulpi_core::domain::StashId>>(command, autostash_args(args))
            .await?;
        if stash.is_none() {
            let _ = action_message_tx.send_notify(
                gitpoulpi_i18n::msg::ai_ui::NO_CHANGES_STASH
                    .get()
                    .to_string(),
            );
        }
        return Ok(());
    }
    if let Some((command, args)) = action.request() {
        return conn
            .call::<serde::de::IgnoredAny>(command, autostash_args(args))
            .await
            .map(|_| ());
    }
    match action {
        Action::Undo { expect, confirmed } => {
            undo_redo_now(
                conn,
                OperationKind::Undo,
                expect,
                confirmed,
                action_message_tx,
                activity,
            )
            .await
        }
        Action::ReflogRestore { refname, oid } => {
            conn.call::<gitpoulpi_core::domain::RestoreOutcome>(
                OperationKind::ReflogRestore,
                serde_json::json!({
                    "ref": refname, "oid": oid, "dry_run": false, "confirmed": true,
                }),
            )
            .await?;
            let key = crate::keymap::current().shown(crate::keymap::Action::GlobalUndo);
            let _ = action_message_tx.send_notify(gitpoulpi_i18n::msg::undo_safety::restored_ref(
                refname.trim_start_matches("refs/heads/"),
                key,
            ));
            Ok(())
        }
        Action::UndoForget { expect } => {
            let outcome = conn
                .call::<gitpoulpi_core::domain::UndoOutcome>(
                    OperationKind::Undo,
                    serde_json::json!({ "expect": expect, "forget": true, "confirmed": true }),
                )
                .await?;
            let _ = action_message_tx.send_notify(gitpoulpi_i18n::msg::undo_journal::forgot(
                &outcome.entry.label,
            ));
            Ok(())
        }
        Action::AutoStashStartOver { confirmed } => {
            undo_redo_now(
                conn,
                OperationKind::AutoStashStartOver,
                None,
                confirmed,
                action_message_tx,
                activity,
            )
            .await
        }
        Action::Redo { expect, confirmed } => {
            undo_redo_now(
                conn,
                OperationKind::Redo,
                expect,
                confirmed,
                action_message_tx,
                activity,
            )
            .await
        }
        Action::ComposeCommits { commits } => {
            // One daemon request, so the whole plan is one lock hold and
            // one undo entry (and no other client can interleave).
            let result = conn
                .call::<Vec<gitpoulpi_core::domain::Oid>>(
                    OperationKind::ComposeCommits,
                    serde_json::json!({
                        "commits": commits
                            .iter()
                            .map(|c| serde_json::json!({ "message": c.message, "paths": c.paths }))
                            .collect::<Vec<_>>(),
                        "opts": CommitOptions {
                            amend: false,
                            sign: false,
                            no_verify: false,
                            author: crate::live_settings::active_profile_author(),
                        },
                    }),
                )
                .await;
            match result {
                Ok(made) => {
                    let _ = action_message_tx.send_notify(l::created_commits(made.len() as u64));
                    Ok(())
                }
                // The daemon names how far it got ("stopped after 2 of 3").
                Err(e) => Err(e),
            }
        }
        Action::Fetch(remote) => {
            // `general.auto_prune`, read live -- see
            // `crate::live_settings::auto_prune`'s doc for why this and
            // `FetchAll` are the two paths it governs.
            reported::<gitpoulpi_core::domain::FetchOutcome>(
                conn,
                action_message_tx,
                OperationKind::Fetch,
                serde_json::json!({
                    "remote": remote,
                    "opts": crate::live_settings::fetch_options(),
                }),
                |o| l::fetched(o.updated_refs.len() as u64, &remote),
            )
            .await
        }
        Action::FetchAll => {
            reported::<gitpoulpi_core::domain::FetchOutcome>(
                conn,
                action_message_tx,
                OperationKind::FetchAll,
                {
                    let mut args = serde_json::json!({
                        "opts": crate::live_settings::fetch_options(),
                    });
                    // The window's own fetch: the daemon skips it while the
                    // repository is busy with a transfer.
                    if crate::activity_log::background_fetch_pending() {
                        args["auto"] = serde_json::json!(true);
                    }
                    args
                },
                |o| l::fetched_all(o.updated_refs.len() as u64),
            )
            .await
        }
        Action::Pull(opts) => {
            reported::<gitpoulpi_core::domain::PullOutcome>(
                conn,
                action_message_tx,
                OperationKind::Pull,
                autostash_args(serde_json::json!({ "opts": opts })),
                describe_pull,
            )
            .await
        }
        Action::Push { remote, opts } => push_now(conn, &remote, &opts, action_message_tx).await,
        // Sequenced here, on the link's own task, so the push only
        // starts once the pull really succeeded -- and a pull that stops
        // on conflicts leaves the push unsent with the reason on screen.
        Action::FetchThenForcePush { remote, branch } => {
            let fetched = conn
                .call::<gitpoulpi_core::domain::FetchOutcome>(
                    OperationKind::Fetch,
                    serde_json::json!({ "remote": remote, "opts": FetchOptions::default() }),
                )
                .await;
            match fetched {
                Ok(_) => {
                    push_now(
                        conn,
                        &remote,
                        &PushOptions {
                            force_with_lease: true,
                            set_upstream: false,
                            branch: Some(branch),
                        },
                        action_message_tx,
                    )
                    .await
                }
                Err(e) => Err(e),
            }
        }
        Action::PullThenPush { remote, branch } => {
            let pulled = conn
                .call::<gitpoulpi_core::domain::PullOutcome>(
                    OperationKind::Pull,
                    serde_json::json!({ "opts": PullOptions {
                    mode: gitpoulpi_core::domain::PullMode::Merge,
                    remote: Some(remote.clone()),
                } }),
                )
                .await;
            match pulled {
                Ok(gitpoulpi_core::domain::PullOutcome::Conflicted) => {
                    let _ =
                        action_message_tx.send_notify(l::PULL_CONFLICTED_PUSH.get().to_string());
                    Ok(())
                }
                Ok(_) => {
                    push_now(
                        conn,
                        &remote,
                        &PushOptions {
                            branch: Some(branch),
                            ..PushOptions::default()
                        },
                        action_message_tx,
                    )
                    .await
                }
                Err(e) => Err(e),
            }
        }
        Action::PushForReview {
            remote,
            branch,
            topic,
        } => conn
            .call::<gitpoulpi_core::domain::PushOutcome>(
                OperationKind::PushForReview,
                serde_json::json!({ "remote": remote, "branch": branch, "topic": topic }),
            )
            .await
            .map(|_| ()),
        Action::CheckoutPullRequestHead {
            remote,
            remote_ref,
            branch,
        } => {
            let fetched = conn
                .call::<gitpoulpi_core::domain::FetchOutcome>(
                    OperationKind::FetchRefAsBranch,
                    serde_json::json!({
                        "remote": remote,
                        "remote_ref": remote_ref,
                        "branch": branch,
                    }),
                )
                .await;
            match fetched {
                Ok(_) => {
                    conn.call::<()>(
                        OperationKind::Checkout,
                        serde_json::json!({ "target": CheckoutTarget::Branch(branch) }),
                    )
                    .await
                }
                Err(e) => Err(e),
            }
        }
        Action::CheckoutThen { branch, then } => {
            let checked_out = conn
                .call::<()>(
                    OperationKind::Checkout,
                    autostash_args(serde_json::json!({ "target": CheckoutTarget::Branch(branch) })),
                )
                .await;
            match checked_out {
                // The follow-up reports its own outcome and errors.
                Ok(()) => {
                    // The follow-up's own refusal under `general.auto_stash =
                    // ask` is caught here and handed up as this action's
                    // refusal, so the dialog opens for the whole
                    // `CheckoutThen`: answering yes re-runs it, the checkout
                    // of the branch already checked out is a no-op and the
                    // follow-up then runs with `autostash: "always"`.
                    let (nested_tx, nested_rx) = std_mpsc::channel();
                    let _ = Box::pin(run_action(
                        conn,
                        *then,
                        action_error_tx,
                        action_message_tx,
                        push_rejected_tx,
                        &nested_tx,
                        activity,
                    ))
                    .await;
                    match nested_rx.try_recv() {
                        Ok(needed) => Err(CoreError::NeedsAutoStash {
                            operation: needed.operation,
                        }),
                        Err(_) => Ok(()),
                    }
                }
                Err(e) => Err(e),
            }
        }
        // Creates the branch and checks it out. A
        // checkout refused by the working tree is reported, saying the
        // branch itself was created.
        Action::BranchCreate { name, at } => {
            let created = conn
                .call::<()>(
                    OperationKind::BranchCreate,
                    serde_json::json!({ "name": name, "at": at }),
                )
                .await;
            match created {
                Ok(()) => conn
                    .call::<()>(
                        OperationKind::Checkout,
                        serde_json::json!({ "target": CheckoutTarget::Branch(name.clone()) }),
                    )
                    .await
                    .map_err(|e| {
                        CoreError::Backend(l::branch_created_checkout_failed(
                            &name,
                            error_text::error_text(&e),
                        ))
                    }),
                Err(e) => Err(e),
            }
        }
        Action::TagAnnotate { name, message } => {
            reported::<()>(
                conn,
                action_message_tx,
                OperationKind::TagAnnotate,
                serde_json::json!({ "name": name, "message": message }),
                |_| l::tag_annotated(&name),
            )
            .await
        }
        // Network, so this arm runs on a spawned task (see
        // `Action::is_network`) and the input loop keeps drawing while
        // it is in flight. The final message is what the user reads --
        // "pushed nothing" is a real answer and says so, rather than
        // leaving the last "pushing..." on screen.
        Action::TagPush { remote, selection } => {
            let all = matches!(selection, TagSelection::AllUnpushed);
            reported::<gitpoulpi_core::domain::PushOutcome>(
                conn,
                action_message_tx,
                OperationKind::TagPush,
                serde_json::json!({ "remote": remote, "selection": selection }),
                move |outcome| {
                    if outcome.pushed_refs.is_empty() {
                        if all {
                            l::tags_all_pushed(&remote)
                        } else {
                            l::nothing_to_push(&remote)
                        }
                    } else {
                        l::pushed_refs(outcome.pushed_refs.join(", "), &remote)
                    }
                },
            )
            .await
        }
        Action::TagDeleteRemote { remote, name } => {
            reported::<()>(
                conn,
                action_message_tx,
                OperationKind::TagDeleteRemote,
                serde_json::json!({ "remote": remote, "name": name }),
                move |_| l::tag_deleted_remote(&name, &remote),
            )
            .await
        }
        Action::TagDeleteBoth { remote, name } => {
            // One request, so one undo entry: the daemon deletes on the
            // remote first (refused: no rights, offline, nothing local was
            // lost), then the local tag.
            let outcome = conn
                .call::<()>(
                    OperationKind::TagDeleteRemote,
                    serde_json::json!({ "remote": remote, "name": name, "and_local": true }),
                )
                .await;
            report(action_message_tx, outcome, move |_| {
                gitpoulpi_i18n::msg::tag_where::msg_deleted_both(&name, &remote)
            })
        }
        Action::TagDeleteKeepRemote { name, remotes } => {
            let outcome = conn
                .call::<()>(
                    OperationKind::TagDelete,
                    serde_json::json!({ "name": name }),
                )
                .await;
            report(action_message_tx, outcome, move |_| {
                gitpoulpi_i18n::msg::tag_where::msg_deleted_local_still(&name, remotes.join(", "))
            })
        }
        Action::TagFetch { remote, name } => {
            reported::<()>(
                conn,
                action_message_tx,
                OperationKind::TagFetch,
                serde_json::json!({ "remote": remote, "name": name }),
                move |_| gitpoulpi_i18n::msg::tag_where::msg_fetched(&name, &remote),
            )
            .await
        }
        Action::BranchDeleteRemote {
            remote,
            name,
            and_local,
        } => {
            // One request (one undo entry): the local branch goes only once
            // the remote accepted the deletion.
            let both = and_local.is_some();
            let outcome = conn
                .call::<()>(
                    OperationKind::BranchDeleteRemote,
                    serde_json::json!({ "remote": remote, "name": name, "and_local": and_local }),
                )
                .await;
            report(action_message_tx, outcome, move |_| {
                if both {
                    l::branch_deleted_both(&name, &remote)
                } else {
                    l::branch_deleted_remote(&name, &remote)
                }
            })
        }
        Action::Merge(from) => {
            // Settings > Merge options > Squash (this repository's own
            // choice, read here, off the render thread): stage the changes,
            // make no merge commit. Nothing moves, so it is not journaled.
            let squash = crate::commit_settings::repo_preferences(&conn.repo.root)
                .squash_merge
                .unwrap_or(false);
            let branch = match &from.kind {
                gitpoulpi_core::domain::RefKind::LocalBranch { name }
                | gitpoulpi_core::domain::RefKind::Tag { name } => name.clone(),
                gitpoulpi_core::domain::RefKind::RemoteBranch { remote, name } => {
                    format!("{remote}/{name}")
                }
            };
            let outcome = conn
                .call::<gitpoulpi_core::domain::MergeOutcome>(
                    OperationKind::Merge,
                    autostash_args(serde_json::json!({
                        "from": from,
                        "opts": MergeOptions {
                            squash,
                            no_commit: squash,
                            ..MergeOptions::default()
                        },
                    })),
                )
                .await;
            match outcome {
                Ok(outcome) => {
                    // `general.delete_orig_files_after_merge`: only on a
                    // merge that actually landed cleanly. A conflicted
                    // merge leaves the working tree mid-resolution, and a
                    // merge driver's own `.orig` backup there is exactly
                    // what a user reaches for -- deleting it out from
                    // under an unresolved conflict would destroy the one
                    // thing this setting exists to clean up *after*.
                    let cleaned = if crate::live_settings::delete_orig_after_merge()
                        && matches!(
                            outcome,
                            gitpoulpi_core::domain::MergeOutcome::FastForwarded(_)
                                | gitpoulpi_core::domain::MergeOutcome::Merged(_)
                        ) {
                        cleanup_orig_files(&conn.repo.root)
                    } else {
                        0
                    };
                    let mut message = match outcome {
                        gitpoulpi_core::domain::MergeOutcome::FastForwarded(oid) => {
                            l::merged_fast_forward(&oid.to_hex()[..7])
                        }
                        gitpoulpi_core::domain::MergeOutcome::Merged(_) if squash => {
                            l::merged_squashed(&branch)
                        }
                        gitpoulpi_core::domain::MergeOutcome::Merged(oid) => {
                            l::merged_new_commit(&oid.to_hex()[..7])
                        }
                        gitpoulpi_core::domain::MergeOutcome::UpToDate => {
                            l::UP_TO_DATE.get().to_string()
                        }
                        gitpoulpi_core::domain::MergeOutcome::Conflicted(files) => {
                            l::merge_conflicts(files.join(", "))
                        }
                    };
                    if cleaned > 0 {
                        message = l::orig_cleaned(cleaned as u64, message);
                    }
                    let _ = action_message_tx.send_notify(message);
                    Ok(())
                }
                Err(e) => Err(e),
            }
        }
        // The four history operations below mirror
        // `crates/gitpoulpi-cli/src/commands.rs` arm for arm -- same
        // `OperationKind`, same JSON keys, same default options -- so the
        // two frontends can never ask the daemon for subtly different
        // things. Each reports its outcome through `action_message_tx`
        // because "it conflicted" is not an error: the repo is now in a
        // state the user has to act on, and the Status pane's conflict
        // rows are where they act.
        Action::Rebase(onto) => {
            reported::<RebaseOutcome>(
                conn,
                action_message_tx,
                OperationKind::Rebase,
                autostash_args(
                    serde_json::json!({ "onto": onto, "opts": RebaseOptions::default() }),
                ),
                describe_rebase(l::REBASED),
            )
            .await
        }
        Action::RebaseInteractive(request) => {
            reported::<RebaseOutcome>(
                conn,
                action_message_tx,
                OperationKind::RebaseInteractive,
                serde_json::json!({ "request": request }),
                describe_rebase(l::REBASED),
            )
            .await
        }
        Action::RebaseContinue => {
            reported::<RebaseOutcome>(
                conn,
                action_message_tx,
                OperationKind::RebaseContinue,
                serde_json::json!({}),
                describe_rebase(l::REBASE_COMPLETED),
            )
            .await
        }
        Action::RebaseSkip => {
            reported::<RebaseOutcome>(
                conn,
                action_message_tx,
                OperationKind::RebaseSkip,
                serde_json::json!({}),
                describe_rebase(l::REBASE_COMPLETED),
            )
            .await
        }
        Action::RebaseAbort => {
            reported::<()>(
                conn,
                action_message_tx,
                OperationKind::RebaseAbort,
                serde_json::json!({}),
                |()| l::REBASE_ABORTED.get().to_string(),
            )
            .await
        }
        Action::AmContinue | Action::AmSkip => {
            let kind = if matches!(action, Action::AmContinue) {
                OperationKind::AmContinue
            } else {
                OperationKind::AmSkip
            };
            reported::<gitpoulpi_core::domain::AmOutcome>(
                conn,
                action_message_tx,
                kind,
                serde_json::json!({}),
                |outcome| {
                    match outcome {
                        gitpoulpi_core::domain::AmOutcome::Completed => {
                            gitpoulpi_i18n::msg::stuck_states::AM_DONE.get()
                        }
                        gitpoulpi_core::domain::AmOutcome::Stopped => {
                            gitpoulpi_i18n::msg::stuck_states::AM_STOPPED.get()
                        }
                    }
                    .to_string()
                },
            )
            .await
        }
        Action::AmAbort => {
            reported::<()>(
                conn,
                action_message_tx,
                OperationKind::AmAbort,
                serde_json::json!({}),
                |()| {
                    gitpoulpi_i18n::msg::stuck_states::AM_ABORTED
                        .get()
                        .to_string()
                },
            )
            .await
        }
        Action::IndexLockRemove => {
            reported::<()>(
                conn,
                action_message_tx,
                OperationKind::IndexLockRemove,
                serde_json::json!({}),
                |()| {
                    gitpoulpi_i18n::msg::stuck_states::LOCK_REMOVED
                        .get()
                        .to_string()
                },
            )
            .await
        }
        Action::OperationAbort => {
            reported::<()>(
                conn,
                action_message_tx,
                OperationKind::OperationAbort,
                serde_json::json!({}),
                |()| l::OPERATION_ABORTED.get().to_string(),
            )
            .await
        }
        Action::OperationSkip => {
            reported::<()>(
                conn,
                action_message_tx,
                OperationKind::OperationSkip,
                serde_json::json!({}),
                |()| l::OPERATION_SKIPPED.get().to_string(),
            )
            .await
        }
        Action::CherryPick(commit) => {
            reported::<CherryPickOutcome>(
                conn,
                action_message_tx,
                OperationKind::CherryPick,
                serde_json::json!({
                    "commits": [commit],
                    "opts": CherryPickOptions::default(),
                }),
                |outcome| match outcome {
                    CherryPickOutcome::Completed(applied) => l::cherry_picked(
                        applied
                            .iter()
                            .map(|o| short(*o))
                            .collect::<Vec<_>>()
                            .join(", "),
                    ),
                    CherryPickOutcome::Conflicted { paused_at, .. } => {
                        l::cherry_pick_paused(short(paused_at))
                    }
                },
            )
            .await
        }
        Action::Revert { commit, mainline } => {
            reported::<RevertOutcome>(
                conn,
                action_message_tx,
                OperationKind::Revert,
                serde_json::json!({ "commit": commit, "mainline": mainline }),
                |outcome| match outcome {
                    RevertOutcome::Completed(oid) => l::reverted(short(oid)),
                    RevertOutcome::Conflicted => l::REVERT_CONFLICTED.get().to_string(),
                },
            )
            .await
        }
        Action::Reset { target, mode } => {
            reported::<()>(
                conn,
                action_message_tx,
                OperationKind::Reset,
                serde_json::json!({ "target": target, "mode": mode }),
                |()| l::reset_done(format!("{mode:?}"), short(target)),
            )
            .await
        }
        Action::Reword { commit, message } => {
            reported::<RewordOutcome>(
                conn,
                action_message_tx,
                OperationKind::Reword,
                serde_json::json!({ "commit": commit, "message": message }),
                |outcome| reword_done_message(commit, &outcome),
            )
            .await
        }
        Action::AmendCommit { commit, amend } => {
            reported::<RewordOutcome>(
                conn,
                action_message_tx,
                OperationKind::AmendCommit,
                serde_json::json!({ "commit": commit, "amend": amend }),
                |outcome| amend_done_message(commit, &outcome),
            )
            .await
        }
        Action::SplitCommit { req } => {
            let commit = req.commit;
            let root = conn.repo.root.clone();
            reported::<SplitOutcome>(
                conn,
                action_message_tx,
                OperationKind::SplitCommit,
                serde_json::json!({ "req": req }),
                |outcome| {
                    if let Some(newest) = outcome.created.last() {
                        note_split_created(&root, *newest);
                    }
                    split_done_message(commit, &outcome)
                },
            )
            .await
        }
        Action::ResolveConflict { path, side } => {
            let shown = path.display().to_string();
            reported::<()>(
                conn,
                action_message_tx,
                OperationKind::ResolveConflict,
                serde_json::json!({
                    "path": path,
                    "resolution": ConflictResolution::TakeSide(side),
                }),
                |()| {
                    l::resolved_with(
                        &shown,
                        match side {
                            ConflictSide::Ours => l::SIDE_OURS,
                            ConflictSide::Theirs => l::SIDE_THEIRS,
                            ConflictSide::Base => l::SIDE_BASE,
                        },
                    )
                },
            )
            .await
        }
        Action::RemoteAdd { name, url } => {
            reported::<()>(
                conn,
                action_message_tx,
                OperationKind::RemoteAdd,
                serde_json::json!({ "name": name, "url": url }),
                |()| l::remote_added(&name),
            )
            .await
        }
        Action::RemoteRemove(name) => {
            reported::<()>(
                conn,
                action_message_tx,
                OperationKind::RemoteRemove,
                serde_json::json!({ "name": name }),
                |()| l::remote_removed(&name),
            )
            .await
        }
        Action::RemoteRename { old_name, new_name } => {
            reported::<()>(
                conn,
                action_message_tx,
                OperationKind::RemoteRename,
                serde_json::json!({ "old_name": old_name, "new_name": new_name }),
                |()| l::remote_renamed(&old_name, &new_name),
            )
            .await
        }
        Action::RemoteSetUrl { name, url } => {
            reported::<()>(
                conn,
                action_message_tx,
                OperationKind::RemoteSetUrl,
                serde_json::json!({ "name": name, "url": url }),
                |()| l::remote_fetches_from(&name, &url),
            )
            .await
        }
        Action::SparseSet { dirs } => {
            reported::<()>(
                conn,
                action_message_tx,
                OperationKind::SparseSet,
                serde_json::json!({ "dirs": dirs }),
                |()| gitpoulpi_i18n::msg::sparse::applied(dirs.join(" ")),
            )
            .await
        }
        Action::SparseDisable => {
            reported::<()>(
                conn,
                action_message_tx,
                OperationKind::SparseDisable,
                serde_json::json!({}),
                |()| gitpoulpi_i18n::msg::sparse::DISABLED.get().to_string(),
            )
            .await
        }
        Action::SetRepoConfig { key, value } => {
            reported::<()>(
                conn,
                action_message_tx,
                OperationKind::SetRepoConfig,
                serde_json::json!({ "key": key, "value": value }),
                |()| match &value {
                    Some(v) => l::repo_config_set(&key, v),
                    None => l::repo_config_unset(&key),
                },
            )
            .await
        }
        Action::RemoteSetPushUrl { name, url } => {
            reported::<()>(
                conn,
                action_message_tx,
                OperationKind::RemoteSetPushUrl,
                serde_json::json!({ "name": name, "url": url }),
                |()| match &url {
                    Some(url) => l::remote_pushes_to(&name, url),
                    None => l::remote_pushes_to_fetch_url(&name),
                },
            )
            .await
        }
        Action::BranchSetUpstream { branch, upstream } => {
            reported::<()>(
                conn,
                action_message_tx,
                OperationKind::BranchSetUpstream,
                serde_json::json!({ "branch": branch, "upstream": upstream }),
                |()| match &upstream {
                    Some(up) => l::upstream_set(&branch, up),
                    None => l::upstream_cleared(&branch),
                },
            )
            .await
        }
        Action::FastForward {
            branch,
            target,
            target_label,
        } => {
            reported::<FastForwardOutcome>(
                conn,
                action_message_tx,
                OperationKind::FastForward,
                serde_json::json!({ "branch": branch, "target": target }),
                |outcome| match outcome {
                    FastForwardOutcome::FastForwarded { from, to } => {
                        l::fast_forwarded(&branch, &target_label, short(from), short(to))
                    }
                    FastForwardOutcome::UpToDate => l::already_contains(&branch, &target_label),
                },
            )
            .await
        }
        Action::WorktreeAdd { path, opts } => {
            reported::<Worktree>(
                conn,
                action_message_tx,
                OperationKind::WorktreeAdd,
                serde_json::json!({ "path": path, "opts": opts }),
                |added| {
                    l::worktree_added(
                        added.path.display(),
                        added
                            .branch
                            .clone()
                            .unwrap_or_else(|| l::DETACHED_HEAD.get().to_string()),
                    )
                },
            )
            .await
        }
        Action::PatchCreate { source, dest } => {
            reported::<PatchCreated>(
                conn,
                action_message_tx,
                OperationKind::PatchCreate,
                serde_json::json!({ "source": source, "dest": dest }),
                |created| l::patch_written(created.commits as u64, created.path.display()),
            )
            .await
        }
        Action::LfsInstall => {
            reported::<()>(
                conn,
                action_message_tx,
                OperationKind::LfsInstall,
                serde_json::json!({ "scope": gitpoulpi_core::domain::LfsInstallScope::Repository }),
                |_| l::LFS_INSTALLED.get().to_string(),
            )
            .await
        }
        Action::LfsTrack { patterns, opts } => {
            reported::<Vec<gitpoulpi_core::domain::LfsPattern>>(
                conn,
                action_message_tx,
                OperationKind::LfsTrack,
                serde_json::json!({ "patterns": patterns, "opts": opts }),
                move |_| l::lfs_tracking(patterns.join(", ")),
            )
            .await
        }
        Action::LfsUntrack(patterns) => {
            reported::<Vec<gitpoulpi_core::domain::LfsPattern>>(
                conn,
                action_message_tx,
                OperationKind::LfsUntrack,
                serde_json::json!({ "patterns": patterns }),
                move |_| l::lfs_untracked(patterns.join(", ")),
            )
            .await
        }
        Action::LfsFetch(remote) => {
            reported::<()>(
                conn,
                action_message_tx,
                OperationKind::LfsFetch,
                serde_json::json!({ "remote": remote }),
                move |_| l::lfs_fetched(&remote),
            )
            .await
        }
        Action::LfsPull(remote) => {
            reported::<()>(
                conn,
                action_message_tx,
                OperationKind::LfsPull,
                serde_json::json!({ "remote": remote }),
                move |_| l::lfs_pulled(&remote),
            )
            .await
        }
        Action::LfsPush(remote) => {
            reported::<()>(
                conn,
                action_message_tx,
                OperationKind::LfsPush,
                serde_json::json!({ "remote": remote }),
                move |_| l::lfs_pushed(&remote),
            )
            .await
        }
        Action::LfsFetchAll(remote) => {
            reported::<()>(
                conn,
                action_message_tx,
                OperationKind::LfsFetch,
                serde_json::json!({ "remote": remote, "opts": { "all_refs": true } }),
                move |_| l::lfs_fetched(&remote),
            )
            .await
        }
        Action::LfsPushAll(remote) => {
            reported::<()>(
                conn,
                action_message_tx,
                OperationKind::LfsPush,
                serde_json::json!({ "remote": remote, "opts": { "all_refs": true } }),
                move |_| gitpoulpi_i18n::msg::lfs_ui::pushed_all(&remote),
            )
            .await
        }
        Action::LfsCheckout => {
            reported::<()>(
                conn,
                action_message_tx,
                OperationKind::LfsCheckout,
                serde_json::json!({}),
                |_| gitpoulpi_i18n::msg::lfs_ui::CHECKED_OUT.get().to_string(),
            )
            .await
        }
        Action::LfsPrune => {
            reported::<()>(
                conn,
                action_message_tx,
                OperationKind::LfsPrune,
                serde_json::json!({}),
                |_| l::LFS_PRUNED.get().to_string(),
            )
            .await
        }
        // The list itself arrives with the next refresh (see
        // `lfs_ui::LfsGate::locks_due`); this action only registers the
        // remote, which the loop did before spawning it.
        Action::LfsLocks(remote) => {
            let _ = action_message_tx.send_notify(l::lfs_reading_locks(&remote));
            Ok(())
        }
        Action::LfsLock { remote, path } => {
            reported::<gitpoulpi_core::domain::LfsLock>(
                conn,
                action_message_tx,
                OperationKind::LfsLock,
                serde_json::json!({ "remote": remote, "path": path }),
                move |_| l::lfs_locked(path.display()),
            )
            .await
        }
        Action::LfsUnlock {
            remote,
            path,
            force,
        } => {
            reported::<()>(
                conn,
                action_message_tx,
                OperationKind::LfsUnlock,
                serde_json::json!({ "remote": remote, "path": path, "force": force }),
                move |_| l::lfs_unlocked(path.display()),
            )
            .await
        }
        Action::PatchApply { path } => {
            reported::<PatchApplied>(
                conn,
                action_message_tx,
                OperationKind::PatchApply,
                serde_json::json!({ "patch": path }),
                |applied| l::patch_applied(applied.files.len() as u64),
            )
            .await
        }
        _ => unreachable!("an action `request` answers is not dispatched"),
    }
}

/// `Undo` / `Redo`. A risky one answers `NeedsConfirmation`: that is not
/// a failure, it is handed to the render loop as a question
/// (`JournalCell::ask`) and nothing was changed. The busy mark set at
/// submit time is cleared here, whatever the answer.
#[allow(clippy::too_many_arguments)]
pub(super) async fn undo_redo_now(
    conn: &LinkConn,
    command: OperationKind,
    expect: Option<u64>,
    confirmed: bool,
    action_message_tx: &std_mpsc::Sender<String>,
    activity: &Arc<crate::activity::Activity>,
) -> GitResult<()> {
    use gitpoulpi_core::domain::{Direction, UndoOutcome};
    let direction = if command != OperationKind::Redo {
        Direction::Undo
    } else {
        Direction::Redo
    };
    let answer = conn
        .call::<UndoOutcome>(
            command,
            serde_json::json!({ "expect": expect, "dry_run": false, "confirmed": confirmed }),
        )
        .await;
    activity.journal.end();
    match answer {
        Ok(outcome) => {
            let _ = action_message_tx.send_notify(if direction == Direction::Undo {
                l::undid(&outcome.entry.label)
            } else {
                l::redid(&outcome.entry.label)
            });
            Ok(())
        }
        Err(CoreError::NeedsConfirmation(prompt)) => {
            activity.journal.ask(crate::undo_ui::PendingConfirm {
                direction,
                expect,
                prompt,
            });
            Ok(())
        }
        Err(e) => Err(e),
    }
}

/// One push, reported the way every push path reports it.
#[allow(clippy::too_many_arguments)]
pub(super) async fn push_now(
    conn: &LinkConn,
    remote: &str,
    opts: &PushOptions,
    action_message_tx: &std_mpsc::Sender<String>,
) -> GitResult<()> {
    let outcome = conn
        .call::<gitpoulpi_core::domain::PushOutcome>(
            OperationKind::Push,
            serde_json::json!({ "remote": remote, "opts": opts }),
        )
        .await;
    let forced = opts.force_with_lease;
    report(action_message_tx, outcome, |o| {
        if forced {
            l::force_pushed_refs(o.pushed_refs.join(", "), remote)
        } else {
            l::pushed_refs(o.pushed_refs.join(", "), remote)
        }
    })
}

/// The push half of a commit with "Push on every commit" on: the checked-out
/// branch goes to its upstream's remote through the usual push request. A
/// branch with no upstream is not pushed, and the notice says so (the first
/// push of a new branch stays the Push button's, which sets the upstream).
async fn push_after_commit(
    conn: &LinkConn,
    action_message_tx: &std_mpsc::Sender<String>,
) -> GitResult<()> {
    use gitpoulpi_core::domain::{Ref, RefKind};
    let refs = conn
        .call::<Vec<Ref>>(OperationKind::Refs, serde_json::json!({}))
        .await?;
    let head = refs
        .iter()
        .find(|r| r.is_head && matches!(r.kind, RefKind::LocalBranch { .. }));
    let target = head.and_then(|head| {
        let RefKind::LocalBranch { name } = &head.kind else {
            return None;
        };
        let upstream = head.upstream.as_deref()?;
        refs.iter().find_map(|r| match &r.kind {
            RefKind::RemoteBranch { remote, name: n } if format!("{remote}/{n}") == upstream => {
                Some((remote.clone(), name.clone()))
            }
            _ => None,
        })
    });
    let Some((remote, branch)) = target else {
        let _ = action_message_tx.send_notify(l::COMMITTED_NOT_PUSHED.get().to_string());
        return Ok(());
    };
    push_now(
        conn,
        &remote,
        &PushOptions {
            branch: Some(branch),
            ..PushOptions::default()
        },
        action_message_tx,
    )
    .await
}
