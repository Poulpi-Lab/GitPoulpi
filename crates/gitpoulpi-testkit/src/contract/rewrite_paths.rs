//! History rewrite, slice S20 (the history-rewrite design):
//! the path rules (rename, move the whole repository into a subdirectory,
//! extract a subdirectory) and the three extraction modes. Expected values
//! come from plain `git` (`rev-parse <commit>:<path>`, `log -- <path>`),
//! never from the engine under test.

use std::ops::ControlFlow;
use std::path::{Path, PathBuf};

use gitpoulpi_core::domain::rewrite::{
    ExtractMode, PathMove, Recipe, RewriteOutcome, RewritePlan, RewriteRefusal, Rule, RuleKind,
    Scope, SignaturePolicy,
};
use gitpoulpi_core::domain::Oid;
use gitpoulpi_core::GitBackend;

use super::rewrite::{git_text, history, object_files, refused, run, run_dirs};
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

fn recipe(moves: Vec<PathMove>, scope: Scope) -> Recipe {
    let mut r = Recipe::new(
        "paths",
        scope,
        moves
            .into_iter()
            .map(|m| Rule::new(RuleKind::MovePaths(m)))
            .collect(),
    );
    r.options.signatures = Some(SignaturePolicy::Strip);
    r
}

fn rename(from: &str, to: &str) -> PathMove {
    PathMove::Rename {
        from: from.into(),
        to: to.into(),
    }
}

fn extract(dir: &str, mode: ExtractMode) -> PathMove {
    PathMove::ExtractSubdir {
        dir: dir.into(),
        mode: Some(mode),
    }
}

fn try_preview(
    backend: &dyn GitBackend,
    repo: &FixtureRepo,
    recipe: &Recipe,
) -> Result<RewritePlan, gitpoulpi_core::Error> {
    backend.rewrite_preview(&repo.repo_ref(), recipe, &mut |_| ControlFlow::Continue(()))
}

fn oid(hex: &str) -> Oid {
    Oid::from_hex(hex).expect("a 40 character id")
}

fn fsck(repo: &FixtureRepo) {
    git_text(repo, &["fsck", "--connectivity-only", "--no-dangling"]);
}

fn at(repo: &FixtureRepo, rev: &str, path: &str) -> String {
    git_text(repo, &["rev-parse", &format!("{rev}:{path}")])
}

fn subjects(repo: &FixtureRepo, extra: &[&str]) -> Vec<String> {
    let mut args = vec!["log", "--reverse", "--topo-order", "--format=%s", "HEAD"];
    args.extend_from_slice(extra);
    git_text(repo, &args).lines().map(str::to_string).collect()
}

/// `git` in another directory (a clone), trimmed.
fn git_in(repo: &FixtureRepo, dir: &Path, args: &[&str]) -> String {
    repo.git_at(dir, args)
}

/// The commits, oldest first, with their lib folder: "initial" has none.
///
/// `c0 initial`, `c1 add lib` (lib/a.txt, lib/sub/b.txt), `c2 other work`,
/// `c3 edit lib`, `c4 more other`, `c5 add lib file` (lib/c.txt).
fn lib_repo() -> FixtureRepo {
    let repo = start();
    put(
        &repo,
        &[("lib/a.txt", b"a1\n"), ("lib/sub/b.txt", b"b\n")],
        "add lib",
    );
    put(&repo, &[("other/x.txt", b"x\n")], "other work");
    put(&repo, &[("lib/a.txt", b"a2\n")], "edit lib");
    put(&repo, &[("other/y.txt", b"y\n")], "more other");
    put(&repo, &[("lib/c.txt", b"c\n")], "add lib file");
    repo
}

/// Every file of the repository with its bytes, `.git` included: how "the
/// original stays byte for byte as it was" is told.
fn everything(root: &Path) -> Vec<(String, Vec<u8>)> {
    fn walk(base: &Path, dir: &Path, out: &mut Vec<(String, Vec<u8>)>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                walk(base, &path, out);
            } else {
                let rel = path
                    .strip_prefix(base)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/");
                out.push((rel, std::fs::read(&path).unwrap()));
            }
        }
    }
    let mut out = Vec::new();
    walk(root, root, &mut out);
    out.sort();
    out
}

