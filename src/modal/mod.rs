
use thiserror::Error;

pub mod app_state;
pub mod project;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Cannot open file")]
    FileError(#[from] std::io::Error),
    #[error("This data is corrupted: {0}")]
    DeserializeError(#[from] toml::de::Error),
}