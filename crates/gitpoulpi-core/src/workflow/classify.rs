//! Classifying branches against a workflow, and finding their parents.
//!
//! Order, in full:
//! 1. A branch whose name is a base's name (or one of its alternative names)
//!    is that **base**.
//! 2. Otherwise every topic pattern (`pattern`, then `patterns`, then `regex`,
//!    of each topic in declaration order) that matches is a candidate; the
//!    candidate fixing the most literal characters wins (`release/{version}`
//!    beats `{slug}`; a regex counts its literal prefix), ties go to the
//!    earlier declaration.
//! 3. Otherwise **unclassified**.

use regex::Regex;
use serde::{Deserialize, Serialize};

use super::pattern::{BranchPattern, Fields};
use super::{BaseBranch, TopicType, Workflow};

/// What a branch is.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Classification {
    /// A long-lived base; carries the configured (preferred) name.
    Base {
        name: String,
    },
    /// A topic branch of a type, with the fields its name carried.
    Topic {
        type_id: String,
        fields: Fields,
    },
    Unclassified,
}

/// Where a parent came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParentSource {
    /// Recorded when GitPoulpi created the branch (`branch.<b>.gitpoulpi-parent`).
    Recorded,
    /// Read from Git Town's `git-town-branch.<b>.parent`.
    GitTown,
    /// The topic type's parent.
    Type,
    /// The base's parent.
    Base,
}

/// The branch a branch starts from and is finished into.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParentInfo {
    pub name: String,
    pub source: ParentSource,
}

enum Matcher {
    Pattern(BranchPattern),
    Regex(Regex, usize),
}

impl Matcher {
    fn specificity(&self) -> usize {
        match self {
            Matcher::Pattern(p) => p.specificity(),
            Matcher::Regex(_, s) => *s,
        }
    }

    fn fields(&self, name: &str) -> Option<Fields> {
        match self {
            Matcher::Pattern(p) => p.parse_name(name),
            Matcher::Regex(re, _) => {
                let caps = re.captures(name)?;
                Some(
                    re.capture_names()
                        .flatten()
                        .filter_map(|n| Some((n.to_string(), caps.name(n)?.as_str().to_string())))
                        .collect(),
                )
            }
        }
    }
}

/// Literal characters at the start of a regex (after `^`).
fn regex_literal_prefix(source: &str) -> usize {
    source
        .trim_start_matches('^')
        .chars()
        .take_while(|c| !"\\.[](){}*+?|$^".contains(*c))
        .count()
}

struct CompiledTopic {
    matchers: Vec<Matcher>,
    /// The rendering pattern (the first valid of `pattern`).
    render: Option<BranchPattern>,
}

/// A [`Workflow`] with its patterns parsed, ready to classify many branches.
pub struct CompiledWorkflow {
    workflow: Workflow,
    topics: Vec<CompiledTopic>,
    warnings: Vec<String>,
}

impl CompiledWorkflow {
    pub fn new(workflow: &Workflow) -> Self {
        let mut warnings = Vec::new();
        let mut topics = Vec::new();
        for t in &workflow.topics {
            let mut matchers = Vec::new();
            let mut render = None;
            if t.id.trim().is_empty() {
                warnings.push("a topic type has no `id`".to_string());
            }
            let sources = std::iter::once((true, t.pattern.as_str()))
                .chain(t.patterns.iter().map(|p| (false, p.as_str())))
                .filter(|(primary, p)| !(*primary && p.is_empty()));
            if t.pattern.is_empty() && t.patterns.is_empty() && t.regex.is_none() {
                warnings.push(format!("topic `{}` has no pattern or regex", t.id));
            }
            for (primary, source) in sources {
                match BranchPattern::parse(source) {
                    Ok(p) => {
                        if primary {
                            render = Some(p.clone());
                        }
                        matchers.push(Matcher::Pattern(p));
                    }
                    Err(e) => warnings.push(format!(
                        "topic `{}`: pattern `{source}` is invalid: {e}",
                        t.id
                    )),
                }
            }
            if let Some(src) = &t.regex {
                match Regex::new(&format!(
                    "^(?:{})$",
                    src.trim_start_matches('^').trim_end_matches('$')
                )) {
                    Ok(re) => matchers.push(Matcher::Regex(re, regex_literal_prefix(src))),
                    Err(e) => {
                        warnings.push(format!("topic `{}`: regex `{src}` is invalid: {e}", t.id))
                    }
                }
            }
            topics.push(CompiledTopic { matchers, render });
        }
        for b in &workflow.bases {
            if b.name.trim().is_empty() {
                warnings.push("a base branch has no `name`".to_string());
            }
            if let Some(p) = &b.parent {
                if !workflow.bases.iter().any(|o| o.is_named(p)) {
                    warnings.push(format!(
                        "base `{}`: parent `{p}` is not a defined base",
                        b.name
                    ));
                }
            }
        }
        for t in &workflow.topics {
            if let Some(p) = &t.parent {
                if !workflow.bases.iter().any(|o| o.is_named(p)) {
                    warnings.push(format!(
                        "topic `{}`: parent `{p}` is not a defined base",
                        t.id
                    ));
                }
            }
        }
        Self {
            workflow: workflow.clone(),
            topics,
            warnings,
        }
    }

