//! The "show it to the user" protocol: view intents an external agent sends
//! (through `gitpoulpi ui ...` or the MCP `ui_*` tools), the outcomes a TUI
//! window reports back, and the viewer bookkeeping around them.
//! Pure serde, no I/O. see the mcp-show-ui design.
//!
//! # View-only by construction
//!
//! No [`UiIntent`] variant carries an action: none can commit, stage, check
//! out, edit a setting, type into a field or run a command. Every variant
//! only asks a window to *move its view* (select, scroll, open a page,
//! highlight, show a toast). `UiIntent::kind` is an exhaustive `match`, so a
//! new variant cannot compile until it is listed there, and the table below
//! is where its safety is argued.
//!
//! | Intent | What the window does | Never |
//! |---|---|---|
//! | `ShowTab` | activates (or opens) the repository tab | checkout, close |
//! | `ShowHome` | opens Repository Management, fills its filter | opens Browse/Clone/Init |
//! | `ShowCommit` | selects commit(s) in the Log | checkout, unhide refs |
//! | `ShowSearch` | opens the Log find bar (dims, never filters) | |
//! | `ShowRef` | selects a ref row, unfolds its folder | checkout |
//! | `ShowHosting` | selects a PR/issue row | opens the browser, links a provider |
//! | `ShowWip` | selects the WIP row / a file / focuses the composer | types into the composer |
//! | `ShowDiff` | opens a diff at a line, tints a range | stage/discard |
//! | `ShowFile` | opens a file (content, blame, history) at a position | edit |
//! | `ShowConflict` | opens the merge editor on a conflicted file, at a conflict | picks, saves, stages |
//! | `ShowSettings` | opens a Settings page, flashes a row | edits a value |
//! | `ShowPalette` | opens the palette with a query filled in | runs the command |
//! | `ShowHelp` | opens the help overlay | |
//! | `Highlight` | tints a range of the open diff/file | |
//! | `Notify` | shows a toast | takes focus |
//! | `Back` | restores the view snapshot taken before the agent's last intent | |
//! | `Propose` | opens an existing dialog (reword, composer, new branch/tag) pre-filled, marked "✦ Proposed by" | confirms it: only the user's own confirm in that dialog changes anything |
//!
//! `Propose` is the one class that *leads to* a change (plan section 11): it
//! carries a suggestion, never an action, and is separately gated
//! (`[mcp] allow_ui_propose`).

use gitpoulpi_core::domain::{Oid, RepoId, UiControl};
use serde::{Deserialize, Serialize};

mod proposal;
pub use proposal::{
    sanitize_file_text, sanitize_message, Proposal, ProposalRecord, ProposalStatus,
    FILE_TEXT_LIMIT, MESSAGE_LIMIT,
};

/// Identifies one TUI *process*; created once and re-sent by every window
/// the process restarts into, so the daemon can recognise a re-attach.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ViewerId(pub String);

