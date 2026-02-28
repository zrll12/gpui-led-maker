use std::io;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use crate::modal::AppError;

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct LedMakerProject {
    pub name: String,
    pub font_path: String,
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
}