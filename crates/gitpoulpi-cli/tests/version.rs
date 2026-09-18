//! `-V` prints one line; `--version` adds the build's features
//! and languages with `--version` (custom builds, see `custom-builds.md`).

use assert_cmd::Command;

#[test]
fn long_version_lists_features_and_languages() {
    let out = Command::cargo_bin("gitpoulpi")
        .unwrap()
        .arg("--version")
        .output()
        .unwrap();
    let text = String::from_utf8(out.stdout).unwrap();
    let mut lines = text.lines();
    assert!(lines.next().unwrap().contains("gitpoulpi"), "{text}");
    let features = lines.next().unwrap();
    let languages = lines.next().unwrap();
    assert!(languages.starts_with("languages: en"), "{text}");
    assert!(
        lines.next().unwrap().starts_with("themes: "),
        "the theme set is part of --version: {text}"
    );
    if cfg!(feature = "full") {
        // Derived from this crate's own `full` feature, so a feature added
        // there but forgotten in the `--version` list fails here.
        let manifest = include_str!("../Cargo.toml");
        let full = manifest
            .lines()
            .find(|l| l.starts_with("full = ["))
            .expect("`full` feature in Cargo.toml");
        let listed: Vec<&str> = features
            .strip_prefix("features: ")
            .unwrap()
            .split(' ')
            .collect();
        for name in full.split('"').skip(1).step_by(2) {
            if name == "all-langs" {
                continue;
            }
            assert!(listed.contains(&name), "`{name}` missing from: {text}");
        }
        assert_eq!(
            features,
            "features: ai github gitlab azure-devops bitbucket jira trello gitea linear youtrack redmine gerrit local-issues mcp lfs worktree submodule avatar-fetch image-view radar mermaid board review history-rewrite global-search workspaces rewrite-scripts",
            "{text}"
        );
        assert!(!text.contains("custom build"), "{text}");
    }
}

#[test]
fn short_version_stays_one_line() {
    let out = Command::cargo_bin("gitpoulpi")
        .unwrap()
        .arg("-V")
        .output()
        .unwrap();
    assert_eq!(String::from_utf8(out.stdout).unwrap().lines().count(), 1);
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
