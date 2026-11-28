//! The Script rule's editor (slice S10): a Lua editor with a live check, the
//! script's parameter rows, a file to load from or save to, and "Test on
//! selection" with its result table.
//!
//! The rule form (`rules_form.rs`) lists the rule's fields in the detail
//! pane, which is only a few rows high; Enter on the script opens this
//! overlay (a master-detail view with its own cross, like the blob diff
//! drill-in) on top of the workbench. What is typed goes into the rule at
//! every key, so the footer, the rules list and the preview's out-of-date
//! marker follow it.
//!
//! **Nothing here waits.** The check and the test are daemon requests
//! (`RewriteScriptCheck`, `RewriteScriptTest`) answered into a [`Reply`]
//! polled each frame. The check is debounced: it runs [`CHECK_DEBOUNCE`]
//! after the last edit, at most one in flight, and an answer for a text that
//! is no longer the one on screen is never shown ([`key_of`] identifies the
//! text). The test cannot be stopped inside the daemon (it is a plain
//! request, bounded by the script's own limits): Cancel forgets the answer.
//!
//! **Files** are repository-relative and read or written on the spot: a
//! script is a few kilobytes, and the path may not leave the repository.

use crate::text_edit::CaretKeys as _;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::path::{Component, Path};
use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;

use gitpoulpi_core::domain::rewrite::script::{
    parse_script_failure, ParamKind, ScriptCheck, ScriptTestRow,
};
use gitpoulpi_core::domain::rewrite::{
    script_hash, RewriteRefusal, Rule, RuleKind, ScriptLanguage, ScriptRef, ScriptSource,
    ScriptTrust,
};
use gitpoulpi_core::domain::{Oid, OperationKind};
use gitpoulpi_i18n::msg::rewrite_view as t;
use gitpoulpi_ipc::rewrite::{ScriptCheckArgs, ScriptTestArgs, TrustArgs};

use super::bars::BarId;
use super::render::{bad, bold, dim, plain, row_bg, wrap, RewriteHit};
use super::rules_form::script_header;
use super::{FieldId, FieldInput, FieldValue, Notice, RewriteView};
use crate::daemon_link::Reply;
use crate::dialog_kit::{self, button_width, ButtonState};
use crate::text_edit::{Caret, KeyEdit};
use crate::text_field::{edit_line, edit_lines, offset_at, EditLook};
use crate::{display_width, th};

/// How long after the last edit the check runs.
pub(crate) const CHECK_DEBOUNCE: Duration = Duration::from_millis(300);
/// How many of the newest commits a test uses when nothing is selected.
pub(crate) const TEST_SAMPLE: usize = 20;
/// Where "Save to file" proposes to write: next to the shared recipes, with
/// the extension of the script's language.
// i18n-english: a path, not interface text.
fn default_path(language: ScriptLanguage) -> String {
    let extension = match language {
        ScriptLanguage::Lua | ScriptLanguage::LuaExternal => "lua",
        ScriptLanguage::Python => "py",
        ScriptLanguage::Shell if cfg!(windows) => "ps1",
        ScriptLanguage::Shell => "sh",
    };
    format!(".gitpoulpi/rewrite/script.{extension}")
}

/// Whether this build has the embedded Lua runtime.
pub(crate) fn scripts_built() -> bool {
    #[cfg(test)]
    {
        SCRIPTS_BUILT.with(std::cell::Cell::get)
    }
    #[cfg(not(test))]
    {
        crate::build_features::REWRITE_SCRIPTS
    }
}

#[cfg(test)]
thread_local! {
    static SCRIPTS_BUILT: std::cell::Cell<bool> = const { std::cell::Cell::new(true) };
}

/// Pretends the build has (or lacks) the `rewrite-scripts` feature, for the
/// current test thread.
#[cfg(test)]
pub(crate) fn set_scripts_built(on: bool) {
    SCRIPTS_BUILT.with(|c| c.set(on));
}

// ------------------------------------------------------------- the check

/// What the live check last learned.
pub(crate) enum CheckOutcome {
    Done(Box<ScriptCheck>),
    Failed(String),
}

/// The debounced, one-at-a-time check of the script under the rule cursor.
#[derive(Default)]
pub(crate) struct CheckState {
    /// The text key last seen.
    seen: Option<u64>,
    /// When the check for `seen` may be sent.
    due: Option<Instant>,
    pending: Option<(u64, Reply<ScriptCheck>)>,
    /// The latest answer and the text it is about.
    pub(crate) answer: Option<(u64, CheckOutcome)>,
}

impl CheckState {
    /// One look at the key of the script on screen (`None`: no script to
    /// check). `true` means "send the check for `key` now".
    pub(crate) fn step(&mut self, key: Option<u64>, now: Instant) -> bool {
        let Some(key) = key else {
            self.seen = None;
            self.due = None;
            return false;
        };
        if self.seen != Some(key) {
            self.seen = Some(key);
            let known = self.answer.as_ref().is_some_and(|(k, _)| *k == key)
                || self.pending.as_ref().is_some_and(|(k, _)| *k == key);
            self.due = (!known).then_some(now + CHECK_DEBOUNCE);
        }
        match self.due {
            Some(due) if due <= now && self.pending.is_none() => {
                self.due = None;
                true
            }
            _ => false,
        }
    }

    /// The request that was just sent, for `key`.
    pub(crate) fn sent(&mut self, key: u64, reply: Reply<ScriptCheck>) {
        self.pending = Some((key, reply));
    }

    /// Takes the answer of the request in flight, if it has come.
    pub(crate) fn take_answer(&mut self) {
        let Some((key, reply)) = &self.pending else {
            return;
        };
        let key = *key;
        let Some(answer) = reply.take() else {
            return;
        };
        self.pending = None;
        self.answer = Some((
            key,
            match answer {
                Ok(check) => CheckOutcome::Done(Box::new(check)),
                Err(error) => {
                    CheckOutcome::Failed(crate::daemon_link::error_text::error_text(&error))
                }
            },
        ));
    }

    /// When the loop should look in again: the debounce running out. A check
    /// in flight wakes the loop itself when it answers.
    pub(crate) fn next_deadline(&self) -> Option<Instant> {
        self.due.filter(|_| self.pending.is_none())
    }

    /// Whether a check is scheduled or in flight.
    pub(crate) fn busy(&self) -> bool {
        self.due.is_some() || self.pending.is_some()
    }
}

/// Identifies the text of a script (not its parameters): the check's answer
/// is only shown while this still matches.
pub(crate) fn key_of(script: &ScriptRef) -> u64 {
    let mut h = DefaultHasher::new();
    std::mem::discriminant(&script.language).hash(&mut h);
    match &script.source {
        ScriptSource::Inline { text } => (0u8, text).hash(&mut h),
        ScriptSource::File { path } => (1u8, path).hash(&mut h),
    }
    h.finish()
}

/// What the check line says.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CheckView {
    /// The build has no scripts.
    NotBuilt,
    /// Nothing typed yet.
    Empty,
    Checking,
    /// Compiles; the entry point found.
    Ok(String),
    /// Compiles but defines no `rewrite_commit`.
    NoFunction,
    /// An external script that nobody trusted yet (it was not started).
    Untrusted,
    /// An external script that answered the dry handshake.
    ExternalOk,
    /// `script:LINE: ...` problems.
    Problems(Vec<String>),
    Failed(String),
}

/// The line numbers a list of `script:LINE: message` problems is about.
pub(crate) fn problem_lines(problems: &[String]) -> HashSet<usize> {
    problems
        .iter()
        .filter_map(|p| {
            // `syntax error: script:3: ...` or `script:3: ...`.
            let rest = &p[p.find("script:")? + "script:".len()..];
            let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
            digits.parse().ok()
        })
        .collect()
}

// -------------------------------------------------------------- the test

