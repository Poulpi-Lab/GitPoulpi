//! The edited choices of the SSH & credentials page, as plain text and flags
//! the rows edit, and their conversion to and from the profile the daemon
//! stores (`RepoAuthProfile`). Pure: no IO, so the rows can be rebuilt every
//! frame and the conversions are unit-tested.

use gitpoulpi_core::domain::{HttpsAuth, RepoAuthProfile, SshAuth};
use gitpoulpi_i18n::msg::repo_auth as t;

/// How HTTPS remotes sign in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum HttpsKind {
    #[default]
    System,
    Account,
    Helper,
    TokenCommand,
}

/// How SSH remotes sign in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum SshKind {
    #[default]
    System,
    KeyFile,
    AccountKey,
}

/// Everything the page edits. Text is kept as typed.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct Draft {
    pub(crate) https: HttpsKind,
    pub(crate) username: String,
    pub(crate) ignore_repo_helpers: bool,
    pub(crate) account: String,
    pub(crate) helper: String,
    /// The token command as typed (split by [`split_command`]).
    pub(crate) command: String,
    pub(crate) hosts: String,
    pub(crate) cache: String,
    pub(crate) ssh: SshKind,
    pub(crate) key: String,
    pub(crate) ssh_account: String,
    pub(crate) known_hosts: String,
    pub(crate) strict: bool,
}

/// Splits a command line into arguments: spaces separate, a single or double
/// quote groups, a backslash escapes inside double quotes only when it
/// precedes a quote or a backslash. No shell runs the result.
pub(crate) fn split_command(text: &str) -> Result<Vec<String>, ()> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut has = false;
    let mut quote: Option<char> = None;
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        match (quote, c) {
            (Some(q), c) if c == q => quote = None,
            (Some('"'), '\\') if matches!(chars.peek(), Some('"' | '\\')) => {
                cur.push(chars.next().unwrap_or('\\'));
            }
            (Some(_), c) => cur.push(c),
            (None, '"' | '\'') => {
                quote = Some(c);
                has = true;
            }
            (None, c) if c.is_whitespace() => {
                if has || !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                    has = false;
                }
            }
            (None, c) => cur.push(c),
        }
    }
    if quote.is_some() {
        return Err(());
    }
    if has || !cur.is_empty() {
        out.push(cur);
    }
    Ok(out)
}

