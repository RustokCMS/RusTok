use async_graphql::{InputObject, SimpleObject};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, InputObject)]
pub struct SignInInput {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, InputObject)]
pub struct SignUpInput {
    pub email: String,
    pub password: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone, InputObject)]
pub struct RefreshTokenInput {
    pub refresh_token: String,
}

#[derive(Debug, Clone, InputObject)]
pub struct ForgotPasswordInput {
    pub email: String,
}

#[derive(Debug, Clone, InputObject)]
pub struct ResetPasswordInput {
    pub token: String,
    pub new_password: String,
}

#[derive(Debug, Clone, SimpleObject, Serialize, Deserialize)]
pub struct AuthPayload {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i32,
    pub user: AuthUser,
}

#[derive(Debug, Clone, SimpleObject, Serialize, Deserialize)]
pub struct AuthUser {
    pub id: String,
    pub email: String,
    pub name: Option<String>,
    pub role: String,
    pub status: String,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct SignOutPayload {
    pub success: bool,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct ForgotPasswordPayload {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct ResetPasswordPayload {
    pub success: bool,
}

#[derive(Debug, Clone, InputObject)]
pub struct UpdateProfileInput {
    pub name: Option<String>,
}

#[derive(Debug, Clone, InputObject)]
pub struct ChangePasswordInput {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct ChangePasswordPayload {
    pub success: bool,
}
