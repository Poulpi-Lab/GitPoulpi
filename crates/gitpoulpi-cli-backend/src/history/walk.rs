//! Reading side of the engine: what HEAD is, which commits the scope walks,
//! and their raw bytes from one long-lived `git cat-file --batch`.
//!
//! Every read runs with `GIT_NO_REPLACE_OBJECTS=1`: a `refs/replace/*` must
//! never leak a replaced object into the new history.

use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Stdio};

use gitpoulpi_core::domain::rewrite::RewriteRefusal;
use gitpoulpi_core::domain::Oid;
use gitpoulpi_core::{Error, Result};

use crate::process::{failed_with, git, run_bytes, spawn_failed};

/// A refusal as the error the backend returns.
pub(crate) fn refuse(refusal: RewriteRefusal, detail: &str) -> Error {
    Error::RewriteRefused(refusal, detail.to_string())
}

/// `git` for a read of the object store: replace refs are ignored.
pub(crate) fn read_git(root: &Path, args: &[&str]) -> std::process::Command {
    let mut cmd = git(root, args);
    cmd.env("GIT_NO_REPLACE_OBJECTS", "1");
    cmd
}

/// What HEAD names.
#[derive(Clone)]
pub(crate) enum Head {
    /// `refs/heads/<branch>` (possibly not born yet).
    Branch(String),
    Detached(Oid),
}

/// Where the repository is, read from the file system (no spawn): the git
/// dir of this worktree, the common one, HEAD, and the refusals that need no
/// history.
pub(crate) struct Probe {
    pub(crate) git_dir: PathBuf,
    pub(crate) common_dir: PathBuf,
    pub(crate) head: Head,
    /// A promisor remote: file content may be fetched lazily.
    pub(crate) partial_clone: bool,
}

fn sha256_refusal() -> Error {
    refuse(
        RewriteRefusal::UnsupportedObjectFormat,
        "only SHA-1 repositories can be rewritten",
    )
}

/// The git dir of the worktree at `root`: a `.git` directory, the target of a
/// `.git` file (linked worktree, submodule), the root itself (bare), else
/// what git says.
fn locate_git_dir(root: &Path) -> Result<PathBuf> {
    let dotgit = root.join(".git");
    if dotgit.is_dir() {
        return Ok(dotgit);
    }
    if let Ok(text) = std::fs::read_to_string(&dotgit) {
        if let Some(target) = text.lines().find_map(|l| l.strip_prefix("gitdir:")) {
            let target = Path::new(target.trim());
            let dir = if target.is_absolute() {
                target.to_path_buf()
            } else {
                root.join(target)
            };
            if dir.is_dir() {
                return Ok(dir);
            }
        }
    }
    if root.join("HEAD").is_file() && root.join("objects").is_dir() {
        return Ok(root.to_path_buf());
    }
    let out = crate::process::run(root, &["rev-parse", "--absolute-git-dir"])?;
    Ok(PathBuf::from(out.trim()))
}

fn common_dir_of(git_dir: &Path) -> PathBuf {
    match std::fs::read_to_string(git_dir.join("commondir")) {
        Ok(text) => {
            let target = Path::new(text.trim());
            if target.is_absolute() {
                target.to_path_buf()
            } else {
                git_dir.join(target)
            }
        }
        Err(_) => git_dir.to_path_buf(),
    }
}

fn read_head(root: &Path, git_dir: &Path) -> Result<Head> {
    let text = std::fs::read_to_string(git_dir.join("HEAD")).unwrap_or_default();
    let text = text.trim();
    // A reftable repository keeps a stub in the file: ask git.
    if text.is_empty() || text == "ref: refs/heads/.invalid" {
        if let Ok(name) = crate::process::run(root, &["symbolic-ref", "-q", "HEAD"]) {
            return Ok(Head::Branch(name.trim().to_string()));
        }
        let id = crate::process::run(root, &["rev-parse", "HEAD"])
            .map_err(|_| refuse(RewriteRefusal::EmptyScope, "the branch has no commit"))?;
        return Oid::from_hex(id.trim())
            .map(Head::Detached)
            .ok_or_else(sha256_refusal);
    }
    if let Some(name) = text.strip_prefix("ref:") {
        return Ok(Head::Branch(name.trim().to_string()));
    }
    Oid::from_hex(text)
        .map(Head::Detached)
        .ok_or_else(sha256_refusal)
}

/// What HEAD names, with none of `probe`'s refusals: a read-only scan works
/// in the middle of a merge or in a shallow clone too.
pub(crate) fn head_of(root: &Path) -> Result<Head> {
    let git_dir = locate_git_dir(root)?;
    read_head(root, &git_dir)
}

