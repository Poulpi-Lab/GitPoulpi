//! Which tab a start opens on (`general.startup_tab_in_repo`,
//! `general.startup_tab_outside_repo`, `general.startup_tab_launcher`).
//!
//! The decision is a pure function of the launch facts and the three
//! settings, so it is unit-tested without a terminal; `run_windows` feeds it
//! and acts on the answer. Read once at startup: a running window's Settings
//! edit changes the next start, not this one.

use gitpoulpi_config::{GeneralConfig, StartupTab};
use std::path::PathBuf;

/// What the start folder, the command line and the saved session say.
#[derive(Debug, Clone, Default)]
pub(crate) struct LaunchFacts {
    /// The git work tree the start folder is in.
    pub in_repo: Option<PathBuf>,
    /// `--repo <path>` was given: that repository is opened, whatever the
    /// settings say.
    pub explicit_repo: bool,
    /// Started by a desktop / Start menu shortcut (`--launcher`).
    pub launcher: bool,
    /// `general.remember_tabs`: without it nothing is "last".
    pub remember: bool,
    /// The repository tab that was active when the session last quit, if it
    /// is still a repository.
    pub last: Option<PathBuf>,
    /// The Radar is built and enabled.
    pub radar_offered: bool,
}

/// What the window opens on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Start {
    /// This repository's tab.
    Repo(PathBuf),
    /// The Home tab.
    Home,
    /// The Radar tab.
    Radar,
}

/// The setting that applies to this start: a launcher start wins, then the
/// start folder decides.
pub(crate) fn setting_for(facts: &LaunchFacts, general: &GeneralConfig) -> StartupTab {
    if facts.launcher {
        general.startup_tab_launcher
    } else if facts.in_repo.is_some() {
        general.startup_tab_in_repo
    } else {
        general.startup_tab_outside_repo
    }
}

/// The tab to open on.
pub(crate) fn decide(facts: &LaunchFacts, general: &GeneralConfig) -> Start {
    if facts.explicit_repo {
        return facts.in_repo.clone().map_or(Start::Home, Start::Repo);
    }
    let last = || {
        facts
            .last
            .clone()
            .filter(|_| facts.remember)
            .map_or(Start::Home, Start::Repo)
    };
    match setting_for(facts, general) {
        StartupTab::CurrentRepo => facts.in_repo.clone().map_or_else(last, Start::Repo),
        StartupTab::Last => last(),
        StartupTab::Home => Start::Home,
        StartupTab::Radar if facts.radar_offered => Start::Radar,
        StartupTab::Radar => Start::Home,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repo() -> PathBuf {
        PathBuf::from("/work/alpha")
    }

    fn beta() -> PathBuf {
        PathBuf::from("/work/beta")
    }

    fn facts() -> LaunchFacts {
        LaunchFacts {
            in_repo: Some(repo()),
            remember: true,
            last: Some(beta()),
            radar_offered: true,
            ..LaunchFacts::default()
        }
    }

    fn general(in_repo: StartupTab, outside: StartupTab, launcher: StartupTab) -> GeneralConfig {
        GeneralConfig {
            startup_tab_in_repo: in_repo,
            startup_tab_outside_repo: outside,
            startup_tab_launcher: launcher,
            ..GeneralConfig::default()
        }
    }

    #[test]
    fn the_defaults_keep_todays_behaviour_and_the_launcher_reopens_the_last_tab() {
        let g = GeneralConfig::default();
        assert_eq!(decide(&facts(), &g), Start::Repo(repo()));
        let outside = LaunchFacts {
            in_repo: None,
            ..facts()
        };
        assert_eq!(decide(&outside, &g), Start::Home);
        let launcher = LaunchFacts {
            launcher: true,
            in_repo: None,
            ..facts()
        };
        // Owner, 2026-10-21: from a launcher, the last tab by default.
        assert_ne!(decide(&launcher, &g), Start::Home);
    }

    #[test]
    fn in_a_repo_each_choice_picks_its_tab() {
        let pick = |tab| decide(&facts(), &general(tab, StartupTab::Home, StartupTab::Home));
        assert_eq!(pick(StartupTab::CurrentRepo), Start::Repo(repo()));
        assert_eq!(pick(StartupTab::Last), Start::Repo(beta()));
        assert_eq!(pick(StartupTab::Home), Start::Home);
        assert_eq!(pick(StartupTab::Radar), Start::Radar);
    }

    #[test]
    fn outside_a_repo_the_outside_setting_applies() {
        let outside = LaunchFacts {
            in_repo: None,
            ..facts()
        };
        let pick = |tab| decide(&outside, &general(StartupTab::Home, tab, StartupTab::Home));
        assert_eq!(pick(StartupTab::Last), Start::Repo(beta()));
        assert_eq!(pick(StartupTab::Home), Start::Home);
        assert_eq!(pick(StartupTab::Radar), Start::Radar);
        // "current repo" with no repo is the last tab.
        assert_eq!(pick(StartupTab::CurrentRepo), Start::Repo(beta()));
    }

    #[test]
    fn a_launcher_start_uses_the_launcher_setting_even_inside_a_repo() {
        let launched = LaunchFacts {
            launcher: true,
            ..facts()
        };
        let g = general(StartupTab::Home, StartupTab::Home, StartupTab::Last);
        assert_eq!(decide(&launched, &g), Start::Repo(beta()));
        let g = general(StartupTab::Last, StartupTab::Last, StartupTab::Radar);
        assert_eq!(decide(&launched, &g), Start::Radar);
    }

    #[test]
    fn last_without_a_remembered_tab_or_with_remember_off_is_home() {
        let g = general(StartupTab::Last, StartupTab::Last, StartupTab::Last);
        let none = LaunchFacts {
            last: None,
            ..facts()
        };
        assert_eq!(decide(&none, &g), Start::Home);
        let off = LaunchFacts {
            remember: false,
            ..facts()
        };
        assert_eq!(decide(&off, &g), Start::Home);
    }

    #[test]
    fn radar_falls_back_to_home_when_it_is_off_or_not_built() {
        let g = general(StartupTab::Radar, StartupTab::Radar, StartupTab::Radar);
        let off = LaunchFacts {
            radar_offered: false,
            ..facts()
        };
        assert_eq!(decide(&off, &g), Start::Home);
    }

    #[test]
    fn an_explicit_repo_argument_always_opens_that_repo() {
        let g = general(StartupTab::Home, StartupTab::Home, StartupTab::Radar);
        let explicit = LaunchFacts {
            explicit_repo: true,
            launcher: true,
            ..facts()
        };
        assert_eq!(decide(&explicit, &g), Start::Repo(repo()));
        // A path that is not a repository opens Home (with the notice).
        let bad = LaunchFacts {
            in_repo: None,
            explicit_repo: true,
            ..facts()
        };
        assert_eq!(decide(&bad, &g), Start::Home);
    }
}
