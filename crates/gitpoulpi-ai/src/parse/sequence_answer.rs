//! The answer of the "Sequence draft" action, parsed into the config crate's
//! own [`SequenceDef`].
//!
//! **Data only.** A [`SequenceProposal`] is a definition for the user to read
//! and save; nothing here saves or runs it. Each step is validated on its own
//! (the very `validate` the editor, the daemon and the CLI run, on a
//! one-step sequence), so one bad step is reported with its reason while the
//! valid ones are kept.

use gitpoulpi_config::sequences::{
    is_valid_id, validate, PauseMode, Scope, SequenceDef, StepSpec, MAX_ID_LEN, MAX_STEPS,
};
use serde_json::Value;

use crate::rules_answer::json_text;
use crate::{Error, Result};

/// A step (or the scope, or a field) of the answer that was left out, and
/// why.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct StepProblem {
    /// 1-based position in the answer's `steps`; 0 for something that is not
    /// a step (`scope`, `shortcut`, `steps`).
    pub position: usize,
    /// The step's `run`, or the field's name.
    pub what: String,
    pub reason: String,
}

/// What the model proposed.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct SequenceProposal {
    /// The sequence: only steps that passed. Scope `installation` unless the
    /// answer named another; no owner (the saver sets it).
    pub def: SequenceDef,
    /// What was left out, with the reason.
    pub problems: Vec<StepProblem>,
    /// The model's remarks (what it could not express, questions).
    pub notes: Vec<String>,
    /// A key the model suggests, as text ("Ctrl+G b"); never bound by this
    /// crate.
    pub suggested_shortcut: Option<String>,
}

fn problem(position: usize, what: &str, reason: impl Into<String>) -> StepProblem {
    StepProblem {
        position,
        what: what.to_string(),
        reason: reason.into(),
    }
}

/// A usable slug from `text`: lower case, anything else becomes `-`, runs
/// collapse, ends trimmed, at most [`MAX_ID_LEN`]. `None` when nothing is
/// left.
fn slug(text: &str) -> Option<String> {
    let mut out = String::new();
    for c in text.trim().chars().flat_map(char::to_lowercase) {
        if c.is_ascii_alphanumeric() {
            out.push(c);
        } else if !out.ends_with('-') {
            out.push('-');
        }
    }
    let out = out.trim_matches(|c| c == '-' || c == '_');
    let out: String = out.chars().take(MAX_ID_LEN).collect();
    let out = out.trim_end_matches('-').to_string();
    (!out.is_empty() && is_valid_id(&out)).then_some(out)
}

/// `base` made unique against `taken` with a numeric suffix, within
/// [`MAX_ID_LEN`].
fn unique(base: String, taken: &[String]) -> String {
    if !taken.contains(&base) {
        return base;
    }
    for n in 2..10_000 {
        let suffix = format!("-{n}");
        let keep: String = base.chars().take(MAX_ID_LEN - suffix.len()).collect();
        let candidate = format!("{}{suffix}", keep.trim_end_matches('-'));
        if !taken.contains(&candidate) {
            return candidate;
        }
    }
    base
}

fn text_field(root: &Value, key: &str) -> Option<String> {
    root.get(key)
        .and_then(Value::as_str)
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// JSON options to a TOML table; `null` entries are dropped (TOML has none).
fn options_table(value: &Value) -> std::result::Result<toml::Table, String> {
    let Value::Object(map) = value else {
        return Err("`options` must be an object".into());
    };
    let mut table = toml::Table::new();
    for (key, v) in map {
        if v.is_null() {
            continue;
        }
        let v = toml::Value::try_from(v)
            .map_err(|e| format!("option \"{key}\" cannot be stored: {e}"))?;
        table.insert(key.clone(), v);
    }
    Ok(table)
}

fn parse_step(item: &Value) -> std::result::Result<StepSpec, (String, String)> {
    let run = item
        .get("run")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|r| !r.is_empty());
    let Some(run) = run else {
        return Err(("step".into(), "the step has no `run`".into()));
    };
    let mut step = StepSpec::new(run);
    let fail = |reason: String| (run.to_string(), reason);
    match item.get("pause") {
        None | Some(Value::Null) => {}
        Some(Value::String(s)) => {
            step.pause = match s.trim() {
                "default" => PauseMode::Default,
                "always" => PauseMode::Always,
                "never" => PauseMode::Never,
                other => {
                    return Err(fail(format!(
                        "pause \"{other}\" is not one of default, always, never"
                    )))
                }
            };
        }
        Some(_) => return Err(fail("`pause` must be a word".into())),
    }
    match item.get("options") {
        None | Some(Value::Null) => {}
        Some(v) => step.options = options_table(v).map_err(fail)?,
    }
    Ok(step)
}

