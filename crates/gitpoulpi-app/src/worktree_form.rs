//! The Create worktree form's rules: the default folder, the seeds the menus
//! hand the form, and what Create may submit.
//!
//! A frontend keeps the typed fields itself (a terminal text field, a desktop
//! input) and asks this module the questions that are not about drawing: the
//! folder a name implies ([`default_worktree_path`]), where the form starts
//! ([`seed_from_head`]) and whether the fields as typed make a valid worktree
//! ([`WorktreeFields::plan`]). The generated names themselves come from
//! `gitpoulpi_core::domain` (`worktree_generated_name`,
//! `worktree_name_after_branch`).

use std::path::{Path, PathBuf};

use gitpoulpi_core::domain::{short, Ref, RefKind, Worktree, WorktreeAddOptions, WorktreeStart};
use gitpoulpi_i18n::msg;

use crate::ref_rules::branch_name_error;

/// What a context menu or a palette command hands the form: where the
/// worktree starts and how to word it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorktreeSeed {
    pub start: WorktreeStart,
    /// What the form calls the start: `feature`, `origin/feature`,
    /// `v1.0`, `a1b2c3d`.
    pub label: String,
    /// A branch name the caller wants for the worktree (a remote branch's
    /// local name); empty = the form proposes `worktree-<id>`.
    pub new_branch: String,
    /// The start branch is already checked out in a worktree, so git
    /// would refuse to check it out again: a new branch is required.
    pub branch_busy: bool,
}

/// A local branch the Ref choice offers, and whether it is checked out in
/// a worktree already.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BranchChoice {
    pub name: String,
    pub busy: bool,
}

/// `name` made folder-safe: letters, digits, `-`, `_` and `.` stay, the rest
/// becomes `-`.
pub fn folder_safe(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || "-_.".contains(c) {
                c
            } else {
                '-'
            }
        })
        .collect()
}

/// `<repo parent>/<repo name>-<label>`, the label made folder-safe: a
/// sibling of the repository, named after what it holds. The form's Folder
/// follows the Name through this until the Folder is edited by hand.
pub fn default_worktree_path(repo_root: &Path, label: &str) -> PathBuf {
    let safe = folder_safe(label);
    let name = repo_root
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "worktree".into());
    let parent = repo_root.parent().unwrap_or(repo_root);
    parent.join(format!("{name}-{safe}"))
}

/// The seed of "From the current branch": the checked-out branch cannot be
/// checked out twice, so the form proposes a new branch on it (named
/// `worktree-<id>`); a detached HEAD starts a detached worktree there.
pub fn seed_from_head(refs: &[Ref], worktrees: &[Worktree]) -> Option<WorktreeSeed> {
    if let Some(head) = refs.iter().find(|r| r.is_head) {
        if let RefKind::LocalBranch { name } = &head.kind {
            return Some(WorktreeSeed {
                start: WorktreeStart::Branch(name.clone()),
                label: name.clone(),
                new_branch: String::new(),
                branch_busy: true,
            });
        }
    }
    // Detached: the main worktree's HEAD.
    let _ = worktrees;
    let oid = crate::ref_rules::head_oid(refs)?;
    Some(WorktreeSeed {
        start: WorktreeStart::Commit(oid),
        label: short(oid),
        new_branch: String::new(),
        branch_busy: false,
    })
}

/// The seed of "Choose a branch...": the form opens on its branch list.
pub fn seed_to_choose(refs: &[Ref], worktrees: &[Worktree]) -> Option<WorktreeSeed> {
    seed_from_head(refs, worktrees)
}

/// The fields of the form as typed, to validate.
#[derive(Debug, Clone, Copy)]
pub struct WorktreeFields<'a> {
    /// The Name field.
    pub name: &'a str,
    /// The Folder field.
    pub path: &'a str,
    /// The Worktree branch field: a new branch's name, an existing local
    /// branch to check out as it is, or empty.
    pub branch: &'a str,
    /// The Ref the new branch starts from.
    pub start: &'a WorktreeStart,
    pub seed: &'a WorktreeSeed,
    /// The Ref's branch is checked out in a worktree already.
    pub start_busy: bool,
    /// The local branches the form offered.
    pub branches: &'a [BranchChoice],
}

