//! Bitbucket Data Center branch permissions, for the history rewrite (S23):
//! `GET /rest/branch-permissions/2.0/projects/{KEY}/repos/{slug}/restrictions`
//! (repository admin; anything less is a 403 and the lookup is "unknown").
//! `fast-forward-only`, `pull-request-only` and `read-only` stop a force
//! push; `no-deletes` does not. The matcher is a branch (`refs/heads/x`), a
//! pattern (`release/*`), or the branching model, which is not resolved.

use serde::Deserialize;

use super::{split_project, BitbucketDcClient};
use crate::client::encode_component;
use crate::protection::ProtectedBranches;
use crate::Result;

#[derive(Deserialize, Default)]
#[serde(default)]
struct Restriction {
    #[serde(rename = "type")]
    kind: String,
    matcher: Matcher,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct Matcher {
    id: String,
    #[serde(rename = "type")]
    kind: MatcherType,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct MatcherType {
    id: String,
}

impl BitbucketDcClient {
    pub(super) async fn protection(&self, project: &str) -> Result<ProtectedBranches> {
        let (key, slug) = split_project(project)?;
        let api = self.api();
        let root = api.strip_suffix("/api/1.0").unwrap_or(&api);
        let url = format!(
            "{root}/branch-permissions/2.0/projects/{}/repos/{}/restrictions?limit={}",
            encode_component(&key),
            encode_component(&slug),
            super::LIMIT
        );
        let rows: Vec<Restriction> = self
            .fetch_all(url, "reading the Bitbucket Data Center branch permissions")
            .await?;
        let mut out = ProtectedBranches::default();
        for row in rows.into_iter().filter(|r| {
            matches!(
                r.kind.as_str(),
                "fast-forward-only" | "pull-request-only" | "read-only"
            )
        }) {
            match row.matcher.kind.id.as_str() {
                "BRANCH" | "PATTERN" => out.add(&row.matcher.id),
                _ => out.note("a permission on a branching-model branch is not resolved"),
            }
        }
        Ok(out)
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

    #[test]
    fn force_push_blocking_permissions_become_patterns() {
        let transport = Arc::new(RecordedTransport::new().on(
            "/rest/branch-permissions/2.0/projects/KEY/repos/slug/restrictions",
            200,
            r#"{"values":[
              {"type":"fast-forward-only","matcher":{"id":"refs/heads/master","type":{"id":"BRANCH"}}},
              {"type":"pull-request-only","matcher":{"id":"release/*","type":{"id":"PATTERN"}}},
              {"type":"no-deletes","matcher":{"id":"refs/heads/keep","type":{"id":"BRANCH"}}},
              {"type":"read-only","matcher":{"id":"PRODUCTION","type":{"id":"MODEL_BRANCH"}}}],
              "isLastPage":true}"#,
        ));
        let c = client_for(
            Integration::new(
                "dc",
                IntegrationKind::BitbucketDataCenter,
                "https://bb.corp.example",
            ),
            "tok".into(),
            transport.clone(),
            &crate::trust::TrustGate::for_tests(),
        )
        .unwrap();
        let found = block_on(lookup(&*c, "KEY/slug"));
        let ProtectionLookup::Known(found) = found else {
            panic!("{found:?}")
        };
        assert_eq!(found.patterns, ["master", "release/*"]);
        assert!(found.note.unwrap().contains("branching-model"));
    }
}
