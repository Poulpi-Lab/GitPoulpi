//! Menus, row activation, effects and the palette commands.

use super::*;
use crate::meta_picker::MetaField;

impl HostingUi {
    // -- menus --------------------------------------------------------------

    fn profile_entries(&self) -> Vec<Entry<HostingAction>> {
        let mut entries: Vec<Entry<HostingAction>> = self
            .profiles
            .iter()
            .map(|(id, label)| {
                let current = self.snapshot.as_ref().and_then(|s| s.profile_id.as_deref())
                    == Some(id.as_str());
                Entry::action(
                    if current {
                        msg::hosting_ui::profile_current(label)
                    } else {
                        label.clone()
                    },
                    HostingAction::UseProfile(id.clone()),
                )
            })
            .collect();
        entries.push(Entry::action(
            msg::hosting_ui::AUTOMATIC_PROFILE.get(),
            HostingAction::AutomaticProfile,
        ));
        entries
    }

    /// How a pull request's source gets onto a local branch, by host
    /// (`gitpoulpi_hosting::CheckoutSource`):
    ///
    /// - the host keeps the head in a ref of the target repository (GitHub,
    ///   GitLab, Bitbucket Data Center): a fork's is fetched from there, any
    ///   other request just checks its source branch out;
    /// - the head is an ordinary branch (Azure DevOps, Bitbucket Cloud): it
    ///   is fetched into a local `pr/N` and checked out; one from a fork is
    ///   refused, its branch lives in another repository and GitPoulpi will
    ///   not add a remote behind the user's back.
    fn checkout_entry(
        snap: &HostingSnapshot,
        pr: &PullRequestSummary,
        plain_label: &'static str,
    ) -> Entry<HostingAction> {
        use gitpoulpi_hosting::CheckoutSource;
        let Some(kind) = snap.kind else {
            return Entry::disabled(plain_label, msg::hosting_ui::WHY_NOT_LINKED.get());
        };
        match (gitpoulpi_hosting::spec_for(kind.family()).checkout)(pr) {
            CheckoutSource::HeadRef { .. } if pr.source_project.is_some() => Entry::action(
                msg::hosting_ui::CHECKOUT_PR_FORK.get(),
                HostingAction::CheckoutPullRequestHead { number: pr.number },
            ),
            CheckoutSource::HeadRef { .. } => Entry::action(
                plain_label,
                HostingAction::CheckoutBranch {
                    branch: pr.source_branch.clone(),
                    remote: snap.remote.clone(),
                },
            ),
            // Azure never says where a fork lives, Bitbucket Cloud does: either
            // way the branch is in another repository.
            CheckoutSource::Branch { fork_url, .. }
                if fork_url.is_some() || pr.source_project.is_some() =>
            {
                Entry::disabled(
                    msg::hosting_ui::CHECKOUT_PR_FORK.get(),
                    msg::hosting_ui::FROM_FORK_ADD_REMOTE.get(),
                )
            }
            CheckoutSource::Branch { .. } => Entry::action(
                msg::hosting_ui::CHECKOUT_PR_BRANCH.get(),
                HostingAction::CheckoutPullRequestHead { number: pr.number },
            ),
        }
    }

    fn remote_of(&self) -> String {
        self.linked_snapshot()
            .map(|s| s.remote.clone())
            .unwrap_or_default()
    }

