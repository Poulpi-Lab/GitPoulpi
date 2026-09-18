//! The persisted column layout (`LogColumnPrefs`) and column config names.

use super::*;

#[test]
fn config_names_round_trip_and_unknown_is_none() {
    for c in LogColumn::ALL {
        assert_eq!(LogColumn::from_config_name(c.config_name()), Some(c));
    }
    assert_eq!(LogColumn::from_config_name("nope"), None);
}

#[test]
fn prefs_ignore_unknown_names_and_complete_the_order() {
    let mut ui = gitpoulpi_config::UiConfig {
        log_columns_hidden: vec!["sha".into(), "bogus".into(), "message".into()],
        log_columns_order: vec!["date".into(), "bogus".into(), "ref".into()],
        ..Default::default()
    };
    ui.log_column_widths.insert("author".into(), 20);
    ui.log_column_widths.insert("bogus".into(), 5);
    let p = LogColumnPrefs::from_ui(&ui);
    // `sha`, plus `changes`, hidden until the user decides.
    assert_eq!(p.hidden.len(), 2);
    assert!(p.hidden.contains(&LogColumn::Sha));
    assert!(p.hidden.contains(&LogColumn::Changes));
    assert_eq!(p.order.len(), 7);
    assert_eq!(&p.order[..2], &[LogColumn::Date, LogColumn::Ref]);
    assert_eq!(p.widths.get(&LogColumn::Author), Some(&20));
    assert_eq!(p.widths.len(), 1);
}

/// A `Message` width stored by an older version (it pinned the column and
/// made a resize leave it behind) is dropped at load; cell widths of the
/// other columns are kept verbatim.
#[test]
fn a_stored_message_width_is_dropped_at_load() {
    let mut ui = gitpoulpi_config::UiConfig::default();
    ui.log_column_widths.insert("message".into(), 40);
    ui.log_column_widths.insert("graph".into(), 12);
    let p = LogColumnPrefs::from_ui(&ui);
    assert_eq!(p.widths.get(&LogColumn::Message), None);
    assert_eq!(p.widths.get(&LogColumn::Graph), Some(&12));
}
