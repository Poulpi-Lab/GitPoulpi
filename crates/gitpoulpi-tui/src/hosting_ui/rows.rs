//! Rows and painting: what the sections list, the toolbar chip, and how both are drawn.

use super::*;

impl HostingUi {
    // -- what to show -------------------------------------------------------

    /// The linked provider is GitLab: `auto` terminology then says "Merge
    /// Request".
    pub(crate) fn linked_is_gitlab(&self) -> bool {
        self.linked_snapshot()
            .and_then(|s| s.kind)
            .is_some_and(|k| k.family() == gitpoulpi_config::ApiFamily::Gitlab)
    }

    /// "Edit on GitHub / GitLab" for the pull request in row `index`
    /// (the row button, `e`).
    pub(crate) fn edit_row(
        &mut self,
        index: usize,
        clipboard: &mut crate::clipboard::Clipboard,
    ) -> HostingEffect {
        let url = match self.rows(HostingList::Pulls).get(index) {
            Some(HostingRow::Pull(_, i)) => {
                let Some(snap) = self.snapshot.as_ref() else {
                    return HostingEffect::None;
                };
                snap.prs.get(*i).map(|pr| edit_url(snap.kind, &pr.web_url))
            }
            _ => None,
        };
        match url {
            Some(url) => self.run(HostingAction::OpenUrl(url), clipboard),
            None => HostingEffect::Message(msg::hosting_ui::SELECT_PR_FIRST.get().into()),
        }
    }

    /// Where the row's edit pencil sits inside `rect` (3 cells at the
    /// right end), or `None` when the row is too narrow to spare them.
    fn edit_button_rect(rect: Rect) -> Option<Rect> {
        const W: u16 = 3;
        (rect.width >= 16).then(|| Rect::new(rect.x + rect.width - W, rect.y, W, 1))
    }

    /// Whether the user linked this section to a provider (persisted in the
    /// repository state). Unlinked sections never fetch and never say
    /// "Loading".
    fn linked_by_state(&self, list: HostingList) -> bool {
        match list {
            HostingList::Pulls => self.pulls_source.is_some() || self.override_id.is_some(),
            HostingList::Issues => self
                .issues_source
                .as_ref()
                .is_some_and(|s| !s.project.is_empty()),
        }
    }

    /// The spinner frame while this section's data has been in flight for
    /// more than [`LOADING_AFTER`].
    fn loading(&self, list: HostingList) -> Option<char> {
        let since = self.in_flight_since?;
        let elapsed = since.elapsed();
        (self.in_flight && elapsed >= LOADING_AFTER && self.linked_by_state(list))
            .then(|| crate::activity::spinner_frame(elapsed))
    }

    pub(super) fn linked_snapshot(&self) -> Option<&HostingSnapshot> {
        self.snapshot.as_ref().filter(|s| s.linked())
    }

    /// `(reference, title)` of the ISSUES section's issues, for the Flow
    /// Start form's issue picker.
    pub(crate) fn issue_choices(&self) -> Vec<(String, String)> {
        let issues = self.snapshot.as_ref().map_or(&[][..], |s| &s.issues[..]);
        crate::issue_ref_picker::choices(issues)
    }

