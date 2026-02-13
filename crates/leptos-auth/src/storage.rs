use gloo_storage::{LocalStorage, Storage};

use crate::{AuthError, AuthSession, AuthUser};
use crate::{ADMIN_SESSION_KEY, ADMIN_TENANT_KEY, ADMIN_TOKEN_KEY, ADMIN_USER_KEY};

pub fn save_session(session: &AuthSession) -> Result<(), AuthError> {
    LocalStorage::set(ADMIN_SESSION_KEY, session)
        .map_err(|_| AuthError::Network)?;
    LocalStorage::set(ADMIN_TOKEN_KEY, &session.token)
        .map_err(|_| AuthError::Network)?;
    LocalStorage::set(ADMIN_TENANT_KEY, &session.tenant)
        .map_err(|_| AuthError::Network)?;
    Ok(())
}

pub fn load_session() -> Result<AuthSession, AuthError> {
    LocalStorage::get(ADMIN_SESSION_KEY)
        .map_err(|_| AuthError::Unauthorized)
}

pub fn save_user(user: &AuthUser) -> Result<(), AuthError> {
    LocalStorage::set(ADMIN_USER_KEY, user)
        .map_err(|_| AuthError::Network)
}

pub fn load_user() -> Result<AuthUser, AuthError> {
    LocalStorage::get(ADMIN_USER_KEY)
        .map_err(|_| AuthError::Unauthorized)
}

pub fn clear_session() {
    let _ = LocalStorage::delete(ADMIN_SESSION_KEY);
    let _ = LocalStorage::delete(ADMIN_TOKEN_KEY);
    let _ = LocalStorage::delete(ADMIN_TENANT_KEY);
    let _ = LocalStorage::delete(ADMIN_USER_KEY);
}

pub fn get_token() -> Option<String> {
    LocalStorage::get(ADMIN_TOKEN_KEY).ok()
}

pub fn get_tenant() -> Option<String> {
    LocalStorage::get(ADMIN_TENANT_KEY).ok()
}
