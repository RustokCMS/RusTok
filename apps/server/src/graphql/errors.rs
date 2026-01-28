use async_graphql::{ErrorExtensions, FieldError};

#[derive(Debug, Clone, Copy)]
pub enum ErrorCode {
    Unauthenticated,
    PermissionDenied,
    InternalError,
}

impl ErrorCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Unauthenticated => "UNAUTHENTICATED",
            Self::PermissionDenied => "PERMISSION_DENIED",
            Self::InternalError => "INTERNAL_ERROR",
        }
    }
}

pub trait GraphQLError {
    fn unauthenticated() -> FieldError;
    fn permission_denied(message: &str) -> FieldError;
    fn internal_error(message: &str) -> FieldError;
}

impl GraphQLError for FieldError {
    fn unauthenticated() -> FieldError {
        FieldError::new("Authentication required").extend_with(|_, e| {
            e.set("code", ErrorCode::Unauthenticated.as_str());
        })
    }

    fn permission_denied(message: &str) -> FieldError {
        FieldError::new(message).extend_with(|_, e| {
            e.set("code", ErrorCode::PermissionDenied.as_str());
        })
    }

    fn internal_error(message: &str) -> FieldError {
        FieldError::new(message).extend_with(|_, e| {
            e.set("code", ErrorCode::InternalError.as_str());
        })
    }
}
