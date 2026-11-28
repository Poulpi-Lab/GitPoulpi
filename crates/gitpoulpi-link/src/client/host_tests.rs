//! The client reaches the frontend only through [`super::host::LinkHost`].

use std::path::Path;

/// The frontend modules the client used to call directly.
const FRONTEND_MODULES: [&str; 16] = [
    "redraw",
    "exit_log",
    "window_ctl",
    "stage_guard_dialog",
    "jobs_ui",
    "push_group_ui",
    "hosting_notify",
    "activity_log",
    "keymap",
    "theme",
    "tabs",
    "graph",
    "ui_kit",
    "safety_panel",
    "history_view",
    "status_optimistic",
];

fn rust_files(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap().flatten() {
        let path = entry.path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

#[test]
fn no_client_file_calls_a_frontend_module_directly() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/client");
    let mut files = Vec::new();
    rust_files(&dir, &mut files);
    assert!(files.len() > 10, "the scan found the module's files");
    let mut offenders = Vec::new();
    for file in files {
        // This file names the modules to look for.
        if file.file_name().is_some_and(|n| n == "host_tests.rs") {
            continue;
        }
        let text = std::fs::read_to_string(&file).unwrap();
        for (n, line) in text.lines().enumerate() {
            let code = line.trim_start();
            if code.starts_with("//") {
                continue;
            }
            for module in FRONTEND_MODULES {
                let paths = [
                    format!("crate::{module}::"),
                    format!("crate::{module};"),
                    format!("super::super::{module}"),
                    format!("crate::{{{module}"),
                ];
                if paths.iter().any(|p| code.contains(p.as_str())) {
                    offenders.push(format!("{}:{}: {code}", file.display(), n + 1));
                }
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "route these through LinkHost:\n{}",
        offenders.join("\n")
    );
}
