//! `gitpoulpi job list|cancel|wait` against a real standalone daemon, and
//! Ctrl+C on a command that started a job.
//!
//! The daemon runs with `GITPOULPI_TEST_TRANSFER_DELAY_MS`, which makes every
//! network job sleep after it has its lock (before git is called), so a push
//! is a job that can be found `Running`. Own config dir, session and fixture
//! per test; assertions are on deserialized JSON.

use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use serde_json::Value;

const BIN: &str = env!("CARGO_BIN_EXE_gitpoulpi");
const LIMIT: Duration = Duration::from_secs(60);

struct Rig {
    config: tempfile::TempDir,
    repo: FixtureRepo,
    session: String,
}

impl Rig {
    /// A fixture with an unpushed `main` and a daemon that holds each
    /// network job for `delay_ms` before it touches git.
    fn start(session: &str, delay_ms: u64) -> Self {
        let repo = FixtureRepoBuilder::new()
            .unwrap()
            .commit("initial")
            .unwrap()
            .with_origin()
            .unwrap()
            .build();
        let rig = Self {
            config: tempfile::tempdir().unwrap(),
            repo,
            session: session.to_string(),
        };
        let (ok, started, text) = rig.run_with(
            &["daemon", "start", "--json"],
            &[("GITPOULPI_TEST_TRANSFER_DELAY_MS", &delay_ms.to_string())],
        );
        assert!(ok, "daemon start: {text}");
        assert_eq!(started["ok"], Value::Bool(true), "{started}");
        rig
    }

    fn command(&self, args: &[&str]) -> Command {
        let mut cmd = Command::new(BIN);
        cmd.env("GITPOULPI_CONFIG_DIR", self.config.path())
            .current_dir(self.repo.root())
            .args(["-s", &self.session])
            .args(args);
        cmd
    }

    /// (exit ok, stdout parsed as JSON or Null, stdout + stderr). Output goes
    /// through files: a detached daemon child would hold a pipe open.
    fn run_with(&self, args: &[&str], env: &[(&str, &str)]) -> (bool, Value, String) {
        let out = tempfile::NamedTempFile::new().unwrap();
        let err = tempfile::NamedTempFile::new().unwrap();
        let mut cmd = self.command(args);
        cmd.envs(env.iter().copied())
            .stdout(out.reopen().unwrap())
            .stderr(err.reopen().unwrap());
        let status = wait_exit(&mut cmd.spawn().unwrap());
        let stdout = std::fs::read_to_string(out.path()).unwrap_or_default();
        let stderr = std::fs::read_to_string(err.path()).unwrap_or_default();
        let json = serde_json::from_str(&stdout).unwrap_or(Value::Null);
        (status, json, format!("{stdout}{stderr}"))
    }

    fn run(&self, args: &[&str]) -> (bool, Value, String) {
        self.run_with(args, &[])
    }

    /// A `push` left running in the background; its output files are kept
    /// alive by the returned handle.
    fn spawn_push(&self) -> Pushing {
        let out = tempfile::NamedTempFile::new().unwrap();
        let err = tempfile::NamedTempFile::new().unwrap();
        let child = self
            .command(&["push", "--json"])
            .stdin(Stdio::null())
            .stdout(out.reopen().unwrap())
            .stderr(err.reopen().unwrap())
            .spawn()
            .unwrap();
        Pushing { child, out, err }
    }

    fn jobs(&self) -> Vec<Value> {
        let (ok, json, text) = self.run(&["job", "list", "--json"]);
        assert!(ok, "job list: {text}");
        json.as_array().unwrap_or_else(|| panic!("{json}")).clone()
    }

    /// Polls `job list` until a job of `kind` is in `state`; returns it.
    fn job_in(&self, kind: &str, state: &str) -> Value {
        let deadline = Instant::now() + LIMIT;
        loop {
            if let Some(job) = self
                .jobs()
                .into_iter()
                .find(|j| j["kind"] == kind && j["state"] == state)
            {
                return job;
            }
            assert!(Instant::now() < deadline, "no {kind} job became {state}");
            std::thread::sleep(Duration::from_millis(50));
        }
    }

    fn origin_has_main(&self) -> bool {
        !self
            .repo
            .git_in_remote(&["branch", "--list", "main"])
            .trim()
            .is_empty()
    }
}

impl Drop for Rig {
    fn drop(&mut self) {
        let _ = self.command(&["daemon", "stop", "--json"]).output();
    }
}

struct Pushing {
    child: Child,
    out: tempfile::NamedTempFile,
    err: tempfile::NamedTempFile,
}

impl Pushing {
    /// (exit ok, stdout + stderr) once the push command has ended.
    fn finish(mut self) -> (bool, String) {
        let ok = wait_exit(&mut self.child);
        let stdout = std::fs::read_to_string(self.out.path()).unwrap_or_default();
        let stderr = std::fs::read_to_string(self.err.path()).unwrap_or_default();
        (ok, format!("{stdout}{stderr}"))
    }
}

fn wait_exit(child: &mut Child) -> bool {
    let deadline = Instant::now() + LIMIT;
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            return status.success();
        }
        if Instant::now() > deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!("a gitpoulpi command did not exit within {LIMIT:?}");
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}

#[test]
fn a_running_push_is_listed_and_job_cancel_ends_it() {
    let rig = Rig::start("jobs-cancel", 20_000);
    let push = rig.spawn_push();

    let job = rig.job_in("Push", "Running");
    assert!(job["label"].as_str().unwrap().contains("Pushing"), "{job}");
    assert!(job["not_cancellable"].is_null(), "{job}");

    let id = job["id"].as_u64().unwrap().to_string();
    let (ok, canceled, text) = rig.run(&["job", "cancel", &id, "--json"]);
    assert!(ok, "{text}");
    assert_eq!(canceled["id"], job["id"], "{canceled}");

    let (pushed_ok, text) = push.finish();
    assert!(!pushed_ok, "the canceled push must fail: {text}");
    assert!(text.to_lowercase().contains("cancel"), "{text}");

    let (ok, waited, text) = rig.run(&["job", "wait", &id, "--json"]);
    assert!(!ok, "a canceled job exits non-zero: {text}");
    assert_eq!(waited["state"], "Canceled", "{waited}");
    assert!(!rig.origin_has_main(), "the canceled push never ran");
}

#[test]
fn job_wait_returns_the_outcome_of_a_push_that_completes() {
    let rig = Rig::start("jobs-wait", 1_500);
    let push = rig.spawn_push();
    let job = rig.job_in("Push", "Running");
    let id = job["id"].as_u64().unwrap().to_string();

    let (ok, waited, text) = rig.run(&["job", "wait", &id, "--json"]);
    assert!(ok, "{text}");
    assert_eq!(waited["state"], "Done", "{waited}");
    assert_eq!(waited["id"], job["id"], "{waited}");
    let (pushed_ok, text) = push.finish();
    assert!(pushed_ok, "{text}");
    assert!(rig.origin_has_main());

    let (ok, _, text) = rig.run(&["job", "wait", "9999", "--json"]);
    assert!(!ok, "an unknown job is an error: {text}");
}

#[test]
fn killing_a_cli_command_cancels_the_job_it_started() {
    let rig = Rig::start("jobs-ctrl-c", 20_000);
    let mut push = rig.spawn_push();
    let job = rig.job_in("Push", "Running");

    // What Ctrl+C does to the process: the connection to the daemon drops.
    push.child.kill().unwrap();
    let _ = push.child.wait();

    let ended = rig.job_in("Push", "Canceled");
    assert_eq!(ended["id"], job["id"], "{ended}");
    assert!(!rig.origin_has_main());
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