/// A test of the script on some commits.
#[derive(Default)]
pub(crate) enum TestState {
    #[default]
    Idle,
    Running {
        reply: Reply<Vec<ScriptTestRow>>,
        since: Instant,
        count: usize,
    },
    Done {
        rows: Vec<ScriptTestRow>,
        key: u64,
        count: usize,
    },
    /// The script failed on `commit` (`None`: it did not compile).
    Failed {
        commit: Option<Oid>,
        reason: String,
    },
    Canceled,
}

/// One line of the result table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResultLine {
    /// The short id, on the first line of each commit.
    pub commit: Option<String>,
    pub text: String,
    pub kind: ResultKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ResultKind {
    Change,
    Unchanged,
    Log,
}

/// The table: a line per changed field (`Author name: alex -> ALEX`), the
/// script's `log` lines under their commit, and `no change` for a commit the
/// script left as it was.
pub(crate) fn result_lines(rows: &[ScriptTestRow]) -> Vec<ResultLine> {
    let mut out = Vec::new();
    for row in rows {
        let short = super::preview::short(&row.commit);
        let mut first = Some(short);
        for change in &row.fields {
            out.push(ResultLine {
                commit: first.take(),
                text: format!(
                    "{}: {} -> {}",
                    super::preview::field_label(&change.field).get(),
                    one_line(&change.old),
                    one_line(&change.new)
                ),
                kind: ResultKind::Change,
            });
        }
        if row.fields.is_empty() {
            out.push(ResultLine {
                commit: first.take(),
                text: t::SCRIPT_NO_CHANGE.get().to_string(),
                kind: ResultKind::Unchanged,
            });
        }
        for line in &row.log {
            out.push(ResultLine {
                commit: first.take(),
                text: t::script_log_line(one_line(line)),
                kind: ResultKind::Log,
            });
        }
    }
    out
}

fn one_line(text: &str) -> String {
    text.trim_end().replace(['\n', '\r'], " ")
}

/// A refused script as the panel and the table word it: the commit it failed
/// on (short id) when there is one, and the reason.
pub(crate) fn failure_lines(detail: &str) -> (Option<Oid>, Vec<String>) {
    let (commit, reason) = parse_script_failure(detail);
    let mut lines = Vec::new();
    match commit {
        Some(oid) => lines.push(format!(
            "{}: {}",
            super::preview::short(&oid),
            reason.lines().next().unwrap_or("")
        )),
        None => lines.extend(reason.lines().map(str::to_string)),
    }
    (commit, lines)
}

// ------------------------------------------------------------ the editor

/// Where the keyboard is in the editor overlay.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ScriptFocus {
    Editor,
    /// Parameter row `.0` of the script's header.
    Param(usize),
    Path,
    Load,
    Save,
    /// The ✦ Write with AI button (only offered with AI on).
    Ai,
    Test,
    Results,
}

/// The open editor overlay.
pub(crate) struct ScriptEditor {
    pub focus: ScriptFocus,
    pub caret: Caret,
    /// The repository-relative file to load from or save to.
    pub path: String,
    pub path_caret: Caret,
    /// What loading or saving or a refused test said last.
    pub note: Option<Notice>,
    /// The result table's cursor and first line.
    pub cursor: usize,
    pub top: usize,
}

/// Why a script language cannot be picked.
enum LanguageBlock {
    /// `rewrite.external_scripts` does not list it.
    Off,
    /// Its interpreter was not found.
    Missing { tool: &'static str, hint: String },
}

/// All the Script rule's state besides the rule itself.
#[derive(Default)]
pub(crate) struct ScriptState {
    pub check: CheckState,
    pub test: TestState,
    pub editor: Option<ScriptEditor>,
    /// Content hashes of the scripts the user typed in this window (the last
    /// 64): vouched for on the next request, so a script written here runs
    /// without the trust question, while one that came with a recipe or a
    /// file of the repository does not.
    pub authored: Vec<String>,
    /// The trust request in flight, and what to do when it is granted.
    pub trusting: Option<(Reply<serde_json::Value>, AfterTrust)>,
    /// The ✦ dialog and the mark on the script the AI wrote (`script_ai.rs`).
    pub ai: super::script_ai::AiState,
}

/// What the question "trust this script?" goes on to do once answered yes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AfterTrust {
    Preview,
    Test,
}

impl ScriptState {
    /// The check line for the script whose key is `key`.
    pub(crate) fn check_view(&self, key: Option<u64>, empty: bool, external: bool) -> CheckView {
        if !scripts_built() {
            return CheckView::NotBuilt;
        }
        if empty {
            return CheckView::Empty;
        }
        match (&self.check.answer, key) {
            (Some((k, outcome)), Some(key)) if *k == key => match outcome {
                CheckOutcome::Failed(why) => CheckView::Failed(why.clone()),
                CheckOutcome::Done(check) if !check.problems.is_empty() => {
                    CheckView::Problems(check.problems.clone())
                }
                CheckOutcome::Done(check) if external => match check.trust {
                    ScriptTrust::Untrusted => CheckView::Untrusted,
                    _ => CheckView::ExternalOk,
                },
                CheckOutcome::Done(check) => {
                    match check
                        .functions
                        .iter()
                        .find(|f| f.as_str() == "rewrite_commit")
                    {
                        Some(f) => CheckView::Ok(f.clone()),
                        None => CheckView::NoFunction,
                    }
                }
            },
            _ => CheckView::Checking,
        }
    }
}

impl RewriteView {
    // ---------------------------------------------------------- queries

    /// The script of the rule under the cursor.
    pub(crate) fn script_ref(&self) -> Option<&ScriptRef> {
        match &self.recipe.rules.get(self.rules.cursor)?.kind {
            RuleKind::Script(s) => Some(s),
            _ => None,
        }
    }

    pub(super) fn script_text(&self) -> Option<&str> {
        match &self.script_ref()?.source {
            ScriptSource::Inline { text } => Some(text),
            ScriptSource::File { .. } => None,
        }
    }

    fn script_blank(&self) -> bool {
        match self.script_ref().map(|s| &s.source) {
            Some(ScriptSource::Inline { text }) => text.trim().is_empty(),
            Some(ScriptSource::File { path }) => path.trim().is_empty(),
            None => true,
        }
    }

    pub(crate) fn script_key(&self) -> Option<u64> {
        self.script_ref().map(key_of)
    }

    /// What the check line says for the script under the cursor.
    pub(crate) fn check_view(&self) -> CheckView {
        self.script.check_view(
            self.script_key(),
            self.script_blank(),
            self.script_ref().is_some_and(|s| s.language.is_external()),
        )
    }

    /// Whether the rule under the cursor is a script whose line the detail
    /// pane should show.
    pub(crate) fn shows_check(&self) -> bool {
        self.script_ref().is_some()
    }

    /// The reason a test cannot start now, if any.
    pub(crate) fn test_blocker(&self) -> Option<String> {
        if !scripts_built() {
            return Some(t::SCRIPT_NOT_BUILT.get().to_string());
        }
        if matches!(self.script.test, TestState::Running { .. }) {
            return Some(t::B_BUSY.get().to_string());
        }
        if self.script_blank() {
            return Some(t::SCRIPT_TEST_FIX_FIRST.get().to_string());
        }
        if matches!(
            self.check_view(),
            CheckView::Problems(_) | CheckView::Failed(_)
        ) {
            return Some(t::SCRIPT_TEST_FIX_FIRST.get().to_string());
        }
        if let Some(s) = self.script_ref() {
            if let Some(header) = script_header(s, self.detail.file_header.as_ref()) {
                let missing = header.missing_params(&s.params);
                if !missing.is_empty() {
                    return Some(t::script_test_params_first(missing.join(", ")));
                }
            }
        }
        if self.scope_ctx.test_commits.is_empty() {
            return Some(t::SCRIPT_TEST_NONE.get().to_string());
        }
        None
    }

