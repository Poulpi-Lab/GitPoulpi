//! Who a post is made as: always known, always shown.

use serde::{Deserialize, Serialize};

use super::model::{Actor, ActorKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IdentityKind {
    PersonalToken,
    /// A GitLab project or group access token (a bot user).
    ProjectOrGroupBot,
    /// A GitHub App installation token (`name[bot]`).
    AppBot,
    LocalAuthor,
    Unknown,
}

/// Where the token came from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "source", rename_all = "snake_case")]
pub enum TokenSource {
    /// The integration profile's own token.
    Profile { id: String },
    /// The repository's token command ("Use for the hosting API").
    RepoTokenCommand,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PostingIdentity {
    /// `None` while an App bot's name is not known.
    pub login: Option<String>,
    /// What to show: the login, or a description when it is unknown.
    pub display: String,
    pub kind: IdentityKind,
    pub source: TokenSource,
}

/// What an App bot is called while its login is not known.
pub const UNKNOWN_APP_BOT: &str = "the GitHub App bot";

impl PostingIdentity {
    pub fn new(login: Option<String>, kind: IdentityKind, source: TokenSource) -> Self {
        let login = login
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty());
        let display = match (&login, kind) {
            (Some(l), _) => l.clone(),
            (None, IdentityKind::AppBot) => UNKNOWN_APP_BOT.to_string(),
            (None, _) => String::new(),
        };
        Self {
            login,
            display,
            kind,
            source,
        }
    }

    /// Whether `actor` is this identity: `Some(true/false)`, or `None` when
    /// the login is not known (the caller then falls back to what it
    /// remembers having posted). Logins compare case-insensitively; an App
    /// bot's comments must also come from a bot, since a person may hold
    /// the same name.
    pub fn authored(&self, actor: &Actor) -> Option<bool> {
        let login = self.login.as_deref()?;
        let same = login.eq_ignore_ascii_case(&actor.login);
        Some(match self.kind {
            IdentityKind::AppBot => same && actor.kind == ActorKind::Bot,
            _ => same,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bot_identity(login: Option<&str>) -> PostingIdentity {
        PostingIdentity::new(
            login.map(str::to_string),
            IdentityKind::AppBot,
            TokenSource::RepoTokenCommand,
        )
    }

    #[test]
    fn authored_compares_logins_case_insensitively() {
        let me = PostingIdentity::new(
            Some("Octo".into()),
            IdentityKind::PersonalToken,
            TokenSource::Profile { id: "gh".into() },
        );
        assert_eq!(
            me.authored(&Actor::new("octo", "Octo", ActorKind::User)),
            Some(true)
        );
        assert_eq!(
            me.authored(&Actor::new("other", "Other", ActorKind::User)),
            Some(false)
        );
    }

    #[test]
    fn an_app_bot_needs_a_bot_author() {
        let me = bot_identity(Some("my-app[bot]"));
        assert_eq!(
            me.authored(&Actor::new("My-App[bot]", "x", ActorKind::Bot)),
            Some(true)
        );
        assert_eq!(
            me.authored(&Actor::new("my-app[bot]", "x", ActorKind::User)),
            Some(false)
        );
    }

    #[test]
    fn an_unknown_login_cannot_tell() {
        let me = bot_identity(None);
        assert_eq!(me.display, UNKNOWN_APP_BOT);
        assert_eq!(
            me.authored(&Actor::new("a[bot]", "a", ActorKind::Bot)),
            None
        );
        assert_eq!(bot_identity(Some("  ")).login, None);
    }
}
