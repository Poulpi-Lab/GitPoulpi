//! The conversation view of an issue or a pull / merge request: read it, and
//! write to it (post, reply, quote, edit and delete your own comments).
//!
//! A full-window modal owned by `RepoDialogs` (`RepoDialog::Item`), like the
//! review and the board: keys, mouse, paste and the per-frame poll all arrive
//! there. The timeline is the shared widget (`conversation_ui`); this module
//! is the surface around it: the frame with its close cross, the header, the
//! "Posting as ..." line, the composer dock (Write / Preview / Split through
//! `md_editor`; Send greyed with its reason), the question before a delete,
//! the menu of an entry's links and the **Back stack**.
//!
//! Writing: the text goes to the host on a worker, the entry shows at once
//! and is taken back, with the text still in the composer and the error
//! under it, when the host refuses. Posting is outward: nothing is journaled
//! and nothing can be undone (`undo_ui` says so).
//!
//! Links: a `#7` of the repository's linked sources opens in this view and
//! the previous one goes on the stack (`Back`); a commit reveals in the Log;
//! everything else asks `link_confirm` first, which shows the real address.
//!
//! Hosting I/O runs on the widget's worker; nothing here blocks the input
//! thread. The saved copy paints at once, the refresh runs behind it, and an
//! unreachable host leaves the copy up with an offline banner.

mod draw;
mod input;
mod links;
#[cfg(test)]
mod tests;

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use gitpoulpi_config::Integration;
use gitpoulpi_hosting::conversation::{ItemKey, ItemKind};
use gitpoulpi_hosting::refs::RefContext;
use ratatui::layout::Rect;

use crate::combo::Combo;
use crate::conversation_ui::worker::Source;
use crate::conversation_ui::write::WriteOp;
use crate::conversation_ui::{self as cu, Conversation};
use crate::dialog_kit::{self, Confirm};
use crate::hosting_task::ClientFactory;
use crate::item_cmd::ItemAvail;
use crate::md_editor::MdEditor;
use crate::md_render;

/// The open item refreshes by itself this often on hosts that answer a
/// conditional request cheaply (GitHub, GitLab, Gitea).
pub(crate) const AUTO_REFRESH: Duration = Duration::from_secs(60);

/// What the view opens on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ItemTarget {
    pub(crate) integration: Integration,
    /// Every profile (the token source reads them all).
    pub(crate) all: Vec<Integration>,
    pub(crate) project: String,
    pub(crate) key: ItemKey,
    /// What a human types for it (`#142`, `ABC-12`): the title until the
    /// header arrives.
    pub(crate) reference: String,
    pub(crate) title: String,
    pub(crate) web_url: String,
    /// The repository the work tree belongs to; `None`: no clone (a commit
    /// link then opens on the host).
    pub(crate) repo_root: Option<PathBuf>,
    /// The repository's linked ISSUES and PULLS sources: what `#N` means.
    pub(crate) bases: RefContext,
}

impl ItemTarget {
    /// What `#N`, `!N`, mentions and commit ids in the item's text mean.
    pub(crate) fn md_context(&self) -> md_render::Context {
        md_render::Context {
            xrefs: gitpoulpi_hosting::xrefs::XrefContext {
                family: Some(self.integration.kind.family()),
                web_base: gitpoulpi_hosting::integration_web_base(&self.integration)
                    .unwrap_or_default(),
                project: self.project.clone(),
                repo_links: self.bases.clone(),
                ..Default::default()
            },
            diagrams: crate::build_features::MERMAID,
        }
    }

    fn source(&self) -> Source {
        Source {
            integration: self.integration.clone(),
            all: self.all.clone(),
            project: self.project.clone(),
            key: self.key.clone(),
        }
    }

    /// The state file of the repository, when there is one.
    fn state_path(&self) -> Option<PathBuf> {
        self.repo_root.as_deref().map(|root| {
            if cfg!(test) {
                // Never the real configuration under test.
                gitpoulpi_session::RepoState::path_for(
                    root,
                    gitpoulpi_config::DEFAULT_REPO_DATA_LOCATION,
                )
            } else {
                crate::remotes_ui::repo_state_path(root)
            }
        })
    }

