//! `gitpoulpi rewrite`: history rewrite from a recipe (the history-rewrite design).
//!
//! `preview` keeps a run in the daemon (a plan plus its digest), `rows` pages
//! it, `apply` runs the reviewed plan as one journal entry, `run` does both
//! for scripts, `discard` drops a run. Refusals are typed
//! (`RewriteRefused` -> exit code 2, `NeedsConfirmation` -> exit code 3);
//! only `push` sends anything, with leases, and only with `--yes`. Secrets come from `--secrets-file`, never from the
//! command line, and are never echoed.

use gitpoulpi_core::domain::rewrite::script::{parse_script_failure, ScriptCheck, ScriptTestRow};
use gitpoulpi_core::domain::rewrite::{script_hash, ScriptTrust};
use gitpoulpi_core::domain::rewrite::{
    BlobChange, BlobLineKind, CommitChange, CommitterPolicy, ContentFind, ContentReplace,
    ExtractMode, IdentityMatch, IdentityRule, IdentitySet, PathMove, PlanDigest, PruneEmpty,
    PushRefStatus, PushRewritten, PushRewrittenOutcome, Recipe, RewriteMaps, RewriteNote,
    RewriteOutcome, RewritePlan, RewriteRefusal, RewriteTools, Rule, RuleKind, RunId, Scope,
    SecretScan, SecretText, SignaturePolicy, SigningCheck, SizeAnalysis, TextPattern, TextReplace,
    TrailerRule, WhichDate, Who,
};
use gitpoulpi_core::domain::rewrite::{LfsExportRule, LfsImportRule};
use gitpoulpi_core::domain::rewrite::{PurgeCheck, PurgeOutcome, PurgeStatus};
use gitpoulpi_core::domain::rewrite::{ScriptLanguage, ScriptRef, ScriptSource};
use gitpoulpi_core::domain::{Oid, OperationKind};
use gitpoulpi_core::TypedError;
use gitpoulpi_ipc::rewrite::{
    AnalyzeArgs, ApplyArgs, BlobDiffArgs, MapsArgs, NoteArgs, PreviewArgs, PurgeArgs, RowFilter,
    RowsArgs, RowsPage, ScanSecretsArgs, ScriptCheckArgs, ScriptTestArgs, ToolsArgs, TrustArgs,
};
use gitpoulpi_ipc::ResponseError;
use serde_json::{json, Value};

use crate::cli::{RewriteAction, RewriteRecipeArgs, RewriteScriptAction};
use crate::client::Session;

use super::*;

pub(super) async fn rewrite(
    session: &mut Session,
    action: RewriteAction,
) -> Result<Emitted, String> {
    match action {
        RewriteAction::Tools { fresh } => {
            let tools: RewriteTools = session
                .call(OperationKind::RewriteTools, to_value(&ToolsArgs { fresh })?)
                .await?;
            Ok(emitted(
                to_value(&tools)?,
                format!("{} tool check(s)", tools.tools.len()),
            ))
        }
        RewriteAction::CheckSigning => {
            let check: SigningCheck = session
                .call(OperationKind::RewriteCheckSigning, serde_json::Value::Null)
                .await?;
            if let Some(reason) = check.reason {
                // Refused (exit 2), like the preview that would re-sign.
                return Ok(refused(
                    reason,
                    format!("{}: {}", check.tool.name(), check.detail),
                ));
            }
            let summary = format!(
                "signing works with {} ({} ms per signature)",
                check.tool.name(),
                check.millis
            );
            Ok(emitted(to_value(&check)?, summary))
        }
        RewriteAction::ScanSecrets {
            all_local,
            refs,
            since,
        } => {
            let scope = if all_local {
                Scope::AllLocal
            } else if !refs.is_empty() {
                Scope::Refs { refs }
            } else {
                let since = match &since {
                    Some(rev) => Some(resolve(session, rev).await?),
                    None => None,
                };
                Scope::CurrentBranch { since }
            };
            match call_typed::<SecretScan>(
                session,
                OperationKind::RewriteScanSecrets,
                to_value(&ScanSecretsArgs { scope })?,
            )
            .await?
            {
                Err(answer) => Ok(answer),
                Ok(scan) => Ok(emitted(to_value(&scan)?, scan_human(&scan))),
            }
        }
        RewriteAction::Analyze {
            all_local,
            refs,
            since,
            top,
        } => {
            let scope = if all_local {
                Scope::AllLocal
            } else if !refs.is_empty() {
                Scope::Refs { refs }
            } else {
                let since = match &since {
                    Some(rev) => Some(resolve(session, rev).await?),
                    None => None,
                };
                Scope::CurrentBranch { since }
            };
            match call_typed::<SizeAnalysis>(
                session,
                OperationKind::RewriteAnalyze,
                to_value(&AnalyzeArgs { scope, top })?,
            )
            .await?
            {
                Err(answer) => Ok(answer),
                Ok(found) => Ok(emitted(to_value(&found)?, analysis_human(&found))),
            }
        }
        RewriteAction::Preview { recipe, limit, all } => {
            let recipe = build_recipe(session, &recipe).await?;
            // S23: a linked repository's host says which branches it protects.
            let recipe = rewrite_host::with_host(session, &recipe).await;
            let plan = match preview(session, &recipe).await? {
                Err(refusal) => return Ok(refusal),
                Ok(plan) => plan,
            };
            let limit = if all { usize::MAX } else { limit };
            let (rows, total) = fetch_rows(session, &plan, limit).await?;
            let mut value = to_value(&plan)?;
            value["rows"] = to_value(&rows)?;
            value["rows_total"] = json!(total);
            let mut human = format!(
                "{}\nrun {} digest {}\n`gitpoulpi rewrite apply --run {} --digest {}` applies it",
                plan_human(&plan),
                plan.run.0,
                plan.digest.0,
                plan.run.0,
                plan.digest.0
            );
            if let Some(host) = &recipe.scope_options.host_protection {
                let blocked = rewrite_host::plan_blocked(host, &plan);
                if !blocked.is_empty() {
                    value["protected_by_host"] = rewrite_host::blocked_json(host, &blocked);
                    human.push_str(&format!(
                        "\n\n{}",
                        value["protected_by_host"]["text"]
                            .as_str()
                            .unwrap_or_default()
                    ));
                }
            }
            Ok(emitted(value, human))
        }
        RewriteAction::Rows {
            run,
            filter,
            offset,
            limit,
        } => {
            let args = RowsArgs {
                run: RunId(run),
                filter: parse_filter(&filter),
                offset,
                limit,
            };
            match call_typed::<RowsPage>(session, OperationKind::RewriteRows, to_value(&args)?)
                .await?
            {
                Err(answer) => Ok(answer),
                Ok(page) => {
                    let human = format!(
                        "{} row(s) from {} of {}",
                        page.rows.len(),
                        page.offset,
                        page.total
                    );
                    Ok(emitted(to_value(&page)?, human))
                }
            }
        }
        RewriteAction::Apply {
            run,
            digest,
            yes,
            target,
        } => {
            let args = ApplyArgs {
                run: RunId(run),
                digest: PlanDigest(digest),
                confirmed: yes,
                target: target.as_deref().map(absolute).transpose()?,
            };
            apply(session, &args).await
        }
        RewriteAction::Run { recipe: flags, yes } => {
            let recipe = build_recipe(session, &flags).await?;
            let recipe = rewrite_host::with_host(session, &recipe).await;
            let target = extraction_target(&recipe, flags.target.as_deref())?;
            let plan = match preview(session, &recipe).await? {
                Err(refusal) => return Ok(refusal),
                Ok(plan) => plan,
            };
            let args = ApplyArgs {
                run: plan.run.clone(),
                digest: plan.digest.clone(),
                confirmed: yes,
                target,
            };
            let applied = apply(session, &args).await?;
            if applied.exit_code != 0 {
                // Leave no run behind for a script that cannot answer.
                discard(session, &plan.run).await.ok();
                return Ok(applied);
            }
            let human = format!("{}\n{}", plan_human(&plan), applied.human);
            Ok(emitted(
                json!({ "plan": plan, "outcome": applied.json }),
                human,
            ))
        }
        RewriteAction::Push {
            run,
            remote,
            include_protected,
            include_all_protected,
            dry_run,
            no_atomic,
            yes,
        } => {
            let mut request = PushRewritten::new(RunId(run));
            request.remote = remote;
            request.include_protected = include_protected;
            request.include_all_protected = include_all_protected;
            request.atomic = !no_atomic;
            push(session, request, dry_run, yes).await
        }
        RewriteAction::Note { run } => {
            let args = NoteArgs { run: RunId(run) };
            match call_typed::<RewriteNote>(session, OperationKind::RewriteNote, to_value(&args)?)
                .await?
            {
                Err(answer) => Ok(answer),
                Ok(note) => {
                    let mut value = to_value(&note)?;
                    let mut human = note.text.clone();
                    // S23: the branches of the note that the host protects.
                    if let Some(host) = rewrite_host::host_for(session, None, true).await {
                        let blocked = rewrite_host::note_blocked(&host, &note);
                        if !blocked.is_empty() {
                            value["protected_by_host"] =
                                rewrite_host::blocked_json(&host, &blocked);
                            human.push_str(&format!(
                                "\n\n{}",
                                value["protected_by_host"]["text"]
                                    .as_str()
                                    .unwrap_or_default()
                            ));
                        }
                    }
                    Ok(emitted(value, human))
                }
            }
        }
        RewriteAction::Maps { run, out } => {
            let args = MapsArgs {
                run: RunId(run),
                out: out.map(std::path::PathBuf::from),
            };
            match call_typed::<RewriteMaps>(session, OperationKind::RewriteMaps, to_value(&args)?)
                .await?
            {
                Err(answer) => Ok(answer),
                Ok(maps) => {
                    let human =
                        format!("{}\n{}", maps.commit_map.display(), maps.ref_map.display());
                    Ok(emitted(to_value(&maps)?, human))
                }
            }
        }
        RewriteAction::Diff { run, commit, path } => {
            let commit = resolve(session, &commit).await?;
            let args = BlobDiffArgs {
                run: RunId(run),
                commit,
                path,
            };
            match call_typed::<BlobChange>(
                session,
                OperationKind::RewriteBlobDiff,
                to_value(&args)?,
            )
            .await?
            {
                Err(answer) => Ok(answer),
                Ok(change) => Ok(emitted(to_value(&change)?, diff_human(&change))),
            }
        }
        RewriteAction::Script { action } => script(session, action).await,
        RewriteAction::Purge {
            entries,
            latest: _,
            all,
            yes,
        } => purge(session, PurgeArgs { entries, all }, yes).await,
        RewriteAction::Discard { run } => {
            let run = RunId(run);
            discard(session, &run).await?;
            Ok(emitted(
                json!({ "discarded": run }),
                format!("discarded {}", run.0),
            ))
        }
    }
}

