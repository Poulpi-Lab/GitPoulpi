//! The Preferences screen: the gear in the toolbar's right-hand end,
//! with two scopes.
//!
//! Two scopes: **Preferences** (the user's,
//! in `config.toml`) and **Repo-Specific Settings** (this
//! repository's, in `.gitpoulpi/repo-state.toml` via
//! `gitpoulpi_session::RepoPreferences`). The sidebar names both, and
//! every page says which of the two it is writing.
//!
//! ## Why this is its own module
//!
//! The screen is *modal*: while it is open every click is either inside
//! it or dismisses it, so it hit-tests itself against rects it captured
//! while drawing rather than pushing entries into `lib.rs`'s
//! `hit_targets`.
//!
//! ## What is actually wired
//!
//! Every row is one of three states, and the screen says which rather
//! than letting the user find out by experiment:
//!
//! - **live**: something reads it. Derived from `gitpoulpi_config::
//!   LIVE_KEYS` - the same constant `gitpoulpi config get` reports from -
//!   so the screen cannot claim something the config crate denies. That
//!   is the discipline `schema.rs`'s module comment asks for, made
//!   visible.
//! - **not wired yet**: the value round-trips through `config.toml` but
//!   nothing acts on it. Still editable - the note under it says so.
//! - **coming** (`Field::coming`): not editable at all, with a one-line
//!   reason in place of both the note and the value, for a setting whose
//!   feature does not exist yet (LFS, submodule sync, ...). The
//!   `every_editable_row_is_live_or_explicitly_coming` test in `tests.rs`
//!   fails if a new row is in neither state.
//!
//! ## Tokens
//!
//! An integration's personal access token never reaches `config.toml`.
//! The file holds `token_ref = "integration:<id>"` and the token goes to
//! `gitpoulpi_secrets::TokenStore` (the git credential helper). The
//! field renders as dots and a presence flag, never as the secret.

use crate::redraw::NotifySend as _;
use std::path::{Path, PathBuf};

use crate::wide_edge::Clear;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEventKind};
use gitpoulpi_config::{
    set_in_file, set_list_in_file, AvatarFallback, AvatarPlacement, AvatarSources, ColorDepth,
    ComposerFiles, Config, ConfigPaths, DefaultBackend, DiffLayout, GraphLaneBackground,
    GraphRenderer, GraphRowTintStrength, ImageProtocol, KittyKeyboard, Profile, PullButtonMode,
    PushBehavior, StatusbarPosition, TabDirFollow, TabsPosition, ToolbarAlign, ToolbarPosition,
    UiLanguage, MAX_MOUSE_SCROLL_LINES, MIN_COMMITS_PER_LAZY_PAGE, MIN_INITIAL_COMMITS_IN_GRAPH,
    MIN_MOUSE_SCROLL_LINES,
};
use gitpoulpi_secrets::{CredentialHelperTokens, TokenStore};
use gitpoulpi_session::RepoState;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};
use ratatui::Frame;

use crate::{display_width, wrap_words};

#[cfg_attr(not(feature = "ai"), path = "ai_page_stub.rs")]
mod ai_page;
mod buttons;
mod choice_labels;
mod close_guard;
mod commit_pages;
mod conflict_pages;
mod editor_close;
mod field_layout;
mod fields;
mod fields_ui;
pub(crate) use fields_ui::{language_choices, language_options};
pub(crate) use launcher_alias::ToolsButton;
pub(crate) use repo_pages::SparseRequest;
mod images_page;
mod input;
mod keybindings_page;
mod launcher_alias;
mod launcher_page;
pub(crate) mod list_editor;
mod list_page;
mod model;
mod nav;
mod open_in_page;
mod pages;
pub(crate) mod palette_settings;
mod persist;
mod quantity;
mod quantity_rows;
mod render;
mod repo_pages;
mod search;
mod sub_tabs;
mod theme_page;
mod view_memory;
mod workflow_page;
use ai_page::AiPage;