    /// The menu of a sidebar row.
    fn row_menu(
        &self,
        list: HostingList,
        index: usize,
        col: u16,
        row: u16,
    ) -> Option<ContextMenu<HostingAction>> {
        let rows = self.rows(list);
        let snap = self.snapshot.as_ref()?;
        match rows.get(index)? {
            HostingRow::Pull(_, i) => {
                let pr = snap.prs.get(*i)?;
                let checkout =
                    Self::checkout_entry(snap, pr, msg::hosting_ui::CHECKOUT_SOURCE.get());
                let view: Entry<HostingAction> = if pr.source_project.is_some() {
                    Entry::disabled(
                        msg::hosting_ui::VIEW_SOURCE.get(),
                        msg::hosting_ui::FROM_FORK.get(),
                    )
                } else {
                    Entry::action(
                        msg::hosting_ui::VIEW_SOURCE.get(),
                        HostingAction::ViewBranch {
                            branch: pr.source_branch.clone(),
                            remote: snap.remote.clone(),
                        },
                    )
                };
                let provider = snap
                    .kind
                    .map_or(msg::hosting_ui::PROVIDER.get(), provider_name);
                let review: Vec<Entry<HostingAction>> = if self.review_prs() {
                    vec![
                        Entry::action(
                            msg::hosting_ui::REVIEW.get(),
                            HostingAction::Review { number: pr.number },
                        ),
                        Entry::Separator,
                    ]
                } else if self.conversation_prs() {
                    vec![
                        Entry::action(
                            msg::item_view::OPEN_CONVERSATION.get(),
                            HostingAction::Conversation {
                                number: pr.number,
                                pr: true,
                            },
                        ),
                        Entry::Separator,
                    ]
                } else {
                    Vec::new()
                };
                Some(ContextMenu::new(
                    format!("{} {}", pr.iid_label, pr.title),
                    review
                        .into_iter()
                        .chain(vec![
                            Entry::action(
                                msg::hosting_ui::OPEN_BROWSER.get(),
                                HostingAction::OpenUrl(pr.web_url.clone()),
                            ),
                            Entry::action(
                                msg::hosting_ui::edit_on(provider),
                                HostingAction::OpenUrl(edit_url(snap.kind, &pr.web_url)),
                            ),
                            Entry::action(
                                msg::hosting_ui::COPY_URL.get(),
                                HostingAction::CopyText {
                                    what: msg::hosting_ui::NOUN_PR_URL.get().into(),
                                    text: pr.web_url.clone(),
                                },
                            ),
                            Entry::action(
                                msg::hosting_ui::COPY_NUMBER.get(),
                                HostingAction::CopyText {
                                    what: msg::hosting_ui::NOUN_PR_NUMBER.get().into(),
                                    text: pr.iid_label.clone(),
                                },
                            ),
                            Entry::Separator,
                            checkout,
                            view,
                        ])
                        .chain(if self.meta_prs() {
                            std::iter::once(Entry::Separator)
                                .chain(meta::menu_entries(
                                    self.linked_snapshot().and_then(|s| s.kind),
                                    true,
                                    pr.number,
                                ))
                                .collect::<Vec<_>>()
                        } else {
                            Vec::new()
                        })
                        .collect(),
                    col,
                    row,
                ))
            }
            HostingRow::Issue(i) => {
                let issue = snap.issues.get(*i)?;
                let local = self.issues_local();
                Some(
                    ContextMenu::new(
                        format!("{} {}", issue.iid_label, issue.title),
                        self.conversation_issues()
                            .then(|| {
                                Entry::action(
                                    msg::item_view::OPEN_CONVERSATION.get(),
                                    HostingAction::Conversation {
                                        number: issue.number,
                                        pr: false,
                                    },
                                )
                            })
                            .into_iter()
                            .chain(vec![
                            Entry::action(
                                if local {
                                    msg::local_issues::OPEN_NOTE.get()
                                } else {
                                    msg::hosting_ui::OPEN_BROWSER.get()
                                },
                                HostingAction::OpenUrl(issue.web_url.clone()),
                            ),
                            Entry::action(
                                msg::hosting_ui::COPY_URL.get(),
                                HostingAction::CopyText {
                                    what: msg::hosting_ui::NOUN_ISSUE_URL.get().into(),
                                    text: issue.web_url.clone(),
                                },
                            ),
                            Entry::action(
                                msg::hosting_ui::COPY_NUMBER.get(),
                                HostingAction::CopyText {
                                    what: msg::hosting_ui::NOUN_ISSUE_NUMBER.get().into(),
                                    text: issue.iid_label.clone(),
                                },
                            ),
                            Entry::Separator,
                            Entry::action(
                                msg::hosting_ui::BRANCH_FROM_ISSUE.get(),
                                HostingAction::NewBranchFromIssue(issue_branch_for(
                                    snap.issues_target.as_ref().map(|t| t.profile.kind.family()),
                                    issue,
                                )),
                            ),
                        ])
                        .chain(if local {
                            std::iter::once(Entry::Separator)
                                .chain(Self::local_entries(issue))
                                .collect::<Vec<_>>()
                        } else {
                            Vec::new()
                        })
                        .chain(if self.meta_issues() {
                            std::iter::once(Entry::Separator)
                                .chain(meta::menu_entries(
                                    self.snapshot
                                        .as_ref()
                                        .and_then(|s| s.issues_target.as_ref())
                                        .map(|t| t.profile.kind),
                                    false,
                                    issue.number,
                                ))
                                .collect::<Vec<_>>()
                        } else {
                            Vec::new()
                        })
                        .collect(),
                        col,
                        row,
                    )
                    // The grayed-out Obsidian row keeps a few words; the whole
                    // reason is on the status line when it is clicked.
                    .with_note(
                        msg::local_issues::OPEN_IN_OBSIDIAN.get(),
                        msg::local_issues::WHY_NO_VAULT.get(),
                    ),
                )
            }
            HostingRow::Reconnect => Some(ContextMenu::new(
                msg::hosting_ui::MENU_INTEGRATION.get(),
                vec![Entry::action(
                    msg::hosting_ui::RECONNECT_PREFS.get(),
                    HostingAction::Preferences(match list {
                        HostingList::Pulls => snap.profile_id.clone(),
                        HostingList::Issues => {
                            snap.issues_target.as_ref().map(|t| t.profile.id.clone())
                        }
                    }),
                )],
                col,
                row,
            )),
            _ => None,
        }
    }

