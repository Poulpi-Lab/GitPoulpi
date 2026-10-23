//! The prompts. The system prompt of an action is its built-in base prompt
//! ([`base_prompt`], not user-visible: it fixes the answer format the
//! parsers rely on) followed by the user's instructions, layered from the
//! most general to the most specific: global, workspace, repository, then
//! the action's own ([`system_prompt_layered`]); the user-message builders below carry
//! everything the model must look at. Pure and unit tested.

use gitpoulpi_config::{AiAction, RepoAiInstructions, RepoInstructionsMode};

mod review;
pub use review::{code_review, BuiltReviewPrompt, ReviewPrompt, ReviewThreadBrief, MAX_FILE_BYTES};

mod conflict;
pub use conflict::{
    conflict_resolution, conflict_resolve, language_of, ConflictContext, CONTEXT_LINES,
    MAX_ANSWER_BYTES, SIDE_BUDGET,
};

mod rewrite_script;
pub use rewrite_script::{
    rewrite_script, rewrite_script_retry, SampleCommit, BASE_PROMPT as REWRITE_SCRIPT_BASE_PROMPT,
    MAX_SAMPLE,
};

mod rewrite_rules;
pub use rewrite_rules::{
    base_prompt as rewrite_rules_base_prompt, rewrite_rules, rewrite_rules_retry, RulesContext,
    VocabEntry, MAX_CONTEXT_ITEMS, VOCABULARY as REWRITE_RULES_VOCABULARY,
};

/// The built-in base prompt of an action.
pub fn base_prompt(action: AiAction) -> &'static str {
    match action {
        AiAction::CommitMessage => {
            "You write git commit messages. Given a diff, answer with a concise summary \
             line of at most 72 characters in the imperative mood, then, only if the \
             change deserves it, a blank line and a short description explaining what \
             changed and why. Answer with the commit message only: no quotes, no code \
             fences, no preamble."
        }
        AiAction::ExplainChanges => {
            "You explain code changes to a colleague. Given a diff, describe in plain \
             language what changed, why it likely changed, and anything a reviewer \
             should double-check. Be concise; use short paragraphs or a short list."
        }
        AiAction::StashMessage => {
            "You name git stashes. Given a diff, answer with a single short line \
             (at most 60 characters) describing the work in progress. Answer with the \
             line only: no quotes, no preamble."
        }
        AiAction::PullRequestDescription => {
            "You write pull request descriptions. Given the commits and the diff of a \
             branch, answer with a title on the first line (at most 72 characters), a \
             blank line, then a markdown description: a short summary, then a bullet \
             list of the notable changes. No preamble."
        }
        AiAction::CommitComposer => {
            "You split a set of working-tree changes into logical, atomic commits. \
             Answer with a JSON array only, no prose and no code fences. Each element \
             is {\"message\": \"<commit message>\", \"files\": [\"<path>\", ...]}. Every \
             changed file must appear in exactly one commit, using the exact paths \
             given. Order the commits so each one builds on the previous."
        }
        AiAction::ConflictResolve => {
            "You resolve one git merge conflict. You get the file path and language, the \
             common ancestor (base) and the two sides' versions of the conflicting region, \
             and some already-merged context around it. Answer with the resolved text that \
             replaces the whole conflicting region, in exactly one code block, and nothing \
             else: no explanation, no conflict markers. Keep the file's indentation and \
             style. When the two changes are independent, keep both; when they contradict, \
             prefer the intent that fits the surrounding code. An empty code block means the \
             region should be removed."
        }
        AiAction::CodeReview => review::BASE_PROMPT,
        AiAction::RewriteScript => REWRITE_SCRIPT_BASE_PROMPT,
        AiAction::RewriteRules => rewrite_rules::base_prompt_static(),
        AiAction::SplitCommit => {
            "You split one git commit into several smaller, logical commits. You get the \
             original message and the changes as numbered pieces: H<n> is one hunk, F<n> a \
             whole file that cannot be split. Answer with a JSON array only, no prose, no \
             code fences. Each element is {\"message\": \"<commit message>\", \"pieces\": \
             [\"H1\", \"F2\", ...]}. Every piece must appear in exactly one commit. Use at \
             least two commits, order them so each builds on the previous ones, keep a hunk \
             with the code it depends on, follow the original message's style."
        }
    }
}

