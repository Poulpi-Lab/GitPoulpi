//! The rows of a commit's context menu, as plain data: which rows a commit gets, in which order, and
//! which are disabled and why. The terminal UI's `commit_menu_entries` is the model (its rows and order,
//! without the glyphs); each row is a palette command, so a click, the keyboard and the palette end in
//! the same dispatcher.

use gitpoulpi_app::keymap;
use gitpoulpi_app::palette::vocab::log_menu::CommitField;
use gitpoulpi_app::palette::vocab::reword_ui::AmendScope;
use gitpoulpi_app::palette::vocab::rewrite_view::RewriteEntry;
use gitpoulpi_app::palette::{entry_shortcut, unavailable_reason, PaletteCommand, PaletteContext};
use gitpoulpi_core::domain::ResetMode;
use gitpoulpi_i18n::msg::{self, menu_log as m};

use crate::commands::catalog::entry_of;
use crate::commands::{op_for, Command};
use crate::daemon::commit_ops::{field_label, FIELDS};
use crate::strings;

/// What a row does.
#[derive(Clone, Debug)]
pub enum Kind {
    /// Runs a command on the selected commit(s).
    Run(Command),
    /// A drill-in list of rows.
    Sub(Vec<Item>),
    /// A rule between two groups.
    Rule,
}

#[derive(Clone, Debug)]
pub struct Item {
    pub label: String,
    pub kind: Kind,
    /// Drawn in the danger colour (revert, hard reset).
    pub danger: bool,
    /// Why the row cannot run (shown beside it); `None` when it can.
    pub disabled: Option<String>,
    pub shortcut: String,
}

/// What the menu needs to know when it opens, a snapshot: a branch that moves under an open menu must
/// not relabel its rows.
#[derive(Clone, Debug)]
pub struct MenuCtx {
    /// The checked-out branch, or `HEAD` when detached.
    pub head: String,
    pub detached: bool,
    /// How many commits are selected in the Log (above one, only the multi-commit rows are offered).
    pub selected: usize,
    pub merge_commit: bool,
    /// The palette's availability facts, with a commit selected.
    pub palette: PaletteContext,
}

fn row(label: impl Into<String>, command: PaletteCommand, ctx: &MenuCtx) -> Item {
    let command = Command::Palette(command);
    let disabled = if op_for(command).is_none() {
        Some(strings::NOT_AVAILABLE_YET.to_string())
    } else {
        match command {
            Command::Palette(c) => unavailable_reason(c, ctx.palette).map(str::to_string),
            Command::Action(_) => None,
        }
    };
    let shortcut = match command {
        Command::Palette(c) => entry_of(c)
            .map(|e| entry_shortcut(&keymap::current(), e))
            .map(|s| s.replace(gitpoulpi_app::palette::BADGE_SUFFIX, ""))
            .unwrap_or_default(),
        Command::Action(_) => String::new(),
    };
    Item {
        label: label.into(),
        kind: Kind::Run(command),
        danger: false,
        disabled,
        shortcut,
    }
}

fn danger(mut item: Item) -> Item {
    item.danger = true;
    item
}

fn why(mut item: Item, reason: Option<&str>) -> Item {
    if let Some(r) = reason {
        item.disabled = Some(r.to_string());
    }
    item
}

fn sub(label: impl Into<String>, items: Vec<Item>) -> Item {
    Item {
        label: label.into(),
        kind: Kind::Sub(items),
        danger: false,
        disabled: None,
        shortcut: String::new(),
    }
}

fn rule() -> Item {
    Item {
        label: String::new(),
        kind: Kind::Rule,
        danger: false,
        disabled: None,
        shortcut: String::new(),
    }
}

