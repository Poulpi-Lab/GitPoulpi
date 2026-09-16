use serde::{Deserialize, Serialize};

/// How far an external AI agent (reaching GitPoulpi through the MCP server
/// or `gitpoulpi ui ...`) may move the user's window by itself.
///
/// It is chosen on the agent's side (`gitpoulpi-mcp --ui-control`, the
/// `[mcp] ui_control` setting) and travels with every view intent, so the
/// TUI that finally receives the intent can honour it:
///
/// - `ask` (the default): the window shows a chip the user confirms.
/// - `auto`: the window applies the intent unless the user is busy.
/// - `off`: the intent is refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiControl {
    Auto,
    #[default]
    Ask,
    Off,
}

impl UiControl {
    pub const ALL: [UiControl; 3] = [UiControl::Auto, UiControl::Ask, UiControl::Off];

    pub fn as_str(self) -> &'static str {
        match self {
            UiControl::Auto => "auto",
            UiControl::Ask => "ask",
            UiControl::Off => "off",
        }
    }
}

impl std::str::FromStr for UiControl {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "auto" => Ok(Self::Auto),
            "ask" => Ok(Self::Ask),
            "off" => Ok(Self::Off),
            other => Err(format!("`{other}` is not one of auto, ask, off")),
        }
    }
}
