//! Drawing the Refs sidebar's rows and section boxes.

use super::*;

impl RefsPanel {
    pub(super) fn row_element(
        me: &WeakEntity<RefsPanel>,
        section: &Arc<Section>,
        ix: usize,
        salt: usize,
        view: &RowView,
        pal: &Palette,
    ) -> AnyElement {
        let row = &section.rows[ix];
        let kind = section.kind;
        let (hidden, soloed) = (
            model::row_hidden(section, row, &view.hidden),
            model::row_soloed(section, row, &view.solo),
        );
        let selected = view.cursor.as_deref() == Some(row.key.as_str());
        let color = pal.section[Kind::ALL.iter().position(|k| *k == kind).unwrap_or(0)];
        let (folder, is_head, tag_where, expanded, remote_root) = match &row.kind {
            RowKind::Folder {
                expanded,
                is_remote_root,
                ..
            } => (true, false, None, *expanded, *is_remote_root),
            RowKind::Ref {
                is_head, tag_where, ..
            } => (false, *is_head, *tag_where, false, false),
            _ => (false, false, None, false, false),
        };
        let refish = kind.ref_kind().is_some();
        // Where a tag lives, in words: the shared tooltip.
        let tip: Option<String> = match &row.kind {
            RowKind::Ref {
                full_name,
                tag_where: Some(_),
                ..
            } => view
                .tags
                .get(full_name)
                .map(|info| tooltip_lines(full_name, info).join("\n")),
            _ => None,
        };
        let (text, muted, accent) = (pal.text, pal.muted, pal.accent);
        let hover_bg = pal.hover;
        let sel_bg = pal.selection;
        let label_color = if hidden {
            muted
        } else if folder {
            pal.text
        } else {
            color
        };
        let me_row = me.clone();
        let (sec_idx, row_idx) = (section.clone(), ix);
        let target = ops::target_of(section, row);
        let draggable = matches!(
            target,
            Some(crate::ref_target::RefTarget::Local(_) | crate::ref_target::RefTarget::Remote(_))
        );
        let bounds = view.bounds.clone();
        let btn = |id: &'static str, icon: AssetIcon, active: bool, tint: Hsla| {
            div()
                .id((id, salt + ix))
                .flex_none()
                .p(px(3.0))
                .rounded(px(3.0))
                .cursor_pointer()
                .opacity(if active { 1.0 } else { 0.0 })
                .group_hover("ref-row", |s| s.opacity(1.0))
                .hover(move |d| d.bg(hover_bg))
                .child(
                    Icon::new(icon)
                        .size(px(12.0))
                        .text_color(if active { tint } else { muted }),
                )
        };
        let hide_btn = {
            let (me, section, row) = (me.clone(), section.clone(), row.clone());
            btn(
                "ref-hide",
                if hidden {
                    AssetIcon::EyeOff
                } else {
                    AssetIcon::Eye
                },
                hidden,
                muted,
            )
            .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                cx.stop_propagation();
                let _ = me.update(cx, |this, cx| this.hide_row(&section, &row, cx));
            })
        };
        let solo_btn = {
            let (me, section, row) = (me.clone(), section.clone(), row.clone());
            btn("ref-solo", AssetIcon::Star, soloed, accent).on_mouse_down(
                MouseButton::Left,
                move |_, _, cx| {
                    cx.stop_propagation();
                    let _ = me.update(cx, |this, cx| this.solo_row(&section, &row, cx));
                },
            )
        };
        h_flex()
            .id(("ref-row", salt + ix))
            .group("ref-row")
            .w_full()
            .h(px(ROW_H))
            .flex_none()
            .items_center()
            .gap(px(5.0))
            .pl(px(8.0 + row.depth as f32 * 14.0))
            .pr(px(4.0))
            .cursor_pointer()
            .when(selected, |d| d.bg(sel_bg))
            .hover(move |d| d.bg(hover_bg))
            .when(selected, |d| {
                d.child(
                    canvas(
                        move |b, _, _| {
                            *bounds.borrow_mut() = Some(b);
                        },
                        |_, _, _, _| {},
                    )
                    .absolute()
                    .size_full(),
                )
            })
            .child(
                div()
                    .w(px(12.0))
                    .flex_none()
                    .text_xs()
                    .text_color(text)
                    .child(if folder {
                        ""
                    } else if is_head {
                        "\u{2713}"
                    } else {
                        ""
                    }),
            )
            .when(folder, |d| {
                d.child(
                    Icon::new(if expanded {
                        AssetIcon::ChevronDown
                    } else {
                        AssetIcon::ChevronRight
                    })
                    .size(px(11.0))
                    .text_color(muted),
                )
                .child(
                    Icon::new(if remote_root {
                        AssetIcon::Globe
                    } else if expanded {
                        AssetIcon::FolderOpen
                    } else {
                        AssetIcon::Folder
                    })
                    .size(px(14.0))
                    .text_color(if remote_root { color } else { pal.folder }),
                )
            })
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .truncate()
                    .text_sm()
                    .text_color(label_color)
                    .when(is_head, |d| d.font_weight(FontWeight::BOLD))
                    .when(hidden, |d| d.opacity(0.6))
                    .child(SharedString::from(row.label.clone())),
            )
            .when_some(tag_where, |d, w| {
                d.child(
                    div()
                        .flex_none()
                        .text_xs()
                        .text_color(match w {
                            TagWhere::Local | TagWhere::Differs => pal.warning,
                            TagWhere::Remote => pal.section[1],
                            TagWhere::Both => muted,
                        })
                        .child(model::tag_glyph(w)),
                )
            })
            .when(refish, |d| d.child(hide_btn).child(solo_btn))
            .when_some(tip, |d, tip| {
                d.tooltip(move |window, cx| {
                    gpui_kit::component::tooltip::Tooltip::new(tip.clone()).build(window, cx)
                })
            })
            .on_mouse_down(MouseButton::Right, {
                let (me, section) = (me.clone(), section.clone());
                move |ev, window, cx| {
                    cx.stop_propagation();
                    let at = ev.position;
                    let _ = me.update(cx, |this, cx| {
                        window.focus(&this.focus, cx);
                        this.row_menu(section.kind, ix, at, cx)
                    });
                }
            })
            .when_some(target.clone().filter(|_| draggable), |d, t| {
                let name = t.name();
                d.on_drag(DraggedRef(t), move |_, _, _, cx| {
                    cx.new(|_| DragPreview(name.clone()))
                })
            })
            .on_drop({
                let (me, section) = (me.clone(), section.clone());
                move |dragged: &DraggedRef, window, cx| {
                    let (source, at) = (dragged.0.clone(), window.mouse_position());
                    let _ = me.update(cx, |this, cx| {
                        this.row_dropped(source, section.kind, ix, at, cx)
                    });
                }
            })
            .on_click(move |ev, window, cx| {
                let section = sec_idx.clone();
                let double = ev.click_count() >= 2;
                let _ = me_row.update(cx, |this, cx| {
                    window.focus(&this.focus, cx);
                    this.select(section.kind, row_idx, cx);
                    if double && !matches!(section.rows[row_idx].kind, RowKind::Folder { .. }) {
                        this.row_double_click(section.kind, row_idx, cx);
                        return;
                    }
                    let row = &section.rows[row_idx];
                    match row.kind {
                        RowKind::Folder { .. } => {
                            let row = row.clone();
                            this.toggle_folder(&section, &row, None, cx);
                        }
                        _ => {
                            if let Some(oid) = row.target() {
                                cx.emit(RefsEvent::Selected(oid));
                            }
                        }
                    }
                });
            })
            .into_any_element()
    }

    pub(super) fn section_box(
        &mut self,
        si: usize,
        pal: &Palette,
        border: Hsla,
        cx: &mut Context<Self>,
    ) -> gpui_kit::Div {
        let section = self.sections[si].clone();
        let kind = section.kind;
        let ki = Kind::ALL.iter().position(|k| *k == kind).unwrap_or(0);
        let color = pal.section[ki];
        let open = !self.closed.contains(&kind);
        let rows = section.rows.len();
        let tall = rows > 9;
        let body_h = (rows.max(1) as f32 * ROW_H + 8.0).min(240.0);
        let soloing = !self.solo.is_empty() && kind.ref_kind().is_some();
        let count_text = if soloing {
            let soloed = section
                .names
                .iter()
                .filter(|n| self.solo.contains(*n))
                .count();
            format!("{} ({soloed}/{})", kind.title(), section.count)
        } else {
            format!("{} ({})", kind.title(), section.count)
        };
        let scroll = self.scrolls.entry(kind).or_default().clone();
        let view = RowView {
            tags: self.tags.clone(),
            bounds: self.row_bounds.clone(),
            hidden: self.hidden.clone(),
            solo: self.solo.clone(),
            cursor: self
                .cursor
                .as_ref()
                .filter(|(k, _)| *k == kind)
                .map(|(_, key)| key.clone()),
        };
        let me = cx.entity().downgrade();
        let salt = (ki + 1) * 1_000_000;
        let pal_rows = pal.clone();
        let rows_section = section.clone();
        let list = uniform_list(
            SharedString::from(format!("refs-{ki}")),
            rows,
            move |range: Range<usize>, _window, _cx| {
                range
                    .map(|ix| Self::row_element(&me, &rows_section, ix, salt, &view, &pal_rows))
                    .collect::<Vec<_>>()
            },
        )
        .size_full()
        .track_scroll(&scroll);
        let header = h_flex()
            .id(("sec-head", si))
            .flex_none()
            .h(px(28.0))
            .px(px(8.0))
            .gap(px(6.0))
            .items_center()
            .cursor_pointer()
            .on_click(cx.listener(move |this, _, _, cx| this.toggle_section(kind, cx)))
            .child(
                Icon::new(if open {
                    AssetIcon::ChevronDown
                } else {
                    AssetIcon::ChevronRight
                })
                .size(px(12.0))
                .text_color(pal.muted),
            )
            .child(
                Icon::new(section_icon(kind))
                    .size(px(15.0))
                    .text_color(color),
            )
            .child(
                div()
                    .min_w_0()
                    .truncate()
                    .text_xs()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(pal.text)
                    .child(count_text),
            )
            .child(div().flex_1())
            .when(kind == Kind::Remote, |d| {
                d.child(
                    Icon::new(AssetIcon::Plus)
                        .size(px(13.0))
                        .text_color(pal.muted),
                )
            });
        let mut boxed = v_flex()
            .w_full()
            .border_1()
            .border_color(border)
            .rounded(px(3.0))
            .overflow_hidden()
            .child(header);
        if open && rows > 0 {
            let body = div()
                .relative()
                .child(list)
                .child(Scrollbar::vertical(&scroll));
            boxed = if tall {
                boxed
                    .flex_1()
                    .min_h(px(120.0))
                    .child(body.flex_1().min_h_0())
            } else {
                boxed.flex_none().child(body.h(px(body_h)))
            };
        }
        boxed
    }
}

/// What a row needs besides the section: the sets and the selection.
pub(super) struct RowView {
    pub(super) tags: Arc<HashMap<String, TagInfo>>,
    pub(super) bounds: RowBounds,
    pub(super) hidden: HashSet<String>,
    pub(super) solo: HashSet<String>,
    pub(super) cursor: Option<String>,
}
