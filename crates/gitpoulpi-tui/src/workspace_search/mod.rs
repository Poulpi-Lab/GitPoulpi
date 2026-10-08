//! The workspace search (`Ctrl+Alt+F`, `Search: Workspace...`): a fuzzy
//! finder over every repository of the scope - the active workspace's, else
//! the open tabs - for repositories, branches, tags, worktrees, stashes,
//! commit messages, and the pull/merge requests and issues of the hosting
//! caches. see the workspaces design ("Workspace search").
//!
//! - **Matching** is the daemon's `Search` read, one per repository, sent on
//!   the window's connection (`Requester::call_in`, off the render thread),
//!   debounced [`DEBOUNCE`] after the last keystroke. A generation number
//!   tags every fan-out: an answer of an older generation is dropped.
//!   `partial` answers (a commit index still building) are asked again with a
//!   growing delay.
//! - **Repositories and hosting kinds** are matched here, in memory, on each
//!   keystroke: the repositories are the scope itself and the hosting caches
//!   are read once, in a background thread, when the popup opens. Never a
//!   network call.
//! - **Enter** hands a [`UiIntent`] to the window (`UiViewer::go`): the
//!   agent-view machinery switches to (or opens) the tab, carries the intent
//!   across the window restart, and reveals the ref, commit, stash or
//!   request (paging the Log in when a commit is not loaded yet).
//!
//! `model` is the pure half, `scope` the repository choice, `view` the paint.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use gitpoulpi_core::domain::{
    OperationKind, RepoId, SearchHit, SearchHits, SearchKind, SearchRequest, SearchTarget,
};
use gitpoulpi_hosting::cache_search::{self, CachedItems};
use gitpoulpi_ipc::ui::{HostingItem, UiIntent};

use crate::daemon_link::{Reply, Requester};
use crate::repo_switcher::RepoTab;

mod model;
mod scope;
mod view;

pub(crate) use model::{first_item_row, Item, Row};
#[cfg(test)]
pub(crate) use scope::search_scope_in;
pub(crate) use scope::{search_scope, Scope};
pub(crate) use view::{chip_at, close_at, render};

/// How long after the last keystroke the fan-out goes out.
pub(crate) const DEBOUNCE: Duration = Duration::from_millis(60);
/// The in-flight spinner appears after this long (the project's 150 ms rule).
const SPINNER_AFTER: Duration = Duration::from_millis(150);
/// A `partial` answer is asked again after this long, doubling up to the cap.
const RETRY_FIRST: Duration = Duration::from_millis(300);
const RETRY_CAP: Duration = Duration::from_secs(2);
/// And at most this many times for one query.
const RETRY_MAX: u32 = 12;
/// Hits asked of each repository.
const ASK_LIMIT: usize = 40;

/// One repository's last answer.
#[derive(Debug, Clone, Default)]
struct Answer {
    hits: Vec<SearchHit>,
    partial: bool,
}

struct Pending {
    generation: u64,
    repo: usize,
    reply: Reply<SearchHits>,
}

/// The open popup.
pub(crate) struct WorkspaceSearch {
    pub(crate) draft: String,
    /// Index into [`Self::rows`] (always an item row when there is one).
    pub(crate) selected: usize,
    /// Which kinds are searched, in [`ORDER`].
    pub(crate) chips: [bool; 8],
    scope: Scope,
    requester: Requester,
    generation: u64,
    /// The query changed at this moment and has not been sent yet.
    dirty_since: Option<Instant>,
    pending: Vec<Pending>,
    answers: Vec<Option<Answer>>,
    /// The hosting caches, filled by the background thread once.
    caches: Arc<Mutex<Option<Vec<CachedItems>>>>,
    loaded_caches: Vec<CachedItems>,
    retries: u32,
    retry_at: Option<Instant>,
    busy_since: Option<Instant>,
    opened: Instant,
    items: Vec<Item>,
    rows: Vec<Row>,
}