impl WorktreeFields<'_> {
    /// Whether `name` is an existing local branch, and then whether another
    /// worktree has it (`Some(true)`); `None` for a name that is not a branch.
    pub fn branch_status(&self, name: &str) -> Option<bool> {
        if let Some(b) = self.branches.iter().find(|b| b.name == name) {
            return Some(b.busy);
        }
        match &self.seed.start {
            WorktreeStart::Branch(b) if b == name => Some(self.seed.branch_busy),
            _ => None,
        }
    }

    /// The worktree Create would add (its absolute folder and options), or
    /// the message saying why it cannot. Checks the name, that the folder is
    /// absolute and neither a file nor a non-empty directory, and the branch:
    /// an existing one is checked out as it is, a busy one is refused, a new
    /// one must be a valid branch name.
    pub fn plan(&self) -> Result<(PathBuf, WorktreeAddOptions), String> {
        let name = self.name.trim();
        if name.is_empty() {
            return Err(msg::worktree::NEED_NAME.get().into());
        }
        if name.contains(['/', '\\']) || name == "." || name == ".." {
            return Err(msg::worktree::BAD_NAME.get().into());
        }
        let path = self.path.trim();
        if path.is_empty() {
            return Err(msg::worktree::CHOOSE_FOLDER.get().into());
        }
        let path = PathBuf::from(path);
        if !path.is_absolute() {
            return Err(msg::worktree::NOT_ABSOLUTE.get().into());
        }
        if path.is_file() {
            return Err(msg::worktree::is_a_file(path.display()));
        }
        if std::fs::read_dir(&path).is_ok_and(|mut d| d.next().is_some()) {
            return Err(msg::worktree::not_empty(path.display()));
        }
        let branch = self.branch.trim();
        if branch.is_empty() {
            if self.start_busy {
                let label = match self.start {
                    WorktreeStart::Branch(b) => b.clone(),
                    WorktreeStart::Commit(_) => self.seed.label.clone(),
                };
                return Err(msg::worktree::already_checked_out(label));
            }
            return Ok((
                path,
                WorktreeAddOptions {
                    start: self.start.clone(),
                    new_branch: None,
                },
            ));
        }
        match self.branch_status(branch) {
            // An existing branch another worktree has: git would refuse.
            Some(true) => Err(msg::worktree::already_checked_out(branch)),
            // An existing branch: checked out as it is, whatever the Ref.
            Some(false) => Ok((
                path,
                WorktreeAddOptions {
                    start: WorktreeStart::Branch(branch.to_string()),
                    new_branch: None,
                },
            )),
            None => {
                if let Some(reason) = branch_name_error(branch) {
                    return Err(reason);
                }
                Ok((
                    path,
                    WorktreeAddOptions {
                        start: self.start.clone(),
                        new_branch: Some(branch.to_string()),
                    },
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn abs(p: &str) -> PathBuf {
        if cfg!(windows) {
            PathBuf::from(format!("C:\\{p}"))
        } else {
            PathBuf::from(format!("/{p}"))
        }
    }

    #[test]
    fn the_folder_is_a_safe_sibling_of_the_repository() {
        assert_eq!(folder_safe("feat/x y"), "feat-x-y");
        assert_eq!(
            default_worktree_path(&abs("src/repo"), "feat/x"),
            abs("src/repo-feat-x")
        );
    }

    #[test]
    fn a_relative_folder_is_refused() {
        let seed = WorktreeSeed {
            start: WorktreeStart::Branch("main".into()),
            label: "main".into(),
            new_branch: String::new(),
            branch_busy: true,
        };
        let start = seed.start.clone();
        let f = WorktreeFields {
            name: "wt",
            path: "relative",
            branch: "",
            start: &start,
            seed: &seed,
            start_busy: true,
            branches: &[],
        };
        assert!(f.plan().is_err(), "a relative folder is refused");
    }

    #[test]
    fn plan_checks_name_branch_and_busy_start() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("wt");
        let path = path.to_str().unwrap().to_string();
        let seed = WorktreeSeed {
            start: WorktreeStart::Branch("main".into()),
            label: "main".into(),
            new_branch: String::new(),
            branch_busy: true,
        };
        let branches = vec![
            BranchChoice {
                name: "main".into(),
                busy: true,
            },
            BranchChoice {
                name: "free".into(),
                busy: false,
            },
        ];
        let start = seed.start.clone();
        let base = WorktreeFields {
            name: "wt",
            path: &path,
            branch: "",
            start: &start,
            seed: &seed,
            start_busy: true,
            branches: &branches,
        };
        // An empty branch on a busy start: a branch is required.
        assert!(base.plan().is_err());
        // An existing free branch is checked out as it is.
        let (_, opts) = WorktreeFields {
            branch: "free",
            ..base
        }
        .plan()
        .unwrap();
        assert_eq!(opts.start, WorktreeStart::Branch("free".into()));
        assert_eq!(opts.new_branch, None);
        // A busy one is refused, a new one is created from the Ref.
        assert!(WorktreeFields {
            branch: "main",
            ..base
        }
        .plan()
        .is_err());
        let (_, opts) = WorktreeFields {
            branch: "new-one",
            ..base
        }
        .plan()
        .unwrap();
        assert_eq!(opts.new_branch.as_deref(), Some("new-one"));
        assert!(WorktreeFields {
            branch: "bad name",
            ..base
        }
        .plan()
        .is_err());
        assert!(WorktreeFields {
            name: "a/b",
            ..base
        }
        .plan()
        .is_err());
    }
}
