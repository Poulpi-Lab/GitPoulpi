//! History rewrite, slices S22 and S24bis (the history-rewrite design):
//! line endings, the exec bit and file encodings. Expected bytes are written
//! out in the tests and read back with plain `git`, never from the engine.

use gitpoulpi_core::domain::rewrite::{
    EolRule, LineEnding, PathChange, Recipe, ReencodeRule, Rule, RuleKind, Scope, SignaturePolicy,
    StripExecRule, TextEncoding,
};
use gitpoulpi_core::GitBackend;

use super::rewrite::{git_bytes, git_text, history, run};
use crate::{FixtureRepo, FixtureRepoBuilder};

fn start() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .with_initial()
        .unwrap()
        .build()
}

fn put(repo: &FixtureRepo, files: &[(&str, &[u8])], message: &str) {
    for (path, bytes) in files {
        let full = repo.root().join(path);
        if let Some(dir) = full.parent() {
            std::fs::create_dir_all(dir).unwrap();
        }
        std::fs::write(full, bytes).unwrap();
    }
    repo.git(&["add", "-A"]);
    repo.git(&["commit", "-q", "-m", message]);
}

fn recipe(kinds: Vec<RuleKind>) -> Recipe {
    let mut r = Recipe::new(
        "textconv",
        Scope::default(),
        kinds.into_iter().map(Rule::new).collect(),
    );
    r.options.signatures = Some(SignaturePolicy::Strip);
    r
}

fn globs(g: &[&str]) -> Vec<String> {
    g.iter().map(|s| s.to_string()).collect()
}

fn eol(target: LineEnding, paths: &[&str], exclude: &[&str]) -> RuleKind {
    RuleKind::LineEndings(EolRule {
        target,
        paths: globs(paths),
        exclude: globs(exclude),
        follow_attributes: false,
        max_blob_bytes: None,
    })
}

fn reencode(target: TextEncoding, from: Option<TextEncoding>, paths: &[&str]) -> RuleKind {
    RuleKind::Reencode(ReencodeRule {
        target,
        from,
        fallback: TextEncoding::Windows1252,
        paths: globs(paths),
        exclude: Vec::new(),
        max_blob_bytes: None,
    })
}

fn blob_id(repo: &FixtureRepo, rev: &str, path: &str) -> String {
    git_text(repo, &["rev-parse", &format!("{rev}:{path}")])
}

fn blob(repo: &FixtureRepo, rev: &str, path: &str) -> Vec<u8> {
    git_bytes(
        repo,
        &["cat-file", "blob", &format!("{rev}:{path}")],
        &[],
        b"",
    )
}

fn mode_of(repo: &FixtureRepo, rev: &str, path: &str) -> String {
    git_text(repo, &["ls-tree", rev, "--", path])
        .split(' ')
        .next()
        .unwrap()
        .to_string()
}

