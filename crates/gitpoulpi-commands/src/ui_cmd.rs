//! `gitpoulpi ui ...` and the MCP `ui_*` tools: showing things to the user in
//! their open GitPoulpi window. see the mcp-show-ui design.
//!
//! The commands build a [`UiIntent`] (view-only by construction, see
//! `gitpoulpi_ipc::ui`), check what they cheaply can **before** routing so a
//! bad revision or path never reaches the screen, and send it to the
//! session's daemon, which hands it to one attached TUI window.
//!
//! Output is the [`UiOutcome`] as JSON (`{"outcome": "applied", "state": ..}`).
//! Exit codes: 0 applied/partial, 3 deferred (the user decides), 1 for
//! refused, not found and no window attached.

use gitpoulpi_core::domain::{DiffFileList, DiffTarget, Oid, OperationKind, RepoId};
use gitpoulpi_ipc::ui::{
    sanitize_display, sanitize_message, sanitize_with_limit, DiffAt, DiffLayout, DiffSide, FileAt,
    FileMode, HostingItem, LineRange, PaneId, Proposal, SettingsPage, UiIntent, UiOutcome,
    UiQueryArgs, UiShowArgs, UiState, ViewerId, ViewerInfo, NO_VIEWER_MESSAGE,
};
use std::path::Path;
use std::time::{Duration, Instant};

use gitpoulpi_core::domain::UiControl;
use serde_json::{json, Value};

use crate::cli::{UiAction, UiProposeAction, UiShowAction};
use crate::client::{discover_repo, Session};
use crate::repo_cmd::Emitted;
use crate::ui_launch::{self, pick, scan_sessions, Pick};

/// An intent that passed the checks, or the outcome to answer instead.
type Built = Result<UiIntent, UiOutcome>;

pub(crate) async fn run(
    ipc: &mut Session,
    viewer: Option<String>,
    launch: bool,
    action: UiAction,
) -> Result<Emitted, String> {
    let viewer = viewer.map(ViewerId);
    match action {
        UiAction::Viewers { all_sessions } => viewers(ipc, all_sessions).await,
        UiAction::State { text } => state(ipc, viewer, text).await,
        UiAction::Show { what } => {
            let built = build_show(ipc, what).await?;
            send(ipc, viewer, launch, built).await
        }
        UiAction::Highlight { range, no_scroll } => {
            let built = parse_range(&range).map(|range| UiIntent::Highlight {
                range,
                scroll: !no_scroll,
            });
            send(ipc, viewer, launch, built.map_err(not_found)).await
        }
        UiAction::Notify { message } => {
            let message = sanitize_display(&message);
            let built = if message.trim().is_empty() {
                Err(not_found("the message is empty".to_string()))
            } else {
                Ok(UiIntent::Notify { message })
            };
            send(ipc, viewer, launch, built).await
        }
        UiAction::Back => send(ipc, viewer, launch, Ok(UiIntent::Back)).await,
        UiAction::Propose { what } => {
            let id = proposal_id();
            let built = build_propose(ipc, &id, what).await?;
            let mut emitted = send(ipc, viewer, launch, built).await?;
            // Every answer names the proposal, `Deferred` included: the
            // agent follows it in `ui state`'s `proposals`.
            if let Value::Object(map) = &mut emitted.json {
                map.entry("proposal").or_insert_with(|| json!(id));
            }
            emitted.human = format!("{} [proposal {id}]", emitted.human);
            Ok(emitted)
        }
    }
}

fn not_found(what: String) -> UiOutcome {
    UiOutcome::NotFound { what }
}

