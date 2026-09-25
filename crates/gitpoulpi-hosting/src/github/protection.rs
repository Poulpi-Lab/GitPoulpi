//! GitHub's branch protection, for the history rewrite (S23).
//!
//! Two sources, both read with the user's token:
//!
//! - the classic rules: `GET /repos/{p}/branches?protected=true` (the
//!   `protected` flag; a classic token with `repo`, a fine-grained one with
//!   Metadata read);
//! - the rulesets: `GET /repos/{p}/rulesets` for the active branch rulesets,
//!   then `GET /repos/{p}/rulesets/{id}` for their `ref_name` conditions
//!   (`~DEFAULT_BRANCH`, `~ALL`, `refs/heads/x*`). Excludes are ignored: a
//!   branch listed too many is only left out by default, never rewritten.
//!
//! One source failing leaves the other's answer with a note; both failing is
//! an error the caller shows as "unknown".

use crate::client::fetch_pages;
use crate::github::GitHubClient;
use crate::protection::ProtectedBranches;
use crate::{Error, Result};

/// Rulesets read in full per lookup; a repository with more is not a
/// repository to read in a hurry.
const MAX_RULESETS: usize = 20;

#[derive(serde::Deserialize)]
struct Branch {
    name: String,
}

#[derive(serde::Deserialize)]
struct RulesetRow {
    id: u64,
    #[serde(default)]
    target: String,
    #[serde(default)]
    enforcement: String,
}

#[derive(serde::Deserialize, Default)]
struct Ruleset {
    #[serde(default)]
    conditions: Conditions,
}

#[derive(serde::Deserialize, Default)]
struct Conditions {
    #[serde(default)]
    ref_name: RefName,
}

#[derive(serde::Deserialize, Default)]
struct RefName {
    #[serde(default)]
    include: Vec<String>,
}

#[derive(serde::Deserialize)]
struct Repo {
    #[serde(default)]
    default_branch: Option<String>,
}

impl GitHubClient {
    pub(super) async fn protection(&self, project: &str) -> Result<ProtectedBranches> {
        let mut out = ProtectedBranches::default();
        let classic = self.classic_protection(project, &mut out).await;
        let rulesets = self.ruleset_protection(project, &mut out).await;
        match (classic, rulesets) {
            (Ok(()), Ok(())) => {}
            (Err(e), Err(_)) => return Err(e),
            (Err(_), Ok(())) => out.note("the branch protection rules could not be read"),
            (Ok(()), Err(_)) => out.note("the rulesets could not be read"),
        }
        Ok(out)
    }

    async fn classic_protection(&self, project: &str, out: &mut ProtectedBranches) -> Result<()> {
        let url = format!(
            "{}/repos/{project}/branches?protected=true",
            self.integration.api_base()
        );
        let rows: Vec<Branch> = fetch_pages(
            &*self.transport,
            |u| self.get(u),
            &url,
            "reading GitHub protected branches",
        )
        .await?;
        for row in rows {
            out.add(&row.name);
        }
        Ok(())
    }

    async fn ruleset_protection(&self, project: &str, out: &mut ProtectedBranches) -> Result<()> {
        let base = self.integration.api_base();
        let url = format!("{base}/repos/{project}/rulesets?includes_parents=true");
        let rows: Vec<RulesetRow> = fetch_pages(
            &*self.transport,
            |u| self.get(u),
            &url,
            "listing GitHub rulesets",
        )
        .await?;
        let mut default_branch: Option<String> = None;
        for row in rows
            .into_iter()
            .filter(|r| r.target == "branch" && r.enforcement == "active")
            .take(MAX_RULESETS)
        {
            let response = self
                .transport
                .send(self.get(format!("{base}/repos/{project}/rulesets/{}", row.id)))
                .await?;
            response.error_for_status("reading a GitHub ruleset")?;
            let ruleset: Ruleset = response.json()?;
            for include in ruleset.conditions.ref_name.include {
                match include.as_str() {
                    "~ALL" => out.add("*"),
                    "~DEFAULT_BRANCH" => {
                        if default_branch.is_none() {
                            default_branch = Some(self.default_branch(project).await?);
                        }
                        if let Some(name) = default_branch.as_deref().filter(|n| !n.is_empty()) {
                            out.add(name);
                        }
                    }
                    other => out.add(other),
                }
            }
        }
        Ok(())
    }

