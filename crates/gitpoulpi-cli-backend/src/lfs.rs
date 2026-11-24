//! Git LFS through the system `git-lfs` program, plus the LFS *reads* that
//! need no program at all (see ADR 0009).
//!
//! What lives here:
//!
//! - the availability probe ([`LfsProbe`]) with its 60 s cache and the
//!   hidden `GITPOULPI_GIT_LFS=missing` test seam;
//! - `lfs_status` and `lfs_files`, which read `.gitattributes`, git
//!   config, the LFS hook and the object store directly, so the UI can
//!   say "LFS object, 12 MB, not downloaded" with `git-lfs` missing.

use std::path::{Path, PathBuf};

use gitpoulpi_core::domain::{
    attributes_mention_lfs, parse_lfs_patterns, LfsBinary, LfsFile, LfsInstallScope, LfsPattern,
    LfsPointer, LfsRepoStatus, LfsTrackOptions, Oid, RepoRef, LFS_POINTER_MAX_BYTES,
};
use gitpoulpi_core::{Error, Result};

use crate::process::{run, run_bytes_with_stdin, run_lenient, run_with_stdin};
use crate::tools::{lfs_of_status, status_of_lfs, ToolProbe};
use gitpoulpi_core::domain::rewrite::Tool;

/// Test seam: `GITPOULPI_GIT_LFS=missing` makes the probe answer
/// [`LfsBinary::Missing`] even where `git-lfs` is installed. The general
/// form is `GITPOULPI_TOOL_GIT_LFS` (see [`crate::tools::parse_override`]);
/// both are honoured.
///
/// It has to be an environment variable rather than a trimmed `PATH`: on
/// Windows `git lfs` finds `git-lfs.exe` through Git for Windows' own
/// `mingw64/bin`, so no `PATH` edit can hide it. An environment variable
/// also crosses the daemon and pty process boundaries, which is what the
/// TUI's missing-binary tests need.
#[allow(dead_code)]
pub const OVERRIDE_ENV: &str = crate::tools::LEGACY_LFS_ENV;

/// The `git-lfs` view of a [`ToolProbe`]: the same 60 s cache, `fresh()` and
/// test seams, answering [`LfsBinary`].
#[derive(Default)]
pub struct LfsProbe {
    inner: ToolProbe,
}

impl LfsProbe {
    /// A probe that always answers `binary`. Unlike the environment
    /// variable it cannot leak into a parallel test.
    pub fn forced(binary: LfsBinary) -> Self {
        Self {
            inner: ToolProbe::forced(Tool::GitLfs, status_of_lfs(&binary)),
        }
    }

    /// The binary's state, from the cache when it is under a minute old.
    pub fn get(&self, repo_root: &Path) -> LfsBinary {
        lfs_of_status(&self.inner.get(Tool::GitLfs, repo_root))
    }

    /// Always asks the program, for explicit user actions (`lfs_install`);
    /// the refresh path uses the cached [`LfsProbe::get`].
    pub fn fresh(&self, repo_root: &Path) -> LfsBinary {
        lfs_of_status(&self.inner.fresh(Tool::GitLfs, repo_root))
    }

    /// `Err(LfsUnavailable)` unless a usable `git-lfs` is installed.
    pub fn require(&self, repo_root: &Path) -> Result<()> {
        match self.get(repo_root).unavailable() {
            None => Ok(()),
            Some(reason) => Err(Error::LfsUnavailable { reason }),
        }
    }
}

/// `GITPOULPI_GIT_LFS` value to forced answer. Only `missing` is defined.
#[cfg(test)]
pub(crate) fn parse_override(value: Option<&str>) -> Option<LfsBinary> {
    match value.map(str::trim) {
        Some(v) if v.eq_ignore_ascii_case("missing") => Some(LfsBinary::Missing),
        _ => None,
    }
}

/// Where the repository keeps the pieces LFS cares about.
struct GitPaths {
    common_dir: PathBuf,
    hooks_dir: PathBuf,
}

fn git_paths(repo: &RepoRef) -> Result<GitPaths> {
    if let Some(paths) = git_paths_fast(repo) {
        return Ok(paths);
    }
    git_paths_spawn(repo)
}