/// Renaming a folder (and a file) moves the very same tree and blob objects:
/// every blob keeps its id, the renamed folder's tree id is the old one, the
/// history keeps its shape and the working tree follows.
pub fn check_rewrite_renames_a_folder_across_history(backend: &dyn GitBackend) {
    let repo = start();
    put(
        &repo,
        &[("docs/a.md", b"a\n"), ("docs/deep/d.md", b"d\n")],
        "add docs",
    );
    put(&repo, &[("src/main.rs", b"fn main() {}\n")], "add source");
    put(&repo, &[("docs/b.md", b"b\n"), ("old.txt", b"o\n")], "more");
    let before = history(&repo);
    let old_trees: Vec<Option<String>> = before
        .iter()
        .map(|c| repo.try_git(&["rev-parse", &format!("{c}:docs")]))
        .collect();
    let main_rs = at(&repo, "main", "src/main.rs");
    let old_blob = at(&repo, "main", "old.txt");

    let (plan, outcome) = run(
        backend,
        &repo,
        &recipe(
            vec![rename("docs", "manual/guide"), rename("old.txt", "new.txt")],
            Scope::default(),
        ),
    );
    assert_eq!(plan.counts.commits_walked, before.len() as u64);
    assert_eq!(outcome.commits_changed, 3);

    let after = history(&repo);
    assert_eq!(after.len(), before.len(), "no commit is dropped");
    assert_eq!(
        after[0], before[0],
        "a commit with nothing to move keeps its id"
    );
    assert_eq!(
        subjects(&repo, &[]),
        ["initial", "add docs", "add source", "more"]
    );
    for (i, id) in after.iter().enumerate().skip(1) {
        assert_eq!(
            Some(at(&repo, id, "manual/guide")),
            old_trees[i],
            "commit {i}: the renamed folder is the same tree object"
        );
        assert!(
            repo.try_git(&["rev-parse", &format!("{id}:docs")])
                .is_none(),
            "commit {i}: the old path is gone"
        );
    }
    assert_eq!(at(&repo, "main", "src/main.rs"), main_rs);
    assert_eq!(at(&repo, "main", "new.txt"), old_blob);
    assert!(!repo.root().join("docs").exists());
    assert_eq!(
        std::fs::read_to_string(repo.root().join("manual/guide/deep/d.md")).unwrap(),
        "d\n"
    );
    assert_eq!(git_text(&repo, &["status", "--porcelain"]), "");
    fsck(&repo);
}

/// "Move the repository into a subdirectory": every path gets the prefix and
/// the folder at the prefix is the old root tree, commit by commit.
pub fn check_rewrite_moves_the_whole_repository_into_a_subdirectory(backend: &dyn GitBackend) {
    let repo = lib_repo();
    let before = history(&repo);
    let old_roots: Vec<String> = before
        .iter()
        .map(|c| git_text(&repo, &["rev-parse", &format!("{c}^{{tree}}")]))
        .collect();
    let blob = at(&repo, "main", "lib/a.txt");

    let (_, outcome) = run(
        backend,
        &repo,
        &recipe(
            vec![PathMove::IntoSubdir {
                dir: "projects/app".into(),
            }],
            Scope::default(),
        ),
    );
    assert_eq!(outcome.commits_changed, before.len() as u64);
    let after = history(&repo);
    assert_eq!(after.len(), before.len());
    for (i, id) in after.iter().enumerate() {
        assert_eq!(
            at(&repo, id, "projects/app"),
            old_roots[i],
            "commit {i}: the old root is the folder"
        );
        let top = git_text(&repo, &["ls-tree", "--name-only", id]);
        assert_eq!(top, "projects", "commit {i}: nothing else at the root");
    }
    assert_eq!(at(&repo, "main", "projects/app/lib/a.txt"), blob);
    assert_eq!(
        std::fs::read_to_string(repo.root().join("projects/app/lib/c.txt")).unwrap(),
        "c\n"
    );
    assert!(!repo.root().join("lib").exists());
    assert_eq!(git_text(&repo, &["status", "--porcelain"]), "");
    fsck(&repo);
}

