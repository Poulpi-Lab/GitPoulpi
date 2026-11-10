//! The Preferences pages' field builders for the `ui.*` rows (a pool, see `pages.rs`) and Git & encoding.

use super::*;
use gitpoulpi_i18n::msg;

/// The encodings the Default encoding dropdowns offer (`encoding_rs` labels).
const ENCODINGS: &[&str] = &[
    "utf-8",
    "windows-1252",
    "iso-8859-1",
    "iso-8859-2",
    "iso-8859-15",
    "windows-1250",
    "windows-1251",
    "koi8-r",
    "shift_jis",
    "euc-jp",
    "gbk",
    "big5",
    "euc-kr",
    "utf-16le",
    "utf-16be",
];

/// [`ENCODINGS`], plus `current` when a hand-edited file holds something
/// else, so the dropdown never hides what is in force.
pub(super) fn encoding_options(current: &str) -> Vec<String> {
    let mut options: Vec<String> = ENCODINGS.iter().map(|e| (*e).to_string()).collect();
    if !current.is_empty() && !options.iter().any(|o| o.eq_ignore_ascii_case(current)) {
        options.push(current.to_string());
    }
    options
}

/// The values the Language combobox lists, in order: `Auto`, every shipped
/// language built into this binary, and `current` when it is a
/// wired-but-unshipped or not-built value read from `config.toml` (shown with
/// `(soon)` / `(not built into this binary)`; it shows English).
pub(crate) fn language_choices(current: UiLanguage) -> Vec<UiLanguage> {
    let shipped = |l: UiLanguage| {
        gitpoulpi_i18n::Lang::parse(l.as_str()).is_some_and(gitpoulpi_i18n::Lang::is_shipped)
    };
    UiLanguage::ALL
        .iter()
        .copied()
        .filter(|l| *l == UiLanguage::Auto || shipped(*l) || *l == current)
        .collect()
}

/// The Language combobox's option labels, in `language_choices` order:
/// `Auto (<detected language>)`, then each language's endonym.
pub(crate) fn language_options(current: UiLanguage) -> Vec<String> {
    use gitpoulpi_i18n::msg::settings as m;
    let detected = gitpoulpi_i18n::system_lang();
    language_choices(current)
        .into_iter()
        .map(|l| match l {
            UiLanguage::Auto => m::language_auto(detected.native_name()),
            other => {
                let shipped = gitpoulpi_i18n::Lang::parse(other.as_str())
                    .is_some_and(gitpoulpi_i18n::Lang::is_shipped);
                let not_built =
                    gitpoulpi_i18n::Lang::parse(other.as_str()).is_some_and(|l| !l.is_built());
                if shipped {
                    other.endonym().to_string()
                } else if not_built {
                    // Configured, but this binary was built without it.
                    m::language_not_built(other.endonym())
                } else {
                    m::language_soon(other.endonym())
                }
            }
        })
        .collect()
}

impl PreferencesState {
    fn language_field(&self) -> Field {
        use gitpoulpi_i18n::msg::settings as m;
        let current = self.config.ui.language;
        let mut help = m::LANGUAGE_HELP.get().to_string();
        if gitpoulpi_i18n::env_override().is_some() {
            help.push(' ');
            help.push_str(m::LANGUAGE_ENV_OVERRIDE.get());
        }
        Field::global(
            m::LANGUAGE.get(),
            "ui.language",
            Control::Choice {
                options: language_options(current),
                index: language_choices(current)
                    .iter()
                    .position(|l| *l == current)
                    .unwrap_or(0),
            },
        )
        .with_help(&help)
    }

