mod editor;
pub mod menu;

use crate::gui::editor::Editor;
use crate::modal::app_state::AppState;
use gpui::{AppContext, Entity, IntoElement, div, Subscription};
use gpui::{Context, ParentElement, Render, Styled, Window, actions};
use gpui_component::resizable::{ResizableState, h_resizable, resizable_panel};
use gpui_component::{PixelsExt, Root, WindowExt};
use simple_gpui::{component, component_property, subscribe};

#[component]
pub fn main_page(window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    init_with_context!();
    component_property!(editor: Entity<Editor> = cx.new(|cx| {
        let project = cx.global::<AppState>().current_project.clone();
        Editor::new(project, cx, window)
    }));
    component_property!(left_panel_width: gpui::Pixels = window.viewport_size().width * 0.7);


    let dialog_layer = Root::render_dialog_layer(window, cx);
    let notification_layer = Root::render_notification_layer(window, cx);

    div()
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
                .child(
                    resizable_panel()
                        .size(self.left_panel_width)
                        .child("111223"),
                )
                .child(resizable_panel().child(self.editor.clone())),
        )
        .children(dialog_layer)
        .children(notification_layer)
}

actions!(set_menus, [Quit, NewFile, OpenFile]);
