use super::*;

/// Blocks (with a timeout, so a real bug fails the test rather than
/// hanging the suite) until `state`'s background token-presence probe
/// has answered for every `token_ref` - the test equivalent of a few
/// frames ticking by with `poll_token_presence` drained each time.
fn wait_for_token_presence(state: &mut PreferencesState, token_refs: &[String]) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        state.poll_token_presence();
        if token_refs
            .iter()
            .all(|r| state.token_presence.get(r).copied().flatten().is_some())
        {
            return;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "the background token-presence probe never answered for {token_refs:?}"
        );
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

/// A [`TokenStore`] that counts every call it receives, wrapping an
/// in-memory store for the actual data. This is what pins down the
/// bug: opening Preferences' SSH/Integrations section shelled to
/// `git credential fill` from inside row building, once per
/// configured integration *per frame* - with a real credential
/// helper that meant a popped GCM window every redraw.
#[derive(Default)]
struct CountingTokens {
    inner: InMemoryTokens,
    has_calls: std::sync::atomic::AtomicUsize,
    load_calls: std::sync::atomic::AtomicUsize,
}

impl CountingTokens {
    fn has_calls(&self) -> usize {
        self.has_calls.load(std::sync::atomic::Ordering::SeqCst)
    }
}

impl TokenStore for CountingTokens {
    fn store(&self, token_ref: &str, token: &str) -> gitpoulpi_secrets::Result<()> {
        self.inner.store(token_ref, token)
    }

    fn load(&self, token_ref: &str) -> gitpoulpi_secrets::Result<Option<String>> {
        self.load_calls
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        self.inner.load(token_ref)
    }

    fn delete(&self, token_ref: &str) -> gitpoulpi_secrets::Result<()> {
        self.inner.delete(token_ref)
    }

    fn has(&self, token_ref: &str) -> bool {
        self.has_calls
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        self.inner.has(token_ref)
    }
}

fn state_with_counting_tokens(
    dir: &Path,
    integrations: usize,
) -> (PreferencesState, std::sync::Arc<CountingTokens>) {
    let counting = std::sync::Arc::new(CountingTokens::default());
    struct Handle(std::sync::Arc<CountingTokens>);
    impl TokenStore for Handle {
        fn store(&self, token_ref: &str, token: &str) -> gitpoulpi_secrets::Result<()> {
            self.0.store(token_ref, token)
        }
        fn load(&self, token_ref: &str) -> gitpoulpi_secrets::Result<Option<String>> {
            self.0.load(token_ref)
        }
        fn delete(&self, token_ref: &str) -> gitpoulpi_secrets::Result<()> {
            self.0.delete(token_ref)
        }
        fn has(&self, token_ref: &str) -> bool {
            self.0.has(token_ref)
        }
    }
    let config_path = dir.join("config.toml");
    let mut state = PreferencesState::with_parts(
        Config::default(),
        Some(config_path),
        dir,
        RepoState::default(),
        RepoState::path_for(dir, ".gitpoulpi"),
        std::sync::Arc::new(Handle(counting.clone())),
    );
    for _ in 0..integrations {
        add_profile(&mut state, IntegrationKind::Gitlab);
    }
    (state, counting)
}

/// Building the Integrations rows - for several configured
/// integrations, repeatedly, as a redraw loop would - must never
/// touch the token store. This is the actual bug: `fields()` is
/// called from `render_body` on every frame.
#[test]
fn building_integration_rows_never_calls_the_token_store() {
    let dir = tempfile::tempdir().unwrap();
    let (mut state, tokens) = state_with_counting_tokens(dir.path(), 3);
    assert_eq!(state.config.integrations.len(), 3);
    // `state_with_counting_tokens` adds its integrations via
    // `AddGitlab` *after* `with_parts` ran (which saw none), and
    // `AddGitlab` inserts the cache entry directly rather than
    // calling `has` -- so nothing has touched the store yet here, and
    // this reset is just making that explicit rather than relying on
    // it, in case a future probe answer lands in between.
    tokens
        .has_calls
        .store(0, std::sync::atomic::Ordering::SeqCst);

    state.section = PrefSection::ALL
        .iter()
        .position(|s| *s == PrefSection::Integrations)
        .unwrap();
    // Simulate several redraws, as would happen while the screen
    // sits open and the rest of the TUI keeps ticking.
    // The page is drawn (and its sections' rows built) every frame,
    // with the mouse hovering, across every product in the list.
    let mut terminal = ratatui::Terminal::new(ratatui::backend::TestBackend::new(100, 30)).unwrap();
    for frame_no in 0..10 {
        state.integrations.select_kind(frame_no % 4, &state.config);
        state.mouse_pos = Some((40, 8));
        terminal
            .draw(|frame| state.render(frame, frame.area()))
            .unwrap();
        let _ = state.fields();
    }

    assert_eq!(
        tokens.has_calls(),
        0,
        "row building must read the cache, not the token store"
    );
    assert_eq!(
        tokens.load_calls.load(std::sync::atomic::Ordering::SeqCst),
        0,
        "drawing must never `load` a secret either"
    );
}