    /// The rows' inputs, borrowed from this state: what both frontends list
    /// comes from `gitpoulpi_link::hosting_rows`.
    pub(super) fn view(&self) -> RowsInput<'_> {
        RowsInput {
            snapshot: self.snapshot.as_ref(),
            collapsed: &self.collapsed,
            pr_query: &self.pr_query,
            issue_query: &self.issue_query,
            loading_pulls: self.loading(HostingList::Pulls),
            loading_issues: self.loading(HostingList::Issues),
            fresh: self.fresh,
            now: (self.now)(),
        }
    }

    /// `(pull requests, issues)` sections to show.
    pub(crate) fn sections_visible(&self) -> (bool, bool) {
        self.view().sections_visible()
    }

    fn query(&self, list: HostingList) -> &str {
        match list {
            HostingList::Pulls => &self.pr_query,
            HostingList::Issues => &self.issue_query,
        }
    }

    /// Indexes into the snapshot's pull requests that `group` shows,
    /// after the search box.
    pub(super) fn group_members(&self, group: PrGroup) -> Vec<usize> {
        self.view().group_members(group)
    }

    pub(crate) fn rows(&self, list: HostingList) -> Vec<HostingRow> {
        self.view().rows(list)
    }

    pub(super) fn data_len(&self, list: HostingList) -> usize {
        self.view().data_len(list)
    }

    pub(crate) fn row_counts(&self) -> (usize, usize) {
        self.view().row_counts()
    }

    /// The count in a section's title: real items, not status rows.
    pub(crate) fn item_count(&self, list: HostingList) -> usize {
        self.view().item_count(list)
    }

    fn view_label(&self, list: HostingList) -> &'static str {
        match list {
            HostingList::Pulls => self.pr_view.label(),
            HostingList::Issues => self.issue_view.label(),
        }
    }

    /// The section's title: `PULL REQUESTS`, and `GITHUB ISSUES` /
    /// `GITLAB ISSUES` once a provider is chosen.
    pub(crate) fn section_title(&self, list: HostingList) -> String {
        self.view().section_title(list)
    }

    /// Text appended to a section's title: how old the data is, when it
    /// is not fresh.
    pub(crate) fn title_suffix(&self, list: HostingList) -> String {
        self.view().title_suffix(list)
    }

    pub(crate) fn head_pr(&self) -> Option<&PullRequestSummary> {
        let snap = self.linked_snapshot()?;
        let head = self.inputs.as_ref()?.head_branch.as_ref()?;
        if snap.head_branch.as_ref() != Some(head) {
            return None;
        }
        snap.head_pr.as_ref()
    }

    /// The branch's request, unless it is finished and
    /// `ui.hide_finished_pr_chip` hides it.
    fn shown_head_pr(&self) -> Option<&PullRequestSummary> {
        let hide = crate::live_settings::hide_finished_pr_chip();
        self.head_pr().filter(|pr| chip_visible(pr.state, hide))
    }

    /// The toolbar chip, when there is something to say.
    pub(crate) fn chip(&self) -> Option<ToolbarChip> {
        let snap = self.linked_snapshot().filter(|s| s.can_pulls)?;
        if let Some(pr) = self.shown_head_pr() {
            return Some(ToolbarChip {
                text: format!("{} {}", provider_tag(snap.kind), pr.iid_label),
                short: format!("{} {}", state_glyph(pr.state), pr.iid_label),
                icon: state_glyph(pr.state).to_string(),
                fg: th().hosting.chip_fg,
                bg: Some(state_color(pr.state)),
                hit: HostingHit::LinkedPrChip,
            });
        }
        // No request, or a finished one hidden by `ui.hide_finished_pr_chip`:
        // the dim "new request" chip, as if the branch had none.
        let inputs = self.inputs.as_ref()?;
        let known_none = snap.head_branch == inputs.head_branch && snap.problem.is_none();
        (inputs.head_pushed && known_none && inputs.head_branch.is_some()).then(|| ToolbarChip {
            text: format!("+ {}", msg::hosting_ui::PR_ABBR.get()),
            short: "+".to_string(),
            icon: "+".to_string(),
            fg: th().ui.muted,
            bg: None,
            hit: HostingHit::NewPrChip,
        })
    }

    /// The tooltip over the linked-PR chip.
    pub(crate) fn chip_tooltip(&self) -> Vec<String> {
        let Some(pr) = self.shown_head_pr() else {
            return vec![
                msg::hosting_ui::TIP_NO_PR.get().to_string(),
                msg::hosting_ui::TIP_START_PR.get().to_string(),
            ];
        };
        vec![
            format!("{} {}", pr.iid_label, pr.title),
            msg::hosting_ui::tip_pr_detail(
                state_word(pr.state),
                &pr.source_branch,
                &pr.target_branch,
                &pr.author,
            ),
            msg::hosting_ui::TIP_CLICK_OPEN.get().to_string(),
        ]
    }

    // -- painting -----------------------------------------------------------

    /// Paints one section (border, title, rows, scrollbar, header `+`)
    /// and registers its hit rects: the `+` first, then rows, then the
    /// header band, so an inner target always wins (`find_hit_target`
    /// returns the first match).
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn render_section(
        &self,
        frame: &mut Frame,
        area: Rect,
        list: HostingList,
        expanded: bool,
        active: bool,
        hovered: Option<&HitTarget>,
        states: &mut SidebarStates,
        hits: &mut Vec<(Rect, HitTarget)>,
    ) {
        let section = list.section();
        let rows = self.rows(list);
        let chevron = crate::fold_mark::glyph_open(expanded);
        let title = format!(
            "{chevron} {} {} ({}){}",
            section.icon(),
            self.section_title(list),
            self.item_count(list),
            self.title_suffix(list)
        );
        let title_w = display_width(&title) as u16;
        let header_style = if active {
            Style::default()
                .fg(th().ui.accent)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .fg(th().ui.muted)
                .add_modifier(Modifier::BOLD)
        };
        let title_style = if hovered == Some(&HitTarget::RefsSectionHeader(section)) {
            header_style.patch(plain_button_style(true))
        } else {
            header_style
        };
        let width = area.width.saturating_sub(2) as usize;
        let selected = states.state(section).selected();
        let items: Vec<ListItem> = if !expanded {
            Vec::new()
        } else if rows.is_empty() {
            vec![ListItem::new("  (none)").style(Style::default().fg(th().ui.muted))]
        } else {
            rows.iter()
                .enumerate()
                .map(|(i, row)| {
                    let is_hovered = hovered == Some(&HitTarget::Hosting(HostingHit::Row(list, i)));
                    self.row_item(list, row, width, is_hovered, selected == Some(i))
                })
                .collect()
        };
        let visible = area.height.saturating_sub(2) as usize;
        let mut render_state =
            list_render_state(states.state(section), states.follow(section), visible);
        frame.render_stateful_clipped(
            List::new(items)
                .block(
                    Block::default()
                        .title(Span::styled(title, title_style))
                        .border_type(BorderType::Rounded)
                        .borders(Borders::ALL)
                        .border_style(header_style),
                )
                .highlight_style(if active {
                    Style::default().bg(th().ui.selection)
                } else {
                    Style::default()
                }),
            area,
            &mut render_state,
        );
        *states.state_mut(section).offset_mut() = render_state.offset();
        if expanded {
            if let Some(track) =
                scrollbar::Track::vertical(area, rows.len(), states.state(section).offset())
            {
                scrollbar::render(frame, track, true, false);
            }
        }

        // The `+` (new pull request / new issue): right end of the header,
        // drawn by the same function as the REMOTE `+` -- same glyph,
        // color, alignment, and a hover tooltip naming the action. The
        // ISSUES one shows only once a provider and repository are chosen.
        let plus = match list {
            HostingList::Pulls => Some((
                HostingHit::NewPr,
                format!(" + {} ", msg::hosting_ui::NEW_PR.get()),
            )),
            HostingList::Issues if self.linked_by_state(HostingList::Issues) => Some((
                HostingHit::NewIssue,
                format!(" + {} ", msg::hosting_ui::NEW_ISSUE.get()),
            )),
            HostingList::Issues => None,
        };
        if let Some((hit, tooltip)) = plus {
            let target = HitTarget::Hosting(hit);
            crate::chrome::render_header_plus(
                frame,
                area,
                title_w,
                &tooltip,
                hovered == Some(&target),
                hovered == Some(&HitTarget::RefsSectionHeader(section)),
                target,
                hits,
            );
        }
        // The Board button, left of the ISSUES `+` (plan L2).
        // The `+`'s own tooltip covers the space the button takes, so it is
        // not drawn while the `+` is hovered.
        let plus_hot = hovered == Some(&HitTarget::Hosting(HostingHit::NewIssue));
        if crate::build_features::BOARD
            && list == HostingList::Issues
            && self.linked_by_state(HostingList::Issues)
            && !plus_hot
        {
            let target = HitTarget::Hosting(HostingHit::Board);
            let tip = format!(" {} ", msg::board::BUTTON_TIP.get());
            let rest = format!(" {} ", msg::board::BUTTON.get());
            crate::chrome::render_header_button(
                frame,
                area,
                title_w,
                (&rest, &tip),
                3,
                hovered == Some(&target),
                hovered == Some(&HitTarget::RefsSectionHeader(section)),
                target,
                hits,
            );
        }
        // Rows, only while expanded, only the ones in view.
        let mut rects = self.row_rects.borrow_mut();
        rects.retain(|(l, _, _)| *l != list);
        if expanded && area.height > 2 {
            let offset = states.state(section).offset();
            for (slot, index) in (offset..rows.len()).take(visible).enumerate() {
                let rect = Rect::new(
                    area.x + 1,
                    area.y + 1 + slot as u16,
                    area.width.saturating_sub(2),
                    1,
                );
                // The pencil of a pull request row, shown while the row is
                // hovered or selected. Pushed (and painted) from this same
                // walk, and before the row so the inner target wins.
                if list == HostingList::Pulls
                    && matches!(rows.get(index), Some(HostingRow::Pull(..)))
                {
                    if let Some(btn) = Self::edit_button_rect(rect) {
                        let edit = HitTarget::Hosting(HostingHit::EditRow(index));
                        let row_hot =
                            hovered == Some(&HitTarget::Hosting(HostingHit::Row(list, index)));
                        let hot = hovered == Some(&edit);
                        let shown = hot || row_hot || (active && selected == Some(index));
                        if shown {
                            let style = if hot {
                                plain_button_style(true)
                            } else {
                                Style::default().fg(th().ui.accent)
                            };
                            frame.render_clipped(
                                Paragraph::new(Span::styled(" \u{270E} ", style)),
                                btn,
                            );
                        }
                        hits.push((btn, edit));
                    }
                }
                hits.push((rect, HitTarget::Hosting(HostingHit::Row(list, index))));
                if let Some(row) = rows.get(index) {
                    crate::pointer_map::register(rect, self.pointer_kind(row));
                }
                rects.push((list, index, rect));
            }
        }
        hits.push((
            Rect::new(area.x, area.y, area.width, 1),
            HitTarget::RefsSectionHeader(section),
        ));
    }

    /// What the New issue form opens with: the profile and project the
    /// ISSUES section is linked to, or `None` while it is unlinked.
    pub(crate) fn issue_seed(&self) -> Option<crate::issue_form::IssueSeed> {
        let source = self
            .issues_source
            .as_ref()
            .filter(|s| !s.project.is_empty())?;
        let integration = (self.integrations)()
            .into_iter()
            .find(|i| i.id == source.integration)?;
        let provider = if integration.name.trim().is_empty() {
            integration.id.clone()
        } else {
            integration.name.clone()
        };
        // A notes folder relative to the repository is read against it, like
        // every other call that reaches the folder.
        let project = if integration.kind.capabilities().local_folder {
            gitpoulpi_hosting::resolve_local_folder(&self.repo_root, &source.project)
                .to_string_lossy()
                .into_owned()
        } else {
            source.project.clone()
        };
        Some(crate::issue_form::IssueSeed {
            integration,
            provider,
            project,
        })
    }

    /// What a row of the sidebar is for the pointer: the comboboxes and the
    /// group headers are the hand, the search box and the query field the
    /// I-beam, the items and notices rows (the arrow, lit as a whole).
    pub(crate) fn pointer_kind(&self, row: &HostingRow) -> crate::pointer_map::Kind {
        use crate::pointer_map::Kind;
        match row {
            HostingRow::Search => Kind::Field,
            HostingRow::IssueQuery => {
                if self.issues_family() == Some(gitpoulpi_config::ApiFamily::Trello) {
                    Kind::Hand
                } else {
                    Kind::Field
                }
            }
            HostingRow::Filter
            | HostingRow::IssueProvider
            | HostingRow::IssueRepo
            | HostingRow::PullProvider
            | HostingRow::PullRepo
            | HostingRow::Group(_)
            | HostingRow::Reconnect => Kind::Hand,
            HostingRow::Notice(_) | HostingRow::Pull(..) | HostingRow::Issue(_) => Kind::Plain,
        }
    }

    /// A combobox-looking row: the text (or its placeholder), padded, a
    /// `v` at the right, on the field background.
    fn combo_line(
        text: Option<&str>,
        placeholder: &str,
        width: usize,
        hovered: bool,
    ) -> Line<'static> {
        let band = width.saturating_sub(2);
        let bg = crate::input_bg(false, hovered);
        let (shown, fg) = match text {
            Some(t) => (t, th().ui.text),
            None => (placeholder, th().ui.muted),
        };
        let inner = band.saturating_sub(4);
        let mut body = truncate_with_ellipsis(shown, inner);
        let pad = inner.saturating_sub(display_width(&body));
        body.push_str(&" ".repeat(pad));
        Line::from(vec![
            Span::raw(" "),
            Span::styled(format!(" {body} "), Style::default().fg(fg).bg(bg)),
            Span::styled(
                "\u{25BE} ",
                Style::default()
                    .fg(if hovered { th().ui.text } else { th().ui.muted })
                    .bg(bg),
            ),
        ])
    }

    fn search_line(&self, list: HostingList, width: usize, hovered: bool) -> Line<'static> {
        let band = width.saturating_sub(2);
        let editing = self.editing == Some(list);
        let bg = crate::input_bg(editing, hovered);
        let query = self.query(list);
        let placeholder = match list {
            HostingList::Pulls => msg::hosting_ui::SEARCH_PRS,
            HostingList::Issues => msg::hosting_ui::SEARCH_ISSUES,
        };
        let (text, fg) = if query.is_empty() && !editing {
            (placeholder.get().to_string(), th().ui.muted)
        } else if editing {
            (format!("{query}\u{2588}"), th().ui.text)
        } else {
            (query.to_string(), th().ui.text)
        };
        let inner = band.saturating_sub(4);
        let mut body = truncate_with_ellipsis(&text, inner);
        let pad = inner.saturating_sub(display_width(&body));
        body.push_str(&" ".repeat(pad));
        Line::from(vec![
            Span::raw(" "),
            Span::styled(format!(" {body} "), Style::default().fg(fg).bg(bg)),
            Span::styled("\u{2315} ", Style::default().fg(th().ui.muted).bg(bg)),
        ])
    }

    /// The issue source's extra-query row: the lists dropdown (Trello) or a
    /// text field (Jira JQL, Azure WIQL).
    fn query_line(&self, width: usize, hovered: bool) -> Line<'static> {
        use gitpoulpi_config::ApiFamily;
        let family = self.issues_family();
        let saved = self.saved_query();
        if family == Some(ApiFamily::Trello) {
            let shown = (!saved.is_empty()).then_some(saved.as_str());
            return Self::combo_line(
                shown,
                msg::hosting_ui::FILTER_LISTS_ALL.get(),
                width,
                hovered,
            );
        }
        let placeholder = if family == Some(ApiFamily::AzureDevops) {
            msg::hosting_ui::FILTER_WIQL.get()
        } else if family == Some(ApiFamily::YouTrack) {
            msg::hosting_ui::FILTER_YOUTRACK.get()
        } else {
            msg::hosting_ui::FILTER_JQL.get()
        };
        let editing = self.editing_query;
        let bg = crate::input_bg(editing, hovered);
        let (text, fg) = if editing {
            (format!("{}\u{2588}", self.query_draft), th().ui.text)
        } else if saved.is_empty() {
            (placeholder.to_string(), th().ui.muted)
        } else {
            (saved, th().ui.text)
        };
        let inner = width.saturating_sub(2).saturating_sub(4);
        let mut body = truncate_with_ellipsis(&text, inner);
        let pad = inner.saturating_sub(display_width(&body));
        body.push_str(&" ".repeat(pad));
        Line::from(vec![
            Span::raw(" "),
            Span::styled(format!(" {body} "), Style::default().fg(fg).bg(bg)),
            Span::styled("\u{270E} ", Style::default().fg(th().ui.muted).bg(bg)),
        ])
    }

    fn row_item(
        &self,
        list: HostingList,
        row: &HostingRow,
        width: usize,
        hovered: bool,
        _selected: bool,
    ) -> ListItem<'static> {
        let hot = |style: Style| {
            if hovered {
                style.patch(plain_button_style(true))
            } else {
                style
            }
        };
        let room = width.saturating_sub(2);
        let snap = self.snapshot.as_ref();
        let line = match row {
            HostingRow::Reconnect => {
                let problem = match list {
                    HostingList::Pulls => snap.and_then(|s| s.problem.as_ref()),
                    HostingList::Issues => snap.and_then(|s| s.issues_problem.as_ref()),
                };
                let why = problem
                    .map(|p| match p {
                        Problem::NoToken => msg::hosting_ui::WHY_NO_TOKEN.get(),
                        _ => msg::hosting_ui::WHY_TOKEN_REJECTED.get(),
                    })
                    .unwrap_or(msg::hosting_ui::WHY_TOKEN.get());
                Line::from(vec![
                    Span::raw("  "),
                    Span::styled(
                        truncate_with_ellipsis(
                            &format!("\u{21BB} {}", msg::hosting_ui::reconnect(why)),
                            room,
                        ),
                        hot(Style::default()
                            .fg(th().status.warning)
                            .add_modifier(Modifier::BOLD)),
                    ),
                ])
            }
            HostingRow::Notice(text) => Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    truncate_with_ellipsis(text, room),
                    Style::default().fg(th().ui.muted),
                ),
            ]),
            HostingRow::Search => self.search_line(list, width, hovered),
            HostingRow::Filter => Self::combo_line(Some(self.view_label(list)), "", width, hovered),
            HostingRow::Group(group) => {
                let count = self.group_members(*group).len().to_string();
                let chev = crate::fold_mark::glyph(self.collapsed.contains(group));
                let left = format!(" {chev} {}", group.label());
                let gap = width
                    .saturating_sub(display_width(&left) + display_width(&count) + 1)
                    .max(1);
                Line::from(vec![
                    Span::styled(
                        truncate_with_ellipsis(&left, room),
                        hot(Style::default().fg(th().ui.text)),
                    ),
                    Span::raw(" ".repeat(gap)),
                    Span::styled(count, hot(Style::default().fg(th().ui.muted))),
                ])
            }
            HostingRow::Pull(_, i) => {
                let Some(pr) = snap.and_then(|s| s.prs.get(*i)) else {
                    return ListItem::new("");
                };
                let is_head = self.head_pr().is_some_and(|h| h.number == pr.number);
                let mut label_style = Style::default().fg(th().ui.text);
                if is_head {
                    label_style = Style::default()
                        .fg(th().ui.accent)
                        .add_modifier(Modifier::BOLD);
                }
                let label = format!("{}  {}", pr.iid_label, pr.title);
                Line::from(vec![
                    Span::raw("   "),
                    Span::styled(
                        state_glyph(pr.state),
                        hot(Style::default().fg(state_color(pr.state))),
                    ),
                    Span::raw(" "),
                    Span::styled(
                        truncate_with_ellipsis(&label, room.saturating_sub(4)),
                        hot(label_style),
                    ),
                ])
            }
            HostingRow::IssueProvider => {
                let label = snap.and_then(|s| s.issues_target.as_ref()).map(|t| {
                    format!(
                        "{} \u{00B7} {}",
                        provider_name(t.profile.kind),
                        t.profile.label
                    )
                });
                Self::combo_line(
                    label.as_deref(),
                    msg::hosting_ui::SELECT_PROVIDER.get(),
                    width,
                    hovered,
                )
            }
            HostingRow::IssueRepo => {
                let project = snap
                    .and_then(|s| s.issues_target.as_ref())
                    .map(|t| t.project.as_str())
                    .filter(|p| !p.is_empty());
                Self::combo_line(
                    project,
                    msg::hosting_ui::SELECT_REPOSITORY.get(),
                    width,
                    hovered,
                )
            }
            HostingRow::PullProvider => {
                let label = snap.and_then(|s| s.pulls_target.as_ref()).map(|t| {
                    format!(
                        "{} \u{00B7} {}",
                        provider_name(t.profile.kind),
                        t.profile.label
                    )
                });
                Self::combo_line(
                    label.as_deref(),
                    msg::hosting_ui::SELECT_PROVIDER.get(),
                    width,
                    hovered,
                )
            }
            HostingRow::PullRepo => {
                let project = snap
                    .and_then(|s| s.pulls_target.as_ref())
                    .map(|t| t.project.as_str())
                    .filter(|p| !p.is_empty());
                Self::combo_line(
                    project,
                    msg::hosting_ui::SELECT_REPOSITORY.get(),
                    width,
                    hovered,
                )
            }
            HostingRow::Issue(i) => {
                let Some(issue) = snap.and_then(|s| s.issues.get(*i)) else {
                    return ListItem::new("");
                };
                let (glyph, color) = match issue.state {
                    IssueState::Open => ("\u{25CF}", th().refs.local),
                    IssueState::Closed => ("\u{25CB}", th().ui.muted),
                };
                let label = format!("{}  {}", issue.iid_label, issue.title);
                // The provider's own words, dim: Jira / Azure type and
                // status, a Trello card's list. Left out when the sidebar is
                // narrow.
                let both = [issue.kind_label.as_deref(), issue.status_label.as_deref()]
                    .into_iter()
                    .flatten()
                    .collect::<Vec<_>>()
                    .join(" \u{00B7} ");
                let room_for_label = room.saturating_sub(1);
                // The fullest wording that fits after the whole title (the title comes first):
                // type and status, else the status, else the type.
                let extra = [
                    both,
                    issue.status_label.clone().unwrap_or_default(),
                    issue.kind_label.clone().unwrap_or_default(),
                ]
                .into_iter()
                .find(|e| {
                    !e.is_empty() && display_width(&label) + 2 + display_width(e) <= room_for_label
                })
                .unwrap_or_default();
                let extra_w = display_width(&extra);
                if extra.is_empty() {
                    Line::from(vec![
                        Span::raw(" "),
                        Span::styled(glyph, hot(Style::default().fg(color))),
                        Span::raw(" "),
                        Span::styled(
                            truncate_with_ellipsis(&label, room_for_label),
                            hot(Style::default().fg(th().ui.text)),
                        ),
                    ])
                } else {
                    let label = truncate_with_ellipsis(&label, room_for_label - extra_w - 2);
                    Line::from(vec![
                        Span::raw(" "),
                        Span::styled(glyph, hot(Style::default().fg(color))),
                        Span::raw(" "),
                        Span::styled(label, hot(Style::default().fg(th().ui.text))),
                        Span::raw("  "),
                        Span::styled(extra, hot(Style::default().fg(th().ui.muted))),
                    ])
                }
            }
            HostingRow::IssueQuery => self.query_line(width, hovered),
        };
        ListItem::new(line)
    }

    // -- dropdowns and the search box ----------------------------------------

    /// What `#123` / `!45` / `ABC-12` in a commit message mean: only for a
    /// repository whose issues / pull requests are linked (`issues_source` /
    /// `pulls_source`), else nothing (`gitpoulpi_hosting::refs`, plan D7).
    pub(crate) fn token_bases(&self) -> crate::issue_tokens::TokenBases {
        use gitpoulpi_config::ApiFamily;
        use gitpoulpi_hosting::refs::{IssueLinker, KnownCard, PullLinker, RefContext};
        let issue_target = self
            .issues_source
            .as_ref()
            .and(self.snapshot.as_ref())
            .and_then(|s| s.issues_target.as_ref())
            .filter(|t| !t.project.is_empty());
        let pull = self
            .pulls_source
            .as_ref()
            .and(self.linked_snapshot())
            .filter(|s| !s.project.is_empty());
        // Jira keys come from the profile's cached project list; Trello's
        // `#N` from the cached cards.
        let project_keys: Vec<String> = issue_target
            .and_then(|t| self.projects.get(&t.profile.id))
            .map(|state| match state {
                ProjectsState::Ready(all) => all.iter().map(|p| p.full_name.clone()).collect(),
                _ => Vec::new(),
            })
            .unwrap_or_default();
        let cards: Vec<KnownCard> = issue_target
            .and(self.snapshot.as_ref())
            .map(|s| {
                s.issues
                    .iter()
                    .filter(|i| !i.web_url.is_empty())
                    .map(|i| KnownCard {
                        id_short: i.number,
                        url: i.web_url.clone(),
                    })
                    .collect()
            })
            .unwrap_or_default();
        let key = format!(
            "{:?}|{:?}|{}|{:?}",
            issue_target.map(|t| (&t.profile.id, &t.project)),
            pull.map(|s| (&s.profile_id, &s.project, s.kind)),
            project_keys.len(),
            cards.iter().map(|c| c.id_short).collect::<Vec<_>>()
        );
        if let Some((k, bases)) = self.token_bases.borrow().as_ref() {
            if *k == key {
                return bases.clone();
            }
        }
        let integrations = (self.integrations)();
        let mut bases = RefContext::default();
        if let Some(t) = issue_target {
            if let Some(i) = integrations.iter().find(|i| i.id == t.profile.id) {
                if let Some(web_base) = gitpoulpi_hosting::integration_web_base(i) {
                    let family = i.kind.family();
                    bases.issues = Some(IssueLinker {
                        family,
                        web_base,
                        project: t.project.clone(),
                        project_keys,
                        cards: if family == ApiFamily::Trello {
                            cards
                        } else {
                            Vec::new()
                        },
                    });
                }
            }
        }
        if let Some(s) = pull {
            if let Some(i) = integrations
                .iter()
                .find(|i| Some(&i.id) == s.profile_id.as_ref())
            {
                if let Some(web_base) = gitpoulpi_hosting::integration_web_base(i) {
                    bases.pulls = Some(PullLinker {
                        family: i.kind.family(),
                        web_base,
                        project: s.project.clone(),
                    });
                }
            }
        }
        *self.token_bases.borrow_mut() = Some((key, bases.clone()));
        bases
    }

    /// The selected commit's resolvable references, as `(label, url)`.
    pub(crate) fn set_commit_references(&mut self, refs: Vec<(String, String)>) {
        self.commit_references = refs;
    }

    /// A message for the status line, once.
    pub(crate) fn take_notice(&mut self) -> Option<String> {
        self.notice.take()
    }

    /// True while a menu or a dropdown owns the mouse and the keyboard.
    pub(crate) fn modal_open(&self) -> bool {
        self.menu.is_some() || self.picker.is_some()
    }
}
