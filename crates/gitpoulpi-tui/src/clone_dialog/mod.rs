//! The Clone dialog.
//!
//! ```text
//! ╭ Clone a repository ─────────────────────────────────────────────────╮
//! │ CLONE FROM          │ GitLab · corp                                 │
//! │   Clone with URL    │ Where to clone to                             │
//! │ ▸ GitLab · corp     │ C:\Users\me\source\repos█         [ Browse… ] │
//! │   GitHub · github…  │ Repository                                    │
//! │                     │ [ search repositories                       ] │
//! │                     │   team/app                                    │
//! │                     │ ▸ team/tool                                   │
//! │                     │ Full path: C:\Users\me\source\repos\tool      │
//! │                     │ [x] Shallow clone (latest commit only)        │
//! │                     │ [x] Sparse checkout (only some directories)   │
//! │                     │     src docs█                                 │
//! │ Tab next · Enter clone · Esc cancel      [ Cancel ] [ Clone the repo! ] │
//! ╰─────────────────────────────────────────────────────────────────────╯
//! ```
//!
//! "Clone with URL" is always there. Under it, one source per connected
//! GitHub / GitLab profile (nothing unsupported, not even grayed): picking
//! one lists that account's remote repositories -- fetched in the
//! background by the owner (`repo_dialogs`) when the source is first
//! picked -- with a search box narrowing them; choosing a row fills the
//! clone URL. Everything a mouse does has a keyboard twin: `Up`/`Down` in
//! the sources and in the list, `Enter` to choose, `Tab` between parts.
//! Sparse checkout is a checkbox with a field for the directories (spaces or
//! commas); checked, at least one is required. The directories are
//! `CloneOptions::sparse` (cone mode), the history is still complete.
//!
//! Pure: validation, keys, clicks and drawing. The clone itself is
//! `clone_task` (the daemon, off the input loop), started by the owner
//! (`repo_dialogs`) from [`CloneOutcome::Submit`].

mod anchor;
mod dest;
mod dismiss;
mod groups;
mod render;

use crate::keymap::term::TermKeymap;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use gitpoulpi_core::domain::{CloneOptions, CloneProgress};
#[cfg(feature = "hosting")]
use gitpoulpi_hosting::HostedProject;
use gitpoulpi_i18n::msg;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};

use crate::dialog_kit::{self, ButtonState, CheckState};
#[cfg(feature = "hosting")]
use crate::hosting_task::ProjectsState;
#[cfg(not(feature = "hosting"))]
mod no_provider;
#[cfg(not(feature = "hosting"))]
use no_provider::{HostedProject, ProjectsState};

/// A connected GitHub / GitLab profile the dialog can list repositories
/// of.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HostedSource {
    pub(crate) profile_id: String,
    /// `GitLab · corp`.
    pub(crate) label: String,
    /// The connected user's login: their own repositories are the first
    /// section of the list.
    pub(crate) login: Option<String>,
}

/// The profiles worth a source: enabled, with a client, able to list
/// repositories, and *connected* - a user recorded by the connect flow,
/// or a token in the profile's environment variable.
pub(crate) fn hosted_sources(integrations: &[gitpoulpi_config::Integration]) -> Vec<HostedSource> {
    if !crate::build_features::HOSTING {
        // No provider in this build: the dialog is the URL form only.
        return Vec::new();
    }
    integrations
        .iter()
        .filter(|i| {
            let caps = i.kind.capabilities();
            i.enabled
                && caps.implemented
                && crate::build_features::kind_built(i.kind)
                && caps.repos
                && (i.user.is_some()
                    || std::env::var(gitpoulpi_hosting::token_env_var(&i.id))
                        .is_ok_and(|t| !t.trim().is_empty()))
        })
        .map(|i| {
            let name = if i.name.trim().is_empty() {
                i.host().unwrap_or_else(|| i.id.clone())
            } else {
                i.name.clone()
            };
            HostedSource {
                profile_id: i.id.clone(),
                label: format!("{} \u{00B7} {name}", provider_name(i.kind)),
                login: i
                    .user
                    .as_ref()
                    .map(|u| u.login.clone())
                    .filter(|l| !l.is_empty()),
            }
        })
        .collect()
}

/// The index of the source a fresh dialog opens on (0 = Clone with URL):
/// `setting` is `repos.clone_start_source` (`url`, `last`, empty = `last`,
/// or a profile id) and `last` the profile id the dialog last showed
/// (`None` = URL). A profile that is not among `hosted` (removed, disabled,
/// not connected, not a repository source) is URL, silently.
pub(crate) fn start_source_index(
    setting: &str,
    last: Option<&str>,
    hosted: &[HostedSource],
) -> usize {
    let setting = setting.trim();
    let wanted =
        if setting.is_empty() || setting.eq_ignore_ascii_case(gitpoulpi_config::CLONE_START_LAST) {
            last
        } else if setting.eq_ignore_ascii_case(gitpoulpi_config::CLONE_START_URL) {
            None
        } else {
            Some(setting)
        };
    wanted
        .and_then(|id| hosted.iter().position(|h| h.profile_id == id))
        .map_or(0, |at| at + 1)
}

/// `GitHub` / `GitLab` / `Azure DevOps` / `Bitbucket` for a profile kind
/// (the same names as the sidebar's dropdowns).
fn provider_name(kind: gitpoulpi_config::IntegrationKind) -> &'static str {
    use gitpoulpi_config::ApiFamily;
    match kind.family() {
        ApiFamily::Github => "GitHub",
        ApiFamily::Gitlab => "GitLab",
        ApiFamily::AzureDevops => "Azure DevOps",
        ApiFamily::Bitbucket => "Bitbucket",
        ApiFamily::Jira => "Jira",
        ApiFamily::Trello => "Trello",
        ApiFamily::Gitea => kind.label(),
        ApiFamily::Linear => "Linear",
        ApiFamily::YouTrack => "YouTrack",
        ApiFamily::Redmine => "Redmine",
        ApiFamily::Gerrit => "Gerrit",
        ApiFamily::LocalMarkdown => "Local Markdown", // i18n-english
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CloneFocus {
    Sources,
    /// The note under the sources when no hosting account is connected:
    /// a link to the Integrations settings.
    Connect,
    Where,
    Browse,
    Url,
    /// The editable folder name the clone lands in.
    Name,
    Search,
    Repos,
    Shallow,
    Sparse,
    /// The directories of a sparse clone (only while the box is checked).
    SparseDirs,
    Clone,
    Cancel,
}

/// What a click or a hover landed on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CloneHit {
    Source(usize),
    /// A row of the repository list (index into the filtered list).
    Repo(usize),
    Part(CloneFocus),
    /// A button of the destination-exists popup.
    Confirm(ConfirmButton),
}

