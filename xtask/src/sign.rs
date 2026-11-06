//! `cargo run -p xtask -- sign --dir <dist> [--version <v>]`: signs every
//! release asset with minisign, writing `<asset>.minisig` next to it.
//!
//! The secret key comes from the environment, never from a file in the
//! repository: `MINISIGN_SECRET_KEY` holds the key box (the text of the
//! `.key` file) and `MINISIGN_PASSWORD` its password (omit it only for an
//! unencrypted key; the task never prompts).
//!
//! Every signature carries the trusted comment `file:<name> version:<v>`, so a
//! valid signature cannot be moved onto another file or onto an older release:
//! the updater compares the comment with what it downloaded. `.sha256` and
//! `.minisig` files are not signed (the manifest and the signatures cover
//! them).

use std::io::Cursor;
use std::path::{Path, PathBuf};

use minisign::{SecretKey, SecretKeyBox};

const HELP: &str = "\
Usage: cargo run -p xtask -- sign --dir <dist> [--version <v>]

Signs every file of <dist> except *.sha256 and *.minisig with minisign and
writes <file>.minisig. Reads MINISIGN_SECRET_KEY (the key box text) and
MINISIGN_PASSWORD from the environment. The trusted comment of each signature
is `file:<name> version:<v>`. Without --version the version of
<dist>/latest.json is used.";

/// The trusted comment bound to one asset.
pub fn trusted_comment(name: &str, version: &str) -> String {
    format!("file:{name} version:{version}")
}

/// True for the files that are never signed.
pub fn is_unsigned_kind(name: &str) -> bool {
    name.ends_with(".sha256") || name.ends_with(".minisig")
}

/// Signs the bytes of one asset; the signature text is the `.minisig` content.
pub fn sign_bytes(
    secret: &SecretKey,
    name: &str,
    version: &str,
    bytes: &[u8],
) -> Result<String, String> {
    let signature = minisign::sign(
        None,
        secret,
        Cursor::new(bytes),
        Some(&trusted_comment(name, version)),
        Some("signature from the GitPoulpi release key"),
    )
    .map_err(|e| format!("{name}: {e}"))?;
    Ok(signature.into_string())
}

/// Signs every eligible file of `dir`; returns the names that got a signature.
pub fn sign_dir(dir: &Path, version: &str, secret: &SecretKey) -> Result<Vec<String>, String> {
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .map_err(|e| format!("{}: {e}", dir.display()))?
        .filter_map(Result::ok)
        .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|n| !is_unsigned_kind(n))
        .collect();
    names.sort();
    for name in &names {
        let bytes = std::fs::read(dir.join(name)).map_err(|e| format!("{name}: {e}"))?;
        let text = sign_bytes(secret, name, version, &bytes)?;
        std::fs::write(dir.join(format!("{name}.minisig")), text)
            .map_err(|e| format!("{name}.minisig: {e}"))?;
    }
    Ok(names)
}

/// Reads the secret key from the two environment values.
pub fn secret_key(box_text: &str, password: Option<String>) -> Result<SecretKey, String> {
    let key_box = SecretKeyBox::from_string(box_text.trim_start())
        .map_err(|e| format!("MINISIGN_SECRET_KEY is not a minisign key: {e}"))?;
    match password.filter(|p| !p.is_empty()) {
        Some(pw) => key_box
            .into_secret_key(Some(pw))
            .map_err(|e| format!("MINISIGN_PASSWORD does not open the key: {e}")),
        None => key_box
            .into_unencrypted_secret_key()
            .map_err(|e| format!("MINISIGN_PASSWORD is not set and the key is encrypted: {e}")),
    }
}

fn version_of_manifest(dir: &Path) -> Result<String, String> {
    let path = dir.join("latest.json");
    let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let value: serde_json::Value =
        serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
    value["version"]
        .as_str()
        .map(str::to_string)
        .ok_or_else(|| format!("{}: no version", path.display()))
}

pub fn run(args: &[String]) -> Result<i32, String> {
    let mut dir: Option<PathBuf> = None;
    let mut version = None;
    let mut it = args.iter();
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                println!("{HELP}");
                return Ok(0);
            }
            "--dir" => dir = it.next().map(PathBuf::from),
            "--version" => version = it.next().cloned(),
            other => return Err(format!("unknown argument '{other}'\n\n{HELP}")),
        }
    }
    let dir = dir.ok_or("--dir <dist> is required")?;
    let version = match version {
        Some(v) => v,
        None => version_of_manifest(&dir)?,
    };
    let key_text = std::env::var("MINISIGN_SECRET_KEY")
        .map_err(|_| "MINISIGN_SECRET_KEY is not set".to_string())?;
    let secret = secret_key(&key_text, std::env::var("MINISIGN_PASSWORD").ok())?;
    let signed = sign_dir(&dir, &version, &secret)?;
    println!(
        "sign: {} file(s) signed for version {version}",
        signed.len()
    );
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use minisign::{KeyPair, SignatureBox};

    #[test]
    fn signs_every_asset_but_checksums_and_signatures() {
        let dir = tempfile::tempdir().unwrap();
        for name in ["a.tar.gz", "install.sh", "a.tar.gz.sha256", "old.minisig"] {
            std::fs::write(dir.path().join(name), name.as_bytes()).unwrap();
        }
        let pair = KeyPair::generate_unencrypted_keypair().unwrap();
        let signed = sign_dir(dir.path(), "1.2.3", &pair.sk).unwrap();
        assert_eq!(
            signed,
            vec!["a.tar.gz".to_string(), "install.sh".to_string()]
        );
        assert!(dir.path().join("a.tar.gz.minisig").exists());
        assert!(!dir.path().join("a.tar.gz.sha256.minisig").exists());
        assert!(!dir.path().join("old.minisig.minisig").exists());

        let text = std::fs::read_to_string(dir.path().join("install.sh.minisig")).unwrap();
        let sig = SignatureBox::from_string(&text).unwrap();
        assert_eq!(
            sig.trusted_comment().unwrap(),
            "file:install.sh version:1.2.3"
        );
        minisign::verify(
            &pair.pk,
            &sig,
            Cursor::new(b"install.sh"),
            true,
            false,
            false,
        )
        .unwrap();
    }

    #[test]
    fn a_signature_does_not_verify_other_bytes_or_another_key() {
        let pair = KeyPair::generate_unencrypted_keypair().unwrap();
        let other = KeyPair::generate_unencrypted_keypair().unwrap();
        let text = sign_bytes(&pair.sk, "x", "1.0.0", b"payload").unwrap();
        let sig = SignatureBox::from_string(&text).unwrap();
        assert!(
            minisign::verify(&pair.pk, &sig, Cursor::new(b"tampered"), true, false, false).is_err()
        );
        assert!(
            minisign::verify(&other.pk, &sig, Cursor::new(b"payload"), true, false, false).is_err()
        );
    }

    #[test]
    fn the_key_comes_from_a_box_text() {
        let pair = KeyPair::generate_unencrypted_keypair().unwrap();
        let text = pair.sk.to_box(None).unwrap().into_string();
        assert!(secret_key(&text, None).is_ok());
        assert!(secret_key("not a key", None).is_err());
    }
}
