//! Git LFS, as far as the domain is concerned.
//!
//! Everything here is pure: pointer parsing, `.gitattributes` scanning
//! and the value types the `lfs_*` operations exchange. The *doing*
//! (downloading, uploading, locking) is always the system `git-lfs`
//! program's job -- see `docs/architecture/decisions/0009-git-lfs-via-system-binary.md`.
//! Reading LFS *state* (is this an LFS repository, which files are
//! pointers, is the object stored locally) needs no binary at all, so a
//! frontend can say "LFS object, 12 MB, not downloaded" even when
//! `git-lfs` is not installed.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::{DiffLine, FileDiff, LineOrigin};

/// Where to send a user whose machine lacks `git-lfs`. One constant, so
/// the TUI button, the CLI message and the docs cannot drift apart.
pub const LFS_INSTALL_URL: &str = "https://git-lfs.com";

/// The oldest `git-lfs` GitPoulpi supports (major, minor).
pub const LFS_MIN_VERSION: (u32, u32) = (3, 0);

/// A pointer file is always tiny; git-lfs itself refuses anything of
/// 1024 bytes or more.
pub const LFS_POINTER_MAX_BYTES: usize = 1024;

const POINTER_VERSIONS: [&str; 2] = [
    "https://git-lfs.github.com/spec/v1",
    "http://git-media.io/v/2", // the legacy "hawser" spelling
];

/// The text stored in git in place of a large file.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LfsPointer {
    /// Lower-case hex SHA-256 of the real content (64 characters).
    pub oid: String,
    /// Byte length of the real content.
    pub size: u64,
}

impl LfsPointer {
    /// Parse a blob as an LFS pointer, or `None` if it is anything else.
    ///
    /// Strict about the shape (the `version` line first, a 64-hex `oid
    /// sha256:` line, a numeric `size` line, under 1024 bytes) so ordinary
    /// text that merely mentions LFS is never mistaken for a pointer, but
    /// tolerant of CRLF endings, a missing final newline and the
    /// `ext-N-*` extension lines.
    pub fn parse(raw: &[u8]) -> Option<Self> {
        if raw.len() >= LFS_POINTER_MAX_BYTES {
            return None;
        }
        let text = std::str::from_utf8(raw).ok()?;
        let mut lines = text.split('\n').map(|l| l.strip_suffix('\r').unwrap_or(l));
        let version = lines.next()?.strip_prefix("version ")?;
        if !POINTER_VERSIONS.contains(&version) {
            return None;
        }
        let mut oid = None;
        let mut size = None;
        for line in lines {
            if line.is_empty() {
                continue;
            }
            if let Some(rest) = line.strip_prefix("oid sha256:") {
                let ok = rest.len() == 64
                    && rest
                        .bytes()
                        .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b));
                if !ok || oid.is_some() {
                    return None;
                }
                oid = Some(rest.to_string());
            } else if let Some(rest) = line.strip_prefix("size ") {
                if size.is_some() {
                    return None;
                }
                size = Some(rest.parse::<u64>().ok()?);
            } else if !line.starts_with("ext-") {
                return None;
            }
        }
        Some(Self {
            oid: oid?,
            size: size?,
        })
    }

    /// The canonical pointer text git-lfs would write.
    pub fn to_text(&self) -> String {
        format!(
            "version https://git-lfs.github.com/spec/v1\noid sha256:{}\nsize {}\n",
            self.oid, self.size
        )
    }

    /// `ab12cd34`: enough of the oid to tell two objects apart on screen.
    pub fn short_oid(&self) -> &str {
        &self.oid[..self.oid.len().min(8)]
    }

    /// Path of this object inside an LFS storage directory:
    /// `objects/aa/bb/<oid>`.
    pub fn storage_rel_path(&self) -> PathBuf {
        let n = self.oid.len();
        let mut p = PathBuf::from("objects");
        p.push(&self.oid[..2.min(n)]);
        p.push(&self.oid[2.min(n)..4.min(n)]);
        p.push(&self.oid);
        p
    }
}