    pub(super) fn ui_rows(&self) -> Vec<Field> {
        let ui = &self.config.ui;
        let mut fields = vec![
            self.language_field(),
            Field::global(
                gitpoulpi_i18n::msg::settings::TRANSLATE_GIT_TERMS.get(),
                "ui.translate_git_terms",
                Control::Toggle(ui.translate_git_terms),
            )
            .with_help(gitpoulpi_i18n::msg::settings::TRANSLATE_GIT_TERMS_HELP.get()),
        ];
        // ja, ko and zh-CN only: force the English git words.
        if gitpoulpi_i18n::current().has_english_git_option() {
            use gitpoulpi_i18n::msg::settings as m;
            let mut field = Field::global(
                m::GIT_TERMS_ENGLISH.get(),
                "ui.git_terms_english",
                Control::Toggle(ui.git_terms_english),
            )
            .with_help(m::GIT_TERMS_ENGLISH_HELP.get());
            if ui.translate_git_terms {
                field = field.coming(m::GIT_TERMS_ENGLISH_OFF.get());
            }
            fields.push(field);
        }
        fields.push(
            Field::global(
                msg::settings_ui::THEME.get(),
                "ui.theme",
                Control::Combo {
                    options: self.theme_options(),
                    value: ui.theme_slug().to_string(),
                    custom: false,
                },
            )
            .with_help(msg::settings_ui::THEME_HELP.get()),
        );
        fields.push(self.theme_swatch_row());
        fields.extend([
            Field::global(
                msg::settings_ui::GRAPH_LANE_BACKGROUND.get(),
                "ui.graph_lane_background",
                Control::Choice {
                    options: GraphLaneBackground::ALL
                        .iter()
                        .map(|b| b.as_str().into())
                        .collect(),
                    index: GraphLaneBackground::ALL
                        .iter()
                        .position(|b| *b == ui.graph_lane_background)
                        .unwrap_or(0),
                },
            )
            .with_help(msg::settings_ui::GRAPH_LANE_BACKGROUND_HELP.get()),
            {
                let tint = Field::global(
                    msg::settings_ui::GRAPH_ROW_TINT_STRENGTH.get(),
                    "ui.graph_row_tint_strength",
                    Control::Choice {
                        options: GraphRowTintStrength::ALL
                            .iter()
                            .map(|b| b.as_str().into())
                            .collect(),
                        index: GraphRowTintStrength::ALL
                            .iter()
                            .position(|b| *b == ui.graph_row_tint_strength)
                            .unwrap_or(0),
                    },
                )
                .with_help(msg::settings_ui::GRAPH_ROW_TINT_STRENGTH_HELP.get());
                if ui.graph_lane_background == GraphLaneBackground::Off {
                    tint.coming(msg::settings_ui::GRAPH_ROW_TINT_STRENGTH_COMING.get())
                } else {
                    tint
                }
            },
            Field::global(
                msg::settings_ui::TOOLBAR_POSITION.get(),
                "ui.toolbar_position",
                Control::Choice {
                    options: ToolbarPosition::ALL
                        .iter()
                        .map(|b| b.as_str().into())
                        .collect(),
                    index: ToolbarPosition::ALL
                        .iter()
                        .position(|b| *b == ui.toolbar_position)
                        .unwrap_or(0),
                },
            )
            .with_help(msg::settings_ui::TOOLBAR_POSITION_HELP.get()),
            Field::global(
                msg::settings_ui::TOOLBAR_ALIGN.get(),
                "ui.toolbar_align",
                Control::Choice {
                    options: ToolbarAlign::ALL
                        .iter()
                        .map(|b| b.as_str().into())
                        .collect(),
                    index: ToolbarAlign::ALL
                        .iter()
                        .position(|b| *b == ui.toolbar_align)
                        .unwrap_or(0),
                },
            )
            .with_help(msg::settings_ui::TOOLBAR_ALIGN_HELP.get()),
            Field::global(
                msg::settings_ui::TOOLBAR_PR_PINNED.get(),
                "ui.toolbar_pr_pinned",
                Control::Toggle(ui.toolbar_pr_pinned),
            )
            .with_help(msg::settings_ui::TOOLBAR_PR_PINNED_HELP.get()),
            Field::global(
                msg::settings_ui::STATUSBAR_POSITION.get(),
                "ui.statusbar_position",
                Control::Choice {
                    options: StatusbarPosition::ALL
                        .iter()
                        .map(|b| b.as_str().into())
                        .collect(),
                    index: StatusbarPosition::ALL
                        .iter()
                        .position(|b| *b == ui.statusbar_position)
                        .unwrap_or(0),
                },
            )
            .with_help(msg::settings_ui::STATUSBAR_POSITION_HELP.get()),
            Field::global(
                msg::settings_ui::TABS_POSITION.get(),
                "ui.tabs_position",
                Control::Choice {
                    options: TabsPosition::ALL
                        .iter()
                        .map(|b| b.as_str().into())
                        .collect(),
                    index: TabsPosition::ALL
                        .iter()
                        .position(|b| *b == ui.tabs_position)
                        .unwrap_or(0),
                },
            )
            .with_help(msg::settings_ui::TABS_POSITION_HELP.get()),
            Field::global(
                msg::settings_ui::GRAPH_COLOR_THEME.get(),
                "ui.graph_color_theme",
                Control::Combo {
                    options: self.lane_options(),
                    value: ui.graph_color_slug().to_string(),
                    custom: false,
                },
            )
            .with_help(msg::settings_ui::GRAPH_COLOR_THEME_HELP.get()),
            Field::global(
                msg::settings_ui::GRAPH_RENDERER.get(),
                "ui.graph_renderer",
                Control::Choice {
                    options: GraphRenderer::ALL
                        .iter()
                        .map(|r| r.as_str().into())
                        .collect(),
                    index: GraphRenderer::ALL
                        .iter()
                        .position(|r| *r == ui.graph_renderer)
                        .unwrap_or(0),
                },
            )
            .with_help(msg::settings_ui::GRAPH_RENDERER_HELP.get()),
            Field::global(
                msg::settings_ui::IMAGE_PROTOCOL.get(),
                "ui.image_protocol",
                Control::Choice {
                    options: ImageProtocol::ALL
                        .iter()
                        .map(|p| p.as_str().into())
                        .collect(),
                    index: ImageProtocol::ALL
                        .iter()
                        .position(|p| *p == ui.image_protocol)
                        .unwrap_or(0),
                },
            )
            .with_help(msg::settings_ui::IMAGE_PROTOCOL_HELP.get()),
            Field::global(
                msg::settings_ui::COLOR_DEPTH.get(),
                "ui.color_depth",
                Control::Choice {
                    options: ColorDepth::ALL.iter().map(|k| k.as_str().into()).collect(),
                    index: ColorDepth::ALL
                        .iter()
                        .position(|k| *k == ui.color_depth)
                        .unwrap_or(0),
                },
            )
            .with_help(msg::settings_ui::COLOR_DEPTH_HELP.get()),
            Field::global(
                msg::settings_ui::KITTY_KEYBOARD.get(),
                "ui.kitty_keyboard",
                Control::Choice {
                    options: KittyKeyboard::ALL
                        .iter()
                        .map(|k| k.as_str().into())
                        .collect(),
                    index: KittyKeyboard::ALL
                        .iter()
                        .position(|k| *k == ui.kitty_keyboard)
                        .unwrap_or(0),
                },
            )
            .with_help(msg::settings_ui::KITTY_KEYBOARD_HELP.get()),
            Field::global(
                msg::settings_ui::GRAPH_SHOW_STASHES.get(),
                "ui.graph_show_stashes",
                Control::Toggle(ui.graph_show_stashes),
            )
            .with_help(msg::settings_ui::GRAPH_SHOW_STASHES_HELP.get()),
            Field::global(
                msg::settings_ui::GRAPH_GHOST_REF.get(),
                "ui.graph_ghost_ref",
                Control::Toggle(ui.graph_ghost_ref),
            )
            .with_help(msg::settings_ui::GRAPH_GHOST_REF_HELP.get()),
            Field::global(
                msg::settings_ui::REF_CONNECTOR.get(),
                "ui.ref_connector",
                Control::Toggle(ui.ref_connector),
            )
            .with_help(msg::settings_ui::REF_CONNECTOR_HELP.get()),
            Field::global(
                msg::settings_ui::LOG_WIP_COUNTS_IN_MESSAGE.get(),
                "ui.log_wip_counts_in_message",
                Control::Toggle(ui.log_wip_counts_in_message),
            )
            .with_help(msg::settings_ui::LOG_WIP_COUNTS_IN_MESSAGE_HELP.get()),
            Field::global(
                msg::settings_ui::AVATARS.get(),
                "ui.avatars",
                Control::Choice {
                    options: AvatarPlacement::ALL
                        .iter()
                        .map(|p| p.as_str().into())
                        .collect(),
                    index: AvatarPlacement::ALL
                        .iter()
                        .position(|p| *p == ui.avatar_placement())
                        .unwrap_or(0),
                },
            )
            .with_help(msg::settings_ui::AVATARS_HELP.get()),
            Field::global(
                msg::settings_ui::AVATAR_SOURCES.get(),
                "ui.avatar_sources",
                Control::Choice {
                    options: AvatarSources::ALL
                        .iter()
                        .map(|p| p.as_str().into())
                        .collect(),
                    index: AvatarSources::ALL
                        .iter()
                        .position(|p| *p == ui.avatar_sources)
                        .unwrap_or(0),
                },
            )
            .with_help(msg::settings_ui::AVATAR_SOURCES_HELP.get()),
            Field::global(
                msg::settings_ui::AVATAR_FALLBACK.get(),
                "ui.avatar_fallback",
                Control::Choice {
                    options: AvatarFallback::ALL
                        .iter()
                        .map(|p| p.as_str().into())
                        .collect(),
                    index: AvatarFallback::ALL
                        .iter()
                        .position(|p| *p == ui.avatar_fallback)
                        .unwrap_or(0),
                },
            )
            .with_help(msg::settings_ui::AVATAR_FALLBACK_HELP.get()),
            Field::global(
                msg::settings_ui::COMPOSER_FILES.get(),
                "ui.composer_files",
                Control::Choice {
                    options: ComposerFiles::ALL
                        .iter()
                        .map(|p| p.as_str().into())
                        .collect(),
                    index: ComposerFiles::ALL
                        .iter()
                        .position(|p| *p == ui.composer_files)
                        .unwrap_or(0),
                },
            )
            .with_help(msg::settings_ui::COMPOSER_FILES_HELP.get()),
            Field::global(
                msg::settings_ui::REF_TREE.get(),
                "ui.ref_tree",
                Control::Toggle(ui.ref_tree),
            )
            .with_help(msg::settings_ui::REF_TREE_HELP.get()),
            Field::global(
                msg::settings_ui::REVEAL_ON_REF_CLICK.get(),
                "ui.reveal_on_ref_click",
                Control::Toggle(ui.reveal_on_ref_click),
            )
            .with_help(msg::settings_ui::REVEAL_ON_REF_CLICK_HELP.get()),
            Field::global(
                msg::settings_ui::MOUSE_ENABLED.get(),
                "ui.mouse_enabled",
                Control::Toggle(ui.mouse_enabled),
            )
            .with_help(msg::settings_ui::MOUSE_ENABLED_HELP.get()),
            Field::global(
                msg::settings_ui::MOUSE_SCROLL_LINES.get(),
                "ui.mouse_scroll_lines",
                Control::Number {
                    value: i64::from(ui.mouse_scroll_lines),
                    min: i64::from(MIN_MOUSE_SCROLL_LINES),
                    max: i64::from(MAX_MOUSE_SCROLL_LINES),
                },
            )
            .with_help(msg::settings_ui::MOUSE_SCROLL_LINES_HELP.get()),
            Field::global(
                msg::settings_ui::DATE_LOCALE.get(),
                "ui.date_locale",
                Control::Choice {
                    options: vec!["system".into(), "english".into()],
                    index: usize::from(ui.date_locale.eq_ignore_ascii_case("english")),
                },
            )
            .with_help(msg::settings_ui::DATE_LOCALE_HELP.get()),
        ]);

        // Without `avatar-fetch` there is no source to choose: initials
        // (or the identicon) only.
        if !crate::build_features::AVATAR_FETCH {
            let label = msg::settings_ui::AVATAR_SOURCES.get();
            fields.retain(|f| f.label != label);
        }

        for (label, key, pattern, help) in [
            (
                msg::settings_ui::DATE_FORMAT.get(),
                "ui.date_format",
                &ui.date_format,
                msg::settings_ui::DATE_FORMAT_HELP.get(),
            ),
            (
                msg::settings_ui::DATE_TIME_FORMAT.get(),
                "ui.date_time_format",
                &ui.date_time_format,
                msg::settings_ui::DATE_TIME_FORMAT_HELP.get(),
            ),
            (
                msg::settings_ui::DATE_VERBOSE_FORMAT.get(),
                "ui.date_verbose_format",
                &ui.date_verbose_format,
                msg::settings_ui::DATE_VERBOSE_FORMAT_HELP.get(),
            ),
        ] {
            fields.push(Field::global(label, key, Control::Text(pattern.clone())).with_help(help));
            fields.push(Field {
                label: format!("  {}", msg::settings_ui::PREVIEW.get()),
                control: Control::Info(format_date(pattern, self.now)),
                help: None,
                warning: None,
                danger: false,
                target: Target::None,
                live: true,
                coming: None,
                placeholder: None,
            });
        }

        fields.push(
            Field::global(
                msg::settings_ui::DIFF_LAYOUT.get(),
                "ui.diff_layout",
                Control::Choice {
                    options: DiffLayout::ALL.iter().map(|l| l.as_str().into()).collect(),
                    index: DiffLayout::ALL
                        .iter()
                        .position(|l| *l == ui.diff_layout)
                        .unwrap_or(0),
                },
            )
            .with_help(msg::settings_ui::DIFF_LAYOUT_HELP.get()),
        );
        fields.push(
            Field::global(
                msg::settings_ui::DIFF_HUNKS_ONLY.get(),
                "ui.diff_hunks_only",
                Control::Toggle(ui.diff_hunks_only),
            )
            .with_help(msg::settings_ui::DIFF_HUNKS_ONLY_HELP.get()),
        );
        fields.push(
            Field::global(
                msg::settings_ui::DIFF_SHOW_WHITESPACE.get(),
                "ui.diff_show_whitespace",
                Control::Toggle(ui.diff_show_whitespace),
            )
            .with_help(msg::settings_ui::DIFF_SHOW_WHITESPACE_HELP.get()),
        );
        let input_labels: Vec<String> = crate::INPUT_PRESETS
            .iter()
            .map(|p| (*p).to_string())
            .collect();
        fields.push(
            Field::global(
                msg::settings_ui::INPUT_COLOR.get(),
                "ui.input_color",
                Control::Choice {
                    options: input_labels,
                    index: crate::INPUT_PRESETS
                        .iter()
                        .position(|p| {
                            p.eq_ignore_ascii_case(&ui.input_color)
                                || (*p == "gray" && ui.input_color.eq_ignore_ascii_case("grey"))
                        })
                        .unwrap_or(0),
                },
            )
            .with_help(msg::settings_ui::INPUT_COLOR_HELP.get()),
        );
        fields.push(
            Field::global(
                msg::settings_ui::INPUT_COLOR_CUSTOM.get(),
                "ui.input_color_custom",
                Control::Text(ui.input_color_custom.clone()),
            )
            .with_help(msg::settings_ui::INPUT_COLOR_CUSTOM_HELP.get()),
        );
        fields.push(
            Field::global(
                msg::settings_ui::MINIMAP.get(),
                "ui.minimap",
                Control::Toggle(ui.minimap),
            )
            .with_help(msg::settings_ui::MINIMAP_HELP.get()),
        );
        fields.push(
            Field::global(
                msg::settings_ui::SIDEBAR_SECTIONS_SCOPE.get(),
                "ui.sidebar_sections_scope",
                Control::Choice {
                    options: gitpoulpi_config::SidebarSectionsScope::ALL
                        .iter()
                        .map(|p| p.as_str().into())
                        .collect(),
                    index: gitpoulpi_config::SidebarSectionsScope::ALL
                        .iter()
                        .position(|p| *p == ui.sidebar_sections_scope)
                        .unwrap_or(0),
                },
            )
            .with_help(msg::settings_ui::SIDEBAR_SECTIONS_SCOPE_HELP.get()),
        );
        fields.push(
            Field::global(
                msg::settings_ui::COLUMN_RESIZE.get(),
                "ui.column_resize",
                Control::Choice {
                    options: gitpoulpi_config::ColumnResize::ALL
                        .iter()
                        .map(|p| p.as_str().into())
                        .collect(),
                    index: gitpoulpi_config::ColumnResize::ALL
                        .iter()
                        .position(|p| *p == ui.column_resize)
                        .unwrap_or(0),
                },
            )
            .with_help(msg::settings_ui::COLUMN_RESIZE_HELP.get()),
        );
        fields.push(
            Field::global(
                msg::settings_ui::LOG_COLUMNS_LAYOUT.get(),
                "ui.log_columns_layout",
                Control::Choice {
                    options: gitpoulpi_config::LogColumnsLayout::ALL
                        .iter()
                        .map(|p| p.as_str().into())
                        .collect(),
                    index: gitpoulpi_config::LogColumnsLayout::ALL
                        .iter()
                        .position(|p| *p == ui.log_columns_layout)
                        .unwrap_or(0),
                },
            )
            .with_help(msg::settings_ui::LOG_COLUMNS_LAYOUT_HELP.get()),
        );
        fields.push(
            Field::global(
                msg::settings_ui::BLAME_SIDE.get(),
                "ui.blame_side",
                Control::Choice {
                    options: gitpoulpi_config::BlameSide::ALL
                        .iter()
                        .map(|p| p.as_str().into())
                        .collect(),
                    index: gitpoulpi_config::BlameSide::ALL
                        .iter()
                        .position(|p| *p == ui.blame_side)
                        .unwrap_or(0),
                },
            )
            .with_help(msg::settings_ui::BLAME_SIDE_HELP.get()),
        );
        fields.push(
            Field::global(
                msg::settings_ui::BLAME_WIDTH.get(),
                "ui.blame_width",
                Control::Number {
                    value: i64::from(ui.blame_width_clamped()),
                    min: i64::from(gitpoulpi_config::MIN_BLAME_WIDTH),
                    max: i64::from(gitpoulpi_config::MAX_BLAME_WIDTH),
                },
            )
            .with_help(msg::settings_ui::BLAME_WIDTH_HELP.get()),
        );
        fields.extend(self.image_view_rows());
        fields
    }

