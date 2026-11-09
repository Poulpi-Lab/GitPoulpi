//! Proposing the closest template from the branches a repository has.
//!
//! Score (0-100) of a template:
//! - **bases, 60 points**: the share of the template's bases (`main` or
//!   `master`, `develop` or `dev`, ...) that exist; a template whose trunk
//!   (first base) is missing scores 0;
//! - **topics, 40 points**: the share of the non-base branches its *specific*
//!   topic patterns (not the catch-all `{slug}`) recognise;
//! - **minus 25** for every long-lived base of another template that the
//!   repository has and this one does not define (a repository with `develop`
//!   is not GitHub Flow).
//!
//! Ties go to the template order of [`WorkflowTemplate::ALL`]; `custom` is
//! always last, with score 1.

use serde::{Deserialize, Serialize};

use super::classify::Classification;
use super::templates::WorkflowTemplate;
use super::Workflow;

/// A template base and the concrete branch standing for it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FoundBase {
    /// The template's base name.
    pub base: String,
    /// The branch found (`master` for `main`), if any.
    pub found: Option<String>,
}

/// How many branches a topic type recognised.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FoundTopic {
    pub id: String,
    pub count: usize,
    pub examples: Vec<String>,
}

/// One ranked template.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Detection {
    pub template: WorkflowTemplate,
    pub score: u32,
    pub bases: Vec<FoundBase>,
    pub topics: Vec<FoundTopic>,
    /// Non-base branches the template's specific patterns did not recognise.
    pub unmatched: usize,
}

impl Detection {
    /// The template adapted to what was found: each base takes the spelling
    /// the repository uses, each topic the prefix variant that matched.
    pub fn workflow(&self) -> Workflow {
        let mut w = self.template.workflow();
        for (b, f) in w.bases.iter_mut().zip(&self.bases) {
            if let Some(found) = &f.found {
                if *found != b.name {
                    let old = std::mem::replace(&mut b.name, found.clone());
                    b.names.retain(|n| n != found);
                    b.names.insert(0, old);
                }
            }
        }
        // Parents refer to names or aliases: they keep working after a rename.
        w
    }
}

/// The template without its catch-all `{slug}` patterns, compiled.
fn without_catch_all(w: &Workflow) -> super::CompiledWorkflow {
    let mut specific = w.clone();
    for t in &mut specific.topics {
        t.patterns.retain(|p| p != "{slug}");
        if t.pattern == "{slug}" {
            t.pattern.clear();
        }
    }
    specific.compile()
}

