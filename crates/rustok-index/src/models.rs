use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Тип документа в индексе
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DocumentType {
    Node,
    Product,
    Category,
}

impl ToString for DocumentType {
    fn to_string(&self) -> String {
        match self {
            Self::Node => "node".into(),
            Self::Product => "product".into(),
            Self::Category => "category".into(),
        }
    }
}

/// Единый документ для индексации
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexDocument {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub doc_type: DocumentType,
    pub locale: String,

    // Основные поля для поиска
    pub title: String,
    pub slug: String,
    pub content: Option<String>,
    pub keywords: Vec<String>,

    // Сортировка и фильтрация
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub published_at: Option<DateTime<Utc>>,
    pub status: String,
    pub price: Option<i64>,

    // Полный JSON объект
    pub payload: serde_json::Value,
}