/// Whether this build has the feature an AI action's Settings block belongs
/// to: the two history rewrite actions are the History workbench's.
#[cfg_attr(not(feature = "ai"), allow(dead_code))]
pub(crate) fn ai_action_built(action: gitpoulpi_config::AiAction) -> bool {
    use gitpoulpi_config::AiAction as A;
    !matches!(action, A::RewriteScript | A::RewriteRules) || crate::build_features::HISTORY_REWRITE
}
pub(crate) use sub_tabs::SubTab;

use crate::live_settings::{
    format_date, install_active_profile, install_auto_prune, install_date_patterns,
    install_delete_orig_after_merge, install_diff_view_prefs, install_language,
    install_log_wip_counts_in_message, install_minimap, DiffViewPrefs,
};

use model::{
    Btn, ChipCell, Control, CtrlCell, Field, PrefAction, PrefFocus, PrefSection, QuantityUnit,
    RepoField, Target, TextSlot,
};

/// Seconds since the unix epoch, for the date previews. `0` if the clock
/// is before 1970, which is not a case worth an error path.
fn now_seconds() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Starts a background thread answering `tokens.has(token_ref)` for each ref,
/// sending each answer as soon as it is known. Never on the input thread: a
/// `TokenStore::has` is a `git credential fill` subprocess, ~2.4s per
/// integration through Git Credential Manager on Windows. Dropping the
/// receiver makes the next `send` fail, which stops the walk.
///
/// Returns `None` when there is nothing to check.
fn spawn_token_presence_probe(
    tokens: std::sync::Arc<dyn TokenStore + Send + Sync>,
    token_refs: Vec<String>,
) -> Option<std::sync::mpsc::Receiver<(String, bool)>> {
    if token_refs.is_empty() {
        return None;
    }
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        for token_ref in token_refs {
            let present = tokens.has(&token_ref);
            if tx.send_notify((token_ref, present)).is_err() {
                return; // the screen closed; nobody is listening any more
            }
        }
    });
    Some(rx)
}

