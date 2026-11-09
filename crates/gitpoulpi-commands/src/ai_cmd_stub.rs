//! `ai_cmd` for a build without the `ai` feature (custom builds, see
//! the custom-builds design §7): the `ai` subcommand does not
//! exist, and the `--ai` flags of `commit` and `stash push` answer with the
//! typed "not built" message instead of calling a model.

use crate::client::Session;

fn not_built() -> String {
    gitpoulpi_core::Error::NotBuilt { feature: "ai" }.to_string()
}

/// Stands in for `gitpoulpi_ai::parse::CommitMessage`.
pub(crate) struct CommitMessage;

impl CommitMessage {
    pub(crate) fn full(&self) -> String {
        String::new()
    }
}

/// Stands in for `gitpoulpi_ai::Cut`.
pub(crate) struct Cut;

pub(crate) async fn generate_commit_message(
    _: &mut Session,
    _: bool,
) -> Result<(CommitMessage, Vec<Cut>), String> {
    Err(not_built())
}

pub(crate) async fn generate_stash_message(
    _: &mut Session,
    _: bool,
) -> Result<(String, Vec<Cut>), String> {
    Err(not_built())
}

pub(crate) fn report_truncated(_: &mut serde_json::Value, _: &mut String, _: &[Cut]) {}
