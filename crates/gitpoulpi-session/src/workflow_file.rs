//! Reading and writing a repository's branching workflow: the shared
//! `.gitpoulpi/workflow.toml` (committed by the user; GitPoulpi writes it
//! only on an explicit save) and the per-user override in the repo state.
//!
//! Saving over an existing file keeps keys this version does not know (a
//! newer GitPoulpi's, a teammate's experiment). Comments and key order of a
//! hand-edited file are not kept when such a file is rewritten.

use std::path::{Path, PathBuf};

use gitpoulpi_core::workflow::{Workflow, WorkflowOverride};
use toml::{Table, Value};

use crate::repo_state::ensure_state_gitignore;
use crate::{RepoState, Result};

/// The file name inside the state directory.
pub const WORKFLOW_FILE: &str = "workflow.toml";

/// Where the shared workflow of the repository at `repo_root` lives.
pub fn workflow_path_for(repo_root: &Path, state_dir: &str) -> PathBuf {
    gitpoulpi_config::repo_data_file(repo_root, state_dir, WORKFLOW_FILE)
}

/// The shared file as read: the model plus what the model does not know.
#[derive(Debug, Clone)]
pub struct SharedWorkflow {
    pub workflow: Workflow,
    /// The whole file, for keeping unknown keys on a rewrite.
    table: Table,
    /// Paths of keys this version ignores (`topic[0].colour`).
    pub unknown_keys: Vec<String>,
}

/// What kind of table a level of the file is.
#[derive(Clone, Copy)]
enum Level {
    Top,
    Base,
    Topic,
    Finish,
    Hooks,
    Issue,
}

impl Level {
    fn known(self) -> &'static [&'static str] {
        match self {
            Level::Top => &["template", "base", "topic", "hooks", "issue_flow"],
            Level::Base => &[
                "name",
                "names",
                "parent",
                "protected",
                "merge",
                "tag_on_merge",
                "environment",
            ],
            Level::Topic => &[
                "id",
                "label",
                "pattern",
                "patterns",
                "regex",
                "parent",
                "finish",
                "finish_mode",
                "delete_after_finish",
            ],
            Level::Finish => &["into", "merge", "tag"],
            Level::Hooks => &["after_start", "before_finish"],
            Level::Issue => &[
                "mode",
                "issue_title",
                "issue_labels",
                "branch_from_issue",
                "wait_for",
                "description_template",
                "timeout_secs",
                "merge_request",
                "assignees",
            ],
        }
    }

    /// The level of the tables under `key`, when it holds some.
    fn child(self, key: &str) -> Option<Level> {
        match (self, key) {
            (Level::Top, "base") => Some(Level::Base),
            (Level::Top, "topic") => Some(Level::Topic),
            (Level::Top, "hooks") => Some(Level::Hooks),
            (Level::Top, "issue_flow") => Some(Level::Issue),
            (Level::Topic, "finish") => Some(Level::Finish),
            _ => None,
        }
    }
}

fn collect_unknown(table: &Table, level: Level, path: &str, out: &mut Vec<String>) {
    for (key, value) in table {
        let here = if path.is_empty() {
            key.clone()
        } else {
            format!("{path}.{key}")
        };
        if !level.known().contains(&key.as_str()) {
            out.push(here);
            continue;
        }
        let Some(child) = level.child(key) else {
            continue;
        };
        match value {
            Value::Table(t) => collect_unknown(t, child, &here, out),
            Value::Array(items) => {
                for (i, item) in items.iter().enumerate() {
                    if let Value::Table(t) = item {
                        collect_unknown(t, child, &format!("{here}[{i}]"), out);
                    }
                }
            }
            _ => {}
        }
    }
}

/// `original`'s unknown keys, with `new`'s (the model's) known keys on top.
fn merge(original: Option<&Table>, new: Table, level: Level) -> Table {
    let mut out = Table::new();
    if let Some(original) = original {
        for (key, value) in original {
            if !level.known().contains(&key.as_str()) {
                out.insert(key.clone(), value.clone());
            }
        }
    }
    for (key, value) in new {
        let orig_child = original.and_then(|o| o.get(&key));
        let merged = match (level.child(&key), value) {
            (Some(child), Value::Table(t)) => {
                Value::Table(merge(orig_child.and_then(Value::as_table), t, child))
            }
            (Some(child), Value::Array(items)) => {
                let origs = orig_child.and_then(Value::as_array);
                Value::Array(
                    items
                        .into_iter()
                        .enumerate()
                        .map(|(i, item)| match item {
                            Value::Table(t) => {
                                let o = origs.and_then(|a| a.get(i)).and_then(Value::as_table);
                                Value::Table(merge(o, t, child))
                            }
                            other => other,
                        })
                        .collect(),
                )
            }
            (_, other) => other,
        };
        out.insert(key, merged);
    }
    out
}