    fn seen_key(&self) -> String {
        gitpoulpi_session::conversation_key(
            &self.integration.id,
            &self.project,
            match self.key.kind {
                ItemKind::Issue => "issue",
                ItemKind::PullRequest => "request",
            },
            self.key.number,
        )
    }

    fn draft_key(&self) -> String {
        crate::md_editor::draft::key(&[
            &self.integration.id,
            &self.project,
            &format!("{}#{}", kind_word(self.key.kind), self.key.number),
            "comment",
        ])
    }

    fn reply_draft_key(&self, discussion: &str) -> String {
        crate::md_editor::draft::key(&[
            &self.integration.id,
            &self.project,
            &format!("{}#{}", kind_word(self.key.kind), self.key.number),
            "reply",
            discussion,
        ])
    }

    /// Whether a change the host announced is about this item.
    fn is(&self, change: &gitpoulpi_ipc::HostingChangeArgs) -> bool {
        use gitpoulpi_ipc::HostingItemKind as K;
        change.integration_id == self.integration.id
            && change.project == self.project
            && change.number == self.key.number
            && matches!(
                (change.kind, self.key.kind),
                (K::Issue, ItemKind::Issue) | (K::PullRequest, ItemKind::PullRequest)
            )
            && (change.id.is_empty() || self.key.id.is_empty() || change.id == self.key.id)
    }
}

fn kind_word(kind: ItemKind) -> &'static str {
    match kind {
        ItemKind::Issue => "issue",
        ItemKind::PullRequest => "request",
    }
}

/// The issues of this provider have a conversation to read here.
pub(crate) fn supports_issues(kind: gitpoulpi_config::IntegrationKind) -> bool {
    !matches!(
        kind.family(),
        gitpoulpi_config::ApiFamily::Bitbucket | gitpoulpi_config::ApiFamily::Gerrit
    )
}

/// The requests of this provider have a conversation to read here, where no
/// review view exists.
pub(crate) fn supports_requests(kind: gitpoulpi_config::IntegrationKind) -> bool {
    matches!(
        kind.family(),
        gitpoulpi_config::ApiFamily::AzureDevops | gitpoulpi_config::ApiFamily::Bitbucket
    )
}

/// The key an issue of the list is read under: the number, or the
/// provider's own key where the tracker addresses it that way.
pub(crate) fn issue_key(
    kind: gitpoulpi_config::IntegrationKind,
    issue: &gitpoulpi_hosting::IssueSummary,
) -> ItemKey {
    use gitpoulpi_config::ApiFamily as F;
    let id = match kind.family() {
        F::Jira | F::Linear | F::YouTrack if !issue.reference.is_empty() => issue.reference.clone(),
        F::Trello => issue.id.clone(),
        _ => String::new(),
    };
    ItemKey {
        kind: ItemKind::Issue,
        number: issue.number,
        id,
    }
}

/// What the owner of the view has to do after it handled something.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ItemEffect {
    None,
    /// Leave the view.
    Close,
    Message(String),
    /// Hand this address to the system.
    OpenUrl(String),
    /// Leave the view and put the Log on this commit (an abbreviated id).
    ShowCommit(String),
    /// Open the diagram viewer on this Mermaid script.
    OpenDiagram(String),
}

/// Where the keyboard is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Focus {
    Timeline,
    Composer,
    /// One of the header's buttons.
    Bar(Bar),
}

/// The header's buttons.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Bar {
    Back,
    Refresh,
    Browser,
    Events,
}

impl Bar {
    pub(super) const ALL: [Bar; 4] = [Bar::Back, Bar::Refresh, Bar::Browser, Bar::Events];
}

/// A click target of the surface (the timeline keeps its own).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Hit {
    Bar(Bar),
    Send,
    Cancel,
    Internal,
}