/// The whole screen's state.
pub(crate) struct PreferencesState {
    config: Config,
    config_path: Option<PathBuf>,
    repo_label: String,
    repo_state: RepoState,
    repo_state_path: PathBuf,
    tokens: std::sync::Arc<dyn TokenStore + Send + Sync>,
    /// Whether a token is stored, by `token_ref`. Never read from `tokens`
    /// while building rows: the store shells out to `git credential fill`
    /// (a popped window per row per frame with an interactive helper).
    ///
    /// `None` means *still checking*: the `tokens.has` calls run on a
    /// background thread (`spawn_token_presence_probe`), because one `git
    /// credential fill` through GCM takes about 2.4 s. Store, delete and
    /// rename update this cache optimistically instead of re-querying.
    token_presence: std::collections::HashMap<String, Option<bool>>,
    /// The background probe started by `with_parts`, if there was at
    /// least one integration to check. Drained by `poll_token_presence`.
    token_presence_rx: Option<std::sync::mpsc::Receiver<(String, bool)>>,
    section: usize,
    field: usize,
    /// On a page with a strip (`PrefSection::has_strip`): the keyboard is in
    /// the strip's generic rows rather than the page's own body.
    strip: bool,
    /// Where the strip was drawn last frame (a click there is the generic list's).
    strip_rect: Rect,
    focus: PrefFocus,
    /// `(field index, draft, masked)` while a text/number field is being
    /// typed into.
    edit: Option<(usize, String, bool)>,
    /// The field index whose typed text was refused when the user left it
    /// (drawn red; the text stays so nothing typed is lost).
    edit_invalid: Option<usize>,
    /// Why the refused number of a quantity row was refused (its field
    /// index, the reason), drawn under the row while it stays red.
    inline_error: Option<(usize, String)>,
    profile_index: usize,
    /// The master list of the current master-detail page (`has_list`), fed by
    /// `sync_list` and painted by `render_list_body`.
    list: list_editor::ListEditor,
    /// Where the list was painted last frame (empty when the page has none).
    list_area: Rect,
    /// The pages of `PrefSection::ALL` (one bit each) showing their list
    /// tab; the others show General (`sub_tabs`).
    list_tabs: u32,
    /// Where each page (and tab) was left: its selected row and scroll
    /// (`view_memory`).
    views: Vec<view_memory::View>,
    /// The sub-tab strip as last painted: each tab's rect.
    tab_rects: Vec<(Rect, SubTab)>,
    /// Rows a page spends above its fields: 2 (the title and a blank row),
    /// 1 when the page painted its own title and tab strip above.
    body_head: u16,
    /// The Open in page's selected entry.
    open_in_index: usize,
    /// The Images page's selected editor.
    image_index: usize,
    /// General's Launcher group (shortcuts), read on a thread.
    launcher: launcher_page::LauncherState,
    /// The command alias and PATH rows under it, read on a thread.
    tools: launcher_alias::ToolsState,
    /// The detected image editors, set by tests; the page otherwise asks the
    /// shared background lookup (`image_editor::detected_ready`).
    image_detected: std::cell::OnceCell<gitpoulpi_config::image_editors::Detected>,
    /// What the machine runs, looked up once per Settings open.
    open_in_detected: std::cell::OnceCell<gitpoulpi_config::open_in::Detected>,
    /// The folder the page's Test button runs against.
    repo_root: PathBuf,
    /// The Integrations page's own view state (`integrations_page`).
    integrations: crate::integrations_page::IntegrationsPage,
    message: Option<String>,
    scroll: usize,
    /// The (section, field, focus) the view last scrolled to keep in sight.
    /// The keep-in-view rule runs only when this changes, so a wheel
    /// scroll is not undone on the next frame.
    kept_in_view: Option<(usize, usize, PrefFocus)>,
    now: i64,
    /// Captured while drawing, consumed by the next mouse event.
    area: Rect,
    /// The two halves of `area`, so a click can be attributed to the
    /// side it landed on. Without them the hit test matched on the row
    /// alone, and a click on a setting selected whichever section
    /// happened to sit at the same height in the sidebar.
    sidebar_area: Rect,
    body_area: Rect,
    /// The `\u{2715} Esc` control's rect, or `None` when the panel is too
    /// small to have drawn it.
    close_rect: Option<Rect>,
    /// The app's octopus in the title line (the main window's own widget):
    /// a click opens About, like the title bar's.
    logo_rect: Option<Rect>,
    /// A line on the bottom border (a command refused while Settings is
    /// open); the next key or mouse event clears it. The status bar is
    /// covered by this screen, so a message there would never be seen.
    notice: Option<String>,
    /// The pending `Ctrl+N` prefix's hint, set by the event loop each frame;
    /// it takes the bottom border's place while it waits.
    mode_hint: Option<String>,
    /// The octopus was clicked: the event loop opens About (`take_open_about`).
    open_about: bool,
    /// A theme / language picker was open at the last frame: when it closes
    /// the screen re-reads `config.toml`, which the picker wrote to.
    picker_was_open: bool,
    /// The mouse's last position, for the close control's hover (the
    /// same strong `plain_button_style` as the gear and the app's own
    /// close cross).
    mouse_pos: Option<(u16, u16)>,
    /// Shift was held on the mouse event being handled.
    mouse_shift: bool,
    /// What the Git & Encoding page says about the `git-lfs` program
    /// (`Missing`, `3.5.1`, ...), pushed in by the event loop from its LFS
    /// state; `None` until the first snapshot.
    lfs_program: Option<String>,
    /// The "Initialize LFS" button was pressed; the event loop turns it
    /// into `Action::LfsInstall`.
    lfs_init_requested: bool,
    /// The repository-local `core.autocrlf` / `core.eol` as git reports
    /// them (pushed in by the event loop; edited optimistically on a pick),
    /// the refresh counter it was last replaced at, and the picks the event
    /// loop has yet to send to the daemon.
    repo_config: std::collections::BTreeMap<String, Option<String>>,
    repo_config_version: u64,
    repo_config_requests: Vec<(String, Option<String>)>,
    sidebar_rows: Vec<(u16, usize)>,
    field_rows: Vec<(u16, usize)>,
    /// The left edge of the pane the field rows were painted in (the body,
    /// or the detail beside a list).
    fields_x: u16,
    /// The open dropdown of a choice field: its field index and the shared
    /// combobox. Modal while open; a pick commits by option index.
    choice: Option<(usize, crate::combo::Combo)>,
    /// Where the open choice's value was painted this frame (dx in the
    /// body, width), so the popup hangs from it.
    choice_value_at: std::cell::Cell<Option<(u16, u16, u16)>>,
    /// The user theme files under `<config>/themes` (re-read when the
    /// Theme combo opens).
    user_themes: gitpoulpi_config::theme::ThemeSet,
    /// The theme the open Theme combo is previewing (installed but not
    /// saved), `None` when nothing is being previewed.
    theme_preview: Option<String>,
    /// The text/number/secret field being edited: its caret and
    /// selection, keyed by field index.
    caret_edit: crate::text_edit::FieldEditor<usize>,
    /// The AI page's own state.
    ai: AiPage,
    /// The Workflow page's own state (`workflow_page`).
    workflow: workflow_page::WorkflowPage,
    /// The Sparse checkout and LFS pages' state (`repo_pages`).
    repo_pages: repo_pages::RepoPages,
    /// The Keyboard Shortcuts page's own state (`keybindings_page`).
    keys: keybindings_page::KeybindingsPage,
    /// Where the edited value sits in its row, as (column offset, width),
    /// recorded by `field_lines` for `render_body` to place on screen.
    edit_value_at: std::cell::Cell<Option<(u16, u16, u16)>>,
    /// The page's value column (0 = not computed yet), set once per frame by
    /// `set_label_col` so every row of a page starts its value at the same cell.
    label_col: std::cell::Cell<(usize, bool)>,
    /// Where each chip of a `Control::Chips` row was painted this frame,
    /// relative to its field's first line: (field, chip, line, dx, width).
    /// Filled by `field_lines`, turned into screen rects by `render_body`.
    chip_cells: std::cell::RefCell<Vec<ChipCell>>,
    /// The chips' screen rects of the last frame: the paint walk's hit
    /// rects (field, chip).
    chip_rects: Vec<(Rect, usize, usize)>,
    /// The same two for the inline buttons of `Control::Buttons` rows
    /// (field, button).
    btn_cells: std::cell::RefCell<Vec<ChipCell>>,
    /// Where each row's own control (a checkbox, a combobox field, a text
    /// input box) was painted, relative to its field's first line:
    /// (field, is_input, line, dx, width). The pointer is the hand over a
    /// control and the I-beam over an input box, never over the label or help.
    ctrl_cells: std::cell::RefCell<Vec<CtrlCell>>,
    btn_rects: Vec<(Rect, usize, usize)>,
    /// The "are you sure" box of a destructive button, with what it runs.
    confirm_action: Option<(crate::dialog_kit::Confirm<bool>, PrefAction)>,
    body_height: usize,
    /// The body's vertical scrollbar as last painted (`None` when the page
    /// fits, or draws its own body), the line count it was computed from,
    /// and its thumb drag.
    body_bar: Option<crate::scrollbar::Track>,
    body_lines: usize,
    body_drag: crate::scrollbar::BarDrag,
    /// The settings search popup, when open (`search.rs`).
    search: Option<search::SettingsSearch>,
    /// Where the sidebar's search box was painted.
    search_box: Option<Rect>,
    /// The row the search last jumped to, as (section, field): drawn
    /// lit until the next key or click.
    flash: Option<(usize, usize)>,
    /// The config key an agent's `ui show settings --setting` landed on
    /// (read back by `ui state`).
    shown_setting: Option<String>,
    /// The "Turning Undo off erases the history" question, when up:
    /// the selected button (0 Cancel, 1 Turn off) and where the two were
    /// painted.
    confirm_undo_off: Option<crate::dialog_kit::Confirm<bool>>,
    /// The "close Settings?" question of the close guard, when up.
    close_ask: Option<close_guard::CloseAsk>,
}

