//! The scope bar: which refs move and which commits may change, and the
//! protected branches the preview left out.
//!
//! The picker edits the recipe's own `scope` and `scope_options`. What it
//! offers (the Log's selection, the local branches and tags) is handed to it
//! in a [`ScopeContext`] when the workbench opens, so nothing here reads the
//! repository.
//!
//! **Protected branches** (maintainer decision, 2026-10-21): a branch the
//! repository's workflow marks protected and that is already pushed is
//! left out of a run unless it is listed in `scope_options.include_protected`.
//! The preview names the ones it left out (and the ones it included); the
//! picker shows one checkbox each, plus "check all" and "uncheck all". A
//! change here means the plan on screen no longer matches the recipe, so the
//! caller marks it out of date.

use gitpoulpi_core::domain::rewrite::{Recipe, RefChange, RewritePlan, RewriteWarning, Scope};
use gitpoulpi_core::domain::Oid;
use gitpoulpi_i18n::{msg::rewrite_view as t, M};

/// What the scope offers, read once when the workbench opens.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ScopeContext {
    /// The Log's multi-selection (empty when none).
    pub selection: Vec<Oid>,
    /// The checked-out branch, short name.
    pub head_branch: Option<String>,
    /// Local branches, short names.
    pub branches: Vec<String>,
    /// Tags, short names.
    pub tags: Vec<String>,
    /// The commit the Log menu's "From this commit to HEAD" named: the
    /// scope's `since` checkbox offers it.
    pub since: Option<Oid>,
    /// Start on the Log's selection instead of the current branch.
    pub use_selection: bool,
    /// The commits "Test on selection" runs a script on: the Log's selected
    /// commits, else the newest ones shown.
    pub test_commits: Vec<Oid>,
    /// `test_commits` is the user's selection (not the newest ones).
    pub test_from_selection: bool,
    /// Author, date and subject of the same commits, for the ✦ script
    /// dialog's optional sample. Never anything else about them.
    pub sample: Vec<super::SampleRow>,
}

/// The kinds of scope the combo offers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ScopeKind {
    CurrentBranch,
    AllLocal,
    Refs,
    Selection,
    Range,
}

impl ScopeKind {
    pub(crate) const ALL: [ScopeKind; 5] = [
        ScopeKind::CurrentBranch,
        ScopeKind::AllLocal,
        ScopeKind::Refs,
        ScopeKind::Selection,
        ScopeKind::Range,
    ];

    pub(crate) const fn label(self) -> M {
        match self {
            ScopeKind::CurrentBranch => t::S_CURRENT_BRANCH,
            ScopeKind::AllLocal => t::S_ALL_LOCAL,
            ScopeKind::Refs => t::S_REFS,
            ScopeKind::Selection => t::S_SELECTION,
            ScopeKind::Range => t::S_RANGE,
        }
    }

    pub(crate) fn of(scope: &Scope) -> Self {
        match scope {
            Scope::CurrentBranch { .. } => ScopeKind::CurrentBranch,
            Scope::AllLocal => ScopeKind::AllLocal,
            Scope::Refs { .. } => ScopeKind::Refs,
            Scope::Selection { .. } => ScopeKind::Selection,
            Scope::Range { .. } => ScopeKind::Range,
        }
    }
}

/// The scope combo, one row per kind that can be chosen right now. A
/// selection scope needs a Log selection, a range needs the two ends the
/// caller gives it.
pub(crate) fn kinds_offered(ctx: &ScopeContext, recipe: &Recipe) -> Vec<ScopeKind> {
    ScopeKind::ALL
        .into_iter()
        .filter(|k| match k {
            ScopeKind::Selection => !ctx.selection.is_empty() || ScopeKind::of(&recipe.scope) == *k,
            ScopeKind::Range => ScopeKind::of(&recipe.scope) == *k,
            _ => true,
        })
        .collect()
}

