//! The show-view intents of an external agent (plan
//! the mcp-show-ui design): show a commit (paging the lazy Log
//! in until it is loaded), a ref, a Log search, a pull request or issue row,
//! and the working changes. [`agent_view`](super::agent_view) decides *when*
//! (deferral, dwell, Back); this decides *what moves*.
//!
//! Everything here is view-only: selections, unfolded folders, a restored
//! pane, the find bar opened with a query nobody submitted, the composer's
//! focus without a character typed. Nothing checks out, links a provider,
//! opens the browser or unhides a ref.
//!
//! A commit that is not loaded yet makes a [`ShowReveal`]: the Log is asked
//! for one page at a time (`ui_intent::plan::commit_step`), ticked from
//! `agent_view_tick`, with a deadline per step and the user's first key
//! canceling it (`Refused("user took over")`). No spinner outlives it.

use std::path::PathBuf;

use crate::event_loop::*;
use crate::hosting_ui::{HostingList, RevealOutcome};
use crate::ui_intent::plan::{self, CommitStep, LogFacts, Presence};
use crate::ui_viewer::{CommitGoal, Reveal, ShowReveal, REVEAL_POLL};
use gitpoulpi_ipc::ui::{sanitize_with_limit, HostingItem, PaneId, UiIntent, UiOrigin, UiOutcome};
use init::WindowState;

use super::agent_view::{en, Applied};

/// How long one step (the Log answering a page request) may take.
const STEP_DEADLINE: Duration = Duration::from_secs(6);

fn not_found(what: impl Into<String>) -> Applied {
    Applied::Not(Box::new(UiOutcome::NotFound { what: what.into() }))
}

impl WindowState {
    /// Applies `ShowCommit`, `ShowRef`, `ShowSearch`, `ShowHosting` or
    /// `ShowWip` (the window already is on the intent's repository).
    pub(super) fn apply_show(
        &mut self,
        id: Option<u64>,
        intent: &UiIntent,
        origin: &UiOrigin,
    ) -> Applied {
        let what = ui_intent::describe(intent);
        match intent {
            UiIntent::ShowCommit {
                commit,
                also,
                files,
                focus,
                ..
            } => self.start_show(
                id,
                origin,
                what,
                CommitGoal {
                    oid: *commit,
                    also: also.clone(),
                    files: *files,
                    focus: *focus,
                },
                None,
                false,
            ),
            UiIntent::ShowRef { name, .. } => self.show_ref(id, origin, what, name),
            UiIntent::ShowSearch { query, .. } => self.show_search(query),
            UiIntent::ShowHosting { item, .. } => self.show_hosting(*item),
            UiIntent::ShowWip {
                path,
                staged,
                composer,
                ..
            } => self.show_wip(path.as_deref(), *staged, *composer),
            _ => Applied::Not(Box::new(UiOutcome::refused(en(|| {
                gitpoulpi_i18n::msg::agent_reason::NOT_SUPPORTED
                    .get()
                    .to_string()
            })))),
        }
    }

    // -- a commit ----------------------------------------------------------

    fn start_show(
        &mut self,
        id: Option<u64>,
        origin: &UiOrigin,
        what: String,
        goal: CommitGoal,
        note: Option<String>,
        moved: bool,
    ) -> Applied {
        // The Log is where a commit is shown (and where its pages land).
        self.central_view = CentralView::Log;
        let now = Instant::now();
        self.step_show(Box::new(ShowReveal {
            id,
            origin: origin.clone(),
            what,
            goal,
            note,
            moved,
            pushed: false,
            started: now,
            until: now + STEP_DEADLINE,
            poll_at: now + REVEAL_POLL,
            pages: 0,
        }))
    }

    /// Where `oid` is in the Log as drawn. With soloed refs the displayed
    /// rows are the soloed ancestry, so a loaded commit outside it is
    /// `Filtered` - never unhidden for an agent.
    pub(super) fn commit_presence(&self, oid: Oid) -> Presence {
        let Ok(graph) = &self.log_result else {
            return Presence::Missing;
        };
        let at = |nodes: &[CommitNode]| nodes.iter().position(|n| n.id == oid);
        if self.solo_refs.is_empty() {
            return at(&graph.nodes).map_or(Presence::Missing, |i| {
                Presence::Shown(self.log_order.commit_display_index(i))
            });
        }
        let shown = apply_solo_filter(&graph.nodes, &self.solo_refs, &self.refs_result);
        match at(&shown) {
            Some(i) => Presence::Shown(self.log_order.commit_display_index(i)),
            None if at(&graph.nodes).is_some() => Presence::Filtered,
            None => Presence::Missing,
        }
    }

