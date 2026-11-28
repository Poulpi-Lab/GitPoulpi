//! Which Settings rows each interface shows, and where a per-interface row of
//! the desktop is written.

use gitpoulpi_app::settings::{rows_for, Home, Interface};
use gitpoulpi_app::surface::TERMINAL_ONLY_SETTINGS;
use gitpoulpi_config::DesktopConfig;

#[test]
fn every_per_interface_row_of_the_terminal_ui_has_a_desktop_row_on_the_desktop_key() {
    let tui = rows_for(Interface::Tui);
    let desktop = rows_for(Interface::Desktop);
    let mut checked = 0;
    for row in tui.iter().filter(|r| r.home == Home::PerInterface) {
        let key = row.row.key;
        let twin = desktop
            .iter()
            .find(|d| d.row.key == key)
            .unwrap_or_else(|| panic!("no desktop row for the per-interface key {key}"));
        assert_eq!(twin.key, format!("desktop.{key}"));
        assert_eq!(twin.row.label, row.row.label);
        // The desktop key is one the desktop schema holds (a typo here would
        // be a row that writes nowhere).
        let relative = twin.key.strip_prefix("desktop.").unwrap();
        DesktopConfig::default()
            .get(relative)
            .unwrap_or_else(|e| panic!("{} is not a desktop.toml key: {e}", twin.key));
        checked += 1;
    }
    assert!(checked > 20, "checked only {checked} per-interface rows");
}

#[test]
fn no_terminal_only_key_appears_for_the_desktop() {
    let tui = rows_for(Interface::Tui);
    let desktop = rows_for(Interface::Desktop);
    for (key, _) in TERMINAL_ONLY_SETTINGS {
        assert!(
            tui.iter().any(|r| r.row.key == *key),
            "{key} is a terminal setting and has a row there"
        );
        assert!(
            desktop.iter().all(|r| r.row.key != *key),
            "{key} must not be offered to the desktop"
        );
    }
    // What `tui.toml` alone owns is a terminal matter as well.
    assert!(desktop.iter().all(|r| r.home != Home::Tui));
}

#[test]
fn common_rows_are_shared_by_both_interfaces_under_the_same_key() {
    let tui = rows_for(Interface::Tui);
    let desktop = rows_for(Interface::Desktop);
    let common: Vec<_> = tui.iter().filter(|r| r.home == Home::Common).collect();
    assert!(!common.is_empty());
    for row in common {
        let twin = desktop
            .iter()
            .find(|d| d.row.key == row.row.key)
            .unwrap_or_else(|| panic!("{} is common and missing on the desktop", row.row.key));
        assert_eq!(twin.key, row.row.key);
    }
}

#[test]
fn the_terminal_ui_writes_the_keys_it_always_wrote() {
    for row in rows_for(Interface::Tui) {
        assert_eq!(row.key, row.row.key);
    }
}