    /// Opens the menu of a sidebar row (right-click, `.`).
    pub(crate) fn open_row_menu(
        &mut self,
        list: HostingList,
        index: usize,
        col: u16,
        row: u16,
    ) -> bool {
        self.menu = self.row_menu(list, index, col, row);
        self.menu.is_some()
    }

    /// Opens the toolbar chip's menu (right-click / `Ctrl+G g` twin).
    pub(crate) fn open_chip_menu(&mut self, col: u16, row: u16) -> bool {
        let Some(snap) = self.linked_snapshot() else {
            return false;
        };
        let mut entries: Vec<Entry<HostingAction>> = Vec::new();
        let caption;
        if let Some(pr) = self.head_pr() {
            caption = format!("{} {}", pr.iid_label, pr.title);
            entries.push(Entry::action(
                msg::hosting_ui::OPEN_BROWSER.get(),
                HostingAction::OpenUrl(pr.web_url.clone()),
            ));
            entries.push(Entry::action(
                msg::hosting_ui::COPY_URL.get(),
                HostingAction::CopyText {
                    what: msg::hosting_ui::NOUN_PR_URL.get().into(),
                    text: pr.web_url.clone(),
                },
            ));
        } else {
            caption = msg::hosting_ui::MENU_PR.get().to_string();
            if let Some(branch) = self.inputs.as_ref().and_then(|i| i.head_branch.clone()) {
                entries.push(Entry::action(
                    msg::hosting_ui::START_PR.get(),
                    HostingAction::StartPullRequest(branch),
                ));
            }
        }
        entries.push(Entry::action(
            msg::hosting_ui::REFRESH.get(),
            HostingAction::Refresh,
        ));
        entries.push(Entry::Separator);
        entries.push(Entry::submenu(
            msg::hosting_ui::switch_profile(&snap.profile_label),
            self.profile_entries(),
        ));
        self.menu = Some(ContextMenu::new(caption, entries, col, row));
        true
    }

    // -- acting -------------------------------------------------------------

