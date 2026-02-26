use async_trait::async_trait;

use crate::error::ScriptResult;
use crate::model::{EventType, Script, ScriptId, ScriptStatus};

pub enum ScriptQuery {
    ById(ScriptId),
    ByName(String),
    ByEvent {
        entity_type: String,
        event: EventType,
    },
    ByApiPath(String),
    Scheduled,
    ByStatus(ScriptStatus),
    All,
}

#[async_trait]
pub trait ScriptRegistry: Send + Sync {
    async fn find(&self, query: ScriptQuery) -> ScriptResult<Vec<Script>>;
    async fn get(&self, id: ScriptId) -> ScriptResult<Script>;
    async fn get_by_name(&self, name: &str) -> ScriptResult<Script>;
    async fn save(&self, script: Script) -> ScriptResult<Script>;
    async fn delete(&self, id: ScriptId) -> ScriptResult<()>;
    async fn set_status(&self, id: ScriptId, status: ScriptStatus) -> ScriptResult<()>;
    async fn record_error(&self, id: ScriptId) -> ScriptResult<bool>;
    async fn reset_errors(&self, id: ScriptId) -> ScriptResult<()>;
}
