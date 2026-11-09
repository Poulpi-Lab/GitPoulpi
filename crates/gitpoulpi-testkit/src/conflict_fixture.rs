//! Repositories stopped on a conflict, one per kind of conflict the merge
//! editor has to understand (plan `merge-conflict-editor.md`, section 4).
//!
//! Every fixture is built by plain `git` (never by a backend: verifying a
//! backend with itself passes when two implementations are wrong the same
//! way), on `main` (ours, `HEAD`) with a `feature` branch (theirs), and
//! ends with the operation stopped on its conflicts.

use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

use gitpoulpi_core::domain::{LfsPointer, RepoRef};

use crate::fixture::isolate_from_user_config;
use crate::{FixtureRepo, FixtureRepoBuilder};

/// Which operation a fixture stopped.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FixtureOperation {
    /// `git merge feature` on `main`.
    Merge,
    /// `git cherry-pick feature` on `main`.
    CherryPick,
    /// `git revert <the feature commit>` on `main`, which already carries a
    /// later change to the same lines.
    Revert,
    /// `git rebase main` on `feature`: ours is `main`, theirs the replayed
    /// feature commit.
    Rebase,
}

/// A repository with a stopped operation and its conflicted paths.
pub struct ConflictFixture {
    repo: FixtureRepo,
    paths: Vec<String>,
    operation: FixtureOperation,
}

impl ConflictFixture {
    pub fn repo(&self) -> &FixtureRepo {
        &self.repo
    }

    /// The repository itself, for a harness that owns one.
    pub fn into_repo(self) -> FixtureRepo {
        self.repo
    }

    pub fn repo_ref(&self) -> RepoRef {
        self.repo.repo_ref()
    }

    pub fn root(&self) -> &Path {
        self.repo.root()
    }

    /// The paths the operation left unmerged, in the order git lists them.
    pub fn paths(&self) -> &[String] {
        &self.paths
    }

    /// The only unmerged path of a single-file fixture.
    pub fn path(&self) -> &str {
        &self.paths[0]
    }

    pub fn operation(&self) -> FixtureOperation {
        self.operation
    }

    /// `git` in the fixture, asserting success.
    pub fn git(&self, args: &[&str]) -> String {
        self.repo.git(args)
    }

    /// The blob id git holds at `stage` (1 base, 2 ours, 3 theirs) of
    /// `path`, or `None` when that stage does not exist.
    pub fn stage_oid(&self, stage: u8, path: &str) -> Option<String> {
        git_raw(
            self.root(),
            &["rev-parse", "--verify", "-q", &format!(":{stage}:{path}")],
        )
        .map(|s| s.trim().to_string())
    }

    /// The bytes git holds at `stage` of `path`.
    pub fn stage_bytes(&self, stage: u8, path: &str) -> Option<Vec<u8>> {
        let mut cmd = Command::new("git");
        cmd.args(["cat-file", "blob", &format!(":{stage}:{path}")])
            .current_dir(self.root());
        isolate_from_user_config(&mut cmd);
        let out = cmd.output().ok()?;
        out.status.success().then_some(out.stdout)
    }

    /// The file git wrote into the working tree (conflict markers and all).
    pub fn worktree_bytes(&self, path: &str) -> Vec<u8> {
        std::fs::read(self.root().join(path)).expect("the conflicted file is in the worktree")
    }
}

// ---- plumbing ---------------------------------------------------------------

fn command(root: &Path, args: &[&str]) -> Command {
    let mut cmd = Command::new("git");
    cmd.args(args).current_dir(root);
    isolate_from_user_config(&mut cmd);
    cmd
}

/// stdout on success, `None` on a non-zero exit.
fn git_raw(root: &Path, args: &[&str]) -> Option<String> {
    let out = command(root, args).output().expect("spawn git");
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).into_owned())
}