impl std::fmt::Display for ViewerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaneId {
    Refs,
    Log,
    Status,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HostingItem {
    Pr(u64),
    Issue(u64),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiffSide {
    Old,
    #[default]
    New,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiffLayout {
    Inline,
    Split,
}

/// 1-based, inclusive lines; columns are 1-based and optional.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LineRange {
    pub start_line: u32,
    pub end_line: u32,
    #[serde(default)]
    pub start_col: Option<u32>,
    #[serde(default)]
    pub end_col: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiffAt {
    Commit(Oid),
    Unstaged,
    Staged,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileAt {
    Rev(Oid),
    Worktree,
    Index,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileMode {
    #[default]
    Content,
    Blame,
    History,
}

/// Settings pages an agent can address, by stable kebab-case id.
///
/// The first group is the sidebar's pages. The last three are the ids the
/// pages had before the thematic regrouping, kept as aliases so an agent's
/// `ui show settings ui-customization` still lands somewhere sensible
/// (`UiCustomization` on Appearance, `Editor` on Diff & editor,
/// `KeyboardShortcuts` on Keyboard & mouse). A `setting` named with a
/// legacy id, or with `general`, is looked up on every page, since rows
/// moved.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SettingsPage {
    General,
    Sync,
    Profiles,
    Ssh,
    Integrations,
    Ai,
    AgentsMcp,
    Appearance,
    LogGraph,
    DiffEditor,
    OpenIn,
    Images,
    KeyboardMouse,
    GitEncoding,
    RepoAi,
    /// The repository's branching workflow (`.gitpoulpi/workflow.toml`).
    Workflow,
    /// The repository's sparse checkout paths.
    SparseCheckout,
    /// The repository's Git LFS state (initialize, tracked patterns).
    Lfs,
    /// The repository's commit behavior (push after commit, skip hooks).
    Commit,
    /// The repository's merge behavior (squash).
    MergeBehavior,
    /// The repository's commit message template.
    CommitTemplate,
    /// The repository's proactive conflict detection (switch, base branches).
    ConflictPrevention,
    /// The repository's SSH key and HTTPS sign-in (the per-repository
    /// authentication profile).
    SshCredentials,
    /// Legacy alias of `appearance`.
    UiCustomization,
    /// Legacy alias of `diff-editor`.
    Editor,
    /// Legacy alias of `keyboard-mouse`.
    KeyboardShortcuts,
}

impl SettingsPage {
    /// The page ids: the sidebar's order, then the legacy aliases.
    pub const IDS: [&'static str; 26] = [
        "general",
        "sync",
        "profiles",
        "ssh",
        "integrations",
        "ai",
        "agents-mcp",
        "appearance",
        "log-graph",
        "diff-editor",
        "open-in",
        "images",
        "keyboard-mouse",
        "git-encoding",
        "repo-ai",
        "workflow",
        "sparse-checkout",
        "lfs",
        "commit",
        "merge-behavior",
        "commit-template",
        "conflict-prevention",
        "ssh-credentials",
        "ui-customization",
        "editor",
        "keyboard-shortcuts",
    ];

    pub fn parse(id: &str) -> Option<Self> {
        serde_json::from_value(serde_json::Value::String(id.to_string())).ok()
    }

    /// `true` for an id that predates the regrouping (or `general`, which
    /// used to hold most settings): its rows may now live on another page.
    pub fn is_legacy(self) -> bool {
        matches!(
            self,
            SettingsPage::General
                | SettingsPage::UiCustomization
                | SettingsPage::Editor
                | SettingsPage::KeyboardShortcuts
        )
    }
}

/// One view change an agent asks for. See the module docs for why none of
/// these can mutate anything.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum UiIntent {
    ShowTab {
        repo: RepoId,
    },
    ShowHome {
        #[serde(default)]
        filter: Option<String>,
    },
    ShowCommit {
        repo: RepoId,
        commit: Oid,
        #[serde(default)]
        also: Vec<Oid>,
        #[serde(default)]
        files: bool,
        #[serde(default)]
        focus: Option<PaneId>,
    },
    ShowSearch {
        repo: RepoId,
        query: String,
    },
    ShowRef {
        repo: RepoId,
        /// A full (`refs/heads/x`) or short (`main`, `origin/main`) name.
        name: String,
    },
    ShowHosting {
        repo: RepoId,
        item: HostingItem,
    },
    ShowWip {
        repo: RepoId,
        #[serde(default)]
        path: Option<String>,
        #[serde(default)]
        staged: bool,
        #[serde(default)]
        composer: bool,
    },
    ShowDiff {
        repo: RepoId,
        target: DiffAt,
        path: String,
        #[serde(default)]
        line: Option<u32>,
        #[serde(default)]
        side: DiffSide,
        #[serde(default)]
        layout: Option<DiffLayout>,
        #[serde(default)]
        whole_file: Option<bool>,
        #[serde(default)]
        highlight: Option<LineRange>,
    },
    ShowFile {
        repo: RepoId,
        path: String,
        at: FileAt,
        #[serde(default)]
        mode: FileMode,
        #[serde(default)]
        line: Option<u32>,
        #[serde(default)]
        column: Option<u32>,
        #[serde(default)]
        highlight: Option<LineRange>,
    },
    /// Opens the merge conflict editor on a conflicted file and scrolls to
    /// conflict `conflict` (1-based). View-only: nothing is picked.
    ShowConflict {
        repo: RepoId,
        /// Repository-relative path, forward slashes.
        path: String,
        #[serde(default)]
        conflict: Option<u32>,
    },
    ShowSettings {
        #[serde(default)]
        page: Option<SettingsPage>,
        #[serde(default)]
        setting: Option<String>,
    },
    ShowPalette {
        #[serde(default)]
        query: String,
    },
    ShowHelp {
        #[serde(default)]
        topic: Option<String>,
    },
    Highlight {
        range: LineRange,
        #[serde(default = "default_true")]
        scroll: bool,
    },
    Notify {
        message: String,
    },
    Back,
    /// Opens a dialog pre-filled with the agent's suggestion; the user
    /// confirms (or not) in the dialog itself.
    Propose {
        repo: RepoId,
        /// The proposal id (`p-<hex>`) the agent follows in `ui state`.
        id: String,
        proposal: Proposal,
    },
}

fn default_true() -> bool {
    true
}

impl UiIntent {
    /// The snake_case name of the variant, exhaustive on purpose.
    pub fn kind(&self) -> &'static str {
        match self {
            UiIntent::ShowTab { .. } => "show_tab",
            UiIntent::ShowHome { .. } => "show_home",
            UiIntent::ShowCommit { .. } => "show_commit",
            UiIntent::ShowSearch { .. } => "show_search",
            UiIntent::ShowRef { .. } => "show_ref",
            UiIntent::ShowHosting { .. } => "show_hosting",
            UiIntent::ShowWip { .. } => "show_wip",
            UiIntent::ShowDiff { .. } => "show_diff",
            UiIntent::ShowFile { .. } => "show_file",
            UiIntent::ShowConflict { .. } => "show_conflict",
            UiIntent::ShowSettings { .. } => "show_settings",
            UiIntent::ShowPalette { .. } => "show_palette",
            UiIntent::ShowHelp { .. } => "show_help",
            UiIntent::Highlight { .. } => "highlight",
            UiIntent::Notify { .. } => "notify",
            UiIntent::Back => "back",
            UiIntent::Propose { .. } => "propose",
        }
    }

    /// The repository the intent is about, when it names one: the daemon
    /// routes to a window that already shows it.
    pub fn repo(&self) -> Option<&RepoId> {
        match self {
            UiIntent::ShowTab { repo }
            | UiIntent::ShowCommit { repo, .. }
            | UiIntent::ShowSearch { repo, .. }
            | UiIntent::ShowRef { repo, .. }
            | UiIntent::ShowHosting { repo, .. }
            | UiIntent::ShowWip { repo, .. }
            | UiIntent::ShowDiff { repo, .. }
            | UiIntent::ShowFile { repo, .. }
            | UiIntent::ShowConflict { repo, .. }
            | UiIntent::Propose { repo, .. } => Some(repo),
            UiIntent::ShowHome { .. }
            | UiIntent::ShowSettings { .. }
            | UiIntent::ShowPalette { .. }
            | UiIntent::ShowHelp { .. }
            | UiIntent::Highlight { .. }
            | UiIntent::Notify { .. }
            | UiIntent::Back => None,
        }
    }

    /// `Highlight` and `Back` act on whatever the window the same client
    /// last addressed shows, so they follow that window.
    pub fn follows_previous_target(&self) -> bool {
        matches!(self, UiIntent::Highlight { .. } | UiIntent::Back)
    }

    /// `Notify` is a toast: it never takes focus, so a window never defers it.
    pub fn is_toast(&self) -> bool {
        matches!(self, UiIntent::Notify { .. })
    }

    /// Strips terminal-control and bidi characters from every free-text
    /// field the window will draw, and caps their length.
    pub fn sanitize(&mut self) {
        match self {
            UiIntent::ShowHome { filter } => sanitize_opt(filter, QUERY_LIMIT),
            UiIntent::ShowSearch { query, .. } | UiIntent::ShowPalette { query } => {
                *query = sanitize_with_limit(query, QUERY_LIMIT)
            }
            UiIntent::ShowHelp { topic } => sanitize_opt(topic, QUERY_LIMIT),
            UiIntent::ShowSettings { setting, .. } => sanitize_opt(setting, QUERY_LIMIT),
            UiIntent::Notify { message } => *message = sanitize_display(message),
            UiIntent::Propose { id, proposal, .. } => {
                *id = sanitize_with_limit(id, 40);
                proposal.sanitize();
            }
            _ => {}
        }
    }
}

fn sanitize_opt(value: &mut Option<String>, limit: usize) {
    if let Some(v) = value {
        *v = sanitize_with_limit(v, limit);
    }
}

pub const DISPLAY_LIMIT: usize = 200;
pub const QUERY_LIMIT: usize = 100;

/// Makes an agent-supplied string safe to draw: C0/C1 controls (ESC
/// included) and the bidi overrides/isolates are removed (a raw ESC in a
/// toast would be a terminal escape injection), tabs and newlines become a
/// space, and the result is capped at [`DISPLAY_LIMIT`] characters.
pub fn sanitize_display(text: &str) -> String {
    sanitize_with_limit(text, DISPLAY_LIMIT)
}

/// [`sanitize_display`] with a custom cap (queries use [`QUERY_LIMIT`]).
pub fn sanitize_with_limit(text: &str, limit: usize) -> String {
    text.chars()
        .filter_map(|c| match c {
            '\t' | '\n' | '\r' => Some(' '),
            c if c.is_control() => None,
            '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}' => None,
            '\u{200E}' | '\u{200F}' | '\u{061C}' | '\u{FEFF}' => None,
            c => Some(c),
        })
        .take(limit)
        .collect()
}

/// Who an intent comes from, shown to the user ("Shown by claude-code").
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiOrigin {
    /// The MCP `clientInfo.name`, or `"cli"`.
    pub client: String,
    #[serde(default)]
    pub label: Option<String>,
    /// How far the agent's setup lets it move the window by itself; the TUI
    /// honours it (`ask` shows a chip, `off` refuses).
    #[serde(default)]
    pub control: UiControl,
}

/// The `UiOrigin::client` of `gitpoulpi --add`: the user's own command, not
/// an agent's. The window applies it without the "shown by" toast or the
/// agent mark, and defers it only while the user is typing in a form.
pub const ADD_CLIENT: &str = "gitpoulpi-add";

impl UiOrigin {
    /// The origin of `gitpoulpi --add` (see [`ADD_CLIENT`]).
    pub fn add_command() -> Self {
        Self {
            client: ADD_CLIENT.into(),
            label: None,
            control: UiControl::Auto,
        }
    }

    pub fn cli() -> Self {
        Self {
            client: "cli".into(),
            label: None,
            control: UiControl::default(),
        }
    }

    pub fn sanitize(&mut self) {
        self.client = sanitize_with_limit(&self.client, 60);
        if self.client.is_empty() {
            self.client = "agent".into();
        }
        sanitize_opt(&mut self.label, 100);
    }
}

/// Why a window deferred an intent instead of applying it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeferReason {
    /// `ui_control = ask`: the user confirms from a chip.
    Ask,
    /// A modal or text input owns the keyboard.
    InputBusy,
    /// The user pressed a key a moment ago.
    RecentInput,
    Dragging,
}

/// What a viewer reports back for an intent.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum UiOutcome {
    Applied {
        state: UiState,
    },
    Deferred {
        reason: DeferReason,
    },
    Refused {
        reason: String,
    },
    NotFound {
        what: String,
    },
    Partial {
        state: UiState,
        note: String,
    },
    /// A proposal's dialog is open, pre-filled: the user decides. Follow
    /// it in `ui state`'s `proposals` by `id`.
    Proposed {
        id: String,
        state: UiState,
    },
    /// Synthesised by the daemon: no window is attached to route to.
    NoViewer {
        message: String,
    },
}