/// The git directory of `root` read from the filesystem: a `.git`
/// directory, a `.git` file (`gitdir: <path>`, linked worktree or
/// submodule), or a bare repository (`root` itself).
fn resolve_git_dir(root: &Path) -> Option<PathBuf> {
    gitpoulpi_core::paths::git_dir(root).or_else(|| {
        (!root.join(".git").exists()
            && root.join("HEAD").is_file()
            && root.join("objects").is_dir())
        .then(|| root.to_path_buf())
    })
}

/// `git_paths` without a spawn. `None` (so the caller asks git) whenever
/// the answer is not certain: a `core.hooksPath` under `~`, or config
/// files using `include`.
fn git_paths_fast(repo: &RepoRef) -> Option<GitPaths> {
    let git_dir = resolve_git_dir(&repo.root)?;
    let common_dir = gitpoulpi_core::paths::common_dir(&git_dir);
    let common_dir = strip_verbatim(std::fs::canonicalize(&common_dir).unwrap_or(common_dir));
    let facts = config_facts(&common_dir)?;
    let hooks_dir = match facts.hooks_path {
        Some(p) if p.starts_with('~') => return None,
        Some(p) => {
            let p = PathBuf::from(p);
            if p.is_absolute() {
                p
            } else {
                repo.root.join(p)
            }
        }
        None => common_dir.join("hooks"),
    };
    Some(GitPaths {
        common_dir,
        hooks_dir,
    })
}

/// Drops the Windows `\\?\` prefix `canonicalize` adds, so paths read and
/// compare like git's own.
fn strip_verbatim(p: PathBuf) -> PathBuf {
    match p.to_str().and_then(|s| s.strip_prefix(r"\\?\")) {
        Some(rest) if !rest.starts_with("UNC") => PathBuf::from(rest),
        _ => p,
    }
}

/// What LFS needs from git config, merged over system, global and
/// repository files.
#[derive(Default)]
struct ConfigFacts {
    filters_configured: bool,
    hooks_path: Option<String>,
}

/// The config files git would read, lowest precedence first, or `None`
/// when the system file cannot be located without asking git.
fn config_files(common_dir: &Path) -> Option<Vec<PathBuf>> {
    let env = |k: &str| {
        std::env::var_os(k)
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
    };
    let mut files = Vec::new();
    if std::env::var_os("GIT_CONFIG_NOSYSTEM").is_none() {
        files.push(match env("GIT_CONFIG_SYSTEM") {
            Some(p) => p,
            None => system_gitconfig()?,
        });
    }
    let home = env("HOME").or_else(|| env("USERPROFILE"));
    match env("GIT_CONFIG_GLOBAL") {
        Some(p) => files.push(p),
        None => {
            let xdg = env("XDG_CONFIG_HOME").or_else(|| home.as_ref().map(|h| h.join(".config")));
            if let Some(x) = xdg {
                files.push(x.join("git").join("config"));
            }
            if let Some(h) = &home {
                files.push(h.join(".gitconfig"));
            }
        }
    }
    files.push(common_dir.join("config"));
    Some(files)
}

/// The system gitconfig next to the `git` found on `PATH`
/// (`<prefix>/etc/gitconfig`; Git for Windows keeps `git.exe` one or two
/// levels below its prefix), else `/etc/gitconfig`.
fn system_gitconfig() -> Option<PathBuf> {
    static CACHE: std::sync::OnceLock<Option<PathBuf>> = std::sync::OnceLock::new();
    CACHE
        .get_or_init(|| {
            if cfg!(not(windows)) && Path::new("/etc/gitconfig").exists() {
                return Some(PathBuf::from("/etc/gitconfig"));
            }
            let exe = if cfg!(windows) { "git.exe" } else { "git" };
            let path = std::env::var_os("PATH")?;
            for dir in std::env::split_paths(&path) {
                if !dir.join(exe).is_file() {
                    continue;
                }
                for up in [1, 2] {
                    let mut prefix = dir.clone();
                    for _ in 0..up {
                        prefix = prefix.parent()?.to_path_buf();
                    }
                    let candidate = prefix.join("etc").join("gitconfig");
                    if candidate.is_file() {
                        return Some(candidate);
                    }
                }
                // Found git but no system file: there is none to read.
                return Some(dir.join("no-system-gitconfig"));
            }
            None
        })
        .clone()
}

fn config_facts(common_dir: &Path) -> Option<ConfigFacts> {
    let mut facts = ConfigFacts::default();
    for file in config_files(common_dir)? {
        let Ok(text) = std::fs::read_to_string(&file) else {
            continue;
        };
        if !scan_config(&text, &mut facts) {
            return None;
        }
    }
    Some(facts)
}

/// Reads the keys LFS cares about out of one config file. `false` when the
/// file uses `include` / `includeIf` (not followed here).
fn scan_config(text: &str, facts: &mut ConfigFacts) -> bool {
    let mut section = String::new();
    let mut subsection = String::new();
    for raw in text.lines() {
        let line = raw.trim();
        if let Some(rest) = line.strip_prefix('[') {
            let Some(end) = rest.find(']') else { continue };
            let header = &rest[..end];
            let (name, sub) = match header.split_once(char::is_whitespace) {
                Some((n, s)) => (n, s.trim().trim_matches('"')),
                None => (header, ""),
            };
            section = name.to_ascii_lowercase();
            subsection = sub.to_string();
            if section == "include" || section == "includeif" {
                return false;
            }
            let tail = rest[end + 1..].trim();
            if !tail.is_empty() && !tail.starts_with(['#', ';']) {
                apply_key(&section, &subsection, tail, facts);
            }
            continue;
        }
        if line.is_empty() || line.starts_with(['#', ';']) {
            continue;
        }
        apply_key(&section, &subsection, line, facts);
    }
    true
}

fn apply_key(section: &str, subsection: &str, line: &str, facts: &mut ConfigFacts) {
    let (key, value) = match line.split_once('=') {
        Some((k, v)) => (k.trim(), v.trim().trim_matches('"')),
        None => (line.trim(), ""),
    };
    match (section, key.to_ascii_lowercase().as_str()) {
        ("filter", "process" | "smudge") if subsection == "lfs" && !value.is_empty() => {
            facts.filters_configured = true;
        }
        ("core", "hookspath") if subsection.is_empty() => {
            facts.hooks_path = Some(value.to_string());
        }
        _ => {}
    }
}

fn git_paths_spawn(repo: &RepoRef) -> Result<GitPaths> {
    // `--git-path hooks` honours `core.hooksPath`.
    let out = run(
        &repo.root,
        &[
            "rev-parse",
            "--path-format=absolute",
            "--git-common-dir",
            "--git-path",
            "hooks",
        ],
    )?;
    let mut lines = out.lines();
    let common_dir = lines
        .next()
        .map(PathBuf::from)
        .ok_or_else(|| Error::Backend("git rev-parse gave no git dir".into()))?;
    let hooks_dir = lines
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| common_dir.join("hooks"));
    Ok(GitPaths {
        common_dir,
        hooks_dir,
    })
}

