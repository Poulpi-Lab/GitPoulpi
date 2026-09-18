//! An agent's proposals (see the mcp-show-ui design section
//! 11): what the process remembers about them, so `ui state` can tell the
//! agent what the user decided, across a tab switch too.
//!
//! - the **active** proposal: the one whose dialog is open, pre-filled;
//! - the **records**: the last [`KEEP`] proposals and their status.
//!
//! The dialogs' own confirm paths call [`confirmed`]; nothing here can submit
//! anything. The window watches the dialog each tick
//! (`event_loop/window/agent_propose.rs`) and settles the record when it
//! closes: confirmed (unchanged or edited) or rejected.

use crate::th;
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use gitpoulpi_core::domain::Oid;
use gitpoulpi_i18n::msg::ui_intent as msg;
use gitpoulpi_ipc::ui::{ProposalRecord, ProposalStatus};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};

/// How many proposals `ui state` lists.
const KEEP: usize = 8;

/// The inline branch/tag box ignores Enter this long after a proposal opened
/// it: a key already on its way cannot confirm it.
pub(crate) const ENTER_GUARD: Duration = Duration::from_millis(600);

/// The dialog a proposal opened.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Dialog {
    Reword(Oid),
    CommitMessage,
    Branch(Oid),
    Tag(Oid),
    /// The split-commit editor over this commit.
    Split(Oid),
    /// The merge editor, open on the proposed file.
    Conflict,
    /// The history rewrite workbench, open on the proposed recipe.
    Rewrite,
}

/// The proposal whose dialog is open.
#[derive(Debug, Clone)]
pub(crate) struct Active {
    pub(crate) id: String,
    pub(crate) client: String,
    /// What the banner calls it ("a new message for commit 1a2b3c4d").
    pub(crate) what: String,
    pub(crate) dialog: Dialog,
    /// The repository of the window that opened it.
    pub(crate) repo: PathBuf,
    /// The text as the dialog first showed it (what "edited" compares to).
    pub(crate) proposed: String,
    /// An annotated tag's proposed message, for its second step.
    pub(crate) tag_message: Option<String>,
    pub(crate) opened_at: Instant,
    /// Set by the dialog's confirm path: the text the user confirmed.
    pub(crate) confirmed: Option<String>,
}

static ACTIVE: Mutex<Option<Active>> = Mutex::new(None);
static RECORDS: Mutex<VecDeque<ProposalRecord>> = Mutex::new(VecDeque::new());

fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

fn now_unix() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() as i64)
}

/// Sets (or adds) proposal `id`'s status.
pub(crate) fn record(id: &str, kind: &str, client: &str, status: ProposalStatus) {
    let mut records = lock(&RECORDS);
    records.retain(|r| r.id != id);
    records.push_front(ProposalRecord {
        id: id.to_string(),
        kind: kind.to_string(),
        client: client.to_string(),
        status,
        at: now_unix(),
    });
    records.truncate(KEEP);
}

/// Changes the status of a recorded proposal, keeping its kind and client.
/// A settled proposal stays settled.
pub(crate) fn settle(id: &str, status: ProposalStatus) {
    let mut records = lock(&RECORDS);
    if let Some(r) = records.iter_mut().find(|r| r.id == id) {
        if !r.status.is_final() {
            r.status = status;
            r.at = now_unix();
        }
    }
}

/// The records, newest first (`ui state`).
pub(crate) fn records() -> Vec<ProposalRecord> {
    lock(&RECORDS).iter().cloned().collect()
}

/// A proposal's dialog just opened.
pub(crate) fn open(active: Active) {
    *lock(&ACTIVE) = Some(active);
}

/// Sets the text the open proposal's dialog first showed (the merge editor
/// only knows it once the file has been read and the picks applied).
pub(crate) fn set_proposed(text: &str) {
    if let Some(active) = lock(&ACTIVE).as_mut() {
        active.proposed = text.to_string();
    }
}

/// The proposal whose dialog is open, if any.
pub(crate) fn active() -> Option<Active> {
    lock(&ACTIVE).clone()
}

