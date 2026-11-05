use std::ffi::{OsStr, OsString};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::OnceLock;

use gitpoulpi_core::Result;

/// `-c key=value` overrides prepended to *every* git invocation.
///
/// The user's own git config must never change what this backend parses:
/// a global `diff.mnemonicPrefix = true` makes git emit `i/f.txt w/f.txt`
/// instead of `a/f.txt b/f.txt`, which breaks every diff and staging
/// operation.
///
/// Only output *shape* is pinned. Anything expressing what the user wants
/// git to do (signing, hooks, push/fetch defaults, autocrlf, credentials,
/// `insteadOf`) is left alone, which is why the global config is
/// overridden key by key rather than suppressed with `GIT_CONFIG_GLOBAL`.
///
/// `diff.external` is *not* in this list: `-c diff.external=` sets it to
/// the empty string and git then fails with "cannot run : No such file or
/// directory". Diff commands pass `--no-ext-diff` instead.
const CONFIG_OVERRIDES: &[&str] = &[
    // `color.ui = always` would inject ANSI escapes into output that is
    // parsed, not displayed.
    "color.ui=false",
    // Non-ASCII paths verbatim instead of `"\303\251.txt"` octal escapes.
    "core.quotepath=false",
    // The knobs that move diff path prefixes off `a/` and `b/`.
    "diff.mnemonicprefix=false",
    "diff.noprefix=false",
    "diff.srcprefix=a/",
    "diff.dstprefix=b/",
    // `log.showSignature = true` interleaves gpg output into `git log`.
    "log.showsignature=false",
    // Paths relative to the repo root, whatever the process' cwd is.
    "status.relativepaths=false",
    // Output is read back as UTF-8, so ask for UTF-8.
    "i18n.logoutputencoding=UTF-8",
];

/// Environment variables that must not leak in from the parent process.
///
/// `GIT_DIR` and friends are set by git itself when it runs a hook, so a
/// GitPoulpi launched from inside one would otherwise operate on whatever
/// repository that hook belonged to rather than on `repo_root`.
const CLEARED_ENV: &[&str] = &[
    "GIT_DIR",
    "GIT_WORK_TREE",
    "GIT_INDEX_FILE",
    "GIT_OBJECT_DIRECTORY",
    "GIT_ALTERNATE_OBJECT_DIRECTORIES",
    "GIT_COMMON_DIR",
    "GIT_PREFIX",
    "GIT_EXTERNAL_DIFF",
    "GIT_PAGER",
    // Inherited `-c` overrides; ours are passed as arguments, and git
    // rebuilds this variable from them for its own subprocesses.
    "GIT_CONFIG_PARAMETERS",
    "GIT_CONFIG_COUNT",
    // git-lfs knobs: a stray value in GitPoulpi's own environment must not
    // silently skip smudging or redirect progress in every child. The
    // operations that want them set them explicitly.
    "GIT_LFS_SKIP_SMUDGE",
    "GIT_LFS_PROGRESS",
];

/// Prepends the global flags and config overrides to a command's own
/// arguments. `--no-pager` is belt-and-braces - git already skips the
/// pager when stdout is a pipe - but costs nothing and covers
/// `GIT_PAGER`-style surprises.
fn full_args(args: &[&str]) -> Vec<String> {
    let mut out = Vec::with_capacity(args.len() + CONFIG_OVERRIDES.len() * 2 + 1);
    out.push("--no-pager".to_string());
    for kv in CONFIG_OVERRIDES {
        out.push("-c".to_string());
        out.push((*kv).to_string());
    }
    out.extend(args.iter().map(|a| (*a).to_string()));
    out
}

thread_local! {
    static SPAWNS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

/// Every `git` command built by any thread of this process (an always-on
/// atomic: one relaxed add per spawn, which is nothing next to a spawn).
static TOTAL_SPAWNS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

fn count_spawn() {
    SPAWNS.with(|c| c.set(c.get() + 1));
    TOTAL_SPAWNS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
}

/// How many `git` commands the whole process has built so far, on any
/// thread. Unlike the per-thread count it sees the daemon's worker threads, so
/// the daemon tests and `xtask perf` use it; callers that share a process
/// with other git users must serialise themselves to diff it.
pub fn total_spawn_count() -> u64 {
    TOTAL_SPAWNS.load(std::sync::atomic::Ordering::Relaxed)
}

/// How many `git` commands the calling thread has built so far (every spawn
/// goes through [`git`] or [`git_async`]). Per thread, so parallel tests do
/// not see each other; a test diffs it to assert a call spawns nothing.
#[cfg(all(test, feature = "lfs"))]
pub fn spawn_count() -> u64 {
    SPAWNS.with(std::cell::Cell::get)
}

/// The `git` executable every spawn uses, resolved once per process.
///
/// Order: the `GITPOULPI_GIT` environment variable; on Windows the real
/// `mingw64\bin\git.exe` when `git` on `PATH` is Git for Windows' `cmd\git.exe`
/// shim (the shim starts a second process, roughly doubling every spawn);
/// otherwise plain `git`, left to the OS `PATH` lookup.
pub fn git_program() -> &'static OsStr {
    static PROGRAM: OnceLock<OsString> = OnceLock::new();
    PROGRAM.get_or_init(resolve_git_program)
}