/// The directory holding `objects/aa/bb/<oid>`: `lfs.storage` when set
/// (relative to the git directory), else `<git dir>/lfs`.
fn storage_dir(repo: &RepoRef, paths: &GitPaths) -> PathBuf {
    let configured = run_lenient(&repo.root, &["config", "--get", "lfs.storage"])
        .ok()
        .filter(|(ok, _)| *ok)
        .map(|(_, v)| v.trim().to_string())
        .filter(|v| !v.is_empty());
    match configured {
        Some(p) => {
            let p = PathBuf::from(p);
            if p.is_absolute() {
                p
            } else {
                paths.common_dir.join(p)
            }
        }
        None => paths.common_dir.join("lfs"),
    }
}

/// Patterns from the root `.gitattributes` and `info/attributes`.
/// Nested `.gitattributes` files are deliberately not scanned (cost); the
/// `.git/lfs` directory check catches such repositories once LFS has been
/// used.
fn read_patterns(repo: &RepoRef, paths: &GitPaths) -> Vec<LfsPattern> {
    let mut patterns = Vec::new();
    for (file, label) in [
        (
            repo.root.join(".gitattributes"),
            PathBuf::from(".gitattributes"),
        ),
        (
            paths.common_dir.join("info").join("attributes"),
            PathBuf::from(".git/info/attributes"),
        ),
    ] {
        if let Ok(text) = std::fs::read_to_string(&file) {
            if attributes_mention_lfs(&text) {
                for mut p in parse_lfs_patterns(&text) {
                    p.source = label.clone();
                    patterns.push(p);
                }
            }
        }
    }
    patterns
}

