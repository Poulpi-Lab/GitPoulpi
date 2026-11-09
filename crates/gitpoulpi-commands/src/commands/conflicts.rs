//! `gitpoulpi conflicts` handlers: list, show, resolve, unresolve, state
//! and abort for the files a stopped merge, rebase, cherry-pick or revert
//! left unmerged.
//!
//! `--both` and `--pick` read the file's `MergeDocument` and compose the
//! text with the same `compose_resolution` the merge editor uses, then
//! write it back with the line terminator, BOM and encoding the document
//! records, guarded by the stage oids it was read at (`stale` when
//! another frontend changed the file in between).

use std::collections::BTreeMap;

use gitpoulpi_core::domain::{
    compose_resolution, AmOutcome, BlobContent, ConflictEntry, ConflictPicks, ConflictResolution,
    ConflictSet, ConflictSide, Eol, IndexLock, MergeDocument, MergeRegion, Oid, OperationKind,
    OperationState, ResolveOptions, SideBlob,
};
use gitpoulpi_core::TypedError;
use serde_json::{json, Value};

use crate::cli::{ConflictSideArg, ConflictsAction, IndexLockAction};
use crate::client::Session;

use super::*;

pub(super) async fn conflicts(
    session: &mut Session,
    action: ConflictsAction,
) -> Result<Emitted, String> {
    match action {
        ConflictsAction::List { names_only: true } => {
            let entries: Vec<ConflictEntry> = session
                .call(OperationKind::ConflictEntries, json!({}))
                .await?;
            let human = list_human(entries.iter().map(|e| {
                format!(
                    "{}  ({})",
                    e.path.display(),
                    kind_name(&to_value(&e.kind).unwrap_or_default())
                )
            }));
            Ok(emitted(to_value(&entries)?, human))
        }
        ConflictsAction::List { names_only: false } => {
            let sets: Vec<ConflictSet> = session.call(OperationKind::Conflicts, json!({})).await?;
            let human = list_human(sets.iter().map(|c| c.path.display().to_string()));
            Ok(emitted(to_value(&sets)?, human))
        }
        ConflictsAction::Check => {
            let prediction: gitpoulpi_core::domain::conflict_prevention::ConflictPrediction =
                session
                    .call(OperationKind::PredictConflicts, json!({}))
                    .await?;
            let human = match (&prediction.base_ref, prediction.files.is_empty()) {
                (Some(base), false) => format!(
                    "conflicts with {base} in {} file(s):\n{}",
                    prediction.files.len(),
                    prediction.files.join("\n")
                ),
                (Some(base), true) => format!("no conflicts with {base}"),
                (None, _) => format!("{:?}", prediction.status).to_lowercase(),
            };
            Ok(emitted(to_value(&prediction)?, human))
        }
        ConflictsAction::Show { path, conflict } => show(session, &path, conflict).await,
        ConflictsAction::Resolve {
            path,
            side,
            ours,
            theirs,
            both,
            theirs_first,
            pick,
            pick_default,
            content_file,
            encoding,
            delete,
            no_stage,
            expect,
        } => {
            let explicit = match expect {
                Some(e) => Some(parse_expect(&e)?),
                None => None,
            };
            let how = if let Some(side) = side {
                How::Side(match side {
                    ConflictSideArg::Ours => ConflictSide::Ours,
                    ConflictSideArg::Theirs => ConflictSide::Theirs,
                    ConflictSideArg::Base => ConflictSide::Base,
                })
            } else if ours {
                How::Side(ConflictSide::Ours)
            } else if theirs {
                How::Side(ConflictSide::Theirs)
            } else if both {
                How::Both { theirs_first }
            } else if let Some(spec) = pick {
                How::Pick {
                    spec,
                    default: pick_default,
                }
            } else if let Some(file) = content_file {
                How::Content { file, encoding }
            } else if delete {
                How::Delete
            } else {
                return Err("conflicts resolve: say how (--side, --both, --pick, --content-file or --delete)".into());
            };
            resolve(session, &path, how, !no_stage, explicit).await
        }
        ConflictsAction::Unresolve { path } => {
            let answer = session
                .call_typed::<()>(OperationKind::ConflictUnresolve, json!({ "path": path }))
                .await?;
            match answer {
                Ok(()) => Ok(emitted(
                    json!({ "unresolved": path }),
                    format!("{path} is conflicted again"),
                )),
                Err(e) => refusal_or_err(e),
            }
        }
        ConflictsAction::State => {
            let state: Option<OperationState> = session
                .call(OperationKind::OperationState, json!({}))
                .await?;
            let human = match &state {
                Some(s) => format!("{} in progress", s.name()),
                None => "no merge, rebase, cherry-pick or revert in progress".to_string(),
            };
            let json = match &state {
                Some(s) => json!({ "operation": s.name(), "state": s }),
                None => Value::Null,
            };
            Ok(emitted(json, human))
        }
        ConflictsAction::AmContinue => am_step(session, OperationKind::AmContinue).await,
        ConflictsAction::AmSkip => am_step(session, OperationKind::AmSkip).await,
        ConflictsAction::AmAbort => {
            let answer = session
                .call_typed::<()>(OperationKind::AmAbort, json!({}))
                .await?;
            match answer {
                Ok(()) => Ok(emitted(
                    json!({ "aborted": "am" }),
                    "am aborted".to_string(),
                )),
                Err(e) => refusal_or_err(e),
            }
        }
        ConflictsAction::Skip => {
            let state: Option<OperationState> = session
                .call(OperationKind::OperationState, json!({}))
                .await?;
            let answer = session
                .call_typed::<()>(OperationKind::OperationSkip, json!({}))
                .await?;
            match answer {
                Ok(()) => {
                    let name = state.as_ref().map_or("operation", OperationState::name);
                    Ok(emitted(
                        json!({ "skipped": name }),
                        format!("{name} skipped"),
                    ))
                }
                Err(e) => refusal_or_err(e),
            }
        }
        ConflictsAction::Abort => {
            let state: Option<OperationState> = session
                .call(OperationKind::OperationState, json!({}))
                .await?;
            // An `am` is not journaled: its abort is its own operation.
            let kind = if matches!(state, Some(OperationState::Am { .. })) {
                OperationKind::AmAbort
            } else {
                OperationKind::OperationAbort
            };
            let answer = session.call_typed::<()>(kind, json!({})).await?;
            match answer {
                Ok(()) => {
                    let name = state.as_ref().map_or("operation", OperationState::name);
                    Ok(emitted(
                        json!({ "aborted": name }),
                        format!("{name} aborted"),
                    ))
                }
                Err(e) => refusal_or_err(e),
            }
        }
    }
}

