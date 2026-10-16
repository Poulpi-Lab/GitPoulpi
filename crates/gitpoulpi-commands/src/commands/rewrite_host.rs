//! S23 in the CLI/MCP: what the host protects, and the three ways through
//! when it blocks a rewrite. English text and JSON only; the TUI words it in
//! its own language from the same `Howto`.

use gitpoulpi_core::domain::rewrite::{
    HostProtection, PushRefStatus, PushRewrittenOutcome, Recipe, RewriteNote, RewritePlan,
    RewriteWarning,
};
use serde_json::{json, Value};

use crate::client::Session;

/// The repository's host, asked (`ask`) for what it protects when the
/// repository is linked. `None` without the hosting providers or when no host
/// can be named.
#[cfg(feature = "hosting")]
pub(super) async fn host_for(
    session: &mut Session,
    hint: Option<&str>,
    ask: bool,
) -> Option<HostProtection> {
    use gitpoulpi_core::domain::{OperationKind, Remote};
    let root = session.repo_root()?.to_path_buf();
    let remotes: Vec<Remote> = session
        .call(OperationKind::Remotes, serde_json::json!({}))
        .await
        .ok()?;
    let state =
        gitpoulpi_session::RepoState::load(&super::repo_state_path(&root)).unwrap_or_default();
    let env = crate::hosting_cmd::HostingEnv::ambient().ok()?;
    env.rewrite_host_protection(&remotes, &state, hint, ask)
        .await
}

#[cfg(not(feature = "hosting"))]
pub(super) async fn host_for(
    _session: &mut Session,
    _hint: Option<&str>,
    _ask: bool,
) -> Option<HostProtection> {
    None
}

/// The recipe with what the host protects attached, for a linked repository.
pub(super) async fn with_host(session: &mut Session, recipe: &Recipe) -> Recipe {
    let mut recipe = recipe.clone();
    if let Some(host) = host_for(session, None, true).await {
        if host.asked {
            recipe.scope_options.host_protection = Some(host);
        }
    }
    recipe
}

/// `protected_by_host`: the how-to as JSON, with the English text.
pub(super) fn blocked_json(host: &HostProtection, branches: &[String]) -> Value {
    let howto = host.howto(branches);
    let ways: Vec<Value> = howto
        .ways
        .iter()
        .map(|w| {
            json!({
                "way": w.way.code(),
                "title": w.way.title(),
                "steps": w.steps.iter().map(|s| s.text()).collect::<Vec<_>>(),
                "links": w.links,
            })
        })
        .collect();
    json!({
        "provider": host.provider(),
        "project": host.project,
        "branches": branches,
        "protection_known": host.asked && host.unknown.is_none(),
        "protection_unknown": host.unknown,
        "ways": ways,
        "recreate_loses": howto.losses.iter().map(|l| l.text()).collect::<Vec<_>>(),
        "text": howto.text(),
    })
}

/// The branches (short names) of a preview the host protects and the run
/// left out or still moves.
pub(super) fn plan_blocked(host: &HostProtection, plan: &RewritePlan) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    let mut add = |full: &str| {
        let short = full.strip_prefix("refs/heads/").unwrap_or(full);
        if host.covers(short) && !names.iter().any(|n| n == short) {
            names.push(short.to_string());
        }
    };
    for change in plan.refs.iter().filter(|r| r.protected) {
        add(&change.name);
    }
    for warning in &plan.warnings {
        if let RewriteWarning::ProtectedExcluded { refs } = warning {
            refs.iter().for_each(|r| add(r));
        }
    }
    names
}

/// The branches a push outcome says the host refused.
pub(super) fn push_blocked(outcome: &PushRewrittenOutcome) -> Vec<(String, String)> {
    outcome
        .results
        .iter()
        .filter(|r| r.status == PushRefStatus::RejectedProtectedByHost)
        .map(|r| {
            (
                r.name
                    .strip_prefix("refs/heads/")
                    .unwrap_or(&r.name)
                    .to_string(),
                r.remote.clone(),
            )
        })
        .collect()
}

/// The branches of a note that the host protects.
pub(super) fn note_blocked(host: &HostProtection, note: &RewriteNote) -> Vec<String> {
    note.refs
        .iter()
        .filter(|r| host.covers(&r.short))
        .map(|r| r.short.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitpoulpi_core::domain::rewrite::{HostKind, PushRefResult, RunId};
    use gitpoulpi_core::domain::Oid;

    fn outcome() -> PushRewrittenOutcome {
        let result = |name: &str, status| PushRefResult {
            name: name.into(),
            remote: "origin".into(),
            expected_old: Oid([1; 20]),
            new: Some(Oid([2; 20])),
            status,
            detail: None,
        };
        PushRewrittenOutcome {
            run: RunId("r".into()),
            dry_run: false,
            results: vec![
                result("refs/heads/main", PushRefStatus::RejectedProtectedByHost),
                result("refs/heads/topic", PushRefStatus::Pushed),
            ],
            atomic_fallback: false,
        }
    }

    #[test]
    fn a_host_rejection_becomes_the_steps_and_the_exact_link() {
        let blocked = push_blocked(&outcome());
        assert_eq!(blocked, [("main".to_string(), "origin".to_string())]);
        let host = HostProtection::new(HostKind::GitLab, "g/sub/p", "https://git.corp.example");
        let json = blocked_json(&host, &["main".to_string()]);
        assert_eq!(json["provider"], "GitLab");
        assert_eq!(json["ways"].as_array().unwrap().len(), 3);
        assert_eq!(
            json["ways"][0]["links"][0],
            "https://git.corp.example/g/sub/p/-/settings/repository#js-protected-branches-settings"
        );
        assert!(json["text"]
            .as_str()
            .unwrap()
            .contains("Protected branches"));
        assert_eq!(json["protection_known"], false);
    }
}