async fn viewers(ipc: &mut Session, all_sessions: bool) -> Result<Emitted, String> {
    let list: Vec<ViewerInfo> = ipc
        .call_without_repo(
            OperationKind::UiQuery,
            serde_json::to_value(UiQueryArgs::Viewers).map_err(|e| e.to_string())?,
        )
        .await?;
    let mut rows: Vec<(String, ViewerInfo)> = list
        .into_iter()
        .map(|v| (ipc.session_name().to_string(), v))
        .collect();
    if all_sessions {
        for found in scan_sessions(Some(ipc.session_name())).await {
            rows.extend(
                found
                    .viewers
                    .into_iter()
                    .map(|v| (found.session.clone(), v)),
            );
        }
    }
    let human = if rows.is_empty() {
        if all_sessions {
            "no GitPoulpi window is attached to any running session".to_string()
        } else {
            "no GitPoulpi window is attached to this session".to_string()
        }
    } else {
        rows.iter()
            .map(|(session, v)| {
                format!(
                    "{}{}  pid {}  repo {}  {}{}",
                    if all_sessions {
                        format!("[{session}] ")
                    } else {
                        String::new()
                    },
                    v.id,
                    v.pid,
                    v.active_repo.as_ref().map_or("-", |r| r.0.as_str()),
                    match v.focused {
                        Some(true) => "focused",
                        Some(false) => "unfocused",
                        None => "focus unknown",
                    },
                    if v.agent_control {
                        ""
                    } else {
                        "  (agent control off)"
                    }
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    let json = if all_sessions {
        // The same objects, each with the session it belongs to.
        Value::Array(
            rows.iter()
                .map(|(session, v)| {
                    let mut value = serde_json::to_value(v).unwrap_or(Value::Null);
                    if let Value::Object(map) = &mut value {
                        map.insert("session".into(), json!(session));
                    }
                    value
                })
                .collect(),
        )
    } else {
        serde_json::to_value(rows.iter().map(|(_, v)| v).collect::<Vec<_>>())
            .map_err(|e| e.to_string())?
    };
    Ok(Emitted {
        json,
        human,
        exit_code: 0,
    })
}

async fn state(
    ipc: &mut Session,
    viewer: Option<ViewerId>,
    include_text: bool,
) -> Result<Emitted, String> {
    let args = serde_json::to_value(UiQueryArgs::State {
        viewer,
        include_text,
    })
    .map_err(|e| e.to_string())?;
    match ipc
        .call_without_repo::<UiState>(OperationKind::UiQuery, args)
        .await
    {
        Ok(state) => Ok(Emitted {
            human: describe_state(&state),
            json: serde_json::to_value(&state).map_err(|e| e.to_string())?,
            exit_code: 0,
        }),
        // "No window attached" is an answer, like for `ui show`.
        Err(message) if message.contains("No GitPoulpi window") => {
            Ok(outcome_emitted(&no_viewer(ipc)))
        }
        Err(message) => Err(message),
    }
}

fn describe_state(state: &UiState) -> String {
    let mut parts = Vec::new();
    if let Some(repo) = &state.repo {
        parts.push(format!("repo {repo}"));
    }
    if let Some(central) = &state.central {
        parts.push(format!("showing {central}"));
    }
    if let Some(focus) = &state.focus {
        parts.push(format!("focus {focus}"));
    }
    if let Some(overlay) = &state.overlay {
        parts.push(format!("overlay {overlay}"));
    }
    if parts.is_empty() {
        "window state received".to_string()
    } else {
        parts.join(", ")
    }
}

/// The repository an intent is about: `--repo`, or the current one.
fn default_repo(ipc: &Session) -> Result<RepoId, String> {
    ipc.repo_ref()
        .map(|r| r.id.clone())
        .ok_or_else(|| "not inside a git repository (pass --repo)".to_string())
}

/// A backend error that means the daemon connection died is fatal (the MCP
/// server reconnects on it); anything else is an answer about the argument.
fn rejected(message: String, what: impl FnOnce(&str) -> String) -> Result<UiOutcome, String> {
    if message.contains("daemon connection lost") {
        Err(message)
    } else {
        Ok(not_found(what(&message)))
    }
}

async fn resolve(ipc: &mut Session, rev: &str) -> Result<Result<Oid, UiOutcome>, String> {
    match ipc
        .call::<Oid>(OperationKind::Resolve, json!({ "rev": rev }))
        .await
    {
        Ok(oid) => Ok(Ok(oid)),
        Err(m) => rejected(m, |m| format!("revision `{rev}` does not resolve ({m})")).map(Err),
    }
}

/// `Ok(Err(outcome))` = answer with `outcome` instead of routing.
macro_rules! resolved {
    ($ipc:expr, $rev:expr) => {
        match resolve($ipc, $rev).await? {
            Ok(oid) => oid,
            Err(outcome) => return Ok(Err(outcome)),
        }
    };
}

fn norm_path(path: &str) -> String {
    path.replace('\\', "/").trim_start_matches("./").to_string()
}

async fn build_show(ipc: &mut Session, what: UiShowAction) -> Result<Built, String> {
    let mut intent = match what {
        UiShowAction::Tab { path } => {
            let repo = match path {
                Some(p) => discover_repo(Some(&p))?.id,
                None => default_repo(ipc)?,
            };
            UiIntent::ShowTab { repo }
        }
        UiShowAction::Home { filter } => UiIntent::ShowHome { filter },
        UiShowAction::Commit {
            rev,
            also,
            files,
            focus,
        } => {
            let repo = default_repo(ipc)?;
            if also.len() > 9 {
                return Ok(Err(not_found("at most 9 `--also` commits".into())));
            }
            let commit = resolved!(ipc, &rev);
            let mut more = Vec::new();
            for r in &also {
                more.push(resolved!(ipc, r));
            }
            UiIntent::ShowCommit {
                repo,
                commit,
                also: more,
                files,
                focus: focus.as_deref().map(pane),
            }
        }
        UiShowAction::Search { query } => UiIntent::ShowSearch {
            repo: default_repo(ipc)?,
            query,
        },
        UiShowAction::Ref { name } => UiIntent::ShowRef {
            repo: default_repo(ipc)?,
            name,
        },
        UiShowAction::Pr { number, view } => UiIntent::ShowHosting {
            repo: default_repo(ipc)?,
            item: HostingItem::Pr(number.into()),
            view: view.into(),
        },
        UiShowAction::Issue { number, view } => UiIntent::ShowHosting {
            repo: default_repo(ipc)?,
            item: HostingItem::Issue(number.into()),
            view: view.into(),
        },
        UiShowAction::Wip {
            path,
            staged,
            composer,
        } => UiIntent::ShowWip {
            repo: default_repo(ipc)?,
            path: path.map(|p| norm_path(&p)),
            staged,
            composer,
        },
        UiShowAction::Diff {
            path,
            rev,
            unstaged,
            staged,
            line,
            side,
            split,
            inline,
            whole_file,
            hunks,
            highlight,
        } => {
            let repo = default_repo(ipc)?;
            let path = norm_path(&path);
            let (target, backend_target, label) = if unstaged {
                (
                    DiffAt::Unstaged,
                    DiffTarget::WorkingTreeVsIndex,
                    "the unstaged changes".to_string(),
                )
            } else if staged {
                (
                    DiffAt::Staged,
                    DiffTarget::IndexVsHead,
                    "the staged changes".to_string(),
                )
            } else {
                let rev = rev.unwrap_or_else(|| "HEAD".into());
                let oid = resolved!(ipc, &rev);
                (
                    DiffAt::Commit(oid),
                    DiffTarget::CommitVsParent(oid),
                    format!("commit `{rev}`"),
                )
            };
            let listed = ipc
                .call::<DiffFileList>(
                    OperationKind::DiffFiles,
                    json!({ "target": backend_target }),
                )
                .await;
            match listed {
                Ok(list) => {
                    let changed = list
                        .files
                        .iter()
                        .any(|f| norm_path(&f.path.to_string_lossy()) == path);
                    if !changed {
                        return Ok(Err(not_found(format!(
                            "`{path}` is not changed in {label}"
                        ))));
                    }
                }
                Err(m) => {
                    return rejected(m, |m| format!("could not list {label}'s files ({m})"))
                        .map(Err)
                }
            }
            let highlight = match highlight.as_deref().map(parse_range).transpose() {
                Ok(h) => h,
                Err(e) => return Ok(Err(not_found(e))),
            };
            UiIntent::ShowDiff {
                repo,
                target,
                path,
                line,
                side: if side.as_deref() == Some("old") {
                    DiffSide::Old
                } else {
                    DiffSide::New
                },
                layout: match (split, inline) {
                    (true, _) => Some(DiffLayout::Split),
                    (_, true) => Some(DiffLayout::Inline),
                    _ => None,
                },
                whole_file: match (whole_file, hunks) {
                    (true, _) => Some(true),
                    (_, true) => Some(false),
                    _ => None,
                },
                highlight,
            }
        }
        UiShowAction::File {
            path,
            rev,
            worktree,
            index,
            blame,
            history,
            line,
            column,
            highlight,
        } => {
            let repo = default_repo(ipc)?;
            let path = norm_path(&path);
            let at = if worktree {
                FileAt::Worktree
            } else if index {
                FileAt::Index
            } else {
                let rev = rev.unwrap_or_else(|| "HEAD".into());
                let oid = resolved!(ipc, &rev);
                match ipc
                    .call::<Vec<std::path::PathBuf>>(OperationKind::ListTree, json!({ "at": oid }))
                    .await
                {
                    Ok(tree) => {
                        if !tree.iter().any(|p| norm_path(&p.to_string_lossy()) == path) {
                            return Ok(Err(not_found(format!("`{path}` is not in `{rev}`"))));
                        }
                    }
                    Err(m) => {
                        return rejected(m, |m| format!("could not list `{rev}`'s files ({m})"))
                            .map(Err)
                    }
                }
                FileAt::Rev(oid)
            };
            let highlight = match highlight.as_deref().map(parse_range).transpose() {
                Ok(h) => h,
                Err(e) => return Ok(Err(not_found(e))),
            };
            UiIntent::ShowFile {
                repo,
                path,
                at,
                mode: if blame {
                    FileMode::Blame
                } else if history {
                    FileMode::History
                } else {
                    FileMode::Content
                },
                line,
                column,
                highlight,
            }
        }
        UiShowAction::Conflict { path, conflict } => {
            let repo = default_repo(ipc)?;
            let path = norm_path(&path);
            let entry = match conflicted_entry(ipc, &path).await? {
                Ok(entry) => entry,
                Err(outcome) => return Ok(Err(outcome)),
            };
            if let Some(n) = conflict {
                // The count is only known for a text conflict; a card has
                // none, so a number is ignored there.
                if n == 0 {
                    return Ok(Err(not_found("conflict numbers start at 1".to_string())));
                }
                if let Some(count) = conflict_count(ipc, &entry.path.to_string_lossy()).await? {
                    if n as usize > count {
                        return Ok(Err(not_found(format!(
                            "`{path}` has {count} conflict{}, not {n}",
                            if count == 1 { "" } else { "s" }
                        ))));
                    }
                }
            }
            UiIntent::ShowConflict {
                repo,
                path,
                conflict,
            }
        }
        UiShowAction::Settings { page, setting } => {
            let page = match page.as_deref().map(|p| {
                SettingsPage::parse(p).ok_or_else(|| format!("`{p}` is not a settings page"))
            }) {
                Some(Ok(p)) => Some(p),
                Some(Err(e)) => return Ok(Err(not_found(e))),
                None => None,
            };
            if let Some(key) = &setting {
                let valid = !key.is_empty()
                    && key
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'));
                if !valid {
                    return Ok(Err(not_found(format!("`{key}` is not a config key"))));
                }
            }
            UiIntent::ShowSettings { page, setting }
        }
        UiShowAction::Palette { query } => UiIntent::ShowPalette {
            query: query.unwrap_or_default(),
        },
        UiShowAction::Keys { topic } => UiIntent::ShowHelp { topic },
    };
    intent.sanitize();
    Ok(Ok(intent))
}

fn pane(name: &str) -> PaneId {
    match name {
        "refs" => PaneId::Refs,
        "status" => PaneId::Status,
        _ => PaneId::Log,
    }
}

/// `A-B` or `A:C-B:C` (1-based lines, optional columns).
pub(crate) fn parse_range(text: &str) -> Result<LineRange, String> {
    let bad = || format!("`{text}` is not a range (use A-B or A:C-B:C)");
    let (start, end) = text.split_once('-').ok_or_else(bad)?;
    let pos = |s: &str| -> Result<(u32, Option<u32>), String> {
        let (line, col) = match s.split_once(':') {
            Some((l, c)) => (l, Some(c)),
            None => (s, None),
        };
        let line: u32 = line.trim().parse().map_err(|_| bad())?;
        let col = col
            .map(|c| c.trim().parse::<u32>().map_err(|_| bad()))
            .transpose()?;
        if line == 0 || col == Some(0) {
            return Err(bad());
        }
        Ok((line, col))
    };
    let (start_line, start_col) = pos(start)?;
    let (end_line, end_col) = pos(end)?;
    if end_line < start_line {
        return Err(format!("`{text}` ends before it starts"));
    }
    Ok(LineRange {
        start_line,
        end_line,
        start_col,
        end_col,
    })
}

fn no_viewer(ipc: &Session) -> UiOutcome {
    let where_ = ipc
        .repo_root()
        .map(|r| format!(" in {}", r.display()))
        .unwrap_or_default();
    UiOutcome::NoViewer {
        message: format!(
            "No GitPoulpi window is open on this session. Ask the user to run `gitpoulpi` \
             in a terminal{where_}."
        ),
    }
}

async fn send(
    ipc: &mut Session,
    viewer: Option<ViewerId>,
    launch: bool,
    built: Built,
) -> Result<Emitted, String> {
    let intent = match built {
        Ok(intent) => intent,
        Err(outcome) => return Ok(outcome_emitted(&outcome)),
    };
    let args = UiShowArgs {
        viewer: viewer.clone(),
        intent,
        origin: ipc.ui_origin().clone(),
        timeout_ms: None,
    };
    let outcome = show_via(ipc, &args).await?;
    let outcome = match outcome {
        UiOutcome::NoViewer { message } if message == NO_VIEWER_MESSAGE => no_viewer(ipc),
        other => other,
    };
    if viewer.is_some() || !matches!(outcome, UiOutcome::NoViewer { .. }) {
        return Ok(outcome_emitted(&outcome));
    }
    fallback(ipc, args, launch, outcome).await
}

async fn show_via(ipc: &mut Session, args: &UiShowArgs) -> Result<UiOutcome, String> {
    ipc.call_without_repo(
        OperationKind::UiShow,
        serde_json::to_value(args).map_err(|e| e.to_string())?,
    )
    .await
}

/// The own session has no window: look for one in another session (route
/// there when exactly one has the repository open), then, when allowed,
/// launch one and wait for it to attach.
///
/// Launching happens when `--launch` is given, or when `[ui] launch` is set
/// and the caller's `ui_control` is `auto`; never when `ui_control` is `off`
/// and never when `[ui] launch` is `off`.
async fn fallback(
    ipc: &mut Session,
    args: UiShowArgs,
    launch: bool,
    none_here: UiOutcome,
) -> Result<Emitted, String> {
    let repo = args.intent.repo().cloned();
    let own = ipc.session_name().to_string();
    match pick(&scan_sessions(Some(&own)).await, repo.as_ref()) {
        Pick::One(session, port) => return route(ipc, &session, port, &args, false).await,
        Pick::Many(sessions) => return Ok(outcome_emitted(&ambiguous(&sessions))),
        Pick::None => {}
    }
    let control = args.origin.control;
    let template = match ui_launch::configured_template() {
        Ok(t) => t,
        Err(message) => return Ok(outcome_emitted(&UiOutcome::NoViewer { message })),
    };
    let wanted = launch || (template.is_some() && control == UiControl::Auto);
    if !wanted || control == UiControl::Off {
        return Ok(outcome_emitted(&none_here));
    }
    let Some(template) = template else {
        return Ok(outcome_emitted(&UiOutcome::NoViewer {
            message: "No GitPoulpi window is open and launching one is off. The user can pick \
                      a terminal or multiplexer in Settings > General > \"Open a window for \
                      agents\" (`ui.launch`), or run `gitpoulpi` in a terminal."
                .into(),
        }));
    };
    let dir = ipc
        .repo_root()
        .map(Path::to_path_buf)
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_default();
    if let Err(message) = ui_launch::spawn(&template, &dir, &own) {
        return Ok(outcome_emitted(&UiOutcome::NoViewer { message }));
    }
    let wait = std::env::var("GITPOULPI_UI_LAUNCH_WAIT_MS")
        .ok()
        .and_then(|v| v.parse().ok())
        .map_or(ui_launch::DEFAULT_WAIT, Duration::from_millis);
    let deadline = Instant::now() + wait;
    loop {
        tokio::time::sleep(Duration::from_millis(250)).await;
        match pick(&ui_launch::scan_sessions(None).await, repo.as_ref()) {
            Pick::One(session, port) => return route(ipc, &session, port, &args, true).await,
            Pick::Many(sessions) => return Ok(outcome_emitted(&ambiguous(&sessions))),
            Pick::None => {}
        }
        if Instant::now() >= deadline {
            return Ok(outcome_emitted(&UiOutcome::NoViewer {
                message: format!(
                    "Launched a GitPoulpi window, but none attached within {} s (check the \
                     launch command in Settings or `ui.launch_command`).",
                    wait.as_secs()
                ),
            }));
        }
    }
}

fn ambiguous(sessions: &[String]) -> UiOutcome {
    UiOutcome::NoViewer {
        message: format!(
            "GitPoulpi windows with this repository are open in several sessions ({}); \
             pass --session <name> to choose one.",
            sessions.join(", ")
        ),
    }
}

/// Delivers the intent through another session's daemon.
async fn route(
    ipc: &Session,
    session: &str,
    port: u16,
    args: &UiShowArgs,
    launched: bool,
) -> Result<Emitted, String> {
    let mut other = Session::connect_port(session, port, ipc.repo_ref().cloned()).await?;
    let outcome = show_via(&mut other, args).await?;
    let mut emitted = outcome_emitted(&outcome);
    if let Value::Object(map) = &mut emitted.json {
        map.insert("routed_session".into(), json!(session));
        if launched {
            map.insert("launched".into(), json!(true));
        }
    }
    emitted.human = format!(
        "{} (via session `{session}`{})",
        emitted.human,
        if launched { ", window launched" } else { "" }
    );
    Ok(emitted)
}

fn outcome_emitted(outcome: &UiOutcome) -> Emitted {
    let human = match outcome {
        UiOutcome::Applied { state } => format!("shown ({})", describe_state(state)),
        UiOutcome::Partial { note, .. } => format!("shown, with a note: {note}"),
        UiOutcome::Proposed { id, .. } => {
            format!("proposed ({id}): the dialog is open, the user decides")
        }
        UiOutcome::Deferred { reason } => {
            format!("the window deferred it ({reason:?}): the user decides whether to show it")
        }
        UiOutcome::Refused { reason } => format!("refused: {reason}"),
        UiOutcome::NotFound { what } => format!("not found: {what}"),
        UiOutcome::NoViewer { message } => message.clone(),
    };
    Emitted {
        json: serde_json::to_value(outcome).unwrap_or(Value::Null),
        human,
        exit_code: outcome.exit_code(),
    }
}

/// A fresh proposal id: `p-` and 12 hex digits.
fn proposal_id() -> String {
    use std::hash::{BuildHasher, Hasher};
    let mut h = std::collections::hash_map::RandomState::new().build_hasher();
    h.write_u128(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos()),
    );
    h.write_u32(std::process::id());
    format!("p-{:012x}", h.finish() & 0xffff_ffff_ffff)
}

/// What `ui propose ...` asks for, checked before routing: revisions
/// resolve, texts are sanitized and not empty.
async fn build_propose(
    ipc: &mut Session,
    id: &str,
    what: UiProposeAction,
) -> Result<Built, String> {
    let repo = default_repo(ipc)?;
    let empty = |field: &str| Ok(Err(not_found(format!("the proposed {field} is empty"))));
    let mut proposal = match what {
        UiProposeAction::Resolution {
            path,
            pick,
            pick_default,
            content_file,
        } => {
            let path = norm_path(&path);
            let entry = match conflicted_entry(ipc, &path).await? {
                Ok(entry) => entry,
                Err(outcome) => return Ok(Err(outcome)),
            };
            let doc = crate::commands::conflicts::document_of(ipc, &path).await?;
            if doc.regions.is_empty() {
                return Ok(Err(not_found(format!(
                    "`{path}` has no text conflicts (binary, deleted on one side, symlink or \
                     submodule): a proposal cannot pre-fill it"
                ))));
            }
            let (picks, content) = match (pick, content_file) {
                (Some(spec), None) => {
                    match crate::commands::conflicts::picks_from_spec(
                        &doc,
                        &spec,
                        pick_default.as_deref(),
                    ) {
                        Ok(picks) => (Some(picks), None),
                        Err(crate::commands::conflicts::PickProblem::Bad(m)) => {
                            return Ok(Err(not_found(m)))
                        }
                        Err(crate::commands::conflicts::PickProblem::Missing(missing)) => {
                            return Ok(Err(not_found(format!(
                                "--pick needs a choice for every conflict; missing {} (or pass \
                                 --default a|b)",
                                missing
                                    .iter()
                                    .map(usize::to_string)
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            ))))
                        }
                    }
                }
                (None, Some(file)) => match crate::commands::conflicts::read_text_file(&file) {
                    Ok(text) if text.trim().is_empty() => return empty("content"),
                    Ok(text) => (None, Some(text)),
                    Err(m) => return Ok(Err(not_found(m))),
                },
                _ => return Ok(Err(not_found("give --pick or --content-file".to_string()))),
            };
            Proposal::ConflictResolution {
                path,
                expected: gitpoulpi_core::domain::ResolveOptions::oids_of(&entry),
                picks,
                content,
            }
        }
        UiProposeAction::Reword { rev, message } => {
            if sanitize_message(&message).trim().is_empty() {
                return empty("message");
            }
            Proposal::Reword {
                commit: resolved!(ipc, &rev),
                message,
            }
        }
        UiProposeAction::Split { rev, plan } => {
            let parts = match crate::commands::split::read_split_plan(&plan) {
                Ok(parts) => parts,
                Err(m) => return Ok(Err(not_found(m))),
            };
            if parts.len() < 2 {
                return Ok(Err(not_found(
                    "a split plan needs at least two parts".to_string(),
                )));
            }
            Proposal::SplitCommit {
                commit: resolved!(ipc, &rev),
                parts,
            }
        }
        #[cfg(feature = "history-rewrite")]
        UiProposeAction::Rewrite { recipe } => match rewrite_recipe(&recipe) {
            Ok(recipe) => {
                // Secret values never travel in a proposal.
                let proposal = Proposal::Rewrite {
                    recipe: Box::new(recipe.saved()),
                };
                if let Some(why) = proposal.problem() {
                    return Ok(Err(not_found(why)));
                }
                proposal
            }
            Err(m) => return Ok(Err(not_found(m))),
        },
        UiProposeAction::CommitMessage { summary, body } => {
            if sanitize_with_limit(&summary, 200).trim().is_empty() {
                return empty("summary");
            }
            Proposal::CommitMessage {
                summary,
                body: body.unwrap_or_default(),
            }
        }
        UiProposeAction::Branch { name, start_point } => {
            if sanitize_with_limit(&name, 200).trim().is_empty() {
                return empty("branch name");
            }
            Proposal::Branch {
                name,
                start_point: resolved!(ipc, start_point.as_deref().unwrap_or("HEAD")),
            }
        }
        UiProposeAction::Tag {
            name,
            target,
            message,
        } => {
            if sanitize_with_limit(&name, 200).trim().is_empty() {
                return empty("tag name");
            }
            if message
                .as_deref()
                .is_some_and(|m| sanitize_message(m).trim().is_empty())
            {
                return empty("tag message");
            }
            Proposal::Tag {
                name,
                target: resolved!(ipc, target.as_deref().unwrap_or("HEAD")),
                message,
            }
        }
    };
    proposal.sanitize();
    Ok(Ok(UiIntent::Propose {
        repo,
        id: id.to_string(),
        proposal,
    }))
}

#[cfg(feature = "history-rewrite")]
fn rewrite_recipe(file: &str) -> Result<gitpoulpi_core::domain::rewrite::Recipe, String> {
    crate::commands::rewrite::read_recipe_file(file)
}

/// The conflicted entry of `path`, or the outcome to answer instead.
async fn conflicted_entry(
    ipc: &mut Session,
    path: &str,
) -> Result<Result<gitpoulpi_core::domain::ConflictEntry, UiOutcome>, String> {
    let entries: Vec<gitpoulpi_core::domain::ConflictEntry> =
        ipc.call(OperationKind::ConflictEntries, json!({})).await?;
    Ok(entries
        .into_iter()
        .find(|e| norm_path(&e.path.to_string_lossy()) == path)
        .ok_or_else(|| not_found(format!("`{path}` is not a conflicted file"))))
}

/// How many text conflicts `path` has, `None` when it has no text regions
/// (a card in the editor).
async fn conflict_count(ipc: &mut Session, path: &str) -> Result<Option<usize>, String> {
    let doc = crate::commands::conflicts::document_of(ipc, path).await?;
    Ok((!doc.regions.is_empty()).then(|| doc.conflict_count()))
}

/// The CLI's own gate on `ui propose` (`[mcp] allow_ui_propose`, off by
/// default): an agent with a shell reaches the CLI as easily as the MCP
/// tools, which `gitpoulpi-mcp`'s policy gates the same way.
pub(crate) fn propose_refused() -> Emitted {
    outcome_emitted(&UiOutcome::refused(
        "proposals are disabled (mcp.allow_ui_propose = false in config.toml; the user can \
         turn on Settings > General > \"Let AI agents propose changes for me to confirm\")",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ranges_parse_with_and_without_columns() {
        assert_eq!(
            parse_range("3-5").unwrap(),
            LineRange {
                start_line: 3,
                end_line: 5,
                start_col: None,
                end_col: None
            }
        );
        let r = parse_range("3:2-5:9").unwrap();
        assert_eq!((r.start_col, r.end_col), (Some(2), Some(9)));
        for bad in ["", "3", "0-4", "5-3", "a-b", "3:0-4", "3-4-5"] {
            assert!(parse_range(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn paths_are_normalized_to_forward_slashes() {
        assert_eq!(norm_path(".\\src\\a.rs"), "src/a.rs");
        assert_eq!(norm_path("./a"), "a");
    }
}