/// The refusals that need no history: SHA-256, an operation in progress, a
/// shallow clone.
pub(crate) fn probe(root: &Path) -> Result<Probe> {
    let git_dir = locate_git_dir(root)?;
    let common_dir = common_dir_of(&git_dir);
    let config = std::fs::read_to_string(common_dir.join("config"))
        .unwrap_or_default()
        .to_ascii_lowercase();
    // `key = value` lines with the blanks removed.
    let config_has = |pair: &str| {
        config
            .lines()
            .any(|l| l.split_whitespace().collect::<String>() == pair)
    };
    if config_has("objectformat=sha256") {
        return Err(sha256_refusal());
    }
    if crate::rewrite::operation_in_progress(&git_dir.to_string_lossy()) {
        return Err(refuse(
            RewriteRefusal::OperationInProgress,
            "a merge, rebase, cherry-pick, revert or bisect is in progress",
        ));
    }
    if common_dir.join("shallow").is_file() {
        return Err(refuse(
            RewriteRefusal::Shallow,
            "a shallow clone has incomplete history",
        ));
    }
    let head = read_head(root, &git_dir)?;
    let partial_clone = config.contains("partialclone") || config_has("promisor=true");
    Ok(Probe {
        git_dir,
        common_dir,
        head,
        partial_clone,
    })
}

/// `git rev-list --topo-order --reverse <tips> --not <not>`: the commits,
/// parents first. `tips` are ref names, ids or rev-list options such as
/// `--branches`.
pub(crate) fn rev_list(root: &Path, tips: &[String], not: &[String]) -> Result<Vec<Oid>> {
    let mut args = vec!["rev-list", "--topo-order", "--reverse"];
    args.extend(tips.iter().map(String::as_str));
    if !not.is_empty() {
        args.push("--not");
        args.extend(not.iter().map(String::as_str));
    }
    let out = read_git(root, &args).output().map_err(spawn_failed)?;
    if !out.status.success() {
        return Err(failed_with(
            &args,
            out.status.code(),
            &String::from_utf8_lossy(&out.stderr),
        ));
    }
    parse_oids(&out.stdout)
}

pub(crate) fn parse_oids(bytes: &[u8]) -> Result<Vec<Oid>> {
    String::from_utf8_lossy(bytes)
        .lines()
        .map(|l| {
            Oid::from_hex(l.trim())
                .ok_or_else(|| Error::Backend(format!("git printed `{l}`, not an object id")))
        })
        .collect()
}

/// One long-lived `git cat-file --batch`: commits are read pipelined, other
/// objects (tags, notes trees) one request at a time. Bytes, never text: a
/// commit may be in any encoding.
pub(crate) struct Reader {
    child: Child,
    stdin: ChildStdin,
    out: BufReader<ChildStdout>,
}

