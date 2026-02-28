use crate::gui::Render;
use crate::modal::project::LedMakerProject;
use gpui::{
    AppContext, Context, Entity, IntoElement, ParentElement, RenderOnce, Styled, Window, div,
};
use gpui::{Subscription, px, rems};
use gpui_component::button::Button;
use gpui_component::input::{Input, InputEvent, InputState};
use gpui_component::resizable::ResizableState;
use gpui_component::{ActiveTheme, h_flex, v_flex, violet};
use rfd::{FileDialog, MessageDialog};
use simple_gpui::{component, component_property, init_with_context};
use crate::modal::app_state::AppState;

#[component]
pub fn editor(window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    init_with_context!();
    component_property!(project_original: LedMakerProject);
    component_property!(project: LedMakerProject = project_original.clone());
    component_property!(project_name_input: Entity<InputState> = cx.new(|cx| {
        let mut input = InputState::new(window, cx).placeholder("Unnamed Project");
        input.set_value(project_original.name.clone(), window, cx);
        input
    }));
    component_property!(_app_state_observer: Subscription = cx.observe_global_in::<AppState>(window, |page, window, cx| {
        let app_state = cx.global::<AppState>();
        let path = app_state.file_path.clone();
        let project = app_state.current_project.clone();
        println!("AppState changed: file_path={:?}, project_name={}", path, project.name);
        page.project = project;
        let project_name = page.project.name.clone();
        page.project_name_input.update(cx, |input, cx| {
            input.set_value(project_name, window, cx)
        });

        cx.notify();
    }));
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
        .child(
            div().flex_grow()
        )
        .child(
            h_flex()
                .w_full()
                .justify_center()
                .child(
                    Button::new("editor-save")
                        .label("Save")
                        .on_click(cx.listener(|view, _, _, cx| {
                            save_project(&view.project, cx);
                        }))
                        .px_6()
                        .py_2()
                )
        )
}

fn save_project(project: &LedMakerProject, cx: &mut Context<Editor>) {
    println!("Saving project: {}", project.name);
    let app_state = cx.global_mut::<AppState>();
    if app_state.file_path.is_none() {
        let files = FileDialog::new()
            .add_filter("project file", &["ledm", "toml"])
            .add_filter("all files", &["*"])
            .set_directory(".")
            .save_file();
        let Some(path) = files else { return; };
        app_state.file_path = Some(path);
    }
    app_state.current_project = project.clone();

    match project.save(app_state.file_path.as_ref().unwrap()) {
        Ok(_) => println!("Project saved successfully."),
        Err(err) => {
            println!("Error saving project: {}", err);
            MessageDialog::new()
                .set_title("Error")
                .set_description(&format!("Failed to save project:\n{}", err))
                .set_buttons(rfd::MessageButtons::Ok)
                .set_level(rfd::MessageLevel::Error)
                .show();
        },
    }
}