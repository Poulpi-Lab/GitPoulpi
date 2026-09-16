mod credential_helper;
mod token_store;

pub use credential_helper::CredentialHelper;
pub use token_store::{CredentialHelperTokens, InMemoryTokens, TokenStore};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("credential helper invocation failed: {0}")]
    HelperFailed(String),
    #[error("no credentials found for {0}")]
    NotFound(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone)]
pub struct Credential {
    pub username: String,
    pub password: String,
}

/// Resolves credentials through the user's own git credential helper
/// (e.g. Git Credential Manager).
pub trait CredentialSource {
    fn resolve(&self, host: &str) -> Result<Credential>;
}
