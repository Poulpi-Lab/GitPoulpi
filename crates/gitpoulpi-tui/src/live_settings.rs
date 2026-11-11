//! Process-wide live settings: statics with `install_*`/getter pairs (date
//! patterns, auto-prune, minimap, delete-orig, active profile, diff-view
//! prefs) and the date-pattern tokenizer. Re-exported by `preferences`.

use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::RwLock;

use gitpoulpi_config::Config;

// Statics read by the views every frame (`MINIMAP`, `BLAME_*`) are
/// process-wide in the binary, but each unit-test thread gets its own copy:
/// tests run in parallel in one process, and a test that turns the minimap on
/// (or drags the blame border) would otherwise change the width of whatever
/// another test is rendering at that moment (a flake that only showed under
/// load). The test thread that renders is the one that installs, so nothing
/// is lost by the split.
#[cfg(not(test))]
macro_rules! published {
    ($name:ident: $ty:ty = $init:expr) => {
        static $name: $ty = $init;
    };
}

#[cfg(test)]
macro_rules! published {
    ($name:ident: $ty:ty = $init:expr) => {
        thread_local! {
            static $name: $ty = const { $init };
        }
    };
}

/// Runs `f` on the published cell `name`, wherever it lives.
#[cfg(not(test))]
macro_rules! cell {
    ($name:ident, $f:expr) => {
        $f(&$name)
    };
}

#[cfg(test)]
macro_rules! cell {
    ($name:ident, $f:expr) => {
        $name.with($f)
    };
}

/// The four `ui.date_*` patterns in force, published here so `lib.rs`'s
/// `format_timestamp`/`format_timestamp_full` can read them without
/// threading a `Config` through every row builder.
///
/// A `RwLock`, not a `OnceLock`: a changed pattern shows on the next frame.
///
/// **How the four combine:** each pattern is an independent, user-editable
/// string with exactly one place in the UI that reads it, so changing one
/// never silently changes a different column:
/// | Key | Reads it |
/// |---|---|
/// | `date_format` | File Insight's Blame/History date column (`file_view::ymd`) |
/// | `date_time_format` | The Log's Date column itself |
/// | `date_verbose_format` | The commit-detail panel's Date line, the Date column's hover tooltip, and "Copy > Date" |
///
/// Month and weekday names follow `ui.language` (catalog) unless
/// `ui.date_locale` is `english`. `ui.date_word_format` is a legacy key,
/// still parsed and read by nothing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DatePatterns {
    pub(crate) short: String,
    pub(crate) time: String,
    pub(crate) verbose: String,
}

impl DatePatterns {
    fn from_config(ui: &gitpoulpi_config::UiConfig) -> Self {
        Self {
            short: ui.date_format.clone(),
            time: ui.date_time_format.clone(),
            verbose: ui.date_verbose_format.clone(),
        }
    }
}

impl Default for DatePatterns {
    fn default() -> Self {
        Self::from_config(&gitpoulpi_config::UiConfig::default())
    }
}

published!(DATE_PATTERNS: RwLock<Option<DatePatterns>> = RwLock::new(None));

/// Publishes every `ui.date_*` pattern at once, from a full `UiConfig`.
/// Called once at startup and again whenever any of the four fields is
/// committed in the Preferences screen.
pub(crate) fn install_date_patterns(ui: &gitpoulpi_config::UiConfig) {
    let english = ui.date_locale.trim().eq_ignore_ascii_case("english");
    cell!(DATE_ENGLISH, |m: &AtomicBool| m
        .store(english, Ordering::Relaxed));
    let patterns = DatePatterns::from_config(ui);
    let time = std::sync::Arc::new(DatePattern::parse(&patterns.time));
    cell!(TIME_PATTERN, |lock: &TimePattern| {
        if let Ok(mut slot) = lock.write() {
            *slot = Some(time);
        }
    });
    cell!(DATE_PATTERNS, |lock: &RwLock<Option<DatePatterns>>| {
        if let Ok(mut slot) = lock.write() {
            *slot = Some(patterns);
        }
    });
}

/// Test helper: publishes only `date_time_format`; production goes through
/// `install_date_patterns`.
#[cfg(test)]
pub(crate) fn install_date_time_format(pattern: &str) {
    let mut ui = date_patterns_or_default_as_config();
    ui.date_time_format = pattern.to_string();
    install_date_patterns(&ui);
}

#[cfg(test)]
fn date_patterns_or_default_as_config() -> gitpoulpi_config::UiConfig {
    let mut ui = gitpoulpi_config::UiConfig::default();
    let patterns = date_patterns();
    ui.date_format = patterns.short;
    ui.date_time_format = patterns.time;
    ui.date_verbose_format = patterns.verbose;
    ui
}

pub(super) fn date_patterns() -> DatePatterns {
    cell!(DATE_PATTERNS, |lock: &RwLock<Option<DatePatterns>>| lock
        .read()
        .ok()
        .and_then(|slot| slot.clone()))
    .unwrap_or_default()
}

/// The pattern `format_timestamp` should render with. Falls back to the
/// schema default, so a poisoned lock degrades to today's behavior
/// rather than to an empty date column.
pub(crate) fn date_time_pattern() -> String {
    date_patterns().time
}

/// The pattern the File Insight overlay's Blame/History date column renders
/// with (`ui.date_format`; default `YYYY-MM-DD`).
pub(crate) fn date_short_pattern() -> String {
    date_patterns().short
}

/// The pattern the commit-detail Date line, the Date column's hover
/// tooltip and "Copy > Date" all render with (`ui.date_verbose_format`;
/// default `D MMMM YYYY HH:mm`).
pub(crate) fn date_verbose_pattern() -> String {
    date_patterns().verbose
}

