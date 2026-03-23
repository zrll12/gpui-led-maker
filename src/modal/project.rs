use crate::modal::AppError;
use crate::modal::app_state::AppState;
use crate::modal::config::NamedPath;
use gpui::Context;
use rfd::{AsyncFileDialog, MessageDialog};
use serde::{Deserialize, Serialize};
use std::io;
use std::path::PathBuf;

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
pub struct LedMakerProject {
    pub name: String,
    pub frames: Vec<Frame>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Frame {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub contents: Vec<PositionedLayer>,
}

impl Default for Frame {
    fn default() -> Self {
        Self {
            name: String::new(),
            width: 32,
            height: 8,
            contents: Vec::new(),
        }
    }
}

/// 带位置信息的图层包装，x/y 为图层左上角相对画布左上角的偏移（可为负）
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct PositionedLayer {
    pub x: i32,
    pub y: i32,
    pub content: ComponentLayer,
}

impl Default for PositionedLayer {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            content: ComponentLayer::Text(TextComponent::default()),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum ComponentLayer {
    Text(TextComponent),
    Rectangle(RectangleComponent),
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
pub struct TextComponent {
    pub text: String,
    pub font: String,
    pub color: (u8, u8, u8),
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
pub struct RectangleComponent {
    pub width: u32,
    pub height: u32,
    pub radius: u32,
    pub color: (u8, u8, u8),
}

impl LedMakerProject {
    pub fn normalize_text_fonts_to_names(&mut self, font_list: &[NamedPath]) {
        for frame in &mut self.frames {
            for layer in &mut frame.contents {
                if let ComponentLayer::Text(text) = &mut layer.content
                    && let Some(named_font) = font_list
                        .iter()
                        .find(|np| !np.name.is_empty() && np.path.to_string_lossy() == text.font)
                {
                    text.font = named_font.name.clone();
                }
            }
        }
    }

    pub fn load(path: &PathBuf) -> Result<Self, AppError> {
        let file = std::fs::read_to_string(path)?;
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
        let mut project = self.clone();
        let font_list = cx.global::<AppState>().config.font_list.clone();
        project.normalize_text_fonts_to_names(&font_list);
        let file_path = cx.global::<AppState>().file_path.clone();

        cx.spawn(move |_: gpui::WeakEntity<T>, cx: &mut gpui::AsyncApp| {
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
        })
        .detach();
    }
}
