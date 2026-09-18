//! What the repository overlay knows about each repository, learned off the
//! input loop.
//!
//! The overlay wants a branch, a dirty marker and an ahead/behind count per
//! row. Each costs a `git` call against a *different* repository than the
//! one this window shows, so none can come from the session's daemon and
//! none may run on the render thread (`docs/developer/where-we-are.md`:
//! **never block the input loop on a git subprocess call**).
//!
//! So: a few threads ([`WORKERS`]), one `git status --porcelain=v2 --branch`
//! per row, handed out in the order of the list, results streamed back on an `std::sync::mpsc` channel that the render
//! loop drains with `try_recv` like [`crate::clone_task`]. A row is drawn
//! the moment the overlay opens, with `facts: None` meaning *still
//! looking*, and fills in when its answer arrives.
//!
//! The thread is fire-and-forget. Closing the overlay drops the [`Probe`],
//! the channel's receiver goes with it, and the next send fails, which stops
//! the walk. Nothing joins it, because a UI that waits for a subprocess to
//! notice it is unwanted is the bug this module avoids.

use crate::redraw::NotifySend as _;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{mpsc, Mutex};

use crate::repo_switcher::RepoFacts;

/// A probe in flight. Dropping it asks the thread to stop.
pub struct Probe {
    rx: mpsc::Receiver<(PathBuf, RepoFacts)>,
}

impl Probe {
    /// Every answer that has arrived since the last call. Never blocks.
    pub fn drain(&self) -> Vec<(PathBuf, RepoFacts)> {
        std::iter::from_fn(|| self.rx.try_recv().ok()).collect()
    }
}

/// What the last probe of each repository found, for the whole process.
///
/// The Repository Management tab is a window of its own, so every visit is
/// a fresh `run_in_repo`; without this every visit would redraw every row
/// as *still looking* and wait on a subprocess per row again. With it the
/// rows come up with what was learned last time and the background walk
/// only *refreshes* them. A branch name half a second stale is honest; a
/// wall of placeholders on every visit is not useful.
static CACHE: Mutex<Option<HashMap<PathBuf, RepoFacts>>> = Mutex::new(None);

/// The facts last learned about `path`, if any. Never waits on git --
/// only on the cache's own lock, held for one map lookup.
pub fn cached(path: &Path) -> Option<RepoFacts> {
    CACHE
        .lock()
        .ok()
        .and_then(|cache| cache.as_ref().and_then(|map| map.get(path).cloned()))
}

fn remember(path: &Path, facts: &RepoFacts) {
    if let Ok(mut cache) = CACHE.lock() {
        cache
            .get_or_insert_with(HashMap::new)
            .insert(path.to_path_buf(), facts.clone());
    }
}

/// Repositories probed at the same time. A `git status` is mostly waiting on
/// the disk, so a handful in parallel turns a walk over twenty repositories
/// from the sum of their costs into about a quarter of it.
const WORKERS: usize = 4;

/// Starts probing `paths`, in the order given - which is the order the
/// list is drawn in, so the rows the user is looking at fill first.
pub fn start(paths: Vec<PathBuf>) -> Probe {
    start_with(paths, WORKERS, probe)
}

fn start_with(
    paths: Vec<PathBuf>,
    workers: usize,
    probe: impl Fn(&Path) -> Option<RepoFacts> + Send + Sync + 'static,
) -> Probe {
    let (tx, rx) = mpsc::channel();
    let paths = std::sync::Arc::new(paths);
    let probe = std::sync::Arc::new(probe);
    let next = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    for _ in 0..workers.min(paths.len()).max(1) {
        let (tx, paths, probe, next) = (
            tx.clone(),
            std::sync::Arc::clone(&paths),
            std::sync::Arc::clone(&probe),
            std::sync::Arc::clone(&next),
        );
        std::thread::spawn(move || loop {
            let i = next.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            let Some(path) = paths.get(i) else { return };
            let Some(facts) = probe(path) else { continue };
            remember(path, &facts);
            // A closed receiver means the overlay is gone. Stop rather
            // than finish a walk nobody will read.
            if tx.send_notify((path.clone(), facts)).is_err() {
                return;
            }
        });
    }
    Probe { rx }
}

/// One repository's facts, or `None` if `git` could not answer - which
/// includes "this is not a repository any more". A row whose probe
/// fails keeps saying *still looking*, which is honest: the overlay
/// decides availability from the filesystem, not from here.
fn probe(path: &Path) -> Option<RepoFacts> {
    let out = Command::new("git")
        .arg("-C")
        .arg(path)
        .args([
            "status",
            "--porcelain=v2",
            "--branch",
            // Untracked files are counted (the `+ n` counter), but a
            // wholly untracked directory is one entry, not a walk.
            "--untracked-files=normal",
            "--no-renames",
        ])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&out.stdout);
    let mut facts = parse_status(&stdout);
    facts.owner = owner_of(path, upstream_of(&stdout).as_deref());
    Some(facts)
}

