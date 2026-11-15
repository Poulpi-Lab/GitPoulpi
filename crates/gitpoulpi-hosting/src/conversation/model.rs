//! The provider-neutral conversation shapes.

use serde::{Deserialize, Serialize};

use crate::review::model::{DiffSide, ReviewState};

/// Whether an item is an issue or a pull / merge request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemKind {
    Issue,
    PullRequest,
}

/// Which item. `number` is GitHub's number or GitLab's `iid`; `id` carries a
/// provider's own key (Jira key, Trello card id, Linear uuid) and is empty
/// where the number is enough.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ItemKey {
    pub kind: ItemKind,
    pub number: u64,
    #[serde(default)]
    pub id: String,
}

impl ItemKey {
    pub fn issue(number: u64) -> Self {
        Self {
            kind: ItemKind::Issue,
            number,
            id: String::new(),
        }
    }

    pub fn pull_request(number: u64) -> Self {
        Self {
            kind: ItemKind::PullRequest,
            number,
            id: String::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActorKind {
    User,
    Bot,
    System,
    #[default]
    Unknown,
}

/// Who wrote a comment or caused an event.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Actor {
    pub login: String,
    /// The name to show: the full name when the host gives one, else the login.
    pub display: String,
    pub kind: ActorKind,
    pub web_url: Option<String>,
}

impl Actor {
    pub fn new(login: impl Into<String>, display: impl Into<String>, kind: ActorKind) -> Self {
        Self {
            login: login.into(),
            display: display.into(),
            kind,
            web_url: None,
        }
    }

    /// The actor of a deleted account or an anonymous author.
    pub fn unknown() -> Self {
        Self::new("ghost", "ghost", ActorKind::Unknown)
    }
}

/// The dialect a text is written in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MarkupFlavor {
    #[default]
    Gfm,
    GitlabMd,
    GiteaMd,
    CommonMark,
    AzureMd,
    BitbucketMd,
    Html,
    JiraWiki,
    Textile,
    YoutrackMd,
    TrelloMd,
    PlainText,
    ObsidianMd,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Markup {
    pub text: String,
    pub flavor: MarkupFlavor,
}

impl Markup {
    pub fn new(text: impl Into<String>, flavor: MarkupFlavor) -> Self {
        Self {
            text: text.into(),
            flavor,
        }
    }
}

/// One reaction kind and how many people gave it (`+1`, `heart`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Reaction {
    pub name: String,
    pub count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Comment {
    pub id: String,
    pub author: Actor,
    pub body: Markup,
    pub created_at: String,
    pub updated_at: Option<String>,
    /// Edited after it was posted.
    pub edited: bool,
    pub reactions: Vec<Reaction>,
    /// The comment this answers (a thread's first comment).
    pub in_reply_to: Option<String>,
    /// The host's discussion id, what `reply_in_discussion` takes.
    pub discussion_id: Option<String>,
    /// Only visible to project members (GitLab internal notes).
    pub internal: bool,
    /// What the host says about the token's rights; `None` = it does not say.
    pub can_edit: Option<bool>,
    pub can_delete: Option<bool>,
    pub web_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EventKind {
    Opened,
    Closed,
    Reopened,
    Merged,
    Labeled {
        added: Vec<String>,
        removed: Vec<String>,
    },
    Assigned {
        added: Vec<String>,
        removed: Vec<String>,
    },
    Renamed {
        from: String,
        to: String,
    },
    CommitsPushed {
        count: u32,
        head: Option<String>,
    },
    CrossReferenced {
        from: String,
        url: Option<String>,
    },
    StatusChanged {
        from: String,
        to: String,
    },
    ReviewRequested {
        who: String,
    },
    /// Anything else, with the host's own wording.
    Other {
        text: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Event {
    pub id: String,
    pub actor: Option<Actor>,
    pub at: String,
    pub kind: EventKind,
}

/// A submitted review: the verdict and its summary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewEntry {
    pub id: String,
    pub author: Actor,
    pub state: ReviewState,
    pub body: Markup,
    pub submitted_at: String,
}

/// A conversation on code: `ReviewThread` with actors, markup and the hunk.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CodeThread {
    pub id: String,
    pub path: String,
    pub line: Option<u32>,
    pub original_line: Option<u32>,
    pub side: DiffSide,
    pub start_line: Option<u32>,
    pub resolved: bool,
    pub outdated: bool,
    pub resolvable: bool,
    /// The diff excerpt the thread is anchored on, when the host gives it.
    pub diff_hunk: Option<String>,
    /// Oldest first.
    pub comments: Vec<Comment>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "entry", rename_all = "snake_case")]
pub enum TimelineEntry {
    Comment(Comment),
    Review(ReviewEntry),
    Thread(CodeThread),
    Event(Event),
}

impl TimelineEntry {
    /// When it happened (a thread: when it was opened).
    pub fn at(&self) -> &str {
        match self {
            Self::Comment(c) => &c.created_at,
            Self::Review(r) => &r.submitted_at,
            Self::Thread(t) => t.comments.first().map_or("", |c| c.created_at.as_str()),
            Self::Event(e) => &e.at,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemHeader {
    pub key: ItemKey,
    /// What a human types to reference it (`#12`, `!7`).
    pub reference: String,
    pub title: String,
    /// `open`, `closed`, `merged`...
    pub state_label: String,
    pub open: bool,
    pub author: Actor,
    pub body: Markup,
    pub created_at: String,
    pub web_url: String,
    pub labels: Vec<String>,
}

/// Opaque: where the window before the one just read starts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PageCursor(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimelinePage {
    pub header: ItemHeader,
    /// Oldest to newest.
    pub entries: Vec<TimelineEntry>,
    /// The cursor of the window before this one; `None` at the start.
    pub older: Option<PageCursor>,
    /// About how many entries the conversation holds, when known.
    pub total_hint: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimelineOpts {
    /// Include system events (labels, state changes, pushes...).
    pub events: bool,
}

impl Default for TimelineOpts {
    fn default() -> Self {
        Self { events: true }
    }
}
