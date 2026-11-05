//! Reveal, the source dropdowns and the search box.

use super::*;

impl HostingUi {
    /// Where an agent's `show pr|issue <number>` lands: the row of that item
    /// in the section's rows, with its group unfolded when it is folded. It
    /// selects and reveals only: no browser, no checkout, and it never links
    /// a provider on the agent's behalf. The search box is the user's, so an
    /// item it hides is `NotListed`, not shown by clearing it.
    pub(crate) fn reveal(&mut self, list: HostingList, number: u64) -> RevealOutcome {
        // The search box is a row only while the section is linked and can
        // list items.
        if !self.rows(list).contains(&HostingRow::Search) {
            return RevealOutcome::NotLinked;
        }
        match list {
            HostingList::Pulls => {
                let Some(index) = self
                    .linked_snapshot()
                    .and_then(|s| s.prs.iter().position(|p| p.number == number))
                else {
                    return RevealOutcome::NotListed;
                };
                let holding: Vec<PrGroup> = PrGroup::ALL
                    .into_iter()
                    .filter(|g| self.group_members(*g).contains(&index))
                    .collect();
                // The first group that already shows it, else the first that
                // holds it (unfolded).
                let Some(group) = holding
                    .iter()
                    .copied()
                    .find(|g| !self.collapsed.contains(g))
                    .or_else(|| holding.first().copied())
                else {
                    return RevealOutcome::NotListed;
                };
                self.collapsed.remove(&group);
                self.rows(list)
                    .iter()
                    .position(|r| *r == HostingRow::Pull(group, index))
                    .map_or(RevealOutcome::NotListed, RevealOutcome::Row)
            }
            HostingList::Issues => {
                let Some(index) = self
                    .snapshot
                    .as_ref()
                    .and_then(|s| s.issues.iter().position(|i| i.number == number))
                else {
                    return RevealOutcome::NotListed;
                };
                self.rows(list)
                    .iter()
                    .position(|r| *r == HostingRow::Issue(index))
                    .map_or(RevealOutcome::NotListed, RevealOutcome::Row)
            }
        }
    }

    /// The row a `Focus` effect should land the sidebar selection on.
    pub(crate) fn focus_row(&self, list: HostingList) -> Option<usize> {
        let wanted = match (&self.picker, self.editing) {
            (Some(p), _) if p.target.list() == list => p.target.row()?,
            (_, Some(l)) if l == list && self.editing_query => HostingRow::IssueQuery,
            (_, Some(l)) if l == list => HostingRow::Search,
            _ => return None,
        };
        self.rows(list).iter().position(|r| *r == wanted)
    }

    fn anchor_of(&self, target: PickerTarget) -> Option<Rect> {
        let list = target.list();
        let row = target.row()?;
        let index = self.rows(list).iter().position(|r| *r == row)?;
        self.row_rects
            .borrow()
            .iter()
            .find(|(l, i, _)| *l == list && *i == index)
            .map(|(_, _, r)| *r)
    }

    /// Asks a provider for its projects, once, unless already asked.
    fn ensure_projects(&mut self, profile_id: &str) {
        if matches!(
            self.projects.get(profile_id),
            Some(ProjectsState::Loading | ProjectsState::Ready(_))
        ) {
            return;
        }
        let Some(factory) = self.factory.clone() else {
            return;
        };
        let all = (self.integrations)();
        let Some(integration) = all.iter().find(|i| i.id == profile_id).cloned() else {
            return;
        };
        self.projects
            .insert(profile_id.to_string(), ProjectsState::Loading);
        self.projects_rx
            .push(start_projects(factory, integration, all, None));
    }

    fn repo_options(&self, profile_id: &str) -> Vec<ComboOption> {
        match self.projects.get(profile_id) {
            Some(ProjectsState::Ready(list)) => list
                .iter()
                .map(|p| ComboOption::plain(p.full_name.clone()))
                .collect(),
            _ => Vec::new(),
        }
    }