/// Ranks the templates for `branches` (local and remote names, remotes
/// stripped; duplicates are ignored). Best first; `custom` last.
pub fn detect(branches: &[String]) -> Vec<Detection> {
    let mut all: Vec<&str> = branches.iter().map(String::as_str).collect();
    all.sort_unstable();
    all.dedup();
    let has = |n: &str| all.contains(&n);

    let templates = [
        WorkflowTemplate::GitFlow,
        WorkflowTemplate::GithubFlow,
        WorkflowTemplate::GitlabFlow,
        WorkflowTemplate::Trunk,
    ];
    let defs: Vec<Workflow> = templates.iter().map(|t| t.workflow()).collect();
    let known_names = |w: &Workflow| -> Vec<String> {
        w.bases
            .iter()
            .flat_map(|b| b.all_names().map(str::to_string))
            .collect()
    };

    let mut out: Vec<(usize, Detection)> = Vec::new();
    for (order, (template, def)) in templates.iter().zip(&defs).enumerate() {
        let bases: Vec<FoundBase> = def
            .bases
            .iter()
            .map(|b| FoundBase {
                base: b.name.clone(),
                found: b.all_names().find(|n| has(n)).map(str::to_string),
            })
            .collect();
        let trunk_found = bases.first().is_some_and(|b| b.found.is_some());
        let found_count = bases.iter().filter(|b| b.found.is_some()).count();

        let compiled = def.compile();
        let specific = without_catch_all(def);
        let mut counts: Vec<FoundTopic> = def
            .topics
            .iter()
            .map(|t| FoundTopic {
                id: t.id.clone(),
                count: 0,
                examples: Vec::new(),
            })
            .collect();
        let mut non_base = 0usize;
        let mut matched = 0usize;
        for b in &all {
            if compiled.base_named(b).is_some() {
                continue;
            }
            non_base += 1;
            if let Classification::Topic { type_id: id, .. } = specific.classify(b) {
                matched += 1;
                if let Some(c) = counts.iter_mut().find(|c| c.id == id) {
                    c.count += 1;
                    if c.examples.len() < 3 {
                        c.examples.push((*b).to_string());
                    }
                }
            }
        }

        let mut score: i64 = 0;
        if trunk_found {
            score += (60 * found_count / bases.len().max(1)) as i64;
            score += (40 * matched).checked_div(non_base).unwrap_or(0) as i64;
            let mine = known_names(def);
            let foreign = defs
                .iter()
                .filter(|o| o.template != def.template)
                .flat_map(known_names)
                .filter(|n| !mine.contains(n))
                .filter(|n| has(n))
                .collect::<std::collections::BTreeSet<_>>();
            // `prod`/`staging` style aliases are the same long-lived branch.
            score -= 25 * foreign.len() as i64;
        }
        let score = score.clamp(0, 100) as u32;
        if score > 0 {
            out.push((
                order,
                Detection {
                    template: *template,
                    score,
                    bases,
                    topics: counts.into_iter().filter(|c| c.count > 0).collect(),
                    unmatched: non_base - matched,
                },
            ));
        }
    }
    out.sort_by(|(oa, a), (ob, b)| b.score.cmp(&a.score).then(oa.cmp(ob)));
    let mut ranked: Vec<Detection> = out.into_iter().map(|(_, d)| d).collect();
    ranked.push(Detection {
        template: WorkflowTemplate::Custom,
        score: 1,
        bases: Vec::new(),
        topics: Vec::new(),
        unmatched: 0,
    });
    ranked
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(branches: &[&str]) -> Vec<Detection> {
        detect(&branches.iter().map(|s| s.to_string()).collect::<Vec<_>>())
    }

    fn best(branches: &[&str]) -> WorkflowTemplate {
        d(branches)[0].template
    }

    #[test]
    fn main_and_develop_with_feature_branches_is_git_flow() {
        let r = d(&[
            "main",
            "develop",
            "feature/a",
            "feature/b",
            "release/1.0",
            "hotfix/x",
        ]);
        assert_eq!(r[0].template, WorkflowTemplate::GitFlow);
        assert!(r[0].score >= 90, "{}", r[0].score);
        assert!(r[0].score > r[1].score);
    }

    #[test]
    fn master_and_dev_and_plural_prefixes_are_recognised() {
        let r = d(&["master", "dev", "features/a", "releases/1.0"]);
        assert_eq!(r[0].template, WorkflowTemplate::GitFlow);
        let found: Vec<_> = r[0]
            .bases
            .iter()
            .map(|b| (b.base.as_str(), b.found.as_deref()))
            .collect();
        assert_eq!(found, [("main", Some("master")), ("develop", Some("dev"))]);
        let w = r[0].workflow();
        assert_eq!(w.bases[0].name, "master");
        assert_eq!(w.bases[1].name, "dev");
        assert!(w.bases[0].names.contains(&"main".to_string()));
        assert_eq!(w.compile().warnings(), Vec::<String>::new());
        assert!(matches!(w.classify("master"), Classification::Base { .. }));
    }

    #[test]
    fn main_only_with_feature_branches_is_github_flow() {
        assert_eq!(
            best(&["main", "feature/a", "feature/b"]),
            WorkflowTemplate::GithubFlow
        );
        assert_eq!(best(&["main"]), WorkflowTemplate::GithubFlow);
    }

    #[test]
    fn environment_branches_make_gitlab_flow() {
        let r = d(&["main", "pre-production", "production", "feature/a"]);
        assert_eq!(r[0].template, WorkflowTemplate::GitlabFlow, "{r:#?}");
        let r = d(&["main", "staging", "prod"]);
        assert_eq!(r[0].template, WorkflowTemplate::GitlabFlow, "{r:#?}");
    }

    #[test]
    fn user_prefixed_short_lived_branches_make_trunk() {
        assert_eq!(
            best(&["main", "ana/fix", "bob/thing"]),
            WorkflowTemplate::Trunk
        );
    }

    #[test]
    fn develop_keeps_github_flow_down() {
        let r = d(&["main", "develop", "feature/a"]);
        let gh = r
            .iter()
            .find(|x| x.template == WorkflowTemplate::GithubFlow);
        assert!(gh.is_none_or(|g| g.score < r[0].score));
        assert_eq!(r[0].template, WorkflowTemplate::GitFlow);
    }

    #[test]
    fn no_known_base_leaves_only_custom() {
        let r = d(&["trunkish", "wip"]);
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].template, WorkflowTemplate::Custom);
        assert_eq!(d(&[]).len(), 1);
    }

    #[test]
    fn custom_is_always_last_and_scores_are_sorted() {
        let r = d(&["main", "develop", "feature/a", "weird"]);
        assert_eq!(r.last().unwrap().template, WorkflowTemplate::Custom);
        assert!(r.windows(2).all(|w| w[0].score >= w[1].score));
        assert!(r.iter().all(|x| x.score <= 100));
    }

    #[test]
    fn duplicates_and_unmatched_are_counted_once() {
        let r = d(&["main", "main", "develop", "feature/a", "feature/a", "odd"]);
        assert_eq!(r[0].template, WorkflowTemplate::GitFlow);
        assert_eq!(r[0].unmatched, 1);
        assert_eq!(r[0].topics.len(), 1);
        assert_eq!(r[0].topics[0].count, 1);
    }
}
