use sea_orm::DbErr;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum ContentError {
    #[error("Database error: {0}")]
    Database(#[from] DbErr),

    #[error("Core error: {0}")]
    Core(#[from] rustok_core::Error),

    #[error("Node not found: {0}")]
    NodeNotFound(Uuid),

    #[error("Translation not found for node {node_id} and locale {locale}")]
    TranslationNotFound { node_id: Uuid, locale: String },

    #[error("Validation error: {0}")]
    Validation(String),
}

pub type ContentResult<T> = Result<T, ContentError>;
