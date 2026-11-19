//! The machine the updater runs on, as traits, plus the checks that decide
//! whether a job may run at all.
//!
//! A job file is written by a user-level process, so none of it is trusted
//! for a decision about rights: the channel is **re-resolved from the files
//! on disk**, the policy is **re-read**, the package manager's command comes
//! from the plan (the job's copy must equal it), and the elevated worker
//! only runs from the installed copy.

use std::path::{Path, PathBuf};

use gitpoulpi_config::policy::{self, EffectiveUpdates, PolicySource};
use gitpoulpi_config::release::{self, MARKER_FILE};
use gitpoulpi_config::{UpdateMode, UpdatesConfig};
use gitpoulpi_update::channel::{self, ChannelEnv, Hive, RegistryOps, Resolved};
use gitpoulpi_update::fsops::FileOps;
use gitpoulpi_update::job::{Action, UpdateJob};
use gitpoulpi_update::plan::{self, PlanContext, Procedure};
use gitpoulpi_update::signing::PublicKey;

use crate::ops::{Clock, DaemonOps, ProcessOps, Registry, Transfer};
use crate::outcome::Outcome;

pub struct Env<'a> {
    pub fs: &'a dyn FileOps,
    pub proc: &'a dyn ProcessOps,
    pub daemons: &'a dyn DaemonOps,
    pub registry: &'a dyn Registry,
    pub clock: &'a dyn Clock,
    pub transfer: &'a dyn Transfer,
    pub channel_env: &'a dyn ChannelEnv,
    pub policy: &'a dyn PolicySource,
    pub host_target: String,
    /// The release signing keys (production: `signing::release_keys()`); the
    /// elevated worker checks the staged archive's signature against them.
    pub keys: Vec<PublicKey>,
    /// This executable, to tell the installed copy from the TUI's copy.
    pub own_exe: PathBuf,
    pub own_pid: u32,
    /// Print each status message on the console.
    pub echo: bool,
    /// Programs the new release no longer ships (production:
    /// `release::RETIRED`). Their files are deleted after a successful swap,
    /// and a relaunch recorded against one starts `gitpoulpi` instead.
    pub retired: &'a [&'a str],
}

/// Lower-case, forward slashes, no trailing separator.
pub fn same_dir(a: &Path, b: &Path) -> bool {
    let norm = |p: &Path| {
        p.to_string_lossy()
            .replace('\\', "/")
            .trim_end_matches('/')
            .to_lowercase()
    };
    norm(a) == norm(b)
}

struct Reader<'a>(&'a dyn Registry);

impl RegistryOps for Reader<'_> {
    fn inno_install_location(&self, hive: Hive) -> Option<String> {
        self.0.reader().inno_install_location(hive)
    }
}

/// What a job turned out to ask for, after the checks.
pub struct Checked {
    pub resolved: Resolved,
    pub procedure: Procedure,
    pub effective: EffectiveUpdates,
}

impl Env<'_> {
    pub fn exe_in(&self, dir: &Path, binary: &str) -> PathBuf {
        dir.join(release::exe_name(binary, &self.host_target))
    }

    /// The file name of this process's own exe when it lives in `dir`: the
    /// one file a Windows swap may rename although it is running.
    pub fn own_name_in(&self, dir: &Path) -> String {
        match (self.own_exe.parent(), self.own_exe.file_name()) {
            (Some(parent), Some(name)) if same_dir(parent, dir) => {
                name.to_string_lossy().into_owned()
            }
            _ => String::new(),
        }
    }

    /// The machine policy, never the user's `config.toml`: this decides
    /// what an elevated process may do.
    pub fn effective(&self) -> EffectiveUpdates {
        let loaded = policy::load(self.policy);
        for line in &loaded.warnings {
            eprintln!("{line}");
        }
        policy::merge(&loaded.policy, &UpdatesConfig::default())
    }

    /// Validates `job` against what is on disk.
    pub fn check(&self, job: &UpdateJob) -> Result<Checked, Outcome> {
        let dir = &job.install_dir;
        if !self.fs.exists(&dir.join(MARKER_FILE)) {
            return Err(Outcome::BadJob(format!(
                "{} holds no GitPoulpi install (no {MARKER_FILE})",
                dir.display()
            )));
        }
        let exe = self.exe_in(dir, "gitpoulpi");
        let resolved = channel::resolve(
            self.fs,
            &Reader(self.registry),
            self.channel_env,
            &exe,
            &self.host_target,
        );
        let effective = self.effective();
        if effective.mode == UpdateMode::Off {
            return Err(Outcome::Refused(
                "updates are turned off by the administrator".into(),
            ));
        }
        if !effective.allow_self_update {
            return Err(Outcome::Refused(
                "the administrator does not allow GitPoulpi to update itself".into(),
            ));
        }
        let ctx = PlanContext {
            to_version: job.to_version.clone(),
            previous_version: Some(job.to_version.clone()),
            ..PlanContext::default()
        };
        let forward = plan::plan(&resolved, &effective, &ctx);
        let procedure = match job.action {
            Action::Apply => forward,
            Action::Revert => plan::revert_plan(&resolved, &effective, &ctx),
            Action::Manager => {
                let wanted = job.manager_argv.clone().unwrap_or_default();
                // Either direction's command: `scoop update` or `scoop reset`.
                [forward, plan::revert_plan(&resolved, &effective, &ctx)]
                    .into_iter()
                    .find(|p| matches!(p, Procedure::Run { argv, .. } if *argv == wanted))
                    .unwrap_or(Procedure::Nothing(plan::NothingReason::NoInstallRecord))
            }
        };
        let fits = matches!(
            (&procedure, job.action),
            (Procedure::Swap { .. }, Action::Apply | Action::Revert)
                | (Procedure::Run { .. }, Action::Manager)
        );
        if !fits {
            return Err(Outcome::BadJob(format!(
                "the job asks for {:?}, which is not what this install's channel does",
                job.action
            )));
        }
        Ok(Checked {
            resolved,
            procedure,
            effective,
        })
    }

    /// The mirror hosts the policy allows for checksum downloads.
    pub fn extra_hosts(effective: &EffectiveUpdates) -> Vec<String> {
        effective.download_base.iter().cloned().collect()
    }
}