/// CRLF and mixed text become LF; a binary file with CRLF bytes, a file
/// the path filter does not name and one the exclude names keep their blob
/// ids; the plan counts the skipped binary file and lists the converted ones.
pub fn check_rewrite_eol_to_lf_converts_text_only_and_keeps_other_blob_ids(
    backend: &dyn GitBackend,
) {
    let repo = start();
    put(
        &repo,
        &[
            ("a.txt", b"one\r\ntwo\r\n"),
            ("mixed.txt", b"one\r\ntwo\nthree\rfour"),
            ("lf.txt", b"already\n"),
            ("pic.dat", b"\x00\x01\r\nbinary\r\n\x00"),
            ("main.rs", b"fn main() {}\r\n"),
            ("vendor/v.txt", b"vendored\r\n"),
        ],
        "add files",
    );
    let before = history(&repo);
    let lf = blob_id(&repo, "main", "lf.txt");
    let pic = blob_id(&repo, "main", "pic.dat");
    let rs = blob_id(&repo, "main", "main.rs");
    let vendor = blob_id(&repo, "main", "vendor/v.txt");

    let (plan, _) = run(
        backend,
        &repo,
        &recipe(vec![eol(LineEnding::Lf, &["*.txt", "*.dat"], &["vendor/"])]),
    );

    assert_eq!(blob(&repo, "main", "a.txt"), b"one\ntwo\n");
    assert_eq!(blob(&repo, "main", "mixed.txt"), b"one\ntwo\nthree\nfour");
    assert_eq!(blob_id(&repo, "main", "lf.txt"), lf, "already LF");
    assert_eq!(blob_id(&repo, "main", "pic.dat"), pic, "binary untouched");
    assert_eq!(blob_id(&repo, "main", "main.rs"), rs, "not in the filter");
    assert_eq!(blob_id(&repo, "main", "vendor/v.txt"), vendor, "excluded");
    // The oracle: the id git gives the expected text.
    let expected = git_bytes(&repo, &["hash-object", "--stdin"], &[], b"one\ntwo\n");
    assert_eq!(
        blob_id(&repo, "main", "a.txt"),
        String::from_utf8(expected).unwrap().trim()
    );
    assert_eq!(plan.counts.blobs_changed, 2);
    assert_eq!(plan.counts.blobs_skipped_binary, 1);
    assert_eq!(plan.counts.commits_changed, 1);
    let row = plan.rows.iter().find(|r| r.paths.len() == 2).expect("row");
    assert!(row.paths.contains(&PathChange::Modified {
        path: "a.txt".into()
    }));
    assert_ne!(history(&repo)[1], before[1]);
    assert_eq!(history(&repo)[0], before[0]);
}

/// LF becomes CRLF and then CR, and every ending is rewritten whatever it
/// was.
pub fn check_rewrite_eol_to_crlf_and_cr_use_the_chosen_ending(backend: &dyn GitBackend) {
    let build = || {
        let repo = start();
        put(&repo, &[("a.txt", b"one\ntwo\r\nthree\n")], "add");
        repo
    };
    let repo = build();
    run(
        backend,
        &repo,
        &recipe(vec![eol(LineEnding::Crlf, &["*.txt"], &[])]),
    );
    assert_eq!(blob(&repo, "main", "a.txt"), b"one\r\ntwo\r\nthree\r\n");

    let repo = build();
    run(
        backend,
        &repo,
        &recipe(vec![eol(LineEnding::Cr, &["*.txt"], &[])]),
    );
    assert_eq!(blob(&repo, "main", "a.txt"), b"one\rtwo\rthree\r");
}

/// With "follow attributes" a `binary` file is skipped and an `eol=crlf`
/// file gets CRLF whatever the target is; without it the target rules alone.
pub fn check_rewrite_eol_can_follow_gitattributes(backend: &dyn GitBackend) {
    let build = || {
        let repo = start();
        put(
            &repo,
            &[
                (".gitattributes", b"*.bat eol=crlf\n*.lock binary\n"),
                ("run.bat", b"echo\n"),
                ("x.lock", b"a\r\nb\r\n"),
                ("n.txt", b"a\r\n"),
            ],
            "add",
        );
        repo
    };
    let repo = build();
    let lock = blob_id(&repo, "main", "x.lock");
    let mut rule = EolRule {
        target: LineEnding::Lf,
        paths: Vec::new(),
        exclude: Vec::new(),
        follow_attributes: true,
        max_blob_bytes: None,
    };
    run(
        backend,
        &repo,
        &recipe(vec![RuleKind::LineEndings(rule.clone())]),
    );
    assert_eq!(blob(&repo, "main", "run.bat"), b"echo\r\n");
    assert_eq!(blob_id(&repo, "main", "x.lock"), lock);
    assert_eq!(blob(&repo, "main", "n.txt"), b"a\n");

    let repo = build();
    rule.follow_attributes = false;
    run(backend, &repo, &recipe(vec![RuleKind::LineEndings(rule)]));
    assert_eq!(blob(&repo, "main", "run.bat"), b"echo\n");
    assert_eq!(blob(&repo, "main", "x.lock"), b"a\nb\n");
}

