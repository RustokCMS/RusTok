use rustok_core::{Permission, UserRole};
use uuid::Uuid;

#[derive(Clone)]
pub struct AuthContext {
    pub user_id: Uuid,
    pub tenant_id: Uuid,
    pub role: UserRole,
    pub permissions: Vec<Permission>,
}
