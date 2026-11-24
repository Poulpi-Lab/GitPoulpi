//! The answer of the "Create Rules with AI" action, parsed into the
//! workbench's own types.
//!
//! **Data only.** [`RulesAnswer`] is a scope, some settings and a list of
//! [`Rule`]s: nothing here previews, applies or even validates against the
//! repository. Each rule is parsed and checked on its own (the same
//! `validate_recipe` the editor, the daemon and the CLI run), so one bad rule
//! is reported with its reason while the valid ones are kept.

use gitpoulpi_core::domain::rewrite::{
    validate_recipe, CommitterPolicy, ContentFind, PathMove, PruneEmpty, Recipe, Rule, RuleKind,
    Scope,
};
use serde_json::Value;

use crate::{Error, Result};

/// A rule (or the scope, or a setting) of the answer that was left out, and
/// why.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct RuleProblem {
    /// 1-based position in the answer's `rules`; 0 for the scope or a
    /// setting.
    pub position: usize,
    /// The rule's label, its `type`, or `scope` / `options`.
    pub what: String,
    pub reason: String,
}

/// The run options the answer stated; `None` leaves the workbench's value.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize)]
pub struct RulesSettings {
    pub include_tags: Option<bool>,
    pub committer: Option<CommitterPolicy>,
    pub prune_empty: Option<PruneEmpty>,
    pub update_message_hashes: Option<bool>,
    pub keep_backup_refs: Option<bool>,
}

impl RulesSettings {
    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }
}

/// What the model proposed.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize)]
pub struct RulesAnswer {
    /// Only `current_branch` (no `since`), `all_local` and `refs`.
    pub scope: Option<Scope>,
    pub settings: RulesSettings,
    /// The rules that parsed and validated, in the answer's order.
    pub rules: Vec<Rule>,
    /// What was left out, with the reason.
    pub problems: Vec<RuleProblem>,
    /// The model's remarks (what it could not express, questions).
    pub notes: Vec<String>,
}

/// The JSON object of an answer: the fenced block, else the outermost braces.
pub(crate) fn json_text(answer: &str) -> Option<String> {
    let mut blocks: Vec<String> = Vec::new();
    let mut open: Option<Vec<&str>> = None;
    for line in answer.lines() {
        if line.trim_start().starts_with("```") {
            match open.take() {
                Some(body) => blocks.push(body.join("\n")),
                None => open = Some(Vec::new()),
            }
        } else if let Some(body) = open.as_mut() {
            body.push(line);
        }
    }
    if let Some(body) = open {
        blocks.push(body.join("\n"));
    }
    if let Some(block) = blocks.into_iter().find(|b| b.contains('{')) {
        return Some(block);
    }
    let start = answer.find('{')?;
    let end = answer.rfind('}')?;
    (end > start).then(|| answer[start..=end].to_string())
}

/// Why `kind` may not come from a model, whatever else is true of it.
fn forbidden(kind: &RuleKind) -> Option<&'static str> {
    match kind {
        RuleKind::Script(_) => Some(
            "a script rule runs code: use Write a Script with AI, which shows the script for review",
        ),
        RuleKind::Mailmap { file: Some(_), .. } => Some("a .mailmap file path is not accepted"),
        RuleKind::ContentReplace(c)
            if matches!(c.find, ContentFind::Secrets { .. }) =>
        {
            Some("secret values are never handed to a model: use the Secrets tab")
        }
        RuleKind::MovePaths(PathMove::ExtractSubdir { mode: Some(_), .. }) => {
            Some("the extraction mode is asked at Apply, not set by the model")
        }
        _ => None,
    }
}

fn problem(position: usize, what: &str, reason: impl Into<String>) -> RuleProblem {
    RuleProblem {
        position,
        what: what.to_string(),
        reason: reason.into(),
    }
}

fn parse_scope(value: &Value, problems: &mut Vec<RuleProblem>) -> Option<Scope> {
    let scope: Scope = match serde_json::from_value(value.clone()) {
        Ok(s) => s,
        Err(e) => {
            problems.push(problem(0, "scope", e.to_string()));
            return None;
        }
    };
    match scope {
        Scope::CurrentBranch { since: None } | Scope::AllLocal => Some(scope),
        Scope::Refs { refs } => {
            let refs: Vec<String> = refs
                .into_iter()
                .map(|r| r.trim().to_string())
                .filter(|r| !r.is_empty())
                .collect();
            if refs.is_empty() {
                problems.push(problem(0, "scope", "no branch or tag is named"));
                None
            } else {
                Some(Scope::Refs { refs })
            }
        }
        _ => {
            problems.push(problem(
                0,
                "scope",
                "only the current branch, every local branch or named refs can be proposed",
            ));
            None
        }
    }
}

