//! The number-and-unit settings: the stored number keeps its key and base
//! unit (an older file loads as before), the unit key only records how it
//! is shown, `auto` picks the largest unit that divides evenly.

use gitpoulpi_config::{Config, DisplayUnit as U};

fn load(toml: &str) -> Config {
    Config::from_toml_str(toml).unwrap().0
}

#[test]
fn auto_fetch_interval_reads_in_minutes_or_hours() {
    let c = load("[general]\nauto_fetch_interval_minutes = 60");
    assert_eq!(c.general.auto_fetch_interval_minutes, 60);
    assert_eq!(c.general.auto_fetch_interval_unit, U::Auto);
    assert_eq!(c.general.auto_fetch_quantity(), (1, U::Hours));
    let c = load("[general]\nauto_fetch_interval_minutes = 45");
    assert_eq!(c.general.auto_fetch_quantity(), (45, U::Minutes));
    let c =
        load("[general]\nauto_fetch_interval_minutes = 0\nauto_fetch_interval_unit = \"hours\"");
    assert_eq!(c.general.auto_fetch_quantity(), (0, U::Hours));
    // Above the range: shown at the maximum the setting accepts.
    let c = load("[general]\nauto_fetch_interval_minutes = 9999");
    assert_eq!(c.general.auto_fetch_quantity(), (1, U::Hours));
}

#[test]
fn hosting_refresh_reads_in_minutes_or_hours() {
    let c = load("[general]\nhosting_refresh_minutes = 120");
    assert_eq!(c.general.hosting_refresh_quantity(), (2, U::Hours));
    let c = load("[general]\nhosting_refresh_minutes = 90\nhosting_refresh_unit = \"hours\"");
    assert_eq!(c.general.hosting_refresh_quantity(), (90, U::Minutes));
    assert_eq!(
        Config::default().general.hosting_refresh_quantity(),
        (5, U::Minutes)
    );
}

#[test]
fn script_timeout_reads_in_milliseconds_or_seconds() {
    let c = load("[rewrite]\nscript_timeout_ms = 5000");
    assert_eq!(c.rewrite.script_timeout_quantity(), (5, U::Seconds));
    let c = load("[rewrite]\nscript_timeout_ms = 1500");
    assert_eq!(c.rewrite.script_timeout_quantity(), (1500, U::Milliseconds));
    let c = load("[rewrite]\nscript_timeout_ms = 2000\nscript_timeout_unit = \"milliseconds\"");
    assert_eq!(c.rewrite.script_timeout_quantity(), (2000, U::Milliseconds));
    // 0 reads as the default, as it does at run time.
    let c = load("[rewrite]\nscript_timeout_ms = 0");
    assert_eq!(c.rewrite.script_timeout_quantity(), (5, U::Seconds));
}

#[test]
fn script_total_timeout_reads_in_seconds_minutes_or_hours() {
    let c = load("[rewrite]\nscript_total_timeout_s = 3600");
    assert_eq!(c.rewrite.script_total_timeout_quantity(), (1, U::Hours));
    let c = load("[rewrite]\nscript_total_timeout_s = 90");
    assert_eq!(c.rewrite.script_total_timeout_quantity(), (90, U::Seconds));
    let c =
        load("[rewrite]\nscript_total_timeout_s = 600\nscript_total_timeout_unit = \"seconds\"");
    assert_eq!(c.rewrite.script_total_timeout_quantity(), (600, U::Seconds));
    let c = load("[rewrite]\nscript_total_timeout_s = 0");
    assert_eq!(c.rewrite.script_total_timeout_quantity(), (1, U::Hours));
}

#[test]
fn log_file_size_reads_in_mib_or_gib() {
    let c = load("[log]\nmax_file_mb = 1024");
    assert_eq!(c.log.max_file_mb, 1024);
    assert_eq!(c.log.max_file_quantity(), (1, U::Gib));
    assert_eq!(Config::default().log.max_file_quantity(), (5, U::Mib));
    let c = load("[log]\nmax_file_mb = 512\nmax_file_unit = \"gib\"");
    assert_eq!(c.log.max_file_quantity(), (512, U::Mib));
    // The byte cap still comes from the stored MiB.
    assert_eq!(load("[log]\nmax_file_mb = 2").log.max_file_bytes(), 2 << 20);
}

#[test]
fn image_size_limit_reads_in_mib_or_gib() {
    let c = load("[ui]\nimage_view_max_mib = 32");
    assert_eq!(c.ui.image_view_max_quantity(), (32, U::Mib));
    let c = load("[ui]\nimage_view_max_mib = 1024");
    assert_eq!(c.ui.image_view_max_quantity(), (1, U::Gib));
    let c = load("[ui]\nimage_view_max_mib = 2048");
    assert_eq!(c.ui.image_view_max_quantity(), (1, U::Gib));
}

#[test]
fn a_unit_key_survives_a_round_trip_and_an_unknown_one_is_refused() {
    let mut cfg = Config::default();
    cfg.set("general.auto_fetch_interval_minutes", "120")
        .unwrap();
    cfg.set("general.auto_fetch_interval_unit", "hours")
        .unwrap();
    let (again, _) = Config::from_toml_str(&toml::to_string(&cfg).unwrap()).unwrap();
    assert_eq!(again.general.auto_fetch_interval_unit, U::Hours);
    assert_eq!(again.general.auto_fetch_interval_minutes, 120);
    assert!(cfg
        .set("general.auto_fetch_interval_unit", "fortnights")
        .is_err());
}
