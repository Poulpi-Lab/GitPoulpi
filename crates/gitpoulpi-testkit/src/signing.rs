//! A throwaway release key pair for the update tests: the production code
//! only ever takes the public keys as a parameter (`release_keys()` in
//! production), so a test generates a pair, hands the public half to the code
//! under test and signs its fixtures with the secret half, exactly the way
//! `xtask sign` signs a release (trusted comment `file:<name> version:<v>`).

use std::io::Cursor;

use minisign::{KeyPair, PublicKey, SecretKey};

/// A generated key pair.
pub struct TestSigner {
    pk: PublicKey,
    sk: SecretKey,
}

impl Default for TestSigner {
    fn default() -> Self {
        Self::new()
    }
}

impl TestSigner {
    pub fn new() -> Self {
        let KeyPair { pk, sk } =
            KeyPair::generate_unencrypted_keypair().expect("a key pair generates");
        Self { pk, sk }
    }

    /// The text of the public key file (`minisign.pub`).
    pub fn public_key_text(&self) -> String {
        self.pk.to_box().expect("a public key boxes").into_string()
    }

    /// The `.minisig` text of `bytes`, with the trusted comment the release
    /// workflow writes.
    pub fn sign(&self, name: &str, version: &str, bytes: &[u8]) -> String {
        self.sign_with_comment(&format!("file:{name} version:{version}"), bytes)
    }

    /// A signature with any trusted comment (for the refusal tests).
    pub fn sign_with_comment(&self, comment: &str, bytes: &[u8]) -> String {
        minisign::sign(
            None,
            &self.sk,
            Cursor::new(bytes),
            Some(comment),
            Some("signature from a test key"),
        )
        .expect("a signature is made")
        .into_string()
    }
}
