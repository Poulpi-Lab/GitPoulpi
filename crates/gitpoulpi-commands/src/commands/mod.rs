//! The CLI half of the backend<->frontend command mapping: one arm per
//! `gitpoulpi` subcommand, each turning the strings a human or an agent
//! typed into exactly one `GitBackend`-mirroring IPC request.
//!
//! **Revisions are resolved by the backend, never by the CLI.** Mutating
//! operations take an `Oid`, while the command line carries `main`,
//! `HEAD~2` or an abbreviated sha. That goes through
//! `OperationKind::Resolve` rather than a local `git rev-parse`, so the
//! CLI stays a pure IPC client and both backends agree on what a revision
//! means.
//!
//! **`--json` output is the contract.** The MCP server and scripts read
//! these payloads, so each arm emits the backend's own serialized domain
//! type wherever one exists, and tests assert on the deserialized value.

use gitpoulpi_core::domain::split_message;
use gitpoulpi_core::domain::{
    short, Blame, BlameContent, Blob, BlobContent, BlobSource, CheckoutTarget, CherryPickOptions,
    CherryPickOutcome, CloneOptions, CommitAmend, CommitGraph, CommitOptions, DiffFileList,
    DiffTarget, DiscardTarget, FetchOptions, FetchOutcome, FileChangeKind, FileDiff,
    FileHistoryPage, HunkId, LineSelector, LogOptions, MergeFastForward, MergeOptions,
    MergeOutcome, Oid, OperationKind, PullMode, PullOptions, PullOutcome, PushOptions, PushOutcome,
    RebaseAction, RefKind, RepoRef, ResetMode, RevertOutcome, RewordOutcome, StageTarget,
    WorkingTree,
};
use serde_json::{json, Value};

use crate::cli::{Cli, Commands, PullModeArg, ResetModeArg};
use crate::client::{discover_repo, Session};

mod blob;
mod branch;
pub(crate) mod conflicts;
mod flow;
pub(crate) mod flow_run;
mod group_run;
#[cfg(feature = "lfs")]
mod lfs;
#[cfg(feature = "mermaid")]
mod log_mermaid;
mod mailmap;
mod rebase;
mod refs;
mod remote;
mod repo_auth;
#[cfg(feature = "history-rewrite")]
pub(crate) mod rewrite;
#[cfg(feature = "history-rewrite")]
mod rewrite_host;
pub(crate) mod split;
mod stash;
#[cfg(feature = "submodule")]
mod submodule;
mod tag;
mod undo;
mod upstream;
#[cfg(feature = "worktree")]
mod worktree;

use self::branch::{branch, patch};
use self::conflicts::conflicts;
use self::flow::flow;
#[cfg(feature = "lfs")]
use self::lfs::lfs;
use self::rebase::{read_plan_steps, rebase_command};
use self::refs::{all_refs, find_ref, ref_name, refs};
use self::remote::{default_remote, remote, repo_config, sparse_checkout, ssh_command};
use self::stash::stash;
use self::tag::tag;
use self::undo::{journal_listing, undo_or_redo};
#[cfg(feature = "worktree")]
use self::worktree::worktree;

/// What a subcommand produced: the machine-readable payload and the line
/// a human sees. Every arm returns both so `--json` is never a second code
/// path, and both the CLI and the MCP server build a response from it.
pub use crate::repo_cmd::Emitted;

pub(crate) fn emitted(json: Value, human: impl Into<String>) -> Emitted {
    Emitted {
        json,
        human: human.into(),
        exit_code: 0,
    }
}

/// Returns the process exit code: 0, or 2 for a refused undo/redo.
///
/// `session` and `backend` are resolved by `main` (the session from `-s` or
/// the repo root; the backend through the precedence ladder in
/// `gitpoulpi_config::Settings`) so the CLI and the TUI cannot apply
/// different rules to the same flag. `backend` only matters when this
/// process has to start the session's daemon itself.
///
/// Connects a fresh (unpublished - see `Session::connect`) session and
/// runs one command through `execute`; the MCP server instead keeps one
/// published `Session` alive and calls `execute` directly.
pub async fn run(
    cli: Cli,
    session: String,
    backend: gitpoulpi_config::DefaultBackend,
) -> Result<i32, String> {
    let json = cli.json;
    // Likewise `completions`: a script on stdout, no repository or daemon.
    if let Some(Commands::Completions { shell }) = &cli.command {
        use clap::CommandFactory;
        let mut command = Cli::command();
        clap_complete::generate(*shell, &mut command, "gitpoulpi", &mut std::io::stdout());
        return Ok(0);
    }
    // `shell-init` likewise prints a script and never connects.
    if let Some(Commands::ShellInit { shell, name }) = &cli.command {
        if !crate::shell_init::valid_name(name) {
            return Err(format!(
                "--name {name:?} is not a usable function name (letters, digits, _ and - only)"
            ));
        }
        print!("{}", crate::shell_init::script(*shell, name));
        return Ok(0);
    }
    // Commands that answer from local files never connect to (and so never
    // start) a daemon.
    let emitted = if cli.command.as_ref().is_some_and(is_local) {
        let command = cli.command.expect("checked above");
        run_local(command, &session, cli.repo.as_deref()).await?
    } else {
        let proposing = matches!(
            cli.command,
            Some(Commands::Ui {
                action: crate::cli::UiAction::Propose { .. },
                ..
            })
        );
        if proposing
            && !gitpoulpi_config::Settings::resolve_ambient()
                .map(|s| s.config().mcp.allow_ui_propose)
                .unwrap_or(false)
        {
            let emitted = crate::ui_cmd::propose_refused();
            if !json {
                eprintln!("gitpoulpi: {}", emitted.human);
                return Ok(emitted.exit_code);
            }
            emit(json, emitted.json, &emitted.human)?;
            return Ok(emitted.exit_code);
        }
        let mut ipc = Session::connect(&session, None, backend, false).await?;
        // Ctrl+C on this command stops the network job it started.
        ipc.set_cancel_on_disconnect(true);
        // A person typed this command: they may trust a history rewrite script.
        ipc.set_may_trust(true);
        if matches!(cli.command, Some(Commands::Ui { .. })) {
            // A human or script typing `gitpoulpi ui ...`: the same
            // `[mcp] ui_control` mode an agent setup would use applies.
            let control = gitpoulpi_config::Settings::resolve_ambient()
                .map(|s| s.config().mcp.ui_control)
                .unwrap_or_default();
            ipc.set_ui_origin(gitpoulpi_ipc::ui::UiOrigin {
                control,
                ..gitpoulpi_ipc::ui::UiOrigin::cli()
            });
        }
        Box::pin(execute(&mut ipc, cli, &session)).await?
    };
    let code = emitted.exit_code;
    if code != 0 && !json {
        eprintln!("gitpoulpi: {}", emitted.human);
        return Ok(code);
    }
    emit(json, emitted.json, &emitted.human)?;
    Ok(code)
}

