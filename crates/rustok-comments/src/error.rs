use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum CommentsError {
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),

    #[error("Comment not found: {0}")]
    CommentNotFound(Uuid),

    #[error("Comment thread not found for target {target_type}:{target_id}")]
    CommentThreadNotFound {
        target_type: String,
        target_id: Uuid,
    },

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Validation error: {0}")]
    Validation(String),
}

pub type CommentsResult<T> = Result<T, CommentsError>;
