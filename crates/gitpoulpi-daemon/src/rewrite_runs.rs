//! The daemon's side of a history rewrite (the history-rewrite design,
//! "Preview first"): the live plan of each repository, the pages of its
//! rows, the preview's progress and cancel hook, and the sweep of run
//! directories nobody came back for.
//!
//! The backend writes the preview's pack under `<git-dir>/gitpoulpi/rewrite/<run>/`
//! and hands back the whole plan, rows included. The plan stays here, one per
//! repository (a new preview replaces the previous one and discards its run);
//! what crosses the IPC is the plan without rows plus pages of them.

use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::ControlFlow;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime};

use gitpoulpi_core::domain::rewrite::{
    Recipe, RewritePhase, RewritePlan, RewriteProgress, RewriteWarning, RunId,
};
use gitpoulpi_core::domain::{RepoId, RepoRef};
use gitpoulpi_core::{Error, GitBackend, Result};
use gitpoulpi_ipc::rewrite::{RowsArgs, RowsPage, MAX_ROWS_PAGE};
use gitpoulpi_ipc::{Event, Request};
use tokio::sync::broadcast;

use crate::dispatch::{err_kind, Dispatched};
use crate::jobs::JobCtl;

/// A run directory older than this is removed (the next preview of the
/// repository sweeps; the maps a finished apply keeps go the same way).
pub(crate) const KEEP_RUNS: Duration = Duration::from_secs(24 * 3600);

/// Progress events are sent at most this often.
const EVERY: Duration = Duration::from_millis(100);

// ------------------------------------------------------------ plan store

static LIVE: Mutex<Option<HashMap<RepoId, Arc<RewritePlan>>>> = Mutex::new(None);

fn live_map<T>(f: impl FnOnce(&mut HashMap<RepoId, Arc<RewritePlan>>) -> T) -> T {
    let mut guard = LIVE.lock().unwrap_or_else(|e| e.into_inner());
    f(guard.get_or_insert_with(HashMap::new))
}

/// The repository's live plan when it is run `run`.
pub(crate) fn live(repo: &RepoRef, run: &RunId) -> Option<Arc<RewritePlan>> {
    live_map(|m| m.get(&repo.id).filter(|p| &p.run == run).cloned())
}

/// The plan of run `run`: the live one, else the one its run directory
/// holds (a preview made by another process, or before a restart). A plan
/// reloaded this way has no rows; it becomes the live plan, and the run it
/// replaces keeps its directory (the sweep removes it when old).
pub(crate) fn live_or_load(
    backend: &dyn GitBackend,
    repo: &RepoRef,
    run: &RunId,
) -> Option<Arc<RewritePlan>> {
    if let Some(plan) = live(repo, run) {
        return Some(plan);
    }
    let plan = backend.rewrite_load(repo, run).ok().flatten()?;
    let plan = Arc::new(plan);
    live_map(|m| m.insert(repo.id.clone(), plan.clone()));
    Some(plan)
}

/// The repository's live plan, whatever its run.
pub(crate) fn current(repo: &RepoRef) -> Option<Arc<RewritePlan>> {
    live_map(|m| m.get(&repo.id).cloned())
}

/// Makes `plan` the repository's live one and returns the plan it replaced.
pub(crate) fn store(repo: &RepoRef, plan: RewritePlan) -> Option<Arc<RewritePlan>> {
    live_map(|m| m.insert(repo.id.clone(), Arc::new(plan)))
}

/// Forgets the live plan when it is run `run`.
pub(crate) fn forget(repo: &RepoRef, run: &RunId) {
    live_map(|m| {
        if m.get(&repo.id).is_some_and(|p| &p.run == run) {
            m.remove(&repo.id);
        }
    });
}

pub(crate) fn forget_all() {
    live_map(HashMap::clear);
}

/// The plan as it crosses the IPC: everything but the rows.
pub(crate) fn wire(plan: &RewritePlan) -> RewritePlan {
    // Field by field: cloning the plan to drop its rows would copy 100 000.
    RewritePlan {
        run: plan.run.clone(),
        digest: plan.digest.clone(),
        counts: plan.counts,
        refs: plan.refs.clone(),
        warnings: plan.warnings.clone(),
        per_rule: plan.per_rule.clone(),
        rows: Vec::new(),
    }
}

/// One page of the live plan's rows.
pub(crate) fn page(plan: &RewritePlan, args: &RowsArgs) -> RowsPage {
    let limit = args.limit.clamp(1, MAX_ROWS_PAGE);
    let matching = || plan.rows.iter().filter(|r| args.filter.matches(r));
    RowsPage {
        run: plan.run.clone(),
        filter: args.filter,
        offset: args.offset,
        total: matching().count(),
        rows: matching().skip(args.offset).take(limit).cloned().collect(),
    }
}

