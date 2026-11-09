//! Azure DevOps branch policies, for the history rewrite (S23).
//!
//! `GET {org}/{project}/_apis/policy/configurations?repositoryId={id}`: each
//! enabled, blocking policy carries `settings.scope[]` entries naming a ref
//! (`refs/heads/main`) and how it matches (`Exact`, `Prefix` for a folder
//! such as `releases/*`, `DefaultBranch`), on this repository or on every
//! repository of the project (`repositoryId` null). Readable with a token
//! that has Code (read). The "force push" permission is a security setting,
//! not a policy, and is not read: a branch without a policy is not listed.

use serde::Deserialize;

use super::AzureDevopsClient;
use crate::client::encode_component as enc;
use crate::protection::ProtectedBranches;
use crate::Result;

use super::super::map::{List, Repo};
use super::super::spec::split_project;

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct Policy {
    is_enabled: bool,
    is_blocking: bool,
    settings: Settings,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct Settings {
    scope: Vec<Scope>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct Scope {
    ref_name: Option<String>,
    match_kind: String,
    repository_id: Option<String>,
}

impl AzureDevopsClient {
    pub(super) async fn protection(&self, project: &str) -> Result<ProtectedBranches> {
        let repo_id = self.repo_id(project).await?;
        let (p, _) = split_project(project);
        let url = self.url(
            Some(p),
            "policy/configurations",
            &format!("repositoryId={}", enc(&repo_id)),
        )?;
        let response = self
            .send(self.get(url), "reading the Azure DevOps branch policies")
            .await?;
        let policies: List<Policy> = response.json()?;
        let mut out = ProtectedBranches::default();
        let mut default_branch: Option<String> = None;
        for scope in policies
            .value
            .into_iter()
            .filter(|p| p.is_enabled && p.is_blocking)
            .flat_map(|p| p.settings.scope)
            .filter(|s| s.repository_id.as_deref().is_none_or(|id| id == repo_id))
        {
            match (scope.match_kind.as_str(), scope.ref_name.as_deref()) {
                ("DefaultBranch", _) => {
                    if default_branch.is_none() {
                        default_branch = Some(self.default_branch_of(p, &repo_id).await?);
                    }
                    if let Some(name) = default_branch.as_deref() {
                        out.add(name);
                    }
                }
                ("Prefix", Some(name)) => {
                    let short = name.strip_prefix("refs/heads/").unwrap_or(name);
                    out.add(&format!("{}/*", short.trim_end_matches('/')));
                }
                (_, Some(name)) if name.starts_with("refs/heads/") => out.add(name),
                // A policy on tags or other refs does not guard a branch.
                _ => {}
            }
        }
        Ok(out)
    }

    async fn default_branch_of(&self, project_name: &str, repo_id: &str) -> Result<String> {
        let url = self.url(
            Some(project_name),
            &format!("git/repositories/{}", enc(repo_id)),
            "",
        )?;
        let response = self
            .send(self.get(url), "reading the Azure DevOps repository")
            .await?;
        let repo: Repo = response.json()?;
        Ok(repo.default_branch.unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use gitpoulpi_config::{Integration, IntegrationKind};

    use super::AzureDevopsClient;
    use crate::http::RecordedTransport;
    use crate::protection::{lookup, ProtectionLookup};

    fn block_on<F: std::future::Future>(f: F) -> F::Output {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(f)
    }

    fn client(transport: &Arc<RecordedTransport>) -> AzureDevopsClient {
        AzureDevopsClient::new(
            Integration::new(
                "az",
                IntegrationKind::AzureDevops,
                "https://dev.azure.com/acme",
            ),
            "tok".to_string(),
            transport.clone(),
        )
    }

    const REPO: &str =
        r#"{"id":"r-1","name":"Repo","defaultBranch":"refs/heads/trunk","project":{"name":"P"}}"#;
    const POLICIES: &str = r#"{"count":5,"value":[
      {"isEnabled":true,"isBlocking":true,"settings":{"scope":[
         {"refName":"refs/heads/main","matchKind":"Exact","repositoryId":"r-1"},
         {"refName":"refs/heads/releases","matchKind":"Prefix","repositoryId":"r-1"}]}},
      {"isEnabled":true,"isBlocking":true,"settings":{"scope":[{"matchKind":"DefaultBranch","repositoryId":"r-1"}]}},
      {"isEnabled":true,"isBlocking":false,"settings":{"scope":[{"refName":"refs/heads/warned","matchKind":"Exact","repositoryId":"r-1"}]}},
      {"isEnabled":false,"isBlocking":true,"settings":{"scope":[{"refName":"refs/heads/off","matchKind":"Exact","repositoryId":"r-1"}]}},
      {"isEnabled":true,"isBlocking":true,"settings":{"scope":[{"refName":"refs/heads/other-repo","matchKind":"Exact","repositoryId":"r-9"}]}}]}"#;

    #[test]
    fn blocking_enabled_policies_of_this_repository_become_branch_patterns() {
        let transport = Arc::new(
            RecordedTransport::new()
                .on("policy/configurations", 200, POLICIES)
                .on("git/repositories/", 200, REPO),
        );
        let found = block_on(lookup(&client(&transport), "P/Repo"));
        let ProtectionLookup::Known(found) = found else {
            panic!("{found:?}")
        };
        assert_eq!(found.patterns, ["main", "releases/*", "trunk"]);
    }

    #[test]
    fn the_sign_in_page_of_a_bad_token_is_unknown() {
        let transport = Arc::new(RecordedTransport::new().on("git/repositories/", 203, "<html>"));
        let found = block_on(lookup(&client(&transport), "P/Repo"));
        assert!(
            matches!(found, ProtectionLookup::Unknown(why) if why.contains("rejected the token"))
        );
    }
}
