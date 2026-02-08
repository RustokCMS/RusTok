use serde::{Deserialize, Serialize};

pub const ADMIN_TOKEN_KEY: &str = "rustok-admin-token";
pub const ADMIN_TENANT_KEY: &str = "rustok-admin-tenant";
pub const ADMIN_USER_KEY: &str = "rustok-admin-user";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthUser {
    pub id: String,
    pub email: String,
    pub name: Option<String>,
    pub role: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthSession {
    pub token: String,
    pub tenant: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthError {
    Unauthorized,
    InvalidCredentials,
    Network,
    Http(u16),
}

impl AuthError {
    pub fn from_status(status: u16, is_login: bool) -> Self {
        match status {
            401 if is_login => Self::InvalidCredentials,
            401 => Self::Unauthorized,
            status => Self::Http(status),
        }
    }
}