    /// The projects landed (or failed): fill the open repository dropdown.
    pub(super) fn refresh_repo_picker(&mut self) {
        let Some(target) = self.picker.as_ref().map(|p| p.target) else {
            return;
        };
        let Some(profile) = self
            .snapshot
            .as_ref()
            .and_then(|s| match target {
                PickerTarget::IssueRepo => s.issues_target.as_ref(),
                PickerTarget::PullRepo => s.pulls_target.as_ref(),
                _ => None,
            })
            .map(|t| t.profile.id.clone())
        else {
            return;
        };
        let options = self.repo_options(&profile);
        let loading = matches!(self.projects.get(&profile), Some(ProjectsState::Loading));
        if let Some(p) = self.picker.as_mut() {
            p.combo.set_options(options);
            p.combo.loading = loading;
        }
    }

    /// Why a dropdown cannot open, or opens it. `fallback` is where the
    /// popup hangs when its row is scrolled out of view.
    pub(crate) fn open_picker(
        &mut self,
        target: PickerTarget,
        fallback: (u16, u16),
    ) -> HostingEffect {
        self.menu = None;
        let mut combo = Combo::new(msg::hosting_ui::SELECT.get());
        match target {
            PickerTarget::PrFilter => {
                combo.set_options(
                    PrView::ALL
                        .iter()
                        .map(|v| ComboOption::plain(v.label()))
                        .collect(),
                );
                combo.select(Some(self.pr_view.label().to_string()));
            }
            PickerTarget::IssueFilter => {
                combo.set_options(
                    IssueView::ALL
                        .iter()
                        .map(|v| ComboOption::plain(v.label()))
                        .collect(),
                );
                combo.select(Some(self.issue_view.label().to_string()));
            }
            PickerTarget::IssueProvider | PickerTarget::PullProvider => {
                let pulls = target == PickerTarget::PullProvider;
                let Some(snap) = self.snapshot.as_ref().filter(|s| {
                    !(if pulls {
                        &s.pull_profiles
                    } else {
                        &s.issue_profiles
                    })
                    .is_empty()
                }) else {
                    return HostingEffect::Message(
                        msg::hosting_ui::PICK_NO_INTEGRATION.get().into(),
                    );
                };
                let profiles = if pulls {
                    &snap.pull_profiles
                } else {
                    &snap.issue_profiles
                };
                // What the remotes propose comes first: one pick links the
                // provider and the repository. Nothing here is ever applied
                // before the user picks.
                let mut options: Vec<ComboOption> = snap
                    .suggestions
                    .iter()
                    .filter(|s| profiles.iter().any(|p| p.id == s.profile.id))
                    .map(|s| {
                        ComboOption::new(
                            format!(
                                "{}  \u{00B7} {} \u{00B7} {} {}",
                                s.project,
                                s.remote,
                                provider_name(s.profile.kind),
                                s.profile.label
                            ),
                            format!("{SUGGEST_PREFIX}{}\u{1f}{}", s.profile.id, s.project),
                        )
                    })
                    .collect();
                options.extend(profiles.iter().map(|p| {
                    ComboOption::new(
                        format!("{} \u{00B7} {}", provider_name(p.kind), p.label),
                        p.id.clone(),
                    )
                }));
                let linked = if pulls {
                    self.pulls_source.is_some() || self.override_id.is_some()
                } else {
                    self.issues_source.is_some()
                };
                if linked {
                    options.insert(0, ComboOption::new(unlink_label(), ""));
                }
                let current = if pulls {
                    snap.pulls_target.as_ref()
                } else {
                    snap.issues_target.as_ref()
                }
                .map(|t| t.profile.id.clone());
                combo.set_options(options);
                combo.select(current);
            }
            PickerTarget::IssueRef => {
                combo = Combo::new(msg::hosting_ui::SELECT.get()).with_search();
                let branch = self.inputs.as_ref().and_then(|i| i.head_branch.clone());
                let issues = self.snapshot.as_ref().map_or(&[][..], |s| &s.issues[..]);
                combo.set_options(crate::issue_ref_picker::options(
                    branch.as_deref(),
                    &self.token_bases(),
                    issues,
                ));
            }
            PickerTarget::IssueLists => {
                self.ensure_lists();
                combo = Combo::new(msg::hosting_ui::FILTER_LISTS_ALL.get());
                combo.set_options(self.list_options());
                combo.loading = self.lists_loading();
            }
            PickerTarget::IssueRepo | PickerTarget::PullRepo => {
                let pulls = target == PickerTarget::PullRepo;
                let Some((profile, current)) = self
                    .snapshot
                    .as_ref()
                    .and_then(|s| {
                        if pulls {
                            s.pulls_target.as_ref()
                        } else {
                            s.issues_target.as_ref()
                        }
                    })
                    .map(|t| (t.profile.id.clone(), t.project.clone()))
                else {
                    return HostingEffect::Message(
                        if pulls {
                            msg::hosting_ui::WHY_PULLS_PICK_PROVIDER.get()
                        } else {
                            msg::hosting_ui::WHY_ISSUES_PICK_PROVIDER.get()
                        }
                        .into(),
                    );
                };
                self.ensure_projects(&profile);
                combo = Combo::new(msg::hosting_ui::SELECT.get()).with_custom();
                combo.set_options(self.repo_options(&profile));
                combo.loading = matches!(self.projects.get(&profile), Some(ProjectsState::Loading));
                if !current.is_empty() {
                    combo.select(Some(current));
                }
            }
        }
        let field = self
            .anchor_of(target)
            .unwrap_or(Rect::new(fallback.0, fallback.1, 28, 1));
        combo.set_anchor(field);
        combo.open();
        self.picker = Some(HostingPicker {
            target,
            combo,
            fallback: field,
        });
        if target == PickerTarget::IssueRef {
            // Opened from the composer: the sidebar keeps the keyboard focus
            // it has, the picker is modal anyway.
            return HostingEffect::None;
        }
        HostingEffect::Focus(target.list())
    }

