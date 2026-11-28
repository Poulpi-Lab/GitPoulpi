//! The palette's generated rows and process facts for this area: `Jobs: Cancel <job>` (one per job that
//! can be cancelled now) and the availability of the "last error" commands. The palette's families are
//! plain function pointers, so what they read lives in `daemon::activity`'s process-wide facts, which the
//! link host keeps current.

use std::sync::Mutex;

use gitpoulpi_app::palette::vocab::jobs_ui::JobsCmd;
use gitpoulpi_app::palette::{install_families, Families, PaletteCommand, PaletteEntry, Surface};
use gitpoulpi_i18n::msg::jobs as m;

use super::jobs;
use crate::daemon::activity::{cancellable_jobs, error_present};

type Key = (u64, u8, Vec<(u64, String)>);

static MODEL: Mutex<Option<(Key, &'static [PaletteEntry])>> = Mutex::new(None);

fn leak(text: String) -> &'static str {
    Box::leak(text.into_boxed_str())
}

fn labels() -> Vec<(u64, String)> {
    cancellable_jobs()
        .into_iter()
        .map(|j| (j.id, jobs::label(&j)))
        .collect()
}

/// A hash of what the per-job rows depend on, so the palette's cache rebuilds when a job starts or ends.
fn signature() -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    for job in cancellable_jobs() {
        job.id.hash(&mut h);
    }
    h.finish()
}

/// The `Jobs: Cancel <label>` entries of the jobs going now. The rows are `&'static` like the
/// catalogue's, so their text is leaked: once per (language, set of cancellable jobs).
fn entries() -> &'static [PaletteEntry] {
    let jobs = labels();
    if jobs.is_empty() {
        return &[];
    }
    let key: Key = (
        gitpoulpi_i18n::epoch(),
        gitpoulpi_i18n::current() as u8,
        jobs.clone(),
    );
    let mut slot = MODEL.lock().unwrap_or_else(|e| e.into_inner());
    if let Some((k, built)) = slot.as_ref() {
        if *k == key {
            return built;
        }
    }
    let built: Vec<PaletteEntry> = jobs
        .iter()
        .map(|(id, label)| PaletteEntry {
            label: m::PALETTE_SHOW,
            shortcut: "",
            hint: None,
            command: PaletteCommand::Jobs(JobsCmd::Cancel(*id)),
            requires: None,
            text: Some(leak(m::palette_cancel(label))),
            text_en: Some(leak(gitpoulpi_i18n::with_lang(
                gitpoulpi_i18n::Lang::En,
                || m::palette_cancel(label),
            ))),
            surface: Surface::Any,
        })
        .collect();
    let built: &'static [PaletteEntry] = Box::leak(built.into_boxed_slice());
    *slot = Some((key, built));
    built
}

/// Publishes this window's side of the palette (the first call wins).
pub fn install() {
    install_families(Families {
        jobs_entries: entries,
        jobs_signature: signature,
        any_job_cancellable: || !cancellable_jobs().is_empty(),
        status_error_present: error_present,
        // A failed entry's suggested action (pull first, stash and retry) is not offered here yet.
        last_error_has_remedy: || false,
        ..Families::NONE
    });
}