/// A rename never overwrites: where the target exists (a folder, a file, a
/// file where a folder is needed) in some commit the preview is refused with
/// the typed reason naming the commit and the path, and nothing changes.
pub fn check_rewrite_a_rename_never_overwrites_and_names_the_collision(backend: &dyn GitBackend) {
    let repo = start();
    put(&repo, &[("docs/a.md", b"a\n")], "add docs");
    put(&repo, &[("guide/g.md", b"g\n")], "add guide");
    let before = (
        git_text(&repo, &["for-each-ref"]),
        object_files(&repo),
        run_dirs(&repo),
    );
    let clash = history(&repo)[2].clone();

    let result = try_preview(
        backend,
        &repo,
        &recipe(vec![rename("docs", "guide")], Scope::default()),
    );
    match &result {
        Err(gitpoulpi_core::Error::RewriteRefused(RewriteRefusal::PathCollision, detail)) => {
            assert!(detail.contains(&clash[..7]), "names the commit: {detail}");
            assert!(detail.contains("guide"), "names the path: {detail}");
        }
        other => panic!("expected PathCollision, got {other:?}"),
    }
    // A file where the target needs a folder.
    refused(
        try_preview(
            backend,
            &repo,
            &recipe(vec![rename("docs", "guide/g.md/inside")], Scope::default()),
        ),
        RewriteRefusal::PathCollision,
    );
    assert_eq!(
        before,
        (
            git_text(&repo, &["for-each-ref"]),
            object_files(&repo),
            run_dirs(&repo)
        ),
        "a refused preview leaves nothing"
    );

    // Where the target does not exist in any commit, the rename goes.
    let ok = try_preview(
        backend,
        &repo,
        &recipe(vec![rename("docs", "manual")], Scope::default()),
    )
    .expect("no collision");
    assert_eq!(ok.counts.commits_changed, 2);
    backend.rewrite_discard(&repo.repo_ref(), &ok.run).unwrap();
}