fn list_human(lines: impl Iterator<Item = String>) -> String {
    let lines: Vec<String> = lines.collect();
    if lines.is_empty() {
        "no conflicts".to_string()
    } else {
        lines.join("\n")
    }
}

fn kind_name(kind: &Value) -> String {
    kind.as_str().unwrap_or("conflicted").to_string()
}

/// A typed conflict refusal becomes a normal answer with exit code 2
/// (`{"refused": {"reason", "message"}}`), like a refused undo; any other
/// error stays an error.
fn refusal_or_err(e: gitpoulpi_ipc::ResponseError) -> Result<Emitted, String> {
    match e.kind {
        Some(TypedError::ConflictRefused(reason)) => {
            Ok(refused(to_value(&reason)?, reason.message()))
        }
        _ => Err(e.message),
    }
}

fn refused(reason: Value, message: String) -> Emitted {
    Emitted {
        json: json!({ "refused": { "reason": reason, "message": message } }),
        human: message,
        exit_code: 2,
    }
}

enum How {
    Side(ConflictSide),
    Both {
        theirs_first: bool,
    },
    Pick {
        spec: String,
        default: Option<String>,
    },
    Content {
        file: String,
        encoding: Option<String>,
    },
    Delete,
}

type Expected = [Option<Oid>; 3];

/// `base,ours,theirs`; `-` (or empty) stands for a missing stage.
fn parse_expect(raw: &str) -> Result<Expected, String> {
    let parts: Vec<&str> = raw.split(',').map(str::trim).collect();
    if parts.len() != 3 {
        return Err(format!(
            "--expect takes three comma-separated oids (base,ours,theirs; - for none), got {raw:?}"
        ));
    }
    let mut out: Expected = [None; 3];
    for (slot, part) in out.iter_mut().zip(parts) {
        if part.is_empty() || part == "-" {
            continue;
        }
        *slot = Some(
            Oid::from_hex(part)
                .ok_or_else(|| format!("--expect: {part:?} is not a 40-character object id"))?,
        );
    }
    Ok(out)
}

