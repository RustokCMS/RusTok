use async_graphql::{Context, FieldError, Object, Result};
use rustok_core::auth::password::hash_password;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set, TransactionTrait};

use crate::context::{AuthContext, TenantContext};
use crate::graphql::errors::GraphQLError;
use crate::graphql::types::{
    CreateUserInput, DeleteUserPayload, TenantModule, UpdateUserInput, User,
};
use crate::models::_entities::users::Column as UsersColumn;
use crate::models::users;
use crate::services::auth::AuthService;
use crate::services::module_lifecycle::{ModuleLifecycleService, ToggleModuleError};
use rustok_core::{Action, ModuleRegistry, Permission, Resource};

#[derive(Default)]
pub struct RootMutation;

#[Object]
impl RootMutation {
    async fn create_user(&self, ctx: &Context<'_>, input: CreateUserInput) -> Result<User> {
        let auth = ctx
            .data::<AuthContext>()
            .map_err(|_| <FieldError as GraphQLError>::unauthenticated())?;
        let tenant = ctx.data::<TenantContext>()?;
        let app_ctx = ctx.data::<loco_rs::prelude::AppContext>()?;

        let can_create_users = AuthService::has_any_permission(
            &app_ctx.db,
            &tenant.id,
            &auth.user_id,
            &[
                rustok_core::Permission::USERS_CREATE,
                rustok_core::Permission::USERS_MANAGE,
            ],
        )
        .await
        .map_err(|err| <FieldError as GraphQLError>::internal_error(&err.to_string()))?;

        if !can_create_users {
            return Err(<FieldError as GraphQLError>::permission_denied(
                "Permission denied: users:create required",
            ));
        }

        let existing = users::Entity::find_by_email(&app_ctx.db, tenant.id, &input.email)
            .await
            .map_err(|err| <FieldError as GraphQLError>::internal_error(&err.to_string()))?;

        if existing.is_some() {
            return Err(FieldError::new("User with this email already exists"));
        }

        let password_hash = hash_password(&input.password)
            .map_err(|err| <FieldError as GraphQLError>::internal_error(&err.to_string()))?;

        let user_role = input
            .role
            .map(Into::into)
            .unwrap_or(rustok_core::UserRole::Customer);

        let tx = app_ctx
            .db
            .begin()
            .await
            .map_err(|err| <FieldError as GraphQLError>::internal_error(&err.to_string()))?;

        let mut model = users::ActiveModel::new(tenant.id, &input.email, &password_hash);

        if let Some(name) = input.name {
            model.name = Set(Some(name));
        }

        model.role = Set(user_role.clone());

        if let Some(status) = input.status {
            let status: rustok_core::UserStatus = status.into();
            model.status = Set(status);
        }

        let user = model
            .insert(&tx)
            .await
            .map_err(|err| <FieldError as GraphQLError>::internal_error(&err.to_string()))?;

        AuthService::assign_role_permissions(&tx, &user.id, &tenant.id, user_role)
            .await
            .map_err(|err| <FieldError as GraphQLError>::internal_error(&err.to_string()))?;

        tx.commit()
            .await
            .map_err(|err| <FieldError as GraphQLError>::internal_error(&err.to_string()))?;

        Ok(User::from(&user))
    }

    async fn update_user(
        &self,
        ctx: &Context<'_>,
        id: uuid::Uuid,
        input: UpdateUserInput,
    ) -> Result<User> {
        let auth = ctx
            .data::<AuthContext>()
            .map_err(|_| <FieldError as GraphQLError>::unauthenticated())?;
        let tenant = ctx.data::<TenantContext>()?;
        let app_ctx = ctx.data::<loco_rs::prelude::AppContext>()?;

        let can_update_users = AuthService::has_any_permission(
            &app_ctx.db,
            &tenant.id,
            &auth.user_id,
            &[
                rustok_core::Permission::USERS_UPDATE,
                rustok_core::Permission::USERS_MANAGE,
            ],
        )
        .await
        .map_err(|err| <FieldError as GraphQLError>::internal_error(&err.to_string()))?;

        if !can_update_users {
            return Err(<FieldError as GraphQLError>::permission_denied(
                "Permission denied: users:update required",
            ));
        }

        let user = users::Entity::find_by_id(id)
            .filter(UsersColumn::TenantId.eq(tenant.id))
            .one(&app_ctx.db)
            .await
            .map_err(|err| <FieldError as GraphQLError>::internal_error(&err.to_string()))?
            .ok_or_else(|| FieldError::new("User not found"))?;

        if let Some(email) = &input.email {
            let existing = users::Entity::find_by_email(&app_ctx.db, tenant.id, email)
                .await
                .map_err(|err| <FieldError as GraphQLError>::internal_error(&err.to_string()))?;

            if existing
                .as_ref()
                .is_some_and(|existing| existing.id != user.id)
            {
                return Err(FieldError::new("User with this email already exists"));
            }
        }

        let mut model: users::ActiveModel = user.into();

        if let Some(email) = input.email {
            model.email = Set(email.to_lowercase());
        }

        if let Some(name) = input.name {
            model.name = Set(Some(name));
        }

        if let Some(role) = input.role {
            let role: rustok_core::UserRole = role.into();
            model.role = Set(role);
        }

        if let Some(status) = input.status {
            let status: rustok_core::UserStatus = status.into();
            model.status = Set(status);
        }

        if let Some(password) = input.password {
            let password_hash = hash_password(&password)
                .map_err(|err| <FieldError as GraphQLError>::internal_error(&err.to_string()))?;
            model.password_hash = Set(password_hash);
        }

        let user = model
            .update(&app_ctx.db)
            .await
            .map_err(|err| <FieldError as GraphQLError>::internal_error(&err.to_string()))?;

        Ok(User::from(&user))
    }