/// The language the UI shows for `ui`: `GITPOULPI_LANG` first, then
/// `ui.language`, then (for `auto`) the OS locale, else English.
/// `env` is the raw `GITPOULPI_LANG` value.
pub(crate) fn resolve_language(
    env: Option<&str>,
    ui: &gitpoulpi_config::UiConfig,
) -> gitpoulpi_i18n::Lang {
    let setting = match ui.language {
        gitpoulpi_config::UiLanguage::Auto => None,
        explicit => gitpoulpi_i18n::Lang::parse(explicit.as_str()),
    };
    gitpoulpi_i18n::resolve(env, setting)
}

/// Publishes `ui.language`, `ui.translate_git_terms` and `ui.git_terms_english` to the catalog
/// (`gitpoulpi-i18n`), the live-switch point: called at start-up, from the
/// config-reload hook and when Settings writes either key. Switching bumps
/// `gitpoulpi_i18n::epoch()`; every label is resolved at draw time, so the
/// next frame (already due: the switch comes from an input event or a reload)
/// shows the new language. A no-op when nothing changed.
pub(crate) fn install_language(ui: &gitpoulpi_config::UiConfig) {
    let env = std::env::var(gitpoulpi_i18n::ENV_VAR).ok();
    gitpoulpi_i18n::set(resolve_language(env.as_deref(), ui));
    gitpoulpi_i18n::set_translate_git_terms(ui.translate_git_terms);
    gitpoulpi_i18n::set_english_git_terms(ui.git_terms_english);
}

/// `general.auto_prune`, published like the date patterns so `daemon_link`'s
/// `Fetch`/`FetchAll` handlers can read it without a `Config`. It governs
/// whether those fetches (background auto-fetch and the toolbar's Fetch
/// included) pass `--prune`; the diverged-push dialog's lease-check fetch
/// never prunes, on purpose.
static AUTO_PRUNE: AtomicBool = AtomicBool::new(false);

/// Serializes the tests that touch the process-wide `AUTO_PRUNE`: the Settings
/// toggle test flips it, and the `Fetch` wire golden expects it off.
#[cfg(test)]
pub(crate) static AUTO_PRUNE_TEST_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

pub(crate) fn install_auto_prune(on: bool) {
    AUTO_PRUNE.store(on, Ordering::Relaxed);
}

/// `general.auto_prune_tags`, published like `AUTO_PRUNE`.
static AUTO_PRUNE_TAGS: AtomicBool = AtomicBool::new(false);

pub(crate) fn install_auto_prune_tags(on: bool) {
    AUTO_PRUNE_TAGS.store(on, Ordering::Relaxed);
}

/// What the Fetch / Fetch All paths send: both prune settings, read live.
pub(crate) fn fetch_options() -> gitpoulpi_core::domain::FetchOptions {
    gitpoulpi_core::domain::FetchOptions {
        prune: AUTO_PRUNE.load(Ordering::Relaxed),
        prune_tags: AUTO_PRUNE_TAGS.load(Ordering::Relaxed),
    }
}

/// `ui.log_wip_counts_in_message`, published like `AUTO_PRUNE`.
static LOG_WIP_COUNTS_IN_MESSAGE: AtomicBool = AtomicBool::new(true);

pub(crate) fn install_log_wip_counts_in_message(on: bool) {
    LOG_WIP_COUNTS_IN_MESSAGE.store(on, Ordering::Relaxed);
}

pub(crate) fn log_wip_counts_in_message() -> bool {
    LOG_WIP_COUNTS_IN_MESSAGE.load(Ordering::Relaxed)
}

/// `ui.reveal_on_ref_click`, published like `AUTO_PRUNE`.
static REVEAL_ON_REF_CLICK: AtomicBool = AtomicBool::new(true);

pub(crate) fn install_reveal_on_ref_click(on: bool) {
    REVEAL_ON_REF_CLICK.store(on, Ordering::Relaxed);
}

pub(crate) fn reveal_on_ref_click() -> bool {
    REVEAL_ON_REF_CLICK.load(Ordering::Relaxed)
}

/// `ui.sidebar_sections_scope == global`, published like `AUTO_PRUNE`.
static SIDEBAR_SECTIONS_GLOBAL: AtomicBool = AtomicBool::new(false);

pub(crate) fn install_sidebar_sections_global(on: bool) {
    SIDEBAR_SECTIONS_GLOBAL.store(on, Ordering::Relaxed);
}

pub(crate) fn sidebar_sections_global() -> bool {
    SIDEBAR_SECTIONS_GLOBAL.load(Ordering::Relaxed)
}

/// `general.group_worktree_tabs`, published like `AUTO_PRUNE`. Turning it on
/// also raises a one-shot request the window consumes to regroup the open
/// tabs ([`take_regroup_request`]).
static GROUP_WORKTREE_TABS: AtomicBool = AtomicBool::new(true);
static REGROUP_PENDING: AtomicBool = AtomicBool::new(false);

pub(crate) fn install_group_worktree_tabs(on: bool) {
    let was = GROUP_WORKTREE_TABS.swap(on, Ordering::Relaxed);
    if on && !was {
        REGROUP_PENDING.store(true, Ordering::Relaxed);
    }
}

pub(crate) fn group_worktree_tabs() -> bool {
    GROUP_WORKTREE_TABS.load(Ordering::Relaxed)
}

/// A window opening regroups too, so a session saved before the setting
/// existed (or by `gitpoulpi repo open`) is tidy at the next launch.
pub(crate) fn request_regroup() {
    if group_worktree_tabs() {
        REGROUP_PENDING.store(true, Ordering::Relaxed);
    }
}

/// Whether the setting was just turned on, once.
pub(crate) fn take_regroup_request() -> bool {
    REGROUP_PENDING.swap(false, Ordering::Relaxed)
}

/// `ui.hide_finished_pr_chip`, published like `AUTO_PRUNE`.
static HIDE_FINISHED_PR_CHIP: AtomicBool = AtomicBool::new(true);

pub(crate) fn install_hide_finished_pr_chip(on: bool) {
    HIDE_FINISHED_PR_CHIP.store(on, Ordering::Relaxed);
}

