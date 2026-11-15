//! The conversation view of an issue or a pull / merge request, read-only
//! (the write slice adds sending, editing and deleting).
//!
//! A full-window modal owned by `RepoDialogs` (`RepoDialog::Item`), like the
//! review and the board: keys, mouse, paste and the per-frame poll all arrive
//! there. The timeline is the shared widget (`conversation_ui`); this module
//! is the surface around it: the frame with its close cross, the header, the
//! "Posting as ..." line, the composer dock (Write / Preview / Split through
//! `md_editor`, its Send greyed with the reason until writing lands), the
//! menu of an entry's links and the **Back stack**.
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
use crate::conversation_ui::{self as cu, Conversation};
use crate::dialog_kit;
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
    pub(super) menu: Option<LinkMenu>,
    pub(super) guard: dialog_kit::DiscardGuard,
    pub(super) hits: Vec<(Rect, Hit)>,
    pub(super) hover: Option<Hit>,
    pub(super) message: Option<String>,
    /// The frame of the last paint.
    pub(super) frame: Rect,
    /// Where the composer's text field last sat (a click there focuses it).
    pub(super) composer_rect: Rect,
}

impl ItemView {
    /// Opens the view and starts the read.
    pub(crate) fn open(target: ItemTarget, factory: Arc<dyn ClientFactory>) -> Self {
        let conv = Self::open_conversation(&target, &factory);
        Self {
            target,
            factory,
            conv,
            stack: Vec::new(),
            focus: Focus::Timeline,
            md: None,
            composer_open: true,
            menu: None,
            guard: Default::default(),
            hits: Vec::new(),
            hover: None,
            message: None,
            frame: Rect::default(),
            composer_rect: Rect::default(),
        }
    }

    fn open_conversation(target: &ItemTarget, factory: &Arc<dyn ClientFactory>) -> Conversation {
        let (seen, events) = match target.state_path() {
            Some(path) => {
                let state = gitpoulpi_session::RepoState::load(&path).unwrap_or_default();
                (
                    state
                        .conversation_seen
                        .get(&target.seen_key())
                        .map(|s| s.at.clone())
                        .filter(|a| !a.is_empty()),
                    state.conversation_events.unwrap_or(true),
                )
            }
            None => (None, true),
        };
        let cache_dir = target
            .state_path()
            .and_then(|p| p.parent().map(PathBuf::from));
        Conversation::new(
            Arc::clone(factory),
            target.source(),
            cache_dir,
            target.md_context(),
            seen,
            events,
        )
    }

    // -- state the owner and the palette read ----------------------------------

    pub(crate) fn busy(&self) -> bool {
        // A read in flight keeps the frame rate up so its spinner can appear
        // once it has run past 150 ms (the timer's quiet reads do not count).
        self.conv.busy() && !self.conv.is_quiet()
    }

    /// When the view next wants a frame without input (the 60 s refresh).
    pub(crate) fn next_deadline(&self) -> Option<std::time::Instant> {
        self.auto_refreshes()
            .then(|| std::time::Instant::now() + AUTO_REFRESH.saturating_sub(self.conv.since_fresh()))
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
        self.focus != Focus::Composer && self.menu.is_none() && !self.guard.is_open()
    }

    /// Why Send does nothing now (`None`: it would send).
    pub(super) fn send_reason(&self) -> Option<&'static str> {
        use gitpoulpi_i18n::msg::item_view as m;
        if !cu::WRITING {
            return Some(m::WHY_WRITING_LATER.get());
        }
        if self.conv.caps.is_none() {
            return Some(m::WHY_NOT_LOADED.get());
        }
        if !self.conv.can_comment() {
            return Some(m::WHY_NO_COMMENT.get());
        }
        if self.conv.is_offline() {
            return Some(m::WHY_OFFLINE.get());
        }
        if self.md.as_ref().is_none_or(|e| e.is_blank()) {
            return Some(m::WHY_EMPTY.get());
        }
        None
    }

    pub(crate) fn avail(&self) -> ItemAvail {
        use gitpoulpi_i18n::msg::item_view as m;
        let has_entry = self.conv.selected_comment().is_some();
        let edit = if !self.conv.caps.as_ref().is_some_and(|c| c.edit_own) {
            Some(m::WHY_NO_EDIT.get())
        } else if !has_entry || !self.conv.can_edit_selected() {
            Some(m::WHY_NOT_YOURS.get())
        } else if !cu::WRITING {
            Some(m::WHY_WRITING_LATER.get())
        } else {
            None
        };
        let delete = if !self.conv.caps.as_ref().is_some_and(|c| c.delete_own) {
            Some(m::WHY_NO_DELETE.get())
        } else if !has_entry || !self.conv.can_delete_selected() {
            Some(m::WHY_NOT_YOURS.get())
        } else if !cu::WRITING {
            Some(m::WHY_WRITING_LATER.get())
        } else {
            None
        };
        ItemAvail {
            loaded: self.conv.header.is_some(),
            has_back: !self.stack.is_empty(),
            has_older: self.conv.older.is_some(),
            has_unread: self.conv.has_unread(),
            has_links: self
                .conv
                .selected()
                .is_some_and(|i| !i.links.is_empty()),
            has_url: self.conv.selected_url().is_some(),
            reply: self.conv.reply_reason(),
            edit,
            delete,
            send: self.send_reason(),
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
            let md = MdEditor::new(String::new(), self.target.md_context())
                .with_draft(self.target.draft_key());
            self.md = Some(md);
        }
    }

    pub(super) fn composer_dirty(&self) -> bool {
        self.md.as_ref().is_some_and(|e| !e.is_blank())
    }

    // -- polling -------------------------------------------------------------------

    /// Drains the worker and runs the timer; what the owner has to do.
    pub(crate) fn poll(&mut self) -> ItemEffect {
        self.conv.poll();
        self.ensure_composer();
        if self.auto_refreshes() && !self.conv.busy() && self.conv.since_fresh() >= AUTO_REFRESH {
            self.conv.refresh_quietly();
        }
        ItemEffect::None
    }

    // -- the back stack and the seen mark -----------------------------------------------

    /// Leaves the item for another one: this one waits on the stack.
    pub(super) fn push_item(&mut self, target: ItemTarget) {
        mark_seen(&self.target, &self.conv);
        let conv = Self::open_conversation(&target, &self.factory);
        let old_target = std::mem::replace(&mut self.target, target);
        let old_conv = std::mem::replace(&mut self.conv, conv);
        self.stack.push(Frame {
            target: old_target,
            conv: old_conv,
        });
        self.md = None;
        self.menu = None;
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
        self.md = None;
        self.menu = None;
        self.focus = Focus::Timeline;
        true
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
