use async_graphql::{ComplexObject, Context, Result, SimpleObject};
use rustok_core::{Permission, Rbac, UserRole};
use std::str::FromStr;
use uuid::Uuid;

use crate::graphql::common::PageInfo;
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