impl SharedWorkflow {
    /// Parses the text of a `workflow.toml`.
    pub fn parse(text: &str) -> Result<Self> {
        let table: Table = toml::from_str(text)?;
        let workflow: Workflow = table.clone().try_into()?;
        let mut unknown_keys = Vec::new();
        collect_unknown(&table, Level::Top, "", &mut unknown_keys);
        Ok(Self {
            workflow,
            table,
            unknown_keys,
        })
    }

    /// The file's text for `workflow`, keeping the unknown keys of this one.
    pub fn render(&self, workflow: &Workflow) -> Result<String> {
        render_toml(workflow, Some(self))
    }
}

/// The TOML text of `workflow`: in model order when there is nothing to keep,
/// else merged over `previous`'s unknown keys.
pub fn render_toml(workflow: &Workflow, previous: Option<&SharedWorkflow>) -> Result<String> {
    match previous.filter(|p| !p.unknown_keys.is_empty()) {
        None => Ok(toml::to_string_pretty(workflow)?),
        Some(prev) => {
            let new = Table::try_from(workflow)?;
            Ok(toml::to_string_pretty(&merge(
                Some(&prev.table),
                new,
                Level::Top,
            ))?)
        }
    }
}

/// The shared workflow at `path`, `None` when the file does not exist.
pub fn load_shared(path: &Path) -> Result<Option<SharedWorkflow>> {
    match std::fs::read_to_string(path) {
        Ok(text) => SharedWorkflow::parse(&text).map(Some),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.into()),
    }
}

/// Writes `workflow` to `path` (an explicit save). Returns `false` and writes
/// nothing when the file exists and `overwrite` is not set. Makes sure the
/// state directory's ignore file lets the file be committed.
pub fn save_shared(path: &Path, workflow: &Workflow, overwrite: bool) -> Result<bool> {
    let previous = if path.exists() {
        if !overwrite {
            return Ok(false);
        }
        load_shared(path).ok().flatten()
    } else {
        None
    };
    let text = render_toml(workflow, previous.as_ref())?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
        ensure_state_gitignore(parent)?;
    }
    std::fs::write(path, text)?;
    Ok(true)
}

/// Everything read for one repository.
#[derive(Debug, Clone, Default)]
pub struct LoadedWorkflow {
    pub shared: Option<SharedWorkflow>,
    pub user: Option<WorkflowOverride>,
}

impl LoadedWorkflow {
    /// Reads the shared file and the user override of the repository at
    /// `repo_root`. An unreadable or invalid file is an error (the message
    /// names it), never silently ignored.
    pub fn load(repo_root: &Path, state_dir: &str) -> std::result::Result<Self, String> {
        let shared_path = workflow_path_for(repo_root, state_dir);
        let shared =
            load_shared(&shared_path).map_err(|e| format!("{}: {e}", shared_path.display()))?;
        let state_path = RepoState::path_for(repo_root, state_dir);
        let user = RepoState::load(&state_path)
            .map_err(|e| format!("{}: {e}", state_path.display()))?
            .workflow
            .filter(|w| !w.is_empty());
        Ok(Self { shared, user })
    }

    /// Whether anything is defined.
    pub fn is_configured(&self) -> bool {
        self.shared.is_some() || self.user.is_some()
    }

