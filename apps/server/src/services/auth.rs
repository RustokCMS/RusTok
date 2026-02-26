use loco_rs::prelude::*;
use sea_orm::{
    sea_query::OnConflict, ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection,
    EntityTrait, QueryFilter,
};
use std::collections::HashSet;

use rustok_core::{Action, Permission, Rbac, Resource, UserRole};

use crate::models::_entities::{permissions, role_permissions, roles, user_roles};

pub struct AuthService;

impl AuthService {
    fn has_effective_permission(
        user_permissions: &[Permission],
        required_permission: &Permission,
    ) -> bool {
        user_permissions.contains(required_permission)
            || user_permissions.contains(&Permission::new(
                required_permission.resource,
                Action::Manage,
            ))
    }

    pub async fn has_permission(
        db: &DatabaseConnection,
        tenant_id: &uuid::Uuid,
        user_id: &uuid::Uuid,
        required_permission: &Permission,
    ) -> Result<bool> {
        let user_permissions = Self::get_user_permissions(db, tenant_id, user_id).await?;
        Ok(Self::has_effective_permission(
            &user_permissions,
            required_permission,
        ))
    }

    pub async fn has_any_permission(
        db: &DatabaseConnection,
        tenant_id: &uuid::Uuid,
        user_id: &uuid::Uuid,
        required_permissions: &[Permission],
    ) -> Result<bool> {
        if required_permissions.is_empty() {
            return Ok(true);
        }

        let user_permissions = Self::get_user_permissions(db, tenant_id, user_id).await?;
        Ok(required_permissions
            .iter()
            .any(|permission| Self::has_effective_permission(&user_permissions, permission)))
    }

    pub async fn has_all_permissions(
        db: &DatabaseConnection,
        tenant_id: &uuid::Uuid,
        user_id: &uuid::Uuid,
        required_permissions: &[Permission],
    ) -> Result<bool> {
        if required_permissions.is_empty() {
            return Ok(true);
        }

        let user_permissions = Self::get_user_permissions(db, tenant_id, user_id).await?;
        Ok(required_permissions
            .iter()
            .all(|permission| Self::has_effective_permission(&user_permissions, permission)))
    }

    pub async fn get_user_permissions(
        db: &DatabaseConnection,
        tenant_id: &uuid::Uuid,
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

        let tenant_role_models = roles::Entity::find()
            .filter(roles::Column::TenantId.eq(*tenant_id))
            .filter(roles::Column::Id.is_in(role_ids))
            .all(db)
            .await?;

        if tenant_role_models.is_empty() {
            return Ok(vec![]);
        }

        let tenant_role_ids: Vec<uuid::Uuid> =
            tenant_role_models.into_iter().map(|role| role.id).collect();

        let role_permission_models = role_permissions::Entity::find()
            .filter(role_permissions::Column::RoleId.is_in(tenant_role_ids))
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
            .filter(permissions::Column::TenantId.eq(*tenant_id))
            .filter(permissions::Column::Id.is_in(permission_ids))
            .all(db)
            .await?;

        let mut result = HashSet::new();
        for permission in permission_models {
            let resource = permission
                .resource
                .parse::<Resource>()
                .map_err(Error::BadRequest)?;
            let action = permission
                .action
                .parse::<Action>()
                .map_err(Error::BadRequest)?;
            result.insert(Permission::new(resource, action));
        }

        Ok(result.into_iter().collect())
    }

    pub async fn assign_role_permissions(
        db: &DatabaseConnection,
        user_id: &uuid::Uuid,
        tenant_id: &uuid::Uuid,
        role: UserRole,
    ) -> Result<()> {
        let role_model = Self::get_or_create_role(db, tenant_id, &role).await?;

        user_roles::Entity::insert(user_roles::ActiveModel {
            id: ActiveValue::Set(rustok_core::generate_id()),
            user_id: ActiveValue::Set(*user_id),
            role_id: ActiveValue::Set(role_model.id),
        })
        .on_conflict(
            OnConflict::columns([user_roles::Column::UserId, user_roles::Column::RoleId])
                .do_nothing()
                .to_owned(),
        )
        .exec(db)
        .await?;

        for permission in Rbac::permissions_for_role(&role).iter() {
            let permission_model =
                Self::get_or_create_permission(db, tenant_id, permission).await?;

            role_permissions::Entity::insert(role_permissions::ActiveModel {
                id: ActiveValue::Set(rustok_core::generate_id()),
                role_id: ActiveValue::Set(role_model.id),
                permission_id: ActiveValue::Set(permission_model.id),
            })
            .on_conflict(
                OnConflict::columns([
                    role_permissions::Column::RoleId,
                    role_permissions::Column::PermissionId,
                ])
                .do_nothing()
                .to_owned(),
            )
            .exec(db)
            .await?;
        }

        Ok(())
    }

