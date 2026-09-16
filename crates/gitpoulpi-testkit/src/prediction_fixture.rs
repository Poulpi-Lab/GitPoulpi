//! The fixture of the conflict prevention tests (daemon, CLI, PTY).

use crate::{FixtureRepo, TestRepoBuilder};

/// What `a.txt` says on `origin/main` (the conflicting side).
pub const ORIGIN_LINE: &str = "two from origin";
/// What `a.txt` says on `feature` before it is fixed.
pub const FEATURE_LINE: &str = "two from feature";

/// `feature` is checked out and edits the second line of `a.txt`; `origin/main`
/// edits the same line, so merging either into the other conflicts on exactly
/// `a.txt`. There is no local `main`: `origin/main` is the only base candidate.
/// `b.txt` is edited on `feature` only (clean).
pub fn conflicting_feature() -> FixtureRepo {
    let repo = TestRepoBuilder::new()
        .with_origin()
        .commit_file("a.txt", "one\ntwo\nthree\n", "initial")
        .build();
    repo.git(&["push", "-q", "origin", "main"]);
    repo.git(&["checkout", "-q", "-b", "feature"]);
    std::fs::write(
        repo.root().join("a.txt"),
        format!("one\n{FEATURE_LINE}\nthree\n"),
    )
    .unwrap();
    std::fs::write(repo.root().join("b.txt"), "only on feature\n").unwrap();
    repo.git(&["add", "-A"]);
    repo.git(&["commit", "-q", "-m", "feature work"]);
    repo.git(&["checkout", "-q", "main"]);
    std::fs::write(
        repo.root().join("a.txt"),
        format!("one\n{ORIGIN_LINE}\nthree\n"),
    )
    .unwrap();
    repo.git(&["commit", "-q", "-am", "main work"]);
    repo.git(&["push", "-q", "origin", "main"]);
    repo.git(&["checkout", "-q", "feature"]);
    repo.git(&["branch", "-q", "-D", "main"]);
    repo
}

/// Fixes the conflict on `feature` by taking `origin/main`'s line.
pub fn fix_the_conflict(repo: &FixtureRepo) {
    std::fs::write(
        repo.root().join("a.txt"),
        format!("one\n{ORIGIN_LINE}\nthree\n"),
    )
    .unwrap();
    repo.git(&["commit", "-q", "-am", "take origin's line"]);
}
