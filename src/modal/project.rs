use std::io;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use gpui::Context;
use rfd::{AsyncFileDialog, MessageDialog};
use crate::modal::AppError;
use crate::modal::app_state::AppState;

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct LedMakerProject {
    pub name: String,
    pub frames: Vec<Frame>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Frame {
    pub name: String,
    pub contents: String,
}

impl LedMakerProject {
    pub fn load(path: &PathBuf) -> Result<Self, AppError> {
        let file = std::fs::read_to_string(&path)?;
        let project: Self = toml::from_str(&file)?;
        
        Ok(project)
    }
    
    pub fn save(&self, path: &PathBuf) -> Result<(), AppError> {
        let toml_string = toml::to_string(self)?;
        std::fs::write(path, toml_string)?;
        
        Ok(())
    }

    pub fn save_project<T: 'static>(&self, cx: &mut Context<T>) {
        println!("Saving project: {}", self.name);
        let project = self.clone();
        let file_path = cx.global::<AppState>().file_path.clone();

        cx.spawn(
            move |_: gpui::WeakEntity<T>, cx: &mut gpui::AsyncApp| {
                let cx = cx.clone();
                async move {
                    let path = if let Some(path) = file_path {
                        path
                    } else {
                        let file = AsyncFileDialog::new()
                            .add_filter("project file", &["ledm", "toml"])
                            .add_filter("all files", &["*"])
                            .set_directory(".")
                            .save_file()
                            .await;
                        let Some(file) = file else {
                            return;
                        };
                        file.path().to_path_buf()
                    };

                    let project_for_state = project.clone();
                    let path_for_state = path.clone();
                    let _ = cx.update_global::<AppState, _>(|app_state, _| {
                        app_state.file_path = Some(path_for_state);
                        app_state.current_project = project_for_state;
                    });

                    match project.save(&path) {
                        Ok(_) => {
                            let project_name = project.name.clone();
                            let path_for_recent = path.clone();
                            let _ = cx.update_global::<AppState, _>(|app_state, _| {
                                app_state
                                    .config
                                    .add_recent_project(path_for_recent, project_name);
                                if let Err(err) = app_state.config.save() {
                                    println!("Error saving config: {}", err);
                                }
                            });

                            println!("Project saved successfully.");
                        }
                        Err(err) => {
                            println!("Error saving project: {}", err);
                            MessageDialog::new()
                                .set_title("Error")
                                .set_description(format!("Failed to save project:\n{}", err))
                                .set_buttons(rfd::MessageButtons::Ok)
                                .set_level(rfd::MessageLevel::Error)
                                .show();
                        }
                    }
                }
            },
        )
        .detach();
    }
}