//! Where a registry entry (an action, a palette command) can be used.
//!
//! Almost everything is `Any`: both frontends offer it. The few things that
//! exist only because of what a terminal is are `TerminalOnly`, with the
//! reason in English. Whoever adds an action or a palette command decides it
//! at that moment; the desktop's parity table reads the answer from here.

/// Which frontends offer an entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Surface {
    /// Every frontend.
    Any,
    /// Only the terminal UI; the text says why (English, shown in the parity
    /// table).
    TerminalOnly(&'static str),
}

impl Surface {
    /// The reason an entry is terminal-only, `None` for `Any`.
    pub const fn terminal_only_reason(self) -> Option<&'static str> {
        match self {
            Surface::Any => None,
            Surface::TerminalOnly(reason) => Some(reason),
        }
    }
}

/// The generated `Settings: ...` rows (their keys) that are terminal-only by
/// nature, with the reason. The frontend that builds the Settings rows marks
/// them through [`settings_surface`]; the parity table lists them from here.
pub const TERMINAL_ONLY_SETTINGS: &[(&str, &str)] = &[
    (
        "ui.color_depth",
        "The colour depth of the terminal; a desktop window always has full colour.",
    ),
    (
        "ui.kitty_keyboard",
        "The kitty keyboard protocol is a way for a terminal to report keys.",
    ),
    (
        "ui.image_protocol",
        "Which terminal graphics protocol draws inline images; a desktop window draws them itself.",
    ),
    (
        "ui.image_view_text_fallback",
        "The text stand-in for images, for terminals that cannot draw them.",
    ),
    ("ui.text_cursor", "The shape of the terminal's text cursor."),
    (
        "ui.mouse_enabled",
        "Whether the terminal reports the mouse; a desktop window always has it.",
    ),
    (
        "ui.launch_terminal",
        "Which terminal the launcher starts the terminal UI in.",
    ),
    (
        "ui.relaunch_in_terminal",
        "Restarting the terminal UI inside another terminal window.",
    ),
];

/// The surface of the generated Settings row for `key`.
pub fn settings_surface(key: &str) -> Surface {
    TERMINAL_ONLY_SETTINGS
        .iter()
        .find(|(k, _)| *k == key)
        .map_or(Surface::Any, |(_, reason)| Surface::TerminalOnly(reason))
}
