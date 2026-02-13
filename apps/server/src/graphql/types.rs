use async_graphql::{
    dataloader::DataLoader, ComplexObject, Context, Enum, InputObject, Result, SimpleObject,
};
use rustok_core::{Permission, Rbac, UserRole, UserStatus};
use std::str::FromStr;
use uuid::Uuid;

use crate::graphql::common::PageInfo;
use crate::graphql::loaders::TenantNameLoader;
use crate::models::users;

#[derive(SimpleObject, Clone)]
pub struct Tenant {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
}

#[derive(SimpleObject, Debug, Clone)]
#[graphql(complex)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub name: Option<String>,
    pub role: String,
    pub status: String,
    pub created_at: String,
    #[graphql(skip)]
    pub tenant_id: Uuid,
}

#[derive(Enum, Copy, Clone, Debug, Eq, PartialEq)]
#[graphql(rename_items = "SCREAMING_SNAKE_CASE")]
pub enum GqlUserRole {
    SuperAdmin,
    Admin,
    Manager,
    Customer,
}

impl From<GqlUserRole> for UserRole {
    fn from(role: GqlUserRole) -> Self {
        match role {
            GqlUserRole::SuperAdmin => UserRole::SuperAdmin,
            GqlUserRole::Admin => UserRole::Admin,
            GqlUserRole::Manager => UserRole::Manager,
            GqlUserRole::Customer => UserRole::Customer,
        }
    }
}

#[derive(Enum, Copy, Clone, Debug, Eq, PartialEq)]
#[graphql(rename_items = "SCREAMING_SNAKE_CASE")]
pub enum GqlUserStatus {
    Active,
    Inactive,
    Banned,
}

impl From<GqlUserStatus> for UserStatus {
    fn from(status: GqlUserStatus) -> Self {
        match status {
            GqlUserStatus::Active => UserStatus::Active,
            GqlUserStatus::Inactive => UserStatus::Inactive,
            GqlUserStatus::Banned => UserStatus::Banned,
        }
    }
}

#[derive(InputObject, Debug, Clone)]
pub struct UsersFilter {
    pub role: Option<GqlUserRole>,
    pub status: Option<GqlUserStatus>,
}

#[derive(InputObject, Debug, Clone)]
pub struct CreateUserInput {
    pub email: String,
    pub password: String,
    pub name: Option<String>,
    pub role: Option<GqlUserRole>,
    pub status: Option<GqlUserStatus>,
}

#[derive(InputObject, Debug, Clone)]
pub struct UpdateUserInput {
    pub email: Option<String>,
    pub password: Option<String>,
    pub name: Option<String>,
    pub role: Option<GqlUserRole>,
    pub status: Option<GqlUserStatus>,
}

#[ComplexObject]
impl User {
    async fn display_name(&self) -> String {
        self.name.clone().unwrap_or_else(|| self.email.clone())
    }

    async fn can(&self, _ctx: &Context<'_>, action: String) -> Result<bool> {
        let role = UserRole::from_str(&self.role).map_err(|err| err.to_string())?;
        let permission = Permission::from_str(&action).map_err(|err| err.to_string())?;
        Ok(Rbac::has_permission(&role, &permission))
    }

    async fn tenant_name(&self, ctx: &Context<'_>) -> Result<Option<String>> {
        let loader = ctx.data::<DataLoader<TenantNameLoader>>()?;
        loader.load_one(self.tenant_id).await
    }
}

impl From<&users::Model> for User {
    fn from(model: &users::Model) -> Self {
        Self {
            id: model.id,
            email: model.email.clone(),
            name: model.name.clone(),
            role: model.role.to_string(),
            status: model.status.to_string(),
            created_at: model.created_at.to_rfc3339(),
            tenant_id: model.tenant_id,
        }
    }
}

#[derive(SimpleObject, Clone)]
pub struct TenantModule {
    pub module_slug: String,
    pub enabled: bool,
    pub settings: String,
}

#[derive(SimpleObject, Clone)]
pub struct ModuleRegistryItem {
    pub module_slug: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub enabled: bool,
    pub dependencies: Vec<String>,
}

#[derive(SimpleObject, Debug, Clone)]
pub struct UserEdge {
    pub node: User,
    pub cursor: String,
}

#[derive(SimpleObject, Debug, Clone)]
pub struct UserConnection {
    pub edges: Vec<UserEdge>,
    pub page_info: PageInfo,
}
