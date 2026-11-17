use std::path::PathBuf;
use std::process::Command;

use gitpoulpi_core::domain::RepoRef;
use tempfile::TempDir;

use crate::{Error, Result};

/// A real temp git repo built by shelling to `git` (plain commits take a
/// libgit2 shortcut, see `add_and_commit`; the backends under test are never
/// used to build a fixture), used
/// by every backend's contract tests and by TUI golden tests. Kept alive
/// for the fixture's lifetime via the held TempDir.
pub struct FixtureRepo {
    _dir: TempDir,
    root: PathBuf,
    remote: Option<PathBuf>,
    extra_remotes: Vec<(String, PathBuf)>,
    submodule_sources: Vec<(String, PathBuf)>,
}

impl FixtureRepo {
    /// The repository `with_submodule(path)` cloned the submodule from.
    pub fn submodule_source(&self, path: &str) -> &std::path::Path {
        self.submodule_sources
            .iter()
            .find(|(p, _)| p == path)
            .map(|(_, source)| source.as_path())
            .expect("this fixture has no such submodule")
    }

    /// `git` in any directory, with the fixture's isolation - for the
    /// assertions and arrangements inside a submodule or a clone.
    pub fn git_at(&self, dir: &std::path::Path, args: &[&str]) -> String {
        git_stdout(dir, args)
    }

    pub fn root(&self) -> &std::path::Path {
        &self.root
    }

    /// Where GitPoulpi keeps this repository's own data with the default
    /// `session.per_repo_state_dir` (`in_git_dir`): the `gitpoulpi` folder
    /// of the repository's git directory.
    pub fn state_dir(&self) -> std::path::PathBuf {
        self.root.join(".git").join("gitpoulpi")
    }

    pub fn repo_ref(&self) -> RepoRef {
        RepoRef::from_root(&self.root)
    }

    /// The bare repository `FixtureRepoBuilder::with_origin` created, or
    /// `None` when the fixture has no remote. A *filesystem path*, not a
    /// URL: the suite must run with no credentials and no network.
    pub fn remote_path(&self) -> Option<&std::path::Path> {
        self.remote.as_deref()
    }

    /// Runs `git` inside this fixture's repository and returns its stdout,
    /// trimmed. For assertions no `GitBackend` method can make ("is this tag
    /// annotated", "what does the bare remote hold"); verifying a backend
    /// with itself would pass for two implementations wrong the same way.
    pub fn git(&self, args: &[&str]) -> String {
        git_stdout(&self.root, args)
    }

    /// Like [`Self::git`] but `None` instead of a panic when git fails. For
    /// polling a repo another process is rewriting: `git stash list` exits 128
    /// (`bad revision 'refs/stash'`) while a stash push/pop swaps `refs/stash`.
    pub fn try_git(&self, args: &[&str]) -> Option<String> {
        let mut cmd = Command::new("git");
        cmd.args(args).current_dir(&self.root);
        isolate_from_user_config(&mut cmd);
        let out = cmd.output().ok()?;
        out.status
            .success()
            .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
    }

    /// Commits on top of HEAD with exactly `message` as its message bytes
    /// (never converted - `commit-tree` would re-encode a message that is
    /// not UTF-8), optionally with an `encoding` header, and moves `main`
    /// there. For fixtures of commits written in a legacy encoding.
    pub fn commit_raw_message(&self, message: &[u8], encoding_header: Option<&str>) {
        let repo = self;
        use std::io::Write;
        use std::process::{Command, Stdio};
        let tree = repo.git(&["rev-parse", "HEAD^{tree}"]).trim().to_string();
        let parent = repo.git(&["rev-parse", "HEAD"]).trim().to_string();
        // The object is written by hand: `commit-tree` would convert a message
        // that is not UTF-8, and the point is to store exactly these bytes.
        let mut object = format!(
            "tree {tree}\nparent {parent}\nauthor Author <a@example.com> 1700000000 +0000\n\
         committer Committer <c@example.com> 1700000000 +0000\n"
        )
        .into_bytes();
        if let Some(enc) = encoding_header {
            object.extend_from_slice(format!("encoding {enc}\n").as_bytes());
        }
        object.push(b'\n');
        object.extend_from_slice(message);
        let mut child = Command::new("git")
            .current_dir(repo.root())
            .args([
                "hash-object",
                "-t",
                "commit",
                "-w",
                "--stdin",
                "--literally",
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        child.stdin.take().unwrap().write_all(&object).unwrap();
        let out = child.wait_with_output().unwrap();
        assert!(out.status.success());
        let id = String::from_utf8(out.stdout).unwrap().trim().to_string();
        repo.git(&["update-ref", "refs/heads/main", &id]);
    }

    /// The object ids stored in the origin's LFS store
    /// (`<origin>/lfs/objects/aa/bb/<oid>`), sorted. Read straight from the
    /// filesystem, so a push can be asserted without the backend under
    /// test.
    pub fn remote_lfs_objects(&self) -> Vec<String> {
        let Some(remote) = &self.remote else {
            return Vec::new();
        };
        let mut found = Vec::new();
        let mut stack = vec![remote.join("lfs").join("objects")];
        while let Some(dir) = stack.pop() {
            let Ok(entries) = std::fs::read_dir(&dir) else {
                continue;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                } else if let Some(name) = path.file_name() {
                    found.push(name.to_string_lossy().to_string());
                }
            }
        }
        found.sort();
        found
    }

    /// `git` inside the bare remote. Panics if the fixture has none.
    pub fn git_in_remote(&self, args: &[&str]) -> String {
        let remote = self
            .remote
            .as_ref()
            .expect("this fixture was not built with_origin()");
        git_stdout(remote, args)
    }

    /// `git` inside the bare repository behind the extra remote `name`
    /// (see `FixtureRepoBuilder::with_remote`). Panics if there is none.
    pub fn git_in_named_remote(&self, name: &str, args: &[&str]) -> String {
        let path = self
            .extra_remotes
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, p)| p)
            .unwrap_or_else(|| panic!("this fixture has no extra remote `{name}`"));
        git_stdout(path, args)
    }

    /// Plays **another user**: clones the bare `origin` into a scratch
    /// directory inside the fixture's temp dir, commits `message` on
    /// `branch` there and pushes it, so `origin` moves ahead of what this
    /// repository last saw. The fixture's own remote-tracking refs stay put
    /// until a backend fetches.
    pub fn push_from_elsewhere(&self, branch: &str, message: &str) {
        self.push_from_elsewhere_to("origin", branch, message);
    }

    /// [`Self::push_from_elsewhere`] for any remote of the fixture:
    /// `origin` (`with_origin`) or one made by `with_remote(name)`. The
    /// branch must already exist on that remote.
    pub fn push_from_elsewhere_to(&self, remote_name: &str, branch: &str, message: &str) {
        let remote = if remote_name == "origin" {
            self.remote.as_ref()
        } else {
            self.extra_remotes
                .iter()
                .find(|(n, _)| n == remote_name)
                .map(|(_, p)| p)
        }
        .unwrap_or_else(|| panic!("this fixture has no remote `{remote_name}`"));
        let parent = self._dir.path();
        let mut n = 0;
        let scratch = loop {
            let candidate = parent.join(format!("elsewhere-{n}"));
            if !candidate.exists() {
                break candidate;
            }
            n += 1;
        };
        let remote_str = remote.to_string_lossy().to_string();
        let scratch_str = scratch.to_string_lossy().to_string();
        git_stdout(
            parent,
            &["clone", "--branch", branch, &remote_str, &scratch_str],
        );
        let file = message_file_name(message);
        std::fs::write(scratch.join(&file), message).expect("write elsewhere file");
        git_stdout(&scratch, &["add", &file]);
        git_stdout(
            &scratch,
            &[
                "-c",
                "user.name=Someone Else",
                "-c",
                "user.email=else@gitpoulpi.local",
                "commit",
                "-m",
                message,
            ],
        );
        git_stdout(&scratch, &["push", "origin", branch]);
    }
}