impl Reader {
    pub(crate) fn new(root: &Path) -> Result<Self> {
        let mut child = read_git(root, &["cat-file", "--batch"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(spawn_failed)?;
        let stdin = child.stdin.take().expect("stdin was piped");
        let out = BufReader::with_capacity(1 << 16, child.stdout.take().expect("piped"));
        Ok(Self { child, stdin, out })
    }

    /// The reply to one request: the kind and the body.
    fn reply(out: &mut BufReader<ChildStdout>, oid: &Oid) -> Result<(String, Vec<u8>)> {
        let mut line = String::new();
        out.read_line(&mut line)
            .map_err(|e| Error::Backend(format!("cat-file: {e}")))?;
        let mut fields = line.split_whitespace();
        let (_, kind, size) = (fields.next(), fields.next(), fields.next());
        let (Some(kind), Some(size)) = (kind, size.and_then(|s| s.parse::<usize>().ok())) else {
            return Err(Error::Backend(format!(
                "{} is not readable: {}",
                oid.to_hex(),
                line.trim_end()
            )));
        };
        let mut body = vec![0u8; size];
        out.read_exact(&mut body)
            .map_err(|e| Error::Backend(format!("cat-file cut an object short: {e}")))?;
        let mut newline = [0u8; 1];
        out.read_exact(&mut newline)
            .map_err(|e| Error::Backend(format!("cat-file: {e}")))?;
        Ok((kind.to_string(), body))
    }

    /// A blob's size and, when it is at most `limit` bytes, its content. A
    /// bigger one is read past and dropped, so a huge file never sits in memory.
    pub(crate) fn blob(&mut self, oid: Oid, limit: u64) -> Result<(u64, Option<Vec<u8>>)> {
        let io = |e: std::io::Error| Error::Backend(format!("cat-file: {e}"));
        self.stdin
            .write_all(format!("{}\n", oid.to_hex()).as_bytes())
            .map_err(io)?;
        self.stdin.flush().map_err(io)?;
        let mut line = String::new();
        self.out.read_line(&mut line).map_err(io)?;
        let mut fields = line.split_whitespace();
        let (_, kind, size) = (fields.next(), fields.next(), fields.next());
        let (Some("blob"), Some(size)) = (kind, size.and_then(|s| s.parse::<u64>().ok())) else {
            return Err(Error::Backend(format!(
                "{} is not a readable blob: {}",
                oid.to_hex(),
                line.trim_end()
            )));
        };
        if size > limit {
            std::io::copy(&mut (&mut self.out).take(size + 1), &mut std::io::sink()).map_err(io)?;
            return Ok((size, None));
        }
        let mut body = vec![0u8; size as usize];
        self.out.read_exact(&mut body).map_err(io)?;
        let mut newline = [0u8; 1];
        self.out.read_exact(&mut newline).map_err(io)?;
        Ok((size, Some(body)))
    }

    /// A blob streamed into the writer `want` returns once the size is known
    /// (`None`: the blob is read past and dropped). A file of any size passes
    /// through a small buffer, never memory.
    pub(crate) fn blob_copy<W: std::io::Write>(
        &mut self,
        oid: Oid,
        want: impl FnOnce(u64) -> Option<W>,
    ) -> Result<(u64, Option<W>)> {
        let io = |e: std::io::Error| Error::Backend(format!("cat-file: {e}"));
        self.stdin
            .write_all(format!("{}\n", oid.to_hex()).as_bytes())
            .map_err(io)?;
        self.stdin.flush().map_err(io)?;
        let mut line = String::new();
        self.out.read_line(&mut line).map_err(io)?;
        let mut fields = line.split_whitespace();
        let (_, kind, size) = (fields.next(), fields.next(), fields.next());
        let (Some("blob"), Some(size)) = (kind, size.and_then(|s| s.parse::<u64>().ok())) else {
            return Err(Error::Backend(format!(
                "{} is not a readable blob: {}",
                oid.to_hex(),
                line.trim_end()
            )));
        };
        let mut body = (&mut self.out).take(size + 1);
        let writer = match want(size) {
            Some(mut w) => {
                std::io::copy(&mut (&mut body).take(size), &mut w).map_err(io)?;
                Some(w)
            }
            None => {
                std::io::copy(&mut (&mut body).take(size), &mut std::io::sink()).map_err(io)?;
                None
            }
        };
        // The newline that ends the answer.
        std::io::copy(&mut body, &mut std::io::sink()).map_err(io)?;
        Ok((size, writer))
    }

    /// One object: its kind and body.
    pub(crate) fn get(&mut self, oid: Oid) -> Result<(String, Vec<u8>)> {
        let io = |e: std::io::Error| Error::Backend(format!("cat-file: {e}"));
        self.stdin
            .write_all(format!("{}\n", oid.to_hex()).as_bytes())
            .map_err(io)?;
        self.stdin.flush().map_err(io)?;
        Self::reply(&mut self.out, &oid)
    }

    /// The raw bytes of each commit in `oids`, in that order.
    pub(crate) fn commits(
        &mut self,
        oids: &[Oid],
        mut tick: impl FnMut(u64) -> Result<()>,
    ) -> Result<Vec<Vec<u8>>> {
        let Self { child, stdin, out } = self;
        let mut input = String::with_capacity(oids.len() * 41);
        for oid in oids {
            input.push_str(&oid.to_hex());
            input.push('\n');
        }
        let mut bodies = Vec::with_capacity(oids.len());
        // Written from its own thread: git answers while it is still reading.
        let result = std::thread::scope(|scope| {
            let writer = scope.spawn(|| {
                stdin
                    .write_all(input.as_bytes())
                    .and_then(|()| stdin.flush())
            });
            let result = (|| -> Result<()> {
                for (i, oid) in oids.iter().enumerate() {
                    let (kind, body) = Self::reply(out, oid)?;
                    if kind != "commit" {
                        return Err(Error::Backend(format!(
                            "{} is not a commit but a {kind}",
                            oid.to_hex()
                        )));
                    }
                    bodies.push(body);
                    if i % 256 == 0 {
                        tick(i as u64)?;
                    }
                }
                Ok(())
            })();
            if result.is_err() {
                let _ = child.kill();
            }
            let _ = writer.join();
            result
        });
        result?;
        Ok(bodies)
    }
}

impl Drop for Reader {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// `git var GIT_COMMITTER_IDENT`: the identity and the time "me, now" means.
pub(crate) fn committer_ident(root: &Path) -> Result<Vec<u8>> {
    let mut bytes = run_bytes(root, &["var", "GIT_COMMITTER_IDENT"])?;
    while bytes.last().is_some_and(|b| b.is_ascii_whitespace()) {
        bytes.pop();
    }
    Ok(bytes)
}
