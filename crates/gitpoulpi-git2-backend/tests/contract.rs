//! Runs the shared `gitpoulpi_testkit::contract` suite against both
//! `GitBackend` implementations so behavioral divergence between git2 and
//! system git is caught immediately, not just at the frontend layer.
//!
//! Each check gets its own `#[test]`, per backend, generated from the
//! single list in `gitpoulpi_testkit::for_each_contract_check!`, so the
//! `git`-spawning fixture builds run in parallel rather than serially
//! (slow on Windows, where spawning is expensive). Every check builds its own fixture repo in its
//! own temp dir and touches no shared cwd or env, so they are safe to run
//! concurrently -- adding a check that is *not* independent means it does
//! not belong in this suite.

macro_rules! contract_tests {
    ($($name:ident),+ $(,)?) => {
        mod git2_backend {
            $(
                #[test]
                fn $name() {
                    gitpoulpi_testkit::contract::$name(
                        &gitpoulpi_git2_backend::Git2Backend::new(),
                    );
                }
            )+
        }

        mod system_git_backend {
            $(
                #[test]
                fn $name() {
                    gitpoulpi_testkit::contract::$name(
                        &gitpoulpi_cli_backend::SystemGitBackend::new(),
                    );
                }
            )+
        }
    };
}

gitpoulpi_testkit::for_each_contract_check!(contract_tests);

// The LFS checks, in their own modules so test names do not clash.
#[cfg(feature = "lfs")]
mod lfs_checks {
    gitpoulpi_testkit::for_each_lfs_check!(contract_tests);
}

// The history rewrite script checks, when the Lua runtime is built.
#[cfg(feature = "rewrite-scripts")]
mod rewrite_script_checks {
    gitpoulpi_testkit::for_each_rewrite_script_check!(contract_tests);
}

// The external script checks (shell, Python, external Lua; S11).
#[cfg(feature = "rewrite-scripts")]
mod rewrite_external_script_checks {
    gitpoulpi_testkit::for_each_rewrite_external_script_check!(contract_tests);
}

/// Without the `rewrite-scripts` feature a script is a typed `Unsupported`.
#[cfg(not(feature = "rewrite-scripts"))]
#[test]
fn rewrite_scripts_are_not_built() {
    use gitpoulpi_core::domain::rewrite::{ScriptLanguage, ScriptRef, ScriptSource};
    use gitpoulpi_core::{Error, GitBackend};
    let repo = gitpoulpi_testkit::FixtureRepoBuilder::new()
        .unwrap()
        .commit("c")
        .unwrap()
        .build();
    let script = ScriptRef {
        language: ScriptLanguage::Lua,
        source: ScriptSource::Inline {
            text: "function rewrite_commit() end".into(),
        },
        params: Default::default(),
    };
    let r = repo.repo_ref();
    for backend in [
        &gitpoulpi_git2_backend::Git2Backend::new() as &dyn GitBackend,
        &gitpoulpi_cli_backend::SystemGitBackend::new(),
    ] {
        assert!(matches!(
            backend.rewrite_script_check(&r, &script, &Default::default()),
            Err(Error::Unsupported { .. })
        ));
        assert!(matches!(
            backend.rewrite_script_test(&r, &script, &Default::default(), &[]),
            Err(Error::Unsupported { .. })
        ));
        let recipe = gitpoulpi_core::domain::rewrite::Recipe::new(
            "s",
            Default::default(),
            vec![gitpoulpi_core::domain::rewrite::Rule::new(
                gitpoulpi_core::domain::rewrite::RuleKind::Script(script.clone()),
            )],
        );
        assert!(matches!(
            backend.rewrite_preview(&r, &recipe, &mut |_| std::ops::ControlFlow::Continue(())),
            Err(Error::Unsupported { .. })
        ));
    }
}

/// Without the `lfs` feature the LFS operations answer a typed `NotBuilt`.
#[cfg(not(feature = "lfs"))]
#[test]
fn lfs_operations_are_not_built() {
    use gitpoulpi_core::domain::LfsInstallScope;
    use gitpoulpi_core::{Error, GitBackend};
    let repo = gitpoulpi_testkit::FixtureRepoBuilder::new()
        .unwrap()
        .commit("c")
        .unwrap()
        .build();
    let backend = gitpoulpi_git2_backend::Git2Backend::new();
    let r = repo.repo_ref();
    assert!(matches!(
        backend.lfs_files(&r, None),
        Err(Error::NotBuilt { feature: "lfs" })
    ));
    assert!(matches!(
        backend.lfs_install(&r, LfsInstallScope::Repository),
        Err(Error::NotBuilt { feature: "lfs" })
    ));
}

#[cfg(feature = "worktree")]
mod worktree_checks {
    gitpoulpi_testkit::for_each_worktree_check!(contract_tests);
}

#[cfg(feature = "submodule")]
mod submodule_checks {
    gitpoulpi_testkit::for_each_submodule_check!(contract_tests);
}

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
    let backend = gitpoulpi_git2_backend::Git2Backend::new();
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