/// The findings as lines. Redacted previews only.
fn scan_human(scan: &SecretScan) -> String {
    let mut lines: Vec<String> = scan
        .findings
        .iter()
        .map(|f| {
            let more = if f.more_paths > 0 {
                format!(" (+{} more file(s))", f.more_paths)
            } else {
                String::new()
            };
            format!(
                "{}  {}  {}{}  {}  {} commit(s) {}..{}, {} blob(s)",
                f.id,
                f.rule,
                f.path,
                more,
                f.preview,
                f.commits,
                &f.first_commit.to_hex()[..7],
                &f.last_commit.to_hex()[..7],
                f.blobs
            )
        })
        .collect();
    lines.push(format!(
        "{} finding(s) in {} commit(s), {} blob(s) read ({} binary, {} too large skipped). Not exhaustive.",
        scan.findings.len(),
        scan.commits,
        scan.blobs_scanned,
        scan.blobs_skipped_binary,
        scan.blobs_skipped_large
    ));
    if !scan.findings.is_empty() {
        lines.push(
            "A secret that was pushed is leaked: rotate it first. Remove one from history with `rewrite preview --remove-finding ID`.".into(),
        );
    }
    lines.join("\n")
}

/// The analysis as lines: totals, then each list.
fn analysis_human(a: &SizeAnalysis) -> String {
    use gitpoulpi_core::domain::rewrite::{human_size, BlobState};
    let short = |id: &gitpoulpi_core::domain::Oid| id.to_hex()[..7].to_string();
    let mut out = vec![format!(
        "{} commit(s), {} unique blob(s), {} ({} on disk); {} in HEAD, {} only in history (about what removing all of it would free, after a purge and gc).",
        a.commits,
        a.blobs_total,
        human_size(a.bytes_total),
        human_size(a.disk_total),
        human_size(a.bytes_in_head),
        human_size(a.bytes_history_only)
    )];
    out.push(format!("Largest blobs (top {}):", a.top));
    for b in &a.blobs {
        let state = match b.state {
            BlobState::InHead => "in HEAD",
            BlobState::Replaced => "old version",
            BlobState::Deleted => "deleted",
        };
        let more = if b.more_paths > 0 {
            format!(" (+{} more path(s))", b.more_paths)
        } else {
            String::new()
        };
        out.push(format!(
            "  {:>10}  {}{}  {}  {}..{}",
            human_size(b.size),
            b.path,
            more,
            state,
            short(&b.first_commit),
            short(&b.last_commit)
        ));
    }
    out.push("Biggest folders:".into());
    for f in &a.folders {
        out.push(format!(
            "  {:>10}  {}/  {} blob(s){}",
            human_size(f.size),
            f.path,
            f.blobs,
            if f.in_head { "" } else { "  deleted" }
        ));
    }
    out.push(format!(
        "Deleted from HEAD, still in history ({} path(s)):",
        a.deleted_paths
    ));
    for d in &a.deleted {
        out.push(format!(
            "  {:>10}  {}  {} version(s)  {}..{}",
            human_size(d.size),
            d.path,
            d.versions,
            short(&d.first_commit),
            short(&d.last_commit)
        ));
    }
    out.join("\n")
}

