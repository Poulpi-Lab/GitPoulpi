use gitpoulpi_core::domain::RepoRef;
use gitpoulpi_core::{Error, Result};

pub fn open(repo: &RepoRef) -> Result<git2::Repository> {
    git2::Repository::open(&repo.root).map_err(map_err)
}

/// `GitBackend::discover` over libgit2.
///
/// `Repository::discover` does the upward walk, including the cases a
/// `.join(".git").exists()` loop gets wrong - a linked worktree's `.git`
/// file, a `.git` file pointing elsewhere, a bare repository. Everything
/// after it is reading the opened handle, so no second process and no
/// second walk can disagree with the first.
pub fn discover(path: &std::path::Path) -> Result<gitpoulpi_core::domain::DiscoveredRepo> {
    use gitpoulpi_core::domain::{DiscoveredRepo, RepoRef};

    // libgit2 happily discovers upwards from a path that does not exist,
    // which would turn a deleted recent-repositories entry into a
    // successful open of whatever repository contains it. The
    // contract says that is `RepoNotFound`, so the existence check is
    // explicit rather than delegated.
    if !path.exists() {
        return Err(Error::RepoNotFound(path.to_path_buf()));
    }
    let repo =
        git2::Repository::discover(path).map_err(|_| Error::RepoNotFound(path.to_path_buf()))?;

    let is_bare = repo.is_bare();
    let is_worktree = repo.is_worktree();
    let root = if is_bare {
        repo.path().to_path_buf()
    } else {
        repo.workdir()
            .map(|w| w.to_path_buf())
            .unwrap_or_else(|| repo.path().to_path_buf())
    };
    // libgit2 hands back workdir paths with a trailing separator; the
    // system backend's `--show-toplevel` does not, and the two must
    // produce byte-identical `RepoId`s or one daemon entry becomes two.
    let root = strip_trailing_separator(&root);

    // `head()` is an error in a repository with no commits (unborn
    // branch) and points at a commit, not a branch, when detached. Both
    // are "no branch" rather than a failure to discover.
    let head_branch = repo
        .head()
        .ok()
        .filter(|h| h.is_branch())
        .and_then(|h| h.shorthand().map(str::to_string));

    Ok(DiscoveredRepo {
        repo: RepoRef::from_root(&root),
        name: DiscoveredRepo::name_for_root(&root),
        is_bare,
        is_worktree,
        head_branch,
    })
}

fn strip_trailing_separator(path: &std::path::Path) -> std::path::PathBuf {
    let raw = path.to_string_lossy();
    let trimmed = raw.trim_end_matches(['/', '\\']);
    if trimmed.is_empty() {
        path.to_path_buf()
    } else {
        std::path::PathBuf::from(trimmed)
    }
}

pub fn map_err(e: git2::Error) -> Error {
    let message = gitpoulpi_core::redact(&format!("git2: {e}"));
    let detail = gitpoulpi_core::ErrorDetail::from_git2(
        "libgit2",
        format!("{:?}", e.class()),
        format!("{:?}", e.code()),
        e.message(),
    );
    Error::git(message, detail)
}

pub fn to_git2_oid(oid: gitpoulpi_core::domain::Oid) -> Result<git2::Oid> {
    git2::Oid::from_str(&oid.to_hex()).map_err(map_err)
}

pub fn to_domain_oid(oid: git2::Oid) -> Result<gitpoulpi_core::domain::Oid> {
    gitpoulpi_core::domain::Oid::from_hex(&oid.to_string())
        .ok_or_else(|| Error::Backend("git2 returned an invalid oid".into()))
}

/// `GitBackend::init` over libgit2.
///
/// `Repository::init_opts` creates the directory chain itself, so the
/// only work here is the part the trait adds on top of git: refusing a
/// path that is *already* a repository root, and reporting the root by
/// re-`discover`ing the result rather than echoing what the caller
/// typed. The second is what keeps the `RepoRef` this hands back
/// byte-identical to the one the system backend produces for the same
/// directory - libgit2's own `workdir()` carries a trailing separator,
/// which `discover` already strips in one place.
pub fn init(
    path: &std::path::Path,
    opts: &gitpoulpi_core::domain::InitOptions,
) -> Result<gitpoulpi_core::domain::RepoRef> {
    if already_a_repository_root(path) {
        return Err(Error::Backend(format!(
            "{} is already a git repository",
            path.display()
        )));
    }

    let mut init_opts = git2::RepositoryInitOptions::new();
    init_opts.bare(opts.bare).mkpath(true);
    if let Some(branch) = &opts.initial_branch {
        init_opts.initial_head(branch);
    }
    git2::Repository::init_opts(path, &init_opts).map_err(map_err)?;

    Ok(discover(path)?.repo)
}

/// Whether `path` *is* a repository root, as opposed to being inside
/// one - the distinction that makes re-initializing an error while a
/// nested repository stays legitimate.
///
/// Four lines duplicated in `gitpoulpi-cli-backend`'s `discover.rs` on
/// purpose: the comparison is the same, but each backend has to ask its
/// *own* `discover`, and `gitpoulpi-core` must not know how either one
/// finds a repository.
fn already_a_repository_root(path: &std::path::Path) -> bool {
    match discover(path) {
        Ok(found) => found.repo.root == RepoRef::from_root(path).root,
        Err(_) => false,
    }
}
