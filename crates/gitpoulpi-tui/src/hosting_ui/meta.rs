//! Assignees, reviewers, labels and milestone of a listed request or issue:
//! the row-menu entries, the palette commands' item lists and the target a
//! picker edits. GitHub and GitLab only (`meta_picker::supports`).

use super::*;
use crate::meta_picker::{self, MetaField, MetaTarget};
use gitpoulpi_hosting::MetaKind;

/// How many items a palette list offers (the sidebar is the place for more).
const LIST_LIMIT: usize = 40;

/// The four fields, in the order menus list them.
const FIELDS: [MetaField; 4] = [
    MetaField::Assignees,
    MetaField::Reviewers,
    MetaField::Labels,
    MetaField::Milestone,
];

fn label_of(field: MetaField) -> &'static str {
    match field {
        MetaField::Assignees => msg::hosting_ui::EDIT_ASSIGNEES.get(),
        MetaField::Reviewers => msg::hosting_ui::EDIT_REVIEWERS.get(),
        MetaField::Labels => msg::hosting_ui::EDIT_LABELS.get(),
        MetaField::Milestone => msg::hosting_ui::SET_MILESTONE.get(),
    }
}

/// The row menu's metadata entries for a request (`pr`) or an issue.
pub(super) fn menu_entries(
    kind: Option<gitpoulpi_config::IntegrationKind>,
    pr: bool,
    number: u64,
) -> Vec<Entry<HostingAction>> {
    FIELDS
        .into_iter()
        .filter(|f| pr || *f != MetaField::Reviewers)
        .filter(|f| kind.map_or(true, |k| meta_picker::supports_field(k, *f)))
        .map(|field| {
            Entry::action(
                label_of(field),
                HostingAction::EditMeta { field, pr, number },
            )
        })
        .collect()
}

impl HostingUi {
    /// The pull / merge request list is on a provider that takes metadata.
    pub(super) fn meta_prs(&self) -> bool {
        self.linked_snapshot()
            .and_then(|s| s.kind)
            .is_some_and(meta_picker::supports)
    }

    /// The issue list is on a provider that takes metadata.
    pub(super) fn meta_issues(&self) -> bool {
        self.snapshot
            .as_ref()
            .and_then(|s| s.issues_target.as_ref())
            .is_some_and(|t| meta_picker::supports(t.profile.kind))
    }

    /// What a picker for request / issue `number` edits, from the lists on
    /// screen.
    pub(super) fn meta_target(&self, pr: bool, number: u64) -> Option<MetaTarget> {
        let snap = self.snapshot.as_ref()?;
        let (profile, project, caption, kind) = if pr {
            let found = snap.prs.iter().find(|p| p.number == number)?;
            (
                snap.profile_id.clone()?,
                snap.project.clone(),
                format!("{} {}", found.iid_label, found.title),
                MetaKind::PullRequest,
            )
        } else {
            let target = snap.issues_target.as_ref()?;
            let found = snap.issues.iter().find(|i| i.number == number)?;
            (
                target.profile.id.clone(),
                target.project.clone(),
                format!("{} {}", found.iid_label, found.title),
                MetaKind::Issue,
            )
        };
        let all = (self.integrations)();
        let integration = all.iter().find(|i| i.id == profile)?.clone();
        meta_picker::supports(integration.kind).then_some(())?;
        Some(MetaTarget {
            integration,
            all,
            project,
            kind,
            number: Some(number),
            caption,
        })
    }

    /// The palette command for `field`: a list of the requests and issues
    /// to choose from (the same menu shape as "Open Issue...").
    pub(super) fn meta_command(&mut self, field: MetaField, anchor: (u16, u16)) -> HostingEffect {
        let mut entries: Vec<Entry<HostingAction>> = Vec::new();
        if self.meta_prs() {
            if let Some(snap) = self.linked_snapshot() {
                entries.extend(snap.prs.iter().take(LIST_LIMIT).map(|pr| {
                    Entry::action(
                        format!("{}  {}", pr.iid_label, pr.title),
                        HostingAction::EditMeta {
                            field,
                            pr: true,
                            number: pr.number,
                        },
                    )
                }));
            }
        }
        let issue_kind = self
            .snapshot
            .as_ref()
            .and_then(|s| s.issues_target.as_ref())
            .map(|t| t.profile.kind);
        if field != MetaField::Reviewers
            && self.meta_issues()
            && issue_kind.map_or(true, |k| meta_picker::supports_field(k, field))
        {
            if let Some(snap) = self.snapshot.as_ref() {
                if !entries.is_empty() && !snap.issues.is_empty() {
                    entries.push(Entry::Separator);
                }
                entries.extend(snap.issues.iter().take(LIST_LIMIT).map(|issue| {
                    Entry::action(
                        format!("{}  {}", issue.iid_label, issue.title),
                        HostingAction::EditMeta {
                            field,
                            pr: false,
                            number: issue.number,
                        },
                    )
                }));
            }
        }
        self.menu = Some(ContextMenu::new(
            msg::hosting_ui::MENU_PICK_ITEM.get(),
            entries,
            anchor.0,
            anchor.1,
        ));
        HostingEffect::None
    }

    /// The effect of "edit `field` of request / issue `number`" (the
    /// Radar's, once the window's lists are in).
    pub(crate) fn meta_edit_effect(
        &self,
        field: MetaField,
        pr: bool,
        number: u64,
    ) -> HostingEffect {
        self.meta_action(field, pr, number)
    }

    /// The effect of picking an item in a menu or palette list.
    pub(super) fn meta_action(&self, field: MetaField, pr: bool, number: u64) -> HostingEffect {
        match self.meta_target(pr, number) {
            Some(target) => HostingEffect::EditMeta(Box::new(target), field),
            None => HostingEffect::Message(msg::hosting_ui::WHY_NO_META.get().to_string()),
        }
    }
}
