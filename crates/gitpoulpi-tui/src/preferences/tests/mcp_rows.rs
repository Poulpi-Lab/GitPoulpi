//! Settings > Agents & MCP: the MCP server's switch (`mcp.enabled`) and its
//! port (a virtual key that edits the port of `mcp.bind`, keeping the host).

use super::*;
use crate::preferences::palette_settings::{rows, SettingKind};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

fn press(st: &mut PreferencesState, code: KeyCode) {
    st.on_key(KeyEvent::new(code, KeyModifiers::NONE));
}

fn on_row(st: &mut PreferencesState, key: &'static str) {
    go(st, PrefSection::AgentsMcp);
    st.focus = PrefFocus::Fields;
    st.field = st
        .fields()
        .iter()
        .position(|f| f.target == Target::Global(key))
        .unwrap_or_else(|| panic!("no row {key}"));
}

fn stored(dir: &Path) -> String {
    std::fs::read_to_string(dir.join("config.toml")).unwrap_or_default()
}

fn type_port(st: &mut PreferencesState, text: &str) {
    press(st, KeyCode::Enter);
    for _ in 0..8 {
        press(st, KeyCode::Backspace);
    }
    for c in text.chars() {
        press(st, KeyCode::Char(c));
    }
    press(st, KeyCode::Enter);
}

#[test]
fn the_server_row_is_a_toggle_that_writes_mcp_enabled() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = state(dir.path());
    on_row(&mut st, "mcp.enabled");
    assert!(matches!(
        st.fields()[st.field].control,
        Control::Toggle(true)
    ));
    press(&mut st, KeyCode::Enter);
    assert!(
        stored(dir.path()).contains("enabled = false"),
        "{}",
        stored(dir.path())
    );
    press(&mut st, KeyCode::Enter);
    assert!(st.config.mcp.enabled);
    assert!(stored(dir.path()).contains("enabled = true"));
}

#[test]
fn the_port_row_is_editable_while_the_server_is_off() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = state(dir.path());
    st.config.mcp.enabled = false;
    go(&mut st, PrefSection::AgentsMcp);
    let port = st
        .fields()
        .into_iter()
        .find(|f| f.target == Target::Global("mcp.port"))
        .unwrap();
    assert!(port.coming.is_none() && port.selectable());
    on_row(&mut st, "mcp.port");
    type_port(&mut st, "9200");
    assert_eq!(st.config.mcp.bind, "127.0.0.1:9200");
}

#[test]
fn a_port_replaces_only_the_port_of_the_bind() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = state(dir.path());
    on_row(&mut st, "mcp.port");
    assert_eq!(st.fields()[st.field].value_text(), "8808");
    type_port(&mut st, "9100");
    assert_eq!(st.config.mcp.bind, "127.0.0.1:9100");
    assert!(stored(dir.path()).contains("bind = \"127.0.0.1:9100\""));

    // A custom host stays.
    st.config.mcp.bind = "[::1]:7000".into();
    on_row(&mut st, "mcp.port");
    assert_eq!(st.fields()[st.field].value_text(), "7000");
    type_port(&mut st, "7001");
    assert_eq!(st.config.mcp.bind, "[::1]:7001");

    // Empty and the legacy "stdio" mean the built-in address.
    for legacy in ["", "stdio"] {
        st.config.mcp.bind = legacy.into();
        on_row(&mut st, "mcp.port");
        assert_eq!(st.fields()[st.field].value_text(), "8808");
        type_port(&mut st, "8900");
        assert_eq!(st.config.mcp.bind, "127.0.0.1:8900");
    }
}

#[test]
fn a_bad_port_is_refused_and_writes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = state(dir.path());
    on_row(&mut st, "mcp.port");
    for bad in ["0", "65536", "abc"] {
        type_port(&mut st, bad);
        assert!(st.edit.is_some() && st.edit_invalid.is_some(), "{bad}");
        press(&mut st, KeyCode::Esc);
    }
    assert!(
        !stored(dir.path()).contains("bind"),
        "{}",
        stored(dir.path())
    );
    type_port(&mut st, "65535");
    assert_eq!(st.config.mcp.bind, "127.0.0.1:65535");
}