    /// One look at the Log: select the commit, give up, or ask for more.
    pub(super) fn step_show(&mut self, mut show: Box<ShowReveal>) -> Applied {
        // A tab just opened has no Log yet: wait for it (the deadline ends it).
        let nothing_yet = match &self.log_result {
            Ok(graph) => graph.nodes.is_empty(),
            Err(_) => true,
        };
        if nothing_yet {
            return Applied::Waiting(show);
        }
        let visible = log_visible_rows(self.log_area);
        let facts = LogFacts {
            presence: self.commit_presence(show.goal.oid),
            loaded: self.lazy_log.loaded(),
            pages: show.pages,
            loading: self.lazy_log.is_loading(),
            can_page: self.lazy_log.next_page(0, visible).is_some(),
        };
        match plan::commit_step(&facts) {
            CommitStep::Select(row) => {
                let missing = self.select_commit_row(&show.goal, row);
                match show.note.take().or(missing) {
                    Some(note) => Applied::Partial(note),
                    None => Applied::Done,
                }
            }
            CommitStep::Hidden => Applied::Partial("hidden_by_filter".into()),
            CommitStep::Wait => Applied::Waiting(show),
            CommitStep::RequestPage => {
                if let Some(request) = self.lazy_log.next_page(0, visible) {
                    self.lazy_log.mark_requested(request);
                    self.link.request_log_page(request);
                }
                show.pages += 1;
                show.until = Instant::now() + STEP_DEADLINE;
                Applied::Waiting(show)
            }
            CommitStep::GiveUp(why) if show.moved => Applied::Partial(why),
            CommitStep::GiveUp(why) => not_found(why),
        }
    }

    /// Moves the Log's selection (and what goes with it) to display row
    /// `row`. Returns a `Partial` note when some `also` commit is not
    /// shown.
    fn select_commit_row(&mut self, goal: &CommitGoal, row: usize) -> Option<String> {
        self.central_view = CentralView::Log;
        self.log_state.select(Some(row));
        self.log_follow = true;
        self.right_panel = RightPanel::CommitDetail;
        // The detail panel is part of what is shown.
        self.right_pane_collapsed = false;
        self.log_multi.reset_to(Some(RowKey::Id(goal.oid)));
        let mut missing = 0;
        for other in &goal.also {
            if matches!(self.commit_presence(*other), Presence::Shown(_)) {
                self.log_multi
                    .toggle(RowKey::Id(*other), Some(RowKey::Id(goal.oid)));
            } else {
                missing += 1;
            }
        }
        self.focus = Focus::Log;
        self.commit_file_focus = false;
        match goal.focus {
            Some(PaneId::Refs) => {
                self.left_pane_collapsed = false;
                self.focus = Focus::Refs;
            }
            Some(PaneId::Status) => self.commit_file_focus = true,
            Some(PaneId::Log) | None => {}
        }
        if goal.files && !self.view_all_files {
            commands::toggle_view_all_files(&mut self.command_ctx());
        }
        (missing > 0)
            .then(|| en(|| gitpoulpi_i18n::msg::agent_reason::others_not_in_log(missing as u64)))
    }

    // -- a ref -------------------------------------------------------------

    fn show_ref(
        &mut self,
        id: Option<u64>,
        origin: &UiOrigin,
        what: String,
        name: &str,
    ) -> Applied {
        if let Some(n) = name
            .strip_prefix("stash@{")
            .and_then(|r| r.strip_suffix('}'))
            .and_then(|n| n.parse::<usize>().ok())
        {
            return self.show_stash(n);
        }
        let Ok(refs) = self.refs_result.as_ref() else {
            return not_found(en(|| {
                gitpoulpi_i18n::msg::agent_reason::REFS_NOT_LOADED
                    .get()
                    .to_string()
            }));
        };
        let Some(target) = plan::find_ref(refs, name) else {
            return not_found(en(|| gitpoulpi_i18n::msg::agent_reason::no_ref_named(name)));
        };
        let wanted = self.refs_filter.to_lowercase();
        let note = if !wanted.is_empty() && !target.name.to_lowercase().contains(&wanted) {
            // The sidebar's search box is the user's: not cleared for us.
            Some("hidden_by_filter".to_string())
        } else {
            self.select_sidebar_ref(&target).err()
        };
        let goal = CommitGoal {
            oid: target.target,
            also: Vec::new(),
            files: false,
            focus: Some(PaneId::Refs),
        };
        self.start_show(id, origin, what, goal, note, true)
    }

