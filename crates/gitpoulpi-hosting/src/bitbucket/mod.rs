//! Bitbucket Cloud hosting client (slice S4).
//!
//! Verified 2026-10-08 against Atlassian's published OpenAPI document
//! (`https://api.bitbucket.org/swagger.json`, the machine-readable form of
//! the REST reference at `https://developer.atlassian.com/cloud/bitbucket/rest/`)
//! and the support pages `using-api-tokens` / `api-token-permissions`:
//!
//! - Auth: Basic `email:api_token`, or Bearer with the token alone
//!   (`crate::auth`). Git over HTTPS uses the user `x-bitbucket-api-token-auth`.
//! - `GET /2.0/user` -> `uuid`, `account_id`, `nickname`, `display_name`,
//!   `links.avatar`/`links.html`. Token scope `read:user:bitbucket`.
//! - Workspaces: the cross-workspace `GET /2.0/workspaces` and
//!   `/2.0/user/permissions/workspaces` are gone (CHANGE-2770), but
//!   `GET /2.0/user/workspaces` (`values[].workspace.slug`, paged) is still
//!   in the reference, so discovery uses it; no `workspaces` config field was
//!   needed. Scope `read:workspace:bitbucket`.
//! - `GET /2.0/repositories/{workspace}?role=member&sort=-updated_on`.
//! - Pull requests: `.../pullrequests?state=OPEN&state=MERGED...` (the
//!   parameter repeats), `q=source.branch.name="x"` (filtering syntax),
//!   `pagelen` (default 10; the pull request endpoints cap at 50), `next` is
//!   a full URL. `state` is `OPEN|MERGED|DECLINED|SUPERSEDED`; `draft` is a
//!   boolean. Participants carry `role` (`PARTICIPANT|REVIEWER`) and
//!   `approved`. Create: `{title, description, source:{branch:{name},
//!   repository:{full_name}}, destination:{branch:{name}}, draft,
//!   close_source_branch}`; the POST url is the destination repository.
//! - SSH keys: `GET/POST /2.0/users/{selected_user}/ssh-keys`, `DELETE
//!   .../{key_id}`; `selected_user` is the user's `uuid` (or `account_id`);
//!   key fields `uuid`, `key`, `label`, `comment`. Scopes
//!   `read|write|delete:ssh-key:bitbucket`.
//! - Errors are `{"type":"error","error":{"message":...}}`.
//! - Not documented anywhere (kept from the plan): the new-pull-request web
//!   page `/pull-requests/new?source=&dest=`.
//! - Issues and wikis were retired on 2026-08-20: `issues` is unsupported.

pub mod client;
#[cfg(test)]
mod conversation_tests;
pub mod dc;
mod map;
pub mod spec;
#[cfg(test)]
mod tests;

pub use client::BitbucketClient;
