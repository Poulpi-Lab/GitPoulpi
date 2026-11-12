pub mod auth;
mod client;
pub mod discovery;
pub mod etag;
mod events;
pub mod git_credential;
pub mod jobs;
pub mod mailmap;
pub mod op_context;
mod protocol;
pub mod rewrite;
mod scope;
pub mod timing;
pub mod transport;
pub mod ui;

pub use auth::{Endpoint, Token};
pub use client::{Client, CALL_TIMEOUT};
pub use events::Event;
pub use jobs::{JobInfo, JobState, CANCEL_ON_DISCONNECT};
pub use protocol::{
    DaemonInfo, DaemonKind, Request, Response, ResponseError, ServerMessage, ShutdownArgs,
};
pub use scope::ChangeScope;
pub use transport::TransportError;

/// The short commit sha this build was made from, baked in by `build.rs`;
/// `None` when the build was not made from a git checkout. Shared by the
/// daemon and the TUI so the mismatch check compares like with like.
pub fn build_git_sha() -> Option<&'static str> {
    match option_env!("GITPOULPI_GIT_SHA") {
        None | Some("") | Some("unknown") => None,
        Some(s) => Some(s),
    }
}

/// `<version> (<sha>)`, the string every binary prints for `--version`, so
/// `gitpoulpi`, `gitpoulpi-tui` and `gitpoulpi-mcp` always agree. The sha is
/// `unknown` when the build was not made from a git checkout.
pub const VERSION_STRING: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    " (",
    env!("GITPOULPI_GIT_SHA"),
    ")"
);

/// What `--version` and `daemon status` say about how a binary was built.
/// Formatting only: the IPC crate has no features, so each binary passes
/// its own `cfg!` table (see `custom-builds.md`).
pub mod build_info {
    /// The names of the features that are on, in table order.
    pub fn enabled(features: &[(&str, bool)]) -> Vec<String> {
        features
            .iter()
            .filter(|(_, on)| *on)
            .map(|(name, _)| (*name).to_string())
            .collect()
    }

    /// The `--version` long form:
    ///
    /// ```text
    /// 0.1.0 (sha)
    /// features: ai github mcp
    /// languages: en fr
    /// custom build: -gitlab -lfs
    /// ```
    ///
    /// The last line only appears when a feature is off.
    pub fn describe(features: &[(&str, bool)], langs: &[&str]) -> String {
        describe_version(crate::VERSION_STRING, features, langs)
    }

    /// [`describe`] with the version line given (for tests).
    pub fn describe_version(version: &str, features: &[(&str, bool)], langs: &[&str]) -> String {
        let on = enabled(features);
        let mut out = format!(
            "{version}\nfeatures: {}\nlanguages: {}",
            on.join(" "),
            langs.join(" ")
        );
        let off: Vec<String> = features
            .iter()
            .filter(|(_, on)| !*on)
            .map(|(name, _)| format!("-{name}"))
            .collect();
        if !off.is_empty() {
            out.push_str("\ncustom build: ");
            out.push_str(&off.join(" "));
        }
        out
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn a_full_build_has_no_custom_line() {
            let text =
                describe_version("0.1.0 (abc)", &[("ai", true), ("lfs", true)], &["en", "fr"]);
            assert_eq!(text, "0.1.0 (abc)\nfeatures: ai lfs\nlanguages: en fr");
        }

        #[test]
        fn a_custom_build_lists_what_is_off() {
            let text = describe_version(
                "0.1.0 (abc)",
                &[("ai", true), ("mcp", false), ("lfs", false)],
                &["en"],
            );
            assert_eq!(
                text,
                "0.1.0 (abc)\nfeatures: ai\nlanguages: en\ncustom build: -mcp -lfs"
            );
            assert_eq!(enabled(&[("a", true), ("b", false)]), vec!["a"]);
        }

        #[test]
        fn describe_starts_with_the_shared_version_string() {
            assert!(describe(&[], &["en"]).starts_with(crate::VERSION_STRING));
        }
    }
}
