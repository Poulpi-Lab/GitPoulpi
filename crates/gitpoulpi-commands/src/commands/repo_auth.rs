//! `gitpoulpi repo-auth show|set|clear|test`: which identity a repository
//! authenticates with (the per-repository authentication design). The
//! profile lives in GitPoulpi's private directory; the daemon owns it, and
//! these commands are its CLI and MCP face.
//!
//! Security shape: nothing printed here is a secret; a command-running
//! source's command is printed only to a session that `may_trust()`, and only
//! such a session may set one (after it has shown the command and its hash).
//! An agent (the MCP server) cannot set or clear a profile at all: the
//! registry excludes those two commands.

use std::io::{BufRead, IsTerminal, Write};

use gitpoulpi_core::domain::{
    HttpsAuth, OperationKind, RemoteAccessReport, RemoteIdentity, RepoAuthProfile, RepoAuthSetArgs,
    RepoAuthTestArgs, RepoAuthView, SshAuth,
};
use gitpoulpi_core::TypedError;
use serde_json::{json, Value};

use crate::cli::RepoAuthAction;
use crate::client::Session;

use super::*;

/// A refusal the caller can tell from a failure (exit code 2): `reason` is a
/// stable word (`needs_user`, `unavailable`).
fn refused(reason: &str, message: impl Into<String>, hint: Option<&str>) -> Emitted {
    let message = message.into();
    let mut body = json!({ "reason": reason, "message": message });
    if let Some(hint) = hint {
        body["hint"] = json!(hint);
    }
    let mut out = emitted(json!({ "refused": body }), format!("refused: {message}"));
    out.exit_code = 2;
    out
}

pub(super) async fn repo_auth(
    session: &mut Session,
    action: RepoAuthAction,
) -> Result<Emitted, String> {
    match action {
        RepoAuthAction::Show => show(session).await,
        RepoAuthAction::Set {
            https,
            username,
            ignore_repo_helpers,
            helper,
            command,
            hosts,
            cache_minutes,
            ssh,
            known_hosts,
            strict,
            yes,
        } => {
            let flags = SetFlags {
                username,
                ignore_repo_helpers,
                helper,
                command,
                hosts,
                cache_minutes,
                known_hosts,
                strict,
            };
            set(session, https, ssh, flags, yes).await
        }
        RepoAuthAction::Clear => clear(session).await,
        RepoAuthAction::Test { remote, push } => test(session, remote, push).await,
    }
}

async fn view(session: &mut Session) -> Result<RepoAuthView, String> {
    session.call(OperationKind::RepoAuthGet, json!({})).await
}

/// The profile as stored, commands included: read from the private file by
/// this process (the daemon's view blanks them). Never printed to a session
/// that may not trust.
fn stored_profile(session: &Session) -> Option<RepoAuthProfile> {
    let root = session.repo_root()?;
    let paths = gitpoulpi_config::ConfigPaths::discover()?;
    gitpoulpi_session::load_repo_auth(&paths, root)
        .ok()
        .flatten()
        .map(|r| r.profile())
}

/// `{set, sha256}` for a command, plus the command itself for a person.
fn command_json(https: &HttpsAuth, reveal: bool, stored: Option<&HttpsAuth>) -> Value {
    // The view's own command is blanked: only the stored one can be hashed.
    let sha = match https {
        HttpsAuth::Helper {
            confirmed_sha256, ..
        }
        | HttpsAuth::TokenCommand {
            confirmed_sha256, ..
        } => confirmed_sha256.clone(),
        _ => None,
    }
    .or_else(|| stored.and_then(HttpsAuth::command_sha256));
    let mut out = json!({ "set": true, "sha256": sha });
    if let (true, Some(source)) = (reveal, stored) {
        match source {
            HttpsAuth::Helper { helper, .. } => out["helper"] = json!(helper),
            HttpsAuth::TokenCommand { argv, .. } => out["argv"] = json!(argv),
            _ => {}
        }
    }
    out
}

/// The view as JSON with the command of a command-running source replaced by
/// `command` (never `helper` / `argv` unless the session may trust).
fn view_json(session: &Session, view: &RepoAuthView) -> Result<Value, String> {
    let reveal = session.may_trust();
    let stored = if reveal {
        stored_profile(session)
    } else {
        None
    };
    view_json_with(view, reveal, stored.as_ref())
}

/// [`view_json`] for a given trust: `stored` is the profile with its commands,
/// read only when `reveal`.
fn view_json_with(
    view: &RepoAuthView,
    reveal: bool,
    stored: Option<&RepoAuthProfile>,
) -> Result<Value, String> {
    let mut value = to_value(view)?;
    if let Some(https) = &view.profile.https {
        if https.command_sha256().is_some() {
            let command = command_json(https, reveal, stored.and_then(|p| p.https.as_ref()));
            if let Some(map) = value
                .pointer_mut("/profile/https")
                .and_then(Value::as_object_mut)
            {
                map.remove("helper");
                map.remove("argv");
                map.insert("command".into(), command);
            }
        }
    }
    Ok(value)
}