    /// Paints the open dropdown over the frame and registers its rows.
    pub(crate) fn render_picker(
        &mut self,
        frame: &mut Frame,
        hovered_row: Option<usize>,
        hits: &mut Vec<(Rect, HitTarget)>,
    ) {
        let Some(target) = self.picker.as_ref().map(|p| p.target) else {
            return;
        };
        // The composer's picker hangs from the button this frame painted.
        let anchor = if target == PickerTarget::IssueRef {
            hits.iter()
                .find(|(_, t)| matches!(t, HitTarget::ComposerIssueRef))
                .map(|(r, _)| *r)
        } else {
            self.anchor_of(target)
        };
        let bounds = frame.area();
        let Some(picker) = self.picker.as_mut() else {
            return;
        };
        picker.combo.set_anchor(anchor.unwrap_or(picker.fallback));
        picker.combo.set_hover(hovered_row);
        picker.combo.render_popup(frame.buffer_mut(), bounds);
        let front: Vec<(Rect, HitTarget)> = picker
            .combo
            .item_rects()
            .into_iter()
            .map(|(rect, row)| (rect, HitTarget::Hosting(HostingHit::PickerItem(row))))
            .collect();
        hits.splice(0..0, front);
    }

    /// A key while a dropdown is open (it is modal).
    pub(crate) fn on_picker_key(&mut self, key: KeyEvent) -> HostingEffect {
        let Some(picker) = self.picker.as_mut() else {
            return HostingEffect::None;
        };
        let target = picker.target;
        match picker.combo.on_key(key) {
            ComboOutcome::Picked(value) | ComboOutcome::Custom(value) => {
                self.picker = None;
                self.apply_pick(target, &value)
            }
            _ => {
                if !picker.combo.is_open() {
                    self.picker = None;
                }
                HostingEffect::None
            }
        }
    }