/// Whether applying `plan` moves a ref that is on a remote: the apply then
/// asks for a confirmation.
pub(crate) fn touches_pushed(plan: &RewritePlan) -> Option<u32> {
    plan.warnings.iter().find_map(|w| match w {
        RewriteWarning::PushedRefs { count } if *count > 0 => Some(*count),
        _ => None,
    })
}

// ----------------------------------------------------------- run directories

fn runs_dir(repo: &RepoRef) -> Option<PathBuf> {
    crate::journal::git_dir(&repo.root).map(|d| d.join("gitpoulpi").join("rewrite"))
}

/// Removes the run directories under `dir` last touched before `now - keep`,
/// except `spare`. Returns how many went.
pub(crate) fn sweep_dir(dir: &Path, now: SystemTime, keep: Duration, spare: Option<&str>) -> usize {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return 0;
    };
    let mut removed = 0;
    for entry in entries.flatten() {
        let name = entry.file_name();
        if spare.is_some_and(|s| name.to_str() == Some(s)) {
            continue;
        }
        let Ok(meta) = entry.metadata() else { continue };
        if !meta.is_dir() {
            continue;
        }
        let old = meta
            .modified()
            .ok()
            .and_then(|t| now.duration_since(t).ok())
            .is_some_and(|age| age > keep);
        if old && std::fs::remove_dir_all(entry.path()).is_ok() {
            removed += 1;
        }
    }
    removed
}

/// The sweep, once per preview: old runs of this repository go, the live one
/// stays.
pub(crate) fn sweep(repo: &RepoRef) {
    let Some(dir) = runs_dir(repo) else { return };
    let live = current(repo);
    let removed = sweep_dir(
        &dir,
        SystemTime::now(),
        KEEP_RUNS,
        live.as_ref().map(|p| p.run.0.as_str()),
    );
    if removed > 0 {
        tracing::info!(target: "daemon", "removed {removed} old history rewrite run(s)");
    }
}

// ---------------------------------------------------- progress and cancel

/// What a running preview needs from its job: the cancel flag and a place
/// to report progress. Installed on the blocking thread that runs the
/// backend, so the progress sink the dispatcher hands the engine reaches it.
pub(crate) struct Control {
    ctl: JobCtl,
    repo: RepoId,
    events: broadcast::Sender<Event>,
    /// `GITPOULPI_TEST_REWRITE_TICK_DELAY_MS`: a test makes every progress
    /// step slow, so there is time to cancel.
    delay: Option<Duration>,
    last: Option<(Instant, RewritePhase)>,
}

impl Control {
    pub(crate) fn new(ctl: JobCtl, repo: RepoId, events: broadcast::Sender<Event>) -> Self {
        let delay = std::env::var("GITPOULPI_TEST_REWRITE_TICK_DELAY_MS")
            .ok()
            .and_then(|v| v.trim().parse::<u64>().ok())
            .map(Duration::from_millis);
        Self {
            ctl,
            repo,
            events,
            delay,
            last: None,
        }
    }

    /// Runs `work` with this control installed on the current thread.
    pub(crate) fn run<T>(self, work: impl FnOnce() -> T) -> T {
        struct Reset;
        impl Drop for Reset {
            fn drop(&mut self) {
                CONTROL.with(|c| *c.borrow_mut() = None);
            }
        }
        CONTROL.with(|c| *c.borrow_mut() = Some(self));
        let _reset = Reset;
        work()
    }

    fn tick(&mut self, progress: RewriteProgress) -> ControlFlow<()> {
        if let Some(delay) = self.delay {
            std::thread::sleep(delay);
        }
        if self.ctl.canceled() {
            return ControlFlow::Break(());
        }
        let now = Instant::now();
        let due = match self.last {
            Some((at, phase)) => phase != progress.phase || now.duration_since(at) >= EVERY,
            None => true,
        };
        if due {
            self.last = Some((now, progress.phase));
            self.ctl
                .progress(phase_name(progress.phase), Some(overall(progress)));
            let _ = self.events.send(Event::RewriteProgress {
                repo: self.repo.clone(),
                job: self.ctl.id(),
                progress,
            });
        }
        ControlFlow::Continue(())
    }
}

