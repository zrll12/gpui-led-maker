mod editor;
pub mod menu;

use crate::gui::editor::Editor;
use crate::modal::app_state::AppState;
use gpui::{size, AppContext, Entity, IntoElement};
use gpui::{Context, ParentElement, Render, Styled, Window, actions};
use gpui_component::resizable::{ResizableState, h_resizable, resizable_panel};
use gpui_component::{PixelsExt, Root, WindowExt, h_flex};
use image::Pixels;
use simple_gpui::component;
use std::sync::Arc;

#[component]
pub fn main_page(window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    init_with_context!();
    component_property!(editor: Entity<Editor> = cx.new(|cx| Editor::new(AppState::new().current_project)));
    component_property!(left_panel_width: gpui::Pixels = window.viewport_size().width * 0.7);

    let dialog_layer = Root::render_dialog_layer(window, cx);
    let notification_layer = Root::render_notification_layer(window, cx);

    h_flex()
        .p_5()
        .gap_2()
        .size_full()
        .items_center()
        .justify_center()
        .child(
            h_resizable("base layout")
                .on_resize(cx.listener(|view, state: &Entity<ResizableState>, _, cx| {
                    let state = state.read(cx);
                    let sizes = state.sizes();
                    view.left_panel_width = sizes[0];

                    cx.notify();
                }))
                .child(resizable_panel().size(self.left_panel_width).child(self.editor.clone()))
                .child(resizable_panel().child("111223")),
        )
        .children(dialog_layer)
        .children(notification_layer)
}

actions!(set_menus, [Quit, NewFile, OpenFile]);
