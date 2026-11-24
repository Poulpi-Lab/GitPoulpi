//! A branch with several remotes in the Push and Pull menus, the toolbar
//! buttons, the branch context menu and the palette: what `Ref.multi_remote`
//! says in the words the menus need, the menu rows, and the one place each
//! of those actions runs (so the toolbar, the menus and the palette cannot
//! disagree).
//!
//! The grouped engine and its dialog live in `push_group_ui`; this module
//! only decides what to ask of it.

use super::*;

use gitpoulpi_core::domain::{RemoteTargetState, TargetRole};

/// The remotes of a branch that has `gitpoulpi-push-to` or
/// `gitpoulpi-pull-from` entries.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct BranchGroup {
    /// `(remote, branch there)` a push goes to: the upstream first, then the
    /// extra targets. Remotes that are no longer configured are left out.
    pub(crate) push: Vec<(String, String)>,
    /// There is at least one extra push target (the Push button then pushes
    /// to all).
    pub(crate) extra_push: bool,
    /// The upstream's remote, when there is one.
    pub(crate) primary: Option<String>,
    /// Remotes a plain Pull also integrates (the upstream excluded).
    pub(crate) pull_sources: Vec<String>,
    /// `(remote, branch there)` of every remote of the group, for the
    /// one-off "Pull from <remote> only" rows.
    pub(crate) pull_one: Vec<(String, String)>,
}

impl BranchGroup {
    pub(crate) fn from_states(states: &[RemoteTargetState]) -> Self {
        let mut group = BranchGroup::default();
        for s in states.iter().filter(|s| !s.missing) {
            match s.role {
                TargetRole::Primary => {
                    group.primary = Some(s.remote.clone());
                    group.push.push((s.remote.clone(), s.branch.clone()));
                }
                TargetRole::PushTo => {
                    group.extra_push = true;
                    group.push.push((s.remote.clone(), s.branch.clone()));
                    if s.pull {
                        group.pull_sources.push(s.remote.clone());
                    }
                }
                TargetRole::PullFrom => group.pull_sources.push(s.remote.clone()),
            }
            group.pull_one.push((s.remote.clone(), s.branch.clone()));
        }
        group
    }