    /// The default action of a row (Enter, click on a control,
    /// double-click on an item).
    pub(crate) fn activate_row(
        &mut self,
        list: HostingList,
        index: usize,
        clipboard: &mut crate::clipboard::Clipboard,
    ) -> HostingEffect {
        let Some(row) = self.rows(list).get(index).cloned() else {
            return HostingEffect::None;
        };
        let snap_profile = match list {
            HostingList::Pulls => self.linked_snapshot().and_then(|s| s.profile_id.clone()),
            HostingList::Issues => self
                .snapshot
                .as_ref()
                .and_then(|s| s.issues_target.as_ref())
                .map(|t| t.profile.id.clone()),
        };
        // Where a dropdown hangs if its row is not on screen.
        let fallback = (2, 2);
        let action = match row {
            HostingRow::Reconnect => HostingAction::Preferences(snap_profile),
            HostingRow::Notice(_) => return HostingEffect::None,
            HostingRow::Search => {
                self.picker = None;
                self.editing = Some(list);
                return HostingEffect::None;
            }
            HostingRow::Filter => {
                let target = match list {
                    HostingList::Pulls => PickerTarget::PrFilter,
                    HostingList::Issues => PickerTarget::IssueFilter,
                };
                return self.open_picker(target, fallback);
            }
            HostingRow::IssueProvider => {
                return self.open_picker(PickerTarget::IssueProvider, fallback)
            }
            HostingRow::IssueRepo => return self.open_picker(PickerTarget::IssueRepo, fallback),
            HostingRow::IssueQuery => return self.activate_query(fallback),
            HostingRow::PullProvider => {
                return self.open_picker(PickerTarget::PullProvider, fallback)
            }
            HostingRow::PullRepo => return self.open_picker(PickerTarget::PullRepo, fallback),
            HostingRow::Group(group) => {
                if !self.collapsed.remove(&group) {
                    self.collapsed.insert(group);
                }
                return HostingEffect::None;
            }
            HostingRow::Pull(_, i) => match self.linked_snapshot().and_then(|s| s.prs.get(i)) {
                // Where a review exists, Enter opens it; elsewhere the page.
                Some(pr) if self.review_prs() => HostingAction::Review { number: pr.number },
                Some(pr) if self.conversation_prs() => HostingAction::Conversation {
                    number: pr.number,
                    pr: true,
                },
                Some(pr) => HostingAction::OpenUrl(pr.web_url.clone()),
                None => return HostingEffect::None,
            },
            HostingRow::Issue(i) => match self.snapshot.as_ref().and_then(|s| s.issues.get(i)) {
                // Where the conversation can be read, Enter opens it; `o`
                // and the row menu keep the browser.
                Some(issue) if self.conversation_issues() => HostingAction::Conversation {
                    number: issue.number,
                    pr: false,
                },
                Some(issue) => HostingAction::OpenUrl(issue.web_url.clone()),
                None => return HostingEffect::None,
            },
        };
        self.run(action, clipboard)
    }

    /// The effect "check out this pull request" has for the Radar: the
    /// plain branch checkout when the request's branch is in this repository
    /// and known here, else the fetch of its head into a local branch (a
    /// fork's, or a branch not fetched yet).
    pub(crate) fn checkout_pr_effect(
        &mut self,
        number: u64,
        refs: &[gitpoulpi_core::domain::Ref],
        clipboard: &mut crate::clipboard::Clipboard,
    ) -> HostingEffect {
        let same_repo = self.linked_snapshot().and_then(|s| {
            let pr = s.prs.iter().find(|p| p.number == number)?;
            pr.source_project
                .is_none()
                .then(|| (pr.source_branch.clone(), s.remote.clone()))
        });
        if let Some((branch, remote)) = same_repo {
            if branch_target(refs, &remote, &branch).is_some() {
                return HostingEffect::Checkout { branch, remote };
            }
        }
        self.run(HostingAction::CheckoutPullRequestHead { number }, clipboard)
    }

