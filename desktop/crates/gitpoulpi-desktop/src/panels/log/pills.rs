//! Ref pills in the Log's Ref column (the terminal UI's `ref_pills_for_commit` and its drawing).

use gitpoulpi_core::domain::{Ref, RefKind};
use gpui_kit::assets::IconName as AssetIcon;
use gpui_kit::component::{h_flex, Icon};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{div, px, FontWeight, Hsla, IntoElement, ParentElement as _, Styled as _};

use crate::theme::Skin;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PillKind {
    Local,
    Remote,
    Tag,
}

/// A ref pill as `log_pills.rs::RefPill`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pill {
    pub name: String,
    pub kind: PillKind,
    pub checked_out: bool,
    /// A local branch whose remote twin points at the same commit shares its pill.
    pub combined_remote: bool,
    /// A tag that also exists on the remote.
    pub tag_remote: bool,
}

/// Port of `ref_pills_for_commit`: the pills of the refs at `idxs`, most important first.
pub fn pills_for(refs: &[Ref], idxs: &[usize]) -> Vec<Pill> {
    let mut locals: Vec<(String, bool)> = Vec::new();
    let mut remotes: Vec<(String, String)> = Vec::new();
    let mut tags: Vec<(String, bool)> = Vec::new();
    for &i in idxs {
        let r = &refs[i];
        match &r.kind {
            RefKind::LocalBranch { name } => locals.push((name.clone(), r.is_head)),
            RefKind::RemoteBranch { remote, name } => remotes.push((remote.clone(), name.clone())),
            RefKind::Tag { name } => tags.push((name.clone(), r.tag_where().is_some())),
        }
    }
    let mut out = Vec::new();
    for (name, head) in locals {
        let combined = match remotes.iter().position(|(_, n)| *n == name) {
            Some(p) => {
                remotes.remove(p);
                true
            }
            None => false,
        };
        out.push(Pill {
            name,
            kind: PillKind::Local,
            checked_out: head,
            combined_remote: combined,
            tag_remote: false,
        });
    }
    for (_, name) in remotes {
        out.push(Pill {
            name,
            kind: PillKind::Remote,
            checked_out: false,
            combined_remote: false,
            tag_remote: false,
        });
    }
    for (name, on_remote) in tags {
        out.push(Pill {
            name,
            kind: PillKind::Tag,
            checked_out: false,
            combined_remote: false,
            tag_remote: on_remote,
        });
    }
    out.sort_by_key(|p| {
        let rank = match (p.kind, p.checked_out) {
            (PillKind::Local, true) => 0u8,
            (PillKind::Local, false) => 1,
            (PillKind::Remote, _) => 2,
            (PillKind::Tag, _) => 3,
        };
        (rank, p.name.clone())
    });
    out
}

/// Pixels a pill's icons take, for auto-fitting the Ref column.
pub fn icons_width(p: &Pill) -> f32 {
    if p.combined_remote {
        42.0
    } else {
        24.0
    }
}

pub fn pill_element(p: &Pill, overflow: usize, lane: Hsla, skin: &Skin) -> impl IntoElement {
    let bg = skin.pill_bg(lane);
    let fg = skin.contrasting_fg(bg);
    let icon = |i: AssetIcon| Icon::new(i).size(px(12.0)).text_color(fg);
    let mut main = h_flex()
        .min_w_0()
        .h(px(18.0))
        .px(px(6.0))
        .gap(px(5.0))
        .items_center()
        .rounded(px(3.0))
        .bg(bg)
        .text_color(fg)
        .text_xs();
    if p.checked_out {
        main = main.child(
            div()
                .flex_none()
                .font_weight(FontWeight::BOLD)
                .child("\u{2713}"),
        );
    }
    main = main.child(
        div()
            .min_w_0()
            .truncate()
            .when(p.checked_out, |d| d.font_weight(FontWeight::BOLD))
            .child(p.name.clone()),
    );
    main = match p.kind {
        PillKind::Local => main
            .child(icon(AssetIcon::Laptop))
            .when(p.combined_remote, |d| d.child(icon(AssetIcon::Globe))),
        PillKind::Remote => main.child(icon(AssetIcon::Globe)),
        PillKind::Tag => main
            .child(icon(AssetIcon::Tag))
            .when(p.tag_remote, |d| d.child(icon(AssetIcon::ArrowUp))),
    };
    h_flex()
        .min_w_0()
        .gap(px(4.0))
        .items_center()
        .child(main)
        .when(overflow > 0, |d| {
            d.child(
                div()
                    .flex_none()
                    .h(px(18.0))
                    .px(px(6.0))
                    .rounded(px(3.0))
                    .bg(bg)
                    .text_color(fg)
                    .text_xs()
                    .flex()
                    .items_center()
                    .child(format!("+{overflow}")),
            )
        })
}
