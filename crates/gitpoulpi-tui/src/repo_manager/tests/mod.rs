use super::*;

fn repo(name: &str) -> KnownRepo {
    KnownRepo {
        path: PathBuf::from(format!("/work/{name}")),
        name: name.to_string(),
        branch_hint: None,
        exists: true,
        is_repo: true,
        tab: None,
        favorite: None,
        recent: None,
        facts: None,
    }
}

/// alpha: open tab 1, recent 0, favorite.
/// beta: open tab 0.
/// gamma: recent 1, gone from disk.
/// delta: favorite only.
fn manager() -> RepoManager {
    let mut alpha = repo("alpha");
    alpha.tab = Some(1);
    alpha.recent = Some(0);
    alpha.favorite = Some(1);
    let mut beta = repo("beta");
    beta.tab = Some(0);
    let mut gamma = repo("gamma");
    gamma.recent = Some(1);
    gamma.exists = false;
    let mut delta = repo("delta");
    delta.favorite = Some(0);
    RepoManager::new(vec![alpha, beta, gamma, delta])
}

fn names(m: &RepoManager, section: Section) -> Vec<String> {
    m.section_repos(section)
        .into_iter()
        .map(|i| m.repos[i].name.clone())
        .collect()
}

fn text(line: &Line) -> String {
    line.spans.iter().map(|s| s.content.as_ref()).collect()
}

mod actions;
mod i18n;
mod keys;
mod mouse;
mod render;
mod sections;
mod workspaces;