/// Extracting a subdirectory in place: only the commits that touch it are
/// left, the folder is the root, branches and tags that have nothing of it
/// are deleted (`RefChange.new = None`) and the rest follow.
pub fn check_rewrite_extracts_a_subdirectory_and_prunes_the_rest(backend: &dyn GitBackend) {
    let repo = lib_repo();
    repo.git(&["branch", "no-lib", "main~5"]);
    repo.git(&["tag", "early", "main~5"]);
    repo.git(&["tag", "mid", "main~3"]);
    repo.git(&["tag", "-a", "-m", "release", "annotated", "main"]);
    let touching: Vec<String> = git_text(&repo, &["rev-list", "--reverse", "main", "--", "lib"])
        .lines()
        .map(str::to_string)
        .collect();
    let expected_trees: Vec<String> = touching.iter().map(|c| at(&repo, c, "lib")).collect();
    let expected_subjects = subjects(&repo, &["--", "lib"]);
    assert_eq!(expected_subjects, ["add lib", "edit lib", "add lib file"]);
    let sub_blob = at(&repo, "main", "lib/sub/b.txt");

    let scope = Scope::AllLocal;
    let (plan, outcome) = run(
        backend,
        &repo,
        &recipe(vec![extract("lib", ExtractMode::InPlace)], scope),
    );

    assert_eq!(subjects(&repo, &[]), expected_subjects);
    let trees: Vec<String> = history(&repo)
        .iter()
        .map(|c| git_text(&repo, &["rev-parse", &format!("{c}^{{tree}}")]))
        .collect();
    assert_eq!(
        trees, expected_trees,
        "each kept commit holds exactly the folder"
    );
    assert_eq!(
        git_text(&repo, &["ls-tree", "-r", "--name-only", "HEAD"]),
        "a.txt\nc.txt\nsub/b.txt"
    );
    assert_eq!(
        at(&repo, "HEAD", "sub/b.txt"),
        sub_blob,
        "blobs keep their ids"
    );
    assert_eq!(
        git_text(&repo, &["rev-list", "--max-parents=0", "HEAD"])
            .lines()
            .count(),
        1,
        "the first commit that has the folder is the root"
    );
    // Refs.
    assert!(
        repo.try_git(&["rev-parse", "--verify", "refs/heads/no-lib"])
            .is_none(),
        "a branch with nothing of the folder is deleted"
    );
    assert!(repo
        .try_git(&["rev-parse", "--verify", "refs/tags/early"])
        .is_none());
    let gone = plan
        .refs
        .iter()
        .find(|r| r.name.ends_with("no-lib"))
        .expect("the plan lists the branch");
    assert_eq!(gone.new, None);
    assert!(outcome
        .moved
        .iter()
        .any(|r| r.name.ends_with("no-lib") && r.new.is_none()));
    // A lightweight tag on a commit that was pruned follows the commit it
    // stood on; the annotated tag points at the new tip.
    assert_eq!(
        git_text(&repo, &["rev-parse", "refs/tags/mid^{commit}"]),
        history(&repo)[0],
        "the tag on pruned 'other work' sits on 'add lib' now"
    );
    assert_eq!(
        git_text(&repo, &["rev-parse", "refs/tags/annotated^{commit}"]),
        git_text(&repo, &["rev-parse", "HEAD"])
    );
    // The working tree is the folder.
    assert!(!repo.root().join("lib").exists());
    assert!(!repo.root().join("other").exists());
    assert_eq!(
        std::fs::read_to_string(repo.root().join("a.txt")).unwrap(),
        "a2\n"
    );
    assert_eq!(git_text(&repo, &["status", "--porcelain"]), "");
    fsck(&repo);

    // No branch is left without the folder: the checked-out one cannot go.
    let bare = start();
    refused(
        try_preview(
            backend,
            &bare,
            &recipe(vec![extract("lib", ExtractMode::InPlace)], Scope::default()),
        ),
        RewriteRefusal::EmptyScope,
    );
}

/// Merges are simplified: a side branch that never touched the folder
/// vanishes with its commits, one that did stays and the merge keeps both
/// parents.
pub fn check_rewrite_extract_simplifies_merges(backend: &dyn GitBackend) {
    let repo = start();
    put(&repo, &[("lib/a.txt", b"a\n")], "add lib");
    repo.git(&["checkout", "-q", "-b", "side-other"]);
    put(&repo, &[("other/o.txt", b"o\n")], "side other");
    repo.git(&["checkout", "-q", "main"]);
    repo.git(&["checkout", "-q", "-b", "side-lib"]);
    put(&repo, &[("lib/s.txt", b"s\n")], "side lib");
    repo.git(&["checkout", "-q", "main"]);
    put(&repo, &[("lib/m.txt", b"m\n")], "main lib");
    repo.git(&["merge", "-q", "--no-ff", "-m", "merge lib", "side-lib"]);
    repo.git(&["merge", "-q", "--no-ff", "-m", "merge other", "side-other"]);
    let expected = subjects(&repo, &["--", "lib"]);
    assert!(expected.contains(&"merge lib".to_string()));

    run(
        backend,
        &repo,
        &recipe(vec![extract("lib", ExtractMode::InPlace)], Scope::default()),
    );
    let after = subjects(&repo, &[]);
    assert!(!after.contains(&"side other".to_string()));
    assert!(
        !after.contains(&"merge other".to_string()),
        "a merge that brings nothing of the folder is dropped"
    );
    assert_eq!(
        after.iter().filter(|s| expected.contains(s)).count(),
        expected.len(),
        "{after:?} vs {expected:?}"
    );
    let merge = git_text(&repo, &["rev-list", "--merges", "--max-count=1", "HEAD"]);
    assert_eq!(
        git_text(&repo, &["rev-list", "--parents", "-n", "1", &merge])
            .split(' ')
            .count(),
        3,
        "the merge of the side branch that touched the folder has both parents"
    );
    assert_eq!(
        git_text(&repo, &["ls-tree", "-r", "--name-only", "HEAD"]),
        "a.txt\nm.txt\ns.txt"
    );
    fsck(&repo);
}

