//! Signing the commits a rewrite re-creates (slice S19), the way git does it
//! (`gpg-interface.c`), so the repository's own setup is honoured:
//!
//! - `gpg.format` picks the program: `gpg` (openpgp), `ssh-keygen` (ssh) or
//!   `gpgsm` (x509), overridable by `gpg.program`, `gpg.openpgp.program`,
//!   `gpg.ssh.program` and `gpg.x509.program`; `user.signingkey` is the key
//!   (for OpenPGP, the committer identity when it is not set).
//! - OpenPGP and X.509: `<program> --status-fd=2 -bsau <key>`, the payload on
//!   stdin, the armored signature on stdout.
//! - SSH: `<program> -Y sign -n git -f <key file> [-U] <payload file>`, the
//!   signature in `<payload file>.sig`; a literal key (`key::ssh-...` or
//!   `ssh-...`) goes to a temporary file and uses the agent (`-U`).
//!
//! The payload is the commit as it will be stored, without its `gpgsig`
//! header; the signature then becomes that header, after the other headers.
//! Every signature spawns a program (and maybe the agent): the cost is
//! measured and shown, never guessed. A signature is cached by its payload,
//! so a preview that has to run its pass twice signs a commit once.

use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use gitpoulpi_core::domain::rewrite::{
    OsFamily, RewriteRefusal, SigningCheck, SigningFormat, Tool, ToolStatus,
};
use sha1::{Digest, Sha1};

use crate::tools::{self, ToolProbe};

/// How long one signature may take: a passphrase prompt nobody sees must not
/// hang a run for ever.
const SIGN_TIMEOUT: Duration = Duration::from_secs(120);

const SMOKE_PAYLOAD: &[u8] = b"tree 4b825dc642cb6eb9a060e54bf8d69288fbee4904\nauthor GitPoulpi <check@example.org> 0 +0000\ncommitter GitPoulpi <check@example.org> 0 +0000\n\nsigning check\n";

/// Why signing cannot happen: a typed reason and the program's own words.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Failure {
    pub(crate) reason: RewriteRefusal,
    pub(crate) detail: String,
}

impl Failure {
    fn missing(detail: String) -> Self {
        Self {
            reason: RewriteRefusal::ToolMissing,
            detail,
        }
    }

    fn failed(detail: String) -> Self {
        Self {
            reason: RewriteRefusal::SigningFailed,
            detail,
        }
    }
}

/// A signing setup that works (or at least resolved), with a place for the
/// temporary files the SSH program needs.
pub(crate) struct Signer {
    format: SigningFormat,
    program: PathBuf,
    key: String,
    /// The SSH key file, when the key is a literal one written to `tmp`.
    literal_key: Option<PathBuf>,
    tmp: tempfile::TempDir,
    cache: HashMap<[u8; 20], String>,
    counter: u32,
    /// Programs run (cached signatures excluded) and the time they took.
    pub(crate) signed: u64,
    pub(crate) millis: u64,
}

impl Signer {
    /// Reads the repository's signing config and finds the program. No
    /// signature is made yet.
    pub(crate) fn open(root: &Path, probe: &ToolProbe) -> Result<Self, Failure> {
        let cfg = tools::signing_config(root);
        let tool = cfg.format.tool();
        match probe.fresh(tool, root) {
            ToolStatus::Installed { .. } => {}
            ToolStatus::NotConfigured { reason } => return Err(Failure::failed(reason)),
            ToolStatus::TooOld { version, min } => {
                return Err(Failure::missing(format!(
                    "{} {version} is older than {min}",
                    tool.name()
                )))
            }
            ToolStatus::Missing => {
                return Err(Failure::missing(format!("{} was not found", tool.name())))
            }
        }
        let program = tools::signing_program(&cfg, root)
            .ok_or_else(|| Failure::missing(format!("{} was not found", tool.name())))?;
        let tmp = tempfile::Builder::new()
            .prefix("gitpoulpi-sign")
            .tempdir()
            .map_err(|e| Failure::failed(format!("cannot make a temporary directory: {e}")))?;
        let key = match (cfg.key.clone(), cfg.format) {
            (Some(key), _) => key,
            (None, SigningFormat::OpenPgp) => identity_for_key(root),
            (None, _) => return Err(Failure::failed("user.signingkey is not set".into())),
        };
        let mut literal_key = None;
        if cfg.format == SigningFormat::Ssh {
            if let Some(text) = literal_ssh_key(&key) {
                let path = tmp.path().join("signing-key.pub");
                std::fs::write(&path, format!("{text}\n"))
                    .map_err(|e| Failure::failed(format!("cannot write the key: {e}")))?;
                literal_key = Some(path);
            }
        }
        Ok(Self {
            format: cfg.format,
            program,
            key,
            literal_key,
            tmp,
            cache: HashMap::new(),
            counter: 0,
            signed: 0,
            millis: 0,
        })
    }