/// Switches the scope to `kind`. Returns `false` when the kind cannot be
/// built from what the context offers (a selection with nothing selected).
pub(crate) fn set_kind(recipe: &mut Recipe, ctx: &ScopeContext, kind: ScopeKind) -> bool {
    if ScopeKind::of(&recipe.scope) == kind {
        return true;
    }
    recipe.scope = match kind {
        ScopeKind::CurrentBranch => Scope::CurrentBranch { since: None },
        ScopeKind::AllLocal => Scope::AllLocal,
        ScopeKind::Refs => Scope::Refs {
            refs: ctx.head_branch.iter().cloned().collect(),
        },
        ScopeKind::Selection => {
            if ctx.selection.is_empty() {
                return false;
            }
            Scope::Selection {
                commits: ctx.selection.clone(),
            }
        }
        // A range comes from a Log action that names both ends.
        ScopeKind::Range => return false,
    };
    true
}

/// The "commits" combo: all of them, or only those after one commit.
pub(crate) fn set_since(recipe: &mut Recipe, since: Option<Oid>) -> bool {
    match &mut recipe.scope {
        Scope::CurrentBranch { since: s } => {
            *s = since;
            true
        }
        _ => false,
    }
}

/// The commit a current-branch scope starts after.
pub(crate) fn since(recipe: &Recipe) -> Option<Oid> {
    match &recipe.scope {
        Scope::CurrentBranch { since } => *since,
        _ => None,
    }
}

pub(crate) fn toggle_tags(recipe: &mut Recipe) {
    recipe.scope_options.include_tags = !recipe.scope_options.include_tags;
}

/// One row of the "Refs..." checklist.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RefChoice {
    pub name: String,
    pub is_tag: bool,
    pub checked: bool,
}

/// The branches and tags the user can tick for a refs scope.
pub(crate) fn ref_choices(recipe: &Recipe, ctx: &ScopeContext) -> Vec<RefChoice> {
    let picked: &[String] = match &recipe.scope {
        Scope::Refs { refs } => refs,
        _ => &[],
    };
    let row = |name: &String, is_tag: bool| RefChoice {
        name: name.clone(),
        is_tag,
        checked: picked.contains(name),
    };
    ctx.branches
        .iter()
        .map(|n| row(n, false))
        .chain(ctx.tags.iter().map(|n| row(n, true)))
        .collect()
}

/// Ticks or unticks `name` of a refs scope.
pub(crate) fn toggle_ref(recipe: &mut Recipe, name: &str) -> bool {
    let Scope::Refs { refs } = &mut recipe.scope else {
        return false;
    };
    match refs.iter().position(|r| r == name) {
        Some(at) => {
            refs.remove(at);
        }
        None => refs.push(name.to_string()),
    }
    true
}

// ----------------------------------------------------- protected branches

/// One protected branch the preview met.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProtectedEntry {
    /// The full ref name (`refs/heads/main`).
    pub name: String,
    /// Ticked: the run rewrites it.
    pub included: bool,
    /// It is already on a remote (a force push follows).
    pub pushed: bool,
}

fn is_included(recipe: &Recipe, name: &str) -> bool {
    let short = name.strip_prefix("refs/heads/").unwrap_or(name);
    recipe
        .scope_options
        .include_protected
        .iter()
        .any(|n| n == name || n == short)
}

fn pushed(change: &RefChange) -> bool {
    matches!(
        change.pushed,
        gitpoulpi_core::domain::rewrite::PushState::OnRemote { .. }
    )
}

