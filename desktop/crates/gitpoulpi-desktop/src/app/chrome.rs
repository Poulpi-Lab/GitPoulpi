//! The window's chrome around the panes: title bar, tab row, toolbar row and status line.

use gitpoulpi_i18n::msg;
use gpui_kit::assets::IconName as AssetIcon;
use gpui_kit::component::{
    button::{Button, ButtonVariants as _},
    h_flex,
    menu::DropdownMenu as _,
    Icon, Sizable as _, TitleBar,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    div, img, px, Anchor, Context, InteractiveElement as _, IntoElement, ParentElement as _,
    StatefulInteractiveElement as _, Styled as _,
};

use super::actions::{OpenRecent, RunCommand, SetLaneTheme, SetLang, SetTheme};
use super::live::Notice;
use super::workspace::Workspace;
use crate::commands;
use crate::prefs::PrefsStore;
use crate::strings;
use crate::theme::{self, CurrentTheme, Skin};

/// Themes offered in the settings menu, in this order (the palette lists every preset).
const THEME_MENU: &[&str] = &[
    "dark",
    "light",
    "dracula",
    "catppuccin-mocha",
    "catppuccin-latte",
    "solarized-dark",
    "solarized-light",
    "nord",
    "gruvbox-dark",
    "tokyo-night",
    "github-light",
    "one-light",
];

/// How many graph colour palettes the menus list.
const LANE_MENU_LEN: usize = 12;

const VERSION: &str = env!("CARGO_PKG_VERSION");