/// `rewrite script check|test`.
async fn script(session: &mut Session, action: RewriteScriptAction) -> Result<Emitted, String> {
    match action {
        RewriteScriptAction::Trust { file } => {
            let text = read_script(&file)?;
            trust_text(session, &text, &file).await?;
            let hash = script_hash(&text);
            Ok(emitted(
                json!({ "trusted": { "file": file, "hash": hash } }),
                format!(
                    "trusted {file} (sha256 {hash}): it may run in this repository until it changes"
                ),
            ))
        }
        RewriteScriptAction::Check {
            file,
            params,
            language,
            trust,
        } => {
            let script = script_from_file(&file, &params, language.as_deref())?;
            if trust {
                trust_scripts(session, &[(file.clone(), inline_text(&script))]).await?;
            }
            let args = ScriptCheckArgs {
                script,
                vouched: Vec::new(),
            };
            match call_typed::<ScriptCheck>(
                session,
                OperationKind::RewriteScriptCheck,
                to_value(&args)?,
            )
            .await?
            {
                Err(answer) => Ok(answer),
                Ok(check) => {
                    let mut human = if check.ok() {
                        format!(
                            "ok: {} ({}), {} parameter(s)",
                            check.header.name,
                            if check.functions.is_empty() {
                                "external script".to_string()
                            } else {
                                check.functions.join(", ")
                            },
                            check.header.params.len()
                        )
                    } else {
                        format!("problems:\n- {}", check.problems.join("\n- "))
                    };
                    let untrusted = check.trust == ScriptTrust::Untrusted;
                    if untrusted {
                        human.push_str(&format!(
                            "\nnot trusted yet: read it, then `gitpoulpi rewrite script trust {file}`"
                        ));
                    }
                    let mut answer = emitted(to_value(&check)?, human);
                    if !check.ok() || untrusted {
                        answer.exit_code = 2;
                    }
                    Ok(answer)
                }
            }
        }
        RewriteScriptAction::Test {
            file,
            params,
            commits,
            language,
            trust,
        } => {
            let script = script_from_file(&file, &params, language.as_deref())?;
            if trust {
                trust_scripts(session, &[(file.clone(), inline_text(&script))]).await?;
            }
            let mut ids = Vec::new();
            for rev in &commits {
                ids.push(resolve(session, rev).await?);
            }
            let args = ScriptTestArgs {
                script,
                commits: ids,
                vouched: Vec::new(),
            };
            match call_typed::<Vec<ScriptTestRow>>(
                session,
                OperationKind::RewriteScriptTest,
                to_value(&args)?,
            )
            .await?
            {
                Err(answer) => Ok(answer),
                Ok(rows) => {
                    let human = rows
                        .iter()
                        .map(|r| {
                            format!(
                                "{}: {} field(s) change{}",
                                r.commit.to_hex(),
                                r.fields.len(),
                                r.log
                                    .iter()
                                    .map(|l| format!("\n  log: {l}"))
                                    .collect::<String>()
                            )
                        })
                        .collect::<Vec<_>>()
                        .join("\n");
                    Ok(emitted(json!({ "rows": rows }), human))
                }
            }
        }
    }
}

fn read_script(file: &str) -> Result<String, String> {
    std::fs::read_to_string(file).map_err(|e| format!("failed to read the script {file}: {e}"))
}

fn inline_text(script: &ScriptRef) -> String {
    match &script.source {
        ScriptSource::Inline { text } => text.clone(),
        ScriptSource::File { .. } => String::new(),
    }
}

/// Remembers that the user trusts these scripts (their text; `(label,
/// text)`). Only a person: a session that is not the CLI's (the MCP server)
/// refuses.
async fn trust_scripts(session: &mut Session, scripts: &[(String, String)]) -> Result<(), String> {
    for (label, text) in scripts {
        trust_text(session, text, label).await?;
    }
    Ok(())
}

async fn trust_text(session: &mut Session, text: &str, label: &str) -> Result<(), String> {
    if !session.may_trust() {
        return Err(
            "only a person can trust a script: run `gitpoulpi rewrite script trust FILE` yourself"
                .into(),
        );
    }
    let args = TrustArgs {
        hash: script_hash(text),
        label: label.to_string(),
    };
    let _: Value = session
        .call(OperationKind::RewriteTrustScript, to_value(&args)?)
        .await?;
    Ok(())
}

/// A script file as a rule's script, with its `NAME=VALUE` parameters. The
/// language is `language` (`lua`, `shell`, `python`, `lua-external`) or, by
/// default, the file's extension, else the embedded Lua.
fn script_from_file(
    file: &str,
    params: &[String],
    language: Option<&str>,
) -> Result<ScriptRef, String> {
    let text = read_script(file)?;
    let language = match language {
        Some(name) => ScriptLanguage::from_name(name)
            .ok_or_else(|| format!("unknown script language `{name}`"))?,
        None => ScriptLanguage::from_extension(file).unwrap_or(ScriptLanguage::Lua),
    };
    let mut map = std::collections::BTreeMap::new();
    for p in params {
        let (k, v) = p
            .split_once('=')
            .ok_or_else(|| format!("the parameter {p:?} is not NAME=VALUE"))?;
        map.insert(k.to_string(), v.to_string());
    }
    Ok(ScriptRef {
        language,
        source: ScriptSource::Inline { text },
        params: map,
    })
}

/// The push: a dry run first when the caller did not say `--yes`, so the
/// confirmation lists exactly what would go; then the real one. Exit code 4
/// when a ref was rejected or not pushed.
async fn push(
    session: &mut Session,
    mut request: PushRewritten,
    dry_run: bool,
    yes: bool,
) -> Result<Emitted, String> {
    request.dry_run = dry_run || !yes;
    let outcome = match call_typed::<PushRewrittenOutcome>(
        session,
        OperationKind::RewritePush,
        to_value(&request)?,
    )
    .await?
    {
        Err(answer) => return Ok(answer),
        Ok(outcome) => outcome,
    };
    let human = push_human(&outcome);
    if !yes && !dry_run && outcome.count(PushRefStatus::WouldPush) > 0 {
        let prompt = format!(
            "This force-pushes (with leases) {} ref(s). The remote keeps the new history even if \
             you undo the rewrite here, and everyone who has the old one has to recover \
             (`gitpoulpi rewrite note`).",
            outcome.count(PushRefStatus::WouldPush)
        );
        return Ok(Emitted {
            json: json!({ "needs_confirmation": { "prompt": prompt, "would_push": outcome } }),
            human: format!("{prompt}\n{human}\nNothing was sent; repeat with --yes to go ahead."),
            exit_code: 3,
        });
    }
    let exit_code = if outcome.is_clean() { 0 } else { 4 };
    let mut value = to_value(&outcome)?;
    let mut human = human;
    // S23: a host that refused the force push gets the three ways through
    // (never done for the user).
    let refused = rewrite_host::push_blocked(&outcome);
    if let Some((_, remote)) = refused.first() {
        if let Some(host) = rewrite_host::host_for(session, Some(remote), false).await {
            let names: Vec<String> = refused.iter().map(|(n, _)| n.clone()).collect();
            value["protected_by_host"] = rewrite_host::blocked_json(&host, &names);
            human.push_str(&format!(
                "\n\n{}",
                value["protected_by_host"]["text"]
                    .as_str()
                    .unwrap_or_default()
            ));
        }
    }
    Ok(Emitted {
        json: value,
        human,
        exit_code,
    })
}

