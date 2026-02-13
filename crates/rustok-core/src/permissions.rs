use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Ресурсы системы
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Resource {
    Users,
    Tenants,
    Modules,
    Settings,
    Products,
    Categories,
    Orders,
    Customers,
    Inventory,
    Discounts,
    Posts,
    Pages,
    Nodes,
    Media,
    Comments,
    Analytics,
    Logs,
    Webhooks,
}

impl fmt::Display for Resource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Users => "users",
            Self::Tenants => "tenants",
            Self::Modules => "modules",
            Self::Settings => "settings",
            Self::Products => "products",
            Self::Categories => "categories",
            Self::Orders => "orders",
            Self::Customers => "customers",
            Self::Inventory => "inventory",
            Self::Discounts => "discounts",
            Self::Posts => "posts",
            Self::Pages => "pages",
            Self::Nodes => "nodes",
            Self::Media => "media",
            Self::Comments => "comments",
            Self::Analytics => "analytics",
            Self::Logs => "logs",
            Self::Webhooks => "webhooks",
        };
        write!(f, "{value}")
    }
}

impl FromStr for Resource {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "users" => Ok(Self::Users),
            "tenants" => Ok(Self::Tenants),
            "modules" => Ok(Self::Modules),
            "settings" => Ok(Self::Settings),
            "products" => Ok(Self::Products),
            "categories" => Ok(Self::Categories),
            "orders" => Ok(Self::Orders),
            "customers" => Ok(Self::Customers),
            "inventory" => Ok(Self::Inventory),
            "discounts" => Ok(Self::Discounts),
            "posts" => Ok(Self::Posts),
            "pages" => Ok(Self::Pages),
            "nodes" => Ok(Self::Nodes),
            "media" => Ok(Self::Media),
            "comments" => Ok(Self::Comments),
            "analytics" => Ok(Self::Analytics),
            "logs" => Ok(Self::Logs),
            "webhooks" => Ok(Self::Webhooks),
            _ => Err(format!("Unknown resource: {value}")),
        }
    }
}

/// Действия над ресурсами
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    Create,
    Read,
    Update,
    Delete,
    List,
    Export,
    Import,
    Manage,
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Create => "create",
            Self::Read => "read",
            Self::Update => "update",
            Self::Delete => "delete",
            Self::List => "list",
            Self::Export => "export",
            Self::Import => "import",
            Self::Manage => "manage",
        };
        write!(f, "{value}")
    }
}

impl FromStr for Action {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "create" => Ok(Self::Create),
            "read" => Ok(Self::Read),
            "update" => Ok(Self::Update),
            "delete" => Ok(Self::Delete),
            "list" => Ok(Self::List),
            "export" => Ok(Self::Export),
            "import" => Ok(Self::Import),
            "manage" | "*" => Ok(Self::Manage),
            _ => Err(format!("Unknown action: {value}")),
        }
    }
}

/// Permission = Resource + Action
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Permission {
    pub resource: Resource,
    pub action: Action,
}

impl Permission {
    pub const fn new(resource: Resource, action: Action) -> Self {
        Self { resource, action }
    }
}

impl FromStr for Permission {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let mut parts = value.split(':');
        let resource = Resource::from_str(parts.next().ok_or("Missing resource")?)?;
        let action = Action::from_str(parts.next().ok_or("Missing action")?)?;
        if parts.next().is_some() {
            return Err("Too many parts in permission string".to_string());
        }
        Ok(Self { resource, action })
    }
}

impl fmt::Display for Permission {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.resource, self.action)
    }
}

impl Permission {
    pub const USERS_CREATE: Self = Self::new(Resource::Users, Action::Create);
    pub const USERS_READ: Self = Self::new(Resource::Users, Action::Read);
    pub const USERS_UPDATE: Self = Self::new(Resource::Users, Action::Update);
    pub const USERS_DELETE: Self = Self::new(Resource::Users, Action::Delete);
    pub const USERS_LIST: Self = Self::new(Resource::Users, Action::List);
    pub const USERS_MANAGE: Self = Self::new(Resource::Users, Action::Manage);

    pub const PRODUCTS_CREATE: Self = Self::new(Resource::Products, Action::Create);
    pub const PRODUCTS_READ: Self = Self::new(Resource::Products, Action::Read);
    pub const PRODUCTS_UPDATE: Self = Self::new(Resource::Products, Action::Update);
    pub const PRODUCTS_DELETE: Self = Self::new(Resource::Products, Action::Delete);
    pub const PRODUCTS_LIST: Self = Self::new(Resource::Products, Action::List);
    pub const PRODUCTS_MANAGE: Self = Self::new(Resource::Products, Action::Manage);

    pub const ORDERS_CREATE: Self = Self::new(Resource::Orders, Action::Create);
    pub const ORDERS_READ: Self = Self::new(Resource::Orders, Action::Read);
    pub const ORDERS_UPDATE: Self = Self::new(Resource::Orders, Action::Update);
    pub const ORDERS_DELETE: Self = Self::new(Resource::Orders, Action::Delete);
    pub const ORDERS_LIST: Self = Self::new(Resource::Orders, Action::List);
    pub const ORDERS_MANAGE: Self = Self::new(Resource::Orders, Action::Manage);

    pub const POSTS_CREATE: Self = Self::new(Resource::Posts, Action::Create);
    pub const POSTS_READ: Self = Self::new(Resource::Posts, Action::Read);
    pub const POSTS_UPDATE: Self = Self::new(Resource::Posts, Action::Update);
    pub const POSTS_DELETE: Self = Self::new(Resource::Posts, Action::Delete);
    pub const POSTS_LIST: Self = Self::new(Resource::Posts, Action::List);
    pub const POSTS_MANAGE: Self = Self::new(Resource::Posts, Action::Manage);

    pub const NODES_CREATE: Self = Self::new(Resource::Nodes, Action::Create);
    pub const NODES_READ: Self = Self::new(Resource::Nodes, Action::Read);
    pub const NODES_UPDATE: Self = Self::new(Resource::Nodes, Action::Update);
    pub const NODES_DELETE: Self = Self::new(Resource::Nodes, Action::Delete);
    pub const NODES_LIST: Self = Self::new(Resource::Nodes, Action::List);
    pub const NODES_MANAGE: Self = Self::new(Resource::Nodes, Action::Manage);

    pub const SETTINGS_READ: Self = Self::new(Resource::Settings, Action::Read);
    pub const SETTINGS_UPDATE: Self = Self::new(Resource::Settings, Action::Update);
    pub const SETTINGS_MANAGE: Self = Self::new(Resource::Settings, Action::Manage);

    pub const ANALYTICS_READ: Self = Self::new(Resource::Analytics, Action::Read);
    pub const ANALYTICS_EXPORT: Self = Self::new(Resource::Analytics, Action::Export);
}
