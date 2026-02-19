pub mod dto;
pub mod entities;
pub mod error;
pub mod services;

use async_trait::async_trait;
use rustok_core::module::{HealthStatus, MigrationSource, ModuleKind, RusToKModule};
use sea_orm_migration::MigrationTrait;

pub struct RbacModule;

impl MigrationSource for RbacModule {
    fn migrations(&self) -> Vec<Box<dyn MigrationTrait>> {
        Vec::new()
    }
}

#[async_trait]
impl RusToKModule for RbacModule {
    fn slug(&self) -> &'static str {
        "rbac"
    }

    fn name(&self) -> &'static str {
        "RBAC"
    }

    fn description(&self) -> &'static str {
        "Role-based access control helpers."
    }

    fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    fn kind(&self) -> ModuleKind {
        ModuleKind::Core
    }

    async fn health(&self) -> HealthStatus {
        HealthStatus::Healthy
    }
}