/// The two buttons of the destination-exists popup.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ConfirmButton {
    /// Clone into it / Open it / Use `<name>-2`, whichever the case offers.
    Go,
    Back,
}

/// Why the popup is open: what the destination holds (see [`dest::DestKind`]).
#[derive(Debug, Clone, PartialEq, Eq)]
struct Confirm {
    kind: dest::DestKind,
    focus: ConfirmButton,
}

/// How long the highlight rests on a source before its repository list is
/// requested (walking Up/Down past three sources asks for none of them).
const SOURCE_REST: Duration = Duration::from_millis(300);

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CloneOutcome {
    Stay,
    Cancel,
    Browse(PathBuf),
    /// The destination already is a clone of the URL: open it as a tab.
    Open(PathBuf),
    /// The note under the sources was chosen: open the Integrations settings.
    OpenIntegrations,
    /// A hosted source was picked and its repositories are not loaded:
    /// the owner fetches them and answers with [`CloneDialog::set_projects`].
    LoadProjects(String),
    Submit {
        url: String,
        dest: PathBuf,
        opts: CloneOptions,
    },
}

/// Rows of the repository list at the dialog's default height; a taller
/// dialog (resized by the user) lists more (`CloneDialog::list_rows`).
const LIST_ROWS: usize = 6;

/// The dialog's height with the repository list at [`LIST_ROWS`].
const HOSTED_HEIGHT: u16 = 29;
/// The fewest rows the list keeps; the dialog never gets shorter than that
/// needs (`WIN`).
const MIN_LIST_ROWS: usize = 1;
/// Dialog rows around the list: the hosted height minus [`LIST_ROWS`].
const ROWS_AROUND_LIST: u16 = HOSTED_HEIGHT - LIST_ROWS as u16;

#[derive(Debug, Clone)]
pub(crate) struct CloneDialog {
    /// 0 = Clone with URL; `n` = the `n`-th hosted source.
    pub source: usize,
    pub where_to: String,
    pub url: String,
    /// The folder created under `where_to`: prefilled from the URL (or
    /// the chosen repository) until the user edits it.
    pub name: String,
    name_edited: bool,
    pub shallow: bool,
    /// Sparse checkout, and the directories typed for it.
    pub sparse: bool,
    pub sparse_dirs: String,
    pub focus: CloneFocus,
    /// A clone is running; its last progress report.
    pub progress: Option<CloneProgress>,
    pub message: Option<String>,
    pub hover: Option<CloneHit>,
    hits: Vec<(Rect, CloneHit)>,
    /// The Where, URL and search fields' caret and selection.
    edit: crate::text_edit::FieldEditor<CloneFocus>,
    hosted: Vec<HostedSource>,
    projects: HashMap<String, ProjectsState>,
    /// Profiles whose shown list is a saved copy, a fresh one on its way.
    refreshing: HashSet<String>,
    /// Profiles whose refresh failed: the saved list stays, with the reason
    /// in the Activity Log.
    refresh_failed: HashMap<String, String>,
    /// The host's fresh list, held while the scrollbar thumb is dragged (a
    /// swap under the pointer would move the rows being dragged).
    pending_fresh: Option<(String, Vec<HostedProject>)>,
    /// What was typed in the repository search.
    search: String,
    /// The highlighted repository of the filtered list (headers are never
    /// highlighted), and the first *row* shown (headers count as rows).
    repo_cursor: usize,
    repo_offset: usize,
    /// Rows of the list as last painted (headers + repositories), and its
    /// scrollbar track and drag.
    list_len: usize,
    /// Rows the list showed at the last paint (more when the dialog is taller).
    list_rows: usize,
    list_bar: Option<crate::scrollbar::Track>,
    bar_drag: crate::scrollbar::BarDrag,
    /// The URL of the repository chosen in a hosted source.
    hosted_url: Option<String>,
    /// Its path (`team/app`), shown as chosen.
    chosen: Option<String>,
    /// What the destination holds, found on a thread (`dest::Probe`).
    probe: Option<dest::Probe>,
    /// Clone was pressed before the probe answered: carry on when it does.
    submit_after_probe: bool,
    /// The destination-exists popup, its frame and its cross.
    confirm: Option<Confirm>,
    confirm_rect: Rect,
    confirm_close: dialog_kit::CloseHit,
    close: dialog_kit::CloseHit,
    /// When the source the highlight rests on asks for its list.
    load_due: Option<Instant>,
    /// The project (and the note to show) a hosted source was opened for
    /// (the Radar's "Clone it, then review"): selected when the list lands.
    target: Option<(String, String)>,
    /// The user (not the opening code) moved to another source since the
    /// last [`Self::take_source_change`].
    source_changed: bool,
}

impl CloneDialog {
    /// What `Ctrl+N` badges: the sources, one repository row, every field,
    /// checkbox and button; only the two buttons while the destination
    /// popup is up.
    pub(crate) fn nav_targets(&self) -> Vec<(Rect, (u16, u16))> {
        let popup = self
            .hits
            .iter()
            .any(|(_, h)| matches!(h, CloneHit::Confirm(_)));
        let mut seen_repo = false;
        dialog_kit::nav_cells(self.hits.iter().filter_map(|(r, hit)| match hit {
            CloneHit::Confirm(_) => Some(*r),
            _ if popup => None,
            CloneHit::Source(_) | CloneHit::Part(_) => Some(*r),
            CloneHit::Repo(_) if !std::mem::replace(&mut seen_repo, true) => Some(*r),
            CloneHit::Repo(_) => None,
        }))
    }