    async fn default_branch(&self, project: &str) -> Result<String> {
        let url = format!("{}/repos/{project}", self.integration.api_base());
        let response = self.transport.send(self.get(url)).await?;
        response.error_for_status("reading the GitHub repository")?;
        let repo: Repo = response.json()?;
        repo.default_branch
            .ok_or_else(|| Error::Decode("the repository names no default branch".into()))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use gitpoulpi_config::IntegrationKind;

    use crate::client::client_for;
    use crate::http::RecordedTransport;
    use crate::integration::Integration;
    use crate::protection::{lookup, ProtectionLookup};

    fn block_on<F: std::future::Future>(f: F) -> F::Output {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(f)
    }

    fn client(transport: Arc<RecordedTransport>) -> Box<dyn crate::HostingClient> {
        client_for(
            Integration::new("gh", IntegrationKind::Github, "https://github.com"),
            "tok".into(),
            transport,
        )
    }

    const BRANCHES: &str =
        r#"[{"name":"main","protected":true},{"name":"release/1.x","protected":true}]"#;
    const RULESETS: &str = r#"[
        {"id":7,"name":"trunk","target":"branch","enforcement":"active"},
        {"id":8,"name":"tags","target":"tag","enforcement":"active"},
        {"id":9,"name":"draft","target":"branch","enforcement":"disabled"}]"#;
    const RULESET_7: &str = r#"{"id":7,"conditions":{"ref_name":{"include":["~DEFAULT_BRANCH","refs/heads/hotfix/*"],"exclude":[]}},
        "rules":[{"type":"non_fast_forward"}]}"#;

    #[test]
    fn classic_rules_and_active_branch_rulesets_are_merged_by_name() {
        let transport = Arc::new(
            RecordedTransport::new()
                .on("/branches?protected=true", 200, BRANCHES)
                .on("/rulesets/7", 200, RULESET_7)
                .on("/rulesets?", 200, RULESETS)
                .on(
                    "api.github.com/repos/octo/thing",
                    200,
                    r#"{"default_branch":"trunk"}"#,
                ),
        );
        let found = block_on(lookup(&*client(transport.clone()), "octo/thing"));
        let ProtectionLookup::Known(found) = found else {
            panic!("{found:?}")
        };
        assert_eq!(
            found.patterns,
            ["main", "release/1.x", "trunk", "hotfix/*"],
            "the tag ruleset and the disabled one add nothing"
        );
        assert!(found.note.is_none());
        let urls: Vec<String> = transport.requests().into_iter().map(|r| r.url).collect();
        assert!(
            urls.iter()
                .all(|u| u.starts_with("https://api.github.com/")),
            "{urls:?}"
        );
        assert!(!urls
            .iter()
            .any(|u| u.contains("/rulesets/8") || u.contains("/rulesets/9")));
    }

    #[test]
    fn a_token_without_the_scope_is_unknown_not_an_error() {
        let transport = Arc::new(
            RecordedTransport::new()
                .on(
                    "/branches?protected=true",
                    403,
                    r#"{"message":"Resource not accessible by personal access token"}"#,
                )
                .on(
                    "/rulesets?",
                    403,
                    r#"{"message":"Resource not accessible"}"#,
                ),
        );
        let found = block_on(lookup(&*client(transport), "octo/thing"));
        let ProtectionLookup::Unknown(why) = found else {
            panic!("{found:?}")
        };
        assert!(why.contains("lacks the scope"), "{why}");
    }

    #[test]
    fn unreadable_rulesets_keep_the_classic_answer_with_a_note() {
        let transport = Arc::new(
            RecordedTransport::new()
                .on("/branches?protected=true", 200, BRANCHES)
                .on("/rulesets?", 404, r#"{"message":"Not Found"}"#),
        );
        let found = block_on(lookup(&*client(transport), "octo/thing"));
        let ProtectionLookup::Known(found) = found else {
            panic!("{found:?}")
        };
        assert_eq!(found.patterns, ["main", "release/1.x"]);
        assert_eq!(
            found.note.as_deref(),
            Some("the rulesets could not be read")
        );
    }
}
