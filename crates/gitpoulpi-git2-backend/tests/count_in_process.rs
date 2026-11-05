//! `log.commit_count = "libgit2"`: the in-process count agrees with the walk
//! the graph loads, on both backends' `log`.

#[test]
fn the_in_process_count_matches_the_git2_walk() {
    gitpoulpi_testkit::contract::counter_matches_the_walk(
        &gitpoulpi_git2_backend::Git2Backend::new(),
        &gitpoulpi_git2_backend::count_commits_in_process,
    );
}

#[test]
fn the_in_process_count_matches_the_system_git_walk() {
    gitpoulpi_testkit::contract::counter_matches_the_walk(
        &gitpoulpi_cli_backend::SystemGitBackend::new(),
        &gitpoulpi_git2_backend::count_commits_in_process,
    );
}

#[test]
fn the_in_process_count_honours_hidden_refs_on_both_backends() {
    gitpoulpi_testkit::contract::counter_honours_hidden(
        &gitpoulpi_git2_backend::Git2Backend::new(),
        &gitpoulpi_git2_backend::count_commits_in_process,
    );
    gitpoulpi_testkit::contract::counter_honours_hidden(
        &gitpoulpi_cli_backend::SystemGitBackend::new(),
        &gitpoulpi_git2_backend::count_commits_in_process,
    );
}
