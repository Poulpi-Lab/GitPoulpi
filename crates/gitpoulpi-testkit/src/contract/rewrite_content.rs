//! History rewrite, slice S9 (the history-rewrite design):
//! the content rules (`ContentReplace`, `RemovePaths`, `RemoveBlobsOver`),
//! empty-commit pruning, the working tree after an apply, the dirty-path and
//! linked-worktree refusals and the blob diff. Expected values come from
//! plain `git`, never from the engine under test. Tokens are fake.

use std::ops::ControlFlow;

use gitpoulpi_core::domain::rewrite::{
    BlobChangeKind, BlobLineKind, ContentFind, ContentReplace, PathChange, PruneEmpty, Recipe,
    RewriteRefusal, Rule, RuleKind, Scope, SecretText, SignaturePolicy, TextPattern,
};
use gitpoulpi_core::domain::Oid;
use gitpoulpi_core::GitBackend;

use super::rewrite::{git_bytes, git_text, history, object_files, preview, refused, run, run_dirs};
use crate::{FixtureRepo, FixtureRepoBuilder};

const TOKEN: &str = "EXAMPLE_TOKEN_123";

fn start() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .with_initial()
        .unwrap()
        .build()
}

/// Writes `files` and commits them all.
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
        "content",
        Scope::default(),
        kinds.into_iter().map(Rule::new).collect(),
    );
    r.options.signatures = Some(SignaturePolicy::Strip);
    r
}

fn replace_literal(find: &str, with: &str) -> RuleKind {
    RuleKind::ContentReplace(ContentReplace {
        paths: Vec::new(),
        find: ContentFind::Pattern {
            pattern: TextPattern::literal(find),
        },
        with: with.to_string(),
        max_blob_bytes: None,
    })
}

fn replace_secret(secret: &str) -> RuleKind {
    RuleKind::ContentReplace(ContentReplace {
        paths: Vec::new(),
        find: ContentFind::Secrets {
            literals: vec![SecretText(secret.to_string())],
            scan: false,
        },
        with: String::new(),
        max_blob_bytes: None,
    })
}

fn remove_paths(globs: &[&str]) -> RuleKind {
    RuleKind::RemovePaths {
        globs: globs.iter().map(|g| g.to_string()).collect(),
    }
}

fn blob_at(repo: &FixtureRepo, rev: &str, path: &str) -> String {
    git_text(repo, &["rev-parse", &format!("{rev}:{path}")])
}

fn subjects(repo: &FixtureRepo) -> Vec<String> {
    git_text(repo, &["log", "--reverse", "--format=%s", "HEAD"])
        .lines()
        .map(str::to_string)
        .collect()
}

fn fsck(repo: &FixtureRepo) {
    git_text(repo, &["fsck", "--connectivity-only", "--no-dangling"]);
}

fn snapshot(repo: &FixtureRepo) -> (String, Vec<String>, Vec<String>) {
    (
        git_text(repo, &["for-each-ref"]),
        object_files(repo),
        run_dirs(repo),
    )
}

fn try_preview(
    backend: &dyn GitBackend,
    repo: &FixtureRepo,
    recipe: &Recipe,
) -> Result<gitpoulpi_core::domain::rewrite::RewritePlan, gitpoulpi_core::Error> {
    backend.rewrite_preview(&repo.repo_ref(), recipe, &mut |_| ControlFlow::Continue(()))
}

fn oid(hex: &str) -> Oid {
    Oid::from_hex(hex).expect("a 40 character id")
}

