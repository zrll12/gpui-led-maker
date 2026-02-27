use serde::{Deserialize, Serialize};

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