    /// Selects the ref's row in its sidebar section: opens the section and
    /// the folders above it (in memory; the user's saved fold state is
    /// theirs) and restores a collapsed Refs pane.
    fn select_sidebar_ref(&mut self, target: &plan::RefTarget) -> Result<(), String> {
        let section = SidebarSection::from_ref_kind(target.kind);
        let Ok(refs) = self.refs_result.as_ref() else {
            return Err(en(|| {
                gitpoulpi_i18n::msg::agent_reason::REFS_NOT_LOADED
                    .get()
                    .to_string()
            }));
        };
        let names = group_refs_kind(refs, &self.refs_filter, target.kind);
        let all = ref_display_rows(
            &names,
            target.kind,
            self.hierarchical_refs,
            &Default::default(),
            &self.remote_names,
        );
        for folder in plan::folders_to_open(&all, &target.name) {
            self.collapsed_ref_folders.remove(&(target.kind, folder));
        }
        let rows = ref_display_rows(
            &names,
            target.kind,
            self.hierarchical_refs,
            &self.collapsed_ref_folders,
            &self.remote_names,
        );
        let Some(row) = rows
            .iter()
            .position(|r| r.leaf_name() == Some(target.name.as_str()))
        else {
            return Err(en(|| {
                gitpoulpi_i18n::msg::agent_reason::REF_NO_ROW
                    .get()
                    .to_string()
            }));
        };
        self.reveal_sidebar_row(section, row);
        Ok(())
    }

    pub(super) fn reveal_sidebar_row(&mut self, section: SidebarSection, row: usize) {
        self.left_pane_collapsed = false;
        self.collapsed_ref_sections.remove(section.title());
        self.sidebar_states.select(section, Some(row));
        self.active_ref_section = section;
        self.focus = Focus::Refs;
    }

    fn show_stash(&mut self, n: usize) -> Applied {
        match &self.stashes_result {
            Ok(stashes) if n < stashes.len() => {
                self.reveal_sidebar_row(SidebarSection::Stashes, n);
                Applied::Done
            }
            _ => not_found(format!("no stash@{{{n}}}")),
        }
    }

    // -- the find bar ------------------------------------------------------

    fn show_search(&mut self, query: &str) -> Applied {
        let query = sanitize_with_limit(query, 100);
        self.central_view = CentralView::Log;
        self.focus = Focus::Log;
        // Opened with the query in it, like the palette: Enter is the user's.
        let hits = log_search_hits(&self.log_result, &self.refs_result, &self.solo_refs, &query);
        if let Some(&node) = hits.first() {
            self.log_state
                .select(Some(self.log_order.commit_display_index(node)));
            self.log_follow = true;
            self.right_panel = RightPanel::CommitDetail;
        }
        self.input_mode = InputMode::SearchingLog(LogSearch { query, current: 0 });
        Applied::Done
    }

    // -- pull requests and issues -------------------------------------------

    fn show_hosting(&mut self, item: HostingItem) -> Applied {
        let (list, number, noun) = match item {
            HostingItem::Pr(n) => (
                HostingList::Pulls,
                n,
                gitpoulpi_i18n::msg::agent_reason::NOUN_PULL_REQUEST,
            ),
            HostingItem::Issue(n) => (
                HostingList::Issues,
                n,
                gitpoulpi_i18n::msg::agent_reason::NOUN_ISSUE,
            ),
        };
        match self.hosting.reveal(list, number) {
            RevealOutcome::NotLinked => Applied::Not(Box::new(UiOutcome::refused(en(|| {
                gitpoulpi_i18n::msg::agent_reason::HOSTING_NOT_LINKED
                    .get()
                    .to_string()
            })))),
            RevealOutcome::NotListed => not_found(en(|| {
                gitpoulpi_i18n::msg::agent_reason::not_in_sidebar_list(noun, number)
            })),
            RevealOutcome::Row(row) => {
                self.reveal_sidebar_row(list.section(), row);
                Applied::Done
            }
        }
    }