/// Replacing text rewrites only the blobs that hold it: every other blob
/// keeps its id, the replaced content is what plain git hashes for the
/// expected text, a regex gives the same trees as the literal, and the
/// working tree follows.
pub fn check_rewrite_replaces_text_and_keeps_unrelated_blob_ids(backend: &dyn GitBackend) {
    let build = || {
        let repo = start();
        put(
            &repo,
            &[
                ("a.txt", format!("key = {TOKEN}\nline\n").as_bytes()),
                ("other.txt", b"unrelated\n"),
            ],
            "add a",
        );
        put(
            &repo,
            &[("a.txt", format!("key = {TOKEN}\nline 2\n").as_bytes())],
            "edit a",
        );
        put(&repo, &[("c/d.txt", b"nothing here\n")], "add d");
        repo
    };
    let repo = build();
    let before = history(&repo);
    let other = blob_at(&repo, "main", "other.txt");
    let d = blob_at(&repo, "main", "c/d.txt");

    let (plan, outcome) = run(
        backend,
        &repo,
        &recipe(vec![replace_literal(TOKEN, "REDACTED")]),
    );

    let after = history(&repo);
    assert_eq!(after.len(), before.len());
    assert_eq!(after[0], before[0], "a commit nothing touches keeps its id");
    for (i, id) in after.iter().enumerate().skip(1) {
        assert_ne!(*id, before[i]);
        assert_eq!(blob_at(&repo, id, "other.txt"), other, "commit {i}");
    }
    assert_eq!(blob_at(&repo, "main", "c/d.txt"), d);
    // The oracle: the id git gives the expected text.
    let expected = git_bytes(
        &repo,
        &["hash-object", "--stdin"],
        &[],
        b"key = REDACTED\nline 2\n",
    );
    assert_eq!(
        blob_at(&repo, "main", "a.txt"),
        String::from_utf8(expected).unwrap().trim()
    );
    let log = git_text(&repo, &["log", "-p", "--all"]);
    assert!(!log.contains(TOKEN), "the token is gone from every commit");
    assert_eq!(plan.counts.blobs_changed, 2);
    assert_eq!(plan.counts.commits_changed, 3);
    assert_eq!(outcome.commits_changed, 3);
    let row = plan
        .rows
        .iter()
        .find(|r| r.old == oid(&before[1]))
        .expect("a row for the first commit that held the token");
    assert_eq!(
        row.paths,
        vec![PathChange::Modified {
            path: "a.txt".into()
        }]
    );
    // The working tree followed the checked-out branch.
    assert_eq!(
        std::fs::read_to_string(repo.root().join("a.txt")).unwrap(),
        "key = REDACTED\nline 2\n"
    );
    assert_eq!(git_text(&repo, &["status", "--porcelain"]), "");
    fsck(&repo);

    // A regex finds the same text.
    let other_repo = build();
    let regex = RuleKind::ContentReplace(ContentReplace {
        paths: Vec::new(),
        find: ContentFind::Pattern {
            pattern: TextPattern::regex(r"EXAMPLE_TOKEN_\d+"),
        },
        with: "REDACTED".into(),
        max_blob_bytes: None,
    });
    run(backend, &other_repo, &recipe(vec![regex]));
    assert_eq!(
        git_text(&other_repo, &["rev-parse", "main^{tree}"]),
        git_text(&repo, &["rev-parse", "main^{tree}"])
    );

    // Only the files a glob names.
    let scoped = build();
    let mut only = replace_literal(TOKEN, "REDACTED");
    if let RuleKind::ContentReplace(c) = &mut only {
        c.paths = vec!["other.txt".into()];
    }
    let plan = preview(backend, &scoped, &recipe(vec![only]));
    assert_eq!(plan.counts.commits_changed, 0, "no matching file holds it");
}

/// A folder (and a pattern) is removed from every commit; the files left keep
/// their ids and the working tree loses the folder.
pub fn check_rewrite_removes_a_folder_from_all_history(backend: &dyn GitBackend) {
    let repo = start();
    put(
        &repo,
        &[("keep.txt", b"keep\n"), ("secrets/key.pem", b"k1\n")],
        "add keys",
    );
    put(
        &repo,
        &[("secrets/deep/more.txt", b"m\n"), ("app.log", b"log\n")],
        "add more",
    );
    put(&repo, &[("src/main.rs", b"fn main() {}\n")], "add source");
    let keep = blob_at(&repo, "main", "keep.txt");
    let src = blob_at(&repo, "main", "src/main.rs");
    let before = history(&repo);

    let (plan, _) = run(
        backend,
        &repo,
        &recipe(vec![remove_paths(&["secrets/", "*.log"])]),
    );

    let objects = git_text(&repo, &["rev-list", "--objects", "--all"]);
    assert!(!objects.contains("secrets"), "{objects}");
    assert!(!objects.contains("app.log"));
    assert_eq!(blob_at(&repo, "main", "keep.txt"), keep);
    assert_eq!(blob_at(&repo, "main", "src/main.rs"), src);
    assert_eq!(
        git_text(&repo, &["log", "--all", "--format=%H", "--", "secrets"]),
        ""
    );
    assert!(!repo.root().join("secrets").exists());
    assert!(!repo.root().join("app.log").exists());
    assert_eq!(git_text(&repo, &["status", "--porcelain"]), "");
    // The first commit that held nothing of it keeps its id.
    assert_eq!(history(&repo)[0], before[0]);
    let first = plan
        .rows
        .iter()
        .find(|r| r.old == oid(&before[1]))
        .expect("a row for 'add keys'");
    assert_eq!(
        first.paths,
        vec![PathChange::Removed {
            path: "secrets".into()
        }]
    );
    fsck(&repo);

    // An anchored glob only matches at the root.
    let repo = start();
    put(
        &repo,
        &[("docs/a.md", b"a\n"), ("x/docs/a.md", b"b\n")],
        "docs",
    );
    run(backend, &repo, &recipe(vec![remove_paths(&["docs/*.md"])]));
    let files = git_text(&repo, &["ls-tree", "-r", "--name-only", "main"]);
    assert!(files.contains("x/docs/a.md"), "{files}");
    assert!(!files.lines().any(|l| l == "docs/a.md"), "{files}");
}

