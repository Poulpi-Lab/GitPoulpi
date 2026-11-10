//! "Ignore the repository's helpers" and a credential manager (ADR-0010).
//!
//! The real Git Credential Manager cannot run in a test, so a global
//! `credential.helper = manager` entry (what an install puts in the system or
//! global config) stands for it: the entry is only read, never run. The
//! machine's helper named like the manager is kept, the repository's own
//! helpers (the plain one and a URL-scoped one) are dropped. On Windows git's
//! own system config usually has the real `manager` entry too, which is
//! machine-wide as well and so is kept the same way.
use gitpoulpi_cli_backend::{credential_helper_entries, machine_wide_helpers};
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};

gitpoulpi_testkit::isolate_binary!();

fn repo() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build()
}

#[test]
fn a_manager_helper_at_machine_scope_is_kept_and_a_local_one_dropped() {
    let global = std::path::PathBuf::from(std::env::var("GIT_CONFIG_GLOBAL").unwrap());
    let mut text = std::fs::read_to_string(&global).unwrap_or_default();
    text.push_str(
        "[credential]\n\thelper = manager\n[credential \"https://corp.example\"]\n\thelper = manager-corp\n",
    );
    std::fs::write(&global, text).unwrap();

    let repo = repo();
    repo.git(&["config", "credential.helper", "!/local/evil-helper"]);
    repo.git(&[
        "config",
        "credential.https://corp.example.helper",
        "!/local/evil-url-helper",
    ]);

    let entries = credential_helper_entries(repo.root()).unwrap();
    assert!(
        entries
            .iter()
            .any(|e| e.scope == "local" && e.value == "!/local/evil-helper"),
        "the repository's helper is listed: {entries:?}"
    );
    let kept = machine_wide_helpers(&entries);
    assert!(
        kept.contains(&(None, "manager".to_string())),
        "the global manager stays: {kept:?}"
    );
    assert!(
        kept.contains(&(
            Some("https://corp.example".to_string()),
            "manager-corp".to_string()
        )),
        "the global URL-scoped helper stays: {kept:?}"
    );
    assert!(
        kept.iter().all(|(_, helper)| !helper.contains("/local/")),
        "no helper of the repository is kept: {kept:?}"
    );
}