impl WorkspaceSearch {
    /// Opens the popup over the scope of `session` and starts reading the
    /// hosting caches.
    pub(crate) fn open(session: &str, open_tabs: &[RepoTab], requester: Requester) -> Self {
        let scope = search_scope(session, open_tabs);
        let caches = Arc::new(Mutex::new(None));
        let mut search = Self::with_parts(scope, requester, Arc::clone(&caches));
        let roots: Vec<_> = search.scope.repos.iter().map(|r| r.path.clone()).collect();
        std::thread::spawn(move || {
            let state_dir = gitpoulpi_config::Settings::resolve_ambient()
                .map(|s| s.config().session.per_repo_state_dir.clone())
                .unwrap_or_else(|_| gitpoulpi_config::DEFAULT_REPO_DATA_LOCATION.to_string());
            let loaded: Vec<CachedItems> = roots
                .iter()
                .map(|root| {
                    cache_search::load(&gitpoulpi_session::RepoState::hosting_cache_path_for(
                        root, &state_dir,
                    ))
                })
                .collect();
            if let Ok(mut slot) = caches.lock() {
                *slot = Some(loaded);
            }
            crate::redraw::poke();
        });
        search.edited();
        search
    }

    /// The popup over `scope` with the hosting caches arriving in `caches`
    /// (tests fill it themselves).
    pub(crate) fn with_parts(
        scope: Scope,
        requester: Requester,
        caches: Arc<Mutex<Option<Vec<CachedItems>>>>,
    ) -> Self {
        let n = scope.repos.len();
        let mut search = Self {
            draft: String::new(),
            selected: 0,
            chips: [true; 8],
            scope,
            requester,
            generation: 0,
            dirty_since: None,
            pending: Vec::new(),
            answers: vec![None; n],
            caches,
            loaded_caches: Vec::new(),
            retries: 0,
            retry_at: None,
            busy_since: None,
            opened: Instant::now(),
            items: Vec::new(),
            rows: Vec::new(),
        };
        search.rebuild(false);
        search
    }

    pub(crate) fn scope(&self) -> &Scope {
        &self.scope
    }

    pub(crate) fn rows(&self) -> &[Row] {
        &self.rows
    }

    pub(crate) fn items(&self) -> &[Item] {
        &self.items
    }

    #[cfg(test)]
    pub(crate) fn generation(&self) -> u64 {
        self.generation
    }

    /// The highlighted item, if the highlight is on one.
    pub(crate) fn selected_item(&self) -> Option<&Item> {
        match self.rows.get(self.selected)? {
            Row::Item(i) => self.items.get(*i),
            Row::Heading(..) => None,
        }
    }

    /// Whether `row` of the list is a group heading (not selectable).
    pub(crate) fn is_heading(&self, row: usize) -> bool {
        matches!(self.rows.get(row), Some(Row::Heading(..)))
    }

    // -- changes -------------------------------------------------------------

    /// The text changed: the list is recomputed for what is local, the
    /// daemon is asked again after the debounce, the highlight goes back to
    /// the first result.
    pub(crate) fn edited(&mut self) {
        self.dirty_since = Some(Instant::now());
        self.retries = 0;
        self.retry_at = None;
        self.rebuild(false);
    }

    /// Switches the chip at `slot` (an index into [`ORDER`]).
    pub(crate) fn toggle_chip(&mut self, slot: usize) {
        if let Some(chip) = self.chips.get_mut(slot) {
            *chip = !*chip;
            self.edited();
        }
    }

    /// Every kind on again.
    pub(crate) fn reset_chips(&mut self) {
        self.chips = [true; 8];
        self.edited();
    }

    /// The highlight after a list key: `to` came from `popup_list::nav`.
    pub(crate) fn move_to(&mut self, to: usize) {
        self.selected = model::skip_headings(&self.rows, self.selected, to);
    }