/// Blobs over a size are dropped from every commit, smaller ones stay.
pub fn check_rewrite_drops_blobs_over_a_size(backend: &dyn GitBackend) {
    let repo = start();
    let big = vec![b'x'; 5000];
    put(
        &repo,
        &[("big.dat", &big), ("small.txt", b"small\n")],
        "add big",
    );
    put(&repo, &[("small.txt", b"small 2\n")], "edit small");
    let small = blob_at(&repo, "main", "small.txt");

    let (plan, _) = run(
        backend,
        &repo,
        &recipe(vec![RuleKind::RemoveBlobsOver { bytes: 1000 }]),
    );

    assert_eq!(blob_at(&repo, "main", "small.txt"), small);
    let objects = git_text(&repo, &["rev-list", "--objects", "--all"]);
    assert!(!objects.contains("big.dat"));
    assert!(plan.counts.commits_changed >= 1);
    for line in git_text(&repo, &["rev-list", "--objects", "--all"]).lines() {
        let Some((id, _)) = line.split_once(' ') else {
            continue;
        };
        let size = git_text(&repo, &["cat-file", "-s", id]);
        assert!(size.parse::<u64>().unwrap() <= 1000, "{line} is {size}");
    }
    fsck(&repo);
}

/// A commit a content rule empties is dropped and its children are
/// re-parented; one that was empty before stays; `never` keeps everything.
pub fn check_rewrite_prunes_commits_that_became_empty(backend: &dyn GitBackend) {
    let build = || {
        let repo = start();
        put(&repo, &[("secret.txt", b"s1\n")], "only secret");
        put(&repo, &[("a.txt", b"a\n")], "real change");
        put(&repo, &[("secret.txt", b"s2\n")], "secret again");
        repo.git(&["commit", "-q", "--allow-empty", "-m", "was already empty"]);
        put(&repo, &[("b.txt", b"b\n")], "another change");
        repo
    };
    let repo = build();
    let old = history(&repo);
    let (plan, outcome) = run(backend, &repo, &recipe(vec![remove_paths(&["secret.txt"])]));
    assert_eq!(
        subjects(&repo),
        [
            "initial",
            "real change",
            "was already empty",
            "another change"
        ]
    );
    assert_eq!(plan.counts.commits_dropped, 2);
    let dropped: Vec<Oid> = plan
        .rows
        .iter()
        .filter(|r| r.new.is_none())
        .map(|r| r.old)
        .collect();
    assert_eq!(dropped, vec![oid(&old[1]), oid(&old[3])]);
    assert_eq!(outcome.commits_changed, plan.counts.commits_changed);
    // The new first commit sits on the initial one (re-parented).
    let new = history(&repo);
    assert_eq!(new[0], old[0]);
    assert_eq!(
        git_text(&repo, &["rev-parse", &format!("{}^", new[1])]),
        new[0]
    );
    // The commit map says a dropped commit has no successor.
    let map_dir = repo.root().join(".git/gitpoulpi/rewrite");
    let map = std::fs::read_dir(map_dir)
        .unwrap()
        .map(|e| e.unwrap().path().join("commit-map"))
        .find(|p| p.is_file())
        .map(|p| std::fs::read_to_string(p).unwrap())
        .expect("the commit-map stays after an apply");
    assert!(
        map.contains(&format!("{} {}", old[1], "0".repeat(40))),
        "{map}"
    );
    fsck(&repo);

    // `never` keeps the emptied commits.
    let repo = build();
    let mut keep = recipe(vec![remove_paths(&["secret.txt"])]);
    keep.options.prune_empty = PruneEmpty::Never;
    let (plan, _) = run(backend, &repo, &keep);
    assert_eq!(plan.counts.commits_dropped, 0);
    assert_eq!(subjects(&repo).len(), 6);

    // A merge whose parents collapse onto one commit goes with it.
    let repo = start();
    put(&repo, &[("m1.txt", b"1\n")], "m1");
    repo.git(&["checkout", "-q", "-b", "side"]);
    put(&repo, &[("secret.txt", b"s\n")], "side only secret");
    repo.git(&["checkout", "-q", "main"]);
    repo.git(&["merge", "--no-ff", "-q", "-m", "merge side", "side"]);
    assert_eq!(subjects(&repo).len(), 4);
    let mut all = recipe(vec![remove_paths(&["secret.txt"])]);
    all.scope = Scope::AllLocal;
    run(backend, &repo, &all);
    assert_eq!(subjects(&repo), ["initial", "m1"]);
    assert_eq!(
        git_text(&repo, &["rev-parse", "side"]),
        git_text(&repo, &["rev-parse", "main"])
    );
    fsck(&repo);
}

