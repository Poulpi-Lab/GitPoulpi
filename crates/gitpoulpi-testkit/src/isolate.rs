//! Cuts a test binary off from the developer's real user state.
//!
//! A daemon, CLI or MCP test resolves its settings from the ambient
//! environment (`Settings::resolve_ambient`: `GITPOULPI_CONFIG_DIR`, then the
//! OS config dir), keeps per-repo state in the data dir, and stores tokens
//! through the git credential helper. Without isolation a test reads the
//! maintainer's real `config.toml` and can write into the real data dir or
//! keychain. [`isolate_process_env`] points all of that at one throwaway
//! directory.
//!
//! `set_var` is not safe against a concurrent `getenv` (libtest runs tests on
//! many threads), so a binary calls [`isolate_binary!`](crate::isolate_binary)
//! once at module level: it registers a pre-`main` initializer, which runs
//! single-threaded, before any test thread exists.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

static ROOT: OnceLock<PathBuf> = OnceLock::new();

/// Sets the isolating environment for this process (once; later calls return
/// the same directory) and returns the directory it lives in.
///
/// `GITPOULPI_CONFIG_DIR`, `HOME`, `USERPROFILE`, `APPDATA`, `LOCALAPPDATA` (not on Windows, see below)
/// and the `XDG_*` dirs become subdirectories of it, the system git config is
/// ignored, and the global one is a file whose credential helper is
/// `store --file <dir>/git-credentials`.
pub fn isolate_process_env() -> &'static Path {
    ROOT.get_or_init(|| {
        let base = std::env::temp_dir();
        remove_stale(&base);
        let root = base.join(format!("gitpoulpi-isolate-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let sub = |name: &str| {
            let dir = root.join(name);
            std::fs::create_dir_all(&dir).expect("create isolation dir");
            dir
        };
        let config = sub("config");
        let home = sub("home");
        let gitconfig = root.join("global.gitconfig");
        let creds = root
            .join("git-credentials")
            .to_string_lossy()
            .replace('\\', "/");
        std::fs::write(
            &gitconfig,
            format!(
                "[user]\n\tname = GitPoulpi Test\n\temail = test@gitpoulpi.invalid\n\
                 [credential]\n\thelper = store --file \"{creds}\"\n"
            ),
        )
        .expect("write isolated git config");
        std::env::set_var("GITPOULPI_CONFIG_DIR", &config);
        // Not on Windows: with these redirected, the Git for Windows shell
        // (sh.exe, ssh-keygen) failed to start under the parallel runner
        // ("add_item (\"\\??\\C:\\Program Files\\Git\", \"/\") failed, errno
        // 1") in 8 PTY tests. The git config and credential redirection
        // below is what keeps tests away from the real credential store.
        if cfg!(not(windows)) {
            std::env::set_var("HOME", &home);
            std::env::set_var("USERPROFILE", &home);
            std::env::set_var("APPDATA", sub("appdata"));
            std::env::set_var("LOCALAPPDATA", sub("localappdata"));
        }
        std::env::set_var("XDG_CONFIG_HOME", sub("xdg-config"));
        std::env::set_var("XDG_DATA_HOME", sub("xdg-data"));
        std::env::set_var("XDG_STATE_HOME", sub("xdg-state"));
        std::env::set_var("XDG_CACHE_HOME", sub("xdg-cache"));
        std::env::set_var("GIT_CONFIG_GLOBAL", &gitconfig);
        std::env::set_var("GIT_CONFIG_NOSYSTEM", "1");
        std::env::set_var("GITPOULPI_DESKTOP_NO_HOOKS", "1");
        #[cfg(unix)]
        {
            extern "C" fn cleanup() {
                if let Some(root) = ROOT.get() {
                    let _ = std::fs::remove_dir_all(root);
                }
            }
            // SAFETY: registers a plain `extern "C"` function.
            unsafe { libc::atexit(cleanup) };
        }
        root
    })
}

/// Statics are never dropped, so the directory is removed by an `atexit`
/// hook on Unix. Where that is not available (Windows) or a run was killed,
/// each run sweeps the ones an hour old or more.
fn remove_stale(base: &Path) {
    let Ok(entries) = std::fs::read_dir(base) else {
        return;
    };
    let day = std::time::Duration::from_secs(3600);
    for entry in entries.flatten() {
        let name = entry.file_name();
        if !name.to_string_lossy().starts_with("gitpoulpi-isolate-") {
            continue;
        }
        let old = entry
            .metadata()
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| t.elapsed().ok())
            .is_some_and(|age| age > day);
        if old {
            let _ = std::fs::remove_dir_all(entry.path());
        }
    }
}

/// Isolates the whole test binary before `main` runs: put
/// `gitpoulpi_testkit::isolate_binary!();` at module level, once per test
/// binary. See the module docs for why this is not a call inside each test.
#[macro_export]
macro_rules! isolate_binary {
    () => {
        const _: () = {
            extern "C" fn init() {
                $crate::isolate_process_env();
            }
            #[used]
            #[cfg_attr(
                any(target_os = "linux", target_os = "android", target_os = "freebsd"),
                link_section = ".init_array"
            )]
            #[cfg_attr(target_os = "macos", link_section = "__DATA,__mod_init_func")]
            #[cfg_attr(windows, link_section = ".CRT$XCU")]
            static INIT: extern "C" fn() = init;
        };
    };
}
