use crate::modal::config::AppConfig;
use crate::modal::project::LedMakerProject;
use gpui::Global;
use std::path::PathBuf;

pub struct AppState {
    pub file_path: Option<PathBuf>,
    pub current_project: LedMakerProject,
    pub config: AppConfig,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            file_path: None,
            current_project: LedMakerProject::default(),
            config: AppConfig::load_or_default(),
        }
    }
}

impl Global for AppState {}