#[test]
fn an_unreadable_bind_is_shown_and_still_takes_a_port() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = state(dir.path());
    st.config.mcp.bind = "localhost".into();
    go(&mut st, PrefSection::AgentsMcp);
    let port = st
        .fields()
        .into_iter()
        .find(|f| f.target == Target::Global("mcp.port"))
        .unwrap();
    assert!(port.warning.as_deref().unwrap().contains("localhost"));
    on_row(&mut st, "mcp.port");
    type_port(&mut st, "8000");
    assert_eq!(st.config.mcp.bind, "localhost:8000");
}

#[test]
fn both_rows_are_in_the_palette() {
    let rows = rows();
    let kind = |key: &str| rows.iter().find(|r| r.key == key).map(|r| r.kind.clone());
    assert_eq!(kind("mcp.enabled"), Some(SettingKind::Toggle));
    assert_eq!(
        kind("mcp.port"),
        Some(SettingKind::Number {
            min: 1,
            max: 65_535
        })
    );
}

// ---- the HTTP endpoint's security rows (`mcp_security`) ----

fn with_secrets(st: &mut PreferencesState, dir: &Path) -> ConfigPaths {
    let paths = ConfigPaths::for_dir(dir.join("secrets"));
    st.mcp_sec.paths = Some(paths.clone());
    st.mcp_sec.token = gitpoulpi_config::mcp_secrets::read_token(&paths);
    paths
}

/// The whole page on one tall screen: the security rows sit below the fold of
/// the usual 50 lines.
fn draw_tall(state: &mut PreferencesState) -> String {
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    let mut terminal = Terminal::new(TestBackend::new(110, 200)).unwrap();
    terminal
        .draw(|frame| state.render(frame, frame.area()))
        .unwrap();
    let buffer = terminal.backend().buffer();
    (0..200)
        .map(|y| {
            (0..110)
                .map(|x| buffer[(x, y)].symbol())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn field_of(st: &PreferencesState, key: &'static str) -> Field {
    st.fields()
        .into_iter()
        .find(|f| f.target == Target::Global(key))
        .unwrap_or_else(|| panic!("no row {key}"))
}

#[test]
fn the_page_has_every_security_row_and_authentication_defaults_on() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = state(dir.path());
    with_secrets(&mut st, dir.path());
    go(&mut st, PrefSection::AgentsMcp);
    for key in [
        "mcp.http_auth",
        "mcp.token",
        "mcp.tls",
        "mcp.tls_cert",
        "mcp.tls_key",
        "mcp.behind_proxy",
    ] {
        field_of(&st, key);
    }
    assert!(matches!(
        field_of(&st, "mcp.http_auth").control,
        Control::Toggle(true)
    ));
    // The fingerprint row follows the key path.
    let labels: Vec<String> = st.fields().iter().map(|f| f.label.clone()).collect();
    let key_at = labels.iter().position(|l| l == "Key path").unwrap();
    assert_eq!(labels[key_at + 1], "Certificate fingerprint");
    let screen = draw_tall(&mut st);
    assert!(screen.contains("MCP HTTP endpoint security"), "{screen}");
    assert!(screen.contains("Access token"), "{screen}");
}

#[test]
fn turning_authentication_off_or_trusting_a_proxy_shows_a_danger_warning() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = state(dir.path());
    with_secrets(&mut st, dir.path());
    go(&mut st, PrefSection::AgentsMcp);
    assert!(field_of(&st, "mcp.http_auth").warning.is_none());
    assert!(field_of(&st, "mcp.behind_proxy").warning.is_none());

    on_row(&mut st, "mcp.http_auth");
    press(&mut st, KeyCode::Enter);
    assert!(stored(dir.path()).contains("http_auth = false"));
    let auth = field_of(&st, "mcp.http_auth");
    assert!(auth.danger && auth.warning.is_some());

    on_row(&mut st, "mcp.behind_proxy");
    press(&mut st, KeyCode::Enter);
    assert!(stored(dir.path()).contains("behind_proxy = true"));
    let proxy = field_of(&st, "mcp.behind_proxy");
    assert!(proxy.danger && proxy.warning.is_some());
    let screen = draw_tall(&mut st);
    assert!(screen.contains("Authentication is off"), "{screen}");
}

#[test]
fn the_tls_choice_and_the_paths_write_their_keys() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = state(dir.path());
    with_secrets(&mut st, dir.path());
    on_row(&mut st, "mcp.tls");
    st.choose_next();
    assert!(
        stored(dir.path()).contains("tls = \"self_signed\""),
        "{}",
        stored(dir.path())
    );
    on_row(&mut st, "mcp.tls_cert");
    press(&mut st, KeyCode::Enter);
    for c in "/etc/c.pem".chars() {
        press(&mut st, KeyCode::Char(c));
    }
    press(&mut st, KeyCode::Enter);
    assert_eq!(st.config.mcp.tls_cert, "/etc/c.pem");
}