/// Tells the job running on this thread which step a purge is in (a purge
/// reports a phase name and a percentage; it has no cancel).
pub(crate) fn purge_step(step: gitpoulpi_core::domain::rewrite::PurgeStep) {
    CONTROL.with(|c| {
        if let Some(control) = c.borrow().as_ref() {
            control.ctl.progress(step.name(), Some(step.percent()));
        }
    });
}

thread_local! {
    static CONTROL: RefCell<Option<Control>> = const { RefCell::new(None) };
}

/// The progress sink handed to the engine: reports to the job running on
/// this thread and breaks when it was canceled. Without a job (a caller that
/// is not a job) it never breaks.
pub(crate) fn tick(progress: RewriteProgress) -> ControlFlow<()> {
    CONTROL.with(|c| match c.borrow_mut().as_mut() {
        Some(control) => control.tick(progress),
        None => ControlFlow::Continue(()),
    })
}

/// The job number the engine's canceled error is about (the backend does not
/// know it).
fn job_number() -> u64 {
    CONTROL.with(|c| c.borrow().as_ref().map_or(0, |control| control.ctl.id()))
}

pub(crate) fn phase_name(phase: RewritePhase) -> &'static str {
    match phase {
        RewritePhase::Walking => "Walking",
        RewritePhase::Rewriting => "Rewriting",
        RewritePhase::Indexing => "Indexing",
        RewritePhase::Updating => "Updating",
        RewritePhase::Signing => "Signing",
    }
}

/// The whole preview as a percentage: reading history is the first half,
/// rewriting the next 40 %, then the pack and the refs.
pub(crate) fn overall(progress: RewriteProgress) -> u8 {
    let (from, span) = match progress.phase {
        RewritePhase::Walking => (0u64, 50u64),
        RewritePhase::Rewriting => (50, 40),
        // Signing happens inside the rewriting stretch.
        RewritePhase::Signing => (50, 40),
        RewritePhase::Indexing => (90, 9),
        RewritePhase::Updating => (99, 0),
    };
    let inside = (progress.done.min(progress.total) * span)
        .checked_div(progress.total)
        .unwrap_or(0);
    (from + inside).min(99) as u8
}

// -------------------------------------------------------------- the preview

/// The names of the repository's protected branches, for a recipe whose
/// caller did not list them: `rewrite.protected_branches` (names and `*`
/// patterns; default `main`, `master`, `develop`, `release/*`), and the
/// workflow's protected bases. The backend reads the files next to the git
/// dir, the daemon also knows the per-repository state directory.
fn protected_names(repo: &RepoRef) -> Vec<String> {
    let mut names: Vec<String> = gitpoulpi_config::Settings::resolve_ambient()
        .map(|s| s.config().rewrite.protected_branches.clone())
        .unwrap_or_else(|_| {
            gitpoulpi_core::domain::rewrite::DEFAULT_PROTECTED_BRANCHES
                .iter()
                .map(|s| s.to_string())
                .collect()
        });
    if let Ok(loaded) =
        gitpoulpi_session::LoadedWorkflow::load(&repo.root, &crate::workflow::state_dir())
    {
        names.extend(
            loaded
                .effective()
                .bases
                .iter()
                .filter(|b| b.protected)
                .flat_map(|b| b.all_names().map(str::to_string).collect::<Vec<_>>()),
        );
    }
    names
}

/// What external scripts may do on a request: the `rewrite.external_scripts`
/// switches and timeouts from the settings, and the scripts trusted for this
/// repository (its state file) plus the ones the caller `vouched` for on this
/// request. Always built here, never taken from a client.
pub(crate) fn script_policy(
    repo: &RepoRef,
    vouched: &[String],
) -> gitpoulpi_core::domain::rewrite::ScriptPolicy {
    use gitpoulpi_core::domain::rewrite::ScriptPolicy;
    let mut policy = gitpoulpi_config::Settings::resolve_ambient().map_or_else(
        |_| ScriptPolicy::default(),
        |s| {
            let r = &s.config().rewrite;
            ScriptPolicy::from_settings(
                &r.external_scripts,
                r.script_timeout_ms,
                r.script_total_timeout_s,
            )
        },
    );
    let state =
        gitpoulpi_session::RepoState::load(&crate::autostash::state_path(repo)).unwrap_or_default();
    policy.trusted = state
        .trusted_scripts
        .iter()
        .map(|t| t.hash.clone())
        .chain(vouched.iter().cloned())
        .collect();
    policy
}

/// The switches for the workbench (`RewriteTools.external_scripts`).
pub(crate) fn script_switches() -> gitpoulpi_core::domain::rewrite::ScriptSwitches {
    let policy = script_policy_without_trust();
    gitpoulpi_core::domain::rewrite::ScriptSwitches {
        shell: policy.shell,
        python: policy.python,
        lua: policy.lua,
    }
}

