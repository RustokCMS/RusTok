use async_trait::async_trait;
use uuid::Uuid;

use crate::models::IndexDocument;
use rustok_core::Error;

#[derive(Debug)]
pub struct SearchQuery {
    pub tenant_id: Uuid,
    pub locale: String,
    pub query: Option<String>,
    pub filters: serde_json::Value,
    pub limit: usize,
    pub offset: usize,
    pub sort: Option<String>,
}

#[derive(Debug, Default)]
pub struct SearchResult {
    pub items: Vec<IndexDocument>,
    pub total: u64,
    pub took_ms: u64,
}

#[async_trait]
pub trait SearchEngine: Send + Sync {
    /// Имя движка (например, "postgres", "meili")
    fn name(&self) -> &str;

    /// Индексация одного документа
    async fn index(&self, doc: IndexDocument) -> Result<(), Error>;

    /// Удаление документа
    async fn delete(&self, id: Uuid, locale: Option<&str>) -> Result<(), Error>;

    /// Удаление всех документов тенанта
    async fn delete_tenant(&self, tenant_id: Uuid) -> Result<(), Error>;

    /// Поиск
    async fn search(&self, query: SearchQuery) -> Result<SearchResult, Error>;
}
