//! Supply-chain pins of the CI configuration: every `uses:` in
//! `.github/workflows/*.yml` is a local action, a full commit SHA with a
//! version comment (Dependabot keeps the comment current), or a docker image
//! by digest; and Dependabot waits at least two days before proposing a new
//! release of either ecosystem.

use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

fn is_sha40(text: &str) -> bool {
    text.len() == 40 && text.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

/// Why a `uses:` line is not acceptable, if it is not. `line` is the text after `uses:`.
fn violation(line: &str) -> Option<String> {
    let (spec, comment) = match line.split_once('#') {
        Some((s, c)) => (s.trim(), Some(c.trim())),
        None => (line.trim(), None),
    };
    let spec = spec.trim_matches(['"', '\'']);
    if spec.starts_with("./") {
        return None;
    }
    if let Some(image) = spec.strip_prefix("docker://") {
        return match image.split_once("@sha256:") {
            Some((_, digest)) if digest.len() == 64 => None,
            _ => Some("a docker:// action must be pinned by @sha256:<digest>".to_string()),
        };
    }
    let Some((_, reference)) = spec.split_once('@') else {
        return Some("no @ref".to_string());
    };
    if !is_sha40(reference) {
        return Some(format!("`{reference}` is not a full 40-hex commit SHA"));
    }
    match comment {
        Some(c) if !c.is_empty() => None,
        _ => Some("a pinned SHA needs a trailing `# vX.Y.Z` comment".to_string()),
    }
}

fn workflow_files() -> Vec<PathBuf> {
    let dir = root().join(".github/workflows");
    let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| matches!(p.extension().and_then(|e| e.to_str()), Some("yml" | "yaml")))
        .collect();
    files.sort();
    files
}

#[test]
fn every_uses_is_local_or_pinned_by_digest() {
    let mut bad = Vec::new();
    let mut seen = 0;
    for file in workflow_files() {
        let text = std::fs::read_to_string(&file).unwrap();
        for (n, raw) in text.lines().enumerate() {
            let code = raw.trim_start().trim_start_matches("- ");
            let Some(rest) = code.strip_prefix("uses:") else {
                continue;
            };
            seen += 1;
            if let Some(why) = violation(rest) {
                bad.push(format!(
                    "{}:{}: {why}: {}",
                    file.display(),
                    n + 1,
                    raw.trim()
                ));
            }
        }
    }
    assert!(seen > 0, "no `uses:` found: the scan is broken");
    assert!(bad.is_empty(), "unpinned actions:\n{}", bad.join("\n"));
}

#[test]
fn the_pin_rule_rejects_tags_branches_and_missing_comments() {
    assert!(violation(" actions/checkout@v4").is_some());
    assert!(violation(" dtolnay/rust-toolchain@stable").is_some());
    let sha = "11d5960a326750d5838078e36cf38b85af677262";
    assert!(violation(&format!(" actions/checkout@{sha}")).is_some());
    assert!(violation(&format!(" actions/checkout@{sha} # v4.4.0")).is_none());
    assert!(violation(" ./.github/actions/x").is_none());
    assert!(violation(" docker://alpine:3").is_some());
    let digest = "a".repeat(64);
    assert!(violation(&format!(" docker://alpine@sha256:{digest}")).is_none());
}

/// `(ecosystem, default-days)` of every update block of dependabot.yml.
fn cooldowns(text: &str) -> Vec<(String, Option<u32>)> {
    let mut out: Vec<(String, Option<u32>)> = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if let Some(eco) = line.strip_prefix("- package-ecosystem:") {
            out.push((eco.trim().trim_matches('"').to_string(), None));
        } else if let Some(days) = line.strip_prefix("default-days:") {
            if let Some(last) = out.last_mut() {
                last.1 = days.split('#').next().unwrap().trim().parse().ok();
            }
        }
    }
    out
}

#[test]
fn dependabot_covers_cargo_and_actions_with_a_cooldown() {
    let text = std::fs::read_to_string(root().join(".github/dependabot.yml"))
        .expect(".github/dependabot.yml must exist");
    let found = cooldowns(&text);
    for ecosystem in ["cargo", "github-actions"] {
        let entry = found
            .iter()
            .find(|(e, _)| e == ecosystem)
            .unwrap_or_else(|| panic!("dependabot.yml has no `{ecosystem}` ecosystem"));
        assert!(
            entry.1.is_some_and(|d| d >= 2),
            "`{ecosystem}` needs `cooldown: default-days:` of at least 2, found {:?}",
            entry.1
        );
    }
}

#[test]
fn a_missing_or_short_cooldown_is_seen() {
    let short = "updates:\n  - package-ecosystem: cargo\n    cooldown:\n      default-days: 1\n  - package-ecosystem: github-actions\n";
    assert_eq!(
        cooldowns(short),
        vec![
            ("cargo".to_string(), Some(1)),
            ("github-actions".to_string(), None)
        ]
    );
}