impl UiOutcome {
    /// CLI exit code: 0 applied/partial, 3 deferred (the user decides), 1
    /// for everything that did not happen.
    pub fn exit_code(&self) -> i32 {
        match self {
            UiOutcome::Applied { .. } | UiOutcome::Partial { .. } | UiOutcome::Proposed { .. } => 0,
            UiOutcome::Deferred { .. } => 3,
            UiOutcome::Refused { .. } | UiOutcome::NotFound { .. } | UiOutcome::NoViewer { .. } => {
                1
            }
        }
    }

    pub fn refused(reason: impl Into<String>) -> Self {
        UiOutcome::Refused {
            reason: reason.into(),
        }
    }
}

/// A snapshot of one window, built by the TUI on demand
/// (`Event::UiStateRequest`) and returned inside `Applied`/`Partial`. The
/// nested view sections are free-form JSON until the TUI side settles them.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiState {
    pub viewer: Option<ViewerId>,
    pub focused: Option<bool>,
    pub repo: Option<String>,
    pub tabs: Vec<UiTab>,
    /// `log | diff | file | editor | home`
    pub central: Option<String>,
    pub focus: Option<String>,
    /// `commit_detail | status`
    pub right_panel: Option<String>,
    /// `palette | help | tab_picker | settings | dialog:<name> | menu`
    pub overlay: Option<String>,
    pub input_busy: bool,
    pub log: Option<serde_json::Value>,
    pub diff: Option<serde_json::Value>,
    pub file: Option<serde_json::Value>,
    pub settings: Option<serde_json::Value>,
    pub wip: Option<serde_json::Value>,
    /// The merge conflict editor, when it is the central view: `{path,
    /// conflict, conflicts, left, card, proposal}`.
    pub merge: Option<serde_json::Value>,
    pub last_intent: Option<serde_json::Value>,
    /// Plain text of the central pane's visible rows (200 lines at most),
    /// only when asked for (`ui state --text`); secret fields are masked.
    pub text: Option<Vec<String>>,
    /// The window process' last proposals, newest first (plan section 11).
    pub proposals: Vec<ProposalRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiTab {
    pub path: String,
    #[serde(default)]
    pub active: bool,
}

