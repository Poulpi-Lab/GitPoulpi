//! The conversation of a local note: the comments kept in the note file
//! itself (see [`super::comments`] for the format). No network.
//!
//! Only the local author's own comments can be edited or deleted: the
//! author is the profile's "me" (its `local.me` setting, else the connected
//! user, else the profile name), the same name "assigned to me" uses.
//! Nothing here asks the repository: the host has no repository.

use super::client::LocalMarkdownClient;
use super::comments::{self, Entry, Parsed};
use super::frontmatter::Document;
use super::store::{iso_datetime, write_atomic, Issue};
use crate::conversation::time::iso_secs;
use crate::conversation::tracker::{check_body, require_issue, window_of};
use crate::conversation::{
    Actor, ActorKind, Comment, IdentityKind, ItemHeader, ItemKey, ItemKind, Markup, MarkupFlavor,
    PageCursor, PostingIdentity, TimelineEntry, TimelineOpts, TimelinePage, TokenSource,
};
use crate::markup::sanitize;
use crate::{Error, Result};

fn local(msg: impl Into<String>) -> Error {
    Error::Local(msg.into())
}

fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn stamp(secs: i64) -> String {
    format!("{}Z", iso_datetime(secs))
}

/// One note open for a change: its path, document and parsed comments.
struct Note {
    issue: Issue,
    doc: Document,
    parsed: Parsed,
}

impl LocalMarkdownClient {
    fn author_of(&self, name: &str) -> Actor {
        Actor::new(name, name, ActorKind::User)
    }

    fn comment_of(&self, e: &Entry) -> Comment {
        let mine = e.author.eq_ignore_ascii_case(&self.me());
        Comment {
            id: e.id(),
            author: self.author_of(&e.author),
            body: Markup::new(sanitize(&e.body), MarkupFlavor::ObsidianMd),
            created_at: e.at.clone(),
            updated_at: e.edited.clone(),
            edited: e.edited.is_some(),
            reactions: Vec::new(),
            in_reply_to: None,
            discussion_id: None,
            internal: false,
            can_edit: Some(mine),
            can_delete: Some(mine),
            web_url: None,
        }
    }

    fn open_note(&self, project: &str, key: &ItemKey) -> Result<Note> {
        require_issue("A local note", key)?;
        let issue = self.store(project).get(key.number)?;
        let bytes = std::fs::read(&issue.path)
            .map_err(|e| local(format!("cannot read `{}`: {e}", issue.path.display())))?;
        let doc = Document::parse(&String::from_utf8_lossy(&bytes));
        let parsed = comments::parse(&doc.body);
        Ok(Note { issue, doc, parsed })
    }

    /// Writes the note with a new body; the frontmatter is only touched in
    /// its `updated` key (and not at all when the note has none).
    fn save_note(&self, mut note: Note, body: String) -> Result<()> {
        let eol = note.doc.eol;
        if let Some(front) = note.doc.front.as_mut() {
            front.set_scalar("updated", &iso_datetime(now_secs()), eol);
        }
        note.doc.body = body;
        write_atomic(&note.issue.path, &note.doc.render())
    }

    fn mine(&self, e: &Entry, what: &str) -> Result<()> {
        if e.author.eq_ignore_ascii_case(&self.me()) {
            Ok(())
        } else {
            Err(Error::Forbidden {
                what: what.to_string(),
                detail: format!(
                    "only your own comments can be changed here (this one is by {})",
                    e.author
                ),
            })
        }
    }