    /// The remotes of "Push to all (...)".
    pub(crate) fn push_names(&self) -> String {
        self.push
            .iter()
            .map(|(r, _)| r.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// The remotes of "Pull from all sources (...)": the upstream and the
    /// sources.
    pub(crate) fn pull_names(&self) -> String {
        self.primary
            .iter()
            .chain(self.pull_sources.iter())
            .map(String::as_str)
            .collect::<Vec<_>>()
            .join(" + ")
    }

    /// The branch name on `remote` for a one-off pull.
    pub(crate) fn branch_on(&self, remote: &str, local: &str) -> String {
        self.pull_one
            .iter()
            .find(|(r, _)| r == remote)
            .map_or_else(|| local.to_string(), |(_, b)| b.clone())
    }
}

/// The group of the local branch `branch`, `None` for a branch with no
/// `gitpoulpi-*` entry (it keeps today's single-remote behaviour).
pub(crate) fn branch_group(refs: &[Ref], branch: &str) -> Option<BranchGroup> {
    refs.iter().find_map(|r| match (&r.kind, &r.multi_remote) {
        (RefKind::LocalBranch { name }, Some(states)) if name == branch => {
            Some(BranchGroup::from_states(states))
        }
        _ => None,
    })
}

/// `git.push_group_preflight`: ask every remote before a grouped push (on
/// when the config cannot be read).
pub(crate) fn push_group_preflight() -> bool {
    gitpoulpi_config::Settings::resolve_ambient()
        .map(|s| s.config().git.push_group_preflight)
        .unwrap_or(true)
}

/// The Push dropdown for `branch`.
///
/// With extra targets: "Push to all (...)", "Push to <r> only" per remote;
/// without, every remote of the repository, the default first (today's
/// rows). Whatever the branch: "Push to selected remotes..." and the
/// branch's Remotes dialog.
pub(crate) fn push_menu_entries(
    branch: &str,
    group: Option<&BranchGroup>,
    remotes: &[String],
    default: Option<&str>,
) -> Vec<Entry<RemoteUiAction>> {
    let mut entries = match group.filter(|g| g.extra_push) {
        Some(g) => {
            let mut rows = vec![
                Entry::action(
                    glyph_label("\u{21E7}", msg::push_to_all(g.push_names())),
                    RemoteUiAction::PushAll {
                        branch: branch.to_string(),
                    },
                )
                .with_key(crate::keymap::Action::GitPush),
                Entry::Separator,
            ];
            rows.extend(g.push.iter().map(|(remote, dest)| {
                Entry::action(
                    glyph_label("\u{21E7}", msg::push_to_only(remote)),
                    RemoteUiAction::PushOne {
                        branch: branch.to_string(),
                        remote: remote.clone(),
                        remote_branch: dest.clone(),
                    },
                )
            }));
            rows
        }
        None => push_to_entries(remotes, default),
    };
    entries.push(Entry::Separator);
    entries.push(Entry::action(
        glyph_label("\u{21E7}", msg::PUSH_TO_SELECTED.get()),
        RemoteUiAction::PushSelected {
            branch: branch.to_string(),
        },
    ));
    entries.push(Entry::action(
        glyph_label("\u{21C5}", msg::remotes_of(branch)),
        RemoteUiAction::RemotesOf {
            branch: branch.to_string(),
        },
    ));
    entries
}

/// The Pull dropdown for the checked-out `branch`: the group's pull first
/// when there are sources, Fetch All and the modes, then one-off pulls from
/// each remote of the group.
pub(crate) fn pull_menu_entries(
    branch: Option<&str>,
    group: Option<&BranchGroup>,
) -> Vec<Entry<RemoteUiAction>> {
    let mut entries = Vec::new();
    if let (Some(branch), Some(g)) = (branch, group.filter(|g| !g.pull_sources.is_empty())) {
        entries.push(Entry::action(
            glyph_label("\u{21E9}", msg::pull_from_all(g.pull_names())),
            RemoteUiAction::PullAll {
                branch: branch.to_string(),
            },
        ));
        entries.push(Entry::Separator);
    }
    entries.extend(pull_mode_entries());
    if let (Some(_), Some(g)) = (branch, group) {
        if !g.pull_one.is_empty() {
            entries.push(Entry::Separator);
            entries.extend(g.pull_one.iter().map(|(remote, dest)| {
                Entry::action(
                    glyph_label("\u{21E9}", msg::pull_from_only(remote)),
                    RemoteUiAction::PullOne {
                        remote: remote.clone(),
                        remote_branch: dest.clone(),
                    },
                )
            }));
        }
    }
    entries
}

impl RemoteUi {
    /// Push `branch` to every remote of its group: the plan, run at once
    /// when every row is a plain push. Returns the status line.
    pub(crate) fn push_all(&self, branch: &str, refs: &[Ref]) -> String {
        let remotes = branch_group(refs, branch).map_or_else(
            || self.default_remote(refs).unwrap_or_default(),
            |g| g.push_names(),
        );
        crate::push_group_ui::PushGroupUi::open_push_all(&self.repo_root, branch);
        msg::pushing_to_all(branch, remotes)
    }

    /// Push `branch` to one remote, once, under its name there.
    pub(crate) fn push_one(
        &self,
        branch: &str,
        remote: &str,
        remote_branch: &str,
        refs: &[Ref],
        link: &DaemonLink,
        ticket: Option<crate::daemon_link::Ticket>,
    ) -> String {
        let has_upstream = local_branch(refs, branch).is_some_and(|b| b.upstream.is_some());
        link.submit_as(
            ticket,
            Action::Push {
            remote: remote.to_string(),
            opts: PushOptions {
                set_upstream: !has_upstream,
                branch: Some(branch.to_string()),
                remote_branch: (remote_branch != branch).then(|| remote_branch.to_string()),
                ..PushOptions::default()
            },
            },
        );
        msg::pushing_to(branch, remote)
    }

    /// The one-off "Push to selected remotes..." dialog.
    pub(crate) fn push_selected(&self, branch: &str) {
        crate::push_group_ui::PushGroupUi::open_selection(&self.repo_root, branch);
    }

    /// Pull the checked-out `branch` from its upstream and every pull
    /// source, in the pull mode of `git.pull_mode`.
    pub(crate) fn pull_all(&self, branch: &str, refs: &[Ref]) -> String {
        let remotes = branch_group(refs, branch).map_or_else(String::new, |g| g.pull_names());
        crate::push_group_ui::PushGroupUi::open_pull_all(&self.repo_root, branch);
        msg::pulling_from_all(branch, remotes)
    }

    /// Pull one remote's branch into the checked-out branch, once, in the
    /// pull mode of `git.pull_mode`.
    pub(crate) fn pull_one(&self, remote: &str, remote_branch: &str, link: &DaemonLink) -> String {
        let (mode, _) = sync_settings();
        let mode = match mode {
            PullButtonMode::Merge => PullMode::Merge,
            PullButtonMode::FfOnly => PullMode::FastForwardOnly,
            PullButtonMode::Rebase => PullMode::Rebase,
        };
        link.submit_action(Action::Pull(PullOptions {
            mode,
            remote: Some(remote.to_string()),
            remote_branch: Some(remote_branch.to_string()),
        }));
        msg::pulling_from(format!("{remote}/{remote_branch}"), remote_branch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitpoulpi_core::domain::RemoteRelation;

    fn state(remote: &str, role: TargetRole, pull: bool, missing: bool) -> RemoteTargetState {
        RemoteTargetState {
            remote: remote.into(),
            branch: if remote == "backup" {
                "feat-x".into()
            } else {
                "feature".into()
            },
            role,
            pull,
            tracking: None,
            relation: RemoteRelation::UpToDate,
            missing,
        }
    }

    fn group() -> BranchGroup {
        BranchGroup::from_states(&[
            state("origin", TargetRole::Primary, false, false),
            state("gitlab", TargetRole::PushTo, true, false),
            state("backup", TargetRole::PushTo, false, false),
            state("old", TargetRole::PushTo, false, true),
            state("mirror", TargetRole::PullFrom, false, false),
        ])
    }

    fn labels(entries: &[Entry<RemoteUiAction>]) -> Vec<String> {
        entries
            .iter()
            .filter_map(|e| match e {
                Entry::Action { label, .. } => Some(label.clone()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn the_group_lists_pushes_and_sources_without_gone_remotes() {
        let g = group();
        assert_eq!(g.push_names(), "origin, gitlab, backup");
        assert_eq!(g.pull_names(), "origin + gitlab + mirror");
        assert!(g.extra_push);
        assert_eq!(g.branch_on("backup", "feature"), "feat-x");
        assert_eq!(g.branch_on("elsewhere", "feature"), "feature");
    }

    #[test]
    fn the_push_menu_always_offers_selected_remotes_and_the_remotes_dialog() {
        let none = push_menu_entries("main", None, &["origin".into()], Some("origin"));
        let l = labels(&none);
        assert!(l.iter().any(|x| x.contains("selected remotes")), "{l:?}");
        assert!(l.iter().any(|x| x.contains("Remotes of main")), "{l:?}");
        // Today's rows stay for a branch without targets.
        assert!(l.iter().any(|x| x.contains("origin")), "{l:?}");
        assert!(!l.iter().any(|x| x.contains("to all")), "{l:?}");

        let g = group();
        let many = push_menu_entries("feature", Some(&g), &[], None);
        let l = labels(&many);
        assert!(l[0].contains("to all (origin, gitlab, backup)"), "{l:?}");
        for r in ["origin", "gitlab", "backup"] {
            assert!(
                l.iter().any(|x| x.contains(&format!("to {r} only"))),
                "{l:?}"
            );
        }
        assert!(l.iter().any(|x| x.contains("selected remotes")), "{l:?}");
    }

    #[test]
    fn the_pull_menu_offers_the_group_only_with_sources() {
        let g = group();
        let l = labels(&pull_menu_entries(Some("feature"), Some(&g)));
        assert!(
            l[0].contains("all sources (origin + gitlab + mirror)"),
            "{l:?}"
        );
        assert!(l.iter().any(|x| x.contains("from mirror only")), "{l:?}");
        let plain = labels(&pull_menu_entries(Some("main"), None));
        assert!(
            !plain.iter().any(|x| x.contains("all sources")),
            "{plain:?}"
        );
        assert!(!plain.iter().any(|x| x.contains("from ")), "{plain:?}");
        // Targets but no sources: no group pull, still one-off rows.
        let targets_only = BranchGroup::from_states(&[
            state("origin", TargetRole::Primary, false, false),
            state("gitlab", TargetRole::PushTo, false, false),
        ]);
        let l = labels(&pull_menu_entries(Some("feature"), Some(&targets_only)));
        assert!(!l.iter().any(|x| x.contains("all sources")), "{l:?}");
        assert!(l.iter().any(|x| x.contains("from gitlab only")), "{l:?}");
    }
}
