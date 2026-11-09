use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Stable identity for a repo, derived from its canonicalized root path.
/// Used as the tab/session key so the same repo always maps to the same id.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RepoId(pub String);

impl RepoId {
    pub fn from_path(path: &Path) -> Self {
        let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        Self(canonical.to_string_lossy().to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepoRef {
    pub id: RepoId,
    pub root: PathBuf,
}

impl RepoRef {
    /// The canonical reference to the repository rooted at `root`.
    ///
    /// **Build every `RepoRef` through this.** `RepoId` canonicalises, so
    /// two spellings of one repository share an id, but a raw `root` can
    /// differ between them and the `RepoRef`s then compare unequal. macOS
    /// puts temporary directories under `/var`, a symlink to
    /// `/private/var`; Windows hands out 8.3 short names, and
    /// `git rev-parse` answers with forward slashes where `PathBuf` uses
    /// backslashes.
    ///
    /// The verbatim prefix `canonicalize` adds on Windows is stripped:
    /// `root` is spawned as a `current_dir` and printed in tab titles.
    /// `id` keeps whatever `RepoId::from_path` produces - an opaque key,
    /// not a path.
    pub fn from_root(root: &Path) -> Self {
        Self {
            id: RepoId::from_path(root),
            root: canonical_root(root),
        }
    }
}

/// `root`, canonicalised when the filesystem can answer, with Windows'
/// verbatim prefix removed (see [`crate::paths::canonicalize_display`]).
/// Falls back to the path as given for a deleted repository - a stale tab
/// still has to be nameable.
fn canonical_root(root: &Path) -> PathBuf {
    crate::paths::canonicalize_display(root)
}

/// What `GitBackend::discover` learned about the repository enclosing a
/// path: enough to label a tab, warn about a bare repo and show which
/// branch is out.
///
/// `repo` is derived from the *root*, never from the path that was typed:
/// `RepoId` is the daemon subscription key, so a TUI keyed on a
/// subdirectory and a CLI keyed on the root would be two different repos
/// to the daemon.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscoveredRepo {
    pub repo: RepoRef,
    /// The last path component of the root - what a tab is labeled
    /// with. Computed here so every frontend labels a tab identically.
    pub name: String,
    /// A repository with no working tree (`git clone --bare`). Most
    /// operations are meaningless on one, so a frontend can refuse to
    /// open it with a real reason instead of failing later and opaquely.
    pub is_bare: bool,
    /// A linked worktree rather than the main checkout.
    pub is_worktree: bool,
    /// The checked-out branch's short name, or `None` on a detached HEAD
    /// or in a repository with no commits yet.
    pub head_branch: Option<String>,
}

impl DiscoveredRepo {
    /// The tab label for a root path, in the one place that decides it.
    pub fn name_for_root(root: &Path) -> String {
        root.file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .filter(|n| !n.is_empty())
            .unwrap_or_else(|| root.to_string_lossy().into_owned())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Oid(pub [u8; 20]);

/// Serialized as its 40-character hex string, not the 20 raw bytes a
/// derived impl would produce: it crosses the IPC wire and comes out of
/// `gitpoulpi --json`, where a human reads it and an agent pastes it back.
/// Both directions live here so the wire format cannot disagree with itself.
impl serde::Serialize for Oid {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_hex())
    }
}

impl<'de> Deserialize<'de> for Oid {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let hex = String::deserialize(deserializer)?;
        Oid::from_hex(&hex).ok_or_else(|| {
            serde::de::Error::custom(format!("`{hex}` is not a 40-character hex object id"))
        })
    }
}

impl Oid {
    pub fn from_hex(hex: &str) -> Option<Self> {
        if hex.len() != 40 {
            return None;
        }
        let mut bytes = [0u8; 20];
        for (i, chunk) in hex.as_bytes().chunks(2).enumerate() {
            let byte_str = std::str::from_utf8(chunk).ok()?;
            bytes[i] = u8::from_str_radix(byte_str, 16).ok()?;
        }
        Some(Self(bytes))
    }