fn git_ok(root: &Path, args: &[&str]) -> String {
    let out = command(root, args).output().expect("spawn git");
    assert!(
        out.status.success(),
        "git {} failed: {}{}",
        args.join(" "),
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// Runs a command that is expected to stop on a conflict (non-zero exit).
fn git_conflicts(root: &Path, args: &[&str]) {
    let out = command(root, args).output().expect("spawn git");
    assert!(
        !out.status.success(),
        "git {} was meant to stop on conflicts but succeeded: {}",
        args.join(" "),
        String::from_utf8_lossy(&out.stdout)
    );
}

fn write(root: &Path, path: &str, bytes: &[u8]) {
    let full = root.join(path);
    if let Some(parent) = full.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(full, bytes).unwrap();
}

fn remove(root: &Path, path: &str) {
    std::fs::remove_file(root.join(path)).unwrap();
}

/// Commits everything in the worktree (`stage_all`), or just what the
/// phase staged itself (a gitlink or symlink put in the index by plumbing
/// has no file for `add -A` to find, and `add -A` would stage its removal).
fn commit_all(root: &Path, message: &str, stage_all: bool) {
    if stage_all {
        git_ok(root, &["add", "-A"]);
    }
    git_ok(root, &["commit", "-q", "-m", message]);
}

/// Stores `bytes` as a blob and returns its id.
fn hash_object(root: &Path, bytes: &[u8]) -> String {
    let mut child = command(root, &["hash-object", "-w", "--stdin"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(bytes).unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success());
    String::from_utf8(out.stdout).unwrap().trim().to_string()
}

/// A symlink (or a plain file with the target as content on a platform
/// without symlinks) staged without going through the filesystem.
fn stage_symlink(root: &Path, path: &str, target: &str) {
    let oid = hash_object(root, target.as_bytes());
    git_ok(
        root,
        &[
            "update-index",
            "--add",
            "--cacheinfo",
            &format!("120000,{oid},{path}"),
        ],
    );
}

fn stage_gitlink(root: &Path, path: &str, commit: &str) {
    git_ok(
        root,
        &[
            "update-index",
            "--add",
            "--cacheinfo",
            &format!("160000,{commit},{path}"),
        ],
    );
}

type Phase<'a> = &'a dyn Fn(&Path);

/// base on `main`, `theirs` on `feature`, `ours` on `main`, then `op`.
fn scenario(
    op: FixtureOperation,
    stage_all: bool,
    paths: &[&str],
    base: Phase<'_>,
    ours: Phase<'_>,
    theirs: Phase<'_>,
) -> ConflictFixture {
    let repo = FixtureRepoBuilder::new().unwrap().build();
    let root = repo.root().to_path_buf();
    let root = root.as_path();

    base(root);
    commit_all(root, "base", stage_all);
    match op {
        FixtureOperation::Merge | FixtureOperation::CherryPick | FixtureOperation::Rebase => {
            git_ok(root, &["checkout", "-q", "-b", "feature"]);
            theirs(root);
            commit_all(root, "feature change", stage_all);
            git_ok(root, &["checkout", "-q", "main"]);
            ours(root);
            commit_all(root, "main change", stage_all);
            match op {
                FixtureOperation::Merge => git_conflicts(root, &["merge", "feature"]),
                FixtureOperation::CherryPick => git_conflicts(root, &["cherry-pick", "feature"]),
                _ => {
                    git_ok(root, &["checkout", "-q", "feature"]);
                    git_conflicts(root, &["rebase", "main"]);
                }
            }
        }
        FixtureOperation::Revert => {
            // The change to undo, then a later one over the same lines.
            theirs(root);
            commit_all(root, "feature change", stage_all);
            ours(root);
            commit_all(root, "main change", stage_all);
            git_conflicts(root, &["revert", "--no-edit", "HEAD~1"]);
        }
    }
    ConflictFixture {
        repo,
        paths: paths.iter().map(|p| p.to_string()).collect(),
        operation: op,
    }
}

fn numbered(prefix: &str, n: usize) -> String {
    (1..=n).map(|i| format!("{prefix}{i}\n")).collect()
}

/// `n` lines `l1..ln` with the lines at `edits` (1-based) replaced.
fn lines_with(n: usize, edits: &[(usize, &str)]) -> String {
    (1..=n)
        .map(|i| {
            let text = edits
                .iter()
                .find(|(at, _)| *at == i)
                .map_or_else(|| format!("l{i}"), |(_, t)| t.to_string());
            format!("{text}\n")
        })
        .collect()
}

impl ConflictFixture {
    /// One text file, one conflict at line 3, plus a change only theirs
    /// made (line 8) that git merges by itself.
    pub fn text() -> Self {
        Self::text_with(FixtureOperation::Merge)
    }

    /// [`ConflictFixture::text`] stopped by `op`.
    pub fn text_with(op: FixtureOperation) -> Self {
        scenario(
            op,
            true,
            &["notes.txt"],
            &|r| write(r, "notes.txt", lines_with(10, &[]).as_bytes()),
            &|r| {
                write(
                    r,
                    "notes.txt",
                    lines_with(10, &[(3, "three (ours)")]).as_bytes(),
                )
            },
            &|r| {
                write(
                    r,
                    "notes.txt",
                    lines_with(10, &[(3, "three (theirs)"), (8, "eight (theirs only)")]).as_bytes(),
                )
            },
        )
    }

    /// [`ConflictFixture::text`] with a big conflict at line 3: each side is
    /// about `side_bytes` of 60-column lines (`first_ours_line` leads ours),
    /// for the AI input size limit.
    pub fn large_text(side_bytes: usize, first_ours_line: &str) -> Self {
        let block = |who: &str, lead: &str| -> String {
            let mut text = String::from(lead);
            let mut n = 0;
            while text.len() < side_bytes {
                n += 1;
                text.push_str(&format!("{who} filler line {n:05} {}\n", "x".repeat(38)));
            }
            text.pop();
            text
        };
        let ours = block("ours", &format!("{first_ours_line}\n"));
        let theirs = block("theirs", "");
        scenario(
            FixtureOperation::Merge,
            true,
            &["notes.txt"],
            &|r| write(r, "notes.txt", lines_with(10, &[]).as_bytes()),
            &|r| write(r, "notes.txt", lines_with(10, &[(3, &ours)]).as_bytes()),
            &|r| write(r, "notes.txt", lines_with(10, &[(3, &theirs)]).as_bytes()),
        )
    }

    /// [`ConflictFixture::text`] plus `side.txt`, changed only by theirs:
    /// the merge brings it in cleanly (staged), so aborting must put the
    /// old content back - which needs that file's index entry to be
    /// up to date with the working tree.
    pub fn text_with_clean_side_file() -> Self {
        scenario(
            FixtureOperation::Merge,
            true,
            &["notes.txt"],
            &|r| {
                write(r, "notes.txt", lines_with(10, &[]).as_bytes());
                write(r, "side.txt", b"side one\nside two\n");
            },
            &|r| {
                write(
                    r,
                    "notes.txt",
                    lines_with(10, &[(3, "three (ours)")]).as_bytes(),
                )
            },
            &|r| {
                write(
                    r,
                    "notes.txt",
                    lines_with(10, &[(3, "three (theirs)")]).as_bytes(),
                );
                write(r, "side.txt", b"side one\nside two (theirs)\n");
            },
        )
    }

    /// Three files (one in a folder), several conflicts in the first.
    /// Paths come back sorted.
    pub fn multi_conflict() -> Self {
        let base = |r: &Path| {
            write(r, "a.txt", lines_with(30, &[]).as_bytes());
            write(r, "b.txt", lines_with(10, &[]).as_bytes());
            write(r, "docs/c.txt", lines_with(10, &[]).as_bytes());
        };
        let ours = |r: &Path| {
            write(
                r,
                "a.txt",
                lines_with(
                    30,
                    &[
                        (3, "A3 ours"),
                        (4, "A4 ours"),
                        (15, "A15 ours"),
                        (28, "A28 ours"),
                    ],
                )
                .as_bytes(),
            );
            write(r, "b.txt", lines_with(10, &[(5, "B5 ours")]).as_bytes());
            write(
                r,
                "docs/c.txt",
                lines_with(10, &[(2, "C2 ours")]).as_bytes(),
            );
        };
        let theirs = |r: &Path| {
            write(
                r,
                "a.txt",
                lines_with(
                    30,
                    &[
                        (3, "A3 theirs"),
                        (4, "A4 theirs"),
                        (15, "A15 theirs"),
                        (28, "A28 theirs"),
                    ],
                )
                .as_bytes(),
            );
            write(r, "b.txt", lines_with(10, &[(5, "B5 theirs")]).as_bytes());
            write(
                r,
                "docs/c.txt",
                lines_with(10, &[(2, "C2 theirs")]).as_bytes(),
            );
        };
        scenario(
            FixtureOperation::Merge,
            true,
            &["a.txt", "b.txt", "docs/c.txt"],
            &base,
            &ours,
            &theirs,
        )
    }

    /// A Markdown file whose own text contains `=======`, `>>>>>>>` and
    /// `<<<<<<<` lines inside the conflicting region.
    pub fn marker_lookalike() -> Self {
        scenario(
            FixtureOperation::Merge,
            true,
            &["README.md"],
            &|r| write(r, "README.md", b"Title\n=======\n\nbody line\nend\n"),
            &|r| {
                write(
                    r,
                    "README.md",
                    b"Title\n=======\n\nours body\n=======\n<<<<<<< not a marker\nend\n",
                )
            },
            &|r| {
                write(
                    r,
                    "README.md",
                    b"Title\n=======\n\ntheirs body\n>>>>>>> not a marker\n=========\nend\n",
                )
            },
        )
    }

    /// Both sides added `new.txt` with different content: no base.
    pub fn add_add() -> Self {
        scenario(
            FixtureOperation::Merge,
            true,
            &["new.txt"],
            &|r| write(r, "keep.txt", b"keep\n"),
            &|r| write(r, "new.txt", b"ours first\nours second\n"),
            &|r| write(r, "new.txt", b"theirs first\ntheirs second\ntheirs third\n"),
        )
    }

    /// One side deleted `gone.txt`, the other modified it.
    pub fn delete_modify(deleted_by_us: bool) -> Self {
        let delete = |r: &Path| remove(r, "gone.txt");
        let modify = |r: &Path| write(r, "gone.txt", lines_with(6, &[(2, "changed")]).as_bytes());
        let (ours, theirs): (Phase<'_>, Phase<'_>) = if deleted_by_us {
            (&delete, &modify)
        } else {
            (&modify, &delete)
        };
        scenario(
            FixtureOperation::Merge,
            true,
            &["gone.txt"],
            &|r| write(r, "gone.txt", lines_with(6, &[]).as_bytes()),
            ours,
            theirs,
        )
    }

    /// A binary file (it has a NUL byte) changed on both sides.
    pub fn binary() -> Self {
        scenario(
            FixtureOperation::Merge,
            true,
            &["image.bin"],
            &|r| write(r, "image.bin", b"\x89PNG\0base\0\x01"),
            &|r| write(r, "image.bin", b"\x89PNG\0ours\0\x02\x03"),
            &|r| write(r, "image.bin", b"\x89PNG\0theirs\0\x04"),
        )
    }

    /// A real picture, `pic.png`, changed on both sides: solid green in the
    /// base, solid red in ours (stage 2, A) and solid blue in theirs
    /// (stage 3, B), 8x8 each.
    pub fn image() -> Self {
        let solid = |c: [u8; 3]| {
            let rgba: Vec<u8> = (0..64).flat_map(|_| [c[0], c[1], c[2], 255]).collect();
            crate::png_bytes(8, 8, &rgba)
        };
        scenario(
            FixtureOperation::Merge,
            true,
            &["pic.png"],
            &|r| write(r, "pic.png", &solid([0, 255, 0])),
            &|r| write(r, "pic.png", &solid([255, 0, 0])),
            &|r| write(r, "pic.png", &solid([0, 0, 255])),
        )
    }

    /// A Git LFS pointer file pointing at different objects on each side.
    pub fn lfs_pointer() -> Self {
        let pointer = |tag: char, size: u64| {
            LfsPointer {
                oid: tag.to_string().repeat(64),
                size,
            }
            .to_text()
        };
        scenario(
            FixtureOperation::Merge,
            true,
            &["big.dat"],
            &|r| write(r, "big.dat", pointer('a', 100).as_bytes()),
            &|r| write(r, "big.dat", pointer('b', 200).as_bytes()),
            &|r| write(r, "big.dat", pointer('c', 300).as_bytes()),
        )
    }

    /// A CRLF text file (autocrlf off, so the bytes in the blobs are CRLF)
    /// with one conflict.
    pub fn crlf() -> Self {
        let crlf = |text: String| text.replace('\n', "\r\n");
        scenario(
            FixtureOperation::Merge,
            true,
            &["crlf.txt"],
            &|r| write(r, "crlf.txt", crlf(lines_with(8, &[])).as_bytes()),
            &|r| {
                write(
                    r,
                    "crlf.txt",
                    crlf(lines_with(8, &[(4, "four ours")])).as_bytes(),
                )
            },
            &|r| {
                write(
                    r,
                    "crlf.txt",
                    crlf(lines_with(8, &[(4, "four theirs")])).as_bytes(),
                )
            },
        )
    }

    /// [`ConflictFixture::crlf`] with a UTF-8 BOM on every side.
    pub fn crlf_bom() -> Self {
        let file = |edit: &[(usize, &str)]| {
            let mut bytes = b"\xEF\xBB\xBF".to_vec();
            bytes.extend(lines_with(8, edit).replace('\n', "\r\n").into_bytes());
            bytes
        };
        scenario(
            FixtureOperation::Merge,
            true,
            &["bom.txt"],
            &|r| write(r, "bom.txt", &file(&[])),
            &|r| write(r, "bom.txt", &file(&[(4, "four ours")])),
            &|r| write(r, "bom.txt", &file(&[(4, "four theirs")])),
        )
    }

    /// A Latin-1 file (`0xE9` is not valid UTF-8) with one conflict.
    pub fn latin1() -> Self {
        let file = |mid: &[u8]| {
            let mut bytes = b"caf\xE9\nl2\nl3\n".to_vec();
            bytes.extend_from_slice(mid);
            bytes.extend_from_slice(b"\nl5\nl6\nl7\n");
            bytes
        };
        scenario(
            FixtureOperation::Merge,
            true,
            &["latin1.txt"],
            &|r| write(r, "latin1.txt", &file(b"l4")),
            &|r| write(r, "latin1.txt", &file(b"ours \xE9")),
            &|r| write(r, "latin1.txt", &file(b"theirs \xE8")),
        )
    }

    /// A symlink retargeted differently on each side.
    pub fn symlink() -> Self {
        scenario(
            FixtureOperation::Merge,
            false,
            &["link"],
            &|r| stage_symlink(r, "link", "target-a"),
            &|r| stage_symlink(r, "link", "target-ours"),
            &|r| stage_symlink(r, "link", "target-theirs"),
        )
    }

    /// Both sides added `run.sh` with the same content, one executable.
    pub fn mode_only() -> Self {
        scenario(
            FixtureOperation::Merge,
            true,
            &["run.sh"],
            &|r| write(r, "keep.txt", b"keep\n"),
            &|r| write(r, "run.sh", b"#!/bin/sh\necho hi\n"),
            &|r| {
                write(r, "run.sh", b"#!/bin/sh\necho hi\n");
                // Executable on disk too, or the tree would look dirty
                // and block the checkout back to `main`.
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let path = r.join("run.sh");
                    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
                }
                git_ok(r, &["add", "run.sh"]);
                git_ok(r, &["update-index", "--chmod=+x", "run.sh"]);
            },
        )
    }

    /// A submodule pointer moved to a different commit on each side. The
    /// commits do not exist in this repository (a pointer needs no
    /// checkout), which is also how a submodule that was never initialized
    /// looks.
    pub fn submodule() -> Self {
        let a = "1111111111111111111111111111111111111111";
        let b = "2222222222222222222222222222222222222222";
        let c = "3333333333333333333333333333333333333333";
        scenario(
            FixtureOperation::Merge,
            false,
            &["sub"],
            &|r| stage_gitlink(r, "sub", a),
            &|r| stage_gitlink(r, "sub", c),
            &|r| stage_gitlink(r, "sub", b),
        )
    }

    /// `a.txt` renamed to `b.txt` on one side and to `c.txt` on the other:
    /// git leaves `a.txt` (base only), `b.txt` (ours only) and `c.txt`
    /// (theirs only).
    pub fn rename_rename() -> Self {
        let body = lines_with(8, &[]);
        scenario(
            FixtureOperation::Merge,
            true,
            &["a.txt", "b.txt", "c.txt"],
            &|r| write(r, "a.txt", body.as_bytes()),
            &|r| {
                git_ok(r, &["mv", "a.txt", "b.txt"]);
            },
            &|r| {
                git_ok(r, &["mv", "a.txt", "c.txt"]);
            },
        )
    }

    /// `a.txt` renamed to `b.txt` on our side and deleted on theirs: git
    /// leaves `b.txt` with a base and our stage.
    pub fn rename_delete() -> Self {
        let body = lines_with(8, &[]);
        scenario(
            FixtureOperation::Merge,
            true,
            &["b.txt"],
            &|r| write(r, "a.txt", body.as_bytes()),
            &|r| {
                git_ok(r, &["mv", "a.txt", "b.txt"]);
            },
            &|r| remove(r, "a.txt"),
        )
    }

    /// A text file just over the 8 MiB cap, changed on the same line on
    /// both sides.
    pub fn huge() -> Self {
        let file = |edit: &str| {
            let mut text = String::new();
            for i in 0..100_000 {
                if i == 50_000 {
                    text.push_str(edit);
                    text.push('\n');
                } else {
                    text.push_str(&format!("line {i:06} {:-<78}\n", ""));
                }
            }
            text
        };
        assert!(file("x").len() as u64 > gitpoulpi_core::domain::MAX_TEXT_BLOB_BYTES);
        scenario(
            FixtureOperation::Merge,
            true,
            &["huge.txt"],
            &|r| write(r, "huge.txt", file("middle").as_bytes()),
            &|r| write(r, "huge.txt", file("middle ours").as_bytes()),
            &|r| write(r, "huge.txt", file("middle theirs").as_bytes()),
        )
    }

    /// A repository with a commit and nothing unmerged.
    pub fn none_unmerged() -> FixtureRepo {
        let repo = FixtureRepoBuilder::new().unwrap().build();
        let root = repo.root().to_path_buf();
        write(&root, "f.txt", numbered("l", 3).as_bytes());
        commit_all(&root, "base", true);
        repo
    }
}
