//! `gitpoulpi hosting radar` (the account-wide lists) and `hosting
//! integrations radar` (a profile's `[integrations.radar]` settings).
//!
//! The lists come from [`gitpoulpi_hosting::HostingClient::account_items`]
//! and are cached per profile in `<config>/radar-account.json` for five
//! minutes, the same file the Radar reads, so the CLI, an agent over MCP and
//! the TUI share one answer instead of each spending the host's search quota.

use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use gitpoulpi_config::{IntegrationCategory, IntegrationRadar, RemoteReviewMode};
use gitpoulpi_hosting::account::{self, AccountCache, AccountQuery, CachedAccount};
use gitpoulpi_hosting::Integration;
use serde_json::{json, Value};

use crate::cli::{OnOff, RadarWhat, RemoteReviewArg};
use crate::repo_cmd::Emitted;

use super::HostingEnv;

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn cache_file(env: &HostingEnv) -> PathBuf {
    let dir = env
        .config_file
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_default();
    account::cache_path(&dir)
}

/// The question a profile is asked: the explicit `--what`, else its saved
/// Radar settings.
fn query_for(integration: &Integration, what: Option<RadarWhat>, drafts: bool) -> AccountQuery {
    let radar = integration.radar_options();
    let (review_requests, open_requests, assigned_issues) = match what {
        None => (
            radar.review_requests,
            radar.open_requests,
            radar.assigned_issues,
        ),
        Some(RadarWhat::Review) => (true, false, false),
        Some(RadarWhat::Open) => (false, true, false),
        Some(RadarWhat::Issues) => (false, false, true),
        Some(RadarWhat::All) => (true, true, true),
    };
    AccountQuery {
        review_requests,
        open_requests,
        assigned_issues,
        include_drafts: drafts || radar.include_drafts,
        namespaces: radar.namespaces,
        namespaces_regex: radar.namespaces_regex,
        limit: 0,
    }
}

/// One profile's answer, from the cache or the host.
struct Round {
    fetched_at: u64,
    cached: bool,
    items: account::AccountItems,
}

async fn round(
    env: &HostingEnv,
    integrations: &[Integration],
    integration: &Integration,
    query: &AccountQuery,
    refresh: bool,
) -> Result<Round, String> {
    let path = cache_file(env);
    let hash = query.fingerprint();
    let at = now();
    if !refresh {
        if let Some(hit) = AccountCache::load(&path).fresh(&integration.id, &hash, at) {
            return Ok(Round {
                fetched_at: hit.fetched_at,
                cached: true,
                items: hit.items.clone(),
            });
        }
    }
    let items = env
        .client(integration, integrations)?
        .account_items(query)
        .await
        .map_err(|e| e.to_string())?;
    // A cache that cannot be written only costs the next call a fetch.
    let _ = AccountCache::store(
        &path,
        &integration.id,
        CachedAccount {
            fetched_at: at,
            query_hash: hash,
            items: items.clone(),
        },
    );
    Ok(Round {
        fetched_at: at,
        cached: false,
        items,
    })
}