    /// A click on row `row` of the open dropdown.
    pub(crate) fn picker_click(&mut self, row: usize) -> HostingEffect {
        let Some(picker) = self.picker.as_mut() else {
            return HostingEffect::None;
        };
        let target = picker.target;
        match picker.combo.click_row(row) {
            ComboOutcome::Picked(value) | ComboOutcome::Custom(value) => {
                self.picker = None;
                self.apply_pick(target, &value)
            }
            _ => HostingEffect::None,
        }
    }

    /// Whether the open dropdown takes this mouse event itself: the
    /// wheel, and the scrollbar's press / drag / release.
    pub(crate) fn picker_wants_mouse(&self, m: &crossterm::event::MouseEvent) -> bool {
        use crossterm::event::{MouseButton, MouseEventKind as K};
        let Some(p) = self.picker.as_ref() else {
            return false;
        };
        match m.kind {
            K::ScrollUp | K::ScrollDown => true,
            K::Down(MouseButton::Left) => p.combo.on_scrollbar(m.column, m.row),
            K::Drag(MouseButton::Left) | K::Up(MouseButton::Left) => p.combo.dragging(),
            _ => false,
        }
    }

    pub(crate) fn picker_mouse(&mut self, m: crossterm::event::MouseEvent) {
        if let Some(p) = self.picker.as_mut() {
            p.combo.on_mouse(m);
        }
    }

    pub(crate) fn close_picker(&mut self) {
        self.picker = None;
    }

    /// Pasted text goes into the open dropdown's search line or the
    /// focused search box.
    pub(crate) fn paste(&mut self, text: &str) {
        let clean: String = text.chars().filter(|c| !c.is_control()).collect();
        if let Some(p) = self.picker.as_mut() {
            p.combo.paste(&clean);
        } else if self.editing_query {
            self.query_draft.push_str(&clean);
        } else if let Some(list) = self.editing {
            match list {
                HostingList::Pulls => self.pr_query.push_str(&clean),
                HostingList::Issues => self.issue_query.push_str(&clean),
            }
        }
    }

