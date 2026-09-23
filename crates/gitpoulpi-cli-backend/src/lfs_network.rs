//! `git lfs fetch | pull | push` with progress.
//!
//! When `GIT_LFS_PROGRESS` names a file, git-lfs *appends* lines of
//! `<direction> <file>/<files> <bytes>/<bytes> <name>` to it. GitPoulpi
//! points it at a temporary file, tails it every 100 ms while the program
//! runs, and turns each line into an [`LfsProgress`]; no human-readable
//! text is parsed.

use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use gitpoulpi_core::domain::{
    LfsDirection, LfsFetchOptions, LfsProgress, LfsProgressSink, LfsPushOptions, RepoRef,
};
use gitpoulpi_core::{Error, Result};
use tokio::io::AsyncReadExt;

use crate::lfs::LfsProbe;
use crate::process::{failed, git_async, run};

/// How often the progress file is read while the program runs.
const POLL_INTERVAL: Duration = Duration::from_millis(100);

/// Parse one line of the `GIT_LFS_PROGRESS` file.
///
/// `download 1/3 4096/12288 path/to/file.bin`: direction, file counter,
/// byte counter, then the file's name (which may contain spaces and is
/// ignored). Anything else - a blank line, a line cut short - is `None`.
pub fn parse_lfs_progress(line: &str) -> Option<LfsProgress> {
    let mut parts = line.split_whitespace();
    let direction = match parts.next()? {
        "download" => LfsDirection::Download,
        "upload" => LfsDirection::Upload,
        "checkout" => LfsDirection::Checkout,
        _ => return None,
    };
    let (files_done, files_total) = split_pair(parts.next()?)?;
    let (bytes_done, bytes_total) = split_pair(parts.next()?)?;
    // A name is always present in real output; requiring it rejects a
    // line that was read half-written.
    parts.next()?;
    Some(LfsProgress {
        direction,
        files_done,
        files_total,
        bytes_done,
        bytes_total,
        done: false,
    })
}

fn split_pair(token: &str) -> Option<(u64, u64)> {
    let (a, b) = token.split_once('/')?;
    Some((a.parse().ok()?, b.parse().ok()?))
}

/// Tails the progress file and feeds the sink, keeping every direction's
/// numbers from ever going backwards (git-lfs re-reports files while it
/// retries, and a progress bar that moves back is worse than none).
struct Tail<'a> {
    path: PathBuf,
    offset: u64,
    partial: String,
    latest: Option<LfsProgress>,
    per_direction: Vec<LfsProgress>,
    sink: LfsProgressSink<'a>,
}

impl<'a> Tail<'a> {
    fn new(path: PathBuf, sink: LfsProgressSink<'a>) -> Self {
        Self {
            path,
            offset: 0,
            partial: String::new(),
            latest: None,
            per_direction: Vec::new(),
            sink,
        }
    }

    /// Reads whatever was appended since the last call. `std::fs` opens
    /// with full sharing on Windows, so this is safe while git-lfs still
    /// has the file open for writing.
    fn poll(&mut self) {
        let Ok(mut file) = std::fs::File::open(&self.path) else {
            return;
        };
        if file.seek(SeekFrom::Start(self.offset)).is_err() {
            return;
        }
        let mut chunk = Vec::new();
        if file.read_to_end(&mut chunk).is_err() || chunk.is_empty() {
            return;
        }
        self.offset += chunk.len() as u64;
        self.partial.push_str(&String::from_utf8_lossy(&chunk));
        while let Some(end) = self.partial.find('\n') {
            let line: String = self.partial.drain(..=end).collect();
            if let Some(parsed) = parse_lfs_progress(line.trim_end()) {
                self.report(parsed);
            }
        }
    }

    fn report(&mut self, mut progress: LfsProgress) {
        match self
            .per_direction
            .iter_mut()
            .find(|p| p.direction == progress.direction)
        {
            Some(prev) => {
                progress.files_done = progress.files_done.max(prev.files_done);
                progress.files_total = progress.files_total.max(prev.files_total);
                progress.bytes_done = progress.bytes_done.max(prev.bytes_done);
                progress.bytes_total = progress.bytes_total.max(prev.bytes_total);
                *prev = progress;
            }
            None => self.per_direction.push(progress),
        }
        self.latest = Some(progress);
        (self.sink)(progress);
    }

