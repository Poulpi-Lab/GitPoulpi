//! Where a tag lives, as the TUI shows it: the marker glyph, its colour, the
//! words of its tooltip, and the snapshot every surface reads.
//!
//! git keeps no remote-tracking refs for tags, so GitPoulpi learns each
//! remote's tags from the last fetch or tag push (the *tag mirror*, see
//! `REMOTE_TAG_MIRROR_PREFIX`) and `refs()` reports, per tag, whether it is
//! **local** only, **remote** only (a remote has it, this repository does
//! not), on **both**, or **differs** (both, at different commits). This
//! module is the single place that turns that into pixels, so the sidebar,
//! the Log pills, the menus and the tooltip cannot tell different stories.
//!
//! The markers reuse arrows the sidebar already draws for tags (up: only
//! here, push it) and mirror them: all are text-presentation glyphs present
//! in every monospace font measured for this project (the standing glyph
//! rule; an emoji arrow would flip presentation on the selected row).
//!
//! | where   | marker | colour      |
//! |---------|--------|-------------|
//! | local   | `U+2B06` up arrow        | warning |
//! | remote  | `U+2B07` down arrow      | remote  |
//! | both    | `U+2194` left right      | muted   |
//! | differs | `U+21C5` up down arrows  | warning |
//!
//! The snapshot is process-wide (like `flow::view`'s) and installed once per
//! frame from the refs the daemon already handed over; nothing here touches
//! the network.

use std::collections::HashMap;

use ratatui::style::Color;

use crate::th;
use gitpoulpi_core::domain::{Ref, TagWhere};
use gitpoulpi_i18n::msg::tag_where as msg;

/// What the snapshot knows about one tag.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Info {
    pub(crate) place: TagWhere,
    /// Remotes that have the tag at the same commit.
    pub(crate) remotes: Vec<String>,
    /// Remotes that have it at another commit.
    pub(crate) differs_on: Vec<String>,
}

impl Info {
    /// Every remote that has a tag of this name, same commit or not.
    pub(crate) fn all_remotes(&self) -> Vec<String> {
        let mut all: Vec<String> = self
            .remotes
            .iter()
            .chain(self.differs_on.iter())
            .cloned()
            .collect();
        all.sort();
        all.dedup();
        all
    }
}

type Snapshot = HashMap<String, Info>;

#[cfg(not(test))]
static SNAPSHOT: std::sync::Mutex<Option<Snapshot>> = std::sync::Mutex::new(None);
#[cfg(test)]
thread_local! {
    static SNAPSHOT: std::cell::RefCell<Option<Snapshot>> = const { std::cell::RefCell::new(None) };
}

fn with_snapshot<R>(f: impl FnOnce(&mut Option<Snapshot>) -> R) -> R {
    #[cfg(not(test))]
    {
        match SNAPSHOT.lock() {
            Ok(mut s) => f(&mut s),
            Err(_) => f(&mut None),
        }
    }
    #[cfg(test)]
    {
        SNAPSHOT.with(|s| f(&mut s.borrow_mut()))
    }
}

/// The snapshot of `refs`' tags.
pub(crate) fn snapshot_of(refs: &[Ref]) -> HashMap<String, Info> {
    refs.iter()
        .filter_map(|r| {
            let name = r.tag_name()?.to_string();
            let sync = r.tag_sync.as_ref()?;
            Some((
                name,
                Info {
                    place: sync.where_(),
                    remotes: sync.pushed_to.clone(),
                    differs_on: sync.differs_on.clone(),
                },
            ))
        })
        .collect()
}

/// Installs the snapshot every surface reads. Called once per frame.
pub(crate) fn install(refs: &[Ref]) {
    let fresh = snapshot_of(refs);
    with_snapshot(|s| *s = Some(fresh));
}

/// What is known about tag `name`.
pub(crate) fn lookup(name: &str) -> Option<Info> {
    with_snapshot(|s| s.as_ref().and_then(|m| m.get(name).cloned()))
}

/// The glyph of a place (no leading space).
pub(crate) fn glyph(place: TagWhere) -> &'static str {
    match place {
        TagWhere::Local => "\u{2B06}",
        TagWhere::Remote => "\u{2B07}",
        TagWhere::Both => "\u{2194}",
        TagWhere::Differs => "\u{21C5}",
    }
}

pub(crate) fn color(place: TagWhere) -> Color {
    match place {
        TagWhere::Local | TagWhere::Differs => th().status.warning,
        TagWhere::Remote => th().refs.remote,
        TagWhere::Both => th().ui.muted,
    }
}

