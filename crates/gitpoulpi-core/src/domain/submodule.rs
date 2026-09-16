use serde::{Deserialize, Serialize};

use super::Oid;

/// Where a submodule stands relative to what the superproject records
/// (`git submodule status`'s first column).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubmoduleState {
    /// Not cloned into the working tree yet (`-`): `submodule update --init`
    /// fetches it.
    Uninitialized,
    /// Checked out at the commit the superproject records.
    UpToDate,
    /// Checked out at another commit than the recorded one (`+`): the
    /// submodule moved on (or the superproject did) and the pointer was not
    /// committed, or `submodule update` has not run yet.
    Modified,
    /// The superproject has merge conflicts on the submodule (`U`).
    Conflicted,
}

/// One submodule of a repository, as `.gitmodules` and the index describe
/// it. Both backends list the same submodules in the same order (by path).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Submodule {
    /// The name in `.gitmodules` (usually the path).
    pub name: String,
    /// Repo-relative, `/`-separated.
    pub path: String,
    /// The URL `.gitmodules` records (empty when it records none).
    pub url: String,
    /// `submodule.<name>.branch`, when set.
    pub branch: Option<String>,
    pub state: SubmoduleState,
    /// The commit the superproject records (index entry), if any.
    pub expected: Option<Oid>,
    /// The commit checked out in the submodule's working tree, `None` while
    /// [`SubmoduleState::Uninitialized`].
    pub head: Option<Oid>,
}

/// What [`crate::GitBackend::submodule_update`] updates and how.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubmoduleUpdateOptions {
    /// Repo-relative paths; empty means every submodule.
    pub paths: Vec<String>,
    /// Clone and initialize submodules that are not yet (`--init`).
    pub init: bool,
    /// Also update the submodules' own submodules (`--recursive`).
    pub recursive: bool,
}