    // -- the working changes -----------------------------------------------

    pub(super) fn show_wip(&mut self, path: Option<&str>, staged: bool, composer: bool) -> Applied {
        if !has_wip_row(&self.status_result) {
            return not_found(en(|| {
                gitpoulpi_i18n::msg::agent_reason::NO_WIP.get().to_string()
            }));
        }
        let section = if staged {
            StatusSection::Staged
        } else {
            StatusSection::Unstaged
        };
        let mut row = None;
        if let Some(path) = path {
            let wanted = PathBuf::from(path);
            // A folder collapsed in tree mode hides the file: open it.
            for ancestor in wanted.ancestors().skip(1) {
                self.collapsed_status_folders
                    .remove(&crate::status_list::fold_key(section, ancestor));
            }
            let Ok(tree) = &self.status_result else {
                return not_found(en(|| {
                    gitpoulpi_i18n::msg::agent_reason::TREE_NOT_LOADED
                        .get()
                        .to_string()
                }));
            };
            let lines = build_status_section_lines(
                tree,
                section,
                self.status_list_mode,
                &self.collapsed_status_folders,
                "",
            );
            row = lines
                .iter()
                .position(|l| l.file() == Some((section, &wanted)));
            if row.is_none() {
                return not_found(en(|| {
                    gitpoulpi_i18n::msg::agent_reason::path_not_among(
                        path,
                        if staged {
                            gitpoulpi_i18n::msg::agent_reason::KIND_STAGED
                        } else {
                            gitpoulpi_i18n::msg::agent_reason::KIND_UNSTAGED
                        },
                    )
                }));
            }
        }
        self.central_view = CentralView::Log;
        self.log_state.select(Some(0));
        self.log_follow = true;
        self.right_panel = RightPanel::Status;
        self.right_pane_collapsed = false;
        self.focus = Focus::Status;
        self.active_status_section = section;
        if let Some(row) = row {
            self.status_states.select(section, Some(row));
        }
        if composer {
            // The keyboard goes to the summary field; nothing is typed in it.
            self.composer_focus = Some(ComposerField::Summary);
        }
        Applied::Done
    }

    // -- the follow-up of a pending reveal ---------------------------------

    /// Ticks a pending `show commit` / `show ref`.
    pub(super) fn tick_show_reveal(&mut self, show: Box<ShowReveal>) {
        let (id, origin, what, pushed) =
            (show.id, show.origin.clone(), show.what.clone(), show.pushed);
        let result = if self.ui_viewer.input_since(show.started) {
            // The user always wins.
            Applied::Not(Box::new(UiOutcome::refused(en(|| {
                gitpoulpi_i18n::msg::agent_reason::USER_TOOK_OVER
                    .get()
                    .to_string()
            }))))
        } else if Instant::now() >= show.until {
            if show.moved {
                Applied::Partial(en(|| {
                    gitpoulpi_i18n::msg::agent_reason::LOG_COMMIT_TIMEOUT
                        .get()
                        .to_string()
                }))
            } else {
                Applied::Not(Box::new(UiOutcome::refused(en(|| {
                    gitpoulpi_i18n::msg::agent_reason::LOG_COMMIT_TIMEOUT
                        .get()
                        .to_string()
                }))))
            }
        } else {
            let mut show = show;
            show.poll_at = Instant::now() + REVEAL_POLL;
            self.step_show(show)
        };
        self.conclude_agent_intent(id, &origin, &what, pushed, result);
    }

    /// Answers a reveal an intent superseded.
    pub(super) fn supersede_reveal(&mut self) {
        let id = match self.ui_viewer.take_reveal() {
            Some(Reveal::Show(old)) => old.id,
            Some(Reveal::View(old)) => old.id,
            Some(Reveal::Highlight(old)) => old.id,
            _ => None,
        };
        if let Some(id) = id {
            self.ui_viewer
                .report_now(self.link.ui(), id, UiOutcome::refused("superseded"));
        }
    }
}
