//! `gitpoulpi hosting` -- integration profiles, what a token can see,
//! pull/merge requests and issues of the current repository, cloning, and
//! SSH keys on the host.
//!
//! Everything here is client-side (hosting stays out of the daemon):
//! profiles live in `config.toml`'s `[[integrations]]`, tokens in the
//! `gitpoulpi-secrets` store under each profile's `token_ref`, and every
//! request goes through a [`HostingEnv`]'s transport factory so tests can
//! substitute a `RecordedTransport` for the network.

use std::path::PathBuf;
use std::sync::Arc;

use gitpoulpi_core::domain::{CloneOptions, OperationKind, RepoRef};
use gitpoulpi_hosting::{
    pull_request_edit_url, pull_request_web_url, DefaultTokenSource, HostedProject, HttpTransport,
    Integration, IssueFilter, MetaKind, PrFilter, TokenSource,
};
use gitpoulpi_secrets::TokenStore;
use serde_json::json;

#[cfg(feature = "board")]
use crate::cli::BoardAction;
use crate::cli::{HostingAction, IssueAction, LinkKind, OpenKind, ReviewAction};
use crate::client::Session;
use crate::repo_cmd::Emitted;

mod assignment;
#[cfg(feature = "board")]
mod board_cmd;
pub mod flow_issue;
mod integrations;
mod keys;
mod link;
mod local;
mod meta;
mod pr_create;
mod protection;
mod prs;
mod radar;
mod refs;
mod review;
mod review_api;
#[cfg(feature = "ai")]
mod review_ctx;
#[cfg(test)]
mod trust_tests;

use self::integrations::*;
use self::keys::*;
use self::link::*;
use self::meta::*;
use self::pr_create::*;
use self::prs::*;
use self::refs::*;

type TransportFactory = Box<dyn Fn() -> Result<Arc<dyn HttpTransport>, String> + Send + Sync>;

/// Everything `hosting` touches outside the process: the config file, the
/// token store and the HTTP transport. [`HostingEnv::ambient`] is the
/// production wiring; tests build one over a temp config, an in-memory
/// store and a `RecordedTransport`.
pub struct HostingEnv {
    pub config_file: PathBuf,
    store: Arc<dyn TokenStore + Send + Sync>,
    transport: TransportFactory,
    /// Where the trust store and the caches live.
    pub paths: gitpoulpi_config::ConfigPaths,
    /// The integration trust check every client goes through.
    pub gate: gitpoulpi_hosting::TrustGate,
}

impl HostingEnv {
    /// An environment whose trust store and caches sit next to
    /// `config_file`, as a test lays them out.
    pub fn new(
        config_file: PathBuf,
        store: Arc<dyn TokenStore + Send + Sync>,
        transport: TransportFactory,
    ) -> Self {
        let dir = config_file
            .parent()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        Self::with_paths(
            config_file,
            store,
            transport,
            gitpoulpi_config::ConfigPaths::for_dir(dir),
        )
    }

    pub fn with_paths(
        config_file: PathBuf,
        store: Arc<dyn TokenStore + Send + Sync>,
        transport: TransportFactory,
        paths: gitpoulpi_config::ConfigPaths,
    ) -> Self {
        let gate = gitpoulpi_hosting::TrustGate::load(&paths);
        Self {
            config_file,
            store,
            transport,
            paths,
            gate,
        }
    }

    /// The real config directory, the git credential helper and reqwest.
    pub fn ambient() -> Result<Self, String> {
        let paths = gitpoulpi_config::ConfigPaths::discover()
            .ok_or_else(|| "could not determine this OS's config directory".to_string())?;
        Ok(Self::with_paths(
            paths.config_file(),
            Arc::new(gitpoulpi_secrets::CredentialHelperTokens::new()),
            Box::new(|| {
                let transport =
                    gitpoulpi_hosting::ReqwestTransport::new().map_err(|e| e.to_string())?;
                Ok(Arc::new(transport) as Arc<dyn HttpTransport>)
            }),
            paths,
        ))
    }

    fn transport(&self) -> Result<Arc<dyn HttpTransport>, String> {
        (self.transport)()
    }

    /// The configured integrations, through the loader (migrations
    /// included) every other frontend uses. The *file* layer: what is
    /// written back by [`Self::save_integrations`] is what was read here.
    fn integrations(&self) -> Result<Vec<Integration>, String> {
        gitpoulpi_config::Config::load(&self.config_file)
            .map(|config| config.integrations)
            .map_err(|e| format!("config.toml could not be read: {e}"))
    }

    fn save_integrations(&self, integrations: &[Integration]) -> Result<(), String> {
        gitpoulpi_config::set_list_in_file(&self.config_file, "integrations", integrations)
            .map_err(|e| format!("config.toml could not be written: {e}"))
    }

    /// The hosting client for `integration`, with its token resolved by the
    /// default order (env var, stored token, unambiguous helper).
    ///
    /// The trust check comes first: an address that is not the recorded one
    /// stops here, before the token is even looked up.
    fn client(
        &self,
        integration: &Integration,
        integrations: &[Integration],
    ) -> Result<Box<dyn gitpoulpi_hosting::HostingClient>, String> {
        self.gate
            .preflight(integration)
            .map_err(|e| e.to_string())?;
        let source =
            DefaultTokenSource::new(Box::new(SharedStore(self.store.clone())), integrations);
        let token = source.token(integration).map_err(|e| e.to_string())?;
        if !integration.kind.capabilities().local_folder {
            // A token exists for it: an integration from before the trust
            // store gets its record now, from the config as it stands.
            self.gate.adopt(integration).map_err(|e| e.to_string())?;
        }
        gitpoulpi_hosting::client_for(integration.clone(), token, self.transport()?, &self.gate)
            .map_err(|e| e.to_string())
    }

    /// [`Self::client`], then the account check of
    /// [`gitpoulpi_hosting::TrustGate::verify_account`] when it is due: the
    /// token must still belong to the account that was trusted.
    async fn client_checked(
        &self,
        integration: &Integration,
        integrations: &[Integration],
    ) -> Result<Box<dyn gitpoulpi_hosting::HostingClient>, String> {
        let client = self.client(integration, integrations)?;
        if !integration.kind.capabilities().local_folder {
            let own_token = self
                .store
                .load(&integration.token_ref)
                .ok()
                .flatten()
                .is_some_and(|t| !t.trim().is_empty())
                && std::env::var_os(gitpoulpi_hosting::token_env_var(&integration.id)).is_none();
            let reason = if own_token {
                gitpoulpi_hosting::trust::VerifyReason::Periodic
            } else {
                gitpoulpi_hosting::trust::VerifyReason::ExternalToken
            };
            self.gate
                .verify_account(integration, client.as_ref(), reason)
                .await
                .map_err(|e| e.to_string())?;
        }
        Ok(client)
    }
}