    pub fn workflow(&self) -> &Workflow {
        &self.workflow
    }

    /// Problems found while compiling (bad patterns, unknown parents).
    pub fn warnings(&self) -> &[String] {
        &self.warnings
    }

    /// The base a branch name denotes, by any of its names.
    pub fn base_named(&self, name: &str) -> Option<&BaseBranch> {
        self.workflow.bases.iter().find(|b| b.is_named(name))
    }

    pub fn topic(&self, id: &str) -> Option<&TopicType> {
        self.workflow.topics.iter().find(|t| t.id == id)
    }

    /// The pattern that renders names of topic `id`.
    pub fn render_pattern(&self, id: &str) -> Option<&BranchPattern> {
        let idx = self.workflow.topics.iter().position(|t| t.id == id)?;
        self.topics.get(idx)?.render.as_ref()
    }

    /// Classifies a local branch name (for a remote branch, pass the name
    /// without the remote).
    pub fn classify(&self, branch: &str) -> Classification {
        if let Some(base) = self.base_named(branch) {
            return Classification::Base {
                name: base.name.clone(),
            };
        }
        let mut best: Option<(usize, usize, Fields)> = None;
        for (idx, topic) in self.topics.iter().enumerate() {
            for m in &topic.matchers {
                let Some(fields) = m.fields(branch) else {
                    continue;
                };
                let spec = m.specificity();
                // Strictly greater: an earlier declaration wins a tie.
                if best.as_ref().is_none_or(|(s, _, _)| spec > *s) {
                    best = Some((spec, idx, fields));
                }
            }
        }
        match best {
            Some((_, idx, fields)) => Classification::Topic {
                type_id: self.workflow.topics[idx].id.clone(),
                fields,
            },
            None => Classification::Unclassified,
        }
    }

    /// The existing branch a base reference (`develop`, or any of its names)
    /// stands for: the first of the base's names `exists` accepts.
    pub fn concrete_base(&self, reference: &str, exists: &dyn Fn(&str) -> bool) -> Option<String> {
        let base = self.base_named(reference)?;
        base.all_names().find(|n| exists(n)).map(str::to_string)
    }

    /// Where `class` starts from: the recorded parent if that branch still
    /// exists, else the type's (or base's) parent as an existing branch.
    pub fn resolve_parent(
        &self,
        class: &Classification,
        recorded: Option<(&str, ParentSource)>,
        exists: &dyn Fn(&str) -> bool,
    ) -> Option<ParentInfo> {
        if let Some((name, source)) = recorded.filter(|(n, _)| exists(n)) {
            return Some(ParentInfo {
                name: name.to_string(),
                source,
            });
        }
        let (reference, source) = match class {
            Classification::Base { name } => (
                self.base_named(name)?.parent.as_deref()?,
                ParentSource::Base,
            ),
            Classification::Topic { type_id, .. } => {
                (self.topic(type_id)?.parent.as_deref()?, ParentSource::Type)
            }
            Classification::Unclassified => return None,
        };
        let name = self.concrete_base(reference, exists)?;
        Some(ParentInfo { name, source })
    }
}

#[cfg(test)]
mod tests {
    use super::super::templates::WorkflowTemplate;
    use super::*;

    fn topic(id: &str, pattern: &str, parent: &str) -> TopicType {
        TopicType {
            id: id.into(),
            pattern: pattern.into(),
            parent: Some(parent.into()),
            ..Default::default()
        }
    }

    fn base(name: &str) -> BaseBranch {
        BaseBranch {
            name: name.into(),
            ..Default::default()
        }
    }

    fn type_of(w: &Workflow, b: &str) -> String {
        match w.classify(b) {
            Classification::Base { name } => format!("base:{name}"),
            Classification::Topic { type_id, .. } => type_id,
            Classification::Unclassified => "-".into(),
        }
    }

    #[test]
    fn bases_win_over_topic_patterns() {
        let w = Workflow {
            bases: vec![base("main")],
            topics: vec![topic("any", "{slug}", "main")],
            ..Default::default()
        };
        assert_eq!(type_of(&w, "main"), "base:main");
        assert_eq!(type_of(&w, "other"), "any");
    }

    #[test]
    fn alias_names_classify_as_the_configured_base() {
        let w = Workflow {
            bases: vec![BaseBranch {
                name: "main".into(),
                names: vec!["master".into()],
                ..Default::default()
            }],
            ..Default::default()
        };
        assert_eq!(type_of(&w, "master"), "base:main");
    }

