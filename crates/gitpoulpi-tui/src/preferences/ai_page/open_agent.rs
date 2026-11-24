//! "Open the agent": starts the profile's agent once, in a visible terminal,
//! in the working folder GitPoulpi will run it in, so the user answers its
//! first-run questions (folder trust, login, theme...) by hand. GitPoulpi
//! never answers them.
//!
//! The terminal is the user's own (the same choice as "Open in terminal"):
//! the agent's command line is appended to that terminal's launcher where
//! its syntax is known (`wt`, `cmd`, `xdg-terminal-exec`, `$TERMINAL -e`);
//! otherwise (a custom terminal template, macOS Terminal) the terminal opens
//! in the folder and the status line says which command to run.

use super::*;
use gitpoulpi_i18n::msg::ai_page as msg;
use std::path::{Path, PathBuf};

// The launch itself is shared with the palette's "Tab: Open AI Agent Here".
#[allow(unused_imports)]
pub(super) use crate::agent_launch::{agent_command_line, launcher_with_agent};

/// Opens the agent of `profile`; the message for the status line.
pub(super) fn open_agent(profile: &AiProfile, repo_root: &Path) -> Result<String, String> {
    if profile.kind != AiProviderKind::LocalAgent || profile.command.trim().is_empty() {
        return Err(msg::OPEN_AGENT_NO_AGENT.get().to_string());
    }
    let mut terminal_profile = profile.clone();
    terminal_profile.run_mode = gitpoulpi_config::AgentRunMode::Terminal;
    let folder: PathBuf =
        gitpoulpi_ai::agent_terminal::workdir_for(&terminal_profile, Some(repo_root))
            .unwrap_or_else(gitpoulpi_ai::agent_terminal::dedicated_dir);
    std::fs::create_dir_all(&folder).map_err(|e| msg::open_agent_failed(e.to_string()))?;
    crate::agent_launch::start_in(profile, &folder)
}

impl PreferencesState {
    pub(super) fn ai_open_agent(&mut self) {
        let Some(i) = self.selected_ai_profile() else {
            return;
        };
        let profile = self.config.ai.profiles[i].clone();
        self.message = Some(match open_agent(&profile, &self.repo_root) {
            Ok(done) => done,
            Err(why) => why,
        });
    }

    /// The palette's "AI: Open the Agent": the default profile's agent.
    pub(crate) fn ai_open_default_agent(&mut self) {
        let ai = &self.config.ai;
        let profile = ai
            .profile(&ai.default_profile)
            .or_else(|| ai.profiles.first())
            .cloned();
        self.message = Some(match profile {
            Some(profile) => match open_agent(&profile, &self.repo_root) {
                Ok(done) => done,
                Err(why) => why,
            },
            None => msg::OPEN_AGENT_NO_AGENT.get().to_string(),
        });
    }
}