/// Ends the active proposal (the window settled it).
pub(crate) fn close() -> Option<Active> {
    lock(&ACTIVE).take()
}

/// A dialog's confirm path: when the open proposal is this dialog's, the
/// user confirmed `text`.
pub(crate) fn confirmed(dialog: Dialog, text: &str) {
    if let Some(active) = lock(&ACTIVE).as_mut() {
        if active.dialog == dialog {
            active.confirmed = Some(text.to_string());
        }
    }
}

/// The Reword popup's confirm path: `action` is what it submits.
pub(crate) fn reword_confirmed(action: &crate::daemon_link::Action) {
    if let crate::daemon_link::Action::Reword { commit, message } = action {
        confirmed(Dialog::Reword(*commit), message);
    }
}

/// What the split editor's plan reads as, for "edited" to compare: the plan
/// itself (messages and pieces), so moving a change is an edit too.
pub(crate) fn split_text(parts: &[gitpoulpi_core::domain::SplitPart]) -> String {
    serde_json::to_string(parts).unwrap_or_default()
}

/// The split editor's submit path: `action` is what it submits.
pub(crate) fn split_confirmed(action: &crate::daemon_link::Action) {
    if let crate::daemon_link::Action::SplitCommit { req } = action {
        confirmed(Dialog::Split(req.commit), &split_text(&req.parts));
    }
}

/// The message an open tag proposal suggests for the annotated tag at `at`
/// (its second step).
pub(crate) fn tag_message(at: Oid) -> Option<String> {
    lock(&ACTIVE)
        .as_ref()
        .filter(|a| a.dialog == Dialog::Tag(at))
        .and_then(|a| a.tag_message.clone())
}

/// Whether Enter in the ref-name box must be ignored: a proposal opened it a
/// moment ago.
pub(crate) fn guards_enter() -> bool {
    lock(&ACTIVE).as_ref().is_some_and(|a| {
        matches!(a.dialog, Dialog::Branch(_) | Dialog::Tag(_))
            && a.opened_at.elapsed() < ENTER_GUARD
    })
}

/// The "✦ Proposed by" line drawn while a proposal's dialog is open.
pub(crate) fn banner() -> Option<String> {
    let active = active()?;
    let confirm = match active.dialog {
        Dialog::Reword(_) => {
            msg::accepts_key(crate::keymap::current().shown(crate::keymap::Action::RewordApply))
        }
        Dialog::Split(_) => {
            msg::accepts_key(crate::keymap::current().shown(crate::keymap::Action::ComposeApply))
        }
        Dialog::CommitMessage => msg::ACCEPTS_COMMIT.get().into(),
        Dialog::Branch(_) | Dialog::Tag(_) => msg::ACCEPTS_ENTER.get().into(),
        Dialog::Conflict => msg::accepts_key(
            crate::keymap::current().shown(crate::keymap::Action::MergeSaveResolve),
        ),
        Dialog::Rewrite => {
            msg::accepts_key(crate::keymap::current().shown(crate::keymap::Action::RewriteApply))
        }
    };
    let text = if active.dialog == Dialog::CommitMessage {
        msg::banner(&active.client, &active.what, confirm)
    } else {
        msg::banner_esc(&active.client, &active.what, confirm)
    };
    Some(format!("{} {text}", crate::theme::AI_SPARK))
}

/// Draws [`banner`] at the right of the row above the chip's (the chip and
/// the toast keep their own row), in the AI purple.
pub(crate) fn render_banner(frame: &mut ratatui::Frame<'_>, area: ratatui::layout::Rect) {
    let Some(text) = banner() else {
        return;
    };
    let text = format!(" {text} ");
    let width = (crate::ui_kit::display_width(&text) as u16).min(area.width);
    let rect = ratatui::layout::Rect::new(area.right() - width, area.bottom() - 3, width, 1);
    frame.render_widget(crate::wide_edge::Clear, rect);
    frame.render_widget(
        ratatui::widgets::Paragraph::new(text).style(
            Style::default()
                .fg(th().ui.on_button)
                .bg(th().chrome.proposal_bg)
                .add_modifier(Modifier::BOLD),
        ),
        rect,
    );
}

