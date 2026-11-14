use std::path::PathBuf;

use super::*;
use gitpoulpi_core::domain::{
    BlobSource, CheckoutTarget, CherryPickOptions, CommitOptions, ConflictEntry,
    ConflictResolution, DiffTarget, DiscardTarget, InitOptions, InteractiveRebase, LfsInstallScope,
    LfsTrackOptions, LogOptions, MergeOptions, Oid, OperationKind, PatchSource, RebaseOptions,
    ResetMode, StageTarget, StashId, SubmoduleUpdateOptions, WorktreeAddOptions,
};
use gitpoulpi_core::GitBackend;
use gitpoulpi_ipc::Request;
use serde_json::Value;

/// [`dispatch`] without the journal.
pub(super) fn dispatch_plain(backend: &dyn GitBackend, request: &Request) -> Dispatched {
    // `Discover` is the one *read* with no repo to target -- it is what
    // turns a path into one -- so, like `Clone`, it has to be answered
    // before `repo_ref` rejects the request for having no `repo`.
    if request.command == OperationKind::Discover {
        let path: PathBuf = arg_or_err!(&request.args, "path", request.id);
        return match backend.discover(&path) {
            Ok(found) => match serde_json::to_value(found) {
                Ok(json) => ok(request.id, json, false),
                Err(e) => err(request.id, format!("failed to encode result: {e}"), false),
            },
            Err(e) => err(request.id, e.to_string(), false),
        };
    }

    // `Init` produces a repo rather than targeting one, so it is answered
    // here, like `Discover`, before `repo_ref` rejects the request for
    // having no `repo`. It is not a network operation (unlike `Clone`) and
    // does not broadcast: nothing can be subscribed to a repository that
    // did not exist a moment ago.
    if request.command == OperationKind::Init {
        let path: PathBuf = arg_or_err!(&request.args, "path", request.id);
        let opts: InitOptions = request
            .args
            .get("opts")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default();
        return match backend.init(&path, &opts) {
            Ok(made) => match serde_json::to_value(made) {
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
    let args = &request.args;

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
        OperationKind::Status => to_json!(backend.status(&repo)).map(|mut status| {
            // The push indicator's label, from the private file (no spawn).
            crate::repo_auth::add_summary(&repo.root, &mut status);
            status
        }),
        OperationKind::ConflictEntries => to_json!(backend.conflict_entries(&repo)),
        OperationKind::ConflictShow => {
            let path: PathBuf = arg_or_err!(args, "path", request.id);
            to_json!(backend.merge_document(&repo, &path))
        }
        OperationKind::OperationState => to_json!(backend.operation_state(&repo)),
        OperationKind::OperationAbort => to_json!(backend.operation_abort(&repo)),
        OperationKind::AmContinue => to_json!(backend.am_continue(&repo)),
        OperationKind::AmSkip => to_json!(backend.am_skip(&repo)),
        OperationKind::AmAbort => to_json!(backend.am_abort(&repo)),
        OperationKind::IndexLock => to_json!(backend.index_lock(&repo)),
        OperationKind::IndexLockRemove => to_json!(backend.index_lock_remove(&repo)),
        OperationKind::OperationSkip => to_json!(backend.operation_skip(&repo)),
        OperationKind::ConflictUnresolve => {
            let path: PathBuf = arg_or_err!(args, "path", request.id);
            // The caller's entry, else what the daemon recorded when it
            // resolved the path.
            let given: Option<ConflictEntry> = args
                .get("entry")
                .and_then(|e| serde_json::from_value(e.clone()).ok());
            let Some(entry) =
                given.or_else(|| crate::resolved_record::get(&repo, &path).map(|r| r.entry))
            else {
                return err(
                    request.id,
                    format!(
                        "{} was not resolved while this operation ran, so there is nothing to put back",
                        path.display()
                    ),
                    false,
                );
            };
            let result = backend.unresolve_conflict(&repo, &path, &entry);
            if result.is_ok() {
                crate::resolved_record::forget(&repo, &path);
            }
            to_json!(result)
        }
        OperationKind::Log => {
            let mut opts: LogOptions = match args.get("opts") {
                Some(v) => match serde_json::from_value(v.clone()) {
                    Ok(o) => o,
                    Err(e) => return err(request.id, format!("invalid `opts`: {e}"), false),
                },
                None => LogOptions::default(),
            };
            // `log.use_mailmap` is the daemon's to say, not the client's.
            opts.use_mailmap = crate::mailmap::enabled();
            to_json!(backend.log(&repo, &opts))
        }
        OperationKind::CountCommits => {
            let opts: LogOptions = match args.get("opts") {
                Some(v) => match serde_json::from_value(v.clone()) {
                    Ok(o) => o,
                    Err(e) => return err(request.id, format!("invalid `opts`: {e}"), false),
                },
                None => LogOptions::default(),
            };
            // `log.commit_count`: the client says which way to count (it is
            // part of the cache key, so a change recounts). "libgit2" runs
            // in this process on either backend and spawns nothing.
            if args.get("method").and_then(Value::as_str) == Some("libgit2") {
                to_json!(gitpoulpi_git2_backend::count_commits_in_process(
                    &repo, &opts
                ))
            } else {
                to_json!(backend.count_commits(&repo, &opts))
            }
        }
        OperationKind::Diff => {
            let target: DiffTarget = arg_or_err!(args, "target", request.id);
            // Optional `paths`: only those files' hunks.
            // Absent keeps the whole diff, so older clients are unchanged.
            match args.get("paths") {
                None | Some(Value::Null) => to_json!(backend.diff(&repo, target)),
                Some(v) => match serde_json::from_value::<Vec<PathBuf>>(v.clone()) {
                    Ok(paths) => to_json!(backend.diff_paths(&repo, target, &paths)),
                    Err(e) => return err(request.id, format!("invalid `paths`: {e}"), false),
                },
            }
        }
        OperationKind::DiffFiles => {
            let target: DiffTarget = arg_or_err!(args, "target", request.id);
            to_json!(backend.diff_files(&repo, target))
        }
        OperationKind::CommitStats => {
            let ids: Vec<gitpoulpi_core::domain::Oid> = arg_or_err!(args, "ids", request.id);
            to_json!(backend.commit_stats(&repo, &ids))
        }
        OperationKind::Refs => to_json!(backend.refs(&repo).map(|mut refs| {
            crate::remote_keys::decorate_refs(backend, &repo, &mut refs);
            refs
        })),
        OperationKind::RemoteGroupStatus => {
            let branch: String = arg_or_err!(args, "branch", request.id);
            to_json!(crate::remote_keys::status(backend, &repo, &branch))
        }
        OperationKind::Remotes => to_json!(backend.remotes(&repo)),
        OperationKind::LeftoverRemotes => to_json!(backend.leftover_remotes(&repo)),
        OperationKind::SshCommand => to_json!(backend.ssh_command(&repo)),
        OperationKind::SparseStatus => to_json!(backend.sparse_checkout_status(&repo)),
        OperationKind::Workflow => crate::workflow::read(backend, &repo),
        OperationKind::FlowPlan => crate::flow::plan_read(backend, &repo, args),
        // One key, or (no `key`) every allowed key, as `{key: value|null}`.
        OperationKind::RepoConfig => {
            let key: Option<String> = args
                .get("key")
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .flatten();
            match key {
                Some(key) => to_json!(backend.repo_config(&repo, &key)),
                None => {
                    let mut all = serde_json::Map::new();
                    let mut failed = None;
                    for (key, _) in gitpoulpi_core::repo_config::REPO_CONFIG_KEYS {
                        match backend.repo_config(&repo, key) {
                            Ok(v) => {
                                all.insert((*key).to_string(), serde_json::json!(v));
                            }
                            Err(e) => {
                                failed = Some((e.to_string(), e.typed(), e.detail().cloned()));
                                break;
                            }
                        }
                    }
                    match failed {
                        Some(e) => Err(e),
                        None => Ok(serde_json::Value::Object(all)),
                    }
                }
            }
        }
        OperationKind::Stashes => to_json!(backend.stashes(&repo)),
        OperationKind::Worktrees => to_json!(backend.worktrees(&repo)),
        OperationKind::Submodules => to_json!(backend.submodules(&repo)),
        OperationKind::SubmoduleInit => {
            let paths: Vec<String> = args
                .get("paths")
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_default();
            to_json!(backend.submodule_init(&repo, &paths))
        }
        OperationKind::SubmoduleUpdate => {
            let opts: SubmoduleUpdateOptions = args
                .get("opts")
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_default();
            // May clone: `credentials.https_mode` and the repository's
            // authentication profile, read live.
            match crate::credentials::apply_for(backend, &repo, &request.session) {
                // The grant (an account source) lives until the update is over.
                // A refused host or a failed token command reads as that.
                Ok(grant) => {
                    let result = backend.submodule_update(&repo, &opts);
                    let result = match (&grant, result) {
                        (Some(grant), Err(e)) => Err(grant.explain(e)),
                        (_, other) => other,
                    };
                    to_json!(result)
                }
                Err(error) => Err((error.to_string(), error.typed(), None)),
            }
        }
        OperationKind::Resolve => {
            let rev: String = arg_or_err!(args, "rev", request.id);
            to_json!(backend.resolve(&repo, &rev))
        }
        OperationKind::PredictConflicts => {
            to_json!(crate::conflict_prevention::global().predict_now(backend, &repo))
        }
        OperationKind::MergeBase => {
            let a: Oid = arg_or_err!(args, "a", request.id);
            let b: Oid = arg_or_err!(args, "b", request.id);
            to_json!(backend.merge_base(&repo, a, b))
        }
        OperationKind::ReadBlob => {
            let path: PathBuf = arg_or_err!(args, "path", request.id);
            let source: BlobSource = arg_or_err!(args, "source", request.id);
            to_json!(backend.read_blob(&repo, &path, &source))
        }
        OperationKind::ReadBlobBytes => {
            let path: PathBuf = arg_or_err!(args, "path", request.id);
            let source: BlobSource = arg_or_err!(args, "source", request.id);
            let max: u64 = arg_or_err!(args, "max", request.id);
            to_json!(backend.read_blob_bytes(&repo, &path, &source, max))
        }
        OperationKind::Blame => {
            let path: PathBuf = arg_or_err!(args, "path", request.id);
            let at: Oid = arg_or_err!(args, "at", request.id);
            to_json!(backend
                .blame(&repo, &path, at)
                .map(|b| crate::mailmap::map_blame(backend, &repo, b)))
        }
        OperationKind::FileHistory => {
            let path: PathBuf = arg_or_err!(args, "path", request.id);
            let start: Oid = arg_or_err!(args, "start", request.id);
            let skip: usize = args
                .get("skip")
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or(0);
            let limit: usize = arg_or_err!(args, "limit", request.id);
            to_json!(backend
                .file_history(&repo, &path, start, skip, limit)
                .map(|p| crate::mailmap::map_file_history(backend, &repo, p)))
        }
        OperationKind::ListTree => {
            let at: Oid = arg_or_err!(args, "at", request.id);
            to_json!(backend.list_tree(&repo, at))
        }
        OperationKind::Conflicts => to_json!(backend.conflicts(&repo)),
        OperationKind::LfsStatus => to_json!(backend.lfs_status(&repo)),
        OperationKind::LfsInstall => {
            let scope: LfsInstallScope = args
                .get("scope")
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_default();
            to_json!(backend.lfs_install(&repo, scope))
        }
        OperationKind::LfsTrack => {
            let patterns: Vec<String> = arg_or_err!(args, "patterns", request.id);
            let opts: LfsTrackOptions = args
                .get("opts")
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_default();
            to_json!(backend.lfs_track(&repo, &patterns, &opts))
        }
        OperationKind::LfsUntrack => {
            let patterns: Vec<String> = arg_or_err!(args, "patterns", request.id);
            to_json!(backend.lfs_untrack(&repo, &patterns))
        }
        OperationKind::LfsPrune => to_json!(backend.lfs_prune(&repo)),
        OperationKind::LfsFiles => {
            let at: Option<Oid> = args
                .get("at")
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .flatten();
            to_json!(backend.lfs_files(&repo, at))
        }
        OperationKind::StashPush => {
            let message: Option<String> = args
                .get("message")
                .and_then(|v| serde_json::from_value(v.clone()).ok());
            let include_untracked: bool = args
                .get("include_untracked")
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or(false);
            let paths: Vec<String> = args
                .get("paths")
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_default();
            if paths.is_empty() {
                to_json!(backend.stash_push(&repo, message.as_deref(), include_untracked))
            } else {
                to_json!(backend.stash_push_paths(&repo, message.as_deref(), &paths))
            }
        }
        OperationKind::StashPop => {
            let id: StashId = arg_or_err!(args, "id", request.id);
            to_json!(backend.stash_pop(&repo, id))
        }
        OperationKind::StashApply => {
            let id: StashId = arg_or_err!(args, "id", request.id);
            to_json!(backend.stash_apply(&repo, id))
        }
        OperationKind::StashDrop => {
            let id: StashId = arg_or_err!(args, "id", request.id);
            to_json!(backend.stash_drop(&repo, id))
        }
        OperationKind::Merge => {
            let from: gitpoulpi_core::domain::Ref = arg_or_err!(args, "from", request.id);
            let opts: MergeOptions = args
                .get("opts")
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_default();
            let opts = crate::merge_policy::apply(&repo, opts);
            to_json!(backend.merge(&repo, &from, &opts))
        }
        OperationKind::StageFile
        | OperationKind::StageFolder
        | OperationKind::StageAll
        | OperationKind::StageHunk
        | OperationKind::StageLine => {
            let target: StageTarget = arg_or_err!(args, "target", request.id);
            match crate::stage_guard::refusal(backend, &repo, request.command, &target, args) {
                Some(refused) => Err(refused),
                None => to_json!(backend.stage(&repo, &target)),
            }
        }
        OperationKind::UnstageFile
        | OperationKind::UnstageFolder
        | OperationKind::UnstageAll
        | OperationKind::UnstageHunk
        | OperationKind::UnstageLine => {
            let target: StageTarget = arg_or_err!(args, "target", request.id);
            to_json!(backend.unstage(&repo, &target))
        }
        OperationKind::Commit => {
            let message: String = arg_or_err!(args, "message", request.id);
            let opts: CommitOptions = match args.get("opts") {
                Some(v) => match serde_json::from_value(v.clone()) {
                    Ok(o) => o,
                    Err(e) => return err(request.id, format!("invalid `opts`: {e}"), false),
                },
                None => CommitOptions::default(),
            };
            let committed = backend.commit(&repo, &message, &opts);
            // Git's own refusal ("Committing is not possible because you have
            // unmerged files") says nothing about what to do: name the files.
            let committed = committed.map_err(|e| match backend.conflict_entries(&repo) {
                Ok(entries) if !entries.is_empty() => {
                    gitpoulpi_core::Error::Backend(unresolved_conflicts_message(&entries))
                }
                _ => e,
            });
            to_json!(committed)
        }
        OperationKind::Checkout => {
            let target: CheckoutTarget = arg_or_err!(args, "target", request.id);
            to_json!(backend.checkout(&repo, &target))
        }
        OperationKind::Discard => {
            let paths: Vec<String> = arg_or_err!(args, "paths", request.id);
            to_json!(backend.discard(&repo, &paths))
        }
        OperationKind::DiscardHunk | OperationKind::DiscardLine => {
            let target: DiscardTarget = arg_or_err!(args, "target", request.id);
            to_json!(backend.discard_part(&repo, &target))
        }
        OperationKind::BranchCreate => {
            let name: String = arg_or_err!(args, "name", request.id);
            let at: Oid = arg_or_err!(args, "at", request.id);
            to_json!(backend.branch_create(&repo, &name, at))
        }
        OperationKind::BranchCreateTracking => {
            let local_name: String = arg_or_err!(args, "local_name", request.id);
            let upstream: String = arg_or_err!(args, "upstream", request.id);
            let created = backend.branch_create_tracking(&repo, &local_name, &upstream);
            if created.is_ok() {
                if let Ok(known) = crate::remote_keys::remote_names(backend, &repo) {
                    if let Some(primary) = crate::remote_keys::remote_of_upstream(&upstream, &known)
                    {
                        crate::remote_keys::apply_default_targets(
                            backend,
                            &repo,
                            &local_name,
                            &primary,
                        );
                    }
                }
            }
            to_json!(created)
        }
        OperationKind::BranchDelete => {
            let name: String = arg_or_err!(args, "name", request.id);
            to_json!(backend.branch_delete(&repo, &name))
        }
        OperationKind::BranchRename => {
            let old_name: String = arg_or_err!(args, "old_name", request.id);
            let new_name: String = arg_or_err!(args, "new_name", request.id);
            to_json!(backend.branch_rename(&repo, &old_name, &new_name))
        }
        // `upstream: null` (or absent) removes the upstream.
        OperationKind::BranchSetUpstream => {
            let branch: String = arg_or_err!(args, "branch", request.id);
            let upstream: Option<String> = args
                .get("upstream")
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .flatten();
            to_json!(backend.branch_set_upstream(&repo, &branch, upstream.as_deref()))
        }
        // `push_to` / `pull_from`: config values, `<remote>` or
        // `<remote>:<branch>`; absent means none.
        OperationKind::BranchSetRemotes => {
            let branch: String = arg_or_err!(args, "branch", request.id);
            let list = |key: &str| -> Vec<String> {
                args.get(key)
                    .and_then(|v| serde_json::from_value(v.clone()).ok())
                    .unwrap_or_default()
            };
            // `upstream` (optional, `remote/branch`) also moves git's own
            // upstream, inside the same journal entry.
            let upstream: Option<String> = args
                .get("upstream")
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .flatten();
            to_json!(crate::remote_keys::set_remotes(
                backend,
                &repo,
                &branch,
                upstream.as_deref(),
                (&list("push_to"), &list("pull_from"))
            ))
        }
        OperationKind::FastForward => {
            let branch: String = arg_or_err!(args, "branch", request.id);
            let target: Oid = arg_or_err!(args, "target", request.id);
            to_json!(backend.fast_forward(&repo, &branch, target))
        }
        OperationKind::WorktreeAdd => {
            let path: PathBuf = arg_or_err!(args, "path", request.id);
            let opts: WorktreeAddOptions = arg_or_err!(args, "opts", request.id);
            to_json!(backend.worktree_add(&repo, &path, &opts))
        }
        OperationKind::PatchCreate => {
            let source: PatchSource = arg_or_err!(args, "source", request.id);
            let dest: PathBuf = arg_or_err!(args, "dest", request.id);
            to_json!(backend.patch_create(&repo, &source, &dest))
        }
        OperationKind::DeleteFile => {
            let paths: Vec<String> = arg_or_err!(args, "paths", request.id);
            to_json!(backend.delete_files(&repo, &paths))
        }
        OperationKind::ConvertFiles => {
            let paths: Vec<String> = arg_or_err!(args, "paths", request.id);
            let op: gitpoulpi_core::textconv::ConvertOp = arg_or_err!(args, "op", request.id);
            to_json!(backend.convert_files(&repo, &paths, op))
        }
        OperationKind::AddToGitignore => {
            let pattern: String = arg_or_err!(args, "pattern", request.id);
            let file: Option<String> = args
                .get("file")
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .flatten();
            to_json!(backend.gitignore_add(&repo, &pattern, file.as_deref()))
        }
        OperationKind::MailmapIdentities => {
            let a: gitpoulpi_ipc::mailmap::IdentitiesArgs =
                serde_json::from_value(args.clone()).unwrap_or_default();
            to_json!(backend.mailmap_identities(&repo, &a.scope, &a.options))
        }
        OperationKind::MailmapResolve => {
            let a: gitpoulpi_ipc::mailmap::ResolveArgs = match serde_json::from_value(args.clone())
            {
                Ok(a) => a,
                Err(e) => return err(request.id, format!("invalid arguments: {e}"), false),
            };
            to_json!(backend.mailmap_resolve(&repo, &a.idents))
        }
        OperationKind::MailmapAdd => {
            let a: gitpoulpi_ipc::mailmap::AddArgs = match serde_json::from_value(args.clone()) {
                Ok(a) => a,
                Err(e) => return err(request.id, format!("invalid arguments: {e}"), false),
            };
            to_json!(backend.mailmap_add(&repo, &a.canonical, &a.from, a.file.as_deref()))
        }
        OperationKind::GitattributesTreat => {
            let paths: Vec<String> = arg_or_err!(args, "paths", request.id);
            let scope: gitpoulpi_core::domain::RuleScope = arg_or_err!(args, "scope", request.id);
            let treat: gitpoulpi_core::domain::Treat = arg_or_err!(args, "treat", request.id);
            let file: Option<String> = args
                .get("file")
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .flatten();
            to_json!(backend.gitattributes_treat(&repo, &paths, scope, file.as_deref(), treat))
        }
        OperationKind::BinaryPaths => {
            let paths: Vec<String> = arg_or_err!(args, "paths", request.id);
            to_json!(backend.binary_paths(&repo, &paths))
        }
        OperationKind::PatchApply => {
            let patch: PathBuf = arg_or_err!(args, "patch", request.id);
            to_json!(backend.patch_apply(&repo, &patch))
        }
        OperationKind::ApplyCommit => {
            let commit: Oid = arg_or_err!(args, "commit", request.id);
            to_json!(backend.apply_commit_to_worktree(&repo, commit))
        }
        OperationKind::StashFromCommit => {
            let commit: Oid = arg_or_err!(args, "commit", request.id);
            let message: Option<String> = args
                .get("message")
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .flatten();
            to_json!(backend.stash_from_commit(&repo, commit, message.as_deref()))
        }
        OperationKind::TagCreate => {
            let name: String = arg_or_err!(args, "name", request.id);
            let at: Oid = arg_or_err!(args, "at", request.id);
            let annotated_message: Option<String> = args
                .get("annotated_message")
                .and_then(|v| serde_json::from_value(v.clone()).ok());
            to_json!(backend.tag_create(&repo, &name, at, annotated_message.as_deref()))
        }
        OperationKind::TagDelete => {
            let name: String = arg_or_err!(args, "name", request.id);
            to_json!(backend.tag_delete(&repo, &name))
        }
        OperationKind::TagAnnotate => {
            let name: String = arg_or_err!(args, "name", request.id);
            let message: String = arg_or_err!(args, "message", request.id);
            to_json!(backend.tag_annotate(&repo, &name, &message))
        }
        // TagPush / TagDeleteRemote are network operations and live in
        // `dispatch_network`, not here.
        OperationKind::Reset => {
            let target: Oid = arg_or_err!(args, "target", request.id);
            let mode: ResetMode = arg_or_err!(args, "mode", request.id);
            to_json!(backend.reset(&repo, target, mode))
        }
        OperationKind::Reword => {
            let commit: Oid = arg_or_err!(args, "commit", request.id);
            let message: String = arg_or_err!(args, "message", request.id);
            to_json!(backend.reword(&repo, commit, &message))
        }
        OperationKind::AmendCommit => {
            let commit: Oid = arg_or_err!(args, "commit", request.id);
            let amend: gitpoulpi_core::domain::CommitAmend = arg_or_err!(args, "amend", request.id);
            to_json!(backend.amend_commit(&repo, commit, &amend))
        }
        OperationKind::SplitSource => {
            let commit: Oid = arg_or_err!(args, "commit", request.id);
            to_json!(backend.split_source(&repo, commit))
        }
        OperationKind::SplitCommit => {
            let req: gitpoulpi_core::domain::SplitCommit = arg_or_err!(args, "req", request.id);
            to_json!(backend.split_commit(&repo, &req))
        }
        OperationKind::Rebase => {
            let onto: gitpoulpi_core::domain::Ref = arg_or_err!(args, "onto", request.id);
            let opts: RebaseOptions = args
                .get("opts")
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_default();
            to_json!(backend.rebase(&repo, &onto, &opts))
        }
        OperationKind::ComposeCommits => {
            #[derive(serde::Deserialize)]
            struct Composed {
                message: String,
                paths: Vec<PathBuf>,
            }
            let commits: Vec<Composed> = arg_or_err!(args, "commits", request.id);
            let opts: CommitOptions = args
                .get("opts")
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_default();
            let total = commits.len();
            let mut made: Vec<Oid> = Vec::new();
            let run = |made: &mut Vec<Oid>| -> gitpoulpi_core::Result<()> {
                backend.unstage(&repo, &StageTarget::All)?;
                for commit in &commits {
                    for path in &commit.paths {
                        backend.stage(&repo, &StageTarget::File(path.clone()))?;
                    }
                    made.push(backend.commit(&repo, &commit.message, &opts)?);
                }
                Ok(())
            };
            match run(&mut made) {
                Ok(()) => to_json!(Ok::<_, gitpoulpi_core::Error>(made.clone())),
                Err(e) => Err((
                    format!("stopped after {} of {total} commits: {e}", made.len()),
                    e.typed(),
                    e.detail().cloned(),
                )),
            }
        }
        OperationKind::RebasePlan => {
            let upstream: Option<Oid> = args
                .get("upstream")
                .and_then(|v| serde_json::from_value(v.clone()).ok());
            to_json!(backend.rebase_plan(&repo, upstream))
        }
        OperationKind::RebaseInteractive => {
            let req: InteractiveRebase = arg_or_err!(args, "request", request.id);
            to_json!(backend.rebase_interactive(&repo, &req))
        }
        OperationKind::RebaseState => {
            to_json!(backend.rebase_state(&repo))
        }
        OperationKind::RebaseContinue => {
            to_json!(backend.rebase_continue(&repo))
        }
        OperationKind::RebaseSkip => {
            to_json!(backend.rebase_skip(&repo))
        }
        OperationKind::RebaseAbort => {
            to_json!(backend.rebase_abort(&repo))
        }
        OperationKind::CherryPick => {
            let commits: Vec<Oid> = arg_or_err!(args, "commits", request.id);
            let opts: CherryPickOptions = args
                .get("opts")
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_default();
            to_json!(backend.cherry_pick(&repo, &commits, &opts))
        }
        OperationKind::Revert => {
            let commit: Oid = arg_or_err!(args, "commit", request.id);
            let mainline: Option<u32> = args
                .get("mainline")
                .and_then(|v| serde_json::from_value(v.clone()).ok());
            to_json!(backend.revert(&repo, commit, mainline))
        }
        OperationKind::RemoteAdd => {
            let name: String = arg_or_err!(args, "name", request.id);
            let url: String = arg_or_err!(args, "url", request.id);
            to_json!(backend.remote_add(&repo, &name, &url))
        }
        OperationKind::RemoteRemove => {
            let name: String = arg_or_err!(args, "name", request.id);
            to_json!(backend.remote_remove(&repo, &name))
        }
        OperationKind::RemoteRename => {
            let old_name: String = arg_or_err!(args, "old_name", request.id);
            let new_name: String = arg_or_err!(args, "new_name", request.id);
            to_json!(backend.remote_rename(&repo, &old_name, &new_name))
        }
        OperationKind::RemoteSetUrl => {
            let name: String = arg_or_err!(args, "name", request.id);
            let url: String = arg_or_err!(args, "url", request.id);
            to_json!(backend.remote_set_url(&repo, &name, &url))
        }
        // `url: null` (or absent) clears the push URL.
        OperationKind::RemoteSetPushUrl => {
            let name: String = arg_or_err!(args, "name", request.id);
            let url: Option<String> = args
                .get("url")
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .flatten();
            to_json!(backend.remote_set_push_url(&repo, &name, url.as_deref()))
        }
        // `value: null` (or absent) clears the key.
        OperationKind::SetRepoConfig => {
            let key: String = arg_or_err!(args, "key", request.id);
            let value: Option<String> = args
                .get("value")
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .flatten();
            to_json!(backend.set_repo_config(&repo, &key, value.as_deref()))
        }
        OperationKind::SparseSet => {
            let dirs: Vec<String> = arg_or_err!(args, "dirs", request.id);
            to_json!(backend.sparse_checkout_set(&repo, &dirs))
        }
        OperationKind::SparseDisable => to_json!(backend.sparse_checkout_disable(&repo)),
        // `command: null` (or absent) clears the pin.
        OperationKind::SetSshCommand => {
            let command: Option<String> = args
                .get("command")
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .flatten();
            to_json!(backend.set_ssh_command(&repo, command.as_deref()))
        }
        OperationKind::ResolveConflict => {
            let path: PathBuf = arg_or_err!(args, "path", request.id);
            let resolution: ConflictResolution = arg_or_err!(args, "resolution", request.id);
            let opts: gitpoulpi_core::domain::ResolveOptions = request
                .args
                .get("opts")
                .and_then(|o| serde_json::from_value(o.clone()).ok())
                .unwrap_or_default();
            // What the path was, for "unresolve" later.
            let before = opts
                .stage
                .then(|| backend.conflict_entries(&repo).ok())
                .flatten()
                .and_then(|all| all.into_iter().find(|e| e.path == path));
            let result = backend.resolve_conflict(&repo, &path, resolution, &opts);
            if let (Ok(()), Some(entry)) = (&result, before) {
                let auto = args
                    .get("auto")
                    .and_then(|a| a.as_str())
                    .map(str::to_string);
                crate::resolved_record::note(&repo, entry, auto);
            }
            to_json!(result)
        }
        OperationKind::Reflog => {
            let name: String = args
                .get("ref")
                .and_then(|v| v.as_str())
                .unwrap_or("HEAD")
                .to_string();
            let skip: usize = args
                .get("skip")
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or(0);
            let limit: usize = args
                .get("limit")
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or(100);
            to_json!(backend.reflog(&repo, &name, skip, limit))
        }
        OperationKind::LostCommits => {
            let opts: gitpoulpi_core::domain::LostCommitOptions =
                serde_json::from_value(args.clone()).unwrap_or_default();
            to_json!(backend.lost_commits(&repo, &opts))
        }
        OperationKind::RewritePreview => {
            let recipe: gitpoulpi_core::domain::rewrite::Recipe =
                arg_or_err!(args, "recipe", request.id);
            let vouched: Vec<String> = args
                .get("vouched")
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_default();
            match crate::rewrite_runs::preview(backend, &repo, recipe, &vouched) {
                Ok(plan) => to_json!(Ok::<_, gitpoulpi_core::Error>(plan)),
                // A cancel is the user's own: no warning in the log.
                Err(e @ gitpoulpi_core::Error::Canceled { .. }) => {
                    return err_kind(request.id, e.to_string(), false, e.typed());
                }
                Err(e) => to_json!(Err::<(), _>(e)),
            }
        }
        OperationKind::RewriteRows => {
            let rows: gitpoulpi_ipc::rewrite::RowsArgs = match serde_json::from_value(args.clone())
            {
                Ok(parsed) => parsed,
                Err(e) => return err(request.id, format!("invalid arguments: {e}"), false),
            };
            match crate::rewrite_runs::live(&repo, &rows.run) {
                Some(plan) => to_json!(Ok::<_, gitpoulpi_core::Error>(crate::rewrite_runs::page(
                    &plan, &rows
                ))),
                None => to_json!(Err::<(), _>(crate::rewrite_runs::stale(
                    "no such preview: run it again"
                ))),
            }
        }
        OperationKind::RewriteBlobDiff => {
            let diff: gitpoulpi_ipc::rewrite::BlobDiffArgs =
                match serde_json::from_value(args.clone()) {
                    Ok(parsed) => parsed,
                    Err(e) => return err(request.id, format!("invalid arguments: {e}"), false),
                };
            to_json!(backend.rewrite_blob_diff(&repo, &diff.run, diff.commit, &diff.path))
        }
        OperationKind::RewriteAnalyze => {
            let args: gitpoulpi_ipc::rewrite::AnalyzeArgs =
                match serde_json::from_value(args.clone()) {
                    Ok(parsed) => parsed,
                    Err(e) => return err(request.id, format!("invalid arguments: {e}"), false),
                };
            match crate::rewrite_runs::analyze(backend, &repo, &args.scope, args.top) {
                Ok(found) => to_json!(Ok::<_, gitpoulpi_core::Error>(found)),
                Err(e @ gitpoulpi_core::Error::Canceled { .. }) => {
                    return err_kind(request.id, e.to_string(), false, e.typed());
                }
                Err(e) => to_json!(Err::<(), _>(e)),
            }
        }
        OperationKind::RewriteScanSecrets => {
            let scan: gitpoulpi_ipc::rewrite::ScanSecretsArgs =
                match serde_json::from_value(args.clone()) {
                    Ok(parsed) => parsed,
                    Err(e) => return err(request.id, format!("invalid arguments: {e}"), false),
                };
            match crate::rewrite_runs::scan_secrets(backend, &repo, &scan.scope) {
                Ok(found) => to_json!(Ok::<_, gitpoulpi_core::Error>(found)),
                // A cancel is the user's own: no warning in the log.
                Err(e @ gitpoulpi_core::Error::Canceled { .. }) => {
                    return err_kind(request.id, e.to_string(), false, e.typed());
                }
                Err(e) => to_json!(Err::<(), _>(e)),
            }
        }
        OperationKind::RewriteScriptCheck => {
            let check: gitpoulpi_ipc::rewrite::ScriptCheckArgs =
                match serde_json::from_value(args.clone()) {
                    Ok(parsed) => parsed,
                    Err(e) => return err(request.id, format!("invalid arguments: {e}"), false),
                };
            let policy = crate::rewrite_runs::script_policy(&repo, &check.vouched);
            to_json!(backend.rewrite_script_check(&repo, &check.script, &policy))
        }
        OperationKind::RewriteScriptTest => {
            let test: gitpoulpi_ipc::rewrite::ScriptTestArgs =
                match serde_json::from_value(args.clone()) {
                    Ok(parsed) => parsed,
                    Err(e) => return err(request.id, format!("invalid arguments: {e}"), false),
                };
            let policy = crate::rewrite_runs::script_policy(&repo, &test.vouched);
            to_json!(backend.rewrite_script_test(&repo, &test.script, &policy, &test.commits))
        }
        OperationKind::FlowTrustHooks => to_json!(crate::flow::trust_hooks(&repo, args)),
        OperationKind::RepoAuthGet => return crate::repo_auth::get(backend, &repo, request),
        OperationKind::RepoAuthSet => return crate::repo_auth::set(&repo, request),
        OperationKind::RepoAuthClear => return crate::repo_auth::clear(&repo, request),
        OperationKind::RewriteTrustScript => {
            let trust: gitpoulpi_ipc::rewrite::TrustArgs =
                match serde_json::from_value(args.clone()) {
                    Ok(parsed) => parsed,
                    Err(e) => return err(request.id, format!("invalid arguments: {e}"), false),
                };
            to_json!(crate::rewrite_runs::trust_script(
                &repo,
                &trust.hash,
                &trust.label
            ))
        }
        OperationKind::RewriteApply => {
            let apply: gitpoulpi_ipc::rewrite::ApplyArgs =
                match serde_json::from_value(args.clone()) {
                    Ok(parsed) => parsed,
                    Err(e) => return err(request.id, format!("invalid arguments: {e}"), false),
                };
            let Some(plan) = crate::rewrite_runs::live_or_load(backend, &repo, &apply.run) else {
                return failed_typed(
                    request,
                    crate::rewrite_runs::stale("no such preview: run it again"),
                );
            };
            if plan.digest != apply.digest {
                return failed_typed(
                    request,
                    crate::rewrite_runs::stale("the plan is not the one that was previewed"),
                );
            }
            if let (Some(count), false, None) = (
                crate::rewrite_runs::touches_pushed(&plan),
                apply.confirmed,
                &apply.target,
            ) {
                let prompt = format!(
                    "{count} of the refs this rewrite moves are already on a remote. \
                     Rewriting them means a force push afterwards, and everyone who \
                     pulled them has to recover."
                );
                let error = gitpoulpi_core::Error::NeedsConfirmation(prompt);
                return err_kind(request.id, error.to_string(), false, error.typed());
            }
            let outcome = match &apply.target {
                Some(target) => backend.rewrite_extract(&repo, &plan, target),
                None => backend.rewrite_apply(&repo, &plan),
            };
            if outcome.is_ok() {
                crate::rewrite_runs::forget(&repo, &apply.run);
            }
            to_json!(outcome)
        }
        OperationKind::RewriteDiscard => {
            let discard: gitpoulpi_ipc::rewrite::DiscardArgs =
                match serde_json::from_value(args.clone()) {
                    Ok(parsed) => parsed,
                    Err(e) => return err(request.id, format!("invalid arguments: {e}"), false),
                };
            crate::rewrite_runs::forget(&repo, &discard.run);
            to_json!(backend.rewrite_discard(&repo, &discard.run))
        }
        OperationKind::RewriteNote => {
            let note: gitpoulpi_ipc::rewrite::NoteArgs = match serde_json::from_value(args.clone())
            {
                Ok(parsed) => parsed,
                Err(e) => return err(request.id, format!("invalid arguments: {e}"), false),
            };
            to_json!(backend.rewrite_record(&repo, &note.run).map(|record| {
                let refs = gitpoulpi_core::domain::rewrite::note_refs(&record);
                gitpoulpi_core::domain::rewrite::RewriteNote {
                    run: record.run,
                    text: gitpoulpi_core::domain::rewrite::note_text(&refs),
                    refs,
                }
            }))
        }
        OperationKind::RewriteMaps => {
            let maps: gitpoulpi_ipc::rewrite::MapsArgs = match serde_json::from_value(args.clone())
            {
                Ok(parsed) => parsed,
                Err(e) => return err(request.id, format!("invalid arguments: {e}"), false),
            };
            to_json!(backend.rewrite_maps(&repo, &maps.run, maps.out.as_deref()))
        }
        OperationKind::RewriteCheckSigning => to_json!(backend.rewrite_check_signing(&repo)),
        OperationKind::RewriteTools => {
            let tools: gitpoulpi_ipc::rewrite::ToolsArgs =
                serde_json::from_value(args.clone()).unwrap_or_default();
            to_json!(backend.rewrite_tools(&repo, tools.fresh).map(|mut t| {
                t.external_scripts = Some(crate::rewrite_runs::script_switches());
                t
            }))
        }
        other => {
            return err(
                request.id,
                format!("{other:?} is not yet wired up to the daemon"),
                true,
            )
        }
    };

    // Reads must not broadcast `RepoChanged`: a client that refreshes on
    // that event would answer its own refresh with another refresh, and
    // every connected TUI would spin.
    let mutated = !request.command.traits().read;

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

/// A typed backend error as a failed `Dispatched` (logged like any other).
fn failed_typed(request: &Request, error: gitpoulpi_core::Error) -> Dispatched {
    failed(
        request,
        error.to_string(),
        error.typed(),
        error.detail().cloned(),
    )
}

/// "cannot commit: 2 files still have conflicts (a.rs, b.rs): resolve them and
/// stage the results first" - the sentence a commit refused over unmerged
/// files carries for the CLI and MCP (the TUI asks before it gets here).
fn unresolved_conflicts_message(entries: &[gitpoulpi_core::domain::ConflictEntry]) -> String {
    const SHOWN: usize = 5;
    let mut names: Vec<String> = entries
        .iter()
        .take(SHOWN)
        .map(|e| e.path.to_string_lossy().replace('\\', "/"))
        .collect();
    if entries.len() > SHOWN {
        names.push(format!("and {} more", entries.len() - SHOWN));
    }
    let files = if entries.len() == 1 {
        "1 file still has conflicts".to_string()
    } else {
        format!("{} files still have conflicts", entries.len())
    };
    format!(
        "cannot commit: {files} ({}): resolve them and stage the results first",
        names.join(", ")
    )
}
