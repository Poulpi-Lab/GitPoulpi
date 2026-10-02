//! Azure DevOps (Services and Server) hosting client: repositories, pull
//! requests and Boards work items (the "issues" half), over a personal
//! access token.
//!
//! Slice S3 of the more-providers design. Every endpoint
//! below was re-read on 2026-10-08 against the official 7.1 reference (the
//! "verify-first" step); where the page disagreed with the plan, the page
//! won (see the notes).
//!
//! # Endpoints (all `{org}` = `https://dev.azure.com/{org}` or the Server
//! collection root; `api-version` 7.1 on Services, 7.0 on Server)
//!
//! * Repositories, org-wide list:
//!   `GET {org}/_apis/git/repositories` --
//!   <https://learn.microsoft.com/en-us/rest/api/azure/devops/git/repositories/list?view=azure-devops-rest-7.1>
//! * Repository by **name or id**, project-scoped:
//!   `GET {org}/{project}/_apis/git/repositories/{repo}` --
//!   <https://learn.microsoft.com/en-us/rest/api/azure/devops/git/repositories/get-repository?view=azure-devops-rest-7.1>
//! * Pull requests: `GET|POST {org}/{project}/_apis/git/repositories/{repoId}/pullrequests`
//!   (`searchCriteria.status|creatorId|reviewerId|sourceRefName`, `$top`, `$skip`) --
//!   <https://learn.microsoft.com/en-us/rest/api/azure/devops/git/pull-requests/get-pull-requests?view=azure-devops-rest-7.1>,
//!   <https://learn.microsoft.com/en-us/rest/api/azure/devops/git/pull-requests/create?view=azure-devops-rest-7.1>
//! * Work items: WIQL `POST {org}/{project}/_apis/wit/wiql?$top=` then batch
//!   `GET {org}/{project}/_apis/wit/workitems?ids=&fields=` (200 ids max) and
//!   `POST {org}/{project}/_apis/wit/workitems/${type}` as
//!   `application/json-patch+json` --
//!   <https://learn.microsoft.com/en-us/rest/api/azure/devops/wit/wiql/query-by-wiql?view=azure-devops-rest-7.1>,
//!   <https://learn.microsoft.com/en-us/rest/api/azure/devops/wit/work-items/list?view=azure-devops-rest-7.1>,
//!   <https://learn.microsoft.com/en-us/rest/api/azure/devops/wit/work-items/create?view=azure-devops-rest-7.1>
//! * Work item types: `GET {org}/{project}/_apis/wit/workitemtypes` --
//!   <https://learn.microsoft.com/en-us/rest/api/azure/devops/wit/work-item-types/list?view=azure-devops-rest-7.1>
//!
//! # Verification notes (doc vs plan)
//!
//! * **`repositoryId` on the pull-request endpoints is documented as an
//!   id only** ("The repository ID of the pull request's target branch"),
//!   so the repo *name* is not sent. The project-scoped *Get Repository*
//!   documents "name or ID", so the client resolves name to id there first
//!   (cached per client). The plan allowed either.
//! * The create-PR sample answers **201** though the table lists 200; any
//!   2xx is accepted. The response has no browser URL: it is built from
//!   the spec (`.../_git/{repo}/pullrequest/{id}`).
//! * `forkSource` exists on `GitPullRequest` (a `GitForkRef` whose
//!   `repository` is the fork) but the create page documents no fork
//!   flow: a pull request from a fork stays `Unsupported`.
//! * Work item batch read: `_links.html` is only present with
//!   `$expand=all|links`, and `fields` cannot be combined with `$expand`;
//!   the browser URL is therefore built from the spec
//!   (`{org}/{project}/_workitems/edit/{id}`). The create sample's
//!   `_links.html` is the legacy `web/wi.aspx?pcguid=` form, not used.
//!   `System.AssignedTo` is an identity object in 7.1 but a
//!   `"Name <email>"` string in some samples: both are read.
//! * Closed work items: the state *category* needs per-type
//!   `workitemtypes/{type}/states`; the fallback names `Done`, `Closed`,
//!   `Removed`, `Resolved` are used (plan D4).
//!
//! # Still unverified (no official REST page)
//!
//! * `GET {org}/_apis/connectionData` for the current user with a PAT is
//!   only known from the extension SDK's `ConnectionData` type
//!   (`authenticatedUser` is an Id). The client reads it leniently and
//!   falls back to `GET {org}/_apis/projects?$top=1`, which only proves
//!   access: "mine" and "review requested" then answer `Unsupported`.
//! * Browser URLs other than the PR and work-item pages (`pullrequestcreate`,
//!   `?version=GB{branch}`, `commit/{sha}`), and `refs/pull/{id}/...`
//!   (never used: checkout fetches the source branch).
//! * An invalid PAT is reported by Azure DevOps as **203 with an HTML
//!   sign-in page**, not as 401; that is mapped to `Unauthorized`.
//! * The work item *new* form URL: left `None`.

mod client;
mod map;
mod remote;
pub mod spec;
#[cfg(test)]
mod tests;

pub use client::AzureDevopsClient;
