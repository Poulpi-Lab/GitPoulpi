use super::*;

/// A body with plenty of distinct characters, `n` long.
fn body(n: usize) -> String {
    const ALPHABET: &[u8] = b"q7Xk2LmZp9RtV4bN8wYcJ3hGfD5sAe6U1oIxT0yKzWvQn";
    (0..n)
        .map(|i| ALPHABET[(i * 7 + i / 3) % ALPHABET.len()] as char)
        .collect()
}

fn rules_in(text: &str) -> Vec<&'static str> {
    scan_text(text.as_bytes(), "src/config.txt")
        .iter()
        .map(|m| m.rule)
        .collect()
}

fn token_of(rule: &str) -> String {
    let shape = TOKEN_SHAPES.iter().find(|s| s.rule == rule).unwrap();
    // The longest body the shape accepts is not needed: the smallest is.
    let n = match rule {
        "github-token" => 36,
        "github-pat" => 50,
        "gitlab-token" => 24,
        "slack-token" => 24,
        "stripe-key" => 26,
        "anthropic-key" | "openai-key" => 40,
        "google-api-key" => 35,
        "npm-token" => 36,
        "aws-access-key" => 16,
        _ => 40,
    };
    let mut b = body(n);
    if rule == "aws-access-key" {
        b = b
            .to_uppercase()
            .replace(|c: char| !c.is_ascii_alphanumeric(), "A");
    }
    format!("{}{b}", shape.prefixes[0])
}

#[test]
fn every_token_shape_is_found_alone_in_a_line() {
    for shape in TOKEN_SHAPES {
        let token = token_of(shape.rule);
        let found = scan_text(format!("value = {token}\n").as_bytes(), "a.txt");
        assert_eq!(found.len(), 1, "{}: {found:?}", shape.rule);
        assert_eq!(found[0].rule, shape.rule);
        assert_eq!(found[0].value.reveal(), token.as_bytes(), "{}", shape.rule);
    }
}

#[test]
fn every_prefix_of_every_shape_is_found() {
    for shape in TOKEN_SHAPES {
        for prefix in shape.prefixes {
            let token = token_of(shape.rule).replacen(shape.prefixes[0], prefix, 1);
            let rules = rules_in(&format!("export X=\"{token}\""));
            assert_eq!(rules, [shape.rule], "{prefix}");
        }
    }
}

#[test]
fn the_anthropic_prefix_wins_over_the_generic_sk_one() {
    let token = token_of("anthropic-key");
    assert_eq!(rules_in(&token), ["anthropic-key"]);
}

#[test]
fn private_key_blocks_are_found_whole() {
    let key = format!(
        "-----BEGIN RSA PRIVATE KEY-----\n{}\n{}\n-----END RSA PRIVATE KEY-----",
        body(64),
        body(64)
    );
    let found = scan_text(format!("before\n{key}\nafter\n").as_bytes(), "id_rsa");
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].rule, RULE_PRIVATE_KEY);
    assert_eq!(found[0].value.reveal(), key.as_bytes());
    // The words in a document are not a key.
    assert!(rules_in("run: openssl genrsa  # -----BEGIN PRIVATE KEY----- header").is_empty());
}

#[test]
fn jwts_urls_and_assignments_are_found() {
    let jwt = format!("eyJ{}.eyJ{}.{}", body(20), body(24), body(30));
    assert_eq!(rules_in(&format!("Authorization: {jwt}")), [RULE_JWT]);

    let url = format!("postgres://app:{}@db.internal/prod", body(16));
    assert_eq!(rules_in(&url), [RULE_URL_CREDENTIALS]);

    assert_eq!(
        rules_in("DB_PASSWORD=\"Zq8!vK3mX9\"\n"),
        [RULE_GENERIC],
        "a quoted assignment"
    );
    assert_eq!(rules_in("api_key: k8Jd9sLq2Xw0\n"), [RULE_GENERIC]);
    assert_eq!(
        rules_in("client_secret = \"a1B2c3D4e5F6g7H8\""),
        [RULE_GENERIC]
    );
}