/// The system prompt sent for `action`: the base prompt, then the global
/// instructions, then the action's instructions, each under its own
/// heading and separated by a blank line. A blank part (empty or only
/// whitespace) is skipped, heading included.
pub fn system_prompt(action: AiAction, global: &str, per_action: &str) -> String {
    system_prompt_for_repo(action, global, &RepoAiInstructions::default(), per_action)
}

/// [`system_prompt`] with a repository's own instructions: in `Add` mode
/// a `## Repository instructions` part follows the global one; in
/// `Replace` mode it takes the global part's place (and the global text
/// is dropped). Empty repository text changes nothing in either mode.
pub fn system_prompt_for_repo(
    action: AiAction,
    global: &str,
    repo: &RepoAiInstructions,
    per_action: &str,
) -> String {
    system_prompt_layered(
        action,
        global,
        &RepoAiInstructions::default(),
        repo,
        per_action,
    )
}

/// The instruction parts that apply to every action, in the order they are
/// sent: global, workspace, repository. A level in `Replace` mode with text
/// drops every level above it (the more general ones); a blank level is
/// skipped in either mode.
pub fn general_layers(
    global: &str,
    workspace: &RepoAiInstructions,
    repo: &RepoAiInstructions,
) -> Vec<(&'static str, String)> {
    let mut out: Vec<(&'static str, String)> = Vec::new();
    for (heading, text, mode) in [
        ("Global instructions", global, RepoInstructionsMode::Add),
        (
            "Workspace instructions",
            workspace.text.as_str(),
            workspace.mode,
        ),
        ("Repository instructions", repo.text.as_str(), repo.mode),
    ] {
        let text = text.trim();
        if text.is_empty() {
            continue;
        }
        if mode == RepoInstructionsMode::Replace {
            out.clear();
        }
        out.push((heading, text.to_string()));
    }
    out
}

/// The action's own instructions once the workspace's are in: the global
/// action instructions followed by the workspace's (`Add`), or the
/// workspace's alone (`Replace`). `global_task` that is already the
/// repository's override (`from_repo`) is the most specific word and is
/// kept as it is.
pub fn task_layers(global_task: &str, workspace: &RepoAiInstructions, from_repo: bool) -> String {
    let (global_task, ws) = (global_task.trim(), workspace.text.trim());
    if from_repo || ws.is_empty() {
        return global_task.to_string();
    }
    match workspace.mode {
        RepoInstructionsMode::Replace => ws.to_string(),
        RepoInstructionsMode::Add if global_task.is_empty() => ws.to_string(),
        RepoInstructionsMode::Add => format!("{global_task}\n\n{ws}"),
    }
}

/// The full system prompt: the base prompt of `action`, then
/// [`general_layers`], then `per_action` under "Instructions for this
/// task" (already layered by [`task_layers`] when a workspace is involved).
/// A blank part is skipped, heading included.
pub fn system_prompt_layered(
    action: AiAction,
    global: &str,
    workspace: &RepoAiInstructions,
    repo: &RepoAiInstructions,
    per_action: &str,
) -> String {
    let mut out = base_prompt(action).to_string();
    let mut parts = general_layers(global, workspace, repo);
    parts.push(("Instructions for this task", per_action.trim().to_string()));
    for (heading, text) in parts {
        if !text.is_empty() {
            out.push_str(&format!("\n\n## {heading}\n{text}"));
        }
    }
    out
}

/// The trailing line that makes a prompt answer in `language` (its English
/// name, as in "French"). Only [`AiAction::ExplainChanges`] takes one: the
/// answer is read in the TUI by the person who asked. Everything that lands
/// in the repository or on the host (commit and stash messages, pull request
/// text, a composer plan) keeps the language of the user's instructions.
/// `None` language or a blank one gives no line.
pub fn answer_language_line(action: AiAction, language: Option<&str>) -> Option<String> {
    let language = language?.trim();
    if language.is_empty() || action != AiAction::ExplainChanges {
        return None;
    }
    Some(format!("Answer in {language}."))
}

/// Default budget for the diff part of a prompt, in bytes. Big enough for
/// a substantial change, small enough for every provider's context window
/// and for a local agent's patience.
pub const DEFAULT_DIFF_BUDGET: usize = 60_000;

/// Cuts `diff` to at most `budget` bytes on a char boundary, preferring a
/// line boundary, and says so at the end so the model knows it is not
/// seeing everything. Returns the text and whether it was cut.
pub fn truncate_diff(diff: &str, budget: usize) -> (String, bool) {
    if diff.len() <= budget {
        return (diff.to_string(), false);
    }
    let mut end = budget;
    while !diff.is_char_boundary(end) {
        end -= 1;
    }
    // Back up to the previous newline if that does not throw away most
    // of the budget.
    if let Some(nl) = diff[..end].rfind('\n') {
        if nl > end / 2 {
            end = nl;
        }
    }
    let shown = &diff[..end];
    (
        format!(
            "{shown}\n\n[diff truncated: showing {} of {} bytes]",
            shown.len(),
            diff.len()
        ),
        true,
    )
}

/// The user message of "commit message": the diff of the changes to be
/// committed.
pub fn commit_message(diff: &str, budget: usize) -> String {
    let (diff, _) = truncate_diff(diff, budget);
    format!("Write the commit message for this change.\n\n```diff\n{diff}\n```")
}

/// "Explain changes": `subject` says what the diff is ("commit abc123 by
/// X", "the working tree changes").
pub fn explain_changes(subject: &str, message: Option<&str>, diff: &str, budget: usize) -> String {
    let (diff, _) = truncate_diff(diff, budget);
    let mut out = format!("Explain {subject}.\n\n");
    if let Some(message) = message.filter(|m| !m.trim().is_empty()) {
        out.push_str(&format!("Its commit message:\n{}\n\n", message.trim()));
    }
    out.push_str(&format!("```diff\n{diff}\n```"));
    out
}

pub fn stash_message(diff: &str, budget: usize) -> String {
    let (diff, _) = truncate_diff(diff, budget);
    format!("Name the stash holding this work in progress.\n\n```diff\n{diff}\n```")
}

/// "PR description": the branch's commit subjects and its diff.
pub fn pull_request(commits: &[String], diff: &str, budget: usize) -> String {
    let (diff, _) = truncate_diff(diff, budget);
    let mut out =
        String::from("Write the title and description of this pull request.\n\nCommits:\n");
    for commit in commits {
        out.push_str(&format!("- {commit}\n"));
    }
    out.push_str(&format!("\n```diff\n{diff}\n```"));
    out
}

/// "Commit composer": the exact paths the plan must cover, then the diff.
pub fn commit_composer(files: &[String], diff: &str, budget: usize) -> String {
    let (diff, _) = truncate_diff(diff, budget);
    let mut out = String::from(
        "Split these changes into logical commits. Changed files (use these exact paths, \
         each in exactly one commit):\n",
    );
    for file in files {
        out.push_str(&format!("- {file}\n"));
    }
    out.push_str(&format!("\n```diff\n{diff}\n```"));
    out
}

/// One numbered piece of a commit to split, as the prompt shows it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SplitPromptPiece {
    /// The id the model answers with: `H<n>` (a hunk) or `F<n>` (a whole file).
    pub id: String,
    /// The `###` header text after the id, e.g. `src/parse.rs hunk 1/2 @@ -10,6 +10,14 @@`.
    pub header: String,
    /// The hunk body (`+`/`-`/space lines); `None` for a whole-file piece.
    pub diff: Option<String>,
}

