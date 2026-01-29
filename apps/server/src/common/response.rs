use axum::{http::StatusCode, Json};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ApiError>,
}

#[derive(Debug, Serialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(code: impl Into<String>, message: impl Into<String>) -> ApiResponse<()> {
        ApiResponse {
            success: false,
            data: None,
            error: Some(ApiError {
                code: code.into(),
                message: message.into(),
                details: None,
            }),
        }
    }
}

impl From<rustok_commerce::CommerceError> for (StatusCode, Json<ApiResponse<()>>) {
    fn from(err: rustok_commerce::CommerceError) -> Self {
        let (status, code) = match &err {
            rustok_commerce::CommerceError::ProductNotFound(_) => {
                (StatusCode::NOT_FOUND, "PRODUCT_NOT_FOUND")
            }
            rustok_commerce::CommerceError::VariantNotFound(_) => {
                (StatusCode::NOT_FOUND, "VARIANT_NOT_FOUND")
            }
            rustok_commerce::CommerceError::DuplicateHandle { .. } => {
                (StatusCode::CONFLICT, "DUPLICATE_HANDLE")
            }
            rustok_commerce::CommerceError::DuplicateSku(_) => {
                (StatusCode::CONFLICT, "DUPLICATE_SKU")
            }
            rustok_commerce::CommerceError::InsufficientInventory { .. } => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "INSUFFICIENT_INVENTORY",
            ),
            rustok_commerce::CommerceError::Validation(_) => {
                (StatusCode::BAD_REQUEST, "VALIDATION_ERROR")
            }
            rustok_commerce::CommerceError::CannotDeletePublished => {
                (StatusCode::CONFLICT, "CANNOT_DELETE_PUBLISHED")
            }
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR"),
        };

        (status, Json(ApiResponse::error(code, err.to_string())))
    }
}