#[cfg(all(test, feature = "hosting"))]
thread_local! {
    /// Per-thread override so parallel tests never fight over the global.
    pub(crate) static HIDE_FINISHED_PR_CHIP_TEST: std::cell::Cell<Option<bool>> =
        const { std::cell::Cell::new(None) };
}

#[cfg(feature = "hosting")]
pub(crate) fn hide_finished_pr_chip() -> bool {
    #[cfg(test)]
    if let Some(v) = HIDE_FINISHED_PR_CHIP_TEST.with(|c| c.get()) {
        return v;
    }
    HIDE_FINISHED_PR_CHIP.load(Ordering::Relaxed)
}

/// `ui.graph_ghost_ref`, published like `AUTO_PRUNE`.
static GRAPH_GHOST_REF: AtomicBool = AtomicBool::new(true);

pub(crate) fn install_graph_ghost_ref(on: bool) {
    GRAPH_GHOST_REF.store(on, Ordering::Relaxed);
}

pub(crate) fn graph_ghost_ref() -> bool {
    GRAPH_GHOST_REF.load(Ordering::Relaxed)
}

// `ui.minimap`: whether the Diff view and the editor draw a minimap beside
// their vertical scrollbar. Published like `AUTO_PRUNE`, so the two views
// (which have no config in hand) and the palette/`M` toggle read one value.
published!(MINIMAP: AtomicBool = AtomicBool::new(false));

pub(crate) fn install_minimap(on: bool) {
    cell!(MINIMAP, |m: &AtomicBool| m.store(on, Ordering::Relaxed));
}

pub(crate) fn minimap() -> bool {
    cell!(MINIMAP, |m: &AtomicBool| m.load(Ordering::Relaxed))
}

// `ui.column_resize`: how the Log, the Radar and the commit column let a
// column boundary be grabbed. Published like `MINIMAP` (per test thread in
// tests); `true` is `header`, the default `line` is `false`.
published!(COLUMN_RESIZE_HEADER: AtomicBool = AtomicBool::new(false));

pub(crate) fn install_column_resize(mode: gitpoulpi_config::ColumnResize) {
    let header = mode == gitpoulpi_config::ColumnResize::Header;
    cell!(COLUMN_RESIZE_HEADER, |m: &AtomicBool| m
        .store(header, Ordering::Relaxed));
}

// `ui.log_columns_layout`: whether the Log fits its columns into the pane or
// lets them keep their widths and scroll sideways. Published like
// `COLUMN_RESIZE_HEADER`; `true` is `scroll`, the default `fit` is `false`.
published!(LOG_COLUMNS_SCROLL: AtomicBool = AtomicBool::new(false));

pub(crate) fn install_log_columns_layout(layout: gitpoulpi_config::LogColumnsLayout) {
    let scroll = layout == gitpoulpi_config::LogColumnsLayout::Scroll;
    cell!(LOG_COLUMNS_SCROLL, |m: &AtomicBool| m
        .store(scroll, Ordering::Relaxed));
}

pub(crate) fn log_columns_layout() -> gitpoulpi_config::LogColumnsLayout {
    if cell!(LOG_COLUMNS_SCROLL, |m: &AtomicBool| m
        .load(Ordering::Relaxed))
    {
        gitpoulpi_config::LogColumnsLayout::Scroll
    } else {
        gitpoulpi_config::LogColumnsLayout::Fit
    }
}

pub(crate) fn column_resize() -> gitpoulpi_config::ColumnResize {
    if cell!(COLUMN_RESIZE_HEADER, |m: &AtomicBool| m
        .load(Ordering::Relaxed))
    {
        gitpoulpi_config::ColumnResize::Header
    } else {
        gitpoulpi_config::ColumnResize::Line
    }
}

// `ui.toolbar_position`, `ui.toolbar_align`, `ui.toolbar_pr_pinned` and
// `ui.statusbar_position`: where the chrome's bands go. Published like
// `MINIMAP`; the frame reads them on every draw, so a change shows at once.
published!(CHROME_LAYOUT: std::sync::Mutex<Option<crate::chrome::ChromeLayout>> = std::sync::Mutex::new(None));

pub(crate) fn install_chrome_layout(ui: &gitpoulpi_config::UiConfig) {
    let layout = crate::chrome::ChromeLayout::from_ui(ui);
    cell!(CHROME_LAYOUT, |m: &std::sync::Mutex<
        Option<crate::chrome::ChromeLayout>,
    >| {
        *m.lock().unwrap_or_else(|e| e.into_inner()) = Some(layout);
    });
}

/// The layout in force (the defaults until something installs one).
pub(crate) fn chrome_layout() -> crate::chrome::ChromeLayout {
    cell!(CHROME_LAYOUT, |m: &std::sync::Mutex<
        Option<crate::chrome::ChromeLayout>,
    >| {
        (*m.lock().unwrap_or_else(|e| e.into_inner())).unwrap_or_default()
    })
}

// `ui.dialog_geometry`: whether a moved or resized dialog keeps its place
// (`dialog_kit::frame`). Stored as 0 session, 1 saved, 2 never.
published!(DIALOG_GEOMETRY: AtomicU8 = AtomicU8::new(0));

pub(crate) fn install_dialog_geometry(mode: gitpoulpi_config::DialogGeometry) {
    use gitpoulpi_config::DialogGeometry as G;
    let n = match mode {
        G::Session => 0,
        G::Saved => 1,
        G::Never => 2,
    };
    cell!(DIALOG_GEOMETRY, |m: &AtomicU8| m.store(n, Ordering::Relaxed));
}

pub(crate) fn dialog_geometry() -> gitpoulpi_config::DialogGeometry {
    use gitpoulpi_config::DialogGeometry as G;
    match cell!(DIALOG_GEOMETRY, |m: &AtomicU8| m.load(Ordering::Relaxed)) {
        1 => G::Saved,
        2 => G::Never,
        _ => G::Session,
    }
}

