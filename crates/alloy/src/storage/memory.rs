use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;

use super::traits::{ScriptPage, ScriptQuery, ScriptRegistry};
use crate::error::{ScriptError, ScriptResult};
use crate::model::{Script, ScriptId, ScriptStatus, ScriptTrigger};

#[derive(Clone)]
pub struct InMemoryStorage {
    scripts: Arc<RwLock<HashMap<ScriptId, Script>>>,
}

impl InMemoryStorage {
    pub fn new() -> Self {
        Self {
            scripts: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for InMemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ScriptRegistry for InMemoryStorage {
    async fn find(&self, query: ScriptQuery) -> ScriptResult<Vec<Script>> {
        let guard = self.scripts.read().await;

        let result = match query {
            ScriptQuery::ById(id) => guard.get(&id).cloned().into_iter().collect(),
            ScriptQuery::ByName(name) => guard
                .values()
                .filter(|script| script.name == name)
                .cloned()
                .collect(),
            ScriptQuery::ByEvent { entity_type, event } => guard
                .values()
                .filter(|script| script.is_executable())
                .filter(|script| {
                    matches!(
                        &script.trigger,
                        ScriptTrigger::Event {
                            entity_type: stored_entity,
                            event: stored_event,
                        } if stored_entity == &entity_type && stored_event == &event
                    )
                })
                .cloned()
                .collect(),
            ScriptQuery::ByApiPath(path) => guard
                .values()
                .filter(|script| script.is_executable())
                .filter(|script| {
                    matches!(
                        &script.trigger,
                        ScriptTrigger::Api { path: stored_path, .. }
                            if stored_path == &path
                    )
                })
                .cloned()
                .collect(),
            ScriptQuery::Scheduled => guard
                .values()
                .filter(|script| script.is_executable())
                .filter(|script| matches!(script.trigger, ScriptTrigger::Cron { .. }))
                .cloned()
                .collect(),
            ScriptQuery::ByStatus(status) => guard
                .values()
                .filter(|script| script.status == status)
                .cloned()
                .collect(),
            ScriptQuery::All => guard.values().cloned().collect(),
        };

        Ok(result)
    }

    async fn find_paginated(
        &self,
        query: ScriptQuery,
        offset: u64,
        limit: u64,
    ) -> ScriptResult<ScriptPage> {
        let all = self.find(query).await?;
        let total = all.len() as u64;
        let items = all
            .into_iter()
            .skip(offset as usize)
            .take(limit as usize)
            .collect();
        Ok(ScriptPage { items, total })
    }

    async fn get(&self, id: ScriptId) -> ScriptResult<Script> {
        let guard = self.scripts.read().await;
        guard.get(&id).cloned().ok_or(ScriptError::NotFound {
            name: id.to_string(),
        })
    }

    async fn get_by_name(&self, name: &str) -> ScriptResult<Script> {
        let guard = self.scripts.read().await;
        guard
            .values()
            .find(|script| script.name == name)
            .cloned()
            .ok_or(ScriptError::NotFound {
                name: name.to_string(),
            })
    }

    async fn save(&self, mut script: Script) -> ScriptResult<Script> {
        let mut guard = self.scripts.write().await;

        if guard.contains_key(&script.id) {
            script.version += 1;
            script.updated_at = chrono::Utc::now();
        }

        guard.insert(script.id, script.clone());
        Ok(script)
    }

    async fn delete(&self, id: ScriptId) -> ScriptResult<()> {
        let mut guard = self.scripts.write().await;
        guard.remove(&id).ok_or(ScriptError::NotFound {
            name: id.to_string(),
        })?;
        Ok(())
    }

    async fn set_status(&self, id: ScriptId, status: ScriptStatus) -> ScriptResult<()> {
        let mut guard = self.scripts.write().await;
        let script = guard.get_mut(&id).ok_or(ScriptError::NotFound {
            name: id.to_string(),
        })?;
        script.status = status;
        script.updated_at = chrono::Utc::now();
        Ok(())
    }

    async fn record_error(&self, id: ScriptId) -> ScriptResult<bool> {
        let mut guard = self.scripts.write().await;
        let script = guard.get_mut(&id).ok_or(ScriptError::NotFound {
            name: id.to_string(),
        })?;

        let should_disable = script.register_error();
        if should_disable {
            script.status = ScriptStatus::Disabled;
        }

        Ok(should_disable)
    }

    async fn reset_errors(&self, id: ScriptId) -> ScriptResult<()> {
        let mut guard = self.scripts.write().await;
        let script = guard.get_mut(&id).ok_or(ScriptError::NotFound {
            name: id.to_string(),
        })?;
        script.reset_errors();
        Ok(())
    }
}