/// What the comment box is writing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Compose {
    /// A new comment on the item (quotes are put in it).
    New,
    /// An answer in a discussion or a code thread.
    Reply { discussion: String, label: String },
    /// A new text for one of our comments; `original` is what it said.
    Edit { id: String, original: String },
}

/// The answers of the question before a delete.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DeleteChoice {
    Keep,
    Delete,
}

/// Whose "discard" the open question is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(super) enum Discarding {
    /// The view is closing.
    #[default]
    Close,
    /// Only the comment box is put away.
    Composer,
}

/// The item the view came from: what Back returns to.
struct Frame {
    target: ItemTarget,
    conv: Conversation,
}

/// The menu of the selected entry's links.
pub(super) struct LinkMenu {
    pub(super) combo: Combo,
    pub(super) links: Vec<md_render::LinkTarget>,
}

pub(crate) struct ItemView {
    pub(super) target: ItemTarget,
    pub(super) factory: Arc<dyn ClientFactory>,
    pub(super) conv: Conversation,
    stack: Vec<Frame>,
    pub(super) focus: Focus,
    pub(super) md: Option<MdEditor>,
    pub(super) composer_open: bool,
    pub(super) compose: Compose,
    /// What the host last said when a send failed (shown under the text).
    pub(super) compose_error: Option<String>,
    /// The new comment is an internal note (hosts that have them).
    pub(super) internal: bool,
    /// "Delete this comment?", with the comment it is about.
    pub(super) delete_ask: Option<(Confirm<DeleteChoice>, String)>,
    pub(super) discarding: Discarding,
    pub(super) menu: Option<LinkMenu>,
    pub(super) guard: dialog_kit::DiscardGuard,
    pub(super) hits: Vec<(Rect, Hit)>,
    pub(super) hover: Option<Hit>,
    pub(super) message: Option<String>,
    /// The frame of the last paint.
    pub(super) frame: Rect,
    /// Where the composer's text field last sat (a click there focuses it).
    pub(super) composer_rect: Rect,
    /// Lives inside another view (the review's Overview): that view owns the
    /// frame, the header, the footer and the close, and routes keys and the
    /// mouse here (`embedded_key`, `on_mouse`).
    pub(super) embedded: bool,
}

impl ItemView {
    /// Opens the view and starts the read.
    pub(crate) fn open(target: ItemTarget, factory: Arc<dyn ClientFactory>) -> Self {
        register_scope(&target);
        let conv = Self::open_conversation(&target, &factory);
        Self {
            target,
            factory,
            conv,
            stack: Vec::new(),
            focus: Focus::Timeline,
            md: None,
            composer_open: true,
            compose: Compose::New,
            compose_error: None,
            internal: false,
            delete_ask: None,
            discarding: Discarding::Close,
            menu: None,
            guard: Default::default(),
            hits: Vec::new(),
            hover: None,
            message: None,
            frame: Rect::default(),
            composer_rect: Rect::default(),
            embedded: false,
        }
    }

    /// Opens the conversation to be shown inside another view (the review's
    /// Overview): the same timeline and docked composer, without a frame.
    pub(crate) fn open_embedded(target: ItemTarget, factory: Arc<dyn ClientFactory>) -> Self {
        let mut view = Self::open(target, factory);
        view.embedded = true;
        view
    }

    fn open_conversation(target: &ItemTarget, factory: &Arc<dyn ClientFactory>) -> Conversation {
        let (seen, events, posted) = match target.state_path() {
            Some(path) => {
                let state = gitpoulpi_session::RepoState::load(&path).unwrap_or_default();
                (
                    state
                        .conversation_seen
                        .get(&target.seen_key())
                        .map(|s| s.at.clone())
                        .filter(|a| !a.is_empty()),
                    state.conversation_events.unwrap_or(true),
                    state.posted_comments_of(&target.seen_key()),
                )
            }
            None => (None, true, Vec::new()),
        };
        let cache_dir = target
            .state_path()
            .and_then(|p| p.parent().map(PathBuf::from));
        let mut conv = Conversation::new(
            Arc::clone(factory),
            target.source(),
            cache_dir,
            target.md_context(),
            seen,
            events,
        );
        conv.set_write_context(target.state_path(), target.seen_key(), posted);
        conv
    }