/// Extract into a new clone: the new repository holds only that history (the
/// old objects are gone from it, it has no remote) and the original is byte
/// for byte what it was, run directory and all.
pub fn check_rewrite_extract_into_a_clone_leaves_the_original_untouched(backend: &dyn GitBackend) {
    let repo = lib_repo();
    repo.git(&["branch", "no-lib", "main~5"]);
    repo.git(&["branch", "feature", "main~2"]);
    let original = everything(repo.root());
    let old_tip = repo.git(&["rev-parse", "main"]);
    let touching = subjects(&repo, &["--", "lib"]);
    let out = tempfile::tempdir().unwrap();
    let target = out.path().join("extracted");

    let plan = backend
        .rewrite_preview(
            &repo.repo_ref(),
            &recipe(vec![extract("lib", ExtractMode::Clone)], Scope::default()),
            &mut |_| ControlFlow::Continue(()),
        )
        .expect("preview");
    let outcome = backend
        .rewrite_extract(&repo.repo_ref(), &plan, &target)
        .expect("extract");
    assert!(outcome.moved.is_empty(), "no ref of the original moves");

    assert_eq!(
        everything(repo.root()),
        original,
        "the original is byte for byte untouched"
    );
    assert_eq!(repo.git(&["rev-parse", "main"]), old_tip);

    // The clone.
    assert_eq!(
        git_in(&repo, &target, &["log", "--reverse", "--format=%s", "main"])
            .lines()
            .collect::<Vec<_>>(),
        touching
    );
    assert_eq!(
        git_in(&repo, &target, &["ls-tree", "-r", "--name-only", "HEAD"]),
        "a.txt\nc.txt\nsub/b.txt"
    );
    assert_eq!(
        git_in(
            &repo,
            &target,
            &["for-each-ref", "--format=%(refname)", "refs/heads"]
        ),
        "refs/heads/feature\nrefs/heads/main",
        "the branch with nothing of the folder is not in the clone"
    );
    assert_eq!(
        git_in(&repo, &target, &["remote"]),
        "",
        "the clone is not tied to the original"
    );
    assert!(
        std::process::Command::new("git")
            .args(["cat-file", "-e", &old_tip])
            .current_dir(&target)
            .status()
            .map(|s| !s.success())
            .unwrap_or(false),
        "the old history is not in the new repository"
    );
    assert_eq!(git_in(&repo, &target, &["status", "--porcelain"]), "");
    assert_eq!(
        std::fs::read_to_string(target.join("a.txt")).unwrap(),
        "a2\n"
    );
    git_in(
        &repo,
        &target,
        &["fsck", "--connectivity-only", "--no-dangling"],
    );
    // Nothing is left to apply: the preview's run directory went too.
    assert!(run_dirs(&repo).is_empty());

    // A target that is not empty is refused, and nothing is written.
    let plan = backend
        .rewrite_preview(
            &repo.repo_ref(),
            &recipe(vec![extract("lib", ExtractMode::Clone)], Scope::default()),
            &mut |_| ControlFlow::Continue(()),
        )
        .expect("preview");
    refused(
        backend.rewrite_extract(&repo.repo_ref(), &plan, &target),
        RewriteRefusal::InvalidRule,
    );
    backend
        .rewrite_discard(&repo.repo_ref(), &plan.run)
        .unwrap();
}

