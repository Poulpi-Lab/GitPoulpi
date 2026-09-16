//! History rewrite, slice S21 (the history-rewrite design):
//! the LFS import and export rules. Expected values come from `git lfs`
//! itself (`git lfs pointer`, `git lfs fsck`, `git check-attr`), never from the
//! engine under test. Every check skips when git-lfs is not installed (the
//! rules need it), like the other LFS checks.

use gitpoulpi_core::domain::rewrite::{
    LfsExportRule, LfsImportRule, PathChange, Recipe, RewriteRefusal, Rule, RuleKind, Scope,
    SignaturePolicy,
};
use gitpoulpi_core::domain::Oid;
use gitpoulpi_core::{Error, GitBackend};

use super::rewrite::{git_bytes, git_text, history, object_files, preview, run, run_dirs};
use crate::{FixtureRepo, FixtureRepoBuilder};

const ATTRS_LINE: &str = "*.bin filter=lfs diff=lfs merge=lfs -text";

fn start() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .with_initial()
        .unwrap()
        .build()
}

/// `len` deterministic bytes with NULs in them (so the file is binary).
fn bytes(seed: u8, len: usize) -> Vec<u8> {
    (0..len)
        .map(|i| (i as u8).wrapping_mul(31).wrapping_add(seed))
        .collect()
}

fn put(repo: &FixtureRepo, files: &[(&str, &[u8])], message: &str) {
    for (path, content) in files {
        let full = repo.root().join(path);
        if let Some(dir) = full.parent() {
            std::fs::create_dir_all(dir).unwrap();
        }
        std::fs::write(full, content).unwrap();
    }
    repo.git(&["add", "-A"]);
    repo.git(&["commit", "-q", "-m", message]);
}

fn import(patterns: &[&str], min_size: Option<u64>) -> RuleKind {
    RuleKind::LfsImport(LfsImportRule {
        patterns: patterns.iter().map(|p| p.to_string()).collect(),
        min_size,
    })
}

fn export(patterns: &[&str]) -> RuleKind {
    RuleKind::LfsExport(LfsExportRule {
        patterns: patterns.iter().map(|p| p.to_string()).collect(),
    })
}

fn recipe(kind: RuleKind) -> Recipe {
    let mut r = Recipe::new("lfs", Scope::default(), vec![Rule::new(kind)]);
    r.options.signatures = Some(SignaturePolicy::Strip);
    r
}

/// What `git lfs pointer` writes for `content`: the oracle for a pointer.
fn oracle_pointer(repo: &FixtureRepo, content: &[u8]) -> String {
    let file = repo.root().join(".git").join("oracle-file");
    std::fs::write(&file, content).unwrap();
    let out = git_bytes(
        repo,
        &["lfs", "pointer", "--file", file.to_str().unwrap()],
        &[],
        b"",
    );
    let _ = std::fs::remove_file(&file);
    String::from_utf8(out).unwrap()
}

fn oid_of(pointer: &str) -> String {
    pointer
        .lines()
        .find_map(|l| l.strip_prefix("oid sha256:"))
        .unwrap()
        .to_string()
}

fn show(repo: &FixtureRepo, rev: &str, path: &str) -> Vec<u8> {
    git_bytes(
        repo,
        &["cat-file", "blob", &format!("{rev}:{path}")],
        &[],
        b"",
    )
}

fn blob_id(repo: &FixtureRepo, rev: &str, path: &str) -> String {
    git_text(repo, &["rev-parse", &format!("{rev}:{path}")])
}

fn store_object(repo: &FixtureRepo, oid: &str) -> std::path::PathBuf {
    repo.root()
        .join(".git")
        .join("lfs")
        .join("objects")
        .join(&oid[..2])
        .join(&oid[2..4])
        .join(oid)
}

fn oid(hex: &str) -> Oid {
    Oid::from_hex(hex).expect("a 40 character id")
}

/// Four commits: a `.gitattributes` of its own, text files, two `.bin`
/// files (one in a folder) and an edit of one of them. Returns the contents
/// by version.
struct Built {
    repo: FixtureRepo,
    big1: Vec<u8>,
    big3: Vec<u8>,
    art: Vec<u8>,
}

fn build() -> Built {
    let repo = start();
    put(
        &repo,
        &[
            (".gitattributes", b"* text=auto\n"),
            ("notes.txt", b"notes\n"),
            ("Makefile", b"all:\n"),
        ],
        "attributes",
    );
    let (big1, big3, art) = (bytes(1, 3000), bytes(3, 5000), bytes(2, 4000));
    put(
        &repo,
        &[("big.bin", &big1), ("sub/deep/art.bin", &art)],
        "add binaries",
    );
    put(&repo, &[("big.bin", &big3)], "edit big");
    put(&repo, &[("docs/readme.md", b"doc\n")], "add docs");
    Built {
        repo,
        big1,
        big3,
        art,
    }
}

