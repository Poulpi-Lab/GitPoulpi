//! The "Sequence draft" action: the fixed contract (the answer's JSON shape
//! and the step vocabulary) and the user message (the description, and in
//! Edit mode the sequence to change).
//!
//! **The vocabulary is generated, never written by hand.** Every line comes
//! from `gitpoulpi_config::sequences::STEP_VOCABULARY` (id, meaning, whether
//! the step can pause and whether `never` is refused, option keys and kinds,
//! one example step), so a step added to the config crate reaches the prompt
//! with no edit here, and an excluded (destructive) command can never appear
//! in it. A test parses and validates every generated example.

use gitpoulpi_config::sequences::{
    OptionKind, OptionSpec, PauseMode, SequenceDef, StepKind, MAX_STEPS, STEP_VOCABULARY,
};
use serde_json::{json, Value};

/// The value an example gives an option.
fn example_value(spec: &OptionSpec) -> Value {
    match spec.kind {
        OptionKind::Bool => json!(true),
        OptionKind::Choice(choices) => json!(choices.first().copied().unwrap_or("")),
        OptionKind::Template => match spec.key {
            "message" => json!("Update ${date}"),
            _ => json!("wip-${date}-${n}"),
        },
        OptionKind::Text => match spec.key {
            "label" => json!("Terminal"),
            "profile" => json!("claude"),
            "branch" => json!("main"),
            _ => json!("text"),
        },
    }
}

fn kind_word(kind: OptionKind) -> String {
    match kind {
        OptionKind::Text => "text".into(),
        OptionKind::Template => "text, may use ${branch} ${date} ${time} ${n}".into(),
        OptionKind::Bool => "true or false".into(),
        OptionKind::Choice(choices) => format!("one of {}", choices.join(" | ")),
    }
}

/// One example step of `kind`: its options all given, the pause left out
/// (`default`), so it is valid on its own.
pub fn example_step(kind: &StepKind) -> Value {
    let mut step = json!({ "run": kind.id });
    if !kind.options.is_empty() {
        let options: serde_json::Map<String, Value> = kind
            .options
            .iter()
            .map(|o| (o.key.to_string(), example_value(o)))
            .collect();
        step["options"] = Value::Object(options);
    }
    step
}

/// The vocabulary entry of one step as text: id, meaning, pause rules,
/// options and the example.
fn vocabulary_line(kind: &StepKind) -> String {
    let mut out = format!("- {}: {}.", kind.id, kind.meaning_en.trim_end_matches('.'));
    if kind.pause_forced {
        out.push_str(" It always asks before it runs: never use pause \"never\" on it.");
    } else if kind.pausable {
        out.push_str(" It may stop for the user (pause: default | always | never).");
    } else {
        out.push_str(" It never stops (leave pause out).");
    }
    if !kind.options.is_empty() {
        out.push_str(" Options:");
        for o in kind.options {
            out.push_str(&format!(" {} ({}", o.key, kind_word(o.kind)));
            if o.required_when_never {
                out.push_str(", required when pause is \"never\"");
            }
            out.push_str(");");
        }
    }
    out.push_str(&format!("\n  {}\n", example_step(kind)));
    out
}

/// [`base_prompt`], built once (the prompt table hands out `&'static str`).
pub fn base_prompt_static() -> &'static str {
    static BASE: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    BASE.get_or_init(base_prompt)
}

