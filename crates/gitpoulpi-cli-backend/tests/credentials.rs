//! `credentials.https_mode` at the git boundary: against a local HTTP
//! server that always answers 401, a configured credential helper is asked
//! in `Helper` mode and never consulted in `None` and `Prompt` mode. The
//! mode is process-wide, so everything runs in one test.
#![cfg(unix)]

use std::io::{Read, Write};
use std::net::TcpListener;

use gitpoulpi_cli_backend::{set_credential_mode, CredentialMode, SystemGitBackend};
use gitpoulpi_core::domain::FetchOptions;
use gitpoulpi_core::GitBackend;
use gitpoulpi_testkit::FixtureRepoBuilder;

fn serve_401() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            std::thread::spawn(move || {
                let mut buf = [0u8; 4096];
                let _ = stream.read(&mut buf);
                let _ = stream.write_all(
                    b"HTTP/1.1 401 Unauthorized\r\nWWW-Authenticate: Basic realm=\"x\"\r\n\
                      Content-Length: 0\r\nConnection: close\r\n\r\n",
                );
            });
        }
    });
    port
}

#[tokio::test]
async fn the_mode_decides_whether_the_credential_helper_is_consulted() {
    // Never let a failing test hang on a terminal prompt.
    std::env::set_var("GIT_TERMINAL_PROMPT", "0");
    let port = serve_401();
    let marker = tempfile::NamedTempFile::new().unwrap();
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    repo.git(&[
        "remote",
        "add",
        "origin",
        &format!("http://127.0.0.1:{port}/r.git"),
    ]);
    repo.git(&[
        "config",
        "credential.helper",
        &format!(
            "!f() {{ echo \"$1\" >> {}; if [ \"$1\" = get ]; then echo username=u; echo password=p; fi; }}; f",
            marker.path().display()
        ),
    ]);
    let backend = SystemGitBackend::new();
    let consulted = || {
        std::fs::read_to_string(marker.path())
            .unwrap()
            .lines()
            .any(|l| l == "get")
    };

    for (mode, expect) in [
        (CredentialMode::None, false),
        (CredentialMode::Prompt, false),
        (CredentialMode::Helper, true),
    ] {
        std::fs::write(marker.path(), "").unwrap();
        set_credential_mode(mode);
        let result = backend
            .fetch(&repo.repo_ref(), "origin", &FetchOptions::default())
            .await;
        assert!(result.is_err(), "{mode:?}: the server always says 401");
        assert_eq!(consulted(), expect, "{mode:?}");
    }
    set_credential_mode(CredentialMode::Helper);
}
