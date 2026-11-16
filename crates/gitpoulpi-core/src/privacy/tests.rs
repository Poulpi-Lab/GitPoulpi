use super::*;

fn defaults() -> Vec<String> {
    default_sensitive_patterns()
}

const BODY: &str = "q7Xk2LmZp9RtV4bN8wYcJ3hGfD5sAe6U1oIx";

fn token() -> String {
    format!("ghp_{BODY}")
}

#[test]
fn the_default_pattern_table() {
    let p = defaults();
    let sensitive = [
        ".env",
        "app/.env",
        ".env.local",
        ".env.production",
        "certs/server.pem",
        "tls/private.key",
        "id_rsa",
        "id_rsa.pub",
        "home/.ssh/id_ed25519",
        "id_ecdsa",
        "store.p12",
        "store.pfx",
        "release.jks",
        "debug.keystore",
        "ca.crt",
        "ca.cer",
        "ca.der",
        "vault.kdbx",
        "SERVER.PEM",
        "dir\\.env",
    ];
    for path in sensitive {
        assert!(
            matching_pattern(path, &p).is_some(),
            "{path} should be sensitive"
        );
    }
    let fine = [
        "src/main.rs",
        "README.md",
        "environment.rs",
        "keyboard.rs",
        "docs/key-concepts.md",
        "Cargo.toml",
        "dotenv.md",
    ];
    for path in fine {
        assert!(
            matching_pattern(path, &p).is_none(),
            "{path} should be fine"
        );
    }
}

#[test]
fn templates_are_re_allowed_by_the_negations() {
    let p = defaults();
    for path in [
        ".env.example",
        ".env.sample",
        ".env.template",
        "a/.env.example",
    ] {
        assert_eq!(matching_pattern(path, &p), None, "{path}");
    }
    // The negation only covers its own name.
    assert_eq!(matching_pattern(".env.example.local", &p), Some(".env.*"));
}

#[test]
fn the_last_matching_pattern_wins_and_slashes_anchor() {
    let p = ["*.log", "!keep.log", "/build/*.bin", "docs/**/secret.txt"];
    assert_eq!(matching_pattern("a/b.log", &p), Some("*.log"));
    assert_eq!(matching_pattern("a/keep.log", &p), None);
    assert_eq!(matching_pattern("build/x.bin", &p), Some("/build/*.bin"));
    assert_eq!(matching_pattern("sub/build/x.bin", &p), None);
    assert_eq!(matching_pattern("build/deep/x.bin", &p), None);
    assert_eq!(
        matching_pattern("docs/a/b/secret.txt", &p),
        Some("docs/**/secret.txt")
    );
    assert_eq!(
        matching_pattern("docs/secret.txt", &p),
        Some("docs/**/secret.txt")
    );
    assert_eq!(matching_pattern("any", &Vec::<String>::new()), None);
}

#[test]
fn classify_reports_the_pattern_and_the_secret_without_the_value() {
    let content = format!("TOKEN={}\n", token());
    let found = classify(".env", Some(content.as_bytes()), &defaults());
    assert!(found
        .iter()
        .any(|s| matches!(&s.reason, Reason::Pattern(p) if p == ".env")));
    let secret = found
        .iter()
        .find_map(|s| match &s.reason {
            Reason::Secret { rule, preview } => Some((rule.clone(), preview.clone())),
            _ => None,
        })
        .expect("a secret finding");
    assert_eq!(secret.0, "github-token");
    // Whatever is serialised or described never holds the value.
    let all = format!("{found:?}{}", serde_json::to_string(&found).unwrap());
    assert!(!all.contains(BODY), "{all}");
    for s in &found {
        assert!(!s.describe().contains("q7Xk2Lm"));
    }
}

#[test]
fn a_plain_file_without_secrets_is_not_sensitive() {
    assert!(classify("src/a.rs", Some(b"fn main() {}\n"), &defaults()).is_empty());
    assert!(classify("src/a.rs", None, &defaults()).is_empty());
}

#[test]
fn redact_replaces_each_finding_and_keeps_the_rest() {
    let text = format!("before {} after\nmore text", token());
    let (out, found) = redact(&text);
    assert_eq!(out, "before [REDACTED:github-token] after\nmore text");
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].rule, "github-token");
    assert!(found[0].preview.starts_with("ghp_****"));
    assert!(!out.contains("q7Xk2Lm"));
    let all = serde_json::to_string(&found).unwrap();
    assert!(!all.contains("q7Xk2LmZp9Rt"), "{all}");
}

#[test]
fn redact_leaves_a_clean_text_alone() {
    let (out, found) = redact("nothing to see here");
    assert_eq!(out, "nothing to see here");
    assert!(found.is_empty());
}

#[test]
fn a_secret_finding_carries_its_one_based_line_and_a_pattern_none() {
    let content = format!("# comment\n\nTOKEN={}\nother\n", token());
    let found = classify(".env", Some(content.as_bytes()), &defaults());
    let pattern = found
        .iter()
        .find(|s| matches!(s.reason, Reason::Pattern(_)))
        .unwrap();
    assert_eq!(pattern.line, None);
    let secret = found
        .iter()
        .find(|s| matches!(s.reason, Reason::Secret { .. }))
        .unwrap();
    assert_eq!(secret.line, Some(3));
    // The first line is line 1, not 0.
    let first = format!("{}\n", token());
    let found = classify("a.txt", Some(first.as_bytes()), &defaults());
    assert_eq!(found[0].line, Some(1));
    // A pattern finding leaves the line out of the wire form.
    let wire = serde_json::to_string(pattern).unwrap();
    assert!(!wire.contains("line"), "{wire}");
    // An answer without the field (an older daemon) still reads.
    let old: Sensitive =
        serde_json::from_str(r#"{"path":"a","reason":{"Pattern":"*.pem"}}"#).unwrap();
    assert_eq!(old.line, None);
}
