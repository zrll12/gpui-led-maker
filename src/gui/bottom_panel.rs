mod fonts;
mod recent_projects;

use crate::gui::Render;
use gpui::{AnyElement, AppContext, Context, Entity, IntoElement, ParentElement, Styled, Window, div};
use gpui_component::tab::{Tab, TabBar};
use simple_gpui::component;

#[component]
pub fn bottom_panel(_window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    init_with_context!();
    component_property!(selected_tab: usize = 0);
    component_entity!(fonts: fonts::Fonts = fonts::Fonts::new());
    component_entity!(recent_projects: recent_projects::RecentProjects = recent_projects::RecentProjects::new());

    let content: AnyElement = match self.selected_tab {
        0 => self.fonts.clone().into_any_element(),
        1 => self.recent_projects.clone().into_any_element(),
        _ => div().size_full().p_3().child("Settings coming soon").into_any_element(),
    };

    div()
        .size_full()
        .child(
            TabBar::new("bottom panel")
                .selected_index(self.selected_tab)
                .on_click(cx.listener(|view, index, _, cx| {
                    view.selected_tab = *index;
                    cx.notify();
                }))
                .child(Tab::new().label("Fonts"))
                .child(Tab::new().label("Recent Projects"))
                .child(Tab::new().label("Settings")),
        )
        .child(content)
}