/// `ui.mermaid_*`: what the Log's Copy as Mermaid gitGraph popup last used,
/// published like `MINIMAP` so the popup (which has no config in hand)
/// opens on it and writes it back when a field changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct MermaidPrefs {
    pub(crate) detail: gitpoulpi_core::domain::MermaidDetail,
    pub(crate) orientation: gitpoulpi_core::domain::MermaidOrientation,
    pub(crate) fence: bool,
    pub(crate) subject_width: usize,
}

impl MermaidPrefs {
    fn from_config(ui: &gitpoulpi_config::UiConfig) -> Self {
        Self {
            detail: ui.mermaid_detail,
            orientation: ui.mermaid_orientation,
            fence: ui.mermaid_fence,
            subject_width: (ui.mermaid_subject_width as usize).max(8),
        }
    }
}

published!(MERMAID: RwLock<Option<MermaidPrefs>> = RwLock::new(None));

pub(crate) fn install_mermaid(ui: &gitpoulpi_config::UiConfig) {
    set_mermaid(MermaidPrefs::from_config(ui));
}

pub(crate) fn set_mermaid(prefs: MermaidPrefs) {
    cell!(MERMAID, |m: &RwLock<Option<MermaidPrefs>>| {
        if let Ok(mut slot) = m.write() {
            *slot = Some(prefs);
        }
    });
}

#[cfg_attr(not(feature = "mermaid"), allow(dead_code))]
pub(crate) fn mermaid() -> MermaidPrefs {
    cell!(MERMAID, |m: &RwLock<Option<MermaidPrefs>>| m
        .read()
        .ok()
        .and_then(|slot| *slot))
    .unwrap_or_else(|| MermaidPrefs::from_config(&gitpoulpi_config::UiConfig::default()))
}

// `ui.blame_side` and `ui.blame_width`, published like `MINIMAP`: the Blame
// view reads them every frame, and a drag of its border writes them back.
published!(BLAME_LEFT: AtomicBool = AtomicBool::new(false));
published!(
    BLAME_WIDTH: std::sync::atomic::AtomicU16 =
        std::sync::atomic::AtomicU16::new(gitpoulpi_config::DEFAULT_BLAME_WIDTH)
);

pub(crate) fn install_blame(ui: &gitpoulpi_config::UiConfig) {
    let left = ui.blame_side == gitpoulpi_config::BlameSide::Left;
    cell!(BLAME_LEFT, |m: &AtomicBool| m
        .store(left, Ordering::Relaxed));
    let width = ui.blame_width_clamped();
    cell!(BLAME_WIDTH, |m: &std::sync::atomic::AtomicU16| m
        .store(width, Ordering::Relaxed));
}

pub(crate) fn blame_left() -> bool {
    cell!(BLAME_LEFT, |m: &AtomicBool| m.load(Ordering::Relaxed))
}

pub(crate) fn blame_width() -> u16 {
    cell!(BLAME_WIDTH, |m: &std::sync::atomic::AtomicU16| m
        .load(Ordering::Relaxed))
}

pub(crate) fn install_blame_width(width: u16) {
    let width = width.clamp(
        gitpoulpi_config::MIN_BLAME_WIDTH,
        gitpoulpi_config::MAX_BLAME_WIDTH,
    );
    cell!(BLAME_WIDTH, |m: &std::sync::atomic::AtomicU16| m
        .store(width, Ordering::Relaxed));
}

/// `ui.image_view`, `ui.image_view_max_mib` and `ui.image_view_text_fallback`,
/// published like `MINIMAP`: the File View's image pane and the daemon-link
/// read (which has no config in hand) read one value. Live: a change
/// applies on the next frame.
static IMAGE_VIEW: AtomicBool = AtomicBool::new(true);
static IMAGE_VIEW_MAX_MIB: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(32);
static IMAGE_CARD_ONLY: AtomicBool = AtomicBool::new(false);
static IMAGE_SYNC: AtomicBool = AtomicBool::new(true);
/// `ui.image_view_pixelated_from` as a scale: 2.0 (the default), 1.0, or
/// infinity for "never" (stored as bits).
static IMAGE_PIXELATED_FROM: std::sync::atomic::AtomicU32 =
    std::sync::atomic::AtomicU32::new(0x4000_0000); // 2.0f32

pub(crate) fn install_image_view(ui: &gitpoulpi_config::UiConfig) {
    IMAGE_VIEW.store(ui.image_view, Ordering::Relaxed);
    IMAGE_VIEW_MAX_MIB.store(ui.image_view_max_mib, Ordering::Relaxed);
    IMAGE_CARD_ONLY.store(
        ui.image_view_text_fallback != gitpoulpi_config::ImageTextFallback::Halfblock,
        Ordering::Relaxed,
    );
    IMAGE_SYNC.store(ui.image_view_sync, Ordering::Relaxed);
    let from = match ui.image_view_pixelated_from {
        gitpoulpi_config::ImagePixelatedFrom::X1 => 1.0f32,
        gitpoulpi_config::ImagePixelatedFrom::X2 => 2.0,
        gitpoulpi_config::ImagePixelatedFrom::Never => f32::INFINITY,
    };
    IMAGE_PIXELATED_FROM.store(from.to_bits(), Ordering::Relaxed);
}

/// `ui.image_view_sync`: the image diff's before and after panes share one
/// zoom and scroll position.
#[cfg_attr(not(feature = "image-view"), allow(dead_code))]
pub(crate) fn image_view_sync() -> bool {
    IMAGE_SYNC.load(Ordering::Relaxed)
}

/// `ui.image_view_pixelated_from` as the scale from which a picture is
/// enlarged as square pixels (infinity: never).
#[cfg_attr(not(feature = "image-view"), allow(dead_code))]
pub(crate) fn image_pixelated_from() -> f32 {
    f32::from_bits(IMAGE_PIXELATED_FROM.load(Ordering::Relaxed))
}

/// Whether a binary file that is an image shows as a picture. Always off in
/// a build without the `image-view` feature.
#[cfg_attr(not(feature = "image-view"), allow(dead_code))]
pub(crate) fn image_view() -> bool {
    crate::build_features::IMAGE_VIEW && IMAGE_VIEW.load(Ordering::Relaxed)
}

