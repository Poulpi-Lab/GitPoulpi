//! Bitbucket Cloud branch restrictions, for the history rewrite (S23):
//! `GET /2.0/repositories/{workspace}/{repo}/branch-restrictions`, which
//! needs the `repository:admin` scope (anything less is a 403 and the lookup
//! is "unknown"). Only the kinds that stop a force push count (`push`,
//! `force`, `delete`); a restriction that names the branching model
//! (`production`, `development`) is not resolved and is said so.

use serde::Deserialize;

use super::BitbucketClient;
use crate::protection::ProtectedBranches;
use crate::Result;

#[derive(Deserialize, Default)]
#[serde(default)]
struct Restriction {
    kind: String,
    pattern: String,
    branch_match_kind: String,
}

impl BitbucketClient {
    pub(super) async fn protection(&self, project: &str) -> Result<ProtectedBranches> {
        let url = format!(
            "{}/repositories/{project}/branch-restrictions?pagelen=100",
            self.api()
        );
        let rows: Vec<Restriction> = self
            .fetch_all(url, "reading the Bitbucket branch restrictions")
            .await?;
        let mut out = ProtectedBranches::default();
        for row in rows
            .into_iter()
            .filter(|r| matches!(r.kind.as_str(), "push" | "force" | "delete"))
        {
            if row.branch_match_kind == "branching_model" || row.pattern.is_empty() {
                out.note("a restriction on a branching-model branch is not resolved");
            } else {
                out.add(&row.pattern);
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

    fn client(transport: Arc<RecordedTransport>) -> Box<dyn crate::HostingClient> {
        client_for(
            Integration::new("bb", IntegrationKind::Bitbucket, "https://bitbucket.org"),
            "tok".into(),
            transport,
            &crate::trust::TrustGate::for_tests(),
        )
        .unwrap()
    }

    #[test]
    fn push_force_and_delete_restrictions_count_and_the_branching_model_is_noted() {
        let transport = Arc::new(RecordedTransport::new().on(
            "/repositories/ws/repo/branch-restrictions",
            200,
            r#"{"values":[
              {"kind":"push","pattern":"main","branch_match_kind":"glob"},
              {"kind":"force","pattern":"release/*","branch_match_kind":"glob"},
              {"kind":"restrict_merges","pattern":"merge-only","branch_match_kind":"glob"},
              {"kind":"delete","pattern":"","branch_match_kind":"branching_model","branch_type":"production"}],
              "next":null}"#,
        ));
        let found = block_on(lookup(&*client(transport), "ws/repo"));
        let ProtectionLookup::Known(found) = found else {
            panic!("{found:?}")
        };
        assert_eq!(found.patterns, ["main", "release/*"]);
        assert!(found.note.unwrap().contains("branching-model"));
    }

    #[test]
    fn a_token_without_the_admin_scope_is_unknown() {
        let transport = Arc::new(RecordedTransport::new().on(
            "/branch-restrictions",
            403,
            r#"{"type":"error","error":{"message":"Your credentials lack one or more required privilege scopes."}}"#,
        ));
        let found = block_on(lookup(&*client(transport), "ws/repo"));
        assert!(matches!(found, ProtectionLookup::Unknown(why) if why.contains("lacks the scope")));
    }
}
