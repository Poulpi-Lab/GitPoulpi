//! Stands in for a GitPoulpi binary as git's credential helper in the daemon's
//! tests (a test executable has no helper entry of its own): the same one
//! call every real binary makes first thing in `main`.

fn main() {
    gitpoulpi_ipc::git_credential::run_if_requested();
}