fn resolve_git_program() -> OsString {
    if let Some(p) = std::env::var_os("GITPOULPI_GIT").filter(|p| !p.is_empty()) {
        return p;
    }
    #[cfg(windows)]
    if let Some(real) = windows_unshimmed_git(std::env::var_os("PATH").as_deref()) {
        return real.into_os_string();
    }
    OsString::from("git")
}

/// Finds `git.exe` on `path`; when it sits in a `cmd` directory next to a
/// `mingw64\bin\git.exe`, returns that real binary instead.
#[cfg_attr(not(windows), allow(dead_code))]
fn windows_unshimmed_git(path: Option<&OsStr>) -> Option<PathBuf> {
    let path = path?;
    let shim = std::env::split_paths(path)
        .map(|d| d.join("git.exe"))
        .find(|c| c.is_file())?;
    let dir = shim.parent()?;
    if !dir.file_name()?.eq_ignore_ascii_case("cmd") {
        return None;
    }
    let real = dir.parent()?.join("mingw64").join("bin").join("git.exe");
    real.is_file().then_some(real)
}

/// `PATH` for children when the resolved git is a Git for Windows
/// `mingw64\bin\git.exe`: the shim put `mingw64\bin` and `usr\bin` on `PATH`
/// (hooks need `sh`, credential helpers and ssh live there), so the direct
/// binary must do it itself. `None` when nothing needs adding.
fn child_path() -> Option<&'static OsString> {
    static PATH_VAR: OnceLock<Option<OsString>> = OnceLock::new();
    PATH_VAR
        .get_or_init(|| {
            let prog = Path::new(git_program());
            let bin = prog.parent()?;
            if !bin.ends_with(Path::new("mingw64").join("bin")) {
                return None;
            }
            let root = bin.parent()?.parent()?;
            let mut paths = vec![bin.to_path_buf(), root.join("usr").join("bin")];
            paths.extend(std::env::split_paths(&std::env::var_os("PATH")?));
            std::env::join_paths(paths).ok()
        })
        .as_ref()
}

fn apply_git_env(cmd: &mut Command) {
    if let Some(p) = child_path() {
        cmd.env("PATH", p);
    }
}

/// How network commands get HTTPS credentials (`credentials.https_mode`).
/// Process-wide because the daemon owns the process and re-reads the
/// setting before each network operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CredentialMode {
    /// git's own handling: the configured helper, then its prompt.
    #[default]
    Helper,
    /// No helper; git may prompt on a terminal.
    Prompt,
    /// No helper and no prompt: anonymous access only.
    None,
}

static CREDENTIAL_MODE: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);

/// Sets the mode every later network command uses.
pub fn set_credential_mode(mode: CredentialMode) {
    CREDENTIAL_MODE.store(mode as u8, std::sync::atomic::Ordering::Relaxed);
}

pub fn credential_mode() -> CredentialMode {
    match CREDENTIAL_MODE.load(std::sync::atomic::Ordering::Relaxed) {
        1 => CredentialMode::Prompt,
        2 => CredentialMode::None,
        _ => CredentialMode::Helper,
    }
}

/// The global `-c` arguments and environment a mode adds.
fn credential_settings(mode: CredentialMode) -> (&'static [&'static str], Option<&'static str>) {
    match mode {
        CredentialMode::Helper => (&[], None),
        // An empty value resets the helper list.
        CredentialMode::Prompt => (&["-c", "credential.helper="], None),
        CredentialMode::None => (&["-c", "credential.helper="], Some("0")),
    }
}

/// The private (else public) key file `ssh.private_key_path` /
/// `ssh.public_key_path` name, process-wide like the credential mode: the
/// daemon re-reads the setting before each network operation.
static SSH_KEY: std::sync::RwLock<Option<String>> = std::sync::RwLock::new(None);

/// Sets the key file every later network command offers to ssh (`None`: leave
/// ssh to the agent and its own configuration). See [`ssh_key_command`].
pub fn set_ssh_key(path: Option<String>) {
    if let Ok(mut slot) = SSH_KEY.write() {
        *slot = path.filter(|p| !p.trim().is_empty());
    }
}