/// The inverse of [`split_command`], for showing a stored argv in the field.
pub(crate) fn join_command(argv: &[String]) -> String {
    argv.iter()
        .map(|a| {
            if a.is_empty() || a.chars().any(|c| c.is_whitespace() || c == '"' || c == '\'') {
                format!("\"{}\"", a.replace('\\', "\\\\").replace('"', "\\\""))
            } else {
                a.clone()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn split_hosts(text: &str) -> Vec<String> {
    text.split([',', ' ', ';'])
        .map(str::trim)
        .filter(|h| !h.is_empty())
        .map(str::to_string)
        .collect()
}

fn some(text: &str) -> Option<String> {
    let text = text.trim();
    (!text.is_empty()).then(|| text.to_string())
}

impl Draft {
    /// The draft of a stored profile.
    pub(crate) fn from_profile(profile: &RepoAuthProfile) -> Self {
        let mut d = Draft::default();
        match &profile.https {
            None => {}
            Some(HttpsAuth::System {
                username,
                ignore_repo_helpers,
            }) => {
                d.username = username.clone().unwrap_or_default();
                d.ignore_repo_helpers = *ignore_repo_helpers;
            }
            Some(HttpsAuth::Integration {
                integration,
                username,
            }) => {
                d.https = HttpsKind::Account;
                d.account = integration.clone();
                d.username = username.clone().unwrap_or_default();
            }
            Some(HttpsAuth::Helper {
                helper,
                username,
                hosts,
                ..
            }) => {
                d.https = HttpsKind::Helper;
                d.helper = helper.clone();
                d.username = username.clone().unwrap_or_default();
                d.hosts = hosts.join(", ");
            }
            Some(HttpsAuth::TokenCommand {
                argv,
                username,
                hosts,
                cache_minutes,
                ..
            }) => {
                d.https = HttpsKind::TokenCommand;
                d.command = join_command(argv);
                d.username = username.clone().unwrap_or_default();
                d.hosts = hosts.join(", ");
                d.cache = cache_minutes.map(|m| m.to_string()).unwrap_or_default();
            }
        }
        match &profile.ssh {
            None | Some(SshAuth::System) => {}
            Some(SshAuth::KeyFile {
                key,
                known_hosts,
                strict_host_key,
            }) => {
                d.ssh = SshKind::KeyFile;
                d.key = key.clone();
                d.known_hosts = known_hosts.clone().unwrap_or_default();
                d.strict = *strict_host_key;
            }
            Some(SshAuth::IntegrationKey {
                integration,
                known_hosts,
                strict_host_key,
            }) => {
                d.ssh = SshKind::AccountKey;
                d.ssh_account = integration.clone();
                d.known_hosts = known_hosts.clone().unwrap_or_default();
                d.strict = *strict_host_key;
            }
        }
        d
    }

    /// The profile this draft means. `stored` supplies the confirmation
    /// fields of an unchanged command, so saving only the SSH half does not
    /// ask again. `Err` is the reason it cannot be a profile.
    pub(crate) fn to_profile(&self, stored: &RepoAuthProfile) -> Result<RepoAuthProfile, String> {
        let username = some(&self.username);
        let https = match self.https {
            HttpsKind::System => {
                if username.is_none() && !self.ignore_repo_helpers {
                    None
                } else {
                    Some(HttpsAuth::System {
                        username,
                        ignore_repo_helpers: self.ignore_repo_helpers,
                    })
                }
            }
            HttpsKind::Account => {
                let integration = some(&self.account).ok_or_else(|| t::INVALID_ACCOUNT.get())?;
                Some(HttpsAuth::Integration {
                    integration,
                    username,
                })
            }
            HttpsKind::Helper => {
                let helper = some(&self.helper).ok_or_else(|| t::INVALID_HELPER.get())?;
                let (confirmed_sha256, script_sha256, confirmed_at) = match &stored.https {
                    Some(HttpsAuth::Helper {
                        helper: old,
                        confirmed_sha256,
                        script_sha256,
                        confirmed_at,
                        ..
                    }) if *old == helper => {
                        (confirmed_sha256.clone(), script_sha256.clone(), *confirmed_at)
                    }
                    _ => (None, None, None),
                };
                Some(HttpsAuth::Helper {
                    helper,
                    username,
                    hosts: split_hosts(&self.hosts),
                    confirmed_sha256,
                    script_sha256,
                    confirmed_at,
                })
            }
            HttpsKind::TokenCommand => {
                let argv = split_command(&self.command).map_err(|()| t::INVALID_QUOTE.get())?;
                if argv.is_empty() {
                    return Err(t::INVALID_COMMAND.get().to_string());
                }
                let cache_minutes = match self.cache.trim() {
                    "" => None,
                    n => Some(
                        n.parse::<u32>()
                            .map_err(|_| t::INVALID_CACHE.get().to_string())?,
                    ),
                };
                let (confirmed_sha256, script_sha256, confirmed_at) = match &stored.https {
                    Some(HttpsAuth::TokenCommand {
                        argv: old,
                        confirmed_sha256,
                        script_sha256,
                        confirmed_at,
                        ..
                    }) if *old == argv => {
                        (confirmed_sha256.clone(), script_sha256.clone(), *confirmed_at)
                    }
                    _ => (None, None, None),
                };
                Some(HttpsAuth::TokenCommand {
                    argv,
                    username,
                    hosts: split_hosts(&self.hosts),
                    cache_minutes,
                    confirmed_sha256,
                    script_sha256,
                    confirmed_at,
                })
            }
        };
        let known_hosts = some(&self.known_hosts);
        let ssh = match self.ssh {
            SshKind::System => None,
            SshKind::KeyFile => Some(SshAuth::KeyFile {
                key: some(&self.key).ok_or_else(|| t::INVALID_KEY.get())?,
                known_hosts,
                strict_host_key: self.strict,
            }),
            SshKind::AccountKey => Some(SshAuth::IntegrationKey {
                integration: some(&self.ssh_account).ok_or_else(|| t::INVALID_ACCOUNT.get())?,
                known_hosts,
                strict_host_key: self.strict,
            }),
        };
        Ok(RepoAuthProfile { https, ssh })
    }

    /// Whether saving this draft needs the person to confirm a command: it
    /// names a command-running source that is not the stored, confirmed one.
    pub(crate) fn needs_confirm(&self, stored: &RepoAuthProfile) -> bool {
        let Ok(profile) = self.to_profile(stored) else {
            return false;
        };
        match &profile.https {
            Some(HttpsAuth::Helper {
                confirmed_sha256, ..
            })
            | Some(HttpsAuth::TokenCommand {
                confirmed_sha256, ..
            }) => confirmed_sha256.is_none(),
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_command_splits_like_a_person_reads_it() {
        assert_eq!(
            split_command(r#"pass show "work token" 'a b'"#).unwrap(),
            ["pass", "show", "work token", "a b"]
        );
        assert_eq!(split_command("  a   b ").unwrap(), ["a", "b"]);
        assert_eq!(split_command(r#"x "" y"#).unwrap(), ["x", "", "y"]);
        assert!(split_command("a \"b").is_err());
        assert_eq!(split_command("").unwrap(), Vec::<String>::new());
    }

    #[test]
    fn join_and_split_round_trip() {
        let argv: Vec<String> = ["pwsh", "-File", "C:/my tools/get.ps1", "a\"b", ""]
            .map(String::from)
            .to_vec();
        assert_eq!(split_command(&join_command(&argv)).unwrap(), argv);
    }

    #[test]
    fn the_default_draft_is_no_profile() {
        let p = Draft::default().to_profile(&RepoAuthProfile::default()).unwrap();
        assert!(p.is_default() && p.https.is_none() && p.ssh.is_none());
    }

    #[test]
    fn a_profile_round_trips_through_the_draft() {
        let profile = RepoAuthProfile {
            https: Some(HttpsAuth::TokenCommand {
                argv: vec!["pass".into(), "show".into(), "my token".into()],
                username: Some("x-access-token".into()),
                hosts: vec!["github.com".into(), "git.example".into()],
                cache_minutes: Some(30),
                confirmed_sha256: Some("abc".into()),
                script_sha256: None,
                confirmed_at: Some(5),
            }),
            ssh: Some(SshAuth::KeyFile {
                key: "/k".into(),
                known_hosts: Some("/kh".into()),
                strict_host_key: true,
            }),
        };
        let draft = Draft::from_profile(&profile);
        assert_eq!(draft.https, HttpsKind::TokenCommand);
        assert_eq!(draft.to_profile(&profile).unwrap(), profile);
    }

    #[test]
    fn an_unchanged_command_keeps_its_confirmation_and_a_changed_one_asks() {
        let stored = RepoAuthProfile {
            https: Some(HttpsAuth::Helper {
                helper: "!get".into(),
                username: None,
                hosts: vec![],
                confirmed_sha256: Some("h".into()),
                script_sha256: None,
                confirmed_at: Some(1),
            }),
            ssh: None,
        };
        let mut draft = Draft::from_profile(&stored);
        assert!(!draft.needs_confirm(&stored));
        draft.helper = "!other".into();
        assert!(draft.needs_confirm(&stored));
        assert!(draft.needs_confirm(&RepoAuthProfile::default()));
        draft.https = HttpsKind::System;
        assert!(!draft.needs_confirm(&stored));
    }

    #[test]
    fn what_cannot_be_a_profile_says_why() {
        let none = RepoAuthProfile::default();
        let mut d = Draft {
            https: HttpsKind::Helper,
            ..Draft::default()
        };
        assert_eq!(d.to_profile(&none).unwrap_err(), t::INVALID_HELPER.get());
        d.https = HttpsKind::TokenCommand;
        assert_eq!(d.to_profile(&none).unwrap_err(), t::INVALID_COMMAND.get());
        d.command = "x \"y".into();
        assert_eq!(d.to_profile(&none).unwrap_err(), t::INVALID_QUOTE.get());
        d.command = "x".into();
        d.cache = "soon".into();
        assert_eq!(d.to_profile(&none).unwrap_err(), t::INVALID_CACHE.get());
        d = Draft {
            ssh: SshKind::KeyFile,
            ..Draft::default()
        };
        assert_eq!(d.to_profile(&none).unwrap_err(), t::INVALID_KEY.get());
        d.https = HttpsKind::Account;
        assert_eq!(d.to_profile(&none).unwrap_err(), t::INVALID_ACCOUNT.get());
    }
}
