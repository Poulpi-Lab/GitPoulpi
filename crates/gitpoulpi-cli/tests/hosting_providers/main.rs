//! The `hosting` CLI against each new provider (Azure DevOps, Bitbucket, Gitea,
//! Jira, Trello, Linear, YouTrack, Redmine, Gerrit), over a throwaway loopback mock: profile validation,
//! links with a filter, lists carrying `reference`, issue creation with a
//! type / list, `issue options`, `open issue`, `refs`, and pull request
//! checkout by branch. Assertions are on deserialized JSON; nothing
//! reaches the internet.

#[cfg(any(feature = "local-issues", feature = "jira"))]
#[cfg(feature = "board")]
mod board;
mod common;
mod integrations;
mod refs;
#[cfg(feature = "gitlab")]
mod trust;

#[cfg(feature = "azure-devops")]
mod azure;
#[cfg(feature = "bitbucket")]
mod bitbucket;
#[cfg(feature = "bitbucket")]
mod bitbucket_dc;
#[cfg(feature = "gerrit")]
mod gerrit;
#[cfg(feature = "gitea")]
mod gitea;
#[cfg(feature = "jira")]
mod jira;
#[cfg(feature = "linear")]
mod linear;
#[cfg(feature = "local-issues")]
mod local_markdown;
#[cfg(feature = "gitlab")]
mod radar;
#[cfg(feature = "redmine")]
mod redmine;
#[cfg(feature = "trello")]
mod trello;
#[cfg(feature = "youtrack")]
mod youtrack;

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