    /// The Images page's viewer settings (placed by `pages::IMAGES`).
    fn image_view_rows(&self) -> Vec<Field> {
        use gitpoulpi_config::{ImagePixelatedFrom, ImageTextFallback};
        use gitpoulpi_i18n::msg::settings_images as t;
        let ui = &self.config.ui;
        vec![
            Field::global(
                t::IMAGE_VIEW.get(),
                "ui.image_view",
                Control::Toggle(ui.image_view),
            )
            .with_help(t::IMAGE_VIEW_HELP.get()),
            Field::global(
                t::TEXT_FALLBACK.get(),
                "ui.image_view_text_fallback",
                Control::Choice {
                    options: ImageTextFallback::ALL
                        .iter()
                        .map(|v| v.as_str().into())
                        .collect(),
                    index: ImageTextFallback::ALL
                        .iter()
                        .position(|v| *v == ui.image_view_text_fallback)
                        .unwrap_or(0),
                },
            )
            .with_help(t::TEXT_FALLBACK_HELP.get()),
            self.image_max_field(),
            Field::global(
                t::PIXELATED_FROM.get(),
                "ui.image_view_pixelated_from",
                Control::Choice {
                    options: ImagePixelatedFrom::ALL
                        .iter()
                        .map(|v| v.as_str().into())
                        .collect(),
                    index: ImagePixelatedFrom::ALL
                        .iter()
                        .position(|v| *v == ui.image_view_pixelated_from)
                        .unwrap_or(0),
                },
            )
            .with_help(t::PIXELATED_FROM_HELP.get()),
            Field::global(
                t::SYNC.get(),
                "ui.image_view_sync",
                Control::Toggle(ui.image_view_sync),
            )
            .with_help(t::SYNC_HELP.get()),
        ]
    }

