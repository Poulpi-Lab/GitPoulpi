//! The remotes badge of a branch with extra push targets: `origin ↑2 ·
//! gitlab ↑2↓1 · backup new` where it fits, `⇉3 ↑2↓1` in a narrow row, `⇉3`
//! as the last resort. One place turns the daemon's `Ref.multi_remote` into
//! words, colour and the tooltip, so the Refs row, the Log pill and the
//! tooltip cannot tell different stories.
//!
//! The glyphs (U+21C9, U+2191, U+2193) are text-default, so no variation
//! selector is needed. Colour: green when every remote is in sync, yellow
//! when some is only ahead, behind or still without the branch, red when one
//! has diverged (or shares no history).

use gitpoulpi_core::domain::{RemoteRelation, RemoteTargetState};
use gitpoulpi_i18n::msg::multi_remote as m;
use ratatui::style::Color;

use crate::th;
use crate::ui_kit::display_width;

/// The "several remotes" glyph (U+21C9, rightwards paired arrows).
pub(crate) const GLYPH: &str = "\u{21C9}";

/// How the remotes of a branch stand as a whole.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Health {
    InSync,
    /// Some remote is only ahead or behind, has no copy yet or is gone.
    Pending,
    Diverged,
}

pub(crate) fn health(targets: &[RemoteTargetState]) -> Health {
    let mut out = Health::InSync;
    for t in targets {
        let h = if t.missing {
            Health::Pending
        } else {
            match t.relation {
                RemoteRelation::UpToDate => Health::InSync,
                RemoteRelation::LocalAhead(_)
                | RemoteRelation::RemoteAhead(_)
                | RemoteRelation::Absent => Health::Pending,
                RemoteRelation::Diverged { .. } | RemoteRelation::Unrelated => Health::Diverged,
            }
        };
        match (out, h) {
            (_, Health::Diverged) => out = Health::Diverged,
            (Health::InSync, Health::Pending) => out = Health::Pending,
            _ => {}
        }
    }
    out
}

pub(crate) fn color(health: Health) -> Color {
    match health {
        Health::InSync => th().status.success,
        Health::Pending => th().status.warning,
        Health::Diverged => th().status.danger,
    }
}

/// `↑2`, `↓1`, `↑2↓1`, `=`, or the word for the other states.
fn relation_text(t: &TargetView<'_>) -> String {
    if t.missing {
        return m::STATE_GONE.get().to_string();
    }
    match t.relation {
        RemoteRelation::UpToDate => "=".to_string(),
        RemoteRelation::LocalAhead(n) => format!("\u{2191}{n}"),
        RemoteRelation::RemoteAhead(n) => format!("\u{2193}{n}"),
        RemoteRelation::Diverged { ahead, behind } => format!("\u{2191}{ahead}\u{2193}{behind}"),
        RemoteRelation::Absent => m::STATE_NEW.get().to_string(),
        RemoteRelation::Unrelated => m::STATE_UNRELATED.get().to_string(),
    }
}

struct TargetView<'a> {
    remote: &'a str,
    relation: RemoteRelation,
    missing: bool,
}

fn views(targets: &[RemoteTargetState]) -> Vec<TargetView<'_>> {
    targets
        .iter()
        .map(|t| TargetView {
            remote: &t.remote,
            relation: t.relation,
            missing: t.missing,
        })
        .collect()
}

/// `origin ↑2 · gitlab ↑2↓1 · backup new`.
pub(crate) fn wide_form(targets: &[RemoteTargetState]) -> String {
    views(targets)
        .iter()
        .map(|t| format!("{} {}", t.remote, relation_text(t)))
        .collect::<Vec<_>>()
        .join(" \u{b7} ")
}

/// `⇉3 ↑2↓1`: the count of remotes and the largest ahead and behind among
/// them (`=` when all are in sync, nothing when only counts are unknown).
pub(crate) fn narrow_form(targets: &[RemoteTargetState]) -> String {
    let (mut ahead, mut behind) = (0u32, 0u32);
    for t in targets.iter().filter(|t| !t.missing) {
        match t.relation {
            RemoteRelation::LocalAhead(n) => ahead = ahead.max(n),
            RemoteRelation::RemoteAhead(n) => behind = behind.max(n),
            RemoteRelation::Diverged { ahead: a, behind: b } => {
                ahead = ahead.max(a);
                behind = behind.max(b);
            }
            _ => {}
        }
    }
    let mut out = compact_form(targets);
    if health(targets) == Health::InSync {
        out.push_str(" =");
    } else if ahead > 0 || behind > 0 {
        out.push(' ');
        if ahead > 0 {
            out.push_str(&format!("\u{2191}{ahead}"));
        }
        if behind > 0 {
            out.push_str(&format!("\u{2193}{behind}"));
        }
    }
    out
}

/// `⇉3`.
pub(crate) fn compact_form(targets: &[RemoteTargetState]) -> String {
    format!("{GLYPH}{}", targets.len())
}

/// The most informative form that fits `budget` display columns, `None`
/// when not even the compact one does.
pub(crate) fn fit(targets: &[RemoteTargetState], budget: usize) -> Option<String> {
    if targets.is_empty() {
        return None;
    }
    [
        wide_form(targets),
        narrow_form(targets),
        compact_form(targets),
    ]
    .into_iter()
    .find(|f| display_width(f) <= budget)
}

/// The tooltip: a title, then one line per remote in words.
pub(crate) fn tooltip_lines(branch: &str, targets: &[RemoteTargetState]) -> Vec<String> {
    let mut lines = vec![m::tip_title(branch)];
    for t in targets {
        lines.push(if t.missing {
            m::tip_gone(&t.remote)
        } else {
            match t.relation {
                RemoteRelation::UpToDate => m::tip_in_sync(&t.remote),
                RemoteRelation::LocalAhead(n) => m::tip_ahead(&t.remote, n),
                RemoteRelation::RemoteAhead(n) => m::tip_behind(&t.remote, n),
                RemoteRelation::Diverged { ahead, behind } => {
                    m::tip_diverged(&t.remote, ahead, behind)
                }
                RemoteRelation::Absent => m::tip_new(&t.remote),
                RemoteRelation::Unrelated => m::tip_unrelated(&t.remote),
            }
        });
    }
    lines.push(m::TIP_OPEN_HINT.get().to_string());
    lines
}

/// The tooltip key of the badge of `branch` (a change of key is a change of
/// hover target).
pub(crate) fn hover_key(branch: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    branch.hash(&mut h);
    h.finish()
}

/// Opens the remotes dialog of local branch `branch`: the same route as the
/// Refs row menu's "Set upstream...", so whatever that opens, the badge (and
/// its `u` key) opens too. Returns the status-line message when it cannot.
pub(crate) fn open_branch_remotes(
    remote_ui: &crate::remotes_ui::RemoteUi,
    refs: &[gitpoulpi_core::domain::Ref],
    branch: &str,
    input_mode: &mut crate::modes::InputMode,
) -> Option<String> {
    match crate::remotes_ui::local_branch(refs, branch) {
        Some(head) => {
            remote_ui.open_upstream_prompt(&head, refs, input_mode);
            None
        }
        None => Some(gitpoulpi_i18n::msg::remotes::SELECT_LOCAL_BRANCH.get().into()),
    }
}

#[cfg(test)]
pub(crate) mod tests;