    /// [`Signer::open`] and the smoke test: a run that re-signs starts only
    /// with a setup that has just signed and verified once.
    pub(crate) fn open_checked(root: &Path, probe: &ToolProbe) -> Result<Self, Failure> {
        let mut signer = Self::open(root, probe)?;
        signer.smoke()?;
        Ok(signer)
    }

    /// Signs a dummy payload and verifies it; the milliseconds one
    /// signature took. The counters start again afterwards.
    pub(crate) fn smoke(&mut self) -> Result<u64, Failure> {
        let started = Instant::now();
        let sig = self.sign(SMOKE_PAYLOAD)?;
        let millis = (started.elapsed().as_millis() as u64).max(1);
        self.verify(SMOKE_PAYLOAD, &sig)?;
        self.signed = 0;
        self.millis = 0;
        Ok(millis)
    }

    /// The armored signature of `payload`.
    pub(crate) fn sign(&mut self, payload: &[u8]) -> Result<String, Failure> {
        let digest: [u8; 20] = Sha1::digest(payload).into();
        if let Some(sig) = self.cache.get(&digest) {
            return Ok(sig.clone());
        }
        let started = Instant::now();
        let sig = match self.format {
            SigningFormat::OpenPgp | SigningFormat::X509 => self.sign_gpg(payload)?,
            SigningFormat::Ssh => self.sign_ssh(payload)?,
        };
        self.millis += started.elapsed().as_millis() as u64;
        self.signed += 1;
        self.cache.insert(digest, sig.clone());
        Ok(sig)
    }

    fn sign_gpg(&self, payload: &[u8]) -> Result<String, Failure> {
        let mut cmd = Command::new(&self.program);
        cmd.args(["--status-fd=2", "-bsau", &self.key]);
        let out = run(cmd, Some(payload))?;
        let sig = String::from_utf8_lossy(&out.stdout).trim_end().to_string();
        let stderr = String::from_utf8_lossy(&out.stderr);
        if !out.success || sig.is_empty() || !stderr.contains("[GNUPG:] SIG_CREATED") {
            return Err(Failure::failed(why(
                &stderr,
                "the program made no signature",
            )));
        }
        Ok(sig)
    }

    fn sign_ssh(&mut self, payload: &[u8]) -> Result<String, Failure> {
        self.counter += 1;
        let buffer = self.tmp.path().join(format!("payload-{}", self.counter));
        std::fs::write(&buffer, payload)
            .map_err(|e| Failure::failed(format!("cannot write the payload: {e}")))?;
        let mut cmd = Command::new(&self.program);
        cmd.args(["-Y", "sign", "-n", "git", "-f"]);
        match &self.literal_key {
            Some(file) => cmd.arg(file).arg("-U"),
            None => cmd.arg(expand_home(&self.key)),
        };
        cmd.arg(&buffer);
        let result = run(cmd, None);
        let sig_file = PathBuf::from(format!("{}.sig", buffer.display()));
        let sig = std::fs::read_to_string(&sig_file);
        let _ = std::fs::remove_file(&buffer);
        let _ = std::fs::remove_file(&sig_file);
        let out = result?;
        match sig {
            Ok(sig) if out.success && !sig.trim().is_empty() => Ok(sig.trim_end().to_string()),
            _ => Err(Failure::failed(why(
                &String::from_utf8_lossy(&out.stderr),
                "the program made no signature",
            ))),
        }
    }

