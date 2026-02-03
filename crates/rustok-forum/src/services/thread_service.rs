use chrono::Utc;
use sea_orm::DatabaseConnection;
use uuid::Uuid;

use rustok_core::{generate_id, EventBus, Result};

use crate::dto::{CreateThreadInput, ThreadListItem, ThreadResponse, UpdateThreadInput};

#[derive(Clone)]
pub struct ThreadService {
    db: DatabaseConnection,
    event_bus: EventBus,
}

impl ThreadService {
    pub fn new(db: DatabaseConnection, event_bus: EventBus) -> Self {
        Self { db, event_bus }
    }

    pub async fn list_threads(&self, _tenant_id: Uuid) -> Result<Vec<ThreadListItem>> {
        let _ = &self.db;
        let _ = &self.event_bus;
        Ok(Vec::new())
    }

    pub async fn get_thread(&self, _thread_id: Uuid) -> Result<ThreadResponse> {
        let _ = &self.db;
        let _ = &self.event_bus;
        Ok(ThreadResponse {
            id: generate_id(),
            locale: "en".to_string(),
            title: "Placeholder thread".to_string(),
            body: "Forum thread body placeholder".to_string(),
            created_at: Utc::now(),
        })
    }

    pub async fn create_thread(
        &self,
        _tenant_id: Uuid,
        input: CreateThreadInput,
    ) -> Result<Uuid> {
        let _ = &self.db;
        let _ = &self.event_bus;
        let _ = input;
        Ok(generate_id())
    }

    pub async fn update_thread(
        &self,
        _thread_id: Uuid,
        _input: UpdateThreadInput,
    ) -> Result<()> {
        let _ = &self.db;
        let _ = &self.event_bus;
        Ok(())
    }

    pub async fn delete_thread(&self, _thread_id: Uuid) -> Result<()> {
        let _ = &self.db;
        let _ = &self.event_bus;
        Ok(())
    }
}
