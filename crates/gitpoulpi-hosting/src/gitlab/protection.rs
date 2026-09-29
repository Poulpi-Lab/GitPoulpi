//! GitLab's protected branches, for the history rewrite (S23):
//! `GET /projects/{id}/protected_branches` (the `read_api` scope; the user
//! must be able to see the project's repository settings, otherwise GitLab
//! answers 403 or 404 and the lookup is "unknown"). The names are exact or
//! wildcard (`release-*`), the same `*` the workbench understands. Works the
//! same on gitlab.com and on a self-managed instance (the API root is
//! derived from the profile's URL).

use crate::client::fetch_pages;
use crate::gitlab::{encode_gitlab_project_path, GitLabClient};
use crate::protection::ProtectedBranches;
use crate::Result;

#[derive(serde::Deserialize)]
struct Row {
    name: String,
}

impl GitLabClient {
    pub(super) async fn protection(&self, project: &str) -> Result<ProtectedBranches> {
        let url = format!(
            "{}/projects/{}/protected_branches?",
            self.integration.api_base(),
            encode_gitlab_project_path(project)
        );
        let rows: Vec<Row> = fetch_pages(
            &*self.transport,
            |u| self.get(u),
            &url,
            "reading GitLab protected branches",
        )
        .await?;
        let mut out = ProtectedBranches::default();
        for row in rows {
            out.add(&row.name);
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

    fn client(url: &str, transport: Arc<RecordedTransport>) -> Box<dyn crate::HostingClient> {
        client_for(
            Integration::new("gl", IntegrationKind::GitlabSelfManaged, url),
            "tok".into(),
            transport,
        )
    }

    #[test]
    fn protected_branches_keep_their_wildcards_on_a_self_managed_host() {
        let transport = Arc::new(RecordedTransport::new().on(
            "/projects/team%2Fsub%2Fproj/protected_branches",
            200,
            r#"[{"id":1,"name":"main","allow_force_push":false},
                {"id":2,"name":"release-*","allow_force_push":false}]"#,
        ));
        let found = block_on(lookup(
            &*client("https://git.corp.example", transport.clone()),
            "team/sub/proj",
        ));
        let ProtectionLookup::Known(found) = found else {
            panic!("{found:?}")
        };
        assert_eq!(found.patterns, ["main", "release-*"]);
        assert!(transport.requests()[0]
            .url
            .starts_with("https://git.corp.example/api/v4/projects/"));
    }

    #[test]
    fn a_token_that_cannot_see_the_settings_is_unknown() {
        let transport = Arc::new(RecordedTransport::new().on(
            "/protected_branches",
            403,
            r#"{"message":"403 Forbidden"}"#,
        ));
        let found = block_on(lookup(
            &*client("https://git.corp.example", transport),
            "g/p",
        ));
        assert!(matches!(found, ProtectionLookup::Unknown(why) if why.contains("lacks the scope")));
    }
}
