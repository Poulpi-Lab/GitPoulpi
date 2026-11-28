//! `Jobs: Cancel <label>` rows: one palette command per background job that
//! can be canceled right now, built from the jobs cell beside the fixed
//! `Jobs: ...` catalogue entries.
//!
//! The rows are `&'static PaletteEntry` like the catalogue's, so their text
//! is leaked: once per (language epoch, set of cancellable jobs), which a
//! long session keeps small (a job starting or ending is the only change).

use crate::jobs_ui::JobsCmd;
use gitpoulpi_app::palette::{PaletteCommand, PaletteEntry, Surface};
use gitpoulpi_i18n::msg::jobs as jobs_msg;
use std::sync::Mutex;

type Key = (u64, u8, bool, Vec<(u64, String)>);

static MODEL: Mutex<Option<(Key, &'static [PaletteEntry])>> = Mutex::new(None);

fn jobs() -> Vec<(u64, String)> {
    crate::jobs_ui::cancellable()
        .into_iter()
        .map(|j| (j.id, crate::jobs_ui::label_of_id(j.id)))
        .collect()
}

fn leak(text: String) -> &'static str {
    Box::leak(text.into_boxed_str())
}

/// What the rows depend on besides the language, for the haystack cache.
pub(crate) fn signature() -> u64 {
    crate::jobs_ui::palette_signature()
}

/// The `Jobs: Cancel <label>` entries of the jobs going now.
pub(crate) fn entries() -> &'static [PaletteEntry] {
    let jobs = jobs();
    if jobs.is_empty() {
        return &[];
    }
    let key: Key = (
        gitpoulpi_i18n::epoch(),
        gitpoulpi_i18n::current() as u8,
        gitpoulpi_i18n::git_terms_translated(),
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
            label: jobs_msg::PALETTE_SHOW,
            shortcut: "",
            hint: None,
            command: PaletteCommand::Jobs(JobsCmd::Cancel(*id)),
            requires: None,
            text: Some(leak(jobs_msg::palette_cancel(label))),
            text_en: Some(leak(gitpoulpi_i18n::with_lang(
                gitpoulpi_i18n::Lang::En,
                || jobs_msg::palette_cancel(label),
            ))),
            surface: Surface::Any,
        })
        .collect();
    let built: &'static [PaletteEntry] = Box::leak(built.into_boxed_slice());
    *slot = Some((key, built));
    built
}