    fn apply_pick(&mut self, target: PickerTarget, value: &str) -> HostingEffect {
        match target {
            PickerTarget::PrFilter => {
                if let Some(v) = PrView::ALL.iter().find(|v| v.label() == value) {
                    self.pr_view = *v;
                    self.demand();
                    return HostingEffect::Message(msg::hosting_ui::pulls_filter_set(v.label()));
                }
            }
            PickerTarget::IssueFilter => {
                if let Some(v) = IssueView::ALL.iter().find(|v| v.label() == value) {
                    self.issue_view = *v;
                    self.demand();
                    return HostingEffect::Message(msg::hosting_ui::issues_filter_set(v.label()));
                }
            }
            PickerTarget::PullProvider | PickerTarget::IssueProvider if value.is_empty() => {
                if target == PickerTarget::PullProvider {
                    self.clear_pulls_source();
                    return HostingEffect::Message(msg::hosting_ui::PULLS_UNLINKED.get().into());
                }
                self.clear_issues_source();
                return HostingEffect::Message(msg::hosting_ui::ISSUES_UNLINKED.get().into());
            }
            PickerTarget::PullProvider | PickerTarget::IssueProvider
                if value.starts_with(SUGGEST_PREFIX) =>
            {
                let (profile, project) = value[SUGGEST_PREFIX.len()..]
                    .split_once('\u{1f}')
                    .unwrap_or((&value[SUGGEST_PREFIX.len()..], ""));
                let (profile, project) = (profile.to_string(), project.to_string());
                self.ensure_projects(&profile);
                if target == PickerTarget::PullProvider {
                    self.set_pulls_source(&profile, project.clone());
                    return HostingEffect::Message(msg::hosting_ui::pulls_come_from(&project));
                }
                self.set_issues_source(&profile, project.clone());
                return HostingEffect::Message(msg::hosting_ui::issues_come_from(&project));
            }
            PickerTarget::PullProvider => {
                let same = self
                    .snapshot
                    .as_ref()
                    .and_then(|s| s.pulls_target.as_ref())
                    .filter(|t| t.profile.id == value)
                    .map(|t| t.project.clone())
                    .unwrap_or_default();
                self.set_pulls_source(value, same);
                self.ensure_projects(value);
                return HostingEffect::Message(msg::hosting_ui::PULLS_PICK_REPO.get().into());
            }
            PickerTarget::PullRepo => {
                let profile = self
                    .snapshot
                    .as_ref()
                    .and_then(|s| s.pulls_target.as_ref())
                    .map(|t| t.profile.id.clone());
                if let Some(profile) = profile {
                    self.set_pulls_source(&profile, value.to_string());
                    return HostingEffect::Message(msg::hosting_ui::pulls_come_from(value));
                }
            }
            PickerTarget::IssueProvider => {
                let same = self
                    .snapshot
                    .as_ref()
                    .and_then(|s| s.issues_target.as_ref())
                    .filter(|t| t.profile.id == value)
                    .map(|t| t.project.clone())
                    .unwrap_or_default();
                // A folder source is its profile's folder: nothing to pick.
                let folder = (self.integrations)()
                    .into_iter()
                    .find(|i| i.id == value && i.kind.capabilities().local_folder)
                    .map(|i| i.url);
                if let Some(folder) = folder {
                    let stored = self
                        .issues_source
                        .as_ref()
                        .filter(|s| s.integration == value)
                        .map(|s| s.project.clone())
                        .unwrap_or_default();
                    let project = if stored.is_empty() { folder } else { stored };
                    self.set_issues_source(value, project.clone());
                    self.ensure_projects(value);
                    return HostingEffect::Message(msg::hosting_ui::issues_come_from(&project));
                }
                self.set_issues_source(value, same);
                self.ensure_projects(value);
                return HostingEffect::Message(msg::hosting_ui::ISSUES_PICK_REPO.get().into());
            }
            PickerTarget::IssueRepo => {
                let profile = self
                    .snapshot
                    .as_ref()
                    .and_then(|s| s.issues_target.as_ref())
                    .map(|t| t.profile.id.clone());
                if let Some(profile) = profile {
                    self.set_issues_source(&profile, value.to_string());
                    return HostingEffect::Message(msg::hosting_ui::issues_come_from(value));
                }
            }
            PickerTarget::IssueRef => {
                return HostingEffect::InsertIssueRef(value.to_string());
            }
            PickerTarget::IssueLists => {
                let next = toggle_list(&self.saved_query(), value);
                let message = self.set_issue_query(next);
                // Several lists can be ticked: the dropdown stays at hand.
                self.notice = Some(message);
                return self.open_picker(PickerTarget::IssueLists, (2, 2));
            }
        }
        HostingEffect::None
    }

    /// Points the ISSUES section at `project` of `profile_id`: remembered
    /// for this repository, shown at once, and asked again.
    pub(super) fn set_issues_source(&mut self, profile_id: &str, project: String) {
        let path = crate::remotes_ui::repo_state_path(&self.repo_root);
        let mut state = gitpoulpi_session::RepoState::load(&path).unwrap_or_default();
        // The extra query belongs to a project: it survives only a re-pick of
        // the same one.
        let filter = self
            .issues_source
            .as_ref()
            .filter(|s| s.integration == profile_id && s.project == project)
            .and_then(|s| s.filter.clone());
        let source = gitpoulpi_session::IssuesSource {
            integration: profile_id.to_string(),
            project: project.clone(),
            filter,
        };
        state.issues_source = Some(source.clone());
        let _ = state.save(&path);
        self.issues_source = Some(source);
        if let Some(snap) = self.snapshot.as_mut() {
            if let Some(profile) = snap.issue_profiles.iter().find(|p| p.id == profile_id) {
                snap.issues_target = Some(crate::hosting_task::IssuesTarget {
                    profile: profile.clone(),
                    project,
                    chosen: true,
                });
                snap.issues.clear();
                snap.issues_problem = None;
            }
        }
        self.issue_query.clear();
        self.demand();
    }

