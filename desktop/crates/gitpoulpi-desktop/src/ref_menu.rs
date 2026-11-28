//! The ref context menu: the rows a ref gets (the terminal UI's own list and order, one menu for the Refs
//! sidebar and the Log's ref pills) and the view that shows them.
//!
//! Rows the desktop cannot run yet (worktrees, reflog, the push group, pull requests, fetching a tag...)
//! are present and dimmed with the reason, so the list stays the terminal UI's. Keyboard: Up / Down move,
//! Enter or Right runs (or opens the Copy submenu), Left goes back, Escape closes; a click outside closes.

use gitpoulpi_app::keymap::Action;
use gitpoulpi_app::palette::vocab::ref_menu::RefField;
use gitpoulpi_core::domain::TagWhere;
use gitpoulpi_i18n::msg::menu_ref as m;
use gpui_kit::component::v_flex;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    div, px, App, Context, EventEmitter, FocusHandle, Focusable, InteractiveElement as _,
    IntoElement, KeyDownEvent, ParentElement as _, Pixels, Point, Render, SharedString,
    StatefulInteractiveElement as _, Styled as _, Window,
};

use crate::commands::refs_ops::RefOp;
use crate::commands::repo_ops::RepoOp;
use crate::daemon::refs_ops::{self as rules, Snapshot};
use crate::ref_target::RefTarget;
use crate::strings;
use crate::theme::Skin;

mod view;
pub use view::{RefMenu, RefMenuEvent};

/// One row of a ref menu.
#[derive(Clone, Debug, PartialEq)]
pub enum Item {
    Action {
        label: String,
        /// `None`: the row is only there to keep the terminal UI's list (dimmed with `disabled`).
        op: Option<RefOp>,
        /// Why the row cannot run now.
        disabled: Option<String>,
        danger: bool,
        key: Option<String>,
    },
    Submenu {
        label: String,
        items: Vec<Item>,
    },
    Separator,
}

impl Item {
    pub(crate) fn act(label: String, op: RefOp) -> Item {
        Item::Action {
            label,
            op: Some(op),
            disabled: None,
            danger: false,
            key: None,
        }
    }

    pub(crate) fn danger(label: String, op: RefOp) -> Item {
        Item::Action {
            label,
            op: Some(op),
            disabled: None,
            danger: true,
            key: None,
        }
    }

    pub(crate) fn off(label: String, why: String) -> Item {
        Item::Action {
            label,
            op: None,
            disabled: Some(why),
            danger: false,
            key: None,
        }
    }

    /// A row of the terminal UI's list the desktop does not run yet.
    pub(crate) fn later(label: String) -> Item {
        Item::off(label, strings::NOT_AVAILABLE_YET.to_string())
    }

    pub(crate) fn keyed(self, action: Action) -> Item {
        let shown = gitpoulpi_app::keymap::current().shown(action);
        match self {
            Item::Action {
                label,
                op,
                disabled,
                danger,
                ..
            } => Item::Action {
                label,
                op,
                disabled,
                danger,
                key: (!shown.is_empty()).then_some(shown),
            },
            other => other,
        }
    }

    fn selectable(&self) -> bool {
        !matches!(self, Item::Separator)
    }

    pub fn label(&self) -> &str {
        match self {
            Item::Action { label, .. } | Item::Submenu { label, .. } => label,
            Item::Separator => "",
        }
    }
}

/// What the rows depend on besides the ref.
#[derive(Clone, Copy, Debug, Default)]
pub struct MenuFacts {
    pub hidden: bool,
    pub solo: bool,
}

pub(crate) fn glyph(g: &str, text: String) -> String {
    format!("{g}  {text}")
}

fn copy_menu() -> Item {
    Item::Submenu {
        label: glyph("\u{2398}", m::COPY.get().to_string()),
        items: RefField::all()
            .into_iter()
            .map(|f| Item::act(f.row_label().to_string(), RefOp::Copy(f)))
            .collect(),
    }
}

fn fast_forward_row(
    label: String,
    snap: &Snapshot,
    moving: &RefTarget,
    to: &RefTarget,
    op: RefOp,
) -> Item {
    let (a, b) = (snap.oid_of(moving), snap.oid_of(to));
    match (a, b) {
        (Some(a), Some(b)) => match rules::ff_refusal(&moving.name(), a, b) {
            Some(why) => Item::off(label, why),
            None => Item::act(label, op),
        },
        _ => Item::act(label, op),
    }
}