/// The base prompt: the answer's shape and rules, then the step vocabulary
/// generated from [`STEP_VOCABULARY`].
pub fn base_prompt() -> String {
    let mut out = format!(
        "\
You draft an action sequence for GitPoulpi from the user's description: a named list of \
commands the app runs one after the other when the user triggers it. You only PROPOSE it: the \
user reads, edits and saves it themselves. You never run anything. Answer with one JSON \
object in exactly one ```json code block and nothing else:

{{
  \"name\": \"Backup\",
  \"id\": \"backup\",
  \"description\": \"one line on what it does\",
  \"scope\": \"installation\",
  \"steps\": [ {{\"run\": \"stage_all\"}}, {{\"run\": \"commit\", \"pause\": \"always\"}} ],
  \"shortcut\": \"Ctrl+G b\",
  \"notes\": [\"anything you could not express, or must warn about\"]
}}

name: the title the user sees. id: a short slug (a-z, 0-9, _ and -). description, scope \
(installation | machine | repo; leave it out unless the user said where it applies), shortcut \
(a suggestion such as \"Ctrl+G b\", leave it out when unsure) and notes are optional.

steps: at most {MAX_STEPS}, run in order. Each step is {{\"run\": <command id>, \"pause\": \
<mode>, \"options\": {{...}}}}; `pause` and `options` are optional. Use only the command ids \
listed below, exactly as spelled, with only the options listed for that command. pause \
\"default\" (the default) lets a step that needs input stop on its usual dialog; \"always\" \
stops before the step and asks to continue; \"never\" does not stop, so every option marked \
required must then be given. When the user wants a review or a confirmation, use \"always\". \
Never put a destructive command (discarding, deleting, resetting, removing, rewriting history) \
in a sequence, and never a step that runs another sequence: say in `notes` that you left it \
out. If a request cannot be done with the commands below, say so in `notes` instead of \
inventing a command. Never invent a branch, remote or label the user did not give or the \
context does not list.

Commands (`run` values), each with an example step:
"
    );
    for kind in STEP_VOCABULARY {
        out.push('\n');
        out.push_str(&vocabulary_line(kind));
    }
    out
}

/// What the model may be told about the installation. Empty unless the user
/// allowed it: the description alone is the default.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize)]
pub struct SequenceContext {
    /// Remote names of the repository.
    pub remotes: Vec<String>,
    /// Ids of the AI profiles (for the agent steps' `profile`).
    pub ai_profiles: Vec<String>,
    /// Labels of the Open in entries (for `tab_open_in`'s `label`).
    pub open_in_labels: Vec<String>,
}

impl SequenceContext {
    pub fn is_empty(&self) -> bool {
        self.remotes.is_empty() && self.ai_profiles.is_empty() && self.open_in_labels.is_empty()
    }
}

/// How many names of one kind a request carries at most.
pub const MAX_CONTEXT_ITEMS: usize = 50;

/// `def` as the JSON the model reads in Edit mode: what the user sees and
/// what the answer's shape holds (no owner, no unknown keys).
pub fn sequence_json(def: &SequenceDef) -> Value {
    let steps: Vec<Value> = def
        .steps
        .iter()
        .map(|s| {
            let mut step = json!({ "run": s.run });
            if s.pause != PauseMode::Default {
                step["pause"] = json!(s.pause.as_str());
            }
            if !s.options.is_empty() {
                step["options"] = serde_json::to_value(&s.options).unwrap_or(Value::Null);
            }
            step
        })
        .collect();
    let mut out = json!({
        "name": def.name,
        "id": def.id,
        "scope": def.scope.as_str(),
        "steps": steps,
    });
    if let Some(d) = &def.description {
        out["description"] = json!(d);
    }
    out
}

/// The user message: the description; in Edit mode the current sequence and
/// the instruction to keep what is not mentioned; then the context (only
/// when it is not empty).
pub fn sequence_draft(
    description: &str,
    current: Option<&SequenceDef>,
    context: &SequenceContext,
) -> String {
    let mut out = String::new();
    match current {
        Some(def) => {
            out.push_str(&format!(
                "The sequence to change:\n```json\n{}\n```\n\nHow to change it:\n{}\n\nChange it as \
                 asked; keep what is not mentioned (the id, the name, the scope and every step \
                 the user did not talk about). Answer with the complete changed sequence.\n",
                serde_json::to_string_pretty(&sequence_json(def)).unwrap_or_default(),
                description.trim()
            ));
        }
        None => out.push_str(&format!(
            "The sequence to create:\n{}\n",
            description.trim()
        )),
    }
    if !context.is_empty() {
        out.push_str("\nThe installation (sent because the user allowed it):\n");
        for (title, names) in [
            ("remotes", &context.remotes),
            ("AI profiles", &context.ai_profiles),
            ("Open in labels", &context.open_in_labels),
        ] {
            if names.is_empty() {
                continue;
            }
            out.push_str(&format!("- {title}:\n"));
            for n in names.iter().take(MAX_CONTEXT_ITEMS) {
                out.push_str(&format!("  - {n}\n"));
            }
        }
    }
    out
}

/// The second attempt: the prompt, then why the first answer was rejected.
pub fn sequence_draft_retry(prompt: &str, reason: &str) -> String {
    format!(
        "{prompt}\nYour previous answer was rejected: {reason}\nAnswer again with the complete \
         JSON object in one ```json code block only, using only the listed commands."
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn without_context_the_message_holds_only_the_description() {
        let p = sequence_draft("  back up  ", None, &SequenceContext::default());
        assert_eq!(p, "The sequence to create:\nback up\n");
    }

    #[test]
    fn the_context_lists_names() {
        let c = SequenceContext {
            remotes: vec!["origin".into()],
            ai_profiles: vec![],
            open_in_labels: vec!["Zed".into()],
        };
        let p = sequence_draft("x", None, &c);
        assert!(p.contains("  - origin\n") && p.contains("  - Zed\n"), "{p}");
        assert!(!p.contains("AI profiles"));
    }
}
