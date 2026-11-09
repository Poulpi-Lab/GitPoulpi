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
//! Unix only: it is a `sh` script (the precedent is
//! `gitpoulpi-cli-backend`'s `ssh_key` test).
#![cfg(unix)]

use std::os::unix::fs::PermissionsExt;
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
        let script = format!(
            "#!/bin/sh\n{{ for a in \"$@\"; do printf '%s\\n' \"$a\"; done; echo '--'; }} >> {}\n{body}",
            sh_quote_path(&log.to_string_lossy())
        );
        std::fs::write(&program, script).expect("write the stub ssh");
        std::fs::set_permissions(&program, std::fs::Permissions::from_mode(0o755)).unwrap();
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
        format!("ssh://git@stub.invalid{}", path.display())
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