fn parse_settings(
    scope_options: Option<&Value>,
    options: Option<&Value>,
    problems: &mut Vec<RuleProblem>,
) -> RulesSettings {
    let mut s = RulesSettings::default();
    let mut bad = |what: &str, reason: String| {
        problems.push(problem(0, "options", format!("{what}: {reason}")))
    };
    if let Some(v) = scope_options {
        match v.get("include_tags") {
            None => {}
            Some(Value::Bool(b)) => s.include_tags = Some(*b),
            Some(_) => bad("include_tags", "must be true or false".into()),
        }
    }
    let Some(o) = options else { return s };
    match o.get("committer") {
        None => {}
        Some(v) => match serde_json::from_value::<CommitterPolicy>(v.clone()) {
            Ok(c) => s.committer = Some(c),
            Err(e) => bad("committer", e.to_string()),
        },
    }
    match o.get("prune_empty") {
        None => {}
        Some(v) => match serde_json::from_value::<PruneEmpty>(v.clone()) {
            Ok(c) => s.prune_empty = Some(c),
            Err(e) => bad("prune_empty", e.to_string()),
        },
    }
    for (key, slot) in [
        ("update_message_hashes", &mut s.update_message_hashes),
        ("keep_backup_refs", &mut s.keep_backup_refs),
    ] {
        match o.get(key) {
            None => {}
            Some(Value::Bool(b)) => *slot = Some(*b),
            Some(_) => bad(key, "must be true or false".into()),
        }
    }
    if o.get("signatures").is_some() {
        bad(
            "signatures",
            "ignored: what happens to signatures is asked when you Apply".into(),
        );
    }
    s
}

/// Parses an answer. `Err` only when no JSON object can be read at all (the
/// caller may ask again); anything else becomes a [`RuleProblem`] next to the
/// rules that were kept.
pub fn rewrite_rules(answer: &str) -> Result<RulesAnswer> {
    let Some(text) = json_text(answer) else {
        return Err(if answer.trim().is_empty() {
            Error::Empty
        } else {
            Error::Invalid("the answer has no JSON object".into())
        });
    };
    let root: Value = serde_json::from_str(&text)
        .map_err(|e| Error::Invalid(format!("the answer is not valid JSON: {e}")))?;
    if !root.is_object() {
        return Err(Error::Invalid("the answer is not a JSON object".into()));
    }
    let mut out = RulesAnswer::default();
    if let Some(v) = root.get("scope") {
        out.scope = parse_scope(v, &mut out.problems);
    }
    out.settings = parse_settings(
        root.get("scope_options"),
        root.get("options"),
        &mut out.problems,
    );
    if let Some(Value::Array(notes)) = root.get("notes") {
        out.notes = notes
            .iter()
            .filter_map(|n| n.as_str())
            .map(|n| n.trim().to_string())
            .filter(|n| !n.is_empty())
            .collect();
    }
    let items: &[Value] = match root.get("rules") {
        Some(Value::Array(items)) => items,
        None => &[],
        Some(_) => {
            return Err(Error::Invalid("`rules` must be a list".into()));
        }
    };
    for (i, item) in items.iter().enumerate() {
        let position = i + 1;
        let name = item
            .get("label")
            .and_then(Value::as_str)
            .filter(|l| !l.trim().is_empty())
            .or_else(|| item.get("kind")?.get("type")?.as_str())
            .unwrap_or("rule")
            .to_string();
        let rule: Rule = match serde_json::from_value(item.clone()) {
            Ok(r) => r,
            Err(e) => {
                out.problems.push(problem(position, &name, e.to_string()));
                continue;
            }
        };
        if let Some(why) = forbidden(&rule.kind) {
            out.problems.push(problem(position, &name, why));
            continue;
        }
        let single = Recipe::new("", Scope::AllLocal, vec![rule.clone()]);
        if let Err(issue) = validate_recipe(&single) {
            out.problems.push(problem(position, &name, issue.detail));
            continue;
        }
        out.rules.push(rule);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fenced_answer_with_prose_parses() {
        let a = rewrite_rules(
            "Here you go:\n```json\n{\"rules\":[{\"label\":\"Drop it\",\"kind\":{\"type\":\"remove_paths\",\"globs\":[\"a.txt\"]}}]}\n```\nDone.",
        )
        .unwrap();
        assert_eq!(a.rules.len(), 1);
        assert_eq!(a.rules[0].label.as_deref(), Some("Drop it"));
        assert!(a.problems.is_empty());
    }

    #[test]
    fn an_answer_without_json_is_an_error() {
        assert!(matches!(rewrite_rules("I cannot."), Err(Error::Invalid(_))));
        assert_eq!(rewrite_rules("  "), Err(Error::Empty));
        assert!(rewrite_rules("{not json}").is_err());
    }
}