/// Are the LFS clean / smudge filters configured (for this repository, its
/// user, or the system)? Without them git leaves pointer text in the
/// working tree and `git lfs pull` skips the checkout.
pub(crate) fn filters_configured(repo: &RepoRef) -> Result<bool> {
    let (found, configured) = run_lenient(
        &repo.root,
        &["config", "--get-regexp", r"^filter\.lfs\.(process|smudge)$"],
    )?;
    Ok(found && !configured.trim().is_empty())
}

/// Does the repository route files through LFS: the root `.gitattributes`
/// or `info/attributes` name `filter=lfs`, or an LFS object store exists.
pub(crate) fn repo_uses_lfs(repo: &RepoRef) -> bool {
    git_paths(repo).is_ok_and(|paths| {
        !read_patterns(repo, &paths).is_empty() || paths.common_dir.join("lfs").is_dir()
    })
}

/// See `GitBackend::lfs_status`.
pub fn lfs_status(repo: &RepoRef, probe: &LfsProbe) -> Result<LfsRepoStatus> {
    // The cached probe: this runs on every refresh that includes LFS, so
    // it must not spawn `git lfs version` each time (60 s staleness).
    let binary = probe.get(&repo.root);
    let paths = git_paths(repo)?;
    let patterns = read_patterns(repo, &paths);
    let uses_lfs = !patterns.is_empty() || paths.common_dir.join("lfs").is_dir();

    let filters_configured = match config_facts(&paths.common_dir) {
        Some(facts) => facts.filters_configured,
        None => filters_configured(repo)?,
    };

    let hooks_installed = ["pre-push", "post-checkout"].iter().any(|hook| {
        std::fs::read_to_string(paths.hooks_dir.join(hook))
            .is_ok_and(|text| text.contains("git lfs") || text.contains("git-lfs"))
    });

    Ok(LfsRepoStatus {
        binary,
        uses_lfs,
        filters_configured,
        hooks_installed,
        patterns,
    })
}

/// See `GitBackend::lfs_install`.
pub fn lfs_install(repo: &RepoRef, probe: &LfsProbe, scope: LfsInstallScope) -> Result<()> {
    // An explicit user action: ask the program, not the 60 s cache.
    if let Some(reason) = probe.fresh(&repo.root).unavailable() {
        return Err(Error::LfsUnavailable { reason });
    }
    match scope {
        LfsInstallScope::Repository => run(&repo.root, &["lfs", "install", "--local"])?,
        // `--skip-repo`: a user-level install must not also touch the
        // repository the command happens to run in.
        LfsInstallScope::User => run(&repo.root, &["lfs", "install", "--skip-repo"])?,
    };
    Ok(())
}

/// The `filter=lfs` patterns of the repository right now.
fn current_patterns(repo: &RepoRef) -> Result<Vec<LfsPattern>> {
    let paths = git_paths(repo)?;
    Ok(read_patterns(repo, &paths))
}

/// See `GitBackend::lfs_track`.
pub fn lfs_track(
    repo: &RepoRef,
    probe: &LfsProbe,
    patterns: &[String],
    opts: &LfsTrackOptions,
) -> Result<Vec<LfsPattern>> {
    probe.require(&repo.root)?;
    if patterns.is_empty() {
        return current_patterns(repo);
    }
    let mut args = vec!["lfs", "track"];
    if opts.lockable {
        args.push("--lockable");
    }
    args.push("--");
    args.extend(patterns.iter().map(String::as_str));
    run(&repo.root, &args)?;

    if opts.renormalize {
        // The already-committed files the patterns now cover, re-staged
        // through the clean filter (so the index gets pointers).
        let mut listed: Vec<String> = Vec::new();
        for pattern in patterns {
            let out = run(&repo.root, &["ls-files", "-z", "--", pattern])?;
            listed.extend(
                out.split('\0')
                    .filter(|p| !p.is_empty())
                    .map(str::to_string),
            );
        }
        listed.sort();
        listed.dedup();
        if !listed.is_empty() {
            let mut input = listed.join("\0");
            input.push('\0');
            run_with_stdin(
                &repo.root,
                &[
                    "add",
                    "--renormalize",
                    "--pathspec-from-file=-",
                    "--pathspec-file-nul",
                ],
                &input,
            )?;
        }
    }
    current_patterns(repo)
}