/// The byte cap on an image read, from `ui.image_view_max_mib`.
pub(crate) fn image_view_max_bytes() -> u64 {
    u64::from(IMAGE_VIEW_MAX_MIB.load(Ordering::Relaxed)) * 1024 * 1024
}

/// `ui.image_view_text_fallback = card`: no half-block picture where the
/// terminal draws no images, an info card instead.
#[cfg_attr(not(feature = "image-view"), allow(dead_code))]
pub(crate) fn image_card_only() -> bool {
    IMAGE_CARD_ONLY.load(Ordering::Relaxed)
}

/// `general.delete_orig_files_after_merge`, published like `AUTO_PRUNE`. Read
/// by `daemon_link::run_action`'s `Merge` arm after a successful merge.
static DELETE_ORIG_AFTER_MERGE: AtomicBool = AtomicBool::new(false);

pub(crate) fn install_delete_orig_after_merge(on: bool) {
    DELETE_ORIG_AFTER_MERGE.store(on, Ordering::Relaxed);
}

pub(crate) fn delete_orig_after_merge() -> bool {
    DELETE_ORIG_AFTER_MERGE.load(Ordering::Relaxed)
}

/// `log.commit_count`, published like `AUTO_PRUNE`: read each frame by the
/// status bar's total (`graph::total`), so a change applies live.
static COMMIT_COUNT: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);

pub(crate) fn install_commit_count(method: gitpoulpi_config::CommitCountMethod) {
    use gitpoulpi_config::CommitCountMethod as M;
    let n = match method {
        M::SystemGit => 0,
        M::Libgit2 => 1,
        M::Off => 2,
    };
    COMMIT_COUNT.store(n, Ordering::Relaxed);
}

pub(crate) fn commit_count() -> gitpoulpi_config::CommitCountMethod {
    use gitpoulpi_config::CommitCountMethod as M;
    match COMMIT_COUNT.load(Ordering::Relaxed) {
        1 => M::Libgit2,
        2 => M::Off,
        _ => M::SystemGit,
    }
}

/// `general.active_profile`, resolved to the `CommitAuthor` a commit should
/// carry, or `None` when it names no profile (`Config::active_profile`'s
/// "falls back to `git config user.*`" rule). Read by
/// `daemon_link::run_action`'s `Commit` arm.
///
/// A profile with an empty name or email counts as no profile: half an
/// identity would make git refuse the commit (`fatal: empty ident`).
static ACTIVE_PROFILE_AUTHOR: RwLock<Option<gitpoulpi_core::domain::CommitAuthor>> =
    RwLock::new(None);

pub(crate) fn install_active_profile(config: &Config) {
    // The profile combobox owns "which profile applies here": this
    // repository's own choice, else `general.active_profile`.
    crate::profile_bar::install(config);
}

/// Publishes the author the resolved profile gives (`None` for no profile or
/// half an identity).
pub(crate) fn publish_author(profile: Option<gitpoulpi_config::Profile>) {
    let author = profile.and_then(|p| {
        if p.author_name.is_empty() || p.author_email.is_empty() {
            None
        } else {
            Some(gitpoulpi_core::domain::CommitAuthor {
                name: p.author_name,
                email: p.author_email,
            })
        }
    });
    if let Ok(mut slot) = ACTIVE_PROFILE_AUTHOR.write() {
        *slot = author;
    }
}

pub(crate) fn active_profile_author() -> Option<gitpoulpi_core::domain::CommitAuthor> {
    ACTIVE_PROFILE_AUTHOR.read().ok().and_then(|s| s.clone())
}

/// The Diff view's four display toggles (layout, hunks only, whitespace, word
/// wrap) as last set by the user: published at startup from `config.toml`,
/// read by `DiffViewState::resume_last_used` whenever a diff opens, and
/// republished by `crate::persist_diff_view_toggle` on every toggle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DiffViewPrefs {
    pub(crate) layout: gitpoulpi_config::DiffLayout,
    pub(crate) hunks_only: bool,
    pub(crate) show_whitespace: bool,
    pub(crate) word_wrap: bool,
}

impl DiffViewPrefs {
    /// The schema's own defaults - kept as a `const fn` so the static
    /// below needs no lazy initialisation.
    pub(crate) const fn const_default() -> Self {
        Self {
            layout: gitpoulpi_config::DiffLayout::Inline,
            hunks_only: true,
            show_whitespace: true,
            word_wrap: false,
        }
    }
}

published!(DIFF_VIEW_PREFS: RwLock<DiffViewPrefs> = RwLock::new(DiffViewPrefs::const_default()));

/// Publishes `prefs` as the session's "last used" Diff view display state, at
/// startup and from `crate::persist_diff_view_toggle` on each change.
pub(crate) fn install_diff_view_prefs(prefs: DiffViewPrefs) {
    cell!(DIFF_VIEW_PREFS, |lock: &RwLock<DiffViewPrefs>| {
        if let Ok(mut slot) = lock.write() {
            *slot = prefs;
        }
    });
}

published!(DIFF_CONFIG_PATH: RwLock<Option<std::path::PathBuf>> = RwLock::new(None));

/// Publishes the `config.toml` the Diff view's toggles are written to, for the
/// views that toggle from outside the event loop (the review's diff pane).
pub(crate) fn install_diff_config_path(path: Option<std::path::PathBuf>) {
    cell!(DIFF_CONFIG_PATH, |lock: &RwLock<
        Option<std::path::PathBuf>,
    >| {
        if let Ok(mut slot) = lock.write() {
            *slot = path;
        }
    });
}

/// Where a Diff view toggle is written (`None`: no config directory, or a test).
#[cfg_attr(not(all(feature = "hosting", feature = "review")), allow(dead_code))]
pub(crate) fn diff_config_path() -> Option<std::path::PathBuf> {
    cell!(DIFF_CONFIG_PATH, |lock: &RwLock<
        Option<std::path::PathBuf>,
    >| lock
        .read()
        .ok()
        .and_then(|slot| slot.clone()))
}

