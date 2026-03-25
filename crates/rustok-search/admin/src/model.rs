use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SearchEngineDescriptor {
    pub kind: String,
    pub label: String,
    #[serde(rename = "providedBy")]
    pub provided_by: String,
    pub enabled: bool,
    #[serde(rename = "defaultEngine")]
    pub default_engine: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SearchSettingsPayload {
    #[serde(rename = "tenantId")]
    pub tenant_id: Option<String>,
    #[serde(rename = "activeEngine")]
    pub active_engine: String,
    #[serde(rename = "fallbackEngine")]
    pub fallback_engine: String,
    pub config: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SearchFacetBucket {
    pub value: String,
    pub count: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SearchFacetGroup {
    pub name: String,
    pub buckets: Vec<SearchFacetBucket>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SearchPreviewResultItem {
    pub id: String,
    #[serde(rename = "entityType")]
    pub entity_type: String,
    #[serde(rename = "sourceModule")]
    pub source_module: String,
    pub title: String,
    pub snippet: Option<String>,
    pub score: f64,
    pub locale: Option<String>,
    pub payload: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SearchPreviewPayload {
    pub items: Vec<SearchPreviewResultItem>,
    pub total: u64,
    #[serde(rename = "tookMs")]
    pub took_ms: u64,
    pub engine: String,
    pub facets: Vec<SearchFacetGroup>,
}

#[derive(Clone, Debug)]
pub struct SearchPreviewFilters {
    pub entity_types: Vec<String>,
    pub source_modules: Vec<String>,
    pub statuses: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SearchAdminBootstrap {
    #[serde(rename = "availableSearchEngines")]
    pub available_search_engines: Vec<SearchEngineDescriptor>,
    #[serde(rename = "searchSettingsPreview")]
    pub search_settings_preview: SearchSettingsPayload,
    #[serde(rename = "searchDiagnostics")]
    pub search_diagnostics: SearchDiagnosticsPayload,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SearchDiagnosticsPayload {
    #[serde(rename = "tenantId")]
    pub tenant_id: String,
    #[serde(rename = "totalDocuments")]
    pub total_documents: u64,
    #[serde(rename = "publicDocuments")]
    pub public_documents: u64,
    #[serde(rename = "contentDocuments")]
    pub content_documents: u64,
    #[serde(rename = "productDocuments")]
    pub product_documents: u64,
    #[serde(rename = "staleDocuments")]
    pub stale_documents: u64,
    #[serde(rename = "newestIndexedAt")]
    pub newest_indexed_at: Option<String>,
    #[serde(rename = "oldestIndexedAt")]
    pub oldest_indexed_at: Option<String>,
    #[serde(rename = "maxLagSeconds")]
    pub max_lag_seconds: u64,
    pub state: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TriggerSearchRebuildPayload {
    pub success: bool,
    pub queued: bool,
    #[serde(rename = "tenantId")]
    pub tenant_id: String,
    #[serde(rename = "targetType")]
    pub target_type: String,
    #[serde(rename = "targetId")]
    pub target_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LaggingSearchDocumentPayload {
    #[serde(rename = "documentKey")]
    pub document_key: String,
    #[serde(rename = "documentId")]
    pub document_id: String,
    #[serde(rename = "sourceModule")]
    pub source_module: String,
    #[serde(rename = "entityType")]
    pub entity_type: String,
    pub locale: String,
    pub status: String,
    #[serde(rename = "isPublic")]
    pub is_public: bool,
    pub title: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    #[serde(rename = "indexedAt")]
    pub indexed_at: String,
    #[serde(rename = "lagSeconds")]
    pub lag_seconds: u64,
}