#[test]
fn a_generic_value_is_the_value_alone() {
    let found = scan_text(b"password = \"Zq8!vK3mX9\" # prod\n", "a.env");
    assert_eq!(found[0].value.reveal(), b"Zq8!vK3mX9");
}

#[test]
fn the_entropy_threshold_separates_words_from_random_values() {
    // Words and repeated characters stay under, random strings over.
    assert!(shannon_entropy(b"aaaaaaaaaaaa") < 0.1);
    assert!(shannon_entropy(b"passwordfield") < ENTROPY_MIN_ALPHA);
    assert!(shannon_entropy(b"Zq8!vK3mX9wL") > ENTROPY_MIN + 0.5);
    assert!(rules_in("password = aaaaaaaaaaaaaaaa").is_empty());
    assert!(rules_in("password = passwordField").is_empty());
    assert!(rules_in("password = abcabcabcabc").is_empty());
    // A value right above the threshold is found, right below it is not.
    assert!(shannon_entropy(b"abcdefgh1") >= ENTROPY_MIN);
    assert_eq!(rules_in("secret = abcdefgh1"), [RULE_GENERIC]);
    assert!(shannon_entropy(b"abcabcab1") < ENTROPY_MIN);
    assert!(rules_in("secret = abcabcab1").is_empty());
}

#[test]
fn placeholders_and_references_are_not_secrets() {
    for line in [
        "password = ${DB_PASSWORD}",
        "password: changeme-please",
        "token = <your token here>",
        "password = os.environ['DB_PASSWORD']",
        "secret = process.env.SECRET_KEY",
        "api_key = getenv(\"KEY_NAME\")",
        "password = %(db_password)s",
        "token = {{ vault_token }}",
        "password = \"********\"",
        "AWS_ACCESS_KEY_ID=AKIAIOSFODNN7EXAMPLE",
    ] {
        assert!(rules_in(line).is_empty(), "{line}: {:?}", rules_in(line));
    }
}

#[test]
fn lockfile_hashes_are_not_secrets() {
    let npm = format!(
        "{{\n  \"packages\": {{\n    \"node_modules/x\": {{\n      \"integrity\": \"sha512-{}==\",\n      \"resolved\": \"https://registry.npmjs.org/x/-/x-1.0.0.tgz\"\n    }}\n  }}\n}}\n",
        body(86)
    );
    assert!(scan_text(npm.as_bytes(), "package-lock.json").is_empty());
    let cargo = format!(
        "[[package]]\nname = \"x\"\nchecksum = \"{}\"\n",
        "ab12".repeat(16)
    );
    assert!(scan_text(cargo.as_bytes(), "Cargo.lock").is_empty());
    // Even a generic-looking token key in a lock file.
    assert!(scan_text(b"token = \"Zq8!vK3mX9wL\"", "yarn.lock").is_empty());
    // And outside lock files an integrity string under a secret-ish key.
    assert!(rules_in(&format!("token: sha512-{}", body(60))).is_empty());
}

#[test]
fn uuids_and_urls_and_data_uris_are_not_secrets() {
    for line in [
        "token = \"123e4567-e89b-12d3-a456-426614174000\"",
        "secret_url = https://example.org/some/long/path/x1y2z3",
        "api_key = \"data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPhfDwAChwGA60e6kgAAAABJRU5ErkJggg==\"",
    ] {
        assert!(rules_in(line).is_empty(), "{line}: {:?}", rules_in(line));
    }
}

#[test]
fn a_token_inside_base64_or_a_longer_word_is_not_a_token() {
    // The shape of an AWS key in the middle of a base64 image.
    let image = format!(
        "{}AKIAZ7VRSQ2OTFNKQXYB{}",
        "iVBORw0KGgoAAAANSUhEUgAAAAEA", "AAAAfFcSJAAAADUlEQVR42mNk"
    );
    assert!(rules_in(&image).is_empty(), "{:?}", rules_in(&image));
    // Letters after a 16-character key make it something else.
    assert!(rules_in("AKIAZ7VRSQ2OTFNKQXYBextra").is_empty());
    // `sk-` at the end of a word.
    assert!(rules_in(&format!("disk-{}", body(40))).is_empty());
    // The same key as a standalone word is found.
    assert_eq!(rules_in("id AKIAZ7VRSQ2OTFNKQXYB end"), ["aws-access-key"]);
}

