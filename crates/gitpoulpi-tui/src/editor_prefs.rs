//! The `[editor]` settings, published process-wide.
//!
//! Same shape as the other installed settings (`live_settings::minimap`):
//! the event loop installs them from `config.toml` at start and after the
//! Settings screen changes one, and the editor and the diff view read them
//! while drawing, neither having a config in hand.
//!
//! Where each one acts:
//! - `tab_size`: the editor (tab stops, outdent) and the diff views;
//! - `eol`: what the editor writes when it saves;
//! - `syntax_highlighting`: the editor;
//! - `line_numbers`: the editor's gutter and the diff views' number
//!   columns (blanked there, so the selection geometry stays put).
//!
//! The blame / File History view draws a tab as one space and has no line
//! numbers, so none of these reach it.

use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};

use gitpoulpi_config::{EditorConfig, EditorEol};

static TAB_SIZE: AtomicU8 = AtomicU8::new(gitpoulpi_config::DEFAULT_EDITOR_TAB_SIZE);
static EOL: AtomicU8 = AtomicU8::new(0);
static SYNTAX: AtomicBool = AtomicBool::new(true);
static LINE_NUMBERS: AtomicBool = AtomicBool::new(true);
static FOLLOW_SELECTION: AtomicBool = AtomicBool::new(true);

pub(crate) fn install(config: &EditorConfig) {
    TAB_SIZE.store(config.tab_size_clamped(), Ordering::Relaxed);
    EOL.store(
        match config.eol {
            EditorEol::Auto => 0,
            EditorEol::Lf => 1,
            EditorEol::Crlf => 2,
        },
        Ordering::Relaxed,
    );
    SYNTAX.store(config.syntax_highlighting, Ordering::Relaxed);
    LINE_NUMBERS.store(config.line_numbers, Ordering::Relaxed);
    FOLLOW_SELECTION.store(config.follow_selection, Ordering::Relaxed);
}

/// Columns between tab stops (1 to 8).
pub(crate) fn tab_size() -> usize {
    usize::from(TAB_SIZE.load(Ordering::Relaxed).max(1))
}

pub(crate) fn eol() -> EditorEol {
    #[cfg(test)]
    if let Some(forced) = EOL_FOR_TEST.with(std::cell::Cell::get) {
        return forced;
    }
    match EOL.load(Ordering::Relaxed) {
        1 => EditorEol::Lf,
        2 => EditorEol::Crlf,
        _ => EditorEol::Auto,
    }
}

pub(crate) fn syntax_highlighting() -> bool {
    SYNTAX.load(Ordering::Relaxed)
}

/// `editor.follow_selection`: selecting another file in the right pane
/// opens it in the open editor.
pub(crate) fn follow_selection() -> bool {
    FOLLOW_SELECTION.load(Ordering::Relaxed)
}

/// Flips `editor.follow_selection`, publishes it and writes it back to
/// `config.toml`. Returns the status-line message.
pub(crate) fn toggle_follow_selection(config_path: &Option<std::path::PathBuf>) -> String {
    let on = !follow_selection();
    FOLLOW_SELECTION.store(on, Ordering::Relaxed);
    if let Some(path) = config_path {
        let _ = gitpoulpi_config::set_in_file(
            path,
            "editor.follow_selection",
            if on { "true" } else { "false" },
        );
    }
    if on {
        gitpoulpi_i18n::msg::editor::FOLLOW_ON.get().to_string()
    } else {
        gitpoulpi_i18n::msg::editor::FOLLOW_OFF.get().to_string()
    }
}

pub(crate) fn line_numbers() -> bool {
    LINE_NUMBERS.load(Ordering::Relaxed)
}

#[cfg(test)]
thread_local! {
    static EOL_FOR_TEST: std::cell::Cell<Option<EditorEol>> = const { std::cell::Cell::new(None) };
}

/// Runs `f` with `editor.eol` set to `eol` on this thread only (tests run in
/// parallel; the process-wide setting is not theirs to change).
#[cfg(test)]
pub(crate) fn with_eol<R>(eol: EditorEol, f: impl FnOnce() -> R) -> R {
    let before = EOL_FOR_TEST.with(|c| c.replace(Some(eol)));
    let out = f();
    EOL_FOR_TEST.with(|c| c.set(before));
    out
}
