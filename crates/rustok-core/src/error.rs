use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Invalid ID format: {0}")]
    InvalidIdFormat(String),

    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Auth error: {0}")]
    Auth(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Permission denied: {0}")]
    Forbidden(String),

    #[error("Cache error: {0}")]
    Cache(String),

    #[error("Scripting error: {0}")]
    Scripting(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("External error: {0}")]
    External(String),
}