/// What a freshly opened diff should resume; the schema defaults on a
/// poisoned lock.
pub(crate) fn diff_view_prefs() -> DiffViewPrefs {
    cell!(DIFF_VIEW_PREFS, |lock: &RwLock<DiffViewPrefs>| lock
        .read()
        .map(|slot| *slot)
        .unwrap_or_else(|_| DiffViewPrefs::const_default()))
}

/// Serializes every test that touches `DIFF_VIEW_PREFS`, here and in
/// `diff_view`'s tests.
#[cfg(test)]
pub(crate) static DIFF_VIEW_PREFS_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Serializes every test that installs or depends on the date patterns (a
/// process-wide static), here and in `file_view`'s tests.
#[cfg(test)]
pub(crate) static DATE_PATTERNS_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Renders `ts` (unix seconds, UTC) through a date pattern (`YYYY-MM-DD HH:mm`-style tokens).
///
/// Token set: `YYYY` `YY` `MMMM` `MMM` `MM` `M` `DD` `D` `dddd` `ddd` `HH`
/// `mm` `ss`. Month and weekday names are in the UI language (or English
/// with `ui.date_locale = "english"`).
/// Longest match first, which is the whole subtlety - scanning `MM`
/// before `MMMM` turns `MMMM` into two numeric months rather than a
/// month name. Anything else passes through verbatim, so `DD.MM.YYYY @
/// HH:mm` keeps its separators and its `@`.
///
/// Deliberately no datetime crate: `lib.rs` already carries `civil_from_days`.
pub(crate) fn format_date(pattern: &str, ts: i64) -> String {
    DatePattern::parse(pattern).render(ts)
}

/// One piece of a tokenized date pattern.
#[derive(Debug, Clone, PartialEq, Eq)]
enum DatePiece {
    Year4,
    Year2,
    MonthName,
    MonthAbbr,
    Month2,
    Month,
    Day2,
    Day,
    WeekdayName,
    WeekdayAbbr,
    Hour,
    Minute,
    Second,
    /// The whole `D MMMM YYYY` run (`DD` when `pad`), written the way the
    /// language writes it (`msg::date::day_month_year`: `5 March 2024`,
    /// `2024年3月5日`, `2024년 3월 5일`).
    DayMonthYear {
        pad: bool,
    },
    Literal(String),
}

/// A date pattern tokenized once, so the Log's Date column (one call per
/// visible row, every frame) does no scanning and builds no throwaway
/// token strings. `format_date` is `parse` + `render`; the installed Log
/// pattern is parsed when it is published (`install_date_patterns`).
///
/// Month and weekday names come straight from the catalog as `&'static str`
/// (no allocation, no lookup table to build per frame); a month written next
/// to a day number uses its genitive form (`5 мая`, `5 maja`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DatePattern {
    pieces: Vec<DatePiece>,
    /// The pattern has a day token, so a spelled-out month is the form that
    /// follows a day number.
    with_day: bool,
}

// `ui.date_locale = "english"`: month and day names stay English whatever
// the interface language. Published by `install_date_patterns`.
published!(DATE_ENGLISH: AtomicBool = AtomicBool::new(false));

fn date_english() -> bool {
    cell!(DATE_ENGLISH, |m: &AtomicBool| m.load(Ordering::Relaxed))
}

fn date_text(m: gitpoulpi_i18n::M) -> &'static str {
    if date_english() {
        m.english()
    } else {
        m.get()
    }
}

impl DatePattern {
    /// Longest token first, which is the whole subtlety (see `format_date`).
    pub(crate) fn parse(pattern: &str) -> Self {
        const TOKENS: [(&str, DatePiece); 13] = [
            ("YYYY", DatePiece::Year4),
            ("YY", DatePiece::Year2),
            ("MMMM", DatePiece::MonthName),
            ("MMM", DatePiece::MonthAbbr),
            ("MM", DatePiece::Month2),
            ("M", DatePiece::Month),
            ("DD", DatePiece::Day2),
            ("D", DatePiece::Day),
            ("dddd", DatePiece::WeekdayName),
            ("ddd", DatePiece::WeekdayAbbr),
            ("HH", DatePiece::Hour),
            ("mm", DatePiece::Minute),
            ("ss", DatePiece::Second),
        ];
        let mut pieces: Vec<DatePiece> = Vec::new();
        let mut i = 0;
        'outer: while i < pattern.len() {
            for (token, piece) in &TOKENS {
                if pattern[i..].starts_with(token) {
                    pieces.push(piece.clone());
                    i += token.len();
                    continue 'outer;
                }
            }
            // Not a token: copy one whole char, not one byte.
            let ch = pattern[i..].chars().next().expect("i is a char boundary");
            match pieces.last_mut() {
                Some(DatePiece::Literal(text)) => text.push(ch),
                _ => pieces.push(DatePiece::Literal(ch.to_string())),
            }
            i += ch.len_utf8();
        }
        let with_day = pieces
            .iter()
            .any(|p| matches!(p, DatePiece::Day | DatePiece::Day2));
        // `D MMMM YYYY` as one piece, so the CJK languages can reorder it.
        let space = |p: &DatePiece| matches!(p, DatePiece::Literal(s) if s == " ");
        if let Some(at) = pieces.windows(5).position(|w| {
            matches!(w[0], DatePiece::Day | DatePiece::Day2)
                && space(&w[1])
                && w[2] == DatePiece::MonthName
                && space(&w[3])
                && w[4] == DatePiece::Year4
        }) {
            let pad = pieces[at] == DatePiece::Day2;
            pieces.splice(at..at + 5, [DatePiece::DayMonthYear { pad }]);
        }
        Self { pieces, with_day }
    }

    pub(crate) fn render(&self, ts: i64) -> String {
        use gitpoulpi_i18n::{month_abbr, month_name, month_name_of, weekday_abbr, weekday_name};
        use std::fmt::Write as _;
        let days = ts.div_euclid(86_400);
        let secs = ts.rem_euclid(86_400);
        let (year, month, day) = crate::civil_from_days(days);
        let (hour, minute, second) = (secs / 3600, (secs % 3600) / 60, secs % 60);
        // 1970-01-01 was a Thursday; 0 = Monday.
        let weekday = (days + 3).rem_euclid(7) as u32;
        let mut out = String::with_capacity(24);
        for piece in &self.pieces {
            // Writing into a `String` cannot fail.
            let _ = match piece {
                DatePiece::Year4 => write!(out, "{year:04}"),
                DatePiece::Year2 => write!(out, "{:02}", year.rem_euclid(100)),
                DatePiece::MonthName => {
                    let m = if self.with_day {
                        month_name_of(month)
                    } else {
                        month_name(month)
                    };
                    write!(out, "{}", date_text(m))
                }
                DatePiece::MonthAbbr => write!(out, "{}", date_text(month_abbr(month))),
                DatePiece::Month2 => write!(out, "{month:02}"),
                DatePiece::Month => write!(out, "{month}"),
                DatePiece::Day2 => write!(out, "{day:02}"),
                DatePiece::Day => write!(out, "{day}"),
                DatePiece::WeekdayName => write!(out, "{}", date_text(weekday_name(weekday))),
                DatePiece::WeekdayAbbr => write!(out, "{}", date_text(weekday_abbr(weekday))),
                DatePiece::Hour => write!(out, "{hour:02}"),
                DatePiece::Minute => write!(out, "{minute:02}"),
                DatePiece::Second => write!(out, "{second:02}"),
                DatePiece::DayMonthYear { pad } => {
                    let day = if *pad {
                        format!("{day:02}")
                    } else {
                        day.to_string()
                    };
                    let month = date_text(month_name_of(month));
                    if date_english() {
                        write!(out, "{day} {month} {year:04}")
                    } else {
                        // The language's own order and marks (`2024年3月5日`).
                        out.push_str(&gitpoulpi_i18n::msg::date::day_month_year(
                            day,
                            month,
                            format_args!("{year:04}"),
                        ));
                        Ok(())
                    }
                }
                DatePiece::Literal(text) => {
                    out.push_str(text);
                    Ok(())
                }
            };
        }
        out
    }
}

