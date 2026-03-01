use crate::gui::Render;
use crate::modal::app_state::AppState;
use crate::modal::project::LedMakerProject;
use gpui::Subscription;
use gpui::{AppContext, Context, Entity, IntoElement, ParentElement, Styled, Window, div};
use gpui_component::button::Button;
use gpui_component::input::{Input, InputEvent, InputState};
use gpui_component::{h_flex, v_flex};
use simple_gpui::component;

#[component]
pub fn editor(window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    init_with_context!();
    component_property!(project_original: LedMakerProject);
    component_property!(project: LedMakerProject = project_original.clone());
    component_entity!(project_name_input: InputState = {
        let mut input = InputState::new(window, cx).placeholder("Unnamed Project");
        input.set_value(project_original.name.clone(), window, cx);
        input
    });
    observe!(AppState, |page, window, cx| {
        let app_state = cx.global::<AppState>();
        let path = app_state.file_path.clone();
        let project = app_state.current_project.clone();
        println!(
            "AppState changed: file_path={:?}, project_name={}",
            path, project.name
        );
        page.project = project;
        let project_name = page.project.name.clone();
        page.project_name_input
            .update(cx, |input, cx| input.set_value(project_name, window, cx));

        cx.notify();
    });
    subscribe!(
        project_name_input,
        |view, _state, event, _window, cx| match event {
            InputEvent::Change => {
                let value = project_name_input.read(cx).value();
                view.project.name = value.to_string();
                cx.notify();
            }
            _ => {}
        }
    );

    v_flex()
        .p_10()
        .gap_2()
        .size_full()
        .child(
            h_flex()
                .gap_2()
                .items_center()
                .child("Project name:")
                .child(Input::new(&self.project_name_input).w_full().flex_grow()),
        )
        .child(div().flex_grow())
        .child(
            h_flex().w_full().justify_center().child(
                Button::new("editor-save")
                    .label("Save")
                    .on_click(cx.listener(|view, _, _, cx| {
                        view.project.save_project(cx);
                    }))
                    .px_6()
                    .py_2(),
            ),
        )
}