fn push_human(outcome: &PushRewrittenOutcome) -> String {
    let mut lines: Vec<String> = outcome
        .results
        .iter()
        .map(|r| {
            let what = match r.status {
                PushRefStatus::Pushed => "pushed",
                PushRefStatus::WouldPush => "would push",
                PushRefStatus::UpToDate => "already up to date",
                PushRefStatus::RejectedRemoteMoved => {
                    "rejected: the remote moved since the preview (fetch and review)"
                }
                PushRefStatus::RejectedProtectedByHost => "rejected: protected by the host",
                PushRefStatus::RejectedOther => "rejected",
                PushRefStatus::SkippedProtected => {
                    "skipped: protected (--include-protected to push it)"
                }
                PushRefStatus::NotPushed => "not pushed",
            };
            let detail = r
                .detail
                .as_deref()
                .map(|d| format!(" ({d})"))
                .unwrap_or_default();
            format!("{} {}: {what}{detail}", r.remote, r.name)
        })
        .collect();
    if outcome.results.is_empty() {
        lines.push("no rewritten ref was on a remote: nothing to push".into());
    }
    if outcome.atomic_fallback {
        lines.push("a remote refused --atomic: its refs were pushed one by one".into());
    }
    lines.join("\n")
}

/// `rewrite purge`: the check first (what is deleted and lost, what blocks,
/// the host steps); without `--yes` that is the answer (exit code 3); with it
/// the job runs.
async fn purge(session: &mut Session, args: PurgeArgs, yes: bool) -> Result<Emitted, String> {
    let check =
        match call_typed::<PurgeCheck>(session, OperationKind::RewritePurgeCheck, to_value(&args)?)
            .await?
        {
            Err(answer) => return Ok(answer),
            Ok(check) => check,
        };
    if !check.outcome.blockers.is_empty() {
        let human = format!(
            "Nothing was changed: something still holds the old history.\n{}",
            blockers_human(&check)
        );
        return Ok(Emitted {
            json: to_value(&check)?,
            human,
            exit_code: 4,
        });
    }
    if !yes {
        let human = format!(
            "{}\n{}\nNothing was changed; repeat with --yes to go ahead.",
            check.prompt,
            hosts_human(&check)
        );
        return Ok(Emitted {
            json: json!({ "needs_confirmation": { "prompt": check.prompt, "check": check } }),
            human,
            exit_code: 3,
        });
    }
    // Exactly the rewrites the confirmation named; the daemon adds the other
    // operations whose pins hold the same history.
    let run = PurgeArgs {
        entries: check
            .entries
            .iter()
            .filter(|e| e.rewrite)
            .map(|e| e.id)
            .collect(),
        all: false,
    };
    let outcome =
        match call_typed::<PurgeOutcome>(session, OperationKind::RewritePurge, to_value(&run)?)
            .await?
        {
            Err(answer) => return Ok(answer),
            Ok(outcome) => outcome,
        };
    let (human, exit_code) = match outcome.status {
        PurgeStatus::Purged => (
            format!(
                "purged: {} old object(s) are gone from this clone ({} pin(s) and {} reflog entr{} dropped)\n{}",
                outcome.old_objects,
                outcome.pins_dropped,
                outcome.reflog_entries,
                if outcome.reflog_entries == 1 { "y" } else { "ies" },
                hosts_human(&check)
            ),
            0,
        ),
        PurgeStatus::Incomplete => (
            format!(
                "{} old object(s) are still in this clone (the undo pins and reflogs are gone): \
                 something outside what GitPoulpi looks at keeps them, for example an index or an alternate object store",
                outcome.remaining
            ),
            4,
        ),
        _ => (
            "Nothing was changed: something still holds the old history (run `rewrite purge` again to see what)."
                .to_string(),
            4,
        ),
    };
    Ok(Emitted {
        json: json!({ "outcome": outcome, "hosts": check.hosts }),
        human,
        exit_code,
    })
}

