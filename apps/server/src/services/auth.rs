use loco_rs::prelude::*;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};

use rustok_core::{Action, Permission, Rbac, Resource, UserRole};

use crate::models::_entities::{permissions, role_permissions, roles, user_roles};

pub struct AuthService;

impl AuthService {
    pub async fn has_permission(
        db: &DatabaseConnection,
        user_id: &uuid::Uuid,
        required_permission: &Permission,
    ) -> Result<bool> {
        let user_permissions = Self::get_user_permissions(db, user_id).await?;
        Ok(user_permissions.contains(required_permission))
    }

    pub async fn get_user_permissions(
        db: &DatabaseConnection,
        user_id: &uuid::Uuid,
    ) -> Result<Vec<Permission>> {
        let user_role_models = user_roles::Entity::find()
            .filter(user_roles::Column::UserId.eq(*user_id))
            .all(db)
            .await?;

        if user_role_models.is_empty() {
            return Ok(vec![]);
        }

        let role_ids: Vec<uuid::Uuid> = user_role_models
            .into_iter()
            .map(|user_role| user_role.role_id)
            .collect();

        let role_permission_models = role_permissions::Entity::find()
            .filter(role_permissions::Column::RoleId.is_in(role_ids))
            .all(db)
            .await?;

        if role_permission_models.is_empty() {
            return Ok(vec![]);
        }

        let permission_ids: Vec<uuid::Uuid> = role_permission_models
            .into_iter()
            .map(|role_permission| role_permission.permission_id)
            .collect();

        let permission_models = permissions::Entity::find()
            .filter(permissions::Column::Id.is_in(permission_ids))
            .all(db)
            .await?;

        let mut result = Vec::new();
        for permission in permission_models {
            let resource = permission
                .resource
                .parse::<Resource>()
                .map_err(|err| Error::BadRequest(err))?;
            let action = permission
                .action
                .parse::<Action>()
                .map_err(|err| Error::BadRequest(err))?;
            result.push(Permission::new(resource, action));
        }

        Ok(result)
    }

    pub async fn assign_role_permissions(
        db: &DatabaseConnection,
        user_id: &uuid::Uuid,
        tenant_id: &uuid::Uuid,
        role: UserRole,
    ) -> Result<()> {
        let role_model = Self::get_or_create_role(db, tenant_id, &role).await?;

        let user_role = user_roles::ActiveModel {
            id: ActiveValue::Set(rustok_core::generate_id()),
            user_id: ActiveValue::Set(*user_id),
            role_id: ActiveValue::Set(role_model.id),
        };
        user_role.insert(db).await?;

        for permission in Rbac::permissions_for_role(&role).iter() {
            if let Some(permission_model) =
                Self::get_or_create_permission(db, tenant_id, permission).await?
            {
                let role_permission = role_permissions::ActiveModel {
                    id: ActiveValue::Set(rustok_core::generate_id()),
                    role_id: ActiveValue::Set(role_model.id),
                    permission_id: ActiveValue::Set(permission_model.id),
                };
                role_permission.insert(db).await?;
            }
        }

        Ok(())
    }

    async fn get_or_create_role(
        db: &DatabaseConnection,
        tenant_id: &uuid::Uuid,
        role: &UserRole,
    ) -> Result<roles::Model> {
        let role_slug = role.to_string();
        if let Some(existing) = roles::Entity::find()
            .filter(roles::Column::TenantId.eq(*tenant_id))
            .filter(roles::Column::Slug.eq(&role_slug))
            .one(db)
            .await?
        {
            return Ok(existing);
        }

        let role = roles::ActiveModel {
            id: ActiveValue::Set(rustok_core::generate_id()),
            tenant_id: ActiveValue::Set(*tenant_id),
            name: ActiveValue::Set(role_slug.clone()),
            slug: ActiveValue::Set(role_slug),
            description: ActiveValue::Set(None),
            is_system: ActiveValue::Set(true),
            created_at: ActiveValue::NotSet,
            updated_at: ActiveValue::NotSet,
        }
        .insert(db)
        .await?;

        Ok(role)
    }

    async fn get_or_create_permission(
        db: &DatabaseConnection,
        tenant_id: &uuid::Uuid,
        permission: &Permission,
    ) -> Result<Option<permissions::Model>> {
        let resource_str = permission.resource.to_string();
        let action_str = permission.action.to_string();

        if let Some(existing) = permissions::Entity::find()
            .filter(permissions::Column::TenantId.eq(*tenant_id))
            .filter(permissions::Column::Resource.eq(&resource_str))
            .filter(permissions::Column::Action.eq(&action_str))
            .one(db)
            .await?
        {
            return Ok(Some(existing));
        }

        let permission = permissions::ActiveModel {
            id: ActiveValue::Set(rustok_core::generate_id()),
            tenant_id: ActiveValue::Set(*tenant_id),
            resource: ActiveValue::Set(resource_str),
            action: ActiveValue::Set(action_str),
            description: ActiveValue::Set(None),
            created_at: ActiveValue::NotSet,
        }
        .insert(db)
        .await?;

        Ok(Some(permission))
    }
}