/// Lets the default token source and the env share one store.
struct SharedStore(Arc<dyn TokenStore + Send + Sync>);

impl TokenStore for SharedStore {
    fn store(&self, token_ref: &str, token: &str) -> gitpoulpi_secrets::Result<()> {
        self.0.store(token_ref, token)
    }
    fn load(&self, token_ref: &str) -> gitpoulpi_secrets::Result<Option<String>> {
        self.0.load(token_ref)
    }
    fn delete(&self, token_ref: &str) -> gitpoulpi_secrets::Result<()> {
        self.0.delete(token_ref)
    }
    fn session_only_reason(&self, token_ref: &str) -> Option<String> {
        self.0.session_only_reason(token_ref)
    }
}

/// Whether this action needs the session's daemon: `clone` (a
/// `GitBackend` operation, like every mutation) and every action that
/// reads the current repository's remotes or branches to find its project.
pub fn needs_daemon(action: &HostingAction) -> bool {
    #[cfg(feature = "board")]
    if matches!(action, HostingAction::Board { .. }) {
        return true;
    }
    // A review of a named project (`--project`) reads the host only; `file`
    // always does.
    if let HostingAction::Review { action } = action {
        return action.project().is_none();
    }
    matches!(
        action,
        HostingAction::Clone { .. }
            | HostingAction::Pr { .. }
            | HostingAction::Prs { .. }
            | HostingAction::PrForBranch { .. }
            | HostingAction::CheckoutPr { .. }
            | HostingAction::Issues { .. }
            | HostingAction::Issue { .. }
            | HostingAction::Open { .. }
            | HostingAction::Refs { .. }
            | HostingAction::EditUrl { .. }
            | HostingAction::Use { .. }
            | HostingAction::Assignment { .. }
            | HostingAction::Link { .. }
            | HostingAction::PrEdit { .. }
            | HostingAction::Members { .. }
            | HostingAction::Labels { .. }
            | HostingAction::Milestones { .. }
    )
}

/// `hosting review <action>` in a repository: one repository-scoped hosting
/// call each (`fetch` also writes a ref through the daemon).
async fn run_review(
    env: &HostingEnv,
    ipc: &mut Session,
    action: ReviewAction,
) -> Result<Emitted, String> {
    // A named project needs no repository, so none is consulted.
    if action.project().is_some() {
        return run_review_with(env, None, action).await;
    }
    run_review_with(env, Some(ipc), action).await
}

/// `hosting review --project P <action>`: no repository, no daemon.
async fn run_review_local(env: &HostingEnv, action: ReviewAction) -> Result<Emitted, String> {
    run_review_with(env, None, action).await
}

/// The repository-bound actions say so instead of unwrapping.
fn repo_only<'a>(ipc: Option<&'a mut Session>, what: &str) -> Result<&'a mut Session, String> {
    ipc.ok_or_else(|| format!("`review {what}` works on the current repository, not on --project"))
}

async fn run_review_with(
    env: &HostingEnv,
    mut ipc: Option<&mut Session>,
    action: ReviewAction,
) -> Result<Emitted, String> {
    let id = match &action {
        ReviewAction::Show { id, .. }
        | ReviewAction::Diff { id, .. }
        | ReviewAction::Threads { id, .. }
        | ReviewAction::Comment { id, .. }
        | ReviewAction::Reply { id, .. }
        | ReviewAction::Resolve { id, .. }
        | ReviewAction::Submit { id, .. }
        | ReviewAction::Fetch { id, .. }
        | ReviewAction::Push { id, .. }
        | ReviewAction::SubmitChange { id, .. }
        | ReviewAction::Abandon { id, .. }
        | ReviewAction::Suggest { id, .. } => id.clone(),
    };
    let target = match (action.project(), ipc.as_deref_mut()) {
        (Some(project), _) => review_api::project_target(env, id.as_deref(), project)?,
        (None, Some(ipc)) => repo_target(env, ipc, id.as_deref(), Which::Pulls).await?,
        (None, None) => return Err("not inside a git repository (pass --project)".to_string()),
    };
    match action {
        ReviewAction::Show {
            number,
            patch_set,
            compare_with,
            ..
        } => review::show(env, &target, number, patch_set, compare_with).await,
        ReviewAction::Diff {
            number,
            api,
            file,
            remote,
            ..
        } => {
            // `--api` reads the host even inside a clone.
            let ipc = if api { None } else { ipc };
            review::diff(env, ipc, &target, number, file, remote).await
        }
        ReviewAction::Threads {
            number,
            unresolved,
            patch_set,
            ..
        } => review::threads(env, &target, number, unresolved, patch_set).await,
        ReviewAction::Comment {
            number,
            body,
            body_file,
            ..
        } => {
            let body = required_body(read_issue_body(body, body_file)?)?;
            review::comment(env, &target, number, body).await
        }
        ReviewAction::Reply {
            number,
            thread,
            body,
            body_file,
            ..
        } => {
            let body = required_body(read_issue_body(body, body_file)?)?;
            review::reply(env, &target, number, &thread, body).await
        }
        ReviewAction::Resolve {
            number,
            thread,
            unresolve,
            ..
        } => review::resolve(env, &target, number, &thread, !unresolve).await,
        ReviewAction::Submit {
            number,
            event,
            body,
            body_file,
            comments,
            votes,
            patch_set,
            ..
        } => {
            let body = read_issue_body(body, body_file)?;
            review::review(
                env, &target, number, event, body, &comments, &votes, patch_set,
            )
            .await
        }
        ReviewAction::Fetch {
            number,
            remote,
            patch_set,
            compare_with,
            ..
        } => {
            let ipc = repo_only(ipc, "fetch")?;
            review::fetch(env, ipc, &target, number, remote, patch_set, compare_with).await
        }
        ReviewAction::SubmitChange { number, .. } => {
            review::submit_change(env, &target, number).await
        }
        ReviewAction::Abandon {
            number, message, ..
        } => review::abandon(env, &target, number, message).await,
        ReviewAction::Push {
            branch,
            topic,
            remote,
            ..
        } => {
            let ipc = repo_only(ipc, "push")?;
            review::push(env, ipc, &target, &branch, topic, remote).await
        }
        ReviewAction::Suggest {
            number,
            remote,
            run_in_repo,
            claude_md,
            agents_md,
            context_file,
            no_context_file,
            send_all,
            ..
        } => {
            let run = crate::cli::ReviewFlags {
                run_in_repo,
                claude_md,
                agents_md,
                context_file,
                no_context_file,
                no_extra_file: None,
            }
            .values();
            review::suggest(env, ipc, &target, number, remote, run, send_all).await
        }
    }
}

/// A comment or reply with no text is refused before any request.
fn required_body(body: String) -> Result<String, String> {
    if body.trim().is_empty() {
        Err("the text is empty (--body or --body-file)".to_string())
    } else {
        Ok(body)
    }
}