    pub(crate) fn new(clone_root: Option<&Path>) -> Self {
        Self {
            source: 0,
            where_to: clone_root
                .map(|p| p.display().to_string())
                .unwrap_or_default(),
            url: String::new(),
            name: String::new(),
            name_edited: false,
            shallow: false,
            sparse: false,
            sparse_dirs: String::new(),
            focus: CloneFocus::Url,
            progress: None,
            message: None,
            hover: None,
            edit: Default::default(),
            hits: Vec::new(),
            hosted: Vec::new(),
            projects: HashMap::new(),
            refreshing: HashSet::new(),
            refresh_failed: HashMap::new(),
            pending_fresh: None,
            search: String::new(),
            repo_cursor: 0,
            repo_offset: 0,
            list_len: 0,
            list_rows: LIST_ROWS,
            list_bar: None,
            bar_drag: Default::default(),
            hosted_url: None,
            chosen: None,
            probe: None,
            submit_after_probe: false,
            confirm: None,
            confirm_rect: Rect::default(),
            confirm_close: Default::default(),
            close: Default::default(),
            load_due: None,
            target: None,
            source_changed: false,
        }
    }

    /// Opens on the source `repos.clone_start_source` / the remembered
    /// `last` one resolve to (see [`start_source_index`]); a hosted one
    /// asks for its list at once, like a picked source. Call after
    /// [`Self::with_hosted`], and not when the caller picks the source
    /// itself.
    pub(crate) fn with_start_source(mut self, setting: &str, last: Option<&str>) -> Self {
        let at = start_source_index(setting, last, &self.hosted);
        if at == 0 {
            return self;
        }
        self.select_source(at);
        self.source_changed = false;
        if self.hosted_mode() {
            self.load_due = Some(Instant::now());
            self.focus = CloneFocus::Search;
        }
        self
    }

    /// The profile id to remember (`None` = URL) when the user changed the
    /// source since the last call; `None` outside that case.
    pub(crate) fn take_source_change(&mut self) -> Option<Option<String>> {
        if !std::mem::take(&mut self.source_changed) {
            return None;
        }
        Some(self.hosted_source().map(|h| h.profile_id.clone()))
    }

    /// Starts with `url` typed in (the Radar's "Clone it, then review"): the
    /// folder name follows from it.
    pub(crate) fn with_url(mut self, url: &str) -> Self {
        self.url = url.to_string();
        self.sync_name();
        self
    }

    /// Opens in the connected-account mode of `profile_id` with `project`
    /// (`owner/name`) to be selected once the account's list has loaded,
    /// `note` shown under the form. The list is asked for at once. Returns
    /// whether the profile is a source of this dialog; if not, nothing
    /// changes (the URL prefill stands).
    pub(crate) fn with_target(mut self, profile_id: &str, project: &str, note: String) -> Self {
        let Some(at) = self.hosted.iter().position(|h| h.profile_id == profile_id) else {
            return self;
        };
        self.select_source(at + 1);
        if !self.hosted_mode() {
            return self;
        }
        self.source_changed = false;
        self.message = Some(note.clone());
        self.target = Some((project.to_string(), note));
        self.load_due = Some(Instant::now());
        self.focus = CloneFocus::Clone;
        self
    }

    /// Whether the dialog is in a connected-account mode.
    #[cfg(all(test, feature = "github", feature = "gitlab"))]
    pub(crate) fn in_account_mode(&self) -> bool {
        self.hosted_mode()
    }

    /// The connected profiles listed under "Clone with URL".
    pub(crate) fn with_hosted(mut self, hosted: Vec<HostedSource>) -> Self {
        self.hosted = hosted;
        self
    }

    /// Typed or picked something worth keeping.
    fn has_work(&self) -> bool {
        !self.url.is_empty()
            || self.name_edited
            || self.hosted_url.is_some()
            || !self.search.is_empty()
            || !self.sparse_dirs.is_empty()
    }

    pub(crate) fn busy(&self) -> bool {
        self.progress.is_some()
    }

    fn sources_len(&self) -> usize {
        1 + self.hosted.len()
    }

    fn source_label(&self, index: usize) -> String {
        match index.checked_sub(1) {
            None => msg::clone::SOURCE_URL.get().to_string(),
            Some(i) => self
                .hosted
                .get(i)
                .map(|h| h.label.clone())
                .unwrap_or_default(),
        }
    }

    /// The hosted source in use, if the source picked is one.
    fn hosted_source(&self) -> Option<&HostedSource> {
        self.source.checked_sub(1).and_then(|i| self.hosted.get(i))
    }

    fn hosted_mode(&self) -> bool {
        self.hosted_source().is_some()
    }

    /// The URL a clone would use: the typed one, or the chosen
    /// repository's.
    fn effective_url(&self) -> &str {
        if self.hosted_mode() {
            self.hosted_url.as_deref().unwrap_or("")
        } else {
            self.url.trim()
        }
    }

    /// Answers a [`CloneOutcome::LoadProjects`] (a plain answer: no saved
    /// copy, the token not rejected).
    #[cfg_attr(not(feature = "hosting"), allow(dead_code))]
    pub(crate) fn set_projects(
        &mut self,
        profile_id: &str,
        result: Result<Vec<HostedProject>, String>,
    ) {
        self.set_projects_answer(profile_id, result, false);
    }

    /// The saved copy of a profile's list: shown at once while the host's
    /// answer is on its way. Ignored once something else is showing.
    #[cfg_attr(not(feature = "hosting"), allow(dead_code))]
    pub(crate) fn set_cached_projects(&mut self, profile_id: &str, list: Vec<HostedProject>) {
        if !matches!(
            self.projects.get(profile_id),
            None | Some(ProjectsState::Loading)
        ) {
            return;
        }
        let has_target = self.target.as_ref().is_some_and(|(project, _)| {
            list.iter()
                .any(|p| p.full_name.eq_ignore_ascii_case(project))
        });
        self.projects
            .insert(profile_id.to_string(), ProjectsState::Ready(list));
        self.refreshing.insert(profile_id.to_string());
        self.refresh_failed.remove(profile_id);
        // A target missing from the copy waits for the host's list.
        if has_target {
            self.settle_target(profile_id);
        }
    }

