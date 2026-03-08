use crate::modal::AppError;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const APP_DIR_NAME: &str = "led-maker";
const CONFIG_FILE_NAME: &str = "config.toml";
const MAX_RECENT_PROJECTS: usize = 20;

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct NamedPath {
    pub path: PathBuf,
    pub name: String,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct AppConfig {
    pub recent_projects: Vec<NamedPath>,
    pub font_list: Vec<NamedPath>,
}

impl AppConfig {
    pub fn default_config_path() -> PathBuf {
        let mut base_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        base_dir.push(APP_DIR_NAME);
        base_dir.push(CONFIG_FILE_NAME);
        base_dir
    }

    pub fn load_or_default() -> Self {
        Self::load().unwrap_or_default()
    }

    pub fn load() -> Result<Self, AppError> {
        let path = Self::default_config_path();
        println!("Loading config file: {}", path.display());
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn save(&self) -> Result<(), AppError> {
        let path = Self::default_config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn add_recent_project(&mut self, path: PathBuf, name: String) {
        self.recent_projects.retain(|item| item.path != path);
        self.recent_projects.insert(0, NamedPath { path, name });
        if self.recent_projects.len() > MAX_RECENT_PROJECTS {
            self.recent_projects.truncate(MAX_RECENT_PROJECTS);
        }
    }

    pub fn add_font(&mut self, path: PathBuf, name: String) {
        self.font_list.retain(|item| item.path != path);
        self.font_list.push(NamedPath { path, name });
    }

    pub fn remove_font<P: AsRef<Path>>(&mut self, path: P) {
        self.font_list.retain(|item| item.path != path.as_ref());
    }

}