    // -- state the owner and the palette read ----------------------------------

    pub(crate) fn busy(&self) -> bool {
        // A read in flight keeps the frame rate up so its spinner can appear
        // once it has run past 150 ms (the timer's quiet reads do not count).
        self.conv.busy() && !self.conv.is_quiet()
    }

    /// When the view next wants a frame without input (the 60 s refresh).
    pub(crate) fn next_deadline(&self) -> Option<std::time::Instant> {
        self.auto_refreshes().then(|| {
            std::time::Instant::now() + AUTO_REFRESH.saturating_sub(self.conv.since_fresh())
        })
    }

    fn auto_refreshes(&self) -> bool {
        use gitpoulpi_config::ApiFamily as F;
        matches!(
            self.target.integration.kind.family(),
            F::Github | F::Gitlab | F::Gitea
        ) && self.conv.header.is_some()
    }

    /// The palette's keys reach the view unless a field or a popup has them.
    pub(crate) fn takes_palette(&self) -> bool {
        self.focus != Focus::Composer
            && self.menu.is_none()
            && !self.guard.is_open()
            && self.delete_ask.is_none()
    }

    /// What the palette may ask of the composer's AI help (`None`: it has
    /// none, or a menu or question is open over the view).
    pub(crate) fn composer_avail(&self) -> Option<crate::composer_cmd::ComposerAvail> {
        if self.menu.is_some() || self.guard.is_open() || self.delete_ask.is_some() {
            return None;
        }
        self.md.as_ref().and_then(|e| e.composer_avail(false))
    }

    /// Runs a palette row on the composer's AI help.
    pub(crate) fn run_composer(&mut self, cmd: crate::composer_cmd::ComposerCmd) -> Option<String> {
        self.focus = Focus::Composer;
        self.md.as_mut().and_then(|e| e.run_composer(cmd))
    }