/// The section the Settings screen was last left on, so reopening it
/// lands where the user was. Process-wide and
/// only for the app's lifetime: `PreferencesState::open` reads it,
/// `Drop` writes it. `with_parts` (the testable constructor) never reads
/// it, so unit tests stay independent of each other.
static LAST_SECTION: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

impl Drop for PreferencesState {
    /// Closing Preferences while a token check runs must not strand the
    /// token: the check finishes in the background and saves the profile.
    fn drop(&mut self) {
        LAST_SECTION.store(self.section, std::sync::atomic::Ordering::Relaxed);
        sub_tabs::LAST_TABS.store(self.list_tabs, std::sync::atomic::Ordering::Relaxed);
        self.save_view();
        view_memory::store_last(&self.views);
        self.integrations.detach_pending();
    }
}

impl PreferencesState {
    /// Lands on the Integrations page, on profile `profile_id` when it
    /// exists (the sidebar's "Reconnect" row, the palette's
    /// `Integrations: ...`); the keyboard goes to the page's body.
    pub(crate) fn show_integration(&mut self, profile_id: Option<&str>) {
        self.section = PrefSection::ALL
            .iter()
            .position(|s| *s == PrefSection::Integrations)
            .unwrap_or(0);
        self.focus = PrefFocus::Fields;
        self.strip = false;
        self.integrations.zone = crate::integrations_page::Zone::Types;
        let Some(profile) = profile_id
            .and_then(|id| self.config.integrations.iter().find(|i| i.id == id))
            .cloned()
        else {
            return;
        };
        if let Some(kind_index) = self.integrations.kind_index(profile.kind) {
            self.integrations.select_kind(kind_index, &self.config);
            self.integrations.sel = self
                .config
                .integrations
                .iter()
                .filter(|i| i.kind == profile.kind)
                .position(|i| i.id == profile.id)
                .unwrap_or(0);
            self.integrations.zone = crate::integrations_page::Zone::Body;
        }
    }

