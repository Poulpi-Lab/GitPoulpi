//! `git.default_encoding` (and the repository's own `encoding` preference):
//! the fallback encoding the reads that produce text for a person -- file
//! content, blame, diffs, the merge conflict editor's document, commit and stash messages -- decode non-UTF-8 bytes with. Staging and patch
//! operations never get it, so what they write stays byte-exact.

use gitpoulpi_ipc::Request;
use gitpoulpi_session::RepoState;

/// The encoding label to scope around `request`, if it is one of the
/// text-producing reads and an encoding other than UTF-8 is configured.
/// The repository's preference wins over `git.default_encoding`.
pub(crate) fn for_request(request: &Request) -> Option<String> {
    use gitpoulpi_core::domain::OperationKind as K;
    if !matches!(
        request.command,
        K::ReadBlob
            | K::Blame
            | K::FileHistory
            | K::Diff
            | K::DiffFiles
            | K::Log
            | K::Stashes
            | K::ConflictShow
    ) {
        return None;
    }
    let settings = gitpoulpi_config::Settings::resolve_ambient().ok();
    let repo = crate::dispatch::repo_ref(request).ok()?;
    let state_dir = settings
        .as_ref()
        .map(|s| s.config().session.per_repo_state_dir.clone())
        .unwrap_or_else(|| gitpoulpi_config::DEFAULT_REPO_DATA_LOCATION.to_string());
    let from_repo = RepoState::load(&RepoState::path_for(&repo.root, &state_dir))
        .ok()
        .and_then(|s| s.preferences.encoding)
        .filter(|e| !e.trim().is_empty());
    let label = from_repo.or_else(|| settings.map(|s| s.config().git.default_encoding.clone()))?;
    let lower = label.trim().to_lowercase();
    (!matches!(lower.as_str(), "" | "utf-8" | "utf8")).then_some(label)
}