    /// What the test would run on, in words.
    pub(crate) fn test_target(&self) -> String {
        let n = self.scope_ctx.test_commits.len() as u64;
        if self.scope_ctx.test_from_selection {
            t::script_test_selected(n)
        } else {
            t::script_test_newest(n)
        }
    }

    /// The parameter fields of the detail pane: `(position in the pane,
    /// index in the header)`.
    pub(crate) fn param_fields(&self) -> Vec<(usize, usize)> {
        self.detail
            .fields
            .iter()
            .enumerate()
            .filter_map(|(at, f)| match f.id {
                FieldId::ScriptParam(i) => Some((at, i)),
                _ => None,
            })
            .collect()
    }

    /// The result table lines of the last test.
    pub(crate) fn test_lines(&self) -> Vec<ResultLine> {
        match &self.script.test {
            TestState::Done { rows, .. } => result_lines(rows),
            _ => Vec::new(),
        }
    }

    // ------------------------------------------------------------ polling

    /// Takes the answers that came and sends the check that is due.
    pub(super) fn poll_script(&mut self, now: Instant) {
        self.script.check.take_answer();
        let key = if scripts_built() && !self.script_blank() {
            self.script_key()
        } else {
            None
        };
        if self.script.check.step(key, now) {
            if let (Some(key), Some(script)) = (key, self.script_ref().cloned()) {
                let reply = self.requester.call(
                    OperationKind::RewriteScriptCheck,
                    serde_json::to_value(ScriptCheckArgs {
                        script,
                        vouched: self.vouched_hashes(),
                    })
                    .unwrap_or_default(),
                );
                self.script.check.sent(key, reply);
            }
        }
        self.sync_file_header();
        self.poll_test();
        self.poll_trusting();
    }

    /// The header of a script read from a file comes from the check.
    fn sync_file_header(&mut self) {
        let want = match (
            self.script_ref(),
            &self.script.check.answer,
            self.script_key(),
        ) {
            (Some(s), Some((k, CheckOutcome::Done(check))), Some(key))
                if matches!(s.source, ScriptSource::File { .. }) && *k == key =>
            {
                Some(check.header.clone())
            }
            _ => None,
        };
        if want != self.detail.file_header {
            self.detail.file_header = want;
            if let Some(rule) = self.recipe.rules.get(self.rules.cursor) {
                self.detail.rebuild(rule);
            }
        }
    }

    fn poll_test(&mut self) {
        let answer = match &self.script.test {
            TestState::Running { reply, count, .. } => reply.take().map(|a| (a, *count)),
            _ => None,
        };
        let Some((answer, count)) = answer else {
            return;
        };
        self.script.test = match answer {
            Ok(rows) => TestState::Done {
                rows,
                key: self.script_key().unwrap_or(0),
                count,
            },
            Err(error) => {
                use gitpoulpi_core::TypedError as T;
                match error.typed() {
                    Some(T::RewriteRefused(RewriteRefusal::ScriptFailed, detail)) => {
                        let (commit, reason) = parse_script_failure(&detail);
                        TestState::Failed { commit, reason }
                    }
                    Some(T::RewriteRefused(RewriteRefusal::ScriptNotTrusted, detail)) => {
                        self.ask_trust(detail, AfterTrust::Test);
                        TestState::Idle
                    }
                    _ => TestState::Failed {
                        commit: None,
                        reason: crate::daemon_link::error_text::error_text(&error),
                    },
                }
            }
        };
        if let Some(ed) = &mut self.script.editor {
            ed.cursor = 0;
            ed.top = 0;
        }
    }

    /// When the loop should look in again (the check's debounce).
    pub(crate) fn next_deadline(&self) -> Option<Instant> {
        self.script.check.next_deadline()
    }

    // ---------------------------------------------------- open and close

    /// Opens the editor on the script of the rule under the cursor. A file
    /// source has no text to edit here; a build without scripts says so.
    pub(crate) fn open_script_editor(&mut self) {
        if !scripts_built() {
            self.notice = Some(Notice {
                text: t::SCRIPT_NOT_BUILT.get().to_string(),
                error: true,
            });
            return;
        }
        if self.script_text().is_none() {
            return;
        }
        self.script.editor = Some(ScriptEditor {
            focus: ScriptFocus::Editor,
            caret: Caret::default(),
            path: default_path(
                self.script_ref()
                    .map_or(ScriptLanguage::Lua, |s| s.language),
            ),
            path_caret: Caret::default(),
            note: None,
            cursor: 0,
            top: 0,
        });
    }

    pub(crate) fn close_script_editor(&mut self) {
        if self.editing.is_some() {
            self.stop_editing(true);
        }
        self.script.editor = None;
    }

    // ------------------------------------------------------------- focus

    /// The foci in Tab order for the script on screen.
    fn script_foci(&self) -> Vec<ScriptFocus> {
        let mut order = vec![ScriptFocus::Editor];
        order.extend(
            self.param_fields()
                .into_iter()
                .map(|(_, i)| ScriptFocus::Param(i)),
        );
        order.extend([ScriptFocus::Path, ScriptFocus::Load, ScriptFocus::Save]);
        if self.ai_offered() {
            order.push(ScriptFocus::Ai);
        }
        order.push(ScriptFocus::Test);
        if !self.test_lines().is_empty() {
            order.push(ScriptFocus::Results);
        }
        order
    }

    pub(crate) fn script_focus(&mut self, to: ScriptFocus) {
        if self.script.editor.is_none() {
            return;
        }
        if self.editing.is_some() {
            self.stop_editing(true);
        }
        if let Some(ed) = &mut self.script.editor {
            ed.focus = to;
        }
        // A text parameter takes the keys at once, like the other fields.
        if let ScriptFocus::Param(i) = to {
            if let Some((at, _)) = self.param_fields().into_iter().find(|(_, p)| *p == i) {
                if matches!(self.detail.fields[at].value, FieldValue::Text(_)) {
                    self.activate_field(at);
                }
            }
        }
    }

    fn script_step_focus(&mut self, delta: isize) {
        let Some(ed) = &self.script.editor else {
            return;
        };
        let order = self.script_foci();
        let at = order.iter().position(|f| *f == ed.focus).unwrap_or(0) as isize;
        let next = order[(at + delta).rem_euclid(order.len() as isize) as usize];
        self.script_focus(next);
    }

    // -------------------------------------------------------------- keys

