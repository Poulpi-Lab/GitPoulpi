//! `gitpoulpi hosting` -- integrations.

use gitpoulpi_config::{AccountField, ConnectedUser};
use gitpoulpi_hosting::{Integration, IntegrationKind};
use serde_json::{json, Value};

use crate::cli::IntegrationsAction;
use crate::repo_cmd::Emitted;

use super::*;

/// Says what to *write*, not merely that something is missing. A user
/// who has never configured an integration has no way to guess the
/// table's shape from "no integrations configured".
pub(super) fn no_integrations_message() -> String {
    let path = gitpoulpi_config::ConfigPaths::discover()
        .map(|p| p.config_file().display().to_string())
        .unwrap_or_else(|| "config.toml".to_string());
    format!(
        "no hosting integrations configured. Add one with \
         `gitpoulpi hosting integrations add --kind gitlab_self_managed --url https://...` \
         then `gitpoulpi hosting login --token-stdin`, or write it to {path}:\n\n\
         [[integrations]]\n\
         id = \"my-gitlab\"\n\
         kind = \"gitlab_self_managed\"  # github | github_enterprise | gitlab | gitlab_self_managed\n\
         url = \"https://gitlab.example.com\"\n\
         token_ref = \"integration:my-gitlab\"\n\n\
         The token itself is never written there: it is read from the \
         profile's stored token, the system credential helper for that host \
         (only when no other profile shares it), or the \
         GITPOULPI_TOKEN_<ID> environment variable."
    )
}

pub(super) fn integration_row(i: &Integration) -> Value {
    json!({
        "id": i.id,
        "kind": i.kind.as_str(),
        "name": i.display_name(),
        "url": i.url,
        "category": match i.kind.category() {
            gitpoulpi_config::IntegrationCategory::Hosting => "hosting",
            gitpoulpi_config::IntegrationCategory::IssueTracker => "issue_tracker",
        },
        // The non-secret identity next to the token (Atlassian email,
        // Trello API key); empty when the kind has none.
        "account": i.account,
        "org": i.azure_org(),
        "api_base": i.api_base(),
        // The reference, never the token. Printing the token here would
        // put it in shell history and in any log the output is piped into.
        "token_ref": i.token_ref,
        "enabled": i.enabled,
        "implemented": i.kind.capabilities().implemented,
        "user": i.user.as_ref().map(|u| json!({
            "login": u.login, "name": u.name, "id": u.id,
        })),
        // The account-wide Radar settings; null while none was ever saved.
        "radar": i.radar,
    })
}

