//! `gitpoulpi-app` is shared by every frontend, so no frontend may leak into
//! it. A UI toolkit, a terminal library or an async runtime in its
//! dependency list fails here before it spreads.

const FORBIDDEN: &[&str] = &[
    "ratatui",
    "crossterm",
    "tokio",
    "gpui",
    "slint",
    "iced",
    "egui",
    "eframe",
    "winit",
    "gtk",
    "gtk4",
    "qt",
    "tauri",
    "wry",
    "gitpoulpi-tui",
    "gitpoulpi-cli",
    "gitpoulpi-mcp",
    "gitpoulpi-daemon",
    "gitpoulpi-ipc",
    "gitpoulpi-commands",
];

fn dependency_names(manifest: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut in_deps = false;
    for line in manifest.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_deps = line.contains("dependencies");
            continue;
        }
        if in_deps {
            if let Some((name, _)) = line.split_once('=') {
                let name = name.trim().trim_end_matches(".workspace").trim();
                if !name.is_empty() && !name.starts_with('#') {
                    names.push(name.to_string());
                }
            }
        }
    }
    names
}

#[test]
fn no_ui_toolkit_or_frontend_crate_is_a_dependency() {
    let manifest = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"))
        .expect("read Cargo.toml");
    let names = dependency_names(&manifest);
    assert!(
        names.iter().any(|n| n == "gitpoulpi-core"),
        "the parser must see the dependencies: {names:?}"
    );
    for name in &names {
        assert!(
            !FORBIDDEN.contains(&name.as_str()),
            "gitpoulpi-app must stay frontend-neutral, but depends on `{name}`"
        );
    }
}