/// The Log Date column's installed pattern, tokenized when published.
type TimePattern = RwLock<Option<std::sync::Arc<DatePattern>>>;
published!(TIME_PATTERN: TimePattern = RwLock::new(None));

/// `ts` through the installed `ui.date_time_format`, with no per-call
/// tokenization. Same fallback as `date_time_pattern` (the schema default).
pub(crate) fn format_date_time(ts: i64) -> String {
    let cached = cell!(TIME_PATTERN, |lock: &TimePattern| lock
        .read()
        .ok()
        .and_then(|slot| slot.clone()));
    match cached {
        Some(pattern) => pattern.render(ts),
        None => format_date(&date_time_pattern(), ts),
    }
}

#[cfg(test)]
mod language_tests {
    use super::*;
    use gitpoulpi_config::{UiConfig, UiLanguage};
    use gitpoulpi_i18n::Lang;

    fn ui(language: UiLanguage) -> UiConfig {
        UiConfig {
            language,
            ..UiConfig::default()
        }
    }

    #[test]
    fn the_environment_beats_the_setting_and_an_explicit_setting_beats_auto() {
        assert_eq!(resolve_language(Some("de"), &ui(UiLanguage::Fr)), Lang::De);
        assert_eq!(resolve_language(None, &ui(UiLanguage::Fr)), Lang::Fr);
        assert_eq!(resolve_language(None, &ui(UiLanguage::PtPt)), Lang::PtPt);
        assert_eq!(resolve_language(None, &ui(UiLanguage::ZhCn)), Lang::ZhCn);
        assert_eq!(resolve_language(None, &ui(UiLanguage::Ja)), Lang::Ja);
        assert_eq!(
            resolve_language(Some("ko_KR"), &ui(UiLanguage::Fr)),
            Lang::Ko
        );
        // An empty or unknown value is ignored.
        assert_eq!(resolve_language(Some(""), &ui(UiLanguage::Es)), Lang::Es);
        assert_eq!(resolve_language(Some("xx"), &ui(UiLanguage::It)), Lang::It);
        // Auto follows the OS locale, else English: one of the real languages.
        assert_ne!(resolve_language(None, &ui(UiLanguage::Auto)), Lang::Qps);
    }

    #[test]
    fn every_ui_language_value_maps_to_a_language() {
        for l in UiLanguage::ALL {
            if *l != UiLanguage::Auto {
                assert_eq!(Lang::parse(l.as_str()).map(Lang::code), Some(l.as_str()));
            }
        }
    }

    #[test]
    fn month_names_follow_the_language() {
        let _lock = DATE_PATTERNS_TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let pattern = DatePattern::parse("D MMMM YYYY MMM");
        // 2024-03-05 00:00 UTC.
        let ts = 1_709_596_800;
        assert_eq!(pattern.render(ts), "5 March 2024 Mar");
        let _g = gitpoulpi_i18n::override_lang(Lang::Fr);
        assert_eq!(pattern.render(ts), "5 mars 2024 mars");
    }

    /// 2024-03-05 12:34:56 UTC, a Tuesday.
    const TS: i64 = 1_709_642_096;

    fn render_in(lang: Lang, pattern: &str) -> String {
        let _g = gitpoulpi_i18n::override_lang(lang);
        DatePattern::parse(pattern).render(TS)
    }

    /// Dates read the process-wide English-names switch, so every test that
    /// renders one holds the date lock.
    fn in_lang(lang: Lang, pattern: &str) -> String {
        let _lock = DATE_PATTERNS_TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        render_in(lang, pattern)
    }

