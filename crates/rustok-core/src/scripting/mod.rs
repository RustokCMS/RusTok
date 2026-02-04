use std::sync::Arc;

use alloy_scripting::{
    create_default_engine, runner::ScriptExecutor, Scheduler, ScriptEngine, ScriptOrchestrator,
    SeaOrmStorage,
};
use sea_orm::DatabaseConnection;

use crate::Error;

pub struct ScriptingContext {
    pub engine: Arc<ScriptEngine>,
    pub storage: Arc<SeaOrmStorage>,
    pub orchestrator: Arc<ScriptOrchestrator<SeaOrmStorage>>,
    pub scheduler: Arc<Scheduler<SeaOrmStorage>>,
}

impl ScriptingContext {
    pub async fn new(db: DatabaseConnection) -> Result<Self, Error> {
        let engine = Arc::new(create_default_engine());
        let storage = Arc::new(SeaOrmStorage::new(db));
        let orchestrator = Arc::new(ScriptOrchestrator::new(engine.clone(), storage.clone()));
        let executor = ScriptExecutor::new(engine.clone(), storage.clone());
        let scheduler = Arc::new(Scheduler::new(executor, storage.clone()));

        scheduler
            .load_jobs()
            .await
            .map_err(|err| Error::Scripting(err.to_string()))?;

        Ok(Self {
            engine,
            storage,
            orchestrator,
            scheduler,
        })
    }

    pub fn start_scheduler(&self) {
        let scheduler = self.scheduler.clone();
        tokio::spawn(async move {
            scheduler.start().await;
        });
    }
}
