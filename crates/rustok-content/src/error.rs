use rustok_core::error::{ErrorKind, RichError};
use sea_orm::DbErr;
use thiserror::Error;
use uuid::Uuid;

/// Content module errors
///
/// Uses both legacy Error enum and new RichError system.
/// Gradually migrate to RichError for better context.
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

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Rich error: {0}")]
    Rich(#[from] RichError),
}

pub type ContentResult<T> = Result<T, ContentError>;

// Conversion from ContentError to RichError for API responses
impl From<ContentError> for RichError {
    fn from(err: ContentError) -> Self {
        match err {
            ContentError::Database(db_err) => {
                RichError::new(ErrorKind::Database, "Database operation failed")
                    .with_user_message("Unable to access content data")
                    .with_source(db_err)
            }
            ContentError::Core(core_err) => {
                core_err.into()
            }
            ContentError::NodeNotFound(id) => {
                RichError::new(ErrorKind::NotFound, format!("Node {} not found", id))
                    .with_user_message("The requested content does not exist")
                    .with_field("node_id", id.to_string())
                    .with_error_code("NODE_NOT_FOUND")
            }
            ContentError::TranslationNotFound { node_id, locale } => {
                RichError::new(
                    ErrorKind::NotFound,
                    format!("Translation for node {} in locale {} not found", node_id, locale),
                )
                .with_user_message("Translation not available for this content")
                .with_field("node_id", node_id.to_string())
                .with_field("locale", locale)
                .with_error_code("TRANSLATION_NOT_FOUND")
            }
            ContentError::Forbidden(msg) => {
                RichError::new(ErrorKind::Forbidden, msg)
                    .with_user_message("You do not have permission to perform this action")
            }
            ContentError::Validation(msg) => {
                RichError::new(ErrorKind::Validation, msg)
                    .with_user_message("Invalid input data")
            }
            ContentError::Rich(rich) => rich,
        }
    }
}

/// Helper functions for creating common content errors
impl ContentError {
    /// Create a node not found error with rich context
    pub fn node_not_found(node_id: Uuid) -> Self {
        ContentError::NodeNotFound(node_id)
    }
    
    /// Create a translation not found error with rich context
    pub fn translation_not_found(node_id: Uuid, locale: impl Into<String>) -> Self {
        ContentError::TranslationNotFound {
            node_id,
            locale: locale.into(),
        }
    }
    
    /// Create a validation error
    pub fn validation(message: impl Into<String>) -> Self {
        ContentError::Validation(message.into())
    }
    
    /// Create a forbidden error
    pub fn forbidden(message: impl Into<String>) -> Self {
        ContentError::Forbidden(message.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_node_not_found_conversion() {
        let id = Uuid::new_v4();
        let err = ContentError::node_not_found(id);
        let rich: RichError = err.into();
        
        assert_eq!(rich.kind, ErrorKind::NotFound);
        assert_eq!(rich.status_code, 404);
        assert!(rich.fields.contains_key("node_id"));
    }
    
    #[test]
    fn test_translation_not_found_conversion() {
        let id = Uuid::new_v4();
        let err = ContentError::translation_not_found(id, "en");
        let rich: RichError = err.into();
        
        assert_eq!(rich.kind, ErrorKind::NotFound);
        assert_eq!(rich.fields.get("locale"), Some(&"en".to_string()));
    }
}