/// A dialog's title, followed by "✦ Proposed by <client>" in the AI purple
/// when an agent's proposal filled it in.
pub(crate) fn dialog_title(title: &'static str, proposed_by: &Option<String>) -> Line<'static> {
    match proposed_by {
        Some(client) => Line::from(vec![
            Span::raw(title),
            Span::styled(
                format!("{} {} ", crate::theme::AI_SPARK, msg::proposed_by(client)),
                Style::default()
                    .fg(th().status.ai)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        None => Line::from(title),
    }
}

/// How the user's confirmed text compares with the proposal.
pub(crate) fn verdict(proposed: &str, confirmed: &str) -> ProposalStatus {
    let norm = |s: &str| {
        s.lines()
            .map(str::trim_end)
            .collect::<Vec<_>>()
            .join("\n")
            .trim()
            .to_string()
    };
    if norm(proposed) == norm(confirmed) {
        ProposalStatus::Accepted
    } else {
        ProposalStatus::AcceptedEdited
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `ACTIVE` is process-wide: tests that open a proposal take turns.
    static TEST_LOCK: Mutex<()> = Mutex::new(());

    fn sample(id: &str, dialog: Dialog) -> Active {
        Active {
            id: id.into(),
            client: "agent".into(),
            what: "x".into(),
            dialog,
            repo: PathBuf::from("/r"),
            proposed: "v1".into(),
            tag_message: Some("release".into()),
            opened_at: Instant::now(),
            confirmed: None,
        }
    }

    #[test]
    fn records_keep_the_newest_and_a_settled_status_sticks() {
        for i in 0..10 {
            record(&format!("t-{i}"), "branch", "agent", ProposalStatus::Open);
        }
        let all = records();
        assert!(all.len() <= KEEP);
        assert_eq!(all[0].id, "t-9");
        settle("t-9", ProposalStatus::Rejected);
        settle("t-9", ProposalStatus::Accepted);
        assert_eq!(records()[0].status, ProposalStatus::Rejected);
    }

    #[test]
    fn the_active_proposal_takes_its_own_dialogs_confirm_only() {
        let oid = Oid([7; 20]);
        let _guard = lock(&TEST_LOCK);
        open(sample("t-a", Dialog::Tag(oid)));
        assert_eq!(tag_message(oid).as_deref(), Some("release"));
        assert!(tag_message(Oid([8; 20])).is_none());
        assert!(guards_enter(), "just opened");
        confirmed(Dialog::Branch(oid), "x");
        assert!(active().unwrap().confirmed.is_none(), "another dialog");
        confirmed(Dialog::Tag(oid), "v1");
        assert_eq!(close().unwrap().confirmed.as_deref(), Some("v1"));
        assert!(active().is_none());
    }

    #[test]
    fn the_banner_reads_in_the_ui_language_and_stays_short() {
        let _guard = lock(&TEST_LOCK);
        let _fr = gitpoulpi_i18n::override_lang(gitpoulpi_i18n::Lang::Fr);
        open(sample("t-b", Dialog::Tag(Oid([1; 20]))));
        let text = banner().unwrap();
        close();
        assert_eq!(
            text,
            "\u{2726} Proposé par agent\u{a0}: x \u{b7} Enter accepte \u{b7} Esc rejette"
        );
        assert!(text.chars().count() < 80, "{text}");
        let title = dialog_title(" Titre ", &Some("agent".into()));
        let shown: String = title.spans.iter().map(|s| s.content.as_ref()).collect();
        assert!(shown.contains("Proposé par agent"), "{shown}");
    }

    #[test]
    fn verdict_ignores_trailing_spaces_only() {
        assert_eq!(verdict("a\n\nb", "a  \n\nb\n"), ProposalStatus::Accepted);
        assert_eq!(verdict("a", "b"), ProposalStatus::AcceptedEdited);
    }
}