/// Picks the integration a command acts on: an explicit `--id` wins; with
/// none, a single configured integration is used implicitly and several is
/// an error naming them, rather than a silent pick of the first, which
/// would send a token to the wrong place.
fn pick<'a>(integrations: &'a [Integration], id: Option<&str>) -> Result<&'a Integration, String> {
    match id {
        Some(id) => {
            integrations
                .iter()
                .find(|i| i.id == id)
                .ok_or_else(|| match integrations.len() {
                    0 => no_integrations_message(),
                    _ => format!(
                        "no integration `{id}`; configured: {}",
                        integrations
                            .iter()
                            .map(|i| i.id.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                })
        }
        None => match integrations {
            [] => Err(no_integrations_message()),
            [only] => Ok(only),
            many => Err(format!(
                "several integrations are configured ({}); pass --id",
                many.iter()
                    .map(|i| i.id.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            )),
        },
    }
}

/// The refusal text an agent (MCP) or a script reads when the security check
/// of an integration needs a person.
pub const TRUST_REFUSAL: &str = "This integration's security check needs the user: open \
    GitPoulpi or run `gitpoulpi hosting integrations trust`";

/// A trust refusal as the CLI answers it: exit code 2 and the change in JSON,
/// instead of a bare failure. Only the problem the error text carries is
/// taken, so a refusal queued by another task is not mistaken for this one.
pub(crate) fn trust_answer(result: Result<Emitted, String>) -> Result<Emitted, String> {
    let Err(message) = result else {
        return result;
    };
    // A repository that blocks assignment: refused, not failed.
    if message.contains(gitpoulpi_hosting::scope::ASSIGNMENT_BLOCKED) {
        return Ok(assignment::assignment_refusal(&message));
    }
    let queue = gitpoulpi_hosting::trust::take_pending();
    let (mine, rest): (Vec<_>, Vec<_>) = queue
        .into_iter()
        .partition(|p| message.contains(&p.to_string()));
    for other in &rest {
        gitpoulpi_hosting::trust::report(other);
    }
    match mine.into_iter().next() {
        Some(problem) => Ok(trust_refusal(&problem)),
        None => Err(message),
    }
}

pub(crate) fn trust_refusal(problem: &gitpoulpi_hosting::TrustProblem) -> Emitted {
    use gitpoulpi_hosting::TrustSubject;
    let (kind, name) = match problem.subject() {
        TrustSubject::Ai(p) => ("ai", p.as_str()),
        TrustSubject::Integration(i) => ("integration", i.as_str()),
        TrustSubject::Host(h) => ("host", h.as_str()),
    };
    Emitted {
        exit_code: 2,
        json: json!({
            "error": "trust_check",
            "subject": name,
            "subject_kind": kind,
            "change": problem.change(),
            "old": problem.old_text(),
            "new": problem.new_text(),
            "fix": problem.fix(),
            "message": problem.to_string(),
        }),
        human: format!("{TRUST_REFUSAL}: {problem}"),
    }
}

/// Everything that needs no repository (and so no daemon), against the
/// real config, credential store and network.
pub async fn run_local(action: HostingAction) -> Result<Emitted, String> {
    run_local_with(&HostingEnv::ambient()?, action).await
}

/// [`run_local`] over an explicit environment, for tests.
pub async fn run_local_with(env: &HostingEnv, action: HostingAction) -> Result<Emitted, String> {
    trust_answer(run_local_inner(env, action).await)
}

async fn run_local_inner(env: &HostingEnv, action: HostingAction) -> Result<Emitted, String> {
    match action {
        HostingAction::List => integrations_list(env),
        HostingAction::Integrations { action } => run_integrations(env, action).await,
        HostingAction::Login { id, token_stdin } => {
            if !token_stdin {
                return Err(
                    "pass --token-stdin and pipe the token in (it is never accepted as an \
                     argument, to keep it out of shell history)"
                        .into(),
                );
            }
            let token = read_token_stdin()?;
            login(env, id.as_deref(), &token).await
        }
        HostingAction::Logout { id } => logout(env, id.as_deref()),

        HostingAction::Projects { id, filter } => {
            let integrations = env.integrations()?;
            let integration = pick(&integrations, id.as_deref())?;
            let projects = env
                .client_checked(integration, &integrations)
                .await?
                .list_projects()
                .await
                .map_err(|e| e.to_string())?;
            let filter = filter.unwrap_or_default();
            let projects: Vec<HostedProject> = projects
                .into_iter()
                .filter(|p| p.matches(&filter))
                .collect();
            let human = projects
                .iter()
                .map(|p| {
                    format!(
                        "{:<50} {}",
                        p.full_name,
                        p.clone_url(true).unwrap_or("(no clone url)")
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");
            Ok(Emitted {
                exit_code: 0,
                json: serde_json::to_value(&projects)
                    .map_err(|e| format!("failed to encode output: {e}"))?,
                human,
            })
        }

        HostingAction::SshKey { action } => run_ssh_key(env, action).await,

        HostingAction::Radar {
            ids,
            what,
            include_drafts,
            refresh,
        } => radar::run_radar(env, ids, what, include_drafts, refresh).await,
        HostingAction::File(args) => review_api::file(env, args).await,
        // Only a named project gets here (`needs_daemon`).
        HostingAction::Review { action } => run_review_local(env, action).await,

        HostingAction::Clone { .. }
        | HostingAction::Pr { .. }
        | HostingAction::Prs { .. }
        | HostingAction::PrForBranch { .. }
        | HostingAction::CheckoutPr { .. }
        | HostingAction::Issues { .. }
        | HostingAction::Issue { .. }
        | HostingAction::Open { .. }
        | HostingAction::Refs { .. }
        | HostingAction::EditUrl { .. }
        | HostingAction::Use { .. }
        | HostingAction::Assignment { .. }
        | HostingAction::Link { .. }
        | HostingAction::PrEdit { .. }
        | HostingAction::Members { .. }
        | HostingAction::Labels { .. }
        | HostingAction::Milestones { .. } => {
            unreachable!("handled by run_via_daemon")
        }
        #[cfg(feature = "board")]
        HostingAction::Board { .. } => unreachable!("handled by run_via_daemon"),
    }
}

/// `hosting clone`, `pr` and the repository-scoped queries - the actions
/// that need the daemon: the former performs a `GitBackend::clone`, the
/// rest read the current repository's branches and remotes to find its
/// project and fill in whatever was not typed on the command line.
pub async fn run_via_daemon(ipc: &mut Session, action: HostingAction) -> Result<Emitted, String> {
    let env = HostingEnv::ambient()?;
    run_via_daemon_with(&env, ipc, action).await
}

/// [`run_via_daemon`] over an explicit environment.
pub async fn run_via_daemon_with(
    env: &HostingEnv,
    ipc: &mut Session,
    action: HostingAction,
) -> Result<Emitted, String> {
    trust_answer(run_via_daemon_inner(env, ipc, action).await)
}

async fn run_via_daemon_inner(
    env: &HostingEnv,
    ipc: &mut Session,
    action: HostingAction,
) -> Result<Emitted, String> {
    match action {
        HostingAction::Pr {
            source,
            target,
            title,
            body,
            draft,
            id,
            web,
            assignees,
            reviewers,
            labels,
            milestone,
        } => {
            let meta = NewMeta {
                assignees: people(assignees),
                reviewers: people(reviewers),
                labels: words(labels),
                milestone,
            };
            return run_pr(env, ipc, source, target, title, body, draft, id, web, meta).await;
        }
        HostingAction::PrEdit {
            number,
            meta,
            add_reviewer,
            remove_reviewer,
            id,
        } => {
            let target = repo_target(env, ipc, id.as_deref(), Which::Pulls).await?;
            let ask = EditAsk::new(meta, add_reviewer, remove_reviewer);
            return edit(env, &target, number, MetaKind::PullRequest, ask).await;
        }
        HostingAction::Issue {
            action: IssueAction::Edit { number, meta, id },
        } => {
            let target = repo_target(env, ipc, id.as_deref(), Which::Issues).await?;
            let ask = EditAsk::new(meta, Vec::new(), Vec::new());
            return edit(env, &target, number, MetaKind::Issue, ask).await;
        }
        HostingAction::Members { query, source, id } => {
            let target = repo_target(env, ipc, id.as_deref(), source_which(source)).await?;
            return members(env, &target, query).await;
        }
        HostingAction::Labels { query, source, id } => {
            let target = repo_target(env, ipc, id.as_deref(), source_which(source)).await?;
            return labels(env, &target, query).await;
        }
        HostingAction::Milestones { query, source, id } => {
            let target = repo_target(env, ipc, id.as_deref(), source_which(source)).await?;
            return milestones(env, &target, query).await;
        }
        HostingAction::Prs {
            state,
            mine,
            review_requested,
            id,
        } => {
            let target = repo_target(env, ipc, id.as_deref(), Which::Pulls).await?;
            return prs(
                env,
                &target,
                PrFilter {
                    state: state_filter(state),
                    mine,
                    review_requested,
                    ..Default::default()
                },
            )
            .await;
        }
        HostingAction::CheckoutPr {
            number,
            remote,
            id,
            no_checkout,
        } => {
            let target = repo_target(env, ipc, id.as_deref(), Which::Pulls).await?;
            let (remote_ref, branch) = checkout_ref(env, &target, number).await?;
            let remote = match remote {
                Some(r) => r,
                None if !target.remote.is_empty() => target.remote.clone(),
                None => default_remote_name(ipc).await?,
            };
            ipc.call::<gitpoulpi_core::domain::FetchOutcome>(
                OperationKind::FetchRefAsBranch,
                json!({ "remote": remote, "remote_ref": remote_ref, "branch": branch }),
            )
            .await?;
            if !no_checkout {
                ipc.call::<()>(
                    OperationKind::Checkout,
                    json!({ "target": gitpoulpi_core::domain::CheckoutTarget::Branch(branch.clone()) }),
                )
                .await?;
            }
            let human = if no_checkout {
                format!("fetched {remote_ref} from {remote} into {branch}")
            } else {
                format!("switched to {branch} (tracking {remote_ref} from {remote})")
            };
            return Ok(Emitted {
                exit_code: 0,
                json: json!({
                    "number": number,
                    "remote": remote,
                    "ref": remote_ref,
                    "branch": branch,
                    "checked_out": !no_checkout,
                }),
                human,
            });
        }
        HostingAction::Review { action } => {
            return run_review(env, ipc, action).await;
        }
        HostingAction::PrForBranch { branch, id } => {
            let branch = match branch {
                Some(b) => b,
                None => current_branch(ipc).await?,
            };
            let target = repo_target(env, ipc, id.as_deref(), Which::Pulls).await?;
            return pr_for_branch(env, &target, &branch).await;
        }
        HostingAction::Issues {
            state,
            assigned,
            id,
        } => {
            let target = repo_target(env, ipc, id.as_deref(), Which::Issues).await?;
            return issues(
                env,
                &target,
                IssueFilter {
                    state: state_filter(state),
                    assigned_to_me: assigned,
                    query: target.filter.clone(),
                    ..Default::default()
                },
            )
            .await;
        }
        HostingAction::Issue {
            action:
                IssueAction::Create {
                    title,
                    body,
                    body_file,
                    labels,
                    issue_type,
                    list,
                    id,
                    web,
                    assignees,
                    milestone,
                },
        } => {
            let body = read_issue_body(body, body_file)?;
            let target = repo_target(env, ipc, id.as_deref(), Which::Issues).await?;
            let meta = NewMeta {
                assignees: people(assignees),
                milestone,
                ..NewMeta::default()
            };
            return issue_create(
                env,
                &target,
                title,
                body,
                labels,
                (issue_type, list),
                web,
                meta,
            )
            .await;
        }
        #[cfg(feature = "board")]
        HostingAction::Board { action, id } => {
            let (move_to, id) = match action {
                Some(BoardAction::Move {
                    reference,
                    column,
                    id: own,
                }) => (Some((reference, column)), own.or(id)),
                Some(BoardAction::Show { id: own }) => (None, own.or(id)),
                None => (None, id),
            };
            let target = repo_target(env, ipc, id.as_deref(), Which::Issues).await?;
            let labels = load_state(ipc)?.board_labels;
            return match move_to {
                None => board_cmd::show(env, &target, labels).await,
                Some((reference, column)) => {
                    board_cmd::move_card(env, &target, labels, &reference, &column).await
                }
            };
        }
        HostingAction::Issue {
            action: IssueAction::Close { number, id },
        } => {
            let target = repo_target(env, ipc, id.as_deref(), Which::Issues).await?;
            return local::set_state(env, &target, number, true).await;
        }
        HostingAction::Issue {
            action: IssueAction::Reopen { number, id },
        } => {
            let target = repo_target(env, ipc, id.as_deref(), Which::Issues).await?;
            return local::set_state(env, &target, number, false).await;
        }
        HostingAction::Issue {
            action: IssueAction::Move { number, status, id },
        } => {
            let target = repo_target(env, ipc, id.as_deref(), Which::Issues).await?;
            return local::move_to(env, &target, number, &status).await;
        }
        HostingAction::Issue {
            action: IssueAction::Base { id },
        } => {
            let target = repo_target(env, ipc, id.as_deref(), Which::Issues).await?;
            return local::create_base(env, &target).await;
        }
        HostingAction::Issue {
            action: IssueAction::Options { id },
        } => {
            let target = repo_target(env, ipc, id.as_deref(), Which::Issues).await?;
            return issue_options(env, &target).await;
        }
        HostingAction::Refs {
            message,
            commit,
            branch,
        } => return run_refs(env, ipc, message, commit, branch).await,
        HostingAction::Open {
            kind,
            reference,
            id,
            browser,
        } => {
            let which = match kind {
                OpenKind::Pr => Which::Pulls,
                OpenKind::Issue => Which::Issues,
            };
            let target = repo_target(env, ipc, id.as_deref(), which).await?;
            let mut emitted = open_url(&target, kind, &reference)?;
            if browser {
                open_in_browser(&emitted.human)?;
            }
            emitted.json["opened_in_browser"] = json!(browser);
            return Ok(emitted);
        }
        HostingAction::EditUrl {
            number,
            branch,
            id,
            open,
        } => {
            let target = repo_target(env, ipc, id.as_deref(), Which::Pulls).await?;
            let (web_url, number) = match number {
                Some(n) => {
                    let url = pull_request_web_url(&target.integration, &target.project, n)
                        .ok_or_else(|| {
                            format!(
                                "{} has no web URL scheme for this",
                                target.integration.kind.label()
                            )
                        })?;
                    (url, Some(n))
                }
                None => {
                    let branch = match branch {
                        Some(b) => b,
                        None => current_branch(ipc).await?,
                    };
                    let found = env
                        .client(&target.integration, &target.integrations)?
                        .pull_request_for_branch(&target.project, &branch)
                        .await
                        .map_err(|e| e.to_string())?
                        .ok_or_else(|| format!("no pull/merge request for `{branch}`"))?;
                    (found.web_url, None)
                }
            };
            let url = pull_request_edit_url(Some(target.integration.kind), &web_url);
            if open {
                open_in_browser(&url)?;
            }
            let mut json = merge(target_json(&target), "url", json!(url));
            if let Some(n) = number {
                json = merge(json, "number", json!(n));
            }
            json = merge(json, "opened_in_browser", json!(open));
            return Ok(Emitted {
                json,
                human: url,
                exit_code: 0,
            });
        }
        HostingAction::Use { id, clear } => return run_use(env, ipc, id, clear).await,
        HostingAction::Assignment { block, allow } => {
            return assignment::run_assignment(ipc, block, allow)
        }
        HostingAction::Link {
            kind,
            id,
            project,
            filter,
            clear,
        } => return run_link(env, ipc, kind, id, project, filter, clear).await,
        HostingAction::Clone { .. } => {}
        // Trusting is a person's decision: a session that is not the CLI's
        // (the MCP server) cannot.
        HostingAction::Integrations {
            action: crate::cli::IntegrationsAction::Trust { .. },
        } if !ipc.may_trust() => return Err(TRUST_REFUSAL.to_string()),
        other => return run_local_with(env, other).await,
    }

    let HostingAction::Clone {
        project,
        id,
        dest,
        https,
    } = action
    else {
        unreachable!("every other action returned above")
    };

    let integrations = env.integrations()?;
    let integration = pick(&integrations, id.as_deref())?;
    let projects = env
        .client(integration, &integrations)?
        .list_projects()
        .await
        .map_err(|e| e.to_string())?;

    // An exact full-path match first, then a unique substring, so
    // `gitpoulpi hosting clone thing` works when there is one `thing`
    // and says so when there are three. Picking the first of several
    // would clone the wrong repository silently.
    let wanted = project.trim().to_lowercase();
    let found = pick_project(&projects, &project, &wanted, &integration.id)?;

    let url = found
        .clone_url(!https)
        .ok_or_else(|| format!("`{}` has no clone url on this host", found.full_name))?
        .to_string();

    let dest = dest.unwrap_or_else(|| found.name.clone());
    let dest = crate::commands::absolute(dest)?;

    let repo: RepoRef = ipc
        .call_without_repo(
            OperationKind::Clone,
            json!({ "url": url, "dest": dest, "opts": CloneOptions::default() }),
        )
        .await?;

    let human = format!("cloned {} into {}", found.full_name, repo.root.display());
    Ok(Emitted {
        exit_code: 0,
        json: json!({ "project": found, "repo": repo }),
        human,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::{IntegrationsAction, SshKeyAction};
    use gitpoulpi_core::domain::Remote;
    use gitpoulpi_hosting::IntegrationKind;
    use gitpoulpi_hosting::RecordedTransport;
    use gitpoulpi_secrets::InMemoryTokens;
    use serde_json::Value;

    fn project(full_name: &str) -> HostedProject {
        HostedProject {
            integration_id: "h".into(),
            id: full_name.into(),
            name: full_name.rsplit('/').next().unwrap_or(full_name).into(),
            full_name: full_name.into(),
            description: None,
            default_branch: None,
            ssh_url: Some(format!("git@h:{full_name}.git")),
            https_url: None,
            web_url: None,
            private: false,
        }
    }

    /// An exact path must win even when it is also a prefix of others,
    /// which is the `group/app` vs `group/app-legacy` case.
    #[test]
    fn an_exact_path_beats_an_ambiguous_substring() {
        let projects = [project("group/app"), project("group/app-legacy")];
        let found = pick_project(&projects, "group/app", "group/app", "h").unwrap();
        assert_eq!(found.full_name, "group/app");
    }

    #[test]
    fn a_unique_substring_is_enough() {
        let projects = [project("group/app"), project("other/thing")];
        let found = pick_project(&projects, "thing", "thing", "h").unwrap();
        assert_eq!(found.full_name, "other/thing");
    }

    /// Cloning the wrong repository silently is only noticed later.
    #[test]
    fn an_ambiguous_substring_is_an_error_naming_the_candidates() {
        let projects = [project("group/app"), project("group/app-legacy")];
        let err = pick_project(&projects, "app", "app", "h").unwrap_err();
        assert!(
            err.contains("group/app") && err.contains("group/app-legacy"),
            "{err}"
        );
    }

    #[test]
    fn no_match_names_the_integration_that_was_searched() {
        let err = pick_project(&[project("group/app")], "nope", "nope", "corp").unwrap_err();
        assert!(err.contains("nope") && err.contains("corp"), "{err}");
    }

    fn integration(id: &str) -> Integration {
        Integration::new(
            id,
            IntegrationKind::GitlabSelfManaged,
            format!("https://{id}.example.com"),
        )
    }

    #[test]
    fn one_configured_integration_is_used_without_being_named() {
        let all = vec![integration("only")];
        assert_eq!(pick(&all, None).unwrap().id, "only");
    }

    /// Picking the first of several would send a token to a host the
    /// user did not name.
    #[test]
    fn several_integrations_require_an_explicit_id() {
        let all = vec![integration("a"), integration("b")];
        let err = pick(&all, None).unwrap_err();
        assert!(err.contains("--id"), "{err}");
        assert!(err.contains('a') && err.contains('b'), "{err}");
        assert_eq!(pick(&all, Some("b")).unwrap().id, "b");
    }

    #[test]
    fn an_unknown_id_names_the_ones_that_exist() {
        let all = vec![integration("a")];
        let err = pick(&all, Some("nope")).unwrap_err();
        assert!(
            err.contains("nope") && err.contains("configured: a"),
            "{err}"
        );
    }

    /// The empty case has to teach the table's shape, not just report
    /// its absence.
    #[test]
    fn no_integrations_tells_the_user_what_to_write() {
        let err = pick(&[], None).unwrap_err();
        for expected in ["[[integrations]]", "token_ref", "GITPOULPI_TOKEN_"] {
            assert!(err.contains(expected), "missing {expected} in:\n{err}");
        }
    }

    // ---- profile management, login and repo-scoped queries -------------

    fn block_on<F: std::future::Future>(f: F) -> F::Output {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(f)
    }

    struct Fixture {
        _dir: tempfile::TempDir,
        env: HostingEnv,
        store: Arc<InMemoryTokens>,
        transport: Arc<RecordedTransport>,
    }

    fn fixture(transport: RecordedTransport) -> Fixture {
        let dir = tempfile::tempdir().unwrap();
        let store = Arc::new(InMemoryTokens::default());
        let transport = Arc::new(transport);
        let for_env = transport.clone();
        let env = HostingEnv::new(
            dir.path().join("config.toml"),
            store.clone(),
            Box::new(move || Ok(for_env.clone() as Arc<dyn HttpTransport>)),
        );
        Fixture {
            _dir: dir,
            env,
            store,
            transport,
        }
    }

    const USER: &str =
        r#"{"login":"octo","id":9,"name":"Octo Cat","html_url":"https://github.com/octo"}"#;

    fn add(env: &HostingEnv, kind: &str, url: Option<&str>, id: Option<&str>) {
        block_on(run_integrations(
            env,
            IntegrationsAction::Add {
                kind: kind.into(),
                url: url.map(Into::into),
                org: None,
                account: None,
                path: None,
                id: id.map(Into::into),
                name: None,
            },
        ))
        .unwrap();
    }

    #[test]
    fn add_list_remove_round_trips_through_the_config_file() {
        let f = fixture(RecordedTransport::new());
        add(&f.env, "github", None, None);
        add(
            &f.env,
            "gitlab_self_managed",
            Some("gitlab.example.com/"),
            Some("corp"),
        );

        let listed = integrations_list(&f.env).unwrap().json;
        let rows = listed["integrations"].as_array().unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0]["id"], "github");
        assert_eq!(rows[0]["api_base"], "https://api.github.com");
        assert_eq!(rows[1]["url"], "https://gitlab.example.com");
        assert_eq!(rows[1]["token_ref"], "integration:corp");

        f.store.store("integration:corp", "secret").unwrap();
        let removed = block_on(run_integrations(
            &f.env,
            IntegrationsAction::Remove { id: "corp".into() },
        ))
        .unwrap();
        assert_eq!(removed.json["token_deleted"], true);
        assert!(!f.store.has("integration:corp"), "token goes with profile");
        assert_eq!(
            integrations_list(&f.env).unwrap().json["integrations"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
    }

    #[test]
    fn add_refuses_bad_input() {
        let f = fixture(RecordedTransport::new());
        let run = |kind: &str, url: Option<&str>, id: Option<&str>| {
            block_on(run_integrations(
                &f.env,
                IntegrationsAction::Add {
                    kind: kind.into(),
                    url: url.map(Into::into),
                    org: None,
                    account: None,
                    path: None,
                    id: id.map(Into::into),
                    name: None,
                },
            ))
            .err()
        };
        assert!(run("jira_cloud", Some("acme.atlassian.net"), None)
            .unwrap()
            .contains("--account"));
        assert!(run("nope", None, None).unwrap().contains("unknown kind"));
        assert!(run("gitlab_self_managed", None, None)
            .unwrap()
            .contains("--url is required"));
        assert!(run("github", None, Some("bad id!"))
            .unwrap()
            .contains("not a valid id"));
        add(&f.env, "github", None, Some("dup"));
        assert!(run("github", None, Some("dup"))
            .unwrap()
            .contains("already exists"));
    }

    /// A valid token is stored under the profile's `token_ref`, the user is
    /// cached in the config, and the token appears nowhere in the output
    /// or the config file.
    #[test]
    fn login_stores_the_token_and_the_connected_user() {
        let f = fixture(RecordedTransport::new().on("/user", 200, USER));
        add(&f.env, "github", None, None);
        let out = block_on(login(&f.env, None, "ghp_supersecret")).unwrap();
        assert_eq!(out.json["user"]["login"], "octo");
        assert_eq!(
            f.store.load("integration:github").unwrap().as_deref(),
            Some("ghp_supersecret")
        );
        let raw = format!("{}{}", out.json, out.human);
        assert!(!raw.contains("ghp_supersecret"));
        let file = std::fs::read_to_string(&f.env.config_file).unwrap();
        assert!(!file.contains("ghp_supersecret"));
        assert!(file.contains("octo"), "user cached: {file}");
        let auth = &f.transport.requests()[0].headers;
        assert!(auth.iter().any(|(_, v)| v.contains("ghp_supersecret")));
    }

    /// A store that keeps the token for the session only (the OS store
    /// refused it) is reported as such, with its error; a persisting store
    /// reports `os`. The fake store never touches the real credential store.
    #[test]
    fn login_says_where_the_token_lives() {
        struct SessionOnly(InMemoryTokens);
        impl TokenStore for SessionOnly {
            fn store(&self, r: &str, t: &str) -> gitpoulpi_secrets::Result<()> {
                self.0.store(r, t)
            }
            fn load(&self, r: &str) -> gitpoulpi_secrets::Result<Option<String>> {
                self.0.load(r)
            }
            fn delete(&self, r: &str) -> gitpoulpi_secrets::Result<()> {
                self.0.delete(r)
            }
            fn session_only_reason(&self, _: &str) -> Option<String> {
                Some("no credential helper configured".into())
            }
        }
        let f = fixture(RecordedTransport::new().on("/user", 200, USER));
        add(&f.env, "github", None, None);
        let ok = block_on(login(&f.env, None, "tok")).unwrap();
        assert_eq!(ok.json["stored"], "os");
        assert!(ok.json["store_error"].is_null());
        assert_eq!(ok.json["token_stored"], true);

        let transport = Arc::new(RecordedTransport::new().on("/user", 200, USER));
        let dir = tempfile::tempdir().unwrap();
        let env = HostingEnv::new(
            dir.path().join("config.toml"),
            Arc::new(SessionOnly(InMemoryTokens::default())),
            Box::new(move || Ok(transport.clone() as Arc<dyn HttpTransport>)),
        );
        add(&env, "github", None, None);
        let out = block_on(login(&env, None, "tok")).unwrap();
        assert_eq!(out.json["user"]["login"], "octo");
        assert_eq!(out.json["stored"], "session");
        assert_eq!(out.json["store_error"], "no credential helper configured");
        assert!(out.human.contains("this session only"), "{}", out.human);
    }

    /// Jira Cloud signs in with the email and the token; the email lives
    /// in the config, the token only in the store.
    #[test]
    #[cfg(feature = "jira")]
    fn a_jira_cloud_login_sends_the_email_and_stores_only_the_token() {
        let f = fixture(RecordedTransport::new().on(
            "/rest/api/3/myself",
            200,
            r#"{"accountId":"a1","displayName":"Me","emailAddress":"me@x.io"}"#,
        ));
        block_on(run_integrations(
            &f.env,
            IntegrationsAction::Add {
                kind: "jira_cloud".into(),
                url: Some("https://acme.atlassian.net".into()),
                org: None,
                account: Some("me@x.io".into()),
                path: None,
                id: None,
                name: None,
            },
        ))
        .unwrap();
        let out = block_on(login(&f.env, None, "jira-secret")).unwrap();
        assert_eq!(out.json["user"]["login"], "me@x.io");
        assert_eq!(
            f.store.load("integration:jira-cloud").unwrap().as_deref(),
            Some("jira-secret")
        );
        let request = &f.transport.requests()[0];
        assert_eq!(request.url, "https://acme.atlassian.net/rest/api/3/myself");
        // base64("me@x.io:jira-secret")
        assert!(request
            .headers
            .iter()
            .any(|(n, v)| n == "Authorization" && v == "Basic bWVAeC5pbzpqaXJhLXNlY3JldA=="));
        let file = std::fs::read_to_string(&f.env.config_file).unwrap();
        assert!(file.contains("account = \"me@x.io\""), "{file}");
        assert!(!file.contains("jira-secret"), "{file}");
    }

    #[test]
    fn a_profile_is_checked_against_what_its_kind_needs() {
        use super::integrations::profile_parts;
        let azure = IntegrationKind::AzureDevops;
        assert_eq!(
            profile_parts(azure, None, Some("acme"), None).unwrap().0,
            "https://dev.azure.com/acme"
        );
        assert!(profile_parts(azure, None, None, None)
            .unwrap_err()
            .contains("--org"));
        assert!(
            profile_parts(azure, Some("https://dev.azure.com/a"), Some("b"), None)
                .unwrap_err()
                .contains("does not match")
        );
        assert!(
            profile_parts(IntegrationKind::AzureDevopsServer, None, Some("a"), None)
                .unwrap_err()
                .contains("--org")
        );
        assert!(
            profile_parts(IntegrationKind::Github, None, None, Some("k"))
                .unwrap_err()
                .contains("--account")
        );
        let (url, account) = profile_parts(IntegrationKind::Bitbucket, None, None, None).unwrap();
        assert_eq!(
            (url.as_str(), account.as_str()),
            ("https://bitbucket.org", "")
        );
    }

    #[test]
    fn a_rejected_token_is_not_stored() {
        let f = fixture(RecordedTransport::new().on("/user", 401, r#"{"message":"Bad"}"#));
        add(&f.env, "github", None, None);
        let err = block_on(login(&f.env, None, "bad")).unwrap_err();
        assert!(err.contains("rejected"), "{err}");
        assert!(!f.store.has("integration:github"));
    }

    #[test]
    fn logout_deletes_the_token_and_keeps_the_profile() {
        let f = fixture(RecordedTransport::new().on("/user", 200, USER));
        add(&f.env, "github", None, None);
        block_on(login(&f.env, None, "tok")).unwrap();
        logout(&f.env, None).unwrap();
        assert!(!f.store.has("integration:github"));
        let rows = integrations_list(&f.env).unwrap().json;
        assert_eq!(rows["integrations"][0]["user"], Value::Null);
    }

    #[test]
    fn test_reports_scopes_without_changing_anything() {
        let f = fixture(RecordedTransport::new().on_with_headers(
            "/user",
            200,
            USER,
            &[("X-OAuth-Scopes", "repo")],
        ));
        add(&f.env, "github", None, None);
        f.store.store("integration:github", "tok").unwrap();
        let out = block_on(run_integrations(
            &f.env,
            IntegrationsAction::Test { id: None },
        ))
        .unwrap();
        assert_eq!(out.json["missing_scopes"][0], "read:user");
        assert!(out.human.contains("MISSING"));
    }

    #[test]
    fn test_reports_excess_scopes_and_the_token_flavor() {
        let f = fixture(RecordedTransport::new().on_with_headers(
            "/user",
            200,
            USER,
            &[(
                "X-OAuth-Scopes",
                "repo, read:user, admin:public_key, delete_repo",
            )],
        ));
        add(&f.env, "github", None, None);
        f.store.store("integration:github", "ghp_tok").unwrap();
        let out = block_on(run_integrations(
            &f.env,
            IntegrationsAction::Test { id: None },
        ))
        .unwrap();
        assert_eq!(out.json["excess_scopes"][0], "delete_repo");
        assert_eq!(out.json["token_flavor"], "classic");
        assert_eq!(out.json["recommend_fine_grained"], true);
        assert!(out.human.contains("more rights than needed"));
    }

    #[test]
    fn test_with_exact_scopes_carries_empty_excess_fields() {
        let f = fixture(RecordedTransport::new().on_with_headers(
            "/user",
            200,
            USER,
            &[("X-OAuth-Scopes", "repo, read:user, admin:public_key")],
        ));
        add(&f.env, "github", None, None);
        f.store.store("integration:github", "tok").unwrap();
        let out = block_on(run_integrations(
            &f.env,
            IntegrationsAction::Test { id: None },
        ))
        .unwrap();
        assert_eq!(out.json["excess_scopes"], serde_json::json!([]));
        assert_eq!(out.json["recommend_fine_grained"], false);
        assert_eq!(out.json["expires_at"], Value::Null);
        assert!(!out.human.contains("more rights"));
    }

    fn remote(name: &str, url: &str) -> Remote {
        Remote {
            name: name.into(),
            fetch_url: url.into(),
            push_url: None,
        }
    }

    #[test]
    fn prs_go_to_the_project_the_remote_resolves_to() {
        let body = r#"[{"number":4,"title":"Fix","state":"open","html_url":"https://github.com/me/thing/pull/4",
            "updated_at":"2026-01-04T00:00:00Z","user":{"login":"octo"},
            "head":{"ref":"feat","repo":{"full_name":"me/thing"}},
            "base":{"ref":"main","repo":{"full_name":"me/thing"}}}]"#;
        let f = fixture(RecordedTransport::new().on("/pulls?", 200, body));
        add(&f.env, "github", None, None);
        f.store.store("integration:github", "tok").unwrap();
        let target = resolve_target(
            &[remote("origin", "git@github.com:me/thing.git")],
            f.env.integrations().unwrap(),
            None,
            None,
        )
        .unwrap();
        assert_eq!(target.project, "me/thing");
        let out = block_on(prs(&f.env, &target, PrFilter::default())).unwrap();
        assert_eq!(out.json["integration"], "github");
        assert_eq!(out.json["pull_requests"][0]["iid_label"], "#4");
        assert!(f.transport.requests()[0]
            .url
            .contains("/repos/me/thing/pulls"));

        let none = block_on(pr_for_branch(&f.env, &target, "feat")).unwrap();
        assert_eq!(none.json["branch"], "feat");
    }

    #[test]
    fn open_builds_the_web_url_without_a_network_call() {
        let f = fixture(RecordedTransport::new());
        add(&f.env, "gitlab", None, None);
        let target = resolve_target(
            &[remote("origin", "https://gitlab.com/g/sub/p.git")],
            f.env.integrations().unwrap(),
            None,
            None,
        )
        .unwrap();
        let pr = open_url(&target, OpenKind::Pr, "!53").unwrap();
        assert_eq!(pr.human, "https://gitlab.com/g/sub/p/-/merge_requests/53");
        let issue = open_url(&target, OpenKind::Issue, "#7").unwrap();
        assert_eq!(issue.human, "https://gitlab.com/g/sub/p/-/issues/7");
        assert!(f.transport.requests().is_empty());
    }

    #[test]
    fn id_beats_the_pinned_profile_which_beats_host_resolution() {
        let a = Integration::new(
            "a",
            IntegrationKind::GitlabSelfManaged,
            "https://gl.example.com",
        );
        let b = Integration::new(
            "b",
            IntegrationKind::GitlabSelfManaged,
            "https://gl.example.com",
        );
        let remotes = [remote("origin", "https://gl.example.com/g/p.git")];
        let by = |id: Option<&str>, pinned: Option<&str>| {
            resolve_target(&remotes, vec![a.clone(), b.clone()], id, pinned)
                .unwrap()
                .integration
                .id
        };
        assert_eq!(by(None, None), "a");
        assert_eq!(by(None, Some("b")), "b");
        assert_eq!(by(Some("a"), Some("b")), "a");
        assert!(resolve_target(&remotes, vec![a], Some("zzz"), None).is_err());
    }

    #[test]
    fn no_matching_remote_says_how_to_fix_it() {
        let err = resolve_target(
            &[remote("origin", "https://elsewhere.example.org/g/p.git")],
            vec![integration("corp")],
            None,
            None,
        )
        .err()
        .unwrap();
        assert!(
            err.contains("hosting use") && err.contains("elsewhere"),
            "{err}"
        );
    }

    #[test]
    fn a_saved_link_decides_and_an_unlinked_repository_fetches_nothing() {
        use gitpoulpi_session::{IssuesSource, RepoState};
        let remotes = [remote("origin", "https://gl.example.com/g/p.git")];
        let all = vec![Integration::new(
            "a",
            IntegrationKind::GitlabSelfManaged,
            "https://gl.example.com",
        )];
        let mut state = RepoState::default();
        // Unlinked: an error that says how to link, however well the remote matches.
        for which in [Which::Pulls, Which::Issues] {
            let err = target_from_state(which, &state, &remotes, all.clone(), None)
                .err()
                .unwrap();
            assert!(err.contains("nothing is fetched"), "{err}");
            assert!(
                err.contains(&format!("hosting link {}", which.arg())),
                "{err}"
            );
        }
        // Linked pulls: the link's project, not the remote's; issues stay unlinked.
        state.pulls_source = Some(IssuesSource {
            integration: "a".into(),
            project: "other/proj".into(),
            filter: None,
        });
        let target = target_from_state(Which::Pulls, &state, &remotes, all.clone(), None).unwrap();
        assert_eq!(target.project, "other/proj");
        assert!(target.linked);
        assert!(target_from_state(Which::Issues, &state, &remotes, all.clone(), None).is_err());
        // An explicit --id is a one-off override: the remote's project.
        let by_id =
            target_from_state(Which::Issues, &state, &remotes, all.clone(), Some("a")).unwrap();
        assert_eq!(by_id.project, "g/p");
        assert!(!by_id.linked);
        // A link to a profile that is gone, or with no repository, is explained.
        let gone = target_from_state(Which::Pulls, &state, &remotes, vec![], None)
            .err()
            .unwrap();
        assert!(gone.contains("not configured any more"), "{gone}");
        state.pulls_source = Some(IssuesSource {
            integration: "a".into(),
            project: String::new(),
            filter: None,
        });
        let empty = target_from_state(Which::Pulls, &state, &remotes, all, None)
            .err()
            .unwrap();
        assert!(empty.contains("no repository is chosen"), "{empty}");
    }

    #[test]
    fn use_pins_clears_and_validates() {
        let all = vec![integration("a")];
        let mut state = gitpoulpi_session::RepoState::default();
        assert!(apply_use(&mut state, &all, Some("a"), false).unwrap());
        assert_eq!(state.hosting_integration.as_deref(), Some("a"));
        assert!(apply_use(&mut state, &all, Some("zzz"), false).is_err());
        assert_eq!(state.hosting_integration.as_deref(), Some("a"));
        assert!(apply_use(&mut state, &all, None, true).unwrap());
        assert_eq!(state.hosting_integration, None);
        assert!(!apply_use(&mut state, &all, None, false).unwrap());
    }

    #[test]
    fn ssh_key_delete_calls_the_host_and_forgets_a_remembered_key() {
        let f = fixture(RecordedTransport::new().on("/user/keys/7", 204, ""));
        add(&f.env, "github", None, None);
        f.store.store("integration:github", "tok").unwrap();
        let mut all = f.env.integrations().unwrap();
        all[0].ssh = Some(gitpoulpi_config::IntegrationSsh {
            remote_key_id: Some("7".into()),
            ..Default::default()
        });
        f.env.save_integrations(&all).unwrap();

        let out = block_on(run_ssh_key(
            &f.env,
            SshKeyAction::Delete {
                key_id: "7".into(),
                id: None,
            },
        ))
        .unwrap();
        assert_eq!(out.json["deleted"], true);
        assert_eq!(f.transport.requests()[0].method.as_str(), "DELETE");
        assert_eq!(
            f.env.integrations().unwrap()[0]
                .ssh
                .as_ref()
                .unwrap()
                .remote_key_id,
            None
        );
    }
}