/// The file a message-named fixture commit writes. Characters Windows
/// forbids in a file name (a message like `fix: crash` has a colon) become
/// `_`, on every OS, so one fixture behaves the same everywhere.
fn message_file_name(message: &str) -> String {
    let name: String = message
        .chars()
        .map(|c| match c {
            ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c => c,
        })
        .collect();
    format!("{name}.txt")
}

/// `file:///C:/dir/origin.git` (Windows) or `file:///tmp/origin.git`:
/// forward slashes, spaces as `%20`.
fn file_url(path: &std::path::Path) -> String {
    let p = path
        .to_string_lossy()
        .replace('\\', "/")
        .replace(' ', "%20");
    if p.starts_with('/') {
        format!("file://{p}")
    } else {
        format!("file:///{p}")
    }
}

fn git_stdout(cwd: &std::path::Path, args: &[&str]) -> String {
    let mut cmd = Command::new("git");
    cmd.args(args).current_dir(cwd);
    isolate_from_user_config(&mut cmd);
    let out = cmd.output().expect("failed to spawn git");
    assert!(
        out.status.success(),
        "git {} failed in {}: {}",
        args.join(" "),
        cwd.display(),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

/// What `git init --initial-branch=main [--bare] --template=<empty>` writes
/// (HEAD, config, the empty `objects/` and `refs/` directories), made once
/// per machine by one real `git init` and copied for every fixture instead of
/// spawning `git init` each time (a `CreateProcess` is ~100 ms on Windows;
/// the contract suite alone built ~1300 repositories). The real `git init`
/// is what decides the platform-dependent config it probes for (`filemode`,
/// `symlinks`, `ignorecase`), so the copy is the repository git would have
/// made in this temp directory.
///
/// Lives beside the fixtures in the temp directory and stays there (it is a
/// few small files, reused by later runs and by every test binary); two
/// processes racing to create it both succeed, the loser's copy is dropped.
fn init_skeleton(bare: bool) -> Result<PathBuf> {
    static CACHE: std::sync::OnceLock<[PathBuf; 2]> = std::sync::OnceLock::new();
    if let Some(paths) = CACHE.get() {
        return Ok(paths[usize::from(bare)].clone());
    }
    let made: Result<[PathBuf; 2]> = (|| {
        let base = std::env::temp_dir().join("gitpoulpi-testkit-init-v1");
        let one = |bare: bool| -> Result<PathBuf> {
            let final_dir = base.join(if bare { "bare" } else { "plain" });
            if final_dir.join("HEAD").is_file() {
                return Ok(final_dir);
            }
            std::fs::create_dir_all(&base)?;
            let scratch = tempfile::Builder::new()
                .prefix("skeleton-")
                .tempdir_in(&base)?;
            let empty_template = scratch.path().join("no-template");
            std::fs::create_dir(&empty_template)?;
            let target = scratch.path().join("skeleton");
            let mut cmd = Command::new("git");
            cmd.args(["init", "--initial-branch=main", "--quiet"]);
            if bare {
                cmd.arg("--bare");
            }
            cmd.arg(format!("--template={}", empty_template.display()))
                .arg(&target);
            isolate_from_user_config(&mut cmd);
            let out = cmd.output()?;
            if !out.status.success() {
                return Err(Error::GitFailed(format!(
                    "git init (skeleton) failed: {}",
                    String::from_utf8_lossy(&out.stderr)
                )));
            }
            // For a plain repository the skeleton is the `.git` directory.
            let skeleton = if bare { target } else { target.join(".git") };
            // Without a template git makes no `hooks/` and `info/`; tests
            // drop hooks into the first and read the second.
            std::fs::create_dir_all(skeleton.join("hooks"))?;
            std::fs::create_dir_all(skeleton.join("info"))?;
            if !place_skeleton(&skeleton, &final_dir) {
                return Err(Error::GitFailed(format!(
                    "cannot place the init skeleton at {}",
                    final_dir.display()
                )));
            }
            Ok(final_dir)
        };
        Ok([one(false)?, one(true)?])
    })();
    let paths = CACHE.get_or_init(|| made.unwrap_or_else(|e| panic!("init skeleton: {e}")));
    Ok(paths[usize::from(bare)].clone())
}

/// Moves a fresh skeleton to `final_dir`; true when a usable skeleton (with
/// `HEAD`) is there afterwards, made by this process or a concurrent one.
/// Windows can refuse to rename a directory whose files were just written
/// (an antivirus or indexer holds them for a moment): retry, then replace a
/// half-made leftover, then fall back to copying.
fn place_skeleton(skeleton: &std::path::Path, final_dir: &std::path::Path) -> bool {
    let ready = || final_dir.join("HEAD").is_file();
    for attempt in 0..10 {
        if std::fs::rename(skeleton, final_dir).is_ok() || ready() {
            return true;
        }
        if final_dir.exists() {
            // Left half-made by an interrupted run.
            let _ = std::fs::remove_dir_all(final_dir);
        }
        std::thread::sleep(std::time::Duration::from_millis(20 * (attempt + 1)));
    }
    if ready() {
        return true;
    }
    let staging = final_dir.with_extension(format!("copy-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&staging);
    copy_dir(skeleton, &staging).is_ok()
        && (std::fs::rename(&staging, final_dir).is_ok() || ready())
        || ready()
}

/// Recursive copy of `src` into `dst` (created), empty directories included.
fn copy_dir(src: &std::path::Path, dst: &std::path::Path) -> Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let to = dst.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir(&entry.path(), &to)?;
        } else {
            std::fs::copy(entry.path(), &to)?;
        }
    }
    Ok(())
}

/// The repo-local config every fixture gets, appended verbatim to the
/// `.git/config` `git init` writes. See `write_local_config`.
///
/// The newlines are escapes rather than real ones so the bytes do not vary
/// with the line endings this source file is checked out with (Windows with
/// `core.autocrlf`).
const FIXTURE_LOCAL_CONFIG: &str = concat!(
    "[user]\n",
    "\tname = GitPoulpi Testkit\n",
    "\temail = testkit@gitpoulpi.local\n",
    "[core]\n",
    "\tautocrlf = false\n",
    "\teol = lf\n",
);

/// Gives the backends under test the same config the fixtures' own commands
/// see, once per process: no system config and a global file holding just
/// what the tests need.
///
/// - Since git 2.38.1 `submodule update` refuses the `file` transport unless
///   config *outside the repository* allows it, and the backends
///   deliberately read the user's config.
/// - A repository the backends create (a clone, an extracted submodule) has
///   no fixture-local config, so it must not pick up the machine's
///   `core.autocrlf` (Git for Windows sets it to `true` system-wide) or its
///   LFS filters: the fixtures' commands, which ignore that config
///   (`isolate_from_user_config`), would then see CRLF files and LFS-smudged
///   files as modified.
///
/// Called by every `FixtureRepoBuilder`; the public name stays for tests
/// that name the submodule allowance.
pub fn allow_file_submodules() {
    // One shared file per temp dir, not a `NamedTempFile` per process: a
    // static is never dropped, so each test binary used to leave its copy
    // behind (tens of thousands in the system temp dir over a few gates).
    static GLOBAL: std::sync::OnceLock<std::path::PathBuf> = std::sync::OnceLock::new();
    const CONTENT: &str =
        "[protocol \"file\"]\n\tallow = always\n[core]\n\tautocrlf = false\n\teol = lf\n";
    let path = GLOBAL.get_or_init(|| {
        let path = std::env::temp_dir().join("gitpoulpi-testkit-gitconfig");
        if std::fs::read_to_string(&path).ok().as_deref() != Some(CONTENT) {
            // Written beside, then renamed: a concurrent test binary never
            // reads a half-written file.
            let tmp =
                tempfile::NamedTempFile::new_in(std::env::temp_dir()).expect("temp gitconfig");
            std::fs::write(tmp.path(), CONTENT).expect("write the temp gitconfig");
            if let Err(e) = tmp.persist(&path) {
                // Another binary won the race (Windows refuses to replace an
                // open file): fine as long as the content is the same.
                assert_eq!(
                    std::fs::read_to_string(&path).ok().as_deref(),
                    Some(CONTENT),
                    "shared test gitconfig: {e}"
                );
            }
        }
        // A test that points `GIT_CONFIG_GLOBAL` at its own file keeps it.
        if std::env::var_os("GIT_CONFIG_GLOBAL").is_none() {
            std::env::set_var("GIT_CONFIG_GLOBAL", &path);
        }
        std::env::set_var("GIT_CONFIG_NOSYSTEM", "1");
        path
    });
    debug_assert!(path.exists());
}

/// The date `FixtureRepoBuilder::fixed_dates` stamps on every commit.
const FIXED_DATE: &str = "2020-01-01T00:00:00+0000";

pub struct FixtureRepoBuilder {
    dir: TempDir,
    root: PathBuf,
    submodule_sources: Vec<(String, PathBuf)>,
    remote: Option<PathBuf>,
    extra_remotes: Vec<(String, PathBuf)>,
    commands: Vec<Vec<String>>,
    fixed_dates: bool,
    /// Set once a step makes git's own add/commit matter (`config`): the
    /// libgit2 shortcut of `add_and_commit` stays off for the rest.
    cli_commits: bool,
}

impl FixtureRepoBuilder {
    pub fn new() -> Result<Self> {
        Self::init(&["init", "--initial-branch=main"])
    }

    /// A fixture whose repository uses the reftable ref format
    /// (`git init --ref-format=reftable`, git 2.45 and later). An error when
    /// the installed git cannot make one: callers skip, with a note.
    pub fn new_reftable() -> Result<Self> {
        Self::init(&["init", "--initial-branch=main", "--ref-format=reftable"])
    }

    fn init(init_args: &[&str]) -> Result<Self> {
        allow_file_submodules();
        let dir = TempDir::new()?;
        // The repo lives in a *subdirectory* of the temp dir so `worktree()`
        // can put linked worktrees beside it: inside the TempDir (cleaned up
        // with it) but outside the repo (never untracked files in `status()`).
        let mut root = dir.path().join("repo");
        std::fs::create_dir_all(&root)?;
        // macOS temp dirs sit behind a symlink (`/var` -> `/private/var`) and
        // the application reports the real path, so tests comparing paths
        // as strings need the real one from the start. (Not on Windows,
        // where the real path carries a `\\?\` prefix.)
        if cfg!(target_os = "macos") {
            root = root.canonicalize()?;
        }
        let mut builder = Self {
            dir,
            root,
            submodule_sources: Vec::new(),
            remote: None,
            extra_remotes: Vec::new(),
            commands: Vec::new(),
            fixed_dates: false,
            cli_commits: false,
        };
        if init_args == ["init", "--initial-branch=main"] {
            builder
                .commands
                .push(init_args.iter().map(|s| s.to_string()).collect());
            copy_dir(&init_skeleton(false)?, &builder.root.join(".git"))?;
        } else {
            builder.run(init_args)?;
        }
        builder.write_local_config()?;
        Ok(builder)
    }

    /// Pins every commit's author and committer date to a constant, so two
    /// builders given the same steps produce the same commit oids (otherwise
    /// they differ whenever the builds straddle a second boundary). Opt-in
    /// and chainable right after `new()`: later steps use it.
    pub fn fixed_dates(mut self) -> Self {
        self.fixed_dates = true;
        self
    }

    /// Writes the fixture's repo-local config in one shot, by appending to
    /// the `.git/config` that `git init` just wrote, instead of spawning
    /// four `git config` processes (on Windows a `CreateProcess` is expensive
    /// enough that they dominated fixture setup). Repeated sections are
    /// valid git config; the unit test at the bottom of this file pins it
    /// by asking `git config --local --get` for each key.
    ///
    /// Line endings are pinned *repo-locally*, the only level that survives:
    /// `isolate_from_user_config` suppresses global and system config for the
    /// fixture's own commands, but the backends under test honour the user's
    /// config and Git for Windows ships `core.autocrlf = true` system-wide.
    /// A test that wants CRLF writes CRLF bytes itself; `autocrlf = false`
    /// stops git rewriting them.
    fn write_local_config(&mut self) -> Result<()> {
        use std::io::Write;

        // `git init` always creates `.git/config`, and a fixture's repo is
        // never a linked worktree, so `.git` here is a directory.
        let config_path = self.root.join(".git").join("config");
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .open(&config_path)?;
        file.write_all(FIXTURE_LOCAL_CONFIG.as_bytes())?;
        Ok(())
    }

    pub fn commit(mut self, message: &str) -> Result<Self> {
        let file_name = message_file_name(message);
        std::fs::write(self.root.join(&file_name), message)?;
        self.add_and_commit(&file_name, Some(message.as_bytes()), message)?;
        Ok(self)
    }

    /// `git add -- path` + `git commit -m message` (`contents` = what was
    /// just written at `path`, `None` for a deletion: the file is already
    /// gone and the path is staged as removed). One process, no `git`
    /// spawn, when the fixture is plain; otherwise the two real commands.
    fn add_and_commit(&mut self, path: &str, contents: Option<&[u8]>, message: &str) -> Result<()> {
        if self.fast_commit_ok() {
            if let Ok(()) = self.fast_commit(path, contents, message) {
                return Ok(());
            }
        }
        if contents.is_some() {
            self.run(&["add", "--", path])?;
        } else {
            self.run(&["rm", "--cached", "--quiet", "--", path])?;
        }
        self.run(&["commit", "-m", message])
    }

    /// Whether `add_and_commit` may skip the `git` CLI: only for a plain
    /// repository where git's own add/commit has nothing to add to what
    /// libgit2 does with raw bytes. Anything that makes the CLI do more than
    /// store the bytes (attributes and filters such as LFS, the reftable
    /// ref format, config a test set on purpose) keeps the real commands.
    /// `GITPOULPI_FIXTURE_GIT_CLI=1` forces them everywhere (to compare).
    fn fast_commit_ok(&self) -> bool {
        if self.cli_commits || std::env::var_os("GITPOULPI_FIXTURE_GIT_CLI").is_some() {
            return false;
        }
        if self.root.join(".gitattributes").exists() {
            return false;
        }
        let config =
            std::fs::read_to_string(self.root.join(".git").join("config")).unwrap_or_default();
        !config.contains("[filter") && !config.contains("refstorage")
    }

    /// Stages `path` (raw bytes, no filters: `autocrlf` is pinned off and
    /// there are no attributes, so the CLI would store the same blob) and
    /// commits the index on HEAD the way `git commit -m` does: same message
    /// cleanup, the fixture identity, the fixed date when asked for, a
    /// `commit` / `commit (initial)` reflog entry. A test pins that the
    /// resulting commit id equals the CLI's.
    fn fast_commit(&mut self, path: &str, contents: Option<&[u8]>, message: &str) -> Result<()> {
        use git2::{IndexEntry, IndexTime, Signature, Time};
        self.commands
            .push(vec!["(libgit2 commit)".into(), path.into()]);
        let err = |e: git2::Error| Error::GitFailed(format!("fixture commit via libgit2: {e}"));
        let repo = git2::Repository::open(&self.root).map_err(err)?;
        let mut index = repo.index().map_err(err)?;
        let rel = std::path::Path::new(path);
        match contents {
            Some(bytes) => {
                let meta = std::fs::metadata(self.root.join(rel))?;
                #[cfg(not(unix))]
                let time = |t: std::io::Result<std::time::SystemTime>| {
                    let d = t
                        .ok()
                        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                        .unwrap_or_default();
                    IndexTime::new(d.as_secs() as i32, d.subsec_nanos())
                };
                #[cfg(unix)]
                let (mode, dev, ino, uid, gid, ctime, mtime) = {
                    use std::os::unix::fs::MetadataExt;
                    let exec = meta.mode() & 0o111 != 0;
                    (
                        if exec { 0o100755 } else { 0o100644 },
                        meta.dev() as u32,
                        meta.ino() as u32,
                        meta.uid(),
                        meta.gid(),
                        IndexTime::new(meta.ctime() as i32, meta.ctime_nsec() as u32),
                        IndexTime::new(meta.mtime() as i32, meta.mtime_nsec() as u32),
                    )
                };
                // Git for Windows stores no device, inode, owner or mode
                // bits either.
                #[cfg(not(unix))]
                let (mode, dev, ino, uid, gid, ctime, mtime) = (
                    0o100644,
                    0,
                    0,
                    0,
                    0,
                    time(meta.created()),
                    time(meta.modified()),
                );
                let entry = IndexEntry {
                    ctime,
                    mtime,
                    dev,
                    ino,
                    mode,
                    uid,
                    gid,
                    file_size: bytes.len() as u32,
                    id: git2::Oid::ZERO_SHA1,
                    flags: 0,
                    flags_extended: 0,
                    path: path.as_bytes().to_vec(),
                };
                index.add_frombuffer(&entry, bytes).map_err(err)?;
            }
            None => index.remove_path(rel).map_err(err)?,
        }
        index.write().map_err(err)?;
        let tree = repo
            .find_tree(index.write_tree().map_err(err)?)
            .map_err(err)?;
        let when = if self.fixed_dates {
            1_577_836_800 // FIXED_DATE: 2020-01-01T00:00:00+0000
        } else {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0)
        };
        let sig = Signature::new(
            "GitPoulpi Testkit",
            "testkit@gitpoulpi.local",
            &Time::new(when, 0),
        )
        .map_err(err)?;
        // `git commit -m` cleans the message up ("whitespace" mode).
        let message = git2::message_prettify(message, None).map_err(err)?;
        let parent = repo.head().ok().and_then(|h| h.peel_to_commit().ok());
        let parents: Vec<&git2::Commit> = parent.iter().collect();
        // Not `commit(Some("HEAD"), ..)`: its reflog line joins the whole
        // first paragraph, the CLI's is the subject line.
        let head = repo.find_reference("HEAD").map_err(err)?;
        let Some(branch) = head.symbolic_target().ok().flatten().map(str::to_owned) else {
            return Err(Error::GitFailed("detached HEAD: use the CLI".into()));
        };
        let id = repo
            .commit(None, &sig, &sig, &message, &tree, &parents)
            .map_err(err)?;
        let subject = message.lines().next().unwrap_or("");
        let kind = if parents.is_empty() {
            "commit (initial)"
        } else {
            "commit"
        };
        repo.reference(&branch, id, true, &format!("{kind}: {subject}"))
            .map_err(err)?;
        Ok(())
    }

    /// The commit nearly every fixture starts from: `commit("initial")`.
    pub fn with_initial(self) -> Result<Self> {
        self.commit("initial")
    }

    /// Like `commit`, but with a full `summary\n\nbody` message - for
    /// exercising `CommitNode::message` against a real multi-paragraph
    /// body, not just a single-line summary.
    pub fn commit_with_body(mut self, summary: &str, body: &str) -> Result<Self> {
        let file_name = message_file_name(summary);
        std::fs::write(self.root.join(&file_name), summary)?;
        let full_message = format!("{summary}\n\n{body}");
        self.add_and_commit(&file_name, Some(summary.as_bytes()), &full_message)?;
        Ok(self)
    }

    /// Commits `contents` at `path`, byte for byte. Takes `&[u8]` because the
    /// read-blob and blame contracts need CRLF content, content with no
    /// final newline and content with an embedded NUL, none of which survive
    /// a helper that appends its own `\n` or insists on UTF-8.
    pub fn commit_bytes(mut self, path: &str, contents: &[u8], message: &str) -> Result<Self> {
        let full = self.root.join(path);
        if let Some(parent) = full.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&full, contents)?;
        self.add_and_commit(path, Some(contents), message)?;
        Ok(self)
    }

    /// Commits a real `w` x `h` PNG (`rgba` is 4 bytes per pixel, row
    /// major) at `path`.
    pub fn commit_png(
        self,
        path: &str,
        w: u32,
        h: u32,
        rgba: &[u8],
        message: &str,
    ) -> Result<Self> {
        self.commit_bytes(path, &png_bytes(w, h, rgba), message)
    }

    /// `commit_bytes` for text, for the common case.
    pub fn commit_file(self, path: &str, contents: &str, message: &str) -> Result<Self> {
        self.commit_bytes(path, contents.as_bytes(), message)
    }

    /// Commits a raw Git LFS pointer text for `oid`/`size` at `path`, as
    /// a plain blob. Needs no `git-lfs` binary (the fixture has no LFS
    /// filter configured, so git stores the text as given) - which is
    /// what the pointer-only contract checks want.
    pub fn commit_pointer_text(
        self,
        path: &str,
        oid: &str,
        size: u64,
        message: &str,
    ) -> Result<Self> {
        let pointer = gitpoulpi_core::domain::LfsPointer {
            oid: oid.to_string(),
            size,
        };
        self.commit_bytes(path, pointer.to_text().as_bytes(), message)
    }

    /// Deletes `path` and commits the deletion - so a per-path log can
    /// be asserted to still include the commit that removed the file.
    pub fn commit_removal(mut self, path: &str, message: &str) -> Result<Self> {
        match std::fs::remove_file(self.root.join(path)) {
            Ok(()) => {}
            // Not on disk: let `git rm --cached` decide whether it is tracked.
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.into()),
        }
        self.add_and_commit(path, None, message)?;
        Ok(self)
    }

    /// Writes `contents` into the working tree without staging or
    /// committing it.
    pub fn write_worktree(self, path: &str, contents: &[u8]) -> Result<Self> {
        let full = self.root.join(path);
        if let Some(parent) = full.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(full, contents)?;
        Ok(self)
    }

    /// Stages whatever is currently in the working tree at `path`.
    pub fn stage(mut self, path: &str) -> Result<Self> {
        self.run(&["add", "--", path])?;
        Ok(self)
    }

    /// Sets a repo-local config key, so a test can build a repository
    /// whose own config reshapes git's output (`diff.mnemonicPrefix`,
    /// `color.ui`, ...) and check that the backends parse it anyway.
    pub fn config(mut self, key: &str, value: &str) -> Result<Self> {
        self.run(&["config", key, value])?;
        // The test wants git to behave differently: later commits are git's.
        self.cli_commits = true;
        Ok(self)
    }

    pub fn branch(mut self, name: &str) -> Result<Self> {
        self.run(&["checkout", "-b", name])?;
        Ok(self)
    }

    pub fn checkout(mut self, name: &str) -> Result<Self> {
        self.run(&["checkout", name])?;
        Ok(self)
    }

    /// An *annotated* tag (`-a`) at the current commit: its own ref points
    /// at a tag object rather than the commit directly, the case `refs()`
    /// must peel.
    pub fn annotated_tag(mut self, name: &str) -> Result<Self> {
        self.run(&["tag", "-a", name, "-m", name])?;
        Ok(self)
    }

    /// Adds a linked worktree (`git worktree add`) on a new branch,
    /// placed beside the repo inside the fixture's temp dir. Returns the
    /// worktree's path alongside the builder so tests can assert on it.
    pub fn worktree(mut self, name: &str, branch: &str) -> Result<(Self, PathBuf)> {
        let path = self.dir.path().join(name);
        let path_str = path.to_string_lossy().to_string();
        self.run(&["worktree", "add", "-b", branch, &path_str])?;
        Ok((self, path))
    }

    /// Gives the fixture a real remote to push to: a **bare repository
    /// beside it inside the same temp dir**, wired up as `origin`.
    ///
    /// A filesystem path is a first-class git remote and exercises the same
    /// `git push` plumbing as a URL, with no credentials or network. It lives
    /// inside the `TempDir` (cleaned up with the fixture) and *outside* the
    /// repository (never an untracked file in a `status()` assertion).
    ///
    /// **Cost: one `git` spawn**, and only for fixtures that ask for one: the
    /// `[remote "origin"]` section is written into `.git/config` directly
    /// instead of spawning `git remote add`, as `write_local_config` does,
    /// and pinned the same way (`git config --get`).
    pub fn with_origin(mut self) -> Result<Self> {
        use std::io::Write;

        let remote = self.dir.path().join("origin.git");
        let remote_str = remote.to_string_lossy().to_string();
        copy_dir(&init_skeleton(true)?, &remote)?;

        // git reads backslashes in a config *value* as escapes, so a
        // Windows path has to be written with forward slashes -- which
        // git accepts everywhere -- or `C:\Users\...` loses its `\U`.
        let url = remote_str.replace('\\', "/");
        let section = format!(
            "[remote \"origin\"]\n\turl = {url}\n\tfetch = +refs/heads/*:refs/remotes/origin/*\n"
        );
        let config_path = self.root.join(".git").join("config");
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .open(&config_path)?;
        file.write_all(section.as_bytes())?;

        self.remote = Some(remote);
        Ok(self)
    }

    /// A second (third, ...) remote named `name`, backed by its own bare
    /// repository inside the temp dir - `with_origin`'s twin for the
    /// operations that are about *more than one* remote (fetch all,
    /// rename, pushing to a chosen remote). Same cost: one `git` spawn.
    pub fn with_remote(mut self, name: &str) -> Result<Self> {
        use std::io::Write;

        let dir_name = format!("{}.git", name.replace('/', "_"));
        let remote = self.dir.path().join(dir_name);
        let remote_str = remote.to_string_lossy().to_string();
        copy_dir(&init_skeleton(true)?, &remote)?;
        let url = remote_str.replace('\\', "/");
        let section = format!(
            "[remote \"{name}\"]\n\turl = {url}\n\tfetch = +refs/heads/*:refs/remotes/{name}/*\n"
        );
        let config_path = self.root.join(".git").join("config");
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .open(&config_path)?;
        file.write_all(section.as_bytes())?;
        self.extra_remotes.push((name.to_string(), remote));
        Ok(self)
    }

    /// Installs `script` (a shell script body, `#!/bin/sh` is prepended) as
    /// the `pre-receive` hook of the bare remote `remote` (`origin` or a
    /// `with_remote` name), so a push to it can be made to refuse. Needs
    /// `sh` on the machine running the test.
    pub fn with_pre_receive_hook(self, remote: &str, script: &str) -> Result<Self> {
        let path = if remote == "origin" {
            self.remote.clone()
        } else {
            self.extra_remotes
                .iter()
                .find(|(n, _)| n == remote)
                .map(|(_, p)| p.clone())
        }
        .unwrap_or_else(|| panic!("this fixture has no remote `{remote}`"));
        let hooks = path.join("hooks");
        std::fs::create_dir_all(&hooks)?;
        let hook = hooks.join("pre-receive");
        std::fs::write(&hook, format!("#!/bin/sh\n{script}\n"))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o755))?;
        }
        Ok(self)
    }

    /// A submodule at `path`, cloned from its own repository inside the
    /// temp dir (one commit, `lib.txt`) and committed in the superproject.
    /// The source's path is [`FixtureRepo::submodule_source`].
    pub fn with_submodule(mut self, path: &str) -> Result<Self> {
        let source = self
            .dir
            .path()
            .join(format!("submodule-source-{}", self.submodule_sources.len()));
        std::fs::create_dir_all(&source)?;
        git_stdout(&source, &["init", "--initial-branch=main"]);
        std::fs::write(source.join("lib.txt"), "library\n")?;
        git_stdout(&source, &["add", "lib.txt"]);
        git_stdout(
            &source,
            &[
                "-c",
                "user.name=Testkit",
                "-c",
                "user.email=testkit@gitpoulpi.local",
                "commit",
                "-m",
                "library",
            ],
        );
        let url = source.to_string_lossy().replace('\\', "/");
        self.run(&[
            "-c",
            "protocol.file.allow=always",
            "submodule",
            "add",
            &url,
            path,
        ])?;
        self.run(&["commit", "-m", &format!("add submodule {path}")])?;
        self.submodule_sources.push((path.to_string(), source));
        Ok(self)
    }

    /// Installs the Git LFS filters and hooks into *this repository only*
    /// (`git lfs install --local`), so the fixture is hermetic even though
    /// the user's global config is ignored. Needs the `git-lfs` binary:
    /// guard the calling check with `require_git_lfs!`.
    pub fn with_lfs(mut self) -> Result<Self> {
        self.run(&["lfs", "install", "--local"])?;
        Ok(self)
    }

    /// Routes `pattern` through LFS (`git lfs track`) and commits the
    /// resulting `.gitattributes`.
    pub fn lfs_track(self, pattern: &str) -> Result<Self> {
        self.lfs_track_with(pattern, false)
    }

    /// Like [`lfs_track`](Self::lfs_track), with the `lockable` attribute.
    pub fn lfs_track_lockable(self, pattern: &str) -> Result<Self> {
        self.lfs_track_with(pattern, true)
    }

    fn lfs_track_with(mut self, pattern: &str, lockable: bool) -> Result<Self> {
        if lockable {
            self.run(&["lfs", "track", "--lockable", "--", pattern])?;
        } else {
            self.run(&["lfs", "track", "--", pattern])?;
        }
        self.run(&["add", "--", ".gitattributes"])?;
        self.run(&["commit", "-m", &format!("track {pattern} with LFS")])?;
        Ok(self)
    }

    /// Commits `contents` at `path` through the LFS clean filter, so git
    /// stores a pointer and the real bytes go to the local LFS store. Same
    /// as `commit_bytes`; call `with_lfs` and `lfs_track` first.
    pub fn commit_lfs_file(self, path: &str, contents: &[u8], message: &str) -> Result<Self> {
        self.commit_bytes(path, contents, message)
    }

    /// [`with_origin`](Self::with_origin) plus an LFS endpoint that is the
    /// origin's own directory, reached through git-lfs's built-in
    /// `file://` adapter - so pushes, fetches and clones that move LFS
    /// objects work with no network and no server.
    ///
    /// The endpoint is written both to a committed `.lfsconfig` (which
    /// travels with clones) and to the repository's own config. That adds
    /// one commit, `configure LFS endpoint`, so call this before the
    /// commits a check asserts on.
    pub fn with_lfs_origin(mut self) -> Result<Self> {
        self = self.with_origin()?;
        let remote = self.remote.clone().expect("with_origin sets the remote");
        let url = file_url(&remote);
        std::fs::write(
            self.root.join(".lfsconfig"),
            format!("[lfs]\n\turl = {url}\n"),
        )?;
        self.run(&["config", "lfs.url", &url])?;
        self.run(&["add", "--", ".lfsconfig"])?;
        self.run(&["commit", "-m", "configure LFS endpoint"])?;
        Ok(self)
    }

    /// Pushes `refspec` to the remote `name`, as the fixture.
    pub fn push_to(mut self, name: &str, refspec: &str) -> Result<Self> {
        self.run(&["push", name, refspec])?;
        Ok(self)
    }

    /// Makes `branch` track `<remote>/<branch>`, by config, the way
    /// `git push -u` would have left it.
    pub fn track(self, branch: &str, remote: &str) -> Result<Self> {
        self.config(&format!("branch.{branch}.remote"), remote)?
            .config(
                &format!("branch.{branch}.merge"),
                &format!("refs/heads/{branch}"),
            )
    }

    /// Writes `branch`'s extra push targets (`gitpoulpi-push-to`, one
    /// value per entry, `<remote>` or `<remote>:<branch>`), in order.
    pub fn push_targets(mut self, branch: &str, targets: &[&str]) -> Result<Self> {
        let key = format!("branch.{branch}.gitpoulpi-push-to");
        for target in targets {
            self.run(&["config", "--add", &key, target])?;
        }
        Ok(self)
    }

    /// A lightweight tag at the current commit.
    pub fn tag(mut self, name: &str) -> Result<Self> {
        self.run(&["tag", name])?;
        Ok(self)
    }

    /// Pushes `refspec` to `origin`, as the *fixture* rather than as a
    /// backend under test - for arranging a starting state ("the remote
    /// already has this") that the check then observes.
    pub fn push(mut self, refspec: &str) -> Result<Self> {
        self.run(&["push", "origin", refspec])?;
        Ok(self)
    }

    pub fn build(self) -> FixtureRepo {
        FixtureRepo {
            root: self.root,
            remote: self.remote,
            extra_remotes: self.extra_remotes,
            submodule_sources: self.submodule_sources,
            _dir: self.dir,
        }
    }

    fn run(&mut self, args: &[&str]) -> Result<()> {
        self.commands
            .push(args.iter().map(|s| s.to_string()).collect());
        let mut cmd = Command::new("git");
        cmd.args(args).current_dir(&self.root);
        isolate_from_user_config(&mut cmd);
        if self.fixed_dates {
            cmd.env("GIT_AUTHOR_DATE", FIXED_DATE);
            cmd.env("GIT_COMMITTER_DATE", FIXED_DATE);
        }
        let output = cmd.output()?;
        if !output.status.success() {
            return Err(Error::GitFailed(format!(
                "git {} failed: {}",
                args.join(" "),
                String::from_utf8_lossy(&output.stderr)
            )));
        }
        Ok(())
    }
}

