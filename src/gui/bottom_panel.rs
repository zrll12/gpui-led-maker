use crate::gui::Render;
use gpui::{div, Context, IntoElement, ParentElement, Styled, Window};
use gpui_component::tab::{Tab, TabBar};
use simple_gpui::{component, component_property, init_with_context};

#[component]
pub fn bottom_panel(window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    init_with_context!();
    component_property!(selected_tab: usize = 0);

    div()
        .size_full()
        .child(TabBar::new("bottom panel")
            .selected_index(self.selected_tab)
            .on_click(cx.listener(|view, index, _, cx| {
                view.selected_tab = *index;
                cx.notify();
            }))
            .child(Tab::new().label("Fonts"))
            .child(Tab::new().label("Recent Projects"))
            .child(Tab::new().label("Settings")))
}