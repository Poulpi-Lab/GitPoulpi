//! `gitpoulpi desktop shortcut status|install|remove`: the scriptable twin of
//! Settings > Launcher (`gitpoulpi_tui::desktop_shortcuts`).

use gitpoulpi_commands::cli::{DesktopAction, ShortcutAction};
use serde_json::json;

pub fn run(action: DesktopAction, json_output: bool) {
    let DesktopAction::Shortcut { action } = action;
    let result = match action {
        ShortcutAction::Status => {
            let report = gitpoulpi_tui::desktop_shortcuts::status();
            let value = json!({
                "os": gitpoulpi_tui::desktop_shortcuts::os_name(),
                "terminal": report.terminal,
                "terminals": report.terminals,
                "shortcuts": report.shortcuts.iter().map(|s| json!({
                    "place": s.place,
                    "state": s.state,
                    "icon_stale": s.icon_stale,
                })).collect::<Vec<_>>(),
            });
            let human = report
                .shortcuts
                .iter()
                .map(|s| format!("{}: {}", s.place, s.state))
                .chain(std::iter::once(format!(
                    "terminal: {}",
                    report.terminal.unwrap_or("none found")
                )))
                .collect::<Vec<_>>()
                .join("\n");
            Ok((value, human))
        }
        ShortcutAction::Install { place } => {
            gitpoulpi_tui::desktop_shortcuts::install(place.as_deref()).map(done)
        }
        ShortcutAction::Remove { place } => {
            gitpoulpi_tui::desktop_shortcuts::remove(place.as_deref()).map(done)
        }
    };
    match result {
        Ok((value, human)) => {
            if json_output {
                println!("{value}");
            } else {
                println!("{human}");
            }
        }
        Err(message) => {
            if json_output {
                println!("{}", json!({ "error": message }));
            } else {
                eprintln!("gitpoulpi: {message}");
            }
            std::process::exit(1);
        }
    }
}

fn done(lines: Vec<String>) -> (serde_json::Value, String) {
    (json!({ "done": lines }), lines.join("\n"))
}