    /// The host's answer for a profile. `unauthorized`: the token was
    /// rejected, so a saved copy must not stand in for the list.
    #[cfg_attr(not(feature = "hosting"), allow(dead_code))]
    pub(crate) fn set_projects_answer(
        &mut self,
        profile_id: &str,
        result: Result<Vec<HostedProject>, String>,
        unauthorized: bool,
    ) {
        let showing = matches!(self.projects.get(profile_id), Some(ProjectsState::Ready(_)));
        match result {
            Ok(fresh) if showing => {
                let on_it = self
                    .hosted_source()
                    .is_some_and(|s| s.profile_id == profile_id);
                if on_it && self.bar_drag.held() {
                    self.pending_fresh = Some((profile_id.to_string(), fresh));
                } else {
                    self.apply_fresh(profile_id, fresh);
                }
            }
            Ok(list) => {
                self.refreshing.remove(profile_id);
                self.projects
                    .insert(profile_id.to_string(), ProjectsState::Ready(list));
                self.settle_target(profile_id);
            }
            Err(why) => {
                crate::activity_log::report_problem(
                    crate::activity_log::ProblemSource::Hosting,
                    &why,
                );
                self.refreshing.remove(profile_id);
                self.pending_fresh = None;
                if showing && !unauthorized {
                    self.refresh_failed.insert(profile_id.to_string(), why);
                } else {
                    self.refresh_failed.remove(profile_id);
                    self.message = Some(msg::clone::list_failed(&why));
                    self.projects
                        .insert(profile_id.to_string(), ProjectsState::Failed(why));
                    self.settle_target(profile_id);
                }
            }
        }
    }

    /// Swaps the shown list for the host's, keeping the highlighted project
    /// highlighted and the viewport on the same projects (see [`anchor`]),
    /// and the chosen project chosen unless it left the host.
    fn apply_fresh(&mut self, profile_id: &str, fresh: Vec<HostedProject>) {
        let on_it = self
            .hosted_source()
            .is_some_and(|s| s.profile_id == profile_id);
        let held = on_it.then(|| {
            let cursor = self
                .filtered()
                .get(self.repo_cursor)
                .map(|p| p.full_name.to_lowercase());
            anchor::capture(
                anchor::row_keys(&self.grouped()),
                cursor,
                self.repo_offset,
                self.list_rows,
            )
        });
        self.projects
            .insert(profile_id.to_string(), ProjectsState::Ready(fresh));
        self.refreshing.remove(profile_id);
        self.refresh_failed.remove(profile_id);
        if let Some(held) = held {
            let rows = anchor::row_keys(&self.grouped());
            (self.repo_cursor, self.repo_offset) = anchor::restore(&held, &rows, self.list_rows);
        }
        // Rows moved: neither the hover nor a hit painted before the swap
        // may point at another project.
        self.hover = None;
        self.hits
            .retain(|(_, hit)| !matches!(hit, CloneHit::Repo(_)));
        if on_it {
            if let Some(name) = self.chosen.clone() {
                let url = match self.projects.get(profile_id) {
                    Some(ProjectsState::Ready(list)) => list
                        .iter()
                        .find(|p| p.full_name.eq_ignore_ascii_case(&name))
                        .map(|p| p.clone_url(false).map(str::to_string)),
                    _ => None,
                };
                match url {
                    Some(url) => self.hosted_url = url.or(self.hosted_url.take()),
                    None => {
                        self.chosen = None;
                        self.hosted_url = None;
                        self.sync_name();
                        self.message = Some(msg::clone::chosen_gone(&name));
                    }
                }
            }
        }
        self.settle_target(profile_id);
    }

    /// The list of the target's profile has landed: select the project in
    /// it, else keep the mode with the search on the project.
    fn settle_target(&mut self, profile_id: &str) {
        let on_it = self
            .hosted_source()
            .is_some_and(|s| s.profile_id == profile_id);
        if !on_it {
            return;
        }
        let Some((project, note)) = self.target.take() else {
            return;
        };
        if !matches!(self.projects.get(profile_id), Some(ProjectsState::Ready(_))) {
            return;
        }
        self.search.clear();
        let found = self
            .filtered()
            .iter()
            .position(|p| p.full_name.eq_ignore_ascii_case(&project));
        match found {
            Some(index) => {
                self.choose_repo(index);
                let rows = groups::rows(&self.grouped());
                self.reveal_cursor(&rows);
                if self.chosen.is_some() {
                    self.message = Some(note);
                }
            }
            None => {
                self.search = project.clone();
                self.repo_cursor = 0;
                self.repo_offset = 0;
                self.message = Some(msg::clone::review_project_missing(&project));
            }
        }
    }