/// See `GitBackend::lfs_untrack`.
pub fn lfs_untrack(
    repo: &RepoRef,
    probe: &LfsProbe,
    patterns: &[String],
) -> Result<Vec<LfsPattern>> {
    probe.require(&repo.root)?;
    if !patterns.is_empty() {
        let mut args = vec!["lfs", "untrack", "--"];
        args.extend(patterns.iter().map(String::as_str));
        run(&repo.root, &args)?;
    }
    current_patterns(repo)
}

/// See `GitBackend::lfs_prune`.
pub fn lfs_prune(repo: &RepoRef, probe: &LfsProbe) -> Result<()> {
    probe.require(&repo.root)?;
    run(&repo.root, &["lfs", "prune"])?;
    Ok(())
}

/// A blob candidate for being a pointer: path, object name and, when
/// known up front, its size.
struct Candidate {
    path: String,
    sha: String,
    size: Option<u64>,
}

/// See `GitBackend::lfs_files`.
pub fn lfs_files(repo: &RepoRef, at: Option<Oid>) -> Result<Vec<LfsFile>> {
    let paths = git_paths(repo)?;
    let patterns = read_patterns(repo, &paths);
    // The gate (the working tree's attributes, an LFS store) only spares
    // the index scan. A revision is asked about for what it holds: its own
    // `.gitattributes` may differ from the checked-out one (another branch,
    // a file since deleted), and the scan is bounded by the pointer size.
    if at.is_none() && patterns.is_empty() && !paths.common_dir.join("lfs").is_dir() {
        return Ok(Vec::new());
    }
    let storage = storage_dir(repo, &paths);

    let mut candidates = match at {
        Some(oid) => tree_candidates(repo, oid)?,
        None => index_candidates(repo)?,
    };

    // Only blobs under the pointer size limit can be pointers. The index
    // listing carries no sizes, so ask for them (one batched process).
    if candidates.iter().any(|c| c.size.is_none()) {
        let input: String = candidates.iter().map(|c| format!("{}\n", c.sha)).collect();
        let out = run_bytes_with_stdin(
            &repo.root,
            &["cat-file", "--batch-check=%(objectsize)"],
            input.as_bytes(),
        )?;
        for (candidate, line) in candidates
            .iter_mut()
            .zip(String::from_utf8_lossy(&out).lines())
        {
            candidate.size = line.trim().parse().ok();
        }
    }
    candidates.retain(|c| {
        c.size
            .is_some_and(|s| (40..LFS_POINTER_MAX_BYTES as u64).contains(&s))
    });
    if candidates.is_empty() {
        return Ok(Vec::new());
    }

    let mut unique: Vec<&str> = candidates.iter().map(|c| c.sha.as_str()).collect();
    unique.sort_unstable();
    unique.dedup();
    let input: String = unique.iter().map(|s| format!("{s}\n")).collect();
    let raw = run_bytes_with_stdin(&repo.root, &["cat-file", "--batch"], input.as_bytes())?;
    let pointers = parse_batch(&raw);

    let mut files = Vec::new();
    for candidate in candidates {
        let Some(Some(pointer)) = pointers.get(&candidate.sha) else {
            continue;
        };
        let local = storage.join(pointer.storage_rel_path()).is_file();
        files.push(LfsFile {
            path: PathBuf::from(candidate.path),
            pointer: pointer.clone(),
            local,
        });
    }
    Ok(files)
}

/// `git ls-files -s -z`: `<mode> <sha> <stage>\t<path>`. Stage-0 regular
/// files only.
fn index_candidates(repo: &RepoRef) -> Result<Vec<Candidate>> {
    let raw = run(&repo.root, &["ls-files", "-s", "-z"])?;
    let mut out = Vec::new();
    for entry in raw.split('\0').filter(|e| !e.is_empty()) {
        let Some((meta, path)) = entry.split_once('\t') else {
            continue;
        };
        let mut parts = meta.split_whitespace();
        let (Some(mode), Some(sha), Some(stage)) = (parts.next(), parts.next(), parts.next())
        else {
            continue;
        };
        if stage == "0" && (mode == "100644" || mode == "100755") {
            out.push(Candidate {
                path: path.to_string(),
                sha: sha.to_string(),
                size: None,
            });
        }
    }
    Ok(out)
}

