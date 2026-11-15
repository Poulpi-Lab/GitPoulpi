//! `general.hosting_post_as`: the "Post as" name of a bot token, settable and
//! readable like every other key, and live.

use gitpoulpi_config::{Config, LIVE_KEYS};

#[test]
fn post_as_is_empty_by_default_and_settable_by_key() {
    let mut config = Config::default();
    assert_eq!(config.general.hosting_post_as, "");
    assert_eq!(
        config.get("general.hosting_post_as").unwrap(),
        serde_json::json!("")
    );
    config
        .set("general.hosting_post_as", "poulpi-lab[bot]")
        .unwrap();
    assert_eq!(config.general.hosting_post_as, "poulpi-lab[bot]");
    assert_eq!(
        config.get("general.hosting_post_as").unwrap(),
        serde_json::json!("poulpi-lab[bot]")
    );
}

#[test]
fn post_as_survives_a_toml_round_trip_and_is_a_live_key() {
    let mut config = Config::default();
    config.general.hosting_post_as = "mine[bot]".into();
    let raw = toml::to_string_pretty(&config).unwrap();
    let (back, _) = Config::from_toml_str(&raw).unwrap();
    assert_eq!(back.general.hosting_post_as, "mine[bot]");
    assert!(LIVE_KEYS.contains(&"general.hosting_post_as"));
}
