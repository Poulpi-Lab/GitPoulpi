//! The hosted repository list, grouped by owner: the connected user's own
//! repositories first, then one section per organisation / group, each
//! under a header row. Pure ordering and filtering; the dialog paints and
//! navigates the result.

use super::HostedProject;

/// One maintainer's section of the (filtered) list.
pub(super) struct Group<'a> {
    /// The owner as the host spells it (`acme`, `corp/tools`).
    pub(super) owner: String,
    pub(super) repos: Vec<&'a HostedProject>,
}

/// A row of the painted list: a section header, or the `n`-th repository
/// of the flattened list (the index the cursor and the hits use).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum ListRow {
    Header { owner: String, count: usize },
    Repo(usize),
}

/// The projects matching `search`, one group per owner. The group of
/// `login` (the connected user) comes first, the others follow
/// alphabetically (case-insensitive); repositories are alphabetical
/// inside. A group with no match does not exist.
pub(super) fn group<'a>(
    list: &'a [HostedProject],
    search: &str,
    login: Option<&str>,
) -> Vec<Group<'a>> {
    let mut groups: Vec<Group<'a>> = Vec::new();
    for project in list.iter().filter(|p| p.matches(search)) {
        let owner = project.owner();
        match groups
            .iter_mut()
            .find(|g| g.owner.eq_ignore_ascii_case(owner))
        {
            Some(g) => g.repos.push(project),
            None => groups.push(Group {
                owner: owner.to_string(),
                repos: vec![project],
            }),
        }
    }
    let own = |g: &Group| login.is_some_and(|l| g.owner.eq_ignore_ascii_case(l));
    groups.sort_by_cached_key(|g| (!own(g), g.owner.to_lowercase()));
    for g in &mut groups {
        g.repos
            .sort_by_cached_key(|p| (p.name.to_lowercase(), p.full_name.to_lowercase()));
    }
    groups
}

/// The header + repository rows of `groups`.
pub(super) fn rows(groups: &[Group<'_>]) -> Vec<ListRow> {
    let mut rows = Vec::new();
    let mut index = 0;
    for g in groups {
        rows.push(ListRow::Header {
            owner: g.owner.clone(),
            count: g.repos.len(),
        });
        for _ in &g.repos {
            rows.push(ListRow::Repo(index));
            index += 1;
        }
    }
    rows
}
