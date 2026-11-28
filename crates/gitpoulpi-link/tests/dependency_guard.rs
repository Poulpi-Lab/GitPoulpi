//! `gitpoulpi-link` is shared by every frontend, so no frontend may leak into
//! it. A UI toolkit, a terminal library or an async runtime in its
//! dependency list fails here before it spreads.

const FORBIDDEN: &[&str] = &[
    "ratatui",
    "crossterm",
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
    "gitpoulpi-commands",
    "gitpoulpi-shim",
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
            "gitpoulpi-link must stay frontend-neutral, but depends on `{name}`"
        );
    }
}

/// The code itself: no terminal or toolkit crate named outside comments.
#[test]
fn no_source_file_names_a_terminal_or_toolkit_crate() {
    fn files(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
        for entry in std::fs::read_dir(dir).unwrap().flatten() {
            let path = entry.path();
            if path.is_dir() {
                files(&path, out);
            } else if path.extension().is_some_and(|e| e == "rs") {
                out.push(path);
            }
        }
    }
    let mut all = Vec::new();
    files(
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
        &mut all,
    );
    assert!(all.len() > 20, "the scan found the sources");
    let mut problems = Vec::new();
    for file in all {
        let text = std::fs::read_to_string(&file).unwrap();
        for (n, line) in text.lines().enumerate() {
            let code = line.trim_start();
            if code.starts_with("//") {
                continue;
            }
            for banned in [
                "ratatui",
                "crossterm",
                "gitpoulpi_tui",
                "gitpoulpi_cli::",
                "gitpoulpi_mcp",
            ] {
                if code.contains(banned) {
                    problems.push(format!("{}:{}: {code}", file.display(), n + 1));
                }
            }
        }
    }
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}