/// The exec bit goes only from the files the globs name; the blobs keep
/// their ids and the other executables keep mode 100755.
pub fn check_rewrite_strip_exec_changes_the_mode_of_matching_paths_only(backend: &dyn GitBackend) {
    let repo = start();
    for path in ["run.sh", "tools/x.sh", "keep.bin"] {
        let full = repo.root().join(path);
        std::fs::create_dir_all(full.parent().unwrap()).unwrap();
        std::fs::write(&full, format!("#!/bin/sh\n# {path}\n")).unwrap();
        repo.git(&["add", "--chmod=+x", path]);
    }
    // The working files carry the bit too, or the repository is "dirty".
    #[cfg(unix)]
    for path in ["run.sh", "tools/x.sh", "keep.bin"] {
        use std::os::unix::fs::PermissionsExt;
        let full = repo.root().join(path);
        std::fs::set_permissions(full, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    repo.git(&["commit", "-q", "-m", "executables"]);
    let ids: Vec<String> = ["run.sh", "tools/x.sh", "keep.bin"]
        .iter()
        .map(|p| blob_id(&repo, "main", p))
        .collect();

    let (plan, _) = run(
        backend,
        &repo,
        &recipe(vec![RuleKind::StripExec(StripExecRule {
            paths: globs(&["*.sh"]),
            exclude: globs(&["tools/"]),
        })]),
    );

    assert_eq!(mode_of(&repo, "main", "run.sh"), "100644");
    assert_eq!(mode_of(&repo, "main", "tools/x.sh"), "100755", "excluded");
    assert_eq!(mode_of(&repo, "main", "keep.bin"), "100755", "not named");
    for (path, id) in ["run.sh", "tools/x.sh", "keep.bin"].iter().zip(&ids) {
        assert_eq!(&blob_id(&repo, "main", path), id, "{path}: same content");
    }
    assert_eq!(plan.counts.commits_changed, 1);
    let row = plan.rows.last().expect("a row");
    assert_eq!(
        row.paths,
        vec![PathChange::Modified {
            path: "run.sh".into()
        }]
    );
}

fn utf16le(text: &str, bom: bool) -> Vec<u8> {
    let mut out = if bom { vec![0xFF, 0xFE] } else { Vec::new() };
    for u in text.encode_utf16() {
        out.extend_from_slice(&u.to_le_bytes());
    }
    out
}

/// Latin-1 / Windows-1252 text becomes UTF-8 (the e acute, the euro sign),
/// UTF-16 with a mark becomes UTF-8, a file already UTF-8 and a binary file
/// keep their ids, and a file the source cannot read is skipped and counted
/// instead of being mangled.
pub fn check_rewrite_reencode_detects_the_source_and_skips_the_undecodable(
    backend: &dyn GitBackend,
) {
    let repo = start();
    let utf8 = "caf\u{e9} \u{20AC}\n".as_bytes().to_vec();
    let bad16: &[u8] = b"\xFF\xFEA";
    put(
        &repo,
        &[
            ("latin.txt", b"caf\xE9\n"),
            ("euro.txt", b"5 \x80\n"),
            ("wide.txt", &utf16le("h\u{e9}llo\n", true)),
            ("already.txt", &utf8),
            ("bad16.txt", bad16),
            ("pic.png", b"\x89PNG\x00\x00\x00"),
            ("skip.md", b"caf\xE9\n"),
        ],
        "add files",
    );
    let already = blob_id(&repo, "main", "already.txt");
    let bad = blob_id(&repo, "main", "bad16.txt");
    let skip = blob_id(&repo, "main", "skip.md");

    let (plan, _) = run(
        backend,
        &repo,
        &recipe(vec![reencode(
            TextEncoding::Utf8,
            None,
            &["*.txt", "*.png"],
        )]),
    );

    assert_eq!(blob(&repo, "main", "latin.txt"), "caf\u{e9}\n".as_bytes());
    assert_eq!(blob(&repo, "main", "euro.txt"), "5 \u{20AC}\n".as_bytes());
    assert_eq!(blob(&repo, "main", "wide.txt"), "h\u{e9}llo\n".as_bytes());
    assert_eq!(blob_id(&repo, "main", "already.txt"), already);
    assert_eq!(blob_id(&repo, "main", "bad16.txt"), bad, "never mangled");
    assert_eq!(blob_id(&repo, "main", "skip.md"), skip, "not in the filter");
    assert_eq!(plan.counts.blobs_changed, 3);
    assert_eq!(plan.counts.blobs_skipped_undecodable, 1);
    assert_eq!(plan.counts.blobs_skipped_binary, 1);
}

/// UTF-8 becomes Windows-1252, ISO-8859-1, UTF-16 and UTF-8 with a mark (and
/// back); a character the target cannot hold (the euro in Latin-1) skips the
/// file; a given source overrides the detection.
pub fn check_rewrite_reencode_writes_each_target_and_refuses_unmappable_characters(
    backend: &dyn GitBackend,
) {
    let build = || {
        let repo = start();
        put(
            &repo,
            &[
                ("a.txt", "caf\u{e9}\n".as_bytes()),
                ("b.txt", "5 \u{20AC}\n".as_bytes()),
            ],
            "add",
        );
        repo
    };
    for (target, a, b) in [
        (
            TextEncoding::Windows1252,
            b"caf\xE9\n".to_vec(),
            b"5 \x80\n".to_vec(),
        ),
        (
            TextEncoding::Utf16Le,
            utf16le("caf\u{e9}\n", true),
            utf16le("5 \u{20AC}\n", true),
        ),
        (
            TextEncoding::Utf8Bom,
            b"\xEF\xBB\xBFcaf\xC3\xA9\n".to_vec(),
            b"\xEF\xBB\xBF5 \xE2\x82\xAC\n".to_vec(),
        ),
    ] {
        let repo = build();
        run(
            backend,
            &repo,
            &recipe(vec![reencode(target, None, &["*.txt"])]),
        );
        assert_eq!(blob(&repo, "main", "a.txt"), a, "{target:?}");
        assert_eq!(blob(&repo, "main", "b.txt"), b, "{target:?}");
    }

    // Latin-1 holds the e acute, not the euro sign: that file stays.
    let repo = build();
    let euro = blob_id(&repo, "main", "b.txt");
    let (plan, _) = run(
        backend,
        &repo,
        &recipe(vec![reencode(TextEncoding::Latin1, None, &["*.txt"])]),
    );
    assert_eq!(blob(&repo, "main", "a.txt"), b"caf\xE9\n");
    assert_eq!(blob_id(&repo, "main", "b.txt"), euro);
    assert_eq!(plan.counts.blobs_skipped_undecodable, 1);

    // A given source is believed: UTF-8 read as Latin-1 is double-encoded.
    let repo = build();
    run(
        backend,
        &repo,
        &recipe(vec![reencode(
            TextEncoding::Utf8,
            Some(TextEncoding::Latin1),
            &["a.txt"],
        )]),
    );
    assert_eq!(blob(&repo, "main", "a.txt"), "caf\u{c3}\u{a9}\n".as_bytes());
}

/// Line endings and the encoding compose in recipe order on one file, and a
/// file converted to UTF-16 by an earlier rule is binary for a later one.
pub fn check_rewrite_eol_and_encoding_rules_compose_in_recipe_order(backend: &dyn GitBackend) {
    let build = || {
        let repo = start();
        put(&repo, &[("a.txt", b"caf\xE9\r\nok\r\n")], "add");
        repo
    };
    let repo = build();
    run(
        backend,
        &repo,
        &recipe(vec![
            reencode(TextEncoding::Utf8, None, &["*.txt"]),
            eol(LineEnding::Lf, &["*.txt"], &[]),
        ]),
    );
    assert_eq!(blob(&repo, "main", "a.txt"), "caf\u{e9}\nok\n".as_bytes());

    let repo = build();
    let (plan, _) = run(
        backend,
        &repo,
        &recipe(vec![
            reencode(TextEncoding::Utf16Le, None, &["a.txt"]),
            eol(LineEnding::Lf, &["a.txt"], &[]),
        ]),
    );
    assert_eq!(
        blob(&repo, "main", "a.txt"),
        utf16le("caf\u{e9}\r\nok\r\n", true),
        "the second rule sees UTF-16 and leaves it"
    );
    assert_eq!(plan.counts.blobs_skipped_binary, 1);
}
