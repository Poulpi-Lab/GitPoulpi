//! The settings shown as a number and a unit on one line (`Control::Quantity`,
//! see `quantity.rs`) besides the log retention: the two minute timers, the
//! log file size, the image size limit and the two history rewrite script
//! timeouts. Each keeps its own key and base unit (minutes, MiB,
//! milliseconds, seconds); a `<name>_unit` key only records the unit it is
//! shown in (`gitpoulpi_config::DisplayUnit`).

use super::*;
use gitpoulpi_config::DisplayUnit;
use gitpoulpi_i18n::msg;
use msg::settings_general as g;

/// What differs between the rows.
struct Row {
    label: &'static str,
    help: Option<&'static str>,
    key: &'static str,
    unit_key: &'static str,
    /// The unit the stored number is in.
    base: DisplayUnit,
    /// The units offered, smallest first.
    choices: &'static [DisplayUnit],
    /// The most the stored number may be.
    max_total: i64,
    /// Whether `0` is allowed (it turns a timer off).
    allow_zero: bool,
    /// The stored number as the config reads it back: count and unit.
    shown: (u64, DisplayUnit),
}

impl Row {
    fn field(self) -> Field {
        let (n, current) = self.shown;
        let units: Vec<QuantityUnit> = self
            .choices
            .iter()
            .map(|u| QuantityUnit {
                label: unit_word(*u, n),
                key: u.as_str(),
                weight: u.weight_in(self.base).unwrap_or(1) as i64,
            })
            .collect();
        let unit = self.choices.iter().position(|u| *u == current).unwrap_or(0);
        let field = Field::global(
            self.label,
            self.key,
            Control::Quantity {
                value: n as i64,
                unit,
                units,
                max_total: self.max_total,
                allow_zero: self.allow_zero,
                unit_key: self.unit_key,
            },
        );
        match self.help {
            Some(help) => field.with_help(help),
            None => field,
        }
    }
}

impl PreferencesState {
    /// `general.auto_fetch_interval_minutes`: minutes or hours, 0 is off.
    pub(super) fn auto_fetch_field(&self) -> Field {
        Row {
            label: g::AUTO_FETCH_INTERVAL_MINUTES.get(),
            help: Some(g::AUTO_FETCH_INTERVAL_MINUTES_HELP.get()),
            key: "general.auto_fetch_interval_minutes",
            unit_key: "general.auto_fetch_interval_unit",
            base: DisplayUnit::Minutes,
            choices: gitpoulpi_config::MINUTES_OR_HOURS,
            max_total: i64::from(gitpoulpi_config::MAX_AUTO_FETCH_INTERVAL_MINUTES),
            allow_zero: true,
            shown: self.config.general.auto_fetch_quantity(),
        }
        .field()
    }

    /// `general.hosting_refresh_minutes`: minutes or hours, 0 is off.
    pub(super) fn hosting_refresh_field(&self) -> Field {
        Row {
            label: g::HOSTING_REFRESH_MINUTES.get(),
            help: Some(g::HOSTING_REFRESH_MINUTES_HELP.get()),
            key: "general.hosting_refresh_minutes",
            unit_key: "general.hosting_refresh_unit",
            base: DisplayUnit::Minutes,
            choices: gitpoulpi_config::MINUTES_OR_HOURS,
            max_total: i64::from(gitpoulpi_config::MAX_HOSTING_REFRESH_MINUTES),
            allow_zero: true,
            shown: self.config.general.hosting_refresh_quantity(),
        }
        .field()
    }

    /// `log.max_file_mb`: MiB or GiB.
    pub(super) fn log_max_file_field(&self) -> Field {
        Row {
            label: g::LOG_MAX_FILE_MB.get(),
            help: None,
            key: "log.max_file_mb",
            unit_key: "log.max_file_unit",
            base: DisplayUnit::Mib,
            choices: gitpoulpi_config::MIB_OR_GIB,
            max_total: i64::from(gitpoulpi_config::MAX_LOG_MAX_FILE_MB),
            allow_zero: false,
            shown: self.config.log.max_file_quantity(),
        }
        .field()
    }

    /// `ui.image_view_max_mib`: MiB or GiB.
    pub(super) fn image_max_field(&self) -> Field {
        Row {
            label: msg::settings_images::MAX_MIB.get(),
            help: Some(msg::settings_images::MAX_MIB_HELP.get()),
            key: "ui.image_view_max_mib",
            unit_key: "ui.image_view_max_unit",
            base: DisplayUnit::Mib,
            choices: gitpoulpi_config::MIB_OR_GIB,
            max_total: i64::from(gitpoulpi_config::MAX_IMAGE_VIEW_MAX_MIB),
            allow_zero: false,
            shown: self.config.ui.image_view_max_quantity(),
        }
        .field()
    }

    /// `rewrite.script_timeout_ms`: milliseconds or seconds.
    pub(super) fn script_timeout_field(&self) -> Field {
        Row {
            label: g::SCRIPT_TIMEOUT.get(),
            help: Some(g::SCRIPT_TIMEOUT_HELP.get()),
            key: "rewrite.script_timeout_ms",
            unit_key: "rewrite.script_timeout_unit",
            base: DisplayUnit::Milliseconds,
            choices: gitpoulpi_config::SCRIPT_TIMEOUT_UNITS,
            max_total: gitpoulpi_config::MAX_SCRIPT_TIMEOUT_MS as i64,
            allow_zero: false,
            shown: self.config.rewrite.script_timeout_quantity(),
        }
        .field()
    }

    /// `rewrite.script_total_timeout_s`: seconds, minutes or hours.
    pub(super) fn script_total_timeout_field(&self) -> Field {
        Row {
            label: g::SCRIPT_TOTAL_TIMEOUT.get(),
            help: Some(g::SCRIPT_TOTAL_TIMEOUT_HELP.get()),
            key: "rewrite.script_total_timeout_s",
            unit_key: "rewrite.script_total_timeout_unit",
            base: DisplayUnit::Seconds,
            choices: gitpoulpi_config::SCRIPT_TOTAL_TIMEOUT_UNITS,
            max_total: gitpoulpi_config::MAX_SCRIPT_TOTAL_TIMEOUT_S as i64,
            allow_zero: false,
            shown: self.config.rewrite.script_total_timeout_quantity(),
        }
        .field()
    }
}

/// The unit's name for `n` ("minute", "minutes", "MiB"): the catalog's plural
/// text with the number cut out, so the dropdown shows only the word.
fn unit_word(unit: DisplayUnit, n: u64) -> String {
    let text = match unit {
        DisplayUnit::Milliseconds => g::unit_milliseconds(n),
        DisplayUnit::Seconds => g::unit_seconds(n),
        DisplayUnit::Hours => g::unit_hours(n),
        DisplayUnit::Mib => g::UNIT_MIB.get().to_string(),
        DisplayUnit::Gib => g::UNIT_GIB.get().to_string(),
        DisplayUnit::Minutes | DisplayUnit::Auto => g::unit_minutes(n),
    };
    let word = text.replacen(&n.to_string(), "", 1);
    let word = word.trim();
    if word.is_empty() {
        text
    } else {
        word.to_string()
    }
}