    /// The last report of a successful run: what was last seen, marked
    /// complete (or an empty completed report when nothing was
    /// transferred).
    fn finish(mut self, default_direction: LfsDirection) {
        self.poll();
        let mut last = self.latest.unwrap_or(LfsProgress {
            direction: default_direction,
            files_done: 0,
            files_total: 0,
            bytes_done: 0,
            bytes_total: 0,
            done: false,
        });
        last.files_done = last.files_total;
        last.bytes_done = last.bytes_total;
        last.done = true;
        (self.sink)(last);
    }
}

fn progress_file() -> PathBuf {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    std::env::temp_dir().join(format!(
        "gitpoulpi-lfs-progress-{}-{}.log",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    ))
}

/// Runs `git <args>` (which must start with `lfs`) with progress tailing.
async fn run_with_progress(
    root: &Path,
    args: &[&str],
    default_direction: LfsDirection,
    progress: LfsProgressSink<'_>,
) -> Result<()> {
    let file = progress_file();
    std::fs::write(&file, b"")?;

    let mut cmd = git_async(root, args);
    cmd.env("GIT_LFS_PROGRESS", &file)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    // Dropping this future (a canceled transfer) kills git-lfs and its
    // helpers, and removes the `lfs/tmp` files it left.
    let mut cleanup = crate::cancel_cleanup::for_command(root, args);
    let spawned = crate::tree_kill::spawn_guarded(&mut cmd);
    let (mut child, mut tree) = match spawned {
        Ok(pair) => pair,
        Err(e) => {
            let _ = std::fs::remove_file(&file);
            return Err(Error::Backend(format!("failed to spawn git: {e}")));
        }
    };
    let mut stderr = child.stderr.take().expect("stderr was piped");
    let stderr_text = tokio::spawn(async move {
        let mut text = String::new();
        let _ = stderr.read_to_string(&mut text).await;
        text
    });

    let mut tail = Tail::new(file.clone(), progress);
    let status = loop {
        tokio::select! {
            status = child.wait() => break status,
            _ = tokio::time::sleep(POLL_INTERVAL) => tail.poll(),
        }
    };
    let stderr_text = stderr_text.await.unwrap_or_default();
    tree.disarm();
    if let Some(cleanup) = cleanup.as_mut() {
        cleanup.disarm();
    }

    let outcome = match status {
        Ok(s) if s.success() => {
            tail.finish(default_direction);
            Ok(())
        }
        Ok(_) => Err(failed(args, stderr_text.trim())),
        Err(e) => Err(Error::Backend(format!("failed to wait for git: {e}"))),
    };
    let _ = std::fs::remove_file(&file);
    outcome
}

/// See `GitBackend::lfs_fetch`.
pub async fn lfs_fetch(
    repo: &RepoRef,
    probe: &LfsProbe,
    remote: &str,
    opts: &LfsFetchOptions,
    progress: LfsProgressSink<'_>,
) -> Result<()> {
    probe.require(&repo.root)?;
    let mut args = vec!["lfs", "fetch", remote];
    if opts.all_refs {
        args.push("--all");
    }
    run_with_progress(&repo.root, &args, LfsDirection::Download, progress).await
}

/// See `GitBackend::lfs_pull`.
pub async fn lfs_pull(
    repo: &RepoRef,
    probe: &LfsProbe,
    remote: &str,
    progress: LfsProgressSink<'_>,
) -> Result<()> {
    probe.require(&repo.root)?;
    // Without the filters `git lfs pull` fetches but *skips the checkout*
    // ("Git LFS is not installed for this repository"), and a later
    // checkout would write pointers again. Pulling means the content is
    // wanted, so install them for this repository only.
    if !crate::lfs::filters_configured(repo)? {
        run(&repo.root, &["lfs", "install", "--local"])?;
    }
    run_with_progress(
        &repo.root,
        &["lfs", "pull", remote],
        LfsDirection::Download,
        progress,
    )
    .await
}

