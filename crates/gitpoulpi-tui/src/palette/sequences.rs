//! `Sequence: <name>` rows: the user's own sequences as palette commands.
//!
//! Generated like the `Settings: ...` rows (`settings.rs`) from the registry
//! of the sequences this installation sees (`sequence_run::registry`), never
//! listed by hand and kept out of `PALETTE_COMMANDS`. The text is leaked
//! (a row is a `&'static PaletteEntry`), once per (language epoch, registry
//! generation): the generation moves only when a sequence is added, renamed,
//! removed or turns (in)valid, so a long session keeps the leak small.
//!
//! A sequence that `validate` rejects stays listed but grayed out, with the
//! reason; its key still resolves, and says the same when pressed.

use super::*;
use crate::sequence_run::registry;
use std::sync::Mutex;

/// What a `Sequence: ...` row does. The number is the keymap slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SeqOp {
    Run(u16),
}

impl SeqOp {
    pub(crate) fn slot(self) -> u16 {
        match self {
            SeqOp::Run(slot) => slot,
        }
    }
}

struct Model {
    entries: &'static [PaletteEntry],
    /// Per entry: why it cannot run, from `validate`.
    reasons: Vec<Option<&'static str>>,
}

type Key = (u64, u8, bool, u64);

static MODEL: Mutex<Option<(Key, &'static Model)>> = Mutex::new(None);

fn leak(text: String) -> &'static str {
    Box::leak(text.into_boxed_str())
}

fn model() -> &'static Model {
    let key: Key = (
        gitpoulpi_i18n::epoch(),
        gitpoulpi_i18n::current() as u8,
        gitpoulpi_i18n::git_terms_translated(),
        registry::generation(),
    );
    let mut slot = MODEL.lock().unwrap_or_else(|e| e.into_inner());
    if let Some((k, built)) = slot.as_ref() {
        if *k == key {
            return built;
        }
    }
    let infos = registry::visible();
    let entries: Vec<PaletteEntry> = infos
        .iter()
        .map(|info| PaletteEntry {
            label: gitpoulpi_i18n::msg::keymap_ctx::SEQUENCE,
            shortcut: "",
            hint: None,
            command: PaletteCommand::Sequence(SeqOp::Run(info.slot)),
            requires: None,
            text: Some(leak(msg::palette::sequence_run(&info.name))),
            text_en: Some(leak(gitpoulpi_i18n::with_lang(
                gitpoulpi_i18n::Lang::En,
                || msg::palette::sequence_run(&info.name),
            ))),
        })
        .collect();
    let reasons = infos.iter().map(|i| i.problem.clone().map(leak)).collect();
    let built: &'static Model = Box::leak(Box::new(Model {
        entries: Box::leak(entries.into_boxed_slice()),
        reasons,
    }));
    *slot = Some((key, built));
    built
}

/// Re-reads `config.toml` when it changed on disk, so a sequence added
/// outside the window (the CLI, an editor) shows up without a restart.
pub(crate) fn sync() {
    registry::reload_if_changed();
}

/// The `Sequence: <name>` entries, in slot order, in the current language.
pub(crate) fn entries() -> &'static [PaletteEntry] {
    model().entries
}

/// Why the sequence cannot run, if it cannot.
pub(crate) fn unavailable(op: SeqOp) -> Option<&'static str> {
    let model = model();
    let at = model
        .entries
        .iter()
        .position(|e| e.command == PaletteCommand::Sequence(op))?;
    model.reasons[at]
}