fn strict_fsck(repo: &FixtureRepo) {
    git_text(repo, &["fsck", "--strict", "--no-dangling"]);
}

/// Import turns every matching blob into the pointer git-lfs itself writes,
/// puts the content in the LFS store under its sha256, adds the attribute
/// line to the `.gitattributes` of every commit that holds a pointer (the rest
/// of the file untouched, nothing duplicated), keeps every other blob id, and
/// the working tree shows the real files.
pub fn check_rewrite_lfs_import_makes_pointers_objects_and_attributes(backend: &dyn GitBackend) {
    crate::require_git_lfs!("check_rewrite_lfs_import_makes_pointers_objects_and_attributes");
    let b = build();
    let repo = &b.repo;
    let before = history(repo);
    let notes = blob_id(repo, "main", "notes.txt");
    let make = blob_id(repo, "main", "Makefile");

    let (plan, outcome) = run(backend, repo, &recipe(import(&["*.bin"], None)));

    let after = history(repo);
    assert_eq!(after.len(), before.len());
    assert_eq!(
        after[..2],
        before[..2],
        "commits with no binary keep their ids"
    );
    // The commit that only adds docs differs: its parent moved.
    let versions: [(usize, &[u8], &[u8]); 3] = [
        (2, &b.big1, &b.art),
        (3, &b.big3, &b.art),
        (4, &b.big3, &b.art),
    ];
    for (i, big, art) in versions {
        let c = &after[i];
        for (path, content) in [("big.bin", big), ("sub/deep/art.bin", art)] {
            let want = oracle_pointer(repo, content);
            assert_eq!(
                String::from_utf8(show(repo, c, path)).unwrap(),
                want,
                "pointer of {path} at commit {i}"
            );
            assert_eq!(
                std::fs::read(store_object(repo, &oid_of(&want))).unwrap(),
                content,
                "the object of {path} at commit {i}"
            );
            // `check-attr --source` is git 2.40; the `.gitattributes` blob is
            // asserted below either way.
            if crate::system_git_at_least(2, 40) {
                assert_eq!(
                    git_text(repo, &["check-attr", "--source", c, "filter", "--", path]),
                    format!("{path}: filter: lfs"),
                    "{path} at commit {i}"
                );
            }
        }
        let attrs = String::from_utf8(show(repo, c, ".gitattributes")).unwrap();
        assert_eq!(attrs, format!("* text=auto\n{ATTRS_LINE}\n"), "commit {i}");
        assert_eq!(blob_id(repo, c, "notes.txt"), notes);
        assert_eq!(blob_id(repo, c, "Makefile"), make);
    }
    git_text(repo, &["lfs", "fsck"]);
    strict_fsck(repo);

    assert_eq!(plan.counts.lfs_blobs, 3, "big v1, big v3 and the art");
    assert_eq!(
        plan.counts.lfs_bytes,
        (b.big1.len() + b.big3.len() + b.art.len()) as u64
    );
    assert_eq!(plan.counts.lfs_objects, 3);
    assert!(plan.counts.lfs_run);
    assert_eq!(outcome.commits_changed, 3);
    let row = plan
        .rows
        .iter()
        .find(|r| r.old == oid(&before[2]))
        .expect("a row for the commit that adds the binaries");
    assert!(
        row.paths.contains(&PathChange::LfsImported {
            path: "big.bin".into()
        }) && row.paths.contains(&PathChange::LfsImported {
            path: "sub/deep/art.bin".into()
        }),
        "{:?}",
        row.paths
    );
    // The checkout shows the real files, and git sees nothing to commit.
    assert_eq!(std::fs::read(repo.root().join("big.bin")).unwrap(), b.big3);
    assert_eq!(
        std::fs::read(repo.root().join("sub/deep/art.bin")).unwrap(),
        b.art
    );
    assert_eq!(git_text(repo, &["status", "--porcelain"]), "");
}