    /// Why Send does nothing now (`None`: it would send).
    pub(super) fn send_reason(&self) -> Option<&'static str> {
        use gitpoulpi_i18n::msg::item_view as m;
        if self.conv.is_writing() {
            return Some(m::WHY_SENDING.get());
        }
        let Some(caps) = &self.conv.caps else {
            return Some(m::WHY_NOT_LOADED.get());
        };
        match &self.compose {
            Compose::Edit { .. } => {
                if !caps.edit_own {
                    return Some(m::WHY_NO_EDIT.get());
                }
            }
            Compose::Reply { .. } => {
                if !caps.reply_in_thread {
                    return Some(m::WHY_NO_REPLY.get());
                }
            }
            Compose::New => {
                if !caps.comment {
                    return Some(m::WHY_NO_COMMENT.get());
                }
            }
        }
        if self.conv.is_offline() {
            return Some(m::WHY_OFFLINE.get());
        }
        if self.md.as_ref().is_none_or(|e| e.is_blank()) {
            return Some(m::WHY_EMPTY.get());
        }
        if self.over_limit() {
            return Some(m::WHY_TOO_LONG.get());
        }
        if !self.composer_dirty() {
            return Some(m::WHY_UNCHANGED.get());
        }
        None
    }

    /// The characters the host takes in one comment, when it says.
    pub(super) fn max_body(&self) -> Option<usize> {
        self.conv.caps.as_ref().and_then(|c| c.max_body)
    }

    /// How many characters the composer holds.
    pub(super) fn body_chars(&self) -> usize {
        self.md.as_ref().map_or(0, |e| e.text().chars().count())
    }

    /// The text is longer than the host takes (the CLI refuses the same
    /// before any request).
    pub(super) fn over_limit(&self) -> bool {
        self.max_body().is_some_and(|max| self.body_chars() > max)
    }

    pub(crate) fn avail(&self) -> ItemAvail {
        use gitpoulpi_i18n::msg::item_view as m;
        let has_entry = self.conv.writable_comment().is_some();
        let sending = self.conv.is_writing().then(|| m::WHY_SENDING.get());
        let unsent = self
            .conv
            .writable_comment()
            .is_some_and(|c| cu::write::is_pending(&c.id));
        let edit = if !self.conv.caps.as_ref().is_some_and(|c| c.edit_own) {
            Some(m::WHY_NO_EDIT.get())
        } else if unsent {
            Some(m::WHY_UNSENT.get())
        } else if !has_entry || !self.conv.can_edit_selected() {
            Some(m::WHY_NOT_YOURS.get())
        } else {
            sending
        };
        let delete = if !self.conv.caps.as_ref().is_some_and(|c| c.delete_own) {
            Some(m::WHY_NO_DELETE.get())
        } else if unsent {
            Some(m::WHY_UNSENT.get())
        } else if !has_entry || !self.conv.can_delete_selected() {
            Some(m::WHY_NOT_YOURS.get())
        } else {
            sending
        };
        let quote = match self.conv.reply_reason() {
            Some(why) => Some(why),
            None if self.conv.selected_entry().is_none() && self.conv.selected().is_none() => {
                Some(m::WHY_NOT_LOADED.get())
            }
            None => None,
        };
        let internal = if !self.conv.caps.as_ref().is_some_and(|c| c.internal_notes)
            || !matches!(self.compose, Compose::New)
        {
            Some(m::WHY_NO_INTERNAL.get())
        } else {
            None
        };
        ItemAvail {
            loaded: self.conv.header.is_some(),
            has_back: !self.stack.is_empty(),
            has_older: self.conv.older.is_some(),
            has_unread: self.conv.has_unread(),
            has_links: self.conv.selected().is_some_and(|i| !i.links.is_empty()),
            has_diagram: self
                .conv
                .selected_links()
                .iter()
                .any(|l| l.diagram.is_some()),
            has_url: self.conv.selected_url().is_some(),
            reply: self.conv.reply_reason(),
            quote,
            edit,
            delete,
            send: self.send_reason(),
            internal,
            has_events: self.conv.caps.as_ref().is_some_and(|c| c.events),
            split_fits: self
                .md
                .as_ref()
                .is_some_and(|e| e.area().width >= crate::md_editor::SPLIT_MIN_WIDTH),
            composer: self.composer_available(),
        }
    }

    /// The host takes comments, so the composer is shown.
    pub(super) fn composer_available(&self) -> bool {
        self.conv.can_comment()
    }

    // -- the composer ------------------------------------------------------------

    /// Creates the composer when the capabilities say it can exist.
    pub(super) fn ensure_composer(&mut self) {
        if self.md.is_none() && self.composer_available() {
            self.compose = Compose::New;
            let md = MdEditor::new(String::new(), self.target.md_context())
                .with_draft(self.target.draft_key());
            self.md = Some(md);
        }
    }

    pub(super) fn composer_dirty(&self) -> bool {
        let Some(md) = self.md.as_ref() else {
            return false;
        };
        match &self.compose {
            Compose::Edit { original, .. } => md.text() != original,
            _ => !md.is_blank(),
        }
    }

    /// Puts the comment box in `mode`, with `text` in it. A box holding
    /// unsent text of another kind is never replaced: that says why.
    pub(super) fn start_compose(
        &mut self,
        mode: Compose,
        text: String,
    ) -> Result<(), &'static str> {
        use gitpoulpi_i18n::msg::item_view as m;
        if self.conv.is_writing() {
            return Err(m::WHY_SENDING.get());
        }
        if self.md.is_some() && self.compose != mode && self.composer_dirty() {
            return Err(m::WHY_COMPOSER_BUSY.get());
        }
        if self.md.is_none() || self.compose != mode {
            let ctx = self.target.md_context();
            let md = MdEditor::new(text, ctx);
            let md = match &mode {
                Compose::New => md.with_draft(self.target.draft_key()),
                Compose::Reply { discussion, .. } => {
                    md.with_draft(self.target.reply_draft_key(discussion))
                }
                Compose::Edit { .. } => md,
            };
            self.md = Some(md);
            self.compose = mode;
            self.compose_error = None;
        }
        self.composer_open = true;
        self.focus = Focus::Composer;
        Ok(())
    }

    /// The comment box is let go of: a draft stays for the next visit.
    pub(super) fn release_composer(&mut self) {
        self.md = None;
        self.compose = Compose::New;
        self.compose_error = None;
        self.internal = false;
    }

    /// The comment box goes back to a plain new comment, its text gone.
    pub(super) fn reset_composer(&mut self) {
        if let Some(md) = self.md.as_mut() {
            md.discard();
        }
        self.md = None;
        self.compose = Compose::New;
        self.compose_error = None;
        self.internal = false;
    }

    /// What a finished write means for the box and the status.
    fn on_write_done(&mut self, done: cu::write::WriteDone) {
        use gitpoulpi_i18n::msg::item_view as m;
        match (&done.op, done.result) {
            (WriteOp::Delete { .. }, Ok(())) => {
                crate::undo_ui::note_outward_comment();
            }
            (WriteOp::Delete { .. }, Err(why)) => {
                self.message = Some(m::delete_failed(why));
            }
            (op, Ok(())) => {
                crate::undo_ui::note_outward_comment();
                if let Some(md) = self.md.as_mut() {
                    md.sent();
                }
                self.md = None;
                self.compose = Compose::New;
                self.compose_error = None;
                self.internal = false;
                self.focus = Focus::Timeline;
                let _ = op;
            }
            (op, Err(why)) => {
                // The text is still in the box: nothing the user typed is lost.
                self.compose_error = Some(match op {
                    WriteOp::Edit { .. } => m::edit_failed(why),
                    _ => m::send_failed(why),
                });
                self.composer_open = true;
                self.focus = Focus::Composer;
            }
        }
    }

    /// The host says an item changed (the CLI or MCP wrote to it): read it
    /// again when it is the one shown, or one waiting on the Back stack.
    pub(crate) fn on_hosting_changed(&mut self, change: &gitpoulpi_ipc::HostingChangeArgs) {
        if self.target.is(change) {
            self.conv.request_refresh();
        }
        for f in &mut self.stack {
            if f.target.is(change) {
                f.conv.request_refresh();
            }
        }
    }

    /// The conversation is read again (its host data changed behind it).
    pub(crate) fn request_refresh(&mut self) {
        self.conv.request_refresh();
    }

    /// The composer is open and has no "Write with AI" help yet (the host
    /// view binds one, `bind_composer_ai`).
    pub(crate) fn composer_needs_ai(&self) -> bool {
        self.md.as_ref().is_some_and(|e| !e.ai_bound())
    }

    /// The discussion the composer answers (`None`: a new comment or an edit).
    pub(crate) fn composer_replies_to(&self) -> Option<&str> {
        match &self.compose {
            Compose::Reply { discussion, .. } => Some(discussion),
            _ => None,
        }
    }

    pub(crate) fn bind_composer_ai(
        &mut self,
        scope: crate::md_editor::ai::AiScope,
        source: Box<dyn crate::md_editor::ai::AiContextSource>,
    ) {
        if let Some(md) = self.md.as_mut() {
            md.bind_ai(scope, source);
        }
    }

    // -- polling -------------------------------------------------------------------

    /// Drains the worker and runs the timer; what the owner has to do.
    pub(crate) fn poll(&mut self) -> ItemEffect {
        self.conv.poll();
        if let Some(done) = self.conv.take_write_done() {
            self.on_write_done(done);
        }
        if self.conv.wants_refresh() {
            self.conv.refresh_quietly();
        }
        self.ensure_composer();
        if self.auto_refreshes() && !self.conv.busy() && self.conv.since_fresh() >= AUTO_REFRESH {
            self.conv.refresh_quietly();
        }
        ItemEffect::None
    }

    // -- the back stack and the seen mark -----------------------------------------------

    /// Leaves the item for another one: this one waits on the stack.
    pub(super) fn push_item(&mut self, target: ItemTarget) {
        register_scope(&target);
        mark_seen(&self.target, &self.conv);
        let conv = Self::open_conversation(&target, &self.factory);
        let old_target = std::mem::replace(&mut self.target, target);
        let old_conv = std::mem::replace(&mut self.conv, conv);
        self.stack.push(Frame {
            target: old_target,
            conv: old_conv,
        });
        self.release_composer();
        self.menu = None;
        self.delete_ask = None;
        self.focus = Focus::Timeline;
    }

    /// Back to the item the view came from.
    pub(super) fn pop_item(&mut self) -> bool {
        let Some(frame) = self.stack.pop() else {
            return false;
        };
        mark_seen(&self.target, &self.conv);
        self.target = frame.target;
        self.conv = frame.conv;
        self.release_composer();
        self.menu = None;
        self.delete_ask = None;
        self.focus = Focus::Timeline;
        true
    }

    /// The composer has the keyboard.
    #[cfg(test)]
    pub(crate) fn composer_focused(&self) -> bool {
        self.focus == Focus::Composer
    }

    /// The composer answers a discussion (`r`, the Reply button).
    #[cfg(test)]
    pub(crate) fn composing_reply(&self) -> bool {
        matches!(self.compose, Compose::Reply { .. })
    }

    /// What the composer holds.
    #[cfg(test)]
    pub(crate) fn composer_text(&self) -> String {
        self.md
            .as_ref()
            .map_or(String::new(), |e| e.text().to_string())
    }

    /// A write finished and nothing is in flight (draining it once).
    #[cfg(test)]
    pub(crate) fn write_idle(&self) -> bool {
        self.conv.take_idle_for_tests()
    }

    /// The timeline: header and identity are in and nothing is being read.
    #[cfg(test)]
    pub(crate) fn loaded(&self) -> bool {
        self.conv.header.is_some() && self.conv.identity.is_some() && !self.conv.busy()
    }

    #[cfg(test)]
    pub(crate) fn conv(&self) -> &Conversation {
        &self.conv
    }

    #[cfg(test)]
    pub(crate) fn depth(&self) -> usize {
        self.stack.len()
    }
}