/// The token is masked until its value button is pressed; Create makes the
/// first one without asking; Rotate asks first and replaces it.
#[test]
fn the_token_row_masks_shows_and_rotates_with_the_keyboard() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = state(dir.path());
    let paths = with_secrets(&mut st, dir.path());
    on_row(&mut st, "mcp.token");
    assert!(st.mcp_sec.token.is_none());
    let screen = draw_tall(&mut st);
    assert!(screen.contains("(none yet)"), "{screen}");

    // Create is the third button: Right, Right, Enter. No confirmation.
    press(&mut st, KeyCode::Right);
    press(&mut st, KeyCode::Right);
    press(&mut st, KeyCode::Enter);
    assert!(st.confirm_action.is_none());
    let first = gitpoulpi_config::mcp_secrets::read_token(&paths).expect("created");
    assert_eq!(st.mcp_sec.token.as_deref(), Some(first.as_str()));

    // Masked: the full token is nowhere on screen; Show reveals it.
    st.ai.chip = 0;
    let masked = draw_tall(&mut st);
    assert!(!masked.contains(&first), "{masked}");
    assert!(masked.contains(&gitpoulpi_config::mcp_secrets::token_hint(&first)));
    press(&mut st, KeyCode::Enter);
    let shown = draw_tall(&mut st);
    assert!(shown.contains(&first), "{shown}");
    press(&mut st, KeyCode::Enter);
    assert!(!draw_tall(&mut st).contains(&first), "Enter hides it again");

    // Rotate asks, Cancel keeps the token, confirming replaces it.
    st.ai.chip = 2;
    press(&mut st, KeyCode::Enter);
    assert!(st.confirm_action.is_some(), "rotating asks first");
    press(&mut st, KeyCode::Enter);
    assert_eq!(
        gitpoulpi_config::mcp_secrets::read_token(&paths).as_deref(),
        Some(first.as_str()),
        "Cancel keeps it"
    );
    press(&mut st, KeyCode::Enter);
    press(&mut st, KeyCode::Right);
    press(&mut st, KeyCode::Enter);
    let second = gitpoulpi_config::mcp_secrets::read_token(&paths).unwrap();
    assert_ne!(second, first);
    assert_eq!(st.mcp_sec.token.as_deref(), Some(second.as_str()));
}

/// Copy puts the token on the clipboard and the status line never echoes it.
#[test]
fn copying_the_token_does_not_echo_it() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = state(dir.path());
    let paths = with_secrets(&mut st, dir.path());
    let (token, _) = gitpoulpi_config::mcp_secrets::ensure_token(&paths).unwrap();
    st.mcp_sec.token = Some(token.clone());
    st.mcp_button(super::super::mcp_security::McpButton::Copy);
    let message = st.take_message().unwrap();
    assert!(!message.contains(&token[..8]), "{message}");
    assert!(message.contains("copied"), "{message}");
}

