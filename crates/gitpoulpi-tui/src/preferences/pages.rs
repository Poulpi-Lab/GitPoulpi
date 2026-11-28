//! Which setting lives on which Settings page.
//!
//! Every plain `config.toml` row is built once in a pool (`general_rows`,
//! `ui_rows`, `editor_rows`) and each page below lists the keys it shows, in
//! order, with the headings between them. Moving a setting is a one-line
//! change here; `tests::pages` pins the mapping.

use super::*;

// The page tables are shared with every frontend
// (`gitpoulpi_app::settings::pages`).
#[cfg(test)]
pub(super) use gitpoulpi_app::settings::pages::tables;
pub(super) use gitpoulpi_app::settings::pages::{
    Entry as Row, AGENTS, APPEARANCE, CONFLICT_GLOBAL, DIFF_EDITOR, GENERAL, IMAGES,
    IMAGES_EDITORS_ONLY, INTEGRATIONS, KEYBOARD, LAUNCHER, LOG_GRAPH, OPEN_IN, SYNC, SYNC_REWRITE,
};

impl PreferencesState {
    /// Every plain `config.toml` row, unplaced, plus the date patterns'
    /// live previews (each right after its pattern).
    pub(super) fn row_pool(&self) -> Vec<Field> {
        let mut pool = self.general_rows();
        pool.extend(self.ui_rows());
        pool.extend(self.editor_rows());
        pool.extend(self.installation_rows());
        pool.extend(self.launcher_rows());
        #[cfg(feature = "mcp")]
        {
            pool.extend(self.mcp_rows());
            pool.extend(self.mcp_security_rows());
        }
        // The rows of a feature this build lacks (shared rule).
        pool.retain(|f| match f.target {
            Target::Global(key) => gitpoulpi_app::settings::key_built(key),
            _ => true,
        });
        pool
    }

    /// The rows of `table`, in its order, taken from a fresh pool.
    pub(super) fn picked(&self, table: &[Row]) -> Vec<Field> {
        gitpoulpi_app::settings::pages::pick(
            table,
            self.row_pool(),
            |f| match f.target {
                Target::Global(key) => Some(key),
                _ => None,
            },
            |f| {
                matches!(f.target, Target::None)
                    && matches!(f.control, Control::Info(_) | Control::Swatches(_))
            },
            Field::heading,
        )
    }

    pub(super) fn general_fields(&self) -> Vec<Field> {
        self.picked(GENERAL)
    }

    /// The Launcher page.
    pub(super) fn launcher_fields(&self) -> Vec<Field> {
        let mut fields = self.picked(LAUNCHER);
        // The alias and PATH buttons sit right under the alias box; the
        // per-place shortcut rows follow.
        let alias_at = fields
            .iter()
            .position(|f| f.target == Target::Global("installation.command_alias_add"))
            .map_or(fields.len(), |i| i + 1);
        fields.splice(alias_at..alias_at, self.tools_action_rows());
        // The other installations and the legacy launchers close the
        // "This installation" group, right under the submenu row.
        let group_end = fields
            .iter()
            .position(|f| f.target == Target::Global("installation.folder_menu_submenu"))
            .map_or(0, |i| i + 1);
        fields.splice(group_end..group_end, self.installation_action_rows());
        fields.extend(self.launcher_action_rows());
        fields.extend(self.folder_menu_rows());
        fields
    }

    /// The global Conflict prevention page.
    pub(super) fn conflict_global_page_fields(&self) -> Vec<Field> {
        let mut fields = self.picked(CONFLICT_GLOBAL);
        // The default list and the named lists follow the global switch.
        if let Some(at) = fields
            .iter()
            .position(|f| f.target == Target::Global("general.conflict_detection"))
        {
            fields.splice(at + 1..at + 1, self.conflict_global_fields());
        }
        fields
    }

    pub(super) fn sync_fields(&self) -> Vec<Field> {
        let mut fields = self.picked(SYNC);
        if crate::build_features::HISTORY_REWRITE {
            fields.extend(self.picked(SYNC_REWRITE));
        }
        fields
    }

    /// The agent-facing rows: the MCP server's switch and port (when this
    /// build has the server), the view-control and proposal switches and the
    /// "launch" pair.
    pub(super) fn agents_fields(&self) -> Vec<Field> {
        #[cfg_attr(not(feature = "mcp"), allow(unused_mut))]
        let mut fields = self.picked(AGENTS);
        // The fingerprint sits under the key path (`mcp_security`).
        #[cfg(feature = "mcp")]
        {
            let after = |fields: &[Field], key: &str| {
                fields
                    .iter()
                    .position(|f| matches!(f.target, Target::Global(k) if k == key))
                    .map(|i| i + 1)
            };
            if let Some(at) = after(&fields, "mcp.tls_key") {
                fields.splice(at..at, self.mcp_fingerprint_rows());
            }
        }
        // No server in this build: no security heading either.
        #[cfg(not(feature = "mcp"))]
        fields.retain(|f| f.label != gitpoulpi_i18n::msg::mcp_security::HEADING.get());
        fields
    }

    pub(super) fn appearance_fields(&self) -> Vec<Field> {
        self.picked(APPEARANCE)
    }

    pub(super) fn log_graph_fields(&self) -> Vec<Field> {
        self.picked(LOG_GRAPH)
    }

    pub(super) fn diff_editor_fields(&self) -> Vec<Field> {
        self.picked(DIFF_EDITOR)
    }

    /// Selectable rows shown above a page that draws its own body.
    pub(super) fn strip_fields(&self, section: PrefSection) -> Vec<Field> {
        match section {
            PrefSection::Integrations => self.picked(INTEGRATIONS),
            _ => Vec::new(),
        }
    }
}

impl PreferencesState {
    /// Whether the keyboard is in the generic rows of the current page: always
    /// on an ordinary page, only after moving up into the strip on a page
    /// that draws its own body below it.
    pub(super) fn list_focused(&self) -> bool {
        self.focus == PrefFocus::Fields && (!self.current_section().has_strip() || self.strip)
    }

    /// The strip's rows have the keyboard (not the page's own body).
    pub(super) fn strip_active(&self) -> bool {
        self.strip && self.focus == PrefFocus::Fields && self.current_section().has_strip()
    }

    /// Up from the top of a custom page's body moves into its strip.
    /// `true` when it did.
    pub(super) fn enter_strip_on(&mut self, key: &KeyEvent) -> bool {
        let section = self.current_section();
        if !section.has_strip()
            || self.focus != PrefFocus::Fields
            || self.strip
            || !matches!(key.code, KeyCode::Up)
            || !key.modifiers.is_empty()
        {
            return false;
        }
        if !self.integrations.at_top() {
            return false;
        }
        let fields = self.fields();
        let Some(last) = (0..fields.len()).rev().find(|i| fields[*i].selectable()) else {
            return false;
        };
        self.strip = true;
        self.field = last;
        true
    }

    /// Down from the strip's last row hands the keyboard to the page body.
    pub(super) fn leave_strip(&mut self) {
        self.strip = false;
        self.integrations.zone = crate::integrations_page::Zone::Types;
    }
}
