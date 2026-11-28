//! The Log's find bar: matches by summary, author or SHA prefix; the other rows dim.

use std::collections::HashSet;
use std::sync::Arc;

use gitpoulpi_core::domain::CommitNode;
use gitpoulpi_i18n::msg;
use gpui_kit::assets::IconName as AssetIcon;
use gpui_kit::component::{
    h_flex,
    input::{Input, InputEvent, InputState},
    Icon, Sizable as _,
};
use gpui_kit::{
    div, px, AppContext as _, Context, Entity, Focusable as _, InteractiveElement as _,
    IntoElement, MouseButton, ParentElement as _, SharedString, Styled as _, Subscription, Window,
};

use super::LogPanel;
use crate::graph::Entry;
use crate::theme::Skin;

pub(super) struct SearchState {
    pub input: Entity<InputState>,
    pub open: bool,
    pub query: String,
    /// Display indices of the matching rows.
    pub matches: Vec<usize>,
    pub match_ix: usize,
}

impl SearchState {
    pub fn new(window: &mut Window, cx: &mut Context<LogPanel>) -> (Self, Vec<Subscription>) {
        let input = cx.new(|cx| {
            InputState::new(window, cx).placeholder(msg::search::EMPTY_NOTHING_YET.get())
        });
        let subs = vec![cx.subscribe_in(
            &input,
            window,
            |this: &mut LogPanel, input, ev: &InputEvent, _window, cx| match ev {
                InputEvent::Change => {
                    this.search.query = input.read(cx).value().to_string();
                    this.recompute_matches(cx);
                }
                InputEvent::PressEnter { shift, .. } => {
                    this.step_match(if *shift { -1 } else { 1 }, cx)
                }
                _ => {}
            },
        )];
        (
            Self {
                input,
                open: false,
                query: String::new(),
                matches: Vec::new(),
                match_ix: 0,
            },
            subs,
        )
    }

    pub fn active(&self) -> bool {
        !self.query.is_empty()
    }

    pub fn match_set(&self) -> Arc<HashSet<usize>> {
        Arc::new(self.matches.iter().copied().collect())
    }
}

fn node_matches(n: &CommitNode, lowercase_query: &str) -> bool {
    n.summary.to_lowercase().contains(lowercase_query)
        || n.author_name.to_lowercase().contains(lowercase_query)
        || n.id.to_hex().starts_with(lowercase_query)
}

/// Display indices of the commit rows matching `query` (empty query: none).
pub(super) fn matching_rows(entries: &[Entry], nodes: &[CommitNode], query: &str) -> Vec<usize> {
    if query.is_empty() {
        return Vec::new();
    }
    let q = query.to_lowercase();
    entries
        .iter()
        .enumerate()
        .filter_map(|(ix, e)| match e {
            Entry::Commit(i) if node_matches(&nodes[*i], &q) => Some(ix),
            _ => None,
        })
        .collect()
}

impl LogPanel {
    pub(super) fn recompute_matches(&mut self, cx: &mut Context<Self>) {
        self.search.matches = matching_rows(
            self.data.layout.entries(),
            &self.data.nodes,
            &self.search.query,
        );
        self.search.match_ix = 0;
        cx.notify();
    }

    fn step_match(&mut self, dir: isize, cx: &mut Context<Self>) {
        if self.search.matches.is_empty() {
            return;
        }
        let n = self.search.matches.len() as isize;
        self.search.match_ix = (self.search.match_ix as isize + dir).rem_euclid(n) as usize;
        let ix = self.search.matches[self.search.match_ix];
        self.select(ix, cx);
    }

    pub fn toggle_search(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.search.open = !self.search.open;
        if self.search.open {
            self.search
                .input
                .read(cx)
                .focus_handle(cx)
                .focus(window, cx);
        } else {
            self.search.query.clear();
            self.search.matches.clear();
            self.focus.focus(window, cx);
        }
        cx.notify();
    }

    pub fn open_search(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.search.open = true;
        self.search
            .input
            .read(cx)
            .focus_handle(cx)
            .focus(window, cx);
        cx.notify();
    }

    pub(super) fn search_bar(&self, skin: &Skin, cx: &mut Context<Self>) -> impl IntoElement {
        let s = &self.search;
        let counter: SharedString = if s.query.is_empty() {
            "".into()
        } else {
            format!(
                "{} / {}",
                if s.matches.is_empty() {
                    0
                } else {
                    s.match_ix + 1
                },
                s.matches.len()
            )
            .into()
        };
        h_flex()
            .flex_none()
            .h(px(34.0))
            .px_3()
            .gap_2()
            .items_center()
            .border_b_1()
            .border_color(skin.border)
            .child(
                Icon::new(AssetIcon::Search)
                    .small()
                    .text_color(skin.toolbar[7]),
            )
            .child(
                div()
                    .w(px(340.0))
                    .child(Input::new(&s.input).small().cleanable(true)),
            )
            .child(div().text_xs().text_color(skin.muted).child(counter))
            .child(div().flex_1())
            .child(
                div()
                    .text_xs()
                    .text_color(skin.muted)
                    .child(msg::footer::search_hint("\u{2193}", "\u{2191}")),
            )
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|_, _, _, cx| cx.stop_propagation()),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitpoulpi_core::domain::Oid;

    fn node(n: u8, summary: &str, author: &str) -> CommitNode {
        CommitNode {
            id: Oid([n; 20]),
            parents: vec![],
            summary: summary.into(),
            message: summary.into(),
            author_name: author.into(),
            author_email: String::new(),
            timestamp: 0,
            refs: vec![],
        }
    }

    #[test]
    fn matches_summary_author_and_sha_prefix_but_not_other_rows() {
        let nodes = vec![
            node(0xab, "Fix the parser", "Alice"),
            node(0xcd, "Add docs", "Bob"),
        ];
        let entries = [Entry::Wip, Entry::Commit(0), Entry::Commit(1)];
        assert_eq!(matching_rows(&entries, &nodes, "PARSER"), vec![1]);
        assert_eq!(matching_rows(&entries, &nodes, "bob"), vec![2]);
        assert_eq!(matching_rows(&entries, &nodes, "abab"), vec![1]);
        assert!(matching_rows(&entries, &nodes, "").is_empty());
        assert!(matching_rows(&entries, &nodes, "zzz").is_empty());
    }
}