#[test]
fn the_fingerprint_row_follows_the_tls_mode() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = state(dir.path());
    let paths = with_secrets(&mut st, dir.path());
    go(&mut st, PrefSection::AgentsMcp);
    let text = |st: &PreferencesState| field_of_label(st, "Certificate fingerprint");
    assert!(text(&st).contains("TLS is off"));
    st.config.mcp.tls = gitpoulpi_config::McpTls::SelfSigned;
    assert!(text(&st).contains("first starts"));
    let (cert, _) = gitpoulpi_mcp_pair(&paths);
    assert!(text(&st).contains(&gitpoulpi_config::mcp_secrets::fingerprint_of_file(&cert).unwrap()));
}

fn field_of_label(st: &PreferencesState, label: &str) -> String {
    st.fields()
        .into_iter()
        .find(|f| f.label == label)
        .unwrap_or_else(|| panic!("no row {label}"))
        .value_text()
}

/// A certificate in the place the server writes the self-signed one.
fn gitpoulpi_mcp_pair(paths: &ConfigPaths) -> (PathBuf, PathBuf) {
    let cert = gitpoulpi_config::mcp_secrets::self_signed_cert(paths);
    std::fs::create_dir_all(cert.parent().unwrap()).unwrap();
    std::fs::write(
        &cert,
        "-----BEGIN CERTIFICATE-----\nYWJj\n-----END CERTIFICATE-----\n",
    )
    .unwrap();
    (cert, gitpoulpi_config::mcp_secrets::self_signed_key(paths))
}

#[test]
fn the_palette_lists_the_security_rows() {
    let rows = rows();
    for key in [
        "mcp.http_auth",
        "mcp.token",
        "mcp.tls",
        "mcp.tls_cert",
        "mcp.tls_key",
        "mcp.behind_proxy",
    ] {
        assert!(
            rows.iter().any(|r| r.key == key),
            "{key} has no palette row"
        );
    }
    let token = rows.iter().find(|r| r.key == "mcp.token").unwrap();
    assert_eq!(token.kind, SettingKind::Page);
}

/// Presses the Suggest a free port row and waits for the thread's answer.
fn suggest(st: &mut PreferencesState) {
    on_row(st, "mcp.suggest_port");
    press(st, KeyCode::Enter);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while st.mcp_sec.suggest.is_some() {
        assert!(std::time::Instant::now() < deadline, "no answer");
        st.poll_token_presence();
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
}

#[test]
fn suggest_a_free_port_saves_this_installations_own_bind_and_keeps_the_host() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = state(dir.path());
    st.config.mcp.bind = "127.0.0.1:8808".into();
    suggest(&mut st);
    let file = stored(dir.path());
    assert!(file.contains("mcp_bind = \"127.0.0.1:"), "{file}");
    assert!(!file.contains("8808\""), "{file}");
    // The port row now shows that installation bind.
    assert_ne!(st.mcp_bind_in_force(), "127.0.0.1:8808");
    let shown = st
        .fields()
        .into_iter()
        .find(|f| f.target == Target::Global("mcp.port"))
        .unwrap();
    let (_, port) = st.mcp_in_force().bind_parts();
    assert!(matches!(shown.control, Control::Number { value, .. } if Some(value as u16) == port));
    assert!(st.take_message().unwrap().contains("MCP bind set to"));
}

#[test]
fn the_primary_without_an_override_writes_mcp_bind() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = state(dir.path());
    let this = crate::preferences::launcher_install::fake_current(true);
    st.config.installation.primary = this.config.installation.primary.clone();
    st.set_this(this);
    suggest(&mut st);
    let file = stored(dir.path());
    assert!(file.contains("[mcp]") && file.contains("bind = "), "{file}");
    assert!(!file.contains("mcp_bind"), "{file}");
}