    /// The connected profiles listed under "Clone with URL".
    /// The current hosted source's repositories matching the search,
    /// one group per owner (see [`groups::group`]).
    fn grouped(&self) -> Vec<groups::Group<'_>> {
        let Some(source) = self.hosted_source() else {
            return Vec::new();
        };
        match self.projects.get(&source.profile_id) {
            Some(ProjectsState::Ready(list)) => {
                groups::group(list, &self.search, source.login.as_deref())
            }
            _ => Vec::new(),
        }
    }

    /// The matching repositories in display order (section by section):
    /// what the cursor and `CloneHit::Repo` index into.
    fn filtered(&self) -> Vec<&HostedProject> {
        self.grouped().into_iter().flat_map(|g| g.repos).collect()
    }

    fn move_repo_cursor(&mut self, delta: i32) {
        let rows = groups::rows(&self.grouped());
        let len = rows
            .iter()
            .filter(|r| matches!(r, groups::ListRow::Repo(_)))
            .count();
        if len == 0 {
            self.repo_cursor = 0;
            return;
        }
        self.repo_cursor =
            (self.repo_cursor as i32 + delta).clamp(0, (len as i32 - 1).max(0)) as usize;
        self.reveal_cursor(&rows);
    }

    /// Scrolls so the highlighted repository is visible; reaching the first
    /// repository of a section shows its header too.
    fn reveal_cursor(&mut self, rows: &[groups::ListRow]) {
        let Some(at) = rows
            .iter()
            .position(|r| *r == groups::ListRow::Repo(self.repo_cursor))
        else {
            return;
        };
        let head = if at > 0 && matches!(rows[at - 1], groups::ListRow::Header { .. }) {
            at - 1
        } else {
            at
        };
        if head < self.repo_offset {
            self.repo_offset = head;
        } else if at >= self.repo_offset + self.list_rows {
            self.repo_offset = at + 1 - self.list_rows;
        }
    }

    /// Chooses the row `index` of the filtered list: its clone URL fills
    /// the form.
    fn choose_repo(&mut self, index: usize) {
        let picked = self
            .filtered()
            .get(index)
            .map(|p| (p.clone_url(false).map(str::to_string), p.full_name.clone()));
        match picked {
            Some((Some(url), name)) => {
                self.repo_cursor = index;
                self.hosted_url = Some(url);
                self.chosen = Some(name);
                self.sync_name();
                self.message = None;
                self.focus = CloneFocus::Clone;
            }
            Some((None, name)) => {
                self.message = Some(msg::clone::no_clone_url(&name));
            }
            None => {}
        }
    }

    /// The folder the clone lands in: *where* joined with the name the
    /// URL implies. `None` until both are there.
    pub(crate) fn dest(&self) -> Option<PathBuf> {
        let name = self.name.trim();
        let where_to = self.where_to.trim();
        if self.effective_url().is_empty() || where_to.is_empty() || name.is_empty() {
            return None;
        }
        Some(PathBuf::from(where_to).join(name))
    }

    /// Re-derives the folder name from the URL / chosen repository
    /// until the user has edited it.
    fn sync_name(&mut self) {
        if self.name_edited {
            return;
        }
        let url = self.effective_url();
        self.name = if url.is_empty() {
            String::new()
        } else {
            repo_name_from_url(url)
        };
    }

    /// The kind of folder at the destination, once the probe has answered.
    fn dest_kind(&self) -> Option<&dest::DestKind> {
        let dest = self.dest()?;
        self.probe
            .as_ref()
            .filter(|p| p.is_for(&dest, self.effective_url()))
            .and_then(|p| p.kind())
    }

    /// The line under the full path about the target folder, and whether it
    /// is a warning: the folder is being looked at, or it has files in it
    /// so the clone would fail. Nothing here touches the disk (the probe
    /// does, on a thread).
    fn dest_warning(&self) -> Option<(String, bool)> {
        self.dest()?;
        validate_folder_name(self.name.trim()).ok()?;
        if !Path::new(self.where_to.trim()).is_absolute() {
            return None;
        }
        match self.dest_kind() {
            None => Some((msg::clone::DEST_CHECKING.get().to_string(), false)),
            Some(dest::DestKind::NotEmpty { .. } | dest::DestKind::SameRepo) => {
                Some((msg::clone::FOLDER_NOT_EMPTY.get().to_string(), true))
            }
            Some(_) => None,
        }
    }

    /// Starts the probe for the current destination if none is running or
    /// answered for it.
    fn ensure_probe(&mut self) {
        let Some(dest) = self.dest() else { return };
        if validate_folder_name(self.name.trim()).is_err()
            || !Path::new(self.where_to.trim()).is_absolute()
        {
            return;
        }
        let url = self.effective_url().to_string();
        if self.probe.as_ref().is_some_and(|p| p.is_for(&dest, &url)) {
            return;
        }
        self.submit_after_probe = false;
        self.probe = Some(dest::Probe::start(dest, url));
    }

    /// Called every loop turn: carries the probe, a Clone waiting for it,
    /// and the rested source's list request on. Never waits.
    pub(crate) fn poll(&mut self) -> CloneOutcome {
        if !self.bar_drag.held() {
            if let Some((id, fresh)) = self.pending_fresh.take() {
                self.apply_fresh(&id, fresh);
            }
        }
        if self.busy() {
            return CloneOutcome::Stay;
        }
        self.ensure_probe();
        let arrived = self.probe.as_mut().is_some_and(|p| p.resolve());
        if arrived && std::mem::take(&mut self.submit_after_probe) {
            return self.submit();
        }
        if self.load_due.is_some_and(|due| Instant::now() >= due) {
            return self.load_now();
        }
        CloneOutcome::Stay
    }

    /// The moment the highlighted source asks for its list, if it waits.
    pub(crate) fn next_deadline(&self) -> Option<Instant> {
        self.load_due
    }

    /// What Clone would send, or why the button is disabled.
    pub(crate) fn plan(&self) -> Result<(String, PathBuf, CloneOptions), String> {
        let url = self.effective_url();
        if url.is_empty() {
            return Err(if self.hosted_mode() {
                msg::clone::CHOOSE_REPO.get().into()
            } else {
                msg::clone::ENTER_URL.get().into()
            });
        }
        if url.chars().any(char::is_whitespace) {
            return Err(msg::clone::URL_NO_SPACES.get().into());
        }
        let where_to = self.where_to.trim();
        if where_to.is_empty() {
            return Err(msg::clone::CHOOSE_WHERE.get().into());
        }
        if !Path::new(where_to).is_absolute() {
            return Err(msg::clone::WHERE_NOT_ABSOLUTE.get().into());
        }
        validate_folder_name(self.name.trim())?;
        let sparse = if self.sparse {
            Some(crate::sparse_dialog::parse_dirs(&self.sparse_dirs)?)
        } else {
            None
        };
        let dest = self.dest().expect("all checked above");
        Ok((
            url.to_string(),
            dest,
            CloneOptions {
                shallow_depth: self.shallow.then_some(1),
                sparse,
                ..CloneOptions::default()
            },
        ))
    }

    pub(crate) fn set_where(&mut self, dir: &Path) {
        self.where_to = dir.display().to_string();
        self.focus = if self.hosted_mode() {
            CloneFocus::Search
        } else {
            CloneFocus::Url
        };
        self.message = None;
    }

    /// The text field that has the keyboard, if one does.
    fn field_focus(&self) -> Option<CloneFocus> {
        match self.focus {
            CloneFocus::Where => Some(CloneFocus::Where),
            CloneFocus::Name => Some(CloneFocus::Name),
            CloneFocus::Url if !self.hosted_mode() => Some(CloneFocus::Url),
            CloneFocus::Search if self.hosted_mode() => Some(CloneFocus::Search),
            CloneFocus::SparseDirs if self.sparse => Some(CloneFocus::SparseDirs),
            _ => None,
        }
    }

    fn field_text_mut(&mut self, id: CloneFocus) -> &mut String {
        match id {
            CloneFocus::Where => &mut self.where_to,
            CloneFocus::Search => &mut self.search,
            CloneFocus::Name => &mut self.name,
            CloneFocus::SparseDirs => &mut self.sparse_dirs,
            _ => &mut self.url,
        }
    }

    pub(crate) fn paste(&mut self, text: &str) {
        if self.busy() {
            return;
        }
        let clean: String = text.chars().filter(|c| !c.is_control()).collect();
        self.edit.sync(self.field_focus());
        if let Some(id) = self.field_focus() {
            let mut field = std::mem::take(self.field_text_mut(id));
            self.edit.caret_mut(id).insert(&mut field, &clean);
            *self.field_text_mut(id) = field;
            if id == CloneFocus::Search {
                self.repo_cursor = 0;
                self.repo_offset = 0;
            }
            self.after_field_edit(id);
        }
    }

    /// Keeps the folder name in step with an edited field.
    fn after_field_edit(&mut self, id: CloneFocus) {
        match id {
            CloneFocus::Name => self.name_edited = true,
            CloneFocus::Url => self.sync_name(),
            _ => {}
        }
    }

    fn submit(&mut self) -> CloneOutcome {
        if self.busy() {
            return CloneOutcome::Stay;
        }
        match self.plan() {
            Ok((url, dest, opts)) => {
                // A clone this process started into that folder is still
                // going: say so before the probe sees its half folder.
                if crate::clone_task::is_running(&dest) {
                    self.message = Some(gitpoulpi_i18n::msg::repo_dialogs::clone_already(
                        dest.display(),
                    ));
                    return CloneOutcome::Stay;
                }
                self.ensure_probe();
                match self.dest_kind().cloned() {
                    // Not answered yet: the poll presses again when it is.
                    None => {
                        self.submit_after_probe = true;
                        CloneOutcome::Stay
                    }
                    Some(dest::DestKind::Missing) => CloneOutcome::Submit { url, dest, opts },
                    Some(kind) => {
                        self.confirm = Some(Confirm {
                            kind,
                            focus: ConfirmButton::Go,
                        });
                        CloneOutcome::Stay
                    }
                }
            }
            Err(reason) => {
                self.message = Some(reason);
                CloneOutcome::Stay
            }
        }
    }

    /// The popup's main button: clone into the empty folder, open the
    /// existing clone, or take the free name.
    fn confirm_go(&mut self) -> CloneOutcome {
        let Some(confirm) = self.confirm.take() else {
            return CloneOutcome::Stay;
        };
        match confirm.kind {
            dest::DestKind::Empty => match self.plan() {
                Ok((url, dest, opts)) => CloneOutcome::Submit { url, dest, opts },
                Err(_) => CloneOutcome::Stay,
            },
            dest::DestKind::SameRepo => self.dest().map_or(CloneOutcome::Stay, CloneOutcome::Open),
            dest::DestKind::NotEmpty { suggestion } => {
                self.name = suggestion;
                self.name_edited = true;
                self.edit.reset();
                // The suggestion was free when probed: no second wait.
                if let Some(dest) = self.dest() {
                    let url = self.effective_url().to_string();
                    self.probe = Some(dest::Probe::known(dest, url, dest::DestKind::Missing));
                }
                self.submit()
            }
            dest::DestKind::Missing => CloneOutcome::Stay,
        }
    }

    /// The popup's keys: Left/Right/Tab move between its two buttons,
    /// Enter presses the focused one, Esc is Back.
    fn confirm_key(&mut self, key: KeyEvent) -> CloneOutcome {
        let Some(confirm) = self.confirm.as_mut() else {
            return CloneOutcome::Stay;
        };
        match key.code {
            KeyCode::Esc => self.confirm = None,
            KeyCode::Left
            | KeyCode::Right
            | KeyCode::Up
            | KeyCode::Down
            | KeyCode::Tab
            | KeyCode::BackTab => {
                confirm.focus = match confirm.focus {
                    ConfirmButton::Go => ConfirmButton::Back,
                    ConfirmButton::Back => ConfirmButton::Go,
                };
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                let focus = confirm.focus;
                return self.confirm_press(focus);
            }
            _ => {}
        }
        CloneOutcome::Stay
    }

    fn confirm_press(&mut self, button: ConfirmButton) -> CloneOutcome {
        match button {
            ConfirmButton::Go => self.confirm_go(),
            ConfirmButton::Back => {
                self.confirm = None;
                CloneOutcome::Stay
            }
        }
    }

    /// Makes `index` the source shown (the form changes with it) without
    /// asking for its list.
    fn select_source(&mut self, index: usize) {
        if index >= self.sources_len() || self.busy() || index == self.source {
            return;
        }
        self.source = index;
        self.source_changed = true;
        self.search.clear();
        self.repo_cursor = 0;
        self.repo_offset = 0;
        self.message = None;
        self.hosted_url = None;
        self.chosen = None;
        self.name_edited = false;
        self.sync_name();
        self.edit.reset();
    }

    /// Asks for the current source's list if it was not asked yet.
    fn load_now(&mut self) -> CloneOutcome {
        self.load_due = None;
        match self.hosted_source().map(|s| s.profile_id.clone()) {
            Some(id) if !self.projects.contains_key(&id) => {
                self.projects.insert(id.clone(), ProjectsState::Loading);
                CloneOutcome::LoadProjects(id)
            }
            _ => CloneOutcome::Stay,
        }
    }

    /// A deliberate pick (click, Enter): shown and loaded now.
    fn pick_source(&mut self, index: usize) -> CloneOutcome {
        if index >= self.sources_len() || self.busy() {
            return CloneOutcome::Stay;
        }
        self.select_source(index);
        self.load_now()
    }

    /// The highlight moved with the keyboard: shown now, its list asked
    /// for only once the highlight rests (see [`SOURCE_REST`]).
    fn highlight_source(&mut self, index: usize) {
        self.select_source(index);
        let waiting = self
            .hosted_source()
            .is_some_and(|s| !self.projects.contains_key(&s.profile_id));
        self.load_due = waiting.then(|| Instant::now() + SOURCE_REST);
    }

    /// Leaves the sources for the part that takes the next input: the
    /// search (hosted) or the URL field. Asks for the list now.
    fn leave_sources(&mut self) -> CloneOutcome {
        let outcome = self.load_now();
        self.focus = if self.hosted_mode() {
            CloneFocus::Search
        } else {
            CloneFocus::Url
        };
        outcome
    }

    /// Moves the keyboard to the sources (or the connect note), from any part.
    pub(crate) fn focus_sources(&mut self) {
        if self.busy() || !crate::build_features::HOSTING {
            return;
        }
        self.focus = if self.hosted.is_empty() {
            CloneFocus::Connect
        } else {
            CloneFocus::Sources
        };
    }

    /// The parts Tab walks through, for the current source.
    fn order(&self) -> Vec<CloneFocus> {
        let mut order = Vec::new();
        if crate::build_features::HOSTING {
            // The left column comes first.
            order.push(if self.hosted.is_empty() {
                CloneFocus::Connect
            } else {
                CloneFocus::Sources
            });
        }
        order.extend([CloneFocus::Where, CloneFocus::Browse]);
        if self.hosted_mode() {
            order.extend([CloneFocus::Search, CloneFocus::Repos]);
        } else {
            order.push(CloneFocus::Url);
        }
        order.push(CloneFocus::Name);
        order.extend([CloneFocus::Shallow, CloneFocus::Sparse]);
        if self.sparse {
            order.push(CloneFocus::SparseDirs);
        }
        order.extend([CloneFocus::Clone, CloneFocus::Cancel]);
        order
    }

    fn activate(&mut self, part: CloneFocus) -> CloneOutcome {
        match part {
            CloneFocus::Browse => CloneOutcome::Browse(PathBuf::from(self.where_to.trim())),
            CloneFocus::Cancel => CloneOutcome::Cancel,
            CloneFocus::Shallow => {
                if !self.busy() {
                    self.shallow = !self.shallow;
                }
                CloneOutcome::Stay
            }
            CloneFocus::Sparse => {
                if !self.busy() {
                    self.sparse = !self.sparse;
                    self.message = None;
                    // Checked: the directories are what to type next.
                    if self.sparse {
                        self.focus = CloneFocus::SparseDirs;
                    }
                }
                CloneOutcome::Stay
            }
            CloneFocus::Sources => self.leave_sources(),
            CloneFocus::Connect => CloneOutcome::OpenIntegrations,
            CloneFocus::Search | CloneFocus::Repos => {
                // Enter on the list (or the search) chooses the highlighted
                // repository; Enter again clones.
                if !self.busy() {
                    self.choose_repo(self.repo_cursor);
                }
                CloneOutcome::Stay
            }
            CloneFocus::Where
            | CloneFocus::Url
            | CloneFocus::Name
            | CloneFocus::SparseDirs
            | CloneFocus::Clone => self.submit(),
        }
    }

    pub(crate) fn on_key(&mut self, key: KeyEvent) -> CloneOutcome {
        if self.confirm.is_some() {
            return self.confirm_key(key);
        }
        let shift = key.modifiers.contains(KeyModifiers::SHIFT);
        // The fields' editing keys, through the shared core: caret,
        // selection, Ctrl+A/U/K, Ctrl+C/X.
        self.edit.sync(self.field_focus());
        if let Some(id) = self.field_focus().filter(|_| !self.busy()) {
            let mut field = std::mem::take(self.field_text_mut(id));
            let handled = self.edit.dialog_key(id, &mut field, &key);
            *self.field_text_mut(id) = field;
            if let Some((edit, copied)) = handled {
                if edit == crate::text_edit::KeyEdit::Edited {
                    self.message = None;
                    if id == CloneFocus::Search {
                        self.repo_cursor = 0;
                        self.repo_offset = 0;
                    }
                    self.after_field_edit(id);
                }
                if copied.is_some() {
                    self.message = copied;
                }
                return CloneOutcome::Stay;
            }
        }
        let in_list = matches!(self.focus, CloneFocus::Search | CloneFocus::Repos);
        // j/k/g/G walk the two lists like the arrows and Home/End, only
        // where no text is typed (the search field keeps its letters).
        let key = vim_nav(
            &key,
            matches!(self.focus, CloneFocus::Sources | CloneFocus::Repos),
        );
        match key.code {
            KeyCode::Esc => return CloneOutcome::Cancel,
            KeyCode::Up | KeyCode::Down | KeyCode::Home | KeyCode::End
                if self.focus == CloneFocus::Sources =>
            {
                let last = self.sources_len() - 1;
                let next = match key.code {
                    KeyCode::Down => (self.source + 1).min(last),
                    KeyCode::Up => self.source.saturating_sub(1),
                    KeyCode::Home => 0,
                    _ => last,
                };
                self.highlight_source(next);
            }
            KeyCode::Right | KeyCode::Tab if self.focus == CloneFocus::Sources && !shift => {
                return self.leave_sources();
            }
            KeyCode::Up | KeyCode::Down if in_list => {
                self.move_repo_cursor(if key.code == KeyCode::Down { 1 } else { -1 });
            }
            KeyCode::Home | KeyCode::End if self.focus == CloneFocus::Repos => {
                self.move_repo_cursor(if key.code == KeyCode::End {
                    i32::MAX / 2
                } else {
                    i32::MIN / 2
                });
            }
            KeyCode::PageUp | KeyCode::PageDown if in_list => {
                let step = self.list_rows as i32;
                self.move_repo_cursor(if key.code == KeyCode::PageDown {
                    step
                } else {
                    -step
                });
            }
            KeyCode::Tab | KeyCode::BackTab | KeyCode::Down | KeyCode::Up => {
                let forward =
                    matches!(key.code, KeyCode::Down) || (key.code == KeyCode::Tab && !shift);
                let order = self.order();
                let at = order.iter().position(|f| *f == self.focus).unwrap_or(0);
                let next = if forward {
                    (at + 1) % order.len()
                } else {
                    (at + order.len() - 1) % order.len()
                };
                self.focus = order[next];
            }
            _ if crate::keymap::current().is(crate::keymap::Action::DialogBrowse, &key) => {
                return self.activate(CloneFocus::Browse)
            }
            _ if crate::keymap::current().is(crate::keymap::Action::DialogSources, &key) => {
                self.focus_sources();
            }
            KeyCode::Enter => return self.activate(self.focus),
            KeyCode::Char(' ') if self.field_focus().is_none() => return self.activate(self.focus),
            _ => {}
        }
        CloneOutcome::Stay
    }

    pub(crate) fn on_mouse(&mut self, mouse: MouseEvent) -> CloneOutcome {
        // The popup owns the pointer: nothing behind it hovers or clicks.
        if self.confirm.is_some() {
            return self.confirm_mouse(mouse);
        }
        // The list's scrollbar: thumb grab and drag, track press, arrows.
        if self.hosted_mode() {
            if let Some(top) =
                self.bar_drag
                    .on_mouse(self.list_bar, mouse, self.list_len, self.repo_offset)
            {
                self.repo_offset = top;
                return CloneOutcome::Stay;
            }
        }
        let hit = dialog_kit::hit_at(&self.hits, mouse.column, mouse.row);
        // A drag selecting in a field, and its release.
        if !matches!(mouse.kind, MouseEventKind::Down(_)) && self.field_mouse(mouse) {
            return CloneOutcome::Stay;
        }
        match mouse.kind {
            MouseEventKind::Moved | MouseEventKind::Drag(_) => self.hover = hit,
            MouseEventKind::ScrollUp | MouseEventKind::ScrollDown if self.hosted_mode() => {
                self.move_repo_cursor(if mouse.kind == MouseEventKind::ScrollDown {
                    1
                } else {
                    -1
                });
            }
            MouseEventKind::Down(MouseButton::Left) => match hit {
                Some(CloneHit::Source(i)) => {
                    self.focus = CloneFocus::Sources;
                    return self.pick_source(i);
                }
                Some(CloneHit::Repo(i)) => {
                    self.focus = CloneFocus::Repos;
                    if !self.busy() {
                        self.choose_repo(i);
                    }
                }
                Some(CloneHit::Part(part)) => {
                    self.focus = part;
                    if !matches!(
                        part,
                        CloneFocus::Where
                            | CloneFocus::Url
                            | CloneFocus::Name
                            | CloneFocus::Search
                            | CloneFocus::SparseDirs
                    ) {
                        return self.activate(part);
                    }
                    self.field_mouse(mouse);
                }
                None | Some(CloneHit::Confirm(_)) => {}
            },
            _ => {}
        }
        CloneOutcome::Stay
    }

    fn confirm_mouse(&mut self, mouse: MouseEvent) -> CloneOutcome {
        let hit = dialog_kit::hit_at(&self.hits, mouse.column, mouse.row);
        match mouse.kind {
            MouseEventKind::Moved | MouseEventKind::Drag(_) => {
                self.hover = hit;
                self.confirm_close.set_hover(mouse.column, mouse.row);
            }
            MouseEventKind::Down(MouseButton::Left) => {
                // The cross and a click outside the popup are Back.
                let outside = !self.confirm_rect.contains((mouse.column, mouse.row).into());
                if hit.is_none() && (outside || self.confirm_close.hit(mouse.column, mouse.row)) {
                    return self.confirm_press(ConfirmButton::Back);
                }
                if let Some(CloneHit::Confirm(button)) = hit {
                    if let Some(confirm) = self.confirm.as_mut() {
                        confirm.focus = button;
                    }
                    return self.confirm_press(button);
                }
            }
            _ => {}
        }
        CloneOutcome::Stay
    }

    /// Hands a mouse event to the fields' editor; see
    /// `FieldEditor::dialog_mouse`.
    fn field_mouse(&mut self, mouse: MouseEvent) -> bool {
        self.edit.sync(self.field_focus());
        let id = self.field_focus();
        let text = id
            .map(|id| self.field_text_mut(id).clone())
            .unwrap_or_default();
        self.edit
            .dialog_mouse(id, &text, mouse.kind, mouse.column, mouse.row)
    }

    #[cfg(test)]
    #[cfg_attr(not(feature = "hosting"), allow(dead_code))]
    fn hit_rect(&self, hit: CloneHit) -> Rect {
        self.hits
            .iter()
            .find(|(_, h)| *h == hit)
            .map(|(r, _)| *r)
            .unwrap_or_else(|| panic!("{hit:?} not drawn"))
    }
}