/// What a window tells the daemon about itself (`UiAttach`, `UiPresence`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ViewerInfo {
    pub id: ViewerId,
    #[serde(default)]
    pub pid: u32,
    /// The repository the window's active tab shows.
    #[serde(default)]
    pub active_repo: Option<RepoId>,
    /// Every repository open in a tab of this window (the active one included).
    #[serde(default)]
    pub tabs: Vec<RepoId>,
    /// Terminal focus, `None` when the terminal does not report it.
    #[serde(default)]
    pub focused: Option<bool>,
    /// The user's master switch ("Allow AI agents to drive the view").
    #[serde(default = "default_true")]
    pub agent_control: bool,
    /// Unix seconds of the last time the window gained focus; filled in by
    /// the daemon, ignored when a window sends it.
    #[serde(default)]
    pub last_focus_at: Option<i64>,
}

/// Args of `OperationKind::UiShow`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UiShowArgs {
    /// Route to this window instead of choosing one (`ui viewers` lists them).
    #[serde(default)]
    pub viewer: Option<ViewerId>,
    pub intent: UiIntent,
    pub origin: UiOrigin,
    /// How long to wait for the window's answer; capped at
    /// [`UI_ANSWER_TIMEOUT_MS`], which is also the default.
    #[serde(default)]
    pub timeout_ms: Option<u64>,
}

