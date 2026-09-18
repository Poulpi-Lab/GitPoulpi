//! `ai_page` for a build without the `ai` feature (custom builds, see
//! the custom-builds design §7): the Settings pages "AI" and
//! "Repo-Specific AI" are not offered (`PrefSection::requires`), so every
//! hook the rest of Preferences calls is a no-op here and no AI row exists.
//! `first_line_preview` is shared with the multi-line rows of other pages
//! and stays real.
#![allow(dead_code)]

use super::*;
use gitpoulpi_i18n::msg::ai_page as msg;

/// Which piece of the AI page a row edits (never built here).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum AiTarget {
    ProfileName,
    ProfileKey,
    ProfileEnv(usize),
    RepoMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum AiButton {
    Test,
}

#[derive(Default)]
pub(super) struct AiPage {
    pub(super) chip: usize,
}

impl AiPage {
    pub(super) fn selected_index(&self) -> usize {
        0
    }

    pub(super) fn select_index(&mut self, _index: usize) {}

    pub(super) fn prompt_open(&self) -> bool {
        false
    }

    pub(super) fn browse_open(&self) -> bool {
        false
    }

    pub(super) fn chip_focus(&self, len: usize) -> usize {
        self.chip.min(len.saturating_sub(1))
    }
}

pub(super) fn first_line_preview(text: &str) -> String {
    let mut lines = text.lines();
    let first = lines.next().unwrap_or("").trim().to_string();
    let more = lines.count();
    let first = if crate::ui_kit::display_width(&first) > 48 {
        format!("{}...", crate::ui_kit::take_width(&first, 48))
    } else {
        first
    };
    if more > 0 {
        format!("{first}  {}", msg::preview_more(more as u64))
    } else if first.is_empty() {
        msg::PREVIEW_EMPTY.get().to_string()
    } else {
        first
    }
}

impl PreferencesState {
    pub(super) fn ai_probe_keys(&mut self) {}

    pub(super) fn ai_commit_choice(&mut self, _target: AiTarget, _next: usize) {}

    pub(super) fn ai_commit_combo(&mut self, _target: AiTarget, _value: &str) {}

    pub(super) fn ai_commit_bool(&mut self, _target: AiTarget, _value: bool) {}

    pub(super) fn ai_commit_text(&mut self, _target: AiTarget, _draft: &str) {}

    pub(super) fn ai_activate(&mut self, _target: AiTarget) -> bool {
        false
    }

    pub(crate) fn open_ai_profile_list(&mut self) {}

    pub(crate) fn open_prompt_variables(&mut self) {}

    pub(crate) fn ai_open_default_agent(&mut self) {}

    pub(super) fn ai_list_spec(&self) -> list_editor::ListSpec {
        list_editor::ListSpec::default()
    }

    pub(super) fn ai_profile_op(&mut self, _op: list_editor::ListOp) {}

    pub(super) fn ai_run_button(&mut self, _button: AiButton) {}

    pub(super) fn poll_ai(&mut self) {}

    pub(super) fn ai_prompt_key(&mut self, _key: KeyEvent) -> bool {
        false
    }

    pub(super) fn ai_prompt_mouse(
        &mut self,
        _kind: MouseEventKind,
        _col: u16,
        _row: u16,
        _shift: bool,
    ) {
    }

    pub(super) fn ai_prompt_paste(&mut self, _text: &str) {}

    pub(super) fn review_browse_key(&mut self, _key: KeyEvent) {}

    pub(super) fn review_browse_mouse(&mut self, _kind: MouseEventKind, _col: u16, _row: u16) {}

    pub(super) fn review_browse_paste(&mut self, _text: &str) {}

    pub(super) fn render_review_browse(&mut self, _frame: &mut Frame, _area: Rect) {}

    pub(super) fn render_ai_prompt(&mut self, _frame: &mut Frame, _area: Rect) {}

    pub(in crate::preferences) fn ai_general_fields(&self) -> Vec<Field> {
        Vec::new()
    }

    pub(in crate::preferences) fn ai_item_fields(&self) -> Vec<Field> {
        Vec::new()
    }

    pub(in crate::preferences) fn repo_ai_fields(&self) -> Vec<Field> {
        Vec::new()
    }
}
