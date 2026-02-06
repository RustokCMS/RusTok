use async_trait::async_trait;
use sea_orm::DatabaseConnection;
use sea_orm_migration::MigrationTrait;
use serde_json::Value;

use crate::permissions::Permission;

pub struct ModuleContext<'a> {
    pub db: &'a DatabaseConnection,
    pub tenant_id: uuid::Uuid,
    pub config: &'a Value,
}

pub trait EventListener: crate::events::EventHandler {}

pub trait MigrationSource: Send + Sync {
    fn migrations(&self) -> Vec<Box<dyn MigrationTrait>>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

#[async_trait]
pub trait RusToKModule: Send + Sync + MigrationSource {
    fn slug(&self) -> &'static str;

    fn name(&self) -> &'static str;

    fn description(&self) -> &'static str;

    fn version(&self) -> &'static str;

    fn dependencies(&self) -> &[&'static str] {
        &[]
    }

    /// Returns the list of permissions this module declares.
    /// Used for dynamic RBAC permission registration.
    fn permissions(&self) -> Vec<Permission> {
        Vec::new()
    }

    fn event_listeners(&self) -> Vec<Box<dyn EventListener>> {
        Vec::new()
    }

    async fn on_enable(&self, _ctx: ModuleContext<'_>) -> crate::Result<()> {
        Ok(())
    }

    async fn on_disable(&self, _ctx: ModuleContext<'_>) -> crate::Result<()> {
        Ok(())
    }

    async fn health(&self) -> HealthStatus {
        HealthStatus::Healthy
    }
}