/// The owner/namespace to show for the repository at `root`, read off its
/// default remote's URL (the remote chosen by the ladder every other part
/// of the app uses, `resolve_default_remote`). A file read, not a `git
/// config` subprocess: this runs once per row.
fn owner_of(root: &Path, upstream: Option<&str>) -> Option<String> {
    let config = std::fs::read_to_string(git_config_path(root)?).ok()?;
    let remotes = parse_remote_urls(&config);
    let names: Vec<String> = remotes.iter().map(|(n, _)| n.clone()).collect();
    let upstream_remote = upstream.and_then(|u| crate::upstream_remote(u, &names));
    let chosen = crate::load_chosen_default_remote(root);
    let remote = gitpoulpi_session::resolve_default_remote(
        chosen.as_deref(),
        upstream_remote.as_deref(),
        &names,
    )?;
    let url = remotes.into_iter().find(|(n, _)| *n == remote)?.1;
    owner_from_url(&url)
}

/// `# branch.upstream origin/main` -> `origin/main`.
fn upstream_of(stdout: &str) -> Option<String> {
    stdout
        .lines()
        .find_map(|l| l.strip_prefix("# branch.upstream "))
        .map(|u| u.trim().to_string())
}

/// Where the repository at `root` keeps its `config`.
///
/// An ordinary repository has it in `.git/`; a linked worktree's `.git`
/// is a *file* naming its private git dir, whose `commondir` points back
/// at the shared one, where `config` lives; a bare repository has it at
/// its root.
pub(crate) fn git_config_path(root: &Path) -> Option<PathBuf> {
    let dot_git = root.join(".git");
    if dot_git.is_dir() {
        return Some(dot_git.join("config"));
    }
    if dot_git.is_file() {
        let pointer = std::fs::read_to_string(&dot_git).ok()?;
        let gitdir = root.join(pointer.trim().strip_prefix("gitdir:")?.trim());
        let common = match std::fs::read_to_string(gitdir.join("commondir")) {
            Ok(rel) => gitdir.join(rel.trim()),
            Err(_) => gitdir,
        };
        return Some(common.join("config"));
    }
    let bare = root.join("config");
    bare.is_file().then_some(bare)
}

/// Every `[remote "name"]` section's `url`, in file order.
///
/// Deliberately small: section headers and `url` keys, which is all the
/// owner column needs. Includes and `insteadOf` rewrites are not followed;
/// the worst that does is show the parent folder's name instead of an owner.
pub fn parse_remote_urls(config: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut current: Option<String> = None;
    for line in config.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            current = line
                .strip_prefix("[remote \"") // i18n-english
                .and_then(|rest| rest.strip_suffix("\"]"))
                .map(str::to_string);
            continue;
        }
        let Some(name) = &current else { continue };
        if let Some((key, value)) = line.split_once('=') {
            if key.trim().eq_ignore_ascii_case("url") {
                out.push((name.clone(), value.trim().trim_matches('"').to_string()));
            }
        }
    }
    out
}

/// The owner or namespace in a remote URL: everything between the host and
/// the repository name.
///
/// `https://github.com/Poulpi-Lab/GitPoulpi.git` -> `Poulpi-Lab`,
/// `git@gitlab.example.com:group/sub/proj.git` -> `group/sub`. A local path
/// (a remote on disk, `file://`) has no owner: `None`, and the caller shows
/// the parent folder instead.
pub fn owner_from_url(url: &str) -> Option<String> {
    let url = url.trim();
    let path = if let Some((scheme, rest)) = url.split_once("://") {
        if scheme.eq_ignore_ascii_case("file") {
            return None;
        }
        // Past the host, and any `user@` or `:port` on it.
        rest.split_once('/')?.1
    } else {
        // scp-like `user@host:path`. A Windows drive (`C:\x`, `C:/x`)
        // has a colon one character in; that is a local path.
        let (host, path) = url.split_once(':')?;
        if host.len() <= 1 || host.contains(['/', '\\']) {
            return None;
        }
        path
    };
    let (owner, _repo) = path.trim_matches('/').rsplit_once('/')?;
    let owner = owner.trim_matches('/');
    (!owner.is_empty()).then(|| owner.to_string())
}

