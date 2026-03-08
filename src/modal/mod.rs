
use thiserror::Error;

pub mod app_state;
pub mod config;
pub mod project;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Cannot open file")]
    FileError(#[from] std::io::Error),
    #[error("This data is corrupted: {0}")]
    DeserializeError(#[from] toml::de::Error),
    #[error("Cannot serialize data: {0}")]
    SerializeError(#[from] toml::ser::Error),
}