    /// Whether `sig` is a good signature of `payload` (the key is not
    /// checked against anyone: this proves the pair works).
    pub(crate) fn verify(&mut self, payload: &[u8], sig: &str) -> Result<(), Failure> {
        self.counter += 1;
        let base = self.tmp.path().join(format!("verify-{}", self.counter));
        let sig_file = PathBuf::from(format!("{}.sig", base.display()));
        std::fs::write(&sig_file, format!("{sig}\n"))
            .map_err(|e| Failure::failed(format!("cannot write the signature: {e}")))?;
        let mut cmd = Command::new(&self.program);
        let out = match self.format {
            SigningFormat::Ssh => {
                // The payload is stdin from a file, never a pipe: Windows'
                // OpenSSH `ssh-keygen -Y check-novalidate` waits forever on
                // an anonymous pipe (it answers at once on a file).
                let payload_file = PathBuf::from(format!("{}.payload", base.display()));
                std::fs::write(&payload_file, payload)
                    .map_err(|e| Failure::failed(format!("cannot write the payload: {e}")))?;
                cmd.args(["-Y", "check-novalidate", "-n", "git", "-s"])
                    .arg(&sig_file);
                let ran = std::fs::File::open(&payload_file)
                    .map_err(|e| Failure::failed(format!("cannot read the payload: {e}")))
                    .and_then(|file| run_with(cmd, Stdin::File(file)));
                let _ = std::fs::remove_file(&payload_file);
                ran
            }
            _ => {
                std::fs::write(&base, payload)
                    .map_err(|e| Failure::failed(format!("cannot write the payload: {e}")))?;
                cmd.arg("--verify").arg(&sig_file).arg(&base);
                run(cmd, None)
            }
        };
        let _ = std::fs::remove_file(&sig_file);
        let _ = std::fs::remove_file(&base);
        let out = out?;
        if out.success {
            Ok(())
        } else {
            Err(Failure::failed(why(
                &String::from_utf8_lossy(&out.stderr),
                "the signature does not verify",
            )))
        }
    }
}

/// The signature as the `gpgsig` header of `body` (a commit without one):
/// after the last header, every continuation line indented by one space.
pub(crate) fn attach(body: &[u8], sig: &str) -> Vec<u8> {
    let at = body
        .windows(2)
        .position(|w| w == b"\n\n")
        .map_or(body.len(), |p| p + 1);
    let mut out = Vec::with_capacity(body.len() + sig.len() + 16);
    out.extend_from_slice(&body[..at]);
    if at == body.len() && !body.is_empty() && !body.ends_with(b"\n") {
        out.push(b'\n');
    }
    out.extend_from_slice(b"gpgsig ");
    out.extend_from_slice(sig.trim_end().replace('\n', "\n ").as_bytes());
    out.push(b'\n');
    out.extend_from_slice(&body[at..]);
    out
}

/// The "Check signing" smoke test: sign a dummy payload and verify it.
pub(crate) fn check(root: &Path, probe: &ToolProbe) -> SigningCheck {
    let cfg = tools::signing_config(root);
    let tool: Tool = cfg.format.tool();
    let done = |reason: Option<RewriteRefusal>, detail: String, millis: u64| SigningCheck {
        format: cfg.format,
        tool,
        hint_key: tool.hint_key(OsFamily::current()).to_string(),
        reason,
        detail,
        millis,
    };
    let mut signer = match Signer::open(root, probe) {
        Ok(signer) => signer,
        Err(f) => return done(Some(f.reason), f.detail, 0),
    };
    match signer.smoke() {
        Ok(millis) => done(None, String::new(), millis),
        Err(f) => done(Some(f.reason), f.detail, 0),
    }
}

/// `Name <email>` of the committer, what git signs with when no key is set.
fn identity_for_key(root: &Path) -> String {
    super::walk::committer_ident(root)
        .ok()
        .and_then(|bytes| String::from_utf8(bytes).ok())
        .and_then(|ident| ident.rfind('>').map(|gt| ident[..=gt].to_string()))
        .unwrap_or_default()
}

/// The key text of a literal SSH key (`key::ssh-ed25519 AAAA...`), `None`
/// for a path.
fn literal_ssh_key(key: &str) -> Option<String> {
    let key = key.trim();
    key.strip_prefix("key::")
        .map(str::to_string)
        .or_else(|| key.starts_with("ssh-").then(|| key.to_string()))
}

fn expand_home(path: &str) -> PathBuf {
    let home = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE"));
    match (path.strip_prefix("~/"), home) {
        (Some(rest), Some(home)) => PathBuf::from(home).join(rest),
        _ => PathBuf::from(path),
    }
}

/// The useful part of a program's stderr (its last lines), or `fallback`.
fn why(stderr: &str, fallback: &str) -> String {
    let lines: Vec<&str> = stderr
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with("[GNUPG:] ") || l.contains("ERROR"))
        .collect();
    let tail: Vec<&str> = lines.iter().rev().take(3).rev().copied().collect();
    if tail.is_empty() {
        fallback.to_string()
    } else {
        tail.join("\n")
    }
}

