//! User-defined action sequences: the run side.
//!
//! * [`registry`] - which slug owns which keymap slot.
//! * [`machine`] - the pure state machine of one run.
//! * [`exec`] - pure helpers that decide how a step is executed.
//! * [`strip`] - the strip over the Log that shows the run.
//! * this file - the process-wide slot holding the one run, the requests
//!   (cancel, continue) the buttons, keys and the Jobs panel leave for the
//!   window's next frame, and `start`.
//!
//! The window's per-frame glue is `event_loop/window/sequence.rs`: it owns
//! the command layer, this module owns no window state.
//!
//! One run at a time, process-wide: a tab switch rebuilds the window, the run
//! goes on in the next one. It is not persisted: quitting leaves one
//! Activity Log line for an interrupted run.

pub(crate) mod exec;
pub(crate) mod machine;
pub(crate) mod registry;
pub(crate) mod strip;

#[cfg(test)]
mod tests;

use machine::Run;
use std::path::Path;
#[cfg(not(test))]
use std::sync::Mutex;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// The run, or a mark that the window has it out for a frame.
struct Slot {
    run: Option<Run>,
    /// The glue holds the run (taken with [`take`], put back with
    /// [`put_back`]).
    taken: bool,
    /// name, k, n of the run that is out, for the "is running" refusal.
    summary: Option<(String, usize, usize)>,
    cancel: bool,
    cont: bool,
    go_there: bool,
    /// The status refreshes the windows applied (the runner waits for one
    /// after a mutation).
    refreshes: u64,
}

impl Slot {
    const fn new() -> Slot {
        Slot {
            run: None,
            taken: false,
            summary: None,
            cancel: false,
            cont: false,
            go_there: false,
            refreshes: 0,
        }
    }
}

#[cfg(not(test))]
static RUN: Mutex<Slot> = Mutex::new(Slot::new());

// Unit tests run in parallel threads, each with a slot of its own.
#[cfg(test)]
thread_local! {
    static RUN: std::cell::RefCell<Slot> = const { std::cell::RefCell::new(Slot::new()) };
}

fn with<R>(f: impl FnOnce(&mut Slot) -> R) -> R {
    #[cfg(not(test))]
    {
        f(&mut RUN.lock().unwrap_or_else(|e| e.into_inner()))
    }
    #[cfg(test)]
    {
        RUN.with(|r| f(&mut r.borrow_mut()))
    }
}

/// A sequence is running (or the glue has it for this frame). A failed one
/// still shown does not count.
pub(crate) fn active() -> bool {
    with(|s| s.taken || s.run.as_ref().is_some_and(|r| !r.is_over()))
}

/// A run is on screen: running, or failed and waiting to be dismissed.
pub(crate) fn visible() -> bool {
    with(|s| s.taken || s.run.is_some())
}

/// The run can be continued from a button right now.
pub(crate) fn can_continue() -> bool {
    with(|s| s.run.as_ref().is_some_and(Run::can_continue))
}

/// Reads the run for drawing. `None` while the glue has it out.
pub(crate) fn peek<R>(f: impl FnOnce(&Run) -> R) -> Option<R> {
    with(|s| s.run.as_ref().map(f))
}

/// The cancel request the next frame carries out (strip ✕, Jobs panel, the
/// palette row, Esc).
pub(crate) fn request_cancel() {
    with(|s| s.cancel = true);
    crate::redraw::poke();
}

/// The Continue request (strip button, palette row, key).
pub(crate) fn request_continue() {
    with(|s| s.cont = true);
    crate::redraw::poke();
}

/// The `[Go there]` button: switch to the tab the step acts on.
pub(crate) fn request_go_there() {
    with(|s| s.go_there = true);
    crate::redraw::poke();
}

/// What was asked since the last frame.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct Requests {
    pub(crate) cancel: bool,
    pub(crate) cont: bool,
    pub(crate) go_there: bool,
}

/// The requests left since the last frame.
pub(crate) fn take_requests() -> Requests {
    with(|s| Requests {
        cancel: std::mem::take(&mut s.cancel),
        cont: std::mem::take(&mut s.cont),
        go_there: std::mem::take(&mut s.go_there),
    })
}

/// A real status refresh was applied by a window.
pub(crate) fn note_status_refresh() {
    with(|s| s.refreshes += 1);
}

pub(crate) fn refreshes() -> u64 {
    with(|s| s.refreshes)
}