/// The problems of `step` alone, as one sentence; `None` when it is fine.
fn alone(step: &StepSpec) -> Option<String> {
    let single = SequenceDef {
        id: "x".into(),
        name: "x".into(),
        steps: vec![step.clone()],
        ..SequenceDef::default()
    };
    let problems = validate(&single);
    (!problems.is_empty()).then(|| {
        problems
            .into_iter()
            .map(|p| p.reason)
            .collect::<Vec<_>>()
            .join("; ")
    })
}

/// Parses an answer. `taken` are the ids already in use: the proposed id is
/// made different from all of them. `Err` only when no sequence can be read
/// at all (the caller may ask again); anything else becomes a [`StepProblem`]
/// next to the steps that were kept.
pub fn sequence_draft(answer: &str, taken: &[String]) -> Result<SequenceProposal> {
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
    let Some(Value::Array(items)) = root.get("steps") else {
        return Err(Error::Invalid(
            "the answer has no `steps` list: give `steps`, a list of {\"run\": ...}".into(),
        ));
    };
    let mut problems = Vec::new();

    let name = text_field(&root, "name");
    let proposed_id = text_field(&root, "id");
    let id = proposed_id
        .as_deref()
        .and_then(slug)
        .or_else(|| name.as_deref().and_then(slug))
        .unwrap_or_else(|| "sequence".into());
    let id = unique(id, taken);
    let name = name
        .or_else(|| proposed_id.clone())
        .unwrap_or_else(|| id.clone());

    let mut def = SequenceDef {
        id,
        name,
        description: text_field(&root, "description"),
        ..SequenceDef::default()
    };
    match root.get("scope") {
        None | Some(Value::Null) => {}
        Some(Value::String(s)) => match s.trim() {
            "installation" => def.scope = Scope::Installation,
            "machine" => def.scope = Scope::Machine,
            "repo" => def.scope = Scope::Repo,
            other => problems.push(problem(
                0,
                "scope",
                format!("\"{other}\" is not installation, machine or repo: kept installation"),
            )),
        },
        Some(_) => problems.push(problem(0, "scope", "`scope` must be a word")),
    }

    for (i, item) in items.iter().enumerate() {
        let position = i + 1;
        let step = match parse_step(item) {
            Ok(step) => step,
            Err((what, reason)) => {
                problems.push(problem(position, &what, reason));
                continue;
            }
        };
        if let Some(reason) = alone(&step) {
            problems.push(problem(position, &step.run, reason));
            continue;
        }
        if def.steps.len() >= MAX_STEPS {
            problems.push(problem(
                position,
                &step.run,
                format!("a sequence has at most {MAX_STEPS} steps"),
            ));
            continue;
        }
        def.steps.push(step);
    }

    let notes = match root.get("notes") {
        Some(Value::Array(notes)) => notes
            .iter()
            .filter_map(Value::as_str)
            .map(|n| n.trim().to_string())
            .filter(|n| !n.is_empty())
            .collect(),
        Some(Value::String(n)) if !n.trim().is_empty() => vec![n.trim().to_string()],
        _ => Vec::new(),
    };
    let suggested_shortcut = text_field(&root, "shortcut");
    Ok(SequenceProposal {
        def,
        problems,
        notes,
        suggested_shortcut,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugs_are_cleaned_and_made_unique() {
        assert_eq!(slug("  My Backup!! ").as_deref(), Some("my-backup"));
        assert_eq!(slug("---"), None);
        assert_eq!(
            unique("backup".into(), &["backup".into(), "backup-2".into()]),
            "backup-3"
        );
    }

    #[test]
    fn a_fenced_answer_with_prose_parses() {
        let p = sequence_draft(
            "Sure:\n```json\n{\"name\":\"B\",\"id\":\"b\",\"steps\":[{\"run\":\"stage_all\"}]}\n```\nok",
            &[],
        )
        .unwrap();
        assert_eq!(p.def.steps.len(), 1);
        assert!(p.problems.is_empty());
    }
}