/// See `GitBackend::lfs_checkout`.
pub async fn lfs_checkout(
    repo: &RepoRef,
    probe: &LfsProbe,
    paths: &[PathBuf],
    progress: LfsProgressSink<'_>,
) -> Result<()> {
    probe.require(&repo.root)?;
    // Same precondition as `lfs_pull`: expanding pointers means the
    // content is wanted, so the repository's own filters go in first.
    if !crate::lfs::filters_configured(repo)? {
        run(&repo.root, &["lfs", "install", "--local"])?;
    }
    let paths: Vec<String> = paths
        .iter()
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .collect();
    let mut args = vec!["lfs", "checkout"];
    if !paths.is_empty() {
        args.push("--");
        args.extend(paths.iter().map(String::as_str));
    }
    run_with_progress(&repo.root, &args, LfsDirection::Checkout, progress).await
}

/// See `GitBackend::lfs_push`.
pub async fn lfs_push(
    repo: &RepoRef,
    probe: &LfsProbe,
    remote: &str,
    opts: &LfsPushOptions,
    progress: LfsProgressSink<'_>,
) -> Result<()> {
    probe.require(&repo.root)?;
    // `git lfs push` wants either `--all` or at least one ref.
    let branch;
    let mut args = vec!["lfs", "push", remote];
    if opts.all_refs {
        args.push("--all");
    } else {
        branch = run(&repo.root, &["rev-parse", "--abbrev-ref", "HEAD"])?
            .trim()
            .to_string();
        args.push(&branch);
    }
    run_with_progress(&repo.root, &args, LfsDirection::Upload, progress).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    #[test]
    fn real_lines_are_parsed() {
        let p = parse_lfs_progress("download 1/3 4096/12288 assets/big file.bin").unwrap();
        assert_eq!(p.direction, LfsDirection::Download);
        assert_eq!((p.files_done, p.files_total), (1, 3));
        assert_eq!((p.bytes_done, p.bytes_total), (4096, 12288));
        assert!(!p.done);

        let p = parse_lfs_progress("upload 2/2 100/100 a.psd").unwrap();
        assert_eq!(p.direction, LfsDirection::Upload);
        let p = parse_lfs_progress("checkout 1/1 30/30 x.bin").unwrap();
        assert_eq!(p.direction, LfsDirection::Checkout);
    }

    #[test]
    fn garbage_and_cut_off_lines_are_ignored() {
        for line in [
            "",
            "hello world",
            "download",
            "download 1/3",
            "download 1/3 100/200", // no name yet: written half-way
            "download x/3 1/2 f",
            "sideways 1/1 1/1 f",
        ] {
            assert_eq!(parse_lfs_progress(line), None, "{line:?}");
        }
    }

    #[test]
    fn the_tail_holds_back_a_partial_last_line_and_never_goes_backwards() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("progress.log");
        std::fs::write(
            &path,
            "download 1/2 100/300 a\ndownload 1/2 50/300 a\ndownload 2/2 20",
        )
        .unwrap();

        let seen = Mutex::new(Vec::new());
        let sink = |p: LfsProgress| seen.lock().unwrap().push(p);
        let mut tail = Tail::new(path.clone(), &sink);
        tail.poll();
        {
            let seen = seen.lock().unwrap();
            assert_eq!(seen.len(), 2, "the half-written third line waits");
            assert_eq!(seen[1].bytes_done, 100, "50 < 100 is clamped up");
        }

        // The rest of the third line arrives.
        use std::io::Write;
        let mut f = std::fs::OpenOptions::new()
            .append(true)
            .open(&path)
            .unwrap();
        f.write_all(b"0/300 b\n").unwrap();
        tail.poll();
        assert_eq!(seen.lock().unwrap().len(), 3);
        assert_eq!(seen.lock().unwrap()[2].bytes_done, 200);

        tail.finish(LfsDirection::Download);
        let seen = seen.lock().unwrap();
        let last = seen.last().unwrap();
        assert!(last.done);
        assert_eq!(last.bytes_done, last.bytes_total);
    }

    #[test]
    fn a_run_that_transferred_nothing_still_finishes() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("empty.log");
        std::fs::write(&path, "").unwrap();
        let seen = Mutex::new(Vec::new());
        let sink = |p: LfsProgress| seen.lock().unwrap().push(p);
        Tail::new(path, &sink).finish(LfsDirection::Upload);
        let seen = seen.lock().unwrap();
        assert_eq!(seen.len(), 1);
        assert!(seen[0].done);
        assert_eq!(seen[0].direction, LfsDirection::Upload);
    }
}
