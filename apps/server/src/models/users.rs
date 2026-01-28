use sea_orm::prelude::*;

use rustok_core::{generate_id, UserRole, UserStatus};

use super::_entities::users::{self};
pub use super::_entities::users::{ActiveModel, Entity, Model};

impl Model {
    pub fn is_active(&self) -> bool {
        self.status == UserStatus::Active
    }

    pub fn is_admin(&self) -> bool {
        matches!(self.role, UserRole::Admin | UserRole::SuperAdmin)
    }

    pub fn is_email_verified(&self) -> bool {
        self.email_verified_at.is_some()
    }
}

impl ActiveModel {
    pub fn new(tenant_id: Uuid, email: &str, password_hash: &str) -> Self {
        Self {
            id: sea_orm::ActiveValue::Set(generate_id()),
            tenant_id: sea_orm::ActiveValue::Set(tenant_id),
            email: sea_orm::ActiveValue::Set(email.to_lowercase()),
            password_hash: sea_orm::ActiveValue::Set(password_hash.to_string()),
            name: sea_orm::ActiveValue::NotSet,
            role: sea_orm::ActiveValue::Set(UserRole::Customer),
            status: sea_orm::ActiveValue::Set(UserStatus::Active),
            email_verified_at: sea_orm::ActiveValue::NotSet,
            last_login_at: sea_orm::ActiveValue::NotSet,
            metadata: sea_orm::ActiveValue::Set(serde_json::json!({})),
            created_at: sea_orm::ActiveValue::NotSet,
            updated_at: sea_orm::ActiveValue::NotSet,
        }
    }

    pub fn new_admin(tenant_id: Uuid, email: &str, password_hash: &str) -> Self {
        let mut model = Self::new(tenant_id, email, password_hash);
        model.role = sea_orm::ActiveValue::Set(UserRole::Admin);
        model
    }
}

impl Entity {
    pub async fn find_by_email(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        email: &str,
    ) -> Result<Option<Model>, DbErr> {
        Self::find()
            .filter(users::Column::TenantId.eq(tenant_id))
            .filter(users::Column::Email.eq(email.to_lowercase()))
            .one(db)
            .await
    }

    pub async fn find_admins(
        db: &DatabaseConnection,
        tenant_id: Uuid,
    ) -> Result<Vec<Model>, DbErr> {
        Self::find()
            .filter(users::Column::TenantId.eq(tenant_id))
            .filter(users::Column::Role.is_in([
                UserRole::Admin.to_string(),
                UserRole::SuperAdmin.to_string(),
            ]))
            .all(db)
            .await
    }
}
