//! The command palette: every action of the window, searchable.

use std::cell::Cell;
use std::rc::Rc;

use gitpoulpi_i18n::msg;
use gpui_kit::assets::IconName as AssetIcon;
use gpui_kit::component::{
    command::{Command, CommandEntry, CommandGroup, CommandItem},
    ActiveTheme as _, WindowExt as _,
};
use gpui_kit::{px, Context, Focusable as _, ParentElement as _, Styled as _, Window};

use super::actions::{
    FitColumns, FocusChanges, FocusLog, FocusRefs, OpenRecent, OpenRepo, ResetColumns,
    SetLaneTheme, SetLang, SetTheme, ToggleSearch,
};
use super::workspace::Workspace;
use crate::prefs::PrefsStore;
use crate::theme;

/// How many graph colour palettes the palette lists.
const LANE_ENTRIES: usize = 12;

impl Workspace {
    fn palette_entries(&self, cx: &Context<Self>) -> Vec<CommandEntry> {
        let nav = CommandGroup::new()
            .item(
                CommandItem::new()
                    .label(msg::palette::NAVIGATION_FOCUS_REFS_PANE.get())
                    .icon(AssetIcon::GitBranch)
                    .action(Box::new(FocusRefs))
                    .keywords(["refs", "branches", "sidebar"]),
            )
            .item(
                CommandItem::new()
                    .label(msg::palette::NAVIGATION_FOCUS_LOG_PANE.get())
                    .icon(AssetIcon::GitGraph)
                    .action(Box::new(FocusLog))
                    .keywords(["log", "commits", "graph"]),
            )
            .item(
                CommandItem::new()
                    .label(msg::palette::NAVIGATION_FOCUS_STATUS_PANE.get())
                    .icon(AssetIcon::FileDiff)
                    .action(Box::new(FocusChanges))
                    .keywords(["files", "diff", "changes"]),
            )
            .item(
                CommandItem::new()
                    .label(msg::toolbar::SEARCH_TIP.get())
                    .icon(AssetIcon::Search)
                    .action(Box::new(ToggleSearch))
                    .keywords(["filter", "find"]),
            )
            .item(
                CommandItem::new()
                    .label(msg::log_draw::COLUMN_MENU_FIT_ALL.get())
                    .icon(AssetIcon::ArrowLeftRight)
                    .action(Box::new(FitColumns)),
            )
            .item(
                CommandItem::new()
                    .label(msg::log_draw::COLUMN_MENU_RESET.get())
                    .icon(AssetIcon::RotateCcw)
                    .action(Box::new(ResetColumns)),
            );
        let mut repo = CommandGroup::new().label(msg::home::TITLE.get()).item(
            CommandItem::new()
                .label(msg::radar::ACT_OPEN_REPO.get())
                .icon(AssetIcon::FolderOpen)
                .action(Box::new(OpenRepo))
                .keywords(["browse", "folder"]),
        );
        for recent in &PrefsStore::get(cx).recent {
            repo = repo.item(
                CommandItem::new()
                    .label(recent.clone())
                    .icon(AssetIcon::FolderGit)
                    .action(Box::new(OpenRecent(recent.clone().into()))),
            );
        }
        let mut themes = CommandGroup::new().label(msg::settings_ui::THEME.get());
        for p in theme::all_presets() {
            themes = themes.item(
                CommandItem::new()
                    .label(format!("{}: {}", msg::settings_ui::THEME.get(), p.name))
                    .icon(if p.light {
                        AssetIcon::Sun
                    } else {
                        AssetIcon::Moon
                    })
                    .action(Box::new(SetTheme(p.slug.clone().into())))
                    .keywords([
                        p.slug.clone(),
                        if p.light {
                            "light".into()
                        } else {
                            "dark".to_string()
                        },
                    ]),
            );
        }
        let lanes_label = msg::footer_hint::GRAPH_COLORS.get();
        let mut lanes = CommandGroup::new().label(lanes_label).item(
            CommandItem::new()
                .label(format!("{lanes_label}: {}", msg::settings_ui::THEME.get()))
                .icon(AssetIcon::Palette)
                .action(Box::new(SetLaneTheme("theme".into()))),
        );
        for p in gitpoulpi_config::theme::lane_palettes()
            .into_iter()
            .take(LANE_ENTRIES)
        {
            lanes = lanes.item(
                CommandItem::new()
                    .label(format!("{lanes_label}: {}", p.label))
                    .icon(AssetIcon::Palette)
                    .action(Box::new(SetLaneTheme(p.slug.clone().into())))
                    .keywords([p.slug]),
            );
        }
        let mut langs = CommandGroup::new().label(msg::settings::LANGUAGE.get());
        for l in gitpoulpi_i18n::SHIPPED.iter().copied() {
            langs = langs.item(
                CommandItem::new()
                    .label(format!(
                        "{}: {}",
                        msg::settings::LANGUAGE.get(),
                        l.native_name()
                    ))
                    .icon(AssetIcon::Languages)
                    .action(Box::new(SetLang(l.native_name().into()))),
            );
        }
        vec![
            nav.into(),
            CommandEntry::Separator,
            repo.into(),
            CommandEntry::Separator,
            langs.into(),
            CommandEntry::Separator,
            lanes.into(),
            CommandEntry::Separator,
            themes.into(),
        ]
    }

    pub fn open_palette(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let palette = self.palette.clone();
        palette.update(cx, |p, cx| p.set_query("", window, cx));
        let entries = self.palette_entries(cx);
        let focus_once = Rc::new(Cell::new(true));
        window.open_dialog(cx, move |dialog, _, _| {
            let palette = palette.clone();
            let entries = entries.clone();
            let focus_once = focus_once.clone();
            dialog
                .close_button(false)
                .overlay_closable(true)
                .p_0()
                .content(move |content, window, cx| {
                    if focus_once.replace(false) {
                        let palette = palette.clone();
                        window.defer(cx, move |window, cx| {
                            palette.read(cx).focus_handle(cx).focus(window, cx)
                        });
                    }
                    let mut cmd = Command::new(&palette)
                        .bordered(false)
                        .rounded(cx.theme().radius_lg)
                        .placeholder(msg::search::EMPTY_NOTHING_YET.get())
                        .max_h(px(460.))
                        .on_confirm(|_, window, cx| window.close_dialog(cx));
                    for e in entries.iter().cloned() {
                        cmd = match e {
                            CommandEntry::Item(i) => cmd.item(i),
                            CommandEntry::Group(g) => cmd.group(g),
                            CommandEntry::Separator => cmd.separator(),
                        };
                    }
                    content.child(cmd)
                })
        });
    }
}