    /// Lands on the Integrations page, on the Radar block of the first
    /// connected code host (the palette's *Radar Options of a Profile*).
    /// With no connected code host it is the page itself.
    pub(crate) fn show_radar_options(&mut self) {
        #[cfg(feature = "hosting")]
        let id = crate::integrations_page::IntegrationsPage::first_radar_profile(&self.config)
            .map(|i| i.id.clone());
        #[cfg(not(feature = "hosting"))]
        let id: Option<String> = None;
        self.show_integration(id.as_deref());
        if id.is_some() {
            self.integrations.focus_radar(&self.config);
        }
    }

    /// Opens the screen against the real config and the real repository.
    pub(crate) fn open(repo_root: &Path) -> Self {
        let paths = ConfigPaths::discover();
        let config_path = paths.as_ref().map(|p| p.config_file());
        // The *file* layer, deliberately, not `Settings::resolve`: this
        // screen edits `config.toml`, and showing a value that an
        // env var is overriding would mean editing a field that
        // visibly does nothing. The banner says when that is the case.
        let config = config_path
            .as_deref()
            .map(Config::load)
            .transpose()
            .ok()
            .flatten()
            .unwrap_or_default();
        let repo_state_path = RepoState::path_for(repo_root, &config.session.per_repo_state_dir);
        let repo_state = RepoState::load(&repo_state_path).unwrap_or_default();
        let mut state = Self::with_parts(
            config,
            config_path,
            repo_root,
            repo_state,
            repo_state_path,
            std::sync::Arc::new(CredentialHelperTokens::new()),
        );
        state.section = LAST_SECTION
            .load(std::sync::atomic::Ordering::Relaxed)
            .min(PrefSection::ALL.len() - 1);
        if !PrefSection::ALL[state.section].is_built() {
            state.section = 0;
        }
        state.list_tabs = sub_tabs::LAST_TABS.load(std::sync::atomic::Ordering::Relaxed);
        state.views = view_memory::load_last();
        state.restore_view();
        state.launcher_start(None);
        state.tools_start(None);
        state
    }