/// A repository without a `.gitattributes` gets one in the commits that need
/// it, and only a size threshold decides what moves.
pub fn check_rewrite_lfs_import_min_size_and_pointers_already_there(backend: &dyn GitBackend) {
    crate::require_git_lfs!("check_rewrite_lfs_import_min_size_and_pointers_already_there");
    let repo = start();
    let (small, large) = (bytes(5, 500), bytes(6, 6000));
    put(
        &repo,
        &[("small.bin", &small), ("large.bin", &large)],
        "add",
    );
    let before = history(&repo);

    let (plan, _) = run(backend, &repo, &recipe(import(&["*.bin"], Some(2000))));
    assert_eq!(plan.counts.lfs_blobs, 1);
    assert_eq!(
        show(&repo, "main", "small.bin"),
        small,
        "below the threshold"
    );
    assert_eq!(
        String::from_utf8(show(&repo, "main", "large.bin")).unwrap(),
        oracle_pointer(&repo, &large)
    );
    assert_eq!(
        String::from_utf8(show(&repo, "main", ".gitattributes")).unwrap(),
        format!("{ATTRS_LINE}\n"),
        "created, one line"
    );
    strict_fsck(&repo);
    let moved = history(&repo);
    assert_ne!(moved[1], before[1]);

    // A file under the threshold that the pattern names is a known quirk of
    // `.gitattributes` (git-lfs converts it on its next add): not pinned here.

    // A pointer is not converted again: nothing is left to do.
    let again_repo = start();
    put(&again_repo, &[("large.bin", &large)], "add");
    run(backend, &again_repo, &recipe(import(&["*.bin"], None)));
    let again = preview(backend, &again_repo, &recipe(import(&["*.bin"], None)));
    assert_eq!(again.counts.commits_changed, 0);
    assert_eq!(again.counts.lfs_blobs, 0);
}

/// Export puts back the very blobs (and, since the `.gitattributes` import
/// created goes away, the very commits) that import started from.
pub fn check_rewrite_lfs_export_round_trips_to_the_original_blob_ids(backend: &dyn GitBackend) {
    crate::require_git_lfs!("check_rewrite_lfs_export_round_trips_to_the_original_blob_ids");
    let b = build();
    let repo = &b.repo;
    let before = history(repo);
    let tree_before = git_text(repo, &["rev-parse", "main^{tree}"]);
    let big_before = blob_id(repo, "main", "big.bin");

    run(backend, repo, &recipe(import(&["*.bin"], None)));
    assert_ne!(blob_id(repo, "main", "big.bin"), big_before);

    let (plan, _) = run(backend, repo, &recipe(export(&["*.bin"])));

    assert_eq!(plan.counts.lfs_restored, 3);
    assert_eq!(blob_id(repo, "main", "big.bin"), big_before);
    assert_eq!(git_text(repo, &["rev-parse", "main^{tree}"]), tree_before);
    assert_eq!(history(repo), before, "the commits are the original ones");
    assert_eq!(
        String::from_utf8(show(repo, "main", ".gitattributes")).unwrap(),
        "* text=auto\n"
    );
    assert_eq!(std::fs::read(repo.root().join("big.bin")).unwrap(), b.big3);
    assert_eq!(git_text(repo, &["status", "--porcelain"]), "");
    strict_fsck(repo);
}

/// An export that needs objects the local store lacks is refused with the
/// list (`<oid> <path>` per line) and changes nothing.
pub fn check_rewrite_lfs_export_refuses_missing_objects_with_the_list(backend: &dyn GitBackend) {
    crate::require_git_lfs!("check_rewrite_lfs_export_refuses_missing_objects_with_the_list");
    let b = build();
    let repo = &b.repo;
    run(backend, repo, &recipe(import(&["*.bin"], None)));
    let lost = oid_of(&oracle_pointer(repo, &b.art));
    std::fs::remove_file(store_object(repo, &lost)).unwrap();
    let snapshot = (
        git_text(repo, &["for-each-ref"]),
        object_files(repo),
        run_dirs(repo),
    );

    let result =
        backend.rewrite_preview(&repo.repo_ref(), &recipe(export(&["*.bin"])), &mut |_| {
            std::ops::ControlFlow::Continue(())
        });
    match result {
        Err(Error::RewriteRefused(RewriteRefusal::LfsObjectsMissing, detail)) => {
            assert_eq!(
                detail.lines().collect::<Vec<_>>(),
                vec![format!("{lost} sub/deep/art.bin").as_str()],
                "only the object that is gone, with its path"
            );
        }
        other => panic!("expected LfsObjectsMissing, got {other:?}"),
    }
    assert_eq!(
        snapshot,
        (
            git_text(repo, &["for-each-ref"]),
            object_files(repo),
            run_dirs(repo)
        ),
        "a refused preview leaves nothing behind"
    );
}