impl Workspace {
    pub(super) fn title_bar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let skin = cx.global::<Skin>();
        let (text, muted) = (skin.text, skin.muted);
        let cur_theme = cx
            .try_global::<CurrentTheme>()
            .map(|c| c.0.clone())
            .unwrap_or_default();
        let cur_lang = gitpoulpi_i18n::current();
        let path = self.backend.repo_root().to_string_lossy().into_owned();
        let recent = PrefsStore::get(cx).recent.clone();
        TitleBar::new()
            .child(
                h_flex()
                    .items_center()
                    .gap(px(8.0))
                    .child(img(self.logo.clone()).w(px(18.0)).h(px(18.0)))
                    .child(
                        div()
                            .font_weight(gpui_kit::FontWeight::SEMIBOLD)
                            .text_color(text)
                            .child("GitPoulpi"),
                    )
                    .child(div().text_color(muted).child(format!("v{VERSION}")))
                    .child(div().text_color(muted).child("-"))
                    .child(
                        div()
                            .text_color(text)
                            .max_w(px(560.0))
                            .truncate()
                            .child(path),
                    )
                    .child(
                        Button::new("open-in")
                            .ghost()
                            .small()
                            .label(msg::open_in_bar::BUTTON.get())
                            .dropdown_menu(move |menu, _, _| {
                                let mut menu = menu.menu(
                                    msg::radar::ACT_OPEN_REPO.get(),
                                    Box::new(RunCommand("palette.browse_repository".into())),
                                );
                                if !recent.is_empty() {
                                    menu = menu.separator();
                                }
                                for r in &recent {
                                    menu = menu
                                        .menu(r.clone(), Box::new(OpenRecent(r.clone().into())));
                                }
                                menu
                            }),
                    ),
            )
            .child(
                h_flex()
                    .items_center()
                    .justify_end()
                    .gap(px(4.0))
                    .px(px(6.0))
                    .on_mouse_down(gpui_kit::MouseButton::Left, |_, _, cx| {
                        cx.stop_propagation()
                    })
                    .child(
                        Button::new("global-search")
                            .ghost()
                            .small()
                            .icon(AssetIcon::Search)
                            .label(msg::tab_bar::SEARCH.get())
                            .on_click(|_, window, cx| {
                                window.dispatch_action(
                                    Box::new(RunCommand("global.palette".into())),
                                    cx,
                                )
                            }),
                    )
                    .child(
                        Button::new("gear")
                            .ghost()
                            .small()
                            .icon(AssetIcon::Settings)
                            .tooltip(msg::toolbar::SETTINGS_TIP.get())
                            .dropdown_menu_with_anchor(
                                Anchor::TopRight,
                                move |menu, window, cx| {
                                    let presets = theme::all_presets();
                                    let cur = cur_theme.clone();
                                    menu.submenu(
                                        msg::settings_ui::THEME.get(),
                                        window,
                                        cx,
                                        move |m, _, _| {
                                            let mut m = m;
                                            for slug in THEME_MENU {
                                                if let Some(p) =
                                                    presets.iter().find(|p| p.slug == *slug)
                                                {
                                                    m = m.menu_with_check(
                                                        p.name.clone(),
                                                        p.slug == cur,
                                                        Box::new(SetTheme(p.slug.clone().into())),
                                                    );
                                                }
                                            }
                                            m
                                        },
                                    )
                                    .submenu(
                                        msg::settings::LANGUAGE.get(),
                                        window,
                                        cx,
                                        move |m, _, _| {
                                            let mut m = m;
                                            for l in gitpoulpi_i18n::SHIPPED.iter().copied() {
                                                m = m.menu_with_check(
                                                    l.native_name(),
                                                    l == cur_lang,
                                                    Box::new(SetLang(l.native_name().into())),
                                                );
                                            }
                                            m
                                        },
                                    )
                                    .submenu(
                                        msg::footer_hint::GRAPH_COLORS.get(),
                                        window,
                                        cx,
                                        |m, _, _| {
                                            let mut m = m.menu(
                                                msg::settings_ui::THEME.get(),
                                                Box::new(SetLaneTheme("theme".into())),
                                            );
                                            for p in gitpoulpi_config::theme::lane_palettes()
                                                .into_iter()
                                                .take(LANE_MENU_LEN)
                                            {
                                                m = m.menu(
                                                    p.label.clone(),
                                                    Box::new(SetLaneTheme(p.slug.clone().into())),
                                                );
                                            }
                                            m
                                        },
                                    )
                                    .separator()
                                    .menu(
                                        msg::log_draw::COLUMN_MENU_FIT_ALL.get(),
                                        Box::new(RunCommand(
                                            "palette.log_column_fit_all_to_content".into(),
                                        )),
                                    )
                                    .menu(
                                        msg::log_draw::COLUMN_MENU_RESET.get(),
                                        Box::new(RunCommand(
                                            "palette.log_column_reset_width".into(),
                                        )),
                                    )
                                },
                            ),
                    ),
            )
    }

    pub(super) fn tab_row(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let skin = cx.global::<Skin>();
        let (text, muted, accent, selection, border) = (
            skin.text,
            skin.muted,
            skin.accent,
            skin.selection,
            skin.border,
        );
        let name = self
            .backend
            .repo_root()
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let tab = |label: String, active: bool, close: bool| {
            h_flex()
                .h(px(24.0))
                .px(px(10.0))
                .gap(px(8.0))
                .items_center()
                .rounded(px(3.0))
                .text_sm()
                .when(active, |d| d.bg(selection).text_color(text))
                .when(!active, |d| d.text_color(muted))
                .child(label)
                .when(close, |d| {
                    d.child(div().text_xs().text_color(accent).child("\u{00D7}"))
                })
        };
        h_flex()
            .flex_none()
            .h(px(32.0))
            .px(px(8.0))
            .gap(px(4.0))
            .items_center()
            .border_b_1()
            .border_color(border)
            .child(
                h_flex().h(px(24.0)).px(px(8.0)).items_center().child(
                    Icon::new(AssetIcon::House)
                        .size(px(14.0))
                        .text_color(accent),
                ),
            )
            .child(tab(msg::radar::TITLE.get().to_string(), false, false))
            .child(tab(name, true, true))
            .child(
                h_flex()
                    .h(px(24.0))
                    .px(px(8.0))
                    .items_center()
                    .text_color(accent)
                    .child("+"),
            )
    }

    pub(super) fn toolbar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let skin = cx.global::<Skin>();
        let (button, button_hover, on_button, muted, border) = (
            skin.tb_bg,
            skin.tb_bg_hover,
            skin.tb_fg,
            skin.tb_disabled_fg,
            skin.border,
        );
        let disabled_bg = skin.tb_disabled_bg;
        let icons = skin.toolbar;
        let can_undo = self
            .backend
            .can_undo_redo(gitpoulpi_core::domain::Direction::Undo);
        let can_redo = self
            .backend
            .can_undo_redo(gitpoulpi_core::domain::Direction::Redo);
        // (id, icon, colour index, label, drop-down chevron, disabled)
        #[allow(clippy::type_complexity)]
        let defs: [(&'static str, AssetIcon, usize, String, bool, bool); 11] = [
            (
                "tb-undo",
                AssetIcon::Undo2,
                0,
                msg::toolbar::UNDO.get().to_string(),
                true,
                !can_undo,
            ),
            (
                "tb-redo",
                AssetIcon::Redo2,
                0,
                msg::toolbar::REDO.get().to_string(),
                false,
                !can_redo,
            ),
            (
                "tb-fetch",
                AssetIcon::RefreshCw,
                1,
                msg::menu_ref::FETCH.get().to_string(),
                false,
                false,
            ),
            (
                "tb-pull",
                AssetIcon::ArrowDownToLine,
                1,
                msg::toolbar::PULL.get().to_string(),
                true,
                false,
            ),
            (
                "tb-push",
                AssetIcon::ArrowUpFromLine,
                2,
                msg::toolbar::PUSH.get().to_string(),
                true,
                false,
            ),
            (
                "tb-branch",
                AssetIcon::GitBranch,
                3,
                msg::toolbar::BRANCH.get().to_string(),
                false,
                false,
            ),
            (
                "tb-tag",
                AssetIcon::Tag,
                4,
                msg::toolbar::TAG.get().to_string(),
                false,
                false,
            ),
            (
                "tb-stash",
                AssetIcon::Archive,
                5,
                msg::toolbar::STASH.get().to_string(),
                false,
                false,
            ),
            (
                "tb-pop",
                AssetIcon::ArchiveRestore,
                5,
                msg::toolbar::POP.get().to_string(),
                false,
                false,
            ),
            (
                "tb-lfs",
                AssetIcon::HardDrive,
                6,
                msg::toolbar::LFS.get().to_string(),
                true,
                false,
            ),
            (
                "tb-search",
                AssetIcon::Search,
                7,
                msg::toolbar::SEARCH.get().to_string(),
                false,
                false,
            ),
        ];
        let mut row = h_flex()
            .flex_none()
            .h(px(34.0))
            .px(px(8.0))
            .gap(px(6.0))
            .items_center()
            .border_t_1()
            .border_color(border);
        for (id, icon, ci, label, chevron, disabled) in defs {
            let mut b = h_flex()
                .id(id)
                .h(px(24.0))
                .px(px(8.0))
                .gap(px(6.0))
                .items_center()
                .rounded(px(3.0))
                .bg(if disabled { disabled_bg } else { button })
                .text_sm()
                .text_color(if disabled { muted } else { on_button })
                .child(Icon::new(icon).size(px(14.0)).text_color(if disabled {
                    muted
                } else {
                    icons[ci]
                }))
                .child(label)
                .when(chevron, |d| {
                    d.child(
                        Icon::new(AssetIcon::ChevronDown)
                            .size(px(11.0))
                            .text_color(muted),
                    )
                });
            if !disabled {
                b = b.hover(move |d| d.bg(button_hover));
                // A button is a command like any other: same dispatcher, same "not available yet".
                b = b.on_click(move |_, window, cx| {
                    commands::run_id(toolbar_command(id), window, cx);
                });
            }
            row = row.child(b);
        }
        row
    }

    /// What the status line says about writes: a running action or job first, else the last result.
    /// `true` marks a failure.
    fn activity_text(&self, _cx: &Context<Self>) -> Option<(String, bool)> {
        let running = self.backend.with_link(|l| {
            l.activity_handle()
                .in_flight_text(std::time::Duration::ZERO)
        });
        if let Some(text) = running {
            return Some((text, false));
        }
        let jobs = self.backend.host().live_jobs();
        if !jobs.is_empty() {
            return Some((strings::jobs_running(jobs.len()), false));
        }
        self.notice
            .as_ref()
            .map(|Notice { text, error }| (text.clone(), *error))
    }

    pub(super) fn status_line(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let skin = cx.global::<Skin>();
        let (muted, text, warning, success, border) = (
            skin.muted,
            skin.text,
            skin.warning,
            skin.success,
            skin.border,
        );
        let summary = self.summary.as_ref();
        let dirty = summary.map(|s| !s.status.is_empty()).unwrap_or(false);
        let commits = match summary {
            Some(s) => crate::panels::log::commit_count_text(s.loaded, s.total),
            None => String::new(),
        };
        let branch = match summary {
            Some(s) => s
                .branch
                .clone()
                .unwrap_or_else(|| msg::sidebar::DETACHED.get().to_string()),
            None => String::new(),
        };
        let text_color = text;
        let hint = |t: String| div().child(t);
        h_flex()
            .flex_none()
            .h(px(24.0))
            .px(px(10.0))
            .gap(px(8.0))
            .items_center()
            .border_t_1()
            .border_color(border)
            .text_xs()
            .text_color(muted)
            .child(
                Icon::new(AssetIcon::GitBranch)
                    .size(px(12.0))
                    .text_color(text),
            )
            .child(div().text_color(text).child(branch))
            .child("\u{2022}")
            .child(div().child(commits))
            .child("\u{2022}")
            .child(
                div()
                    .text_color(if dirty { warning } else { success })
                    .child(if dirty {
                        msg::footer::DIRTY.get().trim().to_string()
                    } else {
                        msg::footer::CLEAN.get().trim().to_string()
                    }),
            )
            .child("\u{2022}")
            .child(div().child(msg::footer::session(self.backend.session())))
            .when_some(crate::commands::status::current(cx), |d, message| {
                d.child("\u{2022}")
                    .child(div().text_color(text).child(message))
            })
            .child("\u{2022}")
            .children(self.activity_text(cx).map(|(text, error)| {
                div()
                    .text_color(if error { warning } else { text_color })
                    .child(text)
            }))
            .child(hint(msg::footer::HINT_HELP.get().to_string()))
            .child(hint(msg::footer::hint_git("Ctrl+G")))
            .child(hint(msg::footer::hint_nav("Ctrl+N")))
            .child(hint(msg::footer::hint_commands("Ctrl+P")))
            .child(hint(msg::footer::hint_menu("Shift+F10/.")))
            .child(hint(msg::footer::HINT_QUIT.get().to_string()))
            .child(div().flex_1())
            .child(
                div()
                    .text_color(text)
                    .child(msg::activity_log::BUTTON.get()),
            )
    }
}

/// The command (an id of the shared registry) a toolbar button stands for.
fn toolbar_command(id: &str) -> &'static str {
    match id {
        "tb-undo" => "global.undo",
        "tb-redo" => "global.redo",
        "tb-fetch" => "git_prefix.fetch_all",
        "tb-pull" => "git_prefix.pull",
        "tb-push" => "git_prefix.push",
        "tb-branch" => "git_prefix.new_branch",
        "tb-tag" => "git_prefix.new_tag",
        "tb-stash" => "git_prefix.stash_push",
        "tb-pop" => "git_prefix.stash_pop",
        "tb-lfs" => "git_prefix.lfs_menu",
        _ => "global.find_in_log",
    }
}