async fn show(session: &mut Session) -> Result<Emitted, String> {
    let view = view(session).await?;
    let json = view_json(session, &view)?;
    let mut human = Vec::new();
    match &view.summary {
        Some(summary) => human.push(format!("profile: {summary}")),
        None => human.push("profile: none (the machine's own identity)".to_string()),
    }
    for r in &view.remotes {
        human.push(format!("  {}: {} ({})", r.remote, r.identity, r.url));
    }
    for o in &view.overrides {
        let note = if o.overridden {
            "overridden"
        } else {
            "competes"
        };
        human.push(format!("  {} = {} [{}, {note}]", o.key, o.value, o.scope));
    }
    for w in &view.warnings {
        human.push(format!("warning: {w}"));
    }
    Ok(emitted(json, human.join("\n")))
}

struct SetFlags {
    username: Option<String>,
    ignore_repo_helpers: bool,
    helper: Option<String>,
    command: Vec<String>,
    hosts: Vec<String>,
    cache_minutes: Option<u32>,
    known_hosts: Option<String>,
    strict: bool,
}

fn goes_with(used: bool, flag: &str, with: &str) -> Result<(), String> {
    if used {
        Err(format!("{flag} goes with {with}"))
    } else {
        Ok(())
    }
}

fn parse_https(source: &str, f: &SetFlags) -> Result<HttpsAuth, String> {
    let username = f.username.clone().filter(|u| !u.is_empty());
    let not_helper = |f: &SetFlags| goes_with(f.helper.is_some(), "--helper", "--https helper");
    let not_command =
        |f: &SetFlags| goes_with(!f.command.is_empty(), "--command", "--https token-command");
    let not_hosts = |f: &SetFlags| {
        goes_with(
            !f.hosts.is_empty(),
            "--hosts",
            "--https helper or token-command",
        )
    };
    let not_cache = |f: &SetFlags| {
        goes_with(
            f.cache_minutes.is_some(),
            "--cache-minutes",
            "--https token-command",
        )
    };
    let not_ignore = |f: &SetFlags| {
        goes_with(
            f.ignore_repo_helpers,
            "--ignore-repo-helpers",
            "--https system",
        )
    };
    match source {
        "system" => {
            not_helper(f)?;
            not_command(f)?;
            not_hosts(f)?;
            not_cache(f)?;
            Ok(HttpsAuth::System {
                username,
                ignore_repo_helpers: f.ignore_repo_helpers,
            })
        }
        "helper" => {
            not_ignore(f)?;
            not_command(f)?;
            not_cache(f)?;
            let helper = f
                .helper
                .clone()
                .filter(|h| !h.trim().is_empty())
                .ok_or("--https helper needs --helper CMD")?;
            Ok(HttpsAuth::Helper {
                helper,
                username,
                hosts: f.hosts.clone(),
                confirmed_sha256: None,
                script_sha256: None,
                confirmed_at: None,
            })
        }
        "token-command" => {
            not_ignore(f)?;
            not_helper(f)?;
            if f.command.is_empty() || f.command[0].trim().is_empty() {
                return Err("--https token-command needs --command ARG...".into());
            }
            if f.hosts.is_empty() {
                return Err("--https token-command needs --hosts HOST".into());
            }
            Ok(HttpsAuth::TokenCommand {
                argv: f.command.clone(),
                username,
                hosts: f.hosts.clone(),
                cache_minutes: f.cache_minutes,
                confirmed_sha256: None,
                script_sha256: None,
                confirmed_at: None,
            })
        }
        other => match other.strip_prefix("integration:") {
            Some(id) if !id.is_empty() => {
                not_ignore(f)?;
                not_helper(f)?;
                not_command(f)?;
                not_hosts(f)?;
                not_cache(f)?;
                Ok(HttpsAuth::Integration {
                    integration: id.to_string(),
                    username,
                })
            }
            _ => Err(format!(
                "unknown --https source {other:?}: use system, integration:<id>, helper or \
                 token-command"
            )),
        },
    }
}