    /// The shared workflow with the user override on top.
    pub fn effective(&self) -> Workflow {
        let base = self
            .shared
            .as_ref()
            .map(|s| s.workflow.clone())
            .unwrap_or_default();
        match &self.user {
            Some(over) => base.merged_with(over),
            None => base,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitpoulpi_core::workflow::{BaseBranch, TopicType, WorkflowTemplate};

    #[test]
    fn missing_file_is_none_and_invalid_is_an_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = workflow_path_for(dir.path(), ".gitpoulpi");
        assert!(load_shared(&path).unwrap().is_none());
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, "template = [").unwrap();
        assert!(load_shared(&path).is_err());
        let err = LoadedWorkflow::load(dir.path(), ".gitpoulpi").unwrap_err();
        assert!(err.contains("workflow.toml"), "{err}");
        std::fs::write(&path, "[[base]]\nprotected = \"yes\"\n").unwrap();
        assert!(
            load_shared(&path).is_err(),
            "a wrongly typed field is an error"
        );
    }

    #[test]
    fn save_creates_but_never_overwrites_without_the_flag() {
        let dir = tempfile::tempdir().unwrap();
        let path = workflow_path_for(dir.path(), ".gitpoulpi");
        let w = WorkflowTemplate::GitFlow.workflow();
        assert!(save_shared(&path, &w, false).unwrap());
        assert_eq!(load_shared(&path).unwrap().unwrap().workflow, w);
        let other = WorkflowTemplate::Trunk.workflow();
        assert!(!save_shared(&path, &other, false).unwrap());
        assert_eq!(load_shared(&path).unwrap().unwrap().workflow, w);
        assert!(save_shared(&path, &other, true).unwrap());
        assert_eq!(load_shared(&path).unwrap().unwrap().workflow, other);
    }

    #[test]
    fn unknown_keys_survive_a_rewrite_and_dropped_known_keys_do_not() {
        let text = r#"
template = "git-flow"
future_setting = "keep me"

[[base]]
name = "main"
protected = true
colour = "blue"

[[base]]
name = "develop"
parent = "main"

[[topic]]
id = "feature"
pattern = "feature/{slug}"
parent = "develop"
icon = "star"

[experimental]
flag = true
"#;
        let shared = SharedWorkflow::parse(text).unwrap();
        assert_eq!(
            shared.unknown_keys,
            [
                "base[0].colour",
                "experimental",
                "future_setting",
                "topic[0].icon"
            ]
        );
        let mut w = shared.workflow.clone();
        w.bases[0].protected = false; // a known key dropped from the file
        w.topics[0].pattern = "feat/{slug}".into();
        let out = shared.render(&w).unwrap();
        let back = SharedWorkflow::parse(&out).unwrap();
        assert_eq!(back.workflow, w);
        assert_eq!(back.unknown_keys, shared.unknown_keys, "{out}");
        assert!(out.contains("keep me") && out.contains("blue") && out.contains("star"));
        assert!(!out.contains("protected"), "{out}");
    }

    #[test]
    fn without_unknown_keys_the_file_is_written_in_model_order() {
        let w = WorkflowTemplate::GitFlow.workflow();
        let out = render_toml(&w, None).unwrap();
        assert!(out.starts_with("template = \"git-flow\""), "{out}");
        let i = |s: &str| out.find(s).unwrap();
        assert!(i("[[base]]") < i("[[topic]]"));
        let first_base = &out[i("[[base]]")..];
        assert!(first_base.starts_with("[[base]]\nname = \"main\""), "{out}");
    }

    #[test]
    fn known_key_lists_cover_the_model() {
        // A fully populated workflow must not look like it has unknown keys.
        let mut w = WorkflowTemplate::GitFlow.workflow();
        w.bases[0] = BaseBranch {
            name: "main".into(),
            names: vec!["master".into()],
            parent: Some("x".into()),
            protected: true,
            merge: Some(gitpoulpi_core::workflow::MergeStrategy::Squash),
            tag_on_merge: true,
            environment: true,
        };
        w.topics[0] = TopicType {
            id: "f".into(),
            label: "F".into(),
            pattern: "f/{slug}".into(),
            patterns: vec!["g/{slug}".into()],
            regex: Some("^h".into()),
            parent: Some("develop".into()),
            finish: vec![gitpoulpi_core::workflow::FinishTarget {
                into: "develop".into(),
                merge: Some(gitpoulpi_core::workflow::MergeStrategy::Merge),
                tag: Some("t".into()),
            }],
            finish_mode: Some(gitpoulpi_core::workflow::FinishMode::Ask),
            delete_after_finish: true,
        };
        w.hooks.after_start = vec!["a".into()];
        w.hooks.before_finish = vec!["b".into()];
        w.issue_flow = Some(gitpoulpi_core::workflow::IssueFlow {
            mode: gitpoulpi_core::workflow::IssueFlowMode::Bot,
            issue_title: Some("t".into()),
            issue_labels: vec!["l".into()],
            branch_from_issue: Some("b".into()),
            wait_for: Some("w".into()),
            description_template: Some("d.md".into()),
            timeout_secs: Some(30),
            merge_request: Some(false),
            assignees: vec!["a".into()],
        });
        let text = toml::to_string_pretty(&w).unwrap();
        let parsed = SharedWorkflow::parse(&text).unwrap();
        assert!(parsed.unknown_keys.is_empty(), "{:?}", parsed.unknown_keys);
        assert_eq!(parsed.workflow, w);
    }

