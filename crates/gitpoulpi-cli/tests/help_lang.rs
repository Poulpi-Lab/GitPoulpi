//! `gitpoulpi -l <lang> --help`: translated explanations, unchanged names.

use std::io::Write;
use std::process::{Command, Stdio};

fn gp(args: &[&str]) -> (String, String, bool) {
    let dir = tempfile::tempdir().unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_gitpoulpi"))
        .env("GITPOULPI_CONFIG_DIR", dir.path())
        .env_remove("GITPOULPI_LANG")
        .args(args)
        .output()
        .unwrap();
    (
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
        out.status.success(),
    )
}

#[test]
fn default_help_is_english_and_ignores_the_environment() {
    let (plain, _, ok) = gp(&["--help"]);
    assert!(ok);
    assert!(plain.contains("Usage: gitpoulpi"), "{plain}");
    assert!(plain.contains("Commands:"));
    let dir = tempfile::tempdir().unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_gitpoulpi"))
        .env("GITPOULPI_CONFIG_DIR", dir.path())
        .env("GITPOULPI_LANG", "fr")
        .env("LANG", "fr_FR.UTF-8")
        .arg("--help")
        .output()
        .unwrap();
    assert_eq!(String::from_utf8_lossy(&out.stdout), plain);
    let (en, _, _) = gp(&["-l", "en", "--help"]);
    assert_eq!(en, plain);
}

fn flags(help: &str) -> Vec<String> {
    let mut v: Vec<String> = help
        .split_whitespace()
        .filter(|w| w.starts_with("--"))
        .map(|w| w.trim_end_matches([',', '.']).to_string())
        .collect();
    v.sort();
    v.dedup();
    v
}

#[test]
fn french_root_help_translates_text_but_not_flags() {
    let (en, _, _) = gp(&["--help"]);
    let (fr, _, ok) = gp(&["-l", "fr", "--help"]);
    assert!(ok, "{fr}");
    assert!(fr.contains("Utilisation"), "{fr}");
    assert!(fr.contains("Commandes"), "{fr}");
    assert!(!fr.contains("Usage: gitpoulpi"));
    assert!(!fr.contains("Lightweight cross-platform"), "{fr}");
    assert_eq!(flags(&en), flags(&fr));
    for name in [
        "status",
        "commit",
        "branch",
        "--json",
        "--session",
        "--lang",
    ] {
        assert!(fr.contains(name), "{name} missing:\n{fr}");
    }
}

#[test]
fn french_subcommand_help_and_alternate_spellings() {
    let (en, _, _) = gp(&["commit", "--help"]);
    let (fr, _, ok) = gp(&["-l", "fr", "commit", "--help"]);
    assert!(ok, "{fr}");
    assert_ne!(en, fr);
    assert!(fr.contains("--message"), "{fr}");
    assert_eq!(flags(&en), flags(&fr));
    assert!(!fr.contains("The commit message."), "{fr}");
    // Same result for the other spellings, and `-l` after the subcommand.
    for args in [
        vec!["--lang", "fr", "commit", "--help"],
        vec!["--lang=fr", "commit", "--help"],
        vec!["-lfr", "commit", "--help"],
        vec!["commit", "-l", "fr", "--help"],
    ] {
        assert_eq!(gp(&args).0, fr, "{args:?}");
    }
}

#[test]
fn every_translated_language_renders_root_help() {
    for lang in [
        "fr", "es", "pt-BR", "pt-PT", "it", "de", "zh-CN", "ja", "ko", "ru", "uk", "pl", "cs",
        "nl", "sv", "tr",
    ] {
        let (out, err, ok) = gp(&["-l", lang, "--help"]);
        assert!(ok && err.is_empty(), "{lang}: {err}");
        assert!(!out.contains("Lightweight cross-platform"), "{lang}: {out}");
    }
}

#[test]
fn unknown_language_is_refused() {
    let (_, err, ok) = gp(&["-l", "klingon", "--help"]);
    assert!(!ok);
    assert!(err.contains("klingon"), "{err}");
}

fn tools_list(extra: &[&str]) -> String {
    let dir = tempfile::tempdir().unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_gitpoulpi"))
        .env("GITPOULPI_CONFIG_DIR", dir.path())
        .args(extra)
        .args(["--session", "cli-help-lang-mcp", "mcp", "serve", "--stdio"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let input = [
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"t","version":"0"}}}"#,
        r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#,
    ]
    .join("\n");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(format!("{input}\n").as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    String::from_utf8(out.stdout).unwrap()
}

#[test]
fn mcp_tool_list_stays_english_whatever_the_help_language() {
    let plain = tools_list(&[]);
    assert!(plain.contains("tools"), "{plain}");
    assert!(
        !plain.contains("\"lang\""),
        "`lang` leaked into the MCP schema"
    );
    assert_eq!(tools_list(&["-l", "fr"]), plain);
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