#[test]
fn a_run_of_one_letter_is_not_a_key() {
    assert!(rules_in(&format!("ghp_{}", "a".repeat(36))).is_empty());
    assert!(rules_in("AKIAAAAAAAAAAAAAAAAA").is_empty());
}

#[test]
fn minified_lines_are_left_to_the_prefix_rules() {
    let long = format!("var a=\"{}\";token=\"Zq8!vK3mX9wL\"", "x".repeat(5000));
    assert!(rules_in(&long).is_empty());
    let with_token = format!(
        "var a=\"{}\";t=\"{}\"",
        "x".repeat(5000),
        token_of("github-token")
    );
    assert_eq!(rules_in(&with_token), ["github-token"]);
}

#[test]
fn matches_do_not_overlap_and_come_in_order() {
    let ghp = token_of("github-token");
    let text = format!("token = {ghp}\nother = {}\n", token_of("npm-token"));
    let found = scan_text(text.as_bytes(), "a.txt");
    // The token prefix wins over the generic assignment around it.
    assert_eq!(
        found.iter().map(|m| m.rule).collect::<Vec<_>>(),
        ["github-token", "npm-token"]
    );
    assert!(found[0].offset < found[1].offset);
}

#[test]
fn previews_show_at_most_a_prefix_and_four_characters() {
    let ghp = SecretValue::new(token_of("github-token"));
    let p = preview("github-token", &ghp);
    assert!(p.starts_with("ghp_****"), "{p}");
    assert_eq!(p.len(), "ghp_****".len() + 4);
    assert!(token_of("github-token").ends_with(&p[p.len() - 4..]));

    let short = SecretValue::new("Zq8!vK3mX9");
    assert_eq!(preview(RULE_GENERIC, &short), "**** (10 chars)");
    assert!(!preview(RULE_GENERIC, &short).contains("Zq8"));

    let long = SecretValue::new(body(40));
    let p = preview(RULE_GENERIC, &long);
    assert!(p.starts_with("****") && p.ends_with("(40 chars)"), "{p}");

    let key = SecretValue::new(format!(
        "-----BEGIN PRIVATE KEY-----\n{}\n-----END PRIVATE KEY-----",
        body(80)
    ));
    assert_eq!(
        preview(RULE_PRIVATE_KEY, &key),
        "-----BEGIN PRIVATE KEY-----****"
    );
}

#[test]
fn a_secret_value_never_prints_or_serializes() {
    let v = SecretValue::new("hunter2hunter2");
    assert!(!format!("{v:?}").contains("hunter2"));
    let m = SecretMatch {
        rule: RULE_GENERIC,
        value: v,
        offset: 3,
    };
    assert!(!format!("{m:?}").contains("hunter2"));
}

#[test]
fn finding_ids_have_one_shape() {
    assert!(is_finding_id("f-0123456789ab"));
    assert!(!is_finding_id("f-0123456789a"));
    assert!(!is_finding_id("f-0123456789AB"));
    assert!(!is_finding_id("g-0123456789ab"));
    assert!(!is_finding_id("hunter2hunter2"));
}

/// The scanner and `redact` read one table: a prefix added to the scanner is
/// masked in the application log, and the log masks nothing the scanner
/// would not call a token.
#[test]
fn the_scanner_and_redact_share_their_token_prefixes() {
    let table: Vec<&str> = token_prefixes().collect();
    assert!(table.len() >= 20, "{table:?}");
    for prefix in &table {
        let word = format!("{prefix}{}", body(40));
        let masked = crate::redact(&format!("pushed with {word} to origin"));
        assert_eq!(masked, "pushed with *** to origin", "{prefix}");
    }
    // And the other way: what redact masks by prefix is in the table.
    for shape in TOKEN_SHAPES {
        for prefix in shape.prefixes {
            assert!(table.contains(prefix));
        }
    }
    // Every rule id is unique.
    let mut ids = rule_ids();
    ids.sort_unstable();
    let n = ids.len();
    ids.dedup();
    assert_eq!(ids.len(), n);
}