/// What the daemon says when no window is attached. The commands layer
/// adds the repository to it.
pub const NO_VIEWER_MESSAGE: &str = "No GitPoulpi window is attached to this session. \
     Ask the user to run `gitpoulpi` in a terminal.";

/// The default (and maximum) wait for a window's answer.
pub const UI_ANSWER_TIMEOUT_MS: u64 = 8_000;

/// Args of `OperationKind::UiQuery`; the answer is a `Vec<ViewerInfo>` or a
/// [`UiState`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "what", rename_all = "snake_case")]
pub enum UiQueryArgs {
    Viewers,
    /// Asks a window to bring its terminal window to the front (the launcher
    /// shortcut's "GitPoulpi is already open" path). Answer: [`RaiseAnswer`].
    /// `viewer` unset: the most recently focused window, whatever its agent
    /// control switch says (this is the user's own click, not an agent).
    Raise {
        #[serde(default)]
        viewer: Option<ViewerId>,
    },
    State {
        #[serde(default)]
        viewer: Option<ViewerId>,
        #[serde(default)]
        include_text: bool,
    },
}

/// The daemon's answer to [`UiQueryArgs::Raise`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RaiseAnswer {
    /// The window did bring itself forward.
    pub raised: bool,
    /// The window asked, when there was one.
    #[serde(default)]
    pub viewer: Option<ViewerInfo>,
    /// Why not, when `raised` is false.
    #[serde(default)]
    pub reason: Option<String>,
}

/// How long the daemon waits for a window to say it raised itself: the
/// launcher must start a new window quickly when it cannot.
pub const RAISE_ANSWER_TIMEOUT_MS: u64 = 1_500;

