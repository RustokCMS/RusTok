use async_trait::async_trait;
use sea_orm::DatabaseConnection;
use serde_json::Value;

pub struct ModuleContext<'a> {
    pub db: &'a DatabaseConnection,
    pub tenant_id: uuid::Uuid,
    pub config: &'a Value,
}

#[async_trait]
pub trait RusToKModule: Send + Sync {
    fn slug(&self) -> &'static str;

    fn name(&self) -> &'static str;

    fn description(&self) -> &'static str;

    fn version(&self) -> &'static str;

    fn dependencies(&self) -> &[&'static str] {
        &[]
    }

    async fn on_enable(&self, _ctx: ModuleContext<'_>) -> crate::Result<()> {
        Ok(())
    }

    async fn on_disable(&self, _ctx: ModuleContext<'_>) -> crate::Result<()> {
        Ok(())
    }
}