struct Ran {
    success: bool,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

/// Runs `cmd` with `stdin` (none: closed), no terminal input, and the
/// timeout; a spawn that fails is a missing program.
fn run(cmd: Command, stdin: Option<&[u8]>) -> Result<Ran, Failure> {
    run_with(cmd, stdin.map_or(Stdin::Null, Stdin::Bytes))
}

/// What a program reads on its standard input.
enum Stdin<'a> {
    Null,
    Bytes(&'a [u8]),
    File(std::fs::File),
}

fn run_with(mut cmd: Command, stdin: Stdin<'_>) -> Result<Ran, Failure> {
    let stdin = match stdin {
        Stdin::Null => {
            cmd.stdin(Stdio::null());
            None
        }
        Stdin::Bytes(bytes) => {
            cmd.stdin(Stdio::piped());
            Some(bytes)
        }
        Stdin::File(file) => {
            cmd.stdin(Stdio::from(file));
            None
        }
    };
    // Output goes to anonymous files, never pipes: Windows' OpenSSH
    // `ssh-keygen` never exits on some error paths (a key that does not
    // exist) when its stdout/stderr are pipes, and exits at once on files.
    let out_file = tempfile::tempfile()
        .map_err(|e| Failure::failed(format!("cannot make a temporary file: {e}")))?;
    let err_file = tempfile::tempfile()
        .map_err(|e| Failure::failed(format!("cannot make a temporary file: {e}")))?;
    let io_err = |e: std::io::Error| Failure::failed(format!("cannot capture the output: {e}"));
    cmd.stdout(Stdio::from(out_file.try_clone().map_err(io_err)?))
        .stderr(Stdio::from(err_file.try_clone().map_err(io_err)?));
    let mut child = tools::retry_busy(|| cmd.spawn()).map_err(|e| {
        Failure::missing(format!(
            "cannot run {}: {e}",
            cmd.get_program().to_string_lossy()
        ))
    })?;
    let writer = stdin.map(|bytes| {
        let mut pipe = child.stdin.take();
        let bytes = bytes.to_vec();
        std::thread::spawn(move || {
            if let Some(pipe) = pipe.as_mut() {
                let _ = pipe.write_all(&bytes);
            }
        })
    });
    let started = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Some(status),
            Ok(None) if started.elapsed() < SIGN_TIMEOUT => {
                std::thread::sleep(Duration::from_millis(2));
            }
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                break None;
            }
        }
    };
    if let Some(writer) = writer {
        let _ = writer.join();
    }
    let slurp = |mut file: std::fs::File| {
        use std::io::{Seek, SeekFrom};
        let mut bytes = Vec::new();
        let _ = file.seek(SeekFrom::Start(0));
        let _ = file.read_to_end(&mut bytes);
        bytes
    };
    let stdout = slurp(out_file);
    let stderr = slurp(err_file);
    match status {
        Some(status) => Ok(Ran {
            success: status.success(),
            stdout,
            stderr,
        }),
        None => Err(Failure::failed(format!(
            "{} did not answer in {} s (a passphrase prompt nobody can see?)",
            cmd.get_program().to_string_lossy(),
            SIGN_TIMEOUT.as_secs()
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_signature_becomes_the_last_header_with_continuation_lines() {
        let body = b"tree 1\nauthor A <a@x> 1 +0000\ncommitter A <a@x> 1 +0000\n\nmessage\n";
        let out = attach(
            body,
            "-----BEGIN SSH SIGNATURE-----\nabc\n\ndef\n-----END SSH SIGNATURE-----\n",
        );
        let text = String::from_utf8(out).unwrap();
        assert_eq!(
            text,
            "tree 1\nauthor A <a@x> 1 +0000\ncommitter A <a@x> 1 +0000\ngpgsig -----BEGIN SSH SIGNATURE-----\n abc\n \n def\n -----END SSH SIGNATURE-----\n\nmessage\n"
        );
    }

    #[test]
    fn a_commit_without_a_message_gets_the_header_at_the_end() {
        let out = attach(b"tree 1\ncommitter A <a@x> 1 +0000\n", "sig");
        assert_eq!(out, b"tree 1\ncommitter A <a@x> 1 +0000\ngpgsig sig\n");
    }

    #[test]
    fn literal_keys_are_told_from_paths() {
        assert_eq!(
            literal_ssh_key("key::ssh-ed25519 AAA"),
            Some("ssh-ed25519 AAA".into())
        );
        assert_eq!(
            literal_ssh_key("ssh-ed25519 AAA"),
            Some("ssh-ed25519 AAA".into())
        );
        assert_eq!(literal_ssh_key("~/.ssh/id_ed25519"), None);
    }
}
