//! Without the `worktree` / `submodule` features the operations answer a
//! typed `NotBuilt`.

/// Without the `worktree` / `submodule` features the reads and mutations
/// answer a typed `NotBuilt`.
#[cfg(any(not(feature = "worktree"), not(feature = "submodule")))]
#[test]
fn worktree_and_submodule_operations_are_not_built() {
    use gitpoulpi_core::{Error, GitBackend};
    let repo = gitpoulpi_testkit::FixtureRepoBuilder::new()
        .unwrap()
        .commit("c")
        .unwrap()
        .build();
    let backend = gitpoulpi_cli_backend::SystemGitBackend::new();
    let r = repo.repo_ref();
    #[cfg(not(feature = "worktree"))]
    {
        use gitpoulpi_core::domain::{WorktreeAddOptions, WorktreeStart};
        let opts = WorktreeAddOptions {
            start: WorktreeStart::Branch("main".into()),
            new_branch: None,
        };
        assert!(matches!(
            backend.worktrees(&r),
            Err(Error::NotBuilt {
                feature: "worktree"
            })
        ));
        assert!(matches!(
            backend.worktree_add(&r, std::path::Path::new("x"), &opts),
            Err(Error::NotBuilt {
                feature: "worktree"
            })
        ));
    }
    #[cfg(not(feature = "submodule"))]
    {
        assert!(matches!(
            backend.submodules(&r),
            Err(Error::NotBuilt {
                feature: "submodule"
            })
        ));
        assert!(matches!(
            backend.submodule_init(&r, &[]),
            Err(Error::NotBuilt {
                feature: "submodule"
            })
        ));
    }
}
