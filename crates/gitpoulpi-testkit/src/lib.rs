pub mod conflict_fixture;
pub mod contract;
pub mod fake_agent;
mod fixture;
mod isolate;
pub mod lfs;
pub mod live;
pub mod prediction_fixture;
pub mod script_fixture;
mod slow_remote;
pub mod spawn_budget;
pub mod terminal_agent;
mod test_builder;

pub use conflict_fixture::{ConflictFixture, FixtureOperation};
pub use fixture::{allow_file_submodules, png_bytes, FixtureRepo, FixtureRepoBuilder};
pub use isolate::isolate_process_env;
pub use slow_remote::SlowRemote;
pub use spawn_budget::SpawnBudget;
pub use test_builder::TestRepoBuilder;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("git command failed: {0}")]
    GitFailed(String),
}

/// Whether the `git` on `PATH` is at least `major.minor`. For checks whose
/// oracle (not the product) needs a newer git, or that a run against an
/// older git (`docs/developer/testing.md`, "Older git") must adapt.
pub fn system_git_at_least(major: u32, minor: u32) -> bool {
    let Ok(out) = std::process::Command::new("git").arg("--version").output() else {
        return false;
    };
    let text = String::from_utf8_lossy(&out.stdout);
    let mut nums = text
        .split_whitespace()
        .find(|w| w.starts_with(|c: char| c.is_ascii_digit()))
        .unwrap_or("")
        .split('.')
        .map(|n| n.parse::<u32>().unwrap_or(0));
    let (a, b) = (nums.next().unwrap_or(0), nums.next().unwrap_or(0));
    (a, b) >= (major, minor)
}
