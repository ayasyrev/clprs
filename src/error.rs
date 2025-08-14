use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClprsError {
    #[error("Failed to access clipboard: {0}")]
    ClipboardError(#[from] arboard::Error),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] toml::de::Error),

    #[error("Empty clipboard")]
    EmptyClipboard,
}

pub type Result<T> = std::result::Result<T, ClprsError>;