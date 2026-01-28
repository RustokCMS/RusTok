use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Invalid ID format: {0}")]
    InvalidIdFormat(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Auth error: {0}")]
    Auth(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}
