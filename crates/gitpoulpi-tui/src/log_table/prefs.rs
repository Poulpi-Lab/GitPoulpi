//! The Log column layout: order and hidden set in the `[ui]` config table
//! (global), widths in the repository's own state file.

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

    /// The global order and hidden set from `[ui]`, the widths from this
    /// repository's own state (`None`: never saved, so no widths yet and the
    /// window fits every column once the Log has painted).
    pub(crate) fn for_repo(
        ui: &gitpoulpi_config::UiConfig,
        saved: Option<&gitpoulpi_session::LogColumnWidths>,
    ) -> Self {
        let mut me = Self::from_ui(ui);
        me.widths.clear();
        me.scroll_widths.clear();
        if let Some(saved) = saved {
            let by_name = |m: &NameWidths| -> std::collections::HashMap<LogColumn, u16> {
                m.iter()
                    .filter_map(|(n, w)| LogColumn::from_config_name(n).map(|c| (c, *w)))
                    .collect()
            };
            me.widths = by_name(&saved.fit);
            // `Message` is flexible in the `fit` layout; never pinned there.
            me.widths.remove(&LogColumn::Message);
            me.scroll_widths = by_name(&saved.scroll);
        }
        me
    }

    /// The widths as this repository's state stores them.
    pub(crate) fn to_repo_widths(&self) -> gitpoulpi_session::LogColumnWidths {
        let (_, _, fit, scroll) = self.to_names();
        gitpoulpi_session::LogColumnWidths { fit, scroll }
    }

    /// Writes the global order and hidden set to `config.toml` on a
    /// short-lived thread (the input loop never waits on the disk); a
    /// process-wide lock keeps two quick successive saves in order.
    pub(crate) fn save_layout_in_background(&self, path: &std::path::Path) {
        static WRITE_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
        let (hidden, order, _, _) = self.to_names();
        let path = path.to_path_buf();
        std::thread::spawn(move || {
            let _guard = WRITE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
            let _ = gitpoulpi_config::set_log_column_layout_in_file(&path, &hidden, &order);
        });
    }

    /// Writes the widths to the repository's state file the same way
    /// (load-modify-save, so its other fields are kept).
    pub(crate) fn save_widths_in_background(&self, state_path: &std::path::Path) {
        static WRITE_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
        let widths = self.to_repo_widths();
        let path = state_path.to_path_buf();
        std::thread::spawn(move || {
            let _guard = WRITE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
            let mut state = gitpoulpi_session::RepoState::load(&path).unwrap_or_default();
            state.log_columns = Some(widths);
            let _ = state.save(&path);
        });
    }
}
