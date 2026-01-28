pub mod auth;
pub mod error;
pub mod id;
pub mod module;
pub mod registry;
pub mod permissions;
pub mod rbac;
pub mod types;
pub use error::{Error, Result};
pub use id::generate_id;
pub use module::ModuleContext;
pub use module::RusToKModule;
pub use registry::ModuleRegistry;
pub use permissions::{Action, Permission, Resource};
pub use rbac::{PermissionScope, Rbac};
pub use types::{UserRole, UserStatus};

pub mod prelude {
    pub use crate::error::{Error, Result};
    pub use crate::id::generate_id;
    pub use crate::permissions::{Action, Permission, Resource};
    pub use crate::rbac::{PermissionScope, Rbac};
    pub use crate::types::{UserRole, UserStatus};
    pub use uuid::Uuid;
}