/// The `GIT_SSH_COMMAND` a network command needs to use the configured key,
/// or `None` when it must be left alone. Precedence, highest first - the
/// configured key only fills a gap, it never overrides:
///
/// 1. `GIT_SSH_COMMAND` / `GIT_SSH` already in the environment,
/// 2. `core.sshCommand` in the effective git config (the repository's own
///    pin from "SSH key for this repository", or a global one),
/// 3. the configured key (`ssh -i "<key>" -o IdentitiesOnly=yes`),
/// 4. nothing: ssh's own configuration and the ssh-agent.
fn ssh_key_command(repo_root: &Path) -> Option<String> {
    let key = SSH_KEY.read().ok()?.clone()?;
    let user_env = |name: &str| std::env::var_os(name).is_some_and(|v| !v.is_empty());
    if user_env("GIT_SSH_COMMAND") || user_env("GIT_SSH") {
        return None;
    }
    // Only asked when a key is configured, so nobody else pays the spawn.
    let pinned = git(repo_root, &["config", "--get", "core.sshCommand"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .is_some_and(|o| !String::from_utf8_lossy(&o.stdout).trim().is_empty());
    if pinned {
        return None;
    }
    Some(gitpoulpi_core::repo_config::ssh_command_for_key(
        &expand_home(&key),
    ))
}

/// `~/x` as the user's home plus `x` (ssh would, but the shell git runs the
/// command in does not expand a quoted tilde).
fn expand_home(path: &str) -> String {
    let rest = path.strip_prefix("~/").or_else(|| path.strip_prefix("~\\"));
    match (
        rest,
        std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")),
    ) {
        (Some(rest), Some(home)) => Path::new(&home).join(rest).display().to_string(),
        _ => path.to_string(),
    }
}

/// Builds a `git` command that is immune to the ambient git environment.
pub(crate) fn git(repo_root: &Path, args: &[&str]) -> Command {
    count_spawn();
    let mut cmd = Command::new(git_program());
    apply_git_env(&mut cmd);
    cmd.args(full_args(args).iter().map(OsStr::new));
    cmd.current_dir(repo_root);
    for key in CLEARED_ENV {
        cmd.env_remove(key);
    }
    cmd
}

/// Async twin of [`git`], for the network operations that run on tokio.
pub(crate) fn git_async(repo_root: &Path, args: &[&str]) -> tokio::process::Command {
    count_spawn();
    let mut cmd = tokio::process::Command::new(git_program());
    if let Some(p) = child_path() {
        cmd.env("PATH", p);
    }
    let (credential_args, prompt) = credential_settings(credential_mode());
    cmd.args(credential_args);
    if let Some(prompt) = prompt {
        cmd.env("GIT_TERMINAL_PROMPT", prompt);
    }
    if let Some(ssh) = ssh_key_command(repo_root) {
        cmd.env("GIT_SSH_COMMAND", ssh);
    }
    cmd.args(full_args(args).iter().map(OsStr::new));
    cmd.current_dir(repo_root);
    for key in CLEARED_ENV {
        cmd.env_remove(key);
    }
    cmd
}

/// Turns a failed git invocation's stderr into an error: the typed
/// [`gitpoulpi_core::Error::LfsUnavailable`] when git says an LFS filter or
/// hook could not run because `git-lfs` is missing, otherwise the plain
/// `Backend` error naming the command.
pub(crate) fn failed(args: &[&str], stderr: &str) -> gitpoulpi_core::Error {
    failed_with(args, None, stderr)
}

/// [`failed`] knowing the exit code. The error carries an
/// [`gitpoulpi_core::ErrorDetail`] (redacted command line, exit code and the
/// tail of stderr); its message is the same text as ever, with credentials
/// masked.
pub(crate) fn failed_with(
    args: &[&str],
    exit_code: Option<i32>,
    stderr: &str,
) -> gitpoulpi_core::Error {
    classify_lfs_failure(stderr).unwrap_or_else(|| {
        let message = gitpoulpi_core::redact(&format!("git {} failed: {}", args.join(" "), stderr));
        gitpoulpi_core::Error::git(
            message,
            gitpoulpi_core::ErrorDetail::from_git(args, exit_code, stderr),
        )
    })
}

/// Recognises the messages git prints when a repository is configured for
/// Git LFS (filters in config, or the hook in `.git/hooks`) but the
/// `git-lfs` program is not installed. Matched on fragments that git and
/// the shell have printed verbatim on every platform tried; each is
/// pinned against a recorded real message in the tests below.
pub fn classify_lfs_failure(stderr: &str) -> Option<gitpoulpi_core::Error> {
    const MISSING: &[&str] = &[
        // Linux / macOS: the filter or hook shell could not find it.
        "git-lfs: command not found",
        "git-lfs: not found",
        // `git lfs ...` when the subcommand is absent.
        "'lfs' is not a git command",
        // The pre-push / post-checkout hook shim git-lfs installs.
        "'git-lfs' was not found on your path",
        // Windows: CreateProcess failing on the filter command.
        "cannot run git-lfs",
    ];
    let lower = stderr.to_lowercase();
    MISSING
        .iter()
        .any(|needle| lower.contains(needle))
        .then_some(gitpoulpi_core::Error::LfsUnavailable {
            reason: gitpoulpi_core::domain::LfsUnavailableReason::Missing,
        })
}

pub(crate) fn spawn_failed(e: std::io::Error) -> gitpoulpi_core::Error {
    gitpoulpi_core::Error::Backend(format!("failed to spawn git: {e}"))
}

fn utf8_stdout(stdout: Vec<u8>) -> Result<String> {
    String::from_utf8(stdout)
        .map_err(|e| gitpoulpi_core::Error::Backend(format!("git output was not utf-8: {e}")))
}

/// `output` itself when git succeeded, else the [`failed`] error.
fn checked(args: &[&str], output: Output) -> Result<Output> {
    if output.status.success() {
        Ok(output)
    } else {
        Err(failed_with(
            args,
            output.status.code(),
            &String::from_utf8_lossy(&output.stderr),
        ))
    }
}

/// The [`StatusOutput`] of a run whose failure is an expected outcome;
/// only a missing `git-lfs` is still an `Err`.
fn status_of(output: &Output) -> Result<StatusOutput> {
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    if !output.status.success() {
        if let Some(e) = classify_lfs_failure(&stderr) {
            return Err(e);
        }
    }
    Ok(StatusOutput {
        success: output.status.success(),
        stderr,
        exit_code: output.status.code(),
    })
}

fn with_env(mut cmd: Command, env: &[(&str, &str)]) -> Command {
    for (key, value) in env {
        cmd.env(key, value);
    }
    cmd
}

fn join_writer(writer: std::thread::JoinHandle<std::io::Result<()>>) -> Result<()> {
    writer
        .join()
        .map_err(|_| gitpoulpi_core::Error::Backend("git stdin writer panicked".into()))?
        .map_err(|e| gitpoulpi_core::Error::Backend(format!("failed to write git stdin: {e}")))
}

/// Spawns `cmd` with `input` on its stdin and waits for it.
///
/// stdin is written from a thread of its own: `git cat-file --batch`
/// answers while it is still reading, and once both pipes are full a
/// writer that waits for its whole input to be taken before reading
/// anything would wait forever. The writer's result is returned for the
/// caller to check once the exit status has been.
fn run_piped_stdin(
    mut cmd: Command,
    input: Vec<u8>,
) -> Result<(Output, std::thread::JoinHandle<std::io::Result<()>>)> {
    let mut child = cmd
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(spawn_failed)?;
    let mut pipe = child.stdin.take().expect("stdin was piped");
    let writer = std::thread::spawn(move || pipe.write_all(&input));
    let output = child
        .wait_with_output()
        .map_err(|e| gitpoulpi_core::Error::Backend(format!("failed to wait for git: {e}")))?;
    Ok((output, writer))
}

/// Runs `git <args>` in `repo_root` and returns stdout as UTF-8. Every
/// SystemGitBackend method goes through this so error formatting and
/// working-directory handling stay consistent.
pub fn run(repo_root: &Path, args: &[&str]) -> Result<String> {
    let output = git(repo_root, args).output().map_err(spawn_failed)?;
    utf8_stdout(checked(args, output)?.stdout)
}

/// [`run`] with git's messages forced to English, for the one caller that
/// recognises a message of git's (`stash push`'s "No local changes to save").
pub fn run_in_english(repo_root: &Path, args: &[&str]) -> Result<String> {
    let output = git(repo_root, args)
        .env("LC_ALL", "C")
        .env("LANGUAGE", "C")
        .output()
        .map_err(spawn_failed)?;
    utf8_stdout(checked(args, output)?.stdout)
}

/// [`run`] for commands that may reach a remote (`submodule update` clones
/// and fetches): `credentials.https_mode` applies, as it does to the async
/// network commands.
#[cfg(feature = "submodule")]
pub fn run_network(repo_root: &Path, args: &[&str]) -> Result<String> {
    let (credential_args, prompt) = credential_settings(credential_mode());
    let mut all: Vec<&str> = credential_args.to_vec();
    all.extend_from_slice(args);
    let mut cmd = git(repo_root, &all);
    if let Some(prompt) = prompt {
        cmd.env("GIT_TERMINAL_PROMPT", prompt);
    }
    if let Some(ssh) = ssh_key_command(repo_root) {
        cmd.env("GIT_SSH_COMMAND", ssh);
    }
    let output = cmd.output().map_err(spawn_failed)?;
    utf8_stdout(checked(args, output)?.stdout)
}

/// Like [`run`], but hands back stdout as raw bytes.
///
/// `run` validates UTF-8, which is right for every machine-readable
/// *format* git produces but wrong for file content: `git cat-file blob`
/// on a PNG is not an error, it is a PNG. Callers decide what those bytes
/// are (see `gitpoulpi_core::domain::BlobContent::from_bytes`).
///
/// Still goes through [`git`], so the config pinning and the cleared
/// `GIT_DIR` family apply here exactly as everywhere else.
pub fn run_bytes(repo_root: &Path, args: &[&str]) -> Result<Vec<u8>> {
    let output = git(repo_root, args).output().map_err(spawn_failed)?;
    Ok(checked(args, output)?.stdout)
}

/// Runs `git <args>` in `repo_root`, writing `stdin` to the child process --
/// used for `git apply --cached [--reverse] -` when staging/unstaging a
/// single hunk or a subset of lines.
pub fn run_with_stdin(repo_root: &Path, args: &[&str], stdin: &str) -> Result<String> {
    run_with_env_and_stdin(repo_root, args, &[], stdin)
}

/// [`run_with_stdin`] with extra environment variables - for
/// `git commit-tree`, whose author identity and date can only be given
/// through `GIT_AUTHOR_*`.
///
/// stdin is written from its own thread, see [`run_piped_stdin`].
pub fn run_with_env_and_stdin(
    repo_root: &Path,
    args: &[&str],
    env: &[(&str, &str)],
    stdin: &str,
) -> Result<String> {
    let cmd = with_env(git(repo_root, args), env);
    let (output, writer) = run_piped_stdin(cmd, stdin.as_bytes().to_vec())?;
    let output = checked(args, output)?;
    join_writer(writer)?;
    utf8_stdout(output.stdout)
}

/// Result of a git invocation that may legitimately fail (e.g. a merge
/// stopping on conflicts) - callers inspect `success`/`stderr` themselves
/// instead of getting an `Err`.
pub struct StatusOutput {
    pub success: bool,
    pub stderr: String,
    pub exit_code: Option<i32>,
}

/// Like `run`, but never turns a non-zero exit into `Err` - used for
/// commands (merge, rebase, cherry-pick, revert) whose failure is an
/// expected outcome (conflicts) that callers turn into a typed `*Outcome`.
pub fn run_status(repo_root: &Path, args: &[&str]) -> Result<StatusOutput> {
    status_of(&git(repo_root, args).output().map_err(spawn_failed)?)
}

/// [`run_status`] with extra environment variables - interactive rebase
/// needs `GIT_SEQUENCE_EDITOR`/`GITPOULPI_TODO`/`GIT_EDITOR` set for the
/// duration of a single `git rebase` invocation.
pub fn run_status_with_env(
    repo_root: &Path,
    args: &[&str],
    env: &[(&str, &str)],
) -> Result<StatusOutput> {
    let output = with_env(git(repo_root, args), env)
        .output()
        .map_err(spawn_failed)?;
    status_of(&output)
}

/// Runs `git <args>` to its end and captures it, like `.output()`, but
/// under a process-tree guard: dropping the future (a canceled job) kills git
/// and every helper it started, and removes the lock files a hard kill leaves
/// (`cancel_cleanup`). Disarmed once git has ended.
async fn output_guarded(repo_root: &Path, args: &[&str]) -> Result<Output> {
    let mut cmd = git_async(repo_root, args);
    crate::tree_kill::captured(&mut cmd);
    let cleanup = crate::cancel_cleanup::for_command(repo_root, args);
    let (child, mut guard) = crate::tree_kill::spawn_guarded(&mut cmd).map_err(spawn_failed)?;
    let output = child.wait_with_output().await.map_err(spawn_failed)?;
    guard.disarm();
    if let Some(mut cleanup) = cleanup {
        cleanup.disarm();
    }
    Ok(output)
}

/// Async variant of `run`, for network operations (fetch/pull/push/clone)
/// that must not block the daemon's executor while they hit the network.
pub async fn run_async(repo_root: &Path, args: &[&str]) -> Result<String> {
    let output = output_guarded(repo_root, args).await?;
    utf8_stdout(checked(args, output)?.stdout)
}

/// Everything an invocation that may legitimately fail produced: exit
/// status, stdout *and* stderr.
pub struct FullOutput {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
}

/// Async, never turns a non-zero exit into `Err`, and keeps stdout --
/// for `git push --porcelain`, whose rejection report is on stdout and
/// arrives *with* a failing exit status.
pub async fn run_full_async(repo_root: &Path, args: &[&str]) -> Result<FullOutput> {
    let output = output_guarded(repo_root, args).await?;
    let status = status_of(&output)?;
    Ok(FullOutput {
        success: status.success,
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: status.stderr,
        exit_code: status.exit_code,
    })
}

/// [`run_full_async`] with git's messages forced to English, for callers that
/// recognise a refusal by its words (`push --atomic` on a server that lacks
/// it). Same credential and cancel behaviour.
pub(crate) async fn run_full_async_english(repo_root: &Path, args: &[&str]) -> Result<FullOutput> {
    let mut cmd = git_async(repo_root, args);
    cmd.env("LC_ALL", "C").env("LANGUAGE", "C");
    crate::tree_kill::captured(&mut cmd);
    let (child, mut guard) = crate::tree_kill::spawn_guarded(&mut cmd).map_err(spawn_failed)?;
    let output = child.wait_with_output().await.map_err(spawn_failed)?;
    guard.disarm();
    let status = status_of(&output)?;
    Ok(FullOutput {
        success: status.success,
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: status.stderr,
        exit_code: status.exit_code,
    })
}

/// Runs `git <args>` in `repo_root`, handing every *chunk* of stderr to
/// `on_stderr` as it arrives. Only `clone` uses this, for its progress
/// output.
///
/// - **Chunks, not lines.** git separates progress updates with `\r`, so a
///   line-oriented reader sees nothing until the phase ends.
/// - **stdout is piped and drained**: a child whose stdout pipe fills
///   blocks forever, and `git clone` on a large repository fills one.
pub async fn run_streaming_stderr<F>(
    repo_root: &Path,
    args: &[&str],
    mut on_stderr: F,
) -> Result<()>
where
    F: FnMut(&str) + Send,
{
    use tokio::io::AsyncReadExt;

    let mut cmd = git_async(repo_root, args);
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
    // Dropping this future (a canceled clone) kills git and its helpers.
    let (mut child, mut tree) = crate::tree_kill::spawn_guarded(&mut cmd).map_err(spawn_failed)?;

    let mut stdout = child.stdout.take().expect("stdout was piped");
    let drain_stdout = tokio::spawn(async move {
        let mut sink = Vec::new();
        let _ = stdout.read_to_end(&mut sink).await;
    });

    let mut stderr = child.stderr.take().expect("stderr was piped");
    let mut buf = [0u8; 4096];
    let mut tail = String::new();
    loop {
        match stderr.read(&mut buf).await {
            Ok(0) | Err(_) => break,
            Ok(n) => {
                let chunk = String::from_utf8_lossy(&buf[..n]);
                tail.push_str(&chunk);
                // Keep only the last few KiB: the whole point is the
                // *latest* state, and a clone of a big repository emits
                // megabytes of progress.
                if tail.len() > 8192 {
                    let cut = tail.len() - 4096;
                    tail = tail[tail
                        .char_indices()
                        .find(|(i, _)| *i >= cut)
                        .map_or(0, |(i, _)| i)..]
                        .to_string();
                }
                on_stderr(&chunk);
            }
        }
    }

    let status = child
        .wait()
        .await
        .map_err(|e| gitpoulpi_core::Error::Backend(format!("failed to wait for git: {e}")))?;
    let _ = drain_stdout.await;
    tree.disarm();

    if !status.success() {
        return Err(failed_with(args, status.code(), tail.trim()));
    }
    Ok(())
}

/// Async variant of `run_status`, for network operations whose failure
/// (e.g. a `pull` stopping on conflicts) is an expected outcome.
pub async fn run_status_async(repo_root: &Path, args: &[&str]) -> Result<StatusOutput> {
    let output = output_guarded(repo_root, args).await?;
    status_of(&output)
}

/// Runs `git <args>` and reports `(success, stdout)`, never turning a
/// non-zero exit into `Err`. For queries whose "no" is an exit status --
/// `git config --get` exits 1 for an unset key.
pub fn run_lenient(repo_root: &Path, args: &[&str]) -> Result<(bool, String)> {
    let output = git(repo_root, args).output().map_err(spawn_failed)?;
    Ok((
        output.status.success(),
        String::from_utf8_lossy(&output.stdout).into_owned(),
    ))
}

/// [`run_bytes`] with bytes written to the child's stdin, for
/// `git cat-file --batch`, whose answers are file content and so cannot
/// go through the UTF-8-checked [`run_with_stdin`].
///
/// stdin is written from its own thread, see [`run_piped_stdin`].
pub fn run_bytes_with_stdin(repo_root: &Path, args: &[&str], stdin: &[u8]) -> Result<Vec<u8>> {
    let (output, writer) = run_piped_stdin(git(repo_root, args), stdin.to_vec())?;
    let output = checked(args, output)?;
    join_writer(writer)?;
    Ok(output.stdout)
}

/// Like `run`, but for commands invoked with `-z` (NUL-delimited output).
pub fn run_z_fields(repo_root: &Path, args: &[&str]) -> Result<Vec<String>> {
    let raw = run(repo_root, args)?;
    Ok(raw
        .split('\0')
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect())
}

/// The installed git's `(major, minor)`, asked once per process
/// (`git --version`: `git version 2.45.2.windows.1`).
pub(crate) fn git_version() -> Option<(u32, u32)> {
    static VERSION: OnceLock<Option<(u32, u32)>> = OnceLock::new();
    *VERSION.get_or_init(|| {
        run(Path::new("."), &["--version"])
            .ok()
            .and_then(|v| parse_git_version(&v))
    })
}

/// `(major, minor)` out of `git version 2.45.2.windows.1`.
pub(crate) fn parse_git_version(text: &str) -> Option<(u32, u32)> {
    let mut parts = text.trim().strip_prefix("git version ")?.split('.');
    Some((parts.next()?.parse().ok()?, parts.next()?.parse().ok()?))
}

/// Whether the installed git is at least `(major, minor)`.
pub(crate) fn git_at_least(major: u32, minor: u32) -> bool {
    git_version().is_some_and(|v| v >= (major, minor))
}

/// What `git cat-file --batch` said about one object: its size, and its
/// bytes unless it was over the cap.
pub(crate) struct BatchObject {
    pub size: u64,
    pub bytes: Option<Vec<u8>>,
}

/// Reads the objects `oids` through **one** `git cat-file --batch`,
/// keeping at most `cap` bytes per object: a larger one is reported with
/// its size and its bytes are drained, never held, so a conflict on a
/// gigabyte file costs no memory. Asked one object at a time so the child
/// is never left mid-stream. A missing object is an error.
pub(crate) fn cat_file_batch_capped(
    repo_root: &Path,
    oids: &[String],
    cap: u64,
) -> Result<Vec<BatchObject>> {
    use std::io::{BufRead, BufReader, Read};

    let mut out = Vec::with_capacity(oids.len());
    if oids.is_empty() {
        return Ok(out);
    }
    let mut child = git(repo_root, &["cat-file", "--batch"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(spawn_failed)?;
    let mut stdin = child.stdin.take().expect("stdin was piped");
    let mut stdout = BufReader::new(child.stdout.take().expect("stdout was piped"));
    let io_err =
        |e: std::io::Error| gitpoulpi_core::Error::Backend(format!("git cat-file --batch: {e}"));
    for oid in oids {
        writeln!(stdin, "{oid}").map_err(io_err)?;
        stdin.flush().map_err(io_err)?;
        let mut header = String::new();
        stdout.read_line(&mut header).map_err(io_err)?;
        let mut words = header.split_whitespace();
        let size: u64 = match (words.next(), words.next(), words.next()) {
            (Some(_), Some("blob"), Some(size)) => size.parse().map_err(|_| {
                gitpoulpi_core::Error::Backend(format!("git cat-file gave `{}`", header.trim()))
            })?,
            _ => {
                return Err(gitpoulpi_core::Error::Backend(format!(
                    "git cat-file --batch: no blob for {oid} (`{}`)",
                    header.trim()
                )))
            }
        };
        let bytes = if size <= cap {
            let mut buf = vec![0u8; size as usize];
            stdout.read_exact(&mut buf).map_err(io_err)?;
            Some(buf)
        } else {
            std::io::copy(&mut (&mut stdout).take(size), &mut std::io::sink()).map_err(io_err)?;
            None
        };
        // The newline git prints after every object.
        let mut newline = [0u8; 1];
        stdout.read_exact(&mut newline).map_err(io_err)?;
        out.push(BatchObject { size, bytes });
    }
    drop(stdin);
    let _ = child.wait();
    Ok(out)
}

/// `git merge-file`, whose exit status is the number of conflicts (0 to
/// 127) and only a larger one is a failure. Returns stdout.
pub(crate) fn run_merge_file(repo_root: &Path, args: &[&str]) -> Result<Vec<u8>> {
    let output = git(repo_root, args).output().map_err(spawn_failed)?;
    match output.status.code() {
        Some(code) if (0..=127).contains(&code) => Ok(output.stdout),
        _ => Err(failed(args, &String::from_utf8_lossy(&output.stderr))),
    }
}

/// How long a failed read waits before its one retry.
const READ_RETRY_PAUSE: std::time::Duration = std::time::Duration::from_millis(100);

/// Runs a read, and once more after a short pause if it failed (the process
/// killed by a signal, or a non-zero exit while the repository was changing
/// under it). A second failure is the answer: the caller gets the error with
/// its detail (exit code, `None` for a signal, and stderr), never a retry
/// loop. Reads only: a mutation that failed must never run twice.
pub(crate) fn read_with_retry<T>(op: impl FnMut() -> Result<T>) -> Result<T> {
    retry_once(op, || std::thread::sleep(READ_RETRY_PAUSE))
}

/// The policy of [`read_with_retry`], with the pause injectable.
fn retry_once<T>(mut op: impl FnMut() -> Result<T>, pause: impl FnOnce()) -> Result<T> {
    match op() {
        Ok(value) => Ok(value),
        Err(_) => {
            pause();
            op()
        }
    }
}

#[cfg(test)]
mod retry_tests {
    use super::*;

    fn died_by_signal() -> gitpoulpi_core::Error {
        failed_with(&["for-each-ref"], None, "")
    }

    #[test]
    fn a_read_that_succeeds_runs_once() {
        let mut calls = 0;
        let mut paused = false;
        let out = retry_once(
            || {
                calls += 1;
                Ok("ok")
            },
            || paused = true,
        );
        assert_eq!(out.unwrap(), "ok");
        assert_eq!(calls, 1);
        assert!(!paused, "no pause without a failure");
    }

    #[test]
    fn a_read_killed_once_is_retried_and_succeeds() {
        let mut calls = 0;
        let out = retry_once(
            || {
                calls += 1;
                if calls == 1 {
                    Err(died_by_signal())
                } else {
                    Ok("ok")
                }
            },
            || {},
        );
        assert_eq!(out.unwrap(), "ok");
        assert_eq!(calls, 2);
    }

    #[test]
    fn a_read_that_fails_twice_returns_the_second_error_with_its_detail() {
        let mut calls = 0;
        let out: Result<()> = retry_once(
            || {
                calls += 1;
                Err(if calls == 1 {
                    failed_with(&["for-each-ref"], Some(128), "first")
                } else {
                    failed_with(&["for-each-ref"], None, "free(): double free detected")
                })
            },
            || {},
        );
        let err = out.unwrap_err();
        assert_eq!(calls, 2);
        let detail = err.detail().expect("detail");
        assert_eq!(detail.exit_code, None);
        assert!(detail.stderr.contains("double free"), "{}", detail.stderr);
    }
}

#[cfg(test)]
mod failure_detail_tests {
    use super::*;

    /// The message and the detail of a failed command never hold a password
    /// that the tool (or the caller's arguments) echoed.
    #[test]
    fn a_failed_command_hides_credentials_everywhere() {
        let err = failed_with(
            &["push", "https://me:topsecret@host/r.git"],
            Some(128),
            "fatal: Authentication failed for 'https://me:topsecret@host/r.git/'\n",
        );
        let all = format!("{err} {err:?}");
        assert!(!all.contains("topsecret"), "{all}");
        let detail = err.detail().expect("detail");
        assert_eq!(detail.exit_code, Some(128));
        assert_eq!(detail.operation, "push");
        assert!(detail.command.as_deref().unwrap().starts_with("git push"));
        assert!(detail.stderr.contains("Authentication failed"));
    }
}

#[cfg(test)]
mod lfs_classifier_tests {
    use super::*;
    use gitpoulpi_core::domain::LfsUnavailableReason;
    use gitpoulpi_core::Error;

    fn is_missing(stderr: &str) -> bool {
        matches!(
            classify_lfs_failure(stderr),
            Some(Error::LfsUnavailable {
                reason: LfsUnavailableReason::Missing
            })
        )
    }

    /// The three messages recorded from real git on machines without
    /// `git-lfs` (a repository whose config or hooks still call it).
    #[test]
    fn the_recorded_missing_program_messages_are_recognized() {
        assert!(is_missing(
            "git-lfs filter-process: git-lfs: command not found\nfatal: a.bin: smudge filter lfs failed"
        ));
        assert!(is_missing(
            "git: 'lfs' is not a git command. See 'git --help'."
        ));
        assert!(is_missing(
            "\nThis repository is configured for Git LFS but 'git-lfs' was not found on your path. \
             If you no longer wish to use Git LFS, remove this hook by deleting the 'pre-push' file in the hooks directory."
        ));
    }

    #[test]
    fn ordinary_failures_are_not_mistaken_for_a_missing_program() {
        assert!(!is_missing("fatal: not a git repository"));
        assert!(!is_missing(
            "error: pathspec 'lfs' did not match any file(s) known to git"
        ));
        assert!(!is_missing("remote: Git LFS: (0 of 1 files) 404 not found"));
    }

    #[test]
    fn a_failed_command_carries_the_typed_error_or_the_plain_one() {
        assert!(matches!(
            failed(&["checkout", "x"], "git-lfs: command not found"),
            Error::LfsUnavailable { .. }
        ));
        assert!(
            matches!(failed(&["checkout", "x"], "boom"), Error::Git { message, .. } if message.contains("checkout x failed: boom"))
        );
    }
}

#[cfg(test)]
mod resolve_tests {
    use super::*;

    #[test]
    fn cmd_shim_resolves_to_the_mingw_binary_next_to_it() {
        let root = tempfile::tempdir().unwrap();
        let cmd = root.path().join("cmd");
        let bin = root.path().join("mingw64").join("bin");
        std::fs::create_dir_all(&cmd).unwrap();
        std::fs::create_dir_all(&bin).unwrap();
        std::fs::write(cmd.join("git.exe"), "").unwrap();
        let path = std::env::join_paths([cmd.clone()]).unwrap();
        // No real binary yet: keep the shim (None means "use plain git").
        assert_eq!(windows_unshimmed_git(Some(&path)), None);
        std::fs::write(bin.join("git.exe"), "").unwrap();
        assert_eq!(
            windows_unshimmed_git(Some(&path)),
            Some(bin.join("git.exe"))
        );
        // A git that is not in a `cmd` directory is left alone.
        let path = std::env::join_paths([bin]).unwrap();
        assert_eq!(windows_unshimmed_git(Some(&path)), None);
    }
}

#[cfg(test)]
mod stdin_writer_tests {
    use super::*;

    #[test]
    fn a_stdin_the_child_never_reads_is_reported_not_swallowed() {
        let dir = tempfile::tempdir().unwrap();
        // `git --version` exits 0 without reading stdin; more than a pipe
        // buffer of input makes the writer thread fail with a broken pipe.
        let input = vec![b'x'; 4 << 20];
        assert!(run_bytes_with_stdin(dir.path(), &["--version"], &input).is_err());
        assert!(run_with_stdin(dir.path(), &["--version"], &"x".repeat(4 << 20)).is_err());
    }
}
