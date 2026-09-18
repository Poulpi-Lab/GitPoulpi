use super::*;

mod detect;
mod encode;

/// An environment as a lookup, so every case below is a pure function
/// call rather than a mutation of this process' own environment --
/// which is shared by every test in the binary and has deadlocked
/// this project once already (`where-we-are.md`, "Never repoint
/// `GITPOULPI_CONFIG_DIR` from a test").
/// The pre-item-41 shape: `ui.image_protocol = "auto"`, no herdr
/// config file anywhere.
fn protocol_for<F: Fn(&str) -> Option<String>>(choice: RendererChoice, get: &F) -> Protocol {
    super::protocol_for(choice, ImageProtocol::Auto, get, &|_: &std::path::Path| {
        None
    })
}

fn env(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
    let map: std::collections::HashMap<String, String> = pairs
        .iter()
        .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
        .collect();
    move |name: &str| map.get(name).cloned()
}

/// A real WezTerm pane on Windows, as this machine reports it.
const WEZTERM: [(&str, &str); 3] = [
    (
        "WEZTERM_EXECUTABLE",
        r"C:\Program Files\WezTerm\wezterm-gui.exe",
    ),
    ("TERM_PROGRAM", "WezTerm"),
    ("TERM", "xterm-256color"),
];

/// The same pane, with a herdr 0.9.1 pane inside it. **Copied from a
/// live `env` dump on 2026-09-23**, which is the point: the WezTerm
/// variables are all still there and all still true about the
/// *outermost* terminal, and every one of them is a lie about who
/// owns these cells.
fn wezterm_inside_herdr() -> Vec<(&'static str, &'static str)> {
    let mut pairs = WEZTERM.to_vec();
    pairs.extend([
        ("HERDR_ENV", "1"),
        ("HERDR_PANE_ID", "w4:p1"),
        ("HERDR_TAB_ID", "w4:t1"),
        ("HERDR_WORKSPACE_ID", "w4"),
        (
            "HERDR_SOCKET_PATH",
            r"C:\Users\ada\AppData\Roaming\herdr\herdr.sock",
        ),
    ]);
    pairs
}