    async fn disable_user(&self, ctx: &Context<'_>, id: uuid::Uuid) -> Result<User> {
        let auth = ctx
            .data::<AuthContext>()
            .map_err(|_| <FieldError as GraphQLError>::unauthenticated())?;
        let tenant = ctx.data::<TenantContext>()?;
        let app_ctx = ctx.data::<loco_rs::prelude::AppContext>()?;

        let can_manage_users = AuthService::has_permission(
            &app_ctx.db,
            &tenant.id,
            &auth.user_id,
            &rustok_core::Permission::USERS_MANAGE,
        )
        .await
        .map_err(|err| <FieldError as GraphQLError>::internal_error(&err.to_string()))?;

        if !can_manage_users {
            return Err(<FieldError as GraphQLError>::permission_denied(
                "Permission denied: users:manage required",
            ));
        }

        let user = users::Entity::find_by_id(id)
            .filter(UsersColumn::TenantId.eq(tenant.id))
            .one(&app_ctx.db)
            .await
            .map_err(|err| <FieldError as GraphQLError>::internal_error(&err.to_string()))?
            .ok_or_else(|| FieldError::new("User not found"))?;

        let mut model: users::ActiveModel = user.into();
        model.status = Set(rustok_core::UserStatus::Inactive);

        let user = model
            .update(&app_ctx.db)
            .await
            .map_err(|err| <FieldError as GraphQLError>::internal_error(&err.to_string()))?;

        Ok(User::from(&user))
    }

    async fn delete_user(&self, ctx: &Context<'_>, id: uuid::Uuid) -> Result<DeleteUserPayload> {
        let auth = ctx
            .data::<AuthContext>()
            .map_err(|_| <FieldError as GraphQLError>::unauthenticated())?;
        let tenant = ctx.data::<TenantContext>()?;
        let app_ctx = ctx.data::<loco_rs::prelude::AppContext>()?;

        let can_manage_users = AuthService::has_permission(
            &app_ctx.db,
            &tenant.id,
            &auth.user_id,
            &rustok_core::Permission::USERS_MANAGE,
        )
        .await
        .map_err(|err| <FieldError as GraphQLError>::internal_error(&err.to_string()))?;

        if !can_manage_users {
            return Err(<FieldError as GraphQLError>::permission_denied(
                "Permission denied: users:manage required",
            ));
        }

        let user = users::Entity::find_by_id(id)
            .filter(UsersColumn::TenantId.eq(tenant.id))
            .one(&app_ctx.db)
            .await
            .map_err(|err| <FieldError as GraphQLError>::internal_error(&err.to_string()))?
            .ok_or_else(|| FieldError::new("User not found"))?;

        let model: users::ActiveModel = user.into();
        model
            .delete(&app_ctx.db)
            .await
            .map_err(|err| <FieldError as GraphQLError>::internal_error(&err.to_string()))?;

        Ok(DeleteUserPayload { success: true })
    }

    async fn toggle_module(
        &self,
        ctx: &Context<'_>,
        module_slug: String,
        enabled: bool,
    ) -> Result<TenantModule> {
        let auth = ctx
            .data::<AuthContext>()
            .map_err(|_| <FieldError as GraphQLError>::unauthenticated())?;

        let app_ctx = ctx.data::<loco_rs::prelude::AppContext>()?;
        let tenant = ctx.data::<TenantContext>()?;

        let can_manage_modules = AuthService::has_permission(
            &app_ctx.db,
            &tenant.id,
            &auth.user_id,
            &Permission::new(Resource::Modules, Action::Manage),
        )
        .await
        .map_err(|err| <FieldError as GraphQLError>::internal_error(&err.to_string()))?;

        if !can_manage_modules {
            return Err(<FieldError as GraphQLError>::permission_denied(
                "Permission denied: modules:manage required",
            ));
        }

        let registry = ctx.data::<ModuleRegistry>()?;

        let module = ModuleLifecycleService::toggle_module(
            &app_ctx.db,
            registry,
            tenant.id,
            &module_slug,
            enabled,
        )
        .await
        .map_err(|err| match err {
            ToggleModuleError::UnknownModule => FieldError::new("Unknown module"),
            ToggleModuleError::CoreModuleCannotBeDisabled(module_slug) => {
                FieldError::new(format!("Core module cannot be disabled: {}", module_slug))
            }
            ToggleModuleError::MissingDependencies(missing) => {
                FieldError::new(format!("Missing module dependencies: {}", missing))
            }
            ToggleModuleError::HasDependents(dependents) => {
                FieldError::new(format!("Module is required by: {}", dependents))
            }
            ToggleModuleError::Database(err) => {
                <FieldError as GraphQLError>::internal_error(&err.to_string())
            }
            ToggleModuleError::HookFailed(err) => FieldError::new(format!(
                "Module lifecycle hook failed, state rolled back: {}",
                err
            )),
        })?;

        Ok(TenantModule {
            module_slug: module.module_slug,
            enabled: module.enabled,
            settings: module.settings.to_string(),
        })
    }
}