impl Drop for ItemView {
    fn drop(&mut self) {
        mark_seen(&self.target, &self.conv);
        for f in &self.stack {
            mark_seen(&f.target, &f.conv);
        }
    }
}

/// Tells the hosting clients which repository this item's project is, so its
/// stored settings (the token command, "Post as", the learned bot login)
/// apply to what is posted.
fn register_scope(target: &ItemTarget) {
    if let Some(root) = &target.repo_root {
        gitpoulpi_daemon::hosting_scope::register(root, &target.integration.id, &target.project);
    }
}

/// Leaves what was shown as "seen" in the repository's state.
fn mark_seen(target: &ItemTarget, conv: &Conversation) {
    let (Some(path), Some((at, id))) = (target.state_path(), conv.newest()) else {
        return;
    };
    let mut state = gitpoulpi_session::RepoState::load(&path).unwrap_or_default();
    state.mark_conversation_seen(&target.seen_key(), &at, &id, cu::now_secs());
    let _ = state.save(&path);
}

/// Remembers whether events are shown.
pub(super) fn save_events(target: &ItemTarget, on: bool) {
    let Some(path) = target.state_path() else {
        return;
    };
    let mut state = gitpoulpi_session::RepoState::load(&path).unwrap_or_default();
    state.conversation_events = Some(on);
    let _ = state.save(&path);
}