    pub async fn replace_user_role(
        db: &DatabaseConnection,
        user_id: &uuid::Uuid,
        tenant_id: &uuid::Uuid,
        role: UserRole,
    ) -> Result<()> {
        let tenant_role_models = roles::Entity::find()
            .filter(roles::Column::TenantId.eq(*tenant_id))
            .all(db)
            .await?;

        let tenant_role_ids: Vec<uuid::Uuid> = tenant_role_models
            .into_iter()
            .map(|tenant_role| tenant_role.id)
            .collect();

        if !tenant_role_ids.is_empty() {
            user_roles::Entity::delete_many()
                .filter(user_roles::Column::UserId.eq(*user_id))
                .filter(user_roles::Column::RoleId.is_in(tenant_role_ids))
                .exec(db)
                .await?;
        }

        Self::assign_role_permissions(db, user_id, tenant_id, role).await
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

        roles::Entity::insert(roles::ActiveModel {
            id: ActiveValue::Set(rustok_core::generate_id()),
            tenant_id: ActiveValue::Set(*tenant_id),
            name: ActiveValue::Set(role_slug.clone()),
            slug: ActiveValue::Set(role_slug),
            description: ActiveValue::Set(None),
            is_system: ActiveValue::Set(true),
            created_at: ActiveValue::NotSet,
            updated_at: ActiveValue::NotSet,
        })
        .on_conflict(
            OnConflict::columns([roles::Column::TenantId, roles::Column::Slug])
                .do_nothing()
                .to_owned(),
        )
        .exec(db)
        .await?;

        roles::Entity::find()
            .filter(roles::Column::TenantId.eq(*tenant_id))
            .filter(roles::Column::Slug.eq(role.to_string()))
            .one(db)
            .await?
            .ok_or_else(|| Error::InternalServerError("role upsert failed".to_string()))
    }

    async fn get_or_create_permission(
        db: &DatabaseConnection,
        tenant_id: &uuid::Uuid,
        permission: &Permission,
    ) -> Result<permissions::Model> {
        let resource_str = permission.resource.to_string();
        let action_str = permission.action.to_string();

        if let Some(existing) = permissions::Entity::find()
            .filter(permissions::Column::TenantId.eq(*tenant_id))
            .filter(permissions::Column::Resource.eq(&resource_str))
            .filter(permissions::Column::Action.eq(&action_str))
            .one(db)
            .await?
        {
            return Ok(existing);
        }

        permissions::Entity::insert(permissions::ActiveModel {
            id: ActiveValue::Set(rustok_core::generate_id()),
            tenant_id: ActiveValue::Set(*tenant_id),
            resource: ActiveValue::Set(resource_str.clone()),
            action: ActiveValue::Set(action_str.clone()),
            description: ActiveValue::Set(None),
            created_at: ActiveValue::NotSet,
        })
        .on_conflict(
            OnConflict::columns([
                permissions::Column::TenantId,
                permissions::Column::Resource,
                permissions::Column::Action,
            ])
            .do_nothing()
            .to_owned(),
        )
        .exec(db)
        .await?;

        permissions::Entity::find()
            .filter(permissions::Column::TenantId.eq(*tenant_id))
            .filter(permissions::Column::Resource.eq(resource_str))
            .filter(permissions::Column::Action.eq(action_str))
            .one(db)
            .await?
            .ok_or_else(|| Error::InternalServerError("permission upsert failed".to_string()))
    }
}