    /// Keys while the editor overlay is open.
    pub(super) fn on_script_key(&mut self, key: KeyEvent) {
        let Some(focus) = self.script.editor.as_ref().map(|e| e.focus) else {
            return;
        };
        let plain_key = !key
            .modifiers
            .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT);
        match key.code {
            KeyCode::F(5) => return self.start_script_test(),
            KeyCode::Tab => return self.script_step_focus(1),
            KeyCode::BackTab => return self.script_step_focus(-1),
            KeyCode::Esc if self.editing.is_none() => return self.close_script_editor(),
            _ => {}
        }
        match focus {
            ScriptFocus::Editor => self.on_script_text_key(key),
            ScriptFocus::Param(i) => self.on_script_param_key(key, i),
            ScriptFocus::Path => match key.code {
                KeyCode::Enter => self.script_step_focus(1),
                KeyCode::Esc => self.close_script_editor(),
                _ => self.on_script_path_key(key),
            },
            ScriptFocus::Load | ScriptFocus::Save | ScriptFocus::Ai | ScriptFocus::Test => {
                match key.code {
                    KeyCode::Enter | KeyCode::Char(' ') if plain_key => match focus {
                        ScriptFocus::Load => self.load_script_file(),
                        ScriptFocus::Save => self.save_script_file(),
                        ScriptFocus::Ai => self.open_ai_dialog(),
                        _ => self.start_script_test(),
                    },
                    KeyCode::Right | KeyCode::Down if plain_key => self.script_step_focus(1),
                    KeyCode::Left | KeyCode::Up if plain_key => self.script_step_focus(-1),
                    _ => {}
                }
            }
            ScriptFocus::Results => {
                let last = self.test_lines().len().saturating_sub(1) as isize;
                let page = self.paint.script_rows.get().max(2) as isize - 1;
                let step = match key.code {
                    KeyCode::Up | KeyCode::Char('k') => -1,
                    KeyCode::Down | KeyCode::Char('j') => 1,
                    KeyCode::PageUp => -page,
                    KeyCode::PageDown => page,
                    KeyCode::Home => isize::MIN / 2,
                    KeyCode::End => isize::MAX / 2,
                    _ => return,
                };
                self.script_results_cursor(step, last);
            }
        }
    }

    fn script_results_cursor(&mut self, delta: isize, last: isize) {
        if let Some(ed) = &mut self.script.editor {
            ed.cursor = (ed.cursor as isize + delta).clamp(0, last.max(0)) as usize;
        }
    }

    /// Scrolls the result table by `delta` lines (the wheel).
    pub(crate) fn script_scroll(&mut self, delta: isize) {
        let total = self.test_lines().len();
        let room = self.paint.script_rows.get().max(1);
        let max = total.saturating_sub(room) as isize;
        if let Some(ed) = &mut self.script.editor {
            ed.top = (ed.top as isize + delta).clamp(0, max) as usize;
        }
    }

    fn on_script_text_key(&mut self, key: KeyEvent) {
        let Some(mut text) = self.script_text().map(str::to_string) else {
            return;
        };
        let Some(ed) = self.script.editor.as_mut() else {
            return;
        };
        // Enter is the caller's in a multi-line field: a new line.
        if key.code == KeyCode::Enter && !key.modifiers.contains(KeyModifiers::CONTROL) {
            ed.caret.insert(&mut text, "\n");
            self.set_script_text(text);
            return self.note_authored();
        }
        match ed.caret.on_key(&mut text, &key, true) {
            KeyEdit::Edited => {
                self.set_script_text(text);
                self.note_authored();
            }
            KeyEdit::Copy(s) => {
                let msg = crate::clipboard::copy_shared(t::SCRIPT_EDITOR_LABEL.get(), &s);
                ed.note = Some(Notice {
                    text: msg,
                    error: false,
                });
            }
            KeyEdit::Cut(s) => {
                let msg = crate::clipboard::copy_shared(t::SCRIPT_EDITOR_LABEL.get(), &s);
                ed.note = Some(Notice {
                    text: msg,
                    error: false,
                });
                self.set_script_text(text);
                self.note_authored();
            }
            KeyEdit::Moved | KeyEdit::Ignored => {}
        }
    }

    /// Puts `text` into the rule's script.
    pub(super) fn set_script_text(&mut self, text: String) {
        if let Some(at) = self
            .detail
            .fields
            .iter()
            .position(|f| f.id == FieldId::ScriptText)
        {
            self.edit_field(at, FieldInput::Text(text));
        }
    }

    /// Pasted text goes into whatever has the keyboard in the overlay.
    pub(crate) fn paste_script(&mut self, pasted: &str) {
        let Some(focus) = self.script.editor.as_ref().map(|e| e.focus) else {
            return;
        };
        match focus {
            ScriptFocus::Editor => {
                let Some(mut text) = self.script_text().map(str::to_string) else {
                    return;
                };
                if let Some(ed) = self.script.editor.as_mut() {
                    ed.caret.insert(&mut text, pasted);
                }
                self.set_script_text(text);
                self.note_authored();
            }
            ScriptFocus::Path => {
                if let Some(ed) = self.script.editor.as_mut() {
                    let one = pasted.replace(['\r', '\n'], " ");
                    ed.path_caret.insert(&mut ed.path, &one);
                }
            }
            _ => self.paste(pasted),
        }
    }

    fn on_script_param_key(&mut self, key: KeyEvent, i: usize) {
        let Some((at, _)) = self.param_fields().into_iter().find(|(_, p)| *p == i) else {
            return;
        };
        if self.editing.is_some() {
            if key.code == KeyCode::Enter {
                self.stop_editing(true);
                self.script_step_focus(1);
            } else {
                self.on_edit_key(key);
            }
            return;
        }
        match key.code {
            KeyCode::Enter | KeyCode::Char(' ') => self.step_field(at, 1),
            KeyCode::Down => self.script_step_focus(1),
            KeyCode::Up => self.script_step_focus(-1),
            _ => {}
        }
    }

    fn on_script_path_key(&mut self, key: KeyEvent) {
        let Some(ed) = self.script.editor.as_mut() else {
            return;
        };
        if let KeyEdit::Copy(s) | KeyEdit::Cut(s) = ed.path_caret.on_key(&mut ed.path, &key, false)
        {
            let msg = crate::clipboard::copy_shared(t::SCRIPT_FILE_LABEL.get(), &s);
            ed.note = Some(Notice {
                text: msg,
                error: false,
            });
        }
    }

    // ------------------------------------------------------------- files

    /// The file `path` inside the repository, or `None` for anything that
    /// could leave it.
    fn script_file(&self, path: &str) -> Option<std::path::PathBuf> {
        let rel = Path::new(path.trim());
        if path.trim().is_empty() || !rel.components().all(|c| matches!(c, Component::Normal(_))) {
            return None;
        }
        Some(self.store.repo_root.join(rel))
    }

    pub(super) fn script_note(&mut self, text: String, error: bool) {
        if let Some(ed) = &mut self.script.editor {
            ed.note = Some(Notice { text, error });
        }
    }

    /// "Load from file": the file's text replaces the script.
    pub(crate) fn load_script_file(&mut self) {
        let Some(path) = self
            .script
            .editor
            .as_ref()
            .map(|e| e.path.trim().to_string())
        else {
            return;
        };
        let Some(full) = self.script_file(&path) else {
            return self.script_note(t::SCRIPT_PATH_BAD.get().to_string(), true);
        };
        match std::fs::read_to_string(&full) {
            Ok(text) => {
                self.set_script_text(text);
                if let Some(ed) = &mut self.script.editor {
                    ed.caret = Caret::default();
                }
                self.script_note(t::script_load_ok(path), false);
            }
            Err(e) => self.script_note(t::script_file_failed(path, e), true),
        }
    }

    /// "Save to file": the script is written to the file (folders made).
    pub(crate) fn save_script_file(&mut self) {
        let Some(path) = self
            .script
            .editor
            .as_ref()
            .map(|e| e.path.trim().to_string())
        else {
            return;
        };
        let Some(full) = self.script_file(&path) else {
            return self.script_note(t::SCRIPT_PATH_BAD.get().to_string(), true);
        };
        let text = self.script_text().unwrap_or("").to_string();
        let written = full
            .parent()
            .map_or(Ok(()), std::fs::create_dir_all)
            .and_then(|()| std::fs::write(&full, text));
        match written {
            Ok(()) => self.script_note(t::script_save_ok(path), false),
            Err(e) => self.script_note(t::script_file_failed(path, e), true),
        }
    }

    // ------------------------------------------------- languages and trust

    /// Why `language` cannot be used now, in the words of its row: the
    /// setting is off, or its interpreter is not installed. `None` while the
    /// tool check has not answered (nothing is greyed on a guess).
    fn language_block(&self, language: ScriptLanguage) -> Option<LanguageBlock> {
        if !language.is_external() {
            return None;
        }
        let tools = self.tools.tools()?;
        if tools
            .external_scripts
            .is_some_and(|switches| !switches.allows(language))
        {
            return Some(LanguageBlock::Off);
        }
        let probe = Rule::new(RuleKind::Script(ScriptRef {
            language,
            source: ScriptSource::Inline {
                text: String::new(),
            },
            params: Default::default(),
        }));
        let missing = super::tools::missing_for_rule(&self.tools, &probe);
        let first = missing.first()?;
        Some(LanguageBlock::Missing {
            tool: first.tool.name(),
            hint: first.hint.get().to_string(),
        })
    }

    /// The short reason beside row `index` of the language dropdown.
    pub(super) fn language_row_blocker(&self, index: usize) -> Option<gitpoulpi_i18n::M> {
        let (language, _) = super::rules_form::LANGUAGE.get(index)?;
        match self.language_block(*language)? {
            LanguageBlock::Off => Some(t::SCRIPT_LANG_OFF),
            LanguageBlock::Missing { .. } => Some(t::SCRIPT_LANG_MISSING),
        }
    }

    /// A pick of a language that cannot be used is refused with the whole
    /// reason (and the install hint) in the notice. `true`: refused.
    pub(super) fn language_refused(&mut self, at: usize, input: &FieldInput) -> bool {
        let (Some(FieldId::ScriptLanguage), FieldInput::Choice(index)) =
            (self.detail.fields.get(at).map(|f| f.id), input)
        else {
            return false;
        };
        let Some((language, label)) = super::rules_form::LANGUAGE.get(*index).copied() else {
            return false;
        };
        let text = match self.language_block(language) {
            None => return false,
            Some(LanguageBlock::Off) => {
                t::script_lang_off_why(label.get(), language.setting_name().unwrap_or(""))
            }
            Some(LanguageBlock::Missing { tool, hint }) => t::script_needs_tool(tool, hint),
        };
        self.notice = Some(Notice { text, error: true });
        true
    }

    /// Remembers that the script under the cursor was typed here.
    fn note_authored(&mut self) {
        let Some(text) = self.script_text() else {
            return;
        };
        let hash = script_hash(text);
        let authored = &mut self.script.authored;
        if !authored.contains(&hash) {
            authored.push(hash);
            let extra = authored.len().saturating_sub(64);
            authored.drain(..extra);
        }
    }

    /// The hashes of the recipe's external inline scripts that were typed in
    /// this window: sent with the requests, so they run without the trust
    /// question. Never a script that only came with a recipe or a file.
    pub(crate) fn vouched_hashes(&self) -> Vec<String> {
        self.recipe
            .rules
            .iter()
            .filter_map(|rule| match &rule.kind {
                RuleKind::Script(s) if s.language.is_external() => match &s.source {
                    ScriptSource::Inline { text } => Some(script_hash(text)),
                    ScriptSource::File { .. } => None,
                },
                _ => None,
            })
            .filter(|h| self.script.authored.contains(h))
            .collect()
    }

    /// The daemon refused a script nobody trusted: ask. No answer is
    /// preselected.
    pub(super) fn ask_trust(&mut self, hash: String, then: AfterTrust) {
        self.question = Some(super::Question::TrustScript {
            hash,
            then,
            cursor: None,
        });
    }

    /// "Trust it": the daemon keeps the hash for this repository; the action
    /// that was refused is repeated when the daemon answered.
    pub(super) fn trust_script(&mut self, hash: String, then: AfterTrust) {
        let reply = self.requester.call(
            OperationKind::RewriteTrustScript,
            serde_json::to_value(TrustArgs {
                hash,
                label: "workbench".to_string(),
            })
            .unwrap_or_default(),
        );
        self.script.trusting = Some((reply, then));
    }

    fn poll_trusting(&mut self) {
        let Some((reply, then)) = &self.script.trusting else {
            return;
        };
        let then = *then;
        let Some(answer) = reply.take() else {
            return;
        };
        self.script.trusting = None;
        match answer {
            Ok(_) => {
                // The check line was about an untrusted script.
                self.script.check.answer = None;
                self.script.check.seen = None;
                match then {
                    AfterTrust::Preview => self.preview(),
                    AfterTrust::Test => self.start_script_test(),
                }
            }
            Err(error) => {
                self.notice = Some(Notice {
                    text: crate::daemon_link::error_text::error_text(&error),
                    error: true,
                });
            }
        }
    }

    // -------------------------------------------------------------- test

    /// "Test on selection": runs the script on the Log's selected commits,
    /// else the newest ones.
    pub(crate) fn start_script_test(&mut self) {
        if let Some(why) = self.test_blocker() {
            return self.script_note(why, true);
        }
        let (Some(script), commits) = (
            self.script_ref().cloned(),
            self.scope_ctx.test_commits.clone(),
        ) else {
            return;
        };
        let count = commits.len();
        let reply = self.requester.call(
            OperationKind::RewriteScriptTest,
            serde_json::to_value(ScriptTestArgs {
                script,
                commits,
                vouched: self.vouched_hashes(),
            })
            .unwrap_or_default(),
        );
        self.script.test = TestState::Running {
            reply,
            since: Instant::now(),
            count,
        };
        if let Some(ed) = &mut self.script.editor {
            ed.note = None;
        }
    }

    /// A finished test with one change and one log line, for the layout test
    /// that draws the editor in every language.
    #[cfg(test)]
    pub(crate) fn script_test_result_for_layout(&mut self) {
        use gitpoulpi_core::domain::rewrite::{FieldChange, RewriteField};
        self.script.test = TestState::Done {
            rows: vec![ScriptTestRow {
                commit: Oid([0xab; 20]),
                fields: vec![FieldChange {
                    field: RewriteField::AuthorName,
                    old: "alex".into(),
                    new: "ALEX".into(),
                }],
                log: vec!["renamed".into()],
            }],
            key: self.script_key().unwrap_or(0),
            count: 2,
        };
    }

    /// Cancel: the answer is forgotten when it comes.
    pub(crate) fn cancel_script_test(&mut self) {
        if matches!(self.script.test, TestState::Running { .. }) {
            self.script.test = TestState::Canceled;
        }
    }

    // ------------------------------------------------------------- mouse

    /// A press on a part of the overlay: the same methods the keys use.
    pub(super) fn on_script_press(&mut self, hit: RewriteHit, col: u16, row: u16) {
        match hit {
            RewriteHit::ScriptClose | RewriteHit::ScriptShield => self.close_script_editor(),
            RewriteHit::ScriptPanel => {}
            RewriteHit::ScriptEditor => {
                self.script_focus(ScriptFocus::Editor);
                let area = self.paint.script_text.get();
                let Some(text) = self.script_text().map(str::to_string) else {
                    return;
                };
                if let Some(ed) = &mut self.script.editor {
                    let at = offset_at(
                        &text,
                        &ed.caret,
                        area.width as usize,
                        true,
                        col.saturating_sub(area.x) as usize,
                        row.saturating_sub(area.y) as usize,
                    );
                    ed.caret.set_cursor(&text, at, false);
                }
            }
            RewriteHit::ScriptParam(i) => {
                self.script_focus(ScriptFocus::Param(i));
                if let Some((at, _)) = self.param_fields().into_iter().find(|(_, p)| *p == i) {
                    if matches!(self.detail.fields[at].value, FieldValue::Toggle(_)) {
                        self.step_field(at, 1);
                    }
                }
            }
            RewriteHit::ScriptPath => {
                self.script_focus(ScriptFocus::Path);
                let area = self.paint.script_path.get();
                if let Some(ed) = &mut self.script.editor {
                    let at = offset_at(
                        &ed.path,
                        &ed.path_caret,
                        area.width as usize,
                        false,
                        col.saturating_sub(area.x) as usize,
                        0,
                    );
                    let path = ed.path.clone();
                    ed.path_caret.set_cursor(&path, at, false);
                }
            }
            RewriteHit::ScriptLoad => {
                self.script_focus(ScriptFocus::Load);
                self.load_script_file();
            }
            RewriteHit::ScriptSave => {
                self.script_focus(ScriptFocus::Save);
                self.save_script_file();
            }
            RewriteHit::ScriptAi => {
                self.script_focus(ScriptFocus::Ai);
                self.open_ai_dialog();
            }
            RewriteHit::ScriptTest => {
                self.script_focus(ScriptFocus::Test);
                self.start_script_test();
            }
            RewriteHit::ScriptCancel => self.cancel_script_test(),
            RewriteHit::ScriptRow(i) => {
                self.script_focus(ScriptFocus::Results);
                if let Some(ed) = &mut self.script.editor {
                    ed.cursor = i;
                }
            }
            _ => {}
        }
    }

    /// The wheel over the editor text or the result table.
    pub(crate) fn on_script_wheel(&mut self, hit: Option<RewriteHit>, delta: isize) {
        match hit {
            Some(RewriteHit::ScriptEditor) => {
                let area = self.paint.script_text.get();
                let Some(text) = self.script_text().map(str::to_string) else {
                    return;
                };
                if let Some(ed) = &mut self.script.editor {
                    let top = (ed.caret.top.get() as isize + delta).max(0) as usize;
                    ed.caret.scroll_lines_to(&text, top, area.height as usize);
                }
            }
            Some(RewriteHit::ScriptRow(_)) => self.script_scroll(delta),
            _ => {}
        }
    }

    // ------------------------------------------------------------- paint

    /// Paints the overlay and returns its click targets, inner-first, ending
    /// with the panel and a shield over the rest of the workbench.
    pub(super) fn paint_script(
        &self,
        buf: &mut Buffer,
        bounds: Rect,
        hovered: Option<RewriteHit>,
    ) -> Vec<(Rect, RewriteHit)> {
        let mut hits = Vec::new();
        let Some(ed) = &self.script.editor else {
            return hits;
        };
        let rect = Rect::new(
            bounds.x + 2,
            bounds.y + 1,
            bounds.width.saturating_sub(4),
            bounds.height.saturating_sub(2),
        );
        let title = if self.script_ref().is_some_and(|s| s.language.is_external()) {
            t::SCRIPT_TITLE_EXTERNAL
        } else {
            t::SCRIPT_TITLE
        };
        // The script the AI wrote says so in the title, until it is edited.
        let titled = match self.ai_title_suffix() {
            Some(mark) => format!("{}  {mark}", title.get()),
            None => title.get().to_string(),
        };
        dialog_kit::panel(buf, rect, &titled);
        self.tint_title_spark(buf, rect);
        if let Some(cross) =
            crate::chrome::close_cross::paint(buf, rect, hovered == Some(RewriteHit::ScriptClose))
        {
            hits.push((cross, RewriteHit::ScriptClose));
        }
        let inner = Rect::new(
            rect.x + 1,
            rect.y + 1,
            rect.width.saturating_sub(2),
            rect.height.saturating_sub(2),
        );
        if inner.width < 40 || inner.height < 12 {
            dialog_kit::put(
                buf,
                inner.x + 1,
                inner.y,
                t::SCRIPT_TOO_SMALL.get(),
                inner.width.saturating_sub(1),
                dim(),
            );
            hits.push((rect, RewriteHit::ScriptPanel));
            hits.push((bounds, RewriteHit::ScriptShield));
            return hits;
        }
        let (left, right) = if inner.width >= 80 {
            let lw = (inner.width * 48 / 100).max(36);
            let left = Rect::new(inner.x, inner.y, lw, inner.height);
            for y in inner.y..inner.bottom() {
                dialog_kit::put(
                    buf,
                    left.right(),
                    y,
                    "\u{2502}",
                    1,
                    Style::default().fg(th().ui.muted).bg(th().ui.panel),
                );
            }
            let right = Rect::new(
                left.right() + 1,
                inner.y,
                inner.width - lw - 1,
                inner.height,
            );
            (left, right)
        } else {
            let top_h = (inner.height * 3 / 5).max(10);
            let left = Rect::new(inner.x, inner.y, inner.width, top_h);
            for x in inner.x..inner.right() {
                dialog_kit::put(
                    buf,
                    x,
                    inner.y + top_h,
                    "\u{2500}",
                    1,
                    Style::default().fg(th().ui.muted).bg(th().ui.panel),
                );
            }
            let right = Rect::new(
                inner.x,
                inner.y + top_h + 1,
                inner.width,
                inner.height.saturating_sub(top_h + 1),
            );
            (left, right)
        };
        self.paint_script_left(buf, left, ed, hovered, &mut hits);
        self.paint_script_right(buf, right, ed, hovered, &mut hits);
        hits.push((rect, RewriteHit::ScriptPanel));
        hits.push((bounds, RewriteHit::ScriptShield));
        hits
    }

    fn paint_script_left(
        &self,
        buf: &mut Buffer,
        area: Rect,
        ed: &ScriptEditor,
        hovered: Option<RewriteHit>,
        hits: &mut Vec<(Rect, RewriteHit)>,
    ) {
        let text = self.script_text().unwrap_or("");
        let check = self.check_view();
        let check_lines = check_text(&check, area.width.saturating_sub(2) as usize);
        // Never more rows than the area has: the editor keeps three, the
        // check gives up lines first, then the parameter rows.
        let check_h =
            (check_lines.len().clamp(1, 3) as u16).min(area.height.saturating_sub(9).max(1));
        let params = self.param_fields();
        // The label, the check, the parameters' heading and the file rows
        // are fixed.
        let room = area.height.saturating_sub(1 + check_h + 1 + 2);
        let param_rows = (params.len().clamp(1, 4) as u16).min(room.saturating_sub(3).max(1));
        let editor_h = room.saturating_sub(param_rows).max(1);
        let mut y = area.y;
        dialog_kit::put(
            buf,
            area.x + 1,
            y,
            t::SCRIPT_EDITOR_LABEL.get(),
            area.width.saturating_sub(1),
            bold(ed.focus == ScriptFocus::Editor),
        );
        y += 1;

        // The editor: a line-number gutter, then the text.
        let box_ = Rect::new(area.x + 1, y, area.width.saturating_sub(2), editor_h);
        let focused = ed.focus == ScriptFocus::Editor;
        let bg = crate::input_bg(focused, hovered == Some(RewriteHit::ScriptEditor));
        dialog_kit::fill(buf, box_, Style::default().bg(bg));
        let lines = text.split('\n').count();
        let gutter = (lines.max(9).to_string().len() + 1) as u16;
        let body = Rect::new(
            box_.x + gutter,
            box_.y,
            box_.width.saturating_sub(gutter),
            box_.height,
        );
        self.paint.script_text.set(body);
        let style = Style::default().fg(th().ui.text).bg(bg);
        let look = EditLook::new(style, "\u{2588}");
        let drawn = edit_lines(
            text,
            Some(&ed.caret),
            body.width.saturating_sub(1) as usize,
            body.height as usize,
            focused,
            look,
        );
        for (i, line) in drawn.iter().enumerate() {
            buf.set_line(
                body.x,
                body.y + i as u16,
                line,
                body.width.saturating_sub(1),
            );
        }
        let top = ed.caret.top.get();
        let bad_lines = match &check {
            CheckView::Problems(p) => problem_lines(p),
            _ => HashSet::new(),
        };
        for i in 0..drawn.len() {
            let number = top + i + 1;
            let style = if bad_lines.contains(&number) {
                Style::default().fg(th().status.danger).bg(bg)
            } else {
                Style::default().fg(th().ui.muted).bg(bg)
            };
            let label = format!("{number:>w$}", w = gutter as usize - 1);
            dialog_kit::put(buf, box_.x, box_.y + i as u16, &label, gutter, style);
        }
        // The bar rides the divider or the panel's border, right of the box.
        self.paint_bar_on(
            buf,
            hits,
            hovered == Some(RewriteHit::Bar(BarId::ScriptText)),
            BarId::ScriptText,
            (area.right(), box_.y, box_.height),
            lines,
            box_.height as usize,
            top,
            None,
        );
        hits.push((body, RewriteHit::ScriptEditor));
        y += editor_h;

        // The check line(s).
        let style = match check {
            CheckView::Problems(_) | CheckView::Failed(_) => bad(),
            CheckView::Ok(_) | CheckView::ExternalOk => plain(),
            _ => dim(),
        };
        for (i, line) in check_lines.iter().take(check_h as usize).enumerate() {
            dialog_kit::put(
                buf,
                area.x + 1,
                y + i as u16,
                line,
                area.width.saturating_sub(2),
                style,
            );
        }
        y += check_h;

        // The parameters, label next to the field.
        dialog_kit::put(
            buf,
            area.x + 1,
            y,
            t::SCRIPT_PARAMS_LABEL.get(),
            area.width.saturating_sub(1),
            bold(matches!(ed.focus, ScriptFocus::Param(_))),
        );
        y += 1;
        if params.is_empty() {
            dialog_kit::put(
                buf,
                area.x + 1,
                y,
                t::SCRIPT_PARAMS_NONE.get(),
                area.width.saturating_sub(2),
                dim(),
            );
        }
        let label_w = params
            .iter()
            .map(|(at, _)| display_width(self.detail.fields[*at].title()))
            .max()
            .unwrap_or(8)
            .clamp(8, 28) as u16;
        // The window follows the focused parameter.
        let focused_param = match ed.focus {
            ScriptFocus::Param(i) => params.iter().position(|(_, p)| *p == i),
            _ => None,
        };
        let mut first = self.paint.script_params_top.get();
        if !self.bar_free(BarId::ScriptParams, focused_param) {
            first = focused_param
                .map(|f| (f + 1).saturating_sub(param_rows as usize))
                .unwrap_or(0);
        }
        first = first.min(params.len().saturating_sub(param_rows as usize));
        self.paint.script_params_top.set(first);
        self.paint_bar_on(
            buf,
            hits,
            hovered == Some(RewriteHit::Bar(BarId::ScriptParams)),
            BarId::ScriptParams,
            (area.right(), y, param_rows),
            params.len(),
            param_rows as usize,
            first,
            focused_param,
        );
        for (slot, (at, i)) in params
            .iter()
            .skip(first)
            .take(param_rows as usize)
            .enumerate()
        {
            let field = &self.detail.fields[*at];
            let ry = y + slot as u16;
            let focus = ed.focus == ScriptFocus::Param(*i);
            let hit = RewriteHit::ScriptParam(*i);
            dialog_kit::put(
                buf,
                area.x + 2,
                ry,
                field.title(),
                label_w,
                Style::default().fg(th().ui.text).bg(th().ui.panel),
            );
            let vx = area.x + 2 + label_w + 2;
            let vw = area.right().saturating_sub(vx + 1).clamp(0, 24);
            if vw < 4 {
                continue;
            }
            let value = Rect::new(vx, ry, vw, 1);
            match &field.value {
                FieldValue::Toggle(on) => {
                    let r = dialog_kit::checkbox(
                        buf,
                        vx,
                        ry,
                        vw,
                        "",
                        *on,
                        dialog_kit::CheckState::Enabled {
                            hovered: hovered == Some(hit),
                            focused: focus,
                        },
                    );
                    hits.push((Rect::new(r.x, r.y, r.width.max(3), 1), hit));
                }
                FieldValue::Text(shown) => {
                    let bg = crate::input_bg(focus, hovered == Some(hit));
                    dialog_kit::fill(buf, value, Style::default().bg(bg));
                    let style = Style::default().fg(th().ui.text).bg(bg);
                    let editing = self
                        .editing
                        .as_ref()
                        .filter(|e| e.target == super::EditTarget::Field(*at));
                    match editing {
                        Some(e) => {
                            let line = edit_line(
                                &e.text,
                                Some(&e.caret),
                                vw as usize - 1,
                                true,
                                EditLook::new(style, "\u{2588}"),
                            );
                            buf.set_line(vx + 1, ry, &line, vw - 1);
                        }
                        None => {
                            // A missing value says so where it is missing.
                            match (&field.error, shown.is_empty()) {
                                (Some(why), true) => {
                                    let warn = Style::default().fg(th().status.danger).bg(bg);
                                    dialog_kit::put(buf, vx + 1, ry, why, vw - 1, warn);
                                }
                                _ => {
                                    dialog_kit::put(buf, vx + 1, ry, shown, vw - 1, style);
                                }
                            }
                        }
                    }
                    hits.push((value, hit));
                }
                FieldValue::Choice { .. } => {}
            }
            if let Some(why) = field.error.as_ref().filter(|_| match &field.value {
                FieldValue::Text(t) => !t.is_empty(),
                _ => true,
            }) {
                let ex = vx + vw + 1;
                if ex + 4 < area.right() {
                    dialog_kit::put(buf, ex, ry, why, area.right() - ex - 1, bad());
                }
            }
        }
        y += param_rows;

        // The file row and its two buttons.
        dialog_kit::put(
            buf,
            area.x + 2,
            y,
            t::SCRIPT_FILE_LABEL.get(),
            label_w.min(area.width),
            Style::default().fg(th().ui.text).bg(th().ui.panel),
        );
        let px = area.x + 2 + label_w + 2;
        let pw = area.right().saturating_sub(px + 1);
        if pw >= 6 {
            let pfocus = ed.focus == ScriptFocus::Path;
            let hit = RewriteHit::ScriptPath;
            let pbg = crate::input_bg(pfocus, hovered == Some(hit));
            let prect = Rect::new(px, y, pw, 1);
            dialog_kit::fill(buf, prect, Style::default().bg(pbg));
            let pstyle = Style::default().fg(th().ui.text).bg(pbg);
            let line = edit_line(
                &ed.path,
                Some(&ed.path_caret),
                pw as usize - 1,
                pfocus,
                EditLook::new(pstyle, "\u{2588}"),
            );
            buf.set_line(px + 1, y, &line, pw - 1);
            self.paint.script_path.set(Rect::new(px + 1, y, pw - 1, 1));
            hits.push((prect, hit));
        }
        y += 1;
        let mut x = area.x + 2;
        let buttons = [
            (
                t::SCRIPT_BTN_LOAD.get(),
                RewriteHit::ScriptLoad,
                ScriptFocus::Load,
            ),
            (
                t::SCRIPT_BTN_SAVE.get(),
                RewriteHit::ScriptSave,
                ScriptFocus::Save,
            ),
        ];
        let ai_button = self.ai_offered();
        for (label, hit, focus) in buttons {
            let w = button_width(label);
            if x + w > area.right() {
                break;
            }
            let r = dialog_kit::button(
                buf,
                x,
                y,
                label,
                ButtonState::Normal {
                    hovered: hovered == Some(hit),
                    focused: ed.focus == focus,
                },
            );
            hits.push((r, hit));
            x += w + 1;
        }
        if ai_button && x + Self::ai_button_width() <= area.right() {
            let hit = RewriteHit::ScriptAi;
            let r =
                self.paint_ai_button(buf, x, y, ed.focus == ScriptFocus::Ai, hovered == Some(hit));
            hits.push((r, hit));
        }
    }

    fn paint_script_right(
        &self,
        buf: &mut Buffer,
        area: Rect,
        ed: &ScriptEditor,
        hovered: Option<RewriteHit>,
        hits: &mut Vec<(Rect, RewriteHit)>,
    ) {
        if area.height < 4 || area.width < 20 {
            return;
        }
        let mut y = area.y;
        let running = matches!(self.script.test, TestState::Running { .. });
        let blocker = self.test_blocker();
        // The two buttons.
        let label = t::SCRIPT_BTN_TEST.get();
        let state = if blocker.is_some() || running {
            ButtonState::Disabled
        } else {
            ButtonState::Primary {
                hovered: hovered == Some(RewriteHit::ScriptTest),
                focused: ed.focus == ScriptFocus::Test,
            }
        };
        let r = dialog_kit::button(buf, area.x + 1, y, label, state);
        hits.push((r, RewriteHit::ScriptTest));
        if running {
            let cancel = t::SCRIPT_BTN_CANCEL.get();
            let c = dialog_kit::button(
                buf,
                r.right() + 1,
                y,
                cancel,
                ButtonState::Normal {
                    hovered: hovered == Some(RewriteHit::ScriptCancel),
                    focused: false,
                },
            );
            hits.push((c, RewriteHit::ScriptCancel));
        } else if !self.scope_ctx.test_commits.is_empty() {
            let nx = r.right() + 2;
            if nx + 4 < area.right() {
                dialog_kit::put(
                    buf,
                    nx,
                    y,
                    &self.test_target(),
                    area.right() - nx - 1,
                    dim(),
                );
            }
        }
        y += 1;

        // The status line(s).
        let key = self.script_key().unwrap_or(0);
        let mut extra: Vec<String> = Vec::new();
        let (status, style): (String, Style) = match &self.script.test {
            // The reason the button is flat, else what it would do.
            TestState::Idle => (
                blocker
                    .clone()
                    .unwrap_or_else(|| t::SCRIPT_TEST_IDLE.get().to_string()),
                dim(),
            ),
            TestState::Canceled => (t::SCRIPT_TEST_CANCELED.get().to_string(), dim()),
            TestState::Running { count, since, .. } => (
                format!(
                    "{} {}s",
                    t::script_testing(*count as u64),
                    since.elapsed().as_secs()
                ),
                plain(),
            ),
            TestState::Done {
                rows,
                key: k,
                count,
            } => {
                let changed = rows.iter().filter(|r| !r.fields.is_empty()).count();
                let mut s = t::script_test_done(*count as u64, changed as u64);
                if *k != key {
                    s = format!("{s} {}", t::SCRIPT_TEST_STALE.get());
                }
                (s, plain())
            }
            TestState::Failed { commit, reason } => {
                let head = match commit {
                    Some(c) => t::script_failed_on(super::preview::short(c)),
                    None => String::new(),
                };
                // The commit on its own line, then why.
                extra = wrap(reason, area.width.saturating_sub(2) as usize);
                (head, bad())
            }
        };
        let mut lines: Vec<(String, Style)> = Vec::new();
        // What loading, saving or a refused test said comes first.
        if let Some(note) = &ed.note {
            let style = if note.error { bad() } else { dim() };
            lines.extend(
                wrap(&note.text, area.width.saturating_sub(2) as usize)
                    .into_iter()
                    .map(|l| (l, style)),
            );
        }
        lines.extend(
            wrap(&status, area.width.saturating_sub(2) as usize)
                .into_iter()
                .chain(extra)
                .map(|l| (l, style)),
        );
        let status_h = lines.len().clamp(1, 4) as u16;
        for (i, (line, style)) in lines.iter().take(status_h as usize).enumerate() {
            dialog_kit::put(
                buf,
                area.x + 1,
                y + i as u16,
                line,
                area.width.saturating_sub(2),
                *style,
            );
        }
        y += status_h + 1;
        // The table.
        let results = self.test_lines();
        if results.is_empty() || y + 2 > area.bottom() {
            self.paint.script_rows.set(0);
            return;
        }
        dialog_kit::put(
            buf,
            area.x + 1,
            y,
            t::SCRIPT_COL_COMMIT.get(),
            8,
            bold(false),
        );
        dialog_kit::put(
            buf,
            area.x + 9,
            y,
            t::SCRIPT_COL_CHANGE.get(),
            area.width.saturating_sub(10),
            bold(false),
        );
        y += 1;
        let room = area.bottom().saturating_sub(y) as usize;
        self.paint.script_rows.set(room);
        let top = ed.top.min(results.len().saturating_sub(room));
        self.paint_bar_on(
            buf,
            hits,
            hovered == Some(RewriteHit::Bar(BarId::ScriptRows)),
            BarId::ScriptRows,
            (area.right(), y, room as u16),
            results.len(),
            room,
            top,
            None,
        );
        for (slot, line) in results.iter().skip(top).take(room).enumerate() {
            let index = top + slot;
            let ry = y + slot as u16;
            let band = Rect::new(area.x, ry, area.width, 1);
            let hit = RewriteHit::ScriptRow(index);
            let selected = ed.focus == ScriptFocus::Results && ed.cursor == index;
            let bg = row_bg(selected, hovered == Some(hit));
            dialog_kit::fill(buf, band, Style::default().bg(bg));
            if let Some(short) = &line.commit {
                dialog_kit::put(
                    buf,
                    area.x + 1,
                    ry,
                    short,
                    8,
                    Style::default().fg(th().ui.accent).bg(bg),
                );
            }
            let style = match line.kind {
                ResultKind::Change => Style::default().fg(th().ui.text).bg(bg),
                ResultKind::Unchanged | ResultKind::Log => {
                    Style::default().fg(th().ui.muted).bg(bg)
                }
            };
            dialog_kit::put(
                buf,
                area.x + 9,
                ry,
                &line.text,
                area.width.saturating_sub(10),
                style,
            );
            hits.push((band, hit));
        }
    }
}