async fn read_document(session: &mut Session, path: &str) -> Result<MergeDocument, String> {
    session
        .call(OperationKind::ConflictShow, json!({ "path": path }))
        .await
}

async fn resolve(
    session: &mut Session,
    path: &str,
    how: How,
    stage: bool,
    explicit: Option<Expected>,
) -> Result<Emitted, String> {
    let mut expected = explicit;
    let resolution = match how {
        How::Side(side) => ConflictResolution::TakeSide(side),
        How::Delete => ConflictResolution::Delete,
        How::Content { file, encoding } => {
            let text = read_text(&file)?;
            match encoding {
                Some(encoding) => ConflictResolution::UseEncoded { text, encoding },
                None => ConflictResolution::UseContent(text),
            }
        }
        How::Both { theirs_first } => {
            let doc = read_document(session, path).await?;
            let sides = if theirs_first {
                [ConflictSide::Theirs, ConflictSide::Ours]
            } else {
                [ConflictSide::Ours, ConflictSide::Theirs]
            };
            let picks: Vec<ConflictPicks> = doc
                .conflicts()
                .map(|region| ConflictPicks::take(region, &sides))
                .collect();
            expected.get_or_insert(ResolveOptions::oids_of(&doc.entry));
            composed(&doc, &picks)?
        }
        How::Pick { spec, default } => {
            let doc = read_document(session, path).await?;
            if let Some(e) = no_text_regions(&doc, path) {
                return Ok(e);
            }
            let picks = match picks_from_spec(&doc, &spec, default.as_deref()) {
                Ok(picks) => picks,
                Err(PickProblem::Bad(message)) => return Err(message),
                Err(PickProblem::Missing(missing)) => {
                    let message = format!(
                        "--pick needs a choice for every conflict; missing {} (or pass --default a|b)",
                        missing
                            .iter()
                            .map(usize::to_string)
                            .collect::<Vec<_>>()
                            .join(", ")
                    );
                    return Ok(refused(
                        json!({ "pick_incomplete": { "missing": missing } }),
                        message,
                    ));
                }
            };
            expected.get_or_insert(ResolveOptions::oids_of(&doc.entry));
            composed(&doc, &picks)?
        }
    };
    let opts = ResolveOptions { stage, expected };
    let answer = session
        .call_typed::<()>(
            OperationKind::ResolveConflict,
            json!({ "path": path, "resolution": resolution, "opts": opts }),
        )
        .await?;
    match answer {
        Ok(()) => {
            let human = if stage {
                format!("resolved {path}")
            } else {
                format!("saved {path} (still conflicted)")
            };
            Ok(emitted(
                json!({ "resolved": path, "staged": stage, "resolution": resolution_summary(&resolution) }),
                human,
            ))
        }
        Err(e) => refusal_or_err(e),
    }
}

/// What was written, without echoing a whole file back.
fn resolution_summary(resolution: &ConflictResolution) -> Value {
    match resolution {
        ConflictResolution::TakeSide(side) => json!({ "take_side": side }),
        ConflictResolution::UseContent(text) => json!({ "content": { "bytes": text.len() } }),
        ConflictResolution::UseEncoded { text, encoding } => {
            json!({ "content": { "bytes": text.len(), "encoding": encoding } })
        }
        ConflictResolution::Delete => json!("delete"),
    }
}

fn no_text_regions(doc: &MergeDocument, path: &str) -> Option<Emitted> {
    doc.regions.is_empty().then(|| {
        Emitted {
            json: json!({ "refused": {
                "reason": "not_a_text_conflict",
                "message": format!(
                    "{path} has no text conflict regions (binary, deleted on one side, symlink or submodule): use --side, --content-file or --delete"
                ),
            } }),
            human: format!(
                "{path} has no text conflict regions: use --side, --content-file or --delete"
            ),
            exit_code: 2,
        }
    })
}

