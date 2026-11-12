//! The operation context of a request: the label and the main arguments of
//! what the daemon was asked to do, for the error that answers it (see
//! [`gitpoulpi_core::OpContext`]). The arguments are the request's JSON, so
//! the summary lives here and not in the core.
//!
//! Arguments are picked from a short list of keys (a name, a branch, a path,
//! a URL): never a message body, never a value that could be a secret, and the
//! result goes through [`gitpoulpi_core::redact`].

use gitpoulpi_core::domain::OperationKind;
use gitpoulpi_core::op_context::{clip, operation_label};
use gitpoulpi_core::OpContext;
use serde_json::Value;

/// The longest one argument value is shown.
const VALUE_MAX: usize = 60;
/// How many entries of a list argument are named.
const LIST_MAX: usize = 3;
/// The longest an argument summary is.
const ARGS_MAX: usize = 160;

/// The context of `kind` called with `args` on `repo`, as request `request`.
pub fn context_of(
    kind: OperationKind,
    args: &Value,
    repo: Option<&str>,
    request: u64,
) -> OpContext {
    OpContext {
        operation: format!("{kind:?}"),
        label: operation_label(kind),
        args: args_summary(kind, args),
        repo: repo.map(str::to_string),
        frontend: None,
        request,
        job: None,
    }
}

/// Argument keys worth showing, in the order they are shown, with the name
/// they are shown under (`None`: as they are).
const KEYS: &[(&str, Option<&str>)] = &[
    ("remote", None),
    ("name", None),
    ("old_name", None),
    ("new_name", None),
    ("local_name", None),
    ("upstream", None),
    ("branch", None),
    ("tag", None),
    ("path", None),
    ("paths", None),
    ("target", None),
    ("dest", None),
    ("url", None),
    ("host", None),
    ("id", None),
    ("commit", None),
    ("commits", None),
    ("at", None),
    ("rev", None),
    ("revision", None),
    ("refspec", None),
    ("key", None),
];

/// Keys that may hold a secret: never shown, whatever the operation.
const HIDDEN: &[&str] = &[
    "token",
    "password",
    "secret",
    "authorization",
    "credential",
    "credentials",
    "passphrase",
    "value",
    "script",
    "message",
    "body",
];

/// A value of the arguments as a short word, or `None` for what is not worth
/// showing (null, flags, large structures).
fn value_text(value: &Value) -> Option<String> {
    match value {
        Value::String(s) if !s.is_empty() => Some(clip(s.lines().next().unwrap_or(""), VALUE_MAX)),
        Value::Number(n) => Some(n.to_string()),
        Value::Array(items) => {
            let words: Vec<String> = items.iter().filter_map(value_text).collect();
            if words.is_empty() {
                return None;
            }
            let more = words.len().saturating_sub(LIST_MAX);
            let mut text = words
                .iter()
                .take(LIST_MAX)
                .cloned()
                .collect::<Vec<_>>()
                .join(" ");
            if more > 0 {
                text.push_str(&format!(" (+{more})"));
            }
            Some(text)
        }
        // `{"File": "src/a.rs"}`: a tagged value is its content.
        Value::Object(map) if map.len() == 1 => map.values().next().and_then(value_text),
        _ => None,
    }
}

/// The main arguments of `kind` called with `args`, redacted.
pub fn args_summary(kind: OperationKind, args: &Value) -> String {
    let Some(map) = args.as_object() else {
        return String::new();
    };
    let remote_ops = matches!(
        kind,
        OperationKind::RemoteAdd
            | OperationKind::RemoteRemove
            | OperationKind::RemoteSetUrl
            | OperationKind::RemoteSetPushUrl
    );
    let rename_remote = kind == OperationKind::RemoteRename;
    let mut parts: Vec<String> = Vec::new();
    let mut take = |key: &str, shown: &str, value: &Value| {
        if HIDDEN.contains(&key) || parts.len() >= 6 {
            return;
        }
        if let Some(text) = value_text(value) {
            parts.push(format!("{shown}={text}"));
        }
    };
    for (key, _) in KEYS {
        let Some(value) = map.get(*key) else { continue };
        let shown = match *key {
            "name" if remote_ops => "remote",
            "old_name" if rename_remote => "from",
            "new_name" if rename_remote => "to",
            other => other,
        };
        take(key, shown, value);
    }
    // The options of a push or a fetch carry the branch and the remote.
    if let Some(opts) = map.get("opts").and_then(Value::as_object) {
        for key in ["remote", "branch", "refspec"] {
            if !map.contains_key(key) {
                if let Some(value) = opts.get(key) {
                    take(key, key, value);
                }
            }
        }
    }
    clip(&gitpoulpi_core::redact(&parts.join(", ")), ARGS_MAX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn a_remote_operation_is_labelled_and_names_its_remote() {
        let ctx = context_of(
            OperationKind::RemoteRemove,
            &json!({ "name": "cm6" }),
            Some("/data/Projects/demo"),
            12,
        );
        assert_eq!(ctx.label, "Remove remote");
        assert_eq!(ctx.operation, "RemoteRemove");
        assert_eq!(ctx.args, "remote=cm6");
        assert_eq!(ctx.repo_name().as_deref(), Some("demo"));
        assert_eq!(ctx.describe(), "Remove remote (remote=cm6)");
        assert_eq!(ctx.request, 12);
    }

    #[test]
    fn arguments_pick_names_paths_and_nested_options_only() {
        let args = json!({
            "opts": { "branch": "main", "remote": "origin", "force": true },
            "target": { "File": "src/a.rs" },
            "paths": ["a", "b", "c", "d", "e"],
            "message": "a long commit message",
            "token": "ghp_abcdefghijklmnopqrstuvwxyz0123456789",
        });
        let text = args_summary(OperationKind::Push, &args);
        assert!(text.contains("target=src/a.rs"), "{text}");
        assert!(text.contains("paths=a b c (+2)"), "{text}");
        assert!(text.contains("branch=main"), "{text}");
        assert!(text.contains("remote=origin"), "{text}");
        assert!(!text.contains("commit message"), "{text}");
        assert!(!text.contains("ghp_"), "{text}");
        assert!(!text.contains("force"), "{text}");
    }

    #[test]
    fn a_rename_says_from_and_to() {
        let text = args_summary(
            OperationKind::RemoteRename,
            &json!({ "old_name": "up", "new_name": "down" }),
        );
        assert_eq!(text, "from=up, to=down");
    }

    #[test]
    fn credentials_in_a_url_never_reach_the_summary() {
        let text = args_summary(
            OperationKind::RemoteAdd,
            &json!({ "name": "up", "url": "https://me:hunter2@host.test/r.git" }),
        );
        assert!(text.contains("remote=up"), "{text}");
        assert!(text.contains("host.test"), "{text}");
        assert!(!text.contains("hunter2"), "{text}");
    }

    #[test]
    fn secret_looking_keys_are_never_shown() {
        let text = args_summary(
            OperationKind::SetRepoConfig,
            &json!({ "key": "user.name", "value": "hunter2hunter2" }),
        );
        assert_eq!(text, "key=user.name");
    }

    #[test]
    fn the_failure_line_of_a_request_names_its_arguments_and_repository() {
        let mut ctx = context_of(
            OperationKind::RemoteRemove,
            &json!({ "name": "cm6" }),
            Some("/data/Projects/demo"),
            12,
        );
        ctx.frontend = Some("CLI".into());
        assert_eq!(
            ctx.failure_line("backend error: no such remote: cm6"),
            "Remove remote failed (remote=cm6) [op #12, CLI, repo demo]: \
             backend error: no such remote: cm6"
        );
    }
}