/// A binary file that holds the text is left alone and counted; a text file
/// beside it is replaced.
pub fn check_rewrite_content_replace_skips_binary_blobs(backend: &dyn GitBackend) {
    let repo = start();
    let mut binary = b"\x00\x01 ".to_vec();
    binary.extend_from_slice(TOKEN.as_bytes());
    binary.extend_from_slice(b"\x00\x02");
    put(
        &repo,
        &[
            ("blob.bin", &binary),
            ("t.txt", format!("x {TOKEN}\n").as_bytes()),
        ],
        "both",
    );
    let bin_id = blob_at(&repo, "main", "blob.bin");

    let (plan, _) = run(
        backend,
        &repo,
        &recipe(vec![replace_literal(TOKEN, "REDACTED")]),
    );

    assert_eq!(blob_at(&repo, "main", "blob.bin"), bin_id);
    assert_eq!(git_text(&repo, &["show", "main:t.txt"]), "x REDACTED");
    assert_eq!(plan.counts.blobs_skipped_binary, 1);
    assert_eq!(plan.counts.blobs_changed, 1);
    fsck(&repo);
}

/// A secret literal is replaced byte for byte (even in a file that is not
/// UTF-8), never shows up in the plan, the stored plan or the blob diff, and
/// the diff hides the removed lines.
pub fn check_rewrite_secret_literals_are_replaced_and_never_stored(backend: &dyn GitBackend) {
    const SECRET: &str = "EXAMPLE_SECRET_456";
    let repo = start();
    let mut legacy = b"\xff\xfe pw=".to_vec();
    legacy.extend_from_slice(SECRET.as_bytes());
    legacy.extend_from_slice(b" \xff\n");
    put(
        &repo,
        &[
            ("legacy.txt", &legacy),
            (
                "conf.txt",
                format!("a\nb\npassword: {SECRET}\nc\n").as_bytes(),
            ),
        ],
        "add config",
    );
    let old_head = git_text(&repo, &["rev-parse", "main"]);
    let r = recipe(vec![replace_secret(SECRET)]);
    let plan = preview(backend, &repo, &r);

    assert!(!format!("{plan:?}").contains(SECRET));
    assert!(!serde_json::to_string(&plan).unwrap().contains(SECRET));
    let dir = repo.root().join(".git/gitpoulpi/rewrite").join(&plan.run.0);
    for entry in std::fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        if path.file_name().is_some_and(|n| n == "objects.pack") {
            continue;
        }
        let bytes = std::fs::read(&path).unwrap();
        assert!(
            !String::from_utf8_lossy(&bytes).contains(SECRET),
            "{} holds the secret",
            path.display()
        );
    }

    let diff = backend
        .rewrite_blob_diff(&repo.repo_ref(), &plan.run, oid(&old_head), "conf.txt")
        .expect("blob diff");
    assert!(diff.redacted);
    assert_eq!(diff.kind, BlobChangeKind::Modified);
    let text = serde_json::to_string(&diff).unwrap();
    assert!(!text.contains(SECRET), "{text}");
    assert!(diff
        .hunks
        .iter()
        .flat_map(|h| &h.lines)
        .any(|l| l.kind == BlobLineKind::Added && l.text.contains("***REMOVED***")));

    backend
        .rewrite_apply(&repo.repo_ref(), &plan)
        .expect("apply");
    assert_eq!(
        git_bytes(&repo, &["show", "main:legacy.txt"], &[], b""),
        b"\xff\xfe pw=***REMOVED*** \xff\n"
    );
    let log = git_bytes(&repo, &["log", "-p", "--all", "main"], &[], b"");
    assert!(!String::from_utf8_lossy(&log).contains(SECRET));
    fsck(&repo);
}