/// `1.2 MB`-style size, decimal units like git-lfs itself prints.
pub fn human_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    if bytes < 1000 {
        return format!("{bytes} B");
    }
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1000.0 && unit < UNITS.len() - 1 {
        value /= 1000.0;
        unit += 1;
    }
    format!("{value:.1} {}", UNITS[unit])
}

/// One-line description shared by every frontend that shows an LFS blob.
pub fn describe_pointer(pointer: &LfsPointer) -> String {
    format!(
        "Git LFS object, {}, sha256:{}...",
        human_size(pointer.size),
        pointer.short_oid()
    )
}

/// A diff of a file whose contents are LFS pointers: the pointer before
/// and after. Either side is `None` for an added, deleted or
/// un-tracked file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LfsChange {
    pub old: Option<LfsPointer>,
    pub new: Option<LfsPointer>,
}

/// If `diff` is a change to an LFS pointer, the pointers on each side.
///
/// A pointer is three lines, so it always sits inside one hunk; the old
/// side is the hunk's context plus deleted lines, the new side its
/// context plus added lines. Returns `None` for any file where neither
/// side parses as a pointer.
pub fn lfs_change(diff: &FileDiff) -> Option<LfsChange> {
    let mut old = String::new();
    let mut new = String::new();
    for hunk in &diff.hunks {
        for DiffLine {
            origin, content, ..
        } in &hunk.lines
        {
            let line = content.strip_suffix('\n').unwrap_or(content);
            let line = line.strip_suffix('\r').unwrap_or(line);
            if matches!(origin, LineOrigin::Context | LineOrigin::Delete) {
                old.push_str(line);
                old.push('\n');
            }
            if matches!(origin, LineOrigin::Context | LineOrigin::Add) {
                new.push_str(line);
                new.push('\n');
            }
        }
    }
    let old = LfsPointer::parse(old.as_bytes());
    let new = LfsPointer::parse(new.as_bytes());
    if old.is_none() && new.is_none() {
        None
    } else {
        Some(LfsChange { old, new })
    }
}

/// One `filter=lfs` line of a `.gitattributes` file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LfsPattern {
    /// The pattern exactly as written (`*.psd`, `assets/big[[:space:]]file.bin`).
    pub pattern: String,
    /// The line also carries the `lockable` attribute.
    pub lockable: bool,
    /// The attributes file it came from, relative to the repository root
    /// (or `info/attributes`). Empty when parsed from a bare string.
    #[serde(default)]
    pub source: PathBuf,
}

fn attribute_lines(text: &str) -> impl Iterator<Item = (&str, Vec<&str>)> {
    text.lines().filter_map(|line| {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            return None;
        }
        let mut parts = line.split_whitespace();
        let pattern = parts.next()?;
        Some((pattern, parts.collect()))
    })
}

/// Does this attributes text route anything through the LFS filter?
pub fn attributes_mention_lfs(text: &str) -> bool {
    attribute_lines(text).any(|(_, attrs)| attrs.contains(&"filter=lfs"))
}

/// The `filter=lfs` patterns in an attributes text, in file order.
pub fn parse_lfs_patterns(text: &str) -> Vec<LfsPattern> {
    attribute_lines(text)
        .filter(|(_, attrs)| attrs.contains(&"filter=lfs"))
        .map(|(pattern, attrs)| LfsPattern {
            pattern: pattern.to_string(),
            lockable: attrs.contains(&"lockable"),
            source: PathBuf::new(),
        })
        .collect()
}

/// Is the `git-lfs` program usable?
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LfsBinary {
    Installed {
        version: String,
    },
    /// Installed, but older than [`LFS_MIN_VERSION`].
    TooOld {
        version: String,
    },
    Missing,
}

impl LfsBinary {
    pub fn is_usable(&self) -> bool {
        matches!(self, LfsBinary::Installed { .. })
    }