/// The protected branches of a preview, ticked as the recipe says: the ones
/// the plan moves and the ones it reported leaving out. Sorted by name.
pub(crate) fn protected_entries(recipe: &Recipe, plan: &RewritePlan) -> Vec<ProtectedEntry> {
    let mut out: Vec<ProtectedEntry> = plan
        .refs
        .iter()
        .filter(|r| r.protected)
        .map(|r| ProtectedEntry {
            name: r.name.clone(),
            included: is_included(recipe, &r.name),
            pushed: pushed(r),
        })
        .collect();
    for warning in &plan.warnings {
        if let RewriteWarning::ProtectedExcluded { refs } = warning {
            for name in refs {
                if !out.iter().any(|e| &e.name == name) {
                    out.push(ProtectedEntry {
                        name: name.clone(),
                        included: is_included(recipe, name),
                        pushed: true,
                    });
                }
            }
        }
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

/// Ticks or unticks one protected branch. `true` when the recipe changed.
pub(crate) fn toggle_protected(recipe: &mut Recipe, name: &str) -> bool {
    let list = &mut recipe.scope_options.include_protected;
    let short = name.strip_prefix("refs/heads/").unwrap_or(name);
    let before = list.len();
    list.retain(|n| n != name && n != short);
    if list.len() == before {
        list.push(name.to_string());
    }
    true
}

/// "Check all": every protected branch of the plan is included.
pub(crate) fn check_all_protected(recipe: &mut Recipe, plan: &RewritePlan) -> bool {
    let all: Vec<String> = protected_entries(recipe, plan)
        .into_iter()
        .map(|e| e.name)
        .collect();
    let list = &mut recipe.scope_options.include_protected;
    let before = list.clone();
    for name in all {
        let short = name
            .strip_prefix("refs/heads/")
            .unwrap_or(&name)
            .to_string();
        if !list.iter().any(|n| *n == name || *n == short) {
            list.push(name);
        }
    }
    *list != before
}

/// "Uncheck all": the global default again, every protected branch left out.
pub(crate) fn uncheck_all_protected(recipe: &mut Recipe) -> bool {
    let had = !recipe.scope_options.include_protected.is_empty();
    recipe.scope_options.include_protected.clear();
    had
}

/// The three numbers of the scope bar's tail ("1 branch, 4 tags, 9,800
/// commits"), from the last preview.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ScopeCounts {
    pub branches: u32,
    pub tags: u32,
    pub commits: u64,
}

pub(crate) fn counts(plan: &RewritePlan) -> ScopeCounts {
    ScopeCounts {
        branches: plan.counts.refs_moved,
        tags: plan.counts.tags_moved,
        commits: plan.counts.commits_walked,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitpoulpi_core::domain::rewrite::{Counts, PlanDigest, PushState, Rule, RuleKind, RunId};

    fn recipe() -> Recipe {
        Recipe::new(
            "t",
            Scope::CurrentBranch { since: None },
            vec![Rule::new(RuleKind::CommitterFromAuthor)],
        )
    }

    fn change(name: &str, protected: bool, on_remote: bool) -> RefChange {
        RefChange {
            name: name.into(),
            old: Oid([1; 20]),
            new: Some(Oid([2; 20])),
            pushed: if on_remote {
                PushState::OnRemote {
                    remote: "origin".into(),
                    remote_oid: Oid([1; 20]),
                    rewritten_on_remote: 2,
                }
            } else {
                PushState::LocalOnly
            },
            protected,
            included: false,
            checked_out_in: None,
        }
    }

    fn plan(refs: Vec<RefChange>, excluded: &[&str]) -> RewritePlan {
        RewritePlan {
            run: RunId("r".into()),
            digest: PlanDigest("d".into()),
            counts: Counts::default(),
            refs,
            warnings: if excluded.is_empty() {
                Vec::new()
            } else {
                vec![RewriteWarning::ProtectedExcluded {
                    refs: excluded.iter().map(|s| s.to_string()).collect(),
                }]
            },
            per_rule: Vec::new(),
            rows: Vec::new(),
        }
    }

    #[test]
    fn protected_branches_start_unchecked_and_each_one_toggles() {
        let mut r = recipe();
        let p = plan(
            vec![
                change("refs/heads/main", true, true),
                change("refs/heads/feature", false, false),
            ],
            &["refs/heads/release"],
        );
        let entries = protected_entries(&r, &p);
        assert_eq!(
            entries.iter().map(|e| e.name.as_str()).collect::<Vec<_>>(),
            ["refs/heads/main", "refs/heads/release"]
        );
        assert!(entries.iter().all(|e| !e.included));
        toggle_protected(&mut r, "refs/heads/release");
        let entries = protected_entries(&r, &p);
        assert!(!entries[0].included && entries[1].included);
        toggle_protected(&mut r, "refs/heads/release");
        assert!(r.scope_options.include_protected.is_empty());
    }

    #[test]
    fn check_all_includes_every_protected_branch_and_uncheck_all_none() {
        let mut r = recipe();
        let p = plan(
            vec![change("refs/heads/main", true, true)],
            &["refs/heads/release"],
        );
        assert!(check_all_protected(&mut r, &p));
        assert!(protected_entries(&r, &p).iter().all(|e| e.included));
        assert!(!check_all_protected(&mut r, &p), "already all checked");
        assert!(uncheck_all_protected(&mut r));
        assert!(protected_entries(&r, &p).iter().all(|e| !e.included));
        assert!(!uncheck_all_protected(&mut r));
    }

    #[test]
    fn a_short_name_in_the_recipe_counts_as_the_full_ref() {
        let mut r = recipe();
        r.scope_options.include_protected = vec!["main".into()];
        let p = plan(vec![change("refs/heads/main", true, true)], &[]);
        assert!(protected_entries(&r, &p)[0].included);
        toggle_protected(&mut r, "refs/heads/main");
        assert!(r.scope_options.include_protected.is_empty());
    }

    #[test]
    fn the_scope_kind_changes_and_a_selection_needs_a_selection() {
        let mut r = recipe();
        let ctx = ScopeContext {
            head_branch: Some("main".into()),
            branches: vec!["main".into(), "dev".into()],
            tags: vec!["v1".into()],
            ..ScopeContext::default()
        };
        assert!(!set_kind(&mut r, &ctx, ScopeKind::Selection));
        assert_eq!(ScopeKind::of(&r.scope), ScopeKind::CurrentBranch);
        assert!(set_kind(&mut r, &ctx, ScopeKind::Refs));
        assert_eq!(
            r.scope,
            Scope::Refs {
                refs: vec!["main".into()]
            }
        );
        assert!(toggle_ref(&mut r, "v1"));
        let choices = ref_choices(&r, &ctx);
        assert_eq!(
            choices.iter().map(|c| c.checked).collect::<Vec<_>>(),
            [true, false, true]
        );
        assert!(choices[2].is_tag);
        assert!(set_kind(&mut r, &ctx, ScopeKind::AllLocal));
        assert!(!toggle_ref(&mut r, "v1"));
    }

    #[test]
    fn the_selection_kind_is_offered_only_with_a_selection() {
        let r = recipe();
        let none = ScopeContext::default();
        assert!(!kinds_offered(&none, &r).contains(&ScopeKind::Selection));
        let some = ScopeContext {
            selection: vec![Oid([3; 20])],
            ..ScopeContext::default()
        };
        assert!(kinds_offered(&some, &r).contains(&ScopeKind::Selection));
        let mut r = r;
        assert!(set_kind(&mut r, &some, ScopeKind::Selection));
        assert_eq!(
            r.scope,
            Scope::Selection {
                commits: vec![Oid([3; 20])]
            }
        );
    }

    #[test]
    fn since_and_tags_edit_the_recipe() {
        let mut r = recipe();
        assert!(set_since(&mut r, Some(Oid([9; 20]))));
        assert_eq!(since(&r), Some(Oid([9; 20])));
        assert!(r.scope_options.include_tags);
        toggle_tags(&mut r);
        assert!(!r.scope_options.include_tags);
        r.scope = Scope::AllLocal;
        assert!(!set_since(&mut r, None));
    }
}