/// A dirty file the rewrite would change is refused and nothing changes; a
/// dirty file it leaves alone keeps its local edit.
pub fn check_rewrite_refuses_dirty_paths_the_rewrite_would_change(backend: &dyn GitBackend) {
    let repo = start();
    put(
        &repo,
        &[
            ("a.txt", format!("key = {TOKEN}\n").as_bytes()),
            ("other.txt", b"other\n"),
        ],
        "add files",
    );
    std::fs::write(repo.root().join("a.txt"), format!("key = {TOKEN}\nmine\n")).unwrap();
    let before = snapshot(&repo);
    let r = recipe(vec![replace_literal(TOKEN, "REDACTED")]);
    refused(
        try_preview(backend, &repo, &r),
        RewriteRefusal::DirtyPathsTouched,
    );
    assert_eq!(snapshot(&repo), before, "a refusal changes nothing");
    assert_eq!(
        std::fs::read_to_string(repo.root().join("a.txt")).unwrap(),
        format!("key = {TOKEN}\nmine\n")
    );

    // Staged counts too.
    repo.git(&["add", "a.txt"]);
    refused(
        try_preview(backend, &repo, &r),
        RewriteRefusal::DirtyPathsTouched,
    );
    repo.git(&["reset", "-q", "a.txt"]);
    repo.git(&["checkout", "--", "a.txt"]);

    // A local edit to a file the rules leave alone survives the apply.
    std::fs::write(repo.root().join("other.txt"), "other\nmine\n").unwrap();
    run(backend, &repo, &r);
    assert_eq!(
        std::fs::read_to_string(repo.root().join("other.txt")).unwrap(),
        "other\nmine\n"
    );
    assert_eq!(
        std::fs::read_to_string(repo.root().join("a.txt")).unwrap(),
        "key = REDACTED\n"
    );
    assert_eq!(git_text(&repo, &["status", "--porcelain"]), "M other.txt");

    // Removing a file the user deleted in the working tree is also refused.
    let repo = start();
    put(&repo, &[("gone.txt", b"g\n"), ("k.txt", b"k\n")], "files");
    std::fs::remove_file(repo.root().join("gone.txt")).unwrap();
    refused(
        try_preview(backend, &repo, &recipe(vec![remove_paths(&["gone.txt"])])),
        RewriteRefusal::DirtyPathsTouched,
    );
}

/// A content run that would change a branch checked out in a linked worktree
/// is refused (that worktree's files cannot be updated); a branch whose
/// content the rules do not touch still moves.
pub fn check_rewrite_content_run_refuses_a_branch_checked_out_elsewhere(backend: &dyn GitBackend) {
    let repo = start();
    put(
        &repo,
        &[("a.txt", format!("key = {TOKEN}\n").as_bytes())],
        "add a",
    );
    let linked = repo.root().with_file_name("linked-content");
    repo.git(&[
        "worktree",
        "add",
        "-b",
        "side",
        &linked.to_string_lossy(),
        "main",
    ]);
    let mut r = recipe(vec![replace_literal(TOKEN, "REDACTED")]);
    r.scope = Scope::AllLocal;
    let before = snapshot(&repo);
    refused(
        try_preview(backend, &repo, &r),
        RewriteRefusal::CheckedOutElsewhere,
    );
    assert_eq!(snapshot(&repo), before);

    // Rules that match nothing there leave the linked branch alone.
    let mut other = recipe(vec![replace_literal("NO_SUCH_TEXT_ANYWHERE", "x")]);
    other.scope = Scope::AllLocal;
    let plan = preview(backend, &repo, &other);
    assert_eq!(plan.counts.commits_changed, 0);
}

