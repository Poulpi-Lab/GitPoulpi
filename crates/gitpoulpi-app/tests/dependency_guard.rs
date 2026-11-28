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

/// The only workspace crates the app may build on: the domain, the config,
/// the catalogs, the log vocabulary and the session files. Anything else
/// (a daemon client, a runtime, a UI) belongs to a frontend or to its own
/// crate.
const ALLOWED_WORKSPACE_DEPENDENCIES: &[&str] = &[
    "gitpoulpi-core",
    "gitpoulpi-config",
    "gitpoulpi-i18n",
    "gitpoulpi-log",
    "gitpoulpi-session",
];

#[test]
fn workspace_dependencies_are_the_allowed_ones() {
    let manifest = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"))
        .expect("read Cargo.toml");
    for name in dependency_names(&manifest) {
        if name.starts_with("gitpoulpi-") {
            assert!(
                ALLOWED_WORKSPACE_DEPENDENCIES.contains(&name.as_str()),
                "gitpoulpi-app may not depend on `{name}`"
            );
        }
    }
}

/// No source file names a terminal library: a type of one reaching a shared
/// signature would pull the terminal into every frontend.
#[test]
fn sources_name_no_terminal_library() {
    fn walk(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
        for entry in std::fs::read_dir(dir).expect("read_dir") {
            let path = entry.expect("entry").path();
            if path.is_dir() {
                walk(&path, out);
            } else if path.extension().is_some_and(|e| e == "rs") {
                out.push(path);
            }
        }
    }
    let mut files = Vec::new();
    walk(
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
        &mut files,
    );
    assert!(files.len() > 10, "the scan must see the sources");
    let mut bad = Vec::new();
    for file in files {
        let text = std::fs::read_to_string(&file).expect("read source");
        for (i, line) in text.lines().enumerate() {
            if line.trim_start().starts_with("//") {
                continue;
            }
            for needle in ["ratatui", "crossterm"] {
                if line.contains(needle) {
                    bad.push(format!("{}:{}: {needle}", file.display(), i + 1));
                }
            }
        }
    }
    assert!(
        bad.is_empty(),
        "terminal libraries named in the app: {bad:#?}"
    );
}