/// The file text for `picks`, with the line terminator, BOM and encoding
/// of the file restored. `Err` for a document with no text regions.
fn composed(doc: &MergeDocument, picks: &[ConflictPicks]) -> Result<ConflictResolution, String> {
    if doc.regions.is_empty() {
        return Err(format!(
            "{} has no text conflict regions (binary, deleted on one side, symlink or submodule): use --side, --content-file or --delete",
            doc.entry.path.display()
        ));
    }
    let mut text = compose_resolution(doc, picks);
    if doc.eol == Eol::Crlf {
        text = text.replace('\n', "\r\n");
    }
    if doc.bom {
        text.insert(0, '\u{feff}');
    }
    Ok(match &doc.encoding {
        Some(label) => ConflictResolution::UseEncoded {
            text,
            encoding: label.clone(),
        },
        None => ConflictResolution::UseContent(text),
    })
}

pub(crate) fn read_text_file(file: &str) -> Result<String, String> {
    read_text(file)
}

fn read_text(file: &str) -> Result<String, String> {
    if file == "-" {
        use std::io::Read;
        let mut buf = String::new();
        std::io::stdin()
            .read_to_string(&mut buf)
            .map_err(|e| format!("failed to read the content from stdin: {e}"))?;
        Ok(buf)
    } else {
        let bytes = std::fs::read(file).map_err(|e| format!("failed to read {file}: {e}"))?;
        String::from_utf8(bytes)
            .map_err(|_| format!("{file} is not UTF-8 text (use --encoding for another encoding)"))
    }
}

/// Why `--pick` could not be turned into picks.
pub(crate) enum PickProblem {
    /// A malformed spec or an unknown conflict number.
    Bad(String),
    /// These conflict numbers (1-based) have no choice and no default.
    Missing(Vec<usize>),
}

/// `--pick 1=a,2=ba [--default a]` as one [`ConflictPicks`] per conflict of
/// `doc`: what `conflicts resolve --pick` and `ui propose resolution --pick`
/// both compose from.
pub(crate) fn picks_from_spec(
    doc: &MergeDocument,
    spec: &str,
    default: Option<&str>,
) -> Result<Vec<ConflictPicks>, PickProblem> {
    let count = doc.conflict_count();
    let chosen = parse_picks(spec, count).map_err(PickProblem::Bad)?;
    let fallback = match default {
        Some(d) => Some(choice_sides(d).map_err(PickProblem::Bad)?),
        None => None,
    };
    let missing: Vec<usize> = (1..=count)
        .filter(|n| !chosen.contains_key(n) && fallback.is_none())
        .collect();
    if !missing.is_empty() {
        return Err(PickProblem::Missing(missing));
    }
    Ok(doc
        .conflicts()
        .enumerate()
        .map(|(i, region)| {
            let sides = chosen
                .get(&(i + 1))
                .or(fallback.as_ref())
                .cloned()
                .unwrap_or_default();
            ConflictPicks::take(region, &sides)
        })
        .collect())
}

/// The document of `path`, for the layers that compose a proposal.
pub(crate) async fn document_of(
    session: &mut Session,
    path: &str,
) -> Result<MergeDocument, String> {
    read_document(session, path).await
}

/// `a`, `b`, `ab`, `ba`, `none` as the sides they take, in order.
fn choice_sides(choice: &str) -> Result<Vec<ConflictSide>, String> {
    use ConflictSide::{Ours, Theirs};
    match choice.trim().to_ascii_lowercase().as_str() {
        "a" => Ok(vec![Ours]),
        "b" => Ok(vec![Theirs]),
        "ab" => Ok(vec![Ours, Theirs]),
        "ba" => Ok(vec![Theirs, Ours]),
        "none" | "" => Ok(vec![]),
        other => Err(format!("{other:?} is not a choice (a, b, ab, ba or none)")),
    }
}

/// `1=a,2=ba` into conflict number (1-based) to sides.
fn parse_picks(spec: &str, count: usize) -> Result<BTreeMap<usize, Vec<ConflictSide>>, String> {
    let mut out = BTreeMap::new();
    for item in spec.split(',').filter(|s| !s.trim().is_empty()) {
        let (n, choice) = item
            .split_once('=')
            .ok_or_else(|| format!("--pick item {item:?} is not N=CHOICE"))?;
        let n: usize = n
            .trim()
            .parse()
            .map_err(|_| format!("--pick: {n:?} is not a conflict number"))?;
        if n == 0 || n > count {
            return Err(format!(
                "--pick: there is no conflict {n} (this file has {count})"
            ));
        }
        if out.insert(n, choice_sides(choice)?).is_some() {
            return Err(format!("--pick: conflict {n} is given twice"));
        }
    }
    Ok(out)
}