    /// Runs a menu entry / row action.
    pub(crate) fn run(
        &mut self,
        action: HostingAction,
        clipboard: &mut crate::clipboard::Clipboard,
    ) -> HostingEffect {
        self.menu = None;
        match action {
            HostingAction::OpenUrl(url) => match crate::repo_menu::open_url(&url) {
                Ok(()) => HostingEffect::Message(msg::hosting_ui::opened_in_browser(&url)),
                Err(err) => HostingEffect::Message(crate::activity_log::report_problem_clear(
                    crate::activity_log::ProblemSource::Tool,
                    &err,
                )),
            },
            HostingAction::CopyText { what, text } => {
                HostingEffect::Message(clipboard.copy(&what, &text))
            }
            HostingAction::Refresh => {
                self.demand();
                HostingEffect::Message(msg::hosting_ui::REFRESHING.get().into())
            }
            HostingAction::UseProfile(id) => {
                self.pin_profile(Some(id.clone()));
                HostingEffect::Message(msg::hosting_ui::now_uses_integration(&id))
            }
            HostingAction::UnlinkIssues => {
                self.clear_issues_source();
                HostingEffect::Message(msg::hosting_ui::ISSUES_UNLINKED.get().into())
            }
            HostingAction::UnlinkPulls => {
                self.clear_pulls_source();
                HostingEffect::Message(msg::hosting_ui::PULLS_UNLINKED.get().into())
            }
            HostingAction::AutomaticProfile => {
                self.clear_pulls_source();
                HostingEffect::Message(msg::hosting_ui::PULLS_UNLINKED.get().into())
            }
            HostingAction::LocalState { number, closed } => self.local_set_state(number, closed),
            HostingAction::OpenObsidian(url) => self.local_open_obsidian(&url),
            HostingAction::Preferences(id) => HostingEffect::Preferences(id),
            HostingAction::CheckoutPullRequestHead { number } => {
                use gitpoulpi_hosting::CheckoutSource;
                let found = self.linked_snapshot().and_then(|s| {
                    let pr = s.prs.iter().find(|p| p.number == number)?;
                    let kind = s.kind?;
                    Some((
                        (gitpoulpi_hosting::spec_for(kind.family()).checkout)(pr),
                        pr,
                    ))
                });
                let remote = match self.remote_of() {
                    r if r.is_empty() => "origin".to_string(),
                    r => r,
                };
                match found {
                    Some((CheckoutSource::HeadRef { remote_ref, local }, _)) => {
                        HostingEffect::CheckoutPullRequestHead {
                            remote,
                            remote_ref,
                            branch: local,
                        }
                    }
                    Some((
                        CheckoutSource::Branch {
                            fork_url,
                            branch,
                            local,
                        },
                        pr,
                    )) => match fork_url.or_else(|| pr.source_project.clone()) {
                        Some(fork) => HostingEffect::Message(msg::hosting_ui::fork_needs_remote(
                            number, fork, branch,
                        )),
                        None => HostingEffect::CheckoutPullRequestHead {
                            remote,
                            remote_ref: format!("refs/heads/{branch}"),
                            branch: local,
                        },
                    },
                    None => HostingEffect::Message(msg::hosting_ui::NO_PR_HEADS.get().into()),
                }
            }
            HostingAction::CheckoutBranch { branch, remote } => {
                HostingEffect::Checkout { branch, remote }
            }
            HostingAction::ViewBranch { branch, remote } => {
                HostingEffect::ViewBranch { branch, remote }
            }
            HostingAction::NewBranchFromIssue(name) => HostingEffect::NewBranch(name),
            HostingAction::StartPullRequest(branch) => HostingEffect::StartPullRequest(branch),
            HostingAction::EditMeta { field, pr, number } => self.meta_action(field, pr, number),
            HostingAction::Review { number } => self.review_effect(number),
            HostingAction::Conversation { number, pr } => self.item_effect(number, pr),
            HostingAction::PushForReview { branch } => self.push_review_effect(branch),
        }
    }