    /// Rebuilds items and rows from what is known now. `keep`: the
    /// highlight stays on the same result when it is still listed.
    fn rebuild(&mut self, keep: bool) {
        let parsed = gitpoulpi_core::domain::parse_query(&self.draft);
        let kinds = model::effective_kinds(&self.draft, &self.chips);
        let wants = |k: SearchKind| kinds.contains(&k);
        let mut items: Vec<Item> = Vec::new();
        if wants(SearchKind::Repo) {
            items.extend(model::repo_items(&self.scope, &parsed.text));
        }
        for (repo, answer) in self.answers.iter().enumerate() {
            let Some(answer) = answer else { continue };
            items.extend(
                answer
                    .hits
                    .iter()
                    .filter(|h| wants(h.kind))
                    .cloned()
                    .map(|h| Item::from_hit(h, repo)),
            );
        }
        let hosting: Vec<SearchKind> = kinds.iter().copied().filter(|k| k.is_hosting()).collect();
        if !hosting.is_empty() {
            items.extend(model::hosting_items(
                &self.loaded_caches,
                &hosting,
                &parsed.text,
            ));
        }
        let before = keep.then(|| self.selected_item().cloned()).flatten();
        let (items, rows) = model::group(items);
        self.items = items;
        self.rows = rows;
        self.selected = before
            .and_then(|b| {
                self.rows.iter().position(|r| match r {
                    Row::Item(i) => self.items[*i].same_as(&b),
                    Row::Heading(..) => false,
                })
            })
            .or_else(|| first_item_row(&self.rows))
            .unwrap_or(0);
    }

    // -- the fan-out ---------------------------------------------------------

    /// Asks every repository of the scope for the current query: a new
    /// generation, so whatever is still on its way is stale.
    fn dispatch(&mut self) {
        self.generation += 1;
        self.pending.clear();
        let git: Vec<SearchKind> = model::effective_kinds(&self.draft, &self.chips)
            .into_iter()
            .filter(|k| !k.is_hosting() && *k != SearchKind::Repo)
            .collect();
        if git.is_empty() {
            // Nothing a daemon answers: what it said before no longer applies.
            self.answers.iter_mut().for_each(|a| *a = None);
            self.rebuild(true);
            return;
        }
        let args = serde_json::json!(SearchRequest {
            query: self.draft.clone(),
            kinds: git,
            limit: ASK_LIMIT,
            wait: false,
        });
        for (i, repo) in self.scope.repos.iter().enumerate() {
            let reply = self.requester.call_in::<SearchHits>(
                Some(repo.repo.clone()),
                OperationKind::Search,
                args.clone(),
            );
            self.pending.push(Pending {
                generation: self.generation,
                repo: i,
                reply,
            });
        }
        self.busy_since.get_or_insert_with(Instant::now);
    }

    /// Takes one repository's answer for `generation`; an older generation
    /// is dropped. Returns whether it was kept.
    pub(crate) fn apply_answer(
        &mut self,
        generation: u64,
        repo: usize,
        answer: Result<SearchHits, String>,
    ) -> bool {
        if generation != self.generation || repo >= self.answers.len() {
            return false;
        }
        self.answers[repo] = Some(match answer {
            Ok(found) => Answer {
                hits: found.hits,
                partial: found.partial,
            },
            // A repository that cannot answer (gone, locked) lists nothing.
            Err(_) => Answer::default(),
        });
        true
    }

    /// Once per frame while the popup is open: starts the fan-out when the
    /// debounce has passed, collects answers, asks again for partial ones,
    /// picks up the hosting caches.
    pub(crate) fn tick(&mut self, now: Instant) {
        let mut changed = false;
        if self.loaded_caches.is_empty() {
            let loaded = self.caches.lock().ok().and_then(|mut slot| slot.take());
            if let Some(loaded) = loaded {
                self.loaded_caches = loaded;
                changed = true;
            }
        }
        if self
            .dirty_since
            .is_some_and(|since| now.duration_since(since) >= DEBOUNCE)
        {
            self.dirty_since = None;
            self.retry_at = None;
            self.dispatch();
        }
        let mut still = Vec::with_capacity(self.pending.len());
        for p in std::mem::take(&mut self.pending) {
            match p.reply.take() {
                Some(result) => {
                    let result = result.map_err(|e| e.to_string());
                    changed |= self.apply_answer(p.generation, p.repo, result);
                }
                None => still.push(p),
            }
        }
        self.pending = still;
        if self.pending.is_empty() && self.dirty_since.is_none() {
            let partial = self.answers.iter().flatten().any(|a| a.partial);
            if !partial {
                self.busy_since = None;
                self.retry_at = None;
            } else if self.retries < RETRY_MAX {
                match self.retry_at {
                    None => {
                        let wait = (RETRY_FIRST * 2u32.saturating_pow(self.retries)).min(RETRY_CAP);
                        self.retry_at = Some(now + wait);
                    }
                    Some(at) if at <= now => {
                        self.retry_at = None;
                        self.retries += 1;
                        self.dispatch();
                    }
                    Some(_) => {}
                }
            } else {
                self.busy_since = None;
            }
        }
        if changed {
            self.rebuild(true);
            crate::redraw::poke();
        }
    }