/// Routes one parsed `Cli` to its `GitBackend` operation(s) over `ipc`,
/// local repo/hosting state, or both, and returns the result with no
/// process-level side effect (`println!`, `exit`), so the CLI's `run` and
/// the MCP server's `call_tool` share exactly this code path.
///
/// `ipc` is expected already connected to `session`'s daemon; `execute`
/// only retargets it (`Session::set_repo`) at the repository `cli.repo`
/// names (or the one enclosing the current directory), and only for the
/// commands that need one (`repo recent/list/close` and most `hosting`
/// actions answer from local files).
pub async fn execute(ipc: &mut Session, cli: Cli, session: &str) -> Result<Emitted, String> {
    let command = cli.command.expect("callers handle the no-subcommand case");

    if is_local(&command) {
        return run_local(command, session, cli.repo.as_deref()).await;
    }

    // `clone`, `repo discover` and `repo open` legitimately run outside a
    // repository (the first produces one, the other two take a path), so a
    // failed discovery is only fatal for the rest; `Session::call` raises
    // it again for any command that turns out to need a repo.
    let repo = match (&command, discover_repo(cli.repo.as_deref())) {
        (_, Ok(repo)) => Some(repo),
        (
            Commands::Clone { .. }
            | Commands::Repo { .. }
            | Commands::Ui { .. }
            | Commands::Job { .. },
            Err(_),
        ) => None,
        #[cfg(feature = "workspaces")]
        (Commands::Workspace { .. }, Err(_)) => None,
        #[cfg(all(feature = "global-search", feature = "workspaces"))]
        (
            Commands::Search {
                workspace: Some(_), ..
            },
            Err(_),
        ) => None,
        // `GITPOULPI_WORKSPACE` stands in for the flag.
        #[cfg(all(feature = "global-search", feature = "workspaces"))]
        (Commands::Search { .. }, Err(_))
            if crate::workspace_cmd::selected_workspace(None).is_some() =>
        {
            None
        }
        #[cfg(feature = "hosting")]
        (Commands::Hosting { .. }, Err(_)) => None,
        (_, Err(e)) => return Err(e),
    };
    ipc.set_repo(repo.clone());

    match command {
        Commands::Repo { action } => crate::repo_cmd::run_via_daemon(session, ipc, action).await,
        #[cfg(feature = "workspaces")]
        Commands::Workspace { action } => crate::workspace_cmd::run_via_daemon(ipc, action).await,
        Commands::Job { action } => crate::job_cmd::run_via_daemon(ipc, action).await,
        #[cfg(feature = "global-search")]
        Commands::Search {
            query,
            #[cfg(feature = "workspaces")]
            workspace,
            kinds,
            limit,
        } => {
            let args = crate::search_cmd::SearchArgs {
                query,
                #[cfg(feature = "workspaces")]
                workspace,
                kinds,
                limit,
            };
            crate::search_cmd::run(ipc, repo, args).await
        }
        #[cfg(feature = "hosting")]
        Commands::Hosting { action } => {
            Box::pin(crate::hosting_cmd::run_via_daemon(ipc, action)).await
        }
        Commands::Ui {
            viewer,
            launch,
            action,
        } => crate::ui_cmd::run(ipc, viewer, launch, action).await,
        other => Box::pin(dispatch(ipc, other)).await,
    }
}

/// The commands that answer from files on this machine, with no repository
/// or daemon involved: `session ...`, `config ...`, `repo recent/list/close`, and every
/// `hosting` action but `clone`/`pr`. So `gitpoulpi repo recent` works from
/// a home directory (ADR-0006 puts that state beside the daemon, not in it).
pub(crate) fn is_local(command: &Commands) -> bool {
    match command {
        Commands::Repo { action } => !crate::repo_cmd::needs_daemon(action),
        #[cfg(feature = "hosting")]
        Commands::Hosting { action } => !crate::hosting_cmd::needs_daemon(action),
        Commands::Session { .. }
        | Commands::Config { .. }
        | Commands::Theme { .. }
        | Commands::Licenses
        | Commands::Alias { .. }
        | Commands::Installation { .. }
        | Commands::Path { .. } => true,
        Commands::Flow {
            action: crate::cli::FlowAction::Templates,
        } => true,
        #[cfg(feature = "workspaces")]
        Commands::Workspace { action } => !crate::workspace_cmd::needs_daemon(action),
        // The review-context defaults live in config files.
        #[cfg(feature = "ai")]
        Commands::Ai {
            action:
                crate::cli::AiCommand::ReviewContext { .. } | crate::cli::AiCommand::Trust { .. },
            ..
        } => true,
        _ => false,
    }
}

async fn run_local(
    command: Commands,
    session: &str,
    #[cfg_attr(not(feature = "ai"), allow(unused_variables))] repo: Option<&str>,
) -> Result<Emitted, String> {
    match command {
        Commands::Repo { action } => crate::repo_cmd::run_local(session, action),
        #[cfg(feature = "hosting")]
        Commands::Hosting { action } => Box::pin(crate::hosting_cmd::run_local(action)).await,
        Commands::Session { action } => crate::session_cmd::run(session, action),
        #[cfg(feature = "workspaces")]
        Commands::Workspace { action } => crate::workspace_cmd::run_local(action),
        Commands::Theme { action } => crate::theme_cmd::execute(action),
        Commands::Licenses => Ok(emitted(
            json!({ "licenses": gitpoulpi_core::licenses::THIRD_PARTY }),
            gitpoulpi_core::licenses::THIRD_PARTY.trim_end(),
        )),
        Commands::Alias { action } => crate::alias_cmd::alias(action),
        Commands::Installation { action } => crate::installation_cmd::execute(action),
        Commands::Path { action } => crate::alias_cmd::path(action),
        Commands::Flow { .. } => flow::templates(),
        Commands::Config { action } => {
            crate::config_cmd::execute(action, &gitpoulpi_config::Overrides::default())
        }
        #[cfg(feature = "ai")]
        Commands::Ai {
            action:
                crate::cli::AiCommand::ReviewContext {
                    project,
                    run_in_repo,
                    claude_md,
                    agents_md,
                    context_file,
                    no_context_file,
                    no_extra_file,
                    reset,
                },
            ..
        } => crate::ai_review_ctx::run(
            repo,
            project.as_deref(),
            &crate::cli::ReviewFlags {
                run_in_repo,
                claude_md,
                agents_md,
                context_file,
                no_context_file,
                no_extra_file,
            },
            reset,
        ),
        #[cfg(feature = "ai")]
        Commands::Ai {
            action:
                crate::cli::AiCommand::Trust {
                    profile,
                    accept_origin,
                    accept_certificate,
                    allow_insecure_http,
                },
            ..
        } => crate::ai_cmd::trust(
            &profile,
            accept_origin.as_deref(),
            accept_certificate.as_deref(),
            allow_insecure_http.map(|o| o == crate::cli::OnOff::On),
        ),
        _ => unreachable!("run_local is only called for `is_local` commands"),
    }
}

fn emit(json_mode: bool, json: Value, human: &str) -> Result<(), String> {
    // Nothing to say: `show --raw` already wrote the bytes to stdout.
    if json.is_null() && human.is_empty() {
        return Ok(());
    }
    if json_mode {
        println!(
            "{}",
            serde_json::to_string_pretty(&json)
                .map_err(|e| format!("failed to encode output: {e}"))?
        );
    } else if !human.is_empty() {
        println!("{human}");
    }
    Ok(())
}