    pub(super) async fn conv_timeline(
        &self,
        project: &str,
        key: &ItemKey,
        older: Option<&PageCursor>,
        _opts: &TimelineOpts,
    ) -> Result<TimelinePage> {
        let note = self.open_note(project, key)?;
        let settings = self.integration.local.clone().unwrap_or_default();
        let prefix = settings.reference_prefix.trim().to_string();
        let reference = if prefix.is_empty() {
            format!("#{}", note.issue.id)
        } else {
            format!("{prefix}-{}", note.issue.id)
        };
        let total = note.parsed.entries.len() as u32;
        let comments: Vec<Comment> = note
            .parsed
            .entries
            .iter()
            .map(|e| self.comment_of(e))
            .collect();
        let (window, older_cursor) = window_of(comments, older)?;
        let header = ItemHeader {
            key: ItemKey {
                kind: ItemKind::Issue,
                number: note.issue.id,
                id: String::new(),
            },
            reference,
            title: note.issue.title.clone(),
            state_label: note.issue.status.clone(),
            open: !note.issue.status.eq_ignore_ascii_case(settings.done()),
            // A note does not record who opened it.
            author: Actor::new("", "", ActorKind::Unknown),
            body: Markup::new(sanitize(&note.parsed.head), MarkupFlavor::ObsidianMd),
            created_at: note.issue.created.clone(),
            web_url: super::file_url(&note.issue.path),
            labels: note.issue.labels.clone(),
        };
        Ok(TimelinePage {
            header,
            entries: window.into_iter().map(TimelineEntry::Comment).collect(),
            older: older_cursor,
            total_hint: Some(total),
        })
    }

    pub(super) async fn conv_post(
        &self,
        project: &str,
        key: &ItemKey,
        body: &str,
        internal: bool,
    ) -> Result<Comment> {
        check_body("commenting on a note", body, None)?;
        if internal {
            return Err(Error::Unsupported(
                "internal comments: a note's comments are all in the file".into(),
            ));
        }
        let note = self.open_note(project, key)?;
        let author = self.me().replace(['\n', '\r'], " ");
        let mut secs = now_secs();
        let mut entry = Entry {
            author,
            at: stamp(secs),
            edited: None,
            body: body.trim_end().to_string(),
        };
        // The id is `<time>|<author>`: two posts in one second must differ.
        while note.parsed.entries.iter().any(|e| e.id() == entry.id()) {
            secs = iso_secs(&entry.at).unwrap_or(secs) + 1;
            entry.at = stamp(secs);
        }
        let new_body = comments::append(&note.doc.body, &entry, note.doc.eol);
        self.save_note(note, new_body)?;
        Ok(self.comment_of(&entry))
    }

    pub(super) async fn conv_edit(
        &self,
        project: &str,
        key: &ItemKey,
        comment_id: &str,
        body: &str,
    ) -> Result<Comment> {
        check_body("editing a note's comment", body, None)?;
        let note = self.open_note(project, key)?;
        let mut entries = note.parsed.entries.clone();
        let entry = entries
            .iter_mut()
            .find(|e| e.id() == comment_id)
            .ok_or_else(|| local(format!("no comment {comment_id} in this note")))?;
        self.mine(entry, "editing a note's comment")?;
        entry.body = body.trim_end().to_string();
        entry.edited = Some(stamp(now_secs()));
        let done = entry.clone();
        let new_body = comments::rewrite(&note.parsed, &entries, note.doc.eol);
        self.save_note(note, new_body)?;
        Ok(self.comment_of(&done))
    }

    pub(super) async fn conv_delete(
        &self,
        project: &str,
        key: &ItemKey,
        comment_id: &str,
    ) -> Result<()> {
        let note = self.open_note(project, key)?;
        let mut entries = note.parsed.entries.clone();
        let at = entries
            .iter()
            .position(|e| e.id() == comment_id)
            .ok_or_else(|| local(format!("no comment {comment_id} in this note")))?;
        self.mine(&entries[at], "deleting a note's comment")?;
        entries.remove(at);
        let new_body = comments::rewrite(&note.parsed, &entries, note.doc.eol);
        self.save_note(note, new_body)
    }

    /// Who writes: the profile's local author. Never asks a network.
    pub(super) async fn conv_identity(&self) -> Result<PostingIdentity> {
        let mut identity = PostingIdentity::new(
            Some(self.me()),
            IdentityKind::LocalAuthor,
            TokenSource::Profile {
                id: self.integration.id.clone(),
            },
        );
        if identity.login.is_none() {
            identity.display = "local author".to_string();
        }
        Ok(identity)
    }
}