    /// Frames keep coming while a request or a timer is outstanding.
    pub(crate) fn animating(&self) -> bool {
        !self.pending.is_empty() || self.dirty_since.is_some()
    }

    /// The next time something happens by itself (a debounce or a retry).
    pub(crate) fn next_deadline(&self) -> Option<Instant> {
        [self.dirty_since.map(|s| s + DEBOUNCE), self.retry_at]
            .into_iter()
            .flatten()
            .min()
    }

    /// The spinner glyph while the search has been working for a while.
    pub(crate) fn spinner(&self) -> Option<char> {
        let since = self.busy_since?;
        (since.elapsed() >= SPINNER_AFTER)
            .then(|| crate::activity::spinner_frame(self.opened.elapsed()))
    }

    /// Whether the search is waiting for anything (spinner or not).
    pub(crate) fn busy(&self) -> bool {
        self.busy_since.is_some()
    }

    // -- going there ---------------------------------------------------------

    /// The view change that reveals the highlighted result.
    pub(crate) fn go_intent(&self) -> Option<UiIntent> {
        let item = self.selected_item()?;
        let repo = self.scope.repos.get(item.repo)?;
        let id = repo.repo.id.clone();
        Some(match &item.target {
            SearchTarget::Repo { path } => UiIntent::ShowTab {
                repo: RepoId::from_path(std::path::Path::new(path)),
            },
            SearchTarget::Worktree { path } => UiIntent::ShowTab {
                repo: RepoId::from_path(std::path::Path::new(path)),
            },
            SearchTarget::Ref { name } => UiIntent::ShowRef {
                repo: id,
                name: name.clone(),
            },
            SearchTarget::Stash { index, .. } => UiIntent::ShowRef {
                repo: id,
                name: format!("stash@{{{index}}}"),
            },
            SearchTarget::Commit { id: commit } => UiIntent::ShowCommit {
                repo: id,
                commit: *commit,
                also: Vec::new(),
                files: false,
                focus: None,
            },
            SearchTarget::PullRequest { number } => UiIntent::ShowHosting {
                repo: id,
                item: HostingItem::Pr(*number),
            },
            SearchTarget::Issue { number } => UiIntent::ShowHosting {
                repo: id,
                item: HostingItem::Issue(*number),
            },
        })
    }

    /// The highlighted result as a request to review: its repository's
    /// folder and the number. `None` for any other kind of result.
    pub(crate) fn review_request(&self) -> Option<(std::path::PathBuf, u64)> {
        if !crate::build_features::REVIEW {
            return None;
        }
        let item = self.selected_item()?;
        let SearchTarget::PullRequest { number } = &item.target else {
            return None;
        };
        let repo = self.scope.repos.get(item.repo)?;
        Some((repo.repo.root.clone(), *number))
    }

    /// The keys the popup owns besides the text field. Returns whether the
    /// key was one of them.
    pub(crate) fn on_chip_key(&mut self, key: &KeyEvent) -> bool {
        if !key.modifiers.contains(KeyModifiers::ALT) {
            return false;
        }
        match key.code {
            KeyCode::Char(c @ '1'..='8') => {
                self.toggle_chip((c as u8 - b'1') as usize);
                true
            }
            KeyCode::Char('0') => {
                self.reset_chips();
                true
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests;