/// The check line wrapped for `width`: one line for OK or checking, the
/// problems (as many as fit) otherwise.
fn check_text(check: &CheckView, width: usize) -> Vec<String> {
    let one = |s: String| wrap(&s, width.max(8)).into_iter().take(1).collect();
    match check {
        CheckView::NotBuilt => one(t::SCRIPT_NOT_BUILT.get().to_string()),
        CheckView::Empty => Vec::new(),
        CheckView::Checking => one(t::SCRIPT_CHECKING.get().to_string()),
        CheckView::Ok(function) => one(t::script_check_ok(function)),
        CheckView::NoFunction => one(t::script_check_nofn("rewrite_commit")),
        CheckView::Untrusted => one(t::SCRIPT_CHECK_UNTRUSTED.get().to_string()),
        CheckView::ExternalOk => one(t::SCRIPT_CHECK_EXTERNAL_OK.get().to_string()),
        CheckView::Failed(why) => one(t::script_check_failed(why)),
        CheckView::Problems(list) => list
            .iter()
            .flat_map(|p| wrap(&one_line(p), width.max(8)))
            .take(3)
            .collect(),
    }
}

/// The detail pane's check line for the rule under the cursor.
pub(super) fn detail_check_line(view: &RewriteView, width: usize) -> Option<(String, bool)> {
    let check = view.check_view();
    let bad = matches!(check, CheckView::Problems(_) | CheckView::Failed(_));
    check_text(&check, width)
        .into_iter()
        .next()
        .map(|l| (l, bad))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn problem_lines_are_read_from_the_prefix() {
        let lines = problem_lines(&[
            "script:3: unexpected symbol near 'x'".into(),
            "script:12: '=' expected".into(),
            "header line 2: unknown header key `foo`".into(),
        ]);
        assert_eq!(lines, HashSet::from([3, 12]));
    }

    #[test]
    fn language_is_part_of_the_key_and_params_are_not() {
        let mut a = ScriptRef {
            language: ScriptLanguage::Lua,
            source: ScriptSource::Inline { text: "x".into() },
            params: Default::default(),
        };
        let key = key_of(&a);
        a.params.insert("p".into(), "1".into());
        assert_eq!(key_of(&a), key);
        a.language = ScriptLanguage::Shell;
        assert_ne!(key_of(&a), key);
    }
}