    /// Points the PULL REQUESTS section at `project` of `profile_id`:
    /// remembered for this repository, shown at once, and asked again.
    fn set_pulls_source(&mut self, profile_id: &str, project: String) {
        let path = crate::remotes_ui::repo_state_path(&self.repo_root);
        let mut state = gitpoulpi_session::RepoState::load(&path).unwrap_or_default();
        let source = gitpoulpi_session::IssuesSource {
            integration: profile_id.to_string(),
            project: project.clone(),
            filter: None,
        };
        state.pulls_source = Some(source.clone());
        // A profile pinned the old way would out-rank nothing, but keep the
        // two in step: the pick is the pin.
        state.hosting_integration = None;
        let _ = state.save(&path);
        self.pulls_source = Some(source);
        self.override_id = None;
        if let Some(snap) = self.snapshot.as_mut() {
            if let Some(profile) = snap.pull_profiles.iter().find(|p| p.id == profile_id) {
                snap.pulls_target = Some(crate::hosting_task::IssuesTarget {
                    profile: profile.clone(),
                    project: project.clone(),
                    chosen: true,
                });
                snap.prs.clear();
                snap.head_pr = None;
                snap.problem = None;
                snap.profile_id = (!project.is_empty()).then(|| profile_id.to_string());
                snap.project = project;
                snap.can_pulls = snap.profile_id.is_some();
                snap.kind = Some(profile.kind);
                snap.profile_label = profile.label.clone();
            }
        }
        self.pr_query.clear();
        self.demand();
    }

    /// Unlinks the PULL REQUESTS section: the choice (and any pinned
    /// profile) is dropped from the repo state, and nothing links it again.
    pub(super) fn clear_pulls_source(&mut self) {
        let path = crate::remotes_ui::repo_state_path(&self.repo_root);
        let mut state = gitpoulpi_session::RepoState::load(&path).unwrap_or_default();
        state.pulls_source = None;
        state.hosting_integration = None;
        let _ = state.save(&path);
        self.pulls_source = None;
        self.override_id = None;
        if let Some(snap) = self.snapshot.as_mut() {
            snap.pulls_target = None;
            snap.profile_id = None;
            snap.can_pulls = false;
            snap.prs.clear();
            snap.head_pr = None;
            snap.problem = None;
        }
        self.pr_query.clear();
        self.demand();
    }

    /// Drops this repository's chosen issue source: `[issues_source]` is
    /// removed from the repo state and the section is unlinked until the
    /// user picks again (the remotes only suggest, they never re-link).
    pub(super) fn clear_issues_source(&mut self) {
        let path = crate::remotes_ui::repo_state_path(&self.repo_root);
        let mut state = gitpoulpi_session::RepoState::load(&path).unwrap_or_default();
        state.issues_source = None;
        let _ = state.save(&path);
        self.issues_source = None;
        if let Some(snap) = self.snapshot.as_mut() {
            snap.issues_target = None;
            snap.issues.clear();
            snap.issues_problem = None;
        }
        self.issue_query.clear();
        self.demand();
    }

    /// A key while a search box has the keyboard. False when none does.
    pub(crate) fn on_search_key(&mut self, key: KeyEvent) -> bool {
        let Some(list) = self.editing else {
            return false;
        };
        if self.editing_query {
            return self.on_query_key(key);
        }
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let alt = key.modifiers.contains(KeyModifiers::ALT);
        let query = match list {
            HostingList::Pulls => &mut self.pr_query,
            HostingList::Issues => &mut self.issue_query,
        };
        match key.code {
            KeyCode::Esc | KeyCode::Enter | KeyCode::Tab | KeyCode::BackTab | KeyCode::Down => {
                self.editing = None;
            }
            KeyCode::Backspace => {
                query.pop();
            }
            KeyCode::Char('u') if ctrl => query.clear(),
            KeyCode::Char(c) if !ctrl && !alt => query.push(c),
            _ => {}
        }
        true
    }