async fn dispatch(session: &mut Session, command: Commands) -> Result<Emitted, String> {
    match command {
        Commands::Status => {
            let tree: WorkingTree = session.call(OperationKind::Status, json!({})).await?;
            let human = if tree.files.is_empty() {
                "clean".to_string()
            } else {
                tree.files
                    .iter()
                    .map(|f| {
                        format!(
                            "{:<10} {:<10} {}",
                            f.index
                                .map(|s| format!("{s:?}"))
                                .unwrap_or_else(|| "-".into()),
                            f.worktree
                                .map(|s| format!("{s:?}"))
                                .unwrap_or_else(|| "-".into()),
                            match &f.renamed_from {
                                Some(from) => format!("{} <- {}", f.path.display(), from.display()),
                                None => f.path.display().to_string(),
                            }
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            };
            // A pending auto-stash (changes GitPoulpi stashed for a
            // checkout/merge/rebase/pull and has not put back): the same
            // record the TUI banner reads. `null` when none.
            let pending = session.repo_root().and_then(|root| {
                let dir = gitpoulpi_config::Settings::resolve_ambient()
                    .map(|s| s.config().session.per_repo_state_dir.clone())
                    .unwrap_or_else(|_| gitpoulpi_config::DEFAULT_REPO_DATA_LOCATION.to_string());
                gitpoulpi_session::RepoState::load(&gitpoulpi_session::RepoState::path_for(
                    root, &dir,
                ))
                .ok()
                .and_then(|state| state.pending_auto_stash)
            });
            let mut json = to_value(&tree)?;
            if let Some(object) = json.as_object_mut() {
                object.insert("pending_auto_stash".into(), to_value(&pending)?);
            }
            let human = match &pending {
                Some(p) => format!(
                    "{human}\npending auto-stash: stash@{{{}}} ({:?}, {:?}) - `gitpoulpi stash pop {}` puts it back",
                    p.stash_id.0, p.reason, p.state, p.stash_id.0
                ),
                None => human,
            };
            Ok(emitted(json, human))
        }

        Commands::Log {
            graph,
            path,
            count: true,
            ..
        } => {
            let opts = LogOptions {
                all_refs: graph,
                paths: path.into_iter().map(Into::into).collect(),
                ..LogOptions::default()
            };
            let total: u64 = session
                .call(OperationKind::CountCommits, json!({ "opts": opts }))
                .await?;
            Ok(emitted(json!({ "total": total }), total.to_string()))
        }

        #[cfg(feature = "mermaid")]
        Commands::Log {
            graph,
            limit,
            skip,
            path,
            count: _,
            format: Some(_),
            detail,
            orientation,
            fence,
            range,
            subject_width,
        } => {
            log_mermaid::log_mermaid(
                session,
                log_mermaid::Args {
                    graph,
                    limit,
                    skip,
                    path,
                    detail: detail.map(Into::into),
                    orientation: orientation.map(Into::into),
                    fence,
                    range,
                    subject_width,
                },
            )
            .await
        }

        Commands::Log {
            graph,
            limit,
            skip,
            path,
            ..
        } => {
            let opts = LogOptions {
                limit,
                skip,
                start_ref: None,
                all_refs: graph,
                paths: path.into_iter().map(Into::into).collect(),
                use_mailmap: false,
                hidden: Default::default(),
            };
            let log: CommitGraph = session
                .call(OperationKind::Log, json!({ "opts": opts }))
                .await?;
            let human = log
                .nodes
                .iter()
                .map(|n| format!("{} {}", short(n.id), n.summary))
                .collect::<Vec<_>>()
                .join("\n");
            Ok(emitted(to_value(&log)?, human))
        }

        Commands::Diff {
            staged,
            path,
            from,
            to,
            name_only,
        } => {
            let target = match from {
                Some(from) => {
                    let from = resolve(session, &from).await?;
                    match to.as_deref() {
                        None | Some("worktree") => DiffTarget::CommitVsWorkingTree(from),
                        Some(to) => DiffTarget::CommitRange(from, resolve(session, to).await?),
                    }
                }
                None if staged => DiffTarget::IndexVsHead,
                None => DiffTarget::WorkingTreeVsIndex,
            };
            let mut diffs: Vec<FileDiff> = if name_only {
                session
                    .call::<DiffFileList>(OperationKind::DiffFiles, json!({ "target": target }))
                    .await?
                    .files
            } else {
                session
                    .call(OperationKind::Diff, json!({ "target": target }))
                    .await?
            };
            if let Some(path) = &path {
                diffs.retain(|d| d.path.to_string_lossy().starts_with(path.as_str()));
            }
            let human = diffs
                .iter()
                .map(|d| {
                    format!(
                        "{} ({:?}, {} hunk(s))",
                        d.path.display(),
                        d.status,
                        d.hunks.len()
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");
            Ok(emitted(to_value(&diffs)?, human))
        }

        Commands::Stage {
            file,
            folder,
            hunk,
            lines,
            paths,
            all,
            allow_sensitive,
        } => {
            let target = stage_target(file, folder, hunk, lines, paths, all)?;
            let kind = stage_kind(&target, true);
            let answer = session
                .call_typed::<()>(
                    kind,
                    json!({ "target": target, "allow_sensitive": allow_sensitive }),
                )
                .await?;
            match answer {
                Ok(()) => Ok(emitted(json!({ "staged": target }), "staged")),
                Err(e) => match e.kind {
                    Some(gitpoulpi_core::TypedError::SensitiveStage { files }) => {
                        Ok(sensitive_refusal(&files))
                    }
                    _ => Err(e.message),
                },
            }
        }

        Commands::Unstage {
            file,
            folder,
            hunk,
            lines,
            paths,
            all,
        } => {
            let target = stage_target(file, folder, hunk, lines, paths, all)?;
            let kind = stage_kind(&target, false);
            session
                .call::<()>(kind, json!({ "target": target }))
                .await?;
            Ok(emitted(json!({ "unstaged": target }), "unstaged"))
        }

        Commands::Discard { file, hunk, lines } => match discard_target(file.clone(), hunk, lines)?
        {
            Some(target) => {
                let kind = match target {
                    DiscardTarget::Hunk(..) => OperationKind::DiscardHunk,
                    DiscardTarget::Line(..) => OperationKind::DiscardLine,
                };
                session
                    .call::<()>(kind, json!({ "target": target }))
                    .await?;
                Ok(emitted(json!({ "discarded": target }), "discarded"))
            }
            None => {
                session
                    .call::<()>(OperationKind::Discard, json!({ "paths": [file.clone()] }))
                    .await?;
                Ok(emitted(
                    json!({ "discarded": { "paths": [file] } }),
                    "discarded",
                ))
            }
        },

        Commands::RmFile { paths } => {
            session
                .call::<()>(OperationKind::DeleteFile, json!({ "paths": paths }))
                .await?;
            Ok(emitted(json!({ "deleted": { "paths": paths } }), "deleted"))
        }

        Commands::Ignore { pattern, file } => {
            let added = session
                .call::<bool>(
                    OperationKind::AddToGitignore,
                    json!({ "pattern": pattern, "file": file }),
                )
                .await?;
            Ok(emitted(
                json!({ "ignored": { "pattern": pattern, "file": file, "added": added } }),
                "ignored",
            ))
        }

        Commands::Treat {
            kind,
            paths,
            extension,
            folder,
            file,
        } => {
            use gitpoulpi_core::domain::{AttributesWrite, RuleScope};
            let scope = if folder {
                RuleScope::Folder
            } else if extension {
                RuleScope::Extension
            } else {
                RuleScope::File
            };
            let treat = gitpoulpi_core::domain::Treat::from(kind);
            let done = session
                .call::<AttributesWrite>(
                    OperationKind::GitattributesTreat,
                    json!({ "paths": paths, "scope": scope, "treat": treat, "file": file }),
                )
                .await?;
            let human = format!("{}: {}", done.file, done.patterns.join(", "));
            Ok(emitted(json!({ "treated": done }), human))
        }

        Commands::Convert {
            paths,
            encoding,
            eol,
            indent,
        } => {
            use gitpoulpi_core::textconv::{Charset, ConvertOp, ConvertReport};
            let op = match (encoding, eol, indent) {
                (Some(name), None, None) => match Charset::from_name(&name) {
                    Some(charset) => ConvertOp::Encoding(charset),
                    None => {
                        let known: Vec<&str> = gitpoulpi_core::textconv::ALL
                            .iter()
                            .map(|c| c.name())
                            .collect();
                        return Err(format!(
                            "unknown encoding {name:?}; one of: {}",
                            known.join(", ")
                        ));
                    }
                },
                (None, Some(eol), None) => ConvertOp::Eol(eol.into()),
                (None, None, Some(spec)) => {
                    let (kind, size) = spec.split_once(':').unwrap_or((spec.as_str(), "4"));
                    let Some(size) = size.parse::<usize>().ok().filter(|n| (1..=8).contains(n))
                    else {
                        return Err(format!("the tab width must be 1-8, got {size:?}"));
                    };
                    match kind {
                        "spaces" => ConvertOp::Indent { tabs: false, size },
                        "tabs" => ConvertOp::Indent { tabs: true, size },
                        other => {
                            return Err(format!(
                                "--indent is spaces:<n> or tabs:<n>, got {other:?}"
                            ))
                        }
                    }
                }
                _ => return Err("give exactly one of --encoding, --eol, --indent".into()),
            };
            let report = session
                .call::<ConvertReport>(
                    OperationKind::ConvertFiles,
                    json!({ "paths": paths, "op": op }),
                )
                .await?;
            let human = format!(
                "converted {}, unchanged {}, skipped {}, failed {}",
                report.converted.len(),
                report.unchanged.len(),
                report.skipped.len(),
                report.failed.len()
            );
            Ok(emitted(json!({ "converted": report }), human))
        }

        Commands::Show {
            path,
            at,
            staged,
            worktree,
            raw,
            output,
        } => {
            let source = if staged {
                BlobSource::Index
            } else if worktree {
                BlobSource::WorkingTree
            } else {
                BlobSource::Commit(resolve(session, at.as_deref().unwrap_or("HEAD")).await?)
            };
            if raw {
                return blob::show_raw(session, &path, source, output).await;
            }
            let blob: Blob = session
                .call(
                    OperationKind::ReadBlob,
                    json!({ "path": path, "source": source }),
                )
                .await?;
            // The human rendering is the file itself -- `gitpoulpi show`
            // has to be pipeable into a pager or a diff like `git show`
            // is, so it must not decorate the content with line numbers
            // or a header.
            let human = match &blob.content {
                BlobContent::Text { lines, .. } => lines.join("\n"),
                BlobContent::Binary => match &blob.image {
                    Some(i) if i.width > 0 => format!(
                        "binary file ({:?} image, {}x{}), {} bytes",
                        i.format, i.width, i.height, blob.size
                    ),
                    _ => format!("binary file, {} bytes", blob.size),
                },
                BlobContent::TooLarge => format!("file too large to read, {} bytes", blob.size),
                BlobContent::LfsPointer(pointer) => {
                    gitpoulpi_core::domain::describe_pointer(pointer)
                }
            };
            Ok(emitted(to_value(&blob)?, human))
        }

        Commands::Blame { path, at } => {
            let at = resolve(session, at.as_deref().unwrap_or("HEAD")).await?;
            let blame: Blame = session
                .call(OperationKind::Blame, json!({ "path": path, "at": at }))
                .await?;
            let human = match &blame.content {
                BlameContent::Lines(lines) => lines
                    .iter()
                    .map(|l| {
                        format!(
                            "{} ({:<16} {:>4}) {}",
                            short(l.commit),
                            l.author_name,
                            l.final_line,
                            l.content
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n"),
                BlameContent::Binary => format!("binary file, {} bytes", blame.size),
                BlameContent::TooLarge => format!("file too large to blame, {} bytes", blame.size),
                BlameContent::LfsPointer(pointer) => {
                    gitpoulpi_core::domain::describe_pointer(pointer)
                }
            };
            Ok(emitted(to_value(&blame)?, human))
        }

        Commands::FileHistory {
            path,
            at,
            skip,
            limit,
        } => {
            let start = resolve(session, at.as_deref().unwrap_or("HEAD")).await?;
            let page: FileHistoryPage = session
                .call(
                    OperationKind::FileHistory,
                    json!({ "path": path, "start": start, "skip": skip, "limit": limit }),
                )
                .await?;
            let mut human = page
                .entries
                .iter()
                .map(|e| {
                    let change = match &e.change {
                        FileChangeKind::Added => "A".to_string(),
                        FileChangeKind::Modified => "M".to_string(),
                        FileChangeKind::Deleted => "D".to_string(),
                        FileChangeKind::Renamed { from } => format!("R {}", from.display()),
                    };
                    let stat = match (e.added, e.removed) {
                        (Some(a), Some(r)) => format!("+{a} -{r}"),
                        _ => "binary".to_string(),
                    };
                    format!(
                        "{} {} {} {} ({stat}) {}",
                        short(e.commit),
                        change,
                        e.path.display(),
                        e.author_name,
                        e.summary
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");
            if page.more {
                human.push_str("\n... older commits exist (use --skip)");
            }
            Ok(emitted(to_value(&page)?, human))
        }

        #[cfg(feature = "ai")]
        Commands::Ai {
            action,
            print_sent,
            send_all,
        } => crate::ai_cmd::run(session, action, print_sent, send_all).await,

        Commands::Commit {
            message,
            amend,
            no_verify,
            ai,
            send_all,
            #[cfg(feature = "workspaces")]
            workspace,
        } => {
            if send_all && !ai {
                return Err("--send-all goes with --ai: it sends the whole diff to the AI".into());
            }
            let mut truncated = Vec::new();
            let message = if ai {
                let (message, cuts) =
                    crate::ai_cmd::generate_commit_message(session, send_all).await?;
                truncated = cuts;
                message.full()
            } else {
                message.ok_or("pass -m <message>, or --ai to have one written")?
            };
            // `--workspace` / `GITPOULPI_WORKSPACE`: the identity the workspace
            // gives (`Config::profile_for`). Without one, nothing changes.
            #[cfg(feature = "workspaces")]
            let author = match (
                crate::workspace_cmd::selected_workspace(workspace.as_deref()),
                session.repo_root(),
            ) {
                (Some(id), Some(root)) => crate::workspace_cmd::commit_author(&id, root)?,
                _ => None,
            };
            #[cfg(not(feature = "workspaces"))]
            let author = None;
            let opts = CommitOptions {
                amend,
                sign: false,
                no_verify,
                author,
            };
            let oid: Oid = session
                .call(
                    OperationKind::Commit,
                    json!({ "message": message, "opts": opts }),
                )
                .await?;
            let mut human = format!("committed {}", short(oid));
            let mut out = json!({ "commit": oid });
            crate::ai_cmd::report_truncated(&mut out, &mut human, &truncated);
            Ok(emitted(out, human))
        }

        Commands::Branch { action } => branch(session, action).await,
        #[cfg(feature = "worktree")]
        Commands::Worktree { action } => worktree(session, action).await,
        #[cfg(feature = "submodule")]
        Commands::Submodule { action } => submodule::submodule(session, action).await,
        Commands::Patch { action } => patch(session, action).await,
        Commands::Tag { action } => tag(session, action).await,
        Commands::Stash { action } => stash(session, action).await,
        Commands::Autostash { action } => match action {
            crate::cli::AutoStashAction::StartOver { dry_run, yes } => {
                undo_or_redo(
                    session,
                    OperationKind::AutoStashStartOver,
                    dry_run,
                    None,
                    yes,
                )
                .await
            }
        },
        Commands::Remote { action } => remote(session, action).await,
        Commands::RemoteGroup { action } => upstream::remote_group(session, action).await,
        Commands::SshCommand { action } => ssh_command(session, action).await,
        Commands::RepoAuth { action } => repo_auth::repo_auth(session, action).await,
        Commands::RepoConfig { action } => repo_config(session, action).await,
        Commands::SparseCheckout { action } => sparse_checkout(session, action).await,
        Commands::Conflicts { action } => conflicts(session, action).await,
        Commands::IndexLock { action } => conflicts::index_lock(session, action).await,
        #[cfg(feature = "lfs")]
        Commands::Lfs { action } => lfs(session, action).await,
        #[cfg(feature = "history-rewrite")]
        Commands::Rewrite { action } => rewrite::rewrite(session, action).await,
        Commands::Mailmap { action } => mailmap::mailmap(session, action).await,

        Commands::Checkout { target, autostash } => {
            let target = checkout_target(session, &target).await?;
            session
                .call::<()>(
                    OperationKind::Checkout,
                    with_autostash(json!({ "target": target }), autostash),
                )
                .await?;
            let human = match &target {
                CheckoutTarget::Branch(name) => format!("switched to {name}"),
                CheckoutTarget::Commit(oid) => format!("checked out {} (detached)", short(*oid)),
                CheckoutTarget::File { path, .. } => format!("restored {path}"),
                CheckoutTarget::Files { paths, .. } => format!("restored {} files", paths.len()),
            };
            Ok(emitted(json!({ "checked_out": target }), human))
        }

        Commands::Merge {
            r#ref,
            squash,
            ff,
            no_ff,
            ff_only,
            autostash,
        } => {
            // No flag: `ff` stays `None` and the daemon applies the setting.
            let ff_policy = if ff {
                Some(MergeFastForward::Allowed)
            } else if no_ff {
                Some(MergeFastForward::Never)
            } else if ff_only {
                Some(MergeFastForward::Only)
            } else {
                None
            };
            let from = find_ref(session, &r#ref).await?;
            let outcome: MergeOutcome = session
                .call(
                    OperationKind::Merge,
                    with_autostash(
                        json!({
                            "from": from,
                            "opts": MergeOptions {
                                squash,
                                no_commit: squash,
                                ff: ff_policy,
                                ..MergeOptions::default()
                            },
                        }),
                        autostash,
                    ),
                )
                .await?;
            let human = match &outcome {
                MergeOutcome::FastForwarded(oid) => format!("fast-forwarded to {}", short(*oid)),
                MergeOutcome::Merged(_) if squash => {
                    "squashed: the changes are staged, commit them".to_string()
                }
                MergeOutcome::Merged(oid) => format!("merged as {}", short(*oid)),
                MergeOutcome::UpToDate => "already up to date".to_string(),
                MergeOutcome::Conflicted(paths) => {
                    format!("conflicted: {}", paths.join(", "))
                }
            };
            Ok(emitted(to_value(&outcome)?, human))
        }

        Commands::Rebase {
            onto,
            interactive,
            plan,
            root,
            onto_commit,
            autosquash,
            continue_,
            skip,
            abort,
            status,
            autostash,
        } => {
            rebase_command(
                session,
                onto,
                interactive,
                plan,
                root,
                onto_commit,
                autosquash,
                continue_,
                skip,
                abort,
                status,
                autostash,
            )
            .await
        }

        Commands::RebasePlan { upstream, root } => {
            if root && upstream.is_some() {
                return Err("rebase-plan: give either an upstream or --root, not both".into());
            }
            let upstream = match upstream {
                Some(u) if !root => Some(resolve(session, &u).await?),
                _ => None,
            };
            let draft: gitpoulpi_core::domain::RebasePlanDraft = session
                .call(OperationKind::RebasePlan, json!({ "upstream": upstream }))
                .await?;
            Ok(emitted(
                to_value(&draft)?,
                format!("{} commit(s) in range", draft.commits.len()),
            ))
        }

        Commands::CherryPick { commits, plan } => {
            let mut oids = Vec::with_capacity(commits.len());
            if let Some(plan) = plan {
                if !commits.is_empty() {
                    return Err("cherry-pick: give either commits or --plan, not both".into());
                }
                for step in read_plan_steps(&plan)? {
                    match step.action {
                        RebaseAction::Pick => oids.push(step.commit),
                        RebaseAction::Drop => {}
                        other => {
                            return Err(format!(
                                "cherry-pick --plan only supports the Pick and Drop actions, \
                                 got {other:?} for {}",
                                short(step.commit)
                            ));
                        }
                    }
                }
                if oids.is_empty() {
                    return Err("cherry-pick: the plan picks no commit".into());
                }
            } else {
                if commits.is_empty() {
                    return Err("cherry-pick needs at least one commit or --plan".into());
                }
                for rev in &commits {
                    oids.push(resolve(session, rev).await?);
                }
            }
            let outcome: CherryPickOutcome = session
                .call(
                    OperationKind::CherryPick,
                    json!({ "commits": oids, "opts": CherryPickOptions::default() }),
                )
                .await?;
            let human = match &outcome {
                CherryPickOutcome::Completed(applied) => format!(
                    "applied {}",
                    applied
                        .iter()
                        .map(|o| short(*o))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                CherryPickOutcome::Conflicted { paused_at, .. } => {
                    format!("paused on a conflict at {}", short(*paused_at))
                }
            };
            Ok(emitted(to_value(&outcome)?, human))
        }

        Commands::Revert { commit, mainline } => {
            let oid = resolve(session, &commit).await?;
            let outcome: RevertOutcome = session
                .call(
                    OperationKind::Revert,
                    json!({ "commit": oid, "mainline": mainline }),
                )
                .await?;
            let human = match &outcome {
                RevertOutcome::Completed(oid) => format!("reverted as {}", short(*oid)),
                RevertOutcome::Conflicted => "reverted with conflicts to resolve".to_string(),
            };
            Ok(emitted(to_value(&outcome)?, human))
        }

        Commands::ApplyCommit { commit } => {
            let oid = resolve(session, &commit).await?;
            let applied: gitpoulpi_core::domain::PatchApplied = session
                .call(OperationKind::ApplyCommit, json!({ "commit": oid }))
                .await?;
            let human = format!(
                "applied the changes of {} to the working tree ({} file(s))",
                short(oid),
                applied.files.len()
            );
            Ok(emitted(to_value(&applied)?, human))
        }

        Commands::Reword { commit, message } => {
            let oid = resolve(session, &commit).await?;
            let outcome: RewordOutcome = session
                .call(
                    OperationKind::Reword,
                    json!({ "commit": oid, "message": message }),
                )
                .await?;
            let human = format!(
                "reworded {} as {} ({} descendant(s) replayed; was {}, `gitpoulpi reset {} --mode soft` puts it back)",
                short(oid),
                short(outcome.reworded),
                outcome.replayed,
                short(outcome.previous_head),
                short(outcome.previous_head),
            );
            Ok(emitted(to_value(&outcome)?, human))
        }

        Commands::Amend {
            commit,
            message,
            summary,
            description,
            author_name,
            author_email,
            author_date,
            committer_name,
            committer_email,
            committer_date,
        } => {
            let oid = resolve(session, &commit).await?;
            // A summary-only or description-only change is a message
            // change: build it from the commit's current message.
            let message = if summary.is_some() || description.is_some() {
                let opts = LogOptions {
                    limit: Some(1),
                    start_ref: Some(oid.to_hex()),
                    ..LogOptions::default()
                };
                let log: CommitGraph = session
                    .call(OperationKind::Log, json!({ "opts": opts }))
                    .await?;
                let current = log
                    .nodes
                    .into_iter()
                    .next()
                    .map(|n| n.message)
                    .ok_or_else(|| format!("commit {} not found", short(oid)))?;
                let (old_summary, old_body) = split_message(&current);
                let summary = summary.unwrap_or(old_summary);
                let body = description.unwrap_or(old_body);
                Some(if body.trim().is_empty() {
                    summary
                } else {
                    format!("{summary}\n\n{body}")
                })
            } else {
                message
            };
            let amend = CommitAmend {
                message,
                author_name,
                author_email,
                author_date,
                committer_name,
                committer_email,
                committer_date,
            };
            if amend == CommitAmend::default() {
                return Err("nothing to amend: give at least one of --message, --summary, --description, --author-name, --author-email, --author-date or a --committer-* option".into());
            }
            let outcome: RewordOutcome = session
                .call(
                    OperationKind::AmendCommit,
                    json!({ "commit": oid, "amend": amend }),
                )
                .await?;
            let human = format!(
                "amended {} as {} ({} descendant(s) replayed; was {}, `gitpoulpi undo` or `gitpoulpi reset {} --mode soft` puts it back)",
                short(oid),
                short(outcome.reworded),
                outcome.replayed,
                short(outcome.previous_head),
                short(outcome.previous_head),
            );
            Ok(emitted(to_value(&outcome)?, human))
        }

        Commands::SplitSource { commit } => split::source(session, &commit).await,

        Commands::SplitCommit {
            commit,
            plan,
            rest_message,
            sign,
            dry_run,
        } => split::commit(session, &commit, &plan, rest_message, sign, dry_run).await,

        Commands::Undo {
            dry_run,
            list,
            all,
            expect,
            yes,
        } => {
            if list {
                journal_listing(session, all).await
            } else {
                undo_or_redo(session, OperationKind::Undo, dry_run, expect, yes).await
            }
        }

        Commands::UndoForget {
            dry_run,
            redo,
            expect,
            yes,
        } => {
            undo::forget(
                session,
                if redo {
                    OperationKind::Redo
                } else {
                    OperationKind::Undo
                },
                dry_run,
                expect,
                yes,
            )
            .await
        }

        Commands::Redo {
            dry_run,
            expect,
            yes,
        } => undo_or_redo(session, OperationKind::Redo, dry_run, expect, yes).await,

        Commands::UndoRestore { id, name, dry_run } => {
            undo::restore(session, id, name, dry_run).await
        }
        Commands::Reflog {
            reference,
            skip,
            limit,
        } => undo::reflog(session, reference, skip, limit).await,
        Commands::ReflogRestore {
            reference,
            oid,
            mode,
            dry_run,
        } => {
            let oid = resolve(session, &oid).await?;
            undo::reflog_restore(session, reference, oid, mode.map(|m| m.as_str()), dry_run).await
        }
        Commands::LostCommits {
            skip,
            limit,
            no_deep,
            budget_ms,
        } => undo::lost_commits(session, skip, limit, !no_deep, budget_ms).await,

        Commands::Reset { target, mode } => {
            let oid = resolve(session, &target).await?;
            let mode = match mode {
                ResetModeArg::Soft => ResetMode::Soft,
                ResetModeArg::Mixed => ResetMode::Mixed,
                ResetModeArg::Hard => ResetMode::Hard,
            };
            session
                .call::<()>(OperationKind::Reset, json!({ "target": oid, "mode": mode }))
                .await?;
            Ok(emitted(
                json!({ "reset_to": oid, "mode": mode }),
                format!("reset {:?} to {}", mode, short(oid)),
            ))
        }

        Commands::Fetch {
            remote,
            all,
            prune,
            prune_tags,
        } => {
            // The flag only turns pruning on; `general.auto_prune` (on by default)
            // decides otherwise, the same setting the TUI's fetches follow.
            let opts = FetchOptions {
                prune: prune || config_auto_prune(),
                prune_tags,
            };
            let (outcome, from): (FetchOutcome, String) = if all {
                let outcome = session
                    .call(OperationKind::FetchAll, json!({ "opts": opts }))
                    .await?;
                (outcome, "every remote".to_string())
            } else {
                let remote = default_remote(session, remote).await?;
                let outcome = session
                    .call(
                        OperationKind::Fetch,
                        json!({ "remote": remote, "opts": opts }),
                    )
                    .await?;
                (outcome, remote)
            };
            let mut human = if outcome.updated_refs.is_empty() {
                format!("fetched {from}: nothing new")
            } else {
                format!("fetched {from}: {}", outcome.updated_refs.join(", "))
            };
            if !outcome.pruned_tags.is_empty() {
                human.push_str(&format!(
                    "; deleted local tags the remote lost: {}",
                    outcome.pruned_tags.join(", ")
                ));
            }
            Ok(emitted(to_value(&outcome)?, human))
        }

        Commands::Pull {
            all_sources: true,
            mode,
            autostash,
            branch,
            resolve,
            no_preflight,
            ..
        } => {
            group_run::run_group(
                session,
                group_run::Group {
                    mode: gitpoulpi_core::domain::GroupMode::Pull,
                    branch,
                    to: Vec::new(),
                    resolve,
                    no_preflight,
                    tags: false,
                    yes: false,
                    autostash,
                    pull_mode: Some(mode),
                },
            )
            .await
        }
        Commands::Pull {
            remote,
            mode,
            autostash,
            ..
        } => {
            let opts = PullOptions {
                mode: match mode {
                    PullModeArg::Merge => PullMode::Merge,
                    PullModeArg::FfOnly => PullMode::FastForwardOnly,
                    PullModeArg::Rebase => PullMode::Rebase,
                },
                remote,
                ..PullOptions::default()
            };
            let outcome: PullOutcome = session
                .call(
                    OperationKind::Pull,
                    with_autostash(json!({ "opts": opts }), autostash),
                )
                .await?;
            let human = match &outcome {
                PullOutcome::UpToDate => "already up to date".to_string(),
                PullOutcome::FastForwarded(oid) => format!("fast-forwarded to {}", short(*oid)),
                PullOutcome::Merged(oid) if opts.mode == PullMode::Rebase => {
                    format!("rebased onto the upstream, now at {}", short(*oid))
                }
                PullOutcome::Merged(oid) => format!("merged as {}", short(*oid)),
                PullOutcome::Conflicted => "pulled with conflicts to resolve".to_string(),
            };
            Ok(emitted(to_value(&outcome)?, human))
        }

        Commands::Push {
            branch,
            all_targets,
            to,
            resolve,
            no_preflight,
            tags,
            yes,
            ..
        } if all_targets || !to.is_empty() => {
            group_run::run_group(
                session,
                group_run::Group {
                    mode: gitpoulpi_core::domain::GroupMode::Push,
                    branch,
                    to,
                    resolve,
                    no_preflight,
                    tags,
                    yes,
                    autostash: false,
                    pull_mode: None,
                },
            )
            .await
        }
        Commands::Push {
            remote,
            branch,
            set_upstream,
            force_with_lease,
            ..
        } => {
            let remote = default_remote(session, remote).await?;
            let opts = PushOptions {
                force_with_lease,
                set_upstream,
                branch,
                ..PushOptions::default()
            };
            let outcome: PushOutcome = session
                .call(
                    OperationKind::Push,
                    json!({ "remote": remote, "opts": opts }),
                )
                .await?;
            let human = format!("pushed {} to {remote}", outcome.pushed_refs.join(", "));
            Ok(emitted(to_value(&outcome)?, human))
        }

        Commands::Clone {
            url,
            dest,
            branch,
            origin,
            depth,
            recurse_submodules,
            sparse,
            #[cfg(feature = "lfs")]
            no_lfs,
        } => {
            // Made absolute *here*: the daemon that performs the clone may
            // be a long-running one started in another directory, so a
            // relative destination would land wherever that daemon lives.
            let dest = absolute(dest.unwrap_or_else(|| default_clone_dest(&url)))?;
            let opts = CloneOptions {
                shallow_depth: depth,
                recurse_submodules,
                branch,
                remote_name: origin,
                sparse: (!sparse.is_empty()).then_some(sparse),
                #[cfg(feature = "lfs")]
                lfs: if no_lfs {
                    gitpoulpi_core::domain::LfsCloneMode::Skip
                } else {
                    gitpoulpi_core::domain::LfsCloneMode::Auto
                },
                #[cfg(not(feature = "lfs"))]
                lfs: gitpoulpi_core::domain::LfsCloneMode::Auto,
            };
            let repo: RepoRef = session
                .call_without_repo(
                    OperationKind::Clone,
                    json!({ "url": url, "dest": dest, "opts": opts }),
                )
                .await?;
            let human = format!("cloned into {}", repo.root.display());
            Ok(emitted(to_value(&repo)?, human))
        }

        // `repo` and `hosting` carry their own state as well as their
        // own IPC calls, so they are routed by `run` rather than
        // dispatched here -- see the two modules.
        Commands::Repo { .. } | Commands::Ui { .. } | Commands::Job { .. } => {
            unreachable!("routed before dispatch")
        }
        #[cfg(feature = "workspaces")]
        Commands::Workspace { .. } => unreachable!("routed before dispatch"),
        #[cfg(feature = "global-search")]
        Commands::Search { .. } => unreachable!("routed before dispatch"),
        #[cfg(feature = "hosting")]
        Commands::Hosting { .. } => unreachable!("routed before dispatch"),

        Commands::Refs { action } => refs(session, action).await,
        Commands::Flow { action } => flow(session, action).await,

        Commands::ShellInit { .. } => Err(
            "`shell-init` prints a shell script and is only available from the gitpoulpi binary"
                .into(),
        ),
        Commands::Desktop { .. } => {
            Err("`desktop` is only available from the gitpoulpi binary".into())
        }
        Commands::Launch { .. } => {
            Err("`launch` is only available from the gitpoulpi binary".into())
        }
        Commands::Uninstall { .. } => {
            Err("`uninstall` is only available from the gitpoulpi binary".into())
        }
        Commands::Completions { .. } => Err(
            "`completions` prints a shell script and is only available from the gitpoulpi binary"
                .into(),
        ),
        #[cfg(feature = "mcp")]
        Commands::Mcp { .. } => {
            Err("`mcp serve` is only available from the gitpoulpi binary".into())
        }

        Commands::Daemon { .. } => {
            Err("`daemon` commands are only available from the gitpoulpi binary".into())
        }

        Commands::Session { .. }
        | Commands::Config { .. }
        | Commands::Theme { .. }
        | Commands::Licenses
        | Commands::Alias { .. }
        | Commands::Installation { .. }
        | Commands::Path { .. } => {
            unreachable!("local commands are handled before dispatch")
        }
    }
}

/// The single bridge from a typed revision to an `Oid`, via the backend.
async fn resolve(session: &mut Session, rev: &str) -> Result<Oid, String> {
    session
        .call(OperationKind::Resolve, json!({ "rev": rev }))
        .await
}

/// A name that is a local branch checks out as a branch; anything else is
/// resolved and checked out as a (detached) commit, which is what `git
/// checkout <sha>` does.
async fn checkout_target(session: &mut Session, name: &str) -> Result<CheckoutTarget, String> {
    let refs = all_refs(session).await?;
    let is_local_branch = refs
        .iter()
        .any(|r| matches!(&r.kind, RefKind::LocalBranch { name: n } if n == name));
    if is_local_branch {
        return Ok(CheckoutTarget::Branch(name.to_string()));
    }
    Ok(CheckoutTarget::Commit(resolve(session, name).await?))
}

/// `.gitpoulpi/repo-state.toml` for the repository at `root`, honouring
/// `session.per_repo_state_dir` - the same file the TUI writes.
pub(crate) fn repo_state_path(root: &std::path::Path) -> std::path::PathBuf {
    let state_dir = gitpoulpi_config::Settings::resolve_ambient()
        .map(|s| s.config().session.per_repo_state_dir.clone())
        .unwrap_or_else(|_| gitpoulpi_config::DEFAULT_REPO_DATA_LOCATION.to_string());
    gitpoulpi_session::RepoState::path_for(root, &state_dir)
}

pub(crate) fn absolute(path: String) -> Result<std::path::PathBuf, String> {
    let path = std::path::PathBuf::from(path);
    if path.is_absolute() {
        return Ok(path);
    }
    let cwd = std::env::current_dir()
        .map_err(|e| format!("could not read the current directory: {e}"))?;
    Ok(cwd.join(path))
}

fn default_clone_dest(url: &str) -> String {
    url.trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or("repo")
        .trim_end_matches(".git")
        .to_string()
}

fn stage_target(
    file: Option<String>,
    folder: Option<String>,
    hunk: Option<u32>,
    lines: Option<String>,
    paths: Vec<String>,
    all: bool,
) -> Result<StageTarget, String> {
    if !paths.is_empty() {
        let mut all_paths: Vec<std::path::PathBuf> = file.into_iter().map(Into::into).collect();
        all_paths.extend(paths.into_iter().map(std::path::PathBuf::from));
        return Ok(StageTarget::Files(all_paths));
    }
    if all {
        return Ok(StageTarget::All);
    }
    if let Some(folder) = folder {
        return Ok(StageTarget::Folder(folder.into()));
    }
    let Some(file) = file else {
        return Err("pass one of --file, --folder or --all".into());
    };
    match (hunk, lines) {
        (Some(hunk), Some(lines)) => {
            let line_indices = lines
                .split(',')
                .map(|s| {
                    s.trim()
                        .parse::<u32>()
                        .map_err(|_| format!("`{s}` is not a line index"))
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(StageTarget::Line(
                file.into(),
                LineSelector {
                    hunk: HunkId(hunk),
                    line_indices,
                },
            ))
        }
        // A line selection is always *within* a hunk, so `--lines` without
        // `--hunk` has no answer to "lines of what?" -- rejected rather
        // than silently staging the whole file.
        (None, Some(_)) => Err("--lines also needs --hunk".into()),
        (Some(hunk), None) => Ok(StageTarget::Hunk(file.into(), HunkId(hunk))),
        (None, None) => Ok(StageTarget::File(file.into())),
    }
}

/// `gitpoulpi discard`'s target: a hunk or some of its lines
/// (`GitBackend::discard_part`), or `None` for the whole file
/// (`GitBackend::discard`). Reuses `stage_target`'s parsing so `--lines`
/// means the same thing in `stage`, `unstage` and `discard`.
fn discard_target(
    file: String,
    hunk: Option<u32>,
    lines: Option<String>,
) -> Result<Option<DiscardTarget>, String> {
    Ok(
        match stage_target(Some(file), None, hunk, lines, Vec::new(), false)? {
            StageTarget::Hunk(path, hunk) => Some(DiscardTarget::Hunk(path, hunk)),
            StageTarget::Line(path, selector) => Some(DiscardTarget::Line(path, selector)),
            _ => None,
        },
    )
}

/// The refusal of a stage that would add sensitive files, as the CLI and MCP
/// report it: `{error: "sensitive_files", files}` (a rule and a masked
/// preview each, never a value) and exit code 2, so a caller can tell
/// "refused" from "failed". `--allow-sensitive` is the retry.
fn sensitive_refusal(files: &[gitpoulpi_core::privacy::Sensitive]) -> Emitted {
    let mut human = String::from("refused: these files look sensitive and were not staged:");
    for f in files {
        human.push_str(&format!("\n  {} ({})", f.path, f.describe()));
    }
    human.push_str("\nStage them anyway with --allow-sensitive (ask the user first).");
    let mut out = emitted(
        json!({
            "error": "sensitive_files",
            "files": files,
            "hint": "ask the user whether these files may be staged; the user stages them with --allow-sensitive",
        }),
        human,
    );
    out.exit_code = 2;
    out
}

/// `OperationKind` splits staging by granularity while `GitBackend::stage`
/// takes one `StageTarget`; the daemon only needs the kind to decide which
/// method to call, so this keeps the two in step.
fn stage_kind(target: &StageTarget, staging: bool) -> OperationKind {
    match (target, staging) {
        (StageTarget::All, true) => OperationKind::StageAll,
        (StageTarget::Folder(_), true) => OperationKind::StageFolder,
        (StageTarget::Hunk(..), true) => OperationKind::StageHunk,
        (StageTarget::Line(..), true) => OperationKind::StageLine,
        (StageTarget::File(_) | StageTarget::Files(_), true) => OperationKind::StageFile,
        (StageTarget::All, false) => OperationKind::UnstageAll,
        (StageTarget::Folder(_), false) => OperationKind::UnstageFolder,
        (StageTarget::Hunk(..), false) => OperationKind::UnstageHunk,
        (StageTarget::Line(..), false) => OperationKind::UnstageLine,
        (StageTarget::File(_) | StageTarget::Files(_), false) => OperationKind::UnstageFile,
    }
}

fn to_value<T: serde::Serialize>(value: &T) -> Result<Value, String> {
    serde_json::to_value(value).map_err(|e| format!("failed to encode output: {e}"))
}

/// `--autostash`: tell the daemon to stash local changes that block the
/// operation whatever `general.auto_stash` says. Absent, the config decides.
pub(super) fn with_autostash(mut args: serde_json::Value, autostash: bool) -> serde_json::Value {
    if autostash {
        args["autostash"] = json!("always");
    }
    args
}

/// `general.auto_prune` from the ambient config (on when it cannot be read):
/// the CLI's fetches follow the same setting the TUI does, so a branch deleted
/// on the server does not linger in `branch list --remote`.
pub(crate) fn config_auto_prune() -> bool {
    gitpoulpi_config::Settings::resolve_ambient()
        .map(|s| s.config().general.auto_prune)
        .unwrap_or(true)
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    #[test]
    fn local_commands_are_the_ones_that_need_no_daemon() {
        let local = |args: &[&str]| {
            let cli = crate::cli::Cli::try_parse_from(
                std::iter::once("gitpoulpi").chain(args.iter().copied()),
            )
            .unwrap();
            super::is_local(&cli.command.unwrap())
        };
        assert!(local(&["repo", "recent"]));
        assert!(local(&["session", "list"]));
        assert!(!local(&["status"]));
    }

    #[cfg(feature = "hosting")]
    #[test]
    fn hosting_list_is_local_and_hosting_prs_is_not() {
        let local = |args: &[&str]| {
            let cli = crate::cli::Cli::try_parse_from(
                std::iter::once("gitpoulpi").chain(args.iter().copied()),
            )
            .unwrap();
            super::is_local(&cli.command.unwrap())
        };
        assert!(local(&["hosting", "list"]));
        assert!(!local(&["hosting", "prs"]));
    }

    use super::stash::stash_id;
    use super::*;
    use gitpoulpi_core::domain::StashId;

    #[test]
    fn stage_target_covers_every_granularity() {
        assert_eq!(
            stage_target(None, None, None, None, Vec::new(), true).unwrap(),
            StageTarget::All
        );
        assert_eq!(
            stage_target(Some("a.txt".into()), None, None, None, Vec::new(), false).unwrap(),
            StageTarget::File("a.txt".into())
        );
        assert_eq!(
            stage_target(None, Some("src".into()), None, None, Vec::new(), false).unwrap(),
            StageTarget::Folder("src".into())
        );
        assert_eq!(
            stage_target(Some("a.txt".into()), None, Some(2), None, Vec::new(), false).unwrap(),
            StageTarget::Hunk("a.txt".into(), HunkId(2))
        );
        assert_eq!(
            stage_target(
                Some("a.txt".into()),
                None,
                Some(2),
                Some("0, 3".into()),
                Vec::new(),
                false
            )
            .unwrap(),
            StageTarget::Line(
                "a.txt".into(),
                LineSelector {
                    hunk: HunkId(2),
                    line_indices: vec![0, 3]
                }
            )
        );
        assert!(stage_target(None, None, None, None, Vec::new(), false).is_err());
        assert!(stage_target(
            Some("a.txt".into()),
            None,
            None,
            Some("1".into()),
            Vec::new(),
            false
        )
        .is_err());
    }

    #[test]
    fn discard_takes_a_hunk_its_lines_or_the_whole_file() {
        assert_eq!(
            discard_target("a.txt".into(), Some(1), None).unwrap(),
            Some(DiscardTarget::Hunk("a.txt".into(), HunkId(1)))
        );
        assert_eq!(
            discard_target("a.txt".into(), Some(1), Some("2,4".into())).unwrap(),
            Some(DiscardTarget::Line(
                "a.txt".into(),
                LineSelector {
                    hunk: HunkId(1),
                    line_indices: vec![2, 4]
                }
            ))
        );
        assert_eq!(discard_target("a.txt".into(), None, None).unwrap(), None);
        assert!(discard_target("a.txt".into(), None, Some("1".into())).is_err());
    }

    #[test]
    fn every_stage_target_maps_to_its_own_operation_kind() {
        // The daemon dispatches on the kind, so a mismatch here would
        // route a hunk stage through the whole-file arm.
        assert_eq!(stage_kind(&StageTarget::All, true), OperationKind::StageAll);
        assert_eq!(
            stage_kind(&StageTarget::Hunk("a".into(), HunkId(0)), false),
            OperationKind::UnstageHunk
        );
    }

    #[test]
    fn stash_ids_accept_both_spellings() {
        assert_eq!(stash_id(None).unwrap(), StashId(0));
        assert_eq!(stash_id(Some("3".into())).unwrap(), StashId(3));
        assert_eq!(stash_id(Some("stash@{3}".into())).unwrap(), StashId(3));
        assert!(stash_id(Some("nope".into())).is_err());
    }

    #[test]
    fn clone_dest_defaults_to_the_repo_name() {
        assert_eq!(
            default_clone_dest("https://example.invalid/foo/bar.git"),
            "bar"
        );
        assert_eq!(
            default_clone_dest("https://example.invalid/foo/bar/"),
            "bar"
        );
    }
}