/// The blob diff of a previewed commit: the old and the new file, a removed
/// file, an untouched one and a run that is gone.
pub fn check_rewrite_blob_diff_reads_the_preview(backend: &dyn GitBackend) {
    let repo = start();
    put(
        &repo,
        &[
            (
                "a.txt",
                format!("one\ntwo\nkey = {TOKEN}\nthree\n").as_bytes(),
            ),
            ("other.txt", b"other\n"),
            ("secrets/k.txt", b"k\n"),
        ],
        "add files",
    );
    let commit = oid(&git_text(&repo, &["rev-parse", "main"]));
    let r = recipe(vec![
        replace_literal(TOKEN, "REDACTED"),
        remove_paths(&["secrets/"]),
    ]);
    let plan = preview(backend, &repo, &r);
    let repo_ref = repo.repo_ref();

    let diff = backend
        .rewrite_blob_diff(&repo_ref, &plan.run, commit, "a.txt")
        .expect("diff of a.txt");
    assert_eq!(diff.kind, BlobChangeKind::Modified);
    assert!(!diff.binary && !diff.redacted);
    let lines: Vec<_> = diff.hunks.iter().flat_map(|h| &h.lines).collect();
    assert!(lines
        .iter()
        .any(|l| l.kind == BlobLineKind::Removed && l.text == format!("key = {TOKEN}")));
    assert!(lines
        .iter()
        .any(|l| l.kind == BlobLineKind::Added && l.text == "key = REDACTED"));
    assert_eq!(
        diff.new_size as usize,
        "one\ntwo\nkey = REDACTED\nthree\n".len()
    );

    let removed = backend
        .rewrite_blob_diff(&repo_ref, &plan.run, commit, "secrets/k.txt")
        .expect("diff of a removed file");
    assert_eq!(removed.kind, BlobChangeKind::Removed);
    let same = backend
        .rewrite_blob_diff(&repo_ref, &plan.run, commit, "other.txt")
        .expect("diff of an untouched file");
    assert_eq!(same.kind, BlobChangeKind::Unchanged);
    assert!(same.hunks.is_empty());

    // Nothing was written by asking.
    backend.rewrite_apply(&repo_ref, &plan).expect("apply");
    refused(
        backend.rewrite_blob_diff(&repo_ref, &plan.run, commit, "a.txt"),
        RewriteRefusal::Stale,
    );
}

/// A linear history of `count` commits on `main` for the content engine's
/// budgets: every commit adds two text files (each with the fake token
/// `EXAMPLE_TOKEN_123`, so there are `2 * count` unique text blobs) and
/// rewrites one key file under `secrets/`; 20 branches and 20 tags sit along
/// it. Built with one `git fast-import`.
pub fn import_content_history(repo: &FixtureRepo, count: usize) {
    use std::fmt::Write as _;
    let mut stream = String::with_capacity(count * 420);
    let put = |stream: &mut String, path: &str, content: &str| {
        let _ = write!(
            stream,
            "M 100644 inline {path}\ndata {}\n{content}\n",
            content.len()
        );
    };
    for i in 0..count {
        let when = 1_700_000_000 + (i as i64) * 600;
        let message = format!("change {i}\n");
        let _ = write!(
            stream,
            "commit refs/heads/main\nmark :{}\nauthor Alice Example <alice@example.com> {when} +0000\ncommitter Casey Example <casey@example.com> {when} +0000\ndata {}\n{message}",
            i + 1,
            message.len()
        );
        if i == 0 {
            stream.push_str("from refs/heads/main^0\n");
        }
        put(
            &mut stream,
            &format!("src/d{}/f{}.txt", i % 20, i % 50),
            &format!("line {i}\nsecret = EXAMPLE_TOKEN_123\nrest {i}\n"),
        );
        put(
            &mut stream,
            &format!("docs/n{}.md", i % 7),
            &format!("note {i}\ntoken: EXAMPLE_TOKEN_123 ({i})\n"),
        );
        put(
            &mut stream,
            &format!("secrets/key{}.pem", i % 5),
            &format!("key {i}\n"),
        );
    }
    for k in 0..20 {
        let at = 1 + (count.saturating_sub(1) * k) / 20;
        let _ = write!(
            stream,
            "reset refs/heads/side{k}\nfrom :{at}\n\nreset refs/tags/t{k}\nfrom :{at}\n\n"
        );
    }
    stream.push_str("done\n");
    git_bytes(
        repo,
        &["fast-import", "--quiet", "--done"],
        &[],
        stream.as_bytes(),
    );
    repo.git(&["reset", "--hard", "-q", "main"]);
}