pub(super) fn integrations_list(env: &HostingEnv) -> Result<Emitted, String> {
    let integrations = env.integrations()?;
    let rows: Vec<Value> = integrations.iter().map(integration_row).collect();
    let human = if rows.is_empty() {
        no_integrations_message()
    } else {
        integrations
            .iter()
            .map(|i| {
                format!(
                    "{:<16} {:<20} {}{}",
                    i.id,
                    i.kind.as_str(),
                    i.url,
                    i.user
                        .as_ref()
                        .map(|u| format!("  (@{})", u.login))
                        .unwrap_or_default()
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    Ok(Emitted {
        exit_code: 0,
        json: json!({ "integrations": rows }),
        human,
    })
}

pub(super) async fn run_integrations(
    env: &HostingEnv,
    action: IntegrationsAction,
) -> Result<Emitted, String> {
    match action {
        IntegrationsAction::List => integrations_list(env),

        IntegrationsAction::Add {
            kind,
            url,
            org,
            account,
            path,
            id,
            name,
        } => {
            let mut integrations = env.integrations()?;
            let kind = parse_kind(&kind)?;
            let local = kind.capabilities().local_folder;
            if path.is_some() && !local {
                return Err(format!(
                    "--path only applies to local_markdown, not {}",
                    kind.as_str()
                ));
            }
            let (url, account) = if local {
                let folder = path
                    .as_deref()
                    .or(url.as_deref())
                    .map(str::trim)
                    .filter(|p| !p.is_empty())
                    .ok_or("local_markdown needs --path <folder>")?;
                (folder.to_string(), String::new())
            } else {
                profile_parts(kind, url.as_deref(), org.as_deref(), account.as_deref())?
            };
            let id = match id {
                Some(id) => {
                    validate_id(&id)?;
                    if integrations.iter().any(|i| i.id == id) {
                        return Err(format!("an integration `{id}` already exists"));
                    }
                    id
                }
                None => Integration::new_id(kind, integrations.iter()),
            };
            let mut integration = Integration::new(id, kind, url);
            integration.account = account;
            if let Some(name) = name.filter(|n| !n.trim().is_empty()) {
                integration.name = name.trim().to_string();
            }
            if !local && integration.host().is_none() {
                return Err(format!("`{}` is not a usable base url", integration.url));
            }
            integrations.push(integration.clone());
            env.save_integrations(&integrations)?;
            let human = if local {
                format!(
                    "added `{}` ({}): link a repository with `gitpoulpi hosting link issues --id {}`; no token needed",
                    integration.id,
                    integration.kind.label(),
                    integration.id
                )
            } else {
                format!(
                    "added `{}` ({}) - now `gitpoulpi hosting login --id {} --token-stdin`",
                    integration.id,
                    integration.kind.label(),
                    integration.id
                )
            };
            Ok(Emitted {
                exit_code: 0,
                json: json!({ "added": integration_row(&integration) }),
                human,
            })
        }

        IntegrationsAction::Remove { id } => {
            let mut integrations = env.integrations()?;
            let position = integrations
                .iter()
                .position(|i| i.id == id)
                .ok_or_else(|| format!("no integration `{id}`"))?;
            let removed = integrations.remove(position);
            // The token goes with the profile; a failure to delete it must
            // not leave the profile half-removed, so it is reported instead.
            let token_error = env.store.delete(&removed.token_ref).err();
            env.save_integrations(&integrations)?;
            // Its trust record and everything cached from its host go too.
            gitpoulpi_hosting::trust::forget(&env.paths, &removed);
            let human = match &token_error {
                None => format!("removed `{id}` and its stored token"),
                Some(e) => format!(
                    "removed `{id}`, but its stored token could not be deleted ({e}); \
                     delete `{}` from your credential store by hand",
                    removed.token_ref
                ),
            };
            Ok(Emitted {
                exit_code: 0,
                json: json!({
                    "removed": id,
                    "token_deleted": token_error.is_none(),
                    "token_error": token_error.map(|e| e.to_string()),
                }),
                human,
            })
        }

        IntegrationsAction::Radar {
            id,
            review_requests,
            open_requests,
            assigned_issues,
            include_drafts,
            namespaces,
            clear_namespaces,
            review_mode,
        } => super::radar::run_settings(
            env,
            super::radar::RadarEdit {
                id,
                review_requests,
                open_requests,
                assigned_issues,
                include_drafts,
                namespaces,
                clear_namespaces,
                review_mode,
            },
        ),

        IntegrationsAction::Trust {
            id,
            accept_origin,
            accept_account,
            accept_certificate,
            strict_tls,
            allow_insecure_http,
        } => {
            trust_integration(
                env,
                &id,
                TrustOptions {
                    accept_origin,
                    accept_account,
                    accept_certificate,
                    strict_tls: strict_tls.map(|o| o == crate::cli::OnOff::On),
                    allow_insecure_http: allow_insecure_http.map(|o| o == crate::cli::OnOff::On),
                },
            )
            .await
        }

        IntegrationsAction::Test { id } => {
            let integrations = env.integrations()?;
            let integration = pick(&integrations, id.as_deref())?;
            let check = env
                .client(integration, &integrations)?
                .check_token()
                .await
                .map_err(|e| e.to_string())?;
            env.gate
                .check_account(
                    integration,
                    &gitpoulpi_hosting::Account {
                        login: check.user.login.clone(),
                        id: check.user.id.clone(),
                    },
                )
                .map_err(|e| e.to_string())?;
            let human = format!(
                "`{}`: token belongs to @{}{}{}",
                integration.id,
                check.user.login,
                match &check.scopes {
                    Some(s) => format!(", scopes: {}", s.join(", ")),
                    None => ", scopes unknown".to_string(),
                },
                if check.missing_scopes.is_empty() {
                    String::new()
                } else {
                    format!(" - MISSING: {}", check.missing_scopes.join(", "))
                }
            );
            Ok(Emitted {
                exit_code: 0,
                json: json!({
                    "id": integration.id,
                    "user": check.user,
                    "scopes": check.scopes,
                    "missing_scopes": check.missing_scopes,
                    "expires_at": check.expires_at,
                }),
                human,
            })
        }
    }
}

/// The implemented kinds only: a profile for a product with no client would
/// list fine and fail on every use.
pub(super) fn parse_kind(text: &str) -> Result<IntegrationKind, String> {
    let wanted = text.trim().to_ascii_lowercase().replace('-', "_");
    let kind = IntegrationKind::ALL
        .iter()
        .copied()
        .find(|k| k.as_str() == wanted)
        .ok_or_else(|| {
            format!(
                "unknown kind `{text}`; supported: {}",
                IntegrationKind::IMPLEMENTED
                    .iter()
                    .map(|k| k.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        })?;
    if !kind.capabilities().implemented {
        return Err(format!(
            "{} has no client yet; supported: {}",
            kind.label(),
            IntegrationKind::IMPLEMENTED
                .iter()
                .map(|k| k.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    Ok(kind)
}

fn azure_org_error(kind: IntegrationKind) -> String {
    format!(
        "{} needs --org <organization> or --url https://dev.azure.com/<organization>",
        kind.as_str()
    )
}

/// The `(url, account)` a new profile of `kind` is written with, after
/// checking the flags against what the kind's capabilities say it needs:
/// the account (Atlassian email, Trello API key) where there is one, the
/// organization on Azure DevOps Services, a host where the kind has no
/// public one. Nothing is written when this fails.
pub(super) fn profile_parts(
    kind: IntegrationKind,
    url: Option<&str>,
    org: Option<&str>,
    account: Option<&str>,
) -> Result<(String, String), String> {
    let caps = kind.capabilities();
    let org = org.map(str::trim).filter(|o| !o.is_empty());
    let account = account.map(str::trim).filter(|a| !a.is_empty());
    if org.is_some() && !caps.org {
        return Err(format!(
            "--org does not apply to {} (it is the Azure DevOps organization)",
            kind.label()
        ));
    }
    let account = match (caps.account, account) {
        (None, Some(_)) => {
            return Err(format!("--account does not apply to {}", kind.label()));
        }
        (None, None) => String::new(),
        (Some(_), Some(account)) => account.to_string(),
        // Optional: without an email the token is sent as a Bearer one.
        (Some(_), None) if kind == IntegrationKind::Bitbucket => String::new(),
        (Some(AccountField::Email), None) => {
            return Err(format!(
                "{} needs --account <the account email>",
                kind.as_str()
            ));
        }
        (Some(AccountField::ApiKey), None) => {
            return Err(format!("{} needs --account <your API key>", kind.as_str()));
        }
        (Some(AccountField::Username), None) => {
            return Err(format!(
                "{} needs --account <your user name>",
                kind.as_str()
            ));
        }
    };
    let url = match (url.map(str::trim).filter(|u| !u.is_empty()), org) {
        (None, Some(org)) => format!("https://dev.azure.com/{org}"),
        (url, _) => resolve_url(kind, url)?,
    };
    if kind.capabilities().org {
        let probe = Integration::new("probe", kind, url.clone());
        match (probe.azure_org(), org) {
            (None, _) => return Err(azure_org_error(kind)),
            (Some(found), Some(wanted)) if !found.eq_ignore_ascii_case(wanted) => {
                return Err(format!(
                    "--org `{wanted}` does not match the organization of --url (`{found}`)"
                ));
            }
            _ => {}
        }
    }
    Ok((url, account))
}

/// `--url` normalized (scheme defaulted to https, trailing `/` dropped), or
/// the kind's fixed public host when none was typed.
pub(super) fn resolve_url(kind: IntegrationKind, url: Option<&str>) -> Result<String, String> {
    match url.map(str::trim).filter(|u| !u.is_empty()) {
        Some(url) => {
            let url = if url.contains("://") {
                url.to_string()
            } else {
                format!("https://{url}")
            };
            Ok(url.trim_end_matches('/').to_string())
        }
        None => kind
            .default_url()
            .map(str::to_string)
            .ok_or_else(|| format!("--url is required for {}", kind.label())),
    }
}

pub(super) fn validate_id(id: &str) -> Result<(), String> {
    if id.is_empty()
        || !id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
    {
        return Err(format!(
            "`{id}` is not a valid id (letters, digits, `-`, `_` and `.` only)"
        ));
    }
    Ok(())
}

pub(super) fn read_token_stdin() -> Result<String, String> {
    let mut line = String::new();
    std::io::stdin()
        .read_line(&mut line)
        .map_err(|e| format!("failed to read the token from stdin: {e}"))?;
    let token = line.trim().to_string();
    if token.is_empty() {
        return Err("no token on stdin".into());
    }
    Ok(token)
}

/// Validates `token` against the host and only then stores it, so a typo
/// never replaces a working token. The token is never echoed.
pub(super) async fn login(
    env: &HostingEnv,
    id: Option<&str>,
    token: &str,
) -> Result<Emitted, String> {
    let mut integrations = env.integrations()?;
    let integration = pick(&integrations, id)?.clone();
    let client = gitpoulpi_hosting::client_for(
        integration.clone(),
        token.to_string(),
        env.transport()?,
        &env.gate,
    )
    .map_err(|e| e.to_string())?;
    let check = client.check_token().await.map_err(|e| e.to_string())?;
    // The account of a token that was just typed must be the one trusted
    // (or the first one this integration is trusted for).
    env.gate
        .check_account(
            &integration,
            &gitpoulpi_hosting::Account {
                login: check.user.login.clone(),
                id: check.user.id.clone(),
            },
        )
        .map_err(|e| e.to_string())?;

    env.store
        .store(&integration.token_ref, token)
        .map_err(|e| format!("the token is valid but could not be stored: {e}"))?;
    if let Some(entry) = integrations.iter_mut().find(|i| i.id == integration.id) {
        entry.user = Some(ConnectedUser {
            login: check.user.login.clone(),
            name: check.user.name.clone(),
            id: check.user.id.clone(),
        });
    }
    env.save_integrations(&integrations)?;

    let human = format!(
        "`{}` connected as @{}{}",
        integration.id,
        check.user.login,
        if check.missing_scopes.is_empty() {
            String::new()
        } else {
            format!(" - token lacks: {}", check.missing_scopes.join(", "))
        }
    );
    Ok(Emitted {
        exit_code: 0,
        json: json!({
            "id": integration.id,
            "user": check.user,
            "scopes": check.scopes,
            "missing_scopes": check.missing_scopes,
            "expires_at": check.expires_at,
            "token_stored": true,
        }),
        human,
    })
}

pub(super) fn logout(env: &HostingEnv, id: Option<&str>) -> Result<Emitted, String> {
    let mut integrations = env.integrations()?;
    let integration = pick(&integrations, id)?.clone();
    env.store
        .delete(&integration.token_ref)
        .map_err(|e| format!("the stored token could not be deleted: {e}"))?;
    if let Some(entry) = integrations.iter_mut().find(|i| i.id == integration.id) {
        entry.user = None;
    }
    env.save_integrations(&integrations)?;
    // Disconnecting forgets what was trusted for it, and what was cached.
    gitpoulpi_hosting::trust::forget(&env.paths, &integration);
    Ok(Emitted {
        exit_code: 0,
        json: json!({ "id": integration.id, "token_deleted": true }),
        human: format!("`{}` disconnected; the profile is kept", integration.id),
    })
}

/// What `integrations trust` was asked to accept.
pub(super) struct TrustOptions {
    pub accept_origin: Option<String>,
    pub accept_account: Option<String>,
    pub accept_certificate: Option<String>,
    pub strict_tls: Option<bool>,
    pub allow_insecure_http: Option<bool>,
}

fn tls_json(tls: &Option<gitpoulpi_hosting::trust::TlsRecord>) -> Value {
    match tls {
        None => Value::Null,
        Some(t) => json!({
            "mode": t.mode.as_str(),
            "spki_sha256": t.cert.spki_sha256,
            "subject": t.cert.subject,
            "issuer": t.cert.issuer,
            "not_before": t.cert.not_before,
            "not_after": t.cert.not_after,
        }),
    }
}

/// The key fingerprint of the certificate `integration`'s host presents
/// right now: one unauthenticated request, over the plain transport (so the
/// verifier records what it saw and no token is involved).
async fn observe_certificate(
    env: &HostingEnv,
    integration: &Integration,
) -> Result<gitpoulpi_hosting::trust::CertInfo, String> {
    use gitpoulpi_hosting::trust::{origin, CertInfo};
    use gitpoulpi_hosting::{Error, HttpRequest, TrustProblem};
    let url = origin::integration_origin(integration).map_err(|e| e.to_string())?;
    let found: Option<CertInfo> = match env.transport()?.send(HttpRequest::get(&url)).await {
        Err(Error::Trust(TrustProblem::UntrustedCertificate { new, .. }))
        | Err(Error::Trust(TrustProblem::CertificateChanged { new, .. })) => Some(new),
        _ => origin::host_of(&url).and_then(|h| gitpoulpi_hosting::trust::tls::observed(&h)),
    };
    found.ok_or_else(|| {
        format!("no certificate could be read from {url} (is the server reachable over https?)")
    })
}

/// `integrations trust`: accept, one explicit value at a time, what the
/// security check stopped. Each `--accept-*` names the exact value to accept,
/// so a change that happened since the refusal was read is not accepted by
/// accident.
pub(super) async fn trust_integration(
    env: &HostingEnv,
    id: &str,
    opts: TrustOptions,
) -> Result<Emitted, String> {
    use gitpoulpi_hosting::trust::{self, origin, TlsMode};
    let integrations = env.integrations()?;
    let integration = pick(&integrations, Some(id))?.clone();
    if integration.kind.capabilities().local_folder {
        return Err(format!(
            "`{id}` is a folder of notes: there is no address to trust"
        ));
    }
    let gate = &env.gate;
    let err = |e: gitpoulpi_hosting::Error| e.to_string();
    let current = trust::integration_origin(&integration).map_err(err)?;
    let mut accepted: Vec<String> = Vec::new();

    if let Some(url) = &opts.accept_origin {
        let given = trust::normalize(url).ok_or_else(|| format!("`{url}` is not an address"))?;
        if given != current {
            return Err(format!(
                "`{id}` points to {current} now, not to {given}: nothing was accepted"
            ));
        }
        gate.trust_origin(&integration).map_err(err)?;
        accepted.push(format!("address {current}"));
    }
    if let Some(on) = opts.allow_insecure_http {
        gate.set_allow_plain_http(&integration, on).map_err(err)?;
        accepted.push(format!(
            "plain http {}",
            if on { "allowed" } else { "refused" }
        ));
    }
    if let Some(account) = &opts.accept_account {
        gate.trust_account(
            &integration,
            &gitpoulpi_hosting::Account {
                login: String::new(),
                id: account.trim().to_string(),
            },
        )
        .map_err(err)?;
        accepted.push(format!("account {}", account.trim()));
    }
    let mode = match opts.strict_tls {
        Some(true) => TlsMode::Strict,
        _ => gate
            .record_of(id)
            .and_then(|r| r.tls.map(|t| t.mode))
            .unwrap_or(TlsMode::Tofu),
    };
    if let Some(fingerprint) = &opts.accept_certificate {
        let fp = fingerprint.trim().to_ascii_lowercase().replace(' ', "");
        if fp.len() != 64 || !fp.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(
                "--accept-certificate takes the SHA-256 fingerprint (64 hex digits)".into(),
            );
        }
        let cert = gitpoulpi_hosting::trust::CertInfo {
            spki_sha256: fp.clone(),
            ..Default::default()
        };
        gate.trust_certificate(&integration, &cert, mode)
            .map_err(err)?;
        accepted.push(format!("certificate {fp} ({})", mode.as_str()));
    } else if let Some(strict) = opts.strict_tls {
        let has_pin = gate.record_of(id).is_some_and(|r| r.tls.is_some());
        if has_pin {
            gate.set_strict(&integration, strict).map_err(err)?;
        } else if strict {
            // Nothing pinned yet: pin what the server presents now.
            let cert = observe_certificate(env, &integration).await?;
            gate.trust_certificate(&integration, &cert, TlsMode::Strict)
                .map_err(err)?;
        }
        accepted.push(format!(
            "strict pinning {}",
            if strict { "on" } else { "off" }
        ));
    }

    let record = gate.record_of(id);
    let pending: Vec<Value> = trust::peek_pending()
        .iter()
        .filter(|p| p.subject().name() == id)
        .map(|p| {
            json!({ "change": p.change(), "old": p.old_text(), "new": p.new_text(), "fix": p.fix() })
        })
        .collect();
    let human = if accepted.is_empty() {
        match &record {
            Some(r) => format!(
                "`{id}` is trusted for {} (account {}{}){}",
                r.origin,
                if r.account_login.is_empty() {
                    r.account_id.clone()
                } else {
                    format!("{} #{}", r.account_login, r.account_id)
                },
                if r.allow_plain_http {
                    ", plain http allowed"
                } else {
                    ""
                },
                match &r.tls {
                    Some(t) => format!("; certificate {} ({})", t.cert.grouped(), t.mode.as_str()),
                    None => String::new(),
                }
            ),
            None => format!("`{id}` has no trust record yet; it gets one on its first connection"),
        }
    } else {
        format!("`{id}`: accepted {}", accepted.join(", "))
    };
    Ok(Emitted {
        exit_code: 0,
        json: json!({
            "id": id,
            "accepted": accepted,
            "current_origin": current,
            "trusted": record.as_ref().map(|r| json!({
                "origin": r.origin,
                "account_id": r.account_id,
                "account_login": r.account_login,
                "allow_plain_http": r.allow_plain_http,
                "tls": tls_json(&r.tls),
            })),
            "pending": pending,
            "plain_http_remote": origin::is_plain_http_remote(&current),
        }),
        human,
    })
}
