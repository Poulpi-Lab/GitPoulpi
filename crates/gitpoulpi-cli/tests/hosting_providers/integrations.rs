//! `hosting integrations add` validates per kind (account, organization,
//! url) before writing anything; `list` shows the non-secret `account` and
//! never a token.
#![allow(dead_code)] // the helpers serve feature-gated tests

use std::path::Path;

use crate::common::*;

fn cli(name: &str) -> Cli {
    Cli::new(name, "", &[])
}

fn here() -> &'static Path {
    Path::new(".")
}

#[test]
#[cfg(feature = "jira")]
fn jira_cloud_needs_an_account_and_a_site() {
    let cli = cli("add-jira");
    let err = cli.error(
        here(),
        &[
            "hosting",
            "integrations",
            "add",
            "--kind",
            "jira_cloud",
            "--url",
            "https://acme.atlassian.net",
        ],
    );
    assert!(err.contains("--account"), "{err}");
    let err = cli.error(
        here(),
        &[
            "hosting",
            "integrations",
            "add",
            "--kind",
            "jira_cloud",
            "--account",
            "me@x.io",
        ],
    );
    assert!(err.contains("--url"), "{err}");
    assert!(
        !cli.config_text().contains("jira_cloud"),
        "nothing was written"
    );

    let added = cli.json(
        here(),
        &[
            "hosting",
            "integrations",
            "add",
            "--kind",
            "jira_cloud",
            "--url",
            "https://acme.atlassian.net/",
            "--account",
            "me@x.io",
        ],
    );
    assert_eq!(added["added"]["kind"], "jira_cloud");
    assert_eq!(added["added"]["account"], "me@x.io");
    assert_eq!(added["added"]["url"], "https://acme.atlassian.net");
    assert_eq!(added["added"]["category"], "issue_tracker");

    let listed = cli.json(here(), &["hosting", "integrations", "list"]);
    let row = &listed["integrations"][0];
    assert_eq!(row["account"], "me@x.io");
    assert_eq!(row["api_base"], "https://acme.atlassian.net/rest/api/3");
    assert!(row.get("token").is_none(), "{row}");
}

#[test]
#[cfg(feature = "azure-devops")]
fn azure_devops_needs_an_organization_and_server_a_url() {
    let cli = cli("add-azure");
    let err = cli.error(
        here(),
        &["hosting", "integrations", "add", "--kind", "azure_devops"],
    );
    assert!(err.contains("--org"), "{err}");

    let added = cli.json(
        here(),
        &[
            "hosting",
            "integrations",
            "add",
            "--kind",
            "azure_devops",
            "--org",
            "acme",
        ],
    );
    assert_eq!(added["added"]["url"], "https://dev.azure.com/acme");
    assert_eq!(added["added"]["org"], "acme");

    // The organization may come from the url instead.
    let added = cli.json(
        here(),
        &[
            "hosting",
            "integrations",
            "add",
            "--kind",
            "azure_devops",
            "--url",
            "https://dev.azure.com/other",
            "--id",
            "az-other",
        ],
    );
    assert_eq!(added["added"]["org"], "other");

    let err = cli.error(
        here(),
        &[
            "hosting",
            "integrations",
            "add",
            "--kind",
            "azure_devops_server",
        ],
    );
    assert!(err.contains("--url"), "{err}");
    cli.json(
        here(),
        &[
            "hosting",
            "integrations",
            "add",
            "--kind",
            "azure_devops_server",
            "--url",
            "https://tfs.corp/tfs/DefaultCollection",
            "--id",
            "tfs",
        ],
    );
}

#[test]
#[cfg(feature = "trello")]
fn trello_needs_its_api_key_as_the_account() {
    let cli = cli("add-trello");
    let err = cli.error(
        here(),
        &["hosting", "integrations", "add", "--kind", "trello"],
    );
    assert!(err.contains("--account"), "{err}");
    let added = cli.json(
        here(),
        &[
            "hosting",
            "integrations",
            "add",
            "--kind",
            "trello",
            "--account",
            "KEY123",
        ],
    );
    assert_eq!(added["added"]["url"], "https://trello.com");
    assert_eq!(added["added"]["account"], "KEY123");
}

#[test]
#[cfg(feature = "bitbucket")]
fn bitbucket_takes_an_optional_account() {
    let cli = cli("add-bitbucket");
    let added = cli.json(
        here(),
        &[
            "hosting",
            "integrations",
            "add",
            "--kind",
            "bitbucket",
            "--account",
            "me@x.io",
        ],
    );
    assert_eq!(added["added"]["url"], "https://bitbucket.org");
    assert_eq!(added["added"]["account"], "me@x.io");
    cli.json(
        here(),
        &[
            "hosting",
            "integrations",
            "add",
            "--kind",
            "bitbucket",
            "--id",
            "bb-bearer",
        ],
    );
}

#[test]
#[cfg(feature = "github")]
fn org_and_account_are_refused_where_the_kind_has_none() {
    let cli = cli("add-github");
    let err = cli.error(
        here(),
        &[
            "hosting",
            "integrations",
            "add",
            "--kind",
            "github",
            "--org",
            "acme",
        ],
    );
    assert!(err.contains("--org"), "{err}");
    let err = cli.error(
        here(),
        &[
            "hosting",
            "integrations",
            "add",
            "--kind",
            "github",
            "--account",
            "me@x.io",
        ],
    );
    assert!(err.contains("--account"), "{err}");
    assert!(!cli.config_text().contains("github"));
}

#[test]
#[cfg(feature = "gitea")]
fn gitea_needs_a_url_and_forgejo_defaults_to_codeberg() {
    let cli = cli("add-gitea");
    let err = cli.error(
        here(),
        &["hosting", "integrations", "add", "--kind", "gitea"],
    );
    assert!(err.contains("--url"), "{err}");
    let added = cli.json(
        here(),
        &[
            "hosting",
            "integrations",
            "add",
            "--kind",
            "gitea",
            "--url",
            "git.example.com/",
        ],
    );
    assert_eq!(added["added"]["url"], "https://git.example.com");
    assert_eq!(added["added"]["api_base"], "https://git.example.com/api/v1");
    let added = cli.json(
        here(),
        &["hosting", "integrations", "add", "--kind", "forgejo"],
    );
    assert_eq!(added["added"]["url"], "https://codeberg.org");
    assert_eq!(added["added"]["id"], "forgejo");
    let err = cli.error(
        here(),
        &[
            "hosting",
            "integrations",
            "add",
            "--kind",
            "forgejo",
            "--account",
            "x",
        ],
    );
    assert!(err.contains("--account"), "{err}");
}