fn script_policy_without_trust() -> gitpoulpi_core::domain::rewrite::ScriptPolicy {
    use gitpoulpi_core::domain::rewrite::ScriptPolicy;
    gitpoulpi_config::Settings::resolve_ambient().map_or_else(
        |_| ScriptPolicy::default(),
        |s| {
            let r = &s.config().rewrite;
            ScriptPolicy::from_settings(
                &r.external_scripts,
                r.script_timeout_ms,
                r.script_total_timeout_s,
            )
        },
    )
}

/// `RewriteTrustScript`: remembers that the user trusts the script with this
/// content hash in this repository.
pub(crate) fn trust_script(repo: &RepoRef, hash: &str, label: &str) -> Result<()> {
    let hex = hash.len() == 64 && hash.bytes().all(|b| b.is_ascii_hexdigit());
    if !hex {
        return Err(Error::RewriteRefused(
            gitpoulpi_core::domain::rewrite::RewriteRefusal::InvalidRule,
            "a script hash is 64 hexadecimal characters".into(),
        ));
    }
    let path = crate::autostash::state_path(repo);
    let mut state = gitpoulpi_session::RepoState::load(&path).unwrap_or_default();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() as i64);
    state.trust_script(&hash.to_ascii_lowercase(), label, now);
    state
        .save(&path)
        .map_err(|e| Error::Io(std::io::Error::other(e.to_string())))?;
    tracing::info!(target: "daemon", "trusted a rewrite script ({label}, {hash})");
    Ok(())
}

/// `RewritePreview`: runs the recipe, keeps the full plan, answers with the
/// plan without rows. A new live plan replaces (and discards the run of) the
/// previous one; a failed or canceled preview leaves the previous one alone.
pub(crate) fn preview(
    backend: &dyn GitBackend,
    repo: &RepoRef,
    mut recipe: Recipe,
    vouched: &[String],
) -> Result<RewritePlan> {
    // Whatever policy the client sent is replaced: it cannot grant itself
    // anything.
    recipe.script_policy = script_policy(repo, vouched);
    // S23: what the host protects (read client side for a linked repository)
    // is protected here exactly like the workflow's and the config's names.
    let from_host = recipe
        .scope_options
        .host_protection
        .iter()
        .flat_map(|h| h.patterns.iter().cloned());
    for name in protected_names(repo).into_iter().chain(from_host) {
        if !recipe.scope_options.protected_branches.contains(&name) {
            recipe.scope_options.protected_branches.push(name);
        }
    }
    sweep(repo);
    let mut sink = |p: RewriteProgress| tick(p);
    let plan = backend
        .rewrite_preview(repo, &recipe, &mut sink)
        .map_err(|e| match e {
            Error::Canceled { .. } => Error::Canceled { job: job_number() },
            other => other,
        })?;
    let answer = wire(&plan);
    if let Some(previous) = store(repo, plan) {
        if previous.run != answer.run {
            let _ = backend.rewrite_discard(repo, &previous.run);
        }
    }
    Ok(answer)
}

/// `RewriteScanSecrets`: findings of the scope's history, a job that
/// reports progress and stops when canceled. Nothing is kept: a scan is a
/// read, and the values stay in the backend.
pub(crate) fn scan_secrets(
    backend: &dyn GitBackend,
    repo: &RepoRef,
    scope: &gitpoulpi_core::domain::rewrite::Scope,
) -> Result<gitpoulpi_core::domain::rewrite::SecretScan> {
    let mut sink = |p: RewriteProgress| tick(p);
    backend
        .rewrite_scan_secrets(repo, scope, &mut sink)
        .map_err(|e| match e {
            Error::Canceled { .. } => Error::Canceled { job: job_number() },
            other => other,
        })
}

/// `RewriteAnalyze`: the size analysis, a job like the scan (progress,
/// cancel, nothing kept).
pub(crate) fn analyze(
    backend: &dyn GitBackend,
    repo: &RepoRef,
    scope: &gitpoulpi_core::domain::rewrite::Scope,
    top: u32,
) -> Result<gitpoulpi_core::domain::rewrite::SizeAnalysis> {
    let mut sink = |p: RewriteProgress| tick(p);
    let top = if top == 0 {
        gitpoulpi_core::domain::rewrite::DEFAULT_TOP
    } else {
        top
    };
    backend
        .rewrite_analyze(repo, scope, top, &mut sink)
        .map_err(|e| match e {
            Error::Canceled { .. } => Error::Canceled { job: job_number() },
            other => other,
        })
}

