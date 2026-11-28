//! The optional parts this build of the client was compiled with. The
//! frontend forwards its own features of the same names.

pub const LFS: bool = cfg!(feature = "lfs");
pub const WORKTREE: bool = cfg!(feature = "worktree");
pub const SUBMODULE: bool = cfg!(feature = "submodule");