    /// Classify the first line of `git lfs version`
    /// (`git-lfs/3.5.1 (GitHub; windows amd64; go 1.21.8)`).
    pub fn from_version_output(output: &str) -> Self {
        let Some(rest) = output.trim().strip_prefix("git-lfs/") else {
            return LfsBinary::Missing;
        };
        let version: String = rest
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == '.')
            .collect();
        let mut nums = version.split('.').map(|n| n.parse::<u32>().unwrap_or(0));
        let (major, minor) = (nums.next().unwrap_or(0), nums.next().unwrap_or(0));
        if (major, minor) >= LFS_MIN_VERSION {
            LfsBinary::Installed { version }
        } else {
            LfsBinary::TooOld { version }
        }
    }

    /// The reason every LFS operation raises when the binary is not
    /// usable, or `None` when it is.
    pub fn unavailable(&self) -> Option<LfsUnavailableReason> {
        match self {
            LfsBinary::Installed { .. } => None,
            LfsBinary::TooOld { version } => Some(LfsUnavailableReason::TooOld(version.clone())),
            LfsBinary::Missing => Some(LfsUnavailableReason::Missing),
        }
    }
}

/// Why an LFS operation cannot run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LfsUnavailableReason {
    Missing,
    /// The installed version, older than [`LFS_MIN_VERSION`].
    TooOld(String),
}

impl LfsUnavailableReason {
    /// The message every frontend shows. One text, with the install link.
    pub fn message(&self) -> String {
        let lead = match self {
            LfsUnavailableReason::Missing => "Git LFS is not installed.".to_string(),
            LfsUnavailableReason::TooOld(v) => format!(
                "The installed Git LFS ({v}) is too old; GitPoulpi needs {}.{} or newer.",
                LFS_MIN_VERSION.0, LFS_MIN_VERSION.1
            ),
        };
        format!(
            "{lead} This repository stores large files with Git LFS; GitPoulpi needs the \
             `git-lfs` program for this. Install it from {LFS_INSTALL_URL} (Windows: bundled \
             with Git for Windows, or `winget install GitHub.GitLFS`; Debian/Ubuntu: `apt \
             install git-lfs`; macOS: `brew install git-lfs`), then retry. See docs/user/lfs.md."
        )
    }
}

/// What GitPoulpi knows about LFS in one repository. Never an error for a
/// missing binary: that is the `binary` field's job.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LfsRepoStatus {
    pub binary: LfsBinary,
    /// A `.gitattributes` (root or `info/attributes`) routes files through
    /// the LFS filter, or the repository already has an LFS object store.
    pub uses_lfs: bool,
    /// `filter.lfs.*` is configured (`git lfs install` has run for this
    /// user or repository).
    pub filters_configured: bool,
    /// The LFS `pre-push` hook is installed.
    pub hooks_installed: bool,
    pub patterns: Vec<LfsPattern>,
}

/// A tracked LFS file: a path whose committed content is a pointer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LfsFile {
    pub path: PathBuf,
    pub pointer: LfsPointer,
    /// The real object is in the local LFS store.
    pub local: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LfsDirection {
    Download,
    Upload,
    Checkout,
}

/// One progress report from `git lfs fetch|pull|push`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LfsProgress {
    pub direction: LfsDirection,
    pub files_done: u64,
    pub files_total: u64,
    pub bytes_done: u64,
    pub bytes_total: u64,
    /// The final report of the operation.
    pub done: bool,
}

impl LfsProgress {
    /// 0..=100, for a progress bar. A total of zero reads as 100 only
    /// once `done`.
    pub fn percent(&self) -> u8 {
        if self.bytes_total == 0 {
            return if self.done { 100 } else { 0 };
        }
        ((self.bytes_done.min(self.bytes_total) * 100) / self.bytes_total) as u8
    }
}

/// Same callback shape as `CloneProgressSink`.
pub type LfsProgressSink<'a> = &'a (dyn Fn(LfsProgress) + Send + Sync);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LfsFetchOptions {
    /// `--all`: every ref, not only the current one.
    #[serde(default)]
    pub all_refs: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LfsPushOptions {
    /// `--all`: every ref, not only the current one.
    #[serde(default)]
    pub all_refs: bool,
}

/// Where `git lfs install` writes its filters and hooks.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum LfsInstallScope {
    /// `--local`: this repository's `.git/config` and hooks. The default,
    /// so a user's global config is never touched silently.
    #[default]
    Repository,
    /// The user's global git config.
    User,
}

