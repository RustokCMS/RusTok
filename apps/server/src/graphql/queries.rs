use async_graphql::{Context, FieldError, Object, Result};
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QuerySelect};

use crate::context::{AuthContext, TenantContext};
use crate::graphql::common::{encode_cursor, PageInfo, PaginationInput};
use crate::graphql::errors::GraphQLError;
use crate::graphql::types::{Tenant, TenantModule, User, UserConnection, UserEdge};
use crate::models::_entities::tenant_modules::Column as TenantModulesColumn;
use crate::models::_entities::tenant_modules::Entity as TenantModulesEntity;
use crate::models::_entities::users::Column as UsersColumn;
use crate::models::users;

#[derive(Default)]
pub struct RootQuery;

#[Object]
impl RootQuery {
    async fn health(&self) -> &str {
        "GraphQL is working!"
    }

    async fn api_version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }

    async fn current_tenant(&self, ctx: &Context<'_>) -> Result<Tenant> {
        let tenant = ctx.data::<TenantContext>()?;
        Ok(Tenant {
            id: tenant.id,
            name: tenant.name.clone(),
            slug: tenant.slug.clone(),
        })
    }

    async fn enabled_modules(&self, ctx: &Context<'_>) -> Result<Vec<String>> {
        let app_ctx = ctx.data::<loco_rs::prelude::AppContext>()?;
        let tenant = ctx.data::<TenantContext>()?;
        let modules = TenantModulesEntity::find_enabled(&app_ctx.db, tenant.id)
            .await
            .map_err(|err| err.to_string())?;

        Ok(modules)
    }

    async fn tenant_modules(&self, ctx: &Context<'_>) -> Result<Vec<TenantModule>> {
        let app_ctx = ctx.data::<loco_rs::prelude::AppContext>()?;
        let tenant = ctx.data::<TenantContext>()?;
        let modules = TenantModulesEntity::find()
            .filter(TenantModulesColumn::TenantId.eq(tenant.id))
            .all(&app_ctx.db)
            .await
            .map_err(|err| err.to_string())?;

        Ok(modules
            .into_iter()
            .map(|module| TenantModule {
                module_slug: module.module_slug,
                enabled: module.enabled,
                settings: module.settings.to_string(),
            })
            .collect())
    }

    async fn me(&self, ctx: &Context<'_>) -> Result<Option<User>> {
        let auth = match ctx.data_opt::<AuthContext>() {
            Some(auth) => auth,
            None => return Ok(None),
        };
        let app_ctx = ctx.data::<loco_rs::prelude::AppContext>()?;
        let tenant = ctx.data::<TenantContext>()?;

        let user = users::Entity::find()
            .filter(UsersColumn::Id.eq(auth.user_id))
            .filter(UsersColumn::TenantId.eq(tenant.id))
            .one(&app_ctx.db)
            .await
            .map_err(|err| err.to_string())?;

        Ok(user.as_ref().map(User::from))
    }

    async fn user(&self, ctx: &Context<'_>, id: uuid::Uuid) -> Result<Option<User>> {
        let auth = ctx
            .data::<AuthContext>()
            .map_err(|_| <FieldError as GraphQLError>::unauthenticated())?;
        let tenant = ctx.data::<TenantContext>()?;
        let app_ctx = ctx.data::<loco_rs::prelude::AppContext>()?;

        if !rustok_core::Rbac::has_permission(&auth.role, &rustok_core::Permission::USERS_READ) {
            return Err(<FieldError as GraphQLError>::permission_denied(
                "Permission denied: users:read required",
            ));
        }

        let user = users::Entity::find_by_id(id)
            .filter(UsersColumn::TenantId.eq(tenant.id))
            .one(&app_ctx.db)
            .await
            .map_err(|err| <FieldError as GraphQLError>::internal_error(&err.to_string()))?;

        Ok(user.as_ref().map(User::from))
    }

    async fn users(
        &self,
        ctx: &Context<'_>,
        #[graphql(default)] pagination: PaginationInput,
    ) -> Result<UserConnection> {
        let auth = ctx
            .data::<AuthContext>()
            .map_err(|_| <FieldError as GraphQLError>::unauthenticated())?;
        let tenant = ctx.data::<TenantContext>()?;
        let app_ctx = ctx.data::<loco_rs::prelude::AppContext>()?;

        if !rustok_core::Rbac::has_permission(&auth.role, &rustok_core::Permission::USERS_LIST) {
            return Err(<FieldError as GraphQLError>::permission_denied(
                "Permission denied: users:list required",
            ));
        }

        let (offset, limit) = pagination.normalize();
        let query = users::Entity::find().filter(UsersColumn::TenantId.eq(tenant.id));
        let total = query
            .clone()
            .count(&app_ctx.db)
            .await
            .map_err(|err| <FieldError as GraphQLError>::internal_error(&err.to_string()))?
            as i64;
        let users = query
            .offset(offset as u64)
            .limit(limit as u64)
            .all(&app_ctx.db)
            .await
            .map_err(|err| <FieldError as GraphQLError>::internal_error(&err.to_string()))?;

        let edges = users
            .iter()
            .enumerate()
            .map(|(index, user)| UserEdge {
                node: User::from(user),
                cursor: encode_cursor(offset + index as i64),
            })
            .collect();

        Ok(UserConnection {
            edges,
            page_info: PageInfo::new(total, offset, limit),
        })
    }
}