/// Args of `OperationKind::UiReport`: the answer to the intent or state
/// request numbered `id`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UiReportArgs {
    pub id: u64,
    pub body: UiReportBody,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum UiReportBody {
    Outcome { outcome: UiOutcome },
    State { state: UiState },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repo() -> RepoId {
        RepoId("/r".into())
    }
    fn oid(n: u8) -> Oid {
        Oid([n; 20])
    }
    fn range() -> LineRange {
        LineRange {
            start_line: 3,
            end_line: 5,
            start_col: Some(2),
            end_col: None,
        }
    }

    /// One value of every `UiIntent` variant, so a new variant is added here
    /// (the `kind()` match forces it to be named, the count below forces it
    /// to be sampled).
    fn every_intent() -> Vec<UiIntent> {
        vec![
            UiIntent::ShowTab { repo: repo() },
            UiIntent::ShowHome {
                filter: Some("x".into()),
            },
            UiIntent::ShowCommit {
                repo: repo(),
                commit: oid(1),
                also: vec![oid(2)],
                files: true,
                focus: Some(PaneId::Status),
            },
            UiIntent::ShowSearch {
                repo: repo(),
                query: "q".into(),
            },
            UiIntent::ShowRef {
                repo: repo(),
                name: "main".into(),
            },
            UiIntent::ShowHosting {
                repo: repo(),
                item: HostingItem::Pr(7),
            },
            UiIntent::ShowWip {
                repo: repo(),
                path: Some("a".into()),
                staged: true,
                composer: true,
            },
            UiIntent::ShowDiff {
                repo: repo(),
                target: DiffAt::Commit(oid(3)),
                path: "src/a.rs".into(),
                line: Some(4),
                side: DiffSide::Old,
                layout: Some(DiffLayout::Split),
                whole_file: Some(true),
                highlight: Some(range()),
            },
            UiIntent::ShowFile {
                repo: repo(),
                path: "src/a.rs".into(),
                at: FileAt::Rev(oid(4)),
                mode: FileMode::Blame,
                line: Some(9),
                column: Some(3),
                highlight: Some(range()),
            },
            UiIntent::ShowSettings {
                page: Some(SettingsPage::UiCustomization),
                setting: Some("ui.minimap".into()),
            },
            UiIntent::ShowPalette { query: "p".into() },
            UiIntent::ShowHelp { topic: None },
            UiIntent::Highlight {
                range: range(),
                scroll: false,
            },
            UiIntent::Notify {
                message: "hi".into(),
            },
            UiIntent::Back,
            UiIntent::ShowConflict {
                repo: repo(),
                path: "src/a.rs".into(),
                conflict: Some(2),
            },
            UiIntent::Propose {
                repo: repo(),
                id: "p-1".into(),
                proposal: Proposal::Reword {
                    commit: oid(5),
                    message: "fix: x\n\nbody".into(),
                },
            },
        ]
    }

    #[test]
    fn every_intent_round_trips_through_json() {
        let all = every_intent();
        assert_eq!(all.len(), 17, "sample every variant");
        let kinds: std::collections::HashSet<_> = all.iter().map(|i| i.kind()).collect();
        assert_eq!(kinds.len(), 17, "kinds are unique");
        for intent in all {
            let json = serde_json::to_value(&intent).unwrap();
            assert_eq!(json["kind"], intent.kind(), "{json}");
            let back: UiIntent = serde_json::from_value(json).unwrap();
            assert_eq!(back, intent);
        }
    }

    #[test]
    fn optional_fields_default_when_absent() {
        let intent: UiIntent = serde_json::from_value(serde_json::json!({
            "kind": "show_diff",
            "repo": "/r",
            "target": "unstaged",
            "path": "a.rs"
        }))
        .unwrap();
        assert!(matches!(
            intent,
            UiIntent::ShowDiff {
                side: DiffSide::New,
                line: None,
                ..
            }
        ));
        let hl: UiIntent = serde_json::from_value(serde_json::json!({
            "kind": "highlight",
            "range": {"start_line": 1, "end_line": 2}
        }))
        .unwrap();
        assert!(matches!(hl, UiIntent::Highlight { scroll: true, .. }));
    }

    /// The wire format has no way to express an action: an unknown `kind`
    /// (a `commit`, a `checkout`...) does not parse.
    #[test]
    fn an_action_is_not_an_intent() {
        for kind in [
            "commit",
            "checkout",
            "stage_all",
            "type_text",
            "run_command",
        ] {
            let parsed = serde_json::from_value::<UiIntent>(serde_json::json!({ "kind": kind }));
            assert!(parsed.is_err(), "{kind} must not parse");
        }
    }

    #[test]
    fn routing_helpers_classify_every_variant() {
        for intent in every_intent() {
            let expects_repo = !matches!(
                intent,
                UiIntent::ShowHome { .. }
                    | UiIntent::ShowSettings { .. }
                    | UiIntent::ShowPalette { .. }
                    | UiIntent::ShowHelp { .. }
                    | UiIntent::Highlight { .. }
                    | UiIntent::Notify { .. }
                    | UiIntent::Back
            );
            assert_eq!(intent.repo().is_some(), expects_repo, "{}", intent.kind());
        }
        assert!(UiIntent::Back.follows_previous_target());
        assert!(!UiIntent::ShowHelp { topic: None }.follows_previous_target());
        assert!(UiIntent::Notify {
            message: "x".into()
        }
        .is_toast());
    }

    #[test]
    fn sanitizer_removes_escapes_controls_and_bidi() {
        assert_eq!(sanitize_display("a\u{1b}[31mred\u{7}"), "a[31mred");
        assert_eq!(sanitize_display("x\u{9b}31m"), "x31m", "C1 CSI is dropped");
        assert_eq!(sanitize_display("ab\u{202E}cd\u{2066}e\u{2069}"), "abcde");
        assert_eq!(sanitize_display("l1\nl2\tl3"), "l1 l2 l3");
        assert_eq!(sanitize_display("\u{7f}ok"), "ok");
        assert_eq!(sanitize_display("héllo ✦"), "héllo ✦", "text is kept");
    }

    #[test]
    fn sanitizer_caps_length_in_characters() {
        let long = "é".repeat(500);
        assert_eq!(sanitize_display(&long).chars().count(), DISPLAY_LIMIT);
        assert_eq!(
            sanitize_with_limit(&long, QUERY_LIMIT).chars().count(),
            QUERY_LIMIT
        );
    }

    #[test]
    fn intent_and_origin_sanitize_every_free_text_field() {
        let mut notify = UiIntent::Notify {
            message: "a\u{1b}b".into(),
        };
        notify.sanitize();
        assert_eq!(
            notify,
            UiIntent::Notify {
                message: "ab".into()
            }
        );
        let mut palette = UiIntent::ShowPalette {
            query: "x".repeat(300),
        };
        palette.sanitize();
        let UiIntent::ShowPalette { query } = palette else {
            unreachable!()
        };
        assert_eq!(query.len(), QUERY_LIMIT);
        let mut origin = UiOrigin {
            client: "\u{1b}".into(),
            label: Some("l\u{202e}".into()),
            control: UiControl::Auto,
        };
        origin.sanitize();
        assert_eq!(origin.client, "agent");
        assert_eq!(origin.label.as_deref(), Some("l"));
    }

    #[test]
    fn settings_page_ids_match_the_serde_names() {
        for id in SettingsPage::IDS {
            let page = SettingsPage::parse(id).unwrap_or_else(|| panic!("{id}"));
            assert_eq!(serde_json::to_value(page).unwrap(), id);
        }
        assert!(SettingsPage::parse("nope").is_none());
    }

    #[test]
    fn outcome_exit_codes_and_wire_shape() {
        let applied = UiOutcome::Applied {
            state: UiState::default(),
        };
        assert_eq!(applied.exit_code(), 0);
        assert_eq!(
            UiOutcome::Deferred {
                reason: DeferReason::Ask
            }
            .exit_code(),
            3
        );
        assert_eq!(UiOutcome::refused("no").exit_code(), 1);
        let json = serde_json::to_value(UiOutcome::refused("no")).unwrap();
        assert_eq!(
            json,
            serde_json::json!({"outcome": "refused", "reason": "no"})
        );
        let back: UiOutcome =
            serde_json::from_value(serde_json::to_value(&applied).unwrap()).unwrap();
        assert_eq!(back, applied);
    }
}
