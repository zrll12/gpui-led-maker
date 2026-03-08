use crate::gui::Render;
use crate::modal::app_state::AppState;
use crate::modal::project::LedMakerProject;
use gpui::{Context, IntoElement, ParentElement, Styled, Window, div};
use gpui_component::button::Button;
use gpui_component::v_flex;
use rfd::MessageDialog;
use simple_gpui::component;

#[component]
pub fn recent_projects(_window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let recent_projects = cx.global::<AppState>().config.recent_projects.clone();

    if recent_projects.is_empty() {
        return div().size_full().p_3().child("No recent projects");
    }

    v_flex()
        .size_full()
        .p_2()
        .gap_2()
        .children(recent_projects.into_iter().enumerate().map(|(index, item)| {
            let path = item.path.clone();
            let button_label = if item.name.is_empty() {
                item.path.display().to_string()
            } else {
                format!("{} ({})", item.name, item.path.display())
            };

            Button::new(("recent-project", index))
                .label(button_label)
                .w_full()
                .on_click(cx.listener(move |_, _, _, cx| {
                    match LedMakerProject::load(&path) {
                        Ok(project) => {
                            let app_state = cx.global_mut::<AppState>();
                            app_state
                                .config
                                .add_recent_project(path.clone(), project.name.clone());
                            if let Err(err) = app_state.config.save() {
                                println!("Error saving config: {}", err);
                            }

                            app_state.file_path = Some(path.clone());
                            app_state.current_project = project;
                            cx.notify();
                        }
                        Err(err) => {
                            println!("Error loading project: {}", err);
                            MessageDialog::new()
                                .set_title("Error")
                                .set_description(format!("Failed to load project:\n{}", err))
                                .set_buttons(rfd::MessageButtons::Ok)
                                .set_level(rfd::MessageLevel::Error)
                                .show();
                        }
                    }
                }))
        }))
}