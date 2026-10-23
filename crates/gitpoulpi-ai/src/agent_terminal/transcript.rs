//! Claude Code writes every conversation to a JSON-lines transcript; in
//! terminal mode the answer is read from there (exact text, no screen
//! scraping) and the file is deleted afterwards.
//!
//! Layout (checked against the installed package, see
//! the AI harness reference):
//! `<config dir>/projects/<sanitised working folder>/<session id>.jsonl`,
//! `<config dir>` being `CLAUDE_CONFIG_DIR` or `~/.claude`. Each line is a
//! JSON object; an assistant turn is `{"type":"assistant","message":
//! {"content":[{"type":"text","text":"..."}],"stop_reason":"end_turn"}}`.
//! The folder name is not recomputed here (the sanitising rule has changed
//! between versions): the file is found by its session id.

use std::path::{Path, PathBuf};

use serde_json::Value;

/// `CLAUDE_CONFIG_DIR` from the profile's environment, else the process's,
/// else `~/.claude`.
pub fn config_dir(profile_env: &[(String, String)]) -> Option<PathBuf> {
    let from_profile = profile_env
        .iter()
        .find(|(k, _)| k == "CLAUDE_CONFIG_DIR")
        .map(|(_, v)| v.clone())
        .filter(|v| !v.trim().is_empty());
    if let Some(dir) = from_profile.or_else(|| {
        std::env::var("CLAUDE_CONFIG_DIR")
            .ok()
            .filter(|v| !v.trim().is_empty())
    }) {
        return Some(PathBuf::from(dir));
    }
    let home = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE"))?;
    Some(Path::new(&home).join(".claude"))
}

/// The transcript of session `id` under `config_dir`, if it exists yet.
pub fn find(config_dir: &Path, id: &str) -> Option<PathBuf> {
    let name = format!("{id}.jsonl");
    std::fs::read_dir(config_dir.join("projects"))
        .ok()?
        .filter_map(Result::ok)
        .map(|d| d.path().join(&name))
        .find(|p| p.is_file())
}

/// What the transcript says about the last turn.
#[derive(Debug, PartialEq, Eq)]
pub struct Turn {
    /// The last text block the assistant wrote since the last user message.
    pub text: String,
    /// The turn ended (`stop_reason` is `end_turn`).
    pub complete: bool,
}

/// The last turn of a transcript, `None` while the assistant wrote no text
/// after the last user message.
pub fn last_turn(jsonl: &str) -> Option<Turn> {
    let mut text: Option<String> = None;
    let mut complete = false;
    for line in jsonl.lines() {
        let Ok(entry) = serde_json::from_str::<Value>(line) else {
            // A half-written last line: the next poll has it whole.
            continue;
        };
        match entry.get("type").and_then(Value::as_str) {
            Some("user") if !is_tool_result(&entry) => {
                text = None;
                complete = false;
            }
            Some("assistant") => {
                let message = entry.get("message");
                if let Some(blocks) = message
                    .and_then(|m| m.get("content"))
                    .and_then(Value::as_array)
                {
                    for block in blocks {
                        if block.get("type").and_then(Value::as_str) == Some("text") {
                            if let Some(t) = block.get("text").and_then(Value::as_str) {
                                if !t.trim().is_empty() {
                                    text = Some(t.to_string());
                                }
                            }
                        }
                    }
                }
                if message
                    .and_then(|m| m.get("stop_reason"))
                    .and_then(Value::as_str)
                    == Some("end_turn")
                {
                    complete = true;
                }
            }
            _ => {}
        }
    }
    text.map(|text| Turn { text, complete })
}

fn is_tool_result(entry: &Value) -> bool {
    entry
        .pointer("/message/content")
        .and_then(Value::as_array)
        .is_some_and(|blocks| {
            blocks
                .iter()
                .any(|b| b.get("type").and_then(Value::as_str) == Some("tool_result"))
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = concat!(
        r#"{"type":"user","message":{"role":"user","content":"hello"}}"#,
        "\n",
        r#"{"type":"assistant","message":{"role":"assistant","content":[{"type":"thinking","thinking":"..."}],"stop_reason":null}}"#,
        "\n",
        r#"{"type":"assistant","message":{"role":"assistant","content":[{"type":"text","text":"first"}],"stop_reason":"tool_use"}}"#,
        "\n",
        r#"{"type":"user","message":{"role":"user","content":[{"type":"tool_result","content":"x"}]}}"#,
        "\n",
        r#"{"type":"assistant","message":{"role":"assistant","content":[{"type":"text","text":"<<GP-a>>\nanswer\n<<GP-END-a>>"}],"stop_reason":"end_turn"}}"#,
        "\n",
    );

    #[test]
    fn the_last_text_of_a_finished_turn_is_the_answer() {
        let turn = last_turn(SAMPLE).unwrap();
        assert!(turn.complete);
        assert_eq!(turn.text, "<<GP-a>>\nanswer\n<<GP-END-a>>");
    }

    #[test]
    fn a_turn_still_running_is_not_complete_and_a_cut_line_is_skipped() {
        let running = SAMPLE.lines().take(3).collect::<Vec<_>>().join("\n") + "\n{\"type\":\"assi";
        let turn = last_turn(&running).unwrap();
        assert!(!turn.complete);
        assert_eq!(turn.text, "first");
        assert_eq!(
            last_turn(r#"{"type":"user","message":{"content":"hi"}}"#),
            None
        );
    }

    #[test]
    fn a_new_user_message_forgets_the_previous_turn() {
        let two = format!(
            "{SAMPLE}{}\n",
            r#"{"type":"user","message":{"role":"user","content":"again"}}"#
        );
        assert_eq!(last_turn(&two), None);
    }

    #[test]
    fn the_file_is_found_by_session_id_under_any_project_folder() {
        let dir = tempfile::tempdir().unwrap();
        let project = dir.path().join("projects").join("-tmp-some-folder");
        std::fs::create_dir_all(&project).unwrap();
        std::fs::write(project.join("abc.jsonl"), "{}").unwrap();
        assert_eq!(find(dir.path(), "abc"), Some(project.join("abc.jsonl")));
        assert_eq!(find(dir.path(), "zzz"), None);
    }

    #[test]
    fn the_profiles_config_dir_wins() {
        let env = vec![("CLAUDE_CONFIG_DIR".to_string(), "/x/claude".to_string())];
        assert_eq!(config_dir(&env), Some(PathBuf::from("/x/claude")));
    }
}