    /// The tooltip over the toolbar chip, hung below `chip`.
    pub(crate) fn render_chip_tooltip(&self, frame: &mut Frame, chip: Rect) -> Rect {
        let lines: Vec<Line> = self.chip_tooltip().into_iter().map(Line::from).collect();
        let width = lines.iter().map(|l| l.width() as u16).max().unwrap_or(20) + 2;
        let height = lines.len() as u16 + 2;
        let popup = anchored_popup(frame.area(), chip.x, chip.y, width, height);
        frame.render_clipped(Clear, popup);
        frame.render_clipped(
            Paragraph::new(lines).block(
                Block::default()
                    .border_type(BorderType::Rounded)
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(th().ui.accent))
                    .style(Style::default().bg(th().ui.panel)),
            ),
            popup,
        );
        popup
    }

    /// Paints the open menu and returns its row hits.
    pub(crate) fn render_menu(
        &self,
        frame: &mut Frame,
        hovered_row: Option<usize>,
        hits: &mut Vec<(Rect, HitTarget)>,
    ) {
        if let Some(menu) = &self.menu {
            let front: Vec<(Rect, HitTarget)> = menu
                .render(frame, hovered_row)
                .into_iter()
                .enumerate()
                .map(|(i, rect)| (rect, HitTarget::Hosting(HostingHit::MenuItem(i))))
                .collect();
            hits.splice(0..0, front);
        }
    }

    // -- the issue source's extra query (Jira JQL, Azure WIQL, Trello lists) ---

    /// The saved extra query of the ISSUES source, or empty.
    pub(super) fn saved_query(&self) -> String {
        self.issues_source
            .as_ref()
            .and_then(|s| s.filter.clone())
            .unwrap_or_default()
    }

    /// The linked ISSUES provider's family, once one is chosen.
    pub(super) fn issues_family(&self) -> Option<gitpoulpi_config::ApiFamily> {
        self.snapshot
            .as_ref()
            .and_then(|s| s.issues_target.as_ref())
            .map(|t| t.profile.kind.family())
    }

    /// The query row was used: Trello opens its lists dropdown, Jira and
    /// Azure give the keyboard to the text field.
    pub(super) fn activate_query(&mut self, fallback: (u16, u16)) -> HostingEffect {
        if self.issues_family() == Some(gitpoulpi_config::ApiFamily::Trello) {
            return self.open_picker(PickerTarget::IssueLists, fallback);
        }
        self.picker = None;
        self.editing = Some(HostingList::Issues);
        self.editing_query = true;
        self.query_draft = self.saved_query();
        HostingEffect::None
    }

    /// Leaves the query field, keeping the text (or, `keep` false, dropping it).
    pub(super) fn finish_query_edit(&mut self, keep: bool) {
        self.editing_query = false;
        if self.editing == Some(HostingList::Issues) {
            self.editing = None;
        }
        let draft = std::mem::take(&mut self.query_draft);
        if keep {
            let typed = draft.trim();
            let next = (!typed.is_empty()).then(|| typed.to_string());
            if next.clone().unwrap_or_default() != self.saved_query() {
                self.notice = Some(self.set_issue_query(next));
            }
        }
    }

    fn on_query_key(&mut self, key: KeyEvent) -> bool {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let alt = key.modifiers.contains(KeyModifiers::ALT);
        match key.code {
            KeyCode::Esc => self.finish_query_edit(false),
            KeyCode::Enter | KeyCode::Tab | KeyCode::BackTab | KeyCode::Down => {
                self.finish_query_edit(true)
            }
            KeyCode::Backspace => {
                self.query_draft.pop();
            }
            KeyCode::Char('u') if ctrl => self.query_draft.clear(),
            KeyCode::Char(c) if !ctrl && !alt => self.query_draft.push(c),
            _ => {}
        }
        true
    }

    /// Saves the issue source's extra query (`None` clears it), asks the host
    /// again and returns the status line to show.
    pub(super) fn set_issue_query(&mut self, query: Option<String>) -> String {
        let path = crate::remotes_ui::repo_state_path(&self.repo_root);
        let mut state = gitpoulpi_session::RepoState::load(&path).unwrap_or_default();
        if state.issues_source.is_none() {
            state.issues_source = self.issues_source.clone();
        }
        if let Some(source) = state.issues_source.as_mut() {
            source.filter = query.clone();
        }
        let _ = state.save(&path);
        if let Some(source) = self.issues_source.as_mut() {
            source.filter = query.clone();
        }
        self.demand();
        match query {
            Some(q) => msg::hosting_ui::issue_query_set(q),
            None => msg::hosting_ui::ISSUE_QUERY_CLEARED.get().to_string(),
        }
    }

    // -- the Trello lists of the linked board ---------------------------------

    fn lists_key(&self) -> Option<(String, String)> {
        let target = self.snapshot.as_ref()?.issues_target.as_ref()?;
        (!target.project.is_empty()).then(|| (target.profile.id.clone(), target.project.clone()))
    }

    /// Asks the board for its lists, once.
    fn ensure_lists(&mut self) {
        let Some((profile, board)) = self.lists_key() else {
            return;
        };
        let known = self
            .lists
            .as_ref()
            .is_some_and(|(p, b, _)| *p == profile && *b == board);
        let asking = self
            .lists_rx
            .as_ref()
            .is_some_and(|(p, b, _)| *p == profile && *b == board);
        if known || asking {
            return;
        }
        let Some(factory) = self.factory.clone() else {
            return;
        };
        let all = (self.integrations)();
        let Some(integration) = all.iter().find(|i| i.id == profile).cloned() else {
            return;
        };
        let rx = crate::hosting_task::start_issue_options(factory, integration, all, board.clone());
        self.lists_rx = Some((profile, board, rx));
    }

    fn lists_loading(&self) -> bool {
        self.lists_rx.is_some()
    }

    /// The dropdown rows: "All lists", then each list, ticked when shown.
    fn list_options(&self) -> Vec<ComboOption> {
        let mut options = vec![ComboOption::new(
            msg::hosting_ui::FILTER_LISTS_ALL.get(),
            String::new(),
        )];
        let key = self.lists_key();
        let names = self.lists.as_ref().and_then(|(p, b, names)| {
            (key.as_ref() == Some(&(p.clone(), b.clone())))
                .then_some(names.as_ref())
                .flatten()
        });
        let shown = chosen_lists(&self.saved_query());
        for name in names.into_iter().flatten() {
            let ticked = shown.iter().any(|s| s.eq_ignore_ascii_case(name));
            options.push(ComboOption::new(
                format!("{} {name}", if ticked { "\u{2713}" } else { " " }),
                name.clone(),
            ));
        }
        options
    }

    /// The lists landed: fill the open lists dropdown.
    pub(super) fn refresh_lists_picker(&mut self) {
        let options = self.list_options();
        let loading = self.lists_loading();
        if let Some(p) = self
            .picker
            .as_mut()
            .filter(|p| p.target == PickerTarget::IssueLists)
        {
            p.combo.set_options(options);
            p.combo.loading = loading;
        }
    }
}

/// The list names a saved Trello query (comma-separated) shows.
pub(super) fn chosen_lists(query: &str) -> Vec<String> {
    query
        .split(',')
        .map(|n| n.trim().to_string())
        .filter(|n| !n.is_empty())
        .collect()
}

/// `query` with `name` ticked or unticked; empty `name` clears every list.
pub(super) fn toggle_list(query: &str, name: &str) -> Option<String> {
    if name.is_empty() {
        return None;
    }
    let mut names = chosen_lists(query);
    match names.iter().position(|n| n.eq_ignore_ascii_case(name)) {
        Some(at) => {
            names.remove(at);
        }
        None => names.push(name.to_string()),
    }
    (!names.is_empty()).then(|| names.join(", "))
}
