//! A stand-in for `ssh` that makes a working SSH remote out of a local
//! directory, with no network and no server.
//!
//! git runs `ssh <options> <host> "git-upload-pack '<path>'"`. The stub
//! records its arguments, then runs that last argument with `sh -c`, so a
//! `ssh://git@stub.invalid/<abs path>` URL reaches the bare repository at
//! `<abs path>`. With `-v` it also prints the two lines a real ssh prints
//! when a key is accepted (`Server accepts key: ...`, `Authenticated to ...`),
//! naming the key given with `-i`, or `agent-key` when none was.
//!
//! The program is called `ssh` on purpose: git recognises the *variant* by
//! the command's base name, and an unknown name makes it probe with `-G`.
//! It is a `sh` script (the precedent is `gitpoulpi-cli-backend`'s `ssh_key`
//! test); on Windows git runs `core.sshCommand` through Git for Windows' `sh`,
//! which starts it.

use std::path::{Path, PathBuf};

use gitpoulpi_core::repo_config::sh_quote_path;

/// See the module docs.
pub struct StubSsh {
    _dir: tempfile::TempDir,
    program: PathBuf,
    log: PathBuf,
}

impl StubSsh {
    /// An ssh that lets the connection through.
    pub fn new() -> Self {
        Self::build(false)
    }

    /// An ssh whose server refuses the key: `Permission denied (publickey).`
    /// and exit status 255, as ssh does.
    pub fn denying() -> Self {
        Self::build(true)
    }

    fn build(deny: bool) -> Self {
        let dir = tempfile::tempdir().expect("stub ssh dir");
        let program = dir.path().join("ssh");
        let log = dir.path().join("calls.txt");
        let body = if deny {
            "echo 'git@stub.invalid: Permission denied (publickey).' >&2\nexit 255\n".to_string()
        } else {
            r#"key=agent-key
verbose=no
prev=
for a in "$@"; do
  [ "$prev" = "-i" ] && key=$a
  [ "$a" = "-v" ] && verbose=yes
  prev=$a
  last=$a
done
if [ "$verbose" = yes ]; then
  echo "debug1: Server accepts key: $key ED25519 SHA256:stubstubstub explicit" >&2
  echo 'Authenticated to stub.invalid ([127.0.0.1]:22) using "publickey".' >&2
fi
exec sh -c "$last"
"#
            .to_string()
        };
        // One record per invocation, appended under a lock: the system
        // backend's fetch runs two ssh processes at once (the remote and its
        // mirrors), and `sh` flushes a `printf` line by line, so a record
        // written without the lock interleaved with the other's. `mkdir` is
        // the portable atomic lock (no `flock` on macOS).
        let log_q = sh_quote_path(&log.to_string_lossy());
        let lock_q = sh_quote_path(&format!("{}.lock", log.to_string_lossy()));
        let script = format!(
            "#!/bin/sh\nuntil mkdir {lock_q} 2>/dev/null; do :; done\n\
             {{ for a in \"$@\"; do printf '%s\\n' \"$a\"; done; echo '--'; }} >> {log_q}\n\
             rmdir {lock_q}\n{body}"
        );
        std::fs::write(&program, script).expect("write the stub ssh");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&program, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
        Self {
            _dir: dir,
            program,
            log,
        }
    }

    /// The stub as a `GIT_SSH_COMMAND`.
    pub fn command(&self) -> String {
        sh_quote_path(&self.program.to_string_lossy())
    }

    /// The stub as the command a key file choice produces
    /// (`<stub> -i "<key>" -o IdentitiesOnly=yes`).
    pub fn command_with_key(&self, key: &Path) -> String {
        format!(
            "{} -i {} -o IdentitiesOnly=yes",
            self.command(),
            sh_quote_path(&key.to_string_lossy())
        )
    }

    /// A URL that reaches the repository at `path` through this stub.
    pub fn url_for(path: &Path) -> String {
        let path = path.display().to_string().replace('\\', "/");
        let slash = if path.starts_with('/') { "" } else { "/" };
        format!("ssh://git@stub.invalid{slash}{path}")
    }

    /// Every invocation so far, each as its argument list.
    pub fn calls(&self) -> Vec<Vec<String>> {
        let text = std::fs::read_to_string(&self.log).unwrap_or_default();
        let mut calls = vec![Vec::new()];
        for line in text.lines() {
            if line == "--" {
                calls.push(Vec::new());
            } else {
                calls.last_mut().unwrap().push(line.to_string());
            }
        }
        calls.pop();
        calls
    }

    /// The `-i` value of every invocation that had one.
    pub fn identity_files(&self) -> Vec<String> {
        self.calls()
            .iter()
            .filter_map(|args| {
                let at = args.iter().position(|a| a == "-i")?;
                args.get(at + 1).cloned()
            })
            .collect()
    }
}

impl Default for StubSsh {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    #[cfg_attr(not(unix), allow(unused_imports))]
    use super::*;

    /// Many invocations at once, each with its own arguments: every logged
    /// call is exactly one invocation's argument list, none torn or mixed.
    #[cfg(unix)] // runs the script itself; on Windows only git's `sh` can start it
    #[test]
    fn concurrent_invocations_are_logged_intact() {
        let stub = StubSsh::new();
        let program = stub.program.clone();
        let threads: Vec<_> = (0..16)
            .map(|t| {
                let program = program.clone();
                std::thread::spawn(move || {
                    for n in 0..8 {
                        let args = [
                            "-i".to_string(),
                            format!("/keys/{t}-{n}"),
                            "-o".to_string(),
                            format!("Opt={t}-{n}"),
                            format!("host-{t}-{n}"),
                        ];
                        // The stub ends by running its last argument: `true`.
                        let status = std::process::Command::new(&program)
                            .args(&args)
                            .arg("true")
                            .status()
                            .unwrap();
                        assert!(status.success());
                    }
                })
            })
            .collect();
        for thread in threads {
            thread.join().unwrap();
        }
        let calls = stub.calls();
        assert_eq!(calls.len(), 16 * 8);
        for call in &calls {
            assert_eq!(call.len(), 6, "{call:?}");
            let id = call[4].strip_prefix("host-").expect("a whole call");
            assert_eq!(call[1], format!("/keys/{id}"), "{call:?}");
            assert_eq!(call[3], format!("Opt={id}"), "{call:?}");
            assert_eq!(call[5], "true", "{call:?}");
        }
    }
}