fn parse_ssh(source: &str, f: &SetFlags) -> Result<SshAuth, String> {
    let known_hosts = f.known_hosts.clone().filter(|k| !k.is_empty());
    match source {
        "system" => {
            if known_hosts.is_some() || f.strict {
                return Err(
                    "--known-hosts and --strict go with --ssh key: or integration-key:".into(),
                );
            }
            Ok(SshAuth::System)
        }
        other => {
            if let Some(key) = other.strip_prefix("key:").filter(|k| !k.is_empty()) {
                return Ok(SshAuth::KeyFile {
                    key: key.to_string(),
                    known_hosts,
                    strict_host_key: f.strict,
                });
            }
            if let Some(id) = other
                .strip_prefix("integration-key:")
                .filter(|k| !k.is_empty())
            {
                return Ok(SshAuth::IntegrationKey {
                    integration: id.to_string(),
                    known_hosts,
                    strict_host_key: f.strict,
                });
            }
            Err(format!(
                "unknown --ssh source {other:?}: use system, key:<path> or integration-key:<id>"
            ))
        }
    }
}

/// Shows the command and its hash and asks a person to confirm. `Ok(false)`
/// when they decline.
fn confirm_command(https: &HttpsAuth, yes: bool) -> Result<bool, String> {
    let (what, text) = match https {
        HttpsAuth::Helper { helper, .. } => ("credential helper", helper.clone()),
        HttpsAuth::TokenCommand { argv, .. } => ("token command", argv.join(" ")),
        _ => return Ok(true),
    };
    let sha = https.command_sha256().unwrap_or_default();
    let mut err = std::io::stderr();
    let _ = writeln!(
        err,
        "This repository would run this {what} to sign in:\n  {text}\n  sha256: {sha}\n\
         It runs with your privileges every time this repository talks to its remote."
    );
    if yes {
        return Ok(true);
    }
    if !std::io::stdin().is_terminal() {
        return Err(
            "a command needs your confirmation: run it in a terminal, or pass --yes".into(),
        );
    }
    let _ = write!(err, "Run it for this repository? [y/N] ");
    let _ = err.flush();
    let mut line = String::new();
    std::io::stdin()
        .lock()
        .read_line(&mut line)
        .map_err(|e| format!("could not read the answer: {e}"))?;
    Ok(matches!(
        line.trim().to_ascii_lowercase().as_str(),
        "y" | "yes"
    ))
}

async fn set(
    session: &mut Session,
    https: Option<String>,
    ssh: Option<String>,
    flags: SetFlags,
    yes: bool,
) -> Result<Emitted, String> {
    if https.is_none() && ssh.is_none() {
        return Err("pass --https SOURCE and/or --ssh SOURCE".into());
    }
    if https.is_none()
        && (flags.username.is_some()
            || flags.ignore_repo_helpers
            || flags.helper.is_some()
            || !flags.command.is_empty()
            || !flags.hosts.is_empty()
            || flags.cache_minutes.is_some())
    {
        return Err("those options go with --https SOURCE".into());
    }
    if ssh.is_none() && (flags.known_hosts.is_some() || flags.strict) {
        return Err("--known-hosts and --strict go with --ssh SOURCE".into());
    }
    // Parse first: a typo is a failure whoever asks.
    let new_https = https
        .as_deref()
        .map(|s| parse_https(s, &flags))
        .transpose()?;
    let new_ssh = ssh.as_deref().map(|s| parse_ssh(s, &flags)).transpose()?;
    // Only a person chooses an identity: an agent is refused before anything
    // else, whatever it asks for.
    if !session.may_trust() {
        return Ok(refused(
            "needs_user",
            "which identity a repository signs in with is the user's decision: run \
             `gitpoulpi repo-auth set` yourself",
            Some("ask the user to run it"),
        ));
    }
    // A protocol that is not named keeps what the repository has.
    let mut profile = stored_profile(session).unwrap_or_default();
    if let Some(h) = new_https {
        if matches!(h, HttpsAuth::Helper { .. } | HttpsAuth::TokenCommand { .. })
            && !confirm_command(&h, yes)?
        {
            return Err("not confirmed: nothing changed".into());
        }
        profile.https = Some(h);
    }
    if let Some(s) = new_ssh {
        profile.ssh = Some(s);
    }
    let args = RepoAuthSetArgs {
        profile,
        confirm: true,
    };
    match session
        .call_typed::<Value>(OperationKind::RepoAuthSet, to_value(&args)?)
        .await?
    {
        Ok(_) => {
            let after = view(session).await?;
            let json = view_json(session, &after)?;
            let human = match &after.summary {
                Some(s) => format!("this repository now signs in as: {s}"),
                None => "this repository uses the machine's own identity".to_string(),
            };
            Ok(emitted(json!({ "set": true, "view": json }), human))
        }
        Err(e) => match e.kind {
            Some(TypedError::RepoAuthUnavailable { reason }) => {
                Ok(refused("unavailable", reason, None))
            }
            _ => Err(e.message),
        },
    }
}