/// Cuts a fixture's `git` off from the machine's global and system config,
/// so a fixture is the same repository on every box: a global
/// `commit.gpgsign = true` would fail every fixture commit, a global
/// `core.hooksPath` would run someone's hooks, and `init.templateDir` would
/// drop their files in.
///
/// Deliberately *not* what the backends do at run time: they honour the
/// user's config and only pin the output shape (see `gitpoulpi-cli-backend`'s
/// `process::CONFIG_OVERRIDES`). Repo-local config still applies here.
pub(crate) fn isolate_from_user_config(cmd: &mut Command) {
    // An empty path reads as "no such file", i.e. no config, on every
    // platform -- including Windows, where `/dev/null` does not exist.
    cmd.env("GIT_CONFIG_GLOBAL", "");
    cmd.env("GIT_CONFIG_SYSTEM", "");
    cmd.env("GIT_CONFIG_NOSYSTEM", "1");
    cmd.env_remove("GIT_CONFIG_PARAMETERS");
    cmd.env_remove("GIT_CONFIG_COUNT");
    cmd.env_remove("GIT_DIR");
    cmd.env_remove("GIT_WORK_TREE");
    cmd.env_remove("GIT_INDEX_FILE");
}

impl Default for FixtureRepoBuilder {
    fn default() -> Self {
        Self::new().expect("failed to create fixture repo temp dir")
    }
}

