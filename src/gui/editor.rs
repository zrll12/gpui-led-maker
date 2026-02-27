use crate::gui::Render;
use gpui::{div, AppContext, Context, Entity, IntoElement, ParentElement, RenderOnce, Window};
use gpui_component::input::InputState;
use gpui_component::v_flex;
use simple_gpui::{component, component_property};
use crate::modal::project::LedMakerProject;

#[component]
pub fn editor(window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    component_property!(project: LedMakerProject);

    v_flex()
        .child(format!("Project Name: '{}'", self.project.name))
        .child("111")
}


pub struct EditorWrapper {
    pub editor: Entity<Editor>,
}