    #[test]
    fn the_most_specific_pattern_wins_whatever_the_order() {
        let w = Workflow {
            bases: vec![base("main")],
            topics: vec![
                topic("any", "{slug}", "main"),
                topic("user", "{user}/{slug}", "main"),
                topic("feature", "feature/{slug}", "main"),
                topic("release", "release/{version}", "main"),
            ],
            ..Default::default()
        };
        assert_eq!(type_of(&w, "feature/x"), "feature");
        assert_eq!(type_of(&w, "release/1.0"), "release");
        // not a version: the release pattern does not match, `{user}/{slug}` does
        assert_eq!(type_of(&w, "release/next"), "user");
        assert_eq!(type_of(&w, "ana/x"), "user");
        assert_eq!(type_of(&w, "plain"), "any");
    }

    #[test]
    fn a_tie_goes_to_the_earlier_declaration() {
        let w = Workflow {
            topics: vec![topic("a", "x/{slug}", "m"), topic("b", "x/{issue}", "m")],
            ..Default::default()
        };
        assert_eq!(type_of(&w, "x/12"), "a");
        let w = Workflow {
            topics: vec![topic("b", "x/{issue}", "m"), topic("a", "x/{slug}", "m")],
            ..Default::default()
        };
        assert_eq!(type_of(&w, "x/12"), "b");
        assert_eq!(type_of(&w, "x/ab"), "a");
    }

    #[test]
    fn regex_topics_match_and_extract_named_groups() {
        let w = Workflow {
            topics: vec![TopicType {
                id: "feat".into(),
                pattern: "feat/{slug}".into(),
                regex: Some("^feat-(?P<slug>.+)$".into()),
                ..Default::default()
            }],
            ..Default::default()
        };
        match w.classify("feat-login") {
            Classification::Topic { type_id, fields } => {
                assert_eq!(type_id, "feat");
                assert_eq!(fields["slug"], "login");
            }
            other => panic!("{other:?}"),
        }
        assert_eq!(type_of(&w, "xfeat-login"), "-", "regex is anchored");
        assert_eq!(type_of(&w, "feat/ok"), "feat");
    }

    #[test]
    fn a_regex_only_topic_has_no_render_pattern() {
        let w = Workflow {
            topics: vec![TopicType {
                id: "bot".into(),
                regex: Some("bot/.*".into()),
                ..Default::default()
            }],
            ..Default::default()
        };
        let c = w.compile();
        assert!(c.render_pattern("bot").is_none());
        assert!(c.warnings().is_empty(), "{:?}", c.warnings());
        assert_eq!(type_of(&w, "bot/1"), "bot");
    }

    #[test]
    fn broken_definitions_warn_and_are_skipped() {
        let w = Workflow {
            bases: vec![BaseBranch {
                name: "d".into(),
                parent: Some("ghost".into()),
                ..Default::default()
            }],
            topics: vec![
                topic("bad", "x/{nope}", "d"),
                TopicType {
                    id: "re".into(),
                    pattern: "re/{slug}".into(),
                    regex: Some("(".into()),
                    parent: Some("zzz".into()),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        let c = w.compile();
        let all = c.warnings().join("\n");
        assert!(all.contains("x/{nope}"), "{all}");
        assert!(all.contains("regex `(`"), "{all}");
        assert!(all.contains("parent `ghost`"), "{all}");
        assert!(all.contains("parent `zzz`"), "{all}");
        assert_eq!(c.classify("re/a"), w.classify("re/a"));
        assert_eq!(type_of(&w, "re/a"), "re", "the valid pattern still works");
        assert_eq!(type_of(&w, "x/y"), "-");
    }

    #[test]
    fn parent_resolution_prefers_recorded_then_type() {
        let w = WorkflowTemplate::GitFlow.workflow();
        let c = w.compile();
        let existing = ["master", "dev", "feature/a", "other"];
        let exists = |n: &str| existing.contains(&n);
        let class = c.classify("feature/a");
        // type parent: develop resolves to the existing alias `dev`
        assert_eq!(
            c.resolve_parent(&class, None, &exists),
            Some(ParentInfo {
                name: "dev".into(),
                source: ParentSource::Type
            })
        );
        // recorded wins when it exists
        assert_eq!(
            c.resolve_parent(&class, Some(("other", ParentSource::Recorded)), &exists),
            Some(ParentInfo {
                name: "other".into(),
                source: ParentSource::Recorded
            })
        );
        // a recorded parent that is gone falls back to the type's
        assert_eq!(
            c.resolve_parent(&class, Some(("gone", ParentSource::Recorded)), &exists)
                .unwrap()
                .source,
            ParentSource::Type
        );
        // a base's parent
        let dev = c.classify("dev");
        assert_eq!(
            c.resolve_parent(&dev, None, &exists),
            Some(ParentInfo {
                name: "master".into(),
                source: ParentSource::Base
            })
        );
        // trunk and unclassified have none
        assert_eq!(c.resolve_parent(&c.classify("master"), None, &exists), None);
        assert_eq!(
            c.resolve_parent(&Classification::Unclassified, None, &exists),
            None
        );
        // type parent that does not exist at all
        assert_eq!(c.resolve_parent(&class, None, &|n| n == "feature/a"), None);
    }
}
