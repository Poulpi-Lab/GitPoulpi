//! The Synchronization page's update rows: the check mode and channel (a row
//! the machine policy sets shows "Set by your administrator" in place of its
//! value and cannot be edited, the same `coming` rule as an unfinished
//! setting), and the read-only "Installed with" row right under them.
//!
//! The "Installed with" text comes from the frontend's background read of the
//! install ([`Facts::updates`]), never from a file read here: until it is in,
//! the row says so.

use super::{Control, PoolCtx, Row, Target};
use gitpoulpi_i18n::msg::updates as t;

pub fn updates_mode_field(cx: &PoolCtx) -> Row {
    let mut field = Row::global(
        t::MODE.get(),
        "updates.mode",
        Control::Choice {
            options: gitpoulpi_config::UpdateMode::ALL
                .iter()
                .map(|m| m.as_str().into())
                .collect(),
            index: gitpoulpi_config::UpdateMode::ALL
                .iter()
                .position(|m| *m == cx.config.updates.mode)
                .unwrap_or(0),
        },
    )
    .with_help(t::MODE_HELP.get());
    if cx.facts.updates().mode_locked {
        field.coming = Some(t::LOCKED.get().to_string());
    }
    field
}

pub fn updates_channel_field(cx: &PoolCtx) -> Row {
    let mut field = Row::global(
        t::CHANNEL.get(),
        "updates.channel",
        Control::Choice {
            options: gitpoulpi_config::UpdateChannel::ALL
                .iter()
                .map(|m| m.as_str().into())
                .collect(),
            index: gitpoulpi_config::UpdateChannel::ALL
                .iter()
                .position(|m| *m == cx.config.updates.channel)
                .unwrap_or(0),
        },
    )
    .with_help(t::CHANNEL_HELP.get());
    if cx.facts.updates().channel_locked {
        field.coming = Some(t::LOCKED.get().to_string());
    }
    field
}

/// Follows the channel row (`picked` keeps an info row with no target next to
/// the row before it).
pub fn updates_installed_with_field(cx: &PoolCtx) -> Row {
    let text = cx
        .facts
        .updates()
        .installed_with
        .unwrap_or_else(|| "...".to_string());
    Row::plain(t::INSTALLED_WITH.get(), Control::Info(text), Target::None)
        .with_help(t::INSTALLED_WITH_HELP.get())
}