/// The tooltip of tag `name`: its name, where it lives in words, and what a
/// fetch does with it. `None` for a tag the snapshot does not know.
pub(crate) fn tooltip_lines(name: &str) -> Option<Vec<String>> {
    let info = lookup(name)?;
    let remotes = info.all_remotes().join(", ");
    let mut lines = vec![name.to_string()];
    lines.push(match info.place {
        TagWhere::Local => msg::TIP_LOCAL.get().to_string(),
        TagWhere::Remote => msg::tip_remote(&remotes),
        TagWhere::Both => msg::tip_both(&remotes),
        TagWhere::Differs => msg::tip_differs(&remotes),
    });
    if info.place == TagWhere::Differs {
        lines.push(msg::TIP_DIFFERS_ACTIONS.get().to_string());
    }
    // What a fetch does with it matters exactly when a remote has it.
    if info.place != TagWhere::Local {
        lines.push(msg::TIP_FETCH_NOTE.get().to_string());
    }
    Some(lines)
}

// ---------------------------------------------------------------------
// The two ways out of a tag that differs
// ---------------------------------------------------------------------

/// The remote a "tag that differs" action runs against: the `index`th of the
/// remotes it differs on (a menu row), else the default remote when it is one
/// of them, else the first (the palette, which has no row to name one).
fn differing_remote(name: &str, index: Option<usize>, default: Option<String>) -> Option<String> {
    let on = lookup(name)?.differs_on;
    match index {
        Some(i) => on.get(i).cloned(),
        None => match default {
            Some(d) if on.contains(&d) => Some(d),
            _ => on.into_iter().next(),
        },
    }
}

/// "Use `name` from `remote`": submits the action (local, no network) and
/// returns the message to show.
pub(crate) fn begin_take_remote(
    name: &str,
    index: Option<usize>,
    default: Option<String>,
    link: &crate::daemon_link::DaemonLink,
) -> String {
    match differing_remote(name, index, default) {
        Some(remote) => {
            link.submit_action(crate::daemon_link::Action::TagTakeRemote {
                remote,
                name: name.to_string(),
            });
            msg::msg_taking_remote(name)
        }
        None => msg::msg_not_differing(name),
    }
}

/// "Push `name` to `remote` (force)": asks first (the remote's tag is
/// replaced for everyone), the red button runs it.
pub(crate) fn begin_force_push(
    name: &str,
    index: Option<usize>,
    default: Option<String>,
    input_mode: &mut crate::modes::InputMode,
    last_message: &mut Option<String>,
) {
    match differing_remote(name, index, default) {
        Some(remote) => {
            *input_mode = crate::modes::InputMode::Confirm {
                prompt: msg::confirm_force_push(name, &remote),
                action: crate::modes::ConfirmAction::Destructive(
                    crate::modes::PendingDestructive::ForcePushTag {
                        remote,
                        name: name.to_string(),
                    },
                ),
            };
        }
        None => *last_message = Some(msg::msg_not_differing(name)),
    }
}

// ---------------------------------------------------------------------
// The hovered sidebar tag (its tooltip)
// ---------------------------------------------------------------------

type Hover = Option<(String, u16, u16)>;

#[cfg(not(test))]
static HOVER: std::sync::Mutex<Hover> = std::sync::Mutex::new(None);
#[cfg(test)]
thread_local! {
    static HOVER: std::cell::RefCell<Hover> = const { std::cell::RefCell::new(None) };
}

/// The pointer is over sidebar tag `name` at `(col, row)`, or over none.
pub(crate) fn set_hover(hover: Hover) {
    #[cfg(not(test))]
    if let Ok(mut slot) = HOVER.lock() {
        *slot = hover;
    }
    #[cfg(test)]
    HOVER.with(|h| *h.borrow_mut() = hover);
}

fn hover() -> Hover {
    #[cfg(not(test))]
    {
        HOVER.lock().ok().and_then(|h| h.clone())
    }
    #[cfg(test)]
    {
        HOVER.with(|h| h.borrow().clone())
    }
}

/// The tooltip key of the hovered tag (a change of key is a change of hover
/// target), `None` when it has no tooltip. The place is part of the key, so
/// the tooltip follows a refresh that changes it.
pub(crate) fn hover_key() -> Option<u64> {
    use std::hash::{Hash, Hasher};
    let (name, _, _) = hover()?;
    let info = lookup(&name)?;
    let mut h = std::collections::hash_map::DefaultHasher::new();
    name.hash(&mut h);
    glyph(info.place).hash(&mut h);
    Some(h.finish())
}

/// `(key, lines, col, row)` of the hovered tag's tooltip.
pub(crate) fn hover_tooltip() -> Option<(u64, Vec<String>, u16, u16)> {
    let key = hover_key()?;
    let (name, col, row) = hover()?;
    Some((key, tooltip_lines(&name)?, col, row))
}

#[cfg(test)]
pub(crate) mod tests;
