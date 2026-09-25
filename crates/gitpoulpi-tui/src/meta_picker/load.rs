//! The picker's two worker calls: load the candidates (and the item's
//! current values), and send the edit. Plain threads over a one-shot
//! channel, drained by the render loop with `try_recv`; nothing here is
//! ever awaited on the input thread.

use std::sync::{mpsc, Arc};

use gitpoulpi_hosting::{HostingClient, Integration, IssueEdit, ItemMeta, MetaKind, PrEdit};
use ratatui::style::Color;

use super::{Applied, Item, MetaField};
use crate::hosting_task::{error_text, request_task_failed, ClientFactory};
use crate::redraw::NotifySend as _;

/// The item (or the project, for a create form) a picker works on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MetaTarget {
    pub(crate) integration: Integration,
    /// Every configured profile (the token source reads them all).
    pub(crate) all: Vec<Integration>,
    pub(crate) project: String,
    pub(crate) kind: MetaKind,
    /// `None` for a create form: nothing exists yet to read from.
    pub(crate) number: Option<u64>,
    /// What the popup says it edits: `#12 Fix the thing`, or the form.
    pub(crate) caption: String,
}

/// What the load thread reports.
pub(super) struct Loaded {
    pub(super) items: Result<Vec<Item>, String>,
    pub(super) current: Option<ItemMeta>,
}

fn runtime() -> Result<tokio::runtime::Runtime, String> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(request_task_failed)
}

/// Starts asking the host for `field`'s candidates.
pub(super) fn start_load(
    factory: Arc<dyn ClientFactory>,
    target: &MetaTarget,
    field: MetaField,
) -> mpsc::Receiver<Loaded> {
    let (tx, rx) = mpsc::channel();
    let target = target.clone();
    std::thread::spawn(move || {
        let loaded = match runtime() {
            Ok(rt) => rt.block_on(load(&*factory, &target, field)),
            Err(why) => Loaded {
                items: Err(why),
                current: None,
            },
        };
        let _ = tx.send_notify(loaded);
    });
    rx
}

async fn load(factory: &dyn ClientFactory, target: &MetaTarget, field: MetaField) -> Loaded {
    let client = match factory.client(&target.integration, &target.all) {
        Ok(client) => client,
        Err(e) => {
            return Loaded {
                items: Err(error_text(&e)),
                current: None,
            }
        }
    };
    let items = candidates(&*client, &target.project, field).await;
    // Candidates and current values are independent calls; the current
    // ones are only asked for an item that exists.
    let current = match (target.number, &items) {
        (Some(number), Ok(_)) => client
            .item_meta(&target.project, number, target.kind)
            .await
            .ok(),
        _ => None,
    };
    Loaded { items, current }
}

async fn candidates(
    client: &dyn HostingClient,
    project: &str,
    field: MetaField,
) -> Result<Vec<Item>, String> {
    match field {
        MetaField::Assignees | MetaField::Reviewers => client
            .project_members(project, "")
            .await
            .map_err(|e| error_text(&e))
            .map(|people| {
                people
                    .into_iter()
                    .map(|p| {
                        let shown = if p.name.is_empty() { &p.login } else { &p.name };
                        let badge = crate::avatar::badge(shown, &p.login);
                        Item {
                            value: p.login.clone(),
                            label: p.login,
                            detail: p.name,
                            color: None,
                            badge: Some((crate::avatar::badge_slot_text(&badge), badge.color)),
                        }
                    })
                    .collect()
            }),
        MetaField::Labels => client
            .project_labels(project)
            .await
            .map_err(|e| error_text(&e))
            .map(|labels| {
                labels
                    .into_iter()
                    .map(|l| Item {
                        value: l.name.clone(),
                        label: l.name,
                        detail: l.description,
                        color: crate::theme::parse_hex_color(&format!("#{}", l.color))
                            .map(|(r, g, b)| Color::Rgb(r, g, b)),
                        badge: None,
                    })
                    .collect()
            }),
        MetaField::Milestone => client
            .project_milestones(project)
            .await
            .map_err(|e| error_text(&e))
            .map(|list| {
                list.into_iter()
                    .map(|m| Item {
                        value: m.id.to_string(),
                        label: m.title,
                        detail: m
                            .due_on
                            .map(gitpoulpi_i18n::msg::meta_picker::due)
                            .unwrap_or_default(),
                        color: None,
                        badge: None,
                    })
                    .collect()
            }),
    }
}

/// What a running edit reports.
pub(crate) struct EditReceiver {
    rx: mpsc::Receiver<Result<(), String>>,
}

impl EditReceiver {
    pub(crate) fn try_recv(&self) -> Option<Result<(), String>> {
        self.rx.try_recv().ok()
    }
}

/// Sends `applied` to the host as an edit of `target`'s item.
pub(crate) fn start_edit(
    factory: Arc<dyn ClientFactory>,
    target: MetaTarget,
    applied: Applied,
) -> EditReceiver {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let result = (|| {
            let number = target.number.ok_or("nothing to edit")?; // i18n-english
            let client = factory
                .client(&target.integration, &target.all)
                .map_err(|e| error_text(&e))?;
            let rt = runtime()?;
            rt.block_on(async {
                match target.kind {
                    MetaKind::Issue => {
                        client
                            .update_issue(&target.project, number, &issue_edit(&applied))
                            .await
                    }
                    MetaKind::PullRequest => {
                        client
                            .update_pull_request(&target.project, number, &pr_edit(&applied))
                            .await
                    }
                }
                .map_err(|e| error_text(&e))
            })
        })();
        let _ = tx.send_notify(result);
    });
    EditReceiver { rx }
}

fn issue_edit(applied: &Applied) -> IssueEdit {
    let mut edit = IssueEdit::default();
    match applied {
        Applied::List(MetaField::Assignees, list) => edit.assignees = Some(list.clone()),
        Applied::List(MetaField::Labels, list) => edit.labels = Some(list.clone()),
        Applied::List(..) => {}
        Applied::Milestone(m) => edit.milestone = Some(m.as_ref().map(|m| m.id)),
    }
    edit
}

fn pr_edit(applied: &Applied) -> PrEdit {
    let mut edit = PrEdit::default();
    match applied {
        Applied::List(MetaField::Assignees, list) => edit.assignees = Some(list.clone()),
        Applied::List(MetaField::Reviewers, list) => edit.reviewers = Some(list.clone()),
        Applied::List(MetaField::Labels, list) => edit.labels = Some(list.clone()),
        Applied::List(..) => {}
        Applied::Milestone(m) => edit.milestone = Some(m.as_ref().map(|m| m.id)),
    }
    edit
}