    #[test]
    fn the_user_override_layers_over_the_shared_file() {
        let dir = tempfile::tempdir().unwrap();
        let state = RepoState::path_for(dir.path(), ".gitpoulpi");
        save_shared(
            &workflow_path_for(dir.path(), ".gitpoulpi"),
            &WorkflowTemplate::GitFlow.workflow(),
            false,
        )
        .unwrap();
        let loaded = LoadedWorkflow::load(dir.path(), ".gitpoulpi").unwrap();
        assert!(loaded.is_configured() && loaded.user.is_none());
        let rs = RepoState {
            workflow: Some(WorkflowOverride {
                disable_topics: vec!["support".into()],
                ..Default::default()
            }),
            ..Default::default()
        };
        rs.save(&state).unwrap();
        let loaded = LoadedWorkflow::load(dir.path(), ".gitpoulpi").unwrap();
        assert!(loaded.user.is_some());
        assert!(loaded.effective().topics.iter().all(|t| t.id != "support"));
        assert_eq!(loaded.effective().topics.len(), 4);
        // no files at all: not configured
        let empty = tempfile::tempdir().unwrap();
        let none = LoadedWorkflow::load(empty.path(), ".gitpoulpi").unwrap();
        assert!(!none.is_configured() && none.effective().is_empty());
    }

    #[test]
    fn the_state_directory_lets_workflow_toml_be_committed() {
        let dir = tempfile::tempdir().unwrap();
        let run = |args: &[&str]| {
            let out = std::process::Command::new("git")
                .args(args)
                .current_dir(dir.path())
                .output()
                .expect("run git");
            assert!(out.status.success(), "git {args:?}");
            String::from_utf8_lossy(&out.stdout).into_owned()
        };
        run(&["init", "-q"]);
        // The state file first (writes the ignore file), the workflow after.
        RepoState::default()
            .save(&RepoState::path_for(dir.path(), ".gitpoulpi"))
            .unwrap();
        save_shared(
            &workflow_path_for(dir.path(), ".gitpoulpi"),
            &WorkflowTemplate::Trunk.workflow(),
            false,
        )
        .unwrap();
        let status = run(&["status", "--porcelain", "--untracked-files=all"]);
        assert!(status.contains(".gitpoulpi/workflow.toml"), "{status}");
        assert!(!status.contains("repo-state.toml"), "{status}");
    }

    #[test]
    fn an_old_ignore_file_gets_the_exception_and_a_users_is_left_alone() {
        let root = tempfile::tempdir().unwrap();
        let dir = root.path().join(".gitpoulpi");
        std::fs::create_dir_all(&dir).unwrap();
        let ignore = dir.join(".gitignore");
        std::fs::write(
            &ignore,
            "# Written by GitPoulpi: per-repository UI state, not project content.\n*\n",
        )
        .unwrap();
        ensure_state_gitignore(&dir).unwrap();
        ensure_state_gitignore(&dir).unwrap();
        let text = std::fs::read_to_string(&ignore).unwrap();
        assert_eq!(text.matches("!workflow.toml").count(), 1, "{text}");
        std::fs::write(&ignore, "mine\n").unwrap();
        ensure_state_gitignore(&dir).unwrap();
        assert_eq!(std::fs::read_to_string(&ignore).unwrap(), "mine\n");
        // Outside the working tree nothing needs hiding.
        let outside = root.path().join("gitpoulpi");
        std::fs::create_dir_all(&outside).unwrap();
        ensure_state_gitignore(&outside).unwrap();
        assert!(!outside.join(".gitignore").exists());
    }
}
