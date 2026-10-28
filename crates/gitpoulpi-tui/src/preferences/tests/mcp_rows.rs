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