/// `AddGitlab`/`AddGithub` populate the cache for a freshly-added
/// integration directly (`Some(false)`: it cannot have a token yet),
/// with no store call at all.
#[test]
fn adding_an_integration_marks_its_token_absent_without_a_store_call() {
    let dir = tempfile::tempdir().unwrap();
    let (state, tokens) = state_with_counting_tokens(dir.path(), 3);
    for integration in &state.config.integrations {
        assert_eq!(
            state.token_presence.get(&integration.token_ref).copied(),
            Some(Some(false))
        );
    }
    assert_eq!(tokens.has_calls(), 0);
}

/// Opening Preferences against a
/// config that already has several integrations configured (the
/// normal case: the file the user actually has, not one built up
/// through `AddGitlab` after the fact) must not call `tokens.has`
/// from `open`/`with_parts` itself - that is what made opening the
/// screen slow: each `has` is a `git credential fill` subprocess, and
/// on a Windows machine that is ~2.4s through Git Credential
/// Manager, times the number of configured integrations, all on the
/// input thread before the screen could even draw its first frame.
///
/// Every row starts "checking..." (`token_presence` seeded with
/// `None`) and the answers land only once the background probe
/// (`spawn_token_presence_probe`) is polled - proving both halves of
/// the fix: `with_parts` itself stays synchronous-call-free, and the
/// answers still arrive.
#[test]
fn opening_preferences_with_configured_integrations_never_blocks_on_the_token_store() {
    let dir = tempfile::tempdir().unwrap();
    let counting = std::sync::Arc::new(CountingTokens::default());
    struct Handle(std::sync::Arc<CountingTokens>);
    impl TokenStore for Handle {
        fn store(&self, token_ref: &str, token: &str) -> gitpoulpi_secrets::Result<()> {
            self.0.store(token_ref, token)
        }
        fn load(&self, token_ref: &str) -> gitpoulpi_secrets::Result<Option<String>> {
            self.0.load(token_ref)
        }
        fn delete(&self, token_ref: &str) -> gitpoulpi_secrets::Result<()> {
            self.0.delete(token_ref)
        }
        fn has(&self, token_ref: &str) -> bool {
            // A slow fake store, standing in for the real ~2.4s `git
            // credential fill`: if `with_parts` ever called this
            // synchronously, this test would itself hang for as long
            // as every configured integration's delay, rather than
            // finishing in milliseconds.
            std::thread::sleep(std::time::Duration::from_millis(300));
            self.0.has(token_ref)
        }
    }

    let mut config = Config::default();
    let refs: Vec<String> = ["one", "two", "three"]
        .iter()
        .map(|id| {
            let integration =
                Integration::new((*id).to_string(), IntegrationKind::Gitlab, "https://x");
            let token_ref = integration.token_ref.clone();
            config.integrations.push(integration);
            token_ref
        })
        .collect();
    // Pre-seed one of the three as actually stored, so the probe's
    // answer is falsifiable rather than every ref just happening to
    // land on the same default.
    counting.inner.store(&refs[1], "glpat-secret").unwrap();

    let config_path = dir.path().join("config.toml");
    let started = std::time::Instant::now();
    let mut state = PreferencesState::with_parts(
        config,
        Some(config_path),
        dir.path(),
        RepoState::default(),
        RepoState::path_for(dir.path(), ".gitpoulpi"),
        std::sync::Arc::new(Handle(counting.clone())),
    );
    // Three integrations at 300ms each, called synchronously, would
    // be >=900ms; allow generous slack: the gate runs many test binaries at
    // once, and a loaded box once took 157 ms against the old 120 ms limit.
    assert!(
        started.elapsed() < std::time::Duration::from_millis(500),
        "with_parts took {:?} - it must not call the token store synchronously",
        started.elapsed()
    );
    for token_ref in &refs {
        assert_eq!(
            state.token_presence.get(token_ref).copied(),
            Some(None),
            "every row must start \"checking...\", not a synchronous answer"
        );
    }

    wait_for_token_presence(&mut state, &refs);
    assert_eq!(
        state.token_presence.get(&refs[0]).copied(),
        Some(Some(false))
    );
    assert_eq!(
        state.token_presence.get(&refs[1]).copied(),
        Some(Some(true))
    );
    assert_eq!(
        state.token_presence.get(&refs[2]).copied(),
        Some(Some(false))
    );
}