/// The rows of the menu of `target`, in the terminal UI's order.
pub fn items_for(target: &RefTarget, snap: &Snapshot, facts: MenuFacts) -> Vec<Item> {
    let reveal = || {
        Item::act(
            glyph("\u{2192}", m::REVEAL_IN_GRAPH.get().to_string()),
            RefOp::Reveal,
        )
        .keyed(Action::RefsReveal)
    };
    if let RefTarget::Stash(_) = target {
        return vec![
            reveal(),
            Item::act(
                glyph(
                    "\u{25A4}",
                    gitpoulpi_i18n::msg::autostash::SHOW_STASH.get().to_string(),
                ),
                RefOp::StashShow,
            ),
            Item::Separator,
            Item::act(
                glyph("\u{21A9}", m::STASH_POP.get().to_string()),
                RefOp::StashPop,
            )
            .keyed(Action::GitStashPop),
            Item::act(
                glyph("\u{2398}", m::STASH_APPLY.get().to_string()),
                RefOp::StashApply,
            ),
            Item::Separator,
            Item::danger(
                glyph("\u{2717}", m::STASH_DROP.get().to_string()),
                RefOp::StashDrop,
            )
            .keyed(Action::GitDeleteRef),
        ];
    }
    let name = target.name();
    let head = snap.head().map(|h| h.0.to_string());
    let head_label = head
        .clone()
        .unwrap_or_else(|| gitpoulpi_i18n::msg::sidebar::DETACHED.get().to_string());
    let is_current = matches!(target, RefTarget::Local(n) if head.as_deref() == Some(n));
    let hide = if facts.hidden {
        m::SHOW_IN_GRAPH.get()
    } else {
        m::HIDE_IN_GRAPH.get()
    };
    let solo = if facts.solo {
        m::CLEAR_SOLO.get()
    } else {
        m::SOLO.get()
    };
    let view = || {
        vec![
            Item::act(glyph("\u{25C9}", hide.to_string()), RefOp::ToggleHide),
            Item::act(glyph("\u{25C9}", solo.to_string()), RefOp::ToggleSolo)
                .keyed(Action::RefsSolo),
        ]
    };
    let worktree = || {
        Item::act(
            glyph("\u{229E}", m::create_worktree(&name)),
            RefOp::Repo(RepoOp::WorktreeFromRef),
        )
    };
    let mut rows = vec![reveal(), Item::Separator];
    let twins = rules::delete_targets(snap, target);
    match target {
        RefTarget::Local(_) => {
            if !is_current {
                rows.push(Item::act(
                    glyph("\u{2713}", m::checkout(&name)),
                    RefOp::Checkout,
                ));
                rows.push(Item::Separator);
                rows.push(Item::act(
                    glyph("\u{21C6}", m::merge_into(&name, &head_label)),
                    RefOp::Merge,
                ));
                rows.push(
                    Item::act(
                        glyph("\u{27F3}", m::rebase_onto(&head_label, &name)),
                        RefOp::Rebase,
                    )
                    .keyed(Action::GitRebaseOnto),
                );
                if head.is_some() {
                    let current = head.clone().map(RefTarget::Local).unwrap_or(target.clone());
                    rows.push(fast_forward_row(
                        glyph("\u{21A0}", m::fast_forward(&name, &head_label)),
                        snap,
                        target,
                        &current,
                        RefOp::FastForwardToCurrent,
                    ));
                    rows.push(fast_forward_row(
                        glyph("\u{21A0}", m::fast_forward(&head_label, &name)),
                        snap,
                        &current,
                        target,
                        RefOp::FastForwardCurrentToThis,
                    ));
                }
            }
            rows.push(worktree());
            rows.push(Item::later(glyph(
                "\u{21BA}",
                gitpoulpi_i18n::msg::undo_safety::REFLOG_MENU_ROW
                    .get()
                    .to_string(),
            )));
            rows.push(Item::Separator);
            if is_current {
                rows.push(
                    Item::act(
                        glyph("\u{21E9}", m::PULL.get().to_string()),
                        RefOp::Repo(RepoOp::BranchPull),
                    )
                    .keyed(Action::GitPull),
                );
            }
            rows.push(
                Item::act(
                    glyph("\u{21E7}", m::push_branch(&name)),
                    RefOp::Repo(RepoOp::BranchPush),
                )
                .keyed(Action::GitPush),
            );
            rows.push(Item::later(glyph(
                "\u{21E7}",
                gitpoulpi_i18n::msg::remotes::PUSH_TO_SELECTED
                    .get()
                    .to_string(),
            )));
            rows.push(Item::later(glyph("\u{21F1}", m::start_pr(&name))));
            rows.push(Item::act(
                glyph("\u{27F2}", m::FETCH.get().to_string()),
                RefOp::Repo(RepoOp::BranchFetch),
            ));
            rows.push(Item::act(
                glyph("\u{21C5}", m::remotes_of(&name)),
                RefOp::SetUpstream,
            ));
            rows.push(Item::Separator);
            rows.extend(view());
            rows.push(Item::Separator);
            rows.push(Item::act(
                glyph("\u{270E}", m::rename(&name)),
                RefOp::Rename,
            ));
            rows.push(
                Item::danger(glyph("\u{2717}", m::delete(&name)), RefOp::DeleteSelected)
                    .keyed(Action::GitDeleteRef),
            );
            if let Some((remote, branch)) = &twins.remote {
                let twin = format!("{remote}/{branch}");
                rows.push(Item::danger(
                    glyph("\u{2717}", m::delete(&twin)),
                    RefOp::DeleteRemoteBranch,
                ));
                rows.push(Item::danger(
                    glyph("\u{2717}", m::delete_both(&name, &twin)),
                    RefOp::DeleteBranchAndRemote,
                ));
            }
            rows.push(Item::Separator);
            if twins.remote.is_some() {
                rows.push(Item::later(glyph(
                    "\u{2398}",
                    m::COPY_BRANCH_LINK.get().to_string(),
                )));
            }
        }
        RefTarget::Remote(_) => {
            rows.push(Item::act(
                glyph("\u{2713}", m::checkout(&name)),
                RefOp::Checkout,
            ));
            if let Some(current) = head.clone().map(RefTarget::Local) {
                rows.push(fast_forward_row(
                    glyph("\u{21A0}", m::fast_forward(&head_label, &name)),
                    snap,
                    &current,
                    target,
                    RefOp::FastForwardCurrentToThis,
                ));
            }
            rows.push(worktree());
            rows.push(Item::Separator);
            rows.extend(view());
            rows.push(Item::Separator);
            rows.push(Item::danger(
                glyph("\u{2717}", m::delete(&name)),
                RefOp::DeleteRemoteBranch,
            ));
            if let Some(local) = &twins.local {
                rows.push(Item::danger(
                    glyph("\u{2717}", m::delete_both(local, &name)),
                    RefOp::DeleteBranchAndRemote,
                ));
            }
            rows.push(Item::Separator);
            rows.push(Item::later(glyph(
                "\u{2398}",
                m::COPY_BRANCH_LINK.get().to_string(),
            )));
        }
        RefTarget::Tag(_) => {
            use gitpoulpi_i18n::msg::tag_where as t;
            let place = snap.tag_place(&name);
            let remote_only = place.place == Some(TagWhere::Remote);
            if !remote_only {
                rows.push(Item::act(
                    glyph("\u{2713}", m::checkout(&name)),
                    RefOp::Checkout,
                ));
                rows.push(worktree());
            }
            rows.push(Item::Separator);
            rows.extend(view());
            rows.push(Item::Separator);
            match place.place {
                Some(TagWhere::Local) => rows.push(Item::act(
                    glyph("\u{2B06}", m::push_tag(&name)),
                    RefOp::PushTag,
                )),
                Some(TagWhere::Remote) => {
                    let remote = place.remotes.first().cloned().unwrap_or_default();
                    rows.push(Item::later(glyph("\u{2B07}", t::fetch_tag(&name, &remote))));
                }
                Some(TagWhere::Differs) => {
                    for remote in &place.differs_on {
                        rows.push(Item::later(glyph(
                            "\u{2B07}",
                            t::take_remote(&name, remote),
                        )));
                        rows.push(Item::later(glyph(
                            "\u{2B06}",
                            t::force_push_tag(&name, remote),
                        )));
                    }
                }
                _ => {}
            }
            rows.push(
                Item::act(
                    glyph("\u{2B06}", m::PUSH_ALL_TAGS.get().to_string()),
                    RefOp::PushAllTags,
                )
                .keyed(Action::GitPushTags),
            );
            if !remote_only {
                rows.push(Item::act(
                    glyph("\u{270E}", m::annotate(&name)),
                    RefOp::AnnotateTag,
                ));
            }
            rows.push(match (place.place, place.remotes.is_empty()) {
                (Some(TagWhere::Remote), _) => Item::danger(
                    glyph(
                        "\u{2717}",
                        t::delete_tag_on_remote(&name, place.remotes.join(", ")),
                    ),
                    RefOp::DeleteTagFromRemote,
                ),
                (Some(TagWhere::Both | TagWhere::Differs), _) => Item::danger(
                    glyph("\u{2717}", t::delete_tag_scope(&name)),
                    RefOp::DeleteSelected,
                ),
                _ => Item::danger(glyph("\u{2717}", m::delete(&name)), RefOp::DeleteSelected),
            });
            rows.push(Item::Separator);
        }
        _ => {}
    }
    rows.push(copy_menu());
    rows
}

/// The rows the drop of `source` on `target` opens: Merge, Rebase and Fast-forward.
pub fn drop_items(
    snap: &Snapshot,
    source: &RefTarget,
    target: &RefTarget,
) -> Vec<(Item, rules::DropOp)> {
    rules::drop_rows(snap, source, target)
        .into_iter()
        .map(|(op, label, why)| {
            let glyph_ = match op {
                rules::DropOp::Merge => "\u{21C6}",
                rules::DropOp::Rebase => "\u{27F3}",
                rules::DropOp::FastForward => "\u{21A0}",
            };
            let label = glyph(glyph_, label);
            let item = match why {
                Some(why) => Item::off(label, why),
                None => Item::act(label, RefOp::DropOnCurrent),
            };
            (item, op)
        })
        .collect()
}

#[cfg(test)]
mod tests;
