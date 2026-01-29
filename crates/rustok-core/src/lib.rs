pub mod auth;
pub mod error;
pub mod events;
pub mod id;
pub mod migrations;
pub mod module;
pub mod permissions;
pub mod rbac;
pub mod registry;
pub mod types;
pub use error::{Error, Result};
pub use events::{
    DispatcherConfig, DomainEvent, EventBus, EventBusStats, EventDispatcher, EventEnvelope,
    EventHandler, HandlerBuilder, HandlerResult, RunningDispatcher,
};
pub use id::generate_id;
pub use migrations::ModuleMigration;
pub use module::ModuleContext;
pub use module::RusToKModule;
pub use module::{EventListener, MigrationSource};
pub use permissions::{Action, Permission, Resource};
pub use rbac::{PermissionScope, Rbac};
pub use registry::ModuleRegistry;
pub use types::{UserRole, UserStatus};

pub mod prelude {
    pub use crate::error::{Error, Result};
    pub use crate::events::{
        DispatcherConfig, DomainEvent, EventBus, EventBusStats, EventDispatcher, EventEnvelope,
        EventHandler, HandlerBuilder, HandlerResult, RunningDispatcher,
    };
    pub use crate::id::generate_id;
    pub use crate::permissions::{Action, Permission, Resource};
    pub use crate::rbac::{PermissionScope, Rbac};
    pub use crate::types::{UserRole, UserStatus};
    pub use uuid::Uuid;
}