/// A `w` x `h` RGBA PNG (4 bytes per pixel, row major).
pub fn png_bytes(w: u32, h: u32, rgba: &[u8]) -> Vec<u8> {
    assert_eq!(rgba.len(), (w * h * 4) as usize, "rgba must be w*h*4 bytes");
    let mut out = Vec::new();
    let mut enc = png::Encoder::new(&mut out, w, h);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header()
        .and_then(|mut w| w.write_image_data(rgba))
        .expect("png encoding of a fixture image");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A fixture must pin its own line endings, because the backends under
    /// test honour the user's git config and Git for Windows ships
    /// `core.autocrlf = true` system-wide; otherwise assertions on bytes a
    /// backend wrote pass on Linux and macOS and fail on Windows.
    #[test]
    fn a_fixture_pins_its_line_endings_for_the_backends_that_read_it() {
        let repo = FixtureRepoBuilder::new()
            .expect("fixture")
            .commit("initial")
            .expect("commit")
            .build();

        for (key, want) in [
            ("core.autocrlf", "false"),
            ("core.eol", "lf"),
            ("user.name", "GitPoulpi Testkit"),
            ("user.email", "testkit@gitpoulpi.local"),
        ] {
            let out = Command::new("git")
                .args(["config", "--local", "--get", key])
                .current_dir(repo.root())
                .output()
                .expect("git config");
            let got = String::from_utf8_lossy(&out.stdout).trim().to_string();
            assert_eq!(got, want, "{key} must be pinned repo-locally");
        }
    }

    /// The libgit2 shortcut of `add_and_commit` must be the repository the
    /// CLI would have made: same commit ids (fixed dates), same reflog, a
    /// clean status and a clean `diff-files` (stat data the CLI trusts), for
    /// new and changed files, a removal and messages the CLI cleans up.
    #[test]
    fn the_libgit2_commit_shortcut_matches_the_cli() {
        let steps = |cli: bool| {
            let mut b = FixtureRepoBuilder::new().unwrap().fixed_dates();
            b.cli_commits = cli;
            b.commit_file("a.txt", "one\n", "first\n\n\n")
                .unwrap()
                .commit_bytes("dir/b.bin", b"\0crlf\r\n", "second  \nbody  \n\n\nmore")
                .unwrap()
                .commit_file("a.txt", "two\n", "third")
                .unwrap()
                .commit_removal("dir/b.bin", "fourth")
                .unwrap()
                .commit_with_body("fifth", "a body")
                .unwrap()
                .build()
        };
        let fast = steps(false);
        let cli = steps(true);
        let q = |r: &FixtureRepo, a: &[&str]| r.git(a);
        assert_eq!(
            q(&fast, &["rev-parse", "HEAD"]),
            q(&cli, &["rev-parse", "HEAD"])
        );
        assert_eq!(
            q(&fast, &["reflog", "--format=%gs"]),
            q(&cli, &["reflog", "--format=%gs"])
        );
        assert_eq!(q(&fast, &["status", "--porcelain"]), "");
        assert_eq!(q(&fast, &["diff-files", "--name-only"]), "");
        assert_eq!(q(&fast, &["diff-index", "--name-only", "HEAD"]), "");
        q(&fast, &["fsck", "--strict"]);
    }
}