/// `git ls-tree -r -z -l <commit>`: `<mode> blob <sha> <size>\t<path>`.
fn tree_candidates(repo: &RepoRef, at: Oid) -> Result<Vec<Candidate>> {
    let rev = at.to_hex();
    let raw = run(&repo.root, &["ls-tree", "-r", "-z", "-l", &rev])?;
    let mut out = Vec::new();
    for entry in raw.split('\0').filter(|e| !e.is_empty()) {
        let Some((meta, path)) = entry.split_once('\t') else {
            continue;
        };
        let mut parts = meta.split_whitespace();
        let (Some(mode), Some(kind), Some(sha), Some(size)) =
            (parts.next(), parts.next(), parts.next(), parts.next())
        else {
            continue;
        };
        if kind == "blob" && (mode == "100644" || mode == "100755") {
            out.push(Candidate {
                path: path.to_string(),
                sha: sha.to_string(),
                size: size.parse().ok(),
            });
        }
    }
    Ok(out)
}

/// Parse `git cat-file --batch` output: `<sha> <type> <size>\n<content>\n`
/// repeated. Returns `sha -> pointer` (None when the content is not one).
fn parse_batch(raw: &[u8]) -> std::collections::HashMap<String, Option<LfsPointer>> {
    let mut out = std::collections::HashMap::new();
    let mut pos = 0;
    while pos < raw.len() {
        let Some(nl) = raw[pos..].iter().position(|b| *b == b'\n') else {
            break;
        };
        let header = String::from_utf8_lossy(&raw[pos..pos + nl]).to_string();
        pos += nl + 1;
        let mut parts = header.split_whitespace();
        let (Some(sha), Some(kind), Some(size)) = (parts.next(), parts.next(), parts.next()) else {
            // `<sha> missing`: nothing follows.
            continue;
        };
        let Ok(size) = size.parse::<usize>() else {
            break;
        };
        if pos + size > raw.len() {
            break;
        }
        let content = &raw[pos..pos + size];
        pos += size + 1; // the trailing newline
        let pointer = if kind == "blob" {
            LfsPointer::parse(content)
        } else {
            None
        };
        out.insert(sha.to_string(), pointer);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_config_scan_reads_the_lfs_filter_and_hooks_path() {
        let mut facts = ConfigFacts::default();
        let text = "[core]
	hooksPath = .githooks
[filter \"lfs\"]
	clean = git-lfs clean -- %f
	smudge = git-lfs smudge -- %f
";
        assert!(scan_config(text, &mut facts));
        assert!(facts.filters_configured);
        assert_eq!(facts.hooks_path.as_deref(), Some(".githooks"));

        let mut other = ConfigFacts::default();
        assert!(scan_config(
            "[filter \"other\"]
	smudge = x
",
            &mut other
        ));
        assert!(!other.filters_configured);
        assert!(!scan_config(
            "[include]
	path = x
",
            &mut other
        ));
    }

    #[test]
    fn a_warm_lfs_status_spawns_no_git() {
        let dir = tempfile::tempdir().unwrap();
        let init = std::process::Command::new("git")
            .args(["init", "-q"])
            .arg(dir.path())
            .status()
            .unwrap();
        assert!(init.success());
        let repo = RepoRef::from_root(dir.path());
        let probe = LfsProbe::forced(LfsBinary::Missing);

        let before = crate::process::spawn_count();
        let status = lfs_status(&repo, &probe).unwrap();
        assert_eq!(
            crate::process::spawn_count(),
            before,
            "lfs_status spawned git"
        );
        assert!(!status.uses_lfs);
    }

    #[test]
    fn the_override_only_knows_missing() {
        assert_eq!(parse_override(Some("missing")), Some(LfsBinary::Missing));
        assert_eq!(parse_override(Some(" MISSING ")), Some(LfsBinary::Missing));
        assert_eq!(parse_override(Some("1")), None);
        assert_eq!(parse_override(None), None);
    }

    #[test]
    fn a_batch_answer_is_split_into_pointers_and_other_blobs() {
        let oid = "a".repeat(64);
        let pointer =
            format!("version https://git-lfs.github.com/spec/v1\noid sha256:{oid}\nsize 9\n");
        let mut raw = Vec::new();
        raw.extend_from_slice(format!("s1 blob {}\n{pointer}\n", pointer.len()).as_bytes());
        raw.extend_from_slice(b"s2 blob 5\nhello\n");
        raw.extend_from_slice(b"s3 missing\n");
        let parsed = parse_batch(&raw);
        assert_eq!(parsed["s1"].as_ref().unwrap().size, 9);
        assert_eq!(parsed["s2"], None);
        assert!(!parsed.contains_key("s3"));
    }
}