/// Parses `git status --porcelain=v2 --branch`.
///
/// A pure function of the output, so the format's two subtleties are
/// testable without a repository: the header lines all start with `# `, and
/// **anything that is not a header is a change** (ordinary `1`/`2`,
/// unmerged `u` and untracked `?` alike). Matching only on `1 ` would call a
/// repository with nothing but untracked files clean.
pub fn parse_status(stdout: &str) -> RepoFacts {
    let mut facts = RepoFacts::default();
    for line in stdout.lines() {
        if let Some(rest) = line.strip_prefix("# branch.head ") {
            let head = rest.trim();
            // git spells a detached HEAD `(detached)`, which is a state
            // rather than a branch name -- but it is still what the
            // user needs to see in the Branch column.
            facts.branch = Some(head.to_string());
        } else if let Some(rest) = line.strip_prefix("# branch.ab ") {
            for token in rest.split_whitespace() {
                let mut chars = token.chars();
                let sign = chars.next();
                // `chars.as_str()`, never `token[1..]`: a byte index
                // into a string that could hold anything is a panic
                // waiting for the one machine whose git says something
                // unexpected here.
                match (sign, chars.as_str().parse::<u32>()) {
                    (Some('+'), Ok(n)) => facts.ahead = n,
                    (Some('-'), Ok(n)) => facts.behind = n,
                    _ => {}
                }
            }
        } else if !line.starts_with('#') && !line.trim().is_empty() {
            facts.dirty = true;
            // `? path` is untracked and `1 A...` is new in the index:
            // both are *added*. Every other entry -- an ordinary change,
            // a rename, an unmerged path -- touches a tracked file.
            if line.starts_with("? ") || line.starts_with("1 A") {
                facts.added += 1;
            } else if !line.starts_with("! ") {
                facts.modified += 1;
            }
        }
    }
    facts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_probe_runs_several_repositories_at_once_and_never_more_than_asked() {
        use std::sync::atomic::{AtomicUsize, Ordering::SeqCst};
        use std::sync::Arc;
        use std::time::{Duration, Instant};
        let live = Arc::new(AtomicUsize::new(0));
        let peak = Arc::new(AtomicUsize::new(0));
        let (l, p) = (Arc::clone(&live), Arc::clone(&peak));
        let paths: Vec<PathBuf> = (0..12)
            .map(|i| PathBuf::from(format!("/probe-test/{i}")))
            .collect();
        let probe = start_with(paths, 3, move |_| {
            let now = l.fetch_add(1, SeqCst) + 1;
            p.fetch_max(now, SeqCst);
            // Hold the slot until a second probe is seen (or give up): run
            // one at a time, this never gets there.
            let give_up = Instant::now() + Duration::from_secs(2);
            while p.load(SeqCst) < 2 && Instant::now() < give_up {
                std::thread::sleep(Duration::from_millis(1));
            }
            std::thread::sleep(Duration::from_millis(5));
            l.fetch_sub(1, SeqCst);
            Some(parse_status("# branch.head main\n"))
        });
        let mut seen = 0;
        let give_up = Instant::now() + Duration::from_secs(10);
        while seen < 12 && Instant::now() < give_up {
            seen += probe.drain().len();
            std::thread::sleep(Duration::from_millis(5));
        }
        assert_eq!(seen, 12, "every repository answers");
        let peak = peak.load(SeqCst);
        assert!((2..=3).contains(&peak), "peak {peak}");
    }

    #[test]
    fn a_clean_repository_reports_its_branch_and_nothing_else() {
        let facts = parse_status(
            "# branch.oid abc123\n# branch.head main\n# branch.upstream origin/main\n\
             # branch.ab +0 -0\n",
        );
        assert_eq!(facts.branch.as_deref(), Some("main"));
        assert!(!facts.dirty);
        assert_eq!((facts.ahead, facts.behind), (0, 0));
    }

    #[test]
    fn ahead_and_behind_are_read_off_the_branch_ab_header() {
        let facts = parse_status("# branch.head side\n# branch.ab +3 -12\n");
        assert_eq!((facts.ahead, facts.behind), (3, 12));
    }

    /// The failure this pins: a parser that only recognises `1 ` entries
    /// calls a repository with nothing but untracked files clean, and
    /// the dirty marker then means something other than what it says.
    #[test]
    fn untracked_and_unmerged_entries_both_count_as_dirty() {
        for entry in [
            "1 .M N... 100644 100644 100644 abc abc src/lib.rs",
            "u UU N... 100644 100644 100644 100644 a b c d conflict.rs",
            "? whatever.txt",
        ] {
            let facts = parse_status(&format!("# branch.head main\n{entry}\n"));
            assert!(facts.dirty, "{entry} should be dirty");
        }
    }

    /// The WIP counters: `✎ n` changed tracked files, `+ n`
    /// added ones. An untracked file and a file new in the index are
    /// both *added*; a modification, a deletion, a rename and a conflict
    /// are all *modified*.
    #[test]
    fn wip_counters_split_added_from_modified() {
        let facts = parse_status(
            "# branch.head main\n\
             1 .M N... 100644 100644 100644 abc abc src/lib.rs\n\
             1 D. N... 100644 000000 000000 abc 000 gone.rs\n\
             1 A. N... 000000 100644 100644 000 abc new_staged.rs\n\
             u UU N... 100644 100644 100644 100644 a b c d conflict.rs\n\
             ? untracked.txt\n\
             ? other.txt\n",
        );
        assert_eq!((facts.modified, facts.added), (3, 3), "{facts:?}");
        assert!(facts.dirty);
    }

    #[test]
    fn the_owner_is_read_off_every_common_url_shape() {
        for (url, owner) in [
            (
                "https://github.com/Poulpi-Lab/GitPoulpi.git",
                Some("Poulpi-Lab"),
            ),
            (
                "https://github.com/Poulpi-Lab/GitPoulpi",
                Some("Poulpi-Lab"),
            ),
            (
                "git@github.com:Poulpi-Lab/GitPoulpi.git",
                Some("Poulpi-Lab"),
            ),
            (
                "ssh://git@gitlab.example.com:2222/group/sub/proj.git",
                Some("group/sub"),
            ),
            (
                "git@gitlab.example.com:group/sub/proj.git",
                Some("group/sub"),
            ),
            ("https://host/proj.git", None),
            ("C:\\repos\\remote.git", None),
            ("C:/repos/remote.git", None),
            ("/srv/git/remote.git", None),
            ("file:///srv/git/team/remote.git", None),
        ] {
            assert_eq!(owner_from_url(url).as_deref(), owner, "{url}");
        }
    }

    #[test]
    fn remote_urls_are_read_from_a_git_config_file() {
        let config = "[core]\n\trepositoryformatversion = 0\n\
                      [remote \"origin\"]\n\turl = git@github.com:me/a.git\n\
                      \tfetch = +refs/heads/*:refs/remotes/origin/*\n\
                      [branch \"main\"]\n\tremote = origin\n\
                      [remote \"up\"]\n\turl = https://github.com/them/a.git\n";
        assert_eq!(
            parse_remote_urls(config),
            vec![
                ("origin".to_string(), "git@github.com:me/a.git".to_string()),
                (
                    "up".to_string(),
                    "https://github.com/them/a.git".to_string()
                ),
            ]
        );
    }

    #[test]
    fn a_detached_head_still_has_something_to_show() {
        let facts = parse_status("# branch.head (detached)\n");
        assert_eq!(facts.branch.as_deref(), Some("(detached)"));
    }

    /// No upstream means no `branch.ab` header at all, which must not
    /// be read as "zero ahead, zero behind" *incorrectly* - both are
    /// zero here, but the parser must not panic on the missing line.
    #[test]
    fn a_branch_with_no_upstream_parses() {
        let facts = parse_status("# branch.oid abc\n# branch.head solo\n");
        assert_eq!(facts.branch.as_deref(), Some("solo"));
        assert_eq!((facts.ahead, facts.behind), (0, 0));
    }

    /// End to end against a real repository, because the flags this
    /// passes to git are as much a part of the behavior as the parser.
    #[test]
    fn probing_a_real_repository_sees_its_branch_and_its_dirt() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path();
        let git = |args: &[&str]| {
            let ok = Command::new("git")
                .arg("-C")
                .arg(path)
                .args(args)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
            matches!(ok, Ok(s) if s.success())
        };
        if !git(&["init", "--initial-branch=trunk"]) {
            return; // no git on this machine; the parser tests still ran
        }
        git(&["config", "user.email", "t@example.invalid"]);
        git(&["config", "user.name", "T"]);
        std::fs::write(path.join("a.txt"), "one").unwrap();
        git(&["add", "."]);
        git(&["commit", "-m", "one"]);

        let clean = probe(path).expect("a repository answers");
        assert_eq!(clean.branch.as_deref(), Some("trunk"));
        assert!(!clean.dirty, "{clean:?}");

        std::fs::write(path.join("b.txt"), "two").unwrap();
        std::fs::write(path.join("a.txt"), "changed").unwrap();
        git(&[
            "remote",
            "add",
            "origin",
            "https://github.com/someone/thing.git",
        ]);
        let dirty = probe(path).expect("still a repository");
        assert!(dirty.dirty);
        assert_eq!((dirty.modified, dirty.added), (1, 1), "{dirty:?}");
        assert_eq!(dirty.owner.as_deref(), Some("someone"));

        // And what was learned is remembered for the next visit, which
        // is what lets a revisited management tab skip the placeholders.
        let _ = start(vec![path.to_path_buf()]);
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        while cached(path).is_none() && std::time::Instant::now() < deadline {
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        assert_eq!(cached(path).map(|f| f.added), Some(1));
    }

    #[test]
    fn a_directory_that_is_not_a_repository_answers_nothing() {
        let dir = tempfile::tempdir().unwrap();
        assert!(probe(dir.path()).is_none());
    }
}