/// The Clone dialog's size in a frame: taller when hosted sources add a
/// repository list.
pub(crate) fn clone_rect(frame: Rect) -> Rect {
    WIN.place(
        frame,
        dialog_kit::centered(frame, frame.width.saturating_sub(4).min(110), 22),
    )
}

/// Resizable: a taller dialog lists more repositories, a wider one gives the
/// fields and the rows more room.
const WIN: dialog_kit::Win =
    dialog_kit::Win::resizable("clone", 70, ROWS_AROUND_LIST + MIN_LIST_ROWS as u16);

/// [`clone_rect`] for a dialog that lists a hosted account's repositories.
pub(crate) fn clone_rect_hosted(frame: Rect) -> Rect {
    WIN.place(
        frame,
        dialog_kit::centered(frame, frame.width.saturating_sub(4).min(110), HOSTED_HEIGHT),
    )
}

impl CloneDialog {
    /// The rectangle this dialog draws in, within `frame`.
    pub(crate) fn rect(&self, frame: Rect) -> Rect {
        if self.hosted.is_empty() {
            clone_rect(frame)
        } else {
            clone_rect_hosted(frame)
        }
    }
}

/// Why `name` cannot be a single folder name, if it cannot.
pub(crate) fn validate_folder_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err(msg::clone::NAME_EMPTY.get().into());
    }
    if name.contains(['/', '\\']) {
        return Err(msg::clone::NAME_SEPARATOR.get().into());
    }
    if let Some(c) = name
        .chars()
        .find(|c| c.is_control() || matches!(c, ':' | '*' | '?' | '"' | '<' | '>' | '|'))
    {
        return Err(msg::clone::name_char(format!("{c:?}")));
    }
    if name == "." || name == ".." {
        return Err(msg::clone::NAME_DOTS.get().into());
    }
    if name.ends_with(['.', ' ']) {
        return Err(msg::clone::NAME_TRAILING.get().into());
    }
    Ok(())
}