    /// "Use integration profile for this repo": saved in the repo state,
    /// then asked again.
    fn pin_profile(&mut self, id: Option<String>) {
        let path = crate::remotes_ui::repo_state_path(&self.repo_root);
        let mut state = gitpoulpi_session::RepoState::load(&path).unwrap_or_default();
        state.hosting_integration = id.clone();
        let _ = state.save(&path);
        self.override_id = id;
        self.demand();
    }

    /// A click on the toolbar chip.
    pub(crate) fn click_chip(
        &mut self,
        hit: &HostingHit,
        clipboard: &mut crate::clipboard::Clipboard,
    ) -> HostingEffect {
        match hit {
            HostingHit::LinkedPrChip => match self.head_pr().map(|p| p.web_url.clone()) {
                Some(url) => self.run(HostingAction::OpenUrl(url), clipboard),
                None => HostingEffect::None,
            },
            HostingHit::NewPrChip => match self.inputs.as_ref().and_then(|i| i.head_branch.clone())
            {
                Some(branch) => self.run(HostingAction::StartPullRequest(branch), clipboard),
                None => HostingEffect::None,
            },
            _ => HostingEffect::None,
        }
    }

    // -- palette ------------------------------------------------------------

    pub(crate) fn availability(&self) -> HostingAvailability {
        let linked = self.linked_snapshot().is_some();
        HostingAvailability {
            linked,
            head_pr: self.head_pr().is_some(),
            has_prs: self.data_len(HostingList::Pulls) > 0,
            has_issues: self.data_len(HostingList::Issues) > 0,
            has_commit_references: !self.commit_references.is_empty(),
            issues_linked: self.token_bases().issues.is_some(),
            has_profiles: linked
                || !self.profiles.is_empty()
                || self
                    .snapshot
                    .as_ref()
                    .is_some_and(|s| !s.issue_profiles.is_empty()),
            has_issue_profiles: self
                .snapshot
                .as_ref()
                .is_some_and(|s| !s.issue_profiles.is_empty()),
            issue_provider: self
                .snapshot
                .as_ref()
                .is_some_and(|s| s.issues_target.is_some()),
            issue_repo: self
                .snapshot
                .as_ref()
                .and_then(|s| s.issues_target.as_ref())
                .is_some_and(|t| !t.project.is_empty()),
            issue_query: self.issues_family().is_some_and(takes_issue_query),
            issues_pinned: self.issues_source.is_some(),
            has_pull_profiles: self
                .snapshot
                .as_ref()
                .is_some_and(|s| !s.pull_profiles.is_empty()),
            pull_provider: self
                .snapshot
                .as_ref()
                .is_some_and(|s| s.pulls_target.is_some()),
            pulls_pinned: self.pulls_source.is_some() || self.override_id.is_some(),
            profile_pinned: self.override_id.is_some(),
            meta_prs: self.meta_prs() && self.data_len(HostingList::Pulls) > 0,
            meta_issues: self.meta_issues() && self.data_len(HostingList::Issues) > 0,
            assignment_blocked: gitpoulpi_daemon::hosting_scope::assignment_blocked(
                &self.repo_root,
            ),
            local_issues: self.issues_local(),
            gerrit_pulls: self.gerrit_pulls(),
        }
    }

    /// Whether the composer's `#` button has a linked issue list to offer.
    pub(crate) fn issue_ref_available(&self) -> bool {
        self.command_unavailable(HostingCommand::InsertIssueRef)
            .is_none()
    }