/// "Split commit": the original message, then every piece under a `###`
/// header. Once `budget` bytes of hunk bodies are used the remaining pieces
/// keep their header with "(diff omitted)", so they can still be assigned.
pub fn split_commit(message: &str, pieces: &[SplitPromptPiece], budget: usize) -> String {
    let mut out = format!(
        "Original commit message:\n{}\n\nPieces:\n",
        message.trim_end()
    );
    let mut left = budget;
    for piece in pieces {
        out.push_str(&format!("### {} {}", piece.id, piece.header));
        match &piece.diff {
            None => out.push('\n'),
            Some(_) if left == 0 => out.push_str(" (diff omitted)\n"),
            Some(diff) => {
                let (shown, _) = truncate_diff(diff, left);
                left = left.saturating_sub(diff.len());
                out.push_str(&format!("\n```diff\n{}\n```\n", shown.trim_end()));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn piece(id: &str, diff: Option<&str>) -> SplitPromptPiece {
        SplitPromptPiece {
            id: id.into(),
            header: "a.rs hunk 1/1 @@ -1 +1 @@".into(),
            diff: diff.map(String::from),
        }
    }

    #[test]
    fn the_split_prompt_lists_pieces_and_omits_diffs_over_budget() {
        let mut file = piece("F3", None);
        file.header = "logo.png (binary, whole file)".into();
        let pieces = [piece("H1", Some("+aaaa")), piece("H2", Some("+bbbb")), file];
        let p = split_commit("feat: x\n", &pieces, 5);
        assert!(
            p.starts_with("Original commit message:\nfeat: x\n\nPieces:\n"),
            "{p}"
        );
        assert!(
            p.contains("### H1 a.rs hunk 1/1 @@ -1 +1 @@\n```diff\n+aaaa\n```"),
            "{p}"
        );
        assert!(
            p.contains("### H2 a.rs hunk 1/1 @@ -1 +1 @@ (diff omitted)"),
            "{p}"
        );
        assert!(p.contains("### F3 logo.png (binary, whole file)\n"), "{p}");
        assert!(!p.contains("+bbbb"), "{p}");
    }

    #[test]
    fn only_explain_gets_an_answer_language_line() {
        assert_eq!(
            answer_language_line(AiAction::ExplainChanges, Some("French")).as_deref(),
            Some("Answer in French.")
        );
        assert_eq!(answer_language_line(AiAction::ExplainChanges, None), None);
        assert_eq!(
            answer_language_line(AiAction::ExplainChanges, Some("  ")),
            None
        );
        for action in AiAction::ALL {
            if action != AiAction::ExplainChanges {
                assert_eq!(answer_language_line(action, Some("French")), None);
            }
        }
    }

    #[test]
    fn every_action_ships_a_base_prompt() {
        for action in AiAction::ALL {
            assert!(!base_prompt(action).is_empty());
        }
    }

    #[test]
    fn the_system_prompt_is_base_then_global_then_action() {
        let text = system_prompt(AiAction::StashMessage, "Be kind.", "Max 3 words.");
        let global = text.find("Be kind.").unwrap();
        let action = text.find("Max 3 words.").unwrap();
        assert!(text.starts_with(base_prompt(AiAction::StashMessage)));
        assert!(global < action, "{text}");
        assert!(
            text.contains("\n\n## Global instructions\nBe kind."),
            "{text}"
        );
        assert!(
            text.contains("\n\n## Instructions for this task\nMax 3 words."),
            "{text}"
        );
    }

    #[test]
    fn repo_instructions_add_after_global_or_replace_it() {
        let a = AiAction::CommitMessage;
        let repo = |text: &str, mode| RepoAiInstructions {
            text: text.into(),
            mode,
        };
        let add = system_prompt_for_repo(
            a,
            "Be kind.",
            &repo("Repo rule.", RepoInstructionsMode::Add),
            "Task.",
        );
        let g = add.find("## Global instructions\nBe kind.").unwrap();
        let r = add.find("## Repository instructions\nRepo rule.").unwrap();
        let t = add.find("## Instructions for this task\nTask.").unwrap();
        assert!(g < r && r < t, "{add}");
        let rep = system_prompt_for_repo(
            a,
            "Be kind.",
            &repo("Repo rule.", RepoInstructionsMode::Replace),
            "Task.",
        );
        assert!(!rep.contains("Be kind.") && !rep.contains("Global instructions"));
        assert!(rep.contains("## Repository instructions\nRepo rule."));
        assert!(rep.contains("## Instructions for this task\nTask."));
        for mode in RepoInstructionsMode::ALL {
            let empty = system_prompt_for_repo(a, "Be kind.", &repo("  ", mode), "");
            assert_eq!(empty, system_prompt(a, "Be kind.", ""));
        }
    }

    fn ins(text: &str, mode: RepoInstructionsMode) -> RepoAiInstructions {
        RepoAiInstructions {
            text: text.into(),
            mode,
        }
    }

    /// Every combination of add / replace across the workspace and
    /// repository levels, with every level set.
    #[test]
    fn layering_covers_every_add_replace_combination() {
        use RepoInstructionsMode::{Add, Replace};
        let a = AiAction::CodeReview;
        for (ws_mode, repo_mode, want) in [
            (Add, Add, [true, true, true]),
            (Add, Replace, [false, false, true]),
            (Replace, Add, [false, true, true]),
            (Replace, Replace, [false, false, true]),
        ] {
            let text = system_prompt_layered(
                a,
                "Gtext",
                &ins("Wtext", ws_mode),
                &ins("Rtext", repo_mode),
                "T",
            );
            let heads = [
                "## Global instructions\nGtext",
                "## Workspace instructions\nWtext",
                "## Repository instructions\nRtext",
            ];
            for (head, kept) in heads.iter().zip(want) {
                assert_eq!(
                    text.contains(head),
                    kept,
                    "{ws_mode:?}/{repo_mode:?}: {head}\n{text}"
                );
            }
            // The headings that are there follow the order global, workspace, repository.
            let at: Vec<usize> = heads.iter().filter_map(|h| text.find(h)).collect();
            assert!(at.windows(2).all(|w| w[0] < w[1]), "{text}");
            // The task part is always last and never dropped.
            assert!(text.ends_with("## Instructions for this task\nT"), "{text}");
            assert!(text.starts_with(base_prompt(a)));
        }
    }

    #[test]
    fn a_blank_level_never_replaces_anything() {
        use RepoInstructionsMode::{Add, Replace};
        let text = system_prompt_layered(
            AiAction::CommitMessage,
            "G",
            &ins("  ", Replace),
            &ins("", Replace),
            "",
        );
        assert_eq!(text, system_prompt(AiAction::CommitMessage, "G", ""));
        let ws_only = system_prompt_layered(
            AiAction::CommitMessage,
            "Gtext",
            &ins("W", Replace),
            &ins("", Add),
            "",
        );
        assert!(!ws_only.contains("Gtext") && ws_only.ends_with("## Workspace instructions\nW"));
        // Without a workspace the layered prompt is the old repository one.
        let repo = ins("R", Replace);
        assert_eq!(
            system_prompt_layered(
                AiAction::StashMessage,
                "G",
                &RepoAiInstructions::default(),
                &repo,
                "T"
            ),
            system_prompt_for_repo(AiAction::StashMessage, "G", &repo, "T")
        );
    }

    #[test]
    fn the_action_instructions_layer_global_then_workspace_unless_the_repo_overrode() {
        use RepoInstructionsMode::{Add, Replace};
        assert_eq!(task_layers("G", &ins("W", Add), false), "G\n\nW");
        assert_eq!(task_layers("G", &ins("W", Replace), false), "W");
        assert_eq!(task_layers("", &ins("W", Add), false), "W");
        assert_eq!(task_layers("G", &ins("", Replace), false), "G");
        assert_eq!(task_layers("G", &ins("  ", Add), false), "G");
        // The repository's override is the most specific word.
        assert_eq!(task_layers("R", &ins("W", Replace), true), "R");
        assert_eq!(task_layers("R", &ins("W", Add), true), "R");
    }

    #[test]
    fn empty_and_blank_parts_are_skipped_with_their_heading() {
        let base = base_prompt(AiAction::CommitMessage);
        assert_eq!(system_prompt(AiAction::CommitMessage, "", ""), base);
        assert_eq!(system_prompt(AiAction::CommitMessage, "  \n", " "), base);
        let only_action = system_prompt(AiAction::CommitMessage, "", "Use gitmoji.");
        assert!(
            !only_action.contains("Global instructions"),
            "{only_action}"
        );
        assert!(only_action.ends_with("## Instructions for this task\nUse gitmoji."));
        let only_global = system_prompt(AiAction::CommitMessage, " Be kind. ", "");
        assert!(only_global.ends_with("## Global instructions\nBe kind."));
        assert!(!only_global.contains("for this task"), "{only_global}");
    }

    #[test]
    fn a_small_diff_is_left_alone() {
        assert_eq!(truncate_diff("abc", 10), ("abc".into(), false));
    }

    #[test]
    fn a_big_diff_is_cut_on_a_line_and_says_so() {
        let diff = "line one\nline two\nline three\nline four\n";
        let (out, cut) = truncate_diff(diff, 25);
        assert!(cut);
        assert!(out.starts_with("line one\nline two"), "{out}");
        assert!(!out.contains("line four"), "{out}");
        assert!(out.contains("[diff truncated"), "{out}");
    }

    #[test]
    fn truncation_never_splits_a_multibyte_char() {
        let diff = "é".repeat(100);
        let (out, cut) = truncate_diff(&diff, 51);
        assert!(cut);
        assert!(out.is_char_boundary(out.len()));
    }

    #[test]
    fn every_prompt_carries_the_diff() {
        assert!(commit_message("+x", 100).contains("+x"));
        assert!(explain_changes("commit abc", Some("msg"), "+x", 100).contains("msg"));
        assert!(stash_message("+x", 100).contains("+x"));
        assert!(pull_request(&["a".into()], "+x", 100).contains("- a"));
        let p = commit_composer(&["src/a.rs".into()], "+x", 100);
        assert!(p.contains("- src/a.rs") && p.contains("+x"));
    }
}