    #[test]
    fn a_month_after_a_day_number_is_genitive_where_the_language_has_one() {
        assert_eq!(in_lang(Lang::Ru, "D MMMM YYYY"), "5 марта 2024");
        assert_eq!(in_lang(Lang::Uk, "D MMMM YYYY"), "5 березня 2024");
        assert_eq!(in_lang(Lang::Pl, "D MMMM YYYY"), "5 marca 2024");
        assert_eq!(in_lang(Lang::Cs, "D MMMM YYYY"), "5. března 2024");
        // On its own it is the nominative.
        assert_eq!(in_lang(Lang::Ru, "MMMM YYYY"), "март 2024");
        assert_eq!(in_lang(Lang::Uk, "MMMM"), "березень");
        assert_eq!(in_lang(Lang::Pl, "MMMM"), "marzec");
        assert_eq!(in_lang(Lang::Cs, "MMMM"), "březen");
        // May is the case where the two forms differ most.
        let may = 1_715_000_000; // 2024-05-06
        let _lock = DATE_PATTERNS_TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let _g = gitpoulpi_i18n::override_lang(Lang::Ru);
        assert_eq!(DatePattern::parse("D MMMM").render(may), "6 мая");
        assert_eq!(DatePattern::parse("MMMM").render(may), "май");
    }

    #[test]
    fn weekday_names_follow_the_language() {
        for (lang, full, abbr) in [
            (Lang::En, "Tuesday", "Tue"),
            (Lang::Fr, "mardi", "mar."),
            (Lang::Es, "martes", "mar"),
            (Lang::PtBr, "terça-feira", "ter"),
            (Lang::PtPt, "terça-feira", "ter"),
            (Lang::It, "martedì", "mar"),
            (Lang::De, "Dienstag", "Di"),
            (Lang::ZhCn, "星期二", "周二"),
            (Lang::Ja, "火曜日", "火"),
            (Lang::Ko, "화요일", "화"),
            (Lang::Ru, "вторник", "вт"),
            (Lang::Uk, "вівторок", "вт"),
            (Lang::Pl, "wtorek", "wt"),
            (Lang::Cs, "úterý", "út"),
            (Lang::Nl, "dinsdag", "di"),
            (Lang::Sv, "tisdag", "tis"),
            (Lang::Tr, "Salı", "Sal"),
        ] {
            assert_eq!(in_lang(lang, "dddd"), full, "{lang:?}");
            assert_eq!(in_lang(lang, "ddd"), abbr, "{lang:?}");
        }
        // The week starts on a Monday and the epoch was a Thursday.
        let _lock = DATE_PATTERNS_TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let _g = gitpoulpi_i18n::override_lang(Lang::En);
        let p = DatePattern::parse("ddd");
        assert_eq!(p.render(0), "Thu");
        let names: Vec<String> = (0..7).map(|d| p.render(4 * 86_400 + d * 86_400)).collect();
        assert_eq!(names, ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]);
    }

    #[test]
    fn every_language_spells_the_default_verbose_date() {
        for (lang, want) in [
            (Lang::En, "5 March 2024 12:34"),
            (Lang::Fr, "5 mars 2024 12:34"),
            (Lang::Es, "5 de marzo de 2024 12:34"),
            (Lang::PtBr, "5 de março de 2024 12:34"),
            (Lang::PtPt, "5 de março de 2024 12:34"),
            (Lang::It, "5 marzo 2024 12:34"),
            (Lang::De, "5. März 2024 12:34"),
            (Lang::ZhCn, "2024年3月5日 12:34"),
            (Lang::Ja, "2024年3月5日 12:34"),
            (Lang::Ko, "2024년 3월 5일 12:34"),
            (Lang::Ru, "5 марта 2024 12:34"),
            (Lang::Uk, "5 березня 2024 12:34"),
            (Lang::Pl, "5 marca 2024 12:34"),
            (Lang::Cs, "5. března 2024 12:34"),
            (Lang::Nl, "5 maart 2024 12:34"),
            (Lang::Sv, "5 mars 2024 12:34"),
            (Lang::Tr, "5 Mart 2024 12:34"),
        ] {
            assert_eq!(in_lang(lang, "D MMMM YYYY HH:mm"), want, "{lang:?}");
        }
        // Zero-padded day, CJK order too; other patterns keep their order.
        assert_eq!(in_lang(Lang::Ja, "DD MMMM YYYY"), "2024年3月05日");
        assert_eq!(in_lang(Lang::Ja, "YYYY-MM-DD"), "2024-03-05");
        assert_eq!(in_lang(Lang::Ja, "MMMM D"), "3月 5");
    }

    #[test]
    fn the_english_date_locale_ignores_the_ui_language() {
        let _lock = DATE_PATTERNS_TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let mut ui = UiConfig {
            date_locale: "english".into(),
            ..UiConfig::default()
        };
        install_date_patterns(&ui);
        assert_eq!(
            render_in(Lang::Ru, "dddd D MMMM YYYY"),
            "Tuesday 5 March 2024"
        );
        assert_eq!(render_in(Lang::Ja, "D MMMM YYYY"), "5 March 2024");
        ui.date_locale = "system".into();
        install_date_patterns(&ui);
        assert_eq!(render_in(Lang::Ru, "D MMMM YYYY"), "5 марта 2024");
    }

    /// The Log renders a date per visible row per frame: a parsed pattern is
    /// pure catalog reads and one small `String` per call.
    #[test]
    fn rendering_dates_stays_cheap() {
        let _lock = DATE_PATTERNS_TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let pattern = DatePattern::parse("D MMMM YYYY HH:mm");
        let _g = gitpoulpi_i18n::override_lang(Lang::Ru);
        let started = std::time::Instant::now();
        let mut total = 0usize;
        for i in 0..50_000i64 {
            total += pattern.render(TS + i * 3600).len();
        }
        assert!(total > 0);
        // ~1 microsecond each in release; the budget is two orders looser
        // so a loaded debug build never flakes.
        assert!(
            started.elapsed() < std::time::Duration::from_secs(5),
            "50k renders took {:?}",
            started.elapsed()
        );
    }
}