    /// Why `cmd` cannot run now, or `None` when it can.
    pub(crate) fn command_unavailable(&self, cmd: HostingCommand) -> Option<&'static str> {
        command_unavailable_from(cmd, self.availability())
    }

    /// Runs a palette command. `anchor` is where a chooser menu hangs.
    pub(crate) fn run_command(
        &mut self,
        cmd: HostingCommand,
        anchor: (u16, u16),
        clipboard: &mut crate::clipboard::Clipboard,
    ) -> HostingEffect {
        if let Some(why) = self.command_unavailable(cmd) {
            return HostingEffect::Message(why.to_string());
        }
        self.editing = None;
        let profile = self.linked_snapshot().and_then(|s| s.profile_id.clone());
        match cmd {
            HostingCommand::OpenLinkedPr => match self.head_pr().map(|p| p.web_url.clone()) {
                Some(url) => self.run(HostingAction::OpenUrl(url), clipboard),
                None => HostingEffect::None,
            },
            HostingCommand::EditLinkedPr => match self.head_pr().map(|p| p.web_url.clone()) {
                Some(url) => {
                    let kind = self.linked_snapshot().and_then(|s| s.kind);
                    self.run(HostingAction::OpenUrl(edit_url(kind, &url)), clipboard)
                }
                None => HostingEffect::None,
            },
            HostingCommand::CopyLinkedPrUrl => match self.head_pr().map(|p| p.web_url.clone()) {
                Some(text) => self.run(
                    HostingAction::CopyText {
                        what: msg::hosting_ui::NOUN_PR_URL.get().into(),
                        text,
                    },
                    clipboard,
                ),
                None => HostingEffect::None,
            },
            HostingCommand::RefreshHosting => self.run(HostingAction::Refresh, clipboard),
            // A Gerrit change is created by pushing, not by a form.
            HostingCommand::CreatePullRequest if self.gerrit_pulls() => {
                self.push_review_menu(anchor)
            }
            HostingCommand::CreatePullRequest => HostingEffect::CreatePullRequest,
            HostingCommand::PushForReview => self.push_review_menu(anchor),
            HostingCommand::CreateIssue => match self.issue_seed() {
                Some(seed) => HostingEffect::NewIssue(Box::new(seed)),
                None => HostingEffect::Message(msg::hosting_ui::ISSUES_PICK_BOTH.get().into()),
            },
            HostingCommand::SearchPullRequests => {
                self.picker = None;
                self.editing = Some(HostingList::Pulls);
                HostingEffect::Focus(HostingList::Pulls)
            }
            HostingCommand::SearchIssues => {
                self.picker = None;
                self.editing = Some(HostingList::Issues);
                HostingEffect::Focus(HostingList::Issues)
            }
            HostingCommand::ChoosePullRequestFilter => {
                self.open_picker(PickerTarget::PrFilter, anchor)
            }
            HostingCommand::ChooseIssueFilter => {
                self.open_picker(PickerTarget::IssueFilter, anchor)
            }
            HostingCommand::EditIssueQuery => match self.activate_query(anchor) {
                HostingEffect::None => HostingEffect::Focus(HostingList::Issues),
                other => other,
            },
            HostingCommand::ChooseIssueProvider => {
                self.open_picker(PickerTarget::IssueProvider, anchor)
            }
            HostingCommand::ChooseIssueRepository => {
                self.open_picker(PickerTarget::IssueRepo, anchor)
            }
            HostingCommand::UnlinkIssueSource => self.run(HostingAction::UnlinkIssues, clipboard),
            HostingCommand::ChoosePullProvider => {
                self.open_picker(PickerTarget::PullProvider, anchor)
            }
            HostingCommand::ChoosePullRepository => {
                self.open_picker(PickerTarget::PullRepo, anchor)
            }
            HostingCommand::UnlinkPullSource => self.run(HostingAction::UnlinkPulls, clipboard),
            HostingCommand::UnlinkProfile => self.run(HostingAction::AutomaticProfile, clipboard),
            HostingCommand::IntegrationsPreferences => HostingEffect::Preferences(profile),
            HostingCommand::ConnectIntegration => HostingEffect::Preferences(None),
            HostingCommand::RadarOptions => HostingEffect::RadarOptions,
            HostingCommand::CheckoutPullRequest => {
                let entries: Vec<Entry<HostingAction>> = self
                    .linked_snapshot()
                    .map(|s| {
                        s.prs
                            .iter()
                            .take(40)
                            .map(|pr| {
                                let label = format!("{}  {}", pr.iid_label, pr.title);
                                match Self::checkout_entry(s, pr, "") {
                                    // The picker lists requests: the entry's own
                                    // wording is for the row menu.
                                    Entry::Action { action, .. } => Entry::action(label, action),
                                    other => other,
                                }
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                self.menu = Some(ContextMenu::new(
                    msg::hosting_ui::CHECKOUT_PR_TITLE.get(),
                    entries,
                    anchor.0,
                    anchor.1,
                ));
                HostingEffect::None
            }
            HostingCommand::OpenCommitReference => {
                if let [(_, url)] = self.commit_references.as_slice() {
                    let url = url.clone();
                    return self.run(HostingAction::OpenUrl(url), clipboard);
                }
                let entries: Vec<Entry<HostingAction>> = self
                    .commit_references
                    .iter()
                    .map(|(label, url)| {
                        Entry::action(label.clone(), HostingAction::OpenUrl(url.clone()))
                    })
                    .collect();
                self.menu = Some(ContextMenu::new(
                    msg::hosting_ui::OPEN_REFERENCE.get(),
                    entries,
                    anchor.0,
                    anchor.1,
                ));
                HostingEffect::None
            }
            HostingCommand::InsertIssueRef => self.open_picker(PickerTarget::IssueRef, anchor),
            HostingCommand::OpenIssue => {
                let entries: Vec<Entry<HostingAction>> = self
                    .snapshot
                    .as_ref()
                    .map(|s| {
                        s.issues
                            .iter()
                            .take(40)
                            .map(|i: &IssueSummary| {
                                Entry::action(
                                    format!("{}  {}", i.iid_label, i.title),
                                    HostingAction::OpenUrl(i.web_url.clone()),
                                )
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                self.menu = Some(ContextMenu::new(
                    msg::hosting_ui::OPEN_ISSUE.get(),
                    entries,
                    anchor.0,
                    anchor.1,
                ));
                HostingEffect::None
            }
            HostingCommand::CloseIssue => self.local_pick_menu(
                msg::local_issues::PICK_CLOSE.get(),
                anchor,
                |i| i.state != IssueState::Closed,
                |i| HostingAction::LocalState {
                    number: i.number,
                    closed: true,
                },
            ),
            HostingCommand::ReopenIssue => self.local_pick_menu(
                msg::local_issues::PICK_REOPEN.get(),
                anchor,
                |i| i.state == IssueState::Closed,
                |i| HostingAction::LocalState {
                    number: i.number,
                    closed: false,
                },
            ),
            HostingCommand::OpenInObsidian => self.local_pick_menu(
                msg::local_issues::PICK_OBSIDIAN.get(),
                anchor,
                |_| true,
                |i| HostingAction::OpenObsidian(i.web_url.clone()),
            ),
            HostingCommand::CreateObsidianBase => self.local_create_base(),
            HostingCommand::EditAssignees => self.meta_command(MetaField::Assignees, anchor),
            HostingCommand::EditReviewers => self.meta_command(MetaField::Reviewers, anchor),
            HostingCommand::EditLabels => self.meta_command(MetaField::Labels, anchor),
            HostingCommand::SetMilestone => self.meta_command(MetaField::Milestone, anchor),
            HostingCommand::ReviewPullRequest => self.review_command(anchor),
            HostingCommand::OpenBoard => self.board_effect(),
            HostingCommand::OpenIssueConversation => self.item_command(anchor),
            HostingCommand::UseProfile => {
                self.menu = Some(ContextMenu::new(
                    msg::hosting_ui::MENU_REPO_INTEGRATION.get(),
                    self.profile_entries(),
                    anchor.0,
                    anchor.1,
                ));
                HostingEffect::None
            }
        }
    }
}