/// Takes the run out for one frame of glue. `None` when there is none (or it
/// is already out).
pub(crate) fn take() -> Option<Run> {
    with(|s| {
        let run = s.run.take()?;
        s.summary = Some({
            let (k, n) = run.position();
            (run.name.clone(), k, n)
        });
        s.taken = true;
        Some(run)
    })
}

/// Puts the run back. A run that ended well or was canceled leaves; one that
/// failed stays until dismissed. Closes the job of a run that is over.
pub(crate) fn put_back(run: Run) {
    let finish = run.is_over();
    let job = run.job_id;
    let keep = !finish || matches!(run.state, machine::StepState::Failed { .. });
    let failed = matches!(run.state, machine::StepState::Failed { .. });
    let canceled = matches!(run.state, machine::StepState::Canceled);
    with(|s| {
        s.taken = false;
        s.summary = None;
        s.run = keep.then_some(run);
    });
    if finish {
        crate::daemon_link::disarm();
        if let Some(job) = job {
            crate::jobs_ui::local_finish(
                job,
                if failed {
                    gitpoulpi_ipc::JobState::Failed
                } else if canceled {
                    gitpoulpi_ipc::JobState::Canceled
                } else {
                    gitpoulpi_ipc::JobState::Done
                },
            );
        }
    }
}

/// Starts `run`. One at a time: a second start is refused with the
/// running one's name and step. A failed run still on screen is dismissed.
pub(crate) fn begin(mut run: Run) -> Result<(), String> {
    let refusal = with(|s| {
        if s.taken || s.run.as_ref().is_some_and(|r| !r.is_over()) {
            let (name, k, n) = match (&s.run, &s.summary) {
                (Some(r), _) => {
                    let (k, n) = r.position();
                    (r.name.clone(), k, n)
                }
                (None, Some(sum)) => sum.clone(),
                _ => (String::new(), 0, 0),
            };
            return Some(gitpoulpi_i18n::msg::sequence_run::say_running(name, k, n));
        }
        s.run = None;
        s.cancel = false;
        s.cont = false;
        None
    });
    if let Some(text) = refusal {
        return Err(text);
    }
    let stop: Arc<dyn Fn() + Send + Sync> = Arc::new(request_cancel);
    run.job_id = Some(crate::jobs_ui::local_start(run.job_label(), stop));
    with(|s| s.run = Some(run));
    crate::redraw::poke();
    Ok(())
}

/// The process is ending with a run unfinished (runs are not persisted): one
/// line in the application log, which the Activity Log shows.
pub(crate) fn note_interrupted() {
    if !active() {
        return;
    }
    let line = with(|s| {
        s.run.as_ref().map(|r| {
            let (k, n) = r.position();
            gitpoulpi_i18n::msg::sequence_run::log_interrupted(&r.name, k, n, r.step_label())
        })
    });
    if let Some(line) = line {
        tracing::warn!("{line}");
    }
}

/// The loop's wake-up while a run goes: the window's frame is what moves it.
pub(crate) fn next_deadline() -> Option<Instant> {
    active().then(|| Instant::now() + Duration::from_millis(100))
}

/// Starts the sequence in `slot` from the repository at `origin_root`; any
/// refusal is on the status line.
pub(crate) fn start(slot: u16, origin_root: &Path, last_message: &mut Option<String>) {
    match start_slot(slot, origin_root) {
        Ok(()) => {}
        Err(why) => *last_message = Some(why),
    }
}

fn start_slot(slot: u16, origin_root: &Path) -> Result<(), String> {
    let info = registry::info(slot).ok_or_else(|| format!("#{slot}"))?;
    let def = load_def(info.slug)
        .ok_or_else(|| gitpoulpi_i18n::msg::sequence_run::def_gone(&info.name))?;
    begin(Run::new(&def, origin_root, Instant::now(), exec::resolve))
}

/// The definition of `slug` among the sequences this installation sees.
fn load_def(slug: &str) -> Option<gitpoulpi_config::sequences::SequenceDef> {
    let paths = gitpoulpi_config::ConfigPaths::discover()?;
    let config = gitpoulpi_config::Config::load(&paths.config_file()).ok()?;
    let install = gitpoulpi_config::installation::current().identity.id.clone();
    gitpoulpi_config::sequences::visible_sequences(&config, &install, None)
        .into_iter()
        .map(|(d, _)| d)
        .find(|d| d.id == slug)
}
