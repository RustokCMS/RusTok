use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid storage path: {0}")]
    InvalidPath(String),

    #[error("Object not found: {0}")]
    NotFound(String),

    #[error("Backend error: {0}")]
    Backend(String),
}

pub type Result<T> = std::result::Result<T, StorageError>;
