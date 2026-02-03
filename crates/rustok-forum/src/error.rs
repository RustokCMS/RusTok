use thiserror::Error;

#[derive(Debug, Error)]
pub enum ForumError {
    #[error("Thread not found: {0}")]
    ThreadNotFound(String),
}