    pub fn to_hex(self) -> String {
        self.0.iter().map(|b| format!("{b:02x}")).collect()
    }
}

impl std::fmt::Display for Oid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitNode {
    pub id: Oid,
    pub parents: Vec<Oid>,
    /// First line only - for the table/graph row. See `message` for the
    /// full `summary\n\nbody` text (body may be empty).
    pub summary: String,
    /// Full raw commit message (git's `summary\n\nbody`), for detail
    /// panels/tooltips that need more than the one-line `summary`.
    pub message: String,
    pub author_name: String,
    pub author_email: String,
    pub timestamp: i64,
    pub refs: Vec<super::RefKind>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CommitGraph {
    pub nodes: Vec<CommitNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Remote {
    pub name: String,
    pub fetch_url: String,
    pub push_url: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorkingTree {
    pub files: Vec<super::StatusEntry>,
    /// Who the repository's remotes sign in as, when its profile is not all
    /// default. Added by the daemon from the private file (no spawn).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repo_auth: Option<super::ActiveIdentity>,
}

#[cfg(test)]
mod tests {
    use super::{canonical_root, Oid};

    /// A canonicalised Windows path must lose its verbatim prefix:
    /// `root` is spawned as a `current_dir`, printed in the title bar
    /// and shown in the repository picker, and none of those want
    /// `\\?\`. Both prefixes were spelled with one leading backslash,
    /// so neither could ever match and every root kept it.
    #[test]
    fn a_canonical_root_has_no_verbatim_prefix() {
        let dir = tempfile::tempdir().unwrap();
        let root = canonical_root(dir.path());
        let text = root.to_string_lossy();
        assert!(
            !text.starts_with(r"\\?\"),
            "the verbatim prefix survived: {text}"
        );
        // ...and it still names the same directory.
        assert_eq!(
            root.canonicalize().unwrap(),
            dir.path().canonicalize().unwrap()
        );
    }

    #[test]
    fn an_oid_is_json_as_its_hex_string() {
        let hex = "fa477cb6b0a9484811549a2b1f0bb2c4c0d1e2f3";
        let oid = Oid::from_hex(hex).unwrap();

        let json = serde_json::to_string(&oid).unwrap();
        assert_eq!(json, format!("\"{hex}\""));
        assert_eq!(serde_json::from_str::<Oid>(&json).unwrap(), oid);
    }

    #[test]
    fn a_malformed_oid_is_a_deserialization_error() {
        // Silently accepting this would hand a zeroed oid to whatever
        // operation asked for one.
        assert!(serde_json::from_str::<Oid>("\"nope\"").is_err());
    }
}

#[cfg(test)]
mod repo_ref_tests {
    use super::*;

    /// Two spellings of one directory must produce **equal** `RepoRef`s,
    /// root included (the ids alone matched on macOS and Windows while the
    /// roots differed). The `.`/`..` detour below is the portable shape.
    #[test]
    fn two_spellings_of_one_root_give_one_ref() {
        let dir = tempfile::tempdir().expect("tempdir");
        let repo = dir.path().join("repo");
        std::fs::create_dir_all(&repo).expect("create");

        let direct = RepoRef::from_root(&repo);
        let detour = RepoRef::from_root(&repo.join("..").join("repo").join("."));
        assert_eq!(direct, detour);
    }

    /// And the root it settles on is usable as a path, not a verbatim
    /// form: it is spawned as a `current_dir` and printed in tab titles.
    #[test]
    fn the_root_is_a_plain_path() {
        let dir = tempfile::tempdir().expect("tempdir");
        let repo = dir.path().join("repo");
        std::fs::create_dir_all(&repo).expect("create");

        let root = RepoRef::from_root(&repo).root;
        assert!(
            !root.to_string_lossy().starts_with(r"\\?\"),
            "{root:?} still carries the verbatim prefix"
        );
        assert!(root.is_dir(), "{root:?} should still name the directory");
    }

    /// A repository deleted from disk still has to be nameable - a stale
    /// tab is closed by the same `RepoRef` it was opened with.
    #[test]
    fn a_vanished_root_falls_back_to_the_path_as_given() {
        let dir = tempfile::tempdir().expect("tempdir");
        let gone = dir.path().join("gone");
        let reference = RepoRef::from_root(&gone);
        assert_eq!(reference.root, gone);
    }
}