    /// The testable constructor: everything that touches the machine is
    /// an argument.
    pub(crate) fn with_parts(
        config: Config,
        config_path: Option<PathBuf>,
        repo_root: &Path,
        repo_state: RepoState,
        repo_state_path: PathBuf,
        tokens: std::sync::Arc<dyn TokenStore + Send + Sync>,
    ) -> Self {
        let repo_label = repo_root
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| repo_root.display().to_string());
        // Every configured integration starts out "checking..." --
        // `spawn_token_presence_probe` below answers each one on a
        // background thread rather than here, synchronously, on the
        // input thread. See `token_presence`'s doc for why.
        let keybindings = config.keybindings.clone();
        let token_presence = config
            .integrations
            .iter()
            .map(|integration| (integration.token_ref.clone(), None))
            .collect();
        let token_presence_rx = spawn_token_presence_probe(
            std::sync::Arc::clone(&tokens),
            config
                .integrations
                .iter()
                .map(|integration| integration.token_ref.clone())
                .collect(),
        );
        let mut state = Self {
            config,
            config_path,
            repo_label,
            repo_state,
            repo_state_path,
            tokens,
            token_presence,
            token_presence_rx,
            section: 0,
            strip: false,
            strip_rect: Rect::new(0, 0, 0, 0),
            field: 0,
            focus: PrefFocus::Sections,
            edit: None,
            edit_invalid: None,
            inline_error: None,
            profile_index: 0,
            list: list_editor::ListEditor::new(list_editor::ListSpec::default()),
            list_area: Rect::default(),
            list_tabs: 0,
            views: view_memory::blank(),
            tab_rects: Vec::new(),
            body_head: 2,
            open_in_index: 0,
            image_index: 0,
            launcher: Default::default(),
            tools: Default::default(),
            image_detected: std::cell::OnceCell::new(),
            open_in_detected: std::cell::OnceCell::new(),
            repo_root: repo_root.to_path_buf(),
            integrations: Default::default(),
            message: None,
            scroll: 0,
            kept_in_view: None,
            now: now_seconds(),
            area: Rect::new(0, 0, 0, 0),
            sidebar_area: Rect::new(0, 0, 0, 0),
            body_area: Rect::new(0, 0, 0, 0),
            close_rect: None,
            logo_rect: None,
            notice: None,
            fields_x: 0,
            mode_hint: None,
            open_about: false,
            picker_was_open: false,
            mouse_pos: None,
            mouse_shift: false,
            lfs_program: None,
            lfs_init_requested: false,
            repo_config: Default::default(),
            repo_config_version: 0,
            repo_config_requests: Vec::new(),
            sidebar_rows: Vec::new(),
            field_rows: Vec::new(),
            choice: None,
            choice_value_at: std::cell::Cell::new(None),
            user_themes: Default::default(),
            theme_preview: None,
            caret_edit: Default::default(),
            ai: Default::default(),
            workflow: Default::default(),
            repo_pages: Default::default(),
            keys: keybindings_page::KeybindingsPage::new(crate::keymap::Keymap::from_config(
                &keybindings,
            )),
            edit_value_at: std::cell::Cell::new(None),
            label_col: std::cell::Cell::new((0, false)),
            chip_cells: Default::default(),
            chip_rects: Vec::new(),
            btn_cells: Default::default(),
            ctrl_cells: Default::default(),
            btn_rects: Vec::new(),
            confirm_action: None,
            body_height: 0,
            body_bar: None,
            body_lines: 0,
            body_drag: Default::default(),
            search: None,
            search_box: None,
            flash: None,
            shown_setting: None,
            confirm_undo_off: None,
            close_ask: None,
        };
        state.rescan_user_themes();
        state.ai_probe_keys();
        state.template_reload(true);
        state
    }

    /// Shows `text` on the bottom border until the next event.
    pub(crate) fn set_notice(&mut self, text: &str) {
        self.notice = Some(text.to_string());
    }

    /// The octopus in the title line was clicked since the last call.
    pub(crate) fn take_open_about(&mut self) -> bool {
        std::mem::take(&mut self.open_about)
    }

    /// Called each frame with whether a theme or language picker is open
    /// over this screen. The picker writes `ui.theme` / `ui.language` to the
    /// file itself, so when it closes the screen reads the file again and its
    /// Theme and Language rows show the new value, not a stale copy.
    pub(crate) fn note_picker(&mut self, open: bool) {
        if self.picker_was_open && !open {
            if let Some(path) = self.config_file() {
                if let Ok(config) = Config::load(&path) {
                    self.config = config;
                    self.theme_preview = None;
                }
            }
        }
        self.picker_was_open = open;
    }

    pub(crate) fn take_message(&mut self) -> Option<String> {
        self.message.take()
    }

    /// Tells the Git & Encoding page what the event loop knows about the
    /// `git-lfs` program (`None`: no snapshot yet).
    pub(crate) fn set_lfs_program(&mut self, text: Option<String>) {
        self.lfs_program = text;
    }

    /// Whether the "Initialize LFS" button was pressed since the last
    /// call.
    pub(crate) fn take_lfs_init_request(&mut self) -> bool {
        std::mem::take(&mut self.lfs_init_requested)
    }

    /// Replaces the shown git config values by what the daemon read, once per
    /// refresh (`version`), so an optimistic pick survives until the next
    /// real answer.
    pub(crate) fn set_repo_config(
        &mut self,
        values: &std::collections::BTreeMap<String, Option<String>>,
        version: u64,
    ) {
        if version != self.repo_config_version {
            self.repo_config_version = version;
            self.repo_config = values.clone();
            self.template_follow_config();
        }
    }

    /// The `core.autocrlf` / `core.eol` picks made since the last call, as
    /// `(key, value)`; `None` clears the repository-local value.
    pub(crate) fn take_repo_config_requests(&mut self) -> Vec<(String, Option<String>)> {
        std::mem::take(&mut self.repo_config_requests)
    }

    /// A spinner is on screen: the render loop keeps drawing.
    pub(crate) fn animating(&self) -> bool {
        self.integrations.animating() || self.open_in_detecting() || self.images_detecting()
    }

    /// Applies whatever the background token-presence probe
    /// (`spawn_token_presence_probe`) has answered since the last call.
    /// Never blocks (an `mpsc::try_recv` loop); called once a frame while
    /// the screen is open. A result for a `token_ref` no longer in
    /// `token_presence` (renamed or removed meanwhile) is dropped.
    pub(crate) fn poll_token_presence(&mut self) {
        self.poll_ai();
        self.poll_integrations();
        self.poll_launcher();
        self.poll_tools();
        let Some(rx) = &self.token_presence_rx else {
            return;
        };
        let answers: Vec<(String, bool)> = std::iter::from_fn(|| rx.try_recv().ok()).collect();
        for (token_ref, present) in answers {
            if let std::collections::hash_map::Entry::Occupied(mut entry) =
                self.token_presence.entry(token_ref)
            {
                entry.insert(Some(present));
            }
        }
    }

    /// Applies a finished background `check_token` on the Integrations
    /// page. A no-op when none is running.
    fn poll_integrations(&mut self) {
        let config_path = self.config_file();
        let Self {
            integrations,
            config,
            tokens,
            token_presence,
            message,
            ..
        } = self;
        let mut env = crate::integrations_page::Env {
            config,
            config_path,
            tokens,
            presence: token_presence,
            message,
        };
        integrations.poll(&mut env);
    }

    fn integrations_key(&mut self, key: &KeyEvent) -> bool {
        let config_path = self.config_file();
        let Self {
            integrations,
            config,
            tokens,
            token_presence,
            message,
            ..
        } = self;
        let mut env = crate::integrations_page::Env {
            config,
            config_path,
            tokens,
            presence: token_presence,
            message,
        };
        let nav = integrations.on_key(key, &mut env);
        if nav == crate::integrations_page::Nav::Sections {
            self.focus = PrefFocus::Sections;
        }
        true
    }

    fn integrations_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        let config_path = self.config_file();
        let Self {
            integrations,
            config,
            tokens,
            token_presence,
            message,
            ..
        } = self;
        let mut env = crate::integrations_page::Env {
            config,
            config_path,
            tokens,
            presence: token_presence,
            message,
        };
        integrations.on_mouse(kind, col, row, &mut env)
    }

    /// A drag/release that began in a text field keeps belonging to it
    /// when the pointer leaves the body.
    fn integrations_page_owns_drag(&self, kind: MouseEventKind) -> bool {
        self.integrations.owns_pointer()
            || !matches!(kind, MouseEventKind::Down(_))
                && self.focus == PrefFocus::Fields
                && self.integrations.zone == crate::integrations_page::Zone::Body
    }

    /// A bracketed paste (a token, usually). Only the Integrations page's
    /// text fields take one.
    pub(crate) fn paste(&mut self, text: &str) {
        if self.search.is_some() {
            self.search_paste(text);
            return;
        }
        if self.ai.browse_open() {
            self.review_browse_paste(text);
            return;
        }
        if self.ai.prompt_open() {
            self.ai_prompt_paste(text);
            return;
        }
        if self.wf_editor_open() {
            self.wf_editor_paste(text);
            return;
        }
        if self.rp_editor_open() {
            self.rp_editor_paste(text);
            return;
        }
        if self.keys_active() {
            self.keys_paste(text);
            return;
        }
        if self.current_section() == PrefSection::Integrations
            && self.focus == PrefFocus::Fields
            && !self.strip
        {
            self.integrations.paste(text, &self.config);
        }
    }

    fn current_section(&self) -> PrefSection {
        PrefSection::ALL[self.section.min(PrefSection::ALL.len() - 1)]
    }

    // ---------------------------------------------------------------
    // The pages
    // ---------------------------------------------------------------

    fn fields(&self) -> Vec<Field> {
        self.fields_of(self.current_section())
    }

    /// The rows of `section` as its front tab shows them.
    fn fields_of(&self, section: PrefSection) -> Vec<Field> {
        self.fields_of_tab(section, self.sub_tab_of(section))
    }

    /// The rows of `section`'s `tab`. A page without tabs has one list of
    /// rows, whatever tab is asked for.
    fn fields_of_tab(&self, section: PrefSection, tab: SubTab) -> Vec<Field> {
        match (section, tab) {
            (PrefSection::Profiles, SubTab::General) => self.profile_general_fields(),
            (PrefSection::Profiles, SubTab::List) => self.profile_item_fields(),
            (PrefSection::Ai, SubTab::General) => self.ai_general_fields(),
            (PrefSection::Ai, SubTab::List) => self.ai_item_fields(),
            (PrefSection::OpenIn, SubTab::General) => self.open_in_general_fields(),
            (PrefSection::OpenIn, SubTab::List) => self.open_in_item_fields(),
            (PrefSection::Images, SubTab::General) => self.images_general_fields(),
            (PrefSection::Images, SubTab::List) => self.images_item_fields(),
            (PrefSection::Workflow, SubTab::General) => self.workflow_general_fields(),
            (PrefSection::Workflow, SubTab::List) => self.workflow_item_fields(),
            (PrefSection::Keybindings, SubTab::General) => self.picked(pages::KEYBOARD),
            (PrefSection::Keybindings, SubTab::List) => Vec::new(),
            _ => self.fields_of_plain(section),
        }
    }

    fn fields_of_plain(&self, section: PrefSection) -> Vec<Field> {
        match section {
            PrefSection::General => self.general_fields(),
            PrefSection::Sync => self.sync_fields(),
            PrefSection::Profiles => Vec::new(),
            PrefSection::Ssh => self.ssh_fields(),
            PrefSection::Integrations => self.strip_fields(section),
            PrefSection::Keybindings => Vec::new(),
            PrefSection::Ai => Vec::new(),
            PrefSection::AgentsMcp => self.agents_fields(),
            PrefSection::Appearance => self.appearance_fields(),
            PrefSection::LogGraph => self.log_graph_fields(),
            PrefSection::DiffEditor => self.diff_editor_fields(),
            PrefSection::OpenIn | PrefSection::Images => Vec::new(),
            PrefSection::GitAndEncoding => self.git_fields(),
            PrefSection::RepoAi => self.repo_ai_fields(),
            PrefSection::Workflow => Vec::new(),
            PrefSection::SparseCheckout => self.sparse_fields(),
            PrefSection::Lfs => self.lfs_fields(),
            PrefSection::Commit => self.commit_fields(),
            PrefSection::MergeBehavior => self.merge_fields(),
            PrefSection::CommitTemplate => self.template_fields(),
            PrefSection::ConflictPrevention => self.conflict_fields(),
        }
    }
}

#[cfg(test)]
pub(crate) mod tests;