fn blockers_human(check: &PurgeCheck) -> String {
    check
        .outcome
        .blockers
        .iter()
        .map(|b| {
            format!(
                "  {} ({} old commit(s)): {}",
                b.name,
                b.commits,
                b.kind.advice()
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn hosts_human(check: &PurgeCheck) -> String {
    if check.hosts.is_empty() {
        return "This repository has no remote: there is nothing to do on a host.".to_string();
    }
    check
        .hosts
        .iter()
        .map(|h| {
            let steps = h
                .steps
                .iter()
                .enumerate()
                .map(|(i, s)| format!("  {}. {}", i + 1, s.text()))
                .collect::<Vec<_>>()
                .join("\n");
            format!("On {} ({:?}):\n{steps}", h.remote, h.host)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

async fn discard(session: &mut Session, run: &RunId) -> Result<(), String> {
    session
        .call::<Value>(OperationKind::RewriteDiscard, json!({ "run": run }))
        .await
        .map(|_| ())
}

async fn apply(session: &mut Session, args: &ApplyArgs) -> Result<Emitted, String> {
    match call_typed::<RewriteOutcome>(session, OperationKind::RewriteApply, to_value(args)?)
        .await?
    {
        Err(answer) => Ok(answer),
        Ok(outcome) => {
            let human = match &args.target {
                Some(target) if outcome.moved.is_empty() => format!(
                    "extracted into {}: a new repository with only that history; this repository was not touched",
                    target.display()
                ),
                Some(target) => format!(
                    "extracted into {} and added one commit here that replaces the folder by a submodule of it; this history is kept; `gitpoulpi undo` puts the folder back",
                    target.display()
                ),
                None => format!(
                    "rewrote {} commit(s), moved {} ref(s); nothing was pushed (`gitpoulpi rewrite push` does, with leases); `gitpoulpi undo` puts it back",
                    outcome.commits_changed,
                    outcome.moved.len()
                ),
            };
            let mut value = to_value(&outcome)?;
            if let Some(target) = &args.target {
                value["extracted_to"] = json!(target);
            }
            Ok(emitted(value, human))
        }
    }
}

/// An absolute path: the daemon's working directory is not this process's.
fn absolute(path: &str) -> Result<std::path::PathBuf, String> {
    std::path::absolute(path).map_err(|e| format!("cannot resolve {path}: {e}"))
}

/// The folder an extraction into a clone or a submodule needs (and a plain
/// run must not be given).
fn extraction_target(
    recipe: &Recipe,
    target: Option<&str>,
) -> Result<Option<std::path::PathBuf>, String> {
    use gitpoulpi_core::domain::rewrite::ExtractMode;
    match (recipe.extraction(), target) {
        (Some((_, Some(ExtractMode::Clone | ExtractMode::Submodule))), None) => Err(
            "--extract-mode clone and submodule need --target PATH, the folder of the new repository"
                .into(),
        ),
        (Some((_, Some(ExtractMode::Clone | ExtractMode::Submodule))), Some(path)) => {
            absolute(path).map(Some)
        }
        (_, Some(_)) => Err("--target is for --extract-mode clone or submodule".into()),
        _ => Ok(None),
    }
}

async fn preview(
    session: &mut Session,
    recipe: &Recipe,
) -> Result<Result<RewritePlan, Emitted>, String> {
    let args = PreviewArgs {
        recipe: recipe.clone(),
        vouched: Vec::new(),
    };
    call_typed::<RewritePlan>(session, OperationKind::RewritePreview, to_value(&args)?).await
}

/// Calls the daemon; a typed refusal or confirmation request comes back as
/// the answer to print (exit code 2 / 3), any other failure as an error.
async fn call_typed<T: serde::de::DeserializeOwned>(
    session: &mut Session,
    command: OperationKind,
    args: Value,
) -> Result<Result<T, Emitted>, String> {
    match session.call_typed::<T>(command, args).await? {
        Ok(value) => Ok(Ok(value)),
        Err(e) => answer_for(e).map(Err),
    }
}

fn answer_for(e: ResponseError) -> Result<Emitted, String> {
    match e.kind {
        Some(TypedError::RewriteRefused(reason, detail)) => Ok(refused(reason, detail)),
        Some(TypedError::NeedsConfirmation(prompt)) => Ok(Emitted {
            json: json!({ "needs_confirmation": { "prompt": prompt } }),
            human: format!("{prompt}\nNothing was changed; repeat with --yes to go ahead."),
            exit_code: 3,
        }),
        _ => Err(e.message),
    }
}

fn refused(reason: RewriteRefusal, detail: String) -> Emitted {
    if reason == RewriteRefusal::ExtractModeNeeded {
        // Asked each time, like the signatures: exit code 3, nothing ran.
        let prompt = "Choose how to extract the subdirectory with --extract-mode: \
            `clone` (a new local clone in --target with only that history, this repository is not touched), \
            `in-place` (this repository's history is rewritten, undoable) or \
            `submodule` (the clone as above, and one new commit here replaces the folder by a submodule of it).";
        return Emitted {
            json: json!({ "needs_choice": {
                "choice": "extract_mode",
                "options": ["clone", "in-place", "submodule"],
                "prompt": prompt,
            } }),
            human: format!("{prompt}\nNothing was changed."),
            exit_code: 3,
        };
    }
    let message = gitpoulpi_core::Error::RewriteRefused(reason, detail.clone()).to_string();
    let mut refusal = json!({ "reason": reason.code(), "message": message, "detail": detail });
    if reason == RewriteRefusal::ScriptNotTrusted {
        // The detail is the content hash of the script that is not trusted.
        refusal["script"] = json!({ "hash": detail });
    }
    if reason == RewriteRefusal::ScriptFailed {
        // `ScriptFailed { commit, reason }`: the commit the script failed on
        // (absent when it did not compile) and why.
        let (commit, why) = parse_script_failure(&detail);
        refusal["script"] = json!({ "commit": commit.map(|c| c.to_hex()), "reason": why });
    }
    let human = if reason == RewriteRefusal::ScriptNotTrusted {
        format!(
            "{message}\nRead the script, then `gitpoulpi rewrite script trust FILE` (or repeat with --trust)."
        )
    } else {
        message
    };
    Emitted {
        json: json!({ "refused": refusal }),
        human,
        exit_code: 2,
    }
}

fn parse_filter(name: &str) -> RowFilter {
    match name {
        "dropped" => RowFilter::Dropped,
        "pushed" => RowFilter::Pushed,
        "all" => RowFilter::All,
        _ => RowFilter::Changed,
    }
}

/// The first `limit` changed rows of a plan, and how many there are.
async fn fetch_rows(
    session: &mut Session,
    plan: &RewritePlan,
    limit: usize,
) -> Result<(Vec<CommitChange>, usize), String> {
    let mut rows: Vec<CommitChange> = Vec::new();
    let mut total;
    loop {
        let want = limit
            .saturating_sub(rows.len())
            .min(gitpoulpi_ipc::rewrite::MAX_ROWS_PAGE);
        let args = RowsArgs {
            run: plan.run.clone(),
            filter: RowFilter::Changed,
            offset: rows.len(),
            limit: want.max(1),
        };
        let page: RowsPage = session
            .call(OperationKind::RewriteRows, to_value(&args)?)
            .await?;
        total = page.total;
        let got = page.rows.len();
        if want > 0 {
            rows.extend(page.rows);
        }
        if got == 0 || want == 0 || rows.len() >= total || rows.len() >= limit {
            break;
        }
    }
    rows.truncate(limit);
    Ok((rows, total))
}

/// A unified-diff-like text of one file's change.
fn diff_human(change: &BlobChange) -> String {
    let mut out = vec![format!(
        "{} ({:?}, {} -> {} bytes)",
        change.path, change.kind, change.old_size, change.new_size
    )];
    if change.binary {
        out.push("binary content: no line diff".into());
    }
    if change.redacted {
        out.push("removed lines are hidden: they held a secret".into());
    }
    for hunk in &change.hunks {
        out.push(format!("@@ -{} +{} @@", hunk.old_start, hunk.new_start));
        for line in &hunk.lines {
            let mark = match line.kind {
                BlobLineKind::Context => ' ',
                BlobLineKind::Removed => '-',
                BlobLineKind::Added => '+',
            };
            out.push(format!("{mark}{}", line.text));
        }
    }
    out.join("\n")
}

fn plan_human(plan: &RewritePlan) -> String {
    let c = &plan.counts;
    let mut lines = vec![format!(
        "{} of {} commit(s) change, {} branch(es) and {} tag(s) move",
        c.commits_changed, c.commits_walked, c.refs_moved, c.tags_moved
    )];
    for warning in &plan.warnings {
        lines.push(format!(
            "warning: {}",
            serde_json::to_string(warning).unwrap_or_default()
        ));
    }
    lines.join("\n")
}

// ---------------------------------------------------------------- recipe

/// The text of every enabled external script of `recipe`, with a label: the
/// inline text, or the repository file it names.
fn external_scripts(session: &Session, recipe: &Recipe) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for (_, rule) in recipe.enabled_rules() {
        let RuleKind::Script(s) = &rule.kind else {
            continue;
        };
        if !s.language.is_external() {
            continue;
        }
        match &s.source {
            ScriptSource::Inline { text } => {
                out.push((format!("{} script", s.language.name()), text.clone()))
            }
            ScriptSource::File { path } => {
                let rel = std::path::Path::new(path);
                let inside = !rel.is_absolute()
                    && rel
                        .components()
                        .all(|c| matches!(c, std::path::Component::Normal(_)));
                if let (true, Some(repo)) = (inside, session.repo_ref()) {
                    if let Ok(text) = std::fs::read_to_string(repo.root.join(rel)) {
                        out.push((path.clone(), text));
                    }
                }
            }
        }
    }
    out
}

async fn build_recipe(session: &mut Session, args: &RewriteRecipeArgs) -> Result<Recipe, String> {
    let recipe = build_recipe_inner(session, args).await?;
    if args.trust {
        let scripts = external_scripts(session, &recipe);
        trust_scripts(session, &scripts).await?;
    }
    Ok(recipe)
}

async fn build_recipe_inner(
    session: &mut Session,
    args: &RewriteRecipeArgs,
) -> Result<Recipe, String> {
    let mut recipe = read_recipe(args)?;
    let since = match &args.since {
        Some(rev) => Some(resolve(session, rev).await?),
        None => None,
    };
    apply_flags(&mut recipe, args, since)?;
    if let Some(path) = &args.secrets_file {
        let text = std::fs::read_to_string(path)
            .map_err(|e| format!("failed to read the secrets file {path}: {e}"))?;
        add_secrets(&mut recipe, &text)?;
    }
    Ok(recipe)
}

/// A `--recipe FILE|-` argument as a recipe (for `ui propose rewrite`).
pub(crate) fn read_recipe_file(path: &str) -> Result<Recipe, String> {
    read_recipe(&RewriteRecipeArgs {
        recipe: Some(path.to_string()),
        ..RewriteRecipeArgs::default()
    })
}

fn read_recipe(args: &RewriteRecipeArgs) -> Result<Recipe, String> {
    let text = match (&args.recipe, &args.recipe_json) {
        (Some(_), Some(_)) => return Err("give --recipe or --recipe-json, not both".into()),
        (None, None) => return Ok(Recipe::new("cli", Scope::default(), Vec::new())),
        (Some(path), None) if path == "-" => {
            use std::io::Read;
            let mut buf = String::new();
            std::io::stdin()
                .read_to_string(&mut buf)
                .map_err(|e| format!("failed to read the recipe from stdin: {e}"))?;
            buf
        }
        (Some(path), None) => std::fs::read_to_string(path)
            .map_err(|e| format!("failed to read the recipe {path}: {e}"))?,
        (None, Some(json)) => json.clone(),
    };
    parse_recipe(&text)
}

/// JSON when the text starts with `{`, else the TOML the workbench saves.
pub(crate) fn parse_recipe(text: &str) -> Result<Recipe, String> {
    if text.trim_start().starts_with('{') {
        serde_json::from_str(text).map_err(|e| format!("the recipe is not valid JSON: {e}"))
    } else {
        toml::from_str(text).map_err(|e| format!("the recipe is not valid TOML: {e}"))
    }
}

/// The flags override the recipe's scope and options and add their rules
/// after the recipe's own.
fn apply_flags(
    recipe: &mut Recipe,
    args: &RewriteRecipeArgs,
    since: Option<Oid>,
) -> Result<(), String> {
    if args.all_local {
        recipe.scope = Scope::AllLocal;
    } else if !args.refs.is_empty() {
        recipe.scope = Scope::Refs {
            refs: args.refs.clone(),
        };
    } else if since.is_some() {
        recipe.scope = Scope::CurrentBranch { since };
    }
    recipe
        .scope_options
        .include_protected
        .extend(args.include_protected.iter().cloned());

    let identity_flags = args.match_email.is_some()
        || args.match_name.is_some()
        || args.set_name.is_some()
        || args.set_email.is_some()
        || (args.who.is_some() && !args.mailmap);
    let who_flag = match args.who.as_deref() {
        Some("author") => Who::Author,
        Some("committer") => Who::Committer,
        _ => Who::Both,
    };
    if args.mailmap {
        recipe.rules.push(Rule::new(RuleKind::Mailmap {
            apply_to: who_flag,
            file: args.mailmap_file.clone(),
        }));
    }
    if args.map_trailers && !identity_flags {
        return Err("--map-trailers needs an identity rule (--match-email, --set-email...)".into());
    }
    if identity_flags {
        if args.set_name.is_none() && args.set_email.is_none() {
            return Err("an identity rule needs --set-name or --set-email".into());
        }
        recipe
            .rules
            .push(Rule::new(RuleKind::Identity(IdentityRule {
                when: IdentityMatch {
                    name: args.match_name.clone(),
                    email: args.match_email.clone(),
                    regex: false,
                },
                set: IdentitySet {
                    name: args.set_name.clone(),
                    email: args.set_email.clone(),
                },
                apply_to: match args.who.as_deref() {
                    Some("author") => Who::Author,
                    Some("committer") => Who::Committer,
                    _ => Who::Both,
                },
                include_taggers: false,
                map_trailers: args.map_trailers,
            })));
    }
    if let Some(by_seconds) = args.shift_dates {
        recipe.rules.push(Rule::new(RuleKind::DateShift {
            by_seconds,
            apply_to: WhichDate::Both,
        }));
    }
    for spec in &args.replace_message {
        let (find, with) = spec
            .split_once('=')
            .ok_or_else(|| format!("--replace-message {spec:?} is not FIND=WITH"))?;
        recipe
            .rules
            .push(Rule::new(RuleKind::MessageReplace(TextReplace {
                find: TextPattern::literal(find),
                with: with.to_string(),
            })));
    }
    for spec in &args.trailer_add {
        let (key, value) = spec
            .split_once('=')
            .ok_or_else(|| format!("--trailer-add {spec:?} is not KEY=VALUE"))?;
        recipe
            .rules
            .push(Rule::new(RuleKind::Trailer(TrailerRule::add(key, value))));
    }
    for key in &args.trailer_remove {
        recipe
            .rules
            .push(Rule::new(RuleKind::Trailer(TrailerRule::remove(key))));
    }
    for spec in &args.trailer_map {
        let (key, change) = spec
            .split_once(':')
            .ok_or_else(|| format!("--trailer-map {spec:?} is not KEY:OLD=NEW"))?;
        let (old, new) = change
            .split_once('=')
            .ok_or_else(|| format!("--trailer-map {spec:?} is not KEY:OLD=NEW"))?;
        recipe
            .rules
            .push(Rule::new(RuleKind::Trailer(TrailerRule::map(
                key, old, new,
            ))));
    }
    if args.update_message_hashes {
        recipe.options.update_message_hashes = true;
    }
    if let Some(dir) = &args.extract {
        // First among the path rules: a clone or a submodule needs the
        // folder as this repository has it.
        recipe
            .rules
            .push(Rule::new(RuleKind::MovePaths(PathMove::ExtractSubdir {
                dir: dir.clone(),
                mode: match args.extract_mode.as_deref() {
                    Some("clone") => Some(ExtractMode::Clone),
                    Some("in-place") => Some(ExtractMode::InPlace),
                    Some("submodule") => Some(ExtractMode::Submodule),
                    _ => None,
                },
            })));
    }
    for spec in &args.move_paths {
        let (from, to) = spec
            .split_once('=')
            .ok_or_else(|| format!("--move-path {spec:?} is not FROM=TO"))?;
        recipe
            .rules
            .push(Rule::new(RuleKind::MovePaths(PathMove::Rename {
                from: from.to_string(),
                to: to.to_string(),
            })));
    }
    if let Some(dir) = &args.into_subdir {
        recipe
            .rules
            .push(Rule::new(RuleKind::MovePaths(PathMove::IntoSubdir {
                dir: dir.clone(),
            })));
    }
    if !args.remove_paths.is_empty() {
        recipe.rules.push(Rule::new(RuleKind::RemovePaths {
            globs: args.remove_paths.clone(),
        }));
    }
    if let Some(bytes) = args.remove_blobs_over {
        recipe
            .rules
            .push(Rule::new(RuleKind::RemoveBlobsOver { bytes }));
    }
    if !args.lfs_import.is_empty() {
        recipe
            .rules
            .push(Rule::new(RuleKind::LfsImport(LfsImportRule {
                patterns: args.lfs_import.clone(),
                min_size: args.lfs_min_size,
            })));
    }
    if !args.lfs_export.is_empty() {
        recipe
            .rules
            .push(Rule::new(RuleKind::LfsExport(LfsExportRule {
                patterns: args.lfs_export.clone(),
            })));
    }
    push_text_rules(recipe, args)?;
    for spec in &args.replace_content {
        let (find, with) = spec
            .split_once('=')
            .ok_or_else(|| format!("--replace-content {spec:?} is not FIND=WITH"))?;
        recipe
            .rules
            .push(Rule::new(RuleKind::ContentReplace(ContentReplace {
                paths: Vec::new(),
                find: ContentFind::Pattern {
                    pattern: TextPattern::literal(find),
                },
                with: with.to_string(),
                max_blob_bytes: None,
            })));
    }
    if !args.remove_findings.is_empty() {
        // Ids name findings; `all` names every one in the scope. A value
        // typed here by mistake is refused without being echoed.
        let mut ids = Vec::new();
        for id in &args.remove_findings {
            if id == "all" {
                ids.clear();
                break;
            }
            if !gitpoulpi_core::domain::rewrite::secrets::is_finding_id(id) {
                return Err(
                    "--remove-finding takes a finding id (`f-` and 12 hex digits, from `rewrite scan-secrets`) or `all`"
                        .into(),
                );
            }
            ids.push(SecretText(id.clone()));
        }
        recipe
            .rules
            .push(Rule::new(RuleKind::ContentReplace(ContentReplace {
                paths: Vec::new(),
                find: ContentFind::Secrets {
                    literals: ids,
                    scan: true,
                },
                with: String::new(),
                max_blob_bytes: None,
            })));
    }
    for file in &args.scripts {
        let script = script_from_file(file, &args.script_params, args.script_language.as_deref())?;
        recipe.rules.push(Rule::new(RuleKind::Script(script)));
    }
    if args.scripts.is_empty() && !args.script_params.is_empty() {
        return Err("--script-param needs a --script".into());
    }
    match args.prune_empty.as_deref() {
        Some("never") => recipe.options.prune_empty = PruneEmpty::Never,
        Some(_) => recipe.options.prune_empty = PruneEmpty::Auto,
        None => {}
    }
    match args.committer.as_deref() {
        Some("me-now") => recipe.options.committer = CommitterPolicy::MeNow,
        Some(_) => recipe.options.committer = CommitterPolicy::Keep,
        None => {}
    }
    if args.remove_signatures {
        recipe.rules.push(Rule::new(RuleKind::RemoveSignatures));
    }
    if args.fix_name_encoding && !args.fix_message_encoding {
        return Err("--fix-name-encoding needs --fix-message-encoding".into());
    }
    if args.fix_message_encoding {
        recipe.rules.push(Rule::new(RuleKind::MessageEncoding {
            names: args.fix_name_encoding,
        }));
    }
    match args.signatures.as_deref() {
        Some("strip") => recipe.options.signatures = Some(SignaturePolicy::Strip),
        Some("resign-signed") => recipe.options.signatures = Some(SignaturePolicy::ResignSigned),
        Some("resign-all") => recipe.options.signatures = Some(SignaturePolicy::ResignAll),
        _ => {}
    }
    Ok(())
}

/// One secret per line (a trailing `\r` is dropped, blank lines ignored)
/// into every enabled secrets rule. The error never repeats a secret.
fn add_secrets(recipe: &mut Recipe, text: &str) -> Result<(), String> {
    let secrets: Vec<SecretText> = text
        .lines()
        .map(|l| l.trim_end_matches('\r'))
        .filter(|l| !l.is_empty())
        .map(|l| SecretText(l.to_string()))
        .collect();
    if secrets.is_empty() {
        return Err("the secrets file holds no secret (one per line)".into());
    }
    let mut placed = false;
    for rule in recipe.rules.iter_mut().filter(|r| r.enabled) {
        if let RuleKind::ContentReplace(replace) = &mut rule.kind {
            if let ContentFind::Secrets { literals, .. } = &mut replace.find {
                literals.extend(secrets.iter().cloned());
                placed = true;
            }
        }
    }
    if placed {
        Ok(())
    } else {
        Err("--secrets-file needs a content_replace rule that searches for secrets".into())
    }
}

/// S22 and S24bis: the line-ending, exec-bit and encoding rules the flags
/// name.
fn push_text_rules(recipe: &mut Recipe, args: &RewriteRecipeArgs) -> Result<(), String> {
    use gitpoulpi_core::domain::rewrite::{
        EolRule, LineEnding, ReencodeRule, StripExecRule, TextEncoding,
    };
    let encoding = |name: &str| {
        TextEncoding::from_name(name).ok_or_else(|| format!("{name:?} is not a known encoding"))
    };
    if let Some(name) = &args.eol {
        let target =
            LineEnding::from_name(name).ok_or_else(|| format!("{name:?} is not lf, crlf or cr"))?;
        recipe.rules.push(Rule::new(RuleKind::LineEndings(EolRule {
            target,
            paths: args.eol_paths.clone(),
            exclude: args.eol_exclude.clone(),
            follow_attributes: args.eol_follow_attributes,
            max_blob_bytes: None,
        })));
    }
    if !args.strip_exec.is_empty() {
        recipe
            .rules
            .push(Rule::new(RuleKind::StripExec(StripExecRule {
                paths: args.strip_exec.clone(),
                exclude: args.strip_exec_exclude.clone(),
            })));
    }
    if let Some(name) = &args.reencode {
        recipe
            .rules
            .push(Rule::new(RuleKind::Reencode(ReencodeRule {
                target: encoding(name)?,
                from: args.reencode_from.as_deref().map(encoding).transpose()?,
                fallback: match &args.encoding_fallback {
                    Some(f) => encoding(f)?,
                    None => TextEncoding::Windows1252,
                },
                paths: args.encoding_paths.clone(),
                exclude: args.encoding_exclude.clone(),
                max_blob_bytes: None,
            })));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitpoulpi_core::domain::rewrite::{validate_recipe, ContentReplace};

    fn secrets_rule() -> Rule {
        Rule::new(RuleKind::ContentReplace(ContentReplace {
            paths: Vec::new(),
            find: ContentFind::Secrets {
                literals: Vec::new(),
                scan: false,
            },
            with: "***".into(),
            max_blob_bytes: None,
        }))
    }

    fn literals(recipe: &Recipe) -> Vec<String> {
        recipe
            .rules
            .iter()
            .filter_map(|r| match &r.kind {
                RuleKind::ContentReplace(ContentReplace {
                    find: ContentFind::Secrets { literals, .. },
                    ..
                }) => Some(literals.iter().map(|s| s.0.clone())),
                _ => None,
            })
            .flatten()
            .collect()
    }

    #[test]
    fn a_secrets_file_fills_the_secrets_rules_and_only_those() {
        let mut recipe = Recipe::new("t", Scope::default(), vec![secrets_rule()]);
        add_secrets(&mut recipe, "alpha\r\n\nbeta gamma\n").unwrap();
        assert_eq!(literals(&recipe), ["alpha", "beta gamma"]);
        assert!(recipe.holds_secrets());
        // What may be saved has none.
        assert!(!recipe.saved().holds_secrets());
    }

    #[test]
    fn a_secrets_file_without_a_rule_or_without_secrets_is_an_error_that_hides_them() {
        let mut bare = Recipe::new("t", Scope::default(), Vec::new());
        let err = add_secrets(&mut bare, "hunter2\n").unwrap_err();
        assert!(!err.contains("hunter2"));
        let mut recipe = Recipe::new("t", Scope::default(), vec![secrets_rule()]);
        assert!(add_secrets(&mut recipe, "\n\r\n").is_err());
    }

    #[test]
    fn flags_become_rules_and_override_the_scope() {
        let args = RewriteRecipeArgs {
            match_email: Some("old@example.com".into()),
            set_name: Some("Casey Example".into()),
            who: Some("author".into()),
            shift_dates: Some(-3600),
            replace_message: vec!["foo=bar".into()],
            all_local: true,
            signatures: Some("strip".into()),
            ..RewriteRecipeArgs::default()
        };
        let mut recipe = Recipe::new("t", Scope::default(), Vec::new());
        apply_flags(&mut recipe, &args, None).unwrap();
        assert_eq!(recipe.scope, Scope::AllLocal);
        assert_eq!(recipe.rules.len(), 3);
        assert_eq!(recipe.options.signatures, Some(SignaturePolicy::Strip));
        validate_recipe(&recipe).unwrap();
    }

    #[test]
    fn trailer_flags_become_rules_and_the_hash_option() {
        let args = RewriteRecipeArgs {
            trailer_add: vec!["Signed-off-by=Sam Roe <sam@example.com>".into()],
            trailer_remove: vec!["Change-Id".into()],
            trailer_map: vec!["Co-authored-by:@old.org=@new.org".into()],
            update_message_hashes: true,
            ..RewriteRecipeArgs::default()
        };
        let mut recipe = Recipe::new("t", Scope::default(), Vec::new());
        apply_flags(&mut recipe, &args, None).unwrap();
        assert_eq!(
            recipe
                .rules
                .iter()
                .map(|r| r.kind.clone())
                .collect::<Vec<_>>(),
            vec![
                RuleKind::Trailer(TrailerRule::add(
                    "Signed-off-by",
                    "Sam Roe <sam@example.com>"
                )),
                RuleKind::Trailer(TrailerRule::remove("Change-Id")),
                RuleKind::Trailer(TrailerRule::map("Co-authored-by", "@old.org", "@new.org")),
            ]
        );
        assert!(recipe.options.update_message_hashes);
        validate_recipe(&recipe).unwrap();
        // Off unless asked.
        let mut plain = Recipe::new("t", Scope::default(), Vec::new());
        apply_flags(&mut plain, &RewriteRecipeArgs::default(), None).unwrap();
        assert!(!plain.options.update_message_hashes);
    }

    #[test]
    fn bad_trailer_flags_are_errors() {
        let mut recipe = Recipe::new("t", Scope::default(), Vec::new());
        for args in [
            RewriteRecipeArgs {
                trailer_add: vec!["no equals".into()],
                ..RewriteRecipeArgs::default()
            },
            RewriteRecipeArgs {
                trailer_map: vec!["Key-without-colon=x".into()],
                ..RewriteRecipeArgs::default()
            },
            RewriteRecipeArgs {
                trailer_map: vec!["Key:no-equals".into()],
                ..RewriteRecipeArgs::default()
            },
            RewriteRecipeArgs {
                map_trailers: true,
                ..RewriteRecipeArgs::default()
            },
        ] {
            assert!(apply_flags(&mut recipe, &args, None).is_err(), "{args:?}");
        }
    }

    #[test]
    fn bad_flag_combinations_are_errors() {
        let mut recipe = Recipe::new("t", Scope::default(), Vec::new());
        let no_set = RewriteRecipeArgs {
            match_email: Some("a@example.com".into()),
            ..RewriteRecipeArgs::default()
        };
        assert!(apply_flags(&mut recipe, &no_set, None).is_err());
        let bad = RewriteRecipeArgs {
            replace_message: vec!["no equals sign".into()],
            ..RewriteRecipeArgs::default()
        };
        assert!(apply_flags(&mut recipe, &bad, None).is_err());
    }

    #[test]
    fn recipes_parse_from_json_and_toml() {
        let recipe = Recipe::new("t", Scope::AllLocal, vec![secrets_rule()]);
        let json = serde_json::to_string(&recipe).unwrap();
        assert_eq!(parse_recipe(&json).unwrap(), recipe);
        let text = toml::to_string(&recipe).unwrap();
        assert_eq!(parse_recipe(&text).unwrap(), recipe);
        assert!(parse_recipe("{ nope").is_err());
    }
}
