pub mod backend;
pub mod diff_text;
pub mod domain;
pub mod error;
pub mod failure;
pub mod fuzzy;
pub mod guard;
pub mod journal;
pub mod licenses;
pub mod paths;
pub mod privacy;
pub mod repo_config;
pub mod textconv;
pub mod time;
pub mod workflow;

pub use backend::GitBackend;
pub use error::{Error, NotCancellableReason, Result, TypedError, BUILD_FEATURES};
pub use failure::{redact, ErrorDetail, FailureKind, Git2Detail};