/// Extract as a submodule: the clone is made as in clone mode, the original
/// keeps all its history and gains one commit that replaces the folder by a
/// gitlink with a `.gitmodules`; `git submodule status` works, the folder
/// is the checked-out submodule, and moving the branch back (what undo does)
/// brings the plain folder back.
pub fn check_rewrite_extract_as_a_submodule_adds_one_commit_and_undoes(backend: &dyn GitBackend) {
    let repo = lib_repo();
    let old_tip = repo.git(&["rev-parse", "main"]);
    let old_count = repo.git(&["rev-list", "--count", "main"]);
    let out = tempfile::tempdir().unwrap();
    let target = out.path().join("lib-repo");

    let preview = |r: &FixtureRepo| {
        backend
            .rewrite_preview(
                &r.repo_ref(),
                &recipe(
                    vec![extract("lib", ExtractMode::Submodule)],
                    Scope::default(),
                ),
                &mut |_| ControlFlow::Continue(()),
            )
            .expect("preview")
    };

    // Local edits in the folder would be lost: refused, nothing created.
    std::fs::write(repo.root().join("lib/a.txt"), "edited\n").unwrap();
    let plan = preview(&repo);
    refused(
        backend.rewrite_extract(&repo.repo_ref(), &plan, &target),
        RewriteRefusal::DirtyPathsTouched,
    );
    assert!(!target.exists(), "nothing is created before the refusal");
    repo.git(&["checkout", "--", "lib/a.txt"]);
    backend
        .rewrite_discard(&repo.repo_ref(), &plan.run)
        .unwrap();

    let plan = preview(&repo);
    let outcome: RewriteOutcome = backend
        .rewrite_extract(&repo.repo_ref(), &plan, &target)
        .expect("extract");
    assert_eq!(
        outcome.moved.len(),
        1,
        "the current branch gets the new commit"
    );
    assert_eq!(outcome.moved[0].old, oid(&old_tip));
    let new_tip = outcome.moved[0].new.expect("a new tip").to_hex();

    // The original: history kept, one commit more.
    assert_eq!(repo.git(&["rev-parse", "main"]), new_tip);
    assert_eq!(repo.git(&["rev-parse", "main^"]), old_tip);
    assert_eq!(
        repo.git(&["rev-list", "--count", "main"]),
        (old_count.parse::<u32>().unwrap() + 1).to_string()
    );
    // The gitlink and `.gitmodules`.
    let clone_head = git_in(&repo, &target, &["rev-parse", "HEAD"]);
    assert_eq!(
        repo.git(&["ls-tree", "main", "lib"])
            .split('\t')
            .next()
            .unwrap(),
        format!("160000 commit {clone_head}")
    );
    assert_eq!(
        repo.git(&["config", "-f", ".gitmodules", "submodule.lib.path"]),
        "lib"
    );
    let url = repo.git(&["config", "-f", ".gitmodules", "submodule.lib.url"]);
    assert_eq!(
        std::fs::canonicalize(&url).unwrap(),
        std::fs::canonicalize(&target).unwrap(),
        "the url is the local path of the new repository"
    );
    let status = repo.git(&["submodule", "status"]);
    assert!(
        status.trim_start().starts_with(&clone_head) && status.contains(" lib"),
        "{status}"
    );
    assert_eq!(repo.git(&["status", "--porcelain"]), "");
    assert_eq!(
        std::fs::read_to_string(repo.root().join("lib/a.txt")).unwrap(),
        "a2\n",
        "the folder is the checked-out submodule"
    );
    // The new repository holds only the folder's history.
    assert_eq!(
        git_in(&repo, &target, &["log", "--reverse", "--format=%s"])
            .lines()
            .collect::<Vec<_>>(),
        ["add lib", "edit lib", "add lib file"]
    );
    assert!(run_dirs(&repo).is_empty());

    // Undo: the branch goes back and the folder is a plain folder again.
    backend
        .move_head_keep(&repo.repo_ref(), "main", oid(&new_tip), oid(&old_tip))
        .expect("move back");
    assert_eq!(repo.git(&["rev-parse", "main"]), old_tip);
    assert_eq!(repo.git(&["status", "--porcelain"]), "");
    assert_eq!(repo.git(&["submodule", "status"]), "");
    assert_eq!(
        repo.git(&["ls-files", "lib"]).lines().collect::<Vec<_>>(),
        ["lib/a.txt", "lib/c.txt", "lib/sub/b.txt"]
    );
    assert_eq!(
        std::fs::read_to_string(repo.root().join("lib/a.txt")).unwrap(),
        "a2\n"
    );
    assert!(!repo.root().join("lib/.git").exists());
    let _: PathBuf = target;
}