    pub(super) fn git_fields(&self) -> Vec<Field> {
        let prefs = &self.repo_state.preferences;
        let git = &self.config.git;
        let mut fields = vec![
            Field::global(
                msg::settings_git::DEFAULT_BACKEND.get(),
                "general.default_backend",
                Control::Choice {
                    options: vec!["git2".into(), "system".into()],
                    index: match self.config.general.default_backend {
                        DefaultBackend::Git2 => 0,
                        DefaultBackend::System => 1,
                    },
                },
            )
            .with_help(msg::settings_git::DEFAULT_BACKEND_HELP.get())
            .with_warning(msg::settings_git::DEFAULT_BACKEND_WARNING.get()),
            Field::global(
                msg::settings_git::PULL_MODE.get(),
                "git.pull_mode",
                Control::Choice {
                    options: PullButtonMode::ALL
                        .iter()
                        .map(|m| m.as_str().into())
                        .collect(),
                    index: PullButtonMode::ALL
                        .iter()
                        .position(|m| *m == git.pull_mode)
                        .unwrap_or(0),
                },
            )
            .with_help(&msg::settings_git::pull_mode_help(crate::keymap::paren(
                &crate::keymap::current(),
                crate::keymap::Action::GitPull,
            ))),
            Field::global(
                msg::settings_git::MERGE_FAST_FORWARD.get(),
                "git.merge_fast_forward",
                Control::Choice {
                    options: MergeFastForward::ALL
                        .iter()
                        .map(|m| m.as_str().into())
                        .collect(),
                    index: MergeFastForward::ALL
                        .iter()
                        .position(|m| *m == git.merge_fast_forward)
                        .unwrap_or(0),
                },
            )
            .with_help(msg::settings_git::MERGE_FAST_FORWARD_HELP.get()),
            Field::global(
                msg::settings_git::DEFAULT_ENCODING.get(),
                "git.default_encoding",
                Control::Choice {
                    options: encoding_options(&git.default_encoding),
                    index: encoding_options(&git.default_encoding)
                        .iter()
                        .position(|o| o.eq_ignore_ascii_case(&git.default_encoding))
                        .unwrap_or(0),
                },
            )
            .with_help(msg::settings_git::DEFAULT_ENCODING_HELP.get()),
            Field::global(
                msg::settings_git::PUSH_BEHAVIOR.get(),
                "git.push_behavior",
                Control::Choice {
                    options: PushBehavior::ALL
                        .iter()
                        .map(|b| b.as_str().into())
                        .collect(),
                    index: PushBehavior::ALL
                        .iter()
                        .position(|b| *b == git.push_behavior)
                        .unwrap_or(0),
                },
            )
            .with_help(msg::settings_git::PUSH_BEHAVIOR_HELP.get()),
            Field::heading(&msg::settings_git::repo_heading(&self.repo_label)),
            Field::repo(
                msg::settings_git::DEFAULT_ENCODING.get(),
                RepoField::Encoding,
                Control::Choice {
                    options: std::iter::once(msg::settings_git::INHERITED.get().to_string())
                        .chain(encoding_options(prefs.encoding.as_deref().unwrap_or("")))
                        .collect(),
                    index: prefs
                        .encoding
                        .as_deref()
                        .and_then(|v| {
                            encoding_options(v)
                                .iter()
                                .position(|o| o.eq_ignore_ascii_case(v))
                        })
                        .map(|i| i + 1)
                        .unwrap_or(0),
                },
            )
            .live()
            .with_help(msg::settings_git::REPO_ENCODING_HELP.get()),
            git_config_field(
                "core.autocrlf",
                RepoField::AutoCrlf,
                &self.repo_config,
                msg::settings_git::CORE_AUTOCRLF_HELP.get(),
            ),
            git_config_field(
                "core.eol",
                RepoField::Eol,
                &self.repo_config,
                msg::settings_git::CORE_EOL_HELP.get(),
            ),
        ];
        // The LFS rows exist only in a build that has LFS.
        if crate::build_features::LFS {
            fields.extend([
                Field::repo(
                    msg::settings_git::GIT_LFS.get(),
                    RepoField::Lfs,
                    Control::Toggle(prefs.lfs_enabled.unwrap_or(git.lfs_enabled)),
                )
                .live()
                .with_help(msg::settings_git::GIT_LFS_HELP.get()),
                Field::global(
                    msg::settings_git::LFS_AUTOFIX_FILTERS.get(),
                    "git.lfs_autofix_filters",
                    Control::Toggle(git.lfs_autofix_filters),
                )
                .with_help(msg::settings_git::LFS_AUTOFIX_FILTERS_HELP.get()),
                Field::plain(
                    msg::settings_git::GIT_LFS_PROGRAM.get(),
                    Control::Info(
                        self.lfs_program.clone().unwrap_or_else(|| {
                            msg::settings_git::LFS_NOT_CHECKED.get().to_string()
                        }),
                    ),
                    Target::None,
                )
                .with_help(msg::settings_git::GIT_LFS_PROGRAM_HELP.get()),
                Field::button(
                    msg::settings_git::INITIALIZE_LFS_FOR_THIS_REPOSITORY.get(),
                    PrefAction::InitializeLfs,
                )
                .with_help(msg::settings_git::INITIALIZE_LFS_FOR_THIS_REPOSITORY_HELP.get()),
            ]);
        }
        fields
    }
}

/// The values `key` accepts (after "(inherited)", which clears it).
pub(super) fn git_config_values(key: &str) -> &'static [&'static str] {
    gitpoulpi_core::repo_config::REPO_CONFIG_KEYS
        .iter()
        .find(|(k, _)| *k == key)
        .map_or(&[][..], |(_, v)| *v)
}

/// A Git & Encoding row for one of git's own repository-local settings:
/// the shown value is what git reports (`values`), not what a file remembers.
fn git_config_field(
    key: &'static str,
    field: RepoField,
    values: &std::collections::BTreeMap<String, Option<String>>,
    help: &str,
) -> Field {
    let accepted = git_config_values(key);
    let current = values.get(key).and_then(|v| v.as_deref());
    Field::repo(
        key,
        field,
        Control::Choice {
            options: std::iter::once(msg::settings_git::INHERITED.get().to_string())
                .chain(accepted.iter().map(|v| (*v).to_string()))
                .collect(),
            index: current
                .and_then(|v| accepted.iter().position(|a| *a == v))
                .map_or(0, |i| i + 1),
        },
    )
    .live()
    .with_help(help)
}