fn side_json(blob: &Option<SideBlob>) -> Value {
    match blob {
        None => Value::Null,
        Some(b) => json!({
            "oid": b.oid,
            "mode": format!("{:o}", b.mode),
            "size": b.size,
            "content_kind": match &b.content {
                BlobContent::Text { .. } => "text",
                BlobContent::Binary => "binary",
                BlobContent::TooLarge => "too_large",
                _ => "lfs_pointer",
            },
        }),
    }
}

async fn show(session: &mut Session, path: &str, only: Option<usize>) -> Result<Emitted, String> {
    let doc = read_document(session, path).await?;
    let total = doc.conflict_count();
    if let Some(n) = only {
        if n == 0 || n > total {
            return Err(format!(
                "there is no conflict {n} in {path} (it has {total})"
            ));
        }
    }
    let regions: Vec<(usize, &MergeRegion)> = doc
        .conflicts()
        .enumerate()
        .map(|(i, r)| (i + 1, r))
        .filter(|(i, _)| only.is_none_or(|n| n == *i))
        .collect();
    let conflicts: Vec<Value> = regions
        .iter()
        .map(|(index, r)| {
            json!({
                "index": index,
                "base": r.lines_of(ConflictSide::Base),
                "ours": r.lines_of(ConflictSide::Ours),
                "theirs": r.lines_of(ConflictSide::Theirs),
            })
        })
        .collect();
    let json = json!({
        "path": doc.entry.path,
        "kind": doc.entry.kind,
        "operation": doc.labels.op,
        "description": doc.labels.description,
        "labels": {
            "ours": doc.labels.current,
            "theirs": doc.labels.incoming,
            "base": doc.labels.base,
        },
        "sides": {
            "base": side_json(&doc.base),
            "ours": side_json(&doc.ours),
            "theirs": side_json(&doc.theirs),
        },
        "conflict_count": total,
        "conflicts": conflicts,
        "worktree": doc.worktree,
        "eol": doc.eol,
        "bom": doc.bom,
        "encoding": doc.encoding,
    });
    let mut human = format!(
        "{}  ({})\n",
        doc.entry.path.display(),
        doc.labels.description
    );
    human.push_str(&format!(
        "A = {}\nB = {}\n",
        doc.labels.current, doc.labels.incoming
    ));
    if total == 0 {
        human.push_str(
            "\nno text conflict regions: resolve with --side, --content-file or --delete\n",
        );
    }
    for (index, r) in &regions {
        human.push_str(&format!("\n--- conflict {index} of {total} ---\nA:\n"));
        for l in r.lines_of(ConflictSide::Ours) {
            human.push_str(&format!("  {l}\n"));
        }
        human.push_str("B:\n");
        for l in r.lines_of(ConflictSide::Theirs) {
            human.push_str(&format!("  {l}\n"));
        }
    }
    Ok(emitted(json, human.trim_end().to_string()))
}

async fn am_step(session: &mut Session, kind: OperationKind) -> Result<Emitted, String> {
    let answer = session.call_typed::<AmOutcome>(kind, json!({})).await?;
    match answer {
        Ok(outcome) => {
            let human = match outcome {
                AmOutcome::Completed => "am finished",
                AmOutcome::Stopped => "am stopped on the next patch",
            };
            Ok(emitted(json!({ "outcome": outcome }), human.to_string()))
        }
        Err(e) => refusal_or_err(e),
    }
}

/// `gitpoulpi index-lock show|remove`.
pub(super) async fn index_lock(
    session: &mut Session,
    action: IndexLockAction,
) -> Result<Emitted, String> {
    match action {
        IndexLockAction::Show => {
            let lock: Option<IndexLock> = session.call(OperationKind::IndexLock, json!({})).await?;
            let human = match &lock {
                Some(l) if l.stale => format!("stale {} ({}s old)", l.path.display(), l.age_secs),
                Some(l) => format!(
                    "{} ({}s old, may belong to a running git)",
                    l.path.display(),
                    l.age_secs
                ),
                None => "no index.lock".to_string(),
            };
            Ok(emitted(to_value(&lock)?, human))
        }
        IndexLockAction::Remove => {
            session
                .call::<()>(OperationKind::IndexLockRemove, json!({}))
                .await?;
            Ok(emitted(
                json!({ "removed": true }),
                "index.lock removed".to_string(),
            ))
        }
    }
}