/// How `lfs_track` behaves beyond writing `.gitattributes`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LfsTrackOptions {
    /// Add the `lockable` attribute, so the files are read-only until
    /// locked.
    #[serde(default)]
    pub lockable: bool,
    /// Also re-stage the already-committed files matching the patterns
    /// (`git add --renormalize`), so their next commit stores pointers.
    /// History is never rewritten (`git lfs migrate` is a non-goal).
    #[serde(default)]
    pub renormalize: bool,
}

/// Whether a clone downloads LFS objects.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum LfsCloneMode {
    /// Download them after the clone, when `git-lfs` is available.
    #[default]
    Auto,
    /// Leave pointers in the working tree.
    Skip,
}

/// A file lock held on an LFS remote.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LfsLock {
    pub id: String,
    pub path: PathBuf,
    pub owner: Option<String>,
    pub locked_at: String,
    /// The lock belongs to the current user.
    pub ours: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{FileStatus, Hunk, HunkId};

    const OID: &str = "4d7a214614ab2935c943f9e0ff69d22eadbb8f32b1258daaa5e2ca24d17e2393";

    fn canonical() -> String {
        format!("version https://git-lfs.github.com/spec/v1\noid sha256:{OID}\nsize 12345\n")
    }

    #[test]
    fn parses_the_canonical_pointer() {
        let p = LfsPointer::parse(canonical().as_bytes()).unwrap();
        assert_eq!(p.oid, OID);
        assert_eq!(p.size, 12345);
        assert_eq!(p.to_text(), canonical());
    }

    #[test]
    fn tolerates_crlf_no_final_newline_and_extension_lines() {
        let crlf = canonical().replace('\n', "\r\n");
        assert!(LfsPointer::parse(crlf.as_bytes()).is_some());
        assert!(LfsPointer::parse(canonical().trim_end().as_bytes()).is_some());
        let ext = format!(
            "version https://git-lfs.github.com/spec/v1\next-0-foo sha256:{OID}\noid sha256:{OID}\nsize 1\n"
        );
        assert!(LfsPointer::parse(ext.as_bytes()).is_some());
    }

    #[test]
    fn accepts_the_legacy_version_url() {
        let legacy = format!("version http://git-media.io/v/2\noid sha256:{OID}\nsize 7\n");
        assert_eq!(LfsPointer::parse(legacy.as_bytes()).unwrap().size, 7);
    }

    #[test]
    fn rejects_lookalikes_and_big_blobs() {
        assert!(LfsPointer::parse(b"").is_none());
        assert!(LfsPointer::parse(b"version 1\n").is_none());
        let no_size = format!("version https://git-lfs.github.com/spec/v1\noid sha256:{OID}\n");
        assert!(LfsPointer::parse(no_size.as_bytes()).is_none());
        assert!(LfsPointer::parse(
            b"version https://git-lfs.github.com/spec/v1\noid sha256:abc\nsize 1\n"
        )
        .is_none());
        let prose = format!("{}hello\n", canonical());
        assert!(LfsPointer::parse(prose.as_bytes()).is_none());
        let big = format!("{}ext-0-x {}\n", canonical(), "a".repeat(1100));
        assert!(LfsPointer::parse(big.as_bytes()).is_none());
        assert!(LfsPointer::parse(&[0xff, 0xfe]).is_none());
    }

    fn diff(lines: &[(LineOrigin, &str)]) -> FileDiff {
        FileDiff {
            path: "a.bin".into(),
            status: FileStatus::Modified,
            hunks: vec![Hunk {
                id: HunkId(0),
                old_start: 1,
                old_lines: 3,
                new_start: 1,
                new_lines: 3,
                lines: lines
                    .iter()
                    .map(|(origin, c)| DiffLine {
                        origin: origin.clone(),
                        content: (*c).to_string(),
                        staged: false,
                    })
                    .collect(),
            }],
        }
    }

    fn pointer_lines(oid_char: char, size: u32) -> Vec<String> {
        vec![
            "version https://git-lfs.github.com/spec/v1".into(),
            format!("oid sha256:{}", oid_char.to_string().repeat(64)),
            format!("size {size}"),
        ]
    }

    #[test]
    fn lfs_change_reads_modify_add_and_delete() {
        use LineOrigin::*;
        let old = pointer_lines('a', 10);
        let new = pointer_lines('b', 20);
        let modify = diff(&[
            (Context, &old[0]),
            (Delete, &old[1]),
            (Delete, &old[2]),
            (Add, &new[1]),
            (Add, &new[2]),
        ]);
        let change = lfs_change(&modify).unwrap();
        assert_eq!(change.old.unwrap().size, 10);
        assert_eq!(change.new.unwrap().size, 20);

        let add = diff(&[(Add, &new[0]), (Add, &new[1]), (Add, &new[2])]);
        let change = lfs_change(&add).unwrap();
        assert!(change.old.is_none());
        assert_eq!(change.new.unwrap().size, 20);

        let delete = diff(&[(Delete, &old[0]), (Delete, &old[1]), (Delete, &old[2])]);
        let change = lfs_change(&delete).unwrap();
        assert_eq!(change.old.unwrap().size, 10);
        assert!(change.new.is_none());
    }

    #[test]
    fn a_plain_text_diff_is_not_an_lfs_change() {
        use LineOrigin::*;
        assert!(lfs_change(&diff(&[(Delete, "one"), (Add, "two")])).is_none());
        let empty = FileDiff {
            path: "x".into(),
            status: FileStatus::Modified,
            hunks: vec![],
        };
        assert!(lfs_change(&empty).is_none());
    }

    #[test]
    fn attributes_scan_finds_lfs_lines_only() {
        let text = "# comment\n*.psd filter=lfs diff=lfs merge=lfs -text\n\
                    assets/big[[:space:]]file.bin filter=lfs diff=lfs merge=lfs -text lockable\n\
                    *.txt text eol=lf\n";
        assert!(attributes_mention_lfs(text));
        assert!(!attributes_mention_lfs("*.txt text\n# filter=lfs\n"));
        let patterns = parse_lfs_patterns(text);
        assert_eq!(patterns.len(), 2);
        assert_eq!(patterns[0].pattern, "*.psd");
        assert!(!patterns[0].lockable);
        assert_eq!(patterns[1].pattern, "assets/big[[:space:]]file.bin");
        assert!(patterns[1].lockable);
    }

    #[test]
    fn version_output_is_classified() {
        let ok =
            LfsBinary::from_version_output("git-lfs/3.5.1 (GitHub; windows amd64; go 1.21.8)\n");
        assert_eq!(
            ok,
            LfsBinary::Installed {
                version: "3.5.1".into()
            }
        );
        assert!(matches!(
            LfsBinary::from_version_output("git-lfs/2.13.3 (GitHub; linux amd64)"),
            LfsBinary::TooOld { .. }
        ));
        assert_eq!(
            LfsBinary::from_version_output("git: 'lfs' is not a git command"),
            LfsBinary::Missing
        );
    }

    #[test]
    fn unavailable_message_carries_the_install_link() {
        assert!(LfsUnavailableReason::Missing
            .message()
            .contains(LFS_INSTALL_URL));
        assert!(LfsUnavailableReason::TooOld("2.1.0".into())
            .message()
            .contains("2.1.0"));
    }

    #[test]
    fn sizes_are_human_readable() {
        assert_eq!(human_size(12), "12 B");
        assert_eq!(human_size(12_345_678), "12.3 MB");
    }

    #[test]
    fn storage_path_is_sharded_by_oid() {
        let p = LfsPointer::parse(canonical().as_bytes()).unwrap();
        let rel = p.storage_rel_path();
        let parts: Vec<_> = rel
            .iter()
            .map(|s| s.to_string_lossy().to_string())
            .collect();
        assert_eq!(parts, ["objects", "4d", "7a", OID]);
    }
}
