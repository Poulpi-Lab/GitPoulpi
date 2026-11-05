//! The Log column layout persisted in the `[ui]` config table.

use super::*;

/// A column-name -> width map as written to `config.toml`.
pub(crate) type NameWidths = std::collections::BTreeMap<String, u16>;

/// The Log column layout as persisted in `[ui]` (`log_columns_hidden`,
/// `log_columns_order`, `log_column_widths`): global, shared by every tab.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LogColumnPrefs {
    pub(crate) order: Vec<LogColumn>,
    pub(crate) hidden: std::collections::HashSet<LogColumn>,
    pub(crate) widths: std::collections::HashMap<LogColumn, u16>,
    /// The `scroll` layout's widths (`ui.log_column_widths_scroll`): apart
    /// from `widths`, and `Message` has one here.
    pub(crate) scroll_widths: std::collections::HashMap<LogColumn, u16>,
}

impl LogColumnPrefs {
    /// Reads the config's name lists; unknown names are ignored, columns
    /// missing from the stored order are appended in default order, and
    /// `Message` is never hidden.
    pub(crate) fn from_ui(ui: &gitpoulpi_config::UiConfig) -> Self {
        let mut order: Vec<LogColumn> = Vec::new();
        for c in ui
            .log_columns_order
            .iter()
            .filter_map(|n| LogColumn::from_config_name(n))
        {
            if !order.contains(&c) {
                order.push(c);
            }
        }
        for c in LogColumn::ALL {
            if !order.contains(&c) {
                order.push(c);
            }
        }
        let mut hidden: std::collections::HashSet<LogColumn> = ui
            .log_columns_hidden
            .iter()
            .filter_map(|n| LogColumn::from_config_name(n))
            .filter(|c| c.hideable())
            .collect();
        // `Changes` is hidden until the user has chosen otherwise. Every
        // save writes the *complete* order (it names every column), so a
        // stored order that does not name it means the user never decided.
        if !ui.log_columns_order.iter().any(|n| n == "changes") {
            hidden.insert(LogColumn::Changes);
        }
        let widths = ui
            .log_column_widths
            .iter()
            .filter_map(|(n, w)| LogColumn::from_config_name(n).map(|c| (c, *w)))
            // `Message` is flexible now; a width an older version stored
            // for it would pin it and defeat that.
            .filter(|(c, _)| *c != LogColumn::Message)
            .collect();
        let scroll_widths = ui
            .log_column_widths_scroll
            .iter()
            .filter_map(|(n, w)| LogColumn::from_config_name(n).map(|c| (c, *w)))
            .collect();
        Self {
            order,
            hidden,
            widths,
            scroll_widths,
        }
    }

    /// Name lists in a deterministic order, ready for the config writer.
    pub(crate) fn to_names(&self) -> (Vec<String>, Vec<String>, NameWidths, NameWidths) {
        let hidden = LogColumn::ALL
            .iter()
            .filter(|c| self.hidden.contains(c))
            .map(|c| c.config_name().to_string())
            .collect();
        let order = self
            .order
            .iter()
            .map(|c| c.config_name().to_string())
            .collect();
        let widths = self
            .widths
            .iter()
            .map(|(c, w)| (c.config_name().to_string(), *w))
            .collect();
        let scroll = self
            .scroll_widths
            .iter()
            .map(|(c, w)| (c.config_name().to_string(), *w))
            .collect();
        (hidden, order, widths, scroll)
    }

    /// Writes the layout to `config.toml` on a short-lived thread (the
    /// input loop never waits on the disk); a process-wide lock keeps two
    /// quick successive saves in order.
    pub(crate) fn save_in_background(&self, path: &std::path::Path) {
        static WRITE_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
        let (hidden, order, widths, scroll) = self.to_names();
        let path = path.to_path_buf();
        std::thread::spawn(move || {
            let _guard = WRITE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
            let _ =
                gitpoulpi_config::set_log_columns_in_file(&path, &hidden, &order, &widths, &scroll);
        });
    }
}
