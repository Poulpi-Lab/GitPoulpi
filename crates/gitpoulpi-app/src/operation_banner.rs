//! The wording of the strip shown above the Log while a merge, a cherry-pick or
//! a revert is stopped (a rebase has its own): what is going on, how many files
//! still have conflicts, and the way forward. A function of an
//! `OperationState` and a conflict count only; a frontend draws the strip
//! itself with these texts and buttons.

use gitpoulpi_core::domain::OperationState;
use gitpoulpi_i18n::msg::merge as tr;

/// The characters of a commit id the banner shows.
const SHORT_SHA_LEN: usize = 7;

/// The banner's buttons, in the order they are drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationBannerButton {
    /// Open the merge editor on the first conflicted file.
    Resolve,
    /// Commit the merge (or continue the cherry-pick / revert): both are
    /// the commit, which concludes them.
    Commit,
    /// Give up on a cherry-pick or revert (empty because the change is
    /// already applied, or not worth resolving).
    Skip,
    Abort,
}

/// Whether `state` has a banner of this kind (the rebase has its own).
pub fn applies(state: &OperationState) -> bool {
    // A rebase and a `git am` keep their own strips.
    !matches!(state, OperationState::Rebase(_) | OperationState::Am { .. })
}

fn short(oid: gitpoulpi_core::domain::Oid) -> String {
    oid.to_hex()[..SHORT_SHA_LEN].to_string()
}

/// The banner's text, independent of layout.
pub fn banner_text(state: &OperationState, conflicts: usize) -> String {
    let head = match state {
        OperationState::Merge {
            head_name: Some(head),
            their_label,
        } => tr::banner_merge(their_label, head),
        OperationState::Merge { their_label, .. } => tr::banner_merge_detached(their_label),
        OperationState::CherryPick { commit } => tr::banner_pick(short(*commit)),
        OperationState::Revert { commit } => tr::banner_revert(short(*commit)),
        OperationState::Rebase(_) | OperationState::Am { .. } => String::new(),
    };
    let tail = if conflicts == 0 {
        tr::BANNER_ALL_RESOLVED.get().to_string()
    } else {
        tr::banner_files(conflicts as u64)
    };
    format!("{head} \u{b7} {tail}")
}

/// The text at every length it can be drawn at, longest first: names the
/// operation and what is left, then only the operation and what is left.
pub fn banner_texts(state: &OperationState, conflicts: usize) -> Vec<String> {
    let tail = if conflicts == 0 {
        tr::BANNER_ALL_RESOLVED.get().to_string()
    } else {
        tr::banner_files(conflicts as u64)
    };
    let name = match state {
        OperationState::Merge { .. } => tr::BANNER_MERGE_SHORT.get().to_string(),
        OperationState::CherryPick { commit } => tr::banner_pick(short(*commit)),
        OperationState::Revert { commit } => tr::banner_revert(short(*commit)),
        OperationState::Rebase(_) | OperationState::Am { .. } => String::new(),
    };
    vec![
        banner_text(state, conflicts),
        format!("{name} \u{b7} {tail}"),
        name,
    ]
}

/// The buttons that make sense now: Resolve while files are conflicted,
/// the commit (or Continue) once none is, Skip for a cherry-pick or
/// revert, and always Abort.
pub fn buttons(state: &OperationState, conflicts: usize) -> Vec<(OperationBannerButton, String)> {
    let mut out = Vec::new();
    if conflicts > 0 {
        out.push((
            OperationBannerButton::Resolve,
            tr::BANNER_RESOLVE.get().to_string(),
        ));
    } else {
        out.push((
            OperationBannerButton::Commit,
            match state {
                OperationState::Merge { .. } => tr::BANNER_COMMIT_MERGE.get(),
                _ => tr::BANNER_CONTINUE.get(),
            }
            .to_string(),
        ));
    }
    if matches!(
        state,
        OperationState::CherryPick { .. } | OperationState::Revert { .. }
    ) {
        out.push((
            OperationBannerButton::Skip,
            tr::BANNER_SKIP.get().to_string(),
        ));
    }
    out.push((
        OperationBannerButton::Abort,
        match state {
            OperationState::Merge { .. } => tr::BANNER_ABORT_MERGE.get(),
            _ => tr::BANNER_ABORT.get(),
        }
        .to_string(),
    ));
    out
}