fn copy_label(field: CommitField) -> String {
    let label = field_label(field);
    let mut chars = label.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

/// The rows for one commit (or for a multi-selection).
pub fn build(ctx: &MenuCtx) -> Vec<Item> {
    use PaletteCommand as C;
    let head = ctx.head.as_str();
    if ctx.selected > 1 {
        return vec![
            row(
                m::cherry_pick_many(ctx.selected, head),
                C::CherryPickSelectedCommit,
                ctx,
            ),
            sub(
                m::REWRITE_HISTORY.get(),
                vec![
                    row(
                        m::REWRITE_SELECTED.get(),
                        C::Rewrite(RewriteEntry::SelectedCommits),
                        ctx,
                    ),
                    row(
                        m::REWRITE_WORKBENCH.get(),
                        C::Rewrite(RewriteEntry::Workbench),
                        ctx,
                    ),
                ],
            ),
            row(msg::mermaid::MENU_COPY.get(), C::CopySelectedAsMermaid, ctx),
        ];
    }
    let merge = ctx.merge_commit;
    let merge_reason = m::MERGE_NO_PATCH.get();
    let split_reason = msg::split_ui::REFUSE_MERGE_COMMIT.get();
    vec![
        row(m::cherry_pick_onto(head), C::CherryPickSelectedCommit, ctx),
        danger(row(m::REVERT.get(), C::RevertSelectedCommit, ctx)),
        sub(
            m::reset_to(head),
            vec![
                row(
                    m::RESET_SOFT.get(),
                    C::ResetToSelectedCommit(ResetMode::Soft),
                    ctx,
                ),
                row(
                    m::RESET_MIXED.get(),
                    C::ResetToSelectedCommit(ResetMode::Mixed),
                    ctx,
                ),
                danger(row(
                    m::RESET_HARD.get(),
                    C::ResetToSelectedCommit(ResetMode::Hard),
                    ctx,
                )),
            ],
        ),
        row(m::EDIT_MESSAGE.get(), C::EditSelectedCommitMessage, ctx),
        row(m::AMEND.get(), C::AmendSelectedCommit(AmendScope::All), ctx),
        why(
            row(m::SPLIT_COMMIT.get(), C::SplitSelectedCommit, ctx),
            merge.then_some(split_reason),
        ),
        why(
            row(
                m::REBASE_HERE.get(),
                C::InteractiveRebaseFromSelectedCommit,
                ctx,
            ),
            merge.then_some(merge_reason),
        ),
        sub(
            m::REWRITE_HISTORY.get(),
            vec![
                row(
                    m::REWRITE_FROM_HERE.get(),
                    C::Rewrite(RewriteEntry::FromSelectedCommit),
                    ctx,
                ),
                row(
                    m::REWRITE_WORKBENCH.get(),
                    C::Rewrite(RewriteEntry::Workbench),
                    ctx,
                ),
            ],
        ),
        rule(),
        row(
            m::CREATE_BRANCH_HERE.get(),
            C::CreateBranchAtSelectedCommit,
            ctx,
        ),
        row(m::CREATE_TAG_HERE.get(), C::CreateTagAtSelectedCommit, ctx),
        row(
            m::CREATE_ANNOTATED_TAG_HERE.get(),
            C::CreateAnnotatedTagAtSelectedCommit,
            ctx,
        ),
        row(m::CHECKOUT_COMMIT.get(), C::CheckoutSelectedCommit, ctx),
        row(
            m::CREATE_WORKTREE_HERE.get(),
            C::CreateWorktreeAtSelectedCommit,
            ctx,
        ),
        why(
            row(
                m::PATCH_FROM_COMMIT.get(),
                C::CreatePatchAtSelectedCommit,
                ctx,
            ),
            merge.then_some(merge_reason),
        ),
        why(
            row(
                m::patch_to_head(head),
                C::CreatePatchFromSelectedCommitToHead,
                ctx,
            ),
            if merge {
                Some(merge_reason)
            } else if ctx.detached {
                Some(m::HEAD_DETACHED.get())
            } else {
                None
            },
        ),
        rule(),
        row(msg::undo_safety::LOST_MENU_ROW.get(), C::LostCommits, ctx),
        row(m::COPY_COMMIT_LINK.get(), C::CopySelectedCommitLink, ctx),
        sub(
            msg::menu_ref::COPY.get(),
            FIELDS
                .iter()
                .map(|f| row(copy_label(*f), C::CopyCommitField(*f), ctx))
                .chain([
                    rule(),
                    row(msg::mermaid::MENU_COPY.get(), C::CopySelectedAsMermaid, ctx),
                ])
                .collect(),
        ),
    ]
}

/// The rows that can be picked (rules left out): what the keyboard walks.
pub fn is_row(item: &Item) -> bool {
    !matches!(item.kind, Kind::Rule)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx(selected: usize) -> MenuCtx {
        MenuCtx {
            head: "main".into(),
            detached: false,
            selected,
            merge_commit: false,
            palette: PaletteContext {
                selected_commit: true,
                ..PaletteContext::default()
            },
        }
    }

    fn labels(items: &[Item]) -> Vec<String> {
        items
            .iter()
            .filter(|i| is_row(i))
            .map(|i| i.label.clone())
            .collect()
    }

    #[test]
    fn a_commit_gets_the_terminal_uis_rows_in_its_order() {
        let items = build(&ctx(1));
        let got = labels(&items);
        assert_eq!(got[0], m::cherry_pick_onto("main"));
        assert_eq!(got[1], m::REVERT.get());
        assert_eq!(got[2], m::reset_to("main"));
        let create = got
            .iter()
            .position(|l| l == m::CREATE_BRANCH_HERE.get())
            .unwrap();
        let checkout = got
            .iter()
            .position(|l| l == m::CHECKOUT_COMMIT.get())
            .unwrap();
        assert!(create < checkout, "{got:?}");
        assert_eq!(got.last().unwrap(), msg::menu_ref::COPY.get());
        assert!(items.iter().any(|i| matches!(i.kind, Kind::Rule)));
    }

    #[test]
    fn a_reset_is_a_submenu_of_soft_mixed_and_a_dangerous_hard() {
        let items = build(&ctx(1));
        let Kind::Sub(modes) = &items[2].kind else {
            panic!("reset is a submenu");
        };
        assert_eq!(modes.len(), 3);
        assert!(!modes[0].danger && !modes[1].danger && modes[2].danger);
        assert!(items[1].danger, "revert is dangerous");
    }

    #[test]
    fn several_selected_commits_offer_only_the_multi_commit_rows() {
        let items = build(&ctx(3));
        let got = labels(&items);
        assert_eq!(got[0], m::cherry_pick_many(3, "main"));
        assert!(!got.contains(&m::REVERT.get().to_string()));
        assert_eq!(got.len(), 3);
    }

    #[test]
    fn a_merge_commit_cannot_become_a_patch() {
        let mut c = ctx(1);
        c.merge_commit = true;
        let items = build(&c);
        let patch = items
            .iter()
            .find(|i| i.label == m::PATCH_FROM_COMMIT.get())
            .unwrap();
        assert_eq!(patch.disabled.as_deref(), Some(m::MERGE_NO_PATCH.get()));
    }
}