/// The folder name `git clone <url>` would choose: the last path segment
/// of the URL (or of a local path, either separator), minus `.git`.
pub(crate) fn repo_name_from_url(url: &str) -> String {
    let trimmed = url.trim().trim_end_matches(['/', '\\']);
    let last = trimmed
        .rsplit(['/', '\\', ':'])
        .find(|s| !s.is_empty())
        .unwrap_or("repo");
    let name = last.strip_suffix(".git").unwrap_or(last);
    if name.is_empty() {
        "repo".to_string()
    } else {
        name.to_string()
    }
}

/// Maps plain `j`/`k`/`g`/`G` to Down/Up/Home/End when `on` (the focus is
/// a list, not a text field); any other key comes back unchanged.
fn vim_nav(key: &KeyEvent, on: bool) -> KeyEvent {
    let plain = !key
        .modifiers
        .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT);
    let code = match key.code {
        KeyCode::Char('j') if on && plain => KeyCode::Down,
        KeyCode::Char('k') if on && plain => KeyCode::Up,
        KeyCode::Char('g') if on && plain => KeyCode::Home,
        KeyCode::Char('G') if on && plain => KeyCode::End,
        _ => return *key,
    };
    KeyEvent::new(code, KeyModifiers::NONE)
}

#[cfg(test)]
mod tests;