async fn clear(session: &mut Session) -> Result<Emitted, String> {
    if !session.may_trust() {
        return Ok(refused(
            "needs_user",
            "which identity a repository signs in with is the user's decision: run \
             `gitpoulpi repo-auth clear` yourself",
            Some("ask the user to run it"),
        ));
    }
    let had: bool = session
        .call(OperationKind::RepoAuthClear, json!({}))
        .await?;
    let human = if had {
        "cleared: this repository uses the machine's own identity"
    } else {
        "nothing to clear"
    };
    Ok(emitted(json!({ "cleared": had }), human))
}

async fn test(
    session: &mut Session,
    remote: Option<String>,
    push: bool,
) -> Result<Emitted, String> {
    let identities: Vec<RemoteIdentity> = view(session).await?.remotes;
    let args = to_value(&RepoAuthTestArgs { remote, push })?;
    let reports: Vec<RemoteAccessReport> = match session
        .call_typed(OperationKind::RepoAuthTest, args)
        .await?
    {
        Ok(reports) => reports,
        Err(e) => {
            return match e.kind {
                Some(TypedError::RepoAuthUnavailable { reason }) => {
                    Ok(refused("unavailable", reason, None))
                }
                _ => Err(e.message),
            }
        }
    };
    let mut human = Vec::new();
    for r in &reports {
        let who = identities
            .iter()
            .find(|i| i.remote == r.remote)
            .map(|i| i.identity.as_str())
            .unwrap_or("?");
        let verdict = if r.ok {
            format!("ok, {} branches, {} ms", r.ref_count, r.duration_ms)
        } else {
            let why = r
                .error
                .as_ref()
                .map(|e| format!("({:?})", e.kind()))
                .unwrap_or_default();
            format!("failed {why}")
        };
        human.push(format!("{} as {who}: {verdict}", r.remote));
    }
    let ok = reports.iter().all(|r| r.ok);
    Ok(emitted(
        json!({ "ok": ok, "reports": reports, "identity": identities }),
        human.join("\n"),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn token_command() -> HttpsAuth {
        HttpsAuth::TokenCommand {
            argv: vec!["pass".into(), "show".into(), "work-token".into()],
            username: None,
            hosts: vec!["github.com".into()],
            cache_minutes: None,
            confirmed_sha256: None,
            script_sha256: None,
            confirmed_at: None,
        }
    }

    /// What the daemon sends: the command blanked, the confirmed hash kept.
    fn view_of(https: HttpsAuth) -> RepoAuthView {
        let sha = https.command_sha256();
        let mut blanked = https;
        if let HttpsAuth::TokenCommand {
            argv,
            confirmed_sha256,
            ..
        } = &mut blanked
        {
            argv.clear();
            *confirmed_sha256 = sha;
        }
        RepoAuthView {
            configured: true,
            profile: RepoAuthProfile {
                https: Some(blanked),
                ssh: None,
            },
            ..Default::default()
        }
    }

    #[test]
    fn a_command_is_shown_only_to_a_session_that_may_trust() {
        let full = token_command();
        let view = view_of(full.clone());
        let stored = RepoAuthProfile {
            https: Some(full.clone()),
            ssh: None,
        };

        let agent = view_json_with(&view, false, None).unwrap();
        let command = &agent["profile"]["https"]["command"];
        assert_eq!(command["set"], true);
        assert_eq!(command["sha256"], json!(full.command_sha256()));
        assert!(command.get("argv").is_none() && command.get("helper").is_none());
        assert!(!agent.to_string().contains("work-token"));

        // Even with the stored profile at hand, an agent's view stays blind.
        let still_agent = view_json_with(&view, false, Some(&stored)).unwrap();
        assert!(!still_agent.to_string().contains("work-token"));

        let person = view_json_with(&view, true, Some(&stored)).unwrap();
        assert_eq!(
            person["profile"]["https"]["command"]["argv"],
            json!(["pass", "show", "work-token"])
        );
        assert!(person["profile"]["https"].get("argv").is_none());
    }

    #[test]
    fn a_refusal_is_exit_code_2_with_a_stable_reason() {
        let out = refused("needs_user", "ask the user", Some("hint"));
        assert_eq!(out.exit_code, 2);
        assert_eq!(out.json["refused"]["reason"], "needs_user");
        assert_eq!(out.json["refused"]["message"], "ask the user");
        assert_eq!(out.json["refused"]["hint"], "hint");
    }

    #[test]
    fn the_hash_follows_the_command() {
        let a = token_command();
        let mut b = token_command();
        if let HttpsAuth::TokenCommand { argv, .. } = &mut b {
            argv[2] = "other".into();
        }
        assert_ne!(a.command_sha256(), b.command_sha256());
        assert_eq!(a.command_sha256().unwrap().len(), 64);
    }
}
