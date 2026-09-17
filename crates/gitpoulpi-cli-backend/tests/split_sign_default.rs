//! The split editor's "Sign commits" default is the *effective*
//! `commit.gpgSign`: a global setting counts, not only the repository's own.
//! Its own test binary because it points `GIT_CONFIG_GLOBAL` at a temp file
//! for the whole process.

use gitpoulpi_cli_backend::SystemGitBackend;
use gitpoulpi_core::domain::LogOptions;
use gitpoulpi_core::GitBackend;
use gitpoulpi_testkit::FixtureRepoBuilder;

#[test]
fn the_sign_default_follows_the_global_config_and_a_local_override() {
    let global = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(global.path(), "[commit]\n\tgpgsign = true\n").unwrap();
    std::env::set_var("GIT_CONFIG_GLOBAL", global.path());

    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .commit("second")
        .unwrap()
        .build();
    let backend = SystemGitBackend::new();
    let repo_ref = repo.repo_ref();
    let head = backend
        .log(&repo_ref, &LogOptions::default())
        .unwrap()
        .nodes[0]
        .id;

    let source = backend.split_source(&repo_ref, head).unwrap();
    assert!(source.sign_default, "global commit.gpgSign=true is ignored");

    // A repository-local `false` wins over the global `true`.
    repo.git(&["config", "--local", "commit.gpgSign", "false"]);
    let source = backend.split_source(&repo_ref, head).unwrap();
    assert!(!source.sign_default, "the local override is ignored");

    // Unset everywhere: off.
    repo.git(&["config", "--local", "--unset", "commit.gpgSign"]);
    std::fs::write(global.path(), "").unwrap();
    let source = backend.split_source(&repo_ref, head).unwrap();
    assert!(!source.sign_default);
}