/// The refusal for an apply whose run is not the live one (restarted
/// daemon, a newer preview, a different digest).
pub(crate) fn stale(detail: &str) -> Error {
    Error::RewriteRefused(
        gitpoulpi_core::domain::rewrite::RewriteRefusal::Stale,
        detail.to_string(),
    )
}

/// The canceled answer of an apply that was canceled before it started.
pub(crate) fn canceled_apply(request: &Request, job: u64) -> Dispatched {
    let error = Error::Canceled { job };
    err_kind(request.id, error.to_string(), false, error.typed())
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitpoulpi_core::domain::rewrite::{CommitChange, Counts, PlanDigest};
    use gitpoulpi_core::domain::Oid;
    use gitpoulpi_ipc::rewrite::RowFilter;

    fn row(i: u8, new: Option<u8>, pushed: bool) -> CommitChange {
        CommitChange {
            old: Oid([i; 20]),
            new: new.map(|n| Oid([n; 20])),
            fields: vec![],
            paths: vec![],
            pushed,
        }
    }

    fn plan() -> RewritePlan {
        RewritePlan {
            run: RunId("r".into()),
            digest: PlanDigest("d".into()),
            counts: Counts::default(),
            refs: vec![],
            warnings: vec![],
            per_rule: vec![],
            rows: vec![
                row(1, Some(2), false),
                row(3, None, true),
                row(5, Some(5), false),
                row(6, Some(7), true),
            ],
        }
    }

    fn args(filter: RowFilter, offset: usize, limit: usize) -> RowsArgs {
        RowsArgs {
            run: RunId("r".into()),
            filter,
            offset,
            limit,
        }
    }

    #[test]
    fn a_page_counts_the_whole_filter_and_slices_it() {
        let p = plan();
        let changed = page(&p, &args(RowFilter::Changed, 0, 2));
        assert_eq!((changed.total, changed.rows.len()), (3, 2));
        let rest = page(&p, &args(RowFilter::Changed, 2, 2));
        assert_eq!((rest.total, rest.rows.len()), (3, 1));
        assert_eq!(page(&p, &args(RowFilter::Dropped, 0, 9)).rows.len(), 1);
        assert_eq!(page(&p, &args(RowFilter::Pushed, 0, 9)).total, 2);
        assert_eq!(page(&p, &args(RowFilter::All, 0, 9)).total, 4);
        assert!(page(&p, &args(RowFilter::All, 9, 9)).rows.is_empty());
    }

    #[test]
    fn a_page_is_never_bigger_than_the_cap() {
        let mut p = plan();
        p.rows = (0..500).map(|i| row(i as u8, Some(1), false)).collect();
        let got = page(&p, &args(RowFilter::All, 0, 10_000));
        assert_eq!(got.rows.len(), MAX_ROWS_PAGE);
        assert_eq!(got.total, 500);
    }

    #[test]
    fn the_wire_plan_has_no_rows() {
        assert!(wire(&plan()).rows.is_empty());
    }

    #[test]
    fn progress_is_one_rising_percentage() {
        let at = |phase, done, total| overall(RewriteProgress { phase, done, total });
        assert_eq!(at(RewritePhase::Walking, 0, 0), 0);
        assert_eq!(at(RewritePhase::Walking, 5, 10), 25);
        assert_eq!(at(RewritePhase::Rewriting, 0, 10), 50);
        assert_eq!(at(RewritePhase::Rewriting, 10, 10), 90);
        assert_eq!(at(RewritePhase::Indexing, 0, 10), 90);
        assert!(at(RewritePhase::Updating, 1, 1) < 100);
    }

    #[test]
    fn the_sweep_removes_old_runs_and_spares_the_live_one() {
        let dir = std::env::temp_dir().join(format!("gp-sweep-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        for name in ["old", "live"] {
            std::fs::create_dir_all(dir.join(name)).unwrap();
        }
        std::fs::write(dir.join("stray-file"), "x").unwrap();
        std::thread::sleep(Duration::from_millis(200));
        let now = SystemTime::now();
        std::thread::sleep(Duration::from_millis(200));
        std::fs::create_dir_all(dir.join("fresh")).unwrap();
        let removed = sweep_dir(&dir, now, Duration::from_millis(50), Some("live"));
        assert_eq!(removed, 1);
        assert!(!dir.join("old").exists());
        assert!(dir.join("live").exists(), "the live run stays");
        assert!(dir.join("fresh").exists(), "a recent run stays");
        assert!(dir.join("stray-file").exists(), "only directories go");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
