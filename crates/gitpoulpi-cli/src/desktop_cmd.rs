//! `gitpoulpi desktop shortcut status|install|remove`: the scriptable twin of
//! Settings > Launcher (`gitpoulpi_tui::desktop_shortcuts`).

use gitpoulpi_commands::cli::{DesktopAction, FolderMenuAction, ShortcutAction};
use serde_json::json;

pub fn run(action: DesktopAction, json_output: bool) {
    let result = match action {
        DesktopAction::Shortcut { action } => shortcut(action),
        DesktopAction::FolderMenu { action } => folder_menu(action),
    };
    finish(result, json_output);
}

type Answer = Result<(serde_json::Value, String), String>;

fn folder_menu(action: FolderMenuAction) -> Answer {
    use gitpoulpi_tui::desktop_shortcuts as ds;
    match action {
        FolderMenuAction::Status => {
            let r = ds::folder_menu_status();
            let value = json!({
                "os": ds::os_name(),
                "entries": r.entries.iter().map(|e| json!({
                    "manager": e.manager,
                    "detected": e.detected,
                    "state": e.state,
                })).collect::<Vec<_>>(),
                "unavailable": r.unavailable,
                "error": r.error,
            });
            let mut lines: Vec<String> = r
                .entries
                .iter()
                .map(|e| {
                    format!(
                        "{}: {}{}",
                        e.name,
                        e.state,
                        if e.detected { "" } else { " (not found)" }
                    )
                })
                .collect();
            if r.unavailable {
                lines.push("the gitpoulpi launcher is missing next to GitPoulpi".into());
            }
            if let Some(e) = r.error {
                lines.push(e);
            }
            Ok((value, lines.join("\n")))
        }
        FolderMenuAction::Install => ds::folder_menu_install().map(done),
        FolderMenuAction::Remove => ds::folder_menu_remove().map(done),
    }
}

fn shortcut(action: ShortcutAction) -> Answer {
    match action {
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
    }
}

fn finish(result: Answer, json_output: bool) {
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