/// `hosting radar`.
pub(super) async fn run_radar(
    env: &HostingEnv,
    ids: Vec<String>,
    what: Option<RadarWhat>,
    include_drafts: bool,
    refresh: bool,
) -> Result<Emitted, String> {
    let integrations = env.integrations()?;
    let explicit = !ids.is_empty();
    let mut chosen: Vec<&Integration> = Vec::new();
    if explicit {
        for id in &ids {
            let found = integrations.iter().find(|i| &i.id == id).ok_or_else(|| {
                format!(
                    "no integration `{id}`; configured: {}",
                    integrations
                        .iter()
                        .map(|i| i.id.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            })?;
            if !account::supports(found.kind) {
                let why = if found.kind == gitpoulpi_hosting::IntegrationKind::Bitbucket {
                    "Bitbucket Cloud has no account-wide pull request endpoint (its lists are per \
                     repository); Bitbucket Data Center does"
                        .to_string()
                } else {
                    format!("`{id}` ({}) has no account-wide lists", found.kind.label())
                };
                return Err(format!(
                    "{why}; supported: GitHub, GitLab, Gitea and Forgejo, Azure DevOps, \
                     Bitbucket Data Center, Gerrit"
                ));
            }
            if !chosen.iter().any(|c| c.id == found.id) {
                chosen.push(found);
            }
        }
    } else {
        chosen.extend(integrations.iter().filter(|i| {
            i.enabled
                && account::supports(i.kind)
                && (what.is_some() || i.radar_options().any_list())
        }));
    }

    let mut profiles: Vec<Value> = Vec::new();
    let mut lines: Vec<String> = Vec::new();
    for integration in chosen {
        let query = query_for(integration, what, include_drafts);
        if query.is_empty() {
            return Err(format!(
                "`{}` has no Radar list turned on; pass --what review|open|issues|all, or run \
                 `gitpoulpi hosting integrations radar {} --review-requests on`",
                integration.id, integration.id
            ));
        }
        match round(env, &integrations, integration, &query, refresh).await {
            Ok(r) => {
                lines.push(format!(
                    "{}: {} request(s), {} issue(s){}{}",
                    integration.id,
                    r.items.prs.len(),
                    r.items.issues.len(),
                    if r.cached { " (cached)" } else { "" },
                    if r.items.truncated { " (cut)" } else { "" },
                ));
                for p in &r.items.prs {
                    lines.push(format!(
                        "  {} {} {}{}{}",
                        p.project,
                        p.pr.iid_label,
                        p.pr.title,
                        if p.pr.review_requested {
                            "  [review]"
                        } else {
                            ""
                        },
                        if p.pr.state == gitpoulpi_hosting::PullRequestState::Draft {
                            "  [draft]"
                        } else {
                            ""
                        },
                    ));
                }
                for i in &r.items.issues {
                    lines.push(format!(
                        "  {} {} {}  [issue]",
                        i.project, i.issue.iid_label, i.issue.title
                    ));
                }
                for note in &r.items.notes {
                    lines.push(format!("  note: {note}"));
                }
                profiles.push(json!({
                    "id": integration.id,
                    "fetched_at": r.fetched_at,
                    "cached": r.cached,
                    "prs": r.items.prs,
                    "issues": r.items.issues,
                    "truncated": r.items.truncated,
                    "notes": r.items.notes,
                    "error": Value::Null,
                }));
            }
            // One profile alone fails the command; among several, the rest
            // still answer and the failure sits in its own entry.
            Err(e) if explicit && ids.len() == 1 => return Err(e),
            Err(e) => {
                lines.push(format!("{}: {e}", integration.id));
                profiles.push(json!({
                    "id": integration.id,
                    "fetched_at": 0,
                    "cached": false,
                    "prs": [],
                    "issues": [],
                    "truncated": false,
                    "notes": [],
                    "error": e,
                }));
            }
        }
    }
    let human = if profiles.is_empty() {
        "no profile has Radar lists turned on (see `gitpoulpi hosting integrations radar`)"
            .to_string()
    } else {
        lines.join("\n")
    };
    Ok(Emitted {
        exit_code: 0,
        json: json!({ "profiles": profiles }),
        human,
    })
}

/// The flags of `hosting integrations radar`.
pub(super) struct RadarEdit {
    pub id: String,
    pub review_requests: Option<OnOff>,
    pub open_requests: Option<OnOff>,
    pub assigned_issues: Option<OnOff>,
    pub include_drafts: Option<OnOff>,
    pub namespaces: Vec<String>,
    pub clear_namespaces: bool,
    pub review_mode: Option<RemoteReviewArg>,
}

fn radar_json(radar: &IntegrationRadar) -> Value {
    json!({
        "review_requests": radar.review_requests,
        "open_requests": radar.open_requests,
        "include_drafts": radar.include_drafts,
        "assigned_issues": radar.assigned_issues,
        "namespaces": radar.namespaces,
        "namespaces_regex": radar.namespaces_regex,
        "review_mode": radar.review_mode.as_str(),
    })
}

/// `hosting integrations radar <id> [...]`: prints, or edits then prints.
pub(super) fn run_settings(env: &HostingEnv, edit: RadarEdit) -> Result<Emitted, String> {
    let mut integrations = env.integrations()?;
    let position = integrations
        .iter()
        .position(|i| i.id == edit.id)
        .ok_or_else(|| format!("no integration `{}`", edit.id))?;
    let integration = &mut integrations[position];
    if integration.kind.category() != IntegrationCategory::Hosting {
        return Err(format!(
            "`{}` ({}) is an issue tracker: Radar settings belong to code hosts",
            edit.id,
            integration.kind.label()
        ));
    }
    let on = |v: OnOff| v == OnOff::On;
    let mut radar = integration.radar_options();
    let before = radar.clone();
    if let Some(v) = edit.review_requests {
        radar.review_requests = on(v);
    }
    if let Some(v) = edit.open_requests {
        radar.open_requests = on(v);
    }
    if let Some(v) = edit.assigned_issues {
        radar.assigned_issues = on(v);
    }
    if let Some(v) = edit.include_drafts {
        radar.include_drafts = on(v);
    }
    if edit.clear_namespaces {
        radar.namespaces.clear();
    } else if !edit.namespaces.is_empty() {
        let mut spaces: Vec<String> = Vec::new();
        for ns in edit
            .namespaces
            .iter()
            .map(|n| n.trim())
            .filter(|n| !n.is_empty())
        {
            if !spaces.iter().any(|s| s.eq_ignore_ascii_case(ns)) {
                spaces.push(ns.to_string());
            }
        }
        radar.namespaces = spaces;
    }
    if let Some(mode) = edit.review_mode {
        radar.review_mode = match mode {
            RemoteReviewArg::Ask => RemoteReviewMode::Ask,
            RemoteReviewArg::Clone => RemoteReviewMode::Clone,
            RemoteReviewArg::Api => RemoteReviewMode::Api,
        };
    }
    let changed = radar != before;
    if changed {
        // All-off defaults leave no table behind.
        integration.radar = (radar != IntegrationRadar::default()).then(|| radar.clone());
        env.save_integrations(&integrations)?;
    }
    let human = format!(
        "{}: review requests {}, open requests {}, assigned issues {}, drafts {}, namespaces {}, \
         not-cloned review {}",
        edit.id,
        on_off(radar.review_requests),
        on_off(radar.open_requests),
        on_off(radar.assigned_issues),
        on_off(radar.include_drafts),
        if radar.namespaces.is_empty() {
            "(membership)".to_string()
        } else {
            radar.namespaces.join(", ")
        },
        radar.review_mode.as_str(),
    );
    Ok(Emitted {
        exit_code: 0,
        json: json!({ "id": edit.id, "radar": radar_json(&radar), "changed": changed }),
        human,
    })
}

fn on_off(value: bool) -> &'static str {
    if value {
        "on"
    } else {
        "off"
    }
}
